//! Washing up: the queue, the minutes, and what a load holds.

use super::*;

/// Clearing away, step by step: fetch what's to be cleared (stage 0); scrape the food off
/// into a bin (1; with none, it's done at the sink); wash up (2; `waited`: ticks waited for a
/// busy sink); put the clean things away at a counter (3; with none, from the sink).
pub(super) fn dishes_next(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
    waited: Option<u32>,
) {
    let content = ctx.content;
    let rules = &content.cooking;
    let tags = rules.dishes_tags;
    let TaskKind::Dishes {
        item,
        stage,
        scraps,
        ..
    } = task.kind
    else {
        return;
    };
    sim.current = None;
    let at = |o: u32, stage: u8| Task {
        kind: TaskKind::Dishes {
            item,
            station: Some(o),
            stage,
            scraps,
        },
        ..task
    };
    match stage {
        0 => {
            let Some(it) = items.get(item).filter(|it| it.household == sim.household) else {
                return;
            };
            walk_to_any(sim, task, ctx, &item_access(ctx, objects, &it.at), tags);
        }
        1 => {
            // (With nowhere to wash up, what they carry goes in the bin too: see `step_done`.)
            let no_sink = station_mask(sim, ctx, objects) & rules.dishes == 0;
            let scraps = if no_sink && sim.held.is_some() {
                scraps.max(1)
            } else {
                scraps
            };
            if scraps > 0
                && let Found::At(o) =
                    find_station(sim, ctx, objects, rules.bin, &[], None, false, false)
                && go_to(
                    sim,
                    Task {
                        kind: TaskKind::Dishes {
                            item,
                            station: Some(o),
                            stage: 1,
                            scraps,
                        },
                        ..task
                    },
                    ctx,
                    objects,
                    o,
                    tags,
                )
            {
                return;
            }
            let sink = Task {
                kind: TaskKind::Dishes {
                    item,
                    station: None,
                    stage: 2,
                    scraps,
                },
                ..task
            };
            dishes_next(sim, sink, ctx, objects, items, None);
        }
        // Everything was thrown away: nothing to wash.
        2 if sim.held.is_none() && scraps == 0 => {}
        2 => {
            let give_up = waited
                .is_some_and(|w| w as f32 * crate::MINUTES_PER_TICK >= rules.station_wait_minutes);
            match find_station(sim, ctx, objects, rules.dishes, &[], None, false, false) {
                Found::At(o) => {
                    go_to(sim, at(o, 2), ctx, objects, o, tags);
                }
                Found::Busy if !give_up => {
                    queue(sim, task, ctx, tags, waited);
                }
                // Nowhere to wash up: what they carry is put down again (after the step).
                _ => {}
            }
        }
        _ => {
            // The nearest cupboard or counter; a cupboard over a counter first.
            if let Found::At(o) = find_station(
                sim,
                ctx,
                objects,
                rules.put_away,
                &rules.put_away_order,
                None,
                false,
                true,
            ) {
                go_to(sim, at(o, 3), ctx, objects, o, tags);
            }
        }
    }
}

/// What clearing away `def` leaves to wash up: itself if it's dirty, else what it is empty (a
/// pot of stew, a dirty pot); none, it's thrown away whole.
pub(super) fn washed(content: &Content, def: usize) -> Option<usize> {
    let d = &content.items[def];
    if d.dirty { Some(def) } else { d.empty_as }
}

/// How long a step of clearing away takes: scraping the food off `scraps` things (at the sink
/// too, if there was no bin), washing what they carry, or putting it away.
pub(crate) fn clearing_minutes(
    rules: &CookingRules,
    stage: u8,
    scraps: u8,
    held: Option<Held>,
) -> f32 {
    let scraping = rules.scrape_minutes * f32::from(scraps);
    match stage {
        0 => rules.pick_up_minutes,
        1 => scraping,
        3 => rules.put_away_minutes,
        _ => rules.dishes_minutes * f32::from(held.map_or(0, |h| h.servings.max(1))) + scraping,
    }
}

/// Waiting for a busy station: try again (or, after a while, do without).
pub(crate) fn queued(
    sim: &mut Sim,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
    waited: u32,
) {
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return;
    };
    match task.kind {
        TaskKind::Cook { .. } => cook_next(sim, task, ctx, objects, items, Some(waited), None),
        TaskKind::Dishes { .. } => dishes_next(sim, task, ctx, objects, items, Some(waited)),
        // Handed something to eat (`hosting.rs`): they sit down with it.
        TaskKind::Eat { .. } => eat_next(sim, task, ctx, objects, items),
        _ => sim.current = None,
    }
}

/// Minutes of the recipe's optional steps done without that are spent at step `step` instead:
/// `carried`, those skipped since the step before (see `cook_next`), and when no later step has
/// a station in `mask`, all the later ones too (a fridge-only kitchen makes the whole meal at
/// the fridge).
pub(super) fn extra_minutes(r: &RecipeDef, step: usize, mask: StationMask, carried: u16) -> f32 {
    let later = &r.steps[(step + 1).min(r.steps.len())..];
    let trailing = if later.iter().all(|s| s.optional && s.mask & mask == 0) {
        later.iter().map(|s| s.minutes).sum()
    } else {
        0.0
    };
    f32::from(carried) + trailing
}

/// Turns to face `o`, standing: straight at it from their own spot along it (`stand_spot`).
pub(super) fn face(sim: &mut Sim, ctx: &Ctx, o: &ObjectInstance) {
    let content = ctx.content;
    let c = o.centre(content);
    sim.yaw = match o
        .slot_of(sim.id)
        .and_then(|k| surface::at_stand_spot(content, &ctx.nav, o, k, false, sim.pos))
    {
        Some((_, yaw)) => yaw,
        None => (c[0] - sim.pos[0]).atan2(c[1] - sim.pos[1]),
    };
    sim.pose = Pose::Stand;
}
