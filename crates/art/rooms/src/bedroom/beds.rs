//! The beds.

use super::common::{
    Bedding, Board, Cap, DUVET, F, M, Mattress, WALL, bedding, board, channel, finish, legs_round,
    legs_square, mattress, pillow_at, post, runner,
};
use artkit::forms::rounded_rect;
use artkit::math::{PI, cos, radians, sin};
use artkit::scene::Obj;
use artkit::{fbox, fextrude, soft};

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

pub(super) const DOUBLE_TOP: f64 = 0.52; // the old part's mattress top (furniture.json beds.bedDouble)

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
