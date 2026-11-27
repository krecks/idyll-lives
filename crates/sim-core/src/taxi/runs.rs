//! Running a booked taxi: picking up, setting down, giving up and settling.

use super::*;

/// Run `k`'s route in, by its own way in unless cars queue there (`queued`, `QUEUE_MOST`): then by
/// the nearest other way in with none queued whose way is no longer than `DETOUR_MOST` over its
/// own. None and `wait`: its own is queued and no other will do (it waits at the edge of town).
fn route_in(w: &World, k: usize) -> Option<(crate::cars::Route, Vec<f32>, Vec<Kerb>, bool)> {
    let (route, at, kerbs, _) = route_of(w, k)?;
    let own = route.at(0.0).0;
    if queued(w, own) < QUEUE_MOST {
        return Some((route, at, kerbs, false));
    }
    let mut others: Vec<[f32; 2]> = w
        .exits
        .iter()
        .copied()
        .filter(|&e| crate::travel::dist(e, own) > 4.0 && queued(w, e) < QUEUE_MOST)
        .collect();
    others.sort_by(|a, b| crate::travel::dist(*a, own).total_cmp(&crate::travel::dist(*b, own)));
    let most = route.ticks() + DETOUR_MOST as f32;
    for e in others.into_iter().take(QUEUE_TRIES) {
        if let Some((r, a, ks, _)) = route_via(w, k, Some(e))
            && r.ticks() <= most
        {
            return Some((r, a, ks, false));
        }
    }
    Some((route, at, kerbs, true))
}

/// The way out of town (`World::exits`) nearest `kerb` as the crow flies.
pub(super) fn nearest_way_out(w: &World, kerb: &Kerb) -> Option<[f32; 2]> {
    let at = w.travel.graph.point(kerb.mouth);
    w.exits
        .iter()
        .copied()
        .min_by(|a, b| crate::travel::dist(*a, at).total_cmp(&crate::travel::dist(*b, at)))
}

/// Run `k` a tick on: into town when it's time, driving, standing at its stop. False: it's over
/// (out of town at the end of its route, or it couldn't run).
pub(super) fn run_on(w: &mut World, k: usize) -> bool {
    let tick = w.tick;
    let run = &w.taxis.runs[k];
    if run.home && run.enter == 0 {
        // (Still gathering at work.)
        return true;
    }
    if run.car.is_none() {
        // (Waiting at the edge of town: tried again a minute on.)
        if tick < run.enter || !(tick - run.enter).is_multiple_of(TICKS_PER_MINUTE) {
            return true;
        }
        // (Nobody left to ride it, all of them gone their own way: it doesn't come.)
        if run.stops.iter().all(|s| s.state == StopState::Done) {
            return false;
        }
        // In it comes, along its route as it is now (a car parked or gone since it was planned),
        // round any place cars are held up now; with no way round one, a run to work none of
        // whose riders is out for it yet waits at the edge of town for it to clear, a while at
        // most, else it doesn't come.
        let Some((route, at, kerbs, queue)) = route_in(w, k) else {
            return false;
        };
        // (No longer than its spare time, a run to work: `time_run`'s `SLACK` and share.)
        let most = if run.home || run.place.is_some() {
            JAM_WAIT
        } else {
            JAM_WAIT
                .min(run.due_out.saturating_sub(run.enter) * SLACK_SHARE / (100 + SLACK_SHARE) / 2)
        };
        let jams = jammed(w);
        // (Not while any of its riders are already out for it: in it comes, round what it can.)
        let out = run.stops.iter().any(|st| {
            st.state != StopState::Done
                && w.sims
                    .get(st.sim as usize)
                    .is_some_and(|x| x.current().is_some_and(|a| run.serves(&a.task.kind)))
        });
        // (Its way in queued and no other near: a minute on, its spare time at most.)
        if queue && tick < run.enter + most {
            return true;
        }
        if !jams.is_empty() && !out && run.picks_up() && through(&route, &jams) {
            if tick < run.enter + most {
                return true;
            }
            // Still held up when it's waited all it can: it doesn't come, and its riders make
            // their own way (not planned again into the same hold-up).
            for st in &run.stops {
                match run.place {
                    // (Not booked again there for a while, `Taxis::no_taxi`.)
                    Some(place) => w.taxis.no_leisure.push((st.sim, place)),
                    None => {
                        if let Some(start) = start_for(w, st.sim as usize) {
                            w.taxis.missed.push((st.sim, start));
                        }
                    }
                }
            }
            return false;
        }
        let Some(c) = slot(w) else {
            return false;
        };
        let progress = w.taxis.runs[k].progress;
        let ticks = route.ticks();
        let car = &mut w.cars[c as usize];
        car.set_taxi_route(route, progress);
        let run = &mut w.taxis.runs[k];
        run.car = Some(c);
        run.laid = None;
        // Its times as its way is now (later in, or a longer way than planned): when it's at each
        // halt still to come, and so each rider's leave (`leave_at`), and when it's out of town.
        let halts = at.len() as u64;
        for st in run
            .stops
            .iter_mut()
            .filter(|st| st.state == StopState::Booked)
        {
            let h = if run.home { 0 } else { usize::from(st.halt) };
            let Some(&t) = at.get(h) else { continue };
            let due = tick + (t - progress).max(0.0).ceil() as u64 + h as u64 * DWELL;
            if due > st.due {
                st.leave += due - st.due;
                st.due = due;
            }
        }
        run.due_out = run
            .due_out
            .max(tick + (ticks - progress).max(0.0).ceil() as u64 + halts * DWELL);
        run.at = at;
        run.kerbs = kerbs;
    }
    let c = w.taxis.runs[k].car.expect("in town") as usize;
    let next = w.taxis.runs[k].next as usize;
    let ticks = w.cars[c].taxi_ticks();
    if next < w.taxis.runs[k].at.len() {
        let stop_at = w.taxis.runs[k]
            .at
            .get(next)
            .copied()
            .unwrap_or(f32::INFINITY);
        if w.cars[c].progress >= stop_at {
            stand(w, k, c, next);
            return true;
        }
    }
    // Held still on its way where nothing gives (`LONE_STILL`: taxis meeting on a lane, each
    // waiting for another's way; a car parked across its lane): it gives its way up (`give_up`).
    if w.taxis
        .still
        .get(c)
        .is_some_and(|s| s.0.is_some() && tick >= s.1 + LONE_STILL)
        && give_up(w, k, c)
    {
        // (In sight it doesn't leave the road on the spot: it turns round and drives off out of
        // town, `cars::drive_off`; out of sight it's gone.)
        if w.cars[c].sight == crate::cars::Sight::Seen {
            w.cars[c].off_run = true;
            w.traffic.give_ups[0] += 1;
        } else {
            w.traffic.give_ups[1] += 1;
        }
        return false;
    }
    crate::cars::drive_taxi(w, c);
    w.taxis.runs[k].progress = w.cars[c].progress;
    w.taxis.runs[k].detour = w.cars[c].detour;
    w.cars[c].progress < ticks
}

