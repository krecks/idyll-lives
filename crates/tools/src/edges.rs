//! The gate's check `edges` and `cargo review edges`: the hard edges a model shows unbevelled at the play camera.
//!
//! Nothing made has an edge sharp to the millimetre: a board's arris is eased, sheet metal is folded round, paint
//! and moulded plastic round every corner. Such an edge catches the light as a thin bright line. Two faces of a
//! mesh that just meet have none, and read as a drawn box. At the play camera a pixel is about 4 mm at its
//! nearest (250 px a metre), so a bevel of 4 mm is that line, one pixel wide.
//!
//! A hard edge is a mesh edge
//!
//! - shared by exactly two triangles once positions are welded at 0.1 mm (over the whole shape: a part's faces
//!   may be of two materials), wound the same way round;
//! - turning 60° or more between their faces, and convex (a corner sticking out, not a crease);
//! - split in the file at both ends: its two faces' corners there have normals 40° or more apart (gamekit's
//!   `SHARP`), so a bevel shaded with hardened normals passes by construction, as does a chamfer (45°);
//! - with both its surfaces 12 mm or more wide across it (a narrower one is a board's edge, too thin to bevel),
//!   measured over the surface from the edge's middle until the surface ends at another such edge;
//! - on a piece (what its triangles are joined to) 10 cm or more across.
//!
//! Hard edges run on as one from each to the next that goes on within 30° of straight, so a round top's rim is one
//! edge and a box's corner ends three. A finding is such a run of which 5 cm or more shows. Of a hard edge, looked at
//! every 10 cm or so, what shows is what
//!
//! - doesn't look down (an underside: the middle of its faces' normals more than 17° under level) nor lies on the
//!   floor, under the 1 cm a house draws its floor at;
//! - is seen, pushed 3 mm out, from two or more of the play camera's views on its side (`views`: 12 round, at four
//!   heights between the camera's lowest and steepest) with no face that looks at the camera between: not inside a
//!   solid, nor in a closed cabinet;
//! - has both its faces open beside it: 3 mm in from the edge no other face lies on it or within 3 mm in front of
//!   it (a door on its carcass, a box against its neighbour), and it doesn't go into another solid before it is
//!   12 mm wide (a frame standing 5 mm proud of its panel).
//!
//! Leaves, bark, glass and what glows are not judged (`Mesh::edged`); glass and cut-out cards hide nothing
//! (`Mesh::opaque`).
//!
//! The rule reads plain meshes (`Mesh`), so it runs the same on a model file (`checkglb::meshes`), on geometry
//! the game builds and on a generator's meshes before they are written. It uses no function that differs from one
//! machine to the next (roots and divisions only), so a count is the same wherever it is taken.
//!
//! ```text
//! cargo review edges [--all] [--check] [model keys…]
//! ```

use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasherDefault, Hasher};

/// Positions nearer than this (m) are one place.
const WELD: f64 = 1e-4;
/// The cosine of the least turn between two faces that makes their edge hard (60°).
const TURN: f64 = 0.5;
/// The cosine of the least angle between two corners' normals that splits them (40°, gamekit's `SHARP`).
const SPLIT: f64 = 0.766_044_443_118_978;
/// The cosine of the most two faces turn and are still one flat surface (2°).
const FLAT: f64 = 0.999_390_827_019_095_8;
/// The cosine of the most a run of edges turns from one to the next (30°: a round thing of 12 sides or more).
const RUN: f64 = 0.866_025_403_784_438_6;
/// An edge shorter than this (m) hardly shows: 12 pixels at the play camera.
pub const LENGTH: f64 = 0.05;
/// A surface narrower than this (m) across an edge is too thin for a bevel that shows (three of 4 mm).
pub const WIDTH: f64 = 0.012;
/// A piece smaller than this (m) is a detail: a knob, a button, a bottle's cap.
pub const PIECE: f64 = 0.10;
/// An edge whose faces look down more than this together (the y of their normals' bisector) is an underside.
const DOWN: f64 = -0.3;
/// An edge wholly under this height (m) lies on the floor, which a house draws 1 cm up.
const FLOOR: f64 = 0.012;
/// How far (m) an edge is pushed out, and its faces looked at in from it, to see whether it is covered.
const OUT: f64 = 0.003;
/// What a measure may be short of its limit by (m, and of a cosine): a size built as 5 cm reads a hair under it.
const SLACK: f64 = 1e-5;

pub type V3 = [f64; 3];

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn mul(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(a: V3) -> f64 {
    dot(a, a).sqrt()
}

/// `a` at length 1 (as it is if it has none).
fn unit(a: V3) -> V3 {
    let l = length(a);
    if l > 0.0 { mul(a, 1.0 / l) } else { a }
}

/// A triangle mesh as drawn: one material of one part, in metres, y up, the floor at y = 0.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    /// The part it belongs to (a node's name), for the reports.
    pub part: String,
    /// Its material's name.
    pub material: String,
    /// The finish it is drawn with (`wood`: a manifest's `material.finish.wood`), or none.
    pub finish: String,
    /// Whether its edges are judged: not leaves, bark, glass or a thing that glows.
    pub edged: bool,
    /// Whether it hides what is behind it: not glass, nor cards cut out of a picture.
    pub opaque: bool,
    pub positions: Vec<V3>,
    /// A normal for each position (none: every face flat).
    pub normals: Vec<V3>,
    /// A UV for each position (none: not textured).
    pub uvs: Vec<[f64; 2]>,
    pub triangles: Vec<[u32; 3]>,
}

/// A hard edge that shows: a run of mesh edges (the module's rule).
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// The part and material of its longest stretch.
    pub part: String,
    pub material: String,
    /// How much of it shows, in metres.
    pub length: f64,
    /// The middle of its longest stretch, and that stretch's direction.
    pub at: V3,
    pub along: V3,
}

/// A hasher for keys that are already spread (welded places, pairs of indices): multiply and rotate.
#[derive(Default)]
struct Mix(u64);

