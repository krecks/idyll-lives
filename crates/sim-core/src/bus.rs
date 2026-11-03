//! The town's bus (content `bus`): through the town's bus stops and out, every `bus.every`
//! minutes by the clock.
//!
//! **Stops and lines.** A bus stop (`travel::ExitKind::Stop`) stands at the kerb of a road. The
//! bus pulls in to the kerb there with the stop on its right and opens its doors on that side. A
//! line comes in from a way out of town by road, calls at its stops and leaves by a way out by
//! road (`line_for`: the quickest, passing each stop on the town file's side of the road where
//! that's no long way round, `WRONG_WAY`; else the stop is served from the other side, by the
//! kerb the bus stands at, `settle`).
//! A cosy or normal town has one line through all its stops. A big world's roads are long for a
//! bus driving them at a car's pace while the clock runs a minute a second, so each stop (or two
//! near each other) has a line of its own, coming in and going out where the roads leave its area
//! (`groups`, `ways_out`). It drives the roads as a car does (`cars::Legs`), pulled in to the kerb
//! at each stop. A stop gets a shelter with a bench for three where there's room for one behind
//! the pavement, off every plot (`Stop::shelter`).
//!
//! **The timetable.** A bus comes into town every `every` minutes from `first` to `last`
//! o'clock, every day the same (`Transit::next_run`): it takes as long as its route says to each
//! stop, stands there `DWELL_TICKS` (about eight seconds at normal speed: its doors open, those
//! waiting step up and in, those home step off) and drives on. One held up on the way
//! stands less at the stops after, to make the time up. Nothing of the buses is saved: they are
//! where the clock says on load (`settle`).
//!
//! **Riding it.** Off to work or school with no car and no bike, someone whose quickest way out
//! of town is the bus (the walk to the stop, the wait there for the next bus and the ride out,
//! against the walk out: `way`) walks to the stop and waits (`Phase::AtStop`), sat on the bench
//! in the shelter or stood by it. When the bus pulls in they walk to its front door and get on
//! (`Transition::Bus`): off to work, out of sight. They leave home in time for the bus that gets
//! them there (`plan`, `life::Commute`). Done at work, someone who left from a stop comes home
//! on the next bus to call there within a headway (`off_work`), out of sight until it pulls in at
//! their stop, and gets off by its middle door and walks home. With none due (after the last), they
//! walk home from the stop. A stop no bus calls at (at night, or that no bus can reach) is a way
//! out on foot, as stops always were in big worlds (`Transit::walks_out`).
//!
//! **Big worlds.** Their stops were ways out on foot before the bus, and nobody's way to work is
//! slower for it: a big world's stop stays a way out on foot, timed as ever (`walk_stops`; no bus
//! plan). Someone who walks out by a stop takes the bus calling there when one comes within a
//! headway of their getting there and before work starts (`at_stop`), else leaves town at the stop
//! as they always did.
//!
//! People on foot wait for a bus driving by and step aside for it (`in_the_way`), as for a car,
//! and it waits for anyone in its way (`drive_on`).
//!
//! Riders aren't saved either: a game saved with someone on a bus home has them back at their
//! stop, on their way home (`save.rs`).

use crate::TICKS_PER_MINUTE;
use crate::cars::{self, Legs, Route};
use crate::content::Pose;
use crate::travel::{Mode, PathEnds, RoadKind, RoadPath, RoadPoint, Travel, dist};
use crate::world::{Ctx, Phase, Sim, World};

/// Most buses in town at once (the snapshot's rows): a big world's lines are long, a few buses
/// on each.
pub const MAX_BUSES: usize = 128;
/// A bus's length and width (m; the renderer's body).
pub const BUS_LEN: f32 = 8.6;
pub const BUS_WID: f32 = 2.3;
/// Its doors (front, middle): how far ahead of its middle they are (m). On by the front one,
/// off by the middle one.
pub const DOOR_ALONG: [f32; 2] = [3.25, -0.55];
/// From the door to where someone stands to get on or off (m, out from its side).
const DOOR_OUT: f32 = 0.35;
/// At a stop, its front door is this far on past the stop's middle (m): by the shelter's end.
const FRONT_DOOR_AT: f32 = 1.4;
/// How long it stands at a stop on time; late, at least this long; and at most this long while
/// someone is still on their way to its door.
const DWELL_TICKS: u64 = 8 * TICKS_PER_MINUTE;
const DWELL_LEAST: u64 = 3 * TICKS_PER_MINUTE;
const DWELL_MOST: u64 = 16 * TICKS_PER_MINUTE;
/// Its doors open this long after it stops and shut this long before it goes (the renderer's
/// doors take about as long).
const DOOR_TICKS: u64 = 12;
/// Planned, at the stop this long before the bus.
const SLACK_TICKS: u64 = 2 * TICKS_PER_MINUTE;
/// The stops someone weighs taking the bus from: the nearest few, this near (m).
const STOP_CANDIDATES: usize = 3;
const STOP_REACH: f32 = 600.0;
/// A big world's bus comes into its area and leaves it this far past the area's box (m): out of
/// sight of the area, its way on (hours of country road at the bus's speed) off the map.
const AREA_OUT: f32 = 70.0;
/// A big world's stops this near each other (m) share a line.
const PAIR_REACH: f32 = 140.0;
/// Over this far (m) it pulls in to the kerb before a stop, and out after.
const PULL: f32 = 12.0;
/// A line's stops tried in every order up to this many (more: nearest first).
const ORDERS_UP_TO: usize = 5;
/// What a stop passed the wrong way round costs a line (s), against the time it takes: a little
/// way round to have the town file's side of the road (else the stop is put on the other side:
/// `settle`).
const WRONG_WAY: f32 = 20.0;
/// Waiting at a stop longer than this (no bus came), they make their own way.
pub(crate) const GIVE_UP_TICKS: u32 = 60 * TICKS_PER_MINUTE as u32;
/// The shelter: how far behind the pavement its middle is (m), its length along the road and
/// depth, and its bench's seats (along it; out from its middle, towards its back).
const SHELTER_OUT: f32 = 0.7;
const SHELTER_LEN: f32 = 3.0;
const SHELTER_DEPTH: f32 = 1.2;
const BENCH: [f32; 3] = [-0.85, 0.0, 0.85];
const BENCH_BACK: f32 = 0.31;
/// Standing by the stop: along the pavement (m), the bus's way first.
const STANDING: [f32; 8] = [1.7, -1.7, 2.5, -2.5, 0.9, -0.9, 3.3, -3.3];

/// A bus stop: one of the town's ways out (`World::exits`) at the kerb of a road.
#[derive(Debug, Clone)]
pub struct Stop {
    /// Its exit (a `World::exits` index), and where that is.
    pub exit: usize,
    pub at: [f32; 2],
    /// Where it is on the road's middle line, and the way the bus goes past it (a unit vector:
    /// the stop on its right).
    pub mid: [f32; 2],
    pub dir: [f32; 2],
    /// Where it is on the road's middle line, as the roads' graph has it.
    road: RoadPoint,
    /// From the road's middle line to the kerb and to the back of the pavement (m).
    pub kerb: f32,
    pub edge: f32,
    /// The line calling here (a `Transit::lines` index) and its place on it.
    pub line: Option<(u16, u16)>,
    /// It has a shelter with a bench (room for it behind the pavement).
    pub shelter: bool,
}

impl Stop {
    /// Right of the way the bus goes: out towards the stop.
    fn out(&self) -> [f32; 2] {
        [-self.dir[1], self.dir[0]]
    }

