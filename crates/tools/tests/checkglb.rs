//! The gate over every shipped model (`cargo check-glb`): it passes, as it does today.

use ll_tools::checkglb::baseline::{Baseline, Found};
use ll_tools::checkglb::{
    BatchCheck, COUNTED, Paths, gate, gate_with, known_keys, measures, update_baseline,
};
use ll_tools::review::{self, Reference};
use std::collections::BTreeMap;

#[test]
fn every_shipped_model_passes_the_gate() {
    let report = gate(&Paths::repo(), &[]);
    assert_eq!(report.failed, 0, "the gate failed:\n{}", report.text);
    assert!(
        report.checked.len() > 100,
        "only {} models checked:\n{}",
        report.checked.len(),
        report.text
    );
    assert!(
        report
            .text
            .ends_with(&format!("{} models, 0 failed\n", report.checked.len()))
    );
}

#[test]
fn a_key_alone_is_checked_alone() {
    let report = gate(&Paths::repo(), &["model.fridge".to_string()]);
    assert_eq!(report.checked, ["model.fridge"]);
    assert_eq!(report.failed, 0, "{}", report.text);
    assert!(
        report.text.starts_with("ok   model.fridge "),
        "{}",
        report.text
    );
}

#[test]
fn an_unknown_key_fails() {
    let report = gate(&Paths::repo(), &["model.noSuchThing".to_string()]);
    assert_eq!(report.failed, 1);
    assert_eq!(
        report.text,
        "FAIL model.noSuchThing\n       error: the command line names it, but no manifest has a model entry model.noSuchThing\n0 models, 1 failed (1 without a file it can read)\n"
    );
}

#[test]
fn the_known_keys_hold_every_checked_model() {
    let paths = Paths::repo();
    let known = known_keys(&paths).unwrap();
    for key in gate(&paths, &[]).checked {
        assert!(known.contains(&key), "{key}");
    }
}

/// The committed baseline is in the format `--update-baseline` writes, and holds only counted checks (the models'
/// and the built world's) and, for those the gate runs, model keys a build may write.
#[test]
fn the_committed_baseline_is_in_its_format() {
    let paths = Paths::repo();
    let text = std::fs::read_to_string(&paths.baseline).unwrap();
    let baseline = Baseline::parse(&text).unwrap();
    assert_eq!(baseline.text(), text);
    let known = known_keys(&paths).unwrap();
    for check in baseline.checks() {
        // (The built world's are by sample, held by `cargo check-glb --built`: tools/ci-local.sh `built`.)
        if ll_tools::checkglb::built::CHECKS.contains(&check) {
            continue;
        }
        assert!(COUNTED.contains(&check), "{check}");
        // (`reference` is counted by `cargo review sizes`, by object.)
        if check != "reference" {
            for key in baseline.keys(check) {
                assert!(known.contains(key), "{check}: {key}");
            }
        }
    }
}

/// An example batch check, no rule of the gate's: two findings in every model.
fn two_each(files: &[(String, String, Option<String>)]) -> Result<BTreeMap<String, Found>, String> {
    Ok(files
        .iter()
        .map(|(key, _, _)| {
            let text = vec!["one".to_string(), "two".to_string()];
            (key.clone(), Found { count: 2, text })
        })
        .collect())
}

/// An example batch check that finds one in every model.
fn one_each(files: &[(String, String, Option<String>)]) -> Result<BTreeMap<String, Found>, String> {
    Ok(files
        .iter()
        .map(|(key, _, _)| {
            (
                key.clone(),
                Found {
                    count: 1,
                    text: vec!["one".into()],
                },
            )
        })
        .collect())
}

/// An example batch check that can't run.
fn broken(_: &[(String, String, Option<String>)]) -> Result<BTreeMap<String, Found>, String> {
    Err("no validator here".into())
}

