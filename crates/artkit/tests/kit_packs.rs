//! The helpers the packs' scripts each had a copy of, now the kit's (`kit::turned!`, `kit::lathe!`, `kit::paint`),
//! against those scripts' own in Blender 5.2 (tests/golden/kit_packs.txt, written by a script outside the repo that
//! calls pack_kind's, pack_lazy's, pack_gloomy's `_turned`, pack_lazy's, pack_hotHeaded's and pack_loner's `_lathe`
//! and pack_cheerful's and pack_foodie's `_paint` as the packs called them): each part's place, turn, points, faces
//! and the material of every face; and `parts::numbered`, their `_name`.
mod common;

use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, PI, atan2};
use artkit::parts::{numbered, numbered_from};
use artkit::place::GameFrame;
use artkit::py;
use artkit::scene::{self, Obj};
use artkit::{fbox, loft, revolve, ring};
use common::*;
use std::collections::BTreeMap;

const F: GameFrame = GameFrame::new(0.0);
const S: GameFrame = GameFrame::new(0.094);
const PORCELAIN: Mat = Mat::new("porcelain", "#F4F1EA", "gloss");
const CRUST: Mat = Mat::new("crust", "#D9A35F", "satin");

/// A face turned to start at its lowest point: what it is called by, whatever order the faces are in.
fn key(face: &[u32]) -> Vec<u32> {
    as_set(&[face.to_vec()]).remove(0)
}

/// `obj` against the golden's lines for `case`: its place, its turn, its points in order, its faces as a set, and
/// each face's material and middle.
fn same(golden: &str, case: &str, obj: Obj) {
    let lines: Vec<&str> = golden
        .lines()
        .skip_while(|l| *l != format!("case {case}"))
        .skip(1)
        .take(7)
        .collect();
    assert_eq!(lines.len(), 7, "{case}: in the golden");
    let words = |i: usize, tag: &str| -> Vec<&str> {
        assert!(lines[i].starts_with(tag), "{case}: {tag}");
        lines[i][tag.len()..].split(' ').collect()
    };
    let l = obj.location();
    assert_eq!(
        bits(&[l.x, l.y, l.z]),
        bits(&f32s(&words(0, "loc "))),
        "{case}: its place"
    );
    let r = obj.rotation_euler();
    assert_eq!(
        bits(&[r.x, r.y, r.z]),
        bits(&f32s(&words(1, "rot "))),
        "{case}: its turn"
    );
    let flat = f32s(&words(2, "v "));
    let want: Vec<[f32; 3]> = flat.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
    let mesh = obj.read(|o| o.mesh.clone());
    assert_eq!(mesh.verts.len(), want.len(), "{case}: its points");
    assert_eq!(points_off(&mesh.verts, &want), (0, 0), "{case}: its points");
    let faces: Vec<Vec<u32>> = words(3, "f ")
        .iter()
        .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
        .collect();
    let smooth: Vec<bool> = lines[4][2..].chars().map(|c| c == '1').collect();
    let mats = words(5, "m ");
    let centres = f32s(&words(6, "c "));
    let mut blender: BTreeMap<Vec<u32>, (bool, &str, Vec<u32>)> = BTreeMap::new();
    for (i, f) in faces.iter().enumerate() {
        blender.insert(
            key(f),
            (smooth[i], mats[i], bits(&centres[3 * i..3 * i + 3])),
        );
    }
    assert_eq!(mesh.faces.len(), faces.len(), "{case}: its faces");
    for f in 0..mesh.faces.len() {
        let corners = mesh.corners(f).to_vec();
        let (s, m, c) = blender
            .get(&key(&corners))
            .unwrap_or_else(|| panic!("{case}: Blender has no face {corners:?}"));
        let face = &mesh.faces[f];
        assert_eq!(face.smooth, *s, "{case}: face {corners:?} smooth");
        assert_eq!(
            mesh.mats[face.mat as usize].name, *m,
            "{case}: face {corners:?}"
        );
        let centre = mesh.face_centre(f);
        assert_eq!(
            &bits(&[centre.x, centre.y, centre.z]),
            c,
            "{case}: the middle of {corners:?}"
        );
    }
}

