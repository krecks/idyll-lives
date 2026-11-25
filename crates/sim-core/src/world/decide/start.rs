//! Starting a task: what it takes, and what happens when it cannot be started.

use super::*;
use crate::content::{HabitTime, Layer, Pose};
use crate::conversation;
use crate::world::{
    Access, Activity, Ctx, News, ObjectInstance, Phase, Sim, Task, TaskKind, access, closed_now,
    conditions_met, exit_on_foot, fetch_source, footstool_slots, guest_may, habit, leisure_ride,
    no_bike_out, open_ground_near, order_access, outing_fits_now, seat_near_object, taxi_from_home,
    unfit, upper_barred, walk_route,
};

pub(crate) fn start_task(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut crate::items::Items,
) {
    let content = ctx.content;
    let nav = &ctx.nav;
    // At the wheel of a car on a trip (on their way there or back, as a game was loaded): they
    // drive on.
    if matches!(task.kind, TaskKind::Visit { .. } | TaskKind::GoHome)
        && let Some(car) = crate::cars::driving(ctx.cars, sim.id)
    {
        sim.bound_for_car = Some(car);
        sim.current = Some(Activity {
            task,
            phase: Phase::Driving,
            tags: Default::default(),
        });
        return;
    }
    // (Fetched for this use, if it's the one: see `Sim::fetched`; sent to play with someone,
    // `Sim::joins`.)
    let fetched = sim.fetched.take();
    let joins = sim.joins.take();
    // Nothing their age isn't for, nor out of its hours, under a roof or with nothing to work
    // on, however it was chosen (`unfit`, `unmet`; an order was refused already).
    if let TaskKind::Use {
        object,
        interaction,
    } = task.kind
        && let Some(o) = objects.get(object as usize)
        && content.objects[o.def]
            .interactions
            .get(interaction)
            .is_some_and(|i| {
                unfit(content, sim.age, i).is_some()
                    || !conditions_met(ctx, sim, o, i)
                    || closed_now(ctx, objects, object, i)
            })
    {
        return;
    }
    // Nobody sets off for a place shut now (its hours, `Plot::open_at`), asked to or not: one shut
    // since it was chosen.
    if let TaskKind::Visit { plot } = task.kind
        && sim.visiting.is_none_or(|v| v.plot != plot)
        && !ctx.plots[plot as usize].open_at(ctx.day, ctx.minute)
    {
        return;
    }
    // An outing of their own choosing weighed by car that they'd walk after all (the car taken
    // meanwhile, say), too far for the way they'd go now: not now (`outing_fits_now`). Nor are
    // they on it any more, set out for their car (`life::Transition::Visit`) and not there: else,
    // still on it at home, they'd wander off to the place on foot after all, hours away.
    if let TaskKind::Visit { plot } = task.kind
        && !task.directed
        && !outing_fits_now(sim, ctx, objects, plot)
    {
        if sim
            .visiting
            .is_some_and(|v| v.plot == plot && v.arrived == 0)
        {
            sim.visiting = None;
        }
        return;
    }
    // Off somewhere else from where they left their bike out (`Sim::bike_parked`): to work, or
    // to another place: it's back at its place, and they go on foot. (Home, they ride it.)
    let elsewhere = match task.kind {
        TaskKind::Work => true,
        TaskKind::Visit { plot } => sim.visiting.is_none_or(|v| v.plot != plot),
        _ => false,
    };
    if sim.bike_parked.is_some() && elsewhere {
        crate::ride::get_off(sim, objects);
    }
    // A crib's care done by day where the baby is carried to (the high chair's feed, `carry.rs`).
    if let Some(carry) = crate::carry::instead(sim, &task, ctx, objects) {
        return crate::carry::start(sim, carry, ctx, objects);
    }
    // A recipe isn't made where it was picked: the cook goes round its stations, starting with
    // the first (see `cooking.rs`).
    if let TaskKind::Use {
        object,
        interaction,
    } = task.kind
        && let Some(recipe) = objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .and_then(|i| i.recipe)
    {
        let kind = TaskKind::Cook {
            recipe,
            step: 0,
            station: None,
            origin: Some(object),
            servings: 0,
            skipped: 0,
            minutes: 0,
        };
        return crate::cooking::start(sim, Task { kind, ..task }, ctx, objects, items);
    }
    let mut reserved_seat = None;
    // (A seat near the thing to sit on for its use, `seatNear`: without one they stand at it.)
    let mut seat_near = false;
    let (target, goal, tags) = match task.kind {
        TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. } => {
            return crate::cooking::start(sim, task, ctx, objects, items);
        }
        TaskKind::Phone { .. } => return crate::phone::start(sim, task, ctx, objects),
        TaskKind::Carry { .. } => return crate::carry::start(sim, task, ctx, objects),
        TaskKind::SetUp { .. } => return crate::setup::start(sim, task, ctx),
        TaskKind::Serve { .. } => return crate::hosting::start(sim, task, ctx, objects),
        TaskKind::Use {
            object,
            interaction,
        } => {
            let Some(obj) = objects.get(object as usize) else {
                return;
            };
            // (Not one it doesn't have: an id from before something was sold or moved.)
            let Some(inter) = content.objects[obj.def].interactions.get(interaction) else {
                return;
            };
            // An order to close it on someone who got in (or set off for it) after it was given:
            // the player is told.
            if task.directed && obj.closes_on_someone(inter, sim.id) {
                sim.news.get_or_insert(News::NotClosed(object));
                return;
            }
            if !obj.can_use_for(sim.id, &content.objects[obj.def], interaction)
                || obj.broken()
                || upper_barred(content, obj, sim.id, sim.age)
            {
                return;
            }
            // A game for two: both its places free, or joining whoever they play it with.
            let along = |u: u32| {
                ctx.briefs
                    .get(u as usize)
                    .and_then(|b| b.game)
                    .is_some_and(|(o, i, _)| (o, i) == (object, interaction))
            };
            if inter.partner
                && !crate::partners::may_take(
                    obj,
                    &content.objects[obj.def],
                    interaction,
                    sim.id,
                    joins,
                    along,
                )
            {
                return;
            }
            // Never other people's things; what the player asks a guest to do, only what guests
            // may (washing hands at the host's sink after the toilet is fine).
            let plot = ctx.object_plot.get(object as usize).copied().flatten();
            let allowed = if task.directed {
                order_access(ctx.households, ctx.plots, sim, plot)
            } else {
                access(ctx, sim, plot)
            };
            if allowed == Access::None
                || (allowed == Access::Guest
                    && task.directed
                    && !guest_may(content, &content.objects[obj.def], inter))
            {
                return;
            }
            if inter.needs_table && ctx.index.links.table_of(object).is_none() {
                return;
            }
            if !obj.allows(inter, sim.id) {
                return;
            }
            // Cleaning the floor with it (`sweep.rs`): to the dirtiest floor at home it cleans
            // (any dirt at all, when the player asked); with none, not at all.
            let swept = match &inter.cleans {
                Some(sweep) => {
                    let threshold = if task.directed {
                        SWEEP_ORDERED
                    } else {
                        content.room_rules.clean.threshold
                    };
                    let Some(at) = sweep_spot(sim, ctx, sweep, threshold) else {
                        return;
                    };
                    Some((at.x, at.z))
                }
                None => None,
            };
            // What it needs fetched first (`Interaction::fetch`: a coffee poured at the machine):
            // off to get it, then back to this (the follow-up); with no way to it, without.
            if fetched != Some(object)
                && let Some((from, k)) = fetch_source(sim, ctx, objects, object, inter)
            {
                start_task(
                    sim,
                    Task {
                        kind: TaskKind::Use {
                            object: from,
                            interaction: k,
                        },
                        ..task
                    },
                    ctx,
                    objects,
                    items,
                );
                sim.fetched = Some(object);
                if sim.current.is_some() {
                    sim.follow_up = Some(Task {
                        routine: false,
                        ..task
                    });
                    return;
                }
                return start_task(sim, task, ctx, objects, items);
            }
            // Sitting to use a thing on a top: on a seat pulled up to the top, if one is free
            // (else on one near, `seatNear`, else they stand at it).
            // (Standing too, when the top has chairs all round: they sit down at it.)
            let on_top = content.objects[obj.def].layer == Layer::Surface;
            let boxed_in = || {
                ctx.index.links.host_of(object).is_some_and(|h| {
                    crate::surface::rider_tiles(content, objects, nav, obj, h).is_empty()
                })
            };
            if on_top && (inter.pose == Pose::Sit || boxed_in()) {
                reserved_seat = crate::surface::seat_at_top(
                    content,
                    objects,
                    &ctx.index.links,
                    object,
                    sim.id,
                    content.life.stage(sim.age),
                );
            }
            if inter.seat_near > 0.0 && reserved_seat.is_none() {
                reserved_seat = seat_near_object(sim, ctx, objects, object, inter);
                seat_near = reserved_seat.is_some();
            }
            // From the tile by the slot they'll get (round to its place, `places`; onto a bed
            // from the side, `seat_approach`), or to their own spot along the front of a thing
            // used standing (`stand_spot`).
            let footstool = if inter.pose == Pose::Sit {
                footstool_slots(content, objects, object)
            } else {
                0
            };
            let slot = obj.slot_for(
                sim.id,
                &content.objects[obj.def],
                interaction,
                sim.pos,
                footstool,
            );
            let spot = slot
                .filter(|_| {
                    reserved_seat.is_none()
                        && inter.pose == Pose::Stand
                        && !inter.inside
                        && !inter.carried
                        && !inter.at_slot
                        && swept.is_none()
                })
                .and_then(|k| {
                    crate::surface::stand_spot(
                        content,
                        nav,
                        obj,
                        k,
                        inter.whole || inter.leaves.is_some(),
                    )
                });
            let t = match reserved_seat {
                Some(seat) => {
                    let s = &objects[seat as usize];
                    let def = &content.objects[s.def];
                    s.pick_slot(
                        sim.id,
                        0..def.slots as usize,
                        Some((def, sim.pos)),
                        footstool_slots(content, objects, seat),
                    )
                    .and_then(|k| crate::surface::seat_approach(content, nav, s, k, sim.pos))
                    .or_else(|| ctx.index.links.approach_of(seat))
                    .unwrap_or_else(|| s.front_tile(content))
                }
                // (A baby's care at the high chair from beside it; the change at the crib's
                // changing table, in front of it; then their own spot, or round to their seat or
                // bed, or a thing on a top from the top's front.)
                None if inter.carried => crate::carry::care_spot(ctx, obj),
                None => slot
                    .filter(|_| inter.at_slot)
                    .and_then(|k| crate::carry::slot_front(ctx, obj, k))
                    .or_else(|| spot.map(|(t, ..)| t))
                    .or_else(|| {
                        slot.and_then(|k| match inter.pose {
                            Pose::Stand if !inter.inside => {
                                crate::surface::slot_approach(content, nav, obj, k)
                            }
                            _ => crate::surface::seat_approach(content, nav, obj, k, sim.pos),
                        })
                    })
                    .or_else(|| ctx.index.links.approach_of(object))
                    .unwrap_or_else(|| obj.front_tile(content)),
            };
            let t = swept.unwrap_or(t);
            sim.sweep = swept;
            let tags = inter.tags;
            (
                spot.map_or([t.0 as f32 + 0.5, t.1 as f32 + 0.5], |(_, p, _)| p),
                t,
                tags,
            )
        }
        TaskKind::MoveTo { x, z } => ([x, z], (x.floor() as i32, z.floor() as i32), 0),
        TaskKind::Clean { x, z } => (
            [x as f32 + 0.5, z as f32 + 0.5],
            (x, z),
            content.room_rules.clean.tags,
        ),
        // Accidents start on the spot (see `accident`), never from the queue.
        TaskKind::Spot { .. } => return,
        TaskKind::Repair { object } => {
            let Some(obj) = objects.get(object as usize).filter(|o| o.broken()) else {
                return;
            };
            let t = ctx
                .index
                .links
                .approach_of(object)
                .unwrap_or_else(|| obj.front_tile(content));
            (
                [t.0 as f32 + 0.5, t.1 as f32 + 0.5],
                t,
                content.object_rules.repair.tags,
            )
        }
        TaskKind::Work => {
            sim.bound_for_car = None;
            // By car (`cars.rs`): to its door. (Not when already on their bike.)
            let riding = sim.riding.is_some();
            // (Called to the work taxi standing at their halt: over to its door, `taxi::door_for`.)
            let car = if riding {
                None
            } else {
                crate::taxi::door_for(ctx, sim, &TaskKind::Work)
                    .or_else(|| crate::cars::ride_for(ctx, sim))
            };
            if let Some((car, seat, door, at)) = car {
                crate::ride::get_off(sim, objects);
                sim.bound_for_car = Some(car);
                sim.car_seat = seat as u8;
                sim.car_door = door as u8;
                goal_at(at)
            } else {
                // Nearest way out of town; with no exits, work starts on the spot.
                let Some(exit) = exit_on_foot(ctx, sim) else {
                    crate::ride::get_off(sim, objects);
                    sim.transition = Some(crate::life::Transition::Work);
                    return;
                };
                // (In an open-world town, a bike only where the way was timed by one: a ride of
                // `life::BIKE_MOST` at most; not before it's been timed, nor where it was timed on
                // foot or by car.)
                let no_bike = no_bike_out(sim);
                if riding && no_bike {
                    // On a bike of theirs from a visit or an outing where the way out isn't by
                    // one (a ride of over `life::BIKE_MOST`, or none timed): the ride cut short,
                    // and on to work as its way is timed, to their kerb for the taxi or on foot
                    // (on it to work, they rode hours out of town, Hollyshaw).
                    crate::ride::get_off(sim, objects);
                    match crate::taxi::kerb_for(ctx, sim, &TaskKind::Work) {
                        Some(kerb) => goal_at(kerb),
                        None => goal_at(exit),
                    }
                } else if riding {
                    // On their bike: on it to the way out.
                    goal_at(exit)
                } else if let Some(kerb) = crate::taxi::kerb_for(ctx, sim, &TaskKind::Work) {
                    // By the work taxi they're booked on (`taxi/`): to the kerb, to wait for it.
                    crate::ride::get_off(sim, objects);
                    goal_at(kerb)
                } else if let Some(bike) = sim.bound_for_bike.or_else(|| {
                    (!no_bike)
                        .then(|| crate::ride::bike_for(ctx, sim, objects, exit))
                        .flatten()
                }) {
                    // On a bike of theirs (`ride.rs`): over to it first.
                    crate::ride::claim(sim, objects, bike);
                    goal_at(crate::ride::spot(content, &objects[bike as usize]))
                } else {
                    goal_at(exit)
                }
            }
        }
        // (Their visit starts once there's a way there: see below.)
        // By the taxi they're booked on there (`taxi/`): to its door as it stands for them, else
        // out to their kerb to wait for it.
        TaskKind::Visit { .. } | TaskKind::GoHome
            if sim.riding.is_none()
                && let Some((car, seat, door, at)) =
                    crate::taxi::door_for(ctx, sim, &task.kind) =>
        {
            sim.bound_for_car = Some(car);
            sim.car_seat = seat as u8;
            sim.car_door = door as u8;
            goal_at(at)
        }
        TaskKind::Visit { .. } | TaskKind::GoHome
            if sim.riding.is_none()
                && let Some(kerb) = crate::taxi::kerb_for(ctx, sim, &task.kind) =>
        {
            sim.bound_for_car = None;
            goal_at(kerb)
        }
        // Beyond walking range, with the household's car at home: to its driver's door first,
        // to drive there (`cars::trip_car`).
        TaskKind::Visit { plot }
            if sim.riding.is_none()
                && let Some((car, door, at)) = crate::cars::trip_car(ctx, sim, plot) =>
        {
            sim.bound_for_car = Some(car);
            sim.car_seat = 0;
            sim.car_door = door as u8;
            (
                at,
                (at[0].floor() as i32, at[1].floor() as i32),
                content.visits.tags,
            )
        }
        // Or to a seat in it, the household's car waiting in the driveway to drive near there
        // (`cars::trip_ride`).
        TaskKind::Visit { plot }
            if sim.riding.is_none()
                && let Some((car, seat, door, at)) = crate::cars::trip_ride(ctx, sim, plot) =>
        {
            sim.bound_for_car = Some(car);
            sim.car_seat = seat as u8;
            sim.car_door = door as u8;
            (
                at,
                (at[0].floor() as i32, at[1].floor() as i32),
                content.visits.tags,
            )
        }
        // On their bike there: to where they'll leave it by the place (`ride::park_spot`), and on
        // to it on foot.
        TaskKind::Visit { plot }
            if sim.riding.is_some()
                && let Some((centre, yaw, stand)) =
                    crate::ride::park_spot(ctx, sim, objects, plot) =>
        {
            sim.bike_spot = Some((centre, yaw));
            (
                stand,
                (stand[0].floor() as i32, stand[1].floor() as i32),
                content.visits.tags,
            )
        }
        // Else beyond a short walk, on a bike of theirs (`ride.rs`, `leisure_ride`): over to it
        // first, to ride there.
        TaskKind::Visit { plot }
            if sim.riding.is_none()
                && sim.bike_parked.is_none()
                && let Some(bike) = sim
                    .bound_for_bike
                    .or_else(|| leisure_ride(sim, ctx, objects, plot)) =>
        {
            sim.bound_for_car = None;
            crate::ride::claim(sim, objects, bike);
            let at = crate::ride::spot(content, &objects[bike as usize]);
            (
                at,
                (at[0].floor() as i32, at[1].floor() as i32),
                content.visits.tags,
            )
        }
        // Else from home, the taxi where it beats walking (`leisure_way`, `taxi::leisure_eta`):
        // booked on one, they wait at home till it's near (`taxi::book_leisure`).
        TaskKind::Visit { plot }
            if sim.riding.is_none() && sim.taxi.is_none() && taxi_from_home(sim, ctx, plot) =>
        {
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Taxi {
                plot,
                directed: task.directed,
            });
            return;
        }
        TaskKind::Visit { plot } => {
            // (On foot: not to a car, whatever they were heading for before.)
            sim.bound_for_car = None;
            // At a house with door rules, up to its front door to knock (`life::knock_spot`).
            let p = &ctx.plots[plot as usize];
            let knock = (content.visits.door.is_some() && !p.public)
                .then(|| crate::life::knock_spot(nav, p))
                .flatten();
            // (At a flat, its own door off the landing: `Plot::doorstep`.)
            let mut door = p.doorstep();
            // A public place without an entry: the open ground nearest its middle, not whatever
            // stands there.
            if p.public && p.entry.is_none() {
                door = open_ground_near(nav, p, door).unwrap_or(door);
            }
            match knock {
                Some((t, at, _)) => (at, t, content.visits.tags),
                None => (
                    door,
                    (door[0].floor() as i32, door[1].floor() as i32),
                    content.visits.tags,
                ),
            }
        }
        // Their car parked out on a trip: back to it, to drive home (`cars::car_out_for`).
        TaskKind::GoHome if let Some((car, door, at)) = crate::cars::car_out_for(ctx, sim) => {
            sim.bound_for_car = Some(car);
            sim.car_seat = 0;
            sim.car_door = door as u8;
            goal_at(at)
        }
        // The car they came in on a trip, parked out there: back to it, to ride home
        // (`cars::ride_home_for`).
        TaskKind::GoHome
            if let Some((car, seat, door, at)) = crate::cars::ride_home_for(ctx, sim) =>
        {
            sim.bound_for_car = Some(car);
            sim.car_seat = seat as u8;
            sim.car_door = door as u8;
            goal_at(at)
        }
        TaskKind::GoHome => {
            // On their bike: back to its place (`ride.rs`); left out by the place they're at:
            // over to it first, to ride home (`Sim::bike_parked`).
            if let Some(bike) = sim.riding.and_then(|b| objects.get(b as usize)) {
                goal_at(crate::ride::spot(content, bike))
            } else if let (Some(_), Some(at)) = (sim.bound_for_bike, sim.bike_parked) {
                goal_at(at)
            } else {
                // (Into a flat, behind its own door: `Plot::home_point`.)
                let home = ctx.households[sim.household as usize].plot;
                let Some(door) = home.map(|p| ctx.plots[p as usize].home_point()) else {
                    sim.transition = Some(crate::life::Transition::Home);
                    return;
                };
                goal_at(door)
            }
        }
        TaskKind::Answer { guest } => {
            let Some(other) = ctx.briefs.get(guest as usize) else {
                return;
            };
            let Some(goal) = conversation::approach_tile(nav, other.pos, sim.pos) else {
                return;
            };
            ([goal.0 as f32 + 0.5, goal.1 as f32 + 0.5], goal, 0)
        }
        TaskKind::Social { target, social } => {
            let Some(other) = ctx.briefs.get(target as usize) else {
                return;
            };
            if target == sim.id {
                return;
            }
            sim.social_waited = 0;
            // Within earshot: they call out to them, and the two meet halfway (see
            // `conversation::update`); otherwise over to them until they are (or right there).
            sim.called = other.available
                && conversation::calls_out(nav, &content.socials[social], sim.pos, other.pos);
            if sim.called {
                sim.pose = Pose::Stand;
                let tags = content.socials[social].tags;
                sim.current = Some(Activity {
                    task,
                    phase: Phase::Calling { ticks: 0 },
                    tags,
                });
                return;
            }
            let Some(goal) = conversation::approach_tile(nav, other.pos, sim.pos) else {
                return;
            };
            (
                [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5],
                goal,
                content.socials[social].tags,
            )
        }
        // Over to the group (`conversation::update` finds them a place in it).
        TaskKind::Join { leader } => {
            let Some(other) = ctx.briefs.get(leader as usize) else {
                return;
            };
            let Some((_, social)) = other
                .conversing
                .filter(|c| c.0 == leader && leader != sim.id)
            else {
                return;
            };
            let Some(goal) = conversation::approach_tile(nav, other.pos, sim.pos) else {
                return;
            };
            (
                [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5],
                goal,
                content.socials[social].tags,
            )
        }
    };
    // A place on the seat for the use (two housemates don't both sit on the same sofa).
    if let Some(seat) = reserved_seat.take() {
        let def = &content.objects[objects[seat as usize].def];
        let footstool = footstool_slots(content, objects, seat);
        if objects[seat as usize].reserve_seat(sim.id, def, sim.pos, footstool) {
            sim.use_seat = Some(seat);
        }
    }
    // No way to the tile by their slot (beside a lounger, shut in by the beds round it), nor to
    // the seat near it they'd take: from another tile it's used from, standing, as buying left a
    // way to one (`keeps_reach`).
    let mut waypoints = walk_route(ctx, sim, goal, target);
    if waypoints.is_none()
        && let TaskKind::Use { object, .. } = task.kind
        && (sim.use_seat.is_none() || seat_near)
    {
        if let Some(seat) = sim.use_seat.take() {
            objects[seat as usize].release(sim.id);
        }
        let tiles = crate::surface::use_tiles(content, objects, nav, &objects[object as usize]);
        waypoints = tiles
            .into_iter()
            .filter(|&t| t != goal)
            .find_map(|t| walk_route(ctx, sim, t, [t.0 as f32 + 0.5, t.1 as f32 + 0.5]));
    }
    let Some(waypoints) = waypoints else {
        if let Some(seat) = sim.use_seat.take() {
            objects[seat as usize].release(sim.id);
        }
        return;
    };
    if let TaskKind::Visit { plot } = task.kind {
        sim.transition = Some(crate::life::Transition::Visit {
            plot,
            directed: task.directed,
        });
    }
    // A place at it for the repair (two housemates don't both set off for it).
    if let TaskKind::Repair { object } = task.kind {
        let obj = &mut objects[object as usize];
        if !obj.reserve(sim.id, content.objects[obj.def].slots) {
            return;
        }
    }
    if let TaskKind::Use {
        object,
        interaction,
    } = task.kind
    {
        let sit = content.objects[objects[object as usize].def].interactions[interaction].pose
            == Pose::Sit;
        let footstool = if sit {
            footstool_slots(content, objects, object)
        } else {
            0
        };
        let obj = &mut objects[object as usize];
        if !obj.reserve_for(
            sim.id,
            &content.objects[obj.def],
            interaction,
            sim.pos,
            footstool,
        ) {
            if let Some(seat) = sim.use_seat.take() {
                objects[seat as usize].release(sim.id);
            }
            return;
        }
        // Off to bed (they can get there): the toilet first, out of habit, and then they
        // choose bed again.
        let sleep = content.objects[obj.def].interactions[interaction].tags
            & content.day_rhythm.sleep_tags
            != 0;
        if sleep
            && !task.directed
            && let Some(first) = habit(
                sim,
                ctx,
                objects,
                HabitTime::Bed,
                ctx.object_plot[object as usize],
            )
        {
            objects[object as usize].release(sim.id);
            if let Some(seat) = sim.use_seat.take() {
                objects[seat as usize].release(sim.id);
            }
            return start_task(sim, first, ctx, objects, items);
        }
    }
    // Off to a game for two: someone to play with is found after the step (`partners::update`),
    // unless they're the one fetched.
    if let TaskKind::Use {
        object,
        interaction,
    } = task.kind
        && content.objects[objects[object as usize].def].interactions[interaction].partner
    {
        sim.wants_partner = joins.is_none();
        sim.partner_wait = 0;
    }
    sim.serve_wait = 0;
    sim.owes = 0;
    sim.pose = Pose::Stand;
    sim.current = Some(Activity {
        task,
        phase: Phase::Routing { waypoints, next: 0 },
        tags,
    });
}
