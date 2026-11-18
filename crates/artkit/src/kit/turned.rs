//! Turned and carved legs.

use crate::forms::{self, Scale, Sweep};
use crate::materials::Mat;
use crate::place::{Coord, GameFrame};
use crate::scene::Obj;
use crate::shapes::options;

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
