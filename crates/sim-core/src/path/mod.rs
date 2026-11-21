//! Grid pathfinding: 8-directional A* with no corner cutting, then string-pulling.
//!
//! Diagonal walls: a tile crossed by a diagonal wall or window can't be entered at all (it
//! blocks movement between its two halves), so residents walk around it on whole tiles. A
//! diagonal door tile is walked across like a doorway. A diagonal step passes the grid corner
//! shared by four tiles; it is refused when any diagonal ends at that corner (it would cross a
//! diagonal wall or clip its end), but allowed past a diagonal that keeps clear of the corner,
//! so a corridor between two parallel diagonal walls can be walked.
//!
//! Storeys: each storey is a block of `depth` rows of the lot (see `World::storeys`; the
//! ground is rows `0 .. depth`, the basement has a block of its own). Off the ground only rooms
//! have a floor to stand on. Stairs are walked up from their foot, step by step, and the top
//! step leads straight to the landing on the storey above.
//!
//! Gardens: the town is one grid, so nothing in it stops a resident cutting across the
//! neighbours' lawns. Given `Grounds`, a step onto another household's plot costs a long
//! detour, and the smoothed line keeps off the plots the path kept off: residents go round
//! by the street, and only cross a garden when there's no other way.

use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::lot::{CHUNK, Lot, OUTDOORS};

/// How a stair is shaped (content `stair`, see `ObjectDef::stair`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StairShape {
    /// A straight flight, one tile wide: entered from the tile in front of its bottom step.
    #[default]
    Flight,
    /// A spiral in a square footprint: its steps wind round the square, and it is walked onto
    /// from any side.
    Spiral,
}

/// The most steps a stair may have: a four-tile flight's four, or a spiral's four, with room to
/// spare.
pub const MAX_STEPS: usize = 8;

/// The way from step `a` to the next, `b`: one tile across (their signs).
fn along(a: (i32, i32), b: (i32, i32)) -> (i32, i32) {
    ((b.0 - a.0).signum(), (b.1 - a.1).signum())
}

/// One stair: the tiles walked, bottom first, and where the top step leads.
///
/// A flight is one tile wide and straight (`foot`, the tile in front of the bottom step, is the
/// only way on); a spiral winds round its square footprint and is walked onto from any side
/// (`open`). Either way the top step leads to `landing` on the storey above.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stair {
    steps: [(i32, i32); MAX_STEPS],
    /// How many steps it has (1..=`MAX_STEPS`).
    pub len: i32,
    pub foot: (i32, i32),
    /// Walked onto (and off) from any side (a spiral).
    pub open: bool,
    pub landing: (i32, i32),
}

impl Stair {
    /// A stair over `steps` (bottom first, at most `MAX_STEPS` of them: a longer list is cut
    /// short), entered at `foot` (ignored when `open`), leading to `landing` on the storey
    /// above.
    pub fn new(steps: &[(i32, i32)], foot: (i32, i32), open: bool, landing: (i32, i32)) -> Stair {
        debug_assert!(steps.len() <= MAX_STEPS, "a stair of {} steps", steps.len());
        let mut tiles = [(0, 0); MAX_STEPS];
        let n = steps.len().clamp(1, MAX_STEPS);
        tiles[..n].copy_from_slice(&steps[..n]);
        Stair {
            steps: tiles,
            len: n as i32,
            foot,
            open,
            landing,
        }
    }

    /// The step tiles, bottom first.
    pub fn tiles(&self) -> &[(i32, i32)] {
        &self.steps[..self.len as usize]
    }

    /// Step `i`'s tile.
    pub fn tile(&self, i: i32) -> (i32, i32) {
        self.steps[i as usize]
    }

    pub fn bottom(&self) -> (i32, i32) {
        self.steps[0]
    }

    pub fn top(&self) -> (i32, i32) {
        self.steps[self.len as usize - 1]
    }

    /// The way onto step `i` (from the step below it; step 0: the way it's climbed).
    pub fn dir(&self, i: i32) -> (i32, i32) {
        if self.len < 2 {
            return (0, 0);
        }
        let i = i.clamp(1, self.len - 1) as usize;
        along(self.steps[i - 1], self.steps[i])
    }

    /// The tile just past the top, where `landing` is above.
    pub fn out(&self) -> (i32, i32) {
        Stair::out_of(self.tiles())
    }

    /// The tile just past the last of `tiles`, the way they climb (nowhere for a one-step
    /// stair).
    pub fn out_of(tiles: &[(i32, i32)]) -> (i32, i32) {
        let Some((&last, before)) = tiles.split_last() else {
            return (0, 0);
        };
        let d = before.last().map_or((0, 0), |&b| along(b, last));
        (last.0 + d.0, last.1 + d.1)
    }

