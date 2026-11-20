//! What has been accepted, and what a manifest still says.

use super::*;

/// A manifest without its model entries: what the models' rows do not cover. Nor its `$comment`, which says what
/// builds the pack (a script, then a crate) and which the game does not read.
pub(crate) fn rest_of(manifest: &J) -> J {
    let mut out = match manifest.entries() {
        Some(fields) => J::Obj(
            fields
                .iter()
                .filter(|(k, _)| k.as_str() != "$comment")
                .cloned()
                .collect(),
        ),
        None => manifest.clone(),
    };
    if let Some(entries) = manifest.get("entries").and_then(J::entries) {
        out.set(
            "entries",
            J::Obj(
                entries
                    .iter()
                    .filter(|(_, e)| e.get("type") != Some(&J::Str("model".into())))
                    .cloned()
                    .collect(),
            ),
        );
    }
    out
}

/// port-accepted.json, {row: {metric: why}}, and what is wrong with it: a metric that is none, a reason left out.
pub(crate) fn accepted(path: &str) -> (BTreeMap<String, BTreeMap<String, String>>, Vec<String>) {
    let (mut out, mut errors) = (BTreeMap::new(), Vec::new());
    let data = match std::fs::read(path) {
        Ok(data) => data,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (out, errors),
        Err(e) => return (out, vec![format!("{path}: {e}")]),
    };
    let json = match super::super::super::json::loads_text(&data) {
        Ok(json @ J::Obj(_)) => json,
        Ok(_) => return (out, vec![format!("{path} is {{key: {{metric: why}}}}")]),
        Err(e) => return (out, vec![format!("{path}: {e}")]),
    };
    for (row, metrics) in json.entries().unwrap_or(&[]) {
        let Some(metrics) = metrics.entries() else {
            errors.push(format!("{row}: its line is {{metric: why}}"));
            continue;
        };
        for (metric, why) in metrics {
            if !METRICS.iter().any(|(m, _)| m == metric) {
                errors.push(format!(
                    "{row}: {} is no metric ({})",
                    py::repr_str(metric),
                    METRICS
                        .iter()
                        .map(|(m, _)| *m)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            } else if !why.str().is_some_and(|w| !w.trim().is_empty()) {
                errors.push(format!(
                    "{row}: {metric} needs its reason: why the new build is right as it is"
                ));
            } else if metric == "decimated" && !row.ends_with(" far") {
                errors.push(format!("{row}: decimated is a far version's (`{row} far`)"));
            } else {
                out.entry(row.clone())
                    .or_insert_with(BTreeMap::new)
                    .insert(metric.clone(), why.str().unwrap_or("").to_string());
            }
        }
    }
    (out, errors)
}
