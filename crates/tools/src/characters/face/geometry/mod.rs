//! The face's geometry: its skin and the pieces laid on it, by feature (see `face`).

use super::*;

mod brows;
mod eyes;
mod mouth;

impl Face {
    /// The face of `head`; `base`: the base stage's head the lids are laid out on (`lid_z`, `seams`), `id`: a
    /// face of its own (identity.rs: then `head` is that face's head, and `base` too).
    pub(in crate::characters::face) fn new(
        family: &str,
        head: Arc<HeadFrame>,
        base: Arc<HeadFrame>,
        id: &Identity,
    ) -> Self {
        let mm = 0.001 * head.hh / ADULT_HH;
        let hw = head.mouth_w / 2.0 * 0.92;
        let bare = round_eyes(&head, &head.bare);
        let base = (base.clone(), round_eyes(&base, &base.bare));
        let mut f = Face {
            h: head,
            st: style_of(family, id),
            mm,
            hw,
            m0: (0.0, 0.0),
            mcoef: Vec::new(),
            mfloor: 0.0,
            bare,
            base,
            seams: Default::default(),
        };
        f.fit_mouth();
        f.seams();
        f
    }

    // --- the skin ------------------------------------------------------------------------------

    /// z of the face's skin at (x, y) (refined on the head's distance field).
    pub(super) fn surf(&self, x: f64, y: f64) -> f64 {
        let h = &self.h;
        let mut p = v3(x, y, h.surface_z(x, y));
        for _ in 0..4 {
            let d = h.sdf(p);
            let dz = (h.sdf(p + v3(0.0, 0.0, 1e-4)) - d) / 1e-4;
            p[2] -= d / dz.max(0.3);
        }
        p[2]
    }

    /// A point moved onto the head's surface along the skull ellipsoid's normal (smooth, whatever
    /// the head's tessellation), then `lift` (m) out along it.
    pub(super) fn project(&self, p: V3, lift: f64) -> V3 {
        let h = &self.h;
        let n = (p - h.centre).component_div(&h.radii.component_mul(&h.radii));
        let n = n / n.norm();
        let (mut lo, mut hi) = (-0.04, 0.04);
        for _ in 0..30 {
            let mid = (lo + hi) / 2.0;
            if h.sdf(p + n * mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        p + n * ((lo + hi) / 2.0 + lift)
    }

    /// A smooth stand-in for the skin round the mouth (a quadratic in x, y fitted to it), raised
    /// so the real skin is nowhere in front of it: the mouth's pieces are placed on it, so they are
    /// smooth however the head is tessellated and never sink into it.
    fn fit_mouth(&mut self) {
        let (mm, hw) = (self.mm, self.hw);
        let my = self.h.mouth[1];
        let gx: Vec<f64> = linspace(-1.35, 1.35, 15).iter().map(|v| v * hw).collect();
        let gy: Vec<f64> = linspace(-34.0, 12.0, 14)
            .iter()
            .map(|v| my + v * mm)
            .collect();
        self.m0 = (self.h.mouth[0], my);
        let mut z = Vec::new();
        let mut rows = Vec::new();
        for &y in &gy {
            for &x in &gx {
                z.push(self.surf(x, y));
                rows.push(self.feat(x, y));
            }
        }
        let f = DMatrix::from_fn(rows.len(), 7, |i, k| rows[i][k]);
        self.mcoef = lstsq(&f, &DVector::from_vec(z.clone()))
            .iter()
            .copied()
            .collect();
        let worst = rows
            .iter()
            .zip(&z)
            .map(|(r, z)| z - dot7(r, &self.mcoef))
            .fold(f64::NEG_INFINITY, f64::max);
        self.mfloor = worst.max(0.0);
    }

    fn feat(&self, x: f64, y: f64) -> [f64; 7] {
        let u = (x - self.m0.0) / self.hw;
        let v = (y - self.m0.1) / self.hw;
        [1.0, u * u, u.powf(4.0), v, v * v, v.powf(3.0), u * u * v]
    }

    /// z of the smoothed skin round the mouth.
    pub(super) fn mz(&self, x: f64, y: f64) -> f64 {
        dot7(&self.feat(x, y), &self.mcoef) + self.mfloor
    }
}
