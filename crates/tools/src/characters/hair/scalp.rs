//! The scalp: points on the skull by direction, and its oval and the line along it.

use super::*;

/// Points on the skull by direction (azimuth a round the vertical axis, 0 = front, + = the
/// resident's right; polar angle phi from the top, in the skull ellipsoid's normalised frame);
/// `cranium`: no further out than the skull ellipsoid (else the face and jaw as drawn).
pub struct Scalp<'a> {
    pub(crate) h: &'a HeadFrame,
    pub(crate) cranium: bool,
    pub(crate) hl: Periodic,
    /// The ears as caps meet them (`round`).
    pub(crate) ovals: Vec<Oval>,
}

impl<'a> Scalp<'a> {
    pub fn new(h: &'a HeadFrame, cranium: bool) -> Self {
        let q: Vec<V3> = h
            .hairline
            .iter()
            .map(|p| (p - h.centre).component_div(&h.radii))
            .collect();
        let a: Vec<f64> = q.iter().map(|q| q.x.atan2(q.z)).collect();
        let phi: Vec<f64> = q
            .iter()
            .map(|q| clip(q.y / q.norm(), -1.0, 1.0).acos())
            .collect();
        let o = super::np::argsort(&a);
        // The hair's own edge: the head's hairline smoothed round (no point at the nape) and
        // brought a little lower over the forehead and temples (fuller hair, a softer face).
        let fa = linspace_open(-PI, PI, 180);
        let per = Periodic::new(
            &o.iter().map(|&i| a[i]).collect::<Vec<_>>(),
            &o.iter().map(|&i| phi[i]).collect::<Vec<_>>(),
            TAU,
        );
        let fp: Vec<f64> = fa.iter().map(|&x| per.at(x)).collect();
        let mut k: Vec<f64> = fa
            .iter()
            .map(|&x| (-0.5 * (x.abs().min(TAU - x.abs()) / radians(16.0)).powi(2)).exp())
            .collect();
        let ks = pairwise_sum(&k);
        for v in &mut k {
            *v /= ks;
        }
        // Circular convolution with the kernel centred on index 0 (np.fft's ifftshift, then fft
        // products; a direct sum here).
        let n = fa.len();
        let kk: Vec<f64> = (0..n).map(|i| k[(i + n / 2) % n]).collect();
        let mut fp: Vec<f64> = (0..n)
            .map(|i| (0..n).fold(0.0, |s, j| s + fp[j] * kk[(i + n - j) % n]))
            .collect();
        for (v, &x) in fp.iter_mut().zip(&fa) {
            let t = x.abs();
            *v += radians(5.0) * smoothstep(radians(70.0), 0.0, t)
                + radians(8.0) * (-((t - radians(70.0)) / radians(28.0)).powi(2)).exp();
        }
        Scalp {
            h,
            cranium,
            hl: Periodic::new(&fa, &fp, TAU),
            ovals: Vec::new(),
        }
    }

    /// The same, meeting the ears as drawn (an `Oval` round each).
    pub fn round(mut self, ears: &Ears) -> Self {
        self.ovals = ears
            .ears
            .iter()
            .filter_map(|e| Oval::new(&self, &e.own))
            .collect();
        self
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
        self.rounded(a, phi, ROUND)
    }

