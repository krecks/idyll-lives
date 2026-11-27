//! The daybed.

use super::common::{Cap, F, M, Mattress, WALL, mattress, pillow_at, post, settle_bed};
use artkit::forms::rounded_rect;
use artkit::parts::join_all;
use artkit::place::eased_prism;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsweep, jitter, revolve, soft};

// ---------------------------------------------------------------- the double bed (`bed`)

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
