//! Generated houses (web/src/game/housegen.ts) checked by the simulation: each loads, has a
//! bed, a fridge and a toilet in rooms of their own, and everything in it can be reached from the
//! front door, upstairs too. Chairs face tables, things on tops stand on one (never on a hob or
//! a sink) and curtains hang over a window; most houses have somewhere to sit down at a table and
//! small things on their tops. No room is bigger than its kind may be (`maxSide` in housegen.json), each is a room of its own in the simulation, and a
//! good share of houses on large lots have a kitchen island with stools pulled up to it. Every
//! room has lights enough for all of it at night (no more than four from its ceiling) and is lit well (windows enough by day where
//! it has an outside wall), every room can be walked into, and nothing stands just inside a
//! door. A good share of houses on medium and large lots have a basement: under the house, no
//! windows, its rooms lit and reached by stairs down that come up inside a living room, an
//! entrance hall, a dining room or a kitchen. A home with a washer has somewhere to dry the
//! washing: a dryer, or a washing line in the garden that can be reached. The houses come turned
//! to face a street on each side in turn, as towns lay their lots.
//!
//! With `DUMP_STATES=1`, the first house with a basement is written to `debug-states/` as
//! `generated-basement` (with a resident at the door), to open in the game with
//! `?state=generated-basement`.
//!
//! Make the houses first: `bun tools/housegen/sample.ts houses.json`,
//! then `HOUSES=houses.json cargo test -p sim-core --test sim generated_houses:: -- --ignored --nocapture`.

use crate::common;

use sim_core::World;
use sim_core::content::{Layer, SurfaceKind, WorkOn};
use sim_core::lot::{Edge, OUTDOORS};
use sim_core::path::NavGrid;
use sim_core::surface;

/// A hob or a grill (stations `stove`, `grill`) or a sink (`dishes`, where hands are washed):
/// what housegen keeps small things off.
fn cooks_or_washes(
    content: &sim_core::content::Content,
    def: &sim_core::content::ObjectDef,
) -> bool {
    let station = |s: &str| {
        content
            .cooking
            .station(s)
            .is_some_and(|m| def.stations & m != 0)
    };
    let handwash = content
        .tags
        .iter()
        .position(|t| t == "handwash")
        .map_or(0, |i| 1 << i);
    station("stove")
        || station("grill")
        || station("dishes") && def.interactions.iter().any(|i| i.tags & handwash != 0)
}

