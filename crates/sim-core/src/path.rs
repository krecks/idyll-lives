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

/// One flight of stairs, one tile wide: entered at `bottom` from `foot`, climbed along `dir`
/// for `len` steps; the top step leads to `landing` on the storey above.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stair {
    pub foot: (i32, i32),
    pub bottom: (i32, i32),
    pub dir: (i32, i32),
    pub len: i32,
    pub landing: (i32, i32),
}

impl Stair {
    pub fn top(&self) -> (i32, i32) {
        (
            self.bottom.0 + self.dir.0 * (self.len - 1),
            self.bottom.1 + self.dir.1 * (self.len - 1),
        )
    }

    /// Which step a tile is (0 the bottom one), if it is one.
    pub fn step(&self, x: i32, z: i32) -> Option<i32> {
        (0..self.len).find(|&i| {
            (
                self.bottom.0 + self.dir.0 * i,
                self.bottom.1 + self.dir.1 * i,
            ) == (x, z)
        })
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
}

/// No storeys, no stairs.
pub static NO_STAIRS: StairMap = StairMap {
    depth: 0,
    basement: 0,
    stairs: Vec::new(),
    steps: Vec::new(),
    links: Vec::new(),
    indexed: true,
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
            for i in 0..s.len {
                let (x, z) = (s.bottom.0 + s.dir.0 * i, s.bottom.1 + s.dir.1 * i);
                steps.push((tile_key(x, z), f as u32, i));
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
        self.steps = steps;
        self.links = links;
        self.indexed = true;
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

/// Going up or down a storey counts as this many metres of walking (for choosing what to do).
pub const STOREY_WALK: f32 = 4.0;

const ORTHO: u32 = 10;
const DIAG: u32 = 14;
/// Extra cost of stepping onto a tile of someone else's plot: twenty tiles of walking.
const TRESPASS: u32 = ORTHO * 20;
/// Extra cost of stepping straight from one plot into another beside a garden (`Grounds::fenced`):
/// three hundred tiles, so a garden is entered from the street at its front wherever there is one,
/// and only crossed into where there isn't. Gardens are kept apart all round, though the renderer
/// draws their fences and hedges (`shared/street.ts`) only from the house's front back, and none
/// round some: next door is by the street, as a visitor would go.
const OVER_THE_FENCE: u32 = ORTHO * 300;

/// Which plots one resident may walk across (their home, public places, where they're going);
/// the rest are other people's gardens, kept off unless there's no other way.
pub struct Grounds<'a> {
    /// Plot per tile of the ground storey, row by row (upper storeys are over the same ground;
    /// `NO_PLOT` off the plots).
    pub tile_plot: &'a [u16],
    /// Per plot (by id): whether this resident may walk across it.
    pub open: Vec<bool>,
    /// Per plot (by id): whether its sides are fenced off from the plots beside it (a garden; a
    /// public place isn't).
    pub fenced: Vec<bool>,
}

impl Grounds<'_> {
    /// The plot a tile is on (on any storey).
    pub fn plot(&self, nav: &NavGrid, x: i32, z: i32) -> Option<u32> {
        let z = nav.stairs.ground_z(z);
        if x < 0 || z < 0 || x >= nav.lot.width as i32 {
            return None;
        }
        self.tile_plot
            .get(z as usize * nav.lot.width + x as usize)
            .filter(|&&p| p != NO_PLOT)
            .map(|&p| u32::from(p))
    }

    /// Whether plot `p` (a tile's) is one this resident shouldn't walk across.
    fn foreign(&self, p: Option<u32>) -> bool {
        p.is_some_and(|p| !self.open.get(p as usize).copied().unwrap_or(true))
    }

    /// A step from plot `a` straight into plot `b` (tiles'), over the fence between them, open
    /// or not: a garden's side or back, not between two public places.
    fn over_the_fence(&self, a: Option<u32>, b: Option<u32>) -> bool {
        let fenced = |p: u32| self.fenced.get(p as usize).copied().unwrap_or(false);
        matches!((a, b), (Some(a), Some(b)) if a != b && (fenced(a) || fenced(b)))
    }
}

/// No plot (`Grounds::tile_plot`, `World::tile_plot`).
pub const NO_PLOT: u16 = u16::MAX;

/// Read-only view of what is walkable: lot edges plus tiles blocked by objects, storeys and stairs.
#[derive(Clone, Copy)]
pub struct NavGrid<'a> {
    pub lot: &'a Lot,
    pub blocked: &'a [bool],
    pub stairs: &'a StairMap,
}

