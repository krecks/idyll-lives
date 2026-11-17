//! Build mode's Land tab (`Command::ShapeLand`, `World::shape_land`): a stroke charged in Living
//! and refunded when undone, free in Creative, refused past the household's funds; its tiles merged
//! into `meta.land` and its zones replacing `meta.zones`; new water that residents walk round and
//! dried water they walk across again, refused on a lot, on a road or on anyone; what it changed
//! kept through the home edits' undo, and in the save.

use serde_json::{Value, json};
use sim_core::path::NavGrid;
use sim_core::world::GameMode;
use sim_core::{Command, World};

use crate::common;

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
    "objects":[{"id":"chair","name":"Chair","price":100,"interactions":[{"id":"sit","label":"Sit","minutes":30,"effects":{"hunger":0.01}}]}],
    "economy":{"startingFunds":5000},
    "build":{"wall":10,"door":50,"window":40,"remove":0,
        "land":{"earth":3,"mountain":0.02,"water":2,"woods":6,"meadow":1,"fields":2,"clear":1.5}}}"#;

/// The ground the masks cover: 48 m across, 32 deep.
const W: usize = 48;
const D: usize = 32;

/// A home lot in the corner, a street along z = 20 (6 m wide with its sidewalks) and open
/// ground east of the lot; Ada stands out on it, Bo at home.
fn town() -> String {
    json!({"width": W, "depth": D,
        "plots": [{"name": "Home", "x": 0, "z": 0, "w": 12, "d": 12, "entry": [6.5, 11.5]}],
        "households": [{"name": "Player", "plot": 0, "player": true}],
        "sims": [{"name": "Ada", "household": 0, "x": 40.5, "z": 4.5},
                 {"name": "Bo", "household": 0, "x": 3.5, "z": 3.5}],
        "roads": [{"points": [[0, 20], [48, 20]], "width": 6}]})
    .to_string()
}

fn world() -> World {
    let mut w = World::from_json(CONTENT, &town(), 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    w
}

/// The water mask, run-length (`[run, value, ...]`), wet where `wet(x, z)`.
fn water(wet: impl Fn(usize, usize) -> bool) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();
    for k in 0..W * D {
        let v = u32::from(wet(k % W, k / W));
        match out.len() {
            n if n >= 2 && out[n - 1] == v => out[n - 2] += 1,
            _ => out.extend([1, v]),
        }
    }
    out
}

/// A pond on the open ground: x 24..32, z 2..12.
fn in_pond(x: usize, z: usize) -> bool {
    (24..32).contains(&x) && (2..12).contains(&z)
}

/// Sends a `shapeLand` command as the page does (JSON; `water` left out when it's unchanged).
fn shape(w: &mut World, cost: i64, meta: Value, water: Option<Vec<u32>>) -> Result<(), String> {
    let mut cmd = json!({"type": "shapeLand", "household": 0, "cost": cost, "meta": meta});
    if let Some(water) = water {
        cmd["water"] = json!(water);
    }
    common::apply(w, cmd).map_err(|e| e.0)
}

fn funds(w: &World) -> i64 {
    w.households[0].funds
}

fn nav(w: &World) -> NavGrid<'_> {
    NavGrid {
        lot: &w.lot,
        blocked: w.blocked(),
        stairs: w.stair_map(),
        kept_off: &[],
        steps: None,
    }
}

/// The way on foot from (20, 7) to (36, 7), across where the pond goes.
fn way_across(w: &World) -> Vec<(i32, i32)> {
    nav(w).find_path((20, 7), (36, 7)).expect("a way across")
}

fn wet_now(w: &World, x: i32, z: i32) -> bool {
    w.blocked()[w.lot.tile_index(x, z)]
}

#[test]
fn a_stroke_is_charged_and_refunded_when_undone() {
    let mut w = world();
    let start = funds(&w);
    shape(&mut w, 300, json!({}), None).unwrap();
    assert_eq!(funds(&w), start - 300);
    // (The land's own undo sends the stroke back with the price as a refund.)
    shape(&mut w, -300, json!({}), None).unwrap();
    assert_eq!(funds(&w), start);
}

