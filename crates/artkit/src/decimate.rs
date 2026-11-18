//! A mesh brought down to a share of its triangles by collapsing its edges, the one that moves its surface
//! least first (Garland and Heckbert's quadric error): what Blender's DECIMATE modifier did for the far versions
//! (`far::decimated`). A build gives the same mesh on every machine and every run: the errors are sums, products,
//! quotients and square roots in double precision, in a fixed order, the cheapest collapse is taken from an ordered
//! heap (ties by the edge's vertices), and each point is rounded to the mesh's single precision as it is placed.
//!
//! What a collapse may not do:
//! - move or take away a locked vertex (the feet and the outermost faces of a far version);
//! - take a vertex off an open edge or off the line between two materials, but along it; or one where such
//!   lines meet, at all: so no material is smeared over another, and an open sheet keeps its outline;
//! - turn a triangle round, or flatten one to nothing;
//! - pinch the surface (two faces left back to back, an edge of more faces than it had, a face left hanging by
//!   a point): a closed mesh stays closed;
//! - leave the mesh's box, or take a vertex out of reach (`REACH`) of the faces it stood for: a post is not
//!   squashed, a rod not shortened, a panel's corner not cut off, whatever the count asks.

use crate::math::V3;
use crate::mesh::{Mesh, edge, normal_of};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

type P = [f64; 3];

/// How much heavier than a face an open edge or a line between two materials weighs: a vertex leaving it costs as
/// much as leaving this many faces by the same distance.
const BORDER: f64 = 100.0;
/// What an edge's length adds to its cost (the length squared, times this): of two collapses that move the
/// surface alike (on a flat face: not at all), the shorter edge goes first.
const LENGTH: f64 = 1e-6;
/// How strongly a collapse's point is drawn to the middle of its edge, as a share of the weight of its planes:
/// what places it where they leave it free (anywhere on a flat face, along a straight crease).
const PULL: f64 = 1e-3;
/// A triangle may not be turned by a collapse so far that its normals' cosine is under this (about 78 degrees).
const TURN: f64 = 0.2;
/// A triangle may not be left with its sides' cross product under this (m², twice its area), nor flatter than
/// this (its height over its longest side), unless it was worse.
const AREA: f64 = 1e-12;
/// How far (m) a collapse may take a vertex from the planes it is held to (its faces' as they were at first,
/// those across its open edges and lines between materials, and those of every vertex merged into it): a part
/// thicker than this is not squashed, a sheet's corner not cut off, a rod not shortened. The first is tried; if
/// the count cannot be met within it, the second, which is what the comparison lets two far versions differ by
/// (`cargo check-glb --against`, 60 mm); and if not within that either, the mesh is left as the first leaves
/// it: more would spoil its posts and panels for a few triangles.
pub const REACH: [f64; 2] = [0.03, 0.06];
const FLAT: f64 = 1e-5;

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: P, b: P) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: P, b: P) -> P {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// A unit vector along `a`; None for one of no length to speak of.
fn unit(a: P) -> Option<P> {
    let len = dot(a, a).sqrt();
    (len > 1e-18).then(|| a.map(|x| x / len))
}

/// A point as the mesh keeps it: single precision.
fn single(p: P) -> P {
    p.map(|x| x as f32 as f64)
}

/// A sum of weighted squared distances to planes, as a function of the point: p·A·p + 2 b·p + c, with A
/// symmetric (xx, xy, xz, yy, yz, zz).
#[derive(Clone, Copy, Default)]
struct Quadric {
    a: [f64; 6],
    b: P,
    c: f64,
}

impl Quadric {
    /// `weight` times the squared distance to the plane through `on` across the unit vector `n`.
    fn plane(n: P, on: P, weight: f64) -> Quadric {
        let d = -dot(n, on);
        Quadric {
            a: [
                n[0] * n[0],
                n[0] * n[1],
                n[0] * n[2],
                n[1] * n[1],
                n[1] * n[2],
                n[2] * n[2],
            ]
            .map(|x| x * weight),
            b: n.map(|x| x * d * weight),
            c: d * d * weight,
        }
    }

    fn add(&mut self, o: &Quadric) {
        for (x, y) in self.a.iter_mut().zip(o.a) {
            *x += y;
        }
        for (x, y) in self.b.iter_mut().zip(o.b) {
            *x += y;
        }
        self.c += o.c;
    }

    fn sum(mut self, o: &Quadric) -> Quadric {
        self.add(o);
        self
    }

    /// The sum at `p` (never under zero, whatever the rounding).
    fn at(&self, p: P) -> f64 {
        let a = &self.a;
        let ap = [
            a[0] * p[0] + a[1] * p[1] + a[2] * p[2],
            a[1] * p[0] + a[3] * p[1] + a[4] * p[2],
            a[2] * p[0] + a[4] * p[1] + a[5] * p[2],
        ];
        (dot(ap, p) + 2.0 * dot(self.b, p) + self.c).max(0.0)
    }

    /// The point where the sum is lowest, drawn to `near` by `PULL`: where the planes fix it, there (a corner,
    /// a point of a curved surface); where they leave it free, the nearest such point to `near`.
    fn lowest(&self, near: P) -> P {
        let a = &self.a;
        let pull = PULL * (a[0] + a[3] + a[5]) / 3.0;
        if pull.partial_cmp(&0.0) != Some(Ordering::Greater) {
            return near;
        }
        // (A + pull) p = pull near - b, by its cofactors: A is a sum of squares, so with the pull it has an inverse
        let m = [
            [a[0] + pull, a[1], a[2]],
            [a[1], a[3] + pull, a[4]],
            [a[2], a[4], a[5] + pull],
        ];
        let r: P = std::array::from_fn(|k| pull * near[k] - self.b[k]);
        let cof = [cross(m[1], m[2]), cross(m[2], m[0]), cross(m[0], m[1])];
        let det = dot(m[0], cof[0]);
        if det.partial_cmp(&0.0) != Some(Ordering::Greater) {
            return near;
        }
        std::array::from_fn(|k| (cof[0][k] * r[0] + cof[1][k] * r[1] + cof[2][k] * r[2]) / det)
    }
}

#[derive(Clone, Copy)]
struct Tri {
    v: [u32; 3],
    mat: u16,
    smooth: bool,
    alive: bool,
    /// The faces it is shaded as one surface with: those it reached at first across edges that were not sharp.
    group: u32,
}

impl Tri {
    fn has(&self, x: u32) -> bool {
        self.v.contains(&x)
    }
}

/// How a vertex lies among the open edges and the lines between materials: on none, along one, or where they
/// meet or end.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lie {
    Inside,
    Along,
    Corner,
}

