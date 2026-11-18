//! Triangles of a polygon of five corners or more (a cylinder's cap, a swept profile's end, the underside of a
//! cushion that rests on it): the ones Blender's mesh is drawn, exported and hit by, its ears clipped as Blender
//! clips them (`polyfill`), every other corner round a polygon with no hollow.
//! And the triangles that fill outlines with holes (`fill`: a slab's two faces), as Blender's `triangle_fill`
//! cuts them: its sweep (BLI_scanfill) and its beauty pass (BLI_polyfill_beautify, bmesh_beautify), GPL-2.0-or-later,
//! worked through again here.

use crate::math::V3;
use std::collections::{BTreeMap, BTreeSet};

/// The triangles of the polygon `p` (five corners or more; `normal`: its unit normal by Newell's sum,
/// `mesh::normal_newell`), as places among its corners: the ones Blender's mesh has for it (`polyfill::ngon`).
pub fn polygon(p: &[V3], normal: V3) -> Vec<[u32; 3]> {
    crate::polyfill::ngon(p, normal)
}

// ---- outlines filled as Blender fills them --------------------------------------------------------------

/// The triangles that fill the closed outlines `loops` (flat, their points numbered on through them), as
/// Blender's `triangle_fill` with `use_beauty` leaves them: a sweep from the top that clips at each point in turn,
/// then its edges turned wherever that makes the two triangles either side rounder (their area over their
/// perimeter), the best turn first. An outline inside another is filled across, as Blender does: `forms::extrude`
/// takes out what lies in a hole. The triangles come as the sweep wound them, in the order Blender's mesh keeps them
/// and each from the corner Blender's starts at: its extrusion numbers the copy's points and edges by them.
///
/// The cuts decide how a bevel rounds the slab's rim (a point slides along the one cut at its corner, and the cut
/// faces say how far a bevel has room to go), so they are Blender's, cut for cut.
pub fn fill(loops: &[Vec<[f32; 2]>]) -> Vec<[u32; 3]> {
    let mut xy: Vec<[f32; 2]> = Vec::new();
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for l in loops {
        let at = xy.len();
        xy.extend_from_slice(l);
        edges.extend((0..l.len()).map(|i| (at + i, at + (i + 1) % l.len())));
    }
    // (Blender sweeps the outlines seen from below: mirrored)
    let mirrored: Vec<[f32; 2]> = xy.iter().map(|p| [-p[0], p[1]]).collect();
    let mut tris = Vec::new();
    for (verts, own) in polys(&mirrored, &edges) {
        sweep(&mirrored, &verts, &own, &mut tris);
    }
    let outline: BTreeSet<(u32, u32)> = edges
        .iter()
        .map(|&(a, b)| key(a as u32, b as u32))
        .collect();
    beautify(&xy, &mut tris, &outline);
    tris
}

fn key(a: u32, b: u32) -> (u32, u32) {
    if a < b { (a, b) } else { (b, a) }
}

/// The points and the edges of outlines filled as one.
type Group = (Vec<usize>, Vec<(usize, usize)>);

