//! The packs' mesh from points and their bar's profile, now the kernel's (`object_of!`, `forms::rect`), against
//! pack_cheerful's and pack_couchPotato's `_obj` in Blender 5.2 (tests/golden/forms_object_of.txt, written by a
//! script outside the repo): each part's points, its faces as Blender turned them, their smooth flags and materials,
//! its sharp edges, and for a part with a bevel the mesh its modifier made.
mod common;

use artkit::bevel::apply;
use artkit::forms::rect;
use artkit::materials::Mat;
use artkit::math::{P3, TAU, cos, sin};
use artkit::mesh::edge;
use artkit::object_of;
use artkit::place::GameFrame;
use artkit::scene::{self, Obj};
use common::*;
use std::collections::BTreeSet;

const F: GameFrame = GameFrame::new(0.0);
const BOOTH: Mat = Mat::new("booth", "#F28D8D", "fabric");
const PIPING: Mat = Mat::new("piping", "#F6C744", "satin");

fn faces_of(line: &str) -> Vec<Vec<u32>> {
    line.split(' ')
        .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
        .collect()
}

fn points_of(line: &str) -> Vec<[f32; 3]> {
    let flat = f32s(&line.split(' ').collect::<Vec<_>>());
    flat.chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
}

/// `obj` against the golden's lines for `case`; with a bevel, how far its rounded points are from Blender's at most.
fn same(golden: &str, case: &str, obj: Obj) -> f32 {
    let lines: Vec<&str> = golden
        .lines()
        .skip_while(|l| *l != format!("case {case}"))
        .skip(1)
        .take(7)
        .collect();
    assert_eq!(lines.len(), 7, "{case}: in the golden");
    let mesh = obj.read(|o| o.mesh.clone());
    let want = points_of(&lines[0][2..]);
    assert_eq!(mesh.verts.len(), want.len(), "{case}: its points");
    assert_eq!(points_off(&mesh.verts, &want), (0, 0), "{case}: its points");
    // the faces in the order given, each turned as Blender turned it
    let got: Vec<Vec<u32>> = (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect();
    assert_eq!(
        as_set(&got),
        as_set(&faces_of(&lines[1][2..])),
        "{case}: its faces"
    );
    let key = |f: &[u32]| as_set(&[f.to_vec()]).remove(0);
    for (f, want) in faces_of(&lines[1][2..]).iter().enumerate() {
        assert_eq!(
            key(&got[f]),
            key(want),
            "{case}: face {f} is where it was given"
        );
    }
    let smooth: String = mesh
        .faces
        .iter()
        .map(|f| if f.smooth { '1' } else { '0' })
        .collect();
    assert_eq!(smooth, &lines[2][2..], "{case}: smooth");
    let mats: Vec<&str> = mesh
        .faces
        .iter()
        .map(|f| mesh.mats[f.mat as usize].name)
        .collect();
    assert_eq!(mats.join(" "), &lines[3][2..], "{case}: materials");
    let sharp: BTreeSet<(u32, u32)> = lines[4][2..]
        .split(' ')
        .filter(|e| !e.is_empty())
        .map(|e| {
            let (a, b) = e.split_once('-').unwrap();
            edge(a.parse().unwrap(), b.parse().unwrap())
        })
        .collect();
    assert_eq!(mesh.sharp, sharp, "{case}: sharp edges");
    if lines[5].len() <= 3 {
        assert!(obj.read(|o| o.bevel.is_none()), "{case}: no bevel");
        return 0.0;
    }
    let rounded = apply(obj).read(|o| o.mesh.clone());
    let want = points_of(&lines[5][3..]);
    let (map, _, far) = pair_points(&rounded.verts, &want, 1e-6)
        .unwrap_or_else(|| panic!("{case}: its bevel's points are not Blender's"));
    let got: Vec<Vec<u32>> = (0..rounded.faces.len())
        .map(|f| rounded.corners(f).to_vec())
        .collect();
    assert_eq!(
        as_set(&renamed(&got, &map)),
        as_set(&faces_of(&lines[6][3..])),
        "{case}: its bevel's faces"
    );
    far
}

/// pack_cheerful's `_slab`: an outline between two depths, as it gave it to `_obj`.
fn slab(name: &str, pts: &[(f64, f64)], z0: f64, z1: f64, bevel: f64) -> Obj {
    let n = pts.len() as u32;
    let mut verts: Vec<P3> = pts.iter().map(|&(x, y)| F.at(x, y, z1)).collect();
    verts.extend(pts.iter().map(|&(x, y)| F.at(x, y, z0)));
    let mut faces: Vec<Vec<u32>> = vec![(0..n).collect(), (n..2 * n).rev().collect()];
    faces.extend((0..n).map(|i| vec![i, (i + 1) % n, n + (i + 1) % n, n + i]));
    object_of!(name, &verts, &faces, &[BOOTH], bevel = bevel)
}

#[test]
fn a_part_of_points_and_faces_is_the_packs_own() {
    let golden = golden("forms_object_of.txt");
    scene::reset();
    let ell = [
        (-0.2, 0.1),
        (0.2, 0.1),
        (0.2, 0.25),
        (0.05, 0.25),
        (0.05, 0.5),
        (-0.2, 0.5),
    ];
    same(&golden, "slab", slab("slab", &ell, -0.03, 0.03, 0.0));
    let far = same(
        &golden,
        "slab bevel",
        slab("slab_bevel", &ell, -0.03, 0.03, 0.004),
    );
    assert!(far <= 1e-6, "{far}");
    // two of its faces given wound inwards, a material a face, smooth under 50 degrees
    let verts: Vec<P3> = [
        (-0.1, -0.05, 0.0),
        (0.1, -0.05, 0.0),
        (0.1, 0.05, 0.0),
        (-0.1, 0.05, 0.0),
        (-0.1, -0.05, 0.08),
        (0.1, -0.05, 0.02),
        (0.1, 0.05, 0.02),
        (-0.1, 0.05, 0.08),
    ]
    .map(P3::from)
    .to_vec();
    let faces: Vec<Vec<u32>> = vec![
        vec![0, 1, 2, 3],
        vec![4, 5, 6, 7],
        vec![0, 1, 5, 4],
        vec![1, 2, 6, 5],
        vec![3, 2, 6, 7],
        vec![0, 3, 7, 4],
    ];
    let by_face: [u16; 6] = [0, 1, 0, 1, 1, 0];
    let wedge = object_of!(
        "wedge",
        &verts,
        &faces,
        &[BOOTH, PIPING],
        face_mats = &by_face[..],
        sharp = 50
    );
    same(&golden, "wedge", wedge);
    // (pack_couchPotato's and pack_energetic's had neither a material a face nor a bevel)
    same(
        &golden,
        "wedge plain",
        object_of!("wedge_plain", &verts, &faces, &[BOOTH]),
    );
    let n = 12u32;
    let ring: Vec<(f64, f64)> = (0..n)
        .map(|k| {
            let a = TAU * k as f64 / n as f64;
            (0.06 * cos(a), 0.06 * sin(a))
        })
        .collect();
    let mut dv: Vec<P3> = ring.iter().map(|&(x, y)| P3 { x, y, z: 0.0 }).collect();
    dv.extend(ring.iter().map(|&(x, y)| P3 { x, y, z: 0.05 }));
    let mut df: Vec<Vec<u32>> = vec![(0..n).collect(), (n..2 * n).rev().collect()];
    df.extend((0..n).map(|i| vec![i, (i + 1) % n, n + (i + 1) % n, n + i]));
    let drum = object_of!(
        "drum",
        &dv,
        &df,
        &[BOOTH, PIPING],
        sharp = 35,
        bevel = 0.003
    );
    let far = same(&golden, "drum", drum);
    assert!(far <= 1e-6, "{far}");
}

#[test]
fn a_bars_profile_is_the_scripts_either_way_they_wrote_it() {
    // pack_natureLover wrote (u * w, v * h) over the corners of a unit square, pack_outgoing and pack_romantic
    // (-w / 2, -h / 2) and so on: the same numbers to the bit
    const SQUARE: [(f64, f64); 4] = [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)];
    for (w, h) in [
        (0.02, 0.02),
        (0.035, 0.012),
        (0.1, 0.3),
        (1e-3, 7e-3),
        (0.0, 0.05),
    ] {
        let scaled: Vec<(u64, u64)> = SQUARE
            .iter()
            .map(|&(u, v)| ((u * w).to_bits(), (v * h).to_bits()))
            .collect();
        let got: Vec<(u64, u64)> = rect(w, h)
            .iter()
            .map(|p| (p.0.to_bits(), p.1.to_bits()))
            .collect();
        assert_eq!(got, scaled, "{w} by {h}");
    }
    assert_eq!(rect(0.04, 0.02)[0], (-0.02, -0.01));
}
