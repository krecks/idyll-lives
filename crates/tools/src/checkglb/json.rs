//! Python's `json` reader (CPython's C scanner), so a file the gate can't read fails with Python's words:
//! `Expecting ',' delimiter: line 3 column 5 (char 41)`. Objects keep their keys in file order (a key given
//! twice keeps its first place and its last value), integers stay apart from floats, and NaN, Infinity and
//! -Infinity are numbers, as in Python.

use super::py::J;

/// `json.load(f)` of a file opened as text: UTF-8, newlines made `\n`, a byte-order mark refused.
pub fn loads_text(bytes: &[u8]) -> Result<J, String> {
    let text = utf8(bytes)?.replace("\r\n", "\n").replace('\r', "\n");
    let s: Vec<char> = text.chars().collect();
    if s.first() == Some(&'\u{feff}') {
        return Err(message(&s, "Unexpected UTF-8 BOM (decode using utf-8-sig)", 0));
    }
    decode(&s)
}

/// `json.loads(b)` of bytes (a .glb's JSON chunk): UTF-8, a byte-order mark skipped.
pub fn loads_bytes(bytes: &[u8]) -> Result<J, String> {
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    let s: Vec<char> = utf8(bytes)?.chars().collect();
    decode(&s)
}

/// UTF-8, or a UnicodeDecodeError's text.
fn utf8(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|e| {
        let at = e.valid_up_to();
        let (end, why) = match e.error_len() {
            None => (bytes.len() - 1, "unexpected end of data"),
            Some(n) => (at + n - 1, if matches!(bytes[at], 0x80..=0xC1 | 0xF5..=0xFF) { "invalid start byte" } else { "invalid continuation byte" }),
        };
        if end == at { format!("'utf-8' codec can't decode byte 0x{:02x} in position {at}: {why}", bytes[at]) } else { format!("'utf-8' codec can't decode bytes in position {at}-{end}: {why}") }
    })
}

/// A JSONDecodeError's text: `msg: line L column C (char N)`.
fn message(s: &[char], msg: &str, pos: usize) -> String {
    let before = &s[..pos.min(s.len())];
    let line = before.iter().filter(|c| **c == '\n').count() + 1;
    let column = match before.iter().rposition(|c| *c == '\n') {
        Some(nl) => pos - nl,
        None => pos + 1,
    };
    format!("{msg}: line {line} column {column} (char {pos})")
}

/// Why a scan stopped: no value where one was expected (StopIteration), or an error at a position.
enum Stop {
    NoValue(usize),
    Error(&'static str, usize),
}

fn decode(s: &[char]) -> Result<J, String> {
    let at = skip(s, 0);
    let mut scanner = Scanner { s, depth: 0 };
    match scanner.scan(at) {
        Ok((v, end)) => {
            let end = skip(s, end);
            if end != s.len() { Err(message(s, "Extra data", end)) } else { Ok(v) }
        }
        Err(Stop::NoValue(at)) => Err(message(s, "Expecting value", at)),
        Err(Stop::Error(msg, at)) => Err(message(s, msg, at)),
    }
}

fn space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

fn skip(s: &[char], mut i: usize) -> usize {
    while i < s.len() && space(s[i]) {
        i += 1;
    }
    i
}

struct Scanner<'a> {
    s: &'a [char],
    depth: usize,
}

