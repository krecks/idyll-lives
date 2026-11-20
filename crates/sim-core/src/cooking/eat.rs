//! Eating a serving: finding a seat, company, and the plate left behind.

use super::*;

/// Made to be eaten at once (a sandwich, a snack from the fridge; `quality` as a cooked
/// dish's): they hold a plate of `recipe` and sit down with it, at a table if there is one.
pub(crate) fn eat_now(
    sim: &mut Sim,
    task: Task,
    recipe: usize,
    quality: f32,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
) {
    sim.held = Some(plate(ctx.content, recipe, quality, ctx.tick));
    let eat = Task {
        kind: TaskKind::Eat {
            item: None,
            seat: None,
            plate: None,
            stage: 1,
        },
        ..task
    };
    eat_next(sim, eat, ctx, objects, items);
}

/// A plate of `recipe` (its serving) made at `tick`, `quality` as a cooked dish's: a snack taken
/// out to eat at once (`eat_now`), or to hand a guest (`hosting.rs`).
pub(crate) fn plate(content: &Content, recipe: usize, quality: f32, tick: u64) -> Held {
    Held {
        def: serving_of(content, recipe),
        servings: 1,
        quality,
        made_at: tick,
        recipe: Some(recipe),
    }
}

/// The item a serving of `recipe` is (its item's `serving`, else the item itself): what `plate`
/// holds, and what a customer is handed (`staff::handed`).
pub(crate) fn serving_of(content: &Content, recipe: usize) -> usize {
    let item = content.recipes[recipe].item;
    content.items[item].serving.unwrap_or(item)
}

/// How good what `inter` gives at `o` comes out (as a cooked dish's quality, a harvest's): the
/// object's quality and the skill it takes, as `skills` have it.
pub(crate) fn made_quality(
    content: &Content,
    o: &ObjectInstance,
    inter: &Interaction,
    skills: &[f32; crate::content::MAX_SKILLS],
) -> f32 {
    content.object_rules.quality_factor(o.quality)
        * inter.skill_factor(&content.skill_rules, skills)
}

/// Where someone eats: on the plot they're on (at home, at a host's, in a park). Its objects,
/// and whose plates are left there: the household living there (a guest's dirty plate is the
/// host's to wash up), else nobody's (`None`: nothing is left behind).
pub(super) fn dining<'a>(sim: &Sim, ctx: &Ctx<'a>) -> (&'a [u32], Option<u32>) {
    let home = ctx.households[sim.household as usize].plot;
    let Some(here) = ctx.briefs.get(sim.id as usize).and_then(|b| b.plot) else {
        return (home_objects(ctx, sim.household), Some(sim.household));
    };
    if home == Some(here) {
        return (ctx.index.on(here), Some(sim.household));
    }
    (
        ctx.index.on(here),
        ctx.households
            .iter()
            .position(|h| h.plot == Some(here))
            .map(|h| h as u32),
    )
}

