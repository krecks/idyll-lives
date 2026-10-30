//! A state for the play checks a bench state can't make: a family home with children and a baby.
//! With `DUMP_STATES=1 cargo test -p sim-core --test sim play_checks::` it writes
//! `debug-states/family-check.json`, opened with `?state=family-check&debug`.
//!
//! - `family-check`: one plot, a house of three rooms and a back garden, 16:30 on day 1 (school's
//!   out). Ada (0) and Ben (1) are the parents, Cal (2), Dee (3) and Fay (4) children, Eve (5) a
//!   baby lying in the crib. Free will off, needs at 0.8, the children's energy and comfort low
//!   (a sleep lasts). $1,000,000.
//!   - Kitchen (x 2–14, z 4–12): a run on each of its four walls, each with a fridge, a counter, a
//!     cooker and a drawer counter, turned to each facing; the first (against z = 4, facing +z) is
//!     fridge 3, counter 4, gas stove 5, sink 6, drawer counter 7, pedal bin 8. The long island at
//!     (6, 7) facing +z, a counter stool behind it at (7, 6).
//!   - Bedroom (x 14–32, z 4–12): bunk (16, 4), car bed (19, 4), crib (22, 4), hamper (26, 4),
//!     all against the z = 4 wall facing +z; exercise bike at (24, 8) facing +z.
//!   - Living room (x 2–32, z 12–22): a dining table for four at (4, 16); the kitchen island turned
//!     to each facing at (10, 15) 0, (14, 15) 2, (18, 14) 1, (22, 14) 3; a second long island at
//!     (10, 19) facing -z.
//!   - Garden (z 22–34): garden chair (4, 22), Adirondack chair (6, 22), outdoor bar (9, 22), all
//!     backs on the house wall facing +z; the hammock at (20, 26).

use crate::common;

use serde_json::json;
use sim_core::{Command, MINUTES_PER_TICK, World, clock};

const FUNDS: i64 = 1_000_000;

fn until(w: &mut World, day: u32, hour: f32) {
    let target = clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < target {
        w.tick_once();
    }
}

/// Resident `i` stops what they're doing and drops their queue.
fn free_up(w: &mut World, i: usize) {
    for _ in 0..30 {
        if w.sims[i].current().is_none()
            && w.sims[i].queue().next().is_none()
            && w.sims[i].held.is_none()
        {
            return;
        }
        while w.sims[i].current().is_some() || w.sims[i].queue().next().is_some() {
            w.apply(Command::Cancel {
                sim: i as u32,
                index: 0,
            })
            .unwrap();
        }
        for _ in 0..(2.0 / MINUTES_PER_TICK) as u32 {
            w.tick_once();
        }
    }
    panic!("resident {i} doesn't come free");
}

fn set_need(w: &mut World, i: usize, id: &str, v: f32) {
    let n = w.content.needs.iter().position(|n| n.id == id).unwrap();
    w.sims[i].needs[n] = v;
}

