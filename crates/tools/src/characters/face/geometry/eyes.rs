//! The face's eyes: eyeballs, lids, lashes and the seams where they meet the face.

use super::super::*;

impl Face {
    // --- eyes ----------------------------------------------------------------------------------

    pub(in crate::characters::face) fn eyeballs(&self) -> Mesh {
        let h = &self.h;
        let mut out = Vec::new();
        for (k, side) in ["l", "r"].into_iter().enumerate() {
            let c = h.eyes[k];
            let m = ellipsoid(c, V3::repeat(h.eye_r), 12, 16);
            let r = rot_x(PI / 2.0);
            let p: Vec<V3> = m.pos.iter().map(|q| rot(&r, q - c) + c).collect();
            let fields = p
                .iter()
                .map(|q| {
                    let front = q.z - c.z > 0.0;
                    [
                        if front { q.x - c.x } else { 1.0 },
                        if front { q.y - c.y } else { 1.0 },
                        h.iris_r * 1.08,
                        h.pupil_r * 1.1,
                    ]
                })
                .collect();
            let nor = p.iter().map(|q| (q - c) / h.eye_r).collect();
            let n = p.len();
            out.push(Mesh {
                pos: p,
                idx: m.idx,
                nor,
                fields,
                dense: skin(n, &format!("eye_{side}")),
                ..Mesh::default()
            });
        }
        merge(&out.iter().collect::<Vec<_>>())
    }

    /// Where the lids of each eye meet the face: per lid column (`lid_theta`; the upper lid's over the
    /// eye, the lower's under it, both at the eye's corners at the ends), on a line round the hollow the
    /// socket is carved into the face (head.rs `sockets`), `LID_MARGIN` past its edge and at least
    /// `LID_REACH` from the eye's axis, so the lids cover all of it.
    pub(super) fn seams(&mut self) {
        let (h, bh) = (self.h.clone(), self.base.0.clone());
        let (r, rb) = (h.eye_r, bh.eye_r);
        let skin = round_eyes(&bh, &bh.tris);
        for k in 0..2 {
            let (e, out) = (h.eyes[k], if k == 1 { 1.0 } else { -1.0 });
            for (s, sy) in [1.0, -1.0].into_iter().enumerate() {
                self.seams[k][s] = lid_theta()
                    .iter()
                    .map(|&th| {
                        let a = radians(th * 90.0 / 104.0);
                        let d = v3(out * a.sin(), sy * a.cos(), 0.0);
                        // (Found on the base stage's head, in its eye's radii, as `lid_z` is.)
                        let carved = |rho: f64| {
                            let q = bh.eyes[k] + d * rho * rb;
                            front_z(&self.base.1, q.x, q.y, bh.centre.z)
                                - front_z(&skin, q.x, q.y, f64::INFINITY)
                                > 0.02e-3 * bh.hh / ADULT_HH
                        };
                        let edge = (0..100)
                            .map(|i| 0.5 + 0.02 * i as f64)
                            .filter(|&rho| carved(rho))
                            .fold(0.0, f64::max);
                        let q = e + d * (edge + LID_MARGIN).max(LID_REACH) * r;
                        v3(q.x, q.y, self.bare_z(q.x, q.y) + LID_LIFT * self.mm)
                    })
                    .collect();
            }
        }
    }

    /// z of the face at (x, y) without the eye sockets.
    pub(super) fn bare_z(&self, x: f64, y: f64) -> f64 {
        front_z(&self.bare, x, y, self.h.centre.z)
    }

