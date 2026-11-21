//! How Blender cuts a polygon into triangles: one of five corners or more by clipping ears (BLI_polyfill_2d, after
//! libGDX's EarClippingTriangulator), wherever it does: for the triangles a mesh is drawn, exported and hit by
//! (`ngon`: bke's mesh_tessellate), for the tree a baked shading's rays are cast at (`fill`: mathutils'
//! BVHTree.FromPolygons) and when a script asks it to (`triangulate`: `bmesh.ops.triangulate`, which cuts a quad
//! along the diagonal that leaves the rounder triangles, BLI_polyfill_2d_beautify). Both are Blender's,
//! GPL-2.0-or-later, worked through again here step for step, down to the tree it keeps its hollow corners in: which
//! triangles a cap, a resting cushion's underside or a leaf cut from its outline is drawn with is Blender's choice,
//! ear for ear.

use crate::math::{V3, dot_v3, normalize_v3};

type Sign = i8;
const CONCAVE: Sign = -1;
const TANGENTIAL: Sign = 0;
const CONVEX: Sign = 1;
const UNSET: u32 = u32::MAX;

fn signum(a: f32) -> Sign {
    if a > 0.0 {
        CONVEX
    } else if a == 0.0 {
        TANGENTIAL
    } else {
        CONCAVE
    }
}

/// Which way the corner v2 turns between v1 and v3.
fn span_sign(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2]) -> Sign {
    // (twice the triangle's signed area, taken from v3)
    let d2 = [v2[0] - v3[0], v2[1] - v3[1]];
    let d3 = [v1[0] - v3[0], v1[1] - v3[1]];
    signum((d2[0] * d3[1]) - (d3[0] * d2[1]))
}

/// A triangle's sides, for telling whether a point lies in it.
struct Sides {
    edge: [[f32; 2]; 3],
    c: [f32; 3],
}

impl Sides {
    fn of(vs: [[f32; 2]; 3]) -> Sides {
        let sub = |a: [f32; 2], b: [f32; 2]| [a[0] - b[0], a[1] - b[1]];
        let edge = [sub(vs[1], vs[0]), sub(vs[2], vs[1]), sub(vs[0], vs[2])];
        let c = std::array::from_fn(|i| (edge[i][0] * vs[i][1]) - (vs[i][0] * edge[i][1]));
        Sides { edge, c }
    }

    /// Whether `co` is in the triangle (on its edge counts).
    fn holds(&self, co: [f32; 2]) -> bool {
        let side =
            |i: usize| (self.edge[i][1] * co[0]) - (self.edge[i][0] * co[1]) + self.c[i] >= 0.0;
        side(1) && side(2) && side(0)
    }
}

/// A node of the tree the polygon's hollow corners are kept in.
#[derive(Clone, Copy)]
struct Node {
    neg: u32,
    pos: u32,
    index: u32,
    axis: u8,
    removed: bool,
    parent: u32,
}

/// The corners that aren't convex, in a tree split by x and y in turn: an ear is one only if none of them lies
/// in it.
struct Tree<'a> {
    nodes: Vec<Node>,
    coords: &'a [[f32; 2]],
    root: u32,
    /// A corner's node, or none once it is out of the tree.
    map: Vec<u32>,
}

/// Sorts `nodes` round their median along `axis` and the halves either side of it along the other: the
/// median's place in the whole array.
fn balance(nodes: &mut [Node], axis: u8, coords: &[[f32; 2]], ofs: u32) -> u32 {
    let node_num = nodes.len();
    if node_num == 0 {
        return UNSET;
    }
    if node_num == 1 {
        return ofs;
    }
    let at = |nodes: &[Node], i: usize| coords[nodes[i].index as usize][axis as usize];
    let (mut neg, mut pos) = (0usize, node_num - 1);
    let median = node_num / 2;
    while pos > neg {
        let co = at(nodes, pos);
        let mut i = neg.wrapping_sub(1);
        let mut j = pos;
        loop {
            loop {
                i = i.wrapping_add(1);
                if at(nodes, i) >= co || at(nodes, i).is_nan() {
                    break;
                }
            }
            loop {
                j -= 1;
                if !(at(nodes, j) > co && j > neg) {
                    break;
                }
            }
            if i >= j {
                break;
            }
            nodes.swap(i, j);
        }
        nodes.swap(i, pos);
        if i >= median {
            pos = i - 1;
        }
        if i <= median {
            neg = i + 1;
        }
    }
    nodes[median].axis = axis;
    let (left, right) = nodes.split_at_mut(median);
    let neg = balance(left, axis ^ 1, coords, ofs);
    let pos = balance(&mut right[1..], axis ^ 1, coords, median as u32 + 1 + ofs);
    (right[0].neg, right[0].pos) = (neg, pos);
    median as u32 + ofs
}

