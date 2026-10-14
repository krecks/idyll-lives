//! Things standing on tops (content `layer: "surface"`): a vase on a table, a lamp on a
//! nightstand, a book on a counter. Each stands wholly on one top, clear of its edges and of
//! anything else standing there; it goes where its top goes, and is sold with it.

mod common;

use sim_core::surface::{host_of, middle};
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
    "objects":[
      {"id":"table","name":"Table","price":200,"footprint":[2,1],"surface":{"height":0.75,"kind":"table"}},
      {"id":"nightstand","name":"Nightstand","price":80,"surface":{"height":0.55,"kind":"low"}},
      {"id":"counter","name":"Counter","price":150,"surface":{"height":0.9,"kind":"counter","size":[1,0.6],"at":[0,-0.2]},
       "interactions":[{"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.2}}]},
      {"id":"vase","name":"Vase","price":30,"layer":"surface","radius":0.1,"category":"decor"},
      {"id":"lamp","name":"Lamp","price":40,"layer":"surface","radius":0.12,"category":"decor","light":{"range":3}},
      {"id":"book","name":"Book","price":10,"layer":"surface","radius":0.08}],
    "roomRules":{"decorByCategory":{"decor":1}},
    "economy":{"startingFunds":5000}}"#;

/// One room (x 1..5, z 1..4) with a door at the bottom, on a 12×10 plot.
const TOWN: &str = r#"{"width":12,"depth":12,
    "walls":[[1,1,6,1],[1,5,6,5],[1,1,1,5],[6,1,6,5]],
    "doors":[{"x":3,"z":5,"axis":"x"}],
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]}],
    "households":[{"name":"Player","plot":0,"player":true}]}"#;

fn world() -> World {
    World::from_json(CONTENT, TOWN, 1).unwrap()
}

fn buy(w: &mut World, def: &str, x: i32, z: i32, rot: i32, offset: [i8; 2]) -> Result<u32, sim_core::Error> {
    w.apply(Command::Buy { household: 0, object: def.into(), at: Some([x, z, rot]), color: None, turn: None, offset: Some(offset), size: None, tall: None })?;
    Ok(w.objects.len() as u32 - 1)
}

fn find(w: &World, def: &str) -> Vec<u32> {
    w.objects.iter().filter(|o| w.content.objects[o.def].id == def).map(|o| o.id).collect()
}

fn near(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 0.011 && (a[1] - b[1]).abs() < 0.011
}

#[test]
fn small_things_stand_on_tables_nightstands_and_counters() {
    let mut w = world();
    let table = buy(&mut w, "table", 2, 2, 0, [0, 0]).unwrap();
    buy(&mut w, "nightstand", 5, 1, 0, [0, 0]).unwrap();
    buy(&mut w, "counter", 1, 1, 0, [0, 0]).unwrap();
    let vase = buy(&mut w, "vase", 2, 2, 0, [-20, 10]).expect("a vase on the table");
    let lamp = buy(&mut w, "lamp", 5, 1, 0, [0, 0]).expect("a lamp on the nightstand");
    // The counter's top is at the back of its tile: the middle of the tile is too near its front.
    let err = buy(&mut w, "book", 1, 1, 0, [0, 0]).unwrap_err().to_string();
    assert!(err.contains("edge"), "{err}");
    let book = buy(&mut w, "book", 1, 1, 0, [0, -20]).expect("a book at the back of the counter");
    assert_eq!(w.objects[vase as usize].offset, [-20, 10]);
    for (thing, host) in [(vase, table), (lamp, 1), (book, 2)] {
        assert_eq!(host_of(&w.content, &w.objects, &w.objects[thing as usize]), Some(host));
    }
    // They take no floor: only the table, the nightstand and the counter do.
    assert_eq!(w.blocked().iter().filter(|b| **b).count(), 4);
    // Shown standing on the top.
    let view: serde_json::Value = serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    let lift = |id: u32| view["objects"][id as usize]["lift"].as_f64();
    assert_eq!(lift(vase), Some(0.75));
    assert_eq!(lift(lamp).map(|l| (l * 100.0).round()), Some(55.0));
    assert_eq!(lift(table), None, "furniture stands on the floor");
    common::check(&w);
}

