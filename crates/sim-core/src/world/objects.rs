//! Using an object, arriving at it, and what follows (leaving, needs, habits).
//!
//! Split from `world.rs` (see `mod.rs`).

use super::*;

/// One tick of using an object (`minutes` long: more when catching up on a turn, see
/// `lod.rs`), with needs `pressing` (`urgent_needs`). Returns whether the Sim is done.
pub(crate) fn use_object(
    sim: &mut Sim,
    ctx: &Ctx,
    obj: &ObjectInstance,
    interaction: usize,
    elapsed: f32,
    minutes: f32,
    pressing: u32,
) -> bool {
    let content = ctx.content;
    let inter = &content.objects[obj.def].interactions[interaction];
    // Helping oneself where nobody serves is quicker than being served (`minutes_where`).
    let total = inter.minutes_where(ctx.served(obj.id));
    // Quality and skill scale what the Sim gets out of it (not what it costs).
    let boost = quality_boost(content, inter, obj.quality)
        * inter.skill_factor(&content.skill_rules, &sim.skills);
    let rhythm = &content.day_rhythm;
    // Napping: no other nap until a while after this one.
    if inter.tags & rhythm.nap_tags != 0 {
        sim.nap_until = ctx.tick + social::minutes_to_ticks(rhythm.nap_cooldown_hours * 60.0);
    }
    if inter.cares()
        && let Some(baby) = obj.users().find(|&u| u != sim.id)
    {
        let per_tick = inter.care.map(|g| g / total * minutes);
        sim.care_out = Some((baby, per_tick));
    }
    let (mut any_gain, mut satisfied, mut urgent, mut wakes) = (false, true, false, false);
    let (mut fills, mut emptying) = (0u32, false);
    let low = content.autonomy.urgent_below;
    for n in 0..content.needs.len() {
        let g = inter.gain_per_minute[n];
        if g > 0.0 {
            any_gain = true;
            fills |= 1 << n;
            let g = g * sim.mods.need_gain[n] * boost;
            sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
            satisfied &= sim.needs[n] >= 0.999;
        } else {
            if g < 0.0 {
                // Costs (a workout makes you hungry and sweaty).
                sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
                emptying |= sim.needs[n] < low && !content.needs[n].room;
            }
            // (Surroundings are never urgent: they follow the room, nothing fills them. Nor is
            // what it's for: a snack fetched to eat sitting down fills hunger only then.)
            urgent |= sim.needs[n] < low && !content.needs[n].room && inter.total_gain[n] <= 0.0;
            wakes |= sim.needs[n] < low && content.day_rhythm.wake_for & (1 << n) != 0;
        }
    }
    for (s, &gain) in inter.skill_gain[..content.skills.len()].iter().enumerate() {
        if gain > 0.0 {
            practise(sim, content, s, gain * boost, minutes);
        }
    }
    let sleep = inter.tags & rhythm.sleep_tags != 0;
    let done = if inter.baby_bed() {
        // A baby lies in the crib until they grow out of it (`lifecycle.rs`), asleep or awake:
        // needing something doesn't get them up (they can't: they cry, and a carer comes), nor
        // does being rested.
        false
    } else if inter.baby {
        // Left in the high chair (`carry.rs`): back in the crib once its time is up.
        elapsed >= inter.minutes
    } else if sleep && crate::planner::planned_now(content, sim, inter) {
        // A planned night's sleep lasts the block unless a need that wakes people gets urgent.
        wakes
    } else if sleep {
        // One sleep until rested (only the need it's for counts: about 8 hours from tired,
        // less in a better bed), but nobody gets up in the middle of the night: once rested,
        // they lie in until a little before they'd get up anyway. On a work or school day the
        // alarm goes off in time to get ready, rested or not.
        let rested = rhythm.rest_need.is_none_or(|n| sim.needs[n] >= 0.999);
        let longest = f32::from(content.planner.max_sleep_minutes);
        wakes || elapsed >= longest || (rested && !before_dawn(sim, ctx)) || getting_up(sim, ctx)
    } else {
        // Workouts and other draining activities stop before they empty the needs they cost.
        // Planned activities go on for their time even once the needs they fill are full.
        // Whatever they chose themselves (not a routine, nor for one of them) stops for a need
        // about to run out that home has something for, if it would before the end (off to the
        // toilet halfway through a film, not off the toilet for a bite to eat: a use that `lasts`
        // isn't left for another need, or worn out with the bladder low they'd be up from it
        // for bed and from bed for it, round and round, until they fell asleep on the floor).
        let planned = crate::planner::planned_now(content, sim, inter);
        let chosen = sim
            .current
            .as_ref()
            .is_some_and(|a| !a.task.directed && !a.task.routine);
        let left = total - elapsed;
        // Time to head home from where they're visiting (an outing's end, `visits.outingHomeBy`):
        // what they chose themselves there ends there and then (up off the park bench), so
        // they're home by the hour.
        let heading_home = sim.visiting.is_some_and(|v| v.arrived > 0)
            && sim.queue.iter().any(|t| matches!(t.kind, TaskKind::GoHome));
        let drained = emptying
            || (urgent && planned)
            || (chosen && heading_home)
            || (chosen
                && !inter.lasts
                && breaks_off(sim, content, pressing, fills, left, inter, low));
        let routine = sim.current.as_ref().is_some_and(|a| a.task.routine);
        // Practising a skill they still want to (as when choosing: at home or in a public place)
        // goes on for its time too, until practice gets boring; stopping it as soon as the needs
        // are full would have them back at it a moment later, over and over.
        let practising = satisfied
            && inter.trains_skills()
            && access(ctx, sim, ctx.object_plot[obj.id as usize]) == Access::Full
            && training_interest(sim, content, inter) > MIN_AUTONOMY_SCORE;
        // A job that changes the thing as the hands go (hanging out the washing, a harvest) goes on to
        // its end, clothes are changed all the way, whatever it does for comfort, and a use that
        // `lasts` (the toilet) takes its time however soon the bladder is empty, once begun (one
        // with nothing to fill at all is given up on its first tick) and while it isn't time to
        // head home (an outing's end, `visits.outingHomeBy`); so does
        // loading a machine or emptying it, or anything that leaves a load to work on next.
        // So does a game for two: one stopping would end it for the other.
        let job = inter.gradual
            || inter.picks
            || inter.changes_outfit
            || (inter.lasts && elapsed > minutes && !heading_home)
            || inter.partner
            || inter.loads
            || inter.unloads
            || inter.makes.is_some();
        elapsed >= total
            || (any_gain && satisfied && !planned && !routine && !practising && !job)
            || drained
    };
    // Halfway through changing: in the next outfit, which views draw (a change of looks).
    let half = inter.minutes * 0.5;
    if inter.changes_outfit && elapsed >= half && elapsed - minutes < half {
        sim.outfit = (sim.outfit + 1) % OUTFITS;
        sim.reshaped = true;
    }
    // Homework done: better at school (by how much of it they did).
    if done && inter.homework > 0.0 && content.life.stage_at(sim.age).is_some_and(|s| s.school) {
        let share = (elapsed / inter.minutes.max(1.0)).min(1.0);
        sim.grade = (sim.grade + inter.homework * share).clamp(0.0, 100.0);
    }
    // A finished cook (or wash, or snack) leaves a mess around the object.
    if done && inter.dirt > 0.0 {
        let (fx, fz) = ctx
            .index
            .links
            .approach_of(obj.id)
            .unwrap_or_else(|| obj.front_tile(content));
        sim.mess = Some((fx, fz, inter.dirt));
    }
    if done && let Some(m) = inter.earns_feeling_over(total, elapsed, &sim.skills) {
        social::add_feeling(&mut sim.feelings, m, &content.feelings, ctx.tick);
    }
    done
}

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
        let cost = inter.cost;
        let funds = ctx.households[sim.household as usize].funds - sim.pending_spend;
        if !affordable(cost, funds) {
            end_activity(sim, content, objects);
            return;
        }
        sim.pending_spend += cost.max(0);
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
            // At their kerb: they wait for the work taxi (`taxi.rs`).
            None if at_kerb => {
                sim.pose = Pose::Stand;
                act.phase = Phase::AtKerb { ticks: 0 };
            }
            None => {
                sim.current = None;
                sim.transition = Some(crate::life::Transition::Work);
            }
        },
        // At the door of the taxi standing for them (`taxi.rs`): facing it, they open it and get
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
            // At a house (not a park) with door rules: knock and wait (`life::doors`).
            if content.visits.door.is_some() && !ctx.plots[plot as usize].public {
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
                // slot's place does); they step back off where they came from (see `end_activity`).
                let slot = obj.slot_of(sim.id).unwrap_or(0);
                sim.seated_from = Some((sim.pos[0].floor() as i32, sim.pos[1].floor() as i32));
                sim.pos = obj.slot_position(content, slot);
                sim.yaw = obj.slot_yaw(content, slot);
                sim.pose = inter.pose;
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

/// A use done that changed the object as the hands went (`Interaction::gradual`, a crop's
/// `picks`): now it is so. The washing is out on the line (or in); the crop is picked, and lies
/// fallow, then grows again; a machine loaded runs, one emptied is empty (`machines::use_done`).
pub(crate) fn leaves_at_end(
    sim: &mut Sim,
    content: &Content,
    o: &mut ObjectInstance,
    interaction: usize,
) {
    let Some(inter) = content.objects[o.def].interactions.get(interaction) else {
        return;
    };
    if let Some(open) = inter.leaves.filter(|_| inter.gradual)
        && o.open != open
    {
        o.open = open;
        sim.reshaped = true;
    }
    if inter.picks && o.growth >= 0.0 {
        o.growth = -1.0;
        sim.reshaped = true;
    }
    crate::machines::use_done(sim, content, o, inter);
}

/// Just before they leave for work or school (`DayRhythm::ready_minutes`): what they do out of
/// habit first (`HabitTime::Leave`), dropping what they chose themselves for it (not what
/// they were told to do, a routine, cooking or eating, or something for that habit anyway).
pub(crate) fn ready_to_leave(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> Option<Task> {
    let content = ctx.content;
    let rhythm = &content.day_rhythm;
    let low = rhythm
        .habits
        .iter()
        .filter(|h| h.at == HabitTime::Leave && sim.needs[h.need] < h.below)
        .fold(0u32, |m, h| m | 1 << h.need);
    let free_will = ctx.autonomy && ctx.households[sim.household as usize].free_will;
    if low == 0
        || !free_will
        || sim.follow_up.is_some()
        || sim.queue.front().is_some_and(|t| t.directed)
        || sim.asleep(content)
    {
        return None;
    }
    let leave = leave_today(sim, ctx)?;
    if leave > ctx.tick + social::minutes_to_ticks(rhythm.ready_minutes) {
        return None;
    }
    match sim.current.as_ref().map(|a| a.task) {
        None => {}
        Some(Task {
            kind: TaskKind::Use {
                object,
                interaction,
            },
            directed: false,
            routine: false,
        }) => {
            let inter = &content.objects[objects[object as usize].def].interactions[interaction];
            if inter.fills() & low != 0 {
                return None;
            }
        }
        Some(_) => return None,
    }
    habit(
        sim,
        ctx,
        objects,
        HabitTime::Leave,
        ctx.briefs[sim.id as usize].plot,
    )
}

/// Whether their home has something working that fills need `n` (a toilet, a bed, a fridge),
/// busy or not.
pub(crate) fn home_fills(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], n: usize) -> bool {
    home_needs(sim, ctx, objects, 1 << n, false) != 0
}

/// Which of the needs in `mask` (bit per need) their home has something working for (not
/// counting what's only done with a baby there, nor a recipe the home lacks a station for: a
/// stove is no food without a fridge). `free`: only what they could go and use now, with a
/// place on it free (or theirs already), and not something they couldn't get to a moment ago
/// (every bed taken is no bed for them tonight).
pub(crate) fn home_needs(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    mask: u32,
    free: bool,
) -> u32 {
    let Some(home) = ctx.households[sim.household as usize].plot else {
        return 0;
    };
    let usable = |o: &ObjectInstance| {
        !free
            || ((o.has_free_slot(ctx.content.objects[o.def].slots) || o.slot_of(sim.id).is_some())
                && !sim.couldnt_reach(o.id, ctx.tick))
    };
    // (Busy or not: a cook waits for a station.)
    let stations = crate::cooking::home_stations(ctx, sim.household, objects);
    let possible = |r: usize| ctx.content.recipes[r].possible(stations);
    let mut found = 0;
    for o in ctx
        .index
        .on(home)
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| !o.broken() && usable(o))
    {
        let def = &ctx.content.objects[o.def];
        let startable = |&(n, _): &(usize, &crate::content::Interaction)| {
            !free || o.can_use_for(sim.id, def, n)
        };
        for (_, i) in def
            .interactions
            .iter()
            .enumerate()
            .filter(startable)
            .filter(|(_, i)| {
                i.autonomous
                    && !i.cares()
                    && i.recipe.is_none_or(possible)
                    && unfit(ctx.content, sim.age, i).is_none()
                    && requirements_met(ctx, sim, objects, o.id, i)
            })
        {
            found |= i.fills() & mask;
        }
        if found == mask {
            break;
        }
    }
    found
}

