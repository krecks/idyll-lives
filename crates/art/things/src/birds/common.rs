//! What a bird is built from: closed meshes from points and faces coloured face by face, the body's fullness
//! along it, and the small parts (legs, toes, eyes).

use artkit::materials::Mat;
use artkit::math::{TAU, V3, cos, pow, sin, sqrt, v3};
use artkit::mesh::Mesh;
use artkit::py::fmod;
use artkit::scene::{self, Obj};

/// `P`: a bird's point (x to its left, f forward, u up) as a Blender point (forward is -y).
pub fn p(x: impl artkit::math::Num, f: impl artkit::math::Num, u: impl artkit::math::Num) -> V3 {
    v3(x.f(), -f.f(), u.f())
}

/// `_mesh`: a closed mesh from `verts` and `faces` [(indices, material)], its normals outwards, smooth.
pub fn mesh(name: &str, verts: Vec<V3>, faces: &[(Vec<u32>, Mat)]) -> Obj {
    let corners: Vec<Vec<u32>> = faces.iter().map(|(f, _)| f.clone()).collect();
    let mut m = Mesh::from_pydata(verts, &corners);
    // the materials in the order the faces first use them
    for (_, mat) in faces {
        m.slot(*mat);
    }
    for (face, (_, mat)) in m.faces.iter_mut().zip(faces) {
        face.mat = m.mats.iter().position(|x| x.name == mat.name).unwrap_or(0) as u16;
        face.smooth = true;
    }
    m.recalc_normals();
    scene::add(name, m)
}

/// `loft`: a closed tube through `rings` (each the same number of points, round the same way), closed at each end
/// by a point (`start`, `end`: a fan) or flat. `zone(i, j)` is the material of the face between ring i and i + 1 at
/// j (from point j to j + 1); the end faces take zone(-1, j) and zone(len(rings) - 1, j).
pub fn loft(
    name: &str,
    rings: &[Vec<V3>],
    zone: impl Fn(i64, usize) -> Mat,
    start: Option<V3>,
    end: Option<V3>,
) -> Obj {
    let n = rings[0].len();
    let mut verts: Vec<V3> = rings.iter().flatten().copied().collect();
    let mut faces: Vec<(Vec<u32>, Mat)> = Vec::new();
    let u = |x: usize| x as u32;
    for i in 0..rings.len() - 1 {
        for j in 0..n {
            let k = (j + 1) % n;
            faces.push((
                vec![
                    u(i * n + j),
                    u(i * n + k),
                    u((i + 1) * n + k),
                    u((i + 1) * n + j),
                ],
                zone(i as i64, j),
            ));
        }
    }
    let last = (rings.len() - 1) * n;
    let top = rings.len() as i64 - 1;
    match start {
        Some(s) => {
            verts.push(s);
            let s = u(verts.len() - 1);
            for j in 0..n {
                faces.push((vec![s, u((j + 1) % n), u(j)], zone(-1, j)));
            }
        }
        None => faces.push(((0..u(n)).rev().collect(), zone(-1, 0))),
    }
    match end {
        Some(e) => {
            verts.push(e);
            let e = u(verts.len() - 1);
            for j in 0..n {
                faces.push((vec![e, u(last + j), u(last + (j + 1) % n)], zone(top, j)));
            }
        }
        None => faces.push(((u(last)..u(last + n)).collect(), zone(top, 0))),
    }
    mesh(name, verts, &faces)
}

/// `octahedron`: a small eight-sided gem (an eye), `r` across its middle, flattened along x by `squash`.
pub fn octahedron(name: &str, at: V3, r: f64, material: Mat, squash: f64) -> Obj {
    let c = at;
    let v: Vec<V3> = [
        (r * squash, 0.0, 0.0),
        (-r * squash, 0.0, 0.0),
        (0.0, r, 0.0),
        (0.0, -r, 0.0),
        (0.0, 0.0, r),
        (0.0, 0.0, -r),
    ]
    .iter()
    .map(|&d| c + V3::from(d))
    .collect();
    let faces: Vec<(Vec<u32>, Mat)> = [
        [0, 2, 4],
        [2, 1, 4],
        [1, 3, 4],
        [3, 0, 4],
        [2, 0, 5],
        [1, 2, 5],
        [3, 1, 5],
        [0, 3, 5],
    ]
    .iter()
    .map(|f| (f.to_vec(), material))
    .collect();
    mesh(name, v, &faces)
}

/// The body's stations from rump (0) to neck (1) and how full it is at each: `BODY_T`/`BODY_K` place the joints (the
/// shoulder, the tail's base), as they always have; the surface is drawn through them smoothly, at `STATIONS`.
pub const BODY_T: [f64; 8] = [0.0, 0.08, 0.2, 0.38, 0.58, 0.76, 0.9, 1.0];
pub const BODY_K: [f64; 8] = [0.32, 0.64, 0.87, 1.0, 1.0, 0.9, 0.7, 0.52];
pub const STATIONS: [f64; 9] = [0.0, 0.05, 0.14, 0.28, 0.46, 0.64, 0.79, 0.9, 1.0];

/// `body_ring`: a body's ring of `n` points round from the top (degrees; 90 = the bird's left), a face centred on the
/// top and on the underside (face centres at 0, 360/n ...).
pub fn body_ring(n: usize) -> Vec<f64> {
    (0..n)
        .map(|j| 180.0 / n as f64 * (2 * j + 1) as f64)
        .collect()
}

