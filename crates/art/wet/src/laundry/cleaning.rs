//! The cleaning things: the upright vacuum cleaner, the mop and bucket, the broom and dustpan, and the ironing board
//! with its iron. Each keeps what a resident takes in hand as a part of its own (content `held`, or the hand prop for
//! the iron), hidden while it's used (`*_off`).

use super::M;
use super::common::WALL;
use artkit::forms::rounded_rect;
use artkit::math::{P3, cos, pow, sin};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, revolve, settle, soft, squash};

/// Hides a held part while it's in the hands (or the iron while the hand prop is drawn).
const OFF: f64 = 0.2;

// --- the upright vacuum cleaner (1x1): standing against the wall, its tools on a clip beside it -------------------

/// The cleaner head's middle on the floor (x, depth): the vacuum pivots there in the hands.
const VAC: (f64, f64) = (-0.04, WALL + 0.175);

pub fn vacuum_cleaner() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (x, z) = VAC;
    // The tools on their clip, on the wall beside it: what stays while the vacuum is out.
    let clip = new_parts(|| {
        let cx = x + 0.235;
        fextrude!(
            f,
            "clip_plate",
            &rounded_rect(0.07, 0.2, 0.02, 3),
            0.014,
            (cx, 0.78, WALL + 0.007),
            M.rubber,
            bevel = 0.003,
            bevel_segments = 1
        );
        // the crevice tool, hanging point down, and the dusting brush
        let crevice: Vec<(f64, f64, f64)> = vec![
            (cx - 0.016, 0.88, WALL + 0.04),
            (cx - 0.016, 0.66, WALL + 0.04),
        ];
        fsweep!(
            f,
            "crevice_tool",
            &crevice,
            0.0,
            M.vac_cup,
            profile = rounded_rect(0.034, 0.022, 0.008, 2),
            up = (0, 0, 1)
        );
        fcyl!(
            f,
            "brush_neck",
            0.017,
            0.07,
            (cx + 0.02, 0.85, WALL + 0.045),
            M.vac_cup
        );
        fcyl!(
            f,
            "brush_head",
            0.03,
            0.035,
            (cx + 0.02, 0.8, WALL + 0.045),
            M.vac_body
        );
        fcyl!(
            f,
            "brush_bristles",
            0.027,
            0.03,
            (cx + 0.02, 0.77, WALL + 0.045),
            M.rubber
        );
    });
    join(&clip, "body", Some(P3::from((0, 0, 0))));
    let vacuum = new_parts(|| {
        // the cleaner head, its bumper and the brush bar's window, the wheels at its back
        soft!(
            f,
            "vac_head",
            0.3,
            0.065,
            0.18,
            (x, 0.045, z + 0.03),
            M.vac_body,
            round = 0.022,
            round_segments = 2
        );
        fbox!(
            f,
            "vac_bumper",
            0.3,
            0.022,
            0.014,
            (x, 0.035, z + 0.118),
            M.rubber,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "vac_window",
            0.2,
            0.006,
            0.06,
            (x, 0.079, z + 0.06),
            M.vac_cup,
            bevel = 0.002,
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            fcyl!(
                f,
                format!("vac_wheel{s}"),
                0.055,
                0.03,
                (x + s * 0.13, 0.055, z - 0.07),
                M.rubber,
                rot = (0, 90, 0),
                verts = 16,
                bevel = 0.008
            );
            fcyl!(
                f,
                format!("vac_hub{s}"),
                0.025,
                0.034,
                (x + s * 0.13, 0.055, z - 0.07),
                M.vac_cup,
                rot = (0, 90, 0),
                verts = 12
            );
        }
        // the motor ball between the wheels, the body up from it, the dust cup on its front
        fsphere!(
            f,
            "vac_motor",
            0.09,
            (x, 0.11, z - 0.07),
            M.vac_body,
            segments = 16,
            rings = 10
        );
        soft!(
            f,
            "vac_body",
            0.15,
            0.52,
            0.1,
            (x, 0.42, z - 0.1),
            M.vac_body,
            round = 0.035,
            round_segments = 2
        );
        fcyl!(
            f,
            "vac_cup",
            0.066,
            0.25,
            (x, 0.4, z - 0.01),
            M.vac_cup,
            verts = 20
        );
        fcyl!(
            f,
            "vac_cup_lid",
            0.07,
            0.035,
            (x, 0.54, z - 0.01),
            M.vac_body,
            verts = 20,
            bevel = 0.008
        );
        fcyl!(
            f,
            "vac_cup_foot",
            0.07,
            0.03,
            (x, 0.262, z - 0.01),
            M.vac_body,
            verts = 20,
            bevel = 0.008
        );
        // the handle's stem and its grip, the hose up its back, the cord wound on its hooks
        fsweep!(
            f,
            "vac_stem",
            &[(x, 0.66, z - 0.1), (x, 1.0, z - 0.105)],
            0.016,
            M.alu,
            verts = 12
        );
        fsweep!(
            f,
            "vac_grip",
            &artkit::kit::fillet!(
                &[
                    (x, 0.98, z - 0.105),
                    (x, 1.06, z - 0.11),
                    (x, 1.075, z - 0.05),
                    (x, 1.035, z - 0.035)
                ],
                0.025
            ),
            0.017,
            M.rubber,
            verts = 12
        );
        fsweep!(
            f,
            "vac_hose",
            &[
                (x + 0.06, 0.16, z - 0.12),
                (x + 0.085, 0.4, z - 0.15),
                (x + 0.07, 0.75, z - 0.15),
                (x + 0.03, 0.9, z - 0.125)
            ],
            0.017,
            M.rubber,
            verts = 10,
            resolution = 4
        );
        soft!(
            f,
            "vac_cord",
            0.07,
            0.2,
            0.035,
            (x - 0.03, 0.74, z - 0.165),
            M.rubber,
            round = 0.014,
            round_segments = 1
        );
    });
    let vacuum = hinged(join(&vacuum, "vacuum", None), f.at(x, 0.0, z));
    let objs = settle!(centre = false);
    squash!(vacuum, "vacuum_off", 0.001, seconds = OFF);
    objs
}

