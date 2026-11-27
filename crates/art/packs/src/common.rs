//! What more than one family of the group builds from: the slob pack's rounded box (the neat pack's appliances
//! and dock too) and the hot-headed pack's fluted ring (the kind pack's basket and planter too).

use artkit::materials::Mat;
use artkit::math::{P3, TAU, cos, sin};
use artkit::place::{Coord, GameFrame};
use artkit::scene::Obj;
use artkit::{loft, ring};

/// A rounded-cornered box lofted upright (a case, a cabinet, the neat pack's appliances and dock), its top and
/// bottom edges eased by `ease`.
#[allow(clippy::too_many_arguments)]
pub fn _box<X: Coord, Z: Coord>(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    at: (X, f64, Z),
    material: Mat,
    sq: f64,
    verts: usize,
    ease: f64,
) -> Obj {
    let (x, y, z) = at;
    let b = f.at(x, y - h / 2.0, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let rings = [(0.0, -ease), (ease, 0.0), (h - ease, 0.0), (h, -ease)];
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(hh, e)| {
            ring!(
                w / 2.0 + e,
                d / 2.0 + e,
                z = bz + hh,
                verts = verts,
                square = sq,
                at = (bx, by)
            )
        })
        .collect();
    loft!(name, &sections, material = material)
}

/// A ring of Blender points of radius `r` at height `y`, in and out `depth` (a fraction) `flutes` times round: the
/// pleats of a zafu, the ribs of a cactus, the kind pack's wicker basket and planter.
pub fn fluted(r: f64, y: f64, verts: usize, at: (f64, f64), flutes: i32, depth: f64) -> Vec<P3> {
    (0..verts)
        .map(|k| {
            let k = k as f64;
            let v = verts as f64;
            let flute = 1.0 + depth * cos(flutes as f64 * TAU * k / v);
            P3 {
                x: at.0 + r * flute * cos(TAU * k / v),
                y: at.1 + r * flute * sin(TAU * k / v),
                z: y,
            }
        })
        .collect()
}
