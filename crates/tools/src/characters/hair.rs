//! The residents' hair, built from primitives over the head interface. Contract:
//!
//! - `build_hair(family, stage, head, joints)`: `hair.<style>` for every style but `none`, and
//!   `hair.beard` where the family has it; each pos, nor, idx (CCW out), dense, fields, tone (y:
//!   shade deep in the hair, z: clump bands 0..1).
//!
//! Every style starts from a skull cap: the scalp inside the hair's edge (the head interface's
//! hairline, smoothed round and brought a little lower over the forehead and temples), cast onto
//! the head's distance field, rounded part way out to the skull ellipsoid and pushed out by a
//! thickness that varies over the head (volume on top, thin at the edge). On it go curtains
//! (sheets round the head with a thickness and a curled hem), fringes that follow the forehead as
//! drawn, lofted lobes and bent capsules, ellipsoid petals, tori and noise lumps. The beard is a
//! shell over the jaw, chin and upper lip with a hole round the lips; its morphs copy the lips'
//! (face::follow_mouth). Each style has one fixed layout, so the same calls give the same topology
//! at every life stage.
//!
//! Skinning: Head, blending below the jaw to neck_01 and spine_03 by height (tails and braids
//! along their length), so hair that falls past the jaw follows the shoulders when the head turns.
//! Nothing comes within 2.6 mm of the head.

use super::contract::{BEARD_FAMILIES, HAIR_STYLES};
use super::head::HeadFrame;
use super::meshops::smooth_normals;
use super::np::{PI, Periodic, TAU, clip, interp, linspace, linspace_open, pairwise_sum, py_mod, radians, smoothstep};
use super::prims::{caps, ellipsoid, grid, loft, noise3, orient, rings_mesh, rot, rot_z, spline, torus};
use super::rig::{NB, b};
use super::{Mesh, V3, v3};
use crate::pcg64::Pcg64;

/// Nearest the hair comes to the skull (m); the check wants more than 2 mm.
const CLEAR: f64 = 0.0026;
/// How far the scalp the hair sits on is rounded out from the skull to its ellipsoid (0..1).
const ROUND: f64 = 0.5;

/// Points on the skull by direction (azimuth a round the vertical axis, 0 = front, + = the
/// resident's right; polar angle phi from the top, in the skull ellipsoid's normalised frame);
/// `cranium`: no further out than the skull ellipsoid (else the face and jaw as drawn).
pub struct Scalp<'a> {
    h: &'a HeadFrame,
    cranium: bool,
    hl: Periodic,
}

impl<'a> Scalp<'a> {
    pub fn new(h: &'a HeadFrame, cranium: bool) -> Self {
        let q: Vec<V3> = h.hairline.iter().map(|p| (p - h.centre).component_div(&h.radii)).collect();
        let a: Vec<f64> = q.iter().map(|q| q.x.atan2(q.z)).collect();
        let phi: Vec<f64> = q.iter().map(|q| clip(q.y / q.norm(), -1.0, 1.0).acos()).collect();
        let o = super::np::argsort(&a);
        // The hair's own edge: the head's hairline smoothed round (no point at the nape) and
        // brought a little lower over the forehead and temples (fuller hair, a softer face).
        let fa = linspace_open(-PI, PI, 180);
        let per = Periodic::new(&o.iter().map(|&i| a[i]).collect::<Vec<_>>(), &o.iter().map(|&i| phi[i]).collect::<Vec<_>>(), TAU);
        let fp: Vec<f64> = fa.iter().map(|&x| per.at(x)).collect();
        let mut k: Vec<f64> = fa.iter().map(|&x| (-0.5 * (x.abs().min(TAU - x.abs()) / radians(16.0)).powi(2)).exp()).collect();
        let ks = pairwise_sum(&k);
        for v in &mut k {
            *v /= ks;
        }
        // Circular convolution with the kernel centred on index 0 (np.fft's ifftshift, then fft
        // products; a direct sum here).
        let n = fa.len();
        let kk: Vec<f64> = (0..n).map(|i| k[(i + n / 2) % n]).collect();
        let mut fp: Vec<f64> = (0..n).map(|i| (0..n).fold(0.0, |s, j| s + fp[j] * kk[(i + n - j) % n])).collect();
        for (v, &x) in fp.iter_mut().zip(&fa) {
            let t = x.abs();
            *v += radians(5.0) * smoothstep(radians(70.0), 0.0, t) + radians(8.0) * (-((t - radians(70.0)) / radians(28.0)).powi(2)).exp();
        }
        Scalp { h, cranium, hl: Periodic::new(&fa, &fp, TAU) }
    }

    /// Polar angle of the hairline at azimuth a.
    pub fn hairline(&self, a: f64) -> f64 {
        self.hl.at(py_mod(a + PI, TAU) - PI)
    }

    pub fn dirs(a: f64, phi: f64) -> V3 {
        v3(phi.sin() * a.sin(), phi.cos(), phi.sin() * a.cos())
    }