    /// Which step a tile is (0 the bottom one), if it is one.
    pub fn step(&self, x: i32, z: i32) -> Option<i32> {
        self.tiles()
            .iter()
            .position(|&t| t == (x, z))
            .map(|i| i as i32)
    }
}

/// The lot's storeys and stairs, for walking.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StairMap {
    /// Rows per storey; 0 when the lot has one storey.
    pub depth: i32,
    /// The block of rows holding the basement (storey -1); 0: no basement. Blocks before it are
    /// storeys 0, 1, …; blocks after it (storeys added later) are one storey lower than their
    /// place.
    pub basement: i32,
    /// The flights (set with `set_stairs`, which indexes them).
    pub stairs: Vec<Stair>,
    /// Every step by tile key (`tile_key`), sorted: `(key, flight, step)`; the first flight
    /// listed wins a tile two claim, as a scan of `stairs` would find it.
    steps: Vec<(u64, u32, i32)>,
    /// Where each flight's top and landing link to by tile key, sorted: `(key, order, to)`.
    links: Vec<(u64, u32, (i32, i32))>,
    /// Whether `steps` and `links` index `stairs` (else they're scanned).
    indexed: bool,
    /// Which `STAIR_BLOCK`-tile squares hold a step or a link's end, a bit each, row by row
    /// `block_cols` across: most tiles a search tries are nowhere near a stair, and are told so
    /// without a search of the index (half a big world's search, Snowdale's walks out of town,
    /// was looking tiles up in it). Empty with `block_cols` 0: none kept, every tile looked up.
    blocks: Vec<u64>,
    block_cols: u32,
}

/// The side of the squares `StairMap::blocks` marks, a power of two (tiles).
const STAIR_BLOCK_SHIFT: u32 = 3;
/// At most this many squares are marked: past it (stairs far apart in a lot as big as no town
/// is), every tile is looked up.
const STAIR_BLOCKS_MOST: u64 = 1 << 24;

/// No storeys, no stairs.
pub static NO_STAIRS: StairMap = StairMap {
    depth: 0,
    basement: 0,
    stairs: Vec::new(),
    steps: Vec::new(),
    links: Vec::new(),
    indexed: true,
    blocks: Vec::new(),
    block_cols: 0,
};

/// A tile's key in the stairs' index.
fn tile_key(x: i32, z: i32) -> u64 {
    (u64::from(z as u32) << 32) | u64::from(x as u32)
}

impl StairMap {
    /// The storey a row is on (0: the ground, -1: the basement).
    pub fn storey(&self, z: i32) -> i32 {
        if self.depth == 0 {
            return 0;
        }
        let block = z.div_euclid(self.depth);
        match self.basement {
            0 => block,
            b if block == b => -1,
            b if block > b => block - 1,
            _ => block,
        }
    }

    /// The first row of a storey.
    pub fn storey_row(&self, storey: i32) -> i32 {
        let block = match self.basement {
            _ if self.depth == 0 => 0,
            b if storey < 0 => b,
            b if b > 0 && storey >= b => storey + 1,
            _ => storey,
        };
        block * self.depth
    }

    /// Rows from a storey's to the same place on the one above.
    pub fn rise(&self, storey: i32) -> i32 {
        self.storey_row(storey + 1) - self.storey_row(storey)
    }

    /// The same place on the ground storey's rows.
    pub fn ground_z(&self, z: i32) -> i32 {
        if self.depth > 0 {
            z.rem_euclid(self.depth)
        } else {
            z
        }
    }

    /// `ground_z` for a point: where it is over the ground. Upstairs and the basement are rows
    /// away in the stacked grid, not that far to walk: how long a trip takes (`travel::Travel`)
    /// is asked from here.
    pub fn ground_pos(&self, p: [f32; 2]) -> [f32; 2] {
        if self.depth > 0 {
            [p[0], p[1].rem_euclid(self.depth as f32)]
        } else {
            p
        }
    }

    /// Sets the flights and indexes their steps and ends by tile, so `step` and `link` cost a
    /// lookup, not a scan of every flight (they're asked for every step a search tries).
    pub fn set_stairs(&mut self, stairs: Vec<Stair>) {
        let mut steps: Vec<(u64, u32, i32)> = Vec::new();
        let mut links: Vec<(u64, u32, (i32, i32))> = Vec::new();
        for (f, s) in stairs.iter().enumerate() {
            for (i, &(x, z)) in s.tiles().iter().enumerate() {
                steps.push((tile_key(x, z), f as u32, i as i32));
            }
            let top = s.top();
            links.push((tile_key(top.0, top.1), 2 * f as u32, s.landing));
            links.push((tile_key(s.landing.0, s.landing.1), 2 * f as u32 + 1, top));
        }
        // (By tile, then in the order a scan would meet them; the first one counts.)
        steps.sort_by_key(|&(k, f, _)| (k, f));
        steps.dedup_by_key(|e| e.0);
        links.sort_by_key(|&(k, o, _)| (k, o));
        links.dedup_by_key(|e| e.0);
        self.stairs = stairs;
        (self.blocks, self.block_cols) =
            Self::blocks_of(steps.iter().map(|e| e.0).chain(links.iter().map(|e| e.0)));
        self.steps = steps;
        self.links = links;
        self.indexed = true;
    }