impl NavGrid<'_> {
    /// Whether a resident can stand on the tile: on the lot, no furniture, no diagonal wall;
    /// off the ground (upstairs or in the basement), only in a room (where there's a floor).
    pub fn tile_free(&self, x: i32, z: i32) -> bool {
        // (A tile whose chunk isn't allocated is open air off the ground: nobody stands there.)
        self.lot.in_bounds(x, z)
            && self.lot.allocated(x, z)
            && !self.blocked[self.lot.tile_index(x, z)]
            && !self.lot.diag_blocks(x, z)
            && (self.stairs.depth == 0
                || z < self.stairs.depth
                || self.lot.room_at(x, z) != OUTDOORS)
    }

    /// Whether a single step from `a` to the neighbouring tile `b` is allowed.
    pub fn can_step(&self, ax: i32, az: i32, bx: i32, bz: i32) -> bool {
        let (dx, dz) = (bx - ax, bz - az);
        if !self.tile_free(bx, bz) {
            return false;
        }
        // Stairs are walked straight up and down, entered at the foot.
        if !self.stairs.stairs.is_empty() {
            let from = self.stairs.step(ax, az);
            let to = self.stairs.step(bx, bz);
            let along = |s: &Stair| (dx, dz) == s.dir || (dx, dz) == (-s.dir.0, -s.dir.1);
            match (from, to) {
                (None, None) => {}
                (Some((a, _)), Some((b, _))) => {
                    if a != b || !along(a) {
                        return false;
                    }
                }
                (Some((s, i)), None) => {
                    if i != 0 || (bx, bz) != s.foot {
                        return false;
                    }
                }
                (None, Some((s, i))) => {
                    if i != 0 || (ax, az) != s.foot {
                        return false;
                    }
                }
            }
        }
        if dx == 0 || dz == 0 {
            return self.lot.edge_walkable(ax, az, bx, bz);
        }
        // Diagonal: the step passes the grid corner shared by the four tiles. No diagonal wall
        // may end there, and both L-shaped routes must be open, so residents never clip corners.
        let (cx, cz) = (ax.max(bx), az.max(bz));
        let clear = |x: i32, z: i32| !self.lot.diag_touches(x, z, cx, cz);
        // The tiles beside the step: no furniture; a diagonal wall across them is fine as long
        // as it keeps clear of the corner (checked above for all four tiles).
        let beside =
            |x: i32, z: i32| self.lot.in_bounds(x, z) && !self.blocked[self.lot.tile_index(x, z)];
        clear(ax, az)
            && clear(bx, bz)
            && clear(ax + dx, az)
            && clear(ax, az + dz)
            && beside(ax + dx, az)
            && beside(ax, az + dz)
            && self.lot.edge_walkable(ax, az, ax + dx, az)
            && self.lot.edge_walkable(ax + dx, az, bx, bz)
            && self.lot.edge_walkable(ax, az, ax, az + dz)
            && self.lot.edge_walkable(ax, az + dz, bx, bz)
    }

    /// A* from `start` to `goal` (tile coordinates). The start tile may be blocked.
    pub fn find_path(&self, start: (i32, i32), goal: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        self.find_path_with(start, goal, None)
    }

    /// `find_path`, round other people's gardens when there are `grounds` to mind. (Their
    /// cost only adds to a step's, so the heuristic still never overestimates.)
    ///
    /// A search stops after `NODE_CAP` tiles. Past that the goal is far or behind a long way
    /// round: a coarse search over the lot's chunks finds the chunks the way goes through, and
    /// the search runs again kept to them and the chunks round them. Only if that finds nothing
    /// does it search the whole lot, as before, so a way that exists is always found.
    pub fn find_path_with(
        &self,
        start: (i32, i32),
        goal: (i32, i32),
        grounds: Option<&Grounds>,
    ) -> Option<Vec<(i32, i32)>> {
        let lot = self.lot;
        if !lot.in_bounds(start.0, start.1) || !self.tile_free(goal.0, goal.1) {
            return None;
        }
        if start == goal {
            return Some(vec![start]);
        }
        match self.search(start, goal, grounds, NODE_CAP, None) {
            Search::Found(path) => return Some(path),
            Search::NoWay => return None,
            Search::Capped => {}
        }
        if let Some(corridor) = self.corridor(start, goal)
            && let Search::Found(path) =
                self.search(start, goal, grounds, CORRIDOR_CAP, Some(&corridor))
        {
            return Some(path);
        }
        match self.search(start, goal, grounds, u64::MAX, None) {
            Search::Found(path) => Some(path),
            _ => None,
        }
    }

    /// One A* search from `start` to `goal`, giving up after `cap` tiles, kept to the chunks
    /// marked in `corridor` if there is one.
    fn search(
        &self,
        start: (i32, i32),
        goal: (i32, i32),
        grounds: Option<&Grounds>,
        cap: u64,
        corridor: Option<&[bool]>,
    ) -> Search {
        let lot = self.lot;
        SCRATCH.with_borrow_mut(|scratch| {
            // (Every storey's rows: the lot's whole depth, by slot.)
            let search = scratch.start(lot.tile_slots());
            let Scratch {
                g,
                came,
                stamp,
                open,
                ..
            } = scratch;
            // Costs from earlier searches don't count.
            let cost_to = |g: &[u32], stamp: &[u32], i: usize| {
                if stamp[i] == search { g[i] } else { u32::MAX }
            };
            // Costs, where from and stamps by slot; the open list by row-major key (`z * width +
            // x`), so ties break the same way however the tiles are stored.
            let width = lot.width as u32;
            let key = |(x, z): (i32, i32)| z as u32 * width + x as u32;
            let s = lot.tile_index(start.0, start.1);
            let goal_i = lot.tile_index(goal.0, goal.1);
            (g[s], came[s], stamp[s]) = (0, u32::MAX, search);
            let heuristic = |a: (i32, i32), b: (i32, i32)| heuristic(self.flat(a), self.flat(b));
            open.push(Reverse((heuristic(start, goal), key(start))));

            let mut expanded = 0u64;
            let found = loop {
                let Some(Reverse((f, k))) = open.pop() else {
                    break Search::NoWay;
                };
                let (x, z) = ((k % width) as i32, (k / width) as i32);
                let i = lot.tile_index(x, z);
                if i == goal_i {
                    break Search::Found(reconstruct(lot, came, (x, z)));
                }
                if f > g[i] + heuristic((x, z), goal) {
                    continue; // stale heap entry
                }
                expanded += 1;
                if expanded > cap {
                    break Search::Capped;
                }
                let link = self
                    .stairs
                    .link(x, z)
                    .filter(|&(lx, lz)| self.tile_free(lx, lz));
                let here = grounds.and_then(|gr| gr.plot(self, x, z));
                for (dx, dz) in NEIGHBOURS
                    .into_iter()
                    .chain(link.map(|(lx, lz)| (lx - x, lz - z)))
                {
                    let (nx, nz) = (x + dx, z + dz);
                    let linked = Some((nx, nz)) == link;
                    if !linked && ((dx.abs() > 1 || dz.abs() > 1) || !self.can_step(x, z, nx, nz)) {
                        continue;
                    }
                    if corridor.is_some_and(|c| !c[lot.chunk_at(nx, nz)]) {
                        continue;
                    }
                    let ni = lot.tile_index(nx, nz);
                    let mut cost = g[i]
                        + if dx != 0 && dz != 0 && !linked {
                            DIAG
                        } else {
                            ORTHO
                        };
                    if let Some(gr) = grounds {
                        let there = gr.plot(self, nx, nz);
                        if gr.foreign(there) {
                            cost += TRESPASS;
                        }
                        if !linked && gr.over_the_fence(here, there) {
                            cost += OVER_THE_FENCE;
                        }
                    }
                    if cost < cost_to(g, stamp, ni) {
                        (g[ni], came[ni], stamp[ni]) = (cost, k, search);
                        open.push(Reverse((cost + heuristic((nx, nz), goal), key((nx, nz)))));
                    }
                }
            };
            STATS.with_borrow_mut(|s| s.add(expanded));
            found
        })
    }

    /// The chunks a way from `start` to `goal` may go through, and the chunks round them: a
    /// coarse A* over the lot's chunks, from one to the next wherever a step crosses between
    /// them (or a stair links them). None if the coarse search finds no way.
    fn corridor(&self, start: (i32, i32), goal: (i32, i32)) -> Option<Vec<bool>> {
        let lot = self.lot;
        let (cols, rows) = lot.chunk_grid();
        let flat = |c: usize| {
            let [x0, z0, ..] = lot.chunk_rect(c);
            (x0 / CHUNK as i32, self.stairs.ground_z(z0) / CHUNK as i32)
        };
        let (from, to) = (lot.chunk_at(start.0, start.1), lot.chunk_at(goal.0, goal.1));
        let target = flat(to);
        let h = |c: usize| heuristic(flat(c), target);
        let mut g = std::collections::HashMap::new();
        let mut came: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        let mut open = BinaryHeap::new();
        g.insert(from, 0u32);
        open.push(Reverse((h(from), from)));
        // Chunks a stair links each chunk to.
        let mut linked: Vec<(usize, usize)> = Vec::new();
        for s in &self.stairs.stairs {
            let (t, l) = (s.top(), s.landing);
            if self.tile_free(t.0, t.1) && self.tile_free(l.0, l.1) {
                let (a, b) = (lot.chunk_at(t.0, t.1), lot.chunk_at(l.0, l.1));
                linked.push((a, b));
                linked.push((b, a));
            }
        }
        let mut found = false;
        while let Some(Reverse((f, c))) = open.pop() {
            if c == to {
                found = true;
                break;
            }
            let gc = g[&c];
            if f > gc + h(c) {
                continue;
            }
            let (cx, cz) = ((c % cols) as i32, (c / cols) as i32);
            let next = NEIGHBOURS
                .into_iter()
                .map(|(dx, dz)| (cx + dx, cz + dz))
                .filter(|&(nx, nz)| {
                    nx >= 0 && nz >= 0 && (nx as usize) < cols && (nz as usize) < rows
                })
                .map(|(nx, nz)| nz as usize * cols + nx as usize)
                .filter(|&n| self.crosses(c, n))
                .chain(linked.iter().filter(|l| l.0 == c).map(|l| l.1));
            for n in next {
                let cost = gc + heuristic(flat(c), flat(n)).max(ORTHO);
                if g.get(&n).is_none_or(|&old| cost < old) {
                    g.insert(n, cost);
                    came.insert(n, c);
                    open.push(Reverse((cost + h(n), n)));
                }
            }
        }
        if !found {
            return None;
        }
        let mut corridor = vec![false; cols * rows];
        let mut c = to;
        loop {
            let (cx, cz) = (c % cols, c / cols);
            for nz in cz.saturating_sub(1)..=(cz + 1).min(rows - 1) {
                for nx in cx.saturating_sub(1)..=(cx + 1).min(cols - 1) {
                    corridor[nz * cols + nx] = true;
                }
            }
            match came.get(&c) {
                Some(&p) => c = p,
                None => break,
            }
        }
        // (Both ends' storeys at stairs: the chunks a stair links to the corridor.)
        for &(a, b) in &linked {
            if corridor[a] {
                corridor[b] = true;
            }
        }
        Some(corridor)
    }

    /// Whether a single step leads from a tile of chunk `a` into the neighbouring chunk `b`.
    fn crosses(&self, a: usize, b: usize) -> bool {
        let lot = self.lot;
        if !lot.chunk_allocated(a) || !lot.chunk_allocated(b) {
            return false;
        }
        let [ax0, az0, ax1, az1] = lot.chunk_rect(a);
        let [bx0, bz0, bx1, bz1] = lot.chunk_rect(b);
        // The tiles of `a` next to `b`.
        let (x0, x1) = ((bx0 - 1).max(ax0), bx1.min(ax1 - 1) + 1);
        let (z0, z1) = ((bz0 - 1).max(az0), bz1.min(az1 - 1) + 1);
        (z0..z1).any(|z| {
            (x0..x1).any(|x| {
                self.tile_free(x, z)
                    && NEIGHBOURS.into_iter().any(|(dx, dz)| {
                        let (nx, nz) = (x + dx, z + dz);
                        nx >= bx0
                            && nx < bx1
                            && nz >= bz0
                            && nz < bz1
                            && self.can_step(x, z, nx, nz)
                    })
            })
        })
    }

    /// A tile as if on the ground storey (so distances between storeys stay honest).
    fn flat(&self, (x, z): (i32, i32)) -> (i32, i32) {
        (x, self.stairs.ground_z(z))
    }

    /// Tiles a resident can walk to from `(x, z)` in one step: neighbours, and up or down stairs.
    pub fn neighbours(&self, x: i32, z: i32) -> impl Iterator<Item = (i32, i32)> + '_ {
        let link = self
            .stairs
            .link(x, z)
            .filter(|&(lx, lz)| self.tile_free(lx, lz));
        NEIGHBOURS
            .into_iter()
            .map(move |(dx, dz)| (x + dx, z + dz))
            .filter(move |&(nx, nz)| self.can_step(x, z, nx, nz))
            .chain(link)
    }

    /// Removes unnecessary waypoints where a straight line is walkable. Returns tile centres.
    pub fn smooth(&self, path: &[(i32, i32)]) -> Vec<[f32; 2]> {
        self.smooth_with(path, None)
    }

    /// `smooth`, with no straight line across a garden the path went round (`line_walkable_with`).
    pub fn smooth_with(&self, path: &[(i32, i32)], grounds: Option<&Grounds>) -> Vec<[f32; 2]> {
        let centre = |(x, z): (i32, i32)| [x as f32 + 0.5, z as f32 + 0.5];
        let mut out = Vec::with_capacity(path.len());
        let Some(&first) = path.first() else {
            return out;
        };
        let mut anchor = 0;
        out.push(centre(first));
        let mut i = 2;
        while i < path.len() {
            if !self.line_walkable_with(centre(path[anchor]), centre(path[i]), grounds) {
                anchor = i - 1;
                out.push(centre(path[anchor]));
            }
            i += 1;
        }
        if path.len() > 1 {
            out.push(centre(path[path.len() - 1]));
        }
        out
    }

    /// Samples the segment and checks every tile transition it makes.
    pub fn line_walkable(&self, a: [f32; 2], b: [f32; 2]) -> bool {
        self.line_walkable_with(a, b, None)
    }

    /// `line_walkable`, and with `grounds`, the line keeps off other people's plots, except
    /// the one either end is on (someone already in a garden walks straight out of it), and
    /// never goes straight from one plot into another (a path that had to keeps that step).
    pub fn line_walkable_with(&self, a: [f32; 2], b: [f32; 2], grounds: Option<&Grounds>) -> bool {
        let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
        let steps = ((dx.abs().max(dz.abs())) / 0.1).ceil().max(1.0) as i32;
        let (mut tx, mut tz) = (a[0].floor() as i32, a[1].floor() as i32);
        let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
        // (The plots the ends are on: crossing them is never trespassing.)
        let ends = grounds.map(|g| [tile(a), tile(b)].map(|(x, z)| g.plot(self, x, z)));
        for s in 1..=steps {
            let t = s as f32 / steps as f32;
            let (nx, nz) = (
                (a[0] + dx * t).floor() as i32,
                (a[1] + dz * t).floor() as i32,
            );
            if (nx, nz) != (tx, tz) {
                if (nx - tx).abs() > 1 || (nz - tz).abs() > 1 || !self.can_step(tx, tz, nx, nz) {
                    return false;
                }
                if let (Some(g), Some(ends)) = (grounds, ends) {
                    let (from, to) = (g.plot(self, tx, tz), g.plot(self, nx, nz));
                    let trespass = g.foreign(to) && !ends.contains(&to);
                    if trespass || g.over_the_fence(from, to) {
                        return false;
                    }
                }
                (tx, tz) = (nx, nz);
            }
        }
        true
    }
}

