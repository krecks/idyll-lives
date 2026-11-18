//! Where `cargo art` builds a family (`ll_art::route`): the registry decides, not whether a script of its name is
//! left. And `--centres`: what the families built here give is written over what Blender's scripts report.
#[path = "../../artkit/tests/common/cypress.rs"]
mod cypress;

use ll_art::Port::{Ported, Switched, Unported};
use ll_art::{Asked, Port, Route, centres_of, registry, route};
use ll_tools::art::{families, write_centres};
use ll_tools::checkglb::{Paths, read_centres};

/// A registry of every kind: the trees switched and their script gone, the plants switched and their script kept
/// as the others' library, the kitchen ported with its script, the ground ported and its script gone (a mistake:
/// nothing builds it but --out and --check), the living room and the birds not ported, the street neither ported
/// nor a script.
const REGISTRY: [(&str, Port); 7] = [
    ("trees", Switched),
    ("plants", Switched),
    ("kitchen", Ported),
    ("ground", Ported),
    ("living", Unported),
    ("street", Unported),
    ("birds", Unported),
];

fn scripts() -> Vec<String> {
    ["birds", "kitchen", "living", "plants"]
        .map(String::from)
        .to_vec()
}

const PLAIN: Asked = Asked {
    all: false,
    out: false,
    check: false,
};
const OUT: Asked = Asked { out: true, ..PLAIN };
const CHECK: Asked = Asked {
    check: true,
    ..PLAIN
};
const ALL: Asked = Asked { all: true, ..PLAIN };

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| n.to_string()).collect()
}

fn routed(asked_names: &[&str], asked: Asked) -> Result<Route, String> {
    route(&names(asked_names), &REGISTRY, &scripts(), asked)
}

/// (here, in Blender, how many lines said).
fn built(asked_names: &[&str], asked: Asked) -> (Vec<String>, Vec<String>, usize) {
    let r = routed(asked_names, asked).unwrap();
    (r.here, r.blender, r.notes.len())
}

#[test]
fn a_switched_family_is_built_here_whatever_is_asked_and_whether_or_not_its_script_is_left() {
    for asked in [PLAIN, OUT, CHECK] {
        for name in ["trees", "plants"] {
            assert_eq!(
                built(&[name], asked),
                (names(&[name]), Vec::new(), 0),
                "{name} {asked:?}"
            );
        }
    }
    // with its script there and without it: the same
    let none: Vec<String> = Vec::new();
    for scripts in [scripts(), none] {
        let r = route(&names(&["plants"]), &REGISTRY, &scripts, PLAIN).unwrap();
        assert_eq!((r.here, r.blender), (names(&["plants"]), Vec::new()));
    }
}

#[test]
fn a_ported_family_not_switched_is_blenders_but_for_out_and_check() {
    let r = routed(&["kitchen"], PLAIN).unwrap();
    assert_eq!((r.here, r.blender), (Vec::new(), names(&["kitchen"])));
    assert_eq!(
        r.notes,
        [
            "[gamekit] kitchen: ported, not switched: built in Blender (--out or --check build it here)"
        ]
    );
    for asked in [OUT, CHECK] {
        assert_eq!(
            built(&["kitchen"], asked),
            (names(&["kitchen"]), Vec::new(), 0),
            "{asked:?}"
        );
    }
    // its script gone and no mark: built here to be proved, and refused otherwise (nothing would build it)
    for asked in [OUT, CHECK] {
        assert_eq!(built(&["ground"], asked).0, names(&["ground"]), "{asked:?}");
    }
    let err = routed(&["ground"], PLAIN).unwrap_err();
    assert!(
        err.starts_with("ground: ported, not switched, and no script")
            && err.contains(".switched()"),
        "{err}"
    );
}

#[test]
fn a_family_not_ported_is_its_script_and_none_with_out_or_check() {
    assert_eq!(
        built(&["living"], PLAIN),
        (Vec::new(), names(&["living"]), 0)
    );
    for asked in [OUT, CHECK] {
        let err = routed(&["trees", "living", "birds"], asked).unwrap_err();
        assert!(
            err.starts_with("living, birds: still built in Blender"),
            "{err}"
        );
    }
    // a script the registry doesn't name is still a script
    let scripts = names(&["stalls"]);
    let r = route(&names(&["stalls"]), &REGISTRY, &scripts, PLAIN).unwrap();
    assert_eq!(r.blender, names(&["stalls"]));
}

#[test]
fn several_families_go_each_its_way_in_the_order_asked() {
    assert_eq!(
        built(&["living", "plants", "kitchen", "trees"], PLAIN),
        (
            names(&["plants", "trees"]),
            names(&["living", "kitchen"]),
            1
        )
    );
    assert_eq!(
        built(&["kitchen", "plants", "trees"], CHECK),
        (names(&["kitchen", "plants", "trees"]), Vec::new(), 0)
    );
}

#[test]
fn a_name_neither_ported_nor_a_script_is_no_family() {
    for asked in [PLAIN, OUT, CHECK] {
        // (the street: named by the registry, its module empty and no script)
        for odd in ["nosuch", "street"] {
            let err = routed(&["trees", odd], asked).unwrap_err();
            assert!(
                err.starts_with(&format!("no family {odd:?} (trees, plants, ")),
                "{err}"
            );
        }
    }
    let err = routed(&[], PLAIN).unwrap_err();
    assert!(err.starts_with("name a family (trees, plants, "), "{err}");
}

