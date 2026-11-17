//! Writing a model out: the .glb (docs/design/objects.md, "Export settings"), laid out as the files the game
//! has always been given: +Y up (Blender -Y is the game's +Z front), a node a part with its name and origin, a
//! primitive a material with POSITION, NORMAL, TEXCOORD_0 (only where a textured finish is used) and COLOR_0 (the
//! baked shading), materials as a name and a base colour, clips as animations from 0 s with a key a frame.

use crate::json::{Json, float_repr};
use crate::materials::Finishes;
use crate::math::{V3, cos, sin};
use crate::motion::{self, Path};
use crate::normals::corner_normals;
use crate::scene::{self, FPS, Obj};
use std::collections::BTreeMap;

const FLOAT: u32 = 5126;
const USHORT: u32 = 5123;
const UINT: u32 = 5125;
const ARRAY_BUFFER: u32 = 34962;
const ELEMENT_ARRAY_BUFFER: u32 = 34963;

/// A number as the files have them: whole ones without a point, the rest as Python writes a float.
fn num(x: f64) -> String {
    if x == x.trunc() && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        float_repr(x)
    }
}

fn nums(xs: &[f32]) -> String {
    let parts: Vec<String> = xs.iter().map(|&x| num(x as f64)).collect();
    format!("[{}]", parts.join(","))
}

#[derive(Default)]
struct File {
    bin: Vec<u8>,
    accessors: Vec<String>,
    views: Vec<String>,
    /// The clips' accessors by their bytes: channels keyed alike share one.
    shared: BTreeMap<(Vec<u8>, &'static str), usize>,
}

impl File {
    fn view(&mut self, bytes: &[u8], target: Option<u32>) -> usize {
        while !self.bin.len().is_multiple_of(4) {
            self.bin.push(0);
        }
        let offset = self.bin.len();
        self.bin.extend_from_slice(bytes);
        let target = target
            .map(|t| format!(",\"target\":{t}"))
            .unwrap_or_default();
        self.views.push(format!(
            "{{\"buffer\":0,\"byteLength\":{},\"byteOffset\":{offset}{target}}}",
            bytes.len()
        ));
        self.views.len() - 1
    }

