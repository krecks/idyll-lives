//! Mesh helpers for the character build: normals, open edges, scalar-field clipping, weights.
//! (meshops.py's welding, adjacency, Laplacian relaxation, geodesics and dense weights had no
//! callers and are not ported.)

use super::np::unique_rows;
use super::{Dense, V3};

/// Area-weighted vertex normals (counter-clockwise front faces).
pub fn smooth_normals(pos: &[V3], idx: &[usize]) -> Vec<V3> {
    let tris: Vec<&[usize]> = idx.chunks(3).collect();
    let fns: Vec<V3> = tris.iter().map(|t| (pos[t[1]] - pos[t[0]]).cross(&(pos[t[2]] - pos[t[0]]))).collect();
    let mut acc = vec![V3::zeros(); pos.len()];
    // np.add.at per corner, in that order.
    for k in 0..3 {
        for (t, f) in tris.iter().zip(&fns) {
            acc[t[k]] += f;
        }
    }
    acc.iter().map(|n| n / n.norm().max(1e-12)).collect()
}

/// Directed edges used by exactly one triangle (in that triangle's winding).
pub fn boundary_edges(idx: &[usize]) -> Vec<[usize; 2]> {
    let tris: Vec<&[usize]> = idx.chunks(3).collect();
    let mut e: Vec<[usize; 2]> = Vec::with_capacity(idx.len());
    for (a, b) in [(0, 1), (1, 2), (2, 0)] {
        e.extend(tris.iter().map(|t| [t[a], t[b]]));
    }
    let key: Vec<[usize; 2]> = e.iter().map(|&[a, b]| [a.min(b), a.max(b)]).collect();
    let (_, inv, cnt) = unique_rows(&key);
    e.iter().zip(&inv).filter(|&(_, &i)| cnt[i] == 1).map(|(&x, _)| x).collect()
}

/// Dense weights -> 4 joints and normalised weights: the heaviest first. Mirrored bones often weigh
/// exactly the same (a clavicle each side on the midline), and which of them makes the four shows,
/// so the order is numpy's `argsort(-dense)` itself (not a stable sort).
pub fn top4(dense: &Dense) -> ([usize; 4], [f64; 4]) {
    let neg: Vec<f64> = dense.iter().map(|v| -v).collect();
    let o = super::np::argsort_numpy(&neg);
    let j = [o[0], o[1], o[2], o[3]];
    let w = j.map(|i| dense[i]);
    let s = (w[0] + w[1] + w[2] + w[3]).max(1e-9);
    (j, w.map(|v| v / s))
}

/// Per new vertex, its source edge (a, b) and how far along it (t).
pub type Src = (Vec<usize>, Vec<usize>, Vec<f64>);

/// The region g > 0 of a triangle mesh, cut exactly along the iso-line g = 0 (linear
/// interpolation), so cut edges are smooth however coarse the region boundaries are. New vertices
/// on a shared edge are shared, so the result stays welded. Returns the faces and, per new vertex,
/// its source edge (a, b, t): the vertex is a + (b - a) t. (The Python also carried UVs and lerped
/// attributes; its one caller used only these.)
pub fn clip_by_field(idx: &[usize], g: &[f64]) -> (Vec<usize>, Src) {
    let inside: Vec<bool> = g.iter().map(|&v| v > 0.0).collect();
    let mut keep: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    let (mut out_a, mut out_b, mut out_t) = (Vec::new(), Vec::new(), Vec::new());
    let mut vert = |a: usize, b: Option<usize>| -> usize {
        let (k, a, b, t) = match b {
            None => ((a, a), a, a, 0.0),
            Some(b) => {
                let k = (a.min(b), a.max(b));
                let (a, b) = if k.0 != a { (b, a) } else { (a, b) };
                (k, a, b, g[a] / (g[a] - g[b]))
            }
        };
        *keep.entry(k).or_insert_with(|| {
            out_a.push(a);
            out_b.push(b);
            out_t.push(t);
            out_a.len() - 1
        })
    };
    let mut faces = Vec::new();
    for t in idx.chunks(3) {
        let s = [inside[t[0]], inside[t[1]], inside[t[2]]];
        let c = s.iter().filter(|&&v| v).count();
        if c == 0 {
            continue;
        }
        if c == 3 {
            faces.extend([vert(t[0], None), vert(t[1], None), vert(t[2], None)]);
            continue;
        }
        // Rotate so the pattern starts at a fixed corner (winding kept).
        if c == 1 {
            let r = s.iter().position(|&v| v).unwrap();
            let (a, b, cc) = (t[r], t[(r + 1) % 3], t[(r + 2) % 3]);
            faces.extend([vert(a, None), vert(a, Some(b)), vert(a, Some(cc))]);
        } else {
            let r = s.iter().position(|&v| !v).unwrap();
            let (cc, a, b) = (t[r], t[(r + 1) % 3], t[(r + 2) % 3]);
            let f1 = [vert(a, None), vert(b, None), vert(b, Some(cc))];
            let f2 = [vert(a, None), vert(b, Some(cc)), vert(cc, Some(a))];
            faces.extend(f1);
            faces.extend(f2);
        }
    }
    (faces, (out_a, out_b, out_t))
}
