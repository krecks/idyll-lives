//! What a `.glb` holds, whatever order its exporter wrote it in: a digest that is the same for two files with the
//! same nodes, materials, triangles and clips (`cargo check-glb --against`'s class `same`), and the raw accessors it
//! is made from.
//!
//! Blender's exporter writes the same triangles in a different order on every run (docs/design/objects.md,
//! "Export settings"), so two builds of one model differ in their bytes. Here a triangle is the raw bytes of its
//! three corners' attributes, turned to start at its smallest corner, and a primitive is the sorted list of its
//! triangles: neither the order of the triangles, nor of the vertices, nor of the nodes, materials and clips in
//! the file counts, and neither does how the buffer is laid out. Every bit of a number does.

use super::glb::{Glb, R, int, item, key};
use super::py::J;
use sha2::{Digest as _, Sha256};

pub type Hash = [u8; 32];

/// An accessor's elements as the file has them.
pub struct Raw<'a> {
    bin: &'a [u8],
    at: usize,
    stride: usize,
    /// Bytes per element.
    pub size: usize,
    pub count: usize,
    /// glTF's component type (5126: float).
    pub component: i64,
    /// Components per element.
    pub width: usize,
    pub normalized: bool,
}

/// The accessor `index` of the file, uncompressed and not sparse (the source models are neither).
pub fn raw<'a>(g: &'a Glb, index: &J) -> R<Raw<'a>> {
    let acc = item(&g.gltf, "accessors", index)?;
    if !acc.has("bufferView") || acc.get("sparse").is_some_and(J::truthy) {
        return Err("an accessor without a buffer view, or a sparse one".into());
    }
    let view = item(&g.gltf, "bufferViews", key(acc, "bufferView")?)?;
    if view.get("extensions").is_some_and(J::truthy) {
        return Err("a compressed buffer view".into());
    }
    let kind = key(acc, "type")?;
    let width = match kind.str() {
        Some("SCALAR") => 1,
        Some("VEC2") => 2,
        Some("VEC3") => 3,
        Some("VEC4") | Some("MAT2") => 4,
        Some("MAT3") => 9,
        Some("MAT4") => 16,
        _ => return Err(format!("an accessor of type {}", super::py::repr(kind))),
    };
    let component = int(acc, "componentType", 0)?;
    let bytes = match component {
        5120 | 5121 => 1,
        5122 | 5123 => 2,
        5125 | 5126 => 4,
        other => return Err(format!("an accessor of component type {other}")),
    };
    let size = width * bytes;
    let count = int(acc, "count", 0)?.max(0) as usize;
    let at = int(view, "byteOffset", 0)? + int(acc, "byteOffset", 0)?;
    let stride = match view.get("byteStride").filter(|s| s.truthy()) {
        Some(_) => int(view, "byteStride", 0)?,
        None => size as i64,
    };
    if at < 0 || stride < size as i64 {
        return Err("an accessor outside the file's buffer".into());
    }
    let (at, stride) = (at as usize, stride as usize);
    if count > 0 && at + (count - 1) * stride + size > g.bin.len() {
        return Err(format!(
            "an accessor of {count} elements at {at} reaches past the buffer's {} bytes",
            g.bin.len()
        ));
    }
    Ok(Raw {
        bin: &g.bin,
        at,
        stride,
        size,
        count,
        component,
        width,
        normalized: acc.get("normalized").is_some_and(J::truthy),
    })
}

