//! `cargo release-version`: one version for the whole game. `web/package.json` (shown in the menu and
//! Credits), the Rust workspace (`Cargo.toml`) and its crates' entries in `Cargo.lock` must always
//! match. (Was tools/release/version.mjs.)
//!
//!   cargo release-version          print the version, fail if they differ (CI runs this)
//!   cargo release-version 0.3.0    set all three to 0.3.0 (then: commit, `git tag v0.3.0`; pushing the tag
//!                          publishes the game, see .github/workflows/pages.yml)

use crate::characters::repo_root;
use regex::Regex;
use std::path::Path;

/// `([workspace.package] ... version = "...")` in Cargo.toml.
fn section_re() -> Regex {
    Regex::new(r#"(?s)(\[workspace\.package\][^\[]*?\nversion = ")([^"]+)(")"#)
        .expect("valid regex")
}

pub fn main_with(args: &[String]) -> i32 {
    match run(args.first().map(String::as_str)) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("version: {e}");
            1
        }
    }
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn run(next: Option<&str>) -> Result<i32, String> {
    let root = repo_root();
    let pkg_path = root.join("web").join("package.json");
    let cargo_path = root.join("Cargo.toml");
    let lock_path = root.join("Cargo.lock");
    let pkg_text = read(&pkg_path)?;
    let pkg: serde_json::Value =
        serde_json::from_str(&pkg_text).map_err(|e| format!("web/package.json: {e}"))?;
    let pkg_version = pkg["version"]
        .as_str()
        .ok_or("web/package.json: no version")?
        .to_string();
    let cargo = read(&cargo_path)?;
    let lock = read(&lock_path)?;
    let section = section_re();
    let cargo_version = section
        .captures(&cargo)
        .map(|c| c[2].to_string())
        .ok_or("No [workspace.package] version in Cargo.toml")?;

    // The workspace's crates (all take the workspace version) and their entries in Cargo.lock.
    let members = member_names(&root, &cargo)?;
    let mut lock_versions: Vec<(String, Option<String>)> = Vec::new();
    for name in &members {
        let re = entry_re(name);
        lock_versions.push((name.clone(), re.captures(&lock).map(|c| c[2].to_string())));
    }

    let Some(next) = next else {
        let stale: Vec<&(String, Option<String>)> = lock_versions
            .iter()
            .filter(|(_, v)| v.as_deref() != Some(&cargo_version))
            .collect();
        if pkg_version != cargo_version || !stale.is_empty() {
            let listed: Vec<String> = lock_versions
                .iter()
                .map(|(n, v)| format!("{n} {}", v.as_deref().unwrap_or("?")))
                .collect();
            eprintln!(
                "Version mismatch: web/package.json {pkg_version}, Cargo.toml {cargo_version}, Cargo.lock {}",
                listed.join(", ")
            );
            return Ok(1);
        }
        println!("{pkg_version}");
        return Ok(0);
    };
    if !Regex::new(r"^\d+\.\d+\.\d+(-[\w.]+)?$")
        .expect("valid regex")
        .is_match(next)
    {
        return Err(format!("Not a version: {next}"));
    }
    // Only the version line changes: the file keeps its formatting (it is JSON.stringify's own).
    let pkg_next = replace_once(
        &pkg_text,
        &Regex::new(r#"(?m)^(\s*"version":\s*")([^"]*)(")"#).expect("valid regex"),
        next,
    )?;
    std::fs::write(&pkg_path, pkg_next).map_err(|e| format!("{}: {e}", pkg_path.display()))?;
    std::fs::write(
        &cargo_path,
        section
            .replace(&cargo, format!("${{1}}{next}${{3}}"))
            .to_string(),
    )
    .map_err(|e| format!("{}: {e}", cargo_path.display()))?;
    let mut next_lock = lock;
    for (name, v) in &lock_versions {
        if v.is_none() {
            return Err(format!("No {name} in Cargo.lock"));
        }
        next_lock = entry_re(name)
            .replace(&next_lock, format!("${{1}}{next}${{3}}"))
            .to_string();
    }
    std::fs::write(&lock_path, next_lock).map_err(|e| format!("{}: {e}", lock_path.display()))?;
    println!("Version set to {next} (web/package.json, Cargo.toml, Cargo.lock).");
    Ok(0)
}

/// The workspace's crate names, from `members = ["crates/x", ...]`.
fn member_names(root: &Path, cargo: &str) -> Result<Vec<String>, String> {
    let member = Regex::new(r#""crates/([\w-]+)""#).expect("valid regex");
    let name = Regex::new(r#"(?m)^name = "([^"]+)""#).expect("valid regex");
    let mut out = Vec::new();
    for m in member.captures_iter(cargo) {
        let manifest = root.join("crates").join(&m[1]).join("Cargo.toml");
        let text = read(&manifest)?;
        let n = name
            .captures(&text)
            .map(|c| c[1].to_string())
            .ok_or_else(|| format!("{}: no name", manifest.display()))?;
        out.push(n);
    }
    Ok(out)
}

fn entry_re(name: &str) -> Regex {
    Regex::new(&format!(
        r#"(\[\[package\]\]\nname = "{}"\nversion = ")([^"]+)(")"#,
        regex::escape(name)
    ))
    .expect("valid regex")
}

fn replace_once(text: &str, re: &Regex, next: &str) -> Result<String, String> {
    let m = re.find(text).ok_or("No version line to replace")?;
    let mut out = String::with_capacity(text.len() + next.len());
    out.push_str(&text[..m.start()]);
    let caps = re.captures(text).expect("just matched");
    out.push_str(&caps[1]);
    out.push_str(next);
    out.push_str(&caps[3]);
    out.push_str(&text[m.end()..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_workspace_version_is_read() {
        let cargo = "[workspace]\nmembers = [\"crates/sim-core\"]\n\n[workspace.package]\nedition = \"2024\"\nversion = \"0.3.2\"\nlicense = \"GPL-3.0-or-later\"\n\n[profile.dev]\nopt-level = 1\n";
        let caps = section_re().captures(cargo).expect("a version");
        assert_eq!(&caps[2], "0.3.2");
    }

    #[test]
    fn the_lock_entry_is_found_and_set() {
        let lock = "[[package]]\nname = \"sim-core\"\nversion = \"0.3.2\"\n\n[[package]]\nname = \"sim-wasm\"\nversion = \"0.3.2\"\n";
        let re = entry_re("sim-wasm");
        assert_eq!(&re.captures(lock).expect("an entry")[2], "0.3.2");
        let next = re.replace(lock, "${1}0.4.0${3}").to_string();
        assert!(next.contains("name = \"sim-wasm\"\nversion = \"0.4.0\""));
        assert!(next.contains("name = \"sim-core\"\nversion = \"0.3.2\""));
    }

    #[test]
    fn only_the_first_version_line_of_the_package_changes() {
        let pkg = "{\n  \"name\": \"idyll-lives\",\n  \"version\": \"0.3.2\",\n  \"scripts\": {\n    \"x\": \"version 1\"\n  }\n}\n";
        let re = Regex::new(r#"(?m)^(\s*"version":\s*")([^"]*)(")"#).expect("valid regex");
        let next = replace_once(pkg, &re, "0.4.0").unwrap();
        assert_eq!(next, pkg.replace("\"0.3.2\"", "\"0.4.0\""));
    }
}
