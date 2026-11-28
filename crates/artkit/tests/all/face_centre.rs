//! A face's centre against Blender 5.2's `MeshPolygon.center` (tests/golden/face_centre.txt): a UV sphere's
//! triangles and quads, turned and squashed, cylinders' caps of 5, 7 and 24, a paper cup of 18 sides, and polygons
//! of 3 to 100 corners anywhere from millimetres to a hundred metres out: every centre to the bit.

use crate::common::*;
use artkit::math::V3;
use artkit::mesh::{Mesh, centre_of};

#[test]
fn a_faces_centre_is_blenders() {
    let all = golden_meshes("face_centre.txt");
    assert_eq!(all.len(), 6);
    let mut by_corners = [0_usize; 3];
    for g in &all {
        let points: Vec<V3> = g.verts.iter().map(|v| V3::new(v[0], v[1], v[2])).collect();
        let mesh = Mesh::from_pydata(points.clone(), &g.faces);
        let want = g.f32s("c");
        assert_eq!(want.len(), 3 * g.faces.len(), "{}", g.what);
        for (f, want) in want.chunks(3).enumerate() {
            let got = mesh.face_centre(f);
            assert_eq!(
                bits(&got.array()),
                bits(want),
                "{}: face {f} of {} corners",
                g.what,
                g.faces[f].len()
            );
            let corners: Vec<V3> = g.faces[f].iter().map(|&v| points[v as usize]).collect();
            assert_eq!(bits(&centre_of(&corners).array()), bits(want));
            by_corners[(g.faces[f].len() - 3).min(2)] += 1;
        }
    }
    // (triangles, quads, larger)
    assert_eq!(by_corners, [35, 217, 15]);
}
