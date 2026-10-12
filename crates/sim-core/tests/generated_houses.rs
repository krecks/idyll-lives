//! Generated houses (web/src/game/housegen.ts) checked by the simulation: each loads, has a
//! bed, a fridge and a toilet in rooms of their own, and everything in it can be reached from the
//! front door, upstairs too. Chairs face tables and things on tops stand on one; most houses have
//! somewhere to sit down at a table and small things on their tops. No room is bigger than its
//! kind may be (`maxSide` in housegen.json), each is a room of its own in the simulation, and a
//! good share of houses on large lots have a kitchen island with stools pulled up to it. Every
//! room has lights enough for all of it at night, and nothing stands just inside a door.
//!
//! Make the houses first: `node --experimental-strip-types tools/housegen/sample.ts houses.json`,
//! then `HOUSES=houses.json cargo test -p sim-core --test generated_houses -- --ignored --nocapture`.

mod common;

use sim_core::World;
use sim_core::content::{Layer, SurfaceKind};
use sim_core::lot::{Edge, OUTDOORS};
use sim_core::path::NavGrid;
use sim_core::surface;

#[test]
#[ignore]
fn generated_houses_are_livable() {
    let Ok(path) = std::env::var("HOUSES") else {
        eprintln!("set HOUSES to a file made by tools/housegen/sample.ts");
        return;
    };
    let houses: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let content = common::content();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/content");
    let config: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("housegen.json")).unwrap()).unwrap();
    let mut failures = Vec::new();
    let (mut dining, mut tops) = (0, 0);
    let (mut large, mut islands) = (0, 0);
    let (mut rooms, mut bright) = (0, 0);
    for h in &houses {
        let label = format!("{} {} ({})", h["size"], h["seed"], h["description"]);
        let w = match World::from_json(&content, &h["town"].to_string(), 1) {
            Ok(w) => w,
            Err(e) => {
                failures.push(format!("{label}: {e}"));
                continue;
            }
        };
        // Things on tops stand on one listed before them.
        common::check(&w);
        // Rooms as planned: no bigger than their kind may be, walled off (or open to the next).
        for r in h["rooms"].as_array().into_iter().flatten() {
            let kind = r["kind"].as_str().unwrap();
            let [x, z, rw, rd] = ["x", "z", "w", "d"].map(|k| r[k].as_i64().unwrap() as i32);
            if let Some(cap) = config["rooms"][kind]["maxSide"].as_array() {
                let (short, long) = (cap[0].as_i64().unwrap() as i32, cap[1].as_i64().unwrap() as i32);
                if rw.min(rd) > short || rw.max(rd) > long {
                    failures.push(format!("{label}: a {kind} of {rw}×{rd}, more than {short}×{long}"));
                }
            }
            let id = w.lot.room_at(x, z);
            let whole = (z..z + rd).all(|tz| (x..x + rw).all(|tx| w.lot.room_at(tx, tz) == id));
            if id == OUTDOORS || !whole {
                failures.push(format!("{label}: the {kind} at {x},{z} isn't a room of its own"));
            }
        }
        let room_of = |is: &dyn Fn(&sim_core::content::ObjectDef) -> bool| {
            w.objects.iter().find(|o| is(&w.content.objects[o.def])).map(|o| w.lot.room_at(o.x, o.z))
        };
        let bed = room_of(&|d| d.interactions.iter().any(|i| i.id == "sleep"));
        let (fridge, toilet) = (room_of(&|d| d.id == "fridge"), room_of(&|d| d.id == "toilet"));
        for (what, room) in [("bed", bed), ("fridge", fridge), ("toilet", toilet)] {
            if room.is_none_or(|r| r == OUTDOORS) {
                failures.push(format!("{label}: no {what} indoors"));
            }
        }
        // Dinner can be made (a meal needs only somewhere to get ingredients: the rest of its
        // steps are done without when there's no counter or stove).
        let stations = w.objects.iter().fold(0, |m, o| m | w.content.objects[o.def].stations);
        let meal = w.content.recipe_index("meal").map(|r| &w.content.recipes[r]);
        if meal.is_some_and(|r| !r.possible(stations)) {
            failures.push(format!("{label}: can't make a meal"));
        }
        if bed.is_some() && (bed == toilet || bed == fridge) || fridge == toilet {
            failures.push(format!("{label}: bedroom, kitchen and bathroom share a room"));
        }
        // Everything reachable from the street.
        let entry = w.plots[0].arrival_point();
        let nav = NavGrid { lot: &w.lot, blocked: w.blocked(), stairs: w.stair_map() };
        let start = (entry[0].floor() as i32, entry[1].floor() as i32);
        let mut seated = 0;
        for o in &w.objects {
            let def = &w.content.objects[o.def];
            // Chairs are put round tables, facing them.
            if def.seat_at != 0 {
                if surface::faced_table(&w.content, &w.objects, &nav, o.id).is_some() {
                    seated += 1;
                } else {
                    failures.push(format!("{label}: the {} at {},{} faces no table", def.id, o.x, o.z));
                }
            }
            // Pictures, ceiling lamps, rugs and tables nobody uses needn't be reached; chairs at
            // a table from any free side.
            if def.interactions.is_empty() {
                continue;
            }
            let reached = surface::approach_tiles(&w.content, &nav, o).any(|t| nav.find_path(start, t).is_some());
            if !reached {
                failures.push(format!("{label}: can't reach the {} at {},{}", def.id, o.x, o.z));
            }
        }
        // Every room has a light, enough for all of it at night (an open kitchen and living room
        // share theirs).
        let lamp_tiles = w.content.room_rules.lamp_tiles;
        for r in h["rooms"].as_array().into_iter().flatten() {
            let kind = r["kind"].as_str().unwrap();
            let [x, z] = ["x", "z"].map(|k| r[k].as_i64().unwrap() as i32);
            let Some(room) = w.room_at_tile(x, z) else { continue };
            rooms += 1;
            bright += usize::from(room.scores.light >= 0.6);
            if room.lamps == 0 {
                failures.push(format!("{label}: the {kind} at {x},{z} has no light"));
            } else if f32::from(room.lamps) * lamp_tiles < room.tiles {
                failures.push(format!("{label}: the {kind} at {x},{z} has {} lights for {} tiles", room.lamps, room.tiles));
            }
        }
        // Nothing stands just inside a door (a lamp beside a doorway makes the way in a squeeze).
        let blocked = w.blocked();
        let (lw, ld) = (w.lot.width as i32, w.lot.depth as i32);
        let clear = |x: i32, z: i32| !w.lot.in_bounds(x, z) || !blocked[w.lot.tile_index(x, z)];
        for z in 0..=ld {
            for x in 0..=lw {
                let h_door = x < lw && w.lot.h_edge(x as usize, z as usize) == Edge::Door;
                let v_door = z < ld && w.lot.v_edge(x as usize, z as usize) == Edge::Door;
                for (door, a, b) in [(h_door, (x, z - 1), (x, z)), (v_door, (x - 1, z), (x, z))] {
                    for (tx, tz) in [a, b] {
                        if door && !clear(tx, tz) {
                            let what = w.objects.iter().find(|o| {
                                let (ow, od) = o.size(&w.content);
                                !w.content.objects[o.def].layer.mounted() && (o.x..o.x + ow).contains(&tx) && (o.z..o.z + od).contains(&tz)
                            });
                            let id = what.map_or("something", |o| w.content.objects[o.def].id.as_str());
                            failures.push(format!("{label}: the {id} at {tx},{tz} stands in a doorway"));
                        }
                    }
                }
            }
        }
        dining += usize::from(seated >= 2);
        // An island: a counter or bar with two stools or more facing it.
        if h["size"] == "large" {
            large += 1;
            let mut at = std::collections::HashMap::new();
            for o in &w.objects {
                if let Some(t) = surface::faced_table(&w.content, &w.objects, &nav, o.id) {
                    let kind = w.content.objects[w.objects[t as usize].def].surface.map(|s| s.kind);
                    if matches!(kind, Some(SurfaceKind::Counter | SurfaceKind::Bar)) {
                        *at.entry(t).or_insert(0) += 1;
                    }
                }
            }
            islands += usize::from(at.values().any(|&n| n >= 2));
        }
        tops += usize::from(w.objects.iter().any(|o| w.content.objects[o.def].layer == Layer::Surface));
    }
    eprintln!("{} houses checked, {} problems", houses.len(), failures.len());
    eprintln!("  {dining} with two chairs at a table or more, {tops} with things on tops, {islands} of {large} on large lots with an island and two stools");
    eprintln!("  {bright} of {rooms} rooms lit well (the rest have too few windows for daylight)");
    for f in failures.iter().take(30) {
        eprintln!("  {f}");
    }
    assert!(failures.is_empty());
    assert!(dining * 10 >= houses.len() * 8, "most houses have a table to sit at");
    assert!(tops * 10 >= houses.len() * 9, "most houses have things on their tops");
    assert!(islands * 4 >= large, "a good share of large houses have a kitchen island with stools");
}