    /// z of the lids of eye k at (x, y) inside their seams: `LID_LIFT` in front of the face without its
    /// socket, or on the eyeball's bulge (`LID_BULGE`) where that stands out of it, the two blended
    /// (`LID_SOFT`): so a closed lid is the face with the eye's round under it. The face is the base
    /// stage's, as far from its eye in its eye's radii: the channels are made on the base stage and
    /// added to every stage, so a lid moves over the eye the same way at every stage and covers it.
    fn lid_z(&self, k: usize, x: f64, y: f64) -> f64 {
        let (e, r) = (self.h.eyes[k], self.h.eye_r);
        let (bh, bare) = (&self.base.0, &self.base.1);
        let (eb, rb) = (bh.eyes[k], bh.eye_r);
        let (qx, qy) = ((x - e.x) / r, (y - e.y) / r);
        let face = (front_z(bare, eb.x + qx * rb, eb.y + qy * rb, bh.centre.z) - eb.z) / rb
            + LID_LIFT * 0.001 / ADULT_HH * bh.hh / rb;
        let rho2 = qx * qx + qy * qy;
        let ball = if rho2 < LID_BULGE * LID_BULGE {
            (LID_BULGE * LID_BULGE - rho2).sqrt()
        } else {
            -1.0
        };
        e.z - r * smin(-face, -ball, LID_SOFT)
    }

    /// Upper and lower lid edges of eye k (polar angle from the top of the eye, degrees) per `lid_theta`.
    fn lid_edges(&self, p: &Params, k: usize) -> (Vec<f64>, Vec<f64>) {
        let st = &self.st;
        let lt = lid_theta();
        let mut uu = Vec::new();
        let mut ll = Vec::new();
        for &th in &lt {
            // + towards the outer corner.
            let t = th / 90.0;
            let to = t;
            let inner = smoothstep(0.1, -0.9, to);
            let (u0, l0) = (
                st.upper + 27.0 * t * t + (st.tilt + st.cant) * to,
                st.lower - 21.0 * t * t + (2.0 + st.cant) * to,
            );
            // (Overlapping by 20 degrees at most.)
            let corner = (LID_CORNER.1 * (t.abs() - LID_CORNER.0).max(0.0).powi(2))
                .min((l0 - u0) / 2.0 + 10.0);
            let (u0, l0) = (u0 + corner, l0 - corner);
            let mut u = u0 - 12.0 * p.wide[k] - 10.0 * p.look_up[k]
                + 20.0 * p.look_down[k]
                + p.brow_down[k] * (3.0 + 12.0 * inner)
                + 5.0 * p.sneer[k]
                - 5.0 * p.brow_inner * inner;
            let mut l = l0
                - 18.0 * p.squint[k] * (1.0 - 0.45 * t * t)
                - 8.0 * p.cheek[k] * (1.0 - 0.3 * t * t)
                + 8.0 * p.look_down[k]
                + 3.0 * p.wide[k]
                - 7.0 * p.sneer[k];
            let b = p.blink[k];
            if b != 0.0 {
                // (Past the corners, where the lids already meet, they stay.)
                let c = 0.52 * l0 + 0.48 * u0;
                u += b * ((c + 4.0).max(u0) - u0);
                l += b * (c.min(l0) - l0);
            }
            uu.push(u);
            ll.push(l);
        }
        (uu, ll)
    }

    /// A point on eye k's sphere frame: polar angle phi from +y, azimuth theta (deg, + outer), radius r.
    fn sph(&self, k: usize, phi: f64, theta: f64, r: f64) -> V3 {
        let c = self.h.eyes[k];
        let out = if k == 1 { 1.0 } else { -1.0 };
        let (ph, th) = (radians(phi), radians(theta) * out);
        c + v3(ph.sin() * th.sin(), ph.cos(), ph.sin() * th.cos()) * r
    }

