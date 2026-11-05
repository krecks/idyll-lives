//! An ONNX model with its big weights stored as 8-bit integers, one float32 scale per channel:
//! every float32 weight of 1,024 values or more with two or more axes becomes int8 (symmetric,
//! `round(w / scale)` with `scale = max |w| / 127` over its channel, rounded half to even), and
//! `Cast` to float32 then `Mul` by its scales are put in front of the graph under the weight's own
//! name. Every operator still sees float32, and ONNX Runtime folds the cast and the multiply once
//! when the session starts, so the GPU does float32 arithmetic as before, on weights a little
//! rounder: a quarter of the download, the same speed and memory. (Not int8 operators: ONNX Runtime
//! Web runs those on the CPU.) Big weights with one axis (biases, norms) are stored as float16 as
//! `fp16.rs` does.
//!
//! The channel is the output's: the first axis (a convolution's output channels, an embedding's
//! rows), but the last for a two-axis weight that only `MatMul` multiplies by (`[in, out]`).
//!
//! Works on the protobuf wire format like `fp16.rs`; deterministic, so the output can be pinned by
//! its SHA-256. Used by `cargo fetch-voice`.

use crate::fp16::{
    FLOAT, MIN_VALUES, bytes_field, fields, half_tensor, read_tensor, string_field, to_half,
    varint_field,
};
use std::collections::HashMap;

const INT8: u64 = 3;
/// `AttributeProto.type` INT.
const ATTR_INT: u64 = 2;

/// What converting a model did.
pub struct Int8 {
    pub model: Vec<u8>,
    /// Weights stored as int8, and as float16.
    pub int8: usize,
    pub fp16: usize,
    /// Bytes saved against float32.
    pub saved: usize,
}

/// A `TensorProto`'s dims.
fn dims(buf: &[u8]) -> Result<Vec<u64>, String> {
    let mut out = Vec::new();
    for f in fields(buf)? {
        match (f.number, f.wire) {
            (1, 0) => out.push(f.varint),
            (1, 2) => {
                let mut at = 0;
                while at < f.payload.len() {
                    let (mut v, mut shift) = (0u64, 0);
                    loop {
                        let b = *f.payload.get(at).ok_or("protobuf: truncated dims")?;
                        at += 1;
                        v |= u64::from(b & 0x7f) << shift;
                        if b < 0x80 {
                            break;
                        }
                        shift += 7;
                    }
                    out.push(v);
                }
            }
            _ => {}
        }
    }
    Ok(out)
}

/// The op types each value is read by.
fn readers(graph: &[u8]) -> Result<HashMap<String, Vec<String>>, String> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for f in fields(graph)? {
        if f.number != 1 {
            continue;
        }
        let node = fields(f.payload)?;
        let op = node
            .iter()
            .find(|n| n.number == 4)
            .map(|n| String::from_utf8_lossy(n.payload).into_owned())
            .unwrap_or_default();
        for n in node.iter().filter(|n| n.number == 1) {
            out.entry(String::from_utf8_lossy(n.payload).into_owned())
                .or_default()
                .push(op.clone());
        }
    }
    Ok(out)
}

/// `values` (shaped `dims`) as int8 per channel along `axis`: the bytes, and the channels' scales.
fn quantize(values: &[f32], dims: &[u64], axis: usize) -> (Vec<u8>, Vec<f32>) {
    let channels = dims[axis] as usize;
    let inner: usize = dims[axis + 1..].iter().product::<u64>() as usize;
    let channel = |i: usize| (i / inner) % channels;
    let mut max = vec![0f32; channels];
    for (i, v) in values.iter().enumerate() {
        let c = channel(i);
        max[c] = max[c].max(v.abs());
    }
    let scales: Vec<f32> = max
        .iter()
        .map(|&m| if m > 0.0 { m / 127.0 } else { 1.0 })
        .collect();
    let bytes = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            (v / scales[channel(i)])
                .round_ties_even()
                .clamp(-127.0, 127.0) as i8 as u8
        })
        .collect();
    (bytes, scales)
}

fn tensor(name: &str, ty: u64, dims: &[u64], raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for &d in dims {
        out.extend_from_slice(&varint_field(1, d));
    }
    out.extend_from_slice(&varint_field(2, ty));
    out.extend_from_slice(&string_field(8, name));
    out.extend_from_slice(&bytes_field(9, raw));
    out
}