/// Who keeps someone eating company, and where: those eating (or on their way to a seat to eat:
/// where they'll sit) first, then those sitting down free for a word (each in id order). At
/// home, their guests there; at a host's, the hosts there. None elsewhere.
pub(super) fn company(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> Vec<(u8, [f32; 2])> {
    let me = sim.id as usize;
    let Some(here) = ctx.briefs.get(me).and_then(|b| b.plot) else {
        return Vec::new();
    };
    let guest = sim.visiting.is_some_and(|v| v.plot == here);
    if !guest && ctx.households[sim.household as usize].plot != Some(here) {
        return Vec::new();
    }
    let mut company: Vec<(u8, [f32; 2])> = ctx
        .briefs
        .iter()
        .enumerate()
        .filter(|&(j, b)| {
            j != me
                && b.here
                && !b.away
                && !b.baby
                && b.plot == Some(here)
                && if guest {
                    b.guest_at.is_none() && ctx.households[b.household as usize].plot == Some(here)
                } else {
                    b.guest_at == Some(here)
                }
        })
        .filter_map(|(j, b)| {
            let seat = b.eats_at.and_then(|o| objects.get(o as usize));
            match seat {
                Some(o) => {
                    let slot = o.slot_of(j as u32).unwrap_or(0);
                    Some((0, o.slot_position(ctx.content, slot)))
                }
                None => (b.seated && b.available).then_some((1, b.pos)),
            }
        })
        .collect();
    company.sort_by_key(|c| c.0);
    company
}

/// The best free place to sit and eat: a chair at a table (`surface::dining_seat`), else a
/// table with seats of its own, else any seat. With company (a guest at home, the hosts at a
/// host's) sitting down, first a seat within talking range of them, in sight (beside one
/// eating before one only sitting), so the two talk over the meal (`conversation::from_seat`);
/// else the best nearest to them.
/// (`skip`: seats there's no way to.)
pub(super) fn eat_seat(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    items: &Items,
    skip: &[u32],
) -> Option<u32> {
    let content = ctx.content;
    let rules = &content.cooking;
    let candidates: Vec<u32> = dining(sim, ctx)
        .0
        .iter()
        .copied()
        .filter(|&o| {
            let obj = &objects[o as usize];
            !skip.contains(&o)
                && !obj.broken()
                && obj.can_use(sim.id, content.objects[obj.def].slots)
        })
        .collect();
    let stage = content.life.stage(sim.age);
    let taken = items.occupancy(content);
    let company = company(sim, ctx, objects);
    for rank in 0..2 {
        let near: Vec<[f32; 2]> = company
            .iter()
            .filter(|c| c.0 == rank)
            .map(|c| c.1)
            .collect();
        let Some(&first) = near.first() else {
            continue;
        };
        // (A place on it free, within talking range of one of them.)
        let beside: Vec<u32> = candidates
            .iter()
            .copied()
            .filter(|&o| {
                let obj = &objects[o as usize];
                let slots = usize::from(content.objects[obj.def].slots);
                (0..slots)
                    .filter(|&k| obj.users[k].is_none())
                    .map(|k| obj.slot_position(content, k))
                    .any(|p| {
                        near.iter().any(|&c| {
                            (p[0] - c[0]).hypot(p[1] - c[1]) <= crate::conversation::SEAT_RANGE
                                && crate::conversation::in_sight(&ctx.nav, p, c)
                        })
                    })
            })
            .collect();
        if let Some(d) = surface::dining_seat(
            content,
            objects,
            &ctx.index.links,
            &beside,
            first,
            stage,
            &taken,
        ) {
            return Some(d.object);
        }
    }
    let from = company.first().map_or(sim.pos, |c| c.1);
    if let Some(d) = surface::dining_seat(
        content,
        objects,
        &ctx.index.links,
        &candidates,
        from,
        stage,
        &taken,
    ) {
        return Some(d.object);
    }
    let sits = |o: u32, tags: TagMask| {
        let obj = &objects[o as usize];
        obj.sits(&content.objects[obj.def], stage, tags, rules.seat_avoid)
    };
    let distance = |o: u32| {
        let (fx, fz) = approach(ctx, &objects[o as usize]);
        ctx.nav
            .stairs
            .distance([fx as f32 + 0.5, fz as f32 + 0.5], from)
    };
    let nearest = |pick: &dyn Fn(u32) -> bool| {
        candidates
            .iter()
            .copied()
            .filter(|&o| pick(o))
            .min_by(|&a, &b| distance(a).total_cmp(&distance(b)))
    };
    nearest(&|o| {
        content.objects[objects[o as usize].def].surface.is_some() && sits(o, rules.eat_tags)
    })
    .or_else(|| nearest(&|o| sits(o, rules.seat_tags)))
}

/// Where the plate goes for someone sitting at `seat`: on the table in front of them, or on
/// the seat's own table.
pub(super) fn plate_spot(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    items: &Items,
    seat: u32,
    radius: f32,
) -> Option<SurfaceSpot> {
    let content = ctx.content;
    let occupied = items.occupancy(content);
    let slot = objects[seat as usize].slot_of(sim.id).unwrap_or(0);
    surface::spot_for_seat(
        content,
        objects,
        &ctx.index.links,
        seat,
        slot,
        radius,
        &occupied,
    )
    .or_else(|| {
        content.objects[objects[seat as usize].def].surface?;
        surface::free_surface_spot_near(
            content,
            objects,
            &[seat],
            sim.pos,
            2.5,
            0,
            radius,
            &occupied,
        )
    })
}

/// Eating, step by step: fetch a serving, then sit down with it (or eat standing).
pub(super) fn eat_next(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
) {
    let content = ctx.content;
    let tags = content.cooking.eat_tags;
    let TaskKind::Eat { item, stage, .. } = task.kind else {
        return;
    };
    sim.current = None;
    if stage == 0 {
        let Some(it) = item
            .and_then(|i| items.get(i))
            .filter(|it| it.edible() && it.household == sim.household)
        else {
            return;
        };
        walk_to_any(sim, task, ctx, &item_access(ctx, objects, &it.at), tags);
        return;
    }
    if sim.held.is_none_or(|h| h.recipe.is_none()) {
        return;
    }
    // (A seat there's no way to, a bedroom kept shut say: the next best, a few at most.)
    let mut skip = Vec::new();
    while skip.len() < 3
        && let Some(seat) = eat_seat(sim, ctx, objects, items, &skip)
    {
        let at = Task {
            kind: TaskKind::Eat {
                item,
                seat: Some(seat),
                plate: None,
                stage: 1,
            },
            ..task
        };
        if go_to(sim, at, ctx, objects, seat, tags) {
            return;
        }
        skip.push(seat);
    }
    // Nowhere to sit: standing up, first stepping off the spot something is used from (the grill
    // they cooked at), so the next one to use it doesn't stand inside them.
    sim.pose = Pose::Stand;
    let standing = Task {
        kind: TaskKind::Eat {
            item,
            seat: None,
            plate: None,
            stage: 1,
        },
        ..task
    };
    if aside(sim, ctx).is_some_and(|t| walk_to(sim, standing, ctx, t, tags)) {
        return;
    }
    sim.current = Some(Activity {
        task: Task {
            kind: TaskKind::Eat {
                item,
                seat: None,
                plate: None,
                stage: 1,
            },
            ..task
        },
        phase: Phase::Using { elapsed: 0.0 },
        tags,
    });
}

/// Takes a serving of `id` into their hands (false: it's gone or spoilt).
pub(super) fn take_serving(sim: &mut Sim, content: &Content, items: &mut Items, id: u32) -> bool {
    let household = sim.household;
    let Some(it) = items
        .get_mut(id)
        .filter(|it| it.edible() && it.household == household)
    else {
        return false;
    };
    let def = &content.items[it.def];
    it.servings -= 1;
    sim.held = Some(Held {
        def: def.serving.unwrap_or(it.def),
        servings: 1,
        quality: it.quality,
        made_at: it.made_at,
        recipe: it.recipe,
    });
    if it.servings == 0 {
        match def.empty_as.filter(|_| def.serving.is_some()) {
            // The empty pot stays to be washed up.
            Some(e) => {
                it.def = e;
                it.recipe = None;
            }
            // A plate of its own: it's in their hands now.
            None => {
                items.remove(id);
            }
        }
    }
    true
}
