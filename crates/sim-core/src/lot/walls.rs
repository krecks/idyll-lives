//! Walls and diagonals: reading and setting the edges of the grid.

use super::*;

impl Lot {
    /// Every wall, door, window, fence and gate, row by row: the horizontal edges first (`z` to
    /// `depth`), then the vertical ones (`x` to `width`), then the diagonals. Chunks with
    /// nothing built are skipped, so this costs what's built, not the lot's size.
    pub fn walls(&self) -> impl Iterator<Item = (EdgeRef, Edge)> + '_ {
        let h = (0..=self.depth).flat_map(move |z| {
            self.built_row(z, self.width).filter_map(move |(x, s)| {
                let e = self.h_edges[s];
                (e != Edge::Open).then_some((EdgeRef::H(x as u16, z as u16), e))
            })
        });
        let v = (0..self.depth).flat_map(move |z| {
            self.built_row(z, self.width + 1).filter_map(move |(x, s)| {
                let e = self.v_edges[s];
                (e != Edge::Open).then_some((EdgeRef::V(x as u16, z as u16), e))
            })
        });
        h.chain(v).chain(
            self.diagonal_walls()
                .map(|(x, z, d)| (EdgeRef::Diag(x as u16, z as u16), d.edge)),
        )
    }

    /// The walls, doors, windows, fences, gates and diagonals stored on tiles `[x0, z0, x1, z1)`
    /// (a tile holds the edges on its `-z` and `-x` sides; far edges are on the tiles one past the
    /// lot), chunk by chunk and row by row in each: what `structure_rect` sends.
    pub fn walls_in(&self, [x0, z0, x1, z1]: [i32; 4]) -> Vec<(EdgeRef, Edge)> {
        let (x0, z0) = (x0.max(0) as usize, z0.max(0) as usize);
        let (x1, z1) = (
            (x1.max(0) as usize).min(self.width + 1),
            (z1.max(0) as usize).min(self.depth + 1),
        );
        let mut out = Vec::new();
        if x0 >= x1 || z0 >= z1 {
            return out;
        }
        for cz in z0 >> SHIFT..=(z1 - 1) >> SHIFT {
            for cx in x0 >> SHIFT..=(x1 - 1) >> SHIFT {
                let c = cz * self.cols + cx;
                if self.built[c] == 0 {
                    continue;
                }
                let b = self.base[c] as usize;
                for z in (cz * CHUNK).max(z0)..(cz * CHUNK + CHUNK).min(z1) {
                    for x in (cx * CHUNK).max(x0)..(cx * CHUNK + CHUNK).min(x1) {
                        let s = b + ((z & MASK) << SHIFT | (x & MASK));
                        let (xu, zu) = (x as u16, z as u16);
                        if x < self.width && self.h_edges[s] != Edge::Open {
                            out.push((EdgeRef::H(xu, zu), self.h_edges[s]));
                        }
                        if z < self.depth && self.v_edges[s] != Edge::Open {
                            out.push((EdgeRef::V(xu, zu), self.v_edges[s]));
                        }
                        if let Some(d) = self.diags[s].filter(|_| x < self.width && z < self.depth)
                        {
                            out.push((EdgeRef::Diag(xu, zu), d.edge));
                        }
                    }
                }
            }
        }
        out
    }

    /// Every diagonal wall, row by row: `(x, z, diagonal)`.
    pub fn diagonal_walls(&self) -> impl Iterator<Item = (usize, usize, Diagonal)> + '_ {
        (0..self.depth).flat_map(move |z| {
            self.built_row(z, self.width)
                .filter_map(move |(x, s)| self.diags[s].map(|d| (x, z, d)))
        })
    }

    /// Horizontal and vertical edges per slot (each tile's `-z` and `-x` side), for counting.
    pub fn edges(&self) -> (&[Edge], &[Edge]) {
        (&self.h_edges, &self.v_edges)
    }

    /// Diagonal wall per slot (see `tile_index`).
    pub fn diagonals(&self) -> &[Option<Diagonal>] {
        &self.diags
    }

    /// Every edge and diagonal row by row, as `from_edges` takes them (for saving): the
    /// horizontal edges `width × (depth + 1)`, the vertical ones `(width + 1) × depth`, the
    /// diagonals `width × depth`.
    pub fn row_major_edges(&self) -> (Vec<Edge>, Vec<Edge>, Vec<Option<Diagonal>>) {
        let (w, d) = (self.width, self.depth);
        let mut h = vec![Edge::Open; w * (d + 1)];
        let mut v = vec![Edge::Open; (w + 1) * d];
        let mut diags = vec![None; w * d];
        for (e, edge) in self.walls() {
            match e {
                EdgeRef::H(x, z) => h[z as usize * w + x as usize] = edge,
                EdgeRef::V(x, z) => v[z as usize * (w + 1) + x as usize] = edge,
                EdgeRef::Diag(x, z) => {
                    diags[z as usize * w + x as usize] = self.diags[self.slot(x.into(), z.into())]
                }
            }
        }
        (h, v, diags)
    }

    /// The diagonal wall across tile `(x, z)`, if any (none off the lot).
    pub fn diag(&self, x: i32, z: i32) -> Option<Diagonal> {
        if self.in_bounds(x, z) {
            self.diags[self.tile_index(x, z)]
        } else {
            None
        }
    }

    /// Notes a change between built and open in chunk `c`.
    pub(crate) fn count(&mut self, c: usize, was: bool, is: bool) {
        match (was, is) {
            (false, true) => self.built[c] += 1,
            (true, false) => self.built[c] -= 1,
            _ => {}
        }
    }

    /// Sets or clears (`None`) the diagonal across a tile. Call `compute_rooms` afterwards.
    pub fn set_diag(&mut self, x: usize, z: usize, d: Option<Diagonal>) {
        let d = d.filter(|d| d.edge != Edge::Open);
        let c = self.chunk_of(x, z);
        if d.is_none() && self.base[c] == 0 {
            return;
        }
        let s = self.slot_mut(x, z);
        let old = std::mem::replace(&mut self.diags[s], d);
        self.count(c, old.is_some(), d.is_some());
        if old.map(|d| d.dir) != d.map(|d| d.dir) {
            self.note(EdgeRef::Diag(x as u16, z as u16));
        }
        if old != d {
            self.changed(LotChange::Diag(x as u16, z as u16, old));
        }
    }
}
