//! Models (`.glb`) made smaller, and read back for the check.
//!
//! - **Quantised geometry** (`KHR_mesh_quantization`): positions to 14 bits of the mesh's box (within a
//!   tenth of a millimetre on a 2 m sofa), normals to 10, texture coordinates within 0..1 to 12, each a 16-bit
//!   integer, not a 32-bit float; the baked shading (COLOR_0, a grey of 32 levels: `gamekit/shade.py`)
//!   to 8 bits, which keep every level. A model with foliage keeps its floats but for the colours (its
//!   leaf cards sit close: quantised, their edges moved).
//! - **A node under each node with a mesh**, named as it is, holds the mesh with the quantisation's
//!   offset and scale: the named nodes keep their own places (the pivots the game takes from them,
//!   `BuildOptions.parts`), the meshes their names. The game bakes each node's place into its vertices.
//! - **Vertices in the order that compresses best** (meshoptimizer's strip order, then first use), and
//!   every buffer view compressed with meshoptimizer (`EXT_meshopt_compression`: lossless on the
//!   quantised data; vertex codec version 0, which every decoder reads). Animations keep their floats.
//!
//! A model this doesn't handle (a skin, morph targets, images of its own, a second buffer) is left as it
//! is: the game loads the source. None of the shipped models is (`cargo assets check` says which).

use serde_json::{Map, Value as J, json};
use std::collections::{BTreeMap, HashMap};

const FLOAT: u64 = 5126;
const BYTE: u64 = 5120;
const UBYTE: u64 = 5121;
const SHORT: u64 = 5122;
const USHORT: u64 = 5123;
const UINT: u64 = 5125;
const ARRAY_BUFFER: u64 = 34962;
const ELEMENT_ARRAY_BUFFER: u64 = 34963;
const MESHOPT: &str = "EXT_meshopt_compression";
const QUANTIZATION: &str = "KHR_mesh_quantization";

/// A `.glb`'s two chunks.
pub struct Glb {
    pub json: J,
    pub bin: Vec<u8>,
}

pub fn parse(data: &[u8]) -> Result<Glb, String> {
    let u32_at = |at: usize| -> Result<usize, String> {
        data.get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
            .ok_or_else(|| "the file ends early".to_string())
    };
    if data.get(..4) != Some(b"glTF") || u32_at(4)? != 2 {
        return Err("not a glTF 2 binary".into());
    }
    let json_len = u32_at(12)?;
    if data.get(16..20) != Some(b"JSON") {
        return Err("its first chunk isn't JSON".into());
    }
    let json_bytes = data
        .get(20..20 + json_len)
        .ok_or("its JSON chunk ends early")?;
    let json: J = serde_json::from_slice(json_bytes).map_err(|e| format!("its JSON: {e}"))?;
    let at = 20 + json_len;
    let bin = if at + 8 <= data.len() {
        let len = u32_at(at)?;
        if data.get(at + 4..at + 8) != Some(b"BIN\0") {
            return Err("its second chunk isn't BIN".into());
        }
        data.get(at + 8..at + 8 + len)
            .ok_or("its BIN chunk ends early")?
            .to_vec()
    } else {
        Vec::new()
    };
    Ok(Glb { json, bin })
}

pub fn write(json: &J, bin: &[u8]) -> Vec<u8> {
    let mut text = serde_json::to_vec(json).expect("JSON");
    while !text.len().is_multiple_of(4) {
        text.push(b' ');
    }
    let bin_len = bin.len().div_ceil(4) * 4;
    let total = 12 + 8 + text.len() + if bin_len > 0 { 8 + bin_len } else { 0 };
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&text);
    if bin_len > 0 {
        out.extend_from_slice(&(bin_len as u32).to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend_from_slice(bin);
        out.resize(total, 0);
    }
    out
}

fn list<'a>(j: &'a J, key: &str) -> &'a [J] {
    j.get(key)
        .and_then(J::as_array)
        .map_or(&[], |a| a.as_slice())
}

