//! The raised vegetable bed, the garden shed, the compost bin and the rain barrel.

use super::common::{F, M, blob, board, ground, pot, prism, rod};
use artkit::far::{is_low, low};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, M4, P3, PI, TAU, V3, atan2, cos, cosd, degrees, radians, sin, sind};
use artkit::motion::{Ease, Rock, Squash, Stages, Swing, rock, squash, stages, swing};
use artkit::parts::numbered;
use artkit::parts::{hinged, join, join_all, measure, new_parts};
use artkit::place::Facing;
use artkit::place::N;
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::shapes::SHARP;
use artkit::{
    cone, fbox, fcone, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, settle, torus,
};

// --- the raised vegetable bed (2x1): boards round the soil, lettuces, carrots and tomato plants --------------
mod barrel;
mod beds;
mod compost;
mod shed;
mod shed_build;

pub use barrel::*;
pub use beds::*;
pub use compost::*;
use shed::*;
pub use shed_build::*;
