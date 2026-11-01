//! A town file's open-world fields (docs/design/open-world.md) are read, kept on the world
//! (`World::town`) and saved with the game: size, roads, areas, typed exits, terrain masks,
//! parking, plots' areas and kinds, driveways' roads. A town without them gets them made up from
//! what it has.

mod common;

use serde_json::json;
use sim_core::World;
use sim_core::lot::{AreaKind, ExitKind, PlotKind, RoadKind, Terrain, WorldSize};

#[test]
fn open_world_fields_are_read_kept_and_saved() {
    let content = common::content();
    let mut town: serde_json::Value = serde_json::from_str(&common::town(&content, 5, 2)).unwrap();
    let (w, d) = (
        town["width"].as_u64().unwrap(),
        town["depth"].as_u64().unwrap(),
    );
    town["size"] = json!("normal");
    town["roads"] = json!([
        {"points": [[0, 30], [50, 30]], "width": 6, "ends": ["out", "join"], "name": "High St"},
        {"points": [[50, 30], [50, 60]], "width": 2, "kind": "trail", "area": 0}
    ]);
    town["areas"] = json!([{"name": "Main", "kind": "hamlet", "x": 0, "z": 0, "w": w, "d": d}]);
    town["exits"] = json!([[0.5, 30.0], {"x": 20.0, "z": 31.0, "kind": "stop"}]);
    town["terrain"] = json!({"water": [w * 2, 0, 3, 1]});
    town["parking"] = json!([{"x": 10.0, "z": 32.0, "rot": 1.5, "area": 0}]);
    town["plots"][0]["area"] = json!(0);
    town["plots"][0]["kind"] = json!("home");
    let mut world = World::from_json(&content, &town.to_string(), 5).unwrap();
    let t = &world.town;
    assert_eq!(t.size, WorldSize::Normal);
    assert_eq!(t.roads.len(), 2);
    assert_eq!(t.roads[0].kind, RoadKind::Street);
    assert_eq!(t.roads[1].kind, RoadKind::Trail);
    assert_eq!(t.roads[1].area, Some(0));
    assert_eq!(t.areas[0].kind, AreaKind::Hamlet);
    assert_eq!(t.exits[0].kind, ExitKind::Road);
    assert_eq!(t.exits[1].kind, ExitKind::Stop);
    assert_eq!(world.exits, vec![[0.5, 30.0], [20.0, 31.0]]);
    assert_eq!(t.parking.len(), 1);
    assert_eq!(t.plot_areas[0], Some(0));
    assert_eq!(t.plot_kinds[0], Some(PlotKind::Home));
    assert_eq!(t.plot_kinds[1], None);
    let water = Terrain::mask(t.terrain.water.as_deref(), w as usize, d as usize).unwrap();
    let wet: Vec<usize> = (0..water.len()).filter(|&i| water[i]).collect();
    let row = 2 * w as usize;
    assert_eq!(wet, vec![row, row + 1, row + 2]);
    // Saved and loaded back as they were.
    world.tick_once();
    let back = World::from_save_json(&content, &world.save_json()).unwrap();
    assert_eq!(back.town, world.town);
}

#[test]
fn a_town_without_them_gets_them_made_up() {
    let content = common::content();
    let mut town: serde_json::Value = serde_json::from_str(&common::town(&content, 5, 2)).unwrap();
    town["meta"] = json!({"name": "Elmbury", "streets": [{"x": 0, "z": 30, "w": 80, "d": 6}]});
    let world = World::from_json(&content, &town.to_string(), 5).unwrap();
    let t = &world.town;
    assert_eq!(t.size, WorldSize::Cosy);
    assert_eq!(t.areas.len(), 1);
    assert_eq!(t.areas[0].name, "Elmbury");
    assert_eq!(t.roads.len(), 1);
    assert_eq!(t.roads[0].points, vec![[0.0, 33.0], [80.0, 33.0]]);
    assert_eq!(t.roads[0].width, 6.0);
    assert!(t.exits.iter().all(|e| e.kind == ExitKind::Road));
    assert_eq!(t.exits.len(), world.exits.len());
}
