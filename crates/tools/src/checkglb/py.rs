//! Python's ways, where the gate prints what check_glb.py printed: JSON values as Python's `json` reads
//! them, objects in file order and integers apart from floats (`J`; `json.rs` reads them), Python's numbers
//! (`Num`: an int stays an int, so a zero negated stays 0, not -0.0), `repr`, `format` and
//! `math.hypot`/`math.dist`, and `os.path`'s `join`/`normpath`/`dirname`.

use std::ops::{Add, Div, Mul, Neg, Sub};

/// A JSON value as Python's `json` reads it: objects keep their keys in file order (a key given twice
/// keeps its first place and its last value), and integers stay apart from floats.
#[derive(Clone, Debug, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    /// `d.get(key)` (None for a missing key, or when this is no object).
    pub fn get(&self, key: &str) -> Option<&J> {
        match self {
            J::Obj(o) => o.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// `d.get(key)`, a missing key and `null` alike (Python's `None`).
    pub fn opt(&self, key: &str) -> Option<&J> {
        self.get(key).filter(|v| **v != J::Null)
    }

    /// `key in d`.
    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Python's truth: None, False, 0, 0.0, "", [] and {} are false.
    pub fn truthy(&self) -> bool {
        match self {
            J::Null => false,
            J::Bool(b) => *b,
            J::Int(i) => *i != 0,
            J::Float(f) => *f != 0.0,
            J::Str(s) => !s.is_empty(),
            J::Arr(a) => !a.is_empty(),
            J::Obj(o) => !o.is_empty(),
        }
    }

    pub fn str(&self) -> Option<&str> {
        match self {
            J::Str(s) => Some(s),
            _ => None,
        }
    }

    /// A number (`isinstance(v, (int, float))`: a bool is an int too).
    pub fn num(&self) -> Option<Num> {
        match self {
            J::Bool(b) => Some(Num::I(*b as i64)),
            J::Int(i) => Some(Num::I(*i)),
            J::Float(f) => Some(Num::F(*f)),
            _ => None,
        }
    }

    pub fn list(&self) -> Option<&[J]> {
        match self {
            J::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn entries(&self) -> Option<&[(String, J)]> {
        match self {
            J::Obj(o) => Some(o),
            _ => None,
        }
    }

    /// `l[i]` for a list (a negative index counts from the end, as in Python).
    pub fn at(&self, i: i64) -> Option<&J> {
        let a = self.list()?;
        let i = if i < 0 { i + a.len() as i64 } else { i };
        if i < 0 { None } else { a.get(i as usize) }
    }

    /// As a `serde_json::Value` (a NaN or an infinity as `null`: JSON has none).
    pub fn to_value(&self) -> serde_json::Value {
        use serde_json::Value;
        match self {
            J::Null => Value::Null,
            J::Bool(b) => Value::Bool(*b),
            J::Int(i) => Value::from(*i),
            J::Float(f) => serde_json::Number::from_f64(*f).map_or(Value::Null, Value::Number),
            J::Str(s) => Value::String(s.clone()),
            J::Arr(a) => Value::Array(a.iter().map(J::to_value).collect()),
            J::Obj(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), v.to_value())).collect()),
        }
    }

    /// A key in a dict: `{key: …}` where the key is this value (a string is itself; None and numbers as
    /// Python prints them).
    pub fn key(&self) -> String {
        match self {
            J::Str(s) => s.clone(),
            other => repr(other),
        }
    }
}

/// A Python number: arithmetic between ints stays exact and int (so `-0` is 0, where `-0.0` is -0.0);
/// with a float it is a float, as in Python.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Num {
    I(i64),
    F(f64),
}

impl Num {
    pub fn f(self) -> f64 {
        match self {
            Num::I(i) => i as f64,
            Num::F(f) => f,
        }
    }
}

macro_rules! num_op {
    ($tr:ident, $m:ident, $op:tt, $checked:ident) => {
        impl $tr for Num {
            type Output = Num;
            fn $m(self, o: Num) -> Num {
                match (self, o) {
                    (Num::I(a), Num::I(b)) => a.$checked(b).map(Num::I).unwrap_or(Num::F(a as f64 $op b as f64)),
                    (a, b) => Num::F(a.f() $op b.f()),
                }
            }
        }
    };
}
num_op!(Add, add, +, checked_add);
num_op!(Sub, sub, -, checked_sub);
num_op!(Mul, mul, *, checked_mul);

