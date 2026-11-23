//! Pulling over and passing: the room, who passes whom, and the pass itself.

use super::*;

/// The last-resort check on a car's move from body `now` to `next`: the first of `them` (the cars
/// on the road and parked) it would come nearer than `GAP` to, and nearer than it is now (one it's
/// already that near, from before, it may draw away from or keep beside, never nearer). None: it
/// may move.
pub(crate) fn guard<'a>(
    now: &Rect,
    next: &Rect,
    layer: crate::travel::Layer,
    them: impl Iterator<Item = &'a Driving>,
) -> Option<u64> {
    let near = |at: [f32; 2], len: f32| {
        crate::travel::dist(at, next.at) < (len + next.half[0] * 2.0) + AHEAD + GAP
    };
    // (Those on its level only: `travel::Layer`.)
    them.into_iter()
        .find(|o| o.layer.meets(layer) && near(o.at, o.len) && closes_in(now, next, &o.body()))
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
    // (Stood a moment held up by nobody, its claim just granted, say: it goes on now, so it pulls
    // over only as far as it can on its way, as on the move. Pulled over where it stood, further
    // than the room it goes on with, it had to pull back in first, and couldn't beside the one
    // it had pulled over for, standing the other way: it waited for that one to pass, which
    // waited for the car behind it, which waited for it, for good, Clover Acres.)
    let going = waited.is_none() && !w.traffic.waiting(me) && w.cars[c].zebra_wait == 0;
    let (mut want, mut whom) = pull_want(w, c, moving || held || going, &after);
    if !(moving || held || going) && whom.is_some_and(waits_me) {
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
        car.layer(),
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

/// None of them (`Near::near`: their numbers have moved, so it looks at every one).
const NO_CARS: &[([f32; 2], usize)] = &[];

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
    pub(crate) fn build(&mut self, cars: &[Car]) {
        self.by_x.clear();
        self.most_len = 0.0;
        self.most_wid = 0.0;
        for (c, car) in cars.iter().enumerate() {
            self.by_x.push((car.middle(), c));
            self.most_len = self.most_len.max(car.len);
            self.most_wid = self.most_wid.max(car.wid);
        }
        self.by_x.sort_unstable_by(by_x_order);
        self.built = cars.len();
    }

    /// Puts car `c` at `at` (as it is now). One not in it is looked at whatever (`near`).
    pub(super) fn put(&mut self, c: usize, at: [f32; 2]) {
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
            (NO_CARS, 0..cars)
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
    // than `GAP` and `PASSING`: there they take turns, `traffic/`, and turning there it faces
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
            || !other.layer().meets(car.layer())
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
