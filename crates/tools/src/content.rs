//! `cargo content`: moves catalog items into the Validate category and back (was `tools/content/validate.py`).
//!
//! Every edit keeps the files' own formatting (only the values it changes, and the lines it adds or removes,
//! differ), and a file is written only once its new text is built and parses (and says what was meant). Several
//! sessions edit these files: run it on a clean file, or commit what it touched right after (`git add -p`).
//!
//! The JSON is edited as text through a parser that keeps every value's span (byte offsets), never re-serialised.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};

pub const HELP: &str = r#"usage: content [-h] [--group GROUP] [--label LABEL] [--to TO] [--dry-run]
               [--content CONTENT] [--plan PLAN]
               {list,in,out} [ids ...]

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
        category and its icon, docs/objects/README.md).

    Options: --dry-run (print the diff, write nothing), --content <dir> (default
    web/public/content: base.json and its includes), --plan <dir> (default docs/objects/).

Free turning: whether a 1x1 item may turn freely comes from its category (`objectRules.freeRotation`
in base.json) unless the item says `"freeRotation": true|false`. Going into Validate, an item that
turns freely at home gets `"freeRotation": true` when Validate itself doesn't turn freely, so it keeps
turning while it's checked; signing off carries an item's own `"freeRotation"` back with it unchanged.

positional arguments:
  {list,in,out}
  ids

options:
  -h, --help         show this help message and exit
  --group GROUP
  --label LABEL
  --to TO
  --dry-run
  --content CONTENT
  --plan PLAN
"#;

/// The repo root (the Python script's `ROOT`).
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/tools sits two below the root")
        .to_path_buf()
}

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

/// Python's `str.isspace` (what `\s` and `strip()` match): Unicode White_Space plus U+001C..U+001F.
fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\x1c'..='\x1f').contains(&c)
}

fn py_strip(s: &str) -> &str {
    s.trim_matches(py_space)
}

/// Python's `str.splitlines()`.
fn py_splitlines(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut it = s.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        let brk = matches!(
            c,
            '\n' | '\r'
                | '\x0b'
                | '\x0c'
                | '\x1c'
                | '\x1d'
                | '\x1e'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if brk {
            out.push(&s[start..i]);
            let mut next = i + c.len_utf8();
            if c == '\r' && it.peek().map(|&(_, n)| n) == Some('\n') {
                it.next();
                next += 1;
            }
            start = next;
        }
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
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

/// `json.dumps(s)` for a string: `ensure_ascii`, so non-ASCII as `\uXXXX` (surrogate pairs above the BMP).
pub fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            ' '..='~' => out.push(c),
            _ => {
                let mut units = [0u16; 2];
                for u in c.encode_utf16(&mut units) {
                    let _ = write!(out, "\\u{u:04x}");
                }
            }
        }
    }
    out.push('"');
    out
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
        for line in py_splitlines(&text) {
            // A list instead: "- `a`, `b`: bathroom / sinks;".
            if let Some(m) = listed_re.captures(line) {
                let group = py_strip(&m[3]);
                for id in tick_re.captures_iter(&m[1]) {
                    homes.entry(id[1].to_string()).or_insert_with(|| {
                        (
                            py_strip(&m[2]).to_string(),
                            (!no_group(group)).then(|| group.to_string()),
                        )
                    });
                }
            }
            if !line.starts_with('|') {
                column = None;
                continue;
            }
            let cells: Vec<&str> = py_strip(line)
                .trim_matches('|')
                .split('|')
                .map(py_strip)
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
                let (cat, group) = (py_strip(home[0]), py_strip(home[1]));
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
        if py_space(c) || c == '_' || c == '-' {
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
    fn dumps_like_python() {
        let u = "\\u";
        assert_eq!(
            json_str("Caf\u{e9} \u{2615} \u{1f600} \"q\" \\ \n\x7f"),
            format!("\"Caf{u}00e9 {u}2615 {u}d83d{u}de00 \\\"q\\\" \\\\ \\n{u}007f\"")
        );
    }

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

    #[test]
    fn splitlines() {
        assert_eq!(py_splitlines("a\r\nb\rc\n\nd"), ["a", "b", "c", "", "d"]);
        assert_eq!(py_splitlines("a\n"), ["a"]);
    }
}