#[test]
fn all_builds_the_switched_here_and_the_others_scripts_in_blender() {
    let r = routed(&[], ALL).unwrap();
    assert_eq!(r.here, names(&["trees", "plants"]));
    // (the plants' script is left, and isn't run; the ground and the street have none)
    assert_eq!(r.blender, names(&["kitchen", "living", "birds"]));
    assert_eq!(r.notes.len(), 2, "{:?}", r.notes);
    assert!(r.notes[0].starts_with("[gamekit] kitchen: ported, not switched: built in Blender"));
    assert!(
        r.notes[1].starts_with("[gamekit] ground: ported, not switched, and no script")
            && r.notes[1].contains("not built"),
        "{}",
        r.notes[1]
    );
    // names given with --all say nothing
    assert_eq!(routed(&["nosuch"], ALL).unwrap(), r);
}

#[test]
fn all_check_holds_the_switched_to_the_committed_files_and_leaves_the_rest_out() {
    for asked in [
        Asked { check: true, ..ALL },
        Asked {
            check: true,
            out: true,
            ..ALL
        },
    ] {
        let r = routed(&[], asked).unwrap();
        assert_eq!(
            (r.here, r.blender),
            (names(&["trees", "plants"]), Vec::new())
        );
        // the kitchen, the ground, the living room and the birds (the street is nothing yet)
        assert_eq!(r.notes.len(), 1, "{:?}", r.notes);
        assert!(
            r.notes[0].starts_with("[gamekit] not switched, left out: 4 "),
            "{}",
            r.notes[0]
        );
    }
    // all switched: nothing to say
    let switched = [("trees", Switched), ("plants", Switched)];
    let r = route(&[], &switched, &scripts(), Asked { check: true, ..ALL }).unwrap();
    assert_eq!((r.here.len(), r.notes.len()), (2, 0));
}

#[test]
fn all_out_writes_every_ported_family_and_leaves_blenders_out() {
    let r = routed(&[], Asked { out: true, ..ALL }).unwrap();
    assert_eq!(r.here, names(&["trees", "plants", "kitchen", "ground"]));
    assert!(r.blender.is_empty());
    assert_eq!(r.notes, ["[gamekit] still built in Blender, left out: 2"]);
}

/// The registry as it is: the trees are switched, and no family is left with nothing to build it (ported, its
/// script deleted and not marked switched).
#[test]
fn the_registry_has_the_trees_switched_and_every_ported_family_switched_or_with_its_script() {
    let registry = registry();
    let ports: Vec<(&str, Port)> = registry
        .iter()
        .map(|(n, f)| (*n, Port::of(f.as_ref())))
        .collect();
    let scripts = families(&Paths::repo().here);
    assert!(ports.contains(&("trees", Switched)));
    for (name, port) in &ports {
        assert!(
            *port != Ported || scripts.iter().any(|s| s == name),
            "{name}: ported, its script gone and not switched: `.switched()` in its module"
        );
        assert!(
            *port != Unported || scripts.iter().any(|s| s == name),
            "{name}: neither ported nor a script"
        );
    }
    let asked = Asked { check: true, ..ALL };
    let r = route(&[], &ports, &scripts, asked).unwrap();
    assert!(r.here.iter().any(|n| n == "trees"));
    assert!(
        r.here
            .iter()
            .all(|n| ports.contains(&(n.as_str(), Switched)))
    );
    assert!(r.blender.is_empty());
}

/// `--centres`: Blender's scripts write the file anew, then the families built here write theirs over it. A
/// switched family whose script is left (plants.py, the others' library) has its module's centres, not its script's.
#[test]
fn centres_of_the_families_built_here_win_over_what_the_scripts_reported() {
    let root = std::env::temp_dir().join(format!("ll-art-centres-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = Paths::at(&root.to_string_lossy());
    std::fs::create_dir_all(std::path::Path::new(&paths.centres).parent().unwrap()).unwrap();
    // what the scripts reported: the cypress off its place, and a model of a family still Blender's
    std::fs::write(
        &paths.centres,
        "{\n  \"model.cypress\": [9.0, 9.0],\n  \"model.living.sofa\": [0.1, 0.2]\n}\n",
    )
    .unwrap();
    let mut family = cypress::family().switched();
    family.models[0].centre = [0.25, -0.5];
    write_centres(&paths, &centres_of(&[&family]), false).unwrap();
    let written = read_centres(&paths.centres).unwrap();
    let centre = |key: &str| {
        let c = written.get(key).and_then(|c| c.list()).unwrap();
        [c[0].num().unwrap().f(), c[1].num().unwrap().f()]
    };
    assert_eq!(centre("model.cypress"), [0.25, -0.5]);
    assert_eq!(centre("model.living.sofa"), [0.1, 0.2]);
    assert_eq!(written.entries().unwrap().len(), 2);
    let _ = std::fs::remove_dir_all(&root);
}
