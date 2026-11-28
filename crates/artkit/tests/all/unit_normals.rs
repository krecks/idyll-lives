//! No file is written with a normal that isn't a unit vector: the gate's validator fails one
//! (ACCESSOR_VECTOR3_NON_UNIT). Held over the bevelled parts of the goldens (tests/golden/bevel.txt,
//! bevel_placed.txt), over boxes bevelled by half their thickness and more and slabs with chamfered holes, each
//! written where it was built and joined after another part off the origin, as a family joins them (where Blender,
//! and the kernel with it, reads no normal at all at some corners), and over a mesh whose own normals are no
//! vectors to write.

use crate::common::*;
use artkit::forms::{Extrude, Holes, Plane, extrude, rounded_rect};
use artkit::materials::{Finish, Finishes, Mat};
use artkit::math::{M4, P3, V3};
use artkit::mesh::Mesh;
use artkit::scene::{self, Obj};
use artkit::shapes::{Cube, Round, cone, cube, cylinder};
use std::collections::BTreeMap;

fn finishes() -> Finishes {
    let mut finishes = Finishes::new();
    finishes.insert(
        "satin".into(),
        Finish {
            roughness: 0.5,
            metallic: 0.0,
            textured: false,
        },
    );
    finishes
}

fn body() -> Option<Mat> {
    Some(Mat::new("body", "#C0683F", "satin"))
}

/// The normals of the file `objs` are written as, read back.
fn written(objs: &[Obj]) -> Vec<[f32; 3]> {
    let file = artkit::glb::read(&artkit::glb::write(objs, false, &finishes())).unwrap();
    file.meshes
        .iter()
        .flat_map(|m| &m.primitives)
        .flat_map(|p| p.normals.iter().copied())
        .collect()
}

/// What a part's file holds: (its normals, those that aren't unit vectors, those written straight up).
fn tally(objs: &[Obj]) -> (usize, usize, usize) {
    let normals = written(objs);
    let off = normals
        .iter()
        .filter(|n| {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            len.is_nan() || (len - 1.0).abs() >= 1e-4
        })
        .count();
    let up = normals
        .iter()
        .filter(|n| n[0] == 0.0 && n[1] == 1.0 && n[2] == 0.0)
        .count();
    (normals.len(), off, up)
}

/// A part to build: at a place, with a material.
type Build = Box<dyn Fn(P3) -> Obj>;

fn a_box(size: (f64, f64, f64), bevel: f64, bevel_segments: usize) -> Build {
    Box::new(move |at| {
        cube(
            "part",
            size,
            Cube {
                at,
                material: body(),
                bevel,
                bevel_segments,
                ..Cube::default()
            },
        )
    })
}

fn a_slab(
    loops: Vec<Vec<(f64, f64)>>,
    depth: f64,
    plane: Plane,
    bevel: f64,
    bevel_segments: usize,
) -> Build {
    Box::new(move |at| {
        extrude(
            "part",
            &loops[0],
            depth,
            Extrude {
                at,
                material: body(),
                holes: Holes(loops[1..].to_vec()),
                plane,
                bevel,
                bevel_segments,
                ..Extrude::default()
            },
        )
    })
}

