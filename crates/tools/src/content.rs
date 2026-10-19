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
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).expect("crates/tools sits two below the root").to_path_buf()
}

/// A path as Python's `Path(s)` spells it: no `.` components, no doubled or trailing slashes.
pub fn py_path(s: &str) -> PathBuf {
    let out: PathBuf = Path::new(s).components().filter(|c| *c != std::path::Component::CurDir).collect();
    if out.as_os_str().is_empty() { PathBuf::from(".") } else { out }
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
        let brk = matches!(c, '\n' | '\r' | '\x0b' | '\x0c' | '\x1c' | '\x1d' | '\x1e' | '\u{85}' | '\u{2028}' | '\u{2029}');
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
    Ok(if raw.contains('\r') { raw.replace("\r\n", "\n").replace('\r', "\n") } else { raw })
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

// --- A JSON parser that keeps where every value is in the text ------------------------------------

/// What Python's `Node.value` held.
#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Str(String),
    Bool(bool),
    Null,
    /// The number's text (its value is never used).
    Num(String),
    Object,
    Array,
}

impl Val {
    /// `str(value)` as Python prints it, `None` for null.
    fn py(&self) -> Option<String> {
        match self {
            Val::Str(s) | Val::Num(s) => Some(s.clone()),
            Val::Bool(b) => Some(if *b { "True" } else { "False" }.into()),
            Val::Null | Val::Object | Val::Array => None,
        }
    }

    fn truthy(&self) -> bool {
        match self {
            Val::Str(s) => !s.is_empty(),
            Val::Bool(b) => *b,
            Val::Num(s) => s.parse::<f64>().is_ok_and(|v| v != 0.0),
            Val::Null | Val::Object | Val::Array => false,
        }
    }
}

/// A JSON value and its span (byte offsets) in the text; objects keep their members' key starts.
#[derive(Debug)]
pub struct Node {
    pub start: usize,
    pub end: usize,
    pub value: Val,
    /// `(key, key start, value)` for objects.
    pub members: Vec<(String, usize, Node)>,
    pub items: Vec<Node>,
}

impl Node {
    fn leaf(start: usize, end: usize, value: Val) -> Node {
        Node { start, end, value, members: Vec::new(), items: Vec::new() }
    }

    pub fn get(&self, key: &str) -> Option<&Node> {
        self.members.iter().find(|m| m.0 == key).map(|m| &m.2)
    }

    /// `(index, key start, value)` of member `key`.
    pub fn member(&self, key: &str) -> Option<(usize, usize, &Node)> {
        self.members.iter().enumerate().find(|(_, m)| m.0 == key).map(|(i, m)| (i, m.1, &m.2))
    }

    /// The value of member `key` as Python's `node.get(key).value` printed (`None` when missing or null).
    fn val(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|n| n.value.py())
    }
}

struct Parser<'a> {
    t: &'a str,
}

