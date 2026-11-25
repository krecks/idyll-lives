//! The beds.

use super::common::{
    Bedding, Board, Cap, DUVET, F, M, Mattress, SECONDS, WALL, bedding, board, channel, finish,
    flap, legs_round, legs_square, mattress, pillow, pillow_at, post, runner, settle_bed, tufts,
    turn_down,
};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{Axis, P3, PI, TAU, cos, radians, sin};
use artkit::parts::{hinged, join, join_all, new_parts, turn};
use artkit::place::N;
use artkit::place::eased_prism;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsweep, jitter, loft, revolve, rod, soft, swing};

// ---------------------------------------------------------------- the double bed (`bed`)

/// The double bed.
pub fn bed() -> Vec<Obj> {
    let top = 0.58; // BEDS['bed']
    legs_round(&[-0.84, 0.84], &[-0.82, 0.88], 0.12, M.legs_teak, 0.026);
    // the upholstered base, piped round its top and bottom
    soft!(
        F,
        "base",
        1.82,
        0.28,
        1.83,
        (0, 0.26, 0.025),
        M.frame_linen,
        round = 0.04,
        piping = 0.008
    );
    // headboard: a piped padded back with four channels, its back on the wall's face
    soft!(
        F,
        "headboard",
        1.9,
        1.03,
        0.035,
        (0, 0.615, WALL + 0.0175),
        M.headboard_linen,
        round = 0.015,
        piping = 0.006
    );
    for (i, x) in [-0.66, -0.22, 0.22, 0.66].into_iter().enumerate() {
        channel(
            &format!("channel{i}"),
            x,
            0.41,
            0.35,
            1.09,
            -0.905,
            0.035,
            M.headboard_linen,
            false,
        );
    }
    mattress("mattress", 1.72, 0.2, 1.78, top, 0.03, Mattress::default());
    for s in [-1, 1] {
        pillow_at(
            &format!("pillow{s}"),
            0.68,
            0.15,
            0.4,
            s as f64 * 0.42,
            top,
            -0.66,
        );
    }
    let (z_top, z_foot) = (-0.42, 0.92);
    let z_fold = bedding(
        top,
        0.86,
        z_top,
        z_foot,
        0.17,
        M.bedding_cream,
        Bedding {
            runner: Some((0.72 - 0.19, 0.72 + 0.19)),
            ..Bedding::default()
        },
    );
    // the runner across the foot, over the duvet and down its sides
    runner(
        "runner",
        0.909,
        top + DUVET + 0.015,
        0.72,
        0.38,
        0.2,
        M.throw_sage,
    );
    finish("bed", (top, 0.86, z_top, z_fold, M.bedding_cream), 2)
}

// ---------------------------------------------------------------- the single bed

/// The single bed.
pub fn bed_single() -> Vec<Obj> {
    let top = 0.5; // BEDS['bedSingle']
    let head = WALL + 0.03; // the head end's posts and boards, their backs on the wall's face
    for s in [-1, 1] {
        let x = s as f64 * 0.45;
        post(
            &format!("post_head{s}"),
            x,
            head,
            0.95,
            0.06,
            M.frame_oak,
            Cap::Ball,
            0.0,
        );
        post(
            &format!("post_foot{s}"),
            x,
            0.94,
            0.62,
            0.06,
            M.frame_oak,
            Cap::Ball,
            0.0,
        );
        fbox!(
            F,
            format!("rail{s}"),
            0.04,
            0.18,
            1.82,
            (x, 0.25, 0),
            M.frame_oak,
            bevel = 0.008
        );
    }
    fbox!(
        F,
        "rail_head",
        0.84,
        0.18,
        0.04,
        (0, 0.25, head),
        M.frame_oak,
        bevel = 0.008
    );
    fbox!(
        F,
        "rail_foot",
        0.84,
        0.18,
        0.04,
        (0, 0.25, 0.94),
        M.frame_oak,
        bevel = 0.008
    );
    // headboard: an arched top rail over five slats rounded at their ends; a framed, panelled footboard
    board(
        "head_top",
        0.86,
        0.86,
        0.94,
        head,
        0.05,
        M.headboard_oak,
        Board {
            arch: 0.035,
            bevel: 0.01,
            ..Board::default()
        },
    );
    fbox!(
        F,
        "head_low",
        0.84,
        0.05,
        0.035,
        (0, 0.47, head),
        M.headboard_oak,
        bevel = 0.008
    );
    for i in 0..5 {
        let x = -0.28 + i as f64 * 0.14;
        let y1 = 0.885 + 0.035 * cos(PI * x / 0.86);
        soft!(
            F,
            format!("slat{i}"),
            0.08,
            y1 - 0.48,
            0.022,
            (x, (0.48 + y1) / 2.0, head),
            M.headboard_oak,
            round = 0.008
        );
    }
    let frame = rounded_rect(0.84, 0.24, 0.01, 1);
    fextrude!(
        F,
        "footboard",
        &frame,
        0.03,
        (0, 0.47, 0.94),
        M.headboard_oak,
        holes = [rounded_rect(0.74, 0.15, 0.012, 2)],
        bevel = 0.005,
        bevel_segments = 1
    );
    fbox!(
        F,
        "foot_panel",
        0.738,
        0.148,
        0.024,
        (0, 0.47, 0.94),
        M.headboard_oak,
        bevel = 0.008,
        bevel_segments = 1
    );
    soft!(
        F,
        "foot_top",
        0.9,
        0.04,
        0.05,
        (0, 0.6, 0.94),
        M.headboard_oak,
        round = 0.012
    );
    mattress("mattress", 0.84, 0.18, 1.8, top, 0, Mattress::default());
    pillow_at("pillow", 0.62, 0.13, 0.34, 0, top, -0.71);
    let (z_top, z_foot) = (-0.3, 0.885);
    let z_fold = bedding(
        top,
        0.42,
        z_top,
        z_foot,
        0.14,
        M.bedding_sage,
        Bedding::default(),
    );
    finish("bedSingle", (top, 0.42, z_top, z_fold, M.bedding_sage), 1)
}

