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
use sim_core::travel::{Edge, Graph, Mode, OneWay, RoadKind, RoadPath, RoadPoint, RoadRaw, dist};

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

/// The network road of the town file's road `source`.
fn road_of(g: &Graph, source: usize) -> u32 {
    g.roads
        .iter()
        .position(|r| r.source == source as u32)
        .expect("in the network") as u32
}

/// The place on road `source` nearest `p`.
fn on(g: &Graph, source: usize, p: [f32; 2]) -> RoadPoint {
    let (s, _) = g.roads[road_of(g, source) as usize].project(p);
    g.at_source(source as u32, s).expect("in the network")
}

/// The stretches of network road `road`, in order along it.
fn edges_of(g: &Graph, road: u32) -> Vec<Edge> {
    g.edges.iter().filter(|e| e.road == road).copied().collect()
}

/// The way by car from `a` to `b` (`Graph::path`), if any.
fn drive(g: &Graph, a: RoadPoint, b: RoadPoint) -> Option<RoadPath> {
    g.path(a, b, Mode::Car, 16.0)
}

#[test]
fn motorway_crossing_makes_no_node() {
    let w = world();
    let g = &w.travel.graph;
    // (Nothing where the street runs under either carriageway: no node, no box.)
    for z in [EAST_Z, WEST_Z] {
        let under = [STREET_X as f32, z as f32];
        for n in &g.nodes {
            assert!(dist(*n, under) > 20.0, "a node at {n:?} by {under:?}");
        }
        for (b, r) in &g.boxes {
            assert!(dist(*b, under) > *r + 10.0, "a box at {b:?} by {under:?}");
        }
    }
    // (Each carriageway split only where its ramp joins it.)
    for side in [MOTORWAY_EAST, MOTORWAY_WEST] {
        assert_eq!(edges_of(g, road_of(g, side)).len(), 2, "road {side}");
    }
    // (The street goes on under it, unbroken but for the ramps' T's.)
    let street = road_of(g, BRIDGE_STREET);
    let along: Vec<f32> = edges_of(g, street).iter().map(|e| e.s1).collect();
    let s = |z: f64| (z - AVENUE_Z) as f32;
    assert_eq!(along.len(), 3, "{along:?}");
    assert!((along[0] - s(EAST_Z - 25.0)).abs() < 0.5, "{along:?}");
    assert!((along[1] - s(WEST_Z + 25.0)).abs() < 0.5, "{along:?}");
    // (Walking or driving under it: straight along the street.)
    let (a, b) = (
        on(g, BRIDGE_STREET, [140.0, 150.0]),
        on(g, BRIDGE_STREET, [140.0, 260.0]),
    );
    for mode in [Mode::Foot, Mode::Car] {
        let path = g.path(a, b, mode, 16.0).expect("a way under it");
        assert!((path.len - 110.0).abs() < 0.5, "{mode:?}: {} m", path.len);
        assert!(path.pts.iter().all(|p| p.2 == RoadKind::Street), "{mode:?}");
    }
    // (Under its deck, off the street: nobody stands in a carriageway there.)
    for p in [[150.0, EAST_Z as f32], [170.0, WEST_Z as f32 + 1.0]] {
        assert!(!w.travel.on_carriageway(p, 0.0), "{p:?}");
    }
    assert!(w.travel.on_carriageway([140.0, EAST_Z as f32], 0.0));
}

#[test]
fn oneway_loop_paths_differ() {
    let w = world();
    let g = &w.travel.graph;
    let (west, east) = (LOOP[0], LOOP[2]);
    let a = on(g, west, [300.0, 40.0]);
    let b = on(g, east, [360.0, 40.0]);
    let there = drive(g, a, b).expect("a way round");
    let back = drive(g, b, a).expect("a way back");
    // (There by the south side, back by the north: each its way round.)
    let by = |p: &RoadPath, z: f32| p.pts.iter().any(|q| (q.0[1] - z).abs() < 0.01);
    assert!(by(&there, 20.0) && !by(&there, 60.0), "{:?}", there.pts);
    assert!(by(&back, 60.0) && !by(&back, 20.0), "{:?}", back.pts);
    // (Back along the west side, the way it goes: round the block; on foot, straight.)
    let (up, down) = (on(g, west, [300.0, 50.0]), on(g, west, [300.0, 30.0]));
    let ahead = drive(g, up, down).expect("ahead");
    assert!((ahead.len - 20.0).abs() < 0.5, "{} m", ahead.len);
    let round = drive(g, down, up).expect("round the block");
    assert!((round.len - 180.0).abs() < 0.5, "{} m", round.len);
    let walk = g.path(down, up, Mode::Foot, 16.0).expect("on foot");
    assert!((walk.len - 20.0).abs() < 0.5, "{} m", walk.len);
    // (Each stretch of the loop driven its way: z falling on the west side, x falling on the
    // north.)
    let side = |p: [f32; 2], x: f32| (p[0] - x).abs() < 0.01;
    for path in [&there, &back, &round] {
        for w in path.pts.windows(2) {
            let (p, q) = (w[0].0, w[1].0);
            if side(p, 300.0) && side(q, 300.0) && p[1] <= 60.0 && q[1] <= 60.0 {
                assert!(q[1] <= p[1], "north on the west side: {p:?} to {q:?}");
            }
            if (p[1] - 60.0).abs() < 0.01 && (q[1] - 60.0).abs() < 0.01 {
                assert!(q[0] <= p[0], "east on the north side: {p:?} to {q:?}");
            }
        }
    }
}

#[test]
fn a_merge_has_no_box_and_no_lights() {
    let w = world();
    let g = &w.travel.graph;
    let merges: Vec<[f32; 2]> = g.merge_nodes().map(|n| g.nodes[n]).collect();
    assert_eq!(merges, vec![[20.0, EAST_Z as f32], [20.0, WEST_Z as f32]]);
    let junctions: Vec<[f32; 2]> = g.junctions().collect();
    for m in &merges {
        assert!(!junctions.contains(m), "{m:?} a junction");
        for (b, r) in &g.boxes {
            assert!(dist(*b, *m) > *r + 5.0, "a box at {b:?} by {m:?}");
        }
        for s in w.travel.signals.iter().chain(&w.travel.give_way) {
            assert!(dist(s.at, *m) > 20.0, "a junction's signal at {:?}", s.at);
        }
    }
    // (The ramps' other ends are T's on the street, junctions as any.)
    for z in [EAST_Z - 25.0, WEST_Z + 25.0] {
        let t = [STREET_X as f32, z as f32];
        assert!(junctions.iter().any(|j| dist(*j, t) < 0.5), "{t:?}");
        assert!(g.boxes.iter().any(|(b, _)| dist(*b, t) < 0.5), "{t:?}");
    }
}

