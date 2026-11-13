//! Who takes which car, where they sit, and how they board.
//!
//! Split from `cars.rs` (see `mod.rs`).

use super::*;

/// The car `sim` would take to work (or school) now, and their seat in it (0: they drive): an
/// owner 18 or older takes their household's car from home when it's parked there; anyone else in
/// the household rides along when an owner is about to drive off in it (or on their way to). Not
/// one booked on a work taxi (`taxi.rs`): it comes for them as planned.
pub(crate) fn seat(
    cars: &[Car],
    briefs: &[SimBrief],
    households: &[Household],
    plots: &[Plot],
    rels: &crate::social::Relationships,
    sim: &Sim,
) -> Option<(u32, usize)> {
    let me = briefs.get(sim.id as usize)?;
    let home = households.get(sim.household as usize)?.plot?;
    // (Called to the household's car on their way out to the kerb, a rider left the taxi to
    // come in for nobody, held up by that car and turned round into the next taxi's way.)
    if !sim.here() || me.baby || sim.taxi.is_some() {
        return None;
    }
    if me.plot.is_none_or(|p| p == home)
        && let Some(seat) = own_seat(cars, briefs, sim, me, home)
    {
        return Some(seat);
    }
    // A lift from a neighbour leaving now (`lift_seat`): from home, or on their way to it.
    me.plot?;
    let door = plots.get(home as usize)?.arrival_point();
    lift_seat(cars, briefs, households, rels, door, sim, me, home)
}

/// Who drives car `c` off now, other than `me`: the driver in it waiting for the others, or (it
/// still waits for one) an owner who drives it on their way to it.
pub(crate) fn setting_off_driver(
    car: &Car,
    c: u32,
    briefs: &[SimBrief],
    me: Option<u32>,
) -> Option<u32> {
    match (car.state, car.driver) {
        (CarState::Boarding, Some(d)) => Some(d).filter(|&d| Some(d) != me),
        (CarState::Parked, _) | (CarState::Boarding, None) => briefs
            .iter()
            .enumerate()
            .map(|(j, b)| (j as u32, b))
            .find(|&(j, b)| Some(j) != me && b.bound_for_car == Some(c) && car.driven_by(briefs, j))
            .map(|(j, _)| j),
        _ => None,
    }
}

/// How many ride in car `c` with `driver`, other than `me`: those in it and on their way to it.
pub(crate) fn riders(
    car: &Car,
    c: u32,
    briefs: &[SimBrief],
    driver: u32,
    me: Option<u32>,
) -> usize {
    car.aboard.iter().filter(|&&a| a != driver).count()
        + briefs
            .iter()
            .enumerate()
            .map(|(j, b)| (j as u32, b))
            .filter(|&(j, b)| Some(j) != me && j != driver && b.bound_for_car == Some(c))
            .count()
}

/// A seat in a car of `sim`'s own household (`seat`).
pub(crate) fn own_seat(
    cars: &[Car],
    briefs: &[SimBrief],
    sim: &Sim,
    me: &SimBrief,
    home: u32,
) -> Option<(u32, usize)> {
    cars.iter().enumerate().find_map(|(c, car)| {
        // At home, or still close by on the street.
        let near = me.plot == Some(home)
            || (sim.pos[0] - car.spot[0]).hypot(sim.pos[1] - car.spot[1]) < CALL_REACH;
        // (Not one setting off on a trip: `trip_ride`.)
        if car.household != sim.household
            || car.plot != home
            || car.blocked
            || car.no_way
            || !near
            || car.trip.is_some()
        {
            return None;
        }
        let c = c as u32;
        let waiting = matches!(
            (car.state, car.driver),
            (CarState::Parked, _) | (CarState::Boarding, None)
        );
        match setting_off_driver(car, c, briefs, Some(sim.id)) {
            // Nobody else to drive it: they do.
            None => (waiting && car.driven_by(briefs, sim.id)).then_some((c, 0)),
            // A seat of their own, past the driver and the riders before them.
            Some(d) => {
                let rider = riders(car, c, briefs, d, Some(sim.id)) + 1;
                (rider < SEATS).then_some((c, rider))
            }
        }
    })
}

