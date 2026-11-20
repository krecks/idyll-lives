//! The comparison itself.

use super::*;
use crate::checkglb::against::{Class, found, json_differ, names};
use crate::checkglb::py::{self, J, join};
use crate::checkglb::{Furniture, Paths};
use std::collections::{BTreeMap, BTreeSet};

/// Compares the new side with the old (`Options`), every row on as many threads as the machine has.
pub fn against(opts: &Options) -> Result<Outcome, String> {
    let scratch = Scratch::new()?;
    let tree = Paths::at(&opts.root);
    let old = Side::open("old", &opts.old, &opts.root, &scratch)?;
    let new = Side::open("new", &opts.new, &opts.root, &scratch)?;
    let partial = matches!(new.origin, Origin::Over { .. });
    let all: BTreeSet<&String> = old.entries.keys().chain(new.entries.keys()).collect();
    let keys: Vec<String> = if !opts.keys.is_empty() {
        let mut keys = opts.keys.clone();
        keys.sort();
        keys.dedup();
        keys
    } else if let Some(family) = &opts.family {
        let of = |side: &Side, key: &str| {
            side.entries.get(key).is_some_and(|(e, m)| {
                families(e, m)
                    .iter()
                    .any(|f| family_name(f) == family_name(family))
            })
        };
        let keys: Vec<String> = all
            .iter()
            .filter(|k| of(&old, k) || of(&new, k))
            .map(|k| k.to_string())
            .collect();
        if keys.is_empty() {
            let mut known: BTreeSet<String> = BTreeSet::new();
            for side in [&old, &new] {
                known.extend(side.entries.values().flat_map(|(e, m)| families(e, m)));
            }
            return Err(format!(
                "no model is of the family {} ({})",
                py::repr_str(family),
                known.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
        keys
    } else if partial {
        let keys: Vec<String> = all
            .iter()
            .filter(|k| {
                ["url", "far"]
                    .iter()
                    .any(|f| new.file(k, f).is_some_and(|p| new.built(&p)))
            })
            .map(|k| k.to_string())
            .collect();
        if keys.is_empty() {
            return Err(
                "the build has no model file a manifest names: name keys or a family".into(),
            );
        }
        keys
    } else {
        all.iter().map(|k| k.to_string()).collect()
    };
    for side in [&old, &new] {
        let files: Vec<String> = keys
            .iter()
            .flat_map(|k| [side.file(k, "url"), side.file(k, "far")])
            .flatten()
            .collect();
        side.fetch(&files)?;
    }

    let defs = crate::checkglb::content(&tree)?;
    let path = opts
        .accepted
        .clone()
        .unwrap_or_else(|| join(&opts.root, "tools/art/port-accepted.json"));
    let (accepted, mut errors) = accepted(&path);
    let context = Context {
        facts: crate::checkglb::content_objects(&defs)?,
        furniture: Furniture::read(&tree)?,
        old,
        new,
        accepted,
    };

    let next = std::sync::atomic::AtomicUsize::new(0);
    let done: std::sync::Mutex<Vec<Vec<Row>>> =
        std::sync::Mutex::new(keys.iter().map(|_| Vec::new()).collect());
    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(keys.len().max(1));
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(key) = keys.get(i) else { break };
                    let rows = context.rows(key);
                    done.lock().unwrap_or_else(|e| e.into_inner())[i] = rows;
                }
            });
        }
    });
    let mut rows: Vec<Row> = done
        .into_inner()
        .unwrap_or_else(|e| e.into_inner())
        .into_iter()
        .flatten()
        .collect();
    let models = keys.len();
    let fars = rows.len() - models;

    // The manifests' own lines (not for keys named one by one): those the models compared are in, and the assets'
    // own when every model is compared; then the centres of what no manifest has.
    let whole = opts.keys.is_empty() && opts.family.is_none() && !partial;
    let mut manifests = 0;
    if opts.keys.is_empty() {
        let of = |side: &Side| -> BTreeMap<String, J> {
            let used: BTreeSet<&String> = keys
                .iter()
                .filter_map(|k| side.entries.get(k).map(|(_, m)| m))
                .collect();
            side.stack
                .iter()
                .filter(|(path, _)| whole || used.contains(path))
                .map(|(path, m)| (side.manifest_name(path), rest_of(m)))
                .collect()
        };
        let (a, b) = (of(&context.old), of(&context.new));
        let names: BTreeSet<&String> = a.keys().chain(b.keys()).collect();
        manifests = names.len();
        for name in names {
            let mut differs = Vec::new();
            json_differ(a.get(name), b.get(name), "", &mut differs);
            let findings = if differs.is_empty() {
                Vec::new()
            } else {
                vec![found(
                    "pack",
                    format!("but for its models' entries: {}", self::names(&differs)),
                )]
            };
            let same = findings.is_empty();
            rows.push(context.classed(
                Row {
                    name: name.clone(),
                    class: Class::Same,
                    findings,
                    accepted: Vec::new(),
                    notes: Vec::new(),
                    numbers: None,
                },
                same,
            ));
        }
    }
    if whole {
        let loose = |side: &Side| -> J {
            J::Obj(
                side.centres
                    .entries()
                    .unwrap_or(&[])
                    .iter()
                    .filter(|(k, _)| !all_has(&context, k))
                    .cloned()
                    .collect(),
            )
        };
        let mut differs = Vec::new();
        json_differ(
            Some(&loose(&context.old)),
            Some(&loose(&context.new)),
            "",
            &mut differs,
        );
        if !differs.is_empty() {
            let row = Row {
                name: "centres.json".into(),
                class: Class::Same,
                findings: vec![found(
                    "centre",
                    format!("of keys no manifest has: {}", names(&differs)),
                )],
                accepted: Vec::new(),
                notes: Vec::new(),
                numbers: None,
            };
            rows.push(context.classed(row, false));
        }
    }

    // A line of port-accepted.json no row needs hides the next finding: an error, as the gate's `accept`.
    for (name, metrics) in &context.accepted {
        let Some(row) = rows.iter().find(|r| r.name == *name) else {
            if whole {
                errors.push(format!(
                    "{name}: no model, far version or manifest of that name is compared"
                ));
            }
            continue;
        };
        // (But `decimated`: what the far version is, whether this build needs its rule or not.)
        for metric in metrics.keys().filter(|m| *m != "decimated") {
            if !row.accepted.iter().any(|(f, _)| f.metric == metric) {
                errors.push(format!(
                    "{name}: {metric} is accepted, but the row passes without it: take it out"
                ));
            }
        }
    }

    let listed = !opts.keys.is_empty() || opts.family.is_some();
    let mut text = String::new();
    for row in &rows {
        let class = match row.class {
            Class::Same => "same",
            Class::Near => "near",
            Class::Changed => "CHANGED",
        };
        if row.class == Class::Same && !listed {
            continue;
        }
        text += &format!("{class:8} {}", row.name);
        if let Some(numbers) = row.numbers.as_ref().filter(|_| row.class != Class::Same) {
            text += &format!("  ({})", numbers.text());
        }
        text.push('\n');
        for f in &row.findings {
            text += &format!("         {}: {}\n", f.metric, f.text);
        }
        for (f, why) in &row.accepted {
            text += &format!("         accepted: {}: {} ({why})\n", f.metric, f.text);
        }
        for note in &row.notes {
            text += &format!("         note: {note}\n");
        }
    }
    for e in &errors {
        text += &format!("error: port-accepted.json: {e}\n");
    }
    let outcome = Outcome {
        rows,
        errors,
        text: String::new(),
    };
    text += &format!(
        "{models} models, {fars} far versions and {manifests} manifests: {} same, {} near, {} CHANGED\n",
        outcome.count(Class::Same),
        outcome.count(Class::Near),
        outcome.count(Class::Changed)
    );
    Ok(Outcome { text, ..outcome })
}

/// Whether either side's manifests have a model entry `key`.
pub(crate) fn all_has(context: &Context, key: &str) -> bool {
    context.old.entries.contains_key(key) || context.new.entries.contains_key(key)
}
