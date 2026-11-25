//! A mug: a hollow turned body and a looped handle.

use super::{Turned, turned};
use crate::fsweep;
use crate::materials::Mat;
use crate::math::{cosd, sind};
use crate::place::GameFrame;
use crate::shapes::options;

options! {
    /// `mug`'s options (`segments` round the body, `verts` round the handle).
    Mug { segments: usize = 16, verts: usize = 6 }
}

/// A mug `r` in radius and `h` tall standing on `y`: hollow, its handle out at `turn` degrees (0: +x, 90: to the
/// front).
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn mug(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    turn: f64,
    o: Mug,
) {
    let name = name.as_ref();
    let profile = [
        (0.0, 0.0),
        (0.88 * r, 0.0),
        (r, 0.06 * h),
        (r, h),
        (0.86 * r, h),
        (0.86 * r, 0.15 * h),
        (0.0, 0.15 * h),
    ];
    let segments = o.segments;
    turned(
        frame,
        name,
        x,
        y,
        z,
        &profile,
        material,
        Turned {
            segments,
            ..Turned::default()
        },
    );
    let (c, s) = (cosd(turn), sind(turn));
    let pts: Vec<(f64, f64, f64)> = [
        (r - 0.003, 0.8),
        (r + 0.016, 0.82),
        (r + 0.024, 0.55),
        (r + 0.015, 0.27),
        (r - 0.003, 0.25),
    ]
    .iter()
    .map(|&(d, t)| (x + c * d, y + t * h, z + s * d))
    .collect();
    fsweep!(
        frame,
        format!("{name}_handle"),
        &pts,
        0.0045,
        material,
        verts = o.verts,
        resolution = 2
    );
}

/// `kit::mug` with its options by name: `kit::mug!(frame, name, x, y, z, r, h, material, turn, segments = 14, verts = 5)`.
#[macro_export]
macro_rules! kit_mug {
    ($frame:expr, $name:expr, $x:expr, $y:expr, $z:expr, $r:expr, $h:expr, $material:expr, $turn:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = (&$frame, $name, $x, $y, $z, $r, $h, $material, $turn);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Mug::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::mug(args.0, args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), $crate::forms::num(args.5), $crate::forms::num(args.6), args.7, $crate::forms::num(args.8), o)
    }};
}