/// Which walkable tiles can reach which, in two levels: per storage chunk (`lot::CHUNK`), the
/// tiles joined by steps inside it (a label per tile); over the town, which chunks' parts join
/// across chunk sides, diagonal corners and stairs (a union-find over the parts). A goal in
/// another region has no path, so a search for it is refused before it can flood the town
/// (P3-08).
///
/// A change to walking is marked by the rectangle it touched (`mark`); `refresh` then labels
/// only the chunks it reached again (and the joins of the chunks round them) and joins the parts
/// up again, which costs what a few chunks cost, not the town. `mark_all` (or a lot of another
/// size) labels every chunk, as a fresh `build` does.
///
/// Labels are only meaningful against the `NavGrid` they were refreshed for; `World` refreshes
/// them when the navigation version changes.
#[derive(Default, Clone)]
pub struct Reach {
    /// Per slot: its part within its chunk (`NO_PART`: nobody can stand there).
    part: Vec<u16>,
    /// Per chunk: how many parts it has.
    parts: Vec<u16>,
    /// Per chunk: plain ground (every tile free, nothing built, no stairs): one part, joined
    /// to its plain neighbours without trying the steps.
    plain: Vec<bool>,
    /// Per chunk: the joins from its parts to other chunks' parts, `(part, chunk, part)`, sorted.
    joins: Vec<Vec<(u16, u32, u16)>>,
    /// Per chunk: the index of its first part in `region` (refreshed with the joins).
    first: Vec<u32>,
    /// Per part (by `first`): its region.
    region: Vec<u32>,
    regions: u32,
    /// The chunk grid it was built over (`Lot::chunk_grid`): chunks across and down.
    grid: (usize, usize),
    /// Chunks to label again (`mark`), and whether all of them are.
    dirty: Vec<u32>,
    all_dirty: bool,
}

