//! Turned and carved legs, and the turned and lofted round pieces the packs stand on a game point.

use crate::forms::{self, Loft, Ring, Scale, Sweep};
use crate::materials::Mat;
use crate::math::{P3, TAU, cos, sin};
use crate::place::{Coord, GameFrame};
use crate::scene::Obj;
use crate::shapes::{self, Revolve, options};

options! {
    /// `cabriole`'s options.
    Cabriole { r: f64 = 0.024, verts: usize = 12 }
}

/// A cabriole leg from the frame at height `h` over (x, z) down to the floor: out by (sx, sz) at the knee, slim
/// at the ankle, flaring into a pad foot that stands on the floor (its lowest point at height 0).
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn cabriole(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x: impl Coord,
    z: impl Coord,
    h: f64,
    material: Mat,
    sx: f64,
    sz: f64,
    o: Cabriole,
) -> Obj {
    let (fx, fz) = (x.f(), z.f());
    let points = [
        frame.at(x, h, z),
        frame.at(fx + sx, h * 0.62, fz + sz),
        frame.at(fx + sx * 0.4, 0.06, fz + sz * 0.4),
        frame.at(fx + sx * 0.55, 0.02, fz + sz * 0.55),
        frame.at(fx + sx * 0.55, 0.0, fz + sz * 0.55),
    ];
    forms::sweep(
        name,
        &points,
        o.r,
        Sweep {
            material: Some(material),
            verts: Some(o.verts),
            resolution: 2,
            scale: Scale::List(vec![1.0, 1.05, 0.7, 1.1, 1.15]),
            ..Sweep::default()
        },
    )
}

options! {
    /// `turned`'s options.
    Turned { segments: usize = 16, rot: P3 = P3::ZERO }
}

/// A turned piece standing at game (x, y, z): `profile` [(radius, height up), ...] from its foot, revolved (the
/// packs' `_turned`: a saucer, a cup, a knob, a lid).
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn turned(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x: impl Coord,
    y: impl Coord,
    z: impl Coord,
    profile: &[(f64, f64)],
    material: Mat,
    o: Turned,
) -> Obj {
    shapes::revolve(
        name,
        profile,
        Revolve {
            at: frame.at(x, y, z),
            material: Some(material),
            segments: Some(o.segments),
            rot: o.rot,
            ..Revolve::default()
        },
    )
}

options! {
    /// `lathe`'s options.
    Lathe { verts: usize = 16, square: f64 = 2.0, sy: f64 = 1.0 }
}

/// A body lofted through rings [(radius, height up), ...] standing at game (x, y, z), round or squarer as `square`
/// grows (`forms::ring`); `sy` squashes them in depth (the packs' `_lathe`: a stool's seat, a bottle, a speaker).
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn lathe(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x: impl Coord,
    y: f64,
    z: impl Coord,
    rings: &[(f64, f64)],
    material: Mat,
    o: Lathe,
) -> Obj {
    let P3 { x: bx, y: by, .. } = frame.at(x, 0, z);
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(r, h)| {
            forms::ring(
                r,
                r * o.sy,
                Ring {
                    z: y + h,
                    verts: o.verts,
                    square: o.square,
                    at: (bx, by),
                },
            )
        })
        .collect();
    forms::loft(
        name,
        &sections,
        Loft {
            material: Some(material),
            ..Loft::default()
        },
    )
}

/// A ring of `flutes` shallow flutes round radius `r` at Blender height `z`, about the origin, turned `phase`
/// radians: `per` points a flute, sunk `depth` of the radius at their middles (a column, a tart's case, a cupcake's
/// paper cup).
pub fn fluted(r: f64, z: f64, flutes: usize, depth: f64, per: usize, phase: f64) -> Vec<P3> {
    let n = flutes * per;
    (0..n)
        .map(|i| {
            let a = TAU * i as f64 / n as f64 + phase;
            let k = 1.0 - depth * (0.5 + 0.5 * cos(flutes as f64 * a));
            P3 {
                x: r * k * cos(a),
                y: r * k * sin(a),
                z,
            }
        })
        .collect()
}
