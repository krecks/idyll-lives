//! Commutes in an open-world town (`life::Commute`): residents leave for work or school the walk
//! to the quickest way out of town (a bus stop, on foot) plus `commuteMinutes` (the part off the
//! map) before it starts, and get there on time.

mod common;

use common::hamlets::two_hamlets;
use sim_core::social::EventKind;
use sim_core::world::TaskKind;
use sim_core::{MINUTES_PER_TICK, World, clock, life};

const TICKS_PER_MINUTE: u64 = (1.0 / MINUTES_PER_TICK) as u64;

fn world(seed: u64) -> World {
    let content = common::content();
    let h = two_hamlets(&content, seed);
    World::from_json(&content, &h.town, seed as u32).expect("the hamlets load")
}

/// When `i`'s work or school starts today, if it does.
fn start_today(w: &World, i: usize) -> Option<u64> {
    let s = &w.sims[i];
    match &s.job {
        Some(job) => life::shift_today(&w.content, job, &s.skills, w.tick),
        None => life::school_today(&w.content, s, w.tick),
    }
    .map(|(_, start, _)| start)
}

/// Per resident: when they set off (their task became work) and when they got there (away).
#[derive(Default, Clone)]
struct Day {
    left: Option<u64>,
    there: Option<u64>,
}

#[test]
fn residents_leave_by_their_walk_and_are_never_late() {
    let mut w = world(11);
    let off_map = (w.content.career_rules.commute_minutes / MINUTES_PER_TICK) as u64;
    let mut days: Vec<Day> = vec![Day::default(); w.sims.len()];
    let mut day = clock::day(w.tick);
    let (mut trips, mut on_plan, mut worst_leave, mut worst_late) = (0, 0, 0i64, i64::MIN);
    let begun = w.tick;
    let end = w.tick + 3 * 24 * 60 * TICKS_PER_MINUTE;
    while w.tick < end {
        w.tick_once();
        if clock::day(w.tick) != day {
            day = clock::day(w.tick);
            days = vec![Day::default(); w.sims.len()];
        }
        for (i, day) in days.iter_mut().enumerate() {
            let s = &w.sims[i];
            if !s.here() {
                continue;
            }
            let working = s
                .current()
                .is_some_and(|a| matches!(a.task.kind, TaskKind::Work));
            if working && day.left.is_none() {
                day.left = Some(w.tick);
            }
            if let Some(left) = day.left
                && s.away_until.is_some()
                && day.there.is_none()
            {
                day.there = Some(w.tick);
                // (By car, the drive is timed by `cars::lead_ticks`, not the walk.)
                let Some(start) = start_today(&w, i) else {
                    continue;
                };
                let Some(c) = s.commute() else { continue };
                if w.cars.iter().any(|car| car.aboard.contains(&(i as u32))) {
                    continue;
                }
                // (Not those due to set off before the game began.)
                if start < begun + c.ticks {
                    continue;
                }
                // Set off when the walk and the way off the map say: within a minute, unless the
                // walk turned out longer than it looked shortly before (then at once)...
                let planned = start - c.ticks;
                let gap = left.abs_diff(planned);
                worst_leave = worst_leave.max(gap as i64);
                if gap <= TICKS_PER_MINUTE {
                    on_plan += 1;
                }
                assert!(
                    gap <= 10 * TICKS_PER_MINUTE,
                    "{} left at {:.2}, not {:.2} (start {:.2}, walk {:.0} min, {:?})",
                    s.name,
                    clock::hour(left),
                    clock::hour(planned),
                    clock::hour(start),
                    c.walk,
                    c
                );
                // ...and at the way out of town in time for the rest.
                let late = w.tick as i64 - (start - off_map) as i64;
                worst_late = worst_late.max(late);
                assert!(
                    late <= 0,
                    "{} late by {:.1} min (a {:.0}-minute walk, left at {:.2})",
                    s.name,
                    late as f32 * MINUTES_PER_TICK,
                    c.walk,
                    clock::hour(left)
                );
                trips += 1;
            }
        }
    }
    eprintln!(
        "{trips} walks to work or school, {on_plan} set off within a minute of plan (at worst {} ticks off); at the exit at worst {:.1} min before time",
        worst_leave,
        -(worst_late as f32) * MINUTES_PER_TICK
    );
    assert!(trips >= 10, "only {trips} walks to work or school");
    assert!(on_plan * 10 >= trips * 9, "{on_plan} of {trips} on plan");
    assert!(
        w.events.iter().all(|e| e.kind != EventKind::MissedWork),
        "someone missed work"
    );
}

#[test]
fn the_walk_is_timed_to_the_quickest_way_out() {
    let mut w = world(12);
    w.tick_once();
    let walkers: Vec<usize> = (0..w.sims.len())
        .filter(|&i| w.sims[i].commute().is_some())
        .collect();
    assert!(!walkers.is_empty(), "nobody's commute was worked out");
    for &i in &walkers {
        let c = w.sims[i].commute().unwrap();
        let off_map = w.content.career_rules.commute_minutes;
        assert_eq!(
            c.ticks,
            ((c.walk + off_map) / MINUTES_PER_TICK) as u64,
            "{}",
            w.sims[i].name
        );
        assert!(c.walk < 60.0, "{}: {:.0} min", w.sims[i].name, c.walk);
    }
    // Each hamlet has a bus stop in the middle of its street: most walk to one of those, not to
    // the end of the road.
    let to_stops = walkers
        .iter()
        .filter(|&&i| w.travel.is_stop(w.sims[i].commute().unwrap().exit))
        .count();
    assert!(
        to_stops * 2 > walkers.len(),
        "{to_stops} of {} to a stop",
        walkers.len()
    );
}
