//! The kernel's primitives against Blender 5.2's own (tests/golden/primitives.txt: each as its operator made it).
mod common;

use artkit::mesh::Mesh;
use common::*;

/// Builds what a golden's comment line names.
fn build(what: &str) -> Mesh {
    let w: Vec<&str> = what.split(' ').collect();
    let n = |i: usize| w[i].parse::<usize>().unwrap();
    let x = |i: usize| f64w(w[i]) as f32;
    match w[0] {
        "uv_sphere" => artkit::shapes::primitive::uv_sphere(n(1), n(2), x(3)),
        "cone" => artkit::shapes::primitive::cone(n(1), x(2), x(3), x(4)),
        "cylinder" => artkit::shapes::primitive::cone(n(1), x(2), x(2), x(3)),
        "cube" => artkit::shapes::primitive::cube(),
        "ico" => artkit::shapes::primitive::ico_sphere(n(1), x(2)),
        other => panic!("{other}"),
    }
}

#[test]
fn primitives_are_blenders_point_for_point() {
    let mut report = Vec::new();
    let mut bad = 0;
    for g in golden_meshes("primitives.txt") {
        let m = build(&g.what);
        if g.what.starts_with("ico") && !g.what.starts_with("ico 1") {
            // the same points to rounding, in an order of its own
            assert_eq!(m.verts.len(), g.verts.len(), "{}", g.what);
            assert_eq!(m.faces.len(), g.faces.len(), "{}", g.what);
            let far = g
                .verts
                .iter()
                .map(|w| {
                    m.verts
                        .iter()
                        .map(|v| (0..3).map(|i| (v[i] - w[i]).abs()).fold(0.0, f32::max))
                        .fold(f32::MAX, f32::min)
                })
                .fold(0.0, f32::max);
            report.push(format!("{}: its points within {far} of Blender's", g.what));
            bad += (far > 1e-6) as usize;
            continue;
        }
        // the same points, to the bit
        let Some((map, inexact, far)) = pair_points(&m.verts, &g.verts, 0.0) else {
            report.push(format!("{}: not Blender's points", g.what));
            bad += 1;
            continue;
        };
        let faces: Vec<Vec<u32>> = (0..m.faces.len()).map(|f| m.corners(f).to_vec()).collect();
        // the same faces, each from the same corner (which decides how a quad is cut), and the same triangles
        let same_faces = sorted(&renamed(&faces, &map)) == sorted(&g.faces);
        let same_tris = as_set(&renamed(&m.triangle_verts(), &map)) == as_set(&g.tris);
        let in_order = map.iter().enumerate().all(|(i, &j)| i == j as usize);
        report.push(format!("{}: {inexact} of {} points another copy (within {far}), in order {in_order}, faces {same_faces}, triangles {same_tris}", g.what, g.verts.len()));
        bad += (!same_faces || !same_tris || !in_order) as usize;
    }
    println!("{}", report.join("\n"));
    assert_eq!(bad, 0, "\n{}", report.join("\n"));
}
