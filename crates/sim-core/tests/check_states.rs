//! States for checking things in the game, so nobody has to make them by hand: with
//! `DUMP_STATES=1 cargo test -p sim-core --test check_states` each test writes one to
//! `debug-states/<name>.json` (gitignored), opened with `?state=<name>&debug` on the dev server.
//! Each test also checks that its state is still what the checks need. (`tools/bench/README.md`
//! lists them and how to use them.)
//!
//! - `bathroom-check`: the house of the town `wants.rs` makes (seed 21), with one resident (0) at
//!   home at 10:00 on day 1, doing nothing, with no job, free will off and every need at 0.8:
//!   nothing pulls them away from a `use`, and a use that fills a need still lasts a while (a full
//!   need ends it at once). Plenty of money. Free wall to buy against in the living room at
//!   `[23, 14, rot]` and `[24, 17, rot]`. (The game's own Free will setting, on by default, is
//!   applied to the player's household as the page opens: turn it off there, or send
//!   `setAutonomy`; `tools/bench/check.mjs` does.)
//! - `needy-check`: the same, with free will on and low needs (bladder 0.05, hunger 0.2, energy
//!   0.25, hygiene 0.3, the rest 0.5): they go to the toilet, then eat, wash and sleep on their own.
//! - `office-night`: `bathroom-check` at 21:30, after dark (lamps, screens, night lighting).
//! - `town-things`: a public lot ("Town things", plot 1) with every community thing
//!   (`community.json`: street, play, food, farm and shops, in rows by group facing the
//!   street, the shops against a wall at the back), beside a home plot with a small house. The
//!   resident stands at the lot's front, free will off, needs at 0.8. Look with `__showPlot(1)`.

mod common;

use serde_json::{Value, json};
use sim_core::{Command, MINUTES_PER_TICK, World, clock};

const SEED: u64 = 21;
/// Money for buying everything a check needs.
const FUNDS: i64 = 1_000_000;

fn until(w: &mut World, day: u32, hour: f32) {
    let target = clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < target {
        w.tick_once();
    }
}

/// Resident 0 stops what they're doing and drops their queue (and puts down what they carry):
/// a `use` sent from the page starts at once.
fn free_up(w: &mut World) {
    for _ in 0..30 {
        if w.sims[0].current().is_none() && w.sims[0].queue().next().is_none() && w.sims[0].held.is_none() {
            return;
        }
        while w.sims[0].current().is_some() || w.sims[0].queue().next().is_some() {
            w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
        }
        for _ in 0..(2.0 / MINUTES_PER_TICK) as u32 {
            w.tick_once();
        }
    }
    panic!("resident 0 doesn't come free: {:?}, holding {:?}", w.sims[0].current().map(|a| &a.task), w.sims[0].held);
}

fn need(w: &World, id: &str) -> usize {
    w.content.needs.iter().position(|n| n.id == id).unwrap_or_else(|| panic!("a need '{id}'"))
}

/// Sets resident `i`'s needs: `id` → value, every other need to `rest`.
fn set_needs(w: &mut World, i: usize, rest: f32, low: &[(&str, f32)]) {
    for n in 0..w.content.needs.len() {
        w.sims[i].needs[n] = rest;
    }
    for &(id, v) in low {
        let n = need(w, id);
        w.sims[i].needs[n] = v;
    }
}

/// The town of `wants.rs` with its newcomer (resident 0) home and free (no job, no job hunting),
/// with free will as given, at `hour` on day 1 (a game starts at 8:00).
fn house(hour: f32, free_will: bool) -> World {
    let content = common::content();
    let file = common::town(&content, SEED, 1);
    let mut w = World::from_json(&content, &file, SEED as u32).unwrap();
    w.sims[0].job = None;
    w.sims[0].job_search_from = u32::MAX;
    let h = w.sims[0].household as usize;
    w.households[h].free_will = false;
    w.households[h].funds = FUNDS;
    until(&mut w, 1, hour);
    free_up(&mut w);
    w.households[h].free_will = free_will;
    w
}

/// Resident 0 is home, not away, on the player's plot.
fn at_home(w: &World) {
    let h = w.sims[0].household as usize;
    assert!(w.households[h].player, "resident 0 is the player's");
    let plot = w.households[h].plot.expect("a home");
    let p = &w.plots[plot as usize];
    let [x, z] = w.sims[0].pos;
    let inside = x >= p.x as f32 && x <= (p.x + p.w) as f32 && z >= p.z as f32 && z <= (p.z + p.d) as f32;
    assert!(inside && w.sims[0].away_until.is_none(), "at home: {:?}", w.sims[0].pos);
}

