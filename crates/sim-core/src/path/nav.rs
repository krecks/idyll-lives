//! The navigation grid: tiles, edges and what may be walked.

use super::*;

/// Read-only view of what is walkable: lot edges plus tiles blocked by objects, storeys and stairs.
#[derive(Clone, Copy)]
pub struct NavGrid<'a> {
    pub lot: &'a Lot,
    pub blocked: &'a [bool],
    pub stairs: &'a StairMap,
    /// Of the blocked tiles, those blocked only to anyone stepping onto them (sorted tile
    /// indices; `Travel::junction_tiles`, a lit junction's box and carriageway): one standing on
    /// one is walked off them by the nearest way (`way_out`), never left there. Empty: none.
    pub kept_off: &'a [usize],
    /// The walkable regions, with each tile's steps (`Reach::steps`), when they were refreshed
    /// for this grid (the world's own: `World::nav`): a search reads a tile's steps from them
    /// rather than working each out. None: every step is worked out (`can_step`).
    pub steps: Option<&'a Reach>,
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

    /// Whether `(x, z)` is one of the tiles blocked only to those stepping onto them (`kept_off`).
    pub fn kept_off_at(&self, x: i32, z: i32) -> bool {
        !self.kept_off.is_empty()
            && self.lot.in_bounds(x, z)
            && self
                .kept_off
                .binary_search(&self.lot.tile_index(x, z))
                .is_ok()
    }

    /// From `start`, on a tile kept off (`kept_off`), the nearest way off them: a step at a time
    /// (no diagonals) over them, to the first tile anyone may stand on; `start` first. None: not
    /// on one, or no way off within `WAY_OUT_MOST` tiles. Kept off is only stepping onto them:
    /// one already on them (set down there, or there when they were first kept off: Canalside's
    /// Mila, saved in a junction's box, stood there for a day, every way out shut to her, and a
    /// T's cars round her) always gets off.
    pub fn way_out(&self, start: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        if !self.kept_off_at(start.0, start.1) {
            return None;
        }
        let lot = self.lot;
        let mut came: std::collections::HashMap<(i32, i32), (i32, i32)> =
            std::collections::HashMap::new();
        let mut todo = std::collections::VecDeque::from([start]);
        came.insert(start, start);
        while let Some((x, z)) = todo.pop_front() {
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if came.contains_key(&(nx, nz))
                    || !lot.in_bounds(nx, nz)
                    || !lot.edge_walkable(x, z, nx, nz)
                {
                    continue;
                }
                let free = self.can_step(x, z, nx, nz);
                if !free
                    && !(self.kept_off_at(nx, nz)
                        && lot.allocated(nx, nz)
                        && !lot.diag_blocks(nx, nz))
                {
                    continue;
                }
                came.insert((nx, nz), (x, z));
                if free {
                    let mut way = vec![(nx, nz)];
                    let mut at = (x, z);
                    while at != start {
                        way.push(at);
                        at = came[&at];
                    }
                    way.push(start);
                    way.reverse();
                    return Some(way);
                }
                if came.len() > WAY_OUT_MOST {
                    return None;
                }
                todo.push_back((nx, nz));
            }
        }
        None
    }

    /// Whether a single step from `a` to the neighbouring tile `b` is allowed.
    pub fn can_step(&self, ax: i32, az: i32, bx: i32, bz: i32) -> bool {
        self.tile_free(bx, bz)
            && self.stairs_allow(ax, az, bx, bz)
            && self.walls_allow(ax, az, bx, bz)
    }

    /// `can_step`'s stairs: a stair is walked straight up and down, entered at its foot (any other
    /// step: allowed).
    fn stairs_allow(&self, ax: i32, az: i32, bx: i32, bz: i32) -> bool {
        let (dx, dz) = (bx - ax, bz - az);
        if !self.stairs.stairs.is_empty() {
            let from = self.stairs.step(ax, az);
            let to = self.stairs.step(bx, bz);
            match (from, to) {
                (None, None) => {}
                (Some((a, i)), Some((b, j))) => {
                    // From one step to the next (or back down one), along the stair: never across
                    // a spiral's winding from its bottom step to its top, nor onto another stair.
                    if a != b || (j - i).abs() != 1 {
                        return false;
                    }
                    // `dir` is the way onto a step: up, the way onto the higher one; down, the
                    // way onto it, walked backwards.
                    let along = a.dir(i.max(j));
                    let travel = if j > i { along } else { (-along.0, -along.1) };
                    if (dx, dz) != travel {
                        return false;
                    }
                }
                (Some((s, i)), None) => {
                    // Off the bottom step: a flight is left the way it's entered (its foot); a
                    // spiral from any side.
                    if i != 0 || (!s.open && (bx, bz) != s.foot) {
                        return false;
                    }
                }
                (None, Some((s, i))) => {
                    if i != 0 || (!s.open && (ax, az) != s.foot) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// `can_step` but for the stairs: `b` free and nothing built in the way (`Reach::steps`
    /// keeps it per tile).
    pub(crate) fn step_open(&self, ax: i32, az: i32, bx: i32, bz: i32) -> bool {
        self.tile_free(bx, bz) && self.walls_allow(ax, az, bx, bz)
    }

    /// `can_step`'s walls, fences, diagonals and furniture between `a` and `b`.
    fn walls_allow(&self, ax: i32, az: i32, bx: i32, bz: i32) -> bool {
        let (dx, dz) = (bx - ax, bz - az);
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
        crate::profile::phase("path/find_path_with", || {
            self.find_path_timed(start, goal, grounds)
        })
    }

    /// `find_path_with`'s work (timed as a whole there, with the `profile` feature).
    fn find_path_timed(
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
        // (On tiles kept off: off them the nearest way first, then on from there.)
        if !self.tile_free(start.0, start.1)
            && let Some(mut way) = self.way_out(start)
        {
            let off = way.pop()?;
            way.extend(self.find_path_timed(off, goal, grounds)?);
            return Some(way);
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
                // (The tile's steps as its walkable regions have them, when they're up to date.)
                let known = self.steps.and_then(|r| r.steps_at(lot.chunk_at(x, z), i));
                for (d, (dx, dz)) in NEIGHBOURS
                    .into_iter()
                    .chain(link.map(|(lx, lz)| (lx - x, lz - z)))
                    .enumerate()
                {
                    let (nx, nz) = (x + dx, z + dz);
                    let linked = Some((nx, nz)) == link;
                    let step = |d: usize| match known {
                        Some(steps) if d < NEIGHBOURS.len() => {
                            let can = steps & (1 << d) != 0 && self.stairs_allow(x, z, nx, nz);
                            debug_assert_eq!(can, self.can_step(x, z, nx, nz));
                            can
                        }
                        _ => self.can_step(x, z, nx, nz),
                    };
                    if !linked && ((dx.abs() > 1 || dz.abs() > 1) || !step(d)) {
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
            let ok = matches!(found, Search::Found(_));
            STATS.with_borrow_mut(|s| s.add(expanded, ok));
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
