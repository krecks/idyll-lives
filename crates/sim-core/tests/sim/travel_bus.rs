//! The town's bus (`bus.rs`, content `bus`): a cosy town's stops are ways out of town, the bus
//! keeps its timetable, someone whose quicker way is the bus walks to the stop, waits there for
//! it, gets on and is at work on time, and comes home on it and walks from the stop; walking stays
//! walking where that's quicker, and a bike at home is ridden; a save mid-wait or on the bus home
//! loads sensibly, and a save without stops has no bus. A bus never drives into another: one
//! coming to a stop where another stands waits behind it, and two lines coming in the same way
//! are sent apart.

use crate::common;

use sim_core::world::{Phase, TaskKind};
use sim_core::{TICKS_PER_MINUTE, World, clock, snapshot};

/// The tests' bus: every this many minutes (their own timetable, not the game's: the times below
/// are worked out for it).
const EVERY: u64 = 20;

/// A clerk's job starting at `start` o'clock for four hours, every day; buses every `EVERY` minutes
/// from 6 to 23 o'clock.
fn content(start: f32) -> String {
    content_every(start, EVERY)
}

/// `content` with a bus every `every` minutes.
fn content_every(start: f32, every: u64) -> String {
    format!(
        r#"{{
    "needs":[{{"id":"hunger","label":"Hunger","decayPerHour":0.01}}],
    "objects":[
        {{"id":"chair","name":"Chair","interactions":[{{"id":"sit","label":"Sit","minutes":30,"effects":{{"hunger":0.01}},"tags":["lounge"]}}]}},
        {{"id":"bike","name":"Bike","footprint":[1,1],"price":200,"interactions":[],"vehicle":{{"kind":"bike","speed":3,"minRide":30}}}}],
    "careers":[{{"id":"office","label":"Office","levels":[
        {{"title":"Clerk","pay":100,"start":{start},"hours":4,"days":[0,1,2,3,4,5,6]}}]}}],
    "careerRules":{{"commuteMinutes":30,"lateGraceMinutes":0,"performancePerShift":10}},
    "bus":{{"every":{every},"first":6,"last":23}},
    "economy":{{"startingFunds":500}}}}"#
    )
}

/// The main street's middle (z), and the stop's x.
const STREET: f32 = 30.0;
const STOP_X: f32 = 100.0;

/// A cosy town 200 m long: a street along x with a way out at either end and (`stop`) a bus stop
/// in the middle on its south side (the bus going east past it), the house of `workers` with a
/// job at `house_x` (south of the street) and `bikes` bikes in its garden.
fn town(workers: usize, bikes: usize, house_x: i32, stop: bool) -> String {
    let sims: Vec<String> = (0..workers)
        .map(|k| {
            format!(
                r#"{{"name":"W{k}","household":0,"job":{{"career":"office"}},"x":{}.5,"z":44.5}}"#,
                house_x + 4 + k as i32
            )
        })
        .collect();
    let mut objects = vec![format!(
        r#"{{"def":"chair","x":{},"z":45,"rot":0}}"#,
        house_x + 2
    )];
    for b in 0..bikes {
        objects.push(format!(
            r#"{{"def":"bike","x":{},"z":48,"rot":0}}"#,
            house_x + 12 + b as i32 * 2
        ));
    }
    let road: Vec<String> = (0..=199).map(|x| format!("[{x}.5,{STREET}]")).collect();
    let stop = if stop {
        format!(r#",{{"x":{STOP_X},"z":{},"kind":"stop"}}"#, STREET + 2.4)
    } else {
        String::new()
    };
    format!(
        r#"{{"width":200,"depth":60,
        "exits":[{{"x":0.5,"z":{STREET},"kind":"road"}},{{"x":199.5,"z":{STREET},"kind":"road"}}{stop}],
        "roads":[{{"points":[{road}],"width":6,"kind":"street","ends":["out","out"]}}],
        "plots":[{{"name":"Home","x":{house_x},"z":38,"w":20,"d":16,"entry":[{e}.5,38.5]}}],
        "households":[{{"name":"Riders","plot":0,"player":true}}],
        "objects":[{objects}],
        "sims":[{sims}]}}"#,
        road = road.join(","),
        e = house_x + 10,
        objects = objects.join(","),
        sims = sims.join(","),
    )
}