/// Neighbours give lifts: a car of another household whose driver is setting off now (on their
/// way to it, or in it and waiting) and gives `sim` a lift (`lift_ok`; `door`: their home's), with
/// a seat free.
#[allow(clippy::too_many_arguments)]
pub(crate) fn lift_seat(
    cars: &[Car],
    briefs: &[SimBrief],
    households: &[Household],
    rels: &crate::social::Relationships,
    door: [f32; 2],
    sim: &Sim,
    me: &SimBrief,
    home: u32,
) -> Option<(u32, usize)> {
    cars.iter().enumerate().find_map(|(c, car)| {
        // (The car's own tests first: who drives it is a look through everyone.)
        if me.plot.is_some_and(|p| p != home && p != car.plot)
            || car.trip.is_some()
            || !lift_car(car, households, door, sim.household)
        {
            return None;
        }
        let c = c as u32;
        let driver = setting_off_driver(car, c, briefs, Some(sim.id))?;
        if !lift_friend(rels, sim.id, driver) {
            return None;
        }
        let rider = riders(car, c, briefs, driver, Some(sim.id)) + 1;
        (rider < SEATS).then_some((c, rider))
    })
}

/// Whether `driver` gives `i` (of `household`, whose home's door is `door`) a lift in `car`:
/// another household's, standing on its own driveway (usable) within `LIFT_REACH` m of the door,
/// and the driver a friend (`LIFT_FRIENDSHIP`).
pub(crate) fn lift_ok(
    car: &Car,
    households: &[Household],
    rels: &crate::social::Relationships,
    door: [f32; 2],
    i: u32,
    household: u32,
    driver: u32,
) -> bool {
    lift_car(car, households, door, household) && lift_friend(rels, i, driver)
}

/// `lift_ok`'s test of the car: another household's than `household`, standing on its own
/// driveway (usable), within `LIFT_REACH` m of `door`.
pub(crate) fn lift_car(
    car: &Car,
    households: &[Household],
    door: [f32; 2],
    household: u32,
) -> bool {
    car.household != household
        && households.get(car.household as usize).and_then(|h| h.plot) == Some(car.plot)
        && !car.blocked
        && !car.no_way
        && (door[0] - car.spot[0]).hypot(door[1] - car.spot[1]) <= LIFT_REACH
}

/// `lift_ok`'s test of the driver: someone else, and a friend of `i`'s (`LIFT_FRIENDSHIP`).
pub(crate) fn lift_friend(rels: &crate::social::Relationships, i: u32, driver: u32) -> bool {
    driver != i && rels.get(i as usize, driver as usize).friendship >= LIFT_FRIENDSHIP
}

/// Neighbours give a lift (`lift_ok`) from a car this close (m) to the door of someone's home...
pub(crate) const LIFT_REACH: f32 = 25.0;
/// ...to a friend this good.
pub(crate) const LIFT_FRIENDSHIP: f32 = 30.0;

/// Where `sim` heads for work (or school) when they go by car: the car, the door they get in by
/// and where they stand for it. None: on foot.
pub(crate) fn ride_for(
    ctx: &crate::world::Ctx,
    sim: &Sim,
) -> Option<(u32, usize, usize, [f32; 2])> {
    let (c, seat) = seat(
        ctx.cars,
        ctx.briefs,
        ctx.households,
        ctx.plots,
        ctx.rels,
        sim,
    )?;
    let (door, at) = ctx.cars[c as usize].door(&ctx.nav, seat)?;
    Some((c, seat, door, at))
}

/// Whether `i` can get into car `c` now (as `seat` found it on their way).
pub(crate) fn may_board(w: &World, i: usize, c: u32) -> bool {
    seat(
        &w.cars,
        &w.briefs,
        &w.households,
        &w.plots,
        &w.relationships,
        &w.sims[i],
    )
    .is_some_and(|(k, _)| k == c)
}

/// How much earlier than on foot `i` leaves for work or school starting at `start` when their
/// household's car is at home to take them: the walk to it, getting in, the drive out of town and
/// on to work. 0 without one, and for one who can't drive it unless someone who can starts about
/// when they do (so they set off together: `seat`).
pub(crate) fn lead_ticks(w: &World, i: usize, start: u64) -> u64 {
    lead_by(
        &w.content,
        &w.cars,
        &w.briefs,
        &w.households,
        &w.plots,
        &w.relationships,
        w.tick,
        &w.sims[i],
        |j| w.sims.get(j),
        start,
    )
}

/// `lead_ticks` for `s` in a resident's update (`s` that one, or another of `ctx.others`): those
/// it doesn't have, the one being updated, drive nobody.
pub(crate) fn lead_ticks_in(ctx: &crate::world::Ctx, s: &Sim, start: u64) -> u64 {
    lead_by(
        ctx.content,
        ctx.cars,
        ctx.briefs,
        ctx.households,
        ctx.plots,
        ctx.rels,
        ctx.tick,
        s,
        |j| ctx.others.get(j),
        start,
    )
}