fn node(op: &str, inputs: &[&str], output: &str, attribute: Option<Vec<u8>>) -> Vec<u8> {
    let mut out = Vec::new();
    for i in inputs {
        out.extend_from_slice(&string_field(1, i));
    }
    out.extend_from_slice(&string_field(2, output));
    out.extend_from_slice(&string_field(
        3,
        &format!("int8_{}_{output}", op.to_lowercase()),
    ));
    out.extend_from_slice(&string_field(4, op));
    if let Some(a) = attribute {
        out.extend_from_slice(&bytes_field(5, &a));
    }
    out
}

/// The model with its big float32 weights stored as int8 (two or more axes) or float16 (one).
pub fn weights_to_int8(model: &[u8]) -> Result<Int8, String> {
    let top = fields(model)?;
    let graph_field = top.iter().find(|f| f.number == 7).ok_or("ONNX: no graph")?;
    let readers = readers(graph_field.payload)?;
    let mut front: Vec<Vec<u8>> = Vec::new();
    let mut parts: Vec<Vec<u8>> = Vec::new();
    let (mut int8, mut fp16, mut saved) = (0, 0, 0);
    for f in fields(graph_field.payload)? {
        if f.number == 5 {
            let t = read_tensor(f.payload)?;
            if let Some(v) = t
                .values
                .as_ref()
                .filter(|v| t.ty as u64 == FLOAT && v.len() >= MIN_VALUES)
            {
                let d = dims(f.payload)?;
                if d.len() >= 2 && d.iter().product::<u64>() as usize == v.len() {
                    let matmul = readers
                        .get(&t.name)
                        .is_some_and(|ops| ops.iter().all(|o| o == "MatMul"));
                    let axis = if d.len() == 2 && matmul { 1 } else { 0 };
                    let (bytes, scales) = quantize(v, &d, axis);
                    let mut scale_dims = vec![1; d.len()];
                    scale_dims[axis] = d[axis];
                    let (q, s, qf) = (
                        format!("{}__q8", t.name),
                        format!("{}__q8s", t.name),
                        format!("{}__q8f", t.name),
                    );
                    let raw: Vec<u8> = scales.iter().flat_map(|x| x.to_le_bytes()).collect();
                    parts.push(bytes_field(5, &tensor(&q, INT8, &d, &bytes)));
                    parts.push(bytes_field(5, &tensor(&s, FLOAT, &scale_dims, &raw)));
                    let mut to = string_field(1, "to");
                    to.extend_from_slice(&varint_field(3, FLOAT));
                    to.extend_from_slice(&varint_field(20, ATTR_INT));
                    front.push(bytes_field(1, &node("Cast", &[&q], &qf, Some(to))));
                    front.push(bytes_field(1, &node("Mul", &[&qf, &s], &t.name, None)));
                    int8 += 1;
                    saved += v.len() * 4 - bytes.len() - raw.len();
                    continue;
                }
                if let Some(half) = to_half(v) {
                    let name = format!("{}__fp16", t.name);
                    parts.push(bytes_field(5, &half_tensor(f.payload, &name, &half)?));
                    front.push(bytes_field(1, &crate::fp16::cast_node(&name, &t.name)));
                    fp16 += 1;
                    saved += v.len() * 2;
                    continue;
                }
            }
        }
        parts.push(f.bytes.to_vec());
    }
    // The dequantizing nodes before every other node (nodes are listed in the order they run).
    let graph: Vec<u8> = front.into_iter().chain(parts).flatten().collect();
    let mut out = Vec::new();
    for f in &top {
        if std::ptr::eq(f, graph_field) {
            out.extend_from_slice(&bytes_field(7, &graph));
        } else {
            out.extend_from_slice(f.bytes);
        }
    }
    Ok(Int8 {
        model: out,
        int8,
        fp16,
        saved,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A graph with float32 initializers (name, dims, values) and one node `op` reading `w`.
    fn model(op: &str, weights: &[(&str, Vec<u64>, Vec<f32>)]) -> Vec<u8> {
        let mut g = bytes_field(1, &node(op, &["x", "w"], "y", None));
        for (name, d, v) in weights {
            let raw: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
            g.extend_from_slice(&bytes_field(5, &tensor(name, FLOAT, d, &raw)));
        }
        let mut m = varint_field(1, 8);
        m.extend_from_slice(&bytes_field(7, &g));
        m
    }

    /// The initializers by name: (type, dims, raw bytes); and the nodes' op types in order.
    /// An initializer: type, dims, raw bytes.
    type Init = (u64, Vec<u64>, Vec<u8>);

    fn parse(m: &[u8]) -> (HashMap<String, Init>, Vec<String>) {
        let top = fields(m).unwrap();
        let g = top.iter().find(|f| f.number == 7).unwrap();
        let (mut inits, mut ops) = (HashMap::new(), Vec::new());
        for f in fields(g.payload).unwrap() {
            let inner = fields(f.payload).unwrap();
            let get = |n: u64| inner.iter().find(|x| x.number == n);
            match f.number {
                5 => {
                    let name = String::from_utf8_lossy(get(8).unwrap().payload).into_owned();
                    inits.insert(
                        name,
                        (
                            get(2).unwrap().varint,
                            dims(f.payload).unwrap(),
                            get(9).unwrap().payload.to_vec(),
                        ),
                    );
                }
                1 => ops.push(String::from_utf8_lossy(get(4).unwrap().payload).into_owned()),
                _ => {}
            }
        }
        (inits, ops)
    }

    fn floats(raw: &[u8]) -> Vec<f32> {
        raw.as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect()
    }

    /// Channel `c` of 4 holds values up to `c + 1`, so each channel has its own scale.
    fn weight(rows: usize, cols: usize, by_row: bool) -> Vec<f32> {
        (0..rows * cols)
            .map(|i| {
                let (r, c) = (i / cols, i % cols);
                let (ch, k) = if by_row { (r, c) } else { (c, r) };
                (ch as f32 + 1.0) * ((k as f32 * 0.37).sin())
            })
            .collect()
    }

    #[test]
    fn conv_weights_go_int8_per_output_channel_and_come_back_close() {
        let w = weight(4, 256, true);
        let out = weights_to_int8(&model("Conv", &[("w", vec![4, 256], w.clone())])).unwrap();
        assert_eq!((out.int8, out.fp16), (1, 0));
        let (inits, ops) = parse(&out.model);
        assert_eq!(ops, ["Cast", "Mul", "Conv"]);
        let (ty, d, q) = &inits["w__q8"];
        assert_eq!((*ty, d.as_slice()), (INT8, &[4u64, 256][..]));
        let (_, sd, s) = &inits["w__q8s"];
        assert_eq!(sd.as_slice(), &[4, 1]);
        let s = floats(s);
        for (i, v) in w.iter().enumerate() {
            let back = (q[i] as i8) as f32 * s[i / 256];
            assert!(
                (back - v).abs() <= s[i / 256] / 2.0 + 1e-6,
                "{i}: {v} -> {back}"
            );
        }
        // The largest value of a channel is exactly ±127.
        assert!(q.iter().any(|&b| b as i8 == 127 || b as i8 == -127));
        assert!(!inits.contains_key("w"));
    }

    #[test]
    fn matmul_weights_go_per_column() {
        let w = weight(256, 4, false);
        let out = weights_to_int8(&model("MatMul", &[("w", vec![256, 4], w)])).unwrap();
        let (inits, _) = parse(&out.model);
        assert_eq!(inits["w__q8s"].1.as_slice(), &[1, 4]);
    }

    #[test]
    fn one_axis_weights_go_float16_and_small_ones_stay() {
        let big: Vec<f32> = (0..1024).map(|i| i as f32 * 0.5).collect();
        let small: Vec<f32> = (0..1023).map(|i| i as f32).collect();
        let out = weights_to_int8(&model(
            "Add",
            &[("w", vec![1024], big), ("b", vec![3, 341], small)],
        ))
        .unwrap();
        assert_eq!((out.int8, out.fp16), (0, 1));
        let (inits, ops) = parse(&out.model);
        assert!(inits.contains_key("w__fp16") && inits.contains_key("b"));
        assert_eq!(ops, ["Cast", "Add"]);
        // Deterministic.
        let again = weights_to_int8(&model(
            "Add",
            &[
                ("w", vec![1024], (0..1024).map(|i| i as f32 * 0.5).collect()),
                ("b", vec![3, 341], (0..1023).map(|i| i as f32).collect()),
            ],
        ))
        .unwrap();
        assert_eq!(again.model, out.model);
    }
}
