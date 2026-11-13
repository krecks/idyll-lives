//! Trips and outings: when to go, how long for, and the way out of town.
//!
//! Split from `world.rs` (see `mod.rs`).

use super::*;

/// `a` (radians) as a facing from -PI to PI.
pub fn wrap_angle(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

/// What `sim` may use where they are: on the plot they're a guest at, else at home and the
/// public things (within `autonomy.reach` in an open-world town).
pub(crate) fn usable_here<'a>(sim: &Sim, ctx: &Ctx<'a>) -> std::borrow::Cow<'a, [u32]> {
    match sim.visiting {
        Some(visit) => std::borrow::Cow::Borrowed(ctx.index.on(visit.plot)),
        None => ctx.index.usable_near(
            ctx.households[sim.household as usize].plot,
            sim.pos,
            ctx.content.autonomy.reach,
        ),
    }
}

/// How far from a trailhead its trail may start (m).
pub(crate) const TRAIL_REACH: f32 = 30.0;

/// Where someone walking the trail nearest `at` is `elapsed` of `total` minutes in, at `walk` m a
/// minute, and which way they face: out along it (the longer way from where it starts near `at`)
/// for the first half, back for the second, turning at its end (round and round a loop). None:
/// no trail near.
pub(crate) fn trail_step(
    travel: &crate::travel::Travel,
    at: [f32; 2],
    elapsed: f32,
    total: f32,
    walk: f32,
) -> Option<([f32; 2], f32)> {
    let g = &travel.graph;
    let road = g
        .roads
        .iter()
        .enumerate()
        .filter(|(_, r)| r.kind == crate::travel::RoadKind::Trail)
        .map(|(i, r)| (i, r.project(at)))
        .filter(|(_, (_, d))| *d <= TRAIL_REACH)
        .min_by(|a, b| a.1.1.total_cmp(&b.1.1).then(a.0.cmp(&b.0)))
        .map(|(i, (s0, _))| (&g.roads[i], s0))?;
    let (r, s0) = road;
    let len = r.len();
    if len < 1.0 {
        return None;
    }
    let ring = crate::travel::dist(r.pts[0], *r.pts.last()?) < 1.0;
    let out = if s0 < len / 2.0 { 1.0 } else { -1.0 };
    let half = total.max(0.0) / 2.0;
    let back = elapsed > half;
    let along = walk * elapsed.min(total - elapsed).clamp(0.0, half);
    let raw = s0 + out * along;
    // Off the end: round a loop, back along a trail that stops.
    let (s, turned) = if ring {
        (raw.rem_euclid(len), false)
    } else {
        let m = raw.rem_euclid(2.0 * len);
        if m > len {
            (2.0 * len - m, true)
        } else {
            (m, false)
        }
    };
    let t = r.tangent(s);
    let k = out * if back != turned { -1.0 } else { 1.0 };
    Some((r.at(s), (t[0] * k).atan2(t[1] * k)))
}

/// The middle of the free tile on plot `p` nearest `at` (rings outwards), if any.
pub(crate) fn open_ground_near(nav: &NavGrid, p: &Plot, at: [f32; 2]) -> Option<[f32; 2]> {
    let (cx, cz) = (at[0].floor() as i32, at[1].floor() as i32);
    let inside = |x: i32, z: i32| x >= p.x && z >= p.z && x < p.x + p.w && z < p.z + p.d;
    (0..p.w.max(p.d)).find_map(|r| {
        (-r..=r)
            .flat_map(|dz| (-r..=r).map(move |dx| (dx, dz)))
            .filter(|&(dx, dz)| dx.abs() == r || dz.abs() == r)
            .map(|(dx, dz)| (cx + dx, cz + dz))
            .find(|&(x, z)| inside(x, z) && nav.tile_free(x, z))
            .map(|(x, z)| [x as f32 + 0.5, z as f32 + 0.5])
    })
}

/// An outing (`Choice::Outing`) needs at least this long there besides the way there and
/// back (minutes); the way puts them off this much a minute (less than a walk across the room
/// does, `ai::falloff`: it's a day out).
pub(crate) const OUTING_STAY_MINUTES: f32 = 60.0;
/// Friends are visited (`Choice::Visit`, an invitation over the phone) no more than this many
/// minutes away, and with time for at least `VISIT_STAY_MINUTES` there besides the way there
/// and back.
pub(crate) const VISIT_TRIP_MINUTES: f32 = 45.0;
pub(crate) const VISIT_STAY_MINUTES: f32 = 45.0;

