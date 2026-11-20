//! Car parks (a real town's public parking, `parking.ts`): rows of nose-in bays 3 m apart, square
//! to the street beyond its verge, on one apron with a P sign. A car parks at exactly the bay's
//! pose as the town file gives it (the one the renderer draws), its body inside the bay's lines.
//! A row fills and empties: its drivers get out between the cars and walk on, come back, get in,
//! back out one by one and drive off. A flats building's bays side by side in its front strip see
//! their cars off to work in the same hour and home again, none held up for good.

use crate::common;
use crate::travel_cars::cars_overlap;

use common::hamlets::{Place, hamlets_with_car_park, with_cars};
use serde_json::{Value, json};
use sim_core::cars::{BAY_DEEP, BAY_LINES, BAY_WIDE, CarState};
use sim_core::{Command, World, clock};

/// How many bays the car park of `row_town` has.
const ROW: usize = 6;

/// Two hamlets 330 m apart along the country road, four plots a side, cars on the driveways, and
/// a car park of `ROW` bays by the second hamlet's park, each turned `slant` (rad) off square to
/// the street; free will off. The world, the park and, of the first hamlet's cars, `ROW` of them
/// with a driver each (car, resident).
fn row_town(seed: u64, slant: f64) -> (World, u32, Vec<(usize, usize)>) {
    let content = common::content();
    let h = hamlets_with_car_park(
        &content,
        seed,
        4,
        &[Place { x: 0, z: 0 }, Place { x: 330, z: 60 }],
        (600, 160),
        Some(1),
        ROW,
    );
    let mut town: Value = serde_json::from_str(&with_cars(&h)).unwrap();
    // (Nobody works: none of them drives off to work instead.)
    for s in town["sims"].as_array_mut().unwrap() {
        s.as_object_mut().unwrap().remove("job");
    }
    for bay in town["parking"].as_array_mut().unwrap() {
        bay["yaw"] = json!(bay["yaw"].as_f64().unwrap() + slant);
    }
    let park = h.plots[1]
        .clone()
        .find(|&p| town["plots"][p]["public"].as_bool() == Some(true))
        .expect("a park in the second hamlet") as u32;
    let mut w = World::from_json(&content, &town.to_string(), seed as u32).expect("the town loads");
    w.autonomy = false;
    for hh in 0..w.households.len() as u32 {
        w.apply(Command::SetAutonomy {
            enabled: false,
            household: Some(hh),
        })
        .unwrap();
    }
    let drivers: Vec<(usize, usize)> = w
        .cars
        .iter()
        .enumerate()
        .filter_map(|(c, car)| {
            let home = w.households[car.household as usize].plot? as usize;
            h.plots[0].contains(&home).then_some(())?;
            (!car.no_way).then_some(())?;
            car.owners.first().map(|&o| (c, o as usize))
        })
        .take(ROW)
        .collect();
    assert_eq!(
        drivers.len(),
        ROW,
        "{ROW} cars with a driver in the first hamlet"
    );
    assert_eq!(w.travel.file.parking.len(), ROW);
    assert!(w.bays_by_a_mouth().is_empty(), "no bay by a driveway");
    (w, park, drivers)
}

/// How far car `c`'s body, parked out, reaches past the lines of the bay it is parked in (m; 0 or
/// less: inside them), and the bay; None when it is parked in none.
fn past_the_lines(w: &World, c: usize) -> Option<(f32, usize)> {
    let car = &w.cars[c];
    let (at, yaw, ..) = car.shown?;
    let k = w
        .travel
        .file
        .parking
        .iter()
        .position(|p| (p.x - at[0]).hypot(p.z - at[1]) < 0.5)?;
    let bay = &w.travel.file.parking[k];
    let (fs, fc) = bay.yaw.sin_cos();
    let (s, co) = yaw.sin_cos();
    let mut worst = f32::NEG_INFINITY;
    for (a, b) in [(1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (-1.0, 1.0)] {
        // (A corner of the body, then in the bay's frame: along its yaw, and across it.)
        let p = [
            at[0] + a * s * car.len / 2.0 + b * co * car.wid / 2.0,
            at[1] + a * co * car.len / 2.0 - b * s * car.wid / 2.0,
        ];
        let (dx, dz) = (p[0] - bay.x, p[1] - bay.z);
        let along = (dx * fs + dz * fc).abs() - BAY_LINES[1] / 2.0;
        let across = (dx * fc - dz * fs).abs() - BAY_LINES[0] / 2.0;
        worst = worst.max(along).max(across);
    }
    Some((worst, k))
}

/// What car `c` is doing, as a trip sees it.
fn doing(w: &World, c: usize) -> &'static str {
    let car = &w.cars[c];
    match car.state {
        CarState::Parked if car.trip.is_none() => "home",
        CarState::Boarding | CarState::Parked => "getting in",
        CarState::Leaving if car.trip.is_some() => "out",
        CarState::Out => "parked there",
        CarState::Arriving if car.trip.is_some() => "back",
        _ => "other",
    }
}