fn world(content: &str, town: &str) -> World {
    let mut w = World::from_json(content, town, 5).unwrap();
    w.autonomy = false;
    w
}

fn waiting(w: &World, i: usize) -> bool {
    w.sims[i]
        .current()
        .is_some_and(|a| matches!(a.phase, Phase::AtStop { .. }))
}

/// The tick work starts today for resident `i`.
fn start(w: &World, i: usize) -> u64 {
    let s = &w.sims[i];
    sim_core::life::shift_today(&w.content, s.job.as_ref().unwrap(), &s.skills, w.tick)
        .unwrap()
        .1
}

/// What one morning to work looks like for resident 0: when they set off, got to the stop, got on
/// (away), and whether they sat waiting and saw the bus in with its doors open.
#[derive(Debug, Default)]
struct Morning {
    set_off: Option<u64>,
    /// Their way to work as they set off.
    commute: Option<sim_core::life::Commute>,
    at_stop: Option<u64>,
    on: Option<u64>,
    sat: bool,
    doors_open: bool,
}

fn morning(w: &mut World, until: u64) -> Morning {
    let mut m = Morning::default();
    let mut out = vec![0.0; snapshot::CAPACITY];
    while w.tick < until && m.on.is_none() {
        w.tick_once();
        let s = &w.sims[0];
        if m.set_off.is_none()
            && s.current()
                .is_some_and(|a| matches!(a.task.kind, TaskKind::Work))
        {
            m.set_off = Some(w.tick);
            m.commute = s.commute();
        }
        if waiting(w, 0) {
            m.at_stop.get_or_insert(w.tick);
            m.sat |= s.pose == sim_core::content::Pose::Sit;
        }
        if m.at_stop.is_some() && !m.doors_open {
            snapshot::write(w, &mut out);
            let n = out[snapshot::header::BUS_COUNT] as usize;
            m.doors_open = (0..n).any(|k| {
                let o = snapshot::BUS_BASE + k * snapshot::BUS_STRIDE;
                out[o + snapshot::bus::DOORS] > 0.0
                    && (out[o + snapshot::bus::X] - STOP_X).abs() < 6.0
            });
        }
        if s.away_until.is_some() {
            m.on = Some(w.tick);
        }
    }
    m
}

#[test]
fn a_cosy_towns_stop_is_a_way_out() {
    let c = content(10.0);
    let w = world(&c, &town(1, 0, 90, true));
    // The stop: a way out the bus calls at, with a shelter behind the pavement.
    assert_eq!(w.transit.stops.len(), 1);
    let stop = &w.transit.stops[0];
    assert_eq!(stop.exit, 2);
    assert!(stop.line.is_some(), "no bus calls at it");
    assert!(stop.shelter, "no room for its shelter");
    assert_eq!(w.transit.lines.len(), 1);
    // While the bus runs it's the bus's; at night, a way out on foot.
    let at = |h: f32| clock::tick_at(2, h * 60.0).unwrap();
    assert!(!w.transit.walks_out(2, at(9.0)));
    assert!(w.transit.walks_out(2, at(2.0)));
    assert!(w.transit.walks_out(0, at(9.0)), "a road out is always one");
}

#[test]
fn at_night_they_leave_town_by_the_stop_on_foot() {
    // Work at 5 o'clock: no bus yet, and the stop is the nearest way out.
    let c = content(5.0);
    let mut w = world(&c, &town(1, 0, 90, true));
    common::run_to(&mut w, 1, 23.5);
    let end = clock::tick_at(2, 6.0 * 60.0).unwrap();
    while w.tick < end && w.sims[0].away_until.is_none() {
        w.tick_once();
        assert!(w.bound_for_bus(0).is_none(), "waited for a bus at night");
    }
    assert!(w.sims[0].away_until.is_some(), "never left for work");
    let pos = w.sims[0].pos;
    assert!(
        (pos[0] - STOP_X).abs() < 1.0 && (pos[1] - (STREET + 2.4)).abs() < 1.0,
        "left from {pos:?}, not the stop"
    );
}

