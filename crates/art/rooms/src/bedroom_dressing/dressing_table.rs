//! The dressing table: a mirror over a drawer case, with the bottles on it.

use super::common::{F, M, ellipse, frame_loop, moulding, oval, side_knob};
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, M4, P3, PI, TAU, cos, radians, sin, v3};
use artkit::parts::{aside, join, new_parts};
use artkit::scene::Obj;
use artkit::{
    fbox, fcone, fextrude, fsphere, fsweep, loft, revolve, ring, rod, settle, slide, soft,
};

// --- the cheval mirror: 1x1, a tall swing frame between two turned posts on feet -----------------------
// The old part's frame leaned back 6°; this one hangs upright, so the reflection lies on the glass.

// --- the dressing table: 1x1, a kneehole table on brass legs, an oval mirror, a pouf -------------------
// Its mirror against the wall, the pouf at the front (depth +0.22, the seat 0.45 high: furniture.json `seats`,
// `back`: the sitter faces the mirror). The numbers below are as it was first built, facing the wall (the
// pouf at -0.22, the mirror at its far edge facing back at it); `dressing_table` turns it round at the end.

const TOP_Y: f64 = 0.76; // the table top's surface
const CASE_Y0: f64 = 0.585; // the drawer cases' bottom
const CASE_Z0: f64 = 0.04; // the cases' face to the seat, and their back
const CASE_Z1: f64 = 0.41;
const OVAL_Y: f64 = 1.165; // the mirror's middle
const OVAL_Z: f64 = 0.395;
const DRAWER_X: f64 = 0.26; // the drawer that opens: on the seated resident's right (game +x: the game is left-handed)
const DRAWER_OUT: f64 = 0.18;
const DRAWER_SECONDS: f64 = 0.8;
pub const DRAWER_HOLD: f64 = 3.0;
const TURN_BACK: f64 = 0.01; // turned round, moved back (Blender +y) to put the mirror's back on the wall face

/// A drawer front on a case's face to the seat, its edges rounded, with a turned brass knob.
fn drawer_front(name: &str, x: f64) -> f64 {
    let y = (CASE_Y0 + TOP_Y - 0.035) / 2.0;
    soft!(
        F,
        format!("{name}_front"),
        0.28,
        0.118,
        0.016,
        (x, y, CASE_Z0 - 0.009),
        M.table,
        round = 0.005,
        round_segments = 1
    );
    let k = kit::knob!(
        F,
        format!("{name}_knob"),
        x,
        y,
        CASE_Z0 - 0.017,
        0.011,
        M.table_frame
    );
    k.set_rotation_euler((radians(-90.0), 0.0, 0.0)); // facing the seat (-z, as built)
    y
}

/// A perfume bottle at game (x, z) on height y0: a squarish faceted body, shoulders, a neck and a gilt cap.
#[allow(clippy::too_many_arguments)]
fn bottle(
    name: &str,
    x: f64,
    z: f64,
    y0: f64,
    h: f64,
    r: f64,
    material: Mat,
    cap: Mat,
    square: f64,
) {
    let b = F.at(x, 0, z);
    let (bx, by) = (b.x, b.y);
    let secs: Vec<Vec<P3>> = [
        (0.0, 0.92),
        (0.03, 1.0),
        (0.66, 1.0),
        (0.76, 0.75),
        (0.8, 0.36),
        (0.88, 0.34),
    ]
    .iter()
    .map(|&(t, k)| {
        ring!(
            r * k,
            r * k * 0.7,
            z = y0 + h * t,
            verts = 16,
            square = square,
            at = (bx, by)
        )
    })
    .collect();
    loft!(name, &secs, material = material);
    revolve!(
        format!("{name}_cap"),
        &[
            (0.0, 0.0),
            (r * 0.42, 0.0),
            (r * 0.46, h * 0.05),
            (r * 0.3, h * 0.35),
            (r * 0.2, h * 0.4),
            (0.0, h * 0.41)
        ],
        at = F.at(x, y0 + h * 0.86, z),
        material = cap,
        segments = 12
    );
}

