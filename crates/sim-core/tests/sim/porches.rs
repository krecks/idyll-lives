//! Build mode's Porch tool (`World::build_porch`, `World::remove_porch`) and porch pieces
//! (`World::build_porch_piece`, `World::remove_porch_piece`): a porch over an outside door, paid
//! for and sold back, undone and made again, kept in blueprints and saves, gone with its door,
//! moved with its room; posts that nobody walks through and platforms and canopies they walk under
//! and across; and a town or a save from before porches keeping the ones that were drawn.

use serde_json::Value;
use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind};
use sim_core::porches::{PieceKind, Porch};
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
    "objects":[{"id":"chair","name":"Chair","price":100,"interactions":[{"id":"sit","label":"Sit","minutes":30,"effects":{"hunger":0.01}}]}],
    "economy":{"startingFunds":5000},
    "objectRules":{"resale":0.5},
    "build":{"wall":10,"door":50,"window":40,"remove":0,"porch":600,"porchPost":60,"porchCanopy":90,"porchStep":30}}"#;

/// Two empty lots side by side, facing opposite ways: the west lot's street is at its far side
/// (entry at the bottom), the east lot's at its near side. Nobody lives on the east lot.
const TOWN: &str = r#"{"width":24,"depth":14,
    "plots":[{"name":"West","x":0,"z":0,"w":12,"d":12,"entry":[6.5,11.5]},
             {"name":"East","x":12,"z":0,"w":12,"d":12,"entry":[18.5,0.5]}],
    "households":[{"name":"Player","plot":0,"player":true},{"name":"Other","plot":1}],
    "sims":[{"name":"Ada","household":0,"x":1.5,"z":10.5}]}"#;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    w
}

/// A 4×3 room with its corner at (2, 2): its door in the bottom wall at (3, 5), facing +z.
fn build_room(w: &mut World) {
    let mut edits = Vec::new();
    for i in 0..4 {
        edits.push(EdgeEdit::new(EdgeAxis::H, 2 + i, 2, EdgeKind::Wall));
        let kind = if i == 1 {
            EdgeKind::Door
        } else {
            EdgeKind::Wall
        };
        edits.push(EdgeEdit::new(EdgeAxis::H, 2 + i, 5, kind));
    }
    for i in 0..3 {
        edits.push(EdgeEdit::new(EdgeAxis::V, 2, 2 + i, EdgeKind::Wall));
        edits.push(EdgeEdit::new(EdgeAxis::V, 6, 2 + i, EdgeKind::Wall));
    }
    w.apply(Command::Build {
        household: 0,
        edits,
    })
    .unwrap();
}