/// Essentials about to run out (`AutonomyRules::urgent`) that something at home they could use
/// now fills (bit per need): residents see to them before anything else.
pub(crate) fn urgent_needs(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> u32 {
    let content = ctx.content;
    let rules = &content.autonomy;
    let low = (0..content.needs.len())
        .filter(|&n| {
            !content.needs[n].room
                && rules.urgent & (1 << n) != 0
                && sim.needs[n] < rules.urgent_below
        })
        .fold(0, |m, n| m | 1 << n);
    if low == 0 {
        0
    } else {
        home_needs(sim, ctx, objects, low, true)
    }
}

/// Whether something they chose themselves, `inter`, with `minutes` to go, stops for one of the
/// `pressing` needs: one it doesn't fill (`fills`, bit per need) that would run out before then.
pub(crate) fn breaks_off(
    sim: &Sim,
    content: &Content,
    pressing: u32,
    fills: u32,
    minutes: f32,
    inter: &crate::content::Interaction,
    low: f32,
) -> bool {
    pressing != 0
        && pressing & fills == 0
        && runs_out(
            sim,
            content,
            pressing,
            minutes,
            false,
            &inter.gain_per_minute,
            &[0.0; MAX_NEEDS],
            low * 0.5,
        )
}

/// Whether one of the needs in `mask` would run down below `floor` in `minutes` of something
/// (asleep or not) that costs `costs` a minute (negative gains), and the walk `after` it to
/// what fills each.
#[allow(clippy::too_many_arguments)]
pub(crate) fn runs_out(
    sim: &Sim,
    content: &Content,
    mask: u32,
    minutes: f32,
    asleep: bool,
    costs: &[f32; MAX_NEEDS],
    after: &[f32; MAX_NEEDS],
    floor: f32,
) -> bool {
    let rhythm = &content.day_rhythm;
    (0..content.needs.len())
        .filter(|&n| mask & (1 << n) != 0)
        .any(|n| {
            let factor = if asleep { rhythm.asleep_decay[n] } else { 1.0 };
            let per_minute = content.needs[n].decay_per_minute * sim.mods.need_decay[n];
            let used = (per_minute * factor - costs[n].min(0.0)) * minutes + per_minute * after[n];
            sim.needs[n] - used < floor
        })
}

/// How many minutes' walk the nearest thing for each need is (one they may use, not just for
/// babies): where they'd go for it after whatever they're about to do.
pub(crate) fn reach(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    usable: &[u32],
    per_minute: f32,
) -> [f32; MAX_NEEDS] {
    let content = ctx.content;
    let mut nearest = [f32::INFINITY; MAX_NEEDS];
    for o in usable
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| !o.broken())
    {
        if access(ctx, sim, ctx.object_plot[o.id as usize]) == Access::None {
            continue;
        }
        let fills = content.objects[o.def]
            .interactions
            .iter()
            .filter(|i| {
                i.autonomous
                    && !i.baby
                    && !i.cares()
                    && requirements_met(ctx, sim, objects, o.id, i)
                    && unfit(content, sim.age, i).is_none()
            })
            .fold(0, |m, i| m | i.fills());
        if fills == 0 {
            continue;
        }
        let (fx, fz) = ctx
            .index
            .links
            .approach_of(o.id)
            .unwrap_or_else(|| o.front_tile(content));
        let minutes = ctx
            .nav
            .stairs
            .distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos)
            / per_minute;
        for n in (0..content.needs.len()).filter(|&n| fills & (1 << n) != 0) {
            nearest[n] = nearest[n].min(minutes);
        }
    }
    nearest.map(|m| if m.is_finite() { m } else { 0.0 })
}

