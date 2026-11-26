//! Reading a `.glb`: its JSON and binary chunks, its accessors (floats, or integers as the spec reads
//! them, normalised if flagged) and the size of a PNG or JPEG in it.

use super::json::loads_bytes;
use super::py::{J, Num};

/// What went wrong, as Python says it (`str(err)`): a missing key `'name'`, an index out of range…
pub type R<T> = Result<T, String>;

/// `d[key]`: the value, or a KeyError's text.
pub fn key<'a>(d: &'a J, k: &str) -> R<&'a J> {
    d.get(k).ok_or_else(|| super::py::repr_str(k))
}

/// `l[i]` for a list and an index from the file.
pub fn index<'a>(l: &'a J, i: &J) -> R<&'a J> {
    let i = match i {
        J::Int(i) => *i,
        J::Bool(b) => *b as i64,
        other => {
            return Err(format!(
                "list indices must be integers or slices, not {}",
                type_name(other)
            ));
        }
    };
    l.at(i).ok_or_else(|| "list index out of range".into())
}

/// `d[key][i]`: a list of the file's (`accessors`, `nodes`…) at an index.
pub fn item<'a>(gltf: &'a J, list: &str, i: &J) -> R<&'a J> {
    index(key(gltf, list)?, i)
}

pub fn type_name(v: &J) -> &'static str {
    match v {
        J::Null => "NoneType",
        J::Bool(_) => "bool",
        J::Int(_) => "int",
        J::Float(_) => "float",
        J::Str(_) => "str",
        J::Arr(_) => "list",
        J::Obj(_) => "dict",
    }
}

/// An integer of the file (`d.get(key, default)`), 0 for a `null`.
pub fn int(d: &J, k: &str, default: i64) -> R<i64> {
    match d.get(k) {
        None => Ok(default),
        Some(J::Int(i)) => Ok(*i),
        Some(J::Bool(b)) => Ok(*b as i64),
        Some(other) => Err(format!(
            "unsupported operand type(s) for +: 'int' and '{}'",
            type_name(other)
        )),
    }
}

/// `view.get('byteStride') or fallback`.
fn stride(view: &J, fallback: i64) -> R<i64> {
    match view.get("byteStride") {
        Some(v) if v.truthy() => int(view, "byteStride", 0),
        _ => Ok(fallback),
    }
}

/// `acc['count']`.
fn count(acc: &J) -> R<i64> {
    key(acc, "count")?;
    int(acc, "count", 0)
}

/// A glTF binary: its JSON, its binary chunk and the file's size in bytes.
pub struct Glb {
    pub gltf: J,
    pub bin: Vec<u8>,
    pub size: usize,
}

/// Reads the chunks of a `.glb`.
pub fn parse_glb(data: &[u8]) -> R<Glb> {
    let word = |off: usize| -> R<u32> {
        data.get(off..off + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).ok_or_else(|| {
            format!("unpack_from requires a buffer of at least {} bytes for unpacking 8 bytes at offset {} (actual buffer size is {})", off + 8, off, data.len())
        })
    };
    if data.len() < 12 {
        return Err(format!(
            "unpack_from requires a buffer of at least 12 bytes for unpacking 12 bytes at offset 0 (actual buffer size is {})",
            data.len()
        ));
    }
    if &data[..4] != b"glTF" || word(4)? != 2 {
        return Err("not a glTF 2.0 binary".into());
    }
    let (mut gltf, mut bin, mut off) = (None, Vec::new(), 12usize);
    while off < data.len() {
        let (n, kind) = (word(off)? as usize, word(off + 4)?);
        let chunk = &data[(off + 8).min(data.len())..(off + 8 + n).min(data.len())];
        if kind == 0x4E4F534A {
            gltf = Some(loads_bytes(chunk)?);
        } else if kind == 0x004E4942 {
            bin = chunk.to_vec();
        }
        off += 8 + n;
    }
    Ok(Glb {
        gltf: gltf.ok_or("'NoneType' object has no attribute 'get'")?,
        bin,
        size: data.len(),
    })
}

/// (width, height) of a PNG or JPEG, or None.
pub fn image_size(blob: &[u8]) -> Option<(u32, u32)> {
    let be16 = |i: usize| {
        blob.get(i..i + 2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]) as usize)
    };
    if blob.starts_with(b"\x89PNG\r\n\x1a\n") {
        let b = blob.get(16..24)?;
        return Some((
            u32::from_be_bytes(b[..4].try_into().unwrap()),
            u32::from_be_bytes(b[4..].try_into().unwrap()),
        ));
    }
    if blob.starts_with(b"\xff\xd8") {
        let mut i = 2;
        while i + 9 < blob.len() {
            let (marker, size) = (blob[i + 1], be16(i + 2)?);
            if (0xC0..=0xCF).contains(&marker) && ![0xC4, 0xC8, 0xCC].contains(&marker) {
                let (h, w) = (be16(i + 5)?, be16(i + 7)?);
                return Some((w as u32, h as u32));
            }
            i += 2 + size;
        }
    }
    None
}

