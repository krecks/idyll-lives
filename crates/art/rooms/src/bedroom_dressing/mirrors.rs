//! The four dressing things.

use super::common::{F, M, ellipse, frame_loop, moulding, oval, pane, rounded, side_knob, turned};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, M4, P3, PI, TAU, cos, radians, sin, v3};
use artkit::parts::{aside, join, join_all, new_parts, settle_all};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{
    cylinder, fbox, fcone, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, rod, settle,
    slide, soft,
};

// --- the cheval mirror: 1x1, a tall swing frame between two turned posts on feet -----------------------
// The old part's frame leaned back 6°; this one hangs upright, so the reflection lies on the glass.

const CHEVAL_Y: f64 = 0.95; // the frame's middle and its pivots
const CHEVAL_W: f64 = 0.57; // the frame's moulding, along its middle
const CHEVAL_H: f64 = 1.57;
const CHEVAL_RAIL: f64 = 0.05; // the moulding's width
const POST_X: f64 = 0.35;

/// The cheval mirror.
pub fn mirror() -> Vec<Obj> {
    let rail = CHEVAL_RAIL;
    // the swing frame: a moulding with a cove and a bead, the glass (front, +z) set in behind its lip, the board
    // behind
    frame_loop(
        "frame",
        &rounded(CHEVAL_W, CHEVAL_H, 0.03, 4),
        &moulding(rail, 0.045),
        (0, CHEVAL_Y, 0.0),
        M.frame,
        1.0,
    );
    pane(
        "glass",
        CHEVAL_W - rail + 0.01,
        CHEVAL_H - rail + 0.01,
        0.004,
        (0, CHEVAL_Y, 0.009),
        M.mirror,
    );
    fbox!(
        F,
        "backing",
        CHEVAL_W - 0.02,
        CHEVAL_H - 0.02,
        0.01,
        (0, CHEVAL_Y, -0.005),
        M.backing
    );
    let top = CHEVAL_Y + CHEVAL_H / 2.0 + rail / 2.0;
    // a crest over it, rising to the finial in a shallow arch with a scroll at each end
    let mut crest = vec![(0.33, top + 0.0), (0.33, top + 0.03), (0.3, top + 0.04)];
    crest.extend((0..13).map(|i| {
        (
            0.27 - 0.54 * i as f64 / 12.0,
            top + 0.04 + 0.012 * sin(PI * i as f64 / 12.0),
        )
    }));
    crest.extend([(-0.3, top + 0.04), (-0.33, top + 0.03), (-0.33, top + 0.0)]);
    crest.reverse();
    fextrude!(
        F,
        "crest",
        &crest,
        0.05,
        (0, 0, 0),
        M.frame,
        bevel = 0.008,
        bevel_segments = 2
    );
    turned(
        "crest_finial",
        0,
        0,
        &[
            (0.0, top + 0.05),
            (0.016, top + 0.05),
            (0.01, top + 0.058),
            (0.024, top + 0.075),
            (0.012, top + 0.093),
            (0.0, top + 0.096),
        ],
        M.frame,
        16,
    );
    // two turned posts on their feet, a turned stretcher low between them
    for s in [-1, 1] {
        let x = s as f64 * POST_X;
        let mut foot = vec![
            (0.28, 0.0),
            (0.28, 0.02),
            (0.24, 0.045),
            (0.1, 0.052),
            (-0.1, 0.052),
            (-0.24, 0.045),
            (-0.28, 0.02),
            (-0.28, 0.0),
            (-0.2, 0.0),
            (-0.18, 0.012),
        ];
        foot.extend((0..9).map(|i| {
            (
                -0.16 + 0.32 * i as f64 / 8.0,
                0.012 + 0.008 * sin(PI * i as f64 / 8.0),
            )
        }));
        foot.extend([(0.18, 0.012), (0.2, 0.0)]);
        foot.reverse();
        fextrude!(
            F,
            format!("foot{s}"),
            &foot,
            0.052,
            (x, 0, 0),
            M.frame,
            plane = Facing::Side,
            bevel = 0.008,
            bevel_segments = 2
        );
        turned(
            &format!("post{s}"),
            x,
            0,
            &[
                (0.0, 0.05),
                (0.026, 0.05),
                (0.028, 0.07),
                (0.026, 0.1),
                (0.018, 0.125),
                (0.022, 0.15),
                (0.015, 0.18),
                (0.015, 0.5),
                (0.019, 0.55),
                (0.016, 0.6),
                (0.02, 0.86),
                (0.027, 0.89),
                (0.027, 0.92),
                (0.022, 0.94),
                (0.027, 0.97),
                (0.027, 0.99),
                (0.012, 1.0),
                (0.0, 1.0),
            ],
            M.frame,
            16,
        );
        turned(
            &format!("finial{s}"),
            x,
            0,
            &[
                (0.0, 0.996),
                (0.013, 0.996),
                (0.009, 1.004),
                (0.024, 1.02),
                (0.026, 1.03),
                (0.02, 1.042),
                (0.008, 1.048),
                (0.0, 1.049),
            ],
            M.frame,
            14,
        );
        // the pivot: an axle from the frame into the post, a turned brass knob outside it
        cylinder!(
            format!("axle{s}"),
            0.008,
            0.07,
            at = F.at(s as f64 * 0.335, CHEVAL_Y, 0),
            material = M.brass,
            rot = (0, 90, 0),
            verts = 10
        );
        side_knob(
            &format!("knob{s}"),
            s as f64 * 0.374,
            CHEVAL_Y,
            0,
            0.014,
            s,
            M.brass,
        );
    }
    fsweep!(
        F,
        "stretcher",
        &[
            (-POST_X, 0.09, 0),
            (-0.2, 0.09, 0),
            (-0.06, 0.09, 0),
            (0.0, 0.09, 0),
            (0.06, 0.09, 0),
            (0.2, 0.09, 0),
            (POST_X, 0.09, 0)
        ],
        0.013,
        M.frame,
        verts = 12,
        scale = [1.0, 0.8, 1.2, 0.85, 1.2, 0.8, 1.0]
    );
    join_all("mirror");
    settle_all()
}

