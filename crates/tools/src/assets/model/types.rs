//! A model as it is compared: its primitives and corners.

/// A triangle's corner as the game gets it.
#[derive(Clone, Copy, Debug)]
pub struct Corner {
    /// Where it is, in the model's own space (every node's place applied).
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// The baked shading, 8 bits a channel.
    pub color: [u8; 4],
}

pub struct Primitive {
    pub material: Option<usize>,
    /// Three corners a triangle.
    pub corners: Vec<Corner>,
}

pub struct Model {
    /// Per mesh (in the file's order), its primitives.
    pub meshes: Vec<Vec<Primitive>>,
    /// Every animation sampler's keys and values, as the bytes of their accessors, in the file's order.
    pub animations: Vec<Vec<u8>>,
    /// The names of the nodes that carry a name.
    pub nodes: Vec<String>,
    /// Per mesh: half its box's longest side as stored (before any node's place), times the most its
    /// node stretches a length. A 14-bit step of the first is what quantising may move a corner by.
    pub spans: Vec<f32>,
}

pub(super) type Mat = [[f32; 4]; 3];
