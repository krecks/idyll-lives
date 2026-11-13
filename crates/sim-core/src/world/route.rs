//! Where to fetch things, seats and grounds, and the ways to them.
//!
//! Split from `world.rs` (see `mod.rs`).

use super::*;

/// Where to fetch what `inter` at `object` needs first (`Interaction::fetch`): the interaction of
/// that id on the working thing on `object`'s plot that provides it, the nearest to `object`
/// that they may use now. None: nothing to fetch, or nowhere to fetch it from.
pub(crate) fn fetch_source(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
) -> Option<(u32, usize)> {
    let content = ctx.content;
    let fetch = inter.fetch.as_deref()?;
    let plot = ctx.object_plot.get(object as usize).copied().flatten()?;
    let at = objects[object as usize].centre(content);
    let mut best: Option<(f32, u32, usize)> = None;
    for &o in ctx.index.on(plot) {
        let obj = &objects[o as usize];
        let def = &content.objects[obj.def];
        if o == object || obj.broken() || def.provides & inter.requires != inter.requires {
            continue;
        }
        let Some(k) = def.interactions.iter().position(|i| i.id == fetch) else {
            continue;
        };
        if !obj.can_use_for(sim.id, def, k)
            || !obj.allows(&def.interactions[k], sim.id)
            || unfit(content, sim.age, &def.interactions[k]).is_some()
        {
            continue;
        }
        let c = obj.centre(content);
        let d = (c[0] - at[0]).hypot(c[1] - at[1]);
        if best.is_none_or(|b| d < b.0) {
            best = Some((d, o, k));
        }
    }
    best.map(|(_, o, k)| (o, k))
}

/// The nearest free seat within `inter`'s `seatNear` metres of `object`, one they may sit on
/// here. Seats are the sitting interactions residents eat in (`cooking.seatTags`). For a use
/// with `seatFacing` the slot they'd get on it looks at the object (within `SEAT_LOOKS`), and
/// for a screen (`SeatFacing::Front`) sits in front of it (within `SEAT_IN_FRONT` of the way it
/// faces): no sofa with its back to the TV.
pub(crate) fn seat_near_object(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
) -> Option<u32> {
    let content = ctx.content;
    let plot = ctx.object_plot.get(object as usize).copied().flatten();
    let rules = &content.cooking;
    let it = &objects[object as usize];
    let (tx, tz) = ctx
        .index
        .links
        .approach_of(object)
        .unwrap_or_else(|| it.front_tile(content));
    let target = [tx as f32 + 0.5, tz as f32 + 0.5];
    let middle = crate::surface::middle(content, it);
    let faces = |o: &ObjectInstance| {
        let Some(facing) = inter.seat_facing else {
            return true;
        };
        let def = &content.objects[o.def];
        let Some(slot) = o.pick_slot(
            sim.id,
            0..def.slots as usize,
            Some((def, sim.pos)),
            footstool_slots(content, objects, o.id),
        ) else {
            return false;
        };
        let at = o.slot_position(content, slot);
        let yaw = o.slot_yaw(content, slot);
        let to = [middle[0] - at[0], middle[1] - at[1]];
        let d = to[0].hypot(to[1]).max(1e-3);
        let looks = (yaw.sin() * to[0] + yaw.cos() * to[1]) / d >= SEAT_LOOKS.cos();
        let a = it.angle();
        let in_front = facing == crate::content::SeatFacing::Around
            || -(a.sin() * to[0] + a.cos() * to[1]) / d >= SEAT_IN_FRONT.cos();
        looks && in_front
    };
    objects
        .iter()
        .filter(|o| o.id != object)
        .filter(|o| {
            plot.is_none_or(|p| ctx.object_plot.get(o.id as usize).copied().flatten() == Some(p))
        })
        .filter(|o| {
            let def = &content.objects[o.def];
            !o.broken()
                && o.can_use(sim.id, def.slots)
                && o.sits(
                    def,
                    content.life.stage(sim.age),
                    rules.seat_tags,
                    rules.seat_avoid,
                )
        })
        .filter(|o| access(ctx, sim, ctx.object_plot[o.id as usize]) != Access::None)
        .map(|o| {
            let (fx, fz) = ctx
                .index
                .links
                .approach_of(o.id)
                .unwrap_or_else(|| o.front_tile(content));
            (
                o,
                ctx.nav
                    .stairs
                    .distance([fx as f32 + 0.5, fz as f32 + 0.5], target),
            )
        })
        // (Near enough before which way it faces: that picks a slot, and looks for footstools.)
        .filter(|&(o, d)| d <= inter.seat_near && faces(o))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(o, _)| o.id)
}

