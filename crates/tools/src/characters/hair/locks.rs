//! Locks: bent capsules, grooved, shingled and laid loose.
//!
//! Split from `characters/hair.rs` (see `mod.rs`).

use super::*;

/// A bent capsule (lofted along a smooth curve through `points`); `hang` (a, b): it follows the
/// body more and more from a to b along its length (a tail).
#[allow(clippy::too_many_arguments)]
pub(crate) fn lobe(
    points: &[V3],
    ry: &[f64],
    rz: &[f64],
    segs: usize,
    hint: V3,
    bands: f64,
    seed: f64,
    y: f64,
    hang: Option<(f64, f64)>,
) -> Comp {
    let m = loft(points, ry, rz, segs, hint, caps(2));
    let n = m.pos.len();
    let z = (0..n)
        .map(|i| {
            0.5 + 0.5 * ((i % segs) as f64 / segs as f64 * TAU * 2.0 + seed + m.t[i] * bands).sin()
        })
        .collect();
    let follow = hang.map(|(a, b)| m.t.iter().map(|&t| smoothstep(a, b, t)).collect());
    let tip = brush(&m.pos, segs, 3, 2.5 * ry[ry.len() - 1], TIP_TAPER, None);
    Comp {
        wisps: vec![tip],
        ..comp(m.pos, m.idx, None, Some(z), Some(vec![y; n]), follow)
    }
}

/// A loft's rings (`segs` points each, laid one after the other) in `grooves` locks round it: each
/// point pushed out from its ring's middle by up to `depth` (a share) on a lock and in between
/// them, twisting by `twist` (rad) along its length (t). Clump bands follow the locks.
pub(crate) fn grooved(
    m: &mut super::prims::Prim,
    segs: usize,
    grooves: f64,
    depth: f64,
    twist: f64,
) -> Vec<f64> {
    let rings = m.t.len() / segs;
    let mut z = vec![0.5; m.pos.len()];
    for r in 0..rings {
        let c = mean_rows(&m.pos[r * segs..(r + 1) * segs]);
        for q in 0..segs {
            let i = r * segs + q;
            let g = (TAU * q as f64 / segs as f64 * grooves + twist * m.t[i]).cos();
            m.pos[i] = c + (m.pos[i] - c) * (1.0 + depth * g);
            z[i] = 0.5 + 0.4 * g;
        }
    }
    z
}

/// Wisps over a shell of rings of `cols` points (a cap that stands off the head: its outline is no
/// edge to grow wisps from), laid like shingles: from every `every`th ring a row round the shell (on
/// every third column), each quad on down over the rings under it, a ring short of the next row's
/// roots, and lifting off the shell by `lift` (m) on the way: tufts standing half up, so the shell's
/// outline, whichever way it is seen, is strands' ends. Their strands are loose curls. (The rings
/// come lowest first; `first`: the lowest one the wisps come down to.)
pub(crate) fn shingles(
    pos: &[V3],
    centre: V3,
    cols: usize,
    first: usize,
    every: usize,
    lift: f64,
) -> Vec<Strip> {
    let rings = pos.len() / cols;
    let reach = every - 1;
    (first + reach..rings)
        .step_by(every)
        .map(|r| {
            let at: Vec<usize> = (0..cols).step_by(3).map(|k| r * cols + k).collect();
            Strip {
                by: at
                    .iter()
                    .map(|&i| {
                        let to = pos[i - reach * cols];
                        to + (to - centre).normalize() * lift - pos[i]
                    })
                    .collect(),
                rooted: vec![0.0; at.len()],
                at,
                closed: true,
                loose: 1.0,
            }
        })
        .collect()
}

/// A component that hangs loose (`Comp::sway`): a curtain past the shoulders, a tail, a braid.
pub(crate) fn loose(mut c: Comp) -> Comp {
    c.sway = 1.0;
    c
}

/// A torus as a component (`_tor`).
pub(crate) fn tor(t: super::prims::Prim) -> Comp {
    let n = t.pos.len();
    let z = (0..n).map(|i| 0.3 + 0.2 * (i as f64 * 0.7).sin()).collect();
    comp(t.pos, t.idx, None, Some(z), Some(vec![0.4; n]), None)
}
