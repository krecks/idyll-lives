//! Choosing nearby things in an open-world town (`index::ObjectIndex::usable_near`,
//! `Choice::Outing`): residents use the public things within reach of where they are, and go on
//! outings to places farther off when the way there is short enough (`visits.outingMinutes`) the
//! way they'd go (`world::leisure_way`: the household's car, a bike of theirs, the taxi, else on
//! foot) and they have the time, home by `visits.outingHomeBy`; an outing's stay counts from when
//! they get there.

use crate::common;

use common::hamlets::{Hamlets, two_hamlets, with_cars};
use serde_json::{Value, json};
use sim_core::taxi::StopState;
use sim_core::{MINUTES_PER_TICK, World, clock, snapshot};

const TICKS_PER_MINUTE: u64 = (1.0 / MINUTES_PER_TICK) as u64;

/// Who is on an outing (visiting a public place out of `autonomy.reach` of home) and where.
fn on_outings(w: &World) -> Vec<(u32, u32)> {
    let reach = w.content.autonomy.reach;
    w.sims
        .iter()
        .filter_map(|s| {
            let v = s.visiting?;
            let plot = &w.plots[v.plot as usize];
            let home = w.households[s.household as usize].plot?;
            let (a, b) = (plot.arrival_point(), w.plots[home as usize].arrival_point());
            (plot.public && sim_core::travel::dist(a, b) > reach).then_some((s.id, v.plot))
        })
        .collect()
}

/// `h` with a car on every driveway and a parking spot on the second hamlet's street by its park
/// (as `travel_trips.rs`).
fn with_cars_and_parking(h: &Hamlets) -> String {
    let mut town: Value = serde_json::from_str(&with_cars(h)).unwrap();
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
    town.to_string()
}

#[test]
fn with_no_car_nor_bike_an_outing_a_walk_of_hours_away_goes_by_taxi() {
    // Two hamlets 160 m apart: the other's park is a walk of about two hours (more than
    // `outingMinutes`), and nobody has a car to drive there in nor a bike to ride: whoever goes
    // goes by the taxi, there and back (`taxi::leisure_eta`), never on foot, and is home by the
    // hour.
    let content = common::content();
    let h = two_hamlets(&content, 31);
    let mut w = World::from_json(&content, &h.town, 31).expect("the hamlets load");
    let home_by = w.content.visits.outing_home_by;
    let end = w.tick + 3 * 24 * 60 * TICKS_PER_MINUTE;
    let mut outings = std::collections::BTreeSet::new();
    let mut by_taxi = std::collections::BTreeSet::new();
    let mut out_late = std::collections::BTreeSet::new();
    while w.tick < end {
        w.tick_once();
        let now = on_outings(&w);
        outings.extend(now.iter().copied());
        for r in w.taxis.runs.iter().filter(|r| !r.home) {
            let Some(place) = r.place else { continue };
            for st in r.stops.iter().filter(|st| st.state == StopState::Aboard) {
                by_taxi.insert((st.sim, place));
            }
        }
        let hour = clock::hour(w.tick);
        if hour >= home_by || hour < 6.0 {
            for (s, plot) in now {
                out_late.insert((clock::day(w.tick), s, plot));
            }
        }
    }
    eprintln!("outings (resident, place): {outings:?}");
    let on_foot: Vec<_> = outings.difference(&by_taxi).collect();
    assert!(
        on_foot.is_empty(),
        "outings (resident, place) a walk of hours away, not by taxi: {on_foot:?}"
    );
    assert!(
        out_late.is_empty(),
        "out on an outing after {home_by}:00 (day, resident, place): {out_late:?}"
    );
}

