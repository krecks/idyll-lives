//! Today's towns stay exactly as they were through changes to how the grid is stored and
//! searched (chunked tiles, the stair index, bounded searches, incremental reach): the saved
//! towns in `tests/saves/` and a generated town give byte-identical `structure_json` when
//! loaded, and the same residents in the same places with the same needs, and the same
//! structure, after two game hours at full detail. The fingerprints are in
//! `tests/grid_identity.txt`, written by the build before the change (`BLESS=1` writes them
//! again; do that only for a change that is meant to move someone).

use crate::common;

use sim_core::World;
use sim_core::view::structure_json;

/// FNV-1a, 64 bits.
fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Who is where with what needs, to the bit.
fn residents(w: &World) -> u64 {
    let mut bytes = Vec::new();
    for s in &w.sims {
        for v in s.pos.iter().chain(&s.needs) {
            bytes.extend_from_slice(&v.to_bits().to_le_bytes());
        }
    }
    hash(&bytes)
}

/// The structure's hash, but for the lit junctions' and their poles' layout for the renderer
/// (`junctions`, `lightPoles`): ways out and yaws from the platform's trigonometry, whose last bit
/// differs between macOS and Linux (CI), so the same build hashed two ways. The game runs on the
/// WASM's own maths; the layout has its own tests (`traffic`, `web/tests/junctions.test.ts`).
fn structure_hash(w: &World) -> u64 {
    let mut v: serde_json::Value = serde_json::from_str(&structure_json(w)).expect("structure");
    if let Some(m) = v.as_object_mut() {
        m.remove("junctions");
        m.remove("lightPoles");
    }
    hash(v.to_string().as_bytes())
}

fn fingerprint(name: &str, mut w: World) -> String {
    let loaded = structure_hash(&w);
    w.lod = false;
    for _ in 0..2 * 60 * sim_core::TICKS_PER_MINUTE {
        w.tick_once();
    }
    let later = structure_hash(&w);
    format!("{name} {loaded:016x} {later:016x} {:016x}", residents(&w))
}

#[test]
fn todays_towns_stay_identical() {
    let content = common::content();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut names: Vec<_> = std::fs::read_dir(dir.join("saves"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".json"))
        .collect();
    names.sort();
    let mut lines = Vec::new();
    for name in &names {
        let json = std::fs::read_to_string(dir.join("saves").join(name)).unwrap();
        let w = World::from_save_json(&content, &json).unwrap_or_else(|e| panic!("{name}: {e}"));
        lines.push(fingerprint(name, w));
    }
    let town = common::town(&content, 3, 2);
    let w = World::from_json(&content, &town, 3).unwrap();
    lines.push(fingerprint("town", w));
    let got = lines.join("\n") + "\n";
    let path = dir.join("grid_identity.txt");
    if std::env::var_os("BLESS").is_some() {
        std::fs::write(&path, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).expect("grid_identity.txt (BLESS=1 writes it)");
    assert_eq!(got, want, "a town changed");
}