    /// The same, rounded out to the skull ellipsoid by `round` (0..1) rather than `ROUND`.
    pub(crate) fn rounded(&self, a: f64, phi: f64, round: f64) -> (V3, V3) {
        let h = self.h;
        let d = Scalp::dirs(a, phi).component_mul(&h.radii);
        // Over the ears no further out than the skull ellipsoid (which holds the cranium), so the
        // ears don't push the hair about; caps go round them (`Oval`), curtains out over them.
        let hi0 = if self.cranium {
            // Past the ellipsoid only away from the ears (the brow and forehead may stand out of it).
            let ear = (-((self::round(a) - PI / 2.0) / 0.6).powi(2)).exp()
                * (-((phi - PI / 2.0) / 0.5).powi(2)).exp();
            1.0 + 0.6 * (1.0 - smoothstep(0.15, 0.5, ear))
        } else {
            2.5
        };
        // March out to the first step outside (the field far from the head can't be trusted), then
        // bisect within that step.
        let steps = linspace(0.2, 2.5, 47);
        let k = steps
            .iter()
            .position(|&s| s >= hi0 || h.sdf(h.centre + d * s) >= 0.0)
            .unwrap_or(0);
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
            t += round * (1.0 - t).max(0.0);
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

    /// The oval of the ear nearest azimuth a, if one was given.
    pub(crate) fn oval(&self, a: f64) -> Option<&Oval> {
        self.ovals
            .iter()
            .min_by(|p, q| round(a - p.a).total_cmp(&round(a - q.a)))
    }
}

/// An ear as a cap meets it: an ellipse round it in the scalp's azimuth and polar angle (the ear's
/// faces seen from the head's centre, fitted round and grown by `EAR_CLEAR` and a rim's thickness),
/// in metres near it (`g`: m per rad of each, where the ear comes nearest the centre, so the margin
/// holds); and how far the ear stands out of the scalp.
pub(crate) struct Oval {
    pub(crate) c: [f64; 2],
    pub(crate) r: [f64; 2],
    /// Its first axis's direction (cos, sin) in that plane.
    pub(crate) e: [f64; 2],
    pub(crate) g: [f64; 2],
    /// Its middle's azimuth, and how far the ear reaches out past the scalp (m).
    pub(crate) a: f64,
    pub(crate) reach: f64,
}

impl Oval {
    fn new(sc: &Scalp, own: &[[V3; 3]]) -> Option<Self> {
        let h = sc.h;
        let pts: Vec<V3> = own.iter().flatten().copied().collect();
        if pts.is_empty() {
            return None;
        }
        let q: Vec<V3> = pts
            .iter()
            .map(|p| (p - h.centre).component_div(&h.radii))
            .collect();
        let ang: Vec<[f64; 2]> = q
            .iter()
            .map(|q| [q.x.atan2(q.z), clip(q.y / q.norm(), -1.0, 1.0).acos()])
            .collect();
        let n = ang.len() as f64;
        let (a0, p0) = (
            ang.iter().map(|v| v[0]).sum::<f64>() / n,
            ang.iter().map(|v| v[1]).sum::<f64>() / n,
        );
        let t = q.iter().map(|q| q.norm()).fold(f64::INFINITY, f64::min);
        let e0 = Scalp::dirs(a0, p0).component_mul(&h.radii);
        let d = 1e-4;
        let g = [
            (Scalp::dirs(a0 + d, p0).component_mul(&h.radii) - e0).norm() / d * t,
            (Scalp::dirs(a0, p0 + d).component_mul(&h.radii) - e0).norm() / d * t,
        ];
        let xy: Vec<[f64; 2]> = ang.iter().map(|v| [v[0] * g[0], v[1] * g[1]]).collect();
        // Its long axis (principal), then the smallest ellipse of its box's proportions round it.
        let m = [
            xy.iter().map(|v| v[0]).sum::<f64>() / n,
            xy.iter().map(|v| v[1]).sum::<f64>() / n,
        ];
        let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
        for v in &xy {
            let (dx, dy) = (v[0] - m[0], v[1] - m[1]);
            sxx += dx * dx;
            sxy += dx * dy;
            syy += dy * dy;
        }
        let th = 0.5 * (2.0 * sxy).atan2(sxx - syy);
        let e = [th.cos(), th.sin()];
        let uv: Vec<[f64; 2]> = xy
            .iter()
            .map(|v| {
                [
                    (v[0] - m[0]) * e[0] + (v[1] - m[1]) * e[1],
                    -(v[0] - m[0]) * e[1] + (v[1] - m[1]) * e[0],
                ]
            })
            .collect();
        let lo = [
            uv.iter().map(|v| v[0]).fold(f64::INFINITY, f64::min),
            uv.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min),
        ];
        let hi = [
            uv.iter().map(|v| v[0]).fold(f64::NEG_INFINITY, f64::max),
            uv.iter().map(|v| v[1]).fold(f64::NEG_INFINITY, f64::max),
        ];
        let (mu, mv) = ((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0);
        let half = [(hi[0] - lo[0]) / 2.0, (hi[1] - lo[1]) / 2.0];
        let k = uv
            .iter()
            .map(|v| (((v[0] - mu) / half[0]).powi(2) + ((v[1] - mv) / half[1]).powi(2)).sqrt())
            .fold(0.0, f64::max);
        // A little more than the margin: an ellipse grown by it along its axes is not quite its offset.
        let grow = 1.15 * (EAR_CLEAR + CLEAR);
        let c = [m[0] + mu * e[0] - mv * e[1], m[1] + mu * e[1] + mv * e[0]];
        let reach = pts
            .iter()
            .zip(&ang)
            .map(|(p, v)| {
                let (s, nrm) = sc.surface(v[0], v[1]);
                (p - s).dot(&nrm)
            })
            .fold(0.0, f64::max);
        Some(Oval {
            c,
            r: [k * half[0] + grow, k * half[1] + grow],
            e,
            g,
            a: a0,
            reach,
        })
    }

