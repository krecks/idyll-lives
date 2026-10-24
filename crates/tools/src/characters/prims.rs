//! Mesh primitives for the generated residents: lofts (cross-sections along a curve, with rounded
//! or flat ends; along a smooth spline they make bent capsules), ellipsoids, tori, sheets through
//! grids of points, seeded value noise. Each has a fixed vertex layout for given counts, so the
//! same call at every life stage gives the same topology (stage deltas need it). (prims.py's lathe,
//! transform and rot_y had no callers and are not ported.)
//!
//! Winding: every primitive is counter-clockwise seen from outside (`cross(b - a, c - a)` points
//! out, as `meshops::smooth_normals` expects); write.rs turns faces round for the game
//! (clockwise front faces).

use super::meshops::smooth_normals;
use super::np::{gradient, linspace, linspace_open, pairwise_sum, sign, PI, TAU};
use super::{M3, V3, v3};

/// A primitive: positions, normals, flat indices and its parameter `t` per vertex.
#[derive(Clone, Default)]
pub struct Prim {
    pub pos: Vec<V3>,
    pub nor: Vec<V3>,
    pub idx: Vec<usize>,
    pub t: Vec<f64>,
}

/// Unit side vector U (towards `hint`, square to t) and V = t x U, so U x V = t.
pub fn frame(t: V3, hint: V3) -> (V3, V3) {
    let mut u = hint - t * hint.dot(&t);
    if u.norm() < 1e-6 {
        u = t.cross(&if t[0].abs() < 0.9 { v3(1.0, 0.0, 0.0) } else { v3(0.0, 1.0, 0.0) });
    }
    u /= u.norm();
    (u, t.cross(&u))
}

/// Stacked rings (k rings of `s`), ordered along the axis, closed around; `start` / `end` add a
/// pole vertex fanned to the first / last ring. Returns pos, idx and the ring index of each vertex.
pub fn rings_mesh(rings: &[Vec<V3>], start: Option<V3>, end: Option<V3>) -> (Vec<V3>, Vec<usize>, Vec<f64>) {
    let k = rings.len();
    let s = rings[0].len();
    let mut pos: Vec<V3> = rings.iter().flatten().copied().collect();
    let mut ring: Vec<f64> = (0..k).flat_map(|i| std::iter::repeat_n(i as f64, s)).collect();
    let mut idx = Vec::new();
    for i in 0..k - 1 {
        for j in 0..s {
            let j1 = (j + 1) % s;
            let (a, b, c, d) = (i * s + j, i * s + j1, (i + 1) * s + j1, (i + 1) * s + j);
            idx.extend([a, b, c, a, c, d]);
        }
    }
    let mut n = k * s;
    if let Some(p) = start {
        pos.push(p);
        ring.push(-1.0);
        for j in 0..s {
            idx.extend([n, (j + 1) % s, j]);
        }
        n += 1;
    }
    if let Some(p) = end {
        pos.push(p);
        ring.push(k as f64);
        let base = (k - 1) * s;
        for j in 0..s {
            idx.extend([n, base + j, base + (j + 1) % s]);
        }
    }
    (pos, idx, ring)
}

/// How `loft` ends.
#[derive(Clone, Copy)]
pub struct Caps {
    /// Rings per rounded end (0: open).
    pub rings: usize,
    pub start: bool,
    pub end: bool,
}

pub const OPEN: Caps = Caps { rings: 0, start: true, end: true };

pub const fn caps(rings: usize) -> Caps {
    Caps { rings, start: true, end: true }
}