/// The outlines grouped as the fill takes them: those whose boxes touch are filled as one (so one inside
/// another is), each group its points and its edges in the order they were given.
fn polys(xy: &[[f32; 2]], edges: &[(usize, usize)]) -> Vec<Group> {
    // which outline each point is of: joined through the edges
    let mut poly = vec![usize::MAX; xy.len()];
    let mut count = 0;
    for v in 0..xy.len() {
        if poly[v] != usize::MAX {
            continue;
        }
        poly[v] = count;
        let mut grew = true;
        while grew {
            grew = false;
            for &(a, b) in edges {
                if poly[a] == usize::MAX && poly[b] == count {
                    poly[a] = count;
                    grew = true;
                } else if poly[b] == usize::MAX && poly[a] == count {
                    poly[b] = count;
                    grew = true;
                }
            }
        }
        count += 1;
    }
    struct Box2 {
        nr: usize,
        min: [f32; 2],
        max: [f32; 2],
        edges: usize,
    }
    let mut boxes: Vec<Box2> = (0..count)
        .map(|nr| Box2 {
            nr,
            min: [1.0e20; 2],
            max: [-1.0e20; 2],
            edges: 0,
        })
        .collect();
    for &(a, _) in edges {
        boxes[poly[a]].edges += 1;
    }
    for (v, p) in xy.iter().enumerate() {
        let b = &mut boxes[poly[v]];
        for i in 0..2 {
            b.min[i] = b.min[i].min(p[i]);
            b.max[i] = b.max[i].max(p[i]);
        }
    }
    boxes.sort_by(|a, b| a.min[0].total_cmp(&b.min[0]));
    for a in 0..count {
        let mut touched = Vec::new();
        for c in a + 1..count {
            let (pa, pc) = (&boxes[a], &boxes[c]);
            if pa.edges == 0 || pc.edges == 0 {
                continue;
            }
            if pa.max[0] < pc.min[0]
                || pa.max[1] < pc.min[1]
                || pa.min[0] > pc.max[0]
                || pa.min[1] > pc.max[1]
            {
                continue;
            }
            let (min, max) = (pc.min, pc.max);
            let pa = &mut boxes[a];
            for i in 0..2 {
                pa.max[i] = pa.max[i].max(max[i]);
                pa.min[i] = pa.min[i].min(min[i]);
            }
            touched.push(c);
        }
        for c in touched.into_iter().rev() {
            let (from, to) = (boxes[c].nr, boxes[a].nr);
            poly.iter_mut()
                .filter(|p| **p == from)
                .for_each(|p| *p = to);
            boxes[a].edges += boxes[c].edges;
            boxes[c].edges = 0;
        }
    }
    boxes
        .iter()
        .filter(|b| b.edges > 1)
        .map(|b| {
            (
                (0..xy.len()).filter(|&v| poly[v] == b.nr).collect(),
                edges
                    .iter()
                    .copied()
                    .filter(|&(a, _)| poly[a] == b.nr)
                    .collect(),
            )
        })
        .collect()
}

