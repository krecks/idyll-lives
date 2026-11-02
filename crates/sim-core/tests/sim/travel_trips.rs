//! Trips by car between hamlets (`cars::Trip`): a resident off to a place in another hamlet,
//! beyond walking range, with the household's car at home drives there along the roads, parks at
//! the public parking spot near it (the town file's `parking`), walks to the place, and when
//! they go home walks back to the car and drives home, the car never off the road. The trip is
//! saved with the car (save v35) and goes on after a load.

use crate::common;

use common::hamlets::{Hamlets, Place, hamlets, with_cars};
use serde_json::{Value, json};
use sim_core::cars::CarState;
use sim_core::travel::Mode;
use sim_core::{Command, World};

/// Two hamlets 330 m apart along the country road, cars on the driveways, and a parking spot
/// on the second hamlet's street by its park.
fn world(seed: u64) -> (World, Hamlets, u32) {
    let content = common::content();
    let h = hamlets(
        &content,
        seed,
        3,
        &[Place { x: 0, z: 0 }, Place { x: 330, z: 60 }],
        (512, 160),
    );
    let mut town: Value = serde_json::from_str(&with_cars(&h)).unwrap();
    let park = h.plots[1]
        .clone()
        .find(|&p| town["plots"][p]["public"].as_bool() == Some(true))
        .expect("a park in the second hamlet");
    let p = &town["plots"][park];
    let num = |k: &str| p[k].as_f64().unwrap();
    let x = num("x") + num("w") / 2.0;
    let (street, ..) = h.streets[1];
    // At the street's edge on the park's side, along it.
    let side = if num("z") > street { 1.0 } else { -1.0 };
    town["parking"] =
        json!([{"x": x, "z": street + side * 2.0, "rot": std::f64::consts::FRAC_PI_2, "area": 1}]);
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

/// How far car `c`, moving, is off the middle of the road it's on (and half that road's width),
/// unless it's on its own driveway.
fn off_the_road(w: &World, c: usize) -> Option<(f32, f32)> {
    let car = &w.cars[c];
    let (at, _, moving, _) = car.shown?;
    if !moving || w.plot_at(at[0].floor() as i32, at[1].floor() as i32) == Some(car.plot) {
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
    for t in 0..20 * 60 * 20 {
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
    for _ in 0..20 * 60 * 20 {
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
    for _ in 0..20 * 60 * 5 {
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
    for _ in 0..20 * 60 * 20 {
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
