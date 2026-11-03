//! The structure by rectangle (`view::structure_rect_json`) and by chunk
//! (`view::chunk_fingerprints`), for views that stream the town: the chunks' rectangles together
//! give what the whole structure gives, and placing a thing changes only its chunk's fingerprint.

use crate::common;

use serde_json::Value;
use sim_core::World;
use sim_core::lot::CHUNK;
use sim_core::view::{chunk_fingerprints, structure_json, structure_rect_json};

/// The entries of `key` in a structure, as sorted JSON strings.
fn entries(v: &Value, key: &str) -> Vec<String> {
    let mut out: Vec<String> = v[key]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|e| e.to_string())
        .collect();
    out.sort();
    out
}

#[test]
fn chunks_together_are_the_whole_structure() {
    let content = common::content();
    let w = World::from_json(&content, &common::town(&content, 4, 2), 4).unwrap();
    let whole: Value = serde_json::from_str(&structure_json(&w)).unwrap();
    let (cols, rows) = w.lot.chunk_grid();
    let mut parts: Vec<Value> = Vec::new();
    for cz in 0..rows as i32 {
        for cx in 0..cols as i32 {
            let s = CHUNK as i32;
            let rect = [cx * s, cz * s, cx * s + s, cz * s + s];
            parts.push(serde_json::from_str(&structure_rect_json(&w, rect)).unwrap());
        }
    }
    for key in [
        "walls",
        "openings",
        "fences",
        "diagonals",
        "floors",
        "objects",
    ] {
        let mut got: Vec<String> = parts.iter().flat_map(|p| entries(p, key)).collect();
        got.sort();
        assert_eq!(got, entries(&whole, key), "{key}");
    }
}

#[test]
fn placing_a_thing_changes_only_its_chunk() {
    let content = common::content();
    let mut w = World::from_json(&content, &common::town(&content, 4, 2), 4).unwrap();
    let (cols, rows, before) = chunk_fingerprints(&w);
    assert_eq!(before.len(), cols * rows);
    let home = &w.plots[w.households[0].plot.unwrap() as usize];
    let spots: Vec<(i32, i32)> = (home.z..home.z + home.d)
        .flat_map(|z| (home.x..home.x + home.w).map(move |x| (x, z)))
        .collect();
    let (x, z) = spots
        .into_iter()
        .find(|&(x, z)| {
            w.apply(sim_core::Command::Buy {
                household: 0,
                object: "plant".into(),
                at: Some([x, z, 0]),
                colors: Default::default(),
                turn: None,
                offset: None,
                size: None,
                tall: None,
            })
            .is_ok()
        })
        .expect("room for a plant");
    let (_, _, after) = chunk_fingerprints(&w);
    let changed: Vec<usize> = (0..after.len())
        .filter(|&c| after[c] != before[c])
        .collect();
    assert_eq!(changed, vec![w.lot.chunk_at(x, z)]);
}