#[test]
fn the_bus_keeps_its_timetable() {
    let c = content(10.0);
    let mut w = world(&c, &town(0, 0, 90, true));
    common::run_to(&mut w, 1, 23.9);
    let line = w.transit.lines[0].clone();
    let every = EVERY * TICKS_PER_MINUTE;
    // When each bus came in, and when its doors opened at the stop.
    let mut came: Vec<u64> = Vec::new();
    let mut opened: Vec<(u64, u64)> = Vec::new();
    let mut most = 0;
    let end = clock::tick_at(3, 0.5 * 60.0).unwrap();
    while w.tick < end {
        w.tick_once();
        most = most.max(w.transit.buses.len());
        for b in &w.transit.buses {
            if !came.contains(&b.run) {
                came.push(b.run);
            }
            if b.shown.is_some_and(|s| s.3 != 0) && !opened.iter().any(|o| o.0 == b.run) {
                opened.push((b.run, w.tick));
            }
        }
    }
    // Every `EVERY` minutes from 6 to 23 o'clock, nothing at night.
    let first = clock::tick_at(2, 6.0 * 60.0).unwrap();
    let last = clock::tick_at(2, 23.0 * 60.0).unwrap();
    let expected: Vec<u64> = (0..)
        .map(|k| first + k * every)
        .take_while(|&t| t <= last)
        .collect();
    assert_eq!(came, expected, "the buses that came in");
    assert_eq!(came.len() as u64, 17 * 60 / EVERY + 1);
    // At the stop when the timetable says, its doors open a moment after.
    assert_eq!(opened.len(), came.len());
    for (run, at) in opened {
        let due = run + line.arrive[0];
        assert!(
            at >= due && at <= due + 20,
            "the {:.2} bus opened its doors at {:.2}, due {:.2}",
            clock::hour(run),
            clock::hour(at),
            clock::hour(due)
        );
    }
    assert!(most <= 4, "{most} buses in town at once");
}

#[test]
fn a_rider_waits_less_than_a_headway_gets_on_and_is_at_work_on_time() {
    let c = content(10.0);
    let mut w = world(&c, &town(1, 0, 90, true));
    // A day in, so their leaving is timed by the bus (the game starts at 8).
    common::run_to(&mut w, 2, 5.0);
    let m = morning(&mut w, clock::tick_at(2, 12.0 * 60.0).unwrap());
    let c0 = m.commute.expect("a commute");
    assert!(c0.bus.is_some(), "not planned by bus: {c0:?}");
    let (Some(set_off), Some(at_stop), Some(on)) = (m.set_off, m.at_stop, m.on) else {
        panic!("never got on: {m:?}");
    };
    let work = start(&w, 0);
    // They set off as planned, waited less than one bus's headway, sat on the bench.
    assert!(
        set_off.abs_diff(work - c0.ticks) <= TICKS_PER_MINUTE,
        "set off at {:.2}",
        clock::hour(set_off)
    );
    assert!(
        on - at_stop < EVERY * TICKS_PER_MINUTE,
        "waited {} min",
        (on - at_stop) / TICKS_PER_MINUTE
    );
    assert!(m.sat, "stood all the while with the bench free");
    assert!(m.doors_open, "the bus's doors never opened");
    // On before work starts (with the way out and on to work still to go), and not late.
    assert!(on < work, "got on at {:.2}", clock::hour(on));
    assert!(!w.sims[0].job.as_ref().unwrap().late, "late for work");
    assert_eq!(sim_core::bus::status(&w, &w.sims[0]), Some("work"));
    // And quicker than the walk: out of town by bus before they'd have walked there.
    let walk = w
        .travel
        .eta([94.5, 44.5], w.exits[0], sim_core::travel::Mode::Foot, 1.5);
    assert!(((work - set_off) as f32) < (walk + 30.0) * TICKS_PER_MINUTE as f32);
}

