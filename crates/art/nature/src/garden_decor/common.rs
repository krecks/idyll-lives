//! What the garden decor is built from.

use crate::ground::rock::{_cut, _lump};
use crate::plants::common::options;
use artkit::bvh::Rays;
use artkit::materials::Mat;
use artkit::math::{M4, P3, PI, TAU, V3, cos, p3, pow, radians, sin, tan, v3};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{boulder, loft, sphere, sweep};

// KIT: kit.fillet
/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
/// it, on a curve that never overshoots, however uneven the spacing: Catmull-Rom loops on a short leg beside a
/// long one). For handles, bent rails and draped cloth swept with `resolution` 1.
// (3.1415 is the script's own limit for a straight corner, not pi.)
#[allow(clippy::approx_constant)]
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
    out.iter()
        .map(|p| (p.x as f64, p.y as f64, p.z as f64))
        .collect()
}

options! {
    /// `songbird`'s options.
    Songbird { eye: Option<Mat> = None, turn: f64 = 0.0, size: f64 = 1.0, head: f64 = 0.0, low: bool = false }
}

// KIT: kit.songbird
/// A small songbird (a robin, a tit, a sparrow) perched with its feet at game (x, y, z), facing the front turned
/// `turn` degrees towards +x: a lofted body tapering into a fanned tail, a `breast` patch, a pointed `beak`,
/// `eye`s, thin legs down to the perch. `size` 1 is a robin (14 cm long); `head` tips the head down (degrees:
/// drinking, pecking); `low` takes fewer sides and no eyes (a far version). About 350 triangles. Returns its
/// pieces.
#[allow(clippy::too_many_arguments)]
pub fn songbird(
    frame: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    body: Mat,
    breast: Mat,
    beak: Mat,
    o: Songbird,
) -> Vec<Obj> {
    let Songbird {
        eye,
        turn,
        size,
        head,
        low,
    } = o;
    let (c, s) = (cos(radians(turn)), sin(radians(turn)));
    let dip = radians(head);
    // The game point `u` across (to the bird's right), `v` forward and `w` up from its feet (at size 1).
    let at = |u: f64, v: f64, w: f64| -> P3 {
        let (mut v, mut w) = (v, w);
        if v > 0.03 {
            // the head and beak tip down about the neck
            let (dv, dw) = (v - 0.03, w - 0.048);
            (v, w) = (
                0.03 + dv * cos(dip) + dw * sin(dip),
                0.048 - dv * sin(dip) + dw * cos(dip),
            );
        }
        let (u, v, w) = (u * size, v * size, w * size);
        frame.at(x + v * s + u * c, y + w, z + v * c - u * s)
    };
    // (forward, half width, half height, centre height): from the tail's root to the face
    let rows = [
        (-0.05, 0.010, 0.010, 0.040),
        (-0.035, 0.022, 0.024, 0.036),
        (-0.012, 0.029, 0.031, 0.034),
        (0.012, 0.027, 0.030, 0.038),
        (0.03, 0.02, 0.022, 0.048),
        (0.045, 0.02, 0.02, 0.058),
        (0.06, 0.016, 0.015, 0.06),
        (0.07, 0.007, 0.007, 0.059),
    ];
    let n = if low { 6 } else { 10 };
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(v, hw, hh, wc)| {
            (0..n)
                .map(|k| {
                    at(
                        hw * cos(TAU * k as f64 / n as f64),
                        v,
                        wc + hh * sin(TAU * k as f64 / n as f64),
                    )
                })
                .collect()
        })
        .collect();
    let mut made = vec![loft!(name, &sections, material = body)];
    made.push(sweep!(
        format!("{name}_beak"),
        &[at(0.0, 0.066, 0.058), at(0.0, 0.088, 0.057)],
        0.0055 * size,
        material = beak,
        verts = if low { 5 } else { 6 },
        scale = [1.0, 0.15]
    ));
    let breast_at: V3 = at(0.0, 0.01, 0.034).into();
    made.push(sphere!(
        format!("{name}_breast"),
        1.0,
        at = breast_at,
        material = breast,
        rot = (0, 0, -turn),
        scale = (0.026 * size, 0.027 * size, 0.025 * size),
        segments = if low { 8 } else { 10 },
        rings = if low { 5 } else { 6 }
    ));
    // the tail: a thin fan, cocked a little up
    made.push(sweep!(
        format!("{name}_tail"),
        &[at(0.0, -0.04, 0.041), at(0.0, -0.088, 0.054)],
        0.0,
        material = body,
        profile = [
            (-0.0018, -0.008),
            (0.0018, -0.008),
            (0.0018, 0.008),
            (-0.0018, 0.008)
        ],
        up = (0, 0, 1),
        scale = [0.9 * size, 1.8 * size]
    ));
    if let Some(eye) = eye
        && !low
    {
        for k in [-1, 1] {
            made.push(sphere!(
                format!("{name}_eye{k}"),
                0.0042 * size,
                at = at(k as f64 * 0.0148, 0.055, 0.062),
                material = eye,
                segments = 6,
                rings = 4
            ));
        }
    }
    for k in [-1, 1] {
        let k_ = k as f64;
        made.push(sweep!(
            format!("{name}_leg{k}"),
            &[
                at(k_ * 0.008, 0.0, 0.026),
                at(k_ * 0.009, 0.004, 0.0),
                at(k_ * 0.009, 0.016, 0.0)
            ],
            0.0016 * size,
            material = beak,
            verts = 4
        ));
    }
    made
}

