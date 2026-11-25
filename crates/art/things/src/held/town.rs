//! The food from town (review M-held-03), each in what it is sold in, and a parcel to carry in.

use super::M;
use super::common::{F, done, loft, rrect};
use super::dishes::_bag_ring;
use artkit::math::{TAU, atan2};
use artkit::scene::Obj;
use artkit::{fsweep, jitter, soft};

/// A paper cup of ice cream from the cart, 9 cm across at its top: pink-striped card, two scoops (strawberry on
/// vanilla), each with a scoop's ragged lip. (Its flat wooden spoon is the one in the hand: `paddle`, actions.ts
/// `EATEN_WITH`.)
pub fn ice_cream_cup() -> Vec<Obj> {
    let cup = lathe!(
        "cup",
        &[
            (0.0, 0.0),
            (0.031, 0.0),
            (0.0315, 0.003),
            (0.045, 0.058),
            (0.0465, 0.06),
            (0.0435, 0.06),
            (0.0, 0.05)
        ],
        [
            M.cup_paper,
            M.cup_paper,
            M.cup_stripe,
            M.cup_paper,
            M.cup_paper,
            M.vanilla
        ],
        segments = 18,
        wave = (9, vec![(3, 0.02)])
    );
    // Its wall in nine pink stripes, white between (each a segment of the eighteen).
    cup.edit(|o| {
        let mesh = &mut o.mesh;
        let paper = mesh
            .mats
            .iter()
            .position(|&m| m == M.cup_paper)
            .expect("the cup's paper") as u16;
        let normals = mesh.face_normals();
        for f in 0..mesh.faces.len() {
            let c = mesh.face_centre(f);
            let cz = c.z as f64;
            if mesh.faces[f].mat != paper && normals[f].z < 0.9 && 0.004 < cz && cz < 0.057 {
                let turn = artkit::py::fmod(atan2(c.y as f64, c.x as f64), TAU);
                if (turn / TAU * 18.0) as i64 % 2 != 0 {
                    mesh.faces[f].mat = paper;
                }
            }
        }
    });
    for (i, (x, y, z, m, s)) in [
        (-0.006, 0.05, 0.004, M.vanilla, 1.0),
        (0.008, 0.074, -0.004, M.strawberry, 0.85),
    ]
    .into_iter()
    .enumerate()
    {
        let sc = lathe!(
            format!("scoop{i}"),
            &[
                (0.0, 0.0),
                (0.03 * s, 0.0),
                (0.034 * s, 0.006 * s),
                (0.032 * s, 0.022 * s),
                (0.02 * s, 0.038 * s),
                (0.0, 0.043 * s)
            ],
            m,
            at = (x, y, z),
            segments = 10,
            wave = (5, vec![(2, 0.12)]),
            rot = (0.0, 0.0, (30 * i) as f64)
        );
        jitter!(sc, 0.0012, size = 0.01, seed = 2.3 + i as f64);
    }
    done("iceCreamCup")
}

/// A hot dog in its bun on a paper boat, 19 cm long: the bun split along its top, the sausage in it longer than
/// the bun, mustard piped along it in a zigzag and a line of ketchup.
pub fn hot_dog() -> Vec<Obj> {
    let ring = |y: f64, hw: f64, hd: f64| -> Vec<(f64, f64, f64)> {
        rrect(hw, hd, 12, 6.0)
            .into_iter()
            .map(|(x, z)| (x, y, z))
            .collect()
    };
    // (The boat 9 cm across its top, the bun 7 cm in it: the bun was 9.8, over the boat's sides.)
    loft(
        "boat",
        &[
            ring(0.0, 0.092, 0.036),
            ring(0.022, 0.1, 0.045),
            ring(0.022, 0.097, 0.042),
            ring(0.004, 0.089, 0.033),
        ],
        M.cup_paper,
        30.0,
    );
    _hot_dog(0.0);
    done("hotDog")
}

