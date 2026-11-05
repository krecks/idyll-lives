//! LFM2.5's 4-bit ONNX build made smaller without changing anything it computes (used by
//! `cargo fetch-llm`):
//!
//! - **Weights stored once:** an initializer whose bytes are the same as an earlier one's points
//!   at those bytes in the weights file. The export keeps the tied 4-bit embedding twice, once for
//!   reading the tokens (`GatherBlockQuantized`) and once for the output layer (`MatMulNBits`):
//!   33.6 MB, which gzip can't fold away (its window is 32 KB).
//! - **Rotary tables cut to [`POSITIONS`]:** `cos_cache` and `sin_cache` hold one row per
//!   position the attention can be at, 128,000 of them (32.8 MB), where a line's prompt and reply
//!   are under a thousand tokens. The rows kept are the first ones, as they were, so every
//!   position below the limit reads the same values; `generate.ts` never goes past it
//!   (`LlmEngine.positions`).
//!
//! Works on the protobuf wire format as `fp16.rs` does: only the initializers' dims and
//! external-data entries change, everything else is copied byte for byte. Deterministic, so the
//! files it writes are pinned by their SHA-256 (`llmfetch.rs`).

use crate::download::sha256_hex;
use crate::fp16::{bytes_field, fields, string_field, varint_field};
use std::collections::HashMap;

/// Positions the attention can be at after the cut: the longest prompt and reply together.
pub const POSITIONS: u64 = 4096;

/// The rotary tables, one row per position.
const ROTARY: [&str; 2] = ["cos_cache", "sin_cache"];

/// Weights start on a multiple of this in the weights file, as in the export.
const ALIGN: usize = 4096;

/// `TensorProto.data_location` EXTERNAL.
const EXTERNAL: u64 = 1;
const FLOAT: u64 = 1;

/// The smaller model: its graph, its weights file, and what changed.
pub struct Slim {
    pub graph: Vec<u8>,
    pub data: Vec<u8>,
    /// Initializers that now share another's bytes, and the bytes that saved.
    pub shared: usize,
    pub shared_bytes: usize,
    /// Bytes cut from the rotary tables.
    pub cut_bytes: usize,
}

/// An initializer's fields the repack reads.
struct Initializer<'a> {
    name: String,
    ty: u64,
    dims: Vec<u64>,
    external: bool,
    /// External data: where its bytes are (`location` kept as it is).
    location: String,
    offset: usize,
    length: usize,
    /// Its fields other than dims and external data, as they were.
    rest: Vec<&'a [u8]>,
}

fn read(buf: &[u8]) -> Result<Initializer<'_>, String> {
    let mut t = Initializer {
        name: String::new(),
        ty: 0,
        dims: Vec::new(),
        external: false,
        location: String::new(),
        offset: 0,
        length: 0,
        rest: Vec::new(),
    };
    let mut length = None;
    for f in fields(buf)? {
        match (f.number, f.wire) {
            (1, 0) => t.dims.push(f.varint),
            (1, 2) => {
                // Packed dims.
                let mut at = 0;
                for d in packed_varints(f.payload, &mut at)? {
                    t.dims.push(d);
                }
            }
            (13, 2) => {
                let (mut key, mut value) = (String::new(), String::new());
                for e in fields(f.payload)? {
                    match e.number {
                        1 => key = String::from_utf8_lossy(e.payload).into_owned(),
                        2 => value = String::from_utf8_lossy(e.payload).into_owned(),
                        _ => {}
                    }
                }
                let number = || {
                    value
                        .parse::<usize>()
                        .map_err(|e| format!("ONNX: {key} {value:?}: {e}"))
                };
                match key.as_str() {
                    "location" => t.location = value,
                    "offset" => t.offset = number()?,
                    "length" => length = Some(number()?),
                    other => {
                        return Err(format!("ONNX: external data entry {other:?} not supported"));
                    }
                }
            }
            _ => {
                match f.number {
                    2 => t.ty = f.varint,
                    8 => t.name = String::from_utf8_lossy(f.payload).into_owned(),
                    14 => t.external = f.varint == EXTERNAL,
                    _ => {}
                }
                t.rest.push(f.bytes);
            }
        }
    }
    if t.external {
        t.length = length.ok_or_else(|| format!("ONNX: {} has no external data length", t.name))?;
    }
    Ok(t)
}