impl Div for Num {
    type Output = Num;
    fn div(self, o: Num) -> Num {
        Num::F(self.f() / o.f())
    }
}

impl Neg for Num {
    type Output = Num;
    fn neg(self) -> Num {
        match self {
            Num::I(i) => Num::I(-i),
            Num::F(f) => Num::F(-f),
        }
    }
}

/// Python's `min(a, b)`: `b` only if it is smaller.
pub fn min(a: Num, b: Num) -> Num {
    if b.f() < a.f() { b } else { a }
}

/// Python's `max(a, b)`: `b` only if it is larger.
pub fn max(a: Num, b: Num) -> Num {
    if b.f() > a.f() { b } else { a }
}

/// `f'{v:.{prec}f}'`.
pub fn fixed(v: f64, prec: usize) -> String {
    if v.is_nan() {
        return "nan".into();
    }
    format!("{v:.prec$}")
}

/// `f'{v:+.{prec}f}'`.
pub fn signed(v: f64, prec: usize) -> String {
    if v.is_nan() {
        return "+nan".into();
    }
    format!("{v:+.prec$}")
}

/// `f'{v:.0%}'`.
pub fn percent(v: f64) -> String {
    fixed(v * 100.0, 0) + "%"
}

/// `repr(v)` of a float: the shortest digits that read back, fixed from 1e-4 up to 1e16, `.0` on a whole
/// number, else `1e-05`, `1e+16`.
pub fn float_repr(v: f64) -> String {
    if v.is_nan() {
        return "nan".into();
    }
    if v.is_infinite() {
        return if v > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if v == 0.0 {
        return if v.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    // Rust's `{:e}` gives the shortest round-trip digits too: "-1.2345e-5".
    let e = format!("{v:e}");
    let (mantissa, exp) = e.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let (sign, mantissa) = mantissa.strip_prefix('-').map(|m| ("-", m)).unwrap_or(("", mantissa));
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let point = exp + 1; // digits are 0.d1d2… x 10^point
    if !(-4 < point && point <= 16) {
        let rest = if digits.len() > 1 { format!(".{}", &digits[1..]) } else { String::new() };
        return format!("{sign}{}{rest}e{}{:02}", &digits[..1], if exp < 0 { '-' } else { '+' }, exp.abs());
    }
    let n = digits.len() as i32;
    if point <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-point) as usize))
    } else if point >= n {
        format!("{sign}{digits}{}.0", "0".repeat((point - n) as usize))
    } else {
        format!("{sign}{}.{}", &digits[..point as usize], &digits[point as usize..])
    }
}

