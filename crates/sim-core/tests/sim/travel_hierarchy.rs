//! The road hierarchy (`travel::RoadKind`'s avenue, motorway and ramp, one-way roads): a small
//! world of every kind, `hierarchy()`, the same layout as the web's
//! `web/src/render/roadHierarchyFixture.ts` (road for road, in the same order):
//!
//! - a motorway edge to edge along z = 200: two one-way carriageways 9 m apart (eastbound, its
//!   points running east; westbound, its points running east too and one-way against them),
//!   raised over a street (`BRIDGE_STREET`, north from the avenue to the world's edge) without
//!   meeting it, and on a bridge over a river;
//! - a half-diamond west of the street: an off-ramp from the eastbound carriageway and an
//!   on-ramp onto the westbound one, each merging into its carriageway at one end and meeting the
//!   street at a T 25 m from the motorway at the other;
//! - an avenue along z = 80 with two cross streets;
//! - a one-way loop round a block, joined to the avenue's east end by a two-way street.
//!
//! Nothing lives there: it is roads, the river and the ways out.

use serde_json::{Value, json};
use sim_core::World;
use sim_core::travel::{OneWay, RoadKind, RoadRaw};

use crate::common;

/// The world's size (m).
pub const WIDTH: i64 = 480;
pub const DEPTH: i64 = 320;
/// The carriageways' middles (z): eastbound, westbound.
pub const EAST_Z: f64 = 200.0;
pub const WEST_Z: f64 = 209.0;
/// The carriageways' ends (x): `s` along either is `x - MOTORWAY_X0`.
pub const MOTORWAY_X0: f64 = 0.5;
pub const MOTORWAY_X1: f64 = 479.5;
/// The street under the motorway (x), from the avenue (z) to the world's edge.
pub const STREET_X: f64 = 140.0;
pub const AVENUE_Z: f64 = 80.0;
/// The river: water from `RIVER_X` (x) on, `RIVER_W` wide, from `RIVER_Z0` (z) to the edge.
pub const RIVER_X: i64 = 330;
pub const RIVER_W: i64 = 18;
pub const RIVER_Z0: i64 = 130;

/// The roads, by index (`hierarchy().roads`, the town file's `roads` and `meta.roads`).
pub const MOTORWAY_EAST: usize = 0;
pub const MOTORWAY_WEST: usize = 1;
pub const BRIDGE_STREET: usize = 2;
pub const OFF_RAMP: usize = 3;
pub const ON_RAMP: usize = 4;
pub const AVENUE: usize = 5;
pub const CROSS_WEST: usize = 6;
pub const CROSS_EAST: usize = 7;
pub const LOOP_LINK: usize = 8;
/// The one-way loop's four sides: west (cars go south), south (east), east (north) and north
/// (west, its points running east).
pub const LOOP: [usize; 4] = [9, 10, 11, 12];

/// What `hierarchy` built: the town file, and its roads (`RoadPlan`s) as written there.
pub struct Hierarchy {
    pub town: String,
    pub roads: Vec<Value>,
}

/// A line through `pts` with a point about every metre (each stretch `common::hamlets::line`'s),
/// to the millimetre (so it reads back from a save as written).
fn polyline(pts: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mm = |v: f64| (v * 1000.0).round() / 1000.0;
    let mut out = vec![pts[0]];
    for w in pts.windows(2) {
        out.extend(common::hamlets::line(w[0], w[1]).into_iter().skip(1));
    }
    out.into_iter().map(|p| [mm(p[0]), mm(p[1])]).collect()
}

/// The water's tiles as a terrain mask (`Terrain::water`: runs of `[length, set]`, row by row).
fn river_mask() -> Vec<u32> {
    let mut rle: Vec<u32> = Vec::new();
    let mut push = |run: i64, set: u32| {
        if run <= 0 {
            return;
        }
        if rle.len() >= 2 && rle[rle.len() - 1] == set {
            let n = rle.len() - 2;
            rle[n] += run as u32;
        } else {
            rle.extend([run as u32, set]);
        }
    };
    for z in 0..DEPTH {
        if z < RIVER_Z0 {
            push(WIDTH, 0);
        } else {
            push(RIVER_X, 0);
            push(RIVER_W, 1);
            push(WIDTH - RIVER_X - RIVER_W, 0);
        }
    }
    rle
}

