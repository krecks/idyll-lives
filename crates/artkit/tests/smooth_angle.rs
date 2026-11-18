//! Smooth by angle against Blender 5.2's (tests/golden/smooth_angle.txt): the operator, which keeps the edges that
//! are sharp, and the mesh's own two calls, which don't, on meshes with a bevel's normals and without.
mod common;

use artkit::math::V3;
use artkit::mesh::{Mesh, edge};
use artkit::normals::corner_normals;
use common::*;
use std::collections::{BTreeMap, BTreeSet};

/// One run on a copy of a golden mesh: how it was smoothed, at what angle, and what Blender left.
struct Run {
    operator: bool,
    degrees: f64,
    after: BTreeSet<(u32, u32)>,
    normals: Vec<V3>,
}

/// A mesh of the golden as Blender held it, what it was built as, its normals as Blender read them and its runs.
struct Case {
    name: String,
    what: String,
    mesh: Mesh,
    normals: Vec<V3>,
    face_normals: Vec<V3>,
    runs: Vec<Run>,
}

fn points(hex: &str) -> Vec<V3> {
    let x: Vec<f32> = (0..hex.len() / 8)
        .map(|i| f32w(&hex[i * 8..i * 8 + 8]))
        .collect();
    x.chunks(3).map(|c| V3::new(c[0], c[1], c[2])).collect()
}

fn edges(line: &str) -> BTreeSet<(u32, u32)> {
    line.split(' ')
        .filter(|w| !w.is_empty())
        .map(|e| {
            let (a, b) = e.split_once('-').unwrap();
            edge(a.parse().unwrap(), b.parse().unwrap())
        })
        .collect()
}

fn cases() -> Vec<Case> {
    let mut out: Vec<Case> = Vec::new();
    let mut lines: BTreeMap<String, String> = BTreeMap::new();
    // (a mesh's lines are all read once its first run, or the next mesh, comes)
    fn built(case: &mut Case, lines: &BTreeMap<String, String>) {
        if !case.mesh.verts.is_empty() {
            return;
        }
        let faces: Vec<Vec<u32>> = lines["f"]
            .split(' ')
            .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
            .collect();
        let mut mesh = Mesh::from_pydata(points(&lines["v"]), &faces);
        for (f, c) in mesh.faces.iter_mut().zip(lines["s"].chars()) {
            f.smooth = c == '1';
        }
        mesh.sharp = edges(&lines["e"]);
        case.normals = points(&lines["n"]);
        case.face_normals = points(&lines["fn"]);
        if let Some(stored) = lines.get("c") {
            mesh.stored = Some(
                stored
                    .split(' ')
                    .map(|s| {
                        let (a, b) = s.split_once(',').unwrap();
                        Some([a.parse().unwrap(), b.parse().unwrap()])
                    })
                    .collect(),
            );
            mesh.normals = Some(case.normals.clone());
        }
        case.mesh = mesh;
    }
    for line in golden("smooth_angle.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "mesh" => {
                let (name, what) = rest.split_once(' ').unwrap();
                lines.clear();
                out.push(Case {
                    name: name.to_string(),
                    what: what.to_string(),
                    mesh: Mesh::default(),
                    normals: Vec::new(),
                    face_normals: Vec::new(),
                    runs: Vec::new(),
                });
            }
            "run" => {
                let case = out.last_mut().unwrap();
                built(case, &lines);
                let kv: BTreeMap<&str, &str> =
                    rest.split(' ').filter_map(|w| w.split_once('=')).collect();
                case.runs.push(Run {
                    operator: kv["way"] == "op",
                    degrees: kv["angle"].parse().unwrap(),
                    after: BTreeSet::new(),
                    normals: Vec::new(),
                });
            }
            "after" => out.last_mut().unwrap().runs.last_mut().unwrap().after = edges(rest),
            "an" => out.last_mut().unwrap().runs.last_mut().unwrap().normals = points(rest),
            _ => {
                lines.insert(key.to_string(), rest.to_string());
            }
        }
    }
    out
}

