//! Things on a wall, the ceiling or the floor's surface (content `layer`) take no floor space:
//! a picture over a sofa, a lamp over a table, a rug under a bed. Only things on the same layer
//! can't overlap; a picture needs a wall behind it, a ceiling lamp a room (and walls can't come
//! down from round it). A lamp at head height takes the floor from walking, not from furniture;
//! what stands against a wall needs one. Bunk beds put their two sleepers in the middle of the bed.

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
    Ok(w.objects.last().unwrap().stable)
}

#[test]
fn things_on_a_wall_or_ceiling_share_tiles_with_furniture() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    // A sofa against the top wall, a picture over it and a rug in front.
    buy(&mut w, "sofa", 2, 1, 0).unwrap();
    buy(&mut w, "picture", 2, 1, 0).expect("a picture hangs over the sofa");
    assert!(
        buy(&mut w, "picture", 2, 1, 0).is_err(),
        "but not two pictures in one place"
    );
    buy(&mut w, "picture", 3, 1, 0).expect("the next spot on the wall is free");
    // Away from a wall a picture has nothing to hang on; facing the wrong way neither.
    let err = buy(&mut w, "picture", 5, 4, 0).unwrap_err().to_string();
    assert!(err.contains("wall"), "{err}");
    assert!(
        buy(&mut w, "picture", 6, 1, 2).is_err(),
        "its back must be on the wall"
    );
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
    assert!(
        free(&w, 5, 4) && free(&w, 6, 4),
        "the rug in front of the sofa is walked on"
    );
    assert!(
        !free(&w, 2, 1) && !free(&w, 3, 1),
        "the sofa under the pictures still takes its tiles"
    );

    // A picture in front of which a sofa now stands is fine (nobody needs to reach it), but a
    // mirror someone uses keeps its front free.
    buy(&mut w, "mirror", 7, 1, 0).unwrap();
    assert!(
        buy(&mut w, "sofa", 7, 2, 0).is_err(),
        "the mirror's front stays reachable"
    );

    // A wall something hangs on can't be taken down or get a door.
    let edit = |kind| sim_core::command::EdgeEdit {
        axis: sim_core::command::EdgeAxis::H,
        x: 3,
        z: 1,
        kind,
        cover: None,
        form: None,
        style: None,
        sill: None,
        head: None,
    };
    let err = w
        .apply(Command::Build {
            household: 0,
            edits: vec![edit(sim_core::command::EdgeKind::Open)],
        })
        .unwrap_err()
        .to_string();
    assert!(err.contains("hangs"), "{err}");
    assert!(
        w.apply(Command::Build {
            household: 0,
            edits: vec![edit(sim_core::command::EdgeKind::Door)]
        })
        .is_err()
    );

    // Saved and loaded with everything in place.
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(loaded.objects.len(), w.objects.len());
    assert!(
        !loaded.content.objects[1].turns,
        "things on a wall don't turn freely"
    );
}

#[test]
fn a_lamp_left_without_a_ceiling_doesnt_spoil_the_save() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    buy(&mut w, "sofa", 2, 1, 0).unwrap();
    buy(&mut w, "pendant", 5, 4, 0).unwrap();
    // Taking down a stretch of the right wall would open the room to the garden, leaving the
    // lamp where there is no ceiling: building refuses (`a_wall_taken_down_leaves_…`).
    let open = sim_core::command::EdgeEdit {
        axis: sim_core::command::EdgeAxis::V,
        x: 9,
        z: 3,
        kind: sim_core::command::EdgeKind::Open,
        cover: None,
        form: None,
        style: None,
        sill: None,
        head: None,
    };
    assert!(
        w.apply(Command::Build {
            household: 0,
            edits: vec![open]
        })
        .is_err()
    );
    assert_ne!(
        w.lot.room_at(5, 4),
        sim_core::lot::OUTDOORS,
        "the room still stands"
    );
    // A save with a lamp outdoors all the same (from before that rule) loads without it; the rest stays.
    let mut save: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
    let lamp = save["objects"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|o| o["def"] == "pendant")
        .unwrap();
    lamp["x"] = 10.into();
    let loaded = World::from_save_json(CONTENT, &save.to_string()).expect("the save loads");
    let defs: Vec<&str> = loaded
        .objects
        .iter()
        .map(|o| loaded.content.objects[o.def].id.as_str())
        .collect();
    assert_eq!(defs, ["sofa"]);
}

