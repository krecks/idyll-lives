//! Tops that things stand on, and seats at tables.
//!
//! Objects with a `surface` (tables, counters, desks, nightstands, shelves) carry small things:
//! objects on the `Surface` layer (vases, books, lamps) and plated food. A thing on a surface
//! stands at its tile's middle moved by its `offset`; its host is worked out from where it
//! stands, never stored (object ids shift when objects are sold or moved). The disc it takes
//! (its `radius`) lies wholly on one host's top, inside the margin kept clear along the edges,
//! and overlaps nothing else standing there.
//!
//! Seats with a `seatAt` (dining chairs, stools) are used from any free tile around them
//! (`Approach::Around`) and, facing a table of a kind they tuck under, are seats at that table.
//!
//! Everything here works on slices, so residents deciding what to do (which only see `Ctx`) can
//! use it. `Links` caches what depends only on where objects and walls stand; it is rebuilt with
//! the object index when the structure changes.

use crate::content::{Approach, Content, DEFAULT_RADIUS, Layer, Pose, Surface};
use crate::lot::Edge;
use crate::path::NavGrid;
use crate::world::{MAX_OFFSET, ObjectInstance};
use std::f32::consts::{FRAC_PI_2, PI};

/// Slack (m) when checking that a disc stays on a top: offsets are whole centimetres.
const SLACK: f32 = 0.006;
/// How far in from a table's edge a plate goes in front of a chair (m).
const PLATE_IN: f32 = 0.3;
/// How far in front of the middle of a seat built into a table its plate goes (m).
const PLATE_AHEAD: f32 = 0.2;
/// Spacing (m) of the places tried on a top.
const STEP: f32 = 0.05;
/// How far a plate may go from right in front of its diner when that place is taken (m).
const PLATE_SEARCH: f32 = 0.6;

mod approach;
mod hosts;
mod seating;
mod types;

pub use approach::*;
pub use hosts::*;
pub use seating::*;
pub use types::*;