#[test]
fn bathroom_check() {
    let mut w = house(10.0, false);
    set_needs(&mut w, 0, 0.8, &[]);
    at_home(&w);
    common::dump_state(&w, "bathroom-check");
    // An hour on, still nothing pulls them away.
    let day = clock::day(w.tick);
    until(&mut w, day, 11.0);
    assert!(w.sims[0].needs.iter().take(w.content.needs.len()).all(|&n| n > w.content.autonomy.urgent_below), "{:?}", w.sims[0].needs);
    // `?debug`'s `__addFunds`: money in, and out down to nothing.
    let h = w.sims[0].household;
    let before = w.households[h as usize].funds;
    w.apply(Command::AddFunds { household: h, amount: 500 }).unwrap();
    assert_eq!(w.households[h as usize].funds, before + 500);
    w.apply(Command::AddFunds { household: h, amount: -10 * FUNDS }).unwrap();
    assert_eq!(w.households[h as usize].funds, 0);
    assert!(w.apply(Command::AddFunds { household: 99, amount: 1 }).is_err());
}

#[test]
fn needy_check() {
    let mut w = house(10.0, true);
    set_needs(&mut w, 0, 0.5, &[("bladder", 0.05), ("hunger", 0.2), ("energy", 0.25), ("hygiene", 0.3)]);
    at_home(&w);
    common::dump_state(&w, "needy-check");
}

#[test]
fn office_night() {
    let mut w = house(21.5, false);
    set_needs(&mut w, 0, 0.8, &[]);
    at_home(&w);
    let hour = clock::hour(w.tick);
    assert!(hour >= 21.0, "after dark: {hour}");
    common::dump_state(&w, "office-night");
}

/// Every community thing in rows by group (each `(def, width, depth)` as `community.json` has it).
fn town_things(content: &Value) -> Vec<(String, String, i64, i64)> {
    let objects = content["objects"].as_array().unwrap();
    let community: Value = serde_json::from_str(&std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/content/community.json"),
    ).unwrap())
    .unwrap();
    community["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            let id = o["id"].as_str().unwrap();
            // As merged (a pack or later file may change it).
            let def = objects.iter().find(|d| d["id"] == id).unwrap_or(o);
            let f = def["footprint"].as_array().map_or((1, 1), |f| (f[0].as_i64().unwrap(), f[1].as_i64().unwrap()));
            (id.to_string(), def["group"].as_str().unwrap_or("").to_string(), f.0, f.1)
        })
        .collect()
}

#[test]
fn town_things_lot() {
    let merged = common::content();
    let content: Value = serde_json::from_str(&merged).unwrap();
    let things = town_things(&content);
    assert!(things.len() >= 36, "every community thing: {}", things.len());
    // Home plot (empty) on the left, the town lot on the right; the street along z = 0.
    let (hw, w, d) = (12, 54, 32);
    // Rows by group, front to back; each row starts a tile in, things two tiles apart. The
    // shops' row stands at the back against a wall, as in a shop.
    let rows = [("street", 3), ("play", 8), ("food", 13), ("farm", 18), ("shops", d - 3)];
    let mut objects = Vec::new();
    for (group, z0) in rows {
        let mut x = hw + 2;
        for (id, _, fw, fd) in things.iter().filter(|t| t.1 == group) {
            // Facing the street (rot 2); a row's things line up at its back.
            let z = if group == "shops" { z0 + 1 - fd } else { z0 };
            objects.push(json!({"def": id, "x": x, "z": z, "rot": 2}));
            x += fw + 2;
        }
        assert!(x <= hw + w, "the {group} row fits: {x}");
    }
    let placed = objects.len();
    assert_eq!(placed, things.len(), "every thing in a row by its group: {:?}", things.iter().map(|t| &t.1).collect::<Vec<_>>());
    let lot = json!({
        "width": hw + w, "depth": d,
        // A small house at home (an empty lot opens in Build mode); the shops' wall on the town lot.
        "walls": [[2, 4, 8, 4], [8, 4, 8, 10], [8, 10, 2, 10], [2, 10, 2, 4], [hw + 1, d - 2, hw + 21, d - 2]],
        "doors": [{"x": 4, "z": 4, "axis": "x"}],
        "objects": objects,
        "plots": [{"name": "Home", "x": 0, "z": 0, "w": hw, "d": d, "entry": [hw as f64 / 2.0, 1.5]},
                  {"name": "Town things", "x": hw, "z": 0, "w": w, "d": d, "public": true, "entry": [hw as f64 + w as f64 / 2.0, 1.5]}],
        "households": [{"name": "Ada's", "plot": 0, "player": true, "funds": FUNDS}],
        "sims": [{"name": "Ada", "traits": ["cheerful"], "x": hw as f64 + 3.5, "z": 1.5}],
    });
    let mut world = World::from_json(&merged, &lot.to_string(), 7).unwrap();
    assert_eq!(world.objects.len(), placed, "every thing loads where it was put");
    world.households[0].free_will = false;
    until(&mut world, 1, 10.0);
    free_up(&mut world);
    set_needs(&mut world, 0, 0.8, &[]);
    common::dump_state(&world, "town-things");
}
