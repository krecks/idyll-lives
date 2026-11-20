//! The household's food after every step: going off, dirt, and invitations.

use super::*;

// ---- The household's food (after every step) ------------------------------------------------

/// After residents stepped: dinner's ready for the households of those who served, things
/// carried by people who stopped cooking, eating or washing up are put down, and (every
/// `ROOM_REFRESH_TICKS`) food goes off, dirty things dirty their room and things whose place
/// is gone go too.
pub(crate) fn after_step(w: &mut World) {
    if w.sims
        .iter()
        .any(|s| s.served.is_some() || s.held.is_some())
    {
        w.refresh_index();
        invite(w);
        settle(w);
    }
    if w.tick.is_multiple_of(crate::world::ROOM_REFRESH_TICKS) {
        update(w);
    }
}

/// "Dinner's ready": everyone else at home, awake and not busy with what the player asked for,
/// is keener to eat for a while (and drops what they chose themselves if they're hungry).
pub(crate) fn invite(w: &mut World) {
    let tick = w.tick;
    for i in 0..w.sims.len() {
        let Some(item) = w.sims[i].served.take() else {
            continue;
        };
        let Some(recipe) = w.items.get(item).and_then(|it| it.recipe) else {
            continue;
        };
        let household = w.sims[i].household;
        let Some(home) = w.households[household as usize].plot else {
            continue;
        };
        let World {
            sims,
            objects,
            content,
            plots,
            stairs,
            ..
        } = &mut *w;
        let until = tick + crate::social::minutes_to_ticks(content.cooking.invite_minutes);
        let r = &content.recipes[recipe];
        for (j, s) in sims.iter_mut().enumerate() {
            let at_home = s.tile();
            let at_home = plots[home as usize].holds(stairs, at_home.0, at_home.1);
            if j == i
                || s.household != household
                || !s.here()
                || s.away_until.is_some()
                || !at_home
                || content.life.baby(s.age)
                || s.asleep(content)
                || s.queue.front().is_some_and(|t| t.directed)
                || s.current.as_ref().is_some_and(|a| a.task.directed)
            {
                continue;
            }
            s.invited_until = until;
            crate::planner::think(s, crate::planner::ThoughtKind::DinnerReady, recipe, tick);
            // Hungry, and only doing something they picked themselves: they come at once.
            let picked = s.current.as_ref().is_some_and(|a| {
                matches!(
                    a.task.kind,
                    TaskKind::Use { .. } | TaskKind::MoveTo { .. } | TaskKind::Clean { .. }
                ) && a.tags & content.day_rhythm.sleep_tags == 0
            });
            if picked && hungry_for(s, content, r) {
                end_activity(s, content, objects);
            }
        }
    }
}

/// Whether `s` goes on with what they carry (`settle` leaves it with them): here, and at
/// something that keeps it (`TaskKind::carries_held`: cooking, carrying a harvest, ...), or about
/// to cook (a recipe loaded from a save).
pub(crate) fn going_on(s: &crate::world::Sim) -> bool {
    s.here()
        && match &s.current {
            Some(a) => a.task.kind.carries_held(),
            None => s
                .queue
                .front()
                .is_some_and(|t| matches!(t.kind, TaskKind::Cook { .. })),
        }
}

/// Whether `settle` has anything to put down.
pub(crate) fn must_settle(w: &World) -> bool {
    w.sims.iter().any(|s| s.held.is_some() && !going_on(s))
}

/// Puts down what people carry who are no longer cooking, eating or washing up (cancelled,
/// called away, off to work, gone). A recipe about to go on (loaded from a save) keeps it.
pub(crate) fn settle(w: &mut World) {
    w.refresh_index();
    for i in 0..w.sims.len() {
        let s = &w.sims[i];
        let Some(held) = s.held.filter(|_| !going_on(s)) else {
            continue;
        };
        w.sims[i].held = None;
        if !w.sims[i].here() {
            continue;
        }
        let (household, pos) = (w.sims[i].household, w.sims[i].pos);
        let candidates = w.households[household as usize]
            .plot
            .map_or(&[][..], |p| w.object_index.on(p));
        let at = Place {
            content: &w.content,
            objects: &w.objects,
            lot: &w.lot,
            candidates,
        };
        at.put_down(&mut w.items, held, household, pos, 4.0, false, None, w.tick);
    }
}