// ---------------------------------------------------------------- the bunk bed

const LOWER: f64 = 0.44; // furniture.json bunks.bedBunk
const UPPER: f64 = 1.38;

/// The depths of the posts and end boards: the head end's backs on the wall's face.
fn bunk_end(s: i32) -> f64 {
    if s < 0 { WALL + 0.03 } else { 0.93 }
}

/// One bunk: its rails, mattress, pillow and end boards (arched; the upper bunk's with a hand hold); the upper one
/// with a guard rail.
fn bunk(name: &str, top: f64, guard: bool) {
    let y = top - 0.16 - 0.06;
    for s in [-1, 1] {
        fbox!(
            F,
            format!("{name}_rail{s}"),
            0.04,
            0.12,
            1.8,
            (s as f64 * 0.45, y, 0),
            M.frame_oak,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("{name}_end{s}"),
            0.86,
            0.12,
            0.04,
            (0, y, bunk_end(s)),
            M.frame_oak,
            bevel = 0.008,
            bevel_segments = 1
        );
        let h = if guard { 0.36 } else { 0.3 };
        let y0 = top - 0.13;
        let holes = if guard {
            vec![(0.0, y0 + h - 0.075, 0.18, 0.05)]
        } else {
            Vec::new()
        };
        board(
            &format!("{name}_board{s}"),
            0.84,
            y0,
            y0 + h,
            bunk_end(s),
            0.025,
            M.headboard_oak,
            Board {
                arch: 0.03,
                holes,
                n: 10,
                ..Board::default()
            },
        );
    }
    mattress(
        &format!("{name}_mattress"),
        0.84,
        0.16,
        1.8,
        top,
        0,
        Mattress {
            round: 0.03,
            flat: 2,
            ..Mattress::default()
        },
    );
    pillow(
        &format!("{name}_pillow"),
        0.56,
        0.11,
        0.3,
        0,
        top,
        -0.71,
        8.0,
        None,
        top,
        true,
    );
    if guard {
        fbox!(
            F,
            "guard_back",
            0.04,
            0.05,
            1.82,
            (-0.45, 1.6, 0),
            M.frame_oak,
            bevel = 0.012
        );
        fbox!(
            F,
            "guard_front",
            0.04,
            0.05,
            1.28,
            (0.45, 1.6, -0.26),
            M.frame_oak,
            bevel = 0.012
        );
        fbox!(
            F,
            "guard_post",
            0.035,
            0.38,
            0.035,
            (0.45, 1.41, 0.38),
            M.frame_oak,
            bevel = 0.008
        );
    }
}

