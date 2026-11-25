//! The weight bench and the dumbbell rack: what the lifter's weights rest on. The lift draws the weight in the hands
//! (content `held`), so the bench's bar (`bar_off`) and a pair of the rack's dumbbells (`pair_off`) leave them.

use super::M;
use super::common::{dumbbell, plate};
use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{P3, cos, radians, sin};
use artkit::parts::{aside, hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, settle, soft, squash, torus};

// --- the weight bench: a padded bench, a rack with the bar at the head end, dumbbells by the feet ----------

/// The bar's middle on the J-hooks.
const BAR: (i32, f64, f64) = (0, 1.02, -0.72);
/// The pad's top (furniture.json `beds.weightBench`).
const PAD_TOP: f64 = 0.44;
/// `bar_off` waits while the lifter sits down on the bench (characters.ts: the `sitDown` clip but its last
/// 0.35 s, 0.98 s), so the bar leaves the hooks as the game starts drawing it in the hands, lying back.
pub const BAR_DELAY: f64 = 0.8;

pub fn weight_bench() -> Vec<Obj> {
    // 2 cm forward of its old part: the rack's back on the wall's face
    let f = GameFrame::new(0.02);
    let tube_p = rounded_rect(0.06, 0.06, 0.008, 1);
    let parts = new_parts(|| {
        // A flat bench, its pad at 0.44 m (a bench-press bench is 0.43–0.45, so the lifter's feet reach the floor
        // either side; review M-fitness-01: 0.50, with leg rollers at its foot where the shins would come down).
        let top = PAD_TOP;
        soft!(
            f,
            "pad",
            0.3,
            0.085,
            1.22,
            (0, top - 0.0425 - 0.004, -0.12),
            M.pads,
            round = 0.025,
            puff = 0.004,
            flat = 2,
            round_segments = 2,
            piping = 0.004
        );
        fbox!(
            f,
            "board",
            0.27,
            0.02,
            1.18,
            (0, top - 0.1, -0.12),
            M.dark,
            bevel = 0.005,
            bevel_segments = 1
        );
        let beam = fsweep!(
            f,
            "beam",
            &[(0, top - 0.14, -0.72), (0, top - 0.14, 0.48)],
            0,
            M.frame,
            profile = rounded_rect(0.06, 0.12, 0.01, 1)
        );
        bevel_edges(beam, 0.003, 1, 50.0, &EdgeOrder::Corners); // its ends' edges
        for z in [0.4, -0.62] {
            fsweep!(
                f,
                format!("post{}", float_repr(z)),
                &[(0, top - 0.11, z), (0, 0.05, z)],
                0,
                M.frame,
                profile = rounded_rect(0.08, 0.08, 0.01, 1),
                up = (1, 0, 0)
            );
            fsweep!(
                f,
                format!("feet{}", float_repr(z)),
                &[(-0.23, 0.025, z), (0.23, 0.025, z)],
                0,
                M.frame,
                profile = rounded_rect(0.05, 0.08, 0.01, 1)
            );
            for s in [-1, 1] {
                soft!(
                    f,
                    format!("cap{}{s}", float_repr(z)),
                    0.04,
                    0.052,
                    0.085,
                    (s as f64 * 0.25, 0.026, z),
                    M.dark,
                    round = 0.012,
                    round_segments = 1
                );
            }
            fbox!(
                f,
                format!("gusset{}", float_repr(z)),
                0.01,
                0.06,
                0.08,
                (0, 0.08, z),
                M.frame
            );
        }
        for s in [-1, 1] {
            fcyl!(
                f,
                format!("pad_bolt{s}"),
                0.008,
                0.006,
                (0, top - 0.09, -0.12 + s as f64 * 0.4),
                M.steel,
                verts = 6
            );
        }
        // the rack: uprights with their peg holes, feet, braces, J-hooks lined in red, a cross brace
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                f,
                format!("upright{s}"),
                &[(sf * 0.36, 0.05, -0.8), (sf * 0.36, 1.2, -0.8)],
                0,
                M.frame,
                profile = &tube_p,
                up = (1, 0, 0)
            );
            fcyl!(
                f,
                format!("upright_cap{s}"),
                0.026,
                0.008,
                (sf * 0.36, 1.2, -0.8),
                M.dark,
                verts = 8,
                bevel = 0.002,
                bevel_segments = 1
            );
            fsweep!(
                f,
                format!("base{s}"),
                &[(sf * 0.36, 0.025, -0.95), (sf * 0.36, 0.025, -0.55)],
                0,
                M.frame,
                profile = rounded_rect(0.05, 0.06, 0.008, 1)
            );
            for z in [-0.94, -0.56] {
                soft!(
                    f,
                    format!("base_cap{s}{}", float_repr(z)),
                    0.064,
                    0.054,
                    0.02,
                    (sf * 0.36, 0.027, z),
                    M.dark,
                    round = 0.008,
                    round_segments = 1
                );
            }
            kit::tube!(
                f,
                format!("brace{s}"),
                &[(sf * 0.36, 0.05, -0.58), (sf * 0.36, 0.45, -0.77)],
                0.015,
                M.frame,
                verts = 8,
                round = 1
            );
            for k in 0..8 {
                fbox!(
                    f,
                    format!("peg_hole{s}{k}"),
                    0.016,
                    0.016,
                    0.002,
                    (sf * 0.36, 0.7 + 0.06 * k as f64, -0.769),
                    M.dark
                );
            }
            // the J-hook: a sleeve round the upright, pinned through, and its hook in red plastic
            fbox!(
                f,
                format!("hook_sleeve{s}"),
                0.07,
                0.1,
                0.07,
                (sf * 0.36, 0.98, -0.8),
                M.frame,
                bevel = 0.004,
                bevel_segments = 1
            );
            fcyl!(
                f,
                format!("hook_pin{s}"),
                0.006,
                0.08,
                (sf * 0.36, 0.965, -0.8),
                M.chrome,
                rot = (0, 90, 0),
                verts = 6
            );
            fextrude!(
                f,
                format!("hook{s}"),
                &[
                    (0.03, 0.04),
                    (0.1, 0.04),
                    (0.12, 0.06),
                    (0.12, 0.1),
                    (0.1, 0.1),
                    (0.1, 0.075),
                    (0.05, 0.075),
                    (0.03, 0.09)
                ],
                0.05,
                (sf * 0.36, 0.93, -0.8),
                M.red,
                plane = Facing::Side,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fsweep!(
            f,
            "rack_brace",
            &[(-0.33, 0.26, -0.8), (0.33, 0.26, -0.8)],
            0,
            M.frame,
            profile = rounded_rect(0.05, 0.05, 0.008, 1)
        );
        // two hex dumbbells on the floor by the feet
        for x in [-0.3, 0.3] {
            dumbbell(
                &f,
                &format!("dumbbell{}", float_repr(x)),
                x,
                0.05,
                0.842,
                0.1,
            );
        }
    });
    join(&parts, "body", Some(P3::ZERO));
    // The bar on the J-hooks, its origin at its middle: hidden while lifted (`bar_off`), the game drawing it
    // in the lifter's hands (content `held`) once they lie down (`BAR_DELAY`).
    let parts = new_parts(|| {
        kit::tube!(
            f,
            "bar",
            &[(-0.3, BAR.1, BAR.2), (0.3, BAR.1, BAR.2)],
            0.0135,
            M.chrome,
            verts = 12,
            round = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fcyl!(
                f,
                format!("knurl{s}"),
                0.0138,
                0.12,
                (sf * 0.16, BAR.1, BAR.2),
                M.frame,
                rot = (0, 90, 0),
                verts = 12
            );
            fcyl!(
                f,
                format!("sleeve{s}"),
                0.017,
                0.16,
                (sf * 0.37, BAR.1, BAR.2),
                M.chrome,
                rot = (0, 90, 0),
                verts = 12,
                bevel = 0.003
            );
            fcyl!(
                f,
                format!("shoulder{s}"),
                0.025,
                0.012,
                (sf * 0.296, BAR.1, BAR.2),
                M.chrome,
                rot = (0, 90, 0),
                verts = 12
            );
            plate(
                &f,
                &format!("plate{s}"),
                sf * 0.41,
                BAR.1,
                BAR.2,
                0.16,
                0.03,
                M.plate,
                Some(M.red),
            );
            torus!(
                format!("collar{s}"),
                0.022,
                0.006,
                at = f.at(sf * 0.38, BAR.1, BAR.2),
                material = M.chrome,
                rot = (0, 90, 0),
                major_segments = 12,
                minor_segments = 4
            );
            kit::tube!(
                f,
                format!("collar_arm{s}"),
                &[
                    (sf * 0.38, BAR.1 + 0.022, BAR.2),
                    (sf * 0.375, BAR.1 + 0.06, BAR.2 + 0.02)
                ],
                0.003,
                M.chrome,
                verts = 4,
                round = 1
            );
        }
    });
    let bar = hinged(join(&parts, "bar", None), f.at(BAR.0, BAR.1, BAR.2));
    settle!(centre = false);
    squash!(bar, "bar_off", 0.001, seconds = 0.2);
    meshes()
}

