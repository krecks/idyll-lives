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
    /// How deep its driveway runs from the street (m to its inner end).
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
    let road: Vec<String> = (0..=599).map(|x| format!("[{x}.5,{STREET}]")).collect();
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
    let json = format!(
        r#"{{"width":600,"depth":80,"size":"normal",
        "exits":[{{"x":599.5,"z":{STREET},"kind":"road"}}],
        "roads":[{road}],"meta":{{"roads":[{road}]}},
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
    // On time: at work by its start (within the late grace).
    let grace = w.content.career_rules.late_grace_minutes as i64 * 20;
    assert!(!w.taxis.arrivals.is_empty());
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
        w.taxis.runs.is_empty() && w.taxis.arrivals.is_empty(),
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
    while loaded.tick < end && loaded.taxis.arrivals.len() < 2 {
        loaded.tick_once();
        no_overlaps(&loaded);
    }
    assert!(
        loaded.taxis.arrivals.len() >= 2,
        "the riders never got to work"
    );
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
    while w.tick < end && (by.0.is_empty() || by.1.is_empty()) {
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
    assert!(!by.0.is_empty() && !by.1.is_empty(), "not both to school");
    assert_eq!(by.0, by.1, "not the same taxi");
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
            if sent && car_done && !w.taxis.arrivals.is_empty() {
                done = Some(w.tick);
            }
        }
        assert!(sent, "({home}, {early}) the taxi never stood at the kerb");
        assert!(done.is_some(), "({home}, {early}) not both on their way");
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
