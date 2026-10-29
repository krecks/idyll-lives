//! `cargo wasm-opt [--names] <file.wasm>...`: the game's WebAssembly (the simulation and the
//! voice), optimised for release with binaryen's wasm-opt (the exact-pinned `binaryen`
//! devDependency): `-O3`, and the name and producers sections stripped. Production builds only:
//! vite.config.ts runs it on the built `dist/assets/` files, so `bun run wasm` for the dev server
//! keeps the names (readable stack traces) and builds no slower. (Was tools/release/wasm-opt.mjs.)
//!
//! - The features are passed explicitly, the set rustc writes into the files (`target_features`,
//!   with simd128 from .cargo/config.toml), so nothing depends on that section surviving.
//! - Never `-tnh` / `-iit` (traps-never-happen, ignore-implicit-traps): with `panic = "abort"`
//!   every Rust panic is a trap, and those flags let the optimiser assume it can't happen.
//! - About 48 s for the simulation on an M3 (binaryen.js is slow), so results are kept in
//!   `web/.assets/wasm-opt/{release,names}/` by the input's SHA-256 and a rebuild of the same wasm
//!   is instant.
//!
//! The `--names` variant runs the same optimisations but leaves the names in, so its function
//! numbers match the shipped wasm and a stack from the site can be read with it.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

const FEATURES: [&str; 9] = [
    "--enable-simd",
    "--enable-bulk-memory",
    "--enable-bulk-memory-opt",
    "--enable-nontrapping-float-to-int",
    "--enable-sign-ext",
    "--enable-mutable-globals",
    "--enable-reference-types",
    "--enable-multivalue",
    "--enable-call-indirect-overlong",
];

/// The shipped optimisations.
fn flags(names: bool) -> Vec<String> {
    let mut out: Vec<String> = FEATURES.iter().map(|s| s.to_string()).collect();
    out.push("-O3".into());
    out.push(if names {
        "-g".into()
    } else {
        "--strip-debug".into()
    });
    out.push("--strip-producers".into());
    out
}

fn web() -> PathBuf {
    crate::characters::repo_root().join("web")
}

/// binaryen's folder: `web/node_modules/binaryen`, or wherever it was hoisted to.
fn binaryen() -> Result<PathBuf, String> {
    let web = web();
    let mut dir = Some(web.as_path());
    while let Some(d) = dir {
        let candidate = d.join("node_modules").join("binaryen");
        if candidate.join("package.json").is_file() {
            return Ok(candidate);
        }
        dir = d.parent();
    }
    Err("binaryen not found: bun install in web/ first".into())
}

fn binaryen_version(dir: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(dir.join("package.json"))
        .map_err(|e| format!("binaryen/package.json: {e}"))?;
    let meta: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("binaryen/package.json: {e}"))?;
    Ok(meta["version"].as_str().unwrap_or("?").to_string())
}

/// The sizes before and after, and whether the result came from the cache.
pub struct Optimized {
    pub before: u64,
    pub after: u64,
    pub cached: bool,
}

/// Optimises `path` in place (`names`: keeping the name section). Older entries for the same file
/// are removed.
pub fn optimize(path: &Path, names: bool) -> Result<Optimized, String> {
    let binaryen = binaryen()?;
    let version = binaryen_version(&binaryen)?;
    let flags = flags(names);
    let name = strip_hash(path);
    let dir = web()
        .join(".assets")
        .join("wasm-opt")
        .join(if names { "names" } else { "release" });
    let input = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hash = Sha256::new();
    hash.update(&input);
    hash.update(format!("\0binaryen {version}\0{}", flags.join(" ")).as_bytes());
    let key: String = format!("{:x}", hash.finalize())[..16].to_string();
    let cached = dir.join(format!("{name}-{key}.wasm"));
    let hit = cached.is_file();
    if !hit {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        // Exactly `name-<key>`: another file's name may start with this one's.
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let file = entry.file_name().to_string_lossy().into_owned();
            let Some(rest) = file.strip_prefix(&format!("{name}-")) else {
                continue;
            };
            let stem = rest
                .strip_suffix(".wasm")
                .or_else(|| rest.strip_suffix(".wasm.part"));
            if stem.is_some_and(|s| {
                s.len() == 16
                    && s.chars()
                        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            }) {
                crate::download::remove(&entry.path());
            }
        }
        let tmp = with_suffix(&cached, ".part");
        run_wasm_opt(&binaryen.join("bin").join("wasm-opt"), &flags, path, &tmp)?;
        std::fs::rename(&tmp, &cached).map_err(|e| format!("{}: {e}", cached.display()))?;
    }
    let part = with_suffix(path, ".part");
    std::fs::copy(&cached, &part).map_err(|e| format!("{}: {e}", part.display()))?;
    std::fs::rename(&part, path).map_err(|e| format!("{}: {e}", path.display()))?;
    let after = std::fs::metadata(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .len();
    Ok(Optimized {
        before: input.len() as u64,
        after,
        cached: hit,
    })
}