impl Raw<'_> {
    /// Element `i`'s bytes.
    pub fn element(&self, i: usize) -> &[u8] {
        let at = self.at + i * self.stride;
        &self.bin[at..at + self.size]
    }

    /// Every component as a number (integers as the spec reads them: normalised if flagged), element after element.
    pub fn floats(&self) -> Vec<f64> {
        let step = self.size / self.width;
        let mut out = Vec::with_capacity(self.count * self.width);
        for i in 0..self.count {
            for c in self.element(i).chunks(step) {
                let (v, top) = match self.component {
                    5126 => (f32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64, None),
                    5120 => (c[0] as i8 as f64, Some(127.0)),
                    5121 => (c[0] as f64, Some(255.0)),
                    5122 => (i16::from_le_bytes([c[0], c[1]]) as f64, Some(32767.0)),
                    5123 => (u16::from_le_bytes([c[0], c[1]]) as f64, Some(65535.0)),
                    _ => (u32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64, None),
                };
                out.push(match top {
                    Some(top) if self.normalized => (v / top).max(-1.0),
                    _ => v,
                });
            }
        }
        out
    }

    /// The elements as indices (a scalar accessor of unsigned integers).
    pub fn indices(&self) -> R<Vec<u32>> {
        if self.width != 1 || !matches!(self.component, 5121 | 5123 | 5125) {
            return Err("indices that are no unsigned integers".into());
        }
        Ok((0..self.count)
            .map(|i| {
                let c = self.element(i);
                match self.component {
                    5121 => c[0] as u32,
                    5123 => u16::from_le_bytes([c[0], c[1]]) as u32,
                    _ => u32::from_le_bytes([c[0], c[1], c[2], c[3]]),
                }
            })
            .collect())
    }
}

/// A primitive's indices: its `indices` accessor, or one after another for one without, each a vertex it has.
pub fn indices(g: &Glb, prim: &J, vertices: usize) -> R<Vec<u32>> {
    let idx = match prim.get("indices") {
        Some(i) => raw(g, i)?.indices()?,
        None => (0..vertices as u32).collect(),
    };
    match idx.iter().find(|i| **i as usize >= vertices) {
        Some(i) => Err(format!("index {i} of a primitive with {vertices} vertices")),
        None => Ok(idx),
    }
}

/// The file's nodes by name: each node's path from its scene root (`body/door/handle`), which is what the game
/// and the clips know a part by, and its parent.
pub struct Tree {
    pub paths: Vec<String>,
    pub parents: Vec<Option<usize>>,
}

/// The nodes' paths (`Tree`). A node without a name is `node<index>`; of two children of one parent with one
/// name the later ones are told apart by `#2`, `#3`.
pub fn tree(gltf: &J) -> R<Tree> {
    let nodes = gltf.get("nodes").and_then(J::list).unwrap_or(&[]);
    let mut parents: Vec<Option<usize>> = vec![None; nodes.len()];
    for (i, n) in nodes.iter().enumerate() {
        for c in n.get("children").and_then(J::list).unwrap_or(&[]) {
            match c {
                J::Int(c) if *c >= 0 && (*c as usize) < nodes.len() => {
                    parents[*c as usize] = Some(i);
                }
                _ => return Err(format!("node {i} has a child that is no node of the file")),
            }
        }
    }
    let name = |i: usize| match nodes[i].get("name").and_then(J::str) {
        Some(name) => name.to_string(),
        None => format!("node{i}"),
    };
    // Parents before their children.
    let mut depth = vec![0usize; nodes.len()];
    for (i, d) in depth.iter_mut().enumerate() {
        let mut k = i;
        while let Some(p) = parents[k] {
            *d += 1;
            k = p;
            if *d > nodes.len() {
                return Err("the nodes' children go round in a circle".into());
            }
        }
    }
    let mut order: Vec<usize> = (0..nodes.len()).collect();
    order.sort_by_key(|i| depth[*i]);
    let mut paths = vec![String::new(); nodes.len()];
    let mut taken = std::collections::HashSet::new();
    for k in order {
        let base = match parents[k] {
            Some(p) => format!("{}/{}", paths[p], name(k)),
            None => name(k),
        };
        let mut path = base.clone();
        let mut n = 1;
        while !taken.insert(path.clone()) {
            n += 1;
            path = format!("{base}#{n}");
        }
        paths[k] = path;
    }
    Ok(Tree { paths, parents })
}

