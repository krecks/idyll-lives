//! Lamps switched on and off (`sim_core::lights`): the player per lamp, residents per room as
//! they come and go after dark (on into a dark room, off as the last one awake leaves or goes to
//! sleep), saved; fires and the garden aren't theirs to switch.

mod common;

use sim_core::{Command, TICKS_PER_MINUTE, World, clock, lights};

const CONTENT: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.0}],
    "dayRhythm":{"sleepTags":["sleep"]},
    "objects":[
      {"id":"lamp","name":"Lamp","price":30,"light":{"range":4}},
      {"id":"fire","name":"Fire","price":300,"light":{"range":3,"switch":false}},
      {"id":"bed","name":"Bed","price":100,"footprint":[2,2],"interactions":[{"id":"sleep","label":"Sleep","minutes":60,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"]}]}],
    "economy":{"startingFunds":1000}}"#;

/// Two rooms side by side, a door between them and one out: room A (x 1–4) with two lamps and a
/// fire, room B (x 5–8) with a lamp and a bed; a lamp in the garden.
const TOWN: &str = r#"{"width":14,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":14,"d":12,"entry":[12.5,10.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "walls":[[1,1,9,1],[1,5,9,5],[1,1,1,5],[5,1,5,5],[9,1,9,5]],
    "doors":[{"x":5,"z":2,"axis":"z"},{"x":2,"z":5,"axis":"x"}],
    "objects":[{"def":"lamp","x":1,"z":1},{"def":"lamp","x":4,"z":1},{"def":"fire","x":2,"z":1},
               {"def":"lamp","x":8,"z":1},{"def":"bed","x":6,"z":2},{"def":"lamp","x":12,"z":8}],
    "sims":[{"name":"Ada","x":12.5,"z":9.5}]}"#;

const A: [u32; 2] = [0, 1];
const FIRE: u32 = 2;
const B: u32 = 3;
const BED: u32 = 4;
const GARDEN: u32 = 5;

const OUTSIDE: [f32; 2] = [12.5, 9.5];
const IN_A: [f32; 2] = [3.5, 3.5];
const IN_B: [f32; 2] = [5.5, 4.5];

fn on(w: &World, lamp: u32) -> bool {
    !w.objects[lamp as usize].off
}

/// The world at `hour` on day 1, every lamp switched off.
fn world_at(hour: f32) -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    set_hour(&mut w, hour);
    for lamp in [A[0], A[1], B, GARDEN] {
        w.apply(Command::SwitchLamp { object: lamp, on: false }).unwrap();
    }
    w.sims[0].needs[0] = 1.0;
    tick(&mut w, 1);
    w
}

fn set_hour(w: &mut World, hour: f32) {
    let now = clock::minute_of_day(w.tick);
    let ahead = (hour * 60.0 - now).rem_euclid(1440.0);
    w.tick += (ahead * TICKS_PER_MINUTE as f32).round() as u64;
}

fn tick(w: &mut World, minutes: u64) {
    for _ in 0..minutes * TICKS_PER_MINUTE {
        w.tick_once();
    }
}

/// Puts Ada at `at` and lets a moment pass.
fn go(w: &mut World, at: [f32; 2]) {
    w.sims[0].pos = at;
    w.tick_once();
    tick(w, 1);
}

#[test]
fn the_player_switches_a_lamp() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    assert!(w.objects.iter().all(|o| !o.off), "bought on");
    let (structure, lights) = (w.structure_version(), w.lights_version());
    w.apply(Command::SwitchLamp { object: B, on: false }).unwrap();
    assert!(!on(&w, B));
    assert_eq!(lights::lamps_off(&w), vec![B]);
    assert_ne!(w.lights_version(), lights, "the renderer is told");
    assert_eq!(w.structure_version(), structure, "without rebuilding the house");

    let w2 = common::reload(&w, CONTENT);
    assert!(!on(&w2, B), "still off after loading");
    assert!(on(&w2, A[0]));

    w.apply(Command::SwitchLamp { object: B, on: true }).unwrap();
    assert!(on(&w, B));
    let fire = w.apply(Command::SwitchLamp { object: FIRE, on: false });
    assert!(fire.is_err_and(|e| e.to_string().contains("no switch")), "a fire isn't a lamp");
    assert!(w.apply(Command::SwitchLamp { object: BED, on: false }).is_err(), "nor is a bed");
}

