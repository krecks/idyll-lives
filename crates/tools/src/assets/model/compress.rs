//! Compressing a stream, and the builder that writes one.

use super::*;

/// One buffer view to be: its elements, `stride` bytes each.
pub(crate) struct Stream {
    pub(crate) bytes: Vec<u8>,
    pub(crate) stride: usize,
    pub(crate) triangles: bool,
    /// `ARRAY_BUFFER`, `ELEMENT_ARRAY_BUFFER`, or none (an animation's).
    pub(crate) target: Option<u64>,
}

pub(crate) fn compress(stream: &Stream) -> Result<Vec<u8>, String> {
    let count = stream.bytes.len() / stream.stride;
    if stream.triangles {
        let indices: Vec<u32> = (0..count)
            .map(|i| {
                let b = &stream.bytes[i * stream.stride..];
                if stream.stride == 2 {
                    u16::from_le_bytes([b[0], b[1]]) as u32
                } else {
                    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
                }
            })
            .collect();
        let vertices = indices.iter().copied().max().map_or(0, |m| m as usize + 1);
        let out = meshopt::encode_index_buffer(&indices, vertices)
            .map_err(|e| format!("meshoptimizer can't encode the triangles: {e}"))?;
        if out.is_empty() && count > 0 {
            return Err("meshoptimizer can't encode the triangles".into());
        }
        return Ok(out);
    }
    let bound = unsafe { meshopt::ffi::meshopt_encodeVertexBufferBound(count, stream.stride) };
    let mut out = vec![0u8; bound];
    // (Level 2, the encoder's default; version 0, which every decoder reads.)
    let size = unsafe {
        meshopt::ffi::meshopt_encodeVertexBufferLevel(
            out.as_mut_ptr(),
            out.len(),
            stream.bytes.as_ptr().cast(),
            count,
            stream.stride,
            2,
            0,
        )
    };
    if size == 0 && count > 0 {
        return Err("meshoptimizer can't encode a vertex stream".into());
    }
    out.truncate(size);
    Ok(out)
}

/// An accessor to be, on stream `stream`.
pub(crate) struct Out {
    pub(crate) stream: usize,
    pub(crate) offset: usize,
    pub(crate) component: u64,
    pub(crate) normalized: bool,
    pub(crate) kind: String,
    pub(crate) count: usize,
    pub(crate) min: Option<J>,
    pub(crate) max: Option<J>,
}

pub(crate) struct Builder {
    pub(crate) streams: Vec<Stream>,
    pub(crate) accessors: Vec<Out>,
}

impl Builder {
    pub(crate) fn stream(&mut self, bytes: Vec<u8>, stride: usize, target: Option<u64>) -> usize {
        self.streams.push(Stream {
            bytes,
            stride,
            triangles: false,
            target,
        });
        self.streams.len() - 1
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn accessor(
        &mut self,
        stream: usize,
        offset: usize,
        component: u64,
        normalized: bool,
        kind: &str,
        count: usize,
        min: Option<J>,
        max: Option<J>,
    ) -> usize {
        self.accessors.push(Out {
            stream,
            offset,
            component,
            normalized,
            kind: kind.to_string(),
            count,
            min,
            max,
        });
        self.accessors.len() - 1
    }
}

/// `a`'s elements in `order` (new place -> old vertex), each padded to `stride` bytes.
pub(crate) fn reordered(a: &Accessor, order: &[u32], stride: usize) -> Vec<u8> {
    let size = a.data.len() / a.count.max(1);
    let mut out = vec![0u8; order.len() * stride];
    for (i, &old) in order.iter().enumerate() {
        let from = old as usize * size;
        out[i * stride..i * stride + size].copy_from_slice(&a.data[from..from + size]);
    }
    out
}

pub(crate) fn bounds_i16(values: &[i16], width: usize) -> (J, J) {
    let mut lo = vec![i16::MAX; width];
    let mut hi = vec![i16::MIN; width];
    for v in values.chunks_exact(width) {
        for c in 0..width {
            lo[c] = lo[c].min(v[c]);
            hi[c] = hi[c].max(v[c]);
        }
    }
    (json!(lo), json!(hi))
}

/// Why a model is left as it is, if it is.
pub(crate) fn unsupported(j: &J) -> Option<String> {
    if list(j, "buffers").len() != 1 {
        return Some(format!("{} buffers", list(j, "buffers").len()));
    }
    if list(j, "buffers")[0].get("uri").is_some() {
        return Some("a buffer outside the file".into());
    }
    for (key, what) in [
        ("skins", "a skin"),
        ("images", "images of its own"),
        ("cameras", "a camera"),
    ] {
        if !list(j, key).is_empty() {
            return Some(what.into());
        }
    }
    if list(j, "extensionsUsed")
        .iter()
        .any(|e| e.as_str() == Some(MESHOPT))
    {
        return Some("already compressed".into());
    }
    for mesh in list(j, "meshes") {
        if mesh.get("weights").is_some() {
            return Some("morph targets".into());
        }
        for p in list(mesh, "primitives") {
            if p.get("targets").is_some() {
                return Some("morph targets".into());
            }
            if p.get("mode").and_then(J::as_u64).unwrap_or(4) != 4 {
                return Some("a primitive that isn't triangles".into());
            }
            if p.get("extensions").is_some() {
                return Some("a primitive with an extension".into());
            }
        }
    }
    if list(j, "nodes").iter().any(|n| n.get("matrix").is_some()) {
        return Some("a node placed by a matrix".into());
    }
    None
}