/// A JSON value as text that is the same for the same value: keys sorted, a number by its value (1 and 1.0 alike).
pub fn canon(v: &J) -> String {
    match v {
        J::Null => "null".into(),
        J::Bool(b) => b.to_string(),
        J::Int(i) => format!("{:?}", *i as f64),
        J::Float(f) => format!("{f:?}"),
        J::Str(s) => super::py::json_str(s),
        J::Arr(a) => format!("[{}]", a.iter().map(canon).collect::<Vec<_>>().join(",")),
        J::Obj(o) => {
            let mut fields: Vec<String> = o
                .iter()
                .map(|(k, v)| format!("{}:{}", super::py::json_str(k), canon(v)))
                .collect();
            fields.sort();
            format!("{{{}}}", fields.join(","))
        }
    }
}

/// SHA-256 over fields, each with its length before it, so two lists of fields never run together.
struct Hasher(Sha256);

impl Hasher {
    fn new(what: &str) -> Hasher {
        let mut h = Hasher(Sha256::new());
        h.bytes(what.as_bytes());
        h
    }

    fn bytes(&mut self, b: &[u8]) {
        self.0.update((b.len() as u64).to_le_bytes());
        self.0.update(b);
    }

    fn text(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }

    fn done(self) -> Hash {
        self.0.finalize().into()
    }
}

/// The hashes of a list, sorted: the list's own, whatever order it was in.
fn sorted(what: &str, mut hashes: Vec<Hash>) -> Hash {
    hashes.sort();
    let mut h = Hasher::new(what);
    for x in &hashes {
        h.bytes(x);
    }
    h.done()
}

/// Which of a primitive's attributes a digest takes.
#[derive(Clone, Copy, PartialEq)]
enum Take {
    All,
    /// Positions, normals and tangents: the shape.
    Shape,
    /// Positions and texture coordinates.
    Uv,
    /// Positions and vertex colours: the baked shading.
    Shade,
}

impl Take {
    fn takes(self, attribute: &str) -> bool {
        attribute == "POSITION"
            || match self {
                Take::All => true,
                Take::Shape => attribute == "NORMAL" || attribute == "TANGENT",
                Take::Uv => attribute.starts_with("TEXCOORD_"),
                Take::Shade => attribute.starts_with("COLOR_"),
            }
    }
}

/// One primitive: its material's name, its attributes' kinds, and its triangles as a sorted list, each the three
/// corners' bytes (hashed to 16) from its smallest corner on. Anything but triangles is hashed as it is written.
fn primitive(g: &Glb, prim: &J, take: Take) -> R<Hash> {
    let mut h = Hasher::new("primitive");
    let material = match prim.get("material") {
        Some(m) => item(&g.gltf, "materials", m)?
            .get("name")
            .map(J::key)
            .unwrap_or_default(),
        None => String::new(),
    };
    h.text(&material);
    let mode = int(prim, "mode", 4)?;
    h.bytes(&mode.to_le_bytes());
    // (Morph targets, which no model of the game has: how many, so one that gains them is not the same.)
    let targets = prim.get("targets").and_then(J::list).map_or(0, |t| t.len());
    h.bytes(&(targets as u64).to_le_bytes());
    let mut attributes: Vec<(&String, Raw)> = Vec::new();
    for (name, index) in key(prim, "attributes")?.entries().unwrap_or(&[]) {
        if take.takes(name) {
            attributes.push((name, raw(g, index)?));
        }
    }
    attributes.sort_by(|a, b| a.0.cmp(b.0));
    let vertices = attributes.iter().map(|(_, a)| a.count).min().unwrap_or(0);
    for (name, a) in &attributes {
        h.text(name);
        h.bytes(&[
            a.component as u8,
            (a.component >> 8) as u8,
            a.width as u8,
            a.normalized as u8,
        ]);
    }
    let corner: Vec<u128> = (0..vertices)
        .map(|i| {
            let mut v = Sha256::new();
            for (_, a) in &attributes {
                v.update(a.element(i));
            }
            let d: Hash = v.finalize().into();
            u128::from_le_bytes(d[..16].try_into().unwrap())
        })
        .collect();
    let idx = indices(g, prim, vertices)?;
    if mode == 4 {
        let mut triangles: Vec<[u128; 3]> = idx
            .as_chunks::<3>()
            .0
            .iter()
            .map(|t| {
                let [a, b, c] = [
                    corner[t[0] as usize],
                    corner[t[1] as usize],
                    corner[t[2] as usize],
                ];
                [[a, b, c], [b, c, a], [c, a, b]].into_iter().min().unwrap()
            })
            .collect();
        triangles.sort_unstable();
        for t in &triangles {
            h.0.update(bytemuck::cast_slice::<u128, u8>(t));
        }
        h.bytes(&(triangles.len() as u64).to_le_bytes());
    } else {
        for i in &idx {
            h.0.update(corner[*i as usize].to_le_bytes());
        }
        h.bytes(&(idx.len() as u64).to_le_bytes());
    }
    Ok(h.done())
}

