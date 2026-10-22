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
//! - `review-stage`: an empty home lot with one walled room of 12 × 12 tiles (a door, two
//!   windows, a crib and a counter, the rest free floor and wall) and a garden behind it, for
//!   buying one thing after another to look at in use (`check.mjs play --objects`). One resident
//!   per life stage, free will off, needs at 0.5, on a Saturday morning (the children home).
//!   Its free spots are at `STAGE_ROOM` below and in `tools/bench/README.md`. A small kitchen
//!   along the right wall (the counter, a fridge and a coffee machine) for cooking and coffee.
//! - `review-stage-night`: the same stage at 22:00, the clock set rather than run; the lamps as
//!   the game leaves them (with free will off nobody switches one; a lamp bought comes on).
//! - `review-garden`: the same stage with every crop kind (content `grows`) in a row at the back
//!   of the garden, ripe (a crop comes ripe with a lot), for their "when ripe" uses
//!   (`check.mjs play --use-existing <def>`).
//! - `review-baby`: the same stage with a dining table (`tableDining4`) and the high chair pulled up to it (at
//!   `BABY_TABLE`), for the baby's care (G-66): Ada (0) feeding Eve (5) from the crib carries her
//!   to the high chair and feeds her there, and changes her at the crib's changing table
//!   (`check.mjs play --use-existing crib --use feed`, `--use change`). Eve's hunger and bladder low.
//! - `crowded-garden`: `common::garden_home()` (two parents and two children in the front yard,
//!   Saturday noon, free will off, needs at 0.5) with its back garden from (5, 14) to (15, 22)
//!   packed with garden things at random (`garden_cluster`, `Packed(31)`: what buying allowed).
//!   The first sun lounger, at (10, 16) facing −z, has both its sides shut in by the beds round
//!   it and only its foot free (I-garden-01): `check.mjs play --use-existing sunLounger`.
//! - `review-resized`: `garden_home()` with things that have moving parts bought resized, each
//!   against a wall of the room (`RESIZED`: a fridge two tiles wide and 30 cm taller, a toilet,
//!   a filing cabinet, a washing machine, a wall bed three by three): their clips with the
//!   resize (A-systems-01; resizing is off in Buy mode, `RESIZING` in `buildmode.ts`).
//!   `review-resized-base`: the same things at their own size.

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

/// `review-stage`'s room: walls from `(x0, z0)` to `(x1, z1)`, a door in the front wall (low z,
/// toward the street) and two windows in the left one. Free spots for a thing of footprint
/// `[w, d]` (as `tools/bench/check.mjs` places them by default and the README lists): the floor in
/// the middle `(10 - w/2, 12 - d/2, 0)`, the back wall inside `(10 - w/2, 18 - d, 2)`, the left
/// wall `(4, 8..=12, 1)` and its windows `(4, 14..=15, 1)`, the right wall `(16 - d, 10..=16, 3)`,
/// the front wall `(7..=12, 6, 0)`, the back wall outside `(4..=15, 18, 0)`, and the garden
/// `(10 - w/2, 22, 0)` (free from z = 19 to 31, and from x = 18 to 31 beside the room; on
/// `review-garden` to z = 27, its crops at `GARDEN_ROW`). The kitchen stands at `(15, 7..=9, 3)`,
/// out of them all.
const STAGE_ROOM: (i32, i32, i32, i32) = (4, 6, 16, 18);
/// `review-baby`'s dining table (2 x 1, facing the front), the high chair at its front facing it.
const BABY_TABLE: (i32, i32) = (8, 12);
/// `review-garden`'s row of crops, facing the house, one every `GARDEN_STEP` tiles from x = 2.
const GARDEN_ROW: i32 = 29;
const GARDEN_STEP: i32 = 3;