#[test]
fn bunk_beds_put_both_sleepers_in_the_middle() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let bed = buy(&mut w, "bunk", 6, 2, 0).unwrap() as usize;
    let a = w.objects[bed].slot_position(&w.content, 0);
    let b = w.objects[bed].slot_position(&w.content, 1);
    let c = w.objects[bed].centre(&w.content);
    assert!(
        (a[0] - c[0]).abs() < 0.05 && (b[0] - c[0]).abs() < 0.05,
        "{a:?} {b:?} {c:?}"
    );
    assert!(
        b[0] > a[0],
        "a hair apart: the second sleeper is on the upper bunk"
    );
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
      {"id":"chandelier","name":"Chandelier","price":50,"layer":"ceiling","headHeight":true,"interactions":[]},
      {"id":"ladder","name":"Towel ladder","price":40,"againstWall":true,"interactions":[]},
      {"id":"arc","name":"Arc lamp","price":50,"overhang":[[0,1]],"interactions":[]},
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
    let e = sim_core::command::EdgeEdit {
        axis: sim_core::command::EdgeAxis::H,
        x,
        z: 1,
        kind,
        cover: None,
        form: None,
        style: None,
        sill: None,
        head: None,
    };
    Command::Build {
        household: 0,
        edits: vec![e],
    }
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
    w.apply(edge(EdgeKind::Window, 6))
        .expect("a window behind the curtains");
    // Moved over the window, the picture is refused and stays where it was.
    let picture = w
        .objects
        .iter()
        .position(|o| w.content.objects[o.def].id == "picture")
        .unwrap() as u32;
    let mv = Command::MoveObject {
        household: 0,
        object: picture,
        x: 4,
        z: 1,
        rot: 0,
        turn: None,
        offset: None,
        size: None,
        tall: None,
    };
    assert!(w.apply(mv).unwrap_err().to_string().contains("window"));
    assert!(
        w.objects
            .iter()
            .any(|o| w.content.objects[o.def].id == "picture" && o.x == 7)
    );
}

#[test]
fn tall_furniture_takes_the_wall_and_the_ceiling_over_it() {
    let mut w = World::from_json(TALL, WINDOWS, 1).unwrap();
    buy(&mut w, "wardrobe", 6, 1, 0).unwrap();
    let err = buy(&mut w, "picture", 6, 1, 0).unwrap_err().to_string();
    assert_eq!(err, "that would hang into the wardrobe");
    // (On any wall of its tile: the side wall of a corner too.)
    buy(&mut w, "wardrobe", 8, 1, 0).unwrap();
    assert!(
        buy(&mut w, "picture", 8, 1, 3).is_err(),
        "not on the side wall beside it"
    );
    assert!(
        buy(&mut w, "pendant", 6, 1, 0)
            .unwrap_err()
            .to_string()
            .contains("hang into the wardrobe")
    );
    buy(&mut w, "picture", 7, 1, 0).expect("beside it is fine");
    buy(&mut w, "rug", 6, 2, 0).expect("so is a rug in front");
    // Nor does a wardrobe go under a picture or a lamp.
    let err = buy(&mut w, "wardrobe", 7, 1, 0).unwrap_err().to_string();
    assert_eq!(err, "that is too tall for the picture hanging there");
    buy(&mut w, "pendant", 2, 4, 0).unwrap();
    assert!(
        buy(&mut w, "wardrobe", 2, 4, 0).is_err(),
        "not under a lamp either"
    );
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
    assert!(
        buy(&mut w, "plant", 7, 5, 0)
            .unwrap_err()
            .to_string()
            .contains("standing there"),
        "not on Ada's head"
    );

    // Saved and loaded, it still takes its tile; sold, it's free again.
    let loaded = World::from_save_json(TALL, &w.save_json()).unwrap();
    assert!(!free(&loaded, 5, 4));
    w.apply(Command::Sell {
        household: 0,
        object: plant,
    })
    .unwrap();
    assert!(free(&w, 5, 4), "the floor is free again");
}