/// Run `k`'s taxi (car `c`), held still on its way where nothing gives, gives its way up (true: the
/// run's over, its taxi gone, `finish`): those aboard on their way home whose walk home from
/// where it is is short (`SHORT_WALK`) get out and walk; the others go home by another run, waiting
/// at work for it (`join_home_run`); on to work or school, they're there, and those still to be
/// picked up are booked again (`release`). A leisure run with someone aboard with a longer walk
/// keeps waiting (false).
fn give_up(w: &mut World, k: usize, c: usize) -> bool {
    let at = w.cars[c].shown.map_or(w.cars[c].spot, |s| s.0);
    let run = &w.taxis.runs[k];
    let (home, leisure, group) = (run.home, run.place.is_some(), run.group);
    let aboard: Vec<(usize, usize, u32)> = (0..run.stops.len())
        .filter(|&s| run.stops[s].state == StopState::Aboard)
        .map(|s| (s, run.stops[s].sim as usize, run.stops[s].plot))
        .collect();
    let walk_home = |w: &World, i: usize, plot: u32| {
        let Some(sim) = w.sims.get(i) else {
            return 0.0;
        };
        let door = w.plots.get(plot as usize).map_or(at, |p| p.arrival_point());
        w.travel.eta(
            at,
            door,
            crate::travel::Mode::Foot,
            sim.walk_per_minute(&w.content),
        )
    };
    if leisure
        && aboard
            .iter()
            .any(|&(_, i, plot)| walk_home(w, i, plot) > SHORT_WALK)
    {
        return false;
    }
    if !home && !leisure {
        // (Not theirs to have missed: those it was to pick up are booked again, `release`, as
        // long as a run is still sooner than the walk.)
        let waiting: Vec<u32> = w.taxis.runs[k]
            .stops
            .iter()
            .filter(|s| matches!(s.state, StopState::Booked | StopState::Boarding))
            .map(|s| s.sim)
            .collect();
        w.taxis.missed.retain(|m| !waiting.contains(&m.0));
    }
    // (Out of it at the kerb side, on ground they can walk on, not where its middle is: let out
    // inside its body, with the way home not found from there, they stood in its way as it
    // turned round, and it waited for them for good, Wheat Acres. Nor, both sides of it kept
    // off where it stands, the carriageway in a lit junction's box, set down there after all:
    // further out (its sides first, then its ends), the nearest ground they can walk on clear of
    // every car, Willowby. None so near: it doesn't give up yet.)
    let out = {
        let car = &w.cars[c];
        let (p, yaw) = car.shown.map_or((car.spot, car.yaw), |s| (s.0, s.1));
        let (sn, cs) = yaw.sin_cos();
        let nav = w.nav();
        let them = crate::cars::vehicles(&w.cars, None);
        let (side, end) = (car.wid / 2.0 + 0.5, car.len / 2.0 + 0.5);
        (0..LET_OUT_STEPS)
            .map(|n| n as f32 * LET_OUT_STEP)
            .flat_map(|n| {
                [
                    ([-cs, sn], side),
                    ([cs, -sn], side),
                    ([-sn, -cs], end),
                    ([sn, cs], end),
                ]
                .map(|(d, k)| [p[0] + d[0] * (k + n), p[1] + d[1] * (k + n)])
            })
            .find(|&q| {
                nav.tile_free(q[0].floor() as i32, q[1].floor() as i32)
                    && crate::cars::clear(&them, crate::cars::Body::WALKER, q)
            })
    };
    let Some(out) = out.or((!(home || leisure) || aboard.is_empty()).then_some(at)) else {
        return false;
    };
    if home || leisure {
        for (s, i, plot) in aboard {
            let short = walk_home(w, i, plot) <= SHORT_WALK;
            w.taxis.runs[k].stops[s].state = StopState::Done;
            let Some(sim) = w.sims.get_mut(i).filter(|x| x.here()) else {
                continue;
            };
            sim.taxi = None;
            sim.bound_for_car = None;
            if !short && !leisure && join_home_run(w, i, group) {
                continue;
            }
            let sim = &mut w.sims[i];
            sim.away_until = None;
            sim.current = None;
            sim.pose = crate::content::Pose::Stand;
            sim.pos = out;
            crate::cars::walk_home(w, i, None);
        }
    }
    true
}

