//! Bending a path round its corners.

use super::*;

/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
/// it, on a curve that never overshoots, however uneven the spacing: Catmull-Rom loops on a short leg beside a long
/// one). For handles, bent rails and draped cloth swept with `resolution` 1.
// (3.1415 is the script's own limit for a straight corner, not pi.)
#[allow(clippy::approx_constant)]
pub fn fillet<P: Corner>(points: &[P], radius: f64, steps: usize) -> Vec<P::Out> {
    let pts: Vec<V3> = points.iter().map(|&p| p.vector()).collect();
    let mut out = vec![pts[0]];
    for corner in pts.windows(3) {
        let (a, p, c) = (corner[0], corner[1], corner[2]);
        let (d1, d2) = (a - p, c - p);
        if d1.length() < 1e-9 || d2.length() < 1e-9 || d1.angle(d2) > 3.1415 {
            out.push(p);
            continue;
        }
        let t = least(&[
            radius / most(1e-6, tan(d1.angle(d2) / 2.0)),
            d1.length() * 0.5,
            d2.length() * 0.5,
        ]);
        let (p1, p2) = (p + d1.normalized() * t, p + d2.normalized() * t);
        for i in 0..=steps {
            let k = i as f64 / steps as f64;
            out.push(p1 * pow(1.0 - k, 2.0) + p * 2 * k * (1.0 - k) + p2 * k * k);
        }
    }
    out.push(pts[pts.len() - 1]);
    out.into_iter().map(P::tuple).collect()
}

options! {
    /// `shaker`'s options (`panel`: the panel's own material).
    Shaker { rail: f64 = 0.06, panel: Option<Mat> = None, recess: f64 = 0.005, raised: bool = false }
}