/// A seat taken for a `seatFacing` use looks at the thing within this of straight ahead...
pub(crate) const SEAT_LOOKS: f32 = std::f32::consts::FRAC_PI_3;
/// ...and, for a screen, is within this of the way it faces.
pub(crate) const SEAT_IN_FRONT: f32 = std::f32::consts::FRAC_PI_3;

/// Where `sim` may walk on the way to tile `goal` (see `path::Grounds`): home, the place
/// they're a guest at, public places, the plot they're on now (nobody is stuck in a garden)
/// and the one they're heading for. Everyone else's garden only when there's no other way.
pub(crate) fn grounds<'a>(
    tile_plot: &'a [u16],
    plots: &[Plot],
    households: &[Household],
    nav: &NavGrid,
    sim: &Sim,
    goal: (i32, i32),
) -> Grounds<'a> {
    let mut grounds = Grounds {
        tile_plot,
        open: plots.iter().map(|p| p.public).collect(),
        fenced: plots.iter().map(|p| !p.public).collect(),
    };
    let here = sim.tile();
    let ends = [here, goal].map(|(x, z)| grounds.plot(nav, x, z));
    let home = households.get(sim.household as usize).and_then(|h| h.plot);
    for p in ends
        .into_iter()
        .chain([home, sim.visiting.map(|v| v.plot)])
        .flatten()
    {
        if let Some(open) = grounds.open.get_mut(p as usize) {
            *open = true;
        }
    }
    grounds
}

/// `route` for `sim`, keeping out of other people's gardens (see `grounds`).
pub(crate) fn walk_route(
    ctx: &Ctx,
    sim: &Sim,
    goal: (i32, i32),
    target: [f32; 2],
) -> Option<Vec<[f32; 2]>> {
    let grounds = grounds(
        ctx.tile_plot,
        ctx.plots,
        ctx.households,
        &ctx.nav,
        sim,
        goal,
    );
    trip(
        &ctx.nav,
        Some(ctx.reach),
        ctx.travel,
        sim.pos,
        sim.tile(),
        goal,
        target,
        Some(&grounds),
    )
}

/// `route`, by road for a long trip in a town with a road network (`travel.rs`): a tile leg to
/// the road, along it, and a tile leg off it to `target`; A* over the tiles as ever for the rest
/// (or when there's no way by road).
#[allow(clippy::too_many_arguments)]
pub(crate) fn trip(
    nav: &NavGrid,
    reach: Option<&crate::path::Reach>,
    travel: &crate::travel::Travel,
    pos: [f32; 2],
    start: (i32, i32),
    goal: (i32, i32),
    target: [f32; 2],
    grounds: Option<&Grounds>,
) -> Option<Vec<[f32; 2]>> {
    // (How far, over the ground: a walk upstairs at home is no trip by road.)
    let ground = |p| nav.stairs.ground_pos(p);
    if travel.by_road(ground(pos), ground(target))
        && let Some(waypoints) = road_route(nav, reach, travel, pos, start, goal, target, grounds)
    {
        return Some(waypoints);
    }
    route(nav, reach, pos, start, goal, target, grounds)
}