/// Every mesh node's primitives (`take`n attributes), by the node's path.
fn meshes(g: &Glb, tree: &Tree, take: Take) -> R<Hash> {
    let nodes = g.gltf.get("nodes").and_then(J::list).unwrap_or(&[]);
    let mut all = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        let Some(mesh) = n.get("mesh") else { continue };
        let prims = key(item(&g.gltf, "meshes", mesh)?, "primitives")?;
        let mut of_node = Vec::new();
        for prim in prims.list().unwrap_or(&[]) {
            of_node.push(primitive(g, prim, take)?);
        }
        let mut h = Hasher::new("mesh");
        h.text(&tree.paths[i]);
        h.bytes(&sorted("primitives", of_node));
        all.push(h.done());
    }
    Ok(sorted("meshes", all))
}

/// The nodes: each one's path (so its name and its parents'), its transform as written, whether it has a mesh,
/// and whatever else it carries; and which of them the scene starts from.
fn nodes(gltf: &J, tree: &Tree) -> R<Hash> {
    let list = gltf.get("nodes").and_then(J::list).unwrap_or(&[]);
    let mut all = Vec::new();
    for (i, n) in list.iter().enumerate() {
        let mut h = Hasher::new("node");
        h.text(&tree.paths[i]);
        let rest: Vec<(String, J)> = n
            .entries()
            .unwrap_or(&[])
            .iter()
            .filter(|(k, _)| !["name", "children", "mesh"].contains(&k.as_str()))
            .cloned()
            .collect();
        h.text(&canon(&J::Obj(rest)));
        h.bytes(&[n.has("mesh") as u8]);
        all.push(h.done());
    }
    let scenes = gltf.get("scenes").and_then(J::list).unwrap_or(&[]);
    let scene = int(gltf, "scene", 0)?;
    let mut roots: Vec<&str> = Vec::new();
    for r in scenes
        .get(scene.max(0) as usize)
        .and_then(|s| s.get("nodes"))
        .and_then(J::list)
        .unwrap_or(&[])
    {
        match r {
            J::Int(r) if *r >= 0 && (*r as usize) < list.len() => {
                roots.push(&tree.paths[*r as usize])
            }
            _ => return Err("the scene names a node the file has not".into()),
        }
    }
    roots.sort();
    let mut h = Hasher::new("nodes");
    h.bytes(&sorted("each", all));
    h.text(&roots.join("\n"));
    Ok(h.done())
}