/// `review-stage` with `more` objects, on Saturday at `hour` (see the module's docs).
fn stage(more: &[Value], hour: f32) -> World {
    let merged = common::content();
    let (x0, z0, x1, z1) = STAGE_ROOM;
    let (w, d) = (32, 32);
    // One resident per life stage (`content.life`): an adult first, the one checks use by
    // default; the baby by the crib, the rest in the front yard, out of the room.
    let sim = |name: &str, gender: &str, age: f32, x: f64, z: f64| json!({"name": name, "gender": gender, "age": age, "traits": ["cheerful"], "x": x, "z": z});
    // A crib for the baby; a counter for things that stand on a top, and beside it along the
    // right wall a fridge and a coffee machine (cooking at a stove bought, coffee at a table),
    // all facing -x.
    let mut objects = vec![
        json!({"def": "crib", "x": 13, "z": z0, "rot": 0}),
        json!({"def": "counter", "x": x1 - 1, "z": 7, "rot": 3}),
        json!({"def": "fridge", "x": x1 - 1, "z": 8, "rot": 3}),
        json!({"def": "coffeeMachine", "x": x1 - 1, "z": 9, "rot": 3}),
    ];
    let fixed = objects.len();
    objects.extend(more.iter().cloned());
    let lot = json!({
        "width": w, "depth": d,
        "walls": [[x0, z0, x1, z0], [x1, z0, x1, z1], [x1, z1, x0, z1], [x0, z1, x0, z0]],
        "doors": [{"x": 5, "z": z0, "axis": "x"}],
        "windows": [{"x": x0, "z": 14, "axis": "z"}, {"x": x0, "z": 15, "axis": "z"}],
        "objects": objects,
        "plots": [{"name": "Home", "x": 0, "z": 0, "w": w, "d": d, "entry": [w as f64 / 2.0, 1.5]}],
        "households": [{"name": "The Stages", "plot": 0, "player": true, "funds": FUNDS}],
        "sims": [sim("Ada", "female", 40.0, 20.5, 3.5), sim("Ben", "male", 22.0, 21.5, 3.5), sim("Cal", "male", 15.0, 22.5, 3.5),
                 sim("Dee", "female", 8.0, 23.5, 3.5), sim("Gus", "male", 70.0, 24.5, 3.5), sim("Eve", "female", 0.5, 13.5, 7.5)],
        "relationships": [
            {"a": 1, "b": 0, "preset": "parent"}, {"a": 2, "b": 0, "preset": "parent"}, {"a": 3, "b": 0, "preset": "parent"},
            {"a": 5, "b": 0, "preset": "parent"}, {"a": 0, "b": 4, "preset": "parent"},
        ],
    });
    let mut world = World::from_json(&merged, &lot.to_string(), 7).unwrap();
    assert_eq!(world.objects.len(), fixed + more.len(), "the crib, the kitchen and the rest load where they were put");
    for s in world.sims.iter_mut() {
        s.job = None;
        s.job_search_from = u32::MAX;
    }
    world.households[0].free_will = false;
    // Saturday (day 6), so the children aren't at school: the clock is set rather than run, as
    // five days would age everyone two and a half years (two days a year).
    world.tick = clock::tick_at(6, hour * 60.0 - 5.0).unwrap();
    assert_eq!(clock::weekday(clock::day(world.tick)), 5);
    for _ in 0..(5.0 / MINUTES_PER_TICK) as u32 {
        world.tick_once();
    }
    free_up(&mut world);
    for i in 0..world.sims.len() {
        // Half full: a use that fills one lasts long enough to watch (at 0.8 a toilet's is
        // over in a second), and a long run of them still doesn't wear anyone out.
        set_needs(&mut world, i, 0.5, &[]);
    }
    // Every life stage, each at home.
    let life = &world.content.life;
    let stages: Vec<&str> = world.sims.iter().map(|s| life.stage_at(s.age).unwrap().id.as_str()).collect();
    assert_eq!(stages, ["adult", "youngAdult", "teen", "child", "elder", "baby"]);
    assert_eq!(life.stages.len(), stages.len(), "one resident per stage");
    for (i, s) in world.sims.iter().enumerate() {
        assert!(s.away_until.is_none(), "resident {i} at home");
    }
    assert_eq!(world.sims[5].pose, sim_core::content::Pose::Lie, "Eve is in the crib");
    // The room is closed: one room of 144 tiles with its door and windows, the garden round it.
    let room = world.room_at_tile(10, 12).expect("a room");
    assert!(!room.garden && room.tiles == 144.0 && room.doors == 1 && room.windows == 2, "{:?}", (room.garden, room.tiles, room.doors, room.windows));
    assert!(world.room_at_tile(10, 22).is_some_and(|r| r.garden), "the garden behind");
    world
}

