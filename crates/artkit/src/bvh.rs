//! Rays against triangles: the ray-triangle test Blender casts with (watertight, single precision), and two trees
//! of boxes over triangles: the kernel's own over a part's mesh for the scene's rays (`Rays`: the perches found on a
//! tree), and Blender's over polygons for a script's `BVHTree` (`Polygons`: the baked shading), built and walked
//! as Blender builds and walks it. A ray meets a built mesh where it met the one Blender built, and where it meets
//! two faces at once it is given Blender's.

use crate::math::{V3, normalize_v3};
use crate::mesh::normal_tri;
use crate::scene::{self, Obj};

struct Node {
    lo: [f32; 3],
    hi: [f32; 3],
    /// A leaf's triangles (`first..first + count` of `order`), or a branch's two children (`first`, `first + 1`).
    first: u32,
    count: u32,
}

pub struct Bvh {
    nodes: Vec<Node>,
    order: Vec<u32>,
    pub tris: Vec<[V3; 3]>,
}

/// A ray's direction prepared for the triangle test: its largest axis and the shear onto it.
struct Shear {
    kx: usize,
    ky: usize,
    kz: usize,
    sx: f32,
    sy: f32,
    sz: f32,
}

impl Shear {
    fn new(dir: V3) -> Shear {
        let (x, y, z) = (dir.x.abs(), dir.y.abs(), dir.z.abs());
        let kz = if x > y {
            if x > z { 0 } else { 2 }
        } else if y > z {
            1
        } else {
            2
        };
        let mut kx = if kz != 2 { kz + 1 } else { 0 };
        let mut ky = if kx != 2 { kx + 1 } else { 0 };
        if dir[kz] < 0.0 {
            std::mem::swap(&mut kx, &mut ky);
        }
        let inv = 1.0 / dir[kz];
        Shear {
            kx,
            ky,
            kz,
            sx: dir[kx] * inv,
            sy: dir[ky] * inv,
            sz: inv,
        }
    }

    /// How far along the ray from `origin` it meets the triangle, if it does (either side of it).
    fn hit(&self, origin: V3, t: &[V3; 3]) -> Option<f32> {
        let (a, b, c) = (t[0] - origin, t[1] - origin, t[2] - origin);
        let (a_kx, a_ky, a_kz) = (a[self.kx], a[self.ky], a[self.kz]);
        let (b_kx, b_ky, b_kz) = (b[self.kx], b[self.ky], b[self.kz]);
        let (c_kx, c_ky, c_kz) = (c[self.kx], c[self.ky], c[self.kz]);
        let ax = a_kx - self.sx * a_kz;
        let ay = a_ky - self.sy * a_kz;
        let bx = b_kx - self.sx * b_kz;
        let by = b_ky - self.sy * b_kz;
        let cx = c_kx - self.sx * c_kz;
        let cy = c_ky - self.sy * c_kz;
        let u = cx * by - cy * bx;
        let v = ax * cy - ay * cx;
        let w = bx * ay - by * ax;
        if (u < 0.0 || v < 0.0 || w < 0.0) && (u > 0.0 || v > 0.0 || w > 0.0) {
            return None;
        }
        let det = u + v + w;
        if det == 0.0 || !det.is_finite() {
            return None;
        }
        let t = (u * a_kz + v * b_kz + w * c_kz) * self.sz;
        let signed = f32::from_bits(t.to_bits() ^ (det.to_bits() & 0x8000_0000));
        if signed < 0.0 {
            return None;
        }
        Some(t * (1.0 / det))
    }
}

impl Bvh {
    pub fn new(tris: Vec<[V3; 3]>) -> Bvh {
        let mut bvh = Bvh {
            nodes: Vec::new(),
            order: (0..tris.len() as u32).collect(),
            tris,
        };
        if !bvh.tris.is_empty() {
            let mid: Vec<[f32; 3]> = bvh
                .tris
                .iter()
                .map(|t| [0, 1, 2].map(|i| (t[0][i] + t[1][i] + t[2][i]) / 3.0))
                .collect();
            bvh.nodes.push(Node {
                lo: [0.0; 3],
                hi: [0.0; 3],
                first: 0,
                count: 0,
            });
            let n = bvh.order.len();
            bvh.build(0, 0, n, &mid);
        }
        bvh
    }

