//! A small JSON value for what a model hands the pack (`motions`, `perches`, a finish's fields): the keys stay in
//! the order they were written, as a Python dict's do.

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Sets `key`, in place if it is there, at the end if not.
    pub fn set(&mut self, key: &str, value: Json) {
        if let Json::Obj(fields) = self {
            match fields.iter_mut().find(|(k, _)| k == key) {
                Some((_, v)) => *v = value,
                None => fields.push((key.to_string(), value)),
            }
        }
    }

    pub fn str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    /// One line, as Python's `json.dumps` writes it.
    pub fn dumps(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    fn write(&self, out: &mut String) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Int(i) => out.push_str(&i.to_string()),
            Json::Num(x) => out.push_str(&float_repr(*x)),
            Json::Str(s) => write_str(s, out),
            Json::Arr(items) => {
                out.push('[');
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    v.write(out);
                }
                out.push(']');
            }
            Json::Obj(fields) => {
                out.push('{');
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    write_str(k, out);
                    out.push_str(": ");
                    v.write(out);
                }
                out.push('}');
            }
        }
    }
}

fn write_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// A float as Python's `repr` writes it: the shortest digits that read back the same, `1e-05` below a
/// ten-thousandth and from 1e16 up, and always a point or an exponent.
pub fn float_repr(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity" } else { "-Infinity" }.into();
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0.0" } else { "0.0" }.into();
    }
    let sci = format!("{x:e}");
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    if (-4..16).contains(&exp) {
        let plain = format!("{x}");
        if plain.contains('.') {
            plain
        } else {
            plain + ".0"
        }
    } else {
        format!(
            "{mantissa}e{}{:02}",
            if exp < 0 { '-' } else { '+' },
            exp.abs()
        )
    }
}

impl From<bool> for Json {
    fn from(b: bool) -> Json {
        Json::Bool(b)
    }
}
impl From<i64> for Json {
    fn from(i: i64) -> Json {
        Json::Int(i)
    }
}
impl From<i32> for Json {
    fn from(i: i32) -> Json {
        Json::Int(i as i64)
    }
}
impl From<f64> for Json {
    fn from(x: f64) -> Json {
        Json::Num(x)
    }
}
impl From<&str> for Json {
    fn from(s: &str) -> Json {
        Json::Str(s.to_string())
    }
}
impl From<String> for Json {
    fn from(s: String) -> Json {
        Json::Str(s)
    }
}
impl<T: Into<Json>> From<Vec<T>> for Json {
    fn from(v: Vec<T>) -> Json {
        Json::Arr(v.into_iter().map(Into::into).collect())
    }
}

/// Reads JSON text (a .glb's JSON chunk): a number without a point or an exponent is `Int`, any other `Num`; an
/// object's keys stay in their order.
pub fn loads(text: &str) -> Result<Json, String> {
    let mut p = Parser {
        s: text.as_bytes(),
        at: 0,
    };
    let value = p.value(0)?;
    p.space();
    if p.at != p.s.len() {
        return Err(p.error("more after the value"));
    }
    Ok(value)
}

/// How deep arrays and objects may nest (a .glb's go four deep): deeper is refused, not a stack overflow.
const DEEPEST: usize = 64;

