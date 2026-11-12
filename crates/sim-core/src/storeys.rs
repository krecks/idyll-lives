//! Storeys: houses can grow upwards (content `build.storeys`) and down into a basement (content
//! `build.basement`).
//!
//! The lot holds every storey as rows of the same grid, one block of `depth` rows per storey,
//! laid out like the ground (the same `x`, and `z` shifted by the block's first row): the ground
//! is block 0, the storeys above follow, and the basement (storey -1) has the block after them
//! (`StairMap::basement`; storeys added later still go on the end, so no block ever moves).
//! Walls, rooms, floors, furniture, pathfinding and saving work on every storey as they do on
//! the ground. Off the ground there is only floor inside rooms; a room upstairs may only stand
//! above a room, while the basement may reach under the garden (but not past the plot), has no
//! windows and isn't outdoors anywhere anyone can stand. Stairs (objects marked `stairs`) link a
//! storey to the one above: residents walk up their steps and step off the top onto the landing
//! above (`path::StairMap`); the tiles above the steps are the stairwell, open to below. Stairs
//! down to the basement are stairs standing in the basement, coming up inside the house. Views
//! get positions on the ground rows plus a height (`World::sim_height`, -1 in the basement), and
//! the renderer draws each storey at its height.

use crate::Error;
use crate::content::Content;
use crate::lot::OUTDOORS;
use crate::path::{Stair, StairMap, StairShape};
use crate::world::{ObjectInstance, Sim, World};

impl World {
    /// Rows per storey (the town's own depth).
    pub fn storey_depth(&self) -> i32 {
        if self.stairs.depth > 0 {
            self.stairs.depth
        } else {
            self.lot.depth as i32
        }
    }

    /// The storey a row is on (0: the ground, -1: the basement).
    pub fn storey_of(&self, z: i32) -> i32 {
        self.stairs.storey(z)
    }

    /// The first row of a storey (see `StairMap::storey_row`).
    pub fn storey_row(&self, storey: i32) -> i32 {
        self.stairs.storey_row(storey)
    }

    /// The lowest storey: -1 with a basement, else 0.
    pub fn lowest_storey(&self) -> i32 {
        if self.stairs.basement > 0 { -1 } else { 0 }
    }

    /// The storeys the lot holds, from the lowest up.
    pub fn storey_range(&self) -> std::ops::Range<i32> {
        self.lowest_storey()..self.storeys as i32
    }

    /// The same place on the ground storey's rows.
    pub fn ground_z(&self, z: i32) -> i32 {
        self.stairs.ground_z(z)
    }

    /// How the lot's storeys and stairs are walked.
    pub fn stair_map(&self) -> &StairMap {
        &self.stairs
    }

    /// Gives the lot as many storeys as the content allows, and a basement if it allows one
    /// (new games, and older saves). New blocks of rows go on the end: storeys above first, then
    /// the basement; with a basement already there, storeys above go after it.
    pub(crate) fn make_storeys(&mut self) {
        let want = self.content.build.storeys.max(1).max(self.storeys);
        let basement = self.content.build.basement && self.stairs.basement == 0;
        if want == self.storeys && !basement {
            return;
        }
        let depth = self.storey_depth() as usize;
        let mut blocks = self.storeys as usize + usize::from(self.stairs.basement > 0);
        blocks += (want - self.storeys) as usize;
        if basement {
            self.stairs.basement = blocks as i32;
            blocks += 1;
        }
        self.lot.grow_depth(depth * blocks);
        self.sync_tile_arrays();
        self.storeys = want;
        self.stairs.depth = depth as i32;
        self.refresh_stairs();
        self.lot_version = self.lot_version.wrapping_add(1);
        self.nav_changed();
        self.rooms_layout_changed();
    }

    /// Rebuilds the stairs list from the stair objects (after objects come or go).
    pub(crate) fn refresh_stairs(&mut self) {
        let stairs: Vec<Stair> = self
            .objects
            .iter()
            .filter_map(|o| stair_of(&self.content, o, &self.stairs))
            .collect();
        self.stairs.set_stairs(stairs);
    }

    /// The tiles an object takes from walking: its footprint, or for stairs the stairwell above
    /// its steps (the steps themselves are walked on).
    pub(crate) fn blocking_tiles(&self, obj: &ObjectInstance) -> Vec<(i32, i32)> {
        // Pictures, ceiling lamps and rugs take no floor space (a plant hanging low, or a lamp at
        // head height, does).
        if self.content.objects[obj.def].walked_under() {
            return Vec::new();
        }
        if self.content.objects[obj.def].stairs {
            if self.stairs.depth == 0 {
                return Vec::new();
            }
            let d = self.stairs.rise(self.storey_of(obj.z));
            return obj
                .tiles(&self.content)
                .map(|(x, z)| (x, z + d))
                .filter(|&(x, z)| self.lot.in_bounds(x, z))
                .collect();
        }
        // (And where it reaches over at head height: an arc lamp's shade.)
        obj.tiles(&self.content)
            .chain(
                obj.overhang_tiles(&self.content)
                    .filter(|&(x, z)| self.lot.in_bounds(x, z)),
            )
            .collect()
    }