    fn build(&mut self, node: usize, start: usize, end: usize, mid: &[[f32; 3]]) {
        let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        for &t in &self.order[start..end] {
            for p in &self.tris[t as usize] {
                for i in 0..3 {
                    lo[i] = lo[i].min(p[i]);
                    hi[i] = hi[i].max(p[i]);
                }
            }
        }
        (self.nodes[node].lo, self.nodes[node].hi) = (lo, hi);
        if end - start <= 4 {
            (self.nodes[node].first, self.nodes[node].count) = (start as u32, (end - start) as u32);
            return;
        }
        let axis = (0..3)
            .max_by(|&a, &b| (hi[a] - lo[a]).total_cmp(&(hi[b] - lo[b])))
            .unwrap_or(0);
        self.order[start..end].sort_by(|&a, &b| {
            mid[a as usize][axis]
                .total_cmp(&mid[b as usize][axis])
                .then(a.cmp(&b))
        });
        let half = (start + end) / 2;
        let first = self.nodes.len();
        self.nodes.push(Node {
            lo: [0.0; 3],
            hi: [0.0; 3],
            first: 0,
            count: 0,
        });
        self.nodes.push(Node {
            lo: [0.0; 3],
            hi: [0.0; 3],
            first: 0,
            count: 0,
        });
        (self.nodes[node].first, self.nodes[node].count) = (first as u32, 0);
        self.build(first, start, half, mid);
        self.build(first + 1, half, end, mid);
    }

    /// The first triangle the ray from `origin` along `dir` (a unit vector) meets within `reach`: (which, how
    /// far). Of two as near as each other, the earlier.
    pub fn ray_cast(&self, origin: V3, dir: V3, reach: f32) -> Option<(u32, f32)> {
        if self.nodes.is_empty() {
            return None;
        }
        let shear = Shear::new(dir);
        let o = [origin.x as f64, origin.y as f64, origin.z as f64];
        let inv = [1.0 / dir.x as f64, 1.0 / dir.y as f64, 1.0 / dir.z as f64];
        let mut best: Option<(u32, f32)> = None;
        let mut limit = reach;
        let mut stack = vec![0u32];
        while let Some(n) = stack.pop() {
            let node = &self.nodes[n as usize];
            // the box, a hair larger: no triangle is passed over for the rounding of this test
            let (mut t0, mut t1) = (0.0_f64, limit as f64 * 1.000_001 + 1e-6);
            let mut miss = false;
            for i in 0..3 {
                let (lo, hi) = (node.lo[i] as f64 - 1e-5, node.hi[i] as f64 + 1e-5);
                if inv[i].is_infinite() {
                    miss |= o[i] < lo || o[i] > hi;
                    continue;
                }
                let (a, b) = ((lo - o[i]) * inv[i], (hi - o[i]) * inv[i]);
                t0 = t0.max(a.min(b));
                t1 = t1.min(a.max(b));
            }
            if miss || t0 > t1 {
                continue;
            }
            if node.count == 0 {
                stack.push(node.first);
                stack.push(node.first + 1);
                continue;
            }
            for &t in &self.order[node.first as usize..(node.first + node.count) as usize] {
                if let Some(dist) = shear.hit(origin, &self.tris[t as usize]) {
                    let nearer = match best {
                        None => dist >= 0.0 && dist < limit,
                        Some((b, d)) => dist >= 0.0 && (dist < d || (dist == d && t < b)),
                    };
                    if nearer {
                        best = Some((t, dist));
                        limit = dist;
                    }
                }
            }
        }
        best
    }
}

// ---- a script's own tree: mathutils' BVHTree.FromPolygons ---------------------------------------------------

/// A node of Blender's tree (BLI_kdopbvh, GPL-2.0-or-later, worked through again here): a leaf is one triangle,
/// a branch up to four nodes.
#[derive(Clone)]
struct Knot {
    /// Its box: the least and the most along x, then y, then z.
    bv: [f32; 6],
    children: [u32; 4],
    /// How many children it has (none: a leaf).
    count: u8,
    /// The axis its children were sorted along.
    axis: u8,
}