#[test]
fn a_lamp_at_head_height_hangs_over_a_table_not_over_heads() {
    let mut w = World::from_json(TALL, WINDOWS, 1).unwrap();
    let free = |w: &World, x: i32, z: i32| !w.blocked()[w.lot.tile_index(x, z)];
    buy(&mut w, "chandelier", 5, 4, 0).unwrap();
    assert!(!free(&w, 5, 4), "nobody walks under it");
    buy(&mut w, "sofa", 4, 4, 0).expect("furniture goes under it");
    assert!(
        buy(&mut w, "wardrobe", 5, 3, 0).is_ok(),
        "beside it, a wardrobe is fine"
    );
    assert!(
        buy(&mut w, "chandelier", 5, 3, 0)
            .unwrap_err()
            .to_string()
            .contains("hang into the wardrobe"),
        "not over one"
    );
    buy(&mut w, "sofa", 2, 3, 0).unwrap();
    buy(&mut w, "chandelier", 3, 3, 0).expect("it hangs over the sofa too");
    assert!(
        buy(&mut w, "chandelier", 7, 5, 0)
            .unwrap_err()
            .to_string()
            .contains("standing there"),
        "not on Ada's head"
    );
    let loaded = World::from_save_json(TALL, &w.save_json()).unwrap();
    assert!(
        !free(&loaded, 3, 3) && !free(&loaded, 5, 4),
        "loaded, it still takes its tiles"
    );
    // Sold, it leaves the sofa under it in the way still (and the tile beside, now free).
    let lamp = w
        .objects
        .iter()
        .find(|o| w.content.objects[o.def].id == "chandelier" && o.x == 3)
        .unwrap()
        .stable;
    w.apply(Command::Sell {
        household: 0,
        object: lamp,
    })
    .unwrap();
    assert!(
        !free(&w, 3, 3),
        "the sofa under the sold lamp is still in the way"
    );
    let lamp = w
        .objects
        .iter()
        .find(|o| w.content.objects[o.def].id == "chandelier" && o.x == 5)
        .unwrap()
        .stable;
    w.apply(Command::Sell {
        household: 0,
        object: lamp,
    })
    .unwrap();
    assert!(
        !free(&w, 5, 4) && free(&w, 6, 4),
        "the sofa's other tile too"
    );
}

#[test]
fn an_arc_lamp_reaches_over_the_sofa_in_front_of_it() {
    // Its shade over the tile in front (content `overhang`), at head height: furniture under it,
    // nobody walking there, nothing tall or hanging low there, no wall in the way.
    let mut w = World::from_json(TALL, WINDOWS, 1).unwrap();
    let free = |w: &World, x: i32, z: i32| !w.blocked()[w.lot.tile_index(x, z)];
    buy(&mut w, "arc", 2, 2, 0).unwrap();
    assert!(!free(&w, 2, 3), "nobody walks under the shade");
    buy(&mut w, "sofa", 2, 3, 0).expect("a sofa under it");
    buy(&mut w, "sofa", 4, 4, 0).unwrap();
    buy(&mut w, "arc", 5, 3, 0).expect("bought behind a sofa, over it");
    buy(&mut w, "arc", 7, 2, 3).unwrap();
    assert!(
        !free(&w, 6, 2) && free(&w, 8, 2),
        "turned, over the tile in front of it"
    );
    buy(&mut w, "arc", 3, 5, 0).unwrap();
    let refused = |w: &mut World, def: &str, x: i32, z: i32, rot: i32| {
        buy(w, def, x, z, rot).unwrap_err().to_string()
    };
    assert!(
        refused(&mut w, "wardrobe", 3, 6, 0).contains("too tall"),
        "nothing tall under it"
    );
    assert!(
        refused(&mut w, "chandelier", 3, 6, 0).contains("hang into"),
        "nor a lamp at head height"
    );
    assert!(
        refused(&mut w, "arc", 4, 6, 3).contains("hang into"),
        "nor another shade"
    );
    assert!(buy(&mut w, "arc", 8, 6, 0).is_err(), "not over the wall");
    buy(&mut w, "arc", 8, 6, 2).expect("turned the other way, over the room");
    let loaded = World::from_save_json(TALL, &w.save_json()).unwrap();
    assert!(
        !free(&loaded, 2, 3) && !free(&loaded, 6, 2),
        "loaded, it still takes its tiles"
    );
    // Sold, the sofa under it is still in the way, and what was only under the shade free.
    let lamp = |w: &World, x: i32| {
        w.objects
            .iter()
            .find(|o| w.content.objects[o.def].id == "arc" && o.x == x)
            .unwrap()
            .stable
    };
    let arc = lamp(&w, 2);
    w.apply(Command::Sell {
        household: 0,
        object: arc,
    })
    .unwrap();
    assert!(!free(&w, 2, 3), "the sofa is still there");
    let arc = lamp(&w, 7);
    w.apply(Command::Sell {
        household: 0,
        object: arc,
    })
    .unwrap();
    for o in &w.objects {
        eprintln!(
            "DBGO {} {} {} rot {}",
            w.content.objects[o.def].id, o.x, o.z, o.rot
        );
    }
    eprintln!(
        "DBGS {:?}",
        w.sims.iter().map(|s| s.pos).collect::<Vec<_>>()
    );
    assert!(free(&w, 6, 2), "the floor under the shade is free again");
}

