//! The residents' hair, built from primitives over the head interface. Contract:
//!
//! - `build_hair(family, stage, head, joints)`: `hair.<style>` for every style but `none`, and
//!   `hair.beard` where the family has it; each pos, nor, idx (CCW out), dense, fields, tone (y:
//!   shade deep in the hair, z: clump bands 0..1; w: the occlusion, baked later in build.rs).
//!
//! Every style starts from a skull cap: the scalp inside the hair's edge (the head interface's
//! hairline, smoothed round and brought a little lower over the forehead and temples), cast onto
//! the head's distance field, rounded part way out to the skull ellipsoid and pushed out by a
//! thickness that varies over the head (volume on top, thin at the edge), in locks that leave the
//! crown in a whorl, with a parting and short tips past the hairline where the style has them. On
//! it go curtains (sheets with a thickness hanging from the scalp's widest, closed round with
//! rounded edges and hem, in locks with tips), fringes that follow the forehead as drawn (the
//! same), lofted locks over the face and scalp, bent capsules, braided strands, puffs, twisted
//! buns and tori. The hair is in closed shapes and locks so it breaks its outline and shades in
//! clumps (tone.z, light in a lock's middle). The beard is a shell over the jaw, chin and upper
//! lip with a hole round the lips, in locks; its morphs copy the lips' (face::follow_mouth). Each
//! style has one fixed layout, so the same calls give the same topology at every life stage.
//!
//! Skinning: Head, blending below the jaw to neck_01 and spine_03 by height (tails and braids
//! along their length), so hair that falls past the jaw follows the shoulders when the head turns.
//! Nothing comes within 2.6 mm of the head.

use super::contract::{BEARD_FAMILIES, HAIR_STYLES};
use super::head::HeadFrame;
use super::meshops::smooth_normals;
use super::np::{PI, Periodic, TAU, clip, interp, linspace, linspace_open, pairwise_sum, py_mod, radians, smoothstep};
use super::prims::{caps, ellipsoid, grid, loft, noise3, orient, rings_mesh, spline, torus};
use super::rig::{NB, b};
use super::{Mesh, V3, v3};
use crate::pcg64::Pcg64;

/// Nearest the hair comes to the skull (m); the check wants more than 2 mm.
const CLEAR: f64 = 0.0026;
/// How far the scalp the hair sits on is rounded out from the skull to its ellipsoid (0..1).
const ROUND: f64 = 0.5;
/// How far a cap's locks turn round the crown (rad): they leave it in a whorl.
const WHORL: f64 = 0.6;

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

/// How far into a lock column j of a sheet or cap is: 0 on the parting between two locks (columns
/// 4k), 1 in a lock's middle (4k + 2).
fn lock_at(j: usize) -> f64 {
    (PI * j as f64 / 4.0).sin().abs().powf(0.6)
}

/// A fixed 0..1 for lock k of a component (seed), so lengths and tips vary from lock to lock.
fn jitter(k: usize, seed: f64) -> f64 {
    let v = ((k as f64 + 1.0) * 12.9898 + seed * 78.233).sin() * 43758.5453;
    v - v.floor()
}

/// The skull cap's shape (see `cap`).
struct Cap<'a> {
    /// Thickness (m) by azimuth and u (0 at the crown .. 1 at the hairline).
    thick: Field<'a>,
    rows: usize,
    /// Columns round the head, a multiple of 4 (a lock is 4 columns wide).
    cols: usize,
    /// The hairline's polar angle by azimuth (else the scalp's).
    limit: Option<&'a dyn Fn(f64) -> f64>,
    /// Thickness at the rim (m).
    edge: f64,
    /// Locks combed down from the crown: the grooves between them this deep (a share of the
    /// thickness), and their tips this far past the hairline (rad of polar angle, by azimuth).
    ridge: f64,
    tips: Option<&'a dyn Fn(f64) -> f64>,
    /// A parting: its azimuth (the columns turn so a groove lies on it) and the u it starts at.
    part: Option<(f64, f64)>,
    /// Clump bands by azimuth and u (else the locks': light in their middles).
    bands: Option<Field<'a>>,
}

/// How far round from the front azimuth a is (rad, 0..pi).
fn round(a: f64) -> f64 {
    (py_mod(a + PI, TAU) - PI).abs()
}