    /// `along` m on from it the way the bus goes, `out` m out from the road's middle.
    fn point(&self, along: f32, out: f32) -> [f32; 2] {
        let o = self.out();
        [
            self.mid[0] + self.dir[0] * along + o[0] * out,
            self.mid[1] + self.dir[1] * along + o[1] * out,
        ]
    }

    /// The shelter's middle, and which way it faces (towards the road, as a resident's yaw).
    pub fn shelter_pose(&self) -> ([f32; 2], f32) {
        let o = self.out();
        (
            self.point(0.0, self.edge + SHELTER_OUT),
            (-o[0]).atan2(-o[1]),
        )
    }

    /// Waiting place `k`: where, facing which way, and whether sat (the bench's three first,
    /// with a shelter). None past the last.
    pub fn spot(&self, k: usize) -> Option<([f32; 2], f32, bool)> {
        let o = self.out();
        let road = (-o[0]).atan2(-o[1]);
        let seats = if self.shelter { BENCH.len() } else { 0 };
        if k < seats {
            let along = BENCH[k];
            return Some((
                self.point(along, self.edge + SHELTER_OUT + BENCH_BACK),
                road,
                true,
            ));
        }
        let along = *STANDING.get(k - seats)?;
        // (Facing the road, turned a little towards where the bus comes from.)
        let back = (-self.dir[0]).atan2(-self.dir[1]);
        let yaw = road + crate::world::wrap_angle(back - road) * 0.35;
        Some((self.point(along, self.edge - 0.6), yaw, false))
    }

    /// How many waiting places it has.
    fn spots(&self) -> usize {
        STANDING.len() + if self.shelter { BENCH.len() } else { 0 }
    }
}

/// A bus line: through its stops from a way out of town by road to another (`line_for`).
#[derive(Debug, Clone)]
pub struct Line {
    /// Its stops (`Transit::stops` indices) in order.
    pub stops: Vec<u16>,
    route: Route,
    /// The route's ticks at each stop, and where the bus stands there (its middle) and its yaw.
    at: Vec<f32>,
    bays: Vec<([f32; 2], f32)>,
    /// On time, ticks after it came into town that it's at each stop, and out of town again.
    pub arrive: Vec<u64>,
    pub end: u64,
}

impl Line {
    /// Where the bus stands at stop `j` (its place on the line): its middle and yaw.
    pub fn bay(&self, j: usize) -> ([f32; 2], f32) {
        self.bays[j]
    }

    /// Ticks after coming into town that it leaves stop `j` (its place on the line), on time.
    fn depart(&self, j: usize) -> u64 {
        self.arrive[j] + DWELL_TICKS
    }

    /// Where someone stands at door `d` of the bus at stop `j` (its place on the line).
    fn door(&self, j: usize, d: usize) -> [f32; 2] {
        let (c, yaw) = self.bays[j];
        let (s, co) = yaw.sin_cos();
        let (along, out) = (DOOR_ALONG[d], BUS_WID / 2.0 + DOOR_OUT);
        // (Forward is (sin, cos); its right (-cos, sin).)
        [c[0] + s * along - co * out, c[1] + co * along + s * out]
    }
}

/// A bus in town on its run.
#[derive(Debug, Clone)]
pub struct Bus {
    pub line: u16,
    /// When it came into town (its run on the timetable).
    pub run: u64,
    /// Ticks along its route (it stops for people in its way).
    progress: f32,
    /// The stop it's on its way to or at (its place on the line; past the last: on its way out).
    next: usize,
    /// At a stop: since when, and until when it stands there.
    stood: u64,
    until: Option<u64>,
    /// Those who got on, on their way to work or school.
    pub riders: Vec<u32>,
    /// Ticks it has waited for someone in its way.
    waited: u32,
    /// Where it is, which way it faces, whether it's moving, and its open doors (bit 0 the
    /// front's, bit 1 the middle's): for the snapshot.
    pub shown: Option<([f32; 2], f32, bool, u8)>,
}

impl Bus {
    /// Whether it stands at a stop with its doors open at `tick` (open a little after it stops,
    /// shut a little before it goes: `DOOR_TICKS`).
    fn doors_open(&self, tick: u64) -> bool {
        self.until
            .is_some_and(|until| tick >= self.stood + DOOR_TICKS && tick + DOOR_TICKS < until)
    }
}

/// Someone done at work coming home by bus: to their stop, on the first bus into town at or
/// after `run`.
#[derive(Debug, Clone, Copy)]
struct Homeward {
    sim: u32,
    stop: u16,
    run: u64,
}

/// The town's bus stops, lines and buses (`World::transit`). Not saved: made again on load.
#[derive(Debug, Clone, Default)]
pub struct Transit {
    pub stops: Vec<Stop>,
    pub lines: Vec<Line>,
    pub buses: Vec<Bus>,
    homeward: Vec<Homeward>,
    /// Buses sent into town up to (not including) this tick.
    sent: u64,
    /// The timetable: ticks between buses, and the ticks of the day the first and the last come.
    every: u64,
    first: u64,
    last: u64,
    /// By stop, the waiting places taken (by whom), as the last tick ended.
    taken: Vec<Vec<(u8, u32)>>,
    /// By exit, its stop (`stops` index).
    by_exit: Vec<Option<u16>>,
    /// A big world's stops stay ways out on foot as they always were, the bus or not
    /// (`walks_out`): a resident's way to work is never slower for it (`at_stop`).
    pub walk_stops: bool,
}

impl Transit {
    /// A new timetable (content `bus`), no stops yet.
    fn new(rules: crate::content::BusRules) -> Self {
        let per_minute = TICKS_PER_MINUTE as f32;
        let every = ((rules.every * per_minute).round() as u64).max(1);
        let first = (rules.first * 60.0 * per_minute).round() as u64;
        let last = ((rules.last * 60.0 * per_minute).round() as u64).max(first);
        Self {
            every,
            first,
            // (The last run on the timetable.)
            last: first + (last - first) / every * every,
            ..Default::default()
        }
    }

    /// The first bus into town at or after `at` (a tick; the next day's first after the last).
    pub fn next_run(&self, at: u64) -> u64 {
        let start = crate::clock::since_midnight(0);
        let abs = crate::clock::since_midnight(at);
        let day = crate::clock::TICKS_PER_DAY;
        let (d, tod) = (abs / day, abs % day);
        let k = tod.saturating_sub(self.first).div_ceil(self.every);
        let m = self.first + k * self.every;
        let run = if m <= self.last {
            d * day + m
        } else {
            (d + 1) * day + self.first
        };
        run - start
    }

    /// The last bus into town at or before `at`, if there was one since the game began.
    fn run_before(&self, at: u64) -> Option<u64> {
        let start = crate::clock::since_midnight(0);
        let abs = crate::clock::since_midnight(at);
        let day = crate::clock::TICKS_PER_DAY;
        let (d, tod) = (abs / day, abs % day);
        let run = if tod >= self.first {
            d * day
                + self.first
                + ((tod - self.first) / self.every * self.every).min(self.last - self.first)
        } else {
            (d.checked_sub(1)? * day) + self.last
        };
        run.checked_sub(start)
    }

    /// Whether a bus is due within one headway of `tick`: the stops are the bus's then.
    fn running(&self, tick: u64) -> bool {
        !self.lines.is_empty() && self.next_run(tick) <= tick + self.every
    }

    /// Whether someone on foot leaves town by exit `i` at `tick`: a road out, or a stop no bus
    /// calls at then (the night, or one no bus reaches).
    pub fn walks_out(&self, i: usize, tick: u64) -> bool {
        match self.by_exit.get(i).copied().flatten() {
            Some(s) => {
                self.walk_stops || self.stops[s as usize].line.is_none() || !self.running(tick)
            }
            None => true,
        }
    }

