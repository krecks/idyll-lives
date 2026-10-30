//! A crowded garden (docs/objects/open.md, Still open): eleven outdoor things bought side by side in a row,
//! facing the house or away from it. Each is still reached and used when a resident is told to,
//! from the free row in front of it, and two residents share what seats two.
//!
//! And a patch of garden crowded every way (I-garden-01, `common::garden_cluster`): rows with
//! one-tile paths and packs with none, all facing one way or any way. Whatever buying allowed
//! can be walked to, each use is got onto when told (by as many as it seats), and a day of free
//! will gets the household there. With a seat or a side shut in by the things round it they go
//! round to another tile it's used from.

mod common;

use sim_core::path::NavGrid;
use sim_core::surface;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, TICKS_PER_MINUTE, World, clock};

const ROW: [&str; 11] = [
    "patioTableRound",
    "patioChair",
    "adirondackChair",
    "benchGarden3",
    "parasol",
    "outdoorBar",
    "sunLounger",
    "hammock",
    "porchSwing",
    "swingSet",
    "hotTubCedar",
];

/// A town with the player's household of two at noon, free will off, and the row bought side by
/// side in their garden facing `rot` (0 or 2), with two free rows on either side: saved, with the
/// content and the ids bought.
fn world(rot: i32) -> (String, String, Vec<u32>) {
    let content = common::content();
    let mut w = World::from_json(&content, &common::town(&content, 3, 2), 3).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    let noon = clock::tick_at(1, 12.0 * 60.0).unwrap();
    while w.tick < noon {
        w.tick_once();
    }
    let h = w.sims[0].household;
    w.households[h as usize].funds = 1_000_000;
    let p = w.plots[w.households[h as usize].plot.unwrap() as usize].clone();
    let width = |w: &World, id: &str| {
        w.content.objects[w.content.object_index(id).unwrap()].footprint[0] as i32
    };
    let total: i32 = ROW.iter().map(|id| width(&w, id)).sum();
    let free = |w: &World, x: i32, z: i32| {
        w.lot.room_at(x, z) == sim_core::lot::OUTDOORS && !w.blocked()[w.lot.tile_index(x, z)]
    };
    let (mut x, z) = (p.z + 2..p.z + p.d - 3)
        .find_map(|z| {
            (p.x..p.x + p.w - total)
                .find(|&x| (x..x + total).all(|tx| (z - 2..z + 4).all(|tz| free(&w, tx, tz))))
                .map(|x| (x, z))
        })
        .expect("room for the row");
    let mut ids = Vec::new();
    for id in ROW {
        let r = w.apply(Command::Buy {
            household: h,
            object: id.into(),
            at: Some([x, z, rot]),
            colors: Default::default(),
            turn: None,
            offset: None,
            size: None,
            tall: None,
        });
        assert!(r.is_ok(), "{id} at {x},{z}: {r:?}");
        ids.push(w.objects.last().unwrap().stable);
        x += width(&w, id);
    }
    (w.save_json(), content, ids)
}

/// What resident `s` is using now (not walking to).
fn using(w: &World, s: usize) -> Option<u32> {
    w.sims[s].current().and_then(|a| match a.task.kind {
        TaskKind::Use { object, .. } if matches!(a.phase, Phase::Using { .. }) => Some(object),
        _ => None,
    })
}

/// The first interaction of `object` that fills a need (what it's for).
fn its_use(w: &World, object: u32) -> Option<usize> {
    let def = &w.content.objects[w.objects[object as usize].def];
    def.interactions
        .iter()
        .position(|i| i.autonomous && i.total_gain.iter().any(|g| *g > 0.0))
}

/// Ticks until every one of `sims` is using `object`; the minutes it took.
fn all_on(w: &mut World, sims: &[usize], object: u32, minutes: u64) -> Option<u64> {
    (0..minutes * TICKS_PER_MINUTE).find_map(|t| {
        let on = sims.iter().all(|&s| using(w, s) == Some(object));
        w.tick_once();
        on.then_some(t / TICKS_PER_MINUTE)
    })
}

#[test]
fn each_thing_in_a_crowded_row_is_used_when_told() {
    for rot in [0, 2] {
        let (save, content, ids) = world(rot);
        for (k, &o) in ids.iter().enumerate() {
            let mut w = World::from_save_json(&content, &save).unwrap();
            let Some(i) = its_use(&w, o) else { continue };
            let n = w.content.needs.len();
            w.sims[0].needs[..n].fill(0.5);
            w.apply(Command::Use {
                sim: 0,
                object: o,
                interaction: i,
            })
            .unwrap();
            assert!(
                all_on(&mut w, &[0], o, 45).is_some(),
                "facing {rot}: the {} is never got to",
                ROW[k]
            );
        }
    }
}