/// The hot dog itself along game x, `y` up from where it sits in its boat: the bun's two lathed halves side by
/// side, the bent sausage longer than it, a zigzag of mustard and a line of ketchup.
pub fn _hot_dog(y: f64) {
    for s in [-1, 1] {
        let half = lathe!(
            format!("bun{s}"),
            &[
                (0.0, -0.075),
                (0.012, -0.072),
                (0.018, -0.055),
                (0.019, 0.0),
                (0.018, 0.055),
                (0.012, 0.072),
                (0.0, 0.075)
            ],
            vec![M.bun; 6],
            at = (0, 0.0, 0),
            segments = 10,
            rot = (0, 90, 0),
            scale = (0.9, 1.0)
        );
        // (Side by side along it: turned 14 degrees apart as they were, their ends stood out over the boat's sides.)
        half.set_location(F.at(0.0, y + 0.022, s as f64 * 0.0125));
    }
    roll!(
        "sausage",
        0.0115,
        0.19,
        M.sausage,
        (0.0, y + 0.034, 0.0),
        segments = 10,
        bend = -0.004
    );
    let pts: Vec<(f64, f64, f64)> = (0..11)
        .map(|k| {
            (
                -0.075 + 0.015 * k as f64,
                y + 0.047,
                0.006 * if k % 2 != 0 { 1.0 } else { -1.0 },
            )
        })
        .collect();
    fsweep!(
        F,
        "mustard",
        &pts,
        0.0028,
        M.mustard,
        verts = 5,
        resolution = 2
    );
    fsweep!(
        F,
        "ketchup",
        &[
            (-0.07, y + 0.0455, 0.0045),
            (0.0, y + 0.046, 0.0055),
            (0.07, y + 0.0455, 0.0045)
        ],
        0.0022,
        M.ketchup,
        verts = 5,
        resolution = 2
    );
}

/// A packet of crisps from the machine, 13 by 18 cm, lying on its back, its top torn open (+depth) and the
/// silvered inside showing, a few crisps spilt out in front: blue with a yellow band, crimped seals.
pub fn crisps() -> Vec<Obj> {
    soft!(
        F,
        "packet",
        0.13,
        0.026,
        0.16,
        (0, 0.013, -0.01),
        M.packet,
        round = 0.01,
        puff = 0.008,
        flat = 2,
        round_segments = 1
    );
    soft!(
        F,
        "band",
        0.132,
        0.03,
        0.04,
        (0, 0.013, -0.02),
        M.packet_band,
        round = 0.01,
        puff = 0.0085,
        flat = 2,
        round_segments = 1
    );
    for z in [-0.095, 0.075] {
        box_!(
            format!("seal{z}"),
            0.128,
            0.004,
            0.014,
            (0, 0.016, z),
            if z < 0.0 { M.packet } else { M.foil },
            bevel = 0.001
        );
    }
    lathe!(
        "mouth",
        &[(0.0, 0.0), (0.06, 0.0), (0.06, 0.006), (0.0, 0.006)],
        M.foil,
        at = (0, 0.024, 0.068),
        segments = 12,
        scale = (1.0, 0.22),
        rot = (-70, 0, 0)
    );
    for (i, (x, z, a)) in [
        (-0.03, 0.095, 10.0),
        (0.01, 0.1, 50.0),
        (0.035, 0.09, 80.0),
        (0.0, 0.08, 130.0),
        (-0.012, 0.106, 170.0),
    ]
    .into_iter()
    .enumerate()
    {
        coin!(
            format!("crisp{i}"),
            0.03,
            0.0015,
            (x, 0.004 + 0.012 * (i % 2) as f64, z),
            M.crisp,
            segments = 8,
            cup = 1.6,
            tilt = (a / 6.0, a / 8.0)
        );
    }
    done("crisps")
}

