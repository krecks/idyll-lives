//! The sources, options and outcomes of a measure.

use crate::checkglb::against::{Class, Finding, Numbers};
use std::path::PathBuf;

/// Where a side's files come from.
#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    /// The working tree of the checkout.
    Tree,
    /// A git revision of the checkout.
    Rev(String),
    /// A directory: a checkout's root or an assets root, whole or a build of some models (the module's note).
    Dir(String),
}

/// What to compare.
pub struct Options {
    /// The checkout: its working tree is `Source::Tree`, its git the revisions, its content and furniture.json
    /// what the gate reads on both sides.
    pub root: String,
    pub old: Source,
    pub new: Source,
    /// Only these keys (none: every model, or `family`'s).
    pub keys: Vec<String>,
    /// Only the models of this folder of `models/` or `packs/` (case and underscores aside, `pack_` or not).
    pub family: Option<String>,
    /// port-accepted.json (None: the checkout's tools/art/port-accepted.json).
    pub accepted: Option<String>,
}

/// One model, far version or manifest compared.
#[derive(Debug)]
pub struct Row {
    /// The model's key, `<key> far` for its far version, or a manifest's path under the assets.
    pub name: String,
    pub class: Class,
    pub findings: Vec<Finding>,
    /// Findings port-accepted.json accepts, each with its reason.
    pub accepted: Vec<(Finding, String)>,
    /// What is worth saying of a row that passes (a perch that is off within its limit).
    pub notes: Vec<String>,
    pub numbers: Option<Numbers>,
}

/// What `against` found.
pub struct Outcome {
    pub rows: Vec<Row>,
    /// What is wrong with port-accepted.json.
    pub errors: Vec<String>,
    /// The report: a line a row that is not `same` (every row when keys or a family were named) and the counts.
    pub text: String,
}

impl Outcome {
    pub fn count(&self, class: Class) -> usize {
        self.rows.iter().filter(|r| r.class == class).count()
    }

    pub fn failed(&self) -> bool {
        self.count(Class::Changed) > 0 || !self.errors.is_empty()
    }
}

/// A folder for this run's copies of a revision's files, gone when the run is.
pub(super) struct Scratch(pub(crate) PathBuf);

impl Scratch {
    pub(super) fn new() -> Result<Scratch, String> {
        // (One a run: two runs of one process, as its tests are, don't share one.)
        static RUNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let run = RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("ll-against-{}-{run}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        Ok(Scratch(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
