//! Which points of a mesh sit on one another, and which of them stays: Blender 5.2's rule (`find_doubles`), so the
//! same one of each cluster survives as in the meshes Blender made. A sphere's seam and poles are such clusters,
//! their points a rounding apart, and the one kept is a vertex of every lump a build makes.

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

/// For each of `verts`: -1 (alone), itself (others merge into it) or the vertex it merges into. Each point in
/// the tree's order gathers those in reach that no earlier one has, and they merge into the one of them nearest
/// their middle (the gatherer weighing twice), the earliest of them where two are as near.
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
    let mut target = vec![-1_i32; verts.len()];
    if root == UNSET {
        return target;
    }
    let range_sq = dist * dist;
    for at in 0..nodes.len() {
        let seed = nodes[at].index as usize;
        if target[seed] != -1 {
            continue;
        }
        let co = nodes[at].co;
        // the points in reach not yet taken (the gatherer among them), as the tree is walked
        let mut found: Vec<usize> = Vec::new();
        let mut stack = vec![root];
        while let Some(i) = stack.pop() {
            let node = nodes[i as usize];
            let (lo, hi) = (
                co[node.d] + dist <= node.co[node.d],
                co[node.d] - dist >= node.co[node.d],
            );
            if lo {
                if node.left != UNSET {
                    stack.push(node.left);
                }
                continue;
            }
            if hi {
                if node.right != UNSET {
                    stack.push(node.right);
                }
                continue;
            }
            if target[node.index as usize] == -1 {
                let mut d = 0.0_f32;
                for (a, b) in node.co.iter().zip(&co) {
                    d += (a - b) * (a - b);
                }
                if d <= range_sq {
                    found.push(node.index as usize);
                }
            }
            if node.right != UNSET {
                stack.push(node.right);
            }
            if node.left != UNSET {
                stack.push(node.left);
            }
        }
        found.push(seed);
        if found.len() == 2 {
            continue;
        }
        let mut mid = [0.0_f32; 3];
        for &i in &found {
            for (m, c) in mid.iter_mut().zip(verts[i].array()) {
                *m += c;
            }
        }
        let mid = mid.map(|m| m / found.len() as f32);
        let far = |i: usize| {
            let v = verts[i].array();
            (v[0] - mid[0]) * (v[0] - mid[0])
                + (v[1] - mid[1]) * (v[1] - mid[1])
                + (v[2] - mid[2]) * (v[2] - mid[2])
        };
        let mut best = seed;
        for &i in &found[..found.len() - 1] {
            if far(i) < far(best) || (far(i) == far(best) && i <= best) {
                best = i;
            }
        }
        for &i in &found {
            target[i] = best as i32;
        }
    }
    target
}