/// The bunk bed.
pub fn bed_bunk() -> Vec<Obj> {
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            post(
                &format!("post{sx}{sz}"),
                sx as f64 * 0.45,
                bunk_end(sz),
                1.9,
                0.06,
                M.frame_oak,
                Cap::Flat,
                0.0,
            );
        }
    }
    bunk("lower", LOWER, false);
    bunk("upper", UPPER, true);
    // the ladder on the right, at the foot end: two rails, rounded, hooked over the upper bunk's rail, and four
    // round rungs
    for z in [0.42, 0.86] {
        fbox!(
            F,
            format!("ladder{}", float_repr(z)),
            0.03,
            1.62,
            0.045,
            (0.465, 0.81, z),
            M.frame_oak,
            bevel = 0.01
        );
        fsweep!(
            F,
            format!("ladder_hook{}", float_repr(z)),
            &[
                (0.465, 1.6, z),
                (0.465, 1.66, z),
                (0.45, 1.69, z),
                (0.43, 1.66, z)
            ],
            0,
            M.frame_oak,
            profile = rounded_rect(0.02, 0.04, 0.008, 1),
            up = (0, 0, 1),
            resolution = 2
        );
    }
    for (i, y) in [0.32, 0.62, 0.92, 1.22].into_iter().enumerate() {
        rod!(
            format!("rung{i}"),
            F.at(0.465, y, 0.42),
            F.at(0.465, y, 0.86),
            0.016,
            material = M.frame_oak,
            verts = 8
        );
    }
    let (z_top, z_foot) = (-0.28, 0.875);
    // Each bunk's duvet turns down for the one who sleeps in it (`slot`: the lower bunk is 0, the upper 1).
    let z_fold_upper = bedding(
        UPPER,
        0.42,
        z_top,
        z_foot,
        0.12,
        M.bedding_teal,
        Bedding {
            seed: 1.0,
            ..Bedding::default()
        },
    );
    let z_fold = bedding(
        LOWER,
        0.42,
        z_top,
        z_foot,
        0.12,
        M.bedding_teal,
        Bedding {
            seed: 2.0,
            ..Bedding::default()
        },
    );
    join(&meshes(), "bedBunk", Some(P3::ZERO));
    let lower = flap(LOWER, 0.42, z_top, z_fold, M.bedding_teal);
    let upper = flap(UPPER, 0.42, z_top, z_fold_upper, M.bedding_teal);
    upper.set_name("duvet_flap_upper");
    settle_bed();
    turn_down(lower, "turn_down", 0.42, 1);
    turn_down(upper, "turn_down_upper", 0.42, 1);
    meshes()
}

// ---------------------------------------------------------------- the futon

const FUTON_TOP: f64 = 0.4; // BEDS['bedFuton']
const FUTON_HINGE: f64 = -0.2; // the depth where the back folds up from the seat
const FUTON_BACK: f64 = 66.0; // the back's lean at rest, degrees up from flat
const FUTON_TUFTS: [f64; 4] = [-0.66, -0.22, 0.22, 0.66];

/// The futon.
pub fn bed_futon() -> Vec<Obj> {
    let top = FUTON_TOP;
    // a low ash platform: a frame round slats, on a recessed plinth
    fbox!(F, "plinth", 1.8, 0.04, 1.8, (0, 0.02, 0), M.frame_ash);
    // the frame's head end stands on the wall's face; rounded solid ash, the ends' tenons showing through the sides
    for s in [-1, 1] {
        fbox!(
            F,
            format!("side{s}"),
            0.06,
            0.16,
            0.98 - WALL,
            (s as f64 * 0.95, 0.12, (WALL + 0.98) / 2.0),
            M.frame_ash,
            bevel = 0.014,
            bevel_segments = 2
        );
        fbox!(
            F,
            format!("end{s}"),
            1.84,
            0.16,
            0.06,
            (0, 0.12, if s > 0 { 0.95 } else { WALL + 0.03 }),
            M.frame_ash,
            bevel = 0.014,
            bevel_segments = 2
        );
        for z in [0.95, WALL + 0.03] {
            fbox!(
                F,
                format!("tenon{s}{}", float_repr(z)),
                0.012,
                0.09,
                0.035,
                (s as f64 * 0.983, 0.12, z),
                M.frame_ash,
                bevel = 0.003
            );
        }
    }
    for i in 0..14 {
        fbox!(
            F,
            format!("slat{i}"),
            1.84,
            0.025,
            0.08,
            (0, 0.1875, -0.86 + i as f64 * 0.132),
            M.frame_ash,
            bevel = 0.006
        );
    }
    // a low headboard with four slots through it, under a rounded cap rail
    let holes: Vec<Vec<(f64, f64)>> = [-0.6, -0.2, 0.2, 0.6]
        .iter()
        .map(|&hx| {
            let (hy, hw, hh) = (0.43, 0.06, 0.26);
            rounded_rect(hw, hh, 0.03, 3)
                .iter()
                .map(|&(u, v)| (hx + u, hy - 0.43 + v))
                .collect()
        })
        .collect();
    fextrude!(
        F,
        "headboard",
        &rounded_rect(1.96, 0.46, 0.02, 2),
        0.04,
        (0, 0.43, WALL + 0.02),
        M.headboard_ash,
        holes = holes,
        bevel = 0.006,
        bevel_segments = 1
    );
    soft!(
        F,
        "headboard_cap",
        1.96,
        0.035,
        0.05,
        (0, 0.665, WALL + 0.025),
        M.headboard_ash,
        round = 0.014
    );
    // the seat: the front of the futon, a tufted mattress
    mattress(
        "seat",
        1.86,
        top - 0.2,
        0.93 - FUTON_HINGE,
        top,
        (FUTON_HINGE + 0.93) / 2.0,
        Mattress {
            material: Some(M.bedding_futon),
            puff: 0.008,
            piping: 0.008,
            round: 0.06,
            ..Mattress::default()
        },
    );
    tufts(
        "tuft",
        &FUTON_TUFTS,
        &[0.05, 0.42, 0.79],
        top - 0.004,
        M.bedding_futon,
    );
    let q = soft!(
        F,
        "quilt",
        1.5,
        0.05,
        0.32,
        (0, top + 0.02, 0.7),
        M.quilt_oat,
        round = 0.022,
        puff = 0.008,
        flat = 3
    );
    jitter!(q, 0.004, size = 0.12, seed = 3, up = 0.3);
    join(&meshes(), "futon", Some(P3::ZERO));
    // the back, built flat (the bed's head end) with the pillows on it, hinged on the seat's back edge
    let parts = new_parts(|| {
        let z0 = WALL + 0.04; // in front of the headboard
        mattress(
            "back",
            1.86,
            top - 0.2,
            FUTON_HINGE - z0 - 0.01,
            top,
            (z0 + FUTON_HINGE - 0.01) / 2.0,
            Mattress {
                material: Some(M.bedding_futon),
                puff: 0.008,
                piping: 0.008,
                round: 0.06,
                ..Mattress::default()
            },
        );
        tufts(
            "back_tuft",
            &FUTON_TUFTS,
            &[-0.69, -0.42],
            top - 0.004,
            M.bedding_futon,
        );
        for s in [-1, 1] {
            pillow(
                &format!("pillow{s}"),
                0.66,
                0.13,
                0.34,
                s as f64 * 0.45,
                top,
                -0.71,
                4.0,
                None,
                0.0,
                false,
            );
        }
    });
    let back = hinged(join(&parts, "futon_back", None), F.at(0, 0.3, FUTON_HINGE));
    // Built flat; leaning up is the rest pose (the clip's first key): its head end up, its top to the front.
    turn(back, FUTON_BACK, Axis::X);
    settle_bed();
    swing!(
        back,
        "unfold",
        -FUTON_BACK,
        seconds = SECONDS,
        axis = Axis::X
    );
    meshes()
}

