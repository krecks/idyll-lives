//! Tables and their chairs as separate pieces: a chair (content `seatAt`) facing a table is a
//! seat at that table, used from any free tile around it (`approach: "around"`); dining there
//! (`needsTable`) needs the table. A table for four is a table and four chairs.

mod common;

use sim_core::content::Pose;
use sim_core::path::NavGrid;
use sim_core::surface::{DiningRank, Links, dining_seat, faced_table, spot_for_seat};
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02},
             {"id":"comfort","label":"Comfort","decayPerHour":0.02}],
    "objects":[
      {"id":"table","name":"Table","price":200,"footprint":[2,1],"surface":{"height":0.75,"kind":"table"}},
      {"id":"bar","name":"Bar","price":200,"surface":{"height":1.0,"kind":"bar"}},
      {"id":"crate","name":"Crate","price":10},
      {"id":"chair","name":"Chair","price":60,"approach":"around","seatAt":["table"],"interactions":[
        {"id":"sit","label":"Sit","minutes":20,"pose":"sit","effects":{"comfort":0.3},"tags":["lounge"]},
        {"id":"dine","label":"Dine","minutes":30,"pose":"sit","needsTable":true,"dirt":0.3,
         "effects":{"hunger":0.6},"tags":["food"]}]},
      {"id":"stool","name":"Stool","price":40,"approach":"around","seatAt":["bar","counter"],"interactions":[
        {"id":"sit","label":"Sit","minutes":20,"pose":"sit","effects":{"comfort":0.2},"tags":["lounge"]}]},
      {"id":"diningSet","name":"Dining set","price":300,"footprint":[2,1],"slots":2,
       "surface":{"height":0.75,"kind":"table","size":[1.6,0.4],"at":[0,0.25]},"interactions":[
        {"id":"eat","label":"Eat","minutes":30,"pose":"sit","effects":{"hunger":0.5},"tags":["food","lounge"]}]},
      {"id":"sofa","name":"Sofa","price":100,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"comfort":0.4},"tags":["lounge"]}]},
      {"id":"longSofa","name":"Long sofa","price":150,"footprint":[3,1],"slots":3,"interactions":[
        {"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"comfort":0.4},"tags":["lounge"]}]},
      {"id":"bench","name":"Bench","price":150,"footprint":[4,1],"slots":4,"interactions":[
        {"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"comfort":0.4},"tags":["lounge"]}]},
      {"id":"toilet","name":"Toilet","price":100,"interactions":[
        {"id":"use","label":"Use","minutes":5,"pose":"sit","effects":{"comfort":0.1},"tags":["bathroom"]}]}],
    "economy":{"startingFunds":5000}}"#;

/// An open 16×14 plot with one resident.
const TOWN: &str = r#"{"width":16,"depth":16,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":14,"entry":[8.5,13.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "sims":[{"name":"Ada","household":0,"x":10.5,"z":10.5}]}"#;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.autonomy = false;
    w
}

/// Its index in `World::objects`, what the seat and table lookups here take (nothing is sold
/// before one is used as a stable id, in a command).
fn buy(w: &mut World, def: &str, x: i32, z: i32, rot: i32) -> Result<u32, sim_core::Error> {
    w.apply(Command::Buy {
        household: 0,
        object: def.into(),
        at: Some([x, z, rot]),
        colors: Default::default(),
        turn: None,
        offset: None,
        size: None,
        tall: None,
    })?;
    Ok(w.objects.len() as u32 - 1)
}

fn nav(w: &World) -> NavGrid<'_> {
    NavGrid {
        lot: &w.lot,
        blocked: w.blocked(),
        stairs: w.stair_map(),
    }
}

fn table_of(w: &World, seat: u32) -> Option<u32> {
    faced_table(&w.content, &w.objects, &nav(w), seat)
}

fn interaction(w: &World, object: u32, id: &str) -> usize {
    w.content.objects[w.objects[object as usize].def]
        .interactions
        .iter()
        .position(|i| i.id == id)
        .unwrap()
}

#[test]
fn chairs_pull_up_to_a_table_bought_before_or_after_them() {
    let mut w = world();
    let table = buy(&mut w, "table", 4, 4, 0).unwrap();
    let chair = buy(&mut w, "chair", 4, 3, 0).expect("a chair facing the table, used from behind");
    assert_eq!(table_of(&w, chair), Some(table));
    let side = buy(&mut w, "chair", 3, 4, 3).expect("one at the end of the table");
    assert_eq!(
        table_of(&w, side),
        None,
        "facing away from it: just a chair"
    );

    // Two chairs facing each other, then a table between them.
    let a = buy(&mut w, "chair", 9, 3, 0).unwrap();
    let b = buy(&mut w, "chair", 9, 5, 2).unwrap();
    assert_eq!((table_of(&w, a), table_of(&w, b)), (None, None));
    let t =
        buy(&mut w, "table", 9, 4, 0).expect("a table needs no free front: chairs go all round it");
    assert_eq!((table_of(&w, a), table_of(&w, b)), (Some(t), Some(t)));
    common::check(&w);
}

#[test]
fn a_chair_needs_one_free_side() {
    let mut w = world();
    buy(&mut w, "table", 4, 4, 0).unwrap();
    for (x, z) in [(3, 3), (5, 3), (4, 2)] {
        buy(&mut w, "crate", x, z, 0).unwrap();
    }
    let err = buy(&mut w, "chair", 4, 3, 0).unwrap_err().to_string();
    assert!(err.contains("block"), "boxed in: {err}");
    let back = w.objects.iter().position(|o| (o.x, o.z) == (4, 2)).unwrap() as u32;
    w.apply(Command::Sell {
        household: 0,
        object: w.objects[back as usize].stable,
    })
    .unwrap();
    let chair = buy(&mut w, "chair", 4, 3, 0).expect("free behind it");
    // Now nothing may take that last free side.
    assert!(
        buy(&mut w, "crate", 4, 2, 0).is_err(),
        "the chair's last way in stays free"
    );
    assert!(table_of(&w, chair).is_some());
}

/// Runs until the resident's pose changes from `from` (at most `limit` ticks).
fn until_pose_changes(w: &mut World, from: Pose, limit: u32) {
    for _ in 0..limit {
        w.tick_once();
        if w.sims[0].pose != from {
            return;
        }
    }
    panic!("still {from:?} after {limit} ticks");
}

#[test]
fn a_diner_comes_round_the_chair_sits_facing_the_table_and_gets_up_behind_it() {
    let mut w = world();
    buy(&mut w, "table", 4, 4, 0).unwrap();
    let chair = buy(&mut w, "chair", 4, 3, 0).unwrap();
    w.sims[0].needs[0] = 0.2;
    let dine = interaction(&w, chair, "dine");
    w.apply(Command::Use {
        sim: 0,
        object: chair,
        interaction: dine,
    })
    .unwrap();
    until_pose_changes(&mut w, Pose::Stand, 2000);
    let s = &w.sims[0];
    assert_eq!(s.pose, Pose::Sit);
    assert_eq!(s.pos, [4.5, 3.5], "on the chair");
    assert_eq!(s.yaw, 0.0, "facing the table");

    // Saved while seated, they come back standing where they sat down from: behind the chair.
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(loaded.sims[0].pos, [4.5, 2.5]);

    until_pose_changes(&mut w, Pose::Sit, 5000);
    assert_eq!(
        w.sims[0].pos,
        [4.5, 2.5],
        "up behind the chair, not on the table"
    );
    // The meal's mess is where they got up.
    let i = w.lot.tile_index(4, 2);
    assert!(w.dirt[i] > 0.0, "crumbs behind the chair");
    common::check(&w);
}

#[test]
fn dining_needs_a_table() {
    let mut w = world();
    let chair = buy(&mut w, "chair", 8, 8, 0).unwrap();
    let dine = interaction(&w, chair, "dine");
    let err = w
        .apply(Command::Use {
            sim: 0,
            object: chair,
            interaction: dine,
        })
        .unwrap_err()
        .to_string();
    assert!(err.contains("table"), "{err}");
    // Left to themselves, a hungry resident never dines at a chair in the middle of nowhere.
    w.autonomy = true;
    for _ in 0..3000 {
        w.sims[0].needs[0] = 0.1;
        w.tick_once();
        let dining = w.sims[0].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Use { object, interaction } if object == chair && interaction == dine));
        assert!(!dining);
    }
}