fn porch(w: &mut World, x: i32, z: i32) -> Result<(), String> {
    w.apply(Command::BuildPorch {
        household: 0,
        axis: EdgeAxis::H,
        x,
        z,
    })
    .map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
fn piece(
    w: &mut World,
    kind: PieceKind,
    x: i32,
    z: i32,
    wd: [u8; 2],
    rot: u8,
) -> Result<(), String> {
    w.apply(Command::BuildPorchPiece {
        household: 0,
        kind,
        x,
        z,
        w: wd[0],
        d: wd[1],
        rot,
    })
    .map_err(|e| e.to_string())
}

fn funds(w: &World) -> i64 {
    w.households[0].funds
}

fn blocked(w: &World, x: i32, z: i32) -> bool {
    w.blocked()[w.lot.tile_index(x, z)]
}

const DOOR: Porch = Porch {
    plot: 0,
    axis: EdgeAxis::H,
    x: 3,
    z: 5,
    out: 1,
};

#[test]
fn a_porch_goes_over_an_outside_door_and_is_sold_back() {
    let mut w = world();
    build_room(&mut w);
    let before = funds(&w);
    assert_eq!(
        porch(&mut w, 2, 5).unwrap_err(),
        "a porch goes over a door",
        "not on a plain wall"
    );
    assert_eq!(porch(&mut w, 3, 2).unwrap_err(), "a porch goes over a door");
    porch(&mut w, 3, 5).expect("a porch over the door");
    assert_eq!(w.porches, vec![DOOR], "outdoors, on the door's +z side");
    assert_eq!(funds(&w), before - 600);
    assert_eq!(
        porch(&mut w, 3, 5).unwrap_err(),
        "that door has a porch already"
    );
    // Its posts don't get in anyone's way: walking stays as it was.
    for (x, z) in DOOR.post_tiles() {
        assert!(!blocked(&w, x, z));
    }
    // The view has it.
    let view: Value = serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    assert_eq!(view["porches"][0]["axis"], "h");
    assert_eq!(view["porches"][0]["out"], 1);
    let shown = w.structure_version();
    w.apply(Command::RemovePorch {
        household: 0,
        axis: EdgeAxis::H,
        x: 3,
        z: 5,
    })
    .unwrap();
    assert!(w.porches.is_empty());
    assert_eq!(funds(&w), before - 300, "half back (objectRules.resale)");
    assert_ne!(w.structure_version(), shown, "the view hears of it");
    assert!(
        w.apply(Command::RemovePorch {
            household: 0,
            axis: EdgeAxis::H,
            x: 3,
            z: 5,
        })
        .is_err()
    );
}

#[test]
fn a_porch_needs_open_ground_and_nothing_at_its_posts() {
    let mut w = world();
    build_room(&mut w);
    w.apply(Command::Buy {
        household: 0,
        object: "chair".into(),
        at: Some([4, 6, 0]),
        colors: Default::default(),
        turn: None,
        offset: None,
        size: None,
        tall: None,
    })
    .unwrap();
    assert_eq!(porch(&mut w, 3, 5).unwrap_err(), "the chair is in the way");
    let chair = w.objects.last().unwrap().stable;
    w.apply(Command::Sell {
        household: 0,
        object: chair,
    })
    .unwrap();
    // A fence across its landing.
    w.apply(Command::Build {
        household: 0,
        edits: vec![EdgeEdit::new(EdgeAxis::V, 4, 5, EdgeKind::Fence)],
    })
    .unwrap();
    assert_eq!(
        porch(&mut w, 3, 5).unwrap_err(),
        "there's a wall or a fence in the way"
    );
    w.apply(Command::Build {
        household: 0,
        edits: vec![EdgeEdit::new(EdgeAxis::V, 4, 5, EdgeKind::Open)],
    })
    .unwrap();
    porch(&mut w, 3, 5).unwrap();
    // Nor does a wall or a fence go through it now.
    let err = w
        .apply(Command::Build {
            household: 0,
            edits: vec![EdgeEdit::new(EdgeAxis::V, 4, 6, EdgeKind::Fence)],
        })
        .unwrap_err();
    assert_eq!(err.to_string(), "a fence can't go through the porch");
    // On the east lot: not the player's.
    assert_eq!(
        w.apply(Command::BuildPorch {
            household: 0,
            axis: EdgeAxis::H,
            x: 15,
            z: 5,
        })
        .unwrap_err()
        .to_string(),
        "a porch goes on your own home"
    );
}

#[test]
fn undo_and_redo_take_a_porch_back_and_build_it_again() {
    let mut w = world();
    build_room(&mut w);
    let before = funds(&w);
    porch(&mut w, 3, 5).unwrap();
    piece(&mut w, PieceKind::Post, 8, 8, [1, 1], 0).unwrap();
    assert!(blocked(&w, 8, 8));
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert!(w.porch_pieces.is_empty());
    assert!(!blocked(&w, 8, 8), "the post's tile is free again");
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert!(w.porches.is_empty());
    assert_eq!(funds(&w), before);
    w.apply(Command::Redo { household: 0 }).unwrap();
    assert_eq!(w.porches, vec![DOOR]);
    w.apply(Command::Redo { household: 0 }).unwrap();
    assert_eq!(w.porch_pieces.len(), 1);
    assert!(blocked(&w, 8, 8), "the post stands again");
}

#[test]
fn a_porch_goes_with_its_door() {
    let mut w = world();
    build_room(&mut w);
    porch(&mut w, 3, 5).unwrap();
    let before = funds(&w);
    // Another door style: still a door, the porch stays.
    let mut restyle = EdgeEdit::new(EdgeAxis::H, 3, 5, EdgeKind::Door);
    restyle.head = Some(200);
    w.apply(Command::Build {
        household: 0,
        edits: vec![restyle],
    })
    .unwrap();
    assert_eq!(w.porches.len(), 1);
    // Walled up: it goes, sold back.
    w.apply(Command::Build {
        household: 0,
        edits: vec![EdgeEdit::new(EdgeAxis::H, 3, 5, EdgeKind::Wall)],
    })
    .unwrap();
    assert!(w.porches.is_empty());
    assert_eq!(
        funds(&w),
        before + 300 - 10,
        "the wall paid, the porch sold back"
    );
    // An undo brings door and porch back together.
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!(w.porches, vec![DOOR]);
    // A window instead of it.
    w.apply(Command::Build {
        household: 0,
        edits: vec![EdgeEdit::new(EdgeAxis::H, 3, 5, EdgeKind::Window)],
    })
    .unwrap();
    assert!(w.porches.is_empty());
}

#[test]
fn a_porch_moves_with_its_room() {
    let mut w = world();
    build_room(&mut w);
    porch(&mut w, 3, 5).unwrap();
    w.apply(Command::MoveRoom {
        household: 0,
        x: 3,
        z: 3,
        dx: 3,
        dz: 1,
    })
    .unwrap();
    assert_eq!(w.porches, vec![Porch { x: 6, z: 6, ..DOOR }]);
    // Moved where there's no ground for it in front of the door (the lot's edge two rows on): it goes.
    let before = funds(&w);
    w.apply(Command::MoveRoom {
        household: 0,
        x: 6,
        z: 4,
        dx: 0,
        dz: 5,
    })
    .unwrap();
    assert!(w.porches.is_empty());
    assert_eq!(funds(&w), before + 300);
}

#[test]
fn posts_block_their_tile_platforms_and_canopies_dont() {
    let mut w = world();
    build_room(&mut w);
    let before = funds(&w);
    piece(&mut w, PieceKind::Post, 8, 7, [1, 1], 0).unwrap();
    piece(&mut w, PieceKind::Step, 7, 6, [3, 2], 0).unwrap();
    piece(&mut w, PieceKind::Canopy, 7, 5, [2, 3], 0).unwrap();
    assert_eq!(funds(&w), before - 60 - 6 * 30 - 6 * 90);
    assert!(blocked(&w, 8, 7), "nobody walks through a post");
    for (x, z) in [(7, 6), (9, 7), (7, 5)] {
        assert!(!blocked(&w, x, z), "({x}, {z}) is walked across");
    }
    // Pieces of a kind don't overlap; a post takes one tile; a canopy is at most 3 across.
    assert_eq!(
        piece(&mut w, PieceKind::Step, 9, 7, [1, 1], 0).unwrap_err(),
        "there's a step platform there already"
    );
    assert_eq!(
        piece(&mut w, PieceKind::Post, 1, 8, [2, 1], 0).unwrap_err(),
        "a post takes one tile"
    );
    assert!(piece(&mut w, PieceKind::Canopy, 0, 8, [4, 1], 0).is_err());
    assert!(
        piece(&mut w, PieceKind::Canopy, 0, 8, [1, 4], 1).is_err(),
        "turned: 4 across"
    );
    piece(&mut w, PieceKind::Canopy, 0, 8, [4, 1], 1).expect("turned: 1 across, 4 deep");
    // Not indoors, not off the lot, not on someone.
    assert_eq!(
        piece(&mut w, PieceKind::Step, 3, 3, [1, 1], 0).unwrap_err(),
        "a step platform goes outdoors, not in a room"
    );
    assert_eq!(
        piece(&mut w, PieceKind::Post, 12, 3, [1, 1], 0).unwrap_err(),
        "a porch post goes on your own lot"
    );
    assert_eq!(
        piece(&mut w, PieceKind::Post, 1, 10, [1, 1], 0).unwrap_err(),
        "someone is standing there"
    );
    // A wall doesn't go through a platform; nothing is put on a post.
    assert_eq!(
        w.apply(Command::Build {
            household: 0,
            edits: vec![EdgeEdit::new(EdgeAxis::V, 8, 6, EdgeKind::Wall)],
        })
        .unwrap_err()
        .to_string(),
        "a wall can't go through the step platform"
    );
    assert!(
        w.apply(Command::Buy {
            household: 0,
            object: "chair".into(),
            at: Some([8, 7, 0]),
            colors: Default::default(),
            turn: None,
            offset: None,
            size: None,
            tall: None,
        })
        .is_err(),
        "no chair on the post"
    );
    // Taken down: the tile is free, half the price back.
    let was = funds(&w);
    w.apply(Command::RemovePorchPiece {
        household: 0,
        kind: PieceKind::Post,
        x: 8,
        z: 7,
    })
    .unwrap();
    assert!(!blocked(&w, 8, 7));
    assert_eq!(funds(&w), was + 30);
}

#[test]
fn a_post_may_not_shut_anyone_in() {
    let mut w = world();
    // Fence Ada in at the lot's corner (x 0..2, z 10..12) but for a gap at (2, 10): a post there
    // shuts her in.
    let mut edits = Vec::new();
    for x in 0..2 {
        edits.push(EdgeEdit::new(EdgeAxis::H, x, 10, EdgeKind::Fence));
        edits.push(EdgeEdit::new(EdgeAxis::H, x, 12, EdgeKind::Fence));
    }
    edits.push(EdgeEdit::new(EdgeAxis::V, 2, 11, EdgeKind::Fence));
    w.apply(Command::Build {
        household: 0,
        edits,
    })
    .unwrap();
    assert_eq!(
        piece(&mut w, PieceKind::Post, 2, 10, [1, 1], 0).unwrap_err(),
        "that would block the way"
    );
    assert!(w.porch_pieces.is_empty());
    assert!(!blocked(&w, 2, 10));
}

#[test]
fn porches_and_pieces_are_kept_in_blueprints_and_saves() {
    let mut w = world();
    build_room(&mut w);
    porch(&mut w, 3, 5).unwrap();
    piece(&mut w, PieceKind::Post, 8, 7, [1, 1], 0).unwrap();
    piece(&mut w, PieceKind::Canopy, 7, 5, [2, 3], 2).unwrap();
    // A save.
    let json = w.save_json();
    let back = World::from_save_json(CONTENT, &json).unwrap();
    assert_eq!(back.porches, w.porches);
    assert_eq!(back.porch_pieces, w.porch_pieces);
    assert!(blocked(&back, 8, 7), "the post stands in the loaded game");
    // A blueprint, built on the east lot (turned round: its street on the other side).
    let bp = w.blueprint(0).unwrap();
    assert_eq!(bp.porches.len(), 1);
    assert_eq!(bp.pieces.len(), 2);
    w.apply(Command::BuildBlueprint {
        household: 1,
        blueprint: bp.clone(),
    })
    .unwrap();
    let east: Vec<&Porch> = w.porches.iter().filter(|p| p.plot == 1).collect();
    assert_eq!(east.len(), 1, "{:?}", w.porches);
    assert_eq!(east[0].out, -1, "turned round, it faces -z");
    assert_eq!(
        w.porch_pieces.iter().filter(|p| p.plot == 1).count(),
        2,
        "{:?}",
        w.porch_pieces
    );
    let again = w.blueprint(1).unwrap();
    assert_eq!(
        serde_json::to_value(&again.porches).unwrap(),
        serde_json::to_value(&bp.porches).unwrap()
    );
    let mut a = serde_json::to_value(&again.pieces).unwrap();
    let mut b = serde_json::to_value(&bp.pieces).unwrap();
    for v in [&mut a, &mut b] {
        v.as_array_mut()
            .unwrap()
            .sort_by_key(|p| p["kind"].as_str().unwrap().to_string());
    }
    assert_eq!(a, b);
}

/// A house with its front door at (4, 6) on a plot whose garden path comes up to it from the street.
const OLD_TOWN: &str = r#"{"width":12,"depth":12,
    "walls":[[2,2,8,2],[8,2,8,6],[2,6,8,6],[2,2,2,6]],
    "doors":[{"x":4,"z":6,"axis":"x"},{"x":6,"z":2,"axis":"x"}],
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[4.5,11.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "meta":{"paths":[{"x":4,"z":6,"w":1,"d":6}]}}"#;

#[test]
fn a_town_or_a_save_from_before_porches_keeps_the_ones_it_had() {
    let w = World::from_json(CONTENT, OLD_TOWN, 1).unwrap();
    assert_eq!(
        w.porches,
        vec![Porch {
            plot: 0,
            axis: EdgeAxis::H,
            x: 4,
            z: 6,
            out: 1,
        }],
        "over the front door the path meets, not the back door"
    );
    // A file with its porches given: as given.
    let none = OLD_TOWN.replacen("\"meta\"", "\"porches\":[],\"meta\"", 1);
    assert!(
        World::from_json(CONTENT, &none, 1)
            .unwrap()
            .porches
            .is_empty()
    );
    // A save from before porches (v38): worked out again.
    let mut save: Value = serde_json::from_str(&w.save_json()).unwrap();
    save["version"] = 38.into();
    save.as_object_mut().unwrap().remove("porches");
    let back = World::from_save_json(CONTENT, &save.to_string()).unwrap();
    assert_eq!(back.porches, w.porches);
    // From v39 on, no porches saved: none.
    save["version"] = 39.into();
    let back = World::from_save_json(CONTENT, &save.to_string()).unwrap();
    assert!(back.porches.is_empty());
}
