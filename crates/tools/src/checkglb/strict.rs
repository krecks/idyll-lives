//! Strict mode: the tags that carry a strict failure up through a check.

use super::*;

/// The checks with one right answer (tools/art/README.md, "The gate"): each an error unless the model's pack entry
/// `accept`s it by this name with its reason (`"accept": {"footprint": "why"}`), as the curtains' rail runs past the
/// window. The rest are warnings: the budgets' aims, and what only an eye can judge (a loop that ends turned or
/// moved, a part turning fast, a long use clip, a texture of its own) and the functional heights.
pub const STRICT: &[(&str, &str)] = &[
    (
        "radius",
        "a thing on a top reaches over 10 % past its content `radius`",
    ),
    (
        "footprint",
        "it reaches past its footprint (or the tiles of its `overhang`)",
    ),
    ("sweep", "a part that goes round sweeps past its footprint"),
    ("wall", "it reaches into the wall"),
    (
        "thin",
        "on the floor, it or a moving part is lower than the floor drawn over it",
    ),
    ("float", "a floor thing floats above its origin"),
    ("centre", "it is off its Model's `centre`"),
    ("flipped", "faces have normals turned against their winding"),
    (
        "flat",
        "a part flattened at rest shows its flat shape (\"Grown from nothing\")",
    ),
    ("unplayed", "a clip plays for no use"),
    ("cubic", "a clip has cubic-spline keys"),
    ("cameras", "the file has cameras"),
    ("lights", "the file has lights"),
    ("mirror", "mirror glass is see-through or isn't flat"),
    (
        "materials",
        "materials unmapped, double-sided, see-through (BLEND), glowing by day or with unused UVs",
    ),
    ("far", "its far version isn't the full model's size"),
    (
        "validator",
        "the Khronos glTF validator reports an error or a warning",
    ),
    (
        "reference",
        "it is outside its kind's range in docs/review/reference.json",
    ),
    ("edges", "hard edges show unbevelled at the play camera"),
    (
        "texels",
        "a textured finish's texels are too sparse, too dense or stretched",
    ),
    (
        "pbr",
        "a finish's PBR values are outside their plausible ranges",
    ),
];

/// The strict checks counted per model against the baseline (`baseline`): each came into the gate after the models
/// it finds, so a model's findings up to its count in tools/art/gate-baseline.json are warnings, and more, or any
/// in a model the file doesn't have, fail. Some run only when asked (`--validate`) or outside the gate (`cargo
/// review sizes`), so an `accept` of one is unused only where it ran.
pub const COUNTED: [&str; 5] = ["validator", "reference", "edges", "texels", "pbr"];

/// Marks a finding of the strict check `kind` (`STRICT`), carried among the warnings until `strictly` sorts it.
const TAG: char = '\u{1f}';

pub(super) fn strict(kind: &str, text: String) -> String {
    debug_assert!(STRICT.iter().any(|(k, _)| *k == kind), "{kind}");
    format!("{TAG}{kind}{TAG}{text}")
}

/// (kind, text) of a strict finding (`strict`), or None.
pub(super) fn untag(s: &str) -> Option<(&str, &str)> {
    s.strip_prefix(TAG)?.split_once(TAG)
}

/// The findings of one model with its strict ones sorted out (`STRICT`): an error each, or, where its entry's
/// `accept` names the check, an accepted note with the reason. An `accept` that names no strict check, gives no
/// reason, or that the model no longer needs is an error: a stale one would hide the next (a counted check's
/// only where it ran, `ran`). Returns (errors, warnings, accepted).
pub(super) fn strictly(
    entry: &J,
    errors: Vec<String>,
    warnings: Vec<String>,
    ran: &[&str],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let none = J::Obj(Vec::new());
    let accept = entry.get("accept").unwrap_or(&none);
    let mut out = (Vec::new(), Vec::new(), Vec::new());
    let mut reasons: BTreeMap<String, Option<String>> = BTreeMap::new();
    // (What is wrong with `accept` itself comes after what the model is found to be.)
    let mut problems = Vec::new();
    match accept.entries() {
        Some(list) => {
            for (k, reason) in list {
                let reason = reason.str().map(str::trim).filter(|r| !r.is_empty());
                if !STRICT.iter().any(|(s, _)| s == k) {
                    let names = STRICT
                        .iter()
                        .map(|(s, _)| *s)
                        .collect::<Vec<_>>()
                        .join(", ");
                    problems.push(format!(
                        "`accept` names {}, no check of the gate's (one of {names})",
                        repr_str(k)
                    ));
                } else if reason.is_none() {
                    problems.push(format!(
                        "`accept.{k}` needs its reason: why the model is right as it is"
                    ));
                }
                reasons.insert(k.clone(), reason.map(String::from));
            }
        }
        None => problems.push("`accept` is {check: reason}".into()),
    }
    let mut used = HashSet::new();
    for (is_error, text) in errors
        .into_iter()
        .map(|e| (true, e))
        .chain(warnings.into_iter().map(|w| (false, w)))
    {
        match untag(&text) {
            Some((kind, plain)) => match reasons.get(kind) {
                Some(Some(reason)) => {
                    used.insert(kind.to_string());
                    out.2.push(format!("{plain} ({kind}: {reason})"));
                }
                Some(None) => {
                    used.insert(kind.to_string());
                    out.0.push(plain.to_string());
                }
                None => out.0.push(plain.to_string()),
            },
            None if is_error => out.0.push(text),
            None => out.1.push(text),
        }
    }
    out.0.extend(problems);
    for (k, reason) in &reasons {
        if reason.is_some()
            && STRICT.iter().any(|(s, _)| s == k)
            && (!COUNTED.contains(&k.as_str()) || ran.contains(&k.as_str()))
            && !used.contains(k)
        {
            out.0.push(format!("`accept.{k}` is unused: the model passes that check (take it out of its Model or entry)"));
        }
    }
    out
}
