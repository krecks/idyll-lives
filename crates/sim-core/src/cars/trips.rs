//! Trips out: who can drive, setting off, riding along and coming home.
//!
//! Split from `cars.rs` (see `mod.rs`).

use super::*;

/// `can_drive`, in a tick (from its `Ctx`).
pub(crate) fn could_drive(ctx: &crate::world::Ctx, sim: &Sim) -> bool {
    can_drive(ctx.travel, ctx.households, ctx.briefs, ctx.cars, sim)
}

/// Whether `sim` could drive somewhere now (`trip_car` without the place): an open-world town
/// with parking, a grown-up owner at home with the household's car parked there. From the
/// world's parts, so it's asked outside a tick too (`World::visit_fits_now`); in one, `could_drive`.
pub(crate) fn can_drive(
    travel: &Travel,
    households: &[crate::world::Household],
    briefs: &[crate::world::SimBrief],
    cars: &[Car],
    sim: &Sim,
) -> bool {
    let Some(home) = households.get(sim.household as usize).and_then(|h| h.plot) else {
        return false;
    };
    !travel.file.parking.is_empty()
        && sim.visiting.is_none()
        && briefs
            .get(sim.id as usize)
            .is_some_and(|b| b.adult && b.plot == Some(home))
        && cars.iter().any(|car| {
            car.household == sim.household
                && car.plot == home
                && car.state == CarState::Parked
                && car.trip.is_none()
                && !car.blocked
                && car.owners.contains(&sim.id)
        })
}

/// Whether `car`'s driveway has a way by road to `at`.
pub(crate) fn drives_to(car: &Car, travel: &Travel, at: [f32; 2]) -> bool {
    let g = &travel.graph;
    let (Some(m), Some(&p)) = (
        car.mouth(travel),
        g.near(at, ROAD_REACH, 1, Mode::Car).first(),
    ) else {
        return false;
    };
    g.path(m, p, Mode::Car, travel.country_speed).is_some()
}

/// The car `sim` drives to plot `plot` in an open-world town, and the door they get in by and
/// where they stand for it: the place is out of walking range (`autonomy.reach`), there's
/// public parking near it and a way there by road, and they're a grown-up owner at home with
/// the household's car parked there and nobody else heading for it. None: they walk.
pub(crate) fn trip_car(
    ctx: &crate::world::Ctx,
    sim: &Sim,
    plot: u32,
) -> Option<(u32, usize, [f32; 2])> {
    let travel = ctx.travel;
    if travel.file.parking.is_empty() || sim.visiting.is_some() {
        return None;
    }
    let me = ctx.briefs.get(sim.id as usize)?;
    let home = ctx.households.get(sim.household as usize)?.plot?;
    if !me.adult || me.plot != Some(home) {
        return None;
    }
    let door = ctx.plots.get(plot as usize)?.arrival_point();
    if crate::travel::dist(sim.pos, door) <= ctx.content.autonomy.reach {
        return None;
    }
    let (at, _) = parking_for(travel, door, ctx.cars, None)?;
    let (c, car) = ctx.cars.iter().enumerate().find(|(_, car)| {
        car.household == sim.household
            && car.plot == home
            && car.state == CarState::Parked
            && car.trip.is_none()
            && !car.blocked
            && !car.no_way
            && car.owners.contains(&sim.id)
    })?;
    let c = c as u32;
    if ctx.briefs.iter().any(|b| b.bound_for_car == Some(c)) || !drives_to(car, travel, at) {
        return None;
    }
    let (d, stand) = car.door(&ctx.nav, 0)?;
    Some((c, d, stand))
}

/// Whether `sim` can set off in car `c` now (at its door, as `trip_car` found it).
pub(crate) fn may_set_off(cars: &[Car], c: u32, sim: u32) -> bool {
    cars.get(c as usize).is_some_and(|car| {
        car.state == CarState::Parked
            && car.trip.is_none()
            && !car.blocked
            && car.owners.contains(&sim)
    })
}

