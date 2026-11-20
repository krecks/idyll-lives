//! What a bevel is asked for and what it makes: the order, the kinds, the picks.
//!
//! Split from `bevel.rs` (see `mod.rs`).

use super::*;

/// How a mesh's edges are numbered and which way each runs, which decides the edge a corner's new points start
/// from and the corner each strip starts at.
#[derive(Clone, Debug)]
pub enum EdgeOrder {
    /// As the faces' corners name them, each face from its first side, each from its lower vertex to its higher
    /// (a mesh made of points and faces).
    Corners,
    /// Each face's closing side first, then its others, each the way that face runs it (a primitive of Blender's
    /// own).
    Faces,
    /// These first, in this order, each from its first vertex to its second; the others after them, as `Corners`
    /// has them (an outline's rims, every edge of a cylinder).
    Listed(Vec<(u32, u32)>),
}

/// What a face of the bevelled mesh is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A face no bevel touched.
    Kept,
    /// A face of the mesh, its corners cut back.
    Cut,
    /// A strip along a bevelled edge.
    Strip,
    /// A corner's polygon or patch.
    Corner,
}

/// A new face whose texture hangs on the old mesh's UVs (`carried`): which old face's it takes is settled once
/// they are known.
#[derive(Clone, Debug)]
pub enum Pick {
    /// A strip's quad (its first corner's place among the new mesh's), step `k` of the strips along the old edge
    /// `edge` from the face `faces.0`, which its first two corners are nearer, to `faces.1`.
    Strip {
        at: usize,
        edge: (u32, u32),
        faces: (u32, u32),
        k: usize,
    },
    /// A corner's polygon at the old vertex `vert`: the face each of its corners lies on, and round the vertex
    /// each bevelled edge's other end with the faces before and after it.
    Corner {
        at: usize,
        vert: u32,
        on: Vec<u32>,
        edges: Vec<(u32, u32, u32)>,
        /// (Of an odd number of segments over one, the polygon in the middle of a patch: as a patch quad's.)
        across: Vec<(u32, u32, u32)>,
    },
    /// A quad of a corner's patch of an odd number of segments at the old vertex `vert`: the one at `place` (new
    /// point, ring, step). `across`: for each new point round the vertex, the bevelled edge its profile to the next
    /// crosses (its other end) and the faces before and after that edge.
    Patch {
        at: usize,
        vert: u32,
        place: (usize, usize, usize),
        across: Vec<(u32, u32, u32)>,
    },
}

/// A mesh with its edges bevelled.
#[derive(Clone, Debug)]
pub struct Bevelled {
    pub mesh: Mesh,
    pub kinds: Vec<Kind>,
    /// For each corner of the new mesh, the face of the old one whose texture it takes (but for `picks`).
    pub source: Vec<u32>,
    pub picks: Vec<Pick>,
    pub segments: usize,
    /// The offset the edges were cut back by: the width, or less where the mesh has no room for it.
    pub offset: f32,
    /// For each point of the new mesh, the old vertex whose corner it was made at (`u32::MAX`: a point kept).
    pub made_at: Vec<u32>,
    /// For each old vertex with a corner: its bevelled edges in Blender's order of the vertex's edges (each one's
    /// other end and the faces before and after it), and the corner's new points (`Charts::centre` reads them).
    pub centres: BTreeMap<u32, Centre>,
}

/// A corner's bevelled edges, each as (other end, face before, face after), and its new points' numbers.
pub type Centre = (Vec<(u32, u32, u32)>, Vec<u32>);
