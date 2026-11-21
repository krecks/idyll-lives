//! Rooms: the flood that finds them and what a tile belongs to.

use super::*;

impl Lot {
    /// The tiles `[x0, z0, x1, z1)` round room `id` (stacked rows), if there is one.
    pub fn room_rect(&self, id: u16) -> Option<[i32; 4]> {
        (id as usize)
            .checked_sub(1)
            .and_then(|k| self.room_meta.get(k))
            .map(RoomMeta::rect)
    }

    /// How many rooms the lot has (their ids are `1..=room_count`).
    pub fn room_count(&self) -> usize {
        self.room_meta.len()
    }

    /// Whether a diagonal wall or window stands across the tile: nobody can stand on it.
    pub fn diag_blocks(&self, x: i32, z: i32) -> bool {
        self.diag(x, z).is_some_and(|d| d.edge.blocks())
    }

    /// Whether tile `(x, z)` has a diagonal ending at grid corner `(cx, cz)`.
    pub fn diag_touches(&self, x: i32, z: i32, cx: i32, cz: i32) -> bool {
        self.diag(x, z).is_some_and(|d| d.touches(x, z, cx, cz))
    }

    /// Which half (0 or 1) of tile `(x, z)` touches `side`; always 0 without a diagonal.
    pub fn half_at_side(&self, x: i32, z: i32, side: Side) -> usize {
        let Some(d) = self.diag(x, z) else {
            return 0;
        };
        match (side, d.dir) {
            (Side::NegZ, _) | (Side::PosX, DiagDir::Dp) | (Side::NegX, DiagDir::Dn) => 0,
            _ => 1,
        }
    }

    /// Room of half `half` (0 or 1) of a tile; `OUTDOORS` off the lot.
    pub fn half_room(&self, x: i32, z: i32, half: usize) -> u16 {
        if self.in_bounds(x, z) {
            self.halves[self.tile_index(x, z)][half & 1]
        } else {
            OUTDOORS
        }
    }

    /// Rooms of both halves of every slot (see `tile_index`; equal for tiles without a diagonal).
    pub fn half_rooms(&self) -> &[[u16; 2]] {
        &self.halves
    }
}

impl Lot {
    pub fn room_at(&self, x: i32, z: i32) -> u16 {
        if self.in_bounds(x, z) {
            self.rooms[self.tile_index(x, z)]
        } else {
            OUTDOORS
        }
    }

    /// Room id per slot (see `tile_index`).
    pub fn rooms(&self) -> &[u16] {
        &self.rooms
    }

    /// Room id per tile, row by row (`width × depth`), for views.
    pub fn rooms_row_major(&self) -> Vec<u16> {
        let mut out = vec![OUTDOORS; self.width * self.depth];
        for (x, z, s) in self.tiles() {
            out[z as usize * self.width + x as usize] = self.rooms[s];
        }
        out
    }
}

impl Lot {
    /// Finds the rooms: over the whole lot the first time (`compute_all_rooms`), then only round
    /// the walls built or taken down since (`rooms_near`, the same rooms numbered the same).
    pub fn compute_rooms(&mut self) {
        let done = match self.pending.take() {
            Some(edges) => self.rooms_near(&edges),
            None => false,
        };
        if !done {
            self.compute_all_rooms();
        }
        self.pending = Some(Vec::new());
    }

