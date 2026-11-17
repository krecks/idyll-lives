//! Which of the points that sit on one another survives a merge, against Blender 5.2's `find_doubles`
//! (tests/golden/doubles.txt: points, the distance, and each point's target).
mod common;

use artkit::math::V3;
use common::*;

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