// ---------------------------------------------------------------- the upholstered king

/// The upholstered king.
pub fn bed_king() -> Vec<Obj> {
    let top = 0.59; // BEDS['bedKing']
    legs_square(&[-0.9, 0.9], &[-0.8, 0.9], 0.1, M.legs_teak, 0.055);
    soft!(
        F,
        "base",
        1.96,
        0.3,
        1.82,
        (0, 0.25, 0.06),
        M.frame_linen,
        round = 0.05,
        piping = 0.008
    );
    // a tall padded headboard, piped, with five channels
    soft!(
        F,
        "headboard",
        1.98,
        1.36,
        0.04,
        (0, 0.72, WALL + 0.02),
        M.headboard_linen,
        round = 0.018,
        piping = 0.006
    );
    for i in 0..5 {
        channel(
            &format!("channel{i}"),
            -0.74 + i as f64 * 0.37,
            0.36,
            0.45,
            1.33,
            -0.91,
            0.04,
            M.headboard_linen,
            false,
        );
    }
    // wings each side, turned in a little round the pillows, their backs on the wall's face: piped too
    for s in [-1, 1] {
        soft!(
            F,
            format!("wing{s}"),
            0.08,
            1.3,
            0.2,
            (s as f64 * 0.935, 0.69, -0.823),
            M.headboard_linen,
            rot = (0, 0, s * 14),
            round = 0.03,
            puff = 0.008,
            flat = 2,
            piping = 0.006
        );
    }
    mattress(
        "mattress",
        1.86,
        0.22,
        1.8,
        top,
        0.04,
        Mattress {
            round: 0.045,
            ..Mattress::default()
        },
    );
    for s in [-1, 1] {
        pillow_at(
            &format!("pillow{s}"),
            0.74,
            0.16,
            0.4,
            s as f64 * 0.45,
            top,
            -0.66,
        );
    }
    let (z_top, z_foot) = (-0.4, 0.945);
    let z_fold = bedding(
        top,
        0.93,
        z_top,
        z_foot,
        0.18,
        M.bedding_cream,
        Bedding {
            runner: Some((0.75 - 0.19, 0.75 + 0.19)),
            ..Bedding::default()
        },
    );
    // (just round the duvet's sides: its ends stay under the edges of the quilts over sleepers)
    runner(
        "runner",
        0.959,
        top + DUVET + 0.015,
        0.75,
        0.38,
        0.21,
        M.throw_sage,
    );
    finish("bedKing", (top, 0.93, z_top, z_fold, M.bedding_cream), 2)
}

