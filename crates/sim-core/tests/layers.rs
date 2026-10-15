//! Things on a wall, the ceiling or the floor's surface (content `layer`) take no floor space:
//! a picture over a sofa, a lamp over a table, a rug under a bed. Only things on the same layer
//! can't overlap; a picture needs a wall behind it, a ceiling lamp a room. Bunk beds put their
//! two sleepers in the middle of the bed.

mod common;

use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.02}],
    "objects":[
      {"id":"sofa","name":"Sofa","price":100,"footprint":[2,1],"interactions":[]},
      {"id":"picture","name":"Picture","price":50,"layer":"wall","interactions":[]},
      {"id":"mirror","name":"Mirror","price":50,"layer":"wall","interactions":[
        {"id":"look","label":"Look","minutes":5,"effects":{"energy":0.01}}]},
      {"id":"pendant","name":"Pendant","price":50,"layer":"ceiling","light":{"range":4},"interactions":[]},
      {"id":"rug","name":"Rug","price":50,"layer":"rug","footprint":[2,2],"interactions":[]},
      {"id":"bunk","name":"Bunk","price":50,"footprint":[1,2],"slots":2,"bunk":true,"interactions":[
        {"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1}}]}],
    "economy":{"startingFunds":5000}}"#;

/// One room (x 1..9, z 1..7) with a door at the bottom, on a 12×10 plot.
const TOWN: &str = r#"{"width":12,"depth":12,
    "walls":[[1,1,9,1],[1,7,9,7],[1,1,1,7],[9,1,9,7]],
    "doors":[{"x":4,"z":7,"axis":"x"}],
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]}],
    "households":[{"name":"Player","plot":0,"player":true}]}"#;

fn buy(w: &mut World, def: &str, x: i32, z: i32, rot: i32) -> Result<u32, sim_core::Error> {
    w.apply(Command::Buy { household: 0, object: def.into(), at: Some([x, z, rot]), colors: Default::default(), turn: None, offset: None, size: None, tall: None })?;
    Ok(w.objects.len() as u32 - 1)
}

#[test]
fn things_on_a_wall_or_ceiling_share_tiles_with_furniture() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    // A sofa against the top wall, a picture over it and a rug in front.
    buy(&mut w, "sofa", 2, 1, 0).unwrap();
    buy(&mut w, "picture", 2, 1, 0).expect("a picture hangs over the sofa");
    assert!(buy(&mut w, "picture", 2, 1, 0).is_err(), "but not two pictures in one place");
    buy(&mut w, "picture", 3, 1, 0).expect("the next spot on the wall is free");
    // Away from a wall a picture has nothing to hang on; facing the wrong way neither.
    let err = buy(&mut w, "picture", 5, 4, 0).unwrap_err().to_string();
    assert!(err.contains("wall"), "{err}");
    assert!(buy(&mut w, "picture", 6, 1, 2).is_err(), "its back must be on the wall");
    // A ceiling lamp over the sofa, but not outdoors.
    buy(&mut w, "pendant", 2, 1, 0).expect("a lamp hangs above the sofa");
    let err = buy(&mut w, "pendant", 10, 9, 0).unwrap_err().to_string();
    assert!(err.contains("ceiling"), "{err}");
    // A rug under furniture, and furniture onto a rug.
    buy(&mut w, "rug", 5, 3, 0).unwrap();
    buy(&mut w, "sofa", 5, 3, 0).expect("a sofa stands on the rug");
    assert!(buy(&mut w, "rug", 6, 4, 0).is_err(), "rugs don't overlap");

    // None of it blocks walking: the tiles under the picture, the lamp and the rug are free.
    let free = |w: &World, x: i32, z: i32| !w.blocked()[w.lot.tile_index(x, z)];
    assert!(free(&w, 5, 4) && free(&w, 6, 4), "the rug in front of the sofa is walked on");
    assert!(!free(&w, 2, 1) && !free(&w, 3, 1), "the sofa under the pictures still takes its tiles");

    // A picture in front of which a sofa now stands is fine (nobody needs to reach it), but a
    // mirror someone uses keeps its front free.
    buy(&mut w, "mirror", 7, 1, 0).unwrap();
    assert!(buy(&mut w, "sofa", 7, 2, 0).is_err(), "the mirror's front stays reachable");

    // A wall something hangs on can't be taken down or get a door.
    let edit = |kind| sim_core::command::EdgeEdit {
        axis: sim_core::command::EdgeAxis::H, x: 3, z: 1, kind, cover: None, form: None, style: None, sill: None, head: None,
    };
    let err = w.apply(Command::Build { household: 0, edits: vec![edit(sim_core::command::EdgeKind::Open)] }).unwrap_err().to_string();
    assert!(err.contains("hangs"), "{err}");
    assert!(w.apply(Command::Build { household: 0, edits: vec![edit(sim_core::command::EdgeKind::Door)] }).is_err());

    // Saved and loaded with everything in place.
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(loaded.objects.len(), w.objects.len());
    assert!(!loaded.content.objects[1].turns, "things on a wall don't turn freely");
}

