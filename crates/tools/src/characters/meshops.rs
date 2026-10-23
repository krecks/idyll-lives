//! Mesh helpers for the character build: normals, open edges, scalar-field clipping, weights, nearest
//! points (`OnMesh`). (meshops.py's welding, adjacency, Laplacian relaxation, geodesics and dense weights
//! had no callers and are not ported.)

use super::kdtree::KdTree;
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

/// The barycentric weights of the point of triangle abc nearest p (Ericson, Real-Time Collision
/// Detection 5.1.5).
fn nearest_weights(p: V3, a: V3, b: V3, c: V3) -> [f64; 3] {
    let (ab, ac, ap) = (b - a, c - a, p - a);
    let (d1, d2) = (ab.dot(&ap), ac.dot(&ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let bp = p - b;
    let (d3, d4) = (ab.dot(&bp), ac.dot(&bp));
    if d3 >= 0.0 && d4 <= d3 {
        return [0.0, 1.0, 0.0];
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [1.0 - v, v, 0.0];
    }
    let cp = p - c;
    let (d5, d6) = (ab.dot(&cp), ac.dot(&cp));
    if d6 >= 0.0 && d5 <= d6 {
        return [0.0, 0.0, 1.0];
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [1.0 - w, 0.0, w];
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [0.0, 1.0 - w, w];
    }
    let den = 1.0 / (va + vb + vc);
    let (v, w) = (vb * den, vc * den);
    [1.0 - v - w, v, w]
}

/// The nearest point of a triangle mesh to a point, as its triangle's corners and their weights: to
/// carry what the mesh has at its vertices (normals, tone) over to what lies on it.
pub struct OnMesh {
    pos: Vec<V3>,
    tris: Vec<[usize; 3]>,
    centres: KdTree,
}

impl OnMesh {
    /// The triangles of `idx` whose corners are all among the first `n` of `pos`.
    pub fn new(pos: &[V3], idx: &[usize], n: usize) -> Self {
        let tris: Vec<[usize; 3]> = idx.chunks(3).filter(|t| t.iter().all(|&i| i < n)).map(|t| [t[0], t[1], t[2]]).collect();
        let centres: Vec<V3> = tris.iter().map(|t| (pos[t[0]] + pos[t[1]] + pos[t[2]]) / 3.0).collect();
        OnMesh { pos: pos.to_vec(), tris, centres: KdTree::new(&centres) }
    }

    /// The nearest point to p (of the triangles with the 16 nearest centres): its corners and weights.
    pub fn at(&self, p: V3) -> ([usize; 3], [f64; 3]) {
        let mut best = (f64::INFINITY, [0; 3], [0.0; 3]);
        for (_, t) in self.centres.query(&p, 16) {
            let [a, b, c] = self.tris[t];
            let w = nearest_weights(p, self.pos[a], self.pos[b], self.pos[c]);
            let d = (self.pos[a] * w[0] + self.pos[b] * w[1] + self.pos[c] * w[2] - p).norm_squared();
            if d < best.0 {
                best = (d, [a, b, c], w);
            }
        }
        (best.1, best.2)
    }
}
