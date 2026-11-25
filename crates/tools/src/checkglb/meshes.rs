//! A model file as plain meshes (`edges::Mesh`): every primitive of the default scene's mesh nodes at rest, in metres
//! as the game draws it (the entry's `scale`), with what its material is to the edge and texel rules; those rules as
//! the gate's counted checks `edges` and `texels`; and every model so read, for `cargo review edges` and `texels`.

use super::glb::{Glb, R, image_size, int, item, key, parse_glb};
use super::py::{self, J, Num, dirname, join, normpath};
use super::{
    Finishes, Found, ModelFile, Paths, asset_manifests, in_hand, model_entries, node_matrices,
    node_name, override_of, primitives,
};
use crate::edges::{self, Mesh};
use crate::texels::{self, Texture, Textures};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// What the manifests' finishes are named by.
const FINISH: &str = "material.finish.";
/// The finishes drawn as leaves and bark: lumps, cards and jittered trunks, not made things with edges.
const GROWN: [&str; 3] = ["foliage", "leafcards", "bark"];
/// The finish whose UVs pick cut-out pictures from an atlas: it tiles nothing, and hides nothing behind it.
const CARDS: &str = "leafcards";

/// What is read of each primitive besides its positions and triangles.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Read {
    /// Every primitive, with its normals: what the edge rule reads.
    Shape,
    /// The primitives on textured finishes, with their UVs: what the texel rule reads.
    Textured,
}

/// The elements of an accessor, `width` numbers each, all in one row: floats, or (`whole`) unsigned integers; None
/// where it can't be read here (compressed, sparse, another type).
fn numbers(g: &Glb, index: &J, width: usize, whole: bool) -> R<Option<Vec<f64>>> {
    let acc = item(&g.gltf, "accessors", index)?;
    if !acc.has("bufferView") || acc.get("sparse").is_some_and(J::truthy) {
        return Ok(None);
    }
    let view = item(&g.gltf, "bufferViews", key(acc, "bufferView")?)?;
    if view.get("extensions").is_some_and(J::truthy) {
        return Ok(None);
    }
    let size = match (key(acc, "componentType")?, whole) {
        (J::Int(5126), false) | (J::Int(5125), true) => 4,
        (J::Int(5123), true) => 2,
        (J::Int(5121), true) => 1,
        _ => return Ok(None),
    };
    let count = int(acc, "count", 0)?.max(0) as usize;
    let stride = match int(view, "byteStride", 0)? {
        s if s > 0 => s as usize,
        _ => size * width,
    };
    let start = (int(view, "byteOffset", 0)? + int(acc, "byteOffset", 0)?).max(0) as usize;
    if count > 0 && start + (count - 1) * stride + size * width > g.bin.len() {
        return Err("an accessor reaches past the file's data".into());
    }
    let mut out = Vec::with_capacity(count * width);
    for i in 0..count {
        for k in 0..width {
            let b = &g.bin[start + i * stride + k * size..];
            out.push(match (size, whole) {
                (4, false) => f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
                (4, true) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
                (2, _) => u16::from_le_bytes([b[0], b[1]]) as f64,
                _ => b[0] as f64,
            });
        }
    }
    Ok(Some(out))
}

