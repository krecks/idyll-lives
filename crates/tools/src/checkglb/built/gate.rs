//! The gate: running the checks over every sample.

use super::*;

// ---- the gate -----------------------------------------------------------------------------------------

/// A sample as the gate measured it.
#[derive(Clone, Debug, Default)]
pub(crate) struct Measured {
    pub sample: Listed,
    /// What a camera draws of it, and what only the shadow map does.
    pub triangles: usize,
    pub draws: usize,
    pub shadow_triangles: usize,
    pub shadow_draws: usize,
    /// Its hard edges and its textured parts' texels (None: a far version, not judged for them).
    pub edges: Option<Edges>,
    pub materials: Option<Vec<texels::Material>>,
    /// Its look, where it is one of a pair, and where a far version's disagrees with its near one's.
    pub look: Option<Look>,
    pub far_near: Option<Vec<String>>,
    /// A house's detail.
    pub detail: Option<Detail>,
}

/// A gate run: what it prints, how many samples failed, what it counted and measured.
pub(crate) struct Report {
    pub text: String,
    pub failed: usize,
    pub counts: Counts,
    pub samples: Vec<Measured>,
    pub textures: Textures,
    /// The looks of the samples that have far versions, by key.
    pub near: HashMap<String, Look>,
}

pub(crate) fn fail(text: String) -> Report {
    Report {
        text,
        failed: 1,
        counts: Counts::new(),
        samples: Vec::new(),
        textures: Textures::new(),
        near: HashMap::new(),
    }
}

