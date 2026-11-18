//! H1: the electric guitar on its stand, the practice amp behind.

use super::M;
use super::common::{frets, guitar_stand, lean, off_stand, strings, upright};
use super::guitars::{dots, neck};
use super::kit::call::{moulding, tube};
use super::kit::{fillet, knob};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr as r;
use artkit::math::P3;
use artkit::parts::{Settle, join, new_parts, settle_with};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, soft};

// The electric guitar's solid body: a double cutaway, the treble horn short (+x), the bass horn long (game x, height
// from its foot).
// (0.318 is a height on the body, not 1/pi)
#[allow(clippy::approx_constant)]
const ELECTRIC: [(f64, f64); 30] = [
    (0.0, 0.0),
    (0.09, 0.01),
    (0.155, 0.05),
    (0.18, 0.12),
    (0.17, 0.19),
    (0.132, 0.26),
    (0.13, 0.3),
    (0.14, 0.345),
    (0.13, 0.39),
    (0.11, 0.418),
    (0.09, 0.425),
    (0.075, 0.41),
    (0.065, 0.37),
    (0.05, 0.33),
    (0.035, 0.315),
    (0.026, 0.318),
    (-0.026, 0.318),
    (-0.036, 0.322),
    (-0.05, 0.35),
    (-0.062, 0.41),
    (-0.078, 0.455),
    (-0.095, 0.468),
    (-0.115, 0.455),
    (-0.128, 0.41),
    (-0.135, 0.34),
    (-0.14, 0.28),
    (-0.165, 0.2),
    (-0.183, 0.12),
    (-0.165, 0.05),
    (-0.09, 0.01),
];
const GUARD: [(f64, f64); 11] = [
    (0.135, 0.17),
    (0.12, 0.25),
    (0.085, 0.31),
    (0.045, 0.34),
    (-0.035, 0.342),
    (-0.075, 0.3),
    (-0.07, 0.2),
    (-0.095, 0.13),
    (-0.06, 0.06),
    (0.05, 0.05),
    (0.12, 0.09),
];

fn electric(fr: &GameFrame) -> Obj {
    let p = new_parts(|| {
        fextrude!(
            fr,
            "body",
            &smooth_outline(&ELECTRIC, 1, true),
            0.045,
            (0, 0, 0),
            M.electric,
            bevel = 0.009,
            bevel_segments = 2
        );
        fextrude!(
            fr,
            "guard",
            &smooth_outline(&GUARD, 2, true),
            0.003,
            (0, 0, 0.0235),
            M.pickguard,
            bevel = 0.001,
            bevel_segments = 1
        );
        for (k, y) in [0.13, 0.2, 0.27].into_iter().enumerate() {
            fbox!(
                fr,
                format!("pickup{k}"),
                0.07,
                0.018,
                0.006,
                (0, y, 0.027),
                M.black,
                bevel = 0.003,
                bevel_segments = 2
            );
        }
        // the bridge: a plate and six saddles, the tremolo arm
        fbox!(
            fr,
            "bridge",
            0.075,
            0.025,
            0.004,
            (0, 0.085, 0.0245),
            M.chrome,
            bevel = 0.0015,
            bevel_segments = 1
        );
        for j in 0..6 {
            fbox!(
                fr,
                format!("saddle{j}"),
                0.009,
                0.012,
                0.006,
                (-0.026 + 0.0105 * j as f64, 0.087, 0.029),
                M.chrome
            );
        }
        tube!(
            fr,
            "trem_arm",
            &fillet(
                &[
                    (0.034, 0.07, 0.027),
                    (0.034, 0.07, 0.045),
                    (0.06, 0.02, 0.05)
                ],
                0.01,
                3
            ),
            0.0018,
            M.chrome,
            verts = 6,
            round = 1
        );
        for (k, (x, y)) in [(0.09, 0.12), (0.1, 0.075), (0.085, 0.035)]
            .into_iter()
            .enumerate()
        {
            knob(fr, &format!("knob{k}"), x, y, 0.0235, 0.009, M.black, 8);
        }
        fbox!(
            fr,
            "switch_slot",
            0.006,
            0.03,
            0.001,
            (0.11, 0.19, 0.0252),
            M.black
        );
        tube!(
            fr,
            "switch",
            &[(0.11, 0.19, 0.025), (0.11, 0.19, 0.036)],
            0.0018,
            M.chrome,
            verts = 6,
            round = 1
        );
        fcyl!(
            fr,
            "switch_tip",
            0.004,
            0.008,
            (0.11, 0.19, 0.038),
            M.pickguard,
            rot = (90, 0, 0),
            verts = 8
        );
        fcyl!(
            fr,
            "jack",
            0.011,
            0.004,
            (0.12, 0.05, 0.0225),
            M.chrome,
            rot = (90, 0, 0),
            verts = 12
        );
        fcyl!(
            fr,
            "strap_button",
            0.005,
            0.01,
            (0, -0.004, 0),
            M.chrome,
            verts = 8
        );
        // a 0.648 m scale from the saddle to the nut (review M-hobbies-03: the neck was half as long again)
        let nut = 0.09 + 0.648;
        neck(fr, "neck", 0.3, nut, 0.0125, 0.05, 0.022, M.maple, 0.86, 8);
        fbox!(
            fr,
            "neck_plate",
            0.05,
            0.05,
            0.002,
            (0, 0.355, -0.0235),
            M.chrome,
            bevel = 0.001,
            bevel_segments = 1
        );
        fbox!(
            fr,
            "fretboard",
            0.048,
            nut - 0.355,
            0.005,
            (0, (nut + 0.355) / 2.0, 0.015),
            M.fretboard
        );
        frets(fr, 15, 0.648, nut, 0.048, 0.0185);
        dots(
            fr,
            "dot",
            &[3, 5, 7, 9, 12],
            0.648,
            nut,
            0.0176,
            M.bone,
            0.0035,
        );
        fbox!(
            fr,
            "nut",
            0.048,
            0.004,
            0.004,
            (0, nut + 0.001, 0.0185),
            M.bone
        );
        let head = [
            (0.022, nut - 0.002),
            (0.028, nut + 0.04),
            (0.032, nut + 0.12),
            (0.046, nut + 0.148),
            (0.034, nut + 0.166),
            (-0.008, nut + 0.162),
            (-0.04, nut + 0.142),
            (-0.042, nut + 0.02),
            (-0.03, nut + 0.006),
            (-0.022, nut - 0.002),
        ];
        fextrude!(
            fr,
            "head",
            &smooth_outline(&head, 1, true),
            0.016,
            (0, 0, -0.002),
            M.black,
            bevel = 0.003,
            bevel_segments = 1
        );
        for k in 0..6 {
            let y = nut + 0.023 + 0.022 * k as f64;
            fcyl!(
                fr,
                format!("tuner{k}"),
                0.006,
                0.012,
                (-0.026, y, 0.011),
                M.chrome,
                verts = 8,
                rot = (90, 0, 0)
            );
            fbox!(
                fr,
                format!("tuner_key{k}"),
                0.016,
                0.008,
                0.004,
                (-0.05, y, -0.006),
                M.chrome
            );
        }
        fbox!(
            fr,
            "string_tree",
            0.012,
            0.004,
            0.004,
            (-0.01, nut + 0.05, 0.008),
            M.chrome
        );
        strings(fr, 6, 0.009, 0.0075, 0.087, nut + 0.001, 0.033, 0.0215);
    });
    upright(&p, "guitar")
}