/// The skull cap: rows from the crown down to the hairline, pushed out by the thickness, tapering to
/// `edge` at the rim; in locks, with a parting.
fn cap(sc: &Scalp, c: Cap) -> Comp {
    let h = sc.h;
    let step = TAU / c.cols as f64;
    let turn = c.part.map_or(0.0, |(ap, _)| ap - (((ap + PI) / (4.0 * step)).round() * 4.0 * step - PI));
    let a: Vec<f64> = linspace_open(-PI, PI, c.cols).iter().map(|&a| a + turn).collect();
    let lk: Vec<f64> = (0..c.cols).map(lock_at).collect();
    let lim: Vec<f64> = (0..c.cols)
        .map(|j| {
            let l = match c.limit {
                Some(l) => l(a[j]),
                None => sc.hairline(a[j]),
            };
            l + c.tips.map_or(0.0, |t| t(a[j]) * lk[j].powf(1.5))
        })
        .collect();
    // How near a column is to the parting (1 on it), from where it starts down to the hairline.
    let parting = |j: usize, u: f64| {
        c.part.map_or(0.0, |(ap, u0)| {
            let d = py_mod(a[j] - ap + PI, TAU) - PI;
            (-(d / (0.6 * step)).powi(2)).exp() * smoothstep(u0, u0 + 0.35, u)
        })
    };
    let us = linspace(1.0, 1.0 / c.rows as f64, c.rows);
    let mut p = Vec::new();
    let (mut z, mut y) = (Vec::new(), Vec::new());
    for &u in &us {
        let mut ring = Vec::new();
        for j in 0..c.cols {
            // The locks part from the crown in a whorl, the grooves between them deepening down to the hairline.
            // None over the ears (the cap is thinnest there).
            let ear = (-((round(a[j]) - PI / 2.0) / 0.5).powi(2)).exp() * smoothstep(0.5, 0.8, u);
            let groove = c.ridge * (1.0 - lk[j]) * smoothstep(0.25, 0.8, u) * (1.0 - ear);
            let pt = parting(j, u);
            let t = (c.thick)(a[j], u) * (1.0 - groove) * (1.0 - 0.8 * pt);
            let t = t + (c.edge - t) * smoothstep(0.82, 1.0, u);
            ring.push(sc.point(a[j] + WHORL * (1.0 - u).powi(2), u * lim[j], t.max(c.edge)));
            z.push(match c.bands {
                Some(bf) => bf(a[j], u),
                None => 0.5 + (0.6 * lk[j] * (0.7 + 0.3 * jitter(j / 4, 3.0)) - 0.3 + 0.1 * (u * 9.0 + j as f64).sin()) * smoothstep(0.1, 0.6, u),
            });
            y.push(0.25 * smoothstep(0.85, 1.0, u) + 0.5 * groove + 0.6 * pt);
        }
        p.push(ring);
    }
    let top = sc.point(0.0, 0.0, (c.thick)(0.0, 0.0));
    let (pos, idx, _) = rings_mesh(&p, None, Some(top));
    z.push(0.5);
    y.push(0.0);
    let out = pos.iter().map(|q| q - h.centre).collect();
    comp(pos, idx, Some(out), Some(z), Some(y), None)
}

/// Unit normals of a grid of points (rows, cols), turned to point along `out(p)`.
fn grid_normals(p: &[Vec<V3>], out: &dyn Fn(V3) -> V3) -> Vec<Vec<V3>> {
    let (r, c) = (p.len(), p[0].len());
    (0..r)
        .map(|i| {
            (0..c)
                .map(|j| {
                    let down = p[(i + 1).min(r - 1)][j] - p[i.saturating_sub(1)][j];
                    let across = p[i][(j + 1).min(c - 1)] - p[i][j.saturating_sub(1)];
                    let n = across.cross(&down);
                    let n = n / n.norm().max(1e-12);
                    if n.dot(&out(p[i][j])) < 0.0 { -n } else { n }
                })
                .collect()
        })
        .collect()
}

/// A row's section round a sheet with a thickness: the outer face (`c + n h k`) across, a rounded
/// edge at the last column (`s`: across the sheet), the inner face back, a rounded edge at the
/// first; k = 0 closes it to the middle. Each point with its direction out and its shade.
fn section(c: &[V3], n: &[V3], s: &[V3], h: &[f64], k: f64) -> Vec<(V3, V3, f64)> {
    let m = c.len();
    let mut out = Vec::with_capacity(2 * m + 4);
    out.extend((0..m).map(|j| (c[j] + n[j] * h[j] * k, n[j], 0.0)));
    for th in [PI / 3.0, 2.0 * PI / 3.0] {
        let d = n[m - 1] * th.cos() + s[m - 1] * (1.1 * th.sin());
        out.push((c[m - 1] + d * h[m - 1] * k, d, 0.3));
    }
    out.extend((0..m).rev().map(|j| (c[j] - n[j] * h[j] * k, -n[j], 0.55)));
    for th in [2.0 * PI / 3.0, PI / 3.0] {
        let d = n[0] * th.cos() - s[0] * (1.1 * th.sin());
        out.push((c[0] + d * h[0] * k, d, 0.3));
    }
    out
}

/// A sheet of hair with a thickness, from its middle surface (rows from the top to the hem, columns
/// across), the normals out from the head and the half thickness: every row a closed section with
/// rounded edges, and under the last a rounded hem, curled in by `curl` (a share of the thickness).
/// Clump bands per column.
fn slab(mid: &[Vec<V3>], nrm: &[Vec<V3>], half: &[Vec<f64>], curl: f64, bands: &[f64]) -> Comp {
    let (rows, cols) = (mid.len(), mid[0].len());
    let unit = |v: V3| v / v.norm().max(1e-12);
    let across = |q: &[V3]| -> Vec<V3> { (0..cols).map(|j| unit(q[(j + 1).min(cols - 1)] - q[j.saturating_sub(1)])).collect() };
    let mut rings = Vec::new();
    let (mut outs, mut y) = (Vec::new(), Vec::new());
    let mut add = |sec: Vec<(V3, V3, f64)>, shade: Option<f64>| {
        rings.push(sec.iter().map(|e| e.0).collect::<Vec<_>>());
        outs.extend(sec.iter().map(|e| e.1));
        y.extend(sec.iter().map(|e| shade.unwrap_or(e.2)));
    };
    for i in 0..rows {
        add(section(&mid[i], &nrm[i], &across(&mid[i]), &half[i], 1.0), None);
    }
    let l = rows - 1;
    let s = across(&mid[l]);
    for (kd, kn, shade) in [(0.55, 0.75, None), (0.9, 0.0, Some(0.2))] {
        let c: Vec<V3> = (0..cols).map(|j| mid[l][j] + (unit(mid[l][j] - mid[l - 1][j]) - nrm[l][j] * curl) * (half[l][j] * kd)).collect();
        add(section(&c, &nrm[l], &s, &half[l], kn), shade);
    }
    let (pos, idx, _) = rings_mesh(&rings, None, None);
    // The column each point of a section belongs to.
    let k = 2 * cols + 4;
    let col = |v: usize| match v % k {
        q if q < cols => q,
        q if q < cols + 2 => cols - 1,
        q if q < 2 * cols + 2 => 2 * cols + 1 - q,
        _ => 0,
    };
    let z = (0..pos.len()).map(|v| bands[col(v)]).collect();
    comp(pos, idx, Some(outs), Some(z), Some(y), None)
}

