//! Choosing nearby things in an open-world town (`index::ObjectIndex::usable_near`,
//! `Choice::Outing`): residents use the public things within reach of where they are, and go on
//! outings to places farther off when the way there is short enough (`visits.outingMinutes`) the
//! way they'd go (`world::leisure_way`: the household's car, a bike of theirs, the taxi, else on
//! foot) and they have the time, home by `visits.outingHomeBy`; an outing's stay counts from when
//! they get there.

use crate::common;

use common::hamlets::{two_hamlets, two_hamlets_with_parking, with_cars};
use serde_json::Value;
use sim_core::cars::CarState;
use sim_core::taxi::StopState;
use sim_core::{World, clock, snapshot};

const TICKS_PER_MINUTE: u64 = common::TICKS_PER_MINUTE;

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

#[test]
#[cfg_attr(not(feature = "slow-tests"), ignore = "slow: --features slow-tests")]
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
#[cfg_attr(not(feature = "slow-tests"), ignore = "slow: --features slow-tests")]
fn an_outing_by_car_in_reach_is_picked_and_theyre_home_by_the_hour() {
    // The same hamlets with a car on every driveway and a parking bay by the other's park
    // (`hamlets::two_hamlets_with_parking`): it's a short drive. Whoever drives there parks in
    // the bay and walks on, and they and their car are home by the hour.
    let content = common::content();
    let h = two_hamlets_with_parking(&content, 31);
    let mut w = World::from_json(&content, &with_cars(&h), 31).expect("the hamlets load");
    let home_by = w.content.visits.outing_home_by;
    let end = w.tick + 3 * 24 * 60 * TICKS_PER_MINUTE;
    let mut outings = std::collections::BTreeSet::new();
    let mut out_late = std::collections::BTreeSet::new();
    // (Who drove where, their car parked out there on the trip; cars still parked out late.)
    let mut drove = std::collections::BTreeSet::new();
    let mut parked_late = std::collections::BTreeSet::new();
    while w.tick < end {
        w.tick_once();
        let now = on_outings(&w);
        outings.extend(now.iter().copied());
        let out: Vec<_> = (w.cars.iter().enumerate())
            .filter_map(|(c, car)| {
                let t = car.trip.filter(|_| car.state == CarState::Out)?;
                Some((c, t.driver, t.plot))
            })
            .collect();
        drove.extend(out.iter().map(|&(_, s, plot)| (s, plot)));
        // Past the hour (and before the morning), nobody is still out on one, nor their car.
        let hour = clock::hour(w.tick);
        if hour >= home_by || hour < 6.0 {
            for (s, plot) in now {
                out_late.insert((clock::day(w.tick), s, plot));
            }
            for (c, ..) in out {
                parked_late.insert((clock::day(w.tick), c));
            }
        }
    }
    eprintln!("outings (resident, place): {outings:?}, driven to (driver, place): {drove:?}");
    assert!(!outings.is_empty(), "nobody went anywhere out of reach");
    assert!(
        drove.iter().any(|d| outings.contains(d)),
        "nobody drove there and parked"
    );
    assert!(
        out_late.is_empty(),
        "out on an outing after {home_by}:00 (day, resident, place): {out_late:?}"
    );
    assert!(
        parked_late.is_empty(),
        "a car still parked out after {home_by}:00 (day, car): {parked_late:?}"
    );
}

