//! Commutes in an open-world town (`life::Commute`): residents leave for work or school the walk
//! to the quickest way out of town on foot plus `commuteMinutes` (the part off the
//! map) before it starts, and get there on time.

use crate::common;

use common::hamlets::two_hamlets;
use sim_core::social::EventKind;
use sim_core::travel::Mode;
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
                let due = start - off_map;
                assert!(
                    !s.job.as_ref().is_some_and(|j| j.late),
                    "{} late for work",
                    s.name
                );
                let late = w.tick as i64 - due as i64;
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
    let off_map = w.content.career_rules.commute_minutes;
    let mut timed = 0;
    for &i in &walkers {
        let c = w.sims[i].commute().unwrap();
        let name = &w.sims[i].name;
        assert_eq!(
            c.ticks,
            ((c.walk + off_map) / MINUTES_PER_TICK) as u64,
            "{name}"
        );
        // To the quickest of the ways out.
        let from = w.stair_map().ground_pos(w.sims[i].pos);
        let eta = |k: usize| w.travel.eta(from, w.exits[k], Mode::Foot, 1.5);
        let best = (0..w.exits.len()).map(eta).fold(f32::INFINITY, f32::min);
        assert!(
            eta(c.exit) <= best + 1e-3,
            "{name}: to exit {} ({:.0} min), the quickest {best:.0}",
            c.exit,
            eta(c.exit)
        );
        timed += 1;
    }
    assert!(timed > 0, "no walk checked");
}

/// A walk worked out from upstairs (in bed at night, say) is timed along the ground, not across
/// the rows the storeys stack in (which had people setting off for work in the small hours).
#[test]
fn walks_from_upstairs_are_timed_on_the_ground() {
    let content = common::content();
    // A two-storey house (the same room on both, stairs inside, the upper storey on rows 32..)
    // in an open-world town, its way out at the foot of the lot: Ada, off to work, upstairs and
    // Ben, the same, downstairs.
    let town = |z: f32| {
        format!(
            r#"{{"width":24,"depth":32,"storeys":2,"size":"big",
            "walls":[[4,2,12,2],[4,10,12,10],[4,2,4,10],[12,2,12,10],[4,34,12,34],[4,42,12,42],[4,34,4,42],[12,34,12,42]],
            "doors":[{{"x":6,"z":10,"axis":"x"}}],
            "plots":[{{"name":"Home","x":2,"z":1,"w":16,"d":16,"entry":[9.5,16.5]}}],
            "households":[{{"name":"Player","plot":0,"player":true}}],
            "objects":[{{"def":"stairs","x":11,"z":3,"rot":0}}],
            "exits":[[9.5,31.5]],
            "sims":[{{"name":"Ada","age":30,"traits":["cheerful"],"x":6.5,"z":{z},"job":{{"career":"journalism.print","level":0}}}},
                    {{"name":"Ben","age":30,"traits":["cheerful"],"x":6.5,"z":5.5,"job":{{"career":"journalism.print","level":0}}}}]}}"#
        )
    };
    let walk = |z: f32| {
        let mut w = World::from_json(&content, &town(z), 1).unwrap();
        assert_eq!(w.stair_map().depth, 32);
        w.tick_once();
        let c = w.sims[0].commute().expect("a commute");
        (c.walk, w.sims[1].commute().unwrap().walk)
    };
    let (upstairs, ben) = walk(5.5 + 32.0);
    let (downstairs, _) = walk(5.5);
    assert_eq!(downstairs, ben);
    // Up the stairs and down again: a few metres more, not a storey's rows (32 m, 20 minutes).
    assert!(
        upstairs > downstairs && upstairs < downstairs + 10.0,
        "{upstairs:.1} minutes from upstairs, {downstairs:.1} from downstairs"
    );
}

/// A new day starts from the day before's walk (`life::refresh_commute`): nobody's way to work is
/// worked out again at midnight, only shortly before they set off (`firm_leave`).
#[test]
fn a_new_day_starts_from_the_day_befores_walk() {
    let mut w = world(11);
    let day = clock::day(w.tick);
    while clock::day(w.tick + 1) == day {
        w.tick_once();
    }
    let before: Vec<_> = w.sims.iter().map(|s| s.commute()).collect();
    assert!(before.iter().any(|c| c.is_some()), "nobody commutes");
    w.tick_once();
    assert_eq!(clock::day(w.tick), day + 1);
    for (s, before) in w.sims.iter().zip(before) {
        let (Some(was), Some(now)) = (before, s.commute()) else {
            continue;
        };
        assert_eq!((now.day, now.firm), (day + 1, false), "{}", s.name);
        // (The same way out, and no shorter: only longer if the walk from where they are now
        // looks longer, `firm_leave`.)
        assert_eq!(now.exit, was.exit, "{}", s.name);
        assert!(now.ticks >= was.ticks, "{}", s.name);
    }
}

/// Whoever gets to the way out of town only as work starts is late for it: the rest of the way
/// (`commuteMinutes`, off the map) is still to go, longer than `lateGraceMinutes`.
#[test]
fn at_the_way_out_as_work_starts_is_late() {
    let content = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],
        "objects":[],
        "careers":[{"id":"office","label":"Office","levels":[
            {"title":"Clerk","pay":100,"start":10,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
        "careerRules":{"commuteMinutes":60,"lateGraceMinutes":15,"performancePerShift":10},
        "economy":{"startingFunds":500}}"#;
    // An open-world town, Ada off to work in it.
    let town = r#"{"width":40,"depth":14,"size":"normal","exits":[[0.5,10.5]],
        "plots":[{"name":"Home","x":10,"z":0,"w":20,"d":9,"entry":[20.5,8.5]}],
        "households":[{"name":"Home","plot":0,"player":true}],
        "sims":[{"name":"Ada","household":0,"job":{"career":"office"},"x":1.5,"z":10.5}]}"#;
    let mut w = World::from_json(content, town, 5).unwrap();
    w.autonomy = false;
    // No work for her today until a minute before it starts, and then she's a step from the way
    // out: off at once, out of town as it starts.
    let shift_day = |w: &mut World, day: u32| w.sims[0].job.as_mut().unwrap().last_shift_day = day;
    common::run_to(&mut w, 2, 5.0);
    shift_day(&mut w, 2);
    common::run_to(&mut w, 2, 9.0 + 59.0 / 60.0);
    assert!(w.sims[0].away_until.is_none(), "already off to work");
    w.sims[0].pos = [1.5, 10.5];
    shift_day(&mut w, 1);
    let end = clock::tick_at(2, 10.5 * 60.0).unwrap();
    while w.sims[0].away_until.is_none() && w.tick < end {
        w.tick_once();
    }
    assert!(w.sims[0].away_until.is_some(), "never set off");
    assert!(
        clock::hour(w.tick) < 10.1,
        "out of town at {:.2}",
        clock::hour(w.tick)
    );
    assert!(w.sims[0].job.as_ref().unwrap().late, "not late for work");
}