#[test]
fn creative_is_free() {
    let mut w = world();
    w.mode = GameMode::Creative;
    let start = funds(&w);
    shape(&mut w, 1_000_000, json!({}), Some(water(in_pond))).unwrap();
    assert_eq!(funds(&w), start);
    shape(&mut w, -1_000_000, json!({}), None).unwrap();
    assert_eq!(funds(&w), start, "nothing back for nothing paid");
}

#[test]
fn a_stroke_past_the_funds_is_refused_and_changes_nothing() {
    let mut w = world();
    let start = funds(&w);
    let patch = json!({"land": {"v": 1, "cell": 2, "cols": 24, "rows": 16, "tile": 64,
        "h": {"0": "AAA"}, "wl": {}, "cover": [96, 0], "seeded": "drawn"}});
    let err = shape(&mut w, start + 1, patch, Some(water(in_pond))).unwrap_err();
    assert!(err.contains("not enough money"), "{err}");
    assert_eq!(funds(&w), start);
    assert!(w.meta.get("land").is_none(), "no land");
    assert!(!wet_now(&w, 28, 7), "no pond");
    // Just enough is enough.
    shape(&mut w, start, json!({}), None).unwrap();
    assert_eq!(funds(&w), 0);
}

#[test]
fn residents_walk_round_new_water_and_across_it_once_dry() {
    let mut w = world();
    let straight = way_across(&w);
    assert_eq!(straight.len(), 17, "straight across: {straight:?}");
    shape(&mut w, 0, json!({}), Some(water(in_pond))).unwrap();
    assert!(wet_now(&w, 24, 2) && wet_now(&w, 31, 11) && !wet_now(&w, 23, 7));
    let round = way_across(&w);
    assert!(round.len() > straight.len(), "round the pond: {round:?}");
    assert!(
        round.iter().all(|&(x, z)| !in_pond(x as usize, z as usize)),
        "never in it: {round:?}"
    );
    // Dried up: straight across again.
    shape(&mut w, 0, json!({}), Some(water(|_, _| false))).unwrap();
    assert!(!wet_now(&w, 28, 7));
    assert_eq!(way_across(&w).len(), straight.len());
}

#[test]
fn someone_on_their_way_goes_round_water_made_ahead_of_them() {
    let mut w = world();
    // Bo walks from home across the open ground; the pond comes up in front of him.
    w.apply(Command::MoveTo {
        sim: 1,
        x: 36.5,
        z: 7.5,
    })
    .unwrap();
    assert!(common::until(&mut w, 60, |w| w.sims[1].pos[0] > 16.0));
    assert!(
        w.sims[1].pos[0] < 23.0,
        "not there yet: {:?}",
        w.sims[1].pos
    );
    shape(&mut w, 0, json!({}), Some(water(in_pond))).unwrap();
    let mut wet = false;
    let arrived = common::until(&mut w, 120, |w| {
        let p = w.sims[1].pos;
        wet |= in_pond(p[0] as usize, p[1] as usize);
        common::apart(p, [36.5, 7.5]) < 0.6
    });
    assert!(arrived, "got there: {:?}", w.sims[1].pos);
    assert!(!wet, "went round, not through");
}

#[test]
fn water_on_a_lot_a_road_or_a_person_is_refused() {
    let mut w = world();
    let start = funds(&w);
    let lot = shape(&mut w, 10, json!({}), Some(water(|x, z| x < 14 && z < 4))).unwrap_err();
    assert!(lot.contains("lot"), "{lot}");
    let road = shape(&mut w, 10, json!({}), Some(water(|x, z| x > 30 && z == 21))).unwrap_err();
    assert!(road.contains("road"), "{road}");
    // (Up to the sidewalk's edge is fine: the street's 3 m either side of z = 20.)
    shape(&mut w, 0, json!({}), Some(water(|x, z| x > 30 && z == 15))).unwrap();
    let ada = shape(&mut w, 10, json!({}), Some(water(|x, z| x > 36 && z < 8))).unwrap_err();
    assert!(ada.contains("Ada"), "{ada}");
    assert_eq!(funds(&w), start, "nothing charged for what was refused");
    assert!(!wet_now(&w, 40, 4));
    // A mask that isn't the ground's size.
    let short = shape(&mut w, 0, json!({}), Some(vec![10, 1])).unwrap_err();
    assert!(short.contains("covers"), "{short}");
}

