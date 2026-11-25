//! H3: the sewing table: a table to the front with the machine, cloth under its needle, thread and a basket; a chair
//! behind. The presser foot and needle lower while a resident sews or mends (`presser`), and the handwheel turns for as
//! long as they do (`handwheel`, a loop).

use super::M;
use super::common::{P, chair};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::kit::{self, fillet};
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, cos, cosd, radians, sin, sind};
use artkit::motion::{Spin, spin};
use artkit::parts::{Settle, hinged, join, new_parts, settle_with};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, jitter, revolve, slide, torus};

const MACHINE_Z: f64 = 0.28;
// The sewer sits behind, facing the front (+z), so their left is game -x: the needle and the head are on their left
// and the pillar and the handwheel on their right, the cloth fed towards them, as on every domestic machine (review
// M-hobbies-08 read it as mirrored: it isn't, checked in play 2026-10-21).
const NEEDLE_X: f64 = -0.12;
const WHEEL: P = (0.212, 0.94, MACHINE_Z);

/// A turned knob (`kit.knob`) on a face turned to the back (game -z), at depth `face`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn knob_back(
    fr: &GameFrame,
    name: &str,
    x: impl Coord,
    y: f64,
    face: f64,
    r: f64,
    material: Mat,
) -> Obj {
    let obj = kit::knob!(fr, name, x, y, face, r, material, segments = 10);
    obj.edit(|o| o.rot.x = radians(-90.0) as f32);
    obj
}