/// A tube along `points` with elliptic cross-sections of semi-axes `rx` (along the side vector,
/// towards `hint`) and `rz` (the other way), and rounded ends. `t` is the axial parameter (0 at
/// the first point, 1 at the last; the caps reach a little beyond). (The Python's `power` and
/// `twist` were never used: the sections are ellipses.)
pub fn loft(points: &[V3], rx: &[f64], rz: &[f64], segs: usize, hint: V3, cp: Caps) -> Prim {
    let p = points;
    let k = p.len();
    let rx: Vec<f64> = if rx.len() == 1 { vec![rx[0]; k] } else { rx.to_vec() };
    let rz: Vec<f64> = if rz.len() == 1 { vec![rz[0]; k] } else { rz.to_vec() };
    let tg: Vec<V3> = gradient(p).iter().map(|t| t / t.norm()).collect();
    let seg: Vec<f64> = (0..k - 1).map(|i| (p[i + 1] - p[i]).norm()).collect();
    let l = pairwise_sum(&seg).max(1e-9);
    let mut tt = vec![0.0];
    let mut acc = 0.0;
    for s in &seg {
        acc += s;
        tt.push(acc);
    }
    let tt: Vec<f64> = tt.iter().map(|v| v / l).collect();
    let th = linspace_open(0.0, TAU, segs);
    let (cs, ss): (Vec<f64>, Vec<f64>) = th.iter().map(|&a| (sign(a.cos()) * a.cos().abs(), sign(a.sin()) * a.sin().abs())).unzip();
    let frames: Vec<(V3, V3)> = (0..k).map(|i| frame(tg[i], hint)).collect();
    let ring = |i: usize, scale: f64, shift: f64| -> Vec<V3> {
        let (u, v) = frames[i];
        (0..segs).map(|j| p[i] + tg[i] * shift + (u * cs[j] * rx[i] + v * ss[j] * rz[i]) * scale).collect()
    };
    let (mut rings, mut params) = (Vec::new(), Vec::new());
    let caps = cp.rings;
    if cp.start && caps > 0 {
        for q in (1..=caps).rev() {
            let a = q as f64 / (caps + 1) as f64 * PI / 2.0;
            let r = rx[0].min(rz[0]);
            rings.push(ring(0, a.cos(), -a.sin() * r));
            params.push(-a.sin() * r / l);
        }
    }
    for (i, &t0) in tt.iter().enumerate() {
        rings.push(ring(i, 1.0, 0.0));
        params.push(t0);
    }
    if cp.end && caps > 0 {
        for q in 1..=caps {
            let a = q as f64 / (caps + 1) as f64 * PI / 2.0;
            let r = rx[k - 1].min(rz[k - 1]);
            rings.push(ring(k - 1, a.cos(), a.sin() * r));
            params.push(1.0 + a.sin() * r / l);
        }
    }
    let start = (cp.start && caps > 0).then(|| p[0] - tg[0] * rx[0].min(rz[0]));
    let end = (cp.end && caps > 0).then(|| p[k - 1] + tg[k - 1] * rx[k - 1].min(rz[k - 1]));
    let (pos, idx, ri) = rings_mesh(&rings, start, end);
    let np_ = params.len();
    let t = ri
        .iter()
        .map(|&r| if r < 0.0 { params[0] - 0.02 } else if r >= np_ as f64 { params[np_ - 1] + 0.02 } else { params[r as usize] })
        .collect();
    let nor = smooth_normals(&pos, &idx);
    Prim { pos, nor, idx, t }
}

/// An ellipsoid of `lat` rings x `lon` segments, with poles. `t` is 0 at the top, 1 at the bottom.
/// (Every caller wanted the whole one; the Python's `top` / `bottom` band was never used.)
pub fn ellipsoid(c: V3, r: V3, lat: usize, lon: usize) -> Prim {
    // Rings from the bottom up (the loft axis is +y, so faces wind outward).
    let lo = PI - PI / (lat + 1) as f64;
    let hi = PI / (lat + 1) as f64;
    let phi = linspace(lo, hi, lat);
    let th = linspace_open(0.0, TAU, lon);
    let rings: Vec<Vec<V3>> = phi
        .iter()
        .map(|&p| th.iter().map(|&t| c + v3(p.sin() * t.sin() * r[0], p.cos() * r[1], p.sin() * t.cos() * r[2])).collect())
        .collect();
    let (pos, idx, _) = rings_mesh(&rings, Some(c - v3(0.0, r[1], 0.0)), Some(c + v3(0.0, r[1], 0.0)));
    let t = pos.iter().map(|p| 1.0 - (p.y - (c.y - r[1])) / (2.0 * r[1])).collect();
    let nor = smooth_normals(&pos, &idx);
    Prim { pos, nor, idx, t }
}

pub fn rot_x(a: f64) -> M3 {
    let (c, s) = (a.cos(), a.sin());
    M3::new(1.0, 0.0, 0.0, 0.0, c, -s, 0.0, s, c)
}

/// `p @ R.T` of one point.
pub fn rot(r: &M3, p: V3) -> V3 {
    v3(p.x * r[(0, 0)] + p.y * r[(0, 1)] + p.z * r[(0, 2)], p.x * r[(1, 0)] + p.y * r[(1, 1)] + p.z * r[(1, 2)], p.x * r[(2, 0)] + p.y * r[(2, 1)] + p.z * r[(2, 2)])
}

/// Faces of a sheet through a grid of points (rows, cols): counter-clockwise seen from the side
/// where the columns run right and the rows run up (`wrap`: the last column joins the first).
pub fn grid(p: &[Vec<V3>], wrap: bool) -> (Vec<V3>, Vec<usize>) {
    let (r, c) = (p.len(), p[0].len());
    let mut idx = Vec::new();
    for i in 0..r - 1 {
        for j in 0..(if wrap { c } else { c - 1 }) {
            let j1 = (j + 1) % c;
            let (a, b, cc, d) = (i * c + j, i * c + j1, (i + 1) * c + j1, (i + 1) * c + j);
            idx.extend([a, b, cc, a, cc, d]);
        }
    }
    (p.iter().flatten().copied().collect(), idx)
}

