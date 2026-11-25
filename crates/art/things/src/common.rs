//! What the group's families share between them.

/// A game point (x, height, depth), every part a float.
pub type P = (f64, f64, f64);

/// `profile` with its corner `i` rounded on an arc of radius `r` (four even steps round it, so each ring turns a
/// quarter of the corner from the last and a lathe shades it round, not as an edge): a plate's rolled rim, a lip.
pub fn round_at(profile: &[(f64, f64)], i: usize, r: f64) -> Vec<(f64, f64)> {
    use artkit::math::{PI, TAU, acos, atan2, cos, sin, sqrt};
    let unit = |(x, y): (f64, f64)| {
        let l = sqrt(x * x + y * y);
        (x / l, y / l)
    };
    let p = profile[i];
    let u1 = unit((profile[i - 1].0 - p.0, profile[i - 1].1 - p.1));
    let u2 = unit((profile[i + 1].0 - p.0, profile[i + 1].1 - p.1));
    let half = acos((u1.0 * u2.0 + u1.1 * u2.1).clamp(-1.0, 1.0)) / 2.0;
    let mid = unit((u1.0 + u2.0, u1.1 + u2.1));
    let centre = (p.0 + mid.0 * r / sin(half), p.1 + mid.1 * r / sin(half));
    let t = r * cos(half) / sin(half);
    let from = atan2(p.1 + u1.1 * t - centre.1, p.0 + u1.0 * t - centre.0);
    let to = atan2(p.1 + u2.1 * t - centre.1, p.0 + u2.0 * t - centre.0);
    let mut turn = to - from;
    if turn > PI {
        turn -= TAU;
    } else if turn < -PI {
        turn += TAU;
    }
    let mut out = profile[..i].to_vec();
    out.extend((0..=4).map(|k| {
        let a = from + turn * k as f64 / 4.0;
        (centre.0 + r * cos(a), centre.1 + r * sin(a))
    }));
    out.extend(&profile[i + 1..]);
    out
}
