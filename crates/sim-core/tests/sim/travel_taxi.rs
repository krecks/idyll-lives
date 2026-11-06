//! The work taxi (`taxi.rs`): colleagues a long way from the way out share one in to work and home
//! again; those with a car seat or a bike, or a short way, don't take one; one not at the kerb is
//! left; pupils go to school by one; a game saved mid-run goes on.

use crate::common;

use sim_core::World;
use sim_core::cars::CarState;
use sim_core::taxi::StopState;
use sim_core::world::Phase;

const STREET: f32 = 30.0;

/// A house north of the street (or `south`) at `x` (a plot 20 m wide), its driveway onto the street.
struct House {
    x: i32,
    south: bool,
    /// Its residents: (name, job or pupil, age stage) as town-file JSON.
    sims: Vec<String>,
    car: bool,
    bike: bool,
    /// How deep its driveway runs from the street (m to its inner end; none, no driveway).
    drive: f32,
}

fn house(x: i32, sims: &[&str]) -> House {
    House {
        x,
        south: false,
        sims: sims.iter().map(|s| s.to_string()).collect(),
        car: false,
        bike: false,
        drive: 12.0,
    }
}

/// A worker's town-file JSON (a print-shop job).
fn worker(name: &str) -> String {
    format!(
        r#"{{"name":"{name}","age":30,"traits":["cheerful"],"job":{{"career":"business.management","level":0}}}}"#
    )
}

/// A normal-sized town: a 600 m street of `width` m along x, its way out at the east end, and
/// `houses` on it.
fn town(width: u32, houses: &[House]) -> World {
    town_on(width, houses, |_| 0.0)
}

/// `town`, its street `bend(x)` m south of `STREET` at `x` (a gentle bend, its houses on the
/// north side where they'd be by a straight one).
fn town_on(width: u32, houses: &[House], bend: impl Fn(f32) -> f32) -> World {
    town_with(width, houses, bend, &[])
}

/// `town_on`, with more roads (the town file's road JSON) meeting the street.
fn town_with(width: u32, houses: &[House], bend: impl Fn(f32) -> f32, more: &[String]) -> World {
    let road: Vec<String> = (0..=599)
        .map(|x| format!("[{x}.5,{}]", STREET - bend(x as f32 + 0.5)))
        .collect();
    let kind = if width < 7 { "lane" } else { "street" };
    let road = format!(
        r#"{{"points":[{}],"width":{width},"kind":"{kind}","ends":["dead","out"]}}"#,
        road.join(",")
    );
    let (mut plots, mut households, mut sims, mut drives, mut cars, mut objects) =
        (vec![], vec![], vec![], vec![], vec![], vec![]);
    for (k, h) in houses.iter().enumerate() {
        let edge = STREET + width as f32 / 2.0;
        let (z, inner, front) = if h.south {
            (edge as i32 - 2 - 16 - h.drive as i32, edge - h.drive, 1)
        } else {
            (edge as i32 + 2, edge + h.drive, -1)
        };
        let entry_z = if h.south { z + 15 } else { z };
        plots.push(format!(
            r#"{{"name":"H{k}","x":{},"z":{z},"w":20,"d":16,"entry":[{}.5,{}.5]}}"#,
            h.x,
            h.x + 10,
            entry_z
        ));
        households.push(format!(r#"{{"name":"F{k}","plot":{k}}}"#));
        for s in &h.sims {
            let mut v: serde_json::Value = serde_json::from_str(s).unwrap();
            v["household"] = k.into();
            v["x"] = (h.x as f32 + 12.5).into();
            v["z"] = (z as f32 + 8.5).into();
            sims.push(v.to_string());
        }
        let cx = h.x as f32 + 3.5;
        if h.drive <= 0.0 {
            continue;
        }
        drives.push(format!(
            r#"{{"plot":{k},"x":{cx},"z":{inner},"front":{front},"street":{STREET},"road":0,"s":{}}}"#,
            cx - 0.5
        ));
        if h.car {
            let spot = inner + front as f32 * (0.45 + 2.2);
            let yaw = if front < 0 { std::f32::consts::PI } else { 0.0 };
            cars.push(format!(
                r#"{{"plot":{k},"x":{cx},"z":{spot},"yaw":{yaw},"street":{STREET},"len":4.4,"wid":1.8,"body":1,"paint":2}}"#
            ));
        }
        if h.bike {
            objects.push(format!(
                r#"{{"def":"bicycle","x":{},"z":{},"rot":0}}"#,
                h.x + 14,
                z + 6
            ));
        }
    }
    let more: String = more.iter().map(|r| format!(",{r}")).collect();
    let json = format!(
        r#"{{"width":600,"depth":80,"size":"normal",
        "exits":[{{"x":599.5,"z":{STREET},"kind":"road"}}],
        "roads":[{road}{more}],"meta":{{"roads":[{road}{more}]}},
        "plots":[{}],"households":[{}],"objects":[{}],"sims":[{}],"drives":[{}],"cars":[{}]}}"#,
        plots.join(","),
        households.join(","),
        objects.join(","),
        sims.join(","),
        drives.join(","),
        cars.join(",")
    );
    let mut w = World::from_json(&common::content(), &json, 5).unwrap();
    w.autonomy = false;
    w
}

/// Ticks `w` on to `day` at `hour`, checking no two cars ever overlap (one on the move).
fn run_to(w: &mut World, day: u32, hour: f32) {
    let end = sim_core::clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < end {
        w.tick_once();
        no_overlaps(w);
    }
}

fn corners(p: [f32; 2], yaw: f32, len: f32, wid: f32) -> [[f32; 2]; 4] {
    let (s, c) = yaw.sin_cos();
    let f = [s * len / 2.0, c * len / 2.0];
    let r = [c * wid / 2.0, -s * wid / 2.0];
    [
        [p[0] + f[0] + r[0], p[1] + f[1] + r[1]],
        [p[0] + f[0] - r[0], p[1] + f[1] - r[1]],
        [p[0] - f[0] - r[0], p[1] - f[1] - r[1]],
        [p[0] - f[0] + r[0], p[1] - f[1] + r[1]],
    ]
}

/// How deep two bodies overlap (m; below 0 apart).
fn depth(a: &[[f32; 2]; 4], b: &[[f32; 2]; 4]) -> f32 {
    let mut best = f32::INFINITY;
    for poly in [a, b] {
        for k in 0..2 {
            let (p, q) = (poly[k], poly[k + 1]);
            let n = [q[1] - p[1], p[0] - q[0]];
            let l = n[0].hypot(n[1]);
            let n = [n[0] / l, n[1] / l];
            let proj = |r: &[[f32; 2]; 4]| {
                r.iter()
                    .map(|v| v[0] * n[0] + v[1] * n[1])
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), x| {
                        (lo.min(x), hi.max(x))
                    })
            };
            let (a0, a1) = proj(a);
            let (b0, b1) = proj(b);
            best = best.min(a1.min(b1) - a0.max(b0));
        }
    }
    best
}

/// No car (a taxi too) overlaps another, one of them on the move, by more than 5 cm.
fn no_overlaps(w: &World) {
    let bodies: Vec<_> = w
        .cars
        .iter()
        .filter_map(|c| c.shown.map(|s| (corners(s.0, s.1, c.len, c.wid), s.2)))
        .collect();
    for i in 0..bodies.len() {
        for j in i + 1..bodies.len() {
            if bodies[i].1 || bodies[j].1 {
                let d = depth(&bodies[i].0, &bodies[j].0);
                assert!(d <= 0.05, "two cars overlap by {d:.2} m at tick {}", w.tick);
            }
        }
    }
}

/// How many have come to work by taxi so far (in time, or by a late run).
fn came(w: &World) -> usize {
    w.taxis.arrivals.len() + w.taxis.late_arrivals.len()
}

/// The ids of the runs to work that resident `i` rode, and of those home, so far.
fn rode(w: &World, i: usize, runs: &mut Vec<(u32, bool)>) {
    for r in &w.taxis.runs {
        if r.stops
            .iter()
            .any(|s| s.sim == i as u32 && s.state == StopState::Aboard)
            && !runs.contains(&(r.id, r.home))
        {
            runs.push((r.id, r.home));
        }
    }
}