/// Minutes before `sim` must leave for work or school today, for a visit or an outing to fit
/// (`leaves_for_work`). Infinite with none today.
pub(crate) fn free_before_work(sim: &Sim, ctx: &Ctx) -> f32 {
    leaves_for_work(sim, ctx).map_or(f32::INFINITY, |leave| {
        leave.saturating_sub(ctx.tick) as f32 * MINUTES_PER_TICK
    })
}

/// When they leave for work or school today (if they haven't yet).
pub(crate) fn leave_today(sim: &Sim, ctx: &Ctx) -> Option<u64> {
    leave_at(sim, ctx.content, ctx.tick)
}

/// When `sim` (the one being updated, or one of `ctx.others`) leaves home for work or school
/// today at the latest, as things stand: the one answer for a visit's guest (`free_before_work`)
/// and its hosts (`hosts_about_till`), as `life`'s leaving has it. On the work taxi they're booked
/// on, at its time (`taxi::booked_leave`, as late as it runs), sooner from further in than the
/// run was timed with (`taxi::indoor_lead`). Else as their way is timed on foot
/// (`life::leave_for`), or by the drive's time if that's sooner (`cars::lead_ticks`: a driver
/// sets off by it, later than on foot as a rule, so never before this). Or now, that time gone by
/// with work still to start (off any moment: a host past her walk's time drove off to her night
/// shift an hour on). None: nothing still to go to today (none, done, or a day stayed home).
pub(crate) fn leaves_for_work(sim: &Sim, ctx: &Ctx) -> Option<u64> {
    let tick = ctx.tick;
    if let Some(leave) = sim
        .taxi
        .and_then(|b| crate::taxi::booked_leave(ctx.taxis, ctx.cars, tick, b))
    {
        let indoors =
            crate::taxi::indoor_lead(ctx.content, ctx.nav.stairs, ctx.households, ctx.plots, sim);
        return Some(leave.saturating_sub(indoors).max(tick));
    }
    let day = crate::clock::day(tick);
    let (leave, start, _) = match &sim.job {
        Some(job) if job.last_shift_day != day => {
            crate::life::shift_today(ctx.content, job, &sim.skills, tick)?
        }
        Some(_) => return None,
        None if sim.school_day != day => crate::life::school_today(ctx.content, sim, tick)?,
        None => return None,
    };
    if start <= tick {
        return None;
    }
    let walk = crate::life::leave_for(sim, leave, start);
    let leave = match crate::cars::lead_ticks_in(ctx, sim, start) {
        0 => walk,
        lead => walk.min(start.saturating_sub(lead)),
    };
    Some(leave.max(tick))
}

