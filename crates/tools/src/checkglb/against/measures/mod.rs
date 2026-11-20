//! The measurements themselves: geometry, surfaces, texels and shading.

mod accepted;
mod against;
mod context;
mod context_of;
mod decimated;
mod differ;
mod git;
mod side;
mod side_of;
mod tolerance;
mod types;
mod whole;

pub(crate) use accepted::*;
pub use against::*;
pub(crate) use context::*;
pub(crate) use decimated::*;
pub(crate) use differ::*;
pub(crate) use git::*;
pub(crate) use side::*;
pub(crate) use side_of::*;
pub(crate) use tolerance::*;
pub use types::*;
pub(crate) use whole::*;
