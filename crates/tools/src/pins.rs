//! `cargo pins`: accepted parts of the game held by their SHA-256, so work on something else can't change
//! them by the way (docs/design/living-town.md, Pins; was `tools/pins/check.ts`). Each pin in
//! `tools/pins/pins.json` names a file and, optionally, a part of it:
//!
//! - `region`: the lines between `pin:<region> begin` and `pin:<region> end` comments (exclusive);
//! - `const`: an `export const <NAME>` statement, from its line to where its brackets close.
//!
//! ```text
//! cargo pins                      check every pin (exit 1 on a difference)
//! cargo pins --list               every pin and whether it holds
//! cargo pins --update <name> --by <who> --reason <why>
//!                                 take a pin's present content as accepted
//! cargo pins --add <name> --file <path> [--region <r> | --const <NAME>] --by <who> --reason <why>
//! ```
//!
//! Only the integrator updates or adds pins; every update is listed in the run's end report.

use crate::download::sha256_hex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const USAGE: &str = "usage: cargo pins [--list]
       cargo pins --update <name> --by <who> --reason <why>
       cargo pins --add <name> --file <path> [--region <r> | --const <NAME>] --by <who> --reason <why>";

/// A pin, its fields in the order the file keeps them.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pin {
    pub name: String,
    pub file: String,
    pub sha256: String,
    pub by: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, rename = "const", skip_serializing_if = "Option::is_none")]
    pub constant: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct File {
    pins: Vec<Pin>,
}

fn word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

