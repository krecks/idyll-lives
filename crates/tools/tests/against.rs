//! `cargo check-glb --against` on models made here: the same model written in another order is `same`, a planted
//! fault is `CHANGED` and names its metric, and a band of a face cut another way is `near`.

use ll_tools::checkglb::against::{Class, Compared, Options, Outcome, Source, against, compare};
use std::path::{Path, PathBuf};

/// One primitive: a material's triangles, with normals, texture coordinates and baked shading (a grey of 16 bits).
#[derive(Clone)]
struct Prim {
    material: usize,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    greys: Vec<u16>,
    triangles: Vec<[u32; 3]>,
}

#[derive(Clone)]
struct Node {
    name: String,
    translation: [f32; 3],
    prims: Vec<Prim>,
}

/// A clip that turns one node about y: the angle (degrees) at each time (s).
#[derive(Clone)]
struct Clip {
    name: String,
    node: usize,
    times: Vec<f32>,
    turns: Vec<f32>,
}

#[derive(Clone)]
struct Model {
    nodes: Vec<Node>,
    materials: Vec<(String, [f32; 4])>,
    clips: Vec<Clip>,
}

/// What a vertex at `p` on a face along `axis` carries: texture coordinates in metres across the face, and a
/// shading that darkens towards the floor. From the place alone, so a face cut another way carries the same.
fn carried(p: [f32; 3], axis: usize) -> ([f32; 2], u16) {
    let uv = [p[(axis + 1) % 3], p[(axis + 2) % 3]];
    (uv, (65535.0 * (0.6 + 0.4 * p[1].clamp(0.0, 1.0))) as u16)
}

impl Prim {
    fn new(material: usize) -> Prim {
        Prim {
            material,
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            greys: Vec::new(),
            triangles: Vec::new(),
        }
    }

    fn vertex(&mut self, p: [f32; 3], normal: [f32; 3], axis: usize) -> u32 {
        let (uv, grey) = carried(p, axis);
        self.positions.push(p);
        self.normals.push(normal);
        self.uvs.push(uv);
        self.greys.push(grey);
        self.positions.len() as u32 - 1
    }

    /// A box from `lo` to `hi`, each face `n` by `n` squares of two triangles, facing out.
    fn boxed(mut self, lo: [f32; 3], hi: [f32; 3], n: u32) -> Prim {
        for axis in 0..3 {
            for (side, out) in [(lo[axis], -1.0f32), (hi[axis], 1.0)] {
                let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
                let mut normal = [0.0; 3];
                normal[axis] = out;
                let first = self.positions.len() as u32;
                for j in 0..=n {
                    for i in 0..=n {
                        let mut p = [0.0; 3];
                        p[axis] = side;
                        p[u] = lo[u] + (hi[u] - lo[u]) * i as f32 / n as f32;
                        p[v] = lo[v] + (hi[v] - lo[v]) * j as f32 / n as f32;
                        self.vertex(p, normal, axis);
                    }
                }
                for j in 0..n {
                    for i in 0..n {
                        let at = |i: u32, j: u32| first + j * (n + 1) + i;
                        let quad = [at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)];
                        for t in [[quad[0], quad[1], quad[2]], [quad[0], quad[2], quad[3]]] {
                            self.triangles
                                .push(if out > 0.0 { t } else { [t[0], t[2], t[1]] });
                        }
                    }
                }
            }
        }
        self
    }
}

/// A cupboard: a wooden body with a brass knob and a small block of the same wood on its top, and a door of its
/// own wood that a clip swings open. 0.6 x 0.8 x 0.4 m.
fn cupboard() -> Model {
    Model {
        materials: vec![
            ("wood".into(), [0.5, 0.3, 0.2, 1.0]),
            ("brass".into(), [0.8, 0.6, 0.2, 1.0]),
            ("door_wood".into(), [0.45, 0.28, 0.2, 1.0]),
        ],
        nodes: vec![
            Node {
                name: "body".into(),
                translation: [0.0, 0.0, 0.0],
                prims: vec![
                    Prim::new(0)
                        .boxed([-0.3, 0.0, -0.2], [0.3, 0.8, 0.2], 6)
                        .boxed([-0.02, 0.8, -0.02], [0.02, 0.83, 0.02], 1),
                    Prim::new(1).boxed([0.2, 0.4, 0.2], [0.22, 0.42, 0.22], 2),
                ],
            },
            Node {
                name: "door".into(),
                translation: [-0.3, 0.0, 0.2],
                prims: vec![Prim::new(2).boxed([0.0, 0.02, 0.0], [0.58, 0.78, 0.02], 6)],
            },
        ],
        clips: vec![Clip {
            name: "door_open".into(),
            node: 1,
            times: vec![0.0, 0.5, 1.0],
            turns: vec![0.0, 45.0, 90.0],
        }],
    }
}

