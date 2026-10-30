//! The dishwasher as it is used (content `rack`, `machines::rack_in`): what's washed up at it goes
//! in its rack and waits, instead of being washed there and then and put away. It runs on its own
//! once the rack is full, or once the washing up is done with it a third full; while it runs, or
//! waits done, the washing up goes to the sink. Someone empties it, and the clean things are away.
//! A dishwasher with things in its rack, or running, is saved and loaded as it was.

use crate::common;

use sim_core::command::ItemAction;
use sim_core::items::{ItemAt, ItemInstance};
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World};

/// A house (walls round 0..10 × 0..8, a door at the bottom) with a kitchen along the back wall
/// and a table for four, on a 16 × 14 plot (as `clear_away.rs`).
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

/// The kitchen: a counter, a dishwasher, a counter (no sink).
const KITCHEN: &str = r#"{"def":"fridge","x":1,"z":1,"rot":0},{"def":"counter","x":2,"z":1,"rot":0},
    {"def":"dishwasher","x":3,"z":1,"rot":0},{"def":"counter","x":4,"z":1,"rot":0}"#;
const DISHWASHER: u32 = 2;
/// With a sink after the counter.
const SINK: u32 = 4;

fn world_with(kitchen: &str) -> World {
    let mut w = World::from_json(&common::content(), &town(kitchen), 7).expect("town loads");
    w.autonomy = false;
    fill(&mut w);
    w
}

fn world() -> World {
    world_with(KITCHEN)
}

fn fill(w: &mut World) {
    for s in &mut w.sims {
        s.needs = [1.0; 8];
    }
}

fn table(w: &World) -> u32 {
    w.objects
        .iter()
        .find(|o| w.content.objects[o.def].id == "tableDining4")
        .unwrap()
        .id
}

/// `n` dirty plates on the table.
fn plates(w: &mut World, n: usize) -> Vec<u32> {
    let table = table(w);
    let def = w.content.item_index("plateDirty").unwrap();
    let radius = w.content.items[def].radius;
    (0..n)
        .map(|_| {
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
            w.items.add(ItemInstance {
                id: 0,
                def,
                at: ItemAt::on(&spot),
                servings: 0,
                quality: 1.0,
                made_at: 0,
                household: 0,
                spoiled: false,
                recipe: None,
            })
        })
        .collect()
}

/// Ticks up to `minutes` until `f` holds (checking the world each tick): whether it did.
fn until(w: &mut World, minutes: f32, mut f: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..(minutes / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        common::check(w);
        if f(w) {
            return true;
        }
    }
    false
}

/// Amara clears away `item` and is done (her queue empty): the stations she washed up or put
/// away at (stage, object), in order.
fn clear(w: &mut World, item: u32) -> Vec<(u8, u32)> {
    fill(w);
    w.apply(Command::UseItem {
        sim: 0,
        item,
        action: ItemAction::Clear,
    })
    .unwrap();
    let mut steps: Vec<(u8, u32)> = Vec::new();
    let done = until(w, 120.0, |w| match w.sims[0].current() {
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
            false
        }
        None => w.sims[0].queue().next().is_none(),
    });
    assert!(
        done,
        "still clearing away: {:?}",
        w.sims[0].current().map(|a| a.task.kind)
    );
    steps
}

fn empty_it(w: &mut World) -> Result<(), String> {
    let def = w.objects[DISHWASHER as usize].def;
    let interaction = w.content.objects[def]
        .interactions
        .iter()
        .position(|i| i.id == "unload")
        .unwrap();
    let object = w.objects[DISHWASHER as usize].stable;
    w.apply(Command::Use {
        sim: 0,
        object,
        interaction,
    })
    .map_err(|e| e.to_string())
}

