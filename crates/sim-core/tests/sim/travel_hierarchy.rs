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

#[test]
#[ignore = "S-TRAFFIC: cars on the deck and under it don't interact"]
fn nobody_stops_under_the_bridge() {}

#[test]
#[ignore = "S-TRAFFIC: on-ramp merges wait for a gap"]
fn merges_settle() {}
