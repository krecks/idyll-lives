//! The kernel the game's item models are generated with.
#![forbid(unsafe_code)]

pub mod bevel;
pub mod botany;
pub mod bvh;
pub mod decimate;
pub mod edit;
pub mod family;
pub mod far;
pub mod forms;
pub mod glb;
pub mod json;
pub mod kit;
mod macros;
pub use macros::Arg;
pub mod materials;
pub mod math;
pub mod mesh;
pub mod motion;
pub mod noise;
mod noise_tables;
pub mod normals;
pub mod parts;
pub mod perch;
pub mod place;
pub mod preview;
pub mod py;
pub mod rand;
pub mod raster;
pub mod scene;
pub mod shade;
pub mod shapes;
pub mod soft;
pub mod tri;
pub mod uv;
pub mod weld;
