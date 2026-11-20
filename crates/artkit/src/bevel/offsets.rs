//! Evens the widths along each run of new points.
//!
//! Split from `bevel.rs` (see `mod.rs`).

use super::*;

/// Evens the widths along each run of new points that slid along an edge: where a point slides, the cut on
/// one side of it is tied to the cut on the other, so the offsets asked for can't all be met; each run (a chain
/// between two points that didn't slide, or a ring) is given the offsets that fit together.
pub(crate) fn adjust_offsets(corners: &mut BTreeMap<u32, Corner>) {
    let keys: Vec<u32> = corners.keys().copied().collect();
    // the point at the far end of bevelled edge `half` of `v`, on the same side of it
    let other = |corners: &BTreeMap<u32, Corner>,
                 v: u32,
                 half: usize,
                 left: bool|
     -> Option<(u32, usize)> {
        let to = corners[&v].edges[half].to;
        let c = corners.get(&to)?;
        let h = &c.edges[c.half_to(v)];
        Some((to, if left { h.left } else { h.right }))
    };
    for &bv in &keys {
        for anchor in 0..corners[&bv].bounds.len() {
            let b = &corners[&bv].bounds[anchor];
            if b.visited || b.eon.is_none() {
                continue;
            }
            // along the edges after each point, then back along those before the first
            let mut v = (bv, anchor);
            let (mut start, mut end) = (v, v);
            let (mut ring, mut len) = (false, 1);
            loop {
                let b = &corners[&v.0].bounds[v.1];
                if b.eon.is_none() || b.visited || ring {
                    break;
                }
                let efirst = b.efirst;
                corners.get_mut(&v.0).expect("a corner").bounds[v.1].visited = true;
                let Some(next) = other(corners, v.0, efirst, true) else {
                    break;
                };
                corners.get_mut(&v.0).expect("a corner").bounds[v.1].chain = Some(next);
                end = next;
                len += 1;
                if corners[&next.0].bounds[next.1].visited {
                    if next != start {
                        break;
                    }
                    adjust_run(corners, start, true);
                    ring = true;
                }
                v = next;
            }
            if !ring {
                corners.get_mut(&v.0).expect("a corner").bounds[v.1].chain = None;
                v = start;
                loop {
                    corners.get_mut(&v.0).expect("a corner").bounds[v.1].visited = true;
                    let elast = corners[&v.0].bounds[v.1].ebev;
                    let Some(next) = other(corners, v.0, elast, false) else {
                        break;
                    };
                    corners.get_mut(&next.0).expect("a corner").bounds[next.1].chain = Some(v);
                    start = next;
                    len += 1;
                    v = next;
                    let b = &corners[&v.0].bounds[v.1];
                    if b.visited || b.eon.is_none() {
                        break;
                    }
                }
                let slid = |p: (u32, usize)| corners[&p.0].bounds[p.1].eon.is_some();
                if len >= 3 && !slid(start) && !slid(end) {
                    adjust_run(corners, start, false);
                }
            }
        }
    }
}

/// One run's offsets: the least squares of each edge's two ends differing in width and, a fifth as heavy, of
/// each offset differing from what it was.
pub(crate) fn adjust_run(corners: &mut BTreeMap<u32, Corner>, start: (u32, usize), ring: bool) {
    let mut run = vec![start];
    while let Some(next) = corners[&run[run.len() - 1].0].bounds[run[run.len() - 1].1].chain {
        if next == start {
            break;
        }
        run.push(next);
    }
    let np = run.len();
    let bound = |p: (u32, usize)| &corners[&p.0].bounds[p.1];
    let ratio: Vec<f64> = run.iter().map(|&p| bound(p).sinratio as f64).collect();
    let right = |p: (u32, usize)| corners[&p.0].edges[bound(p).efirst].offset_r as f64;
    let left = |p: (u32, usize)| corners[&p.0].edges[bound(p).ebev].offset_l as f64;
    let rows = if ring { 3 * np } else { 3 * np - 3 };
    let mut a = vec![vec![0.0_f64; np]; rows];
    let mut rhs = vec![0.0_f64; rows];
    let weight = 0.2_f64;
    for i in 0..np {
        if ring || i < np - 1 {
            a[i][i] += 1.0;
            if ring {
                a[if i > 0 { i - 1 } else { np - 1 }][i] += -ratio[i];
            } else if i > 0 {
                a[i - 1][i] += -ratio[i];
            }
            let row = if ring { np + 2 * i } else { np - 1 + 2 * i };
            a[row][i] += weight;
            rhs[row] += weight * right(run[i]);
            let next = if i == np - 1 { 0 } else { i + 1 };
            a[row + 1][next] += weight * ratio[next];
            rhs[row + 1] += weight * left(run[next]);
        } else {
            a[i - 1][i] += -1.0;
        }
    }
    // the normal equations, by elimination
    let mut m = vec![vec![0.0_f64; np + 1]; np];
    for r in 0..rows {
        for i in 0..np {
            if a[r][i] == 0.0 {
                continue;
            }
            for j in 0..np {
                m[i][j] += a[r][i] * a[r][j];
            }
            m[i][np] += a[r][i] * rhs[r];
        }
    }
    for c in 0..np {
        let pivot = (c..np)
            .max_by(|&x, &y| m[x][c].abs().total_cmp(&m[y][c].abs()))
            .expect("a row");
        if m[pivot][c].abs() < 1e-300 {
            return;
        }
        m.swap(c, pivot);
        for r in 0..np {
            if r != c && m[r][c] != 0.0 {
                let k = m[r][c] / m[c][c];
                for j in c..=np {
                    m[r][j] -= k * m[c][j];
                }
            }
        }
    }
    let x: Vec<f64> = (0..np).map(|i| m[i][np] / m[i][i]).collect();
    let ends: Vec<(usize, usize, f32)> = run
        .iter()
        .map(|&p| (bound(p).efirst, bound(p).ebev, bound(p).sinratio))
        .collect();
    for (i, &p) in run.iter().enumerate() {
        let (efirst, elast, r) = ends[i];
        let corner = corners.get_mut(&p.0).expect("a corner");
        if ring || i < np - 1 {
            corner.edges[efirst].offset_r = x[i] as f32;
            if ring || i > 0 {
                corner.edges[elast].offset_l = (r as f64 * x[i]) as f32;
            }
        } else {
            corner.edges[elast].offset_l = x[i] as f32;
        }
    }
}
