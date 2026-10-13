//! Rooms and their scores: kind from what stands in them, size, light, decor, cleanliness and
//! function, and each plot's garden (see `docs/design/house-matters.md`).

use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind, FloorPaint};
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
    "objects":[
      {"id":"toilet","name":"Toilet","price":100,"interactions":[{"id":"use","label":"Use","minutes":5,"tags":["bathroom"]}]},
      {"id":"sink","name":"Sink","price":100,"interactions":[{"id":"wash","label":"Wash","minutes":5,"tags":["hygiene"]}]},
      {"id":"sofa","name":"Sofa","price":100,"footprint":[2,1],"interactions":[{"id":"sit","label":"Sit","minutes":30,"tags":["lounge"]}]},
      {"id":"bed","name":"Bed","price":100,"footprint":[2,2],"interactions":[{"id":"sleep","label":"Sleep","minutes":480,"tags":["sleep"]}]},
      {"id":"lamp","name":"Lamp","price":30,"category":"decor","light":{"range":4}},
      {"id":"chandelier","name":"Chandelier","price":300,"layer":"ceiling","light":{"range":6.5,"tiles":24}},
      {"id":"candle","name":"Candle","price":5,"light":{"range":2,"tiles":4}},
      {"id":"plant","name":"Plant","price":20,"category":"garden"}],
    "roomKinds":[
      {"id":"bathroom","tags":["bathroom"],"essentials":[["bathroom"],["hygiene"]],"size":[3,6],"exclusive":true},
      {"id":"bedroom","tags":["sleep"],"essentials":[["sleep"]],"size":[6,12],"exclusive":true},
      {"id":"living","tags":["lounge"],"size":[9,20]}],
    "roomRules":{"decorByCategory":{"decor":1,"garden":0.8}},
    "floorCoverings":[{"id":"oak","price":0}],
    "economy":{"startingFunds":10000}}"#;

/// A 16×12 plot: a 2×2 bathroom, a 5×4 living room with three windows and a lamp, and a 3×3
/// bedroom with a toilet in it.
const TOWN: &str = r#"{"width":16,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":12,"entry":[8.5,11.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "walls":[[1,1,3,1],[1,3,3,3],[1,1,1,3],[3,1,3,3],
             [5,1,10,1],[5,5,10,5],[5,1,5,5],[10,1,10,5],
             [11,1,14,1],[11,4,14,4],[11,1,11,4],[14,1,14,4]],
    "doors":[{"x":1,"z":3,"axis":"x"},{"x":6,"z":5,"axis":"x"},{"x":12,"z":4,"axis":"x"}],
    "windows":[{"x":1,"z":1,"axis":"z"},{"x":6,"z":1,"axis":"x"},{"x":8,"z":1,"axis":"x"},{"x":10,"z":2,"axis":"z"}],
    "objects":[{"def":"toilet","x":1,"z":1},{"def":"sink","x":2,"z":1},
               {"def":"sofa","x":6,"z":3},{"def":"lamp","x":5,"z":1},
               {"def":"bed","x":11,"z":1},{"def":"toilet","x":13,"z":2},
               {"def":"plant","x":2,"z":8},{"def":"plant","x":4,"z":8}]}"#;

fn room(w: &World, x: i32, z: i32) -> &sim_core::rooms::RoomInfo {
    w.room_at_tile(x, z).unwrap()
}

fn kind(w: &World, r: &sim_core::rooms::RoomInfo) -> String {
    r.kind.map_or("none".into(), |k| w.content.room_kinds[k].id.clone())
}

