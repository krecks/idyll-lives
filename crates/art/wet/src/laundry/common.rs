//! What the laundry's things are built from.

use artkit::materials::Mat;
use artkit::math::{P3, V3, pow, tan};
use artkit::place::{Coord, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, jitter, revolve, sphere};

pub const WALL: f64 = -0.43; // the wall's face, game depth
pub const DOOR_Z: f64 = 0.152; // the door ring's middle, game depth
pub const DOOR_Y: f64 = 0.4; // the door's middle, height
pub const DOOR_R: f64 = 0.21; // the ring's radius

/// A box with its edges chamfered (`segs` 1) or rounded (2 or more).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn rbox(
    f: &GameFrame,
    name: impl AsRef<str>,
    w: f64,
    h: f64,
    d: f64,
    at: (impl Coord, impl Coord, impl Coord),
    mat: Mat,
    r: f64,
    segs: usize,
    rot: P3,
) -> Obj {
    fbox!(
        f,
        name,
        w,
        h,
        d,
        at,
        mat,
        bevel = r,
        bevel_segments = segs,
        rot = rot
    )
}

/// A turned part facing the front from game depth `z`: `profile` [(radius, out), ...] revolved about the depth axis.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn front_disc(
    f: &GameFrame,
    name: impl AsRef<str>,
    profile: &[(f64, f64)],
    x: impl Coord,
    y: impl Coord,
    z: impl Coord,
    mat: Mat,
    segments: usize,
) -> Obj {
    revolve!(
        name,
        profile,
        at = f.at(x, y, z),
        material = mat,
        segments = segments,
        rot = (90, 0, 0)
    )
}

/// A crumple of washing: a squashed ball, lumpy (Blender scale).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn lump(
    f: &GameFrame,
    name: impl AsRef<str>,
    x: impl Coord,
    y: impl Coord,
    z: impl Coord,
    r: f64,
    mat: Mat,
    scale: (f64, f64, f64),
    seed: f64,
    turn: f64,
) -> Obj {
    let obj = sphere!(
        name,
        r,
        at = f.at(x, y, z),
        material = mat,
        scale = scale,
        rot = (0.0, turn, 0.0),
        segments = 10,
        rings = 6
    );
    jitter!(obj, r * 0.18, size = r * 0.9, seed = seed)
}

/// A folded towel lying on `y`: a soft slab, its folded edge rolled to the front.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn folded(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    d: f64,
    h: f64,
    mat: Mat,
    turn: f64,
) -> Obj {
    rbox(
        f,
        name,
        w,
        h,
        d,
        (x, y + h / 2.0, z),
        mat,
        py_min(0.02, h * 0.45),
        2,
        P3 {
            x: 0.0,
            y: 0.0,
            z: turn,
        },
    )
}

/// Python's `min` of two: the first unless the second is strictly below it.
pub fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

// KIT: kit.fillet
/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
/// it, on a curve that never overshoots, however uneven the spacing: Catmull-Rom loops on a short leg beside a
/// long one). For handles, bent rails and draped cloth swept with `resolution` 1.
// (3.1415 is the kit's own limit for a straight corner, not pi.)
#[allow(clippy::approx_constant)]
pub fn fillet(points: &[(f64, f64, f64)], radius: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let pts: Vec<V3> = points.iter().map(|&p| V3::from(p)).collect();
    let mut out = vec![pts[0]];
    for w in pts.windows(3) {
        let (a, p, c) = (w[0], w[1], w[2]);
        let (d1, d2) = (a - p, c - p);
        if d1.length() < 1e-9 || d2.length() < 1e-9 || d1.angle(d2) > 3.1415 {
            out.push(p);
            continue;
        }
        let t = py_min(
            py_min(
                radius / py_max(1e-6, tan(d1.angle(d2) / 2.0)),
                d1.length() * 0.5,
            ),
            d2.length() * 0.5,
        );
        let (p1, p2) = (p + d1.normalized() * t, p + d2.normalized() * t);
        for i in 0..=steps {
            let k = i as f64 / steps as f64;
            out.push(p1 * pow(1.0 - k, 2.0) + p * 2 * k * (1.0 - k) + p2 * k * k);
        }
    }
    out.push(pts[pts.len() - 1]);
    out.iter()
        .map(|p| (p.x as f64, p.y as f64, p.z as f64))
        .collect()
}

/// Python's `max` of two: the first unless the second is strictly above it.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}
