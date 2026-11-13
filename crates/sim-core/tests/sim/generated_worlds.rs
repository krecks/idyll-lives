//! Generated worlds (web/src/game/world.ts and town.ts: farms, seaside, mountain, lakeside and
//! forest worlds, every one a big one: cosy and normal were dropped in 0.8.0) checked by the
//! simulation: each loads with no more residents than a town holds, its roads make one network (as
//! the sim joins them), every lot can be reached from the ends of every street and back,
//! everything outdoors that residents use (a farm's crops, a park's benches) can be reached, as
//! can the houses' basements from their front doors and everything residents use in the houses
//! (on lots facing either way), and the town keeps going for half a day: in a town with bikes at
//! homes without a car someone rides one to work or school, and nobody is stuck on one in town; no
//! two cars stand or drive on top of each other, nor is one held up for good.
//!
//! A world is looked at as its game starts, the camera on the player's home: what the renderer
//! streams in round it is simulated in full detail and drawn (`common::watch_home`). Out of that
//! and a margin round it cars drive through each other (`cars::sight`: the owner's call,
//! off-camera traffic simplified), so two cars on top of each other or passing too near count only
//! where one of them is drawn (`common::drawn`); a car held up for good counts anywhere.
//!
//! Make the worlds first: `bun tools/worlds/sample.ts worlds.json`,
//! then `WORLDS=worlds.json cargo test --release -p sim-core --test sim generated_worlds:: -- --ignored --nocapture`.

use crate::common;
use common::TICKS_PER_HOUR;

use sim_core::World;
use sim_core::lot::{Edge, OUTDOORS};
use sim_core::path::{NavGrid, Reach};
use sim_core::surface;
use sim_core::travel::Graph;
use sim_core::world::{MAX_SIMS, Plot, TaskKind};

/// `check` for each of `items`, a few at a time (one per core, or `WORLD_THREADS` at most: each
/// world plays on its own, and a path search's budget counts on its own thread), the results in
/// the items' order.
fn each_world<I: Sync, T: Send>(items: &[I], check: impl Fn(&I) -> T + Sync) -> Vec<T> {
    let threads = std::env::var("WORLD_THREADS")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()))
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

/// Where a resident can walk from a tile: across the lot by its walkable regions (`path::Reach`,
/// labelled once), and round a plot by one flood from each place asked about, kept for the next
/// question. (`NavGrid::find_path` apiece would walk the lot again for each of a town's hundreds
/// of things, a minute a world where this takes a second.)
struct Walks<'a> {
    nav: &'a NavGrid<'a>,
    reach: Reach,
    /// The tiles (`Lot::tile_index`) walked to round a plot, by where the walk started and the
    /// ground it may go round in: a few hundred each, not a whole lot's worth per house.
    seen: std::collections::HashMap<(Walk, Rect), std::collections::HashSet<usize>>,
}

/// A tile a walk starts at, and the ground (x and z, the tile past each) it may go round in.
type Walk = (i32, i32);
type Rect = (i32, i32, i32, i32);

impl<'a> Walks<'a> {
    fn new(nav: &'a NavGrid<'a>) -> Self {
        Self {
            nav,
            reach: Reach::build(nav),
            seen: Default::default(),
        }
    }

    /// Whether `to` is a tile to stand on (as `find_path` wants of a goal).
    fn stands(&self, to: Walk) -> bool {
        self.nav.lot.in_bounds(to.0, to.1) && self.nav.tile_free(to.0, to.1)
    }

    /// Whether a resident can walk from `from` to `to`. (As `find_path` would find: the start may
    /// be a blocked tile, a car's spot or a stair's landing, and the goal must be one to stand on.)
    fn reaches(&self, from: Walk, to: Walk) -> bool {
        // (`Reach::reaches` answers yes for a start off the lot; `find_path` finds no way.)
        self.stands(to)
            && (from == to
                || self.nav.lot.in_bounds(from.0, from.1) && self.reach.reaches(self.nav, from, to))
    }

    /// `reaches`, round the plot `plot` and a little of the street: a house's own door reaches
    /// what is in it.
    fn reaches_in(&mut self, from: Walk, to: Walk, plot: &Plot) -> bool {
        if !self.stands(to) {
            return false;
        }
        if from == to {
            return true;
        }
        let pad = 6;
        let round = (
            plot.x - pad,
            plot.z - pad,
            plot.x + plot.w + pad,
            plot.z + plot.d + pad,
        );
        let nav = self.nav;
        self.seen
            .entry((from, round))
            .or_insert_with(|| flood(nav, from, round))
            .contains(&nav.lot.tile_index(to.0, to.1))
    }
}

/// Every tile (`Lot::tile_index`) a resident can walk to from `start` (`NavGrid::neighbours` step
/// by step) within `round`.
fn flood(nav: &NavGrid, start: Walk, round: Rect) -> std::collections::HashSet<usize> {
    let mut seen = std::collections::HashSet::new();
    if !nav.lot.in_bounds(start.0, start.1) {
        return seen;
    }
    seen.insert(nav.lot.tile_index(start.0, start.1));
    let mut walk = vec![start];
    while let Some((x, z)) = walk.pop() {
        // (Neighbours, and a stair's top and its landing however far apart they are.)
        for (nx, nz) in nav.neighbours(x, z) {
            if !nav.lot.in_bounds(nx, nz) {
                continue;
            }
            // (Rows of every storey, by the ground row they are over.)
            let gz = nav.stairs.ground_z(nz);
            if nx < round.0 || gz < round.1 || nx >= round.2 || gz >= round.3 {
                continue;
            }
            if seen.insert(nav.lot.tile_index(nx, nz)) {
                walk.push((nx, nz));
            }
        }
    }
    seen
}

