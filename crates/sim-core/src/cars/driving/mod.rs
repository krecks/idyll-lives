//! The tick’s driving: owners and trips, the way on, and the zebras.

use super::*;

mod ease;
mod jam;
mod pass;
mod sight;
mod unjam;

pub(crate) use ease::*;
pub(crate) use jam::*;
pub(crate) use pass::*;
pub(crate) use sight::*;
pub(crate) use unjam::*;

/// Once a tick (after `life::update`): owners, cars going with their household, setting off and
/// coming home, what can't be used, and the tiles parked cars take up.
pub(crate) fn update(w: &mut World) {
    if w.cars.is_empty() {
        return;
    }
    // (Where each car is, for those pairing up this tick: `pull_want`.)
    w.cars_near.build(&w.cars);
    let tick = w.tick;
    if w.cars_nav != w.nav_version {
        w.cars_nav = w.nav_version;
        w.cars_parked = None;
        crate::profile::phase("cars::update/refresh", || refresh(w));
    }
    let cars_start = crate::profile::start();
    // Someone who set off for a car and turned to using something instead isn't coming: the car
    // doesn't wait for them, nor is it kept from the others.
    for s in &mut w.sims {
        if s.bound_for_car.is_some()
            && s.current()
                .is_some_and(|a| matches!(a.phase, Phase::Using { .. }))
        {
            s.bound_for_car = None;
        }
    }
    for c in 0..w.cars.len() {
        // (A work taxi is the taxis' own: `taxi/`; but one driving off, its run given up in
        // sight.)
        if w.cars[c].taxi {
            if w.cars[c].off_run {
                drive_off(w, c);
            }
            continue;
        }
        owners(w, c);
        relocate(w, c);
        renew(w, c);
        let gone = |w: &World, s: u32| w.sims.get(s as usize).is_none_or(|s| !s.here());
        // Nobody to drive it home (the driver died or moved away while out): it's back. (Not
        // while it's in the picture, nor its spot at home: it would jump there, `in_picture`;
        // meanwhile it goes on.)
        let seen =
            w.cars[c].shown.is_some_and(|s| in_picture(w, s.0)) || in_picture(w, w.cars[c].spot);
        if !w.cars[c].at_home() && w.cars[c].driver.is_none_or(|d| gone(w, d)) && !seen {
            if w.cars[c].trip.take().is_some() {
                // (Those riding with them get out and make their own way.)
                let aboard = w.cars[c].aboard.clone();
                for s in aboard {
                    let_out(w, s as usize);
                }
                let World {
                    cars,
                    exits,
                    travel,
                    ..
                } = &mut *w;
                cars[c].route = None;
                cars[c].riders.clear();
                cars[c].settle(exits, Some(travel));
            }
            let car = &mut w.cars[c];
            let left = std::mem::take(&mut car.done);
            car.aboard.clear();
            car.state = CarState::Parked;
            car.driver = None;
            for s in left {
                w.sims[s as usize].away_until = None;
                walk_home(w, s as usize, None);
            }
            continue;
        }
        if w.cars[c].trip.is_some() {
            crate::profile::phase("cars::update/trip_update", || trip_update(w, c));
            continue;
        }
        let state = w.cars[c].state;
        let since = w.cars[c].since;
        match state {
            // (Only on a trip, above.)
            CarState::Out => {}
            CarState::Parked => {
                // An owner on their way to drive it: the others setting off now come along.
                let bound = |s: &Sim| {
                    s.bound_for_car == Some(c as u32)
                        && s.current()
                            .is_some_and(|a| matches!(a.task.kind, TaskKind::Work))
                };
                if w.cars[c].owners.iter().any(|&o| bound(&w.sims[o as usize])) {
                    one_driver(w, c);
                    call_riders(w, c);
                }
            }
            CarState::Boarding => {
                let driver = w.cars[c].driver;
                let waited = tick - since;
                // No driver after all (the one coming was called away, or left town): those in it
                // have gone on (they're at work).
                if driver.is_some_and(|d| gone(w, d))
                    || (driver.is_none() && waited >= BOARD_WAIT_TICKS)
                {
                    let car = &mut w.cars[c];
                    car.state = CarState::Parked;
                    car.driver = None;
                    car.aboard.clear();
                    continue;
                }
                let expected = call_riders(w, c);
                if driver.is_none() {
                    continue;
                }
                if (!expected && waited >= BOARD_MIN_TICKS) || waited >= BOARD_WAIT_TICKS {
                    let car = &mut w.cars[c];
                    car.state = CarState::Leaving;
                    car.since = tick;
                    car.progress = 0.0;
                }
            }
            CarState::Leaving => {
                let ticks = route_ticks(w, c, false);
                drive_on(w, c);
                if w.cars[c].progress >= ticks {
                    w.cars[c].state = CarState::Away;
                    w.cars[c].since = tick;
                    // (A garage built or taken down while it was leaving: it comes home to it.)
                    let plot = w.cars[c].plot;
                    if w.cars[c].drive_spot.is_some()
                        || w.garages.iter().any(|g| g.plot == plot && g.rot.is_some())
                    {
                        park_by_garage(w, c);
                    }
                }
            }
            CarState::Away => {
                // Those still at work who won't be done in time make their own way home.
                let car = &w.cars[c];
                let driver_done = car.driver.is_some_and(|d| car.done.contains(&d));
                let stays: Vec<u32> = car
                    .aboard
                    .iter()
                    .copied()
                    .filter(|a| {
                        !car.done.contains(a)
                            && (gone(w, *a)
                                || (driver_done
                                    && w.sims[*a as usize]
                                        .away_until
                                        .is_none_or(|u| u > tick + RIDE_WAIT_TICKS)))
                    })
                    .collect();
                w.cars[c].aboard.retain(|a| !stays.contains(a));
                let car = &w.cars[c];
                if driver_done
                    && car.aboard.iter().all(|a| car.done.contains(a))
                    && tick >= car.ready_at
                {
                    let car = &mut w.cars[c];
                    car.state = CarState::Arriving;
                    car.since = tick;
                    car.progress = 0.0;
                }
                // Home once the last of them is done, back in town and driven home.
                let car = &w.cars[c];
                let last = car
                    .aboard
                    .iter()
                    .filter(|a| !car.done.contains(a))
                    .filter_map(|&a| w.sims[a as usize].away_until)
                    .map(|u| u + car.off_map_ticks())
                    .max()
                    .unwrap_or(tick)
                    .max(car.ready_at);
                let drive = route_ticks(w, c, true);
                keep_away(w, c, last.max(tick) + drive.ceil() as u64);
            }
            CarState::Arriving => {
                let ticks = route_ticks(w, c, true);
                drive_on(w, c);
                let left = ticks - w.cars[c].progress;
                if left <= 0.0 {
                    arrive(w, c);
                } else {
                    keep_away(w, c, tick + left.ceil() as u64);
                }
            }
        }
    }
    crate::profile::end("cars::update/cars", cars_start);
    crate::profile::phase("cars::update/reblock", || reblock(w));
    crate::profile::phase("cars::update/poses", || poses(w));
}

