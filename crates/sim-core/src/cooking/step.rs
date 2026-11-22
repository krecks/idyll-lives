//! One resident step of cooking, eating or washing up.

use super::*;

/// Arrived where a cooking, eating or washing-up step happens.
pub(crate) fn arrive(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items) {
    let content = ctx.content;
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return;
    };
    // The station or seat broke on the way (someone else wore it out): another one, or do without.
    let spot = match task.kind {
        TaskKind::Cook { station, .. } | TaskKind::Dishes { station, .. } => station,
        TaskKind::Eat { seat, stage: 1, .. } => seat,
        _ => None,
    };
    if spot.is_some_and(|o| objects[o as usize].broken()) {
        end_activity(sim, content, objects);
        return match task.kind {
            TaskKind::Cook { .. } => {
                cook_next(sim, left_off(task), ctx, objects, items, None, None)
            }
            TaskKind::Dishes {
                item,
                stage,
                scraps,
                ..
            } => dishes_next(
                sim,
                Task {
                    kind: TaskKind::Dishes {
                        item,
                        station: None,
                        stage,
                        scraps,
                    },
                    ..task
                },
                ctx,
                objects,
                items,
                None,
            ),
            TaskKind::Eat { item, .. } => eat_next(
                sim,
                Task {
                    kind: TaskKind::Eat {
                        item,
                        seat: None,
                        plate: None,
                        stage: 1,
                    },
                    ..task
                },
                ctx,
                objects,
                items,
            ),
            _ => {}
        };
    }
    match task.kind {
        // At the table with the dish: putting it down (`step_done`).
        TaskKind::Cook {
            station: Some(o), ..
        } if serving(content, &task) => {
            face(sim, ctx, &objects[o as usize]);
            if let Some(act) = sim.current.as_mut() {
                act.phase = Phase::Using { elapsed: 0.0 };
            }
        }
        TaskKind::Cook {
            recipe,
            step,
            station: Some(o),
            origin,
            servings,
            skipped,
            minutes: carried,
        } => {
            let r = &content.recipes[recipe];
            // The ingredients are paid for at the first step; if the money ran out, never mind.
            let servings = match servings {
                0 => match buy_ingredients(sim, ctx, r) {
                    Some(n) => n,
                    None => return end_activity(sim, content, objects),
                },
                n => n,
            };
            let at_origin = origin
                .and_then(|o| objects.get(o as usize))
                .map_or(0, |o| content.objects[o.def].stations);
            let mask = station_mask(sim, ctx, objects) | at_origin;
            let minutes =
                r.steps[step as usize].minutes + extra_minutes(r, step as usize, mask, carried);
            face(sim, ctx, &objects[o as usize]);
            let act = sim.current.as_mut().expect("checked above");
            act.task.kind = TaskKind::Cook {
                recipe,
                step,
                station: Some(o),
                origin,
                servings,
                skipped,
                minutes: minutes.round().clamp(1.0, f32::from(u16::MAX)) as u16,
            };
            act.phase = Phase::Using { elapsed: 0.0 };
        }
        TaskKind::Eat {
            item: Some(id),
            stage: 0,
            ..
        } => {
            if !take_serving(sim, content, items, id) {
                // The serving was promised to them when the task was made; if it has gone
                // (spoilt or cleared away) there is nothing to fetch, so end cleanly.
                sim.current = None;
                return;
            }
            // The last of it from the fridge: the empty pot comes out too, to be washed up.
            if items
                .get(id)
                .is_some_and(|it| matches!(it.at, ItemAt::Stored { .. }) && it.to_clear(content))
                && let Some(empty) = items.remove(id)
            {
                let dirty = Held {
                    servings: 1,
                    ..Held::of(empty.def)
                };
                let at = place(ctx, objects, empty.household);
                at.put_down(
                    items,
                    dirty,
                    empty.household,
                    sim.pos,
                    content.cooking.serve_range,
                    false,
                    None,
                    ctx.tick,
                );
            }
            let next = Task {
                kind: TaskKind::Eat {
                    item: Some(id),
                    seat: None,
                    plate: None,
                    stage: 1,
                },
                ..task
            };
            eat_next(sim, next, ctx, objects, items);
        }
        TaskKind::Eat {
            item,
            seat: Some(seat),
            stage: 1,
            ..
        } => {
            let o = &objects[seat as usize];
            // They stand back up where they came from (see `end_activity`).
            sim.seated_from = Some(sim.tile());
            let slot = o.slot_of(sim.id).unwrap_or(0);
            sim.pos = o.slot_position(content, slot);
            sim.yaw = o.slot_yaw(content, slot);
            sim.pose = Pose::Sit;
            // The plate goes on the table in front of them.
            let mut plate = None;
            if let Some(held) = sim.held
                && let Some(spot) = plate_spot(
                    sim,
                    ctx,
                    objects,
                    items,
                    seat,
                    content.items[held.def].radius,
                )
            {
                let (candidates, household) = dining(sim, ctx);
                let at = Place {
                    content,
                    objects,
                    lot: ctx.nav.lot,
                    candidates,
                };
                plate = at.put_down(
                    items,
                    held,
                    household.unwrap_or(sim.household),
                    sim.pos,
                    1.0,
                    false,
                    Some(spot),
                    ctx.tick,
                );
                sim.held = None;
            }
            let act = sim.current.as_mut().expect("checked above");
            act.task.kind = TaskKind::Eat {
                item,
                seat: Some(seat),
                plate,
                stage: 1,
            };
            act.phase = Phase::Using { elapsed: 0.0 };
        }
        // Stepped aside to eat standing (`eat_next`).
        TaskKind::Eat {
            seat: None,
            stage: 1,
            ..
        } => {
            sim.pose = Pose::Stand;
            if let Some(act) = sim.current.as_mut() {
                act.phase = Phase::Using { elapsed: 0.0 };
            }
        }
        TaskKind::Dishes { item, stage: 0, .. } => {
            // At the table: bent over it, picking the armful up (`pick_up` once it's done).
            if let Some(it) = items.get(item) {
                let (x, z) = it.at.tile();
                sim.yaw = (x as f32 + 0.5 - sim.pos[0]).atan2(z as f32 + 0.5 - sim.pos[1]);
            }
            sim.pose = Pose::Stand;
            if let Some(act) = sim.current.as_mut() {
                act.phase = Phase::Using { elapsed: 0.0 };
            }
        }
        TaskKind::Dishes {
            station: Some(o), ..
        } => {
            face(sim, ctx, &objects[o as usize]);
            if let Some(act) = sim.current.as_mut() {
                act.phase = Phase::Using { elapsed: 0.0 };
            }
        }
        // Stepped aside to wait for a busy station (`queue`).
        TaskKind::Cook { station: None, .. }
        | TaskKind::Dishes {
            station: None,
            stage: 2,
            ..
        } => {
            if let Some(act) = sim.current.as_mut() {
                act.phase = Phase::Queued { ticks: 0 };
            }
        }
        _ => sim.current = None,
    }
}

