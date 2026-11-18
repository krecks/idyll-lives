//! ground.py's cut (bmesh.ops.bisect_plane with clear_outer, holes_fill, triangulate), stage by stage against
//! Blender 5.2 (tests/golden/api_cut.txt): one of the woods' rock's stones through its two fractures and the ground,
//! a lofted drum cut aslant and again, a slab with a hollow corner cut across both its arms.

mod common;

use artkit::bisect_plane;
use artkit::edit::{NgonMethod, QuadMethod, holes_fill, triangulate};
use artkit::materials::Mat;
use artkit::math::{M4, V3};
use artkit::mesh::Mesh;
use artkit::scene::{self, Obj};
use common::{GoldenMesh, as_set, f32s, golden_meshes, pair_points, renamed};

const STONE: Mat = Mat::new("stone", "#8C8C96", "matte");

/// How far a point the cut makes may be from Blender's, metres: a crossing is worked out from one end of its
/// edge, the lower-numbered here, the one Blender's mesh has first there (which a mesh made by Blender's own
/// primitives does not tell), and the two ends give sums a unit or two in the last place apart.
const CROSSING: f32 = 2e-7;

/// The part against a golden mesh as sets: its points paired with Blender's (within `reach`), its faces
/// Blender's (each wound as Blender's), and their shading. How many points are not Blender's to the bit.
fn same(what: &str, obj: Obj, g: &GoldenMesh, reach: f32) -> usize {
    let mesh = obj.read(|o| o.mesh.clone());
    assert_eq!(mesh.verts.len(), g.verts.len(), "{what}: how many points");
    let (map, inexact, far) =
        pair_points(&mesh.verts, &g.verts, reach).unwrap_or_else(|| panic!("{what}: points apart"));
    let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect();
    let got = as_set(&renamed(&faces, &map));
    let want = as_set(&g.faces);
    let missing = want.iter().filter(|f| !got.contains(f)).count();
    let extra = got.iter().filter(|f| !want.contains(f)).count();
    println!(
        "{what}: {} points ({inexact} not to the bit, the farthest {far:e} m), {} faces ({missing} of Blender's missing, {extra} not Blender's)",
        mesh.verts.len(),
        mesh.faces.len()
    );
    assert_eq!((missing, extra), (0, 0), "{what}: faces");
    // which faces are smooth, by their corners
    let key = |f: &[u32]| as_set(&[f.to_vec()]).remove(0);
    let smooth: std::collections::BTreeMap<Vec<u32>, char> = g
        .faces
        .iter()
        .zip(g.lines["smooth"].chars())
        .map(|(f, s)| (key(f), s))
        .collect();
    for (f, face) in mesh.faces.iter().enumerate() {
        let k = key(&renamed(&[mesh.corners(f).to_vec()], &map)[0]);
        assert_eq!(face.smooth, smooth[&k] == '1', "{what}: face {f} smooth");
    }
    inexact
}

fn v3s(line: &str) -> Vec<f32> {
    f32s(&line.split(' ').collect::<Vec<_>>())
}

#[test]
fn a_part_is_cut_filled_and_triangulated_as_blenders_bmesh_does_it() {
    let all = golden_meshes("api_cut.txt");
    let mut cuts: Vec<&str> = Vec::new();
    for m in &all {
        if !cuts.contains(&m.what.as_str()) {
            cuts.push(&m.what);
        }
    }
    assert_eq!(cuts.len(), 6);
    let mut stale = 0;
    for what in cuts {
        let of = |name: &str| {
            all.iter()
                .find(|m| m.what == what && m.name == name)
                .unwrap()
        };
        let start = of("in");
        let (co, no, mw) = (
            v3s(&start.lines["co"]),
            v3s(&start.lines["no"]),
            v3s(&start.lines["mw"]),
        );
        let flat = !what.contains("drum cut 1");
        scene::reset();
        // the part as Blender had it going in: its points and faces, smooth as they were
        let mut mesh = Mesh::from_pydata(
            start
                .verts
                .iter()
                .map(|p| V3::new(p[0], p[1], p[2]))
                .collect(),
            &start.faces,
        );
        mesh.set_material(Some(STONE));
        for (face, s) in mesh.faces.iter_mut().zip(start.lines["smooth"].chars()) {
            face.smooth = s == '1';
        }
        let obj = scene::add("part", mesh);
        // the matrix the script read: Blender's as of its last update (the identity for a stone just placed)
        let mut world = M4::IDENTITY;
        for r in 0..4 {
            for c in 0..4 {
                world.0[c][r] = mw[r * 4 + c];
            }
        }
        stale += world.is_identity() as usize;
        obj.edit(|o| o.mesh.transform(&world));
        let cut = bisect_plane!(
            obj,
            (co[0], co[1], co[2]),
            (no[0], no[1], no[2]),
            clear_outer = true
        );
        let staged = all.iter().any(|m| m.what == what && m.name == "bisected");
        if staged {
            same(&format!("{what} bisected"), obj, of("bisected"), CROSSING);
            let wanted = of("bisected").lines["cut"].split(' ').count();
            assert_eq!(cut.len(), wanted, "{what}: the cut's edges");
        }
        let made = holes_fill(obj, &cut);
        obj.edit(|o| {
            for &f in &made {
                o.mesh.faces[f].smooth = !flat;
            }
        });
        if staged {
            same(&format!("{what} filled"), obj, of("filled"), CROSSING);
        }
        triangulate(obj, QuadMethod::Beauty, NgonMethod::Beauty);
        obj.edit(|o| o.mesh.transform(&world.inverted().unwrap()));
        same(&format!("{what} out"), obj, of("out"), CROSSING);
    }
    // (the stone's two fractures were cut before Blender had worked out where it stands)
    assert_eq!(stale, 5);
}
