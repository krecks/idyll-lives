//! A town file's open-world fields are read, kept on the world
//! (`World::travel`, `travel::OpenWorld`) and saved with the game: size, roads, areas, typed
//! exits, terrain masks, parking bays and car parks, plots' areas and kinds, flats, driveways'
//! roads. A town without them
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
    town["parking"] = json!([{"x": 10.0, "z": 36.5, "yaw": 3.0, "area": 0}]);
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
    assert_eq!(f.parking[0].yaw, Some(3.0));
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

/// The real town's fixture (`tests/towns/flats.json`): a flats building of two storeys on a lot of
/// its own (plot 0), its four flats (plots 1 to 4: two a storey, the hall between them), two of
/// them with a bay in the front strip, and a car park of four bays down the street.
const FLATS: &str = include_str!("../towns/flats.json");

#[test]
fn flats_are_plots_of_their_own_per_storey() {
    let content = common::content();
    let mut w = World::from_json(&content, FLATS, 1).unwrap();
    assert_eq!(w.plots.len(), 5);
    assert_eq!(w.plots[0].kind, Some(PlotKind::Flats));
    assert!(!w.plots[0].is_flat());
    for (id, storey) in [(1, 0), (2, 0), (3, 1), (4, 1)] {
        let p = &w.plots[id];
        assert_eq!(
            (p.kind, p.of, p.storey),
            (Some(PlotKind::Flat), Some(0), storey)
        );
    }
    // Home is behind the flat's own door; arriving is at the building's street door.
    assert_eq!(w.plots[1].home_point(), [11.5, 15.5]);
    assert_eq!(w.plots[1].arrival_point(), [13.5, 17.5]);
    assert_eq!(w.plots[0].home_point(), w.plots[0].arrival_point());
    // Each tile's plot, on each storey: the flat there, else the building's lot.
    let up = w.storey_row(1);
    assert_eq!(up, 40);
    assert_eq!(w.plot_at(10, 12), Some(1));
    assert_eq!(w.plot_at(16, 12), Some(2));
    assert_eq!(w.plot_at(10, 12 + up), Some(3));
    assert_eq!(w.plot_at(16, 12 + up), Some(4));
    assert_eq!(w.plot_at(13, 12), Some(0), "the hall");
    assert_eq!(w.plot_at(13, 12 + up), Some(0), "the landing upstairs");
    assert_eq!(w.plot_at(10, 25), Some(0), "the front strip");
    assert_eq!(w.plot_at(30, 10), None);
    // The letterboxes by the street door are their flats'.
    let keepers: Vec<Option<u32>> = w
        .objects
        .iter()
        .filter(|o| w.content.objects[o.def].id == "letterbox")
        .map(|o| o.keeper)
        .collect();
    assert_eq!(keepers, [Some(1), Some(2), Some(3), Some(4)]);
    // An hour goes by with them in it.
    for _ in 0..sim_core::TICKS_PER_MINUTE * 60 {
        w.tick_once();
    }
    // Saved and loaded back as they were (the cached tiles made again).
    let back = World::from_save_json(&content, &w.save_json()).unwrap();
    assert_eq!(back.plots, w.plots);
    assert_eq!(back.plot_at(10, 12 + up), Some(3));
    let kept = |w: &World| w.objects.iter().map(|o| o.keeper).collect::<Vec<_>>();
    assert_eq!(kept(&back), kept(&w));
}

#[test]
fn bays_park_as_the_file_says_and_the_structure_sends_them() {
    let content = common::content();
    let w = World::from_json(&content, FLATS, 1).unwrap();
    let f = &w.travel.file;
    assert_eq!(f.parking.len(), 4);
    // (Nose in, away from the street: towards -z.)
    let away = |yaw: f64| (yaw - std::f64::consts::PI).abs() < 1e-4;
    assert!(
        f.parking
            .iter()
            .all(|p| p.yaw.is_some_and(|y| away(f64::from(y))) && p.park == Some(0))
    );
    assert_eq!(f.car_parks.len(), 1);
    let s: serde_json::Value = serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    // The ground lots only (the building), with its kind; the flats on their own.
    assert_eq!(s["plots"].as_array().unwrap().len(), 1);
    assert_eq!(s["plots"][0]["kind"], "flats");
    let flats = s["flats"].as_array().unwrap();
    assert_eq!(flats.len(), 4);
    assert_eq!(flats[2]["id"], 3);
    assert_eq!(flats[2]["of"], 0);
    assert_eq!(flats[2]["storey"], 1);
    assert_eq!(flats[2]["door"], json!([12, 55]));
    // Each bay as a car parks in it: the file's pose.
    let parking = s["parking"].as_array().unwrap();
    assert_eq!(parking.len(), 4);
    for (bay, x) in parking.iter().zip([27.5, 30.5, 33.5, 36.5]) {
        assert_eq!(bay["x"].as_f64(), Some(x));
        assert!(away(bay["yaw"].as_f64().unwrap()));
        assert_eq!(bay["park"], 0);
    }
    assert_eq!(s["carParks"][0]["sign"], json!([25.5, 30.5, 0.0]));
    // The first update of a view sends them; the next, nothing having changed, doesn't again.
    let mut sync = sim_core::view::StructureSync::default();
    let first: serde_json::Value = serde_json::from_str(&sync.update_json(&w, true)).unwrap();
    for key in ["flats", "parking", "carParks"] {
        assert_eq!(first[key], s[key], "{key}");
    }
    let next: serde_json::Value = serde_json::from_str(&sync.update_json(&w, false)).unwrap();
    for key in ["flats", "parking", "carParks"] {
        assert!(next.get(key).is_none(), "{key} again");
    }
}

#[test]
fn flats_out_of_place_are_refused() {
    let content = common::content();
    let town: serde_json::Value = serde_json::from_str(FLATS).unwrap();
    let refused = |change: &dyn Fn(&mut serde_json::Value)| {
        let mut t = town.clone();
        change(&mut t);
        World::from_json(&content, &t.to_string(), 1)
            .err()
            .map(|e| e.0)
    };
    let out = refused(&|t| t["plots"][1]["x"] = json!(0)).unwrap();
    assert!(out.contains("reaches out"), "{out}");
    let up = refused(&|t| t["plots"][3]["storey"] = json!(2)).unwrap();
    assert!(up.contains("no storey"), "{up}");
    let door = refused(&|t| t["plots"][3]["door"] = json!([12, 15])).unwrap();
    assert!(door.contains("door"), "{door}");
    let after = refused(&|t| {
        let lot = t["plots"][0].clone();
        t["plots"].as_array_mut().unwrap().push(lot);
    })
    .unwrap();
    assert!(after.contains("after the flats"), "{after}");
}
