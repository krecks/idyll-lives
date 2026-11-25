//! L6, the staircases: the closed flight (base manifest `model.stairs`), the spiral and the open-tread flight the
//! player buys.

use super::common::disc;
use super::shelves::eased_bar;
use super::{F, M};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{cos, radians, sin};
use artkit::parts::join_all;
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, settle};

// A flight of 14 steps up one storey (house.ts WALL_HEIGHT 2.8) over its 1 × 4 footprint, climbed from the front
// (+depth) to the back, every step where the old part has it: 0.2 m up and 4/14 m deep, the first starting at the
// footprint's front edge, the last level with the floor above at the back edge. Residents are drawn rising evenly
// along the flight (sim-core `World::sim_height`), from the foot of the first step to the top of the last, as over
// the old part.
const STEPS: i32 = 14;
const RISE: f64 = 2.8 / STEPS as f64; // 0.2
const RUN: f64 = 4.0 / STEPS as f64; // 0.2857
const TREAD: f64 = 0.03; // the oak board on each step, its top the step's top
const NOSING: f64 = 0.02; // how far a tread reaches past the riser under it (not the first: footprint)
const STEP_W: f64 = 0.77; // the steps, between the strings
const STRING_X: f64 = 0.4025; // the closed strings each side (to x ±0.42, inside the tile's wall face)
const STRING_T: f64 = 0.035;
const STRING_UP: f64 = 0.12; // how far the strings stand above the nosings' line
const STRING_DOWN: f64 = 0.15; // and below the steps'
const RAIL: f64 = 0.58; // the handrail's middle above the strings' top (the old part's rail height)

/// A closed string's top edge at depth `z`.
fn string_top(z: f64) -> f64 {
    let slope = RISE / RUN;
    slope * (2.0 - z) + RISE + STRING_UP
}

/// A closed string: a band along the flight, cut level on the floor in front and the floor above at the back, from
/// game x0 to x1, its arrises eased.
fn string(name: &str, x0: f64, x1: f64, material: Mat) -> Obj {
    let slope = RISE / RUN;
    let bottom = |z: f64| slope * (2.0 - z) - STRING_DOWN;
    let z_top = 2.0 - (2.8 - RISE - STRING_UP) / slope; // where the top edge reaches the floor above
    let z_foot = 2.0 - STRING_DOWN / slope; // where the bottom edge meets the floor
    let outline = [
        (2.0, 0.0),
        (2.0, string_top(2.0)),
        (z_top, 2.8),
        (-2.0, 2.8),
        (-2.0, bottom(-2.0)),
        (z_foot, 0.0),
    ];
    eased_bar(name, &outline, (x0, x1), 0.004, material, true)
}

