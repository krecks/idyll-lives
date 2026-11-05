//! The town's bus (content `bus`): through the town's bus stops and out, every `bus.every`
//! minutes by the clock (an hour; longer where a line would meet its own next bus where they
//! can't pass, and in a big world every few hours, one bus of a line on the road at a time).
//!
//! **Stops and lines.** A bus stop (`travel::ExitKind::Stop`) stands at the kerb of a road. The
//! bus pulls in to the kerb there with the stop on its right and opens its doors on that side. A
//! line comes in from a way out of town by road, calls at its stops and leaves by a way out by
//! road (`line_for`: the quickest, passing each stop on the town file's side of the road where
//! that's no long way round, `WRONG_WAY`; else the stop is served from the other side, by the
//! kerb the bus stands at, `settle`). One that goes back the way it came from its last stop turns
//! round there, or, the stop near a junction, on along its road `TURN_CLEAR` from every junction
//! (`turn_away`): its body sweeping the road round as it turns, the junction's ways stay clear.
//! A cosy or normal town has one line through all its stops. A big world's roads are long for a
//! bus driving them at a car's pace while the clock runs a minute a second, so each stop (or two
//! near each other) has a line of its own, coming in and going out where the roads leave its area
//! (`groups`, `ways_out`). It drives the roads as a car does (`cars::Legs`), pulled in to the kerb
//! at each stop. A stop gets a shelter with a bench for three where there's room for one behind
//! the pavement, off every plot (`Stop::shelter`).
//!
//! **The timetable.** A line's bus comes into town every `every` minutes from `first` to `last`
//! o'clock, every day the same (`Transit::next_run`), or less often where the line passes a
//! stretch two buses can't pass on both ways (in and out by the same narrow road, as in a forest
//! town: every 80 to 100 minutes there), so one is off it before the next comes on
//! (`Line::every`, `headway`). In a big world a line's headway is its round trip, out and back,
//! in whole ten minutes, an hour at least (`round_trip`): there's one bus of a line on the road at
//! a time, every few hours, as a country bus. One due in while its line's last is still in town
//! waits out of sight at the way in till that one is gone (`after_its_last`); still waiting
//! `WAIT_MOST` after it was due, it isn't run (`Transit::skipped`), and the next is on time.
//! Each line comes in a share of the headway after the one before (in a big world, a share of
//! the timetable's), as near it as keeps its buses from meeting another line's head-on where they
//! can't pass (`Line::offset`, `offsets`: two coming in the same way aren't sent on top of each
//! other; its times to its stops and out count from the run). It takes as long as its route says to each
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
//! on the next bus to call there within `WAIT_MOST` (`off_work`), out of sight until it pulls in at
//! their stop, and gets off by its middle door and walks home. With none due so soon (or after the
//! last), they walk home from the stop. A stop no bus calls at (at night, or that no bus can reach) is a way
//! out on foot, as stops always were in big worlds (`Transit::walks_out`).
//!
//! **Big worlds.** Their stops were ways out on foot before the bus, and nobody's way to work is
//! slower for it: a big world's stop stays a way out on foot, timed as ever (`walk_stops`; no bus
//! plan). Someone who walks out by a stop takes the bus calling there when one comes within
//! `WAIT_MOST` of their getting there and before work starts (`at_stop`), else leaves town at the
//! stop as they always did; with buses every few hours, most walk. A big world's roads are laid
//! 7 m wide (`macro.ts`), wide enough for two buses to pass, one pulled over; a world saved before
//! keeps its 6 m lanes, where buses take turns (below).
//!
//! People on foot wait for a bus driving by and step aside for it (`in_the_way`), as for a car,
//! and it waits for anyone in its way its next tick would bring it nearer to (`drive_on`,
//! `cars::go_on`; someone beside it waits for it).
//!
//! **Among the traffic.** A bus never drives, backs or pulls over into another bus, a car or
//! anyone (`clear_at` checks every move; `go_on` waits for those on foot):
//! - it keeps behind a car or bus in front of it going its way, or across it, as behind one
//!   standing at a stop (`cars::up_against`), and keeps room ahead from one in its way across a
//!   junction or coming the other way where they can't pass (`ahead_of`);
//! - coming towards another it keeps to its lane, clear of the other's where the road has room
//!   (`lane_out`); on a street a little narrower (7 m) it pulls over towards the pavement, only
//!   while passing and only as far as that needs and a margin more (`pull_over`, `PULL_MARGIN`:
//!   on a bend their bodies come nearer than their lanes; two buses half each), no nearer the
//!   edge than the kerbside furniture keeps back to there (`kerbside`, the renderer's
//!   `busStreets`). A 6 m street is too narrow for two buses either way: they take turns on it,
//!   as trains on a single track (below), and the timetable keeps them from meeting there where
//!   it can (`headway`, `offsets`);
//! - where buses still hold each other up for good (a loop of them, each waiting for the next),
//!   one gives way to another once and for all (the one with the most room behind it, then the
//!   latest run): it backs up out of that one's way and of anyone waiting for it, those right
//!   behind it with it, and stands till that one has come on past it (`make_way`, `given_way`);
//!   nobody decides again every tick, so two never take turns backing up for each other. Giving
//!   way isn't being held up; but the other standing still `IMPATIENT` itself, it's let off. Held
//!   up a while a bus no longer keeps room ahead (`IMPATIENT`), and held up past
//!   `OUT_OF_SERVICE` (none can give way: standing, or edging back and forth and no further on),
//!   it's taken out of service out of sight of the part of town in view: gone, as out of town
//!   (`withdraw`);
//! - it doesn't wait for a car that is itself held up by someone on foot (`cars::Driving::held`:
//!   they may be held where they are by the bus beside them, with nowhere to step aside to); and
//!   no car parks in a public bay a bus's body (or the ground in front of it) swings through on
//!   its line's route, its tail sweeping out round a tight corner (`bus_way`,
//!   `travel::Travel::bus_way`);
//! - with the cars it takes turns where their ways cross or meet (`traffic.rs`, `Line::claims`):
//!   into a junction's box, round where its body sweeps out of its lane turning (`traffic::sweeps`),
//!   along a stretch where a car coming the other way couldn't pass it (on any road alongside),
//!   only holding its claim there, asked for `BUS_LEAD` short of it, in the order they asked; and
//!   never into what's left of a car's (`traffic::Traffic::blocked`, backing up too). It comes in
//!   at the way in only on clear ground there, with nobody standing in it.
//!
//! **Taking turns where two can't pass** (`sections`, `signal`). Road too narrow for two buses is
//! cut into blocks (a road's 8 m stretches; a junction's own), the same for every line; each
//! line's route into refuges (where the whole bus stands on road wide enough for two, or off the
//! map at its ends) and the sections between them. A bus leaves a refuge only once it holds every
//! block of the section ahead, granted all at once or not at all, in the order the buses asked
//! (ties by the earlier run), and lets each go as its tail clears it. Buses going the same way may
//! hold a block together, following each other through (one passing it both ways, in and back
//! out, holds it alone). Why no buses can hold each other up for good there: a bus waits for a
//! section only at a refuge, holding none of it; one in a section holds all of it, so nothing
//! coming the other way or across is in it, and it waits for nothing but one ahead of it going its
//! way; a refuge is wide enough for one coming out of the next section to pass it. So every bus in
//! a section gets out of it, its blocks come free, and the first to ask for them gets them.
//!
//! A bus comes into town (into sight) only once there's room for it at the way in.
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
/// How much further out from the door those getting off stand (m).
const STEP_OFF: f32 = 0.2;
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
/// A big world's stops this near one of a line's (m) share it, `LINE_STOPS` at most: fewer lines,
/// fewer buses meeting at the junctions where their ways cross.
const PAIR_REACH: f32 = 140.0;
const LINE_STOPS: usize = 3;
/// Over this far (m) it pulls in to the kerb before a stop, and out after.
const PULL: f32 = 12.0;
/// A line's stops tried in every order up to this many (more: nearest first).
const ORDERS_UP_TO: usize = 5;
/// What a stop passed the wrong way round costs a line (s), against the time it takes: a little
/// way round to have the town file's side of the road (else the stop is put on the other side:
/// `settle`).
const WRONG_WAY: f32 = 20.0;
/// The longest someone waits for a bus they haven't timed their leaving by (at a stop they walked
/// out by, or done at work for one home), rather than make their own way (`Transit::catches`).
const WAIT_MOST: u64 = 20 * TICKS_PER_MINUTE;
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
    /// Ticks between its runs: the timetable's (content `bus.every`), or longer for a line that
    /// would meet its own next bus where they can't pass (`single_track`).
    pub every: u64,
    /// How long after its run on the timetable it comes into town (ticks): the lines spread
    /// over the headway (`settle`), so two coming in the same way, or along the same road, aren't
    /// sent on top of each other, nor meet where they can't pass (`offsets`).
    pub offset: u64,
    /// On time, ticks after its run on the timetable (`offset` and its way since) that it's at
    /// each stop, and out of town again.
    pub arrive: Vec<u64>,
    pub end: u64,
    /// At each of its route's points, how far right of its lane it can pull over (m: `pull_edge`,
    /// `pull_over`), and how far right of the road's middle that brings it.
    room: Vec<f32>,
    reach: Vec<f32>,
    /// Its stretches too narrow for two buses (`Section`), in order along its route.
    sections: Vec<Section>,
    /// Where it holds a claim to drive through, as a car does (`traffic`: with the cars, in the
    /// order they asked; between buses, as ever): the junctions' boxes and the zones it sweeps
    /// round a sharp turn (`Travel::zones`), and where a car coming the other way couldn't pass it.
    claims: Vec<crate::traffic::Claim>,
}

/// A stretch of a line's route between two refuges (where a bus stands clear of any other coming
/// the other way or across: a road wide enough for two, or the route's ends, off the map), through
/// road too narrow for two buses (`single_track`). A bus waits at `from` (in the refuge before)
/// till it holds every block of it (`cells`) at once; and lets each go as its tail clears it
/// (`frees`: by then along its route). `to`: where the refuge after begins.
#[derive(Debug, Clone)]
struct Section {
    from: f32,
    to: f32,
    /// Its blocks, and the way it goes through each (a unit vector: others going that way may
    /// hold it too, following each other through, `signal`; none, passing it both ways: it holds
    /// it alone).
    cells: Vec<(u32, [f32; 2])>,
    frees: Vec<(u32, f32)>,
}

impl Line {
    /// Where its bus is `t` ticks along its route pulled over `shift` m right of its lane, and
    /// which way it faces.
    fn place(&self, t: f32, shift: f32) -> ([f32; 2], f32) {
        let (p, yaw, _) = self.route.at(t);
        // (Forward is (sin, cos); its right (-cos, sin).)
        ([p[0] - yaw.cos() * shift, p[1] + yaw.sin() * shift], yaw)
    }

    /// How far right of its lane it can pull over `t` ticks along (m: the less of the two route
    /// points either side).
    fn room_at(&self, t: f32) -> f32 {
        let pts = &self.route.points;
        let i = pts
            .partition_point(|p| p.2 <= t)
            .clamp(1, pts.len().max(2) - 1);
        let at = |k: usize| self.room.get(k).copied().unwrap_or(0.0);
        at(i - 1).min(at(i))
    }

    /// How far right of the road's middle it can be `t` ticks along, pulled over (m: the less of
    /// the two route points either side).
    fn reach_at(&self, t: f32) -> f32 {
        let pts = &self.route.points;
        let i = pts
            .partition_point(|p| p.2 <= t)
            .clamp(1, pts.len().max(2) - 1);
        let at = |k: usize| self.reach.get(k).copied().unwrap_or(0.0);
        at(i - 1).min(at(i))
    }

    /// On time, ticks after it comes into town that it's `t` ticks along its route (it stands
    /// at each stop before: `DWELL_TICKS`).
    fn time_at(&self, t: f32) -> u64 {
        t as u64 + DWELL_TICKS * self.at.iter().filter(|&&a| a < t).count() as u64
    }

    /// Where the bus stands at stop `j` (its place on the line): its middle and yaw.
    pub fn bay(&self, j: usize) -> ([f32; 2], f32) {
        self.bays[j]
    }