impl<'a> Tree<'a> {
    /// The tree of the corners whose `sign` isn't convex.
    fn new(coords: &'a [[f32; 2]], sign: &[Sign]) -> Tree<'a> {
        let mut nodes: Vec<Node> = (0..coords.len())
            .filter(|&i| sign[i] != CONVEX)
            .map(|i| Node {
                neg: UNSET,
                pos: UNSET,
                index: i as u32,
                axis: 0,
                removed: false,
                parent: UNSET,
            })
            .collect();
        let root = balance(&mut nodes, 0, coords, 0);
        let mut map = vec![UNSET; coords.len()];
        for i in 0..nodes.len() {
            let node = nodes[i];
            if node.neg != UNSET {
                nodes[node.neg as usize].parent = i as u32;
            }
            if node.pos != UNSET {
                nodes[node.pos as usize].parent = i as u32;
            }
            map[node.index as usize] = i as u32;
        }
        nodes[root as usize].parent = UNSET;
        Tree {
            nodes,
            coords,
            root,
            map,
        }
    }

    /// Takes the corner `index` out: a node left with one branch or none gives its place to that branch.
    fn remove(&mut self, index: u32) {
        let mut node_index = self.map[index as usize];
        if node_index == UNSET {
            return;
        }
        self.map[index as usize] = UNSET;
        self.nodes[node_index as usize].removed = true;
        loop {
            let node = self.nodes[node_index as usize];
            if node.parent == UNSET {
                break;
            }
            let child = if node.neg == UNSET {
                node.pos
            } else if node.pos == UNSET {
                node.neg
            } else {
                break; // both branches: nothing to close up
            };
            let parent = &mut self.nodes[node.parent as usize];
            if parent.neg == node_index {
                parent.neg = child;
            } else {
                parent.pos = child;
            }
            let parent_removed = parent.removed;
            if child != UNSET {
                self.nodes[child as usize].parent = node.parent;
            }
            if !parent_removed {
                break;
            }
            node_index = node.parent;
        }
    }

    fn within(
        &self,
        tri: [u32; 3],
        sides: &Sides,
        centre: [f32; 2],
        bounds: [(f32, f32); 2],
        at: u32,
    ) -> u32 {
        let node = self.nodes[at as usize];
        let co = self.coords[node.index as usize];
        if !node.removed && sides.holds(co) && !tri.contains(&node.index) {
            return node.index;
        }
        let axis = node.axis as usize;
        let neg = || {
            if node.neg != UNSET && co[axis] >= bounds[axis].0 {
                self.within(tri, sides, centre, bounds, node.neg)
            } else {
                UNSET
            }
        };
        let pos = || {
            if node.pos != UNSET && co[axis] <= bounds[axis].1 {
                self.within(tri, sides, centre, bounds, node.pos)
            } else {
                UNSET
            }
        };
        if centre[axis] > co[axis] {
            match pos() {
                UNSET => neg(),
                hit => hit,
            }
        } else {
            match neg() {
                UNSET => pos(),
                hit => hit,
            }
        }
    }

    /// A corner of the tree that lies in the triangle `tri` (and isn't one of its own), or none.
    fn in_triangle(&self, tri: [u32; 3]) -> u32 {
        let vs = tri.map(|i| self.coords[i as usize]);
        let span = |k: usize| {
            (
                vs[0][k].min(vs[1][k]).min(vs[2][k]),
                vs[0][k].max(vs[1][k]).max(vs[2][k]),
            )
        };
        let centre = [
            (vs[0][0] + vs[1][0] + vs[2][0]) * (1.0 / 3.0),
            (vs[0][1] + vs[1][1] + vs[2][1]) * (1.0 / 3.0),
        ];
        self.within(tri, &Sides::of(vs), centre, [span(0), span(1)], self.root)
    }

    /// Whether the corner `test` is still in the tree and lies in the triangle `tri`.
    fn single_in_triangle(&self, tri: [u32; 3], test: u32) -> bool {
        self.map[test as usize] != UNSET
            && !tri.contains(&test)
            && Sides::of(tri.map(|i| self.coords[i as usize])).holds(self.coords[test as usize])
    }
}

/// The polygon as the clipping goes: its corners left, each with its neighbours and which way it turns.
struct Fill<'a> {
    coords: &'a [[f32; 2]],
    next: Vec<u32>,
    prev: Vec<u32>,
    sign: Vec<Sign>,
    /// The corner that was last found in a corner's ear, and whether its ear has changed since.
    last_hit: Vec<u32>,
    dirty: Vec<bool>,
    /// The corner the polygon is held by.
    first: u32,
    left: usize,
    concave: usize,
    tree: Option<Tree<'a>>,
}

