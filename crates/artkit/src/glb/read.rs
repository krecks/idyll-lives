//! Reading a model: its chunks, its accessors and its primitives.

use super::*;
use crate::json::Json;

/// The JSON and the binary chunk of a .glb.
pub(super) fn chunks(data: &[u8]) -> Result<(Json, &[u8]), String> {
    let u32_at = |at: usize| -> Result<u32, String> {
        data.get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .ok_or_else(|| "not a .glb: it ends too soon".to_string())
    };
    if data.get(0..4) != Some(b"glTF") {
        return Err("not a .glb (no glTF header)".into());
    }
    let version = u32_at(4)?;
    if version != 2 {
        return Err(format!("a glTF {version} file: only version 2 is read"));
    }
    let total = (u32_at(8)? as usize).min(data.len());
    let (mut json, mut bin) = (None, &data[0..0]);
    let mut at = 12;
    while at + 8 <= total {
        let len = u32_at(at)? as usize;
        let kind = &data[at + 4..at + 8];
        let body = data
            .get(at + 8..at + 8 + len)
            .ok_or("a chunk runs past the end of the file")?;
        match kind {
            b"JSON" => {
                let text = std::str::from_utf8(body).map_err(|_| "its JSON isn't UTF-8")?;
                json = Some(crate::json::loads(text)?);
            }
            b"BIN\0" => bin = body,
            _ => {}
        }
        at += 8 + len;
    }
    Ok((json.ok_or("no JSON chunk")?, bin))
}

pub(super) fn int(v: Option<&Json>) -> Option<usize> {
    match v {
        Some(Json::Int(i)) if *i >= 0 => Some(*i as usize),
        _ => None,
    }
}

pub(super) fn float(v: &Json) -> Option<f32> {
    match v {
        Json::Int(i) => Some(*i as f32),
        Json::Num(x) => Some(*x as f32),
        _ => None,
    }
}

pub(super) fn floats<const N: usize>(v: Option<&Json>, or: [f32; N]) -> Result<[f32; N], String> {
    match v {
        None => Ok(or),
        Some(Json::Arr(items)) if items.len() == N => {
            let mut out = or;
            for (o, item) in out.iter_mut().zip(items) {
                *o = float(item).ok_or("a number that isn't one")?;
            }
            Ok(out)
        }
        Some(_) => Err(format!("expected {N} numbers")),
    }
}

pub(super) fn list<'a>(j: &'a Json, key: &str) -> &'a [Json] {
    match j.get(key) {
        Some(Json::Arr(items)) => items,
        _ => &[],
    }
}

/// Where an accessor's elements lie in the buffer.
struct Layout {
    /// glTF's component type
    pub(super) kind: usize,
    /// a component's bytes
    pub(super) size: usize,
    /// numbers an element
    pub(super) per: usize,
    pub(super) count: usize,
    /// (first byte, bytes from one element to the next), inside the buffer; none without a buffer view (all zeros)
    pub(super) place: Option<(usize, usize)>,
}

/// The most bytes an accessor's elements may take: a count past it is refused before anything is made of it.
const MOST: usize = 1 << 28;

fn layout(j: &Json, bin: &[u8], index: usize) -> Result<Layout, String> {
    let acc = list(j, "accessors")
        .get(index)
        .ok_or_else(|| format!("no accessor {index}"))?;
    if acc.get("sparse").is_some() {
        return Err(format!("accessor {index} is sparse: not read"));
    }
    let per = match acc.get("type").and_then(Json::str) {
        Some("SCALAR") => 1,
        Some("VEC2") => 2,
        Some("VEC3") => 3,
        Some("VEC4") => 4,
        Some("MAT4") => 16,
        other => return Err(format!("accessor {index}: type {other:?}")),
    };
    let count = int(acc.get("count")).ok_or_else(|| format!("accessor {index}: no count"))?;
    let kind = int(acc.get("componentType")).unwrap_or(0);
    let size = match kind {
        5120 | 5121 => 1,
        5122 | 5123 => 2,
        5125 | 5126 => 4,
        _ => return Err(format!("accessor {index}: component type {kind}")),
    };
    if count.checked_mul(per * size).is_none_or(|b| b > MOST) {
        return Err(format!("accessor {index}: {count} elements"));
    }
    let Some(view_at) = int(acc.get("bufferView")) else {
        return Ok(Layout {
            kind,
            size,
            per,
            count,
            place: None,
        });
    };
    let view = list(j, "bufferViews")
        .get(view_at)
        .ok_or_else(|| format!("no buffer view {view_at}"))?;
    if int(view.get("buffer")).unwrap_or(0) != 0 {
        return Err("a second buffer: only the .glb's own is read".into());
    }
    let start = int(view.get("byteOffset"))
        .unwrap_or(0)
        .saturating_add(int(acc.get("byteOffset")).unwrap_or(0));
    let stride = int(view.get("byteStride")).unwrap_or(size * per);
    let end = match count.checked_sub(1) {
        Some(last) => last
            .checked_mul(stride)
            .and_then(|x| x.checked_add(start))
            .and_then(|x| x.checked_add(per * size)),
        None => Some(0),
    };
    if end.is_none_or(|end| end > bin.len()) {
        return Err(format!("accessor {index} runs past the buffer"));
    }
    Ok(Layout {
        kind,
        size,
        per,
        count,
        place: Some((start, stride)),
    })
}