    /// `blocks` and `block_cols` over the tiles with these keys (`tile_key`): none kept (every
    /// tile looked up) if one is off the grid's corner or they'd be too many.
    fn blocks_of(keys: impl Iterator<Item = u64> + Clone) -> (Vec<u64>, u32) {
        let at = |k: u64| ((k & 0xffff_ffff) as u32 as i32, (k >> 32) as u32 as i32);
        if keys.clone().any(|k| {
            let (x, z) = at(k);
            x < 0 || z < 0
        }) {
            return (Vec::new(), 0);
        }
        let (cols, rows) = keys.clone().fold((0u64, 0u64), |(c, r), k| {
            let (x, z) = at(k);
            (
                c.max((x as u64 >> STAIR_BLOCK_SHIFT) + 1),
                r.max((z as u64 >> STAIR_BLOCK_SHIFT) + 1),
            )
        });
        if cols == 0 || cols * rows > STAIR_BLOCKS_MOST {
            return (Vec::new(), 0);
        }
        let mut blocks = vec![0u64; (cols * rows).div_ceil(64) as usize];
        for k in keys {
            let (x, z) = at(k);
            let b = (z as u64 >> STAIR_BLOCK_SHIFT) * cols + (x as u64 >> STAIR_BLOCK_SHIFT);
            blocks[(b / 64) as usize] |= 1 << (b % 64);
        }
        (blocks, cols as u32)
    }

    /// Whether the tile may be a step or a link's end (`blocks`): false only where it surely
    /// isn't one.
    fn near_stairs(&self, x: i32, z: i32) -> bool {
        if self.block_cols == 0 {
            return true;
        }
        if x < 0 || z < 0 {
            return false;
        }
        let (bx, bz) = (x as u64 >> STAIR_BLOCK_SHIFT, z as u64 >> STAIR_BLOCK_SHIFT);
        if bx >= u64::from(self.block_cols) {
            return false;
        }
        let b = bz * u64::from(self.block_cols) + bx;
        self.blocks
            .get((b / 64) as usize)
            .is_some_and(|w| w & (1 << (b % 64)) != 0)
    }

    /// The flight a tile is a step of, and which step.
    pub fn step(&self, x: i32, z: i32) -> Option<(&Stair, i32)> {
        if self.stairs.is_empty() {
            return None;
        }
        if !self.indexed {
            return self
                .stairs
                .iter()
                .find_map(|s| s.step(x, z).map(|i| (s, i)));
        }
        if !self.near_stairs(x, z) {
            return None;
        }
        let k = tile_key(x, z);
        let at = self.steps.binary_search_by_key(&k, |e| e.0).ok()?;
        let (_, f, i) = self.steps[at];
        Some((&self.stairs[f as usize], i))
    }

    /// Where a stair links to from this tile: the landing from the top step, or back.
    pub fn link(&self, x: i32, z: i32) -> Option<(i32, i32)> {
        if self.stairs.is_empty() {
            return None;
        }
        if !self.indexed {
            return self.stairs.iter().find_map(|s| {
                if s.top() == (x, z) {
                    Some(s.landing)
                } else if s.landing == (x, z) {
                    Some(s.top())
                } else {
                    None
                }
            });
        }
        if !self.near_stairs(x, z) {
            return None;
        }
        let k = tile_key(x, z);
        let at = self.links.binary_search_by_key(&k, |e| e.0).ok()?;
        Some(self.links[at].2)
    }

    /// How far apart two points are for someone walking (storeys counted as a few metres).
    pub fn distance(&self, a: [f32; 2], b: [f32; 2]) -> f32 {
        if self.depth == 0 {
            return (a[0] - b[0]).hypot(a[1] - b[1]);
        }
        let d = self.depth as f32;
        let (ka, kb) = (
            self.storey(a[1].floor() as i32),
            self.storey(b[1].floor() as i32),
        );
        let flat = (a[0] - b[0]).hypot(a[1].rem_euclid(d) - b[1].rem_euclid(d));
        flat + (ka - kb).abs() as f32 * STOREY_WALK
    }
}

mod grounds;
mod log;
mod nav;
mod reach;
mod search;

pub use grounds::*;
pub use log::*;
pub use nav::*;
pub use reach::*;
pub use search::*;

#[cfg(test)]
mod tests;