#[test]
fn rooms_get_a_kind_and_scores() {
    let w = World::from_json(CONTENT, TOWN, 1).unwrap();
    assert_eq!(w.rooms().iter().filter(|r| !r.garden).count(), 3);

    let bath = room(&w, 1, 1);
    assert_eq!(kind(&w, bath), "bathroom");
    assert_eq!((bath.tiles, bath.windows, bath.doors), (4.0, 1, 1));
    assert!((bath.scores.size - (0.5 + 0.5 / 3.0)).abs() < 1e-4, "4 tiles between cramped 3 and comfortable 6");
    assert!((bath.scores.light - 0.5).abs() < 1e-4, "full daylight from one window, no lamp at night");
    assert_eq!((bath.scores.function, bath.scores.clean), (1.0, 1.0));

    let living = room(&w, 7, 2);
    assert_eq!(kind(&w, living), "living");
    assert_eq!((living.tiles, living.windows, living.lamps), (20.0, 3, 1));
    assert!((living.scores.light - (0.9 + 0.5) / 2.0).abs() < 1e-4);
    assert!((living.scores.decor - (1.0 / (20.0 * 0.125)) * 0.8).abs() < 1e-4, "the lamp is decor too");

    let mixed = room(&w, 12, 2);
    assert_eq!(kind(&w, mixed), "bathroom", "the first kind that fits");
    assert!(mixed.mixed);
    assert_eq!(mixed.scores.function, 0.6);
    assert!(mixed.scores.overall < bath.scores.overall + 0.2);

    let garden = room(&w, 3, 9);
    assert!(garden.garden);
    assert!(garden.decor > 1.5, "two plants");
    assert!(garden.scores.decor > 0.0 && garden.scores.decor < 1.0);
}

#[test]
fn scores_follow_the_house_as_it_changes() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let before = room(&w, 1, 1).scores;
    // A floor covering is decor.
    let tiles = (1..3).flat_map(|x| (1..3).map(move |z| FloorPaint { x, z, covering: 1 })).collect();
    w.apply(Command::PaintFloor { household: 0, tiles }).unwrap();
    assert!(room(&w, 1, 1).scores.decor > before.decor);

    // Without the sink, the bathroom misses somewhere to wash.
    w.apply(Command::Sell { household: 0, object: 1 }).unwrap();
    let bath = room(&w, 1, 1);
    assert_eq!(bath.missing, Some(1));
    assert_eq!(bath.scores.function, 0.7);

    // Splitting the living room: two smaller rooms, one with the window on the right.
    let edits = (1..5).map(|z| EdgeEdit::new(EdgeAxis::V, 9, z, EdgeKind::Wall)).collect();
    w.apply(Command::Build { household: 0, edits }).unwrap();
    assert_eq!(room(&w, 9, 3).tiles, 4.0);
    assert_eq!(room(&w, 9, 3).windows, 1);
    assert_eq!(room(&w, 7, 2).tiles, 16.0);
    // Undo puts it back.
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!(room(&w, 7, 2).tiles, 20.0);

    // Dirt makes a room less clean.
    let i = w.lot.tile_index(1, 1);
    w.dirt[i] = 1.0;
    w.refresh_rooms();
    assert!((room(&w, 1, 1).scores.clean - 0.75).abs() < 1e-4);
}

/// A 7×5 house: a 2×2 closet in the middle (a door into it), and the rest one room wrapped round
/// it with the front door. Neither has a window.
const CLOSET: &str = r#"{"width":12,"depth":10,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[4.5,8.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "walls":[[1,1,8,1],[1,6,8,6],[1,1,1,6],[8,1,8,6],
             [3,2,5,2],[3,4,5,4],[3,2,3,4],[5,2,5,4]],
    "doors":[{"x":4,"z":6,"axis":"x"},{"x":3,"z":4,"axis":"x"}],
    "objects":[{"def":"lamp","x":3,"z":2},{"def":"lamp","x":7,"z":2}]}"#;

