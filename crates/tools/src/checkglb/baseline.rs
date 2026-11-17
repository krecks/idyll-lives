//! The gate's baseline (`tools/art/gate-baseline.json`): how many findings of each counted check
//! (`checkglb::COUNTED`) each model had when the check came into the gate, `{check: {model key: count}}`.
//!
//! A check can be strict for every new model from its first day while the models it already finds stay
//! visible: a model's count at its baseline is printed as warnings; above it, or in a model the baseline doesn't
//! have, it fails; below it, it fails as stale until `cargo check-glb --update-baseline` lowers the entry, which
//! only lowers counts and removes entries. A check with no section in the file is strict for every model, so a
//! family becomes strict when its last key leaves. A check that is one finding or none per model counts 1 or 0.

use std::collections::BTreeMap;

/// The file, under a checkout's root.
pub const FILE: &str = "tools/art/gate-baseline.json";

/// What one counted check found in one model: how many (what the baseline holds) and what to print of it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Found {
    pub count: u64,
    pub text: Vec<String>,
}

/// One model's count of a check against its baseline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Verdict {
    /// None found, none in the baseline.
    Clean,
    /// As many as the baseline holds: printed, not failed.
    Held(u64),
    /// More than the baseline holds.
    Over { base: u64, count: u64 },
    /// Found in a model the baseline has none for: new models are strict from the start.
    New(u64),
    /// Fewer than the baseline holds: `--update-baseline` lowers it.
    Stale { base: u64, count: u64 },
}

impl Verdict {
    /// Whether the gate fails the model for it.
    pub fn fails(self) -> bool {
        !matches!(self, Verdict::Clean | Verdict::Held(_))
    }
}

/// One model's findings of a check sorted by the baseline (`sort`), as the gate prints them.
#[derive(Debug, Default, PartialEq)]
pub struct Sorted {
    /// Errors unless the model's entry `accept`s the check (the gate's strict findings).
    pub strict: Vec<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// `{check: {model key: count}}`, every count over 0.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Baseline(BTreeMap<String, BTreeMap<String, u64>>);

/// What a gate run counted: `{check: {model key: count}}` for each check that ran, 0 where it found none.
pub type Counts = BTreeMap<String, BTreeMap<String, u64>>;

impl Baseline {
    /// The baseline in `text` (JSON, `{check: {key: count}}`).
    pub fn parse(text: &str) -> Result<Baseline, String> {
        let checks: BTreeMap<String, BTreeMap<String, serde_json::Value>> =
            serde_json::from_str(text)
                .map_err(|e| format!("not {{check: {{key: count}}}}: {e}"))?;
        let mut out = BTreeMap::new();
        for (check, keys) in checks {
            let mut counts = BTreeMap::new();
            for (key, n) in keys {
                match n.as_u64().filter(|n| *n > 0) {
                    Some(n) => counts.insert(key, n),
                    None => {
                        return Err(format!(
                            "{check}.{key} is {n}: a count is a whole number over 0 (a model with none leaves the file)"
                        ));
                    }
                };
            }
            out.insert(check, counts);
        }
        Ok(Baseline(out))
    }

    /// The baseline at `path`: empty if there is no file.
    pub fn read(path: &str) -> Result<Baseline, String> {
        match std::fs::read_to_string(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Baseline::default()),
            Err(e) => Err(format!("{path}: {e}")),
            Ok(text) => Baseline::parse(&text).map_err(|e| format!("{path}: {e}")),
        }
    }

    /// Its text: sorted, two spaces an indent, a newline at the end (what `write` writes).
    pub fn text(&self) -> String {
        let mut out = serde_json::to_string_pretty(&self.0).expect("counts are JSON");
        out.push('\n');
        out
    }

    pub fn write(&self, path: &str) -> Result<(), String> {
        std::fs::write(path, self.text()).map_err(|e| format!("{path}: {e}"))
    }

    /// The checks it has a section for.
    pub fn checks(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }

    /// The model keys it holds for `check`.
    pub fn keys(&self, check: &str) -> impl Iterator<Item = &str> {
        self.0
            .get(check)
            .into_iter()
            .flat_map(|k| k.keys().map(String::as_str))
    }

