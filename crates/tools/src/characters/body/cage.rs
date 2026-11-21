//! The control cage: its shape, and the ring points it is made of.

use super::*;

pub(super) fn shape(family: &str, stage: &str) -> Shape {
    let pr = proportions(family, stage);
    let (h, g) = (pr.h, pr.girth);
    let female = family == "female";
    let yg = youth(pr.hh);
    // Grown: a man's arms and neck thicker, a woman's slighter.
    let m = maturity(stage);
    let grown = |a: f64, b: f64| if female { a * m } else { b * m };
    Shape {
        h,
        female,
        yg,
        m,
        r_th: (if female { 0.058 } else { 0.056 } + grown(0.002, -0.003))
            * h
            * g
            * (1.0 + 0.08 * yg),
        r_a: (if female { 0.029 } else { 0.033 } + grown(-0.0015, 0.002)) * h * g,
        r_n: (if female { 0.031 } else { 0.035 } + grown(-0.0025, 0.0)) * h * g.powf(0.5),
        waist: pr.waist,
        belly: pr.belly * h,
    }
}

/// Ring points round centre c: angle 0 towards +V, 90 towards +U; depth bf in front (+V), bb behind.
#[allow(clippy::too_many_arguments)]
pub(super) fn ring_pts(
    c: V3,
    u: V3,
    v: V3,
    a: f64,
    bf: f64,
    bb: f64,
    ang: &[f64],
    p: f64,
) -> Vec<V3> {
    ang.iter()
        .map(|&t| {
            let (s, co) = (t.sin(), t.cos());
            c + u * (a * se(s, p)) + v * ((if co >= 0.0 { bf } else { bb }) * se(co, p))
        })
        .collect()
}

/// The control cage (targets on the designed surface) of `family` at `stage`, and its joints.
pub fn body_cage(family: &str, stage: &str) -> (Cage, [V3; NB]) {
    body_cage_of(family, stage, &Identity::NONE)
}
