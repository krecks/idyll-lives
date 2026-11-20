//! The context's own measures.

use super::*;
use crate::checkglb::against::{
    BOUNDS, Class, Compared, Finding, Numbers, PERCH, compare, compare_decimated, found,
    json_differ, mm, names, without,
};
use crate::checkglb::glb::R;
use crate::checkglb::py::{self, J, relpath};

impl Context {
    /// What the gate says of `key` on a side; a file it cannot read is its one error.
    fn gate(&self, side: &Side, key: &str) -> crate::checkglb::Checked {
        let unread = |err: String| crate::checkglb::Checked {
            errors: vec![format!("cannot be read: {err}")],
            warnings: Vec::new(),
            accepted: Vec::new(),
            summary: String::new(),
            counts: Default::default(),
        };
        let (Some((entry, _)), Some(path)) = (side.entries.get(key), side.file(key, "url")) else {
            return unread("its entry has no file".into());
        };
        // The gate's own rules only: the counted checks answer to the baseline of one tree, not to a build.
        let baseline = crate::checkglb::baseline::Baseline::default();
        let counting = crate::checkglb::Counting {
            baseline: &baseline,
            file: &[],
            batch: std::collections::BTreeMap::new(),
        };
        crate::checkglb::check(
            key,
            &side.fin,
            entry,
            &path,
            side.file(key, "far").as_deref(),
            self.facts.get(key),
            side.centres.get(key),
            Some(&self.furniture),
            &counting,
        )
        .unwrap_or_else(unread)
    }

    /// The rows that need the model's entry and the content: what the gate says of it that it did not say before,
    /// its draw calls, its size and its functional heights.
    fn in_the_game(&self, key: &str, out: &mut Vec<Finding>) {
        let (old, new) = (self.gate(&self.old, key), self.gate(&self.new, key));
        for (kind, was, is) in [
            ("error", &old.errors, &new.errors),
            ("warning", &old.warnings, &new.warnings),
        ] {
            for text in newly(was, is) {
                // (The budget's target is the triangles' row: "never newly over it".)
                let budget =
                    text.contains(" triangles, aim for ") || text.contains(" triangles, at most ");
                out.push(found(
                    if budget { "triangles" } else { "gate" },
                    format!("the gate's new {kind}: {text}"),
                ));
            }
        }
        if let (Some(was), Some(is)) = (draws(&old.summary), draws(&new.summary))
            && was != is
        {
            out.push(found("draws", format!("{is} draw calls, was {was}")));
        }
        let measured = (|| -> R<()> {
            let side = |s: &Side| -> R<Option<crate::checkglb::Model>> {
                match (s.entries.get(key), s.file(key, "url")) {
                    (Some((entry, _)), Some(path)) => {
                        crate::checkglb::model(entry, &path).map(Some)
                    }
                    _ => Ok(None),
                }
            };
            let (Some(a), Some(b)) = (side(&self.old)?, side(&self.new)?) else {
                return Ok(());
            };
            let off = (0..3)
                .map(|k| (a.size[k] - b.size[k]).abs())
                .fold(0.0, f64::max);
            if off > BOUNDS {
                let text = |s: [f64; 3]| format!("{:.4} x {:.4} x {:.4} m", s[0], s[1], s[2]);
                out.push(found(
                    "size",
                    format!(
                        "{}, was {} ({} off, at most {})",
                        text(b.size),
                        text(a.size),
                        mm(off),
                        mm(BOUNDS)
                    ),
                ));
            }
            for f in self.facts.get(key).map(Vec::as_slice).unwrap_or(&[]) {
                let ha =
                    crate::checkglb::heights(&a.file, &a.entry, &f.object, &self.furniture, true)?;
                let hb =
                    crate::checkglb::heights(&b.file, &b.entry, &f.object, &self.furniture, true)?;
                let id = py::str_of(&f.id);
                if ha.len() != hb.len() {
                    out.push(found(
                        "heights",
                        format!(
                            "object {id}: {} heights measured, {} before",
                            hb.len(),
                            ha.len()
                        ),
                    ));
                    continue;
                }
                for (x, y) in ha.iter().zip(&hb) {
                    let values = |h: &crate::checkglb::Height| -> Vec<Option<f64>> {
                        vec![h.mesh, h.span.map(|s| s.0), h.span.map(|s| s.1)]
                    };
                    for (p, q) in values(x).into_iter().zip(values(y)) {
                        let off = match (p, q) {
                            (Some(p), Some(q)) => (p - q).abs(),
                            (None, None) => 0.0,
                            _ => f64::INFINITY,
                        };
                        if off > BOUNDS {
                            let text = |v: Option<f64>| {
                                v.map_or("nothing".to_string(), |v| format!("{v:.4} m"))
                            };
                            out.push(found(
                                "heights",
                                format!(
                                    "object {id}: its {} at ({}, {}) is at {}, was {} (at most {} off)",
                                    x.what,
                                    x.at[0],
                                    x.at[1],
                                    text(q),
                                    text(p),
                                    mm(BOUNDS)
                                ),
                            ));
                            break;
                        }
                    }
                }
            }
            Ok(())
        })();
        if let Err(e) = measured {
            out.push(found("file", format!("cannot be measured: {e}")));
        }
    }

