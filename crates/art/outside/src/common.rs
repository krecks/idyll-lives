//! What more than one family of the group builds from: the game frame, a round bar and a bent tube (the outdoor
//! things', the yard's and the civic buildings').

use artkit::kit;
use artkit::materials::Mat;
use artkit::place::{GameFrame, Spot};
use artkit::scene::Obj;

pub const F: GameFrame = GameFrame::new(0.0);

/// A round bar from game point `a` to `b`.
pub fn rod(name: &str, a: impl Spot, b: impl Spot, r: f64, material: Mat, verts: usize) -> Obj {
    artkit::rod!(
        name,
        a.through(&F),
        b.through(&F),
        r,
        material = material,
        verts = verts
    )
}

/// One length of round bar or tube through game points, its corners bent round about `bend` (no overshoot).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn bent(
    name: &str,
    pts: &[(f64, f64, f64)],
    r: f64,
    material: Mat,
    bend: f64,
    verts: usize,
    steps: usize,
    scale: f64,
) -> Obj {
    kit::tube!(
        F,
        name,
        &kit::fillet!(pts, bend, steps),
        r,
        material,
        verts = verts,
        round = 1,
        scale = scale
    )
}