#[test]
fn walkers_keep_off_the_motorway_and_its_ramps() {
    let w = world();
    let g = &w.travel.graph;
    let car_only = |k: RoadKind| matches!(k, RoadKind::Motorway | RoadKind::Ramp);
    for p in [
        [20.0, EAST_Z as f32],
        [60.0, WEST_Z as f32 + 7.0],
        [100.0, EAST_Z as f32 - 7.0],
        [300.0, 204.0],
    ] {
        for rp in g.near(p, 60.0, 4, Mode::Foot) {
            assert!(!car_only(g.roads[rp.road as usize].kind), "{p:?}: {rp:?}");
        }
    }
    let starts = [[140.0, 300.0], [60.0, 80.0], [300.0, 40.0], [90.0, 110.0]];
    for &a in &starts {
        for &b in &starts {
            let (Some(a), Some(b)) = (
                g.nearest(a, 20.0, Mode::Foot),
                g.nearest(b, 20.0, Mode::Foot),
            ) else {
                panic!("no road on foot near {a:?} or {b:?}");
            };
            let path = g.path(a, b, Mode::Foot, 16.0).expect("a way on foot");
            assert!(path.pts.iter().all(|p| !car_only(p.2)), "{a:?} to {b:?}");
        }
    }
    // (Nor can they set off along one.)
    let ramp = on(g, OFF_RAMP, [80.0, EAST_Z as f32 - 7.0]);
    let street = on(g, BRIDGE_STREET, [140.0, 120.0]);
    assert!(g.path(ramp, street, Mode::Foot, 16.0).is_none());
    assert!(g.path(ramp, street, Mode::Car, 16.0).is_some());
}

#[test]
fn town_to_the_motorway_by_its_ramps_their_way() {
    let w = world();
    let g = &w.travel.graph;
    let town = on(g, AVENUE, [60.0, AVENUE_Z as f32]);
    let x0 = MOTORWAY_X0 as f32 + 1.0;
    let x1 = MOTORWAY_X1 as f32 - 1.0;
    // Out west: up the street, the on-ramp, and west on the westbound carriageway.
    let out = drive(g, town, on(g, MOTORWAY_WEST, [x0, WEST_Z as f32])).expect("a way out");
    let kinds: Vec<RoadKind> = out.pts.iter().map(|p| p.2).collect();
    let ramp_at = kinds
        .iter()
        .position(|&k| k == RoadKind::Ramp)
        .expect("by a ramp");
    let motorway_at = kinds
        .iter()
        .position(|&k| k == RoadKind::Motorway)
        .expect("onto the motorway");
    assert!(ramp_at < motorway_at, "{kinds:?}");
    assert!(
        kinds[motorway_at..]
            .iter()
            .all(|&k| k == RoadKind::Motorway)
    );
    let on_ramp = &out.pts[ramp_at..motorway_at];
    assert!(
        on_ramp.iter().all(|p| p.0[1] > WEST_Z as f32 - 0.01),
        "the on-ramp: {on_ramp:?}"
    );
    for w in out.pts[motorway_at..].windows(2) {
        assert!(w[1].0[0] <= w[0].0[0], "east on the westbound: {w:?}");
        assert!((w[1].0[1] - WEST_Z as f32).abs() < 0.01, "{w:?}");
    }
    // In from the west: the eastbound carriageway, off by the off-ramp and down the street.
    let back = drive(g, on(g, MOTORWAY_EAST, [x0, EAST_Z as f32]), town).expect("a way in");
    let kinds: Vec<RoadKind> = back.pts.iter().map(|p| p.2).collect();
    let ramp_at = kinds
        .iter()
        .position(|&k| k == RoadKind::Ramp)
        .expect("by a ramp");
    assert!(kinds[..ramp_at].iter().all(|&k| k == RoadKind::Motorway));
    assert!(
        back.pts[..ramp_at]
            .windows(2)
            .all(|w| w[1].0[0] >= w[0].0[0])
    );
    // Never against a carriageway's way: not onto the westbound where cars come onto it from the
    // east, nor the eastbound at all from town (no ramp onto it).
    assert!(drive(g, town, on(g, MOTORWAY_WEST, [x1, WEST_Z as f32])).is_none());
    assert!(drive(g, town, on(g, MOTORWAY_EAST, [x1, EAST_Z as f32])).is_none());
    // (Westbound, no way off before its end: the half-diamond's off-ramp is the eastbound's.)
    let (west_in, west_out) = (
        on(g, MOTORWAY_WEST, [x1, WEST_Z as f32]),
        on(g, MOTORWAY_WEST, [x0, WEST_Z as f32]),
    );
    assert!(drive(g, west_in, town).is_none());
    let through = drive(g, west_in, west_out).expect("along it");
    assert!((through.len - (x1 - x0)).abs() < 0.5, "{} m", through.len);
    assert!(drive(g, west_out, west_in).is_none());
}

#[test]
fn one_way_roads_keep_every_place_reachable() {
    let w = world();
    let g = &w.travel.graph;
    assert!(g.strongly_connected());
    // (The loop's north side turned round: a corner cars come to and can't leave.)
    let mut roads = w.travel.file.roads.clone();
    roads[LOOP[3]].oneway = OneWay::Along;
    assert!(!Graph::build(&roads).strongly_connected());
    // (Or two-way: all well again.)
    roads[LOOP[3]].oneway = OneWay::Both;
    assert!(Graph::build(&roads).strongly_connected());
}

/// A classic world's network (no motorway, no one-way roads) as it was before the hierarchy:
/// its nodes, edges, junctions and boxes, and a fingerprint of every node and edge.
#[test]
fn a_classic_world_makes_the_same_graph() {
    let content = common::content();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/saves");
    let mut worlds = Vec::new();
    for name in ["v042_village.json", "v035_normal.json", "v036_hamlets.json"] {
        let json = std::fs::read_to_string(dir.join(name)).unwrap();
        worlds.push((name, World::from_save_json(&content, &json).unwrap()));
    }
    let (town, _) = common::hamlets::grid(&content, 5, 256, 6, 40, true);
    worlds.push(("grid", World::from_json(&content, &town, 5).unwrap()));
    // (Each: nodes, edges, junctions, boxes, lit junctions, the fingerprint; as on main.)
    let before: [(usize, usize, usize, usize, usize, u64); 4] = [
        (4, 2, 0, 0, 0, 0xb6fea3534c334e4c),
        (14, 19, 12, 76, 12, 0x1492edd1b0b9cb6d),
        (4, 3, 0, 0, 0, 0xc5f5c621679c4e11),
        (6, 5, 0, 0, 0, 0xcf5cab4b527fadaa),
    ];
    for ((name, w), want) in worlds.iter().zip(before) {
        let g = &w.travel.graph;
        let mut h: u64 = 0xcbf29ce484222325;
        let mut mix = |v: u32| h = (h ^ v as u64).wrapping_mul(0x100000001b3);
        for n in &g.nodes {
            mix(n[0].to_bits());
            mix(n[1].to_bits());
        }
        for e in &g.edges {
            for v in [e.road, e.a, e.b, e.s0.to_bits(), e.s1.to_bits()] {
                mix(v);
            }
        }
        let got = (
            g.nodes.len(),
            g.edges.len(),
            g.junctions().count(),
            g.boxes.len(),
            w.travel.signals.len(),
            h,
        );
        assert_eq!(got, want, "{name}: {got:#x?}");
        assert_eq!(g.merge_nodes().count(), 0, "{name}");
        assert!(g.strongly_connected(), "{name}");
    }
}