/// `head_ring`: a head's ring: a face on the crown, then the upper side, the band over the eye, the band through it,
/// the cheek and the throat (each side mirrored), by its number of points (`HEAD_RINGS`).
pub fn head_ring(n: usize) -> Vec<f64> {
    let half: &[f64] = match n {
        10 => &[20.0, 45.0, 70.0, 98.0, 135.0],
        12 => &[14.0, 34.0, 54.0, 72.0, 98.0, 128.0],
        _ => panic!("a head is 10 or 12 points round, not {n}"),
    };
    let mut out = half.to_vec();
    out.extend(half.iter().rev().map(|d| 360.0 - d));
    out
}

/// `_mid`: the direction half way between two, degrees.
pub fn mid(a: f64, b: f64) -> f64 {
    if (a - b).abs() < 180.0 {
        fmod((a + b) / 2.0, 360.0)
    } else {
        fmod((a + b + 360.0) / 2.0, 360.0)
    }
}

/// `_side`: how far round a face centred at `deg` is from the top: 0 (top) to 180 (underneath).
pub fn side(deg: f64) -> f64 {
    180.0 - (180.0 - fmod(deg, 360.0)).abs()
}

/// `_interp`: `ys` through `xs` straight between them.
pub fn interp(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    if x <= xs[0] {
        return ys[0];
    }
    for i in 0..xs.len() - 1 {
        if x <= xs[i + 1] {
            let f = (x - xs[i]) / (xs[i + 1] - xs[i]);
            return ys[i] + (ys[i + 1] - ys[i]) * f;
        }
    }
    ys[ys.len() - 1]
}

/// `_ease`: `ys` through `xs` on a smooth curve that never overshoots them (monotone cubic Hermite): a body's
/// fullness without the kinks a straight line between its stations makes.
pub fn ease(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    if x <= xs[0] {
        return ys[0];
    }
    if x >= xs[xs.len() - 1] {
        return ys[ys.len() - 1];
    }
    let i = (0..xs.len() - 1).filter(|&k| xs[k] <= x).max().unwrap_or(0);
    let secant = |k: usize| (ys[k + 1] - ys[k]) / (xs[k + 1] - xs[k]);
    let slope = |k: usize| {
        // (flat at a station that is a peak or a trough, so the curve never bulges past its stations)
        if k == 0 || k == xs.len() - 1 {
            return secant(k.min(xs.len() - 2));
        }
        let (a, b) = (secant(k - 1), secant(k));
        if a * b <= 0.0 {
            0.0
        } else {
            2.0 * a * b / (a + b)
        }
    };
    let h = xs[i + 1] - xs[i];
    let f = (x - xs[i]) / h;
    let (m0, m1) = (slope(i) * h, slope(i + 1) * h);
    ys[i] * (2.0 * pow(f, 3.0) - 3.0 * pow(f, 2.0) + 1.0)
        + m0 * (pow(f, 3.0) - 2.0 * pow(f, 2.0) + f)
        + ys[i + 1] * (-2.0 * pow(f, 3.0) + 3.0 * pow(f, 2.0))
        + m1 * (pow(f, 3.0) - pow(f, 2.0))
}

/// `_reach`: how far from `base` along `d` (unit) is `length` from `a`; panics where no point ahead of `base` is
/// (a bird too short for its bill and body: its tail would run back through it).
pub fn reach(a: V3, base: V3, d: V3, length: f64) -> f64 {
    let w = base - a;
    let bq = 2.0 * w.dot(d);
    let cq = w.dot(w) - length * length;
    let disc = bq * bq - 4.0 * cq;
    let t = (-bq + sqrt(disc)) / 2.0;
    assert!(
        disc >= 0.0 && t > 0.0,
        "{length} m is too short: no tail reaches it from its base ({t} m)"
    );
    t
}

/// `_tube`: a three-sided rod (a leg).
pub fn tube(name: &str, points: &[V3], r0: f64, r1: f64, material: Mat) -> Obj {
    let pts = points;
    let t = (pts[pts.len() - 1] - pts[0]).normalized();
    let a = t.orthogonal().normalized();
    let b = t.cross(a);
    let rings: Vec<Vec<V3>> = pts
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let r = r0 + (r1 - r0) * i as f64 / (pts.len() - 1) as f64;
            (0..3)
                .map(|k| {
                    let k = k as f64;
                    p + (a * cos(k * TAU / 3.0) + b * sin(k * TAU / 3.0)) * r
                })
                .collect()
        })
        .collect();
    loft(name, &rings, |_, _| material, None, None)
}

/// `_toe`: a toe: three sides from `root` to a point at `tip`, lying on the perch (flat underneath).
pub fn toe(name: &str, root: V3, tip: V3, r: f64, material: Mat) -> Obj {
    let t = (tip - root).normalized();
    let side = t.cross(v3(0, 0, 1)).normalized();
    let ring = [
        root + side * r,
        root + v3(0.0, 0.0, r * 0.9),
        root - side * r,
    ];
    let ring: Vec<V3> = ring.iter().map(|&q| q + v3(0.0, 0.0, -r * 0.5)).collect();
    loft(name, &[ring], |_, _| material, None, Some(tip))
}