/// A collapse: the vertex `gone` merged into `kept`, which is then at `to`.
#[derive(Clone, Copy)]
struct Collapse {
    cost: f64,
    gone: u32,
    kept: u32,
    to: P,
    /// What its two vertices' surroundings were counted at when it was costed: stale once either has changed.
    stamps: (u32, u32),
}

impl Collapse {
    fn key(&self) -> (u32, u32) {
        edge(self.gone, self.kept)
    }
}

// (the heap gives its greatest first: the cheapest collapse, of two as cheap the one of the lower vertices)
impl Ord for Collapse {
    fn cmp(&self, o: &Collapse) -> Ordering {
        o.cost
            .total_cmp(&self.cost)
            .then_with(|| o.key().cmp(&self.key()))
            .then_with(|| o.stamps.cmp(&self.stamps))
    }
}
impl PartialOrd for Collapse {
    fn partial_cmp(&self, o: &Collapse) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl PartialEq for Collapse {
    fn eq(&self, o: &Collapse) -> bool {
        self.cmp(o) == Ordering::Equal
    }
}
impl Eq for Collapse {}

#[derive(Clone)]
struct Work {
    at: Vec<P>,
    locked: Vec<bool>,
    quadric: Vec<Quadric>,
    tris: Vec<Tri>,
    /// A UV a corner, where the mesh has them.
    uvs: Option<Vec<[[f32; 2]; 3]>>,
    /// Each vertex's live triangles, in the order they came to it.
    around: Vec<Vec<u32>>,
    stamp: Vec<u32>,
    /// What each vertex taken away was merged into.
    into: Vec<u32>,
    /// Each triangle's normal before any collapse (twice its area long).
    was: Vec<P>,
    /// The planes the vertices are held to: each face's as it was before any collapse, and the one across each
    /// open edge or line between materials, as a unit normal and an offset.
    planes: Vec<(P, f64)>,
    /// Each vertex's planes: those of the faces and edges it was a corner of, and of every vertex merged into it.
    held: Vec<Vec<u32>>,
    /// How far a collapse may take a vertex from its planes (`REACH`).
    reach: f64,
    live: usize,
    lo: P,
    hi: P,
}

impl Work {
    fn corners(&self, t: u32) -> [P; 3] {
        self.tris[t as usize].v.map(|x| self.at[x as usize])
    }

