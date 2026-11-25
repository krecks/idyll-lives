//! A steam iron, the same in the hand and on the ironing board.

use super::fillet;
use crate::forms::rounded_rect;
use crate::materials::Mat;
use crate::math::{PI, cos, sin};
use crate::place::{Facing, GameFrame};
use crate::{fbox, fcyl, fextrude, fsweep};

/// A steam iron 26 cm long standing on its soleplate at game `at` (its middle 4 cm behind the point's curve), its
/// point towards `toward`, a quarter turn in game (x, depth): (0, 1) to the front, (-1, 0) to the left. The
/// soleplate (`plate`), the shell over it and the handle arched over its back half (`body`), the water window and
/// a 3.2 cm temperature dial under the handle (`trim`).
pub fn steam_iron(
    frame: &GameFrame,
    at: (f64, f64, f64),
    toward: (f64, f64),
    plate: Mat,
    body: Mat,
    trim: Mat,
) {
    let (x, y, z) = at;
    let (tx, tz) = toward;
    let (ax, az) = (tz, -tx); // across it, to its right as it points
    // (a: across, b: along to the point) as game (x, depth)
    let place = |a: f64, b: f64| (x + a * ax + b * tx, z + a * az + b * tz);
    // the soleplate: square at the heel, curving to the point
    let mut sole: Vec<(f64, f64)> = vec![(-0.055, -0.1), (0.055, -0.1)];
    for k in 0..=10 {
        let t = PI / 2.0 * (1.0 - 2.0 * k as f64 / 10.0);
        sole.push((0.055 * sin(t), 0.04 + 0.12 * cos(t).powf(1.6)));
    }
    let outline: Vec<(f64, f64)> = sole.iter().map(|&(a, b)| place(a, b)).collect();
    fextrude!(
        frame,
        "iron_sole",
        &outline,
        0.012,
        (0, y, 0),
        plate,
        plane = Facing::Top,
        bevel = 0.003,
        bevel_segments = 1
    );
    let shell: Vec<(f64, f64)> = sole
        .iter()
        .map(|&(a, b)| place(a * 0.9, b * 0.92 - 0.004))
        .collect();
    fextrude!(
        frame,
        "iron_shell",
        &shell,
        0.05,
        (0, y + 0.012, 0),
        body,
        plane = Facing::Top,
        bevel = 0.016,
        bevel_segments = 2
    );
    let handle: Vec<(f64, f64, f64)> = [(0.06, -0.085), (0.125, -0.07), (0.125, 0.08), (0.06, 0.1)]
        .iter()
        .map(|&(h, b)| {
            let (px, pz) = place(0.0, b);
            (px, y + h, pz)
        })
        .collect();
    fsweep!(
        frame,
        "iron_handle",
        &fillet(&handle, 0.03, 3),
        0.0,
        body,
        // 2.2 cm across, 3 cm in its arch (its `up` across it; the profile is the same either way round)
        profile = rounded_rect(0.022, 0.03, 0.009, 2),
        up = (ax, az, 0.0)
    );
    let (wx, wz) = place(0.047, 0.04);
    fbox!(
        frame,
        "iron_window",
        0.004 * ax.abs() + 0.07 * tx.abs(),
        0.03,
        0.004 * az.abs() + 0.07 * tz.abs(),
        (wx, y + 0.045, wz),
        trim,
        bevel = 0.002,
        bevel_segments = 1
    );
    let (dx, dz) = place(0.0, -0.035);
    fcyl!(
        frame,
        "iron_dial",
        0.016,
        0.012,
        (dx, y + 0.066, dz),
        trim,
        verts = 12
    );
}
