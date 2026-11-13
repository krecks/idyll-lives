//! The tick’s driving: owners and trips, the way on, holding up, pulling over, sight.
//!
//! Split from `cars.rs` (see `mod.rs`).

use super::*;

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
        refresh(w);
    }
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
        // (A work taxi is the taxis' own: `taxi.rs`; but one driving off, its run given up in
        // sight.)
        if w.cars[c].taxi {
            if w.cars[c].off_run {
                drive_off(w, c);
            }
            continue;
        }
        owners(w, c);
        relocate(w, c);
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
            trip_update(w, c);
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
                    .map(|u| u + OFF_MAP_TICKS)
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
    reblock(w);
    poses(w);
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

/// A tick further along its drive, unless someone is in its way: it waits for them (`go_on`).
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
            .get((k & !(1 << 63)) as usize)
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
                    cars.get((k & !(1 << 63)) as usize)
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
    let standing: Vec<Driving> = parked(&w.cars).collect();
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
            cars.get((k & !(1 << 63)) as usize)
                .is_some_and(|o| o.waits_for == Some(key))
                || traffic.waits_for(k).contains(&key)
        })
        .collect();
    let car = &mut cars[c];
    let shift = car.shift;
    // Where it would be a tick on, its body and the ground in front of it (behind it while it
    // backs up): not round its tail, as people may follow it close (on their way the same way),
    // but all along its sides, as its tail swings out on a turn.
    let route = car.route(exits, Some(travel), home);
    let ((at, yaw, reverse), (was, was_yaw, _)) = (route.at(t + 1.0), route.at(t));
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
    let (ahead, behind) = if reverse { (0.0, back) } else { (AHEAD, 0.0) };
    let me = Driving {
        at,
        yaw,
        len: car.len,
        wid: car.wid,
        ahead,
        behind,
        first: key,
    };
    let now = Driving {
        at: was,
        yaw: was_yaw,
        ..me
    };
    car.next = Some((at, yaw));
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
        .blocked(key, &body_now, &body_next, me.heading(), pass, in_drive)
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
    // over, a car came out ahead of it into its way.
    if let Some(k) = traffic.turning.iter().find_map(|(k, path)| {
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
                    crate::travel::dist(r.at, body_next.at) < r.half[0] + body_next.half[0] + 1.0
                        && {
                            // (Kept `TURN_KEEP` off it, so the one backing up comes by clear.)
                            let after = body_next.gap(r);
                            after < GAP + TURN_KEEP && after < body_now.gap(r) - 1e-3
                        }
                })))
        .then_some(*k)
    }) {
        car.waits_for = Some(k);
        return;
    }
    let nav = crate::path::NavGrid {
        lot,
        blocked,
        stairs,
        kept_off: &[],
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
        &mut progress,
        &mut car.on_foot,
    );
    if held_up {
        car.waits_for = against(&me, &others).next().map(|k| others[k].first);
    }
    if progress > t {
        // (The last resort: never deeper into another car than it is.)
        match guard(&body_now, &body_next, others.iter().chain(&standing)) {
            Some(k) => {
                traffic.guarded += 1;
                car.waits_for = Some(k);
            }
            None => {
                car.progress = progress;
                car.moved = true;
            }
        }
    }
    // (Where it is now, for those after it this tick: one coming in behind it sees it there.)
    let progress = car.progress;
    let shift = car.shift;
    if progress > t && progress < ticks {
        let (p, yaw, _) = car.route(exits, Some(travel), home).at(progress);
        car.shown = Some((shifted(p, yaw, shift), yaw, true, 0));
    }
}

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

/// A work taxi a tick on along its run (`taxi.rs`): as any car drives on.
pub(crate) fn drive_taxi(w: &mut World, c: usize) {
    w.cars[c].standing = false;
    drive_on(w, c);
}

/// A work taxi standing at a stop (`taxi.rs`): pulled over for one coming the other way as far as
/// it has room, as any car standing.
pub(crate) fn stand_taxi(w: &mut World, c: usize) {
    // (Standing, it holds no claim but the one it stands in: none on ahead.)
    let t = w.cars[c].progress;
    w.traffic.stand(car_first(c), t);
    let car = &mut w.cars[c];
    car.standing = true;
    car.waits_for = None;
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
        return;
    }
    let others = vehicles(&w.cars, Some(c));
    pull_over(w, c, &others, false, None);
}

/// How long a car and one in its way wait for each other before it backs up (ticks of drive time,
/// `back_off`), and how far back it goes at most (ticks along its route, a tick back each tick).
pub(crate) const JAM_TICKS: u32 = drive_secs(5.0) as u32;
pub(crate) const JAM_BACK: u32 = drive_secs(3.0) as u32;

/// Whether car `k` (its `Driving::first`) waited for car `me` last tick, so that car `me`, waiting
/// for it, should back up (`back_off`): of two cars, the one not holding a claim (holding one,
/// the other was let through where it waits: it's that one that's in the way); else the later.
pub(crate) fn waits_back(w: &World, k: u64, me: u64) -> bool {
    let o = (k & !(1 << 63)) as usize;
    let (mine, theirs) = (w.traffic.holding(me), w.traffic.holding(k));
    // (Turning out of its driveway, it has its way: the other backs up to give it room to finish
    // its turn.)
    let (out, their_out) = (turning_out(w, me), turning_out(w, k));
    // (Of two not holding one, the one waiting its turn for a claim the other is in the way of:
    // that one can't go on out of its way till it's backed up out of that one's.)
    let (asks, their_asks) = (
        !w.traffic.waits_for(me).is_empty(),
        !w.traffic.waits_for(k).is_empty(),
    );
    // (Of the two, the one that can still back up: the other can't, or no further.)
    let mine_to_go = if out != their_out {
        their_out
    } else if mine != theirs {
        theirs
    } else if !mine && asks != their_asks {
        asks
    } else {
        me > k
    };
    // (One backing up out of the other's way already goes on: not the other in turn as their
    // claims change, the one going on by letting go of its claim behind it, say; nor one, then the
    // other, a tick each.)
    let backing = |x: u64| {
        w.cars
            .get((x & !(1 << 63)) as usize)
            .is_some_and(|c| c.jam.1 > 0)
            && !stuck_back(w, x)
    };
    let mine_to_go = match (backing(me), backing(k)) {
        (true, false) => true,
        (false, true) => false,
        _ => mine_to_go,
    };
    let mine_to_go = if mine_to_go {
        !stuck_back(w, me)
    } else {
        stuck_back(w, k) && !stuck_back(w, me)
    };
    mine_to_go
        && (w.cars.get(o).is_some_and(|c| c.waits_for == Some(me))
            || w.traffic.waits_for(k).contains(&me))
}

/// Whether car `k` (its `Driving::first`) is a household's car still turning out of its driveway
/// onto the road: in its route's first mouth (`Route::mouths`).
pub(crate) fn turning_out(w: &World, k: u64) -> bool {
    w.cars.get((k & !(1 << 63)) as usize).is_some_and(|c| {
        !c.taxi
            && c.state == CarState::Leaving
            && c.route.as_ref().is_some_and(|(_, r)| {
                r.mouths
                    .first()
                    .is_some_and(|&(a, b)| a <= 0.0 && c.progress <= b)
            })
    })
}

/// Whether car `k` (its `Driving::first`) can back up no further out of the way of one it holds
/// up as that one holds it up (`back_off`, `waits_back`).
pub(crate) fn stuck_back(w: &World, k: u64) -> bool {
    w.cars
        .get((k & !(1 << 63)) as usize)
        .is_some_and(|c| c.jam.2)
}

/// Car `c` a tick back along its route, out of the way of one it holds up as that one holds it
/// up (`JAM_TICKS`): never into another (`guard`) nor into what's left of another's claim
/// (`traffic::Traffic::blocked`). Whether it moved.
pub(crate) fn back_off(w: &mut World, c: usize) -> bool {
    let key = car_first(c);
    let others = vehicles(&w.cars, Some(c));
    let standing: Vec<Driving> = parked(&w.cars).collect();
    let car = &w.cars[c];
    let Some((home, route)) = car.route.as_ref() else {
        return false;
    };
    let (t, shift, half) = (car.progress, car.shift, [car.len / 2.0, car.wid / 2.0]);
    let to = (t - 1.0).max(0.0);
    // (Turned round, still on the stretch it backs up along, its last fraction of a tick before it
    // drives on: a tick back along it is forward, the way it backed, and there it's turning round
    // again (`drive_on`), with no jam rule of its own: it'd neither back up nor say it can't, and
    // the other would wait for it for good. It can back up no further: the other does.)
    let turned = car.detour.is_some() && route.at(t).2;
    if to >= t || route.at(to).2 != route.at(t).2 || turned {
        return false;
    }
    let body = |t: f32| {
        let (p, yaw, _) = route.at(t);
        Rect {
            at: shifted(p, yaw, shift),
            yaw,
            half,
        }
    };
    let (now, next) = (body(t), body(to));
    // (Not back into a junction's box it's out of: it holds no claim to it, and standing there it
    // would hold up those taking their turn through it, one it's held up with among them: they'd
    // wait for each other for good, Rowan Hollow.)
    let boxes = &w.travel.graph.boxes;
    if in_box(boxes, next.at, car.len) && !in_box(boxes, now.at, car.len) {
        return false;
    }
    let way = [-next.yaw.sin(), -next.yaw.cos()];
    // (Back out of the claim of the one it's held up with, too: away from it is out of its way.)
    let partner = car.waits_for;
    if guard(&now, &next, others.iter().chain(&standing)).is_some()
        || w.traffic
            .blocked(key, &now, &next, way, 0.0, false)
            .is_some_and(|k| Some(k) != partner)
    {
        return false;
    }
    let home = *home;
    let car = &mut w.cars[c];
    car.progress = to;
    car.jam.1 += 1;
    car.backed_for = partner;
    if home == (car.state == CarState::Arriving) {
        car.shown = Some((next.at, next.yaw, true, 0));
    }
    true
}

/// How far right of its lane car `car` has room to pull over `t` ticks along its route
/// (`Route::room_at`).
pub(crate) fn route_room(car: &Car, t: f32) -> f32 {
    car.route.as_ref().map_or(0.0, |(_, r)| r.room_at(t))
}

