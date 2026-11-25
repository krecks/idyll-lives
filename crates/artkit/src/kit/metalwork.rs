//! Metalwork: tube bent in one piece, and flat iron curled into scrolls.

use crate::forms::{self, Profile, Scale, Sweep, by, rounded_rect};
use crate::materials::Mat;
use crate::math::{Num, P3, TAU, atan2, cos, sin};
use crate::place::{Coord, GameFrame, Spot};
use crate::py;
use crate::scene::Obj;
use crate::shapes::options;

options! {
    /// `tube`'s options.
    Tube { verts: Option<usize> = None, round: usize = 3, scale: Scale = Scale::One(1.0) }
}

/// A round tube of radius `r` through game points, its bends rounded (`round` steps between each pair of points;
/// 1 keeps them sharp), its ends capped: a towel rail, a bent-tube frame, a tap's riser, a hose. Replaces the rods
/// joined by spheres the families made one by one. `verts` round: by default 8, and more for one over 13 mm in
/// radius, as many as keep its outline within a millimetre of the circle (`forms::sweep`).
pub fn tube<P: Spot>(
    frame: &GameFrame,
    name: impl AsRef<str>,
    points: &[P],
    r: f64,
    material: Mat,
    o: Tube,
) -> Obj {
    frame.sweep(
        name,
        points,
        r,
        material,
        Sweep {
            verts: o.verts,
            resolution: o.round,
            scale: o.scale,
            least: 8,
            ..Sweep::default()
        },
    )
}

options! {
    /// `scroll`'s options.
    Scroll { turns: f64 = 1.1, end: f64 = 0.25, t: f64 = 0.02, w: f64 = 0.03, steps: usize = 10, profile: Profile = Profile(None) }
}

/// A flat iron scroll in the depth-height plane at the game x of `start`: from `start` (x, height, depth) it curls
/// round `centre` (height, depth) by `turns`, its radius closing to `end` of where it began, tapering as it goes:
/// a bench arm's or a bracket's end.
pub fn scroll(
    frame: &GameFrame,
    name: impl AsRef<str>,
    start: (impl Coord, impl Num, impl Num),
    centre: (impl Num, impl Num),
    material: Mat,
    o: Scroll,
) -> Obj {
    let (x, y0, z0) = (start.0, start.1.f(), start.2.f());
    let (cy, cz) = (centre.0.f(), centre.1.f());
    let r0 = py::hypot(y0 - cy, z0 - cz);
    let a0 = atan2(y0 - cy, z0 - cz);
    let sign = if o.turns > 0.0 { 1.0 } else { -1.0 };
    let mut pts: Vec<P3> = Vec::with_capacity(o.steps + 1);
    for i in 0..=o.steps {
        let k = i as f64 / o.steps as f64;
        let r = r0 * (1.0 - (1.0 - o.end) * k);
        let a = a0 + sign * TAU * o.turns.abs() * k;
        pts.push(frame.at(x, cy + r * sin(a), cz + r * cos(a)));
    }
    let prof = match o.profile.0 {
        Some(p) if !p.is_empty() => p,
        _ => rounded_rect(o.t, o.w, py::min_of(&[o.t, o.w]) * 0.3, 1),
    };
    forms::sweep(
        name,
        &pts,
        0.0,
        Sweep {
            material: Some(material),
            profile: Profile(Some(prof)),
            up: P3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            scale: by(|f| 1.0 - 0.45 * f),
            ..Sweep::default()
        },
    )
}