    /// A point's (azimuth, polar angle) along and across the oval's axes, in its radii.
    fn local(&self, a: f64, phi: f64) -> [f64; 2] {
        let (dx, dy) = (a * self.g[0] - self.c[0], phi * self.g[1] - self.c[1]);
        [
            (dx * self.e[0] + dy * self.e[1]) / self.r[0],
            (-dx * self.e[1] + dy * self.e[0]) / self.r[1],
        ]
    }

    /// How far out of the middle a point is (1 on the oval).
    pub(crate) fn rho(&self, a: f64, phi: f64) -> f64 {
        let [u, v] = self.local(a, phi);
        u.hypot(v)
    }

    /// The oval's top (least polar angle) at azimuth a, if it reaches there.
    fn top(&self, a: f64) -> Option<f64> {
        let dx = a * self.g[0] - self.c[0];
        let [c, s] = self.e;
        let (r0, r1) = (self.r[0] * self.r[0], self.r[1] * self.r[1]);
        let qa = s * s / r0 + c * c / r1;
        let qb = 2.0 * dx * c * s * (1.0 / r0 - 1.0 / r1);
        let qc = dx * dx * (c * c / r0 + s * s / r1) - 1.0;
        let disc = qb * qb - 4.0 * qa * qc;
        (disc >= 0.0).then(|| (self.c[1] + (-qb - disc.sqrt()) / (2.0 * qa)) / self.g[1])
    }

    /// A point pushed out onto the oval if it is inside.
    fn out(&self, a: f64, phi: f64) -> (f64, f64) {
        let rho = self.rho(a, phi);
        if rho >= 1.0 {
            return (a, phi);
        }
        let (dx, dy) = (a * self.g[0] - self.c[0], phi * self.g[1] - self.c[1]);
        let k = 1.0 / rho.max(1e-6);
        (
            (self.c[0] + dx * k) / self.g[0],
            (self.c[1] + dy * k) / self.g[1],
        )
    }

    /// A point inside the oval taken out to it upwards (away from a point below its middle); one
    /// outside as it is.
    pub(crate) fn up(&self, a: f64, phi: f64) -> (f64, f64) {
        if self.rho(a, phi) >= 1.0 {
            return (a, phi);
        }
        let below = 0.5 * self.r[0].min(self.r[1]);
        let (a0, p0) = (self.c[0] / self.g[0], (self.c[1] + below) / self.g[1]);
        let (l0, l1) = (self.local(a0, p0), self.local(a, phi));
        let d = [l1[0] - l0[0], l1[1] - l0[1]];
        let (dd, ld, ll) = (
            d[0] * d[0] + d[1] * d[1],
            l0[0] * d[0] + l0[1] * d[1],
            l0[0] * l0[0] + l0[1] * l0[1],
        );
        let t = (-ld + (ld * ld - dd * (ll - 1.0)).max(0.0).sqrt()) / dd.max(1e-12);
        (a0 + t * (a - a0), p0 + t * (phi - p0))
    }

