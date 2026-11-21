//! Blender's WEIGHTED_NORMAL modifier as the kitchen's glossy shells use it (bevel applied, shaded smooth by
//! angle, then the modifier with keep_sharp), against Blender 5.2 (tests/golden/api_weighted.txt: the mesh going
//! in and the normal at every corner coming out): a bevelled cabinet and door, the fridge's lofted shell, a
//! bevelled knob, and a box without keep_sharp. Then parts the kernel builds and bevels itself, as gamekit did
//! before the modifier (the meshes it left, each named built): the normals the bevel stored are gone from them.

mod common;

use artkit::bevel::{self, EdgeOrder};
use artkit::materials::Mat;
use artkit::math::{P3, V3};
use artkit::mesh::{Mesh, edge};
use artkit::normals::corner_normals;
use artkit::place::GameFrame;
use artkit::scene::{self, Obj};
use artkit::{fbox, loft, ring, weighted_normal};
use common::{golden_meshes, pair_points};
use std::collections::{BTreeMap, BTreeSet};

/// How far a corner's normal may turn from Blender's, degrees: Blender keeps the modifier's normals as two angles
/// of sixteen bits in each run's own frame and reads them back from there.
const KEPT: f64 = 0.03;

/// How far apart two unit vectors point, degrees (from how far apart their ends are).
fn turned(g: V3, w: &[f32]) -> f64 {
    let apart = ((g.x - w[0]) as f64)
        .hypot((g.y - w[1]) as f64)
        .hypot((g.z - w[2]) as f64);
    (2.0 * (apart / 2.0).min(1.0).asin()).to_degrees()
}

