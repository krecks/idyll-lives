//! `cargo art <family> --check`, CI's art step: a copy of the committed tree passes, and the same copy with one byte
//! of a model flipped, or its pack missing from the base manifest, fails. Only the copy is ever changed.

use ll_art::{Options, registry, run_family};
use ll_tools::checkglb::Paths;
use std::path::Path;

fn copy(from: &str, to: &str) {
    std::fs::create_dir_all(Path::new(to).parent().unwrap()).unwrap();
    std::fs::copy(from, to).unwrap_or_else(|e| panic!("{from}: {e}"));
}

#[test]
fn check_passes_the_committed_files_and_fails_a_changed_one() {
    let repo = Paths::repo();
    let root = std::env::temp_dir().join(format!("ll-art-check-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    // what --check reads: the base manifest, every pack, centres.json and the cypress's two files (the content is
    // only read, so it is the repo's)
    let tree = Paths {
        content: repo.content.clone(),
        ..Paths::at(&root.to_string_lossy())
    };
    copy(
        &format!("{}/manifest.json", repo.assets),
        &format!("{}/manifest.json", tree.assets),
    );
    copy(&repo.centres, &tree.centres);
    for e in std::fs::read_dir(format!("{}/packs", repo.assets))
        .unwrap()
        .flatten()
    {
        let name = e.file_name().to_string_lossy().into_owned();
        copy(
            &format!("{}/packs/{name}/manifest.json", repo.assets),
            &format!("{}/packs/{name}/manifest.json", tree.assets),
        );
    }
    for file in ["cypress.glb", "cypress.far.glb"] {
        copy(
            &format!("{}/models/trees/{file}", repo.assets),
            &format!("{}/models/trees/{file}", tree.assets),
        );
    }
    let registry = registry();
    let trees = &registry.iter().find(|(n, _)| *n == "trees").unwrap().1;
    let options = Options {
        only: Some(vec!["cypress".into()]),
        check: true,
        ..Options::default()
    };
    run_family(&tree, trees, &options).unwrap();

    let glb = format!("{}/models/trees/cypress.glb", tree.assets);
    let mut bytes = std::fs::read(&glb).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    std::fs::write(&glb, &bytes).unwrap();
    let err = run_family(&tree, trees, &options).unwrap_err();
    assert!(
        err.contains("not as committed") && err.contains("models/trees/cypress.glb"),
        "{err}"
    );
    bytes[last] ^= 1;
    std::fs::write(&glb, &bytes).unwrap();

    // the pack committed, its line in the base manifest not: a build would add it
    let base = format!("{}/manifest.json", tree.assets);
    let text = std::fs::read_to_string(&base).unwrap();
    let line = "\"packs/trees/manifest.json\"";
    assert!(text.contains(line));
    std::fs::write(&base, text.replace(&format!("    {line},\n"), "")).unwrap();
    let err = run_family(&tree, trees, &options).unwrap_err();
    assert!(err.ends_with("not as committed: manifest.json"), "{err}");
    let _ = std::fs::remove_dir_all(&root);
}
