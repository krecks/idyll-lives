//! Arriving at an object: paying for the use and taking a place at it.

use super::*;
use crate::TICKS_PER_MINUTE;
use crate::content::{HabitTime, Layer, Pose};
use crate::world::{
    Ctx, News, ObjectInstance, Phase, Sim, TaskKind, affordable, closed_at, conditions_met,
    missing_requirement, outing_leave_by,
};

pub(crate) fn arrive(
    sim: &mut Sim,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut crate::items::Items,
) {
    let content = ctx.content;
    // Paid on arrival; if the money ran out on the way, never mind. Nor is something used that
    // broke on the way (someone else wore it out).
    if let Some(TaskKind::Use {
        object,
        interaction,
    }) = sim.current.as_ref().map(|a| a.task.kind)
    {
        // Nor is something opened that someone opened on the way, or closed on someone in it.
        let inter = &content.objects[objects[object as usize].def].interactions[interaction];
        // (Sold at a post that closed since they set off: served still at last orders, `staff::LAST_ORDERS`.)
        let late = (ctx.hour - crate::staff::LAST_ORDERS).rem_euclid(24.0);
        let met = missing_requirement(
            content,
            ctx.object_plot,
            Some(ctx.index),
            objects,
            object,
            inter,
        )
        .is_none()
            && conditions_met(ctx, sim, &objects[object as usize], inter)
            && (!closed_at(ctx, objects, object, inter, ctx.hour)
                || !closed_at(ctx, objects, object, inter, late));
        if objects[object as usize].broken()
            || !met
            || !objects[object as usize].allows(inter, sim.id)
        {
            // (An order to close it on someone who got there first: the player is told.)
            let directed = sim.current.as_ref().is_some_and(|a| a.task.directed);
            if directed
                && !objects[object as usize].broken()
                && objects[object as usize].closes_on_someone(inter, sim.id)
            {
                sim.news.get_or_insert(News::NotClosed(object));
            }
            end_activity(sim, content, objects);
            return;
        }
        let cost = ctx.cost_of(object, inter);
        let funds = ctx.households[sim.household as usize].funds - sim.pending_spend;
        if !affordable(cost, funds) {
            end_activity(sim, content, objects);
            return;
        }
        // (At a post with someone serving, paid once served: `world::route`.)
        if inter.served.is_some() && ctx.served(object) {
            sim.owes = cost.max(0);
        } else {
            sim.owes = 0;
            sim.pending_spend += cost.max(0);
        }
        // A load of washing taken up as the use starts (`life::update` settles it); given back if
        // the use doesn't get done.
        if let Some(w) = inter.work_on {
            sim.worked |= w.bit();
            if w.load().is_some() {
                sim.taken = w.bit();
            }
        }
    }
    // (Booked on a work taxi still to come: at their kerb, they wait for it.)
    let at_kerb = sim
        .current()
        .is_some_and(|a| crate::taxi::kerb_for(ctx, sim, &a.task.kind).is_some());
    let Some(act) = sim.current.as_mut() else {
        return;
    };
    match act.task.kind {
        TaskKind::MoveTo { .. } => sim.current = None,
        TaskKind::Clean { .. } => {
            sim.pose = Pose::Stand;
            act.phase = Phase::Using { elapsed: 0.0 };
        }
        TaskKind::Spot { .. } => {}
        TaskKind::Repair { object } => {
            let c = objects[object as usize].centre(content);
            sim.yaw = (c[0] - sim.pos[0]).atan2(c[1] - sim.pos[1]);
            sim.pose = Pose::Stand;
            act.phase = Phase::Using { elapsed: 0.0 };
        }
        TaskKind::Work => match sim.bound_for_car.and_then(|c| ctx.cars.get(c as usize)) {
            // At the car's door (a work taxi's, standing for them): facing it, they open it and
            // get in.
            Some(car) if car.at_home() || car.taxi && car.standing => {
                sim.yaw = (car.spot[0] - sim.pos[0]).atan2(car.spot[1] - sim.pos[1]);
                sim.pose = Pose::Stand;
                act.phase = Phase::Boarding { ticks: 0 };
            }
            // Gone without them (it didn't wait any longer): on foot to the end of town after all.
            Some(_) => {
                sim.bound_for_car = None;
                if let Some(a) = sim.current.take() {
                    sim.queue.push_front(a.task);
                }
            }
            // At their bike: on they get, and off out of town on it.
            None if sim.bound_for_bike.is_some() => {
                crate::ride::get_on(sim);
                if let Some(a) = sim.current.take() {
                    sim.queue.push_front(a.task);
                }
            }
            // At their kerb: they wait for the work taxi (`taxi/`).
            None if at_kerb => {
                sim.pose = Pose::Stand;
                act.phase = Phase::AtKerb { ticks: 0 };
            }
            None => {
                sim.current = None;
                sim.transition = Some(crate::life::Transition::Work);
            }
        },
        // At the door of the taxi standing for them (`taxi/`): facing it, they open it and get
        // in; at their kerb, they wait for it.
        TaskKind::Visit { .. } | TaskKind::GoHome
            if sim
                .bound_for_car
                .and_then(|c| ctx.cars.get(c as usize))
                .is_some_and(|c| c.taxi && c.standing) =>
        {
            let car = &ctx.cars[sim.bound_for_car.expect("just looked") as usize];
            sim.yaw = (car.spot[0] - sim.pos[0]).atan2(car.spot[1] - sim.pos[1]);
            sim.pose = Pose::Stand;
            act.phase = Phase::Boarding { ticks: 0 };
        }
        TaskKind::Visit { .. } | TaskKind::GoHome if at_kerb => {
            sim.pose = Pose::Stand;
            act.phase = Phase::AtKerb { ticks: 0 };
        }
        // At their car's door, to drive there (`cars::start_trip`).
        TaskKind::Visit { plot }
            if sim
                .bound_for_car
                .is_some_and(|c| crate::cars::may_set_off(ctx.cars, c, sim.id)) =>
        {
            let (car, directed) = (sim.bound_for_car.expect("just looked"), act.task.directed);
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Drive {
                car,
                plot,
                directed,
            });
        }
        // At their household's car's door, to ride along there (`cars::ride`).
        TaskKind::Visit { plot }
            if sim
                .bound_for_car
                .is_some_and(|c| crate::cars::may_ride(ctx.cars, c, sim.id, false)) =>
        {
            let (car, directed) = (sim.bound_for_car.expect("just looked"), act.task.directed);
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Ride {
                car,
                plot: Some(plot),
                directed,
            });
        }
        // At a car that went without them: on foot after all.
        TaskKind::Visit { .. } if sim.bound_for_car.is_some() => {
            sim.bound_for_car = None;
            if let Some(a) = sim.current.take() {
                sim.queue.push_front(a.task);
            }
        }
        // At where they leave their bike by the place (`ride::park_spot`): off it, and on on foot.
        TaskKind::Visit { .. } if sim.riding.is_some() && sim.bike_spot.is_some() => {
            if let Some(bike) = sim.riding.take() {
                sim.bound_for_bike = Some(bike);
                sim.bike_parked = Some(sim.pos);
            }
            if let Some(a) = sim.current.take() {
                sim.queue.push_front(a.task);
            }
        }
        // At their bike at home: on they get, and off there on it (`ride.rs`).
        TaskKind::Visit { .. }
            if sim.bound_for_bike.is_some()
                && sim.riding.is_none()
                && sim.bike_parked.is_none() =>
        {
            crate::ride::get_on(sim);
            if let Some(a) = sim.current.take() {
                sim.queue.push_front(a.task);
            }
        }
        TaskKind::Visit { plot } => {
            sim.bound_for_car = None;
            // There on a bike: off it, and it's left by the place (`Sim::bike_parked`).
            if let Some(bike) = sim.riding.take() {
                sim.bound_for_bike = Some(bike);
                sim.bike_parked = Some(sim.pos);
            }
            // Let in there already (a walk across it set off again): nothing to knock for, the
            // visit goes on as it was (`life::apply_transition`).
            let there = sim
                .visiting
                .is_some_and(|v| v.plot == plot && v.arrived > 0);
            // At a house (not a park) with door rules: knock and wait (`life::doors`).
            if there {
                sim.current = None;
            } else if content.visits.door.is_some() && !ctx.plots[plot as usize].public {
                // Facing the door, at it (else towards the middle of the lot).
                let p = &ctx.plots[plot as usize];
                let centre = [p.x as f32 + p.w as f32 / 2.0, p.z as f32 + p.d as f32 / 2.0];
                sim.yaw = match crate::life::knock_spot(&ctx.nav, p) {
                    Some((_, at, yaw)) if (at[0] - sim.pos[0]).hypot(at[1] - sim.pos[1]) < 0.3 => {
                        yaw
                    }
                    _ => (centre[0] - sim.pos[0]).atan2(centre[1] - sim.pos[1]),
                };
                sim.pose = Pose::Stand;
                act.phase = Phase::Knocking {
                    ticks: 0,
                    host: None,
                };
            } else {
                // There: the visit is from now, the way there doesn't count (a longer one the
                // player asked for keeps its end); an outing ends in time to be home by
                // `visits.outingHomeBy` and before work or school (`outing_leave_by`).
                let directed = act.task.directed;
                let leave_by = (!directed)
                    .then(|| outing_leave_by(sim, ctx, objects, plot, sim.pos))
                    .flatten();
                let rules = &content.visits;
                let hours = if directed {
                    rules.directed_hours
                } else {
                    rules.hours
                };
                let until = ctx.tick + (hours * 60.0 * TICKS_PER_MINUTE as f32) as u64;
                if let Some(v) = sim.visiting.as_mut().filter(|v| v.plot == plot) {
                    v.until = if directed {
                        v.until.max(until)
                    } else {
                        leave_by.map_or(until, |t| until.min(t))
                    };
                    v.home_by = leave_by.is_some();
                    v.arrived = ctx.tick;
                }
                sim.current = None;
            }
        }
        TaskKind::Answer { guest } => {
            if let Some(other) = ctx.briefs.get(guest as usize) {
                sim.yaw = (other.pos[0] - sim.pos[0]).atan2(other.pos[1] - sim.pos[1]);
            }
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Answered { guest });
        }
        // At their car parked out on a trip: in, and home.
        TaskKind::GoHome
            if sim
                .bound_for_car
                .is_some_and(|c| crate::cars::parked_out(ctx.cars, c, sim.id)) =>
        {
            let car = sim.bound_for_car.expect("just looked");
            sim.current = None;
            sim.transition = Some(crate::life::Transition::DriveHome { car });
        }
        // At the car they came in, parked out there: in, to ride home.
        TaskKind::GoHome
            if sim
                .bound_for_car
                .is_some_and(|c| crate::cars::may_ride(ctx.cars, c, sim.id, true)) =>
        {
            let car = sim.bound_for_car.expect("just looked");
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Ride {
                car,
                plot: None,
                directed: false,
            });
        }
        // At their bike, left by the place they were at: on they get, and home on it.
        TaskKind::GoHome if sim.bike_parked.is_some() && sim.riding.is_none() => {
            crate::ride::get_on(sim);
            if let Some(a) = sim.current.take() {
                sim.queue.push_front(a.task);
            }
        }
        // At a car that went without them: home on foot after all.
        TaskKind::GoHome if sim.bound_for_car.is_some() => {
            sim.bound_for_car = None;
            if let Some(a) = sim.current.take() {
                sim.queue.push_front(a.task);
            }
        }
        TaskKind::GoHome => {
            // Back at the bike's place: off it.
            crate::ride::get_off(sim, objects);
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Home);
            // Home again: the toilet first, out of habit.
            sim.follow_up = habit(
                sim,
                ctx,
                objects,
                HabitTime::Home,
                ctx.households[sim.household as usize].plot,
            );
        }
        TaskKind::Social { .. } | TaskKind::Join { .. } => act.phase = Phase::Waiting { ticks: 0 },
        TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. } => {
            crate::cooking::arrive(sim, ctx, objects, items)
        }
        TaskKind::Phone { .. } => crate::phone::arrive(sim, ctx, objects),
        TaskKind::Carry { .. } => crate::carry::arrive(sim, ctx, objects),
        TaskKind::SetUp { .. } => crate::setup::arrive(sim),
        TaskKind::Serve { .. } => crate::hosting::arrive(sim, ctx, objects),
        TaskKind::Use {
            object,
            interaction,
        } => {
            let obj = &objects[object as usize];
            let inter = &content.objects[obj.def].interactions[interaction];
            // A thing on a top is used at its top: standing, facing it, unless they sat down at
            // a seat pulled up to the top.
            // (Facing the place they work at: a crib's changing table, not its middle.)
            let on_top = content.objects[obj.def].layer == Layer::Surface;
            let c = match obj.slot_of(sim.id).filter(|_| inter.at_slot) {
                Some(k) => obj.slot_position(content, k),
                None if on_top => crate::surface::middle(content, obj),
                None => obj.centre(content),
            };
            if let Some(seat) = sim.use_seat {
                // Watching from the sofa: sit on the seat, facing the way it faces, and step
                // back off where they came from (see `end_activity`).
                let s = &objects[seat as usize];
                let slot = s.slot_of(sim.id).unwrap_or(0);
                sim.seated_from = Some((sim.pos[0].floor() as i32, sim.pos[1].floor() as i32));
                sim.pos = s.slot_position(content, slot);
                sim.yaw = s.slot_yaw(content, slot);
                sim.pose = Pose::Sit;
            } else if inter.cleans.is_some() {
                // At the floor they took the tool to (`sweep.rs`): cleaning it where they stand,
                // facing the room.
                if let Some(yaw) = crate::sweep::facing(
                    ctx.nav.lot,
                    ctx.rooms,
                    ctx.households[sim.household as usize].plot,
                    sim.pos,
                ) {
                    sim.yaw = yaw;
                }
                sim.pose = Pose::Stand;
            } else if (inter.pose == Pose::Stand || on_top) && !inter.inside {
                // At their own spot, come to it (the end of a ping-pong table, their basin at a
                // double sink: `stand_spot`), facing the way it says; else facing the thing.
                let slot = obj.slot_of(sim.id).unwrap_or(0);
                let spot = (!inter.carried && !inter.at_slot)
                    .then(|| {
                        crate::surface::at_stand_spot(
                            content,
                            &ctx.nav,
                            obj,
                            slot,
                            inter.whole || inter.leaves.is_some(),
                            sim.pos,
                        )
                    })
                    .flatten();
                if let Some((p, yaw)) = spot {
                    sim.pos = p;
                    sim.yaw = yaw;
                } else {
                    sim.yaw = (c[0] - sim.pos[0]).atan2(c[1] - sim.pos[1]);
                }
                // (A thing on a top is used standing whatever its pose, without a seat.)
                sim.pose = Pose::Stand;
            } else {
                // Onto the seat or bed, or into the shower, facing the way it faces (or its
                // slot's place does, as the place says: standing beside a massage table); they
                // step back off where they came from (see `end_activity`).
                let slot = obj.slot_of(sim.id).unwrap_or(0);
                sim.seated_from = Some((sim.pos[0].floor() as i32, sim.pos[1].floor() as i32));
                sim.pos = obj.slot_position(content, slot);
                sim.yaw = obj.slot_yaw(content, slot);
                sim.pose = content.objects[obj.def]
                    .places
                    .get(slot)
                    .and_then(|p| p.pose)
                    .unwrap_or(inter.pose);
            }
            act.phase = Phase::Using { elapsed: 0.0 };
            // Opened or closed as the use starts (a wall bed folds down as they lie down); what
            // the hands do through it, once it's done (`leaves_at_end`).
            if let Some(open) = inter.leaves.filter(|_| !inter.gradual)
                && objects[object as usize].open != open
            {
                objects[object as usize].open = open;
                sim.reshaped = true;
            }
            // Sitting down by a fire in the dark, they light it (see `lights.rs`).
            let o = &objects[object as usize];
            if o.off
                && content.objects[o.def].fire
                && (crate::lights::dark_at(ctx.minute) || ctx.nav.stairs.storey(o.z) < 0)
            {
                objects[object as usize].off = false;
                sim.lit = true;
            }
        }
    }
}
