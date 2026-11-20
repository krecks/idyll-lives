//! How far a search reached, and what it reached first.

use super::*;

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
    /// Per slot: the steps open from the tile to each of `NEIGHBOURS` (bit `k`), stairs aside
    /// (`NavGrid::step_open`): worked out with the labels, so a search reads them rather than
    /// working each step out. Only for chunks `stepped` says are up to date.
    steps: Vec<u8>,
    /// Per chunk: its tiles' `steps` are as the lot is (labelled since anything near changed).
    stepped: Vec<bool>,
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
                let k = cz * cols + cx;
                self.dirty.push(k as u32);
                if let Some(s) = self.stepped.get_mut(k) {
                    *s = false;
                }
            }
        }
    }

    /// The steps open from slot `s` (a tile of chunk `c`) to each of `NEIGHBOURS` (bit `k`),
    /// stairs aside, if its chunk's are up to date (`steps`). None for the void slot: a chunk not
    /// allocated has no tiles to work steps out for (`step_chunk`), though one may still be stood
    /// on. (A way out of town on the ground's far side is on its last row's far edge, and the row
    /// past it is the storey above's first: in Foundrygate someone set down at one looked up the
    /// void slot's steps, none, where their step back onto the ground was open, and found no way.)
    pub(crate) fn steps_at(&self, c: usize, s: usize) -> Option<u8> {
        if s == crate::lot::VOID || self.all_dirty || !self.stepped.get(c).copied().unwrap_or(false)
        {
            return None;
        }
        self.steps.get(s).copied()
    }

    /// Works out the steps from every tile of chunk `c` (`steps`), labelled just before: on plain
    /// ground (`plain`) every step inside the chunk is open, so only its rim's are worked out.
    fn step_chunk(&mut self, nav: &NavGrid, c: usize) {
        let lot = nav.lot;
        let open_at = |x: i32, z: i32| {
            NEIGHBOURS
                .into_iter()
                .enumerate()
                .filter(|&(_, (dx, dz))| nav.step_open(x, z, x + dx, z + dz))
                .fold(0u8, |m, (k, _)| m | 1 << k)
        };
        if self.plain[c] {
            for (_, _, s) in lot.chunk_tiles(c) {
                self.steps[s] = u8::MAX;
            }
            for (x, z) in lot.chunk_rim(c) {
                self.steps[lot.tile_index(x, z)] = open_at(x, z);
            }
        } else {
            for (x, z, s) in lot.chunk_tiles(c) {
                self.steps[s] = open_at(x, z);
            }
        }
        self.stepped[c] = true;
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
        self.steps.resize(lot.tile_slots(), 0);
        self.stepped.resize(chunks, false);
        // Chunks a flight of stairs is in (steps, feet, tops and landings): never plain.
        let mut stairs: Vec<u32> = nav
            .stairs
            .stairs
            .iter()
            .flat_map(|s| s.tiles().iter().copied().chain([s.foot, s.landing]))
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
                self.step_chunk(nav, c);
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
                self.step_chunk(nav, c as usize);
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
        // (On tiles kept off: from where the way off them comes off.)
        let a = nav
            .way_out(a)
            .and_then(|way| way.last().copied())
            .unwrap_or(a);
        match self.of(nav.lot, a) {
            Some(from) => from == to,
            None if nav.lot.in_bounds(a.0, a.1) => nav
                .neighbours(a.0, a.1)
                .any(|n| self.of(nav.lot, n) == Some(to)),
            None => true,
        }
    }
}