#[test]
fn the_washing_up_goes_in_the_rack_and_it_runs_once_its_done_then_someone_empties_it() {
    let mut w = world();
    let rack = w.content.objects[w.objects[DISHWASHER as usize].def].rack;
    assert_eq!(rack, 12, "the shipped dishwasher's rack");
    // Four plates (a third of the rack once the last is in), cleared a trip's worth at a time.
    let plates = plates(&mut w, 4);
    let mut trips = 0;
    while let Some(&p) = plates.iter().find(|&&p| w.items.get(p).is_some()) {
        trips += 1;
        // Washed up at the dishwasher, and not put away: it's in the rack.
        assert_eq!(clear(&mut w, p), [(2, DISHWASHER)], "trip {trips}");
        assert_eq!(w.sims[0].held, None, "into the rack");
        let o = &w.objects[DISHWASHER as usize];
        let left = plates.iter().filter(|&&p| w.items.get(p).is_some()).count();
        assert_eq!(o.rack as usize, plates.len() - left, "in the rack");
        assert_eq!(
            o.running(),
            left == 0,
            "runs once the washing up is done ({left} left)"
        );
    }
    assert!(trips > 1, "more than one trip, so it waited partly full");
    // Running, nothing more goes in, and nobody is asked to load it.
    let more = self::plates(&mut w, 1)[0];
    fill(&mut w);
    w.apply(Command::UseItem {
        sim: 0,
        item: more,
        action: ItemAction::Clear,
    })
    .unwrap();
    assert!(
        !until(&mut w, 60.0, |w| w.sims[0].current().is_some_and(
            |a| matches!(
                a.task.kind,
                TaskKind::Dishes {
                    station: Some(DISHWASHER),
                    ..
                }
            )
        )),
        "nothing goes in a dishwasher that's running"
    );
    let def = w.objects[DISHWASHER as usize].def;
    let load = w.content.objects[def]
        .interactions
        .iter()
        .position(|i| i.id == "dishes")
        .unwrap();
    let stable = w.objects[DISHWASHER as usize].stable;
    let refused = w
        .apply(Command::Use {
            sim: 0,
            object: stable,
            interaction: load,
        })
        .unwrap_err()
        .to_string();
    assert!(refused.contains("still running"), "{refused}");
    assert!(empty_it(&mut w).unwrap_err().contains("still running"));
    // It washes for its minutes with nobody at it, then waits, done.
    assert!(
        until(&mut w, 100.0, |w| w.objects[DISHWASHER as usize].done()),
        "done after its run"
    );
    assert!(empty_it(&mut w).is_ok());
    assert!(
        until(&mut w, 30.0, |w| w.objects[DISHWASHER as usize]
            .run
            .is_none()),
        "emptied"
    );
    assert_eq!(
        w.objects[DISHWASHER as usize].rack, 0,
        "the clean things put away"
    );
}

#[test]
fn a_full_rack_runs_at_once_and_the_washing_up_goes_on_at_the_sink() {
    let kitchen = format!(r#"{KITCHEN},{{"def":"sink","x":6,"z":1,"rot":0}}"#);
    let mut w = world_with(&kitchen);
    assert_eq!(w.content.objects[w.objects[SINK as usize].def].id, "sink");
    w.objects[DISHWASHER as usize].rack = 11;
    let plates = plates(&mut w, 5);
    // Washed up wherever is nearest: the dishwasher, two tiles nearer the table.
    assert_eq!(clear(&mut w, plates[0]), [(2, DISHWASHER)]);
    assert!(
        w.objects[DISHWASHER as usize].running(),
        "full: it runs with more washing up to come"
    );
    // Running: the rest is washed at the sink and put away there and then.
    let rest = *plates
        .iter()
        .find(|&&p| w.items.get(p).is_some())
        .expect("plates left for the sink");
    let steps = clear(&mut w, rest);
    assert_eq!(steps.first(), Some(&(2, SINK)), "{steps:?}");
    assert!(
        steps.iter().any(|&(stage, _)| stage == 3),
        "put away: {steps:?}"
    );
}

#[test]
fn a_dishwasher_is_saved_and_loaded_as_it_was() {
    let mut w = world();
    w.objects[DISHWASHER as usize].rack = 5;
    let w2 = World::from_save_json(&common::content(), &w.save_json()).unwrap();
    assert_eq!(w2.objects[DISHWASHER as usize].rack, 5, "the rack's things");
    assert_eq!(w2.objects[DISHWASHER as usize].run, None);
    let p = plates(&mut w, 1)[0];
    w.objects[DISHWASHER as usize].rack = 11;
    clear(&mut w, p);
    assert!(w.objects[DISHWASHER as usize].running());
    let left = w.objects[DISHWASHER as usize].run.unwrap();
    let w2 = World::from_save_json(&common::content(), &w.save_json()).unwrap();
    assert_eq!(
        w2.objects[DISHWASHER as usize].run,
        Some(left),
        "still running"
    );
    assert_eq!(w2.objects[DISHWASHER as usize].rack, 12, "full");
}

#[test]
fn free_will_empties_a_dishwasher_once_its_done() {
    let mut w = world();
    w.objects[DISHWASHER as usize].rack = 12;
    w.objects[DISHWASHER as usize].run = Some(0.0);
    w.autonomy = true;
    assert!(
        until(&mut w, 8.0 * 60.0, |w| w.objects[DISHWASHER as usize]
            .run
            .is_none()),
        "nobody emptied it"
    );
    assert_eq!(w.objects[DISHWASHER as usize].rack, 0);
}
