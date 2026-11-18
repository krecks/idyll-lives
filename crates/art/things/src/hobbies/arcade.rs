//! H2: the arcade cabinet: the classic upright, its sides cut to the profile (a control panel jutting out, the screen
//! sloping back, the marquee over it), side art, a glowing screen and marquee at night. The joystick is its own part
//! and waggles while a resident plays (`joystick`, a loop).

use super::M;
use super::common::{P, rr};
use super::kit::Run;
use super::kit::call::{moulding, tube};
use artkit::json::float_repr as r;
use artkit::math::{Axis, P3, cos, radians, sin, tan};
use artkit::motion::{Rock, rock};
use artkit::parts::{hinged, join, new_parts, settle};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, revolve, soft};

const SIDE: [(f64, f64); 11] = [
    (-0.47, 0.0),
    (0.19, 0.0),
    (0.19, 0.84),
    (0.4, 0.865),
    (0.4, 0.955),
    (0.16, 1.0),
    (0.02, 1.5),
    (0.12, 1.53),
    (0.12, 1.78),
    (0.09, 1.82),
    (-0.47, 1.82),
];
const ART: [(f64, f64); 8] = [
    (-0.42, 0.08),
    (0.13, 0.08),
    (0.13, 0.8),
    (0.0, 1.0),
    (-0.1, 1.46),
    (0.06, 1.55),
    (0.06, 1.74),
    (-0.42, 1.74),
];
const STRIPE: [(f64, f64); 4] = [(-0.42, 0.98), (0.0, 1.0), (-0.07, 1.3), (-0.42, 1.22)];
const PANEL_TILT: f64 = 10.0;

/// The control panel's top at depth `z`.
fn panel_y(z: f64) -> f64 {
    0.935 - (z - 0.25) * tan(radians(PANEL_TILT))
}

/// Game point on the control panel's top at (x, depth), `out` above it.
fn on_panel(x: f64, z: f64, out: f64) -> P {
    let t = radians(PANEL_TILT);
    (x, panel_y(z) + out * cos(t), z + out * sin(t))
}