// --- the mop and bucket (1x1): a round bucket with a wringer, the mop standing in it ---------------------------

/// The bucket's middle (x, depth), on the floor.
const BUCKET: (f64, f64) = (0.0, WALL + 0.2);
/// Where the mop stands in it (x, height of the strands' foot).
const MOP: (f64, f64) = (-0.06, 0.035);

pub fn mop_bucket() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (bx, bz) = BUCKET;
    let bucket = new_parts(|| {
        revolve!(
            "bucket",
            &[
                (0.0, 0.0),
                (0.125, 0.0),
                (0.132, 0.008),
                (0.152, 0.262),
                (0.158, 0.266),
                (0.161, 0.272),
                (0.158, 0.278),
                (0.151, 0.2795),
                (0.145, 0.275),
                (0.14, 0.262),
                (0.12, 0.016),
                (0.0, 0.016)
            ],
            at = f.at(bx, 0, bz),
            material = M.bucket,
            segments = 28
        );
        revolve!(
            "bucket_water",
            &[(0.0, 0.0), (0.137, 0.0), (0.137, 0.004), (0.0, 0.004)],
            at = f.at(bx, 0.17, bz),
            material = M.water,
            segments = 28
        );
        // the wringer on the rim's right half: a slotted box, its lever folded down
        fbox!(
            f,
            "wringer",
            0.13,
            0.12,
            0.17,
            (bx + 0.075, 0.25, bz),
            M.wringer,
            bevel = 0.012,
            bevel_segments = 2
        );
        for k in 0..4 {
            fbox!(
                f,
                format!("wringer_slot{k}"),
                0.004,
                0.07,
                0.12,
                (bx + 0.02 + k as f64 * 0.025, 0.315, bz),
                M.rubber
            );
        }
        fsweep!(
            f,
            "wringer_lever",
            &artkit::kit::fillet!(
                &[
                    (bx + 0.14, 0.24, bz - 0.07),
                    (bx + 0.15, 0.38, bz - 0.07),
                    (bx + 0.15, 0.38, bz + 0.07),
                    (bx + 0.14, 0.24, bz + 0.07)
                ],
                0.03
            ),
            0.009,
            M.wringer,
            verts = 8
        );
        // the bail handle, lying back on the rim
        let bail: Vec<(f64, f64, f64)> = (0..=8)
            .map(|k| {
                let a = std::f64::consts::PI * k as f64 / 8.0;
                (bx - 0.16 * cos(a), 0.27 + 0.02 * sin(a), bz - 0.12 * sin(a))
            })
            .collect();
        fsweep!(f, "bucket_bail", &bail, 0.004, M.alu, verts = 6);
    });
    join(&bucket, "body", Some(P3::from((0, 0, 0))));
    let (mx, my) = MOP;
    let mop = new_parts(|| {
        // the strands hanging from the socket into the water, the handle up from it
        revolve!(
            "mop_strands",
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.085, 0.03),
                (0.07, 0.09),
                (0.04, 0.15),
                (0.0, 0.15)
            ],
            at = f.at(bx + mx, my, bz),
            material = M.mop_cotton,
            segments = 16
        );
        fcyl!(
            f,
            "mop_socket",
            0.022,
            0.06,
            (bx + mx, my + 0.17, bz),
            M.wringer,
            verts = 12
        );
        fsweep!(
            f,
            "mop_handle",
            &[(bx + mx, my + 0.2, bz), (bx + mx, 1.29, bz)],
            0.0125,
            M.alu,
            verts = 12
        );
        fcyl!(
            f,
            "mop_grip",
            0.016,
            0.07,
            (bx + mx, 1.29, bz),
            M.rubber,
            verts = 12,
            bevel = 0.006
        );
    });
    let mop = hinged(join(&mop, "mop", None), f.at(bx + mx, my, bz));
    let objs = settle!(centre = false);
    squash!(mop, "mop_off", 0.001, seconds = OFF);
    objs
}