#[test]
fn no_floor_no_crowding_no_hanging_over_the_edge() {
    let mut w = world();
    buy(&mut w, "table", 2, 2, 0, [0, 0]).unwrap();
    buy(&mut w, "nightstand", 4, 1, 0, [0, 0]).unwrap();
    buy(&mut w, "nightstand", 5, 1, 0, [0, 0]).unwrap();
    let err = buy(&mut w, "vase", 3, 4, 0, [0, 0]).unwrap_err().to_string();
    assert!(err.contains("table"), "on the floor: {err}");
    buy(&mut w, "vase", 2, 2, 0, [0, 0]).unwrap();
    let err = buy(&mut w, "vase", 2, 2, 0, [15, 0]).unwrap_err().to_string();
    assert!(err.contains("no room"), "on top of another: {err}");
    buy(&mut w, "vase", 2, 2, 0, [25, 0]).expect("just beside it");
    let err = buy(&mut w, "vase", 3, 2, 0, [0, 45]).unwrap_err().to_string();
    assert!(err.contains("edge"), "over the rim: {err}");
    // Two nightstands side by side are still two tops.
    let err = buy(&mut w, "book", 4, 1, 0, [45, 0]).unwrap_err().to_string();
    assert!(err.contains("edge"), "across two tops: {err}");
    // And the turn is kept, like any freely turning thing's.
    w.apply(Command::Buy { household: 0, object: "book".into(), at: Some([4, 1, 1]), color: None, turn: Some(40), offset: Some([5, -5]), size: None, tall: None }).unwrap();
    let book = w.objects.last().unwrap();
    assert_eq!((book.rot, book.turn, book.offset), (1, 40, [5, -5]));
    // Bought with no place given, it goes on a top with room for it.
    w.apply(Command::Buy { household: 0, object: "lamp".into(), at: None, color: None, turn: None, offset: None, size: None, tall: None }).unwrap();
    assert!(host_of(&w.content, &w.objects, w.objects.last().unwrap()).is_some());
    common::check(&w);
}

/// A table at (2, 2) facing +z with a vase and a book on it.
fn laid_table() -> World {
    let mut w = world();
    buy(&mut w, "nightstand", 5, 1, 0, [0, 0]).unwrap();
    buy(&mut w, "table", 2, 2, 0, [0, 0]).unwrap();
    buy(&mut w, "vase", 2, 2, 0, [-20, 10]).unwrap();
    buy(&mut w, "book", 3, 2, 0, [10, 0]).unwrap();
    w
}

#[test]
fn moving_or_turning_a_table_carries_what_stands_on_it() {
    let mut w = laid_table();
    let before = w.objects.clone();
    // A quarter turn, and over to (4, 2): its middle goes from (3, 2.5) to (4.5, 3).
    w.apply(Command::MoveObject { household: 0, object: 1, x: 4, z: 2, rot: 1, turn: None, offset: None, size: None, tall: None }).unwrap();
    let table = find(&w, "table")[0] as usize;
    let (vase, book) = (find(&w, "vase")[0] as usize, find(&w, "book")[0] as usize);
    assert!(table < vase && table < book, "riders come after their table");
    // Each stands where it stood on the top, turned with it (local (x, z) → (z, -x)).
    assert!(near(middle(&w.content, &w.objects[vase]), [4.5 + 0.1, 3.0 + 0.7]), "{:?}", middle(&w.content, &w.objects[vase]));
    assert!(near(middle(&w.content, &w.objects[book]), [4.5, 3.0 - 0.6]), "{:?}", middle(&w.content, &w.objects[book]));
    assert_eq!(w.objects[vase].rot, 1, "turned with the table");
    common::check(&w);

    // One undo step puts it all back, and redo moves it all again.
    let after = w.objects.clone();
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!(w.objects, before);
    w.apply(Command::Redo { household: 0 }).unwrap();
    assert_eq!(w.objects, after);

    // Saved and loaded with everything on its top.
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(loaded.objects.len(), w.objects.len());
    for (a, b) in loaded.objects.iter().zip(&w.objects) {
        assert_eq!((a.def, a.x, a.z, a.rot, a.offset), (b.def, b.x, b.z, b.rot, b.offset));
    }
    common::check(&loaded);

    // Where it doesn't fit, nothing moves.
    let snapshot = w.objects.clone();
    let funds = w.households[0].funds;
    assert!(w.apply(Command::MoveObject { household: 0, object: table as u32, x: 5, z: 1, rot: 0, turn: None, offset: None, size: None, tall: None }).is_err());
    assert_eq!((w.objects.clone(), w.households[0].funds), (snapshot, funds));
}