#[test]
fn a_lamp_switched_off_stays_off_moved_undone_and_redone() {
    let mut w = world_at(12.0);
    let moved = |w: &World| w.objects.iter().find(|o| o.def == w.objects[GARDEN as usize].def && (o.x, o.z) == (11, 9)).map(|o| o.id);
    w.apply(Command::MoveObject { household: 0, object: GARDEN, x: 11, z: 9, rot: 0, turn: None, offset: None, size: None, tall: None }).unwrap();
    let lamp = moved(&w).expect("moved");
    assert!(!on(&w, lamp), "moving it doesn't turn it on");
    assert!(lights::lamps_off(&w).contains(&lamp));
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert!(moved(&w).is_none() && !on(&w, GARDEN), "back where it was, off");
    w.apply(Command::Redo { household: 0 }).unwrap();
    assert!(!on(&w, moved(&w).expect("moved again")), "moved again, still off");

    // A room moved with its lamps: they stay as they were.
    w.apply(Command::SwitchLamp { object: A[1], on: true }).unwrap();
    w.apply(Command::MoveRoom { household: 0, x: 6, z: 3, dx: 0, dz: 5 }).unwrap_or_else(|e| panic!("{e}"));
    let b = w.objects.iter().find(|o| (o.x, o.z) == (8, 6)).expect("B's lamp moved with its room");
    assert!(b.off, "B's lamp is still off");
    assert!(on(&w, A[1]) && !on(&w, A[0]), "A's are as they were");
    w.apply(Command::Undo { household: 0 }).unwrap();
    w.apply(Command::Redo { household: 0 }).unwrap();
    assert!(w.objects.iter().find(|o| (o.x, o.z) == (8, 6)).is_some_and(|o| o.off), "and after undo and redo");
}

#[test]
fn coming_in_after_dark_turns_the_room_on_and_leaving_turns_it_off() {
    let mut w = world_at(21.0);
    tick(&mut w, 5);
    assert!(!on(&w, GARDEN), "the garden is the player's");

    go(&mut w, IN_A);
    assert!(A.iter().all(|&l| on(&w, l)), "both of room A's lamps go on as Ada comes in");
    assert!(!on(&w, B), "not the next room's");

    go(&mut w, IN_B);
    assert!(A.iter().all(|&l| !on(&w, l)), "off as she leaves A empty");
    assert!(on(&w, B), "on in B");

    // The player turns B's lamp off with Ada in there: it stays off while she stays.
    w.apply(Command::SwitchLamp { object: B, on: false }).unwrap();
    tick(&mut w, 10);
    assert!(!on(&w, B), "what the player set stays until someone comes or goes");
    w.apply(Command::SwitchLamp { object: B, on: true }).unwrap();

    go(&mut w, OUTSIDE);
    assert!(!on(&w, B), "off as she goes out");
    assert!(!on(&w, GARDEN), "the garden lamp stays as the player left it");
}

#[test]
fn a_room_with_a_lamp_on_is_left_as_it_is() {
    let mut w = world_at(21.0);
    w.apply(Command::SwitchLamp { object: A[0], on: true }).unwrap();
    go(&mut w, IN_A);
    assert!(on(&w, A[0]) && !on(&w, A[1]), "it isn't dark: the other lamp stays off");
}