    /// Flood-fills rooms over tile halves. Walls, doors and windows (straight or diagonal)
    /// separate rooms; any area touching the lot border is outdoors. Rooms are numbered in the
    /// order of their first tile row by row, as if every chunk were there: a chunk that isn't
    /// is one open square, walked across as a whole, and allocated if it turns out to be inside
    /// a room.
    pub fn compute_all_rooms(&mut self) {
        struct Region {
            key: u64,
            nodes: std::ops::Range<usize>,
            voids: std::ops::Range<usize>,
        }
        self.halves.fill([UNSET; 2]);
        let width = self.width;
        let key = |x: i32, z: i32, h: usize| (z as u64 * width as u64 + x as u64) * 2 + h as u64;
        let mut void_seen = vec![false; self.base.len()];
        let mut regions: Vec<Region> = Vec::new();
        let (mut nodes, mut voids) = (Vec::new(), Vec::new());
        let mut queue = VecDeque::new();
        for z in 0..self.depth {
            for cx in 0..self.cols {
                let b = self.base[(z >> SHIFT) * self.cols + cx] as usize;
                if b == 0 {
                    continue;
                }
                for x in cx * CHUNK..(cx * CHUNK + CHUNK).min(width) {
                    let si = b + ((z & MASK) << SHIFT | (x & MASK));
                    for sh in 0..2 {
                        if (sh == 1 && self.diags[si].is_none()) || self.halves[si][sh] != UNSET {
                            continue;
                        }
                        let (n0, v0) = (nodes.len(), voids.len());
                        let mut border = false;
                        let mut first = u64::MAX;
                        self.halves[si][sh] = SEEN;
                        queue.push_back(RoomNode::Tile(si, sh, x as i32, z as i32));
                        while let Some(node) = queue.pop_front() {
                            let mut go = |lot: &mut Lot, x: i32, z: i32, side: Side| {
                                border |= lot.flood_step(x, z, side, &mut queue, &mut void_seen);
                            };
                            match node {
                                RoomNode::Tile(i, h, tx, tz) => {
                                    nodes.push((i, h));
                                    first = first.min(key(tx, tz, h));
                                    for side in Side::ALL {
                                        if self.diags[i].is_none()
                                            || self.half_at_side(tx, tz, side) == h
                                        {
                                            go(self, tx, tz, side);
                                        }
                                    }
                                }
                                RoomNode::Void(c) => {
                                    voids.push(c);
                                    let (x0, z0) =
                                        ((c % self.cols) * CHUNK, (c / self.cols) * CHUNK);
                                    let (x1, z1) =
                                        ((x0 + CHUNK).min(width), (z0 + CHUNK).min(self.depth));
                                    first = first.min(key(x0 as i32, z0 as i32, 0));
                                    for tz in z0 as i32..z1 as i32 {
                                        go(self, x0 as i32, tz, Side::NegX);
                                        go(self, x1 as i32 - 1, tz, Side::PosX);
                                    }
                                    for tx in x0 as i32..x1 as i32 {
                                        go(self, tx, z0 as i32, Side::NegZ);
                                        go(self, tx, z1 as i32 - 1, Side::PosZ);
                                    }
                                }
                            }
                        }
                        if border {
                            for &(i, h) in &nodes[n0..] {
                                self.halves[i][h] = OUTDOORS;
                            }
                            nodes.truncate(n0);
                            voids.truncate(v0);
                        } else {
                            regions.push(Region {
                                key: first,
                                nodes: n0..nodes.len(),
                                voids: v0..voids.len(),
                            });
                        }
                    }
                }
            }
        }
        // Numbered by their first tile, row by row.
        regions.sort_unstable_by_key(|r| r.key);
        for (k, r) in regions.iter().enumerate() {
            let id = k as u16 + 1;
            for &(i, h) in &nodes[r.nodes.clone()] {
                self.halves[i][h] = id;
            }
            // A chunk inside a room has a floor to stand on: it's there from now on.
            for &c in &voids[r.voids.clone()] {
                self.alloc(c);
                let b = self.base[c] as usize;
                let (x0, z0) = ((c % self.cols) * CHUNK, (c / self.cols) * CHUNK);
                for tz in z0..(z0 + CHUNK).min(self.depth) {
                    for tx in x0..(x0 + CHUNK).min(width) {
                        self.halves[b + ((tz & MASK) << SHIFT | (tx & MASK))] = [id, UNSET];
                    }
                }
            }
        }
        self.indoor.fill(0);
        for (i, h) in self.halves.iter_mut().enumerate() {
            if h[0] == UNSET {
                // The void slot, and tiles of edge chunks off the lot.
                *h = [OUTDOORS; 2];
            }
            if self.diags[i].is_none() {
                h[1] = h[0];
            }
            self.rooms[i] = if h[0] != OUTDOORS { h[0] } else { h[1] };
            if self.rooms[i] != OUTDOORS {
                self.indoor[self.order[(i - 1) / CHUNK_TILES] as usize] += 1;
            }
        }
        // Each room's key and the tiles round it, over the chunks with rooms in them.
        let mut rects = vec![[i32::MAX, i32::MAX, i32::MIN, i32::MIN]; regions.len()];
        for c in 0..self.indoor.len() {
            if self.indoor[c] == 0 {
                continue;
            }
            let [x0, z0, x1, z1] = self.chunk_rect(c);
            let b = self.base[c] as usize;
            for z in z0..z1 {
                for x in x0..x1 {
                    let s = b + ((z as usize & MASK) << SHIFT | (x as usize & MASK));
                    for id in self.halves[s] {
                        if id != OUTDOORS {
                            grow(&mut rects[id as usize - 1], x, z);
                        }
                    }
                }
            }
        }
        self.room_meta = regions
            .iter()
            .zip(rects)
            .map(|(r, rect)| RoomMeta::new(r.key, rect))
            .collect();
        self.changes = RoomChanges {
            full: true,
            ..Default::default()
        };
    }

    /// One step of the room flood: from tile `(x, z)` across `side`. Queues the tile there (or
    /// its chunk, if that isn't allocated) unless a wall is in the way or it was seen already;
    /// returns whether the step leaves the lot.
    fn flood_step(
        &mut self,
        x: i32,
        z: i32,
        side: Side,
        queue: &mut VecDeque<RoomNode>,
        void_seen: &mut [bool],
    ) -> bool {
        let (dx, dz) = side.offset();
        let (nx, nz) = (x + dx, z + dz);
        if !self.in_bounds(nx, nz) {
            return true;
        }
        if self.edge_between(x, z, nx, nz).is_wall() {
            return false;
        }
        let nc = self.chunk_of(nx as usize, nz as usize);
        if self.base[nc] == 0 {
            if !std::mem::replace(&mut void_seen[nc], true) {
                queue.push_back(RoomNode::Void(nc));
            }
            return false;
        }
        let n = self.tile_index(nx, nz);
        let nh = self.half_at_side(nx, nz, side.opposite());
        if self.halves[n][nh] == UNSET {
            self.halves[n][nh] = SEEN;
            queue.push_back(RoomNode::Tile(n, nh, nx, nz));
        }
        false
    }
}

/// Room flood markers (`Lot::compute_rooms`): not reached yet, and reached by this flood.
const UNSET: u16 = u16::MAX;
const SEEN: u16 = u16::MAX - 1;

/// A node of the room flood: half `h` of the tile in slot `i` at `(x, z)`, or a whole chunk
/// that isn't allocated (all open).
#[derive(Clone, Copy)]
pub(super) enum RoomNode {
    Tile(usize, usize, i32, i32),
    Void(usize),
}

/// Grows tiles `r` (`[x0, z0, x1, z1)`) over tile `(x, z)`.
pub(super) fn grow(r: &mut [i32; 4], x: i32, z: i32) {
    r[0] = r[0].min(x);
    r[1] = r[1].min(z);
    r[2] = r[2].max(x + 1);
    r[3] = r[3].max(z + 1);
}