/// What Blender's tree is built by: how many leaves hang under a branch at each depth of a tree filled from
/// the left, four to a branch.
struct Levels {
    leaves: i64,
    per_child: [i64; 32],
    on_level: [i64; 32],
    remain: i64,
}

const FOUR: i64 = 4;

impl Levels {
    fn new(leaves: i64) -> Levels {
        let (mut per_child, mut on_level) = ([0_i64; 32], [0_i64; 32]);
        per_child[0] = 1;
        while per_child[0] < leaves {
            per_child[0] *= FOUR;
        }
        on_level[0] = 1;
        let mut depth = 1;
        while depth < 32 && per_child[depth - 1] != 0 {
            on_level[depth] = on_level[depth - 1] * FOUR;
            per_child[depth] = per_child[depth - 1] / FOUR;
            depth += 1;
        }
        let remain = leaves - per_child[1];
        let nodes = (remain + FOUR - 2) / (FOUR - 1);
        Levels {
            leaves,
            per_child,
            on_level,
            remain: remain + nodes,
        }
    }

    /// The first leaf under the branch that is `child` along its level at `depth`.
    fn first_leaf(&self, depth: usize, child: i64) -> usize {
        let least = child * self.per_child[depth - 1];
        let at = if least <= self.remain {
            least
        } else if self.per_child[depth] != 0 {
            self.leaves - (self.on_level[depth - 1] - child) * self.per_child[depth]
        } else {
            self.remain
        };
        at as usize
    }
}

/// Blender's partial sort of the leaves `a[begin..end]` by `key`: those before `n` no greater than those after
/// it, left in the order its quicksort leaves them (which decides the order equal leaves are met in).
fn nth_element(
    a: &mut [u32],
    key: &impl Fn(u32) -> f32,
    mut begin: usize,
    mut end: usize,
    n: usize,
) {
    while end - begin > 3 {
        // the median of the first, the middle and the last
        let (lo, mid, hi) = (key(a[begin]), key(a[(begin + end) / 2]), key(a[end - 1]));
        let x = if mid < lo {
            if hi < mid {
                mid
            } else if hi < lo {
                hi
            } else {
                lo
            }
        } else if hi < mid {
            if hi < lo { lo } else { hi }
        } else {
            mid
        };
        let (mut i, mut j) = (begin, end);
        let cut = loop {
            while key(a[i]) < x {
                i += 1;
            }
            j -= 1;
            while x < key(a[j]) {
                j -= 1;
            }
            if i >= j {
                break i;
            }
            a.swap(i, j);
            i += 1;
        };
        if cut <= n {
            begin = cut;
        } else {
            end = cut;
        }
    }
    for i in begin..end {
        let t = a[i];
        let mut j = i;
        while j != begin && key(t) < key(a[j - 1]) {
            a[j] = a[j - 1];
            j -= 1;
        }
        a[j] = t;
    }
}

/// Blender's tree over triangles, as `BLI_bvhtree_new(n, 0, 4, 6)`, an insert a triangle and
/// `BLI_bvhtree_balance` leave it: the leaves first (a triangle each, in the order given), then the branches,
/// the root the first of them.
struct Kdop {
    knots: Vec<Knot>,
    leaves: usize,
}

const NO_BOX: [f32; 6] = [
    f32::MAX,
    -f32::MAX,
    f32::MAX,
    -f32::MAX,
    f32::MAX,
    -f32::MAX,
];

