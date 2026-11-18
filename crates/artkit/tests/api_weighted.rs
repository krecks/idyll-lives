//! Blender's WEIGHTED_NORMAL modifier as the kitchen's glossy shells use it (bevel applied, shaded smooth by
//! angle, then the modifier with keep_sharp), against Blender 5.2 (tests/golden/api_weighted.txt: the mesh going
//! in and the normal at every corner coming out): a bevelled cabinet and door, the fridge's lofted shell, a
//! bevelled knob, and a box without keep_sharp.

mod common;

use artkit::math::V3;
use artkit::mesh::{Mesh, edge};
use artkit::scene;
use artkit::weighted_normal;
use common::golden_meshes;

/// How far a corner's normal may turn from Blender's, degrees: Blender keeps the modifier's normals as two angles
/// of sixteen bits in each run's own frame and reads them back from there.
const KEPT: f64 = 0.03;

#[test]
fn a_shells_corners_take_their_faces_normals_by_area_as_blenders_modifier_gives_them() {
    let all = golden_meshes("api_weighted.txt");
    let mut cases = 0;
    for pair in all.chunks(2) {
        let (before, after) = (&pair[0], &pair[1]);
        let w: Vec<&str> = before.what.split(' ').collect();
        let keep_sharp = w[2] == "1";
        scene::reset();
        let mut mesh = Mesh::from_pydata(
            before
                .verts
                .iter()
                .map(|p| V3::new(p[0], p[1], p[2]))
                .collect(),
            &before.faces,
        );
        for (face, s) in mesh.faces.iter_mut().zip(before.lines["smooth"].chars()) {
            face.smooth = s == '1';
        }
        for e in before.lines["sharp"].split(' ').filter(|e| !e.is_empty()) {
            let (a, b) = e.split_once('-').unwrap();
            mesh.sharp
                .insert(edge(a.parse().unwrap(), b.parse().unwrap()));
        }
        // (the normals it had going in: a bevel's)
        let had = before.f32s("n");
        mesh.normals = Some(had.chunks(3).map(|c| V3::new(c[0], c[1], c[2])).collect());
        let obj = scene::add("shell", mesh);
        weighted_normal!(obj, keep_sharp = keep_sharp);
        let got = obj.read(|o| o.mesh.normals.clone().unwrap());
        let want = after.f32s("n");
        assert_eq!(got.len() * 3, want.len(), "{}: corners", w[0]);
        let (mut worst, mut exact) = (0.0_f64, 0);
        for (g, wn) in got.iter().zip(want.chunks(3)) {
            // (the angle between two unit vectors from how far apart their ends are)
            let apart = ((g.x - wn[0]) as f64)
                .hypot((g.y - wn[1]) as f64)
                .hypot((g.z - wn[2]) as f64);
            let off = (2.0 * (apart / 2.0).min(1.0).asin()).to_degrees();
            worst = worst.max(off);
            exact += (g.array() == [wn[0], wn[1], wn[2]]) as usize;
        }
        println!(
            "{}: {} corners, {exact} to the bit, the farthest {worst:.4} degrees off",
            w[0],
            got.len()
        );
        assert!(
            worst <= KEPT,
            "{}: a corner's normal {worst} degrees off",
            w[0]
        );
        cases += 1;
    }
    assert_eq!(cases, 5);
}
