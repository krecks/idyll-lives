//! `cargo check-glb --validate`: the Khronos glTF validator (web/package.json's `gltf-validator`, run under Bun by
//! tools/art/validate.ts) over each model file the gate checks and its far version, a counted check (`validator`):
//! a model's count is the issues that fail (`fails`) in its file and its far version's (`far: …`).
//!
//! The validator knows glTF, not the game: what fails is every error and warning, and the information the game
//! pays for (`FAILING_INFO`), less what the game's way of drawing makes right (`VALIDATOR_IGNORED`).

use super::baseline::Found;
use super::py::join;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::process::Command;

/// The script that runs the validator, under a checkout's root.
pub const SCRIPT: &str = "tools/art/validate.ts";

/// The validator's package, under a checkout's root: `bun install` in web/ fetches it (pinned in web/package.json).
pub const PACKAGE: &str = "web/node_modules/gltf-validator";

/// The validator's severities, by number.
const SEVERITIES: [&str; 4] = ["error", "warning", "information", "hint"];

/// Issues below a warning that fail all the same: (code, why).
const FAILING_INFO: &[(&str, &str)] = &[(
    "UNUSED_OBJECT",
    "an accessor, a material, a node or a texture nothing uses: bytes downloaded for nothing",
)];

/// Issues that never fail, whatever their severity: (code, a part of the pointer they are at, "" for any; why).
pub const VALIDATOR_IGNORED: &[(&str, &str, &str)] = &[
    (
        "UNUSED_OBJECT",
        "/attributes/TEXCOORD_",
        "UVs of a material with no texture in the file: the finish it maps to in the game is textured (wood, \
         fabric: artkit's uv.rs), which the validator can't know; UVs that no finish reads are the gate's own \
         `materials` finding",
    ),
    (
        "NODE_EMPTY",
        "",
        "a node with nothing under it draws nothing and costs a few bytes (an empty kept for its name or as a pivot)",
    ),
    (
        "BUFFER_VIEW_TARGET_MISSING",
        "",
        "a hint for loaders that upload whole buffer views; PlayCanvas makes its buffers from the accessors",
    ),
];

/// One issue the validator reported, as tools/art/validate.ts prints it.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Issue {
    pub code: String,
    pub severity: u8,
    #[serde(default)]
    pub pointer: String,
    pub message: String,
}

/// One line of tools/art/validate.ts: a file's issues, or why it couldn't be validated.
#[derive(Deserialize)]
struct Line {
    file: String,
    issues: Option<Vec<Issue>>,
    error: Option<String>,
}

/// What the validator made of one file: its issues, or why it couldn't validate it.
pub type Outcome = Result<Vec<Issue>, String>;

/// Whether an issue fails the model (the module's policy).
pub fn fails(issue: &Issue) -> bool {
    let ignored = VALIDATOR_IGNORED
        .iter()
        .any(|(code, at, _)| *code == issue.code && issue.pointer.contains(at));
    !ignored && (issue.severity <= 1 || FAILING_INFO.iter().any(|(c, _)| *c == issue.code))
}

/// How a failing issue prints: `validator: CODE (severity) at /pointer: message`.
fn describe(issue: &Issue) -> String {
    let severity = SEVERITIES
        .get(issue.severity as usize)
        .copied()
        .unwrap_or("severity ?");
    let at = if issue.pointer.is_empty() {
        String::new()
    } else {
        format!(" at {}", issue.pointer)
    };
    format!(
        "validator: {} ({severity}){at}: {}",
        issue.code, issue.message
    )
}

/// Whether the validator can run here: Bun on PATH and the package in web/node_modules (Err says what to do).
pub fn ready(root: &str) -> Result<(), String> {
    if !Path::new(&join(root, PACKAGE)).join("index.js").exists() {
        return Err(format!(
            "the Khronos glTF validator isn't installed ({PACKAGE}): `bun install` in web/"
        ));
    }
    match Command::new("bun").arg("--version").output() {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(format!(
            "`bun --version` failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(e) => Err(format!(
            "bun not found ({e}): the validator runs under Bun (https://bun.sh)"
        )),
    }
}

