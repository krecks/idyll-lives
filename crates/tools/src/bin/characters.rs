//! `cargo characters`: the residents generator and its checks (was `tools/characters/prim/`).
//!
//!     cargo characters build [out]    # writes the set (default web/public/assets/characters), then checks it
//!     cargo characters check [dir]    # the set against the contract
//!     cargo characters diff <a> <b>   # the largest differences between two sets (`-v`: per part)

use std::path::PathBuf;
use std::process::ExitCode;

use ll_tools::characters::{build, check, default_out, diff};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = |i: usize| args.get(i).map(PathBuf::from).unwrap_or_else(default_out);
    match args.first().map(String::as_str) {
        Some("build") => {
            let out = dir(1);
            if let Err(e) = build::build(&out) {
                eprintln!("build: {e}");
                return ExitCode::FAILURE;
            }
            if check::check(&out) { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        }
        Some("check") => {
            if check::check(&dir(1)) { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        }
        Some("diff") => {
            let verbose = args.iter().any(|a| a == "-v");
            let paths: Vec<&String> = args[1..].iter().filter(|a| *a != "-v").collect();
            let [a, b] = paths[..] else {
                eprintln!("usage: cargo characters diff [-v] <a> <b>");
                return ExitCode::from(2);
            };
            match diff::diff(a.as_ref(), b.as_ref()) {
                Ok(r) => {
                    diff::print(&r, verbose);
                    if r.failures.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
                }
                Err(e) => {
                    eprintln!("diff: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!("usage: cargo characters build [out] | check [dir] | diff [-v] <a> <b>");
            ExitCode::from(2)
        }
    }
}
