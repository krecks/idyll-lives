//! Accessors and the buffer views behind them.
//!
//! Split from `assets/model.rs` (see `mod.rs`).

use super::*;

pub(crate) fn list<'a>(j: &'a J, key: &str) -> &'a [J] {
    j.get(key)
        .and_then(J::as_array)
        .map_or(&[], |a| a.as_slice())
}

pub(crate) fn uint(j: &J, key: &str) -> Option<usize> {
    j.get(key).and_then(J::as_u64).map(|v| v as usize)
}

fn components(kind: &str) -> Result<usize, String> {
    Ok(match kind {
        "SCALAR" => 1,
        "VEC2" => 2,
        "VEC3" => 3,
        "VEC4" => 4,
        "MAT2" => 4,
        "MAT3" => 9,
        "MAT4" => 16,
        other => return Err(format!("an accessor of type {other}")),
    })
}

fn component_size(component: u64) -> Result<usize, String> {
    Ok(match component {
        BYTE | UBYTE => 1,
        SHORT | USHORT => 2,
        UINT | FLOAT => 4,
        other => return Err(format!("an accessor of component type {other}")),
    })
}

/// An accessor's elements, packed (little-endian, no padding between them).
#[derive(Clone)]
pub(crate) struct Accessor {
    pub(crate) component: u64,
    pub(crate) normalized: bool,
    pub(crate) kind: String,
    pub(crate) count: usize,
    pub(crate) width: usize,
    pub(crate) data: Vec<u8>,
    pub(crate) min: Option<J>,
    pub(crate) max: Option<J>,
}

impl Accessor {
    /// Element `i`'s component `c` as the number the spec reads it as (a normalised integer as 0..1 or -1..1).
    pub(crate) fn value(&self, i: usize, c: usize) -> f32 {
        let size = self.data.len() / (self.count * self.width).max(1);
        let at = (i * self.width + c) * size;
        let b = &self.data[at..at + size];
        match self.component {
            FLOAT => f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            UINT => u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f32,
            USHORT => {
                let v = u16::from_le_bytes([b[0], b[1]]) as f32;
                if self.normalized { v / 65535.0 } else { v }
            }
            SHORT => {
                let v = i16::from_le_bytes([b[0], b[1]]) as f32;
                if self.normalized {
                    (v / 32767.0).max(-1.0)
                } else {
                    v
                }
            }
            UBYTE => {
                let v = b[0] as f32;
                if self.normalized { v / 255.0 } else { v }
            }
            _ => {
                let v = b[0] as i8 as f32;
                if self.normalized {
                    (v / 127.0).max(-1.0)
                } else {
                    v
                }
            }
        }
    }

    pub(crate) fn index(&self, i: usize) -> u32 {
        let size = self.data.len() / self.count.max(1);
        let b = &self.data[i * size..(i + 1) * size];
        match size {
            1 => b[0] as u32,
            2 => u16::from_le_bytes([b[0], b[1]]) as u32,
            _ => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        }
    }
}

/// The buffer views' bytes: as they are in the file, or decoded where `EXT_meshopt_compression` holds them.
pub(crate) struct Views {
    pub(crate) bytes: Vec<Option<Vec<u8>>>,
    pub(crate) strides: Vec<Option<usize>>,
}

pub(crate) fn views(g: &Glb) -> Result<Views, String> {
    let mut bytes = Vec::new();
    let mut strides = Vec::new();
    for view in list(&g.json, "bufferViews") {
        strides.push(uint(view, "byteStride"));
        let ext = view.get("extensions").and_then(|e| e.get(MESHOPT));
        let Some(ext) = ext else {
            if uint(view, "buffer") != Some(0) {
                bytes.push(None);
                continue;
            }
            let at = uint(view, "byteOffset").unwrap_or(0);
            let len = uint(view, "byteLength").ok_or("a buffer view without a length")?;
            bytes.push(Some(
                g.bin
                    .get(at..at + len)
                    .ok_or("a buffer view past the end of the file")?
                    .to_vec(),
            ));
            continue;
        };
        if uint(ext, "buffer") != Some(0) {
            return Err("compressed data outside the file".into());
        }
        if ext
            .get("filter")
            .and_then(J::as_str)
            .is_some_and(|f| f != "NONE")
        {
            return Err("a meshopt filter (not read here)".into());
        }
        let at = uint(ext, "byteOffset").unwrap_or(0);
        let len = uint(ext, "byteLength").ok_or("a compressed view without a length")?;
        let stride = uint(ext, "byteStride").ok_or("a compressed view without a stride")?;
        let count = uint(ext, "count").ok_or("a compressed view without a count")?;
        let source = g
            .bin
            .get(at..at + len)
            .ok_or("a compressed view past the end of the file")?;
        let mut out = vec![0u8; count * stride];
        let code = match ext.get("mode").and_then(J::as_str) {
            Some("ATTRIBUTES") => unsafe {
                meshopt::ffi::meshopt_decodeVertexBuffer(
                    out.as_mut_ptr().cast(),
                    count,
                    stride,
                    source.as_ptr(),
                    source.len(),
                )
            },
            Some("TRIANGLES") => unsafe {
                meshopt::ffi::meshopt_decodeIndexBuffer(
                    out.as_mut_ptr().cast(),
                    count,
                    stride,
                    source.as_ptr(),
                    source.len(),
                )
            },
            other => return Err(format!("a compressed view of mode {other:?}")),
        };
        if code != 0 {
            return Err(format!("meshoptimizer can't decode a view (error {code})"));
        }
        bytes.push(Some(out));
    }
    Ok(Views { bytes, strides })
}

pub(crate) fn accessor(g: &Glb, views: &Views, index: usize) -> Result<Accessor, String> {
    let a = list(&g.json, "accessors")
        .get(index)
        .ok_or_else(|| format!("no accessor {index}"))?;
    if a.get("sparse").is_some() {
        return Err("a sparse accessor".into());
    }
    let component = a
        .get("componentType")
        .and_then(J::as_u64)
        .ok_or("an accessor without a component type")?;
    let kind = a
        .get("type")
        .and_then(J::as_str)
        .ok_or("an accessor without a type")?
        .to_string();
    let width = components(&kind)?;
    let size = component_size(component)? * width;
    let count = uint(a, "count").ok_or("an accessor without a count")?;
    let view = uint(a, "bufferView").ok_or("an accessor without a buffer view")?;
    let bytes = views
        .bytes
        .get(view)
        .and_then(Option::as_ref)
        .ok_or("an accessor's buffer view isn't in the file")?;
    let stride = views.strides[view].unwrap_or(size).max(size);
    let start = uint(a, "byteOffset").unwrap_or(0);
    let mut data = Vec::with_capacity(count * size);
    for i in 0..count {
        let at = start + i * stride;
        data.extend_from_slice(
            bytes
                .get(at..at + size)
                .ok_or("an accessor past the end of its buffer view")?,
        );
    }
    Ok(Accessor {
        component,
        normalized: a.get("normalized").and_then(J::as_bool).unwrap_or(false),
        kind,
        count,
        width,
        data,
        min: a.get("min").cloned(),
        max: a.get("max").cloned(),
    })
}