/// When they leave for work or school later today (`leave_today`).
pub(crate) fn leave_at(sim: &Sim, content: &Content, tick: u64) -> Option<u64> {
    sim.job
        .as_ref()
        .and_then(|job| crate::life::shift_today(content, job, &sim.skills, tick))
        .or_else(|| crate::life::school_today(content, sim, tick))
        .map(|(leave, start, _)| crate::life::leave_for(sim, leave, start))
        .filter(|&leave| leave > tick)
}

/// What they do out of habit at a turning point of their day (`DayRhythm::habits`): for the
/// habit for `at` whose need is lowest (for it), the nearest quick thing on `plot` that fills
/// it. None: nothing to see to, or nothing there for it.
pub(crate) fn habit(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    at: HabitTime,
    plot: Option<u32>,
) -> Option<Task> {
    let content = ctx.content;
    let funds = ctx.households[sim.household as usize].funds - sim.pending_spend;
    // (The lowest first, for its habit.)
    let mut due: Vec<&crate::content::Habit> = content
        .day_rhythm
        .habits
        .iter()
        .filter(|h| h.at == at && sim.needs[h.need] < h.below)
        .collect();
    due.sort_by(|a, b| (sim.needs[a.need] / a.below).total_cmp(&(sim.needs[b.need] / b.below)));
    due.into_iter().find_map(|h| {
        nearest(sim, ctx, objects, plot, None, |i| {
            i.autonomous
                && i.total_gain[h.need] > 0.0
                && i.minutes <= content.autonomy.quick_minutes
                && !i.baby
                && !i.needs_table
                && i.recipe.is_none()
                && affordable(i.cost, funds)
        })
    })
}

