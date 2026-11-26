//! Writing a model out.

use super::*;
use crate::materials::Finishes;
use crate::motion::{self, Path};
use crate::normals::corner_normals;
use crate::scene::{self, FPS, Obj};
use std::collections::BTreeMap;

/// The .glb of `objs` as they stand in the scene (shaded: `shade::bake_ao`; their UVs made: `uv::ensure`).
/// `texcoords`: whether the UVs are written (a model with a part on a textured finish). `finishes`: the game's,
/// for each material's roughness and metalness.
pub fn write(objs: &[Obj], texcoords: bool, finishes: &Finishes) -> Vec<u8> {
    // the nodes in the order Blender's exporter writes them, whatever order they were given in
    let (objs, clip_order) = file_order(objs);
    let objs = &objs[..];
    let mut model = Model {
        generator: "idyll-lives artkit".into(),
        ..Model::default()
    };
    let index_of: BTreeMap<Obj, usize> = objs.iter().enumerate().map(|(i, &o)| (o, i)).collect();
    let mut children: BTreeMap<Obj, Vec<usize>> = BTreeMap::new();
    for (i, &o) in objs.iter().enumerate() {
        match o.read(|p| p.parent).filter(|p| index_of.contains_key(p)) {
            Some(parent) => children.entry(parent).or_default().push(i),
            None => model.roots.push(i),
        }
    }
    for (i, &o) in objs.iter().enumerate() {
        let (name, data_name, mesh, loc, rot, scale, quat, order) = o.read(|p| {
            (
                p.name.clone(),
                p.data_name.clone(),
                p.mesh.clone(),
                p.loc,
                p.rot,
                p.scale,
                p.quat,
                p.order,
            )
        });
        let normals = corner_normals(&mesh);
        let tris = mesh.triangles();
        let mut primitives = Vec::new();
        // every material, then one pass for the faces no material takes: a part with no material at all, or a
        // face on a slot past `mats`. Those are written without a material, not dropped -- a glTF mesh needs a
        // primitive, and a part that vanishes from the file is a model missing a piece with no error anywhere.
        for slot in (0..mesh.mats.len()).map(Some).chain([None]) {
            // this material's corners, each (vertex, normal, uv) once, in the order of its vertices
            let mut corners: BTreeMap<(u32, [i64; 3], [i64; 2]), u32> = BTreeMap::new();
            let mut wanted = Vec::new();
            for (f, t) in &tris {
                let face = mesh.faces[*f as usize];
                let mine = match slot {
                    Some(slot) => face.mat as usize == slot,
                    None => face.mat as usize >= mesh.mats.len(),
                };
                if !mine {
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
            let (mut positions, mut normal, mut uvs, mut colours) =
                (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for &l in corners.values() {
                let l = l as usize;
                let v = mesh.loops[l] as usize;
                positions.push(y_up(mesh.verts[v]));
                normal.push(rounded_normal(normals[l]));
                let uv = mesh.uvs.as_ref().map(|u| u[l]).unwrap_or([0.0, 0.0]);
                uvs.push([uv[0], 1.0 - uv[1]]);
                let a = mesh.ao.as_ref().map(|ao| ao[v]).unwrap_or(1.0);
                colours.push([a, a, a, 1.0]);
            }
            let material = slot.map(|slot| {
                let mat = &mesh.mats[slot];
                match model.materials.iter().position(|m| m.name == mat.name) {
                    Some(m) => m,
                    None => {
                        let finish = finishes.get(mat.finish);
                        let [r, g, b] = mat.linear().map(|c| c as f32);
                        model.materials.push(Material {
                            name: mat.name.to_string(),
                            base: [r, g, b, 1.0],
                            metallic: mat
                                .extra
                                .metallic
                                .or(finish.map(|f| f.metallic))
                                .unwrap_or(0.0) as f32,
                            roughness: mat
                                .extra
                                .roughness
                                .or(finish.map(|f| f.roughness))
                                .unwrap_or(0.6) as f32,
                            double_sided: false,
                            cutoff: None,
                        });
                        model.materials.len() - 1
                    }
                }
            });
            primitives.push(Primitive {
                positions,
                normals: normal,
                uvs: texcoords.then_some(uvs),
                colours: Some(colours),
                indices: wanted.iter().map(|k| order[k]).collect(),
                material,
            });
        }
        model.meshes.push(Mesh {
            name: data_name,
            primitives,
        });
        model.nodes.push(Node {
            name,
            mesh: Some(i),
            children: children.remove(&o).unwrap_or_default(),
            translation: y_up(loc),
            rotation: turn_y_up(rot, order, quat),
            scale: [scale.x, scale.z, scale.y],
            clips: Vec::new(),
        });
    }
    // the clips: one animation a name, in the order they are first met
    for i in clip_order {
        let o = objs[i];
        let tracks = o.read(|p| p.tracks.clone());
        for track in tracks {
            let at = match model.animations.iter().position(|a| a.name == track.name) {
                Some(a) => a,
                None => {
                    model.animations.push(Animation {
                        name: track.name.clone(),
                        channels: Vec::new(),
                    });
                    model.animations.len() - 1
                }
            };
            model.nodes[i].clips.push(at);
            let frames: Vec<i32> = (1..=track.end).collect();
            let times: Vec<f32> = frames
                .iter()
                .map(|&f| ((f - 1) as f64 / FPS) as f32)
                .collect();
            let (order, quat) = o.read(|p| (p.order, p.quat));
            let mut paths: Vec<Path> = track.channels.iter().map(|c| c.path).collect();
            paths.sort();
            paths.dedup();
            for path in paths {
                let mut values = Vec::new();
                for &f in &frames {
                    let (loc, rot, scale) = motion::pose(o, Some(&track.name), f as f64);
                    match path {
                        Path::Location => values.extend(y_up(loc)),
                        Path::Rotation => values.extend(turn_y_up(rot, order, quat)),
                        Path::Scale => values.extend([scale.x, scale.z, scale.y]),
                    }
                }
                let target = match path {
                    Path::Location => Target::Translation,
                    Path::Rotation => Target::Rotation,
                    Path::Scale => Target::Scale,
                };
                model.animations[at].channels.push(Channel {
                    node: i,
                    target,
                    interpolation: Interpolation::Linear,
                    times: times.clone(),
                    values,
                });
            }
        }
    }
    encode(&model)
}

pub(super) fn quoted(s: &str) -> String {
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