#[test]
fn they_come_home_by_bus_and_walk_from_the_stop() {
    // (Work over at 14:15, the bus last at the stop at 14:08: they wait off the map for the next.)
    let c = content(10.25);
    let mut w = world(&c, &town(1, 0, 90, true));
    common::run_to(&mut w, 2, 5.0);
    morning(&mut w, clock::tick_at(2, 12.0 * 60.0).unwrap())
        .on
        .expect("never got on");
    let funds = w.households[0].funds;
    // Done: off the map until the bus home pulls in at their stop.
    common::run_to(&mut w, 2, 14.26);
    assert_eq!(sim_core::bus::status(&w, &w.sims[0]), Some("home"));
    let mut off = None;
    let end = clock::tick_at(2, 16.0 * 60.0).unwrap();
    while w.tick < end && off.is_none() {
        w.tick_once();
        if w.sims[0].away_until.is_none() {
            off = Some(w.tick);
        }
    }
    let off = off.expect("never came home");
    assert!(off - clock::tick_at(2, 14.25 * 60.0).unwrap() < EVERY * TICKS_PER_MINUTE);
    // Off the bus standing at the stop, by its middle door, and paid once.
    let bus = w
        .transit
        .buses
        .iter()
        .find(|b| b.shown.is_some_and(|s| !s.2 && s.3 != 0))
        .expect("no bus at the stop with its doors open");
    let (at, ..) = bus.shown.unwrap();
    let pos = w.sims[0].pos;
    assert!(
        (pos[0] - at[0]).abs() < 2.0 && pos[1] > at[1],
        "got off at {pos:?}, the bus at {at:?}"
    );
    assert_eq!(w.households[0].funds, funds + 100);
    // Then home on foot.
    let home = common::until(&mut w, 60, |w| {
        let (x, z) = w.sims[0].tile();
        w.plot_at(x, z) == Some(0)
    });
    assert!(home, "never walked home from the stop");
}

#[test]
fn nobody_takes_the_bus_when_walking_is_quicker() {
    // The house by the way out at the west end.
    let c = content(10.0);
    let mut w = world(&c, &town(1, 0, 4, true));
    common::run_to(&mut w, 2, 5.0);
    assert!(w.sims[0].commute().is_some_and(|c| c.bus.is_none()));
    let end = clock::tick_at(2, 12.0 * 60.0).unwrap();
    while w.tick < end && w.sims[0].away_until.is_none() {
        w.tick_once();
        assert!(w.bound_for_bus(0).is_none(), "went for the bus");
    }
    assert!(w.sims[0].away_until.is_some(), "never got to work");
    assert!(w.sims[0].pos[0] < 2.0, "left by {:?}", w.sims[0].pos);
}

#[test]
fn someone_with_a_bike_at_home_rides_it_instead() {
    let c = content(10.0);
    let mut w = world(&c, &town(1, 1, 90, true));
    common::run_to(&mut w, 2, 5.0);
    let mut rode = false;
    let end = clock::tick_at(2, 12.0 * 60.0).unwrap();
    while w.tick < end && w.sims[0].away_until.is_none() {
        w.tick_once();
        assert!(w.bound_for_bus(0).is_none(), "went for the bus");
        rode |= w.sims[0].riding.is_some();
    }
    assert!(rode, "never rode the bike");
    assert!(w.sims[0].away_until.is_some(), "never got to work");
}

#[test]
fn a_game_saved_at_the_stop_loads_with_them_waiting_for_the_next_bus() {
    let c = content(10.0);
    let mut w = world(&c, &town(1, 0, 90, true));
    common::run_to(&mut w, 2, 5.0);
    let end = clock::tick_at(2, 12.0 * 60.0).unwrap();
    while !waiting(&w, 0) {
        w.tick_once();
        assert!(w.tick < end, "never waited at the stop");
    }
    let buses = w.transit.buses.len();
    let mut w = common::reload(&w, &c);
    // The buses where the timetable has them, the stop with its line and shelter.
    assert_eq!(w.transit.buses.len(), buses);
    assert!(w.transit.stops[0].line.is_some() && w.transit.stops[0].shelter);
    // They set off for the stop again, wait, get on and are at work on time.
    let m = morning(&mut w, end);
    assert!(m.at_stop.is_some(), "didn't wait at the stop again: {m:?}");
    let on = m.on.expect("never got on after loading");
    assert!(on < start(&w, 0), "got on at {:.2}", clock::hour(on));
}

#[test]
fn a_game_saved_on_the_bus_home_loads_with_them_walking_home_from_the_stop() {
    // (Work over at 14:15, the bus last at the stop at 14:08: they wait off the map for the next.)
    let c = content(10.25);
    let mut w = world(&c, &town(1, 0, 90, true));
    common::run_to(&mut w, 2, 5.0);
    morning(&mut w, clock::tick_at(2, 12.0 * 60.0).unwrap())
        .on
        .expect("never got on");
    common::run_to(&mut w, 2, 14.26);
    assert_eq!(sim_core::bus::status(&w, &w.sims[0]), Some("home"));
    let funds = w.households[0].funds;
    let mut w = World::from_save_json(&c, &w.save_json()).unwrap();
    w.autonomy = false;
    // Back at their stop, out of sight no longer, on their way home; paid for the day only once.
    assert!(w.sims[0].away_until.is_none());
    let home = common::until(&mut w, 60, |w| {
        let (x, z) = w.sims[0].tile();
        w.plot_at(x, z) == Some(0)
    });
    assert!(home, "never walked home");
    assert_eq!(w.households[0].funds, funds);
}

