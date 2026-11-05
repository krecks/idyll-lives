//! The work taxi (content `taxi`): in an open-world town, someone off to work or school a long
//! way out of town on foot, with no seat in a car (their household's or a neighbour's lift) and
//! no bike or e-scooter for a ride of `life::BIKE_MOST` at most, rides a taxi the work (or the
//! school) sends, shared with those going the same way about the same time.
//!
//! **Who rides.** A resident whose way out of town on foot (`life::Commute::walk`) takes longer
//! than `taxi.minWalkMinutes`, and whom the taxi gets out of town sooner than the walk, its wait at
//! the kerb included (`plan`). Not someone with a seat in a car to go in (`cars::lead_ticks`) or a
//! bike to ride (`Commute::bike`); nobody in a cosy town (no `Commute`: its way out is all
//! `commuteMinutes`).
//!
//! **Shared.** Those with the same career (or pupils, at school) who leave town by the same way out
//! and whose work starts within `taxi.window` minutes of the first's share one (`group`), up to
//! `taxi.seats` a taxi; more take more taxis.
//!
//! **The run there** (`Run`). Planned `PLAN_LEAD` before the first of them would have set off on foot: the riders
//! in a short order from the way in (`order`), the taxi's route in at that way out, nose first into
//! each rider's driveway mouth and backing out of it the way it goes on (`cars::taxi_stop`; with no
//! room in a driveway's mouth, at the kerb by it), and out again (`cars::taxi_route`). Timed back
//! from when it's to be out of town (the first one's work start less the part of the way off the
//! map), each stop `DWELL` long: when it comes in (`Run::enter`) and when it's at each stop
//! (`Stop::due`). Each rider leaves home in time to be at their kerb by then (`Stop::leave`, the
//! walk and `KERB_EARLY`), waits there ("Waiting for the taxi": `world::Phase::AtKerb`) and, as
//! the taxi stands at their stop, gets in by its door (`world::Phase::Boarding`, `boarded`): their
//! shift under way, out of sight as in a household's car, the rest of the ride and the way off the
//! map still to go. Not at the kerb `taxi.waitMinutes` after the taxi got there, they're left: it
//! goes on without them, and they make their own way (`release`).
//!
//! **Home.** Done at work, those who came by taxi go home by one (`off_work`): the first done
//! waits there for those of their group done within `taxi.window` minutes (each `Run::depart`),
//! then their taxi drives back into town by that way out, stands at each one's stop while they get
//! out (`cars::walk_home`) and leaves town again. Riders done later than that make their own way.
//!
//! **On the road.** A taxi is a car (`cars::Car::taxi`) in every way that matters: its route
//! (`cars::Route`), its claims (a driveway's mouth held while it stands in it), pulling over, the
//! last-resort guard. A taxi's car slot is kept once it's come (the snapshot's car rows, `KIND` a
//! taxi) and taken again by the next run.
//!
//! **Saved** (`save.rs`'s `taxis`, absent: none): the runs planned and under way, where each taxi
//! has got to and who's at which stop; their routes, slots and the bookings are worked out again
//! on load (`settle`).

use serde::{Deserialize, Serialize};

use crate::TICKS_PER_MINUTE;
use crate::cars::TaxiStop;
use crate::world::{Phase, Task, TaskKind, World};

