//! Getting to work: the commute, when to leave, and walking it.

use super::*;

/// How long someone's way to work or school takes (`Sim::commute`): in an open-world town the
/// walk to the way out of town quickest for them on foot (`world::quickest_exit`) and, past it,
/// `careerRules.commuteMinutes`, the part off the map. A cosy town's way is all `commuteMinutes`,
/// as ever (no `Commute`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Commute {
    /// The day it's for (carried over from the day before, `refresh_commute`) and the home it
    /// was worked out for.
    pub day: u32,
    pub home: Option<u32>,
    /// The exit (`World::exits` index) they walk to.
    pub exit: usize,
    /// Minutes on foot to it (from where they were at home when it was worked out, with
    /// `OUT_OF_THE_HOUSE`), and in all before work starts: they leave this long before.
    pub walk: f32,
    pub ticks: u64,
    /// On this bike of theirs at home (an object; None: on foot), picked as it's worked out
    /// (`ride::commute_bike`): the one they set off on (`ride::bike_for`).
    pub bike: Option<u32>,
    /// A ride out longer than `BIKE_MOST` on one of theirs: not ridden (the work taxi, or on foot).
    pub long_ride: bool,
    /// Worked out again from where they were shortly before they set off (`FIRM_UP_MINUTES`).
    pub firm: bool,
}

/// For each resident (by index), the walk out of town `firm_leave` last timed (`Travel::eta`,
/// minutes) and what from: where they were, the way out timed by, their pace and whether away
/// from home. Only ever what those give (the roads and the ways out are the town's for good), so
/// another resident in their place, or none kept (a load), comes to the same.
pub(crate) type WalkMemo = Vec<Option<([u32; 6], f32)>>;

/// This long before they'd set off, the walk is worked out again from where they are.
const FIRM_UP_MINUTES: f32 = 6.0;

/// `leave_for`, worked out again from where they are once it's close (`FIRM_UP_MINUTES`).
/// `fresh`: their commute was worked out anew just now, in this tick and from where they are
/// (`update`), at the cost of this many tiles searched.
pub(super) fn firm_leave(
    w: &mut World,
    i: usize,
    leave: u64,
    start: u64,
    day: u32,
    fresh: Option<u64>,
) -> u64 {
    let planned = leave_for(&w.sims[i], leave, start);
    if w.sims[i].commute.is_some_and(|c| !c.firm)
        && w.tick + ticks(FIRM_UP_MINUTES) >= planned
        && !heading_out(&w.sims[i])
    {
        // (Worked out just now, from here, the same again: firm as it is. Its searches still
        // count against the tick's budget, as when they were run twice: on Snowdale at 08:00 a
        // third of the walks out of town were searched for twice in a row, 49,000 tiles each.)
        match fresh {
            Some(nodes) => crate::path::charge_nodes(nodes),
            None => {
                w.sims[i].commute = None;
                refresh_commute(w, i, day);
            }
        }
        if let Some(c) = w.sims[i].commute.as_mut() {
            c.firm = true;
        }
        return leave_for(&w.sims[i], leave, start);
    }
    // Wherever they've got to since (out in the garden, at a neighbour's), at least as long as
    // the walk from there looks. Not out by taxi: it brings them home first, by the end of their
    // stay, and they set off from there. (Timed as a walk out of town from the place it took them
    // to, hours across a big world, the walk set their stay's end: home 38 minutes after getting
    // there, Maple Park, country 241141 at seed base 4.)
    let sim = &w.sims[i];
    if let Some(c) = sim.commute
        && !heading_out(sim)
        && !crate::taxi::out_by_taxi(&w.taxis, sim)
        && let Some(&exit) = w.exits.get(c.exit)
    {
        // At the pace they'd go from there: at home, on the bike their way is timed by
        // (`refresh_commute`); out on one, on that; else on foot. (Timed on foot from home, a
        // cyclist set off for work hours early, from an outing, and rode two hours across a big
        // world to it, Ashfield.)
        let here = w.briefs.get(i).and_then(|b| b.plot);
        let bike = if here.is_some() && here == c.home {
            c.bike
        } else {
            sim.riding
                .or(sim.bound_for_bike.filter(|_| sim.bike_parked.is_some()))
        };
        let speed = bike
            .and_then(|o| w.content.objects[w.objects.get(o as usize)?.def].vehicle)
            .map_or(1.0, |v| v.pace);
        let walk = sim.walk_per_minute(&w.content) * speed;
        // (From over the ground: a commute timed from bed upstairs had them setting off for
        // work in the middle of the night.)
        let from = w.stairs.ground_pos(sim.pos);
        // To the way out they'd walk to from there (`world::exit_on_foot`): from home or the
        // street, the one it was timed by; from somewhere else, the quickest from there. (Timed
        // to the way out nearest home from a neighbour's by the other one, they set off early,
        // or past the time to.)
        let away = w
            .briefs
            .get(i)
            .and_then(|b| b.plot)
            .is_some_and(|p| Some(p) != c.home);
        // (Worked out again only once they've moved: asked each tick, the same as last time
        // while they stay put.)
        let key = [
            from[0].to_bits(),
            from[1].to_bits(),
            exit[0].to_bits(),
            exit[1].to_bits(),
            walk.to_bits(),
            u32::from(away),
        ];
        let eta = match w.walk_memo.get(i) {
            Some(&Some((k, eta))) if k == key => eta,
            _ => {
                let exit = if away {
                    crate::world::quickest_exit(&w.travel, &w.exits, from)
                        .map_or(exit, |k| w.exits[k])
                } else {
                    exit
                };
                let eta = w.travel.eta(from, exit, crate::travel::Mode::Foot, walk);
                if w.walk_memo.len() <= i {
                    w.walk_memo.resize(i + 1, None);
                }
                w.walk_memo[i] = Some((key, eta));
                eta
            }
        };
        let minutes = eta * WALK_MARGIN + OUT_OF_THE_HOUSE;
        let total = ticks(minutes + w.content.career_rules.commute_minutes);
        if total > c.ticks
            && let Some(c) = w.sims[i].commute.as_mut()
        {
            c.walk = minutes;
            c.ticks = total;
            return leave_for(&w.sims[i], leave, start);
        }
    }
    planned
}

