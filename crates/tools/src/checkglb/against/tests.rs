use super::*;
use crate::checkglb::Finishes;
use crate::checkglb::py::join;
use std::collections::{HashMap, HashSet};

#[test]
fn findings_that_differ_in_their_numbers_alone_are_one() {
    assert_eq!(
        masked("4321 triangles, aim for 4000 (1.25 m)"),
        "# triangles, aim for # (# m)"
    );
    assert_eq!(
        masked("wing.L is saved at scale 0."),
        "wing.L is saved at scale #"
    );
    let old = vec!["4 draw calls, aim for 3 at most".to_string(), "a".into()];
    let new = vec![
        "5 draw calls, aim for 3 at most".to_string(),
        "b".into(),
        "a".into(),
        "a".into(),
    ];
    assert_eq!(newly(&old, &new), [&"b".to_string(), &"a".to_string()]);
    assert!(newly(&new, &old).is_empty());
}

/// The draw calls are read from the gate's own line: the coin's, as the gate writes it today.
#[test]
fn the_draw_calls_are_the_gate_s() {
    assert_eq!(
        draws(
            "  305.0 KB   7616 tris 4 draws 26 materials  1.89 x 1.90 x 0.92 m  3 moving parts, 2 clips"
        ),
        Some(4)
    );
    assert_eq!(draws(""), None);
    let fin = Finishes {
        plain: HashMap::new(),
        see_through: HashSet::new(),
        textures: HashMap::new(),
    };
    let path = join(
        &super::super::repo_root(),
        "web/public/assets/models/held/coin.glb",
    );
    let entry = super::super::json::loads_text(br#"{"type": "model", "url": "coin.glb"}"#).unwrap();
    let baseline = super::super::baseline::Baseline::default();
    let counting = super::super::Counting {
        baseline: &baseline,
        file: &[],
        batch: std::collections::BTreeMap::new(),
    };
    let checked = super::super::check(
        "tool.coin",
        &fin,
        &entry,
        &path,
        None,
        None,
        None,
        None,
        &counting,
    )
    .unwrap();
    assert_eq!(draws(&checked.summary), Some(1), "{}", checked.summary);
}

#[test]
fn a_family_is_a_folder_of_models_or_packs_however_it_is_written() {
    let j = |text: &str| super::super::json::loads_text(text.as_bytes()).unwrap();
    assert_eq!(
        families(
            &j(r#"{"url": "../../models/garden/gnome.glb"}"#),
            "/a/assets/packs/gardenDecor/manifest.json"
        ),
        ["garden", "gardenDecor"]
    );
    assert!(
        families(
            &j(r#"{"url": "models/rock.glb"}"#),
            "/a/assets/manifest.json"
        )
        .is_empty()
    );
    for (written, folder) in [
        ("garden_decor", "gardenDecor"),
        ("gloomy", "pack_gloomy"),
        ("pack_gloomy", "pack_gloomy"),
    ] {
        assert_eq!(family_name(written), family_name(folder));
    }
    assert_ne!(family_name("kitchen"), family_name("kitchen_cooking"));
    assert_eq!(family_name("pack"), "pack");
}

#[test]
fn two_values_differ_where_their_paths_say() {
    let j = |text: &str| super::super::json::loads_text(text.as_bytes()).unwrap();
    let differ = |a: &str, b: &str| {
        let mut out = Vec::new();
        json_differ(Some(&j(a)), Some(&j(b)), "", &mut out);
        out
    };
    assert!(differ(r#"{"a": 1, "b": [1, 2.0]}"#, r#"{"b": [1.0, 2], "a": 1.0}"#).is_empty());
    assert_eq!(
        differ(
            r#"{"a": {"x": 1, "y": [1, 2]}, "gone": true}"#,
            r#"{"a": {"x": 2, "y": [1, 3]}, "new": "n"}"#
        ),
        [
            "a.x: 1.0 is now 2.0",
            "a.y[1]: 2.0 is now 3.0",
            "gone (true) is gone",
            "new (\"n\") is new"
        ]
    );
    assert_eq!(
        differ("[1, 2]", "[1, 2, 3]"),
        [": [1.0,2.0] is now [1.0,2.0,3.0]"]
    );
}

#[test]
fn every_metric_has_its_name_once_and_fruit_is_fitted() {
    let names: HashSet<&str> = METRICS.iter().map(|(m, _)| *m).collect();
    assert_eq!(names.len(), METRICS.len());
    assert!(fruit("fruit2") && fruit("apple_blush") && fruit("lemon") && !fruit("leaf"));
}
