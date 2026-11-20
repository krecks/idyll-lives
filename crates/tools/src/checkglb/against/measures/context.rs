//! The context a comparison runs in.

use super::*;

/// What the rows are compared with: the two sides, and the working tree's content for the gate.
pub(crate) struct Context {
    pub(crate) old: Side,
    pub(crate) new: Side,
    pub(crate) facts: HashMap<String, Vec<Facts>>,
    pub(crate) furniture: Furniture,
    /// port-accepted.json: {row: {metric: why}}.
    pub(crate) accepted: BTreeMap<String, BTreeMap<String, String>>,
}

/// A digit run as `#`, so two findings that differ in their numbers alone are one finding.
pub(crate) fn masked(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut digits = false;
    for c in text.chars() {
        let digit = c.is_ascii_digit() || (digits && c == '.');
        if digit && !digits {
            out.push('#');
        } else if !digit {
            out.push(c);
        }
        digits = digit;
    }
    out
}

/// The texts of `new` that `old` has not, numbers aside (each of `old` answers for one of `new`).
pub(crate) fn newly<'a>(old: &[String], new: &'a [String]) -> Vec<&'a String> {
    let mut have: HashMap<String, usize> = HashMap::new();
    for text in old {
        *have.entry(masked(text)).or_default() += 1;
    }
    new.iter()
        .filter(|text| match have.get_mut(&masked(text)) {
            Some(n) if *n > 0 => {
                *n -= 1;
                false
            }
            _ => true,
        })
        .collect()
}

/// The draw calls in the gate's summary of a model (`check`'s "N draws").
pub(crate) fn draws(summary: &str) -> Option<usize> {
    let words: Vec<&str> = summary.split_whitespace().collect();
    let at = words.iter().position(|w| *w == "draws")?;
    words.get(at.checked_sub(1)?)?.parse().ok()
}