/// The file's meshes (`Mesh`) as `read` wants them; None where some of it can't be read here (a compressed copy).
pub(super) fn read(file: &ModelFile, read: Read) -> R<Option<Vec<Mesh>>> {
    let (g, gltf) = (file.glb, &file.glb.gltf);
    let empty = J::Arr(Vec::new());
    let nodes = gltf.get("nodes").unwrap_or(&empty);
    let materials = gltf.get("materials").unwrap_or(&empty);
    let no_overrides = J::Obj(Vec::new());
    let overrides = file.entry.get("materials").unwrap_or(&no_overrides);
    let scale = if file.entry.get("fit").is_some_and(J::truthy) {
        1.0
    } else {
        file.entry.get("scale").and_then(J::num).map_or(1.0, Num::f)
    };
    // A texture is judged on a part at its largest: at rest, or at the largest key of any clip that scales it up
    // (`grown`: past 1.5 times its rest). So one shrunk at rest that a clip grows back (washing flattened on the
    // line, bedding folded into a sofa bed, curtains gathered open) is judged at its grown size, and one a clip
    // stretches past its built size (a cloth scaled out) as stretched. The edges are the model's at rest.
    let pose = match read {
        Read::Textured => Some(super::grown(g, &[])?),
        Read::Shape => None,
    };
    let mut out = Vec::new();
    for p in node_matrices(gltf, pose.as_ref(), None)? {
        let m = p.m.map(Num::f);
        // A normal goes through the matrix's cofactors; a mirrored node turns its faces inside out.
        let cofactor = |r: usize, c: usize| {
            let at = |r: usize, c: usize| m[(c % 3) * 4 + r % 3];
            at(r + 1, c + 1) * at(r + 2, c + 2) - at(r + 1, c + 2) * at(r + 2, c + 1)
        };
        let det = (0..3).map(|c| m[c * 4] * cofactor(0, c)).sum::<f64>();
        let mirrored = det < 0.0;
        for prim in primitives(gltf, &p.mesh)? {
            if !prim
                .get("mode")
                .is_none_or(|m| m.num().is_some_and(|n| n.f() == 4.0))
            {
                continue;
            }
            let material = match prim.get("material") {
                Some(i) => Some(super::glb::index(materials, i)?),
                None => None,
            };
            let name = material
                .and_then(|m| m.get("name"))
                .map(J::key)
                .unwrap_or_default();
            let ov = override_of(overrides, &name);
            if ov.truthy() && ov.get("hide").is_some_and(J::truthy) {
                continue;
            }
            let finish = ov.get("finish").filter(|f| ov.truthy() && f.truthy());
            // (Unmapped, the file's own material says what it is: the gate fails it as `materials`.)
            let (clear, glows) = match finish {
                Some(_) => (
                    file.fin.see_through(finish),
                    ov.get("glow").is_some_and(J::truthy),
                ),
                None => (
                    material.and_then(|m| m.get("alphaMode")) == Some(&J::Str("BLEND".into())),
                    material
                        .and_then(|m| m.get("emissiveFactor"))
                        .and_then(J::list)
                        .is_some_and(|e| e.iter().any(J::truthy)),
                ),
            };
            let finish_key = finish.map(J::key).unwrap_or_default();
            let textured = file.fin.textures.contains_key(&finish_key);
            if read == Read::Textured && !textured {
                continue;
            }
            let finish = finish_key
                .strip_prefix(FINISH)
                .unwrap_or(&finish_key)
                .to_string();
            let attributes = key(prim, "attributes")?;
            let Some(positions) = numbers(g, key(attributes, "POSITION")?, 3, false)? else {
                return Ok(None);
            };
            let positions: Vec<[f64; 3]> = positions
                .as_chunks::<3>()
                .0
                .iter()
                .map(|v| {
                    [0, 1, 2].map(|r| {
                        (m[r] * v[0] + m[4 + r] * v[1] + m[8 + r] * v[2] + m[12 + r]) * scale
                    })
                })
                .collect();
            let normals = match attributes.get("NORMAL").filter(|_| read == Read::Shape) {
                Some(i) => numbers(g, i, 3, false)?.map_or(Vec::new(), |n| {
                    n.as_chunks::<3>()
                        .0
                        .iter()
                        .map(|v| {
                            let sign = if mirrored { -1.0 } else { 1.0 };
                            [0, 1, 2]
                                .map(|r| sign * (0..3).map(|c| cofactor(r, c) * v[c]).sum::<f64>())
                        })
                        .collect()
                }),
                None => Vec::new(),
            };
            let uvs = match attributes
                .get("TEXCOORD_0")
                .filter(|_| read == Read::Textured)
            {
                Some(i) => {
                    numbers(g, i, 2, false)?.map_or(Vec::new(), |uv| uv.as_chunks::<2>().0.to_vec())
                }
                None => Vec::new(),
            };
            let indices: Vec<u32> = match prim.get("indices") {
                Some(i) => match numbers(g, i, 1, true)? {
                    Some(i) => i.iter().map(|&v| v as u32).collect(),
                    None => return Ok(None),
                },
                None => (0..positions.len() as u32).collect(),
            };
            let triangles = indices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|t| if mirrored { [t[0], t[2], t[1]] } else { *t })
                .collect();
            out.push(Mesh {
                part: node_name(nodes, p.node),
                material: name,
                edged: !clear && !glows && !GROWN.contains(&finish.as_str()),
                opaque: !clear && finish != CARDS,
                finish,
                positions,
                normals,
                uvs,
                triangles,
            });
        }
    }
    Ok(Some(out))
}

