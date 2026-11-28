//! Visits and childcare: who minds the baby, knocking, doors and answering.

use super::*;

/// Whether a grown-up of household `h` other than `except` is at home on `plot` to look after
/// a baby (not on their way out: `leaving_home`; back from a visit once they're on the lot).
fn carer_home(w: &World, h: usize, plot: u32, except: Option<usize>) -> bool {
    let carer = |j: usize| {
        let s = &w.sims[j];
        Some(j) != except
            && s.here()
            && s.household as usize == h
            && s.away_until.is_none()
            && !leaving_home(w, s)
            && s.adult(&w.content)
            && w.plot_at(s.tile().0, s.tile().1) == Some(plot)
    };
    let found = w.household_sims(h as u32).any(carer);
    debug_assert_eq!(found, (0..w.sims.len()).any(carer));
    found
}

/// Who `i` would stay home for if they left for work now: a baby at home with no other
/// grown-up there, no babysitter booked for today and no money for one (content
/// `life.childcare`; without it, nobody stays home).
pub(super) fn stays_home_for(w: &World, i: usize) -> Option<usize> {
    let rules = w.content.life.childcare?;
    let h = w.sims[i].household as usize;
    let household = &w.households[h];
    let plot = household.plot?;
    if household.sitter_day == Some(clock::day(w.tick)) || affordable(rules.cost, household.funds) {
        return None;
    }
    let baby = w
        .sims
        .iter()
        .position(|s| s.here() && s.household as usize == h && w.content.life.baby(s.age))?;
    (!carer_home(w, h, plot, Some(i))).then_some(baby)
}

/// Who `i` would leave alone at home by going out now (visiting, say): a baby there with no
/// other grown-up at home, no babysitter booked for today and no money for one. Only the last
/// grown-up at home can.
pub(crate) fn no_cover_for(w: &World, i: usize) -> Option<usize> {
    let s = &w.sims[i];
    let plot = w.households[s.household as usize].plot?;
    let home = s.away_until.is_none()
        && s.visiting.is_none()
        && w.plot_at(s.tile().0, s.tile().1) == Some(plot);
    home.then(|| stays_home_for(w, i)).flatten()
}

/// Why going out would leave a baby home alone (`homebound`), short for the UI.
pub const NO_SITTER: &str = "Nobody to mind the baby";

/// Why `i` can't be sent out (off the home lot: visiting, the park, over to a friend's) now:
/// `NO_SITTER` when that would leave a baby alone at home (`no_cover_for`, as `check_cover`
/// refuses), else None. For the UI, next to what would take them out.
pub fn homebound(w: &World, i: usize) -> Option<&'static str> {
    no_cover_for(w, i).map(|_| NO_SITTER)
}

/// Refuses a player's order that takes `i` off the home lot (a visit, the park, the shops,
/// over to a friend's) when it would leave a baby alone at home: `no_cover_for`.
pub(crate) fn check_cover(w: &World, i: usize) -> Result<(), crate::Error> {
    let Some(baby) = no_cover_for(w, i) else {
        return Ok(());
    };
    let cost = w.content.life.childcare.map_or(0, |c| c.cost);
    Err(crate::Error::new(format!(
        "nobody to look after {}: not enough money for a babysitter (${cost})",
        w.sims[baby].name
    )))
}

/// Refuses a player's order that takes `i` to `plot` (None: off every plot) when that's away
/// from home and would leave a baby alone there (`check_cover`); orders at home go ahead.
pub(crate) fn check_cover_at(w: &World, i: usize, plot: Option<u32>) -> Result<(), crate::Error> {
    if plot != w.households[w.sims[i].household as usize].plot {
        check_cover(w, i)
    } else {
        Ok(())
    }
}

