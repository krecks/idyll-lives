//! The gate over every shipped model (`cargo check-glb`): it passes, as it does today.

use ll_tools::checkglb::{Paths, gate, known_keys};

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
