//! The yoga mat: rolled up at the back with a block and a bottle; it unrolls to the front.

use super::M;
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{Axis, TAU, cos, sin};
use artkit::parts::{aside, hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, grow, revolve, settle, slide, soft, squash, torus};

/// The mat, flat (the house's floor is drawn 1 cm up): a full 1.8 m mat on a 1 x 2 footprint (review M-fitness-02:
/// it unrolled to 0.9).
const MAT_W: f64 = 0.62;
const MAT_L: f64 = 1.8;
const MAT_T: f64 = 0.016;
/// The roll's radius.
const ROLL_R: f64 = 0.065;
/// The mat's back edge (game depth): on the wall's face.
pub const BACK: f64 = -0.93;

/// `_spiral`: a rolled mat's spiral seen end on (game depth, height about its axis): a band `width` wide winding out
/// from `r0` to `r1` in `turns` turns, as one closed outline.
fn spiral(r0: f64, r1: f64, turns: f64, width: f64, n: usize) -> Vec<(f64, f64)> {
    let (mut outer, mut inner) = (Vec::new(), Vec::new());
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let a = TAU * turns * t;
        let r = r0 + (r1 - r0) * t;
        outer.push(((r + width / 2.0) * cos(a), (r + width / 2.0) * sin(a)));
        inner.push(((r - width / 2.0) * cos(a), (r - width / 2.0) * sin(a)));
    }
    outer.extend(inner.into_iter().rev());
    outer
}

pub fn yoga_mat() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    // a foam block, its edges soft, and a bottle with a loop on its cap: drawn beside the mat, not part of
    // its size (the mat is 0.61 m wide rolled; the block stands 0.41 m out to the side)
    let carried = new_parts(|| {
        soft!(
            f,
            "block",
            0.15,
            0.12,
            0.24,
            (0.41, 0.06, BACK + 0.15),
            M.block,
            round = 0.02,
            round_segments = 2
        );
        revolve!(
            "bottle",
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.035, 0.008),
                (0.035, 0.16),
                (0.03, 0.18),
                (0.02, 0.19),
                (0.02, 0.2),
                (0.0, 0.2)
            ],
            at = f.at(-0.4, 0.0, BACK + 0.13),
            material = M.bottle,
            segments = 14
        );
        fcyl!(
            f,
            "bottle_label",
            0.0358,
            0.06,
            (-0.4, 0.09, BACK + 0.13),
            M.block,
            verts = 14
        );
        fcyl!(
            f,
            "bottle_cap",
            0.022,
            0.035,
            (-0.4, 0.2125, BACK + 0.13),
            M.dark,
            verts = 12,
            bevel = 0.004,
            bevel_segments = 1
        );
        torus!(
            "bottle_loop",
            0.012,
            0.0025,
            at = f.at(-0.4, 0.2305, BACK + 0.13 - 0.018),
            material = M.dark,
            major_segments = 10,
            minor_segments = 4
        );
    });
    aside(&carried, "beside");
    // The roll lies at the back, its origin on the floor under its axis: it shrinks onto the floor as it goes. Its
    // ends show the mat's spiral; two straps round it.
    let roll_z = BACK + ROLL_R;
    let parts = new_parts(|| {
        fcyl!(
            f,
            "roll",
            ROLL_R,
            MAT_W,
            (0, ROLL_R, roll_z),
            M.mat,
            rot = (0, 90, 0),
            verts = 24,
            bevel = 0.004
        );
        fcyl!(
            f,
            "roll_core",
            ROLL_R * 0.2,
            MAT_W + 0.002,
            (0, ROLL_R, roll_z),
            M.dark,
            rot = (0, 90, 0),
            verts = 10
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fextrude!(
                f,
                format!("roll_spiral{s}"),
                &spiral(ROLL_R * 0.2, ROLL_R - 0.006, 3.0, 0.003, 40),
                0.002,
                (sf * (MAT_W / 2.0 + 0.0005), ROLL_R, roll_z),
                M.mat_edge,
                plane = Facing::Side
            );
            fcyl!(
                f,
                format!("strap{s}"),
                ROLL_R + 0.0008,
                0.025,
                (sf * 0.2, ROLL_R, roll_z),
                M.dark,
                rot = (0, 90, 0),
                verts = 24
            );
        }
    });
    let roll = hinged(join(&parts, "roll", None), f.at(0, 0, roll_z));
    // The flat mat, its origin under the roll's axis, built full and squeezed to nothing for rest: inside the roll,
    // out of sight (its back edge showed as a strip). Its corners rounded, alignment lines along it. Its underside is
    // 4 mm up (under the house's floor, drawn 1 cm up) and it is 1 cm narrower than the roll, so squeezed it lies
    // inside the roll, clear of the roll's flat bottom and bevelled ends (on the floor it showed under them).
    let lift = 0.004;
    let parts = new_parts(|| {
        fextrude!(
            f,
            "mat",
            &rounded_rect(MAT_W - 0.01, MAT_L, 0.02, 2),
            MAT_T - lift,
            (0, (MAT_T + lift) / 2.0, BACK + MAT_L / 2.0),
            M.mat,
            plane = Facing::Top
        );
        for x in [-0.2, 0.2] {
            fbox!(
                f,
                format!("mat_line{}", float_repr(x)),
                0.006,
                0.001,
                MAT_L - 0.2,
                (x, MAT_T + 0.0004, BACK + MAT_L / 2.0),
                M.mat_edge
            );
        }
        fbox!(
            f,
            "mat_line_mid",
            MAT_W - 0.2,
            0.001,
            0.006,
            (0, MAT_T + 0.0004, BACK + MAT_L / 2.0),
            M.mat_edge
        );
    });
    let mat = hinged(join(&parts, "mat", None), f.at(0, 0, roll_z));
    mat.set_scale_axis(1, 0.001);
    // on its Model centre: the back on the wall face
    settle!();
    // what is left of the roll at the front end: a curled edge
    let left = 0.12;
    let run = MAT_L - ROLL_R - left * ROLL_R;
    grow!(mat, "unroll", seconds = 1.0, axis = Axis::Y);
    slide!(roll, "unroll", (0, -run, 0), seconds = 1.0);
    squash!(roll, "unroll", (1, left, left), seconds = 1.0);
    meshes()
}
