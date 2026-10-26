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

/// An empty house with the public place `id` next door, Ada at home.
fn town_beside(id: &str) -> (World, u32) {
    town_beside_with(id, 1)
}

/// `town_beside` with `n` residents (Ada, Bo, ...).
fn town_beside_with(id: &str, n: usize) -> (World, u32) {
    town_beside_edited(id, n, |_| {})
}

/// `town_beside_with` on content changed by `edit`.
fn town_beside_edited(id: &str, n: usize, edit: impl Fn(&mut Value)) -> (World, u32) {
    let mut edited: Value = serde_json::from_str(&common::content()).unwrap();
    edit(&mut edited);
    let merged = edited.to_string();
    let content: Value = serde_json::from_str(&merged).unwrap();
    let houses = houses();
    let (w, d) = plot_size(&houses);
    let market = houses["parks"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap_or_else(|| panic!("a place '{id}'"));
    let (walls, doors, windows, objects, entry) = stamp(&content, market, (w, d), w, false);
    let sims: Vec<Value> = ["Ada", "Bo", "Cy"][..n]
        .iter()
        .enumerate()
        .map(|(i, name)| json!({"name": name, "traits": ["cheerful"], "x": w as f64 / 2.0 + i as f64, "z": 3.5}))
        .collect();
    let lot = json!({
        "width": w * 2, "depth": d, "walls": walls, "doors": doors, "windows": windows, "objects": objects,
        "plots": [{"name": "Home", "x": 0, "z": 0, "w": w, "d": d, "entry": [w as f64 / 2.0, 1.5]},
                  {"name": market["name"], "x": w, "z": 0, "w": w, "d": d, "public": true, "entry": entry}],
        "households": [{"name": "Ada's", "plot": 0, "player": true, "funds": 500}],
        "sims": sims,
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

// ---- Vendors and shopkeepers (I-community-09, Q-17; `staff.rs`) ---------------------------------

use sim_core::snapshot::{self, HEADER_LEN, SIM_STRIDE, header, sim};

/// Minutes from now to `hour` (later today).
fn until(w: &mut World, hour: f32) {
    let now = sim_core::clock::hour(w.tick);
    minutes(w, ((hour - now) * 60.0).round() as u32);
}

/// The snapshot as the renderer gets it: its header, and server `k`'s row (after the residents').
fn staff_row(w: &World, k: usize) -> (Vec<f32>, Vec<f32>) {
    let mut out = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut out);
    let row = HEADER_LEN + (w.sims.len() + k) * SIM_STRIDE;
    (out[..HEADER_LEN].to_vec(), out[row..row + SIM_STRIDE].to_vec())
}

/// Resident `s`'s row of the snapshot.
fn sim_row(w: &World, s: usize) -> Vec<f32> {
    let mut out = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut out);
    out[HEADER_LEN + s * SIM_STRIDE..HEADER_LEN + (s + 1) * SIM_STRIDE].to_vec()
}

fn anim(w: &World, id: &str) -> f32 {
    w.content.animations.iter().position(|a| a == id).unwrap() as f32
}

/// Every cart, stall, counter, case and café machine in the town's places has someone behind it,
/// on its far side from the customer, on either side of the street; the posts next to each other
/// in the café and the bakery share one; a town with every place fits them in the snapshot.
#[test]
fn every_post_in_town_has_someone_behind_it() {
    let merged = common::content();
    let content: Value = serde_json::from_str(&merged).unwrap();
    let houses = houses();
    let (w, d) = plot_size(&houses);
    let mut most = 0;
    for place in houses["parks"].as_array().unwrap() {
        let mut counts = vec![];
        for rotated in [false, true] {
            let id = format!("{}{}", place["id"], if rotated { " (turned)" } else { "" });
            let (walls, doors, windows, objects, entry) = stamp(&content, place, (w, d), 0, rotated);
            let lot = json!({
                "width": w, "depth": d, "walls": walls, "doors": doors, "windows": windows, "objects": objects,
                "plots": [{"name": place["name"], "x": 0, "z": 0, "w": w, "d": d, "public": true, "entry": entry}],
            });
            let world = World::from_json(&merged, &lot.to_string(), 1).unwrap();
            let staff = sim_core::staff::plan(&world);
            for o in &world.objects {
                let def = &world.content.objects[o.def];
                if def.staff.is_none() {
                    continue;
                }
                let k = staff.server_of(o.id).unwrap_or_else(|| panic!("{id}: nobody serves at the {} at {},{}", def.id, o.x, o.z));
                // Behind it: on the side away from its front, where customers stand.
                let (at, _) = sim_core::staff::place(&world.content, o).unwrap();
                let c = o.centre(&world.content);
                let front = [o.yaw().sin(), o.yaw().cos()];
                let behind = (at[0] - c[0]) * front[0] + (at[1] - c[1]) * front[1];
                assert!(behind < -0.5, "{id}: the {}'s server stands behind it ({behind})", def.id);
                assert!(staff.servers[k].posts.contains(&o.id));
            }
            counts.push(staff.servers.len());
        }
        assert_eq!(counts[0], counts[1], "{}: as many servers on either side", place["id"]);
        most += counts[0];
        eprintln!("{}: {} servers", place["id"], counts[0]);
    }
    assert!(most <= sim_core::staff::MAX_STAFF, "{most} servers in a town with every place");
    // The café's counter and coffee machine share a barista; the bakery has two at its counter.
    let (cafe, _) = town_beside("cafe");
    let staff = sim_core::staff::plan(&cafe);
    let (counter, machine) = (object(&cafe, "shopCounter"), object(&cafe, "coffeeMachine"));
    assert_eq!(staff.server_of(counter), staff.server_of(machine), "the café's counter and machine share a server");
    let (bakery, _) = town_beside("bakery");
    assert_eq!(sim_core::staff::plan(&bakery).servers.len(), 2, "two behind the bakery's counter");
}

/// In the park at 10:30 the ice cream seller is there: Ada orders, is served (the seller shown
/// serving, handing over the cone), pays four dollars once and eats it sitting down.
#[test]
fn a_vendor_serves_an_ice_cream() {
    let (mut w, park) = town_beside("park");
    w.autonomy = false;
    until(&mut w, 10.5);
    let cart = object(&w, "iceCreamCart");
    assert_eq!(w.plot_at(w.objects[cart as usize].x, w.objects[cart as usize].z), Some(park));
    let k = w.staff.server_of(cart).expect("someone sells ice cream");
    let (head, row) = staff_row(&w, k);
    assert!(head[header::STAFF_COUNT] as usize > k);
    assert_eq!(row[sim::AWAY], 0.0, "at work at 10:30");
    let funds = w.households[0].funds;
    let hunger = w.content.needs.iter().position(|n| n.id == "hunger").unwrap();
    w.sims[0].needs[hunger] = 0.3;
    w.apply(Command::Use { sim: 0, object: cart, interaction: interaction(&w, cart, "iceCream") }).unwrap();
    let (sell, buy) = (anim(&w, "sell"), anim(&w, "buy"));
    let cone = w.content.items.iter().position(|i| i.id == "iceCreamCone").unwrap() as f32 + 1.0;
    let (mut served, mut handed, mut ate) = (false, false, false);
    for _ in 0..40 * TICKS_PER_MINUTE {
        w.tick_once();
        let (_, row) = staff_row(&w, k);
        if row[sim::ACTION] == sell {
            served = true;
            assert_eq!(w.staff.servers[k].serving, Some(0), "serving Ada");
            handed |= row[sim::HELD] == cone;
            assert_eq!(sim_row(&w, 0)[sim::ACTION], buy, "Ada buying while she's served");
        }
        ate |= w.sims[0].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Eat { .. }));
    }
    assert!(served && handed, "the seller served Ada ({served}) and handed her a cone ({handed})");
    assert!(ate, "and she ate it");
    assert_eq!(w.households[0].funds, funds - 4, "paid once, four dollars");
    assert!(w.sims[0].needs[hunger] > 0.4, "hunger {}", w.sims[0].needs[hunger]);
}