    fn floats(
        &mut self,
        values: &[f32],
        kind: &'static str,
        per: usize,
        bounds: bool,
        target: Option<u32>,
    ) -> usize {
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        if target.is_none()
            && let Some(&i) = self.shared.get(&(bytes.clone(), kind))
        {
            return i;
        }
        let view = self.view(&bytes, target);
        let range = if bounds {
            let (mut lo, mut hi) = (vec![f32::INFINITY; per], vec![f32::NEG_INFINITY; per]);
            for v in values.chunks(per) {
                for i in 0..per {
                    lo[i] = lo[i].min(v[i]);
                    hi[i] = hi[i].max(v[i]);
                }
            }
            format!(",\"max\":{},\"min\":{}", nums(&hi), nums(&lo))
        } else {
            String::new()
        };
        self.accessors.push(format!(
            "{{\"bufferView\":{view},\"componentType\":{FLOAT},\"count\":{}{range},\"type\":\"{kind}\"}}",
            values.len() / per
        ));
        if target.is_none() {
            self.shared.insert((bytes, kind), self.accessors.len() - 1);
        }
        self.accessors.len() - 1
    }
}

/// Blender (x, y, z) as the file's (x, z, -y).
fn y_up(v: V3) -> [f32; 3] {
    [v.x, v.z, -v.y]
}

/// A normal as the files carry them: to four decimals, then a unit vector again (corners that all but agree
/// share a vertex).
fn rounded_normal(n: V3) -> [f32; 3] {
    let r = y_up(n).map(|x| (x * 10000.0).round_ties_even() / 10000.0);
    let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    if len == 0.0 { r } else { r.map(|x| x / len) }
}

/// A turn (Euler XYZ, radians) as the file's quaternion (x, y, z, w), +Y up.
fn quat_y_up(rot: V3) -> [f32; 4] {
    let (hx, hy, hz) = (rot.x as f64 / 2.0, rot.y as f64 / 2.0, rot.z as f64 / 2.0);
    let (cx, sx, cy, sy, cz, sz) = (cos(hx), sin(hx), cos(hy), sin(hy), cos(hz), sin(hz));
    let w = cx * cy * cz + sx * sy * sz;
    let x = sx * cy * cz - cx * sy * sz;
    let y = cx * sy * cz + sx * cy * sz;
    let z = cx * cy * sz - sx * sy * cz;
    [x as f32, z as f32, -y as f32, w as f32]
}

/// An order for floats that sorts them as numbers.
fn key(x: f32) -> i64 {
    let b = x.to_bits() as i64;
    if b < 0x8000_0000 {
        b
    } else {
        -(b - 0x8000_0000)
    }
}

/// The .glb of `objs` as they stand in the scene (shaded: `shade::bake_ao`; their UVs made: `uv::ensure`).
/// `texcoords`: whether the UVs are written (a model with a part on a textured finish). `finishes`: the game's,
/// for each material's roughness and metalness.
pub fn write(objs: &[Obj], texcoords: bool, finishes: &Finishes) -> Vec<u8> {
    let mut file = File::default();
    let mut materials: Vec<(String, String)> = Vec::new();
    let mut meshes = Vec::new();
    let mut nodes = Vec::new();
    let index_of: BTreeMap<Obj, usize> = objs.iter().enumerate().map(|(i, &o)| (o, i)).collect();
    let mut children: BTreeMap<Obj, Vec<usize>> = BTreeMap::new();
    let mut roots = Vec::new();
    for (i, &o) in objs.iter().enumerate() {
        match o.read(|p| p.parent).filter(|p| index_of.contains_key(p)) {
            Some(parent) => children.entry(parent).or_default().push(i),
            None => roots.push(i),
        }
    }
    for (i, &o) in objs.iter().enumerate() {
        let (name, mesh, loc, rot, scale, quat) = o.read(|p| {
            (
                p.name.clone(),
                p.mesh.clone(),
                p.loc,
                p.rot,
                p.scale,
                p.quat,
            )
        });
        let normals = corner_normals(&mesh);
        let tris = mesh.triangles();
        let mut primitives = Vec::new();
        for (slot, mat) in mesh.mats.iter().enumerate() {
            // this material's corners, each (vertex, normal, uv) once, in the order of its vertices
            let mut corners: BTreeMap<(u32, [i64; 3], [i64; 2]), u32> = BTreeMap::new();
            let mut wanted = Vec::new();
            for (f, t) in &tris {
                let face = mesh.faces[*f as usize];
                if face.mat as usize != slot {
                    continue;
                }
                for &c in t {
                    let l = (face.start + c) as usize;
                    let n = rounded_normal(normals[l]);
                    let uv = if texcoords {
                        mesh.uvs.as_ref().map(|u| u[l]).unwrap_or([0.0, 0.0])
                    } else {
                        [0.0, 0.0]
                    };
                    let k = (mesh.loops[l], n.map(key), uv.map(key));
                    corners.entry(k).or_insert(l as u32);
                    wanted.push(k);
                }
            }
            if wanted.is_empty() {
                continue;
            }
            let order: BTreeMap<(u32, [i64; 3], [i64; 2]), u32> = corners
                .keys()
                .enumerate()
                .map(|(i, k)| (*k, i as u32))
                .collect();
            let (mut position, mut normal, mut uvs, mut colour) =
                (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for &l in corners.values() {
                let l = l as usize;
                let v = mesh.loops[l] as usize;
                position.extend(y_up(mesh.verts[v]));
                normal.extend(rounded_normal(normals[l]));
                let uv = mesh.uvs.as_ref().map(|u| u[l]).unwrap_or([0.0, 0.0]);
                uvs.extend([uv[0], 1.0 - uv[1]]);
                let a = mesh.ao.as_ref().map(|ao| ao[v]).unwrap_or(1.0) as f64;
                let grey = (a * 65535.0).round_ties_even() as u16;
                colour.extend([grey, grey, grey, 65535]);
            }
            let count = corners.len();
            let mut attributes = Vec::new();
            attributes.push(format!(
                "\"POSITION\":{}",
                file.floats(&position, "VEC3", 3, true, Some(ARRAY_BUFFER))
            ));
            attributes.push(format!(
                "\"NORMAL\":{}",
                file.floats(&normal, "VEC3", 3, false, Some(ARRAY_BUFFER))
            ));
            if texcoords {
                attributes.push(format!(
                    "\"TEXCOORD_0\":{}",
                    file.floats(&uvs, "VEC2", 2, false, Some(ARRAY_BUFFER))
                ));
            }
            let bytes: Vec<u8> = colour.iter().flat_map(|c| c.to_le_bytes()).collect();
            let view = file.view(&bytes, Some(ARRAY_BUFFER));
            file.accessors.push(format!(
                "{{\"bufferView\":{view},\"componentType\":{USHORT},\"count\":{count},\"normalized\":true,\"type\":\"VEC4\"}}"
            ));
            attributes.push(format!("\"COLOR_0\":{}", file.accessors.len() - 1));
            let indices: Vec<u32> = wanted.iter().map(|k| order[k]).collect();
            let (bytes, kind): (Vec<u8>, u32) = if count < 65535 {
                (
                    indices
                        .iter()
                        .flat_map(|&i| (i as u16).to_le_bytes())
                        .collect(),
                    USHORT,
                )
            } else {
                (indices.iter().flat_map(|i| i.to_le_bytes()).collect(), UINT)
            };
            let view = file.view(&bytes, Some(ELEMENT_ARRAY_BUFFER));
            file.accessors.push(format!(
                "{{\"bufferView\":{view},\"componentType\":{kind},\"count\":{},\"type\":\"SCALAR\"}}",
                indices.len()
            ));
            let material = match materials.iter().position(|(n, _)| n == mat.name) {
                Some(m) => m,
                None => {
                    let finish = finishes.get(mat.finish);
                    let rough = mat
                        .extra
                        .roughness
                        .or(finish.map(|f| f.roughness))
                        .unwrap_or(0.6) as f32;
                    let metal = mat
                        .extra
                        .metallic
                        .or(finish.map(|f| f.metallic))
                        .unwrap_or(0.0) as f32;
                    let rgb = mat.linear().map(|c| c as f32);
                    let mut pbr = Vec::new();
                    if rgb != [1.0, 1.0, 1.0] {
                        pbr.push(format!(
                            "\"baseColorFactor\":{}",
                            nums(&[rgb[0], rgb[1], rgb[2], 1.0])
                        ));
                    }
                    if metal != 1.0 {
                        pbr.push(format!("\"metallicFactor\":{}", num(metal as f64)));
                    }
                    if rough != 1.0 {
                        pbr.push(format!("\"roughnessFactor\":{}", num(rough as f64)));
                    }
                    let pbr = if pbr.is_empty() {
                        String::new()
                    } else {
                        format!(",\"pbrMetallicRoughness\":{{{}}}", pbr.join(","))
                    };
                    materials.push((
                        mat.name.to_string(),
                        format!("{{\"name\":{}{pbr}}}", quoted(mat.name)),
                    ));
                    materials.len() - 1
                }
            };
            primitives.push(format!(
                "{{\"attributes\":{{{}}},\"indices\":{},\"material\":{material}}}",
                attributes.join(","),
                file.accessors.len() - 1
            ));
        }
        meshes.push(format!(
            "{{\"name\":{},\"primitives\":[{}]}}",
            quoted(&name),
            primitives.join(",")
        ));
        let mut node = Vec::new();
        if let Some(kids) = children.get(&o) {
            let kids: Vec<String> = kids.iter().map(|k| k.to_string()).collect();
            node.push(format!("\"children\":[{}]", kids.join(",")));
        }
        node.push(format!("\"mesh\":{i}"));
        node.push(format!("\"name\":{}", quoted(&name)));
        let turn = match quat {
            Some(q) => [q.x, q.z, -q.y, q.w],
            None => quat_y_up(rot),
        };
        if turn != [0.0, 0.0, 0.0, 1.0] {
            node.push(format!("\"rotation\":{}", nums(&turn)));
        }
        if scale != crate::math::v3(1, 1, 1) {
            node.push(format!("\"scale\":{}", nums(&[scale.x, scale.z, scale.y])));
        }
        if loc != V3::ZERO {
            node.push(format!("\"translation\":{}", nums(&y_up(loc))));
        }
        nodes.push(format!("{{{}}}", node.join(",")));
    }
    // the clips: one animation a name, in the order they are first met
    let mut animations: Vec<(String, Vec<String>, Vec<String>)> = Vec::new();
    for (i, &o) in objs.iter().enumerate() {
        let tracks = o.read(|p| p.tracks.clone());
        for track in tracks {
            let at = match animations.iter().position(|(n, _, _)| *n == track.name) {
                Some(a) => a,
                None => {
                    animations.push((track.name.clone(), Vec::new(), Vec::new()));
                    animations.len() - 1
                }
            };
            let frames: Vec<i32> = (1..=track.end).collect();
            let times: Vec<f32> = frames
                .iter()
                .map(|&f| ((f - 1) as f64 / FPS) as f32)
                .collect();
            let mut paths: Vec<Path> = track.channels.iter().map(|c| c.path).collect();
            paths.sort();
            paths.dedup();
            for path in paths {
                let mut values = Vec::new();
                for &f in &frames {
                    let (loc, rot, scale) = motion::pose(o, Some(&track.name), f as f64);
                    match path {
                        Path::Location => values.extend(y_up(loc)),
                        Path::Rotation => values.extend(quat_y_up(rot)),
                        Path::Scale => values.extend([scale.x, scale.z, scale.y]),
                    }
                }
                let (name, kind, per) = match path {
                    Path::Location => ("translation", "VEC3", 3),
                    Path::Rotation => ("rotation", "VEC4", 4),
                    Path::Scale => ("scale", "VEC3", 3),
                };
                let input = file.floats(&times, "SCALAR", 1, true, None);
                let output = file.floats(&values, kind, per, false, None);
                let (_, channels, samplers) = &mut animations[at];
                channels.push(format!(
                    "{{\"sampler\":{},\"target\":{{\"node\":{i},\"path\":\"{name}\"}}}}",
                    samplers.len()
                ));
                samplers.push(format!(
                    "{{\"input\":{input},\"interpolation\":\"LINEAR\",\"output\":{output}}}"
                ));
            }
        }
    }
    while !file.bin.len().is_multiple_of(4) {
        file.bin.push(0);
    }
    let roots: Vec<String> = roots.iter().map(|r| r.to_string()).collect();
    let mut json = String::from(
        "{\"asset\":{\"generator\":\"idyll-lives artkit\",\"version\":\"2.0\"},\"scene\":0,",
    );
    json.push_str(&format!(
        "\"scenes\":[{{\"name\":\"Scene\",\"nodes\":[{}]}}],",
        roots.join(",")
    ));
    json.push_str(&format!("\"nodes\":[{}],", nodes.join(",")));
    if !animations.is_empty() {
        let all: Vec<String> = animations
            .iter()
            .map(|(name, channels, samplers)| {
                format!(
                    "{{\"channels\":[{}],\"name\":{},\"samplers\":[{}]}}",
                    channels.join(","),
                    quoted(name),
                    samplers.join(",")
                )
            })
            .collect();
        json.push_str(&format!("\"animations\":[{}],", all.join(",")));
    }
    let mats: Vec<&str> = materials.iter().map(|(_, m)| m.as_str()).collect();
    json.push_str(&format!("\"materials\":[{}],", mats.join(",")));
    json.push_str(&format!("\"meshes\":[{}],", meshes.join(",")));
    json.push_str(&format!("\"accessors\":[{}],", file.accessors.join(",")));
    json.push_str(&format!("\"bufferViews\":[{}],", file.views.join(",")));
    json.push_str(&format!(
        "\"buffers\":[{{\"byteLength\":{}}}]}}",
        file.bin.len()
    ));
    while !json.len().is_multiple_of(4) {
        json.push(' ');
    }
    let total = 12 + 8 + json.len() + 8 + file.bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend(b"glTF");
    out.extend(2u32.to_le_bytes());
    out.extend((total as u32).to_le_bytes());
    out.extend((json.len() as u32).to_le_bytes());
    out.extend(b"JSON");
    out.extend(json.as_bytes());
    out.extend((file.bin.len() as u32).to_le_bytes());
    out.extend(b"BIN\0");
    out.extend(&file.bin);
    out
}

fn quoted(s: &str) -> String {
    crate::json::Json::Str(s.to_string()).dumps()
}

/// The names of the materials on `objs`.
pub fn materials_used(objs: &[Obj]) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for &o in objs {
        o.read(|p| out.extend(p.mesh.mats.iter().map(|m| m.name)));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// The scene's triangles.
pub fn triangles() -> usize {
    scene::meshes()
        .iter()
        .map(|o| o.read(|p| p.mesh.triangles().len()))
        .sum()
}

// Reading a .glb back: any file the game is given (Blender's or the writer's above), as a model in the file's own
// axes (+Y up, the front +Z, metres), for the pictures (`preview`) and the gate.

/// What a channel of a clip moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Translation,
    Rotation,
    Scale,
}

impl Target {
    fn name(self) -> &'static str {
        match self {
            Target::Translation => "translation",
            Target::Rotation => "rotation",
            Target::Scale => "scale",
        }
    }

    /// Numbers a key.
    pub fn width(self) -> usize {
        if self == Target::Rotation { 4 } else { 3 }
    }
}

/// How a channel goes from one key to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpolation {
    Linear,
    Step,
    /// Each key an in-tangent, its value and an out-tangent.
    CubicSpline,
}

