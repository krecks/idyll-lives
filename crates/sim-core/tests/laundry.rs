//! The laundry as it is done (I-bathroom-07): machines that run on their own (content `runs`,
//! `machines.rs`). The washing is put in the washer (a few minutes), which washes it with nobody
//! there; once it's done someone takes it out and carries it on in the basket to the dryer (or the
//! line), which dries it the same way; the dry washing is taken out and folded at the basket, a
//! piece in the hands. A machine that's running is saved and loaded as it was.

mod common;

use sim_core::content::Content;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World, clock, snapshot};

/// The shipped laundry on one plot without walls: the washer, the dryer, the basket and the line.
const TOWN: &str = r#"{"width":18,"depth":14,
    "plots":[{"name":"Home","x":0,"z":0,"w":18,"d":14,"entry":[8.5,13.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[{"def":"laundryWasher","x":2,"z":1},{"def":"laundryDryer","x":4,"z":1},
               {"def":"laundryBasket","x":6,"z":1},{"def":"clothesline","x":10,"z":1}],
    "sims":[{"name":"Ada","traits":["bookworm"],"x":4.5,"z":8.5}]}"#;

const WASHER: u32 = 0;
const DRYER: u32 = 1;
const BASKET: u32 = 2;
const LINE: u32 = 3;

/// The shipped content and the town above at 9 am, Ada's needs full, nobody choosing for
/// themselves.
fn world() -> (World, String) {
    let content = common::content();
    let mut w = World::from_json(&content, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    let morning = clock::tick_at(1, 9.0 * 60.0).unwrap();
    while w.tick < morning {
        w.tick_once();
    }
    fill(&mut w);
    (w, content)
}

fn fill(w: &mut World) {
    let n = w.content.needs.len();
    for s in &mut w.sims {
        s.needs[..n].fill(1.0);
    }
}

fn interaction(w: &World, object: u32, id: &str) -> usize {
    let def = w.objects[object as usize].def;
    w.content.objects[def].interactions.iter().position(|i| i.id == id).unwrap_or_else(|| panic!("no {id}"))
}

fn order(w: &mut World, object: u32, id: &str) -> Result<(), String> {
    let interaction = interaction(w, object, id);
    let object = w.objects[object as usize].stable;
    w.apply(Command::Use { sim: 0, object, interaction }).map_err(|e| e.to_string())
}

/// Ticks up to `minutes` until `f` holds (checking the world each tick): whether it did.
fn until(w: &mut World, minutes: u32, mut f: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..minutes * sim_core::TICKS_PER_MINUTE as u32 {
        w.tick_once();
        common::check(w);
        if f(w) {
            return true;
        }
    }
    false
}

/// What Ada is doing at which object: (object, interaction id, using it yet).
fn doing(w: &World) -> Option<(u32, String, bool)> {
    let a = w.sims[0].current()?;
    let TaskKind::Use { object, interaction } = a.task.kind else { return None };
    let id = w.content.objects[w.objects[object as usize].def].interactions[interaction].id.clone();
    Some((object, id, matches!(a.phase, Phase::Using { .. })))
}

fn using(w: &World, object: u32, id: &str) -> bool {
    doing(w).is_some_and(|(o, i, at)| o == object && i == id && at)
}

/// What the snapshot shows Ada carrying (an item id).
fn shown_held(w: &World) -> Option<String> {
    let mut buf = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut buf);
    let held = buf[snapshot::HEADER_LEN + snapshot::sim::HELD] as usize;
    (held > 0).then(|| w.content.items[held - 1].id.clone())
}

#[test]
fn the_washing_is_loaded_washes_on_its_own_and_goes_on_to_the_dryer_and_the_basket() {
    let (mut w, _) = world();
    let [dirty, washed, dried] = w.households[0].laundry;
    assert!(dirty >= 1.0 && washed == 1.0 && dried == 0.0, "a dirty load and a washed one to start with: {:?}", w.households[0].laundry);
    assert!(order(&mut w, WASHER, "unload").is_err_and(|e| e.contains("nothing in the washing machine")));
    order(&mut w, WASHER, "laundry").unwrap();
    // On the way, the basket of washing in the arms.
    assert!(until(&mut w, 2, |w| doing(w).is_some_and(|(_, _, at)| !at)));
    assert_eq!(shown_held(&w).as_deref(), Some("washingBasket"), "carried to the washer");
    assert!(until(&mut w, 10, |w| using(w, WASHER, "laundry")), "loading it");
    let start = w.tick;
    assert!(until(&mut w, 10, |w| w.sims[0].current().is_none()), "done loading");
    let minutes = (w.tick - start) as f32 / sim_core::TICKS_PER_MINUTE as f32;
    assert!(minutes <= 6.0, "a few minutes' loading, not the wash: {minutes}");
    assert!(w.objects[WASHER as usize].running(), "it runs on its own");
    assert!(w.households[0].laundry[0] < 0.1, "the dirty load is in it: {:?}", w.households[0].laundry);
    // Nobody at it, it washes; nothing more goes in, nor comes out, till it's done.
    assert!(order(&mut w, WASHER, "laundry").is_err_and(|e| e.contains("still running")));
    assert!(order(&mut w, WASHER, "unload").is_err_and(|e| e.contains("still running")));
    let view: serde_json::Value = serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    assert_eq!(view["objects"][WASHER as usize]["running"], true, "the renderer turns the drum");
    assert!(until(&mut w, 70, |w| w.objects[WASHER as usize].done()), "done in its hour");
    let view: serde_json::Value = serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    assert!(view["objects"][WASHER as usize].get("running").is_none(), "and stops");
    assert!(order(&mut w, WASHER, "laundry").is_err_and(|e| e.contains("take the washing out first")));

    // Taken out, the wet washing is carried on to the dryer, which is loaded and runs.
    fill(&mut w);
    order(&mut w, WASHER, "unload").unwrap();
    assert!(until(&mut w, 10, |w| using(w, WASHER, "unload")));
    assert!(until(&mut w, 10, |w| doing(w).is_some_and(|(o, _, _)| o == DRYER)), "on to the dryer: {:?}", w.sims[0].current());
    assert!(w.objects[WASHER as usize].run.is_none(), "the washer empty again");
    assert_eq!(shown_held(&w).as_deref(), Some("washingBasket"), "the wet washing in the basket");
    assert!(until(&mut w, 10, |w| using(w, DRYER, "laundry")), "loading the dryer");
    assert!(until(&mut w, 10, |w| w.sims[0].current().is_none()));
    assert!(w.objects[DRYER as usize].running(), "the dryer runs on its own");
    assert!(until(&mut w, 50, |w| w.objects[DRYER as usize].done()), "dry in its 45 minutes");

    // The dry washing taken out, it's folded at the basket, a piece in the hands.
    fill(&mut w);
    let dried = w.households[0].laundry[2];
    order(&mut w, DRYER, "unload").unwrap();
    assert!(until(&mut w, 10, |w| using(w, DRYER, "unload")));
    assert!(until(&mut w, 10, |w| doing(w).is_some_and(|(o, _, _)| o == BASKET)), "on to the basket: {:?}", w.sims[0].current());
    assert!(until(&mut w, 10, |w| using(w, BASKET, "fold")), "folding");
    assert_eq!(shown_held(&w).as_deref(), Some("washingPiece"), "the washing in the hands");
    w.tick_once();
    assert_eq!(w.households[0].laundry[2], dried, "the dry load made and taken up to fold");
    assert!(until(&mut w, 20, |w| w.sims[0].current().is_none()), "folded");
    assert_eq!(shown_held(&w), None);
    assert!(order(&mut w, BASKET, "fold").is_err_and(|e| e.contains("no dry laundry")), "nothing more to fold");
}

#[test]
fn the_washing_line_dries_in_the_sun_and_is_taken_in_to_fold() {
    let (mut w, _) = world();
    order(&mut w, LINE, "washing").unwrap();
    assert!(until(&mut w, 40, |w| w.sims[0].current().is_none()), "hung out");
    let line = &w.objects[LINE as usize];
    assert!(line.open && line.running(), "out on the line, drying");
    assert!(order(&mut w, LINE, "takeIn").is_err_and(|e| e.contains("still running")), "not dry yet");
    assert!(until(&mut w, 190, |w| w.objects[LINE as usize].done()), "dry in three hours");
    fill(&mut w);
    order(&mut w, LINE, "takeIn").unwrap();
    assert!(until(&mut w, 30, |w| doing(w).is_some_and(|(o, _, _)| o == BASKET)), "taken in, on to the basket");
    let line = &w.objects[LINE as usize];
    assert!(!line.open && line.run.is_none(), "the line empty");
    // Not hung out at night.
    let night = clock::tick_at(1, 22.0 * 60.0).unwrap();
    while w.tick < night {
        w.tick_once();
    }
    assert!(order(&mut w, LINE, "washing").is_err());
}

#[test]
fn a_running_machine_is_saved_and_loaded_as_it_was() {
    let (mut w, content) = world();
    order(&mut w, WASHER, "laundry").unwrap();
    assert!(until(&mut w, 15, |w| w.objects[WASHER as usize].running()));
    until(&mut w, 20, |_| false);
    let left = w.objects[WASHER as usize].run.unwrap();
    assert!(left > 30.0 && left < 60.0, "mid-wash: {left}");
    let saved = w.save_json();
    let json: serde_json::Value = serde_json::from_str(&saved).unwrap();
    assert_eq!(json["version"], 33);
    let mut loaded = World::from_save_json(&content, &saved).unwrap();
    assert_eq!(loaded.objects[WASHER as usize].run, Some(left), "running, as long left");
    assert_eq!(loaded.households[0].laundry, w.households[0].laundry);
    assert!(until(&mut loaded, left as u32 + 6, |w| w.objects[WASHER as usize].done()), "and done in its time");
    // Done, saved and loaded, it's still done with the washing in it.
    let again = World::from_save_json(&content, &loaded.save_json()).unwrap();
    assert!(again.objects[WASHER as usize].done());
}

#[test]
fn an_older_save_takes_in_the_washing_left_out_and_has_none_dry() {
    let (mut w, content) = world();
    w.objects[LINE as usize].open = true;
    w.objects[LINE as usize].run = None;
    let mut old: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
    old["version"] = 32.into();
    for h in old["households"].as_array_mut().unwrap() {
        h["laundry"] = serde_json::json!([2.0, 0.5]);
    }
    let loaded = World::from_save_json(&content, &old.to_string()).unwrap();
    assert!(loaded.objects[LINE as usize].done(), "the washing out on the line is dry");
    assert!(loaded.objects[WASHER as usize].run.is_none(), "the washer empty");
    assert_eq!(loaded.households[0].laundry, [2.0, 0.5, 0.0]);
}

#[test]
fn free_will_takes_the_washing_out_once_its_done() {
    let (mut w, _) = world();
    w.apply(Command::SetAutonomy { enabled: true, household: None }).unwrap();
    w.objects[WASHER as usize].run = Some(0.0);
    w.objects[DRYER as usize].run = Some(0.0);
    w.households[0].laundry = [0.0, 0.0, 0.0];
    let unloaded = until(&mut w, 12 * 60, |w| w.objects[WASHER as usize].run.is_none() && w.objects[DRYER as usize].run.is_none());
    assert!(unloaded, "both still full after half a day: washer {:?}, dryer {:?}", w.objects[WASHER as usize].run, w.objects[DRYER as usize].run);
}

#[test]
fn nothing_runs_forever() {
    let (mut w, _) = world();
    for o in [WASHER, DRYER, LINE] {
        w.objects[o as usize].run = Some(1000.0);
    }
    // A run longer than its kind's is cut to it on load; at most its time later each is done.
    let content = common::content();
    let loaded = World::from_save_json(&content, &w.save_json()).unwrap();
    for (o, minutes) in [(WASHER, 60.0), (DRYER, 45.0), (LINE, 180.0)] {
        assert_eq!(loaded.objects[o as usize].run, Some(minutes));
    }
    let mut w = loaded;
    assert!(until(&mut w, 190, |w| [WASHER, DRYER, LINE].iter().all(|&o| w.objects[o as usize].done())));
}

#[test]
fn machines_are_checked_at_load() {
    let base = r#"{"needs":[{"id":"comfort","label":"Comfort","decayPerHour":0.02}],
        "items":[{"id":"basket","label":"Basket"}],
        "laundry":{"perDay":0.4,"most":4},
        "objects":[{"id":"washer","name":"Washer","price":100,RUNS
          "interactions":[{"id":"load","label":"Load it","minutes":5,USE,"effects":{"comfort":0.1}}]}]}"#;
    let load = |runs: &str, use_: &str| Content::from_json(&base.replace("RUNS", runs).replace("USE", use_)).map(|_| ()).map_err(|e| e.to_string());
    load(r#""runs":{"minutes":60},"#, r#""loads":true,"brings":"basket""#).unwrap();
    assert!(load("", r#""loads":true"#).unwrap_err().contains("give the object `runs`"));
    assert!(load(r#""runs":{"minutes":0},"#, r#""loads":true"#).unwrap_err().contains("minutes > 0"));
    assert!(load(r#""runs":{"minutes":60},"#, r#""loads":true,"unloads":true"#).unwrap_err().contains("not both"));
    assert!(load(r#""runs":{"minutes":60},"#, r#""unloads":true,"makes":"dishes""#).unwrap_err().contains("a load of washing"));
    assert!(load(r#""runs":{"minutes":60},"#, r#""holds":"sock""#).unwrap_err().contains("unknown item 'sock'"));
}
