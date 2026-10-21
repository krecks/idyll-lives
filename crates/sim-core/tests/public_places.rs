//! The public places of `houses.json` (`parks`: the town park, the playground, the market square,
//! the café and the shops) and the community things in them: every place loads on either side of
//! the street (the town file is strict, one misfit and the town doesn't load), everything in it is
//! within reach of the street, and residents use what is there: carts and machines charge them,
//! benches seat them, and hungry people go out for a bite when there's nothing at home.

mod common;

use std::collections::{BTreeSet, VecDeque};

use serde_json::{Value, json};
use sim_core::content::Approach;
use sim_core::path::NavGrid;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, MINUTES_PER_TICK, World};

const TICKS_PER_MINUTE: u32 = (1.0 / MINUTES_PER_TICK) as u32;

fn houses() -> Value {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/content");
    serde_json::from_str(&std::fs::read_to_string(dir.join("houses.json")).unwrap()).unwrap()
}

fn footprint(content: &Value, def: &str) -> (i64, i64) {
    content["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == def)
        .and_then(|o| o["footprint"].as_array())
        .map_or((1, 1), |f| (f[0].as_i64().unwrap(), f[1].as_i64().unwrap()))
}

/// What `stamp` gives: walls, doors, windows, objects, and where visitors arrive.
type Stamped = (Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>, [f64; 2]);

/// A public place stamped on a plot at `(x0, 0)` of a medium lot's size, turned round (the south
/// side of the street) like `town.ts` does it: walls, doors, windows, objects (turns kept,
/// offsets the other way) and where visitors arrive.
fn stamp(content: &Value, place: &Value, (w, d): (i64, i64), x0: i64, rotated: bool) -> Stamped {
    let n = |v: &Value| v.as_i64().unwrap();
    let list = |k: &str| place[k].as_array().cloned().unwrap_or_default();
    let walls = list("walls")
        .iter()
        .map(|c| {
            let c: Vec<i64> = c.as_array().unwrap().iter().map(n).collect();
            if rotated {
                json!([x0 + w - c[0], d - c[1], x0 + w - c[2], d - c[3]])
            } else {
                json!([x0 + c[0], c[1], x0 + c[2], c[3]])
            }
        })
        .collect();
    let edge = |e: &Value| {
        let (x, z, axis) = (n(&e["x"]), n(&e["z"]), e["axis"].as_str().unwrap());
        match (rotated, axis) {
            (false, _) => json!({"x": x0 + x, "z": z, "axis": axis}),
            (true, "x") => json!({"x": x0 + w - 1 - x, "z": d - z, "axis": "x"}),
            (true, _) => json!({"x": x0 + w - x, "z": d - 1 - z, "axis": "z"}),
        }
    };
    let doors = list("doors").iter().map(edge).collect();
    let windows = list("windows").iter().map(edge).collect();
    let objects = list("objects")
        .iter()
        .map(|o| {
            let def = o["def"].as_str().unwrap();
            let (fw, fd) = footprint(content, def);
            let rot = n(&o["rot"]);
            let (ow, od) = if rot % 2 == 0 { (fw, fd) } else { (fd, fw) };
            let offset = o["offset"].as_array().map(|v| [n(&v[0]), n(&v[1])]).unwrap_or([0, 0]);
            if rotated {
                json!({"def": def, "x": x0 + w - n(&o["x"]) - ow, "z": d - n(&o["z"]) - od, "rot": (rot + 2) % 4,
                       "turn": o["turn"].as_i64().unwrap_or(0), "offset": [-offset[0], -offset[1]]})
            } else {
                json!({"def": def, "x": x0 + n(&o["x"]), "z": n(&o["z"]), "rot": rot,
                       "turn": o["turn"].as_i64().unwrap_or(0), "offset": offset})
            }
        })
        .collect();
    let e = place["entry"].as_array().map_or([w as f64 / 2.0, 1.5], |e| [e[0].as_f64().unwrap(), e[1].as_f64().unwrap()]);
    let entry = if rotated { [(x0 + w) as f64 - e[0], d as f64 - e[1]] } else { [x0 as f64 + e[0], e[1]] };
    (walls, doors, windows, objects, entry)
}

fn plot_size(houses: &Value) -> (i64, i64) {
    (houses["plot"]["width"].as_i64().unwrap(), houses["plot"]["depth"].as_i64().unwrap())
}

/// Every place, on either side of the street, loads with all its things where the layout put
/// them, and every thing someone can use is within reach of where visitors arrive.
#[test]
fn every_public_place_loads_on_either_side_of_the_street() {
    let merged = common::content();
    let content: Value = serde_json::from_str(&merged).unwrap();
    let houses = houses();
    let (w, d) = plot_size(&houses);
    let places = houses["parks"].as_array().unwrap();
    assert!(places.len() >= 6, "the park, the playground, the market, the café and two shops");
    assert!(places.iter().any(|p| p["kind"] == "shop" && p["walls"].is_array()), "shops are shells with walls");
    for place in places {
        for rotated in [false, true] {
            let id = format!("{}{}", place["id"], if rotated { " (turned)" } else { "" });
            let (walls, doors, windows, objects, entry) = stamp(&content, place, (w, d), 0, rotated);
            let lot = json!({
                "width": w, "depth": d, "walls": walls, "doors": doors, "windows": windows, "objects": objects,
                "plots": [{"name": place["name"], "x": 0, "z": 0, "w": w, "d": d, "public": true, "entry": entry}],
            });
            let world = World::from_json(&merged, &lot.to_string(), 1).unwrap_or_else(|e| panic!("{id}: {e}"));
            assert_eq!(world.objects.len(), objects.len(), "{id}");
            for (o, raw) in world.objects.iter().zip(&objects) {
                assert_eq!(o.turn as i64, raw["turn"].as_i64().unwrap(), "{id}: {} keeps its turn", raw["def"]);
            }
            // Everything someone can use is within reach of the street (through the shop door).
            let nav = NavGrid { lot: &world.lot, blocked: world.blocked(), stairs: world.stair_map() };
            let start = (entry[0].floor() as i32, entry[1].floor() as i32);
            assert!(nav.tile_free(start.0, start.1), "{id}: visitors arrive on a free tile");
            let mut seen = BTreeSet::from([start]);
            let mut open = VecDeque::from([start]);
            while let Some((x, z)) = open.pop_front() {
                for (nx, nz) in [(x + 1, z), (x - 1, z), (x, z + 1), (x, z - 1)] {
                    if nav.can_step(x, z, nx, nz) && seen.insert((nx, nz)) {
                        open.push_back((nx, nz));
                    }
                }
            }
            for o in &world.objects {
                let def = &world.content.objects[o.def];
                if def.interactions.is_empty() {
                    continue;
                }
                let reached = if def.approach == Approach::Around {
                    let (ow, od) = o.size(&world.content);
                    (o.x..o.x + ow).flat_map(|x| (o.z..o.z + od).map(move |z| (x, z))).any(|(x, z)| [(x + 1, z), (x - 1, z), (x, z + 1), (x, z - 1)].iter().any(|t| seen.contains(t)))
                } else {
                    seen.contains(&o.front_tile(&world.content))
                };
                assert!(reached, "{id}: nobody can get to the {} at {},{}", def.id, o.x, o.z);
            }
        }
    }
}

/// Houses and places only use things the content has.
#[test]
fn houses_and_places_use_known_things() {
    let merged = common::content();
    let content = sim_core::Content::from_json(&merged).unwrap();
    let houses = houses();
    let lists = houses["houses"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|h| [&h["objects"], &h["upper"]["objects"]])
        .chain(houses["parks"].as_array().unwrap().iter().map(|p| &p["objects"]));
    let mut n = 0;
    for o in lists.flat_map(|l| l.as_array().into_iter().flatten()) {
        let def = o["def"].as_str().unwrap();
        assert!(content.object_index(def).is_some(), "unknown object '{def}' in houses.json");
        n += 1;
    }
    assert!(n > 200, "{n} things");
}

/// A house with nothing in it next to the market square: residents pay at the machines and
/// carts, sit on the benches, and go there for food on their own when hungry.
fn market_town() -> (World, u32) {
    town_beside("market")
}

/// An empty house with the public place `id` next door.
fn town_beside(id: &str) -> (World, u32) {
    let merged = common::content();
    let content: Value = serde_json::from_str(&merged).unwrap();
    let houses = houses();
    let (w, d) = plot_size(&houses);
    let market = houses["parks"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap_or_else(|| panic!("a place '{id}'"));
    let (walls, doors, windows, objects, entry) = stamp(&content, market, (w, d), w, false);
    let lot = json!({
        "width": w * 2, "depth": d, "walls": walls, "doors": doors, "windows": windows, "objects": objects,
        "plots": [{"name": "Home", "x": 0, "z": 0, "w": w, "d": d, "entry": [w as f64 / 2.0, 1.5]},
                  {"name": market["name"], "x": w, "z": 0, "w": w, "d": d, "public": true, "entry": entry}],
        "households": [{"name": "Ada's", "plot": 0, "player": true, "funds": 500}],
        "sims": [{"name": "Ada", "traits": ["cheerful"], "x": w as f64 / 2.0, "z": 3.5}],
    });
    let world = World::from_json(&merged, &lot.to_string(), 7).unwrap();
    (world, 1)
}

fn object(w: &World, id: &str) -> u32 {
    w.objects.iter().find(|o| w.content.objects[o.def].id == id).unwrap_or_else(|| panic!("a {id}")).id
}

fn interaction(w: &World, object: u32, id: &str) -> usize {
    w.content.objects[w.objects[object as usize].def].interactions.iter().position(|i| i.id == id).unwrap()
}

fn minutes(w: &mut World, n: u32) {
    for _ in 0..n * TICKS_PER_MINUTE {
        w.tick_once();
    }
}

#[test]
fn residents_buy_a_snack_and_sit_on_a_bench_at_the_market() {
    let (mut w, market) = market_town();
    w.autonomy = false;
    let hunger = w.content.needs.iter().position(|n| n.id == "hunger").unwrap();
    let comfort = w.content.needs.iter().position(|n| n.id == "comfort").unwrap();
    w.sims[0].needs[hunger] = 0.2;
    let machine = object(&w, "vendingSnack");
    let at = &w.objects[machine as usize];
    assert_eq!(w.plot_at(at.x, at.z), Some(market));
    let funds = w.households[0].funds;
    w.apply(Command::Use { sim: 0, object: machine, interaction: interaction(&w, machine, "snack") }).unwrap();
    // Fetched from the machine, then eaten sitting down (a bench), not at the machine.
    let mut seated = None;
    for _ in 0..60 {
        minutes(&mut w, 1);
        if let Some(a) = w.sims[0].current()
            && let (TaskKind::Eat { seat: Some(seat), .. }, Phase::Using { .. }) = (a.task.kind, &a.phase)
        {
            seated = Some(seat);
        }
    }
    let seat = seated.expect("eaten sitting down");
    assert_eq!(w.plot_at(w.objects[seat as usize].x, w.objects[seat as usize].z), Some(market), "at the market");
    assert!(w.sims[0].needs[hunger] > 0.4, "a snack from the machine: hunger {}", w.sims[0].needs[hunger]);
    assert_eq!(w.households[0].funds, funds - 4, "and it cost four dollars");

    w.sims[0].needs[comfort] = 0.2;
    let bench = object(&w, "parkBench");
    w.apply(Command::Use { sim: 0, object: bench, interaction: interaction(&w, bench, "sit") }).unwrap();
    let mut sat = false;
    for _ in 0..30 {
        minutes(&mut w, 1);
        sat |= w.sims[0].current().is_some_and(|a| {
            matches!((a.task.kind, &a.phase), (TaskKind::Use { object, .. }, Phase::Using { .. }) if object == bench)
        });
    }
    assert!(sat, "Ada sat down on the bench");
    assert!(w.sims[0].needs[comfort] > 0.25, "and rested there: comfort {}", w.sims[0].needs[comfort]);
}

#[test]
fn hungry_residents_with_an_empty_house_eat_out() {
    let (mut w, market) = market_town();
    let hunger = w.content.needs.iter().position(|n| n.id == "hunger").unwrap();
    w.sims[0].needs[hunger] = 0.15;
    let funds = w.households[0].funds;
    let mut ate_out = false;
    for _ in 0..180 {
        minutes(&mut w, 1);
        ate_out |= w.sims[0].current().is_some_and(|a| match a.task.kind {
            TaskKind::Use { object, interaction } => {
                let o = &w.objects[object as usize];
                w.plot_at(o.x, o.z) == Some(market)
                    && w.content.objects[o.def].interactions[interaction].total_gain[hunger] > 0.0
            }
            _ => false,
        });
    }
    assert!(ate_out, "Ada went out for something to eat");
    assert!(w.households[0].funds < funds, "and paid for it");
    assert!(w.sims[0].needs[hunger] > 0.3, "and isn't hungry any more: {}", w.sims[0].needs[hunger]);
}

/// At the café the chairs pull up to the little tables: coffee is served there (and paid for).
#[test]
fn a_coffee_at_a_cafe_table() {
    let (mut w, cafe) = town_beside("cafe");
    w.autonomy = false;
    let chair = w
        .objects
        .iter()
        .find(|o| w.content.objects[o.def].id == "cafeChair" && w.plot_at(o.x, o.z) == Some(cafe))
        .expect("café chairs")
        .id;
    let funds = w.households[0].funds;
    let machine = object(&w, "coffeeMachine");
    w.apply(Command::Use { sim: 0, object: chair, interaction: interaction(&w, chair, "coffee") }).unwrap();
    // Poured at the café's machine first, then had at the table.
    let (mut poured, mut sat) = (None, None);
    for m in 0..50 {
        minutes(&mut w, 1);
        if let Some(a) = w.sims[0].current()
            && matches!(a.phase, Phase::Using { .. })
            && let TaskKind::Use { object, interaction: i } = a.task.kind
        {
            if object == machine && w.content.objects[w.objects[machine as usize].def].interactions[i].id == "pour" {
                poured.get_or_insert(m);
            }
            if object == chair {
                sat.get_or_insert(m);
            }
        }
    }
    let (poured, sat) = (poured.expect("the coffee poured at the machine"), sat.expect("Ada sat down at a café table"));
    assert!(poured < sat, "poured first ({poured}), then sat ({sat})");
    assert_eq!(w.households[0].funds, funds - 4, "and paid for her coffee");
}

/// Cake and pastries at the café come from its case: the cake is taken to the table first, a
/// pastry is eaten at a table, its plate there till it's eaten (the café's, off Ada's plot).
#[test]
fn cake_and_a_pastry_from_the_case_at_a_cafe_table() {
    let (mut w, cafe) = town_beside("cafe");
    w.autonomy = false;
    let hunger = w.content.needs.iter().position(|n| n.id == "hunger").unwrap();
    let case = object(&w, "bakeryCase");
    let chair = w
        .objects
        .iter()
        .find(|o| w.content.objects[o.def].id == "cafeChair" && w.plot_at(o.x, o.z) == Some(cafe))
        .expect("café chairs")
        .id;
    let take = interaction(&w, case, "takeCake");
    w.apply(Command::Use { sim: 0, object: chair, interaction: interaction(&w, chair, "cake") }).unwrap();
    let (mut took, mut ate) = (false, false);
    for _ in 0..50 {
        minutes(&mut w, 1);
        let at = w.sims[0].current().filter(|a| matches!(a.phase, Phase::Using { .. })).map(|a| a.task.kind);
        took |= matches!(at, Some(TaskKind::Use { object, interaction }) if object == case && interaction == take);
        ate |= took && matches!(at, Some(TaskKind::Use { object, .. }) if object == chair);
    }
    assert!(took && ate, "the cake taken from the case ({took}), then eaten at the table ({ate})");

    w.sims[0].needs[hunger] = 0.2;
    w.apply(Command::Use { sim: 0, object: case, interaction: interaction(&w, case, "pastry") }).unwrap();
    let mut plate_kept = None;
    for _ in 0..60 {
        minutes(&mut w, 1);
        if let Some(a) = w.sims[0].current()
            && let (TaskKind::Eat { seat: Some(seat), plate: Some(p), .. }, Phase::Using { elapsed }) = (a.task.kind, &a.phase)
        {
            assert!(w.items.get(p).is_some(), "the plate stays on the table while it's eaten ({elapsed} minutes in)");
            assert_eq!(w.plot_at(w.objects[seat as usize].x, w.objects[seat as usize].z), Some(cafe));
            plate_kept = Some(*elapsed);
        }
    }
    assert!(plate_kept.is_some_and(|e| e > 7.0), "eaten at a table to the end: {plate_kept:?}");
    assert!(w.sims[0].needs[hunger] > 0.45, "hunger {}", w.sims[0].needs[hunger]);
}