impl Parser<'_> {
    /// The offset as Python (code points) would report it.
    fn cp(&self, i: usize) -> usize {
        self.t[..i.min(self.t.len())].chars().count()
    }

    fn ws(&self, mut i: usize) -> usize {
        while let Some(c) = self.t[i..].chars().next() {
            if !py_space(c) {
                break;
            }
            i += c.len_utf8();
        }
        i
    }

    fn at(&self, i: usize) -> Result<u8, String> {
        self.t.as_bytes().get(i).copied().ok_or_else(|| "string index out of range".to_string())
    }

    /// The end of the string literal at `i` (`"(?:[^"\\]|\\.)*"`), if there is one.
    fn string_end(&self, i: usize) -> Option<usize> {
        let b = self.t.as_bytes();
        if b.get(i) != Some(&b'"') {
            return None;
        }
        let mut j = i + 1;
        while j < b.len() {
            match b[j] {
                b'"' => return Some(j + 1),
                b'\\' => {
                    let c = self.t[j + 1..].chars().next()?;
                    j += 1 + c.len_utf8();
                }
                _ => j += 1,
            }
        }
        None
    }

    fn decode(&self, a: usize, b: usize) -> Result<String, String> {
        serde_json::from_str::<String>(&self.t[a..b]).map_err(|e| format!("bad string at {}: {e}", self.cp(a)))
    }

    fn value(&self, i: usize) -> Result<Node, String> {
        let i = self.ws(i);
        let c = self.at(i)?;
        if c == b'{' {
            let mut members = Vec::new();
            let mut j = self.ws(i + 1);
            if self.at(j)? == b'}' {
                return Ok(Node { start: i, end: j + 1, value: Val::Object, members, items: Vec::new() });
            }
            loop {
                j = self.ws(j);
                let m = self.string_end(j).ok_or_else(|| format!("a key expected at {}", self.cp(j)))?;
                let key = self.decode(j, m)?;
                let ks = j;
                j = self.ws(m);
                if self.at(j)? != b':' {
                    return Err(format!("\":\" expected at {}", self.cp(j)));
                }
                let v = self.value(j + 1)?;
                j = self.ws(v.end);
                members.push((key, ks, v));
                if self.at(j)? == b',' {
                    j += 1;
                    continue;
                }
                if self.at(j)? != b'}' {
                    return Err(format!("\"}}\" expected at {}", self.cp(j)));
                }
                return Ok(Node { start: i, end: j + 1, value: Val::Object, members, items: Vec::new() });
            }
        }
        if c == b'[' {
            let mut items = Vec::new();
            let mut j = self.ws(i + 1);
            if self.at(j)? == b']' {
                return Ok(Node { start: i, end: j + 1, value: Val::Array, members: Vec::new(), items });
            }
            loop {
                let v = self.value(j)?;
                j = self.ws(v.end);
                items.push(v);
                if self.at(j)? == b',' {
                    j += 1;
                    continue;
                }
                if self.at(j)? != b']' {
                    return Err(format!("\"]\" expected at {}", self.cp(j)));
                }
                return Ok(Node { start: i, end: j + 1, value: Val::Array, members: Vec::new(), items });
            }
        }
        if c == b'"' {
            let m = self.string_end(i).ok_or_else(|| format!("a string expected at {}", self.cp(i)))?;
            return Ok(Node::leaf(i, m, Val::Str(self.decode(i, m)?)));
        }
        for (word, v) in [("true", Val::Bool(true)), ("false", Val::Bool(false)), ("null", Val::Null)] {
            if self.t[i..].starts_with(word) {
                return Ok(Node::leaf(i, i + word.len(), v));
            }
        }
        let end = self.number_end(i).ok_or_else(|| format!("a value expected at {}", self.cp(i)))?;
        Ok(Node::leaf(i, end, Val::Num(self.t[i..end].to_string())))
    }

    /// `-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?` at `i`.
    fn number_end(&self, i: usize) -> Option<usize> {
        let b = self.t.as_bytes();
        let digits = |mut j: usize| {
            let s = j;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            (j > s).then_some(j)
        };
        let mut j = i;
        if b.get(j) == Some(&b'-') {
            j += 1;
        }
        match b.get(j) {
            Some(b'0') => j += 1,
            Some(b'1'..=b'9') => j = digits(j)?,
            _ => return None,
        }
        if b.get(j) == Some(&b'.')
            && let Some(k) = digits(j + 1)
        {
            j = k;
        }
        if matches!(b.get(j), Some(b'e' | b'E')) {
            let mut k = j + 1;
            if matches!(b.get(k), Some(b'+' | b'-')) {
                k += 1;
            }
            if let Some(k) = digits(k) {
                j = k;
            }
        }
        Some(j)
    }
}

pub fn parse(text: &str) -> Result<Node, String> {
    Parser { t: text }.value(0)
}

