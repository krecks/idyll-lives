//! `glb::read` and `glb::encode`: a file the writer made reads back and is written again to the same bytes (the
//! cypress, a model with clips on two parts in two orders); every committed model file reads, and what it reads
//! is what its encoding reads back as.
mod common;
#[path = "common/cypress.rs"]
mod cypress;

use artkit::family::build_model;
use artkit::glb::{self, Interpolation, Target};
use artkit::materials::{Finish, Finishes, Mat};
use artkit::math::p3;
use artkit::motion::{Ease, Path, key_channel};
use artkit::scene;
use artkit::shapes::{Cube, cube};
use artkit::{shade, uv};

fn round_trip(bytes: &[u8], what: &str) -> glb::Model {
    let model = glb::read(bytes).unwrap_or_else(|e| panic!("{what}: {e}"));
    assert!(
        glb::encode(&model) == bytes,
        "{what}: written again, not the same bytes"
    );
    model
}

#[test]
fn the_cypress_reads_back_and_is_written_again_to_its_bytes() {
    scene::set_finishes(cypress::finishes());
    let family = cypress::family();
    let out = build_model(&family, &family.models[0]).unwrap();
    let model = round_trip(&out.glb, "cypress");
    assert_eq!(model.generator, "idyll-lives artkit");
    assert_eq!(model.nodes.len(), 1);
    assert_eq!(model.nodes[0].name, "cypress");
    let p = &model.meshes[0].primitives[0];
    assert!(p.colours.is_some() && !p.normals.is_empty());
    round_trip(out.far.as_ref().unwrap(), "cypress far");
}

#[test]
fn clips_on_two_parts_in_two_orders_read_back_and_are_written_again_to_their_bytes() {
    let mut finishes = Finishes::new();
    finishes.insert(
        "gloss".into(),
        Finish {
            roughness: 0.2,
            metallic: 0.0,
            textured: false,
        },
    );
    finishes.insert(
        "wood".into(),
        Finish {
            roughness: 0.6,
            metallic: 0.0,
            textured: true,
        },
    );
    scene::reset();
    scene::set_finishes(finishes.clone());
    let body = cube(
        "body",
        (1.0, 0.5, 1.2),
        Cube {
            material: Some(Mat::new("body", "#AA5522", "gloss")),
            ..Default::default()
        },
    );
    let door = cube(
        "door",
        (0.04, 0.5, 1.0),
        Cube {
            material: Some(Mat::new("door", "#336699", "wood")),
            at: p3(0.6, 0.0, 0.1),
            ..Default::default()
        },
    );
    door.edit(|o| o.parent = Some(body));
    // the body keys "close" first, the door "open": the file's clips are [close, open], the door's keys "open" first
    key_channel(
        body,
        "close",
        Path::Rotation,
        2,
        &[(1, 0.0), (10, 0.5)],
        Ease::Smooth,
        0.0,
    );
    key_channel(
        body,
        "open",
        Path::Location,
        0,
        &[(1, 0.0), (5, 0.2)],
        Ease::Linear,
        0.0,
    );
    key_channel(
        door,
        "open",
        Path::Rotation,
        2,
        &[(1, 0.0), (8, 1.5)],
        Ease::Smooth,
        0.0,
    );
    key_channel(
        door,
        "open",
        Path::Location,
        1,
        &[(1, 0.0), (8, 0.1)],
        Ease::Smooth,
        0.0,
    );
    key_channel(
        door,
        "close",
        Path::Scale,
        1,
        &[(1, 1.0), (6, 0.5)],
        Ease::Linear,
        0.0,
    );
    let objs = [body, door];
    let texcoords = uv::ensure(&objs);
    shade::bake_ao(&objs, None, true);
    let bytes = glb::write(&objs, texcoords, &finishes);
    let model = round_trip(&bytes, "two parts");
    let names: Vec<&str> = model.animations.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["close", "open"]);
    // the nodes as Blender's exporter numbers them: a child before its parent
    let at = |name: &str| model.nodes.iter().position(|n| n.name == name).unwrap();
    let (body, door) = (at("body"), at("door"));
    assert_eq!((door, body), (0, 1));
    assert_eq!(
        model.nodes[door].clips,
        [1, 0],
        "the door's clips in the order its keys lie"
    );
    assert_eq!(model.nodes[body].children, [door]);
    assert_eq!(model.roots, [body]);
    let open = &model.animations[1];
    assert_eq!(
        open.channels
            .iter()
            .filter(|c| c.node == door)
            .map(|c| c.target)
            .collect::<Vec<_>>(),
        [Target::Translation, Target::Rotation]
    );
    assert!(
        open.channels
            .iter()
            .all(|c| c.interpolation == Interpolation::Linear)
    );
    assert!(
        open.channels
            .iter()
            .filter(|c| c.node == door)
            .all(|c| c.times.len() == 8)
    );
    // the textured part has its UVs, and the door's material its colour, linear
    let door_mesh = &model.meshes[model.nodes[door].mesh.unwrap()];
    assert!(door_mesh.primitives[0].uvs.is_some());
    let m = &model.materials[door_mesh.primitives[0].material.unwrap()];
    assert_eq!(m.name, "door");
    assert!((m.base[2] - artkit::materials::linear(0x99 as f64 / 255.0) as f32).abs() < 1e-6);
    // posed: the door half way through "open" stands where its keys put it
    let (t, _, _) = model.local(door, Some((1, 3.5 / 30.0)));
    let (rest, _, _) = model.local(door, None);
    assert!(t[1] > rest[1] - 1e-6 && t != rest, "{t:?} {rest:?}");
}