/// The most each need gets from something on `plot` that `sim` may use there (as a guest at a
/// home, `guest`: what a guest may, `guest_may`), as one day out.
pub(crate) fn place_serves(
    ctx: &Ctx,
    objects: &[ObjectInstance],
    sim: &Sim,
    plot: u32,
    guest: bool,
) -> [f32; MAX_NEEDS] {
    let content = ctx.content;
    let mut best = [0.0f32; MAX_NEEDS];
    for o in ctx
        .index
        .on(plot)
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| !o.broken())
    {
        let def = &content.objects[o.def];
        for i in def.interactions.iter().filter(|i| {
            unfit(content, sim.age, i).is_none() && (!guest || guest_may(content, def, i))
        }) {
            for (b, &g) in best.iter_mut().zip(&i.total_gain) {
                *b = b.max(g);
            }
        }
    }
    best
}

/// Whether `sim`'s needs last `minutes` out (the way to a visit or an outing and
/// `visits.leastMinutes` there): none of them would be below `life::LOW_NEED` by then, and so send
/// them home, that the place has nothing for (`serves`, `place_serves`).
pub(crate) fn lasts(content: &Content, sim: &Sim, serves: &[f32; MAX_NEEDS], minutes: f32) -> bool {
    content.needs.iter().enumerate().all(|(n, def)| {
        let decay = def.decay_per_minute * sim.mods.need_decay[n] * minutes;
        def.room || serves[n] > 0.0 || sim.needs[n] - decay >= crate::life::LOW_NEED
    })
}

/// How someone goes on a visit or an outing (`leisure_way`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Way {
    Car,
    Bike,
    Taxi,
    Foot,
}

/// What `leisure_way` weighs for a resident: whether they'd drive the household's car at home
/// (`cars::could_drive`), and the thing at home they'd ride (`ride::leisure_bike`: its speed,
/// times walking, and its `minRide`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Goer {
    pub drives: bool,
    pub bike: Option<(f32, f32)>,
}

/// How resident `goer` would go from `pos` (over the ground) to `door` of `plot` on a visit or an
/// outing, and about how many minutes the way takes: past `autonomy.reach`, in the household's
/// car if they drive it and there's parking near (`cars::trip_car`: the drive, to and from the car
/// and the walk from the parking); else in an open-world town on a bike of theirs for a ride of
/// its `minRide` or more, at its speed (`ride.rs`); else by the taxi (`taxi`: its minutes, from
/// `taxi::leisure_eta`); else on foot at `walk` m a minute. `on`: where they'd get on the roads, on
/// foot (and a bike) and by car, found once for many places from one place.
#[allow(clippy::too_many_arguments)]
pub(crate) fn leisure_way(
    travel: &crate::travel::Travel,
    content: &Content,
    pos: [f32; 2],
    walk: f32,
    plot: u32,
    door: [f32; 2],
    goer: Goer,
    taxi: Option<f32>,
    on: &mut [Option<Option<crate::travel::RoadPoint>>; 2],
) -> (Way, f32) {
    let distance = (door[0] - pos[0]).hypot(door[1] - pos[1]);
    let car =
        distance > content.autonomy.reach && goer.drives && crate::cars::parks_near(travel, door);
    let bike = goer
        .bike
        .filter(|&(_, min)| distance >= min)
        .map(|(speed, _)| speed);
    let (way, mode, pace) = match (car, bike, taxi) {
        (true, ..) => (Way::Car, crate::travel::Mode::Car, walk),
        (false, Some(speed), _) => (Way::Bike, crate::travel::Mode::Foot, walk * speed),
        (false, None, Some(minutes)) => return (Way::Taxi, minutes),
        (false, None, None) => (Way::Foot, crate::travel::Mode::Foot, walk),
    };
    let from = &mut on[mode as usize];
    let eta = travel.eta_via(
        pos,
        door,
        mode,
        pace,
        || *from.get_or_insert_with(|| travel.entry(pos, mode)),
        || travel.door_entry(plot, door, mode),
    );
    let minutes = match way {
        Way::Car => trip_minutes(eta, true) + park_walk(travel, door, walk),
        _ => eta,
    };
    (way, minutes)
}

