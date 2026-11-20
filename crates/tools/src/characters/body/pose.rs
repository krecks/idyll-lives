//! Sitting: where the legs and the hands go, and how far the seat drops.

use super::*;

fn rx(a: f64) -> M3 {
    let (c, s) = (a.cos(), a.sin());
    M3::new(1.0, 0.0, 0.0, 0.0, c, -s, 0.0, s, c)
}

/// Seated (thighs turned `flex` degrees forward about the hips, a quarter turn for `seat_drop`; shins
/// straight down from the knees; every other bone at rest): each bone's skinning transform
/// `x -> R x + t` (seat_drop; the shells' seated corrective, `characters/clothes/`).
pub fn seated_pose(p: &[V3; NB], flex: f64) -> Vec<(M3, V3)> {
    let up = rx(radians(-flex));
    let down = rx(radians(flex));
    let mut mats: Vec<(M3, V3)> = vec![(M3::identity(), V3::zeros()); NB];
    for sd in ["l", "r"] {
        let (jt, jk) = (p[b(&format!("thigh_{sd}"))], p[b(&format!("calf_{sd}"))]);
        mats[b(&format!("thigh_{sd}"))] = (up, jt - up * jt);
        let knee = up * jk + (jt - up * jt);
        let r = up * down;
        for bn in ["calf", "foot", "ball"] {
            mats[b(&format!("{bn}_{sd}"))] = (r, knee - r * jk);
        }
    }
    mats
}

/// Seated (`seated_pose`): the pelvis joint's height above the lowest skin under the hips.
pub(crate) fn seat_drop(body: &Mesh, p: &[V3; NB]) -> f64 {
    let mats = seated_pose(p, 90.0);
    let n = body.pos.len();
    let mut out = vec![V3::zeros(); n];
    for (bi, (r, t)) in mats.iter().enumerate() {
        if !body.dense.iter().any(|d| d[bi] != 0.0) {
            continue;
        }
        for (o, (q, d)) in out.iter_mut().zip(body.pos.iter().zip(&body.dense)) {
            *o += (super::prims::rot(r, *q) + t) * d[bi];
        }
    }
    let hips = ["pelvis", "thigh_l", "thigh_r", "spine_01"].map(b);
    let low = out
        .iter()
        .zip(&body.dense)
        .filter(|(_, d)| hips.iter().fold(0.0, |s, &i| s + d[i]) > 0.6)
        .map(|(o, _)| o.y)
        .fold(f64::INFINITY, f64::min);
    p[b("pelvis")].y - low
}

/// The left palm's centre (just off the skin) relative to the wrist.
pub(crate) fn grip(family: &str, stage: &str) -> [f64; 3] {
    let s = surface(family, stage);
    let (wr, k1) = (s.p[b("hand_l")], s.p[b("fingers_01_l")]);
    let d = unit(k1 - wr);
    let u = unit(v3(0.0, 1.0, 0.0) - d * d[1]);
    let lp = (k1 - wr).norm();
    let g = d * 0.62 * lp - u * 0.28 * lp;
    [g.x, g.y, g.z]
}

/// Level 1's fingers past the middle joint: subdividing blends `fingers_01` into `fingers_02` all
/// along the outer segment, which a fist then folds flat (blended skinning collapses); instead the
/// split follows the distance past the joint, all `fingers_02` from 0.06 palm lengths on.
pub(crate) fn rigid_fingertips(mut dense: Vec<Dense>, pos: &[V3], p: &[V3; NB]) -> Vec<Dense> {
    for sd in ["l", "r"] {
        let (wr, i1, i2) = (
            p[b(&format!("hand_{sd}"))],
            b(&format!("fingers_01_{sd}")),
            b(&format!("fingers_02_{sd}")),
        );
        let lp = (p[i1] - wr).norm();
        let d = unit(p[i1] - wr);
        for (w, q) in dense.iter_mut().zip(pos) {
            let t = w[i1] + w[i2];
            if w[i2] > 0.0 {
                let s = smoothstep(-0.06, 0.06, (q - p[i2]).dot(&d) / lp);
                (w[i1], w[i2]) = (t * (1.0 - s), t * s);
            }
        }
    }
    dense
}

// Fields and tone -----------------------------------------------------------------------------

/// A smooth even function of the angle round the neck through the samples (least squares).
pub(crate) fn fourier(theta: &[f64], y: &[f64], n: usize) -> impl Fn(f64) -> f64 {
    let a = DMatrix::from_fn(theta.len(), n, |i, k| (k as f64 * theta[i]).cos());
    let coef = lstsq(&a, &DVector::from_column_slice(y));
    move |t: f64| (0..n).fold(0.0, |s, k| s + (k as f64 * t).cos() * coef[k])
}
