//! What the families here share: easing a profile's sharp corners (a rim, a lip) and a prism's ends, so their
//! edges catch the light as made things' do (the gate's `edges`).

use artkit::loft;
use artkit::materials::Mat;
use artkit::math::{P3, V3};
use artkit::place::GameFrame;
use artkit::scene::Obj;

/// A lathe's profile (or any open path, (u, v)) with each corner that turns 60° or more rounded to about `r`, in
/// enough steps that none turns 60°: an eased rim or lip (the gate's `edges`). The other corners are kept, and so
/// are those at its lowest v (a foot on the floor: an underside no one sees).
pub fn eased(profile: &[(f64, f64)], r: f64) -> Vec<(f64, f64)> {
    let low = profile.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let mut out = vec![profile[0]];
    for w in profile.windows(3) {
        let (a, p, c) = (w[0], w[1], w[2]);
        let (d1, d2) = ((p.0 - a.0, p.1 - a.1), (c.0 - p.0, c.1 - p.1));
        let turn = artkit::math::degrees(artkit::math::atan2(
            d1.0 * d2.1 - d1.1 * d2.0,
            d1.0 * d2.0 + d1.1 * d2.1,
        ))
        .abs();
        if turn < 60.0 || p.1 <= low + 1e-9 {
            out.push(p);
            continue;
        }
        let steps = (turn / 50.0).ceil() as usize;
        out.extend(artkit::kit::fillet(&[a, p, c], r, steps)[1..=steps + 1].iter());
    }
    out.push(profile[profile.len() - 1]);
    out
}

/// A closed outline (u, v) moved `d` in all round, each corner along its bisector: the face of a chamfer at an end
/// of a body lofted through it.
pub fn inset(outline: &[(f64, f64)], d: f64) -> Vec<(f64, f64)> {
    let n = outline.len();
    let area: f64 = (0..n)
        .map(|i| {
            let (a, b) = (outline[i], outline[(i + 1) % n]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum();
    let side = if area > 0.0 { 1.0 } else { -1.0 }; // the inside is on the left going round
    let normal = |a: (f64, f64), b: (f64, f64)| {
        let (du, dv) = (b.0 - a.0, b.1 - a.1);
        let l = du.hypot(dv);
        (-dv / l * side, du / l * side)
    };
    (0..n)
        .map(|i| {
            let (p, prev, next) = (outline[i], outline[(i + n - 1) % n], outline[(i + 1) % n]);
            let (n1, n2) = (normal(prev, p), normal(p, next));
            let (mu, mv) = (n1.0 + n2.0, n1.1 + n2.1);
            let m = mu.hypot(mv);
            // as far along the bisector as keeps both sides d in, but no more than 2d (a sharp corner)
            let k = (d / (m / 2.0).max(0.5)) / m;
            (p.0 + mu * k, p.1 + mv * k)
        })
        .collect()
}

/// A body of `outline` ((height, x out) about game x `x`, in `f`), from game depth `z0` to `z1`, its two ends chamfered
/// `c` (lofted through its outline set in at each end): an upholstered arm whose front and back are eased.
#[allow(clippy::too_many_arguments)]
pub fn eased_prism(
    f: &GameFrame,
    name: &str,
    outline: &[(f64, f64)],
    x: f64,
    z0: f64,
    z1: f64,
    c: f64,
    mat: Mat,
) -> Obj {
    let small = inset(outline, c);
    let ring = |o: &[(f64, f64)], z: f64| -> Vec<P3> {
        o.iter().map(|&(u, v)| f.at(x - v, u, z)).collect()
    };
    let depths = [z0, z0 + c, z1 - c, z1];
    let body = loft!(
        name,
        &[
            ring(&small, z0),
            ring(outline, depths[1]),
            ring(outline, depths[2]),
            ring(&small, z1)
        ],
        material = mat,
        angle = 50
    );
    // laid as the sweep it was: round its outline, along its length
    let path: Vec<V3> = depths.iter().map(|&z| f.at(x, 0, z).into()).collect();
    artkit::uv::along(body, 4, outline, &[1.0; 4], &path, false)
}

#[cfg(test)]
mod inset_tests {
    #[test]
    fn inset_moves_a_square_in() {
        let sq = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let small = super::inset(&sq, 0.1);
        assert!((small[0].0 - 0.1).abs() < 1e-12 && (small[0].1 - 0.1).abs() < 1e-12);
        assert!((small[2].0 - 0.9).abs() < 1e-12 && (small[2].1 - 0.9).abs() < 1e-12);
        // wound the other way round, the same
        let rev: Vec<_> = sq.iter().rev().copied().collect();
        assert!((super::inset(&rev, 0.1)[0].0 - 0.1).abs() < 1e-12);
    }
}

#[cfg(test)]
mod eased_tests {
    #[test]
    fn eased_rounds_only_sharp_corners() {
        // a cup's square rim: the two 90° corners round, the shallow bend stays a point
        let rim = super::eased(
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.06, 0.02),
                (0.06, 0.1),
                (0.055, 0.1),
            ],
            0.003,
        );
        assert!(rim.len() == 7 && rim.contains(&(0.06, 0.02)) && rim.contains(&(0.05, 0.0)));
        assert!(
            rim.iter()
                .all(|&(u, v)| u <= 0.06 + 1e-6 && v <= 0.1 + 1e-6)
        );
    }
}