// --- the dressing table: 1x1, a kneehole table on brass legs, an oval mirror, a pouf -------------------
// Its mirror against the wall, the pouf at the front (depth +0.22, the seat 0.45 high: characters.ts SEATS,
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
            &ellipse(0.29, 0.35, 48),
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
            turned(
                &format!("upright_top{s}"),
                sx * 0.33,
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
                12,
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

// --- the wall mirror: on the wall layer, a black metal frame and a small ledge --------------------------
// Its back on the wall face (depth -0.43), at the old part's height: the frame from 1.0 to 1.9 m.

const WALL: f64 = -0.43;
const WM_Y: f64 = 1.45;
const WM_W: f64 = 0.62;
const WM_H: f64 = 0.9;
const WM_RIM: f64 = 0.03;
const WM_T: f64 = 0.035;

/// The wall mirror.
pub fn mirror_wall() -> Vec<Obj> {
    let zf = WALL + WM_T / 2.0;
    frame_loop(
        "frame",
        &rounded(WM_W - WM_RIM, WM_H - WM_RIM, 0.012, 3),
        &moulding(WM_RIM, WM_T),
        (0, WM_Y, zf),
        M.wall_frame,
        1.0,
    );
    pane(
        "glass",
        WM_W - 2.0 * WM_RIM + 0.006,
        WM_H - 2.0 * WM_RIM + 0.006,
        0.004,
        (0, WM_Y, WALL + WM_T - 0.008),
        M.mirror,
    );
    fbox!(
        F,
        "backing",
        WM_W - 0.04,
        WM_H - 0.04,
        0.012,
        (0, WM_Y, WALL + 0.008),
        M.wall_frame
    );
    // the ledge under it: a tray with a rolled lip, on two bent brackets; a cup, a toothbrush and the toothpaste
    let shelf_y = 0.972;
    let tray = [
        (0.0, -0.0425),
        (0.0, 0.045),
        (0.006, 0.0472),
        (0.022, 0.0468),
        (0.026, 0.0428),
        (0.024, 0.0388),
        (0.012, 0.0398),
        (0.012, -0.0425),
    ];
    let profile: Vec<(f64, f64)> = tray.iter().map(|&(u, v)| (u, -v)).collect();
    fsweep!(
        F,
        "shelf",
        &[
            (-0.26, shelf_y - 0.006, WALL + 0.0425),
            (0.26, shelf_y - 0.006, WALL + 0.0425)
        ],
        0,
        M.wall_frame,
        profile = profile,
        up = (0, 0, 1),
        angle = 35
    );
    for s in [-1, 1] {
        let sx = s as f64;
        fsweep!(
            F,
            format!("bracket{s}"),
            &kit::fillet(
                &[
                    (sx * 0.2, shelf_y - 0.036, WALL + 0.002),
                    (sx * 0.2, shelf_y - 0.012, WALL + 0.002),
                    (sx * 0.2, shelf_y - 0.012, WALL + 0.06)
                ],
                0.016,
                3
            ),
            0,
            M.wall_frame,
            profile = rounded_rect(0.004, 0.012, 0.0015, 1),
            up = (0, 1, 0)
        );
    }
    let top = shelf_y + 0.006;
    revolve!(
        "cup",
        &[
            (0.0, 0.0),
            (0.019, 0.0),
            (0.021, 0.004),
            (0.024, 0.082),
            (0.0245, 0.085),
            (0.0215, 0.085),
            (0.018, 0.012),
            (0.0, 0.012)
        ],
        at = F.at(-0.16, top, -0.368),
        material = M.cup,
        segments = 16
    );
    fsweep!(
        F,
        "toothbrush",
        &[
            (-0.165, top + 0.015, -0.368),
            (-0.16, top + 0.07, -0.369),
            (-0.152, top + 0.11, -0.371),
            (-0.149, top + 0.135, -0.372)
        ],
        0.0045,
        M.brush,
        verts = 6,
        resolution = 2,
        scale = [1.0, 0.8, 0.75, 0.9]
    );
    fbox!(
        F,
        "toothbrush_head",
        0.01,
        0.028,
        0.008,
        (-0.148, top + 0.122, -0.3665),
        M.brush,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        "toothbrush_bristles",
        0.008,
        0.024,
        0.01,
        (-0.148, top + 0.122, -0.3585),
        M.bristle,
        bevel = 0.002,
        bevel_segments = 1
    );
    let b = F.at(0.12, 0, -0.385);
    let (bx, by) = (b.x, b.y);
    let secs: Vec<Vec<P3>> = [
        (0.004, 0.002, 0.016),
        (0.01, 0.01, 0.016),
        (0.11, 0.016, 0.015),
        (0.125, 0.012, 0.012),
    ]
    .iter()
    .map(|&(yy, rx, ry)| ring!(rx, ry, z = top + yy, verts = 12, at = (bx + 0.0, by)))
    .collect();
    loft!("paste", &secs, material = M.paste, rot = (0, 0, 0));
    revolve!(
        "paste_cap",
        &[(0.0, 0.0), (0.011, 0.0), (0.011, 0.018), (0.0, 0.018)],
        at = F.at(0.12, top + 0.125, -0.385),
        material = M.paste_cap,
        segments = 10
    );
    join_all("mirrorWall");
    settle!(centre = false, floor = false)
}

// --- the standing mirror: 1x1, a full-length mirror upright on a brass stand ----------------------------
// Where the game places it: the frame a little behind the middle, the stand's strut behind it. The
// old part leaned back 8°; this one stands upright on two sled feet, held by a strut from behind.

const SM_Z: f64 = -0.05; // the frame's middle, in depth
const SM_W: f64 = 0.56;
const SM_H: f64 = 1.66;
const SM_RIM: f64 = 0.028;
const SM_T: f64 = 0.035;
const SM_Y0: f64 = 0.12; // the frame's bottom
const FOOT_X: f64 = 0.22;
const FOOT_Z0: f64 = -0.38;
const FOOT_Z1: f64 = 0.07;

/// The standing mirror.
pub fn mirror_floor() -> Vec<Obj> {
    let y = SM_Y0 + SM_H / 2.0;
    frame_loop(
        "frame",
        &rounded(SM_W - SM_RIM, SM_H - SM_RIM, 0.03, 4),
        &moulding(SM_RIM, SM_T),
        (0, y, SM_Z),
        M.stand,
        1.0,
    );
    pane(
        "glass",
        SM_W - 2.0 * SM_RIM + 0.006,
        SM_H - 2.0 * SM_RIM + 0.006,
        0.004,
        (0, y, SM_Z + SM_T / 2.0 - 0.0105),
        M.mirror,
    );
    fbox!(
        F,
        "backing",
        SM_W - 0.03,
        SM_H - 0.03,
        0.012,
        (0, y, SM_Z - 0.008),
        M.stand_back
    );
    // the stand: two sled feet turned up at the toe, on glides; a clamp up from each to the frame, tightened by a
    // knob; a rear bar, and a strut folding at a hinge up to a plate on the frame's back
    for s in [-1, 1] {
        let x = s as f64 * FOOT_X;
        fsweep!(
            F,
            format!("foot{s}"),
            &kit::fillet(
                &[
                    (x, 0.012, FOOT_Z0),
                    (x, 0.012, FOOT_Z1 - 0.03),
                    (x, 0.018, FOOT_Z1)
                ],
                0.03,
                3
            ),
            0,
            M.stand,
            profile = rounded_rect(0.016, 0.024, 0.006, 2),
            up = (0, 0, 1)
        );
        for z in [FOOT_Z0 + 0.03, FOOT_Z1 - 0.05] {
            fcyl!(
                F,
                format!("glide{s}{}", float_repr(z)),
                0.009,
                0.006,
                (x, 0.003, z),
                M.stand_back,
                verts = 10
            );
        }
        soft!(
            F,
            format!("clamp{s}"),
            0.026,
            SM_Y0 + 0.04 - 0.02,
            SM_T + 0.012,
            (x, (0.02 + SM_Y0 + 0.04) / 2.0, SM_Z),
            M.stand,
            round = 0.006,
            round_segments = 1
        );
        let k = kit::knob!(
            F,
            format!("clamp_knob{s}"),
            x,
            SM_Y0 + 0.01,
            SM_Z - SM_T / 2.0 - 0.006,
            0.01,
            M.stand
        );
        k.set_rotation_euler((radians(-90.0), 0.0, 0.0)); // out to the back
    }
    fsweep!(
        F,
        "rear_bar",
        &[
            (-FOOT_X, 0.012, FOOT_Z0 + 0.02),
            (FOOT_X, 0.012, FOOT_Z0 + 0.02)
        ],
        0.012,
        M.stand,
        verts = 12
    );
    let back = SM_Z - SM_T / 2.0;
    let mid = (0, 0.71, (FOOT_Z0 + 0.02 + back - 0.006) / 2.0);
    kit::tube!(
        F,
        "strut_low",
        &[(0, 0.02, FOOT_Z0 + 0.02), mid],
        0.011,
        M.stand,
        verts = 12,
        round = 1
    );
    kit::tube!(
        F,
        "strut_high",
        &[mid, (0, 1.40, back - 0.006)],
        0.01,
        M.stand,
        verts = 12,
        round = 1
    );
    cylinder!(
        "strut_hinge",
        0.014,
        0.034,
        at = F.at(mid.0, mid.1, mid.2),
        material = M.stand,
        rot = (0, 90, 0),
        verts = 12
    );
    soft!(
        F,
        "strut_plate",
        0.07,
        0.06,
        0.008,
        (0, 1.40, back - 0.004),
        M.stand,
        round = 0.003,
        round_segments = 1
    );
    join_all("mirrorFloor");
    settle!(centre = false)
}