/// A walk home this long at most (minutes) from where a taxi gave its way up (`give_up`) is walked;
/// a longer one waits at work for another run home.
const SHORT_WALK: f32 = 30.0;

/// Those aboard a taxi giving its way up (`give_up`) are set down beside it, out from its sides or
/// ends a step (m) at a time, this many steps at most, on ground they can walk on clear of every car.
const LET_OUT_STEP: f32 = 0.5;
const LET_OUT_STEPS: usize = 20;

/// Per car (`World::cars`), how far along its route it may be put on (`cars::moved_clear`): a work
/// taxi on a run, to its next stop at most (put on past it, it would stand where it's come to, as
/// it is: mid-turn, off its lane); any other car, as far as it likes.
pub(crate) fn stops_ahead(w: &World) -> Vec<f32> {
    let mut out = vec![f32::INFINITY; w.cars.len()];
    for run in &w.taxis.runs {
        if let Some(c) = run.car.map(|c| c as usize)
            && let Some(&at) = run.at.get(run.next as usize)
            && let Some(o) = out.get_mut(c)
        {
            *o = at;
        }
    }
    out
}

/// Run `k`'s taxi (car `c`) stands at its stop `next` (a halt: housemates' one): those there get
/// in (called to its door, a seat each: `door_for`), or out, each by a door of their own; one not
/// out yet within `taxi.waitMinutes`, or on their way but not at its door within `COMING_WAIT` at
/// their pace (and the walk round to it, `DOOR_WALK`), is left, and booked again (`release`).
fn stand(w: &mut World, k: usize, c: usize, next: usize) {
    let tick = w.tick;
    // (Pulling in to park beside the road, `cars::park_beside`: nobody gets in or out yet, its
    // doors still moving.)
    if crate::cars::stand_taxi(w, c) {
        return;
    }
    if w.taxis.runs[k].since == 0 {
        w.taxis.runs[k].since = tick;
    }
    let since = w.taxis.runs[k].since;
    let id = w.taxis.runs[k].id;
    let run = &w.taxis.runs[k];
    let place = run.place;
    let back = run.home && place.is_some();
    // (Those it puts down here, and those it picks up here.)
    let drops: Vec<usize> = (0..run.stops.len())
        .filter(|&s| run.drop_halt(&run.stops[s]) == Some(next as u32))
        .collect();
    let picks: Vec<usize> = (0..run.stops.len())
        .filter(|&s| run.picks_up() && run.pickup_halt(&run.stops[s]) == next as u32)
        .filter(|&s| !drops.contains(&s))
        .collect();
    // (What they're off to as they come to it: work or school, the place, home from it.)
    let task_of = |t: &TaskKind| match (place, back) {
        (None, _) => matches!(t, TaskKind::Work),
        (Some(p), false) => matches!(t, TaskKind::Visit { plot } if *plot == p),
        (Some(_), true) => matches!(t, TaskKind::GoHome),
    };
    // (How long it waits for one not yet at its door, `taxi.waitMinutes`.)
    let wait = (w.content.taxi.wait * crate::TICKS_PER_MINUTE as f32) as u64;
    let mut busy = false;
    for s in drops {
        let st = w.taxis.runs[k].stops[s];
        let i = st.sim as usize;
        let seat = 1 + s % 4;
        match st.state {
            StopState::Aboard => {
                // Out they get at their door and walk home; or at the place, and on to it.
                w.taxis.runs[k].stops[s].state = StopState::Done;
                if w.sims.get(i).is_some_and(|x| x.here()) {
                    w.sims[i].away_until = None;
                    w.sims[i].taxi = None;
                    match (place, back) {
                        (Some(p), false) => {
                            let directed = w.taxis.runs[k].directed;
                            alight_at(w, i, c, seat, p, directed);
                        }
                        _ => crate::cars::walk_home(w, i, Some((c, seat))),
                    }
                }
                busy = true;
            }
            _ => {
                // (Their door shut behind them.)
                busy |= w
                    .sims
                    .get(i)
                    .and_then(|x| x.current())
                    .is_some_and(|a| matches!(a.phase, Phase::Alighting { .. }))
                    && tick <= since + DWELL;
            }
        }
    }
    for s in picks {
        let st = w.taxis.runs[k].stops[s];
        let i = st.sim as usize;
        let seat = 1 + s % 4;
        match st.state {
            StopState::Booked => {
                let taxi_at = w.cars[c].spot;
                let sim = &w.sims[i];
                let on_way = sim.taxi.is_some_and(|b| b.run == id)
                    && sim.current().is_some_and(|a| task_of(&a.task.kind));
                // (At their kerb, or coming: at its door soon at their pace, the way round a
                // little longer than straight there; one already on their way out to it is waited
                // for longer, `COMING_WAIT`, than one yet to set off, `taxi.waitMinutes`.)
                let d = crate::travel::dist(sim.pos, taxi_at);
                let left = COMING_WAIT.saturating_sub(tick - since) as f32;
                let at_kerb = sim
                    .current()
                    .is_some_and(|a| matches!(a.phase, Phase::AtKerb { .. }));
                let soon = at_kerb
                    || d * WAY_ROUND
                        <= left * sim.walk_per_minute(&w.content) * crate::MINUTES_PER_TICK
                            + DOOR_WALK;
                if on_way && soon {
                    // Their turn: over to the taxi's door (`door_for`), and in.
                    let (door, _) = w.cars[c].taxi_door(&w.nav(), seat);
                    let sim = &mut w.sims[i];
                    sim.bound_for_car = Some(c as u32);
                    sim.car_seat = seat as u8;
                    sim.car_door = door as u8;
                    if let Some(a) = sim.current.take() {
                        sim.queue.push_front(a.task);
                    }
                    w.taxis.runs[k].stops[s].state = StopState::Boarding;
                    busy = true;
                } else if !on_way && sim.taxi.is_some_and(|b| b.run == id) && tick < since + wait {
                    // (Not out yet: a moment for them to set off.)
                    busy = true;
                } else {
                    // Not there, nor coming: on without them, and they make their own way (a
                    // leisure rider left is booked again once, `release`: told so, not by their
                    // stop not yet done, which once was marked first, and a guest eating a
                    // takeaway by the kerb as it came was never booked again, and walked home
                    // 540 m).
                    w.taxis.runs[k].stops[s].state = StopState::Done;
                    release(w, i, true);
                }
            }
            StopState::Boarding => {
                // (On their way to its door, or getting in; called away meanwhile, or held up on
                // their way to it too long: on without them.)
                // (In: their getting in done, it's under way this tick, `boarded`.)
                let going = w.sims.get(i).is_some_and(|x| {
                    let to_it = x.bound_for_car == Some(c as u32);
                    let doing = x.current().map(|a| (&a.task.kind, &a.phase));
                    // (The wait for them, the walk round to its door at their pace, and in.)
                    let door_walk = (DOOR_WALK * WAY_ROUND
                        / (x.walk_per_minute(&w.content) * crate::MINUTES_PER_TICK).max(0.01))
                        as u64;
                    let most = since + COMING_WAIT + door_walk;
                    to_it
                        && matches!(doing, Some((_, Phase::Boarding { .. })))
                        && tick <= most + crate::cars::GET_IN_TICKS as u64
                        || to_it
                            && (doing.is_some_and(|d| task_of(d.0))
                                || doing.is_none()
                                    && x.queue.front().is_some_and(|t| task_of(&t.kind)))
                            && tick <= most
                        || matches!(
                            x.transition,
                            Some(crate::life::Transition::Board { car }) if car == c as u32
                        )
                });
                if going {
                    busy = true;
                } else {
                    // (Left, and booked again, as above.)
                    w.taxis.runs[k].stops[s].state = StopState::Done;
                    release(w, i, true);
                }
            }
            StopState::Aboard | StopState::Done => {}
        }
    }
    // (Not off into one going by alongside it, both pulled over as they pass on a narrow lane: it
    // waits till they're past, a while at most.)
    if !busy && tick <= since + PASS_BY_MOST {
        let car = &w.cars[c];
        let me = car.shown.map_or((car.spot, car.yaw), |s| (s.0, s.1));
        let key = crate::cars::car_first(c);
        busy = crate::cars::vehicles(&w.cars, Some(c)).iter().any(|o| {
            // (Not one that waits for it, which won't pass.)
            let waits = w
                .cars
                .get(crate::cars::car_index(o.first))
                .is_some_and(|x| x.waits_for == Some(key))
                || w.traffic.waits_for(o.first).contains(&key);
            !waits
                && crate::cars::too_near(
                    (me.0, me.1, car.len, car.wid + 2.0 * PASS_BY),
                    (o.at, o.yaw, o.len, o.wid),
                )
        });
    }
    // (Nor off while one is out round it in the other lane (`cars::PassBy`): stopped there, for
    // someone on a bike on the far side, it was left behind it as it pulled away, out in the other
    // lane, and a car coming that way waited for it, the taxi for that one, all three for good,
    // Gannet Island. Till it's past, a while at most.)
    if !busy && tick <= since + PASS_ROUND_MOST {
        let key = crate::cars::car_first(c);
        busy = w.traffic.passing.iter().any(|&(o, taxi)| {
            taxi == key
                && w.cars.get(crate::cars::car_index(o)).is_some_and(|x| {
                    x.pass_by
                        .is_some_and(|p| (p.from..p.to).contains(&x.progress))
                })
        });
    }
    if !busy {
        let run = &mut w.taxis.runs[k];
        run.next += 1;
        run.since = 0;
        w.cars[c].standing = false;
    }
}