    /// The vertices joined to `x` by an edge, in order.
    fn ring(&self, x: u32) -> Vec<u32> {
        let mut out: Vec<u32> = self.around[x as usize]
            .iter()
            .flat_map(|&t| self.tris[t as usize].v)
            .filter(|&y| y != x)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// The triangles on the edge from `a` to `b`.
    fn on_edge(&self, a: u32, b: u32) -> impl Iterator<Item = u32> + '_ {
        self.around[a as usize]
            .iter()
            .copied()
            .filter(move |&t| self.tris[t as usize].has(b))
    }

    /// Whether the edge is not one inside a surface of one material: open, between two materials, of three
    /// faces or more, or of two that run it the same way (one is turned).
    fn is_border(&self, a: u32, b: u32) -> bool {
        let mut seen: Option<(u16, bool)> = None;
        let mut n = 0;
        for t in self.on_edge(a, b) {
            let tri = &self.tris[t as usize];
            let i = tri.v.iter().position(|&x| x == a).unwrap_or(0);
            let forward = tri.v[(i + 1) % 3] == b;
            n += 1;
            match seen {
                None => seen = Some((tri.mat, forward)),
                Some((mat, way)) if mat != tri.mat || way == forward => return true,
                Some(_) => {}
            }
        }
        n != 2
    }

    fn lie(&self, x: u32) -> Lie {
        let borders = self
            .ring(x)
            .into_iter()
            .filter(|&y| self.is_border(x, y))
            .count();
        match borders {
            0 => Lie::Inside,
            2 => Lie::Along,
            _ => Lie::Corner,
        }
    }

    /// Whether merging `u` and `v` leaves the surface a surface: the faces on their edge go, and nothing else
    /// closes up (a vertex joined to both is a corner of one of those faces), none of those faces' corners is left
    /// without a face on its edge to the merged vertex, and no two faces are left on the same three vertices.
    fn keeps_surface(&self, u: u32, v: u32) -> bool {
        let thirds: Vec<u32> = self
            .on_edge(u, v)
            .filter_map(|t| {
                self.tris[t as usize]
                    .v
                    .into_iter()
                    .find(|&x| x != u && x != v)
            })
            .collect();
        let ring_v = self.ring(v);
        for w in self.ring(u) {
            if w != v && ring_v.binary_search(&w).is_ok() && !thirds.contains(&w) {
                return false;
            }
        }
        for &w in &thirds {
            let left = self
                .on_edge(u, w)
                .chain(self.on_edge(v, w))
                .filter(|&t| !(self.tris[t as usize].has(u) && self.tris[t as usize].has(v)))
                .count();
            if left == 0 {
                return false;
            }
        }
        let sorted = |mut c: [u32; 3]| {
            c.sort_unstable();
            c
        };
        let of_v: Vec<[u32; 3]> = self.around[v as usize]
            .iter()
            .map(|&t| self.tris[t as usize])
            .filter(|t| !t.has(u))
            .map(|t| sorted(t.v))
            .collect();
        let mut of_u: Vec<[u32; 3]> = Vec::new();
        for &t in &self.around[u as usize] {
            let tri = self.tris[t as usize];
            if tri.has(v) {
                continue;
            }
            let moved = sorted(tri.v.map(|x| if x == u { v } else { x }));
            if of_v.contains(&moved) || of_u.contains(&moved) {
                return false;
            }
            of_u.push(moved);
        }
        true
    }

