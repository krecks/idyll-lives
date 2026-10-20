//! Stable object ids (0.3.0): an object keeps the id it was made with through moves, sales,
//! undo/redo and save/reload, and tasks keep pointing at the same object.

mod common;

use std::collections::HashSet;

use serde_json::{Value, json};
use sim_core::world::TaskKind;
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"comfort","label":"Comfort","decayPerHour":0.01}],
    "objects":[
      {"id":"chair","name":"Chair","price":10,"interactions":[
        {"id":"sit","label":"Sit","minutes":60,"pose":"sit","effects":{"comfort":0.5}}]},
      {"id":"table","name":"Table","price":20,"interactions":[]}],
    "economy":{"startingFunds":10000}}"#;

const TOWN: &str = r#"{"width":14,"depth":14,
    "plots":[{"name":"Home","x":0,"z":0,"w":14,"d":12,"entry":[6.5,11.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "sims":[{"name":"Ada","x":6.5,"z":11.5}]}"#;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    w
}

fn buy(w: &mut World, def: &str, x: i32, z: i32) -> u32 {
    w.apply(Command::Buy { household: 0, object: def.into(), at: Some([x, z, 0]), colors: Default::default(), turn: None, offset: None, size: None, tall: None }).unwrap();
    w.objects.last().unwrap().stable
}

fn at(w: &World, id: u32) -> &sim_core::world::ObjectInstance {
    &w.objects[w.index_of(id).expect("the object is there")]
}

fn task_object(kind: &TaskKind) -> Option<u32> {
    match *kind {
        TaskKind::Use { object, .. } | TaskKind::Repair { object } => Some(object),
        TaskKind::Cook { station, .. } | TaskKind::Dishes { station, .. } => station,
        TaskKind::Eat { seat, .. } | TaskKind::Phone { seat, .. } => seat,
        _ => None,
    }
}

#[test]
fn moving_selling_and_reloading_keeps_every_other_id() {
    let mut w = world();
    let a = buy(&mut w, "chair", 2, 2);
    let b = buy(&mut w, "chair", 4, 2);
    let c = buy(&mut w, "chair", 6, 2);
    assert_eq!([a, b, c].into_iter().collect::<HashSet<_>>().len(), 3);

    w.apply(Command::MoveObject { household: 0, object: a, x: 2, z: 4, rot: 0, turn: None, offset: None, size: None, tall: None }).unwrap();
    w.apply(Command::Sell { household: 0, object: b }).unwrap();
    assert_eq!((at(&w, c).x, at(&w, c).z), (6, 2), "the third chair didn't move");
    assert_eq!(at(&w, a).stable, a, "the moved chair kept its id");

    // Undo and redo keep the ids too: first the sale, then the move.
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!(at(&w, b).stable, b, "the sold chair is back, with its id");
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!((at(&w, a).x, at(&w, a).z), (2, 2));
    w.apply(Command::Redo { household: 0 }).unwrap();
    assert_eq!((at(&w, a).x, at(&w, a).z), (2, 4));
    w.apply(Command::Redo { household: 0 }).unwrap();

    // Reloaded, the same ids name the same things.
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!((at(&loaded, a).x, at(&loaded, a).z), (2, 4));
    assert_eq!((at(&loaded, c).x, at(&loaded, c).z), (6, 2));
    assert!(loaded.index_of(b).is_none(), "the sold chair is gone");
}

#[test]
fn a_queued_order_still_names_the_same_object_after_an_edit_and_a_reload() {
    let mut w = world();
    let extra = buy(&mut w, "table", 1, 1);
    let a = buy(&mut w, "chair", 2, 2);
    let b = buy(&mut w, "chair", 4, 2);
    w.apply(Command::Use { sim: 0, object: a, interaction: 0 }).unwrap();
    w.apply(Command::Use { sim: 0, object: b, interaction: 0 }).unwrap();
    for _ in 0..5 {
        w.tick_once();
    }

    // Selling something listed before them shifts their places in the list.
    w.apply(Command::Sell { household: 0, object: extra }).unwrap();
    let current = w.sims[0].current().expect("walking to the first chair");
    assert_eq!(task_object(&current.task.kind).map(|i| w.objects[i as usize].stable), Some(a));
    let queued = w.sims[0].queue().next().expect("the second chair is queued");
    assert_eq!(task_object(&queued.kind).map(|i| w.objects[i as usize].stable), Some(b));

    // Saved and loaded, both still point at the same chairs.
    let mut loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    for _ in 0..5 {
        loaded.tick_once();
    }
    let current = loaded.sims[0].current().expect("walking to the first chair");
    assert_eq!(task_object(&current.task.kind).map(|i| loaded.objects[i as usize].stable), Some(a));
    let queued = loaded.sims[0].queue().next().expect("the second chair is queued");
    assert_eq!(task_object(&queued.kind).map(|i| loaded.objects[i as usize].stable), Some(b));
}

#[test]
fn an_old_save_without_ids_gets_them_on_load() {
    let mut w = world();
    let a = buy(&mut w, "chair", 2, 2);
    let b = buy(&mut w, "table", 4, 2);
    let mut save: Value = serde_json::from_str(&w.save_json()).unwrap();
    save["version"] = json!(28);
    for o in save["objects"].as_array_mut().unwrap() {
        o.as_object_mut().unwrap().remove("id");
    }
    let mut loaded = World::from_save_json(CONTENT, &save.to_string()).unwrap();
    let ids: Vec<u32> = loaded.objects.iter().map(|o| o.stable).collect();
    assert_eq!(ids.into_iter().collect::<HashSet<_>>().len(), loaded.objects.len(), "all different");
    assert_eq!((at(&loaded, a).x, at(&loaded, a).z), (2, 2));
    assert_eq!((at(&loaded, b).x, at(&loaded, b).z), (4, 2));

    // A new buy can't take an id an old object already has.
    let c = buy(&mut loaded, "chair", 6, 2);
    assert!(loaded.objects.iter().filter(|o| o.stable == c).count() == 1);
    assert!(c > loaded.objects.iter().map(|o| o.stable).max().unwrap() - 1);
}

#[test]
fn a_sold_highest_id_is_not_reused_after_a_reload() {
    let mut w = world();
    let _a = buy(&mut w, "chair", 2, 2);
    let b = buy(&mut w, "chair", 4, 2);
    w.apply(Command::Sell { household: 0, object: b }).unwrap();
    let mut loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    let c = buy(&mut loaded, "chair", 6, 2);
    assert!(c > b, "the sold chair's id stays retired ({c} after {b})");
}
