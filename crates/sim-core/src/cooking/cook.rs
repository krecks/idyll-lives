//! Choosing a recipe, its stations, and walking the cook through its steps.

use super::*;

// ---- Cooking, eating, washing up (a resident's step) --------------------------------------

/// The tile a resident uses `obj` from (`surface::Links`, cached for the tick).
pub(super) fn approach(ctx: &Ctx, obj: &ObjectInstance) -> (i32, i32) {
    ctx.index
        .links
        .approach_of(obj.id)
        .unwrap_or_else(|| obj.front_tile(ctx.content))
}

/// Whether a task is one of the cooking chain's (they go on from one step to the next).
pub(crate) fn chain(kind: &TaskKind) -> bool {
    matches!(
        kind,
        TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. }
    )
}

/// The interaction a recipe was started with (its skills, feeling and own effects count).
pub(crate) fn origin_interaction<'a>(
    content: &'a Content,
    objects: &[ObjectInstance],
    origin: Option<u32>,
    recipe: usize,
) -> Option<&'a Interaction> {
    let o = objects.get(origin? as usize)?;
    content.objects[o.def]
        .interactions
        .iter()
        .find(|i| i.recipe == Some(recipe))
}

/// The stations a resident has at home: working objects there. (A grill in the park is a
/// station only for a recipe started at it; see `find_station`.)
pub(crate) fn station_mask(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> StationMask {
    if sim.visiting.is_some() {
        return 0;
    }
    home_stations(ctx, sim.household, objects)
}

/// The stations household `household`'s home has, wherever they are: working objects there.
pub(crate) fn home_stations(ctx: &Ctx, household: u32, objects: &[ObjectInstance]) -> StationMask {
    home_objects(ctx, household)
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| !o.broken())
        .fold(0, |m, o| m | ctx.content.objects[o.def].stations)
}

/// Who's at home in the household (babies aside), the resident too.
pub(super) fn at_home<'a>(sim: &Sim, ctx: &'a Ctx) -> impl Iterator<Item = usize> + 'a {
    let home = ctx.households[sim.household as usize].plot;
    let household = sim.household;
    let found = move |j: usize| {
        let b = &ctx.briefs[j];
        (b.here
            && !b.away
            && !b.baby
            && b.household == household
            && home.is_some()
            && b.plot == home)
            .then_some(j)
    };
    // (Only the household's: `Ctx::members`.)
    debug_assert!(
        ctx.members
            .of(household)
            .iter()
            .filter_map(|&j| found(j as usize))
            .eq((0..ctx.briefs.len()).filter_map(found))
    );
    ctx.members
        .of(household)
        .iter()
        .filter_map(move |&j| found(j as usize))
}

/// Housemates at home who'd be glad of a serving of `r`.
pub(crate) fn hungry_housemates(sim: &Sim, ctx: &Ctx, r: &RecipeDef) -> usize {
    let below = ctx.content.cooking.hungry_below;
    let needs = ctx.content.needs.len();
    at_home(sim, ctx)
        .filter(|&j| j != sim.id as usize)
        .filter(|&j| (0..needs).any(|n| r.effects[n] > 0.1 && ctx.briefs[j].needs[n] < below))
        .count()
}

/// Room for this much of a need (or half a serving's worth, if less) and a serving does them good.
/// (Half a serving's worth alone: a meal that fills hunger whole was wanted only below half, so a
/// cook a quarter hungry left their own dinner to go off and had a snack later.)
const ROOM_FOR_A_SERVING: f32 = 0.25;
/// A cook the player sent to make a meal has some if they have this much room for it.
const ROOM_FOR_THEIR_OWN: f32 = 0.1;

/// Whether a serving of `r` would do them good (`ROOM_FOR_A_SERVING`).
pub(crate) fn hungry_for(sim: &Sim, content: &Content, r: &RecipeDef) -> bool {
    room_for(sim, content, r, ROOM_FOR_A_SERVING)
}