#[test]
fn what_stands_against_a_wall_needs_one() {
    let mut w = World::from_json(TALL, WINDOWS, 1).unwrap();
    let err = buy(&mut w, "ladder", 5, 4, 0).unwrap_err().to_string();
    assert_eq!(err, "that stands against a wall: put its back to one");
    let err = buy(&mut w, "ladder", 3, 1, 0).unwrap_err().to_string();
    assert_eq!(err, "that can't stand against a window");
    buy(&mut w, "ladder", 7, 1, 0).expect("back to the top wall");
    buy(&mut w, "ladder", 1, 4, 1).expect("or to the left one");
    // Its wall can't come down, nor become a window, behind it.
    use sim_core::command::EdgeKind;
    for kind in [EdgeKind::Open, EdgeKind::Window] {
        let err = w.apply(edge(kind, 7)).unwrap_err().to_string();
        assert!(
            err.contains("towel ladder stands against that wall"),
            "{err}"
        );
    }
    w.apply(edge(EdgeKind::Window, 6))
        .expect("beside it, a window goes in");
}

#[test]
fn a_wall_taken_down_leaves_no_ceiling_lamp_outdoors() {
    let mut w = World::from_json(TALL, WINDOWS, 1).unwrap();
    buy(&mut w, "pendant", 6, 4, 0).unwrap();
    use sim_core::command::EdgeKind;
    let err = w.apply(edge(EdgeKind::Open, 6)).unwrap_err().to_string();
    assert_eq!(err, "the pendant would be left outdoors: move it first");
    assert!(
        w.lot.room_at(6, 4) != sim_core::lot::OUTDOORS,
        "the wall still stands"
    );
    w.apply(Command::Sell {
        household: 0,
        object: w.objects.len() as u32 - 1,
    })
    .unwrap();
    w.apply(edge(EdgeKind::Open, 6))
        .expect("with the lamp gone, it comes down");
}

#[test]
fn the_flags_are_for_their_own_layers() {
    for (flag, layer) in [
        ("overWindow", "floor"),
        ("fillsWall", "wall"),
        ("hangsLow", "floor"),
        ("headHeight", "floor"),
        ("againstWall", "wall"),
    ] {
        let content = format!(
            r#"{{"needs":[{{"id":"energy","label":"Energy"}}],"objects":[{{"id":"x","name":"X","layer":"{layer}","{flag}":true,"interactions":[]}}]}}"#
        );
        let err = World::from_json(&content, WINDOWS, 1)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(err.contains(flag), "{flag} on the {layer}: {err}");
    }
    // A shade over the next tile turns with its thing in quarter turns only: turned between, it would hang over a
    // tile not kept for it.
    let content = r#"{"needs":[{"id":"energy","label":"Energy"}],"objects":[{"id":"x","name":"X","overhang":[[0,1]],"freeRotation":true,"interactions":[]}]}"#;
    let err = World::from_json(content, WINDOWS, 1)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(
        err.contains("quarter turns"),
        "overhang turning freely: {err}"
    );
}

#[test]
fn a_mirror_or_a_mirror_cabinet_hangs_over_every_sink() {
    // The shipped content: every sink against the top wall, a mirror cabinet over it (used to
    // brush teeth, from the tile in front of the sink) and a round mirror over a double's other half.
    let content = common::content();
    for (sink, wide) in [
        ("sink", 1),
        ("sinkPedestal", 1),
        ("sinkVessel", 1),
        ("sinkConsole", 1),
        ("farmhouseSink", 1),
        ("vanityDouble", 2),
        ("counterSinkDouble", 2),
    ] {
        let mut w = World::from_json(&content, TOWN, 1).unwrap();
        w.households[0].funds = 100_000;
        buy(&mut w, sink, 3, 1, 0).unwrap_or_else(|e| panic!("{sink}: {e}"));
        buy(&mut w, "mirrorCabinet", 3, 1, 0)
            .unwrap_or_else(|e| panic!("a mirror cabinet over the {sink}: {e}"));
        if wide == 2 {
            buy(&mut w, "mirrorRound", 4, 1, 0)
                .unwrap_or_else(|e| panic!("a mirror over the {sink}'s other half: {e}"));
        }
        // But not two things on the wall in one place.
        assert!(
            buy(&mut w, "mirrorRound", 3, 1, 0).is_err(),
            "a mirror over the cabinet over the {sink}"
        );
    }
    // Over furniture as tall as the wall, neither hangs.
    let mut w = World::from_json(&content, TOWN, 1).unwrap();
    w.households[0].funds = 100_000;
    for (tall, x) in [("fridgeBuiltIn", 3), ("wardrobe", 6)] {
        buy(&mut w, tall, x, 1, 0).unwrap_or_else(|e| panic!("{tall}: {e}"));
        for mirror in ["mirrorCabinet", "mirrorRound"] {
            let err = buy(&mut w, mirror, x, 1, 0).unwrap_err().to_string();
            assert!(
                err.contains("hang into"),
                "a {mirror} over the {tall}: {err}"
            );
        }
    }
}
