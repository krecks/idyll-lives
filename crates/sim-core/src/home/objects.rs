//! Buying, selling and changing objects at home.
//!
//! Split from `home.rs` (see `mod.rs`).

use super::*;

impl World {
    /// Buys an object for the household's home. Returns the new object's id.
    pub fn buy(
        &mut self,
        household: u32,
        def_id: &str,
        at: Option<[i32; 3]>,
        colors: crate::world::Colors,
    ) -> Result<u32, Error> {
        self.buy_at(household, def_id, at, colors, Layout::default())
    }

    /// Buys an object for the household's home, laid out as `layout` (unchecked: see
    /// `check_layout`; its offset is where it goes on a top, for things on a surface). Without
    /// `at`, it goes at the first good free spot (the offset kept unless it stands on a
    /// surface). Returns the new object's id.
    pub fn buy_at(
        &mut self,
        household: u32,
        def_id: &str,
        at: Option<[i32; 3]>,
        colors: crate::world::Colors,
        layout: Layout,
    ) -> Result<u32, Error> {
        self.buy_charging(household, def_id, at, colors, layout, true)
    }

    /// `buy_at`, paid for now (`charge`) or already (a parcel set up: `setup.rs`).
    pub(crate) fn buy_charging(
        &mut self,
        household: u32,
        def_id: &str,
        at: Option<[i32; 3]>,
        colors: crate::world::Colors,
        layout: Layout,
        charge: bool,
    ) -> Result<u32, Error> {
        let (h, plot) = self.home_of(household)?;
        let def = self
            .content
            .object_index(def_id)
            .ok_or_else(|| Error::new(format!("unknown object '{def_id}'")))?;
        let d = &self.content.objects[def];
        let price = d
            .price
            .ok_or_else(|| Error::new(format!("{} isn't for sale", d.name)))?;
        if charge {
            self.can_pay(h, price, || format!("not enough money for the {}", d.name))?;
        }
        let on_top = d.layer == Layer::Surface;
        let id = match at {
            Some([x, z, rot]) => {
                self.keep_doorway_clear(def, x, z, rot as u8, layout)?;
                self.place_at_home(plot, def, x, z, rot as u8, layout, colors, 0, price)?
            }
            None => self
                .find_spot(plot, def)
                .into_iter()
                .find_map(|(x, z, rot, found)| {
                    let layout = if on_top { Layout::at(found) } else { layout };
                    self.place_at_home(plot, def, x, z, rot, layout, colors.clone(), 0, price)
                        .ok()
                })
                .ok_or_else(|| Error::new("there's no free spot for that at home"))?,
        };
        // A crop bought new is just planted: it ripens over its days (a lawn to mow is there
        // to mow at once: `Growth::bought_ripe`).
        if let Some(g) = self.content.objects[def].grows {
            self.objects[id as usize].growth = if g.bought_ripe { 1.0 } else { 0.0 };
        }
        // A lamp (or a fire) comes switched off: residents switch it on as they come in when
        // it's dark, or the player does (see `lights.rs`).
        if crate::lights::switchable(self, def) {
            self.objects[id as usize].off = true;
            self.lights_version = self.lights_version.wrapping_add(1);
        }
        if charge {
            self.pay(h, price);
        }
        Ok(id)
    }

    /// Puts `def` into the household's home at the first good free spot, free of charge (a
    /// delivery: a crib for a new baby). None if there's no room.
    pub(crate) fn deliver(&mut self, household: u32, def: usize) -> Option<u32> {
        let (_, plot) = self.home_of(household).ok()?;
        let value = self.content.objects[def].price.unwrap_or(0);
        self.find_spot(plot, def)
            .into_iter()
            .find_map(|(x, z, rot, offset)| {
                self.place_at_home(
                    plot,
                    def,
                    x,
                    z,
                    rot,
                    Layout::at(offset),
                    Default::default(),
                    0,
                    value,
                )
                .ok()
            })
    }

