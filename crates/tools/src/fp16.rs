//! An ONNX model with its weights stored as 16-bit floats: every float32 weight of 1,024 values
//! or more becomes float16 (rounded to nearest, ties to even), and a `Cast` back to float32 is put
//! in front of the graph under the weight's own name, so every operator still sees float32 and the
//! GPU does the same arithmetic as before. Half the download; ONNX Runtime folds the casts once
//! when the session starts. A weight whose values don't fit in float16 stays as it is.
//!
//! Works on the protobuf wire format directly: only the graph's initializers and nodes are
//! touched, every other field is copied byte for byte. Deterministic, so the output can be pinned
//! by its SHA-256. Used by `cargo fetch-voice`. (Was tools/voice/fp16.mjs.)

/// Weights smaller than this stay float32 (biases, norms: little to gain).
const MIN_VALUES: usize = 1024;

const FLOAT: u64 = 1;
const FLOAT16: u64 = 10;
/// `AttributeProto.type` INT.
const ATTR_INT: u64 = 2;

/// A protobuf field, its payload still a slice of the message it came from.
struct Field<'a> {
    number: u64,
    wire: u8,
    /// The value of a varint field.
    varint: u64,
    /// A length-delimited or fixed-width field's payload.
    payload: &'a [u8],
    /// The whole field: key, length, payload.
    bytes: &'a [u8],
}

fn varint(buf: &[u8], at: &mut usize) -> Result<u64, String> {
    let mut value: u64 = 0;
    let mut scale: u64 = 1;
    loop {
        let b = *buf.get(*at).ok_or("protobuf: truncated varint")?;
        *at += 1;
        value = value.wrapping_add((b & 0x7f) as u64 * scale);
        if b < 0x80 {
            return Ok(value);
        }
        scale = scale.wrapping_mul(128);
    }
}

fn encode_varint(value: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(10);
    let mut v = value;
    loop {
        let b = (v % 128) as u8;
        v /= 128;
        out.push(if v > 0 { b | 0x80 } else { b });
        if v == 0 {
            return out;
        }
    }
}

/// A message's fields in order.
fn fields(buf: &[u8]) -> Result<Vec<Field<'_>>, String> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < buf.len() {
        let start = at;
        let key = varint(buf, &mut at)?;
        let number = key / 8;
        let wire = (key % 8) as u8;
        let (varint, payload) = match wire {
            0 => {
                let v = varint(buf, &mut at)?;
                (v, &buf[at..at])
            }
            1 | 5 => {
                let n = if wire == 1 { 8 } else { 4 };
                if at + n > buf.len() {
                    return Err("protobuf: truncated fixed field".into());
                }
                let p = &buf[at..at + n];
                at += n;
                (0, p)
            }
            2 => {
                let len = varint(buf, &mut at)? as usize;
                if at + len > buf.len() {
                    return Err("protobuf: truncated bytes field".into());
                }
                let p = &buf[at..at + len];
                at += len;
                (0, p)
            }
            other => return Err(format!("protobuf: wire type {other} not supported")),
        };
        out.push(Field { number, wire, varint, payload, bytes: &buf[start..at] });
    }
    Ok(out)
}

fn bytes_field(field: u64, payload: &[u8]) -> Vec<u8> {
    let mut out = encode_varint(field * 8 + 2);
    out.extend_from_slice(&encode_varint(payload.len() as u64));
    out.extend_from_slice(payload);
    out
}

fn varint_field(field: u64, value: u64) -> Vec<u8> {
    let mut out = encode_varint(field * 8);
    out.extend_from_slice(&encode_varint(value));
    out
}

fn string_field(field: u64, s: &str) -> Vec<u8> {
    bytes_field(field, s.as_bytes())
}