#[test]
fn a_save_without_stops_has_no_bus() {
    let c = content(10.0);
    let w = world(&c, &town(1, 0, 90, false));
    let mut w = World::from_save_json(&c, &w.save_json()).unwrap();
    w.autonomy = false;
    assert!(w.transit.stops.is_empty() && w.transit.lines.is_empty());
    common::run_to(&mut w, 2, 9.0);
    assert!(w.transit.buses.is_empty(), "a bus in a town without stops");
    let mut out = vec![0.0; snapshot::CAPACITY];
    snapshot::write(&w, &mut out);
    assert_eq!(out[snapshot::header::BUS_COUNT], 0.0);
    // They walk out of town as ever (nothing timed by a bus).
    assert!(
        w.sims[0].commute().is_none(),
        "a commute in a cosy town without a bus"
    );
    let end = clock::tick_at(2, 12.0 * 60.0).unwrap();
    while w.tick < end && w.sims[0].away_until.is_none() {
        w.tick_once();
    }
    assert!(w.sims[0].away_until.is_some(), "never got to work");
}

#[test]
fn a_big_worlds_stops_stay_ways_out_on_foot_while_the_bus_runs() {
    let content = common::content();
    let h = common::hamlets::two_hamlets(&content, 12);
    let mut w = World::from_json(&content, &h.town, 12).unwrap();
    assert!(w.transit.runs() && w.transit.walk_stops);
    let stop = w.transit.stops[0].exit;
    assert!(
        w.transit
            .walks_out(stop, clock::tick_at(2, 9.0 * 60.0).unwrap())
    );
    // Timed as the walk-out it always was: nobody's way to work is planned by a bus.
    let end = clock::tick_at(2, 12.0 * 60.0).unwrap();
    let mut rode = 0;
    while w.tick < end {
        w.tick_once();
        for i in 0..w.sims.len() {
            assert!(w.sims[i].commute().is_none_or(|c| c.bus.is_none()));
            rode += usize::from(sim_core::bus::status(&w, &w.sims[i]) == Some("work"));
        }
    }
    assert!(
        rode > 0,
        "nobody took the bus calling at the stop they walked out by"
    );
}

#[test]
fn someone_in_front_of_a_bus_waiting_for_them_doesnt_walk_through_it() {
    // Work in the evening: W0 is free in the morning, when the buses run.
    let c = content(20.0);
    let mut w = world(&c, &town(1, 0, 150, true));
    common::run_to(&mut w, 2, 6.0);
    // A bus coming in from the west end...
    let end = clock::tick_at(2, 8.0 * 60.0).unwrap();
    let bus = loop {
        w.tick_once();
        assert!(w.tick < end, "no bus came");
        if let Some(b) = w.transit.buses.iter().position(|b| {
            b.shown
                .is_some_and(|(at, _, moving, _)| moving && (30.0..50.0).contains(&at[0]))
        }) {
            break b;
        }
    };
    let (at, ..) = w.transit.buses[bus].shown.unwrap();
    // ...and W0 walking west along its lane towards it, past it.
    w.sims[0].pos = [at[0] + 25.5, STREET + 0.5];
    w.apply(sim_core::Command::MoveTo {
        sim: 0,
        x: at[0] - 20.5,
        z: STREET + 0.5,
    })
    .unwrap();
    let (mut closest, mut held) = (f32::INFINITY, false);
    let run = w.transit.buses[bus].run;
    let mut passed = false;
    for _ in 0..3000 {
        w.tick_once();
        let Some(b) = w.transit.buses.iter().find(|b| b.run == run) else {
            break;
        };
        let (at, yaw, moving, doors) = b.shown.unwrap();
        // (Stood still away from the stop with its doors shut: held up for W0.)
        held |= !moving && doors == 0 && (at[0] - STOP_X).abs() > 8.0;
        let (s, k) = yaw.sin_cos();
        let p = w.sims[0].pos;
        let (dx, dz) = (p[0] - at[0], p[1] - at[1]);
        let (along, across) = (dx * s + dz * k, dx * k - dz * s);
        let (ex, ez) = (
            along.abs() - sim_core::bus::BUS_LEN / 2.0,
            across.abs() - sim_core::bus::BUS_WID / 2.0,
        );
        // (A quarter of a metre round them.)
        closest = closest.min(ex.max(0.0).hypot(ez.max(0.0)) + ex.max(ez).min(0.0) - 0.25);
        passed |= along < -sim_core::bus::BUS_LEN / 2.0;
    }
    assert!(held, "the bus never stopped for them");
    assert!(passed, "they never got past it");
    assert!(closest >= 0.0, "they walked {:.2} m into the bus", -closest);
}