/// Clump bands across a sheet's columns (whole locks at its edges): light in the locks' middles,
/// varying from lock to lock.
fn lock_bands(cols: usize, seed: f64) -> Vec<f64> {
    (0..cols).map(|j| clip(0.15 + 0.7 * lock_at(j + 2) * (0.75 + 0.25 * jitter((j + 2) / 4, seed)), 0.0, 1.0)).collect()
}

/// How far out hair hangs at azimuth a, by height: the scalp's section there (a share of the skull
/// ellipsoid's radii), and below its widest straight down from it.
fn hang(sc: &Scalp, a: f64) -> impl Fn(f64) -> f64 {
    let h = sc.h;
    // Depths below the centre (rising) and scales.
    let (mut ds, mut ss) = (Vec::new(), Vec::new());
    let (mut deepest, mut widest) = (f64::NEG_INFINITY, 0.0f64);
    for phi in linspace(radians(10.0), radians(170.0), 33) {
        let p = sc.surface(a, phi).0 - h.centre;
        deepest = deepest.max(-p.y);
        widest = widest.max(((p.x / h.radii[0]).powi(2) + (p.z / h.radii[2]).powi(2)).sqrt());
        ds.push(deepest);
        ss.push(widest);
    }
    let cy = h.centre[1];
    move |y: f64| interp(cy - y, &ds, &ss)
}

/// A curtain's shape (see `curtain`).
struct Curtain<'a> {
    a0: f64,
    a1: f64,
    top: &'a dyn Fn(f64) -> f64,
    bottom: &'a dyn Fn(f64) -> f64,
    /// The inner face's offset out from the scalp it hangs from (m), flaring by `flare` (a share) to the hem.
    off: f64,
    locks: usize,
    rows: usize,
    flare: f64,
    /// Thick in a lock's middle (m; a third of that between locks), the hem curled in by `curl`.
    thick: f64,
    curl: f64,
    /// A lock's tip hangs down to this much lower than the partings beside it (m), the more pointed the
    /// higher `sharp` (1: round).
    tip: f64,
    sharp: f64,
    back: Option<&'a dyn Fn(f64) -> f64>,
    wave: f64,
    seed: i64,
}

/// Hair hanging round the head in locks between azimuths a0..a1, from height top(a) down to
/// bottom(a) (m) and the locks' tips below: a sheet with a thickness, rounded edges and a rounded
/// hem, coming out from under the cap, over the ears and falling straight from the head's widest. `back(y)`
/// (optional): the furthest forward its inner face may come at height y (so it clears the
/// shoulders and back).
fn curtain(sc: &Scalp, c: Curtain) -> Comp {
    let h = sc.h;
    let cols = 4 * c.locks + 1;
    let a = linspace(c.a0, c.a1, cols);
    // Whole locks at the edges, the partings between them.
    let lk: Vec<f64> = (0..cols).map(|j| lock_at(j + 2)).collect();
    let seed = c.seed as f64;
    let tt: Vec<f64> = a.iter().map(|&a| (c.top)(a)).collect();
    let bt: Vec<f64> = (0..cols).map(|j| (c.bottom)(a[j]) - c.tip * (0.35 + 0.65 * jitter((j + 2) / 4, seed)) * lk[j].powf(c.sharp)).collect();
    let hangs: Vec<_> = a.iter().map(|&a| hang(sc, a)).collect();
    let us = linspace(0.0, 1.0, c.rows);
    let inner: Vec<Vec<V3>> = us
        .iter()
        .map(|&u| {
            (0..cols)
                .map(|j| {
                    let y = tt[j] + (bt[j] - tt[j]) * u;
                    let v = hangs[j](y);
                    let f = 1.0 + c.flare * u.powf(1.5);
                    let w = c.wave * (a[j] * 7.0 + seed).sin() * u;
                    let rx = (h.radii[0] * v + c.off + w) * f;
                    let rz = (h.radii[2] * v + c.off + w) * f;
                    let mut p = v3(h.centre[0] + a[j].sin() * rx, y, h.centre[2] + a[j].cos() * rz);
                    if let Some(bk) = c.back {
                        p[2] = p[2].min(bk(y));
                    }
                    // Out over the ears (the scalp it hangs from leaves them out).
                    for [ec, eh] in h.ears {
                        let w = smoothstep(eh.y + 0.015, eh.y, (p.y - ec.y).abs()) * smoothstep(eh.z + 0.015, eh.z, (p.z - ec.z).abs());
                        let clear = ec.x.abs() + eh.x + 0.004;
                        if p.x * ec.x > 0.0 && p.x.abs() < clear {
                            p[0] += ec.x.signum() * (clear - p.x.abs()) * w;
                        }
                    }
                    p
                })
                .collect()
        })
        .collect();
    let nrm = grid_normals(&inner, &|q| v3(q.x - h.centre[0], 0.0, q.z - h.centre[2]));
    // The locks at the edges fuller (they frame the face).
    let full = |j: usize| 1.0 + 0.5 * smoothstep(2.0, 0.0, j.min(cols - 1 - j) as f64);
    let half: Vec<Vec<f64>> = us.iter().map(|&u| (0..cols).map(|j| 0.5 * c.thick * full(j) * (0.35 + 0.65 * lk[j]) * (0.15 + 0.85 * smoothstep(0.0, 0.3, u))).collect()).collect();
    let mid: Vec<Vec<V3>> = (0..c.rows).map(|i| (0..cols).map(|j| inner[i][j] + nrm[i][j] * half[i][j]).collect()).collect();
    slab(&mid, &nrm, &half, c.curl, &lock_bands(cols, seed))
}