/// Whether `p` is to the right of the line from `a` to `b` (on it counts, but for its two ends).
fn right_of(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> bool {
    let inp = (b[0] - a[0]) * (a[1] - p[1]) + (a[1] - b[1]) * (a[0] - p[0]);
    if inp < 0.0 {
        return false;
    }
    !(inp == 0.0 && (a == p || b == p))
}

/// The sweep: the points from the top down (left first), each with the edges that hang from it, left to right;
/// at each point its two leftmost edges are closed into a triangle, unless a point lies in it, which is joined
/// to first.
fn sweep(xy: &[[f32; 2]], verts: &[usize], own: &[(usize, usize)], tris: &mut Vec<[u32; 3]>) {
    #[derive(Clone, Copy)]
    struct Edge {
        v1: usize,
        v2: usize,
        inner: bool,
    }
    let mut order: Vec<usize> = verts.to_vec();
    order.sort_by(|&a, &b| {
        xy[b][1]
            .total_cmp(&xy[a][1])
            .then(xy[a][0].total_cmp(&xy[b][0]))
    });
    let mut place = vec![usize::MAX; xy.len()];
    for (i, &v) in order.iter().enumerate() {
        place[v] = i;
    }
    let mut edge_count = vec![0_i32; xy.len()];
    for &(a, b) in own {
        edge_count[a] += 1;
        edge_count[b] += 1;
    }
    let two_connected = verts.iter().all(|&v| edge_count[v] <= 2);
    let mut store: Vec<Edge> = Vec::new();
    let mut lists: Vec<Vec<usize>> = vec![Vec::new(); order.len()];
    // an edge hangs from its upper end (its left one, if level), among that point's by how far left it leans;
    // false if the point has it already
    let slope = |from: [f32; 2], to: [f32; 2]| {
        let dy = to[1] - from[1];
        if dy == 0.0 {
            1.0e10 * (to[0] - from[0])
        } else {
            (from[0] - to[0]) / dy
        }
    };
    let add = |store: &mut Vec<Edge>,
               lists: &mut Vec<Vec<usize>>,
               a: usize,
               b: usize,
               inner: bool|
     -> Result<usize, usize> {
        let (mut v1, mut v2) = (a, b);
        if xy[v1][1] == xy[v2][1] {
            if xy[v1][0] > xy[v2][0] {
                std::mem::swap(&mut v1, &mut v2);
            }
        } else if xy[v1][1] < xy[v2][1] {
            std::mem::swap(&mut v1, &mut v2);
        }
        let list = &mut lists[place[v1]];
        let fac1 = slope(xy[v1], xy[v2]);
        let mut at = list.len();
        for (i, &e) in list.iter().enumerate() {
            if store[e].v2 == v2 {
                return Err(place[v1]);
            }
            if slope(xy[v1], xy[store[e].v2]) > fac1 {
                at = i;
                break;
            }
        }
        store.push(Edge { v1, v2, inner });
        list.insert(at, store.len() - 1);
        Ok(store.len() - 1)
    };
    for &(a, b) in own {
        if a != b {
            let _ = add(&mut store, &mut lists, a, b, false);
        }
    }
    let mut available = vec![false; xy.len()];
    let most = 2 * order.len();
    let mut made = 0;
    'points: for a in 0..order.len() {
        // (an edge to a loose end goes; the others' far ends are in hand)
        let mut i = 0;
        while i < lists[a].len() {
            let e = store[lists[a][i]];
            if edge_count[e.v1] == 1 || edge_count[e.v2] == 1 {
                lists[a].remove(i);
                for v in [e.v1, e.v2] {
                    if edge_count[v] > 1 {
                        edge_count[v] -= 1;
                    }
                }
            } else {
                available[e.v2] = true;
                i += 1;
            }
        }
        while !lists[a].is_empty() {
            if made >= most {
                break 'points;
            }
            let ed1 = lists[a][0];
            if lists[a].len() == 1 {
                let e = store[ed1];
                lists[a].clear();
                available[e.v2] = false;
                edge_count[e.v1] -= 1;
                edge_count[e.v2] -= 1;
            } else {
                let ed2 = lists[a][1];
                let (v1, v2, v3) = (store[ed1].v2, store[ed1].v1, store[ed2].v2);
                if v1 == v2 || v2 == v3 {
                    break;
                }
                // a point inside the triangle: the one that makes the sharpest corner with the first edge
                let miny = xy[v1][1].min(xy[v3][1]);
                let mut best: Option<usize> = None;
                let mut best_cos = -1.0_f32;
                let mut first_time = false;
                for &p in &order[a + 1..] {
                    if available[p] {
                        continue;
                    }
                    if xy[p][1] <= miny {
                        break;
                    }
                    if right_of(xy[v1], xy[v2], xy[p])
                        && right_of(xy[v2], xy[v3], xy[p])
                        && right_of(xy[v3], xy[v1], xy[p])
                    {
                        match best {
                            None => best = Some(p),
                            Some(b) => {
                                if !first_time {
                                    best_cos = cos_at(xy[v2], xy[v1], xy[b]);
                                    first_time = true;
                                }
                                let cos = cos_at(xy[v2], xy[v1], xy[p]);
                                if cos > best_cos {
                                    best = Some(p);
                                    best_cos = cos;
                                }
                            }
                        }
                    }
                }
                if let Some(p) = best {
                    // an edge to it, before the second, and again
                    store.push(Edge {
                        v1: v2,
                        v2: p,
                        inner: true,
                    });
                    lists[a].insert(1, store.len() - 1);
                    available[p] = true;
                    edge_count[v2] += 1;
                    edge_count[p] += 1;
                } else {
                    tris.push([v1 as u32, v2 as u32, v3 as u32]);
                    made += 1;
                    lists[a].remove(0);
                    available[store[ed1].v2] = false;
                    edge_count[store[ed1].v1] -= 1;
                    edge_count[store[ed1].v2] -= 1;
                    // (the second edge goes too if it is the outline's)
                    if !store[ed2].inner && two_connected {
                        lists[a].remove(0);
                        available[store[ed2].v2] = false;
                        edge_count[store[ed2].v1] -= 1;
                        edge_count[store[ed2].v2] -= 1;
                    }
                    // the edge that closes it
                    edge_count[v1] += 1;
                    edge_count[v3] += 1;
                    if let Err(at) = add(&mut store, &mut lists, v1, v3, true) {
                        // there already: the outline is closed there
                        edge_count[v1] -= 1;
                        edge_count[v3] -= 1;
                        if let Some(i) = lists[at].iter().position(|&e| {
                            (store[e].v1 == v1 && store[e].v2 == v3)
                                || (store[e].v1 == v3 && store[e].v2 == v1)
                        }) && two_connected
                        {
                            lists[at].remove(i);
                            edge_count[v1] -= 1;
                            edge_count[v3] -= 1;
                        }
                    }
                }
            }
            // loose edges go
            let mut i = 0;
            while i < lists[a].len() {
                let e = store[lists[a][i]];
                if edge_count[e.v1] < 2 || edge_count[e.v2] < 2 {
                    lists[a].remove(i);
                    for v in [e.v1, e.v2] {
                        if edge_count[v] > 1 {
                            edge_count[v] -= 1;
                        }
                    }
                } else {
                    i += 1;
                }
            }
        }
    }
}