/// How far two buses' bodies overlap (m; 0 or less: apart), each at its middle facing its yaw: the
/// least along any of their sides' ways.
fn overlap(a: ([f32; 2], f32), b: ([f32; 2], f32)) -> f32 {
    let half = [sim_core::bus::BUS_LEN / 2.0, sim_core::bus::BUS_WID / 2.0];
    let axes = |yaw: f32| {
        let (s, c) = yaw.sin_cos();
        [[s, c], [c, -s]]
    };
    let (aa, ba) = (axes(a.1), axes(b.1));
    let d = [b.0[0] - a.0[0], b.0[1] - a.0[1]];
    let reach = |ax: &[[f32; 2]; 2], u: [f32; 2]| {
        half[0] * (ax[0][0] * u[0] + ax[0][1] * u[1]).abs()
            + half[1] * (ax[1][0] * u[0] + ax[1][1] * u[1]).abs()
    };
    aa.iter()
        .chain(ba.iter())
        .map(|&u| reach(&aa, u) + reach(&ba, u) - (d[0] * u[0] + d[1] * u[1]).abs())
        .fold(f32::INFINITY, f32::min)
}

/// The most two shown buses' bodies overlap (m) in `w` now, and which runs (minutes).
fn worst_overlap(w: &World) -> (f32, u64, u64) {
    let shown: Vec<_> = w
        .transit
        .buses
        .iter()
        .filter_map(|b| b.shown.map(|(at, yaw, ..)| (b.run, at, yaw)))
        .collect();
    let mut worst = (0.0, 0, 0);
    for (i, a) in shown.iter().enumerate() {
        for b in &shown[i + 1..] {
            let o = overlap((a.1, a.2), (b.1, b.2));
            if o > worst.0 {
                worst = (o, a.0 / TICKS_PER_MINUTE, b.0 / TICKS_PER_MINUTE);
            }
        }
    }
    worst
}

#[test]
fn a_bus_arriving_where_one_stands_waits_behind_it() {
    // A bus every 5 minutes: each comes to the stop while the one before still stands there.
    let c = content_every(20.0, 5);
    let mut w = world(&c, &town(0, 0, 150, true));
    common::run_to(&mut w, 2, 6.0);
    let line = w.transit.lines[0].clone();
    let (bay, _) = line.bay(0);
    let mut called = std::collections::BTreeSet::new();
    let mut queued = false;
    let end = clock::tick_at(2, 8.0 * 60.0).unwrap();
    while w.tick < end {
        w.tick_once();
        let (o, a, b) = worst_overlap(&w);
        assert!(o <= 0.0, "the {a} and {b} buses overlap by {o:.2} m");
        for bus in &w.transit.buses {
            let Some((at, _, moving, doors)) = bus.shown else {
                continue;
            };
            if doors != 0 {
                called.insert(bus.run);
            }
            // (One stood behind the stop while another was at it.)
            let behind = at[0] < bay[0] - 4.0 && at[0] > bay[0] - 20.0;
            queued |= behind && !moving && doors == 0;
        }
    }
    assert!(queued, "no bus ever waited behind another at the stop");
    // They keep calling, one after another (stood less, late: `DWELL_LEAST`).
    assert!(
        called.len() >= 10,
        "{} buses called in two hours",
        called.len()
    );
}

