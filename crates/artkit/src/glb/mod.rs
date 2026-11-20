//! Writing a model out: the .glb (docs/design/objects.md, "Export settings"), laid out as the files the game
//! has always been given: +Y up (Blender -Y is the game's +Z front), a node a part with its name and origin, a
//! primitive a material with POSITION, NORMAL, TEXCOORD_0 (only where a textured finish is used) and COLOR_0 (the
//! baked shading), materials as a name and a base colour, clips as animations from 0 s with a key a frame.

use crate::json::{Json, float_repr};
use crate::materials::Finishes;
use crate::math::{EulerOrder, V3, cos, sin};
use crate::motion::{self, Path};
use crate::normals::corner_normals;
use crate::scene::{self, FPS, Obj};
use std::collections::BTreeMap;

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
pub(crate) use json::*;
pub use model::*;
pub(crate) use numbers::*;
pub use numbers::{ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, FLOAT, UINT, USHORT};
pub(crate) use order::*;
pub use parse::*;
pub(crate) use read::*;
pub use target::*;
pub use types::*;
pub use write::*;
pub(crate) use yup::*;