/// The town as it loads, before the cars test's two days (`generated_worlds_cars_keep_to_the_roads`):
/// no more residents than a town holds, a way out of town, a snapshot row for every signal head, the
/// roads one network, every lot reached from every way into town, no garden shut off by the town's
/// fences, everything residents use reached (outdoors, in the houses and their basements); the
/// basement rooms counted.
fn livable_at_start(
    w: &World,
    world: &serde_json::Value,
    label: &str,
    failures: &mut Vec<String>,
) -> usize {
    if w.sims.len() > MAX_SIMS {
        failures.push(format!(
            "{label}: {} residents, more than {MAX_SIMS}",
            w.sims.len()
        ));
    }
    if w.exits.is_empty() {
        failures.push(format!("{label}: no way out of town"));
    }
    // Every junction's lights with a row in the snapshot for each of its heads: none undrawn.
    let heads: usize = w.travel.signals.iter().map(|s| s.arms.len()).sum();
    println!(
        "  {label}: {} lit junctions, {heads} signal heads",
        w.travel.signals.len()
    );
    if heads > sim_core::snapshot::MAX_LIGHTS {
        failures.push(format!(
            "{label}: {heads} signal heads, past the snapshot's {} rows",
            sim_core::snapshot::MAX_LIGHTS
        ));
    }
    // The town's roads one network as the sim joins them.
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
            kept_off: &[],
        };
        let mut walks = Walks::new(&nav);
        let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
        let exits: Vec<_> = w.exits.iter().map(|&e| tile(e)).collect();
        // Every lot from every way into town.
        for plot in &w.plots {
            let entry = tile(plot.arrival_point());
            for &exit in &exits {
                if !walks.reaches(exit, entry) {
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
                .any(|t| walks.reaches(start, t))
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
                .map(|p| (p, tile(w.plots[p as usize].arrival_point())))
        };
        let down = w.storey_row(-1);
        let mut seen = std::collections::HashSet::new();
        for z in down..down + w.storey_depth() {
            for x in 0..w.lot.width as i32 {
                let room = w.lot.room_at(x, z);
                if room == OUTDOORS || !nav.tile_free(x, z) || !seen.insert(room) {
                    continue;
                }
                let Some((plot, from)) = door(x, z) else {
                    continue;
                };
                cellars += 1;
                let walked = (z..down + w.storey_depth())
                    .flat_map(|tz| (0..w.lot.width as i32).map(move |tx| (tx, tz)));
                if !walked
                    .filter(|&(tx, tz)| w.lot.room_at(tx, tz) == room && nav.tile_free(tx, tz))
                    .take(8)
                    .any(|t| walks.reaches_in(from, t, &w.plots[plot as usize]))
                {
                    failures.push(format!(
                        "{label}: can't get into the basement room at {x},{z}"
                    ));
                }
            }
        }
        for o in w.objects.iter().filter(|o| {
            !w.content.objects[o.def].interactions.is_empty() && w.lot.room_at(o.x, o.z) != OUTDOORS
        }) {
            let Some((plot, from)) = door(o.x, o.z) else {
                continue;
            };
            if !surface::use_tiles(&w.content, &w.objects, &nav, o)
                .into_iter()
                .any(|t| walks.reaches_in(from, t, &w.plots[plot as usize]))
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
    common::check(w);
    cellars
}

/// The first half day of the cars test's two (`generated_worlds_cars_keep_to_the_roads`), everyone's
/// free will: those with a bike of their own ride it to work or school (`ride.rs`), and out on a visit
/// or an outing: in a town with bikes at homes without a car someone does, and nobody is on one in
/// town for longer than a ride takes (stuck on it), one way (a ride out to a café and straight back
/// home is two); nobody goes home from a visit before `visits.leastMinutes` there but for a reason;
/// no two cars stand or drive on top of each other where one is drawn (`check_cars`).
struct HalfDay {
    bikes: usize,
    carless_bikes: usize,
    rode: std::collections::BTreeSet<usize>,
    on_since: Vec<Option<(u64, u8)>>,
    /// Who's at a visit or an outing, since when, where.
    there: Vec<Option<(u64, u32)>>,
    short: std::collections::BTreeSet<String>,
    least: u64,
    stuck: std::collections::BTreeSet<String>,
    on_road: std::collections::BTreeSet<String>,
    stacked: Stacked,
}

/// How long `HalfDay` watches (ticks).
const HALF_DAY: u64 = 12 * TICKS_PER_HOUR;

impl HalfDay {
    fn new(w: &World) -> Self {
        // (A household with a car at home drives: those without one ride.)
        let carless = |plot: u32| {
            w.households.iter().enumerate().any(|(h, hh)| {
                hh.plot == Some(plot) && !w.cars.iter().any(|c| c.household == h as u32)
            })
        };
        let vehicles = || {
            w.objects
                .iter()
                .filter(|o| w.content.objects[o.def].vehicle.is_some())
        };
        HalfDay {
            bikes: vehicles().count(),
            carless_bikes: vehicles()
                .filter(|o| w.plot_at(o.x, o.z).is_some_and(carless))
                .count(),
            rode: Default::default(),
            on_since: vec![None; w.sims.len()],
            there: vec![None; w.sims.len()],
            short: Default::default(),
            least: common::mins(w.content.visits.least_minutes),
            stuck: Default::default(),
            on_road: Default::default(),
            stacked: Default::default(),
        }
    }

    /// Tick `t` of the hour just played (`watch`: the world looked at, `common::watch_home`): the
    /// cars every tick, who rides and who's out every tenth.
    fn tick(&mut self, w: &World, watch: [i32; 4], t: u64) {
        check_cars(w, watch, &mut self.stacked, &mut self.on_road);
        if !t.is_multiple_of(10) {
            return;
        }
        for (k, sim) in w.sims.iter().enumerate() {
            let on = sim.riding.is_some() && sim.away_until.is_none();
            if sim.riding.is_some() {
                self.rode.insert(k);
            }
            // (Which way: out, home, to work.)
            let leg = match sim.current().map(|a| a.task.kind) {
                Some(TaskKind::Visit { .. }) => 1,
                Some(TaskKind::GoHome) => 2,
                Some(TaskKind::Work) => 3,
                _ => 0,
            };
            if let Some(at) = self.there.get_mut(k) {
                // (Over once they're off home, or gone home.)
                let home = sim
                    .current()
                    .is_some_and(|a| matches!(a.task.kind, TaskKind::GoHome));
                let now = sim
                    .visiting
                    .filter(|v| v.arrived > 0 && !home)
                    .map(|v| (v.arrived, v.plot));
                if let Some((t0, plot)) = *at
                    && now.is_none_or(|n| n.0 != t0)
                {
                    let content = &w.content;
                    let critical = content
                        .needs
                        .iter()
                        .zip(&sim.needs)
                        .any(|(d, &n)| !d.room && n < content.autonomy.urgent_below + 0.02);
                    let hour = sim_core::clock::hour(w.tick);
                    let work = sim
                        .current()
                        .is_some_and(|a| matches!(a.task.kind, TaskKind::Work));
                    // (Come in someone else's car: they go when its driver does.)
                    let passenger = w.cars.iter().any(|c| {
                        c.trip.is_some_and(|t| t.driver != sim.id) && c.riders.contains(&sim.id)
                    });
                    // (Brought others in their car: they go when one of them must, a
                    // need about to run out, say: `cars::call_party_home`.)
                    let party = w.cars.iter().any(|c| {
                        c.trip.is_some_and(|t| t.driver == sim.id)
                            && c.riders.iter().any(|&r| {
                                w.sims.get(r as usize).is_some_and(|o| {
                                    o.current()
                                        .is_some_and(|a| matches!(a.task.kind, TaskKind::GoHome))
                                })
                            })
                    });
                    if w.tick < t0 + self.least
                        && !critical
                        && !work
                        && !passenger
                        && !party
                        && (6.0..22.0).contains(&hour)
                        && sim.here()
                    {
                        self.short.insert(format!(
                            "{} went home from plot {plot} {} min after getting there",
                            sim.name,
                            (w.tick - t0) / common::TICKS_PER_MINUTE
                        ));
                    }
                }
                *at = now;
            }
            let since = self.on_since.get_mut(k);
            let Some(since) = since else { continue };
            match (on, *since) {
                (true, None) => *since = Some((w.tick, leg)),
                (true, Some((_, was))) if was != leg => *since = Some((w.tick, leg)),
                (true, Some((t0, _))) if w.tick - t0 > RIDE_HOURS * TICKS_PER_HOUR => {
                    self.stuck.insert(sim.name.clone());
                }
                (false, _) => *since = None,
                _ => {}
            }
        }
    }

    /// Its failures into `failures`, a line about the town, and (a town with bikes at homes without
    /// a car) whether someone rode one.
    fn finish(
        self,
        w: &World,
        label: &str,
        cellars: usize,
        failures: &mut Vec<String>,
    ) -> Option<bool> {
        for name in &self.stuck {
            failures.push(format!(
                "{label}: {name} on a bike in town for over {RIDE_HOURS} hours"
            ));
        }
        for s in &self.short {
            failures.push(format!("{label}: {s}"));
        }
        failures.extend(self.on_road.into_iter().map(|f| format!("{label}: {f}")));
        eprintln!(
            "  {label}: {}×{}, {} residents, {} things, {cellars} rooms in basements, {} bikes, {} rode",
            w.lot.width,
            w.lot.depth,
            w.sims.len(),
            w.objects.len(),
            self.bikes,
            self.rode.len(),
        );
        (self.carless_bikes > 0).then_some(!self.rode.is_empty())
    }
}

/// How late the work taxi's riders may be at work on average (game minutes).
const TAXI_LATE: f32 = 0.7;
/// In a big town, a preview in 0.7.0 (`BIG_PREVIEW`), riders held up in a jam in view come later:
/// on average this late at most (min: about twice the most seen), a regression bar.
const BIG_TAXI_LATE: f32 = 6.0;

/// Hours a resident may be on a bike or an e-scooter in town at a time at most (a ride out of town
/// or home takes minutes).
const RIDE_HOURS: u64 = 2;

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

/// Two cars' middles this near (m), facing the same way (within about 25°), are on top of each
/// other, and may be for this long at most (ticks: one turning out through another, or slowly by
/// one standing, goes on past it; two coming into town at the same tick drove all the way home
/// together, one on top of the other, and two parked out at the same spot stood so).
const STACKED: f32 = 0.5;
const STACKED_TICKS: u64 = 10 * sim_core::TICKS_PER_SECOND as u64;

/// Two cars on top of each other (by their indices), since when.
type Stacked = std::collections::HashMap<(usize, usize), u64>;

/// A tick of `w`'s cars: any two of them on top of each other where they're drawn (`STACKED`; one
/// of them drawn with the watch `watch`, `common::drawn`) for longer than `STACKED_TICKS`, in
/// `failures`.
fn check_cars(
    w: &World,
    watch: [i32; 4],
    stacked: &mut Stacked,
    failures: &mut std::collections::BTreeSet<String>,
) {
    let shown: Vec<_> = w
        .cars
        .iter()
        .enumerate()
        .filter_map(|(k, c)| c.shown.map(|(at, yaw, ..)| (k, at, yaw, c.state)))
        .collect();
    let mut now = std::collections::HashSet::new();
    let drawn = common::drawn(w, Some(watch));
    for (i, &(a, pa, ya, sa)) in shown.iter().enumerate() {
        for &(b, pb, yb, sb) in &shown[i + 1..] {
            if (pa[0] - pb[0]).hypot(pa[1] - pb[1]) >= STACKED
                || (ya - yb).cos() < 0.9
                || !(drawn[a] || drawn[b])
            {
                continue;
            }
            now.insert((a, b));
            let since = *stacked.entry((a, b)).or_insert(w.tick);
            if w.tick - since > STACKED_TICKS {
                failures.insert(format!(
                    "cars {a} ({sa:?}) and {b} ({sb:?}) on top of each other at {pa:?} and {pb:?}"
                ));
            }
        }
    }
    stacked.retain(|k, _| now.contains(k));
}

/// A work taxi standing at the kerb for its riders (`taxi.rs`) stands parallel to its lane within
/// this (rad: about 5°), its body clear of its lane's kerb by this at least (m: a little under, as
/// pulled over for one coming the other way on a lane too narrow to pass, `Car::shift`, onto the
/// pavement no further than that) and of the kerbside furniture always (`cars::kerbside`).
const STAND_ANGLE: f32 = 0.09;
/// A street's or a lane's pavements (m, each side: `roads.ts` `SIDEWALK`).
const SIDEWALK: f32 = 1.35;
const STAND_KERB: f32 = -0.05;
/// Two cars on their way passing each other, both moving, one of them a work taxi, keep this far
/// apart at least (m, between their bodies: in their lanes on a 7 m street, 0.25 m on the
/// straight). (By one standing to let it by, the last-resort guard keeps it out of that one.)
const PASS_GAP: f32 = 0.05;

/// The work taxis as `check_taxis` last saw them: standing or not, by car, and the pairs of cars
/// passing each other.
#[derive(Default)]
struct Taxis {
    standing: std::collections::BTreeSet<usize>,
    passing: std::collections::BTreeSet<(usize, usize)>,
    /// Where each car was drawn a tick before.
    was: Vec<Option<[f32; 2]>>,
}

/// A body's corners (`at` its middle, facing `yaw`).
fn body(at: [f32; 2], yaw: f32, len: f32, wid: f32) -> [[f32; 2]; 4] {
    let (s, c) = yaw.sin_cos();
    let f = [s * len / 2.0, c * len / 2.0];
    let r = [c * wid / 2.0, -s * wid / 2.0];
    [
        [at[0] + f[0] + r[0], at[1] + f[1] + r[1]],
        [at[0] + f[0] - r[0], at[1] + f[1] - r[1]],
        [at[0] - f[0] - r[0], at[1] - f[1] - r[1]],
        [at[0] - f[0] + r[0], at[1] - f[1] + r[1]],
    ]
}

/// The gap between two bodies (m; less than none, they overlap), by the separating axes.
fn gap(a: &[[f32; 2]; 4], b: &[[f32; 2]; 4]) -> f32 {
    let mut best = f32::NEG_INFINITY;
    for poly in [a, b] {
        for k in 0..2 {
            let (p, q) = (poly[k], poly[k + 1]);
            let n = [q[1] - p[1], p[0] - q[0]];
            let l = n[0].hypot(n[1]);
            let n = [n[0] / l, n[1] / l];
            let span = |r: &[[f32; 2]; 4]| {
                r.iter()
                    .map(|v| v[0] * n[0] + v[1] * n[1])
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), x| {
                        (lo.min(x), hi.max(x))
                    })
            };
            let ((a0, a1), (b0, b1)) = (span(a), span(b));
            best = best.max(a0.max(b0) - a1.min(b1));
        }
    }
    best
}