/// How far car `car` can pull over `t` ticks along its route (`Route::pass_at`).
pub(crate) fn route_pass(car: &Car, t: f32) -> f32 {
    car.route.as_ref().map_or(0.0, |(_, r)| r.pass_at(t))
}

/// The last-resort check on a car's move from body `now` to `next`: the first of `them` (the cars
/// on the road and parked) it would come nearer than `GAP` to, and nearer than it is now (one it's
/// already that near, from before, it may draw away from or keep beside, never nearer). None: it
/// may move.
pub(crate) fn guard<'a>(
    now: &Rect,
    next: &Rect,
    them: impl Iterator<Item = &'a Driving>,
) -> Option<u64> {
    let near = |at: [f32; 2], len: f32| {
        crate::travel::dist(at, next.at) < (len + next.half[0] * 2.0) + AHEAD + GAP
    };
    them.into_iter()
        .find(|o| near(o.at, o.len) && closes_in(now, next, &o.body()))
        .map(|o| o.first)
}

/// Whether a car going `heading`, its move from body `now` to `next`, goes on past one turning
/// round as it backs (`turning`, where it is now, `Traffic::turning`): going the way that one
/// backs, beside it or behind it (within a lane or so either side), its front further past that
/// one's nose and nearer than `GAP` behind it or beyond. (Ahead of it already, it's on its way out
/// of the way: `drive_on`'s keep off the ground still backed over.)
pub(crate) fn overtakes(turning: Rect, now: &Rect, next: &Rect, heading: [f32; 2]) -> bool {
    let back = [-turning.yaw.sin(), -turning.yaw.cos()];
    let rel = |r: &Rect| sub2(r.at, turning.at);
    let (d_now, d_next) = (rel(now), rel(next));
    let front = |d: [f32; 2], r: &Rect| dot2(d, back) + r.half[0];
    let nose = -turning.half[0] - GAP;
    dot2(heading, back) > 0.5
        && cross2(back, d_next).abs() < turning.half[1] + next.half[1] + 2.0 * LANE
        && dot2(d_now, back) < 0.0
        && front(d_next, next) > nose
        && front(d_next, next) > front(d_now, now) + 1e-3
}

/// Whether a car's move from body `now` to `next` brings it nearer `o` than `GAP`, and nearer than
/// it is (`guard`).
pub(crate) fn closes_in(now: &Rect, next: &Rect, o: &Rect) -> bool {
    let after = next.gap(o);
    after < GAP - 1e-3 && after < now.gap(o) - 1e-3
}

/// Car `c` a tick nearer pulled over as far as it would be (`pull_want`), `PULL_RATE` a tick:
/// back towards its lane only clear of those coming the other way (`too_near`), and never
/// into another (`guard`). Where its room runs out it's back in its lane.
pub(crate) fn pull_over(
    w: &mut World,
    c: usize,
    others: &[Driving],
    moving: bool,
    waited: Option<u64>,
) {
    // (Pulled over for one that waits for it, it pulls over only as far as it can on its way: not
    // for one it waits for in turn (`waited`, or its turn after it), held up by it there, which it
    // would pull back in into the way of, unless that one has backed up clear of it to give it the
    // way.)
    let me = car_first(c);
    // (Not one backed up out of its way, clear of it now by more than it's pulled over and the
    // gap the cars keep: that one gives it the way, and it pulls back in to go on.)
    let body = |car: &Car| {
        car.shown.map(|s| Rect {
            at: s.0,
            yaw: s.1,
            half: [car.len / 2.0, car.wid / 2.0],
        })
    };
    let backed = |k: u64| {
        w.cars.get((k & !(1 << 63)) as usize).is_some_and(|o| {
            o.jam.1 > 0
                && body(o)
                    .zip(body(&w.cars[c]))
                    .is_some_and(|(a, b)| a.gap(&b) > w.cars[c].shift + GAP)
        })
    };
    let after = |k: u64| (waited == Some(k) || w.traffic.waits_for(me).contains(&k)) && !backed(k);
    let waits_me = |k: u64| {
        (w.traffic.waits_for(k).contains(&me)
            || w.cars
                .get((k & !(1 << 63)) as usize)
                .is_some_and(|o| o.waits_for == Some(me)))
            && !after(k)
    };
    let held = w.cars[c].pulled_for.is_some_and(waits_me);
    let (mut want, mut whom) = pull_want(w, c, moving || held, &after);
    if !(moving || held) && whom.is_some_and(waits_me) {
        (want, whom) = pull_want(w, c, true, &after);
    }
    w.cars[c].pulled_for = whom;
    let standing: Vec<Driving> = parked(&w.cars).collect();
    let car = &w.cars[c];
    let Some((_, route)) = car.route.as_ref() else {
        return;
    };
    let t = car.progress;
    let room = route.room_here(t);
    let (p, yaw, _) = route.at(t);
    let half = [car.len / 2.0, car.wid / 2.0];
    let body = |s: f32| Rect {
        at: shifted(p, yaw, s),
        yaw,
        half,
    };
    let rate = PULL_RATE;
    let mut shift = car.shift + (want - car.shift).clamp(-rate, rate);
    // (Back towards its lane only clear of those coming the other way; but for one waiting for
    // it, which won't pass: it pulls back in to go on by, never into it, `guard`.)
    let (at, len, wid) = (body(shift).at, car.len, car.wid);
    let key = car_first(c);
    let waits_for_me = |o: &Driving| {
        w.traffic.waits_for(o.first).contains(&key)
            || w.cars
                .get((o.first & !(1 << 63)) as usize)
                .is_some_and(|x| x.waits_for == Some(key))
    };
    if shift < car.shift
        && let Some(o) = others.iter().find(|o| {
            too_near((at, yaw, len, wid), (o.at, o.yaw, o.len, o.wid)) && !waits_for_me(o)
        })
    {
        shift = car.shift;
        // (It waits for that one to pass: one that waits for it goes by, `drive_on`.)
        w.cars[c].waits_for = Some(o.first);
    }
    let car = &w.cars[c];
    // (Its room run out: back in its lane, whatever.)
    let shift = shift.min(room).max(0.0);
    if (shift - car.shift).abs() < 1e-5 {
        return;
    }
    // (Out, or back in: never into another, one going by that waited for it too. Kept from
    // pulling back in by one, it waits for that one: who waits for whom sees it, `back_off`.)
    if let Some(k) = guard(
        &body(car.shift),
        &body(shift),
        others.iter().chain(&standing),
    ) {
        if shift < car.shift {
            w.cars[c].waits_for = Some(k);
        }
        return;
    }
    // (Where it is now, for those after it this tick, as `drive_on` has it once on its way.)
    let at = body(shift).at;
    let car = &mut w.cars[c];
    car.shift = shift;
    if let Some(s) = car.shown.as_mut() {
        s.0 = at;
    }
}

/// Room to spare (m) in `Near::reach` for how far a car may move between `cars::update` building
/// `Near` and one looking (`drive_on` a tick on, `back_off` a tick back).
pub(crate) const PULL_SLACK: f32 = 2.0;

/// Cars by where they are, so a check that pairs one with another (`pull_want`) need only look at
/// those near it, not at every car in town. `cars::update` builds it each tick; `turn_round_by`
/// puts one back as it's put back along its way (tens of metres at once).
#[derive(Default)]
pub(crate) struct Near {
    /// The cars in it (their middles, as last put), by the x of their middles.
    by_x: Vec<([f32; 2], usize)>,
    /// How many cars it was built from: any since (a taxi setting off mid-tick) aren't in it, and
    /// are looked at whatever.
    built: usize,
    /// The longest and widest car in, for how far a check reaches (`reach`).
    most_len: f32,
    most_wid: f32,
}

/// Orders `by_x`: by x, then by car (the same order whatever order they were put in).
fn by_x_order(a: &([f32; 2], usize), b: &([f32; 2], usize)) -> std::cmp::Ordering {
    a.0[0].total_cmp(&b.0[0]).then(a.1.cmp(&b.1))
}

impl Near {
    /// Builds it afresh from `cars`.
    fn build(&mut self, cars: &[Car]) {
        self.by_x.clear();
        self.by_x
            .extend(cars.iter().enumerate().map(|(c, car)| (car.middle(), c)));
        self.by_x.sort_unstable_by(by_x_order);
        self.built = cars.len();
        self.most_len = cars.iter().map(|c| c.len).fold(0.0, f32::max);
        self.most_wid = cars.iter().map(|c| c.wid).fold(0.0, f32::max);
    }

    /// Puts car `c` at `at` (as it is now). One not in it is looked at whatever (`near`).
    fn put(&mut self, c: usize, at: [f32; 2]) {
        let Some(k) = self.by_x.iter().position(|&(_, o)| o == c) else {
            return;
        };
        self.by_x.remove(k);
        let k = self
            .by_x
            .partition_point(|e| by_x_order(e, &(at, c)).is_lt());
        self.by_x.insert(k, (at, c));
    }

    /// How far apart (m, their middles) `car` and another can be and still have one pull over for
    /// the other (`pull_want`): their bodies end to end, `PULL_AHEAD` of the way ahead it looks,
    /// the gap it keeps (`PASSING`, `PULL_MARGIN`), and `PULL_SLACK` for a tick's driving either
    /// way.
    fn reach(&self, car: &Car) -> f32 {
        car.len
            + self.most_len
            + (car.wid + self.most_wid) / 2.0
            + PULL_AHEAD
            + PASSING
            + PULL_MARGIN
            + PULL_SLACK
    }

    /// Of the town's `cars`, those that may be within `reach` (m) of `at`: those in it whose
    /// middles are (checked all ways), and those not in it.
    fn near(&self, at: [f32; 2], reach: f32, cars: usize) -> impl Iterator<Item = usize> + '_ {
        // (Fewer cars than it was built from: their numbers have moved, so every one.)
        let (window, rest) = if self.built <= cars {
            (
                along_x(&self.by_x, |e| e.0[0], at[0], reach),
                self.built..cars,
            )
        } else {
            (&self.by_x[..0], 0..cars)
        };
        window
            .iter()
            .filter(move |&&(p, _)| crate::travel::dist(p, at) <= reach)
            .map(|&(_, c)| c)
            .chain(rest)
    }
}