/// `lead_ticks` for `s`, the others by index from `sim_of`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn lead_by<'a>(
    content: &crate::content::Content,
    cars: &[Car],
    briefs: &[SimBrief],
    households: &[Household],
    plots: &[Plot],
    rels: &crate::social::Relationships,
    tick: u64,
    s: &Sim,
    sim_of: impl Fn(usize) -> Option<&'a Sim>,
    start: u64,
) -> u64 {
    let me = s.id;
    let home = households.get(s.household as usize).and_then(|h| h.plot);
    let usable = |c: &&Car| c.at_home() && !c.blocked && !c.no_way;
    // An owner who drives it and starts about when they do.
    let sets_off = |car: &Car, j: u32| {
        j != me
            && car.driven_by(briefs, j)
            && sim_of(j as usize).is_some_and(|o| {
                o.here()
                    && crate::life::start_on(content, o, tick)
                        .is_some_and(|st| st.abs_diff(start) <= BOARD_WAIT_TICKS)
            })
    };
    let own = cars
        .iter()
        .filter(usable)
        .find(|c| c.household == s.household && Some(c.plot) == home)
        .filter(|car| car.driven_by(briefs, me) || car.owners.iter().any(|&j| sets_off(car, j)));
    // Else a lift from a neighbour leaving with them (`lift_seat`).
    let door = home
        .and_then(|p| plots.get(p as usize))
        .map(|p| p.arrival_point());
    let car = own.or_else(|| {
        let door = door?;
        cars.iter().filter(|c| c.at_home()).find(|c| {
            c.owners
                .iter()
                .any(|&j| lift_ok(c, households, rels, door, me, s.household, j) && sets_off(c, j))
        })
    });
    let Some(car) = car else {
        return 0;
    };
    WALK_TO_CAR_TICKS + u64::from(GET_IN_TICKS) + drive_out_ticks(car) + off_map(content)
}

/// Once in `car` for work or school: the wait for it to go and the drive out of town.
pub(crate) fn drive_out_ticks(car: &Car) -> u64 {
    BOARD_MIN_TICKS + car.out_ticks.ceil() as u64
}

/// The drive between the end of town and work: `commuteMinutes` (the part of the way off the
/// map).
pub(crate) fn off_map_ticks(w: &World) -> u64 {
    off_map(&w.content)
}

/// `off_map_ticks` for a content alone (`lead_by` has no world to hand).
pub(crate) fn off_map(content: &crate::content::Content) -> u64 {
    crate::life::ticks(content.career_rules.commute_minutes)
}

/// `i` gets in (their shift is under way): the first owner in drives (someone in before them waits
/// for them). Should they come home some other way, it's from the end of town (as they'd have
/// walked there).
pub(crate) fn board(w: &mut World, i: usize, c: u32) {
    let tick = w.tick;
    to_exit(w, i);
    let Some(car) = w.cars.get_mut(c as usize) else {
        return;
    };
    if car.state == CarState::Parked {
        car.state = CarState::Boarding;
        car.since = tick;
        car.driver = None;
        car.aboard.clear();
        car.done.clear();
        car.ready_at = 0;
    }
    let adult = w.content.life.adult(w.sims[i].age);
    if car.driver.is_none() && adult && car.owners.contains(&(i as u32)) {
        car.driver = Some(i as u32);
    }
    if !car.aboard.contains(&(i as u32)) {
        car.aboard.push(i as u32);
    }
}

/// The car `i` is in, waiting for or riding home in after work (they're away, but not at work).
pub(crate) fn homeward(w: &World, i: usize) -> bool {
    w.cars.iter().any(|c| c.done.contains(&(i as u32)))
}

/// `i` is done at work (paid, or out of school): true when they wait there for their car home
/// (or go in it now), false when they make their own way (no car, or its driver works on for
/// longer than `RIDE_WAIT_TICKS`).
pub(crate) fn off_work(w: &mut World, i: usize) -> bool {
    let tick = w.tick;
    let me = i as u32;
    let Some(c) = w.cars.iter().position(|c| {
        c.aboard.contains(&me) && matches!(c.state, CarState::Leaving | CarState::Away)
    }) else {
        return false;
    };
    let driver = w.cars[c].driver;
    let wait = driver.is_some_and(|d| {
        d == me
            || w.cars[c].done.contains(&d)
            || w.sims[d as usize]
                .away_until
                .is_some_and(|until| until <= tick + RIDE_WAIT_TICKS)
    });
    let car = &mut w.cars[c];
    if !wait {
        car.aboard.retain(|&a| a != me);
        return false;
    }
    car.done.push(me);
    car.ready_at = car.ready_at.max(tick + OFF_MAP_TICKS);
    w.sims[i].away_until = Some(tick + 1);
    true
}

/// `i` stands at the town's exit nearest them (out of sight).
pub(crate) fn to_exit(w: &mut World, i: usize) {
    if let Some(k) = nearest_exit(&w.exits, w.sims[i].pos) {
        w.sims[i].pos = w.exits[k];
    }
}
