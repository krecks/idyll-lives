//! The countryside (web/public/content/countryside.json): residents of the hamlet nearby use its
//! spots, a walk from the trailhead goes out along the trail and back, and the pond in the
//! terrain's water mask can't be walked on, whatever stands on it or is taken away.

use crate::common;

use common::hamlets::{POND, trail, two_hamlets, with_countryside};
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, MINUTES_PER_TICK, World};

const TICKS_PER_MINUTE: u64 = common::TICKS_PER_MINUTE;

fn world(seed: u64) -> World {
    let content = common::content();
    let h = two_hamlets(&content, seed);
    World::from_json(&content, &with_countryside(&h.town), seed as u32).expect("the town loads")
}

fn countryside(w: &World, object: u32) -> Option<&str> {
    let id = w.content.objects[w.objects[object as usize].def]
        .id
        .as_str();
    id.starts_with("countryside.").then_some(id)
}

#[test]
fn residents_use_the_countryside_spots() {
    // Over three days in each of four towns: the residents of three of them at least go out to the
    // countryside, and between them to at least three different spots. (One town alone is a coin
    // toss: whether and which spots its handful of residents fancy in three days turns on any
    // change to their timing, as their walks to work.)
    let mut all = std::collections::BTreeSet::new();
    let mut towns = 0;
    for seed in 41..45 {
        let mut w = world(seed);
        let mut used = std::collections::BTreeSet::new();
        let end = w.tick + 3 * 24 * 60 * TICKS_PER_MINUTE;
        while w.tick < end {
            w.tick_once();
            for s in &w.sims {
                if let Some(a) = s.current()
                    && let (TaskKind::Use { object, .. }, Phase::Using { .. }) =
                        (a.task.kind, &a.phase)
                    && let Some(id) = countryside(&w, object)
                {
                    used.insert(id.to_string());
                }
            }
        }
        eprintln!("town {seed}: countryside spots used over three days: {used:?}");
        towns += usize::from(!used.is_empty());
        all.extend(used);
    }
    assert!(
        towns >= 3,
        "only {towns} of 4 towns went out to the countryside"
    );
    assert!(all.len() >= 3, "only {all:?}");
}

#[test]
fn a_trail_walk_goes_along_the_trail_and_back() {
    let mut w = world(42);
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    let head = w
        .objects
        .iter()
        .position(|o| w.content.objects[o.def].id == "countryside.trailhead")
        .unwrap() as u32;
    let walk = common::interaction(&w, head, "walk");
    w.apply(Command::Use {
        sim: 0,
        object: head,
        interaction: walk,
    })
    .unwrap();
    let pts: Vec<[f32; 2]> = trail().iter().map(|p| [p[0] as f32, p[1] as f32]).collect();
    let off_trail = |p: [f32; 2]| {
        pts.windows(2)
            .map(|s| sim_core::travel::line_distance(p, s[0], s[1]))
            .fold(f32::INFINITY, f32::min)
    };
    let (mut walked, mut farthest, mut start) = (0u64, 0.0f32, None);
    for _ in 0..(3 * 60 * TICKS_PER_MINUTE) {
        w.tick_once();
        let s = &w.sims[0];
        let using = s.current().is_some_and(|a| {
            matches!(
                (a.task.kind, &a.phase),
                (TaskKind::Use { object, .. }, Phase::Using { .. }) if object == head
            )
        });
        if using {
            // (From their first step on: they got there at the signpost beside it.)
            if walked > 0 {
                assert!(s.is_moving(), "walking the trail shows as walking");
                assert!(off_trail(s.pos) < 0.5, "off the trail at {:?}", s.pos);
            }
            let at = *start.get_or_insert(s.pos);
            farthest = farthest.max(sim_core::travel::dist(at, s.pos));
            walked += 1;
        } else if walked > 0 {
            break;
        }
    }
    let minutes = walked as f32 * MINUTES_PER_TICK;
    eprintln!("walked the trail {minutes:.0} min, at most {farthest:.0} m from the start");
    assert!(minutes >= 55.0, "the walk ended after {minutes:.0} minutes");
    assert!(farthest > 20.0, "hardly went anywhere ({farthest:.0} m)");
    // Back where they set off.
    let back = sim_core::travel::dist(start.unwrap(), w.sims[0].pos);
    assert!(back < 2.0, "{back:.1} m from where the walk started");
}

#[test]
fn the_pond_stays_water() {
    // With a wet tile in the player's back garden too.
    let content = common::content();
    let h = two_hamlets(&content, 43);
    let mut town: serde_json::Value = serde_json::from_str(&with_countryside(&h.town)).unwrap();
    let wet = (10i64, 9i64);
    let width = town["width"].as_i64().unwrap();
    let i = wet.1 * width + wet.0;
    let rle: Vec<i64> = town["terrain"]["water"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    // (The first run is dry ground up to the pond: split it round the garden's tile.)
    assert_eq!(rle[1], 0);
    let mut runs = vec![i, 0, 1, 1, rle[0] - i - 1, 0];
    runs.extend_from_slice(&rle[2..]);
    town["terrain"]["water"] = serde_json::json!(runs);
    let mut w = World::from_json(&content, &town.to_string(), 43).expect("the town loads");
    let blocked = |w: &World, (x, z): (i64, i64)| w.blocked()[w.lot.tile_index(x as i32, z as i32)];
    assert!(POND.iter().all(|&t| blocked(&w, t)), "the pond is walkable");
    assert!(!blocked(&w, (103, 50)), "beside the pond is dry");
    assert_eq!(w.plot_at(wet.0 as i32, wet.1 as i32), Some(0));
    assert!(blocked(&w, wet), "the garden's pond is walkable");
    // Nothing can be put in it; next to it, yes, and taken away again it leaves the water.
    let bought = |w: &mut World, x: i64| {
        w.apply(Command::Buy {
            household: 0,
            object: "bush".into(),
            at: Some([x as i32, wet.1 as i32, 0]),
            colors: Default::default(),
            turn: None,
            offset: None,
            size: None,
            tall: None,
        })
    };
    assert!(
        bought(&mut w, wet.0).is_err(),
        "a shrub planted in the water"
    );
    bought(&mut w, wet.0 + 1).expect("a shrub beside the water");
    let bush = w.objects.len() as u32 - 1;
    w.apply(Command::Sell {
        household: 0,
        object: bush,
    })
    .unwrap();
    w.tick_once();
    assert!(blocked(&w, wet) && !blocked(&w, (wet.0 + 1, wet.1)));
    // And after a save and a load.
    let w = common::reload(&w, &content);
    assert!(
        POND.iter().all(|&t| blocked(&w, t)),
        "the pond is gone after a load"
    );
    assert!(blocked(&w, wet));
}