/// A tick of cooking, eating or washing up (`minutes` long). Returns whether the step is done.
pub(crate) fn using(
    sim: &mut Sim,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
    elapsed: f32,
    minutes: f32,
) -> bool {
    let content = ctx.content;
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return true;
    };
    match task.kind {
        TaskKind::Cook {
            recipe,
            station,
            origin,
            minutes: total,
            ..
        } => {
            // What cooking does for the cook (fun, a skill), better with good tools (not serving).
            if let Some(inter) = origin_interaction(content, objects, origin, recipe)
                .filter(|_| !serving(content, &task))
            {
                let tools = station
                    .and_then(|o| objects.get(o as usize))
                    .map_or(1.0, |o| content.object_rules.quality_factor(o.quality));
                let boost = tools * inter.skill_factor(&content.skill_rules, &sim.skills);
                for n in 0..content.needs.len() {
                    let g = inter.gain_per_minute[n];
                    let g = if g > 0.0 {
                        g * sim.mods.need_gain[n] * boost
                    } else {
                        g
                    };
                    sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
                }
                for (s, &gain) in inter.skill_gain[..content.skills.len()].iter().enumerate() {
                    if gain > 0.0 {
                        crate::world::practise(sim, content, s, gain * boost, minutes);
                    }
                }
            }
            elapsed >= f32::from(total)
        }
        TaskKind::Eat { plate, .. } => {
            let food = sim.held.map(|h| (h.recipe, h.quality)).or_else(|| {
                plate
                    .and_then(|p| items.get(p))
                    .map(|it| (it.recipe, it.quality))
            });
            let Some((Some(r), quality)) = food else {
                return true;
            };
            let r = &content.recipes[r];
            for n in 0..content.needs.len() {
                let g = r.effects[n] * quality / r.eat_minutes;
                let g = if g > 0.0 {
                    g * sim.mods.need_gain[n]
                } else {
                    g
                };
                sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
            }
            elapsed >= r.eat_minutes
        }
        TaskKind::Dishes { stage, scraps, .. } => {
            elapsed >= clearing_minutes(&content.cooking, stage, scraps, sim.held)
        }
        _ => true,
    }
}

