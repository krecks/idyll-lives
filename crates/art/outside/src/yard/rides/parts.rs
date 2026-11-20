//! The parts every bike shares: the bottom bracket, the spindle and the chain line.

use crate::yard::common::{F, M};
use crate::yard::play::Bike;
use artkit::far::low;
use artkit::fcyl;
use artkit::parts::numbered;

/// The bottom bracket's shell, across the frame where its tubes meet.
pub(crate) fn bb_shell(s: &Bike, width: f64) {
    fcyl!(
        F,
        numbered("bb_shell"),
        0.024 * s.k,
        width,
        (s.bb.0, s.bb.1, s.z),
        s.colour,
        rot = (90, 0, 0),
        verts = low(10, 5)
    );
}

/// The cranks' axle through the bottom bracket (in the cranks, which turn about it).
pub(crate) fn spindle(s: &Bike) {
    fcyl!(
        F,
        numbered("spindle"),
        0.009 * s.k,
        2.0 * s.crank_dz,
        (s.bb.0, s.bb.1, s.z),
        M.rim,
        rot = (90, 0, 0),
        verts = low(8, 4)
    );
}

/// The chain's top run's height at game x.
pub(crate) fn chain_top(s: &Bike, x: f64) -> f64 {
    let (y0, y1) = (s.bb.1 + s.ring, s.r + s.ring * s.chain_drop);
    y0 + (x - s.bb.0) * (y1 - y0) / (s.rear - s.bb.0)
}
