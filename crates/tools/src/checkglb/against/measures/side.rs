//! The two sides of a comparison, and where their files come from.

use super::*;

/// How a side's model files get where its manifests name them.
pub(crate) enum Origin {
    /// They are there.
    Here,
    /// Read from a revision into the scratch folder `at`, laid out as the checkout.
    Rev { rev: String, at: String },
    /// A build of some models (its assets root `built`) over the working tree's (`tree`), copied into the scratch.
    Over { built: String, tree: String },
}

/// One side of the comparison: where it is read from, its manifests stacked and its centres.
pub(crate) struct Side {
    pub(crate) paths: Paths,
    pub(crate) origin: Origin,
    pub(crate) root: String,
    pub(crate) stack: Vec<(String, J)>,
    pub(crate) entries: BTreeMap<String, (J, String)>,
    pub(crate) fin: Finishes,
    pub(crate) centres: J,
}

pub(crate) const ASSETS: &str = "web/public/assets";
pub(crate) const CENTRES: &str = "tools/art/centres.json";
/// Where a revision from before the models' tools moved has its centres (the tags ref/blender and
/// ref/blender-civic).
pub(crate) const CENTRES_BEFORE: &str = "tools/art/blender/centres.json";

pub(crate) fn copy(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(dir) = to.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::copy(from, to)
        .map(|_| ())
        .map_err(|e| format!("{} to {}: {e}", from.display(), to.display()))
}

/// The manifests of the packs under an assets root: `packs/<name>/manifest.json`, from the root.
pub(crate) fn pack_manifests(assets: &str) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(Path::new(assets).join("packs"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .map(|name| format!("packs/{name}/manifest.json"))
        .filter(|rel| Path::new(assets).join(rel).is_file())
        .collect();
    out.sort();
    out
}