/// Ticks `w` on till `done` (at most `hours`), every tick checking no two cars' bodies overlap
/// (one of them on the move) and every car parked out stands inside its bay's lines, and noting
/// what each car did in `seen` (`doing`, by car); false if it never was.
fn run(
    w: &mut World,
    hours: f32,
    seen: &mut [Vec<&'static str>],
    done: impl Fn(&World) -> bool,
) -> bool {
    for _ in 0..common::hours(hours) {
        w.tick_once();
        for (c, seen) in seen.iter_mut().enumerate() {
            let now = doing(w, c);
            if seen.last() != Some(&now) {
                seen.push(now);
            }
        }
        let (o, a, b) = cars_overlap(w);
        assert!(o <= 0.05, "cars {a} and {b} overlap by {o:.2} m");
        for c in 0..w.cars.len() {
            if w.cars[c].state == CarState::Out {
                let (past, k) = past_the_lines(w, c).expect("parked out in a bay");
                assert!(past <= 0.0, "car {c} is {past:.2} m past bay {k}'s lines");
            }
        }
        if done(w) {
            return true;
        }
    }
    false
}

#[test]
fn every_car_body_fits_a_bays_lines_and_the_lines_a_bay() {
    let content: Value = serde_json::from_str(&common::content()).unwrap();
    let bodies = content["carBodies"].as_array().expect("the car bodies");
    assert!(!bodies.is_empty());
    for b in bodies {
        let (len, wid) = (b["len"].as_f64().unwrap(), b["wid"].as_f64().unwrap());
        assert!(
            len <= f64::from(BAY_LINES[1]) - 0.2 && wid <= f64::from(BAY_LINES[0]) - 0.6,
            "{b}: longer or wider than a bay's lines and some"
        );
    }
    const { assert!(BAY_LINES[0] < BAY_WIDE && BAY_LINES[1] < BAY_DEEP) };
}

#[test]
fn a_row_of_bays_fills_and_empties_each_car_parked_as_the_town_file_says() {
    fill_and_empty(0.0);
}

#[test]
fn a_car_stands_at_its_bays_own_yaw_where_the_bay_is_not_square_to_the_street() {
    // (A car park's bays square to a straight stretch, the street bending a little past it: the
    // way in from the road nearest each is a few degrees off its lines.)
    fill_and_empty(0.12);
}

/// The drivers of `row_town` (its bays `slant` off square) all drive to the other hamlet's park
/// at once, each parking in a bay of its own at the bay's pose, and all go home at once.
fn fill_and_empty(slant: f64) {
    let content = common::content();
    let (mut w, park, drivers) = row_town(5, slant);
    let homes: Vec<[f32; 2]> = drivers.iter().map(|&(c, _)| w.cars[c].spot).collect();
    // All six off to the other hamlet's park at once: each drives there and parks in a bay of
    // its own.
    for &(_, i) in &drivers {
        w.apply(Command::Visit {
            sim: i as u32,
            plot: park,
        })
        .unwrap();
    }
    let mut seen = vec![Vec::new(); w.cars.len()];
    let full = |w: &World| {
        drivers
            .iter()
            .all(|&(c, _)| w.cars[c].state == CarState::Out)
    };
    assert!(
        run(&mut w, 6.0, &mut seen, full),
        "the row never filled: {:?}",
        drivers
            .iter()
            .map(|&(c, i)| (
                &seen[c],
                w.sims[i].visiting.map(|v| v.plot),
                w.sims[i].pos,
                format!(
                    "{:?}",
                    w.sims[i].current().map(|a| (&a.task.kind, &a.phase))
                )
                .chars()
                .take(100)
                .collect::<String>()
            ))
            .collect::<Vec<_>>()
    );
    // Every bay taken, each car at its bay's pose as the town file gives it, its driver out of it.
    let mut taken = Vec::new();
    for &(c, i) in &drivers {
        let car = &w.cars[c];
        let trip = car.trip.expect("on a trip");
        let k = w
            .travel
            .file
            .parking
            .iter()
            .position(|p| p.x == trip.at[0] && p.z == trip.at[1])
            .unwrap_or_else(|| panic!("car {c} parked at {:?}, no bay's middle", trip.at));
        assert_eq!(trip.yaw, w.travel.file.parking[k].yaw, "car {c}");
        let (at, yaw, moving, _) = car.shown.expect("drawn");
        assert_eq!((at, yaw, moving), (trip.at, trip.yaw, false), "car {c}");
        assert!(!car.aboard.contains(&(i as u32)), "{i} got out of car {c}");
        taken.push(k);
    }
    taken.sort_unstable();
    assert_eq!(taken, (0..ROW).collect::<Vec<_>>(), "a car in every bay");
    // Saved and loaded there: the row still parked as it was.
    let mut w = common::reload(&w, &content);
    for &(c, _) in &drivers {
        assert_eq!(w.cars[c].state, CarState::Out);
    }
    // All six home at once: back to the cars, in, out of the bays and off, each home to its own
    // driveway.
    for &(_, i) in &drivers {
        w.apply(Command::GoHome { sim: i as u32 }).unwrap();
    }
    let home = |w: &World| {
        drivers.iter().zip(&homes).all(|(&(c, _), spot)| {
            let car = &w.cars[c];
            car.state == CarState::Parked && car.trip.is_none() && car.spot == *spot
        })
    };
    let start = w.tick;
    let gone = run(&mut w, 12.0, &mut seen, home);
    assert!(
        gone,
        "not all home again: {:?}",
        drivers
            .iter()
            .map(|&(c, _)| (w.cars[c].state, w.cars[c].trip.is_some()))
            .collect::<Vec<_>>()
    );
    for &(c, _) in &drivers {
        assert_eq!(
            seen[c],
            ["home", "out", "parked there", "back", "home"],
            "car {c}"
        );
    }
    eprintln!(
        "the row emptied and every car home in {:.0} minutes",
        (w.tick - start) as f32 * sim_core::MINUTES_PER_TICK
    );
}

/// How many flats `flats_row` has, each with its bay in the front strip.
const FLATS: usize = 6;

/// A flats building of `FLATS` flats on the ground floor side by side along a street (along x, its
/// middle at z = 34.5, 7 m wide), the building's lot from the street back 26 m, and in its front
/// strip a bay for each flat side by side (3 m apart, square to the street, as a flats lot's front
/// strip has them: `tests/towns/flats.json`), each with the flat's car on it, every other one
/// nose in. A grown-up lives in each flat and works `career` from its first level, so all six set off
/// for work by car in the same hour.
fn flats_row(content: &str, career: &str) -> World {
    let width = 60;
    let road: Vec<Value> = (0..width).map(|x| json!([x as f64 + 0.5, 34.5])).collect();
    let mut plots = vec![json!({
        "name": "Mill Row", "x": 2, "z": 4, "w": 48, "d": 26, "public": false,
        "entry": [38.5, 18.5], "area": 0, "kind": "flats",
    })];
    let (mut households, mut sims, mut drives, mut cars) = (vec![], vec![], vec![], vec![]);
    for k in 0..FLATS {
        let x = 3 + 7 * k as i64;
        plots.push(json!({
            "name": format!("Flat {}, Mill Row", k + 1), "x": x, "z": 8, "w": 7, "d": 9,
            "public": false, "entry": [38.5, 18.5], "area": 0, "kind": "flat", "of": 0,
            "storey": 0, "door": [x + 3, 17], "inside": [x as f64 + 3.5, 16.5],
        }));
        households.push(json!({"name": format!("F{k}"), "plot": k + 1}));
        sims.push(json!({
            "name": format!("D{k}"), "household": k, "x": x as f64 + 3.5, "z": 12.5,
            "traits": ["cheerful"], "age": 35,
            "job": {"career": career, "level": 0},
        }));
        let bx = 5.5 + 3.0 * k as f64;
        let (len, inner) = (4.6, 21.5);
        drives.push(json!({
            "plot": k + 1, "x": bx, "z": inner, "front": 1, "street": 34.5, "road": 0, "s": bx - 0.5,
        }));
        // (Facing the street, yaw 0, or nose in, away from it.)
        let yaw = if k % 2 == 0 {
            0.0
        } else {
            std::f64::consts::PI
        };
        cars.push(json!({
            "plot": k + 1, "x": bx, "z": inner + 0.45 + len / 2.0, "yaw": yaw, "street": 34.5,
            "len": len, "wid": 1.88, "body": 0, "paint": k,
        }));
    }
    let town = json!({
        "width": width, "depth": 50,
        "walls": [], "doors": [], "windows": [], "objects": [], "relationships": [],
        "plots": plots, "households": households, "sims": sims, "drives": drives, "cars": cars,
        "roads": [{"points": road, "width": 7, "kind": "street", "ends": ["out", "out"]}],
        "exits": [{"x": 0.5, "z": 34.5, "kind": "road"}, {"x": width as f64 - 0.5, "z": 34.5, "kind": "road"}],
        "areas": [{"name": "Mill End", "kind": "hamlet", "x": 0, "z": 0, "w": width, "d": 50}],
    });
    let w = World::from_json(content, &town.to_string(), 3).expect("the town loads");
    assert_eq!(w.cars.len(), FLATS);
    assert!(w.cars.iter().all(|c| !c.no_way), "every car has a way out");
    w
}

#[test]
fn a_flats_buildings_bays_side_by_side_see_their_cars_off_in_the_same_hour() {
    let content = common::content();
    // (Print reporting: on weekdays from nine.)
    let careers: Value = serde_json::from_str(&content).unwrap();
    let career = careers["careerCategories"][0]["tracks"][0]["id"]
        .as_str()
        .expect("a career")
        .to_string();
    let mut w = flats_row(&content, &career);
    // Two days: each morning all six set off for work by car within the hour, and each evening
    // they come home to their bays. None is held up for good: at three in the morning each is
    // parked at home again.
    let spots: Vec<[f32; 2]> = w.cars.iter().map(|c| c.spot).collect();
    let mut seen = vec![Vec::new(); w.cars.len()];
    let mut left: Vec<Vec<u64>> = vec![Vec::new(); w.cars.len()];
    let end = clock::tick_at(clock::day(w.tick) + 2, 3.0 * 60.0).unwrap();
    let mut was: Vec<CarState> = w.cars.iter().map(|c| c.state).collect();
    while w.tick < end {
        w.tick_once();
        let (o, a, b) = cars_overlap(&w);
        assert!(o <= 0.05, "cars {a} and {b} overlap by {o:.2} m");
        for (c, car) in w.cars.iter().enumerate() {
            if car.state != was[c] {
                if car.state == CarState::Leaving {
                    left[c].push(w.tick);
                }
                was[c] = car.state;
                seen[c].push(car.state);
            }
        }
    }
    for (c, car) in w.cars.iter().enumerate() {
        assert_eq!(
            car.state,
            CarState::Parked,
            "car {c} at three in the morning: {:?}",
            seen[c]
        );
        assert_eq!(car.spot, spots[c], "car {c} back in its own bay");
        assert!(
            left[c].len() >= 2,
            "car {c} left {} times: {:?}",
            left[c].len(),
            seen[c]
        );
    }
    // (Each morning's six set off within the hour of each other.)
    for day in 0..2 {
        let (first, last) = left
            .iter()
            .map(|l| l[day])
            .fold((u64::MAX, 0), |(a, b), t| (a.min(t), b.max(t)));
        eprintln!(
            "day {day}: set off from {:05.2}h to {:05.2}h",
            clock::hour(first),
            clock::hour(last)
        );
        assert!(
            last - first <= common::hours(1.0),
            "day {day}: set off from {:05.2}h to {:05.2}h",
            clock::hour(first),
            clock::hour(last)
        );
    }
}