    /// The hair's edge round the ear from azimuth a0 to a1 (the hairline `hl` there, over the oval
    /// where it would come down into it, rounded where the two meet), and `n` points along it between
    /// its ends, closer together near the ear.
    pub(crate) fn rim(
        &self,
        hl: &dyn Fn(f64) -> f64,
        a0: f64,
        a1: f64,
        n: usize,
    ) -> Vec<(f64, f64)> {
        let g = self.g;
        let line: Vec<[f64; 2]> = linspace(a0, a1, 801)
            .iter()
            .map(|&a| [a * g[0], hl(a).min(self.top(a).unwrap_or(PI)) * g[1]])
            .collect();
        // Evenly along it, smoothed (the ends held by reflecting the line past them) and back out of
        // the oval.
        let m = 1201;
        let line = along(&line, &|_| 1.0, &linspace(0.0, 1.0, m));
        let sig = RIM_ROUND / (length(&line) / (m - 1) as f64);
        let w = (3.0 * sig).ceil() as isize;
        let kernel: Vec<f64> = (-w..=w)
            .map(|k| (-0.5 * (k as f64 / sig).powi(2)).exp())
            .collect();
        let at = |i: isize| -> [f64; 2] {
            let (k, end) = if i < 0 {
                (-i, 0)
            } else if i >= m as isize {
                (2 * (m as isize - 1) - i, m - 1)
            } else {
                return line[i as usize];
            };
            let p = line[k.clamp(0, m as isize - 1) as usize];
            [2.0 * line[end][0] - p[0], 2.0 * line[end][1] - p[1]]
        };
        let smooth: Vec<[f64; 2]> = (0..m as isize)
            .map(|i| {
                let (mut sx, mut sy, mut sw) = (0.0, 0.0, 0.0);
                for (k, &wk) in (-w..=w).zip(&kernel) {
                    let p = at(i + k);
                    sx += wk * p[0];
                    sy += wk * p[1];
                    sw += wk;
                }
                let (a, phi) = self.out(sx / sw / g[0], sy / sw / g[1]);
                [a * g[0], phi * g[1]]
            })
            .collect();
        // Spaced by length, twice as close near the ear.
        let near = |q: [f64; 2]| 1.0 + smoothstep(1.8, 1.15, self.rho(q[0] / g[0], q[1] / g[1]));
        let f: Vec<f64> = (1..=n).map(|k| k as f64 / (n + 1) as f64).collect();
        along(&smooth, &near, &f)
            .iter()
            .map(|q| (q[0] / g[0], q[1] / g[1]))
            .collect()
    }
}

/// A polyline's length.
fn length(line: &[[f64; 2]]) -> f64 {
    line.windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .sum()
}

/// The points at shares `f` of the way along a polyline, its length weighted by `w` (at each segment's end).
fn along(line: &[[f64; 2]], w: &dyn Fn([f64; 2]) -> f64, f: &[f64]) -> Vec<[f64; 2]> {
    let mut s = vec![0.0];
    for q in line.windows(2) {
        s.push(s[s.len() - 1] + (q[1][0] - q[0][0]).hypot(q[1][1] - q[0][1]) * w(q[1]));
    }
    let total = s[s.len() - 1];
    let (xs, ys): (Vec<f64>, Vec<f64>) = line.iter().map(|p| (p[0], p[1])).unzip();
    f.iter()
        .map(|&t| [interp(t * total, &s, &xs), interp(t * total, &s, &ys)])
        .collect()
}
