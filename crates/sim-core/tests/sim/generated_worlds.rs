//! Generated worlds (web/src/game/world.ts and town.ts: towns of three sizes, farms, seaside,
//! mountain, lakeside and forest worlds) checked by the simulation: each loads with no more
//! residents than a town holds, its roads make one network (as the sim joins them), every lot can
//! be reached from the ends of every street and back,
//! everything outdoors that residents use (a farm's crops, a park's benches) can be reached, as
//! can the houses' basements from their front doors and everything residents use in the houses
//! (on lots facing either way), and the town keeps going for half a day.
//!
//! Make the worlds first: `bun tools/worlds/sample.ts worlds.json`,
//! then `WORLDS=worlds.json cargo test --release -p sim-core --test sim generated_worlds:: -- --ignored --nocapture`.

use crate::common;
use common::TICKS_PER_HOUR;

use sim_core::World;
use sim_core::lot::{Edge, OUTDOORS};
use sim_core::path::NavGrid;
use sim_core::surface;
use sim_core::travel::Graph;
use sim_core::world::MAX_SIMS;

/// `check` for each of `items`, a few at a time (one per core: each world plays on its own, and a
/// path search's budget counts on its own thread), the results in the items' order.
fn each_world<I: Sync, T: Send>(items: &[I], check: impl Fn(&I) -> T + Sync) -> Vec<T> {
    let threads = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(items.len())
        .max(1);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut out: Vec<(usize, T)> = Vec::with_capacity(items.len());
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(item) = items.get(k) else { break };
                        done.push((k, check(item)));
                    }
                    done
                })
            })
            .collect();
        for worker in workers {
            out.extend(worker.join().expect("a world's check panicked"));
        }
    });
    out.sort_by_key(|&(k, _)| k);
    out.into_iter().map(|(_, t)| t).collect()
}