/// The dressing table.
pub fn dressing_table() -> Vec<Obj> {
    let body = new_parts(|| {
        soft!(
            F,
            "top",
            0.86,
            0.035,
            0.40,
            (0, TOP_Y - 0.0175, 0.22),
            M.table,
            round = 0.012,
            round_segments = 2
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("case{s}"),
                0.30,
                TOP_Y - 0.035 - CASE_Y0,
                CASE_Z1 - CASE_Z0,
                (
                    s as f64 * 0.26,
                    (CASE_Y0 + TOP_Y - 0.035) / 2.0,
                    (CASE_Z0 + CASE_Z1) / 2.0
                ),
                M.table,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fbox!(
            F,
            "apron",
            0.24,
            0.05,
            0.02,
            (0, TOP_Y - 0.06, 0.40),
            M.table,
            bevel = 0.003,
            bevel_segments = 1
        );
        // the kneehole's rail to the seat, its lower edge arched
        let edge: Vec<(f64, f64)> = (0..11)
            .map(|i| {
                (
                    0.12 - 0.24 * i as f64 / 10.0,
                    TOP_Y - 0.065 + 0.012 * sin(PI * i as f64 / 10.0),
                )
            })
            .collect();
        let mut rail = vec![(0.12, TOP_Y - 0.035), (-0.12, TOP_Y - 0.035)];
        rail.extend(edge.iter().rev());
        fextrude!(
            F,
            "rail",
            &rail,
            0.02,
            (0, 0, 0.05),
            M.table,
            bevel = 0.003,
            bevel_segments = 1
        );
        // the left drawer stays shut; behind the right one, the dark of the case
        drawer_front("drawer_l", -DRAWER_X);
        fbox!(
            F,
            "cavity",
            0.25,
            0.1,
            0.0015,
            (DRAWER_X, (CASE_Y0 + TOP_Y - 0.035) / 2.0, CASE_Z0 - 0.0012),
            M.inside
        );
        // four tapered brass legs under the cases, each on a turned shoe
        for x in [-0.385, 0.385] {
            for z in [0.07, 0.38] {
                fcone!(
                    F,
                    format!("leg{}{}", float_repr(x), float_repr(z)),
                    0.011,
                    0.018,
                    CASE_Y0 - 0.03,
                    (x, (CASE_Y0 + 0.03) / 2.0, z),
                    M.table_frame,
                    verts = 12
                );
                revolve!(
                    format!("shoe{}{}", float_repr(x), float_repr(z)),
                    &[
                        (0.0, 0.0),
                        (0.015, 0.0),
                        (0.016, 0.006),
                        (0.011, 0.012),
                        (0.012, 0.024),
                        (0.0105, 0.031),
                        (0.0, 0.031)
                    ],
                    at = F.at(x, 0, z),
                    material = M.table_frame,
                    segments = 12
                );
            }
        }
        // the oval mirror on a brass stand at the far edge, facing the seat
        frame_loop(
            "ring",
            &ellipse(0.29, 0.35, 36),
            &moulding(0.035, 0.03),
            (0, OVAL_Y, OVAL_Z),
            M.table_frame,
            -1.0,
        );
        oval(
            "glass",
            0.545,
            0.665,
            0.003,
            (0, OVAL_Y, OVAL_Z - 0.0055),
            M.mirror,
            &[0.96, 0.62, 0.3],
        );
        oval(
            "back",
            0.56,
            0.68,
            0.008,
            (0, OVAL_Y, OVAL_Z + 0.005),
            M.table,
            &[],
        );
        soft!(
            F,
            "stand_base",
            0.70,
            0.014,
            0.05,
            (0, TOP_Y + 0.007, OVAL_Z),
            M.table_frame,
            round = 0.005,
            round_segments = 1
        );
        for s in [-1, 1] {
            let sx = s as f64;
            kit::tube!(
                F,
                format!("upright{s}"),
                &[
                    (sx * 0.33, TOP_Y, OVAL_Z),
                    (sx * 0.33, OVAL_Y + 0.03, OVAL_Z)
                ],
                0.009,
                M.table_frame,
                verts = 10,
                round = 1
            );
            kit::turned!(
                F,
                format!("upright_top{s}"),
                sx * 0.33,
                0,
                OVAL_Z,
                &[
                    (0.0, OVAL_Y + 0.025),
                    (0.011, OVAL_Y + 0.025),
                    (0.008, OVAL_Y + 0.032),
                    (0.014, OVAL_Y + 0.042),
                    (0.012, OVAL_Y + 0.054),
                    (0.004, OVAL_Y + 0.06),
                    (0.0, OVAL_Y + 0.061),
                ],
                M.table_frame,
                segments = 12
            );
            rod!(
                format!("pivot{s}"),
                F.at(sx * 0.30, OVAL_Y, OVAL_Z),
                F.at(sx * 0.34, OVAL_Y, OVAL_Z),
                0.007,
                material = M.table_frame,
                verts = 10
            );
            side_knob(
                &format!("pivot_knob{s}"),
                sx * 0.339,
                OVAL_Y,
                OVAL_Z,
                0.011,
                s,
                M.table_frame,
            );
        }
        // on the top: two perfume bottles and a jewellery box
        bottle(
            "perfume", 0.30, 0.30, TOP_Y, 0.086, 0.028, M.perfume, M.gilt, 4.0,
        );
        bottle(
            "lotion", 0.355, 0.34, TOP_Y, 0.11, 0.02, M.lotion, M.gilt, 2.2,
        );
        soft!(
            F,
            "jewellery",
            0.16,
            0.036,
            0.10,
            (-0.28, TOP_Y + 0.018, 0.18),
            M.case,
            round = 0.006,
            round_segments = 1
        );
        soft!(
            F,
            "jewellery_lid",
            0.164,
            0.014,
            0.104,
            (-0.28, TOP_Y + 0.042, 0.18),
            M.case,
            round = 0.005,
            round_segments = 1
        );
        fbox!(
            F,
            "jewellery_clasp",
            0.02,
            0.014,
            0.004,
            (-0.28, TOP_Y + 0.032, 0.129),
            M.gilt,
            bevel = 0.001
        );
    });
    let body = join(&body, "body", Some(P3::ZERO));

    // the pouf the resident sits on, out in front of the kneehole: drawn with the table, not part of its size
    // (the kind is the table with its mirror; the pouf is where the seat is measured)
    let carried = new_parts(|| {
        revolve!(
            "pouf",
            &[
                (0.0, 0.045),
                (0.19, 0.045),
                (0.205, 0.06),
                (0.208, 0.36),
                (0.214, 0.39),
                (0.212, 0.415),
                (0.2, 0.437),
                (0.17, 0.449),
                (0.08, 0.45),
                (0.0, 0.447)
            ],
            at = F.at(0, 0, -0.22),
            material = M.pouf,
            segments = 28
        );
        for (k, (r, y)) in [(0.21, 0.405), (0.2, 0.055)].into_iter().enumerate() {
            let pts: Vec<(f64, f64, f64)> = (0..36)
                .map(|i| {
                    (
                        r * cos(TAU * i as f64 / 36.0),
                        y,
                        -0.22 + r * sin(TAU * i as f64 / 36.0),
                    )
                })
                .collect();
            fsweep!(
                F,
                format!("pouf_piping{k}"),
                &pts,
                0.006,
                M.pouf,
                verts = 5,
                closed = true
            );
        }
        fsphere!(
            F,
            "pouf_button",
            0.012,
            (0, 0.444, -0.22),
            M.pouf,
            scale = (1, 1, 0.5),
            segments = 10,
            rings = 6
        );
        for k in 0..4 {
            let a = radians((45 + 90 * k) as f64);
            fcone!(
                F,
                format!("pouf_foot{k}"),
                0.01,
                0.016,
                0.045,
                (0.14 * cos(a), 0.0225, -0.22 + 0.14 * sin(a)),
                M.table_frame,
                verts = 10
            );
        }
    });
    let pouf = aside(&carried, "pouf");

    // the right drawer: its front and the tray behind it, hidden in the case at rest
    let mut y = 0.0;
    let parts = new_parts(|| {
        y = drawer_front("drawer", DRAWER_X);
        let (tray_z0, tray_z1) = (CASE_Z0, CASE_Z0 + 0.3);
        let tz = (tray_z0 + tray_z1) / 2.0;
        fbox!(
            F,
            "drawer_floor",
            0.24,
            0.01,
            0.3,
            (DRAWER_X, y - 0.04, tz),
            M.table,
            bevel = 0.002
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("drawer_side{s}"),
                0.01,
                0.08,
                0.3,
                (DRAWER_X + s as f64 * 0.115, y - 0.005, tz),
                M.table,
                bevel = 0.002
            );
        }
        fbox!(
            F,
            "drawer_back",
            0.24,
            0.08,
            0.01,
            (DRAWER_X, y - 0.005, tray_z1 - 0.005),
            M.table,
            bevel = 0.002
        );
    });
    let drawer = join(&parts, "drawer", Some(F.at(DRAWER_X, y, CASE_Z0)));

    // Built facing the wall (as the old part was), then turned round about the footprint's middle: the
    // mirror's back on the wall face, the pouf at the front, where the resident sits facing the mirror.
    let half = M4::rotation(PI, Axis::Z);
    for o in [body, drawer, pouf] {
        o.edit(|p| {
            p.mesh.transform(&half);
            p.loc = half * p.loc + v3(0, TURN_BACK, 0);
        });
    }
    let objs = settle!(centre = false);
    // out to the seat: the game's +z, Blender's -y
    slide!(
        drawer,
        "drawer_open",
        (0, -DRAWER_OUT, 0),
        seconds = DRAWER_SECONDS
    );
    objs
}