/// Every `ROOM_REFRESH_TICKS`: food goes off, dirty things make their tile dirtier, and things
/// whose place is gone (their table sold, their household moved away) go too, as do those left
/// off their household's plot (a plate at a café or in the park) once nobody eats from them.
pub(crate) fn update(w: &mut World) {
    let content = &w.content;
    let rules = &content.cooking;
    let tick = w.tick;
    let mut changed = false;
    let minutes = crate::world::ROOM_REFRESH_TICKS as f32 * crate::MINUTES_PER_TICK;
    // (Plates someone is eating from stay, wherever they are: a pastry at a café table.)
    let eaten: Vec<u32> = w
        .sims
        .iter()
        .filter_map(|s| match s.current.as_ref()?.task.kind {
            TaskKind::Eat { plate, .. } => plate,
            _ => None,
        })
        .collect();
    let mut keep = Vec::with_capacity(w.items.list.len());
    let ats: Vec<ItemAt> = w.items.list.iter().map(|it| it.at).collect();
    let holders = crate::items::holders(content, &w.objects, &ats);
    for (mut it, holder) in std::mem::take(&mut w.items.list).into_iter().zip(holders) {
        let household = w.households.get(it.household as usize);
        let home = household.and_then(|h| h.plot);
        let (x, z) = it.at.tile();
        let placed = match it.at {
            ItemAt::Floor { .. } => w.lot.in_bounds(x, z),
            _ => holder.is_some(),
        };
        let lives_here = w
            .sims
            .iter()
            .any(|s| s.here() && s.household == it.household);
        let on_home = home.is_some_and(|p| w.on_plot(p, x, z));
        if !placed || !lives_here || (!on_home && !eaten.contains(&it.id)) {
            changed = true;
            continue;
        }
        if it.edible() {
            let limit = if matches!(it.at, ItemAt::Stored { .. }) {
                rules.stored_spoil_minutes
            } else {
                rules.spoil_minutes
            };
            if (tick.saturating_sub(it.made_at)) as f32 * crate::MINUTES_PER_TICK >= limit {
                it.spoiled = true;
                changed = true;
            }
        }
        if it.to_clear(content) && !matches!(it.at, ItemAt::Stored { .. }) && w.lot.in_bounds(x, z)
        {
            let t = w.lot.tile_index(x, z);
            if t != crate::lot::VOID {
                w.dirt[t] = (w.dirt[t] + rules.dirty_dirt_per_hour * minutes / 60.0).min(1.0);
                w.room_tiles.touch(t);
            }
        }
        keep.push(it);
    }
    w.items.list = keep;
    if changed {
        w.items.version = w.items.version.wrapping_add(1);
    }
}

impl World {
    /// The items on or in any of `objects` (their holder, see `items::holder`; not what merely
    /// shares a tile with them, like the dinner on the table under a fruit bowl or over a rug),
    /// with the object holding each. Asked before any of them is lifted or sold.
    pub(crate) fn items_held_by(&self, objects: &[u32]) -> Vec<(u32, u32)> {
        self.items
            .list
            .iter()
            .filter_map(|it| {
                let holder = crate::items::holder(&self.content, &self.objects, &it.at)?;
                objects.contains(&holder).then_some((it.id, holder))
            })
            .collect()
    }

    /// Items `items` (on or in `old`) follow it to where it stands now (`new`, the same object
    /// moved or turned), turning with it.
    pub(crate) fn carry_items(
        &mut self,
        old: &ObjectInstance,
        new: &ObjectInstance,
        items: &[u32],
    ) {
        let content = &self.content;
        let (from, to) = (old.centre(content), new.centre(content));
        let turn = (i32::from(new.rot) - i32::from(old.rot)).rem_euclid(4);
        let mut moved = false;
        for it in &mut self.items.list {
            if matches!(it.at, ItemAt::Floor { .. }) || !items.contains(&it.id) {
                continue;
            }
            let c = it.at.centre();
            // Where it stood on the old top, on the new one (as things standing on it move).
            let p = match (
                surface::Top::of(content, old),
                surface::Top::of(content, new),
            ) {
                (Some(a), Some(b)) => b.world(a.local(c)),
                _ => {
                    let (dx, dz) = (c[0] - from[0], c[1] - from[1]);
                    // A quarter turn (rot + 1) takes local +z (the front) to +x.
                    let (rx, rz) = match turn {
                        0 => (dx, dz),
                        1 => (dz, -dx),
                        2 => (-dx, -dz),
                        _ => (-dz, dx),
                    };
                    [to[0] + rx, to[1] + rz]
                }
            };
            it.at = it.at.moved_to(p);
            if let ItemAt::Stored { .. } = it.at {
                it.at = ItemAt::Stored { x: new.x, z: new.z };
            }
            moved = true;
        }
        if moved {
            self.items.version = self.items.version.wrapping_add(1);
        }
    }

    /// Items `items` go (what they were on or in is being sold).
    pub(crate) fn drop_items(&mut self, items: &[u32]) {
        let before = self.items.list.len();
        self.items.list.retain(|it| !items.contains(&it.id));
        if self.items.list.len() != before {
            self.items.version = self.items.version.wrapping_add(1);
        }
    }

    /// Items on `tiles` move by `(dx, dz)` with the room they're in (`World::move_room`).
    pub(crate) fn shift_items(&mut self, tiles: &[(i32, i32)], dx: i32, dz: i32) {
        let mut moved = false;
        for it in &mut self.items.list {
            if tiles.contains(&it.at.tile()) {
                let c = it.at.centre();
                it.at = it.at.moved_to([c[0] + dx as f32, c[1] + dz as f32]);
                moved = true;
            }
        }
        if moved {
            self.items.version = self.items.version.wrapping_add(1);
        }
    }
}
