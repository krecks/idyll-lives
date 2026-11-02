//! Passengers on trips by car between hamlets (`cars::Trip`, `Car::riders`): a household off to
//! the same place in another hamlet goes in one car. The driver gets in and waits in the
//! driveway for the others (`CarState::Boarding`); they get in, a seat each, ride there sat in
//! it, get out at the parking spot and walk on with the driver. When one of them goes home the
//! others go too: back to the car, in, and home together. Saved and loaded on the way, they go
//! on as they were.

mod common;

use common::hamlets::{Hamlets, Place, hamlets, with_cars};
use serde_json::{Value, json};
use sim_core::cars::CarState;
use sim_core::travel::Mode;
use sim_core::{Command, World};

/// Two hamlets 330 m apart along the country road, cars on the driveways, and a parking spot on
/// the second hamlet's street by its park (as `travel_trips.rs`).
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

/// A car of the first hamlet whose household has at least three residents at home, an owner
/// of it among them: the car, the owner and two more of them.
fn family(w: &World, h: &Hamlets) -> Option<(usize, [usize; 3])> {
    w.cars.iter().enumerate().find_map(|(c, car)| {
        let home = w.households[car.household as usize].plot? as usize;
        h.plots[0].contains(&home).then_some(())?;
        let driver = *car.owners.first()? as usize;
        let others: Vec<usize> = (0..w.sims.len())
            .filter(|&i| {
                i != driver
                    && w.sims[i].household == car.household
                    && w.sims[i].here()
                    && w.sims[i].age >= 4.0
            })
            .take(2)
            .collect();
        (others.len() == 2).then(|| (c, [driver, others[0], others[1]]))
    })
}

fn on_plot(w: &World, i: usize) -> Option<u32> {
    let (x, z) = w.sims[i].tile();
    w.plot_at(x, z)
}

/// A household of three in the first hamlet, all there in their car at the park in the next.
struct There {
    w: World,
    c: usize,
    who: [usize; 3],
    home: u32,
    riders: Vec<u32>,
    parked_at: [f32; 2],
    /// Minutes the ride took, and the walk would have.
    minutes: f32,
    walk: f32,
}

/// A household of three in the first hamlet sent to the park in the next: one car there.
fn ride_there(driver_first: bool) -> There {
    let (mut w, _, family_of) = (5..40)
        .find_map(|seed| {
            let (w, h, park) = world(seed);
            family(&w, &h).map(|f| (w, h, (park, f)))
        })
        .expect("a household of three with a car in the first hamlet");
    let (park, (c, who)) = family_of;
    let household = w.cars[c].household;
    let home = w.households[household as usize].plot.unwrap();
    let parked_at = w.cars[c].spot;
    let cars_of_home = w
        .cars
        .iter()
        .filter(|car| car.household == household)
        .count();
    // A free afternoon hour for all three (school and work done), at home.
    let free = |w: &World| {
        who.iter().all(|&i| {
            let s = &w.sims[i];
            s.here()
                && s.away_until.is_none()
                && s.queue().next().is_none()
                && s.current().is_none()
                && on_plot(w, i) == Some(home)
        }) && w.cars[c].state == CarState::Parked
            && (16.0..17.0).contains(&sim_core::clock::hour(w.tick))
    };
    assert!(
        common::until(&mut w, 7 * 24 * 60, |w| free(w)),
        "never all three at home with nothing to do"
    );
    // All three are sent there, the driver first (or a minute after the others, who set off on
    // foot and turn round for the car once the driver is in it).
    let send = |w: &mut World, i: usize| {
        w.apply(Command::Visit {
            sim: i as u32,
            plot: park,
        })
        .unwrap();
    };
    if driver_first {
        who.iter().for_each(|&i| send(&mut w, i));
    } else {
        send(&mut w, who[1]);
        send(&mut w, who[2]);
        common::until(&mut w, 1, |_| false);
        for &i in &who[1..] {
            assert!(
                w.sims[i]
                    .current()
                    .is_some_and(|a| matches!(a.phase, sim_core::world::Phase::Routing { .. })),
                "{i} on their way on foot"
            );
        }
        send(&mut w, who[0]);
    }
    let walk = w.travel.eta(
        w.plots[home as usize].arrival_point(),
        w.plots[park as usize].arrival_point(),
        Mode::Foot,
        1.5,
    );
    // There: they get in, ride there (all three in the one car while it drives), get out at the
    // parking spot and walk on to the park.
    let mut most_aboard = 0;
    let mut there = None;
    for t in 0..20 * 60 * 20 {
        w.tick_once();
        let car = &w.cars[c];
        if car.state == CarState::Leaving {
            most_aboard = most_aboard.max(car.aboard.len());
            for &i in &who {
                assert!(
                    car.aboard.contains(&(i as u32)),
                    "{i} isn't in the car as it drives off: {:?}",
                    car.aboard
                );
            }
        }
        let others_out = w
            .cars
            .iter()
            .enumerate()
            .any(|(k, o)| k != c && o.household == household && o.trip.is_some());
        assert!(!others_out, "a second car of theirs went");
        if car.state == CarState::Out
            && who.iter().all(|&i| {
                on_plot(&w, i) == Some(park) && w.sims[i].visiting.is_some_and(|v| v.plot == park)
            })
        {
            there = Some(t);
            break;
        }
    }
    let there = there.unwrap_or_else(|| {
        panic!(
            "they never all got there: car {:?}, aboard {:?}, riders {:?}",
            w.cars[c].state, w.cars[c].aboard, w.cars[c].riders
        )
    });
    assert_eq!(most_aboard, 3, "all three rode in it");
    let minutes = there as f32 * sim_core::MINUTES_PER_TICK;
    assert!(
        minutes < walk / 2.0,
        "{minutes:.0} minutes to get there, {walk:.0} on foot"
    );
    let trip = w.cars[c].trip.expect("on a trip");
    // (The first owner off drives: sent first, or one of the others setting off before them.)
    assert!(who.contains(&(trip.driver as usize)));
    if driver_first {
        assert_eq!(trip.driver, who[0] as u32);
    }
    let mut riders = w.cars[c].riders.clone();
    riders.sort_unstable();
    let mut expect: Vec<u32> = who
        .iter()
        .map(|&i| i as u32)
        .filter(|&i| i != trip.driver)
        .collect();
    expect.sort_unstable();
    assert_eq!(riders, expect, "the two who came along ride home with it");
    assert_eq!(cars_of_home, 1);
    There {
        w,
        c,
        who,
        home,
        riders,
        parked_at,
        minutes,
        walk,
    }
}

