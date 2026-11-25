//! What the garden decor is built from.

use artkit::bvh::Rays;
use artkit::math::{P3, PI, TAU, V3, cos, p3, radians, sin, v3};

/// Loft rings for a turned body whose rim is scalloped: `rows` (radius, height), each ring's radius waving by
/// `depth` (a fraction) round `lobes` lobes, fading in from rows at or above `scallop_from` height.
pub fn _scalloped(
    rows: &[(f64, f64)],
    lobes: f64,
    depth: f64,
    verts: usize,
    scallop_from: f64,
) -> Vec<Vec<P3>> {
    let mut out = Vec::new();
    for &(r, z) in rows {
        let k = if z >= scallop_from { depth } else { 0.0 };
        out.push(
            (0..verts)
                .map(|i| {
                    let (i, verts) = (i as f64, verts as f64);
                    p3(
                        r * (1.0 + k * cos(lobes * TAU * i / verts)) * cos(TAU * i / verts),
                        r * (1.0 + k * cos(lobes * TAU * i / verts)) * sin(TAU * i / verts),
                        z,
                    )
                })
                .collect(),
        );
    }
    out
}

/// A loop round the lantern's axis at height `z`: a hexagon `r` to its corners (corners to the sides, flats to
/// the front and back), or with `n` 12 its corners and the middles of its flats (`flat`: their radius; `lift`:
/// the corners raised, a roof's upturned eaves); `phase` turns it (degrees).
pub fn _hex(r: f64, z: f64, n: usize, flat: Option<f64>, lift: f64, phase: f64) -> Vec<P3> {
    let mut out = Vec::new();
    for j in 0..n {
        let a = radians(phase + (360 * j) as f64 / n as f64);
        let corner = n == 6 || j % 2 == 0;
        let rr = if corner {
            r
        } else {
            flat.unwrap_or(r * cos(PI / 6.0))
        };
        out.push(p3(
            rr * cos(a),
            rr * sin(a),
            z + if corner { lift } else { 0.0 },
        ));
    }
    out
}

/// A ring of carved lotus petals from radius r0 at z0 to r1 at z1: rings of `n` x 3 points, each petal's middle
/// proud by `depth` (a fraction), its edges a little in.
pub fn _petals(r0: f64, r1: f64, z0: f64, z1: f64, n: usize, depth: f64) -> Vec<Vec<P3>> {
    let ring = |r: f64, z: f64| -> Vec<P3> {
        let mut out = Vec::new();
        for j in 0..3 * n {
            let k = r * if j % 3 == 1 {
                1.0 + depth
            } else {
                1.0 - depth * 0.25
            };
            out.push(p3(
                k * cos(TAU * j as f64 / (3 * n) as f64),
                k * sin(TAU * j as f64 / (3 * n) as f64),
                z,
            ));
        }
        out
    };
    vec![ring(r0, z0), ring(r1, z1)]
}

/// Where lines straight down through `points` (x, y) first meet what is built so far, `sink` below it.
pub fn _on_top(points: &[(f64, f64)], sink: f64) -> Vec<V3> {
    let rays = Rays::of_scene();
    points
        .iter()
        .filter_map(|&(x, y)| rays.cast(v3(x, y, 5), v3(0, 0, -1), 10.0))
        .map(|hit| hit.at - v3(0, 0, sink))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hexagon_of_twelve_has_its_flats_in_and_its_corners_lifted() {
        let ring = _hex(0.2, 1.0, 12, Some(0.15), 0.03, 0.0);
        assert_eq!(ring.len(), 12);
        assert!((ring[0].x - 0.2).abs() < 1e-12 && (ring[0].z - 1.03).abs() < 1e-12);
        assert!(((ring[1].x * ring[1].x + ring[1].y * ring[1].y).sqrt() - 0.15).abs() < 1e-12);
        assert!((ring[1].z - 1.0).abs() < 1e-12);
    }
}