/// Clearing away, at the table: everything to clear from the same place, up to an armful of one
/// kind, picked up (the food on it to be scraped off), and on to the bin or the sink.
pub(super) fn pick_up(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
) {
    let content = ctx.content;
    let TaskKind::Dishes { item, .. } = task.kind else {
        return;
    };
    // Everything to clear from the same place, up to an armful.
    let Some(target) = items
        .get(item)
        .filter(|it| it.household == sim.household)
        .cloned()
    else {
        sim.current = None;
        return;
    };
    let from = crate::items::holder(content, objects, &target.at);
    let same = |it: &ItemInstance| match from {
        Some(h) => crate::items::holder(content, objects, &it.at) == Some(h),
        None => it.at.tile() == target.at.tile(),
    };
    // (One kind of thing in an armful: plates stack, a pot and a plate don't, so the pot
    // waits for its own trip rather than vanishing under the plate.)
    let kind = washed(content, target.def);
    let mut picked: Vec<u32> = vec![item];
    for it in &items.list {
        if picked.len() >= usize::from(content.cooking.carry_dishes) {
            break;
        }
        if it.id != item
            && it.household == sim.household
            && it.to_clear(content)
            && same(it)
            && washed(content, it.def).is_none_or(|w| kind.is_none_or(|k| k == w))
        {
            picked.push(it.id);
        }
    }
    // Told to clear it: what's left there for being another kind (not for a full armful)
    // is the next trip, straight after this one, so the table is cleared, a trip per kind.
    // (Of their own accord, what's left is a chore for later, like any other.)
    if task.directed
        && let Some(rest) = items
            .list
            .iter()
            .find(|it| {
                !picked.contains(&it.id)
                    && it.household == sim.household
                    && it.to_clear(content)
                    && same(it)
                    && washed(content, it.def).is_some_and(|w| kind.is_some_and(|k| k != w))
            })
            .map(|it| it.id)
        && !sim
            .queue
            .iter()
            .any(|t| matches!(t.kind, TaskKind::Dishes { item, .. } if item == rest))
    {
        sim.queue.push_front(Task {
            kind: TaskKind::Dishes {
                item: rest,
                station: None,
                stage: 0,
                scraps: 0,
            },
            ..task
        });
    }
    // Food on them (eaten or not, gone off or not) is scraped off first; what's left is
    // carried dirty.
    let (mut scraps, mut dirty, mut carried, mut with_food) = (0u8, 0u8, None, None);
    for &id in &picked {
        let Some(it) = items.remove(id) else { continue };
        let food = it.servings > 0 || it.spoiled;
        scraps += u8::from(food);
        if let Some(def) = washed(content, it.def) {
            dirty += 1;
            carried.get_or_insert(def);
            if food {
                with_food.get_or_insert(it.def);
            }
        }
    }
    // One thing with food still on it is carried as it is (the half-eaten plate, the pot of
    // leftovers) until it's scraped (`step_done`); two or more as a stack, where there's one.
    let carried = match with_food {
        Some(def) if dirty == 1 => Some(def),
        _ => carried.map(|def: usize| {
            content.items[def]
                .stack_as
                .filter(|_| dirty > 1)
                .unwrap_or(def)
        }),
    };
    sim.held = carried.map(|def| Held {
        servings: dirty,
        ..Held::of(def)
    });
    let next = Task {
        kind: TaskKind::Dishes {
            item,
            station: None,
            stage: 1,
            scraps,
        },
        ..task
    };
    dishes_next(sim, next, ctx, objects, items, None);
}

/// Wears a station a little for a finished step (as a finished use does).
pub(super) fn wear(sim: &mut Sim, content: &Content, objects: &mut [ObjectInstance], o: u32) {
    let Some(obj) = objects.get_mut(o as usize) else {
        return;
    };
    let def = &content.objects[obj.def];
    let was = obj.broken();
    let w =
        def.wear_per_use * (1.0 - content.object_rules.wear_per_quality * f32::from(obj.quality));
    obj.wear = (obj.wear + w.max(0.0)).min(1.0);
    if !was && obj.broken() {
        sim.news = Some(crate::world::News::Broke(o));
    }
}