/// Minutes on foot from the public parking nearest `door` to it (`cars::parking_near`): a trip
/// there by car walks it, as much as 120 m.
pub(crate) fn park_walk(travel: &crate::travel::Travel, door: [f32; 2], walk: f32) -> f32 {
    crate::cars::parking_near(travel, door).map_or(0.0, |at| {
        travel.eta(at, door, crate::travel::Mode::Foot, walk)
    })
}

/// Until when someone of household `h` is about to host a guest: the latest any of its grown-ups
/// (who let a guest in, `life::opens_door`) not at work (at home, or out and coming back) leaves
/// for work or school today (`leaves_for_work`, worked out only now, for them; never, for one
/// with none to go to). With them all gone a guest goes home (`life::end_visit_if_due`): let in,
/// a guest was off home again 23 minutes on, the one host gone to her night shift (Stillpool).
/// (A baby, or a child, home all day is no host: counted, a guest stayed on with the baby once
/// the one grown-up left for work.)
pub(crate) fn hosts_about_till(ctx: &Ctx, h: u32) -> u64 {
    ctx.briefs
        .iter()
        .enumerate()
        .filter(|(_, b)| b.household == h && b.here && !b.away && b.adult)
        .filter_map(|(j, _)| ctx.others.get(j))
        .map(|s| leaves_for_work(s, ctx).unwrap_or(u64::MAX))
        .max()
        .unwrap_or(0)
}

/// Whether a visit `eta` minutes away is one to make at `hour`, with `free` minutes before
/// they must leave for something else: not a walk of hours, there before the end of visiting
/// hours, and time to stay and get back.
pub(crate) fn visit_fits(content: &Content, eta: f32, hour: f32, free: f32) -> bool {
    eta <= VISIT_TRIP_MINUTES
        && hour + eta / 60.0 < content.visits.latest_hour
        && free >= 2.0 * eta + VISIT_STAY_MINUTES
}

/// Whether an outing `eta` minutes away (by car for one who'd drive there, else on foot) is one
/// to set off on at `hour`, with `free` minutes before they must leave for work or school: as
/// `visit_fits`, the way no longer than `visits.outingMinutes` (a walk of hours had them out
/// past midnight, asleep by the road), and time for `OUTING_STAY_MINUTES` there and the way
/// back, before they leave and home by `visits.outingHomeBy`.
pub(crate) fn outing_fits(content: &Content, eta: f32, hour: f32, free: f32) -> bool {
    let rules = &content.visits;
    let away = 2.0 * eta + OUTING_STAY_MINUTES;
    eta <= rules.outing_minutes && free >= away && hour + away / 60.0 <= rules.outing_home_by
}

/// Whether `sim`, setting off from home on an outing to `plot` (a public place out of
/// `autonomy.reach`), may go: by car (`cars::trip_car`, `cars::trip_ride`), or on the bike they're
/// on or would take (`leisure_ride`), or on foot, where that way fits (`outing_fits`). Anything
/// else (not from home, not such a place): yes.
pub(crate) fn outing_fits_now(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], plot: u32) -> bool {
    let content = ctx.content;
    let Some(p) = ctx.plots.get(plot as usize) else {
        return true;
    };
    let home = ctx
        .households
        .get(sim.household as usize)
        .and_then(|h| h.plot);
    let at_home = home.is_some() && ctx.briefs.get(sim.id as usize).and_then(|b| b.plot) == home;
    let pos = ctx.nav.stairs.ground_pos(sim.pos);
    let door = p.arrival_point();
    // (Booked on a taxi there already: it's the taxi's to time, `taxi::book_leisure`.)
    let visit = TaskKind::Visit { plot };
    if crate::taxi::kerb_for(ctx, sim, &visit).is_some()
        || crate::taxi::door_for(ctx, sim, &visit).is_some()
        || !p.public
        || !at_home
        || crate::travel::dist(pos, door) <= content.autonomy.reach
        || sim.riding.is_none()
            && (crate::cars::trip_car(ctx, sim, plot).is_some()
                || crate::cars::trip_ride(ctx, sim, plot).is_some())
    {
        return true;
    }
    // (On the bike already, or on the way to it, or the one they'd take.)
    let bike = sim
        .riding
        .or(sim.bound_for_bike)
        .and_then(|b| objects.get(b as usize))
        .and_then(|b| content.objects[b.def].vehicle)
        .map(|v| (v.pace, 0.0))
        .or_else(|| {
            leisure_ride(sim, ctx, objects, plot)
                .and_then(|b| content.objects[objects[b as usize].def].vehicle)
                .map(|v| (v.pace, 0.0))
        });
    let goer = Goer {
        drives: false,
        bike,
    };
    let (_, eta) = leisure_way(
        ctx.travel,
        content,
        pos,
        sim.walk_per_minute(content),
        plot,
        door,
        goer,
        taxi_from_home(sim, ctx, plot)
            .then(|| crate::taxi::leisure_eta(ctx.travel, ctx.exits, content, sim, pos, door))
            .flatten(),
        &mut [None, None],
    );
    outing_fits(content, eta, ctx.hour, free_before_work(sim, ctx))
}

