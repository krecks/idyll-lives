//! A crowded garden (plan/future.md, Outdoor): eleven outdoor things bought side by side in a row,
//! facing the house or away from it. Each is still reached and used when a resident is told to,
//! from the free row in front of it, and two residents share what seats two.

mod common;

use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, TICKS_PER_MINUTE, World, clock};

const ROW: [&str; 11] = [
    "patioTableRound", "patioChair", "adirondackChair", "benchGarden3", "parasol", "outdoorBar",
    "sunLounger", "hammock", "porchSwing", "swingSet", "hotTubCedar",
];

/// A town with the player's household of two at noon, free will off, and the row bought side by
/// side in their garden facing `rot` (0 or 2), with two free rows on either side: saved, with the
/// content and the ids bought.
fn world(rot: i32) -> (String, String, Vec<u32>) {
    let content = common::content();
    let mut w = World::from_json(&content, &common::town(&content, 3, 2), 3).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    let noon = clock::tick_at(1, 12.0 * 60.0).unwrap();
    while w.tick < noon {
        w.tick_once();
    }
    let h = w.sims[0].household;
    w.households[h as usize].funds = 1_000_000;
    let p = w.plots[w.households[h as usize].plot.unwrap() as usize].clone();
    let width = |w: &World, id: &str| w.content.objects[w.content.object_index(id).unwrap()].footprint[0] as i32;
    let total: i32 = ROW.iter().map(|id| width(&w, id)).sum();
    let free = |w: &World, x: i32, z: i32| w.lot.room_at(x, z) == sim_core::lot::OUTDOORS && !w.blocked()[w.lot.tile_index(x, z)];
    let (mut x, z) = (p.z + 2..p.z + p.d - 3)
        .find_map(|z| (p.x..p.x + p.w - total).find(|&x| (x..x + total).all(|tx| (z - 2..z + 4).all(|tz| free(&w, tx, tz)))).map(|x| (x, z)))
        .expect("room for the row");
    let mut ids = Vec::new();
    for id in ROW {
        let r = w.apply(Command::Buy { household: h, object: id.into(), at: Some([x, z, rot]), colors: Default::default(), turn: None, offset: None, size: None, tall: None });
        assert!(r.is_ok(), "{id} at {x},{z}: {r:?}");
        ids.push(w.objects.len() as u32 - 1);
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
    def.interactions.iter().position(|i| i.autonomous && i.total_gain.iter().any(|g| *g > 0.0))
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
            w.apply(Command::Use { sim: 0, object: o, interaction: i }).unwrap();
            assert!(all_on(&mut w, &[0], o, 45).is_some(), "facing {rot}: the {} is never got to", ROW[k]);
        }
    }
}

#[test]
fn two_share_what_seats_two_in_a_crowded_row() {
    let (save, content, ids) = world(0);
    for (k, &o) in ids.iter().enumerate() {
        let mut w = World::from_save_json(&content, &save).unwrap();
        let both: Vec<usize> = (0..w.sims.len()).filter(|&s| w.sims[s].household == w.sims[0].household).take(2).collect();
        assert_eq!(both.len(), 2);
        let slots = w.content.objects[w.objects[o as usize].def].slots;
        let Some(i) = its_use(&w, o).filter(|_| slots >= 2) else { continue };
        let n = w.content.needs.len();
        for &s in &both {
            w.sims[s].needs[..n].fill(0.4);
            w.apply(Command::Use { sim: s as u32, object: o, interaction: i }).unwrap();
        }
        assert!(all_on(&mut w, &both, o, 45).is_some(), "both on the {}", ROW[k]);
    }
}
