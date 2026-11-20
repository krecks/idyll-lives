//! Turning round and unjamming: the detours, the mouths and the reasons why.
//!
//! Split from `driving.rs` (see `mod.rs`).

use super::*;

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
    // (Each car's ask looked up by key, not searched for: `waits_on_at`.)
    let asks = crate::traffic::asks_by_key(w);
    for (c, car) in w.cars.iter().enumerate() {
        let by = crate::traffic::waits_on_at(w, car_first(c), &asks);
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
            |k| crate::traffic::waits_on_at(w, k, &asks),
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
    // (Turned round from its way as it is: a work taxi's is its run's, `taxi/`; one turned round
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
