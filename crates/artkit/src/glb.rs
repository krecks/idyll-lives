//! Writing a model out: the .glb (docs/design/objects.md, "Export settings"), laid out as the files the game
//! has always been given: +Y up (Blender -Y is the game's +Z front), a node a part with its name and origin, a
//! primitive a material with POSITION, NORMAL, TEXCOORD_0 (only where a textured finish is used) and COLOR_0 (the
//! baked shading), materials as a name and a base colour, clips as animations from 0 s with a key a frame.

use crate::json::float_repr;
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
/// share a vertex), and only then turned +Y up, as Blender's exporter does it (its squares are summed x, y, z as
/// Blender has them: summed as the file has them, one normal in nine comes out a unit in the last place off).
fn rounded_normal(n: V3) -> [f32; 3] {
    let r = n.array().map(|x| (x * 10000.0).round_ties_even() / 10000.0);
    let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    let unit = if len == 0.0 { r } else { r.map(|x| x / len) };
    // (no zero in a file of Blender's is a negative one)
    y_up(V3::new(unit[0], unit[1], unit[2])).map(|x| x + 0.0)
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