/// The staircase: oak treads with a bullnose nosing over a small cove on white risers, between closed white strings
/// with a moulded capping; a black iron railing each side, a square newel at the foot and the head with a collar and
/// a ball finial, two balusters a step, each with a knuckle at its middle, and a moulded handrail in one piece, where
/// the old part's rails were.
pub fn stairs() -> Vec<Obj> {
    for i in 1..=STEPS {
        let (front, back) = (2.0 - f64::from(i - 1) * RUN, 2.0 - f64::from(i) * RUN);
        let y = f64::from(i) * RISE;
        fbox!(
            F,
            format!("riser{i}"),
            STEP_W,
            y - TREAD,
            RUN,
            (0, (y - TREAD) / 2.0, (front + back) / 2.0),
            M.risers
        );
        let over = if i > 1 { NOSING } else { 0.0 };
        // the nosing rounded over on top (within a centimetre of the tread's top, so the review still counts one step
        // a tread: crates/tools checkglb `steps`), its underside edge eased
        let (nose, r) = (front + over, 0.009);
        let mut outline = vec![
            (back, y - TREAD),
            (nose - 0.004, y - TREAD),
            (nose, y - TREAD + 0.004),
        ];
        outline.extend((0..91).step_by(18).map(|a| {
            let a = radians(f64::from(a));
            (nose - r + r * cos(a), y - r + r * sin(a))
        }));
        outline.extend([(back + 0.004, y), (back, y - 0.004)]); // its back edge eased (the top one shows)
        fextrude!(
            F,
            format!("tread{i}"),
            &outline,
            STEP_W,
            (0, 0, 0),
            M.treads,
            plane = Facing::Side,
            smooth = true,
            angle = 50.0
        );
        if over != 0.0 {
            fsweep!(
                F,
                format!("cove{i}"),
                &[
                    (-STEP_W / 2.0, y - TREAD - 0.006, front + 0.004),
                    (STEP_W / 2.0, y - TREAD - 0.006, front + 0.004)
                ],
                0.008,
                M.risers,
                verts = 6
            );
        }
    }
    let slope = RISE / RUN;
    let (foot_z, head_z) = (1.8, -1.86); // the newels: on the first step and the last
    let z_top = 2.0 - (2.8 - RISE - STRING_UP) / slope; // where the strings' top edge reaches the floor above
    for s in [-1, 1] {
        let x = f64::from(s) * STRING_X;
        string(
            &format!("string{s}"),
            x - STRING_T / 2.0,
            x + STRING_T / 2.0,
            M.risers,
        );
        let top = string_top;
        let rail_y = |z: f64| top(z) + RAIL;
        // the strings' capping, rounded over, along their sloping top edge
        fsweep!(
            F,
            format!("string_cap{s}"),
            &[
                (x, top(1.985) + 0.006, 1.985),
                (x, top(z_top + 0.06) + 0.006, z_top + 0.06)
            ],
            0.0,
            M.risers,
            profile = rounded_rect(0.016, 0.048, 0.007, 2),
            up = (0, 0, 1)
        );
        // the newels: a square post, a collar under a ball finial
        for (k, (z, y0)) in [(foot_z, 0.0), (head_z, 2.8)].into_iter().enumerate() {
            let y1 = rail_y(z) + 0.06;
            fbox!(
                F,
                format!("newel{s}{k}"),
                0.045,
                y1 - y0 - 0.03,
                0.045,
                (x, (y0 + y1 - 0.03) / 2.0, z),
                M.railing,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("newel_collar{s}{k}"),
                0.05,
                0.012,
                0.05,
                (x, y1 - 0.03, z),
                M.railing,
                bevel = 0.004,
                bevel_segments = 1
            );
            disc(
                &format!("newel_finial{s}{k}"),
                &[
                    (y1 - 0.024, 0.012),
                    (y1 - 0.016, 0.008),
                    (y1 - 0.008, 0.018),
                    (y1 + 0.004, 0.024),
                    (y1 + 0.015, 0.016),
                    (y1 + 0.02, 0.004),
                ],
                M.railing,
                x,
                z,
                12,
            );
        }
        // the handrail in one piece between the newels, rounded on top
        fsweep!(
            F,
            format!("handrail{s}"),
            &[(x, rail_y(foot_z), foot_z), (x, rail_y(head_z), head_z)],
            0.0,
            M.railing,
            profile = smooth_outline(
                &[
                    (-0.025, -0.016),
                    (0.018, -0.022),
                    (0.026, 0.0),
                    (0.018, 0.022),
                    (-0.025, 0.016)
                ],
                2,
                true
            ),
            up = (0, 1, 0)
        );
        // two balusters a step, standing on the string, a knuckle half way up each
        for i in 1..=STEPS {
            for f in [0.25, 0.75] {
                let z = 2.0 - (f64::from(i - 1) + f) * RUN;
                if z > foot_z - 0.06 || z < head_z + 0.06 {
                    continue;
                }
                let t = top(z);
                let (y0, y1) = (if 2.8 < t { 2.8 } else { t } - 0.01, rail_y(z) - 0.02);
                let ym = (y0 + y1) / 2.0;
                fsweep!(
                    F,
                    format!("baluster{s}{i}{}", float_repr(f)),
                    &[
                        (x, y0, z),
                        (x, y0 + 0.02, z),
                        (x, ym - 0.03, z),
                        (x, ym, z),
                        (x, ym + 0.03, z),
                        (x, y1, z)
                    ],
                    0.008,
                    M.railing,
                    verts = 6,
                    scale = [1.5, 1.0, 1.0, 1.9, 1.0, 1.0]
                );
            }
        }
    }
    join_all("stairs");
    settle!(centre = false);
    Vec::new()
}