#[test]
fn colleagues_a_long_way_out_share_one_taxi_there_on_time_and_one_home() {
    // Three office workers at the street's dead end, 500 m from the way out: one taxi for them.
    let mut w = town(
        7,
        &[
            house(20, &[&worker("A")]),
            house(50, &[&worker("B")]),
            house(80, &[&worker("C")]),
        ],
    );
    let mut runs: Vec<Vec<(u32, bool)>> = vec![Vec::new(); 3];
    let end = sim_core::clock::tick_at(2, 23.0 * 60.0).unwrap();
    let mut waited = false;
    while w.tick < end {
        w.tick_once();
        no_overlaps(&w);
        for (i, r) in runs.iter_mut().enumerate() {
            rode(&w, i, r);
        }
        waited |= w.sims.iter().any(|s| {
            s.current()
                .is_some_and(|a| matches!(a.phase, Phase::AtKerb { .. }))
        });
    }
    assert!(waited, "nobody waited at their kerb");
    for (i, r) in runs.iter().enumerate() {
        assert!(r.iter().any(|x| !x.1), "{i} never rode to work: {runs:?}");
        assert!(r.iter().any(|x| x.1), "{i} never rode home: {runs:?}");
    }
    // The same taxi there for all three, each day.
    let there: Vec<u32> = runs[0].iter().filter(|x| !x.1).map(|x| x.0).collect();
    for r in &runs[1..] {
        let theirs: Vec<u32> = r.iter().filter(|x| !x.1).map(|x| x.0).collect();
        assert_eq!(theirs, there, "they rode different taxis");
    }
    // On time: at work by its start (within the late grace). (The game begins after they'd have
    // set off on foot, a walk of over five hours: on day 1 a late run, out sooner than on foot.)
    let grace = w.content.career_rules.late_grace_minutes as i64 * 20;
    assert!(!w.taxis.arrivals.is_empty() && !w.taxis.late_arrivals.is_empty());
    for &(s, late) in &w.taxis.late_arrivals {
        assert!(
            late < 4 * 60 * 20,
            "{s} at work {} min late by a late run",
            late / 20
        );
    }
    for &(s, late) in &w.taxis.arrivals {
        assert!(late <= grace, "{s} at work {} min late", late / 20);
    }
    // Home again, out of sight no longer, the taxi gone.
    run_to(&mut w, 3, 2.0);
    for s in &w.sims {
        assert!(s.away_until.is_none() && s.here(), "{} not home", s.name);
    }
    assert!(
        w.cars
            .iter()
            .filter(|c| c.taxi)
            .all(|c| c.state == CarState::Away && c.shown.is_none())
    );
}

#[test]
fn a_rider_with_a_bike_or_a_car_seat_or_a_short_way_does_not_take_one() {
    // A cyclist (a ride out of 45 minutes or so), a driver with their car, and someone by the way
    // out: none of them by taxi.
    let mut cyclist = house(420, &[&worker("Bea")]);
    cyclist.bike = true;
    let mut driver = house(100, &[&worker("Dan")]);
    driver.car = true;
    let near = house(570, &[&worker("Nia")]);
    let mut w = town(7, &[cyclist, driver, near]);
    run_to(&mut w, 2, 12.0);
    for i in 0..3 {
        assert!(w.taxi_booking(i).is_none(), "{i} booked");
    }
    assert!(
        w.taxis.runs.is_empty() && came(&w) == 0,
        "a taxi ran for them"
    );
}

#[test]
fn a_rider_not_at_the_kerb_is_left_and_makes_their_own_way() {
    let mut w = town(7, &[house(20, &[&worker("A")]), house(60, &[&worker("B")])]);
    // Till both are booked, and B then kept from the kerb (sent off to the dead end).
    let end = sim_core::clock::tick_at(2, 12.0 * 60.0).unwrap();
    while w.taxi_booking(1).is_none() {
        w.tick_once();
        assert!(w.tick < end, "never booked");
    }
    let run = w.taxi_booking(1).unwrap().run;
    let mut left = false;
    while w.tick < end {
        w.tick_once();
        no_overlaps(&w);
        if w.taxi_booking(1).is_some() {
            // (Wherever they're going: kept at the far end of their garden, out of reach of the
            // taxi.)
            w.sims[1].pos = [62.5, STREET + 18.0];
        }
        let state = w
            .taxis
            .runs
            .iter()
            .find(|r| r.id == run)
            .and_then(|r| r.stops.iter().find(|s| s.sim == 1))
            .map(|s| s.state);
        if state == Some(StopState::Done) && w.taxi_booking(1).is_none() {
            left = true;
            break;
        }
    }
    assert!(left, "B was never left");
    // A rode it all the same.
    let mut a = Vec::new();
    while w.tick < end && a.is_empty() {
        w.tick_once();
        rode(&w, 0, &mut a);
    }
    assert!(!a.is_empty(), "A never rode");
}

#[test]
fn a_pupil_goes_to_school_by_taxi() {
    let pupil = r#"{"name":"Pip","age":10,"traits":["cheerful"]}"#;
    let mut w = town(7, &[house(20, &[pupil, &worker("Mum")])]);
    let mut pip = Vec::new();
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    while w.tick < end && pip.is_empty() {
        w.tick_once();
        rode(&w, 0, &mut pip);
    }
    assert!(!pip.is_empty(), "Pip never rode to school");
}

#[test]
fn a_game_saved_with_a_taxi_on_its_way_loads_with_it_where_it_was() {
    let mut w = town(7, &[house(20, &[&worker("A")]), house(50, &[&worker("B")])]);
    let end = sim_core::clock::tick_at(2, 12.0 * 60.0).unwrap();
    // Till a taxi is in town with someone in it.
    while !w
        .taxis
        .runs
        .iter()
        .any(|r| r.car.is_some() && r.stops.iter().any(|s| s.state == StopState::Aboard))
    {
        w.tick_once();
        assert!(w.tick < end, "no taxi came");
    }
    let json = w.save_json();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v["taxis"]["runs"].as_array().is_some_and(|r| !r.is_empty()));
    assert_eq!(v["version"], sim_core::save::SAVE_VERSION);
    let mut loaded = World::from_save_json(&common::content(), &json).unwrap();
    let progress = |w: &World| {
        w.taxis
            .runs
            .iter()
            .find(|r| r.car.is_some())
            .map(|r| r.progress)
    };
    assert_eq!(progress(&loaded), progress(&w), "not where it was");
    // On it goes, and they're at work.
    while loaded.tick < end && came(&loaded) < 2 {
        loaded.tick_once();
        no_overlaps(&loaded);
    }
    assert!(came(&loaded) >= 2, "the riders never got to work");
}

#[test]
fn on_a_6_m_lane_a_taxi_at_a_blocked_driveway_stops_at_the_kerb_clear_of_the_car_up_it() {
    // Pip's driveway is too short for a taxi in its mouth with the household's car parked up it
    // (nobody at home drives it): the school taxi stops at the kerb by it, in its lane, and never
    // touches the car. Pat down the lane goes by it too.
    let pip = r#"{"name":"Pip","age":10,"traits":["cheerful"]}"#;
    let pat = r#"{"name":"Pat","age":11,"traits":["cheerful"]}"#;
    let mut short = house(60, &[pip]);
    short.car = true;
    short.drive = 7.5;
    let mut w = town(6, &[short, house(20, &[pat])]);
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    let (mut kerb, mut by) = (false, (Vec::new(), Vec::new()));
    // (To school: home again they may come by one as well, `taxi::off_work`.)
    let to = |r: &Vec<(u32, bool)>| r.iter().any(|x| !x.1);
    while w.tick < end && !(to(&by.0) && to(&by.1)) {
        w.tick_once();
        no_overlaps(&w);
        for c in w.cars.iter().filter(|c| c.taxi && c.standing) {
            let p = c.shown.map(|s| s.0).unwrap_or([0.0, 0.0]);
            // (In its lane on the road, not in the driveway.)
            kerb |= (p[1] - STREET).abs() < 2.0;
        }
        rode(&w, 0, &mut by.0);
        rode(&w, 1, &mut by.1);
    }
    assert!(kerb, "never stood at the kerb");
    let there = |r: &[(u32, bool)]| r.iter().filter(|x| !x.1).map(|x| x.0).next();
    assert!(
        there(&by.0).is_some() && there(&by.1).is_some(),
        "not both to school"
    );
    assert_eq!(there(&by.0), there(&by.1), "not the same taxi");
}

#[test]
fn housemates_going_the_same_way_share_a_taxi_whatever_their_work() {
    // Two of one house in different careers, starting at the same hour: one taxi, one halt.
    let other = r#"{"name":"Pip","age":30,"traits":["cheerful"],"job":{"career":"business.marketing","level":0}}"#;
    let mut w = town(7, &[house(20, &[&worker("Mum"), other])]);
    let (mut mum, mut pip) = (Vec::new(), Vec::new());
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    while w.tick < end && (mum.is_empty() || pip.is_empty()) {
        w.tick_once();
        no_overlaps(&w);
        rode(&w, 0, &mut mum);
        rode(&w, 1, &mut pip);
    }
    let there = |r: &[(u32, bool)]| r.iter().filter(|x| !x.1).map(|x| x.0).next();
    assert!(
        there(&mum).is_some() && there(&mum) == there(&pip),
        "{mum:?} {pip:?}"
    );
}

/// Car `c` sets off now with its driver (resident `d`), or comes home now.
fn drive(w: &mut World, c: usize, d: u32, home: bool) {
    let car = &mut w.cars[c];
    car.state = if home {
        CarState::Away
    } else {
        CarState::Leaving
    };
    car.since = w.tick;
    car.progress = 0.0;
    car.driver = Some(d);
    car.aboard = vec![d];
    if home {
        car.done = vec![d];
        car.ready_at = w.tick;
    }
    w.sims[d as usize].away_until = Some(w.tick + 100_000);
}