    /// Those done at work coming home by bus.
    pub fn homeward_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.homeward.iter().map(|h| h.sim)
    }

    /// Whether any bus runs in town.
    pub fn runs(&self) -> bool {
        !self.lines.is_empty()
    }

    /// The stop at exit `i`, if a line calls there.
    pub fn served(&self, i: usize) -> Option<u16> {
        self.by_exit
            .get(i)
            .copied()
            .flatten()
            .filter(|&s| self.stops[s as usize].line.is_some())
    }

    /// The bus standing at `stop` with its doors open at `tick`, if one is.
    fn boarding(&self, stop: u16, tick: u64) -> Option<usize> {
        let (l, j) = self.stops.get(stop as usize)?.line?;
        self.buses
            .iter()
            .position(|b| b.line == l && b.next == j as usize && b.doors_open(tick))
    }

    /// Where someone waits at `stop`: its first waiting place nobody has taken that they can
    /// stand on (`free`).
    fn free_spot(&self, stop: u16, me: u32, free: impl Fn([f32; 2]) -> bool) -> Option<u8> {
        let st = &self.stops[stop as usize];
        let taken = self.taken.get(stop as usize);
        (0..st.spots()).map(|k| k as u8).find(|&k| {
            taken.is_none_or(|t| t.iter().all(|&(s, who)| s != k || who == me))
                && st.spot(k as usize).is_some_and(|(p, ..)| free(p))
        })
    }

    /// Someone at `stop` (`me`) and their place there: the first nobody has taken that they can
    /// stand on (`free_spot`, on ground `nav` has free), and where it is.
    fn place_at(
        &self,
        nav: &crate::path::NavGrid,
        stop: u16,
        me: u32,
    ) -> Option<(u16, u8, [f32; 2])> {
        let spot = self.free_spot(stop, me, |p| {
            nav.tile_free(p[0].floor() as i32, p[1].floor() as i32)
        })?;
        let (at, ..) = self.stops[stop as usize].spot(spot as usize)?;
        Some((stop, spot, at))
    }

    /// The stops someone at `from` weighs taking the bus from: the nearest few a line calls at,
    /// within `STOP_REACH`, nearest first.
    fn nearest_stops(&self, from: [f32; 2]) -> Vec<u16> {
        let mut near: Vec<(f32, u16)> = self
            .stops
            .iter()
            .enumerate()
            .filter(|(_, s)| s.line.is_some())
            .map(|(k, s)| (dist(from, s.at), k as u16))
            .filter(|&(d, _)| d <= STOP_REACH)
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        near.truncate(STOP_CANDIDATES);
        near.into_iter().map(|(_, k)| k).collect()
    }

    /// Taking the bus from `pos` now, walking at `walk` m a minute: the stop, and the ticks till
    /// it's out of town with them (the walk with `life::WALK_MARGIN`, the wait for the bus they'd
    /// catch, the ride). The quickest of the nearest few stops a line calls at.
    fn best(&self, travel: &Travel, pos: [f32; 2], walk: f32, now: u64) -> Option<(u16, u64)> {
        self.nearest_stops(pos)
            .into_iter()
            .filter_map(|k| {
                let st = &self.stops[k as usize];
                let minutes = travel.eta(pos, st.at, Mode::Foot, walk) * crate::life::WALK_MARGIN;
                let there = now + (minutes * TICKS_PER_MINUTE as f32) as u64;
                let (l, _) = st.line?;
                let line = &self.lines[l as usize];
                let run = self.catches(k, there)?;
                Some((k, run + line.end - now.min(run + line.end)))
            })
            .min_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)))
    }

    /// The run of the bus someone at `stop` at `there` gets on: one standing there with its doors
    /// open, else the next to come, if it comes within a headway (not the day's first, a long way
    /// off yet, nor tomorrow's).
    fn catches(&self, stop: u16, there: u64) -> Option<u64> {
        let (l, j) = self.stops[stop as usize].line?;
        let line = &self.lines[l as usize];
        let j = j as usize;
        // (On a bus still on its way to it, or standing there with time to get on.)
        let on_the_way = self
            .buses
            .iter()
            .filter(|b| b.line == l && b.next <= j)
            .filter(|b| match b.until {
                Some(until) if b.next == j => there + DOOR_TICKS < until,
                _ => b.run + line.depart(j) > there + DOOR_TICKS,
            })
            .map(|b| b.run)
            .min();
        on_the_way
            .or_else(|| {
                // (Its door shut a little before it goes.)
                let after = (there + DOOR_TICKS + 1).saturating_sub(line.depart(j));
                Some(self.next_run(after))
            })
            .filter(|&run| run + line.arrive[j] <= there + self.every + DWELL_TICKS)
    }
}

// ---- Building the lines ------------------------------------------------------------------

/// The town's stops, lines and buses, from its stops and roads and the clock (a new town, a
/// loaded game). Called once the cars are settled (`cars::settle_all` gives a cosy town with
/// stops its road network).
pub(crate) fn settle(w: &mut World) {
    let mut t = Transit::new(w.content.bus);
    t.by_exit = vec![None; w.exits.len()];
    t.walk_stops = w.travel.size == crate::travel::WorldSize::Big;
    if !w.travel.graph.is_empty() {
        for i in 0..w.exits.len() {
            if !w.travel.is_stop(i) || !w.exits[i].iter().all(|v| v.is_finite()) {
                continue;
            }
            if let Some(stop) = make_stop(&w.travel, i, w.exits[i]) {
                t.by_exit[i] = Some(t.stops.len() as u16);
                t.stops.push(stop);
            }
        }
    }
    for (bounds, group) in groups(&w.travel, &t.stops) {
        let outs = ways_out(&w.travel, &w.exits, bounds);
        let Some(line) = line_for(&w.travel, &outs, &t.stops, &group) else {
            continue;
        };
        let l = t.lines.len() as u16;
        for (j, &s) in line.stops.iter().enumerate() {
            let stop = &mut t.stops[s as usize];
            stop.line = Some((l, j as u16));
            // Where the bus stands at it, the stop by its kerb side: passed the other way, on the
            // other side of the road; by a junction, on the road the bus is on.
            let (p, yaw) = line.bays[j];
            let (fwd, right) = ([yaw.sin(), yaw.cos()], [-yaw.cos(), yaw.sin()]);
            let out = (stop.at[0] - p[0]) * right[0] + (stop.at[1] - p[1]) * right[1];
            let along = (stop.at[0] - p[0]) * fwd[0] + (stop.at[1] - p[1]) * fwd[1];
            if out < BUS_WID / 2.0 || fwd[0] * stop.dir[0] + fwd[1] * stop.dir[1] < 0.9 {
                let bay = bay_out(stop.kerb);
                let base = [p[0] + fwd[0] * along, p[1] + fwd[1] * along];
                stop.dir = fwd;
                stop.mid = [base[0] - right[0] * bay, base[1] - right[1] * bay];
                let k = (stop.edge - 0.6).max(bay + BUS_WID / 2.0 + 0.3);
                stop.at = [stop.mid[0] + right[0] * k, stop.mid[1] + right[1] * k];
            }
        }
        t.lines.push(line);
    }
    for k in 0..t.stops.len() {
        t.stops[k].shelter = room_for_shelter(w, &t.stops[k]);
    }
    t.taken = vec![Vec::new(); t.stops.len()];
    // The buses in town now, where the timetable has them.
    let now = w.tick;
    for l in 0..t.lines.len() {
        let end = t.lines[l].end;
        let mut run = t.run_before(now);
        while let Some(r) = run.filter(|&r| r + end > now) {
            if let Some(bus) = on_time(&t.lines[l], l as u16, r, now) {
                t.buses.push(bus);
            }
            run = r.checked_sub(1).and_then(|r| t.run_before(r));
        }
    }
    t.buses.sort_by_key(|b| (b.run, b.line));
    t.buses.truncate(MAX_BUSES);
    t.sent = now + 1;
    w.transit = t;
    poses(w);
}