impl Fill<'_> {
    fn sign_of(&self, i: u32) -> Sign {
        let co = |k: u32| self.coords[k as usize];
        span_sign(co(self.prev[i as usize]), co(i), co(self.next[i as usize]))
    }

    /// Whether the corner `ear` can be clipped: it turns the way asked and no hollow corner lies in its triangle.
    fn is_ear(&mut self, ear: u32, accept: Sign) -> bool {
        // (a polygon with no hollow corner left: any corner will do)
        if self.concave == 0 {
            return true;
        }
        let e = ear as usize;
        if self.sign[e] != accept {
            return false;
        }
        let Some(tree) = &self.tree else {
            return true;
        };
        let tri = [ear, self.next[e], self.prev[e]];
        let cached = self.last_hit[e];
        if cached != UNSET {
            if !self.dirty[e] {
                if tree.map[cached as usize] != UNSET {
                    return false;
                }
            } else if tree.single_in_triangle(tri, cached) {
                self.dirty[e] = false;
                return false;
            }
        }
        let hit = tree.in_triangle(tri);
        if hit != UNSET {
            self.last_hit[e] = hit;
            self.dirty[e] = false;
            return false;
        }
        true
    }

    /// The next ear from `init` on, round the polygon one way or the other: a convex one, else one of no area,
    /// else whatever corner isn't hollow.
    fn find_ear(&mut self, init: u32, reverse: bool) -> u32 {
        for accept in [CONVEX, TANGENTIAL] {
            let mut ear = init;
            for _ in 0..self.left {
                if self.is_ear(ear, accept) {
                    return ear;
                }
                ear = if reverse {
                    self.prev[ear as usize]
                } else {
                    self.next[ear as usize]
                };
            }
        }
        let mut ear = init;
        for _ in 0..self.left {
            if self.sign[ear as usize] != CONCAVE {
                return ear;
            }
            ear = self.next[ear as usize];
        }
        ear
    }

    fn remove(&mut self, i: u32) {
        if let Some(tree) = &mut self.tree {
            tree.remove(i);
        }
        let (p, n) = (self.prev[i as usize], self.next[i as usize]);
        self.prev[n as usize] = p;
        self.next[p as usize] = n;
        if self.first == i {
            self.first = n;
        }
        self.left -= 1;
    }

    /// A neighbour of a clipped ear that wasn't convex may be now.
    fn turned(&mut self, i: u32) {
        if self.sign[i as usize] != CONVEX {
            self.sign[i as usize] = self.sign_of(i);
            if self.sign[i as usize] == CONVEX {
                self.concave -= 1;
                if let Some(tree) = &mut self.tree {
                    tree.remove(i);
                }
            }
        }
    }
}

