//! An ONNX model whose big matrix weights stay 8-bit on the GPU: every float32 weight that only
//! multiplies, through a `MatMul` (as its second input) or a 1x1 `Conv` (a pointwise one: one
//! group, no stride, dilation or padding), is stored as unsigned 8-bit integers with a float32
//! scale per block of 32 along its input axis (symmetric, `round(w / scale) + 128`, `scale = max
//! |w| / 127` over the block, itself rounded to float16 and stored so, the cast folded when the
//! session starts; rounded half to even; the inputs padded to whole blocks with zeros),
//! and read by ONNX Runtime's `MatMulNBits` (`com.microsoft`, 8 bits), which multiplies by them as
//! they are: the GPU holds a quarter of the bytes and dequantizes as it multiplies. (Blocks of 32:
//! ONNX Runtime's WebGPU kernel has a fast path for them; 64 and 128 ran half as fast.) A pointwise
//! `Conv(x, w, b)` on `[batch, channels, length]` becomes `Transpose(MatMulNBits(Transpose(x)) +
//! b)`; ONNX Runtime cancels the transposes against its own around the layer norms.
//!
//! Every other big weight is stored as `int8.rs` stores it (8-bit with a cast and multiply in
//! front, folded to float32 when the session starts; float16 for one axis). Works on the protobuf
//! wire format like `fp16.rs`, deterministic so the output can be pinned by its SHA-256. Used by
//! `cargo fetch-voice`.

use crate::fp16::{
    FLOAT, MIN_VALUES, bytes_field, fields, read_tensor, round_to_half, string_field, varint,
    varint_field,
};
use crate::int8::{Int8, tensor, weights_to_int8};
use std::collections::HashMap;

/// Values per block, each with its own scale.
const BLOCK: usize = 32;
/// The zero `MatMulNBits` assumes for 8 bits.
const ZERO: f32 = 128.0;
const UINT8: u64 = 2;
/// `AttributeProto.type`.
const ATTR_INT: u64 = 2;
const ATTR_INTS: u64 = 7;

/// What converting a model did.
pub struct Nbits {
    pub model: Vec<u8>,
    /// Weights read by `MatMulNBits`; others stored by `int8.rs` as int8 and as float16.
    pub nbits: usize,
    pub int8: usize,
    pub fp16: usize,
}

/// A node: its op type, inputs, outputs and attributes (raw `AttributeProto`s by name).
struct Node<'a> {
    op: String,
    domain: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    attributes: HashMap<String, &'a [u8]>,
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

fn node_of(buf: &[u8]) -> Result<Node<'_>, String> {
    let mut n = Node {
        op: String::new(),
        domain: String::new(),
        inputs: Vec::new(),
        outputs: Vec::new(),
        attributes: HashMap::new(),
    };
    for f in fields(buf)? {
        match f.number {
            1 => n.inputs.push(text(f.payload)),
            2 => n.outputs.push(text(f.payload)),
            4 => n.op = text(f.payload),
            5 => {
                let name = fields(f.payload)?
                    .iter()
                    .find(|a| a.number == 1)
                    .map(|a| text(a.payload))
                    .unwrap_or_default();
                n.attributes.insert(name, f.payload);
            }
            7 => n.domain = text(f.payload),
            _ => {}
        }
    }
    Ok(n)
}

/// An attribute's integers (`i` or `ints`).
fn ints(attribute: &[u8]) -> Result<Vec<i64>, String> {
    let mut out = Vec::new();
    for f in fields(attribute)? {
        match (f.number, f.wire) {
            (3, 0) | (8, 0) => out.push(f.varint as i64),
            (8, 2) => {
                let mut at = 0;
                while at < f.payload.len() {
                    out.push(varint(f.payload, &mut at)? as i64);
                }
            }
            _ => {}
        }
    }
    Ok(out)
}

/// A 1x1 `Conv` that is a plain matrix product: one group, no stride, dilation or padding.
fn pointwise(n: &Node) -> Result<bool, String> {
    let all = |name: &str, ok: &dyn Fn(i64) -> bool| -> Result<bool, String> {
        Ok(match n.attributes.get(name) {
            Some(a) => ints(a)?.iter().all(|&v| ok(v)),
            None => true,
        })
    };
    let auto_pad = n.attributes.get("auto_pad").map(|a| {
        fields(a)
            .ok()
            .and_then(|f| f.iter().find(|x| x.number == 4).map(|x| text(x.payload)))
            .unwrap_or_default()
    });
    Ok(n.op == "Conv"
        && n.domain.is_empty()
        && all("group", &|v| v == 1)?
        && all("strides", &|v| v == 1)?
        && all("dilations", &|v| v == 1)?
        && all("pads", &|v| v == 0)?
        && all("kernel_shape", &|v| v == 1)?
        && auto_pad.is_none_or(|p| p.is_empty() || p == "NOTSET"))
}

