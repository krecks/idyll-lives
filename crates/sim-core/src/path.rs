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

use crate::lot::{Lot, OUTDOORS};

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
        (self.bottom.0 + self.dir.0 * (self.len - 1), self.bottom.1 + self.dir.1 * (self.len - 1))
    }

    /// Which step a tile is (0 the bottom one), if it is one.
    pub fn step(&self, x: i32, z: i32) -> Option<i32> {
        (0..self.len).find(|&i| (self.bottom.0 + self.dir.0 * i, self.bottom.1 + self.dir.1 * i) == (x, z))
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
    pub stairs: Vec<Stair>,
}

/// No storeys, no stairs.
pub static NO_STAIRS: StairMap = StairMap { depth: 0, basement: 0, stairs: Vec::new() };

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
        if self.depth > 0 { z.rem_euclid(self.depth) } else { z }
    }

    /// The flight a tile is a step of, and which step.
    pub fn step(&self, x: i32, z: i32) -> Option<(&Stair, i32)> {
        self.stairs.iter().find_map(|s| s.step(x, z).map(|i| (s, i)))
    }

    /// Where a stair links to from this tile: the landing from the top step, or back.
    pub fn link(&self, x: i32, z: i32) -> Option<(i32, i32)> {
        self.stairs.iter().find_map(|s| {
            if s.top() == (x, z) {
                Some(s.landing)
            } else if s.landing == (x, z) {
                Some(s.top())
            } else {
                None
            }
        })
    }

    /// How far apart two points are for someone walking (storeys counted as a few metres).
    pub fn distance(&self, a: [f32; 2], b: [f32; 2]) -> f32 {
        if self.depth == 0 {
            return (a[0] - b[0]).hypot(a[1] - b[1]);
        }
        let d = self.depth as f32;
        let (ka, kb) = (self.storey(a[1].floor() as i32), self.storey(b[1].floor() as i32));
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
/// Extra cost of stepping straight from one plot into another, over the fence or hedge between
/// gardens (the renderer's, `shared/street.ts`): three hundred tiles, so a plot is entered from
/// the street at its front wherever there is one, and only crossed into where there isn't.
const OVER_THE_FENCE: u32 = ORTHO * 300;

/// Which plots one resident may walk across (their home, public places, where they're going);
/// the rest are other people's gardens, kept off unless there's no other way.
pub struct Grounds<'a> {
    /// Plot per tile of the ground storey (upper storeys are over the same ground).
    pub tile_plot: &'a [Option<u32>],
    /// Per plot (by id): whether this resident may walk across it.
    pub open: Vec<bool>,
}

impl Grounds<'_> {
    /// The plot a tile is on (on any storey).
    pub fn plot(&self, nav: &NavGrid, x: i32, z: i32) -> Option<u32> {
        let z = nav.stairs.ground_z(z);
        if x < 0 || z < 0 || x >= nav.lot.width as i32 {
            return None;
        }
        self.tile_plot.get(z as usize * nav.lot.width + x as usize).copied().flatten()
    }

    /// The plot a tile is on, if it's one this resident shouldn't walk across.
    fn foreign(&self, nav: &NavGrid, x: i32, z: i32) -> Option<u32> {
        self.plot(nav, x, z).filter(|&p| !self.open.get(p as usize).copied().unwrap_or(true))
    }

    /// A step from one plot straight into another (over the fence between them), open or not.
    fn over_the_fence(&self, nav: &NavGrid, (x, z): (i32, i32), (nx, nz): (i32, i32)) -> bool {
        matches!((self.plot(nav, x, z), self.plot(nav, nx, nz)), (Some(a), Some(b)) if a != b)
    }
}

/// Read-only view of what is walkable: lot edges plus tiles blocked by objects, storeys and stairs.
pub struct NavGrid<'a> {
    pub lot: &'a Lot,
    pub blocked: &'a [bool],
    pub stairs: &'a StairMap,
}