/// float32 bits to float16 bits, rounded to nearest, ties to even (as NumPy's `astype(float16)`).
fn half(bits: u32) -> u16 {
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xff) as i32;
    let mut mant = bits & 0x7f_ffff;
    if exp == 0xff {
        return sign | 0x7c00 | if mant != 0 { 0x200 } else { 0 };
    }
    let e = exp - 127 + 15;
    if e >= 0x1f {
        return sign | 0x7c00;
    }
    if e <= 0 {
        if e < -10 {
            return sign;
        }
        mant |= 0x80_0000;
        let shift = (14 - e) as u32;
        let mut h = (mant >> shift) as u16;
        let rest = mant & ((1u32 << shift) - 1);
        let mid = 1u32 << (shift - 1);
        if rest > mid || (rest == mid && h & 1 == 1) {
            h += 1;
        }
        return sign | h;
    }
    let mut h = ((e as u32) << 10 | (mant >> 13)) as u16;
    let rest = mant & 0x1fff;
    if rest > 0x1000 || (rest == 0x1000 && h & 1 == 1) {
        h += 1;
    }
    sign | h
}

/// A float32 tensor's values as float16 bytes, or `None` if one doesn't fit (it would become
/// infinite).
fn to_half(values: &[f32]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(values.len() * 2);
    for v in values {
        let bits = v.to_bits();
        let h = half(bits);
        if h & 0x7c00 == 0x7c00 && bits & 0x7f80_0000 != 0x7f80_0000 {
            return None;
        }
        out.extend_from_slice(&h.to_le_bytes());
    }
    Some(out)
}

/// A `TensorProto`: its name, data type and, for float32, its values.
struct Tensor {
    name: String,
    ty: i64,
    values: Option<Vec<f32>>,
}

fn read_tensor(buf: &[u8]) -> Result<Tensor, String> {
    let mut name = String::new();
    let mut ty: i64 = 0;
    let mut raw: Option<&[u8]> = None;
    let mut floats: Vec<f32> = Vec::new();
    for f in fields(buf)? {
        match (f.number, f.wire) {
            (8, _) => name = String::from_utf8_lossy(f.payload).into_owned(),
            (2, _) => ty = f.varint as i64,
            (9, _) => raw = Some(f.payload),
            (4, 2) => {
                for c in f.payload.as_chunks::<4>().0 {
                    floats.push(f32::from_le_bytes(*c));
                }
            }
            (4, 5) => floats.push(f32::from_le_bytes([f.payload[0], f.payload[1], f.payload[2], f.payload[3]])),
            // External data: left alone.
            (13, _) | (14, _) => return Ok(Tensor { name, ty: -1, values: None }),
            _ => {}
        }
    }
    let values = if ty as u64 == FLOAT {
        match raw {
            Some(r) => Some(r.as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)).collect()),
            None => Some(floats),
        }
    } else {
        None
    };
    Ok(Tensor { name, ty, values })
}

/// The tensor with float16 data under `name`: its dims copied as they are.
fn half_tensor(buf: &[u8], name: &str, data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for f in fields(buf)? {
        if f.number == 1 {
            out.extend_from_slice(f.bytes);
        }
    }
    out.extend_from_slice(&varint_field(2, FLOAT16));
    out.extend_from_slice(&string_field(8, name));
    out.extend_from_slice(&bytes_field(9, data));
    Ok(out)
}

/// `Cast(from) -> to` as float32.
fn cast_node(from: &str, to: &str) -> Vec<u8> {
    let mut attribute = string_field(1, "to");
    attribute.extend_from_slice(&varint_field(3, FLOAT));
    attribute.extend_from_slice(&varint_field(20, ATTR_INT));
    let mut out = string_field(1, from);
    out.extend_from_slice(&string_field(2, to));
    out.extend_from_slice(&string_field(3, &format!("fp16_cast_{to}")));
    out.extend_from_slice(&string_field(4, "Cast"));
    out.extend_from_slice(&bytes_field(5, &attribute));
    out
}

