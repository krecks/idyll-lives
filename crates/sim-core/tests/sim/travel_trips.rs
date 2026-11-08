//! Trips by car between hamlets (`cars::Trip`): a resident off to a place in another hamlet,
//! beyond walking range, with the household's car at home drives there along the roads, parks at
//! the public parking spot near it (the town file's `parking`), walks to the place, and when
//! they go home walks back to the car and drives home, the car never off the road. The trip is
//! saved with the car (save v35) and goes on after a load.

use crate::common;

use common::hamlets::{Hamlets, Place, hamlets_with_parking, with_cars};
use serde_json::Value;
use sim_core::cars::CarState;
use sim_core::travel::Mode;
use sim_core::{Command, World};

/// Two hamlets 330 m apart along the country road, cars on the driveways, and a parking bay
/// off the second hamlet's street by its park (`hamlets::hamlets_with_parking`).
fn world(seed: u64) -> (World, Hamlets, u32) {
    let content = common::content();
    let h = hamlets_with_parking(
        &content,
        seed,
        3,
        &[Place { x: 0, z: 0 }, Place { x: 330, z: 60 }],
        (512, 160),
        Some(1),
    );
    let town: Value = serde_json::from_str(&with_cars(&h)).unwrap();
    let park = h.plots[1]
        .clone()
        .find(|&p| town["plots"][p]["public"].as_bool() == Some(true))
        .expect("a park in the second hamlet");
    let mut w = World::from_json(&content, &town.to_string(), seed as u32).expect("the town loads");
    w.autonomy = false;
    for hh in 0..w.households.len() as u32 {
        w.apply(Command::SetAutonomy {
            enabled: false,
            household: Some(hh),
        })
        .unwrap();
    }
    (w, h, park as u32)
}

/// How far a car's middle may be from its trip's parking bay's (m) and be in it or turning into
/// it off the road: half the bay's 6 m length, the half metre of verge between it and the road and
/// some for the swing in.
const IN_THE_BAY: f32 = 4.0;

/// How far car `c`, moving, is off the middle of the road it's on (and half that road's width),
/// unless it's on its own driveway or in (or turning into or out of) its trip's parking bay.
fn off_the_road(w: &World, c: usize) -> Option<(f32, f32)> {
    let car = &w.cars[c];
    let (at, _, moving, _) = car.shown?;
    if !moving || w.plot_at(at[0].floor() as i32, at[1].floor() as i32) == Some(car.plot) {
        return None;
    }
    let bay = car
        .trip
        .map_or(f32::INFINITY, |t| sim_core::travel::dist(t.at, at));
    if bay <= IN_THE_BAY {
        return None;
    }
    let p = *w.travel.graph.near(at, 20.0, 1, Mode::Car).first()?;
    Some((p.off, w.travel.graph.roads[p.road as usize].width / 2.0))
}

