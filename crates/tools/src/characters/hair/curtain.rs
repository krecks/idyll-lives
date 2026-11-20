//! Curtains and fringes: hair falling from a ring.

use super::*;

/// A curtain's shape (see `curtain`).
pub(super) struct Curtain<'a> {
    pub(super) a0: f64,
    pub(super) a1: f64,
    pub(super) top: &'a dyn Fn(f64) -> f64,
    pub(super) bottom: &'a dyn Fn(f64) -> f64,
    /// The inner face's offset out from the scalp it hangs from (m), flaring by `flare` (a share) to the hem.
    pub(super) off: f64,
    pub(super) locks: usize,
    pub(super) rows: usize,
    pub(super) flare: f64,
    /// Thick in a lock's middle (m; a third of that between locks), the hem curled in by `curl`.
    pub(super) thick: f64,
    pub(super) curl: f64,
    /// A lock's tip hangs down to this much lower than the partings beside it (m), the more pointed the
    /// higher `sharp` (1: round).
    pub(super) tip: f64,
    pub(super) sharp: f64,
    pub(super) back: Option<&'a dyn Fn(f64) -> f64>,
    pub(super) wave: f64,
    pub(super) seed: i64,
    /// Wisps this long (m) hang from the hem.
    pub(super) wisp: f64,
}

/// Hair hanging round the head in locks between azimuths a0..a1, from height top(a) down to
/// bottom(a) (m) and the locks' tips below: a sheet with a thickness, rounded edges and a rounded
/// hem, coming out from under the cap, over the ears and falling straight from the head's widest. `back(y)`
/// (optional): the furthest forward its inner face may come at height y (so it clears the
/// shoulders and back).
pub(super) fn curtain(sc: &Scalp, mut c: Curtain) -> Comp {
    let h = sc.h;
    c.rows = dense_rows(c.rows);
    let cols = LOCK * c.locks + 1;
    let a = linspace(c.a0, c.a1, cols);
    // Whole locks at the edges, the partings between them.
    let lk = locks(cols, LOCK);
    let seed = c.seed as f64;
    let tt: Vec<f64> = a.iter().map(|&a| (c.top)(a)).collect();
    let bt: Vec<f64> = (0..cols)
        .map(|j| {
            (c.bottom)(a[j])
                - c.tip * (0.35 + 0.65 * jitter((j + LOCK / 2) / LOCK, seed)) * lk[j].powf(c.sharp)
        })
        .collect();
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
                    let mut p = v3(
                        h.centre[0] + a[j].sin() * rx,
                        y,
                        h.centre[2] + a[j].cos() * rz,
                    );
                    if let Some(bk) = c.back {
                        p[2] = p[2].min(bk(y));
                    }
                    // Out over the ears (the scalp it hangs from leaves them out).
                    for [ec, eh] in h.ears {
                        let w = smoothstep(eh.y + 0.015, eh.y, (p.y - ec.y).abs())
                            * smoothstep(eh.z + 0.015, eh.z, (p.z - ec.z).abs());
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
    let full =
        |j: usize| 1.0 + 0.5 * smoothstep((LOCK / 2) as f64, 0.0, j.min(cols - 1 - j) as f64);
    let half: Vec<Vec<f64>> = us
        .iter()
        .map(|&u| {
            (0..cols)
                .map(|j| {
                    0.5 * c.thick
                        * full(j)
                        * (0.35 + 0.65 * lk[j])
                        * (0.15 + 0.85 * smoothstep(0.0, 0.3, u))
                })
                .collect()
        })
        .collect();
    let mid: Vec<Vec<V3>> = (0..c.rows)
        .map(|i| {
            (0..cols)
                .map(|j| inner[i][j] + nrm[i][j] * half[i][j])
                .collect()
        })
        .collect();
    let mut out = slab(&mid, &nrm, &half, c.curl, &lock_bands(cols, seed), c.wisp);
    // The wisps' ends no nearer the back than the hair itself.
    if let Some(bk) = c.back {
        for s in &mut out.wisps {
            for (&i, by) in s.at.iter().zip(&mut s.by) {
                let tip = out.pos[i] + *by;
                by[2] = tip[2].min(bk(tip[1])) - out.pos[i][2];
            }
        }
    }
    out
}

/// A fringe's shape (see `fringe`).
pub(super) struct Fringe<'a> {
    pub(super) a0: f64,
    pub(super) a1: f64,
    pub(super) top: &'a dyn Fn(f64) -> f64,
    pub(super) bottom: &'a dyn Fn(f64) -> f64,
    /// Its inner face `off - thin` off the skin (m; more by `flare` towards the hem), at least clear of it.
    pub(super) off: f64,
    pub(super) thin: f64,
    pub(super) locks: usize,
    pub(super) rows: usize,
    /// Thick in a lock's middle (m), and how much lower a lock's tip hangs (m; pointed as `sharp`).
    pub(super) thick: f64,
    pub(super) tip: f64,
    pub(super) sharp: f64,
    pub(super) flare: f64,
    pub(super) seed: f64,
    /// Wisps this long (m) hang from the hem.
    pub(super) wisp: f64,
}