/// The model as a `.glb`: an accessor and a buffer view for each attribute, as an exporter writes one.
fn glb(model: &Model) -> Vec<u8> {
    use serde_json::{Value, json};
    let mut bin: Vec<u8> = Vec::new();
    let (mut views, mut accessors): (Vec<Value>, Vec<Value>) = (Vec::new(), Vec::new());
    let mut put = |bytes: Vec<u8>, mut accessor: Value| -> usize {
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        views.push(json!({"buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len()}));
        bin.extend(bytes);
        accessor["bufferView"] = json!(views.len() - 1);
        accessors.push(accessor);
        accessors.len() - 1
    };
    let floats = |v: &[f32]| -> Vec<u8> { v.iter().flat_map(|x| x.to_le_bytes()).collect() };
    let (mut nodes, mut meshes) = (Vec::new(), Vec::new());
    for node in &model.nodes {
        let mut prims = Vec::new();
        for p in &node.prims {
            let count = p.positions.len();
            let flat: Vec<f32> = p.positions.iter().flatten().copied().collect();
            let bound = |pick: fn(f32, f32) -> f32| -> Vec<f32> {
                (0..3)
                    .map(|k| p.positions.iter().map(|q| q[k]).reduce(pick).unwrap_or(0.0))
                    .collect()
            };
            let position = put(
                floats(&flat),
                json!({"componentType": 5126, "type": "VEC3", "count": count, "min": bound(f32::min), "max": bound(f32::max)}),
            );
            let normal = put(
                floats(&p.normals.iter().flatten().copied().collect::<Vec<_>>()),
                json!({"componentType": 5126, "type": "VEC3", "count": count}),
            );
            let uv = put(
                floats(&p.uvs.iter().flatten().copied().collect::<Vec<_>>()),
                json!({"componentType": 5126, "type": "VEC2", "count": count}),
            );
            let colour = put(
                p.greys
                    .iter()
                    .flat_map(|g| [*g, *g, *g, 65535])
                    .flat_map(u16::to_le_bytes)
                    .collect(),
                json!({"componentType": 5123, "normalized": true, "type": "VEC4", "count": count}),
            );
            let indices = put(
                p.triangles
                    .iter()
                    .flatten()
                    .flat_map(|i| (*i as u16).to_le_bytes())
                    .collect(),
                json!({"componentType": 5123, "type": "SCALAR", "count": p.triangles.len() * 3}),
            );
            prims.push(json!({
                "attributes": {"POSITION": position, "NORMAL": normal, "TEXCOORD_0": uv, "COLOR_0": colour},
                "indices": indices,
                "material": p.material,
            }));
        }
        nodes.push(
            json!({"name": node.name, "mesh": meshes.len(), "translation": node.translation}),
        );
        meshes.push(json!({"name": node.name, "primitives": prims}));
    }
    let mut animations = Vec::new();
    for clip in &model.clips {
        let input = put(
            floats(&clip.times),
            json!({"componentType": 5126, "type": "SCALAR", "count": clip.times.len()}),
        );
        let turns: Vec<f32> = clip
            .turns
            .iter()
            .flat_map(|degrees| {
                let half = degrees.to_radians() / 2.0;
                [0.0, half.sin(), 0.0, half.cos()]
            })
            .collect();
        let output = put(
            floats(&turns),
            json!({"componentType": 5126, "type": "VEC4", "count": clip.turns.len()}),
        );
        animations.push(json!({
            "name": clip.name,
            "channels": [{"sampler": 0, "target": {"node": clip.node, "path": "rotation"}}],
            "samplers": [{"input": input, "interpolation": "LINEAR", "output": output}],
        }));
    }
    let materials: Vec<Value> = model
        .materials
        .iter()
        .map(|(name, colour)| {
            json!({"name": name, "pbrMetallicRoughness": {"baseColorFactor": colour, "metallicFactor": 0, "roughnessFactor": 0.5}})
        })
        .collect();
    let mut gltf = json!({
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [{"nodes": (0..nodes.len()).collect::<Vec<_>>()}],
        "nodes": nodes,
        "meshes": meshes,
        "materials": materials,
        "accessors": accessors,
        "bufferViews": views,
        "buffers": [{"byteLength": bin.len()}],
    });
    if !animations.is_empty() {
        gltf["animations"] = json!(animations);
    }
    chunks(&gltf.to_string(), &bin)
}

/// A `.glb` of a JSON chunk and a binary one, each padded to four bytes.
fn chunks(json: &str, bin: &[u8]) -> Vec<u8> {
    let mut json = json.as_bytes().to_vec();
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    let mut bin = bin.to_vec();
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let mut out = b"glTF".to_vec();
    out.extend(2u32.to_le_bytes());
    out.extend(((28 + json.len() + bin.len()) as u32).to_le_bytes());
    out.extend((json.len() as u32).to_le_bytes());
    out.extend(0x4E4F534Au32.to_le_bytes());
    out.extend(json);
    out.extend((bin.len() as u32).to_le_bytes());
    out.extend(0x004E4942u32.to_le_bytes());
    out.extend(bin);
    out
}

/// The same model written as another exporter might: nodes, materials, triangles and vertices in the opposite
/// order, and every triangle starting at another corner.
fn another_order(model: &Model) -> Model {
    let mut out = model.clone();
    out.nodes.reverse();
    out.materials.reverse();
    for clip in &mut out.clips {
        clip.node = model.nodes.len() - 1 - clip.node;
    }
    for node in &mut out.nodes {
        node.prims.reverse();
        for p in &mut node.prims {
            p.material = model.materials.len() - 1 - p.material;
            let last = p.positions.len() as u32 - 1;
            p.positions.reverse();
            p.normals.reverse();
            p.uvs.reverse();
            p.greys.reverse();
            p.triangles.reverse();
            for t in &mut p.triangles {
                *t = [last - t[1], last - t[2], last - t[0]];
            }
        }
    }
    out
}

/// The metrics a comparison found over their limits, each once, sorted.
fn metrics(c: &Compared) -> Vec<&'static str> {
    let mut out: Vec<&str> = c.findings.iter().map(|f| f.metric).collect();
    out.sort();
    out.dedup();
    out
}

/// The cupboard against itself with `fault` planted: the metrics over their limits.
fn planted(fault: impl Fn(&mut Model)) -> Vec<&'static str> {
    let old = cupboard();
    let mut new = another_order(&old);
    fault(&mut new);
    let found = compare(&glb(&old), &glb(&new));
    assert!(!found.same);
    assert!(!found.findings.is_empty(), "nothing found");
    metrics(&found)
}

fn node<'a>(model: &'a mut Model, name: &str) -> &'a mut Node {
    model.nodes.iter_mut().find(|n| n.name == name).unwrap()
}