impl Interpolation {
    fn name(self) -> &'static str {
        match self {
            Interpolation::Linear => "LINEAR",
            Interpolation::Step => "STEP",
            Interpolation::CubicSpline => "CUBICSPLINE",
        }
    }
}

/// One part's place, turn or scale over a clip: its keys' times (s) and values, `target.width()` a key.
#[derive(Clone, Debug, PartialEq)]
pub struct Channel {
    pub node: usize,
    pub target: Target,
    pub interpolation: Interpolation,
    pub times: Vec<f32>,
    pub values: Vec<f32>,
}

/// A clip.
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    pub name: String,
    pub channels: Vec<Channel>,
}

impl Animation {
    /// Its length: its last key's time (s).
    pub fn length(&self) -> f32 {
        self.channels
            .iter()
            .filter_map(|c| c.times.last().copied())
            .fold(0.0, f32::max)
    }
}

/// A material: its name (the pack maps it to a finish) and its factors (glTF's defaults where the file has none).
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub name: String,
    /// Linear RGBA.
    pub base: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub double_sided: bool,
    /// `alphaMode` MASK: drawn where the texture's alpha is at least this.
    pub cutoff: Option<f32>,
}

/// The triangles of one material on a mesh.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Primitive {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Option<Vec<[f32; 2]>>,
    /// COLOR_0, linear RGBA: the baked shading (grey).
    pub colours: Option<Vec<[f32; 4]>>,
    /// Three a triangle.
    pub indices: Vec<u32>,
    pub material: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Primitive>,
}