    #[allow(clippy::type_complexity)]
    pub(in crate::characters::face) fn lids(
        &self,
        p: &Params,
        k: usize,
    ) -> (Vec<Vec<V3>>, Vec<Vec<V3>>, Vec<Vec<V3>>) {
        let (h, mm) = (&self.h, self.mm);
        let (u, l) = self.lid_edges(p, k);
        let r = h.eye_r;
        let (ru, rl) = (r * 1.105, r * 1.095);
        let lt = lid_theta();
        let n = lt.len();
        let row = |ph: &dyn Fn(usize) -> f64, rr: f64, th: &[f64]| -> Vec<V3> {
            (0..th.len())
                .map(|i| self.sph(k, ph(i), th[i], rr))
                .collect()
        };
        // Rows from the seam (s: 0 upper, 1 lower) towards the edge, on the face or the eye's bulge.
        let over = |s: usize, edge: &[V3], t: f64| -> Vec<V3> {
            self.seams[k][s]
                .iter()
                .zip(edge)
                .map(|(c, e)| {
                    let q = c + (e - c) * t;
                    v3(q.x, q.y, self.lid_z(k, q.x, q.y))
                })
                .collect()
        };
        // Past the eye's corners, where the lids meet at rest, their edges and rims lie on the face (raised
        // by `dz`) instead of on the eye at its sides.
        let (u0, l0) = self.lid_edges(&Params::default(), k);
        let shut: Vec<f64> = (0..n)
            .map(|i| smoothstep(LID_SHUT.0, LID_SHUT.1, u0[i] - l0[i]))
            .collect();
        let lift = |rim: Vec<V3>, dz: &[f64]| -> Vec<V3> {
            rim.iter()
                .zip(dz)
                .map(|(q, d)| q + v3(0.0, 0.0, *d))
                .collect()
        };
        let raised = |edge: Vec<V3>| -> (Vec<V3>, Vec<f64>) {
            let dz: Vec<f64> = edge
                .iter()
                .zip(&shut)
                .map(|(q, s)| s * (self.lid_z(k, q.x, q.y) - q.z).max(0.0))
                .collect();
            (lift(edge, &dz), dz)
        };
        // Upper: from where it meets the face over the socket down to the edge on the eye, then the
        // rim turning in to the eye.
        let (eu, dzu) = raised(row(&|i| u[i], ru, &lt));
        let [t1, t2] = LID_ROWS.0;
        let up = vec![
            self.seams[k][0].clone(),
            over(0, &eu, t1),
            over(0, &eu, t2),
            eu,
            lift(row(&|i| u[i] + 3.0, r * 1.075, &lt), &dzu),
        ];
        // Lower: from the face under the eye up to the edge, then its rim.
        let (el, dzl) = raised(row(&|i| l[i], rl, &lt));
        let lo = vec![
            self.seams[k][1].clone(),
            over(1, &el, LID_ROWS.1),
            el,
            lift(row(&|i| l[i] - 2.5, r * 1.075, &lt), &dzl),
        ];
        debug_assert_eq!(n, 9);
        // Lashes: a dark band along the upper rim, standing out and up; longer towards the outer corner.
        let st = &self.st;
        let ls = lash_theta();
        let ue: Vec<f64> = ls.iter().map(|&t| interp(t, &lt, &u)).collect();
        let gap: Vec<f64> = u.iter().zip(&l).map(|(u, l)| u - l).collect();
        let mut lash = vec![Vec::new(), Vec::new(), Vec::new()];
        for (i, &th) in ls.iter().enumerate() {
            let to = th / 90.0;
            let ln = (st.lash * (0.55 + 0.45 * smoothstep(-0.7, 0.6, to))
                + st.flick * smoothstep(0.3, 0.95, to))
                * mm;
            let lift =
                5.0 + 6.0 * smoothstep(0.2, 0.95, to) * (if st.flick > 1.0 { 1.0 } else { 0.0 });
            // (Out of the edge, and standing off the lid above it by as much as off the eye, raised by 1.5 mm
            // at most where the eye is open there: where the edge lies deep in the eye's inner corner they
            // stay in it. Where the lids meet they come out of the fold between them onto the face.)
            let dz = v3(0.0, 0.0, interp(th, &lt, &dzu));
            let shut = smoothstep(-6.0, 4.0, interp(th, &lt, &gap));
            let off = |q: V3, d: f64| {
                q + v3(
                    0.0,
                    0.0,
                    clip(
                        self.lid_z(k, q.x, q.y) + d - q.z,
                        0.0,
                        1.5 * mm + shut * h.eye_r,
                    ),
                )
            };
            lash[0].push(self.sph(k, ue[i] + 1.5, th, ru + 0.15 * mm) + dz);
            lash[1].push(off(
                self.sph(k, ue[i] - 1.0, th, ru + 0.55 * ln) + dz,
                0.55 * ln,
            ));
            lash[2].push(off(
                self.sph(k, ue[i] - lift * 0.6 - 1.0, th, ru + ln) + dz,
                ln,
            ));
        }
        (up, lo, lash)
    }
}
