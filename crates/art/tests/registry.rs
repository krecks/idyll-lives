//! Which families `cargo art` builds (`ll_art::pick`), and `--centres`: every family's models' centres, and no
//! others.
#[path = "../../artkit/tests/common/cypress.rs"]
mod cypress;

use ll_art::{centres_of, pick, registry};
use ll_tools::art::write_centres;
use ll_tools::checkglb::{Paths, read_centres};

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| n.to_string()).collect()
}

#[test]
fn every_family_of_the_registry_builds_and_is_named_once() {
    let registry = registry();
    let mut seen: Vec<&str> = registry.iter().map(|(n, _)| *n).collect();
    for (name, family) in &registry {
        assert!(!family.models.is_empty(), "{name}: no models");
    }
    seen.sort_unstable();
    assert!(seen.windows(2).all(|w| w[0] != w[1]), "{seen:?}");
    assert!(seen.contains(&"trees") && seen.contains(&"kitchen"));
}

#[test]
fn the_families_asked_for_are_built_in_the_order_asked() {
    let registry = registry();
    let picked = pick(&names(&["plants", "trees"]), &registry, false).unwrap();
    let got: Vec<&str> = picked.iter().map(|f| f.name).collect();
    assert_eq!(got, ["plants", "trees"]);
    // --all: every one, and the names given with it say nothing
    let all = pick(&names(&["nosuch"]), &registry, true).unwrap();
    assert_eq!(all.len(), registry.len());
}

#[test]
fn a_name_the_registry_does_not_have_is_no_family() {
    let registry = registry();
    let err = pick(&names(&["trees", "nosuch"]), &registry, false).unwrap_err();
    assert!(
        err.starts_with("no family \"nosuch\" (trees, plants, "),
        "{err}"
    );
    let err = pick(&[], &registry, false).unwrap_err();
    assert!(err.starts_with("name a family (trees, plants, "), "{err}");
}

/// `--centres` writes the file anew from the families' models: a key no family builds goes.
#[test]
fn centres_are_every_family_s_models_and_no_others() {
    let root = std::env::temp_dir().join(format!("ll-art-centres-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = Paths::at(&root.to_string_lossy());
    std::fs::create_dir_all(std::path::Path::new(&paths.centres).parent().unwrap()).unwrap();
    std::fs::write(
        &paths.centres,
        "{\n  \"model.cypress\": [9.0, 9.0],\n  \"model.gone\": [0.1, 0.2]\n}\n",
    )
    .unwrap();
    let mut family = cypress::family();
    family.models[0].centre = [0.25, -0.5];
    write_centres(&paths, &centres_of(&[&family]), true).unwrap();
    assert_eq!(
        std::fs::read_to_string(&paths.centres).unwrap(),
        "{\n  \"model.cypress\": [0.25, -0.5]\n}\n"
    );
    assert_eq!(
        read_centres(&paths.centres)
            .unwrap()
            .entries()
            .unwrap()
            .len(),
        1
    );
    let _ = std::fs::remove_dir_all(&root);
}