/// The materials, each with everything it says, and what else of the file is drawn or read: its textures and
/// images, cameras, skins, and the extensions it uses.
fn materials(g: &Glb) -> R<Hash> {
    let mut all = Vec::new();
    for m in g.gltf.get("materials").and_then(J::list).unwrap_or(&[]) {
        let mut h = Hasher::new("material");
        h.text(&canon(m));
        all.push(h.done());
    }
    let mut h = Hasher::new("materials");
    h.bytes(&sorted("each", all));
    for list in ["textures", "samplers", "cameras", "skins", "extensions"] {
        h.text(&g.gltf.get(list).map(canon).unwrap_or_default());
    }
    for list in ["extensionsUsed", "extensionsRequired"] {
        let mut names: Vec<String> = g
            .gltf
            .get(list)
            .and_then(J::list)
            .unwrap_or(&[])
            .iter()
            .map(canon)
            .collect();
        names.sort();
        h.text(&names.join(","));
    }
    for image in g.gltf.get("images").and_then(J::list).unwrap_or(&[]) {
        h.text(&image.get("mimeType").map(canon).unwrap_or_default());
        h.text(&image.get("uri").map(canon).unwrap_or_default());
        if let Some(v) = image.get("bufferView") {
            let view = item(&g.gltf, "bufferViews", v)?;
            let at = int(view, "byteOffset", 0)?.max(0) as usize;
            let end = at + int(view, "byteLength", 0)?.max(0) as usize;
            h.bytes(
                g.bin
                    .get(at..end)
                    .ok_or("an image outside the file's buffer")?,
            );
        }
    }
    Ok(h.done())
}

/// The clips: each one's name and its channels (the node's path, what of it moves, how the keys are joined, and
/// the bytes of the times and the keys), whatever order they are in.
fn clips(g: &Glb, tree: &Tree) -> R<Hash> {
    let mut all = Vec::new();
    for a in g.gltf.get("animations").and_then(J::list).unwrap_or(&[]) {
        let mut channels = Vec::new();
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let sampler = super::glb::index(key(a, "samplers")?, key(c, "sampler")?)?;
            let mut h = Hasher::new("channel");
            let node = match target.get("node") {
                Some(J::Int(i)) if *i >= 0 && (*i as usize) < tree.paths.len() => {
                    &tree.paths[*i as usize]
                }
                Some(_) => return Err("a clip moves a node the file has not".into()),
                None => "",
            };
            h.text(node);
            h.text(&target.get("path").map(canon).unwrap_or_default());
            h.text(
                sampler
                    .get("interpolation")
                    .and_then(J::str)
                    .unwrap_or("LINEAR"),
            );
            for side in ["input", "output"] {
                let a = raw(g, key(sampler, side)?)?;
                h.bytes(&[
                    a.component as u8,
                    (a.component >> 8) as u8,
                    a.width as u8,
                    a.normalized as u8,
                ]);
                for i in 0..a.count {
                    h.0.update(a.element(i));
                }
                h.bytes(&(a.count as u64).to_le_bytes());
            }
            channels.push(h.done());
        }
        let mut h = Hasher::new("clip");
        h.text(&a.get("name").map(J::key).unwrap_or_default());
        h.bytes(&sorted("channels", channels));
        all.push(h.done());
    }
    Ok(sorted("clips", all))
}

/// The digest of a file: equal for two files that hold the same model (the module's note).
pub fn digest(g: &Glb) -> R<Hash> {
    let tree = tree(&g.gltf)?;
    let mut h = Hasher::new("glb");
    h.bytes(&nodes(&g.gltf, &tree)?);
    h.bytes(&materials(g)?);
    h.bytes(&meshes(g, &tree, Take::All)?);
    h.bytes(&clips(g, &tree)?);
    Ok(h.done())
}

/// A file's digest in parts, so a difference between two files can be named.
#[derive(PartialEq)]
pub struct Parts {
    pub nodes: Hash,
    pub materials: Hash,
    /// Positions, normals and tangents.
    pub shape: Hash,
    /// Positions and texture coordinates.
    pub uv: Hash,
    /// Positions and vertex colours (the baked shading).
    pub shade: Hash,
    pub clips: Hash,
}

pub fn parts(g: &Glb) -> R<Parts> {
    let tree = tree(&g.gltf)?;
    Ok(Parts {
        nodes: nodes(&g.gltf, &tree)?,
        materials: materials(g)?,
        shape: meshes(g, &tree, Take::Shape)?,
        uv: meshes(g, &tree, Take::Uv)?,
        shade: meshes(g, &tree, Take::Shade)?,
        clips: clips(g, &tree)?,
    })
}