pub fn guitar_electric() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        guitar_stand(&f, 0.0, 0.11, 0.71, 0.005, 0.1, 0.12, 0.15);
        // the practice amp behind: a tolexed cabinet with piped edges, a cloth grille, a control panel, a strap handle
        fbox!(
            f,
            "amp",
            0.46,
            0.4,
            0.28,
            (0, 0.2, -0.28),
            M.amp,
            bevel = 0.02,
            bevel_segments = 2
        );
        fbox!(
            f,
            "amp_baffle",
            0.42,
            0.29,
            0.004,
            (0, 0.165, -0.139),
            M.black,
            bevel = 0.002,
            bevel_segments = 1
        );
        soft!(
            f,
            "amp_grille",
            0.4,
            0.27,
            0.006,
            (0, 0.165, -0.137),
            M.grille,
            round = 0.006,
            round_segments = 1
        );
        let piping: Vec<(f64, f64)> = rounded_rect(0.4, 0.27, 0.01, 1)
            .iter()
            .map(|&(x, y)| (x, y + 0.165))
            .collect();
        moulding!(
            f,
            "amp_piping",
            &piping,
            &[(0.0, 0.0), (0.0, 0.004), (0.005, 0.004), (0.005, 0.0)],
            M.amp_trim,
            at = -0.14
        );
        fbox!(
            f,
            "amp_panel",
            0.42,
            0.06,
            0.004,
            (0, 0.355, -0.139),
            M.amp_trim,
            bevel = 0.002,
            bevel_segments = 1
        );
        for k in 0..5 {
            fcyl!(
                f,
                format!("amp_knob{k}"),
                0.011,
                0.014,
                (-0.14 + 0.06 * k as f64, 0.355, -0.132),
                M.black,
                rot = (90, 0, 0),
                verts = 10,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fcyl!(
            f,
            "amp_jack",
            0.008,
            0.01,
            (0.18, 0.355, -0.134),
            M.chrome,
            rot = (90, 0, 0),
            verts = 8
        );
        fbox!(
            f,
            "amp_light",
            0.01,
            0.01,
            0.004,
            (0.155, 0.355, -0.136),
            M.ball_red,
            bevel = 0.002,
            bevel_segments = 1
        );
        tube!(
            f,
            "amp_handle",
            &fillet(
                &[
                    (-0.08, 0.4, -0.28),
                    (-0.075, 0.418, -0.28),
                    (0.075, 0.418, -0.28),
                    (0.08, 0.4, -0.28)
                ],
                0.008,
                3
            ),
            0.008,
            M.black,
            verts = 8,
            round = 1
        );
        for x in [-0.085, 0.085] {
            fbox!(
                f,
                format!("amp_handle_cap{}", r(x)),
                0.03,
                0.008,
                0.03,
                (x, 0.402, -0.28),
                M.chrome,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "amp_logo",
            0.08,
            0.018,
            0.004,
            (0.12, 0.27, -0.134),
            M.amp_trim,
            bevel = 0.002,
            bevel_segments = 1
        );
        for sx in [-1, 1] {
            for y in [0.01, 0.39] {
                fbox!(
                    f,
                    format!("amp_corner{sx}{}", r(y)),
                    0.03,
                    0.03,
                    0.03,
                    (sx as f64 * 0.215, y + 0.005, -0.145),
                    M.chrome,
                    bevel = 0.006,
                    bevel_segments = 1
                );
            }
        }
    });
    join(&still, "stand", Some(P3::ZERO));
    let guitar = lean(&f, electric(&GameFrame::new(0.0)), 12.0, (0.0, 0.12, 0.106));
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    off_stand(guitar);
    meshes()
}