/// Whether the cook of a meal the player asked for has some of it: if a serving would do them
/// good, or they have a little room for it (`ROOM_FOR_THEIR_OWN`).
pub(crate) fn cook_has_some(sim: &Sim, content: &Content, r: &RecipeDef, directed: bool) -> bool {
    hungry_for(sim, content, r) || (directed && room_for(sim, content, r, ROOM_FOR_THEIR_OWN))
}

fn room_for(sim: &Sim, content: &Content, r: &RecipeDef, most: f32) -> bool {
    (0..content.needs.len())
        .any(|n| r.effects[n] > 0.1 && 1.0 - sim.needs[n] >= (r.effects[n] * 0.5).min(most))
}

/// The recipe of what they're eating (or on their way to eat).
pub(crate) fn eating(sim: &Sim, items: &Items) -> Option<usize> {
    let TaskKind::Eat { item, plate, .. } = sim.current.as_ref()?.task.kind else {
        return None;
    };
    sim.held
        .and_then(|h| h.recipe)
        .or_else(|| plate.and_then(|p| items.get(p)).and_then(|it| it.recipe))
        .or_else(|| item.and_then(|i| items.get(i)).and_then(|it| it.recipe))
}

/// A station for a step.
pub(super) enum Found {
    At(u32),
    /// There is one, but someone's using it.
    Busy,
    Missing,
}

/// The station to go to for `mask`: a working one they may use with a free slot, the one the
/// recipe was started from first, then (for outdoor recipes) outdoor ones or (otherwise)
/// indoor ones, then by the step's order of stations, then the nearest (`nearest`: the nearest
/// before the order, which only settles a tie). A station nobody uses otherwise (no
/// interactions: a wall cabinet) needs the tile in front of it free, and one higher than they
/// reach (a wall cupboard for a child) is passed over.
#[allow(clippy::too_many_arguments)]
pub(super) fn find_station(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    mask: StationMask,
    order: &[u8],
    origin: Option<u32>,
    outdoor: bool,
    nearest: bool,
) -> Found {
    let content = ctx.content;
    let lot = ctx.nav.lot;
    // (Started from it, indoors or out, rank), distance, tie, the object.
    type Rank = (u8, u8, usize);
    let mut best: Option<(Rank, f32, usize, u32)> = None;
    let mut busy = false;
    if sim.visiting.is_some() {
        return Found::Missing;
    }
    let reach = content.life.reach(sim.age);
    for &o in home_objects(ctx, sim.household).iter().chain(origin.iter()) {
        let obj = &objects[o as usize];
        let def = &content.objects[obj.def];
        if def.stations & mask == 0
            || def.reach > reach
            || obj.broken()
            || access(ctx, sim, ctx.object_plot[o as usize]) != Access::Full
        {
            continue;
        }
        // (A machine running, or done with its load still in it: not till it's emptied.)
        if !obj.can_use(sim.id, def.slots) || obj.run.is_some() {
            busy = true;
            continue;
        }
        let (fx, fz) = approach(ctx, obj);
        if def.interactions.is_empty() && !ctx.nav.tile_free(fx, fz) {
            continue;
        }
        let outside = lot.in_bounds(fx, fz) && lot.room_at(fx, fz) == crate::lot::OUTDOORS;
        let rank = order
            .iter()
            .position(|&i| def.stations & (1 << i) != 0)
            .unwrap_or(order.len());
        let (rank, tie) = if nearest { (0, rank) } else { (rank, 0) };
        let key = ((origin != Some(o)) as u8, (outside != outdoor) as u8, rank);
        let distance = ctx
            .nav
            .stairs
            .distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos);
        if best.is_none_or(|(k, d, t, _)| {
            key < k || (key == k && (distance < d || (distance == d && tie < t)))
        }) {
            best = Some((key, distance, tie, o));
        }
    }
    match best {
        Some((_, _, _, o)) => Found::At(o),
        None if busy => Found::Busy,
        None => Found::Missing,
    }
}