    /// For tests: where its bus is along its route, a tick apart, from the way in to the way out.
    #[doc(hidden)]
    pub fn route_points(&self) -> Vec<[f32; 2]> {
        let n = self.route.ticks().ceil() as u32;
        (0..=n).map(|t| self.route.at(t as f32).0).collect()
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
    /// Its run on the timetable (it came into town its line's `offset` after).
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
    /// It waits behind another car or bus (`cars::up_against`), for one coming the other way
    /// (`oncoming`), or gives way to one (`yields`).
    queued: bool,
    /// How far right of its lane it's pulled over (m) to pass one coming the other way
    /// (`pull_over`).
    shift: f32,
    /// It backs up this tick, out of the way of another (`make_way`).
    backs: bool,
    /// The bus it gives way to (its `first`): it backs up out of its way and holds back till
    /// that one is by (`make_way`).
    yields: Option<u64>,
    /// Where the one it gives way to was along its route as it began to, where it last came on
    /// `GIVEN_STEP` from, and the ticks since: past `IMPATIENT` (held up itself, or only edging),
    /// it gives way no longer (`make_way`).
    given: (f32, f32, u32),
    /// The bus it gave way to that came on no further all the same (`IMPATIENT`), while they
    /// still hold each other up: it isn't the one to give way to that one again (`make_way`).
    gave_up: Option<u64>,
    /// Ticks it has stood still on its way (not at a stop): held up past `IMPATIENT`, it no longer
    /// keeps room ahead (`ahead_of`), only clear of the others.
    still: u32,
    /// The furthest along its route it has come since it last stood at a stop, and the ticks since
    /// then (not at a stop, not waiting at a refuge): edging back and forth, as two dodging each
    /// other may for good, it's no further on (`OUT_OF_SERVICE`).
    best: f32,
    stuck: u32,
    /// The section of its line's route it holds (`Line::sections` index: it may drive through
    /// it), and the blocks of it it still holds and till when (`Section::frees`).
    holds: Option<usize>,
    blocks: Vec<(u32, f32)>,
    /// Where it is, which way it faces, whether it's moving, and its open doors (bit 0 the
    /// front's, bit 1 the middle's): for the snapshot.
    pub shown: Option<([f32; 2], f32, bool, u8)>,
    /// Whom it waited for last tick, held up on its way (`Driving::first`; `held_up_by`).
    pub(crate) waits_for: Option<u64>,
}

impl Bus {
    /// Of two up against each other, the one to go first (`cars::Driving::first`): the earlier
    /// run, then the lower line.
    pub(crate) fn first(&self) -> u64 {
        self.run << 16 | u64::from(self.line)
    }

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
    /// Buses taken out of service since the game was loaded (`OUT_OF_SERVICE`).
    pub withdrawn: u32,
    /// Runs not run since the game was loaded: in a big world, its line's last bus still in town
    /// `WAIT_MOST` after it was due in (`after_its_last`).
    pub skipped: u32,
    /// Lines planned through a group of stops that couldn't be built (`line_for`: no way there
    /// and on out along the roads): their stops have no bus. None in a town the generator made
    /// (the generated worlds' check).
    pub unbuilt: u32,
    /// The streets where the kerbside furniture keeps back for buses and cars to pass pulled over:
    /// those a line drives, and every lane too narrow for two cars to pass in their lanes
    /// (`cars::kerbside`): the town file's road (`roads` index) and how far in from its edges the
    /// furniture may reach (`kerbside`). The world's structure carries them to the renderer
    /// (`busStreets`).
    pub bus_streets: Vec<(u32, f32)>,
    /// The blocks (`Section::cells`): which bus holds each (its `first`), and the buses waiting at
    /// a refuge for a section, in the order they asked (the tick they asked, their `first`, the
    /// section: `signal`).
    owners: Vec<Vec<(u64, [f32; 2])>>,
    asks: Vec<(u64, u64, usize)>,
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
            last,
            ..Default::default()
        }
    }

    /// Line `l`'s timetable: the ticks between its runs (`Line::every`) and the tick of the day of
    /// its last run.
    fn times(&self, l: usize) -> (u64, u64) {
        let every = self.lines.get(l).map_or(self.every, |x| x.every);
        (every, self.first + (self.last - self.first) / every * every)
    }

    /// Line `l`'s first run at or after `at` (a tick; the next day's first after the last).
    pub fn next_run(&self, l: usize, at: u64) -> u64 {
        let (every, last) = self.times(l);
        let start = crate::clock::since_midnight(0);
        let abs = crate::clock::since_midnight(at);
        let day = crate::clock::TICKS_PER_DAY;
        let (d, tod) = (abs / day, abs % day);
        let k = tod.saturating_sub(self.first).div_ceil(every);
        let m = self.first + k * every;
        let run = if m <= last {
            d * day + m
        } else {
            (d + 1) * day + self.first
        };
        run - start
    }

    /// Line `l`'s last run at or before `at`, if there was one since the game began.
    fn run_before(&self, l: usize, at: u64) -> Option<u64> {
        let (every, last) = self.times(l);
        let start = crate::clock::since_midnight(0);
        let abs = crate::clock::since_midnight(at);
        let day = crate::clock::TICKS_PER_DAY;
        let (d, tod) = (abs / day, abs % day);
        let run = if tod >= self.first {
            d * day + self.first + ((tod - self.first) / every * every).min(last - self.first)
        } else {
            (d.checked_sub(1)? * day) + last
        };
        run.checked_sub(start)
    }

    /// Whether a bus of some line is due within its headway of `tick`: the stops are the bus's
    /// then.
    fn running(&self, tick: u64) -> bool {
        (0..self.lines.len()).any(|l| self.next_run(l, tick) <= tick + self.times(l).0)
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

    /// Whether bus `b` is in town, on the road: in sight, or just come in this tick (out of town
    /// at its way in, it isn't).
    pub(crate) fn in_town(&self, b: usize) -> bool {
        let bus = &self.buses[b];
        bus.shown.is_some() && (bus.progress > 0.0 || !bus.queued) || bus.progress > 0.0
    }

    /// Where bus `b` is now (its middle) and which way it faces (`Bus::shown` is where it was as
    /// the last tick ended).
    pub(crate) fn now(&self, b: usize) -> ([f32; 2], f32) {
        let bus = &self.buses[b];
        self.lines[bus.line as usize].place(bus.progress, bus.shift)
    }

    /// The bus whose `Driving::first` is `key`: how far along its way it is (ticks), whether it
    /// stands at a stop, and whom it waited for last tick, held up (`held_up_by`).
    pub(crate) fn way_of(&self, key: u64) -> Option<(f32, bool, Option<u64>)> {
        let bus = self.buses.iter().find(|b| b.first() == key)?;
        Some((bus.progress, bus.until.is_some(), bus.waits_for))
    }

    /// Whether bus `b` is on the move: not standing at a stop, nor waiting.
    pub(crate) fn on_the_move(&self, b: usize) -> bool {
        let bus = &self.buses[b];
        bus.until.is_none() && !bus.queued
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
    /// open, else the next to come, if it comes within a headway and `WAIT_MOST` (not the day's
    /// first, a long way off yet, nor tomorrow's; the bus an hour off, they make their own way).
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
                Some(self.next_run(l as usize, after))
            })
            .filter(|&run| run + line.arrive[j] <= there + line.every.min(WAIT_MOST) + DWELL_TICKS)
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
            t.unbuilt += 1;
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
    // Each line's headway and when it comes into town after its run (`single_track`, `offsets`).
    let tracks: Vec<Vec<Track>> = t.lines.iter().map(single_track).collect();
    for (line, track) in t.lines.iter_mut().zip(&tracks) {
        line.every = if t.walk_stops {
            round_trip(t.every, line.end)
        } else {
            headway(t.every, track)
        };
    }
    offsets(&mut t.lines, &tracks, t.walk_stops.then_some(t.every));
    t.bus_streets = bus_streets(&w.travel, &t.lines);
    w.travel.bus_way = bus_way(&w.travel, &t.lines);
    t.owners = vec![Vec::new(); sections(&w.travel, &mut t.lines)];
    // Where a car coming the other way couldn't pass a bus (`Travel::narrow`), where a bus's body
    // sweeps out of its lane round a sharp turn (`Travel::zones`), and each line's claims
    // (`Line::claims`): the junctions' boxes and those zones it drives through, and where a car
    // coming the other way couldn't pass it.
    let mut narrow = crate::traffic::Narrow::default();
    let mut zones = w.travel.graph.boxes.clone();
    let mut ways = Vec::new();
    for line in &t.lines {
        let trace = crate::traffic::trace(
            |at| {
                let (p, yaw) = line.place(at, 0.0);
                (p, yaw, false)
            },
            line.route.ticks(),
        );
        if trace.is_empty() {
            ways.push((trace, Vec::new(), Vec::new(), Vec::new()));
            continue;
        }
        // (Stood at a stop it doesn't pull over.)
        let along = |t: f32| trace[trace.partition_point(|s| s.0 < t).min(trace.len() - 1)].4;
        let stops: Vec<f32> = line.at.iter().map(|&t| along(t)).collect();
        let pass: Vec<f32> = trace
            .iter()
            .map(|s| {
                let at_stop = stops
                    .iter()
                    .any(|&d| (s.4 - d).abs() < BUS_LEN / 2.0 + cars::AHEAD);
                if at_stop { 0.0 } else { line.room_at(s.0) }
            })
            .collect();
        let swing = crate::traffic::swings(&trace, BUS_LEN);
        let g = &w.travel.graph;
        let mut stuck = vec![false; trace.len()];
        for (k, ((s, &p), &out)) in trace.iter().zip(&pass).zip(&swing).enumerate() {
            let way = [s.2.sin(), s.2.cos()];
            narrow.add(s.1, way, p);
            // (A car coming the other way in its lane on any road alongside, this one or
            // another running with it, side by side with it: how far they overlap, and how far
            // it could pull over there.)
            let right = [-way[1], way[0]];
            stuck[k] = g.near(s.1, 4.0, 4, Mode::Car).iter().any(|q| {
                let road = &g.roads[q.road as usize];
                let tan = road.tangent(q.s);
                if (tan[0] * way[0] + tan[1] * way[1]).abs() < 0.9 {
                    return false;
                }
                let (m, lane) = (road.at(q.s), cars::lane_on(road.width, road.kind));
                let c = [m[0] - right[0] * lane, m[1] - right[1] * lane];
                let across = (c[0] - s.1[0]) * right[0] + (c[1] - s.1[1]) * right[1];
                let over = cars::CAR_HALF + BUS_WID / 2.0 - across.abs();
                let room = cars::pull_room(road.width, road.kind, 2.0 * cars::CAR_HALF);
                if across >= 0.0 {
                    over > -crate::traffic::PASS_GAP
                } else {
                    room + p - over - out < crate::traffic::PASS_GAP
                }
            });
        }
        zones.extend(crate::traffic::sweeps(
            &trace,
            BUS_LEN,
            BUS_WID,
            &w.travel.graph.boxes,
        ));
        let spans = crate::traffic::back_by(
            &trace,
            crate::traffic::runs(&trace, |k| stuck[k]),
            crate::traffic::NARROW_BACK,
        );
        ways.push((trace, pass, spans, line.at.clone()));
    }
    for (line, (trace, pass, spans, stops)) in t.lines.iter_mut().zip(&ways) {
        // (It lets go of what it's through as it stands at a stop clear of every junction.)
        let boxed = crate::traffic::in_boxes(trace, BUS_LEN, BUS_WID, &w.travel.graph.boxes);
        let cuts: Vec<f32> = stops
            .iter()
            .copied()
            .filter(|&t| {
                let k = trace
                    .partition_point(|s| s.0 < t)
                    .min(trace.len().max(1) - 1);
                !boxed.get(k).copied().unwrap_or(true)
            })
            .collect();
        line.claims = crate::traffic::claims(
            trace, pass, BUS_LEN, BUS_WID, &zones, spans, BUS_LEAD, &cuts,
        );
    }
    // (Cars along them take turns with the buses there: their claims again.)
    w.travel.narrow = narrow;
    w.travel.zones = zones;
    for car in &mut w.cars {
        car.reclaim(&w.travel);
    }
    for line in &mut t.lines {
        for a in &mut line.arrive {
            *a += line.offset;
        }
        line.end += line.offset;
    }
    for k in 0..t.stops.len() {
        t.stops[k].shelter = room_for_shelter(w, &t.stops[k]);
    }
    t.taken = vec![Vec::new(); t.stops.len()];
    // The buses in town now, where the timetable has them.
    let now = w.tick;
    for l in 0..t.lines.len() {
        let end = t.lines[l].end;
        let mut run = t.run_before(l, now);
        while let Some(r) = run.filter(|&r| r + end > now) {
            if let Some(bus) = on_time(&t.lines[l], l as u16, r, now) {
                t.buses.push(bus);
            }
            run = r.checked_sub(1).and_then(|r| t.run_before(l, r));
        }
    }
    t.buses.sort_by_key(|b| (b.run, b.line));
    t.buses.truncate(MAX_BUSES);
    t.sent = now + 1;
    w.transit = t;
    poses(w);
}