/// The model with its big float32 weights stored as float16; and how many weights and how many
/// bytes were saved.
pub fn weights_to_fp16(model: &[u8]) -> Result<(Vec<u8>, usize, usize), String> {
    let top = fields(model)?;
    let graph_field = top.iter().find(|f| f.number == 7).ok_or("ONNX: no graph")?;
    let mut casts: Vec<Vec<u8>> = Vec::new();
    let mut graph_parts: Vec<Vec<u8>> = Vec::new();
    let mut saved = 0;
    for f in fields(graph_field.payload)? {
        if f.number == 5 {
            let t = read_tensor(f.payload)?;
            let data = match &t.values {
                Some(v) if t.ty as u64 == FLOAT && v.len() >= MIN_VALUES => to_half(v),
                _ => None,
            };
            if let Some(data) = data {
                let name = format!("{}__fp16", t.name);
                graph_parts.push(bytes_field(5, &half_tensor(f.payload, &name, &data)?));
                casts.push(bytes_field(1, &cast_node(&name, &t.name)));
                saved += t.values.as_ref().map_or(0, |v| v.len()) * 2;
                continue;
            }
        }
        graph_parts.push(f.bytes.to_vec());
    }
    // The casts before every other node (nodes are listed in the order they run).
    let mut graph = Vec::new();
    for c in &casts {
        graph.extend_from_slice(c);
    }
    for p in &graph_parts {
        graph.extend_from_slice(p);
    }
    let mut out = Vec::new();
    for f in &top {
        if std::ptr::eq(f, graph_field) {
            out.extend_from_slice(&bytes_field(7, &graph));
        } else {
            out.extend_from_slice(f.bytes);
        }
    }
    Ok((out, casts.len(), saved))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The float16 bits each float32 goes to, as NumPy's `astype(float16)` gives them (the values
    /// tools/voice/fp16.mjs was pinned by, before this was Rust).
    #[test]
    fn halves_as_numpy_does() {
        let cases: [(u32, u16); 15] = [
            (0x3f80_0000, 0x3c00), // 1
            (0x3f00_0000, 0x3800), // 0.5
            (0x477f_e000, 0x7bff), // 65504, the largest float16
            (0x477f_f000, 0x7c00), // 65520: rounds up past it, to infinity
            (0x477f_ef00, 0x7bff), // 65519
            (0x3f80_2000, 0x3c01), // 1 + 2^-10
            (0xc020_0000, 0xc100), // -2.5
            (0x322b_cc77, 0x0000), // 1e-8: under the smallest subnormal
            (0x0000_0000, 0x0000),
            (0x8000_0000, 0x8000), // -0
            (0x3f80_1000, 0x3c00), // halfway between 1 and the next float16: ties to even, down
            (0x3f80_3000, 0x3c02), // halfway above that: ties to even, up
            (0x3727_c5ac, 0x00a8), // 1e-5, a subnormal
            (0x7f7f_ffff, 0x7c00), // f32::MAX
            (0x3880_0000, 0x0400), // the smallest normal float16
        ];
        for (bits, want) in cases {
            assert_eq!(half(bits), want, "{bits:#010x}");
        }
        // A NaN stays a NaN (quiet), and the infinities keep their sign.
        assert_eq!(half(0x7fc0_0000), 0x7e00);
        assert_eq!(half(0x7f80_0000), 0x7c00);
        assert_eq!(half(0xff80_0000), 0xfc00);
    }

    #[test]
    fn varints_round_trip() {
        for v in [0u64, 1, 127, 128, 300, 1 << 20, u64::from(u32::MAX)] {
            let bytes = encode_varint(v);
            let mut at = 0;
            assert_eq!(varint(&bytes, &mut at).unwrap(), v);
            assert_eq!(at, bytes.len());
        }
    }

    /// A graph with one initializer of `n` float32 values called `w`, and one node.
    fn model(n: usize) -> Vec<u8> {
        let mut values = Vec::new();
        for i in 0..n {
            values.extend_from_slice(&(i as f32 * 0.5).to_le_bytes());
        }
        let mut tensor = varint_field(2, FLOAT);
        tensor.extend_from_slice(&varint_field(1, n as u64));
        tensor.extend_from_slice(&string_field(8, "w"));
        tensor.extend_from_slice(&bytes_field(9, &values));
        let node = string_field(4, "Relu");
        let mut graph = bytes_field(1, &node);
        graph.extend_from_slice(&bytes_field(5, &tensor));
        let mut out = varint_field(1, 3);
        out.extend_from_slice(&bytes_field(7, &graph));
        out
    }

    #[test]
    fn converts_the_big_weights_only() {
        let big = weights_to_fp16(&model(1024)).unwrap();
        assert_eq!(big.1, 1);
        assert_eq!(big.2, 2048);
        let small = weights_to_fp16(&model(1023)).unwrap();
        assert_eq!(small.1, 0);
        assert_eq!(small.2, 0);
        assert_eq!(small.0, model(1023));
    }
}
