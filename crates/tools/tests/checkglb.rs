//! The gate over every shipped model (`cargo check-glb`): it passes, as it does today.

use ll_tools::checkglb::{Paths, gate, known_keys, measures};

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
/// and today the model has it where furniture.json does, 0.44 m, and the seat at the content's 0.44: no height
/// warning.
#[test]
fn the_sofa_bed_is_measured_held_open_where_furniture_json_has_it() {
    let paths = Paths::repo();
    let m = measures(&paths).unwrap();
    let sofa = m.objects["sofaBed"].as_ref().unwrap();
    let of = |what: &str| sofa.heights.iter().find(|h| h.what == what).unwrap();
    let (mattress, seat) = (of("mattress"), of("seat"));
    assert_eq!((mattress.expected, mattress.source.as_str(), mattress.open.as_deref()), (0.44, "furniture.json `beds`", Some("bed_out")));
    assert_eq!(mattress.at, [-0.5, 0.6], "the first sleeper's slot, 0.6 m forward");
    assert!((mattress.mesh.unwrap() - 0.44).abs() < 0.002, "mattress {:?}", mattress.mesh);
    assert_eq!((seat.expected, seat.source.as_str(), seat.open.as_deref()), (0.44, "content `seat`", None));
    assert!((seat.mesh.unwrap() - 0.44).abs() < 0.002, "seat {:?}", seat.mesh);
    assert!(mattress.fits() && seat.fits() && seat.disagrees.is_none());
    let report = gate(&paths, &["model.sofaBed".to_string()]);
    assert!(!report.text.contains("warning: seat") && !report.text.contains("warning: mattress"), "{}", report.text);
}

#[test]
fn every_object_is_measured() {
    let m = measures(&Paths::repo()).unwrap();
    assert!(m.objects.len() > 500, "{}", m.objects.len());
    let unread: Vec<_> = m.objects.iter().filter_map(|(id, r)| r.as_ref().err().map(|e| format!("{id}: {e}"))).collect();
    assert!(unread.is_empty(), "{unread:?}");
    let counter = m.objects["counter"].as_ref().unwrap();
    assert_eq!(counter.model, "model.counter");
    assert!((counter.size[0] - 1.0).abs() < 0.01 && (counter.size[1] - 0.95).abs() < 0.01, "{:?}", counter.size);
}
