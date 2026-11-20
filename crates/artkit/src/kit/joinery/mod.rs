//! Joinery and the frames built like it: paths bent round their corners, frame-and-panel fronts, mouldings mitred
//! round an outline, and a bench's cast ends.

use crate::bevel::EdgeOrder;
use crate::forms::{self, Extrude, Holes, Loft, Profile, Scale, Sweep, rounded_rect};
use crate::json::float_repr;
use crate::kit::{Scroll, least, most, scroll};
use crate::materials::Mat;
use crate::math::{Num, P3, PI, V3, pow, sin, tan};
use crate::mesh::{Mesh, edge, normal_newell};
use crate::place::{Coord, Facing, GameFrame};
use crate::py;
use crate::scene::Obj;
use crate::shapes::{Cube, SHARP, options};
use std::collections::BTreeMap;
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
pub(crate) use carve::*;
pub use corner::*;
pub use cut::*;
pub(crate) use extent::*;
pub use fillet::*;
pub(crate) use hole::*;
pub use moulding::*;
pub(crate) use polygon::*;
pub use recessed::*;
pub use shaker::*;
