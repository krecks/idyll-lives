//! A night's sleep with the shipped content: one block, about 8 hours from tired (shorter in a
//! better bed, longer on a futon), nobody up in the middle of the night, and naps that don't
//! stand in for bed or come one after another.

mod common;

use sim_core::world::TaskKind;
use sim_core::social::EventKind;
use sim_core::{Command, World, clock, life};

/// One house plot: a double bed, a futon and a sofa, the resident by the bed.
const TOWN: &str = r#"{"width":14,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":14,"d":10,"entry":[7.5,9.5]}],
    "households":[{"name":"Home","plot":0,"player":true}],
    "objects":[{"def":"bed","x":1,"z":1,"rot":0},{"def":"bedFuton","x":6,"z":1,"rot":0},
               {"def":"sofaThree","x":10,"z":6,"rot":0}],
    "sims":[{"name":"Ada","traits":["bookworm"],"x":2.5,"z":4.5}]}"#;

const BED: u32 = 0;
const FUTON: u32 = 1;

fn world() -> World {
    World::from_json(&common::content(), TOWN, 7).unwrap()
}

fn need(w: &World, id: &str) -> usize {
    w.content.needs.iter().position(|n| n.id == id).unwrap()
}

fn interaction(w: &World, object: u32, id: &str) -> usize {
    let def = w.objects[object as usize].def;
    w.content.objects[def].interactions.iter().position(|i| i.id == id).unwrap()
}

fn until(w: &mut World, day: u32, hour: f32) {
    let target = clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < target {
        w.tick_once();
    }
}

/// Everything full but energy (nothing else to do, nothing to wake them).
fn rested_but(w: &mut World, energy: f32) {
    let n = w.content.needs.len();
    w.sims[0].needs[..n].fill(1.0);
    let e = need(w, "energy");
    w.sims[0].needs[e] = energy;
}

/// What they're using, if anything: `(object, interaction id)`.
fn using(w: &World) -> Option<(u32, String)> {
    w.sims[0].current().and_then(|a| match a.task.kind {
        TaskKind::Use { object, interaction } => {
            let def = w.objects[object as usize].def;
            Some((object, w.content.objects[def].interactions[interaction].id.clone()))
        }
        _ => None,
    })
}

/// Sends the resident to bed with `energy` at 22:30 on day 1 and returns the hour of the next
/// morning they get up (day 2; past 24 is the afternoon after).
fn night(object: u32, quality: u8, energy: f32) -> f32 {
    let mut w = world();
    w.autonomy = false;
    w.objects[object as usize].quality = quality;
    until(&mut w, 1, 22.5);
    rested_but(&mut w, energy);
    let sleep = interaction(&w, object, "sleep");
    w.apply(Command::Use { sim: 0, object, interaction: sleep }).unwrap();
    let mut slept = false;
    let end = clock::tick_at(2, 14.0 * 60.0).unwrap();
    while w.tick < end {
        w.tick_once();
        let in_bed = using(&w).is_some_and(|(o, i)| o == object && i == "sleep");
        if slept && !in_bed {
            return clock::hour(w.tick);
        }
        slept |= in_bed;
    }
    panic!("still in bed at 14:00");
}

#[test]
fn tired_out_they_sleep_about_eight_hours_less_on_a_better_bed() {
    let standard = night(BED, 0, 0.0);
    assert!((6.0..=7.0).contains(&standard), "up at {standard:.2} after a night from empty (8 h ±30 min)");
    let best = night(BED, 3, 0.0);
    assert!(best < standard - 0.5, "a 3-star bed: up at {best:.2}, not {standard:.2}");
    assert!(best >= 5.4, "but not in the middle of the night ({best:.2})");
    let futon = night(FUTON, 0, 0.0);
    assert!(futon > standard + 0.5, "a futon: up at {futon:.2}, not {standard:.2}");
    assert!(futon < 8.5, "about 9 hours on a futon ({futon:.2})");
}

#[test]
fn not_very_tired_they_still_sleep_until_near_morning() {
    let up = night(BED, 0, 0.7);
    assert!((5.4..=7.0).contains(&up), "rested by 1 am, up at {up:.2} (not in the middle of the night)");
}

#[test]
fn exhausted_in_the_evening_they_go_to_bed_not_for_a_nap() {
    for seed in 0..4 {
        let mut w = World::from_json(&common::content(), TOWN, seed).unwrap();
        w.autonomy = false;
        until(&mut w, 1, 18.0);
        rested_but(&mut w, 0.15);
        w.autonomy = true;
        let mut chose = None;
        while chose.is_none() && w.tick < clock::tick_at(1, 19.0 * 60.0).unwrap() {
            w.tick_once();
            chose = using(&w);
        }
        let (object, id) = chose.expect("they pick something within the hour");
        assert_eq!(id, "sleep", "seed {seed}: worn out at 18:00 they pick {id} on object {object}");
    }
}

#[test]
fn a_nap_is_not_followed_by_another() {
    let mut w = world();
    w.autonomy = false;
    until(&mut w, 1, 13.0);
    rested_but(&mut w, 0.35);
    let nap = interaction(&w, BED, "nap");
    w.apply(Command::Use { sim: 0, object: BED, interaction: nap }).unwrap();
    let naps = w.content.day_rhythm.nap_tags;
    let napping = |w: &World| w.sims[0].current().is_some_and(|a| a.tags & naps != 0 && matches!(a.task.kind, TaskKind::Use { .. }));
    while !napping(&w) {
        w.tick_once();
    }
    while napping(&w) {
        w.tick_once();
    }
    let woke = clock::hour(w.tick);
    // (Saved and loaded: the wait carries over.)
    let mut w = World::from_save_json(&common::content(), &w.save_json()).unwrap();
    assert!(w.save_json().contains("\"napMinutes\""));
    w.autonomy = true;
    // Still tired after it: another nap would tempt them, but not this soon.
    let e = need(&w, "energy");
    let end = clock::tick_at(1, 16.5 * 60.0).unwrap();
    while w.tick < end {
        w.sims[0].needs[e] = w.sims[0].needs[e].min(0.4);
        w.tick_once();
        assert!(!napping(&w), "another nap at {:.2}, after one that ended at {woke:.2}", clock::hour(w.tick));
    }
}

#[test]
fn a_late_night_doesnt_make_them_miss_work() {
    let mut w = world();
    w.autonomy = false;
    w.apply(Command::JoinCareer { sim: 0, career: 0, level: 0 }).unwrap();
    // To bed worn out at 1 am: rested only mid-morning, but work comes first.
    until(&mut w, 2, 1.0);
    rested_but(&mut w, 0.0);
    let sleep = interaction(&w, BED, "sleep");
    w.apply(Command::Use { sim: 0, object: BED, interaction: sleep }).unwrap();
    let job = w.sims[0].job.clone().unwrap();
    let (leave, start, _) = life::shift_today(&w.content, &job, &w.sims[0].skills, w.tick).expect("a workday");
    while w.tick < start {
        w.tick_once();
    }
    assert!(w.sims[0].away_until.is_some(), "at work by {:.2} (left at {:.2})", clock::hour(start), clock::hour(leave));
    assert!(w.events.iter().all(|e| e.kind != EventKind::MissedWork));
}