#[test]
fn by_day_nothing_goes_on_until_it_gets_dark() {
    let mut w = world_at(12.0);
    go(&mut w, IN_A);
    assert!(A.iter().all(|&l| !on(&w, l)), "no lamps by day");
    set_hour(&mut w, 18.9);
    tick(&mut w, 10);
    assert!(A.iter().all(|&l| on(&w, l)), "on as night falls, Ada in the room");
    assert!(!on(&w, B), "not in the empty room");
}

#[test]
fn going_to_sleep_turns_the_room_off_and_waking_on_again() {
    let mut w = world_at(22.0);
    go(&mut w, IN_B);
    assert!(on(&w, B));
    w.sims[0].needs[0] = 0.0;
    w.apply(Command::Use { sim: 0, object: BED, interaction: 0 }).unwrap();
    let mut slept = false;
    for _ in 0..30 * TICKS_PER_MINUTE {
        w.tick_once();
        if w.sims[0].asleep(&w.content) {
            slept = true;
            break;
        }
    }
    assert!(slept, "Ada is asleep");
    tick(&mut w, 1);
    assert!(!on(&w, B), "lights out as she falls asleep");
    // Woken (by the player): up again in the night.
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    w.sims[0].needs[0] = 1.0;
    tick(&mut w, 1);
    assert!(!w.sims[0].asleep(&w.content) && on(&w, B), "and the light is on again: it's still night");
}

#[test]
fn without_free_will_the_lights_stay_as_they_were() {
    let mut w = world_at(21.0);
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    go(&mut w, IN_A);
    assert!(A.iter().all(|&l| !on(&w, l)), "nobody switches them on");
    w.apply(Command::SwitchLamp { object: A[0], on: true }).unwrap();
    go(&mut w, OUTSIDE);
    assert!(on(&w, A[0]), "nor off");
}

/// The shipped lamps switch and the fires don't; with `DUMP_STATES=1`, writes `lamps-night`: a
/// two-room house at night, room A's lamps on with Ada in it, room B's off (for the browser).
#[test]
fn shipped_lamps_at_night() {
    let content = common::content();
    let town = r#"{"width":16,"depth":12,
        "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":12,"entry":[13.5,10.5]}],
        "households":[{"name":"Player","plot":0,"player":true}],
        "walls":[[1,1,11,1],[1,6,11,6],[1,1,1,6],[6,1,6,6],[11,1,11,6]],
        "doors":[{"x":6,"z":3,"axis":"z"},{"x":2,"z":6,"axis":"x"}],
        "windows":[{"x":3,"z":6,"axis":"x"},{"x":8,"z":6,"axis":"x"}],
        "objects":[{"def":"lamp","x":1,"z":4},{"def":"wallSconce","x":3,"z":1},{"def":"ceilingPendant","x":3,"z":3},
                   {"def":"lampFloorArc","x":10,"z":1},{"def":"wallSconce","x":8,"z":1},{"def":"ceilingPendant","x":8,"z":3},
                   {"def":"fireplace","x":4,"z":1}],
        "sims":[{"name":"Ada","x":3.5,"z":4.5,"traits":["neat"]}]}"#;
    let mut w = World::from_json(&content, town, 1).unwrap();
    let fire = w.objects.iter().position(|o| w.content.objects[o.def].id == "fireplace").unwrap();
    assert!(!lights::switchable(&w, w.objects[fire].def), "a fire has no switch");
    for id in ["lamp", "wallSconce", "ceilingPendant", "lampFloorArc", "lampTableRound", "nightstandLamp", "lampPost"] {
        let def = w.content.object_index(id).unwrap();
        assert!(lights::switchable(&w, def), "{id} switches");
    }
    set_hour(&mut w, 21.0);
    for lamp in 3..6 {
        w.apply(Command::SwitchLamp { object: lamp, on: false }).unwrap();
    }
    tick(&mut w, 1);
    assert!((0..3).all(|l| on(&w, l)) && (3..6).all(|l| !on(&w, l)));
    common::dump_state(&w, "lamps-night");
}
