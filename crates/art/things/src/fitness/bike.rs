//! The exercise bike: an upright bike, the flywheel at the front, the cranks under the saddle.

use super::M;
use super::common::{Num, on, py, rr};
use artkit::kit;
use artkit::math::{Axis, P3, TAU, cos, cosd, pow, sin, sind};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, fsweep, loft, revolve, settle, soft, spin};

/// The crank axle.
const CRANK: (i32, f64, f64) = (0, 0.3, 0.04);
/// The cranks' length.
const ARM: f64 = 0.16;

/// A point on the console's face (`_on((0, 1.05, 0.36), -30, d)`).
fn console(x: impl Into<Num>, y: f64, z: f64) -> (Num, f64, f64) {
    on((Num::Int(0), 1.05, 0.36), -30, (x.into(), y, z))
}

pub fn exercise_bike() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let frame = rr(0.06, 0.05, 0.016, 2);
    let still = new_parts(|| {
        // the base: a beam on two stabilisers with rubber end caps and levelling feet, transport wheels at the front
        fsweep!(
            f,
            "beam",
            &[(0, 0.05, -0.43), (0, 0.05, 0.43)],
            0,
            M.body,
            profile = rr(0.06, 0.12, 0.014, 2)
        );
        for z in [-0.38, 0.38] {
            fsweep!(
                f,
                format!("stabiliser{}", py(z)),
                &[(-0.22, 0.03, z), (0.22, 0.03, z)],
                0,
                M.body,
                profile = rr(0.05, 0.08, 0.016, 2)
            );
            for s in [-1, 1] {
                soft!(
                    f,
                    format!("cap{}{s}", py(z)),
                    0.05,
                    0.035,
                    0.085,
                    (s as f64 * 0.24, 0.018, z),
                    M.dark,
                    round = 0.012,
                    round_segments = 1
                );
            }
        }
        // the frame: one swept piece from the beam up through the bottom bracket to the seat post, the handlebar
        // post curving up from the front, the flywheel's shroud between
        kit::tube!(
            f,
            "down_tube",
            &[(0, 0.62, -0.11), CRANK],
            0.024,
            M.body,
            verts = 10,
            round = 1
        );
        fsweep!(
            f,
            "seat_tube",
            &[(0, 0.06, -0.14), (0, 0.6, -0.105)],
            0,
            M.body,
            profile = &frame,
            up = (1, 0, 0)
        );
        kit::tube!(
            f,
            "seat_post",
            &[(0, 0.58, -0.106), (0, 0.7, -0.1)],
            0.02,
            M.steel,
            verts = 10,
            round = 1
        );
        fcyl!(
            f,
            "seat_clamp",
            0.034,
            0.03,
            (0, 0.6, -0.105),
            M.dark,
            rot = (-3, 0, 0),
            verts = 10,
            bevel = 0.004,
            bevel_segments = 1
        );
        kit::tube!(
            f,
            "seat_knob_stem",
            &[(0, 0.585, -0.08), (0, 0.585, -0.04)],
            0.006,
            M.steel,
            verts = 6,
            round = 1
        );
        revolve!(
            "seat_knob",
            &[
                (0.0, 0.0),
                (0.016, 0.0),
                (0.02, 0.008),
                (0.018, 0.02),
                (0.0, 0.022)
            ],
            at = f.at(0, 0.585, -0.04),
            material = M.dark,
            segments = 10,
            rot = (90, 0, 0)
        );
        fsweep!(
            f,
            "handle_post",
            &[(0, 0.06, 0.22), (0, 0.5, 0.27), (0, 0.95, 0.32)],
            0,
            M.body,
            profile = &frame,
            up = (1, 0, 0),
            resolution = 2
        );
        fcyl!(
            f,
            "bracket",
            0.045,
            0.1,
            CRANK,
            M.body,
            rot = (0, 90, 0),
            verts = 16,
            bevel = 0.008,
            bevel_segments = 1
        );
        // the flywheel in its shroud: a dark disc with chrome spokes, covers either side
        fcyl!(
            f,
            "flywheel",
            0.2,
            0.05,
            (0, 0.3, 0.27),
            M.dark,
            rot = (0, 90, 0),
            verts = 32,
            bevel = 0.01
        );
        for s in [-1, 1] {
            let sf = s as f64;
            revolve!(
                format!("flywheel_cover{s}"),
                &[(0.0, 0.012), (0.1, 0.01), (0.13, 0.004), (0.135, 0.0)],
                at = f.at(sf * 0.026, 0.3, 0.27),
                material = M.body,
                segments = 28,
                rot = (0, -90 * s, 0)
            );
            fcyl!(
                f,
                format!("flywheel_hub{s}"),
                0.02,
                0.012,
                (sf * 0.04, 0.3, 0.27),
                M.steel,
                rot = (0, 90, 0),
                verts = 10
            );
        }
        kit::tube!(
            f,
            "resistance_cable",
            &[
                (Num::Int(0), 0.95, 0.31),
                (Num::Float(0.04), 0.8, 0.35),
                (Num::Float(0.03), 0.55, 0.36),
                (Num::Int(0), 0.5, 0.33)
            ],
            0.004,
            M.dark,
            verts = 4,
            round = 3
        );
        // the saddle: a lofted seat, wide at the back, narrowing to its nose, a rail under it
        let sections: Vec<Vec<P3>> = [
            (0.7, 0.1, -0.2, 0.03),
            (0.72, 0.19, -0.215, 0.05),
            (0.745, 0.2, -0.22, 0.06),
            (0.758, 0.17, -0.205, 0.05),
        ]
        .iter()
        .map(|&(y, w, back, nose)| {
            saddle_ring(back, nose, 16)
                .iter()
                .map(|&(u, v)| f.at(u * w / 2.0, y, v))
                .collect()
        })
        .collect();
        loft!("saddle", &sections, material = M.pads);
        kit::tube!(
            f,
            "saddle_rail",
            &[(0, 0.705, -0.2), (0, 0.705, -0.04)],
            0.008,
            M.steel,
            verts = 6,
            round = 1
        );
        // the handlebars bent in one piece, foam grips, a console with its screen and buttons
        let bar = [
            (-0.22, 1.0, 0.16),
            (-0.22, 0.98, 0.3),
            (0.22, 0.98, 0.3),
            (0.22, 1.0, 0.16),
        ];
        kit::tube!(
            f,
            "bar",
            &kit::fillet(&bar, 0.05, 3),
            0.017,
            M.steel,
            verts = 10,
            round = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            kit::tube!(
                f,
                format!("grip{s}"),
                &[(sf * 0.22, 0.992, 0.24), (sf * 0.22, 1.0, 0.15)],
                0.022,
                M.pads,
                verts = 10,
                round = 1
            );
            fsphere!(
                f,
                format!("grip_end{s}"),
                0.022,
                (sf * 0.22, 1.0, 0.15),
                M.pads,
                segments = 10,
                rings = 5
            );
        }
        soft!(
            f,
            "console",
            0.2,
            0.12,
            0.04,
            (0, 1.05, 0.36),
            M.dark,
            rot = (-30, 0, 0),
            round = 0.012,
            round_segments = 1
        );
        fbox!(
            f,
            "screen",
            0.16,
            0.08,
            0.004,
            console(0, 0.01, 0.021),
            M.accent,
            rot = (-30, 0, 0)
        );
        for k in 0..3 {
            fbox!(
                f,
                format!("button{k}"),
                0.025,
                0.01,
                0.006,
                console(-0.04 + 0.04 * k as f64, -0.045, 0.021),
                [M.red, M.block, M.block][k],
                rot = (-30, 0, 0),
                bevel = 0.002,
                bevel_segments = 1
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The cranks and pedals, joined round the axle so they turn about it.
    let parts = new_parts(|| {
        for (s, a) in [(1, 0), (-1, 180)] {
            let sf = s as f64;
            let end = (
                sf * 0.075,
                CRANK.1 - ARM * cosd(a as f64),
                CRANK.2 + ARM * sind(a as f64),
            );
            fsweep!(
                f,
                format!("crank{s}"),
                &[(sf * 0.075, CRANK.1, CRANK.2), end],
                0,
                M.steel,
                profile = rr(0.014, 0.034, 0.006, 1),
                up = (1, 0, 0),
                scale = [1.0, 0.8]
            );
            fcyl!(
                f,
                format!("pedal{s}"),
                0.022,
                0.1,
                (sf * 0.13, end.1, end.2),
                M.dark,
                rot = (0, 90, 0),
                verts = 10
            );
            for e in [-1, 1] {
                fcyl!(
                    f,
                    format!("pedal_end{s}{e}"),
                    0.024,
                    0.008,
                    (sf * 0.13 + e as f64 * 0.046, end.1, end.2),
                    M.steel,
                    rot = (0, 90, 0),
                    verts = 10
                );
            }
        }
        fcyl!(
            f,
            "axle",
            0.018,
            0.16,
            CRANK,
            M.steel,
            rot = (0, 90, 0),
            verts = 10
        );
        fcyl!(
            f,
            "chainring",
            0.06,
            0.008,
            (0.06, CRANK.1, CRANK.2),
            M.steel,
            rot = (0, 90, 0),
            verts = 16
        );
    });
    let pedals = hinged(
        join(&parts, "pedals", None),
        f.at(CRANK.0, CRANK.1, CRANK.2),
    );
    // on its Model centre: the back on the wall face
    settle!();
    spin!(
        pedals,
        "pedals_turn",
        turns = 1,
        seconds = 1.2,
        axis = Axis::X
    );
    meshes()
}

/// `_saddle_ring`: a saddle's outline seen from above (game x as a fraction of half the width, depth): round at the
/// back, narrowing to a nose, `n` points.
fn saddle_ring(back: f64, nose: f64, n: usize) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for i in 0..n {
        let a = TAU * i as f64 / n as f64;
        let (u, v) = (cos(a), sin(a));
        let d = back + (nose - back) * (v + 1.0) / 2.0;
        out.push((u * (1.0 - 0.65 * pow(0.0f64.max(v), 1.5)), d));
    }
    out
}