/// A room with no outside wall can't have a window: its lamps light it by day too. A room with
/// outside walls and no windows still misses its daylight (a window could go in).
#[test]
fn a_room_with_no_outside_wall_is_lit_by_its_lamps() {
    let mut w = World::from_json(CONTENT, CLOSET, 1).unwrap();
    let closet = room(&w, 3, 2);
    assert_eq!((closet.tiles, closet.windows, closet.lamps, closet.outside), (4.0, 0, 1, false));
    assert!((closet.scores.light - 1.0).abs() < 1e-4, "one lamp for 4 tiles, by day too: {}", closet.scores.light);
    let ring = room(&w, 1, 1);
    assert_eq!((ring.tiles, ring.windows, ring.lamps, ring.outside), (31.0, 0, 1, true));
    assert!((ring.scores.light - 0.5 * 10.0 / 31.0).abs() < 1e-4, "no daylight: {}", ring.scores.light);

    // Without its lamp the closet is dark.
    w.apply(Command::Sell { household: 0, object: 0 }).unwrap();
    assert_eq!(room(&w, 3, 2).scores.light, 0.0);
    // With a window the ring gets daylight.
    w.apply(Command::Build { household: 0, edits: vec![EdgeEdit::new(EdgeAxis::H, 6, 1, EdgeKind::Window)] }).unwrap();
    let ring = room(&w, 1, 1);
    assert!((ring.scores.light - 0.5 * (6.0 + 10.0) / 31.0).abs() < 1e-4, "{}", ring.scores.light);
    // Moving a closet wall onto the outside wall makes it an outside room (dark by day).
    let edits = vec![EdgeEdit::new(EdgeAxis::H, 3, 2, EdgeKind::Open), EdgeEdit::new(EdgeAxis::H, 4, 2, EdgeKind::Open), EdgeEdit::new(EdgeAxis::V, 3, 1, EdgeKind::Wall), EdgeEdit::new(EdgeAxis::V, 5, 1, EdgeKind::Wall)];
    w.apply(Command::Build { household: 0, edits }).unwrap();
    let closet = room(&w, 3, 1);
    assert_eq!((closet.tiles, closet.outside), (6.0, true));
}

/// A lamp lights as much floor as its `light.tiles` says, else `roomRules.lampTiles` (10): a
/// chandelier lights all of the 5×4 living room at night, a candle a fifth of it.
#[test]
fn lamps_light_as_much_floor_as_they_say() {
    for (def, lit) in [("lamp", 10.0), ("chandelier", 24.0), ("candle", 4.0)] {
        let town = TOWN.replace(r#""def":"lamp""#, &format!(r#""def":"{def}""#));
        let w = World::from_json(CONTENT, &town, 1).unwrap();
        let living = room(&w, 7, 2);
        assert_eq!((living.lamps, living.lit), (1, lit), "{def}");
        let night = (lit / 20.0).min(1.0);
        assert!((living.scores.light - (0.9 + night) / 2.0).abs() < 1e-4, "{def}: three windows by day, the {def} at night");
    }
}

/// A lamp nobody uses could stand anywhere that leaves the rest reachable; it keeps out of
/// doorways (the tiles either side of a door), bought or moved there.
#[test]
fn lamps_keep_out_of_doorways() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let buy = |w: &mut World, x: i32, z: i32, rot: i32| w.apply(Command::Buy { household: 0, object: "lamp".into(), at: Some([x, z, rot]), style: None, turn: None, offset: None, size: None, tall: None });
    // The living room's door is on its bottom wall at x 6: (6, 4) inside, (6, 5) outside.
    for (x, z, rot) in [(6, 4, 2), (6, 5, 0)] {
        let err = buy(&mut w, x, z, rot).unwrap_err().to_string();
        assert!(err.contains("doorway"), "{x},{z}: {err}");
    }
    buy(&mut w, 8, 4, 2).expect("further along the wall is fine");
    // Nor moved there.
    let lamp = w.objects.iter().position(|o| w.content.objects[o.def].id == "lamp" && (o.x, o.z) == (5, 1)).unwrap() as u32;
    let to = |x, z| Command::MoveObject { household: 0, object: lamp, x, z, rot: 2, turn: None, offset: None, size: None, tall: None };
    assert!(w.apply(to(6, 4)).unwrap_err().to_string().contains("doorway"));
    assert_eq!((w.objects[lamp as usize].x, w.objects[lamp as usize].z), (5, 1), "it stays put");
    w.apply(to(9, 4)).expect("into a corner");
}

