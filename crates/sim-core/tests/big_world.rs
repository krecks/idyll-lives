//! The big-world benchmark: a synthetic open world of hamlets (`common::hamlets`: streets of the
//! shipped houses, with the content's storeys and basement, and open country between them) as
//! big as a town file may be, with as many residents as a town holds. It measures what the open
//! world costs the simulation: how long the world takes to load, how much memory it holds (a
//! counting allocator), how fast it ticks with lower detail on and the view on one hamlet (ticks
//! per second, the 99th percentile and the slowest tick), what the searches cost (tiles expanded
//! per A* search), and how long the town takes to catch up after one object is placed.
//!
//! Timing depends on the machine, so it is opt-in and prints its numbers:
//! `cargo test --release -p sim-core --test big_world -- --ignored --nocapture`
//! with `SIZE=1024` (default 256), `RESIDENTS=160` (default: as many as a town holds) and
//! `HOURS=6` (game hours to run, default 4) in the environment.

mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use sim_core::World;
use sim_core::world::MAX_SIMS;

/// The system allocator, counting the bytes held now and at most.
struct Counting;

static HELD: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            let held = HELD.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(held, Ordering::Relaxed);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        HELD.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            if new_size >= layout.size() {
                let grown = new_size - layout.size();
                let held = HELD.fetch_add(grown, Ordering::Relaxed) + grown;
                PEAK.fetch_max(held, Ordering::Relaxed);
            } else {
                HELD.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        p
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

fn env(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn mb(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

#[test]
#[ignore]
fn big_world_benchmark() {
    let size = env("SIZE", 256) as i64;
    let residents = env("RESIDENTS", MAX_SIMS);
    let hours = env("HOURS", 4) as u64;
    let content = common::content();
    let (town, hamlets) = common::hamlets(&content, 7, size, 6, residents);

    // Loading: the town file to a world, as a new game does.
    let before = HELD.load(Ordering::Relaxed);
    PEAK.store(before, Ordering::Relaxed);
    let start = Instant::now();
    let mut w = World::from_json(&content, &town, 7).expect("the big world loads");
    let load = start.elapsed();
    let (held, peak) = (
        HELD.load(Ordering::Relaxed) - before,
        PEAK.load(Ordering::Relaxed) - before,
    );
    eprintln!(
        "{size}²: {} hamlets, {} plots, {} objects, {} residents, grid {}×{} ({} storeys{})",
        hamlets.len(),
        w.plots.len(),
        w.objects.len(),
        w.sims.len(),
        w.lot.width,
        w.lot.depth,
        w.storeys,
        if w.lowest_storey() < 0 {
            " and a basement"
        } else {
            ""
        },
    );
    eprintln!(
        "load {:.1} ms, heap {:.1} MB held after loading ({:.1} MB at most while loading)",
        ms(load),
        mb(held),
        mb(peak)
    );

    // Ticking: lower detail on, the view on the first hamlet (where the player lives).
    w.lod = true;
    w.set_view(Some(hamlets[0]));
    sim_core::path::take_search_stats();
    // The first tick finds the rooms, the walkable regions and the indexes from scratch: part of
    // starting the game, timed on its own.
    let t = Instant::now();
    w.tick_once();
    eprintln!(
        "first tick {:.1} ms (load and first tick {:.1} ms)",
        ms(t.elapsed()),
        ms(load + t.elapsed())
    );
    let ticks = hours * 60 * sim_core::TICKS_PER_MINUTE;
    let mut times = Vec::with_capacity(ticks as usize);
    let run = Instant::now();
    for _ in 0..ticks {
        let t = Instant::now();
        w.tick_once();
        times.push(t.elapsed());
    }
    let total = run.elapsed();
    let searches = sim_core::path::take_search_stats();
    let mut slowest: Vec<(Duration, usize)> = times.iter().copied().zip(0..).collect();
    slowest.sort_unstable_by(|a, b| b.cmp(a));
    eprintln!(
        "slowest ticks (ms, tick): {:?}",
        slowest[..5.min(slowest.len())]
            .iter()
            .map(|(d, t)| (format!("{:.2}", ms(*d)), *t))
            .collect::<Vec<_>>()
    );
    times.sort_unstable();
    let p99 = times[(times.len() * 99 / 100).min(times.len() - 1)];
    let max = *times.last().unwrap();
    eprintln!(
        "{ticks} ticks ({hours} game hours): {:.0} ticks/s, mean {:.3} ms, p99 {:.3} ms, max {:.3} ms",
        ticks as f64 / total.as_secs_f64(),
        ms(total) / ticks as f64,
        ms(p99),
        ms(max),
    );
    eprintln!(
        "A*: {} searches, {:.0} tiles each on average, {} at most",
        searches.searches,
        searches.nodes as f64 / searches.searches.max(1) as f64,
        searches.max_nodes
    );
    eprintln!(
        "heap {:.1} MB held after ticking ({:.1} MB at most)",
        mb(HELD.load(Ordering::Relaxed) - before),
        mb(PEAK.load(Ordering::Relaxed) - before)
    );

    let t = Instant::now();
    let json = w.save_json();
    let saved = t.elapsed();
    let t = Instant::now();
    World::from_save_json(&content, &json).expect("the save loads");
    eprintln!(
        "save {:.0} KB in {:.1} ms, loaded back in {:.1} ms",
        json.len() as f64 / 1024.0,
        ms(saved),
        ms(t.elapsed())
    );

    // One object placed in the player's garden, then what catching up costs: the walkable
    // regions, the objects-by-plot index and the links (`refresh_navigation`), and the rooms.
    let home = w
        .households
        .iter()
        .position(|h| h.player)
        .expect("a player household");
    let plot = &w.plots[w.households[home].plot.expect("a home") as usize];
    let spots: Vec<[i32; 3]> = (plot.z..plot.z + plot.d)
        .flat_map(|z| (plot.x..plot.x + plot.w).map(move |x| [x, z, 0]))
        .collect();
    let mut tried = None;
    let placed = spots.iter().any(|&at| {
        let r = w.apply(sim_core::Command::Buy {
            household: home as u32,
            object: "plant".into(),
            at: Some(at),
            colors: Default::default(),
            turn: None,
            offset: None,
            size: None,
            tall: None,
        });
        tried = r.as_ref().err().map(|e| e.to_string());
        r.is_ok()
    });
    if placed {
        let t = Instant::now();
        w.refresh_walkable();
        let reach = t.elapsed();
        let t = Instant::now();
        w.refresh_navigation();
        let index = t.elapsed();
        let t = Instant::now();
        w.refresh_rooms();
        let rooms = t.elapsed();
        eprintln!(
            "one object placed: walkable regions catch up in {:.3} ms, objects' index and links in {:.3} ms, rooms in {:.3} ms",
            ms(reach),
            ms(index),
            ms(rooms)
        );
    } else {
        eprintln!("one object placed: nowhere to put it ({tried:?})");
    }
}
