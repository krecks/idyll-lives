//! The command: what `cargo check-glb` does with its arguments.
//!
//! Split from `checkglb.rs` (see `mod.rs`).

use super::*;

/// `cargo check-glb [keys…] [--keys] [--preview [--sheet]] [--validate] [--update-baseline]`, or
/// `--against` (`against::main`), or `--built` (`built::main`): the exit code.
pub fn main(args: &[String]) -> i32 {
    if args.iter().any(|a| a == "--against") {
        return against::main(args);
    }
    if args.iter().any(|a| a == "--built") {
        return built::main(args);
    }
    let paths = Paths::repo();
    if args.iter().any(|a| a == "--keys") {
        return match known_keys(&paths) {
            Ok(keys) => {
                let mut out = String::new();
                for k in keys {
                    out += &k;
                    out.push('\n');
                }
                print!("{out}");
                0
            }
            Err(err) => {
                eprintln!("cannot read the content or the manifests: {err}");
                1
            }
        };
    }
    let keys: Vec<String> = args
        .iter()
        .filter(|a| a.starts_with("model.") || a.starts_with("item.") || a.starts_with("tool."))
        .cloned()
        .collect();
    // The counted checks run over every file at once (`BatchCheck`), by name.
    let mut batch: Vec<(&'static str, BatchCheck)> = Vec::new();
    if args.iter().any(|a| a == "--validate") {
        batch.push(("validator", validate::check));
    }
    if args.iter().any(|a| a == "--update-baseline") {
        match update_baseline(&paths, &keys, &batch) {
            Ok(changes) if changes.is_empty() => println!("{}: nothing to lower", baseline::FILE),
            Ok(changes) => {
                for c in changes {
                    println!("{}: {c}", baseline::FILE);
                }
            }
            Err(err) => {
                eprintln!("cannot update {}: {err}", baseline::FILE);
                return 1;
            }
        }
    }
    let report = gate_with(&paths, &keys, &batch);
    print!("{}", report.text);
    let mut code = if report.failed > 0 { 1 } else { 0 };
    let has = |flag: &str| args.iter().any(|a| a == flag);
    if has("--preview") && !report.checked.is_empty() {
        // Draws each file as read to previews/<family>/rt_<name>.png, so export problems show.
        if preview::review(&paths, &report.checked, &report.files, has("--sheet")) != 0 {
            code = 1;
        }
    }
    code
}