/// A dining set has its chairs built in: a chair bought facing it stands behind them, a metre
/// from the top, so it isn't pulled up to the set (I-office-01).
#[test]
fn a_chair_is_no_seat_at_a_dining_set_with_its_own_chairs() {
    let mut w = world();
    buy(&mut w, "diningSet", 4, 4, 0).unwrap();
    let table = buy(&mut w, "table", 9, 4, 0).unwrap();
    let at_set = buy(&mut w, "chair", 4, 3, 0).unwrap();
    let at_table = buy(&mut w, "chair", 9, 3, 0).unwrap();
    assert_eq!(table_of(&w, at_set), None);
    assert_eq!(table_of(&w, at_table), Some(table));
    let dine = interaction(&w, at_set, "dine");
    let err = w
        .apply(Command::Use {
            sim: 0,
            object: at_set,
            interaction: dine,
        })
        .unwrap_err()
        .to_string();
    assert!(err.contains("table"), "{err}");
}

#[test]
fn a_bar_stool_is_no_seat_at_a_dining_table() {
    let mut w = world();
    buy(&mut w, "table", 4, 4, 0).unwrap();
    let bar = buy(&mut w, "bar", 9, 4, 0).unwrap();
    let at_table = buy(&mut w, "stool", 4, 3, 0).unwrap();
    let at_bar = buy(&mut w, "stool", 9, 3, 0).unwrap();
    assert_eq!(table_of(&w, at_table), None);
    assert_eq!(table_of(&w, at_bar), Some(bar));
}