    /// The pack entry and the centre of `key`: whether they are the same, with what differs as findings and what
    /// is off within its limit as notes.
    fn entry_differs(&self, key: &str, out: &mut Vec<Finding>, notes: &mut Vec<String>) -> bool {
        let (Some((a, _)), Some((b, _))) = (self.old.entries.get(key), self.new.entries.get(key))
        else {
            return true;
        };
        let mut differs = Vec::new();
        json_differ(
            Some(&without(a, &["perches"])),
            Some(&without(b, &["perches"])),
            "",
            &mut differs,
        );
        let mut same = differs.is_empty();
        if !same {
            out.push(found("pack", format!("its entry: {}", names(&differs))));
        }
        match (a.get("perches"), b.get("perches")) {
            (None, None) => {}
            (Some(p), Some(q)) => {
                let points = |v: &J| -> Vec<Vec<f64>> {
                    v.list()
                        .unwrap_or(&[])
                        .iter()
                        .map(|p| {
                            p.list()
                                .unwrap_or(&[])
                                .iter()
                                .filter_map(|x| x.num().map(|n| n.f()))
                                .collect()
                        })
                        .collect()
                };
                let (p, q) = (points(p), points(q));
                if p.len() != q.len() {
                    same = false;
                    out.push(found(
                        "perches",
                        format!("{} perches, was {}", q.len(), p.len()),
                    ));
                } else {
                    let worst = p
                        .iter()
                        .zip(&q)
                        .enumerate()
                        .map(|(i, (x, y))| {
                            (
                                if x.len() == y.len() {
                                    py::dist(x, y)
                                } else {
                                    f64::INFINITY
                                },
                                i,
                            )
                        })
                        .max_by(|x, y| x.0.total_cmp(&y.0));
                    if let Some((off, i)) = worst.filter(|(off, _)| *off > 0.0) {
                        same = false;
                        if off > PERCH {
                            out.push(found(
                                "perches",
                                format!(
                                    "perch {i} is {} off, at most {}: {:?}, was {:?}",
                                    mm(off),
                                    mm(PERCH),
                                    q[i],
                                    p[i]
                                ),
                            ));
                        } else {
                            notes.push(format!(
                                "perches up to {} off (perch {i}): the birds' recorded day reads them as written",
                                mm(off)
                            ));
                        }
                    }
                }
            }
            (p, _) => {
                same = false;
                let side = if p.is_some() { "old" } else { "new" };
                out.push(found(
                    "perches",
                    format!("only the {side} entry has perches"),
                ));
            }
        }
        let mut differs = Vec::new();
        json_differ(
            self.old.centres.get(key),
            self.new.centres.get(key),
            "its centre",
            &mut differs,
        );
        if !differs.is_empty() {
            same = false;
            out.push(found("centre", names(&differs)));
        }
        !same
    }