/// A taxi's length and width (m: a family estate's).
pub const TAXI_LEN: f32 = 4.7;
pub const TAXI_WID: f32 = 1.85;
/// A run there is planned this long before the first of its riders would have set off on foot
/// (ticks), and for work or school starting no later than `PLAN_HORIZON` on.
const PLAN_LEAD: u64 = 60 * TICKS_PER_MINUTE;
const PLAN_HORIZON: u64 = 24 * 60 * TICKS_PER_MINUTE;
/// How long a taxi stands at a stop, in all, as its run is timed (ticks: the door, getting in or
/// out, the door again).
const DWELL: u64 = crate::cars::GET_IN_TICKS as u64 + 2 * TICKS_PER_MINUTE;
/// How long before the taxi is due a rider is at their kerb (ticks), and how much spare time a run
/// is timed with (ticks).
const KERB_EARLY: u64 = 3 * TICKS_PER_MINUTE;
const SLACK: u64 = 4 * TICKS_PER_MINUTE;
/// And this share of the ride more (per cent: held up at junctions and by those in its way).
const SLACK_SHARE: u64 = 20;
/// How long the rider's walk from the house to the kerb takes, at least (ticks).
const TO_KERB_LEAST: u64 = 2 * TICKS_PER_MINUTE;
/// How far out from where a taxi stands its rider waits for it (m, past its door: clear of it as
/// it pulls in).
const WAIT_OUT: f32 = 2.5;
/// A rider on their way to work this near the taxi standing at their stop gets in (m).
const NEAR_TAXI: f32 = 6.0;
/// A pupil's group (`group`): their school.
const SCHOOL: u32 = u32::MAX;

/// The runs planned and under way (`World::taxis`), and who rode one to work today (for the run
/// home).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Taxis {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runs: Vec<Run>,
    #[serde(default)]
    pub next_id: u32,
    /// Those who came to work by taxi, by their group (`group`): they go home by one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rode: Vec<(u32, (u32, u32))>,
    /// Those planned for no taxi for the start of work given (none fits them), not to be planned
    /// again for it. Not saved.
    #[serde(skip)]
    declined: Vec<(u32, u64)>,
    /// For checks: each rider's arrival at work by taxi against when their work started (ticks
    /// late; less than none, early), the latest 4096. Not saved.
    #[serde(skip)]
    pub arrivals: Vec<(u32, i64)>,
}

/// A taxi's run: to work (or school) or home.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: u32,
    /// Home from work (drop-offs), else to it (pick-ups).
    #[serde(default)]
    pub home: bool,
    /// Its group (`group`) and the way out it comes in and goes out by (a `World::exits` index).
    pub group: (u32, u32),
    /// When it comes into town (ticks), and (there) when it's to be out of it again.
    pub enter: u64,
    #[serde(default)]
    pub due_out: u64,
    /// Home: it leaves work at this tick (those done by then ride it), and comes in after the way
    /// off the map.
    #[serde(default)]
    pub depart: u64,
    pub stops: Vec<Stop>,
    /// The stop it's on its way to or stands at.
    #[serde(default)]
    pub next: u32,
    /// How far along its route it is (ticks), and since when it stands at its stop (a tick; 0:
    /// not standing).
    #[serde(default)]
    pub progress: f32,
    #[serde(default)]
    pub since: u64,
    /// Its car slot (`World::cars` index) while it's in town. Not saved.
    #[serde(skip)]
    pub car: Option<u32>,
    /// The ticks along its route of each stop. Not saved.
    #[serde(skip)]
    at: Vec<f32>,
}

/// A rider's stop on a run: their house's driveway (the plot), when the taxi is due there and
/// when they leave home for it; how far it's got.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    pub sim: u32,
    pub plot: u32,
    pub due: u64,
    #[serde(default)]
    pub leave: u64,
    pub state: StopState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopState {
    /// On their way to the kerb, or not yet.
    Booked,
    /// Getting in.
    Boarding,
    /// In it.
    Aboard,
    /// Left (not at the kerb in time), or out at home.
    Done,
}

/// The work taxi a resident is booked on (`world::Sim::taxi`): the run and their stop. Not saved:
/// worked out from the runs (`settle`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Booking {
    pub run: u32,
    pub stop: u8,
}

impl Run {
    fn stop_of(&self, sim: u32) -> Option<usize> {
        self.stops.iter().position(|s| s.sim == sim)
    }
}

/// Resident `i`'s group for the taxi: their career (pupils: `SCHOOL`) and the way out they leave
/// by.
fn group(w: &World, i: usize) -> Option<(u32, u32)> {
    let s = &w.sims[i];
    let c = s.commute?;
    let career = s.job.as_ref().map_or(SCHOOL, |j| j.career as u32);
    Some((career, c.exit as u32))
}

