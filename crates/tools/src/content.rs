//! `cargo content`: moves catalog items into the Validate category and back (was `tools/content/validate.py`).
//!
//! Every edit keeps the files' own formatting (only the values it changes, and the lines it adds or removes,
//! differ), and a file is written only once its new text is built and parses (and says what was meant). Several
//! sessions edit these files: run it on a clean file, or commit what it touched right after (`git add -p`).
//!
//! The JSON is edited as text through a parser that keeps every value's span (byte offsets), never re-serialised.

use std::collections::{HashMap, HashSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};

pub const HELP: &str = r#"usage: cargo content list|in|out [<id>...] [--group <group>] [--label <label>]
                     [--to <category>/<group>] [--dry-run] [--content <dir>] [--plan <dir>]

Moves catalog items into Validate and back (sign-off), by id, editing the content JSON as text.

Every edit keeps the files' own formatting (only the values it changes, and the lines it adds or
removes, differ), and a file is written only once its new text is built and parses (and says what
was meant). Several sessions edit these files: run it on a clean file, or commit what it touched
right after (`git add -p`).

    cargo content list
        Validate's groups, their items, and where each goes back to (the "Home" column).
    cargo content in <id>... --group <group> [--label "Label"]
        Into Validate › <group> (made if missing; "outside" when the items' home group is).
        Prints each item's home, for the category table's "Home" column.
    cargo content out <id>... [--to <category>/<group>]
    cargo content out --group <group> [--to <category>/<group>]
        Sign-off: back home. Without --to, each item goes back to its "Home" in the category
        tables (docs/objects/*.md: `living / seating`; `living / —` is the category with no group).
        Validate groups left empty are dropped; the last one leaving is reported (then remove the
        category and its icon).

    Options: --dry-run (print the diff, write nothing), --content <dir> (default
    web/public/content: base.json and its includes), --plan <dir> (default docs/objects/),
    -h/--help (this text).

Free turning: whether a 1x1 item may turn freely comes from its category (`objectRules.freeRotation`
in base.json) unless the item says `"freeRotation": true|false`. Going into Validate, an item that
turns freely at home gets `"freeRotation": true` when Validate itself doesn't turn freely, so it keeps
turning while it's checked; signing off carries an item's own `"freeRotation"` back with it unchanged.
"#;

/// A path as Python's `Path(s)` spells it: no `.` components, no doubled or trailing slashes.
pub fn py_path(s: &str) -> PathBuf {
    let out: PathBuf = Path::new(s)
        .components()
        .filter(|c| *c != std::path::Component::CurDir)
        .collect();
    if out.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        out
    }
}

/// `Path.read_text()`: UTF-8 with universal newlines (`\r\n` and `\r` read as `\n`).
fn read_text(path: &Path) -> Result<String, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(if raw.contains('\r') {
        raw.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        raw
    })
}

/// A JSON string literal (UTF-8 as it is, as the content files have it).
fn json_str(s: &str) -> String {
    serde_json::to_string(s).expect("a string serialises")
}

mod json;
use json::indent_at;
pub use json::{Node, Val, parse};

// --- The content files ----------------------------------------------------------------------------

/// `(start, end, replacement)`.
pub type Edit = (usize, usize, String);

pub struct File {
    pub path: PathBuf,
    pub text: String,
    pub tree: Node,
}

impl File {
    pub fn open(path: PathBuf) -> Result<File, String> {
        let text = read_text(&path)?;
        let tree = parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(File { path, text, tree })
    }

    fn array(&self, key: &str) -> &[Node] {
        self.tree.get(key).map_or(&[], |a| &a.items)
    }
}

/// Edits on one file's text, kept apart from the parsed tree so its nodes can stay borrowed.
pub struct Editor<'a> {
    pub text: &'a str,
    pub edits: Vec<Edit>,
}

impl<'a> Editor<'a> {
    pub fn new(text: &'a str) -> Self {
        Editor {
            text,
            edits: Vec::new(),
        }
    }

    pub fn replace(&mut self, node: &Node, text: &str) {
        self.edits.push((node.start, node.end, text.to_string()));
    }

    /// Sets `key` to `value` (JSON text), adding it after member `after` (or last) if missing.
    pub fn set_member(&mut self, obj: &Node, key: &str, value: &str, after: Option<&str>) {
        if let Some((_, _, v)) = obj.member(key) {
            self.edits.push((v.start, v.end, value.to_string()));
            return;
        }
        let anchor = after.and_then(|a| obj.member(a));
        let (ks, v) = match anchor {
            Some((_, ks, v)) => (ks, v),
            None => {
                let last = obj.members.last().expect("set_member on an empty object");
                (last.1, &last.2)
            }
        };
        let sep = if self.text[obj.start..obj.end].contains('\n') {
            format!(",\n{}", indent_at(self.text, ks))
        } else {
            ", ".to_string()
        };
        self.edits
            .push((v.end, v.end, format!("{sep}\"{key}\": {value}")));
    }

    pub fn remove_member(&mut self, obj: &Node, key: &str) {
        let Some((i, ks, v)) = obj.member(key) else {
            return;
        };
        if i > 0 {
            // From the end of the member before, through this one.
            self.edits
                .push((obj.members[i - 1].2.end, v.end, String::new()));
        } else if obj.members.len() > 1 {
            // The first: through to the next one's key.
            self.edits.push((ks, obj.members[1].1, String::new()));
        } else {
            self.edits.push((obj.start + 1, obj.end - 1, String::new()));
        }
    }

    /// Adds an element after the last, in the style of the ones there. (An empty array, where the
    /// Python script failed, gets the one-line form between its brackets.)
    pub fn append_item(&mut self, arr: &Node, one_line: &str, lines: impl Fn(&str) -> String) {
        let Some(last) = arr.items.last() else {
            self.edits
                .push((arr.start + 1, arr.end - 1, one_line.to_string()));
            return;
        };
        let multi = self.text[last.start..last.end].contains('\n');
        let ind = indent_at(self.text, last.start);
        let body = if multi {
            lines(ind)
        } else {
            one_line.to_string()
        };
        let sep = if self.text[arr.start..arr.end].contains('\n') {
            format!(",\n{ind}")
        } else {
            ", ".to_string()
        };
        self.edits
            .push((last.end, last.end, format!("{sep}{body}")));
    }

    pub fn remove_item(&mut self, arr: &Node, i: usize) {
        let node = &arr.items[i];
        if i > 0 {
            self.edits
                .push((arr.items[i - 1].end, node.end, String::new()));
        } else if arr.items.len() > 1 {
            self.edits
                .push((node.start, arr.items[1].start, String::new()));
        } else {
            self.edits.push((arr.start + 1, arr.end - 1, String::new()));
        }
    }

    /// The text with every edit applied, last first; overlapping edits are refused.
    pub fn new_text(&self, path: &Path) -> Result<String, String> {
        let mut edits: Vec<&Edit> = self.edits.iter().collect();
        // Stable, like Python's `sorted(..., reverse=True)`: equal spans keep their order.
        edits.sort_by_key(|e| std::cmp::Reverse((e.0, e.1)));
        let mut text = self.text.to_string();
        let mut last: Option<usize> = None;
        for (start, end, rep) in edits {
            if last.is_some_and(|l| *end > l) {
                return Err(format!("{}: overlapping edits", path.display()));
            }
            text.replace_range(*start..*end, rep);
            last = Some(*start);
        }
        Ok(text)
    }
}

pub fn load(content: &Path) -> Result<Vec<File>, String> {
    let base = File::open(content.join("base.json"))?;
    let data: serde_json::Value = serde_json::from_str(&base.text).map_err(|e| e.to_string())?;
    let mut files = vec![base];
    if let Some(inc) = data.get("include").and_then(|v| v.as_array()) {
        for name in inc {
            files.push(File::open(content.join(name.as_str().unwrap_or_default()))?);
        }
    }
    Ok(files)
}

/// Item id -> (category, group or None) from the category tables' "Home" (or "Back to") column.
pub fn back_to_table(plan: &Path) -> Result<HashMap<String, (String, Option<String>)>, String> {
    let listed_re =
        regex::Regex::new(r"^- ((?:`[^`]+`(?:, )?)+): ([\w ]+) / ([^;.]+)[;.]?$").unwrap();
    let tick_re = regex::Regex::new(r"`([^`]+)`").unwrap();
    let no_group = |g: &str| matches!(g, "—" | "-");
    let mut mds: Vec<PathBuf> = std::fs::read_dir(plan)
        .map_err(|e| format!("{}: {e}", plan.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".md"))
        })
        .collect();
    mds.sort();
    let mut homes = HashMap::new();
    for md in mds {
        let text = read_text(&md)?;
        let mut column: Option<usize> = None;
        for line in text.lines() {
            // A list instead: "- `a`, `b`: bathroom / sinks;".
            if let Some(m) = listed_re.captures(line) {
                let group = m[3].trim();
                for id in tick_re.captures_iter(&m[1]) {
                    homes.entry(id[1].to_string()).or_insert_with(|| {
                        (
                            m[2].trim().to_string(),
                            (!no_group(group)).then(|| group.to_string()),
                        )
                    });
                }
            }
            if !line.starts_with('|') {
                column = None;
                continue;
            }
            let cells: Vec<&str> = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect();
            if let Some(heading) = ["Home", "Back to"].into_iter().find(|h| cells.contains(h)) {
                column = cells.iter().position(|c| *c == heading);
                continue;
            }
            let Some(col) = column.filter(|&c| c < cells.len()) else {
                continue;
            };
            let id = tick_re.captures(cells[0]);
            let home: Vec<&str> = cells[col].split('/').collect();
            if let (Some(id), 2) = (id, home.len()) {
                let (cat, group) = (home[0].trim(), home[1].trim());
                homes.insert(
                    id[1].to_string(),
                    (
                        cat.to_string(),
                        (!(no_group(group) || group.is_empty())).then(|| group.to_string()),
                    ),
                );
            }
        }
    }
    Ok(homes)
}

/// "kids-play" -> "Kids Play".
pub fn label_of(group: &str) -> String {
    let mut parts = vec![String::new()];
    let mut prev_sep = false;
    for c in group.chars() {
        if c.is_whitespace() || c == '_' || c == '-' {
            if !prev_sep {
                parts.push(String::new());
            }
            prev_sep = true;
        } else {
            parts.last_mut().unwrap().push(c);
            prev_sep = false;
        }
    }
    parts
        .iter()
        .map(|w| {
            let mut cs = w.chars();
            match cs.next() {
                Some(f) => f.to_uppercase().chain(cs).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

mod cli;
pub use cli::{Args, Exit, main_with, parse_args, run};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(label_of("kids-play_zone"), "Kids Play Zone");
        assert_eq!(label_of("_x"), " X");
        assert_eq!(label_of("straße"), "Straße");
    }

    #[test]
    fn spans_are_bytes() {
        let t = "{\"a\": \"é\", \"b\": [1, 2.5e3, true]}";
        let n = parse(t).unwrap();
        let b = n.get("b").unwrap();
        assert_eq!(&t[b.start..b.end], "[1, 2.5e3, true]");
        assert_eq!(n.member("b").unwrap().1, t.find("\"b\"").unwrap());
    }

    #[test]
    fn edits_in_reverse_and_refuse_overlaps() {
        let t = "{\"a\": 1, \"b\": 2}";
        let n = parse(t).unwrap();
        let mut ed = Editor::new(t);
        ed.set_member(&n, "c", "3", Some("a"));
        ed.set_member(&n, "d", "4", Some("a"));
        ed.remove_member(&n, "b");
        assert_eq!(
            ed.new_text(Path::new("x")).unwrap(),
            "{\"a\": 1, \"d\": 4, \"c\": 3}"
        );
        ed.replace(n.get("a").unwrap(), "9");
        ed.replace(n.get("a").unwrap(), "8");
        assert!(ed.new_text(Path::new("x")).is_err());
    }
}