/// The cosine of the angle at `b` between `a` and `c`.
fn cos_at(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    let unit = |p: [f32; 2]| {
        let d = [p[0] - b[0], p[1] - b[1]];
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        if len > 1.0e-35 {
            [d[0] / len, d[1] / len]
        } else {
            [0.0, 0.0]
        }
    };
    let (u, v) = (unit(a), unit(c));
    u[0] * v[0] + u[1] * v[1]
}

fn cross_tri(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (a[0] - b[0]) * (b[1] - c[1]) + (a[1] - b[1]) * (c[0] - b[0])
}

fn len2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (x, y) = (b[0] - a[0], b[1] - a[1]);
    (x * x + y * y).sqrt()
}

/// What turning the edge 2-4 of the quad 1, 2, 3, 4 into 1-3 would gain: under zero if the two new triangles
/// are rounder (the sum of area over perimeter) than the two there.
fn turn_gain(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2], v4: [f32; 2]) -> f32 {
    let eps_zero_area = 1.0e-12_f32;
    let area_234 = cross_tri(v2, v3, v4);
    let area_241 = cross_tri(v2, v4, v1);
    let area_123 = cross_tri(v1, v2, v3);
    let area_134 = cross_tri(v1, v3, v4);
    // (1-3) would fold the quad over, or leave a triangle of no area
    if (area_123 >= 0.0) != (area_134 >= 0.0) {
        return f32::MAX;
    }
    if area_123.abs() <= eps_zero_area || area_134.abs() <= eps_zero_area {
        return f32::MAX;
    }
    // (2-4) is folded or flat itself: turn it
    if (area_234 >= 0.0) != (area_241 >= 0.0) {
        return -f32::MAX;
    }
    if area_234.abs() <= eps_zero_area || area_241.abs() <= eps_zero_area {
        return -f32::MAX;
    }
    let (len_12, len_23, len_34, len_41) = (len2(v1, v2), len2(v2, v3), len2(v3, v4), len2(v4, v1));
    let (len_13, len_24) = (len2(v1, v3), len2(v2, v4));
    let prim_a = len_23 + len_34 + len_24;
    let prim_b = len_41 + len_12 + len_24;
    let fac_24 = (area_234 / prim_a) + (area_241 / prim_b);
    let prim_c = len_12 + len_23 + len_13;
    let prim_d = len_34 + len_41 + len_13;
    let fac_13 = (area_123 / prim_c) + (area_134 / prim_d);
    fac_24 - fac_13
}

/// Blender's heap: the least first; of equals, as its sifting leaves them.
#[derive(Default)]
struct Heap {
    tree: Vec<(f32, usize)>,
    /// Where each edge's node is in the tree.
    at: BTreeMap<usize, usize>,
}