#[test]
fn diners_pick_a_chair_at_a_table_then_a_dining_set_then_anywhere_comfy() {
    let mut w = world();
    let table = buy(&mut w, "table", 2, 2, 0).unwrap();
    let chair = buy(&mut w, "chair", 2, 1, 0).unwrap();
    let set = buy(&mut w, "diningSet", 8, 2, 0).unwrap();
    let sofa = buy(&mut w, "sofa", 8, 8, 0).unwrap();
    let toilet = buy(&mut w, "toilet", 12, 2, 0).unwrap();
    let links = Links::new(&w.content, &w.objects, &nav(&w));
    // From right next to the sofa, the chair at the table is still the best place to eat.
    let from = [8.5, 9.5];
    let pick = |candidates: &[u32]| {
        dining_seat(&w.content, &w.objects, &links, candidates, from, 0).unwrap()
    };
    let all = [table, chair, set, sofa, toilet];
    let best = pick(&all);
    assert_eq!(
        (best.object, best.rank, best.table),
        (chair, DiningRank::ChairAtTable, Some(table))
    );
    assert_eq!(
        best.spot.map(|s| s.host),
        Some(table),
        "the plate goes on the table"
    );
    let next = pick(&[table, set, sofa, toilet]);
    assert_eq!((next.object, next.rank), (set, DiningRank::TableSeat));
    assert_eq!(next.spot.map(|s| s.host), Some(set), "on the set's own top");
    let last = pick(&[table, sofa, toilet]);
    assert_eq!((last.object, last.rank), (sofa, DiningRank::AnySeat));
    assert!(
        dining_seat(&w.content, &w.objects, &links, &[table, toilet], from, 0).is_none(),
        "not on the toilet"
    );
}

#[test]
fn four_chairs_round_a_table_each_get_their_own_place() {
    let mut w = world();
    let table = buy(&mut w, "table", 4, 4, 0).unwrap();
    let chairs = [(4, 3, 0), (5, 3, 0), (4, 5, 2), (5, 5, 2)]
        .map(|(x, z, r)| buy(&mut w, "chair", x, z, r).unwrap());
    let links = Links::new(&w.content, &w.objects, &nav(&w));
    let spots: Vec<_> = chairs
        .iter()
        .map(|&c| spot_for_seat(&w.content, &w.objects, &links, c, 0, 0.12, &[]).unwrap())
        .collect();
    for (i, s) in spots.iter().enumerate() {
        assert_eq!(s.host, table);
        let c = w.objects[chairs[i] as usize].centre(&w.content);
        let p = s.centre();
        assert!(
            (p[0] - c[0]).abs() < 0.05 && (p[1] - c[1]).abs() < 0.9,
            "in front of its chair: {p:?} {c:?}"
        );
        for t in &spots[..i] {
            let q = t.centre();
            assert!(
                (p[0] - q[0]).hypot(p[1] - q[1]) >= 0.24,
                "plates don't overlap: {p:?} {q:?}"
            );
        }
    }
    // With something already there, the plate goes beside it.
    let taken = [(spots[0].centre(), 0.12)];
    let moved = spot_for_seat(&w.content, &w.objects, &links, chairs[0], 0, 0.12, &taken).unwrap();
    assert!(
        (moved.centre()[0] - spots[0].centre()[0]).hypot(moved.centre()[1] - spots[0].centre()[1])
            >= 0.24
    );
}