/// Ticks car `c`'s drive out of town or (`home`) back takes.
pub(crate) fn route_ticks(w: &mut World, c: usize, home: bool) -> f32 {
    let World {
        cars,
        exits,
        travel,
        ..
    } = w;
    cars[c].route(exits, Some(travel), home).ticks()
}

/// A tick further along its drive (on an avenue, a motorway or a ramp, as far as its own speed
/// takes it: `ease`), unless someone is in its way: it waits for them (`go_on`).
/// Someone at something (a bench, a letterbox) is beside its way at most, nothing stands on a
/// street or a driveway: it doesn't wait for them. Coming into town, it waits out of sight at the
/// way in till the way in is clear of the one before it (`room_to_come_in`), the one there longest
/// first: cars coming home at the same time come in one after another. Into a junction's box or
/// across a lane out of (or into) a driveway or a bay only holding the claim to it
/// (`traffic::Traffic::may_go`), nor into another's (`Traffic::blocked`). Pulled over where the
/// lane is narrow, it passes one coming the other way (`pull_over`; it waits while they're too
/// near, `too_near`). And never deeper into another car than it is (`guard`).
pub(crate) fn drive_on(w: &mut World, c: usize) {
    let home = w.cars[c].state == CarState::Arriving;
    w.cars[c].at_red = false;
    match w.cars[c].sight {
        Sight::Seen => {}
        Sight::Ghost => return drive_through(w, c, home),
        Sight::Wait(k) => {
            let car = &mut w.cars[c];
            car.unhold();
            car.waits_for = Some(k);
            return;
        }
    }
    let t = w.cars[c].progress;
    let moving = std::mem::take(&mut w.cars[c].moved);
    // (At a speed of its own (`ease`): standing, unless it goes on below.)
    let carried = w.cars[c].speed;
    w.cars[c].speed = carried.map(|_| 0.0);
    // Held up for good, nose to nose with one waiting for it (`JAM_TICKS`), it backs up a little
    // out of its way (`back_off`).
    let key = car_first(c);
    let jammed = w.cars[c].waits_for.is_some_and(|k| waits_back(w, k, key));
    // (Not on its way since, it's still held up: two pulling over for each other by turns wait
    // for each other every other tick.)
    let jam = &mut w.cars[c].jam;
    *jam = if jammed {
        (jam.0.saturating_add(1), jam.1, jam.2)
    } else if moving {
        (0, 0, false)
    } else {
        *jam
    };
    let jam = *jam;
    if jam.1 == 0 {
        w.cars[c].backed_for = None;
    }
    // (The other can back up no further, and waits for it as it does: it backs up now.)
    let theirs = w.cars[c].waits_for.is_some_and(|k| stuck_back(w, k));
    // (Backed up as far as it goes, `JAM_BACK`, it can back up no further: the other does.)
    // (Backing up a tick on, turned round: `unjam`.)
    let turning = w.cars[c].detour.is_some()
        && w.cars[c]
            .route
            .as_ref()
            .is_some_and(|(_, r)| r.at(t + 1.0).2);
    // (Backed up out of the way to let those it was held up with by, `unjam`: out of the way it
    // waits till they've gone on.)
    if !turning && let Some((until, by)) = w.cars[c].let_by.as_ref() {
        // (Not for one that waits for it in turn, behind it say: that one can't go on first.)
        let me = car_first(c);
        let still = (w.tick < *until)
            .then(|| {
                by.iter().find(|&&(o, p, st)| {
                    w.cars.get(o as usize).is_some_and(|x| {
                        x.state == st
                            && x.progress <= p + 1.0
                            && x.sight == Sight::Seen
                            && !crate::traffic::waits_on(w, car_first(o as usize)).contains(&me)
                    })
                })
            })
            .flatten()
            .map(|&(o, ..)| o);
        match still {
            Some(o) => {
                let car = &mut w.cars[c];
                car.waits_for = Some(car_first(o as usize));
                car.moved = false;
                return;
            }
            None => w.cars[c].let_by = None,
        }
    }
    if jammed && (jam.0 > JAM_TICKS || theirs) && !w.cars[c].outside() && !turning {
        if jam.1 < JAM_BACK && back_off(w, c) {
            return;
        }
        w.cars[c].jam.2 = true;
    }
    // (Backed up out of its way, it stays there while that one still waits for it, or while it
    // still waits its turn after it, or that one is still by it (held up there by others now and
    // then, on foot too): not on into its way again. Not one that could back up no further
    // (`Car::jam`'s third, the other backs instead): it goes on as soon as the other gives it
    // room, else both stood back for each other for good, the other coming up again to wait on
    // it, Tide Sands.)
    let asking = !w.traffic.waits_for(key).is_empty();
    let by = w.cars[c].backed_for.is_some_and(|k| {
        let me = &w.cars[c];
        w.cars
            .get(car_index(k))
            .and_then(|o| Some((o, o.shown.filter(|s| s.2)?)))
            .zip(me.shown)
            .is_some_and(|((o, s), m)| {
                crate::travel::dist(m.0, s.0) < (me.len + o.len) / 2.0 + 2.0 * AHEAD
            })
    });
    if (jammed || asking || by) && jam.1 > 0 && !jam.2 && !w.cars[c].outside() {
        return;
    }
    // (Whom it waited for last tick: pulled over for that one, it doesn't pull back in into its
    // way, `pull_over`.)
    let waited = w.cars[c].waits_for.take();
    w.cars[c].on_foot.clear();
    let others = vehicles(&w.cars, Some(c));
    if w.cars[c].outside() && !room_to_come_in(w, c, &others) {
        return;
    }
    let tick = w.tick;
    // (Turning round, it backs up as it stands, pulled over as far as it was: that's the room it
    // found clear, `turn_round`.)
    if !w.cars[c].outside() && !turning {
        pull_over(w, c, &others, moving, waited);
    }
    {
        let World {
            cars,
            exits,
            travel,
            traffic,
            ..
        } = &mut *w;
        // (At a zebra, someone on it or stepping onto it: it stops at the line for them.)
        let waited = cars[c].zebra_wait;
        let route = cars[c].route(exits, Some(travel), home);
        let whom = zebra_yield(route, &traffic.zebra_walkers, t, waited, |gate| {
            traffic.green(Some(gate))
        });
        match whom {
            Some(whom) if !whom.is_empty() => {
                let car = &mut cars[c];
                car.zebra_wait = car.zebra_wait.saturating_add(1);
                car.on_foot = whom;
                return;
            }
            Some(_) => {}
            None => cars[c].zebra_wait = 0,
        }
        let route = cars[c].route(exits, Some(travel), home);
        if !traffic.may_go(tick, key, u32::from(home), &route.claims, t, t + 1.0) {
            cars[c].at_red = traffic.red_for(key);
            // (Waiting for its claim: for one in its way that waits for it in turn, on the road
            // or for a claim of its own, as one in its way on the road, so that the later of them
            // backs up out of the other's way, `back_off`.)
            let by = traffic.waits_for(key).to_vec();
            cars[c].waits_for = by
                .iter()
                .copied()
                .find(|&k| {
                    cars.get(car_index(k))
                        .is_some_and(|o| o.waits_for == Some(key))
                        || traffic.waits_for(k).contains(&key)
                })
                .or(cars[c].waits_for);
            return;
        }
    }
    // (Pulled over further than it has room on: back towards its lane first. Not turning round:
    // it backs up as it stands, into a driveway's mouth too, `turn_round`.)
    if !turning && w.cars[c].shift > route_room(&w.cars[c], t + 1.0) + 1e-3 {
        // (Pulled over for one coming the other way, it waits for that one to pass; for one
        // waiting for it, it pulls back in first.)
        if w.cars[c].waits_for.is_none()
            && w.cars[c]
                .pulled_for
                .is_none_or(|k| !w.traffic.waits_for(k).contains(&key))
        {
            w.cars[c].waits_for = w.cars[c].pulled_for;
        }
        return;
    }
    // (On an avenue, a motorway or a ramp, as far as its own speed takes it, `ease`; else a tick
    // on, as its way's timing has it. Coming up to a merge, asking for it ahead.)
    if carried.is_some() {
        ask_ahead(w, c, t, carried, moving, home);
    }
    let eased = ease(w, c, t, carried, moving, home, &others);
    if let Some(e) = eased.as_ref() {
        // (Which goes first at a merge, kept with the one merging till it's merged.)
        for &(k, by, first) in &e.decided {
            let list = &mut w.cars[k].merge_first;
            if !list.iter().any(|d| d.0 == by) {
                list.push((by, first));
            }
        }
        if !w.cars[c].merge_first.is_empty() && !w.traffic.is_merging(key) {
            w.cars[c].merge_first.clear();
        }
    }
    let step = eased.as_ref().map_or(1.0, |e| e.step);
    if let Some(e) = eased.as_ref().filter(|e| e.step <= 0.0) {
        let car = &mut w.cars[c];
        car.speed = Some(0.0);
        car.waits_for = e.behind.or(car.waits_for);
        return;
    }
    let World {
        cars,
        exits,
        travel,
        sims,
        lot,
        blocked,
        stairs,
        content,
        objects,
        traffic,
        ..
    } = w;
    // (Those waiting for it, their bodies or claims in its way: it goes on by into the ground one
    // of them turning round still backs up over.)
    let waiting: Vec<u64> = traffic
        .turning
        .iter()
        .map(|t| t.0)
        .filter(|&k| {
            cars.get(car_index(k))
                .is_some_and(|o| o.waits_for == Some(key))
                || traffic.waits_for(k).contains(&key)
        })
        .collect();
    // (Those turning round on another level than its own, `travel::Layer`: not in its way.)
    let mine = cars[c].layer_on(t);
    let off_level: Vec<u64> = traffic
        .turning
        .iter()
        .map(|t| t.0)
        .filter(|&k| {
            cars.get(car_index(k))
                .is_some_and(|o| !o.layer().meets(mine))
        })
        .collect();
    let car = &mut cars[c];
    let shift = car.shift;
    // Where it would be a tick on, its body and the ground in front of it (behind it while it
    // backs up): not round its tail, as people may follow it close (on their way the same way),
    // but all along its sides, as its tail swings out on a turn.
    let route = car.route(exits, Some(travel), home);
    let ((at, yaw, reverse), (was, was_yaw, _)) = (route.at(t + step), route.at(t));
    let (at, was) = (shifted(at, yaw, shift), shifted(was, was_yaw, shift));
    let ticks = route.ticks();
    let first_claim = route.claims.first().map(|c| c.from);
    // (Turning round, `unjam`, it backs slowly into room it found clear: only what's right behind
    // it holds it up, `TURN_BEHIND`.)
    let back = if reverse && car.detour.is_some() {
        TURN_BEHIND
    } else {
        AHEAD
    };
    // (On the move faster than on a country road, it keeps further back from the one in front,
    // the faster the further: `follow_ahead`; at a speed of its own (`ease`), slowing down to.)
    let speed = match &eased {
        Some(e) => e.speed,
        None if moving => crate::travel::dist(at, was) * DRIVE_TICKS_PER_SECOND,
        None => 0.0,
    };
    // (At a speed of its own, it keeps back as it slows down, `ease`: here only `AHEAD`, the last
    // resort.)
    let (ahead, behind) = if reverse {
        (0.0, back)
    } else if eased.is_some() {
        (AHEAD, 0.0)
    } else {
        (follow_ahead(speed, travel.country_speed), 0.0)
    };
    let layer = car.layer_on(t);
    let me = Driving {
        at,
        yaw,
        len: car.len,
        wid: car.wid,
        ahead,
        behind,
        first: key,
        layer,
    };
    let now = Driving {
        at: was,
        yaw: was_yaw,
        ..me
    };
    car.next = Some((at, yaw));
    car.next_ground = [ahead, behind];
    // Not into what's left of a claim another holds. (Pulling over to pass one in it only as far
    // as it will on its way: not into a junction's box just ahead, `pull_want`.)
    let (body_now, body_next) = (now.body(), me.body());
    let pass = ([1.0, drive_ticks(0.5), drive_ticks(1.0)]
        .into_iter()
        .map(|k| route_pass(car, t + k))
        .fold(f32::INFINITY, f32::min)
        - shift)
        .max(0.0);
    let holding_here = traffic.holding(key);
    let in_drive =
        !home && !holding_here && car.detour.is_none() && first_claim.is_some_and(|from| t < from);
    // (Backing up out of where it was held up, turned round, `unjam`: into clear room, never into
    // another, as `guard` keeps it; not kept by the claims of those it backs away from.)
    let turning = reverse && car.detour.is_some();
    if let Some(k) = traffic
        .blocked(
            key,
            &body_now,
            &body_next,
            me.heading(),
            pass,
            in_drive,
            layer,
        )
        .filter(|_| !turning)
    {
        car.waits_for = Some(k);
        return;
    }
    // Not on past one coming the other way too near to pass (pulling over meanwhile); but for
    // one waiting for it (for its claim, or its body in the way of its own): it goes on by, as
    // near as it can, never nearer than `GAP` (`closes_in`, as the guard). Nor in a junction's box
    // holding its claim there: there the claims say who goes (one coming the other way let in with
    // it can pass it in their lanes, nobody pulls over there; one not let in waits for it), and
    // waiting for one coming would let that one's claim through this one's (`Traffic::begin`'s
    // rule 1).
    let boxed = holding_here && in_box(&travel.graph.boxes, at, car.len);
    if !reverse
        && !boxed
        && let Some(o) = others.iter().find(|o| {
            if !o.free(&me) {
                return false;
            }
            if traffic.waits_for(o.first).contains(&key) {
                let (h, way) = (me.heading(), o.heading());
                return h[0] * way[0] + h[1] * way[1] < -ONCOMING
                    && closes_in(&body_now, &body_next, &o.body());
            }
            too_near((at, yaw, car.len, car.wid), (o.at, o.yaw, o.len, o.wid))
                && !too_near(
                    (was, was_yaw, car.len, car.wid),
                    (o.at, o.yaw, o.len, o.wid),
                )
        })
    {
        car.waits_for = Some(o.first);
        return;
    }
    // Nor into the ground one turning round still backs up over (`unjam`): it has the way; but
    // for one waiting for this one (its body in that one's way), which goes on by, never nearer
    // than `GAP` (each kept off the other, one backing up and one it had turned round for or
    // turning round itself stood there for good: Elm Park, Willowbrook). Even so not nearer the
    // ground past where that one stands now, still to back over: going on by is past its front,
    // not cutting in behind its tail (a taxi came by one backing round a junction's turn, cut in
    // behind it into its turn and stood there, each waiting for the other for good, Willowby).
    // Nor, going the way it backs, on past it (`overtakes`): crept along beside the ground it backs
    // over, a car came out ahead of it into its way. One itself backing up on its turn round is
    // kept off others' only by their bodies (`guard`): their ways back were found clear of each
    // other as each turned round, and kept off each other's ground too, two backing up stood
    // facing each other for good, Oakton.
    if let Some(k) = traffic
        .turning
        .iter()
        .filter(|_| !turning)
        .find_map(|(k, path)| {
            if off_level.contains(k) {
                return None;
            }
            let on = if waiting.contains(k) {
                path.iter()
                    .position(|r| path[0].overlap(r) <= 0.0)
                    .unwrap_or(path.len())
            } else {
                0
            };
            (*k != key
                && (overtakes(path[0], &body_now, &body_next, me.heading())
                    || path[on..].iter().any(|r| {
                        crate::travel::dist(r.at, body_next.at)
                            < r.half[0] + body_next.half[0] + 1.0
                            && {
                                // (Kept `TURN_KEEP` off it, so the one backing up comes by clear.)
                                let after = body_next.gap(r);
                                after < GAP + TURN_KEEP && after < body_now.gap(r) - 1e-3
                            }
                    })))
            .then_some(*k)
        })
    {
        car.waits_for = Some(k);
        return;
    }
    let nav = crate::path::NavGrid {
        lot,
        blocked,
        stairs,
        kept_off: &[],
        steps: None,
    };
    let mut progress = t;
    let held_up = go_on(
        sims,
        &nav,
        (content, objects),
        me,
        now,
        &others,
        &mut car.waited,
        (&mut progress, step),
        &mut car.on_foot,
    );
    if held_up {
        car.waits_for = against(&me, &others).next().map(|k| others[k].first);
    }
    if progress > t {
        // (The last resort: never deeper into another car than it is.)
        let standing: Vec<Driving> = parked(cars).collect();
        let car = &mut cars[c];
        match guard(&body_now, &body_next, layer, others.iter().chain(&standing)) {
            Some(k) => {
                traffic.guarded += 1;
                car.waits_for = Some(k);
            }
            None => {
                car.progress = progress;
                car.moved = true;
                car.speed = eased.as_ref().map(|e| e.speed);
            }
        }
    }
    // (Where it is now, for those after it this tick: one coming in behind it sees it there.)
    let car = &mut cars[c];
    let progress = car.progress;
    let shift = car.shift;
    if progress > t && progress < ticks {
        let (p, yaw, _) = car.route(exits, Some(travel), home).at(progress);
        car.shown = Some((shifted(p, yaw, shift), yaw, true, 0));
    }
}