/// Per public parking spot (`travel::Travel::bus_way`): whether a car parked in it, a bay off the
/// road, would stand where a bus swings by on its line's route (its body, `PARKED_CLEAR` to spare,
/// on a car's as long as the longest, `PARKED_HALF`): its tail sweeps out round a tight corner by
/// a bay. (A spot on the road itself is where a car parks along the kerb, as it always did.)
fn bus_way(travel: &Travel, lines: &[Line]) -> Vec<bool> {
    (0..travel.file.parking.len())
        .map(|k| {
            let Some((at, yaw, true)) = cars::parked_at(travel, k) else {
                return false;
            };
            let car = cars::Rect {
                at,
                yaw,
                half: [PARKED_HALF[0] + PARKED_CLEAR, PARKED_HALF[1] + PARKED_CLEAR],
            };
            lines.iter().any(|line| {
                let mut t = 0.0;
                while t < line.route.ticks() {
                    // (Its body and the ground in front of it, which it keeps clear:
                    // `cars::up_against`.)
                    let (p, yaw) = line.place(t, 0.0);
                    let k = cars::AHEAD / 2.0;
                    let bus = cars::Rect {
                        at: [p[0] + yaw.sin() * k, p[1] + yaw.cos() * k],
                        yaw,
                        half: [BUS_LEN / 2.0 + k, BUS_WID / 2.0],
                    };
                    if dist(p, at) < BUS_LEN + cars::AHEAD && bus.overlap(&car) > 0.0 {
                        return true;
                    }
                    t += 1.0;
                }
                false
            })
        })
        .collect()
}
/// The half length and width of the longest car (m: `cars::drive_tiles`), and how far a bus
/// keeps clear of one parked (m), for `bus_way`.
const PARKED_HALF: [f32; 2] = [2.4, 1.0];
const PARKED_CLEAR: f32 = 0.2;

/// How far short of a junction's box a bus waits for its claim to it (m: `traffic::claims`): clear
/// of where one coming out of it the other way goes on, its length and `traffic::ROOM_GAP`.
const BUS_LEAD: f32 = BUS_LEN + crate::traffic::ROOM_GAP;

/// How far apart the blocks of a road too narrow for two buses are (m, `sections`), and how near a
/// junction one is the junction's own (m: every line through it shares it).
const BLOCK_LEN: f32 = 8.0;
const BLOCK_JUNCTION: f32 = 7.0;
/// How far clear of road too narrow for two a bus's ends stand in a refuge (m, `sections`).
const REFUGE_MARGIN: f32 = 1.0;

/// Each line's sections (`Line::sections`) and how many blocks there are in all: the road too
/// narrow for two buses (`single_track`) cut into blocks shared by every line that drives it (a
/// road's, `BLOCK_LEN` long, by where along it; a junction's own), and each line's route into
/// refuges (where its whole body, `REFUGE_MARGIN` to spare, is on road wide enough for two, and its
/// route's ends, off the map) and the sections between them.
fn sections(travel: &Travel, lines: &mut [Line]) -> usize {
    use std::collections::HashMap;
    let g = &travel.graph;
    let junctions: Vec<[f32; 2]> = g.junctions().collect();
    // (A block: a junction, or a road's stretch.)
    let mut ids: HashMap<(i64, i64), u32> = HashMap::new();
    let mut block = |p: [f32; 2]| -> Option<u32> {
        let key = if let Some(j) = junctions.iter().position(|&q| dist(p, q) < BLOCK_JUNCTION) {
            (-1, j as i64)
        } else {
            let q = *g.near(p, 4.0, 1, Mode::Car).first()?;
            (i64::from(q.road), (q.s / BLOCK_LEN).floor() as i64)
        };
        let n = ids.len() as u32;
        Some(*ids.entry(key).or_insert(n))
    };
    let half = BUS_LEN / 2.0 + REFUGE_MARGIN;
    for line in lines.iter_mut() {
        // The route's places: when, how far along, and the block where it's too narrow for two,
        // and the way it goes there.
        let mut places: Vec<(f32, f32, Option<u32>, [f32; 2])> = Vec::new();
        let (mut t, mut along, mut last) = (0.0, 0.0, line.place(0.0, 0.0).0);
        let end = line.route.ticks();
        while t <= end {
            let (p, yaw) = line.place(t, 0.0);
            along += dist(last, p);
            last = p;
            let narrow = line.reach_at(t) < BUS_WID / 2.0 + PASSING / 2.0;
            let way = [yaw.sin(), yaw.cos()];
            places.push((t, along, if narrow { block(p) } else { None }, way));
            if t == end {
                break;
            }
            t = (t + TRACK_STEP).min(end);
        }
        let n = places.len();
        // Clear: the whole body there on road wide enough for two (or at the route's ends).
        let clear: Vec<bool> = (0..n)
            .map(|k| {
                k == 0
                    || k + 1 == n
                    || places
                        .iter()
                        .filter(|q| (q.1 - places[k].1).abs() <= half)
                        .all(|q| q.2.is_none())
            })
            .collect();
        let mut out = Vec::new();
        let mut k = 0;
        while k + 1 < n {
            // From the last clear place of a refuge to the first of the next.
            if !(clear[k] && !clear[k + 1]) {
                k += 1;
                continue;
            }
            let from = k;
            let mut to = k + 1;
            while !clear[to] {
                to += 1;
            }
            // (From the refuge at the route's start, a bus waits at the very start: out of town,
            // out of sight and out of everyone's way, the way out by there too.)
            let start = (0..=from).rev().take_while(|&j| clear[j]).last() == Some(0);
            let (a, b) = (places[from].1 - half, places[to].1 + half);
            let mut frees: Vec<(u32, f32)> = Vec::new();
            let mut ways: Vec<(u32, [f32; 2], f32)> = Vec::new();
            for q in places.iter().filter(|q| q.1 >= a && q.1 <= b) {
                let Some(c) = q.2 else { continue };
                match ways.iter_mut().find(|w| w.0 == c) {
                    Some(w) => {
                        w.1 = [w.1[0] + q.3[0], w.1[1] + q.3[1]];
                        w.2 += 1.0;
                    }
                    None => ways.push((c, q.3, 1.0)),
                }
                // (Its tail past it: `half` further on.)
                let gone = places
                    .iter()
                    .find(|r| r.1 >= q.1 + half)
                    .map_or(end, |r| r.0);
                match frees.iter_mut().find(|f| f.0 == c) {
                    Some(f) => f.1 = f.1.max(gone),
                    None => frees.push((c, gone)),
                }
            }
            // (Passed both ways, in and back out, or turning: its alone, `[0, 0]`.)
            let mut cells: Vec<(u32, [f32; 2])> = ways
                .into_iter()
                .map(|(c, [x, z], k)| {
                    let n = x.hypot(z);
                    if n < 0.7 * k {
                        (c, [0.0, 0.0])
                    } else {
                        (c, [x / n, z / n])
                    }
                })
                .collect();
            cells.sort_unstable_by_key(|c| c.0);
            if !cells.is_empty() {
                out.push(Section {
                    from: if start { 0.0 } else { places[from].0 },
                    to: places[to].0,
                    cells,
                    frees,
                });
            }
            k = to;
        }
        line.sections = out;
    }
    ids.len()
}

/// The section of bus `b`'s route it's at the start of, or in, without holding it: it waits for it
/// at its start (`signal`). In one it holds, or past the last, none.
fn section_ahead(t: &Transit, b: usize) -> Option<usize> {
    let bus = &t.buses[b];
    let line = &t.lines[bus.line as usize];
    let k = line.sections.partition_point(|s| s.to <= bus.progress);
    let s = line.sections.get(k)?;
    (bus.holds != Some(k) && bus.progress + 1.0 > s.from).then_some(k)
}

/// The blocks a tick (before the buses go on): each bus lets go those its tail has cleared; one
/// at a refuge for a section it doesn't hold asks for it (once); and the asks are granted in the
/// order they came (ties by `Bus::first`), each only when every block of its section is free and
/// none an earlier ask still waits for (it takes them all at once, or none: no one is passed over).
/// A bus waits only at a refuge, holding no block of the section ahead, and goes on only holding
/// all of it: no buses can wait for each other's blocks in a ring.
fn signal(w: &mut World) {
    let tick = w.tick;
    let t = &mut w.transit;
    for bus in &mut t.buses {
        let (owners, progress, first) = (&mut t.owners, bus.progress, bus.first());
        bus.blocks.retain(|&(c, gone)| {
            let keep = progress < gone;
            if !keep {
                owners[c as usize].retain(|o| o.0 != first);
            }
            keep
        });
        let line = &t.lines[bus.line as usize];
        if bus.holds.is_some_and(|k| {
            line.sections.get(k).is_none_or(|s| progress >= s.to) && bus.blocks.is_empty()
        }) {
            bus.holds = None;
        }
    }
    // (Asks for the sections ahead; gone, a bus's asks go too.)
    let firsts: Vec<u64> = t.buses.iter().map(|b| b.first()).collect();
    t.asks.retain(|a| firsts.contains(&a.1));
    for b in 0..t.buses.len() {
        if let Some(k) = section_ahead(t, b) {
            let first = t.buses[b].first();
            if !t.asks.iter().any(|a| a.1 == first && a.2 == k) {
                t.asks.retain(|a| a.1 != first);
                t.asks.push((tick, first, k));
            }
        }
    }
    t.asks.sort_unstable();
    let mut wanted: Vec<u32> = Vec::new();
    let mut granted: Vec<u64> = Vec::new();
    for &(_, first, k) in &t.asks {
        let Some(b) = t.buses.iter().position(|x| x.first() == first) else {
            continue;
        };
        let section = &t.lines[t.buses[b].line as usize].sections[k];
        // (Free: nobody in it but those going its way, nor an earlier ask waiting for it.)
        let free = section.cells.iter().all(|&(c, way)| {
            !wanted.contains(&c)
                && t.owners[c as usize].iter().all(|&(o, theirs)| {
                    o == first || way[0] * theirs[0] + way[1] * theirs[1] > SAME_WAY
                })
        });
        if free {
            for &(c, way) in &section.cells {
                let owners = &mut t.owners[c as usize];
                if !owners.iter().any(|o| o.0 == first) {
                    owners.push((first, way));
                }
            }
            let bus = &mut t.buses[b];
            bus.holds = Some(k);
            for &(c, gone) in &section.frees {
                match bus.blocks.iter_mut().find(|x| x.0 == c) {
                    Some(x) => x.1 = x.1.max(gone),
                    None => bus.blocks.push((c, gone)),
                }
            }
            granted.push(first);
        } else {
            wanted.extend(section.cells.iter().map(|c| c.0));
        }
    }
    t.asks.retain(|a| !granted.contains(&a.1));
}

/// Bus `b` gone (out of town, or out of service): the blocks it held are free.
fn let_go(t: &mut Transit, b: usize) {
    let first = t.buses[b].first();
    for (c, _) in std::mem::take(&mut t.buses[b].blocks) {
        t.owners[c as usize].retain(|o| o.0 != first);
    }
    t.asks.retain(|a| a.1 != first);
}

/// The streets `lines` drive where the kerbside furniture keeps back (`Transit::bus_streets`).
fn bus_streets(travel: &Travel, lines: &[Line]) -> Vec<(u32, f32)> {
    let g = &travel.graph;
    // (Every lane too narrow for two cars to pass in their lanes, where they pull over.)
    let mut out: Vec<(u32, f32)> = g
        .roads
        .iter()
        .filter(|r| !matches!(r.kind, RoadKind::Country | RoadKind::Trail))
        .filter_map(|r| {
            let keep = cars::kerbside(r.width, r.kind);
            (keep < KERBSIDE).then_some((r.source, keep))
        })
        .collect();
    for line in lines {
        let mut t = 0.0;
        while t < line.route.ticks() {
            let (p, _) = line.place(t, 0.0);
            if let Some(q) = g.near(p, 3.0, 1, Mode::Car).first() {
                let road = &g.roads[q.road as usize];
                let keep = kerbside(road.width);
                if road.kind != RoadKind::Country
                    && keep < KERBSIDE
                    && !out.iter().any(|&(r, _)| r == road.source)
                {
                    out.push((road.source, keep));
                }
            }
            t += TRACK_STEP;
        }
    }
    out.sort_by_key(|&(r, _)| r);
    out
}

/// A place on a line's route where two buses can't pass, even pulled over (a 6 m street:
/// `kerbside`): where, which way it goes, and when it's there on time (ticks after it comes into
/// town).
type Track = ([f32; 2], f32, u64);

