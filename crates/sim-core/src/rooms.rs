//! Rooms and how good they are ("the house matters").
//!
//! A room is an enclosed area of the lot (`Lot::compute_rooms`); each plot's outdoor tiles are
//! scored as its garden. Room ids change whenever walls do, so nothing here is saved: rooms are
//! recomputed from the lot, the objects and the per-tile dirt (`World::refresh_rooms`).

use crate::content::TagMask;
use crate::lot::{Edge, EdgeRef, OUTDOORS};
use crate::world::World;

/// Scores of a room, each 0..1, and the weighted `overall` (content `roomRules.weights`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RoomScores {
    pub size: f32,
    pub light: f32,
    pub decor: f32,
    pub clean: f32,
    pub function: f32,
    pub overall: f32,
}

impl RoomScores {
    /// The factors in order (size, light, decor, cleanliness, function).
    pub fn factors(&self) -> [f32; 5] {
        [self.size, self.light, self.decor, self.clean, self.function]
    }

    /// The weakest factor (index into `factors`).
    pub fn weakest(&self) -> usize {
        let f = self.factors();
        (0..f.len())
            .min_by(|&a, &b| f[a].total_cmp(&f[b]))
            .unwrap_or(0)
    }
}

/// Factor names, in the order of `RoomScores::factors` (ids for the UI and saves).
pub const FACTORS: [&str; 5] = ["size", "light", "decor", "clean", "function"];

#[derive(Debug, Clone, PartialEq)]
pub struct RoomInfo {
    /// Lot room id (`OUTDOORS` for a plot's garden).
    pub id: u16,
    pub plot: Option<u32>,
    pub garden: bool,
    /// Room kind (index into `Content::room_kinds`); None: nothing tells what it's for.
    pub kind: Option<usize>,
    /// Things of two exclusive kinds stand in it (a bed in the kitchen).
    pub mixed: bool,
    /// The first essential of its kind nothing in it offers (index into the kind's `essentials`).
    pub missing: Option<usize>,
    /// Indoor tiles (a tile split by a diagonal wall counts half).
    pub tiles: f32,
    pub windows: u16,
    pub doors: u16,
    pub lamps: u16,
    /// Floor its lamps light at night, in all (tiles; `ObjectDef::lamp`).
    pub lit: f32,
    /// A wall of it (plain or with a window) is an outside wall, above ground: somewhere a
    /// window could be. A room without (an inside bathroom, a hall, anything in the basement)
    /// gets its daylight score from its lamps too.
    pub outside: bool,
    /// Decor points of what stands in it.
    pub decor: f32,
    /// Share of its floor and wall faces with a chosen covering.
    pub coverings: f32,
    /// Mean dirt of its tiles (0..1).
    pub dirt: f32,
    /// Centre of its tiles (for labels and the camera).
    pub centre: [f32; 2],
    /// Tags offered by what stands in it.
    pub tags: TagMask,
    pub scores: RoomScores,
}

#[derive(Default)]
struct Acc {
    tiles: f32,
    sx: f32,
    sz: f32,
    dirt: f32,
    /// Tiles and their weights, in the order their dirt was added up.
    cells: Vec<(u32, f32)>,
    windows: u16,
    doors: u16,
    outside: bool,
    /// What stands in it.
    things: Things,
    floors: f32,
    faces: f32,
    covered_faces: f32,
    first: (i32, i32),
}

/// What it takes to rescore cleanliness between structure changes without finding the rooms
/// again (see `World::refresh_rooms`): which tiles' dirt counts towards each room and garden,
/// in the order `compute` adds it up (so scores come out exactly the same), which tiles take
/// each one's score, and which had a mess since they were last scored.
///
/// Each room's lists are kept under a handle that stays while other rooms come and go (rooms
/// found again round a wall: `World::refresh_rooms_near`); `handle` gives each room's.
#[derive(Debug, Clone, Default)]
pub(crate) struct RoomTiles {
    /// Per room (index into `World::rooms`): its handle (index into the lists below).
    handle: Vec<u32>,
    /// Per handle: tiles and weights.
    cells: Vec<Vec<(u32, f32)>>,
    /// Per handle: the tiles whose `World::room_score` is its overall score.
    scored: Vec<Vec<u32>>,
    /// Per tile: the handles of the rooms its dirt counts towards (`NONE`: none).
    of_tile: Vec<[u32; 2]>,
    /// Per handle: its dirt changed since it was scored.
    stale: Vec<bool>,
    /// Handles no room has.
    free: Vec<u32>,
    /// `rescore_things`'s working lists, kept here so a rescore allocates nothing: rooms by lot
    /// id, rooms by plot, and the things each room holds.
    by_id: Vec<usize>,
    by_plot: Vec<usize>,
    things: Vec<score::Things>,
}

/// What `RoomTiles::summary` gives: per room its tiles (weights as bits) and the tiles taking its
/// score, and per tile the rooms its dirt counts towards.
pub(crate) type RoomTilesSummary = (Vec<(Vec<(u32, u32)>, Vec<u32>)>, Vec<Vec<u32>>);

const NONE: u32 = u32::MAX;
/// No room sets a tile's score (`World::refresh_rooms`, `RoomTiles::new`).
pub(crate) const NO_ROOM: u32 = NONE;

