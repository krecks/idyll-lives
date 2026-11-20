//! Rooms: finding them, scoring them, and keeping their state as the lot changes.

use super::*;

impl World {
    /// Every room and garden of the town with its scores (current as of the last refresh).
    /// Everything found with the rooms, for tests that compare two ways of finding them: the
    /// rooms, the score per tile, the private and claimed rooms and the places in those.
    #[doc(hidden)]
    pub fn room_state(&self) -> (Vec<crate::rooms::RoomInfo>, Vec<f32>, Vec<u16>, Vec<u16>) {
        (
            self.rooms.clone(),
            self.room_score.clone(),
            self.private_rooms.clone(),
            self.claimed_rooms.clone(),
        )
    }

    pub fn rooms(&self) -> &[crate::rooms::RoomInfo] {
        &self.rooms
    }

    /// The room (or garden) a tile belongs to.
    pub fn room_at_tile(&self, x: i32, z: i32) -> Option<&crate::rooms::RoomInfo> {
        if !self.lot.in_bounds(x, z) {
            return None;
        }
        let id = self.lot.room_at(x, z);
        if id == crate::lot::OUTDOORS {
            let plot = self.plot_at(x, z)?;
            self.rooms.iter().find(|r| r.garden && r.plot == Some(plot))
        } else {
            self.rooms.iter().find(|r| !r.garden && r.id == id)
        }
    }

