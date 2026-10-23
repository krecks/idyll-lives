//! `cargo licenses --header <text> --out <file> [--package <dir>…]`: `dist/THIRD-PARTY-LICENSES.txt`,
//! written by production builds (vite.config.ts): what the game ships that others made, and their
//! licence texts. (Was tools/release/licenses.mjs; its texts/ are now crates/tools/texts/.)
//!
//! - The npm packages bundled into its JavaScript, as the bundler saw them (the page and its
//!   workers), with the licence files from each package.
//! - The Rust crates compiled into the simulation's and the voice's WebAssembly: `cargo metadata`
//!   for the dependencies of sim-wasm and voice-wasm, the licence files from Cargo's sources.
//! - Files shipped as they are (the meshopt and Zstandard decoders), work adapted into the game's
//!   own code (TalkingHead's expressions), and the licences that came with the voice model, the
//!   dictionary and the language model (fetched next to them, see `cargo fetch-voice` and
//!   `cargo fetch-llm`). The art is listed in assets/CREDITS.md.
//!
//! Identical texts (the Apache-2.0 licence of many crates) are printed once and referred to.

use crate::characters::repo_root;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A package or crate to list: its name, version (empty for the ones named in full), licence, where
/// it comes from, and its licence files as [file name, text].
pub struct Package {
    name: String,
    version: String,
    license: String,
    url: String,
    files: Vec<(String, String)>,
}

fn mit(copyright: &str) -> String {
    format!(
        "MIT License

{copyright}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the \"Software\"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"
    )
}

/// Shipped or adapted work with no npm package to read the licence from: its texts, as published,
/// are kept in crates/tools/texts/.
const KEPT: [(&str, &str, &str, &str); 1] = [("zstddec (decoders/zstddec.wasm, built by Babylon.js), with Zstandard", "MIT; BSD-3-Clause", "https://github.com/donmccurdy/zstddec", "zstddec-LICENSE")];
const ADAPTED: [(&str, &str, &str, &str); 1] = [("TalkingHead's emoji expressions, adapted for the residents' faces (render/shared/characters/face.ts)", "MIT", "https://github.com/met4citizen/TalkingHead", "TalkingHead-LICENSE")];

/// The model and dictionary licences, fetched next to what they cover.
const DATA: [(&str, &str, &str, &str); 3] = [
    ("Supertonic 3 voice model (voice/supertonic-3/)", "BigScience OpenRAIL-M", "https://huggingface.co/Supertone/supertonic-3", "web/public/voice/supertonic-3/LICENSE.txt"),
    ("Misaki US English dictionaries (voice/en-us.lexz)", "Apache-2.0", "https://github.com/hexgrad/misaki", "web/public/voice/misaki-LICENSE.txt"),
    ("LFM2.5-350M language model (llm/lfm2.5-350m/)", "LFM Open License v1.0", "https://huggingface.co/LiquidAI/LFM2.5-350M", "web/public/llm/lfm2.5-350m/LICENSE.txt"),
];

/// The licence file names: `LICENSE`, `LICENCE`, `COPYING`, `UNLICENSE`, `NOTICE`, and their `-`/`_`/`.`
/// variants.
fn license_file_re() -> Regex {
    Regex::new(r"(?i)^(licen[cs]e|copying|unlicense|notice)([-._].*)?$").expect("valid regex")
}

/// Case-insensitive, punctuation before letters: how `localeCompare` orders these names (ASCII).
fn collate(a: &str, b: &str) -> std::cmp::Ordering {
    let key = |s: &str| -> Vec<u16> {
        s.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() as u16 + 1 } else { c as u16 })
            .collect()
    };
    key(a).cmp(&key(b)).then_with(|| a.cmp(b))
}

fn text_of(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// The licence files in `dir`: [file name, text].
fn license_files(dir: &Path) -> Vec<(String, String)> {
    let re = license_file_re();
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|f| re.is_match(f) && dir.join(f).is_file())
        .collect();
    names.sort();
    names.into_iter().filter_map(|f| text_of(&dir.join(&f)).ok().map(|t| (f, t))).collect()
}