    /// Storey rules for an object about to be placed: nothing on stairs; off the ground only on
    /// floor (inside a room); stairs need a storey above with their stairwell free, and stairs up
    /// from the basement come up inside a room.
    pub(crate) fn check_storey_fit(&self, obj: &ObjectInstance) -> Result<(), Error> {
        let def = &self.content.objects[obj.def];
        let tiles: Vec<(i32, i32)> = obj.tiles(&self.content).collect();
        if tiles.iter().any(|&(x, z)| self.stairs.step(x, z).is_some()) {
            return Err(Error::new("nothing can stand on the stairs"));
        }
        let storey = self.storey_of(obj.z);
        if storey != 0
            && tiles
                .iter()
                .any(|&(x, z)| self.lot.room_at(x, z) == OUTDOORS)
        {
            return Err(Error::new(if storey > 0 {
                "upstairs, things go inside rooms"
            } else {
                "in the basement, things go inside rooms"
            }));
        }
        if def.stairs {
            if self.stairs.depth == 0 || storey + 1 >= self.storeys as i32 {
                return Err(Error::new("there's no storey above for stairs to lead to"));
            }
            let d = self.stairs.rise(storey);
            if tiles
                .iter()
                .any(|&(x, z)| self.blocked[self.lot.tile_index(x, z + d)])
            {
                return Err(Error::new("something stands where the stairs come up"));
            }
            if storey < 0
                && tiles
                    .iter()
                    .any(|&(x, z)| self.lot.room_at(x, z + d) == OUTDOORS)
            {
                return Err(Error::new(
                    "stairs down to the basement start inside the house",
                ));
            }
            // Stepping off the top: onto a free tile, with no wall in the way.
            if let Some(s) = stair_of(&self.content, obj, &self.stairs) {
                let (tx, tz) = s.top();
                let (lx, lz) = s.landing;
                if !self.lot.in_bounds(lx, lz)
                    || self.blocked[self.lot.tile_index(lx, lz)]
                    || !self.lot.edge_walkable(tx, tz + d, lx, lz)
                {
                    return Err(Error::new(
                        "the top of the stairs needs a clear way off them",
                    ));
                }
            }
        }
        Ok(())
    }

    /// How high a resident stands, in storeys (fractions on the stairs, for drawing).
    pub fn sim_height(&self, sim: &Sim) -> f32 {
        let storey = self.storey_of(sim.pos[1].floor() as i32) as f32;
        let (x, z) = sim.tile();
        match self.stairs.step(x, z) {
            None => storey,
            Some((s, i)) => {
                // Along the step (a spiral turns, so each step has its own way), from the edge it
                // is stepped onto to the edge it is left at; the whole stair rises one storey.
                let d = s.dir(i);
                let (bx, bz) = s.tile(i);
                let along = (sim.pos[0] - (bx as f32 + 0.5 - d.0 as f32 * 0.5)) * d.0 as f32
                    + (sim.pos[1] - (bz as f32 + 0.5 - d.1 as f32 * 0.5)) * d.1 as f32;
                storey + ((i as f32 + along) / s.len as f32).clamp(0.0, 1.0)
            }
        }
    }
}

/// The stair an object is, if it's stairs (with storeys): a flight climbed from its front tile,
/// along its centre column, up to the tile past its far end on the storey above; or a spiral
/// winding round its square footprint, walked onto from any side.
pub(crate) fn stair_of(content: &Content, o: &ObjectInstance, map: &StairMap) -> Option<Stair> {
    if !content.objects[o.def].stairs || map.depth == 0 {
        return None;
    }
    let depth = map.rise(map.storey(o.z));
    let (w, d) = o.size(content);
    let foot = o.front_tile(content);
    let (tiles, open) = match content.objects[o.def].stair {
        StairShape::Spiral => (spiral_steps(o, w, d), true),
        StairShape::Flight => {
            // Climbing away from the front.
            let (dir, len, bottom) = match o.rot % 4 {
                0 => ((0, -1), d, (o.x + (w - 1) / 2, o.z + d - 1)),
                1 => ((-1, 0), w, (o.x + w - 1, o.z + (d - 1) / 2)),
                2 => ((0, 1), d, (o.x + (w - 1) / 2, o.z)),
                _ => ((1, 0), w, (o.x, o.z + (d - 1) / 2)),
            };
            let tiles: Vec<(i32, i32)> = (0..len)
                .map(|i| (bottom.0 + dir.0 * i, bottom.1 + dir.1 * i))
                .collect();
            (tiles, false)
        }
    };
    // Stepped off the top, one storey up: past the last step, the way it was climbing.
    let out = Stair::out_of(&tiles);
    Some(Stair::new(&tiles, foot, open, (out.0, out.1 + depth)))
}

/// The steps of a spiral, bottom first: round its square footprint (turned by `rot`), starting at
/// the front corner on the left and winding to the back left, so it is stepped off past the back.
fn spiral_steps(o: &ObjectInstance, w: i32, d: i32) -> Vec<(i32, i32)> {
    // Two tiles square (content checks it), so its far side is one tile across.
    let far = w.min(d) - 1;
    // The four corners, from the front left (as the front is +z), anticlockwise.
    let base = [(0, far), (far, far), (far, 0), (0, 0)];
    // Turned to the object's facing (a quarter turn puts (a, b) at (b, far - a)) and moved onto it.
    let turn = |(a, b): (i32, i32)| {
        let (a, b) = match o.rot.rem_euclid(4) {
            0 => (a, b),
            1 => (b, far - a),
            2 => (far - a, far - b),
            _ => (far - b, a),
        };
        (o.x + a, o.z + b)
    };
    base.into_iter().map(turn).collect()
}