#[test]
fn a_turned_piece_stands_on_its_game_point() {
    let golden = golden("kit_packs.txt");
    scene::reset();
    let saucer = [
        (0.0, 0.0),
        (0.03, 0.0),
        (0.05, 0.006),
        (0.052, 0.009),
        (0.034, 0.005),
        (0.0, 0.005),
    ];
    let cup = [
        (0.0, 0.0),
        (0.022, 0.0),
        (0.038, 0.045),
        (0.034, 0.045),
        (0.02, 0.006),
        (0.0, 0.006),
    ];
    let biscuit = [(0.0, 0.0), (0.03, 0.004), (0.026, 0.012), (0.0, 0.012)];
    let t = kit::turned!(
        F,
        "saucer",
        0.31,
        0.74,
        -0.12,
        &saucer,
        PORCELAIN,
        segments = 16
    );
    same(&golden, "turned saucer", t);
    // sixteen round where the script gave none; a whole zero is a plain one in the part's place
    let t = kit::turned!(S, "cup", 0, 0.74 + 0.006, 0, &cup, PORCELAIN);
    same(&golden, "turned cup", t);
    assert!(t.location().x.is_sign_positive());
    let t = kit::turned!(S, "cup0", 0.0, 0, 0.0, &cup, PORCELAIN, segments = 12);
    same(&golden, "turned cup0", t);
    assert!(t.location().x.is_sign_negative());
    let t = kit::turned!(
        F,
        "side",
        -0.2,
        0.5,
        0.1,
        &biscuit,
        PORCELAIN,
        segments = 10,
        rot = (90, 0, 25)
    );
    same(&golden, "turned side", t);
    // (pack_gloomy's had no turn to give)
    let t = kit::turned!(
        F,
        "gloomy",
        0.4,
        0.2,
        -0.3,
        &biscuit,
        PORCELAIN,
        segments = 14
    );
    same(&golden, "turned gloomy", t);
}

#[test]
fn a_lathed_body_is_lofted_through_its_rings() {
    let golden = golden("kit_packs.txt");
    scene::reset();
    let seat = [
        (0.27, 0.0),
        (0.29, 0.015),
        (0.3, 0.05),
        (0.295, 0.08),
        (0.27, 0.096),
        (0.18, 0.1),
    ];
    let l = kit::lathe!(F, "seat", 0, 0.35, 0.1, &seat, PORCELAIN, verts = 24);
    same(&golden, "lathe seat", l);
    let bottle = [
        (0.045, 0.0),
        (0.048, 0.006),
        (0.048, 0.17),
        (0.04, 0.195),
        (0.026, 0.21),
    ];
    let l = kit::lathe!(
        S,
        "bottle",
        -0.22,
        0.6,
        -0.79,
        &bottle,
        PORCELAIN,
        verts = 12
    );
    same(&golden, "lathe bottle", l);
    let throne = [
        (0.15, 0.0),
        (0.175, 0.012),
        (0.182, 0.035),
        (0.178, 0.055),
        (0.16, 0.066),
    ];
    let l = kit::lathe!(F, "throne", 0, 0.4, 0.0, &throne, PORCELAIN);
    same(&golden, "lathe throne", l);
    // squarer, and squashed in depth (pack_loner's and pack_lazy's `sy`)
    let bowl = [(0.05, 0.0), (0.09, 0.05), (0.085, 0.06), (0.04, 0.012)];
    let l = kit::lathe!(
        F,
        "bowl",
        0.12,
        0.42,
        0,
        &bowl,
        PORCELAIN,
        verts = 24,
        square = 7,
        sy = 0.68
    );
    same(&golden, "lathe bowl", l);
}

