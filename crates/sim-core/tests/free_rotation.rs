//! Free rotation for decor: 1×1 objects of the `objectRules.freeRotation` categories (or with
//! `freeRotation: true`) stand at any angle past their facing, and anywhere on their tile (an
//! offset off its middle); both are only for looks.

use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
    "objectRules":{"freeRotation":["decor"]},
    "objects":[
      {"id":"lamp","name":"Lamp","price":30,"category":"decor"},
      {"id":"rug","name":"Rug","price":30,"category":"decor","footprint":[2,1]},
      {"id":"vase","name":"Vase","price":30,"category":"kitchen","freeRotation":true},
      {"id":"chair","name":"Chair","price":50,"category":"living"}],
    "economy":{"startingFunds":1000}}"#;

const TOWN: &str = r#"{"width":12,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]}],
    "households":[{"name":"Player","plot":0,"player":true}]}"#;

fn buy(w: &mut World, def: &str, x: i32, turn: u8) -> Result<u32, sim_core::Error> {
    w.apply(Command::Buy { household: 0, object: def.into(), at: Some([x, 1, 1]), style: None, turn: Some(turn), offset: None })?;
    Ok(w.objects.len() as u32 - 1)
}

#[test]
fn decor_turns_freely_and_keeps_its_angle() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let turns: Vec<bool> = w.content.objects.iter().map(|o| o.turns).collect();
    assert_eq!(turns, [true, false, true, false], "1×1 decor, and anything that says so");

    let lamp = buy(&mut w, "lamp", 1, 30).unwrap();
    assert_eq!((w.objects[lamp as usize].rot, w.objects[lamp as usize].turn), (1, 30));
    assert!(buy(&mut w, "chair", 3, 30).unwrap_err().to_string().contains("four ways"));
    assert!(buy(&mut w, "rug", 3, 15).is_err(), "bigger things stay on the grid");
    assert!(buy(&mut w, "vase", 5, 90).is_err(), "90° is the next facing");
    assert_eq!(w.objects.len(), 1, "refused purchases buy nothing");

    // Moving keeps the angle unless a new one is given.
    let mv = |turn| Command::MoveObject { household: 0, object: lamp, x: 4, z: 4, rot: 2, turn, offset: None };
    w.apply(mv(None)).unwrap();
    assert_eq!((w.objects[lamp as usize].rot, w.objects[lamp as usize].turn), (2, 30));
    w.apply(mv(Some(75))).unwrap();
    assert_eq!(w.objects[lamp as usize].turn, 75);
    // Undo puts the old angle back.
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!(w.objects[lamp as usize].turn, 30);

    // Saved and loaded.
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(loaded.objects[lamp as usize].turn, 30);
}

fn buy_at(w: &mut World, household: u32, def: &str, x: i32, z: i32, offset: [i8; 2]) -> Result<u32, sim_core::Error> {
    w.apply(Command::Buy { household, object: def.into(), at: Some([x, z, 0]), style: None, turn: None, offset: Some(offset) })?;
    Ok(w.objects.len() as u32 - 1)
}

#[test]
fn decor_stands_anywhere_on_its_tile() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let lamp = buy_at(&mut w, 0, "lamp", 1, 1, [25, -50]).unwrap();
    assert_eq!(w.objects[lamp as usize].offset, [25, -50]);
    assert!(buy_at(&mut w, 0, "chair", 3, 1, [25, 0]).unwrap_err().to_string().contains("square"));
    assert!(buy_at(&mut w, 0, "vase", 5, 1, [51, 0]).unwrap_err().to_string().contains("off its tile"));
    assert_eq!(w.objects.len(), 1, "refused purchases buy nothing");
    // The footprint stays on its tile: the next tile is still free.
    let vase = buy_at(&mut w, 0, "vase", 2, 1, [-50, 0]).unwrap();
    let at = |w: &World, x, z| w.objects.iter().position(|o| o.x == x && o.z == z).unwrap();

    // Moving keeps the offset unless a new one is given; undo puts the old one back. A move
    // re-adds the object at the end of the list: the new offset is its own, not its neighbour's.
    assert!(lamp < vase);
    w.apply(Command::MoveObject { household: 0, object: lamp, x: 4, z: 4, rot: 0, turn: Some(15), offset: Some([-10, 7]) }).unwrap();
    assert_eq!((w.objects[at(&w, 4, 4)].offset, w.objects[at(&w, 4, 4)].turn), ([-10, 7], 15));
    assert_eq!((w.objects[at(&w, 2, 1)].offset, w.objects[at(&w, 2, 1)].turn), ([-50, 0], 0), "the vase is as it was");
    w.apply(Command::Undo { household: 0 }).unwrap();
    let lamp = at(&w, 1, 1) as u32;
    let mv = |offset| Command::MoveObject { household: 0, object: lamp, x: 4, z: 4, rot: 0, turn: None, offset };
    w.apply(mv(None)).unwrap();
    let lamp = at(&w, 4, 4);
    assert_eq!(w.objects[lamp].offset, [25, -50]);
    w.apply(Command::MoveObject { household: 0, object: lamp as u32, x: 4, z: 4, rot: 0, turn: None, offset: Some([-10, 7]) }).unwrap();
    let lamp = at(&w, 4, 4);
    assert_eq!(w.objects[lamp].offset, [-10, 7]);
    w.apply(Command::Undo { household: 0 }).unwrap();
    let lamp = w.objects.iter().position(|o| o.x == 4 && o.z == 4).unwrap();
    assert_eq!(w.objects[lamp].offset, [25, -50]);

    // Saved and loaded.
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(loaded.objects[lamp].offset, [25, -50]);
}

#[test]
fn a_blueprint_turned_round_turns_the_offset_round_too() {
    // The east lot faces the other way: a blueprint from the west lot is built turned round.
    const TWO_LOTS: &str = r#"{"width":24,"depth":12,
        "plots":[{"name":"West","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]},
                 {"name":"East","x":12,"z":0,"w":12,"d":10,"entry":[18.5,0.5]}],
        "households":[{"name":"Player","plot":0,"player":true},{"name":"Other","plot":1}]}"#;
    let mut w = World::from_json(CONTENT, TWO_LOTS, 1).unwrap();
    buy_at(&mut w, 0, "lamp", 2, 2, [25, -10]).unwrap();
    let bp = w.blueprint(0).unwrap();
    w.apply(Command::BuildBlueprint { household: 1, blueprint: bp.clone() }).unwrap();
    let built = w.objects.iter().find(|o| o.x >= 12).unwrap();
    assert_eq!(built.offset, [-25, 10]);
    assert_eq!(w.blueprint(1).unwrap().objects[0].offset, bp.objects[0].offset, "the same, seen from its own street");
}
