//! The markdown tables of motions and uses.

use super::*;

/// A model's `motions` as a table cell: `uses doze→bed_out; open bed_out`.
fn motions_cell(m: Option<&Value>) -> String {
    let Some(m) = m else { return "—".into() };
    let clip = |v: &Value| match v {
        Value::String(s) => s.clone(),
        Value::Array(l) => l
            .iter()
            .map(|c| {
                c.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| c["clip"].as_str().unwrap_or("?").to_string())
            })
            .collect::<Vec<_>>()
            .join("+"),
        v => v["clip"].as_str().unwrap_or("?").to_string(),
    };
    let mut parts = Vec::new();
    for part in ["uses", "loops"] {
        if let Some(map) = m[part].as_object().filter(|m| !m.is_empty()) {
            parts.push(format!(
                "{part} {}",
                map.iter()
                    .map(|(k, v)| format!("{k}→{}", clip(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    for part in checkglb::CLIP_KINDS {
        if let Some(c) = m[part].as_str() {
            parts.push(format!("{part} {c}"));
        }
    }
    if m["clock"].is_object() {
        parts.push("clock".into());
    }
    if parts.is_empty() {
        "—".into()
    } else {
        parts.join("; ")
    }
}

const USES_HEAD: &str = "| Object | Kind | Uses (id:anim) | Pose | Minutes | Effects | Skills | Slots | Approach | Motions | Sound | Flags |\n|---|---|---|---|---|---|---|---|---|---|---|---|";

/// `cargo review uses`: (stdout, the files for `--md`, the hard flags `--check` fails on as `id: flag`). `motions`
/// by model key; `reference` for the kinds (None: no reference yet, so every object without a use is flagged).
pub fn uses(
    objects: &[Object],
    motions: &BTreeMap<String, Value>,
    actions: &BTreeSet<String>,
    sounds: &BTreeMap<String, String>,
    reference: Option<&Reference>,
) -> (String, BTreeMap<String, String>, Vec<String>) {
    let flagged = flagged(objects, motions, actions, sounds, reference);
    let hard: Vec<String> = flagged
        .iter()
        .flat_map(|(o, _, flags)| {
            flags
                .iter()
                .filter(|f| f.hard())
                .map(move |f| format!("{}: {}", o.id, f.text()))
        })
        .collect();
    let mut out = String::from("# Uses: flags per category\n\n");
    if reference.is_none() {
        let _ = write!(
            out,
            "(No {REFERENCE} yet: no kind is decor, so every object without a use is flagged.)\n\n"
        );
    }
    let _ = writeln!(
        out,
        "| Category | Objects | Flagged | {} |",
        FLAG_COLUMNS.join(" | ")
    );
    let _ = writeln!(out, "|---|---|---|{}", "---|".repeat(FLAG_COLUMNS.len()));
    let mut totals = [0usize; FLAG_COLUMNS.len() + 2];
    let mut files = BTreeMap::new();
    for page in page_order(objects.iter().map(|o| o.page.clone())) {
        let here: Vec<&(&Object, Option<&str>, Vec<Flag>)> =
            flagged.iter().filter(|(o, _, _)| o.page == page).collect();
        let counts: Vec<usize> = FLAG_COLUMNS
            .iter()
            .map(|c| {
                here.iter()
                    .map(|(_, _, f)| f.iter().filter(|f| f.column() == *c).count())
                    .sum()
            })
            .collect();
        let n_flagged = here.iter().filter(|(_, _, f)| !f.is_empty()).count();
        let _ = writeln!(
            out,
            "| {page} | {} | {n_flagged} | {} |",
            here.len(),
            counts
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(" | ")
        );
        totals[0] += here.len();
        totals[1] += n_flagged;
        for (t, c) in totals[2..].iter_mut().zip(&counts) {
            *t += c;
        }
        let mut md = format!(
            "# Uses: {page}\n\n`cargo review uses --md`: per object its uses (one line each), its model's `motions` and what looks wrong.\n\n{USES_HEAD}\n"
        );
        for (o, kind, flags) in here {
            let lines = |f: &dyn Fn(&Use) -> String| {
                if o.uses.is_empty() {
                    "—".to_string()
                } else {
                    o.uses.iter().map(f).collect::<Vec<_>>().join("<br>")
                }
            };
            let list = |l: &[(String, f32)], unit: &str| {
                if l.is_empty() {
                    "—".to_string()
                } else {
                    l.iter()
                        .map(|(n, g)| format!("{n} {}{unit}", short(*g)))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            };
            let flags = if flags.is_empty() {
                "—".to_string()
            } else {
                flags
                    .iter()
                    .map(Flag::text)
                    .collect::<Vec<_>>()
                    .join("<br>")
            };
            let _ = writeln!(
                md,
                "| `{}` {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {flags} |",
                o.id,
                o.name,
                kind.unwrap_or("—"),
                lines(&|u| format!("{}:{}", u.id, if u.anim.is_empty() { "—" } else { &u.anim })),
                lines(&|u| u.pose.to_string()),
                lines(&|u| short(u.minutes)),
                lines(&|u| list(&u.effects, "")),
                lines(&|u| list(&u.skills, "/h")),
                o.slots,
                o.approach,
                motions_cell(motions.get(&o.model)),
                lines(&|u| sound_cell(sounds, o, u)),
            );
        }
        files.insert(format!("uses-{page}.md"), md);
    }
    let _ = writeln!(
        out,
        "| **all** | {} | {} | {} |",
        totals[0],
        totals[1],
        totals[2..]
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(" | ")
    );
    // Why an object with no uses is not flagged: its kind is decor by the reference's decision.
    let silent = objects
        .iter()
        .filter(|o| o.uses.is_empty() && !o.lamp)
        .count();
    let _ = writeln!(
        out,
        "\n- {} objects have no use; decor by `{REFERENCE}`'s `decor` list (the decision record): {}; the rest are flagged.",
        silent,
        silent.saturating_sub(totals[2])
    );
    (out, files, hard)
}

/// Every object with its kind (the reference's) and its flags, in the content's order: the reference says which
/// kinds are decor and which kind owns a sound.
fn flagged<'a>(
    objects: &'a [Object],
    motions: &BTreeMap<String, Value>,
    actions: &BTreeSet<String>,
    sounds: &BTreeMap<String, String>,
    reference: Option<&'a Reference>,
) -> Vec<(&'a Object, Option<&'a str>, Vec<Flag>)> {
    let medians = tag_medians(objects);
    let kind_of =
        |o: &Object| reference.and_then(|r| r.kind_of(&o.id, &o.model, &o.category, &o.group));
    let owners = sound_owners(objects, sounds, &|o| kind_of(o).map(str::to_string));
    objects
        .iter()
        .map(|o| {
            let kind = kind_of(o);
            let decor =
                kind.is_some_and(|k| reference.is_some_and(|r| r.decor.iter().any(|d| d == k)));
            let heard = Sounds {
                tag: sounds,
                owners: &owners,
                kind,
            };
            let flags = use_flags(o, motions.get(&o.model), actions, &medians, decor, &heard);
            (o, kind, flags)
        })
        .collect()
}

/// Writes `files` into `dir` (made if missing).
pub(crate) fn write_md(dir: &Path, files: &BTreeMap<String, String>) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (name, text) in files {
        std::fs::write(dir.join(name), text)
            .map_err(|e| format!("{}: {e}", dir.join(name).display()))?;
    }
    Ok(())
}
