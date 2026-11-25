//! Easing the families' hard edges to the gate's pixel (`cargo check-glb`'s `edges`: 4 mm at the play camera).

use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::scene::Obj;

/// The bevel the families give an edge that shows at the play camera: a pixel there (m).
pub const EASE: f64 = 0.004;

/// Eases a swept part's flat-capped ends by `EASE`: only its cap rims turn 60° or more, so only they are rounded.
pub fn ease_ends(obj: Obj) -> Obj {
    bevel_edges(obj, EASE, 1, 60.0, &EdgeOrder::Corners)
}

/// A turned profile [(radius, height), ...] with each corner that turns 40° or more cut across `e` along each side
/// (less on a short side), again until none does: a lathe's rims, lips and shoulders eased. The first and last
/// points (on the axis) stay.
pub fn eased(profile: &[(f64, f64)], e: f64) -> Vec<(f64, f64)> {
    // cos 40°: a turn sharper than this is cut
    const TURN: f64 = 0.766;
    let mut pts = profile.to_vec();
    let mut e = e;
    for _ in 0..4 {
        let mut out = vec![pts[0]];
        let mut cut = false;
        for i in 1..pts.len() - 1 {
            let (a, b, c) = (pts[i - 1], pts[i], pts[i + 1]);
            let (u, v) = ((b.0 - a.0, b.1 - a.1), (c.0 - b.0, c.1 - b.1));
            let (lu, lv) = (
                (u.0 * u.0 + u.1 * u.1).sqrt(),
                (v.0 * v.0 + v.1 * v.1).sqrt(),
            );
            if lu < 1e-9 || lv < 1e-9 || (u.0 * v.0 + u.1 * v.1) / (lu * lv) > TURN {
                out.push(b);
                continue;
            }
            cut = true;
            let (du, dv) = (e.min(lu / 3.0), e.min(lv / 3.0));
            out.push((b.0 - u.0 / lu * du, b.1 - u.1 / lu * du));
            out.push((b.0 + v.0 / lv * dv, b.1 + v.1 / lv * dv));
        }
        out.push(pts[pts.len() - 1]);
        pts = out;
        if !cut {
            break;
        }
        e /= 2.0;
    }
    pts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_square_rim_is_cut_into_turns_under_forty_degrees() {
        let p = eased(&[(0.0, 0.0), (0.1, 0.0), (0.1, 0.05), (0.0, 0.05)], EASE);
        for w in p.windows(3) {
            let (u, v) = (
                (w[1].0 - w[0].0, w[1].1 - w[0].1),
                (w[2].0 - w[1].0, w[2].1 - w[1].1),
            );
            let cos = (u.0 * v.0 + u.1 * v.1)
                / ((u.0 * u.0 + u.1 * u.1).sqrt() * (v.0 * v.0 + v.1 * v.1).sqrt());
            assert!(cos > 0.766, "{w:?}");
        }
        assert_eq!((p[0], *p.last().unwrap()), ((0.0, 0.0), (0.0, 0.05)));
    }
}