#[test]
fn family_check() {
    let merged = common::content();
    let o = |def: &str, x: i32, z: i32, rot: u8| json!({"def": def, "x": x, "z": z, "rot": rot});
    let objects = vec![
        // Kitchen, run A against z = 4 (facing +z).
        o("fridge", 3, 4, 0),
        o("counter", 4, 4, 0),
        o("stoveGas", 5, 4, 0),
        o("farmhouseSink", 6, 4, 0),
        o("counterDrawers", 7, 4, 0),
        o("trashCan", 8, 4, 0),
        // Run B against z = 12 (facing -z).
        o("fridgeFrenchDoor", 3, 11, 2),
        o("counterMarble", 4, 11, 2),
        o("stoveElectric", 5, 11, 2),
        o("counterDrawers", 6, 11, 2),
        // Run C against x = 2 (facing +x).
        o("fridgeRetro", 2, 6, 1),
        o("counter", 2, 7, 1),
        o("stoveGas", 2, 8, 1),
        o("counterDrawers", 2, 9, 1),
        // Run D against x = 14 (facing -x).
        o("fridgeBuiltIn", 13, 6, 3),
        o("counterOpenShelf", 13, 7, 3),
        o("cooktopCounter", 13, 8, 3),
        o("counterDrawers", 13, 9, 3),
        // The long island, a stool behind it.
        o("islandLong", 6, 7, 0),
        o("stoolCounter", 7, 6, 0),
        // Bedroom.
        o("bedBunk", 16, 4, 0),
        o("kidsBedCar", 19, 4, 0),
        o("crib", 22, 4, 0),
        o("laundryHamper", 26, 4, 0),
        o("exerciseBike", 24, 8, 0),
        // Living room: a table for four, the island four ways, a second long island.
        o("diningTable", 4, 16, 0),
        o("diningChair", 4, 15, 0),
        o("diningChair", 5, 15, 0),
        o("diningChair", 4, 17, 2),
        o("diningChair", 5, 17, 2),
        o("kitchenIsland", 10, 15, 0),
        o("kitchenIsland", 14, 15, 2),
        o("kitchenIsland", 18, 14, 1),
        o("kitchenIsland", 22, 14, 3),
        o("islandLong", 10, 19, 2),
        // Garden.
        o("gardenChair", 4, 22, 0),
        o("adirondackChair", 6, 22, 0),
        o("outdoorBar", 9, 22, 0),
        o("hammock", 20, 26, 0),
    ];
    let placed = objects.len();
    let sim = |name: &str, gender: &str, age: f32, x: f64, z: f64| json!({"name": name, "gender": gender, "age": age, "traits": ["cheerful"], "x": x, "z": z});
    let lot = json!({
        "width": 34, "depth": 34,
        "walls": [[2, 4, 32, 4], [32, 4, 32, 22], [32, 22, 2, 22], [2, 22, 2, 4], [2, 12, 32, 12], [14, 4, 14, 12]],
        "doors": [{"x": 28, "z": 4, "axis": "x"}, {"x": 12, "z": 12, "axis": "x"}, {"x": 20, "z": 12, "axis": "x"},
                  {"x": 16, "z": 22, "axis": "x"}],
        "objects": objects,
        "plots": [{"name": "Home", "x": 0, "z": 0, "w": 34, "d": 34, "entry": [17.0, 1.5]}],
        "households": [{"name": "The Checks", "plot": 0, "player": true, "funds": FUNDS}],
        "sims": [sim("Ada", "female", 34.0, 8.5, 14.5), sim("Ben", "male", 36.0, 9.5, 14.5), sim("Cal", "male", 9.0, 18.5, 9.5),
                 sim("Dee", "female", 11.0, 19.5, 9.5), sim("Fay", "female", 7.0, 20.5, 9.5), sim("Eve", "female", 0.5, 22.5, 6.5)],
        "relationships": [
            {"a": 0, "b": 1, "preset": "partners"},
            {"a": 2, "b": 0, "preset": "parent"}, {"a": 2, "b": 1, "preset": "parent"},
            {"a": 3, "b": 0, "preset": "parent"}, {"a": 3, "b": 1, "preset": "parent"},
            {"a": 4, "b": 0, "preset": "parent"}, {"a": 4, "b": 1, "preset": "parent"},
            {"a": 5, "b": 0, "preset": "parent"}, {"a": 5, "b": 1, "preset": "parent"},
            {"a": 2, "b": 3, "preset": "siblings"}, {"a": 2, "b": 4, "preset": "siblings"}, {"a": 3, "b": 4, "preset": "siblings"},
        ],
    });
    let mut w = World::from_json(&merged, &lot.to_string(), 7).unwrap();
    assert_eq!(
        w.objects.len(),
        placed,
        "every thing loads where it was put"
    );
    for s in w.sims.iter_mut() {
        s.job = None;
        s.job_search_from = u32::MAX;
    }
    w.households[0].free_will = false;
    until(&mut w, 1, 16.5);
    for i in 0..5 {
        free_up(&mut w, i);
    }
    for i in 0..w.sims.len() {
        for n in 0..w.content.needs.len() {
            w.sims[i].needs[n] = 0.8;
        }
    }
    for i in 2..5 {
        set_need(&mut w, i, "energy", 0.2);
        set_need(&mut w, i, "comfort", 0.3);
    }
    // Everyone home (the children back from school), the baby in the crib.
    for (i, s) in w.sims.iter().enumerate() {
        let [x, z] = s.pos;
        assert!(
            x < 34.0 && z < 34.0 && s.away_until.is_none(),
            "resident {i} at home: {:?}",
            s.pos
        );
    }
    assert!(w.content.life.baby(w.sims[5].age));
    assert_eq!(
        w.sims[5].pose,
        sim_core::content::Pose::Lie,
        "Eve is in the crib"
    );
    common::dump_state(&w, "family-check");
}

/// Generated houses to look round in the game: with `HOUSES` (a file made by
/// `tools/housegen/sample.ts`) and `PICK` (indices into it, e.g. `80,140,160`), writes each as
/// `debug-states/generated-<index>.json` with a resident at the door, free will off.
#[test]
#[ignore]
fn generated_houses_to_look_at() {
    let (Ok(path), Ok(pick)) = (std::env::var("HOUSES"), std::env::var("PICK")) else {
        eprintln!("set HOUSES (from tools/housegen/sample.ts) and PICK (indices)");
        return;
    };
    let houses: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let content = common::content();
    for i in pick.split(',').map(|s| s.trim().parse::<usize>().unwrap()) {
        let mut town = houses[i]["town"].clone();
        let entry = town["plots"][0]["entry"].clone();
        town["sims"] =
            json!([{ "name": "Ada", "x": entry[0], "z": entry[1], "traits": ["foodie"] }]);
        town["households"][0]["funds"] = json!(FUNDS);
        let mut w = World::from_json(&content, &town.to_string(), 1).unwrap();
        w.households[0].free_will = false;
        common::dump_state(&w, &format!("generated-{i}"));
    }
}