/// The car `sim` is at the wheel of, on a trip there or back (`Leaving`, `Arriving`), or sat in
/// on one (driving or riding, or waiting in it for the others).
pub(crate) fn driving(cars: &[Car], sim: u32) -> Option<u32> {
    cars.iter()
        .position(|car| {
            (car.trip.is_some_and(|t| t.driver == sim)
                && matches!(car.state, CarState::Leaving | CarState::Arriving))
                || (car.trip.is_some() && car.aboard.contains(&sim))
        })
        .map(|c| c as u32)
}

/// Whether car `c` is parked out on a trip `sim` drives.
pub(crate) fn parked_out(cars: &[Car], c: u32, sim: u32) -> bool {
    cars.get(c as usize)
        .is_some_and(|car| car.state == CarState::Out && car.trip.is_some_and(|t| t.driver == sim))
}

/// `sim`'s car parked out on a trip, the door they get in by and where they stand for it: where
/// they go home from.
pub(crate) fn car_out_for(ctx: &crate::world::Ctx, sim: &Sim) -> Option<(u32, usize, [f32; 2])> {
    let (c, car) = ctx
        .cars
        .iter()
        .enumerate()
        .find(|(c, _)| parked_out(ctx.cars, *c as u32, sim.id))?;
    let trip = car.trip?;
    let (d, stand) = car.posed(trip.at, trip.yaw).door(&ctx.nav, 0)?;
    Some((c as u32, d, stand))
}

/// `i` gets in at car `c`'s door and drives off to plot `plot` (`Transition::Drive`): out of the
/// driveway, along the roads to the parking spot near it. Gone or taken since: they walk.
pub(crate) fn start_trip(w: &mut World, i: usize, c: u32, plot: u32, directed: bool) {
    let visit = Task {
        kind: TaskKind::Visit { plot },
        directed,
        routine: false,
    };
    let door = w.plots.get(plot as usize).map(|p| p.arrival_point());
    let park = door.and_then(|d| parking_for(&w.travel, d, &w.cars, Some(c as usize)));
    let (Some((at, yaw)), true) = (park, may_set_off(&w.cars, c, i as u32)) else {
        w.sims[i].bound_for_car = None;
        w.sims[i].queue.push_front(visit);
        return;
    };
    let tick = w.tick;
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    let car = &mut cars[c as usize];
    car.trip = Some(Trip {
        driver: i as u32,
        plot,
        directed,
        at,
        yaw,
    });
    car.route = None;
    if Route::on_roads(car, exits, travel, false).is_none() {
        car.trip = None;
        car.settle(exits, Some(travel));
        w.sims[i].bound_for_car = None;
        w.sims[i].queue.push_front(visit);
        return;
    }
    car.since = tick;
    car.progress = 0.0;
    car.driver = Some(i as u32);
    car.aboard = vec![i as u32];
    car.riders.clear();
    car.done.clear();
    // Those of the household setting off for somewhere it parks near now come along: it waits
    // for them in the driveway (`CarState::Boarding`).
    car.state = CarState::Boarding;
    let sim = &mut w.sims[i];
    sim.bound_for_car = Some(c);
    sim.current = Some(crate::world::Activity {
        task: visit,
        phase: Phase::Driving,
        tags: Default::default(),
    });
    if !call_trip_riders(w, c as usize) {
        w.cars[c as usize].state = CarState::Leaving;
    }
    seat_party(w, c as usize);
}

/// `i` gets into car `c` parked out on their trip and drives home (`Transition::DriveHome`):
/// at once, or once those who came with them are in (`call_party_home`).
pub(crate) fn drive_home(w: &mut World, i: usize, c: u32) {
    if !parked_out(&w.cars, c, i as u32) {
        w.sims[i].bound_for_car = None;
        walk_home(w, i, None);
        return;
    }
    let tick = w.tick;
    let car = &mut w.cars[c as usize];
    car.since = tick;
    car.route = None;
    car.driver = Some(i as u32);
    // (At the wheel, before any of the others in already.)
    car.aboard.retain(|&a| a != i as u32);
    car.aboard.insert(0, i as u32);
    let sim = &mut w.sims[i];
    sim.bound_for_car = Some(c);
    sim.current = Some(crate::world::Activity {
        task: Task {
            kind: TaskKind::GoHome,
            directed: false,
            routine: false,
        },
        phase: Phase::Driving,
        tags: Default::default(),
    });
    if !call_party_home(w, c as usize) {
        set_off_home(w, c as usize);
    }
    seat_party(w, c as usize);
}