/// How far right of its lane car `c` would be pulled over (m) to pass those coming the other way in
/// front of it (`PULL_AHEAD`) where the lane is too narrow for both in their lanes: half the
/// way past another car (it pulls over the other half, as far as it can: `Route::room`); as far as
/// its room ahead allows (back in its lane before the room runs out). Standing, as far as it can
/// for one it waits for (`after`).
pub(crate) fn pull_want(
    w: &World,
    c: usize,
    moving: bool,
    after: &dyn Fn(u64) -> bool,
) -> (f32, Option<u64>) {
    let car = &w.cars[c];
    let Some((_, route)) = car.route.as_ref() else {
        return (0.0, None);
    };
    let t = car.progress;
    // (On the move, back in its lane before its room runs out; standing, as far as it has room
    // where it stands.)
    let room = if moving {
        [0.0, 1.0, drive_ticks(0.5), drive_ticks(1.0)]
            .into_iter()
            .map(|k| route.room_at(t + k))
            .fold(f32::INFINITY, f32::min)
    } else {
        route.room_here(t)
    };
    let (at, yaw, back) = route.at(t);
    // (In a junction's box only for one coming the other way beside it now, their bodies nearer
    // than `GAP` and `PASSING`: there they take turns, `traffic.rs`, and turning there it faces
    // any way, those following it in too; but two let in together pass each other in their lanes,
    // and round a bend there their ends swing across towards each other further than the claims'
    // samples have it: neither could go on, `guard`. For one coming the other way into it, as
    // anywhere: the claims let that one in counting on this one's pulling over.)
    let boxed = in_box(&w.travel.graph.boxes, at, car.len);
    if room <= 0.0 || back {
        return (0.0, None);
    }
    let body = |p: [f32; 2], yaw: f32, len: f32, wid: f32| Rect {
        at: p,
        yaw,
        half: [len / 2.0, wid / 2.0],
    };
    let mine = body(at, yaw, car.len, car.wid);
    let short = |p: [f32; 2], facing: f32, len: f32, wid: f32| {
        let m = passing((at, yaw, car.len, car.wid), (p, facing, len, wid))?;
        let ends = (car.len + len) / 2.0;
        let ahead = if boxed { 0.0 } else { PULL_AHEAD };
        if m.along < -ends || m.along > ends + ahead || m.apart < 0.0 {
            return None;
        }
        // (Beside it, as near as their bodies are: on a bend nearer than across their ways.)
        let gap = m.gap.min(mine.gap(&body(p, facing, len, wid)));
        if boxed && gap >= GAP + PASSING {
            return None;
        }
        Some(((PASSING + PULL_MARGIN - gap).max(0.0), m.mine, m.theirs))
    };
    let (mut want, mut whom): (f32, Option<u64>) = (0.0, None);
    // (Those near enough to be passing it: checking every car in town pairs every one with every
    // other, `Near`.)
    let reach = w.cars_near.reach(car);
    for o in w.cars_near.near(at, reach, w.cars.len()) {
        let other = &w.cars[o];
        // (Standing, waiting for that one (`after`: its turn after it too), it pulls over as far as
        // it can, that one backing out of its driveway too: that one goes by, as it comes on round
        // a bend or out of a driveway.)
        let after = !moving && !boxed && after(car_first(o));
        // (A taxi standing at the kerb for its riders too, once it waits for it: it pulls over as
        // far as it can to get by, `stand_taxi`.)
        let standing_taxi = other.taxi && other.standing && after;
        if o == c
            || other.shown.is_none_or(|s| !s.2) && !standing_taxi
            || other.backing() && !after
            || other.sight != Sight::Seen
        {
            continue;
        }
        let Some((_, theirs)) = other.route.as_ref() else {
            continue;
        };
        let (p, facing, _) = theirs.at(other.progress);
        if let Some((all, mine, their)) = short(p, facing, other.len, other.wid) {
            let share = if after {
                f32::INFINITY
            } else {
                (all / 2.0).max(all - theirs.room_at(other.progress) * their)
            };
            // (Of those wanting it as much, the first car: `near` goes by where they are, not
            // by their numbers.)
            let v = share / mine;
            if v > want || (v == want && v > 0.0 && whom.is_some_and(|k| car_first(o) < k)) {
                (want, whom) = (v, Some(car_first(o)));
            }
        }
    }
    (want.min(room), whom)
}

/// Whether a body `len` long with its middle at `p` is in one of the junctions' `boxes`, about.
pub(crate) fn in_box(boxes: &[([f32; 2], f32)], p: [f32; 2], len: f32) -> bool {
    boxes
        .iter()
        .any(|&(m, r)| crate::travel::dist(m, p) < r + len / 2.0)
}

/// Which cars on their way are in sight this tick (`Car::sight`, before the claims: `traffic::begin`;
/// see the module notes). With a watch (`World::set_watch`), one whose body is in the seen area
/// (the watch and `SEEN_MARGIN` round it), now or soon (`COMING_IN`, or a claim of its reaching
/// into it), is seen if it was (and till it's `SEEN_SLACK` out of it); one coming into it is seen
/// once clear of every car seen (`GAP`; those in sight first, then the others in turn as they are
/// now), else it waits outside for the one it isn't clear of (`Sight::Wait`), or, the seen area
/// come to it where it touches one, is put where it's clear (`moved_clear`). The rest are out
/// of sight (`Sight::Ghost`); their claims are let go. Without a watch the seen area is the whole
/// town.
pub(crate) fn sight(w: &mut World) {
    // (No watch: every car in sight, as it always was; but one out of sight on top of another
    // still till it's clear of them.)
    if w.watch.is_none() && w.cars.iter().all(|c| c.sight == Sight::Seen) {
        return;
    }
    let m = SEEN_MARGIN;
    let area = w
        .watch
        .map(|[x0, z0, x1, z1]| (x0 as f32 - m, z0 as f32 - m, x1 as f32 + m, z1 as f32 + m));
    let near = |p: [f32; 2], reach: f32| {
        area.is_none_or(|(x0, z0, x1, z1)| {
            let dx = (x0 - p[0]).max(p[0] - x1).max(0.0);
            let dz = (z0 - p[1]).max(p[1] - z1).max(0.0);
            dx.hypot(dz) <= reach
        })
    };
    // (How far on along its way each may be put, `moved_clear`.)
    let stops = crate::taxi::stops_ahead(w);
    // (Those in sight first, which stay or go; then those out of it, coming in clear of them as
    // they are now.)
    let order: Vec<usize> = (0..w.cars.len())
        .filter(|&c| w.cars[c].sight == Sight::Seen)
        .chain((0..w.cars.len()).filter(|&c| w.cars[c].sight != Sight::Seen))
        .collect();
    for c in order {
        let most = stops[c];
        if !matches!(w.cars[c].state, CarState::Leaving | CarState::Arriving) {
            w.cars[c].sight = Sight::Seen;
            continue;
        }
        let home = w.cars[c].state == CarState::Arriving;
        let tick = w.tick;
        let World {
            cars,
            exits,
            travel,
            ..
        } = &mut *w;
        let car = &mut cars[c];
        let (t, shift, half) = (car.progress, car.shift, [car.len / 2.0, car.wid / 2.0]);
        let (was, shown) = (car.sight, car.shown);
        // (Its corners and the ground in front of it.)
        let reach = half[0] + half[1] + AHEAD;
        let route = car.route(exits, Some(travel), home);
        let (p, yaw, _) = route.at(t + 1.0);
        let next = Rect {
            at: shifted(p, yaw, shift),
            yaw,
            half,
        };
        let now = shown.map(|s| Rect {
            at: s.0,
            yaw: s.1,
            half,
        });
        // (In sight, it stays so till `SEEN_SLACK` further out.)
        let reach = if was == Sight::Seen {
            reach + SEEN_SLACK
        } else {
            reach
        };
        let in_now = now.is_some_and(|r| near(r.at, reach));
        // (Coming into sight, it's under the rules a little way short of it, `COMING_IN`: up to a
        // junction there taking turns, not cutting in part way across it. So too short of a claim
        // of its, there or that near, that reaches into sight: it asks for it as anyone there,
        // not driving through on into sight.)
        let last = COMING_IN[COMING_IN.len() - 1];
        let reaches = |k: &crate::traffic::Claim| {
            let (a, b) = (k.from.max(t), k.to);
            let n = ((b - a) / SIGHT_STEP).ceil().max(1.0) as usize;
            (0..=n).any(|i| near(route.at(a + (b - a) * i as f32 / n as f32).0, reach))
        };
        let coming = COMING_IN.iter().any(|&k| near(route.at(t + k).0, reach))
            || route
                .claims
                .iter()
                .filter(|k| k.to > t && k.from <= t + last)
                .any(reaches);
        let mut moved = None;
        let sight = if was == Sight::Seen {
            if in_now || coming {
                Sight::Seen
            } else {
                Sight::Ghost
            }
        } else if !in_now && !coming {
            Sight::Ghost
        } else {
            let on = |o: &Car| {
                o.shown.map(|s| Rect {
                    at: s.0,
                    yaw: s.1,
                    half: [o.len / 2.0, o.wid / 2.0],
                })
            };
            let seen = || {
                cars.iter()
                    .enumerate()
                    .filter(|&(o, other)| o != c && other.sight == Sight::Seen)
                    .filter_map(|(o, other)| Some((car_first(o), on(other)?)))
            };
            // (On top of one in sight where it is, or touching it, the seen area come to it (the
            // camera moved there, nobody has seen it yet): it's put back along its way, or on, to
            // the nearest place clear of those in sight (`moved_clear`), and comes into sight
            // there. Coming in, it waits outside till its way in is clear by `GAP`.)
            let on_top = now.is_some_and(|r| seen().any(|(_, b)| b.gap(&r) < TOUCH));
            let by = seen()
                .find(|(_, b)| b.gap(&next) < GAP || now.is_some_and(|r| b.gap(&r) < GAP))
                .map(|(k, _)| k);
            match by {
                Some(_) if in_now && on_top => {
                    let clear = |r: &Rect| {
                        seen().all(|(_, b)| {
                            crate::travel::dist(b.at, r.at) > b.half[0] + r.half[0] + GAP + 0.5
                                || b.gap(r) >= GAP
                        })
                    };
                    match cars[c]
                        .route
                        .as_ref()
                        .and_then(|(_, r)| moved_clear(r, t, half, &[-1.0, 1.0], most, &clear))
                    {
                        Some(to) => {
                            moved = Some(to);
                            Sight::Seen
                        }
                        None => Sight::Ghost,
                    }
                }
                // (Without a watch the way into town is in sight, where it waits its turn,
                // `room_to_come_in`.)
                Some(k) if !in_now && area.is_some() => Sight::Wait(k),
                // (Its way in held up just inside: out of sight nobody waits, so those coming
                // would all come in at once. It waits outside till the queue there has room for
                // it, as the rules would have let it come, `METER`.)
                None if !in_now && area.is_some() => {
                    let bodies: Vec<(u64, Rect)> = cars
                        .iter()
                        .enumerate()
                        .filter(|&(o, k)| {
                            o != c
                                && k.sight == Sight::Seen
                                && matches!(k.state, CarState::Leaving | CarState::Arriving)
                                && (k.standing || tick >= k.stall.1 + METER_STILL)
                        })
                        .filter_map(|(o, k)| Some((car_first(o), on(k)?)))
                        .collect();
                    let route = &cars[c].route.as_ref().expect("its way").1;
                    // (Its place on its way in, up to the first junction it takes turns at in
                    // sight: the queue for it.)
                    let upto = route
                        .claims
                        .iter()
                        .map(|k| k.from)
                        .find(|&f| f > t && near(route.at(f).0, 0.0))
                        .unwrap_or(t + METER.ahead)
                        .min(t + METER.most);
                    let full = (0..)
                        .map(|i| t + 1.0 + i as f32 * SIGHT_STEP)
                        .take_while(|&s| s < upto)
                        .find_map(|s| {
                            let (p, yaw, _) = route.at(s);
                            let r = Rect {
                                at: shifted(p, yaw, shift),
                                yaw,
                                half,
                            };
                            bodies
                                .iter()
                                .find(|(_, b)| b.gap(&r) < GAP + METER.gap)
                                .map(|(k, _)| *k)
                        });
                    // (Not for good: waited `METER_MOST`, it comes in as it would have.)
                    let since = *cars[c].metered.get_or_insert(tick);
                    match full {
                        Some(k) if tick < since + METER_MOST => Sight::Wait(k),
                        _ => Sight::Seen,
                    }
                }
                _ => Sight::Seen,
            }
        };
        if let Some(to) = moved {
            let car = &mut w.cars[c];
            car.progress = to;
            car.shift = 0.0;
            car.unhold();
            if let Some((_, r)) = car.route.as_ref() {
                let (p, yaw, _) = r.at(to);
                car.shown = Some((p, yaw, true, 0));
                car.next = Some((r.at(to + 1.0).0, r.at(to + 1.0).1));
            }
        }
        if !matches!(sight, Sight::Wait(_)) {
            w.cars[c].metered = None;
        }
        if sight != Sight::Seen && was == Sight::Seen {
            w.traffic.forget(car_first(c));
        }
        w.cars[c].sight = sight;
    }
    unstick(w);
    // (One in sight waits for, stays backed up for or pulled over for nobody out of it.)
    let out: Vec<bool> = w.cars.iter().map(|c| c.sight != Sight::Seen).collect();
    let gone =
        |k: &Option<u64>| k.is_some_and(|k| out.get((k & !(1 << 63)) as usize) == Some(&true));
    for car in w.cars.iter_mut().filter(|c| c.sight == Sight::Seen) {
        if gone(&car.waits_for) {
            car.waits_for = None;
        }
        if gone(&car.backed_for) {
            car.backed_for = None;
        }
        if gone(&car.pulled_for) {
            car.pulled_for = None;
        }
    }
}

