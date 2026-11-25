//! An instrument's body outline: two bouts meeting at a waist.

use crate::forms::smooth_outline;
use crate::math::{asin, cosd, degrees, sind, sqrt};

/// An instrument's body outline (game x, height), its lower and upper bouts the ellipses `lower` and `upper`
/// (middle height, half-width, half-height) meeting at a rounded waist, `n` steps round each bout and smoothed
/// `passes` times; from the foot round. A guitar, a ukulele, a cello.
pub fn figure8(
    lower: (f64, f64, f64),
    upper: (f64, f64, f64),
    n: usize,
    passes: usize,
) -> Vec<(f64, f64)> {
    let ((c1, a1, b1), (c2, a2, b2)) = (lower, upper);
    let half = |c: f64, a: f64, b: f64, y: f64| {
        let t = (y - c) / b;
        let k = 1.0 - t * t;
        a * sqrt(if k > 0.0 { k } else { 0.0 })
    };
    let (mut lo, mut hi) = (c1, c2);
    // the waist: where the bouts are as wide as each other
    for _ in 0..40 {
        let m = (lo + hi) / 2.0;
        (lo, hi) = if half(c1, a1, b1, m) > half(c2, a2, b2, m) {
            (m, hi)
        } else {
            (lo, m)
        };
    }
    let yw = (lo + hi) / 2.0;
    let t1 = degrees(asin(((yw - c1) / b1).clamp(-1.0, 1.0)));
    let t2 = degrees(asin(((yw - c2) / b2).clamp(-1.0, 1.0)));
    let mut right: Vec<(f64, f64)> = (0..=n)
        .map(|i| -90.0 + (t1 + 90.0) * i as f64 / n as f64)
        .map(|t| (a1 * cosd(t), c1 + b1 * sind(t)))
        .collect();
    right.extend(
        (0..=n)
            .map(|i| t2 + (90.0 - t2) * i as f64 / n as f64)
            .map(|t| (a2 * cosd(t), c2 + b2 * sind(t))),
    );
    let left: Vec<(f64, f64)> = right[1..right.len() - 1]
        .iter()
        .rev()
        .map(|&(x, y)| (-x, y))
        .collect();
    right.extend(left);
    smooth_outline(&right, passes, true)
}