/// An accessor's values as floats (normalised integers scaled to 0..1 or -1..1), and how many make one element.
pub(super) fn accessor(j: &Json, bin: &[u8], index: usize) -> Result<(Vec<f32>, usize), String> {
    let Layout {
        kind,
        size,
        per,
        count,
        place,
    } = layout(j, bin, index)?;
    let normalized = list(j, "accessors")[index].get("normalized") == Some(&Json::Bool(true));
    let Some((start, stride)) = place else {
        return Ok((vec![0.0; count * per], per));
    };
    let mut out = Vec::with_capacity(count * per);
    for e in 0..count {
        for k in 0..per {
            let at = start + e * stride + k * size;
            let b = &bin[at..at + size];
            let v = match kind {
                5120 => {
                    let x = b[0] as i8 as f32;
                    if normalized { (x / 127.0).max(-1.0) } else { x }
                }
                5121 => {
                    let x = b[0] as f32;
                    if normalized { x / 255.0 } else { x }
                }
                5122 => {
                    let x = i16::from_le_bytes([b[0], b[1]]) as f32;
                    if normalized {
                        (x / 32767.0).max(-1.0)
                    } else {
                        x
                    }
                }
                5123 => {
                    let x = u16::from_le_bytes([b[0], b[1]]) as f32;
                    if normalized { x / 65535.0 } else { x }
                }
                5125 => u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f32,
                _ => f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            };
            out.push(v);
        }
    }
    Ok((out, per))
}

/// An index accessor's values, exact (a float holds only 24 bits of a 32-bit index).
fn indices(j: &Json, bin: &[u8], index: usize) -> Result<Vec<u32>, String> {
    let Layout {
        kind,
        size,
        per,
        count,
        place,
    } = layout(j, bin, index)?;
    if per != 1 || !matches!(kind, 5121 | 5123 | 5125) {
        return Err(format!("accessor {index}: indices of type {kind}"));
    }
    let Some((start, stride)) = place else {
        return Ok(vec![0; count]);
    };
    Ok((0..count)
        .map(|e| {
            let b = &bin[start + e * stride..];
            match size {
                1 => b[0] as u32,
                2 => u16::from_le_bytes([b[0], b[1]]) as u32,
                _ => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            }
        })
        .collect())
}

fn grouped<const N: usize>(
    (values, per): (Vec<f32>, usize),
    what: &str,
) -> Result<Vec<[f32; N]>, String> {
    if per != N {
        return Err(format!("{what}: {per} numbers an element, not {N}"));
    }
    Ok(values.as_chunks::<N>().0.to_vec())
}

pub(super) fn primitive(j: &Json, bin: &[u8], p: &Json, mi: usize) -> Result<Primitive, String> {
    let mode = int(p.get("mode")).unwrap_or(4);
    if mode != 4 {
        return Err(format!(
            "mesh {mi}: a primitive of mode {mode}: only triangles are read"
        ));
    }
    let attrs = p.get("attributes");
    let attr = |k: &str| int(attrs.and_then(|a| a.get(k)));
    let pos = attr("POSITION").ok_or_else(|| format!("mesh {mi}: a primitive without POSITION"))?;
    let positions: Vec<[f32; 3]> = grouped(accessor(j, bin, pos)?, "POSITION")?;
    let normals = match attr("NORMAL") {
        Some(a) => grouped(accessor(j, bin, a)?, "NORMAL")?,
        None => Vec::new(),
    };
    let uvs = match attr("TEXCOORD_0") {
        Some(a) => Some(grouped(accessor(j, bin, a)?, "TEXCOORD_0")?),
        None => None,
    };
    let colours = match attr("COLOR_0") {
        Some(a) => Some(match accessor(j, bin, a)? {
            (v, 3) => v
                .as_chunks::<3>()
                .0
                .iter()
                .map(|c| [c[0], c[1], c[2], 1.0])
                .collect(),
            read => grouped(read, "COLOR_0")?,
        }),
        None => None,
    };
    let indices = match int(p.get("indices")) {
        Some(a) => indices(j, bin, a)?,
        None => (0..positions.len() as u32).collect(),
    };
    if indices.len() % 3 != 0 {
        return Err(format!(
            "mesh {mi}: {} indices: not triangles",
            indices.len()
        ));
    }
    if let Some(&bad) = indices.iter().find(|&&i| i as usize >= positions.len()) {
        return Err(format!(
            "mesh {mi}: index {bad} past its {} vertices",
            positions.len()
        ));
    }
    let same = |n: usize| n == positions.len();
    if !(normals.is_empty() || same(normals.len()))
        || !uvs.as_ref().is_none_or(|u| same(u.len()))
        || !colours.as_ref().is_none_or(|c| same(c.len()))
    {
        return Err(format!("mesh {mi}: attributes of different counts"));
    }
    Ok(Primitive {
        positions,
        normals,
        uvs,
        colours,
        indices,
        material: int(p.get("material")),
    })
}