#[test]
fn two_share_what_seats_two_in_a_crowded_row() {
    let (save, content, ids) = world(0);
    for (k, &o) in ids.iter().enumerate() {
        let mut w = World::from_save_json(&content, &save).unwrap();
        let both: Vec<usize> = (0..w.sims.len())
            .filter(|&s| w.sims[s].household == w.sims[0].household)
            .take(2)
            .collect();
        assert_eq!(both.len(), 2);
        let slots = w.content.objects[w.objects[o as usize].def].slots;
        let Some(i) = its_use(&w, o).filter(|_| slots >= 2) else {
            continue;
        };
        let n = w.content.needs.len();
        for &s in &both {
            w.sims[s].needs[..n].fill(0.4);
            w.apply(Command::Use {
                sim: s as u32,
                object: o,
                interaction: i,
            })
            .unwrap();
        }
        assert!(
            all_on(&mut w, &both, o, 45).is_some(),
            "both on the {}",
            ROW[k]
        );
    }
}

/// `common::garden_home()` saved, with the content, the household and the patch to crowd.
fn garden() -> (String, String, u32, (i32, i32, i32, i32)) {
    let (world, content) = common::garden_home();
    (world.save_json(), content, 0, common::GARDEN_PATCH)
}

/// Whether a use tile of object `o` (`surface::use_tiles`) can be walked to from `from`.
fn reached(w: &World, o: usize, from: (i32, i32)) -> bool {
    let nav = NavGrid {
        lot: &w.lot,
        blocked: w.blocked(),
        stairs: w.stair_map(),
    };
    surface::use_tiles(&w.content, &w.objects, &nav, &w.objects[o])
        .into_iter()
        .any(|t| nav.find_path(from, t).is_some())
}

/// Residents `sims` are each told to use `interaction` of the object with stable id `object`
/// (needs at 0.4): whether they all get onto it within 45 minutes.
fn all_get_on(w: &mut World, sims: &[usize], object: u32, interaction: usize) -> bool {
    let n = w.content.needs.len();
    for &s in sims {
        w.sims[s].needs[..n].fill(0.4);
        if w.apply(Command::Use {
            sim: s as u32,
            object,
            interaction,
        })
        .is_err()
        {
            return false;
        }
    }
    let o = w.index_of(object).unwrap() as u32;
    all_on(w, sims, o, 45).is_some()
}

/// The ways a crowded garden is laid out and turned: rows with paths, facing each way, and
/// packs with no paths left on purpose, all one way or every way at random. (Before the way
/// round to another use tile, `start_task`, 15 of these left a lounger, a hammock, a bench, a
/// chair, a tree's shade, a birdbath, a picnic table or the fire pit's ring out of reach.)
fn clusters() -> Vec<(common::Cluster, Option<i32>)> {
    let mut all: Vec<_> = (0..4).map(|r| (common::Cluster::Rows, Some(r))).collect();
    all.extend((0..4).map(|r| (common::Cluster::Packed(11 + r as u64), Some(r))));
    all.extend((0..12).map(|k| (common::Cluster::Packed(200 + k), Some(k as i32 % 4))));
    all.extend((0..40).map(|k| (common::Cluster::Packed(31 + k), None)));
    all
}

