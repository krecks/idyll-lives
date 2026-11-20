//! A solid: the vertices welded, and the faces that close it.

use super::*;

pub(crate) fn edge_key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

/// The ways to the play camera from what it looks at: every 30° or so round, at four heights between its lowest
/// (13° over the ground) and its steepest (73°). (From whole numbers, roots and divisions only: the same on every
/// machine.)
pub(crate) fn views() -> &'static [V3] {
    static VIEWS: std::sync::OnceLock<Vec<V3>> = std::sync::OnceLock::new();
    VIEWS.get_or_init(|| {
        let mut round: Vec<(f64, f64)> = Vec::new();
        for (x, z) in [(1.0, 0.0), (2.0, 1.0), (1.0, 2.0)] {
            // (Each a quarter turn on, four times.)
            round.extend([(x, z), (-z, x), (-x, -z), (z, -x)]);
        }
        let mut views = Vec::new();
        for rise in [0.25, 0.75, 1.5, 3.0] {
            for (x, z) in &round {
                let flat = (x * x + z * z).sqrt();
                views.push(unit([*x, flat * rise, *z]));
            }
        }
        views
    })
}

/// For each of the `views`, the triangle that hid the last point looked at from there.
pub(crate) type Hid = [Option<usize>; 48];

/// A box in a tree of boxes over triangles: a leaf's triangles `a..b`, or its two children `a` and `b`.
struct Node {
    lo: V3,
    hi: V3,
    a: u32,
    b: u32,
    leaf: bool,
}

/// The opaque triangles of a shape, to ask what a line through it crosses.
pub struct Solid {
    nodes: Vec<Node>,
    /// Each triangle: a corner, its two edges from there, and its normal.
    tris: Vec<[V3; 4]>,
}

impl Solid {
    /// The triangles of the meshes that hide what is behind them (`Mesh::opaque`).
    pub fn new(meshes: &[Mesh]) -> Solid {
        let mut tris = Vec::new();
        for mesh in meshes.iter().filter(|m| m.opaque) {
            for t in &mesh.triangles {
                let corner = |k: usize| mesh.positions.get(t[k] as usize).copied();
                let (Some(a), Some(b), Some(c)) = (corner(0), corner(1), corner(2)) else {
                    continue;
                };
                let (e1, e2) = (sub(b, a), sub(c, a));
                let n = cross(e1, e2);
                if length(n) >= 1e-12 {
                    tris.push([a, e1, e2, unit(n)]);
                }
            }
        }
        let mut solid = Solid {
            nodes: Vec::new(),
            tris,
        };
        if !solid.tris.is_empty() {
            solid.build(0, solid.tris.len());
        }
        solid
    }

    fn build(&mut self, from: usize, to: usize) -> u32 {
        let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
        let (mut mid_lo, mut mid_hi) = (lo, hi);
        for [a, e1, e2, _] in &self.tris[from..to] {
            let middle = add(*a, mul(add(*e1, *e2), 1.0 / 3.0));
            for k in 0..3 {
                for p in [*a, add(*a, *e1), add(*a, *e2)] {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
                mid_lo[k] = mid_lo[k].min(middle[k]);
                mid_hi[k] = mid_hi[k].max(middle[k]);
            }
        }
        let at = self.nodes.len();
        self.nodes.push(Node {
            lo,
            hi,
            a: from as u32,
            b: to as u32,
            leaf: true,
        });
        if to - from > 4 {
            // Halved across the longest side of its triangles' middles.
            let axis = (0..3)
                .max_by(|&i, &j| (mid_hi[i] - mid_lo[i]).total_cmp(&(mid_hi[j] - mid_lo[j])))
                .unwrap_or(0);
            let middle = |t: &[V3; 4]| t[0][axis] + (t[1][axis] + t[2][axis]) / 3.0;
            let half = (to - from) / 2;
            self.tris[from..to]
                .select_nth_unstable_by(half, |a, b| middle(a).total_cmp(&middle(b)));
            let a = self.build(from, from + half);
            let b = self.build(from + half, to);
            self.nodes[at] = Node {
                lo,
                hi,
                a,
                b,
                leaf: false,
            };
        }
        at as u32
    }

    /// Every triangle the line `from + t × way` crosses with t in `t0..=t1`, until `hit(t, whether it leaves a
    /// solid there, going on along way)` says to stop.
    fn crossed(
        &self,
        from: V3,
        way: V3,
        t0: f64,
        t1: f64,
        hit: &mut impl FnMut(f64, bool) -> bool,
    ) {
        self.crossed_at(from, way, t0, t1, &mut |t, leaves, _| hit(t, leaves))
    }

    /// `crossed`, with which triangle each is.
    fn crossed_at(
        &self,
        from: V3,
        way: V3,
        t0: f64,
        t1: f64,
        hit: &mut impl FnMut(f64, bool, usize) -> bool,
    ) {
        const PAD: f64 = 1e-7;
        if self.nodes.is_empty() {
            return;
        }
        // (Along an axis the line doesn't move along, it is in a box's span or misses the box.)
        let level = way.map(|w| w.abs() < 1e-12);
        let per = way.map(|w| 1.0 / w);
        // (A tree of halves is never deeper than the bits of its count.)
        let mut todo = [0u32; 64];
        let mut left = 1;
        'nodes: while left > 0 {
            left -= 1;
            let node = &self.nodes[todo[left] as usize];
            let (mut near, mut far) = (t0, t1);
            for k in 0..3 {
                if level[k] {
                    if from[k] < node.lo[k] - PAD || from[k] > node.hi[k] + PAD {
                        continue 'nodes;
                    }
                } else {
                    let a = (node.lo[k] - PAD - from[k]) * per[k];
                    let b = (node.hi[k] + PAD - from[k]) * per[k];
                    near = near.max(a.min(b));
                    far = far.min(a.max(b));
                    if near > far {
                        continue 'nodes;
                    }
                }
            }
            if !node.leaf {
                if left + 2 <= todo.len() {
                    todo[left] = node.a;
                    todo[left + 1] = node.b;
                    left += 2;
                }
                continue;
            }
            for i in node.a as usize..node.b as usize {
                if let Some((t, leaves)) = self.meets(i, from, way)
                    && (t0..=t1).contains(&t)
                    && hit(t, leaves, i)
                {
                    return;
                }
            }
        }
    }

