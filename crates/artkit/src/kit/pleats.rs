//! Pleated cloth: a curtain's folds as rows of points.

use crate::math::{P3, PI, sin};
use crate::place::GameFrame;
use crate::shapes::options;

options! {
    /// `pleats`' options: `per_fold` points across each fold; `along_x`: the cloth runs along game x, its folds
    /// swinging in depth about `at` (else along depth, swinging in x).
    Pleats { per_fold: usize = 4, along_x: bool = true }
}

/// A pleated cloth `t` thick from `from` to `to` along its axis, hanging in the plane at `at`: `rows` (height, how
/// far its `folds` swing of `amp`, how thick of `t`, how far they drift, radians), from the hem up, each row a ring
/// round the cloth's front and back. Returns its vertices and faces: the cloth's quads, row by row, then a strip
/// across the cloth's thickness at the bottom and at the top (`2 × folds × per_fold` faces), for the family to make
/// into its own object (a mesh smooth but for its hems, or one sharp past an angle).
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn pleats(
    frame: &GameFrame,
    rows: &[(f64, f64, f64, f64)],
    from: f64,
    to: f64,
    at: f64,
    folds: usize,
    amp: f64,
    t: f64,
    o: Pleats,
) -> (Vec<P3>, Vec<Vec<u32>>) {
    let n = folds * o.per_fold;
    let (mut verts, mut m) = (Vec::new(), 0);
    for &(y, af, tf, phase) in rows {
        let front: Vec<(f64, f64)> = (0..=n)
            .map(|k| {
                (
                    from + (to - from) * k as f64 / n as f64,
                    amp * af * sin(2.0 * PI * folds as f64 * k as f64 / n as f64 + phase)
                        + t * tf / 2.0,
                )
            })
            .collect();
        let mut ring = front.clone();
        ring.extend(front.iter().rev().map(|&(u, d)| (u, d - t * tf)));
        m = ring.len() as u32;
        verts.extend(ring.iter().map(|&(u, d)| {
            if o.along_x {
                frame.at(u, y, at + d)
            } else {
                frame.at(at + d, y, u)
            }
        }));
    }
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for r in 0..rows.len() as u32 - 1 {
        let (a, b) = (r * m, (r + 1) * m);
        faces.extend((0..m).map(|i| vec![a + i, a + (i + 1) % m, b + (i + 1) % m, b + i]));
    }
    let n = n as u32;
    for base in [0, (rows.len() as u32 - 1) * m] {
        // the bottom and top edges: a strip of quads across the cloth's thickness
        faces.extend(
            (0..n).map(|k| vec![base + k, base + k + 1, base + m - 2 - k, base + m - 1 - k]),
        );
    }
    (verts, faces)
}

/// `kit::pleats` with its options by name: `kit::pleats!(frame, rows, from, to, at, folds, amp, t, per_fold = 8, along_x = false)`.
#[macro_export]
macro_rules! kit_pleats {
    ($frame:expr, $rows:expr, $from:expr, $to:expr, $at:expr, $folds:expr, $amp:expr, $t:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = (&$frame, $rows, $from, $to, $at, $folds, $amp, $t);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Pleats::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::pleats(args.0, args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), args.5 as usize, $crate::forms::num(args.6), $crate::forms::num(args.7), o)
    }};
}