/// Resident `sim` is given use `id` of the stage's first `def` (the command is taken and the
/// use queued), and lets it go again.
fn takes(w: &mut World, sim: u32, def: &str, id: &str) {
    let o = w.objects.iter().find(|o| w.content.objects[o.def].id == def).unwrap_or_else(|| panic!("a {def}"));
    let interaction = w.content.objects[o.def].interactions.iter().position(|i| i.id == id).unwrap_or_else(|| panic!("{def} offers '{id}'"));
    let object = o.stable;
    w.apply(Command::Use { sim, object, interaction }).unwrap_or_else(|e| panic!("{def} / {id}: {e}"));
    let s = &w.sims[sim as usize];
    assert!(s.current().is_some() || s.queue().next().is_some(), "{def} / {id} queued");
    w.apply(Command::Cancel { sim, index: 0 }).unwrap();
}

#[test]
fn review_stage() {
    let mut world = stage(&[], 10.0);
    common::dump_state(&world, "review-stage");
    // The kitchen works without anything bought: a coffee from the machine, a meal from the fridge.
    takes(&mut world, 0, "coffeeMachine", "espresso");
    takes(&mut world, 0, "fridge", "meal");
    // `check.mjs play --skills`: a skill set for a check (`Command::SetSkill`), within its range.
    let cooking = world.content.skill_index("cooking").unwrap();
    let max = world.content.skill_rules.max_level;
    world.apply(Command::SetSkill { sim: 0, skill: "cooking".into(), level: 5.0 }).unwrap();
    assert_eq!(world.sims[0].skills[cooking], 5.0);
    world.apply(Command::SetSkill { sim: 0, skill: "cooking".into(), level: max + 3.0 }).unwrap();
    assert_eq!(world.sims[0].skills[cooking], max);
    assert!(world.apply(Command::SetSkill { sim: 0, skill: "juggling".into(), level: 1.0 }).is_err());
    assert!(world.apply(Command::SetSkill { sim: 99, skill: "cooking".into(), level: 1.0 }).is_err());
}

#[test]
fn review_stage_night() {
    let world = stage(&[], 22.0);
    let hour = clock::hour(world.tick);
    assert!(sim_core::lights::dark_at(hour * 60.0) && hour >= 22.0, "after dark: {hour}");
    common::dump_state(&world, "review-stage-night");
}

#[test]
fn review_baby() {
    let (tx, tz) = BABY_TABLE;
    let more = [json!({"def": "tableDining4", "x": tx, "z": tz, "rot": 0}), json!({"def": "highChair", "x": tx, "z": tz + 1, "rot": 2})];
    let mut world = stage(&more, 10.0);
    let hunger = world.content.needs.iter().position(|n| n.id == "hunger").unwrap();
    let bladder = world.content.needs.iter().position(|n| n.id == "bladder").unwrap();
    world.sims[5].needs[hunger] = 0.25;
    world.sims[5].needs[bladder] = 0.25;
    common::dump_state(&world, "review-baby");
    // Ada's feed from the crib goes to the high chair, by day.
    let crib = world.objects.iter().find(|o| world.content.objects[o.def].id == "crib").unwrap().stable;
    let feed = world.content.objects[world.objects[0].def].interactions.iter().position(|i| i.id == "feed").unwrap();
    world.apply(Command::Use { sim: 0, object: crib, interaction: feed }).unwrap();
    let mut seated = false;
    for _ in 0..(30.0 / MINUTES_PER_TICK) as u32 {
        world.tick_once();
        seated |= world.objects.iter().any(|o| world.content.objects[o.def].id == "highChair" && o.users[0] == Some(5));
    }
    assert!(seated, "Eve was carried to the high chair");
}