/// Whether `sim`, at home, goes to `plot` (a visit or an outing) by the taxi (`leisure_way`: no
/// car or bike of theirs to go in, nor the household's car at home, the taxi quicker than walking,
/// `taxi::leisure_eta`), not just turned down for it (`taxi::Taxis::no_taxi`).
pub(crate) fn taxi_from_home(sim: &Sim, ctx: &Ctx, plot: u32) -> bool {
    let home = ctx
        .households
        .get(sim.household as usize)
        .and_then(|h| h.plot);
    let at_home = home.is_some() && ctx.briefs.get(sim.id as usize).and_then(|b| b.plot) == home;
    let Some(p) = ctx.plots.get(plot as usize) else {
        return false;
    };
    let pos = ctx.nav.stairs.ground_pos(sim.pos);
    // (Not with the household's car at home: the household goes in it, or calls them to it.)
    let car_home = ctx
        .cars
        .iter()
        .any(|c| !c.taxi && c.household == sim.household && c.at_home());
    at_home
        && !car_home
        && Some(plot) != home
        && !ctx.taxis.no_taxi(sim.id, plot)
        && ctx.taxis.takes(sim)
        && crate::taxi::leisure_eta(
            ctx.travel,
            ctx.exits,
            ctx.content,
            sim,
            pos,
            p.arrival_point(),
        )
        .is_some()
}

/// The bike (or e-scooter) at home `sim` rides out on to `plot` (a visit or an outing in an
/// open-world town, `leisure_way`): the fastest free one that fits them (`ride::leisure_bike`),
/// for a ride of its `minRide` or more from where they are. None: they walk (or drive).
pub(crate) fn leisure_ride(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    plot: u32,
) -> Option<u32> {
    let content = ctx.content;
    let door = ctx.plots.get(plot as usize)?.arrival_point();
    let (bike, _, min) =
        crate::ride::leisure_bike(content, ctx.index, ctx.briefs, ctx.households, objects, sim)?;
    let pos = ctx.nav.stairs.ground_pos(sim.pos);
    ((door[0] - pos[0]).hypot(door[1] - pos[1]) >= min).then_some(bike)
}

