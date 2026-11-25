//! Easing a whole part's hard edges to the gate's pixel (`cargo check-glb`'s `edges`: 4 mm at the play camera).

use super::*;
use crate::shapes::AUTO_WIDTH;

/// `obj` with its edges turning over 50° chamfered by a pixel at the play camera (what `cube!` gives a box): `ease`
/// at 50° in one strip.
pub fn eased(obj: Obj) -> Obj {
    ease(obj, 50.0, 1)
}

/// `obj` with its edges turning over `angle` degrees bevelled `AUTO_WIDTH` in `segments` strips: none in a far
/// version, nor on a part the gate's `edges` can't find hard (under 12 mm one way: its edges' faces too narrow; or
/// under 10 cm every way: a detail, not a piece). A lower angle where a turn the gate finds meets one under it at a
/// corner (a moulding's mitre, which a bevel can't stop at); two segments on a lip turning past 120° (one would
/// leave two turns of 60° and more). Its points are welded first, so a seam (a closed sweep's, a profile closed
/// on its first point) doesn't stop the bevel. A bevel the kernel can't build at `angle` is tried on the turns
/// over 30° (a mitre's), else on the edges it can be built on (`bevel_edges`).
pub fn ease(obj: Obj, angle: f64, segments: usize) -> Obj {
    if crate::far::is_low() {
        return obj;
    }
    let points = obj.world_points();
    let side = |k: fn(&V3) -> f32| {
        let (lo, hi) = points
            .iter()
            .map(k)
            .fold((f32::MAX, f32::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)));
        f64::from(hi - lo)
    };
    let sides = [side(|p| p.x), side(|p| p.y), side(|p| p.z)];
    if sides.iter().any(|&s| s < 0.0119) || sides.iter().all(|&s| s < 0.099) {
        return obj;
    }
    obj.edit(|o| o.mesh.remove_doubles(1e-6));
    let builds = |angle: f64| {
        obj.read(|o| bevelled(&o.mesh, AUTO_WIDTH, segments, angle, &EdgeOrder::Corners).is_ok())
    };
    let angle = [angle, 30.0]
        .into_iter()
        .find(|&a| builds(a))
        .unwrap_or(angle);
    bevel_edges(obj, AUTO_WIDTH, segments, angle, &EdgeOrder::Corners)
}