/// The triangles of the flat polygon `coords` (clockwise), as places among its corners, in the order Blender
/// clips them: an ear, then the next but one round, turning back where the corner there is hollow.
pub(crate) fn fill(coords: &[[f32; 2]]) -> Vec<[u32; 3]> {
    let n = coords.len();
    let mut f = Fill {
        coords,
        next: (0..n).map(|i| ((i + 1) % n) as u32).collect(),
        prev: (0..n).map(|i| ((i + n - 1) % n) as u32).collect(),
        sign: vec![CONVEX; n],
        last_hit: vec![UNSET; n],
        dirty: vec![false; n],
        first: 0,
        left: n,
        concave: 0,
        tree: None,
    };
    for i in 0..n {
        f.sign[i] = f.sign_of(i as u32);
        f.concave += (f.sign[i] != CONVEX) as usize;
    }
    if f.concave != 0 {
        f.tree = Some(Tree::new(coords, &f.sign));
    }
    let mut tris = Vec::with_capacity(n.saturating_sub(2));
    let mut init = f.first;
    let mut reverse = false;
    while f.left > 3 {
        let ear = f.find_ear(init, reverse);
        if f.sign[ear as usize] != CONVEX {
            f.concave -= 1;
        }
        let (prev, next) = (f.prev[ear as usize], f.next[ear as usize]);
        tris.push([prev, ear, next]);
        f.remove(ear);
        f.dirty[prev as usize] = true;
        f.dirty[next as usize] = true;
        f.turned(prev);
        f.turned(next);
        init = if reverse {
            f.prev[prev as usize]
        } else {
            f.next[next as usize]
        };
        if f.sign[init as usize] != CONVEX {
            // (the extra step: this one isn't a good ear to try first)
            init = if reverse {
                f.prev[init as usize]
            } else {
                f.next[init as usize]
            };
            reverse = !reverse;
        }
    }
    if f.left == 3 {
        let a = f.first;
        let b = f.next[a as usize];
        tris.push([a, b, f.next[b as usize]]);
    }
    tris
}

/// Two vectors square to the unit vector `n` and to each other: the plane a face of that normal is flattened onto.
fn ortho_basis(n: V3) -> (V3, V3) {
    let f = n.x * n.x + n.y * n.y;
    if f > f32::EPSILON {
        let d = 1.0 / f.sqrt();
        let n1 = V3::new(n.y * d, -n.x * d, 0.0);
        let n2 = V3::new(-n.z * n1.y, n.z * n1.x, n.x * n1.y - n.y * n1.x);
        (n1, n2)
    } else {
        (
            V3::new(if n.z < 0.0 { -1.0 } else { 1.0 }, 0.0, 0.0),
            V3::new(0.0, 1.0, 0.0),
        )
    }
}

fn flat(p: V3, (n1, n2): (V3, V3)) -> [f32; 2] {
    [dot_v3(n1, p), dot_v3(n2, p)]
}

fn cross_tri_v2(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2]) -> f32 {
    (v1[0] - v2[0]) * (v2[1] - v3[1]) + (v1[1] - v2[1]) * (v3[0] - v2[0])
}

fn cross_tri_v3(v1: V3, v2: V3, v3: V3) -> V3 {
    (v1 - v2).cross(v2 - v3)
}

fn len_v2v2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (x, y) = (a[0] - b[0], a[1] - b[1]);
    (x * x + y * y).sqrt()
}