impl RoomTiles {
    /// `cells` per room from `compute`; `source` per tile slot: the room that sets its score
    /// (`NO_ROOM`: none).
    pub fn new(cells: Vec<Vec<(u32, f32)>>, source: &[u32]) -> Self {
        let mut scored = vec![Vec::new(); cells.len()];
        for (i, &r) in source.iter().enumerate() {
            if r != NO_ROOM {
                scored[r as usize].push(i as u32);
            }
        }
        let mut of_tile = vec![[NONE; 2]; source.len()];
        for (r, list) in cells.iter().enumerate() {
            for &(i, _) in list {
                let slot = &mut of_tile[i as usize];
                let k = usize::from(slot[0] != NONE);
                slot[k] = r as u32;
            }
        }
        let stale = vec![false; cells.len()];
        Self {
            handle: (0..cells.len() as u32).collect(),
            cells,
            scored,
            of_tile,
            stale,
            free: Vec::new(),
            by_id: Vec::new(),
            by_plot: Vec::new(),
            things: Vec::new(),
        }
    }

    /// Tile `i`'s dirt changed.
    pub fn touch(&mut self, i: usize) {
        for r in self.of_tile.get(i).into_iter().flatten() {
            if *r != NONE {
                self.stale[*r as usize] = true;
            }
        }
    }

    /// Scores cleanliness again where there was a mess, and updates the score per tile.
    pub fn rescore(
        &mut self,
        w: &mut [RoomInfo],
        rules: &crate::content::RoomRules,
        dirt: &[f32],
        room_score: &mut [f32],
    ) {
        for (r, room) in w.iter_mut().enumerate() {
            let h = self.handle[r] as usize;
            if !std::mem::take(&mut self.stale[h]) {
                continue;
            }
            let sum = self.cells[h].iter().fold(0.0, |s, &(i, weight)| {
                s + dirt.get(i as usize).copied().unwrap_or(0.0) * weight
            });
            score_dirt(room, sum, rules);
            for &i in &self.scored[h] {
                room_score[i as usize] = room.scores.overall;
            }
        }
    }

    /// The handles of the rooms, in order (see `set_handles`).
    pub fn take_handles(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.handle)
    }

    /// The handles of the rooms, in the order the rooms now have.
    pub fn set_handles(&mut self, handles: Vec<u32>) {
        self.handle = handles;
    }

    /// Room `h` went (or is found again): its lists go.
    pub fn drop_room(&mut self, h: u32) {
        let cells = std::mem::take(&mut self.cells[h as usize]);
        for &(i, _) in &cells {
            if let Some(slot) = self.of_tile.get_mut(i as usize) {
                for r in slot.iter_mut().filter(|r| **r == h) {
                    *r = NONE;
                }
            }
        }
        self.scored[h as usize] = Vec::new();
        self.stale[h as usize] = false;
        self.free.push(h);
    }

    /// A room found (again) with its tiles: its handle.
    pub fn add_room(&mut self, cells: Vec<(u32, f32)>, scored: Vec<u32>) -> u32 {
        let h = self.free.pop().unwrap_or_else(|| {
            self.cells.push(Vec::new());
            self.scored.push(Vec::new());
            self.stale.push(false);
            self.cells.len() as u32 - 1
        });
        for &(i, _) in &cells {
            let slot = &mut self.of_tile[i as usize];
            let k = usize::from(slot[0] != NONE);
            slot[k] = h;
        }
        self.cells[h as usize] = cells;
        self.scored[h as usize] = scored;
        self.stale[h as usize] = false;
        h
    }

    /// Per room (in order): its tiles and weights, the tiles taking its score (sorted), and per
    /// tile the rooms (indices, sorted) its dirt counts towards: what these lists say, however
    /// they're kept (for `World::check_rooms`).
    pub fn summary(&self) -> RoomTilesSummary {
        let mut index = vec![NONE; self.cells.len()];
        for (r, &h) in self.handle.iter().enumerate() {
            index[h as usize] = r as u32;
        }
        let rooms = self
            .handle
            .iter()
            .map(|&h| {
                let cells = self.cells[h as usize]
                    .iter()
                    .map(|&(i, w)| (i, w.to_bits()))
                    .collect();
                let mut scored = self.scored[h as usize].clone();
                scored.sort_unstable();
                (cells, scored)
            })
            .collect();
        let tiles = self
            .of_tile
            .iter()
            .map(|slot| {
                let mut rs: Vec<u32> = slot
                    .iter()
                    .filter(|&&h| h != NONE)
                    .map(|&h| index[h as usize])
                    .collect();
                rs.sort_unstable();
                rs
            })
            .collect();
        (rooms, tiles)
    }

    /// The per-tile list follows the lot's slots (chunks allocated since).
    pub fn grow(&mut self, slots: usize) {
        if self.of_tile.len() < slots {
            self.of_tile.resize(slots, [NONE; 2]);
        }
    }
}

mod opinions;
mod score;

pub(crate) use opinions::*;
pub(crate) use score::*;

/// Thought subject for a room: its kind (garden 0, kind + 1, or 63 for a room nothing marks)
/// times 8 plus a factor (`FACTORS`).
pub fn room_subject(room: &RoomInfo, factor: usize) -> usize {
    let kind = if room.garden {
        0
    } else {
        room.kind.map_or(63, |k| k + 1)
    };
    kind * 8 + factor.min(7)
}