#[test]
fn selling_a_table_sells_what_stands_on_it() {
    let mut w = laid_table();
    let before = (w.objects.clone(), w.households[0].funds);
    let resale = w.content.object_rules.resale;
    w.apply(Command::Sell { household: 0, object: 1 }).unwrap();
    assert_eq!(w.objects.len(), 1, "only the nightstand is left");
    let refund: i64 = [200.0f32, 30.0, 10.0].iter().map(|p| (p * resale).round() as i64).sum();
    assert_eq!(w.households[0].funds, before.1 + refund);
    let after = (w.objects.clone(), w.households[0].funds);
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!((w.objects.clone(), w.households[0].funds), before, "one undo step");
    w.apply(Command::Redo { household: 0 }).unwrap();
    assert_eq!((w.objects.clone(), w.households[0].funds), after);
    // Selling just the vase leaves the table be.
    w.apply(Command::Undo { household: 0 }).unwrap();
    w.apply(Command::Sell { household: 0, object: 2 }).unwrap();
    assert_eq!(find(&w, "table").len(), 1);
    common::check(&w);
}

#[test]
fn a_blueprint_turned_round_keeps_things_on_their_tops() {
    // The east lot faces the other way: a blueprint from the west lot is built turned round.
    const TWO_LOTS: &str = r#"{"width":24,"depth":12,
        "plots":[{"name":"West","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]},
                 {"name":"East","x":12,"z":0,"w":12,"d":10,"entry":[18.5,0.5]}],
        "households":[{"name":"Player","plot":0,"player":true},{"name":"Other","plot":1}]}"#;
    let mut w = World::from_json(CONTENT, TWO_LOTS, 1).unwrap();
    buy(&mut w, "table", 2, 2, 0, [0, 0]).unwrap();
    buy(&mut w, "vase", 2, 2, 0, [-20, 10]).unwrap();
    buy(&mut w, "counter", 6, 6, 0, [0, 0]).unwrap();
    buy(&mut w, "book", 6, 6, 0, [10, -25]).unwrap();
    let bp = w.blueprint(0).unwrap();
    w.apply(Command::BuildBlueprint { household: 1, blueprint: bp.clone() }).unwrap();
    let built: Vec<&sim_core::world::ObjectInstance> = w.objects.iter().filter(|o| o.x >= 12).collect();
    assert_eq!(built.len(), 4, "all of it, the vase and the book too");
    common::check(&w);
    // Turned round onto the other lot: a middle at (x, z) goes to (24 - x, 10 - z).
    let vase = built.iter().find(|o| w.content.objects[o.def].id == "vase").unwrap();
    let was = middle(&w.content, &w.objects[1]);
    assert!(near(middle(&w.content, vase), [24.0 - was[0], 10.0 - was[1]]), "{:?}", middle(&w.content, vase));
}

#[test]
fn moving_a_room_carries_its_tables_and_what_stands_on_them() {
    let mut w = laid_table();
    let before: Vec<[f32; 2]> = w.objects.iter().map(|o| middle(&w.content, o)).collect();
    w.apply(Command::MoveRoom { household: 0, x: 2, z: 2, dx: 4, dz: 0 }).unwrap();
    common::check(&w);
    let mut after: Vec<[f32; 2]> = w.objects.iter().map(|o| middle(&w.content, o)).collect();
    let mut moved: Vec<[f32; 2]> = before.iter().map(|p| [p[0] + 4.0, p[1]]).collect();
    let key = |p: &[f32; 2]| ((p[0] * 100.0).round() as i32, (p[1] * 100.0).round() as i32);
    after.sort_by_key(key);
    moved.sort_by_key(key);
    assert_eq!(after.iter().map(key).collect::<Vec<_>>(), moved.iter().map(key).collect::<Vec<_>>());
}

#[test]
fn decor_and_lamps_on_tops_count_in_their_room() {
    let mut w = laid_table();
    let room = |w: &World| {
        let r = w.room_at_tile(3, 3).unwrap();
        (r.decor, r.lamps)
    };
    let (decor, lamps) = room(&w);
    buy(&mut w, "lamp", 3, 2, 0, [-30, 0]).unwrap();
    let (more, lit) = room(&w);
    assert!(more > decor, "{more} > {decor}");
    assert_eq!(lit, lamps + 1);
}