// ---------------------------------------------------------------- the oak double

const DOUBLE_TOP: f64 = 0.52; // the old part's mattress top (furniture.json beds.bedDouble)

/// The oak double.
pub fn bed_double() -> Vec<Obj> {
    let top = DOUBLE_TOP;
    legs_square(&[-0.75, 0.75], &[-0.84, 0.9], 0.1, M.frame_oak, 0.06);
    fbox!(
        F,
        "frame",
        1.62,
        0.22,
        1.84,
        (0, 0.21, 0.04),
        M.frame_oak,
        bevel = 0.014,
        bevel_segments = 2
    );
    // a bead along the frame's top edge
    fbox!(
        F,
        "frame_bead",
        1.632,
        0.016,
        1.852,
        (0, 0.29, 0.04),
        M.frame_oak,
        bevel = 0.006
    );
    // headboard: oak posts and an arched, overhanging top rail round teak slats, their backs on the wall's face
    let head = WALL + 0.035;
    for s in [-1, 1] {
        post(
            &format!("post{s}"),
            s as f64 * 0.775,
            head,
            0.95,
            0.07,
            M.frame_oak,
            Cap::Pyramid,
            0.0,
        );
    }
    board(
        "head_top",
        1.5,
        0.87,
        0.95,
        head,
        0.07,
        M.frame_oak,
        Board {
            arch: 0.03,
            bevel: 0.012,
            ..Board::default()
        },
    );
    fbox!(
        F,
        "head_low",
        1.48,
        0.06,
        0.05,
        (0, 0.35, head),
        M.frame_oak,
        bevel = 0.01
    );
    for i in 0..8 {
        let x = -0.69 + i as f64 * 0.197;
        let y1 = 0.89 + 0.03 * cos(PI * x / 1.5);
        soft!(
            F,
            format!("slat{i}"),
            0.1,
            y1 - 0.37,
            0.025,
            (x, (0.37 + y1) / 2.0, head),
            M.headboard_teak,
            round = 0.01
        );
    }
    mattress("mattress", 1.56, 0.2, 1.79, top, 0.025, Mattress::default());
    for s in [-1, 1] {
        pillow_at(
            &format!("pillow{s}"),
            0.69,
            0.14,
            0.34,
            s as f64 * 0.405,
            top,
            -0.68,
        );
    }
    let (z_top, z_foot) = (-0.36, 0.92);
    let z_fold = bedding(
        top,
        0.78,
        z_top,
        z_foot,
        0.2,
        M.bedding_blue,
        Bedding::default(),
    );
    finish("bedDouble", (top, 0.78, z_top, z_fold, M.bedding_blue), 2)
}

// ---------------------------------------------------------------- the upholstered queen

/// The upholstered queen.
pub fn bed_queen_upholstered() -> Vec<Obj> {
    let top = DOUBLE_TOP;
    legs_square(&[-0.8, 0.8], &[-0.84, 0.9], 0.08, M.legs_teak, 0.05);
    soft!(
        F,
        "base",
        1.72,
        0.24,
        1.82,
        (0, 0.2, 0.05),
        M.frame_stone,
        round = 0.03,
        piping = 0.007
    );
    // a tall headboard of five domed channels, its top scalloped round them
    let xs: Vec<f64> = (0..5).map(|i| -0.64 + i as f64 * 0.32).collect();
    let mut top_edge = Vec::new();
    for &x in xs.iter().rev() {
        top_edge.extend((0..181).step_by(20).map(|a| {
            let a = radians(a as f64);
            (x + 0.16 * cos(a), 1.14 + 0.16 * sin(a))
        }));
    }
    let mut outline = vec![(-0.86, 0.02), (0.86, 0.02), (0.86, 1.14)];
    outline.extend(top_edge);
    outline.push((-0.86, 1.14));
    fextrude!(
        F,
        "headboard",
        &outline,
        0.04,
        (0, 0, WALL + 0.02),
        M.headboard_stone,
        bevel = 0.012,
        bevel_segments = 2
    );
    for (i, &x) in xs.iter().enumerate() {
        channel(
            &format!("channel{i}"),
            x,
            0.3,
            0.42,
            1.28,
            -0.91,
            0.04,
            M.headboard_stone,
            true,
        );
    }
    mattress("mattress", 1.66, 0.2, 1.78, top, 0.03, Mattress::default());
    for s in [-1, 1] {
        pillow_at(
            &format!("pillow{s}"),
            0.74,
            0.15,
            0.34,
            s as f64 * 0.43,
            top,
            -0.68,
        );
    }
    let (z_top, z_foot) = (-0.36, 0.935);
    let z_fold = bedding(
        top,
        0.855,
        z_top,
        z_foot,
        0.2,
        M.bedding_oat,
        Bedding::default(),
    );
    finish(
        "bedQueenUpholstered",
        (top, 0.855, z_top, z_fold, M.bedding_oat),
        2,
    )
}