#[test]
fn the_lands_tiles_merge_and_the_zones_are_replaced() {
    let mut w = world();
    // The first stroke needs the land's size.
    let err = shape(&mut w, 0, json!({"land": {"h": {"0": "AAA"}}}), None).unwrap_err();
    assert!(err.contains("size"), "{err}");
    // 200 × 100 samples: 4 × 2 tiles.
    shape(
        &mut w,
        0,
        json!({"land": {"v": 1, "cell": 2, "cols": 200, "rows": 100, "tile": 64,
            "h": {"0": "AAA", "5": "BBB"}, "wl": {"5": "WWW"}, "cover": [5000, 0], "seeded": "drawn"},
            "zones": [{"kind": "woods", "rings": []}]}),
        None,
    )
    .unwrap();
    let land = &w.meta["land"];
    assert_eq!(land["h"], json!(["AAA", "", "", "", "", "BBB", "", ""]));
    assert_eq!(land["wl"], json!(["", "", "", "", "", "WWW", "", ""]));
    assert_eq!(land["cols"], 200);
    // A later stroke: its tiles over the old ones, the rest kept; the cover and zones replaced.
    shape(
        &mut w,
        0,
        json!({"land": {"h": {"5": "CCC", "7": "DDD"}, "wl": {"5": ""}, "cover": [5000, 1]},
            "zones": []}),
        None,
    )
    .unwrap();
    let land = &w.meta["land"];
    assert_eq!(land["h"], json!(["AAA", "", "", "", "", "CCC", "", "DDD"]));
    assert_eq!(land["wl"], json!(["", "", "", "", "", "", "", ""]));
    assert_eq!(land["cover"], json!([5000, 1]));
    assert_eq!(land["seeded"], "drawn");
    assert_eq!(w.meta["zones"], json!([]));
    // A tile past the land's is refused.
    let err = shape(&mut w, 0, json!({"land": {"h": {"8": "EEE"}}}), None).unwrap_err();
    assert!(err.contains("no tile 8"), "{err}");
    assert_eq!(w.meta["land"]["h"][7], "DDD");
}

#[test]
fn home_edits_undone_keep_what_the_land_changed() {
    let mut w = world();
    let start = funds(&w);
    common::buy(&mut w, "chair", 3, 6);
    shape(&mut w, 250, json!({}), Some(water(in_pond))).unwrap();
    // The chair is taken back: its price comes back, the land's doesn't; the pond stays.
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!(funds(&w), start - 250);
    assert!(wet_now(&w, 28, 7));
    w.apply(Command::Redo { household: 0 }).unwrap();
    assert_eq!(funds(&w), start - 250 - 100);
    assert!(wet_now(&w, 28, 7));
}

#[test]
fn the_save_keeps_the_land_and_the_water() {
    let mut w = world();
    shape(
        &mut w,
        40,
        json!({"land": {"v": 1, "cell": 2, "cols": 24, "rows": 16, "tile": 64,
            "h": {"0": "AAEC"}, "wl": {"0": "BQ"}, "cover": [96, 2], "seeded": "drawn"},
            "zones": [{"kind": "meadow"}]}),
        Some(water(in_pond)),
    )
    .unwrap();
    w.tick_once();
    let back = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(back.meta["land"], w.meta["land"]);
    assert_eq!(back.meta["zones"], w.meta["zones"]);
    let mask = |w: &World| w.travel.file.terrain.as_ref().unwrap().water.clone();
    assert_eq!(mask(&back), Some(water(in_pond)));
    assert!(wet_now(&back, 28, 7));
    assert_eq!(way_across(&back), way_across(&w));
    assert_eq!(funds(&back), funds(&w));
}