    /// The count it holds for `check` in model `key` (0: none).
    pub fn get(&self, check: &str, key: &str) -> u64 {
        self.0
            .get(check)
            .and_then(|k| k.get(key))
            .copied()
            .unwrap_or(0)
    }

    /// Model `key`'s `count` of `check` against it.
    pub fn judge(&self, check: &str, key: &str, count: u64) -> Verdict {
        let base = self.get(check, key);
        match (base, count) {
            (0, 0) => Verdict::Clean,
            (0, count) => Verdict::New(count),
            (base, count) if count == base => Verdict::Held(count),
            (base, count) if count > base => Verdict::Over { base, count },
            (base, count) => Verdict::Stale { base, count },
        }
    }

    /// It lowered to what a run counted (`counts`; checks and keys the run didn't count are kept as they are):
    /// each count down to what was found, and out where none was; never raised, nothing added, a check's section
    /// gone with its last key. (The baseline, a line for each change.)
    pub fn lowered(&self, counts: &Counts) -> (Baseline, Vec<String>) {
        let mut out = self.0.clone();
        let mut changes = Vec::new();
        for (check, keys) in &mut out {
            let Some(found) = counts.get(check) else {
                continue;
            };
            keys.retain(|key, base| match found.get(key) {
                Some(0) => {
                    changes.push(format!("{check}: {key} out (was {base})"));
                    false
                }
                Some(&n) if n < *base => {
                    changes.push(format!("{check}: {key} {base} -> {n}"));
                    *base = n;
                    true
                }
                _ => true,
            });
            if keys.is_empty() {
                changes.push(format!("{check}: none left, strict for every model"));
            }
        }
        out.retain(|_, keys| !keys.is_empty());
        (Baseline(out), changes)
    }
}