/// Varints packed one after another (a packed repeated field's payload).
fn packed_varints(buf: &[u8], at: &mut usize) -> Result<Vec<u64>, String> {
    let mut out = Vec::new();
    while *at < buf.len() {
        let mut value: u64 = 0;
        let mut shift = 0;
        loop {
            let b = *buf.get(*at).ok_or("protobuf: truncated packed varint")?;
            *at += 1;
            value |= u64::from(b & 0x7f) << shift;
            if b < 0x80 {
                break;
            }
            shift += 7;
        }
        out.push(value);
    }
    Ok(out)
}

/// The initializer with these dims and this place in the weights file.
fn write(t: &Initializer, dims: &[u64], offset: usize, length: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for &d in dims {
        out.extend_from_slice(&varint_field(1, d));
    }
    for r in &t.rest {
        out.extend_from_slice(r);
    }
    for (key, value) in [
        ("location", t.location.clone()),
        ("offset", offset.to_string()),
        ("length", length.to_string()),
    ] {
        let mut entry = string_field(1, key);
        entry.extend_from_slice(&string_field(2, &value));
        out.extend_from_slice(&bytes_field(13, &entry));
    }
    out
}

/// The model `graph` with its weights file `data`, made smaller (see the module's comment).
pub fn slim(graph: &[u8], data: &[u8], positions: u64) -> Result<Slim, String> {
    let top = fields(graph)?;
    let graph_field = top.iter().find(|f| f.number == 7).ok_or("ONNX: no graph")?;
    let mut out_data: Vec<u8> = Vec::with_capacity(data.len());
    // Bytes already written, by their SHA-256: (offset, length).
    let mut placed: HashMap<String, (usize, usize)> = HashMap::new();
    let (mut shared, mut shared_bytes, mut cut_bytes, mut cut) = (0, 0, 0, 0);
    let mut parts: Vec<Vec<u8>> = Vec::new();
    for f in fields(graph_field.payload)? {
        if f.number != 5 {
            parts.push(f.bytes.to_vec());
            continue;
        }
        let t = read(f.payload)?;
        if !t.external {
            parts.push(f.bytes.to_vec());
            continue;
        }
        let end = t.offset.checked_add(t.length).filter(|&e| e <= data.len());
        let mut bytes = &data[t.offset
            ..end.ok_or_else(|| format!("ONNX: {} lies past the weights file", t.name))?];
        let mut dims = t.dims.clone();
        if ROTARY.contains(&t.name.as_str()) {
            let [rows, width] = dims[..] else {
                return Err(format!("ONNX: {} is not a table ({:?})", t.name, t.dims));
            };
            if t.ty != FLOAT || bytes.len() as u64 != rows * width * 4 {
                return Err(format!("ONNX: {} is not a float32 table", t.name));
            }
            if rows < positions {
                return Err(format!(
                    "ONNX: {} has {rows} rows, fewer than {positions}",
                    t.name
                ));
            }
            let keep = (positions * width * 4) as usize;
            cut_bytes += bytes.len() - keep;
            bytes = &bytes[..keep];
            dims[0] = positions;
            cut += 1;
        }
        let key = sha256_hex(bytes);
        let (offset, length) = match placed.get(&key) {
            Some(&(o, l)) if out_data[o..o + l] == *bytes => {
                shared += 1;
                shared_bytes += l;
                (o, l)
            }
            _ => {
                out_data.resize(out_data.len().next_multiple_of(ALIGN), 0);
                let o = out_data.len();
                out_data.extend_from_slice(bytes);
                placed.insert(key, (o, bytes.len()));
                (o, bytes.len())
            }
        };
        parts.push(bytes_field(5, &write(&t, &dims, offset, length)));
    }
    if cut != ROTARY.len() {
        return Err(format!("ONNX: found {cut} of the rotary tables {ROTARY:?}"));
    }
    let graph_bytes: Vec<u8> = parts.concat();
    let mut out = Vec::new();
    for f in &top {
        if std::ptr::eq(f, graph_field) {
            out.extend_from_slice(&bytes_field(7, &graph_bytes));
        } else {
            out.extend_from_slice(f.bytes);
        }
    }
    Ok(Slim {
        graph: out,
        data: out_data,
        shared,
        shared_bytes,
        cut_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An external float32 initializer.
    fn tensor(name: &str, dims: &[u64], offset: usize, length: usize) -> Vec<u8> {
        let mut t = Vec::new();
        for &d in dims {
            t.extend_from_slice(&varint_field(1, d));
        }
        t.extend_from_slice(&varint_field(2, FLOAT));
        t.extend_from_slice(&string_field(8, name));
        for (k, v) in [
            ("location", "w.onnx_data".to_string()),
            ("offset", offset.to_string()),
            ("length", length.to_string()),
        ] {
            let mut e = string_field(1, k);
            e.extend_from_slice(&string_field(2, &v));
            t.extend_from_slice(&bytes_field(13, &e));
        }
        t.extend_from_slice(&varint_field(14, EXTERNAL));
        t
    }

    /// A graph with a node, two copies of the same weight (`a` and `b`), a different one (`c`), and
    /// rotary tables of `rows` rows of 2; and its weights file.
    fn model(rows: u64) -> (Vec<u8>, Vec<u8>) {
        let mut data = Vec::new();
        let mut add = |bytes: &[u8]| {
            let o = data.len();
            data.extend_from_slice(bytes);
            (o, bytes.len())
        };
        let w: Vec<u8> = (0..64u8).collect();
        let (ao, al) = add(&w);
        let (bo, bl) = add(&w);
        let (co, cl) = add(&[7u8; 16]);
        let table = |at: f32| -> Vec<u8> {
            (0..rows * 2)
                .flat_map(|i| (i as f32 + at).to_le_bytes())
                .collect()
        };
        let (xo, xl) = add(&table(0.0));
        let (yo, yl) = add(&table(0.5));
        let mut g = bytes_field(1, &string_field(4, "Relu"));
        g.extend_from_slice(&bytes_field(5, &tensor("a", &[64], ao, al)));
        g.extend_from_slice(&bytes_field(5, &tensor("b", &[8, 8], bo, bl)));
        g.extend_from_slice(&bytes_field(5, &tensor("c", &[16], co, cl)));
        g.extend_from_slice(&bytes_field(5, &tensor("cos_cache", &[rows, 2], xo, xl)));
        g.extend_from_slice(&bytes_field(5, &tensor("sin_cache", &[rows, 2], yo, yl)));
        let mut m = varint_field(1, 8);
        m.extend_from_slice(&bytes_field(7, &g));
        (m, data)
    }

    /// Each initializer of the graph: name, dims, and its bytes in the weights file.
    fn weights(graph: &[u8], data: &[u8]) -> Vec<(String, Vec<u64>, Vec<u8>, usize)> {
        let top = fields(graph).unwrap();
        let g = top.iter().find(|f| f.number == 7).unwrap();
        fields(g.payload)
            .unwrap()
            .iter()
            .filter(|f| f.number == 5)
            .map(|f| {
                let t = read(f.payload).unwrap();
                (
                    t.name,
                    t.dims,
                    data[t.offset..t.offset + t.length].to_vec(),
                    t.offset,
                )
            })
            .collect()
    }

    #[test]
    fn stores_same_bytes_once_and_cuts_the_rotary_tables() {
        let (graph, data) = model(10);
        let s = slim(&graph, &data, 4).unwrap();
        assert_eq!((s.shared, s.shared_bytes), (1, 64));
        assert_eq!(s.cut_bytes, 2 * 6 * 2 * 4);
        let before = weights(&graph, &data);
        let after = weights(&s.graph, &s.data);
        // The same weights by name, with the same bytes (dims too, but for the tables).
        for (b, a) in before.iter().zip(&after) {
            assert_eq!(a.0, b.0);
            if a.0.ends_with("_cache") {
                assert_eq!(a.1, [4, 2]);
                assert_eq!(a.2, b.2[..4 * 2 * 4]);
            } else {
                assert_eq!((&a.1, &a.2), (&b.1, &b.2));
            }
        }
        // `b` points at `a`'s bytes; the others each have their own.
        assert_eq!(after[0].3, after[1].3);
        let mut own: Vec<usize> = [0, 2, 3, 4].iter().map(|&i| after[i].3).collect();
        own.dedup();
        assert_eq!(own.len(), 4);
        // Every weight starts aligned, and the node is still there.
        assert!(after.iter().all(|w| w.3 % ALIGN == 0));
        assert!(s.graph.windows(4).any(|w| w == b"Relu"));
        // Deterministic.
        let again = slim(&graph, &data, 4).unwrap();
        assert_eq!((again.graph, again.data), (s.graph, s.data));
    }

    #[test]
    fn refuses_tables_shorter_than_the_limit() {
        let (graph, data) = model(3);
        assert!(
            slim(&graph, &data, 4)
                .err()
                .unwrap()
                .contains("fewer than 4")
        );
    }
}