/// Before the cart opens (10:00) and after it closes (20:00) nobody's there: the order is refused,
/// saying when it opens, and a hungry resident doesn't go for one; the snack machine still sells.
#[test]
fn off_shift_the_cart_is_closed() {
    let (mut w, _) = town_beside("market");
    let cart = object(&w, "iceCreamCart");
    let err = w.apply(Command::Use { sim: 0, object: cart, interaction: interaction(&w, cart, "iceCream") }).unwrap_err();
    assert!(err.to_string().contains("closed: it opens at 10:00"), "{err}");
    minutes(&mut w, 1);
    let k = w.staff.server_of(cart).expect("a post");
    assert_eq!(staff_row(&w, k).1[sim::AWAY], 1.0, "nobody at the cart at 8:00");
    until(&mut w, 21.5);
    assert_eq!(staff_row(&w, k).1[sim::AWAY], 1.0, "nor at 21:30");
    assert!(w.apply(Command::Use { sim: 0, object: cart, interaction: interaction(&w, cart, "iceCream") }).is_err());
    // Free will looks elsewhere.
    let hunger = w.content.needs.iter().position(|n| n.id == "hunger").unwrap();
    w.sims[0].needs[hunger] = 0.15;
    let mut at_cart = false;
    for _ in 0..90 {
        minutes(&mut w, 1);
        at_cart |= w.sims[0].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Use { object, .. } if object == cart));
    }
    assert!(!at_cart, "nobody goes to a closed cart");
    let machine = object(&w, "vendingSnack");
    w.apply(Command::Use { sim: 0, object: machine, interaction: interaction(&w, machine, "snack") }).expect("machines sell all night");
}