/// How near one going by a taxi ready to go on may be, besides `cars::PASSING` (m, either side),
/// for it to wait till they're past, and how long it waits so at most since it stood (ticks of
/// drive time).
const PASS_BY: f32 = 0.3;
const PASS_BY_MOST: u64 = crate::drive_secs(3.0);
/// How long a taxi ready to go on waits at most since it stood for one out round it to get past
/// (ticks of drive time).
const PASS_ROUND_MOST: u64 = crate::drive_secs(30.0);

/// Resident `i`, booked on a taxi, isn't riding it: unbooked, and on their way to work or school
/// their own way (from the kerb, or wherever they've got to); planned once more for the same start
/// (`Taxis::missed`), late as it may be, before they're left to it. `left`: the taxi went on (or
/// its run ended) without them, not yet in it: a leisure rider is booked again once
/// (`Taxis::left`). Said by the caller, whatever their stop has been marked.
pub(crate) fn release(w: &mut World, i: usize, left: bool) {
    let Some(sim) = w.sims.get_mut(i) else {
        return;
    };
    let Some(b) = sim.taxi.take() else {
        return;
    };
    if sim
        .bound_for_car
        .is_some_and(|c| w.cars.get(c as usize).is_some_and(|c| c.taxi))
    {
        sim.bound_for_car = None;
    }
    let heading = sim
        .current()
        .is_some_and(|a| matches!(a.task.kind, TaskKind::Work));
    let mut leisure = None;
    let home = match w.taxis.runs.iter_mut().find(|r| r.id == b.run) {
        Some(run) => {
            if let Some(st) = run.stops.get_mut(b.stop as usize)
                && st.state != StopState::Aboard
            {
                st.state = StopState::Done;
                if left && let Some(place) = run.place {
                    leisure = Some((place, run.home, run.directed));
                }
            }
            run.home || run.place.is_some()
        }
        None => false,
    };
    // (Left at the kerb on a leisure run: booked again once, a run of their own, and out to the
    // kerb for it.)
    if let Some((place, back, directed)) = leisure
        && !w.taxis.left.contains(&(i as u32, place))
    {
        w.taxis.left.push((i as u32, place));
        let booked = if back {
            book_back(w, i, place, w.tick)
        } else {
            book_leisure(w, i, place, directed)
        };
        let World {
            sims,
            content,
            objects,
            ..
        } = &mut *w;
        let sim = &mut sims[i];
        let on_way = sim.current().is_some_and(|a| match a.task.kind {
            TaskKind::GoHome => back,
            TaskKind::Visit { plot } => !back && plot == place,
            _ => false,
        });
        if booked && on_way {
            let task = sim.current().expect("just looked").task;
            crate::world::end_activity(sim, content, objects);
            sim.current = None;
            sim.queue.push_front(task);
        }
    }
    // (Planned again once for the same start, a late run of their own if it must be; then their
    // own way it is.)
    let mut again = false;
    if !home && let Some(start) = start_for(w, i) {
        let me = (i as u32, start);
        if w.taxis.missed.contains(&me) {
            w.taxis.declined.push(me);
        } else {
            w.taxis.missed.push(me);
            again = true;
        }
    }
    // (On their way to it: on to work on their own; or, to be booked again, back to what they'd
    // do till it's time to go out for the next (`leave_at`), not waiting at the kerb for it.)
    if heading {
        let World {
            sims,
            content,
            objects,
            ..
        } = &mut *w;
        let sim = &mut sims[i];
        crate::world::end_activity(sim, content, objects);
        sim.current = None;
        if !again {
            sim.queue.push_front(Task {
                kind: TaskKind::Work,
                directed: true,
                routine: false,
            });
        }
    }
}