/// `w(n, k)` (`n` outputs, `k` inputs) as `MatMulNBits` stores it: `[n][blocks][BLOCK]` bytes and
/// `[n][blocks]` scales.
fn quantize(n: usize, k: usize, w: impl Fn(usize, usize) -> f32) -> (Vec<u8>, Vec<f32>) {
    let blocks = k.div_ceil(BLOCK);
    let mut bytes = Vec::with_capacity(n * blocks * BLOCK);
    let mut scales = Vec::with_capacity(n * blocks);
    for row in 0..n {
        for b in 0..blocks {
            let span = b * BLOCK..((b + 1) * BLOCK).min(k);
            let max = span.clone().map(|i| w(row, i).abs()).fold(0f32, f32::max);
            // As float16 holds it (`int8.rs` stores the scales so); one too small for it, a block of
            // zeros in effect, gets 1.
            let scale = round_to_half(max / 127.0);
            let scale = if scale > 0.0 { scale } else { 1.0 };
            scales.push(scale);
            for i in b * BLOCK..(b + 1) * BLOCK {
                let q = if i < k {
                    (w(row, i) / scale).round_ties_even().clamp(-127.0, 127.0)
                } else {
                    0.0
                };
                bytes.push((q + ZERO) as u8);
            }
        }
    }
    (bytes, scales)
}

fn int_attribute(name: &str, value: i64) -> Vec<u8> {
    let mut a = string_field(1, name);
    a.extend_from_slice(&varint_field(3, value as u64));
    a.extend_from_slice(&varint_field(20, ATTR_INT));
    a
}

fn ints_attribute(name: &str, values: &[i64]) -> Vec<u8> {
    let mut a = string_field(1, name);
    for &v in values {
        a.extend_from_slice(&varint_field(8, v as u64));
    }
    a.extend_from_slice(&varint_field(20, ATTR_INTS));
    a
}

fn node(
    op: &str,
    domain: &str,
    name: &str,
    inputs: &[&str],
    output: &str,
    attributes: &[Vec<u8>],
) -> Vec<u8> {
    let mut out = Vec::new();
    for i in inputs {
        out.extend_from_slice(&string_field(1, i));
    }
    out.extend_from_slice(&string_field(2, output));
    out.extend_from_slice(&string_field(3, name));
    out.extend_from_slice(&string_field(4, op));
    for a in attributes {
        out.extend_from_slice(&bytes_field(5, a));
    }
    if !domain.is_empty() {
        out.extend_from_slice(&string_field(7, domain));
    }
    bytes_field(1, &out)
}

/// A weight `MatMulNBits` reads: its stored names and sizes.
struct Packed {
    bytes: String,
    scales: String,
    k: usize,
    n: usize,
}