/// Where the taxi stops for resident `i` at home (`cars::taxi_stop`), as things stand.
fn stop_for(w: &World, i: usize) -> Option<TaxiStop> {
    let s = &w.sims[i];
    let plot = w.households.get(s.household as usize)?.plot?;
    let drive = w.drives.iter().find(|d| d.plot == plot)?;
    // (The household's car parked up the driveway: the taxi keeps clear of it.)
    let parked = w
        .cars
        .iter()
        .find(|c| !c.taxi && c.plot == plot && c.at_home())
        .map(|c| c.len);
    crate::cars::taxi_stop(&w.travel, drive, TAXI_LEN, parked)
}

/// Whether `i` would ride a taxi to work or school starting at `start`, and if so the stop at
/// home: an open-world commute a long way on foot, no car seat to go in, no bike to ride.
fn would_ride(w: &World, i: usize, start: u64) -> Option<TaxiStop> {
    let s = &w.sims[i];
    let c = s.commute?;
    let home = w.households.get(s.household as usize)?.plot;
    if !w.travel.open
        || !s.here()
        || c.home != home
        || c.bike.is_some()
        || c.walk - crate::life::OUT_OF_THE_HOUSE <= w.content.taxi.min_walk
        || crate::cars::lead_ticks(w, i, start) > 0
    {
        return None;
    }
    stop_for(w, i)
}

/// Once a tick (after the cars): plans the runs due (`plan`), drives those under way, the riders
/// in and out at their stops, and frees a taxi done with its run.
pub(crate) fn update(w: &mut World) {
    if !w.travel.open || w.travel.graph.is_empty() {
        return;
    }
    if w.tick.is_multiple_of(TICKS_PER_MINUTE) {
        plan(w);
        close_home_runs(w);
    }
    let mut k = 0;
    while k < w.taxis.runs.len() {
        if !run_on(w, k) {
            finish(w, k);
            w.taxis.runs.remove(k);
            continue;
        }
        k += 1;
    }
}

/// The runs to work due to be planned: everyone who'd ride, by group, those starting within
/// `taxi.window` of the first of a group, once one of them would set off on foot within
/// `PLAN_LEAD` (and not yet on their way).
fn plan(w: &mut World) {
    let tick = w.tick;
    let window = (w.content.taxi.window * TICKS_PER_MINUTE as f32) as u64;
    w.taxis.declined.retain(|d| d.1 + window > tick);
    let mut want: Vec<((u32, u32), u64, usize, u64)> = Vec::new();
    for i in 0..w.sims.len() {
        let Some(start) = crate::life::next_start(w, i) else {
            continue;
        };
        let s = &w.sims[i];
        if start <= tick + 30 * TICKS_PER_MINUTE
            || start > tick + PLAN_HORIZON
            || s.taxi.is_some()
            || s.away_until.is_some()
            || crate::life::heading_out(s)
            || w.taxis.declined.contains(&(i as u32, start))
        {
            continue;
        }
        let Some(g) = group(w, i) else { continue };
        let on_foot = start.saturating_sub(s.commute.map_or(0, |c| c.ticks));
        if would_ride(w, i, start).is_some() {
            want.push((g, start, i, on_foot));
        }
    }
    want.sort_by_key(|&(g, start, i, _)| (g, start, i));
    let mut k = 0;
    while k < want.len() {
        let (g, first, _, _) = want[k];
        let n = want[k..]
            .iter()
            .take_while(|m| m.0 == g && m.1 <= first + window)
            .count();
        let members: Vec<(usize, u64)> = want[k..k + n].iter().map(|m| (m.2, m.1)).collect();
        let soonest = want[k..k + n].iter().map(|m| m.3).min().unwrap_or(0);
        k += n;
        // (Only once one of them would soon set off on foot.)
        if soonest > tick + PLAN_LEAD {
            continue;
        }
        plan_group(w, g, &members);
    }
}

