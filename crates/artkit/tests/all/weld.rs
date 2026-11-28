//! Which of the points that sit on one another survives a merge, against Blender 5.2's `find_doubles`
//! (tests/golden/doubles.txt: points, the distance, and each point's target).

use crate::common::*;
use artkit::math::V3;

#[test]
fn the_same_point_of_each_cluster_survives_as_in_blender() {
    let (mut cases, mut bad) = (0, 0);
    for line in golden("doubles.txt").lines() {
        let (ins, outs) = line.split_once(" | ").unwrap();
        let w: Vec<&str> = ins.split(' ').collect();
        let dist = f32w(w[1]);
        let pts: Vec<V3> = f32s(&w[2..])
            .chunks(3)
            .map(|c| V3::new(c[0], c[1], c[2]))
            .collect();
        let want: Vec<i32> = outs.split(' ').map(|t| t.parse().unwrap()).collect();
        // (Blender's map leaves out a point that others merge into: it is its own target here)
        let got: Vec<i32> = artkit::weld::doubles(&pts, dist)
            .iter()
            .enumerate()
            .map(|(i, &t)| if t == i as i32 { -1 } else { t })
            .collect();
        let off = got.iter().zip(&want).filter(|(a, b)| a != b).count();
        println!(
            "case {cases}: {} points within {dist}: {off} targets differ",
            pts.len()
        );
        bad += (off > 0) as usize;
        cases += 1;
    }
    assert_eq!(bad, 0);
    assert!(cases >= 10);
}

#[test]
fn a_nan_point_does_not_run_the_sort_off_the_end() {
    let got = artkit::weld::doubles(&[V3::new(0.0, 0.0, 0.0), V3::new(f32::NAN, 0.0, 0.0)], 1e-6);
    assert_eq!(got.len(), 2);
}

#[test]
fn merging_doubles_after_shading_keeps_the_sharp_edges() {
    // a cube whose top face has its own copy (8) of corner 4: its two edges there have one face each until merged
    let mut verts: Vec<V3> = (0..8)
        .map(|i| V3::new((i & 1) as f32, (i >> 1 & 1) as f32, (i >> 2) as f32))
        .collect();
    verts.push(verts[4]);
    let faces = [
        vec![0, 2, 3, 1],
        vec![8, 5, 7, 6],
        vec![0, 1, 5, 4],
        vec![2, 6, 7, 3],
        vec![0, 4, 6, 2],
        vec![1, 3, 7, 5],
    ];
    let mut mesh = artkit::mesh::Mesh::from_pydata(verts, &faces);
    mesh.shade_smooth(30.0);
    assert_eq!(mesh.sharp.len(), 10);
    mesh.remove_doubles(1e-6);
    assert_eq!(mesh.verts.len(), 8);
    assert_eq!(mesh.sharp.len(), 10);
}