    /// The skull surface point and normal at (a, phi) (cast along the normalised direction).
    pub fn surface(&self, a: f64, phi: f64) -> (V3, V3) {
        let h = self.h;
        let d = Scalp::dirs(a, phi).component_mul(&h.radii);
        // Over the ears no further out than the skull ellipsoid (which holds the cranium), so the
        // ears don't push the hair about; `finish` keeps it off them.
        let hi0 = if self.cranium {
            // Past the ellipsoid only away from the ears (the brow and forehead may stand out of it).
            let aa = (py_mod(a + PI, TAU) - PI).abs();
            let ear = (-((aa - PI / 2.0) / 0.6).powi(2)).exp() * (-((phi - PI / 2.0) / 0.5).powi(2)).exp();
            1.0 + 0.6 * (1.0 - smoothstep(0.15, 0.5, ear))
        } else {
            2.5
        };
        // March out to the first step outside (the field far from the head can't be trusted), then
        // bisect within that step.
        let steps = linspace(0.2, 2.5, 47);
        let k = steps.iter().position(|&s| s >= hi0 || h.sdf(h.centre + d * s) >= 0.0).unwrap_or(0);
        let mut hi = steps[k].min(hi0);
        let mut lo = steps[k.max(1) - 1];
        for _ in 0..20 {
            let mid = (lo + hi) / 2.0;
            if h.sdf(h.centre + d * mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let mut t = (lo + hi) / 2.0;
        if self.cranium {
            // Hair doesn't follow every bump of the skull: part way out to the smooth ellipsoid.
            t += ROUND * (1.0 - t).max(0.0);
        }
        let p = h.centre + d * t;
        let n = (p - h.centre).component_div(&h.radii.component_mul(&h.radii));
        let mut n = n / n.norm();
        if !self.cranium {
            // The face's own normal (measured over a few mm: the field is rough), unless it is wild.
            let g = grad(h, p, 0.003);
            let g = if g.dot(&n) > 0.3 { g } else { n };
            n = g + n * 0.5;
            n /= n.norm();
        }
        (p, n)
    }

    pub fn point(&self, a: f64, phi: f64, off: f64) -> V3 {
        let (p, n) = self.surface(a, phi);
        p + n * off
    }
}

/// The least distance to the head (sdf) at and round a point.
fn near_sdf(h: &HeadFrame, p: V3) -> f64 {
    let r = 0.0004;
    let mut d = h.sdf(p);
    for e in [v3(r, 0.0, 0.0), v3(0.0, r, 0.0), v3(0.0, 0.0, r), v3(-r, 0.0, 0.0), v3(0.0, -r, 0.0), v3(0.0, 0.0, -r)] {
        d = d.min(h.sdf(p + e));
    }
    d
}

fn grad(h: &HeadFrame, p: V3, e: f64) -> V3 {
    let mut g = V3::zeros();
    for k in 0..3 {
        let mut d = V3::zeros();
        d[k] = e;
        g[k] = (h.sdf(p + d) - h.sdf(p - d)) / (2.0 * e);
    }
    g / g.norm().max(1e-9)
}

/// A component of a style: faces, how far each vertex follows the body rather than the head
/// (`f`, -1: by height, see `finish`), clump bands (`z`) and shade (`y`).
#[derive(Clone, Default)]
pub struct Comp {
    pos: Vec<V3>,
    idx: Vec<usize>,
    f: Vec<f64>,
    z: Vec<f64>,
    y: Vec<f64>,
}

/// A component with faces turned to point along `out` (if given).
fn comp(pos: Vec<V3>, idx: Vec<usize>, out: Option<Vec<V3>>, z: Option<Vec<f64>>, y: Option<Vec<f64>>, follow: Option<Vec<f64>>) -> Comp {
    let n = pos.len();
    let idx = match out {
        Some(o) => orient(&pos, idx, &o),
        None => idx,
    };
    Comp { f: follow.unwrap_or_else(|| vec![-1.0; n]), z: z.unwrap_or_else(|| vec![0.5; n]), y: y.unwrap_or_else(|| vec![0.0; n]), pos, idx }
}

type Field<'a> = &'a dyn Fn(f64, f64) -> f64;

/// The skull cap: rows from the crown down to the hairline (`limit(a)` overrides it), pushed out
/// by `thick(a, u)` (m; u = 0 at the crown .. 1 at the hairline), tapering to `edge` at the rim.
fn cap(sc: &Scalp, thick: Field, rows: usize, cols: usize, limit: Option<&dyn Fn(f64) -> f64>, edge: f64, bands: Option<Field>) -> Comp {
    let h = sc.h;
    let a = linspace_open(-PI, PI, cols);
    let lim: Vec<f64> = a.iter().map(|&a| match limit {
        Some(l) => l(a),
        None => sc.hairline(a),
    }).collect();
    let us = linspace(1.0, 1.0 / rows as f64, rows);
    let mut p = Vec::new();
    let (mut au, mut uu) = (Vec::new(), Vec::new());
    for &u in &us {
        let mut ring = Vec::new();
        for (j, &aj) in a.iter().enumerate() {
            let phi = u * lim[j];
            let t = thick(aj, u);
            let t = t + (edge - t) * smoothstep(0.82, 1.0, u);
            ring.push(sc.point(aj, phi, t.max(edge)));
            au.push(aj);
            uu.push(u);
        }
        p.push(ring);
    }
    let top = sc.point(0.0, 0.0, thick(0.0, 0.0));
    let (pos, idx, _) = rings_mesh(&p, None, Some(top));
    au.push(0.0);
    uu.push(0.0);
    let z: Vec<f64> = au.iter().zip(&uu).map(|(&a, &u)| match bands {
        Some(bf) => bf(a, u),
        None => 0.5 + 0.5 * (a * 13.0 + 2.0 * (u * 3.0).sin()).sin(),
    }).collect();
    let y = uu.iter().map(|&u| 0.25 * smoothstep(0.85, 1.0, u)).collect();
    let out = pos.iter().map(|q| q - h.centre).collect();
    comp(pos, idx, Some(out), Some(z), Some(y), None)
}

/// Horizontal scale of the skull ellipsoid's section at height y (1 below its widest).
fn ell(h: &HeadFrame, y: f64) -> f64 {
    let q = (y - h.centre[1]) / h.radii[1];
    if q > 0.0 { clip(1.0 - q * q, 0.0, 1.0).sqrt() } else { 1.0 }
}

/// A curtain's shape (see `curtain`).
struct Curtain<'a> {
    a0: f64,
    a1: f64,
    top: &'a dyn Fn(f64) -> f64,
    bottom: &'a dyn Fn(f64) -> f64,
    off: f64,
    cols: usize,
    rows: usize,
    flare: f64,
    thick: f64,
    curl: f64,
    back: Option<&'a dyn Fn(f64) -> f64>,
    bands: f64,
    wave: f64,
    seed: i64,
}

/// A sheet of hair hanging round the head between azimuths a0..a1, from height top(a) down to
/// bottom(a) (m), `off` (m) out from the skull ellipsoid, flaring out by `flare` (fraction) towards
/// the hem; `thick` (m) thick with a hem curling in by `curl`. `back(y)` (optional): the furthest
/// forward the sheet's back may come at height y (so it clears the shoulders and back).
fn curtain(sc: &Scalp, c: Curtain) -> Comp {
    let h = sc.h;
    let a = linspace(c.a0, c.a1, c.cols);
    let tt: Vec<f64> = a.iter().map(|&a| (c.top)(a)).collect();
    let bt: Vec<f64> = a.iter().map(|&a| (c.bottom)(a)).collect();
    let us = linspace(0.0, 1.0, c.rows);
    let mut layers = Vec::new();
    for (layer, o_add) in [(0, 0.0), (1, -c.thick)] {
        let mut pl = Vec::new();
        for &u in &us {
            let row: Vec<V3> = (0..c.cols)
                .map(|j| {
                    let y = tt[j] + (bt[j] - tt[j]) * u;
                    let v = ell(h, y);
                    let f = 1.0 + c.flare * u.powf(1.5);
                    let w = c.wave * (a[j] * 7.0 + c.seed as f64).sin() * u;
                    let rx = (h.radii[0] * v + c.off + o_add + w) * f;
                    let rz = (h.radii[2] * v + c.off + o_add + w) * f;
                    let mut p = v3(h.centre[0] + a[j].sin() * rx, y, h.centre[2] + a[j].cos() * rz);
                    if let Some(bk) = c.back {
                        p[2] = p[2].min(bk(y) + (if layer == 0 { 0.0 } else { c.thick }));
                    }
                    p
                })
                .collect();
            pl.push(row);
        }
        layers.push(pl);
    }
    let (outer, inner) = (&layers[0], &layers[1]);
    let rows = c.rows;
    // Hem: the outer layer's last row curls in to the inner one.
    let hem: Vec<V3> = (0..c.cols).map(|j| outer[rows - 1][j] + (inner[rows - 1][j] - outer[rows - 1][j]) * 0.5 + v3(0.0, c.thick * c.curl, 0.0)).collect();
    let mut all: Vec<Vec<V3>> = outer.clone();
    all.push(hem.iter().map(|q| q - v3(0.0, c.thick * 0.4, 0.0)).collect());
    all.extend(inner.iter().rev().cloned());
    let (pos, idx) = grid(&all, false);
    let k = all.len();
    let out: Vec<V3> = pos
        .iter()
        .enumerate()
        .map(|(i, q)| {
            let side = if i / c.cols < rows + 1 { 1.0 } else { -1.0 };
            let cen = v3(h.centre[0], q.y, h.centre[2]);
            (q - cen) * side
        })
        .collect();
    let z = (0..k * c.cols).map(|i| 0.5 + 0.5 * (a[i % c.cols] * c.bands + noise3(pos[i] * 40.0, c.seed, 1.0) * 1.5).sin()).collect();
    let y = (0..k * c.cols).map(|i| {
        let r = i / c.cols;
        if r < rows { 0.0 } else if r == rows { 0.2 } else { 0.55 }
    }).collect();
    comp(pos, idx, Some(out), Some(z), Some(y), None)
}

/// A fringe's shape (see `fringe`).
struct Fringe<'a> {
    a0: f64,
    a1: f64,
    top: &'a dyn Fn(f64) -> f64,
    bottom: &'a dyn Fn(f64) -> f64,
    off: f64,
    cols: usize,
    rows: usize,
    thick: f64,
    flare: f64,
    bands: f64,
    seed: f64,
}