/// The bevelled parts of a golden that the kernel builds from what was asked for.
fn golden_parts(file: &str) -> Vec<(String, Build)> {
    let mut out: Vec<(String, Build)> = Vec::new();
    let mut recs: Vec<(String, BTreeMap<String, String>, String)> = Vec::new();
    for line in golden(file).lines().filter(|l| !l.starts_with('#')) {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        if key == "rec" {
            let mut words = rest.split(' ');
            let name = words.next().unwrap().to_string();
            let kv = words
                .filter_map(|w| w.split_once('='))
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect();
            recs.push((name, kv, String::new()));
        } else if key == "loops" {
            recs.last_mut().unwrap().2 = rest.to_string();
        }
    }
    for (name, kv, loops) in recs {
        let (width, segments) = (f64w(&kv["width"]), kv["seg"].parse().unwrap());
        if width <= 0.0 || kv["changed"] != "0" {
            continue;
        }
        let name = format!("{file} {name}");
        let f64s = |key: &str| -> Vec<f64> { kv[key].split(',').map(f64w).collect() };
        match kv["kind"].as_str() {
            "cube" => {
                let s = f64s("size");
                out.push((name, a_box((s[0], s[1], s[2]), width, segments)));
            }
            "cylinder" => {
                let (radius, depth, s) = (f64w(&kv["radius"]), f64w(&kv["depth"]), f64s("scale"));
                let verts = kv["verts"].parse().unwrap();
                out.push((
                    name,
                    Box::new(move |at| {
                        cylinder(
                            "part",
                            radius,
                            depth,
                            Round {
                                at,
                                material: body(),
                                verts,
                                scale: (s[0], s[1], s[2]).into(),
                                bevel: width,
                                bevel_segments: segments,
                                ..Round::default()
                            },
                        )
                    }),
                ));
            }
            _ => {
                let loops: Vec<Vec<(f64, f64)>> = loops
                    .split(" | ")
                    .map(|l| {
                        l.split(' ')
                            .filter(|w| !w.is_empty())
                            .map(|p| {
                                let (a, b) = p.split_once(',').unwrap();
                                (f64w(a), f64w(b))
                            })
                            .collect()
                    })
                    .collect();
                let plane = match kv["plane"].as_str() {
                    "XZ" => Plane::XZ,
                    "XY" => Plane::XY,
                    _ => Plane::YZ,
                };
                out.push((
                    name,
                    a_slab(loops, f64w(&kv["depth"]), plane, width, segments),
                ));
            }
        }
    }
    out
}

/// Boxes with no room for their bevel, and slabs whose holes' corners a bevel closes up: where Blender reads no
/// normal at some corners once the part is off the origin.
fn probes() -> Vec<(String, Build)> {
    let mut out: Vec<(String, Build)> = vec![
        // (the washer's control strip, the karaoke machine's light strip)
        ("strip".into(), a_box((0.6, 0.012, 0.13), 0.006, 1)),
        ("light".into(), a_box((0.3, 0.006, 0.012), 0.003, 1)),
        // (the built-in fridge's frame round its door, the chalkboard's frame)
        (
            "x_frame".into(),
            a_slab(
                vec![
                    rounded_rect(0.78, 0.62, 0.004, 1),
                    rounded_rect(0.64, 0.48, 0.003, 1),
                ],
                0.025,
                Plane::XZ,
                0.003,
                1,
            ),
        ),
        (
            "chalkboard".into(),
            a_slab(
                vec![
                    rounded_rect(0.62, 0.96, 0.03, 2),
                    rounded_rect(0.52, 0.8, 0.006, 1),
                ],
                0.03,
                Plane::XZ,
                0.006,
                2,
            ),
        ),
    ];
    for thin in [0.002, 0.004, 0.006, 0.012, 0.045] {
        for (share, what) in [
            (0.4, "under half"),
            (0.5, "half"),
            (0.75, "over half"),
            (1.0, "all"),
        ] {
            for segments in 1..=3 {
                for axis in 0..3 {
                    let mut size = [0.3, 0.13, 0.21];
                    size[axis] = thin;
                    out.push((
                        format!(
                            "box {thin} thin along {} bevelled by {what} of it in {segments}",
                            ["x", "y", "z"][axis]
                        ),
                        a_box((size[0], size[1], size[2]), thin * share, segments),
                    ));
                }
            }
        }
    }
    out
}

#[test]
fn no_part_is_written_with_a_normal_that_isnt_a_unit_vector() {
    let mut parts = golden_parts("bevel.txt");
    parts.extend(golden_parts("bevel_placed.txt"));
    let from_goldens = parts.len();
    parts.extend(probes());
    let places = [
        P3::ZERO,
        P3::from((0.0, -0.174, 0.715)),
        P3::from((-0.2, 0.632, 0.52)),
    ];
    let (mut files, mut normals, mut bad, mut unread, mut with_unread) = (0, 0, 0, 0, 0);
    let mut report = Vec::new();
    for (name, build) in &parts {
        let mut part_unread = 0;
        for (i, &at) in places.iter().enumerate() {
            for join in [false, true] {
                scene::reset();
                let obj = if join {
                    let first = cube(
                        "first",
                        (0.1, 0.06, 0.04),
                        Cube {
                            at: (0.05, -0.3, 0.31).into(),
                            material: body(),
                            ..Cube::default()
                        },
                    );
                    artkit::parts::join(&[first, build(at)], "joined", Some(P3::ZERO))
                } else {
                    artkit::bevel::apply(build(at))
                };
                // (the corners the kernel reads no normal at, as Blender reads none there)
                let read = artkit::normals::corner_normals(&obj.read(|o| o.mesh.clone()));
                part_unread += read.iter().filter(|n| **n == V3::ZERO).count();
                let (count, off, _) = tally(&[obj]);
                assert!(count > 0, "{name}: nothing written");
                files += 1;
                normals += count;
                if off > 0 {
                    bad += 1;
                    report.push(format!(
                        "{name} at place {i}{}: {off} of {count} normals aren't unit vectors",
                        if join { ", joined" } else { "" }
                    ));
                }
            }
        }
        unread += part_unread;
        with_unread += (part_unread > 0) as usize;
    }
    println!(
        "{files} files of {} parts ({from_goldens} from the goldens): {normals} normals; no normal read at {unread} corners of {with_unread} parts; {bad} files with a normal that isn't a unit vector",
        parts.len()
    );
    assert_eq!(bad, 0, "\n{}", report.join("\n"));
    // (the test is of something: the kernel does read no normal on some of them)
    assert!(with_unread > 0);
}