/// Resident `i` has got into taxi `c` (`world::Phase::Boarding` done): their shift under way with
/// the rest of the ride and the way off the map still to go, out of sight as in a household's car.
pub(crate) fn boarded(w: &mut World, i: usize, c: u32) {
    let me = i as u32;
    let Some(k) = w
        .taxis
        .runs
        .iter()
        .position(|r| r.car == Some(c) && r.picks_up() && r.stop_of(me).is_some())
    else {
        // (In a taxi with no stop of theirs: their own went on without them.)
        w.sims[i].bound_for_car = None;
        release(w, i, true);
        return;
    };
    let s = w.taxis.runs[k].stop_of(me).expect("just found");
    if let Some(place) = w.taxis.runs[k].place {
        // A leisure ride: sat in it (`seat_riders`), there or home.
        let run = &mut w.taxis.runs[k];
        run.stops[s].state = StopState::Aboard;
        let task = if run.home {
            TaskKind::GoHome
        } else {
            TaskKind::Visit { plot: place }
        };
        let directed = run.directed;
        let sim = &mut w.sims[i];
        sim.bound_for_car = Some(c);
        sim.current = Some(crate::world::Activity {
            task: Task {
                kind: task,
                directed,
                routine: false,
            },
            phase: Phase::Driving,
            tags: Default::default(),
        });
        return;
    }
    let rest_ride = (w.cars[c as usize].taxi_ticks() - w.cars[c as usize].progress).max(0.0);
    let halt = w.taxis.runs[k].stops[s].halt as usize;
    let left = w.taxis.runs[k].at.len().saturating_sub(halt + 1) as u64;
    let rest = rest_ride.ceil() as u64 + left * DWELL + crate::cars::off_map_ticks(w);
    crate::life::start_shift(w, i, rest);
    let g = w.taxis.runs[k].group;
    w.taxis.runs[k].stops[s].state = StopState::Aboard;
    let sim = &mut w.sims[i];
    sim.bound_for_car = None;
    sim.taxi = Some(Booking {
        run: w.taxis.runs[k].id,
        stop: s as u8,
    });
    if let Some(&exit) = w.exits.get(g.1 as usize) {
        sim.pos = exit;
    }
    w.taxis.rode.retain(|r| r.0 != me);
    w.taxis.rode.push((me, g));
}