const NO_PART: u16 = u16::MAX;

impl Reach {
    /// Labels the walkable regions of `nav` from scratch.
    pub fn build(nav: &NavGrid) -> Reach {
        let mut reach = Reach {
            all_dirty: true,
            ..Default::default()
        };
        reach.refresh(nav);
        reach
    }

    /// Notes that walking changed within tiles `[x0, x1) × [z0, z1)`: the chunks within a tile of
    /// it are labelled again at the next `refresh` (a step's corner tiles count for it).
    pub fn mark(&mut self, lot: &Lot, [x0, z0, x1, z1]: [i32; 4]) {
        if self.all_dirty || lot.chunk_grid() != self.grid {
            self.all_dirty = true;
            return;
        }
        let (cols, rows) = self.grid;
        let c = |v: i32, n: usize| (v.max(0) as usize / CHUNK).min(n - 1);
        for cz in c(z0 - 1, rows)..=c(z1, rows) {
            for cx in c(x0 - 1, cols)..=c(x1, cols) {
                self.dirty.push((cz * cols + cx) as u32);
            }
        }
    }

    /// Notes that walking may have changed anywhere.
    pub fn mark_all(&mut self) {
        self.all_dirty = true;
    }

    /// Whether a `refresh` has work to do.
    pub fn stale(&self) -> bool {
        self.all_dirty || !self.dirty.is_empty()
    }

    /// Labels the chunks marked since the last refresh again, and joins every chunk's parts up.
    pub fn refresh(&mut self, nav: &NavGrid) {
        let lot = nav.lot;
        let grid = lot.chunk_grid();
        let chunks = grid.0 * grid.1;
        if grid != self.grid || self.parts.len() != chunks {
            self.grid = grid;
            self.all_dirty = true;
        }
        self.part.resize(lot.tile_slots(), NO_PART);
        // Chunks a flight of stairs is in (steps, feet, tops and landings): never plain.
        let mut stairs: Vec<u32> = nav
            .stairs
            .stairs
            .iter()
            .flat_map(|s| {
                (0..s.len)
                    .map(|i| (s.bottom.0 + s.dir.0 * i, s.bottom.1 + s.dir.1 * i))
                    .chain([s.foot, s.landing])
            })
            .filter(|&(x, z)| lot.in_bounds(x, z))
            .map(|(x, z)| lot.chunk_at(x, z) as u32)
            .collect();
        stairs.sort_unstable();
        stairs.dedup();
        if self.all_dirty {
            self.parts = vec![0; chunks];
            self.plain = vec![false; chunks];
            self.joins = vec![Vec::new(); chunks];
            self.part.fill(NO_PART);
            for c in 0..chunks {
                self.label(nav, c, &stairs);
            }
            for c in 0..chunks {
                self.join(nav, c);
            }
        } else if !self.dirty.is_empty() {
            let mut dirty = std::mem::take(&mut self.dirty);
            dirty.sort_unstable();
            dirty.dedup();
            for &c in &dirty {
                self.label(nav, c as usize, &stairs);
            }
            // The joins out of the chunks labelled again, and into them from round about.
            let (cols, rows) = grid;
            let mut near: Vec<u32> = Vec::new();
            for &c in &dirty {
                let (cx, cz) = (c as usize % cols, c as usize / cols);
                for nz in cz.saturating_sub(1)..=(cz + 1).min(rows - 1) {
                    for nx in cx.saturating_sub(1)..=(cx + 1).min(cols - 1) {
                        near.push((nz * cols + nx) as u32);
                    }
                }
            }
            // (Stairs join chunks far apart: the chunks at their other ends too.)
            for s in &nav.stairs.stairs {
                for (a, b) in [(s.top(), s.landing), (s.landing, s.top())] {
                    if lot.in_bounds(a.0, a.1) && lot.in_bounds(b.0, b.1) {
                        let ca = lot.chunk_at(a.0, a.1) as u32;
                        if dirty.binary_search(&ca).is_ok() {
                            near.push(lot.chunk_at(b.0, b.1) as u32);
                        }
                    }
                }
            }
            near.sort_unstable();
            near.dedup();
            for &c in &near {
                self.join(nav, c as usize);
            }
        }
        self.all_dirty = false;
        self.dirty.clear();
        self.unite(chunks);
    }

    /// Labels the parts of chunk `c`: its free tiles, joined by the steps within it.
    fn label(&mut self, nav: &NavGrid, c: usize, stairs: &[u32]) {
        let lot = nav.lot;
        // Plain ground: every step inside it is open (no edge or diagonal built, nothing
        // blocking, no stairs), so it is one part.
        self.plain[c] = lot.chunk_allocated(c)
            && !lot.chunk_built(c)
            && stairs.binary_search(&(c as u32)).is_err()
            && lot.chunk_tiles(c).all(|(x, z, _)| nav.tile_free(x, z));
        if self.plain[c] {
            let mut any = false;
            for (_, _, s) in lot.chunk_tiles(c) {
                self.part[s] = 0;
                any = true;
            }
            self.parts[c] = u16::from(any);
            return;
        }
        for (_, _, s) in lot.chunk_tiles(c) {
            self.part[s] = NO_PART;
        }
        let mut parts = 0u16;
        let mut queue = Vec::new();
        for (x, z, s) in lot.chunk_tiles(c) {
            if self.part[s] != NO_PART || !nav.tile_free(x, z) {
                continue;
            }
            self.part[s] = parts;
            queue.push((x, z));
            while let Some((x, z)) = queue.pop() {
                for (nx, nz) in nav.neighbours(x, z) {
                    if lot.chunk_at(nx, nz) != c {
                        continue;
                    }
                    let n = lot.tile_index(nx, nz);
                    if self.part[n] == NO_PART {
                        self.part[n] = parts;
                        queue.push((nx, nz));
                    }
                }
            }
            parts += 1;
        }
        self.parts[c] = parts;
    }