// --- the broom and dustpan (1x1): the broom upright on its bristles against the wall, the pan and brush by it ----

/// The broom's head, middle of its bristles on the floor (x, depth).
const BROOM: (f64, f64) = (-0.12, WALL + 0.06);

pub fn broom_dustpan() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    // The dustpan on the floor beside it, its brush lying in it: what stays while the broom is out.
    let pan = new_parts(|| {
        let (px, pz) = (0.17, WALL + 0.17);
        fbox!(
            f,
            "pan_floor",
            0.24,
            0.006,
            0.2,
            (px, 0.003, pz + 0.01),
            M.pan,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "pan_back",
            0.24,
            0.075,
            0.008,
            (px, 0.0375, pz - 0.09),
            M.pan,
            bevel = 0.003,
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            fextrude!(
                f,
                format!("pan_side{s}"),
                &[(pz - 0.094, 0.0), (pz + 0.11, 0.0), (pz - 0.094, 0.075)],
                0.008,
                (px + s * 0.116, 0, 0),
                M.pan,
                plane = Facing::Side
            );
        }
        fsweep!(
            f,
            "pan_handle",
            &[(px, 0.05, pz - 0.094), (px, 0.075, pz - 0.15)],
            0.0,
            M.pan,
            profile = rounded_rect(0.03, 0.014, 0.006, 2),
            up = (0, 1, 0)
        );
        // the brush: its back and handle in one, the bristles under it
        fsweep!(
            f,
            "brush_back",
            &[
                (px - 0.09, 0.03, pz + 0.04),
                (px + 0.1, 0.035, pz + 0.04),
                (px + 0.2, 0.04, pz + 0.04)
            ],
            0.0,
            M.broom_wood,
            profile = rounded_rect(0.04, 0.016, 0.006, 2),
            up = (0, 1, 0)
        );
        fbox!(
            f,
            "brush_bristles",
            0.18,
            0.022,
            0.034,
            (px, 0.015, pz + 0.04),
            M.bristle,
            bevel = 0.004,
            bevel_segments = 1
        );
    });
    join(&pan, "body", Some(P3::from((0, 0, 0))));
    let (x, z) = BROOM;
    let broom = new_parts(|| {
        fbox!(
            f,
            "broom_bristles",
            0.29,
            0.075,
            0.05,
            (x, 0.0375, z),
            M.bristle,
            bevel = 0.006,
            bevel_segments = 1
        );
        soft!(
            f,
            "broom_stock",
            0.3,
            0.04,
            0.06,
            (x, 0.095, z),
            M.broom_wood,
            round = 0.012,
            round_segments = 2
        );
        fsweep!(
            f,
            "broom_handle",
            &[(x, 0.11, z), (x, 1.37, z)],
            0.012,
            M.broom_wood,
            verts = 12
        );
        fcyl!(
            f,
            "broom_cap",
            0.014,
            0.04,
            (x, 1.375, z),
            M.pan,
            verts = 12,
            bevel = 0.006
        );
    });
    let broom = hinged(join(&broom, "broom", None), f.at(x, 0.0, z));
    let objs = settle!(centre = false);
    squash!(broom, "broom_off", 0.001, seconds = OFF);
    objs
}

