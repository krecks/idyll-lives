//! Easing hard edges to the gate's pixel: the eased profiles (`forms::eased`, `eased_loop`, `round_at`, `inset`).

use artkit::forms::{eased, eased_loop, inset, round_at};

fn turns(p: &[(f64, f64)], closed: bool) -> Vec<f64> {
    let n = p.len();
    let corners = if closed { 0..n } else { 1..n - 1 };
    corners
        .map(|i| {
            let (a, b, c) = (p[(i + n - 1) % n], p[i], p[(i + 1) % n]);
            let (u, v) = ((b.0 - a.0, b.1 - a.1), (c.0 - b.0, c.1 - b.1));
            (u.0 * v.1 - u.1 * v.0)
                .atan2(u.0 * v.0 + u.1 * v.1)
                .to_degrees()
                .abs()
        })
        .collect()
}

#[test]
fn eased_rounds_only_sharp_corners() {
    // a cup: its foot (63°) and its square rim round in two steps each; the shallow bend stays a point
    let rim = eased(
        &[
            (0.0, 0.0),
            (0.05, 0.0),
            (0.06, 0.02),
            (0.06, 0.1),
            (0.055, 0.1),
        ],
        0.003,
    );
    assert!(rim.len() == 9 && rim.contains(&(0.06, 0.02)) && !rim.contains(&(0.05, 0.0)));
    assert!(
        rim.iter()
            .all(|&(u, v)| u <= 0.06 + 1e-6 && v <= 0.1 + 1e-6)
    );
    // no corner turns 60° or more: none is hard
    assert!(turns(&rim, false).iter().all(|&t| t <= 50.0 + 1e-9));
}

#[test]
fn eased_loop_rounds_every_square_corner() {
    let sq = [(0.0, 0.0), (0.0, 0.01), (0.02, 0.01), (0.02, 0.0)];
    let out = eased_loop(&sq, 0.003);
    assert_eq!(out.len(), 4 * 3); // each 90° corner in two steps, three points
    assert!(!out.contains(&(0.0, 0.0)));
    assert!(turns(&out, true).iter().all(|&t| t <= 50.0 + 1e-9));
}

#[test]
fn round_at_rolls_one_corner_on_its_radius() {
    let p = [(0.0, 0.0), (0.1, 0.0), (0.1, 0.05)];
    let out = round_at(&p, 1, 0.01);
    assert_eq!(out.len(), 2 + 5);
    // the arc's ends on the legs a radius from the corner, its points a radius from (0.09, 0.01)
    assert!((out[1].0 - 0.09).abs() < 1e-12 && out[1].1.abs() < 1e-12);
    assert!((out[5].0 - 0.1).abs() < 1e-12 && (out[5].1 - 0.01).abs() < 1e-12);
    for q in &out[1..6] {
        assert!(((q.0 - 0.09).hypot(q.1 - 0.01) - 0.01).abs() < 1e-12);
    }
}

#[test]
fn inset_moves_a_square_in() {
    let sq = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    let small = inset(&sq, 0.1);
    assert!((small[0].0 - 0.1).abs() < 1e-12 && (small[0].1 - 0.1).abs() < 1e-12);
    assert!((small[2].0 - 0.9).abs() < 1e-12 && (small[2].1 - 0.9).abs() < 1e-12);
    // wound the other way round, the same; a negative inset moves it out
    let rev: Vec<_> = sq.iter().rev().copied().collect();
    assert!((inset(&rev, 0.1)[0].0 - 0.1).abs() < 1e-12);
    assert!((inset(&sq, -0.1)[0].0 + 0.1).abs() < 1e-12);
}
