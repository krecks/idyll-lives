//! Saves written by released builds load and play on with the shipped content: day 5 of a town
//! saved by v0.1.0 (save version 17), and by the build that was live on the site after it (d7c506e,
//! version 19); the benchmark's village played two days by v0.4.2 (version 32, the last before 33's
//! migration); a Normal open world saved by 0.7.0 at 20 ticks a second at rush hour on day 2, cars
//! and a taxi on the road (version 35, the last before the 60-tick clock). Add one for each release
//! (`tests/saves/`).

use crate::common;

use sim_core::World;
use sim_core::world::TICKS_PER_DAY;

fn play_a_day(file: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/saves")
        .join(file);
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{file}: {e}"));
    let version = serde_json::from_str::<serde_json::Value>(&json).unwrap()["version"].as_u64();
    let content = common::content();
    let mut w = World::from_save_json(&content, &json)
        .unwrap_or_else(|e| panic!("{file} (v{version:?}): {e}"));
    assert!(
        w.sims.iter().filter(|s| s.here()).count() > 10,
        "{file}: the town's residents are there"
    );
    assert_eq!(w.seed, 0, "{file}: older saves don't know their seed");
    common::check(&w);
    // (Everyone in full detail.)
    w.lod = false;
    let end = w.tick + TICKS_PER_DAY;
    while w.tick < end {
        w.tick_once();
        if w.tick.is_multiple_of(600) {
            common::check(&w);
        }
    }
    common::check(&w);
    let again = World::from_save_json(&content, &w.save_json())
        .unwrap_or_else(|e| panic!("{file}, saved again: {e}"));
    common::check(&again);
    assert_eq!(
        (
            again.tick,
            again.sims.len(),
            again.objects.len(),
            again.households.len()
        ),
        (w.tick, w.sims.len(), w.objects.len(), w.households.len())
    );
}

#[test]
fn a_town_saved_by_v0_1_0_plays_on() {
    play_a_day("v010_day5.json");
}

#[test]
fn a_town_saved_by_the_live_build_after_v0_1_0_plays_on() {
    play_a_day("live_day5.json");
}

#[test]
fn a_village_saved_by_v0_4_2_plays_on() {
    play_a_day("v042_village.json");
}

#[test]
fn v0_4_2s_chess_tables_come_through_at_their_new_size() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/saves/v042_village.json");
    let content = common::content();
    let w = World::from_save_json(&content, &std::fs::read_to_string(path).unwrap()).unwrap();
    let def = w
        .content
        .objects
        .iter()
        .position(|d| d.id == "chessTable")
        .unwrap();
    // (Both had room to grow to 2 x 1: none was sold back.)
    assert_eq!(w.objects.iter().filter(|o| o.def == def).count(), 2);
}

#[test]
fn a_town_saved_as_v34_plays_on() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/saves/v034_town.json");
    let content = common::content();
    let mut w = World::from_save_json(&content, &std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(w.seed, 11, "a v34 save knows its seed");
    common::check(&w);
    w.lod = false;
    let end = w.tick + TICKS_PER_DAY;
    while w.tick < end {
        w.tick_once();
        if w.tick.is_multiple_of(600) {
            common::check(&w);
        }
    }
    common::reload(&w, &content);
}

