//! Crops that grow (content `grows`): a vegetable patch comes ripe, is picked by its harvest
//! (`"when": "ripe"`, `"leaves": "picked"`), lies fallow, is planted again by itself and ripens
//! over its `days`. Its stage is saved, and is what the renderer shows (`GrowthStage`).

mod common;

use sim_core::world::GrowthStage;
use sim_core::{Command, TICKS_PER_MINUTE, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01},{"id":"fun","label":"Fun","decayPerHour":0.01}],
    "objects":[{"id":"vegPatch","name":"Vegetable patch","price":90,"grows":{"days":1,"fallow":0.5},"interactions":[
        {"id":"tend","label":"Weed and water","minutes":30,"effects":{"fun":0.1},"tags":["garden"]},
        {"id":"harvest","label":"Harvest vegetables","minutes":20,"effects":{"hunger":0.45},"tags":["food"],"when":"ripe","leaves":"picked"}]}],
    "economy":{"startingFunds":1000}}"#;

const TOWN: &str = r#"{"width":12,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[6.5,11.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[{"def":"vegPatch","x":4,"z":2}],
    "sims":[{"name":"Ada","x":3.5,"z":7.5}]}"#;

const TEND: usize = 0;
const HARVEST: usize = 1;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    w
}

/// Ticks `w` for up to `minutes`, until `done`; the minutes it took, if it got there.
fn until(w: &mut World, minutes: u64, mut done: impl FnMut(&World) -> bool) -> Option<u64> {
    for t in 0..minutes * TICKS_PER_MINUTE {
        if done(w) {
            return Some(t / TICKS_PER_MINUTE);
        }
        w.tick_once();
    }
    done(w).then_some(minutes)
}

#[test]
fn picked_it_lies_fallow_then_grows_back_ripe() {
    let mut w = world();
    assert_eq!(w.objects[0].growth_stage(), GrowthStage::Ripe, "it comes ripe");

    let version = w.structure_version();
    w.apply(Command::Use { sim: 0, object: 0, interaction: HARVEST }).unwrap();
    assert!(until(&mut w, 30, |w| w.objects[0].growth_stage() == GrowthStage::Picked).is_some(), "picked as the harvest starts");
    assert_ne!(w.structure_version(), version, "the patch looks different");
    assert!(until(&mut w, 30, |w| w.sims[0].current().is_none()).is_some(), "the harvest goes on to its end");
    let err = w.apply(Command::Use { sim: 0, object: 0, interaction: HARVEST }).unwrap_err();
    assert!(err.to_string().contains("isn't ripe yet"), "{err}");
    w.apply(Command::Use { sim: 0, object: 0, interaction: TEND }).expect("tending it still works");

    // Fallow for half a day, then planted, growing from a third of the day, and ripe a day later.
    let fallow = until(&mut w, 24 * 60, |w| w.objects[0].growth_stage() == GrowthStage::Planted).expect("planted again");
    assert!((690..=750).contains(&fallow), "fallow {fallow} minutes");
    let planted = until(&mut w, 24 * 60, |w| w.objects[0].growth_stage() == GrowthStage::Growing).expect("growing");
    assert!((460..=500).contains(&planted), "just planted for {planted} minutes");
    let version = w.structure_version();
    let growing = until(&mut w, 24 * 60, |w| w.objects[0].ripe()).expect("ripe");
    assert!((940..=990).contains(&growing), "growing for {growing} minutes");
    assert_ne!(w.structure_version(), version, "ripe looks different");
    w.apply(Command::Use { sim: 0, object: 0, interaction: HARVEST }).expect("ripe again: picked again");
}

#[test]
fn nobody_picks_it_unripe_of_their_own_accord() {
    let mut w = world();
    w.objects[0].growth = 0.5;
    w.apply(Command::SetAutonomy { enabled: true, household: None }).unwrap();
    w.sims[0].needs[0] = 0.05;
    for _ in 0..3 * 60 * TICKS_PER_MINUTE {
        w.tick_once();
        if let Some(a) = w.sims[0].current() {
            assert!(!matches!(a.task.kind, sim_core::world::TaskKind::Use { interaction: HARVEST, .. }), "picking it unripe");
        }
    }
    // Ripe, the hungry one goes for it.
    w.objects[0].growth = 1.0;
    let picked = until(&mut w, 6 * 60, |w| w.objects[0].growth_stage() == GrowthStage::Picked);
    assert!(picked.is_some(), "a hungry resident harvests the ripe patch");
}

#[test]
fn its_growth_is_saved() {
    let mut w = world();
    w.objects[0].growth = 0.42;
    let loaded = common::reload(&w, CONTENT);
    assert_eq!(loaded.objects[0].growth, 0.42);
    assert_eq!(loaded.objects[0].growth_stage(), GrowthStage::Growing);

    // A save from before crops grew: they're ripe.
    let mut save: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
    for o in save["objects"].as_array_mut().unwrap() {
        o.as_object_mut().unwrap().remove("growth");
    }
    let old = World::from_save_json(CONTENT, &save.to_string()).unwrap();
    assert!(old.objects[0].ripe());
}

#[test]
fn the_shipped_crops_grow() {
    let content = sim_core::content::Content::from_json(&common::content()).unwrap();
    for id in ["garden.vegPatch", "garden.tomatoes", "garden.strawberries", "garden.pumpkins", "raisedBed"] {
        let def = content.objects.iter().find(|o| o.id == id).unwrap_or_else(|| panic!("{id}"));
        assert!(def.grows.is_some(), "{id} grows");
        assert!(def.interactions.iter().any(|i| i.ripe && i.picks), "{id}: its harvest wants it ripe and picks it");
        assert!(def.interactions.iter().any(|i| !i.ripe), "{id}: tending it any time");
    }
}
