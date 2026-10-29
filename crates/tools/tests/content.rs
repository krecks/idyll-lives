//! `cargo content` against what `tools/content/validate.py` did: each case in `fixtures/content/cases.json` runs
//! the binary on a fresh copy of `fixtures/content/{content,plan}` and must print the same stdout, stderr and exit
//! code, and leave the same files, as the Python script did (`fixtures/content/expected/<case>/`, generated once by
//! the script; there, argparse's errors name the program `content`, and a Python traceback keeps only its message).

use std::path::{Path, PathBuf};
use std::process::Command;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dst = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dst);
        } else {
            std::fs::copy(e.path(), dst).unwrap();
        }
    }
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            files(&p, out);
        } else {
            out.push(p);
        }
    }
}

#[test]
fn same_as_the_python_script() {
    let fx = fixtures();
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(fx.join("cases.json")).unwrap()).unwrap();
    let mut failed = Vec::new();
    for case in &cases {
        let name = case["name"].as_str().unwrap();
        let args: Vec<&str> = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a.as_str().unwrap())
            .collect();
        let dir =
            std::env::temp_dir().join(format!("ll-content-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        copy_dir(&fx.join("content"), &dir.join("content"));
        copy_dir(&fx.join("plan"), &dir.join("plan"));
        let out = Command::new(env!("CARGO_BIN_EXE_content"))
            .args(&args)
            .current_dir(&dir)
            .output()
            .unwrap();
        let exp = fx.join("expected").join(name);
        let read = |f: &str| std::fs::read(exp.join(f)).unwrap();
        let mut why = Vec::new();
        if out.stdout != read("stdout") {
            why.push(format!("stdout:\n{}", String::from_utf8_lossy(&out.stdout)));
        }
        if out.stderr != read("stderr") {
            why.push(format!("stderr:\n{}", String::from_utf8_lossy(&out.stderr)));
        }
        let code: i32 = String::from_utf8(read("code"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        if out.status.code() != Some(code) {
            why.push(format!("exit {:?}, not {code}", out.status.code()));
        }
        let mut got = Vec::new();
        files(&dir.join("content"), &mut got);
        for f in got {
            let rel = f.strip_prefix(&dir).unwrap();
            let want = exp.join("files").join(rel);
            let want = if want.exists() { want } else { fx.join(rel) };
            if std::fs::read(&f).unwrap() != std::fs::read(&want).unwrap() {
                why.push(format!("{} differs", rel.display()));
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
        if !why.is_empty() {
            failed.push(format!("{name}: {}", why.join("; ")));
        }
    }
    assert!(
        failed.is_empty(),
        "{} of {} cases differ:\n{}",
        failed.len(),
        cases.len(),
        failed.join("\n")
    );
}

/// The Python script failed on an empty `groups` (the state docs/objects/README.md says to start from); the port
/// puts the first group between the brackets.
#[test]
fn first_group_into_an_empty_validate() {
    let fx = fixtures();
    let dir = std::env::temp_dir().join(format!("ll-content-test-{}-empty", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&fx.join("content"), &dir.join("content"));
    let base = dir.join("content/base.json");
    let text = std::fs::read_to_string(&base).unwrap();
    let text = text.replace("\"groups\": [\n        {\"id\": \"seed\", \"label\": \"Seed\"},\n        {\"id\": \"late\", \"label\": \"Late\"}\n      ]", "\"groups\": []");
    let text = text.replace("\"category\": \"validate\"", "\"category\": \"decor\"");
    std::fs::write(&base, text).unwrap();
    std::fs::write(
        dir.join("content/catalog/one.json"),
        std::fs::read_to_string(fx.join("content/catalog/one.json"))
            .unwrap()
            .replace("\"category\": \"validate\"", "\"category\": \"decor\""),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_content"))
        .args([
            "in",
            "sofa",
            "--group",
            "seats",
            "--content",
            "content",
            "--plan",
            "nowhere",
        ])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = std::fs::read_to_string(&base).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        text.contains("\"groups\": [{\"id\": \"seats\", \"label\": \"Seats\"}]"),
        "{text}"
    );
}