/// The pinned text of a file's content, or what is wrong with the pin.
pub fn pinned_text(
    content: &str,
    region: Option<&str>,
    constant: Option<&str>,
) -> Result<String, String> {
    if let Some(region) = region.filter(|r| !r.is_empty()) {
        let lines: Vec<&str> = content.split('\n').collect();
        let begin = lines
            .iter()
            .position(|l| l.contains(&format!("pin:{region} begin")));
        let end = lines
            .iter()
            .position(|l| l.contains(&format!("pin:{region} end")));
        return match (begin, end) {
            (Some(begin), Some(end)) if end >= begin => Ok(lines[begin + 1..end].join("\n")),
            _ => Err(format!(
                "region {region}'s begin and end markers are missing"
            )),
        };
    }
    let Some(name) = constant.filter(|c| !c.is_empty()) else {
        return Ok(content.to_string());
    };
    // `export const <NAME>` at a line's start, the name ending there.
    let head = format!("export const {name}");
    let start = content
        .match_indices(&head)
        .map(|(at, _)| at)
        .find(|&at| {
            (at == 0 || content.as_bytes()[at - 1] == b'\n')
                && !content[at + head.len()..].chars().next().is_some_and(word)
        })
        .ok_or_else(|| format!("no `export const {name}`"))?;
    let mut depth = 0i32;
    let mut opened = false;
    for (i, c) in content[start..].char_indices() {
        let at = start + i;
        let next = at + c.len_utf8();
        match c {
            '(' | '{' | '[' => {
                depth += 1;
                opened = true;
            }
            ')' | '}' | ']' => depth -= 1,
            ';' if depth == 0 => return Ok(content[start..next].to_string()),
            _ => {}
        }
        if opened && depth == 0 && content[next..].starts_with('\n') && c != ',' {
            // A statement without its semicolon ends where its brackets close at a line's end:
            // unless the next line goes on with it (`.`, `;`, `?`, `)` or `as`).
            let rest = content[next + 1..].trim_start();
            let goes_on = rest.starts_with(['.', ';', '?', ')'])
                || (rest.starts_with("as") && !rest[2..].chars().next().is_some_and(word));
            if !goes_on {
                return Ok(content[start..next].to_string());
            }
        }
    }
    Err(format!("`export const {name}` never closes"))
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn hash_of(root: &Path, pin: &Pin) -> Result<String, String> {
    let content = std::fs::read_to_string(root.join(&pin.file))
        .map_err(|_| format!("{} is missing", pin.file))?;
    pinned_text(&content, pin.region.as_deref(), pin.constant.as_deref())
        .map(|text| sha256_hex(text.as_bytes()))
}

/// Every pin against the tree at `root`: the lines to say, and how many pins don't hold.
pub fn check(root: &Path, pins: &[Pin], list: bool) -> (Vec<String>, usize) {
    let mut lines = Vec::new();
    let mut broken = 0;
    for pin in pins {
        let hash = hash_of(root, pin);
        let ok = hash.as_deref() == Ok(pin.sha256.as_str());
        if !ok {
            broken += 1;
        }
        if list || !ok {
            let what = match (&pin.region, &pin.constant) {
                (Some(region), _) if !region.is_empty() => {
                    format!("{} (region {region})", pin.file)
                }
                (_, Some(constant)) if !constant.is_empty() => {
                    format!("{} ({constant})", pin.file)
                }
                _ => pin.file.clone(),
            };
            let why = match (&hash, ok) {
                (_, true) => "holds".to_string(),
                (Ok(_), false) => "CHANGED".to_string(),
                (Err(e), false) => format!("BROKEN: {e}"),
            };
            lines.push(format!(
                "{}{}: {what}: {why}",
                if ok { "  " } else { "✗ " },
                pin.name
            ));
        }
    }
    (lines, broken)
}

fn run(args: &[&str]) -> Result<i32, String> {
    let root = root();
    let path = root.join("tools/pins/pins.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut file: File = serde_json::from_str(&text).map_err(|e| format!("pins.json: {e}"))?;
    let arg = |name: &str| -> Option<&str> {
        args.iter()
            .position(|a| *a == format!("--{name}"))
            .and_then(|i| args.get(i + 1))
            .copied()
    };
    let need = |name: &str| arg(name).ok_or_else(|| format!("--{name} is needed"));
    let (update, add) = (arg("update"), arg("add"));
    if update.is_some() || add.is_some() {
        let (by, reason) = (need("by")?, need("reason")?);
        if let Some(add) = add {
            if file.pins.iter().any(|p| p.name == add) {
                return Err(format!("pin {add} exists: use --update"));
            }
            file.pins.push(Pin {
                name: add.to_string(),
                file: need("file")?.to_string(),
                sha256: String::new(),
                by: by.to_string(),
                reason: reason.to_string(),
                region: arg("region").map(String::from),
                constant: arg("const").map(String::from),
            });
        }
        let name = update.or(add).expect("one of them");
        let pin = file
            .pins
            .iter_mut()
            .find(|p| p.name == name)
            .ok_or_else(|| format!("no pin {name}"))?;
        let hash = hash_of(&root, pin).map_err(|e| format!("{}: {e}", pin.name))?;
        pin.sha256 = hash.clone();
        pin.by = by.to_string();
        pin.reason = reason.to_string();
        let out = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
        std::fs::write(&path, format!("{out}\n"))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        println!(
            "{} {name}: {}",
            if add.is_some() { "Pinned" } else { "Updated" },
            &hash[..12]
        );
        return Ok(0);
    }
    let (lines, broken) = check(&root, &file.pins, args.contains(&"--list"));
    for line in lines {
        println!("{line}");
    }
    if broken > 0 {
        eprintln!(
            "\n{broken} pinned part(s) changed. Pinned parts are accepted work: put them back, or have the integrator take the change with `cargo pins --update <name> --by <who> --reason <why>`."
        );
        return Ok(1);
    }
    println!("{} pins hold.", file.pins.len());
    Ok(0)
}

pub fn main_with(args: &[String]) -> i32 {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    if matches!(a.first(), Some(&"-h" | &"--help" | &"help")) {
        println!("{USAGE}");
        return 0;
    }
    match run(&a) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("pins: {e}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_region_is_the_lines_between_its_markers() {
        let text = "a\n// pin:x begin\nb\nc\n// pin:x end\nd";
        assert_eq!(pinned_text(text, Some("x"), None).unwrap(), "b\nc");
        assert!(pinned_text(text, Some("y"), None).is_err());
        assert!(pinned_text("// pin:x end\n// pin:x begin", Some("x"), None).is_err());
    }

    #[test]
    fn a_const_runs_to_where_it_closes() {
        let text = "export const AB = 1;\nexport const A = {\n  b: [1, 2],\n};\nconst after = 1;\n";
        assert_eq!(
            pinned_text(text, None, Some("A")).unwrap(),
            "export const A = {\n  b: [1, 2],\n};"
        );
        assert_eq!(
            pinned_text(text, None, Some("AB")).unwrap(),
            "export const AB = 1;"
        );
        assert!(pinned_text(text, None, Some("B")).is_err());
        // Without a semicolon it ends where its brackets close at a line's end, unless the next line goes on.
        let open = "export const F = tuned('f', {\n  a: 1,\n})\nexport const G = [\n  1,\n]\n  .map((x) => x) as const\nnext";
        assert_eq!(
            pinned_text(open, None, Some("F")).unwrap(),
            "export const F = tuned('f', {\n  a: 1,\n})"
        );
        assert_eq!(
            pinned_text(open, None, Some("G")).unwrap(),
            "export const G = [\n  1,\n]\n  .map((x) => x) as const"
        );
        assert!(pinned_text("export const H = (", None, Some("H")).is_err());
    }

    #[test]
    fn the_repo_s_pins_hold() {
        let root = root();
        let file: File = serde_json::from_str(
            &std::fs::read_to_string(root.join("tools/pins/pins.json")).unwrap(),
        )
        .unwrap();
        assert!(file.pins.len() >= 19);
        let (lines, broken) = check(&root, &file.pins, false);
        assert_eq!(broken, 0, "{}", lines.join("\n"));
    }

    #[test]
    fn the_file_is_written_as_it_is_kept() {
        let path = root().join("tools/pins/pins.json");
        let text = std::fs::read_to_string(path).unwrap();
        let file: File = serde_json::from_str(&text).unwrap();
        assert_eq!(
            format!("{}\n", serde_json::to_string_pretty(&file).unwrap()),
            text
        );
    }
}