/// At the café the barista keeps the counter and the coffee machine: a coffee ordered steps them
/// over to the machine; once the café shuts, *Have a coffee* at a table is refused.
#[test]
fn the_barista_steps_over_to_the_machine_and_the_cafe_closes() {
    let (mut w, cafe) = town_beside("cafe");
    w.autonomy = false;
    minutes(&mut w, 1);
    let (counter, machine) = (object(&w, "shopCounter"), object(&w, "coffeeMachine"));
    let k = w.staff.server_of(machine).unwrap();
    let home = sim_core::staff::place(&w.content, &w.objects[counter as usize]).unwrap().0;
    assert_eq!(w.staff.servers[k].pos, home, "at the counter when the café opens");
    let chair = w.objects.iter().find(|o| w.content.objects[o.def].id == "cafeChair" && w.plot_at(o.x, o.z) == Some(cafe)).unwrap().id;
    w.apply(Command::Use { sim: 0, object: chair, interaction: interaction(&w, chair, "coffee") }).unwrap();
    let to = sim_core::staff::place(&w.content, &w.objects[machine as usize]).unwrap().0;
    let mut stepped = false;
    for _ in 0..30 * TICKS_PER_MINUTE {
        w.tick_once();
        stepped |= w.staff.servers[k].serving == Some(0) && w.staff.servers[k].pos == to;
    }
    assert!(stepped, "the barista served Ada's coffee at the machine");
    until(&mut w, 19.5);
    let err = w.apply(Command::Use { sim: 0, object: chair, interaction: interaction(&w, chair, "coffee") }).unwrap_err();
    assert!(err.to_string().contains("coffee machine is closed"), "{err}");
}

/// Two at a market stall at once: its stallholder serves one, then the other.
#[test]
fn two_customers_are_served_in_turn() {
    let (mut w, market) = town_beside_with("market", 2);
    w.autonomy = false;
    let stall = w.objects.iter().find(|o| w.content.objects[o.def].id == "marketStall" && w.plot_at(o.x, o.z) == Some(market)).unwrap().id;
    let fruit = interaction(&w, stall, "fruit");
    for s in [0, 1] {
        w.apply(Command::Use { sim: s, object: stall, interaction: fruit }).unwrap();
    }
    minutes(&mut w, 1);
    let k = w.staff.server_of(stall).unwrap();
    let mut order = vec![];
    for _ in 0..40 * TICKS_PER_MINUTE {
        w.tick_once();
        if let Some(c) = w.staff.servers[k].serving
            && order.last() != Some(&c)
        {
            order.push(c);
        }
        let buying = (0..2)
            .filter(|&s| {
                w.sims[s].current().is_some_and(|a| match (a.task.kind, &a.phase) {
                    (TaskKind::Use { object, .. }, Phase::Using { elapsed }) => object == stall && *elapsed > 0.0,
                    _ => false,
                })
            })
            .count();
        assert!(buying <= 1, "one served at a time");
    }
    order.sort();
    assert_eq!(order, vec![0, 1], "both were served, one after the other");
}

/// A cart in a garden of their own has nobody behind it: they help themselves at any hour, as its
/// `selfServe` says.
#[test]
fn a_cart_at_home_is_self_serve() {
    let (mut w, _) = town_beside("park");
    w.autonomy = false;
    w.households[0].funds = 5000;
    let buy: Command = serde_json::from_value(json!({"type": "buy", "household": 0, "object": "iceCreamCart"})).unwrap();
    w.apply(buy).unwrap();
    let cart = w.objects.iter().find(|o| w.content.objects[o.def].id == "iceCreamCart" && w.plot_at(o.x, o.z) == Some(0)).unwrap().id;
    until(&mut w, 22.0);
    assert!(w.staff.server_of(cart).is_none(), "nobody serves at home");
    w.apply(Command::Use { sim: 0, object: cart, interaction: interaction(&w, cart, "iceCream") }).expect("open all hours at home");
    let fetch = anim(&w, "fetch");
    let mut helped = false;
    for _ in 0..20 * TICKS_PER_MINUTE {
        w.tick_once();
        let row = sim_row(&w, 0);
        helped |= row[sim::OBJECT] == w.objects[cart as usize].stable as f32 && row[sim::ACTION] == fetch;
    }
    assert!(helped, "she took one from the freezer herself");
}