/// `repr(s)` of a string.
pub fn repr_str(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c)
            }
            c if (c as u32) < 0x20 || (0x7f..0xa0).contains(&(c as u32)) => out.push_str(&format!("\\x{:02x}", c as u32)),
            c if !printable(c) => {
                let n = c as u32;
                if n <= 0xff {
                    out.push_str(&format!("\\x{n:02x}"))
                } else if n <= 0xffff {
                    out.push_str(&format!("\\u{n:04x}"))
                } else {
                    out.push_str(&format!("\\U{n:08x}"))
                }
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// Python's `str.isprintable` for one character, near enough: not a control, format, separator (but the
/// space) or unassigned-looking code point.
fn printable(c: char) -> bool {
    if c == ' ' {
        return true;
    }
    !(c.is_control() || c.is_whitespace() || matches!(c as u32, 0xad | 0x200b..=0x200f | 0x2028..=0x202e | 0x2060..=0x206f | 0xfeff | 0xfff9..=0xfffb | 0xe000..=0xf8ff))
}

/// `repr(v)`.
pub fn repr(v: &J) -> String {
    match v {
        J::Null => "None".into(),
        J::Bool(true) => "True".into(),
        J::Bool(false) => "False".into(),
        J::Int(i) => i.to_string(),
        J::Float(f) => float_repr(*f),
        J::Str(s) => repr_str(s),
        J::Arr(a) => format!("[{}]", a.iter().map(repr).collect::<Vec<_>>().join(", ")),
        J::Obj(o) => format!("{{{}}}", o.iter().map(|(k, v)| format!("{}: {}", repr_str(k), repr(v))).collect::<Vec<_>>().join(", ")),
    }
}

/// `str(v)`: a string as it is, anything else as `repr`.
pub fn str_of(v: &J) -> String {
    match v {
        J::Str(s) => s.clone(),
        other => repr(other),
    }
}

/// `str(n)` of a number.
pub fn num_str(n: Num) -> String {
    match n {
        Num::I(i) => i.to_string(),
        Num::F(f) => float_repr(f),
    }
}

/// `json.dumps(s)` of a string (ASCII only, as Python writes it).
pub fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{u:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `math.hypot(*v)`: CPython's `vector_norm`, which rounds better than a plain square root of the sum
/// of squares (and so can differ from it in the last bit).
pub fn hypot(v: &[f64]) -> f64 {
    let mut abs: Vec<f64> = v.iter().map(|x| x.abs()).collect();
    let max = abs.iter().cloned().fold(0.0, |m: f64, x| if x > m { x } else { m });
    let nan = abs.iter().any(|x| x.is_nan());
    vector_norm(&mut abs, max, nan)
}

/// `math.dist(p, q)`.
pub fn dist(p: &[f64], q: &[f64]) -> f64 {
    let mut d: Vec<f64> = p.iter().zip(q).map(|(a, b)| (a - b).abs()).collect();
    let max = d.iter().cloned().fold(0.0, |m: f64, x| if x > m { x } else { m });
    let nan = d.iter().any(|x| x.is_nan());
    vector_norm(&mut d, max, nan)
}

fn vector_norm(vec: &mut [f64], max: f64, nan: bool) -> f64 {
    if max.is_infinite() {
        return max;
    }
    if nan {
        return f64::NAN;
    }
    if max == 0.0 || vec.len() <= 1 {
        return max;
    }
    let max_e = frexp_exp(max);
    if max_e < -1023 {
        for x in vec.iter_mut() {
            *x /= f64::MIN_POSITIVE;
        }
        return f64::MIN_POSITIVE * vector_norm(vec, max / f64::MIN_POSITIVE, nan);
    }
    let scale = ldexp1(-max_e);
    let (mut csum, mut frac1, mut frac2) = (1.0f64, 0.0f64, 0.0f64);
    for &x in vec.iter() {
        let x = x * scale;
        let (hi, lo) = dl_mul(x, x);
        let (shi, slo) = dl_fast_sum(csum, hi);
        csum = shi;
        frac1 += lo;
        frac2 += slo;
    }
    let mut h = (csum - 1.0 + (frac1 + frac2)).sqrt();
    let (hi, lo) = dl_mul(-h, h);
    let (shi, slo) = dl_fast_sum(csum, hi);
    csum = shi;
    frac1 += lo;
    frac2 += slo;
    let x = csum - 1.0 + (frac1 + frac2);
    h += x / (2.0 * h);
    h / scale
}

fn dl_mul(x: f64, y: f64) -> (f64, f64) {
    let z = x * y;
    (z, x.mul_add(y, -z))
}

fn dl_fast_sum(a: f64, b: f64) -> (f64, f64) {
    let x = a + b;
    (x, (a - x) + b)
}

/// The exponent `frexp` gives: `x = m * 2^e`, 0.5 <= m < 1.
fn frexp_exp(x: f64) -> i32 {
    let bits = x.to_bits();
    let e = ((bits >> 52) & 0x7ff) as i32;
    if e == 0 {
        // subnormal: normalise
        let lz = (bits << 12).leading_zeros() as i32;
        -1022 - lz
    } else {
        e - 1022
    }
}

/// 2^e, for -1074 <= e <= 1023.
fn ldexp1(e: i32) -> f64 {
    if e >= -1022 { f64::from_bits(((e + 1023) as u64) << 52) } else { f64::from_bits(1u64 << (e + 1074)) }
}

/// Python's `round(x)` to an int: halves to even.
pub fn round_even(x: f64) -> f64 {
    x.round_ties_even()
}

/// `os.path.normpath` (POSIX).
pub fn normpath(path: &str) -> String {
    if path.is_empty() {
        return ".".into();
    }
    let initial = if path.starts_with("//") && !path.starts_with("///") {
        2
    } else if path.starts_with('/') {
        1
    } else {
        0
    };
    let mut parts: Vec<&str> = Vec::new();
    for comp in path.split('/') {
        if comp.is_empty() || comp == "." {
            continue;
        }
        if comp != ".." || (initial == 0 && parts.is_empty()) || parts.last() == Some(&"..") {
            parts.push(comp);
        } else if !parts.is_empty() {
            parts.pop();
        }
    }
    let joined = parts.join("/");
    let out = format!("{}{joined}", "/".repeat(initial));
    if out.is_empty() { ".".into() } else { out }
}

/// `os.path.join(a, b)` (POSIX).
pub fn join(a: &str, b: &str) -> String {
    if b.starts_with('/') {
        b.into()
    } else if a.is_empty() || a.ends_with('/') {
        format!("{a}{b}")
    } else {
        format!("{a}/{b}")
    }
}

/// `os.path.dirname` (POSIX).
pub fn dirname(p: &str) -> String {
    let i = p.rfind('/').map(|i| i + 1).unwrap_or(0);
    let head = &p[..i];
    if !head.is_empty() && head != "/".repeat(head.len()) { head.trim_end_matches('/').into() } else { head.into() }
}

/// `os.path.relpath(path, start)` for two absolute, normalised paths.
pub fn relpath(path: &str, start: &str) -> String {
    let p: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let s: Vec<&str> = start.split('/').filter(|s| !s.is_empty()).collect();
    let common = p.iter().zip(&s).take_while(|(a, b)| a == b).count();
    let mut out: Vec<&str> = vec![".."; s.len() - common];
    out.extend(&p[common..]);
    if out.is_empty() { ".".into() } else { out.join("/") }
}

/// An OSError's text: `[Errno 2] No such file or directory: '/path'`.
pub fn os_error(err: &std::io::Error, path: &str) -> String {
    let text = err.to_string();
    let text = text.split(" (os error").next().unwrap_or(&text).to_string();
    match err.raw_os_error() {
        Some(n) => format!("[Errno {n}] {text}: {}", repr_str(path)),
        None => format!("{text}: {}", repr_str(path)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_repr_is_pythons() {
        for (v, s) in [(0.0, "0.0"), (-0.0, "-0.0"), (1.0, "1.0"), (-0.43, "-0.43"), (0.1, "0.1"), (1e-5, "1e-05"), (0.0001, "0.0001"),
            (1e16, "1e+16"), (1e15, "1000000000000000.0"), (123.456, "123.456"), (1.5e-7, "1.5e-07"), (2.5e20, "2.5e+20")] {
            assert_eq!(float_repr(v), s, "{v}");
        }
    }

    #[test]
    fn repr_is_pythons() {
        let v: J = super::super::json::loads_text(br#"{"a": [1, 2.0, null, true, "it's"], "b": {"c": "x\ny"}}"#).unwrap();
        assert_eq!(repr(&v), r#"{'a': [1, 2.0, None, True, "it's"], 'b': {'c': 'x\ny'}}"#);
    }

    #[test]
    fn objects_keep_their_order() {
        let v = super::super::json::loads_text(br#"{"z": 1, "a": 2, "z": 3}"#).unwrap();
        assert_eq!(v, J::Obj(vec![("z".into(), J::Int(3)), ("a".into(), J::Int(2))]));
    }

    #[test]
    fn ints_stay_ints() {
        assert_eq!(-Num::I(0), Num::I(0));
        assert!((-Num::F(0.0)).f().is_sign_negative());
        assert_eq!(Num::I(2) * Num::F(0.5), Num::F(1.0));
        assert_eq!(Num::I(3) / Num::I(2), Num::F(1.5));
    }

    #[test]
    fn hypot_is_exact_where_it_can_be() {
        assert_eq!(hypot(&[3.0, 4.0]), 5.0);
        assert_eq!(hypot(&[1.0, 2.0, 2.0]), 3.0);
        assert_eq!(dist(&[1.0, 1.0], &[4.0, 5.0]), 5.0);
        assert_eq!(hypot(&[-7.0]), 7.0);
    }

    #[test]
    fn paths_as_os_path() {
        assert_eq!(normpath("/a/b/../c/./d.glb"), "/a/c/d.glb");
        assert_eq!(normpath("/a/packs/x/../../models/f/m.glb"), "/a/models/f/m.glb");
        assert_eq!(dirname("/a/b/manifest.json"), "/a/b");
        assert_eq!(join("/a/b", "../c"), "/a/b/../c");
        assert_eq!(relpath("/r/web/public/assets/packs/x/manifest.json", "/r/web/public"), "assets/packs/x/manifest.json");
    }
}