#[test]
fn two_lines_coming_in_the_same_way_are_sent_apart() {
    let content = common::content();
    // Two hamlets side by side, a line each, both in from the road out at the west end.
    let h = common::hamlets::hamlets(
        &content,
        7,
        3,
        &[
            common::hamlets::Place { x: 0, z: 0 },
            common::hamlets::Place { x: 70, z: 0 },
        ],
        (512, 64),
    );
    let mut w = World::from_json(&content, &h.town, 7).unwrap();
    assert_eq!(w.transit.lines.len(), 2);
    // Where each line's first bus comes into sight, and when.
    let mut first: Vec<Option<(u64, [f32; 2])>> = vec![None; 2];
    let end = clock::tick_at(1, 14.0 * 60.0).unwrap();
    while w.tick < end {
        w.tick_once();
        let (o, a, b) = worst_overlap(&w);
        assert!(o <= 0.0, "the {a} and {b} buses overlap by {o:.2} m");
        for bus in &w.transit.buses {
            if let Some((at, ..)) = bus.shown
                && first[bus.line as usize].is_none()
                && bus.run >= clock::tick_at(1, 9.0 * 60.0).unwrap()
            {
                first[bus.line as usize] = Some((w.tick, at));
            }
        }
    }
    let (Some((t0, p0)), Some((t1, p1))) = (first[0], first[1]) else {
        panic!("a line's bus never came: {first:?}");
    };
    // (The same way in: they came into sight at the same place.)
    assert!(
        (p0[0] - p1[0]).hypot(p0[1] - p1[1]) < 1.0,
        "the lines come in at {p0:?} and {p1:?}"
    );
    assert_ne!(t0, t1, "both lines' buses sent at the same tick");
}

#[test]
fn a_bus_boxed_in_for_good_is_taken_out_of_service_and_the_town_recovers() {
    // A car parked across the street west of the stop: every bus coming in from the west end
    // stands behind it.
    let c = content(20.0);
    let mut town: serde_json::Value = serde_json::from_str(&town(0, 0, 150, true)).unwrap();
    town["cars"] = serde_json::json!([{"plot": 0, "x": 60.0, "z": STREET, "yaw": std::f32::consts::FRAC_PI_2,
        "street": STREET, "len": 4.4, "wid": 1.8, "body": 1, "paint": 2}]);
    let mut w = world(&c, &town.to_string());
    let line = w.transit.lines[0].clone();
    let (bay, _) = line.bay(0);
    let called = |w: &World| {
        w.transit.buses.iter().any(|b| {
            b.shown
                .is_some_and(|(at, _, _, doors)| doors != 0 && (at[0] - bay[0]).abs() < 1.0)
        })
    };
    common::run_to(&mut w, 2, 6.0);
    let end = clock::tick_at(2, 9.0 * 60.0).unwrap();
    while w.tick < end {
        w.tick_once();
        let (o, a, b) = worst_overlap(&w);
        assert!(o <= 0.0, "the {a} and {b} buses overlap by {o:.2} m");
        assert!(!called(&w), "a bus got past the car to the stop");
    }
    assert!(w.transit.withdrawn > 0, "no bus was taken out of service");
    // The car moved off: the buses call at the stop again.
    w.cars[0].spot = [150.0, 50.0];
    let end = clock::tick_at(2, 11.0 * 60.0).unwrap();
    let mut again = false;
    while w.tick < end && !again {
        w.tick_once();
        again = called(&w);
    }
    assert!(again, "no bus called at the stop once the way was clear");
}

#[test]
fn someone_who_misses_the_bus_makes_their_own_way_rather_than_wait_for_the_next() {
    // A bus an hour: held up at home after setting off, they come to the stop with theirs gone and
    // the next half an hour off.
    let c = content_every(10.0, 60);
    let mut w = world(&c, &town(1, 0, 90, true));
    common::run_to(&mut w, 2, 5.0);
    let end = clock::tick_at(2, 12.0 * 60.0).unwrap();
    while w.bound_for_bus(0).is_none() {
        w.tick_once();
        assert!(w.tick < end, "never set off for the bus");
    }
    let home = w.sims[0].pos;
    let held = w.tick + 30 * TICKS_PER_MINUTE;
    while w.tick < held {
        w.tick_once();
        w.sims[0].pos = home;
    }
    let (mut at_stop, mut most) = (0, 0);
    while w.tick < end && w.sims[0].away_until.is_none() {
        w.tick_once();
        at_stop = if waiting(&w, 0) { at_stop + 1 } else { 0 };
        most = most.max(at_stop);
    }
    assert!(
        most < 2 * TICKS_PER_MINUTE,
        "waited {} min at the stop for the next bus",
        most / TICKS_PER_MINUTE
    );
    assert!(w.sims[0].away_until.is_some(), "never got to work");
}