fn prim<'a>(model: &'a mut Model, name: &str, material: &str) -> &'a mut Prim {
    let material = model
        .materials
        .iter()
        .position(|(m, _)| m == material)
        .unwrap();
    node(model, name)
        .prims
        .iter_mut()
        .find(|p| p.material == material)
        .unwrap()
}

#[test]
fn another_order_of_triangles_vertices_nodes_and_materials_is_the_same() {
    let old = cupboard();
    let (a, b) = (glb(&old), glb(&another_order(&old)));
    assert_ne!(a, b);
    let found = compare(&a, &b);
    assert!(found.same, "{:?}", found.findings);
    assert!(found.findings.is_empty());
}

/// A shipped model's own files: the fridge with the triangles of every primitive in the opposite order, as two exports of
/// it differ, is the same; with one number of its buffer changed it is not.
#[test]
fn a_shipped_model_with_its_triangles_in_another_order_is_the_same() {
    let root = ll_tools::checkglb::repo_root();
    let old = std::fs::read(format!(
        "{root}/web/public/assets/models/kitchen/fridge.glb"
    ))
    .unwrap();
    let json_len = u32::from_le_bytes(old[12..16].try_into().unwrap()) as usize;
    let gltf: serde_json::Value = serde_json::from_slice(&old[20..20 + json_len]).unwrap();
    let bin = 20 + json_len + 8;
    let mut new = old.clone();
    let mut indices: Vec<u64> = gltf["meshes"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| m["primitives"].as_array().unwrap())
        .map(|p| p["indices"].as_u64().unwrap())
        .collect();
    indices.sort();
    indices.dedup();
    for i in indices {
        let accessor = &gltf["accessors"][i as usize];
        assert_eq!(accessor["componentType"], 5123);
        let view = &gltf["bufferViews"][accessor["bufferView"].as_u64().unwrap() as usize];
        let at = bin + view["byteOffset"].as_u64().unwrap_or(0) as usize;
        let triangles = accessor["count"].as_u64().unwrap() as usize / 3;
        for t in 0..triangles {
            let from = at + (triangles - 1 - t) * 6;
            new[at + t * 6..at + t * 6 + 6].copy_from_slice(&old[from..from + 6]);
        }
    }
    assert_ne!(old, new);
    assert!(compare(&old, &new).same);
    // The first position's x, one unit of its last place up.
    let first = &gltf["meshes"][0]["primitives"][0]["attributes"]["POSITION"];
    let view = &gltf["bufferViews"]
        [gltf["accessors"][first.as_u64().unwrap() as usize]["bufferView"]
            .as_u64()
            .unwrap() as usize];
    let at = bin + view["byteOffset"].as_u64().unwrap_or(0) as usize;
    let x = f32::from_le_bytes(new[at..at + 4].try_into().unwrap());
    new[at..at + 4].copy_from_slice(&f32::from_bits(x.to_bits() + 1).to_le_bytes());
    let found = compare(&old, &new);
    assert!(!found.same);
    assert!(found.findings.is_empty(), "near: {:?}", found.findings);
    assert_eq!(found.numbers.differs, ["shape", "uv", "shading"]);
}

#[test]
fn a_part_moved_a_millimetre_is_changed() {
    // Its origin.
    assert_eq!(
        planted(|m| node(m, "door").translation[0] += 0.001),
        ["nodes"]
    );
    // Its triangles, in a moving part (half a millimetre is its limit): the box they fill moves too.
    assert_eq!(
        planted(|m| prim(m, "door", "door_wood")
            .positions
            .iter_mut()
            .for_each(|p| p[2] += 0.001)),
        ["bounds", "surface"]
    );
}

#[test]
fn a_vertex_moved_three_millimetres_is_changed() {
    // A point of the body's top (the four squares' corners that meet there), pressed in: the box stays.
    let found = planted(|m| {
        let mut moved = 0;
        for p in &mut prim(m, "body", "wood").positions {
            if (p[0] - 0.2).abs() < 1e-4 && p[1] == 0.8 && p[2].abs() < 1e-4 {
                p[1] -= 0.003;
                moved += 1;
            }
        }
        assert_eq!(moved, 1);
    });
    assert_eq!(found, ["surface"]);
}

