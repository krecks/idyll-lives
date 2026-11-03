//! Build mode: a garage on the home lot (`Command::BuildGarage`, `Command::SellGarage`).
//!
//! The player builds one garage on their home lot, outdoors on the ground, its door facing any
//! of the four ways (`cars::Garage::rot`), for content `build.garage`; taking it down brings back
//! `objectRules.resale` of that (nothing in Creative), and both can be undone like any other edit.
//! It stands clear of walls, fences, things, residents, the plot's edge and where the household's
//! car parks, and may cut nobody off from anything they could reach before (`keeps_reach`). Like
//! the town's garages it takes up its tiles (`cars::reblock`); only those tiles' walking changes
//! (`cars::garages_changed`). A car whose driveway it stands at the end of, its door towards the
//! road, parks in front of it (`cars::park_by_garage`).

use crate::Error;
use crate::cars::{GARAGE_WALL_HALF, Garage};
use crate::command::EdgeAxis;
use crate::lot::{Edge, OUTDOORS};
use crate::world::World;

/// A garage's width along its door's wall and its depth (m, between its walls' centre lines):
/// a single garage (3 m inside) up to a roomy one.
pub const GARAGE_WIDTH: [f32; 2] = [3.0, 4.4];
pub const GARAGE_DEPTH: [f32; 2] = [5.0, 6.6];

/// The tiles a box of wall centre lines touches with its walls' thickness (beyond a sliver).
fn touched(b: [f32; 4]) -> impl Iterator<Item = (i32, i32)> {
    let t = GARAGE_WALL_HALF;
    let lo = |v: f32| (v - t + 0.02).floor() as i32;
    let hi = |v: f32| (v + t - 0.02).ceil() as i32;
    let (x0, z0, x1, z1) = (lo(b[0]), lo(b[1]), hi(b[2]), hi(b[3]));
    (z0..z1).flat_map(move |z| (x0..x1).map(move |x| (x, z)))
}

/// Whether a wall, a fence or a diagonal stands in a box of wall centre lines (its walls'
/// thickness, and the wall's, counted): one along its outline a little way off is no matter (a
/// garage beside the house, as the town builds them).
fn walled(lot: &crate::lot::Lot, b: [f32; 4]) -> bool {
    let t = GARAGE_WALL_HALF;
    let [a0, b0, a1, b1] = [b[0] - t, b[1] - t, b[2] + t, b[3] + t];
    const EPS: f32 = 0.01;
    let (x0, z0) = (a0.floor() as i32, b0.floor() as i32);
    let (x1, z1) = (a1.ceil() as i32, b1.ceil() as i32);
    let across = |lo: f32, hi: f32, a: f32, b: f32| hi > a + EPS && lo < b - EPS;
    // Walls along x at line z (between tiles (x, z - 1) and (x, z)), and along z at line x.
    for z in z0..=z1 {
        for x in x0..x1 {
            if lot.in_bounds(x, z)
                && lot.h_edge(x as usize, z as usize) != Edge::Open
                && across(z as f32 - t, z as f32 + t, b0, b1)
                && across(x as f32, x as f32 + 1.0, a0, a1)
            {
                return true;
            }
        }
    }
    for z in z0..z1 {
        for x in x0..=x1 {
            if lot.in_bounds(x, z)
                && lot.v_edge(x as usize, z as usize) != Edge::Open
                && across(x as f32 - t, x as f32 + t, a0, a1)
                && across(z as f32, z as f32 + 1.0, b0, b1)
            {
                return true;
            }
        }
    }
    touched(b).any(|(x, z)| lot.in_bounds(x, z) && lot.diag(x, z).is_some())
}

impl World {
    /// Whether a wall, a fence, a door or a window on edge (`axis`, `x`, `z`) would stand in a
    /// garage (as `walled` sees it the other way round): Build mode keeps walls out of them.
    pub(crate) fn garage_across(&self, axis: EdgeAxis, x: i32, z: i32) -> bool {
        let t = GARAGE_WALL_HALF;
        const EPS: f32 = 0.01;
        let across = |lo: f32, hi: f32, a: f32, b: f32| hi > a + EPS && lo < b - EPS;
        let (x, z) = (x as f32, z as f32);
        self.garages.iter().any(|g| {
            let [a0, b0, a1, b1] = [g.x0 - t, g.z0 - t, g.x1 + t, g.z1 + t];
            match axis {
                EdgeAxis::H => across(z - t, z + t, b0, b1) && across(x, x + 1.0, a0, a1),
                EdgeAxis::V => across(x - t, x + t, a0, a1) && across(z, z + 1.0, b0, b1),
                EdgeAxis::Dp | EdgeAxis::Dn => {
                    across(x, x + 1.0, a0, a1) && across(z, z + 1.0, b0, b1)
                }
            }
        })
    }