pub fn sewing_table() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        // the table: a top with rounded corners on tapered legs, a shaped apron and a drawer at the back (the sewer's side)
        fextrude!(
            f,
            "top",
            &rounded_rect(0.9, 0.46, 0.025, 2),
            0.035,
            (0, 0.735, 0.2),
            M.sew_table,
            plane = Facing::Top,
            bevel = 0.006,
            bevel_segments = 1
        );
        for sx in [-1, 1] {
            let x = sx as f64 * 0.4;
            for z in [0.02, 0.4] {
                fsweep!(
                    f,
                    format!("leg{sx}{}", r(z)),
                    &[(x, 0.718, z), (x, 0.62, z), (x, 0.0, z)],
                    0,
                    M.sew_legs,
                    profile = rounded_rect(0.04, 0.04, 0.006, 1),
                    up = (1, 0, 0),
                    scale = [1.0, 1.0, 0.65]
                );
            }
            fbox!(
                f,
                format!("apron_side{sx}"),
                0.02,
                0.07,
                0.34,
                (x, 0.68, 0.21),
                M.sew_legs
            );
        }
        let edge: Vec<(f64, f64)> = (0..13)
            .map(|i| {
                (
                    0.38 - 0.76 * i as f64 / 12.0,
                    0.652 - 0.012 * sin(PI * i as f64 / 12.0),
                )
            })
            .collect();
        let mut apron = vec![(0.38, 0.715), (-0.38, 0.715)];
        apron.extend(edge.iter().rev());
        fextrude!(
            f,
            "apron_front",
            &apron,
            0.02,
            (0, 0, 0.4),
            M.sew_legs,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "drawer",
            0.4,
            0.07,
            0.02,
            (0, 0.68, 0.02),
            M.sew_legs,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "drawer_bead",
            0.37,
            0.05,
            0.004,
            (0, 0.68, 0.009),
            M.sew_legs,
            bevel = 0.0015,
            bevel_segments = 1
        );
        knob_back(&f, "drawer_knob", 0, 0.68, 0.007, 0.009, M.brass);
        // the machine: one cast body (the pillar, the arm and the head over a gap for the needle), on a deeper bed
        let body = [
            (-0.17, 0.79),
            (0.21, 0.79),
            (0.21, 0.82),
            (0.2, 0.86),
            (0.2, 1.0),
            (0.18, 1.03),
            (-0.14, 1.03),
            (-0.165, 1.01),
            (-0.165, 0.89),
            (-0.075, 0.885),
            (-0.075, 0.95),
            (0.11, 0.95),
            (0.11, 0.84),
            (0.09, 0.813),
            (-0.17, 0.813),
        ];
        fextrude!(
            f,
            "machine",
            &body,
            0.13,
            (0, 0, MACHINE_Z),
            M.machine,
            bevel = 0.018,
            bevel_segments = 2
        );
        fextrude!(
            f,
            "bed",
            &rounded_rect(0.38, 0.17, 0.02, 2),
            0.06,
            (0.02, 0.783, MACHINE_Z),
            M.machine,
            plane = Facing::Top,
            bevel = 0.01,
            bevel_segments = 2
        );
        // its trim: a stripe along the arm, the stitch dial and lever, a light under the head, the bobbin cover
        fbox!(
            f,
            "stripe",
            0.26,
            0.012,
            0.002,
            (-0.01, 1.0, MACHINE_Z - 0.066),
            M.machine_trim
        );
        knob_back(
            &f,
            "dial",
            0.155,
            0.93,
            MACHINE_Z - 0.063,
            0.016,
            M.machine_trim,
        );
        fbox!(
            f,
            "stitch_lever",
            0.008,
            0.04,
            0.008,
            (0.155, 0.87, MACHINE_Z - 0.068),
            M.machine_trim,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "bobbin_cover",
            0.07,
            0.002,
            0.05,
            (NEEDLE_X + 0.05, 0.8138, MACHINE_Z + 0.02),
            M.chrome,
            bevel = 0.001,
            bevel_segments = 1
        );
        fbox!(
            f,
            "needle_plate",
            0.06,
            0.003,
            0.05,
            (NEEDLE_X, 0.8145, MACHINE_Z),
            M.chrome
        );
        for k in 0..3 {
            fbox!(
                f,
                format!("feed_dog{k}"),
                0.004,
                0.001,
                0.02,
                (
                    NEEDLE_X - 0.008 + 0.008 * k as f64,
                    0.8165,
                    MACHINE_Z + 0.004
                ),
                M.foot
            );
        }
        kit::tube!(
            f,
            "presser_bar_top",
            &[
                (NEEDLE_X, 0.88, MACHINE_Z + 0.012),
                (NEEDLE_X, 0.86, MACHINE_Z + 0.012)
            ],
            0.004,
            M.chrome,
            verts = 6,
            round = 1
        );
        kit::tube!(
            f,
            "presser_lever",
            &fillet(
                &[
                    (NEEDLE_X - 0.03, 0.93, MACHINE_Z + 0.05),
                    (NEEDLE_X - 0.03, 0.93, MACHINE_Z + 0.075),
                    (NEEDLE_X - 0.045, 0.91, MACHINE_Z + 0.085)
                ],
                0.008,
                3
            ),
            0.003,
            M.chrome,
            verts = 6,
            round = 1
        );
        fbox!(
            f,
            "tension_dial",
            0.012,
            0.03,
            0.03,
            (NEEDLE_X + 0.05, 0.99, MACHINE_Z - 0.065),
            M.machine_trim,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the spool on its pin, the thread up through the guide and down to the needle
        fcyl!(
            f,
            "spool_pin",
            0.003,
            0.05,
            (0.05, 1.055, MACHINE_Z),
            M.chrome,
            verts = 6
        );
        revolve!(
            "spool",
            &[
                (0.0, 0.0),
                (0.017, 0.0),
                (0.017, 0.004),
                (0.014, 0.005),
                (0.014, 0.027),
                (0.017, 0.028),
                (0.017, 0.032),
                (0.0, 0.032)
            ],
            at = f.at(0.05, 1.03, MACHINE_Z),
            material = M.thread_red,
            segments = 12
        );
        kit::tube!(
            f,
            "thread",
            &[
                (0.05, 1.05, MACHINE_Z - 0.016),
                (-0.02, 1.035, MACHINE_Z - 0.066),
                (NEEDLE_X + 0.02, 0.96, MACHINE_Z - 0.068),
                (NEEDLE_X, 0.9, MACHINE_Z - 0.06)
            ],
            0.0008,
            M.thread_red,
            verts = 4,
            round = 1
        );
        // the cloth fed under the needle, rumpled; a pin cushion with pins, scissors
        let cloth = fextrude!(
            f,
            "cloth",
            &rounded_rect(0.4, 0.3, 0.01, 1),
            0.004,
            (-0.2, 0.7545, 0.14),
            M.fabric_pink,
            plane = Facing::Top,
            rot = (0, 0, -12)
        );
        jitter!(cloth, 0.003, size = 0.08, seed = 3.0, up = 1.0);
        fsphere!(
            f,
            "cushion",
            0.03,
            (0.3, 0.775, 0.36),
            M.ball_red,
            scale = (1, 1, 0.75),
            segments = 12,
            rings = 7
        );
        for (k, (dx, dz)) in [(0.0, 0.0), (0.012, 0.01), (-0.01, 0.012), (0.008, -0.012)]
            .into_iter()
            .enumerate()
        {
            fsphere!(
                f,
                format!("pin{k}"),
                0.004,
                (0.3 + dx, 0.8, 0.36 + dz),
                [M.sun, M.thread_teal, M.label, M.vice][k],
                segments = 6,
                rings = 4
            );
        }
        for s in [-1, 1] {
            let sf = s as f64;
            fextrude!(
                f,
                format!("blade{s}"),
                &[(0.0, -0.004), (0.11, -0.001), (0.11, 0.001), (0.0, 0.004)],
                0.002,
                (0.14, 0.7535 + 0.002 * (sf + 1.0), 0.05),
                M.chrome,
                plane = Facing::Top,
                rot = (0, 0, 8 * s - 30)
            );
            let a = (30 - 8 * s) as f64;
            torus!(
                format!("scissor_ring{s}"),
                0.014,
                0.003,
                at = f.at(
                    0.14 - 0.016 * cosd(a) + sf * 0.006,
                    0.7555,
                    0.05 + 0.016 * sind(a)
                ),
                material = M.ball_red,
                major_segments = 10,
                minor_segments = 4
            );
        }
        // two reels of thread, a sewing basket of yarn
        for (k, (z, colour)) in [(0.12, M.thread_red), (0.22, M.thread_teal)]
            .into_iter()
            .enumerate()
        {
            revolve!(
                format!("reel{k}"),
                &[
                    (0.0, 0.0),
                    (0.028, 0.0),
                    (0.028, 0.006),
                    (0.022, 0.007),
                    (0.022, 0.043),
                    (0.028, 0.044),
                    (0.028, 0.05),
                    (0.0, 0.05)
                ],
                at = f.at(0.36, 0.7525, z),
                material = M.sew_table,
                segments = 12
            );
            fcyl!(
                f,
                format!("reel_thread{k}"),
                0.0225,
                0.036,
                (0.36, 0.7775, z),
                colour,
                verts = 12
            );
        }
        revolve!(
            "basket",
            &[
                (0.0, 0.0),
                (0.094, 0.0),
                (0.106, 0.12),
                (0.11, 0.124),
                (0.106, 0.128),
                (0.098, 0.12),
                (0.088, 0.012),
                (0.0, 0.012)
            ],
            at = f.at(-0.32, 0.7525, 0.36),
            material = M.basket,
            segments = 20
        );
        let handle: Vec<P> = (0..9)
            .map(|i| {
                (
                    -0.32 + 0.105 * cos(PI * i as f64 / 8.0),
                    0.875 + 0.09 * sin(PI * i as f64 / 8.0),
                    0.36,
                )
            })
            .collect();
        kit::tube!(
            f,
            "basket_handle",
            &handle,
            0.005,
            M.basket,
            verts = 6,
            round = 1
        );
        for (k, (dx, dz, colour)) in [
            (-0.03, 0.0, M.thread_teal),
            (0.035, 0.02, M.fabric_pink),
            (0.0, -0.04, M.machine_trim),
        ]
        .into_iter()
        .enumerate()
        {
            fsphere!(
                f,
                format!("yarn{k}"),
                0.045,
                (-0.32 + dx, 0.84, 0.36 + dz),
                colour,
                segments = 12,
                rings = 8
            );
        }
        kit::tube!(
            f,
            "knitting_needle",
            &[(-0.36, 0.82, 0.36), (-0.24, 0.92, 0.33)],
            0.0025,
            M.chrome,
            verts = 4,
            round = 1
        );
        chair(&f, -0.22, M.sew_table, M.sew_pad, 0.42, 0.4);
    });
    join(&still, "body", Some(P3::ZERO));
    let p = new_parts(|| {
        kit::tube!(
            f,
            "presser_bar",
            &[
                (NEEDLE_X, 0.88, MACHINE_Z + 0.012),
                (NEEDLE_X, 0.835, MACHINE_Z + 0.012)
            ],
            0.0035,
            M.chrome,
            verts = 6,
            round = 1
        );
        fbox!(
            f,
            "presser_foot",
            0.022,
            0.006,
            0.035,
            (NEEDLE_X, 0.833, MACHINE_Z + 0.005),
            M.chrome,
            bevel = 0.002
        );
        kit::tube!(
            f,
            "needle_bar",
            &[
                (NEEDLE_X, 0.88, MACHINE_Z - 0.006),
                (NEEDLE_X, 0.845, MACHINE_Z - 0.006)
            ],
            0.004,
            M.chrome,
            verts = 6,
            round = 1
        );
        fcyl!(
            f,
            "needle_clamp",
            0.006,
            0.008,
            (NEEDLE_X, 0.848, MACHINE_Z - 0.006),
            M.chrome,
            verts = 6
        );
        kit::tube!(
            f,
            "needle",
            &[
                (NEEDLE_X, 0.845, MACHINE_Z - 0.006),
                (NEEDLE_X, 0.828, MACHINE_Z - 0.006)
            ],
            0.0012,
            M.chrome,
            verts = 4,
            round = 1
        );
    });
    let presser = join(&p, "presser", None);
    let w = new_parts(|| {
        // the handwheel: a rim, a dished face, a hub
        revolve!(
            "handwheel",
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.038, -0.002),
                (0.041, 0.002),
                (0.041, 0.02),
                (0.038, 0.024),
                (0.032, 0.022),
                (0.016, 0.016),
                (0.0, 0.016)
            ],
            at = f.at(WHEEL.0 - 0.011, WHEEL.1, WHEEL.2),
            material = M.machine_trim,
            segments = 24,
            rot = (0, -90, 0)
        );
        fcyl!(
            f,
            "handwheel_hub",
            0.015,
            0.03,
            WHEEL,
            M.chrome,
            rot = (0, 90, 0),
            verts = 12
        );
        fcyl!(
            f,
            "handwheel_knob",
            0.006,
            0.02,
            (WHEEL.0 + 0.02, WHEEL.1 + 0.026, WHEEL.2),
            M.machine,
            rot = (0, 90, 0),
            verts = 8
        );
    });
    let wheel = hinged(join(&w, "handwheel", None), f.at(WHEEL.0, WHEEL.1, WHEEL.2));
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    slide!(presser, "presser", (0, 0, -0.016), seconds = 0.4);
    spin(
        wheel,
        "handwheel",
        Spin {
            turns: 1.0,
            seconds: 1.0,
            axis: Axis::X,
        },
    );
    meshes()
}