#[test]
fn a_clip_dropped_or_a_key_of_it_changed_is_changed() {
    assert_eq!(planted(|m| m.clips.clear()), ["clips"]);
    assert_eq!(planted(|m| m.clips[0].turns[1] = 50.0), ["clips"]);
    assert_eq!(planted(|m| m.clips[0].times[2] = 1.1), ["clips"]);
    assert_eq!(planted(|m| m.clips[0].name = "door_shut".into()), ["clips"]);
}

/// More keys along the same arc are the same motion: the game samples it.
#[test]
fn a_clip_with_more_keys_on_the_same_arc_is_near() {
    let old = cupboard();
    let mut new = old.clone();
    new.clips[0].times = vec![0.0, 0.25, 0.5, 0.75, 1.0];
    new.clips[0].turns = vec![0.0, 22.5, 45.0, 67.5, 90.0];
    let found = compare(&glb(&old), &glb(&new));
    assert!(!found.same);
    assert!(found.findings.is_empty(), "{:?}", found.findings);
}

/// One face of the block on the body's top, under a thousandth of the wood's area, with its normals turned round.
#[test]
fn a_normal_flipped_on_a_face_is_changed() {
    let found = planted(|m| {
        let p = prim(m, "body", "wood");
        for i in 0..p.positions.len() {
            if p.positions[i][1] == 0.83 && p.normals[i][1] == 1.0 {
                p.normals[i][1] = -1.0;
            }
        }
    });
    assert_eq!(found, ["normals"]);
}

#[test]
fn a_material_s_colour_changed_is_changed() {
    assert_eq!(planted(|m| m.materials[0].1[0] += 0.02), ["materials"]);
    // One step of 8 bits is rounding.
    let old = cupboard();
    let mut new = old.clone();
    new.materials[0].1[0] += 0.003;
    assert!(compare(&glb(&old), &glb(&new)).findings.is_empty());
}

#[test]
fn a_node_renamed_is_changed() {
    assert_eq!(
        planted(|m| node(m, "door").name = "hatch".into()),
        ["nodes"]
    );
}

#[test]
fn a_part_s_material_shading_or_texture_changed_is_changed() {
    // The knob in the door's wood: the brass is gone.
    assert!(planted(|m| prim(m, "body", "brass").material = 0).contains(&"materials"));
    // The door's baked shading three greys lighter.
    assert_eq!(
        planted(|m| prim(m, "door", "door_wood")
            .greys
            .iter_mut()
            .for_each(|g| *g = g.saturating_add(6200))),
        ["shading"]
    );
    // Its texture slid a centimetre along, and stretched by a twentieth.
    assert_eq!(
        planted(|m| prim(m, "door", "door_wood")
            .uvs
            .iter_mut()
            .for_each(|uv| uv[0] += 0.01)),
        ["uv"]
    );
    assert_eq!(
        planted(|m| prim(m, "door", "door_wood")
            .uvs
            .iter_mut()
            .for_each(|uv| uv[0] *= 1.05)),
        ["texels", "uv"]
    );
    // A face of the body gone: a sixth of the wood.
    let found = planted(|m| {
        let p = prim(m, "body", "wood");
        let top = |p: &Prim, t: &[u32; 3]| t.iter().all(|i| p.positions[*i as usize][1] == 0.8);
        let kept: Vec<[u32; 3]> = p.triangles.iter().filter(|t| !top(p, t)).copied().collect();
        p.triangles = kept;
    });
    assert!(
        found.contains(&"area") && found.contains(&"surface"),
        "{found:?}"
    );
    // And half of the wood's triangles.
    let found = planted(|m| {
        let p = prim(m, "body", "wood");
        p.triangles = p.triangles.iter().copied().step_by(2).collect();
    });
    assert!(found.contains(&"triangles"), "{found:?}");
}

/// A band of the body's top cut another way, as a bevel made by another generator would be: its squares split
/// across the other corners and from a point in their middle, which stands 0.3 mm proud.
fn band_cut_another_way(model: &mut Model) {
    let p = prim(model, "body", "wood");
    let on_band = |p: &Prim, t: &[u32; 3]| {
        t.iter().all(|i| {
            let q = p.positions[*i as usize];
            q[1] == 0.8 && q[2] >= 0.099 && p.normals[*i as usize][1] == 1.0
        })
    };
    let band: Vec<[u32; 3]> = p
        .triangles
        .iter()
        .filter(|t| on_band(p, t))
        .copied()
        .collect();
    assert_eq!(band.len(), 12, "a row of six squares");
    p.triangles = p
        .triangles
        .iter()
        .filter(|t| !on_band(p, t))
        .copied()
        .collect();
    for i in 0..6 {
        let (x0, x1) = (-0.3 + 0.1 * i as f32, -0.3 + 0.1 * (i + 1) as f32);
        let (z0, z1) = (0.2 - 0.4 / 6.0, 0.2);
        let mut at = |x: f32, y: f32, z: f32| p.vertex([x, y, z], [0.0, 1.0, 0.0], 1);
        let corners = [
            at(x0, 0.8, z0),
            at(x0, 0.8, z1),
            at(x1, 0.8, z1),
            at(x1, 0.8, z0),
        ];
        let middle = at((x0 + x1) / 2.0, 0.8003, (z0 + z1) / 2.0);
        for k in 0..4 {
            p.triangles.push([corners[k], corners[(k + 1) % 4], middle]);
        }
    }
}