/// What turning the cut 2-4 of the flat quad 1, 2, 3, 4 into 1-3 would gain: under zero if the two triangles
/// either side of 1-3 are rounder (area over perimeter) than those either side of 2-4.
fn turn_gain(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2], v4: [f32; 2]) -> f32 {
    // (very small faces still count as having an area)
    let eps_zero_area = 1e-12_f32;
    let area_2x_234 = cross_tri_v2(v2, v3, v4);
    let area_2x_241 = cross_tri_v2(v2, v4, v1);
    let area_2x_123 = cross_tri_v2(v1, v2, v3);
    let area_2x_134 = cross_tri_v2(v1, v3, v4);
    // 1-3 can't be cut: its triangles would face opposite ways, or one would have no area
    if (area_2x_123 >= 0.0) != (area_2x_134 >= 0.0) {
        return f32::MAX;
    }
    if area_2x_123.abs() <= eps_zero_area || area_2x_134.abs() <= eps_zero_area {
        return f32::MAX;
    }
    // 2-4 can't: always turn
    if (area_2x_234 >= 0.0) != (area_2x_241 >= 0.0) {
        return -f32::MAX;
    }
    if area_2x_234.abs() <= eps_zero_area || area_2x_241.abs() <= eps_zero_area {
        return -f32::MAX;
    }
    let (len_12, len_23, len_34, len_41) = (
        len_v2v2(v1, v2),
        len_v2v2(v2, v3),
        len_v2v2(v3, v4),
        len_v2v2(v4, v1),
    );
    let (len_13, len_24) = (len_v2v2(v1, v3), len_v2v2(v2, v4));
    let fac_24 = (area_2x_234.abs() / (len_23 + len_34 + len_24))
        + (area_2x_241.abs() / (len_41 + len_12 + len_24));
    let fac_13 = (area_2x_123.abs() / (len_12 + len_23 + len_13))
        + (area_2x_134.abs() / (len_34 + len_41 + len_13));
    fac_24 - fac_13
}

/// `turn_gain` for a quad in space, flattened onto the plane of its two triangles' mean.
fn turn_gain_v3(v1: V3, v2: V3, v3: V3, v4: V3) -> f32 {
    let no = cross_tri_v3(v2, v3, v4) + cross_tri_v3(v2, v4, v1);
    let (no, no_scale) = normalize_v3(no);
    if no_scale == 0.0 {
        return f32::MAX;
    }
    let basis = ortho_basis(no);
    let (a, b, c, d) = (
        flat(v1, basis),
        flat(v2, basis),
        flat(v3, basis),
        flat(v4, basis),
    );
    // (triangles that already face opposite ways, or both of no area: left alone)
    let sign = |x: f32| {
        if x > 1e-5 {
            1
        } else if x < -1e-5 {
            -1
        } else {
            0
        }
    };
    if sign(cross_tri_v2(b, c, d) / no_scale) + sign(cross_tri_v2(b, d, a) / no_scale) == 0 {
        return f32::MAX;
    }
    turn_gain(a, b, c, d)
}

/// The triangles Blender's `triangulate` (quads by beauty, the others by clipping ears) makes of the face of
/// corners `p` and `normal`, as places among its corners: each from the corner Blender's starts at, in the order
/// it makes them (the last one takes the face's own place in the mesh).
pub(crate) fn triangulate(p: &[V3], normal: V3) -> Vec<[u32; 3]> {
    match p.len() {
        0..=2 => Vec::new(),
        3 => vec![[0, 1, 2]],
        4 => {
            let (v1, v2, v3, v4) = (p[1], p[2], p[3], p[0]);
            let (d_12, d_23, d_34, d_41) = (v1 - v2, v2 - v3, v3 - v4, v4 - v1);
            // a quad hollow at a corner is cut from that corner; else the cut that leaves the rounder triangles
            let split_24 = if dot_v3(d_12.cross(d_23), d_34.cross(d_41)) < 0.0 {
                true
            } else if dot_v3(d_23.cross(d_34), d_41.cross(d_12)) < 0.0 {
                false
            } else {
                turn_gain_v3(v1, v2, v3, v4) > 0.0
            };
            if split_24 {
                vec![[0, 1, 2], [0, 2, 3]]
            } else {
                vec![[1, 2, 3], [1, 3, 0]]
            }
        }
        _ => ngon(p, normal),
    }
}

/// The triangles of the polygon of corners `p` (five or more) and unit `normal`, as places among its corners, in
/// the order Blender clips them: the polygon flattened onto the plane square to its normal, seen from behind (so it
/// runs clockwise: axis_dominant_v3_to_m3_negate, in single precision), then its ears. A polygon that isn't flat is
/// cut as its shadow on that plane is, and corners in a line with their neighbours are clipped last, by Blender's
/// own tests on the flattened points: nothing is done to the triangles afterwards.
pub(crate) fn ngon(p: &[V3], normal: V3) -> Vec<[u32; 3]> {
    let basis = ortho_basis(-normal);
    let coords: Vec<[f32; 2]> = p.iter().map(|&v| flat(v, basis)).collect();
    fill(&coords)
}