#[test]
fn a_visit_to_the_next_hamlet_goes_by_car_and_back() {
    let (mut w, h, park) = world(5);
    let content = common::content();
    // A grown-up of the first hamlet who owns a car.
    let (c, i) = w
        .cars
        .iter()
        .enumerate()
        .find_map(|(c, car)| {
            let home = w.households[car.household as usize].plot? as usize;
            h.plots[0].contains(&home).then_some(())?;
            car.owners.first().map(|&o| (c, o as usize))
        })
        .expect("a car owner in the first hamlet");
    let home = w.households[w.sims[i].household as usize].plot.unwrap();
    let parked_at = w.cars[c].spot;
    w.apply(Command::Visit {
        sim: i as u32,
        plot: park,
    })
    .unwrap();
    let mut seen = Vec::new();
    let mut worst: f32 = 0.0;
    let note = |w: &World, worst: &mut f32, seen: &mut Vec<&'static str>| {
        let state = match w.cars[c].state {
            CarState::Leaving if w.cars[c].trip.is_some() => "out",
            CarState::Out => "parked there",
            CarState::Arriving if w.cars[c].trip.is_some() => "back",
            CarState::Parked => "home",
            _ => "other",
        };
        if seen.last() != Some(&state) {
            seen.push(state);
        }
        if let Some((off, half)) = off_the_road(w, c) {
            *worst = worst.max(off);
            assert!(
                off <= half,
                "the car is {off:.2} m off the road's middle (half its width {half})"
            );
        }
    };
    // There: it drives off, parks, and they walk to the park.
    let mut arrived = None;
    for t in 0..common::hours(20.0) {
        w.tick_once();
        let before = seen.len();
        note(&w, &mut worst, &mut seen);
        if seen.len() != before {
            eprintln!(
                "{:.1} min: {}",
                t as f32 * sim_core::MINUTES_PER_TICK,
                seen.last().unwrap()
            );
        }
        let s = &w.sims[i];
        if w.cars[c].state == CarState::Out
            && w.plot_at(s.tile().0, s.tile().1) == Some(park)
            && s.visiting.is_some_and(|v| v.plot == park)
        {
            arrived = Some(t);
            break;
        }
    }
    let arrived = arrived.unwrap_or_else(|| panic!("never got there: {seen:?}"));
    assert_eq!(seen, ["home", "out", "parked there"], "{seen:?}");
    let trip = w.cars[c].trip.expect("on a trip");
    assert_eq!((trip.driver, trip.plot), (i as u32, park));
    // Far quicker than the walk (330 m at 1.5 m a minute is over three and a half hours; cars
    // keep the speeds they're drawn at, so the drive is a good half hour).
    let minutes = arrived as f32 * sim_core::MINUTES_PER_TICK;
    let walk = w.travel.eta(
        w.plots[home as usize].arrival_point(),
        w.plots[park as usize].arrival_point(),
        Mode::Foot,
        1.5,
    );
    assert!(
        minutes < walk / 2.0,
        "{minutes:.0} minutes to get there, {walk:.0} on foot"
    );
    // Saved and loaded there: the car is still parked out, the trip the same.
    let mut w = common::reload(&w, &content);
    assert_eq!(w.cars[c].state, CarState::Out);
    assert_eq!(w.cars[c].trip, Some(trip));
    // Home: back to the car, and the drive home.
    w.apply(Command::GoHome { sim: i as u32 }).unwrap();
    let mut home_again = false;
    for _ in 0..common::hours(20.0) {
        w.tick_once();
        note(&w, &mut worst, &mut seen);
        let s = &w.sims[i];
        if w.cars[c].state == CarState::Parked
            && w.plot_at(s.tile().0, s.tile().1) == Some(home)
            && s.current().is_none()
        {
            home_again = true;
            break;
        }
    }
    assert!(home_again, "never got home: {seen:?}");
    assert_eq!(
        seen,
        ["home", "out", "parked there", "back", "home"],
        "{seen:?}"
    );
    assert!(w.cars[c].trip.is_none());
    assert!(sim_core::travel::dist(w.cars[c].spot, parked_at) < 0.01);
    assert!(w.sims[i].visiting.is_none());
    eprintln!(
        "there in {minutes:.0} minutes ({walk:.0} on foot); at most {worst:.2} m off the road's middle"
    );
}

#[test]
fn a_cosy_town_never_drives_to_a_visit() {
    // The same town without a size: no road network, no trips; they walk.
    let (w, _, park) = world(5);
    let content = common::content();
    let mut town: Value = serde_json::from_str(&w.save_json()).unwrap();
    town["openWorld"]["size"] = Value::Null;
    let mut w = World::from_save_json(&content, &town.to_string()).unwrap();
    assert!(!w.travel.open);
    let (c, i) = w
        .cars
        .iter()
        .enumerate()
        .find_map(|(c, car)| car.owners.first().map(|&o| (c, o)))
        .unwrap();
    w.apply(Command::Visit { sim: i, plot: park }).unwrap();
    for _ in 0..common::hours(5.0) {
        w.tick_once();
        assert!(w.cars[c].trip.is_none() && w.cars[c].state == CarState::Parked);
    }
}

/// The trip in the UI's view of a resident (`SimView.trip`), their panel open: where to and about
/// when, while on the way; gone once there.
fn trip_in_view(w: &World, i: usize) -> Option<(u64, u64)> {
    let mut sync = sim_core::view::UiSync::new();
    sync.inspect(Some(i as u32));
    let view: Value = serde_json::from_str(&sync.update_json(w)).unwrap();
    let s = view["sims"]
        .as_array()?
        .iter()
        .find(|s| s["id"].as_u64() == Some(i as u64))?;
    let t = &s["trip"];
    Some((t["to"].as_u64()?, t["at"].as_u64()?))
}

