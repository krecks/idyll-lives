//! `cargo art`: builds Blender families and checks what they wrote (was tools/art/blender/build.py).
//!
//!   cargo art kitchen                  build, export and register a family
//!   cargo art kitchen --only fridge    one model of it (--only a,b: several)
//!   cargo art --all                    every family
//!   cargo art garden_decor --preview   render previews/ instead of exporting
//!   cargo art kitchen --no-pack        export without writing the pack
//!   cargo art --centres                write centres.json from every family (gamekit/centres.py, no build)
//!
//! A family is a script in tools/art/blender/families/ (those starting with `_` are templates). Blender
//! comes from $BLENDER, the macOS app, or PATH; the models are made with Blender 5.2 LTS. After exporting
//! it runs the gate (`cargo check-glb`) on the keys it built and exits 1 if a build or the gate fails.
//!
//! `--centres` writes centres.json (where each model is centred at rest, for the gate) for every model
//! of every family, as a build writes it for the models it builds: Blender imports the family scripts
//! without building anything. Run it after changing a Model's `centre` without a rebuild (the gate then
//! says which files are off it), or after adding or removing models.

use crate::checkglb::{self, Paths};
use std::path::Path;
use std::process::Command;

/// Blender: `$BLENDER`, the macOS app, or `blender` on PATH.
pub fn blender() -> Result<String, String> {
    let on_path = || -> Option<String> {
        std::env::split_paths(&std::env::var_os("PATH")?).map(|d| d.join("blender")).find(|p| is_executable(p)).map(|p| p.to_string_lossy().into_owned())
    };
    let env = std::env::var("BLENDER").ok().filter(|p| !p.is_empty());
    for path in [env, Some("/Applications/Blender.app/Contents/MacOS/Blender".into()), on_path()].into_iter().flatten() {
        if Path::new(&path).exists() {
            return Ok(path);
        }
    }
    Err("Blender not found: install Blender 5.2 LTS or set $BLENDER".into())
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata().is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// The family scripts (not the `_` templates), sorted.
pub fn families(here: &str) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(Path::new(here).join("families"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".py") && !n.starts_with('_') && !n.starts_with('.'))
        .map(|n| n.trim_end_matches(".py").to_string())
        .collect();
    out.sort();
    out
}

/// What a Blender run printed (newlines as Python's text mode reads them) and whether it succeeded.
pub struct Run {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Runs a script in Blender, in the background, with the factory settings (so the export settings stay
/// fixed) and an exit code on a Python error (else Blender exits 0 on one): `exe --background
/// --factory-startup --python-exit-code 1 --python <script> [-- <args>]`.
pub fn run(exe: &str, script: &str, args: &[String]) -> Run {
    let mut cmd = Command::new(exe);
    cmd.args(["--background", "--factory-startup", "--python-exit-code", "1", "--python", script]);
    if !args.is_empty() {
        cmd.arg("--").args(args);
    }
    let text = |b: &[u8]| String::from_utf8_lossy(b).replace("\r\n", "\n").replace('\r', "\n");
    match cmd.output() {
        Ok(out) => Run { ok: out.status.success(), stdout: text(&out.stdout), stderr: text(&out.stderr) },
        Err(err) => Run { ok: false, stdout: String::new(), stderr: format!("{exe}: {err}") },
    }
}

/// Python's `str.splitlines()`.
fn splitlines(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = s;
    while !rest.is_empty() {
        match rest.char_indices().find(|(_, c)| matches!(c, '\n' | '\r' | '\u{b}' | '\u{c}' | '\u{1c}' | '\u{1d}' | '\u{1e}' | '\u{85}' | '\u{2028}' | '\u{2029}')) {
            Some((i, c)) => {
                out.push(&rest[..i]);
                let skip = if rest[i..].starts_with("\r\n") { 2 } else { c.len_utf8() };
                rest = &rest[i + skip..];
            }
            None => {
                out.push(rest);
                break;
            }
        }
    }
    out
}

/// Prints a run's `[gamekit]` lines and, when it failed, the Python traceback (Blender prints it to stdout;
/// else the last 20 lines) and the end of stderr. Whether it succeeded.
pub fn echo(run: &Run) -> bool {
    let out = splitlines(&run.stdout);
    for line in out.iter().filter(|l| l.starts_with("[gamekit]")) {
        println!("{line}");
    }
    if !run.ok {
        let start = out.iter().position(|l| l.starts_with("Traceback")).unwrap_or(out.len().saturating_sub(20));
        let chars: Vec<char> = run.stderr.chars().collect();
        let tail: String = chars[chars.len().saturating_sub(2000)..].iter().collect();
        println!("{}\n{tail}", out[start..].join("\n"));
    }
    run.ok
}

fn exit(msg: String) -> i32 {
    eprintln!("{msg}");
    1
}

/// `cargo art …`: the exit code.
pub fn main(argv: &[String]) -> i32 {
    let paths = Paths::repo();
    let here = paths.here.clone();
    let has = |flag: &str| argv.iter().any(|a| a == flag);
    let only = argv.iter().position(|a| a == "--only").map(|i| argv.get(i + 1).cloned());
    if has("--centres") {
        // Every family imported in Blender, its models' centres collected (no build), centres.json written.
        let exe = match blender() {
            Ok(exe) => exe,
            Err(err) => return exit(err),
        };
        return if echo(&run(&exe, &format!("{here}/gamekit/centres.py"), &[])) { 0 } else { 1 };
    }
    let mut names: Vec<String> = argv.iter().filter(|a| !a.starts_with('-')).cloned().collect();
    let only = match only {
        Some(Some(id)) => {
            if let Some(i) = names.iter().position(|n| *n == id) {
                names.remove(i);
            }
            Some(id)
        }
        Some(None) => return exit("--only names a model: --only <id>[,<id>…]".into()),
        None => None,
    };
    if has("--all") {
        names = families(&here);
    }
    if names.is_empty() {
        return exit(format!("name a family ({}) or --all", families(&here).join(", ")));
    }
    let mut passthrough: Vec<String> = ["--preview", "--no-pack"].iter().filter(|f| has(f)).map(|f| f.to_string()).collect();
    if let Some(id) = only {
        passthrough.extend(["--only".to_string(), id]);
    }
    let exe = match blender() {
        Ok(exe) => exe,
        Err(err) => return exit(err),
    };
    let (mut built, mut failed) = (Vec::new(), Vec::new());
    let made = regex::Regex::new(r"(?m)^\[gamekit\] built (\S+)").unwrap();
    for name in &names {
        let script = format!("{here}/families/{name}.py");
        if !Path::new(&script).exists() {
            return exit(format!("no family {} ({})", checkglb::py_repr_str(name), families(&here).join(", ")));
        }
        let run = run(&exe, &script, &passthrough);
        if !echo(&run) {
            failed.push(name.clone());
        }
        built.extend(made.captures_iter(&run.stdout).map(|c| c[1].to_string()));
    }
    if !built.is_empty() {
        // (The gate takes the keys among its arguments: model., item., tool.; none, and it checks every model.)
        let keys: Vec<String> = built.into_iter().filter(|k| k.starts_with("model.") || k.starts_with("item.") || k.starts_with("tool.")).collect();
        let report = checkglb::gate(&paths, &keys);
        print!("{}", report.text);
        if report.failed > 0 {
            failed.push("check-glb".into());
        }
        println!("Exports are not byte-for-byte repeatable: commit only the models you meant to change (git checkout the others).");
    }
    if !failed.is_empty() {
        return exit(format!("failed: {}", failed.join(", ")));
    }
    0
}
