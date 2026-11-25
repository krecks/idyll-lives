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
        out.extend(arc(a, p, c, r, turn));
    }
    out.push(profile[profile.len() - 1]);
    out
}

/// The corner `p` between legs from `a` and to `c`, turning `turn` degrees, rounded on a circular arc of radius
/// about `r` (less where a leg is short): points every 40° of it or less, so no two chords turn 60°.
fn arc(a: (f64, f64), p: (f64, f64), c: (f64, f64), r: f64, turn: f64) -> Vec<(f64, f64)> {
    use artkit::math::{atan2, cos, radians, sin, tan};
    let (l1, l2) = ((a.0 - p.0).hypot(a.1 - p.1), (c.0 - p.0).hypot(c.1 - p.1));
    let (u1, u2) = (
        ((a.0 - p.0) / l1, (a.1 - p.1) / l1),
        ((c.0 - p.0) / l2, (c.1 - p.1) / l2),
    );
    let half = radians(180.0 - turn) / 2.0; // half the angle between the legs
    let t = (r / tan(half)).min(l1 / 2.0).min(l2 / 2.0);
    let radius = t * tan(half);
    let (bx, by) = (u1.0 + u2.0, u1.1 + u2.1);
    let b = bx.hypot(by);
    let far = t / cos(half);
    let centre = (p.0 + bx / b * far, p.1 + by / b * far);
    let (p1, p2) = (
        (p.0 + u1.0 * t, p.1 + u1.1 * t),
        (p.0 + u2.0 * t, p.1 + u2.1 * t),
    );
    let (f1, f2) = (
        atan2(p1.1 - centre.1, p1.0 - centre.0),
        atan2(p2.1 - centre.1, p2.0 - centre.0),
    );
    let mut sweep = f2 - f1;
    while sweep > std::f64::consts::PI {
        sweep -= std::f64::consts::TAU;
    }
    while sweep < -std::f64::consts::PI {
        sweep += std::f64::consts::TAU;
    }
    let steps = (turn / 40.0).ceil() as usize;
    (0..=steps)
        .map(|i| {
            let f = f1 + sweep * i as f64 / steps as f64;
            (centre.0 + radius * cos(f), centre.1 + radius * sin(f))
        })
        .collect()
}

/// A closed profile (u, v) with each corner that turns 60° or more rounded to about `r`, as `eased` does, its first
/// and last corners too and those at its lowest v (a moulding's back on the wall shows from the side).
pub fn eased_loop(profile: &[(f64, f64)], r: f64) -> Vec<(f64, f64)> {
    let n = profile.len();
    let mut out = Vec::new();
    for i in 0..n {
        let (a, p, c) = (profile[(i + n - 1) % n], profile[i], profile[(i + 1) % n]);
        let (d1, d2) = ((p.0 - a.0, p.1 - a.1), (c.0 - p.0, c.1 - p.1));
        let turn = artkit::math::degrees(artkit::math::atan2(
            d1.0 * d2.1 - d1.1 * d2.0,
            d1.0 * d2.0 + d1.1 * d2.1,
        ))
        .abs();
        if turn < 60.0 {
            out.push(p);
            continue;
        }
        let steps = (turn / 50.0).ceil() as usize;
        out.extend(artkit::kit::fillet(&[a, p, c], r, steps)[1..=steps + 1].iter());
    }
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
mod eased_loop_tests {
    #[test]
    fn eased_loop_rounds_every_square_corner() {
        let sq = [(0.0, 0.0), (0.0, 0.01), (0.02, 0.01), (0.02, 0.0)];
        let out = super::eased_loop(&sq, 0.003);
        assert_eq!(out.len(), 4 * 3); // each 90° corner in two steps, three points
        assert!(!out.contains(&(0.0, 0.0)));
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
        assert!(rim.len() == 8 && rim.contains(&(0.06, 0.02)) && rim.contains(&(0.05, 0.0)));
        assert!(
            rim.iter()
                .all(|&(u, v)| u <= 0.06 + 1e-6 && v <= 0.1 + 1e-6)
        );
    }
}