/// An npm package's entry, from its `package.json`.
fn npm_package(dir: &Path) -> Result<Package, String> {
    let manifest = dir.join("package.json");
    let text = text_of(&manifest)?;
    let pkg: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", manifest.display()))?;
    let license = match &pkg["license"] {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(o) => o.get("type").and_then(|t| t.as_str()).map(str::to_string).unwrap_or_else(|| {
            pkg["licenses"].as_array().map(|l| l.iter().filter_map(|v| v["type"].as_str()).collect::<Vec<_>>().join(" OR ")).unwrap_or_default()
        }),
        _ => pkg["licenses"].as_array().map(|l| l.iter().filter_map(|v| v["type"].as_str()).collect::<Vec<_>>().join(" OR ")).unwrap_or_default(),
    };
    let license = if license.is_empty() { "unknown".to_string() } else { license };
    let repo = match &pkg["repository"] {
        serde_json::Value::String(s) => Some(s.clone()),
        other => other["url"].as_str().map(str::to_string),
    };
    let raw = pkg["homepage"].as_str().map(str::to_string).or(repo).unwrap_or_else(|| format!("https://www.npmjs.com/package/{}", pkg["name"].as_str().unwrap_or("")));
    let url = github_url(&raw);
    let files = license_files(dir);
    let name = pkg["name"].as_str().unwrap_or("").to_string();
    let files = if files.is_empty() {
        // Packages whose npm release has no licence file: the text from their repository.
        // ONNX Runtime: https://github.com/microsoft/onnxruntime/blob/main/LICENSE
        match name.as_str() {
            "onnxruntime-web" | "onnxruntime-common" => vec![("LICENSE (github.com/microsoft/onnxruntime)".to_string(), mit("Copyright (c) Microsoft Corporation"))],
            _ => Vec::new(),
        }
    } else {
        files
    };
    Ok(Package { name, version: pkg["version"].as_str().unwrap_or("").to_string(), license, url, files })
}

/// `github:user/repo`, `user/repo`, `git+https://…` and `….git` as a link.
fn github_url(raw: &str) -> String {
    let short = Regex::new(r"^(github:)?([\w.-]+/[\w.-]+)$").expect("valid regex");
    let url = short.replace(raw, "https://github.com/$2").into_owned();
    let url = url.strip_prefix("git+").unwrap_or(&url).to_string();
    url.strip_suffix(".git").unwrap_or(&url).to_string()
}

/// The crates (not ours) that sim-wasm and voice-wasm depend on, as built for wasm32.
fn rust_crates(root: &Path) -> Result<Vec<Package>, String> {
    let out = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--locked", "--offline", "--filter-platform", "wasm32-unknown-unknown"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err(format!("cargo metadata: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| format!("cargo metadata: {e}"))?;
    let packages = meta["packages"].as_array().ok_or("cargo metadata: no packages")?;
    let mut by_id: HashMap<&str, &serde_json::Value> = HashMap::new();
    for p in packages {
        if let Some(id) = p["id"].as_str() {
            by_id.insert(id, p);
        }
    }
    let mut nodes: HashMap<&str, &serde_json::Value> = HashMap::new();
    for n in meta["resolve"]["nodes"].as_array().map(Vec::as_slice).unwrap_or_default() {
        if let Some(id) = n["id"].as_str() {
            nodes.insert(id, n);
        }
    }
    let mut stack: Vec<String> = packages.iter().filter(|p| matches!(p["name"].as_str(), Some("sim-wasm" | "voice-wasm"))).filter_map(|p| p["id"].as_str().map(str::to_string)).collect();
    if stack.len() != 2 {
        return Err("sim-wasm or voice-wasm missing from cargo metadata".into());
    }
    let mut seen: Vec<String> = Vec::new();
    while let Some(id) = stack.pop() {
        if seen.contains(&id) {
            continue;
        }
        seen.push(id.clone());
        // Normal dependencies (proc macros included); not build scripts' or tests'.
        let deps = nodes.get(id.as_str()).map(|n| n["deps"].as_array().cloned().unwrap_or_default()).unwrap_or_default();
        for dep in deps {
            let normal = dep["dep_kinds"].as_array().is_some_and(|k| k.iter().any(|kind| kind["kind"].is_null()));
            if let Some(pkg) = dep["pkg"].as_str().filter(|_| normal) {
                stack.push(pkg.to_string());
            }
        }
    }
    let mut out = Vec::new();
    for id in &seen {
        let Some(p) = by_id.get(id.as_str()) else { continue };
        if p["source"].is_null() {
            continue;
        }
        let name = p["name"].as_str().unwrap_or("").to_string();
        let manifest = p["manifest_path"].as_str().unwrap_or("");
        let dir = manifest.strip_suffix("/Cargo.toml").or_else(|| manifest.strip_suffix("\\Cargo.toml")).unwrap_or(manifest);
        out.push(Package {
            version: p["version"].as_str().unwrap_or("").to_string(),
            license: p["license"].as_str().unwrap_or("unknown").to_string(),
            url: p["repository"].as_str().map(str::to_string).unwrap_or_else(|| format!("https://crates.io/crates/{name}")),
            files: license_files(Path::new(dir)),
            name,
        });
    }
    out.sort_by(|a, b| collate(&a.name, &b.name));
    Ok(out)
}

