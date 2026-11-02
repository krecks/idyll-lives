//! Build mode's Garage tool (`World::build_garage`, `World::sell_garage`): a garage on the home lot,
//! paid for and sold back, undone and made again, refused where it can't go, kept in a save, and
//! the household's car parking in front of one built at the end of its driveway.

mod common;

use serde_json::Value;
use sim_core::cars::CarState;
use sim_core::{Command, World};

/// A home on a street running along x (its middle at z = 25.5), with nothing built on it yet, a
/// driveway from the street to (24, 15.55) and the car parked on it facing the street. Two
/// grown-ups with jobs.
const TOWN: &str = r#"{"width":40,"depth":30,"exits":[[0.5,25.5],[39.5,25.5]],
    "plots":[{"name":"Home","x":0,"z":0,"w":30,"d":22,"entry":[15.5,20.5]}],
    "households":[{"name":"Ones","plot":0,"player":true,"funds":10000}],
    "sims":[{"name":"Ada","household":0,"age":30,"traits":["cheerful"],"x":15.5,"z":18.5,"job":{"career":"journalism.print","level":0}},
            {"name":"Bo","household":0,"age":32,"traits":["kind"],"x":16.5,"z":18.5,"job":{"career":"journalism.print","level":0}}],
    "drives":[{"plot":0,"x":24.0,"z":15.55,"front":1.0,"street":25.5}],
    "cars":[{"plot":0,"x":24.0,"z":18.2,"yaw":0.0,"street":25.5,"len":4.4,"wid":1.8,"body":1,"paint":2}]}"#;

/// A single garage (3.14 × 5.4 m between its walls' centre lines) on the line of the driveway,
/// 2 m back from its end, its door towards the street.
const BEHIND: [f32; 4] = [22.43, 8.08, 25.57, 13.48];

fn world() -> (World, String) {
    let content = common::content();
    let w = World::from_json(&content, TOWN, 1).expect("the town loads");
    (w, content)
}

fn build(w: &mut World, [x0, z0, x1, z1]: [f32; 4], rot: u8) -> Result<(), String> {
    w.apply(Command::BuildGarage {
        household: 0,
        x0,
        z0,
        x1,
        z1,
        rot,
    })
    .map_err(|e| e.to_string())
}

fn sell(w: &mut World) -> Result<(), String> {
    w.apply(Command::SellGarage { household: 0 })
        .map_err(|e| e.to_string())
}

fn blocked(w: &World, x: i32, z: i32) -> bool {
    w.blocked()[w.lot.tile_index(x, z)]
}

fn funds(w: &World) -> i64 {
    w.households[0].funds
}

/// A garage `dx`, `dz` m from `BEHIND`.
fn moved([x0, z0, x1, z1]: [f32; 4], dx: f32, dz: f32) -> [f32; 4] {
    [x0 + dx, z0 + dz, x1 + dx, z1 + dz]
}

#[test]
fn a_garage_is_bought_takes_up_its_ground_and_is_sold_back() {
    let (mut w, _) = world();
    let price = w.content.build.garage;
    assert!(price > 0, "a garage has a price in the content");
    let before = funds(&w);
    assert!(!blocked(&w, 23, 10));
    build(&mut w, BEHIND, 0).expect("a garage behind the driveway");
    assert_eq!(funds(&w), before - price);
    assert_eq!(w.garages.len(), 1);
    assert_eq!(w.garages[0].rot, Some(0));
    for (x, z) in [(22, 8), (25, 12), (23, 10)] {
        assert!(blocked(&w, x, z), "({x}, {z}) is the garage's");
    }
    // Nothing is put in it.
    assert!(
        w.apply(Command::Buy {
            household: 0,
            object: "plant".into(),
            at: Some([23, 10, 0]),
            colors: Default::default(),
            turn: None,
            offset: None,
            size: None,
            tall: None,
        })
        .is_err(),
        "no plant in the garage"
    );
    // The view shows it, its door's way with it.
    let view: Value = serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    assert_eq!(view["garages"][0]["rot"], 0);
    let shown = w.structure_version();

    sell(&mut w).expect("sold");
    let resale = (price as f32 * w.content.object_rules.resale).round() as i64;
    assert_eq!(
        funds(&w),
        before - price + resale,
        "sold back for its resale"
    );
    assert!(w.garages.is_empty());
    assert!(!blocked(&w, 23, 10), "its ground is free again");
    assert_ne!(w.structure_version(), shown, "the view hears of it");
    assert_eq!(
        sell(&mut w).unwrap_err(),
        "there's no garage at your home",
        "nothing left to sell"
    );
}