/// Whether `sim` is the last grown-up at home with a baby of theirs there (content
/// `life.childcare`): they don't go out on their own (visits, outings, meeting up, a chat off
/// the lot) and leave the baby alone; sent out by the player, a babysitter comes (see
/// `childcare`). Anyone else on their way out (`SimBrief::leaving`) isn't home for long.
pub(crate) fn minding_baby(sim: &Sim, ctx: &Ctx) -> bool {
    let content = ctx.content;
    if content.life.childcare.is_none() || !sim.adult(content) {
        return false;
    }
    let Some(home) = ctx.households[sim.household as usize].plot else {
        return false;
    };
    let me = sim.id as usize;
    let at_home =
        |b: &SimBrief| b.here && !b.away && b.household == sim.household && b.plot == Some(home);
    // (Only their household's: `Ctx::members`.)
    let theirs = ctx.members.of(sim.household);
    let minding = at_home(&ctx.briefs[me])
        && theirs.iter().any(|&j| {
            let b = &ctx.briefs[j as usize];
            b.baby && at_home(b)
        })
        && !theirs.iter().any(|&j| {
            let b = &ctx.briefs[j as usize];
            j as usize != me && b.adult && !b.leaving && at_home(b)
        });
    debug_assert_eq!(
        minding,
        at_home(&ctx.briefs[me])
            && ctx.briefs.iter().any(|b| b.baby && at_home(b))
            && !ctx
                .briefs
                .iter()
                .enumerate()
                .any(|(j, b)| j != me && b.adult && !b.leaving && at_home(b))
    );
    minding
}

/// A babysitter for baby `b` (content `life.childcare`): booked (and paid) for the day once
/// the baby is left with no grown-up at home while they're out (at work, visiting, wherever);
/// while nobody's home, the babysitter does the care interactions of the baby's crib whenever
/// a need they fill is low.
pub(super) fn childcare(w: &mut World, b: usize) {
    let Some(rules) = w.content.life.childcare else {
        return;
    };
    if !w.content.life.baby(w.sims[b].age) {
        return;
    }
    let h = w.sims[b].household as usize;
    let Some(plot) = w.households[h].plot else {
        return;
    };
    if carer_home(w, h, plot, None) {
        return;
    }
    // The baby's crib (or the home's), for what caring for them takes: what they hold a place
    // at, else the first crib on the home plot (from the objects by plot, unless objects came or
    // went since it was built this tick, when it looks through every object: rebuilding it here
    // costs more than that look, a third of a millisecond for 9,000 objects, and what comes later
    // in the tick would find another index than it does). A baby left with a sitter has this
    // asked every tick, so it looks at their own and their home's objects, not at every object
    // in town.
    let content = &w.content;
    let is_crib = |o: &crate::world::ObjectInstance| {
        content.objects[o.def]
            .interactions
            .iter()
            .any(|i| i.baby_bed())
    };
    let home_crib = || {
        if w.object_index.version == w.objects_version {
            w.object_index
                .on(plot)
                .iter()
                .map(|&o| &w.objects[o as usize])
                .find(|o| is_crib(o))
        } else {
            w.objects
                .iter()
                .zip(&w.object_plot)
                .find(|&(o, &p)| p == Some(plot) && is_crib(o))
                .map(|(o, _)| o)
        }
    };
    let crib = crate::world::held_by(&w.sims[b], &w.objects)
        .next()
        .or_else(home_crib)
        .map(|o| o.def);
    let Some(crib) = crib else { return };
    let day = clock::day(w.tick);
    if w.households[h].sitter_day != Some(day) {
        // Booked by a grown-up out at work, or else whoever else is out (with no grown-up in
        // the household at all, there's nobody to book one).
        let out = |s: &&Sim| s.here() && s.household as usize == h && s.adult(&w.content);
        let at_work = w
            .sims
            .iter()
            .position(|s| out(&s) && s.job.is_some() && s.away_until.is_some());
        let parent = at_work.or_else(|| w.sims.iter().position(|s| out(&s)));
        let Some(parent) = parent.filter(|_| affordable(rules.cost, w.households[h].funds)) else {
            return;
        };
        w.households[h].funds -= rules.cost;
        w.households[h].sitter_day = Some(day);
        w.events
            .push_amount(w.tick, EventKind::Babysitter, parent, b, rules.cost);
    }
    let content = &w.content;
    let baby = &mut w.sims[b];
    for inter in content.objects[crib]
        .interactions
        .iter()
        .filter(|i| i.cares())
    {
        let low =
            (0..content.needs.len()).any(|n| inter.care[n] > 0.0 && baby.needs[n] < rules.below);
        if low {
            for (n, g) in inter.care.iter().enumerate().take(content.needs.len()) {
                baby.needs[n] = (baby.needs[n] + g).clamp(0.0, 1.0);
            }
        }
    }
}