/// The fixture with the motorway's carriageways and ramps (`motorway`, two-way) or the avenue
/// (`avenue`) of another kind: the same roads at another speed.
fn rekinded(motorway: Option<&str>, avenue: Option<&str>) -> World {
    let mut town: Value = serde_json::from_str(&hierarchy().town).unwrap();
    for (i, r) in town["roads"].as_array_mut().unwrap().iter_mut().enumerate() {
        let kind = match i {
            MOTORWAY_EAST | MOTORWAY_WEST | OFF_RAMP | ON_RAMP => motorway,
            AVENUE => avenue,
            _ => None,
        };
        if let Some(kind) = kind {
            r["kind"] = json!(kind);
            r.as_object_mut().unwrap().remove("oneway");
        }
    }
    town["meta"]["roads"] = town["roads"].clone();
    World::from_json(&common::content(), &town.to_string(), 1).expect("the hierarchy loads")
}

/// Ticks of an empty drive from `a` to `b` in `w` (`cars::empty_drive_ticks`).
fn drive_ticks(w: &World, a: [f32; 2], b: [f32; 2]) -> f32 {
    sim_core::cars::empty_drive_ticks(&w.travel, a, b).expect("a way")
}

#[test]
fn motorway_is_faster() {
    // Along the eastbound carriageway, over the street and the river, most of the world across.
    let (a, b) = ([30.0, EAST_Z as f32], [470.0, EAST_Z as f32]);
    let motorway = drive_ticks(&world(), a, b);
    let street = drive_ticks(&rekinded(Some("street"), None), a, b);
    assert!(
        motorway < 0.6 * street,
        "{motorway} ticks by the motorway, {street} by a street"
    );
    // (Faster than the town's speed on average, from standing to standing.)
    let secs = motorway / sim_core::DRIVE_TICKS_PER_SECOND;
    assert!(
        440.0 / secs > 1.5 * sim_core::travel::TOWN_SPEED,
        "{secs} s of drive time"
    );
}

#[test]
fn the_avenue_is_faster_than_a_street() {
    // Its length, past both cross streets (straight through their junctions at its speed).
    let (a, b) = ([10.0, AVENUE_Z as f32], [290.0, AVENUE_Z as f32]);
    let avenue = drive_ticks(&world(), a, b);
    let street = drive_ticks(&rekinded(None, Some("street")), a, b);
    let lane = drive_ticks(&rekinded(None, Some("lane")), a, b);
    assert!(
        avenue < 0.85 * street,
        "{avenue} ticks on the avenue, {street} on a street"
    );
    assert!(
        street < lane,
        "{street} ticks on a street, {lane} on a lane"
    );
}

/// Where a car whose driveway is at `at` leaves the fixture (`cars::way_out`).
fn way_out(w: &World, at: [f32; 2]) -> ([f32; 2], bool) {
    sim_core::cars::way_out(&w.travel, &w.exits, at).expect("a way out")
}

#[test]
fn leaving_cars_take_the_motorway_when_it_is_nearer_by_time() {
    let w = world();
    // North of the motorway on the street: its end of town 70 m up the street, the motorway's
    // west end 165 m off by the on-ramp (and what the motorway saves beyond town).
    let (at, by) = way_out(&w, [STREET_X as f32 + 6.0, 250.0]);
    assert!(by, "by the motorway: {at:?}");
    assert!(at[0] < 1.0, "at its west end: {at:?}");
    // On the avenue by its own end of town: the motorway is far further by time.
    let (at, by) = way_out(&w, [30.0, AVENUE_Z as f32 - 10.0]);
    assert!(!by, "{at:?}");
    assert!(
        at[0] < 1.0 && (at[1] - AVENUE_Z as f32).abs() < 1.0,
        "by the avenue: {at:?}"
    );
}

#[test]
fn no_mouth_parking_or_kerb_on_a_motorway_or_a_ramp() {
    let w = world();
    let g = &w.travel.graph;
    // (`cars::ROAD_REACH`.)
    let reach = 30.0;
    // (A driveway, a parking spot or a kerb anywhere about the motorway and its ramps: on the
    // street under it or by it, else nowhere.)
    let mut found = 0;
    for x in (0..WIDTH).step_by(10) {
        for z in (170..240).step_by(5) {
            let p = [x as f32, z as f32];
            if let Some(r) = sim_core::cars::enterable_near(g, p, reach) {
                let road = &g.roads[r.road as usize];
                assert!(
                    road.enterable(),
                    "{p:?}: road {} ({:?})",
                    road.source,
                    road.kind
                );
                found += 1;
            }
        }
    }
    assert!(found > 0, "the street by the motorway");
    // On a carriageway far from any other road: none.
    assert!(sim_core::cars::enterable_near(g, [60.0, EAST_Z as f32], reach).is_none());
    // By the off-ramp's T: the street, not the ramp nearer.
    let r = sim_core::cars::enterable_near(g, [132.0, EAST_Z as f32 - 22.0], reach).unwrap();
    assert_eq!(g.roads[r.road as usize].source, BRIDGE_STREET as u32);
}

// The lanes (S-LANES): where on each road a car drives.

/// Where a car is and which way it faces each tick of an empty drive from `a` to `b`
/// (`cars::empty_drive`).
fn poses(w: &World, a: [f32; 2], b: [f32; 2]) -> Vec<([f32; 2], f32)> {
    sim_core::cars::empty_drive(&w.travel, a, b).expect("a way")
}

/// An avenue's lanes, right of its middle line the way a car goes (m): its median's half and its
/// carriageway's two lanes (`roadKinds.ts`: 14 m, 1.35 m sidewalks, a 2.6 m median).
const AVENUE_INNER: f32 = 1.3 + 2.175 * 0.5;
const AVENUE_OUTER: f32 = 1.3 + 2.175 * 1.5;
/// A motorway carriageway's outer lane (m from its middle line, away from its pair: `motorway.ts`
/// `MOTORWAY`, 0.55 in from its inside edge, two 3.5 m lanes, its 9 m width's half).
const MOTORWAY_OUTER: f32 = 0.55 + 3.5 * 1.5 - 4.5;

/// Whether `p` is in a junction's box (`Graph::boxes`), or a car's half length (2.5 m) off one.
fn boxed(g: &Graph, p: [f32; 2]) -> bool {
    g.boxes.iter().any(|(b, r)| dist(*b, p) < r + 2.5)
}