impl Hasher for Mix {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.write_u64(*b as u64);
        }
    }

    fn write_u64(&mut self, v: u64) {
        self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x517c_c1b7_2722_0a95);
    }

    fn write_u32(&mut self, v: u32) {
        self.write_u64(v as u64);
    }

    fn write_i64(&mut self, v: i64) {
        self.write_u64(v as u64);
    }

    fn write_usize(&mut self, v: usize) {
        self.write_u64(v as u64);
    }
}

type Map<K, V> = HashMap<K, V, BuildHasherDefault<Mix>>;

/// A triangle of the welded shape.
struct Face {
    mesh: u32,
    /// Its corners' places.
    v: [u32; 3],
    /// Its corners in its mesh.
    c: [u32; 3],
    /// Its normal, from its winding.
    n: V3,
}

/// The triangles on an edge: how many, and the first two as (face, the corner the edge starts at there).
#[derive(Clone, Copy, Default)]
struct Sides {
    count: u32,
    at: [(u32, u8); 2],
}

/// Every mesh's triangles with their positions welded: what the eye takes for one shape.
struct Shape<'a> {
    meshes: &'a [Mesh],
    place: Vec<V3>,
    faces: Vec<Face>,
    edges: Map<(u32, u32), Sides>,
    /// The edges in the order they were met.
    order: Vec<(u32, u32)>,
}