#[test]
#[ignore]
fn generated_houses_are_livable() {
    let path = std::env::var("HOUSES")
        .expect("set HOUSES to a file made by `bun tools/housegen/sample.ts houses.json`");
    let houses: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let content = common::content();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/content");
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("housegen.json")).unwrap()).unwrap();
    let mut failures = Vec::new();
    let (mut dining, mut tops) = (0, 0);
    let (mut large, mut islands) = (0, 0);
    let (mut rooms, mut bright) = (0, 0);
    let (mut lights, mut most_lights) = (0, 0);
    let mut curtains = 0;
    let (mut bigger, mut basements, mut below) = (0, 0, 0);
    let mut dumped = false;
    let (mut washers, mut dryers, mut lines) = (0, 0, 0);
    for h in &houses {
        let label = format!(
            "{} {} facing {} ({})",
            h["size"], h["seed"], h["facing"], h["description"]
        );
        let w = match World::from_json(&content, &h["town"].to_string(), 1) {
            Ok(w) => w,
            Err(e) => {
                failures.push(format!("{label}: {e}"));
                continue;
            }
        };
        // Things on tops stand on one listed before them, never on a hob or a sink.
        common::check(&w);
        for o in w
            .objects
            .iter()
            .filter(|o| w.content.objects[o.def].layer == Layer::Surface)
        {
            let host = surface::host_of(&w.content, &w.objects, o)
                .map(|h| &w.content.objects[w.objects[h as usize].def]);
            if let Some(host) = host.filter(|d| cooks_or_washes(&w.content, d)) {
                failures.push(format!(
                    "{label}: the {} at {},{} is on the {}",
                    w.content.objects[o.def].id, o.x, o.z, host.id
                ));
            }
        }
        // Curtains (`overWindow`) hang over a window, never on a plain wall.
        for o in w
            .objects
            .iter()
            .filter(|o| w.content.objects[o.def].over_window)
        {
            let (ow, od) = o.size(&w.content);
            let back: Vec<Edge> = match o.rot % 4 {
                0 => (o.x..o.x + ow)
                    .map(|x| w.lot.h_edge(x as usize, o.z as usize))
                    .collect(),
                2 => (o.x..o.x + ow)
                    .map(|x| w.lot.h_edge(x as usize, (o.z + od) as usize))
                    .collect(),
                1 => (o.z..o.z + od)
                    .map(|z| w.lot.v_edge(o.x as usize, z as usize))
                    .collect(),
                _ => (o.z..o.z + od)
                    .map(|z| w.lot.v_edge((o.x + ow) as usize, z as usize))
                    .collect(),
            };
            curtains += 1;
            if !back.contains(&Edge::Window) {
                failures.push(format!(
                    "{label}: the {} at {},{} hangs on a wall with no window",
                    w.content.objects[o.def].id, o.x, o.z
                ));
            }
        }
        // Rooms as planned: no bigger than their kind may be, walled off (or open to the next).
        for r in h["rooms"].as_array().into_iter().flatten() {
            let kind = r["kind"].as_str().unwrap();
            let [x, z, rw, rd] = ["x", "z", "w", "d"].map(|k| r[k].as_i64().unwrap() as i32);
            if let Some(cap) = config["rooms"][kind]["maxSide"].as_array() {
                let (short, long) = (
                    cap[0].as_i64().unwrap() as i32,
                    cap[1].as_i64().unwrap() as i32,
                );
                if rw.min(rd) > short || rw.max(rd) > long {
                    failures.push(format!(
                        "{label}: a {kind} of {rw}×{rd}, more than {short}×{long}"
                    ));
                }
            }
            let id = w.lot.room_at(x, z);
            let whole = (z..z + rd).all(|tz| (x..x + rw).all(|tx| w.lot.room_at(tx, tz) == id));
            if id == OUTDOORS || !whole {
                failures.push(format!(
                    "{label}: the {kind} at {x},{z} isn't a room of its own"
                ));
            }
        }
        let room_of = |is: &dyn Fn(&sim_core::content::ObjectDef) -> bool| {
            w.objects
                .iter()
                .find(|o| is(&w.content.objects[o.def]))
                .map(|o| w.lot.room_at(o.x, o.z))
        };
        let bed = room_of(&|d| d.interactions.iter().any(|i| i.id == "sleep"));
        let (fridge, toilet) = (
            room_of(&|d| d.id == "fridge"),
            room_of(&|d| d.id == "toilet"),
        );
        for (what, room) in [("bed", bed), ("fridge", fridge), ("toilet", toilet)] {
            if room.is_none_or(|r| r == OUTDOORS) {
                failures.push(format!("{label}: no {what} indoors"));
            }
        }
        // Dinner can be made (a meal needs only somewhere to get ingredients: the rest of its
        // steps are done without when there's no counter or stove).
        let stations = w
            .objects
            .iter()
            .fold(0, |m, o| m | w.content.objects[o.def].stations);
        let meal = w
            .content
            .recipe_index("meal")
            .map(|r| &w.content.recipes[r]);
        if meal.is_some_and(|r| !r.possible(stations)) {
            failures.push(format!("{label}: can't make a meal"));
        }
        if bed.is_some() && (bed == toilet || bed == fridge) || fridge == toilet {
            failures.push(format!(
                "{label}: bedroom, kitchen and bathroom share a room"
            ));
        }
        // Everything reachable from the street.
        let entry = w.plots[0].arrival_point();
        let nav = NavGrid {
            lot: &w.lot,
            blocked: w.blocked(),
            stairs: w.stair_map(),
        };
        let start = (entry[0].floor() as i32, entry[1].floor() as i32);
        let mut seated = 0;
        for o in &w.objects {
            let def = &w.content.objects[o.def];
            // Chairs are put round tables, facing them.
            if def.seat_at != 0 {
                if surface::faced_table(&w.content, &w.objects, &nav, o.id).is_some() {
                    seated += 1;
                } else {
                    failures.push(format!(
                        "{label}: the {} at {},{} faces no table",
                        def.id, o.x, o.z
                    ));
                }
            }
            // Pictures, ceiling lamps, rugs and tables nobody uses needn't be reached; chairs at
            // a table from any free side.
            if def.interactions.is_empty() {
                continue;
            }
            let reached = surface::use_tiles(&w.content, &w.objects, &nav, o)
                .into_iter()
                .any(|t| nav.find_path(start, t).is_some());
            if !reached {
                failures.push(format!(
                    "{label}: can't reach the {} at {},{}",
                    def.id, o.x, o.z
                ));
            }
        }
        // Every room can be walked into from the front door (upstairs and in the basement too).
        for r in h["rooms"].as_array().into_iter().flatten() {
            let kind = r["kind"].as_str().unwrap();
            let [x, z, rw, rd] = ["x", "z", "w", "d"].map(|k| r[k].as_i64().unwrap() as i32);
            let mut tiles = (z..z + rd).flat_map(|tz| (x..x + rw).map(move |tx| (tx, tz)));
            if !tiles.any(|t| nav.tile_free(t.0, t.1) && nav.find_path(start, t).is_some()) {
                failures.push(format!("{label}: can't get into the {kind} at {x},{z}"));
            }
        }
        // The basement: under the house, underground (no windows), stairs down from a room
        // everyone uses, its rooms lit.
        bigger += usize::from(h["size"] != "small");
        // (The shipped content gives every lot a basement storey; the house says whether it has one.)
        if h["town"]["basement"] == true {
            basements += 1;
            let (down, depth, p) = (w.storey_row(-1), w.storey_depth(), &w.plots[0]);
            let (lw, ld) = (w.lot.width as i32, w.lot.depth as i32);
            let on_plot = |x: i32, z: i32| {
                x >= p.x && x <= p.x + p.w && z >= down + p.z && z <= down + p.z + p.d
            };
            for z in down..=(down + depth).min(ld) {
                for x in 0..=lw {
                    let h_edge = if x < lw {
                        w.lot.h_edge(x as usize, z as usize)
                    } else {
                        Edge::Open
                    };
                    let v_edge = if z < ld && z < down + depth {
                        w.lot.v_edge(x as usize, z as usize)
                    } else {
                        Edge::Open
                    };
                    for (edge, ex, ez) in [(h_edge, x, z), (v_edge, x, z)] {
                        if matches!(edge, Edge::Window | Edge::Fence | Edge::Gate) {
                            failures
                                .push(format!("{label}: a {edge:?} in the basement at {ex},{ez}"));
                        } else if edge != Edge::Open && !on_plot(ex, ez) {
                            failures.push(format!(
                                "{label}: a basement wall off the plot at {ex},{ez}"
                            ));
                        }
                    }
                }
            }
            let planned = |x: i32, z: i32| {
                h["rooms"].as_array().into_iter().flatten().find(|r| {
                    let [rx, rz, rw, rd] =
                        ["x", "z", "w", "d"].map(|k| r[k].as_i64().unwrap() as i32);
                    (rx..rx + rw).contains(&x) && (rz..rz + rd).contains(&z)
                })
            };
            let flights: Vec<_> = w
                .stair_map()
                .stairs
                .iter()
                .filter(|s| w.storey_of(s.bottom.1) == -1)
                .collect();
            if flights.is_empty() {
                failures.push(format!("{label}: a basement with no stairs down"));
            }
            for s in flights {
                // The stairwell (over the steps) and the landing in one room on the ground floor.
                let (lx, lz) = s.landing;
                let room = planned(lx, lz);
                let kind = room.and_then(|r| r["kind"].as_str()).unwrap_or("garden");
                let well = (0..s.len).map(|i| {
                    (
                        s.bottom.0 + s.dir.0 * i,
                        w.ground_z(s.bottom.1 + s.dir.1 * i),
                    )
                });
                if !["living", "entry", "dining", "kitchen"].contains(&kind) || w.storey_of(lz) != 0
                {
                    failures.push(format!(
                        "{label}: the stairs down come up in the {kind} at {lx},{lz}"
                    ));
                } else if !well.clone().all(|(x, z)| planned(x, z) == room) {
                    failures.push(format!(
                        "{label}: the stairwell at {:?} isn't all in the {kind}",
                        well.collect::<Vec<_>>()
                    ));
                }
            }
            for r in h["rooms"].as_array().into_iter().flatten() {
                let [x, z] = ["x", "z"].map(|k| r[k].as_i64().unwrap() as i32);
                below += usize::from(w.storey_of(z) == -1 && w.room_at_tile(x, z).is_some());
            }
            if !dumped && std::env::var_os("DUMP_STATES").is_some() {
                // With someone living there, standing at the door.
                let mut town = h["town"].clone();
                town["sims"] = serde_json::json!([{ "name": "Ada", "x": entry[0], "z": entry[1], "traits": ["foodie"] }]);
                let lived = World::from_json(&content, &town.to_string(), 1).unwrap();
                common::dump_state(&lived, "generated-basement");
                dumped = true;
            }
        }
        // Every room has a light, enough for all of it at night (an open kitchen and living room
        // share theirs), and windows enough by day that the room overlay doesn't call it dim. A
        // few bigger lights do it, not a ceiling full of small ones.
        for r in h["rooms"].as_array().into_iter().flatten() {
            let kind = r["kind"].as_str().unwrap();
            let [x, z, rw, rd] = ["x", "z", "w", "d"].map(|k| r[k].as_i64().unwrap() as i32);
            let Some(room) = w.room_at_tile(x, z) else {
                continue;
            };
            rooms += 1;
            bright += usize::from(room.scores.light >= 0.6);
            let lamps = w.objects.iter().filter(|o| {
                w.content.objects[o.def].lamp > 0.0
                    && (x..x + rw).contains(&o.x)
                    && (z..z + rd).contains(&o.z)
            });
            let hanging = lamps
                .clone()
                .filter(|o| w.content.objects[o.def].layer == Layer::Ceiling)
                .count();
            lights += lamps.count();
            most_lights = most_lights.max(hanging);
            if room.lamps == 0 {
                failures.push(format!("{label}: the {kind} at {x},{z} has no light"));
            } else if room.lit < room.tiles {
                failures.push(format!(
                    "{label}: the {kind} at {x},{z} has lights for {} of its {} tiles",
                    room.lit, room.tiles
                ));
            } else if hanging > 4 {
                failures.push(format!(
                    "{label}: the {kind} at {x},{z} has {hanging} ceiling lights"
                ));
            } else if room.scores.light < 0.6 {
                failures.push(format!(
                    "{label}: the {kind} at {x},{z} is dim ({} windows for {} tiles)",
                    room.windows, room.tiles
                ));
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
                                !w.content.objects[o.def].layer.mounted()
                                    && (o.x..o.x + ow).contains(&tx)
                                    && (o.z..o.z + od).contains(&tz)
                            });
                            let id =
                                what.map_or("something", |o| w.content.objects[o.def].id.as_str());
                            failures.push(format!(
                                "{label}: the {id} at {tx},{tz} stands in a doorway"
                            ));
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
                    let kind = w.content.objects[w.objects[t as usize].def]
                        .surface
                        .map(|s| s.kind);
                    if matches!(kind, Some(SurfaceKind::Counter | SurfaceKind::Bar)) {
                        *at.entry(t).or_insert(0) += 1;
                    }
                }
            }
            islands += usize::from(at.values().any(|&n| n >= 2));
        }
        tops += usize::from(
            w.objects
                .iter()
                .any(|o| w.content.objects[o.def].layer == Layer::Surface),
        );
        // Washing washed has somewhere to dry: something that works on a washed load (a dryer, a
        // washing line outdoors; reached, as everything is, above).
        let uses =
            |o: &sim_core::world::ObjectInstance| w.content.objects[o.def].interactions.iter();
        if w.objects
            .iter()
            .any(|o| uses(o).any(|i| i.makes == Some(WorkOn::Washed)))
        {
            washers += 1;
            let dries: Vec<_> = w
                .objects
                .iter()
                .filter(|o| uses(o).any(|i| i.work_on == Some(WorkOn::Washed)))
                .collect();
            dryers += usize::from(dries.iter().any(|o| !w.content.objects[o.def].outdoors));
            lines += usize::from(dries.iter().any(|o| w.content.objects[o.def].outdoors));
            if dries.is_empty() {
                failures.push(format!("{label}: a washer and nowhere to dry the washing"));
            }
        }
    }
    eprintln!(
        "{} houses checked, {} problems",
        houses.len(),
        failures.len()
    );
    eprintln!(
        "  {dining} with two chairs at a table or more, {tops} with things on tops, {islands} of {large} on large lots with an island and two stools"
    );
    eprintln!(
        "  {bright} of {rooms} rooms lit well, {:.1} lamps a room, at most {most_lights} from one room's ceiling",
        lights as f32 / rooms.max(1) as f32
    );
    eprintln!("  {curtains} curtains");
    eprintln!("  {washers} with a washer: {dryers} with a dryer, {lines} with a washing line");
    eprintln!(
        "  {basements} of {bigger} houses on medium and large lots with a basement, {below} rooms down there (lit by lamps alone)"
    );
    for f in failures.iter().take(30) {
        eprintln!("  {f}");
    }
    assert!(failures.is_empty());
    assert!(
        dining * 10 >= houses.len() * 8,
        "most houses have a table to sit at"
    );
    assert!(
        tops * 10 >= houses.len() * 9,
        "most houses have things on their tops"
    );
    // Over half do (30 to 43 of 60 in samples of other seeds); a third leaves room for reshuffles.
    assert!(
        islands * 3 >= large,
        "a good share of large houses have a kitchen island with stools"
    );
    assert!(
        basements * 5 >= bigger,
        "a good share of houses on medium and large lots have a basement"
    );
}
