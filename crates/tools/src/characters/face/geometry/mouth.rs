//! The face's mouth: lips, plate, teeth and tongue, shaped by `Params`.

use super::super::*;

impl Face {
    // --- mouth ---------------------------------------------------------------------------------

    /// Lip centre lines and radii, the inner plate, teeth and tongue (positions) for parameters p.
    #[allow(clippy::type_complexity)]
    pub(in crate::characters::face) fn mouth(
        &self,
        p: &Params,
    ) -> ([Lip; 2], Vec<Vec<V3>>, [Vec<Vec<V3>>; 2], Vec<Vec<V3>>) {
        let (h, mm, st, hw) = (&self.h, self.mm, &self.st, self.hw);
        let s = lip_s();
        let n = s.len();
        let col = |f: &dyn Fn(usize) -> f64| -> Vec<f64> { (0..n).map(f).collect() };
        let ml = col(&|i| smoothstep(0.35, -0.35, s[i]));
        let sv = |v: [f64; 2], i: usize| v[0] * ml[i] + v[1] * (1.0 - ml[i]);
        let cw = col(&|i| s[i] * s[i]);
        let cen = col(&|i| clip(1.0 - s[i] * s[i], 0.0, 1.0));
        let o = col(&|i| cen[i].powf(0.7));
        let rc = col(&|i| cen[i].sqrt());
        let x = col(&|i| {
            let (sm, fr, st_, dim) = (
                sv(p.smile, i),
                sv(p.frown, i),
                sv(p.stretch, i),
                sv(p.dimple, i),
            );
            let x = s[i] * hw * (1.0 - 0.34 * p.funnel - 0.45 * p.pucker)
                + sign(s[i])
                    * s[i].abs().powf(1.5)
                    * mm
                    * (3.5 * sm + 4.0 * st_ + 1.5 * dim - 1.0 * fr);
            x - 4.0 * mm * p.left
        });
        let y = col(&|i| {
            let (sm, fr, st_, dim) = (
                sv(p.smile, i),
                sv(p.frown, i),
                sv(p.stretch, i),
                sv(p.dimple, i),
            );
            h.mouth[1]
                + mm * (cw[i] * (0.6 + 6.5 * sm - 7.0 * fr - 1.8 * st_ + 0.8 * dim)
                    + 2.2 * p.shrug_l * cen[i])
        });
        let dz = col(&|i| -mm * cw[i] * (2.0 * sv(p.dimple, i) + 1.0 * sv(p.smile, i)));
        let low = col(&|i| {
            let low = mm
                * (JAW * p.jaw * o[i]
                    + 4.5 * sv(p.low_down, i) * o[i].powf(0.6)
                    + 5.0 * p.funnel * rc[i]
                    - 0.8 * JAW * p.close * o[i]);
            low - 1.2 * mm * p.tuck * o[i]
        });
        let up = col(&|i| {
            mm * (3.0 * p.jaw * o[i]
                + 4.5 * sv(p.up_up, i) * o[i].powf(0.6)
                + 1.5 * p.shrug_u * o[i]
                + 2.5 * sv(p.sneer, i) * o[i].sqrt()
                + 3.0 * p.funnel * rc[i]
                + 1.6 * p.tuck * o[i])
        });
        let taper = col(&|i| 0.22 + 0.78 * cen[i].powf(0.55));
        let ry_u = col(&|i| {
            st.lip_u
                * mm
                * taper[i]
                * (1.0 - 0.4 * sv(p.press, i) - 0.55 * p.roll_u + 0.25 * p.pucker + 0.1 * p.funnel)
        });
        let ry_l = col(&|i| {
            st.lip_l
                * mm
                * taper[i]
                * (1.0 - 0.4 * sv(p.press, i) - 0.55 * p.roll_l
                    + 0.25 * p.pucker
                    + 0.1 * p.funnel
                    + 0.15 * p.shrug_l
                    - 0.3 * p.tuck)
        });
        let rz = col(&|i| st.lip_z * mm * (0.35 + 0.65 * rc[i]) * (1.0 + 0.25 * p.pucker));
        let fwd = col(&|i| mm * rc[i] * (4.5 * p.funnel + 5.5 * p.pucker));
        let proud = col(&|i| st.proud * mm * (0.3 + 0.7 * rc[i]));
        let yu = col(&|i| y[i] + up[i] + 0.75 * ry_u[i]);
        let yl = col(&|i| y[i] - low[i] - 0.75 * ry_l[i]);
        let zu = col(&|i| {
            self.mz(x[i], yu[i]) + proud[i] + fwd[i] + dz[i] - 1.5 * mm * p.roll_u * rc[i]
        });
        let zl = col(&|i| {
            self.mz(x[i], yl[i])
                + proud[i]
                + fwd[i]
                + dz[i]
                + mm * rc[i] * (3.0 * p.jaw_fwd - 2.5 * p.tuck - 1.5 * p.roll_l + 1.0 * p.shrug_l)
        });
        // The cupid's bow: the upper lip's top edge raised at its peaks (its middle line and radius up
        // by the same, so the line between the lips stays).
        let bw = col(&|i| BOW * ry_u[i] * bow(s[i]));
        let lips = [
            (
                (0..n).map(|i| v3(x[i], yu[i] + bw[i], zu[i])).collect(),
                (0..n).map(|i| ry_u[i] + bw[i]).collect(),
                rz.clone(),
            ),
            (
                (0..n).map(|i| v3(x[i], yl[i], zl[i])).collect(),
                ry_l.clone(),
                rz,
            ),
        ];
        // The opening: between the lips' inner lines (a little inside each lip, so its edge hides).
        let up_in = col(&|i| y[i] + up[i] + 0.4 * ry_u[i]);
        let low_in = col(&|i| y[i] - low[i] - 0.4 * ry_l[i]);
        // Everything inside sits on the plate's depth (which comes forward with pursed lips).
        let pz = |xx: f64, yy: f64| self.mz(xx, yy) + 0.5 * mm + 0.4 * interp(xx, &x, &fwd);
        let plate = linspace(0.0, 1.0, 4)
            .iter()
            .map(|&v| {
                (0..n)
                    .map(|i| {
                        let py = low_in[i] + (up_in[i] - low_in[i]) * v;
                        v3(x[i], py, pz(x[i], py))
                    })
                    .collect()
            })
            .collect();
        let clamp = |xx: f64, yy: f64| yy.max(interp(xx, &x, &low_in)).min(interp(xx, &x, &up_in));
        // Upper teeth: fixed to the skull; lower teeth: on the jaw.
        let mx = h.mouth[1];
        let ts = teeth_s();
        let tx: Vec<f64> = ts.iter().map(|t| t * h.mouth_w / 2.0).collect();
        let ub: Vec<f64> = ts
            .iter()
            .map(|t| mx - 0.8 * mm + 1.4 * mm * t * t)
            .collect();
        let drop = 0.86 * JAW * p.jaw * mm;
        let lt: Vec<f64> = ts
            .iter()
            .map(|t| mx - 1.4 * mm - drop + 0.6 * mm * t * t - 1.2 * mm * p.tuck)
            .collect();
        let teeth_row = |ys: &dyn Fn(usize) -> f64, zf: f64| -> Vec<V3> {
            (0..ts.len())
                .map(|i| {
                    let yy = clamp(tx[i], ys(i));
                    v3(
                        tx[i],
                        yy,
                        pz(tx[i], yy) + (1.5 - 0.8 * ts[i] * ts[i] + zf) * mm,
                    )
                })
                .collect()
        };
        let zf_l = 3.0 * p.jaw_fwd - 1.0 * p.tuck;
        let teeth = [
            vec![
                teeth_row(&|i| ub[i], 0.0),
                teeth_row(&|i| ub[i] + 7.0 * mm, 0.0),
            ],
            vec![
                teeth_row(&|i| lt[i] - 6.0 * mm, zf_l),
                teeth_row(&|i| lt[i], zf_l),
            ],
        ];
        // Tongue: on the floor of the mouth, rising and coming forward between the teeth for TH.
        let gs = tongue_s();
        let gx: Vec<f64> = gs.iter().map(|g| g * hw).collect();
        let bot: Vec<f64> = gx.iter().map(|&g| interp(g, &x, &low_in)).collect();
        let top: Vec<f64> = gs
            .iter()
            .zip(&gx)
            .map(|(g, &gxx)| {
                let q = g / 0.6;
                clamp(
                    gxx,
                    mx - mm * (2.5 + 0.6 * JAW * p.jaw) + mm * (9.0 * p.tongue)
                        - 3.0 * mm * (q * q),
                )
            })
            .collect();
        let tall: Vec<f64> = (0..gs.len())
            .map(|i| clip((top[i] - bot[i]) / (4.0 * mm), 0.0, 1.0))
            .collect();
        let tongue = [(0.0, 0.0), (0.55, 0.6), (1.0, 0.25)]
            .iter()
            .map(|&(f, bulge)| {
                (0..gs.len())
                    .map(|i| {
                        let yy = bot[i] + (top[i] - bot[i]) * f;
                        v3(
                            gx[i],
                            yy,
                            pz(gx[i], yy) + (0.4 + (bulge + 2.2 * p.tongue * f) * tall[i]) * mm,
                        )
                    })
                    .collect()
            })
            .collect();
        (lips, plate, teeth, tongue)
    }
}