/// A car on its way, as `check_taxis` sees it: its index, its body, its middle, which way it
/// faces, its length and whether it's a taxi.
type Going = (usize, [[f32; 2]; 4], [f32; 2], f32, f32, bool);

/// A tick of `w`'s work taxis: one come to stand at the kerb not parallel to its lane, or with its
/// body over the kerb or the kerbside furniture (`STAND_ANGLE`, `STAND_KERB`); and two cars
/// passing each other, one a taxi, nearer than `PASS_GAP` (one of them drawn with the watch
/// `watch`, `common::drawn`); in `failures`.
fn check_taxis(
    w: &World,
    watch: [i32; 4],
    seen: &mut Taxis,
    failures: &mut std::collections::BTreeSet<String>,
) {
    use sim_core::travel::{Mode, RoadKind};
    let g = &w.travel.graph;
    for (c, car) in w.cars.iter().enumerate() {
        if !car.taxi || !car.standing {
            seen.standing.remove(&c);
            continue;
        }
        let Some((at, yaw, ..)) = car.shown else {
            continue;
        };
        if !seen.standing.insert(c) {
            continue;
        }
        let Some(rp) = g.near(at, 30.0, 1, Mode::Car).first().copied() else {
            continue;
        };
        let road = &g.roads[rp.road as usize];
        let t = road.tangent(rp.s);
        let off = sim_core::world::wrap_angle(yaw - t[0].atan2(t[1])).abs();
        let off = off.min(std::f32::consts::PI - off);
        let (kerb, furniture) = match road.kind {
            RoadKind::Country | RoadKind::Trail => (road.width / 2.0, road.width / 2.0),
            _ => (
                (road.width / 2.0 - SIDEWALK).max(1.3),
                road.width / 2.0 - sim_core::cars::kerbside(road.width, road.kind),
            ),
        };
        // (Each corner from the middle of the road it's on.)
        let out = body(at, yaw, car.len, car.wid)
            .iter()
            .filter_map(|&q| {
                let p = g.near(q, 30.0, 1, Mode::Car).first().copied()?;
                (p.road == rp.road).then_some(p.off)
            })
            .fold(0.0f32, f32::max);
        let at_s = format!(
            "{:.1},{:.1} day {} {:05.2}h",
            at[0],
            at[1],
            sim_core::clock::day(w.tick),
            sim_core::clock::hour(w.tick)
        );
        if off > STAND_ANGLE {
            failures.insert(format!(
                "a taxi stood at the kerb {:.0}° off its lane at {at_s}",
                off.to_degrees()
            ));
        }
        if out > kerb + car.shift - STAND_KERB || out > furniture {
            failures.insert(format!(
                "a taxi stood {:.2} m from the road's middle, its kerb {kerb:.2} m (pulled over {:.2} m), at {at_s}",
                out, car.shift
            ));
        }
    }
    // Passing: two on their way side by side, both moving, facing about opposite ways, one a taxi.
    seen.was.resize(w.cars.len(), None);
    let going: Vec<Going> = w
        .cars
        .iter()
        .enumerate()
        .filter_map(|(k, c)| {
            let (at, yaw, moving, _) = c.shown?;
            let moved = seen.was[k].is_some_and(|p| p != at);
            (moving && moved).then(|| (k, body(at, yaw, c.len, c.wid), at, yaw, c.len, c.taxi))
        })
        .collect();
    for (k, c) in w.cars.iter().enumerate() {
        seen.was[k] = c.shown.map(|s| s.0);
    }
    let mut now = std::collections::BTreeSet::new();
    let drawn = common::drawn(w, Some(watch));
    for (i, a) in going.iter().enumerate() {
        for b in &going[i + 1..] {
            if !(a.5 || b.5)
                || (a.2[0] - b.2[0]).hypot(a.2[1] - b.2[1]) > 6.0
                || !(drawn[a.0] || drawn[b.0])
            {
                continue;
            }
            let fwd = [a.3.sin(), a.3.cos()];
            let along = (b.2[0] - a.2[0]) * fwd[0] + (b.2[1] - a.2[1]) * fwd[1];
            if (a.3 - b.3).cos() > -0.85 || along.abs() > (a.4 + b.4) / 2.0 - 0.3 {
                continue;
            }
            let d = gap(&a.1, &b.1);
            now.insert((a.0, b.0));
            if d < PASS_GAP && !seen.passing.contains(&(a.0, b.0)) {
                failures.insert(format!(
                    "cars {} and {} passed {d:.2} m apart at {:.1},{:.1} day {} {:05.2}h",
                    a.0,
                    b.0,
                    a.2[0],
                    a.2[1],
                    sim_core::clock::day(w.tick),
                    sim_core::clock::hour(w.tick)
                ));
            }
        }
    }
    seen.passing = now;
}