#[test]
fn an_outing_by_car_in_reach_is_picked_and_theyre_home_by_the_hour() {
    // The same hamlets with a car on every driveway and parking by the other's park: it's a
    // short drive.
    let content = common::content();
    let h = two_hamlets(&content, 31);
    let mut w =
        World::from_json(&content, &with_cars_and_parking(&h), 31).expect("the hamlets load");
    let home_by = w.content.visits.outing_home_by;
    let end = w.tick + 3 * 24 * 60 * TICKS_PER_MINUTE;
    let mut outings = std::collections::BTreeSet::new();
    let mut out_late = std::collections::BTreeSet::new();
    let mut drove = std::collections::BTreeSet::new();
    while w.tick < end {
        w.tick_once();
        let now = on_outings(&w);
        outings.extend(now.iter().copied());
        for &(s, _) in &now {
            let a = w.sims[s as usize].current();
            if a.is_some_and(|a| matches!(a.phase, sim_core::world::Phase::Driving)) {
                drove.insert(s);
            }
        }
        // Past the hour (and before the morning), nobody is still out on one.
        let hour = clock::hour(w.tick);
        if hour >= home_by || hour < 6.0 {
            for (s, plot) in now {
                out_late.insert((clock::day(w.tick), s, plot));
            }
        }
    }
    eprintln!("outings (resident, place): {outings:?}, at the wheel or in the car: {drove:?}");
    assert!(!outings.is_empty(), "nobody went anywhere out of reach");
    assert!(!drove.is_empty(), "nobody drove there");
    assert!(
        out_late.is_empty(),
        "out on an outing after {home_by}:00 (day, resident, place): {out_late:?}"
    );
}

