//! Faces turned over on a mesh with normals of its own, against Blender 5.2's (tests/golden/flip_normals.txt): a
//! bevelled box joined with plain parts, mirrored by its mesh's own transform and its faces then turned back, as a
//! script mirrors a model it has joined.

use crate::common::*;
use artkit::math::{M4, V3};
use artkit::mesh::Mesh;
use artkit::normals::corner_normals;
use artkit::scene;
use artkit::shapes::{Cube, Round, cone, cube, cylinder};
use std::collections::BTreeMap;

/// A stage of a golden record: Blender's mesh then, with a normal a corner.
struct Stage {
    name: String,
    points: Vec<V3>,
    faces: Vec<Vec<u32>>,
    normals: Vec<V3>,
}

struct Rec {
    name: String,
    axis: String,
    /// The normals a part was given, one a point.
    given: Vec<V3>,
    stages: Vec<Stage>,
}

fn points(hex: &str) -> Vec<V3> {
    let x: Vec<f32> = (0..hex.len() / 8)
        .map(|i| f32w(&hex[i * 8..i * 8 + 8]))
        .collect();
    x.chunks(3).map(|c| V3::new(c[0], c[1], c[2])).collect()
}

fn records() -> Vec<Rec> {
    let mut out: Vec<Rec> = Vec::new();
    for line in golden("flip_normals.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        fn stage(out: &mut [Rec]) -> &mut Stage {
            out.last_mut().unwrap().stages.last_mut().unwrap()
        }
        match key {
            "rec" => {
                let (name, axis) = rest.split_once(" axis=").unwrap();
                out.push(Rec {
                    name: name.to_string(),
                    axis: axis.to_string(),
                    given: Vec::new(),
                    stages: Vec::new(),
                });
            }
            "stage" => out.last_mut().unwrap().stages.push(Stage {
                name: rest.to_string(),
                points: Vec::new(),
                faces: Vec::new(),
                normals: Vec::new(),
            }),
            "g" => out.last_mut().unwrap().given = points(rest),
            "v" => stage(&mut out).points = points(rest),
            "f" => {
                stage(&mut out).faces = rest
                    .split(' ')
                    .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
                    .collect()
            }
            "n" => stage(&mut out).normals = points(rest),
            _ => {}
        }
    }
    out
}

/// The parts of a record, joined; or its one part, given the record's normals for its own.
fn built(r: &Rec) -> Mesh {
    scene::reset();
    let name = r.name.as_str();
    if let Some(what) = name.strip_prefix("given_") {
        let part = if what == "cone" {
            let verts = 7;
            cone(
                "c",
                0.05,
                0.02,
                0.1,
                Round {
                    verts,
                    ..Round::default()
                },
            )
        } else {
            cube("b", (0.1, 0.06, 0.04), Cube::default())
        };
        let mut mesh = part.read(|o| o.mesh.clone());
        mesh.normals = Some(artkit::normals::custom_from_verts(&mesh, &r.given));
        return mesh;
    }
    if name == "box" {
        cube(
            "a",
            (0.1, 0.06, 0.04),
            Cube {
                at: (0.1, 0.0, 0.0).into(),
                bevel: 0.01,
                bevel_segments: 1,
                ..Cube::default()
            },
        );
        cube(
            "b",
            (0.05, 0.05, 0.05),
            Cube {
                at: (-0.1, 0.0, 0.0).into(),
                ..Cube::default()
            },
        );
    } else {
        cube(
            "a",
            (0.1, 0.06, 0.04),
            Cube {
                at: (0.1, 0.02, 0.0).into(),
                bevel: 0.01,
                bevel_segments: 2,
                ..Cube::default()
            },
        );
        cylinder(
            "b",
            0.03,
            0.2,
            Round {
                at: (-0.1, 0.0, 0.05).into(),
                verts: 10,
                ..Round::default()
            },
        );
        cone(
            "c",
            0.05,
            0.01,
            0.2,
            Round {
                at: (0.0, 0.2, 0.1).into(),
                verts: 7,
                ..Round::default()
            },
        );
    }
    artkit::parts::join_all("m").read(|o| o.mesh.clone())
}

/// How a mesh is off a stage of Blender's: (points not Blender's to the bit, faces that aren't Blender's from the
/// same corner on, the most a corner's normal is turned from Blender's in degrees, how many are over a degree off).
fn off(mesh: &Mesh, want: &Stage) -> (usize, usize, f64, usize) {
    let (points, _) = points_off(
        &mesh.verts,
        &want.points.iter().map(|p| p.array()).collect::<Vec<_>>(),
    );
    let mut golden: BTreeMap<Vec<u32>, Vec<V3>> = BTreeMap::new();
    let mut at = 0;
    for f in &want.faces {
        golden.insert(f.clone(), want.normals[at..at + f.len()].to_vec());
        at += f.len();
    }
    let mine = corner_normals(mesh);
    let (mut missing, mut worst, mut turned) = (0, 0.0_f64, 0);
    for f in 0..mesh.faces.len() {
        let Some(normals) = golden.get(mesh.corners(f)) else {
            missing += 1;
            continue;
        };
        let start = mesh.faces[f].start as usize;
        for (i, w) in normals.iter().enumerate() {
            let n = mine[start + i];
            let cos = n.dot(*w) / (n.length() * w.length()).max(1e-30);
            let degrees = cos.clamp(-1.0, 1.0).acos().to_degrees();
            if (0..3).any(|k| n[k] != w[k]) {
                worst = worst.max(degrees);
            }
            turned += (degrees > 1.0) as usize;
        }
    }
    (points, missing, worst, turned)
}

#[test]
fn a_joined_meshs_normals_follow_its_faces_turned_over() {
    let mut report = Vec::new();
    let mut bad = 0;
    for r in records() {
        let mut mesh = built(&r);
        for stage in &r.stages {
            match stage.name.as_str() {
                "mirrored" => {
                    let mut k = V3::new(1.0, 1.0, 1.0);
                    k[if r.axis == "x" { 0 } else { 1 }] = -1.0;
                    mesh.transform(&M4::diagonal(k));
                }
                "flipped" => mesh.flip_normals(),
                _ => {}
            }
            let (points, missing, worst, turned) = off(&mesh, stage);
            report.push(format!(
                "{} (mirror: {}) {}: {points} points not Blender's, {missing} faces not Blender's, normals within {worst:.4} degrees ({turned} of {} over one)",
                r.name,
                r.axis,
                stage.name,
                mesh.loops.len()
            ));
            bad += (points > 0 || missing > 0 || worst > 0.05) as usize;
        }
    }
    println!("{}", report.join("\n"));
    assert_eq!(bad, 0, "\n{}", report.join("\n"));
}