    /// Sells an object at home for part of what was spent on it (nothing in Creative). What
    /// stands on its top is sold with it.
    pub(crate) fn sell(&mut self, household: u32, object: u32) -> Result<(), Error> {
        let (h, _) = self.home_object(household, object)?;
        let obj = &self.objects[object as usize];
        if self.content.objects[obj.def].price.is_none() {
            return Err(Error::new("that can't be sold"));
        }
        let mut sold = crate::surface::riders(&self.content, &self.objects, object);
        sold.push(object);
        sold.sort_unstable();
        // Food and dishes on it (or kept in it) go too, not what's on the table under it.
        let held: Vec<u32> = self
            .items_held_by(&sold)
            .into_iter()
            .map(|(item, _)| item)
            .collect();
        self.drop_items(&held);
        let mut refund = 0;
        // Highest id first: the ids before it stay put.
        for &id in sold.iter().rev() {
            let obj = &self.objects[id as usize];
            // Something broken brings back half.
            let broken = if obj.broken() { 0.5 } else { 1.0 };
            refund += (obj.value as f32 * self.content.object_rules.resale * broken).round() as i64;
            self.remove_object(id);
        }
        self.households[h].funds += self.build_price(refund);
        Ok(())
    }

    /// Moves or rotates an object at home. Its look, quality and value stay. Returns its new id
    /// (it's placed again at the end of the list, so the ids after it shift down).
    pub fn move_object(
        &mut self,
        household: u32,
        object: u32,
        x: i32,
        z: i32,
        rot: u8,
    ) -> Result<u32, Error> {
        self.move_object_to(household, object, x, z, rot, None, None, None, None)
    }

    /// Moves, rotates or resizes an object at home, with a new free turn, offset, size
    /// (centimetres across and deep, before turning) and height (centimetres taller) if given, else it keeps
    /// its own (all unchecked: see `check_turn`, `check_layout`). Things standing on its top go
    /// with it, turned with it. Its look, quality and value stay. Returns its new id: it's placed again at the end of the list (its riders
    /// after it), so the ids after it shift down. If anything doesn't fit, nothing moves.
    #[allow(clippy::too_many_arguments)]
    pub fn move_object_to(
        &mut self,
        household: u32,
        object: u32,
        x: i32,
        z: i32,
        rot: u8,
        turn: Option<u8>,
        offset: Option<[i8; 2]>,
        size: Option<[u16; 2]>,
        tall: Option<i16>,
    ) -> Result<u32, Error> {
        let (_, plot) = self.home_object(household, object)?;
        let before = self
            .home_snapshot(household)
            .expect("the household has a home");
        let riders = crate::surface::riders(&self.content, &self.objects, object);
        let old_top = crate::surface::Top::of(&self.content, &self.objects[object as usize]);
        // Lift it and its riders out, highest id first (the ids before each stay put).
        let mut ids = riders.clone();
        ids.push(object);
        ids.sort_unstable();
        // Food and dishes on or in them (not what's on the table under it) come along.
        let held = self.items_held_by(&ids);
        let mut lifted: Vec<ObjectInstance> =
            ids.iter().rev().map(|&id| self.remove_object(id)).collect();
        lifted.reverse();
        let old = lifted
            .iter()
            .find(|o| o.id == object)
            .expect("lifted it")
            .clone();
        let result = (|| {
            let layout = Layout {
                offset: offset.unwrap_or(old.offset),
                extent: Some(size.unwrap_or(old.extent)),
                tall: tall.unwrap_or(old.tall),
            };
            self.keep_doorway_clear(old.def, x, z, rot, layout)?;
            let id = self.replace_at_home(plot, &old, x, z, rot, layout)?;
            self.objects[id as usize].turn = turn.unwrap_or(old.turn);
            let new_top = crate::surface::Top::of(&self.content, &self.objects[id as usize]);
            let turned = |o: &ObjectInstance| (o.rot as i32 * 90 + o.turn as i32).rem_euclid(360);
            let delta = turned(&self.objects[id as usize]) - turned(&old);
            let mut placed = vec![(old.clone(), id)];
            for r in lifted.iter().filter(|o| o.id != object) {
                let (x, z, offset) = match (old_top, new_top) {
                    (Some(a), Some(b)) => {
                        // Where it stood on the old top, on the new one.
                        let p = b.world(a.local(crate::surface::middle(&self.content, r)));
                        let spot = crate::surface::SurfaceSpot::at(id, p, b.surface.height);
                        (spot.x, spot.z, spot.offset)
                    }
                    _ => (r.x, r.z, r.offset),
                };
                let facing = (turned(r) + delta).rem_euclid(360);
                let name = self.content.objects[r.def].name.to_lowercase();
                let rid = self
                    .replace_at_home(
                        plot,
                        r,
                        x,
                        z,
                        (facing / 90) as u8,
                        Layout {
                            offset,
                            ..Layout::of(r)
                        },
                    )
                    .map_err(|_| Error::new(format!("the {name} on it wouldn't fit there")))?;
                self.objects[rid as usize].turn = if self.content.objects[r.def].turns {
                    (facing % 90) as u8
                } else {
                    0
                };
                placed.push((r.clone(), rid));
            }
            for (was, now) in placed {
                let items: Vec<u32> = held
                    .iter()
                    .filter(|&&(_, holder)| holder == was.id)
                    .map(|&(item, _)| item)
                    .collect();
                if !items.is_empty() {
                    let new = self.objects[now as usize].clone();
                    self.carry_items(&was, &new, &items);
                }
            }
            Ok(id)
        })();
        if result.is_err() {
            self.restore(before);
        }
        result
    }