/// Plans the runs to work for `members` of group `g` (each with when their work starts): in
/// taxis of `taxi.seats` in a short order from the way in. Those no run fits (no way by the roads,
/// or the taxi no sooner than the walk) are declined for that start.
fn plan_group(w: &mut World, g: (u32, u32), members: &[(usize, u64)]) {
    let Some(&exit) = w.exits.get(g.1 as usize) else {
        return;
    };
    let mut left: Vec<(usize, u64, TaxiStop)> = members
        .iter()
        .filter_map(|&(i, start)| Some((i, start, would_ride(w, i, start)?)))
        .collect();
    for &(i, start) in members {
        if !left.iter().any(|m| m.0 == i) {
            w.taxis.declined.push((i as u32, start));
        }
    }
    let seats = w.content.taxi.seats;
    let pos = |w: &World, st: &TaxiStop| w.travel.graph.point(st.mouth);
    while !left.is_empty() {
        // (A short order: nearest the way in first, then on nearest to the last.)
        let mut chunk: Vec<(usize, u64, TaxiStop)> = Vec::new();
        let mut at = exit;
        while chunk.len() < seats && !left.is_empty() {
            let j = (0..left.len())
                .min_by(|&a, &b| {
                    let d = |m: &(usize, u64, TaxiStop)| crate::travel::dist(pos(w, &m.2), at);
                    d(&left[a])
                        .total_cmp(&d(&left[b]))
                        .then(left[a].0.cmp(&left[b].0))
                })
                .expect("some left");
            let m = left.remove(j);
            at = pos(w, &m.2);
            chunk.push(m);
        }
        // (No way by the roads, or no sooner than the walk for some: without them, once.)
        let mut tries = 0;
        while !chunk.is_empty() && tries < 3 {
            tries += 1;
            match time_run(w, g, exit, &chunk) {
                Ok(run) => {
                    book(w, run);
                    break;
                }
                Err(slow) if !slow.is_empty() => {
                    for i in slow {
                        if let Some(p) = chunk.iter().position(|m| m.0 == i) {
                            let m = chunk.remove(p);
                            w.taxis.declined.push((m.0 as u32, m.1));
                        }
                    }
                }
                Err(_) => break,
            }
        }
        for m in chunk {
            if w.sims[m.0].taxi.is_none() {
                w.taxis.declined.push((m.0 as u32, m.1));
            }
        }
    }
}

/// The run to work for `chunk` (each rider with their work's start and their stop), timed back
/// from the first one's start: Err with the riders the taxi gets out no sooner than their walk
/// (none: no way by the roads at all).
fn time_run(
    w: &World,
    g: (u32, u32),
    exit: [f32; 2],
    chunk: &[(usize, u64, TaxiStop)],
) -> Result<Run, Vec<usize>> {
    let stops: Vec<TaxiStop> = chunk.iter().map(|m| m.2).collect();
    let (route, at) =
        crate::cars::taxi_route(&w.travel, exit, &stops, exit, TAXI_LEN, TAXI_WID).ok_or(vec![])?;
    let off_map = crate::cars::off_map_ticks(w);
    let first = chunk.iter().map(|m| m.1).min().unwrap_or(0);
    let ride = route.ticks().ceil() as u64 + chunk.len() as u64 * DWELL;
    let due_out = first.saturating_sub(off_map);
    // (Spare time for the way: `SLACK` and a share of it, those in its way on the roads.)
    let spare = SLACK + ride * SLACK_SHARE / 100;
    // (Too late to come in time: none of them by taxi.)
    if due_out < w.tick + ride + spare {
        return Err(chunk.iter().map(|m| m.0).collect());
    }
    let enter = due_out - ride - spare;
    let mut slow = Vec::new();
    let mut out = Vec::new();
    for (k, &(i, start, ref st)) in chunk.iter().enumerate() {
        let due = enter + at[k].ceil() as u64 + k as u64 * DWELL;
        let sim = &w.sims[i];
        let walk = sim.walk_per_minute(&w.content);
        let door = w
            .households
            .get(sim.household as usize)
            .and_then(|h| h.plot)
            .map(|p| w.plots[p as usize].arrival_point())
            .unwrap_or(sim.pos);
        let to_kerb = ((w.travel.eta(
            door,
            wait_spot(&w.travel, st),
            crate::travel::Mode::Foot,
            walk,
        ) + crate::life::OUT_OF_THE_HOUSE)
            / crate::MINUTES_PER_TICK) as u64;
        let leave = due.saturating_sub(to_kerb.max(TO_KERB_LEAST) + KERB_EARLY);
        // (Sooner than the walk: from leaving home to out of town, against the walk out.)
        let walk_out = sim.commute.map_or(0, |c| c.ticks).saturating_sub(off_map);
        let by_taxi = due_out.saturating_sub(leave);
        if by_taxi >= walk_out || leave <= w.tick || start < due_out {
            slow.push(i);
        }
        out.push(Stop {
            sim: i as u32,
            plot: w.households[sim.household as usize]
                .plot
                .unwrap_or(u32::MAX),
            due,
            leave,
            state: StopState::Booked,
        });
    }
    if !slow.is_empty() {
        return Err(slow);
    }
    Ok(Run {
        id: 0,
        home: false,
        group: g,
        enter,
        due_out,
        depart: 0,
        stops: out,
        next: 0,
        progress: 0.0,
        since: 0,
        car: None,
        at,
    })
}