/// The poses of `drive` with `x` between `x0` and `x1`, out of the junctions' boxes.
fn between(g: &Graph, drive: &[([f32; 2], f32)], x0: f32, x1: f32) -> Vec<[f32; 2]> {
    let out: Vec<[f32; 2]> = drive
        .iter()
        .map(|d| d.0)
        .filter(|p| p[0] > x0 && p[0] < x1 && !boxed(g, *p))
        .collect();
    assert!(out.len() > 5, "{x0} to {x1}: {} poses", out.len());
    out
}

#[test]
fn avenue_cars_keep_to_the_outer_lane_each_way() {
    let w = world();
    let g = &w.travel.graph;
    let z = AVENUE_Z as f32;
    // (East, its right the +z side; west, the -z side; straight along it past both cross
    // streets.)
    for (a, b, side) in [([10.0, z], [290.0, z], 1.0), ([290.0, z], [10.0, z], -1.0)] {
        let drive = poses(&w, a, b);
        for p in between(g, &drive, 20.0, 280.0) {
            let off = (p[1] - z) * side;
            assert!(
                (off - AVENUE_OUTER).abs() < 0.05,
                "{a:?} to {b:?}: at {p:?}"
            );
        }
    }
}

#[test]
fn avenue_left_turners_take_the_inner_lane() {
    let w = world();
    let g = &w.travel.graph;
    let z = AVENUE_Z as f32;
    // East, turning left (north, -z) into the east cross street: the inner lane from Bridge
    // Street's T on (x 140), the outer before it (straight on at the west cross street and the T);
    // in the cross street's lane after.
    let left = poses(&w, [10.0, z], [210.0, 50.0]);
    for p in between(g, &left, 20.0, 125.0) {
        assert!((p[1] - z - AVENUE_OUTER).abs() < 0.05, "before: {p:?}");
    }
    for p in between(g, &left, 155.0, 195.0) {
        assert!(
            (p[1] - z - AVENUE_INNER).abs() < 0.05,
            "the stretch it turns left at the end of: {p:?}"
        );
    }
    let north: Vec<[f32; 2]> = left
        .iter()
        .map(|d| d.0)
        .filter(|p| p[1] < 70.0 && p[1] > 50.0 && !boxed(g, *p))
        .collect();
    assert!(!north.is_empty());
    for p in north {
        assert!(
            (p[0] - 210.0 - 1.05).abs() < 0.05,
            "up the cross street: {p:?}"
        );
    }
    // East, turning right (south) into it: the outer lane all along.
    let right = poses(&w, [10.0, z], [210.0, 110.0]);
    for p in between(g, &right, 20.0, 195.0) {
        assert!(
            (p[1] - z - AVENUE_OUTER).abs() < 0.05,
            "turning right: {p:?}"
        );
    }
    // West, turning left (south, +z) into the west cross street: the inner lane from the T on.
    let west = poses(&w, [290.0, z], [90.0, 110.0]);
    for p in between(g, &west, 155.0, 280.0) {
        assert!((z - p[1] - AVENUE_OUTER).abs() < 0.05, "before: {p:?}");
    }
    for p in between(g, &west, 105.0, 125.0) {
        assert!(
            (z - p[1] - AVENUE_INNER).abs() < 0.05,
            "turning left at its end: {p:?}"
        );
    }
}

#[test]
fn avenue_cars_change_lanes_only_in_a_box() {
    let w = world();
    let g = &w.travel.graph;
    let z = AVENUE_Z as f32;
    // Every way along the avenue, onto it and off it at the cross streets, either way: out of the
    // junctions' boxes, always in one lane or the other (but for the start of a turn's curve, a
    // few cm), so no two going the same way cross each other's lanes there.
    let ends = [
        [10.0, z],
        [290.0, z],
        [90.0, 50.0],
        [90.0, 110.0],
        [210.0, 50.0],
        [210.0, 110.0],
    ];
    let mut on_avenue = 0;
    for &a in &ends {
        for &b in &ends {
            if a == b || a[0] == b[0] {
                continue;
            }
            for (p, _) in poses(&w, a, b) {
                if (p[1] - z).abs() > 7.0 || p[0] < 15.0 || p[0] > 285.0 || boxed(g, p) {
                    continue;
                }
                let off = (p[1] - z).abs();
                let in_lane = [AVENUE_INNER, AVENUE_OUTER]
                    .iter()
                    .any(|l| (off - l).abs() < 0.1);
                assert!(in_lane, "{a:?} to {b:?}: between lanes at {p:?}");
                on_avenue += 1;
            }
        }
    }
    assert!(on_avenue > 1000, "{on_avenue} poses on the avenue");
}

#[test]
fn one_way_loop_cars_go_its_way_down_its_middle() {
    let w = world();
    let g = &w.travel.graph;
    // Round the block from the west side back to it, the long way (one-way).
    let drive = poses(&w, [300.0, 30.0], [300.0, 50.0]);
    let (mut south, mut east, mut north) = (0, 0, 0);
    for k in 1..drive.len() {
        let (p, q) = (drive[k - 1].0, drive[k].0);
        if boxed(g, q) || dist(p, q) < 1e-3 {
            continue;
        }
        let corner = |x: f32, z: f32| dist(q, [x, z]) < 8.0;
        if corner(300.0, 20.0) || corner(360.0, 20.0) || corner(360.0, 60.0) || corner(300.0, 60.0)
        {
            continue;
        }
        // (Each side its way, its middle line under the car.)
        if (q[1] - 20.0).abs() < 3.0 {
            assert!(
                q[0] > p[0] && (q[1] - 20.0).abs() < 0.05,
                "the south side: {q:?}"
            );
            south += 1;
        } else if (q[0] - 360.0).abs() < 3.0 {
            assert!(
                q[1] > p[1] && (q[0] - 360.0).abs() < 0.05,
                "the east side: {q:?}"
            );
            east += 1;
        } else if (q[1] - 60.0).abs() < 3.0 {
            assert!(
                q[0] < p[0] && (q[1] - 60.0).abs() < 0.05,
                "the north side: {q:?}"
            );
            north += 1;
        }
    }
    assert!(
        south > 10 && east > 10 && north > 10,
        "{south} {east} {north}"
    );
}

#[test]
fn motorway_cars_keep_to_the_outer_lane_of_their_carriageway() {
    let w = world();
    let g = &w.travel.graph;
    // (Each carriageway's outer lane away from the other: the eastbound's north of its middle,
    // the westbound's south.)
    let (east, west) = (EAST_Z as f32, WEST_Z as f32);
    let along = poses(&w, [30.0, east], [470.0, east]);
    for p in between(g, &along, 40.0, 460.0) {
        assert!(
            (p[1] - (east - MOTORWAY_OUTER)).abs() < 0.05,
            "eastbound: {p:?}"
        );
    }
    let back = poses(&w, [470.0, west], [30.0, west]);
    for p in between(g, &back, 40.0, 460.0) {
        assert!(
            (p[1] - (west + MOTORWAY_OUTER)).abs() < 0.05,
            "westbound: {p:?}"
        );
    }
}