/// Whether what they're doing right now fills need `n` (on the toilet, in bed, eating). A meal
/// on its way counts all along: while it cooks, while they fetch it and sit down.
pub(crate) fn filling(
    sim: &Sim,
    content: &Content,
    objects: &[ObjectInstance],
    items: &crate::items::Items,
    n: usize,
) -> bool {
    let Some(act) = sim.current.as_ref() else {
        return false;
    };
    let using = matches!(act.phase, Phase::Using { .. });
    match act.task.kind {
        TaskKind::Use {
            object,
            interaction,
        } if using => objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .is_some_and(|i| i.total_gain[n] > 0.0),
        TaskKind::Spot { accident } if using => content.accidents[accident]
            .rest
            .as_ref()
            .is_some_and(|r| r.gain_per_minute[n] > 0.0),
        TaskKind::Cook { recipe, .. } => content.recipes[recipe].effects[n] > 0.0,
        TaskKind::Eat { .. } => {
            crate::cooking::eating(sim, items).is_some_and(|r| content.recipes[r].effects[n] > 0.0)
        }
        _ => false,
    }
}

/// Closing `o` again after sleeping or napping in it, if that opened it (`Interaction::leaves`):
/// its interaction that closes it, if `sim` may do it now. A routine, like washing hands.
pub(crate) fn closing(
    sim: &Sim,
    content: &Content,
    o: &ObjectInstance,
    object: u32,
) -> Option<Task> {
    if !o.open {
        return None;
    }
    let interactions = &content.objects[o.def].interactions;
    let interaction = interactions
        .iter()
        .position(|i| i.leaves == Some(false) && o.allows(i, sim.id))?;
    Some(Task {
        kind: TaskKind::Use {
            object,
            interaction,
        },
        directed: false,
        routine: true,
    })
}