/// Stop at exit `i` (at `at`): on the nearest road a bus can drive within reach.
fn make_stop(travel: &Travel, exit: usize, at: [f32; 2]) -> Option<Stop> {
    let g = &travel.graph;
    let p = *g.near(at, cars::ROAD_REACH, 1, Mode::Car).first()?;
    let road = &g.roads[p.road as usize];
    let mid = road.at(p.s);
    let tan = road.tangent(p.s);
    // Right of the road's way (-z, x): the stop on that side, or the other.
    let side = (at[0] - mid[0]) * -tan[1] + (at[1] - mid[1]) * tan[0];
    let dir = if side >= 0.0 { tan } else { [-tan[0], -tan[1]] };
    Some(Stop {
        exit,
        at,
        mid,
        dir,
        road: p,
        kerb: kerb_of(road.width, road.kind),
        edge: road.width / 2.0,
        line: None,
        shelter: false,
    })
}

/// How far right of a road's middle line the bus drives on it (`kerb`: its kerb's), and pulls
/// in to at a stop: its lane, kept off the pavement on a narrow road.
fn lane_out(kerb: f32) -> f32 {
    cars::LANE.min(kerb - BUS_WID / 2.0 - 0.05).max(0.0)
}

fn bay_out(kerb: f32) -> f32 {
    (kerb - BUS_WID / 2.0 - 0.1).max(0.0)
}

/// A road's kerb from its middle line, for one of the bus's route points (its width and kind).
fn kerb_of(width: f32, kind: RoadKind) -> f32 {
    let pavement = if kind == RoadKind::Country {
        0.6
    } else {
        cars::SIDEWALK
    };
    (width / 2.0 - pavement).max(0.5)
}

/// Whether there's room for a shelter behind the pavement at `stop`: its ground on the town,
/// off every plot and every road, nothing standing there.
fn room_for_shelter(w: &World, stop: &Stop) -> bool {
    let (c, _) = stop.shelter_pose();
    let out = stop.out();
    let (hl, hd) = (SHELTER_LEN / 2.0 + 0.1, SHELTER_DEPTH / 2.0 + 0.05);
    for a in [-1.0f32, -0.5, 0.0, 0.5, 1.0] {
        for b in [-1.0f32, 0.0, 1.0] {
            let p = [
                c[0] + stop.dir[0] * hl * a + out[0] * hd * b,
                c[1] + stop.dir[1] * hl * a + out[1] * hd * b,
            ];
            let (x, z) = (p[0].floor() as i32, p[1].floor() as i32);
            if !w.lot.in_bounds(x, z)
                || w.plot_at(x, z).is_some()
                || w.blocked[w.lot.tile_index(x, z)]
            {
                return false;
            }
            // (Clear of the road, and of any other: its back edge may just touch the pavement.)
            let g = &w.travel.graph;
            if g.near(p, 4.0, 4, Mode::Foot)
                .iter()
                .any(|q| q.off < g.roads[q.road as usize].width / 2.0 - 0.25)
            {
                return false;
            }
        }
    }
    true
}

/// The town's lines' stops: one line through all of a cosy or normal town's; in a big world, whose
/// roads are long (the bus drives them at a car's pace while the clock runs a minute a second),
/// a line for each stop or each two near each other (`PAIR_REACH`) in an area (the one each is
/// nearest), coming in and going out where the roads leave the area (`ways_out`): each with the
/// bounds it goes in and out of (x0, z0, x1, z1; none: the town's ways out).
fn groups(travel: &Travel, stops: &[Stop]) -> Vec<(Option<[f32; 4]>, Vec<u16>)> {
    if stops.is_empty() {
        return Vec::new();
    }
    let areas = &travel.file.areas;
    if travel.size != crate::travel::WorldSize::Big || areas.is_empty() {
        return vec![(None, (0..stops.len() as u16).collect())];
    }
    let off = |a: &crate::travel::Area, p: [f32; 2]| {
        let dx = (a.x - p[0]).max(p[0] - (a.x + a.w)).max(0.0);
        let dz = (a.z - p[1]).max(p[1] - (a.z + a.d)).max(0.0);
        dx.hypot(dz)
    };
    let area_of = |p: [f32; 2]| {
        (0..areas.len())
            .min_by(|&a, &b| off(&areas[a], p).total_cmp(&off(&areas[b], p)))
            .expect("areas")
    };
    let of: Vec<usize> = stops.iter().map(|s| area_of(s.at)).collect();
    let mut taken = vec![false; stops.len()];
    let mut out = Vec::new();
    for k in 0..stops.len() {
        if taken[k] {
            continue;
        }
        taken[k] = true;
        let mut group = vec![k as u16];
        let mate = (k + 1..stops.len())
            .filter(|&m| {
                !taken[m] && of[m] == of[k] && dist(stops[m].at, stops[k].at) <= PAIR_REACH
            })
            .min_by(|&a, &b| {
                dist(stops[a].at, stops[k].at).total_cmp(&dist(stops[b].at, stops[k].at))
            });
        if let Some(m) = mate {
            taken[m] = true;
            group.push(m as u16);
        }
        let a = &areas[of[k]];
        let mut b = [a.x, a.z, a.x + a.w, a.z + a.d];
        for &g in &group {
            let p = stops[g as usize].at;
            b = [
                b[0].min(p[0]),
                b[1].min(p[1]),
                b[2].max(p[0]),
                b[3].max(p[1]),
            ];
        }
        out.push((Some(b), group));
    }
    out
}

/// Where a line comes into town and leaves it: the town's ways out by road; in a big world, where
/// the roads leave its `bounds` (`AREA_OUT` on past them) and the world's ways out in them, its own
/// way out of the world being hours of country road on (the world's ways out where none do).
fn ways_out(travel: &Travel, exits: &[[f32; 2]], bounds: Option<[f32; 4]>) -> Vec<RoadPoint> {
    let g = &travel.graph;
    let roads_out: Vec<RoadPoint> = (0..exits.len())
        .filter(|&i| !travel.is_stop(i) && exits[i].iter().all(|v| v.is_finite()))
        .filter_map(|i| {
            g.near(exits[i], cars::ROAD_REACH, 1, Mode::Car)
                .first()
                .copied()
        })
        .collect();
    let Some(b) = bounds else {
        return roads_out;
    };
    let m = AREA_OUT;
    let inside =
        |p: [f32; 2]| p[0] >= b[0] - m && p[0] < b[2] + m && p[1] >= b[1] - m && p[1] < b[3] + m;
    let mut out: Vec<RoadPoint> = roads_out
        .iter()
        .copied()
        .filter(|p| inside(g.point(*p)))
        .collect();
    for road in g.roads.iter().filter(|r| r.kind != RoadKind::Trail) {
        for i in 1..road.pts.len() {
            if inside(road.pts[i - 1]) != inside(road.pts[i]) {
                let s = if inside(road.pts[i - 1]) {
                    road.cum[i]
                } else {
                    road.cum[i - 1]
                };
                out.extend(g.at_source(road.source, s));
            }
        }
    }
    if out.is_empty() {
        return roads_out;
    }
    out
}

/// Whether a way along the roads (its `ends`) leaves where it's from (or comes in where it's to:
/// `at_end`) going `dir`.
fn goes(ends: PathEnds, dir: [f32; 2], at_end: bool) -> bool {
    let Some((first, last)) = ends else {
        return true;
    };
    let [a, b] = if at_end { last } else { first };
    (b[0] - a[0]) * dir[0] + (b[1] - a[1]) * dir[1] >= 0.0
}