/// How far right of the town file's road `source`'s middle line (the way its points run) `p` is
/// (m).
fn right_of(g: &Graph, source: usize, p: [f32; 2]) -> f32 {
    let road = &g.roads[road_of(g, source) as usize];
    let (s, _) = road.project(p);
    let (q, t) = (road.at(s), road.tangent(s));
    (p[0] - q[0]) * -t[1] + (p[1] - q[1]) * t[0]
}

/// The most a drive moves across its road's middle line a metre along it (m), with `x` between
/// `x0` and `x1`: the road `source_at(x)`'s.
fn sideways(
    g: &Graph,
    drive: &[([f32; 2], f32)],
    x0: f32,
    x1: f32,
    source_at: impl Fn(f32) -> usize,
) -> f32 {
    let mut most = 0.0f32;
    for k in 1..drive.len() {
        let (p, q) = (drive[k - 1].0, drive[k].0);
        let step = dist(p, q);
        let (a, b) = (source_at(p[0]), source_at(q[0]));
        if step < 0.05 || a != b || q[0] < x0 || q[0] > x1 || p[0] < x0 || p[0] > x1 {
            continue;
        }
        let across = (right_of(g, b, q) - right_of(g, a, p)).abs();
        most = most.max(across / step);
    }
    most
}

#[test]
fn ramps_ease_into_the_carriageways_lane_and_out_of_it() {
    let w = world();
    let g = &w.travel.graph;
    let (east, west) = (EAST_Z as f32, WEST_Z as f32);
    // Onto the westbound by the on-ramp: eased into its outer lane by the merge, then in it.
    let on = poses(&w, [STREET_X as f32, 260.0], [3.0, west]);
    let merged: Vec<[f32; 2]> = on.iter().map(|d| d.0).filter(|p| p[0] < 18.0).collect();
    assert!(merged.len() > 5);
    for p in merged {
        assert!(
            (p[1] - (west + MOTORWAY_OUTER)).abs() < 0.05,
            "merged: {p:?}"
        );
    }
    // Off the eastbound by the off-ramp: out of its outer lane along the ramp, down its middle
    // past the taper.
    let off = poses(&w, [3.0, east], [STREET_X as f32, 120.0]);
    let ramp: Vec<[f32; 2]> = off
        .iter()
        .map(|d| d.0)
        .filter(|p| p[0] > 60.0 && p[0] < 100.0)
        .collect();
    assert!(ramp.len() > 5);
    for p in ramp {
        assert!(right_of(g, OFF_RAMP, p).abs() < 0.05, "on the ramp: {p:?}");
    }
    // (No step across: never more than 0.2 m across its road a metre along it about the merge
    // and the diverge, the ramp's bends rounded and all; a step from the ramp's middle to the
    // lane there, with no taper, moved it 0.8 m a metre at the merge and 0.36 at the diverge.)
    let on_most = sideways(g, &on, 5.0, 60.0, |x| {
        if x < 20.0 { MOTORWAY_WEST } else { ON_RAMP }
    });
    let off_most = sideways(g, &off, 5.0, 60.0, |x| {
        if x < 20.0 { MOTORWAY_EAST } else { OFF_RAMP }
    });
    assert!(on_most < 0.2, "on-ramp: {on_most} m a metre");
    assert!(off_most < 0.2, "off-ramp: {off_most} m a metre");
}

// The traffic (S-TRAFFIC): the deck and the ground, following at speed, the merges.

use sim_core::cars::drill;

/// The fixture with its motorway's decks taken off it (`raised`, `bridge`): the same roads, the
/// carriageways on the ground all along.
fn without_decks() -> World {
    let mut town: Value = serde_json::from_str(&hierarchy().town).unwrap();
    for r in town["roads"].as_array_mut().unwrap() {
        let r = r.as_object_mut().unwrap();
        r.remove("raised");
        r.remove("bridge");
    }
    town["meta"]["roads"] = town["roads"].clone();
    World::from_json(&common::content(), &town.to_string(), 1).expect("the hierarchy loads")
}

/// Carriageway `source`'s middle line's z (its points run along x) and the way its cars go along x
/// (+1, -1): as laid, whichever side it lies on.
fn carriageway(g: &Graph, source: usize) -> (f32, f32) {
    let r = &g.roads[road_of(g, source) as usize];
    let way = match r.oneway {
        OneWay::Against => -1.0,
        _ => 1.0,
    };
    let (a, b) = (r.at(0.0), r.at(r.len()));
    (r.at(r.len() / 2.0)[1], way * (b[0] - a[0]).signum())
}

/// A drive along carriageway `source` its way, from 30 m in from the end it comes in by to 5 m
/// short of the other.
fn along(g: &Graph, source: usize) -> ([f32; 2], [f32; 2]) {
    let (z, way) = carriageway(g, source);
    let (x0, x1) = (MOTORWAY_X0 as f32, MOTORWAY_X1 as f32);
    if way > 0.0 {
        ([x0 + 30.0, z], [x1 - 5.0, z])
    } else {
        ([x1 - 30.0, z], [x0 + 5.0, z])
    }
}

/// The tick of drill car `c`'s way where it's nearest `p`.
fn tick_at(w: &World, c: usize, p: [f32; 2]) -> f32 {
    let n = drill::ticks(w, c).floor() as usize;
    (0..=n)
        .min_by(|&a, &b| {
            dist(drill::at(w, c, a as f32), p).total_cmp(&dist(drill::at(w, c, b as f32), p))
        })
        .unwrap() as f32
}

/// Whether car `c` waits for another car (`Car::waits_for`: a car's key).
fn waits_for_a_car(w: &World, c: usize) -> Option<usize> {
    w.cars[c]
        .waits_for
        .filter(|&k| k >= 1 << 63)
        .map(|k| (k & !(1 << 63)) as usize)
}