    pub(crate) fn recolor(
        &mut self,
        household: u32,
        object: u32,
        colors: crate::world::Colors,
    ) -> Result<(), Error> {
        self.home_object(household, object)?;
        self.objects[object as usize].colors = colors;
        self.structure_version = self.structure_version.wrapping_add(1);
        Ok(())
    }

    /// A quick fix in Buy mode: as good as new, for a share of the price times how worn it is
    /// (free in Creative).
    pub(crate) fn repair(&mut self, household: u32, object: u32) -> Result<(), Error> {
        let (h, _) = self.home_object(household, object)?;
        let obj = &self.objects[object as usize];
        let def = &self.content.objects[obj.def];
        if obj.wear <= 0.0 {
            return Err(Error::new(format!("the {} is as good as new", def.name)));
        }
        let cost = repair_cost(&self.content, obj);
        let name = def.name.clone();
        self.can_pay(h, cost, || format!("not enough money to repair the {name}"))?;
        self.pay(h, cost);
        self.objects[object as usize].wear = 0.0;
        self.structure_version = self.structure_version.wrapping_add(1);
        let o = &self.objects[object as usize];
        self.rooms_things_changed((o.x, o.z));
        Ok(())
    }

    /// Buys the next quality level for an object at home (Buy mode, instant). Returns the new quality.
    pub(crate) fn upgrade(&mut self, household: u32, object: u32) -> Result<u8, Error> {
        let (h, _) = self.home_object(household, object)?;
        let rules = &self.content.object_rules;
        let obj = &self.objects[object as usize];
        let def = &self.content.objects[obj.def];
        let price = def
            .price
            .ok_or_else(|| Error::new(format!("the {} can't be upgraded", def.name)))?;
        if obj.quality >= rules.max_quality {
            return Err(Error::new(format!(
                "the {} is as good as it gets",
                def.name
            )));
        }
        let cost = rules.upgrade_price(price);
        let name = def.name.clone();
        self.can_pay(h, cost, || {
            format!("not enough money to upgrade the {name}")
        })?;
        self.pay(h, cost);
        let obj = &mut self.objects[object as usize];
        obj.quality += 1;
        obj.value += cost;
        let quality = obj.quality;
        self.structure_version = self.structure_version.wrapping_add(1);
        let o = &self.objects[object as usize];
        self.rooms_things_changed((o.x, o.z));
        // Told about the household's first resident (nobody may live there yet).
        if let Some(member) = self
            .sims
            .iter()
            .position(|s| s.here() && s.household as usize == h)
        {
            self.events.push_detail(
                self.tick,
                EventKind::Upgraded,
                member,
                Some(quality as i64),
                None,
            );
        }
        Ok(quality)
    }
}

