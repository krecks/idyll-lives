//! Generated worlds (web/src/game/world.ts and town.ts: towns of three sizes, farms, seaside,
//! mountain, lakeside and forest worlds) checked by the simulation: each loads with no more
//! residents than a town holds, every lot can be reached from the ends of every street and back,
//! everything outdoors that residents use (a farm's crops, a park's benches) can be reached, as
//! can the houses' basements from their front doors and everything residents use in the houses
//! (on lots facing either way), and the town keeps going for half a day.
//!
//! Make the worlds first: `bun tools/worlds/sample.ts worlds.json`,
//! then `WORLDS=worlds.json cargo test --release -p sim-core --test sim generated_worlds:: -- --ignored --nocapture`.

use crate::common;

use sim_core::lot::OUTDOORS;
use sim_core::path::NavGrid;
use sim_core::surface;
use sim_core::world::MAX_SIMS;
use sim_core::{MINUTES_PER_TICK, World};

const TICKS_PER_HOUR: u64 = (60.0 / MINUTES_PER_TICK) as u64;

#[test]
#[ignore]
fn generated_worlds_are_livable() {
    let path = std::env::var("WORLDS")
        .expect("set WORLDS to a file made by `bun tools/worlds/sample.ts worlds.json`");
    let worlds: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
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
            failures.push(format!(
                "{label}: {} residents, more than {MAX_SIMS}",
                w.sims.len()
            ));
        }
        if w.exits.is_empty() {
            failures.push(format!("{label}: no way out of town"));
        }
        let mut cellars = 0;
        {
            let nav = NavGrid {
                lot: &w.lot,
                blocked: w.blocked(),
                stairs: w.stair_map(),
            };
            let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
            let exits: Vec<_> = w.exits.iter().map(|&e| tile(e)).collect();
            // Every lot from every way into town.
            for plot in &w.plots {
                let entry = tile(plot.arrival_point());
                for &exit in &exits {
                    if nav.find_path(exit, entry).is_none() {
                        failures.push(format!(
                            "{label}: {} can't be reached from {exit:?}",
                            plot.name
                        ));
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
                if !surface::use_tiles(&w.content, &w.objects, &nav, o)
                    .into_iter()
                    .any(|t| nav.find_path(start, t).is_some())
                {
                    failures.push(format!(
                        "{label}: can't reach the {} at {},{}",
                        def.id, o.x, o.z
                    ));
                }
            }
            // Every room in the basements, from the front door of its house; and everything residents
            // use in the houses (upstairs, downstairs and in the basement), on lots facing either way.
            let door = |x: i32, z: i32| {
                w.plot_at(x, z)
                    .map(|p| tile(w.plots[p as usize].arrival_point()))
            };
            let down = w.storey_row(-1);
            let mut seen = std::collections::HashSet::new();
            for z in down..down + w.storey_depth() {
                for x in 0..w.lot.width as i32 {
                    let room = w.lot.room_at(x, z);
                    if room == OUTDOORS || !nav.tile_free(x, z) || !seen.insert(room) {
                        continue;
                    }
                    let Some(from) = door(x, z) else { continue };
                    cellars += 1;
                    let walked = (z..down + w.storey_depth())
                        .flat_map(|tz| (0..w.lot.width as i32).map(move |tx| (tx, tz)));
                    if !walked
                        .filter(|&(tx, tz)| w.lot.room_at(tx, tz) == room && nav.tile_free(tx, tz))
                        .take(8)
                        .any(|t| nav.find_path(from, t).is_some())
                    {
                        failures.push(format!(
                            "{label}: can't get into the basement room at {x},{z}"
                        ));
                    }
                }
            }
            for o in w.objects.iter().filter(|o| {
                !w.content.objects[o.def].interactions.is_empty()
                    && w.lot.room_at(o.x, o.z) != OUTDOORS
            }) {
                let Some(from) = door(o.x, o.z) else { continue };
                if !surface::use_tiles(&w.content, &w.objects, &nav, o)
                    .into_iter()
                    .any(|t| nav.find_path(from, t).is_some())
                {
                    let at = match w.storey_of(o.z) {
                        -1 => "in the basement",
                        0 => "downstairs",
                        _ => "upstairs",
                    };
                    failures.push(format!(
                        "{label}: can't reach the {} {at} at {},{}",
                        w.content.objects[o.def].id, o.x, o.z
                    ));
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
        eprintln!(
            "  {label}: {}×{}, {} residents, {} things, {cellars} rooms in basements",
            w.lot.width,
            w.lot.depth,
            w.sims.len(),
            w.objects.len()
        );
    }
    eprintln!(
        "{} worlds checked, {} problems",
        worlds.len(),
        failures.len()
    );
    for f in failures.iter().take(30) {
        eprintln!("  {f}");
    }
    assert!(failures.is_empty());
}