/// The fixture's town file (see the top of this file).
pub fn hierarchy() -> Hierarchy {
    let s = |x: f64| x - MOTORWAY_X0;
    let street_s = |z: f64| z - AVENUE_Z;
    let motorway = |z: f64, oneway: i8, name: &str| {
        json!({
            "points": polyline(&[[MOTORWAY_X0, z], [MOTORWAY_X1, z]]),
            "width": 9, "ends": ["out", "out"], "kind": "motorway", "oneway": oneway,
            "name": name,
            // (Over the street, a 30 m deck; over the river, 4 m past its banks.)
            "raised": [[s(STREET_X - 15.0), s(STREET_X + 15.0)]],
            "bridge": [s(RIVER_X as f64 - 4.0), s((RIVER_X + RIVER_W) as f64 + 4.0)],
            "crosses": [{"road": BRIDGE_STREET, "s": s(STREET_X)}],
        })
    };
    let street = |pts: &[[f64; 2]], ends: [&str; 2], name: &str| json!({"points": polyline(pts), "width": 7, "ends": ends, "kind": "street", "name": name});
    let one_way = |pts: &[[f64; 2]], oneway: i8, name: &str| json!({"points": polyline(pts), "width": 7, "ends": ["join", "join"], "kind": "street", "oneway": oneway, "name": name});
    let ramp = |pts: &[[f64; 2]], ends: [&str; 2], name: &str| json!({"points": polyline(pts), "width": 5, "ends": ends, "kind": "ramp", "oneway": 1, "name": name});
    let mut bridge_street = street(
        &[[STREET_X, AVENUE_Z], [STREET_X, DEPTH as f64 - 0.5]],
        ["join", "out"],
        "Bridge Street",
    );
    bridge_street["crosses"] = json!([
        {"road": MOTORWAY_EAST, "s": street_s(EAST_Z)},
        {"road": MOTORWAY_WEST, "s": street_s(WEST_Z)},
    ]);
    let (x0, x1, y0, y1) = (300.0, 360.0, 20.0, 60.0);
    let roads = vec![
        motorway(EAST_Z, 1, "Motorway East"),
        motorway(WEST_Z, -1, "Motorway West"),
        bridge_street,
        // (Off the eastbound carriageway's middle, out beside it, and up to the street 25 m short.)
        ramp(
            &[
                [20.0, EAST_Z],
                [40.0, EAST_Z - 7.0],
                [110.0, EAST_Z - 7.0],
                [STREET_X, EAST_Z - 25.0],
            ],
            ["merge", "join"],
            "Off-ramp",
        ),
        ramp(
            &[
                [STREET_X, WEST_Z + 25.0],
                [110.0, WEST_Z + 7.0],
                [40.0, WEST_Z + 7.0],
                [20.0, WEST_Z],
            ],
            ["join", "merge"],
            "On-ramp",
        ),
        json!({
            "points": polyline(&[[0.5, AVENUE_Z], [x0, AVENUE_Z]]),
            "width": 14, "ends": ["out", "join"], "kind": "avenue", "name": "The Avenue",
        }),
        street(
            &[[90.0, 40.0], [90.0, 120.0]],
            ["dead", "dead"],
            "West Cross",
        ),
        street(
            &[[210.0, 40.0], [210.0, 120.0]],
            ["dead", "dead"],
            "East Cross",
        ),
        street(&[[x0, AVENUE_Z], [x0, y1]], ["join", "join"], "Loop Link"),
        one_way(&[[x0, y1], [x0, y0]], 1, "Loop West"),
        one_way(&[[x0, y0], [x1, y0]], 1, "Loop South"),
        one_way(&[[x1, y0], [x1, y1]], 1, "Loop East"),
        one_way(&[[x0, y1], [x1, y1]], -1, "Loop North"),
    ];
    let town = json!({
        "width": WIDTH, "depth": DEPTH, "size": "big",
        "walls": [], "doors": [], "windows": [], "objects": [],
        "plots": [], "households": [], "sims": [], "relationships": [],
        "roads": roads,
        "meta": {"roads": roads},
        "exits": [
            [MOTORWAY_X0, EAST_Z], [MOTORWAY_X1, EAST_Z],
            [MOTORWAY_X0, WEST_Z], [MOTORWAY_X1, WEST_Z],
            [0.5, AVENUE_Z], [STREET_X, DEPTH as f64 - 0.5],
        ],
        "terrain": {"water": river_mask()},
    });
    Hierarchy {
        town: town.to_string(),
        roads,
    }
}

fn world() -> World {
    let content = common::content();
    World::from_json(&content, &hierarchy().town, 1).expect("the hierarchy loads")
}

/// Each road's kind and way, as the fixture lays them.
fn expected() -> Vec<(RoadKind, OneWay)> {
    let mut e = vec![(RoadKind::Street, OneWay::Both); LOOP[3] + 1];
    e[MOTORWAY_EAST] = (RoadKind::Motorway, OneWay::Along);
    e[MOTORWAY_WEST] = (RoadKind::Motorway, OneWay::Against);
    e[OFF_RAMP] = (RoadKind::Ramp, OneWay::Along);
    e[ON_RAMP] = (RoadKind::Ramp, OneWay::Along);
    e[AVENUE] = (RoadKind::Avenue, OneWay::Both);
    for side in LOOP {
        e[side] = (RoadKind::Street, OneWay::Along);
    }
    e[LOOP[3]] = (RoadKind::Street, OneWay::Against);
    for two_way in [BRIDGE_STREET, CROSS_WEST, CROSS_EAST, LOOP_LINK] {
        assert_eq!(e[two_way], (RoadKind::Street, OneWay::Both));
    }
    e
}