/// A line through `group` (`stops` indices): of the orders of its stops (every one, for a few)
/// and the ways in and out of town by road, the quickest, each stop passed the right way round
/// if it can be (`WRONG_WAY`). None: no way in, out or between them by road.
fn line_for(travel: &Travel, outs: &[RoadPoint], stops: &[Stop], group: &[u16]) -> Option<Line> {
    let g = &travel.graph;
    let country = travel.country_speed;
    let stop = |k: u16| &stops[k as usize];
    let wrong = |ok: bool| if ok { 0.0 } else { WRONG_WAY };
    // Between each two stops, and in from and out to each way out: weighed by their cost and the
    // way they set off and come in (`travel::Graph::path_ends`), laid out only for the legs taken.
    let mut between: std::collections::HashMap<(u16, u16), Option<(f32, PathEnds)>> =
        Default::default();
    for &a in group {
        for &b in group {
            if a != b {
                between.insert(
                    (a, b),
                    g.path_ends(stop(a).road, stop(b).road, Mode::Car, country),
                );
            }
        }
    }
    // The quickest way in to stop `k` (`into`) or out from it: its cost, and the way out.
    let way = |k: u16, into: bool| {
        outs.iter()
            .filter_map(|&o| {
                let (from, to) = if into {
                    (o, stop(k).road)
                } else {
                    (stop(k).road, o)
                };
                let (cost, ends) = g.path_ends(from, to, Mode::Car, country)?;
                Some((cost + wrong(goes(ends, stop(k).dir, into)), o))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
    };
    let ins: Vec<Option<(f32, RoadPoint)>> = group.iter().map(|&k| way(k, true)).collect();
    let outs_: Vec<Option<(f32, RoadPoint)>> = group.iter().map(|&k| way(k, false)).collect();
    let place = |k: u16| group.iter().position(|&x| x == k).expect("in the group");
    let orders: Vec<Vec<u16>> = if group.len() <= ORDERS_UP_TO {
        permutations(group)
    } else {
        // (Nearest first, from the stop nearest a way in.)
        let mut left: Vec<u16> = group.to_vec();
        let first = *left
            .iter()
            .min_by(|&&a, &&b| {
                let c = |k: u16| ins[place(k)].as_ref().map_or(f32::INFINITY, |x| x.0);
                c(a).total_cmp(&c(b))
            })
            .expect("a stop");
        left.retain(|&k| k != first);
        let mut order = vec![first];
        while !left.is_empty() {
            let at = stop(*order.last().expect("one")).at;
            let (i, _) = left
                .iter()
                .enumerate()
                .min_by(|a, b| dist(at, stop(*a.1).at).total_cmp(&dist(at, stop(*b.1).at)))
                .expect("left");
            order.push(left.remove(i));
        }
        vec![order]
    };
    let mut best: Option<(f32, Vec<u16>)> = None;
    for order in orders {
        let (first, last) = (order[0], *order.last().expect("a stop"));
        let (Some(i), Some(o)) = (ins[place(first)], outs_[place(last)]) else {
            continue;
        };
        let mut cost = i.0 + o.0;
        let mut ok = true;
        for w in order.windows(2) {
            match between.get(&(w[0], w[1])).copied().flatten() {
                Some((c, ends)) => {
                    cost += c
                        + wrong(goes(ends, stop(w[0]).dir, false))
                        + wrong(goes(ends, stop(w[1]).dir, true));
                }
                None => ok = false,
            }
        }
        if ok && best.as_ref().is_none_or(|b| cost < b.0) {
            best = Some((cost, order));
        }
    }
    let (_, order) = best?;
    let (first, last) = (order[0], *order.last()?);
    let leg = |from: RoadPoint, to: RoadPoint| g.path(from, to, Mode::Car, country);
    let mut legs = vec![leg(ins[place(first)]?.1, stop(first).road)?];
    for w in order.windows(2) {
        legs.push(leg(stop(w[0]).road, stop(w[1]).road)?);
    }
    legs.push(leg(stop(last).road, outs_[place(last)]?.1)?);
    build_line(order, &legs, stops, travel, g.junctions().collect())
}

/// Every order of `items`.
fn permutations(items: &[u16]) -> Vec<Vec<u16>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let x = rest.remove(i);
        for mut p in permutations(&rest) {
            p.insert(0, x);
            out.push(p);
        }
    }
    out
}

/// The line's route from its legs along the roads (in to the first stop, between stops, out
/// from the last): on in from past the way in (`cars::OVERSHOOT`) and out past the way out, in
/// its lane, pulled in to the kerb at each stop and stopping there; its times to each stop.
fn build_line(
    order: Vec<u16>,
    legs: &[RoadPath],
    stops: &[Stop],
    travel: &Travel,
    junctions: Vec<[f32; 2]>,
) -> Option<Line> {
    // The middle line, and where each stop is on it.
    let mut centre: Vec<([f32; 2], f32, RoadKind)> = Vec::new();
    let mut marks: Vec<usize> = Vec::new();
    for (k, leg) in legs.iter().enumerate() {
        for &p in &leg.pts {
            if centre.last().is_some_and(|q| dist(q.0, p.0) < 0.05) {
                continue;
            }
            centre.push(p);
        }
        if k + 1 < legs.len() {
            marks.push(centre.len().checked_sub(1)?);
        }
    }
    if centre.len() < 2 || marks.len() != order.len() {
        return None;
    }
    // On past either end, off the map.
    let (w0, k0) = (centre[0].1, centre[0].2);
    let before = cars::extend_past(centre[1].0, centre[0].0);
    centre.insert(0, (before, w0, k0));
    for m in &mut marks {
        *m += 1;
    }
    let n = centre.len();
    let after = cars::extend_past(centre[n - 2].0, centre[n - 1].0);
    centre.push((after, centre[n - 1].1, centre[n - 1].2));
    // How far along each point is; the way the bus passes each stop; where it stands there (its
    // front door by the shelter's end, `FRONT_DOOR_AT`).
    let mut cum = vec![0.0f32; centre.len()];
    for i in 1..centre.len() {
        cum[i] = cum[i - 1] + dist(centre[i - 1].0, centre[i].0);
    }
    let dirs: Vec<[f32; 2]> = marks.iter().map(|&m| way_at(&centre, m)).collect();
    let back = DOOR_ALONG[0] - FRONT_DOOR_AT;
    for k in 0..marks.len() {
        let m = marks[k];
        let floor = if k == 0 { 1 } else { marks[k - 1] + 1 };
        let want = cum[m] - back;
        let to = (floor..=m)
            .min_by(|&a, &b| (cum[a] - want).abs().total_cmp(&(cum[b] - want).abs()))
            .unwrap_or(m);
        // (Only along a straight run: by a corner it stands at the stop itself.)
        let d = dirs[k];
        let straight = (to..=m).all(|i| {
            let w = way_at(&centre, i);
            w[0] * d[0] + w[1] * d[1] > 0.97
        });
        marks[k] = if straight { to } else { m };
    }
    let offs: Vec<f32> = (0..centre.len())
        .map(|i| {
            let (_, width, kind) = centre[i];
            let lane = lane_out(kerb_of(width, kind));
            marks.iter().zip(&order).fold(lane, |off, (&m, &s)| {
                let d = (cum[i] - cum[m]).abs();
                if d >= PULL {
                    return off;
                }
                let bay = bay_out(stops[s as usize].kerb);
                let k = 1.0 - d / PULL;
                let k = k * k * (3.0 - 2.0 * k);
                lane + (bay - lane) * k
            })
        })
        .collect();
    let n = centre.len();
    let lane: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let [dx, dz] = way_at(&centre, i);
            [centre[i].0[0] - dz * offs[i], centre[i].0[1] + dx * offs[i]]
        })
        .collect();
    // Thinned between the stops (they stay), timed: stopping at each.
    let mut cuts = vec![0];
    cuts.extend(marks.iter().copied());
    cuts.push(n - 1);
    let top = |i: usize| {
        let (p, _, kind) = centre[i];
        cars::top_speed(p, kind, travel, &junctions)
    };
    let mut legs_out = Legs::default();
    let mut at_index = Vec::new();
    for c in cuts.windows(2) {
        let piece =
            crate::travel::thin_with(&lane[c[0]..=c[1]], cars::ROUTE_STEP, cars::ROUTE_TOLERANCE);
        for (k, p) in piece.iter().enumerate() {
            // (The piece's points, with the top speed of the nearest middle-line point.)
            let near = (c[0]..=c[1])
                .min_by(|&a, &b| dist(lane[a], *p).total_cmp(&dist(lane[b], *p)))
                .unwrap_or(c[0]);
            let stop_here = k + 1 == piece.len() && c[1] != n - 1;
            legs_out.push(*p, false, if stop_here { 0.0 } else { top(near) });
            if stop_here {
                at_index.push(legs_out.len() - 1);
            }
        }
    }
    let mut route = legs_out.timed(false, false);
    if at_index.len() != order.len() {
        return None;
    }
    // (Stood at a stop facing the way it came in: a line turning back there turns as it leaves.)
    for &i in &at_index {
        if i > 0 {
            let (a, b) = (route.points[i - 1].0, route.points[i].0);
            route.points[i].1 = (b[0] - a[0]).atan2(b[1] - a[1]);
            route.points[i].3 = false;
        }
    }
    let at: Vec<f32> = at_index.iter().map(|&i| route.points[i].2).collect();
    let bays: Vec<([f32; 2], f32)> = at
        .iter()
        .map(|&t| {
            let (p, yaw, _) = route.at(t);
            (p, yaw)
        })
        .collect();
    let arrive: Vec<u64> = at
        .iter()
        .enumerate()
        .map(|(j, &t)| t.ceil() as u64 + j as u64 * DWELL_TICKS)
        .collect();
    let end = route.ticks().ceil() as u64 + order.len() as u64 * DWELL_TICKS;
    Some(Line {
        stops: order,
        route,
        at,
        bays,
        arrive,
        end,
    })
}

