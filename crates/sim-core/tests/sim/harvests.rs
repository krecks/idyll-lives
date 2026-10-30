//! Crops and fruit trees in the shipped content (G-68, I-systems-08, I-garden-10): weeding and
//! watering bring a crop on (`tends`), a harvest is carried in to the fridge (`yields`), and the
//! fruit trees ripen, are picked bare and fruit again (`grows`); the lemon is for cooking.

use crate::common;

use sim_core::items::ItemAt;
use sim_core::world::{GrowthStage, TaskKind};
use sim_core::{Command, TICKS_PER_MINUTE, World};

/// A fridge by the house wall, a vegetable patch, an apple tree and a lemon tree.
const TOWN: &str = r#"{"width":20,"depth":16,
    "plots":[{"name":"Home","x":0,"z":0,"w":20,"d":16,"entry":[9.5,15.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[{"def":"fridge","x":2,"z":1},{"def":"garden.vegPatch","x":10,"z":8},
               {"def":"garden.apple","x":15,"z":3},{"def":"garden.lemonTree","x":6,"z":10}],
    "sims":[{"name":"Ada","traits":["bookworm"],"x":8.5,"z":12.5}]}"#;

const FRIDGE: u32 = 0;
const PATCH: u32 = 1;
const APPLE: u32 = 2;
const LEMON: u32 = 3;

fn world(town: &str) -> World {
    let mut w = World::from_json(&common::content(), town, 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    let n = w.content.needs.len();
    w.sims[0].needs[..n].fill(1.0);
    w
}

fn interaction(w: &World, object: u32, id: &str) -> usize {
    let def = w.objects[object as usize].def;
    w.content.objects[def]
        .interactions
        .iter()
        .position(|i| i.id == id)
        .unwrap_or_else(|| panic!("no {id}"))
}

/// Ticks `w` for up to `minutes`, until `done`; true if it got there.
fn until(w: &mut World, minutes: u64, mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..minutes * TICKS_PER_MINUTE {
        if done(w) {
            return true;
        }
        w.tick_once();
    }
    done(w)
}

fn produce(w: &World) -> usize {
    w.content
        .items
        .iter()
        .position(|i| i.id == "gardenProduce")
        .unwrap()
}

#[test]
fn weeding_and_watering_bring_the_crop_on() {
    let mut w = world(TOWN);
    w.objects[PATCH as usize].growth = 0.4;
    let tend = interaction(&w, PATCH, "tend");
    w.apply(Command::Use {
        sim: 0,
        object: PATCH,
        interaction: tend,
    })
    .unwrap();
    assert!(
        until(&mut w, 5, |w| w.sims[0].current().is_some_and(
            |a| matches!(a.phase, sim_core::world::Phase::Using { .. })
        )),
        "at it"
    );
    assert!(until(&mut w, 60, |w| w.sims[0].current().is_none()), "done");
    let g = w.objects[PATCH as usize].growth;
    // A whole tend brings it on 0.15 of its way, besides the half hour it grew by itself (of two days).
    assert!((0.55..0.57).contains(&g), "growth {g}");
    // Untended, the same crop only grows with time.
    let mut idle = world(TOWN);
    idle.objects[PATCH as usize].growth = 0.4;
    until(&mut idle, 60, |_| false);
    assert!(idle.objects[PATCH as usize].growth < 0.43);
    // Ripe or picked, tending does nothing for it.
    let mut ripe = world(TOWN);
    ripe.apply(Command::Use {
        sim: 0,
        object: PATCH,
        interaction: tend,
    })
    .unwrap();
    assert!(until(&mut ripe, 60, |w| w.sims[0].current().is_none()
        && w.sims[0].queue().count() == 0));
    assert_eq!(ripe.objects[PATCH as usize].growth, 1.0);
}

#[test]
fn the_harvest_goes_in_the_fridge() {
    let mut w = world(TOWN);
    let harvest = interaction(&w, PATCH, "harvest");
    let hunger = w
        .content
        .needs
        .iter()
        .position(|n| n.id == "hunger")
        .unwrap();
    w.apply(Command::Use {
        sim: 0,
        object: PATCH,
        interaction: harvest,
    })
    .unwrap();
    let mut carried = false;
    let stored = until(&mut w, 90, |w| {
        let s = &w.sims[0];
        carried |= s.held.is_some_and(|h| h.def == produce(w))
            && s.current().is_some_and(|a| {
                matches!(
                    a.task.kind,
                    TaskKind::Carry {
                        baby: None,
                        to: FRIDGE,
                        ..
                    }
                )
            });
        w.items.list.iter().any(|it| it.def == produce(w))
    });
    assert!(carried, "carried in to the fridge in the hands");
    assert!(stored, "put away");
    let fridge = &w.objects[FRIDGE as usize];
    let it = w
        .items
        .list
        .iter()
        .find(|it| it.def == produce(&w))
        .unwrap();
    assert_eq!(
        it.at,
        ItemAt::Stored {
            x: fridge.x,
            z: fridge.z
        },
        "in the fridge"
    );
    assert_eq!(it.servings, 2);
    assert!(it.edible(), "food for later");
    assert!(w.sims[0].needs[hunger] > 0.95, "not eaten on the spot");
    assert!(w.sims[0].held.is_none());
}

#[test]
fn a_hungry_harvester_has_some_then_and_there() {
    let mut w = world(TOWN);
    let hunger = w
        .content
        .needs
        .iter()
        .position(|n| n.id == "hunger")
        .unwrap();
    w.sims[0].needs[hunger] = 0.3;
    let harvest = interaction(&w, PATCH, "harvest");
    w.apply(Command::Use {
        sim: 0,
        object: PATCH,
        interaction: harvest,
    })
    .unwrap();
    // Off to the fridge with it, and some of it at once, sitting down.
    let mut before = None;
    let eating = |w: &World| {
        w.sims[0]
            .current()
            .is_some_and(|a| matches!(a.task.kind, TaskKind::Eat { stage: 1, .. }))
    };
    let ate = until(&mut w, 120, |w| {
        if eating(w) {
            before.get_or_insert(w.sims[0].needs[hunger]);
        }
        before.is_some() && !eating(w)
    });
    assert!(ate, "ate a serving");
    assert!(
        w.sims[0].needs[hunger] > before.unwrap() + 0.1,
        "and it did her good: {} from {before:?}",
        w.sims[0].needs[hunger]
    );
    let left: u8 = w
        .items
        .list
        .iter()
        .filter(|it| it.def == produce(&w))
        .map(|it| it.servings)
        .sum();
    assert_eq!(left, 1, "the rest in the fridge");
}

#[test]
fn with_no_fridge_they_sit_down_and_eat_it() {
    let town = TOWN.replace(r#"{"def":"fridge","x":2,"z":1},"#, "");
    let mut w = world(&town);
    let patch = PATCH - 1;
    let harvest = interaction(&w, patch, "harvest");
    w.apply(Command::Use {
        sim: 0,
        object: patch,
        interaction: harvest,
    })
    .unwrap();
    let ate = until(&mut w, 60, |w| {
        w.sims[0]
            .current()
            .is_some_and(|a| matches!(a.task.kind, TaskKind::Eat { .. }))
    });
    assert!(ate, "eating it");
    assert!(
        !w.items.list.iter().any(|it| it.def == produce(&w)),
        "nothing put away"
    );
}

#[test]
fn the_apple_tree_is_picked_bare_and_fruits_again() {
    let mut w = world(TOWN);
    assert!(w.objects[APPLE as usize].ripe(), "in fruit with the lot");
    let one = interaction(&w, APPLE, "pick");
    let all = interaction(&w, APPLE, "harvest");
    w.apply(Command::Use {
        sim: 0,
        object: APPLE,
        interaction: one,
    })
    .unwrap();
    assert!(
        until(&mut w, 40, |w| w.sims[0].current().is_none()
            && w.sims[0].queue().count() == 0),
        "ate an apple"
    );
    assert!(
        w.objects[APPLE as usize].ripe(),
        "one apple doesn't strip it"
    );
    w.apply(Command::Use {
        sim: 0,
        object: APPLE,
        interaction: all,
    })
    .unwrap();
    assert!(
        until(&mut w, 30, |w| w.objects[APPLE as usize].growth_stage()
            != GrowthStage::Ripe),
        "picked bare"
    );
    assert!(
        until(&mut w, 120, |w| w
            .items
            .list
            .iter()
            .any(|it| it.def == produce(w) && it.servings == 3)),
        "the apples in the fridge"
    );
    for i in [one, all] {
        let err = w
            .apply(Command::Use {
                sim: 0,
                object: APPLE,
                interaction: i,
            })
            .unwrap_err();
        assert!(err.to_string().contains("isn't ripe yet"), "{err}");
    }
    assert!(
        until(&mut w, 4 * 24 * 60, |w| w.objects[APPLE as usize].ripe()),
        "in fruit again in a few days"
    );
}

#[test]
fn lemons_are_for_cooking() {
    let mut w = world(TOWN);
    let hunger = w
        .content
        .needs
        .iter()
        .position(|n| n.id == "hunger")
        .unwrap();
    w.sims[0].needs[hunger] = 0.5;
    let pick = interaction(&w, LEMON, "pick");
    let inter = &w.content.objects[w.objects[LEMON as usize].def].interactions[pick];
    assert_eq!(inter.total_gain[hunger], 0.0, "not eaten like an apple");
    w.apply(Command::Use {
        sim: 0,
        object: LEMON,
        interaction: pick,
    })
    .unwrap();
    assert!(until(&mut w, 30, |w| w.sims[0].current().is_none()
        && w.sims[0].queue().count() == 0));
    let lemons = w
        .content
        .feelings
        .iter()
        .position(|f| f.id == "garden.freshLemons")
        .unwrap();
    assert!(
        w.sims[0].feelings.iter().any(|f| f.def == lemons),
        "fresh lemons: a better cook for a while"
    );
    assert!(!w.objects[LEMON as usize].ripe(), "picked");
    let water = interaction(&w, LEMON, "water");
    let before = w.objects[LEMON as usize].growth;
    w.apply(Command::Use {
        sim: 0,
        object: LEMON,
        interaction: water,
    })
    .unwrap();
    assert!(until(&mut w, 30, |w| w.sims[0].current().is_none()
        && w.sims[0].queue().count() == 0));
    assert!(
        w.objects[LEMON as usize].growth > before + 0.09,
        "watering brings the lemons on"
    );
}