    /// Finds the joins out of chunk `c`: steps from its tiles on its sides (and up or down
    /// stairs) into other chunks.
    fn join(&mut self, nav: &NavGrid, c: usize) {
        let lot = nav.lot;
        let mut joins = Vec::new();
        let (cols, rows) = self.grid;
        let (cx, cz) = (c % cols, c / cols);
        let around: Vec<usize> = (cz.saturating_sub(1)..=(cz + 1).min(rows - 1))
            .flat_map(|nz| {
                (cx.saturating_sub(1)..=(cx + 1).min(cols - 1)).map(move |nx| nz * cols + nx)
            })
            .filter(|&n| n != c)
            .collect();
        // Plain among plain (or open air, or off the lot): every step across is open.
        let easy = |n: usize| {
            self.plain[n] || !lot.chunk_allocated(n) || {
                let [x0, z0, x1, z1] = lot.chunk_rect(n);
                x0 >= x1 || z0 >= z1
            }
        };
        if self.plain[c] && self.parts[c] > 0 && around.iter().all(|&n| easy(n)) {
            joins = around
                .iter()
                .filter(|&&n| self.plain[n] && self.parts[n] > 0)
                .map(|&n| (0, n as u32, 0))
                .collect();
        } else if self.parts[c] > 0 {
            let link_tiles = nav
                .stairs
                .stairs
                .iter()
                .flat_map(|s| [s.top(), s.landing])
                .filter(|&(x, z)| lot.in_bounds(x, z) && lot.chunk_at(x, z) == c);
            for (x, z) in lot.chunk_rim(c).chain(link_tiles) {
                let p = self.part[lot.tile_index(x, z)];
                if p == NO_PART {
                    continue;
                }
                for (nx, nz) in nav.neighbours(x, z) {
                    let nc = lot.chunk_at(nx, nz);
                    if nc == c {
                        continue;
                    }
                    let q = self.part[lot.tile_index(nx, nz)];
                    if q != NO_PART {
                        joins.push((p, nc as u32, q));
                    }
                }
            }
            joins.sort_unstable();
            joins.dedup();
        }
        self.joins[c] = joins;
    }

    /// Joins every chunk's parts into regions (a union-find over the joins).
    fn unite(&mut self, chunks: usize) {
        self.first.clear();
        let mut n = 0u32;
        for c in 0..chunks {
            self.first.push(n);
            n += u32::from(self.parts[c]);
        }
        let mut parent: Vec<u32> = (0..n).collect();
        fn root(parent: &mut [u32], mut a: u32) -> u32 {
            while parent[a as usize] != a {
                let up = parent[parent[a as usize] as usize];
                parent[a as usize] = up;
                a = up;
            }
            a
        }
        for c in 0..chunks {
            for &(p, nc, q) in &self.joins[c] {
                // (A join into a part a chunk no longer has is left over: the chunk changed.)
                if q >= self.parts[nc as usize] || p >= self.parts[c] {
                    continue;
                }
                let a = root(&mut parent, self.first[c] + u32::from(p));
                let b = root(&mut parent, self.first[nc as usize] + u32::from(q));
                if a != b {
                    parent[a.max(b) as usize] = a.min(b);
                }
            }
        }
        let mut region = vec![u32::MAX; n as usize];
        let mut regions = 0;
        for i in 0..n {
            let r = root(&mut parent, i) as usize;
            if region[r] == u32::MAX {
                region[r] = regions;
                regions += 1;
            }
            region[i as usize] = region[r];
        }
        self.region = region;
        self.regions = regions;
    }

    /// How many separate walkable regions the lot has (an open lot has one).
    pub fn regions(&self) -> u32 {
        self.regions
    }

    /// The region of a tile, if it can be stood on.
    fn of(&self, lot: &Lot, (x, z): (i32, i32)) -> Option<u32> {
        if !lot.in_bounds(x, z) {
            return None;
        }
        let p = *self.part.get(lot.tile_index(x, z))?;
        if p == NO_PART {
            return None;
        }
        let c = lot.chunk_at(x, z);
        self.region
            .get((*self.first.get(c)? + u32::from(p)) as usize)
            .copied()
    }

    /// Whether `nav` can get from `a` to `b`. Unknown or out-of-bounds ends answer `true`: the
    /// search itself decides, as before. A walker standing where something now stands (a wall
    /// built over them) counts as able to reach the region of any tile they can step onto.
    pub fn reaches(&self, nav: &NavGrid, a: (i32, i32), b: (i32, i32)) -> bool {
        let Some(to) = self.of(nav.lot, b) else {
            return true;
        };
        match self.of(nav.lot, a) {
            Some(from) => from == to,
            None if nav.lot.in_bounds(a.0, a.1) => nav
                .neighbours(a.0, a.1)
                .any(|n| self.of(nav.lot, n) == Some(to)),
            None => true,
        }
    }
}

/// The last changes to walking (`World::nav_changed_in`), with the navigation versions they made
/// and the tiles `[x0, z0, x1, z1)` they touched: a walker whose way doesn't go near any change
/// since it last looked needn't find its way again. The changes of one tick share a place (cars
/// coming and going, a build: several patches each), so the log goes back `NAV_LOG` ticks with
/// changes however many there were in each.
#[derive(Debug, Clone, Default)]
pub struct NavLog {
    entries: Vec<NavChanges>,
    next: usize,
}

/// Places `NavLog` keeps: at least as many ticks as a resident far from the view waits between
/// turns (`lod::FAR_TICKS`), so one that keeps walking while cars come and go every tick still
/// finds what changed since its last turn; rounded up to a power of two (64 for three game minutes
/// of 20 ticks).
pub const NAV_LOG: usize = (crate::lod::FAR_TICKS as usize).next_power_of_two();
const _: () = assert!(NAV_LOG as u64 >= crate::lod::FAR_TICKS);

/// The most separate patches one place in `NavLog` keeps (more: the two earlier ones whose box
/// round them grows least share one).
const NAV_LOG_PATCHES: usize = 16;

/// The changes to walking made in one tick (`group`): navigation versions `first..=last`, over
/// tiles `rects[..len]`, each with the last version that touched them.
#[derive(Debug, Clone, Copy)]
struct NavChanges {
    group: u64,
    first: u32,
    last: u32,
    rects: [([i32; 4], u32); NAV_LOG_PATCHES],
    len: usize,
}

impl NavChanges {
    /// Adds the change that made `version`, over tiles `r`. (With no room, two earlier patches
    /// make room, so the new one is kept as it is and a walker that looked between them isn't
    /// told of a box round older ones.)
    fn add(&mut self, r: [i32; 4], version: u32) {
        if self.len < NAV_LOG_PATCHES {
            self.rects[self.len] = (r, version);
            self.len += 1;
            return;
        }
        let area = |a: [i32; 4]| i64::from(a[2] - a[0]) * i64::from(a[3] - a[1]);
        let cost = |a: [i32; 4], b: [i32; 4]| {
            let both = area(join(a, b));
            (both - area(a) - area(b), both)
        };
        let rects = &mut self.rects;
        let (mut bi, mut bj) = (0, 1);
        let mut best = cost(rects[0].0, rects[1].0);
        for i in 0..NAV_LOG_PATCHES {
            for j in i + 1..NAV_LOG_PATCHES {
                let c = cost(rects[i].0, rects[j].0);
                if c < best {
                    (bi, bj, best) = (i, j, c);
                }
            }
        }
        let ((a, va), (b, vb)) = (rects[bi], rects[bj]);
        // (The later of the two: the closer to `version`.)
        let later = if version.wrapping_sub(va) < version.wrapping_sub(vb) {
            va
        } else {
            vb
        };
        rects[bi] = (join(a, b), later);
        rects[bj] = (r, version);
    }
}

impl NavLog {
    /// Notes the change that made navigation version `version`, over tiles `rect`, in tick
    /// `group`: with the tick's other changes if it follows on from them.
    pub fn push(&mut self, group: u64, version: u32, rect: [i32; 4]) {
        if !self.entries.is_empty() {
            let newest = &mut self.entries[(self.next + NAV_LOG - 1) % NAV_LOG];
            if newest.group == group && newest.last.wrapping_add(1) == version {
                newest.last = version;
                newest.add(rect, version);
                return;
            }
        }
        let mut place = NavChanges {
            group,
            first: version,
            last: version,
            rects: [([0; 4], 0); NAV_LOG_PATCHES],
            len: 0,
        };
        place.add(rect, version);
        if self.entries.len() < NAV_LOG {
            self.entries.push(place);
        } else {
            self.entries[self.next] = place;
        }
        self.next = (self.next + 1) % NAV_LOG;
    }

