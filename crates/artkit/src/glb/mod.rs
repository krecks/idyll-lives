//! Writing a model out: the .glb (docs/design/objects.md, "Export settings"), laid out as the files the game
//! has always been given: +Y up (Blender -Y is the game's +Z front), a node a part with its name and origin, a
//! primitive a material with POSITION, NORMAL, TEXCOORD_0 (only where a textured finish is used) and COLOR_0 (the
//! baked shading), materials as a name and a base colour, clips as animations from 0 s with a key a frame.

mod affine;
mod encode;
mod file;
mod json;
mod model;
mod numbers;
mod order;
mod parse;
mod read;
mod target;
mod types;
mod write;
mod yup;

pub use affine::*;
pub use encode::*;
use json::*;
pub use model::*;
pub(crate) use numbers::*;
pub use numbers::{ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, FLOAT, UINT, USHORT};
use order::*;
pub use parse::*;
use read::*;
pub use target::*;
pub use types::*;
pub use write::*;
use yup::*;