#[test]
fn a_file_cut_short_or_not_a_glb_is_an_error_not_a_panic() {
    scene::set_finishes(cypress::finishes());
    let family = cypress::family();
    let out = build_model(&family, &family.models[0]).unwrap();
    for n in [0, 4, 12, 19, 40, out.glb.len() / 2, out.glb.len() - 4] {
        assert!(glb::read(&out.glb[..n]).is_err(), "{n} bytes");
    }
    let mut odd = out.glb.clone();
    odd[0] = b'x';
    assert!(glb::read(&odd).is_err());
}

/// Every .glb under web/public/assets/models.
fn committed() -> Vec<std::path::PathBuf> {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/assets/models");
    let mut out = Vec::new();
    let mut todo = vec![root];
    while let Some(dir) = todo.pop() {
        for e in std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .flatten()
        {
            let path = e.path();
            if path.is_dir() {
                todo.push(path);
            } else if path.extension().is_some_and(|x| x == "glb") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn every_committed_model_reads_and_its_encoding_reads_back_the_same() {
    let files = committed();
    assert!(files.len() > 700, "{} files", files.len());
    for path in &files {
        let what = path.display();
        let bytes = std::fs::read(path).unwrap();
        let model = glb::read(&bytes).unwrap_or_else(|e| panic!("{what}: {e}"));
        let triangles: usize = model
            .meshes
            .iter()
            .flat_map(|m| &m.primitives)
            .map(|p| p.indices.len() / 3)
            .sum();
        assert!(triangles > 0, "{what}: no triangles");
        for p in model.meshes.iter().flat_map(|m| &m.primitives) {
            assert!(
                p.positions.iter().flatten().all(|x| x.is_finite()),
                "{what}"
            );
            assert_eq!(
                p.normals.len(),
                p.positions.len(),
                "{what}: a normal a vertex"
            );
            assert!(
                p.material.is_some(),
                "{what}: a primitive without its material"
            );
        }
        assert!(!model.drawn().is_empty(), "{what}: nothing in its scene");
        let again =
            glb::read(&glb::encode(&model)).unwrap_or_else(|e| panic!("{what} encoded: {e}"));
        if again != model {
            let first = |a: &glb::Model, b: &glb::Model| {
                if a.nodes != b.nodes {
                    return "nodes".to_string();
                }
                if a.materials != b.materials {
                    return format!("materials {:?} {:?}", a.materials, b.materials);
                }
                if a.animations != b.animations {
                    return "animations".to_string();
                }
                for (i, (m, n)) in a.meshes.iter().zip(&b.meshes).enumerate() {
                    for (j, (p, q)) in m.primitives.iter().zip(&n.primitives).enumerate() {
                        if p.colours != q.colours {
                            return format!("mesh {i} prim {j} colours");
                        }
                        if p.positions != q.positions {
                            return format!("mesh {i} prim {j} positions");
                        }
                        if p.normals != q.normals {
                            return format!("mesh {i} prim {j} normals");
                        }
                        if p.uvs != q.uvs {
                            return format!("mesh {i} prim {j} uvs");
                        }
                        if p.indices != q.indices {
                            return format!("mesh {i} prim {j} indices");
                        }
                    }
                }
                "other".to_string()
            };
            panic!(
                "{what}: its encoding reads back otherwise: {}",
                first(&model, &again)
            );
        }
    }
}