#[test]
fn a_band_cut_another_way_within_a_third_of_a_millimetre_is_near() {
    let old = cupboard();
    let mut new = another_order(&old);
    band_cut_another_way(&mut new);
    let found = compare(&glb(&old), &glb(&new));
    assert!(!found.same);
    assert!(found.findings.is_empty(), "{:?}", found.findings);
    let n = &found.numbers;
    assert!(
        n.surface_max > 0.00025 && n.surface_max < 0.00035,
        "{}",
        n.surface_max
    );
    assert!(n.triangles > 0.0 && n.triangles < 0.05, "{}", n.triangles);
    // The same band a centimetre proud is no bevel.
    let mut far_out = new.clone();
    for p in &mut prim(&mut far_out, "body", "wood").positions {
        if p[1] == 0.8003 {
            p[1] = 0.81;
        }
    }
    assert!(metrics(&compare(&glb(&old), &glb(&far_out))).contains(&"surface"));
}

/// A checkout of its own: the cupboard and its far version in a pack, with the manifests, the centres, and the
/// content and furniture.json the gate reads.
struct Checkout(PathBuf);

impl Drop for Checkout {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const ENTRY: &str = r#"{"type": "model", "url": "../../models/fixture/cupboard.glb", "far": "../../models/fixture/cupboard_far.glb",
    "materials": {"wood": {"finish": "material.finish.wood"}, "brass": {"finish": "material.finish.metal"},
    "door_wood": {"finish": "material.finish.wood"}}, "motions": {"uses": {"fetch": "door_open"}},
    "perches": [[0.1, 0.83, 0.0], [-0.2, 0.8, 0.1]]}"#;

/// The far version: the body alone, coarser.
fn far() -> Model {
    Model {
        materials: vec![("wood".into(), [0.5, 0.3, 0.2, 1.0])],
        nodes: vec![Node {
            name: "cupboard_far".into(),
            translation: [0.0, 0.0, 0.0],
            prims: vec![Prim::new(0).boxed([-0.3, 0.0, -0.2], [0.3, 0.8, 0.22], 2)],
        }],
        clips: Vec::new(),
    }
}

impl Checkout {
    fn new(name: &str) -> Checkout {
        let dir =
            std::env::temp_dir().join(format!("ll-against-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let c = Checkout(dir);
        c.write(
            "web/public/content/base.json",
            br#"{"objects": [], "items": []}"#,
        );
        c.write("web/src/render/shared/characters/furniture.json", b"{}");
        c.write(
            "web/public/assets/manifest.json",
            br#"{"version": 1, "packs": ["packs/fixture/manifest.json"], "entries": {
                "material.finish.wood": {"type": "material", "texture": "wood"},
                "material.finish.metal": {"type": "material"}}}"#,
        );
        c.entry(ENTRY);
        c.write(
            "tools/art/centres.json",
            br#"{"model.cupboard": [0.0, 0.0]}"#,
        );
        c.write(
            "web/public/assets/models/fixture/cupboard.glb",
            &glb(&cupboard()),
        );
        c.write(
            "web/public/assets/models/fixture/cupboard_far.glb",
            &glb(&far()),
        );
        c
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.0.join(rel)
    }

    fn root(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }

    fn write(&self, rel: &str, data: &[u8]) {
        let path = self.path(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, data).unwrap();
    }

    /// The pack with the cupboard's entry as `entry`.
    fn entry(&self, entry: &str) {
        self.write(
            "web/public/assets/packs/fixture/manifest.json",
            format!(r#"{{"$comment": "a pack", "version": 1, "entries": {{"model.cupboard": {entry}}}}}"#).as_bytes(),
        );
    }

    /// This checkout's working tree against `old`'s, with `accepted` as port-accepted.json.
    fn against(&self, old: &Checkout, accepted: &str) -> Outcome {
        self.write("accepted.json", accepted.as_bytes());
        against(&Options {
            root: self.root(),
            old: Source::Dir(old.root()),
            new: Source::Tree,
            keys: Vec::new(),
            family: None,
            accepted: Some(self.path("accepted.json").to_string_lossy().into_owned()),
        })
        .unwrap()
    }
}

fn row<'a>(outcome: &'a Outcome, name: &str) -> &'a ll_tools::checkglb::against::Row {
    outcome
        .rows
        .iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("no row {name}:\n{}", outcome.text))
}

fn found(outcome: &Outcome, name: &str) -> Vec<&'static str> {
    row(outcome, name)
        .findings
        .iter()
        .map(|f| f.metric)
        .collect()
}

