//! Placing objects: what a spot must keep clear, and putting things there.

use super::*;

impl World {
    pub(super) fn home_object(&self, household: u32, object: u32) -> Result<(usize, u32), Error> {
        let (h, plot) = self.home_of(household)?;
        if object as usize >= self.objects.len() {
            return Err(Error::new(format!("unknown object {object}")));
        }
        if self.object_plot[object as usize] != Some(plot) {
            return Err(Error::new("that isn't at your home"));
        }
        Ok((h, plot))
    }

    /// Whether every tile of `obj` and the tile in front of it lie on `plot` (nobody uses things
    /// without interactions, like tables, or things standing on a top: their front can be off it).
    pub(super) fn fits_on_plot(&self, obj: &ObjectInstance, plot: u32) -> bool {
        let def = &self.content.objects[obj.def];
        let (fx, fz) = obj.front_tile(&self.content);
        let on = |x: i32, z: i32| self.on_plot(plot, x, z);
        let used = !def.interactions.is_empty() && def.layer != Layer::Surface;
        (!used || on(fx, fz)) && obj.tiles(&self.content).all(|(x, z)| on(x, z))
    }

    /// Tiles reachable on foot from the plot's entry (a flat's: from behind its own door, on its
    /// storey), as a mask over the whole lot.
    pub(crate) fn reachable(&self, plot: u32) -> Vec<bool> {
        let lot = &self.lot;
        let nav = NavGrid {
            lot,
            blocked: &self.blocked,
            stairs: &self.stairs,
            kept_off: &self.travel.junction_tiles,
            steps: None,
        };
        let p = &self.plots[plot as usize];
        let inside = |x: i32, z: i32| {
            let gz = self.ground_z(z);
            x >= p.x - REACH_MARGIN
                && gz >= p.z - REACH_MARGIN
                && x < p.x + p.w + REACH_MARGIN
                && gz < p.z + p.d + REACH_MARGIN
                && p.on_storey(&self.stairs, z)
        };
        let mut seen = vec![false; lot.tile_slots()];
        let entry = p.home_point();
        let start = (entry[0].floor() as i32, entry[1].floor() as i32);
        if !nav.tile_free(start.0, start.1) {
            return seen;
        }
        let mut open = VecDeque::from([start]);
        seen[lot.tile_index(start.0, start.1)] = true;
        while let Some((x, z)) = open.pop_front() {
            let steps = [(1, 0), (-1, 0), (0, 1), (0, -1)].map(|(dx, dz)| (x + dx, z + dz));
            let up_or_down = self
                .stairs
                .link(x, z)
                .filter(|&(lx, lz)| nav.tile_free(lx, lz));
            for (nx, nz) in steps
                .into_iter()
                .filter(|&(nx, nz)| nav.can_step(x, z, nx, nz))
                .chain(up_or_down)
            {
                if inside(nx, nz) && !std::mem::replace(&mut seen[lot.tile_index(nx, nz)], true) {
                    open.push_back((nx, nz));
                }
            }
        }
        seen
    }

    /// Front tiles and empty tiles of `plot` that can be reached, as `(tile index)` list.
    pub(crate) fn reach_signature(&self, plot: u32) -> (Vec<bool>, Vec<bool>) {
        let seen = self.reachable(plot);
        let fronts = self
            .objects
            .iter()
            .map(|o| {
                let def = &self.content.objects[o.def];
                // Nobody needs to reach a picture, a rug, a table nobody uses or what stands on a
                // top; chairs pulled up to a table, from any side.
                self.object_plot[o.id as usize] != Some(plot)
                    || !def.reached()
                    || self.approach_tiles(o).iter().any(|&(fx, fz)| {
                        self.lot.in_bounds(fx, fz) && seen[self.lot.tile_index(fx, fz)]
                    })
            })
            .collect();
        (fronts, seen)
    }

    /// Whether every object and Sim that could be reached `before` still can. Empty floor may
    /// be closed off (draw a room's walls first, then add its door).
    pub(crate) fn keeps_reach(
        &self,
        plot: u32,
        before: &(Vec<bool>, Vec<bool>),
        skip: Option<u32>,
    ) -> bool {
        let after = self.reach_signature(plot);
        let objects_ok = before
            .0
            .iter()
            .zip(&after.0)
            .enumerate()
            .all(|(i, (b, a))| Some(i as u32) == skip || !*b || *a);
        // No Sim at home gets shut in: nor one in bed or on a seat, where they'll stand up (in
        // bed, their tile is the bed's, which nobody walks to).
        let sims_ok = self.sims.iter().all(|s| {
            let (x, z) = s.seated_from.unwrap_or_else(|| s.tile());
            let on_plot = self.on_plot(plot, x, z);
            let i = self.lot.tile_index(x, z);
            let seen = |s: &[bool]| s.get(i).copied().unwrap_or(false);
            s.away_until.is_some() || !on_plot || !seen(&before.1) || seen(&after.1)
        });
        objects_ok && sims_ok
    }

