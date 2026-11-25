//! The treadmill: belt along the depth, the console at the back; the runner faces the back.

use super::M;
use super::common::on;
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::P3;
use artkit::motion::scroll;
use artkit::parts::{join, new_parts};
use artkit::place::{GameFrame, N};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, fsweep, loft, revolve, ring, settle, soft};

/// The slats' pitch: the belt's loop moves them one on.
const SLAT: f64 = 0.3;
/// One slat on in 0.13 s (the clip, 4 frames at 30 a second): 2.25 m/s, a run.
const BELT_SECONDS: f64 = 0.13;
/// The motor hood's front edge (the belt comes out from under it).
const HOOD: f64 = -0.63;
/// The rear roller cap's front edge (the belt goes under it).
const CAP: f64 = 0.80;
/// The console's middle, tilted back to face the runner.
const CON: (N, f64, f64) = (N::I(0), 1.3, -0.58);
const TILT: i32 = -35;

/// A point on the console's face (`_on(CON, TILT, d)`).
fn con(x: impl Into<N>, y: f64, z: f64) -> (N, f64, f64) {
    on(CON, TILT, (x.into(), y, z))
}

pub fn treadmill() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        // the deck: a rounded base, the belt between two side rails capped by ribbed foot treads
        soft!(
            f,
            "deck",
            0.66,
            0.1,
            1.56,
            (0, 0.09, 0.12),
            M.body,
            round = 0.025,
            round_segments = 2
        );
        fbox!(
            f,
            "belt",
            0.56,
            0.012,
            1.5,
            (0, 0.146, 0.1),
            M.belt,
            bevel = 0.003
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                f,
                format!("rail{s}"),
                &[(sf * 0.33, 0.165, -0.65), (sf * 0.33, 0.165, 0.85)],
                0,
                M.body,
                profile = vec![
                    (-0.025, -0.035),
                    (0.02, -0.035),
                    (0.025, -0.03),
                    (0.025, 0.03),
                    (0.02, 0.035),
                    (-0.025, 0.035)
                ],
                up = (0, 0, 1)
            );
            fbox!(
                f,
                format!("tread{s}"),
                0.05,
                0.004,
                1.4,
                (sf * 0.33, 0.1915, 0.1),
                M.pads,
                bevel = 0.0015,
                bevel_segments = 1
            );
            for k in 0..8 {
                fbox!(
                    f,
                    format!("tread_rib{s}{k}"),
                    0.046,
                    0.002,
                    0.012,
                    (sf * 0.33, 0.1945, -0.5 + 0.18 * k as f64),
                    M.dark
                );
            }
        }
        // the rear roller cap, rounded, its roller's ends and adjusting bolts
        soft!(
            f,
            "cap",
            0.66,
            0.055,
            0.12,
            (0, 0.15, 0.86),
            M.body,
            round = 0.02,
            round_segments = 2
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fcyl!(
                f,
                format!("roller_end{s}"),
                0.028,
                0.01,
                (sf * 0.335, 0.13, 0.86),
                M.dark,
                rot = (0, 90, 0),
                verts = 12
            );
            fcyl!(
                f,
                format!("roller_bolt{s}"),
                0.006,
                0.012,
                (sf * 0.335, 0.13, 0.86),
                M.steel,
                rot = (0, 90, 0),
                verts = 6
            );
        }
        // the motor hood: lofted, rounding over to the front, a band of colour round it and vents on its top
        let h = f.at(0, 0, -0.78);
        let (hx, hz) = (h.x, h.y);
        let rings = [
            (0.35, 0.15, 0.05),
            (0.35, 0.15, 0.2),
            (0.345, 0.145, 0.235),
            (0.33, 0.13, 0.25),
        ];
        let hood: Vec<Vec<P3>> = rings
            .iter()
            .map(|&(rx, rz, y)| ring!(rx, rz, z = y, verts = 24, square = 6, at = (hx, hz)))
            .collect();
        loft!("hood", &hood, material = M.body);
        let band: Vec<Vec<P3>> = [0.19, 0.215]
            .iter()
            .map(|&y| ring!(0.352, 0.152, z = y, verts = 24, square = 6, at = (hx, hz)))
            .collect();
        loft!("band", &band, material = M.accent);
        for k in 0..5 {
            fbox!(
                f,
                format!("vent{k}"),
                0.3,
                0.003,
                0.012,
                (0, 0.25, -0.86 + 0.03 * k as f64),
                M.dark
            );
        }
        for x in [-0.28, 0.28] {
            for z in [-0.86, 0.84] {
                fcyl!(
                    f,
                    format!("foot{}{}", float_repr(x), float_repr(z)),
                    0.03,
                    0.04,
                    (x, 0.02, z),
                    M.dark,
                    verts = 10,
                    bevel = 0.006,
                    bevel_segments = 1
                );
            }
        }
        // transport wheels at the front of the hood
        for s in [-1, 1] {
            fcyl!(
                f,
                format!("wheel{s}"),
                0.035,
                0.025,
                (s as f64 * 0.31, 0.037, -0.89),
                M.dark,
                rot = (0, 90, 0),
                verts = 12,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        // the uprights: oval tubes from the hood to the console, the handlebars bent in one piece each, foam grips
        // with chrome pulse plates
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                f,
                format!("upright{s}"),
                &[(sf * 0.3, 0.2, -0.8), (sf * 0.3, 1.22, -0.62)],
                0,
                M.steel,
                profile = rounded_rect(0.07, 0.045, 0.02, 2),
                up = (0, 1, 0)
            );
            let bar = kit::fillet(
                &[
                    (sf * 0.3, 1.18, -0.63),
                    (sf * 0.33, 1.16, -0.6),
                    (sf * 0.33, 1.08, -0.38),
                    (sf * 0.33, 1.06, -0.28),
                ],
                0.04,
                3,
            );
            kit::tube!(
                f,
                format!("handlebar{s}"),
                &bar,
                0.018,
                M.steel,
                verts = 10,
                round = 1
            );
            kit::tube!(
                f,
                format!("grip{s}"),
                &[(sf * 0.33, 1.083, -0.39), (sf * 0.33, 1.062, -0.29)],
                0.024,
                M.pads,
                verts = 10,
                round = 1
            );
            fsweep!(
                f,
                format!("pulse{s}"),
                &[(sf * 0.33, 1.109, -0.46), (sf * 0.33, 1.087, -0.4)],
                0.023,
                M.chrome,
                verts = 10
            );
            fsphere!(
                f,
                format!("bar_cap{s}"),
                0.024,
                (sf * 0.33, 1.06, -0.285),
                M.pads,
                segments = 10,
                rings = 5
            );
        }
        // the console: rounded, a screen in a bezel, a light strip, buttons, speakers and the safety key
        soft!(
            f,
            "console",
            0.72,
            0.3,
            0.1,
            CON,
            M.body,
            rot = (TILT, 0, 0),
            round = 0.035,
            round_segments = 2
        );
        fbox!(
            f,
            "bezel",
            0.38,
            0.2,
            0.006,
            con(0, 0.03, 0.05),
            M.dark,
            rot = (TILT, 0, 0),
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "screen",
            0.34,
            0.16,
            0.006,
            con(0, 0.03, 0.052),
            M.screen,
            rot = (TILT, 0, 0),
            bevel = 0.003
        );
        fbox!(
            f,
            "strip",
            0.6,
            0.02,
            0.006,
            con(0, -0.075, 0.051),
            M.accent,
            rot = (TILT, 0, 0)
        );
        for (k, (x, m)) in [
            (-0.25, M.red),
            (-0.19, M.block),
            (0.19, M.block),
            (0.25, M.accent),
        ]
        .into_iter()
        .enumerate()
        {
            fbox!(
                f,
                format!("button{k}"),
                0.04,
                0.03,
                0.01,
                con(x, -0.11, 0.052),
                m,
                rot = (TILT, 0, 0),
                bevel = 0.004
            );
        }
        for s in [-1, 1] {
            for k in 0..4 {
                fbox!(
                    f,
                    format!("speaker{s}{k}"),
                    0.07,
                    0.006,
                    0.004,
                    con(s as f64 * 0.28, 0.07 - 0.02 * k as f64, 0.051),
                    M.dark,
                    rot = (TILT, 0, 0)
                );
            }
        }
        fcyl!(
            f,
            "safety_key",
            0.016,
            0.012,
            con(0.12, -0.11, 0.056),
            M.red,
            rot = (90 + TILT, 0, 0),
            verts = 10
        );
        kit::tube!(
            f,
            "lanyard",
            &[
                con(0.12, -0.12, 0.06),
                con(0.13, -0.2, 0.08),
                con(0.12, -0.27, 0.07)
            ],
            0.003,
            M.red,
            verts = 4,
            round = 3
        );
        // a bottle in its holder
        revolve!(
            "holder",
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.042, 0.03),
                (0.038, 0.03),
                (0.036, 0.006),
                (0.0, 0.006)
            ],
            at = f.at(0.39, 0.885, -0.68),
            material = M.steel,
            segments = 14
        );
        revolve!(
            "bottle",
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.035, 0.01),
                (0.035, 0.13),
                (0.03, 0.155),
                (0.022, 0.17),
                (0.022, 0.18),
                (0.0, 0.18)
            ],
            at = f.at(0.39, 0.89, -0.68),
            material = M.bottle,
            segments = 14
        );
        fcyl!(
            f,
            "bottle_cap",
            0.022,
            0.03,
            (0.39, 1.085, -0.68),
            M.dark,
            verts = 12,
            bevel = 0.004,
            bevel_segments = 1
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The slats: the first under the hood, the last one slat short of the cap's far edge, so one slat on (the loop)
    // the first comes out from under the hood where the second was and the last goes under the cap.
    let parts = new_parts(|| {
        let last = CAP + 0.105 - SLAT;
        let mut k = 0;
        while last - k as f64 * SLAT > HOOD - SLAT {
            fbox!(
                f,
                format!("slat{k}"),
                0.54,
                0.004,
                0.03,
                (0, 0.154, last - k as f64 * SLAT),
                M.slat
            );
            k += 1;
        }
    });
    let slats = join(&parts, "slats", None);
    settle!(centre = false);
    // Back towards the rear (game +z) at a running pace, 2.25 m/s (a slat each 0.13 s, `BELT_SECONDS`): a
    // long pitch keeps each frame's move under half of it, so the slats are seen going back, not jittering, at
    // 60 frames a second up to the fastest game speed (2.5x: 9 cm a frame of the 30).
    scroll(slats, "belt_run", (0, -SLAT, 0), Some(BELT_SECONDS));
    meshes()
}