fn uint(j: &J, key: &str) -> Option<usize> {
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
struct Accessor {
    component: u64,
    normalized: bool,
    kind: String,
    count: usize,
    width: usize,
    data: Vec<u8>,
    min: Option<J>,
    max: Option<J>,
}

impl Accessor {
    /// Element `i`'s component `c` as the number the spec reads it as (a normalised integer as 0..1 or -1..1).
    fn value(&self, i: usize, c: usize) -> f32 {
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

    fn index(&self, i: usize) -> u32 {
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
struct Views {
    bytes: Vec<Option<Vec<u8>>>,
    strides: Vec<Option<usize>>,
}

fn views(g: &Glb) -> Result<Views, String> {
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

fn accessor(g: &Glb, views: &Views, index: usize) -> Result<Accessor, String> {
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

/// A material named for a finish of leaves (`name@foliage`, `name@leafcards`, or `foliage` itself): its
/// model's geometry keeps its floats, as does one a manifest maps to such a finish.
pub fn foliage_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower == "foliage" {
        return true;
    }
    ["@foliage", "@leafcards"].iter().any(|tag| {
        lower.match_indices(tag).any(|(at, _)| {
            lower[at + tag.len()..]
                .chars()
                .next()
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_'))
        })
    })
}

/// 14 bits of -1..1 in a 16-bit integer (the top bits repeated below, so 1 is 32767).
pub fn quantize14(x: f32) -> i16 {
    let q = (x.clamp(-1.0, 1.0) * 8191.0).round() as i32;
    let m = q.unsigned_abs();
    let e = ((m << 2) | (m >> 11)) as i32;
    (if q < 0 { -e } else { e }) as i16
}

/// 10 bits of -1..1 in a 16-bit integer.
pub fn quantize10(x: f32) -> i16 {
    let q = (x.clamp(-1.0, 1.0) * 511.0).round() as i32;
    let m = q.unsigned_abs();
    let e = ((m << 6) | (m >> 3)) as i32;
    (if q < 0 { -e } else { e }) as i16
}

/// 12 bits of 0..1 in a 16-bit integer.
pub fn quantize12(x: f32) -> u16 {
    let m = (x.clamp(0.0, 1.0) * 4095.0).round() as u32;
    ((m << 4) | (m >> 8)) as u16
}

/// One buffer view to be: its elements, `stride` bytes each.
struct Stream {
    bytes: Vec<u8>,
    stride: usize,
    triangles: bool,
    /// `ARRAY_BUFFER`, `ELEMENT_ARRAY_BUFFER`, or none (an animation's).
    target: Option<u64>,
}

fn compress(stream: &Stream) -> Result<Vec<u8>, String> {
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
struct Out {
    stream: usize,
    offset: usize,
    component: u64,
    normalized: bool,
    kind: String,
    count: usize,
    min: Option<J>,
    max: Option<J>,
}

struct Builder {
    streams: Vec<Stream>,
    accessors: Vec<Out>,
}

impl Builder {
    fn stream(&mut self, bytes: Vec<u8>, stride: usize, target: Option<u64>) -> usize {
        self.streams.push(Stream {
            bytes,
            stride,
            triangles: false,
            target,
        });
        self.streams.len() - 1
    }

    #[allow(clippy::too_many_arguments)]
    fn accessor(
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
fn reordered(a: &Accessor, order: &[u32], stride: usize) -> Vec<u8> {
    let size = a.data.len() / a.count.max(1);
    let mut out = vec![0u8; order.len() * stride];
    for (i, &old) in order.iter().enumerate() {
        let from = old as usize * size;
        out[i * stride..i * stride + size].copy_from_slice(&a.data[from..from + size]);
    }
    out
}

fn bounds_i16(values: &[i16], width: usize) -> (J, J) {
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
fn unsupported(j: &J) -> Option<String> {
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

/// The model made smaller (the module's header), or why it is left as it is.
pub fn optimize(data: &[u8], foliage: bool) -> Result<Vec<u8>, String> {
    let g = parse(data)?;
    if let Some(why) = unsupported(&g.json) {
        return Err(why);
    }
    let v = views(&g)?;
    let foliage = foliage
        || list(&g.json, "materials")
            .iter()
            .any(|m| m.get("name").and_then(J::as_str).is_some_and(foliage_name));
    // (The exporter gives primitives with the same triangles one index accessor: each gets its own here, in
    // its own vertices' new order. Vertex data is never shared; it couldn't be put in two orders.)
    let mut uses: HashMap<usize, usize> = HashMap::new();
    for mesh in list(&g.json, "meshes") {
        for p in list(mesh, "primitives") {
            let attributes = p
                .get("attributes")
                .and_then(J::as_object)
                .ok_or("a primitive without attributes")?;
            for a in attributes.values() {
                *uses
                    .entry(a.as_u64().ok_or("an accessor index that isn't a number")? as usize)
                    .or_default() += 1;
            }
        }
    }
    if uses.values().any(|&n| n > 1) {
        return Err("vertex data two primitives share".into());
    }

    let mut b = Builder {
        streams: Vec::new(),
        accessors: Vec::new(),
    };
    let mut json = g.json.clone();
    let mut meshes = list(&g.json, "meshes").to_vec();
    // Every primitive's triangles, one after another: one view for all, as one stream compresses best.
    let mut indices: Vec<u32> = Vec::new();
    let mut index_accessors: Vec<(usize, usize, usize)> = Vec::new();
    let mut frames: Vec<Option<([f32; 3], f32)>> = Vec::new();
    let mut most_vertices = 0usize;

    for mesh in &mut meshes {
        let prims = mesh
            .get("primitives")
            .and_then(J::as_array)
            .cloned()
            .unwrap_or_default();
        // The mesh's box: one offset and one scale for all its primitives, so they still meet.
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        let mut positions = Vec::new();
        for p in &prims {
            let at = uint(&p["attributes"], "POSITION").ok_or("a primitive without positions")?;
            let a = accessor(&g, &v, at)?;
            if a.width != 3 || a.component != FLOAT {
                return Err("positions that aren't three floats".into());
            }
            for i in 0..a.count {
                for c in 0..3 {
                    lo[c] = lo[c].min(a.value(i, c));
                    hi[c] = hi[c].max(a.value(i, c));
                }
            }
            positions.push(a);
        }
        let frame = (!foliage && !prims.is_empty()).then(|| {
            let centre = [0, 1, 2].map(|c| (lo[c] + hi[c]) * 0.5);
            let half = (0..3).fold(0f32, |m, c| m.max((hi[c] - lo[c]) * 0.5));
            (centre, if half > 0.0 { half } else { 1.0 })
        });
        frames.push(frame);

        let mut out_prims = Vec::new();
        for (p, position) in prims.iter().zip(positions) {
            let count = position.count;
            let attributes = p["attributes"].as_object().expect("checked above");
            let source: Vec<u32> = match uint(p, "indices") {
                Some(i) => {
                    let a = accessor(&g, &v, i)?;
                    (0..a.count).map(|k| a.index(k)).collect()
                }
                None => (0..count as u32).collect(),
            };
            if !source.len().is_multiple_of(3) || source.iter().any(|&i| i as usize >= count) {
                return Err("triangles that don't fit their vertices".into());
            }
            let strip = meshopt::optimize_vertex_cache_strip(&source, count);
            // First use first: the order the triangles reach the vertices in.
            let mut place = vec![u32::MAX; count];
            let mut order: Vec<u32> = Vec::new();
            let triangles: Vec<u32> = strip
                .iter()
                .map(|&old| {
                    if place[old as usize] == u32::MAX {
                        place[old as usize] = order.len() as u32;
                        order.push(old);
                    }
                    place[old as usize]
                })
                .collect();
            let n = order.len();
            most_vertices = most_vertices.max(n);

            let mut out_attributes = Map::new();
            for (name, at) in attributes {
                let a = if name == "POSITION" {
                    position.clone()
                } else {
                    accessor(&g, &v, at.as_u64().expect("checked above") as usize)?
                };
                if a.count != count {
                    return Err(format!("{name} with another count than the positions"));
                }
                let made = if name == "POSITION" {
                    match frame {
                        Some((centre, scale)) => {
                            let mut q = Vec::with_capacity(n * 3);
                            for &old in &order {
                                for (c, centre) in centre.iter().enumerate() {
                                    q.push(quantize14((a.value(old as usize, c) - centre) / scale));
                                }
                            }
                            let (min, max) = bounds_i16(&q, 3);
                            let mut bytes = vec![0u8; n * 8];
                            for (i, v3) in q.as_chunks::<3>().0.iter().enumerate() {
                                for c in 0..3 {
                                    bytes[i * 8 + c * 2..i * 8 + c * 2 + 2]
                                        .copy_from_slice(&v3[c].to_le_bytes());
                                }
                            }
                            let s = b.stream(bytes, 8, Some(ARRAY_BUFFER));
                            b.accessor(s, 0, SHORT, true, "VEC3", n, Some(min), Some(max))
                        }
                        None => {
                            let mut lo = [f32::INFINITY; 3];
                            let mut hi = [f32::NEG_INFINITY; 3];
                            for &old in &order {
                                for c in 0..3 {
                                    lo[c] = lo[c].min(a.value(old as usize, c));
                                    hi[c] = hi[c].max(a.value(old as usize, c));
                                }
                            }
                            let s = b.stream(reordered(&a, &order, 12), 12, Some(ARRAY_BUFFER));
                            b.accessor(
                                s,
                                0,
                                FLOAT,
                                false,
                                "VEC3",
                                n,
                                Some(json!(lo)),
                                Some(json!(hi)),
                            )
                        }
                    }
                } else if name == "NORMAL"
                    && frame.is_some()
                    && a.width == 3
                    && a.component == FLOAT
                {
                    let mut bytes = vec![0u8; n * 8];
                    for (i, &old) in order.iter().enumerate() {
                        for c in 0..3 {
                            bytes[i * 8 + c * 2..i * 8 + c * 2 + 2].copy_from_slice(
                                &quantize10(a.value(old as usize, c)).to_le_bytes(),
                            );
                        }
                    }
                    let s = b.stream(bytes, 8, Some(ARRAY_BUFFER));
                    b.accessor(s, 0, SHORT, true, "VEC3", n, None, None)
                } else if name.starts_with("TEXCOORD_")
                    && frame.is_some()
                    && a.width == 2
                    && a.component == FLOAT
                    && (0..count).all(|i| (0..2).all(|c| (0.0..=1.0).contains(&a.value(i, c))))
                {
                    let mut bytes = vec![0u8; n * 4];
                    for (i, &old) in order.iter().enumerate() {
                        for c in 0..2 {
                            bytes[i * 4 + c * 2..i * 4 + c * 2 + 2].copy_from_slice(
                                &quantize12(a.value(old as usize, c)).to_le_bytes(),
                            );
                        }
                    }
                    let s = b.stream(bytes, 4, Some(ARRAY_BUFFER));
                    b.accessor(s, 0, USHORT, true, "VEC2", n, None, None)
                } else if name.starts_with("COLOR_")
                    && (a.width == 3 || a.width == 4)
                    && (a.component == FLOAT || a.normalized)
                {
                    let mut bytes = vec![0u8; n * 4];
                    for (i, &old) in order.iter().enumerate() {
                        for c in 0..a.width {
                            bytes[i * 4 + c] =
                                (a.value(old as usize, c).clamp(0.0, 1.0) * 255.0).round() as u8;
                        }
                    }
                    let s = b.stream(bytes, 4, Some(ARRAY_BUFFER));
                    b.accessor(s, 0, UBYTE, true, &a.kind, n, None, None)
                } else {
                    let size = a.data.len() / a.count.max(1);
                    let stride = size.div_ceil(4) * 4;
                    let s = b.stream(reordered(&a, &order, stride), stride, Some(ARRAY_BUFFER));
                    b.accessor(
                        s,
                        0,
                        a.component,
                        a.normalized,
                        &a.kind,
                        n,
                        a.min.clone(),
                        a.max.clone(),
                    )
                };
                out_attributes.insert(name.clone(), json!(made));
            }
            // (Its place among the accessors is set once the index stream's width is known.)
            index_accessors.push((indices.len(), triangles.len(), b.accessors.len()));
            b.accessors.push(Out {
                stream: usize::MAX,
                offset: 0,
                component: USHORT,
                normalized: false,
                kind: "SCALAR".into(),
                count: triangles.len(),
                min: None,
                max: None,
            });
            let mut out = p.as_object().cloned().unwrap_or_default();
            out.insert("attributes".into(), J::Object(out_attributes));
            out.insert("indices".into(), json!(b.accessors.len() - 1));
            out.insert("mode".into(), json!(4));
            out_prims.push(J::Object(out));
            indices.extend(triangles);
        }
        mesh["primitives"] = J::Array(out_prims);
    }

    // The triangles: 16-bit unless a primitive has more vertices than that holds.
    if !index_accessors.is_empty() {
        let wide = most_vertices > 65536;
        let stride = if wide { 4 } else { 2 };
        let mut bytes = Vec::with_capacity(indices.len() * stride);
        for &i in &indices {
            if wide {
                bytes.extend_from_slice(&i.to_le_bytes());
            } else {
                bytes.extend_from_slice(&(i as u16).to_le_bytes());
            }
        }
        b.streams.push(Stream {
            bytes,
            stride,
            triangles: true,
            target: Some(ELEMENT_ARRAY_BUFFER),
        });
        let stream = b.streams.len() - 1;
        for &(start, _, at) in &index_accessors {
            b.accessors[at].stream = stream;
            b.accessors[at].offset = start * stride;
            b.accessors[at].component = if wide { UINT } else { USHORT };
        }
    }

    // Animations: their keys and values as they are (an accessor two samplers share stays one).
    let mut animations = list(&g.json, "animations").to_vec();
    let mut moved: BTreeMap<usize, usize> = BTreeMap::new();
    for animation in &mut animations {
        let Some(samplers) = animation.get_mut("samplers").and_then(J::as_array_mut) else {
            continue;
        };
        for sampler in samplers {
            for key in ["input", "output"] {
                let old = uint(sampler, key).ok_or("an animation sampler without its accessors")?;
                let new = match moved.get(&old) {
                    Some(&new) => new,
                    None => {
                        let a = accessor(&g, &v, old)?;
                        let size = a.data.len() / a.count.max(1);
                        if !size.is_multiple_of(4) {
                            return Err("animation data that isn't in whole 4-byte steps".into());
                        }
                        let s = b.stream(a.data.clone(), size, None);
                        let new = b.accessor(
                            s,
                            0,
                            a.component,
                            a.normalized,
                            &a.kind,
                            a.count,
                            a.min.clone(),
                            a.max.clone(),
                        );
                        moved.insert(old, new);
                        new
                    }
                };
                sampler[key] = json!(new);
            }
        }
    }

    // The compressed data, and the file's two buffers: it, and what it decodes to (never loaded).
    let mut bin: Vec<u8> = Vec::new();
    let mut raw = 0usize;
    let mut buffer_views = Vec::new();
    for stream in &b.streams {
        let packed = compress(stream)?;
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        let count = stream.bytes.len() / stream.stride;
        let mut view = Map::new();
        view.insert("buffer".into(), json!(1));
        view.insert("byteOffset".into(), json!(raw));
        view.insert("byteLength".into(), json!(stream.bytes.len()));
        if let Some(target) = stream.target {
            view.insert("target".into(), json!(target));
            if target == ARRAY_BUFFER {
                view.insert("byteStride".into(), json!(stream.stride));
            }
        }
        view.insert(
            "extensions".into(),
            json!({ MESHOPT: {
                "buffer": 0,
                "byteOffset": bin.len(),
                "byteLength": packed.len(),
                "byteStride": stream.stride,
                "count": count,
                "mode": if stream.triangles { "TRIANGLES" } else { "ATTRIBUTES" },
            } }),
        );
        buffer_views.push(J::Object(view));
        bin.extend_from_slice(&packed);
        raw += stream.bytes.len().div_ceil(4) * 4;
    }
    let accessors: Vec<J> = b
        .accessors
        .iter()
        .map(|a| {
            let mut out = Map::new();
            out.insert("bufferView".into(), json!(a.stream));
            out.insert("byteOffset".into(), json!(a.offset));
            out.insert("componentType".into(), json!(a.component));
            if a.normalized {
                out.insert("normalized".into(), json!(true));
            }
            out.insert("count".into(), json!(a.count));
            out.insert("type".into(), json!(a.kind));
            if let Some(min) = &a.min {
                out.insert("min".into(), min.clone());
            }
            if let Some(max) = &a.max {
                out.insert("max".into(), max.clone());
            }
            J::Object(out)
        })
        .collect();

    // A node under each node with a quantised mesh: the mesh, at the quantisation's offset and scale.
    let mut nodes = list(&g.json, "nodes").to_vec();
    let mut quantized = false;
    for i in 0..nodes.len() {
        let Some(mesh) = uint(&nodes[i], "mesh") else {
            continue;
        };
        let Some((centre, scale)) = frames.get(mesh).copied().flatten() else {
            continue;
        };
        quantized = true;
        let mut child = Map::new();
        if let Some(name) = nodes[i].get("name") {
            child.insert("name".into(), name.clone());
        }
        child.insert("mesh".into(), json!(mesh));
        child.insert("translation".into(), json!(centre));
        child.insert("scale".into(), json!([scale, scale, scale]));
        let at = nodes.len();
        nodes.push(J::Object(child));
        let node = nodes[i].as_object_mut().expect("a node is an object");
        node.remove("mesh");
        match node.get_mut("children").and_then(J::as_array_mut) {
            Some(children) => children.push(json!(at)),
            None => {
                node.insert("children".into(), json!([at]));
            }
        }
    }

    let root = json.as_object_mut().ok_or("its JSON isn't an object")?;
    root.insert("accessors".into(), J::Array(accessors));
    root.insert("bufferViews".into(), J::Array(buffer_views));
    root.insert(
        "buffers".into(),
        json!([
            { "byteLength": bin.len() },
            { "byteLength": raw, "extensions": { MESHOPT: { "fallback": true } } },
        ]),
    );
    if !meshes.is_empty() {
        root.insert("meshes".into(), J::Array(meshes));
    }
    if !nodes.is_empty() {
        root.insert("nodes".into(), J::Array(nodes));
    }
    if !animations.is_empty() {
        root.insert("animations".into(), J::Array(animations));
    }
    for key in ["extensionsUsed", "extensionsRequired"] {
        let mut names: Vec<String> = root
            .get(key)
            .and_then(J::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|n| n.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        for name in [MESHOPT, QUANTIZATION] {
            if (name == MESHOPT || quantized) && !names.iter().any(|n| n == name) {
                names.push(name.to_string());
            }
        }
        root.insert(key.into(), json!(names));
    }
    if let Some(asset) = root.get_mut("asset").and_then(J::as_object_mut) {
        let from = asset
            .get("generator")
            .and_then(J::as_str)
            .unwrap_or("unknown")
            .to_string();
        asset.insert(
            "generator".into(),
            json!(format!("cargo assets (from {from})")),
        );
    }
    Ok(write(&json, &bin))
}

// ---- reading a model back, to compare one with another ---------------------------------------

/// A triangle's corner as the game gets it.
#[derive(Clone, Copy, Debug)]
pub struct Corner {
    /// Where it is, in the model's own space (every node's place applied).
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// The baked shading, 8 bits a channel.
    pub color: [u8; 4],
}

pub struct Primitive {
    pub material: Option<usize>,
    /// Three corners a triangle.
    pub corners: Vec<Corner>,
}

pub struct Model {
    /// Per mesh (in the file's order), its primitives.
    pub meshes: Vec<Vec<Primitive>>,
    /// Every animation sampler's keys and values, as the bytes of their accessors, in the file's order.
    pub animations: Vec<Vec<u8>>,
    /// The names of the nodes that carry a name.
    pub nodes: Vec<String>,
    /// Per mesh: half its box's longest side as stored (before any node's place), times the most its
    /// node stretches a length. A 14-bit step of the first is what quantising may move a corner by.
    pub spans: Vec<f32>,
}

type Mat = [[f32; 4]; 3];

fn local(node: &J) -> Mat {
    let get = |key: &str, default: &[f32]| -> Vec<f32> {
        node.get(key)
            .and_then(J::as_array)
            .map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect())
            .unwrap_or_else(|| default.to_vec())
    };
    let t = get("translation", &[0.0, 0.0, 0.0]);
    let q = get("rotation", &[0.0, 0.0, 0.0, 1.0]);
    let s = get("scale", &[1.0, 1.0, 1.0]);
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let r = [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y - z * w),
            2.0 * (x * z + y * w),
        ],
        [
            2.0 * (x * y + z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z - x * w),
        ],
        [
            2.0 * (x * z - y * w),
            2.0 * (y * z + x * w),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ];
    [0, 1, 2].map(|i| [r[i][0] * s[0], r[i][1] * s[1], r[i][2] * s[2], t[i]])
}

fn chain(a: &Mat, b: &Mat) -> Mat {
    [0, 1, 2].map(|i| {
        let mut row = [0f32; 4];
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
        row[3] += a[i][3];
        row
    })
}

/// A model's geometry and animations as a loader reads them (compressed or not).
pub fn read(data: &[u8]) -> Result<Model, String> {
    let g = parse(data)?;
    let v = views(&g)?;
    let nodes = list(&g.json, "nodes");
    // Each mesh's place: its first node's, down from the scene.
    let mut parent = vec![usize::MAX; nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        for child in list(node, "children") {
            if let Some(c) = child.as_u64() {
                parent[c as usize] = i;
            }
        }
    }
    let world = |mut i: usize| -> Mat {
        let mut m = local(&nodes[i]);
        while parent[i] != usize::MAX {
            i = parent[i];
            m = chain(&local(&nodes[i]), &m);
        }
        m
    };
    let mut meshes = Vec::new();
    let mut spans = Vec::new();
    for (index, mesh) in list(&g.json, "meshes").iter().enumerate() {
        let place = nodes
            .iter()
            .position(|n| uint(n, "mesh") == Some(index))
            .map(world)
            .unwrap_or([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ]);
        let mut prims = Vec::new();
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for p in list(mesh, "primitives") {
            let attribute = |name: &str| -> Result<Option<Accessor>, String> {
                uint(&p["attributes"], name)
                    .map(|i| accessor(&g, &v, i))
                    .transpose()
            };
            let position = attribute("POSITION")?.ok_or("a primitive without positions")?;
            let normal = attribute("NORMAL")?;
            let uv = attribute("TEXCOORD_0")?;
            let color = attribute("COLOR_0")?;
            let indices: Vec<u32> = match uint(p, "indices") {
                Some(i) => {
                    let a = accessor(&g, &v, i)?;
                    (0..a.count).map(|k| a.index(k)).collect()
                }
                None => (0..position.count as u32).collect(),
            };
            let mut corners = Vec::with_capacity(indices.len());
            for &i in &indices {
                let i = i as usize;
                if i >= position.count {
                    return Err("a triangle past its vertices".into());
                }
                let at = [0, 1, 2].map(|c| position.value(i, c));
                for c in 0..3 {
                    lo[c] = lo[c].min(at[c]);
                    hi[c] = hi[c].max(at[c]);
                }
                corners.push(Corner {
                    position: [0, 1, 2].map(|r| {
                        place[r][0] * at[0]
                            + place[r][1] * at[1]
                            + place[r][2] * at[2]
                            + place[r][3]
                    }),
                    normal: normal
                        .as_ref()
                        .map_or([0.0; 3], |n| [0, 1, 2].map(|c| n.value(i, c))),
                    uv: uv
                        .as_ref()
                        .map_or([0.0; 2], |u| [0, 1].map(|c| u.value(i, c))),
                    color: color.as_ref().map_or([255; 4], |k| {
                        [0, 1, 2, 3].map(|c| {
                            if c < k.width {
                                (k.value(i, c).clamp(0.0, 1.0) * 255.0).round() as u8
                            } else {
                                255
                            }
                        })
                    }),
                });
            }
            prims.push(Primitive {
                material: uint(p, "material"),
                corners,
            });
        }
        meshes.push(prims);
        let half = (0..3).fold(0f32, |m, c| m.max((hi[c] - lo[c]) * 0.5));
        let stretch = (0..3).fold(0f32, |m, c| {
            m.max(
                (place[0][c] * place[0][c] + place[1][c] * place[1][c] + place[2][c] * place[2][c])
                    .sqrt(),
            )
        });
        spans.push(if half.is_finite() {
            half * stretch
        } else {
            0.0
        });
    }
    let mut animations = Vec::new();
    for animation in list(&g.json, "animations") {
        for sampler in list(animation, "samplers") {
            for key in ["input", "output"] {
                let a = accessor(
                    &g,
                    &v,
                    uint(sampler, key).ok_or("an animation sampler without its accessors")?,
                )?;
                animations.push(a.data);
            }
        }
    }
    Ok(Model {
        meshes,
        animations,
        nodes: nodes
            .iter()
            .filter_map(|n| n.get("name").and_then(J::as_str).map(String::from))
            .collect(),
        spans,
    })
}

/// How far a copy is from its source at most: positions (m), normals and texture coordinates (a
/// component's difference).
#[derive(Clone, Copy, Debug, Default)]
pub struct Drift {
    pub position: f32,
    pub normal: f32,
    pub uv: f32,
    pub triangles: usize,
}

/// Compares a copy with its source: the same meshes, primitives and materials, every triangle of the
/// source once in the copy (in any order, started at any corner) with its corners within the
/// quantisation's steps and its shading the same, and the animations the same to the bit.
pub fn compare(source: &Model, copy: &Model) -> Result<Drift, String> {
    if source.meshes.len() != copy.meshes.len() {
        return Err(format!(
            "{} meshes, the source has {}",
            copy.meshes.len(),
            source.meshes.len()
        ));
    }
    if source.animations != copy.animations {
        return Err("its animations differ from the source's".into());
    }
    let mut names = source.nodes.clone();
    names.sort();
    let mut copied = copy.nodes.clone();
    copied.sort();
    copied.dedup();
    names.dedup();
    if names != copied {
        return Err("its nodes' names differ from the source's".into());
    }
    let mut drift = Drift::default();
    for (m, (a, b)) in source.meshes.iter().zip(&copy.meshes).enumerate() {
        if a.len() != b.len() {
            return Err(format!(
                "mesh {m}: {} primitives, the source has {}",
                b.len(),
                a.len()
            ));
        }
        // What 14 bits of the mesh's box allow: a step (half of one from rounding, a quarter from the top bits
        // repeated below, the rest for the floats' own rounding on the way through the nodes). The box is the
        // mesh's own, as stored: a node may squash it (a stream of water flat until it runs), and the step
        // with it only along the squash.
        let reach = source.spans.get(m).copied().unwrap_or(0.0) / 8191.0 + 2e-6;
        for (p, (a, b)) in a.iter().zip(b).enumerate() {
            let at = format!("mesh {m}, primitive {p}");
            if a.material != b.material {
                return Err(format!("{at}: another material than the source's"));
            }
            if a.corners.len() != b.corners.len() {
                return Err(format!(
                    "{at}: {} triangles, the source has {}",
                    b.corners.len() / 3,
                    a.corners.len() / 3
                ));
            }
            // The source's triangles by the cell their middle is in; each of the copy's takes the one it fits
            // best. Where two of the source's are within a step of each other (a sliver, a face made twice)
            // one may be taken by the wrong one of the copy's: its taker then moves on to another it fits.
            let cell = (reach * 4.0).max(1e-5);
            let key = |t: &[Corner]| -> [i64; 3] {
                [0, 1, 2].map(|k| {
                    ((t[0].position[k] + t[1].position[k] + t[2].position[k]) / 3.0 / cell).floor()
                        as i64
                })
            };
            let mut cells: HashMap<[i64; 3], Vec<usize>> = HashMap::new();
            for (i, t) in a.corners.as_chunks::<3>().0.iter().enumerate() {
                cells.entry(key(t)).or_default().push(i);
            }
            let off =
                |u: &[f32], w: &[f32]| u.iter().zip(w).fold(0f32, |m, (p, q)| m.max((p - q).abs()));
            // How far the source's triangle `i` is from `t`, started at the corner where it is nearest (`fits`: within the steps, shaded the same).
            let apart = |i: usize, t: &[Corner]| -> (Drift, bool) {
                let s = &a.corners[i * 3..i * 3 + 3];
                let mut best: Option<(Drift, bool)> = None;
                for turn in 0..3 {
                    let mut d = Drift::default();
                    let mut shaded = true;
                    for c in 0..3 {
                        let (x, y) = (&s[(c + turn) % 3], &t[c]);
                        d.position = d.position.max(off(&x.position, &y.position));
                        d.normal = d.normal.max(off(&x.normal, &y.normal));
                        d.uv = d.uv.max(off(&x.uv, &y.uv));
                        shaded &= x.color == y.color;
                    }
                    let fits =
                        shaded && d.position <= reach && d.normal <= 0.0025 && d.uv <= 0.0003;
                    if best
                        .as_ref()
                        .is_none_or(|(b, f)| (fits, -d.position) > (*f, -b.position))
                    {
                        best = Some((d, fits));
                    }
                }
                best.expect("three turns")
            };
            // The source's triangles `t` fits, the nearest first.
            let fitting = |t: &[Corner]| -> Vec<(usize, Drift)> {
                let k = key(t);
                let mut out = Vec::new();
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            for &i in cells
                                .get(&[k[0] + dx, k[1] + dy, k[2] + dz])
                                .map_or(&[][..], Vec::as_slice)
                            {
                                let (d, fits) = apart(i, t);
                                if fits {
                                    out.push((i, d));
                                }
                            }
                        }
                    }
                }
                out.sort_by(|x, y| x.1.position.total_cmp(&y.1.position));
                out
            };
            const FREE: usize = usize::MAX;
            let mut taker = vec![FREE; a.corners.len() / 3];
            for (n, t) in b.corners.as_chunks::<3>().0.iter().enumerate() {
                let fits = fitting(t);
                let mut took = fits
                    .iter()
                    .find(|(i, _)| taker[*i] == FREE)
                    .map(|&(i, d)| (i, d));
                if took.is_none() {
                    'swap: for &(i, d) in &fits {
                        let other = taker[i];
                        for (j, _) in fitting(&b.corners[other * 3..other * 3 + 3]) {
                            if taker[j] == FREE {
                                taker[j] = other;
                                took = Some((i, d));
                                break 'swap;
                            }
                        }
                    }
                }
                let Some((i, worst)) = took else {
                    // The source's nearest triangle, and how it differs.
                    let nearest = (0..a.corners.len() / 3)
                        .map(|i| (i, apart(i, t).0))
                        .min_by(|x, y| x.1.position.total_cmp(&y.1.position));
                    let said = nearest.map_or(String::new(), |(i, d)| {
                        format!(
                            ": the nearest is {:.6} m off, its normals {:.4}, its texture coordinates {:.5}, its shading {}{}",
                            d.position,
                            d.normal,
                            d.uv,
                            if (0..3).any(|turn| (0..3).all(|c| a.corners[i * 3 + (c + turn) % 3].color == t[c].color)) { "the same" } else { "another" },
                            if taker[i] == FREE { "" } else { ", and another triangle of the copy has taken it" }
                        )
                    });
                    return Err(format!(
                        "{at}: a triangle at {:?} that the source doesn't have (within {reach:.6} m){said}",
                        t[0].position
                    ));
                };
                taker[i] = n;
                drift.position = drift.position.max(worst.position);
                drift.normal = drift.normal.max(worst.normal);
                drift.uv = drift.uv.max(worst.uv);
                drift.triangles += 1;
            }
        }
    }
    Ok(drift)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantised_values_fill_their_sixteen_bits() {
        assert_eq!(quantize14(1.0), 32767);
        assert_eq!(quantize14(-1.0), -32767);
        assert_eq!(quantize14(0.0), 0);
        // (glTF-Transform's own, which the game's models were made with until this tool: 1800 -> 7200, 7651 -> 30607.)
        assert_eq!(quantize14(1800.0 / 8191.0), 7200);
        assert_eq!(quantize14(7651.0 / 8191.0), 30607);
        assert_eq!(quantize10(1.0), 32767);
        assert_eq!(quantize10(-1.0), -32767);
        assert_eq!(quantize12(1.0), 65535);
        assert_eq!(quantize12(0.0), 0);
        // Read back as the spec says (value / 32767), a value is within half a step and what repeating the
        // top bits adds (under one sixteen-bit unit): three quarters of a 14-bit step at most.
        for i in -1000..=1000 {
            let x = i as f32 / 1000.0;
            assert!((quantize14(x) as f32 / 32767.0 - x).abs() <= 0.75 / 8191.0 + 1e-6);
            assert!((quantize10(x) as f32 / 32767.0 - x).abs() <= 0.5 / 511.0 + 4e-5);
            let u = (x + 1.0) / 2.0;
            assert!((quantize12(u) as f32 / 65535.0 - u).abs() <= 0.5 / 4095.0 + 2e-5);
        }
    }

    #[test]
    fn foliage_by_material_name() {
        assert!(foliage_name("leaves@foliage"));
        assert!(foliage_name("crown@leafcards"));
        assert!(foliage_name("Foliage"));
        assert!(foliage_name("a@foliage.dark"));
        assert!(!foliage_name("a@foliageish"));
        assert!(!foliage_name("trunk@bark"));
    }

    /// A quad of two triangles under a moved node, with a turn over two keys.
    fn sample(material: &str) -> Vec<u8> {
        let positions: [[f32; 3]; 4] = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.5, 0.0],
            [0.0, 0.5, 0.25],
        ];
        let normals: [[f32; 3]; 4] = [[0.0, 0.0, 1.0]; 4];
        let uvs: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let colors: [[u16; 4]; 4] = [
            [65535, 65535, 65535, 65535],
            [32896, 32896, 32896, 65535],
            [0, 0, 0, 65535],
            [8224, 8224, 8224, 65535],
        ];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let times: [f32; 2] = [0.0, 1.0];
        let turns: [[f32; 4]; 2] = [[0.0, 0.0, 0.0, 1.0], [0.0, 0.70710677, 0.0, 0.70710677]];
        let mut bin: Vec<u8> = Vec::new();
        let mut views = Vec::new();
        let mut push = |bytes: Vec<u8>, bin: &mut Vec<u8>| {
            while !bin.len().is_multiple_of(4) {
                bin.push(0);
            }
            views.push(json!({ "buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len() }));
            bin.extend(bytes);
        };
        push(
            positions
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            &mut bin,
        );
        push(
            normals
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            &mut bin,
        );
        push(
            uvs.iter().flatten().flat_map(|v| v.to_le_bytes()).collect(),
            &mut bin,
        );
        push(
            colors
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            &mut bin,
        );
        push(
            indices.iter().flat_map(|v| v.to_le_bytes()).collect(),
            &mut bin,
        );
        push(
            times.iter().flat_map(|v| v.to_le_bytes()).collect(),
            &mut bin,
        );
        push(
            turns
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            &mut bin,
        );
        let json = json!({
            "asset": { "version": "2.0", "generator": "test" },
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": [{ "name": "door", "mesh": 0, "translation": [2.0, 0.0, -1.0] }],
            "materials": [{ "name": material }],
            "meshes": [{ "name": "door", "primitives": [{ "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2, "COLOR_0": 3 }, "indices": 4, "material": 0 }] }],
            "animations": [{ "name": "open", "channels": [{ "sampler": 0, "target": { "node": 0, "path": "rotation" } }], "samplers": [{ "input": 5, "output": 6, "interpolation": "LINEAR" }] }],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": 4, "type": "VEC3", "min": [0.0, 0.0, 0.0], "max": [1.0, 0.5, 0.25] },
                { "bufferView": 1, "componentType": 5126, "count": 4, "type": "VEC3" },
                { "bufferView": 2, "componentType": 5126, "count": 4, "type": "VEC2" },
                { "bufferView": 3, "componentType": 5123, "normalized": true, "count": 4, "type": "VEC4" },
                { "bufferView": 4, "componentType": 5123, "count": 6, "type": "SCALAR" },
                { "bufferView": 5, "componentType": 5126, "count": 2, "type": "SCALAR", "min": [0.0], "max": [1.0] },
                { "bufferView": 6, "componentType": 5126, "count": 2, "type": "VEC4" },
            ],
            "bufferViews": views,
            "buffers": [{ "byteLength": bin.len() }],
        });
        write(&json, &bin)
    }

    #[test]
    fn a_model_comes_back_as_it_was_within_the_steps() {
        let source = sample("paint");
        let small = optimize(&source, false).unwrap();
        let g = parse(&small).unwrap();
        assert_eq!(g.json["extensionsRequired"], json!([MESHOPT, QUANTIZATION]));
        // The named node keeps its place and gets a node under it with the mesh.
        assert_eq!(g.json["nodes"][0]["translation"], json!([2.0, 0.0, -1.0]));
        assert!(g.json["nodes"][0].get("mesh").is_none());
        assert_eq!(g.json["nodes"][1]["mesh"], json!(0));
        assert_eq!(g.json["nodes"][1]["name"], json!("door"));
        let drift = compare(&read(&source).unwrap(), &read(&small).unwrap()).unwrap();
        assert_eq!(drift.triangles, 2);
        assert!(drift.position <= 0.5 * 0.75 / 8191.0 + 1e-6, "{drift:?}");
        // A corner in the model's space: the node's place is in it.
        let back = read(&small).unwrap();
        assert!(
            back.meshes[0][0]
                .corners
                .iter()
                .any(|c| (c.position[0] - 3.0).abs() < 1e-4)
        );
    }

    #[test]
    fn a_model_with_foliage_keeps_its_floats() {
        let source = sample("crown@leafcards");
        let small = optimize(&source, false).unwrap();
        let g = parse(&small).unwrap();
        assert_eq!(g.json["extensionsRequired"], json!([MESHOPT]));
        assert_eq!(g.json["nodes"][0]["mesh"], json!(0));
        let drift = compare(&read(&source).unwrap(), &read(&small).unwrap()).unwrap();
        assert_eq!((drift.position, drift.normal, drift.uv), (0.0, 0.0, 0.0));
    }

    #[test]
    fn a_change_is_found() {
        let source = read(&sample("paint")).unwrap();
        let mut moved = read(&optimize(&sample("paint"), false).unwrap()).unwrap();
        moved.meshes[0][0].corners[0].position[1] += 0.01;
        assert!(compare(&source, &moved).is_err());
        let mut shaded = read(&optimize(&sample("paint"), false).unwrap()).unwrap();
        shaded.meshes[0][0].corners[0].color[0] ^= 1;
        assert!(compare(&source, &shaded).is_err());
        let mut keyed = read(&optimize(&sample("paint"), false).unwrap()).unwrap();
        keyed.animations[1][0] ^= 1;
        assert!(compare(&source, &keyed).is_err());
    }

    #[test]
    fn what_it_does_not_handle_is_left() {
        let mut g = parse(&sample("paint")).unwrap();
        g.json["skins"] = json!([{ "joints": [0] }]);
        assert_eq!(
            optimize(&write(&g.json, &g.bin), false).unwrap_err(),
            "a skin"
        );
    }
}
