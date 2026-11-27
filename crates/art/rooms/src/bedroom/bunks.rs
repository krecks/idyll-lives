//! The bunk bed and the futon.

use super::common::{
    Bedding, Board, Cap, F, M, Mattress, SECONDS, WALL, bedding, board, flap, mattress, pillow,
    post, settle_bed, tufts, turn_down,
};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{Axis, P3};
use artkit::parts::{hinged, join, new_parts, turn};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsweep, jitter, rod, soft, swing};

// ---------------------------------------------------------------- the double bed (`bed`)

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
