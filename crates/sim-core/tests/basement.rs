//! Basements: a storey below the ground, dug out by building walls down there, reached by
//! stairs that stand in the basement and come up inside the house. Residents walk down to use
//! what's there and back up, rooms down there are rooms, and saves keep it all (older saves
//! grow a basement without anything moving).

mod common;

use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind};
use sim_core::content::Pose;
use sim_core::lot::OUTDOORS;
use sim_core::{Command, World, snapshot};

fn content(storeys: u8, basement: bool) -> String {
    format!(
        r#"{{
    "needs":[{{"id":"comfort","label":"Comfort","decayPerHour":0.01}}],
    "objects":[
      {{"id":"stairs","name":"Stairs","price":400,"footprint":[1,4],"stairs":true,"interactions":[]}},
      {{"id":"chair","name":"Chair","price":50,"interactions":[{{"id":"sit","label":"Sit","minutes":60,"pose":"sit","effects":{{"comfort":0.5}}}}]}}],
    "economy":{{"startingFunds":20000}},
    "build":{{"wall":10,"door":50,"window":40,"fence":5,"gate":10,"remove":0,"storeys":{storeys},"basement":{basement}}}}}"#
    )
}

/// One 16×12 plot; the town is 16 deep, so storey 1 is rows 16..32 and the basement (with two
/// storeys above it) rows 32..48. Ada stands in the garden.
const TOWN: &str = r#"{"width":20,"depth":16,
    "plots":[{"name":"Home","x":2,"z":1,"w":16,"d":12,"entry":[9.5,12.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "sims":[{"name":"Ada","x":9.5,"z":11.5}]}"#;

const UP: i32 = 16;
const DOWN: i32 = 32;

fn world_with(storeys: u8, basement: bool) -> World {
    let mut w = World::from_json(&content(storeys, basement), TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    w
}

fn world() -> World {
    world_with(2, true)
}

/// The walls of a `w`×`d` room with its corner at `(x, z)`, and a door at the bottom if `door`.
fn room(x: i32, z: i32, w: i32, d: i32, door: bool) -> Vec<EdgeEdit> {
    let mut edits = Vec::new();
    for i in 0..w {
        edits.push(EdgeEdit::new(EdgeAxis::H, x + i, z, EdgeKind::Wall));
        let kind = if door && i == 1 { EdgeKind::Door } else { EdgeKind::Wall };
        edits.push(EdgeEdit::new(EdgeAxis::H, x + i, z + d, kind));
    }
    for i in 0..d {
        edits.push(EdgeEdit::new(EdgeAxis::V, x, z + i, EdgeKind::Wall));
        edits.push(EdgeEdit::new(EdgeAxis::V, x + w, z + i, EdgeKind::Wall));
    }
    edits
}

fn build(w: &mut World, edits: Vec<EdgeEdit>) -> Result<(), sim_core::Error> {
    w.apply(Command::Build { household: 0, edits })
}

fn buy(w: &mut World, def: &str, x: i32, z: i32, rot: i32) -> Result<u32, sim_core::Error> {
    w.apply(Command::Buy { household: 0, object: def.into(), at: Some([x, z, rot]), colors: Default::default(), turn: None, offset: None, size: None, tall: None })?;
    Ok(w.objects.last().unwrap().stable)
}

fn height(w: &World, i: usize) -> f32 {
    let mut out = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut out);
    out[snapshot::HEADER_LEN + i * snapshot::SIM_STRIDE + snapshot::sim::HEIGHT]
}

/// A 6×6 room on the ground (door at the bottom), a bigger basement room under it reaching
/// under the garden behind, and stairs down along the right wall: they stand in the basement
/// at x = 9, rows 3..7, climbing towards the back, and come up onto the landing at (9, 2).
fn house() -> World {
    let mut w = world();
    assert_eq!(w.lot.depth, 48, "two storeys and a basement of 16 rows");
    build(&mut w, room(4, 2, 6, 6, true)).unwrap();
    build(&mut w, room(4, 2 + DOWN, 8, 9, false)).unwrap();
    buy(&mut w, "stairs", 9, 3 + DOWN, 0).unwrap();
    w
}

/// Runs until resident 0 sits down on the chair's storey (or gives up), noting the heights
/// they passed.
fn walk_to_sit(w: &mut World, chair: u32) -> Vec<f32> {
    w.apply(Command::Use { sim: 0, object: chair, interaction: 0 }).unwrap();
    let storey = w.storey_of(w.objects[w.index_of(chair).unwrap()].z);
    let mut heights = Vec::new();
    for _ in 0..20 * 60 * 5 {
        w.tick_once();
        heights.push(height(w, 0));
        if w.sims[0].pose == Pose::Sit && w.storey_of(w.sims[0].tile().1) == storey {
            break;
        }
    }
    heights
}

#[test]
fn the_lot_holds_a_basement_below_the_ground() {
    let w = world();
    assert_eq!((w.storeys, w.lowest_storey()), (2, -1));
    assert_eq!(w.storey_range(), -1..2);
    assert_eq!((w.storey_of(5), w.storey_of(5 + UP), w.storey_of(5 + DOWN)), (0, 1, -1));
    assert_eq!((w.storey_row(0), w.storey_row(1), w.storey_row(-1)), (0, UP, DOWN));
    assert_eq!(w.ground_z(5 + DOWN), 5);
    // Without a basement in the content, none.
    let flat = world_with(2, false);
    assert_eq!((flat.lot.depth, flat.lowest_storey()), (32, 0));
    // A basement under a one-storey house still makes a second block of rows.
    let low = world_with(1, true);
    assert_eq!((low.lot.depth, low.storey_of(5 + UP)), (32, -1));
    let structure: serde_json::Value = serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    assert_eq!((structure["storeys"].as_i64(), structure["basement"].as_i64()), (Some(2), Some(2)));
}

#[test]
fn a_basement_room_is_a_room_even_under_the_garden() {
    let mut w = world();
    // Under the garden, with nothing built above: still a room, and not outdoors.
    build(&mut w, room(4, 2 + DOWN, 6, 6, false)).unwrap();
    let id = w.lot.room_at(6, 4 + DOWN);
    assert_ne!(id, OUTDOORS);
    assert_eq!(w.lot.room_at(6, 4), OUTDOORS, "the garden above stays outdoors");
    let info = w.rooms().iter().find(|r| r.id == id && !r.garden).expect("scored like any room");
    assert_eq!((info.tiles, info.windows), (36.0, 0));
    // No window could go in: no daylight to miss, its lamps (none yet) are all its light.
    assert!(!info.outside);
    assert_eq!(info.scores.light, 0.0);
    // Like upstairs: nothing stands outside rooms, and outside rooms isn't floor to stand on.
    assert!(buy(&mut w, "chair", 12, 4 + DOWN, 0).is_err());
    assert!(buy(&mut w, "chair", 5, 3 + DOWN, 0).is_ok());
    let nav = sim_core::path::NavGrid { lot: &w.lot, blocked: w.blocked(), stairs: w.stair_map() };
    assert!(!nav.tile_free(12, 4 + DOWN) && nav.tile_free(6, 6 + DOWN));
}

#[test]
fn a_basement_has_no_windows_and_stays_on_the_plot() {
    let mut w = world();
    build(&mut w, room(4, 2 + DOWN, 6, 6, false)).unwrap();
    let window = build(&mut w, vec![EdgeEdit::new(EdgeAxis::H, 6, 2 + DOWN, EdgeKind::Window)]).unwrap_err();
    assert!(window.to_string().contains("no windows"), "{window}");
    assert!(build(&mut w, vec![EdgeEdit::new(EdgeAxis::H, 12, 4 + DOWN, EdgeKind::Fence)]).is_err());
    // Doors between basement rooms are fine.
    build(&mut w, room(10, 2 + DOWN, 3, 6, false)).unwrap();
    build(&mut w, vec![EdgeEdit::new(EdgeAxis::V, 10, 4 + DOWN, EdgeKind::Door)]).unwrap();
    // Not past the plot (x 2..18, rows 1..13 of each storey).
    let off = build(&mut w, vec![EdgeEdit::new(EdgeAxis::V, 19, 4 + DOWN, EdgeKind::Wall)]).unwrap_err();
    assert!(off.to_string().contains("own lot"), "{off}");
    assert!(build(&mut w, vec![EdgeEdit::new(EdgeAxis::H, 5, 14 + DOWN, EdgeKind::Wall)]).is_err());
}

#[test]
fn stairs_down_start_inside_the_house() {
    let mut w = world();
    build(&mut w, room(4, 2 + DOWN, 8, 9, false)).unwrap();
    // Nothing built over them: they'd come up in the garden.
    let err = buy(&mut w, "stairs", 9, 3 + DOWN, 0).unwrap_err();
    assert!(err.to_string().contains("inside the house"), "{err}");
    build(&mut w, room(4, 2, 6, 6, true)).unwrap();
    // One row further back, the top would come up against the back wall.
    let err = buy(&mut w, "stairs", 9, 2 + DOWN, 0).unwrap_err();
    assert!(err.to_string().contains("clear way off"), "{err}");
    let stairs = buy(&mut w, "stairs", 9, 3 + DOWN, 0).unwrap();
    let s = w.stair_map().stairs[0];
    assert_eq!((s.bottom, s.top(), s.landing), ((9, 6 + DOWN), (9, 3 + DOWN), (9, 2)), "up to the ground floor");
    assert_eq!(w.storey_of(w.objects[w.index_of(stairs).unwrap()].z), -1);
    // The stairwell in the ground floor is open: nothing stands there.
    assert!(buy(&mut w, "chair", 9, 4, 0).is_err(), "the stairwell is open");
    let nav = sim_core::path::NavGrid { lot: &w.lot, blocked: w.blocked(), stairs: w.stair_map() };
    assert!(!nav.tile_free(9, 4) && nav.tile_free(9, 2), "a hole over the steps, the landing free");
    // Taking the room away would leave them coming up in the garden.
    let down: Vec<EdgeEdit> = room(4, 2, 6, 6, true).into_iter().map(|e| EdgeEdit { kind: EdgeKind::Open, ..e }).collect();
    let err = build(&mut w, down).unwrap_err();
    assert!(err.to_string().contains("room over them"), "{err}");
}

#[test]
fn residents_walk_down_to_the_basement_and_back_up() {
    let mut w = house();
    let below = buy(&mut w, "chair", 5, 3 + DOWN, 0).unwrap();
    let above = buy(&mut w, "chair", 5, 3, 0).unwrap();
    let heights = walk_to_sit(&mut w, below);
    assert_eq!(w.sims[0].pose, Pose::Sit, "sat down in the basement");
    assert!(heights.iter().any(|&h| h > -0.8 && h < -0.2), "went down step by step");
    assert!(heights.iter().all(|&h| (-1.0..=0.0).contains(&h)), "never above the ground floor");
    assert_eq!(w.storey_of(w.sims[0].tile().1), -1);
    assert_eq!(height(&w, 0), -1.0);
    assert_eq!(w.plot_at(w.sims[0].tile().0, w.sims[0].tile().1), Some(0), "still at home");

    // And back up to the ground floor.
    let heights = walk_to_sit(&mut w, above);
    assert_eq!(w.sims[0].pose, Pose::Sit, "sat down on the ground floor");
    assert!(heights.iter().any(|&h| h > -0.8 && h < -0.2), "went up step by step");
    assert_eq!((w.storey_of(w.sims[0].tile().1), height(&w, 0)), (0, 0.0));
}

#[test]
fn saves_keep_the_basement_and_older_saves_grow_one() {
    let mut w = house();
    buy(&mut w, "chair", 5, 3 + DOWN, 0).unwrap();
    let saved = w.save_json();
    let loaded = World::from_save_json(&content(2, true), &saved).unwrap();
    assert_eq!((loaded.storeys, loaded.lowest_storey(), loaded.lot.depth), (2, -1, 48));
    assert_ne!(loaded.lot.room_at(6, 4 + DOWN), OUTDOORS);
    assert_eq!(loaded.objects.len(), w.objects.len());
    assert_eq!(loaded.stair_map().stairs, w.stair_map().stairs);
    assert_eq!(loaded.save_json(), saved, "a round trip changes nothing");

    // A game from before basements (v21: no `basement`), with a room upstairs: it grows one on
    // the end and everything stays where it was.
    let mut old = world_with(2, false);
    build(&mut old, room(4, 2, 6, 6, true)).unwrap();
    build(&mut old, room(4, 2 + UP, 6, 6, false)).unwrap();
    buy(&mut old, "stairs", 9, 3, 0).unwrap();
    let mut file: serde_json::Value = serde_json::from_str(&old.save_json()).unwrap();
    assert!(file.get("basement").is_none());
    file["version"] = 21.into();
    let grown = World::from_save_json(&content(2, true), &file.to_string()).unwrap();
    assert_eq!((grown.storeys, grown.lowest_storey(), grown.lot.depth), (2, -1, 48));
    assert_ne!(grown.lot.room_at(6, 4 + UP), OUTDOORS, "the room upstairs stays upstairs");
    assert_eq!(grown.stair_map().stairs, old.stair_map().stairs);
    assert_eq!(grown.storey_of(4 + DOWN), -1);

    // Content that later allows a storey more: it goes on after the basement, which stays put.
    let taller = World::from_save_json(&content(3, true), &saved).unwrap();
    assert_eq!((taller.storeys, taller.lot.depth), (3, 64));
    assert_eq!((taller.storey_of(4 + DOWN), taller.storey_of(4 + 48), taller.storey_row(2)), (-1, 2, 48));
    assert_ne!(taller.lot.room_at(6, 4 + DOWN), OUTDOORS);
    assert_eq!(taller.stair_map().stairs, w.stair_map().stairs);
    // Content without basements keeps the one a game has.
    let kept = World::from_save_json(&content(2, false), &saved).unwrap();
    assert_eq!((kept.lowest_storey(), kept.lot.depth), (-1, 48));
}

#[test]
fn a_house_with_a_basement_goes_through_a_blueprint() {
    let two_lots = r#"{"width":40,"depth":16,
        "plots":[{"name":"Home","x":2,"z":1,"w":16,"d":12,"entry":[9.5,12.5]},{"name":"Next door","x":22,"z":1,"w":16,"d":12,"entry":[29.5,12.5]}],
        "households":[{"name":"Player","plot":0,"player":true},{"name":"Other","plot":1}],
        "sims":[{"name":"Ada","household":0,"x":9.5,"z":11.5}]}"#;
    let mut w = World::from_json(&content(2, true), two_lots, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    build(&mut w, room(4, 2, 6, 6, true)).unwrap();
    build(&mut w, room(4, 2 + DOWN, 8, 9, false)).unwrap();
    buy(&mut w, "stairs", 9, 3 + DOWN, 0).unwrap();
    buy(&mut w, "chair", 5, 3 + DOWN, 0).unwrap();
    let bp = w.blueprint(0).unwrap();
    assert!(bp.edges.iter().any(|e| e.storey == -1));
    assert!(bp.objects.iter().any(|o| o.storey == -1));
    w.apply(Command::BuildBlueprint { household: 1, blueprint: bp.clone() }).unwrap();
    assert_ne!(w.lot.room_at(26, 4 + DOWN), OUTDOORS, "the basement next door");
    assert!(w.objects.iter().any(|o| o.def == 1 && o.x >= 22 && w.storey_of(o.z) == -1), "with its chair");
    assert_eq!(w.stair_map().stairs.len(), 2);
    // A game without basements can't build it.
    let mut flat = World::from_json(&content(2, false), two_lots, 1).unwrap();
    let err = flat.apply(Command::BuildBlueprint { household: 1, blueprint: bp }).unwrap_err();
    assert!(err.to_string().contains("no basements"), "{err}");
}

#[test]
fn a_town_file_can_bring_a_basement() {
    // One storey and a basement: the basement's walls and things on the block after the
    // ground's (rows 16..32), the stairs down from the room above; the storey the content adds
    // goes after it.
    let room = |z: i32| -> String {
        format!("[4,{z},10,{z}],[4,{b},10,{b}],[4,{z},4,{b}],[10,{z},10,{b}]", b = z + 6)
    };
    let town = format!(
        r#"{{"width":20,"depth":16,"storeys":1,"basement":true,
        "walls":[{},{}],"doors":[{{"x":5,"z":8,"axis":"x"}}],
        "objects":[{{"def":"stairs","x":9,"z":19,"rot":0}},{{"def":"chair","x":5,"z":19,"rot":0}}],
        "plots":[{{"name":"Home","x":2,"z":1,"w":16,"d":12,"entry":[9.5,12.5]}}],
        "households":[{{"name":"Player","plot":0,"player":true}}],
        "sims":[{{"name":"Ada","x":9.5,"z":11.5}}]}}"#,
        room(2),
        room(2 + UP)
    );
    let mut w = World::from_json(&content(2, true), &town, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    assert_eq!((w.storeys, w.lowest_storey(), w.lot.depth), (2, -1, 48));
    assert_eq!((w.storey_row(-1), w.storey_row(1)), (UP, DOWN), "the basement where the file put it, the storey added after");
    assert_ne!(w.lot.room_at(6, 4 + UP), OUTDOORS);
    let s = w.stair_map().stairs[0];
    assert_eq!(s.landing, (9, 2), "up into the room on the ground");
    let chair = w.objects.last().unwrap().stable;
    walk_to_sit(&mut w, chair);
    assert_eq!((w.sims[0].pose, w.storey_of(w.sims[0].tile().1)), (Pose::Sit, -1), "went down to sit");
}

#[test]
fn a_shipped_house_gets_a_basement() {
    // The shipped content allows one: dug out under the player's house, stairs down from
    // inside, a gym and a den down there, and a resident who goes down to work out.
    let content = common::content();
    let file = common::town(&content, 5, 1);
    let mut w = World::from_json(&content, &file, 5).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    assert_eq!(w.lowest_storey(), -1, "the shipped content has basements");
    let p = w.plots[w.households[0].plot.unwrap() as usize].clone();
    w.households[0].funds = 1_000_000;
    // Under the house: the box round its ground floor rooms.
    let indoor: Vec<(i32, i32)> = (p.z..p.z + p.d).flat_map(|z| (p.x..p.x + p.w).map(move |x| (x, z))).filter(|&(x, z)| w.lot.room_at(x, z) != OUTDOORS).collect();
    let (x0, z0) = (indoor.iter().map(|t| t.0).min().unwrap(), indoor.iter().map(|t| t.1).min().unwrap());
    let (x1, z1) = (indoor.iter().map(|t| t.0).max().unwrap() + 1, indoor.iter().map(|t| t.1).max().unwrap() + 1);
    let down = w.storey_row(-1);
    let mut edits = room(x0, z0 + down, x1 - x0, z1 - z0, false);
    // Split in two, with a door between.
    let mid = z0 + (z1 - z0) / 2;
    edits.extend((x0..x1).map(|x| EdgeEdit::new(EdgeAxis::H, x, mid + down, if x == x0 + 1 { EdgeKind::Door } else { EdgeKind::Wall })));
    build(&mut w, edits).unwrap();
    assert_ne!(w.lot.room_at(x0, z0 + down), OUTDOORS);
    // Stairs down wherever they fit.
    let spots: Vec<(i32, i32, i32)> = (z0..z1).flat_map(|z| (x0..x1).flat_map(move |x| (0..4).map(move |r| (x, z, r)))).collect();
    assert!(spots.iter().any(|&(x, z, r)| buy(&mut w, "stairs", x, z + down, r).is_ok()), "the stairs fit somewhere");
    let mut placed = Vec::new();
    for def in ["treadmill", "weightBench", "sofa", "tv", "bookshelf", "lamp", "workbench", "armchair"] {
        placed.extend(spots.iter().find_map(|&(x, z, r)| buy(&mut w, def, x, z + down, r).ok()));
    }
    assert!(placed.len() >= 5, "furnished: {placed:?}");
    w.apply(Command::Use { sim: 0, object: placed[0], interaction: 0 }).unwrap();
    for _ in 0..20 * 60 * 5 {
        w.tick_once();
        if w.sims[0].current().is_some_and(|a| matches!(a.phase, sim_core::world::Phase::Using { .. })) {
            break;
        }
    }
    assert_eq!(w.storey_of(w.sims[0].tile().1), -1, "went down to work out");
    common::dump_state(&w, "basement");
}