/// `songbird` with its options by name, as the family scripts wrote them:
/// `songbird!(frame, name, x, y, z, body, breast, beak, key = value, ...)`.
#[macro_export]
macro_rules! songbird {
    ($frame:expr, $name:expr, $x:expr, $y:expr, $z:expr, $body:expr, $breast:expr, $beak:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::garden_decor::common::Songbird::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::garden_decor::common::songbird($frame, $name, $x, $y, $z, $body, $breast, $beak, o)
    }};
}

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

/// A weathered stone `top` high: a boulder roughened (as the woods' rock, the ground's), faces split off flat by
/// `fractures` (direction, depth in from its outside, metres), and its foot cut off flat where it goes into the
/// lawn, so it sits broad and heavy in it, not on it.
#[allow(clippy::too_many_arguments)]
pub fn _stone(
    name: &str,
    radius: f64,
    at: (f64, f64),
    material: Mat,
    scale: (f64, f64, f64),
    seed: f64,
    top: f64,
    fractures: &[((f64, f64, f64), f64)],
    subdivisions: usize,
) -> Obj {
    let o = boulder!(
        name,
        radius,
        at = (at.0, at.1, 0),
        material = material,
        scale = (1, 1, 1),
        seed = seed,
        noise = 0.3,
        subdivisions = subdivisions
    );
    _lump(o, 0.1, 4.0, seed);
    o.edit(|p| p.mesh.transform(&M4::diagonal(scale.into())));
    for &(direction, depth) in fractures {
        let n = V3::from(direction).normalized();
        let reach = o.read(|p| {
            p.mesh
                .verts
                .iter()
                .map(|v| v.dot(n))
                .fold(f64::NEG_INFINITY, f64::max)
        });
        // (the stone's matrix is still the one it was made with, at the origin, while these are cut)
        _cut(o, &M4::IDENTITY, n * (reach - depth), n);
    }
    o.edit(|p| {
        let highest = p
            .mesh
            .verts
            .iter()
            .map(|v| v.z)
            .fold(f32::NEG_INFINITY, f32::max);
        p.loc.z = (top - highest as f64) as f32;
    });
    _cut(o, &o.matrix_world(), (0, 0, 0), (0, 0, -1))
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
    use artkit::materials::Mat;
    use artkit::scene;

    #[test]
    fn a_fillet_rounds_inner_corners_and_keeps_the_ends() {
        let path = fillet(&[(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (1.0, 1.0, 0.0)], 0.1, 3);
        assert_eq!(path.len(), 2 + 4);
        assert_eq!((path[0], path[5]), ((0.0, 0.0, 0.0), (1.0, 1.0, 0.0)));
        // a square corner rounded by 0.1 starts and ends 0.1 from it
        assert!((path[1].0 - 0.9).abs() < 1e-6 && (path[4].1 - 0.1).abs() < 1e-6);
        // a straight run has nothing to round
        let straight = fillet(&[(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 0.0, 0.0)], 0.1, 3);
        assert_eq!(straight.len(), 3);
    }

    #[test]
    fn a_hexagon_of_twelve_has_its_flats_in_and_its_corners_lifted() {
        let ring = _hex(0.2, 1.0, 12, Some(0.15), 0.03, 0.0);
        assert_eq!(ring.len(), 12);
        assert!((ring[0].x - 0.2).abs() < 1e-12 && (ring[0].z - 1.03).abs() < 1e-12);
        assert!(((ring[1].x * ring[1].x + ring[1].y * ring[1].y).sqrt() - 0.15).abs() < 1e-12);
        assert!((ring[1].z - 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_far_songbird_has_no_eyes() {
        const BODY: Mat = Mat::new("body", "#7A5A40", "satin");
        scene::reset();
        let frame = GameFrame::new(0.0);
        let with = |low: bool| {
            let o = Songbird {
                eye: Some(BODY),
                low,
                ..Songbird::default()
            };
            songbird(&frame, "bird", 0.0, 1.0, 0.0, BODY, BODY, BODY, o).len()
        };
        assert_eq!((with(false), with(true)), (8, 6));
    }
}
