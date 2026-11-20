//! Chunks: the tiles of a lot, allocated as they are built on.

use super::*;

impl Lot {
    /// The chunk holding tile `(x, z)` (either may be one past the lot, for far edges).
    pub(super) fn chunk_of(&self, x: usize, z: usize) -> usize {
        (z >> SHIFT) * self.cols + (x >> SHIFT)
    }

    /// The slot of tile `(x, z)` (either may be one past the lot); `VOID` if its chunk isn't
    /// allocated.
    pub(super) fn slot(&self, x: usize, z: usize) -> usize {
        match self.base[self.chunk_of(x, z)] as usize {
            0 => VOID,
            b => b + ((z & MASK) << SHIFT | (x & MASK)),
        }
    }

    /// Allocates chunk `c` (its slots go on the end, all open and outdoors).
    pub(super) fn alloc(&mut self, c: usize) {
        if self.base[c] != 0 {
            return;
        }
        self.base[c] = (1 + self.order.len() * CHUNK_TILES) as u32;
        self.order.push(c as u32);
        let n = self.h_edges.len() + CHUNK_TILES;
        self.h_edges.resize(n, Edge::Open);
        self.v_edges.resize(n, Edge::Open);
        self.diags.resize(n, None);
        self.rooms.resize(n, OUTDOORS);
        self.halves.resize(n, [OUTDOORS; 2]);
    }

    /// The slot of tile `(x, z)` to write to, allocating its chunk first.
    pub(super) fn slot_mut(&mut self, x: usize, z: usize) -> usize {
        let c = self.chunk_of(x, z);
        self.alloc(c);
        let s = self.slot(x, z);
        debug_assert!(s != VOID, "never written: the void slot");
        s
    }

    /// Makes sure the chunks over tiles `[x0, x1) × [z0, z1)` are allocated (clamped to the lot,
    /// far edges included). Returns whether any was new: per-tile arrays elsewhere must then
    /// grow to `tile_slots` (`World::sync_tile_arrays`).
    pub fn ensure(&mut self, x0: usize, z0: usize, x1: usize, z1: usize) -> bool {
        let (x1, z1) = (x1.min(self.width + 1), z1.min(self.depth + 1));
        if x0 >= x1 || z0 >= z1 {
            return false;
        }
        let before = self.order.len();
        for cz in z0 >> SHIFT..=(z1 - 1) >> SHIFT {
            for cx in x0 >> SHIFT..=(x1 - 1) >> SHIFT {
                self.alloc(cz * self.cols + cx);
            }
        }
        self.order.len() != before
    }

    /// `ensure` for one tile (one past the lot either way is fine, for far edges).
    pub fn ensure_tile(&mut self, x: i32, z: i32) -> bool {
        if x < 0 || z < 0 {
            return false;
        }
        self.ensure(x as usize, z as usize, x as usize + 1, z as usize + 1)
    }

    /// The chunk grid: chunks across and down (one tile more than the lot each way).
    pub fn chunk_grid(&self) -> (usize, usize) {
        (self.cols, self.rows)
    }

    /// The chunk (row-major over `chunk_grid`) holding tile `(x, z)`, which must be on the
    /// chunk grid.
    pub fn chunk_at(&self, x: i32, z: i32) -> usize {
        self.chunk_of(x as usize, z as usize)
    }

    /// Whether anything (a wall, door, window, fence, gate or diagonal) is built in chunk `c`.
    pub fn chunk_built(&self, c: usize) -> bool {
        self.built.get(c).is_some_and(|&n| n > 0)
    }

    /// Whether chunk `c` is allocated.
    pub fn chunk_allocated(&self, c: usize) -> bool {
        self.base.get(c).is_some_and(|&b| b != 0)
    }

    /// The tiles `[x0, z0, x1, z1)` chunk `c` covers on the lot (empty past its sides).
    pub fn chunk_rect(&self, c: usize) -> [i32; 4] {
        let (x0, z0) = ((c % self.cols) * CHUNK, (c / self.cols) * CHUNK);
        [
            x0 as i32,
            z0 as i32,
            (x0 + CHUNK).min(self.width) as i32,
            (z0 + CHUNK).min(self.depth) as i32,
        ]
    }

