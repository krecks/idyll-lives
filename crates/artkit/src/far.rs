//! Far versions: a model's own build run again at low detail (`Model::far_of`), so the far version has the same
//! shape, colours and size as the model, standing where it stands. The build asks `low(full, reduced)` for each
//! detail (segments, rings, counts) and `is_low()` for what to leave out (a flower's centre, the moving parts kept
//! apart: a far version is all one still part). It draws every random number the full build draws, in the same
//! order, so the far version's plants stand where the model's do.

use crate::scene;

/// `full` detail, or `reduced` while a far version is being built.
pub fn low<T>(full: T, reduced: T) -> T {
    if is_low() { reduced } else { full }
}

/// Whether a far version is being built.
pub fn is_low() -> bool {
    scene::with(|s| s.low)
}

/// Runs `build` as a far version.
pub fn far_of<R>(build: impl FnOnce() -> R) -> R {
    scene::with(|s| s.low = true);
    let out = build();
    scene::with(|s| s.low = false);
    out
}