/// The script's output (`text`) for `files`: each file's outcome. A line that doesn't read, or a file it has
/// no line for, is an error.
fn parse(text: &str, files: &[&str]) -> Result<HashMap<String, Outcome>, String> {
    let mut out = HashMap::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let Line {
            file,
            issues,
            error,
        } = serde_json::from_str(line).map_err(|e| {
            format!("{SCRIPT} printed a line that isn't a file's issues ({e}): {line}")
        })?;
        let outcome = match (issues, error) {
            (Some(issues), None) => Ok(issues),
            (None, Some(error)) => Err(error),
            _ => {
                return Err(format!(
                    "{SCRIPT}: {file} has neither issues nor an error, or both"
                ));
            }
        };
        out.insert(file, outcome);
    }
    if let Some(missing) = files.iter().find(|f| !out.contains_key(**f)) {
        return Err(format!("{SCRIPT} printed nothing for {missing}"));
    }
    Ok(out)
}

/// Runs the validator over `files` (paths it can open) from the checkout at `root`.
fn run(root: &str, files: &[&str]) -> Result<HashMap<String, Outcome>, String> {
    let out = Command::new("bun")
        .arg(join(root, SCRIPT))
        .args(files)
        .current_dir(root)
        .output()
        .map_err(|e| format!("cannot run bun {SCRIPT}: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let chars: Vec<char> = err.trim().chars().collect();
        let tail: String = chars[chars.len().saturating_sub(2000)..].iter().collect();
        return Err(format!("bun {SCRIPT} failed ({}): {tail}", out.status));
    }
    parse(&String::from_utf8_lossy(&out.stdout), files)
}

/// Each model's failing issues (`(key, file, far file)`; `results`: each file's outcome), its far version's
/// as `far: …`; a model whose files weren't validated is left out. A file the validator couldn't read is one.
fn found(
    models: &[(String, String, Option<String>)],
    results: &HashMap<String, Outcome>,
) -> BTreeMap<String, Found> {
    let mut out = BTreeMap::new();
    for (key, file, far) in models {
        let mut text = Vec::new();
        let mut validated = false;
        for (path, label) in [(Some(file), ""), (far.as_ref(), "far: ")] {
            let Some(outcome) = path.and_then(|p| results.get(p)) else {
                continue;
            };
            validated = true;
            match outcome {
                Ok(issues) => text.extend(
                    issues
                        .iter()
                        .filter(|i| fails(i))
                        .map(|i| format!("{label}{}", describe(i))),
                ),
                Err(err) => text.push(format!("{label}validator: cannot validate it: {err}")),
            }
        }
        if validated {
            let count = text.len() as u64;
            out.insert(key.clone(), Found { count, text });
        }
    }
    out
}

/// The counted check `validator` (`checkglb::BatchCheck`): each model's failing issues, its file's and its far
/// version's, of those files that are there (the gate fails one that isn't). Err when the validator can't run.
pub fn check(
    models: &[(String, String, Option<String>)],
) -> Result<BTreeMap<String, Found>, String> {
    let root = super::repo_root();
    ready(&root)?;
    let mut files: Vec<&str> = models
        .iter()
        .flat_map(|(_, file, far)| std::iter::once(file).chain(far))
        .map(String::as_str)
        .filter(|f| Path::new(f).is_file())
        .collect();
    files.sort_unstable();
    files.dedup();
    let results = if files.is_empty() {
        HashMap::new()
    } else {
        run(&root, &files)?
    };
    Ok(found(models, &results))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(code: &str, severity: u8, pointer: &str) -> Issue {
        Issue {
            code: code.into(),
            severity,
            pointer: pointer.into(),
            message: "It is so.".into(),
        }
    }

    #[test]
    fn errors_warnings_and_unused_objects_fail_but_the_finishes_uvs() {
        let fail = |code, severity, pointer| fails(&issue(code, severity, pointer));
        assert!(fail("ACCESSOR_INDEX_TRIANGLE_DEGENERATE", 0, "/meshes/0"));
        assert!(fail("MESH_PRIMITIVE_GENERATED_TANGENT_SPACE", 1, ""));
        assert!(fail("UNUSED_OBJECT", 2, "/accessors/12"));
        assert!(fail("UNUSED_OBJECT", 2, "/materials/3"));
        assert!(!fail(
            "UNUSED_OBJECT",
            2,
            "/meshes/0/primitives/4/attributes/TEXCOORD_0"
        ));
        assert!(!fail(
            "UNUSED_OBJECT",
            2,
            "/meshes/1/primitives/0/attributes/TEXCOORD_1"
        ));
        assert!(!fail("UNUSED_MESH_TANGENT", 2, "/meshes/0"));
        // Ignored whatever the severity.
        assert!(!fail("NODE_EMPTY", 1, "/nodes/2"));
        assert!(!fail("BUFFER_VIEW_TARGET_MISSING", 0, "/bufferViews/0"));
        // Every ignored or failing information code says why.
        assert!(VALIDATOR_IGNORED.iter().all(|(_, _, why)| !why.is_empty()));
        assert!(FAILING_INFO.iter().all(|(_, why)| !why.is_empty()));
    }

    #[test]
    fn a_line_a_file() {
        let text = concat!(
            r#"{"file": "a.glb", "issues": [{"code": "UNUSED_OBJECT", "severity": 2, "pointer": "/accessors/1", "message": "This object may be unused."}]}"#,
            "\n",
            r#"{"file": "b.glb", "error": "Invalid data: could not detect glTF format."}"#,
            "\n\n",
            r#"{"file": "c.glb", "issues": []}"#,
            "\n"
        );
        let out = parse(text, &["a.glb", "b.glb", "c.glb"]).unwrap();
        assert_eq!(
            out["a.glb"],
            Ok(vec![Issue {
                code: "UNUSED_OBJECT".into(),
                severity: 2,
                pointer: "/accessors/1".into(),
                message: "This object may be unused.".into()
            }])
        );
        assert_eq!(
            out["b.glb"],
            Err("Invalid data: could not detect glTF format.".into())
        );
        assert_eq!(out["c.glb"], Ok(Vec::new()));
        // A file it printed nothing for, a line that isn't one, a line with neither or both.
        let e = parse(text, &["a.glb", "d.glb"]).unwrap_err();
        assert!(e.ends_with("printed nothing for d.glb"), "{e}");
        assert!(parse("Bun v1.4.4\n", &[]).is_err());
        assert!(parse(r#"{"file": "a.glb"}"#, &[]).is_err());
        assert!(parse(r#"{"file": "a.glb", "issues": [], "error": "x"}"#, &[]).is_err());
    }

    #[test]
    fn a_model_counts_its_file_s_and_its_far_version_s() {
        let results: HashMap<String, Outcome> = HashMap::from([
            (
                "a.glb".to_string(),
                Ok(vec![
                    issue(
                        "UNUSED_OBJECT",
                        2,
                        "/meshes/0/primitives/0/attributes/TEXCOORD_0",
                    ),
                    issue("ACCESSOR_NON_UNIT", 0, "/accessors/2"),
                ]),
            ),
            (
                "a_far.glb".to_string(),
                Ok(vec![issue("UNUSED_OBJECT", 2, "/accessors/7")]),
            ),
            ("b.glb".to_string(), Ok(Vec::new())),
            ("c.glb".to_string(), Err("Invalid data".to_string())),
        ]);
        let models = [
            (
                "model.a".to_string(),
                "a.glb".to_string(),
                Some("a_far.glb".to_string()),
            ),
            ("model.b".to_string(), "b.glb".to_string(), None),
            ("model.c".to_string(), "c.glb".to_string(), None),
            // Its files not validated (not there): left out.
            ("model.d".to_string(), "d.glb".to_string(), None),
        ];
        let out = found(&models, &results);
        assert_eq!(
            out["model.a"],
            Found {
                count: 2,
                text: vec![
                    "validator: ACCESSOR_NON_UNIT (error) at /accessors/2: It is so.".into(),
                    "far: validator: UNUSED_OBJECT (information) at /accessors/7: It is so.".into(),
                ]
            }
        );
        // Clean: counted as none, so the check ran on it.
        assert_eq!(out["model.b"], Found::default());
        assert_eq!(
            out["model.c"].text,
            ["validator: cannot validate it: Invalid data"]
        );
        assert!(!out.contains_key("model.d"));
        assert_eq!(
            describe(&issue("NODE_EMPTY", 2, "")),
            "validator: NODE_EMPTY (information): It is so."
        );
    }

    /// The script itself, where Bun and the package are installed (CI's `models` step runs it over every model).
    #[test]
    fn the_validator_reads_a_model_and_fails_a_broken_file() {
        let root = super::super::repo_root();
        if let Err(why) = ready(&root) {
            eprintln!("skipped: {why}");
            return;
        }
        let dir = std::env::temp_dir().join(format!("ll-validate-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let broken = dir.join("broken.glb").to_string_lossy().into_owned();
        std::fs::write(&broken, b"glTF but not one").unwrap();
        let coin = join(&root, "web/public/assets/models/held/coin.glb");
        let models = [
            ("tool.coin".to_string(), coin, None),
            ("model.broken".to_string(), broken, None),
        ];
        let out = check(&models).unwrap();
        assert_eq!(out["tool.coin"], Found::default());
        assert_eq!(out["model.broken"].count, 1);
        assert!(
            out["model.broken"].text[0].starts_with("validator: "),
            "{:?}",
            out["model.broken"]
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