/// The ground a car going `speed` (m a second of drive time, `0`: standing or setting off) keeps
/// clear in front of it (m): `AHEAD`, as ever, up to a country road's speed (`country`, and
/// `FOLLOW_SLACK` more: every classic town's cars keep it as they always did); faster (the
/// motorway), `FOLLOW` s of drive for each m/s over that more. At 30 m/s some 28 m: a car on the
/// motorway keeps that far back from the one in front as a driver would, and slows down to stop
/// behind one stopped (`ease`, at a speed of its own there; held up at once only `AHEAD` short of
/// another, the last resort).
pub(crate) fn follow_ahead(speed: f32, country: f32) -> f32 {
    AHEAD + FOLLOW * (speed - country - FOLLOW_SLACK).max(0.0)
}

/// `follow_ahead`'s seconds of drive kept back for each m/s over a country road's speed, and how
/// far over it (m/s) that begins.
pub(crate) const FOLLOW: f32 = 2.0;
pub(crate) const FOLLOW_SLACK: f32 = 1.0;

/// How long a car stands at a zebra's stop line for those stepping onto it at most, and for those
/// on it (ticks of real time, as they walk: `zebra_yield`). A stream of them coming up to it, it
/// drives on once those on it are across, and the next wait for it to pass, as anywhere; and
/// anyone standing about on it, it comes on up to as to anyone in its way (`go_on`), not held at
/// the line for good.
pub(crate) const ZEBRA_KERB_WAIT: u32 = secs(6.0) as u32;
pub(crate) const ZEBRA_WAIT: u32 = secs(30.0) as u32;

