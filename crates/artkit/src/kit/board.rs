//! A board between two points, at any angle.

use crate::cube;
use crate::materials::Mat;
use crate::math::{M3, M4, V3};
use crate::place::{GameFrame, Spot};
use crate::scene::Obj;

/// A board from game point `a` to `b`, `w` wide and `t` thick, its edges eased by `bevel`, its broad face turned
/// towards game direction `face` (up for a seat slat, forwards for a back slat): slats, rails and legs at any angle.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn board(
    frame: &GameFrame,
    name: &str,
    a: impl Spot,
    b: impl Spot,
    w: f64,
    t: f64,
    material: Mat,
    face: impl Spot,
    bevel: f64,
) -> Obj {
    let (a, b) = (V3::from(a.through(frame)), V3::from(b.through(frame)));
    let z = (b - a).normalized();
    let x = V3::from(face.through(frame)).cross(z).normalized();
    let y = z.cross(x);
    let obj = cube!(
        name,
        (w, t, (b - a).length()),
        material = material,
        bevel = bevel
    );
    let rot = M3([x.array(), y.array(), z.array()]);
    obj.set_matrix_world(M4::translation((a + b) / 2) * rot.to_4x4())
}
