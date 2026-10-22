//! Ambient occlusion baked into a part's tone.w (docs/characters-format.md "Tone"): how much of the
//! sky each vertex does not see, 0 (open) to 1 (enclosed), at rest, from rays over the hemisphere
//! about its normal against the part itself and what lies round it (the body for a shell, the head
//! for hair). The shader darkens the colour by it, as the furniture's baked grey does
//! (tools/art/blender/gamekit/shade.py, which this follows: the same directions, twist, falloff and
//! rule for buried vertices). Deterministic: fixed directions, no randomness.

use nalgebra::{Rotation3, Unit};

use super::{Mesh, V3, v3};

/// Rays per vertex.
const SAMPLES: usize = 48;
/// How far a crease is looked for (m): the furniture's rule, a fifth of the model's largest side (a
/// young adult's height).
pub const REACH: f64 = 0.34;
/// Rays start this far off the surface (m), so they don't hit their own face.
const LIFT: f64 = 0.0015;
/// Steps it is rounded to: neighbours share a value, which packs tightly.
const LEVELS: f64 = 32.0;
/// Triangles per leaf of the tree.
const LEAF: usize = 4;

/// `n` directions over the hemisphere about +z, cosine-weighted (more towards the top), evenly
/// spread (a Fibonacci spiral).
fn directions(n: usize) -> Vec<V3> {
    let golden = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    (0..n)
        .map(|i| {
            let r = ((i as f64 + 0.5) / n as f64).sqrt();
            let a = golden * i as f64;
            v3(r * a.cos(), r * a.sin(), (1.0 - r * r).max(0.0).sqrt())
        })
        .collect()
}

struct Node {
    lo: V3,
    hi: V3,
    /// A leaf's triangles (`start..end` of `Bvh::order`), or a split's two children (`start`, `end`).
    start: usize,
    end: usize,
    leaf: bool,
}

/// A bounding-volume tree over triangles (split at the median along the longest side).
struct Bvh {
    tris: Vec<[V3; 3]>,
    order: Vec<usize>,
    nodes: Vec<Node>,
}

impl Bvh {
    fn new(meshes: &[&Mesh]) -> Self {
        let tris: Vec<[V3; 3]> = meshes.iter().flat_map(|m| m.idx.chunks(3).map(|t| [m.pos[t[0]], m.pos[t[1]], m.pos[t[2]]])).collect();
        let mut b = Bvh { order: (0..tris.len()).collect(), tris, nodes: Vec::new() };
        if !b.tris.is_empty() {
            b.build(0, b.tris.len());
        }
        b
    }

    fn bounds(&self, ids: &[usize]) -> (V3, V3) {
        let (mut lo, mut hi) = (V3::repeat(f64::INFINITY), V3::repeat(f64::NEG_INFINITY));
        for &t in ids {
            for p in &self.tris[t] {
                lo = lo.inf(p);
                hi = hi.sup(p);
            }
        }
        (lo, hi)
    }

    fn build(&mut self, start: usize, end: usize) -> usize {
        let (lo, hi) = self.bounds(&self.order[start..end]);
        let at = self.nodes.len();
        self.nodes.push(Node { lo, hi, start, end, leaf: true });
        if end - start <= LEAF {
            return at;
        }
        let axis = (hi - lo).imax();
        let tris = &self.tris;
        let c = |t: usize| tris[t][0][axis] + tris[t][1][axis] + tris[t][2][axis];
        self.order[start..end].sort_by(|&a, &b| c(a).total_cmp(&c(b)).then(a.cmp(&b)));
        let mid = (start + end) / 2;
        let l = self.build(start, mid);
        let r = self.build(mid, end);
        self.nodes[at] = Node { lo, hi, start: l, end: r, leaf: false };
        at
    }

    /// The nearest triangle the ray `o + t d` crosses for `t` in `(0, reach)`: its distance and
    /// whether it is the triangle's back (counter-clockwise from outside).
    fn cast(&self, o: &V3, d: &V3, reach: f64) -> Option<(f64, bool)> {
        if self.nodes.is_empty() {
            return None;
        }
        let inv = v3(1.0 / d.x, 1.0 / d.y, 1.0 / d.z);
        let mut best: Option<(f64, bool)> = None;
        let mut far = reach;
        let mut stack = vec![0usize];
        while let Some(k) = stack.pop() {
            let nd = &self.nodes[k];
            // Slab test against the box.
            let (mut t0, mut t1) = (0.0f64, far);
            for a in 0..3 {
                let (u, v) = ((nd.lo[a] - o[a]) * inv[a], (nd.hi[a] - o[a]) * inv[a]);
                t0 = t0.max(u.min(v));
                t1 = t1.min(u.max(v));
            }
            if t0 > t1 {
                continue;
            }
            if !nd.leaf {
                stack.push(nd.end);
                stack.push(nd.start);
                continue;
            }
            for &t in &self.order[nd.start..nd.end] {
                let [a, b, c] = &self.tris[t];
                // Möller-Trumbore, both sides.
                let (e1, e2) = (b - a, c - a);
                let p = d.cross(&e2);
                let det = e1.dot(&p);
                if det.abs() < 1e-14 {
                    continue;
                }
                let s = o - a;
                let u = s.dot(&p) / det;
                if !(0.0..=1.0).contains(&u) {
                    continue;
                }
                let q = s.cross(&e1);
                let v = d.dot(&q) / det;
                if v < 0.0 || u + v > 1.0 {
                    continue;
                }
                let dist = e2.dot(&q) / det;
                if dist > 1e-9 && dist < far {
                    far = dist;
                    best = Some((dist, e1.cross(&e2).dot(d) > 0.0));
                }
            }
        }
        best
    }
}