/// The longest ride out of town on a bike or e-scooter (game minutes: a bike's 4.5 m a second, on
/// the traffic's clock (`crate::DRIVE_SCALE`), is about 22 m a game minute, so some 2 km of
/// street, across a big world): longer, the way is ridden in the work taxi or walked
/// (`refresh_commute`), and nobody rides two hours in town.
const BIKE_MOST: f32 = 90.0;

/// Minutes added to the walk from the front door for getting out of the house to it, and a
/// share more for the rest of the way (a car passing, someone in the way).
pub const OUT_OF_THE_HOUSE: f32 = 5.0;
/// A guest with a need (but the room's) below this heads home (`end_visit_if_due`); one there for
/// less than `visits.leastMinutes` only with one below `autonomy.urgentBelow`. A visit or an outing
/// isn't set off on when a need would be down to it by then, with nothing there for it
/// (`world::lasts`).
pub(crate) const LOW_NEED: f32 = 0.15;
pub const WALK_MARGIN: f32 = 1.05;

/// When someone with work or school starting at `start` leaves (the content's `leave`, unless
/// they live in an open-world town: then by their walk, `Sim::commute`). (A worker sets off a
/// little before that: `early`.)
pub fn leave_for(sim: &crate::world::Sim, leave: u64, start: u64) -> u64 {
    match sim.commute {
        Some(c) => start.saturating_sub(c.ticks),
        None => leave,
    }
}

/// The most a worker sets off before their way to work says they must (game minutes). Off (0)
/// for now: at the full fill the sample worlds' checks did better with everyone off at the last
/// minute. With 20 minutes, seed bases 1, 3 and 4 failed where they passed without (a taxi
/// turning round and the car behind it waiting for each other for good at Alder Mill, 84 cars
/// held up behind them; visits ended within minutes at Stillwater and Stationgate), work taxis
/// were no more on time, and more cars were held up in view, not fewer.
///
/// Tried again at 20 for 0.8.3 (2026-11-17), against 0, and still left at 0: those four worlds
/// (Alder Mill and Foundryside of seed base 1, Stillwater of 3, Stationgate of 4) passed both ways,
/// none held up for good or in view either way, but the work taxi's riders came a little later
/// (0.04 minutes on average against 0.00, 316 rides against 331) and the queues at the ways out
/// weren't measured, so it wasn't shown to keep every check. Its tick rate was the same on the
/// seaside and forest cities filled in full (a game day's instructions -0.6 % and +0.7 %), but a
/// quarter of 0's on the river city (Ashbourne, 104734): there everyone off at the last minute
/// jams the morning's ways out, its slowest game hour at 07:00, a day's ticking four times the
/// others'. Worth the try again with the queues counted.
const EARLY_MOST: f32 = 0.0;

