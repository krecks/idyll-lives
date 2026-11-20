//! Reports: the gate, its baseline and updating one.

use super::*;

/// The largest side (px) of a texture of its own for a model `side` m across (`TEXTURE_MAX`).
pub(super) fn texture_most(side: f64) -> u32 {
    TEXTURE_MAX
        .iter()
        .find(|(up, _)| side <= *up)
        .map_or(TEXTURE_MAX[1].1, |(_, most)| *most)
}

/// What the gate found: its text, the keys it checked (in order), the model file of each, whether anything
/// failed, and what the counted checks that ran counted (`Baseline::lowered`).
pub struct Report {
    pub text: String,
    pub checked: Vec<String>,
    pub files: Vec<String>,
    pub failed: usize,
    pub counts: Counts,
}

/// Runs the gate over `keys` (every model if none) into a report.
pub fn gate(paths: &Paths, keys: &[String]) -> Report {
    gate_with(paths, keys, &[])
}

/// The gate with counted checks run over every file at once (`BatchCheck`), each by its name in `COUNTED`.
pub fn gate_with(paths: &Paths, keys: &[String], batch: &[(&'static str, BatchCheck)]) -> Report {
    run_gate(paths, keys, FILE_CHECKS, batch)
}

pub(super) fn run_gate(
    paths: &Paths,
    keys: &[String],
    file: &[(&'static str, FileCheck)],
    batch: &[(&'static str, BatchCheck)],
) -> Report {
    let mut text = String::new();
    let fail = |text: String| Report {
        text,
        checked: Vec::new(),
        files: Vec::new(),
        failed: 1,
        counts: Counts::new(),
    };
    let read = || -> Result<_, String> {
        let stack = asset_manifests(paths)?;
        let defs = content(paths)?;
        Ok((
            stack,
            content_objects(&defs)?,
            content_keys(&defs)?,
            Furniture::read(paths)?,
        ))
    };
    let (stack, facts, named, furniture) = match read() {
        Ok(r) => r,
        Err(err) => {
            return fail(format!(
                "FAIL content and manifests\n       error: cannot read them: {err}\n"
            ));
        }
    };
    let (entries, fin) = model_entries(&stack);
    let centres = match read_centres(&paths.centres) {
        Ok(c) => c,
        Err(err) => {
            return fail(format!(
                "FAIL centres.json\n       error: cannot read it: {err}\n"
            ));
        }
    };
    let baseline = match Baseline::read(&paths.baseline) {
        Ok(b) => b,
        Err(err) => {
            return fail(format!(
                "FAIL gate-baseline.json\n       error: cannot read it: {err}\n"
            ));
        }
    };
    let mut failed = 0;
    let mut checked = Vec::new();
    let mut files = Vec::new();
    // What can't be read at all: a model the content draws (or the command line names) that no manifest has,
    // or whose entry has no file once the packs are stacked (its pack missing from `packs`), and a file that
    // is no .glb. The game draws a box for each.
    let mut problems: BTreeMap<String, String> = BTreeMap::new();
    let wanted = |k: &str| keys.is_empty() || keys.iter().any(|x| x == k);
    let all: BTreeSet<&String> = named.keys().chain(keys.iter()).collect();
    for k in all {
        if !wanted(k) {
            continue;
        }
        let what = named
            .get(k)
            .map(String::as_str)
            .unwrap_or("the command line");
        let url = entries
            .get(k)
            .and_then(|(e, _)| e.get("url"))
            .is_some_and(J::truthy);
        if k.is_empty() {
            problems.insert(k.clone(), format!("{what} has an empty `model`: the game looks up \"\" and draws a box (leave it out to draw model.<id>)"));
        } else if !entries.contains_key(k) {
            problems.insert(
                k.clone(),
                format!("{what} names it, but no manifest has a model entry {k}"),
            );
        } else if !url {
            problems.insert(k.clone(), format!("{what} names it, but its entry has no `url` once the packs are stacked (is its pack in the base manifest's `packs`?)"));
        }
    }
    for (k, (entry, _)) in &entries {
        if let Some(url) = entry.get("url").and_then(J::str).filter(|u| !u.is_empty())
            && !url.to_lowercase().ends_with(".glb")
            && !url.contains("://")
            && wanted(k)
        {
            problems.insert(
                k.clone(),
                format!("its file {url} is no .glb: the game's models are glTF binaries"),
            );
        }
    }
    for (k, problem) in &problems {
        failed += 1;
        text += &format!(
            "FAIL {}\n       error: {problem}\n",
            if k.is_empty() { "(empty model)" } else { k }
        );
    }
    // Each model to check with its file and its far version's.
    let mut todo = Vec::new();
    for (k, (entry, manifest)) in &entries {
        let url = entry
            .get("url")
            .filter(|u| u.truthy())
            .map(J::key)
            .unwrap_or_default();
        if !wanted(k) || problems.contains_key(k) || url.is_empty() || url.contains("://") {
            continue;
        }
        let path = normpath(&join(&dirname(manifest), &url));
        let far = entry
            .get("far")
            .filter(|f| f.truthy())
            .map(|f| normpath(&join(&dirname(manifest), &f.key())));
        todo.push((k, entry, path, far));
    }
    let mut counting = Counting {
        baseline: &baseline,
        file,
        batch: BTreeMap::new(),
    };
    let model_files: Vec<(String, String, Option<String>)> = todo
        .iter()
        .map(|(k, _, path, far)| ((*k).clone(), path.clone(), far.clone()))
        .collect();
    for (name, run) in batch {
        match run(&model_files) {
            Ok(found) => {
                counting.batch.insert(name, found);
            }
            Err(err) => {
                failed += 1;
                text += &format!("FAIL {name}\n       error: {err}\n");
            }
        }
    }
    let mut counts = Counts::new();
    for (k, entry, path, far) in todo {
        let Checked {
            errors,
            warnings,
            accepted,
            summary,
            counts: counted,
        } = check(
            k,
            &fin,
            entry,
            &path,
            far.as_deref(),
            facts.get(k),
            centres.get(k),
            Some(&furniture),
            &counting,
        )
        .unwrap_or_else(|err| Checked {
            errors: vec![format!("cannot read {path}: {err}")],
            warnings: Vec::new(),
            accepted: Vec::new(),
            summary: String::new(),
            counts: Vec::new(),
        });
        for (c, n) in counted {
            counts.entry(c.into()).or_default().insert(k.clone(), n);
        }
        failed += !errors.is_empty() as usize;
        checked.push(k.clone());
        files.push(path.clone());
        text += &format!(
            "{} {k:28} {summary}\n",
            if errors.is_empty() { "ok  " } else { "FAIL" }
        );
        for e in errors {
            text += &format!("       error: {e}\n");
        }
        for w in warnings {
            text += &format!("       warning: {w}\n");
        }
        for a in accepted {
            text += &format!("       accepted: {a}\n");
        }
    }
    // The baseline itself: a section for no counted check, and, over every model, a key of a check that ran that is
    // no model the gate checked (counted as none found, so `--update-baseline` takes it out).
    let mut stale = Vec::new();
    let seen: HashSet<&str> = checked.iter().map(String::as_str).collect();
    for check in baseline.checks() {
        // (The built world's sections are its own gate's: `built`.)
        if built::CHECKS.contains(&check) {
            continue;
        }
        if !COUNTED.contains(&check) {
            stale.push(format!(
                "{} is no counted check (one of {}): take it out",
                repr_str(check),
                COUNTED.join(", ")
            ));
            continue;
        }
        let Some(ran) = counts.get_mut(check).filter(|_| keys.is_empty()) else {
            continue;
        };
        for key in baseline.keys(check).filter(|k| !seen.contains(k)) {
            stale.push(format!("{check}: {key} is no model the gate checked: take it out (`cargo check-glb --update-baseline`)"));
            ran.insert(key.into(), 0);
        }
    }
    if !stale.is_empty() {
        failed += 1;
        text += "FAIL gate-baseline.json\n";
        for e in stale {
            text += &format!("       error: {e}\n");
        }
    }
    text += &format!("{} models, {failed} failed", checked.len());
    if !problems.is_empty() {
        text += &format!(" ({} without a file it can read)", problems.len());
    }
    text += "\n";
    Report {
        text,
        checked,
        files,
        failed,
        counts,
    }
}

/// Lowers the baseline to what the gate counts over `keys` (every model if none; `Baseline::lowered`) and writes
/// it if that changes it: a line for each change.
pub fn update_baseline(
    paths: &Paths,
    keys: &[String],
    batch: &[(&'static str, BatchCheck)],
) -> Result<Vec<String>, String> {
    update_with(paths, keys, FILE_CHECKS, batch)
}

pub(super) fn update_with(
    paths: &Paths,
    keys: &[String],
    file: &[(&'static str, FileCheck)],
    batch: &[(&'static str, BatchCheck)],
) -> Result<Vec<String>, String> {
    let report = run_gate(paths, keys, file, batch);
    let (lowered, changes) = Baseline::read(&paths.baseline)?.lowered(&report.counts);
    if !changes.is_empty() {
        lowered.write(&paths.baseline)?;
    }
    Ok(changes)
}

/// The model entries of the manifests once stacked ({key: (entry, the manifest it's in)}), and their finishes.
pub(super) fn model_entries(stack: &[(String, J)]) -> (BTreeMap<String, (J, String)>, Finishes) {
    let mut entries: BTreeMap<String, (J, String)> = BTreeMap::new();
    let mut fin = Finishes {
        plain: HashMap::new(),
        see_through: HashSet::new(),
        textures: HashMap::new(),
    };
    for (path, m) in stack {
        for (k, entry) in m
            .get("entries")
            .filter(|e| e.truthy())
            .and_then(J::entries)
            .unwrap_or(&[])
        {
            let kind = entry.get("type");
            if kind == Some(&J::Str("model".into())) {
                // A pack's entry replaces the entry, keeping its `vary` unless it gives its own
                // (registry.ts `extended`).
                let mut entry = entry.clone();
                if let Some((old, _)) = entries.get(k).filter(|(old, _)| old.truthy())
                    && let (Some(vary), J::Obj(fields)) = (old.get("vary"), &mut entry)
                {
                    match fields.iter().position(|(f, _)| f == "vary") {
                        Some(at) => {
                            let mine = fields.remove(at);
                            fields.insert(0, mine);
                        }
                        None => fields.insert(0, ("vary".into(), vary.clone())),
                    }
                }
                entries.insert(k.clone(), (entry, path.clone()));
            } else if kind == Some(&J::Str("material".into())) && k.starts_with("material.finish.")
            {
                let alpha = entry.get("alpha").and_then(J::num).map_or(1.0, |a| a.f());
                fin.plain.insert(
                    k.clone(),
                    !entry.get("texture").is_some_and(J::truthy) && alpha >= 1.0,
                );
                if alpha < 1.0 {
                    fin.see_through.insert(k.clone());
                }
                match entry
                    .get("texture")
                    .and_then(J::str)
                    .filter(|t| !t.is_empty())
                {
                    Some(texture) => {
                        let scale = entry.get("uvScale").and_then(J::num).map_or(1.0, |s| s.f());
                        let file = normpath(&join(&dirname(path), texture));
                        fin.textures.insert(k.clone(), (scale, file));
                    }
                    None => {
                        fin.textures.remove(k);
                    }
                }
            }
        }
    }
    (entries, fin)
}
