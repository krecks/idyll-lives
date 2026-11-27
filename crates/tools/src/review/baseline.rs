//! The gate baseline: counting, judging, lowering and seeding it, for `sizes` (its `reference` section) and
//! `uses` (its `USES_CHECKS` sections), each by object id.

use super::*;

/// The baseline's section (tools/art/gate-baseline.json) `cargo review sizes --check` judges by: how many
/// findings each measured row has (its `outside`), by its id (an object's, or a model key no object draws).
pub const CHECK: &str = "reference";

/// One id's findings of a section: how many (None: not measured, kept as the baseline has it), what they are, and
/// what else is wrong with it (an error in its place: a model that couldn't be measured).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tally {
    pub id: String,
    pub count: Option<u64>,
    pub text: String,
    pub error: Option<String>,
}

/// A section of the baseline as a run found it: its name, what a finding is (for the messages) and each id's tally.
pub struct Section<'a> {
    pub check: &'a str,
    pub what: &'a str,
    pub tallies: Vec<Tally>,
}

fn size_section(rows: &[SizeRow]) -> Section<'static> {
    Section {
        check: CHECK,
        what: "outside its kind's range",
        tallies: rows
            .iter()
            .map(|r| Tally {
                id: r.id.clone(),
                count: r.problem.is_none().then_some(r.outside.len() as u64),
                text: r.outside.join("; "),
                error: r
                    .problem
                    .as_deref()
                    .filter(|p| p.starts_with("unmeasured") && *p != "unmeasured: no model")
                    .map(|p| format!("`{}`: {p}", r.id)),
            })
            .collect(),
    }
}

/// Each measured row's findings, by its id (a row that couldn't be measured is left out).
pub fn counts(rows: &[SizeRow]) -> BTreeMap<String, u64> {
    tally_counts(&size_section(rows))
}

fn tally_counts(s: &Section) -> BTreeMap<String, u64> {
    s.tallies
        .iter()
        .filter_map(|t| Some((t.id.clone(), t.count?)))
        .collect()
}

/// What `--check` fails on, against `baseline`'s `reference` section: a row with more findings than it holds, or
/// any where it holds none (new objects are strict); one with fewer (stale: `--update-baseline` lowers it); an
/// id it holds that no row has; and a row whose model couldn't be measured (one with no model is not judged).
pub fn judged(rows: &[SizeRow], baseline: &Baseline) -> Vec<String> {
    judge(&[size_section(rows)], baseline, "sizes")
}

/// `judged` for every section of `cargo review <cmd>`.
pub fn judge(sections: &[Section], baseline: &Baseline, cmd: &str) -> Vec<String> {
    let mut out = Vec::new();
    for s in sections {
        let (check, what) = (s.check, s.what);
        for t in &s.tallies {
            if let Some(e) = &t.error {
                out.push(e.clone());
            }
            let Some(count) = t.count else { continue };
            let (id, text) = (&t.id, &t.text);
            match baseline.judge(check, id, count) {
                Verdict::Clean | Verdict::Held(_) => {}
                Verdict::New(n) => out.push(format!(
                    "`{id}`: {n} {what}, none in the baseline: {text}"
                )),
                Verdict::Over { base, count } => out.push(format!(
                    "`{id}`: {count} {what}, {base} in the baseline: fix the new ones: {text}"
                )),
                Verdict::Stale { base, count } => out.push(format!(
                    "`{id}`: {count} {what}, {base} in the baseline: lower it (`cargo review {cmd} --update-baseline`)"
                )),
            }
        }
        let ids: HashSet<&str> = s.tallies.iter().map(|t| t.id.as_str()).collect();
        for key in baseline.keys(check).filter(|k| !ids.contains(k)) {
            out.push(format!(
                "`{key}`: in the baseline, but no object or model the review measured: take it out (`cargo review {cmd} --update-baseline`)"
            ));
        }
    }
    out
}

