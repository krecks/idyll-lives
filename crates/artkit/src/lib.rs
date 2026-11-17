//! The kernel the game's item models are generated with.
#![forbid(unsafe_code)]

pub mod bvh;
pub mod far;
pub mod forms;
pub mod json;
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
pub mod py;
pub mod rand;
pub mod scene;
pub mod shapes;
pub mod tri;
pub mod uv;
pub mod weld;