/// Hair falling over the forehead, on the face as drawn (`face`: a Scalp that isn't held to the
/// cranium): between azimuths a0..a1, from polar angle top(a) (under the cap) down to height
/// bottom(a) (m), `off` (m) off the skin (more by `flare` towards the hem), `thick` thick.
fn fringe(face: &Scalp, c: Fringe) -> Comp {
    let h = face.h;
    let a = linspace(c.a0, c.a1, c.cols);
    let t0: Vec<f64> = a.iter().map(|&a| (c.top)(a)).collect();
    let t1: Vec<f64> = (0..c.cols)
        .map(|j| {
            let want = (c.bottom)(a[j]);
            let (mut lo, mut hi) = (t0[j] + 0.02, radians(150.0));
            for _ in 0..24 {
                let mid = (lo + hi) / 2.0;
                if face.surface(a[j], mid).0[1] < want {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            (lo + hi) / 2.0
        })
        .collect();
    let us = linspace(0.0, 1.0, c.rows);
    let ph = |i: usize, j: usize| t0[j] + (t1[j] - t0[j]) * us[i];
    let o = |i: usize| c.off * (1.0 + c.flare * us[i].powf(1.5)) * 1.0;
    let outer: Vec<Vec<V3>> = (0..c.rows).map(|i| (0..c.cols).map(|j| face.point(a[j], ph(i, j), o(i))).collect()).collect();
    let inner: Vec<Vec<V3>> = (0..c.rows).rev().map(|i| (0..c.cols).map(|j| face.point(a[j], ph(i, j), (o(i) - c.thick).max(CLEAR + 0.0005))).collect()).collect();
    let r = c.rows;
    let hem: Vec<V3> = (0..c.cols).map(|j| (outer[r - 1][j] + inner[0][j]) / 2.0 + (outer[r - 1][j] - face.point(a[j], ph(r - 2, j), o(r - 2))) * 0.25).collect();
    let mut all = outer;
    all.push(hem);
    all.extend(inner);
    let (pos, idx) = grid(&all, false);
    let out: Vec<V3> = pos.iter().enumerate().map(|(i, q)| (q - h.centre) * (if i / c.cols < r + 1 { 1.0 } else { -1.0 })).collect();
    let n = pos.len();
    let z = (0..n).map(|i| 0.5 + 0.5 * (a[i % c.cols] * c.bands + c.seed).sin()).collect();
    let y = (0..n).map(|i| {
        let row = i / c.cols;
        if row < r { 0.0 } else if row == r { 0.2 } else { 0.55 }
    }).collect();
    comp(pos, idx, Some(out), Some(z), Some(y), None)
}

/// A flattened ellipsoid lying on the scalp at (a, phi): size (long, wide, thick) (m), its long
/// axis turned `turn` from straight down the head, tipped up by `lift` (rad).
#[allow(clippy::too_many_arguments)]
fn petal(sc: &Scalp, a: f64, phi: f64, size: V3, turn: f64, lift: f64, off: f64, seed: f64) -> Comp {
    let (p, n) = sc.surface(a, phi);
    let mut down = sc.point(a, phi + 0.05, 0.0) - p;
    down -= n * down.dot(&n);
    down /= down.norm();
    let side = n.cross(&down);
    let t = down * turn.cos() + side * turn.sin();
    let t = t * lift.cos() + n * lift.sin();
    let s = n.cross(&t);
    let s = s / s.norm();
    let nn = t.cross(&s);
    let m = ellipsoid(V3::zeros(), v3(size[1], size[0], size[2]), 4, 7);
    // Local x -> side, y -> along, z -> normal.
    let c = p + n * (off + size[2] * 0.6) + t * size[0] * 0.55;
    let pos: Vec<V3> = m.pos.iter().map(|q| v3(q.x * s.x + q.y * t.x + q.z * nn.x, q.x * s.y + q.y * t.y + q.z * nn.y, q.x * s.z + q.y * t.z + q.z * nn.z) + c).collect();
    let z = m.pos.iter().map(|q| clip(0.5 + 0.35 * (seed * 2.3).sin() + 0.25 * (q.x / size[1] * 3.0).cos(), 0.0, 1.0)).collect();
    let y = m.t.iter().map(|t| 0.15 * (1.0 - t)).collect();
    let out = pos.iter().map(|q| q - c).collect();
    comp(pos, m.idx, Some(out), Some(z), Some(y), None)
}

/// A bent capsule (lofted along a smooth curve through `points`); `hang` (a, b): it follows the
/// body more and more from a to b along its length (a tail).
#[allow(clippy::too_many_arguments)]
fn lobe(points: &[V3], ry: &[f64], rz: &[f64], segs: usize, hint: V3, bands: f64, seed: f64, y: f64, hang: Option<(f64, f64)>) -> Comp {
    let m = loft(points, ry, rz, segs, hint, caps(2));
    let n = m.pos.len();
    let z = (0..n).map(|i| 0.5 + 0.5 * ((i % segs) as f64 / segs as f64 * TAU * 2.0 + seed + m.t[i] * bands).sin()).collect();
    let follow = hang.map(|(a, b)| m.t.iter().map(|&t| smoothstep(a, b, t)).collect());
    comp(m.pos, m.idx, None, Some(z), Some(vec![y; n]), follow)
}

/// Noise lumps: vertices pushed out along their direction from the head's centre.
fn lumps(mut m: Comp, sc: &Scalp, amount: f64, scale: f64, seed: i64) -> Comp {
    let h = sc.h;
    let mut z = Vec::new();
    for p in &mut m.pos {
        let d = *p - h.centre;
        let d = d / d.norm();
        let k = noise3(*p, seed, scale) + 0.5 * noise3(*p, seed + 1, scale * 2.3);
        *p += d * (amount * k);
        z.push(clip(0.5 + 0.6 * k, 0.0, 1.0));
    }
    m.z = z;
    m
}

/// A torus as a component (`_tor`).
fn tor(t: super::prims::Prim) -> Comp {
    let n = t.pos.len();
    let z = (0..n).map(|i| 0.3 + 0.2 * (i as f64 * 0.7).sin()).collect();
    comp(t.pos, t.idx, None, Some(z), Some(vec![0.4; n]), None)
}

pub fn build_hair(family: &str, _stage: &str, head: &HeadFrame, p: &[V3; NB]) -> Vec<(String, Mesh)> {
    let h = head;
    let hh = h.hh;
    let sc = Scalp::new(h, true);
    let skin = Scalp::new(h, false);
    let brow_y = (h.brows[0][0].y + h.brows[0][1].y + h.brows[1][0].y + h.brows[1][1].y) / 4.0;
    let mut styles: Vec<(&str, Vec<Comp>)> = Vec::new();
    let deg = radians;
    let neck_y = p[b("neck_01")].y;
    let chin = h.chin_y;
    // The body's back (m, z) at height y, roughly: behind the neck, then the shoulder blades.
    let sp = p[b("spine_03")];
    let neck_z = p[b("neck_01")].z;
    let back = move |y: f64| interp(y, &[sp[1] - 0.25 * hh, sp[1], neck_y, chin], &[sp[2] - 0.5 * hh, sp[2] - 0.46 * hh, neck_z - 0.3 * hh, h.centre[2] - 0.2 * hh]) - 0.012;

    /// A thickness over the head: top at the crown, side over the ears, back_ behind.
    fn thick(top: f64, side: f64, back_: f64, nape: Option<f64>) -> impl Fn(f64, f64) -> f64 {
        move |a: f64, u: f64| {
            let ca = a.cos();
            let s = top + (side - top) * smoothstep(0.2, 0.9, u);
            let bk = match nape {
                None => back_,
                Some(nv) => back_ + (nv - back_) * smoothstep(0.5, 1.0, u),
            };
            s + (bk - s) * smoothstep(0.2, -0.8, ca) * smoothstep(0.1, 0.8, u)
        }
    }

    // Parted: volume on top, combed away from a side parting at x = +0.3 (the resident's right).
    let base = thick(0.05 * hh, 0.03 * hh, 0.028 * hh, Some(0.014 * hh));
    let parted = |a: f64, u: f64| {
        let t = base(a, u);
        let x = a.sin() * (u * 1.1).sin();
        let crease = (-((x - 0.3) / 0.07).powi(2)).exp() * smoothstep(-0.2, 0.6, a.cos()) * smoothstep(0.1, 0.4, u);
        let quiff = 0.022 * hh * (-(a / 0.6).powi(2)).exp() * smoothstep(0.35, 0.75, u) * smoothstep(1.0, 0.85, u);
        t * (1.0 - 0.6 * crease) + quiff
    };
    let part_bands = |a: f64, u: f64| {
        let x = a.sin() * (u * 1.1).sin();
        0.5 + 0.5 * ((x - 0.3) * 40.0 * super::np::sign(x - 0.3 + 1e-6) + u * 4.0).sin()
    };
    styles.push(("short", vec![cap(&sc, &parted, 10, 32, None, CLEAR, Some(&part_bands))]));

    // Crop: a thin cap, a squared hairline and a tiny fringe.
    let front = sc.hairline(0.0) + deg(3.0);
    let square = |a: f64| if a.abs() < deg(55.0) { front + deg(4.0) * (a.abs() / deg(55.0)).powf(6.0) } else { sc.hairline(a) };
    let crop_bands = |a: f64, u: f64| 0.5 + 0.5 * (a * 23.0 + u * 9.0).sin();
    styles.push(("crop", vec![cap(&sc, &thick(0.022 * hh, 0.016 * hh, 0.016 * hh, Some(0.011 * hh)), 8, 28, Some(&square), CLEAR, Some(&crop_bands))]));

    // Layered: a cap and three flared tiers, the lowest to the jaw.
    let mut lay = vec![cap(&sc, &thick(0.05 * hh, 0.04 * hh, 0.04 * hh, None), 9, 28, None, CLEAR, None)];
    for (k, (yt, yb, off, fl)) in [(0.72, 0.5, 0.04, 0.06), (0.58, 0.3, 0.033, 0.09), (0.44, 0.08, 0.026, 0.12)].into_iter().enumerate() {
        let span = deg(62.0 + 6.0 * k as f64);
        let (top, bot) = (chin + yt * hh, chin + yb * hh);
        lay.push(curtain(&sc, Curtain { a0: span, a1: TAU - span, top: &move |_| top, bottom: &move |_| bot, off: off * hh, cols: 15, rows: 5, flare: fl, thick: 0.01 * hh, curl: 0.5, back: None, bands: 17.0, wave: 0.0, seed: k as i64 }));
    }
    styles.push(("layered", lay));

    // Tousled: a cap and seeded petals pointing every which way, a few over the forehead.
    let mut rng = Pcg64::new(11);
    let mut pet = vec![cap(&sc, &thick(0.034 * hh, 0.028 * hh, 0.026 * hh, Some(0.016 * hh)), 8, 24, None, CLEAR, None)];
    for k in 0..22 {
        let a = if k > 3 { rng.uniform(-PI, PI) } else { rng.uniform(-0.7, 0.7) };
        let lim = sc.hairline(a);
        let phi = if k > 3 { rng.uniform(0.15, 0.85) * lim } else { lim * rng.uniform(0.75, 0.9) };
        let size = v3(rng.uniform(0.15, 0.21), rng.uniform(0.07, 0.095), rng.uniform(0.02, 0.026)) * hh;
        let turn = rng.uniform(-0.9, 0.9);
        let lift = rng.uniform(-0.12, 0.12);
        pet.push(petal(&sc, a, phi, size, turn, lift, 0.008 * hh, k as f64));
    }
    styles.push(("tousled", pet));

    // Side-swept: a cap and one fat lobe from a high side parting across the forehead.
    let mut sw = vec![cap(&sc, &thick(0.045 * hh, 0.03 * hh, 0.028 * hh, Some(0.014 * hh)), 9, 28, None, CLEAR, None)];
    let fa = [0.75, 0.35, 0.0, -0.4, -0.75, -1.05];
    let fk = [0.55, 0.78, 0.92, 0.98, 1.02, 1.08];
    let fo = [0.05, 0.05, 0.048, 0.045, 0.038, 0.028];
    let pts: Vec<V3> = (0..6).map(|i| sc.point(fa[i], sc.hairline(fa[i]) * fk[i], fo[i] * hh)).collect();
    let pts = spline(&pts, 9);
    let t9 = linspace(0.0, 1.0, 9);
    let ry: Vec<f64> = t9.iter().map(|&t| interp(t, &[0.0, 0.4, 1.0], &[0.05, 0.065, 0.03]) * hh).collect();
    let rz: Vec<f64> = t9.iter().map(|&t| interp(t, &[0.0, 0.5, 1.0], &[0.022, 0.028, 0.014]) * hh).collect();
    sw.push(lobe(&pts, &ry, &rz, 9, v3(0.0, 1.0, 0.3), 9.0, 0.0, 0.0, None));
    styles.push(("sideSwept", sw));

    // Afro: a big rounded cap with noise lumps.
    let round = |a: f64, u: f64| {
        // Out to a rounded shell 1.35 times the skull, a little up and back.
        let e = Scalp::dirs(a, u * sc.hairline(a)).component_mul(&h.radii);
        let e = e / e.norm();
        let q = e.component_div(&h.radii);
        let r_s = 1.0 / (q.x * q.x + q.y * q.y + q.z * q.z).sqrt();
        let rr = h.radii.component_mul(&v3(1.42, 1.3, 1.36));
        let w = e.component_div(&rr).map(|v| v.abs().powf(2.4));
        let sh = v3(0.0, 0.1, -0.08).component_mul(&h.radii);
        let r_t = 1.0 / (w.x + w.y + w.z).sqrt().powf(1.0 / 1.2) + (e.x * sh.x + e.y * sh.y + e.z * sh.z);
        (r_t - r_s).max(0.03 * hh)
    };
    let afro = cap(&sc, &round, 13, 34, None, CLEAR + 0.004 * hh, None);
    styles.push(("afro", vec![lumps(afro, &sc, 0.03 * hh, 7.0 / hh, 3)]));

    // Bob: a shell to the jaw with a curled-in hem, and a straight fringe.
    let mut bob = vec![cap(&sc, &thick(0.05 * hh, 0.04 * hh, 0.04 * hh, None), 9, 28, None, CLEAR, None)];
    let (bt, bb) = (chin + 0.72 * hh, chin + 0.06 * hh);
    bob.push(curtain(&sc, Curtain { a0: deg(58.0), a1: TAU - deg(58.0), top: &move |_| bt, bottom: &move |_| bb, off: 0.036 * hh, cols: 19, rows: 7, flare: 0.12, thick: 0.014 * hh, curl: 1.2, back: None, bands: 19.0, wave: 0.0, seed: 0 }));
    let under = |a: f64| sc.hairline(a) * 0.8;
    let bob_bot = |a: f64| brow_y + hh * (0.035 + 0.03 * (a / deg(64.0)).powi(2));
    bob.push(fringe(&skin, Fringe { a0: -deg(64.0), a1: deg(64.0), top: &under, bottom: &bob_bot, off: 0.03 * hh, cols: 13, rows: 4, thick: 0.012 * hh, flare: 0.15, bands: 21.0, seed: 2.0 }));
    styles.push(("bob", bob));

    // Long: a cap, the sides to the shoulders, the back to mid-back; a parted fringe framing the face.
    let mut lng = vec![cap(&sc, &thick(0.045 * hh, 0.035 * hh, 0.035 * hh, None), 9, 28, None, CLEAR, None)];
    let shoulder = p[b("clavicle_l")].y;
    let side_bot = |a: f64| interp((py_mod(a + PI, TAU) - PI).abs(), &[deg(70.0), deg(110.0), deg(150.0)], &[shoulder + 0.25 * hh, shoulder - 0.1 * hh, sp[1] - 0.5 * hh]);
    let lt = chin + 0.72 * hh;
    lng.push(curtain(&sc, Curtain { a0: deg(66.0), a1: TAU - deg(66.0), top: &move |_| lt, bottom: &side_bot, off: 0.034 * hh, cols: 21, rows: 9, flare: 0.18, thick: 0.014 * hh, curl: 0.8, back: Some(&back), bands: 21.0, wave: 0.004 * hh, seed: 0 }));
    for s in [-1.0, 1.0] {
        let (a0, a1) = if s > 0.0 { (deg(4.0), deg(70.0)) } else { (-deg(70.0), -deg(4.0)) };
        let bot = |a: f64| brow_y + hh * interp(a.abs(), &[deg(4.0), deg(35.0), deg(55.0), deg(72.0)], &[0.2, 0.08, -0.1, -0.3]);
        lng.push(fringe(&skin, Fringe { a0, a1, top: &under, bottom: &bot, off: 0.028 * hh, cols: 7, rows: 5, thick: 0.012 * hh, flare: 0.2, bands: 23.0, seed: 4.0 + s }));
    }
    styles.push(("long", lng));

    // Ponytail: a slicked cap, a band at the back of the head and a tapering tail.
    let b15 = |a: f64, _u: f64| 0.5 + 0.5 * (a * 15.0).sin();
    let mut pt = vec![cap(&sc, &thick(0.024 * hh, 0.018 * hh, 0.02 * hh, Some(0.012 * hh)), 9, 28, None, CLEAR, Some(&b15))];
    let tie = sc.point(PI, deg(95.0), 0.03 * hh);
    pt.push(tor(torus(tie + v3(0.0, 0.0, -0.012 * hh), 0.05 * hh, 0.02 * hh, 12, 6, v3(0.0, 0.25, -1.0))));
    let mut tail = [
        tie + v3(0.0, 0.0, -0.03 * hh),
        tie + v3(0.0, -0.08 * hh, -0.13 * hh),
        tie + v3(0.0, -0.3 * hh, -0.16 * hh),
        tie + v3(0.0, -0.6 * hh, -0.1 * hh),
        tie + v3(0.0, -0.85 * hh, -0.07 * hh),
    ];
    for q in &mut tail {
        q[2] = q[2].min(back(q[1]) - 0.035 * hh);
    }
    let tp = spline(&tail, 10);
    let t10 = linspace(0.0, 1.0, 10);
    let ry: Vec<f64> = t10.iter().map(|&t| interp(t, &[0.0, 0.25, 1.0], &[0.08, 0.105, 0.025]) * hh).collect();
    let rz: Vec<f64> = t10.iter().map(|&t| interp(t, &[0.0, 0.25, 1.0], &[0.07, 0.085, 0.02]) * hh).collect();
    pt.push(lobe(&tp, &ry, &rz, 10, v3(1.0, 0.0, 0.0), 14.0, 0.0, 0.0, Some((0.1, 0.9))));
    styles.push(("ponytail", pt));

    // Braid: a cap and a chain of nine overlapping lobes down the back.
    let mut br = vec![cap(&sc, &thick(0.03 * hh, 0.024 * hh, 0.026 * hh, Some(0.018 * hh)), 9, 28, None, CLEAR, Some(&b15))];
    let start = sc.point(PI, deg(120.0), 0.03 * hh);
    for k in 0..9 {
        let sg = if k % 2 == 0 { 1.0 } else { -1.0 };
        let y = start[1] - k as f64 * 0.095 * hh;
        let mut c = v3(0.012 * hh * sg, y, 0.0);
        c[2] = (back(y) - 0.045 * hh).min(start[2] - 0.01 * hh * k as f64);
        let r = 0.07 * hh * (1.0 - 0.045 * k as f64);
        let e = ellipsoid(c, v3(r, r * 1.05, r * 0.75), 4, 8);
        let rz = rot_z(0.45 * sg);
        let pos: Vec<V3> = e.pos.iter().map(|q| rot(&rz, q - c) + c).collect();
        let z = pos.iter().map(|q| 0.5 + 0.5 * ((q.y - c.y) / r * 3.0 + k as f64).sin()).collect();
        let yy = pos.iter().map(|q| 0.3 * (1.0 - (q.y - c.y).abs() / r)).collect();
        let n = pos.len();
        let out = pos.iter().map(|q| q - c).collect();
        br.push(comp(pos, e.idx, Some(out), Some(z), Some(yy), Some(vec![smoothstep(0.0, 6.0, k as f64); n])));
    }
    let mut knot = tor(torus(v3(0.0, start[1] - 9.1 * 0.095 * hh, back(start[1] - 9.0 * 0.095 * hh) - 0.045 * hh), 0.03 * hh, 0.012 * hh, 10, 5, v3(0.0, 1.0, 0.0)));
    knot.f = vec![1.0; knot.pos.len()];
    br.push(knot);
    styles.push(("braid", br));

    // Buns: a cap and two buns high on each side.
    let mut bun = vec![cap(&sc, &thick(0.026 * hh, 0.02 * hh, 0.022 * hh, Some(0.014 * hh)), 9, 28, None, CLEAR, Some(&b15))];
    for s in [-1i64, 1] {
        let c = sc.point(s as f64 * deg(118.0), deg(38.0), 0.13 * hh);
        let e = ellipsoid(c, v3(0.15, 0.14, 0.15) * hh, 7, 12);
        let out = e.pos.iter().map(|q| q - c).collect();
        let m = comp(e.pos, e.idx, Some(out), None, None, None);
        bun.push(lumps(m, &sc, 0.008 * hh, 14.0 / hh, 7 + s));
    }
    styles.push(("bun", bun));

    let mut names: Vec<&str> = styles.iter().map(|s| s.0).collect();
    names.push("none");
    assert!(HAIR_STYLES.iter().all(|s| names.contains(s)) && names.len() == HAIR_STYLES.len());
    let mut out: Vec<(String, Mesh)> = styles.into_iter().map(|(name, comps)| (format!("hair.{name}"), finish(&comps, h, p, false))).collect();
    if BEARD_FAMILIES.contains(&family) {
        out.push(("hair.beard".into(), finish(&[beard(&skin)], h, p, true)));
    }
    out
}

/// A shell over the jaw, chin and upper lip with a hole round the lips (cells inside it collapse
/// onto its edge).
fn beard(sc: &Scalp) -> Comp {
    let h = sc.h;
    let hh = h.hh;
    let (cols, rows) = (23, 12);
    let a = linspace(-radians(94.0), radians(94.0), cols);
    // Mouth direction (polar angle) and the hole round it.
    let qm = (h.mouth - h.centre).component_div(&h.radii);
    let phm = (qm[1] / qm.norm()).acos();
    let top: Vec<f64> = a.iter().map(|&a| interp(a.abs(), &[0.0, 22.0, 45.0, 70.0, 85.0, 94.0].map(radians), &[phm - 0.17, phm - 0.15, phm - 0.02, phm - 0.1, radians(96.0), radians(94.0)])).collect();
    // The lower edge: just under the jaw, rising to the jaw's corners at the sides.
    let bot: Vec<f64> = (0..cols)
        .map(|j| {
            let want = h.chin_y + hh * interp(a[j].abs(), &[0.0, 45.0, 80.0, 94.0].map(radians), &[-0.01, 0.02, 0.15, 0.22]);
            let (mut lo, mut hi) = (top[j] + 0.1, radians(178.0));
            for _ in 0..24 {
                let mid = (lo + hi) / 2.0;
                if sc.surface(a[j], mid).0[1] < want {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            (lo + hi) / 2.0
        })
        .collect();
    let us = linspace(0.0, 1.0, rows);
    // The hole: an ellipse in (side distance, polar angle) round the mouth.
    let wx = (h.mouth_w / 2.0 + 0.04 * hh) / h.radii[0];
    let mut pg = Vec::new();
    let (mut z, mut y) = (Vec::new(), Vec::new());
    for &u in &us {
        let mut row = Vec::new();
        for j in 0..cols {
            let aa = a[j];
            let phi = top[j] + (bot[j] - top[j]) * u;
            let hx = aa.sin() * phi.sin() / wx;
            let sy = if phi < phm { 0.1 } else { 0.13 };
            let hy = (phi - phm) / sy;
            let r = hx.hypot(hy);
            let inside = r < 1.0;
            let scale = if inside { 1.0 / r.max(1e-6) } else { 1.0 };
            let a2 = if inside { clip(hx * scale * wx / phi.sin().max(1e-3), -1.0, 1.0).asin() } else { aa };
            let phi2 = if inside { phm + hy * scale * sy } else { phi };
            let edge = smoothstep(1.0, 1.6, r).min(smoothstep(0.0, 0.2, u)).min(smoothstep(radians(94.0), radians(78.0), aa.abs()));
            let off = CLEAR + (0.02 * hh) * edge;
            row.push(sc.point(a2, phi2, off));
            z.push(0.5 + 0.5 * (aa * 26.0 + phi * 8.0).sin());
            y.push(0.2 * (1.0 - edge));
        }
        pg.push(row);
    }
    let (pos, idx) = grid(&pg, false);
    let out = pos.iter().map(|q| q - h.centre).collect();
    comp(pos, idx, Some(out), Some(z), Some(y), None)
}

/// The components merged, pushed clear of the head, skinned, with fields and tone.
fn finish(comps: &[Comp], h: &HeadFrame, p: &[V3; NB], head_only: bool) -> Mesh {
    let mut pos = Vec::new();
    let (mut idx, mut zz, mut yy, mut ff) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for c in comps {
        let n = pos.len();
        idx.extend(c.idx.iter().map(|&i| i + n));
        pos.extend(&c.pos);
        zz.extend(&c.z);
        yy.extend(&c.y);
        ff.extend(&c.f);
    }
    // Nothing closer than CLEAR to the head: push such vertices out along the field's gradient
    // (measured a little round each point, so the clearance holds after the positions are rounded
    // in the files, wherever the field is rough). Only the vertices pushed can still be near.
    let mut near: Vec<usize> = (0..pos.len()).collect();
    for _ in 0..24 {
        let d: Vec<(usize, f64)> = near.iter().map(|&i| (i, near_sdf(h, pos[i]))).filter(|&(_, d)| d < CLEAR).collect();
        if d.is_empty() {
            break;
        }
        for &(i, di) in &d {
            let g = grad(h, pos[i], 1e-3);
            let o = pos[i] - h.centre;
            let o = o / o.norm();
            let g = if g.dot(&o) > 0.2 { g } else { o };
            pos[i] += g * clip(CLEAR + 0.0004 - di, 0.0003, 0.01);
        }
        near = d.iter().map(|e| e.0).collect();
    }
    let n = pos.len();
    let nor = smooth_normals(&pos, &idx);
    let mut dense = vec![[0.0; NB]; n];
    let (hd, nk, s3) = (b("Head"), b("neck_01"), b("spine_03"));
    for i in 0..n {
        if head_only {
            dense[i][hd] = 1.0;
            continue;
        }
        let neck_y = p[nk].y;
        let (mut a, mut bb) = (smoothstep(h.chin_y + 0.05 * h.hh, neck_y, pos[i].y), smoothstep(neck_y, p[s3].y, pos[i].y));
        if ff[i] >= 0.0 {
            a = clip(ff[i] * 1.6, 0.0, 1.0);
            bb = smoothstep(0.4, 1.0, ff[i]);
        }
        dense[i][hd] = 1.0 - a;
        dense[i][nk] = a * (1.0 - bb);
        dense[i][s3] = a * bb;
    }
    let fields = vec![[0.1, 0.0, 0.0, 0.0]; n];
    let tone = (0..n).map(|i| [0.0, clip(yy[i], 0.0, 1.0), clip(zz[i], 0.0, 1.0), 0.0]).collect();
    Mesh { pos, idx, nor, dense, fields, tone, ..Mesh::default() }
}