/// A street car up Bridge Street under both carriageways, and a car along each, each over the
/// street `offset` ticks after the street car is under it: whether any car waited for another, and
/// how near the street car's middle came to a motorway car's (m), and whether it was on a deck
/// then.
fn under_the_bridge(make: &dyn Fn() -> World, offset: f32) -> (bool, f32, bool) {
    let mut w = make();
    let g = w.travel.graph.clone();
    let x = STREET_X as f32;
    let (za, zb) = (
        carriageway(&g, MOTORWAY_EAST).0,
        carriageway(&g, MOTORWAY_WEST).0,
    );
    let (z0, z1) = (za.min(zb) - 90.0, za.max(zb) + 50.0);
    let street = drill::add(&mut w, [x, z0], [x, z1], 0.0).expect("up the street");
    let mut cars = vec![street];
    const LEAD: f32 = 90.0;
    for side in [MOTORWAY_EAST, MOTORWAY_WEST] {
        let (z, _) = carriageway(&g, side);
        let (a, b) = along(&g, side);
        let c = drill::add(&mut w, a, b, 0.0).expect("along the carriageway");
        let under = tick_at(&w, street, [x, z]);
        let over = tick_at(&w, c, [x, z]);
        // (Both there at the same tick: the street car `LEAD` ticks before it gets under, the
        // motorway car `LEAD` before it's over it, `offset` later.)
        drill::put(&mut w, c, (over - LEAD - offset).max(0.0));
        cars.push(c);
        // (The street car set off `LEAD` before it's under the first.)
        if side == MOTORWAY_EAST {
            drill::put(&mut w, street, (under - LEAD).max(0.0));
        }
    }
    let mut waited = false;
    let mut nearest = (f32::INFINITY, false);
    for _ in 0..3000 {
        drill::step(&mut w);
        for &c in &cars {
            waited |= waits_for_a_car(&w, c).is_some();
        }
        if let Some(s) = w.cars[street].shown {
            for &c in &cars[1..] {
                if let Some(m) = w.cars[c].shown {
                    let d = dist(s.0, m.0);
                    if d < nearest.0 {
                        nearest = (d, drill::on_deck(&w, c));
                    }
                }
            }
        }
        if cars.iter().all(|&c| w.cars[c].shown.is_none()) {
            break;
        }
    }
    assert!(
        cars.iter().all(|&c| w.cars[c].shown.is_none()),
        "all through"
    );
    (waited || w.traffic.guarded > 0, nearest.0, nearest.1)
}

#[test]
fn nobody_stops_under_the_bridge() {
    // On the deck over the street, the motorway's cars and the street's go by each other however
    // they come: right over and under each other, nobody waits for anyone.
    let mut met = false;
    for offset in [-6.0, -3.0, 0.0, 3.0, 6.0] {
        let (waited, nearest, deck) = under_the_bridge(&world, offset);
        assert!(!waited, "offset {offset}: a car waited for another");
        if nearest < 2.0 {
            assert!(deck, "offset {offset}: over the street, on its deck");
            met = true;
        }
    }
    assert!(met, "right over each other at least once");
    // (The same roads with the carriageways on the ground: they meet there, and wait.)
    let held = [-6.0, -3.0, 0.0, 3.0, 6.0]
        .iter()
        .any(|&o| under_the_bridge(&without_decks, o).0);
    assert!(held, "on the ground, one waits for the other");
    // On foot under the deck: a car over them is no car in their way; on the ground it would be.
    for (make, deck) in [
        (&world as &dyn Fn() -> World, true),
        (&without_decks, false),
    ] {
        let mut w = make();
        let g = w.travel.graph.clone();
        let (z, _) = carriageway(&g, MOTORWAY_EAST);
        let (a, b) = along(&g, MOTORWAY_EAST);
        let c = drill::add(&mut w, a, b, 0.0).unwrap();
        let t = tick_at(&w, c, [STREET_X as f32, z]);
        drill::put(&mut w, c, t);
        drill::step(&mut w);
        let p = w.cars[c].shown.expect("on its way").0;
        assert_eq!(drill::on_deck(&w, c), deck);
        let walk = drill::walks_into(&w, [p[0] - 0.6, p[1]], [p[0] + 0.6, p[1]]);
        assert_eq!(walk.is_some(), !deck, "walking under it: {walk:?}");
    }
}

/// Drill cars `cars` of `w` nearer each other than a body (their middles under 2.5 m apart, the
/// widest car 1.9 m across): pairs on levels that meet, overlapping.
fn overlapping(w: &World, cars: &[usize]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (i, &a) in cars.iter().enumerate() {
        for &b in &cars[i + 1..] {
            if let (Some(p), Some(q)) = (w.cars[a].shown, w.cars[b].shown)
                && dist(p.0, q.0) < 2.5
            {
                out.push((a, b));
            }
        }
    }
    out
}

/// The way on-ramp cars come to their ramp: from 50 m along Bridge Street beyond the ramp's T,
/// away from the motorway, to the far end of the carriageway the ramp merges into. And that
/// carriageway.
fn up_the_on_ramp(g: &Graph) -> ([f32; 2], [f32; 2], usize) {
    let ramp = &g.roads[road_of(g, ON_RAMP) as usize];
    let (t, merge) = (ramp.at(0.0), ramp.at(ramp.len()));
    let side = [MOTORWAY_EAST, MOTORWAY_WEST]
        .into_iter()
        .min_by(|&a, &b| {
            let off = |s: usize| (carriageway(g, s).0 - merge[1]).abs();
            off(a).total_cmp(&off(b))
        })
        .unwrap();
    let away = (t[1] - merge[1]).signum();
    let (_, to) = along(g, side);
    ([t[0], t[1] + away * 50.0], to, side)
}

#[test]
fn merges_settle() {
    // Cars onto the motorway by the on-ramp, one every 3 s, into a stream along its carriageway,
    // one every 2 s: each waits up the ramp for its gap, none stops a car on the carriageway
    // before it's on it, nobody overlaps, all get through, and every claim is let go after.
    let mut w = world();
    let g = w.travel.graph.clone();
    let (from, to, side) = up_the_on_ramp(&g);
    let (a, b) = along(&g, side);
    let (z, _) = carriageway(&g, side);
    let mut ramp: Vec<usize> = Vec::new();
    let mut through: Vec<usize> = Vec::new();
    let mut waited_up_ramp = 0;
    for tick in 0..30_000u32 {
        let room = |w: &World, at: [f32; 2]| {
            w.cars
                .iter()
                .all(|k| k.shown.is_none_or(|s| dist(s.0, at) > 8.0))
        };
        if tick < 1500 && tick % 45 == 0 && room(&w, from) {
            let c = drill::add(&mut w, from, to, 0.0)
                .unwrap_or_else(|| panic!("onto the motorway: {from:?} to {to:?}"));
            assert_eq!(drill::merges(&w, c).len(), 1, "one merge");
            ramp.push(c);
        }
        if tick < 1500 && tick % 30 == 0 && room(&w, a) {
            through.push(drill::add(&mut w, a, b, 0.0).expect("along it"));
        }
        drill::step(&mut w);
        let all: Vec<usize> = ramp.iter().chain(&through).copied().collect();
        let o = overlapping(&w, &all);
        assert_eq!(o, vec![], "tick {tick}");
        // (On the carriageway, waiting only for one on it already: in its lane, or its taper.)
        for &c in &through {
            if let Some(k) = waits_for_a_car(&w, c).filter(|k| ramp.contains(k)) {
                let p = w.cars[k].shown.expect("on its way").0;
                assert!(
                    (p[1] - z).abs() < 4.5 + 1.5,
                    "tick {tick}: car {c} waits for {k} up the ramp at {p:?}"
                );
            }
        }
        for &c in &ramp {
            let m = drill::merges(&w, c);
            let t = w.cars[c].progress;
            if w.cars[c].shown.is_some()
                && !m.is_empty()
                && t < m[0]
                && w.traffic.waiting(sim_core::cars::car_key(c))
            {
                waited_up_ramp += 1;
            }
        }
        if tick > 1500 && all.iter().all(|&c| w.cars[c].shown.is_none()) {
            break;
        }
    }
    let all: Vec<usize> = ramp.iter().chain(&through).copied().collect();
    assert!(
        all.iter().all(|&c| w.cars[c].shown.is_none()),
        "all through"
    );
    assert!(waited_up_ramp > 0, "a gap waited for now and then");
    assert_eq!(
        w.traffic.guarded, 0,
        "nobody kept out of another by the last resort"
    );
    assert_eq!(w.traffic.claims_out(), (0, 0), "every claim let go");
}