/// Starts (or carries on with) a cooking, eating or washing-up task.
pub(crate) fn start(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
) {
    match task.kind {
        TaskKind::Cook { recipe, step, .. } => {
            // A dish being served goes on only with the dish in hand (a save made on the way to
            // the table); else it was put down already.
            if serving(ctx.content, &task) && sim.held.is_none_or(|h| h.recipe != Some(recipe)) {
                return;
            }
            // Back to it after a crying baby (`called_away`): what the last step left is picked
            // up again.
            if sim.held.is_none() && step > 0 {
                sim.held = ctx.content.recipes[recipe]
                    .steps
                    .get(step as usize - 1)
                    .and_then(|s| s.holds)
                    .map(Held::of);
            }
            cook_next(sim, task, ctx, objects, items, None, None)
        }
        TaskKind::Eat { .. } => eat_next(sim, task, ctx, objects, items),
        TaskKind::Dishes { .. } => dishes_next(sim, task, ctx, objects, items, None),
        _ => {}
    }
}

/// Heads for `obj` (reserving a slot) for `task`; false if it can't be reached or is taken.
pub(super) fn go_to(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    obj: u32,
    tags: TagMask,
) -> bool {
    let content = ctx.content;
    let o = &objects[obj as usize];
    // (A seat of its own: round to it, `seat_approach`; at a station, their own spot along it,
    // `stand_spot`: two cooks at an island side by side.)
    let def = &content.objects[o.def];
    let slot = o.pick_slot(sim.id, 0..def.slots as usize, Some((def, sim.pos)), 0);
    let seat = matches!(task.kind, TaskKind::Eat { .. });
    let spot = slot
        .filter(|_| !seat)
        .and_then(|k| surface::stand_spot(content, &ctx.nav, o, k, false));
    let (fx, fz) = match spot {
        Some((t, ..)) => t,
        None => slot
            .filter(|_| seat)
            .and_then(|k| surface::seat_approach(content, &ctx.nav, o, k, sim.pos))
            .unwrap_or_else(|| approach(ctx, o)),
    };
    let target = spot.map_or([fx as f32 + 0.5, fz as f32 + 0.5], |(_, p, _)| p);
    let Some(waypoints) = walk_route(ctx, sim, (fx, fz), target) else {
        return false;
    };
    let o = &mut objects[obj as usize];
    if !o.reserve_seat(sim.id, &content.objects[o.def], sim.pos, 0) {
        return false;
    }
    sim.pose = Pose::Stand;
    sim.current = Some(Activity {
        task,
        phase: Phase::Routing { waypoints, next: 0 },
        tags,
    });
    true
}

/// Walks to tile `goal` for `task`.
pub(super) fn walk_to(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    goal: (i32, i32),
    tags: TagMask,
) -> bool {
    let target = [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5];
    let Some(waypoints) = walk_route(ctx, sim, goal, target) else {
        return false;
    };
    sim.pose = Pose::Stand;
    sim.current = Some(Activity {
        task,
        phase: Phase::Routing { waypoints, next: 0 },
        tags,
    });
    true
}