impl Kdop {
    fn new(tris: &[[V3; 3]]) -> Kdop {
        let leaves = tris.len();
        let mut knots: Vec<Knot> = tris
            .iter()
            .map(|t| {
                let mut bv = NO_BOX;
                for p in t {
                    for i in 0..3 {
                        bv[2 * i] = bv[2 * i].min(p[i]);
                        bv[2 * i + 1] = bv[2 * i + 1].max(p[i]);
                    }
                }
                // (a tree asked for with no margin is given the least one)
                for i in 0..3 {
                    bv[2 * i] -= f32::EPSILON;
                    bv[2 * i + 1] += f32::EPSILON;
                }
                Knot {
                    bv,
                    children: [0; 4],
                    count: 0,
                    axis: 0,
                }
            })
            .collect();
        if leaves == 0 {
            return Kdop { knots, leaves };
        }
        let branches = ((leaves as i64 + FOUR - 3) / (FOUR - 1)).max(1) as usize;
        knots.resize(
            leaves + branches,
            Knot {
                bv: [0.0; 6],
                children: [0; 4],
                count: 0,
                axis: 0,
            },
        );
        // (the leaves as the build sorts them; branch j, counted from one, is knot `leaves - 1 + j`)
        let mut order: Vec<u32> = (0..leaves as u32).collect();
        let around = |knots: &[Knot], order: &[u32]| {
            let mut bv = NO_BOX;
            for &l in order {
                let leaf = &knots[l as usize].bv;
                for i in 0..3 {
                    bv[2 * i] = bv[2 * i].min(leaf[2 * i]);
                    bv[2 * i + 1] = bv[2 * i + 1].max(leaf[2 * i + 1]);
                }
            }
            let (x, y, z) = (bv[1] - bv[0], bv[3] - bv[2], bv[5] - bv[4]);
            let axis = if x > y {
                if x > z { 0 } else { 2 }
            } else if y > z {
                1
            } else {
                2
            };
            (bv, axis)
        };
        if leaves == 1 {
            let (bv, axis) = around(&knots, &order);
            knots[1] = Knot {
                bv,
                children: [0; 4],
                count: 1,
                axis,
            };
            return Kdop { knots, leaves };
        }
        let levels = Levels::new(leaves as i64);
        let (mut i, mut depth) = (1_i64, 1_usize);
        while i <= branches as i64 {
            let next_level = i * FOUR - 2;
            for j in i..next_level.min(branches as i64 + 1) {
                let (begin, end) = (
                    levels.first_leaf(depth, j - i),
                    levels.first_leaf(depth, j - i + 1),
                );
                let (bv, axis) = around(&knots, &order[begin..end]);
                // the leaves dealt to the children, sorted along the box's longest side by where they end
                let child = |k: i64| j * FOUR - 2 + k;
                let mut nth = [begin; 5];
                nth[4] = end;
                for (k, first) in nth.iter_mut().enumerate().take(4).skip(1) {
                    *first = levels.first_leaf(depth + 1, child(k as i64) - next_level);
                }
                let key = |l: u32| knots[l as usize].bv[2 * axis as usize + 1];
                for k in 0..3 {
                    if nth[k] >= nth[4] {
                        break;
                    }
                    nth_element(&mut order, &key, nth[k], nth[4], nth[k + 1]);
                }
                let mut parent = Knot {
                    bv,
                    children: [0; 4],
                    count: 0,
                    axis,
                };
                for k in 0..4 {
                    let at = child(k) - next_level;
                    let (first, last) = (
                        levels.first_leaf(depth + 1, at),
                        levels.first_leaf(depth + 1, at + 1),
                    );
                    if last > first + 1 {
                        parent.children[k as usize] = (leaves as i64 - 1 + child(k)) as u32;
                    } else if last == first + 1 {
                        parent.children[k as usize] = order[first];
                    } else {
                        break;
                    }
                    parent.count += 1;
                }
                knots[leaves - 1 + j as usize] = parent;
            }
            i = next_level;
            depth += 1;
        }
        Kdop { knots, leaves }
    }
}

/// A ray on its way down Blender's tree.
struct Dive<'a> {
    tris: &'a [[V3; 3]],
    origin: V3,
    shear: Shear,
    /// The ray's direction along each axis (zero where it hardly moves), and one over it.
    along: [f32; 3],
    over: [f32; 3],
    /// Which of a box's six bounds the ray meets first and last along each axis.
    bound: [usize; 6],
    hit: Option<u32>,
    dist: f32,
}