impl Heap {
    fn swap(&mut self, i: usize, j: usize) {
        self.tree.swap(i, j);
        self.at.insert(self.tree[i].1, i);
        self.at.insert(self.tree[j].1, j);
    }
    fn up(&mut self, mut i: usize) {
        while i > 0 {
            let p = (i - 1) >> 1;
            if self.tree[p].0 < self.tree[i].0 {
                break;
            }
            self.swap(p, i);
            i = p;
        }
    }
    fn down(&mut self, mut i: usize) {
        let size = self.tree.len();
        loop {
            let (l, r) = ((i << 1) + 1, (i << 1) + 2);
            let mut smallest = i;
            if l < size && self.tree[l].0 < self.tree[smallest].0 {
                smallest = l;
            }
            if r < size && self.tree[r].0 < self.tree[smallest].0 {
                smallest = r;
            }
            if smallest == i {
                break;
            }
            self.swap(i, smallest);
            i = smallest;
        }
    }
    fn insert(&mut self, value: f32, edge: usize) {
        self.tree.push((value, edge));
        let i = self.tree.len() - 1;
        self.at.insert(edge, i);
        self.up(i);
    }
    fn pop(&mut self) -> Option<usize> {
        let last = self.tree.len().checked_sub(1)?;
        let edge = self.tree[0].1;
        if last > 0 {
            self.swap(0, last);
        }
        self.tree.pop();
        self.at.remove(&edge);
        if !self.tree.is_empty() {
            self.down(0);
        }
        Some(edge)
    }
    fn remove(&mut self, edge: usize) {
        let Some(&at) = self.at.get(&edge) else {
            return;
        };
        let mut i = at;
        while i > 0 {
            let p = (i - 1) >> 1;
            self.swap(p, i);
            i = p;
        }
        self.pop();
    }
}

