//! Clearing away (the base game's furniture): a resident fetches food and dishes left on the
//! dining table, scrapes the food off into a bin (or at the sink), washes up and puts the clean
//! things away at a counter. The player's "Clear away" and washing up of their own accord go
//! the same way.

use crate::common;

use sim_core::command::ItemAction;
use sim_core::items::{ItemAt, ItemInstance};
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World};

/// A house (walls round 0..10 × 0..8, a door at the bottom) with a kitchen along the back wall
/// and a table for four with a chair on each side (as the house generator sets one), on a
/// 16 × 14 plot.
fn town(kitchen: &str) -> String {
    format!(
        r#"{{"width":16,"depth":16,
        "walls":[[0,0,10,0],[0,8,10,8],[0,0,0,8],[10,0,10,8]],
        "doors":[{{"x":5,"z":8,"axis":"x"}}],
        "plots":[{{"name":"Home","x":0,"z":0,"w":16,"d":14,"entry":[8.5,12.5]}}],
        "households":[{{"name":"Player","plot":0,"player":true}}],
        "objects":[{kitchen},
            {{"def":"tableDining4","x":3,"z":5,"rot":0}},
            {{"def":"chairWood","x":3,"z":4,"rot":0}},{{"def":"chairWood","x":4,"z":4,"rot":0}},
            {{"def":"chairWood","x":3,"z":6,"rot":2}},{{"def":"chairWood","x":4,"z":6,"rot":2}}],
        "sims":[{{"name":"Amara","household":0,"x":7.5,"z":6.5,"traits":["cheerful"]}},
                {{"name":"Ben","household":0,"x":8.5,"z":6.5,"traits":["bookworm"]}}]}}"#
    )
}

const KITCHEN: &str = r#"{"def":"fridge","x":1,"z":1,"rot":0},{"def":"counter","x":2,"z":1,"rot":0},
    {"def":"stoveElectric","x":3,"z":1,"rot":0},{"def":"sink","x":6,"z":1,"rot":0}"#;

/// Object ids in the town (the kitchen's first).
const COUNTER: u32 = 1;
const SINK: u32 = 3;
/// With a pedal bin after the sink.
const BIN: u32 = 4;

fn world_with(content: &str, kitchen: &str) -> World {
    let mut w = World::from_json(content, &town(kitchen), 7).expect("town loads");
    w.autonomy = false;
    for s in &mut w.sims {
        s.needs = [1.0; 8];
    }
    w
}

fn world() -> World {
    world_with(&common::content(), KITCHEN)
}

/// The dining table (after the kitchen's `kitchen` objects).
fn table(w: &World) -> u32 {
    w.objects
        .iter()
        .find(|o| w.content.objects[o.def].id == "tableDining4")
        .unwrap()
        .id
}

/// Puts an item of `def` on `table` (a free spot): `servings` of the base game's meal, made at
/// tick `made_at`.
fn put_on(w: &mut World, table: u32, def: &str, servings: u8, made_at: u64) -> u32 {
    let def = w.content.item_index(def).unwrap();
    let radius = w.content.items[def].radius;
    let occupied = w.items.occupancy(&w.content);
    let centre = w.objects[table as usize].centre(&w.content);
    let spot = sim_core::surface::free_surface_spot_near(
        &w.content,
        &w.objects,
        &[table],
        centre,
        3.0,
        0,
        radius,
        &occupied,
    )
    .expect("room on the table");
    let recipe = (servings > 0).then(|| {
        w.content
            .recipes
            .iter()
            .position(|r| r.id == "meal")
            .unwrap()
    });
    w.items.add(ItemInstance {
        id: 0,
        def,
        at: ItemAt::on(&spot),
        servings,
        quality: 1.0,
        made_at,
        household: 0,
        spoiled: false,
        recipe,
    })
}

fn ticks(w: &mut World, minutes: f32) {
    for _ in 0..common::mins(minutes) {
        w.tick_once();
        common::check(w);
    }
}

/// Runs for up to `minutes` while `sim` clears away; returns the steps done (stage, station),
/// in order. They're done when they've stopped clearing away.
fn clearing(w: &mut World, sim: usize, minutes: f32) -> Vec<(u8, u32)> {
    let mut steps: Vec<(u8, u32)> = Vec::new();
    for _ in 0..common::mins(minutes) {
        w.tick_once();
        common::check(w);
        match w.sims[sim].current() {
            Some(a) => {
                if let (
                    TaskKind::Dishes {
                        stage,
                        station: Some(o),
                        ..
                    },
                    Phase::Using { .. },
                ) = (a.task.kind, &a.phase)
                    && steps.last() != Some(&(stage, o))
                {
                    steps.push((stage, o));
                }
            }
            None if w.sims[sim].queue().next().is_none() => return steps,
            None => {}
        }
    }
    panic!(
        "{} is still clearing away: {:?}",
        w.sims[sim].name,
        w.sims[sim].current().map(|a| a.task.kind)
    );
}