    /// Where the line `from + t × way` crosses triangle `i`, and whether it leaves a solid there (Möller-Trumbore,
    /// with a hair of slack at the triangle's sides: a line down a seam hits).
    fn meets(&self, i: usize, from: V3, way: V3) -> Option<(f64, bool)> {
        let [a, e1, e2, n] = self.tris[i];
        let h = cross(way, e2);
        let det = dot(e1, h);
        if det.abs() < 1e-14 {
            return None;
        }
        let s = sub(from, a);
        let u = dot(s, h) / det;
        if !(-1e-9..=1.0 + 1e-9).contains(&u) {
            return None;
        }
        let q = cross(s, e1);
        let v = dot(way, q) / det;
        if v < -1e-9 || u + v > 1.0 + 1e-9 {
            return None;
        }
        Some((dot(e2, q) / det, dot(n, way) > 0.0))
    }

    /// Whether `p` is inside something opaque: along the line through it by `way`, more faces leave a solid than
    /// enter one, both ways from it (so one solid standing in another still counts, and a sheet seen from behind
    /// on one side only doesn't).
    pub fn inside(&self, p: V3, way: V3) -> bool {
        let mut hits: Vec<(i64, bool)> = Vec::new();
        self.crossed(
            p,
            way,
            f64::NEG_INFINITY,
            f64::INFINITY,
            &mut |t, leaves| {
                // (To the micrometre: a line down a seam hits both its triangles at one place.)
                hits.push(((t * 1e6).round() as i64, leaves));
                false
            },
        );
        hits.sort_unstable();
        hits.dedup();
        let (mut on, mut back) = (0, 0);
        for &(t, leaves) in &hits {
            match t {
                1.. => on += if leaves { 1 } else { -1 },
                ..=-1 => back += if leaves { -1 } else { 1 },
                0 => {}
            }
        }
        on >= 1 && back >= 1
    }

    /// Whether a face is within `reach` of `from` along `way` (of length 1), but not just at it.
    pub fn near(&self, from: V3, way: V3, reach: f64) -> bool {
        let mut any = false;
        self.crossed(from, way, 1e-5, reach, &mut |_, _| {
            any = true;
            true
        });
        any
    }

    /// The triangle that hides `p` from a camera far off along `way`, if one does: a face between them that looks
    /// at the camera (one seen from behind isn't drawn). `last`: the one that hid the point looked at before
    /// along this way, tried first, as it mostly hides its neighbour too.
    pub fn hides(&self, p: V3, way: V3, last: Option<usize>) -> Option<usize> {
        let hides = |hit: Option<(f64, bool)>| hit.is_some_and(|(t, leaves)| t >= 1e-5 && leaves);
        if let Some(i) = last.filter(|&i| hides(self.meets(i, p, way))) {
            return Some(i);
        }
        let mut found = None;
        self.crossed_at(p, way, 1e-5, f64::INFINITY, &mut |_, leaves, i| {
            if leaves {
                found = Some(i);
            }
            leaves
        });
        found
    }

    /// Whether an edge's point `p` shows, `out` the way between its faces: pushed `OUT` that way it is seen from
    /// two or more of the play camera's `views` on its side (of those `from` takes), and neither of its faces (normal, the way into it from
    /// the edge) is covered: `OUT` in from the edge another face lies on it or within `OUT` in front of it, or it
    /// goes into something before it is `WIDTH` wide.
    pub(crate) fn shows(
        &self,
        p: V3,
        out: V3,
        faces: [(V3, V3); 2],
        last: &mut Hid,
        from: &(dyn Fn(V3) -> bool + Sync),
    ) -> bool {
        if faces
            .iter()
            .any(|&(n, into)| self.near(add(p, mul(into, OUT)), n, OUT))
        {
            return false;
        }
        // (The views nearest the way it faces first: they are the likeliest to be open.)
        let off = add(p, mul(out, OUT));
        let mut ways: Vec<(f64, usize)> = views()
            .iter()
            .enumerate()
            .map(|(i, way)| (dot(*way, out), i))
            .filter(|(facing, i)| *facing > 0.0 && from(views()[*i]))
            .collect();
        ways.sort_by(|a, b| b.0.total_cmp(&a.0));
        let seen = ways
            .iter()
            .filter(|&&(_, i)| {
                last[i] = self.hides(off, views()[i], last[i]);
                last[i].is_none()
            })
            .take(2)
            .count();
        seen == 2
            && !faces.iter().any(|&(n, into)| {
                let lift = mul(n, OUT / 3.0);
                self.inside(add(p, add(mul(into, OUT), lift)), n)
                    || self.inside(add(p, add(mul(into, WIDTH - OUT / 3.0), lift)), n)
            })
    }
}
