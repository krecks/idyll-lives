//! A town file's open-world fields are read, kept on the world
//! (`World::travel`, `travel::OpenWorld`) and saved with the game: size, roads, areas, typed
//! exits, terrain masks, parking, plots' areas and kinds, driveways' roads. A town without them
//! takes its meta's roads as the network (every town is open now: cosy and normal were dropped in
//! 0.8.0).

use crate::common;

use serde_json::json;
use sim_core::World;
use sim_core::travel::{AreaKind, PlotKind, RoadKind, Terrain};

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
    town["parking"] = json!([{"x": 10.0, "z": 36.5, "rot": 0, "area": 0}]);
    town["plots"][0]["area"] = json!(0);
    town["plots"][0]["kind"] = json!("home");
    let mut world = World::from_json(&content, &town.to_string(), 5).unwrap();
    let t = &world.travel;
    let f = &t.file;
    assert_eq!(f.roads.len(), 2);
    assert_eq!(f.roads[0].kind, RoadKind::Street);
    assert_eq!(f.roads[1].kind, RoadKind::Trail);
    assert_eq!(f.roads[1].area, Some(0));
    assert_eq!(t.graph.roads.len(), 2);
    assert_eq!(f.areas[0].kind, AreaKind::Hamlet);
    // (A bus stop, in a town file from when the town had buses, is no way out: left out.)
    assert_eq!(world.exits, vec![[0.5, 30.0]]);
    assert_eq!(f.parking.len(), 1);
    assert_eq!(f.plot_areas[0], Some(0));
    assert_eq!(f.plot_kinds[0], Some(PlotKind::Home));
    assert_eq!(f.plot_kinds[1], None);
    let water = Terrain::mask(f.terrain.as_ref().unwrap(), w as usize, d as usize);
    let wet: Vec<usize> = (0..water.len()).filter(|&i| water[i]).collect();
    let row = 2 * w as usize;
    assert_eq!(wet, vec![row, row + 1, row + 2]);
    // Saved and loaded back as they were.
    world.tick_once();
    let back = World::from_save_json(&content, &world.save_json()).unwrap();
    let json = |w: &World| serde_json::to_value(&w.travel.file).unwrap();
    assert_eq!(json(&back), json(&world));
    assert_eq!(back.exits, world.exits);
    assert_eq!(back.travel.graph.roads.len(), 2);
}

#[test]
fn a_town_without_them_is_big_and_uses_its_metas_roads() {
    let content = common::content();
    let mut town: serde_json::Value = serde_json::from_str(&common::town(&content, 5, 2)).unwrap();
    town["meta"] = json!({"name": "Elmbury", "streets": [{"x": 0, "z": 30, "w": 80, "d": 6}],
        "roads": [{"points": [[0, 33], [80, 33]], "width": 6}]});
    let world = World::from_json(&content, &town.to_string(), 5).unwrap();
    let t = &world.travel;
    assert_eq!(t.graph.roads.len(), 1, "the meta's road is the network");
    assert!(t.file.is_empty(), "no open-world fields of its own");
    // Saved without them (its network made again from its meta), and still open after a load.
    let json: serde_json::Value = serde_json::from_str(&world.save_json()).unwrap();
    assert!(json.get("openWorld").is_none());
    let back = World::from_save_json(&content, &world.save_json()).unwrap();
    assert_eq!(back.travel.graph.roads.len(), 1);
}
