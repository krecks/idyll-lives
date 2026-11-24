//! Sight: what a car can see, what it waits for, and being stuck out of the picture.

use super::*;

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
        // (Its level now and a tick on: only those on a level that meets it are in its way.)
        let (level_now, level_next) = (route.layer_at(t), route.layer_at(t + 1.0));
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
            // (Looked along by time alone, `sight_steps`'s own points are these: none sampled
            // again.)
            let mut hit = false;
            let (n, by_time) = sight_samples(route, a, b, |p| hit = hit || near(p, reach));
            if n == by_time {
                return hit;
            }
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
            let seen: Vec<(u64, Rect, crate::travel::Layer)> = cars
                .iter()
                .enumerate()
                .filter(|&(o, other)| o != c && other.sight == Sight::Seen)
                .filter_map(|(o, other)| Some((car_first(o), other.shown_rect()?, other.layer())))
                .collect();
            // (On top of one in sight where it is, or touching it, the seen area come to it (the
            // camera moved there, nobody has seen it yet): it's put back along its way, or on, to
            // the nearest place clear of those in sight (`moved_clear`), and comes into sight
            // there. Coming in, it waits outside till its way in is clear by `GAP`.)
            let on_top = now.is_some_and(|r| {
                seen.iter()
                    .any(|(_, b, l)| l.meets(level_now) && b.gap(&r) < TOUCH)
            });
            let by = seen
                .iter()
                .find(|(_, b, l)| {
                    l.meets(level_next) && b.gap(&next) < GAP
                        || l.meets(level_now) && now.is_some_and(|r| b.gap(&r) < GAP)
                })
                .map(|&(k, ..)| k);
            match by {
                Some(_) if in_now && on_top => {
                    let clear = |r: &Rect, level: crate::travel::Layer| {
                        clear_of(seen.iter().map(|(_, b, l)| (b, *l)), r, level)
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
                // `waits_for_queue`.)
                Some(k) if !in_now && area.is_some() => Sight::Wait(k),
                // (Its way in held up just inside: out of sight nobody waits, so those coming
                // would all come in at once. It waits outside till the queue there has room for
                // it, as the rules would have let it come, `METER`.)
                None if !in_now && area.is_some() => {
                    waits_for_queue(&mut cars[..], c, t, shift, half, tick, near)
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
                car.next_ground = forward_ground();
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
    let gone = |k: &Option<u64>| k.is_some_and(|k| out.get(car_index(k)) == Some(&true));
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

/// `Sight` for one coming into town held up just inside (`sight`): it waits outside for the one in sight its way in is
/// held up by, if there is one where it would come in within `METER_MOST` of first waiting
/// (`Sight::Wait`), else it comes in (`Sight::Seen`). `near`: whether a place is in the seen area.
fn waits_for_queue(
    cars: &mut [Car],
    c: usize,
    t: f32,
    shift: f32,
    half: [f32; 2],
    tick: u64,
    near: impl Fn([f32; 2], f32) -> bool,
) -> Sight {
    let bodies: Vec<(u64, Rect, crate::travel::Layer)> = cars
        .iter()
        .enumerate()
        .filter(|&(o, k)| {
            o != c
                && k.sight == Sight::Seen
                && matches!(k.state, CarState::Leaving | CarState::Arriving)
                && (k.standing || tick >= k.stall.1 + METER_STILL)
        })
        .filter_map(|(o, k)| Some((car_first(o), k.shown_rect()?, k.layer())))
        .collect();
    // (The way it came in by, made above (`Car::route`): one out of sight waiting has one by now,
    // but a car without one simply comes in.)
    let Some((_, route)) = cars[c].route.as_ref() else {
        return Sight::Seen;
    };
    // (Its place on its way in, up to the first junction it takes turns at in sight: the queue for
    // it.)
    let upto = route
        .claims
        .iter()
        .map(|k| k.from)
        .find(|&f| f > t && near(route.at(f).0, 0.0))
        .unwrap_or(t + METER.ahead)
        .min(t + METER.most);
    // (`SIGHT_STEP` ticks a step, as ever; finer where that's further than `SIGHT_APART`.)
    let span = (upto - t - 1.0).max(0.0);
    let (by_time, n) = (
        (span / SIGHT_STEP).ceil().max(1.0) as usize,
        sight_steps(route, t + 1.0, t + 1.0 + span),
    );
    let step = if n > by_time {
        span / n as f32
    } else {
        SIGHT_STEP
    };
    let full = (0..)
        .map(|i| t + 1.0 + i as f32 * step)
        .take_while(|&s| s < upto)
        .find_map(|s| {
            let (p, yaw, _) = route.at(s);
            let r = Rect {
                at: shifted(p, yaw, shift),
                yaw,
                half,
            };
            let level = route.layer_at(s);
            bodies
                .iter()
                .find(|(_, b, l)| l.meets(level) && b.gap(&r) < GAP + METER.gap)
                .map(|(k, ..)| *k)
        });
    // (Not for good: waited `METER_MOST`, it comes in as it would have.)
    let since = *cars[c].metered.get_or_insert(tick);
    match full {
        Some(k) if tick < since + METER_MOST => Sight::Wait(k),
        _ => Sight::Seen,
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
    clear: &dyn Fn(&Rect, crate::travel::Layer) -> bool,
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
        .find(|&s| {
            !claiming(s)
                && clear(&body(s), route.layer_at(s))
                && clear(&body(s + 1.0), route.layer_at(s + 1.0))
        })
}

/// How far apart a car's way from `a` to `b` ticks along `route` is looked along coming into
/// sight (`sight`, `waits_for_queue`): steps of `SIGHT_STEP` ticks, and no more than `SIGHT_APART`
/// m apart (on a motorway, 7.5 m a step: it'd skip a junction's 6 m box). How many steps.
pub(crate) fn sight_steps(route: &Route, a: f32, b: f32) -> usize {
    sight_samples(route, a, b, |_| {}).0
}

/// `sight_steps`, and how many steps by time alone, each of the points of those handed to `each`
/// on the way.
fn sight_samples(route: &Route, a: f32, b: f32, mut each: impl FnMut([f32; 2])) -> (usize, usize) {
    let by_time = ((b - a) / SIGHT_STEP).ceil().max(1.0) as usize;
    let k = by_time as f32;
    let mut far: f32 = 0.0;
    let mut was = None;
    for i in 0..=by_time {
        let p = route.at(a + (b - a) * i as f32 / k).0;
        each(p);
        if let Some(q) = was {
            far = far.max(crate::travel::dist(q, p));
        }
        was = Some(p);
    }
    let n = if far <= SIGHT_APART {
        by_time
    } else {
        (by_time as f32 * far / SIGHT_APART).ceil() as usize
    };
    (n, by_time)
}

/// Whether body `r` on `level` is clear of `bodies` (each on its level) by `GAP`, as a car is put
/// to come into sight (`moved_clear`: `sight`, `unstick`).
fn clear_of<'a>(
    bodies: impl IntoIterator<Item = (&'a Rect, crate::travel::Layer)>,
    r: &Rect,
    level: crate::travel::Layer,
) -> bool {
    bodies.into_iter().all(|(b, l)| {
        !l.meets(level)
            || crate::travel::dist(b.at, r.at) > b.half[0] + r.half[0] + GAP + 0.5
            || b.gap(r) >= GAP
    })
}

/// The furthest apart (m) a car's way coming into sight is looked along (`sight_steps`): finer
/// than a junction's smallest box (`traffic::boxes`, 6 m), and no finer than at a country road's
/// speed (4 m a step).
pub(crate) const SIGHT_APART: f32 = 5.0;

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
                let car = &w.cars[car_index(o)];
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
        let others: Vec<(Rect, crate::travel::Layer)> = w
            .cars
            .iter()
            .enumerate()
            .filter(|&(o, k)| o != c && k.sight == Sight::Seen)
            .filter_map(|(_, k)| Some((k.shown_rect()?, k.layer())))
            .collect();
        let car = &w.cars[c];
        let half = [car.len / 2.0, car.wid / 2.0];
        let clear = |r: &Rect, level: crate::travel::Layer| {
            unseen(r.at) && clear_of(others.iter().map(|(b, l)| (b, *l)), r, level)
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
            car.next_ground = forward_ground();
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