#[test]
fn clear_away_takes_gone_off_food_and_plates_off_a_table_with_chairs_all_round() {
    // The player's report: a pot gone off and plates on the dining table; "Clear away" did
    // nothing (the table's front, where they stood to reach it, has a chair in it).
    let mut w = world();
    let table = table(&w);
    let pot = put_on(&mut w, table, "mealPot", 2, 0);
    let plates = [
        put_on(&mut w, table, "mealPlate", 1, 0),
        put_on(&mut w, table, "plateDirty", 0, 0),
    ];
    ticks(&mut w, 9.0 * 60.0);
    assert!(w.items.get(pot).unwrap().spoiled, "gone off");
    for s in &mut w.sims {
        s.needs = [1.0; 8];
    }
    w.apply(Command::UseItem {
        sim: 0,
        item: pot,
        action: ItemAction::Clear,
    })
    .unwrap();
    w.tick_once();
    assert!(
        w.sims[0]
            .current()
            .is_some_and(|a| matches!(a.task.kind, TaskKind::Dishes { stage: 0, .. })),
        "on the way to the table: {:?}",
        w.sims[0].current().map(|a| a.task.kind)
    );
    // No bin: scraped and washed at the sink, put away in the counter's cupboard; one kind to an
    // armful, so the pot first, then the plates.
    assert_eq!(
        clearing(&mut w, 0, 120.0),
        [(2, SINK), (3, COUNTER), (2, SINK), (3, COUNTER)]
    );
    assert!(
        w.items.get(pot).is_none() && plates.iter().all(|&p| w.items.get(p).is_none()),
        "the pot and plates went with them"
    );
    assert!(
        w.items.list.is_empty(),
        "nothing put down again: {:?}",
        w.items.list
    );
    assert_eq!(w.sims[0].held, None, "put away");
}