/// How far along `route` (ticks) a car `half` its length and width, `t` along it, is put to come
/// into sight clear of those in sight (`clear`, now and a tick on), the camera come to it on top of
/// one of them (`sight`): the nearest such place back along its way, else on along it, up to
/// `MOVED_MOST` either way, out of its claims (part way through one, it would cut in), and no further on than `most` (a
/// work taxi's next stop, `taxi::stops_ahead`: put on past it, it stood there mid-turn). None:
/// none.
pub(crate) fn moved_clear(
    route: &Route,
    t: f32,
    half: [f32; 2],
    ways: &[f32],
    most: f32,
    clear: &dyn Fn(&Rect) -> bool,
) -> Option<f32> {
    let ticks = route.ticks();
    let claiming = |s: f32| route.claims.iter().any(|k| k.from + 1e-3 < s && s < k.to);
    let body = |s: f32| {
        let (at, yaw, _) = route.at(s);
        Rect { at, yaw, half }
    };
    (1..=MOVED_MOST)
        .flat_map(|d| ways.iter().map(move |k| t + k * d as f32))
        .filter(|&s| s >= 1.0 && s + 1.0 < ticks && s <= most)
        .find(|&s| !claiming(s) && clear(&body(s)) && clear(&body(s + 1.0)))
}

/// Cars held up in the seen area but out of the picture (further than `UNSEEN` from the watch, the
/// renderer's: `World::set_watch`), with all they wait for, standing still `STUCK_UNSEEN`: each is
/// put on along its way to the nearest place clear of those in sight (`moved_clear`), out of the
/// picture too and out of its claims, letting go of those it held. Nobody sees it go: a hold-up
/// nobody can see is gone, as one out of sight (`sight`), but no car drives through another where
/// the camera could come any moment.
pub(crate) fn unstick(w: &mut World) {
    let Some([x0, z0, x1, z1]) = w.watch else {
        return;
    };
    let tick = w.tick;
    let unseen = |p: [f32; 2]| {
        let dx = (x0 as f32 - p[0]).max(p[0] - x1 as f32).max(0.0);
        let dz = (z0 as f32 - p[1]).max(p[1] - z1 as f32).max(0.0);
        dx.hypot(dz) > UNSEEN
    };
    let on_way = |c: &Car| {
        matches!(c.state, CarState::Leaving | CarState::Arriving) && c.sight == Sight::Seen
    };
    let stuck = |c: &Car| {
        on_way(c) && tick >= c.stall.1 + STUCK_UNSEEN && c.shown.is_some_and(|s| s.2 && unseen(s.0))
    };
    let mut out = Vec::new();
    for c in 0..w.cars.len() {
        if !stuck(&w.cars[c]) {
            continue;
        }
        // (All it waits for, however far round, as held up and as far out of the picture.)
        let mut seen = vec![car_first(c)];
        let mut todo = vec![car_first(c)];
        let mut all = true;
        while let Some(k) = todo.pop() {
            for o in crate::traffic::waits_on(w, k) {
                if o < 1 << 63 || seen.contains(&o) {
                    continue;
                }
                let car = &w.cars[(o & !(1 << 63)) as usize];
                if !stuck(car) {
                    all = false;
                }
                seen.push(o);
                todo.push(o);
            }
        }
        if all {
            out.push(c);
        }
    }
    let stops = crate::taxi::stops_ahead(w);
    for c in out {
        let body = |o: &Car| {
            o.shown.map(|s| Rect {
                at: s.0,
                yaw: s.1,
                half: [o.len / 2.0, o.wid / 2.0],
            })
        };
        let others: Vec<Rect> = w
            .cars
            .iter()
            .enumerate()
            .filter(|&(o, k)| o != c && k.sight == Sight::Seen)
            .filter_map(|(_, k)| body(k))
            .collect();
        let car = &w.cars[c];
        let half = [car.len / 2.0, car.wid / 2.0];
        let clear = |r: &Rect| {
            unseen(r.at)
                && others.iter().all(|b| {
                    crate::travel::dist(b.at, r.at) > b.half[0] + r.half[0] + GAP + 0.5
                        || b.gap(r) >= GAP
                })
        };
        let Some(to) = car
            .route
            .as_ref()
            .and_then(|(_, r)| moved_clear(r, car.progress, half, &[1.0], stops[c], &clear))
        else {
            continue;
        };
        w.traffic.forget(car_first(c));
        w.traffic.unstuck += 1;
        let car = &mut w.cars[c];
        car.progress = to;
        car.shift = 0.0;
        car.unhold();
        car.stall = (to, tick);
        if let Some((_, r)) = car.route.as_ref() {
            let (p, yaw, _) = r.at(to);
            car.shown = Some((p, yaw, true, 0));
            let (q, qy, _) = r.at(to + 1.0);
            car.next = Some((q, qy));
        }
    }
}

/// Where each car on its way is furthest along it in sight and since when (`Car::stall`), before
/// the claims each tick (`unstick`, `unjam`): standing there since, it's held up.
pub(crate) fn track_stall(w: &mut World) {
    let tick = w.tick;
    for car in w.cars.iter_mut() {
        let going = matches!(car.state, CarState::Leaving | CarState::Arriving);
        if !going {
            car.detour = None;
            car.let_by = None;
        }
        let on_way = going && car.sight == Sight::Seen;
        // (Well back along its way: a new way, begun afresh.)
        let restarted = car.progress < car.stall.0 - drive_ticks(10.0);
        if !on_way || car.progress > car.stall.0 + 1e-3 || restarted {
            car.stall = (car.progress, tick);
        }
    }
}

/// How far ahead along its way (ticks of drive time) a car in sight is looked along for a work taxi
/// standing at the kerb in its lane, or alongside one the other way (`pass_stands`); one further
/// off than `PASS_FAR` (m: well past where a car gets in `PASS_LOOK` at any speed) isn't.
pub(crate) const PASS_LOOK: f32 = drive_ticks(45.0);
pub(crate) const PASS_FAR: f32 = 900.0;
/// How far apart (ticks: a tenth of a second of drive time) a way past a standing taxi is looked
/// along (`leg_past`, `span_by`), and how far on from where it comes up against it its middle is
/// looked for alongside the taxi's (ticks: 4 s of drive time).
pub(crate) const PASS_STEP: f32 = drive_ticks(0.1);
pub(crate) const PASS_ALONGSIDE: usize = drive_secs(4.0) as usize;
/// Passing a work taxi standing at the kerb in its lane (`PassBy`): how far clear of it its body
/// keeps alongside (m), how far behind it and past it (m) it's out of its lane all the way, and
/// how far it eases out before and back in after (m; out, `PASS_EASE_LEAST` at least, right
/// behind it).
pub(crate) const PASS_SIDE: f32 = 0.35;
pub(crate) const PASS_ENDS: f32 = 0.5;
pub(crate) const PASS_EASE: f32 = 8.0;
pub(crate) const PASS_EASE_LEAST: f32 = 3.0;
/// How long a car waits for its way round a work taxi standing in its lane, or behind one with
/// no way round it, before it turns round as one held up by a car parked in its way (ticks,
/// `unjam`; drive time).
pub(crate) const PASS_WAIT: u64 = drive_secs(3.0);

