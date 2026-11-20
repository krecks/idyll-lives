//! Marking a clip's peak.
//!
//! Split from `motion.rs` (see `mod.rs`).

use super::*;

/// gamekit's `motion._peak`, which a script may call itself: marks where clip `clip` is most open, for its preview
/// (default: its last key).
pub fn peak(obj: Obj, clip: &str, frame: impl crate::math::Num) -> Obj {
    mark_peak(obj, clip, py::round(frame.f()) as i32)
}
