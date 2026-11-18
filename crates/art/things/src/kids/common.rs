//! What the kids' things are built from.

use artkit::math::{TAU, cos, pow, sin};
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
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
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