/// How much of its way out round a standing taxi a car is at `t` ticks along (`PassBy`: 0 in its
/// lane, 1 out alongside it), easing out and back in.
pub(crate) fn pass_share(pass: &PassBy, t: f32) -> f32 {
    let ease = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    };
    if t < pass.out {
        ease((t - pass.from) / (pass.out - pass.from))
    } else if t <= pass.back {
        1.0
    } else {
        1.0 - ease((t - pass.back) / (pass.to - pass.back))
    }
}

/// Work taxis standing at the kerb in sight (`stand_taxi`), before the claims each tick: each
/// such stretch of road is narrow for a while, taken in turns. A car in sight on its way up behind
/// one in its lane goes out round it into the other lane and back in past it (`PassBy`, laid on
/// its way: `Route::lay_leg`), a claim of its own; one coming the other way alongside it takes its
/// turn for that stretch (`Route::spans`), begun short of it as a narrow stretch's is: they go
/// through in the order they came, as on a 6 m lane's tight bend. Once the taxi pulls away, a car
/// not yet out round it keeps to its lane, and none takes turns there. (No way round it, or none
/// clear for `PASS_WAIT`: it turns round, `unjam`.) The cars are looked at again as the taxis
/// standing change (`Traffic::stand_gen`), or their ways do.
pub(crate) fn pass_stands(w: &mut World) {
    let stands: Vec<(usize, Rect)> = w
        .cars
        .iter()
        .enumerate()
        .filter(|(_, k)| {
            k.taxi
                && k.standing
                && k.sight == Sight::Seen
                && matches!(k.state, CarState::Leaving | CarState::Arriving)
        })
        .filter_map(|(o, k)| {
            // (Where it stands in its lane, as it stood still: not as it pulls over for one coming
            // the other way.)
            let (_, route) = k.route.as_ref()?;
            let (p, yaw, _) = route.at(k.progress);
            Some((
                o,
                Rect {
                    at: p,
                    yaw,
                    half: [k.len / 2.0, k.wid / 2.0],
                },
            ))
        })
        .collect();
    let now: Vec<(u64, [f32; 2])> = stands.iter().map(|(o, r)| (car_first(*o), r.at)).collect();
    if w.traffic.stand_gen == 0 || now != w.traffic.stands {
        w.traffic.stand_gen += 1;
        w.traffic.stands = now;
    }
    let now_gen = w.traffic.stand_gen;
    let mut passing = Vec::new();
    for c in 0..w.cars.len() {
        let car = &w.cars[c];
        if !matches!(car.state, CarState::Leaving | CarState::Arriving)
            || car.sight != Sight::Seen
            || car.standing
        {
            continue;
        }
        let Some((home, route)) = car.route.as_ref() else {
            continue;
        };
        let (home, t) = (*home, car.progress);
        let mut lay = None;
        let (mut unlay, mut drop) = (false, false);
        // Its way round one laid already: kept while it's on it or the taxi stands; laid again on
        // its way worked out again (a game loaded) where it was made on it.
        if let Some(pass) = car.pass_by {
            let fits = pass.home == home
                && match route.leg.as_ref() {
                    Some(l) => l.pass == pass,
                    None => crate::travel::dist(route.at(pass.from).0, pass.at) < 0.3,
                };
            let gone = !stands.iter().any(|(o, _)| *o as u32 == pass.taxi);
            if !fits || t > pass.to {
                drop = true;
            } else if gone && t < pass.from {
                unlay = true;
            } else {
                if route.leg.is_none() {
                    lay = Some(pass);
                }
                passing.push((car_first(c), car_first(pass.taxi as usize)));
            }
        }
        let fresh = route.stand_gen != now_gen;
        let spans = if fresh {
            stands
                .iter()
                .filter(|(o, _)| *o != c)
                .filter_map(|(_, r)| span_by(route, t, car.len, r))
                .collect()
        } else {
            route.spans.clone()
        };
        if fresh && (car.pass_by.is_none() || drop) && !unlay {
            lay = stands
                .iter()
                .filter(|(o, _)| *o != c)
                .find_map(|&(o, r)| leg_past(w, c, home, o, &r));
            if let Some(p) = lay {
                passing.push((car_first(c), car_first(p.taxi as usize)));
            }
        }
        let World {
            cars,
            travel,
            traffic,
            ..
        } = &mut *w;
        let car = &mut cars[c];
        let Some((_, route)) = car.route.as_mut() else {
            continue;
        };
        route.stand_gen = now_gen;
        if drop {
            car.pass_by = None;
        }
        let mut changed = spans != route.spans;
        route.spans = spans;
        if unlay {
            route.unlay_leg();
            car.pass_by = None;
            changed = true;
        }
        if let Some(p) = lay {
            route.unlay_leg();
            if route.lay_leg(p) {
                car.pass_by = Some(p);
            } else {
                car.pass_by = None;
                passing.retain(|x| x.0 != car_first(c));
            }
            changed = true;
        }
        // (Its claims worked out again: what it held and asked for let go, taken again as it goes
        // on, `traffic::Traffic::may_go`; but the group it's in the middle of held on, all of it,
        // `Traffic::reclaimed`.)
        if changed {
            car.reclaim(travel);
            if let Some((home, route)) = car.route.as_ref() {
                traffic.reclaimed(car_first(c), u32::from(*home), &route.claims, car.progress);
            } else {
                traffic.forget(car_first(c));
            }
        }
    }
    w.traffic.passing = passing;
}

/// Car `c`'s way out round work taxi `o` standing at the kerb (its body `taxi`) in its lane on its
/// way (`home`) ahead (`PassBy`): out into the other lane, clear of it by `PASS_SIDE`, from
/// `PASS_ENDS` behind it to as far past it, easing out and back in over `PASS_EASE`. None: the taxi
/// isn't in its lane ahead, or there's no such way: too near behind it to ease out, a junction's
/// box, its driveway or a stop of its on it, off the paving, or not clear of another standing.
pub(crate) fn leg_past(w: &World, c: usize, home: bool, o: usize, taxi: &Rect) -> Option<PassBy> {
    let car = &w.cars[c];
    let route = &car.route.as_ref()?.1;
    let t = car.progress;
    let half = [car.len / 2.0, car.wid / 2.0];
    let end = route.ticks().min(t + PASS_LOOK);
    if t + 2.0 >= end || crate::travel::dist(route.at(t).0, taxi.at) > PASS_FAR {
        return None;
    }
    // (Ticks along it from here a tick apart, and how far along that is, m.)
    let mut ts = vec![t];
    let mut ds = vec![0.0f32];
    let mut s = t;
    while s + 1.0 < end {
        let a = route.at(s).0;
        s += 1.0;
        ts.push(s);
        ds.push(ds[ds.len() - 1] + crate::travel::dist(a, route.at(s).0));
    }
    let body = |s: f32, off: f32| {
        let (p, yaw, _) = route.at(s);
        Rect {
            at: shifted(p, yaw, off),
            yaw,
            half,
        }
    };
    // (Where it would come up against it, going its way in its lane.)
    let near = half[0] + taxi.half[0] + 1.0;
    let hit = (0..ts.len()).find(|&k| {
        let b = body(ts[k], 0.0);
        crate::travel::dist(b.at, taxi.at) < near && b.gap(taxi) < PASS_SIDE
    })?;
    let (_, yaw, back) = route.at(ts[hit]);
    let way = [taxi.yaw.sin(), taxi.yaw.cos()];
    if back || yaw.sin() * way[0] + yaw.cos() * way[1] < 0.9 {
        return None;
    }
    // (Its middle alongside the taxi's, and how far right of it the taxi's is.)
    let kc = (hit..ts.len().min(hit + PASS_ALONGSIDE)).min_by(|&a, &b| {
        crate::travel::dist(route.at(ts[a]).0, taxi.at)
            .total_cmp(&crate::travel::dist(route.at(ts[b]).0, taxi.at))
    })?;
    let (pc, yc, _) = route.at(ts[kc]);
    let lat = (taxi.at[0] - pc[0]) * -yc.cos() + (taxi.at[1] - pc[1]) * yc.sin();
    if lat.abs() > 1.2 {
        return None;
    }
    let off = lat - taxi.half[1] - half[1] - PASS_SIDE;
    let dc = ds[kc];
    let d_out = dc - taxi.half[0] - half[0] - PASS_ENDS;
    let d_back = dc + taxi.half[0] + half[0] + PASS_ENDS;
    let d_from = (d_out - PASS_EASE).max(0.3);
    if d_out - d_from < PASS_EASE_LEAST {
        return None;
    }
    let at_d = |d: f32| {
        let k = ds.partition_point(|x| *x < d);
        ts.get(k).copied()
    };
    let (from, out, back, to) = (
        at_d(d_from)?,
        at_d(d_out)?,
        at_d(d_back)?,
        at_d(d_back + PASS_EASE)?,
    );
    if !(t < from && from < out && out <= back && back < to) {
        return None;
    }
    // (Not where it turns into or out of a driveway or a bay, nor a stop of its, a taxi.)
    if route.mouths.iter().any(|&(a, b)| a < to && b > from) {
        return None;
    }
    if car.taxi && crate::taxi::halts_within(w, c, from - 1.0, to + 1.0) {
        return None;
    }
    let pass = PassBy {
        home,
        taxi: o as u32,
        from,
        out,
        back,
        to,
        off,
        at: route.at(from).0,
    };
    // (Every pose of it out of the junctions' boxes, on the paving, and clear of those standing.)
    let g = &w.travel.graph;
    let standing: Vec<Rect> = parked(&w.cars)
        .filter(|d| d.first != car_first(c))
        .map(|d| d.body())
        .collect();
    let mut s = from;
    while s <= to {
        let (p, yaw0, _) = route.at(s);
        let (q, _, _) = route.at(s + 1.0);
        let (a, b) = (
            shifted(p, yaw0, off * pass_share(&pass, s)),
            shifted(q, yaw0, off * pass_share(&pass, s + 1.0)),
        );
        let r = Rect {
            at: a,
            yaw: (b[0] - a[0]).atan2(b[1] - a[1]),
            half,
        };
        if in_box(&g.boxes, r.at, car.len) {
            return None;
        }
        let (sn, cs) = r.yaw.sin_cos();
        let paved = [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)]
            .into_iter()
            .all(|(i, j)| {
                let k = [
                    r.at[0] + sn * i * half[0] + cs * j * half[1],
                    r.at[1] + cs * i * half[0] - sn * j * half[1],
                ];
                g.near(k, 8.0, 1, Mode::Car)
                    .first()
                    .is_some_and(|x| x.off <= kerb_off(&g.roads[x.road as usize]) + KERB_SLACK)
            });
        if !paved || standing.iter().any(|o| o.gap(&r) < PASS_SIDE - 0.1) {
            return None;
        }
        s += PASS_STEP;
    }
    Some(pass)
}

