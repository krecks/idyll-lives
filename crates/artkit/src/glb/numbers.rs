//! Numbers as glTF writes them.

use super::*;

pub const FLOAT: u32 = 5126;
pub const USHORT: u32 = 5123;
pub const UINT: u32 = 5125;
pub const ARRAY_BUFFER: u32 = 34962;
pub const ELEMENT_ARRAY_BUFFER: u32 = 34963;

/// A number as the files have them: whole ones without a point, the rest as Python writes a float.
pub(crate) fn num(x: f64) -> String {
    if x == x.trunc() && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        float_repr(x)
    }
}

pub(crate) fn nums(xs: &[f32]) -> String {
    let parts: Vec<String> = xs.iter().map(|&x| num(x as f64)).collect();
    format!("[{}]", parts.join(","))
}

#[derive(Default)]
pub(crate) struct File {
    pub(crate) bin: Vec<u8>,
    pub(crate) accessors: Vec<String>,
    pub(crate) views: Vec<String>,
    /// The clips' accessors by their bytes: channels keyed alike share one.
    pub(crate) shared: BTreeMap<(Vec<u8>, &'static str), usize>,
}