    /// Whether the triangles round `moved` that stay (those without `other`) keep their shape with it at `to`:
    /// none turned round, none flattened to nothing.
    fn keeps_shape(&self, moved: u32, other: u32, to: P) -> bool {
        for &t in &self.around[moved as usize] {
            let tri = &self.tris[t as usize];
            let old = self.corners(t);
            if tri.has(other) {
                continue;
            }
            let new: [P; 3] = std::array::from_fn(|k| if tri.v[k] == moved { to } else { old[k] });
            let measure = |p: &[P; 3]| {
                let c = cross(sub(p[1], p[0]), sub(p[2], p[0]));
                let longest = (0..3)
                    .map(|k| {
                        let d = sub(p[(k + 1) % 3], p[k]);
                        dot(d, d)
                    })
                    .fold(0.0, f64::max);
                let size = dot(c, c).sqrt();
                (c, size, if longest > 0.0 { size / longest } else { 0.0 })
            };
            let (co, size_o, flat_o) = measure(&old);
            let (cn, size_n, flat_n) = measure(&new);
            if size_n < AREA.min(size_o) || flat_n < FLAT.min(flat_o) {
                return false;
            }
            if size_o >= AREA && dot(co, cn) < TURN * size_o * size_n {
                return false;
            }
            // (nor, collapse by collapse, round from the way it faced at first)
            if dot(self.was[t as usize], cn) < 0.0 {
                return false;
            }
        }
        true
    }

    /// Whether `to` is within reach of every plane `a` and `b` are held to.
    fn in_reach(&self, a: u32, b: u32, to: P) -> bool {
        self.held[a as usize]
            .iter()
            .chain(&self.held[b as usize])
            .all(|&i| {
                let (normal, offset) = self.planes[i as usize];
                (dot(normal, to) + offset).abs() <= self.reach
            })
    }

    /// Collapses the cheapest edge, and again, until `target` triangles are left or no collapse is allowed.
    fn run(&mut self, target: usize) {
        let mut edges: BTreeSet<(u32, u32)> = BTreeSet::new();
        for tri in self.tris.iter().filter(|t| t.alive) {
            for k in 0..3 {
                edges.insert(edge(tri.v[k], tri.v[(k + 1) % 3]));
            }
        }
        let mut heap: BinaryHeap<Collapse> =
            edges.iter().filter_map(|&(a, b)| self.plan(a, b)).collect();
        while self.live > target {
            let Some(c) = heap.pop() else {
                break;
            };
            if !self.fresh(&c) {
                continue;
            }
            let changed = self.collapse(&c);
            // every edge at a vertex whose surroundings changed is costed again
            let mut again: BTreeSet<(u32, u32)> = BTreeSet::new();
            for &x in &changed {
                for y in self.ring(x) {
                    again.insert(edge(x, y));
                }
            }
            heap.extend(again.into_iter().filter_map(|(a, b)| self.plan(a, b)));
        }
    }

    /// The cheapest collapse of the edge between `a` and `b` that is allowed (the module's note), if any.
    fn plan(&self, a: u32, b: u32) -> Option<Collapse> {
        let (a, b) = edge(a, b);
        let border = self.is_border(a, b);
        let (lie_a, lie_b) = (self.lie(a), self.lie(b));
        let may_go = |x: u32, lie: Lie| {
            !self.locked[x as usize] && (lie == Lie::Inside || (lie == Lie::Along && border))
        };
        let (go_a, go_b) = (may_go(a, lie_a), may_go(b, lie_b));
        if !(go_a || go_b) || !self.keeps_surface(a, b) {
            return None;
        }
        let (pa, pb) = (self.at[a as usize], self.at[b as usize]);
        let q = self.quadric[a as usize].sum(&self.quadric[b as usize]);
        let d = sub(pb, pa);
        let length = dot(d, d);
        // (gone, kept, where): the first of the cheapest is taken
        let mut ways: Vec<(u32, u32, P)> = Vec::new();
        if go_a && go_b && lie_a == lie_b {
            // two free vertices of one kind: wherever between them costs least, if that is by the edge and in
            // the mesh's box; else its middle or either end
            let mid = single(std::array::from_fn(|k| (pa[k] + pb[k]) / 2.0));
            let best = single(q.lowest(mid));
            let off = sub(best, mid);
            let inside = (0..3).all(|k| self.lo[k] <= best[k] && best[k] <= self.hi[k]);
            if inside && dot(off, off) <= length {
                ways.push((b, a, best));
            }
            ways.push((b, a, mid));
        }
        if go_b {
            ways.push((b, a, pa));
        }
        if go_a {
            ways.push((a, b, pb));
        }
        let mut best: Option<Collapse> = None;
        for (gone, kept, to) in ways {
            let cost = q.at(to) + LENGTH * length;
            if best.is_some_and(|c| c.cost <= cost) {
                continue;
            }
            let stays = to == self.at[kept as usize];
            if self.in_reach(a, b, to)
                && self.keeps_shape(gone, kept, to)
                && (stays || self.keeps_shape(kept, gone, to))
            {
                best = Some(Collapse {
                    cost,
                    gone,
                    kept,
                    to,
                    stamps: (self.stamp[a as usize], self.stamp[b as usize]),
                });
            }
        }
        best
    }