/// Whom a car `t` ticks along `route` stops for at a zebra of a junction giving way
/// (`traffic::ZebraStop`), its nose at the stop line or a tick short of it, having stood there
/// `waited` ticks: those on foot on the zebra of the arm it comes in by or stepping onto it, and
/// those on the one of the arm it goes out by (`Traffic::zebra_walkers`), each for as long as
/// `ZEBRA_KERB_WAIT` and `ZEBRA_WAIT` say. None: at no zebra's line. Empty: it drives on. (Not one
/// stepping onto the zebra it goes out by: out in the box by then, it'd stand there in the way of
/// the others; they wait for it to pass.) At a lit junction, turning into a side road whose
/// crosswalk is walked on its green (`Signal::opens_with`), those on that one, while its light is
/// `green` (on red it waits for the green as ever, asking for it).
pub(crate) fn zebra_yield(
    route: &Route,
    walkers: &[crate::traffic::OnZebra],
    t: f32,
    waited: u32,
    green: impl Fn(crate::traffic::Gate) -> bool,
) -> Option<Vec<u32>> {
    let mut at_line = false;
    let mut whom = Vec::new();
    for z in route
        .zebras
        .iter()
        .filter(|z| t <= z.at + 1e-3 && z.at < t + 1.0)
        .filter(|z| z.lit.is_none_or(|phase| green((z.junction, phase))))
    {
        at_line = true;
        for w in walkers {
            let (a, id, on) = (w.arm, w.id, w.on);
            if w.junction != z.junction || w.lit != z.lit.is_some() || whom.contains(&id) {
                continue;
            }
            let into = z.arms[0] == Some(a)
                && if on {
                    waited < ZEBRA_WAIT
                } else {
                    waited < ZEBRA_KERB_WAIT
                };
            let out = z.arms[1] == Some(a) && on && waited < ZEBRA_WAIT;
            if into || out {
                whom.push(id);
            }
        }
    }
    at_line.then_some(whom)
}