impl NavGrid<'_> {
    /// Whether a resident can stand on the tile: on the lot, no furniture, no diagonal wall;
    /// off the ground (upstairs or in the basement), only in a room (where there's a floor).
    pub fn tile_free(&self, x: i32, z: i32) -> bool {
        self.lot.in_bounds(x, z)
            && !self.blocked[self.lot.tile_index(x, z)]
            && !self.lot.diag_blocks(x, z)
            && (self.stairs.depth == 0 || z < self.stairs.depth || self.lot.room_at(x, z) != OUTDOORS)
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
    pub fn find_path_with(&self, start: (i32, i32), goal: (i32, i32), grounds: Option<&Grounds>) -> Option<Vec<(i32, i32)>> {
        let lot = self.lot;
        if !lot.in_bounds(start.0, start.1) || !self.tile_free(goal.0, goal.1) {
            return None;
        }
        if start == goal {
            return Some(vec![start]);
        }
        SCRATCH.with_borrow_mut(|scratch| {
            // (Every storey's rows: the lot's whole depth.)
            let search = scratch.start(lot.width * lot.depth);
            let Scratch { g, came, stamp, open, .. } = scratch;
            // Costs from earlier searches don't count.
            let cost_to = |g: &[u32], stamp: &[u32], i: usize| if stamp[i] == search { g[i] } else { u32::MAX };
            let s = lot.tile_index(start.0, start.1);
            let goal_i = lot.tile_index(goal.0, goal.1);
            (g[s], came[s], stamp[s]) = (0, u32::MAX, search);
            let heuristic = |a: (i32, i32), b: (i32, i32)| heuristic(self.flat(a), self.flat(b));
            open.push(Reverse((heuristic(start, goal), s as u32)));

            while let Some(Reverse((f, i))) = open.pop() {
                let i = i as usize;
                if i == goal_i {
                    return Some(reconstruct(lot, came, i));
                }
                let (x, z) = ((i % lot.width) as i32, (i / lot.width) as i32);
                if f > g[i] + heuristic((x, z), goal) {
                    continue; // stale heap entry
                }
                let link = self.stairs.link(x, z).filter(|&(lx, lz)| self.tile_free(lx, lz));
                for (dx, dz) in NEIGHBOURS.into_iter().chain(link.map(|(lx, lz)| (lx - x, lz - z))) {
                    let (nx, nz) = (x + dx, z + dz);
                    let linked = Some((nx, nz)) == link;
                    if !linked && ((dx.abs() > 1 || dz.abs() > 1) || !self.can_step(x, z, nx, nz)) {
                        continue;
                    }
                    let ni = lot.tile_index(nx, nz);
                    let mut cost = g[i] + if dx != 0 && dz != 0 && !linked { DIAG } else { ORTHO };
                    if let Some(gr) = grounds {
                        if gr.foreign(self, nx, nz).is_some() {
                            cost += TRESPASS;
                        }
                        if !linked && gr.over_the_fence(self, (x, z), (nx, nz)) {
                            cost += OVER_THE_FENCE;
                        }
                    }
                    if cost < cost_to(g, stamp, ni) {
                        (g[ni], came[ni], stamp[ni]) = (cost, i as u32, search);
                        open.push(Reverse((cost + heuristic((nx, nz), goal), ni as u32)));
                    }
                }
            }
            None
        })
    }

    /// A tile as if on the ground storey (so distances between storeys stay honest).
    fn flat(&self, (x, z): (i32, i32)) -> (i32, i32) {
        (x, self.stairs.ground_z(z))
    }

    /// Tiles a resident can walk to from `(x, z)` in one step: neighbours, and up or down stairs.
    pub fn neighbours(&self, x: i32, z: i32) -> impl Iterator<Item = (i32, i32)> + '_ {
        let link = self.stairs.link(x, z).filter(|&(lx, lz)| self.tile_free(lx, lz));
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
        let ends = grounds.map(|g| [tile(a), tile(b)].map(|(x, z)| g.foreign(self, x, z)));
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
                if let (Some(g), Some(ends)) = (grounds, ends)
                    && (g.over_the_fence(self, (tx, tz), (nx, nz)) || g.foreign(self, nx, nz).is_some_and(|p| !ends.contains(&Some(p))))
                {
                    return false;
                }
                (tx, tz) = (nx, nz);
            }
        }
        true
    }
}

/// Which walkable tiles can reach which: a label per tile, shared by every tile in one
/// connected region. A goal in another region has no path, so a search for it is refused
/// before it can flood the town (P3-08).
///
/// Labels are only meaningful against the `NavGrid` they were built from; `World` rebuilds
/// them when the navigation version changes.
#[derive(Default)]
pub struct Reach {
    label: Vec<u32>,
    regions: u32,
}