// L6, the two staircases the player buys (`stairsSpiral`, `stairsOpen`). The spiral winds round a newel at the
// middle of its 2 × 2 tiles: sim-core walks the four tiles anticlockwise from the front-left (`spiral_steps`), one
// tile a step, and steps off left of the back-left tile, so the treads wind 315° from the front-left tile to due
// left, where the top tread's outer edge faces -x. The walking line 0.71 m from the newel, at the tiles' corners,
// rises 0.7 m a tile: the four tile centres are 0.35, 1.05, 1.75 and 2.45 m up, and the top tread meets the floor
// above at 2.8 m.
const SPIRAL_TREADS: i32 = 14;
const SPIRAL_FIRST: f64 = 0.35;
const SPIRAL_TOP: f64 = 2.8;
const SPIRAL_TURN: f64 = 315.0; // from the front-left tile (135°) to due left (-180°)
const SPIRAL_R0: f64 = 0.045; // round the newel, inside the 2 × 2 footprint
const SPIRAL_R1: f64 = 0.92;
const SPIRAL_TREAD: f64 = 0.045;
const SPIRAL_RAIL: f64 = 0.92; // the handrail's middle above a tread

/// A tread's outline (game x, depth) between two radii, `a0`..`a1` in degrees round the newel.
fn wedge(r0: f64, r1: f64, a0: f64, a1: f64) -> Vec<(f64, f64)> {
    let segments = 6;
    let angles: Vec<f64> = (0..=segments)
        .map(|i| a0 + (a1 - a0) * f64::from(i) / f64::from(segments))
        .collect();
    let mut out: Vec<(f64, f64)> = angles
        .iter()
        .map(|&a| (r0 * cos(radians(a)), r0 * sin(radians(a))))
        .collect();
    out.extend(
        angles
            .iter()
            .rev()
            .map(|&a| (r1 * cos(radians(a)), r1 * sin(radians(a)))),
    );
    out
}

/// A spiral stair: 14 oak wedge treads round a dark steel newel, winding anticlockwise from the front-left tile to
/// due left, where the way off is; a slim black handrail on the treads' outer ends, a baluster each.
pub fn stairs_spiral() -> Vec<Obj> {
    let step = SPIRAL_TURN / f64::from(SPIRAL_TREADS - 1);
    let rise = (SPIRAL_TOP - SPIRAL_FIRST) / f64::from(SPIRAL_TREADS - 1);
    let half = step / 2.0 + 1.2; // a little overlap, so no gap shows between the boards
    for k in 0..SPIRAL_TREADS {
        let a = 135.0 - f64::from(k) * step;
        let y = SPIRAL_FIRST + f64::from(k) * rise;
        fextrude!(
            F,
            format!("tread{k}"),
            &wedge(SPIRAL_R0, SPIRAL_R1, a - half, a + half),
            SPIRAL_TREAD,
            (0, y - SPIRAL_TREAD / 2.0, 0),
            M.treads,
            plane = Facing::Top,
            bevel = 0.005,
            bevel_segments = 1
        );
    }
    // the newel: a dark steel spine from the floor to the floor above, on a plate, with a cap
    fcyl!(
        F,
        "newel_plate",
        0.105,
        0.03,
        (0, 0.015, 0),
        M.spine,
        verts = 20,
        bevel = 0.006
    );
    fcyl!(
        F,
        "newel",
        0.048,
        2.85,
        (0, 1.425, 0),
        M.spine,
        verts = 16,
        bevel = 0.004
    );
    fcyl!(
        F,
        "newel_cap",
        0.062,
        0.02,
        (0, 2.86, 0),
        M.spine,
        verts = 16,
        bevel = 0.006
    );
    let rail_y = |deg: f64| SPIRAL_FIRST + (135.0 - deg) / 90.0 * 0.7 + SPIRAL_RAIL;
    // the handrail: a slim tube winding with the treads, on a baluster at each tread's outer end
    let rail: Vec<(f64, f64, f64)> = (0..=SPIRAL_TREADS * 3)
        .map(|i| {
            let a = 141.0 - f64::from(i) * (SPIRAL_TURN + 12.0) / f64::from(SPIRAL_TREADS * 3);
            (0.875 * cos(radians(a)), rail_y(a), 0.875 * sin(radians(a)))
        })
        .collect();
    fsweep!(
        F,
        "handrail",
        &rail,
        0.013,
        M.railing,
        verts = 8,
        resolution = 2
    );
    for k in 0..SPIRAL_TREADS {
        let a = radians(135.0 - f64::from(k) * step);
        let (x, z) = (0.862 * cos(a), 0.862 * sin(a));
        let y = SPIRAL_FIRST + f64::from(k) * rise;
        fsweep!(
            F,
            format!("baluster{k}"),
            &[
                (x, y - 0.005, z),
                (x, rail_y(135.0 - f64::from(k) * step) - 0.012, z)
            ],
            0.008,
            M.railing,
            verts = 6
        );
    }
    join_all("stairsSpiral");
    settle!(centre = false);
    Vec::new()
}

