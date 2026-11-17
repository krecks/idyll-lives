//! `cargo m1-table` against reports tools/m1/table.mjs (its JavaScript predecessor) wrote from the same logs
//! (tests/fixtures/m1: one run with its calibration of the m1 container taken from an earlier run, the
//! earlier run itself), but the margin column's header, which the JavaScript wrote as `${n(NEED)}`.

use ll_tools::m1table::{Settings, report};
use std::path::Path;

fn fixture(run: &str, env: &[(&str, &str)], expected: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/m1");
    let settings = Settings::from(|name| {
        env.iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.to_string())
    })
    .unwrap();
    let got = report(&root.join("bench-results/m1").join(run), &settings);
    let want = std::fs::read_to_string(root.join(expected)).unwrap();
    assert_eq!(got, want, "{expected}");
}

#[test]
fn the_default_settings() {
    fixture("20261102-000000", &[], "expected-default.md");
}

#[test]
fn settings_from_the_environment_and_no_watch() {
    fixture(
        "20261102-000000",
        &[
            ("WATCH", ""),
            ("HOURS", "6"),
            ("ROUNDS", "3"),
            ("M1_QUOTA", "6500"),
            ("TIERS", "full near"),
            ("WORLDS_AT", "0 2"),
        ],
        "expected-settings.md",
    );
}

#[test]
fn an_earlier_run_reads_only_its_own_and_older_logs() {
    fixture(
        "20261101-000000",
        &[("TIERS", "near half"), ("WORLDS_AT", "2 1")],
        "expected-earlier.md",
    );
}