impl Reach {
    /// Labels the walkable regions of `nav`. One pass over the lot, in the same steps a walker
    /// may take (so diagonal walls and stairs are respected), costs what one failed search
    /// costs but is paid once per change to the house instead of once per unreachable goal.
    pub fn build(nav: &NavGrid) -> Reach {
        let lot = nav.lot;
        let tiles = lot.width * lot.depth;
        let mut label = vec![u32::MAX; tiles];
        let mut queue = Vec::new();
        let mut regions = 0;
        for start in 0..tiles {
            let (x, z) = ((start % lot.width) as i32, (start / lot.width) as i32);
            if label[start] != u32::MAX || !nav.tile_free(x, z) {
                continue;
            }
            label[start] = regions;
            queue.push(start);
            while let Some(t) = queue.pop() {
                let (x, z) = ((t % lot.width) as i32, (t / lot.width) as i32);
                for (nx, nz) in nav.neighbours(x, z) {
                    let n = lot.tile_index(nx, nz);
                    if label[n] == u32::MAX {
                        label[n] = regions;
                        queue.push(n);
                    }
                }
            }
            regions += 1;
        }
        Reach { label, regions }
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
        match self.label[lot.tile_index(x, z)] {
            u32::MAX => None,
            l => Some(l),
        }
    }

    /// Whether `nav` can get from `a` to `b`. Unknown or out-of-bounds ends answer `true`: the
    /// search itself decides, as before. A walker standing where something now stands (a wall
    /// built over them) counts as able to reach the region of any tile they can step onto.
    pub fn reaches(&self, nav: &NavGrid, a: (i32, i32), b: (i32, i32)) -> bool {
        let Some(to) = self.of(nav.lot, b) else { return true };
        match self.of(nav.lot, a) {
            Some(from) => from == to,
            None if nav.lot.in_bounds(a.0, a.1) => {
                nav.neighbours(a.0, a.1).any(|n| self.of(nav.lot, n) == Some(to))
            }
            None => true,
        }
    }
}

/// Search state kept between searches: a big town is tens of thousands of tiles, too many to
/// allocate and clear for every route. An entry counts only if stamped by the current search.
#[derive(Default)]
struct Scratch {
    g: Vec<u32>,
    came: Vec<u32>,
    stamp: Vec<u32>,
    search: u32,
    open: BinaryHeap<Reverse<(u32, u32)>>,
}

