//! The search itself: scratch space, node budget, stats and reconstructing the way.

use super::*;

/// How a search ended.
pub(super) enum Search {
    Found(Vec<(i32, i32)>),
    /// Every tile it could reach was tried.
    NoWay,
    /// It gave up at its cap.
    Capped,
}

/// The most tiles a search tries before it looks for a corridor (`NavGrid::find_path_with`).
pub const NODE_CAP: u64 = 30_000;
/// How many tiles kept off (`NavGrid::kept_off`) the way off them looks over at most: a lit
/// junction's box and carriageway are a few hundred.
pub(super) const WAY_OUT_MOST: usize = 4096;
/// The most a search kept to a corridor tries.
pub(super) const CORRIDOR_CAP: u64 = 4 * NODE_CAP;

/// Search state kept between searches: a big town is tens of thousands of tiles (and a big
/// world's roads thousands of nodes, `travel::Graph`), too many to allocate and clear for every
/// search. An entry counts only if stamped by the current search. By tile or node: the cost
/// there (`G`), where from (`C`); and the open list (`K`).
#[derive(Default)]
pub(crate) struct Scratch<G, C, K: Ord> {
    pub g: Vec<G>,
    pub came: Vec<C>,
    pub stamp: Vec<u32>,
    search: u32,
    pub open: BinaryHeap<Reverse<K>>,
}

impl<G: Clone + Default, C: Clone + Default, K: Ord> Scratch<G, C, K> {
    /// Gets ready for a search over `n` tiles or nodes; returns its stamp.
    pub fn start(&mut self, n: usize) -> u32 {
        if self.g.len() != n {
            *self = Self {
                g: vec![G::default(); n],
                came: vec![C::default(); n],
                stamp: vec![0; n],
                search: 0,
                open: BinaryHeap::new(),
            };
        }
        self.search = self.search.wrapping_add(1);
        if self.search == 0 {
            self.stamp.fill(0);
            self.search = 1;
        }
        self.open.clear();
        self.search
    }
}

thread_local! {
    pub(super) static SCRATCH: RefCell<Scratch<u32, u32, (u32, u32)>> = RefCell::default();
    pub(super) static STATS: RefCell<SearchStats> = RefCell::default();
}

/// What the searches on this thread cost since the last `take_search_stats`: for benchmarks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SearchStats {
    /// A* searches run (refused ones, unreachable or with a blocked goal, don't count).
    pub searches: u64,
    /// Tiles expanded over all of them, and by the biggest one.
    pub nodes: u64,
    pub max_nodes: u64,
    /// Of them, those that found no way or gave up at their cap, and the tiles they expanded.
    pub failed: u64,
    pub failed_nodes: u64,
}

impl SearchStats {
    pub(super) fn add(&mut self, nodes: u64, found: bool) {
        self.searches += 1;
        self.nodes += nodes;
        self.max_nodes = self.max_nodes.max(nodes);
        if !found {
            self.failed += 1;
            self.failed_nodes += nodes;
        }
        NODES.set(NODES.get() + nodes);
    }
}

thread_local! {
    static NODES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Tiles every search on this thread has expanded so far (never reset): for a budget per tick,
/// read it before and after (`find_path_with` counts its capped, corridor and full passes).
pub fn nodes_searched() -> u64 {
    NODES.get()
}

/// Counts `nodes` as searched (`nodes_searched`) without a search: for a search's answer used
/// again, so a budget per tick comes out as it did when it was searched for again.
pub(crate) fn charge_nodes(nodes: u64) {
    NODES.set(NODES.get() + nodes);
}

/// The search costs on this thread since the last call (and starts counting again).
pub fn take_search_stats() -> SearchStats {
    STATS.with_borrow_mut(std::mem::take)
}

pub(super) const NEIGHBOURS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

pub(super) fn heuristic(a: (i32, i32), b: (i32, i32)) -> u32 {
    let dx = (a.0 - b.0).unsigned_abs();
    let dz = (a.1 - b.1).unsigned_abs();
    ORTHO * dx.max(dz) + (DIAG - ORTHO) * dx.min(dz)
}

/// The path to `at`, back along `came` (by slot: the row-major key of the tile before).
pub(super) fn reconstruct(lot: &Lot, came: &[u32], mut at: (i32, i32)) -> Vec<(i32, i32)> {
    let width = lot.width as u32;
    let mut out = Vec::new();
    loop {
        out.push(at);
        match came[lot.tile_index(at.0, at.1)] {
            u32::MAX => break,
            prev => at = ((prev % width) as i32, (prev / width) as i32),
        }
    }
    out.reverse();
    out
}
