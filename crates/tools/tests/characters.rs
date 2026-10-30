//! The residents' set: the committed one passes the checks, and a fresh build matches it (within
//! the diff's tolerances). The build is slow unoptimised; `cargo test -p ll-tools --release`.

use ll_tools::characters::{build, check, default_out, diff};

#[test]
fn committed_set_passes_check() {
    assert!(
        check::check(&default_out()),
        "the committed character set fails its checks"
    );
}

#[test]
fn build_matches_committed_set() {
    let out = std::env::temp_dir().join(format!("ll-characters-test-{}", std::process::id()));
    build::build(&out).expect("build");
    let r = diff::diff(&default_out(), &out).expect("diff");
    diff::print(&r, false);
    let ok = check::check(&out);
    std::fs::remove_dir_all(&out).ok();
    assert!(
        r.failures.is_empty(),
        "the build differs from the committed set"
    );
    assert!(ok, "the build fails its checks");
}