#[test]
fn a_shells_corners_take_their_faces_normals_by_area_as_blenders_modifier_gives_them() {
    let all: Vec<_> = golden_meshes("api_weighted.txt")
        .into_iter()
        .filter(|g| !g.what.starts_with("built "))
        .collect();
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
            worst = worst.max(turned(*g, wn));
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

const BODY: Mat = Mat::new("body", "#F3F3F1", "gloss");

/// The part `name` as the capture script made it with gamekit, its bevel still to be given: the mini fridge's
/// liner and its door (whose bevel is cut back for want of room), the retro fridge's lofted shell in sixteen
/// points a ring with its rims rounded, and a box.
fn built(name: &str) -> Obj {
    let f = GameFrame::default();
    match name {
        "liner" => fbox!(
            f,
            "liner",
            0.47,
            0.66,
            0.51,
            (0, 0.46, -0.135),
            BODY,
            bevel = 0.015
        ),
        "door" => fbox!(
            f,
            "door",
            0.555,
            0.785,
            0.045,
            (0, 0.4525, 0.0925),
            BODY,
            bevel = 0.035,
            bevel_segments = 3
        ),
        "plain" => fbox!(
            f,
            "plain",
            0.3,
            0.2,
            0.1,
            (0, 0.1, 0),
            BODY,
            bevel = 0.02,
            bevel_segments = 3
        ),
        "shell" => {
            let rings = [
                (0.08, 0.35, 0.31),
                (0.1, 0.36, 0.32),
                (1.54, 0.36, 0.32),
                (1.585, 0.354, 0.314),
                (1.615, 0.34, 0.3),
                (1.633, 0.315, 0.275),
                (1.64, 0.27, 0.23),
            ];
            let sections: Vec<Vec<P3>> = rings
                .iter()
                .map(|&(y, rx, rz)| ring!(rx, rz, z = y, verts = 16, square = 7.0))
                .collect();
            let shell = loft!("shell", &sections, material = BODY);
            bevel::bevel_edges(shell, 0.01, 2, 40.0, &EdgeOrder::Corners)
        }
        _ => panic!("{name}: not a part the capture built"),
    }
}

#[test]
fn a_bevelled_parts_weighted_normals_replace_the_ones_its_bevel_stored() {
    let mut cases = 0;
    for g in golden_meshes("api_weighted.txt") {
        let w: Vec<&str> = g.what.split(' ').collect();
        if w[0] != "built" {
            continue;
        }
        let (name, keep_sharp) = (w[1], w[3] == "1");
        // as kitchen.py's _soft: the bevel applied, smooth by 40 degrees with its marks kept, the modifier
        scene::reset();
        let obj = built(name);
        bevel::apply(obj);
        obj.edit(|o| o.mesh.shade_smooth_by_angle(40.0));
        weighted_normal!(obj, keep_sharp = keep_sharp);
        let mesh = obj.read(|o| o.mesh.clone());
        let (map, inexact, far) =
            pair_points(&mesh.verts, &g.verts, 1e-6).unwrap_or_else(|| panic!("{name}: points"));
        let sharp: BTreeSet<(u32, u32)> = mesh
            .sharp
            .iter()
            .map(|&(a, b)| edge(map[a as usize], map[b as usize]))
            .collect();
        let want_sharp: BTreeSet<(u32, u32)> = g.lines["sharp"]
            .split(' ')
            .filter(|e| !e.is_empty())
            .map(|e| {
                let (a, b) = e.split_once('-').unwrap();
                edge(a.parse().unwrap(), b.parse().unwrap())
            })
            .collect();
        assert_eq!(sharp, want_sharp, "{name}: sharp edges");
        // Blender's faces by their points (it lists them in an order of its own): where each one's corners start
        let mut theirs: BTreeMap<Vec<u32>, (usize, usize)> = BTreeMap::new();
        let mut start = 0;
        for (f, c) in g.faces.iter().enumerate() {
            let mut key = c.clone();
            key.sort();
            theirs.insert(key, (f, start));
            start += c.len();
        }
        let want = g.f32s("n");
        let want_kept: Vec<[i16; 2]> = g.lines["c"]
            .split(' ')
            .map(|p| {
                let (a, b) = p.split_once(',').unwrap();
                [a.parse().unwrap(), b.parse().unwrap()]
            })
            .collect();
        // the normals the file is written from, and the two angles kept for each
        let got = corner_normals(&mesh);
        let kept = mesh.stored.as_ref().expect("stored angles");
        assert!(kept.iter().all(Option::is_some), "{name}: a corner without");
        let (mut worst, mut exact, mut same) = (0.0_f64, 0, 0);
        for f in 0..mesh.faces.len() {
            let c: Vec<u32> = mesh.corners(f).iter().map(|&v| map[v as usize]).collect();
            let mut key = c.clone();
            key.sort();
            let &(b, first) = theirs
                .get(&key)
                .unwrap_or_else(|| panic!("{name}: face {f} is not one of Blender's"));
            assert_eq!(
                mesh.faces[f].smooth,
                &g.lines["smooth"][b..b + 1] == "1",
                "{name}: face {f} smooth"
            );
            for (i, v) in c.iter().enumerate() {
                let corner = first + g.faces[b].iter().position(|x| x == v).unwrap();
                let (at, wn) = (
                    mesh.faces[f].start as usize + i,
                    &want[corner * 3..corner * 3 + 3],
                );
                worst = worst.max(turned(got[at], wn));
                exact += (got[at].array() == [wn[0], wn[1], wn[2]]) as usize;
                same += (kept[at] == Some(want_kept[corner])) as usize;
            }
        }
        println!(
            "{name}: {} points ({inexact} not to the bit, the farthest {far:e} m), {} corners, {exact} to the bit, the farthest {worst:.4} degrees off, {same} kept as Blender's two angles",
            mesh.verts.len(),
            got.len()
        );
        assert_eq!(got.len(), want_kept.len(), "{name}: corners");
        assert!(
            worst <= KEPT,
            "{name}: a corner's normal {worst} degrees off"
        );
        cases += 1;
    }
    assert_eq!(cases, 4);
}