#[test]
fn everything_in_a_packed_garden_is_reached() {
    let (empty, content, h, area) = garden();
    let mut failures = Vec::new();
    for (layout, rot) in clusters() {
        let mut w = World::from_save_json(&content, &empty).unwrap();
        let ids = common::garden_cluster(&mut w, h, area, layout, rot);
        let label = format!("{layout:?} facing {rot:?}");
        let used: Vec<u32> = ids
            .iter()
            .copied()
            .filter(|&s| {
                !w.content.objects[w.objects[w.index_of(s).unwrap()].def]
                    .interactions
                    .is_empty()
            })
            .collect();
        assert!(
            used.len() >= 12,
            "{label}: a crowded garden ({} things, {} with uses)",
            ids.len(),
            used.len()
        );
        let plot = &w.plots[w.households[h as usize].plot.unwrap() as usize];
        let door = plot.arrival_point();
        let from = (door[0].floor() as i32, door[1].floor() as i32);
        // What buying allowed can be walked to (as `generated_worlds.rs` checks).
        for &s in &used {
            let o = w.index_of(s).unwrap();
            if !reached(&w, o, from) {
                let ob = &w.objects[o];
                failures.push(format!(
                    "{label}: the {} at {},{} facing {} can't be walked to",
                    w.content.objects[ob.def].id, ob.x, ob.z, ob.rot
                ));
            }
        }
        // And in play: each use when told, by one resident, and by as many as it seats at once.
        let save = w.save_json();
        let home: Vec<usize> = (0..w.sims.len())
            .filter(|&s| w.sims[s].household == h)
            .collect();
        for &s in &used {
            let o = w.index_of(s).unwrap();
            let (def, x, z, r) = (
                w.objects[o].def,
                w.objects[o].x,
                w.objects[o].z,
                w.objects[o].rot,
            );
            let d = &w.content.objects[def];
            let slots = (d.slots as usize).min(home.len());
            let group = d
                .interactions
                .iter()
                .position(|i| i.autonomous && i.total_gain.iter().any(|g| *g > 0.0) && !i.partner);
            for i in 0..d.interactions.len() {
                let mut tries = vec![vec![home[0]]];
                if Some(i) == group && slots >= 2 {
                    tries.push(home[..slots].to_vec());
                }
                for who in tries {
                    let mut played = World::from_save_json(&content, &save).unwrap();
                    if all_get_on(&mut played, &who, s, i) {
                        continue;
                    }
                    // Not the crowding's doing if it isn't got onto on its own either (not ripe,
                    // not for their age, nothing to do yet).
                    let mut alone = World::from_save_json(&content, &empty).unwrap();
                    let bought = alone.apply(Command::Buy {
                        household: h,
                        object: d.id.clone(),
                        at: Some([x, z, r as i32]),
                        colors: Default::default(),
                        turn: None,
                        offset: None,
                        size: None,
                        tall: None,
                    });
                    if bought.is_ok() {
                        let one = alone.objects.last().unwrap().stable;
                        if all_get_on(&mut alone, &who, one, i) {
                            failures.push(format!("{label}: {} told to {} at the {} at {x},{z} facing {r}: never got on", who.len(), d.interactions[i].id, d.id));
                        }
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} in a crowded garden:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn a_day_of_free_will_in_a_packed_garden() {
    let (empty, content, h, area) = garden();
    let mut report = Vec::new();
    for (layout, rot) in [
        (common::Cluster::Packed(31), None),
        (common::Cluster::Packed(41), None),
        (common::Cluster::Rows, Some(1)),
    ] {
        let mut w = World::from_save_json(&content, &empty).unwrap();
        let ids = common::garden_cluster(&mut w, h, area, layout, rot);
        let cluster: std::collections::HashSet<u32> =
            ids.iter().map(|&s| w.index_of(s).unwrap() as u32).collect();
        w.apply(Command::SetAutonomy {
            enabled: true,
            household: None,
        })
        .unwrap();
        let home: Vec<usize> = (0..w.sims.len())
            .filter(|&s| w.sims[s].household == h)
            .collect();
        // Who is on the way to what in the garden, since when; the walks that never got there.
        let mut walking: Vec<Option<(u32, u64)>> = vec![None; w.sims.len()];
        let (mut got, mut gave_up) = (0, Vec::new());
        let end = w.tick + 24 * 60 * TICKS_PER_MINUTE;
        while w.tick < end {
            for &s in &home {
                let now = w.sims[s].current().and_then(|a| match a.task.kind {
                    TaskKind::Use { object, .. } if cluster.contains(&object) => {
                        Some((object, matches!(a.phase, Phase::Using { .. })))
                    }
                    _ => None,
                });
                match (walking[s], now) {
                    (Some((o, _)), Some((n, true))) if o == n => {
                        got += 1;
                        walking[s] = None;
                    }
                    (Some((o, since)), next) if next.is_none_or(|(n, _)| n != o) => {
                        let ob = &w.objects[o as usize];
                        let what = w.sims[s].current().map(|a| format!("{:?}", a.task.kind));
                        gave_up.push(format!("resident {s} set off for the {} at {},{} facing {} at {since}, stopped at {} for {what:?}", w.content.objects[ob.def].id, ob.x, ob.z, ob.rot, w.tick));
                        walking[s] = None;
                    }
                    (None, Some((n, false))) => walking[s] = Some((n, w.tick)),
                    _ => {}
                }
            }
            w.tick_once();
        }
        report.push(format!(
            "{layout:?} facing {rot:?}: {got} walks to a garden thing got there, {} didn't:\n  {}",
            gave_up.len(),
            gave_up.join("\n  ")
        ));
        assert!(
            got >= 4 && gave_up.len() * 10 <= got,
            "the household uses the garden in a day, and gets there: {}",
            report.last().unwrap()
        );
    }
    println!("{}", report.join("\n"));
}