    /// Whether a resident stands where `obj` would go (or would stand up there from a seat).
    fn sim_in_the_way(&self, obj: &ObjectInstance) -> bool {
        self.sims.iter().any(|s| {
            let seated = s.seated_from;
            // (Under an arc lamp's shade, only someone on their feet: one sitting on the sofa
            // under it is out of its way.)
            let standing = s.pose == crate::content::Pose::Stand;
            s.away_until.is_none()
                && (obj
                    .tiles(&self.content)
                    .any(|t| t == s.tile() || Some(t) == seated)
                    || (standing && obj.overhang_tiles(&self.content).any(|t| t == s.tile())))
        })
    }

    /// Places object `def` on the home plot (laid out as `layout`) if it fits there and blocks
    /// nothing.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn place_at_home(
        &mut self,
        plot: u32,
        def: usize,
        x: i32,
        z: i32,
        rot: u8,
        layout: Layout,
        colors: crate::world::Colors,
        quality: u8,
        value: i64,
    ) -> Result<u32, Error> {
        let obj = self.check_fit(def, x, z, rot, layout).map_err(|e| {
            let e = e.to_string();
            // The storeys' rules say what's wrong in words already (`check_storey_fit`).
            if !e.starts_with('\'') {
                return Error::new(e);
            }
            // Why it doesn't fit beside something else, as it says it.
            for why in [
                "would hang into ",
                "would hang too low over ",
                "is too tall for ",
                "is under ",
                "can't hang over a window",
                "can't stand against a window",
            ] {
                if let Some(i) = e.find(why) {
                    return Error::new(format!("that {}", &e[i..]));
                }
            }
            Error::new(if e.contains("diagonal") {
                "furniture can't stand on a tile with a diagonal wall"
            } else if e.contains("no wall to hang on") {
                "that hangs on a wall: put its back to one"
            } else if e.contains("needs a wall behind it") {
                "that stands against a wall: put its back to one"
            } else if e.contains("needs a ceiling") {
                "that hangs from a ceiling: put it in a room"
            } else if e.contains("nothing to stand on") {
                "that goes on a table, a counter, a desk or a shelf"
            } else if e.contains("over the edge") {
                "that would hang over the edge"
            } else {
                "there's no room for that here"
            })
        })?;
        if !self.fits_on_plot(&obj, plot) {
            return Err(Error::new("that has to go on your own lot"));
        }
        if self.content.objects[def].stairs && self.plots[plot as usize].is_flat() {
            return Err(Error::new("a flat is all on one storey: no stairs in it"));
        }
        if self.content.objects[def].outdoors && !self.all_outdoors(&obj) {
            return Err(Error::new("that has to go outside"));
        }
        let crown = self.content.objects[def].crown;
        if crown > 0.0 && self.crown_meets_wall(&obj, crown) {
            return Err(Error::new(
                "its crown would grow into the house: plant it further from the walls",
            ));
        }
        // Residents walk under pictures and lamps and over rugs (not into a plant hanging low, nor
        // under a lamp at head height).
        let d = &self.content.objects[def];
        let mounted = d.walked_under();
        if !mounted && self.sim_in_the_way(&obj) {
            return Err(Error::new("someone is standing there"));
        }
        let before = self.reach_signature(plot);
        let id = self.place(def, x, z, rot, layout, colors, quality, value, None)?;
        let approach = self.approach_tiles(&self.objects[id as usize]);
        let (fx, fz) = approach
            .first()
            .copied()
            .unwrap_or_else(|| self.objects[id as usize].front_tile(&self.content));
        let seen = self.reachable(plot);
        // Upstairs before there are stairs, nothing in the room can be reached yet anyway.
        let cut_off = || {
            let room = self.lot.room_at(fx, fz);
            let p = &self.plots[plot as usize];
            let k = self.storey_row(self.storey_of(fz));
            room != OUTDOORS
                && !(p.z + k..p.z + p.d + k).any(|z| {
                    (p.x..p.x + p.w)
                        .any(|x| self.lot.room_at(x, z) == room && seen[self.lot.tile_index(x, z)])
                })
        };
        // A picture, a rug, a table nobody uses or a thing on a top can go anywhere (over a bed,
        // behind a sofa, with chairs all round); a chair pulled up to a table needs one free side.
        let unused = !self.content.objects[def].reached();
        let reached = approach
            .iter()
            .any(|&(ax, az)| self.lot.in_bounds(ax, az) && seen[self.lot.tile_index(ax, az)]);
        let front_ok =
            unused || reached || (!approach.is_empty() && self.storey_of(fz) != 0 && cut_off());
        if !front_ok || !self.keeps_reach(plot, &before, Some(id)) {
            self.remove_object(id);
            return Err(Error::new("that would block the way"));
        }
        Ok(id)
    }

    /// Places `old`, lifted out, back on the home plot at `x`, `z` (see `place_at_home`), keeping
    /// what it was (`ObjectInstance::carry_state`).
    pub(crate) fn replace_at_home(
        &mut self,
        plot: u32,
        old: &ObjectInstance,
        x: i32,
        z: i32,
        rot: u8,
        layout: Layout,
    ) -> Result<u32, Error> {
        let id = self.place_at_home(
            plot,
            old.def,
            x,
            z,
            rot,
            layout,
            old.colors.clone(),
            old.quality,
            old.value,
        )?;
        self.objects[id as usize].carry_state(old);
        Ok(id)
    }
}