    /// Mess residents made, and cleaning they finished, this tick: a mess spreads over the
    /// tile and (less) the tiles around it in the same room; cleaning clears them all.
    pub(super) fn apply_messes(&mut self) {
        for i in 0..self.sims.len() {
            if let Some(partner) = self.sims[i].conceived.take() {
                // (Both done trying at once: one chance, not two.)
                if let Some(p) = self
                    .sims
                    .get_mut(partner as usize)
                    .filter(|p| p.conceived == Some(i as u32))
                {
                    p.conceived = None;
                }
                crate::lifecycle::conceive(self, i, partner as usize);
            }
            if let Some((baby, gains)) = self.sims[i].care_out.take()
                && let Some(b) = self.sims.get_mut(baby as usize)
            {
                for (n, g) in gains.iter().enumerate() {
                    b.needs[n] = (b.needs[n] + g).min(1.0);
                }
            }
            if std::mem::take(&mut self.sims[i].reshaped) {
                // An object opened or closed, or someone changed clothes: how it looks changes.
                self.structure_version = self.structure_version.wrapping_add(1);
                self.rooms_need_clean_rescore();
            }
            if std::mem::take(&mut self.sims[i].lit) {
                self.lights_version = self.lights_version.wrapping_add(1);
            }
            if let Some(news) = self.sims[i].news.take() {
                // Broken or mended: what the house offers and how it looks change.
                self.structure_version = self.structure_version.wrapping_add(1);
                let at = match news {
                    News::Broke(o) | News::Repaired(o) | News::NotClosed(o) => {
                        let o = &self.objects[o as usize];
                        (o.x, o.z)
                    }
                    News::Accident(_) => self.sims[i].tile(),
                };
                self.rooms_things_changed(at);
                let (kind, n) = match news {
                    News::Broke(o) => (
                        social::EventKind::Broke,
                        self.objects[o as usize].def as i64,
                    ),
                    News::Repaired(o) => (
                        social::EventKind::Repaired,
                        self.objects[o as usize].def as i64,
                    ),
                    News::Accident(a) => (social::EventKind::Accident, a as i64),
                    News::NotClosed(o) => (
                        social::EventKind::NotClosed,
                        self.objects[o as usize].def as i64,
                    ),
                };
                self.events.push_detail(self.tick, kind, i, Some(n), None);
            }
            if let Some((x, z, sweep)) = self.sims[i].swept.take() {
                crate::sweep::clean(self, x, z, &sweep);
            }
            let Some((x, z, amount)) = self.sims[i].mess.take() else {
                continue;
            };
            if !self.lot.in_bounds(x, z) {
                continue;
            }
            let room = self.lot.room_at(x, z);
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let (tx, tz) = (x + dx, z + dz);
                    if !self.lot.in_bounds(tx, tz) || self.lot.room_at(tx, tz) != room {
                        continue;
                    }
                    let share = if amount < 0.0 || (dx, dz) == (0, 0) {
                        1.0
                    } else if dx == 0 || dz == 0 {
                        0.5
                    } else {
                        0.25
                    };
                    let t = self.tile_slot_mut(tx, tz);
                    self.dirt[t] = (self.dirt[t] + amount * share).clamp(0.0, 1.0);
                    self.room_tiles.touch(t);
                }
            }
        }
    }

    /// Notes that the lot's rooms changed (walls, storeys, a blueprint, an undo): the next tick
    /// finds and scores the rooms again.
    pub(crate) fn rooms_layout_changed(&mut self) {
        self.room_layout_version = self.room_layout_version.wrapping_add(1);
    }

    /// Notes that a room's scores changed without its layout (a wall painted, a floor covered):
    /// the next tick recomputes the rooms once.
    pub(crate) fn rooms_need_rescore(&mut self) {
        self.rooms_dirty = true;
    }

    /// Notes that a thing standing on tile `at` came, went, broke, was mended or upgraded, and
    /// nothing else about the rooms changed: the next tick scores the room (or garden) there
    /// again for what stands in it (`refresh_room_things`) without finding it again.
    pub(crate) fn rooms_things_changed(&mut self, at: (i32, i32)) {
        self.rooms_things = true;
        if self.rooms_things_at.len() < MOST_THINGS_AT {
            self.rooms_things_at.push(at);
        } else {
            self.rooms_things_all = true;
        }
    }

    /// Notes that things may have changed anywhere (an undo put them back): every room is scored
    /// again for what stands in it.
    pub(crate) fn rooms_things_changed_everywhere(&mut self) {
        self.rooms_things = true;
        self.rooms_things_all = true;
    }

    /// Notes that only cleanliness may have changed (a crop grew, a thing opened): the next
    /// tick rescores the rooms' cleanliness without walking the whole lot.
    pub(crate) fn rooms_need_clean_rescore(&mut self) {
        self.rooms_rescore = true;
    }

    /// Recomputes rooms and their scores (see `rooms.rs`). Call it after changing `dirt`
    /// directly; messes residents make are rescored on their own.
    pub fn refresh_rooms(&mut self) {
        self.sync_tile_arrays();
        self.map_plots();
        // (Everything is found again: what the lot says changed since doesn't matter.)
        self.lot.take_room_changes();
        let (rooms, cells) = crate::rooms::compute(self);
        self.rooms = rooms;
        self.map_halls();
        self.rooms_out = crate::privacy::doors_out(&self.content, &self.rooms, &self.lot);
        self.refresh_privacy();
        self.rooms_version = self.room_layout_version;
        self.rooms_dirty = false;
        self.rooms_things = false;
        self.rooms_things_all = false;
        self.rooms_things_at.clear();
        self.rooms_rescore = false;
        // The room (index into `rooms`) whose score each tile takes: its plot's garden, then
        // indoor tiles anywhere their room; off the plots, none (`roomRules.away`).
        let rooms = &self.rooms;
        let mut source = vec![crate::rooms::NO_ROOM; self.lot.tile_slots()];
        // (A flat has no garden: its rectangle's ground is its building's.)
        for (pi, p) in self.lots().iter().enumerate() {
            let garden = rooms
                .iter()
                .position(|r| r.garden && r.plot == Some(pi as u32))
                .map_or(crate::rooms::NO_ROOM, |k| k as u32);
            for z in p.z..p.z + p.d {
                for x in p.x..p.x + p.w {
                    if self.lot.in_bounds(x, z) {
                        source[self.lot.tile_index(x, z)] = garden;
                    }
                }
            }
        }
        source[crate::lot::VOID] = crate::rooms::NO_ROOM;
        let max_id = rooms
            .iter()
            .filter(|r| !r.garden)
            .map(|r| r.id as usize)
            .max()
            .unwrap_or(0);
        let mut by_id = vec![None; max_id + 1];
        for (k, r) in rooms.iter().enumerate().filter(|(_, r)| !r.garden) {
            by_id[r.id as usize] = Some(k);
        }
        for (_, _, i) in self.lot.indoor_tiles() {
            let id = self.lot.rooms()[i];
            if let Some(Some(k)) = by_id
                .get(id as usize)
                .filter(|_| id != crate::lot::OUTDOORS)
            {
                source[i] = *k as u32;
            }
        }
        let away = self.content.room_rules.away;
        self.room_score = source
            .iter()
            .map(|&k| match k {
                crate::rooms::NO_ROOM => away,
                k => rooms[k as usize].scores.overall,
            })
            .collect();
        self.room_tiles = crate::rooms::RoomTiles::new(cells, &source);
    }

    /// Catches up on the rooms after walls, coverings, things or dirt changed: as `refresh_rooms`
    /// would give, but finding and scoring again only the rooms round what was built
    /// (`refresh_rooms_near`) and where things changed (`refresh_room_things`), and the rest only
    /// for their dirt where there was a mess.
    pub(super) fn update_rooms(&mut self) {
        // The per-tile lists follow the lot's slots (chunks allocated since: outdoors).
        let slots = self.lot.tile_slots();
        if self.room_score.len() < slots {
            self.room_score.resize(slots, self.content.room_rules.away);
        }
        self.room_tiles.grow(slots);
        let layout = self.rooms_version != self.room_layout_version || self.rooms_dirty;
        if layout {
            let changes = self.lot.take_room_changes();
            if changes.full {
                self.refresh_rooms();
                return;
            }
            self.refresh_rooms_near(&changes);
            self.rooms_version = self.room_layout_version;
            self.rooms_dirty = false;
        }
        let things = self.rooms_things;
        if things {
            self.refresh_room_things();
        } else {
            self.rescore_rooms();
        }
        self.rooms_rescore = false;
        if layout || things {
            self.refresh_privacy();
        }
    }

    /// The rooms found and scored again inside `changes.rect` (grown a tile, for the rooms across
    /// a wall from it) and the gardens of the plots there; the rest keep their scores, renumbered
    /// as the lot renumbered them (`Lot::take_room_changes`). What `refresh_rooms` would give.
    fn refresh_rooms_near(&mut self, changes: &crate::lot::RoomChanges) {
        use crate::rooms::room_order;
        self.sync_tile_arrays();
        self.map_plots();
        self.map_halls();
        let away = self.content.room_rules.away;
        let Some([x0, z0, x1, z1]) = changes.rect else {
            return;
        };
        let rect = [x0 - 1, z0 - 1, x1 + 1, z1 + 1];
        let meets =
            |r: [i32; 4]| r[0] < rect[2] && rect[0] < r[2] && r[1] < rect[3] && rect[1] < r[3];
        // The lots there, on any storey (what stands upstairs outdoors is in the garden; a flat
        // has none).
        let depth = self.storey_depth().max(1);
        let blocks = (self.lot.depth as i32 / depth).max(1);
        let plots: Vec<u32> = self
            .lots()
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                (0..blocks).any(|b| meets([p.x, p.z + b * depth, p.x + p.w, p.z + p.d + b * depth]))
            })
            .map(|(k, _)| k as u32)
            .collect();
        // The rooms there now, and the ones kept (renumbered).
        let ids: Vec<u16> = (1..=self.lot.room_count() as u16)
            .filter(|&id| self.lot.room_rect(id).is_some_and(meets))
            .collect();
        let rooms = std::mem::take(&mut self.rooms);
        let handles = self.room_tiles.take_handles();
        let old_out = std::mem::take(&mut self.rooms_out);
        let mut kept: Vec<(crate::rooms::RoomInfo, u32)> = Vec::with_capacity(rooms.len());
        let mut out_kept: Vec<(u16, bool)> = Vec::new();
        for (mut room, h) in rooms.into_iter().zip(handles) {
            let keep = if room.garden {
                room.plot.is_some_and(|p| plots.binary_search(&p).is_err())
            } else {
                let id = changes
                    .remap
                    .as_ref()
                    .map_or(room.id, |m| m.get(room.id as usize).copied().unwrap_or(0));
                if id != crate::lot::OUTDOORS {
                    out_kept.push((id, old_out.get(room.id as usize).copied().unwrap_or(false)));
                }
                room.id = id;
                id != crate::lot::OUTDOORS && ids.binary_search(&id).is_err()
            };
            if keep {
                kept.push((room, h));
            } else {
                self.room_tiles.drop_room(h);
            }
        }
        // Found and scored again, with the tiles that take each one's score.
        let found = crate::rooms::compute_near(self, &ids, &plots);
        let lot = &self.lot;
        // The last plot over each tile sets its garden's score there (as `refresh_rooms`).
        let last_plot = |x: i32, z: i32| {
            plots
                .iter()
                .rev()
                .copied()
                .find(|&p| self.plots[p as usize].contains(x, z))
        };
        let gardens: Vec<u32> = found
            .iter()
            .filter(|(r, _)| r.garden)
            .filter_map(|(r, _)| r.plot)
            .collect();
        for (room, cells) in found {
            let mut scored = Vec::new();
            if room.garden {
                let pi = room.plot.expect("a garden is a plot's");
                let p = &self.plots[pi as usize];
                // (A later plot over the same tiles scores them: as `refresh_rooms`.)
                let later: Vec<&Plot> = self.lots()[pi as usize + 1..]
                    .iter()
                    .filter(|q| {
                        q.x < p.x + p.w && p.x < q.x + q.w && q.z < p.z + p.d && p.z < q.z + q.d
                    })
                    .collect();
                for z in p.z..p.z + p.d {
                    for x in p.x..p.x + p.w {
                        if !lot.in_bounds(x, z) {
                            continue;
                        }
                        let i = lot.tile_index(x, z);
                        if i != crate::lot::VOID
                            && lot.rooms()[i] == crate::lot::OUTDOORS
                            && !later.iter().any(|q| q.contains(x, z))
                        {
                            scored.push(i as u32);
                        }
                    }
                }
            } else if let Some([rx0, rz0, rx1, rz1]) = lot.room_rect(room.id) {
                for z in rz0..rz1 {
                    for x in rx0..rx1 {
                        let i = lot.tile_index(x, z);
                        if i != crate::lot::VOID && lot.rooms()[i] == room.id {
                            scored.push(i as u32);
                        }
                    }
                }
            }
            for &i in &scored {
                self.room_score[i as usize] = room.scores.overall;
            }
            let h = self.room_tiles.add_room(cells, scored);
            kept.push((room, h));
        }
        // Tiles there no room or garden scores: away from home.
        for z in rect[1].max(0)..rect[3].min(lot.depth as i32) {
            for x in rect[0].max(0)..rect[2].min(lot.width as i32) {
                let i = lot.tile_index(x, z);
                if i == crate::lot::VOID || lot.rooms()[i] != crate::lot::OUTDOORS {
                    continue;
                }
                let garden = (z < depth)
                    .then(|| last_plot(x, z))
                    .flatten()
                    .filter(|p| gardens.contains(p));
                if garden.is_none() {
                    self.room_score[i] = away;
                }
            }
        }
        kept.sort_by_key(|(r, _)| room_order(r));
        let (rooms, handles): (Vec<_>, Vec<_>) = kept.into_iter().unzip();
        self.rooms = rooms;
        self.room_tiles.set_handles(handles);
        // Doors to the outside, per room id (as `privacy::doors_out`).
        if self.content.room_kinds.iter().any(|k| k.private) {
            let most = self.rooms.iter().map(|r| r.id as usize).max().unwrap_or(0);
            let mut out = vec![false; most + 1];
            for (id, o) in out_kept {
                if let Some(slot) = out.get_mut(id as usize) {
                    *slot = o;
                }
            }
            for &id in &ids {
                if let Some(slot) = out.get_mut(id as usize) {
                    *slot = crate::privacy::door_out(&self.lot, id);
                }
            }
            self.rooms_out = out;
        }
    }

    /// Scores the rooms again for what stands in them, the rooms themselves as they are (things
    /// came, went, broke, were mended or upgraded, and nothing was built: `rooms_things_changed`):
    /// the rooms where things changed, or all of them if things may have changed anywhere, and
    /// the rest for their dirt where there was a mess. The same as `refresh_rooms` would give,
    /// at the cost of the things and the rooms' dirt, not of finding the rooms and walking every
    /// tile.
    fn refresh_room_things(&mut self) {
        let mut rooms = std::mem::take(&mut self.rooms);
        let mut tiles = std::mem::take(&mut self.room_tiles);
        let mut score = std::mem::take(&mut self.room_score);
        let at = std::mem::take(&mut self.rooms_things_at);
        let only = (!self.rooms_things_all).then(|| {
            let mut picked = vec![false; rooms.len()];
            for &(x, z) in &at {
                let id = self.lot.room_at(x, z);
                let k = if id == crate::lot::OUTDOORS {
                    let plot = self.plot_at(x, z);
                    rooms
                        .iter()
                        .position(|r| r.garden && plot.is_some() && r.plot == plot)
                } else {
                    rooms.iter().position(|r| !r.garden && r.id == id)
                };
                if let Some(k) = k {
                    picked[k] = true;
                }
            }
            picked
        });
        crate::rooms::rescore_things(self, &mut rooms, &mut tiles, &mut score, only.as_deref());
        (self.rooms, self.room_tiles, self.room_score) = (rooms, tiles, score);
        self.rooms_things = false;
        self.rooms_things_all = false;
        self.rooms_rescore = false;
    }

    /// The rooms kept shut and the claimed ones among them, and the places in those (`privacy`),
    /// from the rooms as they are.
    fn refresh_privacy(&mut self) {
        let (private, claimed) =
            crate::privacy::private_rooms(&self.content, &self.plots, &self.rooms, &self.rooms_out);
        self.private_rooms = private;
        self.claimed_rooms = claimed;
        self.claimed_places = crate::privacy::claimed_places(
            &self.content,
            &self.lot,
            &self.claimed_rooms,
            &self.objects,
        );
        self.claimants_cache.clear();
    }

    /// Catches up on the rooms as the next tick would (found again, scored again for what stands
    /// in them, or for their dirt), for benchmarks and tools.
    pub fn catch_up_rooms(&mut self) {
        if self.rooms_version != self.room_layout_version
            || self.rooms_dirty
            || self.rooms_things
            || self.rooms_rescore
        {
            self.update_rooms();
        }
    }

    /// For tests: catches up on the rooms as the next tick would, and checks that what that gave
    /// (found again only round what changed, scored again only where things changed) is what
    /// finding and scoring every room again gives: the lot's rooms and their numbers, every room
    /// and garden with its scores, the score per tile, the rooms' tiles, doors to the outside,
    /// private and claimed rooms, and `structure_json`. The rooms are left as they were kept.
    #[doc(hidden)]
    pub fn check_rooms(&mut self) -> Result<(), String> {
        self.catch_up_rooms();
        // (Messes since the tick began are scored at the next refresh, as any refresh does.)
        self.rescore_rooms();
        // (Chunks allocated since, by an edit given up: outdoors, away, as a short list reads.)
        let slots = self.lot.tile_slots();
        if self.room_score.len() < slots {
            self.room_score.resize(slots, self.content.room_rules.away);
        }
        self.room_tiles.grow(slots);
        let mut lot = self.lot.clone();
        lot.compute_all_rooms();
        if lot != self.lot || lot.rooms_row_major() != self.lot.rooms_row_major() {
            return Err("the lot's rooms differ from finding them all again".into());
        }
        if lot.room_count() != self.lot.room_count()
            || (1..=lot.room_count() as u16).any(|id| lot.room_rect(id) != self.lot.room_rect(id))
            || !lot.indoor_tiles().eq(self.lot.indoor_tiles())
        {
            return Err("the lot's rooms are kept differently".into());
        }
        let kept = (
            std::mem::take(&mut self.rooms),
            std::mem::take(&mut self.room_score),
            std::mem::take(&mut self.room_tiles),
            std::mem::take(&mut self.rooms_out),
            std::mem::take(&mut self.private_rooms),
            std::mem::take(&mut self.claimed_rooms),
            std::mem::take(&mut self.claimed_places),
        );
        self.rooms.clone_from(&kept.0);
        self.room_score.clone_from(&kept.1);
        self.rooms_out.clone_from(&kept.3);
        self.private_rooms.clone_from(&kept.4);
        self.claimed_rooms.clone_from(&kept.5);
        self.claimed_places.clone_from(&kept.6);
        let structure = crate::view::structure_json(self);
        let tiles = kept.2.summary();
        self.refresh_rooms();
        let mut wrong = Vec::new();
        if self.rooms != kept.0 {
            let n = self.rooms.len().max(kept.0.len());
            let first = (0..n).find(|&k| self.rooms.get(k) != kept.0.get(k));
            wrong.push(format!(
                "rooms ({} kept, {} found; first apart: {:?} kept, {:?} found)",
                kept.0.len(),
                self.rooms.len(),
                first.and_then(|k| kept.0.get(k)),
                first.and_then(|k| self.rooms.get(k)),
            ));
        }
        let bits = |v: &[f32]| v.iter().map(|f| f.to_bits()).collect::<Vec<_>>();
        if bits(&self.room_score) != bits(&kept.1) {
            let at = (0..self.room_score.len().max(kept.1.len())).find(|&i| {
                self.room_score.get(i).map(|f| f.to_bits()) != kept.1.get(i).map(|f| f.to_bits())
            });
            wrong.push(format!(
                "score per tile (first at slot {at:?}: {:?} kept, {:?} found)",
                at.and_then(|i| kept.1.get(i)),
                at.and_then(|i| self.room_score.get(i)),
            ));
        }
        let full = self.room_tiles.summary();
        if full.0 != tiles.0 {
            wrong.push("the rooms' tiles".into());
        }
        if full.1 != tiles.1 {
            wrong.push("the rooms per tile".into());
        }
        if self.rooms_out != kept.3 {
            wrong.push("doors to the outside".into());
        }
        if self.private_rooms != kept.4 || self.claimed_rooms != kept.5 {
            wrong.push("private rooms".into());
        }
        if self.claimed_places != kept.6 {
            wrong.push("claimed places".into());
        }
        if crate::view::structure_json(self) != structure {
            wrong.push("structure_json".into());
        }
        (
            self.rooms,
            self.room_score,
            self.room_tiles,
            self.rooms_out,
            self.private_rooms,
            self.claimed_rooms,
            self.claimed_places,
        ) = kept;
        if wrong.is_empty() {
            Ok(())
        } else {
            Err(wrong.join("; "))
        }
    }

    /// Scores cleanliness again in rooms that had a mess since: between structure changes
    /// only the dirt changes, so the rooms needn't be found again (a whole-town job).
    pub(super) fn rescore_rooms(&mut self) {
        self.room_tiles.rescore(
            &mut self.rooms,
            &self.content.room_rules,
            &self.dirt,
            &mut self.room_score,
        );
    }
}