/// Turns the inner edges of `tris` (all but the `outline`'s) for rounder triangles, the best turn first, as
/// Blender's `beautify_fill` does. The triangles stay in the order Blender's mesh has them in after it, each from
/// the corner Blender's starts at: a turn joins the two triangles into a face that takes the place left free by
/// the turn before (a new one at the end, the first time) and splits the other off it into the older one's place,
/// and the newer one's is left free.
fn beautify(xy: &[[f32; 2]], tris: &mut Vec<[u32; 3]>, outline: &BTreeSet<(u32, u32)>) {
    let mut free: Option<usize> = None;
    // each edge's triangles, the newer last (the one Blender measures the quad from)
    let mut on: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    let mut inner: Vec<(u32, u32)> = Vec::new();
    let mut index: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (t, tri) in tris.iter().enumerate() {
        for (a, b) in [(tri[2], tri[0]), (tri[0], tri[1]), (tri[1], tri[2])] {
            let k = key(a, b);
            let faces = on.entry(k).or_default();
            if faces.is_empty() && !outline.contains(&k) {
                index.insert(k, inner.len());
                inner.push(k);
            }
            faces.push(t);
        }
    }
    // the quad about edge i: 2 and 4 its ends as the newer triangle runs it, 1 that triangle's other corner,
    // 3 the older one's
    let quad = |tris: &[[u32; 3]],
                on: &BTreeMap<(u32, u32), Vec<usize>>,
                e: (u32, u32)|
     -> Option<[u32; 4]> {
        let faces = &on[&e];
        if faces.len() != 2 {
            return None;
        }
        let (newer, older) = (tris[faces[1]], tris[faces[0]]);
        let i = (0..3).find(|&i| key(newer[i], newer[(i + 1) % 3]) == e)?;
        let (v2, v4, v1) = (newer[i], newer[(i + 1) % 3], newer[(i + 2) % 3]);
        let v3 = *older.iter().find(|&&v| v != e.0 && v != e.1)?;
        (v1 != v3).then_some([v1, v2, v3, v4])
    };
    let gain = |tris: &[[u32; 3]], on: &BTreeMap<(u32, u32), Vec<usize>>, e: (u32, u32)| -> f32 {
        let Some([v1, v2, v3, v4]) = quad(tris, on, e) else {
            return f32::MAX;
        };
        let p = |v: u32| xy[v as usize];
        // seen from the side the two triangles face: mirrored if they are wound clockwise
        let (no_a, no_b) = (
            cross_tri(p(v2), p(v3), p(v4)),
            cross_tri(p(v2), p(v4), p(v1)),
        );
        let no = no_a + no_b;
        if no == 0.0 {
            return f32::MAX;
        }
        let scale = no.abs();
        let flip = |q: [f32; 2]| if no < 0.0 { [-q[0], q[1]] } else { q };
        let (q1, q2, q3, q4) = (flip(p(v1)), flip(p(v2)), flip(p(v3)), flip(p(v4)));
        let sign = |x: f32| {
            if x > 1e-5 {
                1
            } else if x < -1e-5 {
                -1
            } else {
                0
            }
        };
        if sign(cross_tri(q2, q3, q4) / scale) + sign(cross_tri(q2, q4, q1) / scale) == 0 {
            return f32::MAX;
        }
        turn_gain(q1, q2, q3, q4)
    };
    let mut heap = Heap::default();
    for (i, &e) in inner.iter().enumerate() {
        let cost = gain(tris, &on, e);
        if cost < 0.0 {
            heap.insert(cost, i);
        }
    }
    // the states each edge has been turned into, so it isn't turned back for ever
    let mut been: Vec<BTreeSet<[(u32, u32); 2]>> = vec![BTreeSet::new(); inner.len()];
    while let Some(i) = heap.pop() {
        let e = inner[i];
        let Some([v1, v2, v3, v4]) = quad(tris, &on, e) else {
            continue;
        };
        let new = key(v1, v3);
        if on.contains_key(&new) {
            continue;
        }
        // the two triangles become (1, 2, 3) and (1, 3, 4); the second is the one the new edge is measured from
        let faces = on.remove(&e).expect("the edge");
        let (newer, tb) = (faces[1], faces[0]);
        let ta = free.replace(newer).unwrap_or_else(|| {
            tris.push([0; 3]);
            tris.len() - 1
        });
        tris[ta] = [v1, v2, v3];
        tris[tb] = [v4, v1, v3];
        on.insert(new, vec![ta, tb]);
        for (a, b, t) in [(v1, v2, ta), (v2, v3, ta), (v3, v4, tb), (v4, v1, tb)] {
            let list = on.get_mut(&key(a, b)).expect("a side of the quad");
            list.retain(|&f| f != newer && f != tb);
            list.push(t);
        }
        index.remove(&e);
        index.insert(new, i);
        inner[i] = new;
        been[i].insert([new, key(v2, v4)]);
        // the four sides' own turns are worth something else now
        for (a, b) in [(v3, v4), (v4, v1), (v1, v2), (v2, v3)] {
            let side = key(a, b);
            let Some(&j) = index.get(&side) else { continue };
            heap.remove(j);
            if !been[j].is_empty()
                && let Some([q1, _, q3, _]) = quad(tris, &on, side)
                && been[j].contains(&[key(q1, q3), side])
            {
                continue;
            }
            let cost = gain(tris, &on, side);
            if cost < 0.0 {
                heap.insert(cost, j);
            }
        }
    }
    if let Some(free) = free {
        tris.remove(free);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::v3;

    #[test]
    fn a_hexagon_is_clipped_at_every_other_corner() {
        let p: Vec<V3> = (0..6)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 6.0;
                v3(a.cos(), a.sin(), 0.0)
            })
            .collect();
        assert_eq!(
            polygon(&p, V3::Z),
            [[5, 0, 1], [1, 2, 3], [3, 4, 5], [1, 3, 5]]
        );
    }

    #[test]
    fn a_dented_polygon_is_filled_inside_its_outline() {
        // an arrow head: the dent at corner 3
        let p = [
            v3(0, 0, 0),
            v3(2, 0, 0),
            v3(2, 2, 0),
            v3(1, 0.5, 0),
            v3(0, 2, 0),
        ];
        let tris = polygon(&p, V3::Z);
        assert_eq!(tris.len(), 3);
        let area: f64 = tris
            .iter()
            .map(|t| {
                let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
                ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)) as f64 / 2.0
            })
            .sum();
        assert!(tris.iter().all(|t| {
            let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
            (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) > 0.0
        }));
        assert!((area - 2.5).abs() < 1e-6, "{area}");
    }
}
