//! The `--against` command line.

use super::*;
use std::path::Path;

/// `cargo check-glb --against <rev|dir> [keys… | --family <f>] [--dir <built>] [--accept <file>]`: the exit code.
pub fn main(args: &[String]) -> i32 {
    let root = crate::checkglb::repo_root();
    let mut opts = Options {
        root: root.clone(),
        old: Source::Tree,
        new: Source::Tree,
        keys: Vec::new(),
        family: None,
        accepted: None,
    };
    let mut args = args.iter();
    let mut held = false;
    while let Some(arg) = args.next() {
        let mut value = |what: &str| {
            args.next()
                .filter(|v| !v.starts_with("--"))
                .cloned()
                .ok_or_else(|| format!("{arg} takes {what}"))
        };
        let taken = match arg.as_str() {
            "--against" => value("a revision or a directory").and_then(|v| {
                held = true;
                // (A revision first: a directory of a revision's name is written ./name.)
                opts.old = if is_rev(&root, &v) {
                    Source::Rev(v)
                } else if Path::new(&v).is_dir() {
                    Source::Dir(v)
                } else {
                    return Err(format!(
                        "{v} is neither a revision git knows nor a directory"
                    ));
                };
                Ok(())
            }),
            "--dir" => value("a directory").map(|v| opts.new = Source::Dir(v)),
            "--family" => value("a family").map(|v| opts.family = Some(v)),
            "--accept" => value("a file").map(|v| opts.accepted = Some(v)),
            key if key.starts_with("model.")
                || key.starts_with("item.")
                || key.starts_with("tool.") =>
            {
                opts.keys.push(key.to_string());
                Ok(())
            }
            other => Err(format!(
                "{other}: --against takes keys (model., item., tool.), --family <f>, --dir <built> and --accept <file>"
            )),
        };
        if let Err(e) = taken {
            eprintln!("{e}");
            return 2;
        }
    }
    if !held {
        eprintln!("--against takes a revision or a directory");
        return 2;
    }
    if !opts.keys.is_empty() && opts.family.is_some() {
        eprintln!("name keys or a family, not both");
        return 2;
    }
    let started = std::time::Instant::now();
    match against(&opts) {
        Ok(outcome) => {
            print!("{}", outcome.text);
            eprintln!("compared in {:.1} s", started.elapsed().as_secs_f64());
            outcome.failed() as i32
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}
