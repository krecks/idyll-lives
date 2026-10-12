//! Residents see to their needs before they run out, with the shipped content: a film they
//! chose is left for the toilet (one the player asked for isn't), nobody starts a two-hour film
//! on a bladder that won't last it, and the toilet comes on getting up, before bed and before
//! leaving for work (`dayRhythm.habits`).

mod common;

use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, MINUTES_PER_TICK, World, clock, life};

/// One house plot: a toilet and a sink, a home cinema, a fridge and a bed.
const TOWN: &str = r#"{"width":16,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":10,"entry":[8.5,9.5]}],
    "households":[{"name":"Home","plot":0,"player":true}],
    "objects":[{"def":"toilet","x":5,"z":1,"rot":0},{"def":"sink","x":7,"z":1,"rot":0},
               {"def":"homeCinema","x":9,"z":1,"rot":0},{"def":"fridge","x":13,"z":1,"rot":0},
               {"def":"bed","x":1,"z":1,"rot":0}],
    "sims":[{"name":"Ada","traits":["bookworm"],"x":6.5,"z":6.5}]}"#;

const TOILET: u32 = 0;
const CINEMA: u32 = 2;
const BED: u32 = 4;

/// The house, with the resident doing nothing on their own until `start` (day 1, hours).
fn world(seed: u32, start: f32) -> World {
    let mut w = World::from_json(&common::content(), TOWN, seed).unwrap();
    w.autonomy = false;
    until(&mut w, 1, start);
    w.autonomy = true;
    w
}

fn need(w: &World, id: &str) -> usize {
    w.content.needs.iter().position(|n| n.id == id).unwrap()
}

fn until(w: &mut World, day: u32, hour: f32) {
    let target = clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < target {
        w.tick_once();
    }
}

/// Everything full but these needs.
fn full_but(w: &mut World, but: &[(&str, f32)]) {
    let n = w.content.needs.len();
    w.sims[0].needs[..n].fill(1.0);
    for &(id, value) in but {
        let k = need(w, id);
        w.sims[0].needs[k] = value;
    }
}

/// What they're using right now (not on their way to): `(object, interaction id)`.
fn using(w: &World) -> Option<(u32, String)> {
    w.sims[0].current().filter(|a| matches!(a.phase, Phase::Using { .. })).and_then(|a| match a.task.kind {
        TaskKind::Use { object, interaction } => {
            let def = w.objects[object as usize].def;
            Some((object, w.content.objects[def].interactions[interaction].id.clone()))
        }
        _ => None,
    })
}

/// Ticks until resident 0 uses `object` (any interaction), at most `hours`.
fn until_using(w: &mut World, object: u32, hours: f32) -> Option<String> {
    let end = w.tick + (hours * 60.0 / MINUTES_PER_TICK) as u64;
    while w.tick < end {
        w.tick_once();
        if let Some((_, id)) = using(w).filter(|(o, _)| *o == object) {
            return Some(id);
        }
    }
    None
}

/// Watches the cinema with the bladder nearly empty: returns whether they got up for the toilet
/// before the end, and the lowest the bladder went.
fn bladder_during_a_film(w: &mut World) -> (bool, f32) {
    let b = need(w, "bladder");
    w.sims[0].needs[b] = 0.12;
    let mut lowest = 1.0f32;
    let end = w.tick + (3.0 * 60.0 / MINUTES_PER_TICK) as u64;
    let mut left = false;
    while w.tick < end {
        w.tick_once();
        lowest = lowest.min(w.sims[0].needs[b]);
        match using(w) {
            Some((o, _)) if o == CINEMA => {}
            Some((o, _)) if o == TOILET => return (left, lowest),
            _ => left = true,
        }
    }
    (false, lowest)
}

#[test]
fn a_film_they_chose_is_left_for_the_toilet_before_the_bladder_runs_out() {
    for seed in 0..3 {
        let mut w = world(seed, 14.0);
        // Bored, nothing else wanting: off to the cinema.
        full_but(&mut w, &[("fun", 0.0)]);
        until_using(&mut w, CINEMA, 1.0).expect("bored, they watch something");
        let (left, lowest) = bladder_during_a_film(&mut w);
        assert!(left, "seed {seed}: up from the cinema and to the toilet");
        assert!(lowest > 0.05, "seed {seed}: before the bladder ran out (it got to {lowest:.2})");
    }
}

