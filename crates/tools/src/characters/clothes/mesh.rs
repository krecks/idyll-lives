//! The mesh helpers: turning triangles outward and stripping rows into shells.

use super::*;

/// Triangles turned to face away from a vertical axis through `centre` (x, z) where most don't.
pub(super) fn outward(pos: &[V3], idx: Vec<usize>, centre: (f64, f64)) -> Vec<usize> {
    let mut wrong = 0usize;
    let nt = idx.len() / 3;
    for t in idx.chunks(3) {
        let n = (pos[t[1]] - pos[t[0]]).cross(&(pos[t[2]] - pos[t[0]]));
        let c = (pos[t[0]] + pos[t[1]] + pos[t[2]]) / 3.0;
        let radial = v3(c.x - centre.0, 0.0, c.z - centre.1);
        if n.x * radial.x + n.y * radial.y + n.z * radial.z < 0.0 {
            wrong += 1;
        }
    }
    if wrong as f64 / nt as f64 > 0.5 {
        flip(&idx)
    } else {
        idx
    }
}

pub(super) fn flip(idx: &[usize]) -> Vec<usize> {
    idx.chunks(3).flat_map(|t| [t[0], t[2], t[1]]).collect()
}

/// A closed mesh's triangles, turned round if they enclose a negative volume.
pub(super) fn closed_outward(pos: &[V3], idx: Vec<usize>) -> Vec<usize> {
    let v: Vec<f64> = idx
        .chunks(3)
        .map(|t| pos[t[0]].dot(&pos[t[1]].cross(&pos[t[2]])))
        .collect();
    if super::np::pairwise_sum(&v) < 0.0 {
        flip(&idx)
    } else {
        idx
    }
}

/// Points where the mesh's edges (of triangles whose corners are all in `keep`) cross height y.
pub(super) fn section(pos: &[V3], tris: &[usize], y: f64, keep: &[bool]) -> Vec<V3> {
    let t: Vec<&[usize]> = tris
        .chunks(3)
        .filter(|t| t.iter().all(|&v| keep[v]))
        .collect();
    let mut out = Vec::new();
    for (i, j) in [(0, 1), (1, 2), (2, 0)] {
        for tt in &t {
            let (a, bb) = (pos[tt[i]], pos[tt[j]]);
            if (a.y - y) * (bb.y - y) < 0.0 {
                let f = (y - a.y) / (bb.y - a.y);
                out.push(a + (bb - a) * f);
            }
        }
    }
    out
}

/// Skin weights from the nearest body vertices.
pub(super) fn follow(pos: &[V3], s: &super::body::Surface) -> Vec<Dense> {
    let tree = KdTree::new(&s.pos);
    pos.iter()
        .map(|p| {
            let nb = tree.query(p, 4);
            let w: Vec<f64> = nb.iter().map(|&(d, _)| 1.0 / d.max(1e-4)).collect();
            let ws = w.iter().fold(0.0, |a, v| a + v);
            let mut d = [0.0; NB];
            for (&(_, i), wi) in nb.iter().zip(&w) {
                let wi = wi / ws;
                for (dk, sk) in d.iter_mut().zip(&s.dense[i]) {
                    *dk += sk * wi;
                }
            }
            d
        })
        .collect()
}

/// Quads of a grid of points (rows along the strip's length).
pub(super) fn strip(rows: &[Vec<V3>], closed: bool) -> (Vec<V3>, Vec<usize>) {
    let (r, c) = (rows.len(), rows[0].len());
    let mut idx = Vec::new();
    let cols = if closed { c } else { c - 1 };
    for i in 0..r - 1 {
        for j in 0..cols {
            let (a, bb) = (i * c + j, i * c + (j + 1) % c);
            idx.extend([a, bb, bb + c, a, bb + c, a + c]);
        }
    }
    (rows.iter().flatten().copied().collect(), idx)
}