pub fn arcade_cabinet() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        for s in [-1, 1] {
            let sf = s as f64;
            fextrude!(
                f,
                format!("side{s}"),
                &SIDE,
                0.04,
                (sf * 0.31, 0, 0),
                M.arcade,
                plane = Facing::Side,
                bevel = 0.003,
                bevel_segments = 1
            );
            fextrude!(
                f,
                format!("art{s}"),
                &ART,
                0.004,
                (sf * 0.332, 0, 0),
                M.art,
                plane = Facing::Side
            );
            fextrude!(
                f,
                format!("stripe{s}"),
                &STRIPE,
                0.002,
                (sf * 0.335, 0, 0),
                M.art_stripe,
                plane = Facing::Side
            );
            // the T-moulding along the side's edge, from the floor at the front round to the top at the back
            let mut edge: Vec<P> = vec![(sf * 0.31, 0.01, 0.19)];
            edge.extend(
                SIDE[2..SIDE.len() - 1]
                    .iter()
                    .map(|&(z, y)| (sf * 0.31, y, z)),
            );
            fsweep!(
                f,
                format!("tmould{s}"),
                &edge,
                0,
                M.art_stripe,
                profile = rr(0.044, 0.006, 0.002, 1),
                up = (1, 0, 0)
            );
            fcyl!(
                f,
                format!("leveller{s}"),
                0.02,
                0.012,
                (sf * 0.25, 0.006, 0.12),
                M.bezel,
                verts = 8
            );
            fcyl!(
                f,
                format!("leveller_b{s}"),
                0.02,
                0.012,
                (sf * 0.25, 0.006, -0.42),
                M.bezel,
                verts = 8
            );
        }
        fbox!(f, "back", 0.58, 1.8, 0.3, (0, 0.9, -0.32), M.arcade);
        fbox!(f, "lower", 0.58, 0.84, 0.44, (0, 0.42, -0.03), M.arcade);
        fbox!(
            f,
            "kick_plate",
            0.58,
            0.08,
            0.006,
            (0, 0.05, 0.193),
            M.bezel,
            bevel = 0.002,
            bevel_segments = 1
        );
        // the coin door: a framed steel door, two coin mechanisms with their slots and lit reject buttons, a lock
        fbox!(
            f,
            "coin_door",
            0.3,
            0.22,
            0.01,
            (0, 0.55, 0.192),
            M.bezel,
            bevel = 0.004,
            bevel_segments = 1
        );
        moulding!(
            f,
            "coin_frame",
            &[
                (0.155, 0.665),
                (-0.155, 0.665),
                (-0.155, 0.435),
                (0.155, 0.435)
            ],
            &[(0.0, 0.0), (0.0, 0.006), (0.012, 0.006), (0.012, 0.0)],
            M.chrome,
            at = 0.197,
            plane = Run::Front
        );
        for x in [-0.06, 0.06] {
            fbox!(
                f,
                format!("coin_plate{}", r(x)),
                0.07,
                0.11,
                0.006,
                (x, 0.585, 0.199),
                M.chrome,
                bevel = 0.002,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("coin{}", r(x)),
                0.03,
                0.05,
                0.006,
                (x, 0.6, 0.202),
                M.ball_red,
                bevel = 0.002,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("coin_slot{}", r(x)),
                0.004,
                0.026,
                0.002,
                (x, 0.6, 0.2055),
                M.bezel
            );
            fbox!(
                f,
                format!("coin_return{}", r(x)),
                0.03,
                0.018,
                0.004,
                (x, 0.545, 0.2015),
                M.bezel,
                bevel = 0.0015,
                bevel_segments = 1
            );
        }
        fcyl!(
            f,
            "coin_lock",
            0.011,
            0.008,
            (0, 0.47, 0.2),
            M.chrome,
            rot = (90, 0, 0),
            verts = 10
        );
        // the control panel: a bevelled deck tilted to the player, art on it, convex buttons and the stick's washer
        fbox!(
            f,
            "panel",
            0.58,
            0.07,
            0.3,
            (0, 0.9, 0.25),
            M.arcade,
            rot = (PANEL_TILT, 0.0, 0.0),
            bevel = 0.012,
            bevel_segments = 2
        );
        fbox!(
            f,
            "panel_art",
            0.56,
            0.004,
            0.24,
            (0, 0.935, 0.26),
            M.art,
            rot = (PANEL_TILT, 0.0, 0.0)
        );
        fbox!(
            f,
            "panel_edge",
            0.6,
            0.012,
            0.012,
            (0, panel_y(0.4) - 0.006, 0.4),
            M.art_stripe,
            bevel = 0.004,
            bevel_segments = 1
        );
        let button = [
            (0.0, 0.0),
            (0.022, 0.0),
            (0.022, 0.004),
            (0.026, 0.005),
            (0.026, 0.008),
            (0.017, 0.01),
            (0.015, 0.016),
            (0.008, 0.018),
            (0.0, 0.018),
        ];
        for (k, (x, z, colour)) in [
            (0.05, 0.25, M.ball_red),
            (0.13, 0.29, M.ball_blue),
            (0.21, 0.25, M.ball_yellow),
            (0.29, 0.29, M.ball_green),
            (0.06, 0.33, M.ball_white),
            (0.14, 0.37, M.ball_white),
        ]
        .into_iter()
        .enumerate()
        {
            let at = on_panel(x - 0.03, z, 0.0);
            revolve!(
                format!("button{k}"),
                &button,
                at = f.at(at.0, at.1, at.2),
                material = colour,
                segments = 12,
                rot = (PANEL_TILT, 0.0, 0.0)
            );
        }
        for (k, x) in [-0.04, 0.04].into_iter().enumerate() {
            let at = on_panel(x, 0.15, 0.0);
            revolve!(
                format!("start{k}"),
                &[
                    (0.0, 0.0),
                    (0.012, 0.0),
                    (0.012, 0.008),
                    (0.008, 0.01),
                    (0.0, 0.01)
                ],
                at = f.at(at.0, at.1, at.2),
                material = M.marquee,
                segments = 10,
                rot = (PANEL_TILT, 0.0, 0.0)
            );
        }
        fcyl!(
            f,
            "stick_washer",
            0.026,
            0.004,
            on_panel(-0.15, 0.28, 0.002),
            M.bezel,
            verts = 14,
            rot = (PANEL_TILT, 0.0, 0.0)
        );
        // the screen: a curved tube behind a bezel, set back under the marquee
        fbox!(
            f,
            "screen_shelf",
            0.58,
            0.06,
            0.24,
            (0, 0.97, 0.1),
            M.arcade
        );
        fbox!(
            f,
            "bezel",
            0.58,
            0.54,
            0.03,
            (0, 1.25, 0.06),
            M.bezel,
            rot = (-15.6, 0, 0),
            bevel = 0.008
        );
        soft!(
            f,
            "screen",
            0.48,
            0.38,
            0.012,
            (0, 1.252, 0.077),
            M.screen,
            rot = (-15.6, 0, 0),
            round = 0.012,
            puff = 0.006,
            flat = 3,
            round_segments = 1
        );
        // the marquee: a lit box in chrome trim, its speaker grille under it
        fbox!(
            f,
            "marquee",
            0.58,
            0.2,
            0.04,
            (0, 1.655, 0.1),
            M.marquee,
            bevel = 0.006
        );
        for y in [1.555, 1.755] {
            fbox!(
                f,
                format!("marquee_trim{}", r(y)),
                0.58,
                0.012,
                0.012,
                (0, y, 0.122),
                M.chrome,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "marquee_trim",
            0.58,
            0.04,
            0.05,
            (0, 1.535, 0.095),
            M.bezel
        );
        fbox!(
            f,
            "speaker",
            0.4,
            0.05,
            0.006,
            (0, 1.535, 0.121),
            M.arcade,
            bevel = 0.002,
            bevel_segments = 1
        );
        for k in 0..7 {
            fbox!(
                f,
                format!("speaker_slot{k}"),
                0.003,
                0.034,
                0.002,
                (-0.15 + 0.05 * k as f64, 1.535, 0.1245),
                M.bezel
            );
        }
        fbox!(
            f,
            "top",
            0.66,
            0.04,
            0.6,
            (0, 1.8, -0.17),
            M.arcade,
            bevel = 0.006
        );
    });
    join(&still, "body", Some(P3::ZERO));
    let base = on_panel(-0.15, 0.28, 0.0);
    let j = new_parts(|| {
        tube!(
            f,
            "stick",
            &[base, (base.0, base.1 + 0.075, base.2)],
            0.006,
            M.chrome,
            verts = 8,
            round = 1
        );
        fcyl!(
            f,
            "stick_boot",
            0.012,
            0.012,
            (base.0, base.1 + 0.008, base.2),
            M.bezel,
            verts = 10
        );
        fsphere!(
            f,
            "stick_ball",
            0.022,
            (base.0, base.1 + 0.09, base.2),
            M.ball_red,
            segments = 14,
            rings = 9
        );
    });
    let stick = hinged(join(&j, "joystick", None), f.at(base.0, base.1, base.2));
    // on its Model's `centre`: its back moved onto the wall face
    settle(&meshes());
    rock(
        stick,
        "joystick",
        16.0,
        Rock {
            seconds: 0.8,
            axis: Axis::X,
        },
    );
    meshes()
}
