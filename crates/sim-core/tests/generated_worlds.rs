//! Generated worlds (web/src/game/world.ts and town.ts: towns of three sizes, farms, seaside,
//! mountain, lakeside and forest worlds) checked by the simulation: each loads with no more
//! residents than a town holds, its roads make one network (as the sim joins them), every lot can
//! be reached from the ends of every street and back,
//! everything outdoors that residents use (a farm's crops, a park's benches) can be reached, as
//! can the houses' basements from their front doors and everything residents use in the houses
//! (on lots facing either way), and the town keeps going for half a day.
//!
//! Make the worlds first: `bun tools/worlds/sample.ts worlds.json`,
//! then `WORLDS=worlds.json cargo test --release -p sim-core --test generated_worlds -- --ignored --nocapture`.

mod common;

use sim_core::lot::{Edge, OUTDOORS};
use sim_core::path::NavGrid;
use sim_core::surface;
use sim_core::travel::Graph;
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
        // The town's roads one network as the sim joins them (built here for a cosy town too,
        // which plays without one).
        let networks = Graph::from_town(&w.travel.file, &world["town"]["meta"]).networks();
        if networks != 1 {
            failures.push(format!(
                "{label}: the roads make {networks} networks, not one"
            ));
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
            // No garden shut off by the town's fences: every tile of the ground walked to from the
            // way into town with the fences gone is walked to with them standing.
            let reach = |through_fences: bool| {
                let (wd, dd) = (w.lot.width as i32, w.storey_depth());
                let mut seen = vec![false; (wd * dd) as usize];
                let (sx, sz) = exits[0];
                let mut stack = vec![(sx, sz)];
                while let Some((x, z)) = stack.pop() {
                    if x < 0 || z < 0 || x >= wd || z >= dd || seen[(z * wd + x) as usize] {
                        continue;
                    }
                    seen[(z * wd + x) as usize] = true;
                    for (nx, nz) in [(x + 1, z), (x - 1, z), (x, z + 1), (x, z - 1)] {
                        if nx < 0 || nz < 0 || nx >= wd || nz >= dd || !nav.tile_free(nx, nz) {
                            continue;
                        }
                        let edge = if nx != x {
                            w.lot.v_edge(x.max(nx) as usize, z as usize)
                        } else {
                            w.lot.h_edge(x as usize, z.max(nz) as usize)
                        };
                        if !edge.blocks() || (through_fences && edge == Edge::Fence) {
                            stack.push((nx, nz));
                        }
                    }
                }
                seen
            };
            let (fenced, open) = (reach(false), reach(true));
            let mut shut = std::collections::BTreeMap::<&str, (usize, (i32, i32))>::new();
            for (i, (&f, &o)) in fenced.iter().zip(&open).enumerate() {
                let (x, z) = (i as i32 % w.lot.width as i32, i as i32 / w.lot.width as i32);
                if o && !f && w.lot.room_at(x, z) == OUTDOORS {
                    let name = w
                        .plot_at(x, z)
                        .map_or("the street", |p| &w.plots[p as usize].name);
                    shut.entry(name).or_insert((0, (x, z))).0 += 1;
                }
            }
            for (name, (n, at)) in shut {
                failures.push(format!(
                    "{label}: {n} tiles of {name} shut off by fences (one at {at:?})"
                ));
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