#[test]
fn the_hierarchy_loads_with_its_kinds_and_ways() {
    let w = world();
    let kinds = |roads: &[RoadRaw]| roads.iter().map(|r| (r.kind, r.oneway)).collect::<Vec<_>>();
    assert_eq!(kinds(&w.travel.file.roads), expected());
    let g = &w.travel.graph;
    assert_eq!(g.roads.len(), expected().len());
    for r in &g.roads {
        assert_eq!(
            (r.kind, r.oneway),
            expected()[r.source as usize],
            "road {}",
            r.source
        );
        let car_only = matches!(r.kind, RoadKind::Motorway | RoadKind::Ramp);
        assert_eq!(r.walkable(), !car_only, "road {}", r.source);
        assert!(r.drivable(), "road {}", r.source);
        assert_eq!(r.enterable(), !car_only, "road {}", r.source);
    }
    // (The carriageways 9 m apart, the ramps 110 to 140 m long.)
    let mid = |i: usize| g.roads.iter().find(|r| r.source == i as u32).unwrap();
    let apart = mid(MOTORWAY_WEST).at(100.0)[1] - mid(MOTORWAY_EAST).at(100.0)[1];
    assert!((apart - 9.0).abs() < 1e-3, "{apart} m apart");
    for ramp in [OFF_RAMP, ON_RAMP] {
        let len = mid(ramp).len();
        assert!((110.0..=140.0).contains(&len), "ramp {ramp}: {len} m");
    }
}

#[test]
fn the_hierarchy_saves_and_loads_back_as_it_was() {
    let content = common::content();
    let mut w = world();
    w.tick_once();
    let saved = w.save_json();
    let back = World::from_save_json(&content, &saved).expect("the save loads");
    let file = |w: &World| serde_json::to_value(&w.travel.file).unwrap();
    assert_eq!(file(&back), file(&w));
    let kinds = |w: &World| {
        w.travel
            .graph
            .roads
            .iter()
            .map(|r| (r.source, r.kind, r.oneway))
            .collect::<Vec<_>>()
    };
    assert_eq!(kinds(&back), kinds(&w));
    // (Saved as the web writes it: the kinds by name, the ways as 1 and -1.)
    let roads = &file(&w)["roads"];
    assert_eq!(roads[MOTORWAY_EAST]["kind"], "motorway");
    assert_eq!(roads[MOTORWAY_EAST]["oneway"], 1);
    assert_eq!(roads[MOTORWAY_WEST]["oneway"], -1);
    assert_eq!(roads[OFF_RAMP]["kind"], "ramp");
    assert_eq!(roads[AVENUE]["kind"], "avenue");
    assert!(roads[AVENUE].get("oneway").is_none(), "{}", roads[AVENUE]);
    // (The meta's roads, kept whole: the ends, the overpass and the crossings.)
    let meta: Value = serde_json::from_str(&saved).unwrap();
    let meta_roads = &meta["meta"]["roads"];
    assert_eq!(meta_roads, &Value::Array(hierarchy().roads));
}

#[test]
fn a_two_way_road_saves_without_a_way() {
    let raw: RoadRaw =
        serde_json::from_str(r#"{"points": [[0, 0], [10, 0]], "width": 7, "kind": "lane"}"#)
            .unwrap();
    assert_eq!(raw.oneway, OneWay::Both);
    let json = serde_json::to_string(&raw).unwrap();
    assert!(!json.contains("oneway"), "{json}");
    for bad in ["2", "true", "\"along\""] {
        let r = serde_json::from_str::<RoadRaw>(&format!(
            r#"{{"points": [[0, 0], [10, 0]], "width": 7, "oneway": {bad}}}"#
        ));
        assert!(r.is_err(), "oneway {bad} read as {:?}", r.map(|r| r.oneway));
    }
}

// What the hierarchy's streams test on this fixture (stage 1 and 2 of the plan).

#[test]
#[ignore = "S-GRAPH: no junction where the motorway crosses a road"]
fn motorway_crossing_makes_no_node() {}

#[test]
#[ignore = "S-GRAPH: one-way edges"]
fn oneway_loop_paths_differ() {}

#[test]
#[ignore = "S-SPEED: kind_speed in the costs and the lanes' tops"]
fn motorway_is_faster() {}

#[test]
#[ignore = "S-TRAFFIC: cars on the deck and under it don't interact"]
fn nobody_stops_under_the_bridge() {}

#[test]
#[ignore = "S-TRAFFIC: on-ramp merges wait for a gap"]
fn merges_settle() {}