/// A fringe's shape (see `fringe`).
struct Fringe<'a> {
    a0: f64,
    a1: f64,
    top: &'a dyn Fn(f64) -> f64,
    bottom: &'a dyn Fn(f64) -> f64,
    /// Its inner face `off - thin` off the skin (m; more by `flare` towards the hem), at least clear of it.
    off: f64,
    thin: f64,
    locks: usize,
    rows: usize,
    /// Thick in a lock's middle (m), and how much lower a lock's tip hangs (m; pointed as `sharp`).
    thick: f64,
    tip: f64,
    sharp: f64,
    flare: f64,
    seed: f64,
}

/// Hair falling over the forehead in locks, on the face as drawn (`face`: a Scalp that isn't held
/// to the cranium): between azimuths a0..a1, from polar angle top(a) (under the cap) down to height
/// bottom(a) (m) and the locks' tips below.
fn fringe(face: &Scalp, c: Fringe) -> Comp {
    let cols = 4 * c.locks + 1;
    let a = linspace(c.a0, c.a1, cols);
    let lk: Vec<f64> = (0..cols).map(|j| lock_at(j + 2)).collect();
    let t0: Vec<f64> = a.iter().map(|&a| (c.top)(a)).collect();
    let t1: Vec<f64> = (0..cols).map(|j| at_height(face, a[j], (c.bottom)(a[j]) - c.tip * (0.5 + 0.5 * jitter((j + 2) / 4, c.seed)) * lk[j].powf(c.sharp), t0[j] + 0.02)).collect();
    let us = linspace(0.0, 1.0, c.rows);
    let (mut mid, mut nrm, mut half) = (Vec::new(), Vec::new(), Vec::new());
    for &u in &us {
        let (mut mr, mut nr, mut hr) = (Vec::new(), Vec::new(), Vec::new());
        for j in 0..cols {
            let (p, n) = face.surface(a[j], t0[j] + (t1[j] - t0[j]) * u);
            let inner = (c.off * (1.0 + c.flare * u.powf(1.5)) - c.thin).max(CLEAR + 0.0005);
            let hf = 0.5 * c.thick * (0.6 + 0.4 * lk[j]) * (0.15 + 0.85 * smoothstep(0.0, 0.3, u));
            mr.push(p + n * (inner + hf));
            nr.push(n);
            hr.push(hf);
        }
        mid.push(mr);
        nrm.push(nr);
        half.push(hr);
    }
    slab(&mid, &nrm, &half, 0.3, &lock_bands(cols, c.seed))
}