/// The most two lists of normals are apart, by coordinate.
fn apart(got: &[V3], want: &[V3]) -> f32 {
    assert_eq!(got.len(), want.len(), "normals");
    got.iter()
        .zip(want)
        .flat_map(|(g, w)| (0..3).map(move |i| (g[i] - w[i]).abs()))
        .fold(0.0, f32::max)
}

#[test]
fn a_meshs_own_normals_read_as_blenders_and_give_its_faces_theirs() {
    let mut report = Vec::new();
    for case in cases() {
        let corners = apart(&corner_normals(&case.mesh), &case.normals);
        let line = match case.mesh.own_face_normals() {
            Some(faces) => {
                let off = apart(&faces, &case.face_normals);
                let exact = faces
                    .iter()
                    .zip(&case.face_normals)
                    .filter(|(g, w)| (0..3).all(|i| g[i].to_bits() == w[i].to_bits()))
                    .count();
                // the faces' own normals, where their points are: not what such a mesh gives
                let turned = apart(&case.mesh.face_normals(), &case.face_normals);
                assert!(off < 2e-6, "{}: faces' normals {off:e} off", case.name);
                format!(
                    "corners within {corners:e}, faces within {off:e} ({exact} of {} to the bit; where their points are: {turned:.3} off)",
                    faces.len()
                )
            }
            None => {
                assert!(case.mesh.stored.is_none(), "{}", case.name);
                format!("corners within {corners:e}, no normals of its own")
            }
        };
        report.push(format!("{}: {line}", case.name));
        assert!(corners < 2e-6, "{}", report.last().unwrap());
    }
    println!("{}", report.join("\n"));
}

#[test]
fn smooth_by_angle_marks_the_edges_blender_marks() {
    let mut report = Vec::new();
    let mut bad = 0;
    for case in cases() {
        for run in &case.runs {
            let mut mesh = case.mesh.clone();
            if run.operator {
                mesh.shade_smooth_by_angle(run.degrees);
            } else {
                mesh.shade_smooth(run.degrees);
            }
            assert!(mesh.faces.iter().all(|f| f.smooth));
            let (missing, extra) = (
                run.after.difference(&mesh.sharp).count(),
                mesh.sharp.difference(&run.after).count(),
            );
            let normals = apart(&corner_normals(&mesh), &run.normals);
            // what telling the turn by where the faces' points are would mark
            let mut plain = case.mesh.clone();
            (plain.normals, plain.stored) = (None, None);
            if run.operator {
                plain.shade_smooth_by_angle(run.degrees);
            } else {
                plain.shade_smooth(run.degrees);
            }
            let by_points = plain.sharp.symmetric_difference(&run.after).count();
            report.push(format!(
                "{} {} at {} degrees: {} sharp edges before, {} after ({missing} missing, {extra} too many; by the faces' points {by_points} would differ), normals then within {normals:e}",
                case.name,
                if run.operator { "operator" } else { "mesh calls" },
                run.degrees,
                case.mesh.sharp.len(),
                run.after.len(),
            ));
            bad += (missing + extra > 0 || normals > 2e-6) as usize;
        }
    }
    println!("{}", report.join("\n"));
    assert_eq!(bad, 0, "\n{}", report.join("\n"));
}

#[test]
fn the_cabinet_gains_four_sharp_edges_not_thirty() {
    // the retro mini fridge's cabinet: its cavity's faces carry the cutter's stored normals, read in fans they
    // were not stored in, and all but four of the thirty edges that turn more than 40 degrees shade as smooth
    let case = cases().into_iter().find(|c| c.name == "cabinet").unwrap();
    assert!(case.what.contains("cavity"));
    let mut mesh = case.mesh.clone();
    mesh.shade_smooth_by_angle(40.0);
    assert_eq!(mesh.sharp.len() - case.mesh.sharp.len(), 4);
    let mut plain = case.mesh.clone();
    (plain.normals, plain.stored) = (None, None);
    plain.shade_smooth_by_angle(40.0);
    assert_eq!(plain.sharp.len() - case.mesh.sharp.len(), 30);
}