impl Parts {
    /// What of two files differs, by name; empty when only what no part covers does (an attribute of another kind).
    pub fn differing(&self, other: &Parts) -> Vec<&'static str> {
        [
            ("nodes", self.nodes != other.nodes),
            ("materials", self.materials != other.materials),
            ("shape", self.shape != other.shape),
            ("uv", self.uv != other.uv),
            ("shading", self.shade != other.shade),
            ("clips", self.clips != other.clips),
        ]
        .into_iter()
        .filter(|(_, differs)| *differs)
        .map(|(name, _)| name)
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::glb::parse_glb;
    use super::super::glb::tests::glb;
    use super::super::json::loads_text;
    use super::*;

    fn j(text: &str) -> J {
        loads_text(text.as_bytes()).unwrap()
    }

    #[test]
    fn a_value_reads_the_same_whatever_its_keys_order_and_its_numbers_spelling() {
        assert_eq!(
            canon(&j(r#"{"b": [1, 2.5, "x"], "a": {"d": null, "c": true}}"#)),
            r#"{"a":{"c":true,"d":null},"b":[1.0,2.5,"x"]}"#
        );
        assert_eq!(
            canon(&j(r#"{"a": 1, "b": 2}"#)),
            canon(&j(r#"{"b": 2.0, "a": 1.0}"#))
        );
        assert_ne!(canon(&j("[1, 2]")), canon(&j("[2, 1]")));
    }

    #[test]
    fn nodes_are_known_by_their_paths() {
        let t = tree(&j(r#"{"nodes": [{"name": "handle"}, {"name": "door", "children": [0, 3]}, {"children": [1]},
            {"name": "handle"}, {"name": "door"}]}"#))
        .unwrap();
        assert_eq!(
            t.paths,
            [
                "node2/door/handle",
                "node2/door",
                "node2",
                "node2/door/handle#2",
                "door"
            ]
        );
        assert_eq!(t.parents, [Some(1), Some(2), None, Some(1), None]);
        assert!(tree(&j(r#"{"nodes": [{"children": [1]}, {"children": [0]}]}"#)).is_err());
        assert!(tree(&j(r#"{"nodes": [{"children": [7]}]}"#)).is_err());
    }

    #[test]
    fn accessors_are_read_raw_and_as_numbers() {
        // Two shorts with a stride of 4, a normalised byte pair, and a float.
        let mut bin = vec![7, 0, 0xEE, 0xEE, 9, 0, 0xEE, 0xEE, 255, 0];
        bin.extend([0u8; 2]);
        bin.extend(1.5f32.to_le_bytes());
        let json = r#"{"bufferViews": [{"byteOffset": 0, "byteLength": 8, "byteStride": 4}, {"byteOffset": 8, "byteLength": 2},
            {"byteOffset": 12, "byteLength": 4}, {"byteOffset": 12, "byteLength": 400}],
            "accessors": [{"bufferView": 0, "componentType": 5123, "type": "SCALAR", "count": 2},
            {"bufferView": 1, "componentType": 5121, "normalized": true, "type": "VEC2", "count": 1},
            {"bufferView": 2, "componentType": 5126, "type": "SCALAR", "count": 1},
            {"bufferView": 3, "componentType": 5126, "type": "VEC3", "count": 30},
            {"componentType": 5126, "type": "VEC3", "count": 1}]}"#;
        let g = parse_glb(&glb(json, &bin)).unwrap();
        let read = |i: i64| raw(&g, &J::Int(i));
        let shorts = read(0).unwrap();
        assert_eq!(
            (shorts.element(1), shorts.indices().unwrap()),
            (&[9u8, 0][..], vec![7, 9])
        );
        assert_eq!(read(1).unwrap().floats(), [1.0, 0.0]);
        assert_eq!(read(2).unwrap().floats(), [1.5]);
        assert!(read(2).unwrap().indices().is_err(), "floats are no indices");
        assert!(read(3).err().unwrap().contains("reaches past the buffer"));
        assert!(read(4).is_err(), "no buffer view");
    }
}
