//! bmesh.ops.subdivide_edges as bedroom_dressing.py's mirror glass uses it (cuts, use_grid_fill), against
//! Blender 5.2 (tests/golden/api_subdivide.txt: each mesh going in with its edges in Blender's order, the edges
//! chosen, and the mesh coming out): the cheval mirror's pane, panes of one and four cuts, a cube gridded all
//! round, a pane without the grid fill, a rail cut across its length, and two sheets of quads and a triangle.

use crate::common::{bits, golden_meshes};
use artkit::edit::edges;
use artkit::fbox;
use artkit::materials::Mat;
use artkit::math::V3;
use artkit::mesh::Mesh;
use artkit::place::GameFrame;
use artkit::scene;
use artkit::subdivide_edges;

const BODY: Mat = Mat::new("body", "#F3F3F1", "gloss");

/// A box of glass: its case in the golden file, its width, height and depth, where it stands.
type Glass = (&'static str, [f64; 3], (f64, f64, f64));

fn pairs(line: &str) -> Vec<(u32, u32)> {
    line.split(' ')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let (a, b) = w.split_once('-').unwrap();
            (a.parse().unwrap(), b.parse().unwrap())
        })
        .collect()
}

#[test]
fn chosen_edges_take_their_points_and_the_faces_their_cuts_as_blenders() {
    let all = golden_meshes("api_subdivide.txt");
    let mut cases = 0;
    for pair in all.chunks(2) {
        let (before, after) = (&pair[0], &pair[1]);
        let w: Vec<&str> = before.what.split(' ').collect();
        let (cuts, grid): (i32, bool) = (w[2].parse().unwrap(), w[4] == "1");
        scene::reset();
        let mesh = Mesh::from_pydata(
            before
                .verts
                .iter()
                .map(|p| V3::new(p[0], p[1], p[2]))
                .collect(),
            &before.faces,
        );
        let obj = scene::add("glass", mesh);
        let chosen = pairs(&before.lines["chosen"]);
        subdivide_edges!(obj, &chosen, cuts = cuts, use_grid_fill = grid);
        // the points to the bit and in Blender's order; the faces Blender's, each from Blender's corner, in order
        let (got_v, got_f) = obj.read(|o| {
            let v: Vec<f32> = o.mesh.verts.iter().flat_map(|v| v.array()).collect();
            let f: Vec<Vec<u32>> = (0..o.mesh.faces.len())
                .map(|f| o.mesh.corners(f).to_vec())
                .collect();
            (v, f)
        });
        let want_v: Vec<f32> = after.verts.iter().flatten().copied().collect();
        assert_eq!(bits(&got_v), bits(&want_v), "{}: points", w[0]);
        assert_eq!(got_f, after.faces, "{}: faces", w[0]);
        println!(
            "{}: {} points, {} faces: Blender's",
            w[0],
            got_v.len() / 3,
            got_f.len()
        );
        cases += 1;
    }
    assert_eq!(cases, 8);
}

#[test]
fn a_boxs_edges_come_in_blenders_order() {
    let all = golden_meshes("api_subdivide.txt");
    let f = GameFrame::default();
    let t = 0.004;
    let boxes: [Glass; 3] = [
        (
            "cheval",
            [0.46 - 0.035 + 0.01, 1.5 - 0.035 + 0.01, t],
            (0.0, 0.9, 0.009),
        ),
        ("cube", [0.3, 0.3, 0.3], (0.0, 0.0, 0.0)),
        ("rail", [1.0, 0.1, 0.1], (0.0, 0.0, 0.0)),
    ];
    for (label, [w, h, d], at) in boxes {
        let before = all
            .iter()
            .find(|m| m.name == "in" && m.what.starts_with(label))
            .unwrap();
        scene::reset();
        let obj = fbox!(f, "glass", w, h, d, at, BODY, bevel = 0);
        let (got_v, got_f) = obj.read(|o| {
            let v: Vec<f32> = o.mesh.verts.iter().flat_map(|v| v.array()).collect();
            let f: Vec<Vec<u32>> = (0..o.mesh.faces.len())
                .map(|f| o.mesh.corners(f).to_vec())
                .collect();
            (v, f)
        });
        let want_v: Vec<f32> = before.verts.iter().flatten().copied().collect();
        assert_eq!(bits(&got_v), bits(&want_v), "{label}: the box's points");
        assert_eq!(got_f, before.faces, "{label}: the box's faces");
        assert_eq!(
            edges(obj),
            pairs(&before.lines["e"]),
            "{label}: the box's edges"
        );
    }
}