/// A v34 save (dense edge strings, chemistry per pair, no open-world fields) loads, saves as
/// v35 (run-length edges, chemistry packed, still a cosy town) and loads back the same.
#[test]
fn a_v34_save_comes_through_v35_unchanged() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/saves/v034_town.json");
    let old_json = std::fs::read_to_string(path).unwrap();
    let old: serde_json::Value = serde_json::from_str(&old_json).unwrap();
    assert_eq!(old["version"], 34);
    let content = common::content();
    let w = World::from_save_json(&content, &old_json).unwrap();
    // A cosy town, as it always was: no open-world fields, no road network (whatever roads its
    // meta has), every exit a road out.
    assert_eq!(w.travel.size, sim_core::travel::WorldSize::Cosy);
    assert!(!w.travel.open && w.travel.graph.is_empty());
    assert!(w.travel.file.is_empty());
    let json = w.save_json();
    let new: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(new["version"], sim_core::save::SAVE_VERSION);
    assert!(new["lot"]["hEdges"].as_str().unwrap().starts_with('~'));
    assert!(new["chemistry"].as_str().is_some_and(|c| !c.is_empty()));
    assert!(
        new["relationships"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r.get("chemistry").is_none())
    );
    assert!(
        json.len() < old_json.len(),
        "{} vs {}",
        json.len(),
        old_json.len()
    );
    let back = World::from_save_json(&content, &json).unwrap();
    assert_eq!(back.lot, w.lot);
    assert!(new.get("openWorld").is_none());
    assert!(!back.travel.open && back.travel.file.is_empty());
    let n = w.sims.len();
    for a in 0..n {
        for b in 0..n {
            assert_eq!(back.relationships.get(a, b), w.relationships.get(a, b));
            assert_eq!(
                back.relationships.chemistry(a, b).to_bits(),
                w.relationships.chemistry(a, b).to_bits(),
                "{a}, {b}"
            );
        }
    }
    assert_eq!(back.dirt, w.dirt);
    assert_eq!(back.save_json(), json, "saves the same again");
}

/// A game saved while the town had buses (0.7.0's development builds: its bus stops among the
/// exits, `openWorld.stops` saying which, the content's `bus` timetable) loads with no bus: the
/// stops are no ways out, nobody sets off for one, and it saves without them.
#[test]
fn a_game_saved_while_the_town_had_buses_loads_without_them() {
    let content = common::content();
    let h = common::hamlets::two_hamlets(&content, 21);
    let w = World::from_json(&content, &common::hamlets::with_cars(&h), 21).unwrap();
    let roads_out = w.exits.clone();
    assert!(!roads_out.is_empty());
    let mut save: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
    // (A stop by the first street's middle, between the roads out, and the bus's own fields.)
    let street = &w.travel.graph.roads[0];
    let at = street.at(street.cum.last().copied().unwrap_or(0.0) / 2.0);
    let exits = save["exits"].as_array_mut().unwrap();
    exits.insert(0, serde_json::json!([at[0], at[1] + 2.5]));
    let stop = 0;
    save["openWorld"]["stops"] = serde_json::json!([stop]);
    save["bus"] = serde_json::json!({"every": 20, "first": 6, "last": 23});
    let mut c: serde_json::Value = serde_json::from_str(&content).unwrap();
    c["bus"] = serde_json::json!({"every": 20, "first": 6, "last": 23});
    let mut loaded = World::from_save_json(&c.to_string(), &save.to_string()).unwrap();
    assert_eq!(loaded.exits, roads_out, "the stop is no way out");
    let again: serde_json::Value = serde_json::from_str(&loaded.save_json()).unwrap();
    assert!(again["openWorld"].get("stops").is_none());
    assert_eq!(again["exits"].as_array().unwrap().len(), roads_out.len());
    // (And plays on: off to work by the roads out.)
    loaded.autonomy = true;
    let end = loaded.tick + TICKS_PER_DAY;
    while loaded.tick < end {
        loaded.tick_once();
    }
    common::check(&loaded);
}

/// 0.7.0's Normal world at 07:40 on day 2, saved at 20 ticks a second and a 20-tick game minute,
/// plays on from the same minute with its cars and the taxi still on their way.
#[test]
fn an_open_world_saved_by_0_7_0_plays_on() {
    use sim_core::clock;
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/saves/v035_normal.json");
    let json = std::fs::read_to_string(path).unwrap();
    let old: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(old["version"], 35);
    let content = common::content();
    let mut w = World::from_save_json(&content, &json).unwrap();
    assert_eq!(
        (clock::day(w.tick), clock::minute_of_day(w.tick)),
        (2, 460.0)
    );
    assert!(w.travel.open, "an open world");
    assert!(
        w.cars
            .iter()
            .any(|c| !matches!(c.state, sim_core::cars::CarState::Parked)),
        "cars on the road"
    );
    assert!(!w.taxis.runs.is_empty(), "a taxi on its way");
    common::check(&w);
    w.lod = false;
    let end = w.tick + TICKS_PER_DAY;
    while w.tick < end {
        w.tick_once();
        if w.tick.is_multiple_of(sim_core::TICKS_PER_MINUTE * 30) {
            common::check(&w);
        }
    }
    common::reload(&w, &content);
}
