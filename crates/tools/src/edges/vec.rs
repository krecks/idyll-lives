//! The vector arithmetic the check needs.
//!
//! Split from `crates/tools/src/edges.rs` (see `mod.rs`).

use super::*;

pub(crate) fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub(crate) fn mul(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub(crate) fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(crate) fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(crate) fn length(a: V3) -> f64 {
    dot(a, a).sqrt()
}

/// `a` at length 1 (as it is if it has none).
pub(crate) fn unit(a: V3) -> V3 {
    let l = length(a);
    if l > 0.0 { mul(a, 1.0 / l) } else { a }
}