#[test]
fn the_households_car_pulling_out_or_coming_home_as_the_taxi_stands_at_its_kerb_waits_its_turn() {
    // Pip's house has a car (Dad's) on its driveway: as the taxi stands at their kerb, it pulls
    // out (and, another day, comes home). Never into each other; both on their way soon. And so
    // as the taxi comes up to their kerb (`early`): pulling out as it's a few metres off, or
    // coming home as it comes into town (the taxi turning where the car parks).
    for (home, early) in [(false, false), (true, false), (false, true), (true, true)] {
        let pip = r#"{"name":"Pip","age":10,"traits":["cheerful"]}"#;
        let dad = r#"{"name":"Dad","age":40,"traits":["cheerful"]}"#;
        let mut h = house(60, &[pip, dad]);
        h.car = true;
        let mut w = town(6, &[h]);
        if home {
            // (Away, coming home when the taxi stands there.)
            w.cars[0].state = CarState::Away;
            w.cars[0].aboard = vec![1];
            w.sims[1].away_until = Some(w.tick + 100_000);
        }
        let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
        let mut sent = false;
        let mut done = None;
        while w.tick < end && done.is_none() {
            w.tick_once();
            no_overlaps(&w);
            let mouth = w
                .drives
                .iter()
                .find(|d| d.plot == w.cars[0].plot)
                .map(|d| [d.x, d.street])
                .expect("Pip's driveway");
            let coming = |c: &sim_core::cars::Car| {
                c.taxi
                    && !c.standing
                    && c.state == CarState::Arriving
                    && c.shown.is_some_and(|s| {
                        home || (s.0[0] - mouth[0]).hypot(s.0[1] - mouth[1]) < 12.0
                    })
            };
            let now = if early {
                w.cars.iter().any(coming)
            } else {
                w.cars.iter().any(|c| c.taxi && c.standing)
            };
            if !sent && now {
                drive(&mut w, 0, 1, home);
                sent = true;
            }
            let car_done = if home {
                w.cars[0].state == CarState::Parked
            } else {
                w.cars[0].state == CarState::Away
            };
            if sent && car_done && came(&w) > 0 {
                done = Some(w.tick);
            }
        }
        assert!(sent, "({home}, {early}) the taxi never stood at the kerb");
        assert!(done.is_some(), "({home}, {early}) not both on their way");
    }
}