// --- the dumbbell rack: two tilted shelves of hex dumbbell pairs on A-frames, a rubber mat in front ---------
// The rack stands back on the wall's face and its rubber mat fills the front of the tile, where the lifter steps
// in to stand (characters.ts `STAND_BACK`; review I-fitness-02: they stood on the tile in front, the mat empty
// between them and the rack).
const RACK_SHIFT: f64 = -0.1;
/// The shelves' (height, depth): the lower, then the upper.
const SHELVES: [(f64, f64); 2] = [(0.32, -0.08), (0.56, -0.17)];
/// The pair lifted: the upper shelf's left pair.
const LIFTED: (usize, usize) = (1, 0);

/// `_bell`: the width across the heads of the rack's dumbbell `i` on shelf `k`: pairs, the lower shelf's heavier.
fn bell(k: usize, i: usize) -> f64 {
    0.1 + (i / 2) as f64 * 0.012 + (1 - k as i64) as f64 * 0.024
}

pub fn dumbbells() -> Vec<Obj> {
    // the rack back against the wall's face, the mat in front of it
    let f = GameFrame::new(RACK_SHIFT);
    let tilt = radians(18.0);
    let tray = [
        (-0.075, -0.0125),
        (0.075, -0.0125),
        (0.075, 0.03),
        (0.063, 0.03),
        (0.063, 0.0125),
        (-0.075, 0.0125),
    ];
    let parts = new_parts(|| {
        // the A-frames: each one square tube bent over the top, on a foot rail with rubber caps, a cap on the apex
        for s in [-1, 1] {
            let x = s as f64 * 0.36;
            let frame = kit::fillet(
                &[(x, 0.012, 0.08), (x, 0.7, -0.21), (x, 0.012, -0.3)],
                0.05,
                3,
            );
            fsweep!(
                f,
                format!("frame{s}"),
                &frame,
                0,
                M.frame,
                profile = rounded_rect(0.04, 0.04, 0.008, 1),
                up = (1, 0, 0)
            );
            fsweep!(
                f,
                format!("base{s}"),
                &[(x, 0.015, -0.33), (x, 0.015, 0.11)],
                0,
                M.frame,
                profile = rounded_rect(0.03, 0.05, 0.008, 1)
            );
            for z in [-0.32, 0.1] {
                soft!(
                    f,
                    format!("base_cap{s}{}", float_repr(z)),
                    0.054,
                    0.034,
                    0.02,
                    (x, 0.017, z),
                    M.dark,
                    round = 0.008,
                    round_segments = 1
                );
            }
            soft!(
                f,
                format!("top{s}"),
                0.05,
                0.04,
                0.08,
                (x, 0.69, -0.21),
                M.dark,
                round = 0.012,
                round_segments = 1
            );
        }
        // the shelves: trays tilted to the front with a lip, rubber-lined, end caps; a cross brace low down
        for (k, &(y, z)) in SHELVES.iter().enumerate() {
            let prof: Vec<(f64, f64)> = tray
                .iter()
                .map(|&(d, h)| {
                    (
                        z + d * cos(tilt) + h * sin(tilt),
                        y - d * sin(tilt) + h * cos(tilt),
                    )
                })
                .collect();
            fextrude!(
                f,
                format!("shelf{k}"),
                &prof,
                0.76,
                (0, 0, 0),
                M.frame,
                plane = Facing::Side,
                bevel = 0.003,
                bevel_segments = 1
            );
            let (d, h) = (-0.006, 0.0135);
            fbox!(
                f,
                format!("liner{k}"),
                0.74,
                0.002,
                0.126,
                (
                    0,
                    y - d * sin(tilt) + h * cos(tilt),
                    z + d * cos(tilt) + h * sin(tilt)
                ),
                M.dark,
                rot = (18, 0, 0)
            );
            for s in [-1, 1] {
                fbox!(
                    f,
                    format!("shelf_end{s}{k}"),
                    0.012,
                    0.05,
                    0.15,
                    (s as f64 * 0.384, y + 0.004, z),
                    M.dark,
                    rot = (18, 0, 0),
                    bevel = 0.003,
                    bevel_segments = 1
                );
            }
            for i in 0..4 {
                if (k, i / 2) != LIFTED {
                    let w = bell(k, i);
                    dumbbell(
                        &f,
                        &format!("dumbbell{k}{i}"),
                        -0.27 + i as f64 * 0.18,
                        y + 0.012 + w / 2.0,
                        z,
                        w,
                    );
                }
            }
        }
        fsweep!(
            f,
            "brace",
            &[(-0.34, 0.12, -0.25), (0.34, 0.12, -0.25)],
            0,
            M.frame,
            profile = rounded_rect(0.03, 0.03, 0.006, 1)
        );
    });
    join(&parts, "body", Some(P3::ZERO));
    // the rubber floor mat in front, in tiles: drawn, not the rack's size
    let mat = new_parts(|| {
        let mat = GameFrame::new(0.0);
        fbox!(
            mat,
            "floor_mat",
            0.7,
            0.014,
            0.48,
            (0, 0.007, 0.26),
            M.dark,
            bevel = 0.004
        );
        fbox!(
            mat,
            "mat_seam_x",
            0.69,
            0.001,
            0.003,
            (0, 0.0144, 0.26),
            M.plate
        );
        fbox!(
            mat,
            "mat_seam_z",
            0.003,
            0.001,
            0.47,
            (0, 0.0144, 0.26),
            M.plate
        );
    });
    aside(&mat, "mat");
    // The lifted pair, each its own part with its origin at its middle: hidden while lifted (`pair_off`), the
    // game drawing one in each hand (content `held`).
    let (k, p) = LIFTED;
    let (y, z) = SHELVES[k];
    let mut pair = Vec::new();
    for (i, name) in [(2 * p, "dumbbell_l"), (2 * p + 1, "dumbbell_r")] {
        let w = bell(k, i);
        let at = (-0.27 + i as f64 * 0.18, y + 0.012 + w / 2.0, z);
        let one = new_parts(|| dumbbell(&f, name, at.0, at.1, at.2, w));
        pair.push(hinged(join(&one, name, None), f.at(at.0, at.1, at.2)));
    }
    settle!(centre = false);
    for d in pair {
        squash!(d, "pair_off", 0.001, seconds = 0.2);
    }
    meshes()
}