#[test]
fn someone_with_a_bike_rides_to_a_place_beyond_walking_reach_and_is_home_by_the_hour() {
    // The hamlets again, no cars, and a city bike in the garden of the player's home: the other
    // hamlet's park is a ride of about forty minutes.
    let content = common::content();
    let h = two_hamlets(&content, 31);
    let mut w = World::from_json(&content, &h.town, 31).expect("the hamlets load");
    let home = w.households[0].plot.expect("the player's home");
    let p = w.plots[home as usize].clone();
    let placed = (p.z..p.z + p.d).rev().any(|z| {
        (p.x..p.x + p.w).any(|x| {
            w.apply(sim_core::Command::Buy {
                household: 0,
                object: "bicycle".into(),
                at: Some([x, z, 0]),
                colors: Default::default(),
                turn: None,
                offset: None,
                size: None,
                tall: None,
            })
            .is_ok()
        })
    });
    assert!(placed, "no room for the bike at home");
    let bike = w.objects.len() - 1;
    let home_by = w.content.visits.outing_home_by;
    let end = w.tick + 3 * 24 * 60 * TICKS_PER_MINUTE;
    let (mut rode, mut out_late) = (
        std::collections::BTreeSet::new(),
        std::collections::BTreeSet::new(),
    );
    let mut parked = false;
    let stable = w.objects[bike].stable as f32;
    let mut left = Vec::new();
    let mut buf = vec![0.0; snapshot::CAPACITY];
    while w.tick < end {
        w.tick_once();
        let now = on_outings(&w);
        for &(s, plot) in &now {
            let sim = &w.sims[s as usize];
            if sim.riding == Some(bike as u32) {
                rode.insert((s, plot));
            }
            // (There, off it: it's not at home, nor under them; the snapshot says where it's left.)
            let (x, z) = sim.tile();
            let off = w.plot_at(x, z) == Some(plot)
                && sim.riding.is_none()
                && sim.bike() == Some(bike as u32);
            parked |= off;
            if off && w.tick.is_multiple_of(600) {
                snapshot::write(&w, &mut buf);
                let o = snapshot::HEADER_LEN + s as usize * snapshot::SIM_STRIDE;
                if buf[o + snapshot::sim::RIDE] == -2.0 - stable {
                    left.push((
                        plot,
                        [
                            buf[o + snapshot::sim::PARK_X],
                            buf[o + snapshot::sim::PARK_Z],
                        ],
                    ));
                }
            }
        }
        let hour = clock::hour(w.tick);
        if hour >= home_by || hour < 6.0 {
            for (s, plot) in now {
                out_late.insert((clock::day(w.tick), s, plot));
            }
        }
    }
    eprintln!("rode out on the bike (resident, place): {rode:?}");
    assert!(!rode.is_empty(), "nobody rode anywhere out of reach");
    assert!(parked, "nobody left the bike by the place they went to");
    // Left on the place's lot, on free ground off its paths, near its way in.
    assert!(
        !left.is_empty(),
        "the snapshot never said where the bike was left"
    );
    let paths: Vec<Value> = serde_json::from_str::<Value>(&h.town).unwrap()["meta"]["paths"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for &(plot, at) in &left {
        let p = &w.plots[plot as usize];
        let (x, z) = (at[0].floor() as i32, at[1].floor() as i32);
        assert_eq!(
            w.plot_at(x, z),
            Some(plot),
            "left at {at:?}, off the place's lot"
        );
        assert!(
            sim_core::travel::dist(at, p.arrival_point()) <= 9.0,
            "left at {at:?}, far from the way in at {:?}",
            p.arrival_point()
        );
        let on_path = paths.iter().any(|r| {
            let k = |key: &str| r[key].as_f64().unwrap_or(0.0) as f32;
            at[0] > k("x") && at[0] < k("x") + k("w") && at[1] > k("z") && at[1] < k("z") + k("d")
        });
        assert!(!on_path, "left at {at:?}, on a path");
        assert!(
            !w.objects
                .iter()
                .any(|o| o.id as usize != bike && (o.x, o.z) == (x, z)),
            "left at {at:?}, on something"
        );
    }
    assert!(
        out_late.is_empty(),
        "out on an outing after {home_by}:00 (day, resident, place): {out_late:?}"
    );
    // (Home again, it's back at its place, nobody's.)
    assert!(
        w.sims
            .iter()
            .all(|s| s.bike() != Some(bike as u32) || s.visiting.is_some()),
        "the bike is still out with someone home"
    );
}

#[test]
fn an_outings_stay_counts_from_when_they_get_there() {
    // Sent to the other hamlet's park (about two hours away on foot, near an hour by the taxi
    // they take): their stay lasts its hours from when they're there, the way there not counted.
    let content = common::content();
    let h = two_hamlets(&content, 31);
    let mut w = World::from_json(&content, &h.town, 31).expect("the hamlets load");
    let park = w
        .plots
        .iter()
        .position(|p| p.public && h.plots[1].contains(&(p.id as usize)))
        .expect("a park in the other hamlet") as u32;
    let sim = w
        .sims
        .iter()
        .position(|s| w.households[s.household as usize].player && s.adult(&w.content))
        .expect("a grown-up of the player's");
    common::apply(
        &mut w,
        serde_json::json!({"type": "visit", "sim": sim, "plot": park}),
    )
    .unwrap();
    let set_off = w.tick;
    let mut there = None;
    for _ in 0..6 * 60 * TICKS_PER_MINUTE {
        w.tick_once();
        let s = &w.sims[sim];
        let (x, z) = s.tile();
        let going = s
            .current()
            .is_some_and(|a| matches!(a.task.kind, sim_core::world::TaskKind::Visit { .. }));
        if w.plot_at(x, z) == Some(park) && !going {
            there = Some((w.tick, s.visiting.expect("visiting the park").until));
            break;
        }
    }
    let (arrived, until) = there.expect("they got to the park");
    let walk = (arrived - set_off) / TICKS_PER_MINUTE;
    assert!(
        walk > 30,
        "the way there took {walk} minutes: too short to tell"
    );
    let stay = until.saturating_sub(arrived) as f32 / TICKS_PER_MINUTE as f32;
    let hours = w.content.visits.directed_hours * 60.0;
    assert!(
        (stay - hours).abs() <= 2.0,
        "their stay is {stay:.0} minutes from when they got there, after {walk} minutes on the way \
         ({hours:.0} asked for)"
    );
}