/// The tiles to stand on to reach an item, best first: in front of what it's on or in, then
/// the free tiles round it nearest the item (a table with chairs pulled up to its front); on
/// the floor, its tile, then the free ones beside it.
pub(super) fn item_access(ctx: &Ctx, objects: &[ObjectInstance], at: &ItemAt) -> Vec<(i32, i32)> {
    let nav = &ctx.nav;
    let free = |(x, z): (i32, i32)| nav.tile_free(x, z) && nav.stairs.step(x, z).is_none();
    let Some(o) = crate::items::holder(ctx.content, objects, at) else {
        let (x, z) = at.tile();
        return [(x, z), (x, z + 1), (x + 1, z), (x, z - 1), (x - 1, z)]
            .into_iter()
            .filter(|&t| free(t))
            .collect();
    };
    let obj = &objects[o as usize];
    let front = approach(ctx, obj);
    let footprint: Vec<(i32, i32)> = obj.tiles(ctx.content).collect();
    let storey = nav.stairs.storey(obj.z);
    let mut tiles: Vec<(i32, i32)> = if free(front) { vec![front] } else { Vec::new() };
    let mut round = Vec::new();
    for &(x, z) in &footprint {
        for t in [(x, z + 1), (x + 1, z), (x, z - 1), (x - 1, z)] {
            let open = nav.lot.in_bounds(t.0, t.1)
                && nav.lot.edge_between(x, z, t.0, t.1) == crate::lot::Edge::Open;
            if open
                && free(t)
                && nav.stairs.storey(t.1) == storey
                && !footprint.contains(&t)
                && t != front
                && !round.contains(&t)
            {
                round.push(t);
            }
        }
    }
    let c = at.centre();
    let d = |t: &(i32, i32)| (t.0 as f32 + 0.5 - c[0]).hypot(t.1 as f32 + 0.5 - c[1]);
    round.sort_by(|a, b| d(a).total_cmp(&d(b)));
    tiles.extend(round);
    tiles
}

/// Waits for a busy station for `task` (`waited` ticks so far: `Phase::Queued`), first stepping
/// aside off the spot they used (`aside`) so the next one to use it doesn't stand inside them.
pub(super) fn queue(sim: &mut Sim, task: Task, ctx: &Ctx, tags: TagMask, waited: Option<u32>) {
    if waited.is_none() && aside(sim, ctx).is_some_and(|t| walk_to(sim, task, ctx, t, tags)) {
        return;
    }
    sim.current = Some(Activity {
        task,
        phase: Phase::Queued {
            ticks: waited.unwrap_or(0),
        },
        tags,
    });
}

/// The tile beside `sim` to wait on: free, on their storey, nothing used from it (no object's
/// approach tile) and nobody else standing there; the ones straight beside first. None when
/// they aren't on a tile something is used from, or there is no such tile.
pub(super) fn aside(sim: &Sim, ctx: &Ctx) -> Option<(i32, i32)> {
    let nav = &ctx.nav;
    let used = |t: (i32, i32)| ctx.index.links.approach.contains(&t);
    let (x, z) = sim.tile();
    if !used((x, z)) {
        return None;
    }
    let taken = |t: (i32, i32)| {
        ctx.briefs.iter().enumerate().any(|(i, b)| {
            i as u32 != sim.id && !b.away && (b.pos[0].floor() as i32, b.pos[1].floor() as i32) == t
        })
    };
    let storey = nav.stairs.storey(z);
    [
        (0, 1),
        (1, 0),
        (0, -1),
        (-1, 0),
        (1, 1),
        (-1, 1),
        (1, -1),
        (-1, -1),
    ]
    .into_iter()
    .map(|(dx, dz)| (x + dx, z + dz))
    .find(|&t| {
        nav.tile_free(t.0, t.1)
            && nav.stairs.storey(t.1) == storey
            && nav.stairs.step(t.0, t.1).is_none()
            && !used(t)
            && !taken(t)
            && ((t.0 - x).abs() + (t.1 - z).abs() == 2
                || nav.lot.edge_between(x, z, t.0, t.1) == crate::lot::Edge::Open)
    })
}

/// Walks for `task` to the first of `goals` (a few at most) there's a way to.
pub(super) fn walk_to_any(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    goals: &[(i32, i32)],
    tags: TagMask,
) -> bool {
    goals
        .iter()
        .take(4)
        .any(|&goal| walk_to(sim, task, ctx, goal, tags))
}

/// Tags of a recipe being made (its interaction's, for "busy" checks and plans).
pub(super) fn cook_tags(
    content: &Content,
    objects: &[ObjectInstance],
    origin: Option<u32>,
    recipe: usize,
) -> TagMask {
    origin_interaction(content, objects, origin, recipe).map_or(0, |i| i.tags)
}

