//! Curtains drawn and opened (`sim_core::curtains`): the player per pair, residents per room
//! (shut as someone goes to sleep in it, as night falls on someone awake there or someone awake
//! comes in at night; open as someone awake comes in by day or day breaks on them, unless someone
//! still sleeps there), saved; only at home with free will. One bought comes open; a new game's
//! are as fits the hour; a save opens with them as they were.

use crate::common;

use sim_core::{Command, TICKS_PER_MINUTE, World, clock, curtains};

const CONTENT: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.0}],
    "dayRhythm":{"sleepTags":["sleep"]},
    "objects":[
      {"id":"curtains","name":"Curtains","price":70,"layer":"wall","overWindow":true},
      {"id":"lamp","name":"Lamp","price":30,"light":{"range":4}},
      {"id":"bed","name":"Bed","price":100,"footprint":[2,2],"slots":2,"interactions":[{"id":"sleep","label":"Sleep","minutes":60,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"]}]}],
    "economy":{"startingFunds":1000}}"#;

/// Two rooms side by side, a door between them and one out, each with a window and curtains
/// over it: room A (x 1–4) with a lamp, room B (x 5–8) with a bed for two.
const TOWN: &str = r#"{"width":14,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":14,"d":12,"entry":[12.5,10.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "walls":[[1,1,9,1],[1,5,9,5],[1,1,1,5],[5,1,5,5],[9,1,9,5]],
    "doors":[{"x":5,"z":2,"axis":"z"},{"x":2,"z":5,"axis":"x"}],
    "windows":[{"x":3,"z":1,"axis":"x"},{"x":8,"z":1,"axis":"x"}],
    "objects":[{"def":"curtains","x":3,"z":1},{"def":"curtains","x":8,"z":1},
               {"def":"lamp","x":1,"z":1},{"def":"bed","x":6,"z":2}],
    "sims":[{"name":"Ada","x":12.5,"z":9.5},{"name":"Bo","x":12.5,"z":8.5}]}"#;

const A: u32 = 0;
const B: u32 = 1;
const LAMP: u32 = 2;
const BED: u32 = 3;

const OUTSIDE: [f32; 2] = [12.5, 9.5];
const IN_A: [f32; 2] = [3.5, 3.5];
const IN_B: [f32; 2] = [5.5, 4.5];

fn shut(w: &World, curtains: u32) -> bool {
    w.objects[curtains as usize].shut
}

fn draw(w: &mut World, curtains: u32, shut: bool) {
    w.apply(Command::DrawCurtains {
        object: curtains,
        shut,
    })
    .unwrap();
}

/// The world at `hour` on day 1 (its curtains as fit the hour), everyone rested and out in the
/// garden.
fn world_at(hour: f32) -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    set_hour(&mut w, hour);
    for s in &mut w.sims {
        s.needs[0] = 1.0;
    }
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

/// Puts resident `sim` at `at` and lets a moment pass.
fn go(w: &mut World, sim: usize, at: [f32; 2]) {
    w.sims[sim].pos = at;
    w.tick_once();
    tick(w, 1);
}

/// Sends resident `sim` to bed and waits till they're asleep.
fn sleep(w: &mut World, sim: usize) {
    w.sims[sim].needs[0] = 0.0;
    w.apply(Command::Use {
        sim: sim as u32,
        object: BED,
        interaction: 0,
    })
    .unwrap();
    for _ in 0..30 * TICKS_PER_MINUTE {
        w.tick_once();
        if w.sims[sim].asleep(&w.content) {
            tick(w, 1);
            return;
        }
    }
    panic!("{} doesn't fall asleep", w.sims[sim].name);
}

/// Wakes resident `sim` (the player cancels their sleep), rested.
fn wake(w: &mut World, sim: usize) {
    w.apply(Command::Cancel {
        sim: sim as u32,
        index: 0,
    })
    .unwrap();
    w.sims[sim].needs[0] = 1.0;
    tick(w, 1);
    assert!(!w.sims[sim].asleep(&w.content));
}

#[test]
fn the_player_draws_curtains() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    assert!(w.objects.iter().all(|o| !o.shut), "open as they come");
    let (structure, version) = (w.structure_version(), w.curtains_version());
    draw(&mut w, B, true);
    assert!(shut(&w, B) && !shut(&w, A));
    assert_eq!(curtains::shut(&w), vec![B]);
    assert_ne!(w.curtains_version(), version, "the renderer is told");
    assert_eq!(
        w.structure_version(),
        structure,
        "without rebuilding the house"
    );

    let w2 = common::reload(&w, CONTENT);
    assert!(shut(&w2, B) && !shut(&w2, A), "still drawn after loading");

    draw(&mut w, B, false);
    assert!(!shut(&w, B));
    for other in [LAMP, BED] {
        assert!(
            w.apply(Command::DrawCurtains {
                object: other,
                shut: true
            })
            .is_err_and(|e| e.to_string().contains("no curtains")),
            "a lamp or a bed has no curtains"
        );
    }

    // Bought: open, by day or by night.
    set_hour(&mut w, 22.0);
    let bought = common::buy(&mut w, "curtains", 6, 1);
    let bought = w.index_of(bought).unwrap();
    assert!(!w.objects[bought].shut, "bought open");
}

#[test]
fn night_falls_on_a_room_with_someone_awake_in_it() {
    let mut w = world_at(12.0);
    assert!(!shut(&w, A) && !shut(&w, B), "open by day");
    go(&mut w, 0, IN_A);
    assert!(!shut(&w, A), "still open: it's day");
    set_hour(&mut w, 18.9);
    tick(&mut w, 10);
    assert!(shut(&w, A), "drawn as night falls, Ada in the room");
    assert!(!shut(&w, B), "not in the empty room");

    // Into B at night: she draws its curtains too.
    go(&mut w, 0, IN_B);
    assert!(shut(&w, B), "drawn as she comes in after dark");
    // The player opens them with her in there: they stay open while she stays.
    draw(&mut w, B, false);
    tick(&mut w, 10);
    assert!(
        !shut(&w, B),
        "what the player set stays until the next moment"
    );
}

#[test]
fn coming_in_by_day_opens_them() {
    let mut w = world_at(12.0);
    draw(&mut w, A, true);
    draw(&mut w, B, true);
    tick(&mut w, 5);
    assert!(
        shut(&w, A) && shut(&w, B),
        "nobody comes: as the player left them"
    );
    go(&mut w, 0, IN_A);
    assert!(!shut(&w, A), "opened as Ada comes in by day");
    assert!(shut(&w, B), "not the next room's");
    draw(&mut w, A, true);
    tick(&mut w, 10);
    assert!(
        shut(&w, A),
        "drawn again by the player, they stay so while she stays"
    );
    go(&mut w, 0, OUTSIDE);
    assert!(shut(&w, A), "leaving opens nothing");
}

#[test]
fn going_to_sleep_draws_them_and_waking_by_day_opens_them() {
    let mut w = world_at(14.0);
    go(&mut w, 0, IN_B);
    assert!(!shut(&w, B));
    sleep(&mut w, 0);
    assert!(shut(&w, B), "drawn for a nap in the afternoon");
    assert!(!shut(&w, A), "not the room nobody sleeps in");
    wake(&mut w, 0);
    assert!(!shut(&w, B), "open again as she wakes by day");

    // At night: drawn as she goes to sleep, and still drawn when she's up again in the night.
    set_hour(&mut w, 23.0);
    tick(&mut w, 1);
    draw(&mut w, B, false);
    sleep(&mut w, 0);
    assert!(shut(&w, B), "drawn as she goes to bed");
    wake(&mut w, 0);
    assert!(shut(&w, B), "still night: they stay drawn");
}

#[test]
fn the_first_one_up_leaves_a_sleeper_s_curtains_drawn() {
    let mut w = world_at(23.0);
    go(&mut w, 0, IN_B);
    go(&mut w, 1, IN_B);
    sleep(&mut w, 0);
    sleep(&mut w, 1);
    assert!(shut(&w, B));
    // Morning: Ada gets up, Bo sleeps on.
    set_hour(&mut w, 7.0);
    tick(&mut w, 1);
    wake(&mut w, 0);
    assert!(shut(&w, B), "Bo still asleep there: they stay drawn");
    go(&mut w, 0, IN_A);
    assert!(
        !shut(&w, A),
        "the next room's opened as she comes in by day"
    );
    wake(&mut w, 1);
    assert!(!shut(&w, B), "and B's as Bo gets up");
}

#[test]
fn day_breaks_on_a_room_with_someone_awake_in_it() {
    let mut w = world_at(5.0);
    assert!(
        shut(&w, A) && shut(&w, B),
        "a new game's are drawn at night"
    );
    go(&mut w, 0, IN_A);
    assert!(shut(&w, A), "still dark: they stay drawn");
    set_hour(&mut w, 6.4);
    tick(&mut w, 10);
    assert!(!shut(&w, A), "opened as day breaks, Ada up in the room");
    assert!(shut(&w, B), "not in the empty room");
}

#[test]
fn a_new_games_curtains_are_as_fits_the_hour() {
    let w = world_at(12.0);
    assert!(!shut(&w, A) && !shut(&w, B), "open by day");
    let w = world_at(21.0);
    assert!(shut(&w, A) && shut(&w, B), "drawn at night");
    let mut w2 = common::reload(&w, CONTENT);
    set_hour(&mut w2, 12.0);
    tick(&mut w2, 1);
    assert!(
        shut(&w2, A) && shut(&w2, B),
        "a save opens with them as they were, nobody coming in"
    );
}

#[test]
fn without_free_will_the_curtains_stay_as_they_were() {
    let mut w = world_at(18.5);
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    go(&mut w, 0, IN_A);
    set_hour(&mut w, 18.9);
    tick(&mut w, 10);
    assert!(!shut(&w, A), "nobody draws them at nightfall");
    go(&mut w, 0, IN_B);
    sleep(&mut w, 0);
    assert!(!shut(&w, B), "nor going to bed");
}

#[test]
fn moved_curtains_stay_drawn() {
    let mut w = world_at(12.0);
    draw(&mut w, A, true);
    w.apply(Command::MoveRoom {
        household: 0,
        x: 2,
        z: 3,
        dx: 0,
        dz: 5,
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let moved = |w: &World| {
        w.objects
            .iter()
            .find(|o| w.content.objects[o.def].id == "curtains" && (o.x, o.z) == (3, 6))
            .map(|o| o.shut)
    };
    assert_eq!(moved(&w), Some(true), "moved with its room, still drawn");
    w.apply(Command::Undo { household: 0 }).unwrap();
    w.apply(Command::Redo { household: 0 }).unwrap();
    assert_eq!(moved(&w), Some(true), "and after undo and redo");
}

/// The shipped curtains are drawn; with `DUMP_STATES=1`, writes `curtains-day` and
/// `curtains-night`: a room with each kind of curtains over its windows, open by day and drawn
/// at night with Ada in it (for the browser).
#[test]
fn shipped_curtains_by_day_and_night() {
    let content = common::content();
    let town = r#"{"width":16,"depth":12,
        "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":12,"entry":[13.5,10.5]}],
        "households":[{"name":"Player","plot":0,"player":true}],
        "walls":[[1,1,11,1],[1,6,11,6],[1,1,1,6],[11,1,11,6]],
        "doors":[{"x":2,"z":1,"axis":"x"}],
        "windows":[{"x":3,"z":6,"axis":"x"},{"x":5,"z":6,"axis":"x"},{"x":6,"z":6,"axis":"x"},
                   {"x":8,"z":6,"axis":"x"},{"x":9,"z":6,"axis":"x"}],
        "objects":[{"def":"curtains","x":3,"z":5,"rot":2},{"def":"curtainsWide","x":5,"z":5,"rot":2},
                   {"def":"curtainsLong","x":8,"z":5,"rot":2},{"def":"ceilingPendant","x":5,"z":3},
                   {"def":"lamp","x":10,"z":1}],
        "sims":[{"name":"Ada","x":5.5,"z":3.5,"traits":["neat"]}]}"#;
    let w = World::from_json(&content, town, 1).unwrap();
    for id in ["curtains", "curtainsWide", "curtainsLong"] {
        let def = w.content.object_index(id).unwrap();
        assert!(curtains::is_curtain(&w, def), "{id} draws");
    }
    for (hour, name) in [(11.0, "curtains-day"), (21.0, "curtains-night")] {
        let mut w = World::from_json(&content, town, 1).unwrap();
        assert_eq!(w.objects.len(), 5, "every one placed");
        w.autonomy = false;
        set_hour(&mut w, hour);
        tick(&mut w, 1);
        let drawn = hour > 12.0;
        assert!(
            (0..3).all(|c| shut(&w, c) == drawn),
            "{name}: as fits the hour"
        );
        common::dump_state(&w, name);
    }
}
