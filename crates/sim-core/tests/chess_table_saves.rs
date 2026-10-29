//! The home chess table grew from 1 × 1 to 2 × 1 (a chair at each end, M-hobbies-11; save
//! version 33). A version-32 save's table stood on one tile, hard against a wall or another
//! thing: loaded, it goes over the tile it stood on where it now fits and blocks nothing, else at
//! a free spot at home, else it is sold back to the household. Never overlapping anything.

mod common;

use serde_json::{Value, json};
use sim_core::World;

/// Floor tiles each object stands on (its corner and footprint, turned).
fn tiles(w: &World, i: usize) -> Vec<(i32, i32)> {
    let o = &w.objects[i];
    let [fw, fd] = o.footprint;
    let (tw, td) = if o.rot.is_multiple_of(2) { (fw, fd) } else { (fd, fw) };
    (0..td as i32).flat_map(|dz| (0..tw as i32).map(move |dx| (o.x + dx, o.z + dz))).collect()
}

/// No two things on the floor share a tile (the wall and ceiling layers and things on tops aside).
fn no_overlap(w: &World) {
    let floor: Vec<usize> = (0..w.objects.len())
        .filter(|&i| matches!(w.content.objects[w.objects[i].def].layer, sim_core::content::Layer::Floor))
        .collect();
    for (k, &a) in floor.iter().enumerate() {
        for &b in &floor[k + 1..] {
            let shared: Vec<_> = tiles(w, a).into_iter().filter(|t| tiles(w, b).contains(t)).collect();
            assert!(shared.is_empty(), "objects {a} and {b} share {shared:?}");
        }
    }
}

/// `w` saved as version 32 with a chess table of the old size added at `x`, `z` facing `rot`,
/// worth `value`, then loaded.
fn load_v32_with_table(w: &World, content: &str, x: i32, z: i32, rot: u8, value: i64) -> World {
    load_v32(w, content, x, z, rot, value, false)
}

/// As `load_v32_with_table`; `first`: the table first in the file, before what stands beside it.
fn load_v32(w: &World, content: &str, x: i32, z: i32, rot: u8, value: i64, first: bool) -> World {
    let mut saved: Value = serde_json::from_str(&w.save_json()).unwrap();
    saved["version"] = 32.into();
    let next = saved["nextObjectId"].as_u64().unwrap_or(100);
    let table = json!({"id": next, "def": "chessTable", "x": x, "z": z, "rot": rot, "value": value});
    let objects = saved["objects"].as_array_mut().unwrap();
    if first {
        objects.insert(0, table);
    } else {
        objects.push(table);
    }
    saved["nextObjectId"] = (next + 1).into();
    World::from_save_json(content, &saved.to_string()).unwrap()
}

fn chess_tables(w: &World) -> Vec<usize> {
    let def = w.content.object_index("chessTable").unwrap();
    (0..w.objects.len()).filter(|&i| w.objects[i].def == def).collect()
}

#[test]
fn an_old_chess_table_against_another_thing_moves_over_to_fit() {
    let (mut w, content) = common::garden_home();
    // Inside the house (walls from x 3 to 19, z 3 to 11), a bookshelf on the tile to the table's right.
    common::buy(&mut w, "bookshelf", 6, 3);
    let funds = w.households[0].funds;
    let back = load_v32_with_table(&w, &content, 5, 3, 0, 300);
    let tables = chess_tables(&back);
    assert_eq!(tables.len(), 1, "the table is kept");
    let t = tables[0];
    assert!(tiles(&back, t).contains(&(5, 3)), "still over the tile it stood on: {:?}", tiles(&back, t));
    assert_eq!(back.objects.len(), w.objects.len() + 1, "nothing else is left out");
    assert_eq!(back.households[0].funds, funds, "nothing sold");
    no_overlap(&back);
    common::check(&back);
    // Saved again (v33), it stays where it went.
    let again = World::from_save_json(&content, &back.save_json()).unwrap();
    let t2 = chess_tables(&again)[0];
    assert_eq!((again.objects[t2].x, again.objects[t2].z, again.objects[t2].rot), (back.objects[t].x, back.objects[t].z, back.objects[t].rot));
    // First in the file, it still goes in after the bookshelf, which keeps its place.
    let first = load_v32(&w, &content, 5, 3, 0, 300, true);
    assert_eq!(first.objects.len(), w.objects.len() + 1, "nothing else is left out");
    let shelf = first.content.object_index("bookshelf").unwrap();
    assert!(first.objects.iter().any(|o| o.def == shelf && (o.x, o.z) == (6, 3)), "the bookshelf where it was");
    no_overlap(&first);
}

#[test]
fn an_old_chess_table_against_a_wall_moves_in_from_it() {
    let (w, content) = common::garden_home();
    // The house's last column inside its right-hand wall (x 18; the wall runs along x = 19).
    let back = load_v32_with_table(&w, &content, 18, 6, 0, 300);
    let t = chess_tables(&back)[0];
    assert!(tiles(&back, t).contains(&(18, 6)), "still over the tile it stood on: {:?}", tiles(&back, t));
    assert!(tiles(&back, t).iter().all(|&(x, _)| x < 19), "inside the wall: {:?}", tiles(&back, t));
    no_overlap(&back);
    common::check(&back);
}

#[test]
fn an_old_chess_table_with_no_room_for_it_is_sold_back() {
    let content = common::content();
    // A lot one tile wide: nothing 2 × 1 stands on it facing a tile on the lot.
    let lot = json!({
        "width": 1, "depth": 4,
        "walls": [], "doors": [], "objects": [],
        "plots": [{"name": "Home", "x": 0, "z": 0, "w": 1, "d": 4, "entry": [0.5, 3.5]}],
        "households": [{"name": "The Narrows", "plot": 0, "player": true, "funds": 1000}],
        "sims": [{"name": "Ada", "gender": "female", "age": 40.0, "traits": ["cheerful"], "x": 0.5, "z": 3.5}],
    });
    let w = World::from_json(&content, &lot.to_string(), 5).unwrap();
    let back = load_v32_with_table(&w, &content, 0, 0, 0, 300);
    assert!(chess_tables(&back).is_empty(), "no room: left out");
    assert_eq!(back.households[0].funds, 1300, "sold back for what it was worth");
}

#[test]
fn a_chess_table_from_this_version_loads_where_it_was() {
    let (mut w, content) = common::garden_home();
    common::buy(&mut w, "chessTable", 8, 6);
    let back = World::from_save_json(&content, &w.save_json()).unwrap();
    let t = chess_tables(&back)[0];
    assert_eq!((back.objects[t].x, back.objects[t].z, back.objects[t].footprint), (8, 6, [2, 1]));
}