#[test]
fn motorway_cars_keep_further_back_the_faster_they_go() {
    // A car setting off from standing on the carriageway, and a string of cars coming up behind
    // it already at speed: none runs into the one in front, and at speed each keeps well back.
    let mut w = world();
    let g = w.travel.graph.clone();
    let side = MOTORWAY_EAST;
    let (a, b) = along(&g, side);
    let way = (b[0] - a[0]).signum();
    let lead = drill::add(&mut w, [a[0] + way * 220.0, a[1]], b, 0.0).unwrap();
    let mut cars = vec![lead];
    for k in 0..6 {
        let c = drill::add(&mut w, a, b, 0.0).unwrap();
        // (Each 30 m behind the one before as they come, the first 50 m short of the lead.)
        let x = a[0] + way * (170.0 - 30.0 * k as f32);
        let t = tick_at(&w, c, [x, a[1]]);
        drill::put(&mut w, c, t);
        cars.push(c);
    }
    let mut fast = 0;
    // (Each car's progress the last two ticks: on the move both, it's driving on at speed, not
    // stopping and setting off again.)
    let mut was: Vec<[f32; 2]> = cars.iter().map(|&c| [w.cars[c].progress; 2]).collect();
    for tick in 0..20_000 {
        drill::step(&mut w);
        assert_eq!(overlapping(&w, &cars), vec![], "tick {tick}");
        for (i, &c) in cars.iter().enumerate() {
            let car = &w.cars[c];
            let (t, [t1, t2]) = (car.progress, was[i]);
            was[i] = [t, t1];
            let Some(me) = car.shown else { continue };
            let speed = dist(drill::at(&w, c, t), drill::at(&w, c, t + 1.0))
                * sim_core::DRIVE_TICKS_PER_SECOND;
            if i == 0 || speed < 25.0 || !(t > t1 && t1 > t2) {
                continue;
            }
            // (The nearest car ahead of it on its way.)
            let ahead = cars
                .iter()
                .filter(|&&o| o != c)
                .filter_map(|&o| w.cars[o].shown)
                .map(|o| (o.0[0] - me.0[0]) * way)
                .filter(|&d| d > 0.0)
                .fold(f32::INFINITY, f32::min);
            if ahead < f32::INFINITY {
                fast += 1;
                assert!(
                    ahead > 20.0,
                    "tick {tick}: car {c} at {speed} m/s {ahead} m behind"
                );
            }
        }
        if cars.iter().all(|&c| w.cars[c].shown.is_none()) {
            break;
        }
    }
    assert!(
        cars.iter().all(|&c| w.cars[c].shown.is_none()),
        "all through"
    );
    assert!(fast > 100, "{fast} ticks behind another at speed");
    assert_eq!(w.traffic.guarded, 0);
}

// The junctions (S-TRAFFIC): the avenue's lit, the arms nobody comes in by without a phase.

/// The junction (`Travel::signals`, `Travel::give_way`) at `p`, and whether it's lit.
fn junction_at(w: &World, p: [f32; 2]) -> (&sim_core::traffic::Signal, bool) {
    w.travel
        .signals
        .iter()
        .map(|s| (s, true))
        .chain(w.travel.give_way.iter().map(|s| (s, false)))
        .find(|(s, _)| dist(s.at, p) < 1.0)
        .unwrap_or_else(|| panic!("a junction at {p:?}"))
}

/// Where ramp `source` meets the street: its end away from the motorway's merges.
fn ramp_t(g: &Graph, source: usize) -> [f32; 2] {
    let r = &g.roads[road_of(g, source) as usize];
    let merges: Vec<[f32; 2]> = g.merge_nodes().map(|n| g.nodes[n]).collect();
    let near_merge = |p: [f32; 2]| {
        merges
            .iter()
            .map(|&m| dist(m, p))
            .fold(f32::INFINITY, f32::min)
    };
    let (a, b) = (r.at(0.0), r.at(r.len()));
    if near_merge(a) > near_merge(b) { a } else { b }
}

#[test]
fn avenue_junctions_are_lit_with_their_carriageways_width() {
    let w = world();
    let g = &w.travel.graph;
    let avenue = road_of(g, AVENUE);
    let z = AVENUE_Z as f32;
    for x in [90.0, STREET_X as f32, 210.0] {
        let (sig, lit) = junction_at(&w, [x, z]);
        assert!(lit, "the avenue's junction at x {x} is lit");
        let arms: Vec<_> = sig.arms.iter().filter(|a| a.road == avenue).collect();
        assert_eq!(arms.len(), 2, "x {x}: the avenue through it");
        for a in arms {
            // (Kerb to kerb: 14 m less its sidewalks, the renderer's `carriageway`.)
            assert!(
                (2.0 * a.kerb - 11.3).abs() < 1e-3,
                "x {x}: {} m",
                2.0 * a.kerb
            );
            assert!(a.crosswalk && a.inbound(), "x {x}");
        }
        // (The two arms of the avenue go together; the cross road's another phase.)
        assert!(sig.phases >= 2, "x {x}");
    }
    // The ramps' T's: lit, the ramp's arm no crosswalk; the street's crosswalks as any.
    for ramp in [OFF_RAMP, ON_RAMP] {
        let (sig, lit) = junction_at(&w, ramp_t(g, ramp));
        assert!(lit, "ramp {ramp}'s T is lit");
        let r = road_of(g, ramp);
        for a in &sig.arms {
            assert_eq!(
                a.crosswalk,
                a.road != r,
                "ramp {ramp}: arm of road {}",
                a.road
            );
        }
        let ramp_arm = sig.arms.iter().find(|a| a.road == r).unwrap();
        assert!(
            (2.0 * ramp_arm.kerb - 5.0).abs() < 1e-3,
            "a ramp has no sidewalks"
        );
    }
}