#[test]
fn a_checkout_against_another_is_compared_model_by_model() {
    let (old, new) = (Checkout::new("rows-old"), Checkout::new("rows-new"));
    new.write(
        "web/public/assets/models/fixture/cupboard.glb",
        &glb(&another_order(&cupboard())),
    );
    let outcome = new.against(&old, "{}");
    assert!(!outcome.failed(), "{}", outcome.text);
    assert_eq!(
        outcome.text,
        "1 models, 1 far versions and 2 manifests: 4 same, 0 near, 0 CHANGED\n"
    );
    assert_eq!(
        outcome
            .rows
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>(),
        [
            "model.cupboard",
            "model.cupboard far",
            "manifest.json",
            "packs/fixture/manifest.json"
        ]
    );

    // A fault in the model, the far version left alone.
    let mut dented = cupboard();
    node(&mut dented, "door").translation[0] += 0.001;
    new.write(
        "web/public/assets/models/fixture/cupboard.glb",
        &glb(&dented),
    );
    let outcome = new.against(&old, "{}");
    assert!(outcome.failed());
    assert_eq!(found(&outcome, "model.cupboard"), ["nodes"]);
    assert_eq!(row(&outcome, "model.cupboard far").class, Class::Same);
    assert!(
        outcome
            .text
            .starts_with("CHANGED  model.cupboard  (differs in nodes; ")
            && outcome.text.contains(
                "\n         nodes: door: its origin moved 1.000 mm, at most 0.100 mm (from "
            )
            && outcome.text.ends_with("3 same, 0 near, 1 CHANGED\n"),
        "{}",
        outcome.text
    );

    // A file gone.
    std::fs::remove_file(new.path("web/public/assets/models/fixture/cupboard_far.glb")).unwrap();
    let outcome = new.against(&old, "{}");
    assert_eq!(found(&outcome, "model.cupboard far"), ["file"]);
    // (The gate says so too, of the model.)
    assert!(
        found(&outcome, "model.cupboard").contains(&"gate"),
        "{}",
        outcome.text
    );
}

#[test]
fn a_perch_moved_two_millimetres_is_changed_and_one_moved_less_is_said() {
    let (old, new) = (Checkout::new("perch-old"), Checkout::new("perch-new"));
    new.entry(&ENTRY.replace("[0.1, 0.83, 0.0]", "[0.102, 0.83, 0.0]"));
    let outcome = new.against(&old, "{}");
    assert_eq!(found(&outcome, "model.cupboard"), ["perches"]);
    assert!(
        outcome
            .text
            .contains("perches: perch 0 is 2.000 mm off, at most 1.000 mm"),
        "{}",
        outcome.text
    );

    new.entry(&ENTRY.replace("[0.1, 0.83, 0.0]", "[0.1004, 0.83, 0.0]"));
    let outcome = new.against(&old, "{}");
    let cupboard = row(&outcome, "model.cupboard");
    assert_eq!(
        (cupboard.class, cupboard.findings.len()),
        (Class::Near, 0),
        "{}",
        outcome.text
    );
    assert!(
        cupboard.notes[0].starts_with("perches up to 0.400 mm off (perch 0)"),
        "{:?}",
        cupboard.notes
    );

    // One fewer, and in another order.
    new.entry(&ENTRY.replace("[0.1, 0.83, 0.0], ", ""));
    assert_eq!(
        found(&new.against(&old, "{}"), "model.cupboard"),
        ["perches"]
    );
    new.entry(&ENTRY.replace(
        "[[0.1, 0.83, 0.0], [-0.2, 0.8, 0.1]]",
        "[[-0.2, 0.8, 0.1], [0.1, 0.83, 0.0]]",
    ));
    assert_eq!(
        found(&new.against(&old, "{}"), "model.cupboard"),
        ["perches"]
    );
}