/// A part: its name, mesh and children, and where it stands in its parent at rest.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub name: String,
    pub mesh: Option<usize>,
    pub children: Vec<usize>,
    pub translation: [f32; 3],
    /// A quaternion (x, y, z, w).
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    /// The clips (animations) that move it, in the order their keys lie in the file (the order of its tracks):
    /// what `encode` lays them out by.
    pub clips: Vec<usize>,
}

impl Default for Node {
    fn default() -> Node {
        Node {
            name: String::new(),
            mesh: None,
            children: Vec::new(),
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
            clips: Vec::new(),
        }
    }
}

/// A model as a .glb has it: the scene's root parts, the parts, meshes, materials and clips.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub generator: String,
    pub roots: Vec<usize>,
    pub nodes: Vec<Node>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub animations: Vec<Animation>,
}

/// A transform as three rows of a 4×4 matrix (the fourth is 0 0 0 1): a point p goes to (row · (p, 1)).
pub type Affine = [[f32; 4]; 3];

pub const IDENTITY: Affine = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
];

/// The matrix product a·b: `b`, then `a`.
pub fn compose(a: &Affine, b: &Affine) -> Affine {
    let mut out = [[0.0; 4]; 3];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, v) in row.iter_mut().enumerate() {
            *v = a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c];
        }
        row[3] += a[r][3];
    }
    out
}

/// A point through `m`.
pub fn apply(m: &Affine, p: [f32; 3]) -> [f32; 3] {
    m.map(|r| r[0] * p[0] + r[1] * p[1] + r[2] * p[2] + r[3])
}

/// A normal through `m` (its inverse transpose: right under a scale that differs by axis), unit length.
pub fn apply_normal(m: &Affine, n: [f32; 3]) -> [f32; 3] {
    let a = |r: usize, c: usize| m[r][c];
    // the cofactors of the 3×3 part: its inverse transpose times its determinant
    let cof = [
        [
            a(1, 1) * a(2, 2) - a(1, 2) * a(2, 1),
            a(1, 2) * a(2, 0) - a(1, 0) * a(2, 2),
            a(1, 0) * a(2, 1) - a(1, 1) * a(2, 0),
        ],
        [
            a(0, 2) * a(2, 1) - a(0, 1) * a(2, 2),
            a(0, 0) * a(2, 2) - a(0, 2) * a(2, 0),
            a(0, 1) * a(2, 0) - a(0, 0) * a(2, 1),
        ],
        [
            a(0, 1) * a(1, 2) - a(0, 2) * a(1, 1),
            a(0, 2) * a(1, 0) - a(0, 0) * a(1, 2),
            a(0, 0) * a(1, 1) - a(0, 1) * a(1, 0),
        ],
    ];
    let det = a(0, 0) * cof[0][0] + a(0, 1) * cof[0][1] + a(0, 2) * cof[0][2];
    let sign = if det < 0.0 { -1.0 } else { 1.0 };
    let v = cof.map(|r| (r[0] * n[0] + r[1] * n[1] + r[2] * n[2]) * sign);
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len == 0.0 { v } else { v.map(|x| x / len) }
}

