//! A footstool in front of an armchair: it can be put there (the armchair is sat in from beside
//! it then), and the armchair is still sat in; a fridge with its front blocked still isn't
//! allowed. (The feet go up on it in the renderer: characters.ts `footstoolFor`.)

use sim_core::content::Pose;
use sim_core::{Command, TICKS_PER_MINUTE, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"comfort","label":"Comfort","decayPerHour":0.01}],
    "objects":[
      {"id":"armchair","name":"Armchair","price":50,"interactions":[
        {"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"comfort":0.5},"tags":["lounge"]}]},
      {"id":"pouf","name":"Pouf","price":50,"interactions":[
        {"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"comfort":0.3},"tags":["lounge"]}]},
      {"id":"fridge","name":"Fridge","price":50,"interactions":[
        {"id":"look","label":"Look in","minutes":5,"effects":{"comfort":0.1}}]}],
    "economy":{"startingFunds":20000}}"#;

const TOWN: &str = r#"{"width":16,"depth":16,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":16,"entry":[8.5,15.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "sims":[{"name":"Ada","x":8.5,"z":12.5}]}"#;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    w
}

fn buy(w: &mut World, def: &str, x: i32, z: i32, rot: i32) -> Result<(), sim_core::Error> {
    w.apply(Command::Buy { household: 0, object: def.into(), at: Some([x, z, rot]), colors: Default::default(), turn: None, offset: None, size: None, tall: None })
}

#[test]
fn an_armchair_with_a_footstool_in_front_is_sat_in_from_beside_it() {
    let mut w = world();
    buy(&mut w, "armchair", 6, 6, 0).unwrap();
    buy(&mut w, "pouf", 6, 7, 0).expect("a footstool in front of the armchair");
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    let mut sat = false;
    for _ in 0..10 * TICKS_PER_MINUTE {
        w.tick_once();
        if w.sims[0].pose == Pose::Sit {
            sat = true;
            break;
        }
    }
    assert!(sat, "sat in the armchair");
    let p = w.sims[0].pos;
    assert!((p[0] - 6.5).abs() < 0.3 && (p[1] - 6.5).abs() < 0.3, "in the armchair, not on the pouf: {p:?}");

    // Bought the other way round too: the footstool first.
    let mut w = world();
    buy(&mut w, "pouf", 6, 7, 0).unwrap();
    buy(&mut w, "armchair", 6, 6, 0).expect("an armchair behind a footstool");
    // Walled in on both sides as well, there's no way in.
    buy(&mut w, "fridge", 5, 6, 0).unwrap();
    assert!(buy(&mut w, "fridge", 7, 6, 0).is_err(), "the armchair's last way in kept free");
}

#[test]
fn other_things_still_need_their_front_free() {
    let mut w = world();
    buy(&mut w, "pouf", 6, 7, 0).unwrap();
    assert!(buy(&mut w, "fridge", 6, 6, 0).is_err(), "a fridge facing a pouf would block the way");
}