/// Two equal rooms with a sofa each: the left one bright and decorated, the right one dark and
/// dirty. Ada stands halfway between the sofas.
const TWO_ROOMS: &str = r#"{"width":20,"depth":10,
    "plots":[{"name":"Home","x":0,"z":0,"w":20,"d":10,"entry":[10.5,9.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "walls":[[1,1,6,1],[1,5,6,5],[1,1,1,5],[6,1,6,5],[13,1,18,1],[13,5,18,5],[13,1,13,5],[18,1,18,5]],
    "doors":[{"x":3,"z":5,"axis":"x"},{"x":15,"z":5,"axis":"x"}],
    "windows":[{"x":2,"z":1,"axis":"x"},{"x":4,"z":1,"axis":"x"},{"x":1,"z":2,"axis":"z"}],
    "objects":[{"def":"sofa","x":2,"z":2},{"def":"sofa","x":14,"z":2},
               {"def":"lamp","x":5,"z":1},{"def":"plant","x":1,"z":1},{"def":"plant","x":5,"z":3}],
    "sims":[{"name":"Ada","x":9.5,"z":7.5}]}"#;

const ROOM_CONTENT: &str = r#"{
    "needs":[{"id":"comfort","label":"Comfort","decayPerHour":0.05},{"id":"environment","label":"Surroundings","room":true}],
    "objects":[
      {"id":"sofa","name":"Sofa","price":100,"footprint":[2,1],"interactions":[{"id":"sit","label":"Sit","minutes":30,"effects":{"comfort":0.6},"tags":["lounge"]}]},
      {"id":"lamp","name":"Lamp","price":30,"category":"decor","light":{"range":4}},
      {"id":"plant","name":"Plant","price":20,"category":"garden"}],
    "roomKinds":[{"id":"living","tags":["lounge"],"size":[9,20]}],
    "roomRules":{"decorByCategory":{"decor":1,"garden":0.8},"drift":0.5,"preference":1.0}}"#;

fn two_rooms(seed: u32) -> World {
    let mut w = World::from_json(ROOM_CONTENT, TWO_ROOMS, seed).unwrap();
    // The right room is filthy.
    for z in 1..5 {
        for x in 13..18 {
            let i = w.lot.tile_index(x, z);
            w.dirt[i] = 0.8;
        }
    }
    w.refresh_rooms();
    w
}

#[test]
fn surroundings_follow_the_room_and_draw_residents_to_nicer_ones() {
    let w = two_rooms(1);
    let (good, bad) = (room(&w, 3, 3).scores.overall, room(&w, 15, 3).scores.overall);
    assert!(good > 0.7 && bad < 0.4, "good {good}, bad {bad}");

    // Sitting in the bright room for four hours lifts the need towards its score.
    let mut w = two_rooms(1);
    w.sims[0].pos = [3.5, 3.5];
    w.sims[0].needs[1] = 0.2;
    w.apply(sim_core::Command::SetAutonomy { enabled: false, household: None }).unwrap();
    for _ in 0..4 * 60 * 20 {
        w.tick_once();
    }
    let need = w.sims[0].needs[1];
    assert!((need - good).abs() < 0.15, "surroundings {need} near the room's {good}");

    // Low on comfort and in poor surroundings, Ada mostly picks the sofa in the nicer room.
    let (mut nice, mut grim) = (0, 0);
    for seed in 1..=60 {
        let mut w = two_rooms(seed);
        w.sims[0].needs = [0.05, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        for _ in 0..200 {
            w.tick_once();
            // Heading for (or sitting on) a sofa holds a slot on it.
            if w.objects[0].users().any(|u| u == 0) {
                nice += 1;
                break;
            }
            if w.objects[1].users().any(|u| u == 0) {
                grim += 1;
                break;
            }
        }
    }
    assert!(nice > grim + 10, "nice room {nice}, grim room {grim}");
}

const MESS_CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.2},{"id":"environment","label":"Surroundings","room":true}],
    "traits":[{"id":"neat","label":"Neat","effects":{"tagPreference":{"chores":3}}}],
    "objects":[
      {"id":"fridge","name":"Fridge","price":100,"interactions":[{"id":"cook","label":"Cook","minutes":30,"effects":{"hunger":1},"tags":["cooking"]}]}],
    "activities":[{"id":"chores","label":"Chores","icon":"","tags":["chores","cleaning"]}],
    "roomKinds":[{"id":"kitchen","tags":["cooking"],"size":[6,12],"exclusive":true}],
    "roomRules":{"dirt":{"cooking":0.3},"clean":{"minutes":10,"threshold":0.15,"amount":0.7,"interest":0.6}}}"#;