#[test]
fn a_film_the_player_asked_for_goes_on() {
    let mut w = world(1, 14.0);
    full_but(&mut w, &[("fun", 0.0)]);
    let def = w.objects[CINEMA as usize].def;
    let film = w.content.objects[def].interactions.iter().position(|i| i.id == "film").unwrap();
    w.apply(Command::Use { sim: 0, object: CINEMA, interaction: film }).unwrap();
    until_using(&mut w, CINEMA, 0.5).expect("they sit down to the film");
    let b = need(&w, "bladder");
    w.sims[0].needs[b] = 0.12;
    for _ in 0..(30.0 / MINUTES_PER_TICK) as u32 {
        w.tick_once();
        assert_eq!(using(&w).map(|(o, _)| o), Some(CINEMA), "the player's film isn't left for the toilet");
    }
}

#[test]
fn nobody_starts_a_two_hour_film_on_a_bladder_that_wont_last_it() {
    let b_at_film: Vec<f32> = (0..6)
        .filter_map(|seed| {
            let mut w = world(seed, 14.0);
            full_but(&mut w, &[("fun", 0.0), ("bladder", 0.25)]);
            let b = need(&w, "bladder");
            let end = w.tick + (2.0 * 60.0 / MINUTES_PER_TICK) as u64;
            while w.tick < end {
                w.tick_once();
                if using(&w).is_some_and(|(o, id)| o == CINEMA && id == "film") {
                    return Some(w.sims[0].needs[b]);
                }
            }
            None
        })
        .collect();
    assert!(b_at_film.iter().all(|&b| b > 0.4), "a film only on a bladder that lasts it: {b_at_film:?}");
}

#[test]
fn the_toilet_before_leaving_for_work() {
    let mut w = world(3, 8.0);
    w.apply(Command::JoinCareer { sim: 0, career: 0, level: 0 }).unwrap();
    until(&mut w, 2, 0.5);
    let job = w.sims[0].job.clone().unwrap();
    let (leave, ..) = life::shift_today(&w.content, &job, &w.sims[0].skills, w.tick).expect("a workday");
    // An hour before they leave, up and about with a bladder not quite full.
    let ready = (w.content.day_rhythm.ready_minutes / MINUTES_PER_TICK) as u64;
    while w.tick < leave - ready - (15.0 / MINUTES_PER_TICK) as u64 {
        w.tick_once();
    }
    full_but(&mut w, &[("bladder", 0.7), ("fun", 0.3)]);
    let b = need(&w, "bladder");
    let mut toilet_at = None;
    while w.sims[0].away_until.is_none() && w.tick < leave + (60.0 / MINUTES_PER_TICK) as u64 {
        w.tick_once();
        if toilet_at.is_none() && using(&w).is_some_and(|(o, _)| o == TOILET) {
            toilet_at = Some(w.tick);
        }
    }
    assert!(w.sims[0].away_until.is_some(), "off to work");
    let toilet_at = toilet_at.expect("the toilet before leaving");
    assert!(toilet_at + ready >= leave, "in the last {:.0} minutes before leaving", ready as f32 * MINUTES_PER_TICK);
    assert!(w.sims[0].needs[b] > 0.85, "left with a bladder at {:.2}", w.sims[0].needs[b]);
}

#[test]
fn the_toilet_before_bed_and_on_getting_up() {
    for seed in 0..3 {
        let mut w = world(seed, 22.0);
        // Tired, a bladder that would see the night through, everything else full.
        full_but(&mut w, &[("energy", 0.2), ("bladder", 0.7)]);
        let mut seen = Vec::new();
        let end = clock::tick_at(2, 10.0 * 60.0).unwrap();
        while w.tick < end {
            w.tick_once();
            if let Some((o, id)) = using(&w)
                && seen.last() != Some(&(o, id.clone()))
            {
                seen.push((o, id));
            }
        }
        let first_sleep = seen.iter().position(|(o, id)| *o == BED && id == "sleep").expect("a night's sleep");
        assert!(seen[..first_sleep].iter().any(|(o, _)| *o == TOILET), "seed {seed}: the toilet before bed: {seen:?}");
        let last_sleep = seen.iter().rposition(|(o, id)| *o == BED && id == "sleep").unwrap();
        assert_eq!(seen.get(last_sleep + 1).map(|s| s.0), Some(TOILET), "seed {seed}: the toilet on getting up: {seen:?}");
    }
}