/// Car `c` out of sight (`Sight::Ghost`) a tick on along its way (`home`: back): through anyone,
/// claiming nothing; back into its lane as it goes.
pub(crate) fn drive_through(w: &mut World, c: usize, home: bool) {
    let World {
        cars,
        exits,
        travel,
        ..
    } = w;
    let car = &mut cars[c];
    car.unhold();
    let t = car.progress;
    let shift = car.shift;
    let route = car.route(exits, Some(travel), home);
    let ticks = route.ticks();
    let shift = (shift - PULL_RATE).min(route.room_at(t + 1.0)).max(0.0);
    let (p, yaw, _) = route.at(t + 1.0);
    let at = shifted(p, yaw, shift);
    car.shift = shift;
    car.next = Some((at, yaw));
    car.next_ground = forward_ground();
    car.progress = t + 1.0;
    car.moved = true;
    if car.progress < ticks {
        car.shown = Some((at, yaw, true, 0));
    }
}

/// A work taxi whose run is over, given up in sight (`Car::off_run`), a tick on: it turns round
/// (`turn_round`, as one held up does, tried each `UNJAM_AGAIN`) and drives out of town, where
/// it's parked off the map (`Car::park_taxi`).
pub(crate) fn drive_off(w: &mut World, c: usize) {
    let tick = w.tick;
    if w.cars[c].detour.is_none()
        && (tick.is_multiple_of(UNJAM_AGAIN) || w.cars[c].stall.1 + 1 >= tick)
        && let Ok((_, d)) = turn_round(w, c)
    {
        turn_round_by(w, c, d);
    }
    drive_taxi(w, c);
    if w.cars[c].progress >= w.cars[c].taxi_ticks() {
        w.cars[c].park_taxi();
    }
}

