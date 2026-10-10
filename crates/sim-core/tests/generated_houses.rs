//! Generated houses (web/src/game/housegen.ts) checked by the simulation: each loads, has a
//! bed, a fridge and a toilet in rooms of their own, and everything in it can be reached from the
//! front door, upstairs too. Chairs face tables and things on tops stand on one; most houses have
//! somewhere to sit down at a table and small things on their tops.
//!
//! Make the houses first: `node --experimental-strip-types tools/housegen/sample.ts houses.json`,
//! then `HOUSES=houses.json cargo test -p sim-core --test generated_houses -- --ignored --nocapture`.

mod common;

use sim_core::World;
use sim_core::content::Layer;
use sim_core::lot::OUTDOORS;
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
    let mut failures = Vec::new();
    let (mut dining, mut tops) = (0, 0);
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
        dining += usize::from(seated >= 2);
        tops += usize::from(w.objects.iter().any(|o| w.content.objects[o.def].layer == Layer::Surface));
    }
    eprintln!("{} houses checked, {} problems", houses.len(), failures.len());
    eprintln!("  {dining} with two chairs at a table or more, {tops} with things on tops");
    for f in failures.iter().take(30) {
        eprintln!("  {f}");
    }
    assert!(failures.is_empty());
    assert!(dining * 10 >= houses.len() * 8, "most houses have a table to sit at");
    assert!(tops * 10 >= houses.len() * 9, "most houses have things on their tops");
}