/// The model with its matrix weights read by 8-bit `MatMulNBits` and the rest as `int8.rs` has them.
pub fn weights_to_nbits8(model: &[u8]) -> Result<Nbits, String> {
    let top = fields(model)?;
    let graph_field = top.iter().find(|f| f.number == 7).ok_or("ONNX: no graph")?;
    let parts = fields(graph_field.payload)?;
    let nodes: Vec<Node> = parts
        .iter()
        .filter(|f| f.number == 1)
        .map(|f| node_of(f.payload))
        .collect::<Result<_, _>>()?;

    // Which weights every reader multiplies by: `MatMul` second input (`[k, n]`), or a pointwise
    // `Conv` weight (`[n, k, 1]`).
    let mut readers: HashMap<&str, Vec<&Node>> = HashMap::new();
    for n in &nodes {
        for i in &n.inputs {
            readers.entry(i.as_str()).or_default().push(n);
        }
    }
    let mut packed: HashMap<String, Packed> = HashMap::new();
    let mut graph: Vec<Vec<u8>> = Vec::new();
    for f in &parts {
        if f.number != 5 {
            continue;
        }
        let t = read_tensor(f.payload)?;
        let Some(values) = t
            .values
            .as_ref()
            .filter(|v| t.ty as u64 == FLOAT && v.len() >= MIN_VALUES)
        else {
            continue;
        };
        let dims = crate::int8::dims(f.payload)?;
        let rs = readers
            .get(t.name.as_str())
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if rs.is_empty() {
            continue;
        }
        let matmul = dims.len() == 2
            && rs.iter().all(|n| {
                n.op == "MatMul"
                    && n.domain.is_empty()
                    && n.inputs.get(1) == Some(&t.name)
                    && n.inputs[0] != t.name
            });
        let conv = dims.len() == 3 && dims[2] == 1 && {
            let mut ok = true;
            for n in rs {
                ok &= pointwise(n)?
                    && n.inputs.get(1) == Some(&t.name)
                    && n.inputs[0] != t.name
                    && n.inputs.get(2) != Some(&t.name);
            }
            ok
        };
        let (k, n, (bytes, scales)) = if matmul {
            let (k, n) = (dims[0] as usize, dims[1] as usize);
            (k, n, quantize(n, k, |row, i| values[i * n + row]))
        } else if conv {
            let (n, k) = (dims[0] as usize, dims[1] as usize);
            (k, n, quantize(n, k, |row, i| values[row * k + i]))
        } else {
            continue;
        };
        let blocks = k.div_ceil(BLOCK) as u64;
        let p = Packed {
            bytes: format!("{}__nb8", t.name),
            scales: format!("{}__nb8s", t.name),
            k,
            n,
        };
        let raw: Vec<u8> = scales.iter().flat_map(|x| x.to_le_bytes()).collect();
        graph.push(bytes_field(
            5,
            &tensor(&p.bytes, UINT8, &[n as u64, blocks, BLOCK as u64], &bytes),
        ));
        // One axis: `int8.rs` stores them as float16.
        graph.push(bytes_field(
            5,
            &tensor(&p.scales, FLOAT, &[n as u64 * blocks], &raw),
        ));
        packed.insert(t.name.clone(), p);
    }

    let mut out_nodes = 0;
    let mut node_index = 0;
    let mut rest: Vec<Vec<u8>> = Vec::new();
    for f in &parts {
        match f.number {
            5 => {
                let name = read_tensor(f.payload)?.name;
                if !packed.contains_key(&name) {
                    rest.push(f.bytes.to_vec());
                }
            }
            1 => {
                let n = &nodes[node_index];
                node_index += 1;
                let Some(p) = n.inputs.get(1).and_then(|w| packed.get(w)) else {
                    rest.push(f.bytes.to_vec());
                    continue;
                };
                out_nodes += 1;
                let id = format!("nb8_{out_nodes}");
                let attrs = [
                    int_attribute("K", p.k as i64),
                    int_attribute("N", p.n as i64),
                    int_attribute("bits", 8),
                    int_attribute("block_size", BLOCK as i64),
                ];
                let y = &n.outputs[0];
                if n.op == "MatMul" {
                    rest.push(node(
                        "MatMulNBits",
                        "com.microsoft",
                        &id,
                        &[&n.inputs[0], &p.bytes, &p.scales],
                        y,
                        &attrs,
                    ));
                    continue;
                }
                // Conv on [batch, channels, length]: channels last, multiply, bias, back.
                let perm = [ints_attribute("perm", &[0, 2, 1])];
                let (xt, mm, biased) =
                    (format!("{id}_in"), format!("{id}_mm"), format!("{id}_bias"));
                rest.push(node(
                    "Transpose",
                    "",
                    &format!("{id}_t0"),
                    &[&n.inputs[0]],
                    &xt,
                    &perm,
                ));
                rest.push(node(
                    "MatMulNBits",
                    "com.microsoft",
                    &id,
                    &[&xt, &p.bytes, &p.scales],
                    &mm,
                    &attrs,
                ));
                let last = match n.inputs.get(2).filter(|b| !b.is_empty()) {
                    Some(b) => {
                        rest.push(node(
                            "Add",
                            "",
                            &format!("{id}_add"),
                            &[&mm, b],
                            &biased,
                            &[],
                        ));
                        biased
                    }
                    None => mm,
                };
                rest.push(node(
                    "Transpose",
                    "",
                    &format!("{id}_t1"),
                    &[&last],
                    y,
                    &perm,
                ));
            }
            _ => rest.push(f.bytes.to_vec()),
        }
    }
    let nbits = packed.len();
    // The packed weights first, then the graph as it was (nodes in the order they run).
    graph.extend(rest);
    let graph: Vec<u8> = graph.into_iter().flatten().collect();

    let mut out = Vec::new();
    let mut has_ms = false;
    for f in &top {
        if f.number == 8 {
            has_ms |= fields(f.payload)?
                .iter()
                .any(|x| x.number == 1 && x.payload == b"com.microsoft");
        }
    }
    for f in &top {
        if std::ptr::eq(f, graph_field) {
            out.extend_from_slice(&bytes_field(7, &graph));
        } else {
            out.extend_from_slice(f.bytes);
        }
    }
    if !has_ms {
        let mut opset = string_field(1, "com.microsoft");
        opset.extend_from_slice(&varint_field(2, 1));
        out.extend_from_slice(&bytes_field(8, &opset));
    }
    let Int8 {
        model, int8, fp16, ..
    } = weights_to_int8(&out)?;
    Ok(Nbits {
        model,
        nbits,
        int8,
        fp16,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init(name: &str, dims: &[u64], values: &[f32]) -> Vec<u8> {
        let raw: Vec<u8> = values.iter().flat_map(|x| x.to_le_bytes()).collect();
        bytes_field(5, &tensor(name, FLOAT, dims, &raw))
    }

    /// A model: the nodes (already `bytes_field(1, …)`) and float32 initializers.
    fn model(nodes: &[Vec<u8>], weights: &[(&str, Vec<u64>, Vec<f32>)]) -> Vec<u8> {
        let mut g: Vec<u8> = nodes.concat();
        for (name, d, v) in weights {
            g.extend_from_slice(&init(name, d, v));
        }
        let mut m = varint_field(1, 8);
        m.extend_from_slice(&bytes_field(7, &g));
        m
    }

    /// The nodes' (domain, op, inputs, outputs) in order, the initializers by name (type, dims,
    /// raw), and the model's opset domains.
    #[allow(clippy::type_complexity)]
    fn parse(
        m: &[u8],
    ) -> (
        Vec<(String, String, Vec<String>, Vec<String>)>,
        HashMap<String, (u64, Vec<u64>, Vec<u8>)>,
        Vec<String>,
    ) {
        let top = fields(m).unwrap();
        let g = top.iter().find(|f| f.number == 7).unwrap();
        let (mut nodes, mut inits) = (Vec::new(), HashMap::new());
        for f in fields(g.payload).unwrap() {
            match f.number {
                1 => {
                    let n = node_of(f.payload).unwrap();
                    nodes.push((n.domain, n.op, n.inputs, n.outputs));
                }
                5 => {
                    let inner = fields(f.payload).unwrap();
                    let get = |n: u64| inner.iter().find(|x| x.number == n).unwrap();
                    inits.insert(
                        text(get(8).payload),
                        (
                            get(2).varint,
                            crate::int8::dims(f.payload).unwrap(),
                            get(9).payload.to_vec(),
                        ),
                    );
                }
                _ => {}
            }
        }
        let domains = top
            .iter()
            .filter(|f| f.number == 8)
            .map(|f| {
                fields(f.payload)
                    .unwrap()
                    .iter()
                    .find(|x| x.number == 1)
                    .map(|x| text(x.payload))
                    .unwrap_or_default()
            })
            .collect();
        (nodes, inits, domains)
    }

    /// Output `o` of `n` has values up to `o + 1`, so blocks have their own scales.
    fn weight(n: usize, k: usize) -> Vec<f32> {
        (0..n * k)
            .map(|i| ((i / k) as f32 + 1.0) * ((i as f32 * 0.37).sin()))
            .collect()
    }

    /// `w(n, k)` back from the packed bytes and the scales (float16, or float32 if few).
    fn dequantize(bytes: &[u8], scales: &[u8], half: bool, n: usize, k: usize) -> Vec<f32> {
        let blocks = k.div_ceil(BLOCK);
        let scales: Vec<f32> = if half {
            scales
                .as_chunks::<2>()
                .0
                .iter()
                .map(|h| {
                    let h = u16::from_le_bytes(*h);
                    let (e, m) = (i32::from((h >> 10) & 0x1f), f32::from(h & 0x3ff));
                    if e == 0 {
                        m * 2f32.powi(-24)
                    } else {
                        (1.0 + m / 1024.0) * 2f32.powi(e - 15)
                    }
                })
                .collect()
        } else {
            scales
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| f32::from_le_bytes(*c))
                .collect()
        };
        (0..n * k)
            .map(|i| {
                let (row, col) = (i / k, i % k);
                let at = row * blocks * BLOCK + col;
                (f32::from(bytes[at]) - ZERO) * scales[row * blocks + col / BLOCK]
            })
            .collect()
    }

    #[test]
    fn a_pointwise_conv_becomes_matmulnbits_between_transposes() {
        // 32 outputs, 1000 inputs: 32 blocks, the last padded (1,024 scales: float16).
        let (n, k) = (32, 1000);
        let w = weight(n, k);
        let conv = node(
            "Conv",
            "",
            "c",
            &["x", "w", "b"],
            "y",
            &[ints_attribute("kernel_shape", &[1])],
        );
        let m = model(
            &[conv],
            &[
                ("w", vec![n as u64, k as u64, 1], w.clone()),
                ("b", vec![n as u64], vec![0.5; n]),
            ],
        );
        let out = weights_to_nbits8(&m).unwrap();
        assert_eq!((out.nbits, out.int8), (1, 0));
        let (nodes, inits, domains) = parse(&out.model);
        let ops: Vec<&str> = nodes.iter().map(|x| x.1.as_str()).collect();
        // The float16 scales' cast first, then the conv's replacement, its output the same name.
        assert_eq!(
            ops,
            ["Cast", "Transpose", "MatMulNBits", "Add", "Transpose"]
        );
        assert_eq!(nodes[2].0, "com.microsoft");
        assert_eq!(nodes[2].2, ["nb8_1_in", "w__nb8", "w__nb8s"]);
        assert_eq!(nodes[3].2[1], "b");
        assert_eq!(nodes[4].3, ["y"]);
        assert!(domains.contains(&"com.microsoft".to_string()));
        let (ty, d, bytes) = &inits["w__nb8"];
        assert_eq!((*ty, d.as_slice()), (UINT8, &[32u64, 32, 32][..]));
        // Padding is the zero.
        assert!(bytes[31 * 32 + 8..32 * 32].iter().all(|&b| b == 128));
        let (sty, sd, scales) = &inits["w__nb8s__fp16"];
        assert_eq!((*sty, sd.as_slice()), (10, &[1024u64][..]));
        let back = dequantize(bytes, scales, true, n, k);
        for (i, (a, b)) in w.iter().zip(&back).enumerate() {
            let block_max = (i / k) as f32 + 1.0;
            assert!((a - b).abs() <= block_max / 127.0 * 0.51, "{i}: {a} -> {b}");
        }
        assert!(!inits.contains_key("w"));
    }

    #[test]
    fn a_matmul_weight_is_read_by_matmulnbits_per_output_column() {
        // 64 inputs, 32 outputs (64 scales: they stay float32, already rounded as float16 would).
        let (k, n) = (64, 32);
        // MatMul's weight is [k, n]: column `o` holds output `o`.
        let wt = weight(n, k);
        let w: Vec<f32> = (0..k * n).map(|i| wt[(i % n) * k + i / n]).collect();
        let mm = node("MatMul", "", "m", &["x", "w"], "y", &[]);
        let out = weights_to_nbits8(&model(&[mm], &[("w", vec![k as u64, n as u64], w)])).unwrap();
        let (nodes, inits, _) = parse(&out.model);
        assert_eq!(nodes.last().unwrap().1, "MatMulNBits");
        assert_eq!(nodes.last().unwrap().3, ["y"]);
        let (sty, _, scales) = &inits["w__nb8s"];
        assert_eq!(*sty, FLOAT);
        let back = dequantize(&inits["w__nb8"].2, scales, false, n, k);
        for (i, (a, b)) in wt.iter().zip(&back).enumerate() {
            assert!(
                (a - b).abs() <= ((i / k) as f32 + 1.0) / 127.0 * 0.51,
                "{i}: {a} -> {b}"
            );
        }
    }

    #[test]
    fn other_weights_go_as_int8_stores_them_and_it_is_deterministic() {
        // A strided conv and a gather: not plain matrix products.
        let conv = node(
            "Conv",
            "",
            "c",
            &["x", "w"],
            "y",
            &[ints_attribute("strides", &[2])],
        );
        let gather = node("Gather", "", "g", &["e", "i"], "z", &[]);
        let weights = [
            ("w", vec![8, 128, 1], weight(8, 128)),
            ("e", vec![16, 64], weight(16, 64)),
        ];
        let out = weights_to_nbits8(&model(&[conv.clone(), gather.clone()], &weights)).unwrap();
        assert_eq!((out.nbits, out.int8), (0, 2));
        let again = weights_to_nbits8(&model(&[conv, gather], &weights)).unwrap();
        assert_eq!(again.model, out.model);
    }
}
