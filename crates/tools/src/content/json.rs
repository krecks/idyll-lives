use super::{py_space, py_strip};

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
    pub(super) fn py(&self) -> Option<String> {
        match self {
            Val::Str(s) | Val::Num(s) => Some(s.clone()),
            Val::Bool(b) => Some(if *b { "True" } else { "False" }.into()),
            Val::Null | Val::Object | Val::Array => None,
        }
    }

    pub(super) fn truthy(&self) -> bool {
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
        Node {
            start,
            end,
            value,
            members: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Node> {
        self.members.iter().find(|m| m.0 == key).map(|m| &m.2)
    }

    /// `(index, key start, value)` of member `key`.
    pub fn member(&self, key: &str) -> Option<(usize, usize, &Node)> {
        self.members
            .iter()
            .enumerate()
            .find(|(_, m)| m.0 == key)
            .map(|(i, m)| (i, m.1, &m.2))
    }

    /// The value of member `key` as Python's `node.get(key).value` printed (`None` when missing or null).
    pub(super) fn val(&self, key: &str) -> Option<String> {
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
        self.t
            .as_bytes()
            .get(i)
            .copied()
            .ok_or_else(|| "string index out of range".to_string())
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
        serde_json::from_str::<String>(&self.t[a..b])
            .map_err(|e| format!("bad string at {}: {e}", self.cp(a)))
    }

    fn value(&self, i: usize) -> Result<Node, String> {
        let i = self.ws(i);
        let c = self.at(i)?;
        if c == b'{' {
            let mut members = Vec::new();
            let mut j = self.ws(i + 1);
            if self.at(j)? == b'}' {
                return Ok(Node {
                    start: i,
                    end: j + 1,
                    value: Val::Object,
                    members,
                    items: Vec::new(),
                });
            }
            loop {
                j = self.ws(j);
                let m = self
                    .string_end(j)
                    .ok_or_else(|| format!("a key expected at {}", self.cp(j)))?;
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
                return Ok(Node {
                    start: i,
                    end: j + 1,
                    value: Val::Object,
                    members,
                    items: Vec::new(),
                });
            }
        }
        if c == b'[' {
            let mut items = Vec::new();
            let mut j = self.ws(i + 1);
            if self.at(j)? == b']' {
                return Ok(Node {
                    start: i,
                    end: j + 1,
                    value: Val::Array,
                    members: Vec::new(),
                    items,
                });
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
                return Ok(Node {
                    start: i,
                    end: j + 1,
                    value: Val::Array,
                    members: Vec::new(),
                    items,
                });
            }
        }
        if c == b'"' {
            let m = self
                .string_end(i)
                .ok_or_else(|| format!("a string expected at {}", self.cp(i)))?;
            return Ok(Node::leaf(i, m, Val::Str(self.decode(i, m)?)));
        }
        for (word, v) in [
            ("true", Val::Bool(true)),
            ("false", Val::Bool(false)),
            ("null", Val::Null),
        ] {
            if self.t[i..].starts_with(word) {
                return Ok(Node::leaf(i, i + word.len(), v));
            }
        }
        let end = self
            .number_end(i)
            .ok_or_else(|| format!("a value expected at {}", self.cp(i)))?;
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
pub(super) fn indent_at(text: &str, pos: usize) -> &str {
    let line = text[..pos].rfind('\n').map_or(0, |i| i + 1);
    let s = &text[line..pos];
    if py_strip(s).is_empty() { s } else { "" }
}