/// A step is done: on to the next one (another station, sitting down, the sink) or finished.
pub(crate) fn step_done(
    sim: &mut Sim,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
) {
    let content = ctx.content;
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return;
    };
    match task.kind {
        TaskKind::Dishes { stage: 0, .. } => {
            sim.current = None;
            pick_up(sim, task, ctx, objects, items);
        }
        TaskKind::Cook { station, .. } if serving(content, &task) => {
            sim.current = None;
            put_dish_down(sim, task, ctx, objects, items, station);
            leave_for_baby(sim, content, objects);
        }
        TaskKind::Cook {
            recipe,
            step,
            station,
            origin,
            servings,
            skipped,
            ..
        } => {
            if let Some(o) = station {
                objects[o as usize].release(sim.id);
                wear(sim, content, objects, o);
            }
            sim.held = content.recipes[recipe].steps[step as usize]
                .holds
                .map(Held::of);
            let next = TaskKind::Cook {
                recipe,
                step: step + 1,
                station: None,
                origin,
                servings,
                skipped,
                minutes: 0,
            };
            cook_next(
                sim,
                Task { kind: next, ..task },
                ctx,
                objects,
                items,
                None,
                station,
            );
            leave_for_baby(sim, content, objects);
        }
        TaskKind::Eat { seat, plate, .. } => {
            let food = sim.held.map(|h| (h.recipe, h.quality)).or_else(|| {
                plate
                    .and_then(|p| items.get(p))
                    .map(|it| (it.recipe, it.quality))
            });
            if let Some((Some(r), quality)) = food
                && quality >= 1.0
                && let Some(f) = content.recipes[r].eat_feeling
            {
                crate::social::add_feeling(&mut sim.feelings, f, &content.feelings, ctx.tick);
            }
            // A plate on the table is left there dirty; one in hand is put down.
            if let Some(p) = plate
                && let Some(it) = items.get_mut(p)
            {
                match content.items[it.def].empty_as {
                    Some(e) => {
                        it.def = e;
                        it.servings = 0;
                        it.recipe = None;
                        it.made_at = ctx.tick;
                    }
                    None => {
                        items.remove(p);
                    }
                }
            }
            let held = sim.held.take();
            let spot = held
                .and_then(|h| content.items[h.def].empty_as)
                .and_then(|e| {
                    seat.and_then(|s| {
                        plate_spot(sim, ctx, objects, items, s, content.items[e].radius)
                    })
                });
            // (Standing up first: a plate left on the floor goes where they stand.)
            end_activity(sim, content, objects);
            let (candidates, household) = dining(sim, ctx);
            if let Some(e) = held.and_then(|h| content.items[h.def].empty_as)
                && let Some(household) = household
            {
                let dirty = Held {
                    servings: 1,
                    ..Held::of(e)
                };
                let range = content.cooking.serve_range;
                let at = Place {
                    content,
                    objects,
                    lot: ctx.nav.lot,
                    candidates,
                };
                let put = at.put_down(
                    items, dirty, household, sim.pos, range, false, spot, ctx.tick,
                );
                // With nowhere to put it but the ground (eaten standing in the garden), it's taken
                // in to be washed up straight away, not left on the grass at their feet.
                if let Some(id) = put
                    .filter(|&id| {
                        matches!(items.get(id).map(|it| it.at), Some(ItemAt::Floor { .. }))
                    })
                    .filter(|_| household == sim.household)
                    .filter(|_| {
                        station_mask(sim, ctx, objects)
                            & (content.cooking.dishes | content.cooking.bin)
                            != 0
                    })
                    && !sim
                        .queue
                        .iter()
                        .any(|t| matches!(t.kind, TaskKind::Dishes { item, .. } if item == id))
                {
                    sim.queue.push_front(Task {
                        kind: TaskKind::Dishes {
                            item: id,
                            station: None,
                            stage: 0,
                            scraps: 0,
                        },
                        directed: false,
                        routine: false,
                    });
                }
            }
        }
        TaskKind::Dishes {
            item,
            station,
            stage,
            ..
        } => {
            if stage >= 3 {
                return end_activity(sim, content, objects);
            }
            if let Some(o) = station {
                objects[o as usize].release(sim.id);
            }
            // Scraped: on to the sink (with none, the dirty things went in the bin as well), what
            // they carry now as it is once scraped (the dirty plate). Washed: the clean things are
            // put away (out of hand now: called away, they're away already).
            if stage == 1
                && let Some(h) = sim.held.as_mut()
                && let Some(w) = washed(content, h.def)
            {
                h.def = w;
            }
            if stage == 1 && station_mask(sim, ctx, objects) & content.cooking.dishes == 0 {
                sim.held = None;
            }
            if stage == 2 {
                let n = sim.held.take().map_or(0, |h| h.servings.max(1));
                if let Some(o) = station {
                    wear(sim, content, objects, o);
                    // Into a dishwasher's rack: put away when it's emptied, not now.
                    if content.objects[objects[o as usize].def].rack > 0 {
                        let more = items.list.iter().any(|it| {
                            it.household == sim.household
                                && it.to_clear(content)
                                && washed(content, it.def).is_some()
                        });
                        crate::machines::rack_in(sim, content, &mut objects[o as usize], n, more);
                        return end_activity(sim, content, objects);
                    }
                }
            }
            let next = Task {
                kind: TaskKind::Dishes {
                    item,
                    station: None,
                    stage: stage + 1,
                    scraps: 0,
                },
                ..task
            };
            dishes_next(sim, next, ctx, objects, items, None);
        }
        _ => end_activity(sim, content, objects),
    }
}