impl World {
    /// Removes an object. Sims using it stop, and later object ids shift down by one (in what
    /// they do, have queued, do next out of habit, and couldn't get to).
    pub(crate) fn remove_object(&mut self, id: u32) -> ObjectInstance {
        let World {
            sims,
            objects,
            content,
            ..
        } = self;
        for sim in sims.iter_mut() {
            if sim
                .current
                .as_ref()
                .is_some_and(|a| uses_object(&a.task.kind, id))
            {
                end_activity(sim, content, objects);
            }
            sim.queue.retain(|t| !uses_object(&t.kind, id));
            // (A routine's next step, washing hands at this sink, goes with it.)
            sim.follow_up = sim.follow_up.take().filter(|t| !uses_object(&t.kind, id));
            // Talking from the seat that goes: on their feet, and nothing to sit back down to.
            if sim
                .paused
                .as_ref()
                .is_some_and(|a| uses_object(&a.task.kind, id))
            {
                sim.paused = None;
                if sim.use_seat.is_none() {
                    sim.seated_from = None;
                    sim.pose = crate::content::Pose::Stand;
                }
            }
            let current = sim.current.as_mut().map(|a| &mut a.task);
            let paused = sim.paused.as_mut().map(|a| &mut a.task);
            for t in current
                .into_iter()
                .chain(paused)
                .chain(sim.queue.iter_mut())
                .chain(sim.follow_up.iter_mut())
            {
                shift_objects(&mut t.kind, id);
            }
            sim.remap_unreachable(|o| {
                if o == id {
                    None
                } else {
                    Some(if o > id { o - 1 } else { o })
                }
            });
            // A bike that goes (sold, moved) while they're off to it or on it: they walk.
            for b in [&mut sim.riding, &mut sim.bound_for_bike] {
                *b = match *b {
                    Some(o) if o == id => None,
                    Some(o) if o > id => Some(o - 1),
                    other => other,
                };
            }
            // Their way to work timed by it: worked out again; by another: ids shift down too.
            if let Some(c) = sim.commute.as_mut() {
                match c.bike {
                    Some(o) if o == id => sim.commute = None,
                    Some(o) if o > id => c.bike = Some(o - 1),
                    _ => {}
                }
            }
            // A seat taken to use something else (content `seatNear`): ids shift down too, and
            // if the seat itself goes they stand up but may carry on with what they were using.
            match sim.use_seat {
                Some(seat) if seat == id => {
                    sim.use_seat = None;
                    if sim.pose == crate::content::Pose::Sit {
                        sim.seated_from = None;
                        sim.pose = crate::content::Pose::Stand;
                    }
                }
                Some(seat) if seat > id => sim.use_seat = Some(seat - 1),
                _ => {}
            }
        }
        let obj = self.objects.remove(id as usize);
        self.object_plot.remove(id as usize);
        for (i, o) in self.objects.iter_mut().enumerate() {
            o.id = i as u32;
        }
        // Its tiles are free again, but for what else takes them: the table under a lamp at head
        // height, the lamp over a sofa.
        let freed = self.blocking_tiles(&obj);
        let rect = self.object_rect(&obj);
        for &(x, z) in &freed {
            let i = self.lot.tile_index(x, z);
            if i != crate::lot::VOID {
                // (A porch's post may stand there too: a town's porch had a thing at one.)
                self.blocked[i] =
                    self.travel.ground_blocked(i) || self.car_tiles.binary_search(&i).is_ok();
            }
        }
        for o in 0..self.objects.len() {
            for (x, z) in self.blocking_tiles(&self.objects[o]) {
                if freed.contains(&(x, z)) {
                    let i = self.tile_slot_mut(x, z);
                    self.blocked[i] = true;
                }
            }
        }
        if self.content.objects[obj.def].stairs {
            self.refresh_stairs();
            self.nav_changed();
        } else {
            self.objects_changed_in(rect);
        }
        self.rooms_things_changed((obj.x, obj.z));
        obj
    }
}