const KITCHEN: &str = r#"{"width":12,"depth":10,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "walls":[[1,1,7,1],[1,6,7,6],[1,1,1,6],[7,1,7,6]],
    "doors":[{"x":3,"z":6,"axis":"x"}],
    "objects":[{"def":"fridge","x":2,"z":1}],
    "sims":[{"name":"Ada","x":3.5,"z":4.5,"traits":["neat"]}]}"#;

#[test]
fn using_things_makes_a_mess_and_residents_tidy_it_up() {
    let mut w = World::from_json(MESS_CONTENT, KITCHEN, 3).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    // Cooking leaves a mess in front of the fridge (and a little around it).
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    for _ in 0..45 * 20 {
        w.tick_once();
    }
    let front = w.lot.tile_index(2, 2);
    assert!((w.dirt[front] - 0.3).abs() < 1e-4, "dirt {}", w.dirt[front]);
    assert!((w.dirt[w.lot.tile_index(3, 2)] - 0.15).abs() < 1e-4);
    assert_eq!(w.dirt[w.lot.tile_index(2, 7)], 0.0, "not through the wall");
    w.refresh_rooms();
    let dirty = room(&w, 2, 2).scores.clean;
    assert!(dirty < 1.0);

    // Saved and loaded.
    let loaded = World::from_save_json(MESS_CONTENT, &w.save_json()).unwrap();
    assert!((loaded.dirt[front] - 0.3).abs() < 0.01);

    // With free will, the neat resident tidies it up.
    w.apply(Command::SetAutonomy { enabled: true, household: None }).unwrap();
    w.sims[0].needs[0] = 1.0;
    let mut tidied = false;
    for _ in 0..3 * 60 * 20 {
        w.tick_once();
        if w.dirt[front] < 0.05 {
            tidied = true;
            break;
        }
    }
    assert!(tidied, "dirt left {}", w.dirt[front]);
    w.refresh_rooms();
    assert!(room(&w, 2, 2).scores.clean > dirty);
}

#[test]
fn two_tidy_residents_dont_both_go_for_the_same_mess() {
    let town = KITCHEN.replace(r#""sims":[{"name":"Ada","x":3.5,"z":4.5,"traits":["neat"]}]"#, r#""sims":[{"name":"Ada","x":3.5,"z":4.5,"traits":["neat"]},{"name":"Bo","x":9.5,"z":8.5,"traits":["neat"]}]"#);
    let mut w = World::from_json(MESS_CONTENT, &town, 3).unwrap();
    let (worst, other) = (w.lot.tile_index(2, 2), w.lot.tile_index(5, 4));
    w.dirt[worst] = 0.9;
    w.dirt[other] = 0.6;
    for s in &mut w.sims {
        s.needs[0] = 1.0;
    }
    // Bo is a step away first; Ada sets off for the worst of it before he's free.
    w.apply(Command::MoveTo { sim: 1, x: 10.5, z: 8.5 }).unwrap();
    let cleaning = |w: &World, i: usize| match w.sims[i].current().map(|a| a.task.kind) {
        Some(sim_core::world::TaskKind::Clean { x, z }) => Some((x, z)),
        _ => None,
    };
    for _ in 0..60 * 20 {
        w.tick_once();
        let (a, b) = (cleaning(&w, 0), cleaning(&w, 1));
        assert!(a.is_none() || a != b, "both went for the mess at {a:?}");
    }
    assert!(w.dirt[worst] < 0.5 && w.dirt[other] < 0.1, "both tidied: {:.2} {:.2}", w.dirt[worst], w.dirt[other]);
}
