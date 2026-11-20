//! The beard.
//!
//! Split from `characters/hair.rs` (see `mod.rs`).

use super::*;

/// A shell over the jaw, chin and upper lip with a hole round the lips (cells inside it collapse
/// onto its edge), in locks: thicker down their middles, their tips hanging below the jaw at the
/// chin, thinning to the skin at its lower edge.
pub(crate) fn beard(sc: &Scalp) -> Comp {
    let h = sc.h;
    let hh = h.hh;
    let (cols, rows) = (25, 12);
    let a = linspace(-radians(94.0), radians(94.0), cols);
    let lk = locks(cols, LAYOUT_LOCK);
    // Mouth direction (polar angle) and the hole round it.
    let qm = (h.mouth - h.centre).component_div(&h.radii);
    let phm = (qm[1] / qm.norm()).acos();
    let top: Vec<f64> = a
        .iter()
        .map(|&a| {
            interp(
                a.abs(),
                &[0.0, 22.0, 45.0, 70.0, 85.0, 94.0].map(radians),
                &[
                    phm - 0.17,
                    phm - 0.15,
                    phm - 0.02,
                    phm - 0.1,
                    radians(96.0),
                    radians(94.0),
                ],
            )
        })
        .collect();
    // The lower edge: just under the jaw, rising to the jaw's corners at the sides; the locks' tips
    // lower round the chin.
    let bot: Vec<f64> = (0..cols)
        .map(|j| {
            let tip = 0.035
                * hh
                * lk[j].powf(2.5)
                * (0.6 + 0.4 * jitter((j + 2) / 4, 5.0))
                * smoothstep(radians(70.0), radians(30.0), a[j].abs());
            let want = h.chin_y
                + hh * interp(
                    a[j].abs(),
                    &[0.0, 45.0, 80.0, 94.0].map(radians),
                    &[-0.01, 0.02, 0.15, 0.22],
                )
                - tip;
            at_height(sc, a[j], want, top[j] + 0.1)
        })
        .collect();
    let us = linspace(0.0, 1.0, rows);
    // The hole: an ellipse in (side distance, polar angle) round the mouth.
    let wx = (h.mouth_w / 2.0 + 0.04 * hh) / h.radii[0];
    // The sideburns end in front of the ears (their boxes, EAR_CLEAR and the beard's thin edge clear).
    let side = h
        .ears
        .iter()
        .map(|[c, e]| (c.x.abs() - e.x).atan2(c.z + e.z + EAR_CLEAR + CLEAR - h.centre[2]))
        .fold(radians(94.0), f64::min);
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
            let a2 = if inside {
                clip(hx * scale * wx / phi.sin().max(1e-3), -1.0, 1.0).asin()
            } else {
                aa
            };
            let phi2 = if inside { phm + hy * scale * sy } else { phi };
            // The sides (past 60 degrees) drawn in to stop in front of the ears.
            let a2 = if a2.abs() > radians(60.0) {
                a2.signum()
                    * interp(
                        a2.abs(),
                        &[radians(60.0), radians(94.0)],
                        &[radians(60.0), side],
                    )
            } else {
                a2
            };
            let edge = smoothstep(1.0, 1.6, r)
                .min(smoothstep(0.0, 0.2, u))
                .min(smoothstep(radians(94.0), radians(78.0), aa.abs()))
                .min(smoothstep(1.0, 0.8, u));
            let off = CLEAR + (0.02 * hh) * edge * (0.65 + 0.35 * lk[j]);
            row.push(sc.point(a2, phi2, off));
            z.push(0.5 + (0.35 * lk[j] - 0.1) * (0.7 + 0.3 * (phi * 8.0 + aa * 3.0).sin()));
            y.push(0.2 * (1.0 - edge) + 0.2 * (1.0 - lk[j]));
        }
        pg.push(row);
    }
    let (pos, idx) = grid(&pg, false);
    let out = pos.iter().map(|q| q - h.centre).collect();
    // Wisps: below its lower edge (none at the sideburns' ends, by the ears), and up each cheek from
    // its upper edge (not under the nose or up the sideburns), each on along the shell from its edge.
    let on = |r: usize, from: usize, j: usize| (pg[r][j] - pg[from][j]).normalize();
    let lower = Strip {
        at: (0..cols).map(|j| (rows - 1) * cols + j).collect(),
        by: (0..cols)
            .map(|j| {
                on(rows - 1, rows - 2, j)
                    * (0.035 * hh * smoothstep(radians(94.0), radians(70.0), a[j].abs()))
            })
            .collect(),
        closed: false,
        loose: 0.0,
        rooted: vec![0.0; cols],
    };
    let cheek = |side: f64| {
        let at: Vec<usize> = (0..cols)
            .filter(|&j| (radians(30.0)..radians(75.0)).contains(&(a[j] * side)))
            .collect();
        Strip {
            by: at.iter().map(|&j| on(0, 1, j) * (0.02 * hh)).collect(),
            rooted: vec![0.0; at.len()],
            at,
            closed: false,
            loose: 0.0,
        }
    };
    Comp {
        wisps: vec![lower, cheek(-1.0), cheek(1.0)],
        ..comp(pos, idx, Some(out), Some(z), Some(y), None)
    }
}
