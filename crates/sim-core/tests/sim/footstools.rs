//! A footstool in front of an armchair: it can be put there (the armchair is sat in from beside
//! it then), and the armchair is still sat in; a fridge with its front blocked still isn't
//! allowed. (The feet go up on it in the renderer: characters.ts `footstoolFor`.) A desk, sat
//! at from behind, may stand with its screen to a wall, between other things; with its back to
//! one and its front tile taken, it is sat at from the other half of its front.

use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind};
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
        {"id":"look","label":"Look in","minutes":5,"effects":{"comfort":0.1}}]},
      {"id":"desk","name":"Desk","price":50,"footprint":[2,1],"approach":"behind","interactions":[
        {"id":"work","label":"Work","minutes":30,"pose":"sit","effects":{"comfort":0.1},"tags":["lounge"]}]}],
    "economy":{"startingFunds":20000}}"#;

const TOWN: &str = r#"{"width":16,"depth":16,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":16,"entry":[8.5,15.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "sims":[{"name":"Ada","x":8.5,"z":12.5}]}"#;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    w
}

fn buy(w: &mut World, def: &str, x: i32, z: i32, rot: i32) -> Result<(), sim_core::Error> {
    w.apply(Command::Buy {
        household: 0,
        object: def.into(),
        at: Some([x, z, rot]),
        colors: Default::default(),
        turn: None,
        offset: None,
        size: None,
        tall: None,
    })
}

#[test]
fn an_armchair_with_a_footstool_in_front_is_sat_in_from_beside_it() {
    let mut w = world();
    buy(&mut w, "armchair", 6, 6, 0).unwrap();
    buy(&mut w, "pouf", 6, 7, 0).expect("a footstool in front of the armchair");
    w.apply(Command::Use {
        sim: 0,
        object: 0,
        interaction: 0,
    })
    .unwrap();
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
    assert!(
        (p[0] - 6.5).abs() < 0.3 && (p[1] - 6.5).abs() < 0.3,
        "in the armchair, not on the pouf: {p:?}"
    );

    // Bought the other way round too: the footstool first.
    let mut w = world();
    buy(&mut w, "pouf", 6, 7, 0).unwrap();
    buy(&mut w, "armchair", 6, 6, 0).expect("an armchair behind a footstool");
    // Walled in on both sides as well, there's no way in.
    buy(&mut w, "fridge", 5, 6, 0).unwrap();
    assert!(
        buy(&mut w, "fridge", 7, 6, 0).is_err(),
        "the armchair's last way in kept free"
    );
}

#[test]
fn a_desk_stands_with_its_screen_to_the_wall_between_other_things() {
    // Against a wall (along z 2), facing it, a fridge either side: sat at from behind.
    let walled = || {
        let mut w = world();
        let wall: Vec<EdgeEdit> = (3..11)
            .map(|x| EdgeEdit::new(EdgeAxis::H, x, 2, EdgeKind::Wall))
            .collect();
        w.build(0, &wall).unwrap();
        buy(&mut w, "fridge", 5, 2, 0).unwrap();
        buy(&mut w, "fridge", 8, 2, 0).unwrap();
        w
    };
    let mut w = walled();
    buy(&mut w, "desk", 6, 2, 2).expect("a desk with its screen to the wall");
    let desk = w
        .objects
        .iter()
        .find(|o| w.content.objects[o.def].id == "desk")
        .unwrap()
        .id;
    w.apply(Command::Use {
        sim: 0,
        object: desk,
        interaction: 0,
    })
    .unwrap();
    let mut from = None;
    for _ in 0..10 * TICKS_PER_MINUTE {
        if w.sims[0].pose == Pose::Sit {
            break;
        }
        from = Some(w.sims[0].tile());
        w.tick_once();
    }
    assert_eq!(w.sims[0].pose, Pose::Sit, "sat at the desk");
    assert_eq!(from.map(|t| t.1), Some(3), "from behind it: {from:?}");
    // With the tiles behind it taken too, nowhere to sit down from.
    let mut w = walled();
    buy(&mut w, "pouf", 6, 3, 0).unwrap();
    buy(&mut w, "pouf", 7, 3, 0).unwrap();
    assert!(buy(&mut w, "desk", 6, 2, 2).is_err(), "boxed in");
}

#[test]
fn a_desk_with_its_back_to_the_wall_is_sat_at_from_the_other_half_of_its_front() {
    // Back to a wall (along z 6), facing into the room, a fridge either side and a chair on its
    // front tile: the other middle tile in front of it is as good (a generated house turned round
    // swaps which is the front; "dense" 314192 had a writing desk with only that one free).
    let mut w = world();
    let wall: Vec<EdgeEdit> = (3..11)
        .map(|x| EdgeEdit::new(EdgeAxis::H, x, 6, EdgeKind::Wall))
        .collect();
    w.build(0, &wall).unwrap();
    buy(&mut w, "fridge", 5, 6, 0).unwrap();
    buy(&mut w, "fridge", 8, 6, 0).unwrap();
    buy(&mut w, "desk", 6, 6, 0).unwrap();
    buy(&mut w, "pouf", 6, 7, 0)
        .expect("a chair on the desk's front tile, the other half still free");
    let desk = w
        .objects
        .iter()
        .find(|o| w.content.objects[o.def].id == "desk")
        .unwrap()
        .id;
    w.apply(Command::Use {
        sim: 0,
        object: desk,
        interaction: 0,
    })
    .unwrap();
    let mut from = None;
    for _ in 0..10 * TICKS_PER_MINUTE {
        if w.sims[0].pose == Pose::Sit {
            break;
        }
        from = Some(w.sims[0].tile());
        w.tick_once();
    }
    assert_eq!(w.sims[0].pose, Pose::Sit, "sat at the desk");
    assert_eq!(from, Some((7, 7)), "from the other half of its front");
    // With that taken too, nowhere to sit down from.
    assert!(
        buy(&mut w, "pouf", 7, 7, 0).is_err(),
        "the desk's last way in kept free"
    );
}

#[test]
fn other_things_still_need_their_front_free() {
    let mut w = world();
    buy(&mut w, "pouf", 6, 7, 0).unwrap();
    assert!(
        buy(&mut w, "fridge", 6, 6, 0).is_err(),
        "a fridge facing a pouf would block the way"
    );
}
