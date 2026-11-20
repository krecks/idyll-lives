//! Comparing two models: the measurements and their tolerances.

use super::*;
use crate::checkglb::digest;
use crate::checkglb::glb::{Glb, R, parse_glb};
use crate::checkglb::near::{self, Part, Rest};
use crate::checkglb::py::J;
use std::collections::{BTreeSet, HashSet};

/// Compares two `.glb` files' bytes: `same` if their digests are equal, else every row of `METRICS` a file alone
/// answers (the pack, the centre and the gate's rows need its entry: `against`).
pub fn compare(old: &[u8], new: &[u8]) -> Compared {
    compare_by(old, new, &measure)
}

/// `compare` for a far version held to `decimated` (the `DECIMATED_` limits); `model` is the new build's full
/// model, whose lowest point the far version's is held to (None: not held to it).
pub fn compare_decimated(old: &[u8], new: &[u8], model: Option<&[u8]>) -> Compared {
    let floor = model
        .and_then(|data| parse_glb(data).ok())
        .and_then(|g| whole(&g).ok())
        .and_then(|part| part.bounds())
        .map(|(lo, _)| lo[1]);
    compare_by(old, new, &|o, n, out, numbers| {
        measure_decimated(o, n, floor, out, numbers)
    })
}

/// What two files that are not the same are measured by: their findings and numbers.
pub(super) type Measure<'a> = &'a dyn Fn(&Glb, &Glb, &mut Vec<Finding>, &mut Numbers) -> R<()>;

pub(super) fn compare_by(old: &[u8], new: &[u8], measure: Measure) -> Compared {
    let mut out = Compared::default();
    if old == new {
        out.same = true;
        return out;
    }
    let read = |data: &[u8], which: &str| {
        parse_glb(data).map_err(|e| found("file", format!("the {which} file is no .glb: {e}")))
    };
    let (o, n) = match (read(old, "old"), read(new, "new")) {
        (Ok(o), Ok(n)) => (o, n),
        (o, n) => {
            out.findings.extend(o.err().into_iter().chain(n.err()));
            return out;
        }
    };
    let measured = (|| -> R<()> {
        if digest::digest(&o)? == digest::digest(&n)? {
            out.same = true;
            return Ok(());
        }
        out.numbers.differs = digest::parts(&o)?.differing(&digest::parts(&n)?);
        measure(&o, &n, &mut out.findings, &mut out.numbers)
    })();
    if let Err(e) = measured {
        out.findings
            .push(found("file", format!("cannot be compared: {e}")));
    }
    out
}

/// A list of names for a finding: the first six, then how many more.
pub(super) fn names(list: &[String]) -> String {
    let mut out = list.iter().take(6).cloned().collect::<Vec<_>>().join(", ");
    if list.len() > 6 {
        out += &format!(" and {} more", list.len() - 6);
    }
    out
}

/// What is only in one of two sets, as findings.
pub(super) fn only<'a>(
    metric: &'static str,
    what: &str,
    old: impl Iterator<Item = &'a String>,
    new: impl Iterator<Item = &'a String>,
    out: &mut Vec<Finding>,
) {
    let (old, new): (BTreeSet<&String>, BTreeSet<&String>) = (old.collect(), new.collect());
    for (side, list) in [("old", old.difference(&new)), ("new", new.difference(&old))] {
        let list: Vec<String> = list.map(|s| s.to_string()).collect();
        if !list.is_empty() {
            out.push(found(
                metric,
                format!("{what} only in the {side} file: {}", names(&list)),
            ));
        }
    }
}

/// A file read for the rows of `METRICS`: its nodes, their rests, its clips and its parts.
pub(crate) struct Model {
    pub(super) tree: digest::Tree,
    pub(super) rests: Vec<Rest>,
    pub(super) clips: Vec<near::Clip>,
    pub(super) parts: Vec<Part>,
    pub(super) meshes: HashSet<String>,
}

pub(super) fn read(g: &Glb) -> R<Model> {
    let tree = digest::tree(&g.gltf)?;
    let nodes = g.gltf.get("nodes").and_then(J::list).unwrap_or(&[]);
    Ok(Model {
        rests: near::rests(&g.gltf)?,
        clips: near::clips(g, &tree)?,
        parts: near::parts(g, &tree)?,
        meshes: nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.has("mesh"))
            .map(|(i, _)| tree.paths[i].clone())
            .collect(),
        tree,
    })
}

impl Model {
    pub(crate) fn rest(&self, path: &str) -> Option<&Rest> {
        self.tree
            .paths
            .iter()
            .position(|p| p == path)
            .map(|i| &self.rests[i])
    }
}

pub(super) fn measure(o: &Glb, n: &Glb, out: &mut Vec<Finding>, numbers: &mut Numbers) -> R<()> {
    let (old, new) = (read(o)?, read(n)?);
    nodes_differ(&old, &new, out);
    materials_differ(o, n, out);
    clips_differ(&old, &new, out);
    parts_differ(&old, &new, out, numbers);
    Ok(())
}