/// Model `key`'s findings of `check` (`found`) by its `verdict`: over or new, its findings are strict (an
/// error each unless its entry accepts the check), led by the count over; held, warnings; stale, warnings and an
/// error that says to lower the baseline.
pub fn sort(check: &str, verdict: Verdict, found: Found) -> Sorted {
    let mut out = Sorted::default();
    let text = if found.text.is_empty() && found.count > 0 {
        vec![format!("{check}: {} found", found.count)]
    } else {
        found.text
    };
    match verdict {
        Verdict::Clean => {}
        Verdict::New(_) => out.strict = text,
        Verdict::Over { base, count } => {
            out.strict.push(format!(
                "{check}: {count} found, {base} in the baseline ({FILE}): fix the new ones"
            ));
            out.strict.extend(text);
        }
        Verdict::Held(count) => {
            out.warnings.push(format!(
                "{check}: {count} found, as the baseline has ({FILE})"
            ));
            out.warnings.extend(text);
        }
        Verdict::Stale { base, count } => {
            out.errors.push(format!(
                "{check}: {count} found, {base} in the baseline: lower it (`cargo check-glb --update-baseline`)"
            ));
            out.warnings.extend(text);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(text: &str) -> Baseline {
        Baseline::parse(text).unwrap()
    }

    fn found(count: u64) -> Found {
        Found {
            count,
            text: (0..count).map(|i| format!("finding {i}")).collect(),
        }
    }

    #[test]
    fn the_text_is_sorted_one_key_a_line_with_a_newline_at_the_end() {
        let b = base(r#"{"edges": {"model.b": 2, "model.a": 12}, "texels": {"model.c": 1}}"#);
        assert_eq!(
            b.text(),
            "{\n  \"edges\": {\n    \"model.a\": 12,\n    \"model.b\": 2\n  },\n  \"texels\": {\n    \"model.c\": 1\n  }\n}\n"
        );
        assert_eq!(base(&b.text()), b);
        assert_eq!(Baseline::default().text(), "{}\n");
    }

    #[test]
    fn a_count_is_a_whole_number_over_0() {
        for bad in ["0", "-1", "1.5", "\"2\"", "null"] {
            let e = Baseline::parse(&format!(r#"{{"edges": {{"model.a": {bad}}}}}"#)).unwrap_err();
            assert!(e.starts_with("edges.model.a is "), "{bad}: {e}");
        }
        assert!(Baseline::parse(r#"{"edges": [1]}"#).is_err());
        assert!(Baseline::parse("").is_err());
    }

    #[test]
    fn no_file_is_an_empty_baseline() {
        let b = Baseline::read("/nonexistent/gate-baseline.json").unwrap();
        assert_eq!(b, Baseline::default());
        assert_eq!(b.checks().count(), 0);
    }

    #[test]
    fn a_count_is_judged_against_its_model_s() {
        let b = base(r#"{"edges": {"model.a": 3}}"#);
        assert_eq!(b.judge("edges", "model.a", 3), Verdict::Held(3));
        assert_eq!(
            b.judge("edges", "model.a", 4),
            Verdict::Over { base: 3, count: 4 }
        );
        assert_eq!(
            b.judge("edges", "model.a", 1),
            Verdict::Stale { base: 3, count: 1 }
        );
        assert_eq!(
            b.judge("edges", "model.a", 0),
            Verdict::Stale { base: 3, count: 0 }
        );
        // A model the section doesn't have, and a check with no section: strict.
        assert_eq!(b.judge("edges", "model.b", 2), Verdict::New(2));
        assert_eq!(b.judge("texels", "model.a", 1), Verdict::New(1));
        assert_eq!(b.judge("texels", "model.a", 0), Verdict::Clean);
        let fails: Vec<bool> = [0, 1, 3, 4]
            .iter()
            .map(|n| b.judge("edges", "model.a", *n).fails())
            .collect();
        assert_eq!(fails, [true, true, false, true]);
        assert!(!Verdict::Clean.fails());
    }

    #[test]
    fn findings_over_the_baseline_are_strict_and_at_it_warnings() {
        let s = sort("edges", Verdict::New(2), found(2));
        assert_eq!(
            (s.strict.len(), s.errors.len(), s.warnings.len()),
            (2, 0, 0)
        );
        let s = sort("edges", Verdict::Over { base: 1, count: 2 }, found(2));
        assert_eq!(
            s.strict[0],
            "edges: 2 found, 1 in the baseline (tools/art/gate-baseline.json): fix the new ones"
        );
        assert_eq!(s.strict.len(), 3);
        let s = sort("edges", Verdict::Held(2), found(2));
        assert_eq!(s.strict.len() + s.errors.len(), 0);
        assert_eq!(
            s.warnings,
            [
                "edges: 2 found, as the baseline has (tools/art/gate-baseline.json)",
                "finding 0",
                "finding 1"
            ]
        );
        let s = sort("edges", Verdict::Stale { base: 3, count: 1 }, found(1));
        assert_eq!(
            s.errors,
            ["edges: 1 found, 3 in the baseline: lower it (`cargo check-glb --update-baseline`)"]
        );
        assert_eq!(s.warnings, ["finding 0"]);
        assert_eq!(sort("edges", Verdict::Clean, found(0)), Sorted::default());
        // A count with nothing to print says how many.
        let s = sort(
            "validator",
            Verdict::New(1),
            Found {
                count: 1,
                text: Vec::new(),
            },
        );
        assert_eq!(s.strict, ["validator: 1 found"]);
    }

    #[test]
    fn update_only_lowers_and_removes() {
        let b = base(
            r#"{"edges": {"model.a": 3, "model.b": 2, "model.c": 5}, "texels": {"model.a": 1}, "pbr": {"model.a": 4}}"#,
        );
        let counts: Counts = serde_json::from_str(
            r#"{"edges": {"model.a": 1, "model.b": 0, "model.c": 9, "model.new": 4}, "texels": {"model.a": 0}}"#,
        )
        .unwrap();
        let (lowered, changes) = b.lowered(&counts);
        // a lowered, b out, c not raised, model.new not added; texels gone with its last key; pbr not counted: kept.
        assert_eq!(
            lowered,
            base(r#"{"edges": {"model.a": 1, "model.c": 5}, "pbr": {"model.a": 4}}"#)
        );
        assert_eq!(
            changes,
            [
                "edges: model.a 3 -> 1",
                "edges: model.b out (was 2)",
                "texels: model.a out (was 1)",
                "texels: none left, strict for every model",
            ]
        );
        // Again: nothing to change.
        let (again, changes) = lowered.lowered(&counts);
        assert_eq!((again, changes.len()), (lowered, 0));
    }
}