/// Translation, rotation (x, y, z, w) and scale as a matrix: T·R·S.
pub fn trs(t: [f32; 3], q: [f32; 4], s: [f32; 3]) -> Affine {
    let [x, y, z, w] = q;
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
    let mut out = [[0.0; 4]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = r[i][j] * s[j];
        }
        out[i][3] = t[i];
    }
    out
}

/// A clip at a time: (animation, seconds from its start); None, the rest pose.
pub type Pose = Option<(usize, f32)>;

impl Model {
    /// The animation named `clip`.
    pub fn clip(&self, clip: &str) -> Option<usize> {
        self.animations.iter().position(|a| a.name == clip)
    }

    /// Node `n`'s translation, rotation and scale in `pose` (its rest where the clip doesn't move it).
    pub fn local(&self, n: usize, pose: Pose) -> ([f32; 3], [f32; 4], [f32; 3]) {
        let node = &self.nodes[n];
        let (mut t, mut r, mut s) = (node.translation, node.rotation, node.scale);
        if let Some((a, time)) = pose
            && let Some(anim) = self.animations.get(a)
        {
            for c in anim.channels.iter().filter(|c| c.node == n) {
                let v = sample(c, time);
                match c.target {
                    Target::Translation => t = [v[0], v[1], v[2]],
                    Target::Rotation => r = [v[0], v[1], v[2], v[3]],
                    Target::Scale => s = [v[0], v[1], v[2]],
                }
            }
        }
        (t, r, s)
    }

    /// The nodes the scene draws, parents first.
    pub fn drawn(&self) -> Vec<usize> {
        let mut out = Vec::new();
        let mut seen = vec![false; self.nodes.len()];
        let mut todo: Vec<usize> = self.roots.iter().rev().copied().collect();
        while let Some(n) = todo.pop() {
            if n >= self.nodes.len() || seen[n] {
                continue;
            }
            seen[n] = true;
            out.push(n);
            todo.extend(self.nodes[n].children.iter().rev());
        }
        out
    }

    /// Each node's place in the model in `pose` (identity for one the scene doesn't reach).
    pub fn world(&self, pose: Pose) -> Vec<Affine> {
        let mut out = vec![IDENTITY; self.nodes.len()];
        let mut parent = vec![None; self.nodes.len()];
        for (n, node) in self.nodes.iter().enumerate() {
            for &c in &node.children {
                parent[c] = Some(n);
            }
        }
        for n in self.drawn() {
            let (t, r, s) = self.local(n, pose);
            let own = trs(t, r, s);
            out[n] = match parent[n] {
                Some(p) => compose(&out[p], &own),
                None => own,
            };
        }
        out
    }
}

/// A channel's value at `time` (s): held before its first key and after its last; between two keys a straight
/// line (a turn: the quaternions' normalised blend, the keys being a frame apart).
pub fn sample(c: &Channel, time: f32) -> Vec<f32> {
    let w = c.target.width();
    let (stride, at) = match c.interpolation {
        Interpolation::CubicSpline => (3 * w, w),
        _ => (w, 0),
    };
    let key = |i: usize| &c.values[i * stride + at..i * stride + at + w];
    let n = c.times.len().min(c.values.len() / stride);
    if n == 0 {
        return vec![0.0; w];
    }
    if time <= c.times[0] {
        return key(0).to_vec();
    }
    if time >= c.times[n - 1] {
        return key(n - 1).to_vec();
    }
    let i = c.times[..n].partition_point(|&t| t <= time) - 1;
    let (a, b) = (key(i), key(i + 1));
    if c.interpolation == Interpolation::Step {
        return a.to_vec();
    }
    let u = (time - c.times[i]) / (c.times[i + 1] - c.times[i]);
    if c.target == Target::Rotation {
        let dot: f32 = (0..4).map(|k| a[k] * b[k]).sum();
        let sign = if dot < 0.0 { -1.0 } else { 1.0 };
        let q: Vec<f32> = (0..4).map(|k| a[k] + (b[k] * sign - a[k]) * u).collect();
        let len = q.iter().map(|x| x * x).sum::<f32>().sqrt();
        return q.iter().map(|x| x / len).collect();
    }
    (0..w).map(|k| a[k] + (b[k] - a[k]) * u).collect()
}

/// The JSON and the binary chunk of a .glb.
fn chunks(data: &[u8]) -> Result<(Json, &[u8]), String> {
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

fn int(v: Option<&Json>) -> Option<usize> {
    match v {
        Some(Json::Int(i)) if *i >= 0 => Some(*i as usize),
        _ => None,
    }
}

fn float(v: &Json) -> Option<f32> {
    match v {
        Json::Int(i) => Some(*i as f32),
        Json::Num(x) => Some(*x as f32),
        _ => None,
    }
}

fn floats<const N: usize>(v: Option<&Json>, or: [f32; N]) -> Result<[f32; N], String> {
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

fn list<'a>(j: &'a Json, key: &str) -> &'a [Json] {
    match j.get(key) {
        Some(Json::Arr(items)) => items,
        _ => &[],
    }
}

/// Where an accessor's elements lie in the buffer.
struct Layout {
    /// glTF's component type
    kind: usize,
    /// a component's bytes
    size: usize,
    /// numbers an element
    per: usize,
    count: usize,
    /// (first byte, bytes from one element to the next); none without a buffer view (all zeros)
    place: Option<(usize, usize)>,
}

fn layout(j: &Json, index: usize) -> Result<Layout, String> {
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
    let start = int(view.get("byteOffset")).unwrap_or(0) + int(acc.get("byteOffset")).unwrap_or(0);
    let stride = int(view.get("byteStride")).unwrap_or(size * per);
    Ok(Layout {
        kind,
        size,
        per,
        count,
        place: Some((start, stride)),
    })
}