/// The run `k` is over: its taxi out of town, its slot free; those on it to work at work (their
/// arrival noted, `Taxis::arrivals`); anyone still booked on it makes their own way.
pub(super) fn finish(w: &mut World, k: usize) {
    let tick = w.tick;
    let off_map = crate::cars::off_map_ticks(w);
    let run = w.taxis.runs[k].clone();
    // (Not one driving off, given up in sight: it parks once out of town, `cars::drive_off`.)
    if let Some(c) = run.car.filter(|&c| !w.cars[c as usize].off_run) {
        w.cars[c as usize].park_taxi();
    }
    for st in &run.stops {
        let i = st.sim as usize;
        if run.place.is_some() && st.state == StopState::Aboard {
            // (A leisure ride it couldn't finish: out where it is, and home on foot.)
            if let Some(sim) = w.sims.get_mut(i) {
                sim.taxi = None;
                sim.bound_for_car = None;
                sim.current = None;
                sim.pose = crate::content::Pose::Stand;
                if let Some(car) = run.car.and_then(|c| w.cars.get(c as usize)) {
                    sim.pos = car.spot;
                }
            }
            crate::cars::walk_home(w, i, None);
            continue;
        }
        if !run.home && st.state == StopState::Aboard {
            if let Some(start) = crate::life::start_on(&w.content, &w.sims[i], tick) {
                let late = (tick + off_map) as i64 - start as i64;
                let planned = if run.planned_out > 0 {
                    run.planned_out
                } else {
                    run.due_out
                };
                let list = if planned + off_map > start {
                    &mut w.taxis.late_arrivals
                } else {
                    &mut w.taxis.arrivals
                };
                if list.len() >= 4096 {
                    list.remove(0);
                }
                list.push((st.sim, late));
            }
            if w.sims[i].taxi.is_some_and(|b| b.run == run.id) {
                w.sims[i].taxi = None;
            }
            continue;
        }
        if run.home && st.state == StopState::Aboard {
            // (Never got home by it: from the way out on foot.)
            if w.sims.get(i).is_some_and(|s| s.here()) {
                w.sims[i].away_until = None;
                w.sims[i].taxi = None;
                crate::cars::walk_home(w, i, None);
            }
            continue;
        }
        // (Still booked on it, not yet in it: the run's over without them.)
        if w.sims
            .get(i)
            .is_some_and(|s| s.taxi.is_some_and(|b| b.run == run.id))
        {
            release(w, i, true);
        }
    }
}

/// Resident `i` is done at work (or school): having come by taxi today, or on foot with a long
/// walk home (`homeward`), they wait there for one home (true), with the others of their group
/// done about when they are (`Run::depart`).
pub(crate) fn off_work(w: &mut World, i: usize) -> bool {
    warm_kerbs(w);
    let me = i as u32;
    let g = match w.taxis.rode.iter().position(|r| r.0 == me) {
        Some(p) => w.taxis.rode.remove(p).1,
        // (Come on foot: home by one all the same, a long walk home.)
        None => match homeward(w, i) {
            Some(g) => g,
            None => return false,
        },
    };
    join_home_run(w, i, g)
}