/// How long a car on its way may stand still while nothing it waits for goes on, however far
/// round (ticks: `traffic::waits_on`; held up for good, two or more waiting for each other), and
/// how long it may stand still at all (waiting its turn for a busy junction). A car queued right behind another going its way isn't counted:
/// the one in front of the queue is.
const CAR_STUCK_TICKS: u64 = 10 * sim_core::TICKS_PER_MINUTE;
const CAR_WAIT_TICKS: u64 = 60 * sim_core::TICKS_PER_MINUTE;

/// The cars standing still on their way: by car, the furthest along its way it had come and since
/// when (a tick), and the last tick anything it waited for went on; and by car, the furthest along
/// its way it has come while driving (`traffic::on_its_way`). Going on is coming further than that:
/// a car backing up a little and coming back up to where it was, again and again, isn't. A car
/// on a new way begins again: a work taxi on its next run (straight on from the last), one
/// turned round (`Car::detour`, `cars::unjam`); by car, the way it's on (its run, the way it
/// turned round to).
#[derive(Default)]
struct Standing {
    cars: std::collections::BTreeMap<usize, (f32, u64, u64)>,
    furthest: std::collections::BTreeMap<u64, f32>,
    ways: std::collections::BTreeMap<usize, (Option<u32>, Option<sim_core::cars::Detour>)>,
}