/// An apple from the stall, 7.5 cm across: red with a yellow blush, a dimple at the stalk and the eye, its
/// stalk and a leaf.
pub fn apple() -> Vec<Obj> {
    let a = lathe!(
        "apple",
        &[
            (0.0, 0.006),
            (0.014, 0.0),
            (0.03, 0.01),
            (0.037, 0.03),
            (0.0365, 0.045),
            (0.031, 0.06),
            (0.016, 0.068),
            (0.006, 0.063),
            (0.0, 0.058)
        ],
        [
            M.apple_blush,
            M.apple,
            M.apple,
            M.apple,
            M.apple,
            M.apple,
            M.apple_blush,
            M.apple_blush
        ],
        segments = 14,
        wave = (5, vec![(2, 0.025), (3, 0.02)])
    );
    jitter!(a, 0.0008, size = 0.02, seed = 1.7);
    fsweep!(
        F,
        "stalk",
        &[
            (0.0, 0.06, 0.0),
            (0.002, 0.072, 0.001),
            (0.005, 0.08, 0.003)
        ],
        0.0017,
        M.stalk,
        verts = 5
    );
    leaf!(
        "leaf",
        (0.012, 0.072, 0.004),
        0.034,
        0.016,
        30,
        M.leaf,
        tilt = -20.0
    );
    done("apple")
}

/// A pastry from the bakery in a white paper bag, 12 cm wide and 17 cm high, its top folded open and a croissant
/// poking out of it.
pub fn pastry_bag() -> Vec<Obj> {
    let rings = [
        _bag_ring(0.0, 0.06, 0.03, 0.016),
        _bag_ring(0.13, 0.061, 0.031, 0.016),
        _bag_ring(0.155, 0.064, 0.036, 0.008),
        _bag_ring(0.165, 0.06, 0.03, 0.004),
        _bag_ring(0.1675, 0.057, 0.027, 0.003), // the fold rolled over
    ];
    loft("bag", &rings, M.bag_white, 30.0);
    // the croissant: a crescent of lobes standing up out of the bag, its tips curling down to the front
    for (i, t) in [-1.0, -0.5, 0.0, 0.5, 1.0].into_iter().enumerate() {
        let r = 0.022 * (1.0 - 0.35 * f64::abs(t));
        lump!(
            format!("lobe{i}"),
            r,
            (0.032 * t, 0.12 + 0.03 * (1.0 - t * t), 0.008 * t * t),
            M.pastry_baked,
            scale = (0.8, 1.1, 0.95),
            seed = 4.0 + i as f64,
            noise = 0.12
        );
    }
    done("pastryBag")
}

/// A parcel from the shop, 42 cm wide, 30 cm high and 34 cm deep: a corrugated card box, the seam where its top
/// flaps meet running front to back, brown tape along it and down the front and the back, a white address label on
/// the top.
pub fn parcel() -> Vec<Obj> {
    let (w, h, d) = (0.42, 0.30, 0.34);
    box_!("box", w, h, d, (0.0, h / 2.0, 0.0), M.carton, bevel = 0.006);
    // where the flaps meet, under the tape
    box_!(
        "seam",
        0.004,
        0.001,
        d - 0.01,
        (0.0, h + 0.0002, 0.0),
        M.carton_seam
    );
    // the tape: over the top and a hand's width down the front and the back
    box_!(
        "tape",
        0.05,
        0.0016,
        d + 0.002,
        (0.0, h + 0.0008, 0.0),
        M.tape
    );
    for z in [-1, 1] {
        box_!(
            format!("tape{z}"),
            0.05,
            0.1,
            0.0016,
            (0.0, h - 0.05, z as f64 * (d / 2.0 + 0.0008)),
            M.tape
        );
    }
    // the label, on the top to one side of the tape
    box_!(
        "label",
        0.1,
        0.0012,
        0.07,
        (0.11, h + 0.0006, 0.06),
        M.paper,
        yaw = 4.0
    );
    for (i, (x, l)) in [(0.016, 0.07), (0.0, 0.06), (-0.016, 0.045)]
        .into_iter()
        .enumerate()
    {
        box_!(
            format!("line{i}"),
            l,
            0.0004,
            0.004,
            (0.105, h + 0.0014, 0.06 + x),
            M.paper_red,
            yaw = 4.0
        );
    }
    done("parcel")
}