impl Scratch {
    /// Gets ready for a search over `tiles` tiles; returns its stamp.
    fn start(&mut self, tiles: usize) -> u32 {
        if self.g.len() != tiles {
            *self = Self { g: vec![0; tiles], came: vec![0; tiles], stamp: vec![0; tiles], ..Self::default() };
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
    static SCRATCH: RefCell<Scratch> = RefCell::default();
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

fn reconstruct(lot: &Lot, came: &[u32], mut i: usize) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    loop {
        out.push(((i % lot.width) as i32, (i / lot.width) as i32));
        match came[i] {
            u32::MAX => break,
            prev => i = prev as usize,
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
        let blocked = vec![false; 100];
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
        let blocked = vec![false; 100];
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
        vec![false; lot.width * lot.depth]
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
        let blocked = vec![false; 36];
        let nav = NavGrid {
            lot: &lot,
            blocked: &blocked,
            stairs: &NO_STAIRS,
        };
        assert!(nav.find_path((0, 0), (3, 3)).is_none());
    }

    /// Three gardens side by side, x 0..10, 10..20 and 20..30 (plots 0, 1, 2) over rows
    /// 0..10, with the street along rows 10..14. No fences: only the plots tell them apart.
    fn three_gardens(walls: &str) -> (Lot, Vec<Option<u32>>) {
        let json = format!(r#"{{"width":30,"depth":14,"walls":[{walls}]}}"#);
        let lot = Lot::from_file(&serde_json::from_str::<LotFile>(&json).unwrap()).unwrap();
        let tile_plot = (0..30 * 14).map(|i| (i / 30 < 10).then_some((i % 30 / 10) as u32)).collect();
        (lot, tile_plot)
    }

    fn in_middle_garden((x, z): (i32, i32)) -> bool {
        (10..20).contains(&x) && z < 10
    }

    #[test]
    fn residents_go_round_by_the_street_not_through_the_neighbours_garden() {
        let (lot, tile_plot) = three_gardens("");
        let blocked = nav_of(&lot);
        let nav = NavGrid { lot: &lot, blocked: &blocked, stairs: &NO_STAIRS };
        // Without plots to mind, straight across the middle garden.
        let direct = nav.find_path((5, 8), (25, 8)).unwrap();
        assert!(direct.iter().any(|&t| in_middle_garden(t)), "{direct:?}");
        // From the front of the first to the front of the third: the middle one isn't theirs.
        let grounds = Grounds { tile_plot: &tile_plot, open: vec![true, false, true] };
        let path = nav.find_path_with((5, 8), (25, 8), Some(&grounds)).unwrap();
        assert_walkable(&nav, &path);
        assert!(!path.iter().any(|&t| in_middle_garden(t)), "through the neighbours' garden: {path:?}");
        assert!(path.iter().any(|&(_, z)| z >= 10), "by the street: {path:?}");
        // The smoothed line doesn't cut back across it either.
        let smooth = nav.smooth_with(&path, Some(&grounds));
        for w in smooth.windows(2) {
            assert!(nav.line_walkable_with(w[0], w[1], Some(&grounds)), "{w:?}");
            for k in 0..=100 {
                let t = k as f32 / 100.0;
                let p = [w[0][0] + (w[1][0] - w[0][0]) * t, w[0][1] + (w[1][1] - w[0][1]) * t];
                assert!(!in_middle_garden((p[0].floor() as i32, p[1].floor() as i32)), "smoothed across at {p:?}");
            }
        }
        assert!(!nav.line_walkable_with([5.5, 8.5], [25.5, 8.5], Some(&grounds)));
        assert!(nav.line_walkable([5.5, 8.5], [25.5, 8.5]), "only the plots stand in the way");
    }

    #[test]
    fn into_the_garden_next_door_by_the_street_not_over_the_hedge() {
        let (lot, tile_plot) = three_gardens("");
        let blocked = nav_of(&lot);
        let nav = NavGrid { lot: &lot, blocked: &blocked, stairs: &NO_STAIRS };
        // From their own garden to the neighbours' (both open to them: they're going there), the
        // two side by side: out at the front, along the street and in at the neighbours' front.
        let grounds = Grounds { tile_plot: &tile_plot, open: vec![true, true, false] };
        let path = nav.find_path_with((8, 3), (12, 3), Some(&grounds)).unwrap();
        assert_walkable(&nav, &path);
        assert!(path.windows(2).all(|w| (w[0].0 < 10) == (w[1].0 < 10) || w[0].1 >= 10), "over the boundary between the gardens: {path:?}");
        assert!(path.iter().any(|&(_, z)| z >= 10), "by the street: {path:?}");
        let smooth = nav.smooth_with(&path, Some(&grounds));
        for w in smooth.windows(2) {
            assert!(nav.line_walkable_with(w[0], w[1], Some(&grounds)), "{w:?}");
        }
        assert!(!nav.line_walkable_with([8.5, 3.5], [12.5, 3.5], Some(&grounds)), "no straight line over it");
    }

    #[test]
    fn a_garden_is_crossed_when_there_is_no_other_way() {
        // The street is closed off in front of the middle garden.
        let (lot, tile_plot) = three_gardens("[10,10,10,14],[20,10,20,14]");
        let blocked = nav_of(&lot);
        let nav = NavGrid { lot: &lot, blocked: &blocked, stairs: &NO_STAIRS };
        let grounds = Grounds { tile_plot: &tile_plot, open: vec![true, false, true] };
        let path = nav.find_path_with((5, 12), (25, 12), Some(&grounds)).expect("through the garden");
        assert_walkable(&nav, &path);
        assert!(path.iter().any(|&t| in_middle_garden(t)));
        // Someone standing in a garden they shouldn't be in can still get out of it, the
        // shortest way (not across the gardens either side).
        let grounds = Grounds { tile_plot: &tile_plot, open: vec![false; 3] };
        let out = nav.find_path_with((15, 2), (15, 12), Some(&grounds)).expect("out to the street");
        assert_eq!(out.len(), 11, "straight down: {out:?}");
        assert_eq!(nav.smooth_with(&out, Some(&grounds)).len(), 2, "in one straight line");
    }

    #[test]
    fn search_buffers_follow_the_lot_size() {
        // The same thread routes on a small lot, a lot grown by two storeys, and the small one
        // again: the kept buffers are resized each time and earlier searches don't leak in.
        let small = lot_with_wall();
        let mut tall = lot_with_wall();
        tall.grow_depth(30);
        let (bs, bt) = (nav_of(&small), nav_of(&tall));
        let flat = NavGrid { lot: &small, blocked: &bs, stairs: &NO_STAIRS };
        let up = NavGrid { lot: &tall, blocked: &bt, stairs: &NO_STAIRS };
        let first = flat.find_path((2, 2), (7, 2)).unwrap();
        assert!(up.find_path((2, 22), (7, 22)).is_some(), "rows on the third storey are searched");
        assert_eq!(flat.find_path((2, 2), (7, 2)).unwrap(), first);
    }
}