/// A tick of `w`'s cars on their way: any standing still for too long (`CAR_STUCK_TICKS`,
/// `CAR_WAIT_TICKS`), in `failures`.
fn check_stuck(
    w: &World,
    standing: &mut Standing,
    failures: &mut std::collections::BTreeSet<String>,
    in_view: &mut std::collections::BTreeSet<String>,
) {
    use sim_core::cars::{CarState, Sight, car_key};
    use sim_core::traffic::{at_red, on_its_way, waits_on};
    // Who went on this tick: further along its way than it has been since it began to drive (a
    // taxi standing at a stop, then on along its next way, begins again).
    let mut went = std::collections::BTreeSet::new();
    let mut now = std::collections::BTreeMap::new();
    for (c, car) in w.cars.iter().enumerate() {
        let k = car_key(c);
        // (On a new way: from where it is on it.)
        let run = w
            .taxis
            .runs
            .iter()
            .find(|r| r.car == Some(c as u32))
            .map(|r| r.id);
        let way = (run, car.detour);
        let was = standing.ways.insert(c, way).unwrap_or((None, None));
        if way.0.is_some() && way.0 != was.0 || way.1.is_some() && way.1 != was.1 {
            standing.furthest.remove(&k);
            standing.cars.remove(&c);
        }
        let Some(p) = on_its_way(w, k).filter(|_| car.shown.is_some_and(|s| s.2)) else {
            continue;
        };
        let furthest = standing.furthest.get(&k).copied();
        if furthest.is_none_or(|f| p > f) {
            went.insert(k);
        }
        now.insert(k, furthest.map_or(p, |f| f.max(p)));
    }
    standing.furthest = now;
    // (Right behind one going its way.)
    let queued = |c: usize| {
        let me = w.cars[c].shown;
        w.cars[c].waits_for.is_some_and(|x| {
            x >= 1 << 63
                && w.cars
                    .get((x & !(1 << 63)) as usize)
                    .and_then(|o| o.shown)
                    .zip(me)
                    .is_some_and(|(o, m)| {
                        let way = [m.1.sin(), m.1.cos()];
                        let d = [o.0[0] - m.0[0], o.0[1] - m.0[1]];
                        (m.1 - o.1).cos() > 0.9
                            && d[0] * way[0] + d[1] * way[1] > 0.5
                            && (d[0] * way[1] - d[1] * way[0]).abs() < 1.2
                    })
        })
    };
    for (c, car) in w.cars.iter().enumerate() {
        let driving = matches!(car.state, CarState::Leaving | CarState::Arriving)
            && car.shown.is_some_and(|s| s.2);
        let entry = standing
            .cars
            .entry(c)
            .or_insert((car.progress, w.tick, w.tick));
        if !driving || car.progress > entry.0 || queued(c) {
            // (Not driving, it begins again: a car parked keeps its last way's progress.)
            let from = if driving {
                car.progress
            } else {
                f32::NEG_INFINITY
            };
            *entry = (from, w.tick, w.tick);
            continue;
        }
        // (Anything it waits for, however far round, going on; or a red light, which changes:
        // waiting there longer than `CAR_WAIT_TICKS` is all the same.)
        let mut seen = vec![car_key(c)];
        let mut i = 0;
        while i < seen.len() && seen.len() < 64 {
            if at_red(w, seen[i]) {
                entry.2 = w.tick;
            }
            for k in waits_on(w, seen[i]) {
                if went.contains(&k) {
                    entry.2 = w.tick;
                }
                if !seen.contains(&k) {
                    seen.push(k);
                }
            }
            i += 1;
        }
        let (since, going) = (entry.1, entry.2.max(entry.1));
        if w.tick - going > CAR_STUCK_TICKS || w.tick - since > CAR_WAIT_TICKS {
            let f = format!(
                "car {c} ({:?}) stood still at {:?} for {} minutes{}",
                car.state,
                car.shown.map(|s| s.0),
                (w.tick - since) / sim_core::TICKS_PER_MINUTE,
                if w.tick - going > CAR_STUCK_TICKS {
                    ", held up for good"
                } else {
                    ""
                }
            );
            // (A big world, a preview in 0.7.0: one stuck in the seen area, or at its edge waiting
            // to come in behind one there, is counted, not a failure, `BIG_PREVIEW`; out of sight
            // otherwise it is.)
            let in_jam = matches!(car.sight, Sight::Seen | Sight::Wait(_));
            if in_jam {
                in_view.insert(f);
            } else {
                failures.insert(f);
            }
        }
    }
}

/// Big towns are a preview in 0.7.0 (the owner's decision, 2026-11-07): at rush hour traffic can
/// still jam at a busy junction near the camera, to be finished in a later release, and its direct
/// consequences are known too. In a big world a car held up in the seen area (`cars::Sight::Seen`),
/// or at its edge waiting to drive in behind one there (`Sight::Wait`), standing too long or still
/// on its way at three in the morning (having stood two hours at most on that drive but there),
/// is counted and printed (`print_in_view`), not a failure,
/// and so is the work taxi's lateness up to `BIG_TAXI_LATE`; one held up out of sight otherwise
/// still fails, as does everything in the picture (overlaps, touches, vanishing, popping in on
/// top, jumps) and every other check.
const BIG_PREVIEW: &str = "a big town, a preview in 0.7.0: counted, not failing";

/// The cars held up in the seen area of a big world (`BIG_PREVIEW`), printed with their number.
fn print_in_view(label: &str, in_view: &std::collections::BTreeSet<String>) {
    if in_view.is_empty() {
        return;
    }
    eprintln!(
        "  {label}: {} held up in view ({BIG_PREVIEW})",
        in_view.len()
    );
    for f in in_view {
        eprintln!("    in view: {f}");
    }
}