/// A long walk by road (see `trip`): None if the goal can't be reached, or not by road.
#[allow(clippy::too_many_arguments)]
pub(crate) fn road_route(
    nav: &NavGrid,
    reach: Option<&crate::path::Reach>,
    travel: &crate::travel::Travel,
    pos: [f32; 2],
    start: (i32, i32),
    goal: (i32, i32),
    target: [f32; 2],
    grounds: Option<&Grounds>,
) -> Option<Vec<[f32; 2]>> {
    if let Some(reach) = reach
        && !reach.reaches(nav, start, goal)
    {
        return None;
    }
    let plan = travel.walk_plan(pos, target)?;
    let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
    let at = |p: &crate::travel::RoadPoint| travel.graph.roads[p.road as usize].at(p.s);
    // Off the road where the last leg can get to the goal from, then on it where the first can
    // get to from here (the nearest stretches first).
    // (Straight to and from the walkers' side of the road where that's open ground, else to
    // and from its middle.)
    let open = |p: [f32; 2]| {
        let (x, z) = tile(p);
        nav.tile_free(x, z) && reach.is_none_or(|r| r.reaches(nav, (x, z), goal))
    };
    let (off, tail) = plan.offs.iter().find_map(|off| {
        let q = at(off);
        let leg = route(nav, reach, q, tile(q), goal, target, grounds)?;
        Some((*off, leg))
    })?;
    let (head, mut road) = plan.ons.iter().find_map(|on| {
        let (road, _) = travel.walk_between(*on, off)?;
        let p = road.first().copied().filter(|&p| open(p)).unwrap_or(at(on));
        let head = route(nav, reach, pos, start, tile(p), p, grounds)?;
        Some((head, road))
    })?;
    if road.len() > 1
        && head
            .last()
            .is_some_and(|&h| crate::travel::dist(h, road[0]) < 0.05)
    {
        road.remove(0);
    }
    let mut waypoints = head;
    // Along the road: where something stands in the way (a thing on the pavement), round it.
    for &p in &road {
        let from = *waypoints.last().unwrap_or(&pos);
        if crate::travel::dist(from, p) < 0.05 {
            continue;
        }
        if !nav.line_walkable_with(from, p, grounds) {
            let around = route(nav, reach, from, tile(from), tile(p), p, grounds)?;
            waypoints.extend(around);
        } else {
            waypoints.push(p);
        }
    }
    let from = *waypoints.last().unwrap_or(&pos);
    match tail.first() {
        Some(&first) if !nav.line_walkable_with(from, first, grounds) => {
            let q = tail[0];
            waypoints.extend(route(nav, reach, from, tile(from), tile(q), q, grounds)?);
            waypoints.extend(tail.into_iter().skip(1));
        }
        _ => waypoints.extend(tail),
    }
    Some(waypoints)
}

/// Smoothed waypoints from `pos` to `target`, or None if unreachable; round the gardens
/// `grounds` keeps them out of, if any. With `reach`, a goal in another walkable region is
/// refused before the search starts (it would flood the town and still find nothing).
pub(crate) fn route(
    nav: &NavGrid,
    reach: Option<&crate::path::Reach>,
    pos: [f32; 2],
    start: (i32, i32),
    goal: (i32, i32),
    target: [f32; 2],
    grounds: Option<&Grounds>,
) -> Option<Vec<[f32; 2]>> {
    if let Some(reach) = reach
        && !reach.reaches(nav, start, goal)
    {
        return None;
    }
    let path = nav.find_path_with(start, goal, grounds)?;
    let mut waypoints = nav.smooth_with(&path, grounds);
    // The first waypoint is the centre of the current tile; skip it when we can head on directly.
    if waypoints.len() > 1 && nav.line_walkable_with(pos, waypoints[1], grounds) {
        waypoints.remove(0);
    }
    match waypoints.last_mut() {
        Some(last) => *last = target,
        None => waypoints.push(target),
    }
    Some(waypoints)
}