#[test]
fn a_joined_model_mirrored_and_smoothed_is_written_with_unit_normals() {
    // a bevelled box joined with plain parts, then as a script goes on with it: mirrored and its faces turned
    // back, smoothed by angle
    for step in 0..4 {
        scene::reset();
        a_box((0.1, 0.06, 0.04), 0.01, 2)(P3::from((0.1, 0.02, 0.0)));
        a_box((0.3, 0.006, 0.012), 0.003, 1)(P3::from((0.0, -0.174, 0.715)));
        let round = Round {
            at: (-0.1, 0.0, 0.05).into(),
            material: body(),
            verts: 10,
            ..Round::default()
        };
        cylinder("b", 0.03, 0.2, round.clone());
        cone("c", 0.05, 0.01, 0.2, Round { verts: 7, ..round });
        let obj = artkit::parts::join_all("m");
        obj.edit(|o| {
            if step >= 1 {
                o.mesh.transform(&M4::diagonal(V3::new(-1.0, 1.0, 1.0)));
            }
            if step >= 2 {
                o.mesh.flip_normals();
            }
            if step >= 3 {
                o.mesh.shade_smooth_by_angle(25.0);
            }
        });
        let (count, off, _) = tally(&[obj]);
        assert!(count > 0 && off == 0, "step {step}: {off} of {count}");
    }
}

#[test]
fn the_writer_makes_a_unit_vector_of_whatever_normal_it_is_given() {
    // a mesh whose own normals are no unit vectors: none at all, too short to tell once rounded to the file's
    // four decimals, long, not numbers
    let given = [
        V3::new(0.0, 0.0, 0.0),
        V3::new(-0.0, 0.0, -0.0),
        V3::new(0.000_04, -0.000_04, 0.000_04),
        V3::new(1e-30, 0.0, 0.0),
        V3::new(f32::MIN_POSITIVE, f32::MIN_POSITIVE, 0.0),
        V3::new(3.0, 4.0, 0.0),
        V3::new(0.3, 0.2, 0.1),
        V3::new(1e30, 0.0, 1e30),
        V3::new(f32::NAN, 0.0, 1.0),
        V3::new(f32::INFINITY, 0.0, 0.0),
        V3::new(0.0, f32::NEG_INFINITY, f32::NAN),
        V3::new(0.0, 0.000_06, 0.0),
    ];
    scene::reset();
    let mut mesh = Mesh::default();
    for (i, _) in given.chunks(4).enumerate() {
        let (x, v0) = (i as f32, mesh.verts.len() as u32);
        mesh.verts.extend([
            V3::new(x, 0.0, 0.0),
            V3::new(x + 0.5, 0.0, 0.0),
            V3::new(x + 0.5, 0.5, 0.0),
            V3::new(x, 0.5, 0.0),
        ]);
        mesh.push_face(&[v0, v0 + 1, v0 + 2, v0 + 3], 0, false);
    }
    mesh.set_material(body());
    mesh.normals = Some(given.to_vec());
    let obj = scene::add("odd", mesh);
    let (count, off, up) = tally(&[obj]);
    assert_eq!(off, 0, "{:?}", written(&[obj]));
    // (the three with a length that can be told are themselves; the others, straight up)
    assert!(count >= 4 && up >= 1, "{count} written, {up} straight up");
    let file = written(&[obj]);
    assert!(file.contains(&[0.6, 0.0, -0.8]), "{file:?}");
}
