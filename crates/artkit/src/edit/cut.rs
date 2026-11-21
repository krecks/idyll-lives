//! A face while it is cut, and bmesh's split of one face in two.
//!
//! `OpenEdge` (a face along an edge) is here too: the shell's rim and `holes_fill` both read it.

use crate::math::V3;

/// A face along an edge: (the face, the vertex it comes from, the vertex it goes to).
pub(super) type OpenEdge = (usize, u32, u32);

/// A face while it is being cut: its corners from the one it starts at, its material and shading, and the
/// normal it had before the cut (bmesh keeps a face's normal through its splits).
#[derive(Clone)]
pub(super) struct Cut {
    pub(super) corners: Vec<u32>,
    pub(super) mat: u16,
    pub(super) smooth: bool,
    pub(super) normal: V3,
}

/// bmesh's split of a face from the corner at ia to the corner at ib (not neighbours): the part that keeps the
/// face's place, from ia round to ib, and the new face, from ib round to ia, each starting where bmesh starts it.
pub(super) fn split_face(c: &[u32], ia: usize, ib: usize) -> (Vec<u32>, Vec<u32>) {
    let n = c.len();
    // the corners from one place round to (not with) another
    let run = |from: usize, to: usize| -> Vec<usize> {
        let mut out = Vec::new();
        let mut i = from;
        while i != to {
            out.push(i);
            i = (i + 1) % n;
        }
        out
    };
    // the kept face: a new corner at ib's vertex, then ia .. the corner before ib
    let kept = run(ia, ib);
    let other = run(ib, ia);
    let in_kept = kept.contains(&0);
    let ids = |first: usize, new_at: usize, rest: &[usize]| -> Vec<u32> {
        // the cycle as bmesh has it: the new corner (at vertex new_at), then the run; turned to start at first
        let mut cycle: Vec<(bool, usize)> = vec![(true, new_at)];
        cycle.extend(rest.iter().map(|&i| (false, i)));
        cycle.rotate_left(first);
        cycle.into_iter().map(|(_, i)| c[i]).collect()
    };
    // (places in each cycle: 0 is its new corner, k + 1 the run's k-th)
    let (kept_first, other_first) = if in_kept {
        let at = kept.iter().position(|&i| i == 0).unwrap() + 1;
        let other_first = if at == 1 {
            other.len() // the corner before the other's new one
        } else if at == kept.len() {
            1 // the corner after it
        } else {
            0
        };
        (at, other_first)
    } else {
        let at = other.iter().position(|&i| i == 0).unwrap() + 1;
        let kept_first = if at == 1 {
            kept.len()
        } else if at == other.len() {
            1
        } else {
            0
        };
        (kept_first, at)
    };
    (ids(kept_first, ib, &kept), ids(other_first, ia, &other))
}