struct Parser<'a> {
    s: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn error(&self, what: &str) -> String {
        format!("JSON: {what} at byte {}", self.at)
    }

    fn space(&mut self) {
        while matches!(self.s.get(self.at), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.at += 1;
        }
    }

    fn eat(&mut self, c: u8) -> Result<(), String> {
        self.space();
        if self.s.get(self.at) == Some(&c) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.error(&format!("expected '{}'", c as char)))
        }
    }

    fn word(&mut self, word: &str, value: Json) -> Result<Json, String> {
        if self.s[self.at..].starts_with(word.as_bytes()) {
            self.at += word.len();
            Ok(value)
        } else {
            Err(self.error("not a value"))
        }
    }

    fn value(&mut self, depth: usize) -> Result<Json, String> {
        if depth > DEEPEST {
            return Err(self.error("nested too deep"));
        }
        self.space();
        match self.s.get(self.at) {
            Some(b'{') => {
                self.at += 1;
                let mut fields = Vec::new();
                self.space();
                if self.s.get(self.at) == Some(&b'}') {
                    self.at += 1;
                    return Ok(Json::Obj(fields));
                }
                loop {
                    self.space();
                    let key = self.string()?;
                    self.eat(b':')?;
                    fields.push((key, self.value(depth + 1)?));
                    self.space();
                    match self.s.get(self.at) {
                        Some(b',') => self.at += 1,
                        Some(b'}') => {
                            self.at += 1;
                            return Ok(Json::Obj(fields));
                        }
                        _ => return Err(self.error("expected ',' or '}'")),
                    }
                }
            }
            Some(b'[') => {
                self.at += 1;
                let mut items = Vec::new();
                self.space();
                if self.s.get(self.at) == Some(&b']') {
                    self.at += 1;
                    return Ok(Json::Arr(items));
                }
                loop {
                    items.push(self.value(depth + 1)?);
                    self.space();
                    match self.s.get(self.at) {
                        Some(b',') => self.at += 1,
                        Some(b']') => {
                            self.at += 1;
                            return Ok(Json::Arr(items));
                        }
                        _ => return Err(self.error("expected ',' or ']'")),
                    }
                }
            }
            Some(b'"') => self.string().map(Json::Str),
            Some(b't') => self.word("true", Json::Bool(true)),
            Some(b'f') => self.word("false", Json::Bool(false)),
            Some(b'n') => self.word("null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.error("not a value")),
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.at;
        while matches!(
            self.s.get(self.at),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.at += 1;
        }
        let text =
            std::str::from_utf8(&self.s[start..self.at]).map_err(|_| self.error("a number"))?;
        let whole = !text.contains(['.', 'e', 'E']);
        if whole && let Ok(i) = text.parse::<i64>() {
            return Ok(Json::Int(i));
        }
        text.parse::<f64>()
            .map(Json::Num)
            .map_err(|_| self.error(&format!("{text:?} is not a number")))
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let digits = self
            .s
            .get(self.at..self.at + 4)
            .and_then(|d| std::str::from_utf8(d).ok())
            .and_then(|d| u32::from_str_radix(d, 16).ok())
            .ok_or_else(|| self.error("a \\u escape"))?;
        self.at += 4;
        Ok(digits)
    }

    fn string(&mut self) -> Result<String, String> {
        if self.s.get(self.at) != Some(&b'"') {
            return Err(self.error("expected a string"));
        }
        self.at += 1;
        let mut out = Vec::new();
        loop {
            let Some(&c) = self.s.get(self.at) else {
                return Err(self.error("a string that doesn't end"));
            };
            self.at += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let Some(&e) = self.s.get(self.at) else {
                        return Err(self.error("a string that doesn't end"));
                    };
                    self.at += 1;
                    let ch = match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let mut unit = self.hex4()?;
                            if (0xD800..0xDC00).contains(&unit)
                                && self.s.get(self.at..self.at + 2) == Some(b"\\u")
                            {
                                self.at += 2;
                                let low = self.hex4()?;
                                unit = 0x10000
                                    + ((unit - 0xD800) << 10)
                                    + (low.wrapping_sub(0xDC00) & 0x3FF);
                            }
                            char::from_u32(unit).ok_or_else(|| self.error("a \\u escape"))?
                        }
                        _ => return Err(self.error("an escape")),
                    };
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                }
                c => out.push(c),
            }
        }
        String::from_utf8(out).map_err(|_| self.error("a string that isn't UTF-8"))
    }
}

/// A JSON value written as JSON: `json!({"uses": {"pick": {"clip": "pick", "delay": 0.4}}, "growth": "ripen"})`.
/// A value that isn't a literal goes in brackets: `json!({"hold": (seconds)})`.
#[macro_export]
macro_rules! json {
    (null) => { $crate::json::Json::Null };
    ([ $($item:tt),* $(,)? ]) => { $crate::json::Json::Arr(vec![$($crate::json!($item)),*]) };
    ({ $($key:literal : $value:tt),* $(,)? }) => {
        $crate::json::Json::Obj(vec![$(($key.to_string(), $crate::json!($value))),*])
    };
    ($other:expr) => { $crate::json::Json::from($other) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_read_as_pythons_repr() {
        assert_eq!(float_repr(0.1), "0.1");
        assert_eq!(float_repr(1.0), "1.0");
        assert_eq!(float_repr(-0.011836886405944824), "-0.011836886405944824");
        assert_eq!(float_repr(0.0001), "0.0001");
        assert_eq!(float_repr(0.00001), "1e-05");
        assert_eq!(float_repr(1.5e-7), "1.5e-07");
        assert_eq!(float_repr(1e16), "1e+16");
        assert_eq!(float_repr(123456789012345.0), "123456789012345.0");
        assert_eq!(float_repr(-0.0), "-0.0");
    }

    #[test]
    fn a_value_is_written_as_json_dumps_writes_it() {
        let hold = 2.5;
        let j = json!({"uses": {"pick": {"clip": "pick", "delay": 0.4}}, "hold": (hold), "open": [1, true, null], "name": "caf\u{e9}"});
        assert_eq!(
            j.dumps(),
            r#"{"uses": {"pick": {"clip": "pick", "delay": 0.4}}, "hold": 2.5, "open": [1, true, null], "name": "caf\u00e9"}"#
        );
        assert_eq!(j.get("hold"), Some(&Json::Num(2.5)));
    }

    #[test]
    fn what_dumps_writes_loads_reads_back() {
        let name = "caf\u{e9} \"q\"\n\u{1f333}";
        let j = json!({"uses": {"pick": {"delay": 0.4}}, "n": 3, "big": 1e300, "open": [1, true, null], "name": (name)});
        assert_eq!(loads(&j.dumps()), Ok(j));
        assert_eq!(
            loads(r#"{"a":[-1,2.0,1e-05]}"#).unwrap().get("a"),
            Some(&Json::Arr(vec![
                Json::Int(-1),
                Json::Num(2.0),
                Json::Num(1e-5)
            ]))
        );
        assert_eq!(
            loads(r#""\ud83c\udf33/""#),
            Ok(Json::Str("\u{1f333}/".into()))
        );
        for bad in [
            "",
            "{",
            "[1,]",
            "{\"a\" 1}",
            "tru",
            "\"open",
            "1 2",
            "[".repeat(100).as_str(),
        ] {
            assert!(loads(bad).is_err(), "{bad:?}");
        }
    }
}