#[test]
fn the_pack_entry_the_centre_and_the_rest_of_a_manifest_are_compared() {
    let (old, new) = (Checkout::new("pack-old"), Checkout::new("pack-new"));
    // The same entry with its fields in another order and a number written another way.
    new.entry(
        &ENTRY
            .replace(r#""type": "model", "#, "")
            .replace(r#""perches""#, r#""type": "model", "perches""#)
            .replace("0.83, 0.0]", "0.83, 0]"),
    );
    assert_eq!(
        row(&new.against(&old, "{}"), "model.cupboard").class,
        Class::Same
    );

    new.entry(&ENTRY.replace("material.finish.metal", "material.finish.wood"));
    let outcome = new.against(&old, "{}");
    // (Mapped to another finish it is another draw call too.)
    assert!(
        found(&outcome, "model.cupboard").contains(&"pack"),
        "{}",
        outcome.text
    );
    assert!(
        outcome.text.contains("pack: its entry: materials.brass.finish: \"material.finish.metal\" is now \"material.finish.wood\""),
        "{}",
        outcome.text
    );

    new.entry(ENTRY);
    new.write(
        "tools/art/centres.json",
        br#"{"model.cupboard": [0.0, 0.01]}"#,
    );
    let outcome = new.against(&old, "{}");
    assert_eq!(found(&outcome, "model.cupboard"), ["centre"]);
    // a checkout from before the models' tools moved keeps its centres where they were: read there
    let before = Checkout::new("centres-before");
    std::fs::remove_file(before.path("tools/art/centres.json")).unwrap();
    before.write(
        "tools/art/blender/centres.json",
        br#"{"model.cupboard": [0.0, 0.01]}"#,
    );
    let outcome = new.against(&before, "{}");
    assert!(
        found(&outcome, "model.cupboard").is_empty(),
        "{}",
        outcome.text
    );

    new.write(
        "tools/art/centres.json",
        br#"{"model.cupboard": [0.0, 0.0]}"#,
    );
    // (its `$comment` says what builds the pack: the game does not read it, and it is no finding)
    new.write(
        "web/public/assets/packs/fixture/manifest.json",
        format!(
            r#"{{"$comment": "another", "version": 1, "entries": {{"model.cupboard": {ENTRY}}}}}"#
        )
        .as_bytes(),
    );
    let outcome = new.against(&old, "{}");
    assert!(found(&outcome, "packs/fixture/manifest.json").is_empty());
    new.write(
        "web/public/assets/packs/fixture/manifest.json",
        format!(
            r#"{{"$comment": "another", "version": 2, "entries": {{"model.cupboard": {ENTRY}}}}}"#
        )
        .as_bytes(),
    );
    let outcome = new.against(&old, "{}");
    assert_eq!(found(&outcome, "packs/fixture/manifest.json"), ["pack"]);
    assert_eq!(row(&outcome, "model.cupboard").class, Class::Same);
}

#[test]
fn what_is_accepted_passes_with_its_reason_and_what_need_not_be_is_an_error() {
    let (old, new) = (Checkout::new("accept-old"), Checkout::new("accept-new"));
    new.entry(&ENTRY.replace("[0.1, 0.83, 0.0]", "[0.102, 0.83, 0.0]"));
    let outcome = new.against(
        &old,
        r#"{"model.cupboard": {"perches": "the block was moved on purpose"}}"#,
    );
    assert!(!outcome.failed(), "{}", outcome.text);
    let cupboard = row(&outcome, "model.cupboard");
    assert_eq!((cupboard.class, cupboard.accepted.len()), (Class::Near, 1));
    assert!(
        outcome
            .text
            .contains("accepted: perches: perch 0 is 2.000 mm off, at most 1.000 mm")
            && outcome.text.contains("(the block was moved on purpose)\n"),
        "{}",
        outcome.text
    );

    // Not needed, no reason, no metric, no such row.
    for (accepted, error) in [
        (
            r#"{"model.cupboard": {"perches": "why", "nodes": "why"}}"#,
            "model.cupboard: nodes is accepted, but the row passes without it",
        ),
        (
            r#"{"model.cupboard": {"perches": " "}}"#,
            "model.cupboard: perches needs its reason",
        ),
        (
            r#"{"model.cupboard": {"looks": "fine"}}"#,
            "model.cupboard: 'looks' is no metric",
        ),
        (
            r#"{"model.cupboard": {"perches": "why"}, "model.sideboard": {"nodes": "why"}}"#,
            "model.sideboard: no model, far version or manifest of that name is compared",
        ),
    ] {
        let outcome = new.against(&old, accepted);
        assert!(outcome.failed(), "{accepted}");
        assert!(
            outcome.errors.iter().any(|e| e.starts_with(error))
                && outcome.text.contains("error: port-accepted.json: "),
            "{accepted}: {:?}",
            outcome.errors
        );
    }
}

/// A far version Blender decimated is another on every run of Blender: listed as `decimated` it is held to its
/// size, its model's floor, its triangles and 6 cm.
#[test]
fn a_decimated_far_version_is_held_to_its_own_rule() {
    let (old, new) = (Checkout::new("far-old"), Checkout::new("far-new"));
    // Made again: each face `n` squares a side, its sides `dent` in at their middles, the whole `lift`ed and
    // `wider`.
    let made = |n: u32, dent: f32, lift: f32, wider: f32| {
        let mut model = far();
        model.nodes[0].prims = vec![Prim::new(0).boxed([-0.3, 0.0, -0.2], [0.3, 0.8, 0.22], n)];
        for p in &mut model.nodes[0].prims[0].positions {
            if p[1] == 0.4 && p[0].abs() == 0.3 {
                p[0] -= dent * p[0].signum();
            }
            p[0] *= wider;
            p[1] += lift;
        }
        glb(&model)
    };
    let far_file = "web/public/assets/models/fixture/cupboard_far.glb";
    new.write(far_file, &made(2, 0.01, 0.0, 1.0));
    let outcome = new.against(&old, "{}");
    assert!(
        found(&outcome, "model.cupboard far").contains(&"surface"),
        "{}",
        outcome.text
    );

    let listed = r#"{"model.cupboard far": {"decimated": "decimated, not repeated"}}"#;
    let outcome = new.against(&old, listed);
    assert!(!outcome.failed(), "{}", outcome.text);
    assert_eq!(row(&outcome, "model.cupboard far").class, Class::Near);
    assert!(
        outcome.text.contains(
            "accepted: decimated: held to the decimated far versions' rule (decimated, not repeated)"
        ),
        "{}",
        outcome.text
    );

    // Over the rule: 7 cm in, 3 % wider, a centimetre off the model's floor, and over a tenth more triangles.
    for (file, metric) in [
        (made(2, 0.07, 0.0, 1.0), "surface"),
        (made(2, 0.01, 0.0, 1.03), "size"),
        (made(2, 0.01, 0.01, 1.0), "bounds"),
        (made(3, 0.0, 0.0, 1.0), "triangles"),
    ] {
        new.write(far_file, &file);
        let outcome = new.against(&old, listed);
        assert!(
            found(&outcome, "model.cupboard far").contains(&metric),
            "{metric}: {}",
            outcome.text
        );
    }
    // The same file, or one within the other rows, passes without the rule: no error, it says what the file is.
    new.write(far_file, &glb(&far()));
    let outcome = new.against(&old, listed);
    assert!(
        !outcome.failed() && outcome.errors.is_empty(),
        "{}",
        outcome.text
    );
    assert_eq!(row(&outcome, "model.cupboard far").class, Class::Same);
    // It is a far version's.
    let outcome = new.against(
        &old,
        r#"{"model.cupboard": {"decimated": "decimated, not repeated"}}"#,
    );
    assert!(
        outcome.errors[0].starts_with("model.cupboard: decimated is a far version's"),
        "{:?}",
        outcome.errors
    );
}

/// The list in the repo names far versions the manifests have, each with its reason.
#[test]
fn the_repo_s_list_names_far_versions() {
    let root = ll_tools::checkglb::repo_root();
    let list: serde_json::Value = serde_json::from_slice(
        &std::fs::read(format!("{root}/tools/art/port-accepted.json")).unwrap(),
    )
    .unwrap();
    let report = ll_tools::checkglb::gate(&ll_tools::checkglb::Paths::repo(), &[]);
    for (name, metrics) in list.as_object().unwrap() {
        for (metric, why) in metrics.as_object().unwrap() {
            assert!(!why.as_str().unwrap().trim().is_empty(), "{name}: {metric}");
            if metric == "decimated" {
                let key = name
                    .strip_suffix(" far")
                    .unwrap_or_else(|| panic!("{name}"));
                let line = report
                    .text
                    .lines()
                    .find(|l| l.split_whitespace().nth(1) == Some(key));
                assert!(
                    line.is_some_and(|l| l.contains("(far: ")),
                    "{name} has no far version"
                );
            }
        }
    }
}

/// A build of some models (no manifest of its own) is compared in the working tree's place, for what it has.
#[test]
fn a_build_of_some_models_stands_in_for_the_working_tree() {
    let tree = Checkout::new("build-tree");
    let built = Checkout(tree.path("target/art"));
    let mut dented = cupboard();
    node(&mut dented, "door").translation[0] += 0.001;
    let file = built.path("models/fixture/cupboard.glb");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, glb(&dented)).unwrap();
    let options = |keys: &[&str], family: Option<&str>| Options {
        root: tree.root(),
        old: Source::Tree,
        new: Source::Dir(built.root()),
        keys: keys.iter().map(|k| k.to_string()).collect(),
        family: family.map(str::to_string),
        accepted: Some(tree.path("none.json").to_string_lossy().into_owned()),
    };
    let outcome = against(&options(&[], None)).unwrap();
    assert_eq!(
        found(&outcome, "model.cupboard"),
        ["nodes"],
        "{}",
        outcome.text
    );
    assert_eq!(row(&outcome, "model.cupboard far").class, Class::Same);
    // By family (the models' folder or the pack's, however it is written) and by key, every row is listed.
    for family in ["fixture", "pack_Fixture"] {
        let outcome = against(&options(&[], Some(family))).unwrap();
        assert!(
            outcome.text.contains("\nsame     model.cupboard far\n"),
            "{}",
            outcome.text
        );
    }
    assert!(
        against(&options(&[], Some("kitchen")))
            .err()
            .unwrap()
            .starts_with("no model is of the family 'kitchen' (fixture)")
    );
    let outcome = against(&options(&["model.cupboard", "model.none"], None)).unwrap();
    assert_eq!(found(&outcome, "model.none"), ["pack"]);
    assert_eq!(outcome.rows.len(), 3);
    drop(built);
}

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The old side as a revision: what was committed, whatever the working tree holds now.
#[test]
fn a_revision_is_read_from_git() {
    let tree = Checkout::new("git");
    git(&tree.0, &["init", "-q"]);
    git(&tree.0, &["add", "."]);
    git(&tree.0, &["commit", "-q", "-m", "the cupboard"]);
    let options = |rev: &str| Options {
        root: tree.root(),
        old: Source::Rev(rev.into()),
        new: Source::Tree,
        keys: Vec::new(),
        family: None,
        accepted: None,
    };
    let outcome = against(&options("HEAD")).unwrap();
    assert_eq!(
        outcome.text,
        "1 models, 1 far versions and 2 manifests: 4 same, 0 near, 0 CHANGED\n"
    );

    let mut dented = cupboard();
    node(&mut dented, "door").translation[0] += 0.001;
    tree.write(
        "web/public/assets/models/fixture/cupboard.glb",
        &glb(&dented),
    );
    tree.entry(&ENTRY.replace("[0.1, 0.83, 0.0]", "[0.102, 0.83, 0.0]"));
    let outcome = against(&options("HEAD")).unwrap();
    assert_eq!(
        found(&outcome, "model.cupboard"),
        ["perches", "nodes"],
        "{}",
        outcome.text
    );
    // Committed, it is the revision before that differs.
    git(&tree.0, &["commit", "-q", "-a", "-m", "the door moved"]);
    assert!(!against(&options("HEAD")).unwrap().failed());
    assert_eq!(
        found(&against(&options("HEAD^")).unwrap(), "model.cupboard"),
        ["perches", "nodes"]
    );
    assert!(against(&options("no-such-revision")).is_err());
}