/// A line's route looked at this many ticks apart for where two buses can't pass (`single_track`),
/// two places as one within this far (m); and two buses there this near in time meet (ticks).
const TRACK_STEP: f32 = 4.0;
const TRACK_NEAR: f32 = 3.0;
const TRACK_MEET: u64 = 2 * TICKS_PER_MINUTE;
/// A line's headway is a whole number of these (ticks), and leaves this long between one bus
/// leaving a stretch where they can't pass and the next coming onto it (`headway`).
const HEADWAY_STEP: u64 = 10 * TICKS_PER_MINUTE;
const HEADWAY_SLACK: u64 = 5 * TICKS_PER_MINUTE;

/// The places along `line`'s route where two buses can't pass (`Track`).
fn single_track(line: &Line) -> Vec<Track> {
    let mut out = Vec::new();
    let mut t = 0.0;
    while t < line.route.ticks() {
        if line.reach_at(t) < BUS_WID / 2.0 + PASSING / 2.0 {
            let (p, yaw) = line.place(t, 0.0);
            out.push((p, yaw, line.time_at(t)));
        }
        t += TRACK_STEP;
    }
    out
}

/// Where two buses, going `a` and `b` (yaws), meet head-on if they're at `p` and `q` together.
fn head_on(p: [f32; 2], a: f32, q: [f32; 2], b: f32) -> bool {
    dist(p, q) < TRACK_NEAR && (a - b).cos() < -SAME_WAY
}

/// A line's headway (ticks): the timetable's (`every`), or, where the line passes a stretch it
/// can't pass on both ways (in and out by the same narrow road, `track`), long enough for one bus
/// to be off it before the next comes on (`HEADWAY_SLACK`), in whole `HEADWAY_STEP`s.
fn headway(every: u64, track: &[Track]) -> u64 {
    let mut span = 0;
    for (i, a) in track.iter().enumerate() {
        for b in &track[i + 1..] {
            if head_on(a.0, a.1, b.0, b.1) {
                span = span.max(b.2.abs_diff(a.2));
            }
        }
    }
    if span == 0 {
        return every;
    }
    every.max((span + HEADWAY_SLACK).div_ceil(HEADWAY_STEP) * HEADWAY_STEP)
}

/// A big world's line's headway (ticks): its round trip in town (`end`, out and back), in whole
/// `HEADWAY_STEP`s, the timetable's (`every`) at least: one bus of it on the road at a time (one
/// late waits out of town for the last to be gone: `waits_to_come_in`).
fn round_trip(every: u64, end: u64) -> u64 {
    every.max(end.div_ceil(HEADWAY_STEP) * HEADWAY_STEP)
}