/// The way a line of middle-line points runs at point `i` (a unit vector): from the one before to
/// the one after, or the way it came in where it turns back on itself there (a line turning back
/// at a stop, at the end of a dead end).
fn way_at(pts: &[([f32; 2], f32, RoadKind)], i: usize) -> [f32; 2] {
    let n = pts.len();
    let unit = |a: [f32; 2], b: [f32; 2]| {
        let d = dist(a, b).max(1e-6);
        [(b[0] - a[0]) / d, (b[1] - a[1]) / d]
    };
    let (p, c, q) = (
        pts[i.saturating_sub(1)].0,
        pts[i].0,
        pts[(i + 1).min(n - 1)].0,
    );
    let (inn, out) = (unit(p, c), unit(c, q));
    if i > 0 && i + 1 < n && inn[0] * out[0] + inn[1] * out[1] < 0.0 {
        return inn;
    }
    unit(p, q)
}

/// Bus of line `l` in town since `run`, where it is at `now` on time; None once it's gone.
fn on_time(line: &Line, l: u16, run: u64, now: u64) -> Option<Bus> {
    let e = now.checked_sub(run)?;
    let mut bus = Bus {
        line: l,
        run,
        progress: 0.0,
        next: 0,
        stood: 0,
        until: None,
        riders: Vec::new(),
        waited: 0,
        shown: None,
    };
    for j in 0..line.stops.len() {
        if e < line.arrive[j] {
            bus.progress = (e - j as u64 * DWELL_TICKS) as f32;
            bus.progress = bus.progress.min(line.at[j]);
            bus.next = j;
            return Some(bus);
        }
        if e < line.depart(j) {
            bus.progress = line.at[j];
            bus.next = j;
            bus.stood = run + line.arrive[j];
            bus.until = Some(run + line.depart(j));
            return Some(bus);
        }
    }
    bus.next = line.stops.len();
    bus.progress = (e - line.stops.len() as u64 * DWELL_TICKS) as f32;
    (bus.progress < line.route.ticks()).then_some(bus)
}

// ---- Each tick -----------------------------------------------------------------------------

/// Once a tick (after the cars): buses sent into town by the timetable, along their way, at
/// their stops (off and on), out of town; those waiting at the stops in their places.
pub(crate) fn update(w: &mut World) {
    if w.transit.lines.is_empty() {
        return;
    }
    let tick = w.tick;
    // Into town as the timetable says.
    loop {
        let run = w.transit.next_run(w.transit.sent);
        if run > tick {
            break;
        }
        for l in 0..w.transit.lines.len() {
            if w.transit.buses.len() < MAX_BUSES
                && let Some(bus) = on_time(&w.transit.lines[l], l as u16, run, tick)
            {
                w.transit.buses.push(bus);
            }
        }
        w.transit.sent = run + 1;
    }
    w.transit.sent = w.transit.sent.max(tick + 1);
    // Whoever set off for a stop and is doing something else now isn't coming.
    for s in &mut w.sims {
        if s.bound_for_bus.is_some()
            && !s
                .current()
                .is_some_and(|a| matches!(a.task.kind, crate::world::TaskKind::Work))
        {
            s.bound_for_bus = None;
        }
    }
    let mut b = 0;
    while b < w.transit.buses.len() {
        if step_bus(w, b) {
            b += 1;
        } else {
            w.transit.buses.remove(b);
        }
    }
    // Those coming home: shown back as the bus is due at their stop. Long overdue (a bus that
    // never came): home on foot from the stop.
    let mut k = 0;
    while k < w.transit.homeward.len() {
        let h = w.transit.homeward[k];
        let i = h.sim as usize;
        let due = due_at(&w.transit, h);
        let gone = w.sims.get(i).is_none_or(|s| !s.here());
        if gone || tick > due + 2 * w.transit.every + DWELL_MOST {
            w.transit.homeward.remove(k);
            if !gone {
                get_off(w, i, None);
            }
            continue;
        }
        w.sims[i].away_until = Some(due.max(tick + 1));
        k += 1;
    }
    // The waiting places taken.
    let t = &mut w.transit;
    for list in &mut t.taken {
        list.clear();
    }
    for s in &w.sims {
        if let Some(stop) = s.bound_for_bus
            && let Some(list) = t.taken.get_mut(stop as usize)
        {
            list.push((s.bus_spot, s.id));
        }
    }
    poses(w);
}

/// When the bus `h` is to come home on is due at their stop.
fn due_at(t: &Transit, h: Homeward) -> u64 {
    let Some((l, j)) = t.stops[h.stop as usize].line else {
        return h.run;
    };
    let line = &t.lines[l as usize];
    let j = j as usize;
    // (The bus on its way there now, if it is.)
    t.buses
        .iter()
        .filter(|b| b.line == l && b.run >= h.run && b.next <= j)
        .map(|b| b.run)
        .min()
        .unwrap_or(t.next_run(h.run))
        + line.arrive[j]
}

