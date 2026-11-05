//! The work taxi (content `taxi`): in an open-world town, someone off to work or school a long
//! way out of town on foot, with no seat in a car (their household's or a neighbour's lift) and
//! no bike or e-scooter for a ride of `life::BIKE_MOST` at most, rides a taxi the work (or the
//! school) sends, shared with those going the same way about the same time.
//!
//! **Who rides.** A resident whose way out of town on foot (`life::Commute::walk`) takes longer
//! than `taxi.minWalkMinutes`, and whom the taxi gets out of town sooner than the walk, its wait at
//! the kerb included (`time_run`). Not someone with a seat in a car to go in (`cars::lead_ticks`)
//! or a bike to ride (`Commute::bike`); nobody in a cosy town (no `Commute`: its way out is all
//! `commuteMinutes`). With or without a driveway (`home_way`: else by the front path).
//!
//! **Shared.** Those who leave town by the same way out, whatever their work (or school), and
//! whose work starts within `taxi.window` minutes of the first's share one (`group`), up to
//! `taxi.seats` a taxi, housemates side by side; more take more taxis. Those already too late to be
//! there in time on foot (a game begun) share one late run that way, whenever their work starts.
//!
//! **The run there** (`Run`). Planned `PLAN_LEAD` before the first of them would have set off on
//! foot: the riders in a short order from the way in, a stand at the kerb for each house
//! (`kerb_at`: in the lane, parallel to it, where the road runs straight for the body, clear of
//! junctions and of another taxi standing then), and the taxi's way in at that way out, to each
//! stand and out again as a car drives (`plan_way`, `lay_route`): on along its lane from a stand,
//! turning round only in a dead end's turning circle or nose first into a side street at a
//! junction, else round the block; round where cars are held up now (`jammed`). Timed back from when
//! it's to be out of town (the first one's work start less the part of the way off the map), each
//! stop `DWELL` long; too late for that (a game begun after they'd have set off, a rider missed
//! before), as soon as it can come, if that's still sooner than on foot (a late run). Each rider
//! leaves home in time to be at their kerb by then, or later as the taxi runs late (`leave_at`),
//! waits there ("Waiting for the taxi": `world::Phase::AtKerb`) and, as the taxi stands for them,
//! walks to its door and gets in (`door_for`, `world::Phase::Boarding`, `boarded`): their shift
//! under way, out of sight as in a household's car, the rest of the ride and the way off the map
//! still to go. Not at its door soon after it got there, they're left: it goes on without them, and
//! they're planned once more (`release`), else they make their own way. Someone just off on foot
//! when they're booked (as a game begins) goes back for it (`setting_off`, `book`).
//!
//! **Home.** Done at work, those who came by taxi, and those come on foot with a long walk home,
//! go home by one (`off_work`): the first done waits there for those of their group done within
//! `taxi.window` minutes (each `Run::depart`), then their taxi drives back into town by that way
//! out, stands at each one's stop while they get out (`cars::walk_home`) and leaves town again.
//! Riders done later than that make their own way.
//!
//! **On the road.** A taxi is a car (`cars::Car::taxi`) in every way that matters: its route
//! (`cars::Route`), its claims (its stretch of the lane held while it stands there, a turn round
//! while it turns), pulling over, the last-resort guard. A taxi's car slot is kept once it's come
//! (the snapshot's car rows, `KIND` a taxi) and taken again by the next run.
//!
//! **Saved** (`save.rs`'s `taxis`, absent: none): the runs planned and under way, where each taxi
//! has got to and who's at which stop; their routes, stands, slots and the bookings are worked out
//! again on load (`settle`).

use serde::{Deserialize, Serialize};

use crate::TICKS_PER_MINUTE;
use crate::world::{Phase, Task, TaskKind, World};

/// A taxi's length and width (m: a family estate's).
pub const TAXI_LEN: f32 = 4.7;
pub const TAXI_WID: f32 = 1.85;
/// A run there is planned this long before the first of its riders would have set off on foot
/// (ticks: before their walk is firmed up and they're off, `life::firm_leave`), and for work or
/// school starting no later than `PLAN_HORIZON` on.
const PLAN_LEAD: u64 = 3 * 60 * TICKS_PER_MINUTE;
/// Someone already off to work on foot this near their door is still booked on a taxi (m: off
/// to their kerb instead, `setting_off`).
const SETTING_OFF: f32 = 25.0;
const PLAN_HORIZON: u64 = 24 * 60 * TICKS_PER_MINUTE;
/// How long a taxi stands at a stop, in all, as its run is timed (ticks: the door, getting in or
/// out, the door again).
const DWELL: u64 = crate::cars::GET_IN_TICKS as u64 + TICKS_PER_MINUTE;
/// How long before the taxi is due a rider is at their kerb (ticks), and how much spare time a run
/// is timed with (ticks).
const KERB_EARLY: u64 = 4 * TICKS_PER_MINUTE;
const SLACK: u64 = 4 * TICKS_PER_MINUTE;
/// And this share of the ride more (per cent: held up at junctions and by those in its way).
const SLACK_SHARE: u64 = 30;
/// How long the rider's walk from the house to the kerb takes, at least (ticks).
const TO_KERB_LEAST: u64 = 3 * TICKS_PER_MINUTE;
/// How far along the road from their driveway's middle a rider waits for the taxi at least (m,
/// `wait_spot`).
const WAIT_ASIDE: f32 = 2.0;
/// The taxi stands at a stop for a rider not yet at the kerb for this long at most (ticks: at its
/// door within it at their pace, else they're left), and for one called to its door this long
/// in all (ticks: held up on their way to it, they're left).
const ARRIVE_SOON: u64 = 10 * TICKS_PER_MINUTE;
const TO_DOOR_MOST: u64 = 2 * ARRIVE_SOON;
const WAY_ROUND: f32 = 1.3;

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
    /// Those a taxi went on without for the start of work given, once: planned again (a late
    /// run, sooner out of town than on foot), but not after a second. Not saved.
    #[serde(skip)]
    missed: Vec<(u32, u64)>,
    /// Each car on its way: where it stood still last (none: not on its way), and since when (a tick), for the
    /// places held up (`jammed`). Not saved.
    #[serde(skip)]
    still: Vec<(Option<[f32; 2]>, u64)>,
    /// For tests: junctions' boxes held to be jammed whatever (`jammed`). Not saved.
    #[doc(hidden)]
    #[serde(skip)]
    pub jams: Vec<usize>,
    /// For checks: each rider's arrival at work by taxi against when their work started (ticks
    /// late; less than none, early), the latest 4096, of the runs timed to be there in time; and
    /// of the late runs (`time_run`: too late for that, sooner than on foot). Not saved.
    #[serde(skip)]
    pub arrivals: Vec<(u32, i64)>,
    #[serde(skip)]
    pub late_arrivals: Vec<(u32, i64)>,
    /// Those put down at a place by a leisure taxi, walking in (`alight_at`): a taxi back is booked
    /// once they're there (`book_backs`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub out_at: Vec<(u32, u32)>,
    /// Those turned down for a taxi to a place just now (`book_leisure` found none), walking there
    /// instead: not asked again till it's cleared (each hour). Not saved.
    #[serde(skip)]
    no_leisure: Vec<(u32, u32)>,
}

/// A taxi's run: to work (or school) or home.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: u32,
    /// Home from work (drop-offs), else to it (pick-ups).
    #[serde(default)]
    pub home: bool,
    /// A leisure run (a visit or an outing, `book_leisure`, `book_back`): the place (a plot) it
    /// takes its riders to (`home` false: picked up at home, put down there) or back from (`home`
    /// true: picked up there, put down at home); and whether the player sent them. None: work or
    /// school.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub directed: bool,
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
    /// Where it stands at each of its halts (`kerb_at`): as planned, else worked out as it comes
    /// into town (and again on load). Not saved.
    #[serde(skip)]
    kerbs: Vec<Kerb>,
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
    /// Where the taxi stands for them along its run (`Run::at`): housemates share one.
    #[serde(default)]
    pub halt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopState {
    /// On their way to the kerb, or not yet.
    Booked,
    /// Called to its door as it stands for them (`door_for`): on their way to it, or getting in.
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

    /// Whether it picks its riders up at the kerb (to work or school, out to a place, or back from
    /// one), and at which halt `st` is picked up (back from a place: its first, the place's).
    fn picks_up(&self) -> bool {
        !self.home || self.place.is_some()
    }

    /// Whether it's the taxi for one off on `task`: to work or school, to its place, home from it.
    fn serves(&self, task: &TaskKind) -> bool {
        match (self.place, self.home) {
            (None, _) => matches!(task, TaskKind::Work),
            (Some(p), false) => matches!(task, TaskKind::Visit { plot } if *plot == p),
            (Some(_), true) => matches!(task, TaskKind::GoHome),
        }
    }

    fn pickup_halt(&self, st: &Stop) -> u32 {
        if self.home { 0 } else { st.halt as u32 }
    }

    /// The halt where `st` gets out: home (`st.halt`), or out to a place, its last; none on to
    /// work.
    fn drop_halt(&self, st: &Stop) -> Option<u32> {
        match (self.home, self.place) {
            (true, _) => Some(st.halt as u32),
            (false, Some(_)) => Some(self.halts() - 1),
            (false, None) => None,
        }
    }

    /// How many halts it makes: one for each house (housemates' one), and the place it goes to or
    /// back from.
    fn halts(&self) -> u32 {
        let most = self
            .stops
            .iter()
            .map(|s| s.halt as u32 + 1)
            .max()
            .unwrap_or(0);
        most + u32::from(self.place.is_some() && !self.home)
    }
}

/// Resident `i`'s group for the taxi: the way out they leave by (whatever their work or school:
/// the taxi's shared; 0, then the way out's index).
fn group(w: &World, i: usize) -> Option<(u32, u32)> {
    let c = w.sims[i].commute?;
    Some((0, c.exit as u32))
}

/// Where the taxi would stand for resident `i` at home (`kerb_at`), no other taxi about.
fn stop_for(w: &World, i: usize) -> Option<Kerb> {
    let plot = w.households.get(w.sims[i].household as usize)?.plot?;
    kerb_at(&w.travel, &w.meta, home_way(w, plot)?, &[])
}

