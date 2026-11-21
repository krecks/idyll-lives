//! Small parsing helpers shared by the content loaders.

use super::*;

/// A time content gives (minutes or hours): a number (1e39 reads as infinity, which would
/// overflow the ticks it becomes).
pub(super) fn time(v: f32, ctx: &str) -> Result<f32, Error> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(Error::new(format!("{ctx}: times must be numbers")))
    }
}

pub(crate) fn lookup(
    index: &HashMap<&str, usize>,
    key: &str,
    ctx: &str,
    kind: &str,
) -> Result<usize, Error> {
    index
        .get(key)
        .copied()
        .ok_or_else(|| Error::new(format!("{ctx}: unknown {kind} '{key}'")))
}

/// Build looks from content: ids must be unique, prices not negative, and (stored as a byte) at
/// most 250 of each.
pub(super) fn build_styles(raw: &[BuildStyleRaw], what: &str) -> Result<Vec<BuildStyle>, Error> {
    if raw.len() > 250 {
        return Err(Error::new(format!("too many {what}s (at most 250)")));
    }
    let mut out: Vec<BuildStyle> = Vec::with_capacity(raw.len());
    for r in raw {
        if out.iter().any(|s| s.id == r.id) {
            return Err(Error::new(format!("duplicate {what} '{}'", r.id)));
        }
        if r.price < 0 {
            return Err(Error::new(format!(
                "{what} '{}' has a negative price",
                r.id
            )));
        }
        out.push(BuildStyle {
            id: r.id.clone(),
            price: r.price,
            soft: r.finish.as_deref() == Some("carpet"),
        });
    }
    Ok(out)
}