/// A work taxi a tick on along its run (`taxi/`): as any car drives on.
pub(crate) fn drive_taxi(w: &mut World, c: usize) {
    w.cars[c].standing = false;
    drive_on(w, c);
}

/// A work taxi standing at a stop (`taxi/`): parked beside the road (`park_beside`); where it
/// can't, pulled over for one coming the other way as far as it has room, as any car standing.
/// Whether it's still pulling in to park (nobody gets in or out yet).
pub(crate) fn stand_taxi(w: &mut World, c: usize) -> bool {
    // (Standing, it holds no claim but the one it stands in: none on ahead.)
    let t = w.cars[c].progress;
    w.traffic.stand(car_first(c), t);
    let car = &mut w.cars[c];
    car.standing = true;
    car.waits_for = None;
    // (Standing on an avenue, it sets off from standing, `ease`.)
    let fast = car
        .route
        .as_ref()
        .is_some_and(|(_, r)| r.kind_at(t).is_some());
    car.speed = (fast || car.speed.is_some()).then_some(0.0);
    // (Where it stands, for its doors and seats: `Car::door`, `getting`.)
    if let Some((_, route)) = car.route.as_ref() {
        let (p, yaw, _) = route.at(car.progress);
        car.spot = shifted(p, yaw, car.shift);
        car.yaw = yaw;
    }
    // (Out of sight, back in its lane: nobody to pull over for.)
    if car.sight != Sight::Seen {
        car.unhold();
        car.shift = (car.shift - PULL_RATE).max(0.0);
        return false;
    }
    match park_beside(w, c) {
        Some(pulling) => pulling,
        None => {
            let others = vehicles(&w.cars, Some(c));
            pull_over(w, c, &others, false, None);
            false
        }
    }
}