/// Books `run`'s riders on it and adds it.
fn book(w: &mut World, mut run: Run) {
    run.id = w.taxis.next_id;
    w.taxis.next_id = w.taxis.next_id.wrapping_add(1);
    for (k, st) in run.stops.iter().enumerate() {
        w.sims[st.sim as usize].taxi = Some(Booking {
            run: run.id,
            stop: k as u8,
        });
    }
    w.taxis.runs.push(run);
}

/// The stop rider `i` is booked at and when to leave home for it, if they're booked for the next
/// run to work (`life`'s leaving).
pub(crate) fn leave_at(w: &World, i: usize) -> Option<u64> {
    let b = w.sims[i].taxi?;
    let run = w.taxis.runs.iter().find(|r| r.id == b.run && !r.home)?;
    let st = run.stops.get(b.stop as usize)?;
    (st.state == StopState::Booked).then_some(st.leave)
}

/// Where `sim`, booked on a taxi to work, waits for it (`world`'s way to work), if it's still to
/// come for them.
pub(crate) fn kerb_for(ctx: &crate::world::Ctx, sim: &crate::world::Sim) -> Option<[f32; 2]> {
    let b = sim.taxi?;
    let run = ctx.taxis.runs.iter().find(|r| r.id == b.run && !r.home)?;
    let st = run.stops.get(b.stop as usize)?;
    if st.state != StopState::Booked || run.next > b.stop as u32 {
        return None;
    }
    let plot = ctx.households.get(sim.household as usize)?.plot?;
    let drive = ctx.drives.iter().find(|d| d.plot == plot)?;
    let parked = ctx
        .cars
        .iter()
        .find(|c| !c.taxi && c.plot == plot && c.at_home())
        .map(|c| c.len);
    let stop = crate::cars::taxi_stop(ctx.travel, drive, TAXI_LEN, parked)?;
    Some(wait_spot(ctx.travel, &stop))
}

/// Where a rider waits for the taxi at `stop`: beside where it'll stand in the driveway's mouth,
/// on the pavement out past its side, clear of it as it pulls in; at a kerb stop, on the pavement
/// by the driveway's mouth.
fn wait_spot(travel: &crate::travel::Travel, stop: &TaxiStop) -> [f32; 2] {
    let g = &travel.graph;
    let mp = g.point(stop.mouth);
    let road = &g.roads[stop.mouth.road as usize];
    if stop.kerb {
        let k = road.width / 2.0 - 0.5;
        return [mp[0] - stop.axis[0] * k, mp[1] - stop.axis[1] * k];
    }
    let side = [stop.axis[1], -stop.axis[0]];
    let k = TAXI_WID / 2.0 + WAIT_OUT;
    let edge = road.width / 2.0 - 0.6;
    [
        mp[0] - stop.axis[0] * edge + side[0] * k,
        mp[1] - stop.axis[1] * edge + side[1] * k,
    ]
}