/// binaryen's `bin/wasm-opt` is a Node script: run it as one, or as the shell would (its shebang),
/// or with bun if Node isn't installed.
fn run_wasm_opt(script: &Path, flags: &[String], input: &Path, out: &Path) -> Result<(), String> {
    let mut errors = Vec::new();
    for runner in [Some("node"), Some("bun"), None] {
        let mut cmd = match runner {
            Some(program) => {
                let mut c = Command::new(program);
                c.arg(script);
                c
            }
            None => Command::new(script),
        };
        let result = cmd.args(flags).arg(input).arg("-o").arg(out).output();
        match result {
            Ok(o) if o.status.success() => return Ok(()),
            Ok(o) => errors.push(format!(
                "{}: {}",
                runner.unwrap_or("wasm-opt"),
                String::from_utf8_lossy(&o.stderr).trim()
            )),
            Err(e) => errors.push(format!("{}: {e}", runner.unwrap_or("wasm-opt"))),
        }
        let _ = std::fs::remove_file(out);
    }
    Err(format!("wasm-opt failed ({})", errors.join("; ")))
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

/// `sim_wasm_bg-1a2b3c4d.wasm` and `sim_wasm_bg.wasm` are both `sim_wasm_bg`.
fn strip_hash(path: &Path) -> String {
    let file = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let stem = file.strip_suffix(".wasm").unwrap_or(&file);
    match stem.rsplit_once('-') {
        Some((base, hash))
            if hash.len() == 8 && hash.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') =>
        {
            base.to_string()
        }
        _ => stem.to_string(),
    }
}

pub fn main_with(args: &[String]) -> i32 {
    let names = args.first().is_some_and(|a| a == "--names");
    let files: Vec<&String> = if names {
        args[1..].iter().collect()
    } else {
        args.iter().collect()
    };
    if files.is_empty() {
        eprintln!("usage: cargo wasm-opt [--names] <file.wasm>...");
        return 2;
    }
    // One thread per file (the plugin passes both WASMs, and binaryen uses one core each).
    let results: Vec<Result<(String, Optimized, f64), String>> = std::thread::scope(|scope| {
        let handles: Vec<_> = files
            .iter()
            .map(|file| {
                scope.spawn(move || {
                    let start = std::time::Instant::now();
                    let path = Path::new(file.as_str());
                    optimize(path, names)
                        .map(|o| ((*file).clone(), o, start.elapsed().as_secs_f64()))
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_else(|_| Err("wasm-opt panicked".into())))
            .collect()
    });
    let mut failed = false;
    for result in results {
        match result {
            Ok((file, o, secs)) => {
                let cached = if o.cached { " (cached)" } else { "" };
                println!(
                    "wasm-opt {file}: {} -> {} bytes{cached}, {secs:.1} s",
                    o.before, o.after
                );
            }
            Err(e) => {
                eprintln!("{e}");
                failed = true;
            }
        }
    }
    i32::from(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cache_name_drops_the_hash() {
        assert_eq!(
            strip_hash(Path::new("dist/assets/sim_wasm_bg-1a2b3c4d.wasm")),
            "sim_wasm_bg"
        );
        assert_eq!(strip_hash(Path::new("sim_wasm_bg.wasm")), "sim_wasm_bg");
        assert_eq!(
            strip_hash(Path::new("voice_wasm_bg-deadbeef.wasm")),
            "voice_wasm_bg"
        );
    }

    #[test]
    fn the_named_flags_keep_the_names() {
        assert!(flags(true).contains(&"-g".to_string()));
        assert!(!flags(true).contains(&"--strip-debug".to_string()));
        assert!(flags(false).contains(&"--strip-debug".to_string()));
        assert_eq!(flags(false).last().unwrap(), "--strip-producers");
    }
}
