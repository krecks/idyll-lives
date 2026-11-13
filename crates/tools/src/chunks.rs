//! `cargo check-chunks`: checks that what the PlayCanvas renderer patches or reads inside PlayCanvas is
//! still there, in the release build the game imports (`web/node_modules/playcanvas/build/playcanvas/src`):
//! the shader chunks' anchor lines the renderer inserts its code at, and the few private names it reaches
//! (`render/pc/*`, marked `SAFETY`). A PlayCanvas upgrade that moves one fails here, in CI, rather than
//! quietly drawing without it. `cd web && bun run check:chunks` (was `tools/render/check_chunks.ts`).
//!
//! The list is `tools/render/chunks.json`: groups, each with what relies on them (`for`) and its anchors.
//! An anchor is a file (below the release build's `src`), the text that must be in it, and who relies on
//! it; or, for a shader chunk, the same anchor in its GLSL and its WGSL
//! (`scene/shader-lib/{glsl,wgsl}/chunks/<chunk>`).

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct List {
    groups: Vec<Group>,
}

#[derive(Deserialize)]
struct Group {
    #[serde(rename = "for")]
    _for: String,
    anchors: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum Entry {
    File {
        file: String,
        text: String,
        who: String,
    },
    Chunk {
        chunk: String,
        glsl: String,
        wgsl: String,
        who: String,
    },
}

/// One thing that must be there: `text` in `file`, for `who`.
#[derive(Debug, PartialEq, Eq)]
pub struct Anchor {
    pub file: String,
    pub text: String,
    pub who: String,
}

/// The list's anchors, a chunk's as its two (GLSL, then WGSL).
pub fn anchors(json: &str) -> Result<Vec<Anchor>, String> {
    let list: List = serde_json::from_str(json).map_err(|e| format!("chunks.json: {e}"))?;
    let mut out = Vec::new();
    for entry in list.groups.into_iter().flat_map(|g| g.anchors) {
        match entry {
            Entry::File { file, text, who } => out.push(Anchor { file, text, who }),
            Entry::Chunk {
                chunk,
                glsl,
                wgsl,
                who,
            } => {
                for (lang, text) in [("glsl", glsl), ("wgsl", wgsl)] {
                    out.push(Anchor {
                        file: format!("scene/shader-lib/{lang}/chunks/{chunk}"),
                        text,
                        who: format!("{who} ({})", lang.to_uppercase()),
                    });
                }
            }
        }
    }
    Ok(out)
}

/// The anchors that aren't in the source tree at `src` (a file that is missing has none of its own).
pub fn missing<'a>(src: &Path, anchors: &'a [Anchor]) -> Vec<&'a Anchor> {
    let mut files: HashMap<&str, String> = HashMap::new();
    anchors
        .iter()
        .filter(|a| {
            !files
                .entry(a.file.as_str())
                .or_insert_with(|| std::fs::read_to_string(src.join(&a.file)).unwrap_or_default())
                .contains(&a.text)
        })
        .collect()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn main_with(args: &[String]) -> i32 {
    if !args.is_empty() {
        eprintln!("usage: cargo check-chunks");
        return if matches!(args[0].as_str(), "-h" | "--help" | "help") {
            0
        } else {
            2
        };
    }
    let root = root();
    let list = root.join("tools/render/chunks.json");
    let anchors = match std::fs::read_to_string(&list)
        .map_err(|e| format!("{}: {e}", list.display()))
        .and_then(|text| anchors(&text))
    {
        Ok(anchors) => anchors,
        Err(e) => {
            eprintln!("check_chunks: {e}");
            return 2;
        }
    };
    let playcanvas = root.join("web/node_modules/playcanvas");
    let Some(version) = std::fs::read_to_string(playcanvas.join("package.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|j| j["version"].as_str().map(String::from))
    else {
        eprintln!("check_chunks: no PlayCanvas in web/node_modules (run `bun install` in web/)");
        return 2;
    };
    let gone = missing(&playcanvas.join("build/playcanvas/src"), &anchors);
    for a in &gone {
        eprintln!(
            "check_chunks: not found in {}: {}\n  ({})",
            a.file, a.text, a.who
        );
    }
    if !gone.is_empty() {
        eprintln!(
            "check_chunks: {} of {} missing in PlayCanvas {version}",
            gone.len(),
            anchors.len()
        );
        return 1;
    }
    println!(
        "check_chunks: {} anchors and names found in PlayCanvas {version}",
        anchors.len()
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chunk_is_its_glsl_and_its_wgsl() {
        let list = r#"{ "groups": [{ "for": "x.ts", "anchors": [
            { "file": "a.js", "text": "one(", "who": "x.ts: one" },
            { "chunk": "lit/frag/end.js", "glsl": "void f() {", "wgsl": "fn f() {", "who": "x.ts: f" }
        ] }] }"#;
        let anchors = anchors(list).unwrap();
        assert_eq!(anchors.len(), 3);
        assert_eq!(
            anchors[1].file,
            "scene/shader-lib/glsl/chunks/lit/frag/end.js"
        );
        assert_eq!(
            anchors[2].file,
            "scene/shader-lib/wgsl/chunks/lit/frag/end.js"
        );
        assert_eq!(anchors[2].who, "x.ts: f (WGSL)");
        assert!(
            super::anchors(r#"{ "groups": [{ "for": "", "anchors": [{ "file": "a.js" }] }] }"#)
                .is_err()
        );
    }

    #[test]
    fn what_is_gone_is_found() {
        let dir = std::env::temp_dir().join(format!("ll-chunks-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.js"), "function one(x) {}").unwrap();
        let anchors = vec![
            Anchor {
                file: "a.js".into(),
                text: "one(".into(),
                who: "w".into(),
            },
            Anchor {
                file: "a.js".into(),
                text: "two(".into(),
                who: "w".into(),
            },
            Anchor {
                file: "none.js".into(),
                text: "one(".into(),
                who: "w".into(),
            },
        ];
        let gone = missing(&dir, &anchors);
        assert_eq!(gone, [&anchors[1], &anchors[2]]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_repo_s_list_reads() {
        let text = std::fs::read_to_string(root().join("tools/render/chunks.json")).unwrap();
        let anchors = anchors(&text).unwrap();
        assert!(anchors.len() >= 235, "only {} anchors", anchors.len());
        assert!(
            anchors
                .iter()
                .all(|a| !a.file.is_empty() && !a.text.is_empty() && !a.who.is_empty())
        );
    }
}