/// Elements per accessor type, for the clips' accessors (no VEC2).
fn clip_width(acc: &J) -> R<usize> {
    let t = key(acc, "type")?;
    match t.str() {
        Some("SCALAR") => Ok(1),
        Some("VEC3") => Ok(3),
        Some("VEC4") => Ok(4),
        _ => Err(super::py::repr(t)),
    }
}

fn bytes(bin: &[u8], at: i64, len: usize) -> R<&[u8]> {
    // (Checked, not added and compared: a `byteOffset` near i64::MAX wraps to a small number and
    // passes `at + len <= bin.len()` before the slice panics.)
    let range = usize::try_from(at)
        .ok()
        .and_then(|at| Some(at..at.checked_add(len)?));
    match range.and_then(|r| bin.get(r)) {
        Some(slice) => Ok(slice),
        None => Err(format!(
            "unpack_from requires a buffer of at least {} bytes for unpacking {len} bytes at offset {at} (actual buffer size is {})",
            at.saturating_add(len as i64),
            bin.len()
        )),
    }
}

fn floats(bin: &[u8], at: i64, n: usize) -> R<Vec<f64>> {
    let len = n
        .checked_mul(4)
        .ok_or("an accessor too long for any buffer")?;
    Ok(bytes(bin, at, len)?
        .chunks(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()) as f64)
        .collect())
}

/// The first (or `last`) element of a float accessor (a clip's first or last key); None if it isn't floats.
pub fn first_value(g: &Glb, accessor: &J, last: bool) -> R<Option<Vec<f64>>> {
    let acc = item(&g.gltf, "accessors", accessor)?;
    if acc.get("componentType") != Some(&J::Int(5126)) {
        return Ok(None);
    }
    let view = item(&g.gltf, "bufferViews", key(acc, "bufferView")?)?;
    let n = clip_width(acc)?;
    let at = if last {
        count(acc)?
            .saturating_sub(1)
            .saturating_mul(stride(view, 4 * n as i64)?)
    } else {
        0
    };
    Ok(Some(floats(
        &g.bin,
        int(view, "byteOffset", 0)?
            .saturating_add(int(acc, "byteOffset", 0)?)
            .saturating_add(at),
        n,
    )?))
}

/// Every element of a clip's accessor (its keys: floats, or integers normalised as `accessor` reads them).
pub fn values(g: &Glb, index: &J) -> R<Vec<Vec<f64>>> {
    clip_width(item(&g.gltf, "accessors", index)?)?;
    Ok(accessor(g, index)?
        .ok_or("a clip's keys can't be read here (compressed, sparse)")?
        .into_iter()
        .map(|e| e.into_iter().map(Num::f).collect())
        .collect())
}