/// The side (px) of the picture at `path` (the root of its width times its height), read once; None: not read.
fn side(path: &str) -> Option<f64> {
    static SIDES: OnceLock<Mutex<HashMap<String, Option<f64>>>> = OnceLock::new();
    let mut sides = SIDES.get_or_init(Default::default).lock().ok()?;
    *sides.entry(path.to_string()).or_insert_with(|| {
        let (w, h) = image_size(&std::fs::read(path).ok()?)?;
        Some((w as f64 * h as f64).sqrt())
    })
}

/// The textured finishes the texel rule judges (`Textures`): each with its `uvScale` and its texture's size, but the
/// leaf cards.
fn textures(fin: &Finishes) -> Textures {
    fin.textures
        .iter()
        .filter_map(|(finish, (uv_scale, file))| {
            let name = finish.strip_prefix(FINISH).unwrap_or(finish);
            (name != CARDS).then(|| {
                let texture = Texture {
                    uv_scale: *uv_scale,
                    px: side(file),
                };
                (name.to_string(), texture)
            })
        })
        .collect()
}

/// Whether the model `key`'s edges are judged: not a thing only ever in a hand, small and half hidden by it.
pub fn edged(key: &str) -> bool {
    !in_hand(key)
}

/// The counted check `edges` (`edges::hard_edges`): a model's hard edges that show unbevelled, one line for them
/// all. A far version is seen from too far for an edge to show, and isn't judged.
pub(super) fn hard_edges(file: &ModelFile) -> R<Found> {
    if file.far || !edged(file.key) {
        return Ok(Found::default());
    }
    let Some(meshes) = read(file, Read::Shape)? else {
        return Ok(Found::default());
    };
    let found = edges::hard_edges(&meshes);
    Ok(Found {
        count: found.len() as u64,
        text: edges::line(file.key, &found).into_iter().collect(),
    })
}

/// The counted check `texels` (`texels::findings`): each material whose textured finish is laid too sparse, too
/// dense, stretched or smeared, a finding for each of those. Not a far version's, seen from too far.
pub(super) fn texels(file: &ModelFile) -> R<Found> {
    if file.far || file.fin.textures.is_empty() {
        return Ok(Found::default());
    }
    let Some(meshes) = read(file, Read::Textured)? else {
        return Ok(Found::default());
    };
    let text: Vec<String> = texels::measure(&meshes, &textures(file.fin))
        .iter()
        .flat_map(texels::findings)
        .collect();
    Ok(Found {
        count: text.len() as u64,
        text,
    })
}

/// Every model the manifests have a file for, or those of `keys`, as plain meshes: `each(key, family, meshes)`,
/// the family its file's folder; `textured`: only what is on textured finishes, with its UVs (else everything, with
/// its normals).
pub fn each(
    paths: &Paths,
    keys: &[String],
    textured: bool,
    each: &mut dyn FnMut(&str, &str, Vec<Mesh>),
) -> Result<(), String> {
    let (entries, fin) = model_entries(&asset_manifests(paths)?);
    if let Some(key) = keys.iter().find(|k| !entries.contains_key(*k)) {
        return Err(format!("no manifest has a model entry {key}"));
    }
    for (k, (entry, manifest)) in &entries {
        let url = entry
            .get("url")
            .filter(|u| u.truthy())
            .map(J::key)
            .unwrap_or_default();
        if url.is_empty() || url.contains("://") || !(keys.is_empty() || keys.contains(k)) {
            continue;
        }
        let path = normpath(&join(&dirname(manifest), &url));
        let meshes = std::fs::read(&path)
            .map_err(|e| py::os_error(&e, &path))
            .and_then(|data| {
                let glb = parse_glb(&data)?;
                let file = ModelFile {
                    key: k,
                    entry,
                    glb: &glb,
                    fin: &fin,
                    far: false,
                };
                read(
                    &file,
                    if textured {
                        Read::Textured
                    } else {
                        Read::Shape
                    },
                )
            })
            .map_err(|e| format!("cannot read {path}: {e}"))?;
        let family = dirname(&path);
        let family = family.rsplit('/').next().unwrap_or_default();
        each(k, family, meshes.unwrap_or_default());
    }
    Ok(())
}

/// The textured finishes of the manifests under `paths` that the texel rule judges.
pub fn textures_of(paths: &Paths) -> Result<Textures, String> {
    Ok(textures(&model_entries(&asset_manifests(paths)?).1))
}

#[cfg(test)]
mod tests {
    use super::super::glb::tests::glb;
    use super::*;
    use std::collections::HashSet;