/// `i` is off to work or school (out of town on foot, on a bike, in the car), with `rest` ticks
/// of the way still to go (`still_to_go`): away until it's over. Late for work when that gets
/// them there more than `careerRules.lateGraceMinutes` after it starts. (In a work taxi,
/// `taxi::boarded`: `rest` is the ride and the way off the map.)
pub(crate) fn start_shift(w: &mut World, i: usize, rest: u64) {
    let tick = w.tick;
    let content = &w.content;
    let mood = w.sims[i].mood(content);
    let sim = &mut w.sims[i];
    let skills = sim.skills;
    // A pupil: school until the afternoon.
    if sim.job.is_none() {
        if let Some((_, start, end)) = school_today(content, sim, tick) {
            sim.school_day = clock::day(tick);
            sim.shift_from = start;
            sim.away_until = Some(end.max(start).max(tick + 1));
            sim.queue.clear();
        }
        return;
    }
    let Some(job) = sim.job.as_mut() else { return };
    let Some((_, start, end)) = shift_today(content, job, &skills, tick) else {
        return;
    };
    job.last_shift_day = clock::day(tick);
    job.shift_mood = mood;
    let due = start.saturating_sub(rest);
    job.late = tick > due + ticks(content.career_rules.late_grace_minutes);
    // Early arrivals still work until the shift's end; late ones lose that time. Before
    // it starts they're on their way (`at_work`).
    sim.shift_from = start;
    sim.away_until = Some(end.max(start).max(tick + 1));
    sim.queue.clear();
}

/// What's left of the way to work once they're off (`start_shift`), as their way was timed: the
/// part off the map (`cars::off_map_ticks`) and, before it, `out` (the drive out of town).
fn still_to_go(w: &World, out: u64) -> u64 {
    out + crate::cars::off_map_ticks(w)
}

pub(super) fn apply_transition(w: &mut World, i: usize, transition: Transition) {
    let tick = w.tick;
    match transition {
        // At the way out of town (or, with no way out, where they are).
        Transition::Work => start_shift(w, i, still_to_go(w, 0)),
        Transition::Visit { plot, directed } => {
            // On their way again to the place they're visiting: there already (a walk across it
            // set off again after a pause, `world::decide`), or not yet (on from the bike they
            // walked over to at home, `ride.rs`): the visit goes on as it was, not begun again
            // (nor told again), its goodbye queued for a need about to run out still withdrawn
            // once that's seen to there (`Visit::leaving`; begun again, it stayed queued, and a
            // guest let in went home 20 minutes in with the need long seen to). Sent there by the
            // player on the way, it's as long as a visit they ask for (`directedHours`).
            let directed_until = tick + ticks(w.content.visits.directed_hours * 60.0);
            if let Some(v) = w.sims[i].visiting.as_mut().filter(|v| v.plot == plot) {
                if directed && v.arrived == 0 {
                    v.until = v.until.max(directed_until);
                }
                return;
            }
            begin_visit(w, i, plot, directed);
        }
        // Into a work taxi (`taxi/`).
        Transition::Board { car } if w.cars.get(car as usize).is_some_and(|c| c.taxi) => {
            crate::taxi::boarded(w, i, car);
        }
        Transition::Board { car } => {
            // Gone without them (or not theirs to take now): on foot after all.
            if !crate::cars::may_board(w, i, car) {
                w.sims[i].queue.push_front(Task {
                    kind: TaskKind::Work,
                    directed: true,
                    routine: false,
                });
                return;
            }
            // (The drive out of town and on to work still to go: `cars::lead_ticks`.)
            let out = w
                .cars
                .get(car as usize)
                .map_or(0, crate::cars::drive_out_ticks);
            start_shift(w, i, still_to_go(w, out));
            if w.sims[i].away_until.is_some() {
                crate::cars::board(w, i, car);
            }
        }
        Transition::Drive {
            car,
            plot,
            directed,
        } => {
            on_the_way(w, i, plot, directed);
            crate::cars::start_trip(w, i, car, plot, directed);
        }
        Transition::DriveHome { car } => {
            // (Off home: not off on another visit for `visits.againMinutes`.)
            if w.sims[i].visiting.take().is_some() {
                w.sims[i].back_at = tick;
            }
            crate::cars::drive_home(w, i, car);
        }
        Transition::Ride {
            car,
            plot,
            directed,
        } => {
            if let Some(plot) = plot {
                on_the_way(w, i, plot, directed);
            }
            crate::cars::ride(w, i, car, plot, directed);
        }
        Transition::Home => {
            // (Back from a visit or an outing: not off on another for `visits.againMinutes`.)
            if w.sims[i].visiting.take().is_some() {
                w.sims[i].back_at = tick;
            }
        }
        Transition::Taxi { plot, directed } => {
            // (No taxi after all: on foot, `taxi::Taxis::no_taxi`.)
            if !crate::taxi::book_leisure(w, i, plot, directed) {
                w.taxis.turned_down(i as u32, plot);
                w.sims[i].queue.push_front(Task {
                    kind: TaskKind::Visit { plot },
                    directed,
                    routine: false,
                });
            }
        }
        Transition::Answered { guest } => answer(w, i, guest as usize),
    }
}