/// The whitespace before `pos` on its line, or "" if anything else precedes it there.
fn indent_at(text: &str, pos: usize) -> &str {
    let line = text[..pos].rfind('\n').map_or(0, |i| i + 1);
    let s = &text[line..pos];
    if py_strip(s).is_empty() { s } else { "" }
}

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
        Editor { text, edits: Vec::new() }
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
        let sep = if self.text[obj.start..obj.end].contains('\n') { format!(",\n{}", indent_at(self.text, ks)) } else { ", ".to_string() };
        self.edits.push((v.end, v.end, format!("{sep}\"{key}\": {value}")));
    }

    pub fn remove_member(&mut self, obj: &Node, key: &str) {
        let Some((i, ks, v)) = obj.member(key) else { return };
        if i > 0 {
            // From the end of the member before, through this one.
            self.edits.push((obj.members[i - 1].2.end, v.end, String::new()));
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
            self.edits.push((arr.start + 1, arr.end - 1, one_line.to_string()));
            return;
        };
        let multi = self.text[last.start..last.end].contains('\n');
        let ind = indent_at(self.text, last.start);
        let body = if multi { lines(ind) } else { one_line.to_string() };
        let sep = if self.text[arr.start..arr.end].contains('\n') { format!(",\n{ind}") } else { ", ".to_string() };
        self.edits.push((last.end, last.end, format!("{sep}{body}")));
    }

    pub fn remove_item(&mut self, arr: &Node, i: usize) {
        let node = &arr.items[i];
        if i > 0 {
            self.edits.push((arr.items[i - 1].end, node.end, String::new()));
        } else if arr.items.len() > 1 {
            self.edits.push((node.start, arr.items[1].start, String::new()));
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
    let listed_re = regex::Regex::new(r"^- ((?:`[^`]+`(?:, )?)+): ([\w ]+) / ([^;.]+)[;.]?$").unwrap();
    let tick_re = regex::Regex::new(r"`([^`]+)`").unwrap();
    let no_group = |g: &str| matches!(g, "—" | "-");
    let mut mds: Vec<PathBuf> = std::fs::read_dir(plan)
        .map_err(|e| format!("{}: {e}", plan.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(".md")))
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
                    homes.entry(id[1].to_string()).or_insert_with(|| (py_strip(&m[2]).to_string(), (!no_group(group)).then(|| group.to_string())));
                }
            }
            if !line.starts_with('|') {
                column = None;
                continue;
            }
            let cells: Vec<&str> = py_strip(line).trim_matches('|').split('|').map(py_strip).collect();
            if let Some(heading) = ["Home", "Back to"].into_iter().find(|h| cells.contains(h)) {
                column = cells.iter().position(|c| *c == heading);
                continue;
            }
            let Some(col) = column.filter(|&c| c < cells.len()) else { continue };
            let id = tick_re.captures(cells[0]);
            let home: Vec<&str> = cells[col].split('/').collect();
            if let (Some(id), 2) = (id, home.len()) {
                let (cat, group) = (py_strip(home[0]), py_strip(home[1]));
                homes.insert(id[1].to_string(), (cat.to_string(), (!(no_group(group) || group.is_empty())).then(|| group.to_string())));
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

// --- The command line (argparse's rules, as far as they matter here) -------------------------------

#[derive(Debug, Default)]
pub struct Args {
    pub what: String,
    pub ids: Vec<String>,
    pub group: Option<String>,
    pub label: Option<String>,
    pub to: Option<String>,
    pub dry_run: bool,
    pub content: Option<String>,
    pub plan: Option<String>,
}

/// A run's end: what to print to stderr and the exit code.
#[derive(Debug)]
pub struct Exit {
    pub code: i32,
    pub msg: String,
}

fn usage_error(msg: &str) -> Exit {
    let usage = HELP.split("\n\n").next().unwrap();
    Exit { code: 2, msg: format!("{usage}\ncontent: error: {msg}") }
}

fn fail(msg: impl Into<String>) -> Exit {
    Exit { code: 1, msg: msg.into() }
}

enum Arg {
    Positional,
    /// A known option (by its full name) and its `=value`.
    Opt(&'static str, Option<String>),
    Unknown,
}

/// argparse's `_parse_optional`: exact name, `name=value`, unique prefix, negative number, space.
fn classify(s: &str) -> Result<Arg, Exit> {
    const OPTS: [&str; 8] = ["-h", "--help", "--group", "--label", "--to", "--dry-run", "--content", "--plan"];
    if !s.starts_with('-') || s == "-" {
        return Ok(Arg::Positional);
    }
    let full = |o: &'static str| if o == "-h" { "--help" } else { o };
    if let Some(o) = OPTS.iter().find(|o| **o == s) {
        return Ok(Arg::Opt(full(o), None));
    }
    let (name, explicit) = match s.split_once('=') {
        Some((n, v)) => (n, Some(v.to_string())),
        None => (s, None),
    };
    if let Some(o) = OPTS.iter().find(|o| **o == name) {
        return Ok(Arg::Opt(full(o), explicit));
    }
    if name.starts_with("--") {
        let hits: Vec<&'static str> = OPTS.iter().copied().filter(|o| o.starts_with(name)).collect();
        match hits.len() {
            0 => {}
            1 => return Ok(Arg::Opt(hits[0], explicit)),
            _ => return Err(usage_error(&format!("ambiguous option: {name} could match {}", hits.join(", ")))),
        }
    }
    // `^-\d+$|^-\d*\.\d+$`
    let num = &s[1..];
    let digits = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
    let negative = digits(num) || num.split_once('.').is_some_and(|(a, b)| (a.is_empty() || digits(a)) && digits(b));
    if negative || s.contains(' ') {
        return Ok(Arg::Positional);
    }
    Ok(Arg::Unknown)
}

pub fn parse_args(argv: &[String]) -> Result<Args, Exit> {
    let mut a = Args::default();
    let mut chunks: Vec<Vec<String>> = Vec::new();
    let mut extras: Vec<String> = Vec::new();
    let mut in_chunk = false;
    let mut i = 0;
    let mut rest = false;
    while i < argv.len() {
        let s = &argv[i];
        i += 1;
        let kind = if rest || s == "--" { Arg::Positional } else { classify(s)? };
        if s == "--" && !rest {
            rest = true;
            in_chunk = false;
            continue;
        }
        let opt = match kind {
            Arg::Positional => {
                if !in_chunk {
                    chunks.push(Vec::new());
                    in_chunk = true;
                }
                chunks.last_mut().unwrap().push(s.clone());
                continue;
            }
            Arg::Unknown => {
                in_chunk = false;
                extras.push(s.clone());
                continue;
            }
            Arg::Opt(o, explicit) => {
                in_chunk = false;
                (o, explicit)
            }
        };
        let (opt, explicit) = opt;
        match opt {
            "--help" | "--dry-run" => {
                if let Some(v) = explicit {
                    return Err(usage_error(&format!("argument {}: ignored explicit argument '{v}'", if opt == "--help" { "-h/--help" } else { opt })));
                }
                if opt == "--help" {
                    return Err(Exit { code: 0, msg: String::new() });
                }
                a.dry_run = true;
            }
            _ => {
                let v = match explicit {
                    Some(v) => v,
                    None => match argv.get(i) {
                        Some(v) if !rest && v != "--" && matches!(classify(v)?, Arg::Positional) => {
                            i += 1;
                            v.clone()
                        }
                        _ => return Err(usage_error(&format!("argument {opt}: expected one argument"))),
                    },
                };
                let slot = match opt {
                    "--group" => &mut a.group,
                    "--label" => &mut a.label,
                    "--to" => &mut a.to,
                    "--content" => &mut a.content,
                    _ => &mut a.plan,
                };
                *slot = Some(v);
            }
        }
    }
    // `what` takes the first positional; `ids` the rest of its run of positionals, else the next run.
    let mut ids_done = false;
    for (k, chunk) in chunks.into_iter().enumerate() {
        if k == 0 {
            let what = &chunk[0];
            if !["list", "in", "out"].contains(&what.as_str()) {
                return Err(usage_error(&format!("argument what: invalid choice: '{what}' (choose from list, in, out)")));
            }
            a.what = what.clone();
            if chunk.len() > 1 {
                a.ids = chunk[1..].to_vec();
                ids_done = true;
            }
        } else if !ids_done {
            a.ids = chunk;
            ids_done = true;
        } else {
            extras.extend(chunk);
        }
    }
    if a.what.is_empty() {
        return Err(usage_error("the following arguments are required: what"));
    }
    if !extras.is_empty() {
        return Err(usage_error(&format!("unrecognized arguments: {}", extras.join(" "))));
    }
    Ok(a)
}

/// An insertion-ordered map with Python dict semantics (a re-assigned key keeps its place).
struct Ordered<V> {
    keys: Vec<String>,
    map: HashMap<String, V>,
}

impl<V> Ordered<V> {
    fn new() -> Self {
        Ordered { keys: Vec::new(), map: HashMap::new() }
    }

    fn insert(&mut self, k: String, v: V) {
        if self.map.insert(k.clone(), v).is_none() {
            self.keys.push(k);
        }
    }

    fn iter(&self) -> impl Iterator<Item = (&String, &V)> {
        self.keys.iter().map(|k| (k, &self.map[k]))
    }
}

fn opt(s: &Option<String>) -> String {
    s.clone().unwrap_or_else(|| "None".into())
}

fn dash(s: &Option<String>) -> String {
    s.clone().unwrap_or_else(|| "—".into())
}

/// Runs the tool; stdout goes to `out`.
pub fn run(a: &Args, out: &mut dyn std::io::Write) -> Result<(), Exit> {
    let root = root();
    let content = a.content.as_deref().map_or_else(|| root.join("web/public/content"), py_path);
    let plan = a.plan.as_deref().map_or_else(|| root.join("docs/objects"), py_path);
    let files = load(&content).map_err(fail)?;
    let mut eds: Vec<Editor> = files.iter().map(|f| Editor::new(&f.text)).collect();
    let mut by_id: Ordered<(usize, &Node)> = Ordered::new();
    for (fi, f) in files.iter().enumerate() {
        for o in f.array("objects") {
            if let Some(id) = o.get("id") {
                by_id.insert(id.value.py().unwrap_or_else(|| "None".into()), (fi, o));
            }
        }
    }
    let mut cats: Ordered<(usize, &Node)> = Ordered::new();
    for (fi, f) in files.iter().enumerate() {
        for c in f.array("buyCategories") {
            cats.insert(c.val("id").unwrap_or_else(|| "None".into()), (fi, c));
        }
    }
    let Some(&(vfile, vcat)) = cats.map.get("validate") else {
        return Err(fail("validate: there is no Validate category in the content"));
    };
    let vgroups = vcat.get("groups");
    let vitems: &[Node] = vgroups.map_or(&[], |g| &g.items);
    let where_ = |o: &Node| (o.val("category"), o.val("group"));
    let groups_of = |cat: &str| -> Vec<(String, &Node)> {
        cats.map.get(cat).and_then(|(_, c)| c.get("groups")).map_or(Vec::new(), |arr| arr.items.iter().map(|g| (g.val("id").unwrap_or_else(|| "None".into()), g)).collect())
    };
    let homes = if plan.is_dir() { back_to_table(&plan).map_err(fail)? } else { HashMap::new() };
    let base_data: serde_json::Value = serde_json::from_str(&files[0].text).map_err(|e| fail(e.to_string()))?;
    let free_cats: HashSet<String> = base_data
        .get("objectRules")
        .and_then(|r| r.get("freeRotation"))
        .and_then(|f| f.as_array())
        .map_or(HashSet::new(), |a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect());
    let w = |out: &mut dyn std::io::Write, s: String| {
        let _ = writeln!(out, "{s}");
    };

    if a.what == "list" {
        let mut in_validate: Ordered<Vec<String>> = Ordered::new();
        for (id, (_, o)) in by_id.iter() {
            let (cat, group) = where_(o);
            if cat.as_deref() == Some("validate") {
                let g = opt(&group);
                if !in_validate.map.contains_key(&g) {
                    in_validate.insert(g.clone(), Vec::new());
                }
                in_validate.map.get_mut(&g).unwrap().push(id.clone());
            }
        }
        for g in vitems {
            let gid = opt(&g.val("id"));
            let items = in_validate.map.remove(&gid).unwrap_or_default();
            in_validate.keys.retain(|k| *k != gid);
            w(out, format!("{gid} ({})", items.len()));
            for id in items {
                let home = match homes.get(&id) {
                    Some((c, g)) => format!("{c} / {}", dash(g)),
                    None => "(no Home in docs/objects/)".into(),
                };
                w(out, format!("  {id}  ->  {home}"));
            }
        }
        for (gid, items) in in_validate.iter() {
            w(out, format!("{gid} (not a Validate group!): {}", items.join(", ")));
        }
        return Ok(());
    }

    let mut ids: Vec<String> = a.ids.clone();
    if a.what == "out"
        && let Some(group) = &a.group
    {
        for (id, (_, o)) in by_id.iter() {
            if where_(o) == (Some("validate".into()), Some(group.clone())) && !ids.contains(id) {
                ids.push(id.clone());
            }
        }
    }
    if ids.is_empty() {
        return Err(fail("validate: which items? (ids, or --group with out)"));
    }
    let missing: Vec<&str> = ids.iter().filter(|i| !by_id.map.contains_key(*i)).map(|s| s.as_str()).collect();
    if !missing.is_empty() {
        return Err(fail(format!("validate: no such item: {}", missing.join(", "))));
    }

    let mut expect: HashMap<String, (String, Option<String>)> = HashMap::new();
    if a.what == "in" {
        let Some(group) = &a.group else { return Err(fail("validate: in needs --group <validate group>")) };
        let existing: Vec<String> = groups_of("validate").into_iter().map(|(k, _)| k).collect();
        let mut outside = false;
        for id in &ids {
            let (fi, o) = by_id.map[id];
            let (cat, g) = where_(o);
            if cat.as_deref() == Some("validate") {
                return Err(fail(format!("validate: {id} is in Validate already ({})", opt(&g))));
            }
            let Some(cat) = cat.clone().filter(|c| cats.map.contains_key(c)) else {
                return Err(fail(format!("validate: {id} has no known category ({})", opt(&cat))));
            };
            let home_group = g.as_ref().and_then(|g| groups_of(&cat).into_iter().find(|(k, _)| k == g).map(|(_, n)| n));
            outside |= home_group.and_then(|h| h.get("outside")).is_some_and(|n| n.value.truthy());
            let ed = &mut eds[fi];
            ed.replace(o.get("category").unwrap(), "\"validate\"");
            ed.set_member(o, "group", &json_str(group), Some("category"));
            if o.get("freeRotation").is_none() && free_cats.contains(&cat) && !free_cats.contains("validate") {
                ed.set_member(o, "freeRotation", "true", Some("category"));
            }
            w(out, format!("{id}: {cat} / {} -> validate / {group}", dash(&g)));
        }
        if !existing.contains(group) {
            let label = json_str(&a.label.clone().unwrap_or_else(|| label_of(group)));
            let gid = json_str(group);
            let extra = if outside { ", \"outside\": true" } else { "" };
            let Some(vgroups) = vgroups else { return Err(fail("validate: the Validate category has no \"groups\"")) };
            eds[vfile].append_item(vgroups, &format!("{{\"id\": {gid}, \"label\": {label}{extra}}}"), |ind| {
                format!("{{\n{ind}  \"id\": {gid},\n{ind}  \"label\": {label}{}\n{ind}}}", if outside { format!(",\n{ind}  \"outside\": true") } else { String::new() })
            });
            w(out, format!("new Validate group: {group}{}", if outside { " (outside)" } else { "" }));
        }
        for id in &ids {
            expect.insert(id.clone(), ("validate".into(), Some(group.clone())));
        }
    } else {
        let mut emptied: Vec<Option<String>> = Vec::new();
        let plan_s = plan.display().to_string();
        for id in &ids {
            let (fi, o) = by_id.map[id];
            let (cat, group) = where_(o);
            if cat.as_deref() != Some("validate") {
                return Err(fail(format!("validate: {id} is not in Validate ({} / {})", opt(&cat), opt(&group))));
            }
            let home = if let Some(to) = &a.to {
                let (c, g) = to.split_once('/').unwrap_or((to, ""));
                let g = py_strip(g);
                (py_strip(c).to_string(), (!g.is_empty()).then(|| g.to_string()))
            } else if let Some(h) = homes.get(id) {
                h.clone()
            } else {
                return Err(fail(format!("validate: no \"Home\" for {id} in {plan_s}: give --to <category>/<group>")));
            };
            if !cats.map.contains_key(&home.0) {
                return Err(fail(format!("validate: {id}: no category \"{}\"", home.0)));
            }
            if let Some(hg) = &home.1 {
                let groups = groups_of(&home.0);
                if !groups.iter().any(|(k, _)| k == hg) {
                    let names: Vec<&str> = groups.iter().map(|(k, _)| k.as_str()).collect();
                    return Err(fail(format!("validate: {id}: category \"{}\" has no group \"{hg}\" ({})", home.0, names.join(", "))));
                }
            }
            let ed = &mut eds[fi];
            ed.replace(o.get("category").unwrap(), &json_str(&home.0));
            match &home.1 {
                None => ed.remove_member(o, "group"),
                Some(hg) => ed.set_member(o, "group", &json_str(hg), Some("category")),
            }
            if !emptied.contains(&group) {
                emptied.push(group.clone());
            }
            w(out, format!("{id}: validate / {} -> {} / {}", opt(&group), home.0, dash(&home.1)));
            expect.insert(id.clone(), home);
        }
        // Validate groups nothing is left in.
        let still: HashSet<Option<String>> = by_id.iter().filter(|(i, (_, o))| !expect.contains_key(*i) && where_(o).0.as_deref() == Some("validate")).map(|(_, (_, o))| where_(o).1).collect();
        let dropped: Vec<usize> = (0..vitems.len()).filter(|&k| {
            let gid = vitems[k].val("id");
            emptied.contains(&gid) && !still.contains(&gid)
        }).collect();
        for &k in &dropped {
            eds[vfile].remove_item(vgroups.unwrap(), k);
            w(out, format!("dropped the empty Validate group: {}", opt(&vitems[k].val("id"))));
        }
        if dropped.len() == vitems.len() {
            w(out, "Validate is empty now: remove the category and its icon (docs/objects/README.md, \"Validate\").".into());
        }
    }

    // Build, parse and check every changed file before writing any.
    let mut done: Vec<(usize, String)> = Vec::new();
    for (fi, f) in files.iter().enumerate() {
        if eds[fi].edits.is_empty() {
            continue;
        }
        let text = eds[fi].new_text(&f.path).map_err(fail)?;
        let data: serde_json::Value = serde_json::from_str(&text).map_err(|e| fail(format!("{}: {e}", f.path.display())))?;
        for o in data.get("objects").and_then(|v| v.as_array()).map_or(&[][..], |v| v) {
            let Some(id) = o.get("id").and_then(|v| v.as_str()) else { continue };
            if let Some((cat, group)) = expect.get(id) {
                let got = (o.get("category").and_then(|v| v.as_str()), o.get("group").and_then(|v| v.as_str()));
                if got != (Some(cat.as_str()), group.as_deref()) {
                    return Err(fail(format!("{id}: {} / {}", got.0.unwrap_or("None"), got.1.unwrap_or("None"))));
                }
            }
        }
        done.push((fi, text));
    }
    for (fi, text) in done {
        let f = &files[fi];
        let (rel, header) = match f.path.strip_prefix(&root) {
            Ok(r) => (r.display().to_string(), format!("a/{}", r.display())),
            Err(_) if f.path.is_relative() => (f.path.display().to_string(), format!("a/{}", f.path.display())),
            Err(_) => (f.path.display().to_string(), f.path.display().to_string()),
        };
        if a.dry_run {
            let header_b = if header.starts_with("a/") { format!("b/{rel}") } else { header.clone() };
            let diff = similar::TextDiff::configure().algorithm(similar::Algorithm::Myers).diff_lines(f.text.as_str(), text.as_str());
            let _ = write!(out, "{}", diff.unified_diff().context_radius(3).missing_newline_hint(false).header(&header, &header_b));
        } else {
            std::fs::write(&f.path, &text).map_err(|e| fail(format!("{}: {e}", f.path.display())))?;
            w(out, format!("wrote {rel}"));
        }
    }
    Ok(())
}

/// The binary's entry: parses `argv` (without the program name), runs, and returns the exit code.
pub fn main_with(argv: &[String]) -> i32 {
    let mut stdout = std::io::stdout().lock();
    let res = parse_args(argv).and_then(|a| run(&a, &mut stdout));
    let _ = stdout.flush();
    match res {
        Ok(()) => 0,
        Err(e) if e.code == 0 => {
            print!("{HELP}");
            0
        }
        Err(e) => {
            eprintln!("{}", e.msg);
            e.code
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dumps_like_python() {
        let u = "\\u";
        assert_eq!(json_str("Caf\u{e9} \u{2615} \u{1f600} \"q\" \\ \n\x7f"), format!("\"Caf{u}00e9 {u}2615 {u}d83d{u}de00 \\\"q\\\" \\\\ \\n{u}007f\""));
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
        assert_eq!(ed.new_text(Path::new("x")).unwrap(), "{\"a\": 1, \"d\": 4, \"c\": 3}");
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
