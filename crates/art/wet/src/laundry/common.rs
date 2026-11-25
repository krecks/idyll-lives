//! What the laundry's things are built from.

use artkit::materials::Mat;
use artkit::math::P3;
use artkit::place::{Coord, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, jitter, revolve, sphere};

pub use artkit::kit::WALL;
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
        artkit::py::min(0.02, h * 0.45),
        2,
        P3 {
            x: 0.0,
            y: 0.0,
            z: turn,
        },
    )
}