impl<'a> Shape<'a> {
    fn new(meshes: &'a [Mesh]) -> Shape<'a> {
        let mut ids: Map<[i64; 3], u32> = Map::default();
        let mut shape = Shape {
            meshes,
            place: Vec::new(),
            faces: Vec::new(),
            edges: Map::default(),
            order: Vec::new(),
        };
        for (m, mesh) in meshes.iter().enumerate() {
            let weld: Vec<u32> = mesh
                .positions
                .iter()
                .map(|p| {
                    let key = p.map(|c| (c / WELD).round() as i64);
                    *ids.entry(key).or_insert_with(|| {
                        shape.place.push(*p);
                        shape.place.len() as u32 - 1
                    })
                })
                .collect();
            for t in &mesh.triangles {
                let Some(v) = t
                    .iter()
                    .map(|&i| weld.get(i as usize).copied())
                    .collect::<Option<Vec<u32>>>()
                else {
                    continue;
                };
                let v = [v[0], v[1], v[2]];
                if v[0] == v[1] || v[1] == v[2] || v[0] == v[2] {
                    continue;
                }
                let [a, b, c] = v.map(|i| shape.place[i as usize]);
                let n = cross(sub(b, a), sub(c, a));
                if length(n) < 1e-12 {
                    continue;
                }
                let face = shape.faces.len() as u32;
                shape.faces.push(Face {
                    mesh: m as u32,
                    v,
                    c: *t,
                    n: unit(n),
                });
                for k in 0..3 {
                    let key = edge_key(v[k], v[(k + 1) % 3]);
                    let sides = shape.edges.entry(key).or_insert_with(|| {
                        shape.order.push(key);
                        Sides::default()
                    });
                    if let Some(slot) = sides.at.get_mut(sides.count as usize) {
                        *slot = (face, k as u8);
                    }
                    sides.count += 1;
                }
            }
        }
        shape
    }

    /// The normal of `face`'s corner `k` as the file has it (the face's own where the mesh has none).
    fn normal(&self, face: &Face, k: usize) -> V3 {
        self.meshes[face.mesh as usize]
            .normals
            .get(face.c[k] as usize)
            .map_or(face.n, |n| unit(*n))
    }

    /// The two faces of an edge as (face, corner the edge starts at), if it has just two wound the same way
    /// round (each runs along it the other way).
    fn pair(&self, sides: &Sides) -> Option<[(usize, usize); 2]> {
        if sides.count != 2 {
            return None;
        }
        let [(a, ka), (b, kb)] = sides.at.map(|(f, k)| (f as usize, k as usize));
        (self.faces[a].v[ka] == self.faces[b].v[(kb + 1) % 3]).then_some([(a, ka), (b, kb)])
    }

    /// Whether the two faces of an edge are split there in the file: their corners' normals `SPLIT` apart at
    /// both of its ends.
    fn split(&self, [(a, ka), (b, kb)]: [(usize, usize); 2]) -> bool {
        let (fa, fb) = (&self.faces[a], &self.faces[b]);
        let apart =
            |i: usize, j: usize| dot(self.normal(fa, i), self.normal(fb, j)) <= SPLIT + SLACK;
        apart(ka, (kb + 1) % 3) && apart((ka + 1) % 3, kb)
    }

    /// Whether a surface ends at an edge: it has not just two faces, or they turn there and are split.
    fn ends(&self, sides: &Sides) -> bool {
        match self.pair(sides) {
            None => true,
            Some(pair) => {
                dot(self.faces[pair[0].0].n, self.faces[pair[1].0].n) < FLAT && self.split(pair)
            }
        }
    }

    /// Whether the surface on `face`'s side of its edge `k` is `limit` wide across the edge's middle: a line drawn
    /// from there straight over the surface (unfolded from triangle to triangle) gets that far before the surface
    /// ends (`ends`).
    fn wide(&self, mut face: usize, mut k: usize, limit: f64) -> bool {
        let f = &self.faces[face];
        let (a, b) = (
            self.place[f.v[k] as usize],
            self.place[f.v[(k + 1) % 3] as usize],
        );
        let mut at = mul(add(a, b), 0.5);
        let mut way = unit(cross(f.n, sub(b, a)));
        let mut gone = 0.0;
        for _ in 0..256 {
            let f = &self.faces[face];
            // Out through one of the triangle's other two edges: (how far, the corner it starts at).
            let mut out: Option<(f64, usize)> = None;
            for j in [(k + 1) % 3, (k + 2) % 3] {
                let p = self.place[f.v[j] as usize];
                let pq = sub(self.place[f.v[(j + 1) % 3] as usize], p);
                let det = dot(cross(way, pq), f.n);
                if det.abs() < 1e-15 {
                    continue;
                }
                let to = sub(p, at);
                let far = dot(cross(to, pq), f.n) / det;
                let on = dot(cross(to, way), f.n) / det;
                if far > 1e-9 && (-1e-6..=1.0 + 1e-6).contains(&on) && out.is_none_or(|o| far < o.0)
                {
                    out = Some((far, j));
                }
            }
            let Some((far, j)) = out else {
                return false;
            };
            gone += far;
            if gone >= limit - SLACK {
                return true;
            }
            let (p, q) = (f.v[j], f.v[(j + 1) % 3]);
            let Some(sides) = self.edges.get(&edge_key(p, q)).filter(|s| !self.ends(s)) else {
                return false;
            };
            // Over the edge, turned into the next triangle's plane about it.
            let edge = unit(sub(self.place[q as usize], self.place[p as usize]));
            let outward = cross(edge, f.n);
            let (next, nk) = sides.at.map(|(f, k)| (f as usize, k as usize))
                [usize::from(sides.at[0].0 as usize == face)];
            at = add(at, mul(way, far));
            way = unit(add(
                mul(edge, dot(way, edge)),
                mul(cross(edge, self.faces[next].n), dot(way, outward)),
            ));
            (face, k) = (next, nk);
        }
        false
    }

    /// The largest side (m) of the box round each piece (the places joined through triangles), by place.
    fn pieces(&self) -> Vec<f64> {
        let mut parent: Vec<u32> = (0..self.place.len() as u32).collect();
        fn root(parent: &mut [u32], mut i: u32) -> u32 {
            while parent[i as usize] != i {
                parent[i as usize] = parent[parent[i as usize] as usize];
                i = parent[i as usize];
            }
            i
        }
        for f in &self.faces {
            let a = root(&mut parent, f.v[0]);
            for &v in &f.v[1..] {
                let b = root(&mut parent, v);
                parent[b as usize] = a;
            }
        }
        let mut boxes: Vec<(V3, V3)> =
            vec![([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]); self.place.len()];
        let roots: Vec<u32> = (0..self.place.len() as u32)
            .map(|i| root(&mut parent, i))
            .collect();
        for (i, p) in self.place.iter().enumerate() {
            let (lo, hi) = &mut boxes[roots[i] as usize];
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        roots
            .iter()
            .map(|&r| {
                let (lo, hi) = boxes[r as usize];
                (0..3).map(|k| hi[k] - lo[k]).fold(0.0, f64::max)
            })
            .collect()
    }
}

fn edge_key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

/// The ways to the play camera from what it looks at: every 30° or so round, at four heights between its lowest
/// (13° over the ground) and its steepest (73°). (From whole numbers, roots and divisions only: the same on every
/// machine.)
fn views() -> &'static [V3] {
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
type Hid = [Option<usize>; 48];

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
    fn shows(
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

/// A mesh edge that is hard (the module's rule, but for how much of it shows).
struct Hard {
    ends: [u32; 2],
    faces: [usize; 2],
    length: f64,
    shown: f64,
}

/// `each(what it keeps from one to the next, i)` for every i under `n`, in order, shared out over the machine's
/// threads when there is enough of it (each thread keeping its own).
fn spread<T: Send, K>(
    n: usize,
    keep: impl Fn() -> K + Sync,
    each: impl Fn(&mut K, usize) -> T + Sync,
) -> Vec<T> {
    let threads = std::thread::available_parallelism().map_or(1, |t| t.get().min(8));
    if threads < 2 || n < 64 {
        let mut kept = keep();
        return (0..n).map(|i| each(&mut kept, i)).collect();
    }
    let share = n.div_ceil(threads);
    std::thread::scope(|scope| {
        let (keep, each) = (&keep, &each);
        let shares: Vec<_> = (0..n)
            .step_by(share)
            .map(|from| {
                scope.spawn(move || {
                    let mut kept = keep();
                    (from..(from + share).min(n))
                        .map(|i| each(&mut kept, i))
                        .collect::<Vec<T>>()
                })
            })
            .collect();
        shares
            .into_iter()
            .flat_map(|s| {
                s.join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    })
}

/// The hard edges of a shape that show unbevelled (the module's rule), the longest first.
pub fn hard_edges(meshes: &[Mesh]) -> Vec<Edge> {
    hard_edges_from(meshes, &|_| true)
}

/// `hard_edges` where only the play camera's views `from` takes (each by the way to it) see the shape: a house
/// with the walls on one side cut away is what the cameras on that side see, and no other.
pub fn hard_edges_from(meshes: &[Mesh], from: &(dyn Fn(V3) -> bool + Sync)) -> Vec<Edge> {
    let shape = Shape::new(meshes);
    let mut pieces: Option<Vec<f64>> = None;
    let mut hard: Vec<Hard> = Vec::new();
    for key in &shape.order {
        let Some(pair) = shape.pair(&shape.edges[key]) else {
            continue;
        };
        let [(a, ka), (b, kb)] = pair;
        let (fa, fb) = (&shape.faces[a], &shape.faces[b]);
        if !meshes[fa.mesh as usize].edged || !meshes[fb.mesh as usize].edged {
            continue;
        }
        if dot(fa.n, fb.n) > TURN + SLACK {
            continue;
        }
        let ends = [fa.v[ka], fa.v[(ka + 1) % 3]];
        // Convex: the far corner of one face lies behind the other.
        let far = shape.place[fb.v[(kb + 2) % 3] as usize];
        if dot(fa.n, sub(far, shape.place[ends[0] as usize])) > -1e-6 || !shape.split(pair) {
            continue;
        }
        let pieces = pieces.get_or_insert_with(|| shape.pieces());
        if pieces[ends[0] as usize] < PIECE - SLACK {
            continue;
        }
        if !shape.wide(a, ka, WIDTH) || !shape.wide(b, kb, WIDTH) {
            continue;
        }
        let [p0, p1] = ends.map(|e| shape.place[e as usize]);
        hard.push(Hard {
            ends,
            faces: [a, b],
            length: length(sub(p1, p0)),
            shown: 0.0,
        });
    }
    // Runs: at each place, two hard edges go on as one where each is the other's straightest way on.
    let mut at_place: Map<u32, Vec<u32>> = Map::default();
    for (i, h) in hard.iter().enumerate() {
        for end in h.ends {
            at_place.entry(end).or_default().push(i as u32);
        }
    }
    let way_from = |i: u32, place: u32| {
        let h = &hard[i as usize];
        let other = h.ends[usize::from(h.ends[0] == place)];
        unit(sub(
            shape.place[other as usize],
            shape.place[place as usize],
        ))
    };
    let mut run: Vec<u32> = (0..hard.len() as u32).collect();
    fn root(run: &mut [u32], mut i: u32) -> u32 {
        while run[i as usize] != i {
            run[i as usize] = run[run[i as usize] as usize];
            i = run[i as usize];
        }
        i
    }
    for (&place, edges) in at_place.iter().filter(|(_, e)| e.len() > 1) {
        // Each edge's straightest way on: the edge leaving most nearly opposite to it.
        let on = |i: u32| {
            let way = way_from(i, place);
            edges
                .iter()
                .filter(|&&j| j != i)
                .map(|&j| (j, -dot(way, way_from(j, place))))
                .filter(|(_, straight)| *straight >= RUN - SLACK)
                .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
                .map(|(j, _)| j)
        };
        for &i in edges {
            if let Some(j) = on(i).filter(|&j| j > i && on(j) == Some(i)) {
                let (a, b) = (root(&mut run, i), root(&mut run, j));
                run[b.max(a) as usize] = a.min(b);
            }
        }
    }
    let run: Vec<u32> = (0..hard.len() as u32).map(|i| root(&mut run, i)).collect();
    let mut whole = vec![0.0; hard.len()];
    for (i, h) in hard.iter().enumerate() {
        whole[run[i] as usize] += h.length;
    }
    // How much of each shows: none of an underside or of one on the floor (nor is one looked at whose run is too
    // short whatever shows of it); else looked at every 10 cm or so.
    let out_of = |h: &Hard| unit(add(shape.faces[h.faces[0]].n, shape.faces[h.faces[1]].n));
    let open: Vec<usize> = (0..hard.len())
        .filter(|&i| {
            let [p0, p1] = hard[i].ends.map(|e| shape.place[e as usize]);
            whole[run[i] as usize] >= LENGTH - SLACK
                && out_of(&hard[i])[1] >= DOWN
                && p0[1].max(p1[1]) > FLOOR
        })
        .collect();
    if !open.is_empty() {
        let solid = Solid::new(meshes);
        let shown = spread(
            open.len(),
            || [None; 48],
            |last: &mut Hid, i| {
                let h = &hard[open[i]];
                let [p0, p1] = h.ends.map(|e| shape.place[e as usize]);
                let along = sub(p1, p0);
                let [na, nb] = h.faces.map(|f| shape.faces[f].n);
                let faces = [(na, unit(cross(na, along))), (nb, unit(cross(along, nb)))];
                let n = ((h.length / (2.0 * LENGTH)).round() as usize).clamp(1, 6);
                let seen = (0..n)
                    .filter(|&k| {
                        let p = add(p0, mul(along, (k as f64 + 0.5) / n as f64));
                        solid.shows(p, out_of(h), faces, last, from)
                    })
                    .count();
                h.length * seen as f64 / n as f64
            },
        );
        for (i, shown) in open.into_iter().zip(shown) {
            hard[i].shown = shown;
        }
    }
    // Each run: how much of it shows, and its longest stretch that does.
    let mut runs: BTreeMap<u32, (f64, usize)> = BTreeMap::new();
    for i in 0..hard.len() {
        let (shown, longest) = runs.entry(run[i]).or_insert((0.0, i));
        *shown += hard[i].shown;
        if hard[i].shown > hard[*longest].shown {
            *longest = i;
        }
    }
    let mut out: Vec<Edge> = runs
        .values()
        .filter(|(shown, _)| *shown >= LENGTH - SLACK)
        .map(|&(shown, longest)| {
            let h = &hard[longest];
            let mesh = &meshes[shape.faces[h.faces[0]].mesh as usize];
            let [a, b] = h.ends.map(|e| shape.place[e as usize]);
            Edge {
                part: mesh.part.clone(),
                material: mesh.material.clone(),
                length: shown,
                at: mul(add(a, b), 0.5),
                along: mul(sub(b, a), 1.0 / h.length.max(1e-12)),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.length
            .total_cmp(&a.length)
            .then_with(|| a.at.map(f64::to_bits).cmp(&b.at.map(f64::to_bits)))
    });
    out
}

/// One model's hard edges that show (`hard_edges`).
#[derive(Debug, Clone, Default)]
pub struct Found {
    pub key: String,
    pub family: String,
    pub triangles: usize,
    pub edges: Vec<Edge>,
}

impl Found {
    /// What its edges come to, in metres.
    pub fn length(&self) -> f64 {
        self.edges.iter().fold(0.0, |sum, e| sum + e.length)
    }
}

/// A coordinate to the centimetre, no `-0.00`.
fn cm(v: f64) -> String {
    format!("{:.2}", if v.abs() < 0.005 { 0.0 } else { v })
}

/// Where an edge is, as its model is built (x across, y back, z up: a file's x, -z and y) and which way it runs.
fn place(e: &Edge) -> String {
    let built = |p: V3| [p[0], -p[2], p[1]];
    let (at, along) = (built(e.at), built(e.along));
    let axis = (0..3).find(|&k| along[k].abs() > 0.985);
    format!(
        "({}, {}, {}) {}",
        cm(at[0]),
        cm(at[1]),
        cm(at[2]),
        axis.map_or("slanted", |k| ["along x", "along y", "upright"][k])
    )
}

/// An edge as the reports name it: its part, material, length and place.
fn named(e: &Edge) -> String {
    format!(
        "{} ({}) {:.2} m at {}",
        e.part,
        e.material,
        e.length,
        place(e)
    )
}

/// The gate's line for a model's hard edges (none: no line): how many, how long, the longest.
pub fn line(key: &str, edges: &[Edge]) -> Option<String> {
    let longest = edges.first()?;
    Some(format!(
        "{} hard edge{} of 5 cm or more show{} unbevelled at the play camera, {:.1} m (`cargo review edges {key}` lists them; as built, z up): the longest {}",
        edges.len(),
        if edges.len() == 1 { "" } else { "s" },
        if edges.len() == 1 { "s" } else { "" },
        edges.iter().fold(0.0, |sum, e| sum + e.length),
        named(longest)
    ))
}

/// The report's text: the families and the models with the most; `all`: every model with one; `listed`: every
/// edge of every model given.
pub fn report(found: &[Found], all: bool, listed: bool) -> String {
    use std::fmt::Write as _;
    let mut text = String::new();
    let with: Vec<&Found> = found.iter().filter(|f| !f.edges.is_empty()).collect();
    let edges: usize = with.iter().map(|f| f.edges.len()).sum();
    let metres = with.iter().fold(0.0, |sum, f| sum + f.length());
    let _ = writeln!(
        text,
        "{} models, {} with hard edges of {} cm or more that show unbevelled at the play camera: {edges} edges, {metres:.0} m of them\n",
        found.len(),
        with.len(),
        LENGTH * 100.0,
    );
    if listed {
        for f in found {
            let _ = writeln!(
                text,
                "{} ({}, {} triangles): {} edges, {:.2} m (as built: x across, y back, z up)",
                f.key,
                f.family,
                f.triangles,
                f.edges.len(),
                f.length()
            );
            for e in &f.edges {
                let _ = writeln!(text, "  {}", named(e));
            }
        }
        return text;
    }
    let mut families: BTreeMap<&str, (usize, usize, usize, f64)> = BTreeMap::new();
    for f in found {
        let family = families.entry(&f.family).or_default();
        family.0 += 1;
        family.1 += usize::from(!f.edges.is_empty());
        family.2 += f.edges.len();
        family.3 += f.length();
    }
    let _ = writeln!(
        text,
        "| family | models | with edges | edges | metres |\n|---|---|---|---|---|"
    );
    let mut rows: Vec<_> = families.iter().collect();
    rows.sort_by(|a, b| b.1.3.total_cmp(&a.1.3).then(a.0.cmp(b.0)));
    for (family, (models, with, edges, metres)) in rows {
        let _ = writeln!(
            text,
            "| {family} | {models} | {with} | {edges} | {metres:.1} |"
        );
    }
    let mut worst = with;
    worst.sort_by(|a, b| b.length().total_cmp(&a.length()).then(a.key.cmp(&b.key)));
    let _ = writeln!(
        text,
        "\nThe most{} (places as built: x across, y back, z up):\n\n| model | triangles | edges | metres | the longest |\n|---|---|---|---|---|",
        if all {
            ""
        } else {
            " (25; `--all` for every one, a model's key for each of its edges)"
        },
    );
    for f in worst.iter().take(if all { usize::MAX } else { 25 }) {
        let longest: Vec<String> = f.edges.iter().take(3).map(named).collect();
        let _ = writeln!(
            text,
            "| {} | {} | {} | {:.1} | {} |",
            f.key,
            f.triangles,
            f.edges.len(),
            f.length(),
            longest.join("; ")
        );
    }
    text
}

/// Every model's hard edges (but a held tool's: `checkglb::meshes::edged`), or those of `keys`.
pub fn survey(paths: &crate::checkglb::Paths, keys: &[String]) -> Result<Vec<Found>, String> {
    let mut out = Vec::new();
    crate::checkglb::meshes::each(paths, keys, false, &mut |key, family, meshes| {
        if crate::checkglb::meshes::edged(key) {
            out.push(Found {
                key: key.to_string(),
                family: family.to_string(),
                triangles: meshes.iter().map(|m| m.triangles.len()).sum(),
                edges: hard_edges(&meshes),
            });
        }
    })?;
    Ok(out)
}

/// `cargo review edges [--all] [--check] [keys…]`: the exit code (1 with `--check` when a model has one).
pub fn main(args: &[String]) -> i32 {
    let flag = |name: &str| args.iter().any(|a| a == name);
    let keys: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .cloned()
        .collect();
    let found = match survey(&crate::checkglb::Paths::repo(), &keys) {
        Ok(found) => found,
        Err(e) => {
            eprintln!("review edges: {e}");
            return 1;
        }
    };
    print!("{}", report(&found, flag("--all"), !keys.is_empty()));
    i32::from(flag("--check") && found.iter().any(|f| !f.edges.is_empty()))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A flat face of four corners, its own corners with its own normal, wound to look along `n`.
    fn quad(mesh: &mut Mesh, corners: [V3; 4], n: V3) {
        let at = mesh.positions.len() as u32;
        mesh.positions.extend(corners);
        mesh.normals.extend([n; 4]);
        let turned = dot(
            cross(sub(corners[1], corners[0]), sub(corners[2], corners[0])),
            n,
        ) < 0.0;
        for t in [[0, 1, 2], [0, 2, 3]] {
            let t = if turned { [t[0], t[2], t[1]] } else { t };
            mesh.triangles.push(t.map(|i| at + i));
        }
    }

    fn solid_mesh() -> Mesh {
        Mesh {
            part: "body".into(),
            material: "paint".into(),
            edged: true,
            opaque: true,
            ..Mesh::default()
        }
    }

    /// A box from `lo` to `hi`, flat shaded, without the sides named in `but` (0-5: -x, +x, -y, +y, -z, +z).
    pub(crate) fn cuboid_but(lo: V3, hi: V3, but: &[usize]) -> Mesh {
        let mut mesh = solid_mesh();
        for axis in 0..3 {
            for (side, at) in [lo[axis], hi[axis]].into_iter().enumerate() {
                if but.contains(&(axis * 2 + side)) {
                    continue;
                }
                let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
                let corner = |a: f64, b: f64| {
                    let mut p = [0.0; 3];
                    (p[axis], p[u], p[v]) = (at, a, b);
                    p
                };
                let mut n = [0.0; 3];
                n[axis] = if side == 0 { -1.0 } else { 1.0 };
                quad(
                    &mut mesh,
                    [
                        corner(lo[u], lo[v]),
                        corner(hi[u], lo[v]),
                        corner(hi[u], hi[v]),
                        corner(lo[u], hi[v]),
                    ],
                    n,
                );
            }
        }
        mesh
    }

    pub(crate) fn cuboid(lo: V3, hi: V3) -> Mesh {
        cuboid_but(lo, hi, &[])
    }

    /// The sides of an outline (x, z; going round with its inside on the left) drawn up from `y0` to `y1`, open at
    /// both ends; `normal(side, point)`: the normal of that side's corners at that point of the outline (None: the
    /// side's own, flat).
    fn prism(
        outline: &[[f64; 2]],
        y0: f64,
        y1: f64,
        normal: impl Fn(usize, usize) -> Option<V3>,
    ) -> Mesh {
        let mut mesh = solid_mesh();
        for i in 0..outline.len() {
            let j = (i + 1) % outline.len();
            let (a, b) = (outline[i], outline[j]);
            let flat = unit([b[1] - a[1], 0.0, a[0] - b[0]]);
            let at = mesh.positions.len() as u32;
            mesh.positions.extend([
                [a[0], y0, a[1]],
                [b[0], y0, b[1]],
                [b[0], y1, b[1]],
                [a[0], y1, a[1]],
            ]);
            let (na, nb) = (normal(i, i).unwrap_or(flat), normal(i, j).unwrap_or(flat));
            mesh.normals.extend([na, nb, nb, na]);
            let corners: Vec<V3> = mesh.positions[at as usize..].to_vec();
            let turned = dot(
                cross(sub(corners[1], corners[0]), sub(corners[2], corners[0])),
                flat,
            ) < 0.0;
            for t in [[0, 1, 2], [0, 2, 3]] {
                let t = if turned { [t[0], t[2], t[1]] } else { t };
                mesh.triangles.push(t.map(|k| at + k));
            }
        }
        mesh
    }

    /// A flat cap on an outline at height `y`, a fan from its middle, looking up or down.
    fn cap(mesh: &mut Mesh, outline: &[[f64; 2]], y: f64, up: bool) {
        let n = [0.0, if up { 1.0 } else { -1.0 }, 0.0];
        let middle = outline.iter().fold([0.0, y, 0.0], |m, p| {
            [
                m[0] + p[0] / outline.len() as f64,
                y,
                m[2] + p[1] / outline.len() as f64,
            ]
        });
        for i in 0..outline.len() {
            let (a, b) = (outline[i], outline[(i + 1) % outline.len()]);
            let at = mesh.positions.len() as u32;
            mesh.positions
                .extend([middle, [a[0], y, a[1]], [b[0], y, b[1]]]);
            mesh.normals.extend([n; 3]);
            let turned = dot(
                cross(sub([a[0], y, a[1]], middle), sub([b[0], y, b[1]], middle)),
                n,
            ) < 0.0;
            mesh.triangles.push(if turned {
                [at, at + 2, at + 1]
            } else {
                [at, at + 1, at + 2]
            });
        }
    }

    fn square(half: f64) -> Vec<[f64; 2]> {
        vec![[half, half], [-half, half], [-half, -half], [half, -half]]
    }

    fn lengths(edges: &[Edge]) -> Vec<f64> {
        edges
            .iter()
            .map(|e| (e.length * 1e4).round() / 1e4)
            .collect()
    }

    #[test]
    fn a_box_on_the_floor_shows_its_top_and_its_upright_edges() {
        // 30 cm wide, 20 cm high, 40 cm deep: not the four on the floor, which look down.
        let found = hard_edges(&[cuboid([-0.15, 0.0, -0.2], [0.15, 0.2, 0.2])]);
        assert_eq!(lengths(&found), [0.4, 0.4, 0.3, 0.3, 0.2, 0.2, 0.2, 0.2]);
        assert!(
            found
                .iter()
                .all(|e| e.part == "body" && e.material == "paint")
        );
        assert!(found[..4].iter().all(|e| (e.at[1] - 0.2).abs() < 1e-9));
        assert!(found[4..].iter().all(|e| e.along[1].abs() > 0.999));
        // Hung in the air its underside still doesn't count.
        let hung = hard_edges(&[cuboid([-0.15, 1.0, -0.2], [0.15, 1.2, 0.2])]);
        assert_eq!(hung.len(), 8);
    }

    #[test]
    fn a_chamfer_passes_and_so_does_an_edge_shaded_round() {
        // A post 30 cm square and 50 cm high, its four upright edges sharp: four findings.
        let sharp = prism(&square(0.15), 0.1, 0.6, |_, _| None);
        assert_eq!(lengths(&hard_edges(&[sharp])), [0.5; 4]);
        // Each cut back 4 mm, one segment: the turns are 45°, hardened normals or flat.
        let (h, c) = (0.15, 0.004);
        let cut = vec![
            [h, h - c],
            [h - c, h],
            [c - h, h],
            [-h, h - c],
            [-h, c - h],
            [c - h, -h],
            [h - c, -h],
            [h, c - h],
        ];
        let flat_of = |side: usize| {
            let (a, b) = (cut[side], cut[(side + 1) % 8]);
            unit([b[1] - a[1], 0.0, a[0] - b[0]])
        };
        // (Odd sides are the long flat ones; a chamfer's corners take the flat side's normal each end.)
        let hardened = prism(&cut, 0.1, 0.6, |side, point| {
            Some(if side % 2 == 1 {
                flat_of(side)
            } else if point == side {
                flat_of((side + 7) % 8)
            } else {
                flat_of(side + 1)
            })
        });
        assert_eq!(hard_edges(&[hardened]), []);
        assert_eq!(hard_edges(&[prism(&cut, 0.1, 0.6, |_, _| None)]), []);
        // The sharp post shaded smooth round its corners: not split, so not this rule's (its shading is wrong
        // another way).
        let round = prism(&square(0.15), 0.1, 0.6, |_, point| {
            let p = square(0.15)[point];
            Some(unit([p[0], 0.0, p[1]]))
        });
        assert_eq!(hard_edges(&[round]), []);
    }

    #[test]
    fn a_crease_is_not_an_edge() {
        // An L: five corners stick out, the one inside doesn't.
        let l = [
            [0.3, 0.0],
            [0.3, 0.15],
            [0.15, 0.15],
            [0.15, 0.3],
            [0.0, 0.3],
            [0.0, 0.0],
        ];
        let found = hard_edges(&[prism(&l, 0.1, 0.6, |_, _| None)]);
        assert_eq!(found.len(), 5);
        assert!(
            found
                .iter()
                .all(|e| (e.at[0] - 0.15).abs() > 1e-6 || (e.at[2] - 0.15).abs() > 1e-6)
        );
    }

    #[test]
    fn thin_short_and_small_things_pass() {
        let count = |lo: V3, hi: V3| hard_edges(&[cuboid(lo, hi)]).len();
        // A board 10 mm thick: its edge is too narrow to bevel. At 20 mm its top's four edges count (its
        // uprights are 2 cm long).
        assert_eq!(count([0.0, 0.5, 0.0], [0.5, 0.51, 0.3]), 0);
        assert_eq!(count([0.0, 0.5, 0.0], [0.5, 0.52, 0.3]), 4);
        // A block 8 cm across is a detail; one of 10 cm is a piece.
        assert_eq!(count([0.0, 0.5, 0.0], [0.08, 0.58, 0.08]), 0);
        assert_eq!(count([0.0, 0.5, 0.0], [0.1, 0.6, 0.1]), 8);
        // Edges of 4 cm on a piece of 30: only the long ones.
        assert_eq!(count([0.0, 0.5, 0.0], [0.3, 0.54, 0.04]), 2);
        // Lying on the floor, 1 cm high: under the floor a house draws.
        assert_eq!(count([0.0, 0.0, 0.0], [0.5, 0.012, 0.5]), 0);
    }

    #[test]
    fn a_covered_edge_passes() {
        let low = cuboid([0.0, 0.0, 0.0], [0.4, 0.3, 0.4]);
        // Under a wider box standing on it: its top edges are covered, its uprights show; the upper box's twelve
        // but its underside's four.
        let on = cuboid([-0.1, 0.3, -0.1], [0.5, 0.5, 0.5]);
        let found = hard_edges(&[low.clone(), on]);
        assert_eq!(found.len(), 4 + 8);
        assert!(
            !found
                .iter()
                .any(|e| (e.at[1] - 0.3).abs() < 1e-9 && e.along[1].abs() < 0.5 && e.length < 0.5)
        );
        // Beside a box of its size, face to face: the faces between them are no edges, and the top's edges run
        // on from one to the other.
        let beside = cuboid([0.4, 0.0, 0.0], [0.8, 0.3, 0.4]);
        assert_eq!(
            lengths(&hard_edges(&[low.clone(), beside])),
            [0.8, 0.8, 0.4, 0.4, 0.3, 0.3, 0.3, 0.3]
        );
        // Beside a shorter box that covers half of its side: half of that top edge shows.
        let half = cuboid([0.4, 0.0, 0.0], [0.8, 0.3, 0.2]);
        let found = hard_edges(&[low.clone(), half]);
        let edge = found
            .iter()
            .find(|e| {
                (e.at[0] - 0.4).abs() < 1e-9
                    && (e.at[1] - 0.3).abs() < 1e-9
                    && e.along[2].abs() > 0.5
            })
            .unwrap();
        assert!((edge.length - 0.2).abs() < 1e-9, "{}", edge.length);
        // A face 2 mm in front of its side covers it too (a door on its carcass), 2 cm off it doesn't.
        let count_at = |gap: f64| {
            let door = cuboid([0.4 + gap, 0.0, 0.0], [0.42 + gap, 0.3, 0.4]);
            hard_edges(&[low.clone(), door])
                .iter()
                .filter(|e| (e.at[0] - 0.4).abs() < 1e-9)
                .count()
        };
        assert_eq!((count_at(0.002), count_at(0.02)), (0, 3));
    }

    #[test]
    fn what_no_camera_sees_passes() {
        // A box in a chest with walls 5 cm thick: seen from nowhere. The lid off, from above; a chest of glass, as
        // if nothing were there.
        let inner = cuboid([-0.15, 0.1, -0.15], [0.15, 0.4, 0.15]);
        let chest = |lid: bool| {
            let mut parts = vec![
                cuboid([-0.5, 0.0, -0.5], [0.5, 0.05, 0.5]),
                cuboid([-0.5, 0.05, -0.5], [-0.45, 1.0, 0.5]),
                cuboid([0.45, 0.05, -0.5], [0.5, 1.0, 0.5]),
                cuboid([-0.45, 0.05, -0.5], [0.45, 1.0, -0.45]),
                cuboid([-0.45, 0.05, 0.45], [0.45, 1.0, 0.5]),
            ];
            if lid {
                parts.push(cuboid([-0.5, 1.0, -0.5], [0.5, 1.05, 0.5]));
            }
            parts
        };
        let count = |chest: Vec<Mesh>| {
            let meshes: Vec<Mesh> = std::iter::once(inner.clone()).chain(chest).collect();
            hard_edges(&meshes)
                .iter()
                .filter(|e| e.at.iter().all(|c| c.abs() < 0.42))
                .count()
        };
        assert_eq!(count(chest(true)), 0);
        assert_eq!(count(chest(false)), 8);
        let of = |edged: bool, opaque: bool| {
            chest(true)
                .into_iter()
                .map(|part| Mesh {
                    edged,
                    opaque,
                    ..part
                })
                .collect::<Vec<Mesh>>()
        };
        assert_eq!(count(of(false, false)), 8);
        // What isn't judged has no edges of its own, but hides (a hollow trunk: bark).
        assert_eq!(count(of(false, true)), 0);
    }

    #[test]
    fn a_round_rim_is_one_edge() {
        // A drum 24 round, 10 cm in radius, shaded smooth round its side: its top rim is one edge of 24 pieces,
        // each 2.6 cm; its bottom rim looks down.
        let ring: Vec<[f64; 2]> = (0..24)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 24.0;
                [0.1 * a.cos(), 0.1 * a.sin()]
            })
            .collect();
        let mut drum = prism(&ring, 0.2, 0.5, |_, point| {
            Some(unit([ring[point][0], 0.0, ring[point][1]]))
        });
        cap(&mut drum, &ring, 0.5, true);
        cap(&mut drum, &ring, 0.2, false);
        let found = hard_edges(&[drum]);
        assert_eq!(found.len(), 1);
        let rim = 24.0 * 2.0 * 0.1 * (std::f64::consts::PI / 24.0).sin();
        assert!((found[0].length - rim).abs() < 1e-9, "{}", found[0].length);
        // Eight sides, each its own flat face: eight corners, and its rim eight edges of 7.7 cm.
        let eight: Vec<[f64; 2]> = ring.iter().step_by(3).copied().collect();
        let mut post = prism(&eight, 0.2, 0.5, |_, _| None);
        cap(&mut post, &eight, 0.5, true);
        cap(&mut post, &eight, 0.2, false);
        let found = hard_edges(&[post]);
        assert_eq!(found.len(), 8, "{:?}", lengths(&found));
        assert!(found.iter().all(|e| e.along[1].abs() < 0.5));
    }

    #[test]
    fn a_box_of_two_materials_is_one_shape() {
        // Its top another material's mesh: the edges between the two are found, by the first's name.
        let whole = cuboid([0.0, 0.5, 0.0], [0.3, 0.7, 0.3]);
        let mut top = Mesh {
            material: "top".into(),
            ..cuboid_but([0.0, 0.5, 0.0], [0.3, 0.7, 0.3], &[0, 1, 2, 4, 5])
        };
        let rest = cuboid_but([0.0, 0.5, 0.0], [0.3, 0.7, 0.3], &[3]);
        assert_eq!(
            top.triangles.len() + rest.triangles.len(),
            whole.triangles.len()
        );
        assert_eq!(hard_edges(&[rest.clone(), top.clone()]).len(), 8);
        // A top that isn't judged (a glowing panel) takes its edges with it.
        top.edged = false;
        assert_eq!(hard_edges(&[rest, top]).len(), 4);
    }

    #[test]
    fn a_point_is_inside_or_seen() {
        let solid = Solid::new(&[cuboid([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])]);
        let up = [0.0, 1.0, 0.0];
        assert!(solid.inside([0.5, 0.5, 0.5], up));
        // (Down a seam of its top's two triangles too.)
        assert!(solid.inside([0.25, 0.5, 0.25], up) && solid.inside([0.5, 0.999, 0.5], up));
        assert!(!solid.inside([0.5, 1.5, 0.5], up) && !solid.inside([1.5, 0.5, 0.5], up));
        assert!(solid.near([0.5, 1.002, 0.5], [0.0, -1.0, 0.0], 0.003));
        assert!(!solid.near([0.5, 1.02, 0.5], [0.0, -1.0, 0.0], 0.003));
        // From inside, its top hides the sky; from over it, nothing does; from under it, its faces do.
        assert!(solid.hides([0.5, 0.5, 0.5], up, None).is_some());
        assert_eq!(solid.hides([0.5, 1.5, 0.5], up, None), None);
        let hid = solid.hides([0.5, -0.5, 0.5], up, None);
        assert!(hid.is_some());
        assert!(solid.hides([0.4, -0.5, 0.6], up, hid).is_some());
        assert_eq!(views().len(), 48);
        assert!(
            views()
                .iter()
                .all(|v| (length(*v) - 1.0).abs() < 1e-12 && v[1] > 0.24 && v[1] < 0.96)
        );
    }

    #[test]
    fn the_report_names_each_edge() {
        let found = Found {
            key: "model.crate".into(),
            family: "yard".into(),
            triangles: 12,
            edges: hard_edges(&[cuboid([-0.15, 0.0, -0.2], [0.15, 0.2, 0.2])]),
        };
        let line = line(&found.key, &found.edges).unwrap();
        assert!(
            line.starts_with("8 hard edges of 5 cm or more show unbevelled at the play camera, 2.2 m (`cargo review edges model.crate` lists them; as built, z up): the longest body (paint) 0.40 m at ("),
            "{line}"
        );
        assert_eq!(super::line("model.none", &[]), None);
        // As built: a file's z is its depth turned round, its y its height.
        assert_eq!(
            place(&Edge {
                part: String::new(),
                material: String::new(),
                length: 0.4,
                at: [0.15, 0.2, -0.1],
                along: [0.0, 0.0, 1.0],
            }),
            "(0.15, 0.10, 0.20) along y"
        );
        let clean = Found {
            key: "model.ball".into(),
            family: "kids".into(),
            ..Found::default()
        };
        let text = report(&[found.clone(), clean.clone()], false, false);
        assert!(
            text.starts_with("2 models, 1 with hard edges of 5 cm or more that show unbevelled at the play camera: 8 edges, 2 m of them\n"),
            "{text}"
        );
        assert!(
            text.contains("| yard | 1 | 1 | 8 | 2.2 |\n| kids | 1 | 0 | 0 | 0.0 |"),
            "{text}"
        );
        assert!(
            text.contains("| model.crate | 12 | 8 | 2.2 | body (paint) 0.40 m at ("),
            "{text}"
        );
        let listed = report(&[found], false, true);
        assert_eq!(
            listed
                .lines()
                .filter(|l| l.starts_with("  body (paint) "))
                .count(),
            8,
            "{listed}"
        );
    }

    #[test]
    fn the_shipped_models_are_read() {
        let paths = crate::checkglb::Paths::repo();
        let found = survey(&paths, &[]).unwrap();
        assert!(found.len() > 550, "{} models", found.len());
        assert!(
            found
                .iter()
                .all(|f| f.triangles > 0 && !f.key.starts_with("tool."))
        );
        // The same edges each time, whatever the threads.
        let keys: Vec<String> = found.iter().take(40).map(|f| f.key.clone()).collect();
        let again = survey(&paths, &keys).unwrap();
        for (a, b) in again.iter().zip(&found) {
            assert_eq!((&a.key, &a.edges), (&b.key, &b.edges));
        }
        assert!(survey(&paths, &["model.noSuchThing".into()]).is_err());
    }
}