/// A household of three in the first hamlet sent to the park in the next: one car there and back.
fn goes_in_one_car(driver_first: bool) {
    let content = common::content();
    let There {
        w,
        c,
        who,
        home,
        riders,
        parked_at,
        minutes,
        walk,
    } = ride_there(driver_first);

    // Saved and loaded there: the car still parked out with who came in it.
    let mut w = common::reload(&w, &content);
    assert_eq!(w.cars[c].state, CarState::Out);
    assert_eq!(w.cars[c].riders, riders);

    // One of them (not the driver) goes home: all three go back to the car and home in it.
    let rider = *riders.last().expect("riders");
    w.apply(Command::GoHome { sim: rider }).unwrap();
    let mut home_aboard = 0;
    let mut reloaded = false;
    let mut back = None;
    for t in 0..20 * 60 * 20 {
        w.tick_once();
        let car = &w.cars[c];
        if car.state == CarState::Arriving {
            home_aboard = home_aboard.max(car.aboard.len());
            // Saved and loaded on the way home: they ride on.
            if !reloaded && car.progress > 200.0 {
                reloaded = true;
                w = common::reload(&w, &content);
                assert_eq!(w.cars[c].state, CarState::Arriving);
                assert_eq!(w.cars[c].aboard.len(), 3);
                continue;
            }
        }
        if w.cars[c].state == CarState::Parked
            && who.iter().all(|&i| {
                on_plot(&w, i) == Some(home)
                    && w.sims[i].current().is_none()
                    && w.sims[i].visiting.is_none()
            })
        {
            back = Some(t);
            break;
        }
    }
    let back = back.unwrap_or_else(|| {
        panic!(
            "they never all got home: car {:?}, aboard {:?}, riders {:?}, where {:?}",
            w.cars[c].state,
            w.cars[c].aboard,
            w.cars[c].riders,
            who.map(|i| on_plot(&w, i))
        )
    });
    assert_eq!(home_aboard, 3, "all three rode home in it");
    assert!(reloaded, "saved and loaded on the way home");
    let minutes_back = back as f32 * sim_core::MINUTES_PER_TICK;
    assert!(
        minutes_back < walk / 2.0,
        "{minutes_back:.0} minutes to get home, {walk:.0} on foot"
    );
    assert!(w.cars[c].trip.is_none() && w.cars[c].riders.is_empty());
    assert!(sim_core::travel::dist(w.cars[c].spot, parked_at) < 0.01);
    eprintln!(
        "three there in {minutes:.0} minutes and home in {minutes_back:.0} ({walk:.0} on foot)"
    );
}

#[test]
fn a_household_off_to_the_next_hamlet_goes_in_one_car_and_comes_back_in_it() {
    goes_in_one_car(true);
}

#[test]
fn those_already_on_their_way_on_foot_turn_round_for_the_car() {
    goes_in_one_car(false);
}

#[test]
fn riders_waiting_for_a_driver_at_work_make_their_own_way_home() {
    let There {
        mut w,
        c,
        who,
        home,
        riders,
        ..
    } = ride_there(true);
    let trip = w.cars[c].trip.expect("on a trip");
    // The driver off to work from there (off the map for the afternoon); one of the others goes
    // home: back to the car, in, and (the driver not coming) out again and home on foot.
    let driver = trip.driver as usize;
    w.sims[driver].away_until = Some(w.tick + 8 * common::TICKS_PER_HOUR);
    let rider = *riders.last().expect("riders");
    w.apply(Command::GoHome { sim: rider }).unwrap();
    let others: Vec<usize> = who.iter().copied().filter(|&i| i != driver).collect();
    let mut sat = 0;
    let mut home_by = None;
    for t in 0..20 * 60 * 20 {
        w.tick_once();
        sat = sat.max(w.cars[c].aboard.len());
        assert_eq!(
            w.cars[c].state,
            CarState::Out,
            "it stays out there for its driver"
        );
        if others.iter().all(|&i| on_plot(&w, i) == Some(home)) {
            home_by = Some(t);
            break;
        }
    }
    assert!(
        home_by.is_some(),
        "they never got home: aboard {:?}, riders {:?}, where {:?}",
        w.cars[c].aboard,
        w.cars[c].riders,
        others.iter().map(|&i| on_plot(&w, i)).collect::<Vec<_>>()
    );
    assert_eq!(sat, 0, "nobody sat waiting in it for the driver");
    assert!(
        w.cars[c].riders.is_empty(),
        "nobody left to ride home in it"
    );
    assert_eq!(w.cars[c].trip.map(|t| t.driver), Some(trip.driver));
}