// ---------------------------------------------------------------- the four-poster

/// A ring of `n` points (Blender x, y about 0) round a gathered curtain's cross-section: `width` along the bed's side
/// (Blender y), `depth` across it, in `folds` soft pleats.
fn folds(k: f64, n: usize, depth: f64, width: f64, folds: i32) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let a = TAU * i as f64 / n as f64;
            let r = 1.0 + k * sin(folds as f64 * a);
            (depth / 2.0 * cos(a) * r, width / 2.0 * sin(a) * r)
        })
        .collect()
}

/// A curtain gathered at a post, hanging from the top rail down the side: pleated, wide at the rail, drawn in by a
/// tie-back a metre up and spreading again to the floor. `inward`: the depth direction along the side, away from
/// the post.
fn drape(name: &str, x: f64, z: f64, inward: i32) {
    let b = F.at(x, 0, z);
    let (bx, by) = (b.x, b.y);
    let shift = -inward as f64 * 0.05; // Blender y: its middle a little along the side from the post
    let secs: Vec<Vec<P3>> = [
        (0.12, 0.16, 0.06),
        (0.75, 0.11, 0.05),
        (1.0, 0.06, 0.045),
        (1.25, 0.1, 0.045),
        (1.6, 0.14, 0.04),
        (2.05, 0.16, 0.03), // its head in the top rail, where it hangs from
    ]
    .iter()
    .map(|&(y, width, depth)| {
        folds(0.1, 20, depth, width, 5)
            .iter()
            .map(|&(u, v)| P3 {
                x: bx + u,
                y: by + shift + v,
                z: y,
            })
            .collect()
    })
    .collect();
    // (gathered cloth turns round at its edges: shaded so, and laid flat on its folds, not round its middle)
    let c = loft!(
        format!("{name}_curtain"),
        &secs,
        material = M.canopy,
        angle = 150
    );
    c.edit(|p| p.uv = artkit::mesh::UvWay::Box);
    let tie: Vec<(f64, f64, f64)> = (0..10)
        .map(|i| {
            (
                x + 0.03 * cos(TAU * i as f64 / 10.0),
                1.0,
                z + inward as f64 * 0.05 + 0.04 * sin(TAU * i as f64 / 10.0),
            )
        })
        .collect();
    fsweep!(
        F,
        format!("{name}_tie"),
        &tie,
        0.009,
        M.canopy,
        verts = 4,
        closed = true
    );
}

/// A turned post `h` tall, about `r` thick: a square-ish foot block below, then a vase, rings and a long shaft up to
/// a finial.
fn turned_post(name: &str, x: f64, z: f64, h: f64, r: f64, material: artkit::materials::Mat) {
    let prof = [
        (0.0, 0.12),
        (r * 1.15, 0.12),
        (r * 0.8, 0.16),
        (r * 1.15, 0.25),
        (r * 1.25, 0.33),
        (r * 0.85, 0.42),
        (r * 0.65, 0.46),
        (r * 1.0, 0.49),
        (r * 1.0, 0.52),
        (r * 0.62, 0.55),
        (r * 0.7, 1.05),
        (r * 0.55, h - 0.18),
        (r * 0.9, h - 0.155),
        (r * 0.9, h - 0.135),
        (r * 0.5, h - 0.115),
        (r * 0.5, h - 0.075),
        (r * 0.85, h - 0.045),
        (r * 0.6, h - 0.012),
        (0.0, h),
    ];
    revolve!(
        name,
        &prof,
        at = F.at(x, 0, z),
        material = material,
        segments = 10
    );
    fbox!(
        F,
        format!("{name}_foot"),
        2.0 * r * 1.28,
        0.12,
        2.0 * r * 1.28,
        (x, 0.06, z),
        material,
        bevel = 0.012,
        bevel_segments = 1
    );
}

/// A valance's outline `w` across and `h` deep, hanging from its top (0) in `n` scallops.
fn scallops(w: f64, h: f64, n: usize) -> Vec<(f64, f64)> {
    let mut bottom: Vec<(f64, f64)> = (0..=n * 4)
        .map(|i| {
            let u = w / 2.0 - w * i as f64 / (n * 4) as f64;
            (u, -h + 0.04 * sin(PI * i as f64 / 4.0).abs())
        })
        .collect();
    bottom.reverse();
    let mut out = vec![(w / 2.0, 0.0), (-w / 2.0, 0.0)];
    out.extend(bottom);
    out
}

