//! Tufts and tousled locks.

use super::*;

/// How far a lock's tip wisps are drawn in from the ring they leave it at (`brush`).
pub(crate) const TIP_TAPER: f64 = 0.45;

/// A lock lying on the scalp from (a, phi), `len` (rad of arc) long, turned `turn` from straight
/// down the head and lifting off it by `lift` (m) at its tip: a flattened tube `wide` across and
/// `thick` deep, tapering to a point.
#[allow(clippy::too_many_arguments)]
pub(crate) fn tuft(
    sc: &Scalp,
    a: f64,
    phi: f64,
    len: f64,
    turn: f64,
    lift: f64,
    off: f64,
    wide: f64,
    thick: f64,
    seed: f64,
) -> Comp {
    let s = linspace(0.0, 1.0, 6);
    let st = phi.sin().max(0.3);
    let pts: Vec<V3> = s
        .iter()
        .map(|&s| {
            sc.point(
                a + turn.sin() * len * s / st,
                (phi + turn.cos() * len * s).max(0.05),
                off + lift * s * s,
            )
        })
        .collect();
    let rx: Vec<f64> = s
        .iter()
        .map(|&s| thick * interp(s, &[0.0, 0.3, 1.0], &[0.8, 1.0, 0.3]))
        .collect();
    let rz: Vec<f64> = s.iter().map(|&s| wide * tuft_breadth(s)).collect();
    let m = loft(&pts, &rx, &rz, 6, sc.surface(a, phi).1, caps(1));
    let k = 0.3 + 0.4 * jitter(0, seed);
    let z =
        m.t.iter()
            .map(|&t| clip(k + 0.3 * (1.0 - t), 0.0, 1.0))
            .collect();
    let y = m.t.iter().map(|&t| 0.2 * (1.0 - t).max(0.0)).collect();
    let tip = brush(&m.pos, 6, 2, 0.4 * wide, TIP_TAPER, Some(1));
    Comp {
        wisps: vec![tip],
        ..comp(m.pos, m.idx, None, Some(z), Some(y), None)
    }
}

/// A tuft's breadth along its length (s: 0 its root, 1 its tip), of `wide`.
fn tuft_breadth(s: f64) -> f64 {
    interp(s, &[0.0, 0.25, 0.7, 1.0], &[0.7, 1.0, 0.6, 0.08])
}

/// Whether a tuft (as `tuft` lays it) stays off the ears' ovals by its breadth.
fn tuft_clears(sc: &Scalp, a: f64, phi: f64, len: f64, turn: f64, wide: f64) -> bool {
    let st = phi.sin().max(0.3);
    linspace(0.0, 1.0, 11).iter().all(|&s| {
        let (ta, tp) = (
            a + turn.sin() * len * s / st,
            (phi + turn.cos() * len * s).max(0.05),
        );
        sc.oval(ta).is_none_or(|o| {
            (o.rho(ta, tp) - 1.0) * o.r[0].min(o.r[1]) >= wide * tuft_breadth(s) + EAR_CLEAR
        })
    })
}

/// The tousled style's locks as its seed lays them: each one's azimuth, how far down to the hairline it
/// grows (of the hairline's polar angle), its length, its turn, and its lift and breadth (of hh).
pub(crate) fn tousled_locks() -> Vec<[f64; 6]> {
    let mut rng = Pcg64::new(11);
    (0..22)
        .map(|k| {
            // Spread over the head on a spiral (each about as much of it), nudged.
            let a = py_mod(
                k as f64 * PI * (3.0 - 5f64.sqrt()) + rng.uniform(-0.2, 0.2) + PI,
                TAU,
            ) - PI;
            let down = 0.82 * ((k as f64 + 0.5) / 22.0).sqrt() + rng.uniform(-0.05, 0.05);
            let len = rng.uniform(0.4, 0.65);
            let turn = rng.uniform(-0.65, 0.65);
            let lift = rng.uniform(0.015, 0.04);
            let wide = rng.uniform(0.055, 0.075);
            [a, down, len, turn, lift, wide]
        })
        .collect()
}

/// A tousled lock's length on a head: one that would lie on an ear is cut shorter until it ends above it
/// (`tuft_clears`); none, if it grows from beside one.
pub(crate) fn tousled_cut(sc: &Scalp, hh: f64, lock: [f64; 6]) -> Option<f64> {
    let [a, down, len, turn, _, wide] = lock;
    let phi = sc.hairline(a) * down;
    (0..8)
        .map(|cut| len * 0.85f64.powi(cut))
        .find(|&len| tuft_clears(sc, a, phi, len, turn, wide * hh))
}

static TOUSLED: Cache<String, Vec<bool>> = Cache::new();

/// Which of the tousled locks a family has: those that clear the ears on its base stage. Every stage has
/// the same ones (the style is one topology, and the ears' ovals differ a little from stage to stage).
pub(crate) fn tousled_kept(family: &str) -> Arc<Vec<bool>> {
    TOUSLED.get(family.to_string(), || {
        let h = head_frame(family, BASE_STAGE);
        let sc = Scalp::new(&h, true).round(&Ears::of(family, BASE_STAGE, &h));
        tousled_locks()
            .into_iter()
            .map(|lock| tousled_cut(&sc, h.hh, lock).is_some())
            .collect()
    })
}