#[test]
fn long_seats_spread_their_sitters_across_their_width() {
    let mut w = world();
    let long = buy(&mut w, "longSofa", 2, 2, 0).unwrap();
    let bench = buy(&mut w, "bench", 2, 6, 0).unwrap();
    for (seat, n) in [(long, 3usize), (bench, 4)] {
        let o = &w.objects[seat as usize];
        let xs: Vec<f32> = (0..n).map(|s| o.slot_position(&w.content, s)[0]).collect();
        for pair in xs.windows(2) {
            assert!(
                (pair[1] - pair[0] - 1.0).abs() < 1e-4,
                "a metre apart: {xs:?}"
            );
        }
        assert!(
            xs.iter()
                .all(|&x| x > o.x as f32 && x < (o.x + n as i32) as f32)
        );
    }
    // Three residents on the long sofa sit side by side.
    for i in 0..2 {
        let spawn: sim_core::lot::SimSpawn = serde_json::from_str(&format!(
            r#"{{"name":"S{i}","household":0,"x":{}.5,"z":10.5}}"#,
            4 + i
        ))
        .unwrap();
        w.spawn_sim(&spawn).unwrap();
    }
    let sit = interaction(&w, long, "sit");
    for sim in 0..3 {
        w.sims[sim as usize].needs[1] = 0.1;
        w.apply(Command::Use {
            sim,
            object: long,
            interaction: sit,
        })
        .unwrap();
    }
    for _ in 0..2000 {
        w.tick_once();
        if w.sims.iter().all(|s| s.pose == Pose::Sit) {
            break;
        }
    }
    let mut xs: Vec<f32> = w.sims.iter().map(|s| s.pos[0]).collect();
    xs.sort_by(f32::total_cmp);
    assert_eq!(xs, [2.5, 3.5, 4.5], "one to a place");
}

/// The shipped content on the open plot (resident 0 a bookworm, whose food does for them what
/// it does for anyone): a fridge, and maybe a dining table.
fn kitchen(table: bool) -> (World, u32, Option<u32>) {
    let town = TOWN.replace(r#""z":10.5}"#, r#""z":10.5,"traits":["bookworm"]}"#);
    let mut w = World::from_json(&common::content(), &town, 1).unwrap();
    w.autonomy = false;
    let fridge = buy(&mut w, "fridge", 2, 2, 0).unwrap();
    let table = table.then(|| buy(&mut w, "diningTable", 8, 6, 0).unwrap());
    (w, fridge, table)
}

#[test]
fn a_snack_is_eaten_sitting_at_the_table_if_there_is_one_else_standing() {
    for with_table in [true, false] {
        let (mut w, fridge, table) = kitchen(with_table);
        let hunger = w
            .content
            .needs
            .iter()
            .position(|n| n.id == "hunger")
            .unwrap();
        w.sims[0].needs[hunger] = 0.3;
        let snack = interaction(&w, fridge, "snack");
        let gain = w.content.objects[w.objects[fridge as usize].def].interactions[snack].total_gain
            [hunger];
        w.apply(Command::Use {
            sim: 0,
            object: fridge,
            interaction: snack,
        })
        .unwrap();
        // Fetched at the fridge, then eaten: sitting at the table with the plate on it, or
        // standing with it in hand.
        let mut eating = None;
        for _ in 0..4000 {
            w.tick_once();
            if let Some(a) = w.sims[0].current()
                && let TaskKind::Eat { seat, plate, .. } = a.task.kind
                && matches!(a.phase, Phase::Using { .. })
            {
                eating = Some((seat, plate));
                break;
            }
        }
        let (seat, plate) = eating.expect("eats the snack");
        if with_table {
            assert_eq!(seat, table, "at the table");
            assert_eq!(w.sims[0].pose, Pose::Sit);
            assert!(
                plate.is_some() && w.sims[0].held.is_none(),
                "the plate on the table"
            );
        } else {
            assert_eq!(seat, None, "nowhere to sit");
            assert_eq!(w.sims[0].pose, Pose::Stand);
            assert!(w.sims[0].held.is_some(), "the plate in hand");
            // The snack (a recipe made for the fridge's `eat`) is still food after a reload
            // (put down, as eating starts again from what's there).
            let loaded = World::from_save_json(&common::content(), &w.save_json()).unwrap();
            let recipe = w.sims[0].held.and_then(|h| h.recipe);
            assert!(recipe.is_some_and(|r| loaded.content.recipes[r].hidden));
            assert!(
                loaded
                    .items
                    .list
                    .iter()
                    .any(|it| it.recipe == recipe && it.edible()),
                "the snack is still there"
            );
        }
        for _ in 0..4000 {
            w.tick_once();
            if w.sims[0].current().is_none() {
                break;
            }
        }
        assert!(w.sims[0].current().is_none(), "done eating");
        assert!(
            w.sims[0].needs[hunger] > 0.3 + gain - 0.05,
            "the snack fed them: {}",
            w.sims[0].needs[hunger]
        );
        let dirty = w
            .items
            .list
            .iter()
            .filter(|it| w.content.items[it.def].dirty)
            .count();
        assert_eq!(dirty, 1, "a plate to wash up");
        common::check(&w);
    }
}