/// `baseline` lowered to what `rows` found (`Baseline::lowered`): never raised, nothing added; an id no row
/// has goes, one whose row couldn't be measured stays. (The baseline, a line for each change.)
pub fn lowered(rows: &[SizeRow], baseline: &Baseline) -> (Baseline, Vec<String>) {
    lower(&[size_section(rows)], baseline)
}

/// `lowered` for every section.
pub fn lower(sections: &[Section], baseline: &Baseline) -> (Baseline, Vec<String>) {
    let mut all = Counts::new();
    for s in sections {
        let mut found = tally_counts(s);
        let ids: HashSet<&str> = s.tallies.iter().map(|t| t.id.as_str()).collect();
        for key in baseline.keys(s.check).filter(|k| !ids.contains(k)) {
            found.insert(key.to_string(), 0);
        }
        all.insert(s.check.to_string(), found);
    }
    baseline.lowered(&all)
}

/// `baseline` with a `reference` section of what `rows` found, the day the check comes in: refused where it has
/// one (only `--update-baseline` changes it then, and only down).
pub fn seeded(rows: &[SizeRow], baseline: &Baseline) -> Result<Baseline, String> {
    seed(&[size_section(rows)], baseline)
}

/// `seeded` for every section: refused where it has any of them; a section that finds nothing is not written.
pub fn seed(sections: &[Section], baseline: &Baseline) -> Result<Baseline, String> {
    if let Some(s) = sections
        .iter()
        .find(|s| baseline.checks().any(|c| c == s.check))
    {
        return Err(format!(
            "it has a `{}` section already: `--update-baseline` lowers it",
            s.check
        ));
    }
    let mut all: Value = serde_json::from_str(&baseline.text()).map_err(|e| e.to_string())?;
    for s in sections {
        let section: serde_json::Map<String, Value> = tally_counts(s)
            .into_iter()
            .filter(|(_, n)| *n > 0)
            .map(|(id, n)| (id, Value::from(n)))
            .collect();
        if let Some(all) = all.as_object_mut().filter(|_| !section.is_empty()) {
            all.insert(s.check.into(), Value::Object(section));
        }
    }
    Baseline::parse(&all.to_string())
}

/// `sizes`'s baseline flags over `rows` (`ratchet`).
pub(super) fn baselined(paths: &Paths, rows: &[SizeRow], args: &[String]) -> Result<i32, String> {
    ratchet(paths, "sizes", &[size_section(rows)], args)
}

/// `cargo review <cmd>`'s baseline flags over `sections`: `--seed-baseline` writes them (`seed`),
/// `--update-baseline` lowers them (`lower`), then `--check` judges by them (`judge`). The exit code: 1 if
/// `--check` found something.
pub(super) fn ratchet(
    paths: &Paths,
    cmd: &str,
    sections: &[Section],
    args: &[String],
) -> Result<i32, String> {
    use crate::checkglb::baseline::FILE;
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let mut baseline = Baseline::read(&paths.baseline)?;
    let held = |b: &Baseline| -> usize { sections.iter().map(|s| b.keys(s.check).count()).sum() };
    if has("--seed-baseline") {
        baseline = seed(sections, &baseline).map_err(|e| format!("{FILE}: {e}"))?;
        baseline.write(&paths.baseline)?;
        eprintln!("{FILE}: `{cmd}`'s sections hold {} rows", held(&baseline));
    }
    if has("--update-baseline") {
        let (low, changes) = lower(sections, &baseline);
        if changes.is_empty() {
            eprintln!("{FILE}: nothing to lower");
        } else {
            low.write(&paths.baseline)?;
            for c in &changes {
                eprintln!("{FILE}: {c}");
            }
        }
        baseline = low;
    }
    if !has("--check") {
        return Ok(0);
    }
    let errors = judge(sections, &baseline, cmd);
    let held = held(&baseline);
    if errors.is_empty() {
        eprintln!("review {cmd}: every row clean or held by {FILE} ({held} held)");
        return Ok(0);
    }
    eprintln!(
        "review {cmd}: {} rows against {FILE} ({held} held):",
        errors.len()
    );
    for e in &errors {
        eprintln!("  {e}");
    }
    Ok(1)
}