/// Goes on to the recipe's next step that can be done (or serves it): to its station, or
/// waiting for a busy one (`waited` ticks so far). `last` is the station of the step just done.
/// The minutes of optional steps done without are carried in the task's `minutes` to the next
/// step that has a station (no counter: they chop by the stove).
pub(super) fn cook_next(
    sim: &mut Sim,
    mut task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
    waited: Option<u32>,
    last: Option<u32>,
) {
    let content = ctx.content;
    sim.current = None;
    let TaskKind::Cook {
        recipe,
        step,
        origin,
        servings,
        skipped,
        minutes: carried,
        ..
    } = task.kind
    else {
        return;
    };
    let r = &content.recipes[recipe];
    let tags = cook_tags(content, objects, origin, recipe);
    // Before anything is bought: the home has to have every station the recipe needs.
    let at_origin = origin
        .and_then(|o| objects.get(o as usize))
        .map_or(0, |o| content.objects[o.def].stations);
    if step == 0 && servings == 0 && !r.possible(station_mask(sim, ctx, objects) | at_origin) {
        return cannot_cook(sim, ctx, tags);
    }
    // (A step the recipe can't do without, the ingredients from the fridge, is waited for longer:
    // whoever has the station may still be on the way to it from the garden.)
    let give_up = |optional: bool| {
        let limit = content.cooking.station_wait_minutes * if optional { 1.0 } else { 3.0 };
        waited.is_some_and(|w| w as f32 * crate::MINUTES_PER_TICK >= limit)
    };
    let (mut step, mut skipped, mut carried) = (step, skipped, carried);
    while let Some(s) = r.steps.get(step as usize) {
        task.kind = TaskKind::Cook {
            recipe,
            step,
            station: None,
            origin,
            servings,
            skipped,
            minutes: carried,
        };
        match find_station(sim, ctx, objects, s.mask, &s.at, origin, r.outdoor, false) {
            Found::At(o) => {
                let at = Task {
                    kind: TaskKind::Cook {
                        recipe,
                        step,
                        station: Some(o),
                        origin,
                        servings,
                        skipped,
                        minutes: carried,
                    },
                    ..task
                };
                if go_to(sim, at, ctx, objects, o, tags) {
                    return;
                }
            }
            Found::Busy if !give_up(s.optional) => {
                queue(sim, task, ctx, tags, waited);
                return;
            }
            _ => {}
        }
        if !s.optional {
            // Given up after the ingredients were bought: they go back (the money too).
            if servings > 0 {
                sim.pending_spend -= r.cost(servings).max(0);
            }
            return cannot_cook(sim, ctx, tags);
        }
        // Done without (no counter: chopped at the next station).
        carried = carried.saturating_add(s.minutes.round() as u16);
        step += 1;
        skipped += 1;
    }
    task.kind = TaskKind::Cook {
        recipe,
        step,
        station: None,
        origin,
        servings,
        skipped,
        minutes: 0,
    };
    serve(sim, task, ctx, objects, items, last);
}

/// Minutes until a cook could leave off for a crying baby (`World::wake_for_crying_babies`):
/// none on their way to a station or waiting for one, else what's left of the step they're at.
/// None if they aren't cooking.
pub(crate) fn step_left(sim: &Sim) -> Option<f32> {
    let act = sim.current.as_ref()?;
    let TaskKind::Cook { minutes, .. } = act.task.kind else {
        return None;
    };
    Some(match act.phase {
        Phase::Using { elapsed } => (f32::from(minutes) - elapsed).max(0.0),
        _ => 0.0,
    })
}