/// `idx` turned round if its faces mostly point against `out` (per vertex directions).
pub fn orient(pos: &[V3], idx: Vec<usize>, out: &[V3]) -> Vec<usize> {
    let mut prods = Vec::with_capacity(idx.len());
    for t in idx.chunks(3) {
        let f = (pos[t[1]] - pos[t[0]]).cross(&(pos[t[2]] - pos[t[0]]));
        let o = out[t[0]] + out[t[1]] + out[t[2]];
        prods.extend([f.x * o.x, f.y * o.y, f.z * o.z]);
    }
    if pairwise_sum(&prods) >= 0.0 { idx } else { idx.chunks(3).flat_map(|t| [t[0], t[2], t[1]]).collect() }
}

/// `n` points along a Catmull-Rom curve through `points`, evenly spaced in parameter.
pub fn spline(points: &[V3], n: usize) -> Vec<V3> {
    let k = points.len();
    let mut q = vec![points[0] * 2.0 - points[1]];
    q.extend(points);
    q.push(points[k - 1] * 2.0 - points[k - 2]);
    linspace(0.0, (k - 1) as f64, n)
        .iter()
        .map(|&u| {
            let i = (u as usize).min(k - 2);
            let t = u - i as f64;
            let (p0, p1, p2, p3) = (q[i], q[i + 1], q[i + 2], q[i + 3]);
            ((p1 * 2.0) + (p2 - p0) * t + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t * t + (p1 * 3.0 - p0 - p2 * 3.0 + p3) * t.powf(3.0)) * 0.5
        })
        .collect()
}

/// A torus round `axis` through `c`: ring radius `big`, tube radius `r`.
pub fn torus(c: V3, big: f64, r: f64, segs: usize, sides: usize, axis: V3) -> Prim {
    let ax = axis / axis.norm();
    let (u, v) = frame(ax, if ax[2].abs() < 0.9 { v3(0.0, 0.0, 1.0) } else { v3(1.0, 0.0, 0.0) });
    let a = linspace_open(0.0, TAU, segs);
    let bb = linspace_open(0.0, TAU, sides);
    let rz = r;
    let d: Vec<V3> = a.iter().map(|&a| u * a.cos() + v * a.sin()).collect();
    let p: Vec<Vec<V3>> = d.iter().map(|di| bb.iter().map(|&b| c + di * (big + b.cos() * r) + ax * b.sin() * rz).collect()).collect();
    let (pos, mut idx) = grid(&p, true);
    // Close the ring of tubes too.
    for l in 0..sides {
        let j1 = (l + 1) % sides;
        let (a_, b_, c_, d_) = ((segs - 1) * sides + l, (segs - 1) * sides + j1, j1, l);
        idx.extend([a_, b_, c_, a_, c_, d_]);
    }
    let out: Vec<V3> = pos.iter().enumerate().map(|(i, q)| q - (c + d[i / sides] * big)).collect();
    let idx = orient(&pos, idx, &out);
    assert_eq!(pos.len(), segs * sides);
    let nor = smooth_normals(&pos, &idx);
    Prim { pos, nor, idx, t: Vec::new() }
}

/// Seeded smooth value noise (-1..1) at `p`, features about 1 / `scale` apart (int64 hashing,
/// wrapping as numpy's does).
pub fn noise3(p: V3, seed: i64, scale: f64) -> f64 {
    let q = p * scale;
    let i = q.map(|v| v.floor() as i64);
    let f = v3(q.x - i.x as f64, q.y - i.y as f64, q.z - i.z as f64);
    let f = f.map(|f| f * f * (3.0 - 2.0 * f));
    let h = |x: i64, y: i64, z: i64| -> f64 {
        let v = x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663) ^ z.wrapping_mul(83492791) ^ seed.wrapping_mul(2654435761);
        let v = (v ^ (v >> 13)).wrapping_mul(1274126177);
        ((v ^ (v >> 16)) & 0xFFFF) as f64 / 32767.5 - 1.0
    };
    let mut out = 0.0;
    for dx in 0..2 {
        for dy in 0..2 {
            for dz in 0..2 {
                let w = (if dx == 1 { f.x } else { 1.0 - f.x }) * (if dy == 1 { f.y } else { 1.0 - f.y }) * (if dz == 1 { f.z } else { 1.0 - f.z });
                out += w * h(i.x + dx, i.y + dy, i.z + dz);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_matches_numpy() {
        // prims.noise3(np.array([[0.31, -1.7, 2.2], [5.5, 0.0, -3.25]]), seed, scale) in Python.
        let a = noise3(v3(0.31, -1.7, 2.2), 3, 7.0);
        let b = noise3(v3(5.5, 0.0, -3.25), 12, 40.0);
        assert_eq!([a, b], [-0.26219018415968975, 0.8859540703440909]);
    }
}
