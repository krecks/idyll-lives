//! What a model is read into: its materials, meshes, nodes and channels.
//!
//! Split from `glb.rs` (see `mod.rs`).

use super::*;

/// A material: its name (the pack maps it to a finish) and its factors (glTF's defaults where the file has none).
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub name: String,
    /// Linear RGBA.
    pub base: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub double_sided: bool,
    /// `alphaMode` MASK: drawn where the texture's alpha is at least this.
    pub cutoff: Option<f32>,
}

/// The triangles of one material on a mesh.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Primitive {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Option<Vec<[f32; 2]>>,
    /// COLOR_0, linear RGBA: the baked shading (grey).
    pub colours: Option<Vec<[f32; 4]>>,
    /// Three a triangle.
    pub indices: Vec<u32>,
    pub material: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Primitive>,
}

/// A part: its name, mesh and children, and where it stands in its parent at rest.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub name: String,
    pub mesh: Option<usize>,
    pub children: Vec<usize>,
    pub translation: [f32; 3],
    /// A quaternion (x, y, z, w).
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    /// The clips (animations) that move it, in the order their keys lie in the file (the order of its tracks):
    /// what `encode` lays them out by.
    pub clips: Vec<usize>,
}

impl Default for Node {
    fn default() -> Node {
        Node {
            name: String::new(),
            mesh: None,
            children: Vec::new(),
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
            clips: Vec::new(),
        }
    }
}

/// A model as a .glb has it: the scene's root parts, the parts, meshes, materials and clips.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub generator: String,
    pub roots: Vec<usize>,
    pub nodes: Vec<Node>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub animations: Vec<Animation>,
}

/// A transform as three rows of a 4×4 matrix (the fourth is 0 0 0 1): a point p goes to (row · (p, 1)).
pub type Affine = [[f32; 4]; 3];

pub const IDENTITY: Affine = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
];