/// How much earlier than their way needs a household's workers set off for work (ticks, up to
/// `EARLY_MOST`), by a hash of the household (all in its car go together): the same every day
/// and after a load, so it is never saved. With every worker off at the last minute, a full
/// town's commuters met at the ways out in one queue. Only the setting off moves
/// (`schedule_work`): what's planned round it (getting up and ready, a visit or an outing, a
/// way home in time) keeps to the leave their way says, and whatever they're at is dropped for
/// work as ever. (Moved for all of it, every timing done by the leave moved, and a guest was
/// called home from a visit minutes after getting there: Foundryside, seed base 1.) Not for
/// school: its day starts for every pupil at once anyway.
pub(crate) fn early(household: u32) -> u64 {
    let h = (u64::from(household) + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15) >> 32;
    h % (ticks(EARLY_MOST) + 1)
}

/// How long `i`'s way to work takes (`Commute`), worked out when they first need it and when they
/// move house; in towns laid out before the open world, nothing: `commuteMinutes` is the whole
/// way, as it always was. A new day starts from the day before's (firmed up from where they were
/// as they set off, mostly): it is worked out again only shortly before they leave
/// (`firm_leave`), not for everyone in town at midnight, a walk across a big world each.
/// Whether it was worked out anew.
pub(crate) fn refresh_commute(w: &mut World, i: usize, day: u32) -> bool {
    // (Nowhere to go: no job, and not of an age for school.)
    let sim = &w.sims[i];
    let commutes = sim.job.is_some()
        || w.content.life.school.is_some()
            && w.content.life.stage_at(sim.age).is_some_and(|s| s.school);
    if w.exits.is_empty() || !commutes {
        w.sims[i].commute = None;
        return false;
    }
    let home = w
        .households
        .get(w.sims[i].household as usize)
        .and_then(|h| h.plot);
    if let Some(c) = w.sims[i].commute.as_mut()
        && c.home == home
    {
        if c.day != day {
            c.day = day;
            c.firm = false;
        }
        return false;
    }
    let sim = &w.sims[i];
    // From where they are at home, else from the front door.
    let at_home = home.is_some() && w.plot_at(sim.tile().0, sim.tile().1) == home;
    let from = match home {
        Some(p) if !at_home => w.plots[p as usize].arrival_point(),
        _ => sim.pos,
    };
    // (How far, from over the ground: see `firm_leave`.)
    let ground = w.stairs.ground_pos(from);
    // The next start of work or school, and the way out quickest on foot.
    let start = next_start(w, i);
    let Some(exit) = crate::world::quickest_exit(&w.travel, &w.exits, ground) else {
        w.sims[i].commute = None;
        return false;
    };
    // On foot, or on a bike of theirs at home (`ride.rs`): picked now, and the one they set off
    // on, so the time they leave is the time their way takes. (Not with a car to go in: it's
    // left for a housemate.)
    let by_car = start.is_some_and(|start| crate::cars::lead_ticks(w, i, start) > 0);
    let mut bike = (!by_car)
        .then(|| crate::ride::commute_bike(w, i, w.exits[exit]))
        .flatten();
    let on_foot = w.sims[i].walk_per_minute(&w.content);
    let walk = on_foot * bike.map_or(1.0, |(_, speed)| speed);
    // (The way is found once, for the ride and, if the ride is too long, for the walk: the same
    // way at another pace.)
    let searched = crate::path::nodes_searched();
    let metres = commute_metres(w, i, from, ground, exit);
    let (mut minutes, mut total) = walk_commute(w, metres, ground, walk, exit);
    // (A ride out longer than `BIKE_MOST` is ridden in the work taxi, `taxi/`, or walked.)
    let long_ride = bike.is_some() && minutes - OUT_OF_THE_HOUSE > BIKE_MOST;
    if long_ride {
        bike = None;
        // (Counted against the tick's search budget as when it was searched for again.)
        crate::path::charge_nodes(crate::path::nodes_searched() - searched);
        (minutes, total) = walk_commute(w, metres, ground, on_foot, exit);
    }
    let c = Commute {
        day,
        home,
        exit,
        walk: minutes,
        ticks: total,
        bike: bike.map(|(o, _)| o),
        long_ride,
        firm: false,
    };
    w.sims[i].commute = Some(c);
    true
}

