//! JSON as Python's `json` wrote and read it: maps keep their insertion order (`OMap` when
//! writing, `JValue` when reading; serde_json's own `Map` sorts its keys) and floats print as
//! Python's `repr` (`PyFormatter`).

use std::fmt;
use std::io;

use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};

use super::np::py_repr;

/// A map written in insertion order.
#[derive(Clone, Debug, Default)]
pub struct OMap<T>(pub Vec<(String, T)>);

impl<T> OMap<T> {
    pub fn new() -> Self {
        OMap(Vec::new())
    }

    pub fn insert(&mut self, k: impl Into<String>, v: T) {
        let k = k.into();
        if let Some(e) = self.0.iter_mut().find(|e| e.0 == k) {
            e.1 = v;
        } else {
            self.0.push((k, v));
        }
    }

    pub fn get(&self, k: &str) -> Option<&T> {
        self.0.iter().find(|e| e.0 == k).map(|e| &e.1)
    }

    pub fn get_mut(&mut self, k: &str) -> Option<&mut T> {
        self.0.iter_mut().find(|e| e.0 == k).map(|e| &mut e.1)
    }

    pub fn contains(&self, k: &str) -> bool {
        self.get(k).is_some()
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|e| e.0.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.0.iter().map(|e| (e.0.as_str(), &e.1))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Serialize> Serialize for OMap<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            m.serialize_entry(k, v)?;
        }
        m.end()
    }
}

/// serde_json's compact output with Python's float repr (`1e-05`, `4000.0`).
pub struct PyFormatter;

impl serde_json::ser::Formatter for PyFormatter {
    fn write_f64<W: ?Sized + io::Write>(&mut self, w: &mut W, v: f64) -> io::Result<()> {
        w.write_all(py_repr(v).as_bytes())
    }

    fn write_f32<W: ?Sized + io::Write>(&mut self, w: &mut W, v: f32) -> io::Result<()> {
        w.write_all(py_repr(v as f64).as_bytes())
    }
}

/// `json.dump(v, separators=(',', ':'))`.
pub fn to_string_py<T: Serialize>(v: &T) -> String {
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, PyFormatter);
    v.serialize(&mut ser).expect("json");
    String::from_utf8(buf).expect("utf-8")
}

/// A parsed JSON value that keeps its objects' key order and tells ints from floats.
#[derive(Clone, Debug, PartialEq)]
pub enum JValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Arr(Vec<JValue>),
    Obj(Vec<(String, JValue)>),
}

impl JValue {
    pub fn parse(s: &str) -> Result<JValue, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn get(&self, k: &str) -> Option<&JValue> {
        match self {
            JValue::Obj(m) => m.iter().find(|e| e.0 == k).map(|e| &e.1),
            _ => None,
        }
    }

    /// The value under `k` (panics if missing: the set is malformed).
    pub fn at(&self, k: &str) -> &JValue {
        self.get(k).unwrap_or_else(|| panic!("rig.json: missing key {k}"))
    }

    pub fn f64(&self) -> f64 {
        match *self {
            JValue::Int(v) => v as f64,
            JValue::Float(v) => v,
            _ => panic!("rig.json: not a number: {self:?}"),
        }
    }

    pub fn usize(&self) -> usize {
        match *self {
            JValue::Int(v) => v as usize,
            JValue::Float(v) => v as usize,
            _ => panic!("rig.json: not a number: {self:?}"),
        }
    }

    pub fn str(&self) -> &str {
        match self {
            JValue::Str(s) => s,
            _ => panic!("rig.json: not a string: {self:?}"),
        }
    }

    pub fn arr(&self) -> &[JValue] {
        match self {
            JValue::Arr(a) => a,
            _ => panic!("rig.json: not an array: {self:?}"),
        }
    }

    pub fn obj(&self) -> &[(String, JValue)] {
        match self {
            JValue::Obj(m) => m,
            _ => panic!("rig.json: not an object: {self:?}"),
        }
    }

    pub fn keys(&self) -> Vec<&str> {
        self.obj().iter().map(|e| e.0.as_str()).collect()
    }

    pub fn strs(&self) -> Vec<String> {
        self.arr().iter().map(|v| v.str().to_string()).collect()
    }

    pub fn f64s(&self) -> Vec<f64> {
        self.arr().iter().map(JValue::f64).collect()
    }
}

impl<'de> Deserialize<'de> for JValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<JValue, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = JValue;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON")
            }
            fn visit_unit<E>(self) -> Result<JValue, E> {
                Ok(JValue::Null)
            }
            fn visit_bool<E>(self, v: bool) -> Result<JValue, E> {
                Ok(JValue::Bool(v))
            }
            fn visit_i64<E>(self, v: i64) -> Result<JValue, E> {
                Ok(JValue::Int(v))
            }
            fn visit_u64<E>(self, v: u64) -> Result<JValue, E> {
                Ok(JValue::Int(v as i64))
            }
            fn visit_f64<E>(self, v: f64) -> Result<JValue, E> {
                Ok(JValue::Float(v))
            }
            fn visit_str<E>(self, v: &str) -> Result<JValue, E> {
                Ok(JValue::Str(v.to_string()))
            }
            fn visit_string<E>(self, v: String) -> Result<JValue, E> {
                Ok(JValue::Str(v))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<JValue, A::Error> {
                let mut out = Vec::new();
                while let Some(v) = a.next_element()? {
                    out.push(v);
                }
                Ok(JValue::Arr(out))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<JValue, A::Error> {
                let mut out = Vec::new();
                while let Some((k, v)) = a.next_entry::<String, JValue>()? {
                    out.push((k, v));
                }
                Ok(JValue::Obj(out))
            }
        }
        d.deserialize_any(V)
    }
}