    /// Whether a change after version `seen`, up to `now`, may touch tiles `[x0, z0, x1, z1)`:
    /// one does, or the log doesn't go back that far (then any might).
    pub fn changed_since(&self, seen: u32, now: u32, rect: [i32; 4]) -> bool {
        self.changes_since(seen, now)
            .is_none_or(|mut changes| changes.any(|r| overlaps(r, rect)))
    }

    /// The tiles the changes after version `seen`, up to `now`, touched (and maybe a few more,
    /// where a tick's patches were joined); None if the log doesn't go back that far (any tile
    /// may have changed).
    pub fn changes_since(&self, seen: u32, now: u32) -> Option<impl Iterator<Item = [i32; 4]>> {
        let since = now.wrapping_sub(seen);
        // (How many of the versions after `seen` each place made.)
        let after = move |e: &NavChanges| {
            let newest = now.wrapping_sub(e.last);
            if newest >= since {
                return 0;
            }
            now.wrapping_sub(e.first).min(since - 1) - newest + 1
        };
        let found: u64 = self.entries.iter().map(|e| u64::from(after(e))).sum();
        (found >= u64::from(since)).then(|| {
            self.entries
                .iter()
                // (A place whose last change is older skips its patches at once.)
                .filter(move |e| now.wrapping_sub(e.last) < since)
                .flat_map(|e| e.rects[..e.len].iter())
                .filter(move |r| now.wrapping_sub(r.1) < since)
                .map(|r| r.0)
        })
    }
}

/// The box round two tile rectangles `[x0, z0, x1, z1)`.
pub fn join(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[2].max(b[2]),
        a[3].max(b[3]),
    ]
}

/// Whether two tile rectangles `[x0, z0, x1, z1)` share a tile.
pub fn overlaps(a: [i32; 4], b: [i32; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// How a search ended.
enum Search {
    Found(Vec<(i32, i32)>),
    /// Every tile it could reach was tried.
    NoWay,
    /// It gave up at its cap.
    Capped,
}

/// The most tiles a search tries before it looks for a corridor (`NavGrid::find_path_with`).
pub const NODE_CAP: u64 = 30_000;
/// The most a search kept to a corridor tries.
const CORRIDOR_CAP: u64 = 4 * NODE_CAP;

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
    static SCRATCH: RefCell<Scratch<u32, u32, (u32, u32)>> = RefCell::default();
    static STATS: RefCell<SearchStats> = RefCell::default();
}

/// What the searches on this thread cost since the last `take_search_stats`: for benchmarks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SearchStats {
    /// A* searches run (refused ones, unreachable or with a blocked goal, don't count).
    pub searches: u64,
    /// Tiles expanded over all of them, and by the biggest one.
    pub nodes: u64,
    pub max_nodes: u64,
}