/// The stretch of `route` ahead of `t` (ticks along it) alongside taxi `taxi` standing at the kerb,
/// going the other way, where one passing the taxi (`PassBy`) would be out in its lane, begun and
/// ended a car of `len` short of that: its turn there with them (`pass_stands`). None: its way
/// doesn't go by it so.
pub(crate) fn span_by(route: &Route, t: f32, len: f32, taxi: &Rect) -> Option<(f32, f32)> {
    let u = [taxi.yaw.sin(), taxi.yaw.cos()];
    let n = [-taxi.yaw.cos(), taxi.yaw.sin()];
    let reach = taxi.half[0] + len + PASS_ENDS + PASS_EASE + 1.0;
    let end = route.ticks().min(t + PASS_LOOK);
    let mut s = t + 1.0;
    let (mut first, mut last) = (None, None);
    while s < end {
        let (q, yaw, back) = route.at(s);
        let rel = [q[0] - taxi.at[0], q[1] - taxi.at[1]];
        let along = rel[0] * u[0] + rel[1] * u[1];
        let lat = rel[0] * n[0] + rel[1] * n[1];
        let facing = yaw.sin() * u[0] + yaw.cos() * u[1];
        if !back && along.abs() <= reach && (-4.5..-0.5).contains(&lat) && facing < -0.9 {
            first.get_or_insert(s);
            last = Some(s);
        } else if last.is_some() {
            break;
        }
        s += PASS_STEP;
    }
    Some((first?, last?))
}

/// The cars held up round car `c` (`unjam`): it and those it waits for (`waits`), however far
/// round, each held up (`kind`: `Some(false)`) or a car parked in its way (`Some(true)`, not one of
/// them); with whether they're held up by each other alone, and whether one is by a car parked.
/// Not by each other alone: someone on foot among those waited for (they step aside first), one
/// on its way or out of sight (`kind`: None), or one at a red light (`red`) waiting for nothing
/// else (it goes on the green). One there that a car's in the way of waits for it on the green too
/// (`Traffic::begin`): with the others on another phase, one of them would always be at a red
/// light, and they'd stand there for good.
pub(crate) fn held_round(
    c: usize,
    waits: impl Fn(u64) -> Vec<u64>,
    red: impl Fn(u64) -> bool,
    kind: impl Fn(usize) -> Option<bool>,
) -> (Vec<usize>, bool, bool) {
    let mut ring = vec![c];
    let mut todo = vec![car_first(c)];
    let mut ok = true;
    let mut parked = false;
    while let Some(k) = todo.pop() {
        let by = waits(k);
        if by.is_empty() && red(k) {
            ok = false;
        }
        for o in by {
            if o < 1 << 63 {
                ok = false;
                continue;
            }
            let o = (o & !(1 << 63)) as usize;
            if ring.contains(&o) {
                continue;
            }
            match kind(o) {
                Some(true) => parked = true,
                Some(false) => {
                    ring.push(o);
                    todo.push(car_first(o));
                }
                None => ok = false,
            }
        }
    }
    (ring, ok, parked)
}

/// Cars in sight held up by each other (`traffic::waits_on`, round in a ring or on to one that
/// waits for nothing), or by a car parked in the way, none of them on `UNJAM` (`UNJAM_TICKS`),
/// none at a red light waiting for nothing else (one a car's in the way of there waits for it on
/// the green too) and nobody on foot among them (they step aside first, `go_on`): the one of them
/// with the most clear room behind it does as a driver would (`Detour`). It backs up along the way
/// it came, round the turn it took, till it's out of the last junction it came through, and goes
/// on from there another way, round the stretch it was held up on (`Route::detour`). Of those with
/// as much room: one with nobody riding with it first, then one on a trip before one on its way to
/// or from work, then one with riders, a work taxi last, and only once its stops are behind it
/// (they're along its way). None can (no junction behind it within the room it has, or no other
/// way): counted with why (`Traffic::unjam_impossible`), tried again `UNJAM_AGAIN` later.
pub(crate) fn unjam(w: &mut World) {
    let tick = w.tick;
    let held = |c: &Car| {
        matches!(c.state, CarState::Leaving | CarState::Arriving)
            && c.sight == Sight::Seen
            && tick >= c.stall.1 + UNJAM_TICKS
            && c.shown.is_some_and(|s| s.2)
    };
    let mut rings: Vec<Vec<usize>> = Vec::new();
    let mut taken = vec![false; w.cars.len()];
    // (Held up `PASS_WAIT` by a work taxi standing at the kerb in its way with no way out round it,
    // or waiting for its way round it while none of those it waits for goes on either: as by a
    // car parked there. Waiting its turn behind those coming the other way, it gets it.)
    let standing = |k: u64| {
        w.cars
            .get((k & !(1 << 63)) as usize)
            .is_some_and(|x| x.taxi && x.standing)
    };
    let still = |k: u64| {
        w.cars
            .get((k & !(1 << 63)) as usize)
            .is_some_and(|x| x.standing || tick >= x.stall.1 + PASS_WAIT)
    };
    for (c, car) in w.cars.iter().enumerate() {
        let by = crate::traffic::waits_on(w, car_first(c));
        let waiting = matches!(car.state, CarState::Leaving | CarState::Arriving)
            && car.sight == Sight::Seen
            && !car.standing
            && tick >= car.stall.1 + PASS_WAIT
            && car.shown.is_some_and(|s| s.2)
            && match car.pass_by {
                Some(p) => car.progress <= p.from && by.iter().all(|&k| k >= 1 << 63 && still(k)),
                None => by.iter().any(|&k| standing(k)),
            };
        let key = car_first(c);
        if waiting
            && w.traffic
                .unjam_tried
                .iter()
                .all(|&(k, at)| k != key || tick >= at + UNJAM_AGAIN)
        {
            taken[c] = true;
            rings.push(vec![c]);
        }
    }
    for c in 0..w.cars.len() {
        if taken[c] || !held(&w.cars[c]) {
            continue;
        }
        let (ring, ok, parked) = held_round(
            c,
            |k| crate::traffic::waits_on(w, k),
            |k| crate::traffic::at_red(w, k),
            |o| {
                if w.cars
                    .get(o)
                    .is_some_and(|x| x.state == CarState::Parked && !x.taxi)
                {
                    Some(true)
                } else {
                    w.cars.get(o).is_some_and(held).then_some(false)
                }
            },
        );
        for &o in &ring {
            taken[o] = true;
        }
        let key = ring.iter().map(|&o| car_first(o)).min().unwrap_or(0);
        if ok
            && (ring.len() >= 2 || parked)
            && w.traffic
                .unjam_tried
                .iter()
                .all(|&(k, at)| k != key || tick >= at + UNJAM_AGAIN)
        {
            rings.push(ring);
        }
    }
    for ring in rings {
        let key = ring.iter().map(|&o| car_first(o)).min().unwrap_or(0);
        // (A ring tried a little while ago and still there: counted once.)
        let again = w.traffic.unjam_tried.iter().any(|&(k, _)| k == key);
        w.traffic
            .unjam_tried
            .retain(|&(k, at)| k != key && tick < at + 4 * UNJAM_AGAIN);
        w.traffic.unjam_tried.push((key, tick));
        // Who of them can turn round, and how much room each has behind it: the most room
        // first, then by who's in it (`unjam`).
        let mut can: Vec<((f32, u8), usize, Detour)> = Vec::new();
        let mut why = "";
        for &c in &ring {
            match turn_round(w, c) {
                Ok((room, d)) => {
                    let car = &w.cars[c];
                    let riders = car.aboard.len() > 1 || !car.riders.is_empty();
                    let rank = match (car.taxi, riders, car.trip.is_some()) {
                        (true, ..) => 3,
                        (_, false, true) => 0,
                        (_, false, false) => 1,
                        (_, true, _) => 2,
                    };
                    can.push((((room * 2.0).floor() / 2.0, rank), c, d));
                }
                Err(e) => {
                    // (Why the household's cars couldn't, before that one of them was a taxi.)
                    let rank = |e: &str| UNJAM_WHY.iter().position(|x| *x == e).unwrap_or(0);
                    if why.is_empty() || rank(e) > rank(why) {
                        why = e;
                    }
                }
            }
        }
        can.sort_by(|a, b| {
            b.0.0
                .total_cmp(&a.0.0)
                .then(a.0.1.cmp(&b.0.1))
                .then(a.1.cmp(&b.1))
        });
        let mut fired = false;
        for (_, c, d) in can {
            if turn_round_by(w, c, d) {
                fired = true;
                // (No other way than back where it was held up: it lets the others by first.)
                let blocked = w.cars[c]
                    .route
                    .as_ref()
                    .and_then(|(_, r)| {
                        let back = r.points.iter().position(|q| !q.3)?;
                        let at = r.points.first()?.0;
                        Some(
                            r.points[back..]
                                .iter()
                                .any(|q| crate::travel::dist(q.0, at) < 3.0),
                        )
                    })
                    .unwrap_or(false);
                if blocked {
                    let by = ring
                        .iter()
                        .filter(|&&o| o != c)
                        .map(|&o| (o as u32, w.cars[o].progress, w.cars[o].state))
                        .collect();
                    w.cars[c].let_by = Some((tick + LET_BY_MOST, by));
                }
                break;
            }
            why = "no way";
        }
        if !fired && !again {
            *w.traffic.unjam_impossible.entry(why).or_default() += 1;
        }
    }
    // The ground each car turning round backs up over still (`Traffic::turning`): the others
    // keep out of it till it has (`drive_on`).
    let mut turning = Vec::new();
    for (c, car) in w.cars.iter().enumerate() {
        let (Some(_), Some((_, r))) = (car.detour, car.route.as_ref()) else {
            continue;
        };
        // (Backed up as far as it goes, driving on from the next tick: no ground left to back
        // over, only where it stands, which the others keep out of anyway (`guard`). Kept as
        // its, one backing up out of its way waited for it there for good while it waited for
        // that one, Oakview.)
        if !r.at(car.progress + 1.0).2 {
            continue;
        }
        let half = [car.len / 2.0, car.wid / 2.0];
        let mut t = car.progress;
        let mut path = Vec::new();
        while r.at(t).2 && t < r.ticks() {
            let (p, yaw, _) = r.at(t);
            path.push(Rect {
                at: shifted(p, yaw, car.shift),
                yaw,
                half,
            });
            t += 1.0;
        }
        if !path.is_empty() {
            turning.push((car_first(c), path));
        }
    }
    w.traffic.turning = turning;
}