/// Bus `b` a tick on: along its way, at a stop, or out of town (false: gone).
fn step_bus(w: &mut World, b: usize) -> bool {
    let tick = w.tick;
    let bus = &w.transit.buses[b];
    let line = &w.transit.lines[bus.line as usize];
    let n = line.stops.len();
    if let Some(until) = bus.until {
        let (l, j, stood, run) = (bus.line, bus.next, bus.stood, bus.run);
        let stop = line.stops[j];
        let mut coming = false;
        // Its doors open: those coming home get off (one done at work while it stands there too),
        // those waiting come to get on.
        if bus.doors_open(tick) {
            // (No allocation when nobody gets off here: an empty Vec holds nothing.)
            let off: Vec<u32> = w
                .transit
                .homeward
                .iter()
                .filter(|h| h.stop == stop && h.run <= run)
                .map(|h| h.sim)
                .collect();
            if !off.is_empty() {
                w.transit.homeward.retain(|h| !off.contains(&h.sim));
            }
            for (k, s) in off.into_iter().enumerate() {
                let door = w.transit.lines[l as usize].door(j, 1);
                // (One after another, a step apart.)
                let back = w.transit.lines[l as usize].bays[j].1;
                let along = [back.sin(), back.cos()];
                let at = [
                    door[0] - along[0] * 0.5 * k as f32,
                    door[1] - along[1] * 0.5 * k as f32,
                ];
                get_off(w, s as usize, Some(at));
            }
            let door = w.transit.lines[l as usize].door(j, 0);
            for s in &mut w.sims {
                if s.bound_for_bus != Some(stop) || s.away_until.is_some() || !s.here() {
                    continue;
                }
                let Some(act) = s.current.as_mut() else {
                    continue;
                };
                match act.phase {
                    Phase::AtStop { .. } => {
                        act.phase = Phase::Routing {
                            waypoints: vec![door],
                            next: 0,
                        };
                        s.pose = Pose::Stand;
                        coming = true;
                    }
                    Phase::Routing { ref waypoints, .. } => {
                        // (On their way to its door, or to the stop close by.)
                        coming |= waypoints.last().is_some_and(|&p| dist(p, door) < 0.05)
                            || dist(s.pos, door) < 12.0;
                    }
                    _ => {}
                }
            }
        }
        let bus = &mut w.transit.buses[b];
        let mut until = until;
        if coming {
            until = until.max(tick + DOOR_TICKS + 1).min(stood + DWELL_MOST);
            bus.until = Some(until);
        }
        if tick >= until {
            bus.until = None;
            bus.next += 1;
            bus.waited = 0;
        }
        return true;
    }
    drive_on(w, b);
    let bus = &mut w.transit.buses[b];
    let line = &w.transit.lines[bus.line as usize];
    if bus.next < n && bus.progress >= line.at[bus.next] {
        let j = bus.next;
        bus.progress = line.at[j];
        bus.stood = tick;
        // (Late, it stands less: on time again at the stops after.)
        let due = bus.run + line.depart(j);
        bus.until = Some(due.max(tick + DWELL_LEAST).min(tick + DWELL_TICKS));
        return true;
    }
    if bus.next >= n && bus.progress >= line.route.ticks() {
        return false;
    }
    true
}

/// A tick further along its way, unless someone is in its way (`cars::go_on`, as a car): it waits
/// for them, and someone standing there a while steps aside. Not for long (`cars::PASS_TICKS`).
/// Someone waiting for a bus, or stepping up to one, is out of its way.
fn drive_on(w: &mut World, b: usize) {
    let World {
        transit,
        sims,
        lot,
        blocked,
        stairs,
        ..
    } = w;
    let bus = &mut transit.buses[b];
    let line = &transit.lines[bus.line as usize];
    let (at, yaw, _) = line.route.at(bus.progress);
    let me = cars::Driving {
        at,
        yaw,
        len: BUS_LEN,
        wid: BUS_WID,
        ahead: cars::AHEAD,
        behind: -BUS_LEN / 2.0,
    };
    let nav = crate::path::NavGrid {
        lot,
        blocked,
        stairs,
    };
    // (Those bound for the stop it calls at next: anyone else on their way to a stop is in its
    // way as anyone is.)
    let next = line.stops.get(bus.next).copied();
    let aside = |s: &Sim| {
        (next.is_some() && s.bound_for_bus == next)
            || matches!(s.current().map(|a| &a.phase), Some(Phase::AtStop { .. }))
    };
    cars::go_on(sims, &nav, me, aside, &mut bus.waited, &mut bus.progress);
}

/// The bus driving by that a step from `from` to `to` would walk into (stepping out of its way
/// is fine): where it is and which way it faces.
pub(crate) fn in_the_way(t: &Transit, from: [f32; 2], to: [f32; 2]) -> Option<([f32; 2], f32)> {
    t.buses.iter().find_map(|b| {
        let (at, yaw, true, _) = b.shown? else {
            return None;
        };
        cars::walks_into(at, yaw, BUS_LEN, BUS_WID, from, to).then_some((at, yaw))
    })
}

/// Where every bus is and which doors are open (`Bus::shown`).
fn poses(w: &mut World) {
    let tick = w.tick;
    let t = &mut w.transit;
    for bus in &mut t.buses {
        let line = &t.lines[bus.line as usize];
        let (at, yaw, _) = line.route.at(bus.progress);
        let doors = if bus.doors_open(tick) { 0b11 } else { 0 };
        bus.shown = Some((at, yaw, bus.until.is_none() && bus.waited == 0, doors));
    }
}

// ---- Riders ----------------------------------------------------------------------------------

/// `sim` off to work or school: the stop to wait at for the bus and their place there, when the
/// bus gets them out of town sooner than walking (`walk_out`: the ticks the walk out takes).
pub(crate) fn way(ctx: &Ctx, sim: &Sim, walk_out: Option<u64>) -> Option<(u16, u8, [f32; 2])> {
    let t = ctx.transit;
    if t.lines.is_empty() || sim.bus_given_up {
        return None;
    }
    let pos = ctx.nav.stairs.ground_pos(sim.pos);
    let walk = sim.walk_per_minute(ctx.content);
    let (stop, by_bus) = t.best(ctx.travel, pos, walk, ctx.tick)?;
    if walk_out.is_some_and(|w| w <= by_bus) {
        return None;
    }
    t.place_at(&ctx.nav, stop, sim.id)
}

/// A big world's resident off to work or school by way of a bus stop (`exit`, the way out they
/// walk to, timed as ever: `walks_out`): the stop and their place there when a bus calls there
/// once they're there, within a headway and in time for work; else they leave town at the stop
/// as they always did.
pub(crate) fn at_stop(ctx: &Ctx, sim: &Sim, exit: [f32; 2]) -> Option<(u16, u8, [f32; 2])> {
    let t = ctx.transit;
    if !t.walk_stops || sim.bus_given_up {
        return None;
    }
    let stop = t.stops.iter().position(|s| {
        s.line.is_some() && ctx.exits.get(s.exit).is_some_and(|e| dist(*e, exit) < 0.01)
    })? as u16;
    let st = &t.stops[stop as usize];
    let (l, j) = st.line?;
    let pos = ctx.nav.stairs.ground_pos(sim.pos);
    let walk = sim.walk_per_minute(ctx.content);
    let minutes = ctx.travel.eta(pos, st.at, Mode::Foot, walk) * crate::life::WALK_MARGIN;
    let there = ctx.tick + (minutes * TICKS_PER_MINUTE as f32) as u64;
    let run = t.catches(stop, there)?;
    // (On before work starts.)
    let start = crate::life::start_on(ctx.content, sim, ctx.tick);
    let board = (run + t.lines[l as usize].arrive[j as usize]).max(there);
    if start.is_some_and(|start| board > start) {
        return None;
    }
    t.place_at(&ctx.nav, stop, sim.id)
}