/// Hair falling over the forehead in locks, on the face as drawn (`face`: a Scalp that isn't held
/// to the cranium): between azimuths a0..a1, from polar angle top(a) (under the cap) down to height
/// bottom(a) (m) and the locks' tips below.
pub(super) fn fringe(face: &Scalp, mut c: Fringe) -> Comp {
    c.rows = dense_rows(c.rows);
    let cols = LOCK * c.locks + 1;
    let a = linspace(c.a0, c.a1, cols);
    let lk = locks(cols, LOCK);
    let t0: Vec<f64> = a.iter().map(|&a| (c.top)(a)).collect();
    let t1: Vec<f64> = (0..cols)
        .map(|j| {
            at_height(
                face,
                a[j],
                (c.bottom)(a[j])
                    - c.tip
                        * (0.5 + 0.5 * jitter((j + LOCK / 2) / LOCK, c.seed))
                        * lk[j].powf(c.sharp),
                t0[j] + 0.02,
            )
        })
        .collect();
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
    slab(&mid, &nrm, &half, 0.3, &lock_bands(cols, c.seed), c.wisp)
}

/// The polar angle (below `from`) at which the surface at azimuth a comes down to height y.
pub(super) fn at_height(sc: &Scalp, a: f64, y: f64, from: f64) -> f64 {
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
pub(super) fn face_lock(
    face: &Scalp,
    a0: f64,
    phi0: f64,
    pts: &[(f64, f64, f64)],
    wide: f64,
    thick: f64,
    seed: f64,
) -> Comp {
    let mut ctl = vec![face.point(a0, phi0, 0.4 * pts[0].2)];
    ctl.extend(
        pts.iter()
            .map(|&(a, y, off)| face.point(a, at_height(face, a, y, phi0), off)),
    );
    let path = spline(&ctl, 10);
    let t = linspace(0.0, 1.0, 10);
    let rx: Vec<f64> = t
        .iter()
        .map(|&t| thick * interp(t, &[0.0, 0.3, 1.0], &[0.8, 1.0, 0.35]))
        .collect();
    let rz: Vec<f64> = t
        .iter()
        .map(|&t| wide * interp(t, &[0.0, 0.3, 0.75, 1.0], &[0.7, 1.0, 0.7, 0.12]))
        .collect();
    let m = loft(
        &path,
        &rx,
        &rz,
        6,
        face.surface(pts[0].0, at_height(face, pts[0].0, pts[0].1, phi0))
            .1,
        caps(1),
    );
    let z = m
        .pos
        .iter()
        .enumerate()
        .map(|(i, _)| {
            clip(
                0.55 + 0.35 * ((i % 6) as f64 * PI / 3.0 + seed).cos(),
                0.0,
                1.0,
            )
        })
        .collect();
    let tip = brush(&m.pos, 6, 2, 0.5 * wide, TIP_TAPER, Some(1));
    Comp {
        wisps: vec![tip],
        ..comp(
            m.pos,
            m.idx,
            None,
            Some(z),
            Some(vec![0.1; m.t.len()]),
            None,
        )
    }
}