/// Called away to a crying baby (`World::wake_for_crying_babies`) while cooking: halfway
/// through a step, they finish it first (true: `step_done` sees the baby waiting); on their way
/// to a station or waiting for one, they leave off now (false), and the recipe waits at the
/// front of the queue to go on from that step (the care goes in front of it).
pub(crate) fn called_away(
    sim: &mut Sim,
    content: &Content,
    objects: &mut [ObjectInstance],
) -> bool {
    let Some(act) = sim
        .current
        .as_ref()
        .filter(|a| matches!(a.task.kind, TaskKind::Cook { .. }))
    else {
        return false;
    };
    if matches!(act.phase, Phase::Using { .. }) {
        return true;
    }
    let rest = left_off(act.task);
    end_activity(sim, content, objects);
    // (On the way to the table with the dish: it's put down where they are, `settle`.)
    if !serving(content, &rest) {
        sim.queue.push_front(rest);
    }
    false
}

/// A recipe left off before its step: to go on later from there, finding a station again.
pub(super) fn left_off(task: Task) -> Task {
    let TaskKind::Cook {
        recipe,
        step,
        origin,
        servings,
        skipped,
        minutes,
        ..
    } = task.kind
    else {
        return task;
    };
    Task {
        kind: TaskKind::Cook {
            recipe,
            step,
            station: None,
            origin,
            servings,
            skipped,
            minutes,
        },
        ..task
    }
}

/// Whether `task` is care for a crying baby they were called away to (not the player's).
pub(super) fn baby_care(task: &Task, content: &Content, objects: &[ObjectInstance]) -> bool {
    let TaskKind::Use {
        object,
        interaction,
    } = task.kind
    else {
        return false;
    };
    !task.directed
        && objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .is_some_and(|i| i.cares())
}

/// A step done with a crying baby waiting (`called_away`): they go now. The rest of the recipe
/// waits in the queue after the baby; a dish just served stays where it was put (they don't
/// sit down to it now: a plate in hand is put down, `settle`).
pub(super) fn leave_for_baby(sim: &mut Sim, content: &Content, objects: &mut [ObjectInstance]) {
    if !sim
        .queue
        .front()
        .is_some_and(|t| baby_care(t, content, objects))
    {
        return;
    }
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return;
    };
    end_activity(sim, content, objects);
    if matches!(task.kind, TaskKind::Cook { .. }) && !serving(content, &task) {
        let at = sim
            .queue
            .iter()
            .take_while(|t| baby_care(t, content, objects))
            .count();
        sim.queue.insert(at, left_off(task));
    }
}

/// Nowhere to make it: they think about wanting somewhere to cook.
pub(super) fn cannot_cook(sim: &mut Sim, ctx: &Ctx, tags: TagMask) {
    let content = ctx.content;
    sim.current = None;
    let eat = content.cooking.eat_tags;
    let fits =
        |a: &crate::planner::ActivityDef, t: TagMask| a.tags & t != 0 && !a.social && !a.visit;
    let activity = content
        .activities
        .iter()
        .position(|a| fits(a, tags & !eat))
        .or_else(|| content.activities.iter().position(|a| fits(a, tags)));
    if let Some(a) = activity {
        crate::planner::think(sim, crate::planner::ThoughtKind::NoPlace, a, ctx.tick);
    }
}

/// How many servings to make: one each for whoever's at home, within the recipe's.
pub(super) fn servings_for(sim: &Sim, ctx: &Ctx, r: &RecipeDef) -> u8 {
    (at_home(sim, ctx).count().clamp(1, 255) as u8).clamp(r.servings[0], r.servings[1])
}

/// Pays for the ingredients (how many servings); None if the money ran out.
pub(super) fn buy_ingredients(sim: &mut Sim, ctx: &Ctx, r: &RecipeDef) -> Option<u8> {
    let n = servings_for(sim, ctx, r);
    let cost = r.cost(n);
    let funds = ctx.households[sim.household as usize].funds - sim.pending_spend;
    if !affordable(cost, funds) {
        return None;
    }
    sim.pending_spend += cost.max(0);
    Some(n)
}

/// The household's objects that things can be put on or in.
pub(super) fn home_objects<'a>(ctx: &Ctx<'a>, household: u32) -> &'a [u32] {
    ctx.households[household as usize]
        .plot
        .map_or(&[], |p| ctx.index.on(p))
}