pub(crate) fn progress(
    sim: &mut Sim,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut crate::items::Items,
    rng: &mut Rng,
    ticks: u32,
) {
    let content = ctx.content;
    // (Before `current` is borrowed: how fast they walk, `walk_speed`.)
    let walk = sim.walk_speed(content) * crate::ride::speed(content, sim, objects);
    // (And how far they reach, on foot or on a bike, for the cars they keep clear of.)
    let body = crate::cars::Body::of(sim, content, objects, 0.0);
    let minutes = MINUTES_PER_TICK * ticks as f32;
    // On their way into a room someone keeps shut (a bathroom in use): they wait outside.
    if crate::privacy::wait_outside(sim, ctx, objects, ticks) {
        return;
    }
    let Some(act) = sim.current.as_mut() else {
        return;
    };
    // A use that ends on its first step, short of its time: given up straight away (see `after`).
    let mut given_up = false;
    let finished = match &mut act.phase {
        // Waiting at the door; `life::doors` lets them in or sends them home.
        Phase::Knocking { ticks: waited, .. } => {
            *waited += ticks;
            false
        }
        // At the wheel: the car takes them (`cars::update`).
        Phase::Driving => return,
        // At their kerb: the taxi calls them to its door (`taxi::update`), or goes without them.
        Phase::AtKerb { ticks: waited } => {
            *waited += ticks;
            // (Left by it: on their way after all, their own way.)
            if sim.taxi.is_none() {
                let task = act.task;
                sim.current = None;
                sim.queue.push_front(task);
            }
            return;
        }
        // Getting into the car: in, and off to work (`life`, `cars::board`).
        Phase::Boarding { ticks: waited } => {
            *waited += ticks;
            let t = *waited;
            if let Some(car) = sim.bound_for_car.and_then(|c| ctx.cars.get(c as usize)) {
                crate::cars::getting(sim, car, t, true);
            }
            if t >= crate::cars::GET_IN_TICKS {
                sim.current = None;
                sim.transition = Some(match sim.bound_for_car.take() {
                    Some(car) => crate::life::Transition::Board { car },
                    None => crate::life::Transition::Work,
                });
            }
            return;
        }
        // Getting out of the car, then on their way in.
        Phase::Alighting { ticks: waited } => {
            *waited += ticks;
            let t = *waited;
            let after = act.task;
            if let Some(car) = sim.bound_for_car.and_then(|c| ctx.cars.get(c as usize)) {
                crate::cars::getting(sim, car, t, false);
            }
            if t >= crate::cars::GET_OUT_TICKS {
                // (Out of a taxi at a place they're visiting: on in; else home.)
                let task = match after.kind {
                    TaskKind::Visit { .. } => after,
                    _ => Task {
                        kind: TaskKind::GoHome,
                        directed: false,
                        routine: false,
                    },
                };
                sim.current = None;
                sim.bound_for_car = None;
                sim.pose = Pose::Stand;
                sim.queue.push_front(task);
            }
            return;
        }
        // Waiting for a busy station: try again.
        Phase::Queued { ticks: waited } => {
            *waited += ticks;
            let waited = *waited;
            return crate::cooking::queued(sim, ctx, objects, items, waited);
        }
        Phase::Routing { waypoints, next } => {
            let per_tick = WALK_SPEED * walk;
            let mut budget = per_tick * ticks as f32;
            while budget > 0.0 && *next < waypoints.len() {
                let wp = waypoints[*next];
                // Off the top step onto the landing (or back): a storey's rows are far apart.
                let stairs = &ctx.nav.stairs;
                if stairs.depth > 0
                    && stairs.storey(wp[1].floor() as i32)
                        != stairs.storey(sim.pos[1].floor() as i32)
                {
                    sim.pos = wp;
                    budget -= 1.0;
                    *next += 1;
                    continue;
                }
                let (dx, dz) = (wp[0] - sim.pos[0], wp[1] - sim.pos[1]);
                let dist = dx.hypot(dz);
                // A car driving by: they wait for it (and after a while step aside, out of its way).
                // (Looked for at least 5 cm on, however short this tick's step: a distance, not a
                // pace, so slow walkers and more ticks a second see a car as soon.)
                let step = budget.min(dist).max(0.05);
                let to = [
                    sim.pos[0] + dx / dist.max(1e-4) * step,
                    sim.pos[1] + dz / dist.max(1e-4) * step,
                ];
                let passing = crate::cars::in_the_way(ctx.cars, sim.id, body, sim.pos, to)
                    .and_then(|c| ctx.cars[c].shown.map(|(at, yaw, ..)| (at, yaw)));
                if let Some((at, yaw)) = passing {
                    sim.car_wait = sim.car_wait.saturating_add(ticks);
                    // (Not into the way of another car.)
                    let others = || {
                        let all = crate::cars::vehicles(ctx.cars, None);
                        all.into_iter().filter(|d| d.at != at).collect::<Vec<_>>()
                    };
                    if sim.car_wait > crate::cars::MAKE_WAY_TICKS
                        && let others = others()
                        && let Some(p) = crate::cars::make_way(
                            at,
                            yaw,
                            &ctx.nav,
                            sim.pos,
                            (crate::cars::STEP_ASIDE * ticks as f32).min(SIDESTEP_MOST),
                            |p| crate::cars::clear(&others, body, p),
                        )
                    {
                        sim.pos = p;
                    }
                    break;
                }
                sim.car_wait = 0;
                if dist > 1e-4 {
                    sim.yaw = dx.atan2(dz);
                }
                // At a lit junction's crosswalk, closed to them: on foot, they wait at the kerb for
                // its walk phase, which their waiting brings round (`traffic::kerb_stop`; a side
                // road's crosswalk, for its main road's green: `Signal::opens_with`). Not in
                // the way of a car waiting for them, though: on, out of its way (a taxi turning the
                // corner and someone waiting there at the kerb waited for each other for good).
                // On a bike or an e-scooter they ride on with the traffic (stopped at every
                // junction along the road, riders were hours on their way out of town).
                let go = budget.min(dist);
                let ahead = [
                    sim.pos[0] + dx / dist.max(1e-4) * go,
                    sim.pos[1] + dz / dist.max(1e-4) * go,
                ];
                if sim.riding.is_none()
                    && let Some((s, arm, f)) =
                        crate::traffic::kerb_stop(ctx.travel, ctx.lights, sim.pos, ahead)
                    && !crate::cars::holding_up(ctx.cars, sim.id, body, sim.pos)
                {
                    sim.pos[0] += (ahead[0] - sim.pos[0]) * f;
                    sim.pos[1] += (ahead[1] - sim.pos[1]) * f;
                    sim.kerb_wait = Some((s, arm, ctx.tick));
                    break;
                }
                sim.kerb_wait = None;
                if dist <= budget {
                    sim.pos = wp;
                    budget -= dist;
                    *next += 1;
                } else {
                    sim.pos[0] += dx / dist * budget;
                    sim.pos[1] += dz / dist * budget;
                    budget = 0.0;
                }
            }
            if *next >= waypoints.len() {
                arrive(sim, ctx, objects, items);
                // Catching up on a turn (`lod.rs`): they got there partway through it, and
                // what they came for gets the rest.
                let rest = ((budget / per_tick) as u32).min(ticks - 1);
                let using = sim
                    .current
                    .as_ref()
                    .is_some_and(|a| matches!(a.phase, Phase::Using { .. }));
                if rest > 0 && using {
                    progress(sim, ctx, objects, items, rng, rest);
                }
            }
            false
        }
        Phase::Using { elapsed } => {
            // Done behind a shut door (`privacy.rs`): with someone else in the room as they get
            // there, not at all.
            if *elapsed == 0.0
                && let TaskKind::Use {
                    object,
                    interaction,
                } = act.task.kind
                && content.objects[objects[object as usize].def]
                    .interactions
                    .get(interaction)
                    .is_some_and(|i| i.private)
                && crate::privacy::overlooked(ctx, objects, sim.id, (object, interaction))
            {
                return end_activity(sim, content, objects);
            }
            // Sold at a post: not begun until its server serves them (`staff::hold`).
            if let TaskKind::Use {
                object,
                interaction,
            } = act.task.kind
                && let Some(inter) = content.objects[objects[object as usize].def]
                    .interactions
                    .get(interaction)
            {
                match crate::staff::hold(
                    ctx,
                    sim.being_served,
                    &mut sim.serve_wait,
                    object,
                    inter,
                    ticks,
                ) {
                    None | Some(crate::staff::Hold::Serve) => {}
                    Some(crate::staff::Hold::Wait) => return,
                    // (Not served: the money paid on arrival back.)
                    Some(crate::staff::Hold::GiveUp) => {
                        sim.pending_spend -= inter.cost.max(0);
                        return end_activity(sim, content, objects);
                    }
                }
            }
            // A game for two: not begun until the one they play with is at their place too, and
            // over once they've stopped (`partners::hold`).
            let mut over = false;
            if let TaskKind::Use {
                object,
                interaction,
            } = act.task.kind
                && let Some(inter) = content.objects[objects[object as usize].def]
                    .interactions
                    .get(interaction)
                    .filter(|i| i.partner)
            {
                match crate::partners::hold(
                    ctx,
                    sim.id,
                    (object, interaction),
                    *elapsed > 0.0,
                    sim.wants_partner,
                    &mut sim.partner_wait,
                    ticks,
                ) {
                    crate::partners::Hold::Play => {}
                    crate::partners::Hold::Wait => return,
                    crate::partners::Hold::Over => over = true,
                    // Nobody's coming: the game's use for one, if they were told to play, else given up.
                    crate::partners::Hold::Alone => match inter.alone.filter(|_| act.task.directed)
                    {
                        Some(k) => {
                            act.tags =
                                content.objects[objects[object as usize].def].interactions[k].tags;
                            act.task.kind = TaskKind::Use {
                                object,
                                interaction: k,
                            };
                            return;
                        }
                        None => return end_activity(sim, content, objects),
                    },
                }
            }
            let first = *elapsed == 0.0;
            if !over {
                *elapsed += minutes;
            }
            let elapsed = *elapsed;
            // A walk along a trail (`Content::trail_walks`): out along it and back while it lasts.
            if let TaskKind::Use {
                object,
                interaction,
            } = act.task.kind
            {
                let def = objects[object as usize].def;
                if content
                    .trail_walks
                    .binary_search(&(def, interaction))
                    .is_ok()
                {
                    let total = content.objects[def].interactions[interaction].minutes;
                    let at = objects[object as usize].centre(content);
                    if let Some((p, yaw)) = trail_step(
                        ctx.travel,
                        at,
                        elapsed,
                        total,
                        WALK_SPEED * walk * TICKS_PER_MINUTE as f32,
                    ) {
                        sim.pos = p;
                        sim.yaw = yaw;
                        sim.on_trail = Some(object);
                    }
                }
            }
            match act.task.kind {
                TaskKind::Use {
                    object,
                    interaction,
                } => {
                    let urgent = urgent_needs(sim, ctx, objects);
                    let done = over
                        || use_object(
                            sim,
                            ctx,
                            &objects[object as usize],
                            interaction,
                            elapsed,
                            minutes,
                            urgent,
                        );
                    // Tried for a baby for its time, the other one still with them: a baby may come.
                    if done
                        && !over
                        && content.objects[objects[object as usize].def]
                            .interactions
                            .get(interaction)
                            .is_some_and(|i| i.conceive && elapsed >= i.minutes)
                    {
                        sim.conceived = objects[object as usize].users().find(|&u| {
                            u != sim.id
                                && ctx.briefs.get(u as usize).is_some_and(|b| {
                                    b.game.is_some_and(|g| g.0 == object && g.1 == interaction)
                                })
                        });
                    }
                    let taken = content.objects[objects[object as usize].def]
                        .interactions
                        .get(interaction)
                        .map_or(0.0, |i| i.minutes_where(ctx.served(object)));
                    given_up = done && first && elapsed < taken;
                    // Every finished use wears it a little, by how much of a full use it was (a
                    // nap cut short when they're rested is a fraction); better quality wears slower.
                    if done {
                        let o = &mut objects[object as usize];
                        let def = &content.objects[o.def];
                        let rules = &content.object_rules;
                        let was = o.broken();
                        let share = def.interactions.get(interaction).map_or(1.0, |it| {
                            (elapsed / it.minutes_where(ctx.served(object)).max(1.0)).min(1.0)
                        });
                        let wear = def.wear_per_use
                            * share
                            * (1.0 - rules.wear_per_quality * f32::from(o.quality));
                        o.wear = (o.wear + wear.max(0.0)).min(1.0);
                        // Weeded and watered: the crop comes on (only while it grows).
                        let tends = def.interactions.get(interaction).map_or(0.0, |it| it.tends);
                        if tends > 0.0 && (0.0..1.0).contains(&o.growth) && def.grows.is_some() {
                            let before = o.growth_stage();
                            o.growth = (o.growth + tends * share).min(1.0);
                            sim.reshaped |= o.growth_stage() != before;
                        }
                        if !was && o.broken() {
                            sim.news = Some(News::Broke(object));
                        }
                    }
                    done
                }
                TaskKind::Repair { object } => {
                    let rules = &content.object_rules.repair;
                    if let Some(s) = rules.skill {
                        let max = content.skill_rules.max_level;
                        sim.skills[s] =
                            (sim.skills[s] + rules.skill_gain * minutes / 60.0).min(max);
                    }
                    let done = elapsed >= rules.minutes || !objects[object as usize].broken();
                    if done && objects[object as usize].broken() {
                        let level = rules.skill.map_or(0.0, |s| sim.skills[s]);
                        if rng.next_f32() < rules.chance_at(level) {
                            objects[object as usize].wear = 0.0;
                            sim.news = Some(News::Repaired(object));
                        }
                    }
                    done
                }
                TaskKind::Spot { accident } => {
                    let Some(rest) = content.accidents[accident].rest.as_ref() else {
                        return end_activity(sim, content, objects);
                    };
                    for (n, g) in rest
                        .gain_per_minute
                        .iter()
                        .enumerate()
                        .take(content.needs.len())
                    {
                        sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
                    }
                    elapsed >= rest.minutes
                }
                TaskKind::Clean { x, z } => {
                    let rules = &content.room_rules.clean;
                    let done = elapsed >= rules.minutes;
                    if done {
                        sim.mess = Some((x, z, -rules.amount));
                    }
                    done
                }
                TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. } => {
                    crate::cooking::using(sim, ctx, objects, items, elapsed, minutes)
                }
                TaskKind::Phone { .. } => {
                    let task = act.task;
                    crate::phone::using(sim, content, task, elapsed, minutes)
                }
                TaskKind::Carry { .. } => crate::carry::using(sim, elapsed),
                TaskKind::SetUp { .. } => crate::setup::using(sim, content, elapsed),
                TaskKind::Serve { .. } => crate::hosting::using(sim, ctx, objects, elapsed),
                _ => unreachable!(
                    "only Use, Repair, Spot, Clean, Cook, Eat, Dishes, Phone, Carry, SetUp and Serve tasks have a Using phase"
                ),
            }
        }
        // Conversations are driven by `conversation::update`, which sees both Sims.
        Phase::Calling { .. } | Phase::Waiting { .. } | Phase::Conversing { .. } => false,
    };
    if finished {
        match sim.current.as_ref().map(|a| a.task.kind) {
            // A carry's moment of lifting, putting in or away: on to what comes next.
            Some(TaskKind::Carry { .. }) => {
                crate::carry::step_done(sim, ctx, objects, items);
                if sim.current.is_some() {
                    return;
                }
            }
            // The parcel picked up, or the new thing put together (`setup.rs`).
            Some(TaskKind::SetUp { .. }) => crate::setup::step_done(sim, ctx, objects),
            // Something for a guest taken out, or handed over: on to them, or done.
            Some(TaskKind::Serve { .. }) => {
                crate::hosting::step_done(sim, ctx, objects);
                if sim.current.is_some() {
                    return;
                }
            }
            // A recipe's step, eating, washing up: on to what comes next.
            Some(kind) if crate::cooking::chain(&kind) => {
                let ate = matches!(kind, TaskKind::Eat { stage: 1, .. });
                crate::cooking::step_done(sim, ctx, objects, items);
                if sim.current.is_some() {
                    return;
                }
                // After a meal: the toilet, out of habit.
                if ate && sim.follow_up.is_none() {
                    sim.follow_up = habit(
                        sim,
                        ctx,
                        objects,
                        HabitTime::Meal,
                        ctx.briefs[sim.id as usize].plot,
                    );
                }
            }
            _ => {
                let used = sim.current.as_ref().map(|a| a.task);
                if let Some(Task {
                    kind:
                        TaskKind::Use {
                            object,
                            interaction,
                        },
                    ..
                }) = used
                    && !given_up
                {
                    leaves_at_end(sim, content, &mut objects[object as usize], interaction);
                }
                end_activity(sim, content, objects);
                if let Some(task) = used
                    && let TaskKind::Use {
                        object,
                        interaction,
                    } = task.kind
                    && let Some(o) = objects.get(object as usize)
                    && let Some(inter) = content.objects[o.def].interactions.get(interaction)
                {
                    // Food fetched: they sit down with it (at a table if there is one) and eat.
                    if let Some(recipe) = inter.eats {
                        let quality = crate::cooking::made_quality(content, o, inter, &sim.skills);
                        crate::cooking::eat_now(
                            sim,
                            Task {
                                routine: false,
                                ..task
                            },
                            recipe,
                            quality,
                            ctx,
                            objects,
                            items,
                        );
                        if sim.current.is_some() {
                            return;
                        }
                    } else if let Some(y) = inter.yields.filter(|_| !given_up) {
                        // A harvest: its produce carried in to the fridge (`carry.rs`).
                        let quality = crate::cooking::made_quality(content, o, inter, &sim.skills);
                        crate::carry::harvested(sim, task, y, quality, ctx, objects, items);
                        if sim.current.is_some() {
                            return;
                        }
                    } else if let Some(made) = inter.makes.filter(|_| !given_up && inter.after == 0)
                    {
                        // The washing taken out: on with it to the dryer or the line, or to fold it.
                        sim.follow_up = crate::machines::carry_on(sim, ctx, objects, object, made);
                    } else if inter.carried {
                        // Fed in the high chair: back to the crib in the arms.
                        sim.follow_up = crate::carry::carry_back(sim, task, ctx, objects, object);
                    } else if inter.after != 0 {
                        // (Not after a use given up the moment it began: no hands to wash after a toilet
                        // visit that didn't happen.)
                        if !given_up {
                            sim.follow_up = follow_up(sim, ctx, objects, object, inter.after);
                        }
                    } else if inter.close_after
                        || inter.tags
                            & (content.day_rhythm.sleep_tags | content.day_rhythm.nap_tags)
                            != 0
                    {
                        // Up from a night's sleep: the toilet first, out of habit.
                        if inter.tags & content.day_rhythm.sleep_tags != 0 {
                            sim.follow_up = habit(
                                sim,
                                ctx,
                                objects,
                                HabitTime::Wake,
                                ctx.object_plot[object as usize],
                            );
                        }
                        // Then what sleeping or napping opened is closed again (a wall bed folded
                        // up, a sofa bed's bed put away; a hot tub's cover put on after a soak,
                        // `close_after`), unless someone is still in it: whoever gets up last does it.
                        if let Some(close) = closing(sim, content, o, object) {
                            match sim.follow_up {
                                None => sim.follow_up = Some(close),
                                Some(_) => sim.queue.push_front(close),
                            }
                        }
                    }
                }
            }
        }
        // Catching up on a turn: it ended partway through, on average half a turn ago.
        sim.idle_ticks = (ticks - 1) / 2;
    }
}