// --- the ironing board (2x1): a padded board on crossed legs, the iron on its rest at the right ----------------

/// The board's top (height) and its middle (depth): a little to the front, to stand at.
const BOARD_Y: f64 = 0.9;
const BOARD_Z: f64 = 0.1;

pub fn ironing_board() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let board = new_parts(|| {
        // the board's outline: square at the right end, narrowing to a rounded point at the left
        let mut outline: Vec<(f64, f64)> = vec![(0.6, -0.19), (0.6, 0.19)];
        for k in 0..=12 {
            let t = k as f64 / 12.0;
            let a = std::f64::consts::FRAC_PI_2 * (1.0 - 2.0 * t);
            // a half ellipse from the right end's corners to the nose
            outline.push((-0.1 - 0.52 * pow(cos(a), 0.8), 0.19 * sin(a)));
        }
        let shift = |p: &(f64, f64)| (p.0, p.1 + BOARD_Z);
        let deck: Vec<(f64, f64)> = outline.iter().map(shift).collect();
        fextrude!(
            f,
            "board_deck",
            &deck,
            0.018,
            (0, BOARD_Y - 0.03, 0),
            M.alu,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        let cover: Vec<(f64, f64)> = outline
            .iter()
            .map(|p| (p.0 * 1.006, p.1 * 1.03 + BOARD_Z))
            .collect();
        fextrude!(
            f,
            "board_cover",
            &cover,
            0.014,
            (0, BOARD_Y - 0.014, 0),
            M.board_cover,
            plane = Facing::Top,
            bevel = 0.006,
            bevel_segments = 2
        );
        // the iron's rest at the right end: a wire frame
        fsweep!(
            f,
            "board_rest",
            &artkit::kit::fillet!(
                &[
                    (0.58, BOARD_Y - 0.02, BOARD_Z - 0.09),
                    (0.8, BOARD_Y - 0.02, BOARD_Z - 0.09),
                    (0.8, BOARD_Y - 0.02, BOARD_Z + 0.09),
                    (0.58, BOARD_Y - 0.02, BOARD_Z + 0.09)
                ],
                0.03
            ),
            0.005,
            M.alu,
            verts = 6
        );
        // the crossed legs: a U of tube each, from the floor at one end up under the board towards the other
        for (name, foot, top) in [("leg_left", -0.48, 0.24), ("leg_right", 0.46, -0.26)] {
            for s in [-1.0, 1.0] {
                fsweep!(
                    f,
                    format!("{name}{s}"),
                    &[
                        (foot, 0.02, BOARD_Z + s * 0.14),
                        (top, BOARD_Y - 0.035, BOARD_Z + s * 0.12)
                    ],
                    0.011,
                    M.alu,
                    verts = 10
                );
            }
            fsweep!(
                f,
                format!("{name}_foot"),
                &[(foot, 0.02, BOARD_Z - 0.17), (foot, 0.02, BOARD_Z + 0.17)],
                0.013,
                M.alu,
                verts = 10
            );
            for s in [-1.0, 1.0] {
                fsphere!(
                    f,
                    format!("{name}_cap{s}"),
                    0.02,
                    (foot, 0.02, BOARD_Z + s * 0.175),
                    M.rubber,
                    segments = 10,
                    rings = 6
                );
            }
        }
        fsweep!(
            f,
            "leg_pivot",
            &[(-0.01, 0.47, BOARD_Z - 0.15), (-0.01, 0.47, BOARD_Z + 0.15)],
            0.007,
            M.alu,
            verts = 8
        );
    });
    join(&board, "body", Some(P3::from((0, 0, 0))));
    let iron = new_parts(|| {
        artkit::kit::steam_iron(
            f,
            (0.69, BOARD_Y - 0.012, BOARD_Z),
            (-1.0, 0.0),
            M.iron_plate,
            M.iron_body,
            M.iron_accent,
        )
    });
    let iron = hinged(
        join(&iron, "iron", None),
        f.at(0.69, BOARD_Y - 0.012, BOARD_Z),
    );
    let objs = settle!(centre = false);
    squash!(iron, "iron_off", 0.001, seconds = OFF);
    objs
}
