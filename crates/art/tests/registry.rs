//! Which families `cargo art` builds (`ll_art::pick`), and `--centres`: every family's models' centres, and no
//! others.
#[path = "../../artkit/tests/all/common/cypress.rs"]
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

/// A family dropped from its group's list would fall out of `--all`, `--check` and `--centres` with no error: every
/// pack on disk is some family's, and every group has families.
#[test]
fn every_pack_on_disk_is_a_family_s() {
    for (group, families) in [
        ("nature", art_nature::families()),
        ("wet", art_wet::families()),
        ("rooms", art_rooms::families()),
        ("packs", art_packs::families()),
        ("outside", art_outside::families()),
        ("things", art_things::families()),
    ] {
        assert!(!families.is_empty(), "{group}: no families");
    }
    let registry = registry();
    let packs = format!("{}/packs", Paths::repo().assets);
    for e in std::fs::read_dir(&packs).unwrap().flatten() {
        let pack = e.file_name().to_string_lossy().into_owned();
        if e.path().join("manifest.json").exists() {
            assert!(
                registry.iter().any(|(_, f)| f.pack == pack),
                "packs/{pack}/manifest.json: no family of the registry builds it"
            );
        }
    }
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

/// Every group's crate folder under crates/art (`crates/art/<group>/src/<family>/`).
const GROUPS: [&str; 6] = ["nature", "wet", "rooms", "packs", "outside", "things"];

/// A family is a unit with its own notes, and imports only artkit and its group's `src/common.rs`: never another
/// family of its group (`crate::<family>`, also `$crate::<family>` in a macro of another's).
#[test]
fn every_family_has_notes_and_imports_no_other_family() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut faults = Vec::new();
    for group in GROUPS {
        if !root.join(group).join("NOTES.md").is_file() {
            faults.push(format!("{group}: no NOTES.md"));
        }
        let src = root.join(group).join("src");
        let mut families: Vec<String> = std::fs::read_dir(&src)
            .unwrap_or_else(|e| panic!("{}: {e}", src.display()))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        families.sort();
        for family in &families {
            if !src.join(family).join("NOTES.md").is_file() {
                faults.push(format!("{group}/{family}: no NOTES.md"));
            }
            let mut files = vec![src.join(family)];
            while let Some(path) = files.pop() {
                if path.is_dir() {
                    files.extend(std::fs::read_dir(&path).unwrap().map(|e| e.unwrap().path()));
                } else if path.extension().is_some_and(|x| x == "rs") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    for other in families.iter().filter(|o| *o != family) {
                        let needle = format!("crate::{other}");
                        let imported = text.match_indices(&needle).any(|(i, _)| {
                            !text[i + needle.len()..]
                                .starts_with(|c: char| c.is_alphanumeric() || c == '_')
                        });
                        if imported {
                            faults.push(format!(
                                "{}: imports the family {other}",
                                path.strip_prefix(root).unwrap().display()
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}