/// Every element of an accessor (floats, or integers read as the spec says: normalised if flagged), or
/// None if it can't be read here (compressed, sparse).
pub fn accessor(g: &Glb, index: &J) -> R<Option<Vec<Vec<Num>>>> {
    let acc = item(&g.gltf, "accessors", index)?;
    if !acc.has("bufferView") || acc.get("sparse").is_some_and(J::truthy) {
        return Ok(None);
    }
    let view = item(&g.gltf, "bufferViews", key(acc, "bufferView")?)?;
    if view.get("extensions").is_some_and(J::truthy) {
        return Ok(None);
    }
    let t = key(acc, "type")?;
    let n = match t.str() {
        Some("SCALAR") => 1,
        Some("VEC2") => 2,
        Some("VEC3") => 3,
        Some("VEC4") => 4,
        _ => return Err(super::py::repr(t)),
    };
    let ct = key(acc, "componentType")?;
    let (size, top): (usize, Option<f64>) = match ct {
        J::Int(5126) => (4, None),
        J::Int(5120) => (1, Some(127.0)),
        J::Int(5121) => (1, Some(255.0)),
        J::Int(5122) => (2, Some(32767.0)),
        J::Int(5123) => (2, Some(65535.0)),
        J::Int(5125) => (4, None),
        other => return Err(super::py::repr(other)),
    };
    let kind = ct.num().map(|n| n.f() as i64).unwrap_or(0);
    let stride = stride(view, (size * n) as i64)?;
    // (Saturating: a sum too big for an i64 is past the end, which `bytes` reports.)
    let at = int(view, "byteOffset", 0)?.saturating_add(int(acc, "byteOffset", 0)?);
    let normalized = acc.get("normalized").is_some_and(J::truthy) && top.is_some();
    let n_items = count(acc)?.max(0);
    // (A count from the file is a claim, not a size: reserving it would abort on a big number
    // before the first read's own bound check could report the file.)
    let mut out = Vec::with_capacity((n_items as usize).min(g.bin.len()));
    for k in 0..n_items {
        let b = bytes(
            &g.bin,
            at.saturating_add(k.saturating_mul(stride)),
            size * n,
        )?;
        let e: Vec<Num> = b
            .chunks(size)
            .map(|c| match kind {
                5126 => Num::F(f32::from_le_bytes(c.try_into().unwrap()) as f64),
                5120 => Num::I(c[0] as i8 as i64),
                5121 => Num::I(c[0] as i64),
                5122 => Num::I(i16::from_le_bytes(c.try_into().unwrap()) as i64),
                5123 => Num::I(u16::from_le_bytes(c.try_into().unwrap()) as i64),
                _ => Num::I(u32::from_le_bytes(c.try_into().unwrap()) as i64),
            })
            .map(|v| {
                if normalized {
                    Num::F((v.f() / top.unwrap()).max(-1.0))
                } else {
                    v
                }
            })
            .collect();
        out.push(e);
    }
    Ok(Some(out))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A `byteOffset` near `i64::MAX` used to wrap the guard's sum and panic on the slice.
    #[test]
    fn an_offset_past_the_end_is_reported_not_a_panic() {
        for (at, len) in [(i64::MAX, 4), (8, usize::MAX)] {
            let err = bytes(&[0u8; 8], at, len).unwrap_err();
            assert!(
                err.starts_with("unpack_from requires a buffer of at least"),
                "{at}/{len}: {err}"
            );
        }
        assert!(floats(&[0u8; 8], 8, usize::MAX).is_err());
        // An accessor's offsets and stride from the file: sums past an i64 are reported too.
        let json = r#"{"bufferViews": [{"byteLength": 8, "byteOffset": 9223372036854775807},
            {"byteLength": 8, "byteStride": 4611686018427387904}],
            "accessors": [{"bufferView": 0, "byteOffset": 1, "componentType": 5126, "type": "SCALAR", "count": 1},
            {"bufferView": 1, "componentType": 5126, "type": "SCALAR", "count": 1099511627776}]}"#;
        let g = parse_glb(&glb(json, &[0u8; 8])).unwrap();
        assert!(accessor(&g, &J::Int(0)).is_err());
        assert!(first_value(&g, &J::Int(0), false).is_err());
        assert!(first_value(&g, &J::Int(1), true).is_err());
    }

    /// A .glb of `json` and `bin`, each chunk padded to 4 bytes as the format wants.
    pub fn glb(json: &str, bin: &[u8]) -> Vec<u8> {
        let mut j = json.as_bytes().to_vec();
        while !j.len().is_multiple_of(4) {
            j.push(b' ');
        }
        let mut b = bin.to_vec();
        while !b.len().is_multiple_of(4) {
            b.push(0);
        }
        let total = 12 + 8 + j.len() + if b.is_empty() { 0 } else { 8 + b.len() };
        let mut out = b"glTF".to_vec();
        out.extend(2u32.to_le_bytes());
        out.extend((total as u32).to_le_bytes());
        out.extend((j.len() as u32).to_le_bytes());
        out.extend(0x4E4F534Au32.to_le_bytes());
        out.extend(j);
        if !b.is_empty() {
            out.extend((b.len() as u32).to_le_bytes());
            out.extend(0x004E4942u32.to_le_bytes());
            out.extend(b);
        }
        out
    }

    #[test]
    fn reads_a_glb() {
        let data = glb(
            r#"{"asset": {"version": "2.0"}, "nodes": [{"name": "a"}]}"#,
            &[1, 2, 3],
        );
        let g = parse_glb(&data).unwrap();
        assert_eq!(g.size, data.len());
        assert_eq!(g.bin, vec![1, 2, 3, 0]);
        assert_eq!(
            g.gltf.get("nodes").unwrap().at(0).unwrap().get("name"),
            Some(&J::Str("a".into()))
        );
        assert_eq!(
            parse_glb(b"glTF\x01\0\0\0\0\0\0\0").err().unwrap(),
            "not a glTF 2.0 binary"
        );
        assert!(parse_glb(b"PK\x03\x04\0\0\0\0\0\0\0\0").is_err());
    }

    #[test]
    fn reads_image_sizes() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend(2048u32.to_be_bytes());
        png.extend(512u32.to_be_bytes());
        png.extend([8, 6, 0, 0, 0]);
        assert_eq!(image_size(&png), Some((2048, 512)));
        // A JPEG: SOI, an APP0 segment of 16 bytes, then SOF0 (precision, height, width).
        let mut jpg = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        jpg.extend([0u8; 14]);
        jpg.extend([
            0xFF, 0xC0, 0x00, 0x11, 0x08, 0x01, 0x80, 0x02, 0x00, 0x03, 0x01, 0x22, 0x00,
        ]);
        assert_eq!(image_size(&jpg), Some((512, 384)));
        // DHT (C4) is no frame header: skipped.
        let mut dht = vec![0xFF, 0xD8, 0xFF, 0xC4, 0x00, 0x04, 0x00, 0x00];
        dht.extend([
            0xFF, 0xC2, 0x00, 0x11, 0x08, 0x00, 0x10, 0x00, 0x20, 0x03, 0x01,
        ]);
        assert_eq!(image_size(&dht), Some((32, 16)));
        assert_eq!(image_size(b"GIF89a....."), None);
    }

    #[test]
    fn decodes_accessors() {
        // Floats (VEC3), normalised shorts and bytes (VEC2), plain unsigned shorts with a stride.
        let mut bin = Vec::new();
        for v in [1.5f32, -2.0, 0.25] {
            bin.extend(v.to_le_bytes());
        }
        for v in [32767i16, -32768] {
            bin.extend(v.to_le_bytes());
        }
        bin.extend([255u8, 0]);
        for v in [7u16, 0, 9, 0] {
            bin.extend(v.to_le_bytes());
        }
        bin.extend([0x81u8, 0x7f]); // -127 and 127 as normalised signed bytes
        let json = r#"{"bufferViews": [{"byteOffset": 0, "byteLength": 12}, {"byteOffset": 12, "byteLength": 4},
            {"byteOffset": 16, "byteLength": 2}, {"byteOffset": 18, "byteLength": 8, "byteStride": 4},
            {"byteOffset": 26, "byteLength": 2}, {"byteOffset": 0, "byteLength": 2, "extensions": {"EXT_meshopt_compression": {}}}],
            "accessors": [{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": 1},
            {"bufferView": 1, "componentType": 5122, "normalized": true, "type": "VEC2", "count": 1},
            {"bufferView": 2, "componentType": 5121, "normalized": true, "type": "VEC2", "count": 1},
            {"bufferView": 3, "componentType": 5123, "type": "SCALAR", "count": 2},
            {"bufferView": 4, "componentType": 5120, "normalized": true, "type": "VEC2", "count": 1},
            {"bufferView": 5, "componentType": 5126, "type": "SCALAR", "count": 1},
            {"componentType": 5126, "type": "SCALAR", "count": 1},
            {"bufferView": 1, "componentType": 5122, "normalized": true, "type": "SCALAR", "count": 2}]}"#;
        let g = parse_glb(&glb(json, &bin)).unwrap();
        let read = |i: i64| accessor(&g, &J::Int(i)).unwrap();
        assert_eq!(
            read(0),
            Some(vec![vec![Num::F(1.5), Num::F(-2.0), Num::F(0.25)]])
        );
        // -32768 / 32767 is under -1: clamped, as the spec says.
        assert_eq!(read(1), Some(vec![vec![Num::F(1.0), Num::F(-1.0)]]));
        assert_eq!(read(2), Some(vec![vec![Num::F(1.0), Num::F(0.0)]]));
        assert_eq!(read(3), Some(vec![vec![Num::I(7)], vec![Num::I(9)]]));
        assert_eq!(read(4), Some(vec![vec![Num::F(-1.0), Num::F(1.0)]]));
        assert_eq!(read(5), None, "compressed");
        assert_eq!(read(6), None, "no buffer view");
        assert_eq!(values(&g, &J::Int(0)).unwrap(), vec![vec![1.5, -2.0, 0.25]]);
        // A clip's normalised shorts (quantised keys) as the numbers they stand for.
        assert_eq!(values(&g, &J::Int(7)).unwrap(), vec![vec![1.0], vec![-1.0]]);
        assert!(values(&g, &J::Int(5)).is_err(), "compressed");
        assert_eq!(
            first_value(&g, &J::Int(0), true).unwrap(),
            Some(vec![1.5, -2.0, 0.25])
        );
        assert_eq!(
            first_value(&g, &J::Int(1), false).unwrap(),
            None,
            "not floats"
        );
    }
}