#[test]
fn faces_are_painted_by_their_middles() {
    let golden = golden("kit_packs.txt");
    scene::reset();
    // a speaker's trim, by radius, on a revolve turned to face the front
    let r = 0.09;
    let prof = [
        (0.0, 0.0),
        (r - 0.004, 0.0),
        (r, 0.004),
        (r, 0.014),
        (r - 0.017, 0.011),
        (r - 0.022, 0.003),
        (r * 0.36, -0.04),
        (0.0, -0.022),
    ];
    let d = revolve!(
        "driver",
        &prof,
        at = F.at(0, 0.6, 0.2),
        material = PORCELAIN,
        segments = 16,
        rot = (90, 0, 0)
    );
    let d = kit::paint(d, CRUST, |c| py::hypot(c.x as f64, c.y as f64) > r - 0.024);
    same(&golden, "paint driver", d);
    // a tumbler's band, by height, on a loft: its caps are faces of ten corners
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = F.at(0.18, 0.7375, 0.4);
    let (h, rr) = (0.11, 0.03);
    let rows = [
        (0.0, 0.92),
        (0.02, 1.0),
        (0.62, 1.0),
        (0.76, 0.6),
        (0.84, 0.38),
        (0.95, 0.38),
    ];
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(t, k)| ring!(rr * k, rr * k, z = bz + h * t, verts = 10, at = (bx, by)))
        .collect();
    let b = loft!("tumbler", &sections, material = PORCELAIN);
    let b = kit::paint(b, CRUST, |c| {
        0.2 * h < c.z as f64 - bz && c.z as f64 - bz < 0.5 * h
    });
    same(&golden, "paint tumbler", b);
    // a bowl's gores, by angle
    let bowl = revolve!(
        "popcorn_bowl",
        &[
            (0.0, 0.0),
            (0.05, 0.0),
            (0.09, 0.05),
            (0.085, 0.052),
            (0.045, 0.006),
            (0.0, 0.006)
        ],
        at = F.at(0.14, 0.7375, 0.66),
        material = PORCELAIN,
        segments = 16
    );
    let bowl = kit::paint(bowl, CRUST, |c| {
        py::fmod(atan2(c.y as f64, c.x as f64), PI / 4.0) < PI / 8.0
    });
    same(&golden, "paint bowl", bowl);
    let boiler = revolve!(
        "boiler",
        &[
            (0.0, 0.0),
            (0.12, 0.0),
            (0.122, 0.02),
            (0.12, 0.06),
            (0.1, 0.15),
            (0.06, 0.21),
            (0.02, 0.23),
            (0.0, 0.23)
        ],
        at = F.at(-0.06, 1.23, -0.27),
        material = PORCELAIN,
        segments = 20
    );
    let boiler = kit::paint(boiler, CRUST, |c| (c.z as f64) < 0.025);
    same(&golden, "paint boiler", boiler);
    // a middle on the very edge of its test: a single read as a double, as Python reads `c.z`
    let b = fbox!(F, "box", 0.2, 0.1, 0.3, (0, 0.05, 0), PORCELAIN);
    let b = kit::paint(b, CRUST, |c| c.z as f64 >= 0.05 || c.x as f64 <= -0.1);
    same(&golden, "paint box", b);
    // a material the part has already is not added again
    let again = kit::paint(b, PORCELAIN, |_| true);
    assert_eq!(again.read(|o| o.mesh.mats.len()), 2);
    assert!(again.read(|o| o.mesh.faces.iter().all(|f| f.mat == 0)));
}

#[test]
fn parts_are_numbered_in_one_count() {
    assert_eq!(
        [numbered("pot"), numbered("leaf"), numbered("pot")],
        ["pot1", "leaf2", "pot3"]
    );
    // the scene is not what counts
    scene::reset();
    assert_eq!(numbered("pot"), "pot4");
    // a model whose count ran on from the models before it in the script
    numbered_from(23);
    assert_eq!(numbered("door"), "door24");
    // each thread its own count: a model is built on one
    let other = std::thread::spawn(|| numbered("pot")).join().unwrap();
    assert_eq!(other, "pot1");
}