#[test]
fn a_lamp_left_without_a_ceiling_doesnt_spoil_the_save() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    buy(&mut w, "sofa", 2, 1, 0).unwrap();
    buy(&mut w, "pendant", 5, 4, 0).unwrap();
    // Taking down a stretch of the right wall opens the room to the garden: the lamp, which
    // building lets stay, now hangs where there is no ceiling.
    let open = sim_core::command::EdgeEdit {
        axis: sim_core::command::EdgeAxis::V, x: 9, z: 3, kind: sim_core::command::EdgeKind::Open, cover: None, form: None, style: None, sill: None, head: None,
    };
    w.apply(Command::Build { household: 0, edits: vec![open] }).unwrap();
    assert_eq!(w.lot.room_at(5, 4), sim_core::lot::OUTDOORS, "the room is open to the outdoors");
    assert_eq!(w.objects.len(), 2);
    // The save loads, without the lamp that can't hang there; the rest stays.
    let loaded = World::from_save_json(CONTENT, &w.save_json()).expect("the save loads");
    let defs: Vec<&str> = loaded.objects.iter().map(|o| loaded.content.objects[o.def].id.as_str()).collect();
    assert_eq!(defs, ["sofa"]);
}

#[test]
fn bunk_beds_put_both_sleepers_in_the_middle() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let bed = buy(&mut w, "bunk", 6, 2, 0).unwrap() as usize;
    let a = w.objects[bed].slot_position(&w.content, 0);
    let b = w.objects[bed].slot_position(&w.content, 1);
    let c = w.objects[bed].centre(&w.content);
    assert!((a[0] - c[0]).abs() < 0.05 && (b[0] - c[0]).abs() < 0.05, "{a:?} {b:?} {c:?}");
    assert!(b[0] > a[0], "a hair apart: the second sleeper is on the upper bunk");
}

/// Curtains (content `overWindow`), a wardrobe as tall as the wall (`fillsWall`) and a plant
/// hanging low from the ceiling (`hangsLow`), with the things above.
const TALL: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.02}],
    "objects":[
      {"id":"sofa","name":"Sofa","price":100,"footprint":[2,1],"interactions":[]},
      {"id":"picture","name":"Picture","price":50,"layer":"wall","interactions":[]},
      {"id":"curtains","name":"Curtains","price":50,"layer":"wall","overWindow":true,"interactions":[]},
      {"id":"pendant","name":"Pendant","price":50,"layer":"ceiling","interactions":[]},
      {"id":"wardrobe","name":"Wardrobe","price":100,"fillsWall":true,"interactions":[]},
      {"id":"plant","name":"Hanging plant","price":50,"layer":"ceiling","hangsLow":true,"interactions":[]},
      {"id":"rug","name":"Rug","price":50,"layer":"rug","interactions":[]}],
    "economy":{"startingFunds":5000}}"#;

