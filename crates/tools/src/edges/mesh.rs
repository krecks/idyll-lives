//! A mesh, its edges and the faces that meet at them.

use super::*;

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
pub(super) struct Mix(u64);

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

pub(super) type Map<K, V> = HashMap<K, V, BuildHasherDefault<Mix>>;

/// A triangle of the welded shape.
pub(super) struct Face {
    pub(super) mesh: u32,
    /// Its corners' places.
    pub(super) v: [u32; 3],
    /// Its corners in its mesh.
    pub(super) c: [u32; 3],
    /// Its normal, from its winding.
    pub(super) n: V3,
}

/// The triangles on an edge: how many, and the first two as (face, the corner the edge starts at there).
#[derive(Clone, Copy, Default)]
pub(super) struct Sides {
    pub(super) count: u32,
    pub(super) at: [(u32, u8); 2],
}

/// Every mesh's triangles with their positions welded: what the eye takes for one shape.
pub(super) struct Shape<'a> {
    pub(super) meshes: &'a [Mesh],
    pub(super) place: Vec<V3>,
    pub(super) faces: Vec<Face>,
    pub(super) edges: Map<(u32, u32), Sides>,
    /// The edges in the order they were met.
    pub(super) order: Vec<(u32, u32)>,
}