/// States to look at the vendors with (`DUMP_STATES=1`; tools/bench/README.md): each town place beside an empty
/// house at 11:30, everyone serving there at work, Ada and Bo at the house's front, free will off; and
/// `vendors-<place>-night`, the same at 21:30, everyone gone home.
#[test]
fn vendor_states_for_play() {
    for id in ["park", "market", "cafe", "grocer", "bakery"] {
        let (mut w, _) = town_beside_with(id, 2);
        w.autonomy = false;
        until(&mut w, 11.5);
        assert!(!w.staff.servers.is_empty() && w.staff.servers.iter().all(|s| s.present), "{id}: everyone at work at 11:30");
        common::dump_state(&w, &format!("vendors-{id}"));
        // And at 21:30, everything closed but the hot dog stand (till 21:00: closed too) and the machines.
        until(&mut w, 21.5);
        assert!(w.staff.servers.iter().all(|s| !s.present), "{id}: everyone gone home at 21:30");
        common::dump_state(&w, &format!("vendors-{id}-night"));
    }
}

/// The town's staff are nobody: a day at the market with free will on adds no resident, household or
/// career, and the household pays for what Ada and Bo buy and nothing more (the staff's wages are no one's
/// in town).
#[test]
fn staff_are_not_residents_and_cost_nothing_but_what_is_bought() {
    let (mut w, _) = town_beside_with("market", 2);
    let careers = w.content.careers.len();
    let (mut bought, mut paid) = (0, 0);
    let mut was: [Option<(u32, usize)>; 2] = [None; 2];
    for _ in 0..24 * 60 * TICKS_PER_MINUTE {
        let funds = w.households[0].funds;
        w.tick_once();
        paid += funds - w.households[0].funds;
        for s in 0..2 {
            // A use paid for as it starts (on arrival).
            let now = w.sims[s].current().and_then(|a| match (a.task.kind, &a.phase) {
                (TaskKind::Use { object, interaction }, Phase::Using { .. }) => Some((object, interaction)),
                _ => None,
            });
            if let Some((o, i)) = now
                && was[s] != now
            {
                bought += w.content.objects[w.objects[o as usize].def].interactions[i].cost;
            }
            was[s] = now;
        }
    }
    assert_eq!(w.sims.len(), 2, "nobody moved in");
    assert_eq!(w.households.len(), 1);
    assert_eq!(w.content.careers.len(), careers, "no vendor's career");
    assert!(!w.staff.servers.is_empty());
    eprintln!("bought {bought}, paid {paid}");
    assert!(bought > 0, "they bought something in a day");
    assert!(paid > 0 && paid <= bought, "they paid for what they bought (less any given up), and nothing for the staff");
}

/// Bo, waiting behind Ada at the stall longer than `service.waitMinutes` (here a few seconds), gives up
/// and gets his money back; Ada is served and pays.
#[test]
fn a_customer_who_waits_too_long_gives_up_and_is_paid_back() {
    let (mut w, market) = town_beside_edited("market", 2, |c| c["service"]["waitMinutes"] = json!(0.2));
    w.autonomy = false;
    let stall = w.objects.iter().find(|o| w.content.objects[o.def].id == "marketStall" && w.plot_at(o.x, o.z) == Some(market)).unwrap().id;
    let fruit = interaction(&w, stall, "fruit");
    let funds = w.households[0].funds;
    for s in [0, 1] {
        w.apply(Command::Use { sim: s, object: stall, interaction: fruit }).unwrap();
    }
    let mut ate = [false; 2];
    for _ in 0..40 * TICKS_PER_MINUTE {
        w.tick_once();
        for s in 0..2 {
            ate[s] |= w.sims[s].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Eat { .. }));
        }
    }
    assert_eq!(ate.iter().filter(|&&a| a).count(), 1, "one of them was served ({ate:?})");
    assert_eq!(w.households[0].funds, funds - 5, "the other paid nothing in the end");
}

/// Last orders: Ada sets off for the ice cream cart a minute before it closes (20:00) and gets there
/// after; the seller waits for her and serves her, then goes home.
#[test]
fn last_orders_at_closing_time() {
    let (mut w, _) = town_beside("park");
    w.autonomy = false;
    until(&mut w, 19.0 + 59.0 / 60.0);
    let cart = object(&w, "iceCreamCart");
    let k = w.staff.server_of(cart).unwrap();
    w.apply(Command::Use { sim: 0, object: cart, interaction: interaction(&w, cart, "iceCream") }).unwrap();
    let (mut served_after, mut ate) = (false, false);
    for _ in 0..30 * TICKS_PER_MINUTE {
        w.tick_once();
        served_after |= sim_core::clock::hour(w.tick) >= 20.0 && w.staff.servers[k].serving == Some(0);
        ate |= w.sims[0].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Eat { .. }));
    }
    assert!(served_after && ate, "served after 20:00 ({served_after}) and ate it ({ate})");
    assert!(!w.staff.servers[k].present, "and the seller went home");
}