/// A slot for a taxi in `World::cars`: one free (a taxi done with its run), else a new one (none
/// past `cars::MAX_CARS`).
fn slot(w: &mut World) -> Option<u32> {
    let busy: Vec<u32> = w.taxis.runs.iter().filter_map(|r| r.car).collect();
    if let Some(c) = (0..w.cars.len()).find(|&c| w.cars[c].taxi && !busy.contains(&(c as u32))) {
        return Some(c as u32);
    }
    if w.cars.len() >= crate::cars::MAX_CARS {
        return None;
    }
    let at = w.exits.first().copied().unwrap_or([0.0, 0.0]);
    w.cars.push(crate::cars::Car::taxi(at, TAXI_LEN, TAXI_WID));
    Some(w.cars.len() as u32 - 1)
}

/// The stops of run `k` as the taxi stops at them, and its route (`cars::taxi_route`); None: no
/// way.
fn route_of(w: &World, k: usize) -> Option<(crate::cars::Route, Vec<f32>)> {
    let run = &w.taxis.runs[k];
    let &exit = w.exits.get(run.group.1 as usize)?;
    let stops: Vec<TaxiStop> = run
        .stops
        .iter()
        .map(|st| {
            let drive = w.drives.iter().find(|d| d.plot == st.plot)?;
            let parked = w
                .cars
                .iter()
                .find(|c| !c.taxi && c.plot == st.plot && c.at_home())
                .map(|c| c.len);
            crate::cars::taxi_stop(&w.travel, drive, TAXI_LEN, parked)
        })
        .collect::<Option<_>>()?;
    crate::cars::taxi_route(&w.travel, exit, &stops, exit, TAXI_LEN, TAXI_WID)
}