/// The tick a guest on an outing to `plot` (a public place in an open-world town) heads home by
/// to be home by `visits.outingHomeBy` today, and before they leave for work or school
/// (`free_before_work`, as `outing_fits` gave them time for): the way back from `pos` on foot (or
/// by the taxi they came by), or for one whose car is parked out for them the walk back to it and
/// the drive. None: not such an outing.
pub(crate) fn outing_leave_by(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    plot: u32,
    pos: [f32; 2],
) -> Option<u64> {
    let content = ctx.content;
    if !ctx.plots.get(plot as usize)?.public {
        return None;
    }
    let home = ctx.households.get(sim.household as usize)?.plot?;
    let door = ctx.plots.get(home as usize)?.arrival_point();
    let walk = sim.walk_per_minute(content);
    let travel = ctx.travel;
    // (By car: the walk back to it where it's parked out, and the drive; by bike, the ride.)
    let car = crate::cars::car_out_for(ctx, sim)
        .map(|(.., at)| at)
        .or_else(|| crate::cars::ride_home_for(ctx, sim).map(|(.., at)| at));
    let bike = sim
        .bike_parked
        .zip(sim.riding.or(sim.bound_for_bike))
        .and_then(|(at, b)| {
            Some((
                at,
                content.objects[objects.get(b as usize)?.def].vehicle?.pace,
            ))
        });
    let foot = crate::travel::Mode::Foot;
    let minutes = match (car, bike) {
        (Some(at), _) => {
            travel.eta(pos, at, foot, walk)
                + trip_minutes(travel.eta(at, door, crate::travel::Mode::Car, walk), true)
        }
        (None, Some((at, speed))) => {
            travel.eta(pos, at, foot, walk) + travel.eta(at, door, foot, walk * speed)
        }
        // (Come by taxi: the taxi back, `taxi::back_minutes`.)
        (None, None) => crate::taxi::back_minutes(ctx, sim, plot, pos, door)
            .unwrap_or_else(|| travel.eta(pos, door, foot, walk)),
    };
    let back = ((minutes + OUTING_LEAVING_MINUTES) * TICKS_PER_MINUTE as f32) as u64;
    let home_by = crate::clock::tick_at(ctx.day, content.visits.outing_home_by * 60.0)?;
    // (Stayed on past it, they set off for work from the place: on a bike left there, a ride of
    // two hours across a big world, Highholm.)
    let free = free_before_work(sim, ctx);
    let home_by = if free.is_finite() {
        home_by.min(ctx.tick + crate::mins(free))
    } else {
        home_by
    };
    Some(home_by.saturating_sub(back))
}

/// An outing's end: game minutes to get going home besides the way itself (up from where they
/// sat, a word with whoever they were talking to), so they're home by the hour, not a few minutes
/// past it.
pub(crate) const OUTING_LEAVING_MINUTES: f32 = 10.0;

/// An outing by car: minutes besides the drive, to the car and from it at either end.
pub(crate) const CAR_TRIP_MINUTES: f32 = 10.0;
pub(crate) const OUTING_K: f32 = 0.01;

/// The game minutes a trip takes that `Travel::eta` says takes `eta` (by car, `car`): a drive's
/// minutes at 1× are `travel::DRIVE_MINUTES` of the game's each, and a few more to and from the car.
pub(crate) fn trip_minutes(eta: f32, car: bool) -> f32 {
    if car {
        eta * crate::travel::DRIVE_MINUTES + CAR_TRIP_MINUTES
    } else {
        eta
    }
}

/// Whether `sim`'s way out to work or school isn't by bike (a bike only where the way was timed
/// by one: a ride of `life::BIKE_MOST` at most; not before it's been timed, nor where it was
/// timed on foot or by car).
pub(crate) fn no_bike_out(sim: &Sim) -> bool {
    sim.commute.is_none_or(|c| c.long_ride || c.bike.is_none())
}

/// The way out of town `sim` takes on foot: the quickest (`travel::Travel::eta`).
pub(crate) fn exit_on_foot(ctx: &Ctx, sim: &Sim) -> Option<[f32; 2]> {
    // From home, the way their commute was timed by (on foot).
    let home = ctx
        .households
        .get(sim.household as usize)
        .and_then(|h| h.plot);
    if let Some(c) = sim.commute
        && c.home == home
        && let Some(&exit) = ctx.exits.get(c.exit)
        && ctx
            .briefs
            .get(sim.id as usize)
            .is_some_and(|b| b.plot.is_none() || b.plot == home)
    {
        return Some(exit);
    }
    let pos = ctx.nav.stairs.ground_pos(sim.pos);
    quickest_exit(ctx.travel, ctx.exits, pos).map(|i| ctx.exits[i])
}