/// Car `c` turned round `d` (`Detour`, `unjam`): its way that way from now on, its claims let go.
/// False: there's no such way (as it was).
pub(crate) fn turn_round_by(w: &mut World, c: usize, d: Detour) -> bool {
    let tick = w.tick;
    let home = d.home;
    let World {
        cars,
        exits,
        travel,
        traffic,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    car.avoided.retain(|a| a.1 > tick);
    let old = car.route.take();
    let had = car.detour;
    let twice = had.is_some();
    let passing = car.pass_by.take();
    car.detour = Some(d);
    let mut halts: Option<Box<dyn Fn(f32) -> f32>> = None;
    // (Turned round from its way as it is: a work taxi's is its run's, `taxi.rs`; one turned round
    // before, the way it turned round to. After a load that's worked out from the way it always
    // takes, `Car::route`: a car turned round twice is where the second way has it then. So too
    // one whose way isn't the way it always takes as it backs up along it, as one set off from
    // where it parked before stands: from that, it jumped there, tens of metres.)
    let plain = (!travel.graph.is_empty())
        .then(|| Route::on_roads(car, exits, travel, home))
        .flatten();
    let off_plain = match (old.as_ref(), plain.as_ref()) {
        (Some((_, had)), Some(p)) => [d.back, (d.back + d.from) / 2.0, d.from]
            .iter()
            .any(|&t| crate::travel::dist(had.at(t).0, p.at(t).0) > 0.05),
        _ => false,
    };
    if car.taxi || twice || off_plain {
        let new = old
            .as_ref()
            .and_then(|(_, had)| car.detoured(exits, travel, home, &d, had));
        // (Its stops still ahead, on the way it had: where they are along it now.)
        if let (true, Some(new), Some((_, had))) = (d.same, new.as_ref(), old.as_ref()) {
            halts = Some(Box::new(moved_along(had, new, d.back)));
        }
        car.route = new.map(|r| (home, r));
    } else {
        // (As `Car::route` has it after a load; but no such way, it isn't turned round: not off
        // on its way afresh from where it set off, which it jumped back to.)
        car.route = plain
            .as_ref()
            .and_then(|p| car.detoured(exits, travel, home, &d, p))
            .map(|r| (home, r));
    }
    // (Backing up as the new way has it, every pose of it clear of the others by the gap, as it
    // found the room, and of the ground any other turning round still backs up over
    // (`Traffic::turning`): two backing up into each other's way would wait for each other for
    // good. Else not.)
    let shift = car.shift;
    let half = [car.len / 2.0, car.wid / 2.0];
    let me = car_first(c);
    let others: Vec<Rect> = cars
        .iter()
        .enumerate()
        .filter(|&(o, k)| o != c && k.sight == Sight::Seen)
        .filter_map(|(_, k)| {
            k.shown.map(|s| Rect {
                at: s.0,
                yaw: s.1,
                half: [k.len / 2.0, k.wid / 2.0],
            })
        })
        .chain(
            traffic
                .turning
                .iter()
                .filter(|(k, _)| *k != me)
                .flat_map(|(_, path)| path.iter().copied()),
        )
        .collect();
    let car = &mut cars[c];
    let clear = car.route.as_ref().is_some_and(|(_, r)| {
        let body = |t: f32| {
            let (p, yaw, _) = r.at(t);
            Rect {
                at: shifted(p, yaw, shift),
                yaw,
                half,
            }
        };
        let now = body(0.0);
        let mut t = 0.0;
        let mut ok = true;
        while ok && t < r.ticks() && r.at(t).2 {
            let b = body(t);
            ok = others.iter().all(|o| {
                crate::travel::dist(o.at, b.at) > o.half[0] + b.half[0] + GAP + 0.5
                    || o.gap(&b) >= GAP.min(o.gap(&now))
            });
            t += 1.0;
        }
        ok
    });
    if car
        .route
        .as_ref()
        .is_none_or(|(h, r)| *h != home || r.ticks() <= 1.0)
        || !clear
    {
        // (As it was: on the way it had, turned round before or not. Not left on the way it
        // turned round to with no turn round, as on its way: backing up along it, it waited on
        // the one it backed into for good, Tide Sands.)
        car.detour = had;
        car.route = old;
        car.pass_by = passing;
        return false;
    }
    traffic.forget(car_first(c));
    traffic.unjams += 1;
    car.avoided.retain(|a| a.1 > tick && a.0 != d.avoid);
    car.avoided.push((d.avoid, tick + AVOID_FOR));
    car.progress = 0.01;
    car.unhold();
    car.stall = (car.progress, tick);
    if let Some(f) = halts {
        crate::taxi::moved_halts(w, c, &*f);
    }
    // (Put back where it now is, for those pairing up this tick: `pull_want`.)
    if let Some((_, r)) = w.cars[c].route.as_ref() {
        let at = r.at(w.cars[c].progress).0;
        w.cars_near.put(c, at);
    }
    true
}

/// Turned round only backing up out of the way (`Detour::same`) from `had` to `new`, backing up
/// to `back`: where a point so far along `had` (ticks) is along `new` (the stretch from `back` on
/// is the same, after the stretch it backs up).
pub(crate) fn moved_along(had: &Route, new: &Route, back: f32) -> impl Fn(f32) -> f32 + use<> {
    let i0 = had.points.partition_point(|q| q.2 <= back);
    let n_rev = new.points.len().saturating_sub(had.points.len() - i0);
    let (hp, np): (Vec<f32>, Vec<f32>) = (
        had.points.iter().map(|q| q.2).collect(),
        new.points.iter().map(|q| q.2).collect(),
    );
    move |a: f32| {
        let i = hp.partition_point(|&t| t <= a).saturating_sub(1).max(i0);
        let j = (n_rev + i - i0).min(np.len().saturating_sub(1));
        let (t0, t1) = (hp[i], hp.get(i + 1).copied().unwrap_or(hp[i]));
        let (u0, u1) = (np[j], np.get(j + 1).copied().unwrap_or(np[j]));
        let k = if t1 > t0 {
            ((a - t0) / (t1 - t0)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        u0 + (u1 - u0) * k
    }
}

/// Work taxi `c`, its run loaded (`taxi::settle`) on the way it always takes, as far along it as
/// it had come on the way it turned round to (`d`, `Run::detour`): on that way, its halts still
/// ahead where they are along it. (Turned round twice, the second way from the way it always
/// takes: as a household's car, `Car::route`.)
pub(crate) fn turned_round(w: &mut World, c: usize, d: Detour) {
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    let Some((_, had)) = car.route.take() else {
        return;
    };
    let progress = car.progress;
    let Some(new) = car.detoured(exits, travel, d.home, &d, &had) else {
        car.route = Some((true, had));
        return;
    };
    let halts = d.same.then(|| moved_along(&had, &new, d.back));
    car.detour = Some(d);
    car.route = Some((d.home, new));
    car.progress = progress;
    if let Some(f) = halts {
        crate::taxi::moved_halts(w, c, &f);
    }
}

/// How far car `c` can back up along the way it came (m) clear of every other car (`GAP`), and
/// where it would turn round (`Detour`): out of the last junction's box it came through, along the
/// stretch before it, within that room. Err: why it can't.
pub(crate) fn turn_round(w: &World, c: usize) -> Result<(f32, Detour), &'static str> {
    let car = &w.cars[c];
    // (A work taxi with its stops still ahead backs up out of the way only, and goes on its way.)
    let same = car.taxi && crate::taxi::halts_ahead(w, c);
    // (Turned round already, backing up still or letting the others by: not again.)
    if car.let_by.is_some() {
        return Err("turning round");
    }
    if car.detour.is_some()
        && car
            .route
            .as_ref()
            .is_some_and(|(_, r)| r.at(car.progress).2)
    {
        return Err("turning round");
    }
    let Some((home, route)) = car.route.as_ref() else {
        return Err("no way");
    };
    let t = car.progress;
    let half = [car.len / 2.0, car.wid / 2.0];
    let body = |s: f32| {
        let (p, yaw, _) = route.at(s);
        Rect {
            at: shifted(p, yaw, car.shift),
            yaw,
            half,
        }
    };
    let others: Vec<Rect> = w
        .cars
        .iter()
        .enumerate()
        .filter(|&(o, k)| o != c && k.sight == Sight::Seen)
        .filter_map(|(_, k)| {
            k.shown.map(|s| Rect {
                at: s.0,
                yaw: s.1,
                half: [k.len / 2.0, k.wid / 2.0],
            })
        })
        .collect();
    let clear = |r: &Rect| {
        others.iter().all(|b| {
            crate::travel::dist(b.at, r.at) > b.half[0] + r.half[0] + GAP + 0.5 || b.gap(r) >= GAP
        })
    };
    // (Not back into the driveway it came out of, nor out of town.)
    let first = route
        .mouths
        .first()
        .filter(|m| m.0 <= 0.0 && !*home)
        .map_or(1.0, |m| m.1.max(1.0));
    let boxes = &w.travel.graph.boxes;
    // (The stretch it was held up on: where it is, or on along its way.)
    let (mut ahead, step) = (t, drive_ticks(0.2));
    while ahead + step < route.ticks() && crate::travel::dist(route.at(ahead).0, body(t).at) <= 6.0
    {
        ahead += step;
    }
    let on = *w
        .travel
        .graph
        .near(route.at(ahead).0, ROAD_REACH, 1, Mode::Car)
        .first()
        .ok_or("no way")?;
    let turn = |back: f32, room: f32| {
        Ok((
            room,
            Detour {
                home: *home,
                from: t,
                back,
                avoid: on.edge,
                same,
                mouth: None,
            },
        ))
    };
    // Back along the way it came, while that's clear: out of the last junction's box it came
    // through and on a little (`TURN_PAST`), where it turns into another way; or, short of one, the
    // furthest place back where the road is wide enough for one to pass it pulled over
    // (`PASS_ROOM`) and it's a length and more back, where it waits for the others to go by.
    let (mut s, mut room, mut was) = (t, 0.0f32, body(t).at);
    let mut boxed = in_box(boxes, was, car.len);
    let mut seen_box = false;
    let mut wide: Option<(f32, f32)> = None;
    let mut why = "no junction behind";
    loop {
        let next = (s - 1.0).max(first);
        if next >= s {
            break;
        }
        let r = body(next);
        if !clear(&r) {
            why = if seen_box { "no room" } else { why };
            break;
        }
        room += crate::travel::dist(was, r.at);
        was = r.at;
        s = next;
        if room > TURN_ROOM {
            break;
        }
        let inside = in_box(boxes, r.at, car.len);
        if inside && !boxed {
            seen_box = true;
        }
        boxed = inside;
        if !inside && room > car.len + TURN_PAST && route.room_at(s) >= PASS_ROOM {
            wide = Some((s, room));
        }
        // (Out of the box it came through, and on a little: there it turns.)
        if seen_box && !inside {
            let mut back = s;
            let mut more = 0.0;
            while more < TURN_PAST {
                let n = (back - 1.0).max(first);
                if n >= back || !clear(&body(n)) {
                    break;
                }
                more += crate::travel::dist(body(back).at, body(n).at);
                back = n;
            }
            return turn(back, room + more);
        }
    }
    // Short of a junction, a driveway's mouth it came by within that clear room (not a taxi on
    // its run's way: it backs up only).
    if !same && let Some((back, m)) = mouth_behind(w, c, route, s, t, &clear) {
        let room = crate::travel::dist(body(back).at, body(t).at);
        return Ok((
            room,
            Detour {
                home: *home,
                from: t,
                back,
                avoid: on.edge,
                same,
                mouth: Some(m),
            },
        ));
    }
    match wide {
        Some((back, room)) => turn(back, room),
        None => Err(why),
    }
}

/// The nearest driveway's mouth behind car `c` (`t` ticks along `route`, clear back to `lo`) it can
/// turn round in (`MouthTurn`): one its way came by, empty (`clear`, every pose backing into it
/// and pulling out of it the other way), and where along its way it backs up to before backing
/// into it.
pub(crate) fn mouth_behind(
    w: &World,
    c: usize,
    route: &Route,
    lo: f32,
    t: f32,
    clear: &dyn Fn(&Rect) -> bool,
) -> Option<(f32, MouthTurn)> {
    let car = &w.cars[c];
    let g = &w.travel.graph;
    let half = [car.len / 2.0, car.wid / 2.0];
    let at = |s: f32| route.at(s);
    let mut best: Option<(f32, f32, MouthTurn)> = None;
    for d in &w.drives {
        let Some(way) = d.way() else { continue };
        let Some(p) = d.meets().and_then(|(r, s)| g.at_source(r, s)) else {
            continue;
        };
        let road = &g.roads[p.road as usize];
        let mp = road.at(p.s);
        // (Where its way passes the mouth, behind it and within the clear room.)
        let mut s = t;
        let mut near = None;
        while s >= lo {
            if crate::travel::dist(at(s).0, mp) < road.width / 2.0 + 1.0 {
                near = Some(s);
                break;
            }
            s -= 1.0;
        }
        let Some(sm) = near else { continue };
        if best.is_some_and(|b| b.0 >= sm) {
            continue;
        }
        let (_, yaw, _) = at(sm);
        let tan = road.tangent(p.s);
        let go = if tan[0] * yaw.sin() + tan[1] * yaw.cos() >= 0.0 {
            1.0
        } else {
            -1.0
        };
        let m = MouthTurn {
            road: p.road,
            s: p.s,
            way,
            go,
        };
        let Some(bay) = mouth_turn(&m, &w.travel, car.len) else {
            continue;
        };
        let (mut into, mut out) = (Legs::default(), Legs::default());
        bay.leave(&mut into, go);
        bay.leave(&mut out, -go);
        let Some(l) = into.last() else { continue };
        // (Where its way passes where backing into the mouth begins: it backs up to there.)
        let mut back = None;
        let mut s = t;
        while s >= lo {
            if crate::travel::dist(at(s).0, l) < 1.0 {
                back = Some(s);
                break;
            }
            s -= 1.0;
        }
        let Some(back) = back else { continue };
        // (Every pose of it, backing in and pulling out, clear of the others.)
        let poses = |pts: &[([f32; 2], bool, f32)]| {
            pts.windows(2).all(|k| {
                let (a, b) = (k[0].0, k[1].0);
                let r = Rect {
                    at: b,
                    yaw: (b[0] - a[0]).atan2(b[1] - a[1]),
                    half,
                };
                crate::travel::dist(a, b) < 1e-3 || clear(&r)
            })
        };
        if poses(&into.pts) && poses(&out.pts) {
            best = Some((sm, back, m));
        }
    }
    best.map(|(_, back, m)| (back, m))
}

/// Why none of a ring of cars could turn round (`unjam`), the most telling last: the one counted.
pub(crate) const UNJAM_WHY: [&str; 4] =
    ["turning round", "no room", "no junction behind", "no way"];

/// Longest a car turned round with no other way waits out of the way for those it was held up
/// with to go on (ticks of drive time, `unjam`).
pub(crate) const LET_BY_MOST: u64 = drive_secs(15.0);

/// How long a car keeps off a stretch it turned round from, turning round again (ticks of drive
/// time, `unjam`).
pub(crate) const AVOID_FOR: u64 = drive_secs(30.0);

/// How far the others keep off the ground a car turning round backs up over, past `GAP` (m,
/// `drive_on`).
pub(crate) const TURN_KEEP: f32 = 0.5;

/// The ground right behind a car turning round that holds it up (m, `drive_on`).
pub(crate) const TURN_BEHIND: f32 = 0.3;
/// A road where a car can pull over this far (m, `Route::room_at`) is wide enough for one to pass
/// it: a car held up with no junction behind it backs up to such a place (`turn_round`).
pub(crate) const PASS_ROOM: f32 = 0.6;
/// Furthest a car backs up to turn round (m, `turn_round`).
pub(crate) const TURN_ROOM: f32 = 60.0;
/// How far on past a junction's box a car backs up to turn round there (m).
pub(crate) const TURN_PAST: f32 = 3.0;
/// Held up this long (ticks of drive time), in sight, a car turns round (`unjam`): well before a
/// check counts it held up for good (10 game minutes).
pub(crate) const UNJAM_TICKS: u64 = drive_secs(4.0);
/// Cars none of which could turn round are tried again this much later (ticks of drive time,
/// `unjam`).
pub(crate) const UNJAM_AGAIN: u64 = drive_secs(2.0);

/// Whether car `c` on a trip, its driver no longer at the wheel, is in the picture, or where it
/// parks out or at home is (`in_picture`): it drives on there, not to be seen to jump there.
pub(crate) fn trip_seen(w: &World, c: usize) -> bool {
    let car = &w.cars[c];
    car.shown.is_some_and(|s| in_picture(w, s.0))
        || car.trip.is_some_and(|t| in_picture(w, t.at))
        || in_picture(w, car.spot)
}

/// Whether `p` is in the picture or near it (within `UNSEEN` of the watch, `World::set_watch`).
/// Without a watch nothing is: every car is seen, as in a headless run.
pub(crate) fn in_picture(w: &World, p: [f32; 2]) -> bool {
    w.watch.is_some_and(|[x0, z0, x1, z1]| {
        let dx = (x0 as f32 - p[0]).max(p[0] - x1 as f32).max(0.0);
        let dz = (z0 as f32 - p[1]).max(p[1] - z1 as f32).max(0.0);
        dx.hypot(dz) <= UNSEEN
    })
}

/// A car this far (m, its middle) from the watch is out of the picture (`unstick`): its body and
/// the ground in front of it well clear of what's drawn.
pub(crate) const UNSEEN: f32 = 8.0;

/// Standing still this long (ticks of drive time), with all it waits for, out of the picture, a
/// car is put on (`unstick`).
pub(crate) const STUCK_UNSEEN: u64 = drive_secs(3.0);

/// The bay a car `len` long turns round in at driveway mouth `m` (`Detour::mouth`): just inside
/// the driveway, off the road, facing out (as if parked there nose out). None: no road there, or
/// the driveway too short for the car off the road.
pub(crate) fn mouth_turn(m: &MouthTurn, travel: &Travel, len: f32) -> Option<Bay> {
    let g = &travel.graph;
    let road = g.roads.get(m.road as usize)?;
    let mp = road.at(m.s);
    let depth = kerb_off(road) + len / 2.0 + MOUTH_IN;
    let spot = [mp[0] - m.way[0] * depth, mp[1] - m.way[1] * depth];
    Bay::new(spot, m.way[0].atan2(m.way[1]), Some(m.way), mp, road, m.s)
}

/// How far into a driveway past the kerb a car turning round in its mouth backs (m, past its
/// length).
pub(crate) const MOUTH_IN: f32 = 0.5;