    fn fresh(&self, c: &Collapse) -> bool {
        let (a, b) = c.key();
        c.stamps == (self.stamp[a as usize], self.stamp[b as usize])
    }

    /// A moved corner's UV: where `to` lies on the triangle's own texture, as it was laid before the move.
    fn carried(old: &[P; 3], uv: &[[f32; 2]; 3], corner: usize, to: P) -> [f32; 2] {
        let (e0, e1, w) = (sub(old[1], old[0]), sub(old[2], old[0]), sub(to, old[0]));
        let (d00, d01, d11) = (dot(e0, e0), dot(e0, e1), dot(e1, e1));
        let (d20, d21) = (dot(w, e0), dot(w, e1));
        let den = d00 * d11 - d01 * d01;
        if den <= 1e-9 * d00 * d11 {
            return uv[corner];
        }
        let v = (d11 * d20 - d01 * d21) / den;
        let w = (d00 * d21 - d01 * d20) / den;
        let u = 1.0 - v - w;
        std::array::from_fn(|k| {
            (u * uv[0][k] as f64 + v * uv[1][k] as f64 + w * uv[2][k] as f64) as f32
        })
    }

    /// Makes the collapse; the vertices whose surroundings it changed.
    fn collapse(&mut self, c: &Collapse) -> Vec<u32> {
        let (u, v) = (c.gone, c.kept);
        if let Some(mut uvs) = self.uvs.take() {
            for (moved, other) in [(u, v), (v, u)] {
                if self.at[moved as usize] == c.to {
                    continue;
                }
                for &t in &self.around[moved as usize] {
                    let tri = self.tris[t as usize];
                    if tri.has(other) {
                        continue;
                    }
                    let corner = tri.v.iter().position(|&x| x == moved).unwrap_or(0);
                    uvs[t as usize][corner] =
                        Work::carried(&self.corners(t), &uvs[t as usize], corner, c.to);
                }
            }
            self.uvs = Some(uvs);
        }
        self.at[v as usize] = c.to;
        let qu = self.quadric[u as usize];
        self.quadric[v as usize].add(&qu);
        let held = std::mem::take(&mut self.held[u as usize]);
        self.held[v as usize].extend(held);
        self.held[v as usize].sort_unstable();
        self.held[v as usize].dedup();
        for t in std::mem::take(&mut self.around[u as usize]) {
            let tri = &mut self.tris[t as usize];
            if tri.has(v) {
                tri.alive = false;
                self.live -= 1;
                for x in tri.v {
                    if x != u {
                        self.around[x as usize].retain(|&s| s != t);
                    }
                }
            } else {
                for x in &mut tri.v {
                    if *x == u {
                        *x = v;
                    }
                }
                self.around[v as usize].push(t);
            }
        }
        self.into[u as usize] = v;
        let mut changed = self.ring(v);
        changed.push(v);
        changed.sort_unstable();
        for &x in &changed {
            self.stamp[x as usize] += 1;
        }
        changed
    }
}

/// A face's triangles as the modifier cuts them, each as three of the face's corners: a quad along its shorter
/// diagonal (unless that folds it), a polygon of more as it is drawn.
fn cut(p: &[V3]) -> Vec<[u32; 3]> {
    match p.len() {
        0..=2 => Vec::new(),
        3 => vec![[0, 1, 2]],
        4 => {
            let q: Vec<P> = p.iter().map(|v| v.array().map(f64::from)).collect();
            let len = |a: usize, b: usize| {
                let d = sub(q[a], q[b]);
                dot(d, d)
            };
            // a cut folds the quad if its other two corners lie on one side of it
            let folds = |a: usize, b: usize, c: usize, d: usize| {
                let along = sub(q[b], q[a]);
                dot(cross(sub(q[c], q[a]), along), cross(sub(q[d], q[a]), along)) > 0.0
            };
            let first = if len(0, 2) <= len(1, 3) {
                !folds(0, 2, 1, 3) || folds(1, 3, 0, 2)
            } else {
                folds(1, 3, 0, 2) && !folds(0, 2, 1, 3)
            };
            if first {
                vec![[0, 1, 2], [0, 2, 3]]
            } else {
                vec![[0, 1, 3], [1, 2, 3]]
            }
        }
        _ => crate::tri::polygon(p, normal_of(p)),
    }
}

/// `mesh` cut into triangles and brought down to `ratio` of them (rounded down), or as near it as the module's
/// rules allow. `locked`: for each vertex, whether it must stay as it is. The vertices and triangles left keep
/// their order, each triangle its face's material and shading; an edge is sharp where it was, and where two
/// smooth faces meet that were not shaded as one surface; the UVs follow their corners; the corners' own normals
/// and the baked shading are not kept (they were the old surface's).
pub fn collapse(mesh: &Mesh, ratio: f64, locked: &[bool]) -> Mesh {
    collapse_traced(mesh, ratio, locked).0
}

/// `collapse`, and for each triangle left the normal it had before any collapse (not of unit length): what tells
/// a triangle the collapses have turned round from one on the other side of a thin part.
pub fn collapse_traced(mesh: &Mesh, ratio: f64, locked: &[bool]) -> (Mesh, Vec<[f64; 3]>) {
    assert_eq!(
        locked.len(),
        mesh.verts.len(),
        "a lock for each of the mesh's vertices"
    );
    let at: Vec<P> = mesh
        .verts
        .iter()
        .map(|v| v.array().map(f64::from))
        .collect();
    let mut tris: Vec<Tri> = Vec::new();
    let mut uvs = mesh.uvs.as_ref().map(|_| Vec::new());
    for f in 0..mesh.faces.len() {
        let face = mesh.faces[f];
        let c = mesh.corners(f);
        let points: Vec<V3> = c.iter().map(|&v| mesh.verts[v as usize]).collect();
        for t in cut(&points) {
            let v = t.map(|k| c[k as usize]);
            if v[0] == v[1] || v[1] == v[2] || v[2] == v[0] {
                continue;
            }
            tris.push(Tri {
                v,
                mat: face.mat,
                smooth: face.smooth,
                alive: true,
                group: f as u32,
            });
            if let (Some(out), Some(have)) = (&mut uvs, &mesh.uvs) {
                out.push(t.map(|k| have[(face.start + k) as usize]));
            }
        }
    }
    let n = at.len();
    let mut around: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (t, tri) in tris.iter().enumerate() {
        for x in tri.v {
            around[x as usize].push(t as u32);
        }
    }
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for p in &at {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let mut work = Work {
        at,
        locked: locked.to_vec(),
        quadric: vec![Quadric::default(); n],
        live: tris.len(),
        tris,
        uvs,
        around,
        stamp: vec![0; n],
        into: (0..n as u32).collect(),
        was: Vec::new(),
        planes: Vec::new(),
        held: vec![Vec::new(); n],
        reach: REACH[0],
        lo,
        hi,
    };
    // each vertex's planes: its faces', and across each open edge or line between materials, square to its faces
    let mut edges: BTreeSet<(u32, u32)> = BTreeSet::new();
    for t in 0..work.tris.len() {
        let tri = work.tris[t];
        let p = work.corners(t as u32);
        work.was.push(cross(sub(p[1], p[0]), sub(p[2], p[0])));
        for k in 0..3 {
            edges.insert(edge(tri.v[k], tri.v[(k + 1) % 3]));
        }
        if let Some(normal) = unit(cross(sub(p[1], p[0]), sub(p[2], p[0]))) {
            let q = Quadric::plane(normal, p[0], 1.0);
            work.planes.push((normal, -dot(normal, p[0])));
            for x in tri.v {
                work.quadric[x as usize].add(&q);
                work.held[x as usize].push(work.planes.len() as u32 - 1);
            }
        }
    }
    for &(a, b) in &edges {
        if !work.is_border(a, b) {
            continue;
        }
        let (pa, pb) = (work.at[a as usize], work.at[b as usize]);
        let faces: Vec<u32> = work.on_edge(a, b).collect();
        for t in faces {
            let p = work.corners(t);
            let across = unit(cross(sub(p[1], p[0]), sub(p[2], p[0])))
                .and_then(|normal| unit(cross(sub(pb, pa), normal)));
            if let Some(across) = across {
                let q = Quadric::plane(across, pa, BORDER);
                work.planes.push((across, -dot(across, pa)));
                for x in [a, b] {
                    work.quadric[x as usize].add(&q);
                    work.held[x as usize].push(work.planes.len() as u32 - 1);
                }
            }
        }
    }
    // the faces shaded as one surface: joined across each edge of two smooth faces that is not sharp
    let mut group: Vec<u32> = (0..mesh.faces.len() as u32).collect();
    fn root(group: &mut [u32], mut x: u32) -> u32 {
        while group[x as usize] != x {
            group[x as usize] = group[group[x as usize] as usize];
            x = group[x as usize];
        }
        x
    }
    for &(a, b) in &edges {
        let on: Vec<u32> = work.on_edge(a, b).collect();
        if let [s, t] = on[..] {
            let (s, t) = (work.tris[s as usize], work.tris[t as usize]);
            if s.smooth && t.smooth && !mesh.sharp.contains(&(a, b)) {
                let (x, y) = (root(&mut group, s.group), root(&mut group, t.group));
                group[x.max(y) as usize] = x.min(y);
            }
        }
    }
    for tri in &mut work.tris {
        tri.group = root(&mut group, tri.group);
    }
    let target = if ratio.is_finite() {
        (work.live as f64 * ratio.clamp(0.0, 1.0)).floor() as usize
    } else {
        work.live
    };
    work.run(target);
    if work.live > target {
        let mut further = work.clone();
        further.reach = REACH[1];
        further.run(target);
        if further.live <= target {
            work = further;
        }
    }
    let was = work
        .tris
        .iter()
        .zip(&work.was)
        .filter(|(t, _)| t.alive)
        .map(|(_, n)| *n)
        .collect();
    (rebuilt(mesh, &work), was)
}

/// What is left of `mesh` after the collapses.
fn rebuilt(mesh: &Mesh, work: &Work) -> Mesh {
    let n = work.at.len();
    let mut used = vec![false; n];
    for tri in work.tris.iter().filter(|t| t.alive) {
        for x in tri.v {
            used[x as usize] = true;
        }
    }
    let mut index = vec![u32::MAX; n];
    let mut out = Mesh {
        mats: mesh.mats.clone(),
        uv_metres: mesh.uv_metres,
        ..Mesh::default()
    };
    let mut card = mesh.card_shade.as_ref().map(|_| Vec::new());
    for x in 0..n {
        if used[x] {
            index[x] = out.verts.len() as u32;
            let p = work.at[x];
            out.verts
                .push(V3::new(p[0] as f32, p[1] as f32, p[2] as f32));
            if let (Some(card), Some(have)) = (&mut card, &mesh.card_shade) {
                card.push(have[x]);
            }
        }
    }
    out.card_shade = card;
    let mut uvs = work.uvs.as_ref().map(|_| Vec::new());
    let mut left: BTreeMap<(u32, u32), Vec<&Tri>> = BTreeMap::new();
    for (t, tri) in work.tris.iter().enumerate().filter(|(_, t)| t.alive) {
        let v = tri.v.map(|x| index[x as usize]);
        out.push_face(&v, tri.mat, tri.smooth);
        for k in 0..3 {
            left.entry(edge(v[k], v[(k + 1) % 3]))
                .or_default()
                .push(tri);
        }
        if let (Some(out), Some(have)) = (&mut uvs, &work.uvs) {
            out.extend(have[t]);
        }
    }
    out.uvs = uvs;
    // an edge the collapses brought two smooth faces to that were not shaded as one surface is sharp
    for (e, faces) in &left {
        if let [s, t] = faces[..]
            && s.smooth
            && t.smooth
            && s.group != t.group
        {
            out.sharp.insert(*e);
        }
    }
    // and a sharp edge stays sharp where its two vertices, or what they were merged into, still share an edge
    let mut merged: BTreeMap<u32, u32> = BTreeMap::new();
    let mut end = |x: u32| {
        let mut at = x;
        while work.into[at as usize] != at {
            at = work.into[at as usize];
        }
        *merged.entry(x).or_insert(at)
    };
    for &(a, b) in &mesh.sharp {
        if (a as usize) < n && (b as usize) < n {
            let (a, b) = (end(a), end(b));
            if a != b && used[a as usize] && used[b as usize] {
                let e = edge(index[a as usize], index[b as usize]);
                if left.contains_key(&e) {
                    out.sharp.insert(e);
                }
            }
        }
    }
    out
}
