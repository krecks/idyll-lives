use super::*;

fn row(id: &str, outside: usize, problem: Option<&str>) -> SizeRow {
    SizeRow {
        id: id.into(),
        outside: (0..outside).map(|i| format!("finding {i}")).collect(),
        problem: problem.map(String::from),
        ..SizeRow::default()
    }
}

fn base(text: &str) -> Baseline {
    Baseline::parse(text).unwrap()
}

#[test]
fn check_judges_each_measured_row_by_the_baseline() {
    let b = base(r#"{"reference": {"a": 2, "c": 3, "gone": 1}, "edges": {"model.x": 4}}"#);
    let rows = [
        row("a", 2, None),
        row("b", 0, None),
        row("c", 0, Some("unmeasured: cannot read it")),
    ];
    assert_eq!(
        counts(&rows),
        BTreeMap::from([("a".to_string(), 2), ("b".to_string(), 0)])
    );
    // At its count: held; none and none held: clean; unmeasured: not judged; an id no row has: stale.
    assert_eq!(
        judged(&rows, &b),
        [
            "`gone`: in the baseline, but no object or model the review measured: take it out (`cargo review sizes --update-baseline`)"
        ]
    );
    let rows = [
        row("a", 3, None),
        row("b", 1, None),
        row("c", 0, Some("unmeasured: x")),
        row("gone", 0, None),
    ];
    let errors = judged(&rows, &b);
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert!(errors[0].starts_with(
        "`a`: 3 outside its kind's range, 2 in the baseline: fix the new ones: finding 0; "
    ));
    assert!(
        errors[1].starts_with("`b`: 1 outside its kind's range, none in the baseline: finding 0")
    );
    assert!(
        errors[2].starts_with("`gone`: 0 outside its kind's range, 1 in the baseline: lower it")
    );
}

#[test]
fn update_only_lowers_and_seed_only_starts() {
    let b = base(r#"{"reference": {"a": 2, "c": 3, "gone": 1}, "edges": {"model.x": 4}}"#);
    let rows = [
        row("a", 1, None),
        row("c", 0, Some("unmeasured: x")),
        row("new", 5, None),
    ];
    let (low, changes) = lowered(&rows, &b);
    // a lowered, gone out, c (unmeasured) kept, new not added, the gate's section untouched.
    assert_eq!(
        low,
        base(r#"{"reference": {"a": 1, "c": 3}, "edges": {"model.x": 4}}"#)
    );
    assert_eq!(
        changes,
        ["reference: a 2 -> 1", "reference: gone out (was 1)"]
    );
    assert!(seeded(&rows, &b).unwrap_err().contains("--update-baseline"));
    let start = seeded(&rows, &base(r#"{"edges": {"model.x": 4}}"#)).unwrap();
    assert_eq!(
        start,
        base(r#"{"edges": {"model.x": 4}, "reference": {"a": 1, "new": 5}}"#)
    );
    // Nothing found: no section (every row strict).
    let clean = seeded(&[row("a", 0, None)], &Baseline::default()).unwrap();
    assert_eq!(clean, Baseline::default());
}

#[test]
fn the_shipped_reference_names_its_sources_and_its_species_are_species() {
    let root = checkglb::repo_root();
    let text = std::fs::read_to_string(format!("{root}/{REFERENCE}")).unwrap();
    let r = Reference::parse(&text).unwrap();
    for (name, k) in &r.kinds {
        for s in &k.sources {
            assert!(r.sources.contains_key(s), "{name}: no source `{s}`");
        }
        if let Some(sp) = &k.species {
            assert!(sp.problems().is_empty(), "{name}: {:?}", sp.problems());
            assert!(k.h.is_some(), "{name}: a species' kind has its height");
        }
    }
    for (id, kind) in &r.ids {
        assert!(r.kinds.contains_key(kind), "{id}: no kind `{kind}`");
    }
    // The game's twelve trees are a species each.
    for id in [
        "tree",
        "garden.maple",
        "garden.birch",
        "garden.cherry",
        "garden.apple",
        "garden.japaneseMaple",
        "garden.magnolia",
        "garden.willow",
        "pine",
        "garden.spruce",
        "garden.cypress",
        "garden.lemonTree",
    ] {
        let kind = r.ids.get(id).and_then(|k| r.kind(k));
        assert!(kind.is_some_and(|k| k.species.is_some()), "{id}");
    }
}