/// The room above, with windows in its top wall at x 3 and 4.
const WINDOWS: &str = r#"{"width":12,"depth":12,
    "walls":[[1,1,9,1],[1,7,9,7],[1,1,1,7],[9,1,9,7]],
    "doors":[{"x":4,"z":7,"axis":"x"}],
    "windows":[{"x":3,"z":1,"axis":"x"},{"x":4,"z":1,"axis":"x"}],
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "sims":[{"name":"Ada","x":7.5,"z":5.5}]}"#;

fn edge(kind: sim_core::command::EdgeKind, x: i32) -> Command {
    let e = sim_core::command::EdgeEdit { axis: sim_core::command::EdgeAxis::H, x, z: 1, kind, cover: None, form: None, style: None, sill: None, head: None };
    Command::Build { household: 0, edits: vec![e] }
}

#[test]
fn only_what_belongs_over_a_window_hangs_on_one() {
    let mut w = World::from_json(TALL, WINDOWS, 1).unwrap();
    let err = buy(&mut w, "picture", 3, 1, 0).unwrap_err().to_string();
    assert_eq!(err, "that can't hang over a window");
    buy(&mut w, "curtains", 3, 1, 0).expect("curtains hang over a window");
    buy(&mut w, "curtains", 6, 1, 0).expect("and on a plain wall");
    buy(&mut w, "picture", 7, 1, 0).expect("a picture on a plain wall");
    // Neither does a window go in behind a picture; behind curtains it does.
    use sim_core::command::EdgeKind;
    let err = w.apply(edge(EdgeKind::Window, 7)).unwrap_err().to_string();
    assert!(err.contains("picture") && err.contains("window"), "{err}");
    w.apply(edge(EdgeKind::Window, 6)).expect("a window behind the curtains");
    // Moved over the window, the picture is refused and stays where it was.
    let picture = w.objects.iter().position(|o| w.content.objects[o.def].id == "picture").unwrap() as u32;
    let mv = Command::MoveObject { household: 0, object: picture, x: 4, z: 1, rot: 0, turn: None, offset: None, size: None, tall: None };
    assert!(w.apply(mv).unwrap_err().to_string().contains("window"));
    assert!(w.objects.iter().any(|o| w.content.objects[o.def].id == "picture" && o.x == 7));
}

#[test]
fn tall_furniture_takes_the_wall_and_the_ceiling_over_it() {
    let mut w = World::from_json(TALL, WINDOWS, 1).unwrap();
    buy(&mut w, "wardrobe", 6, 1, 0).unwrap();
    let err = buy(&mut w, "picture", 6, 1, 0).unwrap_err().to_string();
    assert_eq!(err, "that would hang into the wardrobe");
    // (On any wall of its tile: the side wall of a corner too.)
    buy(&mut w, "wardrobe", 8, 1, 0).unwrap();
    assert!(buy(&mut w, "picture", 8, 1, 3).is_err(), "not on the side wall beside it");
    assert!(buy(&mut w, "pendant", 6, 1, 0).unwrap_err().to_string().contains("hang into the wardrobe"));
    buy(&mut w, "picture", 7, 1, 0).expect("beside it is fine");
    buy(&mut w, "rug", 6, 2, 0).expect("so is a rug in front");
    // Nor does a wardrobe go under a picture or a lamp.
    let err = buy(&mut w, "wardrobe", 7, 1, 0).unwrap_err().to_string();
    assert_eq!(err, "that is too tall for the picture hanging there");
    buy(&mut w, "pendant", 2, 4, 0).unwrap();
    assert!(buy(&mut w, "wardrobe", 2, 4, 0).is_err(), "not under a lamp either");
    buy(&mut w, "sofa", 2, 4, 0).expect("a sofa under the lamp is fine");
}

