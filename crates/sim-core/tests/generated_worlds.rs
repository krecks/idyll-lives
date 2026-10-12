//! Generated worlds (web/src/game/world.ts and town.ts: towns of three sizes, farms, seaside,
//! mountain, lakeside and forest worlds) checked by the simulation: each loads with no more
//! residents than a town holds, every lot can be reached from the ends of every street and back,
//! everything outdoors that residents use (a farm's crops, a park's benches) can be reached, and
//! the town keeps going for half a day.
//!
//! Make the worlds first: `node --experimental-transform-types tools/worlds/sample.ts worlds.json`,
//! then `WORLDS=worlds.json cargo test --release -p sim-core --test generated_worlds -- --ignored --nocapture`.

mod common;

use sim_core::lot::OUTDOORS;
use sim_core::path::NavGrid;
use sim_core::surface;
use sim_core::world::MAX_SIMS;
use sim_core::{MINUTES_PER_TICK, World};

const TICKS_PER_HOUR: u64 = (60.0 / MINUTES_PER_TICK) as u64;

#[test]
#[ignore]
fn generated_worlds_are_livable() {
    let Ok(path) = std::env::var("WORLDS") else {
        eprintln!("set WORLDS to a file made by tools/worlds/sample.ts");
        return;
    };
    let worlds: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let content = common::content();
    let mut failures = Vec::new();
    for (i, world) in worlds.iter().enumerate() {
        let label = format!("{} {} ({})", world["kind"], world["seed"], world["name"]);
        let mut w = match World::from_json(&content, &world["town"].to_string(), i as u32 + 1) {
            Ok(w) => w,
            Err(e) => {
                failures.push(format!("{label}: {e}"));
                continue;
            }
        };
        if w.sims.len() > MAX_SIMS {
            failures.push(format!("{label}: {} residents, more than {MAX_SIMS}", w.sims.len()));
        }
        if w.exits.is_empty() {
            failures.push(format!("{label}: no way out of town"));
        }
        {
            let nav = NavGrid { lot: &w.lot, blocked: w.blocked(), stairs: w.stair_map() };
            let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
            let exits: Vec<_> = w.exits.iter().map(|&e| tile(e)).collect();
            // Every lot from every way into town.
            for plot in &w.plots {
                let entry = tile(plot.arrival_point());
                for &exit in &exits {
                    if nav.find_path(exit, entry).is_none() {
                        failures.push(format!("{label}: {} can't be reached from {exit:?}", plot.name));
                    }
                }
            }
            // Everything outdoors that residents use.
            let start = exits[0];
            for o in &w.objects {
                let def = &w.content.objects[o.def];
                if def.interactions.is_empty() || w.lot.room_at(o.x, o.z) != OUTDOORS {
                    continue;
                }
                if !surface::approach_tiles(&w.content, &nav, o).any(|t| nav.find_path(start, t).is_some()) {
                    failures.push(format!("{label}: can't reach the {} at {},{}", def.id, o.x, o.z));
                }
            }
        }
        common::check(&w);
        // Half a day of everyone's free will.
        for hour in 0..12 {
            for _ in 0..TICKS_PER_HOUR {
                w.tick_once();
            }
            if hour % 3 == 2 {
                common::check(&w);
            }
        }
        eprintln!("  {label}: {}×{}, {} residents, {} things", w.lot.width, w.lot.depth, w.sims.len(), w.objects.len());
    }
    eprintln!("{} worlds checked, {} problems", worlds.len(), failures.len());
    for f in failures.iter().take(30) {
        eprintln!("  {f}");
    }
    assert!(failures.is_empty());
}
