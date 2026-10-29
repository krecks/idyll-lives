//! A k-nearest KD-tree over 3-D points (for scipy's `cKDTree(points).query(p, k)`): the k nearest
//! by Euclidean distance, nearest first, ties broken by the lower index.

use super::V3;

const LEAF: usize = 8;

enum Node {
    Leaf {
        start: usize,
        end: usize,
    },
    Split {
        axis: usize,
        at: f64,
        lo: usize,
        hi: usize,
    },
}

pub struct KdTree {
    pts: Vec<V3>,
    /// Point ids, grouped by leaf.
    ids: Vec<usize>,
    nodes: Vec<Node>,
}

/// Squared distance, summed x, y, z in order.
fn dist2(a: &V3, b: &V3) -> f64 {
    let (dx, dy, dz) = (a.x - b.x, a.y - b.y, a.z - b.z);
    dx * dx + dy * dy + dz * dz
}

impl KdTree {
    pub fn new(pts: &[V3]) -> Self {
        let mut t = KdTree {
            pts: pts.to_vec(),
            ids: (0..pts.len()).collect(),
            nodes: Vec::new(),
        };
        if !pts.is_empty() {
            t.build(0, pts.len());
        }
        t
    }

    pub fn len(&self) -> usize {
        self.pts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pts.is_empty()
    }

    fn build(&mut self, start: usize, end: usize) -> usize {
        let me = self.nodes.len();
        if end - start <= LEAF {
            self.nodes.push(Node::Leaf { start, end });
            return me;
        }
        // Split the widest extent at the median.
        let mut lo = V3::repeat(f64::INFINITY);
        let mut hi = V3::repeat(f64::NEG_INFINITY);
        for &i in &self.ids[start..end] {
            lo = lo.inf(&self.pts[i]);
            hi = hi.sup(&self.pts[i]);
        }
        let ext = hi - lo;
        let axis = if ext.x >= ext.y && ext.x >= ext.z {
            0
        } else if ext.y >= ext.z {
            1
        } else {
            2
        };
        let mid = (start + end) / 2;
        let pts = &self.pts;
        self.ids[start..end].select_nth_unstable_by(mid - start, |&a, &b| {
            pts[a][axis].total_cmp(&pts[b][axis]).then(a.cmp(&b))
        });
        let at = self.pts[self.ids[mid]][axis];
        self.nodes.push(Node::Leaf { start: 0, end: 0 });
        let l = self.build(start, mid);
        let h = self.build(mid, end);
        self.nodes[me] = Node::Split {
            axis,
            at,
            lo: l,
            hi: h,
        };
        me
    }

    /// The k nearest points to `p` as (distance, index), nearest first (fewer if the tree is smaller).
    pub fn query(&self, p: &V3, k: usize) -> Vec<(f64, usize)> {
        let mut best: Vec<(f64, usize)> = Vec::with_capacity(k + 1);
        if !self.pts.is_empty() && k > 0 {
            self.search(0, p, k, &mut best);
        }
        best.iter().map(|&(d2, i)| (d2.sqrt(), i)).collect()
    }

    /// The k nearest with ties shared: points within `eps` of each other in distance count as tied,
    /// and a tied group that straddles the k-th place shares the places left evenly. Returns
    /// (distance, index, share); the shares sum to k. Mirror-image points (exactly equidistant by
    /// design) are then never split by rounding noise.
    pub fn query_shared(&self, p: &V3, k: usize, eps: f64) -> Vec<(f64, usize, f64)> {
        let nb = self.query(p, k + 8);
        let mut out = Vec::new();
        let (mut i, mut taken) = (0, 0);
        while i < nb.len() && taken < k {
            let mut j = i + 1;
            while j < nb.len() && nb[j].0 - nb[j - 1].0 <= eps {
                j += 1;
            }
            let left = k - taken;
            let share = if j - i <= left {
                1.0
            } else {
                left as f64 / (j - i) as f64
            };
            out.extend(nb[i..j].iter().map(|&(d, idx)| (d, idx, share)));
            taken += (j - i).min(left);
            i = j;
        }
        out
    }

    /// The nearest point's index.
    pub fn nearest(&self, p: &V3) -> usize {
        self.query(p, 1)[0].1
    }

    fn search(&self, node: usize, p: &V3, k: usize, best: &mut Vec<(f64, usize)>) {
        match self.nodes[node] {
            Node::Leaf { start, end } => {
                for &i in &self.ids[start..end] {
                    let d = dist2(p, &self.pts[i]);
                    if best.len() == k {
                        let w = best[k - 1];
                        if d > w.0 || (d == w.0 && i > w.1) {
                            continue;
                        }
                    }
                    let at = best.partition_point(|&(bd, bi)| bd < d || (bd == d && bi < i));
                    best.insert(at, (d, i));
                    best.truncate(k);
                }
            }
            Node::Split { axis, at, lo, hi } => {
                let diff = p[axis] - at;
                let (near, far) = if diff < 0.0 { (lo, hi) } else { (hi, lo) };
                self.search(near, p, k, best);
                // Points on the far side are at least |diff| away (equal distance can still win a tie).
                if best.len() < k || diff * diff <= best[k - 1].0 {
                    self.search(far, p, k, best);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_ties() {
        // From the origin: one point at 1, a mirror pair at 2 (one a hair nearer), one at 3.
        let pts = [
            V3::new(1.0, 0.0, 0.0),
            V3::new(0.0, 2.0, 0.0),
            V3::new(0.0, -2.0 + 1e-15, 0.0),
            V3::new(0.0, 0.0, 3.0),
        ];
        let t = KdTree::new(&pts);
        let got = t.query_shared(&V3::zeros(), 2, 1e-9);
        assert_eq!(
            got.iter().map(|e| (e.1, e.2)).collect::<Vec<_>>(),
            [(0, 1.0), (2, 0.5), (1, 0.5)]
        );
        let got = t.query_shared(&V3::zeros(), 3, 1e-9);
        assert_eq!(
            got.iter().map(|e| (e.1, e.2)).collect::<Vec<_>>(),
            [(0, 1.0), (2, 1.0), (1, 1.0)]
        );
    }

    #[test]
    fn matches_brute_force() {
        // A lattice (many exact ties) plus scattered points.
        let mut pts = Vec::new();
        for i in 0..6 {
            for j in 0..6 {
                for k in 0..4 {
                    pts.push(V3::new(i as f64 * 0.1, j as f64 * 0.1, k as f64 * 0.1));
                }
            }
        }
        let mut s = 7u64;
        let mut rnd = || {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (s >> 11) as f64 / (1u64 << 53) as f64
        };
        for _ in 0..300 {
            pts.push(V3::new(rnd() * 0.6 - 0.05, rnd() * 0.6, rnd() * 0.4));
        }
        let tree = KdTree::new(&pts);
        let mut queries: Vec<V3> = (0..200)
            .map(|_| V3::new(rnd() * 0.8 - 0.1, rnd() * 0.7, rnd() * 0.5))
            .collect();
        queries.extend(pts.iter().take(40).cloned());
        queries.push(V3::new(0.25, 0.25, 0.15));
        for q in &queries {
            for k in [1, 2, 6, 16] {
                let mut all: Vec<(f64, usize)> = pts
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (dist2(q, p), i))
                    .collect();
                all.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                let want: Vec<(f64, usize)> =
                    all[..k].iter().map(|&(d, i)| (d.sqrt(), i)).collect();
                assert_eq!(tree.query(q, k), want);
            }
        }
    }
}