#[test]
fn review_garden() {
    let merged: Value = serde_json::from_str(&common::content()).unwrap();
    // Every kind of crop, in a row at the back of the garden facing the house.
    let mut crops = Vec::new();
    let mut x = 2;
    for o in merged["objects"].as_array().unwrap().iter().filter(|o| o.get("grows").is_some()) {
        let fw = o["footprint"].as_array().map_or(1, |f| f[0].as_i64().unwrap() as i32);
        assert!(fw < GARDEN_STEP, "{} fits its place in the row", o["id"]);
        crops.push(json!({"def": o["id"], "x": x, "z": GARDEN_ROW, "rot": 2}));
        x += GARDEN_STEP;
    }
    assert!(crops.len() >= 6 && x <= 32, "every crop in one row: {}", crops.len());
    let mut world = stage(&crops, 10.0);
    let ripe: Vec<(String, String)> = world
        .objects
        .iter()
        .filter(|o| world.content.objects[o.def].grows.is_some())
        .map(|o| {
            let d = &world.content.objects[o.def];
            assert!(o.ripe(), "{} ripe", d.id);
            (d.id.clone(), d.interactions.iter().find(|i| i.ripe).unwrap_or_else(|| panic!("{} has a 'when ripe' use", d.id)).id.clone())
        })
        .collect();
    assert_eq!(ripe.len(), crops.len());
    common::dump_state(&world, "review-garden");
    // Each one's "when ripe" use is offered.
    for (def, id) in &ripe {
        takes(&mut world, 0, def, id);
    }
}

#[test]
fn crowded_garden() {
    let (mut world, _) = common::garden_home();
    common::garden_cluster(&mut world, 0, common::GARDEN_PATCH, common::Cluster::Packed(31), None);
    for i in 0..world.sims.len() {
        set_needs(&mut world, i, 0.5, &[]);
    }
    // The lounger whose sides are shut in by the beds round it, its foot free.
    let lounger = world.objects.iter().find(|o| world.content.objects[o.def].id == "sunLounger").expect("a lounger in the crowd");
    assert_eq!((lounger.x, lounger.z, lounger.rot), (10, 16, 2), "the first lounger is the shut-in one");
    let (object, at) = (lounger.stable, lounger.id);
    common::dump_state(&world, "crowded-garden");
    // Ada gets onto it all the same, from its foot.
    let sunbathe = world.content.objects[world.objects[at as usize].def].interactions.iter().position(|i| i.id == "sunbathe").unwrap();
    world.apply(Command::Use { sim: 0, object, interaction: sunbathe }).unwrap();
    let on = (0..(30.0 / MINUTES_PER_TICK) as u32).any(|_| {
        world.tick_once();
        world.objects[at as usize].users.contains(&Some(0)) && world.sims[0].pose == sim_core::content::Pose::Lie
    });
    assert!(on, "Ada lies on the shut-in lounger");
}

/// A thing bought resized: `(def, x, z, rot, size in cm, cm taller)`.
type Resized = (&'static str, i32, i32, i32, [u16; 2], i16);

/// Things with moving parts in `garden_home()`'s room, each against a wall, as `review-resized`
/// resizes them (A-systems-01).
const RESIZED: [Resized; 5] = [
    ("fridge", 4, 10, 2, [200, 100], 30),
    ("toilet", 7, 10, 2, [100, 100], 25),
    ("filingCabinet", 13, 10, 2, [100, 100], 40),
    ("laundryWasher", 16, 10, 2, [200, 100], 20),
    ("wallBed", 12, 3, 0, [300, 300], 30),
];

#[test]
fn review_resized() {
    for (resized, name) in [(true, "review-resized"), (false, "review-resized-base")] {
        let (mut world, _) = common::garden_home();
        for (def, x, z, rot, size, tall) in RESIZED {
            let (size, tall) = if resized { (Some(size), Some(tall)) } else { (None, None) };
            world
                .apply(Command::Buy { household: 0, object: def.into(), at: Some([x, z, rot]), colors: Default::default(), turn: None, offset: None, size, tall })
                .unwrap_or_else(|e| panic!("{def}: {e}"));
            let o = world.objects.last().unwrap();
            assert_eq!(o.tall, tall.unwrap_or(0), "{def} as bought");
        }
        for i in 0..world.sims.len() {
            set_needs(&mut world, i, 0.5, &[]);
        }
        common::dump_state(&world, name);
    }
}