#[test]
fn a_plant_hanging_low_takes_the_floor_under_it() {
    let mut w = World::from_json(TALL, WINDOWS, 1).unwrap();
    let free = |w: &World, x: i32, z: i32| !w.blocked()[w.lot.tile_index(x, z)];
    let plant = buy(&mut w, "plant", 5, 4, 0).unwrap();
    assert!(!free(&w, 5, 4), "nobody walks into its pot");
    let err = buy(&mut w, "sofa", 4, 4, 0).unwrap_err().to_string();
    assert_eq!(err, "that is under the hanging plant, which hangs too low");
    buy(&mut w, "sofa", 2, 3, 0).unwrap();
    let err = buy(&mut w, "plant", 3, 3, 0).unwrap_err().to_string();
    assert_eq!(err, "that would hang too low over the sofa");
    buy(&mut w, "pendant", 3, 3, 0).expect("a lamp hangs over the sofa");
    buy(&mut w, "rug", 5, 4, 0).expect("a rug lies under the plant");
    assert!(buy(&mut w, "plant", 7, 5, 0).unwrap_err().to_string().contains("standing there"), "not on Ada's head");

    // Saved and loaded, it still takes its tile; sold, it's free again.
    let loaded = World::from_save_json(TALL, &w.save_json()).unwrap();
    assert!(!free(&loaded, 5, 4));
    w.apply(Command::Sell { household: 0, object: plant }).unwrap();
    assert!(free(&w, 5, 4), "the floor is free again");
}

#[test]
fn the_flags_are_for_their_own_layers() {
    for (flag, layer) in [("overWindow", "floor"), ("fillsWall", "wall"), ("hangsLow", "floor")] {
        let content = format!(r#"{{"needs":[{{"id":"energy","label":"Energy"}}],"objects":[{{"id":"x","name":"X","layer":"{layer}","{flag}":true,"interactions":[]}}]}}"#);
        let err = World::from_json(&content, WINDOWS, 1).err().map(|e| e.to_string()).unwrap_or_default();
        assert!(err.contains(flag), "{flag} on the {layer}: {err}");
    }
}

#[test]
fn a_mirror_or_a_mirror_cabinet_hangs_over_every_sink() {
    // The shipped content: every sink against the top wall, a mirror cabinet over it (used to
    // brush teeth, from the tile in front of the sink) and a round mirror over a double's other half.
    let content = common::content();
    for (sink, wide) in [("sink", 1), ("sinkPedestal", 1), ("sinkVessel", 1), ("sinkConsole", 1), ("farmhouseSink", 1), ("vanityDouble", 2), ("counterSinkDouble", 2)] {
        let mut w = World::from_json(&content, TOWN, 1).unwrap();
        w.households[0].funds = 100_000;
        buy(&mut w, sink, 3, 1, 0).unwrap_or_else(|e| panic!("{sink}: {e}"));
        buy(&mut w, "mirrorCabinet", 3, 1, 0).unwrap_or_else(|e| panic!("a mirror cabinet over the {sink}: {e}"));
        if wide == 2 {
            buy(&mut w, "mirrorRound", 4, 1, 0).unwrap_or_else(|e| panic!("a mirror over the {sink}'s other half: {e}"));
        }
        // But not two things on the wall in one place.
        assert!(buy(&mut w, "mirrorRound", 3, 1, 0).is_err(), "a mirror over the cabinet over the {sink}");
    }
    // Over furniture as tall as the wall, neither hangs.
    let mut w = World::from_json(&content, TOWN, 1).unwrap();
    w.households[0].funds = 100_000;
    for (tall, x) in [("fridgeBuiltIn", 3), ("wardrobe", 6)] {
        buy(&mut w, tall, x, 1, 0).unwrap_or_else(|e| panic!("{tall}: {e}"));
        for mirror in ["mirrorCabinet", "mirrorRound"] {
            let err = buy(&mut w, mirror, x, 1, 0).unwrap_err().to_string();
            assert!(err.contains("hang into"), "a {mirror} over the {tall}: {err}");
        }
    }
}
