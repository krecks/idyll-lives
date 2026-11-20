//! The rides: the step-through city bike, the mountain, road, cargo and e-bikes, the kids' bike on stabilisers and the
//! e-scooter. No clips: the renderer turns their parts by code, each a top-level node on its pivot: `frame`,
//! `wheel_front`, `wheel_rear`, `cranks`, `stand`.

use super::common::{F, M, arc_points, bent, board, rod};
use super::play::{Bike, bike, wheel};
use artkit::far::{is_low, low};
use artkit::forms::rounded_rect;
use artkit::math::{P3, cosd, pow, sind};
use artkit::parts::numbered;
use artkit::parts::{Settle, join, join_all, new_parts, settle_with};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, loft, revolve, ring, soft, torus};

mod bikes;
mod build;
mod extras;
mod parts;
mod scooter;

pub(crate) use bikes::*;
pub use build::*;
pub(crate) use extras::*;
pub(crate) use parts::*;
pub use scooter::*;