/// Whether `i` would ride a taxi to work or school starting at `start`, and if so the stop at
/// home: an open-world commute a long way on foot, no car seat to go in, no bike to ride.
fn would_ride(w: &World, i: usize, start: u64) -> Option<Kerb> {
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
    note_still(w);
    if w.tick.is_multiple_of(TICKS_PER_MINUTE) {
        plan(w);
        close_home_runs(w);
        leisure_leaving(w);
        book_backs(w);
        if w.tick.is_multiple_of(60 * TICKS_PER_MINUTE) {
            w.taxis.no_leisure.clear();
        }
    }
    seat_riders(w);
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

/// The start of resident `i`'s work (or school) a taxi would take them to: today's still to come,
/// or begun no longer than `late_minutes` ago with them not yet gone (a game begun after they'd
/// have set off: late, but sooner than on foot); else the next.
fn start_for(w: &World, i: usize) -> Option<u64> {
    let s = &w.sims[i];
    let day = crate::clock::day(w.tick);
    let gone = match &s.job {
        Some(job) => job.last_shift_day == day,
        None => s.school_day == day,
    };
    let late = (w.content.career_rules.late_minutes * TICKS_PER_MINUTE as f32) as u64;
    crate::life::start_on(&w.content, s, w.tick)
        .filter(|&start| start + late > w.tick && !gone)
        .or_else(|| crate::life::next_start(w, i))
}

/// One who'd ride (`plan`): their group, the start they're grouped by, their start, who, and when
/// they'd set off on foot.
type Want = ((u32, u32), u64, u64, usize, u64);

/// The runs to work due to be planned: everyone who'd ride, by group (their way out), those
/// starting within `taxi.window` of the first of a group (all too late on foot already, together),
/// once one of them would set off on foot within `PLAN_LEAD` (and not yet on their way).
fn plan(w: &mut World) {
    let tick = w.tick;
    let window = (w.content.taxi.window * TICKS_PER_MINUTE as f32) as u64;
    w.taxis.declined.retain(|d| d.1 + window > tick);
    w.taxis.missed.retain(|d| d.1 + window > tick);
    let mut want: Vec<Want> = Vec::new();
    for i in 0..w.sims.len() {
        let Some(start) = start_for(w, i) else {
            continue;
        };
        let s = &w.sims[i];
        if start > tick + PLAN_HORIZON
            || s.taxi.is_some()
            || s.away_until.is_some()
            || crate::life::heading_out(s) && !setting_off(w, i)
            || w.taxis.declined.contains(&(i as u32, start))
        {
            continue;
        }
        let Some(g) = group(w, i) else { continue };
        let on_foot = start.saturating_sub(s.commute.map_or(0, |c| c.ticks));
        // (Too late to be there in time on foot already: one late run for all of them going out
        // that way, whenever their work starts.)
        let key = if on_foot <= tick { 0 } else { start };
        if would_ride(w, i, start).is_some() {
            want.push((g, key, start, i, on_foot));
        }
    }
    want.sort_by_key(|&(g, key, start, i, _)| (g, key, start, i));
    let mut k = 0;
    while k < want.len() {
        let (g, first, ..) = want[k];
        let n = want[k..]
            .iter()
            .take_while(|m| m.0 == g && m.1 <= first + window)
            .count();
        let members: Vec<(usize, u64)> = want[k..k + n].iter().map(|m| (m.3, m.2)).collect();
        let soonest = want[k..k + n].iter().map(|m| m.4).min().unwrap_or(0);
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
    let mut left: Vec<(usize, u64, Kerb)> = members
        .iter()
        .filter_map(|&(i, start)| Some((i, start, would_ride(w, i, start)?)))
        .collect();
    for &(i, start) in members {
        if !left.iter().any(|m| m.0 == i) {
            w.taxis.declined.push((i as u32, start));
        }
    }
    let seats = w.content.taxi.seats;
    let pos = |w: &World, st: &Kerb| w.travel.graph.point(st.mouth);
    while !left.is_empty() {
        // (A short order: nearest the way in first, then on nearest to the last.)
        let mut chunk: Vec<(usize, u64, Kerb)> = Vec::new();
        let mut at = exit;
        while chunk.len() < seats && !left.is_empty() {
            let j = (0..left.len())
                .min_by(|&a, &b| {
                    let d = |m: &(usize, u64, Kerb)| crate::travel::dist(pos(w, &m.2), at);
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
    chunk: &[(usize, u64, Kerb)],
) -> Result<Run, Vec<usize>> {
    // (Housemates, next to each other in the order: one halt.)
    let plot_of = |i: usize| {
        w.households[w.sims[i].household as usize]
            .plot
            .unwrap_or(u32::MAX)
    };
    let mut halts: Vec<u32> = Vec::new();
    let mut halt_of = Vec::with_capacity(chunk.len());
    for m in chunk {
        let plot = plot_of(m.0);
        if halts.last() != Some(&plot) {
            halts.push(plot);
        }
        halt_of.push(halts.len() - 1);
    }
    let off_map = crate::cars::off_map_ticks(w);
    let first = chunk.iter().map(|m| m.1).min().unwrap_or(0);
    // (Clear of other taxis standing about then: `kerbs_for`.)
    let due = first.saturating_sub(off_map);
    let kerbs = kerbs_for(w, &halts, None, due).ok_or(vec![])?;
    let (route, at) = way_for(w, exit, &kerbs, exit).ok_or(vec![])?;
    let ride = route.ticks().ceil() as u64 + halts.len() as u64 * DWELL;
    let due_out = first.saturating_sub(off_map);
    // (Spare time for the way: `SLACK` and a share of it, those in its way on the roads.)
    let spare = SLACK + ride * SLACK_SHARE / 100;
    // Each rider's walk to their kerb (ticks: from home, or from wherever they are) and when the
    // taxi's there.
    let tick = w.tick;
    let lead: Vec<u64> = chunk
        .iter()
        .enumerate()
        .map(|(k, &(i, _, _))| {
            let sim = &w.sims[i];
            let walk = sim.walk_per_minute(&w.content);
            // (From their door: home by then.)
            let from = w
                .households
                .get(sim.household as usize)
                .and_then(|h| h.plot)
                .map_or(sim.pos, |p| w.plots[p as usize].arrival_point());
            let to_kerb = ((w.travel.eta(
                from,
                wait_spot(&w.travel, &kerbs[halt_of[k]]),
                crate::travel::Mode::Foot,
                walk,
            ) + crate::life::OUT_OF_THE_HOUSE)
                / crate::MINUTES_PER_TICK) as u64;
            to_kerb.max(TO_KERB_LEAST) + KERB_EARLY
        })
        .collect();
    let offset = |h: usize| at[h].ceil() as u64 + h as u64 * DWELL;
    // Out of town in time for the first of them; else, too late for that (a game just begun, a
    // rider missed by a taxi before), as soon as it can come with each of them at their kerb by
    // then: late, but out sooner than on foot.
    let mut enter = due_out.saturating_sub(ride + spare).max(tick + 1);
    for (k, &l) in lead.iter().enumerate() {
        enter = enter.max((tick + 1 + l).saturating_sub(offset(halt_of[k])));
    }
    let due_out = enter + ride + spare;
    let mut slow = Vec::new();
    let mut out = Vec::new();
    for (k, &(i, start, _)) in chunk.iter().enumerate() {
        let h = halt_of[k];
        let due = enter + offset(h);
        let sim = &w.sims[i];
        let leave = due.saturating_sub(lead[k]);
        // (Sooner than on foot: from leaving home to out of town no longer than the walk out, and
        // out of town no later than setting off on foot now would have them.)
        let ticks = sim.commute.map_or(0, |c| c.ticks);
        let walk_out = ticks.saturating_sub(off_map);
        let by_taxi = due_out.saturating_sub(leave);
        let on_foot_out = start.saturating_sub(ticks).max(tick) + walk_out;
        if by_taxi >= walk_out || due_out > on_foot_out {
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
            halt: h as u8,
        });
    }
    if !slow.is_empty() {
        return Err(slow);
    }
    Ok(Run {
        id: 0,
        home: false,
        place: None,
        directed: false,
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
        kerbs,
    })
}

/// Whether resident `i` has only just set off to work or school on foot: on their way out of
/// town walking (no car, bike or taxi), still by their door (`SETTING_OFF`). As at a game's start,
/// when those with a long walk set off at once.
fn setting_off(w: &World, i: usize) -> bool {
    let s = &w.sims[i];
    let walking = s
        .current()
        .is_none_or(|a| matches!(a.phase, Phase::Routing { .. }));
    let door = w
        .households
        .get(s.household as usize)
        .and_then(|h| h.plot)
        .map(|p| w.plots[p as usize].arrival_point());
    crate::life::heading_out(s)
        && walking
        && s.taxi.is_none()
        && s.bound_for_car.is_none()
        && s.bound_for_bike.is_none()
        && s.riding.is_none()
        && door.is_some_and(|d| crate::travel::dist(d, s.pos) < SETTING_OFF)
}

/// Books `run`'s riders on it and adds it; any just off on foot stop, to leave again for their
/// kerb in time for it (`leave_at`).
fn book(w: &mut World, mut run: Run) {
    run.id = w.taxis.next_id;
    w.taxis.next_id = w.taxis.next_id.wrapping_add(1);
    for (k, st) in run.stops.iter().enumerate() {
        let sim = &mut w.sims[st.sim as usize];
        sim.taxi = Some(Booking {
            run: run.id,
            stop: k as u8,
        });
        if sim
            .current()
            .is_some_and(|a| matches!(a.task.kind, TaskKind::Work))
        {
            sim.current = None;
        }
        if sim
            .queue
            .front()
            .is_some_and(|t| matches!(t.kind, TaskKind::Work))
        {
            sim.queue.pop_front();
        }
    }
    w.taxis.runs.push(run);
}

/// When rider `i` leaves home for their kerb, if they're booked for the next run to work
/// (`life`'s leaving): in time for the taxi as planned, or as late as it's running (held up on its
/// way, or still at the edge of town when it was due in, `run_on`).
pub(crate) fn leave_at(w: &World, i: usize) -> Option<u64> {
    booked_leave(&w.taxis, &w.cars, w.tick, w.sims[i].taxi?)
}

/// `leave_at` for booking `b` at `tick`: the planned leave, later by as much as the taxi runs late
/// (`world`'s free time before work follows it too).
pub(crate) fn booked_leave(
    taxis: &Taxis,
    cars: &[crate::cars::Car],
    tick: u64,
    b: Booking,
) -> Option<u64> {
    let run = taxis.runs.iter().find(|r| r.id == b.run && !r.home)?;
    let st = run.stops.get(b.stop as usize)?;
    if st.state != StopState::Booked {
        return None;
    }
    let late = match run.car.and_then(|c| cars.get(c as usize)) {
        Some(car) => {
            let at = run.at.get(st.halt as usize).copied().unwrap_or(0.0);
            let to_go = (at - car.progress).max(0.0).ceil() as u64;
            let halts = (st.halt as u64).saturating_sub(run.next as u64) * DWELL;
            (tick + to_go + halts).saturating_sub(st.due)
        }
        None => tick.saturating_sub(run.enter),
    };
    Some(st.leave + late)
}

/// Where `sim`, booked on a taxi to work, waits for it (`world`'s way to work), if it's still to
/// come for them.
pub(crate) fn kerb_for(
    ctx: &crate::world::Ctx,
    sim: &crate::world::Sim,
    task: &TaskKind,
) -> Option<[f32; 2]> {
    let b = sim.taxi?;
    let run = ctx
        .taxis
        .runs
        .iter()
        .find(|r| r.id == b.run && r.picks_up() && r.serves(task))?;
    let st = run.stops.get(b.stop as usize)?;
    let halt = run.pickup_halt(st);
    if st.state != StopState::Booked || run.next > halt {
        return None;
    }
    let stop = run.kerbs.get(halt as usize)?;
    Some(wait_spot(ctx.travel, stop))
}

/// The taxi `sim` is called to (`stand`), standing at their halt: the car, their seat, the door
/// they get in by and where they stand for it (`world`'s way to work: over to it, and in).
pub(crate) fn door_for(
    ctx: &crate::world::Ctx,
    sim: &crate::world::Sim,
    task: &TaskKind,
) -> Option<(u32, usize, usize, [f32; 2])> {
    let b = sim.taxi?;
    let run = ctx
        .taxis
        .runs
        .iter()
        .find(|r| r.id == b.run && r.picks_up() && r.serves(task))?;
    let st = run.stops.get(b.stop as usize)?;
    let c = run.car?;
    let car = ctx.cars.get(c as usize)?;
    if st.state != StopState::Boarding || !car.standing || run.next != run.pickup_halt(st) {
        return None;
    }
    let seat = usize::from(sim.car_seat);
    let (door, at) = car.taxi_door(&ctx.nav, seat);
    Some((c, seat, door, at))
}

/// Where a rider waits for the taxi standing at `stop`: at the back of the pavement on their
/// house's side, by where it stands (a step to its doors, or across the road to them), not in
/// their driveway's mouth (`WAIT_ASIDE` along from it at least).
fn wait_spot(travel: &crate::travel::Travel, stop: &Kerb) -> [f32; 2] {
    let g = &travel.graph;
    let road = &g.roads[stop.road as usize];
    let mouth = g.point(stop.mouth);
    let off = stop.s - stop.mouth.s;
    let s = if off.abs() >= WAIT_ASIDE {
        stop.s
    } else {
        stop.mouth.s + if off < 0.0 { -WAIT_ASIDE } else { WAIT_ASIDE }
    };
    let (c, t) = (road.at(s), road.tangent(s));
    let right = [-t[1], t[0]];
    let house = [mouth[0] - stop.axis[0], mouth[1] - stop.axis[1]];
    let k = if (house[0] - c[0]) * right[0] + (house[1] - c[1]) * right[1] >= 0.0 {
        1.0
    } else {
        -1.0
    };
    // (A road with no pavement, a country road: on its edge.)
    let back = road.width / 2.0 - 0.3;
    [c[0] + right[0] * back * k, c[1] + right[1] * back * k]
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

/// The plots of run `k`'s halts, in order (housemates' one; a leisure run's place, `Run::place`,
/// last going there, first coming back).
fn halts_of(run: &Run) -> Vec<u32> {
    let n = run.halts() as usize;
    let mut halts = vec![u32::MAX; n];
    for st in &run.stops {
        if let Some(h) = halts.get_mut(st.halt as usize) {
            *h = st.plot;
        }
    }
    if let Some(place) = run.place {
        let k = if run.home { 0 } else { n - 1 };
        halts[k] = place;
    }
    halts
}

/// Where a taxi stands for each of `halts` (their plots, in order; `kerb_at`): clear of the other
/// runs' taxis standing about then (but `skip`'s, from `due` back two hours or so: those not out of
/// town by then), and one stand for two halts as near each other as a taxi's length and a gap.
fn kerbs_for(w: &World, halts: &[u32], skip: Option<u32>, due: u64) -> Option<Vec<Kerb>> {
    let span = 2 * 60 * TICKS_PER_MINUTE;
    let from = due.saturating_sub(span);
    let taken: Vec<(u32, f32)> = w
        .taxis
        .runs
        .iter()
        .filter(|r| Some(r.id) != skip)
        .filter(|r| {
            let (a, b) = if r.home {
                let a = if r.enter == 0 { r.depart } else { r.enter };
                (a, a + span)
            } else {
                (r.enter, r.due_out)
            };
            a <= due && b >= from
        })
        .flat_map(|r| r.kerbs.iter().map(|k| (k.road, k.s)))
        .collect();
    let mut out: Vec<Kerb> = Vec::with_capacity(halts.len());
    for &plot in halts {
        let mut kerb = kerb_at(&w.travel, &w.meta, home_way(w, plot)?, &taken)?;
        if let Some(last) = out.last()
            && last.road == kerb.road
            && (last.s - kerb.s).abs() < TAXI_LEN + STAND_GAP
        {
            kerb.s = last.s;
        }
        out.push(kerb);
    }
    Some(out)
}

/// A taxi's way in at `exit`, to stand at each of `kerbs` and out at `exit` again: its route and
/// the ticks along it where it stands at each (`lay_route`), clear of places held up where it can
/// be (`jammed`). None: no way.
fn way_for(
    w: &World,
    way_in: [f32; 2],
    kerbs: &[Kerb],
    way_out: [f32; 2],
) -> Option<(crate::cars::Route, Vec<f32>)> {
    let net = Net::new(&w.travel, &w.meta, &jammed(w));
    let legs = plan_way(&w.travel, &net, way_in, kerbs, way_out)?;
    lay_route(&w.travel, &net, &legs)
}

/// Run `k`'s route and the ticks along it where it stands at each halt, and where it stands there
/// (its kerbs as planned; worked out again, after a load). None: no way.
fn route_of(w: &World, k: usize) -> Option<(crate::cars::Route, Vec<f32>, Vec<Kerb>)> {
    let run = &w.taxis.runs[k];
    let halts = halts_of(run);
    let kerbs = if run.kerbs.len() == halts.len() {
        run.kerbs.clone()
    } else {
        let due = if run.home || run.place.is_some() {
            w.tick.max(run.enter)
        } else {
            run.due_out
        };
        kerbs_for(w, &halts, Some(run.id), due)?
    };
    let (way_in, way_out) = match run.place {
        // (A leisure run: in by the way out nearest its first halt, out by the one nearest its
        // last.)
        Some(_) => (
            nearest_way_out(w, &kerbs[0])?,
            nearest_way_out(w, kerbs.last()?)?,
        ),
        None => {
            let &exit = w.exits.get(run.group.1 as usize)?;
            (exit, exit)
        }
    };
    let (route, at) = way_for(w, way_in, &kerbs, way_out)?;
    Some((route, at, kerbs))
}

/// The way out of town (`World::exits`) nearest `kerb` as the crow flies.
fn nearest_way_out(w: &World, kerb: &Kerb) -> Option<[f32; 2]> {
    let at = w.travel.graph.point(kerb.mouth);
    w.exits
        .iter()
        .copied()
        .min_by(|a, b| crate::travel::dist(*a, at).total_cmp(&crate::travel::dist(*b, at)))
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
        let Some((route, at, kerbs)) = route_of(w, k) else {
            return false;
        };
        // (No longer than its spare time, a run to work: `time_run`'s `SLACK` and share.)
        let most = if run.home {
            JAM_WAIT
        } else {
            JAM_WAIT
                .min(run.due_out.saturating_sub(run.enter) * SLACK_SHARE / (100 + SLACK_SHARE) / 2)
        };
        let jams = jammed(w);
        // (Not while any of its riders are already out for it: in it comes, round what it can.)
        let out = run.stops.iter().any(|st| {
            st.state != StopState::Done
                && w.sims.get(st.sim as usize).is_some_and(|x| {
                    x.current()
                        .is_some_and(|a| matches!(a.task.kind, TaskKind::Work))
                })
        });
        if !jams.is_empty() && !out && !run.home && through(&route, &jams) {
            if tick < run.enter + most {
                return true;
            }
            // Still held up when it's waited all it can: it doesn't come, and its riders make
            // their own way (not planned again into the same hold-up).
            for st in &run.stops {
                if let Some(start) = start_for(w, st.sim as usize) {
                    w.taxis.missed.push((st.sim, start));
                }
            }
            return false;
        }
        let Some(c) = slot(w) else {
            return false;
        };
        let progress = w.taxis.runs[k].progress;
        let car = &mut w.cars[c as usize];
        car.set_taxi_route(route, progress);
        let run = &mut w.taxis.runs[k];
        run.car = Some(c);
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
    crate::cars::drive_taxi(w, c);
    w.taxis.runs[k].progress = w.cars[c].progress;
    w.cars[c].progress < ticks
}

/// Run `k`'s taxi (car `c`) stands at its stop `next` (a halt: housemates' one): those there get
/// in (called to its door, a seat each: `door_for`), or out, each by a door of their own; one not
/// on their way to it, or not at its door within `ARRIVE_SOON` at their pace, is left.
fn stand(w: &mut World, k: usize, c: usize, next: usize) {
    let tick = w.tick;
    crate::cars::stand_taxi(w, c);
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
                // little longer than straight there.)
                let d = crate::travel::dist(sim.pos, taxi_at);
                let left = ARRIVE_SOON.saturating_sub(tick - since) as f32;
                let soon = d * WAY_ROUND
                    <= left * sim.walk_per_minute(&w.content) * crate::MINUTES_PER_TICK + 3.0;
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
                } else {
                    // Not there, nor coming: on without them, and they make their own way.
                    w.taxis.runs[k].stops[s].state = StopState::Done;
                    release(w, i);
                }
            }
            StopState::Boarding => {
                // (On their way to its door, or getting in; called away meanwhile, or held up on
                // their way to it too long: on without them.)
                // (In: their getting in done, it's under way this tick, `boarded`.)
                let going = w.sims.get(i).is_some_and(|x| {
                    let to_it = x.bound_for_car == Some(c as u32);
                    let doing = x.current().map(|a| (&a.task.kind, &a.phase));
                    to_it && matches!(doing, Some((_, Phase::Boarding { .. })))
                        || to_it
                            && (doing.is_some_and(|d| task_of(d.0))
                                || doing.is_none()
                                    && x.queue.front().is_some_and(|t| task_of(&t.kind)))
                            && tick <= since + TO_DOOR_MOST
                        || matches!(
                            x.transition,
                            Some(crate::life::Transition::Board { car }) if car == c as u32
                        )
                });
                if going {
                    busy = true;
                } else {
                    w.taxis.runs[k].stops[s].state = StopState::Done;
                    release(w, i);
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
                .get((o.first & !(1 << 63)) as usize)
                .is_some_and(|x| x.waits_for == Some(key))
                || w.traffic.waits_for(o.first).contains(&key);
            !waits
                && crate::cars::too_near(
                    (me.0, me.1, car.len, car.wid + 2.0 * PASS_BY),
                    (o.at, o.yaw, o.len, o.wid),
                )
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
/// for it to wait till they're past, and how long it waits so at most since it stood (ticks).
const PASS_BY: f32 = 0.3;
const PASS_BY_MOST: u64 = 2 * ARRIVE_SOON;

/// Resident `i`, booked on a taxi, isn't riding it: unbooked, and on their way to work or school
/// their own way (from the kerb, or wherever they've got to); planned once more for the same start
/// (`Taxis::missed`), late as it may be, before they're left to it.
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
            run.home || run.place.is_some()
        }
        None => false,
    };
    // (Planned again once for the same start, a late run of their own if it must be; then their
    // own way it is.)
    if !home && let Some(start) = start_for(w, i) {
        let me = (i as u32, start);
        if w.taxis.missed.contains(&me) {
            w.taxis.declined.push(me);
        } else {
            w.taxis.missed.push(me);
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
        w.sims[i].bound_for_car = None;
        release(w, i);
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
                let list = if run.due_out + off_map > start {
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
        if w.sims
            .get(i)
            .is_some_and(|s| s.taxi.is_some_and(|b| b.run == run.id))
        {
            release(w, i);
        }
    }
}

/// Resident `i` is done at work (or school): having come by taxi today, or on foot with a long
/// walk home (`homeward`), they wait there for one home (true), with the others of their group
/// done about when they are (`Run::depart`).
pub(crate) fn off_work(w: &mut World, i: usize) -> bool {
    let me = i as u32;
    let g = match w.taxis.rode.iter().position(|r| r.0 == me) {
        Some(p) => w.taxis.rode.remove(p).1,
        // (Come on foot: home by one all the same, a long walk home.)
        None => match homeward(w, i) {
            Some(g) => g,
            None => return false,
        },
    };
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
                depart: tick + window,
                stops: Vec::new(),
                next: 0,
                progress: 0.0,
                since: 0,
                car: None,
                at: Vec::new(),
                kerbs: Vec::new(),
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
    if !w.travel.open
        || c.home != home
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
        let Some((route, at, kerbs)) = route_of(w, k) else {
            // (No way: they make their own way home from the way out.)
            w.taxis.runs[k].enter = tick;
            w.taxis.runs[k].at = Vec::new();
            continue;
        };
        w.taxis.runs[k].kerbs = kerbs;
        let enter = tick + off_map;
        let stops = w.taxis.runs[k].stops.clone();
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
                (Some((route, at, kerbs)), Some(c)) => {
                    w.cars[c as usize].set_taxi_route(route, run.progress);
                    w.taxis.runs[k].car = Some(c);
                    w.taxis.runs[k].at = at;
                    w.taxis.runs[k].kerbs = kerbs;
                }
                _ => {
                    finish(w, k);
                    w.taxis.runs.remove(k);
                    continue;
                }
            }
        } else if (!run.home || run.enter != 0)
            && let Some((_, at, kerbs)) = route_of(w, k)
        {
            w.taxis.runs[k].at = at;
            w.taxis.runs[k].kerbs = kerbs;
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

/// How far either side of a taxi's body the road it stands on is kept straight and clear of a
/// junction's box (m), and how far it may turn over that stretch at most (rad: about 4°, the
/// body parallel to its lane all along).
const STAND_ROOM: f32 = 1.0;
const STAND_STRAIGHT: f32 = 0.07;
/// How far along the road from a rider's driveway mouth the taxi may stand for them at most (m),
/// tried this often (m) nearest first.
const STAND_REACH: f32 = 30.0;
const STAND_STEP: f32 = 0.5;
/// Room between two taxis standing on one stretch (m, between their bodies).
const STAND_GAP: f32 = 3.0;
/// How far from a way out of town along its road a taxi doesn't stand (m).
const STAND_OUT: f32 = 20.0;
/// How far the claim of a taxi standing at the kerb reaches before and after its middle, past half
/// its length (m): those behind it, or turning out of a driveway by it, wait for it.
const KERB_CLAIM: f32 = 2.0;
/// Its own lane's kerb a standing taxi keeps this far from (m: its corners too, on a lane that
/// turns a little over its length, `STAND_STRAIGHT`).
const KERB_KEEP: f32 = 0.15;
/// Over how much of the way before it the taxi eases to where it stands (m).
const STAND_EASE: f32 = 8.0;
/// Turning round in a dead end's turning circle (`u_turn`): how far its kerb reaches past the
/// lane's (m, `roads.ts` `TURNING`), how far over it a front corner may swing (m), the swerve's
/// radius into it and out (m), and the speed (m/s).
const TURNING: f32 = 2.4;
const TURN_OVER: f32 = 0.3;
const SWERVE: f32 = 2.0;
const TURN_SPEED: f32 = crate::cars::JUNCTION_SPEED;
/// What turning round costs the way it's weighed (s), on top of the drive, and standing for a
/// rider on the far side of the road from their house (s: they cross it).
const TURN_COST: f32 = 6.0;
/// What turning round in a side street costs, the way it's weighed (s: slowly in and backing out).
const SIDE_COST: f32 = 12.0;
const FAR_SIDE: f32 = 8.0;
/// A stretch through a place cars are held up now (`jammed`) as a run's way is weighed (s): driven
/// only with no other way near as quick.
const JAM_COST: f32 = 600.0;

/// Where a taxi stands for a halt (`kerb_at`): its middle `s` m along road `road` (the network's),
/// in its lane and parallel to it, the way it comes there deciding which side; and the rider's
/// driveway, whose mouth they wait by (`wait_spot`) and on whose side the house is.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Kerb {
    pub road: u32,
    pub s: f32,
    pub mouth: crate::travel::RoadPoint,
    /// The way the driveway runs from the house to the road (a unit vector).
    pub axis: [f32; 2],
}

/// The way a driveway runs from its inner end to the street (`cars::Drive::way`).
fn drive_axis(d: &crate::cars::Drive) -> Option<[f32; 2]> {
    let (x, z) = (d.front_x, d.front);
    let l = x.hypot(z);
    (l.is_finite() && l > 0.5).then(|| [x / l, z / l])
}

/// Where the house on `plot` meets the road its riders wait by and the way to it from the house
/// (a unit vector): its driveway's mouth, else (no driveway) the road nearest its front door,
/// where its front path comes out.
fn home_way(w: &World, plot: u32) -> Option<(crate::travel::RoadPoint, [f32; 2])> {
    let g = &w.travel.graph;
    if let Some(d) = w.drives.iter().find(|d| d.plot == plot)
        && let Some(axis) = drive_axis(d)
        && let Some(mouth) = d
            .road
            .zip(d.s.filter(|s| s.is_finite()))
            .and_then(|(r, s)| g.at_source(r, s))
    {
        return Some((mouth, axis));
    }
    let door = w.plots.get(plot as usize)?.arrival_point();
    let mouth = *g
        .near(door, crate::cars::ROAD_REACH, 1, crate::travel::Mode::Car)
        .first()?;
    let p = g.point(mouth);
    let l = crate::travel::dist(p, door).max(1e-3);
    Some((mouth, [(p[0] - door[0]) / l, (p[1] - door[1]) / l]))
}

/// Whether end `end` (0: its start, 1: its end) of the network's road `road` is a dead end with a
/// turning circle (the town file's `ends`, `dead`): a street's or a lane's.
fn dead_end(
    travel: &crate::travel::Travel,
    meta: &serde_json::Value,
    road: u32,
    end: usize,
) -> bool {
    use crate::travel::RoadKind;
    let r = &travel.graph.roads[road as usize];
    matches!(r.kind, RoadKind::Street | RoadKind::Lane)
        && meta["roads"][r.source as usize]["ends"][end].as_str() == Some("dead")
}

/// Whether end `end` of the network's road `road` is a way out of town (`ends`, `out`).
fn out_end(
    meta: &serde_json::Value,
    travel: &crate::travel::Travel,
    road: u32,
    end: usize,
) -> bool {
    let r = &travel.graph.roads[road as usize];
    meta["roads"][r.source as usize]["ends"][end].as_str() == Some("out")
}

/// Where the taxi stands for a house whose way to the road meets it at `mouth`, running there
/// along `axis` (`home_way`): by it, nearest first along the road, where the road runs straight for the taxi's body and a little
/// more (`STAND_ROOM`, `STAND_STRAIGHT`), clear of junctions' boxes, of a dead end's turning
/// circle, of the way out of town, and of another taxi standing (`taken`: (road, s), with
/// `STAND_GAP` between them). None: not a road for cars, or no such stretch within
/// `STAND_REACH`.
pub(crate) fn kerb_at(
    travel: &crate::travel::Travel,
    meta: &serde_json::Value,
    (mouth, axis): (crate::travel::RoadPoint, [f32; 2]),
    taken: &[(u32, f32)],
) -> Option<Kerb> {
    let g = &travel.graph;
    let road = &g.roads[mouth.road as usize];
    if !road.drivable() {
        return None;
    }
    let half = TAXI_LEN / 2.0 + STAND_ROOM;
    let len = road.len();
    // (From each end: past a dead end's turning circle and the swerve into it, or the way out.)
    let keep = |end: usize| {
        if dead_end(travel, meta, mouth.road, end) {
            u_turn_reach(road.width, road.kind) + TAXI_LEN
        } else if out_end(meta, travel, mouth.road, end) {
            STAND_OUT
        } else {
            0.0
        }
    };
    let (lo, hi) = (keep(0) + half, len - keep(1) - half);
    let steps = (STAND_REACH / STAND_STEP) as i32;
    (0..=2 * steps)
        .map(|k| mouth.s + ((k + 1) / 2) as f32 * STAND_STEP * if k % 2 == 1 { 1.0 } else { -1.0 })
        .filter(|&s| s >= lo && s <= hi)
        .find(|&s| {
            let (a, b) = (road.at(s - half), road.at(s + half));
            let chord = (b[1] - a[1]).atan2(b[0] - a[0]);
            let n = (2.0 * half / STAND_STEP).ceil() as usize;
            let straight = (0..=n).all(|i| {
                let t = road.tangent(s - half + i as f32 * 2.0 * half / n as f32);
                crate::world::wrap_angle(t[1].atan2(t[0]) - chord).abs() <= STAND_STRAIGHT
            });
            let clear = (0..=n).all(|i| {
                let p = road.at(s - half + i as f32 * 2.0 * half / n as f32);
                g.boxes
                    .iter()
                    .all(|&(m, r)| crate::travel::dist(m, p) >= r + STAND_ROOM + TAXI_WID)
            });
            let free = taken
                .iter()
                .all(|&(r, ts)| r != mouth.road || (ts - s).abs() >= TAXI_LEN + STAND_GAP);
            straight && clear && free
        })
        .map(|s| Kerb {
            road: mouth.road,
            s,
            mouth,
            axis,
        })
}

/// The radii of a taxi's turn round in a dead end's turning circle on a road `width` m wide of
/// `kind` (`u_turn`): the loop round its middle, the lane's offset, and how far before the end
/// the swerve into it begins (m).
fn u_turn_radii(width: f32, kind: crate::travel::RoadKind) -> (f32, f32, f32) {
    let kerb = crate::cars::kerb_of(width, kind) + TURNING;
    // (A front corner swinging over the kerb by `TURN_OVER` at most.)
    let reach = (kerb + TURN_OVER).powi(2) - (TAXI_LEN / 2.0).powi(2);
    let r = (reach.max(0.0).sqrt() - TAXI_WID / 2.0).max(2.5);
    let lane = crate::cars::lane_on(width, kind);
    let before = ((r + SWERVE).powi(2) - (lane + SWERVE).powi(2))
        .max(0.0)
        .sqrt();
    (r, lane, before)
}

/// How far before a dead end's end a taxi turning round there begins to (m).
fn u_turn_reach(width: f32, kind: crate::travel::RoadKind) -> f32 {
    u_turn_radii(width, kind).2
}

/// The taxis' road network as they drive it (`way_between`): the stretches of road at each node
/// a car may take, those of the nodes a car can turn round at (a dead end's turning circle with
/// its stretch long enough to swerve into it), and the stretches through a place held up now
/// (`jammed`).
struct Net {
    adj: Vec<Vec<u32>>,
    turn: Vec<bool>,
    jam: Vec<bool>,
}

impl Net {
    fn new(
        travel: &crate::travel::Travel,
        meta: &serde_json::Value,
        jammed: &[([f32; 2], f32)],
    ) -> Net {
        let g = &travel.graph;
        let mut adj = vec![Vec::new(); g.nodes.len()];
        for (k, e) in g.edges.iter().enumerate() {
            if g.roads[e.road as usize].drivable() {
                adj[e.a as usize].push(k as u32);
                adj[e.b as usize].push(k as u32);
            }
        }
        let turn = (0..g.nodes.len())
            .map(|n| {
                let [e] = adj[n][..] else { return false };
                let edge = &g.edges[e as usize];
                let road = &g.roads[edge.road as usize];
                let end = if n as u32 == edge.b && edge.s1 >= road.len() - 0.01 {
                    1
                } else if n as u32 == edge.a && edge.s0 <= 0.01 {
                    0
                } else {
                    return false;
                };
                dead_end(travel, meta, edge.road, end)
                    && edge.len() > u_turn_reach(road.width, road.kind) + 1.0
            })
            .collect();
        // (A stretch through one, about: sampled every few metres.)
        let jam = g
            .edges
            .iter()
            .map(|e| {
                if jammed.is_empty() {
                    return false;
                }
                let road = &g.roads[e.road as usize];
                let n = (e.len() / 4.0).ceil().max(1.0) as usize;
                (0..=n).any(|k| {
                    let p = road.at(e.s0 + e.len() * k as f32 / n as f32);
                    jammed
                        .iter()
                        .any(|&(m, r)| crate::travel::dist(p, m) <= r + JAM_NEAR)
                })
            })
            .collect();
        Net { adj, turn, jam }
    }
}

/// A stretch of road driven: the network's road, from and to how far along it (m).
type Piece = (u32, f32, f32);

/// The ways for a taxi from `from`, going `dir` along its road (+1: as `s` rises; both if
/// `None`), to `to` along the roads, as a car drives them, turning round only in a dead end's
/// turning circle (`Net::turn`): by the way it comes to `to` (+1, then -1), the cost (s) and the
/// stretches driven, one after another. None: no such way.
#[allow(clippy::type_complexity)]
fn way_between(
    travel: &crate::travel::Travel,
    net: &Net,
    from: crate::travel::RoadPoint,
    dir: Option<f32>,
    to: crate::travel::RoadPoint,
) -> [Option<(f32, Vec<Piece>)>; 2] {
    use std::cmp::Reverse;
    let g = &travel.graph;
    let speed = |road: u32| {
        if g.roads[road as usize].kind == crate::travel::RoadKind::Country {
            travel.country_speed
        } else {
            crate::travel::TOWN_SPEED
        }
    };
    let slot = |d: f32| usize::from(d < 0.0);
    let mut best: [Option<(f32, Vec<Piece>)>; 2] = [None, None];
    let fe = g.edges[from.edge as usize];
    let te = g.edges[to.edge as usize];
    let dirs: Vec<f32> = dir.map_or(vec![1.0, -1.0], |d| vec![d]);
    // (Straight along the same stretch, ahead.)
    if from.edge == to.edge {
        for &d in &dirs {
            if (to.s - from.s) * d >= -0.01 {
                best[slot(d)] = Some((
                    (to.s - from.s).abs() / speed(from.road),
                    vec![(from.road, from.s, to.s)],
                ));
            }
        }
    }
    // States: the end of a stretch (edge × 2 + 1 at its `b` end) come to along it.
    let n = g.edges.len() * 2;
    let mut cost = vec![f32::INFINITY; n];
    let mut came: Vec<Option<usize>> = vec![None; n];
    let mut heap = std::collections::BinaryHeap::new();
    let key = |c: f32| (c * 1000.0) as u64;
    for &d in &dirs {
        let st = from.edge as usize * 2 + usize::from(d > 0.0);
        let c = if d > 0.0 {
            fe.s1 - from.s
        } else {
            from.s - fe.s0
        }
        .max(0.0)
            / speed(fe.road);
        if c < cost[st] {
            cost[st] = c;
            heap.push(Reverse((key(c), st)));
        }
    }
    let node_of = |st: usize| {
        let e = &g.edges[st / 2];
        if st % 2 == 1 { e.b } else { e.a }
    };
    let mut ends: [Option<(f32, usize)>; 2] = [None, None];
    while let Some(Reverse((k, st))) = heap.pop() {
        let c = cost[st];
        if key(c) < k {
            continue;
        }
        let worst = (0..2)
            .map(|i| match (&best[i], ends[i]) {
                (Some(b), _) => b.0,
                (None, Some(e)) => e.0,
                _ => f32::INFINITY,
            })
            .fold(0.0f32, f32::max);
        if c >= worst {
            break;
        }
        let u = node_of(st) as usize;
        let came_by = (st / 2) as u32;
        for &e in &net.adj[u] {
            let pass = if net.jam[e as usize] { JAM_COST } else { 0.0 };
            let back = e == came_by;
            if back && !net.turn[u] && side_turn(g, net, u as u32, came_by).is_none() {
                continue;
            }
            let edge = &g.edges[e as usize];
            let extra = pass
                + match (back, net.turn[u]) {
                    (false, _) => 0.0,
                    (true, true) => TURN_COST,
                    (true, false) => SIDE_COST,
                };
            // Onto the goal's stretch: there, coming the way it runs from here.
            if e == to.edge {
                let (s_at, d) = if edge.a as usize == u {
                    (edge.s0, 1.0)
                } else {
                    (edge.s1, -1.0)
                };
                let total = c + extra + (to.s - s_at).abs() / speed(edge.road);
                let i = slot(d);
                if best[i].as_ref().is_none_or(|b| total < b.0)
                    && ends[i].is_none_or(|x| total < x.0)
                {
                    ends[i] = Some((total, st));
                }
            }
            let next = e as usize * 2 + usize::from(edge.a as usize == u);
            let nc = c + extra + edge.len() / speed(edge.road);
            if nc < cost[next] {
                cost[next] = nc;
                came[next] = Some(st);
                heap.push(Reverse((key(nc), next)));
            }
        }
    }
    // (Back from where it came onto the goal's stretch to the start.)
    for i in 0..2 {
        let Some((total, last)) = ends[i] else {
            continue;
        };
        if best[i].as_ref().is_some_and(|b| b.0 <= total) {
            continue;
        }
        let mut chain = vec![last];
        while let Some(p) = came[*chain.last().expect("one")] {
            chain.push(p);
        }
        chain.reverse();
        let mut pieces: Vec<Piece> = Vec::with_capacity(chain.len() + 2);
        let first = chain[0];
        let s_end = if first % 2 == 1 { fe.s1 } else { fe.s0 };
        pieces.push((from.road, from.s, s_end));
        for &st in &chain[1..] {
            let e = &g.edges[st / 2];
            pieces.push(if st % 2 == 1 {
                (e.road, e.s0, e.s1)
            } else {
                (e.road, e.s1, e.s0)
            });
        }
        let u = node_of(last);
        let s_at = if te.a == u { te.s0 } else { te.s1 };
        pieces.push((to.road, s_at, to.s));
        pieces.retain(|p| (p.2 - p.1).abs() > 1e-4);
        best[i] = Some((total, pieces));
    }
    best
}

/// The network's place for `kerb` (its road and `s`).
fn kerb_point(g: &crate::travel::Graph, kerb: &Kerb) -> Option<crate::travel::RoadPoint> {
    g.at_source(g.roads[kerb.road as usize].source, kerb.s)
        .filter(|p| p.road == kerb.road)
}

/// Whether a taxi coming to `kerb` going `d` (+1: as `s` rises) stands on the far side of the
/// road from the house.
fn far_side(g: &crate::travel::Graph, kerb: &Kerb, d: f32) -> bool {
    let road = &g.roads[kerb.road as usize];
    let (c, t) = (road.at(kerb.s), road.tangent(kerb.s));
    let right = [-t[1] * d, t[0] * d];
    let m = g.point(kerb.mouth);
    let house = [m[0] - kerb.axis[0], m[1] - kerb.axis[1]];
    (house[0] - c[0]) * right[0] + (house[1] - c[1]) * right[1] < 0.0
}

/// A taxi's way round town (`route`): from `way_in` to each of `kerbs` in turn and out at
/// `way_out`, as a car drives (`way_between`), the way it comes to each kerb chosen for the
/// quickest way on (`FAR_SIDE` on top for standing across the road from the house). The stretches
/// driven, in legs: one up to each kerb, then out. None: no way.
fn plan_way(
    travel: &crate::travel::Travel,
    net: &Net,
    way_in: [f32; 2],
    kerbs: &[Kerb],
    way_out: [f32; 2],
) -> Option<Vec<Vec<Piece>>> {
    use crate::travel::Mode;
    let g = &travel.graph;
    let entry = *g
        .near(way_in, crate::cars::ROAD_REACH, 1, Mode::Car)
        .first()?;
    let exit = *g
        .near(way_out, crate::cars::ROAD_REACH, 1, Mode::Car)
        .first()?;
    let points: Vec<crate::travel::RoadPoint> = kerbs
        .iter()
        .map(|k| kerb_point(g, k))
        .collect::<Option<_>>()?;
    // Per kerb, by the way it comes there (+1, -1): the cost so far, the way it came to the one
    // before (its slot), and the leg there.
    type Best = Option<(f32, usize, Vec<Piece>)>;
    let mut table: Vec<[Best; 2]> = Vec::with_capacity(kerbs.len());
    for (k, &to) in points.iter().enumerate() {
        let mut here: [Best; 2] = [None, None];
        let starts: Vec<(f32, usize, crate::travel::RoadPoint, Option<f32>)> = if k == 0 {
            vec![(0.0, 0, entry, None)]
        } else {
            (0..2)
                .filter_map(|i| {
                    let c = table[k - 1][i].as_ref()?.0;
                    Some((c, i, points[k - 1], Some(if i == 0 { 1.0 } else { -1.0 })))
                })
                .collect()
        };
        for (c0, i0, from, dir) in starts {
            let ways = way_between(travel, net, from, dir, to);
            for (i, way) in ways.into_iter().enumerate() {
                let Some((c, pieces)) = way else { continue };
                let d = if i == 0 { 1.0 } else { -1.0 };
                let total = c0
                    + c
                    + if far_side(g, &kerbs[k], d) {
                        FAR_SIDE
                    } else {
                        0.0
                    };
                if here[i].as_ref().is_none_or(|b| total < b.0) {
                    here[i] = Some((total, i0, pieces));
                }
            }
        }
        if here.iter().all(Option::is_none) {
            return None;
        }
        table.push(here);
    }
    // Out: from the last kerb (or straight through with none).
    let mut out: Option<(f32, usize, Vec<Piece>)> = None;
    let starts: Vec<(f32, usize, crate::travel::RoadPoint, Option<f32>)> = match table.last() {
        None => vec![(0.0, 0, entry, None)],
        Some(last) => (0..2)
            .filter_map(|i| {
                let c = last[i].as_ref()?.0;
                Some((
                    c,
                    i,
                    *points.last().expect("some"),
                    Some(if i == 0 { 1.0 } else { -1.0 }),
                ))
            })
            .collect(),
    };
    for (c0, i0, from, dir) in starts {
        for (c, pieces) in way_between(travel, net, from, dir, exit)
            .into_iter()
            .flatten()
        {
            if out.as_ref().is_none_or(|b| c0 + c < b.0) {
                out = Some((c0 + c, i0, pieces));
            }
        }
    }
    let (_, mut i, pieces) = out?;
    let mut legs = vec![pieces];
    for k in (0..table.len()).rev() {
        let (_, prev, pieces) = table[k][i].clone()?;
        legs.push(pieces);
        i = prev;
    }
    legs.reverse();
    Some(legs)
}

/// A taxi's route (`len` by `wid`) along `legs` (`plan_way`): in from off the map, standing at the
/// kerb at the end of each leg but the last (in its lane, eased in to clear the kerb by
/// `KERB_KEEP` where its lane comes nearer, slowing to a stand; a claim of its stretch held as it
/// stands, `KERB_CLAIM`), turning round where it goes back the way it came (in a dead end's
/// turning circle, `u_turn`; else nose first into a side street at a junction and backing out,
/// `side_turn`; each a claim of its own), and off the map at the end. Its claims worked out
/// (`Route::claim`); with the ticks along it where it stands at each kerb.
fn lay_route(
    travel: &crate::travel::Travel,
    net: &Net,
    legs: &[Vec<Piece>],
) -> Option<(crate::cars::Route, Vec<f32>)> {
    use crate::cars::{Legs, kerb_of, lane_on, pull_room, top_speed};
    use crate::travel::{RoadKind, dist};
    let g = &travel.graph;
    let junctions: Vec<[f32; 2]> = g.junctions().collect();
    let mut out = Legs::default();
    let mut at_stops = Vec::new();
    // (A stand's claim: from the point where it stands, `KERB_CLAIM` past its body either way.)
    let mut stood: Option<usize> = None;
    // (Turned round into a side street: its bay, the way it goes on, where its turn out of it is
    // along the road and how much it takes, and where the turn began, for its claim.)
    let mut leaving: Option<(crate::cars::Bay, f32, f32, f32, usize)> = None;
    let last_leg = legs.len() - 1;
    for (k, leg) in legs.iter().enumerate() {
        // Its runs: split where it turns round (a stretch back along the one before).
        let mut runs: Vec<Vec<Piece>> = vec![Vec::new()];
        for &p in leg {
            let turned = runs.last().and_then(|r| r.last()).is_some_and(|q| {
                q.0 == p.0 && (q.2 - q.1) * (p.2 - p.1) < 0.0 && (q.2 - p.1).abs() < 0.01
            });
            if turned {
                runs.push(Vec::new());
            }
            runs.last_mut().expect("one").push(p);
        }
        let n_runs = runs.len();
        // (How each run but the last ends: turning round at a dead end, else in a side street.)
        let rounds: Vec<Option<(u32, Option<SideTurn>)>> = (0..n_runs)
            .map(|j| {
                let q = *runs[j].last()?;
                (j + 1 < n_runs).then(|| {
                    let (e, u) = edge_end(g, q)?;
                    Some((
                        u,
                        (!net.turn[u as usize])
                            .then(|| side_turn(g, net, u, e))
                            .flatten(),
                    ))
                })?
            })
            .collect();
        let firsts: Vec<Option<Piece>> = runs.iter().map(|r| r.first().copied()).collect();
        for (j, run) in runs.iter_mut().enumerate() {
            // (Turning round at a dead end at its end, or after one: the swerve's way off the
            // stretch.)
            let dead = |j: usize| {
                rounds
                    .get(j)
                    .copied()
                    .flatten()
                    .is_some_and(|r| r.1.is_none())
            };
            if j + 1 < n_runs && dead(j) {
                let last = run.last_mut()?;
                let road = &g.roads[last.0 as usize];
                let before = u_turn_reach(road.width, road.kind);
                let d = (last.2 - last.1).signum();
                last.2 -= d * before;
            }
            if j > 0 && dead(j - 1) {
                let first = run.first_mut()?;
                let road = &g.roads[first.0 as usize];
                let before = u_turn_reach(road.width, road.kind);
                let d = (first.2 - first.1).signum();
                first.1 += d * before;
            }
            let mut centre: Vec<[f32; 2]> = Vec::new();
            let mut along: Vec<(f32, RoadKind)> = Vec::new();
            for &(r, s0, s1) in run.iter() {
                let road = &g.roads[r as usize];
                for p in road.slice(s0, s1) {
                    if centre.last().is_some_and(|q| dist(*q, p) < 0.05) {
                        continue;
                    }
                    centre.push(p);
                    along.push((road.width, road.kind));
                }
            }
            if centre.len() < 2 {
                // (A stand right where the last one was: it stands there again.)
                continue;
            }
            let mut offs: Vec<f32> = along.iter().map(|&(w, kind)| lane_on(w, kind)).collect();
            let first_run = k == 0 && j == 0;
            // (Eased to where it stands, clear of its kerb, and out again after.)
            let ease = |offs: &mut [f32], order: &mut dyn Iterator<Item = usize>| {
                let mut gone = 0.0;
                let mut was: Option<usize> = None;
                for i in order {
                    if let Some(q) = was {
                        gone += dist(centre[q], centre[i]);
                    }
                    was = Some(i);
                    if gone > STAND_EASE {
                        break;
                    }
                    let (w, kind) = along[i];
                    let lane = lane_on(w, kind);
                    let off = lane.min(kerb_of(w, kind) - TAXI_WID / 2.0 - KERB_KEEP);
                    offs[i] = off + (lane - off) * (gone / STAND_EASE).clamp(0.0, 1.0);
                }
            };
            if j + 1 == n_runs && k < last_leg {
                ease(&mut offs, &mut (0..centre.len()).rev());
            }
            if j == 0 && k > 0 {
                ease(&mut offs, &mut (0..centre.len()));
            }
            if first_run {
                // (In from off the map.)
                centre.insert(0, crate::cars::extend_past(centre[1], centre[0]));
                offs.insert(0, offs[0]);
                along.insert(0, along[0]);
            }
            if k == last_leg && j + 1 == n_runs {
                let n = centre.len();
                centre.push(crate::cars::extend_past(centre[n - 2], centre[n - 1]));
                offs.push(offs[n - 1]);
                along.push(along[n - 1]);
            }
            let full = crate::cars::offset_right_by(&centre, &offs);
            let mut lane = crate::travel::thin_with(
                &full,
                crate::cars::ROUTE_STEP,
                crate::cars::ROUTE_TOLERANCE,
            );
            // (On from a stand: from a point just ahead on the straight, facing along it as it
            // stands; not its own first, where it stands again but for the lane's offset worked
            // out along this way rather than the last, a hair's breadth across.)
            let from_stand = j == 0 && k > 0 && out.len() > 0;
            if from_stand && full.len() > 2 {
                if lane.get(1) != Some(&full[1]) {
                    lane.insert(1, full[1]);
                }
                lane.remove(0);
            }
            let nearest = |p: [f32; 2]| {
                (0..full.len())
                    .min_by(|&a, &b| dist(full[a], p).total_cmp(&dist(full[b], p)))
                    .unwrap_or(0)
            };
            let way = |i: usize| {
                let (a, b) = (lane[i.saturating_sub(1)], lane[(i + 1).min(lane.len() - 1)]);
                let l = dist(a, b).max(1e-6);
                [(b[0] - a[0]) / l, (b[1] - a[1]) / l]
            };
            let along_bay = |bay: &crate::cars::Bay, i: usize, d: f32| {
                let w = way(i);
                (w[0] * bay.t[0] + w[1] * bay.t[1]) * d > 0.7
            };
            // (Into a side street at its end: it leaves the lane short of its turn in.)
            let into = rounds[j].and_then(|r| r.1).and_then(|st| {
                let bay = st.bay(g)?;
                let n = lane.len();
                let (a, b) = (lane[n.saturating_sub(2)], lane[n - 1]);
                let go = (b[0] - a[0]) * bay.t[0] + (b[1] - a[1]) * bay.t[1];
                let d = if go >= 0.0 { -1.0 } else { 1.0 };
                let (turn, at) = bay.entry(d);
                Some((bay, d, turn, at))
            });
            let mut on = false;
            for (i, &p) in lane.iter().enumerate() {
                if !on
                    && let Some((bay, d, at, taken, _)) = &leaving
                    && bay.near(p, *at, *taken, *d)
                    && along_bay(bay, i, *d)
                {
                    continue;
                }
                if let Some((bay, d, turn, at)) = &into
                    && !(from_stand && i == 0)
                    && bay.near(p, *at, turn.taken, *d)
                    && along_bay(bay, i, -*d)
                {
                    break;
                }
                if !on && let Some((.., from)) = leaving.take() {
                    // (Back in the lane out of the side street: its claim from where it turned.)
                    let last = out.last().unwrap_or(p);
                    out.line(last, p, false, crate::cars::JUNCTION_SPEED);
                    out.mouths.push((from, Some(out.len().saturating_sub(1))));
                }
                on = true;
                let i = nearest(p);
                let (w, kind) = along[i];
                out.push_lane(
                    p,
                    top_speed(p, kind, travel, &junctions),
                    pull_room(w, kind, TAXI_WID),
                    offs[i],
                );
            }
            // (The claim of the stand before, closed once there's way on to close it at.)
            if let Some(s) = stood.take() {
                let end = reach_along(&out, s, TAXI_LEN / 2.0 + KERB_CLAIM, true);
                let begin = reach_along(&out, s, TAXI_LEN / 2.0 + KERB_CLAIM, false);
                out.mouths.push((begin, Some(end)));
            }
            if j + 1 < n_runs {
                let from = out.len().saturating_sub(1);
                match into {
                    Some((bay, _, turn, _)) => {
                        // Nose first into the side street, and back out the way it came.
                        bay.enter(&mut out, &turn);
                        let next = firsts.get(j + 1).copied().flatten()?;
                        let road = &g.roads[next.0 as usize];
                        let (a, b) = (
                            road.at(next.1),
                            road.at(next.1 + (next.2 - next.1).signum() * 2.0),
                        );
                        let ahead = (b[0] - a[0]) * bay.t[0] + (b[1] - a[1]) * bay.t[1];
                        let d = if ahead >= 0.0 { 1.0 } else { -1.0 };
                        let (at, taken) = bay.leave(&mut out, d);
                        leaving = Some((bay, d, at, taken, from));
                    }
                    None => {
                        // Round in the turning circle, and on back the way it came.
                        let &(r, _, s1) = run.last().expect("pieces");
                        let road = &g.roads[r as usize];
                        let end_s = if s1 > road.len() / 2.0 {
                            road.len()
                        } else {
                            0.0
                        };
                        let t = road.tangent(end_s);
                        let t = if end_s > 0.0 { t } else { [-t[0], -t[1]] };
                        u_turn(&mut out, road.at(end_s), t, road.width, road.kind);
                        let to = out.len() - 1;
                        out.mouths.push((from, Some(to)));
                    }
                }
            }
        }
        if k < last_leg {
            // At the kerb: it slows to a stand at the lane's end there.
            if let Some(p) = out.pts.last_mut() {
                p.2 = 0.0;
            }
            let s = out.len().checked_sub(1)?;
            at_stops.push(s);
            stood = Some(s);
        }
    }
    if out.len() < 2 || leaving.is_some() {
        return None;
    }
    let mut route = out.timed(false, false);
    let ticks = at_stops.iter().map(|&i| route.points[i].2).collect();
    route.claim(TAXI_LEN, TAXI_WID, Some(travel));
    Some((route, ticks))
}

/// The stretch of road (`Graph::edges` index) piece `q` ends on, and the node it ends at.
fn edge_end(g: &crate::travel::Graph, q: Piece) -> Option<(u32, u32)> {
    let (r, s0, s1) = q;
    let d = (s1 - s0).signum();
    let mid = s1 - d * 0.25;
    g.edges.iter().enumerate().find_map(|(k, e)| {
        (e.road == r && e.s0 <= mid && mid <= e.s1).then(|| {
            let u = if d > 0.0 { e.b } else { e.a };
            (k as u32, u)
        })
    })
}

/// Turning round at a junction by nosing into a side street (`side_turn`): its bay's place there
/// (as a driveway's: `cars::Bay`), along the road come by.
#[derive(Debug, Clone, Copy)]
struct SideTurn {
    spot: [f32; 2],
    yaw: f32,
    axis: [f32; 2],
    road: u32,
    s: f32,
}

impl SideTurn {
    fn bay(&self, g: &crate::travel::Graph) -> Option<crate::cars::Bay> {
        let road = &g.roads[self.road as usize];
        crate::cars::Bay::new(
            self.spot,
            self.yaw,
            Some(self.axis),
            road.at(self.s),
            road,
            self.s,
        )
    }
}

/// How far into a side street a taxi turning round in it noses in, past the road come by's edge
/// and half its length (m).
const SIDE_IN: f32 = 0.5;

/// Whether a taxi come to junction node `u` along stretch `came` can turn round there, nose first
/// into a side street square to its road (that road going on past it) and backing out the way it
/// came, and where: None, it can't.
fn side_turn(g: &crate::travel::Graph, net: &Net, u: u32, came: u32) -> Option<SideTurn> {
    let e_in = &g.edges[came as usize];
    let road = &g.roads[e_in.road as usize];
    let adj = &net.adj[u as usize];
    if adj.len() < 3
        || !adj
            .iter()
            .any(|&e| e != came && g.edges[e as usize].road == e_in.road)
    {
        return None;
    }
    let s = if e_in.b == u { e_in.s1 } else { e_in.s0 };
    let at = road.at(s);
    let t = road.tangent(s);
    adj.iter().find_map(|&e| {
        let arm = &g.edges[e as usize];
        if arm.road == e_in.road {
            return None;
        }
        let side = &g.roads[arm.road as usize];
        let depth = road.width / 2.0 + TAXI_LEN / 2.0 + SIDE_IN;
        if arm.len() < depth + TAXI_LEN {
            return None;
        }
        let (s_u, sign) = if arm.a == u {
            (arm.s0, 1.0)
        } else {
            (arm.s1, -1.0)
        };
        let spot = side.at(s_u + sign * depth);
        let l = crate::travel::dist(spot, at).max(1e-3);
        let a = [(spot[0] - at[0]) / l, (spot[1] - at[1]) / l];
        if (a[0] * t[0] + a[1] * t[1]).abs() > 0.5 {
            return None;
        }
        let turn = SideTurn {
            spot,
            yaw: a[0].atan2(a[1]),
            axis: [-a[0], -a[1]],
            road: e_in.road,
            s,
        };
        turn.bay(g).map(|_| turn)
    })
}

/// The point of `legs` `reach` m along from point `from` (forward, else back), or its last (first).
fn reach_along(legs: &crate::cars::Legs, from: usize, reach: f32, forward: bool) -> usize {
    let pts = &legs.pts;
    let mut d = 0.0;
    let mut i = from;
    loop {
        let j = if forward {
            if i + 1 >= pts.len() {
                return i;
            }
            i + 1
        } else {
            if i == 0 {
                return 0;
            }
            i - 1
        };
        d += crate::travel::dist(pts[i].0, pts[j].0);
        i = j;
        if d >= reach {
            return i;
        }
    }
}

/// Turning round in the turning circle of a dead end at `end` (its middle), come to going `t` (a
/// unit vector) on a road `width` m wide of `kind`: a swerve right out of the lane onto a loop
/// round the circle's middle the far way round (left), and a swerve right back into the lane the
/// other way (`u_turn_radii`: the loop's front corners over the kerb by `TURN_OVER` at most).
fn u_turn(
    legs: &mut crate::cars::Legs,
    end: [f32; 2],
    t: [f32; 2],
    width: f32,
    kind: crate::travel::RoadKind,
) {
    let (r, lane, before) = u_turn_radii(width, kind);
    let right = [-t[1], t[0]];
    // (x along the way it comes, y to its right; the dead end's middle at 0.)
    let world = |x: f32, y: f32| {
        [
            end[0] + t[0] * x + right[0] * y,
            end[1] + t[1] * x + right[1] * y,
        ]
    };
    let xa = -before;
    let arc = |legs: &mut crate::cars::Legs, c: (f32, f32), rad: f32, a0: f32, a1: f32| {
        let n = (((a1 - a0).abs() * rad) / 0.5).ceil().max(1.0) as usize;
        for i in 0..=n {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            legs.push(
                world(c.0 + rad * a.cos(), c.1 + rad * a.sin()),
                false,
                TURN_SPEED,
            );
        }
    };
    let ya = lane + SWERVE;
    // The swerve out: round (xa, ya) from below it to where it meets the loop.
    let beta = (-ya).atan2(-xa);
    arc(legs, (xa, ya), SWERVE, -std::f32::consts::FRAC_PI_2, beta);
    // The loop: round the middle the far way, from that side to the other.
    let alpha = ya.atan2(xa);
    arc(legs, (0.0, 0.0), r, alpha, -alpha);
    // The swerve back in: round (xa, -ya) to the lane the other way.
    arc(legs, (xa, -ya), SWERVE, -beta, std::f32::consts::FRAC_PI_2);
}

/// How long two cars or more by a junction's box stand still on their way before it counts as
/// jammed (ticks, `jammed`), how far round its box they're by it (m), and how long a taxi due
/// into town through one waits at the edge of town for it to clear at most (ticks, `run_on`: a
/// run to work, half its spare time at most).
const JAM_STILL: u64 = 5 * TICKS_PER_MINUTE;
/// How long one car on its way stands still before the place counts as held up on its own (ticks).
const LONE_STILL: u64 = 30 * TICKS_PER_MINUTE;
const JAM_NEAR: f32 = 8.0;
const JAM_WAIT: u64 = 30 * TICKS_PER_MINUTE;
/// A car on its way that stays within this of where it was (m) stands still, as `jammed` has it.
const STILL_REACH: f32 = 3.0;

/// Since when each car on its way has stood still (`Taxis::still`).
fn note_still(w: &mut World) {
    let tick = w.tick;
    let still = &mut w.taxis.still;
    still.resize(w.cars.len(), (None, tick));
    for (c, car) in w.cars.iter().enumerate() {
        let at = car.shown.filter(|s| s.2).map(|s| s.0).filter(|_| {
            matches!(
                car.state,
                crate::cars::CarState::Leaving | crate::cars::CarState::Arriving
            )
        });
        // (Inching on or backing up a little, as one held up does, is standing still.)
        let moved = match (at, still[c].0) {
            (Some(p), Some(q)) => crate::travel::dist(p, q) > STILL_REACH,
            _ => true,
        };
        if moved {
            still[c] = (at, tick);
        }
    }
}

/// Where cars are held up now (middles and how far round them): a junction's box with two cars or
/// more by it standing still on their way for `JAM_STILL` or longer (a ring of them waiting for
/// each other, or a queue into one); anywhere else two such cars stand near each other (held up
/// face to face on a stretch); and any one car standing still on its way `LONE_STILL` or longer.
fn jammed(w: &World) -> Vec<([f32; 2], f32)> {
    let g = &w.travel.graph;
    let mut out: Vec<([f32; 2], f32)> = w
        .taxis
        .jams
        .iter()
        .filter_map(|&b| g.boxes.get(b).copied())
        .collect();
    let held: Vec<[f32; 2]> = w
        .cars
        .iter()
        .zip(&w.taxis.still)
        .filter(|(_, s)| s.0.is_some() && w.tick >= s.1 + JAM_STILL)
        .filter_map(|(c, _)| c.shown.map(|s| s.0))
        .collect();
    // (One held up a long while on its own: there.)
    out.extend(
        w.cars
            .iter()
            .zip(&w.taxis.still)
            .filter(|(_, s)| s.0.is_some() && w.tick >= s.1 + LONE_STILL)
            .filter_map(|(c, _)| c.shown.map(|s| (s.0, JAM_NEAR))),
    );
    if held.len() >= 2 {
        let boxed = |p: [f32; 2]| {
            g.boxes
                .iter()
                .any(|&(m, r)| crate::travel::dist(p, m) < r + JAM_NEAR)
        };
        out.extend(g.boxes.iter().copied().filter(|&(m, r)| {
            held.iter()
                .filter(|p| crate::travel::dist(**p, m) < r + JAM_NEAR)
                .count()
                >= 2
        }));
        for (i, &a) in held.iter().enumerate() {
            for &b in &held[i + 1..] {
                let d = crate::travel::dist(a, b);
                if d < 2.0 * JAM_NEAR && !boxed(a) && !boxed(b) {
                    let m = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
                    out.push((m, d / 2.0 + JAM_NEAR));
                }
            }
        }
    }
    out
}

/// Whether `route` goes through one of the places held up given (`jammed`), or by it (`JAM_NEAR`:
/// the junction it holds up, the queue into it).
fn through(route: &crate::cars::Route, jams: &[([f32; 2], f32)]) -> bool {
    jams.iter().any(|&(m, r)| {
        route
            .points
            .iter()
            .any(|p| crate::travel::dist(p.0, m) < r + JAM_NEAR)
    })
}

// ---------------------------------------------------------------------------------------------
// The taxi for visits and outings.

/// The most a taxi ride to a visit or an outing may take, door to door, against walking there:
/// less than this share of the walk, else they walk (per cent).
const LEISURE_BEATS: f32 = 0.9;
/// A rider's minutes from their door to their kerb and from the place's kerb to its door, about,
/// and the taxi's stands on the way (minutes, `leisure_eta`).
const LEISURE_KERBS: f32 = 8.0;
/// Those off to the same place within this long of each other share a taxi (ticks).
const LEISURE_WINDOW: u64 = 30 * TICKS_PER_MINUTE;
/// A taxi back from a place comes this long after its rider's visit ends (ticks: a goodbye and
/// out to the kerb).
const BACK_SLACK: u64 = 8 * TICKS_PER_MINUTE;

/// About how many minutes a taxi would take `sim` from `from` to `plot` on a visit or an outing
/// (`world::leisure_way`, after the household's car and a bike of theirs, before walking): its
/// coming in from the way out nearest their home, the ride from their kerb to the place's, the
/// steps to and from the kerbs. Some, and they'd go by it: an open-world town, the place out of a
/// short walk (`autonomy.reach`), and the taxi quicker than walking (`LEISURE_BEATS`).
pub(crate) fn leisure_eta(
    travel: &crate::travel::Travel,
    exits: &[[f32; 2]],
    content: &crate::content::Content,
    sim: &crate::world::Sim,
    from: [f32; 2],
    door: [f32; 2],
) -> Option<f32> {
    use crate::travel::Mode;
    if !travel.open || exits.is_empty() || crate::travel::dist(from, door) <= content.autonomy.reach
    {
        return None;
    }
    let walk = sim.walk_per_minute(content);
    let on_foot = travel.eta(from, door, Mode::Foot, walk);
    let exit = exits
        .iter()
        .copied()
        .min_by(|a, b| crate::travel::dist(*a, from).total_cmp(&crate::travel::dist(*b, from)))?;
    let come = travel.eta(exit, from, Mode::Car, walk);
    let ride = travel.eta(from, door, Mode::Car, walk);
    let by_taxi = come + ride + LEISURE_KERBS;
    (by_taxi.is_finite() && by_taxi < on_foot * LEISURE_BEATS).then_some(by_taxi)
}

/// Resident `i` sets off from home to `plot` (a visit or an outing) by the taxi (`world`'s way
/// there: `leisure_way`): booked on one coming for those off to the same place about now (a
/// housemate, a visit party), not yet in town, with a seat; else one of their own, coming at once.
/// They wait at home till it's near (`leisure_leaving`), go out to their kerb and get in as for
/// work; it puts them down at the place's kerb and they walk on in. False: no taxi can take them
/// (they walk).
pub(crate) fn book_leisure(w: &mut World, i: usize, plot: u32, directed: bool) -> bool {
    let tick = w.tick;
    let Some(home) = w
        .households
        .get(w.sims[i].household as usize)
        .and_then(|h| h.plot)
    else {
        return false;
    };
    if w.sims[i].taxi.is_some() || plot == home {
        return false;
    }
    let seats = w.content.taxi.seats;
    // (Riding along with those off there about now.)
    let join = w.taxis.runs.iter().position(|r| {
        r.place == Some(plot)
            && !r.home
            && r.car.is_none()
            && r.stops.len() < seats
            && r.enter + LEISURE_WINDOW > tick
    });
    let mut run = match join {
        Some(k) => w.taxis.runs[k].clone(),
        None => {
            let id = w.taxis.next_id;
            Run {
                id,
                home: false,
                place: Some(plot),
                directed,
                group: (0, 0),
                enter: tick + 1,
                due_out: 0,
                depart: 0,
                stops: Vec::new(),
                next: 0,
                progress: 0.0,
                since: 0,
                car: None,
                at: Vec::new(),
                kerbs: Vec::new(),
            }
        }
    };
    // (Their halt: their housemates', else a new one after the others.)
    let halt = match run.stops.iter().find(|st| st.plot == home) {
        Some(st) => st.halt,
        None => run.stops.iter().map(|st| st.halt + 1).max().unwrap_or(0),
    };
    run.stops.push(Stop {
        sim: i as u32,
        plot: home,
        due: 0,
        leave: 0,
        state: StopState::Booked,
        halt,
    });
    run.kerbs = Vec::new();
    if !time_leisure(w, &mut run) {
        return false;
    }
    let id = run.id;
    match join {
        Some(k) => w.taxis.runs[k] = run,
        None => {
            w.taxis.next_id = w.taxis.next_id.wrapping_add(1);
            w.taxis.runs.push(run);
        }
    }
    rebook(w, id);
    true
}

/// Each rider of run `id` booked on it at their stop (`Sim::taxi`).
fn rebook(w: &mut World, id: u32) {
    let Some(r) = w.taxis.runs.iter().find(|r| r.id == id) else {
        return;
    };
    for (s, st) in r.stops.iter().enumerate() {
        if st.state != StopState::Done
            && let Some(sim) = w.sims.get_mut(st.sim as usize)
        {
            sim.taxi = Some(Booking {
                run: id,
                stop: s as u8,
            });
        }
    }
}

/// Times leisure run `run` (out to its place, or back from it), coming into town as soon as it can
/// (or, back from the place, so as to be there at its riders' `due`): its kerbs, route, when it's at
/// each halt, and when each rider leaves for their kerb (the walk there and `KERB_EARLY`). False:
/// no way.
fn time_leisure(w: &World, run: &mut Run) -> bool {
    let tick = w.tick;
    let halts = halts_of(run);
    let Some(kerbs) = kerbs_for(w, &halts, Some(run.id), tick) else {
        return false;
    };
    let (Some(way_in), Some(way_out)) = (
        nearest_way_out(w, &kerbs[0]),
        kerbs.last().and_then(|k| nearest_way_out(w, k)),
    ) else {
        return false;
    };
    let Some((route, at)) = way_for(w, way_in, &kerbs, way_out) else {
        return false;
    };
    let offset = |h: usize| at.get(h).map_or(0.0, |t| t.ceil()) as u64 + h as u64 * DWELL;
    // (Back from the place: there when the first of them is done; else at once.)
    let want = run.stops.iter().filter(|_| run.home).map(|st| st.due).min();
    run.enter = match want {
        Some(due) => due.saturating_sub(offset(0)).max(tick + 1),
        None => run.enter.max(tick + 1),
    };
    for st in &mut run.stops {
        let h = if run.home { 0 } else { st.halt as usize };
        st.due = run.enter + offset(h);
        let sim = &w.sims[st.sim as usize];
        let from = if run.home {
            sim.pos
        } else {
            w.plots[st.plot as usize].arrival_point()
        };
        let to_kerb = ((w.travel.eta(
            from,
            wait_spot(&w.travel, &kerbs[h]),
            crate::travel::Mode::Foot,
            sim.walk_per_minute(&w.content),
        ) + crate::life::OUT_OF_THE_HOUSE)
            / crate::MINUTES_PER_TICK) as u64;
        st.leave = st
            .due
            .saturating_sub(to_kerb.max(TO_KERB_LEAST) + KERB_EARLY);
    }
    run.due_out = run.enter + route.ticks().ceil() as u64 + halts.len() as u64 * DWELL;
    run.at = at;
    run.kerbs = kerbs;
    true
}

/// Those booked on a leisure run whose time to go out to their kerb has come: off they go, there
/// (their visit or outing, `world`'s way there: to the kerb, `kerb_for`) or home from the place,
/// their stay over (the taxi back comes when the first of those sharing it is done, `book_back`).
fn leisure_leaving(w: &mut World) {
    let tick = w.tick;
    let mut go: Vec<(usize, u32, bool, bool)> = Vec::new();
    for r in &w.taxis.runs {
        let Some(place) = r.place else { continue };
        for st in r.stops.iter().filter(|st| st.state == StopState::Booked) {
            let i = st.sim as usize;
            let leave = booked_leave(
                &w.taxis,
                &w.cars,
                tick,
                Booking {
                    run: r.id,
                    stop: r.stop_of(st.sim).unwrap_or(0) as u8,
                },
            );
            let Some(sim) = w.sims.get(i) else { continue };
            let on_way = |t: &TaskKind| match t {
                TaskKind::GoHome => r.home,
                TaskKind::Visit { plot } => !r.home && *plot == place,
                _ => false,
            };
            let off = sim.current().is_some_and(|a| on_way(&a.task.kind))
                || sim.queue.front().is_some_and(|t| on_way(&t.kind));
            let ready = if r.home {
                sim.visiting.is_some_and(|v| v.plot == place)
            } else {
                sim.here() && sim.away_until.is_none()
            };
            if leave.is_some_and(|l| tick >= l) && !off && ready {
                go.push((i, place, r.directed, r.home));
            }
        }
    }
    for (i, plot, directed, home) in go {
        let World {
            sims,
            content,
            objects,
            ..
        } = &mut *w;
        let sim = &mut sims[i];
        if sim.current.is_some() {
            crate::world::end_activity(sim, content, objects);
        }
        sim.current = None;
        let kind = if home {
            if let Some(v) = sim.visiting.as_mut() {
                v.until = v.until.min(tick);
            }
            TaskKind::GoHome
        } else {
            TaskKind::Visit { plot }
        };
        sim.queue.push_front(Task {
            kind,
            directed,
            routine: false,
        });
    }
}

/// Those put down at a place by a leisure taxi (`alight_at`), there now on their visit or outing:
/// a taxi back home booked for when it ends (`crate::world::Visit::until`), shared with others
/// there going home about then.
fn book_backs(w: &mut World) {
    let mut k = 0;
    while k < w.taxis.out_at.len() {
        let (i, place) = w.taxis.out_at[k];
        let Some(sim) = w.sims.get(i as usize) else {
            w.taxis.out_at.remove(k);
            continue;
        };
        let visit = sim.visiting.filter(|v| v.plot == place);
        // (Not there yet: getting out, walking in from the kerb, knocking. There, their visit's
        // end is known.)
        let coming = |t: &TaskKind| matches!(t, TaskKind::Visit { plot } if *plot == place);
        if visit.is_some()
            && (sim.current().is_some_and(|a| coming(&a.task.kind))
                || sim.queue.front().is_some_and(|t| coming(&t.kind)))
        {
            k += 1;
            continue;
        }
        let Some(v) = visit.filter(|_| sim.taxi.is_none()) else {
            w.taxis.out_at.remove(k);
            continue;
        };
        // (No stand free for it at the place yet, the taxi that brought them still there, say:
        // again next tick.)
        if book_back(w, i as usize, place, v.until.max(w.tick) + BACK_SLACK) {
            w.taxis.out_at.remove(k);
        } else {
            k += 1;
        }
    }
}

/// Resident `i`, at `place` on a visit or an outing they came to by taxi, books one home for
/// `due` (when the visit ends): with others there going home about then, else one of its own.
/// False: none can come for them now.
fn book_back(w: &mut World, i: usize, place: u32, due: u64) -> bool {
    let Some(home) = w
        .households
        .get(w.sims[i].household as usize)
        .and_then(|h| h.plot)
    else {
        return false;
    };
    let seats = w.content.taxi.seats;
    let join = w.taxis.runs.iter().position(|r| {
        r.place == Some(place)
            && r.home
            && r.car.is_none()
            && r.stops.len() < seats
            && r.stops
                .iter()
                .all(|st| st.due.abs_diff(due) <= LEISURE_WINDOW)
    });
    let mut run = match join {
        Some(k) => w.taxis.runs[k].clone(),
        None => Run {
            id: w.taxis.next_id,
            home: true,
            place: Some(place),
            directed: false,
            group: (0, 0),
            enter: due,
            due_out: 0,
            depart: 0,
            stops: Vec::new(),
            next: 0,
            progress: 0.0,
            since: 0,
            car: None,
            at: Vec::new(),
            kerbs: Vec::new(),
        },
    };
    // (Their halt home: their housemates', else a new one; the place's is the first.)
    let halt = match run.stops.iter().find(|st| st.plot == home) {
        Some(st) => st.halt,
        None => run.stops.iter().map(|st| st.halt + 1).max().unwrap_or(1),
    };
    run.stops.push(Stop {
        sim: i as u32,
        plot: home,
        due,
        leave: 0,
        state: StopState::Booked,
        halt,
    });
    run.kerbs = Vec::new();
    if !time_leisure(w, &mut run) {
        return false;
    }
    let id = run.id;
    match join {
        Some(k) => w.taxis.runs[k] = run,
        None => {
            w.taxis.next_id = w.taxis.next_id.wrapping_add(1);
            w.taxis.runs.push(run);
        }
    }
    rebook(w, id);
    true
}

/// Minutes `sim`, on a visit or an outing at `place` they came to by taxi (or booked on one back
/// from it), takes from `pos` to `door` at home by the taxi back: its coming once they're done
/// (`BACK_SLACK`), the ride, the steps to and from the kerbs (`world::outing_leave_by`). None:
/// they didn't come by taxi.
pub(crate) fn back_minutes(
    ctx: &crate::world::Ctx,
    sim: &crate::world::Sim,
    place: u32,
    pos: [f32; 2],
    door: [f32; 2],
) -> Option<f32> {
    let came = ctx.taxis.out_at.contains(&(sim.id, place))
        || sim.taxi.is_some_and(|b| {
            ctx.taxis
                .runs
                .iter()
                .any(|r| r.id == b.run && r.home && r.place == Some(place))
        });
    came.then(|| {
        BACK_SLACK as f32 * crate::MINUTES_PER_TICK
            + ctx.travel.eta(
                pos,
                door,
                crate::travel::Mode::Car,
                sim.walk_per_minute(ctx.content),
            )
            + LEISURE_KERBS
    })
}

/// Resident `i` gets out of taxi `c` (seat `seat`) at `place`'s kerb and walks on in, a visit or
/// an outing (`directed`: the player's), a taxi back booked once they're there (`book_backs`).
fn alight_at(w: &mut World, i: usize, c: usize, seat: usize, place: u32, directed: bool) {
    let door = w.cars[c].taxi_door(&w.nav(), seat).0;
    let car = &w.cars[c];
    let (at, yaw) = (car.seat_at(seat, door), car.yaw);
    let sim = &mut w.sims[i];
    sim.pos = at;
    sim.yaw = yaw;
    sim.pose = crate::content::Pose::Sit;
    sim.bound_for_car = Some(c as u32);
    sim.car_seat = seat as u8;
    sim.car_door = door as u8;
    sim.current = Some(crate::world::Activity {
        task: Task {
            kind: TaskKind::Visit { plot: place },
            directed,
            routine: false,
        },
        phase: Phase::Alighting { ticks: 0 },
        tags: Default::default(),
    });
    w.taxis.out_at.retain(|o| o.0 != i as u32);
    w.taxis.out_at.push((i as u32, place));
}

/// Riders in a leisure taxi sit in their seats as it drives (as in a household's car on a trip).
fn seat_riders(w: &mut World) {
    for r in w.taxis.runs.iter().filter(|r| r.place.is_some()) {
        let Some(c) = r.car else { continue };
        let car = &w.cars[c as usize];
        let Some((at, yaw, ..)) = car.shown else {
            continue;
        };
        let posed = car.posed(at, yaw);
        for (s, st) in r.stops.iter().enumerate() {
            if st.state != StopState::Aboard {
                continue;
            }
            let seat = 1 + s % 4;
            if let Some(sim) = w.sims.get_mut(st.sim as usize)
                && sim
                    .current()
                    .is_some_and(|a| matches!(a.phase, Phase::Driving))
            {
                sim.pos =
                    posed.seat_at(seat, usize::from(sim.car_door).min(crate::cars::DOORS - 1));
                sim.yaw = yaw;
                sim.pose = crate::content::Pose::Sit;
            }
        }
    }
}