/// `i` sets off to visit `plot` (`Transition::Visit`): a guest from now on, till `visits.hours`
/// from now (the player's visits `directedHours`), and the story told if the hosts are home (with
/// door rules, when they're let in: `answer`).
fn begin_visit(w: &mut World, i: usize, plot: u32, directed: bool) {
    let tick = w.tick;
    let visits = &w.content.visits;
    let hours = if directed {
        visits.directed_hours
    } else {
        visits.hours
    };
    // (Till they're there: then it's from then, `world::arrive`, or from being let in,
    // `answer`.)
    let until = tick + ticks(hours * 60.0);
    w.sims[i].visiting = Some(Visit {
        plot,
        until,
        idle: 0,
        farewell: 0,
        offer_at: 0,
        arrived: 0,
        leaving: false,
        home_by: false,
    });
    // With door rules the story is told when they're let in (`answer`).
    if w.content.visits.door.is_some() && !w.plots[plot as usize].public {
        return;
    }
    // Tell the story if the hosts are home.
    let host = w
        .households
        .iter()
        .find(|h| h.plot == Some(plot))
        .map(|h| h.id);
    if let Some(host) = host
        && let Some(j) = w.sims.iter().position(|s| {
            s.here() && s.household == host && w.plot_at(s.tile().0, s.tile().1) == Some(plot)
        })
    {
        w.events.push(tick, EventKind::Visited, i, j, None);
    }
}

/// At a car's door to drive or ride to `plot` (`Transition::Drive`, `Transition::Ride`): on their
/// way to a visit, as they've been since they set off for the car, or from now when they set off
/// at its door (stood there already, they got to it in the step they set off: getting in took the
/// place of the visit's start, `Sim::transition`, and they drove off a guest of nowhere, at home as
/// far as moving house knew, `lifecycle::may_move`).
fn on_the_way(w: &mut World, i: usize, plot: u32, directed: bool) {
    if !w.sims[i].visiting.is_some_and(|v| v.plot == plot) {
        begin_visit(w, i, plot, directed);
    }
}

/// The plot a visitor is knocking at, and how long they've waited and who is coming, if they are.
pub(super) fn knocking(sim: &crate::world::Sim) -> Option<(u32, u32, Option<u32>)> {
    match sim.current.as_ref()? {
        Activity {
            task:
                Task {
                    kind: TaskKind::Visit { plot },
                    ..
                },
            phase: Phase::Knocking { ticks, host },
            ..
        } => Some((*plot, *ticks, *host)),
        _ => None,
    }
}

/// Visitors at front doors (`visits.door`): someone at home is sent to answer; with nobody to
/// answer, the visitor gives up after a while and goes home.
pub(super) fn doors(w: &mut World) {
    let Some(rules) = w.content.visits.door.clone() else {
        return;
    };
    let tick = w.tick;
    let wait = ticks(rules.wait_minutes);
    for i in 0..w.sims.len() {
        let Some((plot, waited, host)) = knocking(&w.sims[i]) else {
            continue;
        };
        let waited = u64::from(waited);
        // Still coming? (They may have been called away, gone to bed or to work.)
        let host = host.filter(|&h| {
            let s = &w.sims[h as usize];
            let answering =
                |t: &Task| matches!(t.kind, TaskKind::Answer { guest } if guest as usize == i);
            s.current.as_ref().is_some_and(|a| answering(&a.task)) || s.queue.iter().any(answering)
        });
        let host = host.or_else(|| {
            let h = door_host(w, plot, i)?;
            let World {
                sims,
                content,
                objects,
                ..
            } = &mut *w;
            let s = &mut sims[h];
            if s.current.is_some() {
                end_activity(s, content, objects);
            }
            s.queue.push_front(Task {
                kind: TaskKind::Answer { guest: i as u32 },
                directed: false,
                routine: false,
            });
            Some(h as u32)
        });
        if let Some(Activity {
            phase: Phase::Knocking { host: h, .. },
            ..
        }) = w.sims[i].current.as_mut()
        {
            *h = host;
        }
        // Nobody comes (or whoever was coming never makes it): home again. Someone up and about
        // at home who is busy (in a conversation, say) comes once they're free: worth the wait.
        let someone_up = host.is_some() || awake_at_home(w, plot, i);
        if (!someone_up && waited >= wait) || waited >= wait * 3 {
            let sim = &mut w.sims[i];
            sim.current = None;
            if !sim.queue.iter().any(|t| matches!(t.kind, TaskKind::GoHome)) {
                sim.queue.push_front(Task {
                    kind: TaskKind::GoHome,
                    directed: false,
                    routine: false,
                });
            }
            let household = w.households.iter().position(|h| h.plot == Some(plot));
            if let Some(member) = household.and_then(|h| {
                w.sims
                    .iter()
                    .position(|s| s.here() && s.household as usize == h)
            }) {
                w.events.push(tick, EventKind::NobodyHome, i, member, None);
            }
        }
    }
}

