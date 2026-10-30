//! The game clock in long games: what happens once a day (births and ageing at midnight, the
//! job market, rent, the planner's review) happens every day, also after day 1167, where the
//! old f32 clock skipped midnight (the release review's R3-01 proofs).

use crate::common;

use sim_core::social::EventKind;
use sim_core::{World, clock};

/// The days on which `tick_at(day, minute)` names a tick that `day()` doesn't put on that day,
/// as ranges: every exact-tick gate (`tick == tick_at(day(tick), m)`) is dead on them.
fn dead_days(minute: f32, last: u32) -> Vec<(u32, u32)> {
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for day in 2..=last {
        let Some(t) = clock::tick_at(day, minute) else {
            continue;
        };
        if clock::tick_at(clock::day(t), minute) == Some(t) {
            continue;
        }
        match ranges.last_mut() {
            Some(r) if r.1 + 1 == day => r.1 = day,
            _ => ranges.push((day, day)),
        }
    }
    ranges
}

#[test]
fn every_days_gates_fire() {
    let mut failed = false;
    for minute in [
        0.0,
        6.0 * 60.0,
        9.0 * 60.0,
        12.0 * 60.0,
        18.0 * 60.0,
        23.5 * 60.0,
    ] {
        let dead = dead_days(minute, 6000);
        if !dead.is_empty() {
            eprintln!("minute {minute:>6}: dead on days {dead:?}");
        }
        failed |= !dead.is_empty();
    }
    assert!(!failed, "exact-tick gates stop on some days");
}

/// Around every midnight up to day 6000: the minute of the day stays below 1440 and days don't
/// go backwards.
#[test]
fn clock_edges() {
    let (mut at_1440, mut backwards, mut first) = (0, 0, None);
    for day in 2..=6000u32 {
        let t0 = clock::tick_at(day, 0.0).unwrap();
        for t in t0 - 3..t0 + 3 {
            if clock::minute_of_day(t) >= 1440.0 {
                at_1440 += 1;
                first.get_or_insert((day, t));
            }
            if clock::day(t + 1) < clock::day(t) {
                backwards += 1;
            }
        }
    }
    assert_eq!(
        at_1440 + backwards,
        0,
        "minute_of_day >= 1440: {at_1440} (first {first:?}); day going backwards: {backwards}"
    );
}

/// Residents' ages at each midnight from day `from` for `days` days (the town jumped there).
fn ages_from(from: u32, days: u32) -> Vec<(u32, f32, usize)> {
    let content = common::content();
    let town = common::town(&content, 7, 2);
    let mut w = World::from_json(&content, &town, 7).unwrap();
    w.tick = clock::tick_at(from, 23.0 * 60.0).unwrap();
    let mut out = Vec::new();
    let end = clock::tick_at(from + days, 60.0).unwrap();
    let mut last_day = clock::day(w.tick);
    let first = w.sims.iter().position(|s| s.here()).unwrap();
    while w.tick < end {
        w.tick_once();
        let day = clock::day(w.tick);
        if day != last_day {
            last_day = day;
            let grew = w
                .events
                .iter()
                .filter(|e| e.kind == EventKind::GrewOlder)
                .count();
            out.push((day, w.sims[first].age, grew));
        }
    }
    common::check(&w);
    out
}

/// A late game, crossing day 1167: residents still grow a year-fraction older every midnight.
#[test]
fn residents_still_age_after_day_1167() {
    let early = ages_from(10, 3);
    let late = ages_from(1166, 3);
    let grows = |a: &[(u32, f32, usize)]| a.windows(2).filter(|p| p[1].1 > p[0].1).count();
    assert_eq!(grows(&early), 2, "ages each midnight early on: {early:?}");
    assert_eq!(
        grows(&late),
        2,
        "ages each midnight on days 1167 and 1168 too: {late:?}"
    );
}