/// The polar angle (below `from`) at which the surface at azimuth a comes down to height y.
fn at_height(sc: &Scalp, a: f64, y: f64, from: f64) -> f64 {
    let (mut lo, mut hi) = (from, radians(150.0));
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if sc.surface(a, mid).0[1] < y {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    (lo + hi) / 2.0
}

/// A lock falling over the face (`face`: a Scalp that isn't held to the cranium) from deep in the cap
/// at (a0, phi0) through points (azimuth, height, offset off the skin; m): a flattened tube `wide`
/// across, `thick` deep, tapering to a point.
fn face_lock(face: &Scalp, a0: f64, phi0: f64, pts: &[(f64, f64, f64)], wide: f64, thick: f64, seed: f64) -> Comp {
    let mut ctl = vec![face.point(a0, phi0, 0.4 * pts[0].2)];
    ctl.extend(pts.iter().map(|&(a, y, off)| face.point(a, at_height(face, a, y, phi0), off)));
    let path = spline(&ctl, 10);
    let t = linspace(0.0, 1.0, 10);
    let rx: Vec<f64> = t.iter().map(|&t| thick * interp(t, &[0.0, 0.3, 1.0], &[0.8, 1.0, 0.35])).collect();
    let rz: Vec<f64> = t.iter().map(|&t| wide * interp(t, &[0.0, 0.3, 0.75, 1.0], &[0.7, 1.0, 0.7, 0.12])).collect();
    let m = loft(&path, &rx, &rz, 6, face.surface(pts[0].0, at_height(face, pts[0].0, pts[0].1, phi0)).1, caps(1));
    let z = m.pos.iter().enumerate().map(|(i, _)| clip(0.55 + 0.35 * ((i % 6) as f64 * PI / 3.0 + seed).cos(), 0.0, 1.0)).collect();
    comp(m.pos, m.idx, None, Some(z), Some(vec![0.1; m.t.len()]), None)
}

/// A lock lying on the scalp from (a, phi), `len` (rad of arc) long, turned `turn` from straight
/// down the head and lifting off it by `lift` (m) at its tip: a flattened tube `wide` across and
/// `thick` deep, tapering to a point.
#[allow(clippy::too_many_arguments)]
fn tuft(sc: &Scalp, a: f64, phi: f64, len: f64, turn: f64, lift: f64, off: f64, wide: f64, thick: f64, seed: f64) -> Comp {
    let s = linspace(0.0, 1.0, 6);
    let st = phi.sin().max(0.3);
    let pts: Vec<V3> = s.iter().map(|&s| sc.point(a + turn.sin() * len * s / st, (phi + turn.cos() * len * s).max(0.05), off + lift * s * s)).collect();
    let rx: Vec<f64> = s.iter().map(|&s| thick * interp(s, &[0.0, 0.3, 1.0], &[0.8, 1.0, 0.3])).collect();
    let rz: Vec<f64> = s.iter().map(|&s| wide * interp(s, &[0.0, 0.25, 0.7, 1.0], &[0.7, 1.0, 0.6, 0.08])).collect();
    let m = loft(&pts, &rx, &rz, 6, sc.surface(a, phi).1, caps(1));
    let k = 0.3 + 0.4 * jitter(0, seed);
    let z = m.t.iter().map(|&t| clip(k + 0.3 * (1.0 - t), 0.0, 1.0)).collect();
    let y = m.t.iter().map(|&t| 0.2 * (1.0 - t).max(0.0)).collect();
    comp(m.pos, m.idx, None, Some(z), Some(y), None)
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

/// A loft's rings (`segs` points each, laid one after the other) in `grooves` locks round it: each
/// point pushed out from its ring's middle by up to `depth` (a share) on a lock and in between
/// them, twisting by `twist` (rad) along its length (t). Clump bands follow the locks.
fn grooved(m: &mut super::prims::Prim, segs: usize, grooves: f64, depth: f64, twist: f64) -> Vec<f64> {
    let rings = m.t.len() / segs;
    let mut z = vec![0.5; m.pos.len()];
    for r in 0..rings {
        let c = m.pos[r * segs..(r + 1) * segs].iter().fold(V3::zeros(), |s, p| s + p) / segs as f64;
        for q in 0..segs {
            let i = r * segs + q;
            let g = (TAU * q as f64 / segs as f64 * grooves + twist * m.t[i]).cos();
            m.pos[i] = c + (m.pos[i] - c) * (1.0 + depth * g);
            z[i] = 0.5 + 0.4 * g;
        }
    }
    z
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
    // Short locks past the hairline at the temples and the nape (none over the forehead or the ears).
    let nape_tips = move |a: f64| deg(4.5) * smoothstep(deg(105.0), deg(135.0), round(a)).max(smoothstep(deg(70.0), deg(50.0), round(a)) * smoothstep(deg(30.0), deg(45.0), round(a)));

    // Short: volume on top, combed away from a side parting (the resident's right) in locks, a quiff
    // over the forehead.
    let base = thick(0.05 * hh, 0.03 * hh, 0.028 * hh, Some(0.014 * hh));
    let quiffed = |a: f64, u: f64| base(a, u) + 0.022 * hh * (-(a / 0.6).powi(2)).exp() * smoothstep(0.35, 0.75, u) * smoothstep(1.0, 0.85, u);
    styles.push(("short", vec![cap(&sc, Cap { thick: &quiffed, rows: 11, cols: 40, limit: None, edge: CLEAR, ridge: 0.35, tips: Some(&nape_tips), part: Some((deg(20.0), 0.3)), bands: None })]));

    // Crop: a thin cap, a squared hairline and a tiny fringe in short points.
    let front = sc.hairline(0.0) + deg(3.0);
    let square = |a: f64| if a.abs() < deg(55.0) { front + deg(4.0) * (a.abs() / deg(55.0)).powf(6.0) } else { sc.hairline(a) };
    let crop_tips = |a: f64| deg(3.0) * smoothstep(deg(60.0), deg(20.0), round(a)) + 0.5 * nape_tips(a);
    styles.push(("crop", vec![cap(&sc, Cap { thick: &thick(0.022 * hh, 0.016 * hh, 0.016 * hh, Some(0.011 * hh)), rows: 8, cols: 40, limit: Some(&square), edge: CLEAR, ridge: 0.25, tips: Some(&crop_tips), part: None, bands: None })]));

    // Layered: a cap and three flared tiers in pointed locks, the lowest to the jaw.
    let mut lay = vec![cap(&sc, Cap { thick: &thick(0.05 * hh, 0.04 * hh, 0.04 * hh, None), rows: 9, cols: 32, limit: None, edge: CLEAR, ridge: 0.3, tips: None, part: None, bands: None })];
    for (k, (yt, yb, off, fl)) in [(0.72, 0.5, 0.028, 0.06), (0.58, 0.3, 0.02, 0.09), (0.44, 0.08, 0.012, 0.12)].into_iter().enumerate() {
        let span = deg(62.0 + 6.0 * k as f64);
        let (top, bot) = (chin + yt * hh, chin + yb * hh);
        lay.push(curtain(&sc, Curtain { a0: span, a1: TAU - span, top: &move |_| top, bottom: &move |_| bot, off: off * hh, locks: 6, rows: 3, flare: fl, thick: 0.017 * hh, curl: 0.5, tip: 0.1 * hh, sharp: 3.5, back: None, wave: 0.0, seed: k as i64 }));
    }
    styles.push(("layered", lay));

    // Tousled: a cap and seeded locks pointing every which way, their tips lifting, a few over the
    // forehead.
    let mut rng = Pcg64::new(11);
    let tb = thick(0.034 * hh, 0.028 * hh, 0.026 * hh, Some(0.016 * hh));
    let lumpy = |a: f64, u: f64| tb(a, u) * (1.0 + 0.35 * noise3(v3(a.cos(), 2.0 * u, a.sin()), 5, 2.5));
    let mut tou = vec![cap(&sc, Cap { thick: &lumpy, rows: 8, cols: 24, limit: None, edge: CLEAR, ridge: 0.3, tips: Some(&nape_tips), part: None, bands: None })];
    for k in 0..22 {
        // Spread over the head on a spiral (each about as much of it), nudged.
        let a = py_mod(k as f64 * PI * (3.0 - 5f64.sqrt()) + rng.uniform(-0.2, 0.2) + PI, TAU) - PI;
        let phi = sc.hairline(a) * (0.82 * ((k as f64 + 0.5) / 22.0).sqrt() + rng.uniform(-0.05, 0.05));
        let len = rng.uniform(0.4, 0.65);
        let turn = rng.uniform(-0.65, 0.65);
        let lift = rng.uniform(0.015, 0.04) * hh;
        let wide = rng.uniform(0.055, 0.075) * hh;
        tou.push(tuft(&sc, a, phi, len, turn, lift, 0.03 * hh, wide, 0.028 * hh, k as f64));
    }
    styles.push(("tousled", tou));

    // Side-swept: a cap with a high side parting and three locks swept from it across the forehead.
    let mut sw = vec![cap(&sc, Cap { thick: &thick(0.045 * hh, 0.03 * hh, 0.028 * hh, Some(0.014 * hh)), rows: 9, cols: 36, limit: None, edge: CLEAR, ridge: 0.3, tips: Some(&nape_tips), part: Some((deg(45.0), 0.35)), bands: None })];
    let fa = [0.75, 0.35, 0.0, -0.4, -0.75, -1.05];
    let fk = [0.55, 0.78, 0.92, 0.98, 1.02, 1.08];
    let fo = [0.05, 0.05, 0.048, 0.045, 0.038, 0.028];
    for (k, (da, dk, len, size)) in [(0.0, 0.0, 1.0, 1.0), (0.07, -0.06, 0.8, 0.85), (-0.06, 0.05, 0.62, 0.8)].into_iter().enumerate() {
        let n = 3 + (len * 3.0) as usize;
        let pts: Vec<V3> = (0..n).map(|i| sc.point(fa[i] + da, sc.hairline(fa[i] + da) * (fk[i] + dk), fo[i] * hh * (0.9 + 0.1 * size))).collect();
        let pts = spline(&pts, 9);
        let t9 = linspace(0.0, 1.0, 9);
        let ry: Vec<f64> = t9.iter().map(|&t| interp(t, &[0.0, 0.4, 1.0], &[0.05, 0.068, 0.016]) * hh * size).collect();
        let rz: Vec<f64> = t9.iter().map(|&t| interp(t, &[0.0, 0.5, 1.0], &[0.022, 0.028, 0.01]) * hh * size).collect();
        sw.push(lobe(&pts, &ry, &rz, 8, v3(0.0, 1.0, 0.3), 9.0, k as f64, 0.0, None));
    }
    styles.push(("sideSwept", sw));

    // Afro: a big rounded cap of round puffs.
    let puffs: Vec<V3> = (0..46)
        .map(|i| {
            // A Fibonacci spiral over the head, in the skull ellipsoid's normalised frame.
            let y = 1.0 - 2.0 * (i as f64 + 0.5) / 46.0;
            let r = (1.0 - y * y).sqrt();
            let t = i as f64 * PI * (3.0 - 5f64.sqrt());
            v3(r * t.sin(), y, r * t.cos())
        })
        .collect();
    // How far up a puff a direction is (0 between puffs .. 1 on a puff's top).
    let puff = |e: V3| puffs.iter().map(|q| (1.0 - (e.dot(q).clamp(-1.0, 1.0).acos() / 0.32).powi(2)).max(0.0).sqrt()).fold(0.0, f64::max);
    let dir = |a: f64, u: f64| {
        let e = Scalp::dirs(a, u * sc.hairline(a));
        e / e.norm()
    };
    let rounded = |a: f64, u: f64| {
        // Out to a rounded shell 1.35 times the skull, a little up and back, in puffs.
        let e = Scalp::dirs(a, u * sc.hairline(a)).component_mul(&h.radii);
        let e = e / e.norm();
        let q = e.component_div(&h.radii);
        let r_s = 1.0 / (q.x * q.x + q.y * q.y + q.z * q.z).sqrt();
        let rr = h.radii.component_mul(&v3(1.36, 1.3, 1.36));
        let w = e.component_div(&rr).map(|v| v.abs().powf(2.1));
        let sh = v3(0.0, 0.1, -0.08).component_mul(&h.radii);
        let r_t = 1.0 / (w.x + w.y + w.z).sqrt().powf(1.0 / 1.2) + (e.x * sh.x + e.y * sh.y + e.z * sh.z);
        ((r_t - r_s - 0.02 * hh).max(0.02 * hh) + 0.06 * hh * puff(dir(a, u))) * (0.3 + 0.7 * smoothstep(1.0, 0.55, u))
    };
    let puff_bands = |a: f64, u: f64| 0.15 + 0.75 * puff(dir(a, u));
    styles.push(("afro", vec![cap(&sc, Cap { thick: &rounded, rows: 16, cols: 44, limit: None, edge: CLEAR + 0.004 * hh, ridge: 0.0, tips: None, part: None, bands: Some(&puff_bands) })]));

    // Bob: a shell to the jaw in blunt locks with a curled-in hem, and a straight fringe.
    let mut bob = vec![cap(&sc, Cap { thick: &thick(0.05 * hh, 0.04 * hh, 0.04 * hh, None), rows: 9, cols: 32, limit: None, edge: CLEAR, ridge: 0.15, tips: None, part: None, bands: None })];
    let (bt, bb) = (chin + 0.72 * hh, chin + 0.06 * hh);
    bob.push(curtain(&sc, Curtain { a0: deg(58.0), a1: TAU - deg(58.0), top: &move |_| bt, bottom: &move |_| bb, off: 0.012 * hh, locks: 9, rows: 5, flare: 0.12, thick: 0.03 * hh, curl: 1.0, tip: 0.03 * hh, sharp: 1.5, back: None, wave: 0.0, seed: 0 }));
    let under = |a: f64| sc.hairline(a) * 0.8;
    let bob_bot = |a: f64| brow_y + hh * (0.05 + 0.03 * (a / deg(64.0)).powi(2));
    bob.push(fringe(&skin, Fringe { a0: -deg(64.0), a1: deg(64.0), top: &under, bottom: &bob_bot, off: 0.03 * hh, thin: 0.012 * hh, locks: 6, rows: 3, thick: 0.022 * hh, tip: 0.03 * hh, sharp: 3.0, flare: 0.15, seed: 2.0 }));
    styles.push(("bob", bob));

    // Long: a cap parted in the middle, the sides to the shoulders, the back to mid-back, all in locks
    // with pointed tips; a fringe either side of the parting framing the face.
    let mut lng = vec![cap(&sc, Cap { thick: &thick(0.045 * hh, 0.035 * hh, 0.035 * hh, None), rows: 9, cols: 32, limit: None, edge: CLEAR, ridge: 0.15, tips: None, part: Some((0.0, 0.3)), bands: None })];
    let shoulder = p[b("clavicle_l")].y;
    let side_bot = |a: f64| interp(round(a), &[deg(70.0), deg(110.0), deg(150.0)], &[shoulder + 0.25 * hh, shoulder - 0.1 * hh, sp[1] - 0.5 * hh]);
    let lt = chin + 0.72 * hh;
    let long_back = |y: f64| back(y) + 0.014 * hh;
    lng.push(curtain(&sc, Curtain { a0: deg(66.0), a1: TAU - deg(66.0), top: &move |_| lt, bottom: &side_bot, off: 0.012 * hh, locks: 8, rows: 7, flare: 0.18, thick: 0.032 * hh, curl: 0.6, tip: 0.07 * hh, sharp: 2.5, back: Some(&long_back), wave: 0.004 * hh, seed: 0 }));
    for s in [-1.0, 1.0] {
        // Two locks either side of the parting, the outer one down past the cheek to the jaw.
        let outer = [(0.45 * s, brow_y + 0.14 * hh, 0.036 * hh), (0.85 * s, brow_y + 0.0 * hh, 0.038 * hh), (1.04 * s, brow_y - 0.2 * hh, 0.036 * hh), (1.13 * s, brow_y - 0.36 * hh, 0.032 * hh)];
        lng.push(face_lock(&skin, 0.1 * s, sc.hairline(0.1 * s) * 0.7, &outer, 0.065 * hh, 0.028 * hh, 1.0 + s));
        let inner = [(0.35 * s, brow_y + 0.15 * hh, 0.032 * hh), (0.62 * s, brow_y + 0.05 * hh, 0.034 * hh), (0.86 * s, brow_y - 0.08 * hh, 0.032 * hh)];
        lng.push(face_lock(&skin, 0.06 * s, sc.hairline(0.06 * s) * 0.7, &inner, 0.055 * hh, 0.025 * hh, 2.0 + s));
    }
    styles.push(("long", lng));

    // Ponytail: a slicked cap, a band at the back of the head and a tapering tail in twisting locks.
    let b15 = |a: f64, _u: f64| 0.5 + 0.5 * (a * 15.0).sin();
    let mut pt = vec![cap(&sc, Cap { thick: &thick(0.024 * hh, 0.018 * hh, 0.02 * hh, Some(0.012 * hh)), rows: 9, cols: 32, limit: None, edge: CLEAR, ridge: 0.2, tips: None, part: None, bands: None })];
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
    let tp = spline(&tail, 12);
    let t12 = linspace(0.0, 1.0, 12);
    let ry: Vec<f64> = t12.iter().map(|&t| interp(t, &[0.0, 0.25, 1.0], &[0.08, 0.105, 0.025]) * hh).collect();
    let rz: Vec<f64> = t12.iter().map(|&t| interp(t, &[0.0, 0.25, 1.0], &[0.07, 0.085, 0.02]) * hh).collect();
    let mut m = loft(&tp, &ry, &rz, 12, v3(1.0, 0.0, 0.0), caps(2));
    let z = grooved(&mut m, 12, 4.0, 0.16, 2.5);
    let f = m.t.iter().map(|&t| smoothstep(0.1, 0.9, t)).collect();
    pt.push(comp(m.pos, m.idx, None, Some(z), None, Some(f)));
    styles.push(("ponytail", pt));

    // Braid: a cap and three strands plaited down the back, tied off with a tuft below the tie.
    let mut br = vec![cap(&sc, Cap { thick: &thick(0.03 * hh, 0.024 * hh, 0.026 * hh, Some(0.018 * hh)), rows: 9, cols: 32, limit: None, edge: CLEAR, ridge: 0.2, tips: None, part: None, bands: Some(&b15) })];
    let start = sc.point(PI, deg(120.0), 0.03 * hh);
    let len = 0.82 * hh;
    // The braid's middle at s (0 at the nape .. 1 at the tie).
    let axis = |s: f64| {
        let y = start[1] - s * len;
        v3(0.0, y, (back(y) - 0.055 * hh).min(start[2] - 0.09 * hh * s))
    };
    for k in 0..3 {
        let ph = TAU * k as f64 / 3.0;
        let s = linspace(0.0, 1.0, 22);
        let pts: Vec<V3> = s
            .iter()
            .map(|&s| {
                let th = TAU * 3.5 * s + ph;
                let w = 0.05 * hh * (0.75 - 0.25 * s) * smoothstep(-0.1, 0.12, s);
                axis(s) + v3(w * th.sin(), 0.0, -0.022 * hh * (2.0 * th).sin() * smoothstep(-0.1, 0.12, s))
            })
            .collect();
        let r: Vec<f64> = s.iter().map(|&s| 0.036 * hh * (1.0 - 0.35 * s)).collect();
        let m = loft(&pts, &r, &r, 6, v3(0.0, 0.0, -1.0), caps(1));
        let z = m.t.iter().map(|&t| 0.5 + 0.4 * (TAU * 7.0 * t + ph).sin()).collect();
        let f = m.t.iter().map(|&t| smoothstep(0.0, 0.67, t)).collect();
        br.push(comp(m.pos, m.idx, None, Some(z), Some(vec![0.1; m.t.len()]), Some(f)));
    }
    let end = axis(1.0);
    let mut knot = tor(torus(end + v3(0.0, -0.01 * hh, 0.0), 0.03 * hh, 0.012 * hh, 10, 5, v3(0.0, 1.0, 0.0)));
    knot.f = vec![1.0; knot.pos.len()];
    br.push(knot);
    let tp: Vec<V3> = linspace(0.0, 1.0, 5).iter().map(|&s| end + v3(0.0, -0.13 * hh * s - 0.015 * hh, -0.01 * hh * s)).collect();
    let r: Vec<f64> = linspace(0.0, 1.0, 5).iter().map(|&s| interp(s, &[0.0, 0.4, 1.0], &[0.025, 0.04, 0.012]) * hh).collect();
    let mut m = loft(&tp, &r, &r, 8, v3(0.0, 0.0, -1.0), caps(1));
    let z = grooved(&mut m, 8, 3.0, 0.2, 1.0);
    br.push(comp(m.pos, m.idx, None, Some(z), None, Some(vec![1.0; m.t.len()])));
    styles.push(("braid", br));

    // Buns: a cap and two buns high on each side, twisted, each with a band round its foot.
    let mut bun = vec![cap(&sc, Cap { thick: &thick(0.026 * hh, 0.02 * hh, 0.022 * hh, Some(0.014 * hh)), rows: 9, cols: 32, limit: None, edge: CLEAR, ridge: 0.2, tips: None, part: Some((0.0, 0.3)), bands: Some(&b15) })];
    for s in [-1.0, 1.0] {
        let c = sc.point(s * deg(118.0), deg(38.0), 0.13 * hh);
        let ax = (c - h.centre) / (c - h.centre).norm();
        let e = ellipsoid(c, v3(0.15, 0.14, 0.15) * hh, 9, 16);
        // A twist round the bun's axis (out from the head): coils that wind up to its top.
        let (u, v) = super::prims::frame(ax, v3(0.0, 1.0, 0.0));
        let mut z = Vec::new();
        let pos: Vec<V3> = e
            .pos
            .iter()
            .map(|q| {
                let d = q - c;
                let up = d.dot(&ax) / (0.15 * hh);
                let g = (3.0 * d.dot(&v).atan2(d.dot(&u)) + 6.0 * up).cos();
                z.push(0.5 + 0.4 * g);
                c + d * (1.0 + 0.09 * g * (1.0 - up.abs().powi(4)))
            })
            .collect();
        let out = pos.iter().map(|q| q - c).collect();
        bun.push(comp(pos, e.idx, Some(out), Some(z), None, None));
        bun.push(tor(torus(c - ax * (0.115 * hh), 0.085 * hh, 0.02 * hh, 12, 5, ax)));
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
/// onto its edge), in locks: thicker down their middles, their tips hanging below the jaw at the
/// chin, thinning to the skin at its lower edge.
fn beard(sc: &Scalp) -> Comp {
    let h = sc.h;
    let hh = h.hh;
    let (cols, rows) = (25, 12);
    let a = linspace(-radians(94.0), radians(94.0), cols);
    let lk: Vec<f64> = (0..cols).map(|j| lock_at(j + 2)).collect();
    // Mouth direction (polar angle) and the hole round it.
    let qm = (h.mouth - h.centre).component_div(&h.radii);
    let phm = (qm[1] / qm.norm()).acos();
    let top: Vec<f64> = a.iter().map(|&a| interp(a.abs(), &[0.0, 22.0, 45.0, 70.0, 85.0, 94.0].map(radians), &[phm - 0.17, phm - 0.15, phm - 0.02, phm - 0.1, radians(96.0), radians(94.0)])).collect();
    // The lower edge: just under the jaw, rising to the jaw's corners at the sides; the locks' tips
    // lower round the chin.
    let bot: Vec<f64> = (0..cols)
        .map(|j| {
            let tip = 0.035 * hh * lk[j].powf(2.5) * (0.6 + 0.4 * jitter((j + 2) / 4, 5.0)) * smoothstep(radians(70.0), radians(30.0), a[j].abs());
            let want = h.chin_y + hh * interp(a[j].abs(), &[0.0, 45.0, 80.0, 94.0].map(radians), &[-0.01, 0.02, 0.15, 0.22]) - tip;
            at_height(sc, a[j], want, top[j] + 0.1)
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
            let edge = smoothstep(1.0, 1.6, r).min(smoothstep(0.0, 0.2, u)).min(smoothstep(radians(94.0), radians(78.0), aa.abs())).min(smoothstep(1.0, 0.8, u));
            let off = CLEAR + (0.02 * hh) * edge * (0.65 + 0.35 * lk[j]);
            row.push(sc.point(a2, phi2, off));
            z.push(0.5 + (0.35 * lk[j] - 0.1) * (0.7 + 0.3 * (phi * 8.0 + aa * 3.0).sin()));
            y.push(0.2 * (1.0 - edge) + 0.2 * (1.0 - lk[j]));
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