#[test]
fn full_food_leftovers_and_empty_plates_are_all_cleared_and_scraped_into_a_bin() {
    let kitchen = format!(r#"{KITCHEN},{{"def":"trashCan","x":7,"z":1,"rot":0}}"#);
    let mut w = world_with(&common::content(), &kitchen);
    let table = table(&w);
    // A pot nobody touched, a plate left half eaten till it went off, an empty one.
    let pot = put_on(&mut w, table, "mealPot", 3, 0);
    let plate = put_on(&mut w, table, "mealPlate", 1, 0);
    w.items.get_mut(plate).unwrap().spoiled = true;
    put_on(&mut w, table, "plateDirty", 0, 0);
    w.apply(Command::UseItem {
        sim: 0,
        item: pot,
        action: ItemAction::Clear,
    })
    .unwrap();
    let (mut scraped, mut scraps_by_trip) = (0.0, Vec::new());
    let mut steps = Vec::new();
    for _ in 0..common::mins(120.0) {
        w.tick_once();
        common::check(&w);
        let Some(a) = w.sims[0].current() else {
            if w.sims[0].queue().next().is_none() {
                break;
            }
            continue;
        };
        if let (
            TaskKind::Dishes {
                stage,
                station: Some(o),
                scraps,
                ..
            },
            Phase::Using { elapsed },
        ) = (a.task.kind, &a.phase)
        {
            if steps.last() != Some(&(stage, o)) {
                steps.push((stage, o));
                if stage == 1 {
                    scraps_by_trip.push(scraps);
                }
            }
            if stage == 1 {
                scraped += sim_core::MINUTES_PER_TICK;
                let _ = elapsed;
            }
        }
    }
    // One kind to an armful: the pot (its food scraped off), then the plates (the one with food on it).
    assert_eq!(
        scraps_by_trip,
        [1, 1],
        "the pot, then the plate with food on it"
    );
    assert_eq!(
        steps,
        [
            (1, BIN),
            (2, SINK),
            (3, COUNTER),
            (1, BIN),
            (2, SINK),
            (3, COUNTER)
        ],
        "the bin, the sink, the cupboard, twice"
    );
    assert!(
        scraped >= 2.0 * w.content.cooking.scrape_minutes - 0.1,
        "scraped both: {scraped}"
    );
    assert!(w.items.list.is_empty(), "all cleared: {:?}", w.items.list);
}

#[test]
fn eating_from_a_table_with_chairs_all_round() {
    let mut w = world();
    let table = table(&w);
    let pot = put_on(&mut w, table, "mealPot", 2, 0);
    w.sims[0].needs[0] = 0.3;
    w.apply(Command::UseItem {
        sim: 0,
        item: pot,
        action: ItemAction::Eat,
    })
    .unwrap();
    ticks(&mut w, 60.0);
    assert_eq!(
        w.items.get(pot).map(|it| it.servings),
        Some(1),
        "took a serving"
    );
    assert!(w.sims[0].needs[0] > 0.8, "ate: {}", w.sims[0].needs[0]);
}

#[test]
fn no_sink_no_clearing_away() {
    let kitchen = r#"{"def":"fridge","x":1,"z":1,"rot":0},{"def":"counter","x":2,"z":1,"rot":0},
        {"def":"stoveElectric","x":3,"z":1,"rot":0}"#;
    let mut w = world_with(&common::content(), kitchen);
    let table = table(&w);
    let pot = put_on(&mut w, table, "mealPot", 2, 0);
    let err = w
        .apply(Command::UseItem {
            sim: 0,
            item: pot,
            action: ItemAction::Clear,
        })
        .unwrap_err();
    assert!(err.to_string().contains("nowhere to wash up"), "{err}");
}

#[test]
fn with_no_sink_dirty_things_go_in_the_bin() {
    let kitchen = r#"{"def":"fridge","x":1,"z":1,"rot":0},{"def":"counter","x":2,"z":1,"rot":0},
        {"def":"stoveElectric","x":3,"z":1,"rot":0},{"def":"trashCan","x":7,"z":1,"rot":0}"#;
    let bin = 3;
    // Told to.
    let mut w = world_with(&common::content(), kitchen);
    let at = table(&w);
    let pot = put_on(&mut w, at, "mealPot", 2, 0);
    put_on(&mut w, at, "plateDirty", 0, 0);
    w.apply(Command::UseItem {
        sim: 0,
        item: pot,
        action: ItemAction::Clear,
    })
    .unwrap();
    assert_eq!(clearing(&mut w, 0, 120.0), [(1, bin)]);
    assert!(
        w.items.list.is_empty(),
        "nothing put down again: {:?}",
        w.items.list
    );
    assert_eq!(w.sims[0].held, None);
    // Of their own accord.
    let mut w = world_with(&common::content(), kitchen);
    w.autonomy = true;
    let at = table(&w);
    for _ in 0..3 {
        put_on(&mut w, at, "plateDirty", 0, 0);
    }
    for _ in 0..common::hours(4.0) {
        w.tick_once();
        common::check(&w);
        if !w.items.list.iter().any(|it| it.to_clear(&w.content))
            && w.sims.iter().all(|s| s.held.is_none())
        {
            break;
        }
    }
    let dirty: Vec<_> = w
        .items
        .list
        .iter()
        .filter(|it| it.to_clear(&w.content))
        .collect();
    assert!(dirty.is_empty(), "the plates were cleared: {dirty:?}");
}

#[test]
fn residents_clear_away_of_their_own_accord_the_same_way() {
    let mut w = world();
    w.autonomy = true;
    let table = table(&w);
    for _ in 0..3 {
        put_on(&mut w, table, "plateDirty", 0, 0);
    }
    let mut steps = Vec::new();
    for _ in 0..common::hours(6.0) {
        w.tick_once();
        common::check(&w);
        for s in &w.sims {
            if let Some(a) = s.current()
                && let (
                    TaskKind::Dishes {
                        stage,
                        station: Some(o),
                        ..
                    },
                    Phase::Using { .. },
                ) = (a.task.kind, &a.phase)
                && steps.last() != Some(&(stage, o))
            {
                steps.push((stage, o));
            }
        }
        let dirty = w.items.list.iter().any(|it| it.to_clear(&w.content));
        if !dirty && w.sims.iter().all(|s| s.held.is_none()) && steps.last() == Some(&(3, COUNTER))
        {
            break;
        }
    }
    // (They may have cooked meanwhile: food is left for later.)
    let dirty: Vec<_> = w
        .items
        .list
        .iter()
        .filter(|it| it.to_clear(&w.content))
        .collect();
    assert!(dirty.is_empty(), "the plates were cleared: {dirty:?}");
    assert!(
        steps.windows(2).any(|s| s == [(2, SINK), (3, COUNTER)]),
        "washed up and put away: {steps:?}"
    );
}

#[test]
fn clean_things_go_in_a_wall_cabinet_over_the_counter_and_the_nearest_cupboard_or_counter() {
    // A wall cabinet over the counter is as near as the counter: the cupboard (`putAwayAt`'s
    // first) has them, and its doors open (`fetch`).
    // (The kitchen against the back wall, for the wall cabinet to hang on.)
    let kitchen = format!(
        r#"{},{{"def":"wallCabinet","x":2,"z":0,"rot":0}}"#,
        KITCHEN.replace(r#""z":1"#, r#""z":0"#)
    );
    let cupboard = 4;
    let mut w = world_with(&common::content(), &kitchen);
    let at = table(&w);
    let pot = put_on(&mut w, at, "mealPot", 2, 0);
    w.apply(Command::UseItem {
        sim: 0,
        item: pot,
        action: ItemAction::Clear,
    })
    .unwrap();
    assert_eq!(clearing(&mut w, 0, 120.0), [(2, SINK), (3, cupboard)]);
    assert_eq!(w.sims[0].held, None, "put away");
    // A china cabinet across the kitchen is further than the counter: the counter has them.
    let kitchen = format!(r#"{KITCHEN},{{"def":"chinaCabinet","x":0,"z":1,"rot":0}}"#);
    let mut w = world_with(&common::content(), &kitchen);
    let at = table(&w);
    let pot = put_on(&mut w, at, "mealPot", 2, 0);
    w.apply(Command::UseItem {
        sim: 0,
        item: pot,
        action: ItemAction::Clear,
    })
    .unwrap();
    assert_eq!(clearing(&mut w, 0, 120.0), [(2, SINK), (3, COUNTER)]);
    // A sideboard by the sink is nearer: it has them.
    let kitchen = format!(r#"{KITCHEN},{{"def":"sideboard","x":7,"z":1,"rot":0}}"#);
    let sideboard = 4;
    let mut w = world_with(&common::content(), &kitchen);
    let at = table(&w);
    let pot = put_on(&mut w, at, "mealPot", 2, 0);
    w.apply(Command::UseItem {
        sim: 0,
        item: pot,
        action: ItemAction::Clear,
    })
    .unwrap();
    assert_eq!(clearing(&mut w, 0, 120.0), [(2, SINK), (3, sideboard)]);
}

#[test]
fn a_child_leaves_the_clean_things_on_the_counter_under_a_wall_cabinet_out_of_reach() {
    // The wall cabinet's bottom shelf (`reach` 1.47 m) is over a child's head (a hand up to
    // about 1.3 m): the counter under it has them. A teenager reaches it.
    let kitchen = format!(
        r#"{},{{"def":"wallCabinet","x":2,"z":0,"rot":0}}"#,
        KITCHEN.replace(r#""z":1"#, r#""z":0"#)
    );
    let cupboard = 4;
    for (age, put) in [(8.0, COUNTER), (15.0, cupboard)] {
        let mut w = world_with(&common::content(), &kitchen);
        w.sims[0].age = age;
        let at = table(&w);
        let pot = put_on(&mut w, at, "mealPot", 2, 0);
        // (After school.)
        w.tick = sim_core::clock::tick_at(0, 17.0 * 60.0).unwrap();
        w.apply(Command::UseItem {
            sim: 0,
            item: pot,
            action: ItemAction::Clear,
        })
        .unwrap();
        assert_eq!(
            clearing(&mut w, 0, 120.0),
            [(2, SINK), (3, put)],
            "age {age}"
        );
        assert_eq!(w.sims[0].held, None, "age {age}: put away");
    }
}

#[test]
fn an_armful_of_plates_is_a_stack_picked_up_bent_over_the_table() {
    let mut w = world();
    let table = table(&w);
    let plates = [
        put_on(&mut w, table, "plateDirty", 0, 0),
        put_on(&mut w, table, "plateDirty", 0, 0),
        put_on(&mut w, table, "plateDirty", 0, 0),
    ];
    w.apply(Command::UseItem {
        sim: 0,
        item: plates[0],
        action: ItemAction::Clear,
    })
    .unwrap();
    // At the table a while (picking them up: `pickUpMinutes`), nothing in hand yet...
    let mut picking = 0;
    for _ in 0..common::mins(30.0) {
        w.tick_once();
        let s = &w.sims[0];
        match s.current().map(|a| (a.task.kind, &a.phase)) {
            Some((TaskKind::Dishes { stage: 0, .. }, Phase::Using { .. })) => {
                assert_eq!(s.held, None, "not picked up yet");
                picking += 1;
            }
            _ if s.held.is_some() => break,
            _ => {}
        }
    }
    assert!(picking > 0, "bent over the table a while first");
    // ...then all three in hand, as a stack.
    let held = w.sims[0].held.expect("carrying them");
    assert_eq!(w.content.items[held.def].id, "plateStackDirty");
    assert_eq!(held.servings, 3, "three to wash");
    assert!(plates.iter().all(|&p| w.items.get(p).is_none()));
}

#[test]
fn a_stack_of_plates_put_down_part_way_is_so_many_plates() {
    let mut w = world();
    let table = table(&w);
    for _ in 0..3 {
        put_on(&mut w, table, "plateDirty", 0, 0);
    }
    let first = w.items.list[0].id;
    w.apply(Command::UseItem {
        sim: 0,
        item: first,
        action: ItemAction::Clear,
    })
    .unwrap();
    for _ in 0..common::mins(30.0) {
        w.tick_once();
        if w.sims[0].held.is_some() {
            break;
        }
    }
    assert!(w.sims[0].held.is_some(), "carrying the stack");
    // Called away with it in hand: put down where they are, as the plates it's a stack of.
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    for _ in 0..common::TICKS_PER_MINUTE {
        w.tick_once();
    }
    assert_eq!(w.sims[0].held, None, "put down");
    let plate = w.content.item_index("plateDirty").unwrap();
    assert_eq!(w.items.list.len(), 3, "three: {:?}", w.items.list);
    assert!(
        w.items.list.iter().all(|it| it.def == plate),
        "plates, not stacks"
    );
}

#[test]
fn a_half_eaten_plate_is_carried_with_its_food_until_scraped_into_the_bin() {
    let kitchen = format!(r#"{KITCHEN},{{"def":"trashCan","x":7,"z":1,"rot":0}}"#);
    let mut w = world_with(&common::content(), &kitchen);
    let table = table(&w);
    let plate = put_on(&mut w, table, "mealPlate", 1, 0);
    w.apply(Command::UseItem {
        sim: 0,
        item: plate,
        action: ItemAction::Clear,
    })
    .unwrap();
    let id = |w: &World| w.sims[0].held.map(|h| w.content.items[h.def].id.clone());
    let (mut before, mut after) = (Vec::new(), Vec::new());
    for _ in 0..common::mins(60.0) {
        w.tick_once();
        let Some(a) = w.sims[0].current() else { break };
        if let TaskKind::Dishes { stage, .. } = a.task.kind
            && let Some(held) = id(&w)
        {
            if stage <= 1 {
                before.push(held);
            } else {
                after.push(held);
            }
        }
    }
    assert!(
        !before.is_empty() && !after.is_empty(),
        "carried to the bin, then on"
    );
    assert!(
        before.iter().all(|h| h == "mealPlate"),
        "with its food on the way to the bin: {before:?}"
    );
    assert!(
        after.iter().all(|h| h == "plateDirty"),
        "scraped, a dirty plate: {after:?}"
    );
}

#[test]
fn a_half_eaten_plate_put_down_part_way_is_a_dirty_plate() {
    let kitchen = format!(r#"{KITCHEN},{{"def":"trashCan","x":7,"z":1,"rot":0}}"#);
    let mut w = world_with(&common::content(), &kitchen);
    let table = table(&w);
    let plate = put_on(&mut w, table, "mealPlate", 1, 0);
    w.apply(Command::UseItem {
        sim: 0,
        item: plate,
        action: ItemAction::Clear,
    })
    .unwrap();
    for _ in 0..common::mins(30.0) {
        w.tick_once();
        if w.sims[0].held.is_some() {
            break;
        }
    }
    assert!(w.sims[0].held.is_some(), "picked up");
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    for _ in 0..common::TICKS_PER_MINUTE {
        w.tick_once();
    }
    let dirty = w.content.item_index("plateDirty").unwrap();
    assert_eq!(w.sims[0].held, None);
    assert!(
        w.items.list.len() == 1 && w.items.list[0].def == dirty,
        "put down as a dirty plate, not lost: {:?}",
        w.items.list
    );
}