/// Run `k` a tick on: into town when it's time, driving, standing at its stop. False: it's over
/// (out of town at the end of its route, or it couldn't run).
fn run_on(w: &mut World, k: usize) -> bool {
    let tick = w.tick;
    let run = &w.taxis.runs[k];
    if run.home && run.enter == 0 {
        // (Still gathering at work.)
        return true;
    }
    if run.car.is_none() {
        if tick < run.enter {
            return true;
        }
        // In it comes, along its route as it is now (a car parked or gone since it was planned).
        let Some((route, at)) = route_of(w, k) else {
            return false;
        };
        let Some(c) = slot(w) else {
            return false;
        };
        let progress = w.taxis.runs[k].progress;
        let car = &mut w.cars[c as usize];
        car.set_taxi_route(route, progress);
        let run = &mut w.taxis.runs[k];
        run.car = Some(c);
        run.at = at;
    }
    let c = w.taxis.runs[k].car.expect("in town") as usize;
    let next = w.taxis.runs[k].next as usize;
    let ticks = w.cars[c].taxi_ticks();
    if next < w.taxis.runs[k].stops.len() {
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
    crate::cars::drive_taxi(w, c);
    w.taxis.runs[k].progress = w.cars[c].progress;
    w.cars[c].progress < ticks
}

/// Run `k`'s taxi (car `c`) stands at its stop `next`: a rider gets in, or out; one not there in
/// time is left.
fn stand(w: &mut World, k: usize, c: usize, next: usize) {
    let tick = w.tick;
    crate::cars::stand_taxi(w, c);
    if w.taxis.runs[k].since == 0 {
        w.taxis.runs[k].since = tick;
    }
    let since = w.taxis.runs[k].since;
    let wait = (w.content.taxi.wait * TICKS_PER_MINUTE as f32) as u64;
    let st = w.taxis.runs[k].stops[next];
    let i = st.sim as usize;
    let done = |w: &mut World| {
        let run = &mut w.taxis.runs[k];
        run.next += 1;
        run.since = 0;
        w.cars[c].standing = false;
    };
    if w.taxis.runs[k].home {
        match st.state {
            StopState::Aboard => {
                // Home: out they get at their door and walk home.
                let seat = 1 + next % 4;
                w.taxis.runs[k].stops[next].state = StopState::Done;
                if w.sims.get(i).is_some_and(|s| s.here()) {
                    w.sims[i].away_until = None;
                    w.sims[i].taxi = None;
                    crate::cars::walk_home(w, i, Some((c, seat)));
                }
                w.taxis.runs[k].since = tick;
            }
            _ => {
                // (Their door shut behind them: on.)
                let out = w
                    .sims
                    .get(i)
                    .and_then(|s| s.current())
                    .is_some_and(|a| matches!(a.phase, Phase::Alighting { .. }));
                if !out || tick > since + DWELL {
                    done(w);
                }
            }
        }
        return;
    }
    match st.state {
        StopState::Booked => {
            // (At their kerb, or about it on their way to work: stepped out of the taxi's way.)
            let taxi_at = w.cars[c].spot;
            let waiting = w.sims.get(i).is_some_and(|s| {
                s.taxi.is_some_and(|b| b.run == w.taxis.runs[k].id)
                    && s.current().is_some_and(|a| {
                        matches!(a.phase, Phase::AtKerb { .. })
                            || matches!(a.task.kind, TaskKind::Work)
                                && crate::travel::dist(s.pos, taxi_at) < NEAR_TAXI
                    })
            });
            if waiting {
                // Their turn: to the taxi's door, and in.
                let seat = 1 + next % 4;
                let (d, _) = w.cars[c].taxi_door(&w.nav(), seat);
                {
                    let sim = &mut w.sims[i];
                    sim.bound_for_car = Some(c as u32);
                    sim.car_seat = seat as u8;
                    sim.car_door = d as u8;
                    sim.current = Some(crate::world::Activity {
                        task: Task {
                            kind: TaskKind::Work,
                            directed: true,
                            routine: false,
                        },
                        phase: Phase::Boarding { ticks: 0 },
                        tags: Default::default(),
                    });
                    w.taxis.runs[k].stops[next].state = StopState::Boarding;
                    return;
                }
            }
            if tick > since + wait {
                // Not there in time: on without them, and they make their own way.
                w.taxis.runs[k].stops[next].state = StopState::Done;
                release(w, i);
                done(w);
            }
        }
        StopState::Boarding => {
            // (Called away meanwhile: on without them.)
            let boarding = w.sims.get(i).is_some_and(|s| {
                s.current()
                    .is_some_and(|a| matches!(a.phase, Phase::Boarding { .. }))
                    && s.bound_for_car == Some(c as u32)
            });
            if !boarding {
                w.taxis.runs[k].stops[next].state = StopState::Done;
                release(w, i);
                done(w);
            }
        }
        StopState::Aboard | StopState::Done => done(w),
    }
}

/// Resident `i`, booked on a taxi, isn't riding it: unbooked, and on their way to work or school
/// their own way (from the kerb, or wherever they've got to).
pub(crate) fn release(w: &mut World, i: usize) {
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
    if heading {
        sim.current = None;
        sim.queue.push_front(Task {
            kind: TaskKind::Work,
            directed: true,
            routine: false,
        });
    }
    let home = match w.taxis.runs.iter_mut().find(|r| r.id == b.run) {
        Some(run) => {
            if let Some(st) = run.stops.get_mut(b.stop as usize)
                && st.state != StopState::Aboard
            {
                st.state = StopState::Done;
            }
            run.home
        }
        None => false,
    };
    // (Not booked on another for the same start: their own way it is.)
    if !home && let Some(start) = crate::life::next_start(w, i) {
        w.taxis.declined.push((i as u32, start));
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
        .position(|r| r.car == Some(c) && !r.home && r.stop_of(me).is_some())
    else {
        w.sims[i].bound_for_car = None;
        release(w, i);
        return;
    };
    let s = w.taxis.runs[k].stop_of(me).expect("just found");
    let rest_ride = (w.cars[c as usize].taxi_ticks() - w.cars[c as usize].progress).max(0.0);
    let left = (w.taxis.runs[k].stops.len() - s - 1) as u64;
    let rest = rest_ride.ceil() as u64 + left * DWELL + crate::cars::off_map_ticks(w);
    crate::life::start_taxi_shift(w, i, rest);
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
fn finish(w: &mut World, k: usize) {
    let tick = w.tick;
    let off_map = crate::cars::off_map_ticks(w);
    let run = w.taxis.runs[k].clone();
    if let Some(c) = run.car {
        w.cars[c as usize].park_taxi();
    }
    for st in &run.stops {
        let i = st.sim as usize;
        if !run.home && st.state == StopState::Aboard {
            if let Some(start) = crate::life::start_on(&w.content, &w.sims[i], tick) {
                let late = (tick + off_map) as i64 - start as i64;
                if w.taxis.arrivals.len() >= 4096 {
                    w.taxis.arrivals.remove(0);
                }
                w.taxis.arrivals.push((st.sim, late));
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
        if w.sims
            .get(i)
            .is_some_and(|s| s.taxi.is_some_and(|b| b.run == run.id))
        {
            release(w, i);
        }
    }
}

/// Resident `i` is done at work (or school): having come by taxi today, they wait there for one
/// home (true), with the others of their group done about when they are (`Run::depart`).
pub(crate) fn off_work(w: &mut World, i: usize) -> bool {
    let me = i as u32;
    let Some(p) = w.taxis.rode.iter().position(|r| r.0 == me) else {
        return false;
    };
    let g = w.taxis.rode.remove(p).1;
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
                group: g,
                enter: 0,
                due_out: 0,
                depart: tick + window,
                stops: Vec::new(),
                next: 0,
                progress: 0.0,
                since: 0,
                car: None,
                at: Vec::new(),
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

/// The runs home leaving work now: their stops in a short order from the way in, and when the
/// taxi comes into town (after the way off the map); each rider away till they're dropped off.
fn close_home_runs(w: &mut World) {
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
        w.taxis.runs[k].stops = order;
        w.taxis.runs[k].enter = tick + off_map;
        let Some((route, at)) = route_of(w, k) else {
            // (No way: they make their own way home from the way out.)
            w.taxis.runs[k].enter = tick;
            w.taxis.runs[k].at = Vec::new();
            continue;
        };
        let enter = tick + off_map;
        let stops = w.taxis.runs[k].stops.clone();
        for (s, st) in stops.iter().enumerate() {
            let back = enter + at[s].ceil() as u64 + s as u64 * DWELL;
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
                (Some((route, at)), Some(c)) => {
                    w.cars[c as usize].set_taxi_route(route, run.progress);
                    w.taxis.runs[k].car = Some(c);
                    w.taxis.runs[k].at = at;
                }
                _ => {
                    finish(w, k);
                    w.taxis.runs.remove(k);
                    continue;
                }
            }
        } else if (!run.home || run.enter != 0)
            && let Some((_, at)) = route_of(w, k)
        {
            w.taxis.runs[k].at = at;
        }
        k += 1;
    }
}

/// What resident `s` is doing with the taxi, for the UI (`view.rs`): waiting for it at the kerb,
/// in it to work (and when they're there, a tick), or on their way home in it (waiting at work for
/// it, or in it).
pub fn status(w: &World, s: &crate::world::Sim) -> Option<(&'static str, Option<u64>)> {
    let b = s.taxi?;
    let run = w.taxis.runs.iter().find(|r| r.id == b.run)?;
    let st = run.stops.get(b.stop as usize)?;
    if run.home {
        return (st.state == StopState::Aboard).then_some(("home", s.away_until));
    }
    match st.state {
        StopState::Aboard => Some(("work", Some(run.due_out + crate::cars::off_map_ticks(w)))),
        StopState::Booked | StopState::Boarding
            if s.current().is_some_and(|a| {
                matches!(a.phase, Phase::AtKerb { .. } | Phase::Boarding { .. })
            }) =>
        {
            Some(("wait", None))
        }
        _ => None,
    }
}

impl World {
    /// For tests: the taxi resident `i` is booked on, if any (its run, their stop).
    #[doc(hidden)]
    pub fn taxi_booking(&self, i: usize) -> Option<Booking> {
        self.sims.get(i)?.taxi
    }
}

impl Taxis {
    /// Nothing planned or under way, nobody waiting at work for one home (saves leave it out).
    pub fn is_empty(&self) -> bool {
        self.runs.is_empty() && self.rode.is_empty()
    }
}