/// What follows using `object` when its interaction has an `after` (`tags`): the nearest
/// working object on the same plot with an interaction of those tags, one they may use and can
/// get to (a guest washes their hands at the host's sink). None: there's nothing like it.
pub(crate) fn follow_up(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    object: u32,
    tags: TagMask,
) -> Option<Task> {
    let plot = ctx.object_plot.get(object as usize).copied().flatten();
    nearest(sim, ctx, objects, plot, Some(object), |i| {
        i.tags & tags != 0 && !i.baby && i.recipe.is_none()
    })
}

/// The nearest working object on `plot` (but `except`) with an interaction that `fits`, one
/// they may use and can get to, as a routine task.
pub(crate) fn nearest(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    plot: Option<u32>,
    except: Option<u32>,
    fits: impl Fn(&crate::content::Interaction) -> bool,
) -> Option<Task> {
    let content = ctx.content;
    if access(ctx, sim, plot) == Access::None {
        return None;
    }
    let usable = usable_here(sim, ctx);
    let usable: &[u32] = &usable;
    let approach = |o: &ObjectInstance| {
        ctx.index
            .links
            .approach_of(o.id)
            .unwrap_or_else(|| o.front_tile(content))
    };
    let mut near: Vec<(f32, u32, usize)> = usable
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| Some(o.id) != except && ctx.object_plot[o.id as usize] == plot)
        .filter(|o| !o.broken() && o.has_free_slot(content.objects[o.def].slots))
        .filter(|o| !(sim.visiting.is_some() && content.objects[o.def].private))
        .filter_map(|o| {
            let def = &content.objects[o.def];
            let i = (0..def.interactions.len()).find(|&i| {
                fits(&def.interactions[i])
                    && unfit(content, sim.age, &def.interactions[i]).is_none()
                    && !upper_barred(content, o, sim.id, sim.age)
                    && requirements_met(ctx, sim, objects, o.id, &def.interactions[i])
                    && o.allows(&def.interactions[i], sim.id)
                    && o.can_use_for(sim.id, def, i)
            })?;
            let (fx, fz) = approach(o);
            Some((
                ctx.nav
                    .stairs
                    .distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos),
                o.id,
                i,
            ))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (_, object, interaction) = near.into_iter().take(3).find(|&(_, o, _)| {
        let (fx, fz) = approach(&objects[o as usize]);
        route(
            &ctx.nav,
            Some(ctx.reach),
            sim.pos,
            sim.tile(),
            (fx, fz),
            [fx as f32 + 0.5, fz as f32 + 0.5],
            None,
        )
        .is_some()
    })?;
    Some(Task {
        kind: TaskKind::Use {
            object,
            interaction,
        },
        directed: false,
        routine: true,
    })
}

