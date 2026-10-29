//! Rough per-tick timing for the shipped content: `cargo run --release --example tickbench`.

use std::time::Instant;

use sim_core::World;

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../web/public/content/");
const LOT: &str = include_str!("../../../web/public/content/lots/starter.json");

/// base.json plus every file in its `"include"` list, merged the way the game loads them.
fn content() -> String {
    let read = |name: &str| {
        std::fs::read_to_string(format!("{DIR}{name}")).unwrap_or_else(|e| panic!("{name}: {e}"))
    };
    let base = read("base.json");
    let includes: Vec<String> = serde_json::from_value(
        serde_json::from_str::<serde_json::Value>(&base).unwrap()["include"].clone(),
    )
    .unwrap();
    let parts: Vec<String> = includes.iter().map(|n| read(n)).collect();
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
    sim_core::merge_content(&base, &parts).unwrap_or_else(|e| panic!("{e}"))
}

fn main() {
    let mut lot: serde_json::Value = serde_json::from_str(LOT).unwrap();
    let names = ["Ada", "Bo", "Cy", "Di"];
    lot["sims"] = names
        .iter()
        .enumerate()
        .map(|(i, n)| serde_json::json!({ "name": n, "traits": ["cheerful"], "x": 8.5 + i as f32, "z": 3.0 }))
        .collect();
    let content: serde_json::Value = serde_json::from_str(&content()).unwrap();
    for (label, socials) in [("with socials", true), ("without socials", false)] {
        let mut c = content.clone();
        if !socials {
            for s in c["socials"].as_array_mut().unwrap() {
                s["autonomous"] = false.into();
            }
        }
        let mut world = World::from_json(&c.to_string(), &lot.to_string(), 1).unwrap();
        let ticks = 24 * 60 * sim_core::TICKS_PER_SECOND;
        let start = Instant::now();
        for _ in 0..ticks {
            world.tick_once();
        }
        let events = world.events.iter().count();
        println!(
            "{label}: {:?}/tick, {events} social events",
            start.elapsed() / ticks
        );
    }
}