    /// Two files of a row compared, by path: one side without the file is a finding.
    /// With `decimated`, as a far version held to that rule, its lowest point to the model's (the file `model`).
    fn files(
        &self,
        old: Option<&String>,
        new: Option<&String>,
        decimated: Option<Option<&String>>,
    ) -> Compared {
        let read = |path: Option<&String>, which: &str| -> Result<Vec<u8>, Finding> {
            let path =
                path.ok_or_else(|| found("file", format!("the {which} entry names no file")))?;
            std::fs::read(path).map_err(|e| {
                let name = relpath(
                    path,
                    &if which == "old" { &self.old } else { &self.new }
                        .paths
                        .assets,
                );
                found("file", format!("the {which} side has no {name}: {e}"))
            })
        };
        match (read(old, "old"), read(new, "new")) {
            (Ok(a), Ok(b)) => match decimated {
                Some(model) => {
                    let model = model.and_then(|path| std::fs::read(path).ok());
                    compare_decimated(&a, &b, model.as_deref())
                }
                None => compare(&a, &b),
            },
            (a, b) => Compared {
                same: false,
                findings: a.err().into_iter().chain(b.err()).collect(),
                numbers: Numbers::default(),
            },
        }
    }

    /// The rows of one key: the model, and its far version if either side has one.
    pub(crate) fn rows(&self, key: &str) -> Vec<Row> {
        let row = |name: String| Row {
            name,
            class: Class::Same,
            findings: Vec::new(),
            accepted: Vec::new(),
            notes: Vec::new(),
            numbers: None,
        };
        let mut main = row(key.to_string());
        let sides = (
            self.old.entries.contains_key(key),
            self.new.entries.contains_key(key),
        );
        if sides != (true, true) {
            main.findings.push(found(
                "pack",
                match sides {
                    (true, false) => "only the old manifests have it".into(),
                    (false, true) => "only the new manifests have it".into(),
                    _ => "no manifest of either side has it".into(),
                },
            ));
            return vec![self.classed(main, false)];
        }
        let entry_differs = self.entry_differs(key, &mut main.findings, &mut main.notes);
        let file = |field: &str| (self.old.file(key, field), self.new.file(key, field));
        let (old, new) = file("url");
        let body = if old.is_none() && new.is_none() {
            // (An entry drawn from elsewhere: nothing here to compare.)
            Compared {
                same: true,
                ..Compared::default()
            }
        } else {
            self.files(old.as_ref(), new.as_ref(), None)
        };
        let model = new;
        let far_name = format!("{key} far");
        let (old, new) = file("far");
        let far = (old.is_some() || new.is_some()).then(|| {
            let found = self.files(old.as_ref(), new.as_ref(), None);
            // A far version listed as decimated is held to its own rule, if the rows it is not held to fail.
            let decimated = self
                .accepted
                .get(&far_name)
                .is_some_and(|a| a.contains_key("decimated"));
            if decimated && !found.same && !found.findings.is_empty() {
                (
                    self.files(old.as_ref(), new.as_ref(), Some(model.as_ref())),
                    true,
                )
            } else {
                (found, false)
            }
        });
        if entry_differs || !body.same || far.as_ref().is_some_and(|(f, _)| !f.same) {
            self.in_the_game(key, &mut main.findings);
        }
        let same = !entry_differs && body.same;
        main.findings.extend(body.findings);
        main.numbers = (!body.same).then_some(body.numbers);
        let mut out = vec![self.classed(main, same)];
        if let Some((found, decimated)) = far {
            let mut far = row(far_name);
            far.numbers = (!found.same).then_some(found.numbers);
            far.findings = found.findings;
            if decimated {
                let why = self.accepted[&far.name]["decimated"].clone();
                far.accepted.push((
                    Finding {
                        metric: "decimated",
                        text: "held to the decimated far versions' rule".into(),
                    },
                    why,
                ));
            }
            out.push(self.classed(far, found.same));
        }
        out
    }

    /// A row with what port-accepted.json accepts of it taken out of its findings, and its class.
    pub(crate) fn classed(&self, mut row: Row, same: bool) -> Row {
        if let Some(accepted) = self.accepted.get(&row.name) {
            let (taken, left): (Vec<Finding>, Vec<Finding>) = std::mem::take(&mut row.findings)
                .into_iter()
                .partition(|f| accepted.contains_key(f.metric));
            row.findings = left;
            row.accepted.extend(
                taken
                    .into_iter()
                    .map(|f| (f.clone(), accepted[f.metric].clone())),
            );
        }
        row.class = if !row.findings.is_empty() {
            Class::Changed
        } else if same && row.accepted.is_empty() {
            Class::Same
        } else {
            Class::Near
        };
        row
    }
}
