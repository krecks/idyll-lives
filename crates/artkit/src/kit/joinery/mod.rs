//! Joinery and the frames built like it: paths bent round their corners, frame-and-panel fronts, mouldings mitred
//! round an outline, and a bench's cast ends.

mod bench;
mod carve;
mod corner;
mod cut;
mod extent;
mod fillet;
mod hole;
mod moulding;
mod polygon;
mod recessed;
mod shaker;

pub use bench::*;
use carve::*;
pub use corner::*;
pub use cut::*;
use extent::*;
pub use fillet::*;
use hole::*;
pub use moulding::*;
use polygon::*;
pub use recessed::*;
pub use shaker::*;