/// Whether a grown-up of the household living at `plot` is on it, at home and awake (who
/// would answer the door to `guest` once free).
fn awake_at_home(w: &World, plot: u32, guest: usize) -> bool {
    door_people(w, plot, guest).next().is_some()
}

/// The grown-ups of the household living at `plot` (but `guest`) who are on it, at home and
/// awake: who answer the door (`door_host`, once free).
fn door_people(
    w: &World,
    plot: u32,
    guest: usize,
) -> impl Iterator<Item = (usize, &crate::world::Sim)> {
    let household = w.households.iter().position(|h| h.plot == Some(plot));
    w.sims.iter().enumerate().filter(move |&(j, s)| {
        household.is_some_and(|h| s.household as usize == h)
            && j != guest
            && s.here()
            && s.away_until.is_none()
            && s.visiting.is_none()
            && s.pose != Pose::Lie
            && s.adult(&w.content)
            && w.plot_at(s.tile().0, s.tile().1) == Some(plot)
    })
}

/// `door_people` by the tick's briefs, for a visitor deciding to set off (`world::pick_autonomous`):
/// whether `b` of `household` is on its home `plot`, a grown-up, up and about, and not on their
/// way out.
pub(crate) fn opens_door(b: &crate::world::SimBrief, household: u32, plot: u32) -> bool {
    b.household == household && b.plot == Some(plot) && !b.away && b.adult && !b.lying && !b.leaving
}

/// How far (m) out from its front door a visitor stands to knock: an arm's length.
const KNOCK_REACH: f32 = 0.45;

/// Where a visitor knocks at `plot`: at the front door (the door from outdoors into the house
/// nearest the plot's `entry`, within a few tiles of it), `KNOCK_REACH` out from it, facing it;
/// and the tile they stand on. At a flat, its own door off the landing (`Plot::door`, the way in
/// to `Plot::inside`). None without a door like that, or with something in front of it.
pub(crate) fn knock_spot(nav: &crate::path::NavGrid, plot: &crate::world::Plot) -> Option<Knock> {
    let ((x, z), (dx, dz)) = match (plot.door, plot.inside) {
        (Some((x, z)), Some(inside)) => {
            let (ix, iz) = (inside[0].floor() as i32, inside[1].floor() as i32);
            let (dx, dz) = (ix - x, iz - z);
            if dx.abs() + dz.abs() != 1
                || !nav.tile_free(x, z)
                || !nav.lot.in_bounds(ix, iz)
                || nav.lot.edge_between(x, z, ix, iz) != crate::lot::Edge::Door
            {
                return None;
            }
            ((x, z), (dx, dz))
        }
        _ => front_door(nav, plot.arrival_point(), |x, z| plot.contains(x, z))?,
    };
    let door = [
        x as f32 + 0.5 + dx as f32 * 0.5,
        z as f32 + 0.5 + dz as f32 * 0.5,
    ];
    let at = [
        door[0] - dx as f32 * KNOCK_REACH,
        door[1] - dz as f32 * KNOCK_REACH,
    ];
    Some(((x, z), at, (dx as f32).atan2(dz as f32)))
}

/// A front door (`front_door`): the tile outdoors before it, and the step in through it.
pub(crate) type FrontDoor = ((i32, i32), (i32, i32));

