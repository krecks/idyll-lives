//! Finding a spot: crowns, doorways and the places an object may stand.
//!
//! Split from `home.rs` (see `mod.rs`).

use super::*;

impl World {
    /// Whether a house wall (with a door or window or not, on any storey above ground) stands
    /// closer than `crown` metres to the middle of `obj` (a tree: its crown would grow into it).
    pub(crate) fn crown_meets_wall(&self, obj: &ObjectInstance, crown: f32) -> bool {
        let (mut sx, mut sz, mut n) = (0.0f32, 0.0f32, 0u32);
        for (x, z) in obj.tiles(&self.content) {
            sx += x as f32 + 0.5;
            sz += z as f32 + 0.5;
            n += 1;
        }
        if n == 0 {
            return false;
        }
        let (cx, cz) = (sx / n as f32, sz / n as f32);
        // On the storey's own rows: the same place on every storey above.
        let lz = cz - self.storey_row(self.storey_of(cz as i32)) as f32;
        let (width, depth) = (self.lot.width as i32, self.storey_depth());
        // How far (cx, lz) is from the unit edge from (x0, z0) along x (`along_x`) or z.
        let dist = |x0: f32, z0: f32, along_x: bool| {
            let (dx, dz) = if along_x {
                ((cx - cx.clamp(x0, x0 + 1.0)), lz - z0)
            } else {
                (cx - x0, lz - lz.clamp(z0, z0 + 1.0))
            };
            (dx * dx + dz * dz).sqrt()
        };
        let (x0, x1) = (
            ((cx - crown).floor() as i32).max(0),
            ((cx + crown).ceil() as i32).min(width),
        );
        let (z0, z1) = (
            ((lz - crown).floor() as i32).max(0),
            ((lz + crown).ceil() as i32).min(depth),
        );
        (0..self.storeys as i32).any(|k| {
            let row = self.storey_row(k);
            (z0..=z1).any(|z| {
                (x0..=x1).any(|x| {
                    let (ux, uz) = (x as usize, (row + z) as usize);
                    (x < width
                        && z <= depth
                        && self.lot.h_edge(ux, uz).is_wall()
                        && dist(x as f32, z as f32, true) < crown)
                        || (x <= width
                            && z < depth
                            && self.lot.v_edge(ux, uz).is_wall()
                            && dist(x as f32, z as f32, false) < crown)
                })
            })
        })
    }

    /// Whether every tile of `obj` is outdoors (a tile split by a diagonal wall counts as indoors
    /// when either half is).
    pub(crate) fn all_outdoors(&self, obj: &ObjectInstance) -> bool {
        obj.tiles(&self.content)
            .all(|(x, z)| self.lot.room_at(x, z) == OUTDOORS)
    }

    /// Whether a tile has a door on one of its sides.
    fn by_door(&self, x: i32, z: i32) -> bool {
        if !self.lot.in_bounds(x, z) {
            return false;
        }
        let (ux, uz, lot) = (x as usize, z as usize, &self.lot);
        lot.h_edge(ux, uz) == Edge::Door
            || lot.h_edge(ux, uz + 1) == Edge::Door
            || lot.v_edge(ux, uz) == Edge::Door
            || lot.v_edge(ux + 1, uz) == Edge::Door
    }

    /// Furniture on the floor (not pictures, ceiling lamps up out of the way, rugs, things on tops or stairs)
    /// doesn't go just inside a door, where it would make the way through a squeeze: a lamp
    /// nobody needs to reach could otherwise stand anywhere that leaves the rest reachable.
    pub(crate) fn keep_doorway_clear(
        &self,
        def: usize,
        x: i32,
        z: i32,
        rot: u8,
        layout: Layout,
    ) -> Result<(), Error> {
        let d = &self.content.objects[def];
        if d.walked_under() || d.stairs {
            return Ok(());
        }
        // Where it doesn't fit at all, `place_at_home` says why.
        let Ok(obj) = self.check_fit(def, x, z, rot, layout) else {
            return Ok(());
        };
        if obj
            .tiles(&self.content)
            .any(|(tx, tz)| self.by_door(tx, tz))
        {
            return Err(Error::new("that would stand in a doorway"));
        }
        Ok(())
    }

    /// Good free spots for `def` on the plot, best first: indoors (outdoors for garden things),
    /// back to a wall, away from doors. Things on a surface go on the first top with room for
    /// them, in the middle if it's free.
    pub(crate) fn find_spot(&self, plot: u32, def: usize) -> Vec<(i32, i32, u8, [i8; 2])> {
        if self.content.objects[def].layer == Layer::Surface {
            let radius = self.content.objects[def].radius;
            return self
                .objects
                .iter()
                .filter(|o| self.object_plot[o.id as usize] == Some(plot))
                .filter_map(|o| {
                    let c = crate::surface::middle(&self.content, o);
                    crate::surface::free_surface_spot_near(
                        &self.content,
                        &self.objects,
                        &[o.id],
                        c,
                        f32::INFINITY,
                        0,
                        radius,
                        &[],
                    )
                })
                .map(|s| (s.x, s.z, 0, s.offset))
                .collect();
        }
        let p = &self.plots[plot as usize];
        let lot = &self.lot;
        // (What could be in its way anywhere on the plot, found once: as far past the plot's
        // edges as it reaches, turned any way, from a tile on it.)
        let d = &self.content.objects[def];
        let reach = i32::from(d.footprint[0].max(d.footprint[1]))
            + d.overhang
                .iter()
                .map(|&(r, f)| r.abs().max(f.abs()))
                .max()
                .unwrap_or(0)
            + 1;
        let near = self.objects_reaching([
            p.x - reach,
            p.z - reach,
            p.x + p.w + reach,
            p.z + p.d + reach,
        ]);
        let mut spots = Vec::new();
        for z in p.z..p.z + p.d {
            for x in p.x..p.x + p.w {
                for rot in 0..4u8 {
                    let Ok(obj) =
                        self.check_fit_among(def, x, z, rot, Layout::default(), Some(&near))
                    else {
                        continue;
                    };
                    if !self.fits_on_plot(&obj, plot)
                        || obj
                            .tiles(&self.content)
                            .any(|(tx, tz)| self.by_door(tx, tz))
                    {
                        continue;
                    }
                    let (w, d) = obj.size(&self.content);
                    let wall = |e: Edge| e.blocks();
                    let back_to_wall = match rot {
                        0 => (x..x + w).all(|tx| wall(lot.h_edge(tx as usize, z as usize))),
                        2 => (x..x + w).all(|tx| wall(lot.h_edge(tx as usize, (z + d) as usize))),
                        1 => (z..z + d).all(|tz| wall(lot.v_edge(x as usize, tz as usize))),
                        _ => (z..z + d).all(|tz| wall(lot.v_edge((x + w) as usize, tz as usize))),
                    };
                    let indoors = lot.room_at(x, z) != OUTDOORS;
                    let wanted = indoors != self.content.objects[def].outdoors;
                    let score = wanted as i32 * 10 + back_to_wall as i32 * 5;
                    spots.push((score, x, z, rot));
                }
            }
        }
        spots.sort_by_key(|s| -s.0);
        spots
            .into_iter()
            .map(|(_, x, z, r)| (x, z, r, [0, 0]))
            .collect()
    }
}