/// Resident `i` waits at work (away) for a run home with group `g`: the one gathering there now
/// with a seat free, else a new one leaving `taxi.window` minutes on (`off_work`; one whose run
/// home gave up its way, `give_up`, waits for another). False: no home to go to.
fn join_home_run(w: &mut World, i: usize, g: (u32, u32)) -> bool {
    let me = i as u32;
    let tick = w.tick;
    let window = (w.content.taxi.window * TICKS_PER_MINUTE as f32) as u64;
    let seats = w.content.taxi.seats;
    let plot = match w
        .households
        .get(w.sims[i].household as usize)
        .and_then(|h| h.plot)
    {
        Some(p) => p,
        None => return false,
    };
    let stop = Stop {
        sim: me,
        plot,
        due: 0,
        leave: 0,
        state: StopState::Aboard,
        halt: 0,
    };
    let k = match w.taxis.runs.iter().position(|r| {
        r.home && r.group == g && r.enter == 0 && r.depart >= tick && r.stops.len() < seats
    }) {
        Some(k) => k,
        None => {
            let id = w.taxis.next_id;
            w.taxis.next_id = id.wrapping_add(1);
            w.taxis.runs.push(Run {
                id,
                home: true,
                place: None,
                directed: false,
                group: g,
                enter: 0,
                due_out: 0,
                planned_out: 0,
                depart: tick + window,
                stops: Vec::new(),
                next: 0,
                progress: 0.0,
                detour: None,
                since: 0,
                car: None,
                at: Vec::new(),
                kerbs: Vec::new(),
                laid: None,
            });
            w.taxis.runs.len() - 1
        }
    };
    let s = w.taxis.runs[k].stops.len();
    w.taxis.runs[k].stops.push(stop);
    let id = w.taxis.runs[k].id;
    // (Away till it's back: worked out as it leaves work, `close_home_runs`.)
    let back = tick + window + 4 * crate::cars::off_map_ticks(w).max(TICKS_PER_MINUTE);
    let sim = &mut w.sims[i];
    sim.taxi = Some(Booking {
        run: id,
        stop: s as u8,
    });
    sim.away_until = Some(back);
    true
}

/// Resident `i`'s group (`group`), if they'd go home from work by taxi having come some other
/// way: a long walk home, no bike of theirs to ride, a stop at home (`would_ride`'s, but for the
/// car seat: the household's car has gone home without them).
fn homeward(w: &World, i: usize) -> Option<(u32, u32)> {
    let s = &w.sims[i];
    let c = s.commute?;
    let home = w.households.get(s.household as usize)?.plot;
    if c.home != home
        || c.bike.is_some()
        || c.walk - crate::life::OUT_OF_THE_HOUSE <= w.content.taxi.min_walk
    {
        return None;
    }
    stop_for(w, i)?;
    group(w, i)
}

/// The runs home leaving work now: their stops in a short order from the way in, and when the
/// taxi comes into town (after the way off the map); each rider away till they're dropped off.
pub(super) fn close_home_runs(w: &mut World) {
    let tick = w.tick;
    let off_map = crate::cars::off_map_ticks(w);
    for k in 0..w.taxis.runs.len() {
        let run = &w.taxis.runs[k];
        if !run.home || run.enter != 0 || run.depart > tick {
            continue;
        }
        let Some(&exit) = w.exits.get(run.group.1 as usize) else {
            continue;
        };
        // (Nearest the way in first, then on nearest to the last.)
        let mut left = run.stops.clone();
        let mut order = Vec::new();
        let mut at = exit;
        let place = |w: &World, st: &Stop| {
            w.plots
                .get(st.plot as usize)
                .map_or(exit, |p| p.arrival_point())
        };
        while !left.is_empty() {
            let j = (0..left.len())
                .min_by(|&a, &b| {
                    let d = |s: &Stop| crate::travel::dist(place(w, s), at);
                    d(&left[a])
                        .total_cmp(&d(&left[b]))
                        .then(left[a].sim.cmp(&left[b].sim))
                })
                .expect("some left");
            let st = left.remove(j);
            at = place(w, &st);
            order.push(st);
        }
        // (Housemates next to each other: one halt.)
        let mut h = 0u8;
        for j in 0..order.len() {
            if j > 0 && order[j].plot != order[j - 1].plot {
                h += 1;
            }
            order[j].halt = h;
        }
        w.taxis.runs[k].stops = order;
        w.taxis.runs[k].enter = tick + off_map;
        let Some((route, at, kerbs, later)) = route_of(w, k) else {
            // (No way: they make their own way home from the way out.)
            w.taxis.runs[k].enter = tick;
            w.taxis.runs[k].at = Vec::new();
            continue;
        };
        w.taxis.runs[k].kerbs = kerbs;
        // (Later, a stand of its another's till then: `stands_and_way`.)
        let enter = tick + off_map + later;
        w.taxis.runs[k].enter = enter;
        let stops = w.taxis.runs[k].stops.clone();
        // (When it stands at each, for the other runs' stands, `stand_window`.)
        w.taxis.runs[k].at = at.clone();
        for (s, st) in stops.iter().enumerate() {
            let h = st.halt as usize;
            let back = enter + at.get(h).copied().unwrap_or(0.0).ceil() as u64 + h as u64 * DWELL;
            let i = st.sim as usize;
            if w.sims.get(i).is_some_and(|x| x.here()) {
                w.sims[i].away_until = Some(back.max(tick + 1) + DWELL);
                w.sims[i].taxi = Some(Booking {
                    run: w.taxis.runs[k].id,
                    stop: s as u8,
                });
            }
        }
        let _ = route;
    }
}