/// Where a car was shown on its way a tick, its yaw, and its slip (degrees: `Slips`).
type SlipAt = ([f32; 2], f32, f32);

/// How far each car on its way faces off the way it goes (`SLIP=1`: printed per world, not a
/// check; `SLIP=trace`, the last few seconds up to the worst too): tick by tick, the angle between
/// its yaw and the way it moved (either way: it backs up too), its pulling over to pass taken out
/// (that moves it sideways, standing as it may be).
#[derive(Default)]
struct Slips {
    /// Per car, where it was shown the last ticks on its way (up to 150), its yaw, and its slip
    /// (degrees).
    last: Vec<std::collections::VecDeque<SlipAt>>,
    /// The worst, where and when, and the ticks before it.
    worst: (f32, String, Vec<SlipAt>),
    /// Ticks more than 5, 10, 20 and 40 degrees off.
    over: [usize; 4],
}

impl Slips {
    fn sample(&mut self, w: &World) {
        use sim_core::world::wrap_angle;
        self.last.resize(w.cars.len(), Default::default());
        for (c, car) in w.cars.iter().enumerate() {
            let h = &mut self.last[c];
            let on_way = matches!(
                car.state,
                sim_core::cars::CarState::Leaving | sim_core::cars::CarState::Arriving
            );
            let Some((at, yaw, true, _)) = car.shown.filter(|_| on_way) else {
                h.clear();
                continue;
            };
            // (Back by its shift: `cars::shifted`.)
            let at = [at[0] + yaw.cos() * car.shift, at[1] - yaw.sin() * car.shift];
            let mut slip = 0.0;
            if let Some(&(p, was, _)) = h.back() {
                let (dx, dz) = (at[0] - p[0], at[1] - p[1]);
                let len = dx.hypot(dz);
                if len > 0.5 {
                    h.clear();
                } else if len > 0.002 {
                    let off = wrap_angle(dx.atan2(dz) - was - wrap_angle(yaw - was) / 2.0).abs();
                    slip = off.min(std::f32::consts::PI - off).to_degrees();
                }
            }
            h.push_back((at, yaw, slip));
            if h.len() > 150 {
                h.pop_front();
            }
            for (k, most) in [5.0, 10.0, 20.0, 40.0].into_iter().enumerate() {
                self.over[k] += usize::from(slip > most);
            }
            if slip > self.worst.0 {
                self.worst = (
                    slip,
                    format!(
                        "car {c} (plot {}{}) {:?} at {:.1},{:.1} day {} {:05.2}h",
                        car.plot,
                        if car.taxi { ", a taxi" } else { "" },
                        car.state,
                        at[0],
                        at[1],
                        sim_core::clock::day(w.tick),
                        sim_core::clock::hour(w.tick)
                    ),
                    h.iter().copied().collect(),
                );
            }
        }
    }

    fn print(&self, label: &str, trace: bool) {
        eprintln!(
            "  {label}: worst slip {:.1} deg, {}; ticks over 5/10/20/40 deg: {:?}",
            self.worst.0, self.worst.1, self.over
        );
        if trace {
            for &(p, yaw, slip) in &self.worst.2 {
                eprintln!(
                    "    {:.2},{:.2} yaw {:6.1} slip {slip:5.1}",
                    p[0],
                    p[1],
                    yaw.to_degrees()
                );
            }
        }
    }
}