/// The four-poster.
pub fn bed_canopy() -> Vec<Obj> {
    let top = DOUBLE_TOP;
    // the posts' depths: the head end's feet, crest and valance stand on the wall's face
    let posts = |s: i32| if s < 0 { WALL + 0.045 } else { 0.92 };
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            turned_post(
                &format!("post{sx}{sz}"),
                sx as f64 * 0.82,
                posts(sz),
                2.135,
                0.035,
                M.frame_walnut,
            );
        }
        fbox!(
            F,
            format!("rail_top{sx}"),
            0.06,
            0.06,
            1.77,
            (sx as f64 * 0.82, 2.06, 0),
            M.frame_walnut,
            bevel = 0.012,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("rail_end{sx}"),
            1.57,
            0.06,
            0.06,
            (0, 2.06, posts(sx)),
            M.frame_walnut,
            bevel = 0.012,
            bevel_segments = 1
        );
    }
    fbox!(
        F,
        "frame",
        1.6,
        0.22,
        1.78,
        (0, 0.21, 0),
        M.frame_walnut,
        bevel = 0.016,
        bevel_segments = 2
    );
    fbox!(
        F,
        "frame_bead",
        1.612,
        0.016,
        1.792,
        (0, 0.29, 0),
        M.frame_walnut,
        bevel = 0.006
    );
    // a headboard of three raised panels in a frame, under an arched crest rail
    let holes: Vec<Vec<(f64, f64)>> = [-0.52, 0.0, 0.52]
        .iter()
        .map(|&x| {
            rounded_rect(0.42, 0.4, 0.02, 2)
                .iter()
                .map(|&(u, v)| (x + u, v + 0.02))
                .collect()
        })
        .collect();
    fextrude!(
        F,
        "headboard",
        &rounded_rect(1.57, 0.6, 0.004, 1),
        0.04,
        (0, 0.68, WALL + 0.02),
        M.headboard_walnut,
        holes = holes,
        bevel = 0.006,
        bevel_segments = 1
    );
    for (i, x) in [N::F(-0.52), N::I(0), N::F(0.52)].into_iter().enumerate() {
        fbox!(
            F,
            format!("panel{i}"),
            0.418,
            0.398,
            0.024,
            (x, 0.7, WALL + 0.02),
            M.headboard_walnut,
            bevel = 0.009,
            bevel_segments = 1
        );
    }
    board(
        "crest",
        1.64,
        0.965,
        1.1,
        WALL + 0.035,
        0.07,
        M.headboard_walnut,
        Board {
            arch: 0.065,
            bevel: 0.012,
            n: 12,
            ..Board::default()
        },
    );
    // the canopy: a scalloped linen valance round the open top (sleepers stay in view from above), a curtain
    // gathered at each post
    for s in [-1, 1] {
        fextrude!(
            F,
            format!("valance_side{s}"),
            &scallops(1.77, 0.2, 5),
            0.008, // lined linen, not a board
            (s as f64 * 0.85, 2.09, 0),
            M.canopy,
            plane = artkit::place::Facing::Side
        );
        fextrude!(
            F,
            format!("valance_end{s}"),
            &scallops(1.57, 0.2, 4),
            0.008,
            (0, 2.09, if s > 0 { 0.95 } else { WALL + 0.006 }),
            M.canopy
        );
    }
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            drape(
                &format!("drape{sx}{sz}"),
                sx as f64 * 0.83,
                sz as f64 * 0.8,
                -sz,
            );
        }
    }
    mattress("mattress", 1.56, 0.2, 1.76, top, 0, Mattress::default());
    for s in [-1, 1] {
        pillow_at(
            &format!("pillow{s}"),
            0.69,
            0.14,
            0.34,
            s as f64 * 0.405,
            top,
            -0.68,
        );
    }
    // (the duvet's foot 1.5 cm past the mattress's end, not in its plane)
    let (z_top, z_foot) = (-0.36, 0.87);
    let z_fold = bedding(
        top,
        0.775,
        z_top,
        z_foot,
        0.2,
        M.bedding_rust,
        Bedding::default(),
    );
    finish("bedCanopy", (top, 0.775, z_top, z_fold, M.bedding_rust), 2)
}

// ---------------------------------------------------------------- the daybed

const DAYBED_TOP: f64 = 0.51; // the old part's mattress top (furniture.json beds.bedDaybed)