/// Accident `a` happens to `sim` where they are (see `Content::accidents`).
pub(crate) fn accident(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], a: usize) {
    let content = ctx.content;
    let def = &content.accidents[a];
    sim.accident_at[a] = ctx.tick + def.cooldown_ticks;
    for (n, e) in def.effects.iter().enumerate().take(content.needs.len()) {
        sim.needs[n] = (sim.needs[n] + e).clamp(0.0, 1.0);
    }
    if def.dirt > 0.0 {
        let (x, z) = sim.tile();
        sim.mess = Some((x, z, def.dirt));
    }
    if let Some(f) = def.feeling {
        social::add_feeling(&mut sim.feelings, f, &content.feelings, ctx.tick);
    }
    sim.pending_spend += def.cost;
    if def.story {
        sim.news = Some(News::Accident(a));
        crate::planner::think(sim, crate::planner::ThoughtKind::Accident, a, ctx.tick);
    }
    if let Some(rest) = &def.rest {
        // (Asleep on the way to a car: they don't go on to it, and it doesn't wait for them.)
        if sim
            .current()
            .is_some_and(|a| matches!(a.phase, Phase::Routing { .. }))
        {
            sim.bound_for_car = None;
        }
        end_activity(sim, content, objects);
        sim.queue.retain(|t| t.directed);
        sim.pose = rest.pose;
        sim.current = Some(Activity {
            task: Task {
                kind: TaskKind::Spot { accident: a },
                directed: false,
                routine: false,
            },
            phase: Phase::Using { elapsed: 0.0 },
            tags: rest.tags,
        });
    }
}

/// Stops what they're doing: the object they used (or cooked at, sat at to eat, washed up at)
/// is free again, and they stand up in front of it (or step out of the shower; climbing down from
/// the upper bunk first: `Sim::ladder_ticks`). What they carry is put down after the step
/// (`cooking::after_step`).
pub(crate) fn end_activity(sim: &mut Sim, content: &Content, objects: &mut [ObjectInstance]) {
    sim.join_spot = None;
    sim.kept_out = 0;
    // A talk from the seat over: back to what they sat down to do, still sitting.
    if let Some(paused) = sim.paused.take() {
        if matches!(
            sim.current.as_ref().map(|a| a.task.kind),
            Some(TaskKind::Social { .. } | TaskKind::Join { .. })
        ) {
            sim.current = Some(paused);
            sim.idle_ticks = 0;
            return;
        }
        // (Ended some other way: what they sat down to do ends too.)
        if let Some(o) = held_object(paused.task.kind).filter(|&o| (o as usize) < objects.len()) {
            objects[o as usize].release(sim.id);
        }
        if let Some((fx, fz)) = sim.seated_from {
            sim.pos = [fx as f32 + 0.5, fz as f32 + 0.5];
        }
    }
    // A load of washing taken up for a use that didn't get done goes back on its pile.
    sim.made |= std::mem::take(&mut sim.taken);
    let held = sim
        .current
        .take()
        .and_then(|act| held_object(act.task.kind));
    if let Some(o) = held.map(|o| o as usize).filter(|&o| o < objects.len()) {
        // Up from the upper bunk (its second slot, see `ObjectInstance::slot_position`): down its ladder first.
        if sim.pose == Pose::Lie
            && content.objects[objects[o].def].bunk
            && objects[o].users[1] == Some(sim.id)
        {
            sim.ladder_ticks = LADDER_DOWN_TICKS;
        }
        objects[o].release(sim.id);
        if sim.pose != Pose::Stand || sim.seated_from.is_some() {
            let (fx, fz) = sim
                .seated_from
                .unwrap_or_else(|| objects[o].front_tile(content));
            let to = [fx as f32 + 0.5, fz as f32 + 0.5];
            // Out of bed facing the way they get out, square to it: off its side, sitting up
            // on the edge with their back to the pillows, or over its foot.
            if sim.pose == Pose::Lie {
                let a = objects[o].angle();
                let d = (to[0] - sim.pos[0]).atan2(to[1] - sim.pos[1]) - a;
                sim.yaw = wrap_angle(a + (d / FRAC_PI_2).round() * FRAC_PI_2);
            }
            sim.pos = to;
        }
    }
    if let Some(seat) = sim.use_seat.take()
        && let Some(o) = objects.get_mut(seat as usize)
    {
        o.release(sim.id);
    }
    sim.seated_from = None;
    sim.pose = Pose::Stand;
    sim.idle_ticks = 0;
}