/// An accessor's values as floats (normalised integers scaled to 0..1 or -1..1), and how many make one element.
fn accessor(j: &Json, bin: &[u8], index: usize) -> Result<(Vec<f32>, usize), String> {
    let Layout {
        kind,
        size,
        per,
        count,
        place,
    } = layout(j, index)?;
    let normalized = list(j, "accessors")[index].get("normalized") == Some(&Json::Bool(true));
    let Some((start, stride)) = place else {
        return Ok((vec![0.0; count * per], per));
    };
    let mut out = Vec::with_capacity(count * per);
    for e in 0..count {
        for k in 0..per {
            let at = start + e * stride + k * size;
            let b = bin
                .get(at..at + size)
                .ok_or_else(|| format!("accessor {index} runs past the buffer"))?;
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
    } = layout(j, index)?;
    if per != 1 || !matches!(kind, 5121 | 5123 | 5125) {
        return Err(format!("accessor {index}: indices of type {kind}"));
    }
    let Some((start, stride)) = place else {
        return Ok(vec![0; count]);
    };
    (0..count)
        .map(|e| {
            let at = start + e * stride;
            let b = bin
                .get(at..at + size)
                .ok_or_else(|| format!("accessor {index} runs past the buffer"))?;
            Ok(match size {
                1 => b[0] as u32,
                2 => u16::from_le_bytes([b[0], b[1]]) as u32,
                _ => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            })
        })
        .collect()
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

fn primitive(j: &Json, bin: &[u8], p: &Json, mi: usize) -> Result<Primitive, String> {
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

/// Reads a .glb: its nodes, meshes (positions, normals, UVs, COLOR_0 and indices of each primitive), materials
/// and clips. Triangles only, in the file's own buffer; a part placed by a matrix, a sparse accessor or another
/// buffer is refused (no file the game has uses one).
pub fn read(data: &[u8]) -> Result<Model, String> {
    let (j, bin) = chunks(data)?;
    let generator = j
        .get("asset")
        .and_then(|a| a.get("generator"))
        .and_then(Json::str)
        .unwrap_or_default()
        .to_string();
    let name = |v: &Json| {
        v.get("name")
            .and_then(Json::str)
            .unwrap_or_default()
            .to_string()
    };
    let mut materials = Vec::new();
    for m in list(&j, "materials") {
        let pbr = m.get("pbrMetallicRoughness");
        let factor = |k: &str| pbr.and_then(|p| p.get(k)).and_then(float);
        materials.push(Material {
            name: name(m),
            base: floats(pbr.and_then(|p| p.get("baseColorFactor")), [1.0; 4])?,
            metallic: factor("metallicFactor").unwrap_or(1.0),
            roughness: factor("roughnessFactor").unwrap_or(1.0),
            double_sided: m.get("doubleSided") == Some(&Json::Bool(true)),
            cutoff: (m.get("alphaMode").and_then(Json::str) == Some("MASK"))
                .then(|| m.get("alphaCutoff").and_then(float).unwrap_or(0.5)),
        });
    }
    let mut meshes = Vec::new();
    for (mi, m) in list(&j, "meshes").iter().enumerate() {
        let primitives = list(m, "primitives")
            .iter()
            .map(|p| primitive(&j, bin, p, mi))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(bad) = primitives
            .iter()
            .filter_map(|p| p.material)
            .find(|&x| x >= materials.len())
        {
            return Err(format!("mesh {mi}: no material {bad}"));
        }
        meshes.push(Mesh {
            name: name(m),
            primitives,
        });
    }
    let mut nodes = Vec::new();
    for (ni, n) in list(&j, "nodes").iter().enumerate() {
        if n.get("matrix").is_some() {
            return Err(format!("node {ni} is placed by a matrix: not read"));
        }
        let mesh = int(n.get("mesh"));
        if let Some(m) = mesh.filter(|&m| m >= meshes.len()) {
            return Err(format!("node {ni}: no mesh {m}"));
        }
        nodes.push(Node {
            name: name(n),
            mesh,
            children: list(n, "children")
                .iter()
                .filter_map(|c| int(Some(c)))
                .collect(),
            translation: floats(n.get("translation"), [0.0; 3])?,
            rotation: floats(n.get("rotation"), [0.0, 0.0, 0.0, 1.0])?,
            scale: floats(n.get("scale"), [1.0; 3])?,
            clips: Vec::new(),
        });
    }
    if let Some(bad) = nodes
        .iter()
        .flat_map(|n| &n.children)
        .find(|&&c| c >= nodes.len())
    {
        return Err(format!("no node {bad}"));
    }
    let scene = int(j.get("scene")).unwrap_or(0);
    let roots: Vec<usize> = match list(&j, "scenes").get(scene) {
        Some(s) => list(s, "nodes")
            .iter()
            .filter_map(|n| int(Some(n)))
            .collect(),
        None => (0..nodes.len()).collect(),
    };
    if let Some(bad) = roots.iter().find(|&&r| r >= nodes.len()) {
        return Err(format!("the scene has no node {bad}"));
    }
    // (node, animation) by the first of its keys' accessors: the order its clips' keys lie in the file
    let mut first: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut animations = Vec::new();
    for (ai, a) in list(&j, "animations").iter().enumerate() {
        let samplers = list(a, "samplers");
        let mut channels = Vec::new();
        for c in list(a, "channels") {
            let target = c.get("target");
            let Some(node) = int(target.and_then(|t| t.get("node"))) else {
                continue;
            };
            if node >= nodes.len() {
                return Err(format!("animation {ai}: no node {node}"));
            }
            let target = match target.and_then(|t| t.get("path")).and_then(Json::str) {
                Some("translation") => Target::Translation,
                Some("rotation") => Target::Rotation,
                Some("scale") => Target::Scale,
                // (morph weights: no file has them)
                _ => continue,
            };
            let s = int(c.get("sampler"))
                .and_then(|s| samplers.get(s))
                .ok_or_else(|| format!("animation {ai}: a channel without its sampler"))?;
            let (Some(input), Some(output)) = (int(s.get("input")), int(s.get("output"))) else {
                return Err(format!("animation {ai}: a sampler without keys"));
            };
            let interpolation = match s.get("interpolation").and_then(Json::str) {
                None | Some("LINEAR") => Interpolation::Linear,
                Some("STEP") => Interpolation::Step,
                Some("CUBICSPLINE") => Interpolation::CubicSpline,
                Some(other) => return Err(format!("animation {ai}: interpolation {other}")),
            };
            let at = first.entry((node, ai)).or_insert(usize::MAX);
            *at = (*at).min(input.min(output));
            let (times, _) = accessor(&j, bin, input)?;
            let (values, per) = accessor(&j, bin, output)?;
            if per != target.width() {
                return Err(format!(
                    "animation {ai}: {per} numbers a key for a {}",
                    target.name()
                ));
            }
            channels.push(Channel {
                node,
                target,
                interpolation,
                times,
                values,
            });
        }
        animations.push(Animation {
            name: name(a),
            channels,
        });
    }
    for (n, node) in nodes.iter_mut().enumerate() {
        let mut clips: Vec<(usize, usize)> = first
            .iter()
            .filter(|((m, _), _)| *m == n)
            .map(|((_, a), &at)| (at, *a))
            .collect();
        clips.sort_unstable();
        node.clips = clips.into_iter().map(|(_, a)| a).collect();
    }
    Ok(Model {
        generator,
        roots,
        nodes,
        meshes,
        materials,
        animations,
    })
}

fn material_json(m: &Material) -> String {
    let mut pbr = Vec::new();
    if m.base[..3] != [1.0, 1.0, 1.0] {
        pbr.push(format!("\"baseColorFactor\":{}", nums(&m.base)));
    }
    if m.metallic != 1.0 {
        pbr.push(format!("\"metallicFactor\":{}", num(m.metallic as f64)));
    }
    if m.roughness != 1.0 {
        pbr.push(format!("\"roughnessFactor\":{}", num(m.roughness as f64)));
    }
    let mut out = format!("{{\"name\":{}", quoted(&m.name));
    if !pbr.is_empty() {
        out.push_str(&format!(",\"pbrMetallicRoughness\":{{{}}}", pbr.join(",")));
    }
    if m.double_sided {
        out.push_str(",\"doubleSided\":true");
    }
    if let Some(cut) = m.cutoff {
        out.push_str(&format!(
            ",\"alphaMode\":\"MASK\",\"alphaCutoff\":{}",
            num(cut as f64)
        ));
    }
    out.push('}');
    out
}

fn primitive_json(file: &mut File, p: &Primitive) -> String {
    let flat = |v: &[[f32; 3]]| -> Vec<f32> { v.iter().flatten().copied().collect() };
    let mut attributes = Vec::new();
    attributes.push(format!(
        "\"POSITION\":{}",
        file.floats(&flat(&p.positions), "VEC3", 3, true, Some(ARRAY_BUFFER))
    ));
    if !p.normals.is_empty() {
        attributes.push(format!(
            "\"NORMAL\":{}",
            file.floats(&flat(&p.normals), "VEC3", 3, false, Some(ARRAY_BUFFER))
        ));
    }
    if let Some(uvs) = &p.uvs {
        let uvs: Vec<f32> = uvs.iter().flatten().copied().collect();
        attributes.push(format!(
            "\"TEXCOORD_0\":{}",
            file.floats(&uvs, "VEC2", 2, false, Some(ARRAY_BUFFER))
        ));
    }
    if let Some(colours) = &p.colours {
        let bytes: Vec<u8> = colours
            .iter()
            .flatten()
            .flat_map(|&c| {
                let c = (c.clamp(0.0, 1.0) as f64 * 65535.0).round_ties_even() as u16;
                c.to_le_bytes()
            })
            .collect();
        let view = file.view(&bytes, Some(ARRAY_BUFFER));
        file.accessors.push(format!(
            "{{\"bufferView\":{view},\"componentType\":{USHORT},\"count\":{},\"normalized\":true,\"type\":\"VEC4\"}}",
            colours.len()
        ));
        attributes.push(format!("\"COLOR_0\":{}", file.accessors.len() - 1));
    }
    let (bytes, kind): (Vec<u8>, u32) = if p.positions.len() < 65535 {
        (
            p.indices
                .iter()
                .flat_map(|&i| (i as u16).to_le_bytes())
                .collect(),
            USHORT,
        )
    } else {
        (
            p.indices.iter().flat_map(|i| i.to_le_bytes()).collect(),
            UINT,
        )
    };
    let view = file.view(&bytes, Some(ELEMENT_ARRAY_BUFFER));
    file.accessors.push(format!(
        "{{\"bufferView\":{view},\"componentType\":{kind},\"count\":{},\"type\":\"SCALAR\"}}",
        p.indices.len()
    ));
    let material = p
        .material
        .map(|m| format!(",\"material\":{m}"))
        .unwrap_or_default();
    format!(
        "{{\"attributes\":{{{}}},\"indices\":{}{material}}}",
        attributes.join(","),
        file.accessors.len() - 1
    )
}

fn node_json(n: &Node) -> String {
    let mut node = Vec::new();
    if !n.children.is_empty() {
        let kids: Vec<String> = n.children.iter().map(|k| k.to_string()).collect();
        node.push(format!("\"children\":[{}]", kids.join(",")));
    }
    if let Some(m) = n.mesh {
        node.push(format!("\"mesh\":{m}"));
    }
    node.push(format!("\"name\":{}", quoted(&n.name)));
    if n.rotation != [0.0, 0.0, 0.0, 1.0] {
        node.push(format!("\"rotation\":{}", nums(&n.rotation)));
    }
    if n.scale != [1.0; 3] {
        node.push(format!("\"scale\":{}", nums(&n.scale)));
    }
    if n.translation != [0.0; 3] {
        node.push(format!("\"translation\":{}", nums(&n.translation)));
    }
    format!("{{{}}}", node.join(","))
}

/// Writes `model` as a .glb laid out as `write` lays one out, so that a file `write` made, read, is written again to
/// the same bytes. The clips' keys go into the buffer a node at a time, its clips in the order `Node::clips` gives
/// (then the model's), its channels by what they move; each clip lists its channels in its own order.
pub fn encode(model: &Model) -> Vec<u8> {
    let mut file = File::default();
    let meshes: Vec<String> = model
        .meshes
        .iter()
        .map(|mesh| {
            let primitives: Vec<String> = mesh
                .primitives
                .iter()
                .map(|p| primitive_json(&mut file, p))
                .collect();
            format!(
                "{{\"name\":{},\"primitives\":[{}]}}",
                quoted(&mesh.name),
                primitives.join(",")
            )
        })
        .collect();
    let nodes: Vec<String> = model.nodes.iter().map(node_json).collect();
    // the keys' accessors (input, output) of each channel of each clip
    let mut keys: Vec<Vec<(usize, usize)>> = model
        .animations
        .iter()
        .map(|a| vec![(0, 0); a.channels.len()])
        .collect();
    for (i, n) in model.nodes.iter().enumerate() {
        let mut order: Vec<usize> = n.clips.clone();
        for (a, anim) in model.animations.iter().enumerate() {
            if !order.contains(&a) && anim.channels.iter().any(|c| c.node == i) {
                order.push(a);
            }
        }
        for a in order {
            let Some(anim) = model.animations.get(a) else {
                continue;
            };
            let mut channels: Vec<(usize, &Channel)> = anim
                .channels
                .iter()
                .enumerate()
                .filter(|(_, c)| c.node == i)
                .collect();
            channels.sort_by_key(|(_, c)| c.target);
            for (k, c) in channels {
                let kind = if c.target.width() == 4 {
                    "VEC4"
                } else {
                    "VEC3"
                };
                let input = file.floats(&c.times, "SCALAR", 1, true, None);
                let output = file.floats(&c.values, kind, c.target.width(), false, None);
                keys[a][k] = (input, output);
            }
        }
    }
    while !file.bin.len().is_multiple_of(4) {
        file.bin.push(0);
    }
    let roots: Vec<String> = model.roots.iter().map(|r| r.to_string()).collect();
    let mut json = format!(
        "{{\"asset\":{{\"generator\":{},\"version\":\"2.0\"}},\"scene\":0,",
        quoted(&model.generator)
    );
    json.push_str(&format!(
        "\"scenes\":[{{\"name\":\"Scene\",\"nodes\":[{}]}}],",
        roots.join(",")
    ));
    json.push_str(&format!("\"nodes\":[{}],", nodes.join(",")));
    if !model.animations.is_empty() {
        let all: Vec<String> = model
            .animations
            .iter()
            .zip(&keys)
            .map(|(a, keys)| {
                let (mut channels, mut samplers) = (Vec::new(), Vec::new());
                for (k, (c, (input, output))) in a.channels.iter().zip(keys).enumerate() {
                    channels.push(format!(
                        "{{\"sampler\":{k},\"target\":{{\"node\":{},\"path\":\"{}\"}}}}",
                        c.node,
                        c.target.name()
                    ));
                    samplers.push(format!(
                        "{{\"input\":{input},\"interpolation\":\"{}\",\"output\":{output}}}",
                        c.interpolation.name()
                    ));
                }
                format!(
                    "{{\"channels\":[{}],\"name\":{},\"samplers\":[{}]}}",
                    channels.join(","),
                    quoted(&a.name),
                    samplers.join(",")
                )
            })
            .collect();
        json.push_str(&format!("\"animations\":[{}],", all.join(",")));
    }
    let mats: Vec<String> = model.materials.iter().map(material_json).collect();
    json.push_str(&format!("\"materials\":[{}],", mats.join(",")));
    json.push_str(&format!("\"meshes\":[{}],", meshes.join(",")));
    json.push_str(&format!("\"accessors\":[{}],", file.accessors.join(",")));
    json.push_str(&format!("\"bufferViews\":[{}],", file.views.join(",")));
    json.push_str(&format!(
        "\"buffers\":[{{\"byteLength\":{}}}]}}",
        file.bin.len()
    ));
    while !json.len().is_multiple_of(4) {
        json.push(' ');
    }
    let total = 12 + 8 + json.len() + 8 + file.bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend(b"glTF");
    out.extend(2u32.to_le_bytes());
    out.extend((total as u32).to_le_bytes());
    out.extend((json.len() as u32).to_le_bytes());
    out.extend(b"JSON");
    out.extend(json.as_bytes());
    out.extend((file.bin.len() as u32).to_le_bytes());
    out.extend(b"BIN\0");
    out.extend(&file.bin);
    out
}
