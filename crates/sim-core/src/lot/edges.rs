//! Edges and what is on them: walls, looks and floors, tile by tile.
//!
//! Split from `lot.rs` (see `mod.rs`).

use super::*;

impl Lot {
    pub fn h_edge(&self, x: usize, z: usize) -> Edge {
        self.h_edges[self.slot(x, z)]
    }

    pub fn v_edge(&self, x: usize, z: usize) -> Edge {
        self.v_edges[self.slot(x, z)]
    }

    pub fn set_h(&mut self, x: usize, z: usize, e: Edge) {
        let c = self.chunk_of(x, z);
        if e == Edge::Open && self.base[c] == 0 {
            return;
        }
        let s = self.slot_mut(x, z);
        let old = std::mem::replace(&mut self.h_edges[s], e);
        self.count(c, old != Edge::Open, e != Edge::Open);
        if old.is_wall() != e.is_wall() {
            self.note(EdgeRef::H(x as u16, z as u16));
        }
        if old != e {
            self.changed(LotChange::H(x as u16, z as u16, old));
        }
    }

    pub fn set_v(&mut self, x: usize, z: usize, e: Edge) {
        let c = self.chunk_of(x, z);
        if e == Edge::Open && self.base[c] == 0 {
            return;
        }
        let s = self.slot_mut(x, z);
        let old = std::mem::replace(&mut self.v_edges[s], e);
        self.count(c, old != Edge::Open, e != Edge::Open);
        if old.is_wall() != e.is_wall() {
            self.note(EdgeRef::V(x as u16, z as u16));
        }
        if old != e {
            self.changed(LotChange::V(x as u16, z as u16, old));
        }
    }

    /// How the wall at `e` looks (the default look where there's no wall).
    pub fn look(&self, e: EdgeRef) -> EdgeLook {
        self.looks.get(&e).copied().unwrap_or_default()
    }

    /// Sets the look of the wall at `e` (the default look is not stored).
    pub fn set_look(&mut self, e: EdgeRef, look: EdgeLook) {
        let old = self.look(e);
        if old != look {
            self.changed(LotChange::Look(e, old));
        }
        if look == EdgeLook::default() {
            self.looks.remove(&e);
        } else {
            self.looks.insert(e, look);
        }
    }

    /// Every wall that doesn't have the default look, in a stable order (for saving and views).
    pub fn looks(&self) -> impl Iterator<Item = (EdgeRef, EdgeLook)> + '_ {
        self.looks.iter().map(|(e, l)| (*e, *l))
    }

    /// Floor covering of tile `(x, z)` (0: the automatic look).
    pub fn floor(&self, x: u16, z: u16) -> u8 {
        self.floors.get(&(x, z)).copied().unwrap_or(0)
    }

    /// Sets the floor covering of tile `(x, z)` (0, the automatic look, is not stored).
    pub fn set_floor(&mut self, x: u16, z: u16, covering: u8) {
        let old = self.floor(x, z);
        if old != covering {
            self.changed(LotChange::Floor(x, z, old));
        }
        if covering == 0 {
            self.floors.remove(&(x, z));
        } else {
            self.floors.insert((x, z), covering);
        }
    }

    /// Every tile with a floor covering, as `(x, z, covering)` in a stable order.
    pub fn floors(&self) -> impl Iterator<Item = (u16, u16, u8)> + '_ {
        self.floors.iter().map(|(&(x, z), &c)| (x, z, c))
    }

    pub fn in_bounds(&self, x: i32, z: i32) -> bool {
        x >= 0 && z >= 0 && (x as usize) < self.width && (z as usize) < self.depth
    }

    /// The slot of tile `(x, z)` (see `Lot`): where per-tile arrays keep its values. A tile
    /// whose chunk isn't allocated, or off the chunk grid, has the void slot.
    pub fn tile_index(&self, x: i32, z: i32) -> usize {
        let (x, z) = (x as usize, z as usize);
        let c = (z >> SHIFT)
            .wrapping_mul(self.cols)
            .wrapping_add(x >> SHIFT);
        match self.base.get(c).copied().unwrap_or(0) as usize {
            b if b != 0 && x <= self.width && z <= self.depth => {
                b + ((z & MASK) << SHIFT | (x & MASK))
            }
            _ => VOID,
        }
    }

    /// Edge between two orthogonally adjacent in-bounds tiles.
    pub(crate) fn edge_between(&self, ax: i32, az: i32, bx: i32, bz: i32) -> Edge {
        match (bx - ax, bz - az) {
            (1, 0) => self.v_edge(bx as usize, az as usize),
            (-1, 0) => self.v_edge(ax as usize, az as usize),
            (0, 1) => self.h_edge(ax as usize, bz as usize),
            (0, -1) => self.h_edge(ax as usize, az as usize),
            _ => unreachable!("tiles are not orthogonally adjacent"),
        }
    }

    /// Whether a Sim can walk across the edge between two orthogonally adjacent tiles.
    pub fn edge_walkable(&self, ax: i32, az: i32, bx: i32, bz: i32) -> bool {
        self.in_bounds(bx, bz) && !self.edge_between(ax, az, bx, bz).blocks()
    }
}