impl SearchStats {
    fn add(&mut self, nodes: u64) {
        self.searches += 1;
        self.nodes += nodes;
        self.max_nodes = self.max_nodes.max(nodes);
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

/// The search costs on this thread since the last call (and starts counting again).
pub fn take_search_stats() -> SearchStats {
    STATS.with_borrow_mut(std::mem::take)
}

const NEIGHBOURS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

fn heuristic(a: (i32, i32), b: (i32, i32)) -> u32 {
    let dx = (a.0 - b.0).unsigned_abs();
    let dz = (a.1 - b.1).unsigned_abs();
    ORTHO * dx.max(dz) + (DIAG - ORTHO) * dx.min(dz)
}

/// The path to `at`, back along `came` (by slot: the row-major key of the tile before).
fn reconstruct(lot: &Lot, came: &[u32], mut at: (i32, i32)) -> Vec<(i32, i32)> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lot::LotFile;

    fn lot_with_wall() -> Lot {
        // Wall along x = 5 from z = 0 to z = 8, door-less: must go around at z >= 8.
        let file: LotFile =
            serde_json::from_str(r#"{"width":10,"depth":10,"walls":[[5,0,5,8]]}"#).unwrap();
        Lot::from_file(&file).unwrap()
    }

    /// A 10×10 lot with a closed 4×4 room in its middle.
    fn lot_with_sealed_room(door: bool) -> Lot {
        let bottom = if door {
            r#"[1,5,3,5],[4,5,5,5]"#
        } else {
            r#"[1,5,5,5]"#
        };
        let json = format!(
            r#"{{"width":10,"depth":10,"walls":[[1,1,5,1],{bottom},[1,1,1,5],[5,1,5,5]]}}"#
        );
        let file: LotFile = serde_json::from_str(&json).unwrap();
        Lot::from_file(&file).unwrap()
    }

    #[test]
    fn reach_labels_separate_regions() {
        let lot = lot_with_sealed_room(false);
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        let reach = Reach::build(&nav);
        assert_eq!(reach.regions(), 2, "the room is sealed off");
        assert!(reach.reaches(&nav, (2, 2), (3, 3)));
        assert!(!reach.reaches(&nav, (2, 2), (8, 8)));
        assert!(!reach.reaches(&nav, (8, 8), (2, 2)));
        // Standing on a tile something now blocks: a free neighbour decides.
        let mut blocked = blocked;
        blocked[lot.tile_index(2, 2)] = true;
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        assert!(reach.reaches(&nav, (2, 2), (3, 3)));
        assert!(!reach.reaches(&nav, (2, 2), (8, 8)));
    }

    #[test]
    fn a_door_joins_the_regions() {
        let lot = lot_with_sealed_room(true);
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        let reach = Reach::build(&nav);
        assert_eq!(reach.regions(), 1);
        assert!(reach.reaches(&nav, (2, 2), (8, 8)));
        assert!(nav.find_path((2, 2), (8, 8)).is_some());
    }

    #[test]
    fn routes_around_wall() {
        let lot = lot_with_wall();
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        let path = nav.find_path((2, 2), (7, 2)).expect("path exists");
        assert!(
            path.iter().any(|&(_, z)| z >= 8),
            "must go around the wall end"
        );
        for w in path.windows(2) {
            assert!(nav.can_step(w[0].0, w[0].1, w[1].0, w[1].1));
        }
    }

    #[test]
    fn smoothing_keeps_walkability() {
        let lot = lot_with_wall();
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        let path = nav.find_path((2, 2), (7, 2)).unwrap();
        let smooth = nav.smooth(&path);
        assert!(smooth.len() < path.len());
        for w in smooth.windows(2) {
            assert!(nav.line_walkable(w[0], w[1]));
        }
    }

    fn nav_of(lot: &Lot) -> Vec<bool> {
        vec![false; lot.tile_slots()]
    }

    /// Every step of a path is allowed and no tile on it is crossed by a diagonal wall.
    fn assert_walkable(nav: &NavGrid, path: &[(i32, i32)]) {
        for &(x, z) in path {
            assert!(
                !nav.lot.diag_blocks(x, z),
                "path enters diagonal wall tile {x},{z}"
            );
        }
        for w in path.windows(2) {
            assert!(nav.can_step(w[0].0, w[0].1, w[1].0, w[1].1), "{w:?}");
        }
    }

    /// A diagonal wall `/` from (2, 0) to (8, 6) across tiles (2,0)..(7,5), then straight up
    /// x = 8 to z = 10: the lower-right side is closed off except around the far end.
    fn lot_with_diagonal(door: bool) -> Lot {
        let mut diags: Vec<String> = (0..6)
            .map(|k| format!(r#"{{"x":{},"z":{},"dir":"dp"}}"#, 2 + k, k))
            .collect();
        if door {
            diags[3] = r#"{"x":5,"z":3,"dir":"dp","kind":"door"}"#.into();
        }
        let json = format!(
            r#"{{"width":12,"depth":12,"walls":[[8,6,8,11],[2,0,0,0]],"diagonals":[{}]}}"#,
            diags.join(",")
        );
        let file: LotFile = serde_json::from_str(&json).unwrap();
        Lot::from_file(&file).unwrap()
    }

    #[test]
    fn diagonal_wall_tiles_block_and_steps_cannot_cross_it() {
        let lot = lot_with_diagonal(false);
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        assert!(!nav.tile_free(4, 2), "nobody stands on a diagonal wall");
        // Orthogonal steps into the wall tile are refused from both halves.
        assert!(!nav.can_step(4, 1, 4, 2));
        assert!(!nav.can_step(3, 2, 4, 2));
        // Diagonal steps across the joint of two diagonal tiles are refused both ways.
        assert!(
            !nav.can_step(5, 2, 4, 3),
            "crosses the wall at corner (5, 3)"
        );
        assert!(
            !nav.can_step(6, 3, 5, 4),
            "crosses the wall at corner (6, 4)"
        );
        assert!(!nav.can_step(5, 4, 6, 3));
        // Beside the wall (parallel to it) is fine.
        assert!(nav.can_step(5, 2, 6, 3));
        // Past a free end (the wall stops at (8, 6) and the h wall at x = 0..2, z = 0).
        assert!(
            !nav.can_step(7, 6, 8, 5),
            "would clip the wall's end at (8, 6)"
        );
    }

    #[test]
    fn routes_around_a_diagonal_wall() {
        let lot = lot_with_diagonal(false);
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        // From the lower-right side (below the wall) to the upper-left side.
        let path = nav.find_path((6, 1), (2, 4)).expect("path around the end");
        assert_walkable(&nav, &path);
        assert!(
            path.iter().any(|&(x, z)| x >= 8 && z >= 11) || path.iter().any(|&(x, _)| x >= 9),
            "must go around via x > 8: {path:?}"
        );
        let smooth = nav.smooth(&path);
        for w in smooth.windows(2) {
            assert!(nav.line_walkable(w[0], w[1]));
        }
    }

    #[test]
    fn diagonal_door_lets_residents_through() {
        let lot = lot_with_diagonal(true);
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        assert!(nav.tile_free(5, 3), "a doorway can be walked");
        let path = nav.find_path((6, 2), (4, 4)).expect("through the door");
        assert!(path.contains(&(5, 3)), "{path:?}");
        assert!(path.len() <= 5, "short way through the door: {path:?}");
        assert_walkable(&nav, &path);
        // Not along the wall through the door's ends.
        assert!(!nav.can_step(4, 2, 5, 3));
        assert!(!nav.can_step(5, 3, 6, 4));
    }

    #[test]
    fn corridor_between_parallel_diagonals_is_walkable() {
        // Two `/` walls one tile apart: (0,0)..(3,3) and (2,0)..(5,3) (x - z = 0 and 2).
        let diags: Vec<String> = (0..3)
            .flat_map(|k| {
                [
                    format!(r#"{{"x":{k},"z":{k},"dir":"dp"}}"#),
                    format!(r#"{{"x":{},"z":{k},"dir":"dp"}}"#, k + 2),
                ]
            })
            .collect();
        let json = format!(
            r#"{{"width":8,"depth":8,"diagonals":[{}]}}"#,
            diags.join(",")
        );
        let lot = Lot::from_file(&serde_json::from_str::<LotFile>(&json).unwrap()).unwrap();
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        assert!(nav.can_step(1, 0, 2, 1), "along the corridor");
        assert!(nav.can_step(2, 1, 3, 2));
        assert!(!nav.can_step(1, 0, 0, 1), "across the left wall");
    }

    #[test]
    fn no_path_into_closed_box() {
        let file: LotFile = serde_json::from_str(
            r#"{"width":6,"depth":6,"walls":[[2,2,4,2],[2,4,4,4],[2,2,2,4],[4,2,4,4]]}"#,
        )
        .unwrap();
        let lot = Lot::from_file(&file).unwrap();
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        assert!(nav.find_path((0, 0), (3, 3)).is_none());
    }

    /// Three gardens side by side, x 0..10, 10..20 and 20..30 (plots 0, 1, 2) over rows
    /// 0..10, with the street along rows 10..14. No fences: only the plots tell them apart.
    fn three_gardens(walls: &str) -> (Lot, Vec<u16>) {
        let json = format!(r#"{{"width":30,"depth":14,"walls":[{walls}]}}"#);
        let lot = Lot::from_file(&serde_json::from_str::<LotFile>(&json).unwrap()).unwrap();
        let tile_plot = (0..30 * 14)
            .map(|i| {
                if i / 30 < 10 {
                    (i % 30 / 10) as u16
                } else {
                    NO_PLOT
                }
            })
            .collect();
        (lot, tile_plot)
    }

    fn in_middle_garden((x, z): (i32, i32)) -> bool {
        (10..20).contains(&x) && z < 10
    }

    #[test]
    fn residents_go_round_by_the_street_not_through_the_neighbours_garden() {
        let (lot, tile_plot) = three_gardens("");
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        // Without plots to mind, straight across the middle garden.
        let direct = nav.find_path((5, 8), (25, 8)).unwrap();
        assert!(direct.iter().any(|&t| in_middle_garden(t)), "{direct:?}");
        // From the front of the first to the front of the third: the middle one isn't theirs.
        let grounds = Grounds {
            tile_plot: &tile_plot,
            open: vec![true, false, true],
            fenced: vec![true; 3],
        };
        let path = nav.find_path_with((5, 8), (25, 8), Some(&grounds)).unwrap();
        assert_walkable(&nav, &path);
        assert!(
            !path.iter().any(|&t| in_middle_garden(t)),
            "through the neighbours' garden: {path:?}"
        );
        assert!(
            path.iter().any(|&(_, z)| z >= 10),
            "by the street: {path:?}"
        );
        // The smoothed line doesn't cut back across it either.
        let smooth = nav.smooth_with(&path, Some(&grounds));
        for w in smooth.windows(2) {
            assert!(nav.line_walkable_with(w[0], w[1], Some(&grounds)), "{w:?}");
            for k in 0..=100 {
                let t = k as f32 / 100.0;
                let p = [
                    w[0][0] + (w[1][0] - w[0][0]) * t,
                    w[0][1] + (w[1][1] - w[0][1]) * t,
                ];
                assert!(
                    !in_middle_garden((p[0].floor() as i32, p[1].floor() as i32)),
                    "smoothed across at {p:?}"
                );
            }
        }
        assert!(!nav.line_walkable_with([5.5, 8.5], [25.5, 8.5], Some(&grounds)));
        assert!(
            nav.line_walkable([5.5, 8.5], [25.5, 8.5]),
            "only the plots stand in the way"
        );
    }

    #[test]
    fn into_the_garden_next_door_by_the_street_not_over_the_hedge() {
        let (lot, tile_plot) = three_gardens("");
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        // From their own garden to the neighbours' (both open to them: they're going there), the
        // two side by side: out at the front, along the street and in at the neighbours' front.
        let grounds = Grounds {
            tile_plot: &tile_plot,
            open: vec![true, true, false],
            fenced: vec![true; 3],
        };
        let path = nav.find_path_with((8, 3), (12, 3), Some(&grounds)).unwrap();
        assert_walkable(&nav, &path);
        assert!(
            path.windows(2)
                .all(|w| (w[0].0 < 10) == (w[1].0 < 10) || w[0].1 >= 10),
            "over the boundary between the gardens: {path:?}"
        );
        assert!(
            path.iter().any(|&(_, z)| z >= 10),
            "by the street: {path:?}"
        );
        let smooth = nav.smooth_with(&path, Some(&grounds));
        for w in smooth.windows(2) {
            assert!(nav.line_walkable_with(w[0], w[1], Some(&grounds)), "{w:?}");
        }
        assert!(
            !nav.line_walkable_with([8.5, 3.5], [12.5, 3.5], Some(&grounds)),
            "no straight line over it"
        );
        // Between plots nobody fences (public places side by side), straight across.
        let grounds = Grounds {
            tile_plot: &tile_plot,
            open: vec![true; 3],
            fenced: vec![false; 3],
        };
        assert_eq!(
            nav.find_path_with((8, 3), (12, 3), Some(&grounds))
                .unwrap()
                .len(),
            5
        );
        assert!(nav.line_walkable_with([8.5, 3.5], [12.5, 3.5], Some(&grounds)));
    }

    #[test]
    fn a_garden_is_crossed_when_there_is_no_other_way() {
        // The street is closed off in front of the middle garden.
        let (lot, tile_plot) = three_gardens("[10,10,10,14],[20,10,20,14]");
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        let grounds = Grounds {
            tile_plot: &tile_plot,
            open: vec![true, false, true],
            fenced: vec![true; 3],
        };
        let path = nav
            .find_path_with((5, 12), (25, 12), Some(&grounds))
            .expect("through the garden");
        assert_walkable(&nav, &path);
        assert!(path.iter().any(|&t| in_middle_garden(t)));
        // Someone standing in a garden they shouldn't be in can still get out of it, the
        // shortest way (not across the gardens either side).
        let grounds = Grounds {
            tile_plot: &tile_plot,
            open: vec![false; 3],
            fenced: vec![true; 3],
        };
        let out = nav
            .find_path_with((15, 2), (15, 12), Some(&grounds))
            .expect("out to the street");
        assert_eq!(out.len(), 11, "straight down: {out:?}");
        assert_eq!(
            nav.smooth_with(&out, Some(&grounds)).len(),
            2,
            "in one straight line"
        );
    }

    #[test]
    fn a_long_way_round_is_found_through_a_corridor() {
        // A wall down the middle of a big open lot with one gap at the far end: straight at the
        // goal, a search would try most of the near half before turning back.
        let mut lot = Lot::new(400, 400);
        for z in 0..396 {
            lot.set_v(200, z, crate::lot::Edge::Wall);
        }
        lot.compute_rooms();
        let blocked = nav_of(&lot);
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        take_search_stats();
        let before = nodes_searched();
        let path = nav.find_path((100, 10), (300, 10)).expect("round the end");
        assert_walkable(&nav, &path);
        assert!(path.iter().any(|&(_, z)| z >= 396), "through the gap");
        let stats = take_search_stats();
        assert_eq!(
            nodes_searched() - before,
            stats.nodes,
            "the budget counter follows"
        );
        assert!(
            stats.searches >= 2,
            "capped, then kept to a corridor: {stats:?}"
        );
        assert!(stats.max_nodes <= CORRIDOR_CAP + 1, "{stats:?}");
        // As long as the full search's, near enough (the corridor holds the best way here).
        let shortest = 2 * 386 + 200;
        assert!(path.len() <= shortest + 40, "{} tiles", path.len());
    }

    #[test]
    fn stairs_are_found_by_tile() {
        let mut map = StairMap {
            depth: 10,
            ..Default::default()
        };
        let flight = |x| Stair {
            foot: (x, 6),
            bottom: (x, 5),
            dir: (0, -1),
            len: 3,
            landing: (x, 12),
        };
        map.set_stairs(vec![flight(2), flight(5)]);
        assert_eq!(
            map.step(5, 4).map(|(s, i)| (s.bottom, i)),
            Some(((5, 5), 1))
        );
        assert_eq!(map.step(5, 2), None);
        assert_eq!(map.link(2, 3), Some((2, 12)));
        assert_eq!(map.link(2, 12), Some((2, 3)));
        assert_eq!(map.link(3, 3), None);
    }

    #[test]
    fn search_buffers_follow_the_lot_size() {
        // The same thread routes on a small lot, a lot grown by two storeys, and the small one
        // again: the kept buffers are resized each time and earlier searches don't leak in.
        let small = lot_with_wall();
        let mut tall = lot_with_wall();
        tall.grow_depth(30);
        // (The new rows are built on: their chunks are there.)
        tall.ensure(0, 10, 10, 30);
        let (bs, bt) = (nav_of(&small), nav_of(&tall));
        let flat = NavGrid {
            lot: &small,
            blocked: &bs,
            stairs: &NO_STAIRS,
        };
        let up = NavGrid {
            lot: &tall,
            blocked: &bt,
            stairs: &NO_STAIRS,
        };
        let first = flat.find_path((2, 2), (7, 2)).unwrap();
        assert!(
            up.find_path((2, 22), (7, 22)).is_some(),
            "rows on the third storey are searched"
        );
        assert_eq!(flat.find_path((2, 2), (7, 2)).unwrap(), first);
    }

    #[test]
    fn a_ticks_changes_to_walking_share_a_place_in_the_log() {
        let mut log = NavLog::default();
        let far = [500, 500, 501, 501];
        // Tick 1: more patches than a place keeps, all round (0, 0) to (40, 0), then one at
        // (100, 0).
        for v in 1..=40u32 {
            let x = v as i32;
            log.push(1, v, [x, 0, x + 1, 1]);
        }
        log.push(1, 41, [100, 0, 101, 1]);
        assert!(log.changed_since(0, 41, [100, 0, 101, 1]));
        assert!(!log.changed_since(0, 41, far));
        // Looked after the first 40: only the newest is news.
        assert!(!log.changed_since(40, 41, [0, 0, 41, 1]));
        assert!(log.changed_since(40, 41, [100, 0, 101, 1]));
        // More ticks of changes than the log keeps: anything may have changed since tick 1.
        for t in 0..NAV_LOG as u32 {
            log.push(2 + u64::from(t), 42 + t, [0, 0, 1, 1]);
        }
        let now = 41 + NAV_LOG as u32;
        assert!(!log.changed_since(41, now, far), "every tick since is kept");
        assert!(log.changed_since(40, now, far), "tick 1 isn't");
        // A change nobody noted where (the version bumped without the log): anything.
        assert!(log.changed_since(now, now + 1, far));
        assert!(!log.changed_since(now, now, far), "nothing since");
    }
}