// The open-tread flight: the same 14 steps, rise and run as `stairs`, but oak boards on two slim steel plates with
// no risers, and a thin steel handrail on posts: at a glance, not the closed white staircase.
const OPEN_TREAD: f64 = 0.045;
const OPEN_STRING: f64 = 0.02; // the plate's thickness across
const OPEN_STRING_X: f64 = 0.395; // its middle: the plates' outer faces at ±0.405
const OPEN_STRING_D: f64 = 0.17; // how deep the plate hangs under the boards
const OPEN_RAIL: f64 = 0.95; // the handrail's middle above the treads

/// A slim plate along the flight, its top edge under the boards and cut level at both ends.
fn open_string(name: &str, x: f64, material: Mat) -> Obj {
    let slope = RISE / RUN;
    let top = |z: f64| slope * (2.0 - z) - OPEN_TREAD;
    let z_foot = 2.0 - (OPEN_TREAD + OPEN_STRING_D) / slope; // where the underside meets the floor
    let outline = [
        (z_foot, 0.0),
        (z_foot, top(z_foot)),
        (-2.0, top(-2.0)),
        (-2.0, top(-2.0) - OPEN_STRING_D),
    ];
    let (x0, x1) = (x - OPEN_STRING / 2.0, x + OPEN_STRING / 2.0);
    eased_bar(name, &outline, (x0, x1), 0.004, material, true)
}

/// An open-tread flight: 14 oak boards on two slim steel plates, no risers, a thin steel handrail on a post at the
/// foot and every third step.
pub fn stairs_open() -> Vec<Obj> {
    for i in 1..=STEPS {
        let (front, back) = (2.0 - f64::from(i - 1) * RUN, 2.0 - f64::from(i) * RUN);
        let y = f64::from(i) * RISE;
        let over = if i > 1 { NOSING } else { 0.0 };
        fbox!(
            F,
            format!("tread{i}"),
            0.80,
            OPEN_TREAD,
            RUN + over,
            (0, y - OPEN_TREAD / 2.0, (front + over + back) / 2.0),
            M.treads,
            bevel = 0.005,
            bevel_segments = 1
        );
    }
    let rail_y = |z: f64| RISE / RUN * (2.0 - z) + OPEN_RAIL;
    for s in [-1, 1] {
        let x = f64::from(s) * OPEN_STRING_X;
        open_string(&format!("string{s}"), x, M.black_metal);
        fsweep!(
            F,
            format!("handrail{s}"),
            &[(x, rail_y(1.95), 1.95), (x, rail_y(-1.95), -1.95)],
            0.014,
            M.railing,
            verts = 8
        );
        for (k, i) in [0, 3, 6, 9, 12, 14].into_iter().enumerate() {
            let z = if i == 0 {
                1.94
            } else {
                2.0 - (f64::from(i) - 0.5) * RUN
            };
            let y0 = if i == 0 { 0.0 } else { f64::from(i) * RISE };
            fsweep!(
                F,
                format!("post{s}{k}"),
                &[(x, y0, z), (x, rail_y(z), z)],
                0.010,
                M.railing,
                verts = 6
            );
        }
    }
    join_all("stairsOpen");
    settle!(centre = false);
    Vec::new()
}