#[test]
fn the_ui_says_where_a_trip_goes_and_about_when_they_are_there() {
    let (mut w, h, park) = world(5);
    let i = w
        .cars
        .iter()
        .find_map(|car| {
            let home = w.households[car.household as usize].plot? as usize;
            h.plots[0].contains(&home).then_some(())?;
            car.owners.first().map(|&o| o as usize)
        })
        .expect("a car owner in the first hamlet");
    assert_eq!(
        trip_in_view(&w, i),
        None,
        "at home, not on the way anywhere"
    );
    w.apply(Command::Visit {
        sim: i as u32,
        plot: park,
    })
    .unwrap();
    let mut first = None;
    for _ in 0..common::hours(20.0) {
        w.tick_once();
        if first.is_none() {
            first = trip_in_view(&w, i);
        }
        let s = &w.sims[i];
        if w.plot_at(s.tile().0, s.tile().1) == Some(park) {
            break;
        }
    }
    let (to, at) = first.expect("on the way: the UI has the trip");
    assert_eq!(to, park as u64);
    assert_eq!(at % 5, 0, "in 5-minute steps");
    // About when they got there (an estimate made as they set off, by the roads, without the
    // car's stops or the walk from the parking spot to the park).
    let now = sim_core::clock::minute_of_day(w.tick) as i64;
    let off = (now - at as i64)
        .rem_euclid(24 * 60)
        .min((at as i64 - now).rem_euclid(24 * 60));
    eprintln!("estimated {at}, there at {now} ({off} minutes off)");
    assert!(off <= 30, "estimated {at}, there at {now}");
    assert_eq!(trip_in_view(&w, i), None, "there: no longer on the way");
}

#[test]
fn a_walk_past_a_car_parked_out_on_a_trip_goes_round_it() {
    let (mut w, h, park) = world(5);
    let (c, i) = w
        .cars
        .iter()
        .enumerate()
        .find_map(|(c, car)| {
            let home = w.households[car.household as usize].plot? as usize;
            h.plots[0].contains(&home).then_some(())?;
            car.owners.first().map(|&o| (c, o as usize))
        })
        .expect("a car owner in the first hamlet");
    w.apply(Command::Visit {
        sim: i as u32,
        plot: park,
    })
    .unwrap();
    let end = w.tick + common::hours(20.0);
    while w.cars[c].state != CarState::Out {
        w.tick_once();
        assert!(w.tick < end, "never parked out there");
    }
    let trip = w.cars[c].trip.unwrap();
    let (len, wid) = (w.cars[c].len, w.cars[c].wid);
    // Someone of the second hamlet walks past it from one end of it to the other: from the road
    // across its bay (it's parked nose in, square to the road) to the ground behind.
    let j = h.plots[1]
        .clone()
        .find_map(|p| {
            let hh = w
                .households
                .iter()
                .position(|hh| hh.plot == Some(p as u32))?;
            w.sims.iter().position(|s| s.household == hh as u32)
        })
        .expect("someone in the second hamlet");
    let (fs, fc) = trip.yaw.sin_cos();
    let along = |k: f32| [trip.at[0] + fs * k, trip.at[1] + fc * k];
    w.sims[j].pos = along(-6.0);
    let to = along(6.0);
    w.apply(Command::MoveTo {
        sim: j as u32,
        x: to[0],
        z: to[1],
    })
    .unwrap();
    let mut closest = f32::INFINITY;
    for _ in 0..common::secs(100.0) {
        w.tick_once();
        assert_eq!(w.cars[c].state, CarState::Out, "the car left");
        let p = w.sims[j].pos;
        let (dx, dz) = (p[0] - trip.at[0], p[1] - trip.at[1]);
        let (a, b) = (
            (dx * fs + dz * fc).abs() - len / 2.0,
            (dx * fc - dz * fs).abs() - wid / 2.0,
        );
        closest = closest.min(a.max(0.0).hypot(b.max(0.0)) + a.max(b).min(0.0));
        if sim_core::travel::dist(p, to) < 0.5 {
            break;
        }
    }
    assert!(
        sim_core::travel::dist(w.sims[j].pos, to) < 0.5,
        "never got past it: at {:?}",
        w.sims[j].pos
    );
    // (Their middle: a tile's worth of the car at its edge is walkable, as at home.)
    assert!(
        closest >= 0.0,
        "walked {:.2} m into the car parked there",
        -closest
    );
}