impl Scanner<'_> {
    fn at(&self, i: usize, word: &str) -> bool {
        word.chars().enumerate().all(|(k, c)| self.s.get(i + k) == Some(&c))
    }

    /// One value at `idx`: it and where it ends.
    fn scan(&mut self, idx: usize) -> Result<(J, usize), Stop> {
        let s = self.s;
        let len = s.len();
        if idx >= len {
            return Err(Stop::NoValue(idx));
        }
        match s[idx] {
            '"' => return self.string(idx + 1).map(|(v, end)| (J::Str(v), end)),
            '{' | '[' => {
                if self.depth > 5000 {
                    return Err(Stop::Error("maximum recursion depth exceeded while decoding a JSON document", idx));
                }
                self.depth += 1;
                let r = if s[idx] == '{' { self.object(idx + 1) } else { self.array(idx + 1) };
                self.depth -= 1;
                return r;
            }
            'n' if idx + 3 < len && self.at(idx, "null") => return Ok((J::Null, idx + 4)),
            't' if idx + 3 < len && self.at(idx, "true") => return Ok((J::Bool(true), idx + 4)),
            'f' if idx + 4 < len && self.at(idx, "false") => return Ok((J::Bool(false), idx + 5)),
            'N' if idx + 2 < len && self.at(idx, "NaN") => return Ok((J::Float(f64::NAN), idx + 3)),
            'I' if idx + 7 < len && self.at(idx, "Infinity") => return Ok((J::Float(f64::INFINITY), idx + 8)),
            '-' if idx + 8 < len && self.at(idx, "-Infinity") => return Ok((J::Float(f64::NEG_INFINITY), idx + 9)),
            _ => {}
        }
        self.number(idx)
    }

    fn number(&self, start: usize) -> Result<(J, usize), Stop> {
        let s = self.s;
        let end_idx = s.len() - 1;
        let digit = |i: usize| s.get(i).is_some_and(|c| c.is_ascii_digit());
        let mut idx = start;
        if s[idx] == '-' {
            idx += 1;
            if idx > end_idx {
                return Err(Stop::NoValue(start));
            }
        }
        if ('1'..='9').contains(&s[idx]) {
            idx += 1;
            while idx <= end_idx && digit(idx) {
                idx += 1;
            }
        } else if s[idx] == '0' {
            idx += 1;
        } else {
            return Err(Stop::NoValue(start));
        }
        let mut float = false;
        if idx < end_idx && s[idx] == '.' && digit(idx + 1) {
            float = true;
            idx += 2;
            while idx <= end_idx && digit(idx) {
                idx += 1;
            }
        }
        if idx < end_idx && (s[idx] == 'e' || s[idx] == 'E') {
            let e_start = idx;
            idx += 1;
            if idx < end_idx && (s[idx] == '-' || s[idx] == '+') {
                idx += 1;
            }
            while idx <= end_idx && digit(idx) {
                idx += 1;
            }
            if digit(idx - 1) {
                float = true;
            } else {
                idx = e_start;
            }
        }
        let text: String = s[start..idx].iter().collect();
        let v = if float {
            J::Float(text.parse().unwrap_or(f64::NAN))
        } else {
            text.parse::<i64>().map(J::Int).unwrap_or_else(|_| J::Float(text.parse().unwrap_or(f64::NAN)))
        };
        Ok((v, idx))
    }

    /// A string whose opening quote is just before `end`: it and where it ends (after its closing quote).
    fn string(&self, mut end: usize) -> Result<(String, usize), Stop> {
        let s = self.s;
        let len = s.len();
        let begin = end - 1;
        let mut out = String::new();
        loop {
            let mut next = end;
            let mut c = None;
            while next < len {
                let d = s[next];
                if d == '"' || d == '\\' {
                    c = Some(d);
                    break;
                }
                if (d as u32) <= 0x1f {
                    return Err(Stop::Error("Invalid control character at", next));
                }
                next += 1;
            }
            let Some(c) = c else { return Err(Stop::Error("Unterminated string starting at", begin)) };
            out.extend(&s[end..next]);
            next += 1;
            if c == '"' {
                end = next;
                break;
            }
            if next == len {
                return Err(Stop::Error("Unterminated string starting at", begin));
            }
            let e = s[next];
            if e != 'u' {
                end = next + 1;
                out.push(match e {
                    '"' | '\\' | '/' => e,
                    'b' => '\u{8}',
                    'f' => '\u{c}',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => return Err(Stop::Error("Invalid \\escape", end - 2)),
                });
                continue;
            }
            next += 1;
            end = next + 4;
            if end >= len {
                return Err(Stop::Error("Invalid \\uXXXX escape", next - 1));
            }
            let hex = |from: usize, to: usize| -> Option<u32> { s[from..to].iter().try_fold(0u32, |n, d| d.to_digit(16).map(|v| n << 4 | v)) };
            let mut u = hex(next, end).ok_or(Stop::Error("Invalid \\uXXXX escape", end - 5))?;
            next = end;
            if (0xD800..0xDC00).contains(&u) && end + 6 < len && s[next] == '\\' && s[next + 1] == 'u' {
                next += 2;
                end += 6;
                let low = hex(next, end).ok_or(Stop::Error("Invalid \\uXXXX escape", end - 5))?;
                if (0xDC00..0xE000).contains(&low) {
                    u = 0x10000 + ((u - 0xD800) << 10) + (low - 0xDC00);
                } else {
                    end -= 6;
                }
            }
            // (A lone surrogate, which Python keeps, can't be in a Rust string.)
            out.push(char::from_u32(u).unwrap_or('\u{fffd}'));
        }
        Ok((out, end))
    }

    fn object(&mut self, mut idx: usize) -> Result<(J, usize), Stop> {
        let s = self.s;
        let end_idx = s.len() as isize - 1;
        let le = |i: usize| (i as isize) <= end_idx;
        let mut out: Vec<(String, J)> = Vec::new();
        idx = skip(s, idx);
        if !le(idx) || s[idx] != '}' {
            loop {
                if !le(idx) || s[idx] != '"' {
                    return Err(Stop::Error("Expecting property name enclosed in double quotes", idx));
                }
                let (key, next) = self.string(idx + 1)?;
                idx = skip(s, next);
                if !le(idx) || s[idx] != ':' {
                    return Err(Stop::Error("Expecting ':' delimiter", idx));
                }
                idx = skip(s, idx + 1);
                let (val, next) = self.scan(idx)?;
                match out.iter_mut().find(|(k, _)| *k == key) {
                    Some(slot) => slot.1 = val,
                    None => out.push((key, val)),
                }
                idx = skip(s, next);
                if le(idx) && s[idx] == '}' {
                    break;
                }
                if !le(idx) || s[idx] != ',' {
                    return Err(Stop::Error("Expecting ',' delimiter", idx));
                }
                let comma = idx;
                idx = skip(s, idx + 1);
                if le(idx) && s[idx] == '}' {
                    return Err(Stop::Error("Illegal trailing comma before end of object", comma));
                }
            }
        }
        Ok((J::Obj(out), idx + 1))
    }

    fn array(&mut self, mut idx: usize) -> Result<(J, usize), Stop> {
        let s = self.s;
        let end_idx = s.len() as isize - 1;
        let le = |i: usize| (i as isize) <= end_idx;
        let mut out = Vec::new();
        idx = skip(s, idx);
        if !le(idx) || s[idx] != ']' {
            loop {
                let (val, next) = self.scan(idx)?;
                out.push(val);
                idx = skip(s, next);
                if le(idx) && s[idx] == ']' {
                    break;
                }
                if !le(idx) || s[idx] != ',' {
                    return Err(Stop::Error("Expecting ',' delimiter", idx));
                }
                let comma = idx;
                idx = skip(s, idx + 1);
                if le(idx) && s[idx] == ']' {
                    return Err(Stop::Error("Illegal trailing comma before end of array", comma));
                }
            }
        }
        Ok((J::Arr(out), idx + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(text: &str) -> String {
        loads_text(text.as_bytes()).err().unwrap()
    }

    /// What Python 3.13's `json.loads` says of each.
    #[test]
    fn errors_as_python_says_them() {
        for (text, msg) in [
            ("", "Expecting value: line 1 column 1 (char 0)"),
            (" ", "Expecting value: line 1 column 2 (char 1)"),
            ("{", "Expecting property name enclosed in double quotes: line 1 column 2 (char 1)"),
            ("{\"a\": }", "Expecting value: line 1 column 7 (char 6)"),
            ("{\"a\" 1}", "Expecting ':' delimiter: line 1 column 6 (char 5)"),
            ("{\"a\": 1 \"b\": 2}", "Expecting ',' delimiter: line 1 column 9 (char 8)"),
            ("{\"a\": 1,}", "Illegal trailing comma before end of object: line 1 column 8 (char 7)"),
            ("[1,]", "Illegal trailing comma before end of array: line 1 column 3 (char 2)"),
            ("[1 2]", "Expecting ',' delimiter: line 1 column 4 (char 3)"),
            ("[", "Expecting value: line 1 column 2 (char 1)"),
            ("{\"a\": [1, 2", "Expecting ',' delimiter: line 1 column 12 (char 11)"),
            ("\"abc", "Unterminated string starting at: line 1 column 1 (char 0)"),
            ("\"a\\qb\"", "Invalid \\escape: line 1 column 3 (char 2)"),
            ("\"a\\u12\"", "Invalid \\uXXXX escape: line 1 column 4 (char 3)"),
            ("\"a\\u12zz\"", "Invalid \\uXXXX escape: line 1 column 4 (char 3)"),
            ("\"\u{1}\"", "Invalid control character at: line 1 column 2 (char 1)"),
            ("nul", "Expecting value: line 1 column 1 (char 0)"),
            ("True", "Expecting value: line 1 column 1 (char 0)"),
            ("-", "Expecting value: line 1 column 1 (char 0)"),
            ("01", "Extra data: line 1 column 2 (char 1)"),
            ("1.", "Extra data: line 1 column 2 (char 1)"),
            ("1e+", "Extra data: line 1 column 2 (char 1)"),
            ("{\"a\":1} \n x", "Extra data: line 2 column 2 (char 10)"),
            ("\n\n  {\"a\": [1, 2,]}", "Illegal trailing comma before end of array: line 3 column 14 (char 15)"),
            ("\u{feff}{}", "Unexpected UTF-8 BOM (decode using utf-8-sig): line 1 column 1 (char 0)"),
            ("[\n1,\n\n]", "Illegal trailing comma before end of array: line 2 column 2 (char 3)"),
            ("\"\\\"", "Unterminated string starting at: line 1 column 1 (char 0)"),
            ("\"\\u\"", "Invalid \\uXXXX escape: line 1 column 3 (char 2)"),
            ("{\"é\": 1,}", "Illegal trailing comma before end of object: line 1 column 8 (char 7)"),
            ("{\"a\": 1}\r\n,", "Extra data: line 2 column 1 (char 9)"),
        ] {
            assert_eq!(err(text), msg, "{text:?}");
        }
        assert_eq!(loads_text(b"[1, \xff]").err().unwrap(), "'utf-8' codec can't decode byte 0xff in position 4: invalid start byte");
        assert_eq!(loads_text(b"\"\xf0\x9f\x98(\"").err().unwrap(), "'utf-8' codec can't decode bytes in position 1-3: invalid continuation byte");
    }

    #[test]
    fn values_as_python_reads_them() {
        let v = loads_text(br#"[Infinity, NaN, -0, -0.0, 1E400, 12, 1.5e3, "\ud83d\ude00", "\u00e9\n", {"a": 1, "a": 2, "b": 3}]"#).unwrap();
        let J::Arr(v) = v else { panic!() };
        assert_eq!(v[0], J::Float(f64::INFINITY));
        assert!(matches!(v[1], J::Float(f) if f.is_nan()));
        assert_eq!(v[2], J::Int(0));
        assert!(matches!(v[3], J::Float(f) if f == 0.0 && f.is_sign_negative()));
        assert_eq!(v[4], J::Float(f64::INFINITY));
        assert_eq!(v[5], J::Int(12));
        assert_eq!(v[6], J::Float(1500.0));
        assert_eq!(v[7], J::Str("😀".into()));
        assert_eq!(v[8], J::Str("é\n".into()));
        assert_eq!(v[9], J::Obj(vec![("a".into(), J::Int(2)), ("b".into(), J::Int(3))]));
        assert_eq!(loads_bytes(b"\xef\xbb\xbf{} ").unwrap(), J::Obj(Vec::new()));
    }
}
