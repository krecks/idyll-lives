//! Corners: one end of an edge at a vertex, and the order round it.

use super::*;

/// One end of an edge at a vertex.
#[derive(Clone, Debug)]
pub(super) struct Half {
    pub(super) to: u32,
    pub(super) bev: bool,
    /// The face between this edge and the next round the vertex.
    pub(super) fnext: u32,
    /// How far it is cut back on the face before it and on the face after it.
    pub(super) offset_l: f32,
    pub(super) offset_r: f32,
    /// The new points either side of it (places among the vertex's).
    pub(super) left: usize,
    pub(super) right: usize,
}

/// A new point at a vertex, between two bevelled edges.
#[derive(Clone, Debug)]
pub(super) struct Bound {
    pub(super) co: V3,
    /// The bevelled edge before it round the vertex.
    pub(super) efirst: usize,
    /// The bevelled edge after it, which its profile to the next point crosses.
    pub(super) ebev: usize,
    /// The edge it slid along, if it did, and the sines of that edge's angles to the bevelled edges after
    /// and before it, one over the other.
    pub(super) eon: Option<usize>,
    pub(super) sinratio: f32,
    /// The face it lies on, for the texture.
    pub(super) face: u32,
    pub(super) visited: bool,
    /// The point along the next bevelled edge whose width is evened with this one's.
    pub(super) chain: Option<(u32, usize)>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Corner {
    pub(super) v: u32,
    pub(super) edges: Vec<Half>,
    pub(super) bounds: Vec<Bound>,
    /// Which of `edges` Blender's own list of them starts at.
    pub(super) listed_from: usize,
}

/// A face a corner makes: its corners, each one's source face, its material, for the corner's polygon the vertex,
/// and for a patch's quad the vertex and its place in the patch (new point, ring, step).
pub(super) type CornerFace = (
    Vec<u32>,
    Vec<u32>,
    u16,
    Option<u32>,
    Option<(u32, usize, usize, usize)>,
);

/// Each edge's faces, with the places of its two ends among the mesh's corners.
pub(super) type Sides = BTreeMap<(u32, u32), Vec<(u32, usize, usize)>>;

/// A vertex's new points' numbers by (bound, ring, step).
pub(super) type Numbers = BTreeMap<(usize, usize, usize), u32>;

impl Corner {
    pub(super) fn half_to(&self, to: u32) -> usize {
        self.edges
            .iter()
            .position(|h| h.to == to)
            .expect("an edge of the vertex")
    }
}

/// The patch point that stands for (i, j, k): each is made once.
pub(super) fn canon(n: usize, ns: usize, i: usize, j: usize, k: usize) -> (usize, usize, usize) {
    let ns2 = ns / 2;
    if ns % 2 == 1 {
        if j <= ns2 && k <= ns2 {
            (i, j, k)
        } else if k <= ns2 {
            ((i + n - 1) % n, k, ns - j)
        } else {
            ((i + 1) % n, ns - k, j)
        }
    } else if j < ns2 && k <= ns2 {
        (i, j, k)
    } else if j == ns2 && k == ns2 {
        (0, j, k)
    } else if k <= ns2 {
        ((i + n - 1) % n, k, ns - j)
    } else {
        ((i + 1) % n, ns - k, j)
    }
}