/// When each line comes into town after its run (`Line::offset`): the lines of a headway spread
/// over it (in a big world, every line over the timetable's headway, `spread`, whatever its
/// own), each as near its share as keeps its buses from meeting another line's head-on where
/// they can't pass (`tracks`), whole minutes on (the fewest such meetings, if none does).
fn offsets(lines: &mut [Line], tracks: &[Vec<Track>], spread: Option<u64>) {
    // Every two places of two lines' where their buses would meet head-on, by the time between them.
    let meets = |a: usize, b: usize| -> Vec<i64> {
        let mut out = Vec::new();
        for p in &tracks[a] {
            for q in &tracks[b] {
                if head_on(p.0, p.1, q.0, q.1) {
                    out.push(p.2 as i64 - q.2 as i64);
                }
            }
        }
        out
    };
    let n = lines.len();
    let mut done: Vec<usize> = Vec::new();
    for l in 0..n {
        let every = lines[l].every;
        let same: Vec<usize> = (0..n).filter(|&k| lines[k].every == every).collect();
        let share = match spread {
            Some(span) => l as u64 * span.min(every) / n as u64,
            None => {
                same.iter().position(|&k| k == l).unwrap_or(0) as u64 * every / same.len() as u64
            }
        };
        let others: Vec<(usize, Vec<i64>)> = done
            .iter()
            .filter(|&&k| lines[k].every == every)
            .map(|&k| (k, meets(l, k)))
            .filter(|(_, m)| !m.is_empty())
            .collect();
        let clashes = |offset: u64| {
            others
                .iter()
                .map(|(k, m)| {
                    let shift = offset as i64 - lines[*k].offset as i64;
                    m.iter()
                        .filter(|&&d| {
                            let gap = (d + shift).rem_euclid(every as i64) as u64;
                            gap.min(every - gap) < TRACK_MEET
                        })
                        .count()
                })
                .sum::<usize>()
        };
        let step = TICKS_PER_MINUTE;
        lines[l].offset = (0..every / step)
            .map(|k| (share + k * step) % every)
            .min_by_key(|&o| clashes(o))
            .unwrap_or(share);
        done.push(l);
    }
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
/// in to at a stop: its lane, clear of a bus coming the other way in its own (`PASSING`) where the
/// road has room for both, kept off the pavement on a narrow road (where the two pull over to
/// pass, `pull_over`).
fn lane_out(kerb: f32) -> f32 {
    (BUS_WID / 2.0 + PASSING / 2.0)
        .min(kerb - BUS_WID / 2.0 - 0.05)
        .max(0.0)
}

/// How far apart two buses pass each other in their lanes, at least, where the road has room (m).
pub(crate) const PASSING: f32 = 0.1;
/// How much further apart than `PASSING` two coming towards each other pull over to pass, where
/// the road has room (m, `pull_over`): its measure is across a way between theirs, the check that
/// holds them up (`too_near`) their bodies as they are, which on a bend come nearer; pulled over
/// only as far as one says, the other could hold both up.
pub(crate) const PULL_MARGIN: f32 = 0.3;
/// How near the road's edge a bus pulled over to pass another comes (m from it, `pull_over`):
/// a country road's verge; on a street or lane, short of the kerbside furniture on its pavement
/// (the renderer's street lamps, hydrants, bins and signs, `KERBSIDE` in from the edge; the
/// shelter and the bench are behind the pavement), clear of it by `EDGE`.
fn pull_edge(kind: RoadKind, width: f32, buses: bool) -> f32 {
    if kind == RoadKind::Country {
        EDGE
    } else if buses {
        bus_kerbside(width) + EDGE
    } else {
        kerbside(width) + EDGE
    }
}

/// How far in from a bus street's edge its kerbside furniture may reach (m; `busStreets` in the
/// world's structure, which the renderer's street furniture keeps to): on a street wide enough
/// for two buses pulled over but not for them in their lanes (7 m or so), only as far as leaves
/// them room to pass (`PASSING`), at the back of the pavement; elsewhere where it always stood
/// (`KERBSIDE`; a 6 m street is too narrow for two buses either way: one waits, `single_track`).
pub fn kerbside(width: f32) -> f32 {
    bus_kerbside(width).min(cars::kerbside(width, RoadKind::Street))
}

/// `kerbside` for two buses to pass each other (`Line::reach`, `single_track`): on a 6 m lane,
/// where it always stood (`KERBSIDE`: one waits there), whatever room it leaves cars to pass a
/// bus (`cars::kerbside`).
pub(crate) fn bus_kerbside(width: f32) -> f32 {
    let back = width / 2.0 - BUS_WID - PASSING / 2.0 - EDGE;
    if back >= KERBSIDE_BACK {
        back.min(KERBSIDE)
    } else {
        KERBSIDE
    }
}
const EDGE: f32 = 0.1;
/// How far in from a street's edge its kerbside furniture reaches (m: `street.ts` stands it
/// `SIDEWALK - KERB - 0.3` in, its widest about 0.3 across).
pub(crate) const KERBSIDE: f32 = cars::SIDEWALK - 0.14 - 0.3 + 0.3;
/// The least room at the back of the pavement for kerbside furniture (m from the edge: a lamp
/// post's or a bin's width and a little).
const KERBSIDE_BACK: f32 = 0.8;
/// How far a bus pulls over (or back) a tick (m), and for one coming the other way how far ahead
/// of its front (m, from the other's front).
pub(crate) const PULL_RATE: f32 = 0.06;
pub(crate) const PULL_AHEAD: f32 = 16.0;

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
/// a line for each stop or each few near each other (`PAIR_REACH`, `LINE_STOPS`) in an area (the
/// one each is nearest), coming in and going out where the roads leave the area (`ways_out`): each
/// with the bounds it goes in and out of (x0, z0, x1, z1; none: the town's ways out).
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
        // (The nearest of its area's to any of the line's so far, one at a time.)
        while group.len() < LINE_STOPS {
            let near = |m: usize| {
                group
                    .iter()
                    .map(|&g| dist(stops[m].at, stops[g as usize].at))
                    .fold(f32::INFINITY, f32::min)
            };
            let mate = (k + 1..stops.len())
                .filter(|&m| !taken[m] && of[m] == of[k] && near(m) <= PAIR_REACH)
                .min_by(|&a, &b| near(a).total_cmp(&near(b)));
            let Some(m) = mate else { break };
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
    let out = outs_[place(last)]?.1;
    // (The way out one leg, turning round on along the road where it can: `turn_away`.)
    let away = turn_away(travel, stop(last), out).and_then(|turn| {
        let (mut a, b) = (leg(stop(last).road, turn)?, leg(turn, out)?);
        a.pts.extend(b.pts);
        (a.cost, a.len) = (a.cost + b.cost, a.len + b.len);
        Some(a)
    });
    legs.push(match away {
        Some(a) => a,
        None => leg(stop(last).road, out)?,
    });
    build_line(order, &legs, stops, travel, g.junctions().collect())
}

/// Where a line whose way out from its last stop `last` to `out` turns back at it turns round
/// instead, when the stop is near a junction (`TURN_CLEAR`): on along its road the way the bus
/// goes past it to where it's that far from every junction (its body sweeping the road round, the
/// junction's ways stay clear). None: it goes on, or turns round where it is.
fn turn_away(travel: &Travel, last: &Stop, out: RoadPoint) -> Option<RoadPoint> {
    let g = &travel.graph;
    let (_, ends) = g.path_ends(last.road, out, Mode::Car, travel.country_speed)?;
    let junctions: Vec<[f32; 2]> = g.junctions().collect();
    let clear = |p: [f32; 2]| junctions.iter().all(|&j| dist(j, p) >= TURN_CLEAR);
    if goes(ends, last.dir, false) || clear(last.mid) {
        return None;
    }
    let road = &g.roads[last.road.road as usize];
    let tan = road.tangent(last.road.s);
    let k = if tan[0] * last.dir[0] + tan[1] * last.dir[1] >= 0.0 {
        1.0
    } else {
        -1.0
    };
    (1..=6).find_map(|n| {
        let s = last.road.s + k * 20.0 * n as f32;
        if s <= 0.0 || s >= road.len() {
            return None;
        }
        let p = road.at(s);
        clear(p)
            .then(|| g.near(p, 1.0, 1, Mode::Car).first().copied())
            .flatten()
            .filter(|q| q.road == last.road.road)
    })
}

/// A line turns round no nearer a junction than this (m, `turn_away`).
const TURN_CLEAR: f32 = 45.0;

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
    // (How far right of its lane it can pull over at each: to the road's edge, `EDGE` short.)
    let mut room = Vec::new();
    let mut reach = Vec::new();
    for c in cuts.windows(2) {
        let piece =
            crate::travel::thin_with(&lane[c[0]..=c[1]], cars::ROUTE_STEP, cars::ROUTE_TOLERANCE);
        for (k, p) in piece.iter().enumerate() {
            // (The piece's points, with the top speed of the nearest middle-line point.)
            let near = (c[0]..=c[1])
                .min_by(|&a, &b| dist(lane[a], *p).total_cmp(&dist(lane[b], *p)))
                .unwrap_or(c[0]);
            let stop_here = k + 1 == piece.len() && c[1] != n - 1;
            let had = legs_out.len();
            legs_out.push(*p, false, if stop_here { 0.0 } else { top(near) });
            if legs_out.len() > had {
                let (_, width, kind) = centre[near];
                let edge = width / 2.0 - pull_edge(kind, width, false) - BUS_WID / 2.0;
                room.push((edge - offs[near]).max(0.0));
                // (Two buses pass only where they always did: `bus_kerbside`.)
                let edge = width / 2.0 - pull_edge(kind, width, true) - BUS_WID / 2.0;
                reach.push(edge.max(offs[near]));
            }
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
        every: 0,
        offset: 0,
        arrive,
        end,
        room,
        reach,
        sections: Vec::new(),
        claims: Vec::new(),
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

/// Two buses going within about this of the other way (the cosine of the angle between them, less
/// than minus it) pass each other (`passing`).
const SAME_WAY: f32 = 0.7;

/// What bus `b` keeps clear of: every car and bus on the road (`cars::vehicles`) and the cars
/// parked (it never drives into one: off the road, where it pulls over, by a stop).
fn obstacles(w: &World, b: usize) -> Vec<cars::Driving> {
    let mut all = cars::vehicles(&w.cars, &w.transit, None, Some(b));
    all.extend(cars::parked(&w.cars));
    all
}

/// Bus `b` where it would be a tick on, for those in its way (`cars::go_on`): driving on, or
/// (`back`) backing up.
fn driving(t: &Transit, b: usize, back: bool) -> cars::Driving {
    let bus = &t.buses[b];
    let step = if back { -1.0 } else { 1.0 };
    let (at, yaw) = t.lines[bus.line as usize].place((bus.progress + step).max(0.0), bus.shift);
    // (Backing up out of town, it's past the way in: `drive_on`.)
    let (ahead, behind) = if back {
        (0.0, cars::AHEAD)
    } else {
        (cars::AHEAD, 0.0)
    };
    cars::Driving {
        at,
        yaw,
        len: BUS_LEN,
        wid: BUS_WID,
        ahead,
        behind,
        first: bus.first(),
        held: false,
    }
}

/// Bus `b` where it is now, as others keep clear of it (`cars::vehicles`).
fn standing(t: &Transit, b: usize) -> cars::Driving {
    let bus = &t.buses[b];
    let (at, yaw) = t.lines[bus.line as usize].place(bus.progress, bus.shift);
    cars::Driving {
        at,
        yaw,
        len: BUS_LEN,
        wid: BUS_WID,
        ahead: cars::AHEAD,
        behind: 0.0,
        first: bus.first(),
        held: false,
    }
}

/// Those of `others` (indices) bus `b` waits for (`me`: where it would be a tick on, `back`:
/// backing up): any in the ground behind it, backing up; else one it would come up against or too
/// near (`cars::up_against`, `oncoming`), or one it keeps room from on its way ahead (`ahead_of`);
/// and either way any it would move into (`clear_at`: turning, its back swings out).
fn waits_for(
    t: &Transit,
    b: usize,
    me: &cars::Driving,
    others: &[cars::Driving],
    back: bool,
) -> Vec<usize> {
    let bus = &t.buses[b];
    let to = if back {
        (bus.progress - 1.0).max(0.0)
    } else {
        bus.progress + 1.0
    };
    // (Any it would move into, turning or swinging round: `clear_at`.)
    let bumps = (0..others.len())
        .filter(|&k| !clear_at(t, std::slice::from_ref(&others[k]), b, to, bus.shift));
    let mut near: Vec<usize> = if back {
        cars::behind(me, others).chain(bumps).collect()
    } else {
        oncoming(me, others)
            .chain(cars::against(me, others, keeps_across(bus)))
            .chain(ahead_of(t, b, others))
            .chain(bumps)
            .collect()
    };
    near.sort_unstable();
    near.dedup();
    near
}

/// The buses (indices) of those bus `b` waits for (`waits_for`).
fn held_by<'a>(
    t: &'a Transit,
    b: usize,
    me: &'a cars::Driving,
    others: &'a [cars::Driving],
    back: bool,
) -> impl Iterator<Item = usize> + 'a {
    waits_for(t, b, me, others, back)
        .into_iter()
        .filter_map(move |k| t.buses.iter().position(|x| x.first() == others[k].first))
}

/// How far ahead along its way a bus keeps room (m, `ahead_of`), and its way looked at this many
/// ticks apart, at most this many ticks on.
const ROOM_AHEAD: f32 = 6.0;
/// How far in front of its body a bus looks for someone on foot in its way (m, `drive_on`).
const BUS_WALK_AHEAD: f32 = 0.3;
/// Where a bus will be on its way (ticks on) that those it waits for step clear of (`drive_on`).
const BUS_SWEEP_LOOK: [f32; 4] = [5.0, 10.0, 20.0, 30.0];
/// A bus turning further than this over those (radians) sweeps out to one side (`drive_on`).
const BUS_SWEEP_TURN: f32 = 0.5;
/// Held up this long on its way (ticks), a bus no longer keeps room ahead (`Bus::still`), and one
/// it gives way to standing still that long is given way to no longer (`make_way`): those keep it
/// out of the others' way where they'd hold each other up, never for good.
const IMPATIENT: u32 = 10 * TICKS_PER_MINUTE as u32;
/// Held up this long on its way (ticks), a bus is taken out of service (`step_bus`), out of sight:
/// further than this (m) from the part of town in view.
const OUT_OF_SERVICE: u32 = 40 * TICKS_PER_MINUTE as u32;
const VIEW_REACH: f32 = 20.0;
const ROOM_STEP: f32 = 3.0;
const ROOM_TICKS: f32 = 60.0;

/// Those of `others` (indices) bus `b` keeps room from on its way ahead (`ROOM_AHEAD`), stopping
/// where it still has room to back up if it must (`make_way`): one whose body is on its way there
/// going across it (into a junction another is in), or a bus coming the other way where the two
/// couldn't pass even pulled right over (`pull_over`; one standing at a stop doesn't).
fn ahead_of(t: &Transit, b: usize, others: &[cars::Driving]) -> Vec<usize> {
    let bus = &t.buses[b];
    if bus.still > IMPATIENT {
        return Vec::new();
    }
    let line = &t.lines[bus.line as usize];
    let (from, _) = line.place(bus.progress, bus.shift);
    let me = standing(t, b);
    let mut out: Vec<usize> = Vec::new();
    let mut k = ROOM_STEP;
    while k <= ROOM_TICKS && bus.progress + k <= line.route.ticks() {
        let at = bus.progress + k;
        let (p, yaw) = line.place(at, bus.shift);
        if dist(p, from) > ROOM_AHEAD {
            break;
        }
        let mine = cars::Rect {
            at: p,
            yaw,
            half: [BUS_LEN / 2.0, BUS_WID / 2.0],
        };
        let (over, _) = line.place(at, line.room_at(at).max(bus.shift));
        for (i, o) in others.iter().enumerate() {
            if out.contains(&i) || !o.free(&me) || dist(o.at, p) > BUS_LEN + o.len {
                continue;
            }
            let ([s, c], way) = ([yaw.sin(), yaw.cos()], o.heading());
            let cos = s * way[0] + c * way[1];
            let theirs = cars::Rect {
                at: o.at,
                yaw: o.yaw,
                half: [o.len / 2.0, o.wid / 2.0],
            };
            let held = if cos < -SAME_WAY {
                // (A bus: pulled right over, unless it stands at a stop.)
                t.buses
                    .iter()
                    .position(|x| x.first() == o.first)
                    .is_some_and(|x| {
                        let them = &t.buses[x];
                        let room = if them.until.is_some() {
                            them.shift
                        } else {
                            t.lines[them.line as usize]
                                .room_at(them.progress)
                                .max(them.shift)
                        };
                        let (q, facing) = t.lines[them.line as usize].place(them.progress, room);
                        too_near((over, yaw, BUS_LEN, BUS_WID), (q, facing, BUS_LEN, BUS_WID))
                    })
            } else {
                cos <= SAME_WAY && mine.overlap(&theirs) > 0.0
            };
            if held {
                out.push(i);
            }
        }
        k += ROOM_STEP;
    }
    out
}

/// Buses holding each other up for good, a loop of them each waiting for the next (two coming
/// towards each other where they can't pass, even pulled over; each in the other's way across a
/// junction or where one turns round): one of them gives way to another it holds up, once and for
/// all (`Bus::yields`): the one with the most room behind it, then the last by the timetable. It
/// backs up (`Bus::backs`) till it's out of that one's way and a little more (`GIVE_WAY_LOOK`),
/// then stands, and doesn't go on till that one is by (`given_way`): nobody decides again every
/// tick, so they never take turns backing up. Only if the one it gives way to stands still past
/// `IMPATIENT` itself (held up by something else) is it let off. A bus right behind one backing up
/// backs up too. So the first of every such loop gets on, and nobody drives into anyone. (Loops
/// through cars are the claims' to keep from forming: `traffic.rs`, where one giving way waits for
/// the one it gives way to, `movers`.)
fn make_way(w: &mut World) {
    let t = &w.transit;
    let n = t.buses.len();
    let index = |f: u64| t.buses.iter().position(|x| x.first() == f);
    // Those given way to already: till they're by, or come on no further themselves that long
    // (`GIVEN_STEP` at a time, not edging a tick now and then).
    let mut given: Vec<(f32, f32, u32)> = vec![(0.0, 0.0, 0); n];
    let mut gave_up: Vec<Option<u64>> = t.buses.iter().map(|b| b.gave_up).collect();
    let mut yields: Vec<Option<u64>> = (0..n)
        .map(|b| {
            let bus = &t.buses[b];
            let h = bus.yields?;
            let o = index(h)?;
            let at = t.buses[o].progress;
            let (from, mark, still) = bus.given;
            given[b] = if (at - mark).abs() >= GIVEN_STEP {
                (from, at, 0)
            } else {
                (from, mark, still + 1)
            };
            if given[b].2 > IMPATIENT {
                gave_up[b] = Some(h);
            }
            (given[b].2 <= IMPATIENT && !given_way(t, b, o)).then_some(h)
        })
        .collect();
    // Who holds up whom: those driving for others in their way a tick on, or giving way to one.
    let mut holds: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (b, held) in holds.iter_mut().enumerate() {
        let bus = &t.buses[b];
        if bus.until.is_some() || bus.shown.is_none() && bus.progress > 0.0 {
            continue;
        }
        let others = obstacles(w, b);
        let me = driving(t, b, false);
        *held = held_by(t, b, &me, &others, false).collect();
        if let Some(h) = yields[b].and_then(index) {
            held.push(h);
        }
    }
    let reach = |from: usize| {
        let mut seen = vec![false; n];
        let mut todo = holds[from].clone();
        while let Some(k) = todo.pop() {
            if !std::mem::replace(&mut seen[k], true) {
                todo.extend(&holds[k]);
            }
        }
        seen
    };
    let reaches: Vec<Vec<bool>> = (0..n)
        .map(|b| {
            if holds[b].is_empty() {
                vec![false; n]
            } else {
                reach(b)
            }
        })
        .collect();
    // (Out of every loop, it may give way to anyone again.)
    for (b, g) in gave_up.iter_mut().enumerate() {
        if !reaches[b][b] {
            *g = None;
        }
    }
    for b in (0..n).filter(|&b| reaches[b][b]) {
        // (The loop's: those it holds up, one way or another, that hold it up.)
        let looped = |o: usize| reaches[b][o] && reaches[o][b];
        // (One of the loop gives way to another of it already: it's being let by.)
        if (0..n).any(|x| looped(x) && yields[x].and_then(index).is_some_and(looped)) {
            continue;
        }
        // (One can back up where there's ground behind it, or another bus on its way that can
        // back up too, and so on: not one of the loop, nor a car, nor one standing at a stop.)
        // (Not one that gave way to another of the loop that came on no further all the same.)
        let able = (0..n).filter(|&o| {
            looped(o)
                && can_back(w, o, &looped, 8)
                && !gave_up[o].and_then(index).is_some_and(looped)
        });
        let Some(giver) = able.max_by_key(|&o| (room_behind(w, o), t.buses[o].first())) else {
            continue;
        };
        if giver != b {
            continue;
        }
        if let Some(h) = (0..n)
            .filter(|&o| looped(o) && holds[o].contains(&b))
            .min_by_key(|&o| t.buses[o].first())
        {
            yields[b] = Some(t.buses[h].first());
            given[b] = (t.buses[h].progress, t.buses[h].progress, 0);
            // (Not both giving way to each other.)
            if yields[h] == Some(t.buses[b].first()) {
                yields[h] = None;
            }
        }
    }
    // Those giving way back up while they're in the way of the one they give way to, or will be
    // as it comes on a little, or of anyone waiting for them (whom that one may be waiting for in
    // turn), as far as they can.
    let mut backs = vec![false; n];
    for b in 0..n {
        let Some(o) = yields[b].and_then(index) else {
            continue;
        };
        let me = [standing(t, b)];
        let theirs = &t.buses[o];
        let first = t.buses[b].first();
        let in_way = !waits_for(t, o, &driving(t, o, false), &me, false).is_empty()
            || GIVE_WAY_LOOK
                .iter()
                .any(|&k| !clear_at(t, &me, o, theirs.progress + k, theirs.shift))
            || w.cars.iter().any(|c| c.waits_for == Some(first))
            || (0..n).any(|x| x != b && holds[x].contains(&b));
        backs[b] = in_way && can_back(w, b, &|x| x == o, 8);
    }
    // (Right behind one backing up: it backs up too, and gives way to the one that gives way to.)
    let mut todo: Vec<usize> = (0..n).filter(|&b| backs[b]).collect();
    while let Some(b) = todo.pop() {
        let others = obstacles(w, b);
        // (Any following it close: they back up together.)
        let me = cars::Driving {
            behind: 3.0 * cars::AHEAD,
            ..driving(t, b, true)
        };
        for o in held_by(t, b, &me, &others, true) {
            if !backs[o] && t.buses[o].until.is_none() && yields[b] != Some(t.buses[o].first()) {
                backs[o] = true;
                yields[o] = yields[b].filter(|&h| h != t.buses[o].first());
                todo.push(o);
            }
        }
    }
    for (b, bus) in w.transit.buses.iter_mut().enumerate() {
        bus.backs = backs[b];
        bus.yields = yields[b];
        bus.given = given[b];
        bus.gave_up = gave_up[b];
    }
}

/// How far on (ticks along its route) one giving way looks at the one it gives way to: it backs
/// up till it's clear of where that one will be (`make_way`).
const GIVE_WAY_LOOK: [f32; 2] = [10.0, 20.0];

/// Whether bus `b` has given way to bus `o` (`Bus::yields`): that one is far off, or has come
/// `PASSED` on since and neither is in the other's way now.
fn given_way(t: &Transit, b: usize, o: usize) -> bool {
    let (mine, theirs) = (standing(t, b), standing(t, o));
    if dist(mine.at, theirs.at) > BUS_LEN + PULL_AHEAD {
        return true;
    }
    let on = t.buses[o].progress - t.buses[b].given.0;
    let held = |x: usize, y: usize| {
        !waits_for(t, x, &driving(t, x, false), &[standing(t, y)], false).is_empty()
    };
    on >= PASSED && !held(b, o) && !held(o, b)
}

/// How far on one given way to has come (ticks along its route) before the one giving way may go
/// on, the two out of each other's way (`given_way`).
const PASSED: f32 = 40.0;

/// How far on (ticks along its route) one given way to comes at a time, to be coming on at all
/// (`make_way`: no further on in `IMPATIENT`, it's given way to no longer).
const GIVEN_STEP: f32 = 10.0;

/// How far bus `b` could back up (ticks, in steps of five, at most `ROOM_BEHIND`): with more room
/// behind it, it gives way (`make_way`).
fn room_behind(w: &World, b: usize) -> u32 {
    let t = &w.transit;
    let bus = &t.buses[b];
    let all = obstacles(w, b);
    let mut k = 0;
    while k < ROOM_BEHIND
        && bus.progress - (k + 5) as f32 > 0.0
        && clear_at(t, &all, b, bus.progress - (k + 5) as f32, bus.shift)
    {
        k += 5;
    }
    k
}

/// The most room behind a bus `room_behind` looks for (ticks).
const ROOM_BEHIND: u32 = 100;

/// Whether bus `b` can back up a tick: it's in town, it would move into no other, and whatever is
/// right behind it is a bus on its way (not one of `looped`, nor standing at a stop) that can
/// back up too, `depth` deep at most (a car never backs up for a bus).
fn can_back(w: &World, b: usize, looped: &dyn Fn(usize) -> bool, depth: u32) -> bool {
    let t = &w.transit;
    let others = obstacles(w, b);
    let me = cars::Driving {
        behind: 3.0 * cars::AHEAD,
        ..driving(t, b, true)
    };
    let bus = &t.buses[b];
    bus.progress > 0.0
        && clear_at(t, &others, b, (bus.progress - 1.0).max(0.0), bus.shift)
        && cars::behind(&me, &others).all(|k| {
            t.buses
                .iter()
                .position(|x| x.first() == others[k].first)
                .is_some_and(|x| {
                    !looped(x)
                        && t.buses[x].until.is_none()
                        && depth > 0
                        && can_back(w, x, looped, depth - 1)
                })
        })
}

/// Bus of line `l` on its run `run`, where it is at `now` on time; None before it comes into
/// town (`Line::offset`) and once it's gone.
fn on_time(line: &Line, l: u16, run: u64, now: u64) -> Option<Bus> {
    let e = now.checked_sub(run)?;
    // (Ticks since it came into town.)
    let driven = e.checked_sub(line.offset)?;
    let mut bus = Bus {
        line: l,
        run,
        progress: 0.0,
        next: 0,
        stood: 0,
        until: None,
        riders: Vec::new(),
        waited: 0,
        queued: false,
        shift: 0.0,
        backs: false,
        yields: None,
        given: (0.0, 0.0, 0),
        gave_up: None,
        still: 0,
        best: 0.0,
        stuck: 0,
        holds: None,
        blocks: Vec::new(),
        shown: None,
        waits_for: None,
    };
    for j in 0..line.stops.len() {
        if e < line.arrive[j] {
            bus.progress = (driven - j as u64 * DWELL_TICKS) as f32;
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
    bus.progress = (driven - line.stops.len() as u64 * DWELL_TICKS) as f32;
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
    // Into town as the timetable says: each line its `offset` after the run.
    let t = &mut w.transit;
    for l in 0..t.lines.len() {
        let offset = t.lines[l].offset;
        let mut run = t.next_run(l, t.sent.saturating_sub(offset));
        while run + offset <= tick {
            if t.buses.len() < MAX_BUSES
                && let Some(bus) = on_time(&t.lines[l], l as u16, run, tick)
            {
                t.buses.push(bus);
            }
            run = t.next_run(l, run + 1);
        }
    }
    t.sent = tick + 1;
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
    signal(w);
    make_way(w);
    let mut b = 0;
    while b < w.transit.buses.len() {
        if step_bus(w, b) {
            b += 1;
        } else {
            let_go(&mut w.transit, b);
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
        let every = w.transit.stops[h.stop as usize]
            .line
            .map_or(w.transit.every, |(l, _)| w.transit.lines[l as usize].every);
        if gone || tick > due + 2 * every + DWELL_MOST {
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
        .unwrap_or(t.next_run(l as usize, h.run))
        + line.arrive[j]
}

/// Bus `b` a tick on: along its way, at a stop, or out of town (false: gone).
fn step_bus(w: &mut World, b: usize) -> bool {
    let tick = w.tick;
    // Its line's last bus still in town long after it was due in (a big world's): it isn't run,
    // and the next is on time (those waiting for it go another way, `bus_due`).
    let bus = &w.transit.buses[b];
    let line = &w.transit.lines[bus.line as usize];
    if after_its_last(&w.transit, b) && tick > bus.run + line.offset + WAIT_MOST {
        let (l, run) = (bus.line, bus.run);
        next_brings_home(&mut w.transit, l, run);
        w.transit.skipped += 1;
        return false;
    }
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
                // (One after another, a step apart, and a step out from the door: clear of the
                // bus as it pulls away, `cars::go_on`.)
                let back = w.transit.lines[l as usize].bays[j].1;
                let along = [back.sin(), back.cos()];
                let out = [-along[1], along[0]];
                let at = [
                    door[0] - along[0] * 0.5 * k as f32 + out[0] * STEP_OFF,
                    door[1] - along[1] * 0.5 * k as f32 + out[1] * STEP_OFF,
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
        shift_to(w, b, 0.0);
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
    let was = w.transit.buses[b].progress;
    drive_on(w, b);
    w.transit.buses[b].waits_for = if w.transit.buses[b].progress == was {
        held_up_by(w, b)
    } else {
        None
    };
    let asking = w.traffic.waiting(w.transit.buses[b].first());
    count_still(&mut w.transit, b, was, asking);
    // Held up that long, among others holding each other up where none can give way, it's taken
    // out of service: gone, as out of town (its riders are on their way; those it would have
    // brought home come on the next, or walk).
    if w.transit.buses[b].stuck > OUT_OF_SERVICE && !in_view(w, w.transit.now(b).0) {
        withdraw(w, b);
        return false;
    }
    let bus = &mut w.transit.buses[b];
    let line = &w.transit.lines[bus.line as usize];
    if bus.next < n && bus.progress >= line.at[bus.next] {
        let j = bus.next;
        bus.progress = line.at[j];
        bus.stood = tick;
        bus.queued = false;
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
/// for them, and someone standing there a while steps aside (or, with nowhere to step, goes
/// round). Someone waiting for a bus, or stepping up to one, is out of its way. It keeps behind a
/// car or bus in front of it, going its way or across it (`cars::up_against`), and pulls over to
/// pass one coming the other way (`pull_over`), waiting till it's clear of it (`oncoming`); where
/// two hold each other up, one gives way to the other, backing up out of its way and standing
/// till it's by (`make_way`).
fn drive_on(w: &mut World, b: usize) {
    shift_to(w, b, pull_over(w, b));
    let t = &mut w.transit;
    // (At a refuge, the section ahead not its yet: it waits there, `signal`.)
    if section_ahead(t, b).is_some() {
        t.buses[b].queued = true;
        return;
    }
    let bus = &t.buses[b];
    let (backs, yields) = (bus.backs, bus.yields);
    // (Giving way, out of the other's way now: it stands till that one is by.)
    if !backs && let Some(h) = yields {
        match t.buses.iter().position(|x| x.first() == h) {
            Some(o) if !given_way(t, b, o) => {
                t.buses[b].queued = true;
                return;
            }
            _ => t.buses[b].yields = None,
        }
    }
    if waits_to_come_in(t, b) {
        t.buses[b].queued = true;
        return;
    }
    // (Out of town, it comes in, holding the claim at the way in, only with clear ground there.)
    if t.buses[b].progress <= 0.0 && !t.buses[b].backs {
        let all = obstacles(w, b);
        let t = &mut w.transit;
        if !clear_at(t, &all, b, 1.0, 0.0) {
            t.buses[b].queued = true;
            return;
        }
    }
    let t = &mut w.transit;
    // (Into a junction's box only holding its claim to it, with the cars: `traffic`.)
    let (tick, progress) = (w.tick, t.buses[b].progress);
    if !backs
        && !w.traffic.may_go(
            tick,
            t.buses[b].first(),
            0,
            &t.lines[t.buses[b].line as usize].claims,
            progress,
            progress + 1.0,
        )
    {
        t.buses[b].queued = true;
        return;
    }
    let me_first = t.buses[b].first();
    let all = obstacles(w, b);
    // (Not for a car waiting for it, `cars::Car::waits_for`: it never drives into one, `clear_at`.)
    let others: Vec<cars::Driving> = all
        .iter()
        .copied()
        .filter(|o| {
            o.first < 1 << 63
                || w.cars
                    .get((o.first & !(1 << 63)) as usize)
                    .is_none_or(|c| c.waits_for != Some(me_first))
                    && !w.traffic.waits_for(o.first).contains(&me_first)
        })
        .collect();
    let World {
        transit,
        sims,
        lot,
        blocked,
        stairs,
        content,
        objects,
        traffic,
        ..
    } = w;
    // (Where it would be a tick on, as a car: `cars::drive_on`; or backing up.)
    let me = driving(transit, b, backs);
    // (Backing up, not held up by one behind backing up too: they go together.)
    let in_way = if backs {
        cars::behind(&me, &others).any(|k| {
            !transit
                .buses
                .iter()
                .any(|x| x.backs && x.first() == others[k].first)
        })
    } else {
        oncoming(&me, &others).next().is_some() || !ahead_of(transit, b, &others).is_empty()
    };
    let bus = &mut transit.buses[b];
    let line = &transit.lines[bus.line as usize];
    // (Not yet pulled over far enough to pass one coming the other way, or backing up into
    // another: it waits, pulling over.)
    if in_way {
        bus.queued = true;
        return;
    }
    let nav = crate::path::NavGrid {
        lot,
        blocked,
        stairs,
    };
    // (Those bound for the stop it calls at next: anyone else on their way to a stop is in its
    // way as anyone is.)
    let next = line.stops.get(bus.next).copied();
    // (Pulled over onto the pavement, anyone there is in its way.)
    let in_lane = bus.shift < 0.05;
    let aside = |s: &Sim| {
        in_lane
            && ((next.is_some() && s.bound_for_bus == next)
                || matches!(s.current().map(|a| &a.phase), Some(Phase::AtStop { .. })))
    };
    let bodies = (&*content, &objects[..]);
    let mut step = 0.0;
    let (was, was_yaw) = line.place(bus.progress, bus.shift);
    let now = cars::Driving {
        at: was,
        yaw: was_yaw,
        ..me
    };
    // (Waiting for someone on foot only in its body a tick on: turning round, its front sweeps
    // across the pavement, and those beside its way step clear as it comes, out of where it'll be
    // on its way turning, `go_on_by`.)
    let turning = {
        let (_, now) = line.place(bus.progress, bus.shift);
        let far = (bus.progress + BUS_SWEEP_LOOK[BUS_SWEEP_LOOK.len() - 1]).min(line.route.ticks());
        crate::world::wrap_angle(line.place(far, bus.shift).1 - now).abs() > BUS_SWEEP_TURN
    };
    let sweep: Vec<cars::Driving> = if backs || !turning {
        Vec::new()
    } else {
        BUS_SWEEP_LOOK
            .iter()
            .filter(|&&k| bus.progress + k < line.route.ticks())
            .map(|&k| {
                let (at, yaw) = line.place(bus.progress + k, bus.shift);
                cars::Driving {
                    at,
                    yaw,
                    ahead: 0.0,
                    ..me
                }
            })
            .collect()
    };
    let queued = cars::go_on_by(
        sims,
        &nav,
        bodies,
        me,
        now,
        &others,
        aside,
        keeps_across(bus),
        (BUS_WALK_AHEAD, &sweep),
        &mut bus.waited,
        &mut step,
    );
    // Held up that long by someone (where its line turns round, among the others), it edges on:
    // a bus still drives on past `cars::PASS_TICKS`, as it always did, where a car never does.
    if !backs && bus.waited >= cars::PASS_TICKS && !queued {
        step += 1.0;
    }
    let (progress, shift) = (bus.progress, bus.shift);
    let to = if backs {
        (progress - step).max(0.0)
    } else {
        progress + step
    };
    // (Nor into what's left of a claim a car holds: `traffic`.)
    let line = &transit.lines[transit.buses[b].line as usize];
    let body = |(at, yaw): ([f32; 2], f32)| cars::Rect {
        at,
        yaw,
        half: [BUS_LEN / 2.0, BUS_WID / 2.0],
    };
    let (here, there) = (
        body(line.place(progress, shift)),
        body(line.place(to, shift)),
    );
    let pass = (line.room_at(progress) - shift).max(0.0);
    // (Backing up, the way it goes is behind it.)
    let k = if to < progress { -1.0 } else { 1.0 };
    let way = [there.yaw.sin() * k, there.yaw.cos() * k];
    let crossing = to != progress
        && traffic
            .blocked(me_first, &here, &there, way, pass)
            .is_some();
    let moves =
        to != progress && clear_at(transit, &all, b, to, shift) && (to > 0.0 || backs) && !crossing;
    let bus = &mut transit.buses[b];
    // (Backed up out of town, it's out of sight there till it comes in again.)
    bus.queued = queued || to != progress && !moves || to <= 0.0;
    if moves {
        bus.progress = to;
    }
}

/// Whom bus `b`, come no further on this tick, waits for (`Driving::first`), as `drive_on` sees
/// it: one holding blocks of the section ahead, one it gives way to, the claims it asks for, one
/// in its way, one whose claim it would drive into.
fn held_up_by(w: &World, b: usize) -> Option<u64> {
    let t = &w.transit;
    let bus = &t.buses[b];
    if bus.until.is_some() {
        return None;
    }
    if let Some(k) = section_ahead(t, b) {
        let line = &t.lines[bus.line as usize];
        return line.sections[k]
            .cells
            .iter()
            .flat_map(|&(c, _)| t.owners[c as usize].iter().map(|o| o.0))
            .find(|&o| o != bus.first());
    }
    if let Some(h) = bus.yields {
        return Some(h);
    }
    if w.traffic.waiting(bus.first()) {
        return w.traffic.waits_for(bus.first()).first().copied();
    }
    let all = obstacles(w, b);
    let me = driving(t, b, bus.backs);
    if let Some(&k) = waits_for(t, b, &me, &all, bus.backs).first() {
        return Some(all[k].first);
    }
    let line = &t.lines[bus.line as usize];
    let body = |(at, yaw): ([f32; 2], f32)| cars::Rect {
        at,
        yaw,
        half: [BUS_LEN / 2.0, BUS_WID / 2.0],
    };
    let (here, there) = (
        body(line.place(bus.progress, bus.shift)),
        body(line.place(bus.progress + 1.0, bus.shift)),
    );
    let way = [there.yaw.sin(), there.yaw.cos()];
    w.traffic.blocked(bus.first(), &here, &there, way, 0.0)
}

/// The buses in town for the claims (`traffic::begin`): where along their routes they are
/// (`places`), and their bodies, the way they go, and on the move where they'll be along their way
/// (`movers`).
pub(crate) fn movers(
    w: &World,
    movers: &mut Vec<crate::traffic::Mover>,
    places: &mut Vec<crate::traffic::Place>,
) {
    let t = &w.transit;
    let half = [BUS_LEN / 2.0, BUS_WID / 2.0];
    for (b, bus) in t.buses.iter().enumerate() {
        let key = bus.first();
        let line = &t.lines[bus.line as usize];
        places.push((key, 0, bus.progress));
        if !t.in_town(b) {
            continue;
        }
        let rect = |(at, yaw): ([f32; 2], f32)| cars::Rect { at, yaw, half };
        let body = rect(line.place(bus.progress, bus.shift));
        let moving = t.on_the_move(b);
        let ahead = if moving {
            crate::traffic::LOOK
                .iter()
                .filter(|&&dt| bus.progress + dt < line.route.ticks())
                .map(|&dt| rect(line.place(bus.progress + dt, bus.shift)))
                .collect()
        } else {
            Vec::new()
        };
        let k = if bus.backs { -1.0 } else { 1.0 };
        movers.push(crate::traffic::Mover {
            key,
            way: [body.yaw.sin() * k, body.yaw.cos() * k],
            // (Stood at a stop it doesn't pull over.)
            pass: if bus.until.is_some() {
                0.0
            } else {
                (line.room_at(bus.progress) - bus.shift).max(0.0)
            },
            rect: body,
            moving,
            ahead,
            parked: false,
            // (Giving way to one, it waits for that one, backing up or not: `make_way`.)
            waits: bus.yields.or(bus.waits_for),
        });
    }
}

/// Whether `bus` keeps its distance (`cars::AHEAD`) from one going across its way in front of it
/// (`cars::up_against`): not once held up twice `IMPATIENT`, when only its body keeps clear of the
/// others' (`clear_at`; held up for good by one, they back up: `make_way`).
fn keeps_across(bus: &Bus) -> bool {
    bus.still <= 2 * IMPATIENT
}

/// Whether `at` is in or near the part of town in view (`World::view`): a bus held up for good is
/// taken out of service only out of sight (with nothing in view, any time).
fn in_view(w: &World, at: [f32; 2]) -> bool {
    let Some([x0, z0, x1, z1]) = w.view else {
        return false;
    };
    let m = VIEW_REACH;
    at[0] >= x0 as f32 - m
        && at[0] <= x1 as f32 + m
        && at[1] >= z0 as f32 - m
        && at[1] <= z1 as f32 + m
}

/// Bus `b` taken out of service (`OUT_OF_SERVICE`): those on it to work are on their way (their
/// way out was timed as they got on: `out_in`); those it was to bring home come on the next.
fn withdraw(w: &mut World, b: usize) {
    let t = &mut w.transit;
    let (line, run) = (t.buses[b].line, t.buses[b].run);
    next_brings_home(t, line, run);
    t.withdrawn += 1;
}

/// Those line `line`'s run `run` was to bring home come on the next instead (`homeward`).
fn next_brings_home(t: &mut Transit, line: u16, run: u64) {
    for h in &mut t.homeward {
        if t.stops[h.stop as usize]
            .line
            .is_some_and(|(l, _)| l == line)
            && h.run <= run
        {
            h.run = run + 1;
        }
    }
}

/// Bus `b` a tick on its way: how long it has stood still (`Bus::still`), and how long it has
/// come no further on (`Bus::stuck`).
fn count_still(t: &mut Transit, b: usize, was: f32, asking: bool) {
    // (Waiting at a refuge for the section ahead isn't held up: the one in it goes on, or is
    // taken out of service itself. Nor is giving way to one on its way: `make_way`.)
    let gives_way = t.buses[b].yields.is_some() && t.buses[b].given.2 <= IMPATIENT;
    let waits = section_ahead(t, b).is_some() || after_its_last(t, b) || asking || gives_way;
    let bus = &mut t.buses[b];
    let held = bus.until.is_none() && !waits;
    bus.still = if bus.progress == was && held {
        bus.still.saturating_add(1)
    } else {
        0
    };
    if bus.progress > bus.best || !held {
        bus.best = bus.best.max(bus.progress);
        bus.stuck = 0;
    } else {
        bus.stuck = bus.stuck.saturating_add(1);
    }
}

/// Whether bus `b`, out of town at its way in (not yet come in, or backed up out again), waits
/// there: another out of town, an earlier run, would come in where it would (out of sight, no
/// other keeps clear of them: they come in one at a time; not one waiting there for its line's
/// last bus to be gone); in a big world, its line's last bus is still in town (`after_its_last`).
fn waits_to_come_in(t: &Transit, b: usize) -> bool {
    let bus = &t.buses[b];
    if bus.progress > 0.0 {
        return false;
    }
    if after_its_last(t, b) {
        return true;
    }
    let body = |(at, yaw): ([f32; 2], f32)| cars::Rect {
        at,
        yaw,
        half: [BUS_LEN / 2.0, BUS_WID / 2.0],
    };
    let mine = body(t.lines[bus.line as usize].place(1.0, 0.0));
    t.buses.iter().enumerate().any(|(o, other)| {
        o != b
            && other.progress <= 0.0
            && other.first() < bus.first()
            && !after_its_last(t, o)
            && body(t.now(o)).overlap(&mine) > 0.0
    })
}

/// Whether bus `b`, in a big world out of town at its way in, waits for its line's last bus (an
/// earlier run) to be out of town: one of a line on the road at a time (`round_trip`). It isn't
/// held up (`count_still`): that one goes on, or is taken out of service.
fn after_its_last(t: &Transit, b: usize) -> bool {
    let bus = &t.buses[b];
    t.walk_stops
        && bus.progress <= 0.0
        && t.buses
            .iter()
            .any(|o| o.line == bus.line && o.run < bus.run && o.progress > 0.0)
}

/// Whether bus `b` may move to `t` ticks along its route pulled over `shift` m: its body there
/// overlaps no other bus's, nor a car's (`cars::vehicles`), more than it does now. Whatever else it
/// waits for, it never drives, backs or pulls over into another bus.
fn clear_at(tr: &Transit, others: &[cars::Driving], b: usize, t: f32, shift: f32) -> bool {
    let bus = &tr.buses[b];
    let line = &tr.lines[bus.line as usize];
    let body = |(at, yaw): ([f32; 2], f32)| cars::Rect {
        at,
        yaw,
        half: [BUS_LEN / 2.0, BUS_WID / 2.0],
    };
    let (now, then) = (
        body(line.place(bus.progress, bus.shift)),
        body(line.place(t, shift)),
    );
    let me = standing(tr, b);
    others.iter().all(|o| {
        // (A car backing up, or tangled with it already, it doesn't wait for: `Driving::free`;
        // but coming into town it needs clear ground, whatever stands at the way in.)
        if dist(o.at, then.at) > BUS_LEN + o.len || bus.progress > 0.0 && !o.free(&me) {
            return true;
        }
        let theirs = cars::Rect {
            at: o.at,
            yaw: o.yaw,
            half: [o.len / 2.0, o.wid / 2.0],
        };
        let after = then.overlap(&theirs);
        // (Coming into town, it needs clear ground.)
        after <= 0.0 || bus.progress > 0.0 && after <= now.overlap(&theirs)
    })
}

/// Bus `b` a tick nearer pulled over `want` m right of its lane (`PULL_RATE`): back towards its
/// lane only where that's clear of those coming the other way (`too_near`; pulling over, it moves
/// away from them).
fn shift_to(w: &mut World, b: usize, want: f32) {
    let others = obstacles(w, b);
    let t = &w.transit;
    let bus = &t.buses[b];
    let shift = bus.shift + (want - bus.shift).clamp(-PULL_RATE, PULL_RATE);
    if shift < bus.shift {
        let (at, yaw) = t.lines[bus.line as usize].place(bus.progress, shift);
        if others.iter().any(|o| {
            o.free(&standing(t, b))
                && too_near((at, yaw, BUS_LEN, BUS_WID), (o.at, o.yaw, o.len, o.wid))
        }) {
            return;
        }
    }
    if clear_at(t, &others, b, bus.progress, shift) {
        w.transit.buses[b].shift = shift;
    }
}

/// How far right of its lane bus `b` would be pulled over (m) to pass those coming the other way
/// in front of it (`PULL_AHEAD`) where the road is too narrow for both in their lanes: by the
/// road's edge at most (`Line::room`), over the pavement. Half the way past another bus on the
/// move (it pulls over the other half), all the way past a car or a bus standing at a stop.
fn pull_over(w: &World, b: usize) -> f32 {
    let t = &w.transit;
    let bus = &t.buses[b];
    let line = &t.lines[bus.line as usize];
    let room = line.room_at(bus.progress);
    if room <= 0.0 {
        return 0.0;
    }
    let (at, yaw) = line.place(bus.progress, 0.0);
    // How far over it would be, of how far the two are short of passing (`passing`, both in their
    // lanes) for one at `p` facing `facing`, `len` by `wid`, coming the other way in front of it.
    let short = |p: [f32; 2], facing: f32, len: f32, wid: f32| {
        let Some(m) = passing((at, yaw, BUS_LEN, BUS_WID), (p, facing, len, wid)) else {
            return (0.0, 1.0, 1.0);
        };
        let ends = (BUS_LEN + len) / 2.0;
        if m.along < -ends || m.along > ends + PULL_AHEAD || m.apart < 0.0 {
            return (0.0, 1.0, 1.0);
        }
        ((PASSING + PULL_MARGIN - m.gap).max(0.0), m.mine, m.theirs)
    };
    let mut want: f32 = 0.0;
    for car in w.cars.iter().filter(|c| !c.backing()) {
        if let Some((p, facing, true, _)) = car.shown {
            let (all, mine, _) = short(p, facing, car.len, car.wid);
            want = want.max(all / mine);
        }
    }
    for (o, other) in t.buses.iter().enumerate() {
        if o == b || other.shown.is_none() {
            continue;
        }
        let theirs = &t.lines[other.line as usize];
        let (p, facing) = theirs.place(other.progress, 0.0);
        let (all, mine, their) = short(p, facing, BUS_LEN, BUS_WID);
        // (The other half theirs, as far as they can.)
        let share = if other.until.is_none() {
            (all / 2.0).max(all - theirs.room_at(other.progress) * their)
        } else {
            all
        };
        want = want.max(share / mine);
    }
    want.min(room)
}

/// Two coming towards each other, measured along and across the way between theirs (the same
/// for both: on a bend either may look to be off the other's right): how far the second is along
/// it from the first; how far apart their middles are across it (the second on the first's left:
/// more), and the gap there is between their bodies (m: less than none, they may overlap); and
/// how far across it each moves pulling over a metre (`pull_over`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Passing {
    pub along: f32,
    pub apart: f32,
    pub gap: f32,
    pub mine: f32,
    pub theirs: f32,
}

/// `Passing` for `a` and `b` (each where, facing which way, how long and wide), if they're coming
/// towards each other (else none).
pub(crate) fn passing(
    a: ([f32; 2], f32, f32, f32),
    b: ([f32; 2], f32, f32, f32),
) -> Option<Passing> {
    let (ahead, theirs) = ([a.1.sin(), a.1.cos()], [b.1.sin(), b.1.cos()]);
    if ahead[0] * theirs[0] + ahead[1] * theirs[1] >= -SAME_WAY {
        return None;
    }
    let u = [ahead[0] - theirs[0], ahead[1] - theirs[1]];
    let n = u[0].hypot(u[1]).max(1e-6);
    let (u, left) = ([u[0] / n, u[1] / n], [u[1] / n, -u[0] / n]);
    let d = [b.0[0] - a.0[0], b.0[1] - a.0[1]];
    let (along, apart) = (d[0] * u[0] + d[1] * u[1], d[0] * left[0] + d[1] * left[1]);
    // (How far each reaches across that way, turned as it is; and moves, pulling over to its
    // right.)
    let reach = |way: [f32; 2], len: f32, wid: f32| {
        let k = way[0] * left[0] + way[1] * left[1];
        len / 2.0 * k.abs() + wid / 2.0 * (1.0 - k * k).max(0.0).sqrt()
    };
    let pulls = |way: [f32; 2]| (way[0] * u[0] + way[1] * u[1]).abs().max(0.3);
    Some(Passing {
        along,
        apart,
        gap: apart.abs() - reach(ahead, a.2, a.3) - reach(theirs, b.2, b.3),
        mine: pulls(ahead),
        theirs: pulls(theirs),
    })
}

/// Whether `a` (where it would be a tick on) would come too near `b` coming the other way: its
/// body, half `PASSING` wider either side, on the other's. (Pulled over as far as `passing` says,
/// they're further apart than that: it measures across a way between theirs.)
pub(crate) fn too_near(a: ([f32; 2], f32, f32, f32), b: ([f32; 2], f32, f32, f32)) -> bool {
    let (ahead, theirs) = ([a.1.sin(), a.1.cos()], [b.1.sin(), b.1.cos()]);
    let mine = cars::Rect {
        at: a.0,
        yaw: a.1,
        half: [a.2 / 2.0, a.3 / 2.0 + PASSING / 2.0],
    };
    let other = cars::Rect {
        at: b.0,
        yaw: b.1,
        half: [b.2 / 2.0, b.3 / 2.0],
    };
    ahead[0] * theirs[0] + ahead[1] * theirs[1] < -SAME_WAY && mine.overlap(&other) > 0.0
}

/// Those of `them` (indices) coming the other way that bus `me` (where it would be a tick on) would
/// come too near (`too_near`): it waits for them, pulling over to pass them (`pull_over`).
fn oncoming<'a>(
    me: &'a cars::Driving,
    them: &'a [cars::Driving],
) -> impl Iterator<Item = usize> + 'a {
    let mine = (me.at, me.yaw, me.len, me.wid);
    them.iter()
        .enumerate()
        .filter(move |(_, o)| o.free(me) && too_near(mine, (o.at, o.yaw, o.len, o.wid)))
        .map(|(k, _)| k)
}

/// The bus driving by that a step from `from` to `to` with `body` would walk into (stepping out
/// of its way is fine): where it is and which way it faces. Held up by someone in its way, it's
/// still driving by, as a car is. (Stood at a stop it isn't: a stop where the bus turns round
/// can stand on the way along the road, and a bus stood there after another would shut in
/// whoever is on their way past.)
pub(crate) fn in_the_way(
    t: &Transit,
    body: cars::Body,
    from: [f32; 2],
    to: [f32; 2],
) -> Option<([f32; 2], f32)> {
    t.buses.iter().find_map(|b| {
        let (at, yaw, ..) = b.shown.filter(|_| b.until.is_none())?;
        cars::walks_into(at, yaw, BUS_LEN, BUS_WID, body, from, to).then_some((at, yaw))
    })
}

/// How far two buses' bodies overlap (m; 0 or less: apart), each at its middle facing its yaw.
pub fn overlap(a: ([f32; 2], f32), b: ([f32; 2], f32)) -> f32 {
    let body = |(at, yaw): ([f32; 2], f32)| cars::Rect {
        at,
        yaw,
        half: [BUS_LEN / 2.0, BUS_WID / 2.0],
    };
    body(a).overlap(&body(b))
}

/// Where every bus is and which doors are open (`Bus::shown`).
fn poses(w: &mut World) {
    let tick = w.tick;
    let t = &mut w.transit;
    for bus in &mut t.buses {
        let line = &t.lines[bus.line as usize];
        let (at, yaw) = line.place(bus.progress, bus.shift);
        let doors = if bus.doors_open(tick) { 0b11 } else { 0 };
        let moving = bus.until.is_none() && bus.waited == 0 && !bus.queued;
        // (Waiting to come into town, it's not in sight yet.)
        bus.shown = (bus.progress > 0.0 || !bus.queued).then_some((at, yaw, moving, doors));
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
/// once they're there, within `WAIT_MOST` and in time for work; else they leave town at the stop
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

/// Whether a bus is due at `stop` for someone waiting there at `tick`: one of its line on its
/// way there (late or not, come into town or not yet), or standing there with time to get on,
/// or the next to come within `WAIT_MOST` (`Transit::catches`). With none, they make their own
/// way (`give_up`) rather than wait for one a headway off.
pub(crate) fn bus_due(t: &Transit, stop: u16, tick: u64) -> bool {
    let Some((l, j)) = t.stops.get(stop as usize).and_then(|s| s.line) else {
        return false;
    };
    let j = j as usize;
    t.buses.iter().any(|b| {
        b.line == l
            && (b.next < j || b.next == j && b.until.is_none_or(|until| tick + DOOR_TICKS < until))
    }) || t.catches(stop, tick).is_some()
}

/// Waited at the stop far too long (`GIVE_UP_TICKS`: no bus came), or with no bus due
/// (`bus_due`): they go another way.
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
    // (The next to call there, within `WAIT_MOST`: one in town already, or to come.)
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
        let Some(run) = out_by
            .checked_sub(line.end)
            .and_then(|r| t.run_before(l as usize, r))
        else {
            continue;
        };
        // (Not the night before's last: the first buses of the day are too late.)
        if out_by - (run + line.end) > line.every {
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

/// The ticks until the bus at `stop` (its doors open) is out of town, as its timetable has it.
pub(crate) fn out_in(w: &World, stop: u16) -> Option<u64> {
    let t = &w.transit;
    let bus = &t.buses[t.boarding(stop, w.tick)?];
    let out = bus.run + t.lines.get(bus.line as usize)?.end;
    Some(out.saturating_sub(w.tick))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A town 200 m long, a `width` m street along x with a way out at each end and a bus stop in
    /// the middle.
    fn street(width: f32) -> World {
        let content = r#"{"needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],
            "objects":[],"bus":{"every":60,"first":6,"last":23},"economy":{"startingFunds":500}}"#;
        let road: Vec<String> = (0..=199).map(|x| format!("[{x}.5,30]")).collect();
        let town = format!(
            r#"{{"width":200,"depth":60,
            "exits":[{{"x":0.5,"z":30,"kind":"road"}},{{"x":199.5,"z":30,"kind":"road"}},{{"x":100,"z":{},"kind":"stop"}}],
            "roads":[{{"points":[{}],"width":{width},"kind":"street","ends":["out","out"]}}],
            "plots":[],"households":[],"objects":[],"sims":[]}}"#,
            30.0 + width / 2.0 - 0.6,
            road.join(",")
        );
        World::from_json(content, &town, 1).unwrap()
    }

    /// A bus pulled right over to pass another on a 7 m street, or a car on a 6 m lane, stays in
    /// the strip the kerbside furniture leaves it (`kerbside`, the renderer's `busStreets`), and
    /// the street is listed either way (a 6 m lane for the cars, `cars::LANE_KERBSIDE`); a 6 m
    /// street is too narrow for two buses either way: one waits there (`single_track`).
    #[test]
    fn a_bus_pulled_over_stays_in_the_strip_the_kerbside_furniture_leaves() {
        for width in [6.0f32, 7.0] {
            let w = street(width);
            let line = &w.transit.lines[0];
            let keep = kerbside(width);
            let mut most: f32 = 0.0;
            let mut t = 0.0;
            while t < line.route.ticks() {
                let (p, _) = line.place(t, line.room_at(t));
                // (On the map: past either end, it's out of town.)
                if (2.0..198.0).contains(&p[0]) {
                    most = most.max((p[1] - 30.0).abs() + BUS_WID / 2.0);
                }
                t += 1.0;
            }
            assert!(
                most <= width / 2.0 - keep + 1e-3,
                "on a {width} m street a bus pulled over reaches {most:.2} m from the middle, the furniture {:.2}",
                width / 2.0 - keep
            );
            let listed = w.transit.bus_streets.iter().find(|&&(r, _)| r == 0);
            let want = if width >= 7.0 {
                keep
            } else {
                cars::LANE_KERBSIDE
            };
            assert_eq!(
                listed.map(|l| l.1),
                Some(want),
                "{width} m: {:?}",
                w.transit.bus_streets
            );
            // (Wide enough to pass, pulled over; or not even that.)
            let passes = line.reach_at(line.route.ticks() / 2.0) >= BUS_WID / 2.0 + PASSING / 2.0;
            assert_eq!(passes, width >= 7.0, "{width} m");
        }
    }

    /// Walkers and riders beside a country road pass a bus pulled in at a stop by its edge with
    /// room to spare (they walked into its side, and at a stop where buses turn round a rider
    /// was held there among them all evening).
    #[test]
    fn the_way_beside_a_country_road_clears_a_bus_at_a_stop() {
        for width in [5.0, 6.0, 8.0] {
            let bus = bay_out(kerb_of(width, RoadKind::Country)) + BUS_WID / 2.0;
            // (A city bike's handlebars, content `vehicle.reach`.)
            let rider = crate::travel::side_offset(width, RoadKind::Country) - 0.3;
            assert!(
                rider - bus >= cars::KEEP,
                "on a {width} m road the bike is {:.2} m from the bus",
                rider - bus
            );
        }
    }
}
