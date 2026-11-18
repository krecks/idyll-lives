//! What the group's families share between them.

use artkit::forms::rounded_rect;

/// A game point (x, height, depth), every part a float.
pub type P = (f64, f64, f64);

/// `_rr` of hobbies.py and fitness.py: a rounded rectangle outline (`rounded_rect`), for profiles and slabs.
pub fn rr(w: f64, h: f64, r: f64, corner: usize) -> Vec<(f64, f64)> {
    rounded_rect(w, h, r, corner)
}