#[test]
fn no_wall_or_fence_goes_through_a_garage() {
    use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind};
    let (mut w, _) = world();
    build(&mut w, BEHIND, 0).unwrap();
    let wall = |w: &mut World, axis, x, z, kind| {
        w.apply(Command::Build {
            household: 0,
            edits: vec![EdgeEdit {
                style: (kind == EdgeKind::Fence).then_some(0),
                ..EdgeEdit::new(axis, x, z, kind)
            }],
        })
        .map_err(|e| e.to_string())
    };
    assert_eq!(
        wall(&mut w, EdgeAxis::H, 23, 10, EdgeKind::Wall).unwrap_err(),
        "a wall can't go through the garage"
    );
    assert_eq!(
        wall(&mut w, EdgeAxis::V, 25, 9, EdgeKind::Fence).unwrap_err(),
        "a fence can't go through the garage"
    );
    // Along its side a little way off, and once it's gone, they do.
    wall(&mut w, EdgeAxis::V, 22, 9, EdgeKind::Fence).expect("beside it");
    sell(&mut w).unwrap();
    wall(&mut w, EdgeAxis::H, 23, 10, EdgeKind::Wall).expect("where it stood");
}

#[test]
fn creative_garages_are_free() {
    let content = common::content();
    let town = TOWN.replacen('{', r#"{"mode":"creative","#, 1);
    let mut w = World::from_json(&content, &town, 1).unwrap();
    let before = funds(&w);
    build(&mut w, BEHIND, 0).unwrap();
    sell(&mut w).unwrap();
    assert_eq!(funds(&w), before);
    // (Free, yet an edit to take back.)
    assert!(w.undo_steps(0) == 2);
}

#[test]
fn building_and_selling_a_garage_can_be_undone_and_made_again() {
    let (mut w, _) = world();
    let before = funds(&w);
    let spot = w.cars[0].spot;
    build(&mut w, BEHIND, 0).unwrap();
    let built = funds(&w);
    let parked = w.cars[0].spot;
    assert_ne!(parked, spot, "the car parks by it");

    w.apply(Command::Undo { household: 0 }).unwrap();
    assert!(w.garages.is_empty());
    assert_eq!(funds(&w), before);
    assert!(!blocked(&w, 23, 10));
    assert_eq!(w.cars[0].spot, spot, "the car is back where it was");
    assert_eq!(w.cars[0].drive_spot, None);

    w.apply(Command::Redo { household: 0 }).unwrap();
    assert_eq!(w.garages.len(), 1);
    assert_eq!(funds(&w), built);
    assert!(blocked(&w, 23, 10));
    assert_eq!(w.cars[0].spot, parked);

    sell(&mut w).unwrap();
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!(w.garages.len(), 1, "the sale undone");
    assert_eq!(funds(&w), built);
    assert!(blocked(&w, 23, 10));
}

#[test]
fn a_garage_goes_only_where_it_fits() {
    let (mut w, _) = world();
    let before = funds(&w);
    let refused = |w: &mut World, b: [f32; 4], rot: u8| build(w, b, rot).unwrap_err();
    assert_eq!(
        refused(&mut w, moved(BEHIND, -23.0, 0.0), 0),
        "a garage has to go on your own lot",
        "over the plot's edge"
    );
    assert_eq!(
        refused(&mut w, [22.43, 8.08, 28.0, 13.48], 0),
        "a garage is 3 to 4.4 m wide and 5 to 6.6 m deep"
    );
    assert_eq!(
        refused(&mut w, BEHIND, 4),
        "a garage faces one of four ways, not 4"
    );
    assert_eq!(
        refused(&mut w, moved(BEHIND, 0.0, 4.0), 0),
        "that's where the car parks"
    );
    // A thing there, a fence across it, a room.
    common::buy(&mut w, "plant", 4, 4);
    assert_eq!(
        refused(&mut w, [2.43, 1.08, 5.57, 6.48], 0),
        "the potted plant is in the way"
    );
    w.apply(Command::Build {
        household: 0,
        edits: vec![sim_core::command::EdgeEdit {
            style: Some(0),
            ..sim_core::command::EdgeEdit::new(
                sim_core::command::EdgeAxis::V,
                12,
                3,
                sim_core::command::EdgeKind::Fence,
            )
        }],
    })
    .unwrap();
    assert_eq!(
        refused(&mut w, [10.43, 1.08, 13.57, 6.48], 0),
        "there's a wall or a fence in the way"
    );
    // Someone standing there.
    let ada = w.sims[0].tile();
    let at = [
        ada.0 as f32 - 1.57,
        ada.1 as f32 - 2.2,
        ada.0 as f32 + 1.57,
        ada.1 as f32 + 3.2,
    ];
    assert_eq!(refused(&mut w, at, 0), "someone is standing there");
    // Not enough money.
    w.apply(Command::AddFunds {
        household: 0,
        amount: -funds(&w),
    })
    .unwrap();
    assert_eq!(refused(&mut w, BEHIND, 0), "not enough money for a garage");
    w.apply(Command::AddFunds {
        household: 0,
        amount: before,
    })
    .unwrap();
    // One a home.
    build(&mut w, BEHIND, 0).unwrap();
    assert_eq!(
        refused(&mut w, moved(BEHIND, -8.0, 0.0), 0),
        "your home has a garage already"
    );
    assert_eq!(w.garages.len(), 1, "the refused ones left nothing behind");
}

#[test]
fn a_garage_never_shuts_anything_in() {
    let (mut w, _) = world();
    // A pen of fences in the plot's corner (x 0..5, z 0..10), open only at tiles (3, 9) and
    // (4, 9) on its side towards the street, a bookshelf in it.
    let fence = |axis, x, z| sim_core::command::EdgeEdit {
        style: Some(0),
        ..sim_core::command::EdgeEdit::new(axis, x, z, sim_core::command::EdgeKind::Fence)
    };
    let mut edits: Vec<_> = (0..10)
        .map(|z| fence(sim_core::command::EdgeAxis::V, 5, z))
        .collect();
    edits.extend((0..3).map(|x| fence(sim_core::command::EdgeAxis::H, x, 10)));
    w.apply(Command::Build {
        household: 0,
        edits,
    })
    .unwrap();
    common::buy(&mut w, "bookshelf", 1, 2);
    let before = funds(&w);
    // In front of the way out of the pen.
    let shut = [3.43, 10.3, 6.57, 15.7];
    assert_eq!(
        build(&mut w, shut, 0).unwrap_err(),
        "that would block the way"
    );
    assert!(w.garages.is_empty());
    assert!(!blocked(&w, 4, 11), "its tiles are free again");
    assert_eq!(funds(&w), before);
    // A little further along, the way stays open.
    build(&mut w, moved(shut, 3.0, 0.0), 0).expect("clear of the way out");
}

#[test]
fn the_car_parks_in_front_of_a_garage_at_the_end_of_its_driveway() {
    let (mut w, _) = world();
    let (spot, yaw) = (w.cars[0].spot, w.cars[0].yaw);
    // One with its door away from the street changes nothing.
    build(&mut w, BEHIND, 2).unwrap();
    assert_eq!(w.cars[0].spot, spot);
    assert_eq!(w.cars[0].drive_spot, None);
    sell(&mut w).unwrap();
    // Nor one off to the side of the driveway.
    build(&mut w, moved(BEHIND, -6.0, 0.0), 0).unwrap();
    assert_eq!(w.cars[0].spot, spot);
    sell(&mut w).unwrap();

    build(&mut w, BEHIND, 0).unwrap();
    let car = &w.cars[0];
    // In front of its door (on the outer face of its wall at z 13.55), nose in.
    let reach = 0.45 + car.len / 2.0;
    assert!((car.spot[0] - 24.0).abs() < 1e-4);
    assert!(
        (car.spot[1] - (13.55 + reach)).abs() < 1e-3,
        "parked at {:?}",
        car.spot
    );
    assert!(
        (car.yaw.abs() - std::f32::consts::PI).abs() < 1e-4,
        "nose in"
    );
    assert_eq!(car.drive_spot, Some([spot[0], spot[1], yaw]));
    assert!(!car.blocked);
    assert!(blocked(&w, 24, 15), "its new spot is taken up");
    assert!(!blocked(&w, 24, 19), "and the old one freed");

    sell(&mut w).unwrap();
    assert_eq!(
        (w.cars[0].spot, w.cars[0].yaw),
        (spot, yaw),
        "back on its spot"
    );
    assert_eq!(w.cars[0].drive_spot, None);
    assert!(blocked(&w, 24, 19));
}

#[test]
fn a_car_away_comes_home_to_the_new_garage() {
    let (mut w, _) = world();
    // Off to work by car.
    assert!(
        common::until(&mut w, 3 * 24 * 60, |w| w.cars[0].state == CarState::Away),
        "the car never left"
    );
    build(&mut w, BEHIND, 0).unwrap();
    let parked = w.cars[0].spot;
    assert!((parked[1] - (13.55 + 0.45 + w.cars[0].len / 2.0)).abs() < 1e-3);
    assert!(
        common::until(&mut w, 24 * 60, |w| w.cars[0].state == CarState::Parked),
        "it never came home"
    );
    let car = &w.cars[0];
    assert_eq!(car.spot, parked, "home to the garage");
    assert!(
        (car.yaw.abs() - std::f32::consts::PI).abs() < 0.05,
        "nose in"
    );
    assert!(!car.blocked);
}

#[test]
fn garages_survive_a_save_and_old_files_stay_as_they_were() {
    let (mut w, content) = world();
    build(&mut w, BEHIND, 0).unwrap();
    let json = w.save_json();
    let back = World::from_save_json(&content, &json).unwrap();
    assert_eq!(back.garages, w.garages);
    assert_eq!(back.cars[0].spot, w.cars[0].spot);
    assert_eq!(back.cars[0].drive_spot, w.cars[0].drive_spot);
    assert!(blocked(&back, 23, 10));
    assert!(blocked(&back, 24, 15));
    // And sold after loading, the car goes back.
    let mut back = back;
    sell(&mut back).unwrap();
    assert_eq!(back.cars[0].spot, [24.0, 18.2]);

    // A town's garage (no facing) saves as it came, and its car parks where the town put it.
    let town = TOWN.replace(
        r#""cars":"#,
        r#""garages":[{"plot":0,"x0":22.43,"z0":10.08,"x1":25.57,"z1":15.48}],"cars":"#,
    );
    let w = World::from_json(&content, &town, 1).unwrap();
    assert_eq!(w.garages[0].rot, None);
    assert_eq!(w.cars[0].spot, [24.0, 18.2]);
    let saved: Value = serde_json::from_str(&w.save_json()).unwrap();
    assert!(saved["garages"][0].get("rot").is_none());
    assert!(saved["cars"][0].get("driveSpot").is_none());
}