/// A turned spindle from y0 up to y1: a bead at each end, swelling in the middle.
fn spindle(name: &str, x: f64, y0: f64, y1: f64, z: f64, material: artkit::materials::Mat) -> Obj {
    let r = 0.014;
    let h = y1 - y0;
    let prof = [
        (0.0, 0.0),
        (r * 1.1, 0.0),
        (r * 1.15, 0.03),
        (r * 0.7, 0.06),
        (r * 1.0, h * 0.5),
        (r * 0.7, h - 0.06),
        (r * 1.15, h - 0.03),
        (r * 1.1, h),
        (0.0, h),
    ];
    revolve!(
        name,
        &prof,
        at = F.at(x, y0, z),
        material = material,
        segments = 6
    )
}

/// The daybed.
pub fn bed_daybed() -> Vec<Obj> {
    let top = DAYBED_TOP;
    // the ends' depths: the head end's posts and arm stand on the wall's face, the base and the mattress between
    // the arms
    let ends = |s: i32| if s < 0 { WALL + 0.03 } else { 0.95 };
    let (z0, z1) = (ends(-1) + 0.03, 0.92);
    // four posts, the back's taller, each with a ball on top; a base between them
    for sx in [-1, 1] {
        let h = if sx < 0 { 0.84 } else { 0.74 };
        for sz in [-1, 1] {
            post(
                &format!("post{sx}{sz}"),
                sx as f64 * 0.45,
                ends(sz),
                h,
                0.06,
                M.frame_white,
                Cap::Ball,
                0.0,
            );
        }
    }
    fbox!(
        F,
        "base",
        0.9,
        0.23,
        z1 - z0,
        (0, 0.215, (z0 + z1) / 2.0),
        M.frame_white,
        bevel = 0.014,
        bevel_segments = 2
    );
    fbox!(
        F,
        "base_bead",
        0.912,
        0.016,
        z1 - z0 + 0.012,
        (0, 0.11, (z0 + z1) / 2.0),
        M.frame_white,
        bevel = 0.006
    );
    // the back along the left side: a rounded top rail over turned spindles (the headboard)
    // (its ends stand above the posts' shafts: chamfered)
    let rail: Vec<(f64, f64)> = rounded_rect(0.05, 0.06, 0.018, 2)
        .iter()
        .map(|&(u, v)| (u + 0.79, v))
        .collect();
    eased_prism(
        &F,
        "back_rail",
        &rail,
        -0.45,
        ends(-1) + 0.03,
        0.92,
        0.004,
        M.back_white,
    );
    fbox!(
        F,
        "back_low",
        0.05,
        0.03,
        0.93 - ends(-1) - 0.06,
        (-0.45, 0.345, (ends(-1) + 0.93) / 2.0),
        M.back_white,
        bevel = 0.008
    );
    for i in 0..15 {
        spindle(
            &format!("back_spindle{i}"),
            -0.45,
            0.33,
            0.8,
            -0.84 + i as f64 * 0.12,
            M.back_white,
        );
    }
    // an arm at each end, the same
    for s in [-1, 1] {
        fsweep!(
            F,
            format!("arm_rail{s}"),
            &[(-0.42, 0.665, ends(s)), (0.42, 0.665, ends(s))],
            0,
            M.frame_white,
            profile = rounded_rect(0.05, 0.06, 0.018, 2),
            up = (0, 0, 1)
        );
        fbox!(
            F,
            format!("arm_low{s}"),
            0.86,
            0.06,
            0.05,
            (0, 0.3, ends(s)),
            M.frame_white,
            bevel = 0.01
        );
        for i in 0..5 {
            spindle(
                &format!("arm_spindle{s}{i}"),
                -0.3 + i as f64 * 0.15,
                0.32,
                0.645,
                ends(s),
                M.frame_white,
            );
        }
    }
    // the mattress in a fitted cover (the bedding), two back cushions clear of a sleeper, a pillow
    mattress(
        "mattress",
        0.86,
        top - 0.33,
        z1 - z0,
        top,
        (z0 + z1) / 2.0,
        Mattress {
            x: 0.02,
            material: Some(M.bedding_sage),
            round: 0.035,
            ..Mattress::default()
        },
    );
    // (thin and close to the back, leaning on it, so a sleeper's shoulder and hip stay clear of them)
    for (i, z) in [-0.32, 0.3].into_iter().enumerate() {
        let c = soft!(
            F,
            format!("cushion{i}"),
            0.08,
            0.34,
            0.56,
            (-0.385, top + 0.16, z),
            M.cushion_rose,
            rot = (0, -8, 0),
            round = 0.03,
            puff = 0.016,
            flat = 3,
            piping = 0.006
        );
        jitter!(c, 0.003, size = 0.1, seed = i as f64, up = 0.3);
    }
    pillow_at("pillow", 0.56, 0.13, 0.3, 0.06, top, -0.72);
    join_all("bedDaybed");
    settle_bed();
    meshes()
}