/// Per vertex of `mesh`: how enclosed it is at rest by itself and `around`, 0 (open) to 1, from
/// rays over the hemisphere about its normal, a hit counting less the farther it is (up to
/// `reach`, m). A vertex buried in another part (most rays first meet a back face) stays open:
/// shaded, it would darken the faces it shares.
pub fn occlusion(mesh: &Mesh, around: &[&Mesh], reach: f64) -> Vec<f64> {
    let mut all = vec![mesh];
    all.extend(around);
    let bvh = Bvh::new(&all);
    let dirs = directions(SAMPLES);
    let z = Unit::new_normalize(v3(0.0, 0.0, 1.0));
    (0..mesh.len())
        .map(|i| {
            let n = mesh.nor[i].try_normalize(1e-12).unwrap_or(*z);
            // A turn taking +z onto the normal, twisted by the vertex's index so neighbours don't band.
            let onto = Rotation3::rotation_between(&z, &n).unwrap_or_else(|| Rotation3::from_axis_angle(&V3::x_axis(), std::f64::consts::PI));
            let turn = onto * Rotation3::from_axis_angle(&z, i as f64 * 2.399963);
            let start = mesh.pos[i] + n * LIFT;
            let (mut blocked, mut inside) = (0.0, 0);
            for d in &dirs {
                match bvh.cast(&start, &(turn * d), reach) {
                    Some((_, true)) => inside += 1,
                    Some((dist, false)) => blocked += 1.0 - (dist / reach).powi(2),
                    None => {}
                }
            }
            if inside > SAMPLES / 2 { 0.0 } else { (blocked / SAMPLES as f64 * LEVELS).round() / LEVELS }
        })
        .collect()
}

/// Writes `occlusion` into `mesh`'s tone.w.
pub fn bake(mesh: &mut Mesh, around: &[&Mesh], reach: f64) {
    let occ = occlusion(mesh, around, reach);
    mesh.tone.resize(mesh.len(), [0.0; 4]);
    for (t, a) in mesh.tone.iter_mut().zip(occ) {
        t[3] = a;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2 m square in the xy plane at `z`, facing +z (or -z: `flip`).
    fn quad(z: f64, flip: bool) -> Mesh {
        let pos = vec![v3(-1.0, -1.0, z), v3(1.0, -1.0, z), v3(1.0, 1.0, z), v3(-1.0, 1.0, z)];
        let idx = if flip { vec![0, 2, 1, 0, 3, 2] } else { vec![0, 1, 2, 0, 2, 3] };
        let s = if flip { -1.0 } else { 1.0 };
        Mesh { nor: vec![v3(0.0, 0.0, s); 4], pos, idx, ..Mesh::default() }
    }

    #[test]
    fn open_covered_and_buried() {
        let floor = quad(0.0, false);
        // Nothing above: open.
        assert!(occlusion(&floor, &[], 0.5).iter().all(|&a| a == 0.0));
        // A ceiling facing down 5 cm above the middle: enclosed (its front faces the floor).
        let ceiling = Mesh { pos: vec![v3(-0.3, -0.3, 0.05), v3(0.3, -0.3, 0.05), v3(0.3, 0.3, 0.05), v3(-0.3, 0.3, 0.05)], ..quad(0.05, true) };
        let mid = Mesh { pos: vec![v3(0.0, 0.0, 0.0)], nor: vec![v3(0.0, 0.0, 1.0)], ..Mesh::default() };
        let a = occlusion(&mid, &[&ceiling], 0.5)[0];
        assert!(a > 0.8, "{a}");
        // The same seen from its back: buried, so open.
        let back = Mesh { idx: vec![0, 1, 2, 0, 2, 3], ..ceiling };
        assert_eq!(occlusion(&mid, &[&back], 0.5)[0], 0.0);
    }
}