impl Dive<'_> {
    /// How far along the ray it enters the box (single precision, as Blender tests it: a box the ray only touches
    /// may be passed by), or f32::MAX if it doesn't within the nearest hit so far.
    fn enters(&self, bv: &[f32; 6]) -> f32 {
        let o = self.origin;
        let t1x = (bv[self.bound[0]] - o.x) * self.over[0];
        let t2x = (bv[self.bound[1]] - o.x) * self.over[0];
        let t1y = (bv[self.bound[2]] - o.y) * self.over[1];
        let t2y = (bv[self.bound[3]] - o.y) * self.over[1];
        let t1z = (bv[self.bound[4]] - o.z) * self.over[2];
        let t2z = (bv[self.bound[5]] - o.z) * self.over[2];
        if (t1x > t2y || t2x < t1y || t1x > t2z || t2x < t1z || t1y > t2z || t2y < t1z)
            || (t2x < 0.0 || t2y < 0.0 || t2z < 0.0)
            || (t1x > self.dist || t1y > self.dist || t1z > self.dist)
        {
            return f32::MAX;
        }
        // (the greatest of the three, as std::max folds them)
        let m = if t1x < t1y { t1y } else { t1x };
        if m < t1z { t1z } else { m }
    }

    fn down(&mut self, tree: &Kdop, at: u32) {
        let knot = &tree.knots[at as usize];
        if self.enters(&knot.bv) >= self.dist {
            return;
        }
        if knot.count == 0 {
            // a nearer triangle only: of two as far as each other, the one met first on the way down stays
            if let Some(dist) = self.shear.hit(self.origin, &self.tris[at as usize])
                && dist >= 0.0
                && dist < self.dist
            {
                (self.hit, self.dist) = (Some(at), dist);
            }
        } else if self.along[knot.axis as usize] > 0.0 {
            for &child in &knot.children[..knot.count as usize] {
                self.down(tree, child);
            }
        } else {
            for &child in knot.children[..knot.count as usize].iter().rev() {
                self.down(tree, child);
            }
        }
    }
}

/// Where a ray met a script's tree.
#[derive(Clone, Copy, Debug)]
pub struct PolyHit {
    pub at: V3,
    /// The normal of the polygon it met.
    pub normal: V3,
    /// Which polygon, among those the tree was built of.
    pub index: u32,
    pub dist: f32,
}

/// `mathutils.bvhtree.BVHTree.FromPolygons(verts, polys, all_triangles=False, epsilon=0.0)`: polygons to cast rays
/// at, cut into triangles and kept in a tree as Blender does both, so that a ray which meets two faces in one
/// plane, an edge or a corner is given the face Blender gives it. That depends on the order the triangles are
/// come to (the tree's boxes, sorted as its build sorts them, the nearer child first by the ray's direction alone),
/// on boxes tested in single precision, and on a hit only replacing a farther one.
pub struct Polygons {
    tris: Vec<[V3; 3]>,
    /// Each triangle's polygon.
    poly: Vec<u32>,
    /// A normal a polygon: a triangle's own, any other's by Newell's sum (none for one of no area).
    normals: Vec<V3>,
    tree: Kdop,
}

impl Polygons {
    pub fn new(verts: &[V3], polys: &[Vec<u32>]) -> Polygons {
        let (mut tris, mut poly) = (Vec::new(), Vec::new());
        let mut normals = Vec::with_capacity(polys.len());
        for (index, corners) in polys.iter().enumerate() {
            let p: Vec<V3> = corners.iter().map(|&v| verts[v as usize]).collect();
            if p.len() < 3 {
                normals.push(V3::ZERO);
                continue;
            }
            if p.len() == 3 {
                normals.push(normal_tri(p[0], p[1], p[2]));
                tris.push([p[0], p[1], p[2]]);
                poly.push(index as u32);
                continue;
            }
            let normal = normalize_v3(crate::mesh::newell(&p)).0;
            normals.push(normal);
            for t in crate::polyfill::ngon(&p, normal) {
                tris.push(t.map(|i| p[i as usize]));
                poly.push(index as u32);
            }
        }
        let tree = Kdop::new(&tris);
        Polygons {
            tris,
            poly,
            normals,
            tree,
        }
    }