/// A 7 m street with a car of house `taxi_x` made a work taxi standing at the kerb in its lane
/// (driven off east, stopped 30 m on along the street and held there, as a taxi waiting for its
/// rider), and the cars of houses `more`, each with its driver: the world, the taxi's index, and
/// each car's.
fn street_with_standing_taxi(taxi_x: i32, more: &[i32]) -> (World, usize, Vec<usize>) {
    let driver = |n: &str| format!(r#"{{"name":"{n}","age":40,"traits":["cheerful"]}}"#);
    let mut houses = Vec::new();
    for (k, &x) in std::iter::once(&taxi_x).chain(more).enumerate() {
        let d = driver(&format!("D{k}"));
        let mut h = house(x, &[&d]);
        h.car = true;
        houses.push(h);
    }
    let mut w = town(7, &houses);
    let car_of = |w: &World, k: usize| {
        w.cars
            .iter()
            .position(|c| c.plot == k as u32)
            .expect("its car")
    };
    let t = car_of(&w, 0);
    let others = (1..houses.len()).map(|k| car_of(&w, k)).collect();
    drive(&mut w, t, 0, false);
    let stop = taxi_x as f32 + 33.0;
    for _ in 0..4000 {
        w.tick_once();
        if w.cars[t].shown.is_some_and(|s| s.0[0] >= stop) {
            break;
        }
    }
    let car = &mut w.cars[t];
    let s = car.shown.expect("on the street");
    assert!(s.0[0] >= stop, "the taxi-to-be never got along the street");
    car.taxi = true;
    car.standing = true;
    car.shown = Some((s.0, s.1, false, 0));
    (w, t, others)
}

/// The least gap between two cars' bodies (m; below 0 overlapping).
fn gap_between(w: &World, a: usize, b: usize) -> f32 {
    let body = |c: &sim_core::cars::Car| {
        let s = c.shown.expect("shown");
        corners(s.0, s.1, c.len, c.wid)
    };
    -depth(&body(&w.cars[a]), &body(&w.cars[b]))
}

#[test]
fn a_car_behind_a_taxi_standing_at_the_kerb_on_a_7_m_street_passes_it() {
    // A taxi stands at the kerb in its lane (waiting for its rider, as Canalside's did while one
    // behind it waited a quarter of an hour): the car coming up behind it goes out round it into
    // the other lane, the way clear, and back in past it (`cars::PassBy`), never touching it, nor
    // turning round.
    let (mut w, t, cars) = street_with_standing_taxi(100, &[20]);
    let f = cars[0];
    let taxi_x = w.cars[t].shown.unwrap().0[0];
    drive(&mut w, f, 1, false);
    let (mut out, mut past, mut least) = (false, None, f32::INFINITY);
    let start = w.tick;
    while w.tick < start + 3000 && past.is_none() {
        w.tick_once();
        no_overlaps(&w);
        let car = &w.cars[f];
        assert!(w.cars[t].standing, "the taxi left");
        assert!(
            car.detour.is_none(),
            "it turned round instead, tick {}",
            w.tick
        );
        out |= car.pass_by.is_some();
        if car.shown.is_some() {
            least = least.min(gap_between(&w, f, t));
        }
        if car.shown.is_some_and(|s| s.0[0] > taxi_x + 10.0) {
            past = Some(w.tick);
        }
    }
    assert!(out, "it never went out round the taxi");
    assert!(past.is_some(), "it never got past the taxi");
    assert!(least >= 0.15, "it came {least:.2} m from the taxi going by");
}

#[test]
fn one_coming_the_other_way_waits_its_turn_short_of_a_taxi_standing_at_the_kerb() {
    // As one goes out round a taxi standing at the kerb into the other lane, one coming the other
    // way waits short of the stretch alongside it till it's back in its lane: they take turns
    // there (`Route::spans`), in the order they came. Started at times one apart: never two
    // cars touching, both through, nobody turning round; and the one coming the other way stood
    // waiting outside that stretch as the other went by.
    let mut waited = 0;
    for k in 0..10u64 {
        let (mut w, t, cars) = street_with_standing_taxi(300, &[220, 160]);
        let (f, o) = (cars[0], cars[1]);
        let taxi = w.cars[t].shown.unwrap().0;
        let len = w.cars[t].len;
        // (The other coming home from the way out, east, to its house west of the taxi.)
        drive(&mut w, o, 2, true);
        let start = w.tick;
        let (mut sent, mut home, mut gone) = (false, false, false);
        let mut still = (0.0f32, 0u32);
        let mut stood_out = false;
        while w.tick < start + 6000 && !(home && gone) {
            w.tick_once();
            no_overlaps(&w);
            if !sent && w.tick >= start + 300 + 25 * k {
                drive(&mut w, f, 1, false);
                sent = true;
            }
            for c in [f, o] {
                assert!(w.cars[c].detour.is_none(), "car {c} turned round, case {k}");
            }
            let (fc, oc) = (&w.cars[f], &w.cars[o]);
            // (The other standing still east of the taxi as this one is out round it.)
            let going_by = fc
                .pass_by
                .is_some_and(|p| fc.progress > p.from && fc.progress < p.to);
            if oc.state == CarState::Arriving {
                if (oc.progress - still.0).abs() < 1e-3 {
                    still.1 += 1;
                } else {
                    still = (oc.progress, 0);
                }
                let east = oc
                    .shown
                    .is_some_and(|s| s.0[0] > taxi[0] + len / 2.0 + oc.len / 2.0);
                if going_by && east && still.1 >= 10 {
                    stood_out = true;
                }
            }
            home |= oc.state == CarState::Parked && w.tick > start + 10;
            gone |= sent && fc.state == CarState::Away;
        }
        assert!(home && gone, "case {k}: not both through ({home}, {gone})");
        waited += usize::from(stood_out);
    }
    assert!(
        waited > 0,
        "the one coming the other way never waited its turn"
    );
}

/// A's taxi comes in from the way out (east) along a 7 m street to A's kerb (south of the street,
/// in its lane coming in): two neighbours west of A come home right behind it (up behind it as it
/// stands: out round it), and two drive off east as it stands there (meeting them alongside it).
/// Each tick (no two cars ever overlapping) `each` sees the world and the neighbours' cars, and may
/// stop it (true); else till all are through. Whether anyone went out round the taxi
/// (`Car::pass_by`) and was alongside it.
fn cars_round_a_standing_taxi(mut each: impl FnMut(&World, &[usize]) -> bool) -> bool {
    let driver = |n: &str| format!(r#"{{"name":"{n}","age":40,"traits":["cheerful"]}}"#);
    let mut a = house(300, &[&worker("A")]);
    a.south = true;
    let mut houses = vec![a];
    for (k, x) in [100, 160, 40, 250].into_iter().enumerate() {
        let d = driver(&format!("D{k}"));
        let mut h = house(x, &[&d]);
        h.car = true;
        houses.push(h);
    }
    let mut w = town(7, &houses);
    let car_of = |w: &World, k: usize| {
        w.cars
            .iter()
            .position(|c| !c.taxi && c.plot == k as u32)
            .expect("its car")
    };
    let cars: Vec<usize> = (1..=4).map(|k| car_of(&w, k)).collect();
    // (The first two away, to come home as the taxi comes.)
    for (k, &c) in cars.iter().enumerate().take(2) {
        w.cars[c].state = CarState::Away;
        w.cars[c].aboard = vec![k as u32 + 1];
        w.sims[k + 1].away_until = Some(w.tick + 100_000);
    }
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    let (mut home, mut off) = (false, None);
    let mut passed = false;
    let taxi_at = |w: &World| {
        w.cars
            .iter()
            .find(|c| c.taxi && c.state == CarState::Arriving)
            .and_then(|c| c.shown)
    };
    while w.tick < end && off.is_none_or(|t| w.tick < t + 4000) {
        w.tick_once();
        no_overlaps(&w);
        passed |= cars.iter().any(|&c| {
            w.cars[c]
                .pass_by
                .is_some_and(|p| w.cars[c].progress > p.out && w.cars[c].progress < p.back)
        });
        if each(&w, &cars) {
            return passed;
        }
        let taxi = taxi_at(&w);
        if !home && taxi.is_some_and(|s| s.0[0] < 585.0) {
            for (k, &c) in cars.iter().enumerate().take(2) {
                drive(&mut w, c, k as u32 + 1, true);
            }
            home = true;
        }
        if home && off.is_none() && w.cars.iter().any(|c| c.taxi && c.standing) {
            for (k, &c) in cars.iter().enumerate().skip(2) {
                drive(&mut w, c, k as u32 + 1, false);
            }
            off = Some(w.tick);
        }
    }
    assert!(off.is_some(), "the taxi never came and stood");
    for (k, &c) in cars.iter().enumerate() {
        let want = if k < 2 {
            CarState::Parked
        } else {
            CarState::Away
        };
        assert_eq!(w.cars[c].state, want, "car {k} isn't through");
    }
    passed
}

#[test]
fn round_a_work_taxi_standing_at_the_kerb_cars_both_ways_never_touch() {
    // Whoever goes out round the taxi, waits its turn or turns round, no two cars ever touch
    // (bodies nearer than a centimetre, one of them on the move), one goes out round it, and
    // everyone gets where they were going (`cars_round_a_standing_taxi`).
    let passed = cars_round_a_standing_taxi(|w, _| {
        let moving: Vec<usize> = (0..w.cars.len())
            .filter(|&c| w.cars[c].shown.is_some_and(|s| s.2))
            .collect();
        for &a in &moving {
            for b in 0..w.cars.len() {
                if b != a && w.cars[b].shown.is_some() && (b > a || !moving.contains(&b)) {
                    let g = gap_between(w, a, b);
                    assert!(
                        g >= 0.01,
                        "cars {a} and {b} touch ({g:.3} m), tick {}",
                        w.tick
                    );
                }
            }
        }
        false
    });
    assert!(passed, "nobody went out round the taxi");
}

#[test]
fn a_game_saved_with_a_car_part_way_round_a_standing_taxi_loads_with_it_where_it_was() {
    // Saved as one is out round the taxi in the other lane (`cars_round_a_standing_taxi`), and
    // loaded: it's where it was (`cars::PassBy`, saved with it), not back in its own lane on top of
    // the taxi, and goes on by as before, touching nobody, never jumping.
    let mut saved = None;
    cars_round_a_standing_taxi(|w, cars| {
        saved = cars
            .iter()
            .copied()
            .find(|&c| {
                w.cars[c]
                    .pass_by
                    .is_some_and(|p| w.cars[c].progress > (p.out + p.back) / 2.0)
            })
            .map(|c| (c, w.cars[c].shown.expect("on the street").0, w.save_json()));
        saved.is_some()
    });
    let (c, at, json) = saved.expect("nobody went out round the taxi");
    let mut l = World::from_save_json(&common::content(), &json).unwrap();
    let mut was = None;
    for _ in 0..300 {
        l.tick_once();
        no_overlaps(&l);
        let Some(s) = l.cars[c].shown else { break };
        let (from, limit) = match was {
            None => (at, 0.5),
            Some(p) => (p, 0.6),
        };
        let step = (s.0[0] - from[0]).hypot(s.0[1] - from[1]);
        assert!(
            step < limit,
            "it moved {step:.2} m at once, to {:?} from {from:?}",
            s.0
        );
        was = Some(s.0);
    }
}

#[test]
fn a_car_held_up_by_a_crowd_crossing_in_front_of_it_gets_its_gap() {
    // (As at Willowley's busy corner: seven on foot crossing the street back and forth in front of
    // a car coming home, on and on.) Those already in its way go on out of it; those not yet in
    // it wait for it once it's waiting: it gets home soon, never into anyone.
    let mut crowd = house(300, &[]);
    crowd.sims = (0..7)
        .map(|k| format!(r#"{{"name":"W{k}","age":30,"traits":["cheerful"]}}"#))
        .collect();
    let mut home = house(200, &[r#"{"name":"Dad","age":40,"traits":["cheerful"]}"#]);
    home.car = true;
    let mut w = town(7, &[home, crowd]);
    run_to(&mut w, 1, 9.0);
    // (Away, coming home now, along the street past the crowd.)
    drive(&mut w, 0, 0, true);
    let end = w.tick + 4 * 60 * 20;
    let (mut k, mut reached) = (0, None);
    while w.cars[0].state != CarState::Parked && w.tick < end {
        // (From when it comes up to them: past them soon.)
        let x = w.cars[0].shown.map_or(f32::INFINITY, |s| s.0[0]);
        if reached.is_none()
            && x < 295.0
            && w.cars[0].state == CarState::Arriving
            && w.cars[0].progress > 0.0
        {
            reached = Some(w.tick);
        }
        w.tick_once();
        no_overlaps(&w);
        // (Each of them off across the street again every so often, between x 240 and 280.)
        if w.tick.is_multiple_of(40) {
            k += 1;
            let i = 1 + k % 7;
            let x = 240.5 + (k * 13 % 40) as f32;
            let z = if k % 2 == 0 {
                STREET - 5.0
            } else {
                STREET + 5.0
            };
            let _ = w.apply(sim_core::Command::MoveTo {
                sim: i as u32,
                x,
                z,
            });
        }
        if let Some(t) = reached {
            assert!(
                x < 230.0 || w.tick < t + 20 * 20,
                "the car still among them after 20 minutes: {:?}",
                w.cars[0].shown
            );
        }
    }
    assert!(reached.is_some(), "it never came up to them");
}

/// Each work taxi standing at the kerb now: where (its middle), facing which way.
fn standing(w: &World) -> Vec<(usize, [f32; 2], f32)> {
    w.cars
        .iter()
        .enumerate()
        .filter(|(_, c)| c.taxi && c.standing)
        .filter_map(|(k, c)| c.shown.map(|s| (k, s.0, s.1)))
        .collect()
}

/// How far `yaw` is from running along x either way (rad).
fn off_x(yaw: f32) -> f32 {
    let a = sim_core::world::wrap_angle(yaw - std::f32::consts::FRAC_PI_2).abs();
    a.min(std::f32::consts::PI - a)
}

#[test]
fn the_taxi_stands_in_its_lane_parallel_to_it_clear_of_the_kerb_and_the_riders_walk_to_its_door() {
    // Two office workers far from the way out: the taxi stands at the kerb by each house, in its
    // lane along the street (not turned into a driveway), its body clear of the pavement; each
    // walks from their kerb to its door and gets in there, never jumping to it.
    let mut w = town(7, &[house(20, &[&worker("A")]), house(60, &[&worker("B")])]);
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    let mut stood = 0;
    let mut was: Vec<Option<[f32; 2]>> = vec![None; 2];
    let mut aboard = [false; 2];
    let mut got_in = [false; 2];
    while w.tick < end && !(aboard[0] && aboard[1]) {
        w.tick_once();
        no_overlaps(&w);
        for (_, at, yaw) in standing(&w) {
            stood += 1;
            assert!(
                off_x(yaw) < 0.03,
                "stood {:.1}° off its lane",
                off_x(yaw).to_degrees()
            );
            // (A 7 m street's kerbs 2.15 m from its middle: its body, 0.93 m either side of its
            // middle, inside them.)
            assert!(
                (at[1] - STREET).abs() + 1.85 / 2.0 < 2.15,
                "over the kerb at {at:?}"
            );
        }
        for i in 0..2 {
            let s = &w.sims[i];
            let state = w
                .taxis
                .runs
                .iter()
                .filter(|r| !r.home)
                .flat_map(|r| r.stops.iter().filter(|st| st.sim == i as u32))
                .map(|st| st.state)
                .next();
            if state == Some(StopState::Aboard) {
                aboard[i] = true;
                was[i] = None;
                continue;
            }
            let doing = s.current().map(|a| &a.phase);
            if matches!(doing, Some(Phase::Boarding { .. })) && !got_in[i] {
                got_in[i] = true;
                // (At the taxi's side, by a door.)
                let c = s.car_bound().expect("bound for the taxi") as usize;
                let taxi = w.cars[c].shown.expect("in sight").0;
                let d = (s.pos[0] - taxi[0]).hypot(s.pos[1] - taxi[1]);
                assert!(
                    d < 2.6,
                    "{} began getting in {d:.1} m from the taxi's middle",
                    s.name
                );
            }
            // (Out at the kerb on: step by step, no jumps.)
            let out =
                matches!(doing, Some(Phase::AtKerb { .. })) || was[i].is_some() && state.is_some();
            if out {
                if let Some(p) = was[i] {
                    let step = (s.pos[0] - p[0]).hypot(s.pos[1] - p[1]);
                    assert!(step < 0.6, "{} moved {step:.2} m in a tick", s.name);
                }
                was[i] = Some(s.pos);
            }
        }
    }
    assert!(aboard[0] && aboard[1], "not both rode");
    assert!(got_in[0] && got_in[1], "not both got in by its door");
    assert!(stood > 0, "never stood");
}

#[test]
fn where_the_street_winds_the_taxi_stands_where_its_body_lies_along_it() {
    // B's driveway meets the street where it winds (a bend each way every 10 m, from x 40 to 80):
    // the taxi stands for B where the street runs straight for its body (it turns less than 4°
    // from its back to its front), parallel to it there.
    let bend = |x: f32| {
        if (40.0..=80.0).contains(&x) {
            0.6 * (1.0 - (std::f32::consts::TAU * (x - 40.0) / 20.0).cos())
        } else {
            0.0
        }
    };
    // (The way the street runs at x, as the town's yaw has it: 0 along +z, PI/2 along +x.)
    let heading = |x: f32| {
        let dz = -(bend(x + 0.25) - bend(x - 0.25)) / 0.5;
        1.0f32.atan2(dz)
    };
    let mut w = town_on(7, &[house(50, &[&worker("B")])], bend);
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    let mut by = Vec::new();
    let mut stood = None;
    while w.tick < end && by.is_empty() {
        w.tick_once();
        no_overlaps(&w);
        if let Some(&(_, at, yaw)) = standing(&w).first() {
            stood.get_or_insert((at, yaw));
        }
        rode(&w, 0, &mut by);
    }
    let (at, yaw) = stood.expect("never stood");
    let (back, front) = (heading(at[0] - 2.35), heading(at[0] + 2.35));
    assert!(
        (front - back).abs() < 0.07,
        "stood where the street turns {:.1}° over its body, at {at:?}",
        (front - back).abs().to_degrees()
    );
    let off = sim_core::world::wrap_angle(yaw - heading(at[0])).abs();
    let off = off.min(std::f32::consts::PI - off);
    assert!(
        off < 0.07,
        "stood {:.1}° off the street at {at:?}",
        off.to_degrees()
    );
    assert!(by.iter().any(|r| !r.1), "B never rode");
}

#[test]
fn a_house_without_a_driveway_has_its_taxi_at_the_kerb_by_its_front_path() {
    // No driveway at A's: the taxi stops at the kerb by the front path all the same.
    let mut h = house(40, &[&worker("A")]);
    h.drive = 0.0;
    let mut w = town(7, &[h]);
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    let mut by = Vec::new();
    while w.tick < end && !by.iter().any(|r: &(u32, bool)| !r.1) {
        w.tick_once();
        no_overlaps(&w);
        rode(&w, 0, &mut by);
    }
    assert!(by.iter().any(|r| !r.1), "A never rode to work");
}

#[test]
fn two_taxis_for_one_house_never_stand_on_one_stretch() {
    // Five of one house, colleagues: two taxis (four seats a taxi). They never stand at the kerb
    // within a taxi's length and a gap of each other: the second stops further on (or waits).
    let five: Vec<String> = ["A", "B", "C", "D", "E"]
        .iter()
        .map(|n| worker(n))
        .collect();
    let five: Vec<&str> = five.iter().map(String::as_str).collect();
    let mut w = town(7, &[house(40, &five)]);
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    let mut runs: Vec<Vec<(u32, bool)>> = vec![Vec::new(); 5];
    while w.tick < end && runs.iter().any(|r| !r.iter().any(|x| !x.1)) {
        w.tick_once();
        no_overlaps(&w);
        let now = standing(&w);
        for (i, a) in now.iter().enumerate() {
            for b in &now[i + 1..] {
                let d = (a.1[0] - b.1[0]).hypot(a.1[1] - b.1[1]);
                assert!(d > 4.7 + 2.0, "two taxis stood {d:.1} m apart");
            }
        }
        for (i, r) in runs.iter_mut().enumerate() {
            rode(&w, i, r);
        }
    }
    assert!(
        runs.iter().all(|r| r.iter().any(|x| !x.1)),
        "not all rode: {runs:?}"
    );
    let mut taxis: Vec<u32> = runs
        .iter()
        .flat_map(|r| r.iter().filter(|x| !x.1).map(|x| x.0))
        .collect();
    taxis.sort();
    taxis.dedup();
    assert!(taxis.len() >= 2, "one taxi for five: {taxis:?}");
}

#[test]
fn a_taxi_due_in_through_a_jammed_junction_goes_round_it_or_waits_at_the_edge_of_town() {
    // A side street joins the street at x 300, between the way out and A's house, its junction
    // jammed (cars held up in it: `taxi::jammed`). With a way round it (a loop south of the street
    // from x 360 to x 240), the taxi takes that, never into it; with none, it waits at the edge of
    // town a while rather than drive into it, and comes once it's cleared (15 minutes on).
    let road = |pts: Vec<[f32; 2]>, ends: &str| {
        let pts: Vec<String> = pts.iter().map(|p| format!("[{},{}]", p[0], p[1])).collect();
        format!(
            r#"{{"points":[{}],"width":7,"kind":"street","ends":{ends}}}"#,
            pts.join(",")
        )
    };
    let side = road(
        (0..=40).map(|z| [300.5, STREET + z as f32]).collect(),
        r#"["join","dead"]"#,
    );
    let mut lp: Vec<[f32; 2]> = (0..=20).map(|z| [360.5, STREET - z as f32]).collect();
    lp.extend((1..120).map(|x| [360.5 - x as f32, STREET - 20.0]));
    lp.extend((0..=20).rev().map(|z| [240.5, STREET - z as f32]));
    let lp = road(lp, r#"["join","join"]"#);
    for round in [false, true] {
        let more = if round {
            vec![side.clone(), lp.clone()]
        } else {
            vec![side.clone()]
        };
        let mut w = town_with(7, &[house(20, &[&worker("A")])], |_| 0.0, &more);
        let jam = w
            .travel
            .graph
            .boxes
            .iter()
            .position(|(m, _)| (m[0] - 300.5).abs() < 2.0)
            .expect("the side street's junction");
        let (m, r) = w.travel.graph.boxes[jam];
        w.taxis.jams = vec![jam];
        let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
        // (When the run was due in, and when its taxi came.)
        let mut entered = None;
        while w.tick < end && came(&w) < 2 {
            w.tick_once();
            no_overlaps(&w);
            // (Cleared a quarter of an hour after the taxi was due in.)
            if !round
                && let Some(run) = w.taxis.runs.iter().find(|r| !r.home)
                && run.enter > 0
                && w.tick >= run.enter + 15 * 20
            {
                w.taxis.jams.clear();
            }
            if let Some(run) = w.taxis.runs.iter().find(|r| !r.home)
                && run.car.is_some()
                && entered.is_none()
            {
                entered = Some((run.enter, w.tick));
            }
            for c in w.cars.iter().filter(|c| c.taxi) {
                if let Some((at, ..)) = c.shown {
                    let d = (at[0] - m[0]).hypot(at[1] - m[1]);
                    assert!(!round || d > r, "a taxi in the jammed junction at {at:?}");
                }
            }
        }
        let (enter, at) = entered.expect("no taxi came");
        if round {
            assert!(
                at <= enter + 20,
                "({round}) waited at the edge with a way round"
            );
        } else {
            assert!(
                at >= enter + 15 * 20,
                "({round}) came in at once: {} min",
                (at - enter) / 20
            );
        }
        assert!(came(&w) > 0, "({round}) A never rode");
    }
}

#[test]
fn neighbours_in_different_work_going_the_same_way_at_the_same_hour_share_a_taxi() {
    // Two houses a long way from the way out, one a manager's, one a marketer's, both starting at
    // the same hour: one taxi for both (shared by the way out, whatever the work).
    let other = r#"{"name":"Pip","age":30,"traits":["cheerful"],"job":{"career":"business.marketing","level":0}}"#;
    let mut w = town(7, &[house(20, &[&worker("Mum")]), house(60, &[other])]);
    let (mut mum, mut pip) = (Vec::new(), Vec::new());
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    let day2 = sim_core::clock::tick_at(2, 0.0).unwrap();
    while w.tick < end
        && (mum.iter().all(|x: &(u32, bool)| x.1)
            || pip.iter().all(|x: &(u32, bool)| x.1)
            || w.tick < day2)
    {
        w.tick_once();
        no_overlaps(&w);
        rode(&w, 0, &mut mum);
        rode(&w, 1, &mut pip);
    }
    let there = |r: &[(u32, bool)]| r.iter().filter(|x| !x.1).map(|x| x.0).collect::<Vec<_>>();
    assert!(!there(&mum).is_empty(), "Mum never rode: {mum:?}");
    assert!(
        there(&mum).iter().any(|r| there(&pip).contains(r)),
        "never the same taxi: {mum:?} {pip:?}"
    );
}

#[test]
fn a_riders_time_before_work_ends_when_the_taxi_comes_late_run_or_held_up() {
    // A's taxi is held at the edge of town by a jam: A leaves for it (and their free time before
    // work ends) as much later as it's held. And booked on a late run once work has begun (a game
    // begun after they'd have set off), A has a leave time all the same: no free time without end
    // before it (`world::free_before_work`).
    let road = |pts: Vec<[f32; 2]>, ends: &str| {
        let pts: Vec<String> = pts.iter().map(|p| format!("[{},{}]", p[0], p[1])).collect();
        format!(
            r#"{{"points":[{}],"width":7,"kind":"street","ends":{ends}}}"#,
            pts.join(",")
        )
    };
    let side = road(
        (0..=40).map(|z| [300.5, STREET + z as f32]).collect(),
        r#"["join","dead"]"#,
    );
    let mut w = town_with(7, &[house(20, &[&worker("A")])], |_| 0.0, &[side]);
    // (Day 1: booked on a late run, work begun.)
    let day1 = sim_core::clock::tick_at(1, 12.0 * 60.0).unwrap();
    let mut late_run = false;
    while w.tick < day1 && !late_run {
        w.tick_once();
        if let Some(b) = w.taxi_booking(0)
            && let Some(run) = w.taxis.runs.iter().find(|r| r.id == b.run && !r.home)
        {
            let start = run.stops[b.stop as usize].due;
            late_run = w.taxi_leave(0).is_some() && run.due_out > start.min(run.due_out);
            assert!(w.taxi_leave(0).is_some(), "booked with no leave time");
        }
    }
    // (Day 2: the run held at the edge of town past when it was due in.)
    let jam = w
        .travel
        .graph
        .boxes
        .iter()
        .position(|(m, _)| (m[0] - 300.5).abs() < 2.0)
        .expect("the side street's junction");
    let end = sim_core::clock::tick_at(2, 23.0 * 60.0).unwrap();
    let day2 = sim_core::clock::tick_at(2, 0.0).unwrap();
    while w.tick < day2 {
        w.tick_once();
    }
    w.taxis.jams = vec![jam];
    let mut seen = false;
    while w.tick < end && !seen {
        w.tick_once();
        let Some(b) = w.taxi_booking(0) else { continue };
        let Some(run) = w.taxis.runs.iter().find(|r| r.id == b.run && !r.home) else {
            continue;
        };
        let st = run.stops[b.stop as usize];
        if run.car.is_none() && w.tick > run.enter + 5 * 20 && st.state == StopState::Booked {
            let leave = w.taxi_leave(0).expect("a leave time");
            assert_eq!(
                leave,
                st.leave + (w.tick - run.enter),
                "not as late as it's held"
            );
            seen = true;
        }
    }
    assert!(seen, "the run was never held at the edge of town");
}

#[test]
fn a_visit_out_of_walking_reach_goes_by_taxi_there_and_back() {
    // A (no car, no bike) is sent to visit B's house 540 m along the street (a walk of six hours,
    // the taxi from the way out and the ride about half that): a taxi comes for A at the kerb,
    // puts A down at B's, and when the visit's over one brings A home.
    let mut w = town(
        7,
        &[
            house(20, &[r#"{"name":"A","age":40,"traits":["cheerful"]}"#]),
            house(560, &[r#"{"name":"B","age":40,"traits":["cheerful"]}"#]),
        ],
    );
    run_to(&mut w, 1, 9.0);
    let _ = w.apply(sim_core::Command::Visit { sim: 0, plot: 1 });
    let end = sim_core::clock::tick_at(2, 12.0 * 60.0).unwrap();
    let (mut out, mut there, mut back, mut home) = (false, None, false, false);
    while w.tick < end && !home {
        w.tick_once();
        no_overlaps(&w);
        for r in &w.taxis.runs {
            let aboard = r
                .stops
                .iter()
                .any(|s| s.sim == 0 && s.state == StopState::Aboard);
            if r.place == Some(1) && aboard {
                if r.home {
                    back = true;
                } else {
                    out = true;
                }
            }
        }
        let a = &w.sims[0];
        if out && there.is_none() && a.visiting.is_some_and(|v| v.plot == 1) {
            let door = w.plots[1].arrival_point();
            if (a.pos[0] - door[0]).hypot(a.pos[1] - door[1]) < 6.0 {
                there = Some(w.tick);
            }
        }
        home |= back && a.visiting.is_none() && a.pos[0] < 45.0 && w.taxi_booking(0).is_none();
    }
    assert!(out, "A never rode a taxi there");
    assert!(there.is_some(), "A never got to B's door");
    assert!(back, "A never rode a taxi home");
    assert!(home, "A never got home");
}

#[test]
fn a_cyclist_with_a_ride_out_of_over_an_hour_and_a_half_takes_the_taxi() {
    // C's bike would take well over `BIKE_MOST` out of town from the street's far end: C rides
    // the work taxi instead.
    let mut far = house(20, &[&worker("C")]);
    far.bike = true;
    let mut w = town(7, &[far]);
    let mut c = Vec::new();
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    while w.tick < end && !c.iter().any(|x: &(u32, bool)| !x.1) {
        w.tick_once();
        rode(&w, 0, &mut c);
    }
    assert!(c.iter().any(|x| !x.1), "C never rode the taxi to work");
}

#[test]
fn a_taxi_stands_clear_of_every_driveways_mouth_its_riders_own_too() {
    // Three houses side by side, their driveways 20 m apart: the taxi for the middle one stands
    // a car's length and more clear of each mouth (a taxi at its rider's own mouth held the
    // household's car in its driveway), between two of them.
    let mut w = town(
        7,
        &[house(20, &[]), house(40, &[&worker("A")]), house(60, &[])],
    );
    let mouths: Vec<f32> = w.drives.iter().filter_map(|d| d.s).collect();
    assert_eq!(mouths.len(), 3);
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    let mut stood = 0;
    while w.tick < end && came(&w) < 2 {
        w.tick_once();
        for (c, at, _) in standing(&w) {
            stood += 1;
            let half = w.cars[c].len / 2.0;
            for &m in &mouths {
                assert!(
                    (at[0] - m).abs() >= half + 5.0,
                    "a taxi standing at x {:.1}, across the driveway at {m:.1}",
                    at[0]
                );
            }
        }
    }
    assert!(stood > 0, "no taxi stood");
    assert!(came(&w) > 0, "A never rode");
}

#[test]
fn a_rider_called_to_the_taxi_who_doesnt_get_there_is_waited_for_the_walk_and_booked_again() {
    // B, called to the taxi's door, is kept from it (held 8 m off): the taxi waits for one on
    // their way and the walk round to its door at most (`taxi::COMING_WAIT`, a few minutes, no
    // longer: a taxi stood 20 minutes in its lane for a rider, a car out of its driveway behind
    // it), goes on without them, and they're booked on another.
    let mut w = town(7, &[house(20, &[&worker("A")]), house(60, &[&worker("B")])]);
    let end = sim_core::clock::tick_at(2, 12.0 * 60.0).unwrap();
    let mut run = None;
    let mut since: Option<u64> = None;
    let mut longest = 0;
    let mut again = false;
    while w.tick < end {
        w.tick_once();
        let b = w.taxi_booking(1);
        let state = b.and_then(|b| {
            w.taxis
                .runs
                .iter()
                .find(|r| r.id == b.run)
                .and_then(|r| r.stops.get(b.stop as usize))
                .map(|s| s.state)
        });
        if state == Some(StopState::Boarding) {
            run = run.or(b.map(|b| b.run));
            if let Some((_, at, _)) = standing(&w).first() {
                w.sims[1].pos = [at[0], at[1] + 8.0];
            }
        }
        // (How long a taxi stands, each time.)
        if standing(&w).is_empty() {
            if let Some(s) = since.take() {
                longest = longest.max(w.tick - s);
            }
        } else {
            since.get_or_insert(w.tick);
        }
        if run.is_some() && b.is_some_and(|b| Some(b.run) != run) {
            again = true;
            break;
        }
    }
    assert!(run.is_some(), "B was never called to the taxi");
    // (Five minutes for one on their way, the walk round to its door, and in.)
    assert!(
        longest <= 5 * 20 + 6 * 20 + 3 * 20,
        "a taxi stood {} minutes",
        longest / 20
    );
    assert!(again, "B wasn't booked on another taxi");
}

#[test]
fn a_taxi_doesnt_go_hours_round_a_hold_up_it_could_wait_out() {
    // The side street's junction at x 300 is held up, and the only way round it is a loop of a
    // kilometre and more south of the street: rather than that, the taxi takes the street (waiting
    // at the edge of town till it clears, a while at most), never the loop.
    let road = |pts: Vec<[f32; 2]>, ends: &str| {
        let pts: Vec<String> = pts.iter().map(|p| format!("[{},{}]", p[0], p[1])).collect();
        format!(
            r#"{{"points":[{}],"width":7,"kind":"street","ends":{ends}}}"#,
            pts.join(",")
        )
    };
    let side = road(
        (0..=40).map(|z| [300.5, STREET + z as f32]).collect(),
        r#"["join","dead"]"#,
    );
    // (Out east to x 590, then far south and back: about 1.2 km.)
    let mut lp: Vec<[f32; 2]> = (0..=20).map(|z| [360.5, STREET - z as f32]).collect();
    lp.extend((1..=220).map(|x| [360.5 + x as f32, STREET - 20.0]));
    lp.extend((1..=10).map(|z| [580.5, STREET - 20.0 - z as f32]));
    lp.extend((1..=340).rev().map(|x| [240.5 + x as f32, STREET - 30.0]));
    lp.extend((0..=30).rev().map(|z| [240.5, STREET - z as f32]));
    let lp = road(lp, r#"["join","join"]"#);
    let mut w = town_with(7, &[house(20, &[&worker("A")])], |_| 0.0, &[side, lp]);
    let jam = w
        .travel
        .graph
        .boxes
        .iter()
        .position(|(m, _)| (m[0] - 300.5).abs() < 2.0)
        .expect("the side street's junction");
    w.taxis.jams = vec![jam];
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    while w.tick < end && came(&w) < 1 {
        w.tick_once();
        if let Some(run) = w.taxis.runs.iter().find(|r| !r.home)
            && run.enter > 0
            && w.tick >= run.enter + 15 * 20
        {
            w.taxis.jams.clear();
        }
        for c in w.cars.iter().filter(|c| c.taxi) {
            if let Some((at, ..)) = c.shown {
                assert!(at[1] > STREET - 15.0, "a taxi on the loop round at {at:?}");
            }
        }
    }
    assert!(came(&w) > 0, "A never rode");
}

#[test]
fn a_run_coming_in_a_longer_way_than_planned_tells_its_riders() {
    // Planned along the street; as it comes in the side street's junction is held up and it goes
    // round by the loop south (a few minutes more): the rider's taxi is due at their kerb that
    // much later, they leave home as much later, and it's due out of town no sooner than planned
    // (later, were the way longer than its spare time).
    let road = |pts: Vec<[f32; 2]>, ends: &str| {
        let pts: Vec<String> = pts.iter().map(|p| format!("[{},{}]", p[0], p[1])).collect();
        format!(
            r#"{{"points":[{}],"width":7,"kind":"street","ends":{ends}}}"#,
            pts.join(",")
        )
    };
    let side = road(
        (0..=40).map(|z| [300.5, STREET + z as f32]).collect(),
        r#"["join","dead"]"#,
    );
    let mut lp: Vec<[f32; 2]> = (0..=20).map(|z| [360.5, STREET - z as f32]).collect();
    lp.extend((1..120).map(|x| [360.5 - x as f32, STREET - 20.0]));
    lp.extend((0..=20).rev().map(|z| [240.5, STREET - z as f32]));
    let lp = road(lp, r#"["join","join"]"#);
    let mut w = town_with(7, &[house(20, &[&worker("A")])], |_| 0.0, &[side, lp]);
    let jam = w
        .travel
        .graph
        .boxes
        .iter()
        .position(|(m, _)| (m[0] - 300.5).abs() < 2.0)
        .expect("the side street's junction");
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    // (Planned, not yet in: its times then.)
    let mut planned = None;
    let mut revised = None;
    while w.tick < end && came(&w) < 1 {
        w.tick_once();
        let Some(r) = w.taxis.runs.iter().find(|r| !r.home) else {
            continue;
        };
        let times = (r.stops[0].due, r.stops[0].leave, r.due_out);
        if r.car.is_none() && planned.is_none() {
            planned = Some(times);
            w.taxis.jams = vec![jam];
        } else if r.car.is_some() && revised.is_none() {
            revised = Some(times);
        }
    }
    let (due, leave, out) = planned.expect("never planned");
    let (due2, leave2, out2) = revised.expect("never came");
    assert!(
        due2 > due,
        "the taxi due at the kerb as planned: {due} {due2}"
    );
    assert_eq!(leave2 - leave, due2 - due, "the rider's leave not told");
    assert!(out2 >= out, "due out of town sooner than planned");
    assert!(came(&w) > 0, "A never rode");
}

#[test]
fn a_house_with_one_stand_clear_of_the_driveways_has_its_taxi_for_each_run_an_hour_apart() {
    // Driveways all along the street but one stretch past the riders' own, the only stand clear of
    // them (`DRIVE_CLEAR`, `DRIVE_LEAST`); housemates whose work starts an hour apart, a run each,
    // there and home. A run's stand is held for about when it's there, not all its time in town:
    // held so, the second run found no stand and didn't come, and its rider walked (there, or
    // home from the way out).
    let kit = r#"{"name":"Kit","age":30,"traits":["cheerful"],"job":{"career":"business.entrepreneur","level":0}}"#;
    let south = |x: i32| House {
        south: true,
        ..house(x, &[])
    };
    let mut w = town(
        7,
        &[
            south(70),
            house(80, &[]),
            south(90),
            house(100, &[kit, &worker("Mum")]),
            house(120, &[]),
            south(130),
        ],
    );
    // (From the day after the game begins: on its first morning, both too late on foot, one late
    // run takes them.)
    run_to(&mut w, 2, 0.0);
    let (mut kit_runs, mut mum_runs) = (Vec::new(), Vec::new());
    let both = |r: &[(u32, bool)]| r.iter().any(|x| !x.1) && r.iter().any(|x| x.1);
    let end = sim_core::clock::tick_at(5, 0.0).unwrap();
    while w.tick < end && !(both(&kit_runs) && both(&mum_runs)) {
        w.tick_once();
        no_overlaps(&w);
        rode(&w, 0, &mut kit_runs);
        rode(&w, 1, &mut mum_runs);
    }
    for (who, r) in [("Kit", &kit_runs), ("Mum", &mum_runs)] {
        assert!(r.iter().any(|x| !x.1), "{who} never rode to work: {r:?}");
        assert!(r.iter().any(|x| x.1), "{who} never rode home: {r:?}");
    }
    assert!(
        kit_runs.iter().all(|x| !mum_runs.contains(x)),
        "one run for both: {kit_runs:?} {mum_runs:?}"
    );
}

#[test]
fn a_taxi_held_still_where_nothing_gives_gives_its_way_up_and_its_riders_still_go_by_taxi() {
    // A car stands across the street between the way in and the rider's house, its household's
    // and never driven: every taxi coming in is held behind it. Held still LONE_STILL (30 min),
    // a taxi gives its way up; in sight, as everything is here, it stays on the road and drives
    // off once it can turn round (`cars::drive_off`: the owner's rule, nothing vanishes in view;
    // out of sight it would be gone at once). A rider it
    // was to pick up is booked again; a rider it was taking home, a long walk from where it was
    // held, isn't let out to walk it (walks of hours, all night) but waits at work for another
    // run.
    let mut car = house(300, &[]);
    car.car = true;
    let mut w = town(7, &[house(100, &[&worker("A")]), car]);
    let c = w
        .cars
        .iter()
        .position(|k| !k.taxi)
        .expect("the household's car");
    let home = (w.cars[c].spot, w.cars[c].yaw);
    let across = |w: &mut World| {
        w.cars[c].spot = [303.5, STREET];
        w.cars[c].yaw = std::f32::consts::FRAC_PI_2;
    };
    // (A's runs, to work or home, a tick ago: id, home, its taxi, A aboard.)
    let runs = |w: &World| -> Vec<(u32, bool, Option<u32>, bool)> {
        w.taxis
            .runs
            .iter()
            .filter(|r| r.place.is_none() && r.stops.iter().any(|s| s.sim == 0))
            .map(|r| {
                let aboard = r
                    .stops
                    .iter()
                    .any(|s| s.sim == 0 && s.state == StopState::Aboard);
                (r.id, r.home, r.car, aboard)
            })
            .collect()
    };
    across(&mut w);
    // Booked to work, held behind it: given up, and A booked again.
    let day3 = sim_core::clock::tick_at(3, 0.0).unwrap();
    let mut first = None;
    let mut again = false;
    while w.tick < day3 && !again {
        let before = runs(&w);
        w.tick_once();
        for &(id, _, car, _) in before.iter().filter(|b| !b.1 && b.2.is_some()) {
            if w.taxis.runs.iter().any(|r| r.id == id) {
                continue;
            }
            // (In sight, as everything is here, it never leaves the road on the spot: it drives
            // off out of town once it can turn round, `cars::drive_off`; the owner's rule.)
            let k = &w.cars[car.unwrap() as usize];
            assert!(
                k.shown.is_some(),
                "the taxi gave its way up in sight and left the road where it was"
            );
            first.get_or_insert(id);
        }
        again = first.is_some_and(|f| w.taxi_booking(0).is_some_and(|b| b.run != f));
    }
    assert!(
        first.is_some(),
        "no run to work gave its way up behind the car"
    );
    assert!(
        again,
        "A wasn't booked again after their run to work gave up"
    );
    // The street clear, A rides to work; across it again, A's run home is held behind it.
    w.cars[c].spot = home.0;
    w.cars[c].yaw = home.1;
    let end = sim_core::clock::tick_at(6, 0.0).unwrap();
    let mut rode_out = false;
    while w.tick < end && !rode_out {
        w.tick_once();
        rode_out = w.taxis.runs.iter().any(|r| {
            !r.home
                && r.stops
                    .iter()
                    .any(|s| s.sim == 0 && s.state == StopState::Aboard)
        });
    }
    assert!(rode_out, "A never rode to work with the street clear");
    across(&mut w);
    let mut held_home = None;
    while w.tick < end && held_home.is_none() {
        let before = runs(&w);
        w.tick_once();
        held_home = before
            .iter()
            .find(|b| b.1 && b.3 && b.2.is_some() && !w.taxis.runs.iter().any(|r| r.id == b.0))
            .copied();
    }
    let (id, _, car, _) = held_home.expect("A's run home never gave its way up behind the car");
    assert!(
        w.cars[car.unwrap() as usize].shown.is_some(),
        "the taxi gave its way up in sight and left the road where it was"
    );
    // (Not let out on foot a long way from home: away at work, on another run home.)
    let a = &w.sims[0];
    assert!(
        a.away_until.is_some() && w.taxi_booking(0).is_some_and(|b| b.run != id),
        "A was let out to walk home: {:?} at {:?}",
        a.current().map(|x| x.task.kind),
        a.pos
    );
}

#[test]
fn a_taxi_turned_round_in_a_hold_up_loads_where_it_was_on_the_way_it_turned_round_to() {
    // As above, a car across the street, and a side street joining it behind (at x 340, from
    // the way in): a taxi held behind it in sight turns round (`cars::unjam`), backing up out of
    // the junction it came through. Saved as it backs up and loaded, it's where it was, on the
    // way it turned round to (`Run::detour`), not back on the way it had been held up on.
    let pts: Vec<String> = (0..=40)
        .map(|z| format!("[340.5,{}]", STREET + z as f32))
        .collect();
    let side = format!(
        r#"{{"points":[{}],"width":7,"kind":"street","ends":["join","dead"]}}"#,
        pts.join(",")
    );
    let mut car = house(300, &[]);
    car.car = true;
    let mut w = town_with(7, &[house(100, &[&worker("A")]), car], |_| 0.0, &[side]);
    let c = w
        .cars
        .iter()
        .position(|k| !k.taxi)
        .expect("the household's car");
    w.cars[c].spot = [303.5, STREET];
    w.cars[c].yaw = std::f32::consts::FRAC_PI_2;
    let day3 = sim_core::clock::tick_at(3, 0.0).unwrap();
    // (On its run still: not one given up, driving off out of town.)
    let turned = |w: &World| {
        w.taxis.runs.iter().filter_map(|r| r.car).find_map(|t| {
            let k = &w.cars[t as usize];
            (k.detour.is_some() && k.progress > 20.0).then_some(t as usize)
        })
    };
    while w.tick < day3 && turned(&w).is_none() {
        w.tick_once();
    }
    let t = turned(&w).unwrap_or_else(|| {
        panic!(
            "no taxi turned round behind the car: unjams {}, none could {:?}",
            w.traffic.unjams, w.traffic.unjam_impossible
        )
    });
    let mut loaded = World::from_save_json(&common::content(), &w.save_json()).unwrap();
    w.tick_once();
    loaded.tick_once();
    let at = w.cars[t].shown.expect("the taxi on the road").0;
    let there = loaded
        .cars
        .iter()
        .filter(|k| k.taxi && k.detour.is_some())
        .filter_map(|k| k.shown)
        .map(|s| (s.0[0] - at[0]).hypot(s.0[1] - at[1]))
        .fold(f32::INFINITY, f32::min);
    assert!(
        there < 0.05,
        "loaded {there:.2} m from where it was, turned round: {:?}",
        loaded
            .cars
            .iter()
            .filter(|k| k.taxi)
            .map(|k| (k.shown.map(|s| s.0), k.detour.is_some()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_taxi_with_two_stands_close_along_one_street_stands_straight_at_both() {
    // A generated normal town (Snowdale, v4 s7) whose run home puts down neighbours a car's length
    // and a little apart along one street: it keeps to the kerb between the two and stands
    // straight along its lane at each (easing out of one undid easing into the next: it stood
    // 11 degrees across its lane there, 30 in Brightdale). Every taxi's stand till the evening,
    // as `generated_worlds`' check has it (`STAND_ANGLE`).
    let town = include_str!("../towns/snowdale_stands.json");
    let mut w = World::from_json(&common::content(), town, 160171).expect("Snowdale loads");
    let g = &w.travel.graph.clone();
    let end = sim_core::clock::tick_at(sim_core::clock::day(w.tick), 19.0 * 60.0).unwrap();
    let mut was: Vec<bool> = Vec::new();
    let mut stood = 0;
    while w.tick < end {
        w.tick_once();
        was.resize(w.cars.len(), false);
        for (c, car) in w.cars.iter().enumerate() {
            let now = car.taxi && car.standing;
            if now
                && !was[c]
                && let Some((at, yaw, ..)) = car.shown
                && let Some(rp) = g.near(at, 30.0, 1, sim_core::travel::Mode::Car).first()
            {
                stood += 1;
                let t = g.roads[rp.road as usize].tangent(rp.s);
                let off = sim_core::world::wrap_angle(yaw - t[0].atan2(t[1])).abs();
                let off = off.min(std::f32::consts::PI - off);
                assert!(
                    off <= 0.09,
                    "a taxi stood {:.0}° off its lane at {at:?}, {:.2}h",
                    off.to_degrees(),
                    sim_core::clock::hour(w.tick)
                );
            }
            was[c] = now;
        }
    }
    assert!(stood >= 5, "only {stood} stands");
}

#[test]
fn one_upstairs_booked_on_a_taxi_out_leaves_for_it_timed_from_over_the_ground() {
    // A, upstairs, is sent to visit B's house a long walk along the street: booked on a taxi,
    // A's walk out to the kerb is timed from where A is over the ground (an upstairs storey lies
    // further along the map: timed from there, A was out at the kerb two hours early, waiting,
    // and the taxi came that much later, Swan Reach).
    let mut w = town(
        7,
        &[
            house(20, &[r#"{"name":"A","age":40,"traits":["cheerful"]}"#]),
            house(560, &[r#"{"name":"B","age":40,"traits":["cheerful"]}"#]),
        ],
    );
    run_to(&mut w, 1, 9.0);
    let up = w.stair_map().storey_row(1) as f32;
    assert!(up > 0.0, "no storey upstairs");
    w.sims[0].pos[1] += up;
    let _ = w.apply(sim_core::Command::Visit { sim: 0, plot: 1 });
    let end = w.tick + 30 * 20;
    let mut stop = None;
    while w.tick < end && stop.is_none() {
        w.tick_once();
        stop = w
            .taxis
            .runs
            .iter()
            .filter(|r| r.place == Some(1) && !r.home)
            .flat_map(|r| r.stops.iter().filter(|s| s.sim == 0))
            .map(|s| (s.due, s.leave))
            .next();
    }
    let (due, leave) = stop.expect("A wasn't booked on a taxi there");
    let early = due.saturating_sub(leave) / 20;
    assert!(
        early <= 20,
        "A was to leave {early} min before the taxi's due"
    );
}

#[test]
fn one_booked_on_a_taxi_while_out_at_another_house_isnt_called_home_hours_early() {
    // A, booked on a taxi to work, is at a house along the street: the lead for the walk out
    // of the house (`indoor_lead`) is for in and about their own, not a walk across the map from
    // elsewhere (timed so from a visit, a cyclist came home on their bike an hour early and was
    // still on it at the kerb two hours on, Swan Reach). From there, as from their own door.
    let mut w = town(7, &[house(20, &[&worker("A")]), house(500, &[])]);
    let end = sim_core::clock::tick_at(3, 0.0).unwrap();
    while w.tick < end && w.taxi_leave(0).is_none() {
        w.tick_once();
    }
    let door = w.plots[0].arrival_point();
    w.sims[0].pos = door;
    let at_door = w.taxi_leave(0).expect("A booked on a taxi to work");
    w.sims[0].pos = w.plots[1].arrival_point();
    let away = w.taxi_leave(0).expect("still booked");
    assert_eq!(
        away,
        at_door,
        "out at another house, A leaves {} min sooner",
        at_door.saturating_sub(away) / 20
    );
}

#[test]
fn a_cyclist_out_on_a_visit_when_work_calls_doesnt_ride_out_on_the_bike_their_way_isnt_by() {
    // C's way out is by the work taxi (by bike, well over `BIKE_MOST`). Sent on a visit along
    // the street just before leaving for the taxi, C sets off on the bike; leaving time comes
    // on the way: the ride cut short, off to the kerb for the taxi. (On, they rode hours out of
    // town on it, Hollyshaw and Stationgate.)
    let mut far = house(20, &[&worker("C")]);
    far.bike = true;
    let mut w = town(
        7,
        &[
            far,
            house(140, &[r#"{"name":"D","age":40,"traits":["cheerful"]}"#]),
        ],
    );
    run_to(&mut w, 2, 0.0);
    let end = sim_core::clock::tick_at(3, 12.0 * 60.0).unwrap();
    let leave = loop {
        w.tick_once();
        assert!(w.tick < end, "C never booked on a taxi to work");
        if let Some(l) = w.taxi_leave(0)
            && l > w.tick + 10 * 20
        {
            break l;
        }
    };
    while w.tick + 10 * 20 < leave {
        w.tick_once();
    }
    let _ = w.apply(sim_core::Command::Visit { sim: 0, plot: 1 });
    let mut rode_off = false;
    let mut to_work = false;
    let mut aboard = Vec::new();
    while w.tick < end && aboard.iter().all(|x: &(u32, bool)| x.1) {
        w.tick_once();
        let c = &w.sims[0];
        rode_off |= c.riding.is_some() && c.away_until.is_none();
        let work = c
            .current()
            .is_some_and(|a| matches!(a.task.kind, sim_core::world::TaskKind::Work));
        to_work |= work;
        assert!(
            !(work && c.riding.is_some()),
            "C rode to work on a bike their way out isn't by, at {:?}",
            c.pos
        );
        rode(&w, 0, &mut aboard);
    }
    assert!(rode_off, "C never set off on the bike");
    assert!(to_work, "C never left for work");
    assert!(aboard.iter().any(|x| !x.1), "C never rode the taxi to work");
}