/// `sim` is there (at their place at the stop, or at the bus's door): on they get, or they wait
/// (sat on the bench, or stood facing the road), or the bus is in: to its door.
pub(crate) fn arrive(sim: &mut Sim, ctx: &Ctx) {
    let Some(stop) = sim.bound_for_bus else {
        return;
    };
    let t = ctx.transit;
    let st = &t.stops[stop as usize];
    if t.boarding(stop, ctx.tick).is_some()
        && let Some((l, j)) = st.line
    {
        let door = t.lines[l as usize].door(j as usize, 0);
        if dist(sim.pos, door) < 0.3 {
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Bus { stop });
            return;
        }
        if let Some(act) = sim.current.as_mut() {
            act.phase = Phase::Routing {
                waypoints: vec![door],
                next: 0,
            };
        }
        return;
    }
    let Some(act) = sim.current.as_mut() else {
        return;
    };
    match st.spot(sim.bus_spot as usize) {
        // Back to their place (from the door of a bus that went).
        Some((p, ..)) if dist(sim.pos, p) > 0.4 => {
            act.phase = Phase::Routing {
                waypoints: vec![p],
                next: 0,
            };
        }
        Some((p, yaw, sit)) => {
            sim.pos = p;
            sim.yaw = yaw;
            sim.pose = if sit { Pose::Sit } else { Pose::Stand };
            act.phase = Phase::AtStop { ticks: 0 };
        }
        None => act.phase = Phase::AtStop { ticks: 0 },
    }
}

/// Waited at the stop far too long (`GIVE_UP_TICKS`: no bus came): they go another way.
pub(crate) fn give_up(sim: &mut Sim) {
    if let Some(a) = sim.current.take() {
        sim.bus_given_up = true;
        sim.bound_for_bus = None;
        sim.pose = Pose::Stand;
        sim.queue.push_front(a.task);
    }
}

/// Whether the bus at `stop` has its doors open for `i`.
pub(crate) fn may_board(w: &World, stop: u16) -> bool {
    w.transit.boarding(stop, w.tick).is_some()
}

/// `i` gets on the bus at `stop` (they're off to work: `life`): out of sight in it.
pub(crate) fn board(w: &mut World, i: usize, stop: u16) {
    let tick = w.tick;
    let t = &mut w.transit;
    let Some(b) = t.boarding(stop, tick) else {
        return;
    };
    let me = i as u32;
    if !t.buses[b].riders.contains(&me) {
        t.buses[b].riders.push(me);
    }
    // (Coming home some other way, it's from the stop, as they'd have walked there.)
    let sim = &mut w.sims[i];
    sim.pos = t.stops[stop as usize].at;
    sim.bound_for_bus = None;
    sim.bus_given_up = false;
    sim.pose = Pose::Stand;
}

/// Whether `i` is coming home by bus (done at work, they're away until it pulls in; `save.rs`:
/// saved back at their stop, on their way home).
pub(crate) fn homeward(w: &World, i: usize) -> bool {
    w.transit.homeward.iter().any(|h| h.sim == i as u32)
}

/// `i` is done at work: true when they come home by bus (they left from a stop a bus calls at
/// and one comes into town before long), false when they make their own way.
pub(crate) fn off_work(w: &mut World, i: usize) -> bool {
    let t = &w.transit;
    if t.lines.is_empty() {
        return false;
    }
    let pos = w.sims[i].pos;
    let Some(stop) = t.stops.iter().position(|s| {
        s.line.is_some()
            && (dist(s.at, pos) < 1.5 || w.exits.get(s.exit).is_some_and(|&e| dist(e, pos) < 1.5))
    }) else {
        return false;
    };
    let tick = w.tick;
    // (The next to call there, within a headway: one in town already, or to come.)
    let Some(run) = t.catches(stop as u16, tick) else {
        return false;
    };
    let h = Homeward {
        sim: i as u32,
        stop: stop as u16,
        run,
    };
    let due = due_at(t, h);
    w.transit.homeward.push(h);
    w.sims[i].away_until = Some(due.max(tick + 1));
    true
}

/// `i` comes back from work at their stop (`at`: by the bus's middle door; else where they are)
/// and walks home.
fn get_off(w: &mut World, i: usize, at: Option<[f32; 2]>) {
    let sim = &mut w.sims[i];
    sim.away_until = None;
    sim.pose = Pose::Stand;
    if let Some(at) = at {
        sim.pos = at;
    }
    cars::walk_home(w, i, None);
}

/// When `i` leaves home for work or school starting at `start` when they go by bus from where
/// they are at home (`from`, over the ground), walking at `walk` m a minute: the stop and the
/// ticks before `start` they leave, the walk to the stop (minutes, as `life::Commute::walk`).
/// None when the bus is no quicker to the way out than the walk out (`walk_out`, minutes), or
/// no bus gets them there in time.
pub(crate) fn plan(
    w: &World,
    from: [f32; 2],
    walk: f32,
    start: u64,
    walk_out: f32,
) -> Option<(u16, u64, f32)> {
    let t = &w.transit;
    // (A big world's stops are timed as the walk-out they always were.)
    if t.lines.is_empty() || t.walk_stops {
        return None;
    }
    // Out of town with time for the rest of the way (as a car: `cars::off_map_ticks`).
    let out_by = start.checked_sub(cars::off_map_ticks(w))?;
    let mut best: Option<(u16, u64, f32)> = None;
    for k in t.nearest_stops(from) {
        let st = &t.stops[k as usize];
        let (l, j) = st.line?;
        let line = &t.lines[l as usize];
        let j = j as usize;
        let minutes = w.travel.eta(from, st.at, Mode::Foot, walk) * crate::life::WALK_MARGIN
            + crate::life::OUT_OF_THE_HOUSE;
        // The last bus out of town in time.
        let Some(run) = out_by.checked_sub(line.end).and_then(|r| t.run_before(r)) else {
            continue;
        };
        // (Not the night before's last: the first buses of the day are too late.)
        if out_by - (run + line.end) > t.every {
            continue;
        }
        let board = run + line.arrive[j];
        let walk_ticks = (minutes * TICKS_PER_MINUTE as f32).ceil() as u64;
        let Some(leave) = board.checked_sub(walk_ticks + SLACK_TICKS) else {
            continue;
        };
        // Quicker than walking out: door to the way out.
        let by_bus = run + line.end - leave;
        if by_bus as f32 >= walk_out * TICKS_PER_MINUTE as f32 {
            continue;
        }
        let lead = start - leave;
        if best.is_none_or(|b| lead < b.1) {
            best = Some((k, lead, minutes));
        }
    }
    best
}

/// What `s` is doing about the bus, for the UI: waiting for it at a stop (there, not on their
/// way to it), on it to work, or coming home on it.
pub fn status(w: &World, s: &Sim) -> Option<&'static str> {
    let t = &w.transit;
    let me = s.id;
    if t.homeward.iter().any(|h| h.sim == me) {
        return Some("home");
    }
    if t.buses.iter().any(|b| b.riders.contains(&me)) {
        return Some("work");
    }
    // (At the stop: not while on their way to it.)
    let stop = t.stops.get(s.bound_for_bus? as usize)?;
    let there = matches!(s.current().map(|a| &a.phase), Some(Phase::AtStop { .. }))
        || dist(s.pos, stop.at) < 8.0;
    there.then_some("wait")
}

impl World {
    /// For tests: the stop resident `i` is waiting at or on their way to, if any.
    #[doc(hidden)]
    pub fn bound_for_bus(&self, i: usize) -> Option<u16> {
        self.sims.get(i)?.bound_for_bus
    }
}
