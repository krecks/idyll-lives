//! A throw folded over a seat cushion.

use crate::forms::rounded_rect;
use crate::materials::Mat;
use crate::math::radians;
use crate::place::GameFrame;
use crate::scene::Obj;
use crate::shapes::options;
use crate::{fsweep, jitter};

options! {
    /// `throw`'s options (`turn`: degrees about the upright, clockwise seen from above; `seed`: its wobble).
    Throw { turn: f64 = 0.0, seed: f64 = 0.0 }
}

/// A throw `w` wide folded over a seat cushion at game x `x`: lying on its top (`top`) from depth z0 to its front
/// edge z1, rolling over it and hanging `hang` down its front, with a soft wobble.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn throw(
    frame: &GameFrame,
    name: &str,
    x: f64,
    w: f64,
    top: f64,
    z0: f64,
    z1: f64,
    material: Mat,
    hang: f64,
    o: Throw,
) -> Obj {
    let t = 0.012;
    let y = top + t / 2.0;
    let pts = [
        (x, y, z0),
        (x, y + 0.006, (z0 + z1) / 2.0),
        (x, y + 0.002, z1 - 0.035),
        (x, y - 0.012, z1 - 0.004),
        (x, y - 0.04, z1 + t / 2.0 - 0.004),
        (x, y - hang, z1 + t / 2.0 - 0.006),
    ];
    let obj = fsweep!(
        frame,
        name,
        &pts,
        0.0,
        material,
        profile = rounded_rect(w, t, t / 2.0, 1),
        up = (1, 0, 0),
        resolution = 3
    );
    jitter!(obj, 0.004, size = 0.12, seed = o.seed, up = 0.6);
    if o.turn != 0.0 {
        obj.edit(|p| p.rot.z = radians(-o.turn) as f32);
    }
    obj
}

/// `kit::throw` with its options by name: `kit::throw!(frame, name, x, w, top, z0, z1, material, hang, turn = 4, seed = 3)`.
#[macro_export]
macro_rules! kit_throw {
    ($frame:expr, $name:expr, $x:expr, $w:expr, $top:expr, $z0:expr, $z1:expr, $material:expr, $hang:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = (&$frame, $name, $x, $w, $top, $z0, $z1, $material, $hang);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Throw::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::throw(args.0, args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), $crate::forms::num(args.5), $crate::forms::num(args.6), args.7, $crate::forms::num(args.8), o)
    }};
}