    /// Builds a garage on `household`'s home lot: its walls' centre lines `[x0, z0, x1, z1]`, its
    /// door facing `rot`. Refused (saying why) off the lot or the ground, indoors, over walls,
    /// fences, things or someone standing there, where the household's car parks, where it would
    /// shut anyone or anything off, beside a garage already there, or without the money for it.
    pub fn build_garage(&mut self, household: u32, b: [f32; 4], rot: u8) -> Result<(), Error> {
        let (h, plot) = self.home_of(household)?;
        if rot > 3 {
            return Err(Error::new(format!(
                "a garage faces one of four ways, not {rot}"
            )));
        }
        let [x0, z0, x1, z1] = b;
        if b.iter().any(|v| !v.is_finite()) || x1 <= x0 || z1 <= z0 {
            return Err(Error::new("that isn't a garage's outline"));
        }
        let (across, deep) = if rot.is_multiple_of(2) {
            (x1 - x0, z1 - z0)
        } else {
            (z1 - z0, x1 - x0)
        };
        let within = |v: f32, [a, b]: [f32; 2]| v >= a - 0.01 && v <= b + 0.01;
        if !within(across, GARAGE_WIDTH) || !within(deep, GARAGE_DEPTH) {
            return Err(Error::new(format!(
                "a garage is {} to {} m wide and {} to {} m deep",
                GARAGE_WIDTH[0], GARAGE_WIDTH[1], GARAGE_DEPTH[0], GARAGE_DEPTH[1]
            )));
        }
        if self.garages.iter().any(|g| g.plot == plot) {
            return Err(Error::new("your home has a garage already"));
        }
        // On the plot's ground rows, its walls too.
        let p = &self.plots[plot as usize];
        let t = GARAGE_WALL_HALF;
        if x0 - t < p.x as f32 - 0.01
            || z0 - t < p.z as f32 - 0.01
            || x1 + t > (p.x + p.w) as f32 + 0.01
            || z1 + t > (p.z + p.d) as f32 + 0.01
        {
            return Err(Error::new("a garage has to go on your own lot"));
        }
        let garage = Garage {
            plot,
            x0,
            z0,
            x1,
            z1,
            rot: Some(rot),
        };
        let tiles: Vec<(i32, i32)> = touched(b).collect();
        let lot = &self.lot;
        let on = |x: i32, z: i32| tiles.contains(&(x, z));
        for &(x, z) in &tiles {
            if !lot.in_bounds(x, z) {
                return Err(Error::new("a garage has to go on your own lot"));
            }
            if lot.room_at(x, z) != OUTDOORS {
                return Err(Error::new("a garage goes outdoors, not in a room"));
            }
        }
        if walled(lot, b) {
            return Err(Error::new("there's a wall or a fence in the way"));
        }
        // (Only things within a few tiles across: a big world has thousands.)
        let near = |x: i32| x >= x0 as i32 - 8 && x <= x1 as i32 + 8;
        for (i, o) in self.objects.iter().enumerate() {
            if !near(o.x) {
                continue;
            }
            if self.blocking_tiles(o).iter().any(|&(x, z)| on(x, z))
                || (self.object_plot[i] == Some(plot)
                    && o.tiles(&self.content).any(|(x, z)| on(x, z)))
            {
                return Err(Error::new(format!(
                    "the {} is in the way",
                    self.content.objects[o.def].name.to_lowercase()
                )));
            }
        }
        // Where the household's car parks (on its driveway, and where it parked before a garage).
        let parks = self.cars.iter().filter(|c| c.plot == plot).any(|c| {
            let mut spots = vec![(c.spot, c.yaw)];
            spots.extend(c.drive_spot.map(|[x, z, yaw]| ([x, z], yaw)));
            spots.into_iter().any(|(spot, yaw)| {
                crate::cars::car_tiles_at(c, spot, yaw)
                    .iter()
                    .any(|&(x, z)| on(x, z))
            })
        }) || self
            .drives
            .iter()
            .filter(|d| d.plot == plot)
            .any(|d| crate::cars::drive_tiles(d).iter().any(|&(x, z)| on(x, z)));
        if parks {
            return Err(Error::new("that's where the car parks"));
        }
        let standing = garage.tiles();
        if self
            .sims
            .iter()
            .any(|s| s.away_until.is_none() && standing.contains(&s.tile()))
        {
            return Err(Error::new("someone is standing there"));
        }
        if standing
            .iter()
            .any(|&(x, z)| lot.in_bounds(x, z) && self.blocked[lot.tile_index(x, z)])
        {
            return Err(Error::new("there's no room for a garage there"));
        }
        let price = self.content.build.garage;
        self.can_pay(h, price, || "not enough money for a garage".into())?;
        let before = self.reach_signature(plot);
        self.garages.push(garage);
        crate::cars::garages_changed(self, plot);
        if !self.keeps_reach(plot, &before, None) {
            self.garages.pop();
            crate::cars::garages_changed(self, plot);
            return Err(Error::new("that would block the way"));
        }
        self.pay(h, price);
        self.structure_version = self.structure_version.wrapping_add(1);
        Ok(())
    }

    /// Takes down the garage on `household`'s home lot (one the town built too), for
    /// `objectRules.resale` of a garage's price (nothing in Creative).
    pub fn sell_garage(&mut self, household: u32) -> Result<(), Error> {
        let (h, plot) = self.home_of(household)?;
        let Some(i) = self.garages.iter().position(|g| g.plot == plot) else {
            return Err(Error::new("there's no garage at your home"));
        };
        self.garages.remove(i);
        crate::cars::garages_changed(self, plot);
        let refund =
            (self.content.build.garage as f32 * self.content.object_rules.resale).round() as i64;
        self.households[h].funds += self.build_price(refund);
        self.structure_version = self.structure_version.wrapping_add(1);
        Ok(())
    }
}
