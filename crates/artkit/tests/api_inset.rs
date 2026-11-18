//! bmesh.ops.inset_individual against Blender 5.2 (tests/golden/api_inset.txt): outdoor.py's mosaic (the round
//! patio table's 108 tiles, each drawn 3 mm in from its edges and raised 3 mm) and a small grid with a triangle
//! beside it under the operator's other settings.

mod common;

use artkit::inset_individual;
use artkit::materials::Mat;
use artkit::math::V3;
use artkit::mesh::Mesh;
use artkit::scene;
use common::{as_set, golden_meshes, pair_points, points_off, renamed};

#[test]
fn every_face_is_drawn_in_from_its_edges_as_blenders_inset_draws_it() {
    let all = golden_meshes("api_inset.txt");
    let mut cases = 0;
    for pair in all.chunks(2) {
        let (before, after) = (&pair[0], &pair[1]);
        let w: Vec<&str> = before.what.split(' ').collect();
        let number = |key: &str| -> f64 {
            let at = w.iter().position(|x| *x == key).unwrap();
            w[at + 1].parse().unwrap()
        };
        scene::reset();
        let mut mesh = Mesh::from_pydata(
            before
                .verts
                .iter()
                .map(|p| V3::new(p[0], p[1], p[2]))
                .collect(),
            &before.faces,
        );
        for m in 0..3 {
            mesh.slot(Mat::new(["a", "b", "c"][m], "#808080", "matte"));
        }
        for (face, m) in mesh.faces.iter_mut().zip(before.lines["mat"].split(' ')) {
            face.mat = m.parse().unwrap();
        }
        let obj = scene::add("tiles", mesh);
        inset_individual!(
            obj,
            thickness = number("thickness"),
            depth = number("depth"),
            use_even_offset = number("even") == 1.0,
            use_relative_offset = number("relative") == 1.0
        );
        let got = obj.read(|o| o.mesh.clone());
        let faces: Vec<Vec<u32>> = (0..got.faces.len())
            .map(|f| got.corners(f).to_vec())
            .collect();
        let mats: Vec<String> = got.faces.iter().map(|f| f.mat.to_string()).collect();
        if w[0] == "mosaic" {
            // Blender numbers the mosaic's new points by its mesh's memory: the ones its remove_doubles freed are
            // taken again from the last one down, the rest come after. The kernel numbers them after the old.
            // So: the same points to the bit, each once, and the same faces on the same materials, by the points
            let (map, inexact, _) =
                pair_points(&got.verts, &after.verts, 0.0).expect("the same points");
            assert_eq!(inexact, 0);
            let renamed = renamed(&faces, &map);
            assert_eq!(as_set(&renamed), as_set(&after.faces), "mosaic: faces");
            let key = |f: &[u32]| as_set(&[f.to_vec()]).remove(0);
            let want: std::collections::BTreeMap<Vec<u32>, &str> = after
                .faces
                .iter()
                .zip(after.lines["mat"].split(' '))
                .map(|(f, m)| (key(f), m))
                .collect();
            for (f, m) in renamed.iter().zip(&mats) {
                assert_eq!(want[&key(f)], m, "mosaic: a face's material");
            }
        } else {
            // the points to the bit and in Blender's order, the faces in Blender's order, each from the corner
            // Blender's starts at, and each on its material
            assert_eq!(
                points_off(&got.verts, &after.verts),
                (0, 0),
                "{}: points",
                w[0]
            );
            assert_eq!(faces, after.faces, "{}: faces, in order", w[0]);
            assert_eq!(mats.join(" "), after.lines["mat"], "{}: materials", w[0]);
        }
        println!(
            "{}: {} points, {} faces: Blender's",
            w[0],
            got.verts.len(),
            faces.len()
        );
        cases += 1;
    }
    assert_eq!(cases, 5);
}