/// After a load (and once the cars are settled): the bookings from the runs, and the taxis under
/// way back in town where they'd got to.
pub(crate) fn settle(w: &mut World) {
    for s in &mut w.sims {
        s.taxi = None;
    }
    let mut k = 0;
    while k < w.taxis.runs.len() {
        let run = w.taxis.runs[k].clone();
        for (s, st) in run.stops.iter().enumerate() {
            if st.state != StopState::Done
                && let Some(sim) = w.sims.get_mut(st.sim as usize)
            {
                sim.taxi = Some(Booking {
                    run: run.id,
                    stop: s as u8,
                });
            }
            // (Getting in as it was saved: at the kerb again.)
            if st.state == StopState::Boarding {
                w.taxis.runs[k].stops[s].state = StopState::Booked;
            }
        }
        let started = !(run.home && run.enter == 0) && w.tick >= run.enter && run.car.is_none();
        if started {
            match (route_of(w, k), slot(w)) {
                (Some((route, at, kerbs, _)), Some(c)) => {
                    w.cars[c as usize].set_taxi_route(route, run.progress);
                    w.taxis.runs[k].car = Some(c);
                    w.taxis.runs[k].at = at;
                    w.taxis.runs[k].kerbs = kerbs;
                    // (Turned round on its way: on the way it turned round to.)
                    if let Some(d) = run.detour {
                        crate::cars::turned_round(w, c as usize, d);
                    }
                }
                _ => {
                    finish(w, k);
                    w.taxis.runs.remove(k);
                    continue;
                }
            }
        } else if (!run.home || run.enter != 0)
            && let Some((_, at, kerbs, _)) = route_of(w, k)
        {
            w.taxis.runs[k].at = at;
            w.taxis.runs[k].kerbs = kerbs;
        }
        k += 1;
    }
}

/// What resident `s` is doing with the taxi, for the UI (`view/`): waiting for it at the kerb,
/// in it to work (and when they're there, a tick), or on their way home in it (waiting at work for
/// it, or in it).
pub fn status(w: &World, s: &crate::world::Sim) -> Option<(&'static str, Option<u64>)> {
    let b = s.taxi?;
    let run = w.taxis.runs.iter().find(|r| r.id == b.run)?;
    let st = run.stops.get(b.stop as usize)?;
    if run.home && !(run.place.is_some() && st.state != StopState::Aboard) {
        return (st.state == StopState::Aboard).then_some(("home", s.away_until));
    }
    match st.state {
        // (Out to a place: their visit or outing says where.)
        StopState::Aboard if run.place.is_some() => None,
        StopState::Aboard => Some(("work", Some(run.due_out + crate::cars::off_map_ticks(w)))),
        StopState::Booked
            if s.current()
                .is_some_and(|a| matches!(a.phase, Phase::AtKerb { .. })) =>
        {
            Some(("wait", None))
        }
        // (Over to its door and getting in.)
        StopState::Boarding => Some(("wait", None)),
        _ => None,
    }
}

impl World {
    /// For tests: the taxi resident `i` is booked on, if any (its run, their stop).
    #[doc(hidden)]
    pub fn taxi_booking(&self, i: usize) -> Option<Booking> {
        self.sims.get(i)?.taxi
    }

    /// For tests: when resident `i` leaves for the taxi they're booked on to work, as late as it
    /// runs (`booked_leave`: their free time before work ends then).
    #[doc(hidden)]
    pub fn taxi_leave(&self, i: usize) -> Option<u64> {
        leave_at(self, i)
    }
}

impl Taxis {
    /// Whether `LEISURE_FLEET` taxis are out on visits and outings now: no more are sent there.
    pub(crate) fn fleet_out(&self) -> bool {
        self.runs.iter().filter(|r| r.place.is_some()).count() >= LEISURE_FLEET
    }

    /// Whether a taxi would take `sim` out to a visit or an outing now (`book_leisure`): not one
    /// booked on a taxi already (to work, say: they'd walk it, hours across a big world, the
    /// way timed by the taxi), nor with the fleet out (`fleet_out`).
    pub(crate) fn takes(&self, sim: &crate::world::Sim) -> bool {
        sim.taxi.is_none() && !self.fleet_out()
    }

    /// Whether `sim` was just turned down for a taxi to `plot` (`turned_down`): they walk.
    pub(crate) fn no_taxi(&self, sim: u32, plot: u32) -> bool {
        self.no_leisure.iter().any(|&(s, p)| s == sim && p == plot)
    }

    pub(crate) fn turned_down(&mut self, sim: u32, plot: u32) {
        if !self.no_taxi(sim, plot) {
            self.no_leisure.push((sim, plot));
        }
    }

    /// Nothing planned or under way, nobody waiting at work for one home (saves leave it out).
    pub fn is_empty(&self) -> bool {
        self.runs.is_empty() && self.rode.is_empty()
    }
}

// ---------------------------------------------------------------------------------------------
// Where a taxi stands for its riders, and its way round town.
