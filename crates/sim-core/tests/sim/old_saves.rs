//! Saves written by released builds load and play on with the shipped content: day 5 of a town
//! saved by v0.1.0 (save version 17), and by the build that was live on the site after it (d7c506e,
//! version 19); the benchmark's village played two days by v0.4.2 (version 32, the last before 33's
//! migration). Add one for each release (`tests/saves/`).

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
