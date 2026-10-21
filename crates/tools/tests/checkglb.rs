//! The gate over every shipped model (`cargo check-glb`): it passes, as it does today.

use ll_tools::checkglb::{Paths, gate, known_keys, measures};
use ll_tools::review::{self, Reference};

#[test]
fn every_shipped_model_passes_the_gate() {
    let report = gate(&Paths::repo(), &[]);
    assert_eq!(report.failed, 0, "the gate failed:\n{}", report.text);
    assert!(report.checked.len() > 100, "only {} models checked:\n{}", report.checked.len(), report.text);
    assert!(report.text.ends_with(&format!("{} models, 0 failed\n", report.checked.len())));
}

#[test]
fn a_key_alone_is_checked_alone() {
    let report = gate(&Paths::repo(), &["model.fridge".to_string()]);
    assert_eq!(report.checked, ["model.fridge"]);
    assert_eq!(report.failed, 0, "{}", report.text);
    assert!(report.text.starts_with("ok   model.fridge "), "{}", report.text);
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
    assert_eq!((mattress.expected, mattress.source.as_str(), mattress.open.as_deref()), (0.44, "furniture.json `beds`", Some("bed_out")));
    assert_eq!(mattress.at, [-0.5, 0.6], "the first sleeper's slot, 0.6 m forward");
    assert!((mattress.mesh.unwrap() - 0.44).abs() < 0.002, "mattress {:?}", mattress.mesh);
    assert_eq!((seat.expected, seat.source.as_str(), seat.open.as_deref()), (0.44, "furniture.json `seats`", None));
    assert!((seat.mesh.unwrap() - 0.44).abs() < 0.002, "seat {:?}", seat.mesh);
    assert!(mattress.fits() && seat.fits() && seat.disagrees.is_none());
    let report = gate(&paths, &["model.sofaBed".to_string()]);
    assert!(!report.text.contains("warning: seat") && !report.text.contains("warning: mattress"), "{}", report.text);
}

/// What `cargo review sizes` measures besides, on the pilot's cases: a crib's mattress at slot 0 (not its changing
/// pad), the top of a workbench and a sink the content gives none, the dining table without its chairs (what is
/// left over its top is the vase's flowers) and the stairs' top step.
#[test]
fn the_review_measures_the_pilots_cases() {
    let paths = Paths::repo();
    let root = ll_tools::checkglb::repo_root();
    let reference = Reference::parse(&std::fs::read_to_string(format!("{root}/{}", review::REFERENCE)).unwrap()).unwrap();
    let objects = review::objects(std::path::Path::new(&paths.content)).unwrap();
    let m = measures(&paths, &review::tops(&objects, &reference)).unwrap();
    let of = |id: &str| m.objects[id].as_ref().unwrap();
    let crib = of("crib").heights.iter().find(|h| h.what == "mattress").unwrap();
    assert_eq!(crib.at[0], -0.5, "slot 0");
    assert!((crib.mesh.unwrap() - 0.56).abs() < 0.01, "{:?}", crib.mesh);
    let (_, workbench) = of("workbench").top.unwrap();
    // A standing height since G-29 (review M-hobbies-01: it was a desk at 0.78).
    assert!((workbench - 0.90).abs() < 0.01, "{workbench}");
    let (_, sink) = of("sink").top.unwrap();
    // The base sink is a kitchen sink unit since G-11 (reference kind `kitchenSink`): its worktop, not a basin rim.
    assert!((0.85..=1.0).contains(&sink), "{sink}");
    let table = of("diningTable");
    assert!(table.table.unwrap() <= table.size[1] + 1e-9 && table.top.is_none(), "{:?}", table.table);
    let (top, steps) = of("stairs").steps.unwrap();
    assert_eq!(((top * 100.0).round(), steps), (280.0, 14));
}

#[test]
fn every_object_is_measured() {
    let m = measures(&Paths::repo(), &Default::default()).unwrap();
    assert!(m.objects.len() > 500, "{}", m.objects.len());
    let unread: Vec<_> = m.objects.iter().filter_map(|(id, r)| r.as_ref().err().map(|e| format!("{id}: {e}"))).collect();
    assert!(unread.is_empty(), "{unread:?}");
    let counter = m.objects["counter"].as_ref().unwrap();
    assert_eq!(counter.model, "model.counter");
    assert!((counter.size[0] - 1.0).abs() < 0.01 && (counter.size[1] - 0.95).abs() < 0.01, "{:?}", counter.size);
}