    /// The tiles of chunk `c` on the lot, row by row: `(x, z, slot)` (none if it isn't
    /// allocated).
    pub fn chunk_tiles(&self, c: usize) -> impl Iterator<Item = (i32, i32, usize)> + '_ {
        let b = self.base.get(c).copied().unwrap_or(0) as usize;
        let [x0, z0, x1, z1] = if b == 0 { [0; 4] } else { self.chunk_rect(c) };
        (z0..z1).flat_map(move |z| {
            (x0..x1).map(move |x| {
                (
                    x,
                    z,
                    b + ((z as usize & MASK) << SHIFT | (x as usize & MASK)),
                )
            })
        })
    }

    /// The tiles round the sides of chunk `c` on the lot (none if it isn't allocated).
    pub fn chunk_rim(&self, c: usize) -> impl Iterator<Item = (i32, i32)> + '_ {
        let [x0, z0, x1, z1] = self.chunk_rect(c);
        self.chunk_tiles(c)
            .filter(move |&(x, z, _)| x == x0 || z == z0 || x == x1 - 1 || z == z1 - 1)
            .map(|(x, z, _)| (x, z))
    }

    /// Whether tile `(x, z)`'s chunk is allocated.
    pub fn allocated(&self, x: i32, z: i32) -> bool {
        self.tile_index(x, z) != VOID
    }

    /// How many slots per-tile arrays need (the void slot and every allocated chunk's).
    pub fn tile_slots(&self) -> usize {
        1 + self.order.len() * CHUNK_TILES
    }

    /// The tile a slot holds (`(-1, -1)` for the void slot). It may be past the lot's sides: a
    /// chunk on the edge has slots for tiles off the lot, which stay open and outdoors.
    pub fn tile_xz(&self, slot: usize) -> (i32, i32) {
        if slot == VOID || slot >= self.tile_slots() {
            return (-1, -1);
        }
        let c = self.order[(slot - 1) / CHUNK_TILES] as usize;
        let local = (slot - 1) % CHUNK_TILES;
        (
            ((c % self.cols) * CHUNK + (local & MASK)) as i32,
            ((c / self.cols) * CHUNK + (local >> SHIFT)) as i32,
        )
    }

    /// Every tile of the chunks with a room in them, row by row: `(x, z, slot)` (outdoor tiles
    /// of those chunks too). Whole-lot passes over rooms cost the houses, not the land.
    pub fn indoor_tiles(&self) -> impl Iterator<Item = (i32, i32, usize)> + '_ {
        (0..self.depth).flat_map(move |z| {
            let first = (z >> SHIFT) * self.cols;
            (0..self.cols)
                .filter(move |&cx| self.indoor[first + cx] > 0)
                .flat_map(move |cx| {
                    let b = self.base[first + cx] as usize + ((z & MASK) << SHIFT);
                    (cx * CHUNK..(cx * CHUNK + CHUNK).min(self.width))
                        .map(move |x| (x as i32, z as i32, b + (x & MASK)))
                })
        })
    }

    /// Every allocated tile on the lot, row by row: `(x, z, slot)`.
    pub fn tiles(&self) -> impl Iterator<Item = (i32, i32, usize)> + '_ {
        (0..self.depth).flat_map(move |z| self.row(z, self.width))
    }

    /// The allocated tiles of row `z` with `x < width`, left to right: `(x, z, slot)`.
    fn row(&self, z: usize, width: usize) -> impl Iterator<Item = (i32, i32, usize)> + '_ {
        let first = (z >> SHIFT) * self.cols;
        (0..self.cols)
            .filter_map(move |cx| match self.base[first + cx] as usize {
                0 => None,
                b => Some((cx, b + ((z & MASK) << SHIFT))),
            })
            .flat_map(move |(cx, b)| {
                (cx * CHUNK..(cx * CHUNK + CHUNK).min(width))
                    .map(move |x| (x as i32, z as i32, b + (x & MASK)))
            })
    }

    /// Row `z`'s slots with `x < width` in chunks with something built, left to right.
    pub(super) fn built_row(
        &self,
        z: usize,
        width: usize,
    ) -> impl Iterator<Item = (usize, usize)> + '_ {
        let first = (z >> SHIFT) * self.cols;
        (0..self.cols)
            .filter(move |&cx| self.built[first + cx] > 0)
            .flat_map(move |cx| {
                let b = self.base[first + cx] as usize + ((z & MASK) << SHIFT);
                (cx * CHUNK..(cx * CHUNK + CHUNK).min(width)).map(move |x| (x, b + (x & MASK)))
            })
    }
}
