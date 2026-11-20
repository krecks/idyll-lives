//! A recessed box.
//!
//! Split from `kit/joinery.rs` (see `mod.rs`).

use super::*;

/// A box (game `w` x `h` x `d` at `at`) with a box-shaped recess in it: `cavity` (its width, height and depth)
/// at game `cavity_at`, reaching out of the box through the one face it opens: an oven's carcass and its mouth,
/// a dishwasher's tub. Reaching out of the opposite face too, it is a hole right through. `cut` of the two.
#[allow(clippy::too_many_arguments)] // (as the scripts wrote the two boxes)
pub fn recessed_box(
    frame: &GameFrame,
    name: impl AsRef<str>,
    w: f64,
    h: f64,
    d: f64,
    at: (impl Coord, impl Coord, impl Coord),
    material: Mat,
    cavity: (f64, f64, f64),
    cavity_at: (impl Coord, impl Coord, impl Coord),
    o: RecessedBox,
) -> Obj {
    let name = name.as_ref();
    let body = frame.box_(
        name,
        w,
        h,
        d,
        at,
        material,
        Cube {
            bevel: o.bevel,
            bevel_segments: o.bevel_segments,
            ..Cube::default()
        },
    );
    if !o.round_mouth {
        crate::bevel::apply(body);
    }
    let cutter = frame.box_(
        format!("{name}_cavity"),
        cavity.0,
        cavity.1,
        cavity.2,
        cavity_at,
        o.lining.unwrap_or(material),
        Cube::default(),
    );
    cut(
        body,
        cutter,
        Cut {
            lined: o.lining.is_some(),
            ..Cut::default()
        },
    )
}