/// `dir`'s `built.json`.
pub(crate) fn index(dir: &str) -> Result<Index, String> {
    let path = join(dir, "built.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("{path}: {e} (`bun tools/built/dump.ts {dir}` writes it)"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))
}

/// The gate over the samples dumped in `dir` (`keys`: only these), judged by the baseline at `paths`.
pub(crate) fn gate(paths: &Paths, dir: &str, keys: &[String]) -> Report {
    let read = || -> Result<_, String> {
        Ok((
            index(dir)?,
            surfaces(paths)?,
            Baseline::read(&paths.baseline)?,
        ))
    };
    let (index, surfaces, baseline) = match read() {
        Ok(r) => r,
        Err(err) => {
            return fail(format!(
                "FAIL the built world\n       error: cannot read it: {err}\n"
            ));
        }
    };
    if let Some(key) = keys
        .iter()
        .find(|k| !index.samples.iter().any(|s| &s.key == *k))
    {
        return fail(format!(
            "FAIL {key}\n       error: built.json lists no such sample\n"
        ));
    }
    let mut pictures = Pictures::default();
    let textures = textures(&surfaces, &mut pictures);
    let parts_of = |key: &str| -> Result<Vec<Part>, String> {
        let path = join(dir, &format!("{key}.glb"));
        let data = std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
        read_parts(&data).map_err(|e| format!("{path}: {e}"))
    };
    let mut text = String::new();
    let mut failed = 0;
    let mut counts = Counts::new();
    let mut samples = Vec::new();
    // The near samples' looks, each measured once for all its far versions.
    let mut looks: HashMap<String, Look> = HashMap::new();
    for sample in index
        .samples
        .iter()
        .filter(|s| keys.is_empty() || keys.contains(&s.key))
    {
        let measure = |pictures: &mut Pictures,
                       looks: &mut HashMap<String, Look>|
         -> Result<Measured, String> {
            let parts = parts_of(&sample.key)?;
            let mut m = Measured {
                sample: sample.clone(),
                ..Measured::default()
            };
            for p in parts.iter().filter(|p| !p.mesh.triangles.is_empty()) {
                let (triangles, draws) = if p.shadow_only {
                    (&mut m.shadow_triangles, &mut m.shadow_draws)
                } else {
                    (&mut m.triangles, &mut m.draws)
                };
                *triangles += p.mesh.triangles.len();
                *draws += 1;
            }
            if !DISTANT.contains(&sample.kind.as_str()) {
                m.edges = Some(sample_edges(sample, &parts, &surfaces, &index));
                m.materials = Some(texels::measure(&textured(&parts, &surfaces), &textures));
            }
            if sample.kind == "house" {
                m.detail = Some(detail(&parts, &surfaces));
            }
            if FAR.contains(&sample.kind.as_str()) {
                let of = sample
                    .of
                    .as_ref()
                    .ok_or("a far version, but built.json doesn't say of what (`of`)")?;
                if !looks.contains_key(of) {
                    let near = look(&parts_of(of)?, &surfaces, pictures);
                    looks.insert(of.clone(), near);
                }
                let far = look(&parts, &surfaces, pictures);
                m.far_near = Some(disagreements(&looks[of], &far));
                m.look = Some(far);
            }
            Ok(m)
        };
        let m = match measure(&mut pictures, &mut looks) {
            Ok(m) => m,
            Err(err) => {
                failed += 1;
                text += &format!("FAIL {}\n       error: {err}\n", sample.key);
                continue;
            }
        };
        let mut found: Vec<(&str, Found)> = Vec::new();
        if let Some(edges) = &m.edges {
            found.push((
                EDGES,
                Found {
                    count: edges.count() as u64,
                    text: edges_line(edges).into_iter().collect(),
                },
            ));
        }
        if let Some(materials) = &m.materials {
            let lines: Vec<String> = materials.iter().flat_map(texels::findings).collect();
            found.push((
                TEXELS,
                Found {
                    count: lines.len() as u64,
                    text: lines,
                },
            ));
        }
        if let Some(lines) = &m.far_near {
            found.push((
                FAR_NEAR,
                Found {
                    count: lines.len() as u64,
                    text: lines.clone(),
                },
            ));
        }
        let (mut errors, mut warnings) = (Vec::new(), Vec::new());
        for (check, found) in found {
            counts
                .entry(check.into())
                .or_default()
                .insert(sample.key.clone(), found.count);
            let sorted = baseline::sort(
                check,
                baseline.judge(check, &sample.key, found.count),
                found,
            );
            errors.extend(sorted.strict);
            errors.extend(sorted.errors);
            warnings.extend(sorted.warnings);
        }
        failed += usize::from(!errors.is_empty());
        text += &format!(
            "{} {:28} {} triangles, {} draws{}\n",
            if errors.is_empty() { "ok  " } else { "FAIL" },
            sample.key,
            m.triangles,
            m.draws,
            if m.shadow_draws > 0 {
                format!(
                    " (and {} triangles in {} only the shadow map draws)",
                    m.shadow_triangles, m.shadow_draws
                )
            } else {
                String::new()
            }
        );
        // (The baseline's own lines name the models' command: this gate's lowers these sections.)
        let own = |line: String| {
            line.replace(
                "`cargo check-glb --update-baseline`",
                "`cargo check-glb --built <dir> --update-baseline`",
            )
        };
        for e in errors {
            text += &format!("       error: {}\n", own(e));
        }
        for w in warnings {
            text += &format!("       warning: {}\n", own(w));
        }
        samples.push(m);
    }
    // The baseline's own keys: one that is no sample a check ran on is stale (counted as none found).
    let mut stale = Vec::new();
    if keys.is_empty() {
        for check in CHECKS {
            let ran = counts.entry(check.into()).or_default();
            for key in baseline
                .keys(check)
                .filter(|k| !ran.contains_key(*k))
                .collect::<Vec<_>>()
            {
                stale.push(format!(
                    "{check}: {key} is no sample this check ran on: take it out (`cargo check-glb --built <dir> --update-baseline`)"
                ));
                ran.insert(key.into(), 0);
            }
        }
    }
    if !stale.is_empty() {
        failed += 1;
        text += "FAIL gate-baseline.json\n";
        for e in stale {
            text += &format!("       error: {e}\n");
        }
    }
    text += &format!("{} samples, {failed} failed\n", samples.len());
    Report {
        text,
        failed,
        counts,
        samples,
        textures,
        near: looks,
    }
}

/// A sample's file as its parts.
pub(crate) fn read_parts(data: &[u8]) -> R<Vec<Part>> {
    read(&parse_glb(data)?)
}

/// `baseline` with the sections of `CHECKS` it has none of, from what a run counted (the day a check comes in):
/// a section it has stays as it is (only `--update-baseline` changes it, and only down).
pub(crate) fn seeded(
    baseline: &Baseline,
    counts: &Counts,
) -> Result<(Baseline, Vec<String>), String> {
    let mut all: serde_json::Value =
        serde_json::from_str(&baseline.text()).map_err(|e| e.to_string())?;
    let mut written = Vec::new();
    for check in CHECKS {
        if baseline.checks().any(|c| c == check) {
            continue;
        }
        let section: serde_json::Map<String, serde_json::Value> = counts
            .get(check)
            .into_iter()
            .flatten()
            .filter(|(_, n)| **n > 0)
            .map(|(key, n)| (key.clone(), serde_json::Value::from(*n)))
            .collect();
        if section.is_empty() {
            continue;
        }
        written.push(format!(
            "{check}: {} samples, {} findings",
            section.len(),
            section
                .values()
                .filter_map(serde_json::Value::as_u64)
                .sum::<u64>()
        ));
        if let Some(all) = all.as_object_mut() {
            all.insert(check.into(), serde_json::Value::Object(section));
        }
    }
    Ok((Baseline::parse(&all.to_string())?, written))
}