/// The exit (an index into `exits`) quickest to get to on foot from `pos` (over the ground,
/// `StairMap::ground_pos`).
pub(crate) fn quickest_exit(
    travel: &crate::travel::Travel,
    exits: &[[f32; 2]],
    pos: [f32; 2],
) -> Option<usize> {
    let mode = crate::travel::Mode::Foot;
    // Nearest first by the straight line, which no way along the roads beats: past the first
    // whose straight line takes longer than the quickest found, none can be quicker.
    let mut order: Vec<(f32, usize)> = (0..exits.len())
        .filter(|&i| exits[i].iter().all(|v| v.is_finite()))
        .map(|i| (travel.eta_at_least(pos, exits[i], mode, 1.5), i))
        .collect();
    order.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut best: Option<(f32, usize)> = None;
    // (Where they'd get on the roads from here, found once for all the ways out.)
    let mut on = None;
    for (least, i) in order {
        if best.is_some_and(|(eta, _)| least > eta) {
            break;
        }
        let eta = travel.eta_via(
            pos,
            exits[i],
            mode,
            1.5,
            || *on.get_or_insert_with(|| travel.entry(pos, mode)),
            || travel.entry(exits[i], mode),
        );
        if best.is_none_or(|(b, j)| eta.total_cmp(&b).then(i.cmp(&j)).is_lt()) {
            best = Some((eta, i));
        }
    }
    best.map(|(_, i)| i)
}

/// The town's way out (an index into `World::exits`) nearest `pos` in a straight line, if it has
/// any.
pub(crate) fn nearest_exit(exits: &[[f32; 2]], pos: [f32; 2]) -> Option<usize> {
    let d = |i: usize| (exits[i][0] - pos[0]).hypot(exits[i][1] - pos[1]);
    (0..exits.len())
        .filter(|&i| exits[i].iter().all(|v| v.is_finite()))
        .min_by(|&a, &b| d(a).total_cmp(&d(b)))
}

/// The object a task holds a place at (using it, repairing it, cooking at it, the seat they eat
/// or phone from), released when it ends.
pub(crate) fn held_object(kind: TaskKind) -> Option<u32> {
    match kind {
        TaskKind::Use { object, .. } | TaskKind::Repair { object } => Some(object),
        // (Beside the crib, lifting the baby out.)
        TaskKind::Carry { from, stage: 0, .. } | TaskKind::Serve { from, stage: 0, .. } => {
            Some(from)
        }
        TaskKind::Cook { station, .. } | TaskKind::Dishes { station, .. } => station,
        TaskKind::Eat { seat, .. } | TaskKind::Phone { seat, .. } => seat,
        _ => None,
    }
}

/// A free crib at home for a baby (one with no other baby in it): they're put straight in.
pub(crate) fn crib_for(sim: &mut Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> Option<Task> {
    let home = ctx.households[sim.household as usize].plot?;
    let content = ctx.content;
    let (object, interaction) = ctx.index.on(home).iter().find_map(|&o| {
        let o = &objects[o as usize];
        let def = &content.objects[o.def];
        let i = def.interactions.iter().position(|it| it.baby_bed())?;
        let taken = o
            .users()
            .any(|u| ctx.briefs.get(u as usize).is_some_and(|b| b.baby));
        (!o.broken() && !taken && o.can_use_for(sim.id, def, i)).then_some((o.id, i))
    })?;
    let (fx, fz) = ctx
        .index
        .links
        .approach_of(object)
        .unwrap_or_else(|| objects[object as usize].front_tile(content));
    sim.pos = [fx as f32 + 0.5, fz as f32 + 0.5];
    Some(Task {
        kind: TaskKind::Use {
            object,
            interaction,
        },
        directed: false,
        routine: false,
    })
}

/// Whether partners `a` and `b` could have a baby now: at home together, grown-ups young enough,
/// none on the way, and room for one more (in the home and in the town).
pub(crate) fn could_conceive(ctx: &Ctx, a: usize, b: usize) -> bool {
    let Some(rules) = ctx.content.life.pregnancy else {
        return false;
    };
    let (x, y) = (&ctx.briefs[a], &ctx.briefs[b]);
    let h = x.household as usize;
    let fits = |s: &SimBrief| s.adult && s.age <= rules.max_age;
    let members = ctx
        .briefs
        .iter()
        .filter(|s| s.here && s.household as usize == h)
        .count();
    fits(x)
        && fits(y)
        && y.household == x.household
        && ctx.households[h].expecting.is_none()
        && members < ctx.content.rules.max_household
        && ctx.briefs.iter().filter(|s| s.here).count() < MAX_SIMS
}
