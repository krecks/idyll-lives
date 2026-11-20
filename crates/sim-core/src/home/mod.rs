//! Build and buy: changing a household's home. Buying, selling, moving, restyling and
//! upgrading objects, and building walls, doors and windows.
//!
//! Edits are addressed to a household (it may have nobody living in it yet: build first, move
//! a family in later). Every change is checked against its home plot and paid from household
//! funds, except in Creative games, where building is free. Nothing
//! may cut Sims off from what they could reach before (an object's front tile, a room).
//! With diagonal walls the same rule holds: a tile crossed by a diagonal wall can't be stood
//! on, so walling across an object's front tile or around a resident is refused.

use std::collections::VecDeque;

use crate::Error;
use crate::command::{EdgeAxis, EdgeEdit, EdgeKind, FacePaint, FloorPaint};
use crate::content::Layer;
use crate::lot::{Diagonal, Edge, EdgeLook, EdgeRef, FORM_HALF, OUTDOORS};
use crate::path::NavGrid;
use crate::social::EventKind;
use crate::world::{
    GameMode, Layout, MAX_OFFSET, MAX_SHORTER, MAX_SIZE, MAX_TALLER, MIN_SIZE, ObjectInstance,
    TaskKind, World, end_activity, kind_size,
};

/// Tiles around a plot that still count when checking who can reach what (the path from the street).
const REACH_MARGIN: i32 = 2;
/// Build and buy edits kept for undo.
const UNDO_DEPTH: usize = 30;

/// The world as it was before a build or buy edit: everything an edit can change (walls,
/// objects and money, and residents stopped or moved by it). Time stands still while
/// edits are kept (see `World::apply`), so putting it back restores exactly that moment.
#[derive(Debug, Clone)]
pub(crate) struct HomeSnapshot {
    household: usize,
    /// Where the lot's log stood when it was taken (`Lot::log_begin`), and once the edit is
    /// done (`remember`), what it changed on the lot, as it was before: walls, looks and floors
    /// (the rooms follow from them). A big world's lot is too big to copy for every edit.
    lot_mark: crate::lot::LogMark,
    lot: Option<Vec<crate::lot::LotChange>>,
    objects: Vec<ObjectInstance>,
    object_plot: Vec<Option<u32>>,
    blocked: Vec<bool>,
    sims: Vec<crate::world::Sim>,
    households: Vec<crate::world::Household>,
    plots: Vec<crate::world::Plot>,
    /// Food and dishes (they move and go with the furniture they're on).
    items: crate::items::Items,
    /// The garages, the cars (one parks in front of a garage built at the end of its driveway) and
    /// the tiles they take up.
    garages: Vec<crate::cars::Garage>,
    /// The porches and porch pieces (a post's tile is in `car_tiles`, as a garage's).
    porches: Vec<crate::porches::Porch>,
    porch_pieces: Vec<crate::porches::PorchPiece>,
    cars: Vec<crate::cars::Car>,
    car_tiles: Vec<usize>,
    cars_parked: Option<Vec<u64>>,
    /// The story's latest event id when it was taken, and the events the edit after it told
    /// (an upgrade): undo takes them out of the story, redo tells them again.
    events_before: u64,
    story: Vec<crate::social::SocialEvent>,
}

fn edge_kind(kind: EdgeKind) -> Edge {
    match kind {
        EdgeKind::Wall => Edge::Wall,
        EdgeKind::Door => Edge::Door,
        EdgeKind::Window => Edge::Window,
        EdgeKind::Open => Edge::Open,
        EdgeKind::Fence => Edge::Fence,
        EdgeKind::Gate => Edge::Gate,
    }
}

/// Where an edit's wall is (after the bounds checks: coordinates are on the lot).
fn edge_ref(axis: EdgeAxis, x: i32, z: i32) -> EdgeRef {
    let (x, z) = (x as u16, z as u16);
    match axis {
        EdgeAxis::H => EdgeRef::H(x, z),
        EdgeAxis::V => EdgeRef::V(x, z),
        EdgeAxis::Dp | EdgeAxis::Dn => EdgeRef::Diag(x, z),
    }
}

/// Whether a task is at object `id` (using it, repairing it, cooking at it, sitting at it to
/// eat or to be on the phone, washing up at it): it stops when the object goes.
fn uses_object(t: &TaskKind, id: u32) -> bool {
    match *t {
        TaskKind::Use { object, .. } | TaskKind::Repair { object } => object == id,
        TaskKind::Cook { station, .. } | TaskKind::Dishes { station, .. } => station == Some(id),
        TaskKind::Eat { seat, .. } | TaskKind::Phone { seat, .. } => seat == Some(id),
        TaskKind::Carry { from, to, .. } => from == id || to == id,
        // (Once it's in hand, whatever it was taken from may go.)
        TaskKind::Serve { from, stage: 0, .. } => from == id,
        _ => false,
    }
}

/// Object `id` is gone: the task's object ids after it shift down by one (a recipe started
/// from it goes on without). Tasks hold indices into `World::objects` rather than stable ids,
/// so this stays even though player-facing ids no longer move.
fn shift_objects(t: &mut TaskKind, id: u32) {
    let shift = |o: &mut u32| {
        if *o > id {
            *o -= 1;
        }
    };
    match t {
        TaskKind::Use { object, .. } | TaskKind::Repair { object } => shift(object),
        TaskKind::Cook {
            station, origin, ..
        } => {
            if *origin == Some(id) {
                *origin = None;
            }
            station.iter_mut().chain(origin.iter_mut()).for_each(shift);
        }
        TaskKind::Dishes { station, .. } => station.iter_mut().for_each(shift),
        TaskKind::Eat { seat, .. } | TaskKind::Phone { seat, .. } => {
            seat.iter_mut().for_each(shift)
        }
        TaskKind::Carry { from, to, .. } => {
            shift(from);
            shift(to);
        }
        TaskKind::Serve { from, .. } => shift(from),
        _ => {}
    }
}

impl HomeSnapshot {
    /// Where the lot's log stood when it was taken.
    pub(crate) fn lot_mark(&self) -> crate::lot::LogMark {
        self.lot_mark
    }
}

mod looks;
mod objects;
mod place;
mod rules;
mod spots;
mod undo;
mod walls;

pub(crate) use looks::*;

#[cfg(test)]
mod tests;
