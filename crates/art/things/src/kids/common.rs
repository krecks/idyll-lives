//! What the kids' things are built from.

use artkit::math::{TAU, V3, cos, pow, sin, tan};
use artkit::parts::{Settle, measure, settle_with};
use artkit::scene::{self, meshes};

/// Stands the model on the floor where it was built (over where it stood) and says its centre; `forward` (game
/// depth, m) moves it towards the front first: its back onto the wall face.
pub fn place(forward: f64) {
    for o in meshes() {
        o.edit(|p| p.loc.y -= forward as f32);
    }
    let (lo, hi) = measure(&meshes());
    scene::log(format!(
        "[kids] centre (game x, depth) {:.4}, {:.4}",
        -(lo[0] + hi[0]) / 2.0,
        -(lo[1] + hi[1]) / 2.0
    ));
    settle_with(&meshes(), Settle { centre: false, floor: true });
}

/// A heart's outline round (cx, cy), about `r` across from its middle.
pub fn heart(cx: f64, cy: f64, r: f64, n: usize) -> Vec<(f64, f64)> {
    let mut pts = Vec::new();
    for i in 0..n {
        let t = TAU * i as f64 / n as f64;
        let x = 16.0 * pow(sin(t), 3.0);
        let y = 13.0 * cos(t) - 5.0 * cos(2.0 * t) - 2.0 * cos(3.0 * t) - cos(4.0 * t);
        pts.push((cx + r * x / 16.0, cy + r * (y + 2.0) / 16.0));
    }
    pts.reverse();
    pts
}

/// gamekit's `kit.fillet`, until the kit has it: the path through `points` (any axes) with each inner corner
/// rounded to about `radius` (`steps` points round it, on a curve that never overshoots, however uneven the
/// spacing). For handles, bent rails and draped cloth swept with `resolution` 1.
pub fn fillet(points: &[(f64, f64, f64)], radius: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let pts: Vec<V3> = points.iter().map(|&p| V3::from(p)).collect();
    let mut out = vec![pts[0]];
    for w in pts.windows(3) {
        let (a, p, c) = (w[0], w[1], w[2]);
        let (d1, d2) = (a - p, c - p);
        if d1.length() < 1e-9 || d2.length() < 1e-9 || d1.angle(d2) > 3.1415 {
            out.push(p);
            continue;
        }
        let t = (radius / 1e-6_f64.max(tan(d1.angle(d2) / 2.0)))
            .min(d1.length() * 0.5)
            .min(d2.length() * 0.5);
        let (p1, p2) = (p + d1.normalized() * t, p + d2.normalized() * t);
        for i in 0..=steps {
            let k = i as f64 / steps as f64;
            out.push(p1 * pow(1.0 - k, 2.0) + p * 2 * k * (1.0 - k) + p2 * k * k);
        }
    }
    out.push(pts[pts.len() - 1]);
    out.iter().map(|p| (p.x as f64, p.y as f64, p.z as f64)).collect()
}