/// The door from outdoors into a building nearest `entry` (within a few tiles of it, from a tile
/// `on` the lot): the free tile outdoors before it, and the step in through it. None: no door
/// like that, or something in front of it.
pub(crate) fn front_door(
    nav: &crate::path::NavGrid,
    entry: [f32; 2],
    on: impl Fn(i32, i32) -> bool,
) -> Option<FrontDoor> {
    let lot = nav.lot;
    let (ex, ez) = (entry[0].floor() as i32, entry[1].floor() as i32);
    let mut best: Option<(f32, FrontDoor)> = None;
    for z in ez - 3..=ez + 3 {
        for x in ex - 3..=ex + 3 {
            if !on(x, z)
                || !lot.in_bounds(x, z)
                || lot.room_at(x, z) != crate::lot::OUTDOORS
                || !nav.tile_free(x, z)
            {
                continue;
            }
            for (dx, dz) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let (ix, iz) = (x + dx, z + dz);
                if !lot.in_bounds(ix, iz)
                    || lot.room_at(ix, iz) == crate::lot::OUTDOORS
                    || lot.edge_between(x, z, ix, iz) != crate::lot::Edge::Door
                {
                    continue;
                }
                let door = [
                    x as f32 + 0.5 + dx as f32 * 0.5,
                    z as f32 + 0.5 + dz as f32 * 0.5,
                ];
                let d = (door[0] - entry[0]).hypot(door[1] - entry[1]);
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, ((x, z), (dx, dz))));
                }
            }
        }
    }
    best.map(|(_, k)| k)
}

/// Where a visitor knocks (`knock_spot`): the tile, the point and the way they face.
pub(crate) type Knock = ((i32, i32), [f32; 2], f32);

/// Who answers the door of `plot`: the grown-up of the household living there who is nearest to
/// the door, at home, awake, and not busy with something the player asked for or a conversation.
fn door_host(w: &World, plot: u32, guest: usize) -> Option<usize> {
    let door = w.plots[plot as usize].doorstep();
    // (Not one the player walks: sent, they'd never come, and the guest wait on.)
    let free = |s: &crate::world::Sim| {
        !s.steered
            && s.current.as_ref().is_none_or(|a| {
                !a.task.directed
                    && !matches!(a.phase, Phase::Conversing { .. })
                    && !matches!(
                        a.task.kind,
                        TaskKind::Work
                            | TaskKind::GoHome
                            | TaskKind::Visit { .. }
                            | TaskKind::Answer { .. }
                    )
            })
            && !s.queue.front().is_some_and(|t| t.directed)
    };
    door_people(w, plot, guest)
        .filter(|&(_, s)| s.engaged_with.is_none() && free(s))
        .map(|(j, s)| (j, (s.pos[0] - door[0]).hypot(s.pos[1] - door[1])))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(j, _)| j)
}

/// Host `h` opened the door to `guest`: friends (and acquaintances) come in, people the host
/// can't stand and visitors at an unreasonable hour are sent away.
fn answer(w: &mut World, h: usize, guest: usize) {
    let Some(rules) = w.content.visits.door.clone() else {
        return;
    };
    if w.sims.get(guest).and_then(knocking).is_none() {
        return;
    }
    let tick = w.tick;
    let hour = clock::hour(tick);
    let visits = &w.content.visits;
    let sensible_hour = (visits.earliest_hour - 2.0..=visits.latest_hour + 2.0).contains(&hour);
    let welcome =
        sensible_hour && w.relationships.get(h, guest).friendship >= rules.welcome_friendship;
    w.sims[guest].current = None;
    if welcome {
        // The visit is from now: the way there doesn't count (a longer one the player asked
        // for keeps its end).
        let length = ticks(w.content.visits.hours * 60.0);
        if let Some(v) = w.sims[guest].visiting.as_mut() {
            v.until = v.until.max(tick + length);
            v.arrived = tick;
        }
        w.events.push(tick, EventKind::Visited, guest, h, None);
        if let Some(greet) = rules.greet {
            w.sims[h].queue.push_front(Task {
                kind: TaskKind::Social {
                    target: guest as u32,
                    social: greet,
                },
                directed: false,
                routine: false,
            });
        }
        return;
    }
    let World {
        sims,
        content,
        events,
        ..
    } = &mut *w;
    let sim = &mut sims[guest];
    sim.queue.push_front(Task {
        kind: TaskKind::GoHome,
        directed: false,
        routine: false,
    });
    if let Some(m) = rules.turned_away {
        social::add_feeling(&mut sim.feelings, m, &content.feelings, tick);
    }
    events.push(tick, EventKind::TurnedAway, guest, h, None);
}