/// A counted check run over every file at once, through the public gate: strict for a model the baseline doesn't
/// have, warnings at its count, stale under it until `update_baseline` lowers it.
#[test]
fn a_batch_check_is_judged_by_the_baseline() {
    let dir = std::env::temp_dir().join(format!("ll-checkglb-baseline-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut paths = Paths::repo();
    paths.baseline = dir
        .join("gate-baseline.json")
        .to_string_lossy()
        .into_owned();
    let keys = vec!["model.fridge".to_string()];
    let two: &[(&'static str, BatchCheck)] = &[("validator", two_each)];
    let r = gate_with(&paths, &keys, two);
    assert_eq!(r.failed, 1);
    assert!(
        r.text.contains("\n       error: one\n       error: two\n"),
        "{}",
        r.text
    );
    assert_eq!(r.counts["validator"]["model.fridge"], 2);
    std::fs::write(&paths.baseline, "{\"validator\": {\"model.fridge\": 2}}\n").unwrap();
    let r = gate_with(&paths, &keys, two);
    assert_eq!(r.failed, 0, "{}", r.text);
    assert!(r.text.contains(
        "\n       warning: validator: 2 found, as the baseline has (tools/art/gate-baseline.json)\n       warning: one\n"
    ));
    // Not run (plain `cargo check-glb`): the baseline's entry is kept, and nothing fails.
    let r = gate(&paths, &keys);
    assert_eq!(r.failed, 0, "{}", r.text);
    assert!(!r.counts.contains_key("validator"));
    assert!(update_baseline(&paths, &[], &[]).unwrap().is_empty());
    // Fewer: stale until lowered.
    let one: &[(&'static str, BatchCheck)] = &[("validator", one_each)];
    assert_eq!(gate_with(&paths, &keys, one).failed, 1);
    assert_eq!(
        update_baseline(&paths, &keys, one).unwrap(),
        ["validator: model.fridge 2 -> 1"]
    );
    assert_eq!(gate_with(&paths, &keys, one).failed, 0);
    // More again: never raised.
    assert_eq!(gate_with(&paths, &keys, two).failed, 1);
    assert!(update_baseline(&paths, &keys, two).unwrap().is_empty());
    // A check that can't run fails the gate, and counts nothing.
    let r = gate_with(&paths, &keys, &[("validator", broken)]);
    assert!(
        r.text
            .starts_with("FAIL validator\n       error: no validator here\n"),
        "{}",
        r.text
    );
    assert_eq!(r.failed, 1);
    assert!(!r.counts.contains_key("validator"));
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The sofa bed (the plan's known case, "0.44 against 0.42 m"): its mattress is measured pulled out (`bed_out`),
/// and today the model has it where furniture.json does, 0.44 m, and the seat at 0.44 (furniture.json `seats`, the
/// content's copied there with its `front`): no height
/// warning.
#[test]
fn the_sofa_bed_is_measured_held_open_where_furniture_json_has_it() {
    let paths = Paths::repo();
    let m = measures(&paths, &Default::default()).unwrap();
    let sofa = m.objects["sofaBed"].as_ref().unwrap();
    let of = |what: &str| sofa.heights.iter().find(|h| h.what == what).unwrap();
    let (mattress, seat) = (of("mattress"), of("seat"));
    assert_eq!(
        (
            mattress.expected,
            mattress.source.as_str(),
            mattress.open.as_deref()
        ),
        (0.44, "furniture.json `beds`", Some("bed_out"))
    );
    assert_eq!(
        mattress.at,
        [-0.5, 0.6],
        "the first sleeper's slot, 0.6 m forward"
    );
    assert!(
        (mattress.mesh.unwrap() - 0.44).abs() < 0.002,
        "mattress {:?}",
        mattress.mesh
    );
    assert_eq!(
        (seat.expected, seat.source.as_str(), seat.open.as_deref()),
        (0.44, "furniture.json `seats`", None)
    );
    assert!(
        (seat.mesh.unwrap() - 0.44).abs() < 0.002,
        "seat {:?}",
        seat.mesh
    );
    assert!(mattress.fits() && seat.fits() && seat.disagrees.is_none());
    let report = gate(&paths, &["model.sofaBed".to_string()]);
    assert!(
        !report.text.contains("warning: seat") && !report.text.contains("warning: mattress"),
        "{}",
        report.text
    );
}

/// What `cargo review sizes` measures besides, on the pilot's cases: a crib's mattress at slot 0, its middle (not
/// its changing pad), the top of a workbench and a sink the content gives none, the dining table without its chairs (what is
/// left over its top is the vase's flowers) and the stairs' top step.
#[test]
fn the_review_measures_the_pilots_cases() {
    let paths = Paths::repo();
    let root = ll_tools::checkglb::repo_root();
    let reference = Reference::parse(
        &std::fs::read_to_string(format!("{root}/{}", review::REFERENCE)).unwrap(),
    )
    .unwrap();
    let objects = review::objects(std::path::Path::new(&paths.content)).unwrap();
    let m = measures(&paths, &review::tops(&objects, &reference)).unwrap();
    let of = |id: &str| m.objects[id].as_ref().unwrap();
    let crib = of("crib")
        .heights
        .iter()
        .find(|h| h.what == "mattress")
        .unwrap();
    // The middle: a baby is put at the crib's centre (sim-core `place_at`), not spread across its width, so
    // slot 1's changing pad is not read as a second mattress.
    assert_eq!(crib.at, [0.0, 0.0], "slot 0");
    assert!((crib.mesh.unwrap() - 0.56).abs() < 0.01, "{:?}", crib.mesh);
    let (_, workbench) = of("workbench").top.unwrap();
    // A standing height since G-29 (review M-hobbies-01: it was a desk at 0.78).
    assert!((workbench - 0.90).abs() < 0.01, "{workbench}");
    let (_, sink) = of("sink").top.unwrap();
    // The base sink is a kitchen sink unit since G-11 (reference kind `kitchenSink`): its worktop, not a basin rim.
    assert!((0.85..=1.0).contains(&sink), "{sink}");
    let table = of("diningTable");
    assert!(
        table.table.unwrap() <= table.size[1] + 1e-9 && table.top.is_none(),
        "{:?}",
        table.table
    );
    let (top, steps) = of("stairs").steps.unwrap();
    assert_eq!(((top * 100.0).round(), steps), (280.0, 14));
}

#[test]
fn every_object_is_measured() {
    let m = measures(&Paths::repo(), &Default::default()).unwrap();
    assert!(m.objects.len() > 500, "{}", m.objects.len());
    let unread: Vec<_> = m
        .objects
        .iter()
        .filter_map(|(id, r)| r.as_ref().err().map(|e| format!("{id}: {e}")))
        .collect();
    assert!(unread.is_empty(), "{unread:?}");
    let counter = m.objects["counter"].as_ref().unwrap();
    assert_eq!(counter.model, "model.counter");
    assert!(
        (counter.size[0] - 1.0).abs() < 0.01 && (counter.size[1] - 0.95).abs() < 0.01,
        "{:?}",
        counter.size
    );
}

/// The birds the renderer flies (`packs/birds`, families/birds.py) carry their sizes, and the trees, shrubs and hedges
/// where they sit carry their perches (crates/art/nature/src/trees, families/plants.py): 12-30 a tree, 3-8 a shrub or hedge segment, each
/// three numbers, and their `habitat`. The gate checks both against the files.
#[test]
fn the_birds_and_their_perches_are_in_the_packs() {
    let paths = Paths::repo();
    let pack = |name: &str| -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(format!("{}/packs/{name}/manifest.json", paths.assets))
                .unwrap(),
        )
        .unwrap()
    };
    let (birds, trees, plants) = (pack("birds"), pack("trees"), pack("plants"));
    let mut keys = Vec::new();
    for id in [
        "blueTit",
        "chaffinch",
        "warbler",
        "blackbird",
        "collaredDove",
    ] {
        let key = format!("model.bird.{id}");
        let bird = &birds["entries"][&key]["bird"];
        let (length, span) = (
            bird["length"].as_f64().unwrap(),
            bird["wingspan"].as_f64().unwrap(),
        );
        assert!(length > 0.1 && span > length, "{key}: {bird}");
        assert!(
            bird["perchHeight"].as_f64().is_some() && bird["pitch"].as_f64().is_some(),
            "{key}: {bird}"
        );
        keys.push(key);
    }
    // Each perched model's perches, and what they are on (`habitat`).
    let count = |pack: &serde_json::Value, key: &str, habitat: &str| {
        let entry = &pack["entries"][key];
        let perches = entry["perches"]
            .as_array()
            .unwrap_or_else(|| panic!("{key} has no perches"));
        assert!(
            perches.iter().all(|p| p
                .as_array()
                .is_some_and(|p| p.len() == 3 && p.iter().all(serde_json::Value::is_number))),
            "{key}"
        );
        assert_eq!(entry["habitat"], habitat, "{key}");
        perches.len()
    };
    let tree_keys = [
        ("model.tree", "tree"),
        ("model.tree.maple", "tree"),
        ("model.tree.birch", "tree"),
        ("model.tree.blossom", "tree"),
        ("model.tree.fruit", "tree"),
        ("model.tree.japaneseMaple", "tree"),
        ("model.tree.magnolia", "tree"),
        ("model.tree.willow", "tree"),
        ("model.pine", "conifer"),
        ("model.spruce", "conifer"),
        ("model.cypress", "conifer"),
        ("model.garden.lemonTree", "tree"),
    ];
    for (key, habitat) in tree_keys {
        let n = count(&trees, key, habitat);
        assert!((12..=30).contains(&n), "{key}: {n}");
        keys.push(key.to_string());
    }
    let shrub_keys = [
        ("model.bush", "bush"),
        ("model.bush.small", "bush"),
        ("model.bush.flowering", "bush"),
        ("model.hedge", "hedge"),
        ("model.garden.boxwood", "bush"),
        ("model.garden.topiaryCone", "bush"),
        ("model.garden.hedge", "hedge"),
        ("model.garden.hydrangea", "bush"),
        ("model.garden.rosebush", "bush"),
        ("model.garden.azalea", "bush"),
    ];
    for (key, habitat) in shrub_keys {
        let n = count(&plants, key, habitat);
        assert!((3..=8).contains(&n), "{key}: {n}");
        keys.push(key.to_string());
    }
    let report = gate(&paths, &keys);
    assert_eq!(report.failed, 0, "{}", report.text);
    assert_eq!(report.checked.len(), keys.len());
}