/// One of the kept texts (`crates/tools/texts/`).
fn kept(root: &Path, (name, license, url, file): (&str, &str, &str, &str)) -> Result<Package, String> {
    let path = root.join("crates").join("tools").join("texts").join(file);
    Ok(Package { name: name.to_string(), version: String::new(), license: license.to_string(), url: url.to_string(), files: vec![(file.to_string(), text_of(&path)?)] })
}

/// A licence text, printed once however many packages share it.
struct Text {
    n: usize,
    text: String,
    users: Vec<String>,
}

/// The file's text, and what it lists. `package_dirs`: the npm packages the bundles hold; `header`:
/// the first lines. Fails if something has no licence text at all.
pub fn third_party_licenses(package_dirs: &[String], header: &str) -> Result<(String, (usize, usize, usize)), String> {
    let root = repo_root();
    let web = root.join("web");
    let mut dirs: Vec<PathBuf> = Vec::new();
    for dir in package_dirs {
        let path = std::fs::canonicalize(dir).map_err(|e| format!("{dir}: {e}"))?;
        if !dirs.contains(&path) {
            dirs.push(path);
        }
    }
    let mut npm = Vec::new();
    for dir in &dirs {
        npm.push(npm_package(dir)?);
    }
    npm.sort_by(|a, b| collate(&a.name, &b.name));
    let rust = rust_crates(&root)?;

    let as_is = |name: &str, files: &str| -> Result<Package, String> {
        let dir = std::fs::canonicalize(web.join("node_modules").join(name)).map_err(|e| format!("web/node_modules/{name}: {e}"))?;
        let pkg = npm_package(&dir)?;
        Ok(Package { name: format!("{name} {} ({files})", pkg.version), version: String::new(), ..pkg })
    };
    let mut shipped = vec![as_is("@fontsource-variable/inter", "the Inter font: assets/inter-*.woff2, its licence also in assets/inter-LICENSE.txt")?, as_is("meshoptimizer", "decoders/meshopt_decoder.js")?];
    for k in KEPT {
        shipped.push(kept(&root, k)?);
    }
    let mut data = Vec::new();
    for (name, license, url, file) in DATA {
        let path = root.join(file);
        if !path.is_file() {
            return Err(format!("{file} missing (run cargo fetch-voice and cargo fetch-llm)"));
        }
        let short = file.rsplit('/').next().unwrap_or(file).to_string();
        data.push(Package { name: name.to_string(), version: String::new(), license: license.to_string(), url: url.to_string(), files: vec![(short, text_of(&path)?)] });
    }

    let mut texts: Vec<Text> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut out: Vec<String> = vec![header.trim().to_string(), String::new()];
    let mut section = |out: &mut Vec<String>, title: &str, list: &[Package]| -> Result<(), String> {
        out.push(format!("== {title} =="));
        out.push(String::new());
        for p in list {
            if p.files.is_empty() {
                return Err(format!("no licence text for {} {}", p.name, p.version));
            }
            let mut refs = Vec::new();
            for (file, text) in &p.files {
                let key = format!("{:x}", Sha256::digest(normalize(text).as_bytes()));
                let n = *index.entry(key).or_insert_with(|| {
                    texts.push(Text { n: texts.len() + 1, text: text.clone(), users: Vec::new() });
                    texts.len() - 1
                });
                let t = &mut texts[n];
                let user = format!("{} {}", p.name, p.version);
                t.users.push(user.trim().to_string());
                refs.push(format!("[{}] {file}", t.n));
            }
            out.push(format!("{}{}: {}, {}", p.name, if p.version.is_empty() { String::new() } else { format!(" {}", p.version) }, p.license, p.url));
            out.push(format!("  licence text: {}", refs.join(", ")));
        }
        out.push(String::new());
        Ok(())
    };
    section(&mut out, "JavaScript bundled into assets/*.js (npm packages)", &npm)?;
    section(&mut out, "Rust crates compiled into assets/sim_wasm_bg-*.wasm and voice_wasm_bg-*.wasm", &rust)?;
    section(&mut out, "Files shipped as they are", &shipped)?;
    let adapted: Vec<Package> = ADAPTED.iter().map(|k| kept(&root, *k)).collect::<Result<_, _>>()?;
    section(&mut out, "Adapted into the game's own code", &adapted)?;
    section(&mut out, "Voice model, dictionary and language model (the copies next to them are the same texts)", &data)?;
    out.push("== Licence texts ==".to_string());
    out.push(String::new());
    for t in &texts {
        out.push(format!("---- [{}] for {} ----", t.n, t.users.join(", ")));
        out.push(String::new());
        out.push(t.text.trim().to_string());
        out.push(String::new());
        out.push(String::new());
    }
    Ok((out.join("\n"), (npm.len(), rust.len(), texts.len())))
}