    /// `tree.ray_cast(origin, direction, distance)`: the nearest polygon the ray meets within `distance`.
    pub fn ray_cast(&self, origin: V3, direction: V3, distance: f64) -> Option<PolyHit> {
        if self.tris.is_empty() {
            return None;
        }
        let dir = normalize_v3(direction).0;
        let (mut along, mut over, mut bound) = ([0.0_f32; 3], [0.0_f32; 3], [0_usize; 6]);
        for i in 0..3 {
            along[i] = dir[i];
            if along[i].abs() < f32::EPSILON {
                along[i] = 0.0;
                over[i] = f32::MAX;
            } else {
                over[i] = 1.0 / along[i];
            }
            let first = (over[i] < 0.0) as usize;
            bound[2 * i] = 2 * i + first;
            bound[2 * i + 1] = 2 * i + 1 - first;
        }
        let mut dive = Dive {
            tris: &self.tris,
            origin,
            shear: Shear::new(dir),
            along,
            over,
            bound,
            hit: None,
            dist: distance as f32,
        };
        dive.down(&self.tree, self.tree.leaves as u32);
        let t = dive.hit?;
        let index = self.poly[t as usize];
        Some(PolyHit {
            at: V3::new(
                origin.x + dir.x * dive.dist,
                origin.y + dir.y * dive.dist,
                origin.z + dir.z * dive.dist,
            ),
            normal: self.normals[index as usize],
            index,
            dist: dive.dist,
        })
    }
}

/// Where a ray met the scene.
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub at: V3,
    /// The normal of the triangle it met.
    pub normal: V3,
    pub obj: Obj,
    /// The face it met, among the part's.
    pub face: u32,
}

/// The scene's parts as they stand now, to cast rays at (`scene.ray_cast`).
pub struct Rays {
    parts: Vec<(Obj, Bvh, Vec<u32>)>,
}

impl Rays {
    /// Every part in the scene, where it is.
    pub fn of_scene() -> Rays {
        Rays::of(&scene::meshes())
    }

    pub fn of(objs: &[Obj]) -> Rays {
        let parts = objs
            .iter()
            .map(|&o| {
                let points = o.world_points();
                let (tris, faces) = o.read(|p| {
                    let mut tris = Vec::new();
                    let mut faces = Vec::new();
                    for (f, t) in p.mesh.triangles() {
                        let c = p.mesh.corners(f as usize);
                        tris.push([
                            points[c[t[0] as usize] as usize],
                            points[c[t[1] as usize] as usize],
                            points[c[t[2] as usize] as usize],
                        ]);
                        faces.push(f);
                    }
                    (tris, faces)
                });
                (o, Bvh::new(tris), faces)
            })
            .collect();
        Rays { parts }
    }

    /// The first face the ray from `origin` along `direction` meets within `distance`.
    pub fn cast(&self, origin: V3, direction: V3, distance: f64) -> Option<Hit> {
        let distance = distance as f32;
        let direction = normalize_v3(direction).0;
        let mut depth = distance;
        let mut out = None;
        for (obj, bvh, faces) in &self.parts {
            // each part takes the ray into its own frame, and makes it a unit vector again
            // (by division, as the code that casts it does: the kernel's own normalising, as Blender's, multiplies by
            // the reciprocal, which is now and then a unit in the last place off it)
            let squared = crate::math::dot_v3(direction, direction);
            let (dir, scale) = if squared > 1.0e-35 {
                let length = squared.sqrt();
                (
                    V3::new(
                        direction.x / length,
                        direction.y / length,
                        direction.z / length,
                    ),
                    length,
                )
            } else {
                (V3::ZERO, 0.0)
            };
            if let Some((t, dist)) = bvh.ray_cast(origin, dir, depth * scale) {
                let far = dist / scale;
                if far <= depth {
                    depth = far;
                    let tri = &bvh.tris[t as usize];
                    let at = V3::new(
                        origin.x + dir.x * dist,
                        origin.y + dir.y * dist,
                        origin.z + dir.z * dist,
                    );
                    // (the triangle's normal, then made a unit vector once more as the scene hands it back: by division)
                    let n = normal_tri(tri[0], tri[1], tri[2]);
                    let len = crate::math::len_v3(n);
                    let normal = V3::new(n.x / len, n.y / len, n.z / len);
                    out = Some(Hit {
                        at,
                        normal,
                        obj: *obj,
                        face: faces[t as usize],
                    });
                }
            }
        }
        out
    }
}
