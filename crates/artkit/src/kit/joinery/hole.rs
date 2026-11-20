//! A hole in a face, triangulated into it.

use super::*;
use std::collections::BTreeMap;

/// A face with a hole whole inside it (`outer` anticlockwise, `hole` the other way round, each point's place
/// in `place`) as the two faces Blender leaves: one between the face's side nearest the hole and the hole, the
/// rest round the hole's other sides. (The rest, the one beside.)
pub(crate) fn round_hole(
    outer: &[u32],
    hole: &[u32],
    place: &BTreeMap<u32, P2>,
) -> (Vec<u32>, Vec<u32>) {
    let (n, h) = (outer.len(), hole.len());
    let at = |v: u32| place[&v];
    let apart = |x: P2, y: P2| ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt();
    // how far a point is from a stretch
    let off = |from: P2, to: P2, x: P2| {
        let t = along(from, to, x).clamp(0.0, 1.0);
        apart(
            [
                from[0] + (to[0] - from[0]) * t,
                from[1] + (to[1] - from[1]) * t,
            ],
            x,
        )
    };
    // the face's side nearest the hole; of sides as near as each other (a hole in the middle) neither, as
    // Blender's solver leaves a carcass's oven mouth
    let near: Vec<f64> = (0..n)
        .map(|i| {
            hole.iter()
                .map(|&v| off(at(outer[i]), at(outer[(i + 1) % n]), at(v)))
                .fold(f64::INFINITY, f64::min)
        })
        .collect();
    let alone = |i: usize| (0..n).all(|j| j == i || (near[j] - near[i]).abs() > 1e-7);
    let side = (0..n)
        .filter(|&i| alone(i))
        .min_by(|&x, &y| near[x].total_cmp(&near[y]))
        .unwrap_or_else(|| (0..n).min_by(|&x, &y| near[x].total_cmp(&near[y])).unwrap());
    let (from, to) = (outer[side], outer[(side + 1) % n]);
    // the hole's corner nearest a point of those that see it: the point is outside one of the corner's two sides
    let nearest = |x: u32| -> usize {
        (0..h)
            .filter(|&i| {
                let (before, here, after) = (
                    at(hole[(i + h - 1) % h]),
                    at(hole[i]),
                    at(hole[(i + 1) % h]),
                );
                turn(before, here, at(x)) > 0.0 || turn(here, after, at(x)) > 0.0
            })
            .min_by(|&i, &j| apart(at(hole[i]), at(x)).total_cmp(&apart(at(hole[j]), at(x))))
            .expect("a hole's corner is seen from outside it")
    };
    let first = nearest(from);
    let mut second = nearest(to);
    if second == first {
        second = (first + h - 1) % h;
    }
    // beside: from the hole along the face's side, then back along the hole; the rest: from the hole round the
    // face, then round the hole (each from the corner Blender's most often starts at)
    let mut beside = vec![hole[first], from, to];
    let mut i = second;
    while i != first {
        beside.push(hole[i]);
        i = (i + 1) % h;
    }
    let mut rest = vec![hole[second]];
    rest.extend((0..n).map(|i| outer[(side + 1 + i) % n]));
    let mut i = first;
    while i != second {
        rest.push(hole[i]);
        i = (i + 1) % h;
    }
    (rest, beside)
}