#[test]
fn arms_cars_only_leave_by_have_no_phase() {
    let w = world();
    let g = &w.travel.graph;
    // (Cars only leave the street's T up the on-ramp; they only come in down the off-ramp.)
    for (ramp, leaves) in [(ON_RAMP, true), (OFF_RAMP, false)] {
        let (sig, _) = junction_at(&w, ramp_t(g, ramp));
        let r = road_of(g, ramp);
        for a in &sig.arms {
            let out_only = a.road == r && leaves;
            assert_eq!(
                !a.inbound(),
                out_only,
                "ramp {ramp}: arm of road {}",
                a.road
            );
            if a.inbound() {
                assert!(a.phase < sig.phases);
            } else {
                assert_eq!(a.phase, sim_core::traffic::NO_PHASE);
            }
        }
        // (Every phase someone comes in on.)
        for p in 0..sig.phases {
            assert!(
                sig.arms.iter().any(|a| a.phase == p),
                "ramp {ramp}: phase {p}"
            );
        }
    }
    // The one-way loop's corner on its link: its west side only leaves it, its north side only
    // comes in.
    let (sig, _) = junction_at(&w, [300.0, 60.0]);
    let (west, north) = (road_of(g, LOOP[0]), road_of(g, LOOP[3]));
    for a in &sig.arms {
        assert_eq!(a.inbound(), a.road != west, "arm of road {}", a.road);
    }
    assert!(sig.arms.iter().any(|a| a.road == north && a.inbound()));
    // (Shown to the renderer: no head on the out-only arms.)
    let view: Value = serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    let junctions = view["junctions"].as_array().expect("junctions");
    let at = |p: [f32; 2]| {
        junctions
            .iter()
            .find(|j| {
                dist(
                    [
                        j["x"].as_f64().unwrap() as f32,
                        j["z"].as_f64().unwrap() as f32,
                    ],
                    p,
                ) < 1.0
            })
            .expect("in the view")
    };
    let on = at(ramp_t(g, ON_RAMP));
    for a in on["arms"].as_array().unwrap() {
        let ramp = a["road"].as_u64() == Some(ON_RAMP as u64);
        assert_eq!(a["light"].is_null(), ramp, "{a}");
    }
    // (Nor a pole: the kerbside furniture isn't kept off a pole that isn't there.)
    let poles: Vec<[f32; 2]> = view["lightPoles"]
        .as_array()
        .expect("poles")
        .iter()
        .map(|p| [p[0].as_f64().unwrap() as f32, p[1].as_f64().unwrap() as f32])
        .collect();
    let heads: usize = w
        .travel
        .signals
        .iter()
        .map(|s| s.arms.iter().filter(|a| a.inbound()).count())
        .sum();
    assert_eq!(poles.len(), heads, "a pole for each head");
    let (sig, _) = junction_at(&w, ramp_t(g, ON_RAMP));
    let ramp = road_of(g, ON_RAMP);
    for a in &sig.arms {
        let stands = poles.iter().any(|p| dist(*p, a.pole) < 1e-3);
        assert_eq!(stands, a.road != ramp, "the pole of road {}'s arm", a.road);
    }
}

#[test]
fn avenue_driveways_set_off_to_the_right_and_are_come_into_from_it() {
    let w = world();
    let z = AVENUE_Z as f32;
    // A driveway either side of the avenue, 60 m from its end of town: each car sets off along
    // the lanes on its own side (on the right of the way it goes), never across the median; on
    // the +z side that's east (+x), away from that end, round by the roads; and home, it comes
    // along those lanes into it.
    for (side, east) in [(1.0f32, true), (-1.0, false)] {
        let spot = [60.0, z + side * 12.0];
        let facing = if side > 0.0 {
            std::f32::consts::PI
        } else {
            0.0
        };
        for home in [false, true] {
            let drive = drill::from_driveway(&w, spot, facing, home).expect("a way");
            // (Its poses on the avenue's carriageway by the driveway, setting off from it (or,
            // home, coming into it): till it's first 20 m off.)
            let mut ends = drive.clone();
            if home {
                ends.reverse();
            }
            let near: Vec<([f32; 2], f32)> = ends
                .iter()
                .copied()
                .take_while(|(p, _)| (p[0] - 60.0).abs() < 20.0)
                .filter(|(p, _)| (p[1] - z).abs() < 5.6)
                .collect();
            assert!(near.len() > 3, "side {side} home {home}: on the avenue");
            for (p, yaw) in &near {
                assert!(
                    (p[1] - z) * side > 1.3,
                    "side {side} home {home}: across the median at {p:?}"
                );
                let going_east = yaw.sin() > 0.0;
                assert_eq!(
                    going_east, east,
                    "side {side} home {home}: at {p:?} facing {yaw}"
                );
            }
        }
    }
    // A driveway on the one-way loop's south side (cars go east there), from either side of it:
    // out and home, along it its way only.
    for side in [1.0f32, -1.0] {
        let spot = [330.0, 20.0 + side * 10.0];
        let facing = if side > 0.0 {
            std::f32::consts::PI
        } else {
            0.0
        };
        for home in [false, true] {
            let drive = drill::from_driveway(&w, spot, facing, home).expect("a way");
            let on: Vec<f32> = drive
                .iter()
                .filter(|(p, _)| (p[1] - 20.0).abs() < 0.5 && (p[0] - 330.0).abs() < 20.0)
                .map(|(_, yaw)| *yaw)
                .collect();
            assert!(!on.is_empty(), "side {side} home {home}");
            assert!(
                on.iter().all(|y| y.sin() > 0.5),
                "side {side} home {home}: {on:?}"
            );
        }
    }
}

#[test]
fn taxis_stand_on_an_avenue_on_the_riders_side() {
    // A front door either side of the avenue: the taxi in by either way into town and out by
    // either stands in the lanes on the door's side, never across the median from it (round by
    // the roads to come at it from that side). (West of West Cross by the avenue's end of town
    // only: nowhere to turn round between there and the end, the lanes on one side are only come
    // to from the end, and those on the other only lead to it; from Bridge Street the taxi
    // stands across the median as a last resort.)
    let w = world();
    let z = AVENUE_Z as f32;
    let (end, street) = ([0.5, z], [STREET_X as f32, DEPTH as f32 - 0.5]);
    let mut stands = 0;
    for (x, ways) in [
        (60.0, &[end][..]),
        (115.0, &[end, street][..]),
        (180.0, &[end, street][..]),
        (250.0, &[end, street][..]),
    ] {
        for side in [1.0f32, -1.0] {
            let door = [x, z + side * 10.0];
            for &way_in in ways {
                for &way_out in ways {
                    let (at, _) = drill::taxi_stand(&w, way_in, door, way_out)
                        .unwrap_or_else(|| panic!("door {door:?}: a stand"));
                    stands += 1;
                    assert!(
                        (at[1] - z) * side > 1.3,
                        "door {door:?}, in by {way_in:?}, out by {way_out:?}: stands at {at:?}"
                    );
                }
            }
        }
    }
    assert_eq!(stands, 26);
}