#[test]
fn the_shipped_content_prices_the_land() {
    let content = sim_core::Content::from_json(&common::content()).unwrap();
    let catalog: Value = serde_json::from_str(&sim_core::view::catalog_json(&content)).unwrap();
    assert_eq!(
        catalog["build"]["land"],
        json!({"earth": 3.0, "mountain": 0.02, "water": 2.0, "woods": 6.0, "meadow": 1.0,
            "fields": 2.0, "clear": 1.5})
    );
}

#[test]
fn the_page_reads_the_water_mask_and_sends_it_back_changed() {
    let mut w = world();
    // No water yet: all dry, the ground's size.
    let (mask, width, depth) = w.water_mask();
    assert_eq!((width, depth), (W, D));
    assert_eq!(mask, vec![(W * D) as u32, 0]);
    // A pond: the mask the page reads is the one it sent.
    shape(&mut w, 0, json!({}), Some(water(in_pond))).unwrap();
    assert_eq!(w.water_mask().0, water(in_pond));
    let json: Value = serde_json::from_str(&w.water_mask_json()).unwrap();
    assert_eq!(json["water"], json!(water(in_pond)));
    assert_eq!(
        (json["width"].clone(), json["depth"].clone()),
        (json!(W), json!(D))
    );
    // A second pond laid over the mask read back keeps the first: nothing else changes.
    let (now, _, _) = w.water_mask();
    let mut wet: Vec<bool> = Vec::new();
    for pair in now.chunks(2) {
        wet.extend(std::iter::repeat_n(pair[1] != 0, pair[0] as usize));
    }
    let both = water(|x, z| wet[z * W + x] || ((36..44).contains(&x) && (24..30).contains(&z)));
    shape(&mut w, 0, json!({}), Some(both)).unwrap();
    assert!(wet_now(&w, 28, 7) && wet_now(&w, 40, 26));
}

#[test]
fn a_strokes_bridges_merge_into_its_roads() {
    let mut w = world();
    w.meta = json!({"roads": [
        {"points": [[0, 20], [48, 20]], "width": 6, "kind": "street"},
        {"points": [[20, 0], [20, 20]], "width": 6, "kind": "street", "bridge": [2, 8]}
    ]});
    // Water across the street (x 30..36), the deck's tiles (x 28..38, z 15..25) left dry: the street bridged there.
    let wet = water(|x, z| (30..36).contains(&x) && !(15..25).contains(&z));
    shape(
        &mut w,
        0,
        json!({"roads": {"0": {"bridges": [[28, 38]]}}}),
        Some(wet.clone()),
    )
    .unwrap();
    assert_eq!(w.meta["roads"][0]["bridges"], json!([[28, 38]]));
    // The other road and its old deck as they were.
    assert_eq!(w.meta["roads"][1]["bridge"], json!([2, 8]));
    assert!(w.meta["roads"][1].get("bridges").is_none());
    assert!(wet_now(&w, 32, 5) && !wet_now(&w, 32, 20));
    // A second span joins the first (the page sends the road's whole list); none takes them away.
    shape(
        &mut w,
        0,
        json!({"roads": {"0": {"bridges": [[4, 9], [28, 38]]}}}),
        None,
    )
    .unwrap();
    assert_eq!(w.meta["roads"][0]["bridges"], json!([[4, 9], [28, 38]]));
    shape(&mut w, 0, json!({"roads": {"0": {"bridges": []}}}), None).unwrap();
    assert!(w.meta["roads"][0].get("bridges").is_none());
    // A road that isn't there, or spans that aren't, are refused and change nothing.
    let err = shape(
        &mut w,
        0,
        json!({"roads": {"5": {"bridges": [[1, 2]]}}}),
        None,
    )
    .unwrap_err();
    assert!(err.contains("no road 5"), "{err}");
    let err = shape(
        &mut w,
        0,
        json!({"roads": {"0": {"bridges": [[9, 2]]}}}),
        None,
    )
    .unwrap_err();
    assert!(err.contains("spans"), "{err}");
    assert!(w.meta["roads"][0].get("bridges").is_none());
}