#[test]
#[ignore]
fn generated_worlds_are_livable() {
    let path = std::env::var("WORLDS")
        .expect("set WORLDS to a file made by `bun tools/worlds/sample.ts worlds.json`");
    let worlds: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let content = common::content();
    let items: Vec<(usize, &serde_json::Value)> = worlds.iter().enumerate().collect();
    let failures: Vec<String> = each_world(&items, |&(i, world)| {
        let mut failures = Vec::new();
        let label = format!("{} {} ({})", world["kind"], world["seed"], world["name"]);
        let mut w = match World::from_json(&content, &world["town"].to_string(), i as u32 + 1) {
            Ok(w) => w,
            Err(e) => {
                failures.push(format!("{label}: {e}"));
                return failures;
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
        // A town with bus stops has a bus calling at them (`bus.rs`).
        let stops = (0..w.exits.len()).filter(|&k| w.travel.is_stop(k)).count();
        if stops > 0 && w.transit.lines.is_empty() {
            failures.push(format!("{label}: {stops} bus stops and no bus"));
        }
        // Half a day of everyone's free will: the bus comes, and nobody waits at a stop for long.
        let mut waited = vec![0u64; w.sims.len()];
        let mut bound: Vec<bool> = (0..w.sims.len()).map(|i| w.bound_for_bus(i).is_some()).collect();
        let (mut most_buses, mut rode) = (0, 0);
        for hour in 0..12 {
            for _ in 0..TICKS_PER_HOUR {
                w.tick_once();
                most_buses = most_buses.max(w.transit.buses.len());
                for (i, t) in waited.iter_mut().enumerate() {
                    let at_stop = w.sims[i].current().is_some_and(|a| {
                        matches!(a.phase, sim_core::world::Phase::AtStop { .. })
                    });
                    if at_stop {
                        *t += 1;
                        if *t == TICKS_PER_HOUR {
                            failures.push(format!(
                                "{label}: {} waited an hour at a bus stop",
                                w.sims[i].name
                            ));
                        }
                    } else {
                        *t = 0;
                    }
                    // (On the bus: bound for it last tick, away now.)
                    rode += usize::from(bound[i] && w.sims[i].away_until.is_some());
                    bound[i] = w.bound_for_bus(i).is_some();
                }
            }
            if hour % 3 == 2 {
                common::check(&w);
            }
        }
        if !w.transit.lines.is_empty() && most_buses == 0 {
            failures.push(format!("{label}: no bus came all day"));
        }
        eprintln!(
            "  {label}: {}×{}, {} residents, {} things, {cellars} rooms in basements, {stops} bus stops, {} lines ({} min out and back at most), {rode} rode the bus",
            w.lot.width,
            w.lot.depth,
            w.sims.len(),
            w.objects.len(),
            w.transit.lines.len(),
            w.transit.lines.iter().map(|l| l.end / sim_core::TICKS_PER_MINUTE).max().unwrap_or(0)
        );
        failures
    })
    .into_iter()
    .flatten()
    .collect();
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

/// Cosy and normal worlds of the file whose cars are driven (each for a day), the first of each
/// size; every big one is (for two days).
const CAR_WORLDS: usize = 99;
/// Hours a resident may be sat in a car standing parked, or bound for one but not on their way, at
/// most (a trip's driver waits up to two hours for the others: `cars::TRIP_WAIT_TICKS`).
const CAR_HOURS: u64 = 4;
/// The most of a size's cars that may have no way onto the road.
const NO_WAY: f32 = 0.1;
/// How far the middle of a car on the move may be off the paving (m): the roads, the driveways
/// and their way across the verge (`meta.paths`), a dead end's turning circle, a parking spot.
/// (Its body is about a metre either side: out of a driveway at a bend it may cut the edge.)
const OFF_PAVING: f32 = 1.5;

/// How far `at` is off the paving (m): from the nearest road's edge, a path (a driveway and its
/// way across the verge) or the parking spot of the car's trip, whichever is nearest. 0 off the
/// map (past a way out of town).
fn off_paving(
    w: &World,
    car: &sim_core::cars::Car,
    at: [f32; 2],
    paths: &[[f32; 4]],
    caps: &[([f32; 2], f32)],
) -> f32 {
    if at[0] < 0.0 || at[1] < 0.0 || at[0] >= w.lot.width as f32 || at[1] >= w.storey_depth() as f32
    {
        return 0.0;
    }
    let g = &w.travel.graph;
    let road = g
        .near(at, 30.0, 1, sim_core::travel::Mode::Car)
        .first()
        .map_or(f32::INFINITY, |p| {
            (p.off - g.roads[p.road as usize].width / 2.0).max(0.0)
        });
    let path = paths
        .iter()
        .map(|&[x, z, pw, pd]| {
            let dx = (x - at[0]).max(at[0] - x - pw).max(0.0);
            let dz = (z - at[1]).max(at[1] - z - pd).max(0.0);
            dx.hypot(dz)
        })
        .fold(f32::INFINITY, f32::min);
    let park = car.trip.map_or(f32::INFINITY, |t| {
        ((t.at[0] - at[0]).hypot(t.at[1] - at[1]) - car.len / 2.0).max(0.0)
    });
    let cap = caps
        .iter()
        .map(|&(c, r)| ((c[0] - at[0]).hypot(c[1] - at[1]) - r).max(0.0))
        .fold(f32::INFINITY, f32::min);
    road.min(path).min(park).min(cap)
}

/// Whether a fence or gate stands where `car`'s doors would swing out (within 1.7 m of its sides,
/// along its length).
fn fenced_in(w: &World, car: &sim_core::cars::Car) -> bool {
    let (s, c) = car.yaw.sin_cos();
    (-4..=4).any(|a| {
        [-1.0f32, 1.0].into_iter().any(|k| {
            (1..=7).any(|o| {
                let along = a as f32 * car.len / 8.0;
                let left = k * (car.wid / 2.0 + o as f32 * 0.25);
                let p = [
                    car.spot[0] + s * along + c * left,
                    car.spot[1] + c * along - s * left,
                ];
                let (x, z) = (p[0].floor() as i32, p[1].floor() as i32);
                // (The edges round the tile: `h_edge(x, z)` is its north side, `v_edge(x, z)` its west.)
                let fence = |e: sim_core::lot::Edge| e.is_fence();
                w.lot.in_bounds(x, z)
                    && w.lot.in_bounds(x + 1, z + 1)
                    && (fence(w.lot.h_edge(x as usize, z as usize))
                        || fence(w.lot.h_edge(x as usize, z as usize + 1))
                        || fence(w.lot.v_edge(x as usize, z as usize))
                        || fence(w.lot.v_edge(x as usize + 1, z as usize)))
            })
        })
    })
}

/// A town's dead ends' turning circles (as `render/shared/roads.ts` paves them, to the sidewalk's
/// outer edge: half the road's width and `TURNING` more): their middles and radii.
fn turning_circles(town: &serde_json::Value) -> Vec<([f32; 2], f32)> {
    const TURNING: f32 = 2.4;
    let pt = |p: &serde_json::Value| {
        [
            p[0].as_f64().unwrap_or(0.0) as f32,
            p[1].as_f64().unwrap_or(0.0) as f32,
        ]
    };
    let mut out = Vec::new();
    for r in town["roads"].as_array().into_iter().flatten() {
        let (Some(points), Some(width)) = (r["points"].as_array(), r["width"].as_f64()) else {
            continue;
        };
        for (k, end) in [points.first(), points.last()].into_iter().enumerate() {
            if let Some(end) = end
                && r["ends"][k].as_str() == Some("dead")
            {
                out.push((pt(end), width as f32 / 2.0 + TURNING));
            }
        }
    }
    out
}

/// The households' cars of generated worlds (`town.ts` writes them where the street drew them,
/// with the road each driveway meets): every one loads parked on its driveway on its lot, nothing
/// built over it, and few stay parked for want of a way onto the road (`NO_WAY`, a driveway that
/// comes alongside its road: `Car::no_way`); driven for a day (two in a big world), the
/// cars on the move keep to the paving (`OFF_PAVING`), some drive to work (and in a big world some
/// go on trips to another hamlet), and at three in the morning every car is parked at home, parked
/// out at a public spot, or away with someone at work in it; nobody waits for a car for hours
/// (`CAR_HOURS`), and in a cosy or normal town nobody is stuck at all (`common::Watch`; a big
/// world's long stretches away or walking are about as many without cars).
#[test]
#[ignore]
fn generated_worlds_cars_keep_to_the_roads() {
    use sim_core::cars::CarState;
    use sim_core::clock;
    let path = std::env::var("WORLDS")
        .expect("set WORLDS to a file made by `bun tools/worlds/sample.ts worlds.json`");
    let worlds: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let content = common::content();
    let trace: Option<(u64, u32)> = std::env::var("CARS_TRACE").ok().and_then(|v| {
        let (seed, plot) = v.split_once(':')?;
        Some((seed.parse().ok()?, plot.parse().ok()?))
    });
    // Per size: worlds driven, cars, cars with no way out, drives to work, trips to another hamlet.
    let mut sizes: std::collections::BTreeMap<String, [usize; 5]> = Default::default();
    // (The first `CAR_WORLDS` of each size, every big one.)
    let mut picked: std::collections::BTreeMap<String, usize> = Default::default();
    let items: Vec<(usize, String, &serde_json::Value)> = worlds
        .iter()
        .enumerate()
        .filter_map(|(i, world)| {
            let size = world["town"]["size"].as_str().unwrap_or("cosy");
            let n = picked.entry(size.to_string()).or_default();
            *n += 1;
            (size == "big" || *n <= CAR_WORLDS).then(|| (i, size.to_string(), world))
        })
        .collect();
    let results = each_world(&items, |(i, size, world)| {
        let (i, size) = (*i, size.clone());
        let mut failures = Vec::new();
        let big = size == "big";
        // (Worlds driven, cars, with no way out, drives to work, trips: summed per size below.)
        let mut tally = [1usize, 0, 0, 0, 0];
        let label = format!(
            "{size} {} {} ({})",
            world["kind"], world["seed"], world["name"]
        );
        // (Seeded by the world's own seed: one picked out of the file runs the same alone.)
        let seed = world["seed"].as_u64().unwrap_or(i as u64) as u32;
        let mut w = match World::from_json(&content, &world["town"].to_string(), seed + 1) {
            Ok(w) => w,
            Err(e) => {
                failures.push(format!("{label}: {e}"));
                return (size, tally, failures);
            }
        };
        let filed = world["town"]["cars"].as_array().map_or(0, Vec::len);
        tally[1] += w.cars.len();
        if w.cars.len() != filed {
            failures.push(format!(
                "{label}: {} cars of the file's {filed}",
                w.cars.len()
            ));
        }
        for car in &w.cars {
            let at = |p: [f32; 2]| format!("{:.1},{:.1}", p[0], p[1]);
            if car.blocked {
                failures.push(format!(
                    "{label}: the car at {} is built over",
                    at(car.spot)
                ));
            }
            // A door with room to open, clear of the house, a garden fence or gate and things:
            // the driver's own or another (`Car::door`). A town's fences keep clear of every
            // driveway's sides (`gardenFences.ts`), so only a house on one side and a tree or a
            // thing next door on the other leave a car none (it stays parked: they walk).
            let nav = NavGrid {
                lot: &w.lot,
                blocked: w.blocked(),
                stairs: w.stair_map(),
            };
            let shut = car.door(&nav, 0).is_none();
            if shut && fenced_in(&w, car) {
                failures.push(format!(
                    "{label}: a fence leaves the car at {} no door with room to open",
                    at(car.spot)
                ));
            }
            tally[2] += usize::from(car.no_way || shut);
            if !w.plots[car.plot as usize]
                .contains(car.spot[0].floor() as i32, car.spot[1].floor() as i32)
            {
                failures.push(format!(
                    "{label}: the car at {} isn't on its lot",
                    at(car.spot)
                ));
            }
        }
        let paths: Vec<[f32; 4]> = w.meta["paths"]
            .as_array()
            .map(|list| {
                list.iter()
                    .map(|r| ["x", "z", "w", "d"].map(|k| r[k].as_f64().unwrap_or(0.0) as f32))
                    .collect()
            })
            .unwrap_or_default();
        let caps = turning_circles(&world["town"]);
        // (A normal town is 256 m across, a big world 1 km: walks across them take hours.)
        let mut watch = common::Watch::new().routing_hours(if size == "cosy" { 4.0 } else { 8.0 });
        let days = if big { 2 } else { 1 };
        let end = clock::tick_at(clock::day(w.tick) + days, 3.0 * 60.0).unwrap();
        let mut was: Vec<CarState> = w.cars.iter().map(|c| c.state).collect();
        // Per resident: since when they've been on their way to a car, or sat in one standing
        // parked (none: neither).
        let mut held: Vec<Option<u64>> = vec![None; w.sims.len()];
        let mut worst = (0.0f32, String::new());
        let started = std::time::Instant::now();
        while w.tick < end {
            w.tick_once();
            watch.sample(&w);
            if w.tick % 4 != 0 {
                continue;
            }
            held.resize(w.sims.len(), None);
            for (i, s) in w.sims.iter().enumerate() {
                // (Sat in it, or bound for it but not walking there, while it stands: not while
                // it drives, nor on a long walk back to it, which `Watch` follows.)
                let walking = s
                    .current()
                    .is_some_and(|a| matches!(a.phase, sim_core::world::Phase::Routing { .. }));
                let standing = w.cars.iter().enumerate().any(|(c, car)| {
                    (car.aboard.contains(&(i as u32))
                        || (s.car_bound() == Some(c as u32) && !walking))
                        && matches!(
                            car.state,
                            CarState::Parked | CarState::Boarding | CarState::Out
                        )
                });
                if !standing {
                    held[i] = None;
                    continue;
                }
                let since = *held[i].get_or_insert(w.tick);
                if w.tick - since == CAR_HOURS * TICKS_PER_HOUR {
                    let car = w.cars.iter().enumerate().find(|(c, car)| {
                        car.aboard.contains(&(i as u32)) || s.car_bound() == Some(*c as u32)
                    });
                    failures.push(format!(
                        "{label}: {} (#{i}) has waited for a car since day {} {:05.2}h ({}, {}, the car {:?})",
                        s.name,
                        clock::day(since),
                        clock::hour(since),
                        format!("{:?}", s.current().map(|a| (&a.task.kind, &a.phase))).chars().take(120).collect::<String>(),
                        if s.car_bound().is_some() { "on their way" } else { "in it" },
                        car.map(|(c, car)| (c, car.plot, car.state, car.trip.is_some(), &car.aboard, car.driver))
                    ));
                }
            }
            for (c, car) in w.cars.iter().enumerate() {
                if was[c] != car.state {
                    if car.state == CarState::Leaving {
                        tally[if car.trip.is_some() { 4 } else { 3 }] += 1;
                    }
                    was[c] = car.state;
                }
                let Some((at, _, true, _)) = car.shown else {
                    continue;
                };
                let off = off_paving(&w, car, at, &paths, &caps);
                // (CARS_TRACE=seed:plot: that car's way, a point every second.)
                if trace == Some((world["seed"].as_u64().unwrap_or(0), car.plot))
                    && w.tick % 20 == 0
                {
                    let (_, yaw, ..) = car.shown.unwrap();
                    eprintln!(
                        "    {:?} {:.1},{:.1} yaw {:.2} off {off:.2}",
                        car.state, at[0], at[1], yaw
                    );
                }
                if off > worst.0 {
                    worst = (
                        off,
                        format!(
                            "the car of plot {} {:?}{} at {:.1},{:.1} day {} {:05.2}h",
                            car.plot,
                            car.state,
                            if car.trip.is_some() { " (a trip)" } else { "" },
                            at[0],
                            at[1],
                            clock::day(w.tick),
                            clock::hour(w.tick)
                        ),
                    );
                }
            }
        }
        if worst.0 > OFF_PAVING {
            failures.push(format!(
                "{label}: {} is {:.2} m off the paving",
                worst.1, worst.0
            ));
        }
        // At three in the morning: parked at home, parked out, or away at work with someone; or
        // on its way (a drive across a big world takes game hours, and it gives way to everyone
        // in front of it for a few minutes each, a crowd at the way out of town too: one that
        // has stood for two hours of it, or someone getting in for longer than a trip waits, is
        // stuck).
        for car in &w.cars {
            let home = w.households[car.household as usize].plot;
            let since = w.tick.saturating_sub(car.since);
            let wrong = match car.state {
                CarState::Parked => home.is_some_and(|h| h != car.plot),
                CarState::Out => !car.trip.is_some_and(|t| {
                    w.travel
                        .file
                        .parking
                        .iter()
                        .any(|p| (p.x - t.at[0]).hypot(p.z - t.at[1]) < 0.1)
                }),
                CarState::Away => car.aboard.is_empty(),
                CarState::Boarding => since > CAR_HOURS * TICKS_PER_HOUR,
                CarState::Leaving | CarState::Arriving => {
                    since as f32 - car.progress > 2.0 * TICKS_PER_HOUR as f32
                }
            };
            if wrong {
                failures.push(format!(
                    "{label}: the car of plot {} is {:?} at three in the morning (since day {} {:05.2}h)",
                    car.plot,
                    car.state,
                    clock::day(car.since),
                    clock::hour(car.since)
                ));
            }
        }
        // (A big world's residents away or walking for long are about as many with cars as
        // without: only those waiting for a car count there, above.)
        if !big {
            for s in &watch.stuck {
                failures.push(format!("{label}: {s}"));
            }
        }
        eprintln!(
            "  {label}: {} cars, worst {:.2} m off the paving, {} long stretches, {:.1} s",
            w.cars.len(),
            worst.0,
            watch.stuck.len(),
            started.elapsed().as_secs_f32()
        );
        (size, tally, failures)
    });
    let mut failures = Vec::new();
    for (size, tally, mut found) in results {
        let sum = sizes.entry(size).or_default();
        for (s, t) in sum.iter_mut().zip(tally) {
            *s += t;
        }
        failures.append(&mut found);
    }
    for (size, [n, cars, stay, commutes, trips]) in &sizes {
        eprintln!(
            "{size}: {n} worlds, {cars} cars ({stay} with no way out or no door to open), {commutes} drives to work, {trips} trips"
        );
        if *cars == 0 || *stay as f32 > *cars as f32 * NO_WAY {
            failures.push(format!(
                "{size}: {stay} of {cars} cars have no way out by road"
            ));
        }
        if *n > 0 && *commutes == 0 {
            failures.push(format!("{size}: nobody drove to work"));
        }
        if size == "big" && *n > 0 && *trips == 0 {
            failures.push(format!("{size}: nobody drove to another hamlet"));
        }
    }
    eprintln!("{} problems", failures.len());
    for f in failures.iter().take(400) {
        eprintln!("  {f}");
    }
    assert!(failures.is_empty());
}