/// A text with its whitespace runs squeezed, so the same licence in two files is one text.
fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn main_with(args: &[String]) -> i32 {
    let mut header = String::new();
    let mut out_path: Option<PathBuf> = None;
    let mut dirs: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                header = args.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "--out" => {
                out_path = args.get(i + 1).map(PathBuf::from);
                i += 2;
            }
            "--package" => {
                if let Some(d) = args.get(i + 1) {
                    dirs.push(d.clone());
                }
                i += 2;
            }
            other => {
                eprintln!("usage: cargo licenses --header <text> --out <file> [--package <dir>…] (not {other})");
                return 2;
            }
        }
    }
    let Some(out_path) = out_path else {
        eprintln!("usage: cargo licenses --header <text> --out <file> [--package <dir>…]");
        return 2;
    };
    match third_party_licenses(&dirs, &header).and_then(|(text, counts)| {
        crate::download::write(&out_path, text.as_bytes())?;
        Ok(counts)
    }) {
        Ok((npm, rust, texts)) => {
            println!("THIRD-PARTY-LICENSES.txt: {npm} npm packages, {rust} Rust crates, {texts} licence texts");
            0
        }
        Err(e) => {
            eprintln!("licenses: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_are_links() {
        assert_eq!(github_url("github:foo/bar"), "https://github.com/foo/bar");
        assert_eq!(github_url("foo/bar"), "https://github.com/foo/bar");
        assert_eq!(github_url("git+https://github.com/foo/bar.git"), "https://github.com/foo/bar");
        assert_eq!(github_url("https://example.com/x"), "https://example.com/x");
    }

    #[test]
    fn licence_files_are_recognised() {
        let re = license_file_re();
        for name in ["LICENSE", "LICENSE.md", "licence", "COPYING", "UNLICENSE", "NOTICE.txt", "LICENSE-MIT"] {
            assert!(re.is_match(name), "{name}");
        }
        for name in ["package.json", "readme.md", "licensing"] {
            assert!(!re.is_match(name), "{name}");
        }
    }

    #[test]
    fn identical_texts_are_one() {
        assert_eq!(normalize("a  b\n\tc\n"), "a b c");
    }

    /// The order `localeCompare` gives: `@` and `-` before letters, whatever the case.
    #[test]
    fn names_collate_as_locale_compare_does() {
        let mut names = vec!["svelte", "@babylonjs/core", "onnxruntime-web", "@fontsource-variable/inter", "binaryen"];
        names.sort_by(|a, b| collate(a, b));
        assert_eq!(names, ["@babylonjs/core", "@fontsource-variable/inter", "binaryen", "onnxruntime-web", "svelte"]);
    }
}