/// Generated worlds as they load (`livable_at_start`) and their first half day (`HalfDay`): the
/// livable towns' checks, run here over the same play as the cars' (they were a test of their own,
/// which made and played every world a second time).
///
/// The households' cars of generated worlds (`town.ts` writes them where the street drew them,
/// with the road each driveway meets): every one loads parked on its driveway on its lot, nothing
/// built over it, and few stay parked for want of a way onto the road (`NO_WAY`, a driveway that
/// comes alongside its road: `Car::no_way`); driven for two days, the
/// cars on the move keep to the paving (`OFF_PAVING`), some drive to work and some go on trips to
/// another hamlet, and at three in the morning every car is parked at home, parked
/// out at a public spot, or away with someone at work in it; nobody waits for a car for hours
/// (`CAR_HOURS`; a world's long stretches away or walking are about as many without cars). A
/// preview in 0.7.0 (the owner's decision, 2026-11-07), a car held up in the seen area is counted,
/// not a failure: in-view jams are to be finished in a later release (`BIG_PREVIEW`).
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
    let items: Vec<(usize, &serde_json::Value)> = worlds.iter().enumerate().collect();
    let results = each_world(&items, |&(i, world)| {
        let mut failures = Vec::new();
        // (Worlds driven, cars, with no way out, drives to work, trips, taxi riders at work and
        // their lateness: summed over the worlds below.)
        let mut tally = [1usize, 0, 0, 0, 0, 0, 0];
        let label = format!("{} {} ({})", world["kind"], world["seed"], world["name"]);
        // (Seeded by the world's own seed: one picked out of the file runs the same alone.)
        let seed = world["seed"].as_u64().unwrap_or(i as u64) as u32;
        let mut w = match World::from_json(&content, &world["town"].to_string(), seed + 1) {
            Ok(w) => w,
            Err(e) => {
                failures.push(format!("{label}: {e}"));
                return (tally, failures, None);
            }
        };
        // Its porches are the ones its houses and shops had drawn before porches were the
        // simulation's (as an older file or save has them worked out: `World::guess_porches`), so
        // the town looks as it did.
        let key = |p: &sim_core::porches::Porch| (p.plot, p.x, p.z, p.out);
        let mut filed_porches = w.porches.clone();
        let mut guessed = w.guess_porches();
        filed_porches.sort_by_key(key);
        guessed.sort_by_key(key);
        if filed_porches != guessed {
            failures.push(format!(
                "{label}: the town's {} porches aren't the {} drawn before (only filed: {:?}; only drawn: {:?})",
                filed_porches.len(),
                guessed.len(),
                filed_porches.iter().filter(|p| !guessed.contains(p)).collect::<Vec<_>>(),
                guessed.iter().filter(|p| !filed_porches.contains(p)).collect::<Vec<_>>(),
            ));
        }
        // Nothing stands where a porch's posts do (nobody walks through them, `Porch::post_tiles`).
        for p in &w.porches {
            for (x, z) in p.post_tiles() {
                let on = w.objects.iter().find(|o| {
                    let (sw, sd) = o.size(&w.content);
                    w.content.objects[o.def].layer == sim_core::content::Layer::Floor
                        && (o.x..o.x + sw).contains(&x)
                        && (o.z..o.z + sd).contains(&z)
                });
                if let Some(o) = on {
                    failures.push(format!(
                        "{label}: the {} at {},{} stands where a porch's post does",
                        w.content.objects[o.def].id, o.x, o.z
                    ));
                }
            }
        }
        // The town as it loads (`livable_at_start`); its first half day is `HalfDay`'s too.
        let cellars = livable_at_start(&w, world, &label, &mut failures);
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
                kept_off: &[],
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
        // (A world is about 1 km across: walks across it take hours.)
        let mut watch = common::Watch::new().routing_hours(8.0);
        let days = 2;
        let end = clock::tick_at(clock::day(w.tick) + days, 3.0 * 60.0).unwrap();
        let mut was: Vec<CarState> = w.cars.iter().map(|c| c.state).collect();
        // Per resident: since when they've been on their way to a car, or sat in one standing
        // parked (none: neither).
        let mut held: Vec<Option<u64>> = vec![None; w.sims.len()];
        let mut worst = (0.0f32, String::new());
        let started = std::time::Instant::now();
        // (Every car on its way, both days: none held up for good, `check_stuck`, the day-1 check's
        // rule, on day 2 too. The work taxis: standing at the kerb parallel to their lanes and
        // clear of the kerb; passing others clear of them, `check_taxis`.)
        let mut standing: Standing = Default::default();
        let mut stood = std::collections::BTreeSet::new();
        let mut stood_in_view = std::collections::BTreeSet::new();
        // (Per car, how long it has stood on this drive, in all and in the seen area or at its edge
        // waiting to come in, and where along its way it was: a big world's jam in view,
        // `BIG_PREVIEW`. Stood, not how far behind its way's time it is: turned round, a car's on
        // a new way, its time along the old one no hold-up.)
        let mut held_in_view: Vec<(u64, u64, f32)> = Vec::new();
        let mut taxis: Taxis = Default::default();
        // (The world looked at as its game starts: `common::watch_home`.)
        let view = common::watch_home(&mut w);
        let mut half = HalfDay::new(&w);
        let start = w.tick;
        let mut taxi_faults = std::collections::BTreeSet::new();
        let slip = std::env::var("SLIP").ok();
        let mut slips = Slips::default();
        while w.tick < end {
            w.tick_once();
            // (The first half day: `HalfDay`'s checks, and the whole town checked every three hours.)
            let k = w.tick - start;
            if k <= HALF_DAY {
                half.tick(&w, view, (k - 1) % TICKS_PER_HOUR);
                if k.is_multiple_of(3 * TICKS_PER_HOUR) {
                    common::check(&w);
                }
            }
            watch.sample(&w);
            check_stuck(&w, &mut standing, &mut stood, &mut stood_in_view);
            held_in_view.resize(w.cars.len(), (0, 0, f32::NAN));
            for (c, car) in w.cars.iter().enumerate() {
                let held = &mut held_in_view[c];
                if !matches!(car.state, CarState::Leaving | CarState::Arriving) {
                    *held = (0, 0, f32::NAN);
                    continue;
                }
                let in_jam = matches!(
                    car.sight,
                    sim_core::cars::Sight::Seen | sim_core::cars::Sight::Wait(_)
                );
                if (car.progress - held.2).abs() < 1e-3 {
                    held.1 += 1;
                    if in_jam {
                        held.0 += 1;
                    }
                }
                held.2 = car.progress;
            }
            check_taxis(&w, view, &mut taxis, &mut taxi_faults);
            if slip.is_some() {
                slips.sample(&w);
            }
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
            // (The work taxis' slots come as they're needed: keep to the roads as any car.)
            was.resize(w.cars.len(), CarState::Away);
            for (c, car) in w.cars.iter().enumerate() {
                if was[c] != car.state && !car.taxi {
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
                    && w.tick.is_multiple_of(common::secs(1.0))
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
        failures.extend(taxi_faults.into_iter().map(|f| format!("{label}: {f}")));
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
        // stuck). (A work taxi, the households' no more than a bus was: at three in the morning on
        // its way to an early shift or home from a late one, as on any run, kept to the roads
        // and held up nowhere for good by the checks above.)
        for car in w.cars.iter().filter(|c| !c.taxi) {
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
                let f = format!(
                    "the car of plot {} is {:?} at three in the morning (since day {} {:05.2}h)",
                    car.plot,
                    car.state,
                    clock::day(car.since),
                    clock::hour(car.since)
                );
                // (A big world's car still on its way, held up in the seen area, or having stood two
                // hours at most on this drive but there, `BIG_PREVIEW`.)
                let on_way = matches!(car.state, CarState::Leaving | CarState::Arriving);
                let in_jam = matches!(
                    car.sight,
                    sim_core::cars::Sight::Seen | sim_core::cars::Sight::Wait(_)
                );
                let c = w
                    .cars
                    .iter()
                    .position(|k| std::ptr::eq(k, car))
                    .unwrap_or(0);
                let (seen, all) = held_in_view.get(c).map_or((0, 0), |h| (h.0, h.1));
                let but_for = all - seen <= 2 * TICKS_PER_HOUR;
                if on_way && (in_jam || but_for) {
                    stood_in_view.insert(f);
                } else {
                    failures.push(format!("{label}: {f}"));
                }
            }
        }
        // (The first time each car stood still too long in the seen area of a big world.)
        let mut seen_once = std::collections::BTreeSet::new();
        let in_view: std::collections::BTreeSet<String> = stood_in_view
            .into_iter()
            .filter(|f| seen_once.insert(f.split(" (").next().unwrap_or_default().to_string()))
            .collect();
        print_in_view(&label, &in_view);
        // (The first time each car stood still too long: once a car, not once a tick.)
        let mut cars_stood = std::collections::BTreeSet::new();
        for f in stood {
            let car = f.split(" (").next().unwrap_or_default().to_string();
            if cars_stood.insert(car) {
                failures.push(format!("{label}: {f}"));
            }
        }
        // The work taxi's riders at work, and how late (ticks, early as on time).
        tally[5] += w.taxis.arrivals.len();
        tally[6] += w
            .taxis
            .arrivals
            .iter()
            .map(|a| a.1.max(0) as usize)
            .sum::<usize>();
        // (A world's residents away or walking for long are about as many with cars as without:
        // only those waiting for a car count, above.)
        if let Some(how) = &slip {
            slips.print(&label, how == "trace");
        }
        eprintln!(
            "  {label}: {} cars, worst {:.2} m off the paving, {} long stretches, {:.1} s",
            w.cars.len(),
            worst.0,
            watch.stuck.len(),
            started.elapsed().as_secs_f32()
        );
        let ridden = half.finish(&w, &label, cellars, &mut failures);
        (tally, failures, ridden)
    });
    let mut failures = Vec::new();
    // In a town with bikes at homes without a car, someone rode one to work or school in its first
    // half day in half the towns at least (half a day may be the weekend, or a household's work may
    // start after it).
    let towns = results.iter().filter(|(_, _, r)| r.is_some()).count();
    let ridden = results.iter().filter(|(_, _, r)| *r == Some(true)).count();
    if ridden * 2 < towns {
        failures.push(format!(
            "someone rode a bike to work or school in {ridden} of {towns} towns with bikes at homes without a car"
        ));
    }
    let mut sum = [0usize; 7];
    for (tally, mut found, _) in results {
        for (s, t) in sum.iter_mut().zip(tally) {
            *s += t;
        }
        failures.append(&mut found);
    }
    let [n, cars, stay, commutes, trips, taxi, late] = &sum;
    if *n > 0 {
        let mean_late = *late as f32 / (*taxi).max(1) as f32 / sim_core::TICKS_PER_MINUTE as f32;
        eprintln!(
            "{n} worlds, {cars} cars ({stay} with no way out or no door to open), {commutes} drives to work, {trips} trips, {taxi} at work by taxi ({mean_late:.2} min late on average)"
        );
        // Some residents a long way out of town ride the work taxi, about on time (`taxi.rs`).
        if *taxi == 0 {
            failures.push("nobody went to work by taxi".to_string());
        }
        // (A big town's taxis held up in a jam in view come late: counted, not failing, but past
        // `BIG_TAXI_LATE`, `BIG_PREVIEW`.)
        if mean_late > BIG_TAXI_LATE {
            failures.push(format!(
                "taxi riders {mean_late:.2} min late on average, more than {BIG_TAXI_LATE}"
            ));
        } else if mean_late > TAXI_LATE {
            eprintln!(
                "taxi riders {mean_late:.2} min late on average, more than {TAXI_LATE} ({BIG_PREVIEW})"
            );
        }
        if *cars == 0 || *stay as f32 > *cars as f32 * NO_WAY {
            failures.push(format!("{stay} of {cars} cars have no way out by road"));
        }
        if *commutes == 0 {
            failures.push("nobody drove to work".to_string());
        }
        if *trips == 0 {
            failures.push("nobody drove to another hamlet".to_string());
        }
    }
    eprintln!("{} problems", failures.len());
    for f in failures.iter().take(400) {
        eprintln!("  {f}");
    }
    assert!(failures.is_empty());
}

/// How long a drive across a world takes with the roads to itself, from its road furthest south
/// to its road furthest west (some one to two km of road), on the traffic's own clock
/// (`DRIVE_SCALE`), 32 km/h in the hamlets, straight through junctions at speed: about 20 to 25
/// game minutes for the middle world of a sample (`ACROSS_MINUTES` at most; 20.6 to 30.7 over seed
/// bases 16 to 35: about half an hour, as asked, with room for a sample's spread, bases 28 and 34
/// sitting at 30.0 to 30.7), none more than `ACROSS_MOST` (game minutes; 55.2 the most seen,
/// Otterpool). At real speeds they took two to five game hours.
const ACROSS_MINUTES: f32 = 32.0;
const ACROSS_MOST: f32 = 60.0;

#[test]
#[ignore]
fn generated_worlds_are_driven_across_in_about_half_an_hour() {
    use sim_core::travel::Mode;
    let path = std::env::var("WORLDS")
        .expect("set WORLDS to a file made by `bun tools/worlds/sample.ts worlds.json`");
    let worlds: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let content = common::content();
    let items: Vec<(usize, &serde_json::Value)> = worlds.iter().enumerate().collect();
    let results = each_world(&items, |&(i, world)| {
        let label = format!("{} {} ({})", world["kind"], world["seed"], world["name"]);
        let w = match World::from_json(&content, &world["town"].to_string(), i as u32 + 1) {
            Ok(w) => w,
            Err(e) => return Err(format!("{label}: {e}")),
        };
        // (The junctions and ends of the roads cars drive on: x east, z south.)
        let g = &w.travel.graph;
        let nodes: Vec<[f32; 2]> = g
            .nodes
            .iter()
            .copied()
            .filter(|&p| !g.near(p, 1.0, 1, Mode::Car).is_empty())
            .collect();
        let south = nodes.iter().copied().max_by(|a, b| a[1].total_cmp(&b[1]));
        let west = nodes.iter().copied().min_by(|a, b| a[0].total_cmp(&b[0]));
        let (Some(south), Some(west)) = (south, west) else {
            return Err(format!("{label}: no roads"));
        };
        // (The road furthest south furthest west too: no way across to time.)
        if south == west {
            return Ok(None);
        }
        let Some(ticks) = sim_core::cars::empty_drive_ticks(&w.travel, south, west) else {
            return Err(format!(
                "{label}: no way by road from {south:?} to {west:?}"
            ));
        };
        let minutes = ticks / sim_core::TICKS_PER_MINUTE as f32;
        eprintln!("  {label}: {south:?} to {west:?} in {minutes:.1} game minutes");
        if minutes > ACROSS_MOST {
            return Err(format!(
                "{label}: the drive from {south:?} to {west:?} takes {minutes:.1} game minutes, more than {ACROSS_MOST}"
            ));
        }
        Ok(Some(minutes))
    });
    let mut failures: Vec<String> = results
        .iter()
        .filter_map(|r| r.as_ref().err().cloned())
        .collect();
    let mut minutes: Vec<f32> = results
        .iter()
        .filter_map(|r| r.as_ref().ok().copied().flatten())
        .collect();
    minutes.sort_by(f32::total_cmp);
    if let Some(&middle) = minutes.get(minutes.len() / 2) {
        eprintln!(
            "{} worlds driven across in {middle:.1} game minutes (the middle one), {:.1} at most",
            minutes.len(),
            minutes.last().copied().unwrap_or(0.0)
        );
        if middle > ACROSS_MINUTES {
            failures.push(format!(
                "the middle drive across takes {middle:.1} game minutes, more than {ACROSS_MINUTES}"
            ));
        }
    }
    for f in &failures {
        eprintln!("  {f}");
    }
    assert!(failures.is_empty());
}