    /// A file with one square of 1 × 1 in its node's x and y, looking along z, of material `top`; the node as given.
    fn square(node: &str) -> Glb {
        let mut bin = Vec::new();
        let corners = [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        for c in corners {
            for v in [c[0], c[1], 0.0] {
                bin.extend(v.to_le_bytes());
            }
        }
        for _ in corners {
            for v in [0.0f32, 0.0, 1.0] {
                bin.extend(v.to_le_bytes());
            }
        }
        for c in corners {
            for v in [c[0] * 0.25, c[1]] {
                bin.extend(v.to_le_bytes());
            }
        }
        for i in [0u16, 1, 2, 0, 2, 3] {
            bin.extend(i.to_le_bytes());
        }
        let json = format!(
            r#"{{"asset": {{"version": "2.0"}}, "scenes": [{{"nodes": [0]}}], "nodes": [{{"name": "leaf", "mesh": 0{node}}}],
            "meshes": [{{"primitives": [{{"attributes": {{"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}}, "indices": 3, "material": 0}}]}}],
            "materials": [{{"name": "top"}}],
            "accessors": [{{"bufferView": 0, "componentType": 5126, "count": 4, "type": "VEC3", "min": [0, 0, 0], "max": [1, 1, 0]}},
                {{"bufferView": 1, "componentType": 5126, "count": 4, "type": "VEC3"}},
                {{"bufferView": 2, "componentType": 5126, "count": 4, "type": "VEC2"}},
                {{"bufferView": 3, "componentType": 5123, "count": 6, "type": "SCALAR"}}],
            "bufferViews": [{{"buffer": 0, "byteOffset": 0, "byteLength": 48}}, {{"buffer": 0, "byteOffset": 48, "byteLength": 48}},
                {{"buffer": 0, "byteOffset": 96, "byteLength": 32}}, {{"buffer": 0, "byteOffset": 128, "byteLength": 12}}],
            "buffers": [{{"byteLength": 140}}]}}"#
        );
        parse_glb(&glb(&json, &bin)).unwrap()
    }

    fn finishes() -> Finishes {
        let finish = |name: &str| format!("{FINISH}{name}");
        Finishes {
            plain: ["satin", "glass"]
                .iter()
                .map(|f| (finish(f), true))
                .chain(
                    ["wood", "bark", "leafcards"]
                        .iter()
                        .map(|f| (finish(f), false)),
                )
                .collect(),
            see_through: HashSet::from([finish("glass")]),
            textures: ["wood", "bark", "leafcards"]
                .iter()
                .map(|f| (finish(f), (2.0, format!("/nonexistent/{f}.jpg"))))
                .collect(),
        }
    }

    fn entry(text: &str) -> J {
        super::super::json::loads_bytes(text.as_bytes()).unwrap()
    }

    fn mapped(finish: &str) -> J {
        entry(&format!(
            r#"{{"materials": {{"top": {{"finish": "material.finish.{finish}"}}}}}}"#
        ))
    }

    fn meshes(g: &Glb, entry: &J, fin: &Finishes, read: Read) -> Vec<Mesh> {
        let file = ModelFile {
            key: "model.leaf",
            entry,
            glb: g,
            fin,
            far: false,
        };
        super::read(&file, read).unwrap().unwrap()
    }

    #[test]
    fn a_file_is_read_as_the_game_draws_it() {
        let fin = finishes();
        // Through its node (moved 1 m up, turned a quarter round y so it looks along x) and the entry's scale.
        let g = square(
            r#", "translation": [0, 1, 0], "rotation": [0, 0.7071067811865476, 0, 0.7071067811865476]"#,
        );
        let entry =
            entry(r#"{"scale": 2, "materials": {"top": {"finish": "material.finish.wood"}}}"#);
        let read = meshes(&g, &entry, &fin, Read::Shape);
        assert_eq!(read.len(), 1);
        let m = &read[0];
        assert_eq!(
            (m.part.as_str(), m.material.as_str(), m.finish.as_str()),
            ("leaf", "top", "wood")
        );
        assert!(m.edged && m.opaque && m.uvs.is_empty());
        let near = |a: [f64; 3], b: [f64; 3]| (0..3).all(|k| (a[k] - b[k]).abs() < 1e-6);
        assert!(
            near(m.positions[0], [0.0, 2.0, 0.0]) && near(m.positions[2], [0.0, 4.0, -2.0]),
            "{:?}",
            m.positions
        );
        assert!(
            m.normals.iter().all(|n| near(*n, [1.0, 0.0, 0.0])),
            "{:?}",
            m.normals
        );
        assert_eq!(m.triangles, [[0, 1, 2], [0, 2, 3]]);
        // For the texel rule: its UVs, no normals.
        let read = meshes(&g, &entry, &fin, Read::Textured);
        assert_eq!(
            read[0].uvs,
            [[0.0, 0.0], [0.25, 0.0], [0.25, 1.0], [0.0, 1.0]]
        );
        assert!(read[0].normals.is_empty());
        // A node that mirrors turns its triangles round, so they still face out, and its normals with them.
        let g = square(r#", "scale": [1, 1, -1]"#);
        let read = meshes(&g, &entry, &fin, Read::Shape);
        assert_eq!(read[0].triangles, [[0, 2, 1], [0, 3, 2]]);
        assert!(
            read[0].normals.iter().all(|n| near(*n, [0.0, 0.0, -1.0])),
            "{:?}",
            read[0].normals
        );
    }

    #[test]
    fn a_material_is_what_its_finish_makes_it() {
        let (fin, g) = (finishes(), square(""));
        let flags = |entry: &J| {
            let read = meshes(&g, entry, &fin, Read::Shape);
            read.first().map(|m| (m.edged, m.opaque))
        };
        assert_eq!(flags(&mapped("satin")), Some((true, true)));
        assert_eq!(flags(&mapped("glass")), Some((false, false)));
        assert_eq!(flags(&mapped("bark")), Some((false, true)));
        assert_eq!(flags(&mapped("leafcards")), Some((false, false)));
        let glows = entry(
            r##"{"materials": {"top": {"finish": "material.finish.satin", "glow": "#FFEEAA"}}}"##,
        );
        assert_eq!(flags(&glows), Some((false, true)));
        let hidden =
            entry(r#"{"materials": {"top": {"finish": "material.finish.satin", "hide": true}}}"#);
        assert_eq!(flags(&hidden), None);
        // Unmapped: as the file has it.
        assert_eq!(flags(&entry("{}")), Some((true, true)));
        // Only what is on a textured finish is read for the texel rule, and the leaf cards aren't judged by it.
        assert_eq!(meshes(&g, &mapped("satin"), &fin, Read::Textured).len(), 0);
        assert_eq!(meshes(&g, &mapped("bark"), &fin, Read::Textured).len(), 1);
        let judged = textures(&fin);
        assert_eq!(judged.keys().collect::<Vec<_>>(), ["bark", "wood"]);
        assert_eq!(
            judged["wood"],
            Texture {
                uv_scale: 2.0,
                px: None
            }
        );
    }

    #[test]
    fn the_checks_judge_a_model_but_not_its_far_version_nor_a_held_tool_s_edges() {
        let (fin, g, entry) = (finishes(), square(""), mapped("wood"));
        let file = |key: &'static str, far: bool| ModelFile {
            key,
            entry: &entry,
            glb: &g,
            fin: &fin,
            far,
        };
        // The square's UVs lay the texture a quarter as far along x as along y: stretched all over.
        let found = texels(&file("model.leaf", false)).unwrap();
        assert_eq!(
            (found.count, found.text),
            (
                1,
                vec![
                    "material top (wood): stretched over 2× on 100 % of its area (at most 5.0 %)"
                        .to_string()
                ]
            )
        );
        assert_eq!(texels(&file("tool.leaf", false)).unwrap().count, 1);
        assert_eq!(texels(&file("model.leaf", true)).unwrap(), Found::default());
        // (A square alone has no edge of two faces: the shipped models show the edge rule at work.)
        assert_eq!(
            hard_edges(&file("model.leaf", false)).unwrap(),
            Found::default()
        );
        assert_eq!(
            hard_edges(&file("model.leaf", true)).unwrap(),
            Found::default()
        );
        assert!(edged("model.fridge") && edged("item.plate") && !edged("tool.fork"));
    }

    #[test]
    fn a_picture_s_side_is_read_from_its_file() {
        let wood = join(
            &super::super::repo_root(),
            "web/public/assets/textures/wood_oak/albedo.jpg",
        );
        assert_eq!(side(&wood), Some(512.0));
        assert_eq!(side("/nonexistent/albedo.jpg"), None);
        let judged = textures_of(&Paths::repo()).unwrap();
        assert_eq!(
            judged["wood"],
            Texture {
                uv_scale: 1.2,
                px: Some(512.0)
            }
        );
        assert_eq!(judged["fabric"].uv_scale, 2.5);
    }
}