/// Car `c`, parked out on a trip with its driver in, sets off home with whoever is in it (the
/// rest of those who came make their own way).
pub(crate) fn set_off_home(w: &mut World, c: usize) {
    let tick = w.tick;
    let car = &mut w.cars[c];
    car.state = CarState::Arriving;
    car.since = tick;
    car.progress = 0.0;
    car.route = None;
    let aboard = car.aboard.clone();
    car.riders.retain(|r| aboard.contains(r));
}

/// The door the one in seat `seat` gets in by on their own side (as `Car::door` tries first).
pub(crate) fn own_door(seat: usize) -> usize {
    [0, 1, 3, 2][seat.min(DOORS - 1)]
}

/// Those in car `c` on a trip sat in their seats where it is now: the driver at the wheel, the
/// others a seat each in the order they got in.
pub(crate) fn seat_party(w: &mut World, c: usize) {
    let World {
        cars,
        exits,
        travel,
        sims,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    if car.trip.is_none() {
        return;
    }
    let pose = car.pose_on(exits, Some(travel));
    let Some((at, yaw, _)) = pose.or(car.trip.map(|t| (t.at, t.yaw, false))) else {
        return;
    };
    let posed = car.posed(at, yaw);
    for (seat, &a) in car.aboard.iter().enumerate() {
        if let Some(s) = sims.get_mut(a as usize) {
            s.pos = posed.seat_at(seat, own_door(seat));
            s.yaw = yaw;
            s.pose = crate::content::Pose::Sit;
        }
    }
}

/// Once a tick for a car on a trip: on along the roads, parked out there, or back home.
pub(crate) fn trip_update(w: &mut World, c: usize) {
    let Some(trip) = w.cars[c].trip else { return };
    let d = trip.driver as usize;
    // At the wheel; or (just loaded, or not their turn yet) about to be.
    let phase = w.sims[d].current().map(|a| a.phase.clone());
    let at_wheel = matches!(phase, None | Some(Phase::Driving));
    // (Those riding with them who left town or were called away get out.)
    let gone: Vec<u32> = w.cars[c]
        .aboard
        .iter()
        .copied()
        .filter(|&a| {
            a != trip.driver
                && w.sims.get(a as usize).is_none_or(|s| {
                    !s.here()
                        || !s
                            .current()
                            .is_some_and(|a| matches!(a.phase, Phase::Driving))
                })
        })
        .collect();
    if !gone.is_empty() {
        w.cars[c].aboard.retain(|a| !gone.contains(a));
        w.cars[c].riders.retain(|a| !gone.contains(a));
    }
    match w.cars[c].state {
        // In the driveway, the driver in: waiting for the others coming along.
        CarState::Boarding => {
            let waited = w.tick - w.cars[c].since;
            if !at_wheel || !w.sims[d].here() {
                // (The driver called away after all: the trip is off.)
                let aboard = std::mem::take(&mut w.cars[c].aboard);
                for a in aboard {
                    let_out(w, a as usize);
                }
                let World {
                    cars,
                    exits,
                    travel,
                    ..
                } = &mut *w;
                let car = &mut cars[c];
                car.state = CarState::Parked;
                car.trip = None;
                car.riders.clear();
                car.driver = None;
                car.route = None;
                car.settle(exits, Some(travel));
                return;
            }
            let expected = call_trip_riders(w, c);
            if (!expected && waited >= BOARD_MIN_TICKS) || waited >= TRIP_WAIT_TICKS {
                let tick = w.tick;
                let car = &mut w.cars[c];
                car.state = CarState::Leaving;
                car.since = tick;
                car.progress = 0.0;
            }
            seat_party(w, c);
        }
        CarState::Leaving => {
            let ticks = route_ticks(w, c, false);
            if at_wheel || trip_seen(w, c) {
                drive_on(w, c);
                seat_party(w, c);
            } else {
                // (Called away at the wheel, as they can't be: the car is there. Not in the
                // picture, nor where it's going: it would be seen to jump there, `trip_seen`.)
                w.cars[c].progress = ticks;
            }
            if w.cars[c].progress >= ticks {
                park_out(w, c);
            }
        }
        CarState::Arriving => {
            let ticks = route_ticks(w, c, true);
            if at_wheel || trip_seen(w, c) {
                drive_on(w, c);
                seat_party(w, c);
            } else {
                w.cars[c].progress = ticks;
            }
            if w.cars[c].progress >= ticks {
                let car = &mut w.cars[c];
                // Everyone in it gets out at home, the driver first (`arrive`).
                car.done = car
                    .aboard
                    .iter()
                    .copied()
                    .filter(|&a| at_wheel || a != d as u32)
                    .collect();
                car.driver = Some(d as u32);
                arrive(w, c);
            }
        }
        CarState::Out => {
            // (The driver, in it to go home, called away after all, to work say: out of it. They
            // come back to it going home, or it goes home when they get there.)
            if !at_wheel && w.cars[c].aboard.contains(&(d as u32)) {
                w.cars[c].aboard.retain(|&a| a != d as u32);
                let sim = &mut w.sims[d];
                if sim.bound_for_car == Some(c as u32) {
                    sim.bound_for_car = None;
                }
                sim.pose = crate::content::Pose::Stand;
            }
            let car = &w.cars[c];
            let driver_in = at_wheel && car.aboard.contains(&(d as u32));
            if driver_in {
                // The driver in to go home: once the others who came are in too (or have been
                // waited for long enough).
                let waited = w.tick - w.cars[c].since;
                let expected = call_party_home(w, c);
                if !expected || waited >= TRIP_WAIT_TICKS {
                    set_off_home(w, c);
                }
                seat_party(w, c);
                return;
            }
            // One of them in it, or on their way to it, to go home: the rest go home too.
            let going = !car.aboard.is_empty()
                || car
                    .riders
                    .iter()
                    .any(|&r| w.sims[r as usize].bound_for_car == Some(c as u32));
            // Unless the driver can't come (at work, out of town): those in it get out and make
            // their own way, and so do the rest of them. Or the driver hasn't come for as long as
            // a driver waits for them (`TRIP_WAIT_TICKS` since the first of them got in): those in
            // it get out, and the rest may still ride home with the driver.
            if going {
                let driver_away = w.sims[d].away_until.is_some() || !w.sims[d].here();
                let waited = !car.aboard.is_empty() && w.tick - car.since >= TRIP_WAIT_TICKS;
                if driver_away || waited {
                    let aboard = std::mem::take(&mut w.cars[c].aboard);
                    for &a in &aboard {
                        let_out(w, a as usize);
                    }
                    if driver_away {
                        w.cars[c].riders.clear();
                    } else {
                        w.cars[c].riders.retain(|r| !aboard.contains(r));
                    }
                } else {
                    call_party_home(w, c);
                }
            }
            seat_party(w, c);
            // Home some other way (and not on their way back to it): it's home too.
            let home = w
                .households
                .get(w.sims[d].household as usize)
                .and_then(|h| h.plot);
            let there = home.is_some() && w.briefs.get(d).and_then(|b| b.plot) == home;
            // (Not while the camera is on where it's parked out or where it parks at home: it
            // isn't seen to jump there, `in_picture`.)
            let seen = {
                let car = &w.cars[c];
                car.trip.is_some_and(|t| in_picture(w, t.at)) || in_picture(w, car.spot)
            };
            if there && w.sims[d].bound_for_car != Some(c as u32) && !seen {
                let car = &mut w.cars[c];
                car.done = car.aboard.clone();
                car.aboard.clear();
                arrive(w, c);
            }
        }
        // (A trip on a car that isn't on its way, out or back: done with.)
        CarState::Parked | CarState::Away => {
            let aboard = std::mem::take(&mut w.cars[c].aboard);
            for a in aboard {
                let_out(w, a as usize);
            }
            let World {
                cars,
                exits,
                travel,
                ..
            } = &mut *w;
            cars[c].trip = None;
            cars[c].riders.clear();
            cars[c].route = None;
            cars[c].settle(exits, Some(travel));
        }
    }
}

/// `i`, sat in a car on a trip that won't take them now, gets out where they are and goes on
/// their own way (on foot).
pub(crate) fn let_out(w: &mut World, i: usize) {
    let Some(sim) = w.sims.get_mut(i) else {
        return;
    };
    if let Some(a) = sim.current.take_if(|a| matches!(a.phase, Phase::Driving)) {
        sim.queue.push_front(a.task);
    }
    sim.bound_for_car = None;
    sim.pose = crate::content::Pose::Stand;
}

/// Whether household member `j` sets off now for somewhere car `c`, about to leave on a trip,
/// parks near (`Trip::at`): at home (or still close by), on their way there on foot, or about to
/// be. The place they're off to.
pub(crate) fn sets_off_with(w: &World, j: usize, c: usize) -> Option<(u32, bool)> {
    let car = &w.cars[c];
    let trip = car.trip?;
    let s = &w.sims[j];
    if s.household != car.household
        || j as u32 == trip.driver
        || !s.here()
        || s.away_until.is_some()
        || car.aboard.contains(&(j as u32))
        || w.briefs.get(j).is_none_or(|b| b.baby)
    {
        return None;
    }
    let near = w.briefs.get(j).and_then(|b| b.plot) == Some(car.plot)
        || crate::travel::dist(s.pos, car.spot) < CALL_REACH;
    let task = match s.current() {
        Some(a) if matches!(a.phase, Phase::Routing { .. }) => a.task,
        Some(_) => return None,
        None => *s.queue.front()?,
    };
    let TaskKind::Visit { plot } = task.kind else {
        return None;
    };
    // (A guest there already as they set off from home: `Transition::Visit`; not from a visit
    // elsewhere.)
    if s.visiting.is_some_and(|v| v.plot != plot) {
        return None;
    }
    let door = w.plots.get(plot as usize)?.arrival_point();
    let far = crate::travel::dist(s.pos, door) > w.content.autonomy.reach;
    let parks = parking_for(&w.travel, door, &w.cars, Some(c)).is_some_and(|(at, _)| at == trip.at);
    (near && far && parks).then_some((plot, task.directed))
}

/// While car `c` waits in the driveway on a trip: the rest of the household setting off now for
/// somewhere it parks near is called to it (those on their way on foot turn round, while they're
/// still close by), as many as it has seats. Whether anyone is on their way to it.
pub(crate) fn call_trip_riders(w: &mut World, c: usize) -> bool {
    let me = c as u32;
    let mut coming = 0;
    for j in 0..w.sims.len() {
        let s = &w.sims[j];
        if s.bound_for_car == Some(me)
            && !w.cars[c].aboard.contains(&(j as u32))
            && s.here()
            && s.current()
                .is_some_and(|a| matches!(a.task.kind, TaskKind::Visit { .. }))
        {
            coming += 1;
        }
    }
    for j in 0..w.sims.len() {
        if w.cars[c].aboard.len() + coming >= SEATS {
            break;
        }
        if w.sims[j].bound_for_car == Some(me) || sets_off_with(w, j, c).is_none() {
            continue;
        }
        // To the car instead (`trip_ride` picks it now).
        let sim = &mut w.sims[j];
        if let Some(a) = sim.current.take() {
            sim.queue.push_front(a.task);
        }
        coming += 1;
    }
    coming > 0
}

/// Car `c` parked out on a trip, one of those who came in it (or on their way to it) to go home:
/// the others who came go home too (with it: `ride_home_for`), leaving what they chose to do.
/// Whether any of them is still to get in.
pub(crate) fn call_party_home(w: &mut World, c: usize) -> bool {
    let me = c as u32;
    let Some(trip) = w.cars[c].trip else {
        return false;
    };
    let party: Vec<u32> = std::iter::once(trip.driver)
        .chain(w.cars[c].riders.iter().copied())
        .collect();
    let home = w
        .households
        .get(w.cars[c].household as usize)
        .and_then(|h| h.plot);
    let mut expected = false;
    for j in party {
        if w.cars[c].aboard.contains(&j) {
            continue;
        }
        let i = j as usize;
        // (Home some other way already: not coming.)
        if j != trip.driver && home.is_some() && w.briefs.get(i).and_then(|b| b.plot) == home {
            w.cars[c].riders.retain(|&r| r != j);
            continue;
        }
        let World {
            sims,
            content,
            objects,
            ..
        } = &mut *w;
        let Some(s) = sims
            .get_mut(i)
            .filter(|s| s.here() && s.away_until.is_none())
        else {
            continue;
        };
        let homeward = |t: &Task| matches!(t.kind, TaskKind::GoHome);
        let going = s.current().is_some_and(|a| homeward(&a.task)) || s.queue.iter().any(homeward);
        if j != trip.driver {
            expected = true;
        }
        if s.bound_for_car == Some(me) || going {
            continue;
        }
        // What they chose for themselves ends; what they were told to do comes first.
        if s.current.as_ref().is_some_and(|a| !a.task.directed) {
            crate::world::end_activity(s, content, objects);
        }
        s.queue.push_front(Task {
            kind: TaskKind::GoHome,
            directed: false,
            routine: false,
        });
    }
    expected
}

/// The car `sim` rides in to plot `plot` in an open-world town, their seat in it, and the door
/// they get in by and where they stand for it: their household's car waiting in the driveway on
/// a trip to park near there (`CarState::Boarding`), with a seat for them, the place out of
/// walking range and they at home (or still close by). None: they walk (or drive: `trip_car`).
pub(crate) fn trip_ride(
    ctx: &crate::world::Ctx,
    sim: &Sim,
    plot: u32,
) -> Option<(u32, usize, usize, [f32; 2])> {
    let travel = ctx.travel;
    // (A guest there already as they set off from home: `Transition::Visit`.)
    if travel.file.parking.is_empty() || sim.visiting.is_some_and(|v| v.plot != plot) {
        return None;
    }
    let me = ctx.briefs.get(sim.id as usize)?;
    let home = ctx.households.get(sim.household as usize)?.plot?;
    let door = ctx.plots.get(plot as usize)?.arrival_point();
    if me.baby || crate::travel::dist(sim.pos, door) <= ctx.content.autonomy.reach {
        return None;
    }
    let (c, car) = ctx.cars.iter().enumerate().find(|&(c, car)| {
        car.household == sim.household
            && car.plot == home
            && car.state == CarState::Boarding
            && car.trip.is_some_and(|t| {
                t.driver != sim.id
                    && parking_for(travel, door, ctx.cars, Some(c))
                        .is_some_and(|(at, _)| at == t.at)
            })
            && !car.blocked
            && !car.aboard.contains(&sim.id)
    })?;
    let near = me.plot == Some(home) || crate::travel::dist(sim.pos, car.spot) < CALL_REACH;
    let c = c as u32;
    // A seat of their own: after those in it and on their way to it.
    let seat = car.aboard.len()
        + ctx
            .briefs
            .iter()
            .enumerate()
            .filter(|(j, b)| {
                *j as u32 != sim.id && b.trip_car == Some(c) && !car.aboard.contains(&(*j as u32))
            })
            .count();
    if !near || seat >= SEATS {
        return None;
    }
    let (d, stand) = car.door(&ctx.nav, seat)?;
    Some((c, seat, d, stand))
}

/// Whether `sim` can get into car `c` now to ride along on its trip: out (`home` false), it
/// waits in the driveway for them; home, it's parked out there and they came in it.
pub(crate) fn may_ride(cars: &[Car], c: u32, sim: u32, home: bool) -> bool {
    cars.get(c as usize).is_some_and(|car| {
        car.trip.is_some_and(|t| t.driver != sim)
            && car.aboard.len() < SEATS
            && !car.aboard.contains(&sim)
            && if home {
                car.state == CarState::Out && car.riders.contains(&sim)
            } else {
                car.state == CarState::Boarding
            }
    })
}

/// The car `sim` came in on a trip, parked out there, to ride home in: the car, their seat,
/// the door they get in by and where they stand for it.
pub(crate) fn ride_home_for(
    ctx: &crate::world::Ctx,
    sim: &Sim,
) -> Option<(u32, usize, usize, [f32; 2])> {
    let (c, car) = ctx
        .cars
        .iter()
        .enumerate()
        .find(|(c, _)| may_ride(ctx.cars, *c as u32, sim.id, true))?;
    let trip = car.trip?;
    let seat = 1 + car.riders.iter().position(|&r| r == sim.id)?;
    let (d, stand) = car
        .posed(trip.at, trip.yaw)
        .door(&ctx.nav, seat.min(SEATS - 1))?;
    Some((c as u32, seat, d, stand))
}

/// `i` at car `c`'s door gets in to ride along (`Transition::Ride`): off to `plot` with it, or
/// (None) home in it. Gone since, or full: they make their own way.
pub(crate) fn ride(w: &mut World, i: usize, c: u32, plot: Option<u32>, directed: bool) {
    let task = Task {
        kind: plot.map_or(TaskKind::GoHome, |plot| TaskKind::Visit { plot }),
        directed,
        routine: false,
    };
    let me = i as u32;
    if !may_ride(&w.cars, c, me, plot.is_none()) {
        w.sims[i].bound_for_car = None;
        w.sims[i].queue.push_front(task);
        return;
    }
    let tick = w.tick;
    let car = &mut w.cars[c as usize];
    // (Parked out there, the first of them in to go home: how long they've waited for the driver.)
    if plot.is_none() && car.aboard.is_empty() {
        car.since = tick;
    }
    car.aboard.push(me);
    if plot.is_some() && !car.riders.contains(&me) {
        car.riders.push(me);
    }
    let sim = &mut w.sims[i];
    if plot.is_none() {
        sim.visiting = None;
    }
    sim.bound_for_car = Some(c);
    sim.current = Some(crate::world::Activity {
        task,
        phase: Phase::Driving,
        tags: Default::default(),
    });
    seat_party(w, c as usize);
}

/// Car `c` is at its trip's parking spot: parked there (`CarState::Out`), and its driver gets
/// out beside it and walks on to the place.
pub(crate) fn park_out(w: &mut World, c: usize) {
    let Some(trip) = w.cars[c].trip else { return };
    let tick = w.tick;
    let at_wheel = matches!(
        w.sims[trip.driver as usize].current().map(|a| &a.phase),
        None | Some(Phase::Driving)
    );
    let (at, yaw) = {
        let World {
            cars,
            exits,
            travel,
            ..
        } = &mut *w;
        let route = cars[c].route(exits, Some(travel), false);
        let (p, yaw, _) = route.at(route.ticks());
        (p, yaw)
    };
    let car = &mut w.cars[c];
    car.state = CarState::Out;
    car.since = tick;
    car.route = None;
    let aboard = std::mem::take(&mut car.aboard);
    car.trip = Some(Trip { at, yaw, ..trip });
    let posed = car.posed(at, yaw);
    // Those who came along get out by their doors and walk on to where they're going (they come
    // home with it: `Car::riders`).
    let riders: Vec<(usize, u32)> = aboard
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, a)| a != trip.driver)
        .collect();
    w.cars[c].riders = riders.iter().map(|&(_, a)| a).collect();
    for (seat, a) in riders {
        let door = posed.door(&w.nav(), seat).map(|(_, p)| p);
        let sim = &mut w.sims[a as usize];
        sim.pos = door.unwrap_or(posed.door_at(own_door(seat)));
        sim.yaw = yaw;
        sim.pose = crate::content::Pose::Stand;
        sim.bound_for_car = None;
        if let Some(act) = sim.current.take() {
            sim.queue.push_front(act.task);
        }
    }
    if !at_wheel {
        return;
    }
    let door = posed.door(&w.nav(), 0).map(|(_, p)| p);
    let sim = &mut w.sims[trip.driver as usize];
    sim.pos = door.unwrap_or(posed.door_at(0));
    sim.yaw = yaw;
    sim.pose = crate::content::Pose::Stand;
    sim.bound_for_car = None;
    sim.current = None;
    sim.queue.push_front(Task {
        kind: TaskKind::Visit { plot: trip.plot },
        directed: trip.directed,
        routine: false,
    });
}