/// The metres of an open-world town's way out for `i` from `from` (`ground`: over the ground) to
/// `exit`: the way they'd walk it (in the house's grounds and the public ways), if one is found.
fn commute_metres(
    w: &World,
    i: usize,
    from: [f32; 2],
    ground: [f32; 2],
    exit: usize,
) -> Option<f32> {
    let sim = &w.sims[i];
    let target = w.exits[exit];
    // The way they'd walk it (in the house's grounds and the public ways), or the estimate.
    let nav = w.nav();
    let goal = (target[0].floor() as i32, target[1].floor() as i32);
    let start = (from[0].floor() as i32, from[1].floor() as i32);
    let grounds = crate::world::grounds(
        &w.tile_plot,
        &w.halls,
        &w.plots,
        &w.households,
        &nav,
        sim,
        goal,
    );
    let reach = (w.reach_version == w.nav_version).then_some(&w.reach);
    crate::world::trip(
        &nav,
        reach,
        &w.travel,
        from,
        start,
        goal,
        target,
        Some(&grounds),
    )
    .map(|wp| {
        let wp: Vec<[f32; 2]> = wp.iter().map(|&p| w.stairs.ground_pos(p)).collect();
        crate::travel::length(ground, &wp)
    })
}

/// An open-world town's walk out over `metres` (`commute_metres`; None: the estimate from
/// `ground`, over the ground) at `walk` m a minute to `exit` (the way out quickest on foot): the
/// minutes to it (with `WALK_MARGIN` and `OUT_OF_THE_HOUSE`), and the ticks in all before work
/// starts.
fn walk_commute(
    w: &World,
    metres: Option<f32>,
    ground: [f32; 2],
    walk: f32,
    exit: usize,
) -> (f32, u64) {
    let target = w.exits[exit];
    let minutes = match metres {
        Some(m) => m / walk.max(1e-3),
        None => w
            .travel
            .eta(ground, target, crate::travel::Mode::Foot, walk),
    } * WALK_MARGIN
        + OUT_OF_THE_HOUSE;
    let total = minutes + w.content.career_rules.commute_minutes;
    (minutes, ticks(total))
}

/// A commute not needed for this long yet can wait for a tick with searches to spare (`update`):
/// its first use is when the work taxis are planned (`taxi::PLAN_LEAD` before they'd set off on
/// foot), and nobody's walk out of town is hours long.
const UNHURRIED_MINUTES: f32 = 6.0 * 60.0;

/// Whether `i`'s work or school doesn't start for `UNHURRIED_MINUTES` yet: neither today's, if
/// it's still to come or begun no longer than `late_minutes` ago (a game begun after they'd have
/// set off), nor the next.
pub(super) fn unhurried(w: &World, i: usize) -> bool {
    let late = ticks(w.content.career_rules.late_minutes);
    let first = start_on(&w.content, &w.sims[i], w.tick)
        .filter(|&start| start + late > w.tick)
        .or_else(|| next_start(w, i));
    first.is_none_or(|start| start > w.tick + ticks(UNHURRIED_MINUTES))
}

/// When `i`'s work or school next starts: today's, else tomorrow's (if either).
pub(crate) fn next_start(w: &World, i: usize) -> Option<u64> {
    let at = |tick: u64| start_on(&w.content, &w.sims[i], tick);
    at(w.tick)
        .filter(|&start| start >= w.tick)
        .or_else(|| at(w.tick + clock::TICKS_PER_DAY))
}

/// When `sim`'s work (or school) starts on the day of `tick`, if they have it that day.
pub(crate) fn start_on(content: &Content, sim: &Sim, tick: u64) -> Option<u64> {
    match &sim.job {
        Some(job) => shift_today(content, job, &sim.skills, tick),
        None => school_today(content, sim, tick),
    }
    .map(|(_, start, _)| start)
}
