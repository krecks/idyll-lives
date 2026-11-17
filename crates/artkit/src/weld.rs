//! Which points of a mesh sit on one another: Blender's search (a k-d tree, each point asked in its turn for the
//! others within reach), so the same one of each cluster survives as in the meshes Blender made.

use crate::math::V3;

const UNSET: u32 = u32::MAX;

#[derive(Clone, Copy)]
struct Node {
    left: u32,
    right: u32,
    co: [f32; 3],
    index: u32,
    d: usize,
}

/// Sorts `nodes` about their median along `axis` and so on down, as Blender balances its tree; the root.
fn balance(nodes: &mut [Node], axis: usize, ofs: u32) -> u32 {
    let len = nodes.len();
    if len == 0 {
        return UNSET;
    }
    if len == 1 {
        return ofs;
    }
    let (mut left, mut right, median) = (0usize, len - 1, len / 2);
    while right > left {
        let co = nodes[right].co[axis];
        let mut i = left.wrapping_sub(1);
        let mut j = right;
        loop {
            loop {
                i = i.wrapping_add(1);
                if nodes[i].co[axis] >= co {
                    break;
                }
            }
            loop {
                j -= 1;
                if !(nodes[j].co[axis] > co && j > left) {
                    break;
                }
            }
            if i >= j {
                break;
            }
            swap_heads(nodes, i, j);
        }
        swap_heads(nodes, i, right);
        if i >= median {
            right = i.wrapping_sub(1);
            if i == 0 {
                break;
            }
        }
        if i <= median {
            left = i + 1;
        }
    }
    nodes[median].d = axis;
    let next = (axis + 1) % 3;
    let (lo, rest) = nodes.split_at_mut(median);
    let l = balance(lo, next, ofs);
    let r = balance(&mut rest[1..], next, median as u32 + 1 + ofs);
    nodes[median].left = l;
    nodes[median].right = r;
    median as u32 + ofs
}

/// (The tree's links stay with their places; a node's point and index move.)
fn swap_heads(nodes: &mut [Node], a: usize, b: usize) {
    let (na, nb) = (nodes[a], nodes[b]);
    nodes[a].co = nb.co;
    nodes[a].index = nb.index;
    nodes[b].co = na.co;
    nodes[b].index = na.index;
}

struct Search<'a> {
    nodes: &'a [Node],
    range: f32,
    range_sq: f32,
    target: Vec<i32>,
    found: usize,
}

impl Search<'_> {
    fn visit(&mut self, i: u32, search: u32, co: [f32; 3]) {
        let node = self.nodes[i as usize];
        if co[node.d] + self.range <= node.co[node.d] {
            if node.left != UNSET {
                self.visit(node.left, search, co);
            }
        } else if co[node.d] - self.range >= node.co[node.d] {
            if node.right != UNSET {
                self.visit(node.right, search, co);
            }
        } else {
            if search != node.index && self.target[node.index as usize] == -1 {
                let mut d = 0.0_f32;
                for (a, b) in node.co.iter().zip(&co) {
                    d += (a - b) * (a - b);
                }
                if d <= self.range_sq {
                    self.target[node.index as usize] = search as i32;
                    self.found += 1;
                }
            }
            if node.left != UNSET {
                self.visit(node.left, search, co);
            }
            if node.right != UNSET {
                self.visit(node.right, search, co);
            }
        }
    }
}

/// For each of `verts`: -1 (alone), itself (others merge into it) or the vertex it merges into. A cluster's
/// points merge into the one of them nearest their middle.
pub fn doubles(verts: &[V3], dist: f32) -> Vec<i32> {
    let mut nodes: Vec<Node> = verts
        .iter()
        .enumerate()
        .map(|(i, v)| Node {
            left: UNSET,
            right: UNSET,
            co: v.array(),
            index: i as u32,
            d: 0,
        })
        .collect();
    let root = balance(&mut nodes, 0, 0);
    let mut s = Search {
        nodes: &nodes,
        range: dist,
        range_sq: dist * dist,
        target: vec![-1; verts.len()],
        found: 0,
    };
    if root == UNSET {
        return s.target;
    }
    // each point in its turn gathers the others in reach that no earlier one has
    for (index, v) in verts.iter().enumerate() {
        let t = s.target[index];
        if t == -1 || t == index as i32 {
            let before = s.found;
            s.visit(root, index as u32, v.array());
            if s.found != before {
                s.target[index] = index as i32;
            }
        }
    }
    let mut clusters: std::collections::BTreeMap<i32, Vec<usize>> = Default::default();
    for (i, &t) in s.target.iter().enumerate() {
        if t >= 0 {
            clusters.entry(t).or_default().push(i);
        }
    }
    let mut target = vec![-1; verts.len()];
    for members in clusters.values() {
        let mut mid = [0.0_f32; 3];
        for &i in members {
            for (m, c) in mid.iter_mut().zip(verts[i].array()) {
                *m += c;
            }
        }
        let k = 1.0 / members.len() as f32;
        let mid = mid.map(|m| m * k);
        let far = |i: usize| {
            let v = verts[i].array();
            (0..3)
                .map(|d| (v[d] - mid[d]) * (v[d] - mid[d]))
                .sum::<f32>()
        };
        let mut best = members[0];
        for &i in members {
            if far(i) < far(best) {
                best = i;
            }
        }
        for &i in members {
            target[i] = best as i32;
        }
    }
    target
}