#[test]
#[cfg_attr(not(feature = "slow-tests"), ignore = "slow: --features slow-tests")]
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
            if off && w.tick.is_multiple_of(common::mins(30.0)) {
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

/// The other hamlet's park (`two_hamlets`'s second), as a plot index.
fn far_park(w: &World, h: &common::hamlets::Hamlets) -> u32 {
    w.plots
        .iter()
        .position(|p| p.public && h.plots[1].contains(&(p.id as usize)))
        .expect("a park in the other hamlet") as u32
}

/// Sent to a place that shuts before they could get there (its hours, `Plot::open_for`, and the
/// way they'd go, about an hour by the taxi), they're told so; to one with no hours, off they go.
#[test]
fn a_visit_to_a_place_shutting_before_they_could_get_there_is_refused() {
    let content = common::content();
    let h = two_hamlets(&content, 31);
    let mut w = World::from_json(&content, &h.town, 31).expect("the hamlets load");
    let park = far_park(&w, &h);
    let sim = w
        .sims
        .iter()
        .position(|s| w.households[s.household as usize].player && s.adult(&w.content))
        .expect("a grown-up of the player's");
    let now = clock::minute_of_day(w.tick) as u16;
    let close = now + 20;
    w.plots[park as usize].hours = Some(sim_core::world::PlotHours {
        open: 0,
        close,
        days: 0b1111111,
    });
    let err = w
        .apply(sim_core::Command::Visit {
            sim: sim as u32,
            plot: park,
        })
        .unwrap_err()
        .to_string();
    assert_eq!(
        err,
        format!(
            "{} closes at {:02}:{:02}, before {} could get there",
            w.plots[park as usize].name,
            close / 60,
            close % 60,
            w.sims[sim].name
        )
    );
    w.plots[park as usize].hours = None;
    w.apply(sim_core::Command::Visit {
        sim: sim as u32,
        plot: park,
    })
    .expect("off to a place with no hours");
}

#[test]
#[cfg_attr(not(feature = "slow-tests"), ignore = "slow: --features slow-tests")]
fn nobody_sets_out_for_a_closed_place_and_those_there_go_home_as_it_shuts() {
    // The hamlets with cars and parking (`an_outing_by_car_in_reach_...`), the other's park open
    // 9:00 to 16:00: those who go on an outing there get there while it's open, and are gone by
    // last orders after it shuts. (Over a few seeds: whether anyone picks the park in three days
    // is chance, about four seeds in five, the way there accurately weighed.)
    let content = common::content();
    let h = two_hamlets_with_parking(&content, 31);
    let (open, close) = (9 * 60, 16 * 60);
    let late = f32::from(close) + sim_core::staff::LAST_ORDERS * 60.0 + 15.0;
    let (mut arrivals, mut shut_arrivals, mut stayed_late) = (Vec::new(), Vec::new(), Vec::new());
    for seed in 31..35 {
        let mut w = World::from_json(&content, &with_cars(&h), seed).expect("the hamlets load");
        let park = far_park(&w, &h);
        w.plots[park as usize].hours = Some(sim_core::world::PlotHours {
            open,
            close,
            days: 0b1111111,
        });
        let end = w.tick + 3 * 24 * 60 * TICKS_PER_MINUTE;
        let mut there = std::collections::BTreeSet::new();
        while w.tick < end {
            w.tick_once();
            let minute = clock::minute_of_day(w.tick);
            for (i, s) in w.sims.iter().enumerate() {
                let (x, z) = s.tile();
                let at = s.here()
                    && s.visiting.is_some_and(|v| v.plot == park)
                    && w.plot_at(x, z) == Some(park);
                let when = (seed, clock::day(w.tick), minute as u32, i);
                if at && !there.contains(&i) {
                    arrivals.push(when);
                    if !w.plots[park as usize].open_at_tick(w.tick) {
                        shut_arrivals.push(when);
                    }
                }
                if at && minute >= late {
                    stayed_late.push(when);
                }
                if at {
                    there.insert(i);
                } else {
                    there.remove(&i);
                }
            }
        }
    }
    eprintln!("got to the park (seed, day, minute, resident): {arrivals:?}");
    assert!(!arrivals.is_empty(), "nobody went to the park");
    assert!(
        shut_arrivals.is_empty(),
        "got there while it was shut (seed, day, minute, resident): {shut_arrivals:?}"
    );
    stayed_late.dedup_by_key(|k| (k.0, k.1, k.3));
    assert!(
        stayed_late.is_empty(),
        "still there well after it shut (seed, day, minute, resident): {stayed_late:?}"
    );
}

/// Sent to a place while busy, the visit queued behind a use at home that ends shortly before
/// it shuts: when the use is over it shuts before they could get there (the way they'd go,
/// half an hour by the taxi), so they don't set out (as the order would be refused then).
#[test]
fn a_visit_queued_till_too_late_to_get_there_before_it_shuts_isnt_set_out_for() {
    let content = common::content();
    let h = two_hamlets(&content, 31);
    let mut w = World::from_json(&content, &h.town, 31).expect("the hamlets load");
    let park = far_park(&w, &h);
    let sim = w
        .sims
        .iter()
        .position(|s| w.households[s.household as usize].player && s.adult(&w.content))
        .expect("a grown-up of the player's");
    let home = w.households[w.sims[sim].household as usize].plot;
    // Something at home that takes a while, but much less than the way there.
    let uses: Vec<(u32, usize)> = (0..w.objects.len())
        .filter(|&o| w.plot_at(w.objects[o].x, w.objects[o].z) == home)
        .flat_map(|o| {
            let def = &w.content.objects[w.objects[o].def];
            (0..def.interactions.len())
                .filter(|&i| (10.0..=20.0).contains(&def.interactions[i].minutes))
                .map(move |i| (o as u32, i))
        })
        .collect();
    let (object, interaction) = uses
        .into_iter()
        .find(|&(object, interaction)| {
            w.apply(sim_core::Command::Use {
                sim: sim as u32,
                object,
                interaction,
            })
            .is_ok()
        })
        .expect("something at home to use for a while");
    // The park shuts as soon as the order still fits: just as they could get there now.
    let now = clock::minute_of_day(w.tick) as u16;
    let close = (now + 1..now + 6 * 60)
        .find(|&close| {
            w.plots[park as usize].hours = Some(sim_core::world::PlotHours {
                open: 0,
                close,
                days: 0b1111111,
            });
            w.apply(sim_core::Command::Visit {
                sim: sim as u32,
                plot: park,
            })
            .is_ok()
        })
        .expect("the visit is taken");
    assert!(
        close > now + 25,
        "the way there takes {} minutes: too short to tell",
        close - now
    );
    let (mut used, mut set_out) = (false, false);
    common::until(&mut w, 60, |w| {
        let s = &w.sims[sim];
        let doing = s.current().map(|a| &a.task.kind);
        used |=
            matches!(doing, Some(&sim_core::world::TaskKind::Use { object: o, .. }) if o == object);
        set_out |= matches!(doing, Some(&sim_core::world::TaskKind::Visit { plot }) if plot == park)
            || s.visiting.is_some_and(|v| v.plot == park);
        false
    });
    assert!(used, "they used object {object} ({interaction}) first");
    assert!(
        !set_out,
        "they set out for the park after the use, though it shuts at {:02}:{:02}",
        close / 60,
        close % 60
    );
}

/// On their bike out to the other hamlet's park, halfway there, sent back to their own hamlet's
/// instead (the far park called off), shutting in twice the time they've ridden so far: a ride
/// back that far gets them there in time, a walk (three times as slow) wouldn't. The way is timed
/// by the bike they're on, as the order is taken and as they set off on it again (`leisure_eta`;
/// `ride::leisure_bike` offers one at home only: timed walking, it was refused), and they ride
/// there before it shuts.
#[test]
fn on_their_bike_sent_where_a_ride_gets_them_before_it_shuts_they_ride_there() {
    let content = common::content();
    let h = two_hamlets(&content, 31);
    let mut w = World::from_json(&content, &h.town, 31).expect("the hamlets load");
    let far = far_park(&w, &h);
    let near = w
        .plots
        .iter()
        .position(|p| p.public && h.plots[0].contains(&(p.id as usize)))
        .expect("a park in their own hamlet") as u32;
    let sim = w
        .sims
        .iter()
        .position(|s| w.households[s.household as usize].player && s.adult(&w.content))
        .expect("a grown-up of the player's");
    let home = w.households[w.sims[sim].household as usize]
        .plot
        .expect("their home");
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
    w.apply(sim_core::Command::Visit {
        sim: sim as u32,
        plot: far,
    })
    .expect("off to the far park");
    // (Ridden halfway there: as far as from there back to their own hamlet's park.)
    let (door, there) = (
        w.plots[near as usize].arrival_point(),
        w.plots[far as usize].arrival_point(),
    );
    let half = sim_core::travel::dist(door, there) / 2.0;
    let mut rode = 0;
    let out = common::until(&mut w, 3 * 60, |w| {
        let s = &w.sims[sim];
        rode += u64::from(s.riding.is_some());
        s.riding.is_some() && sim_core::travel::dist(s.pos, door) >= half
    });
    assert!(out, "they never rode halfway to the far park");
    let now = clock::minute_of_day(w.tick) as u16;
    let close = now + (2 * rode / TICKS_PER_MINUTE) as u16;
    w.plots[near as usize].hours = Some(sim_core::world::PlotHours {
        open: 0,
        close,
        days: 0b1111111,
    });
    w.apply(sim_core::Command::Visit {
        sim: sim as u32,
        plot: near,
    })
    .expect("sent back to their own hamlet's park, a ride in time");
    w.apply(sim_core::Command::Cancel {
        sim: sim as u32,
        index: 0,
    })
    .expect("the far park called off");
    let mut ridden = false;
    let got = common::until(&mut w, 3 * 60, |w| {
        let s = &w.sims[sim];
        let doing = s.current().map(|a| a.task.kind);
        ridden |= s.riding.is_some()
            && matches!(doing, Some(sim_core::world::TaskKind::Visit { plot }) if plot == near);
        s.visiting.is_some_and(|v| v.plot == near && v.arrived > 0)
    });
    assert!(ridden, "they didn't ride there");
    assert!(got, "they never got there");
    assert!(
        (clock::minute_of_day(w.tick) as u16) < close,
        "they got there after it shut at {:02}:{:02}",
        close / 60,
        close % 60
    );
}
