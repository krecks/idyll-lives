//! A side's files, read.

use super::*;
use crate::checkglb::Paths;
use crate::checkglb::py::{self, J, dirname, join, normpath, relpath};
use std::path::Path;

impl Side {
    pub(crate) fn open(
        what: &str,
        source: &Source,
        root: &str,
        scratch: &Scratch,
    ) -> Result<Side, String> {
        let tree = Paths::at(root);
        let at = scratch.0.join(what).to_string_lossy().into_owned();
        let under = |base: &str, assets: String, centres: String| Paths {
            art: normpath(&join(base, "tools/art")),
            assets,
            content: tree.content.clone(),
            furniture: tree.furniture.clone(),
            centres,
            baseline: tree.baseline.clone(),
        };
        let laid_out = |base: &str| {
            let before = Path::new(base).join(CENTRES_BEFORE);
            let centres = if !Path::new(base).join(CENTRES).exists() && before.exists() {
                CENTRES_BEFORE
            } else {
                CENTRES
            };
            under(
                base,
                normpath(&join(base, ASSETS)),
                normpath(&join(base, centres)),
            )
        };
        let (paths, origin) = match source {
            Source::Tree => (Paths::at(root), Origin::Here),
            Source::Rev(rev) => {
                let listed = git(
                    root,
                    &[
                        "ls-tree",
                        "-r",
                        "--name-only",
                        "-z",
                        rev,
                        "--",
                        &format!("{ASSETS}/manifest.json"),
                        &format!("{ASSETS}/packs"),
                        CENTRES,
                        CENTRES_BEFORE,
                    ],
                )?;
                let files: Vec<String> = String::from_utf8_lossy(&listed)
                    .split('\0')
                    .filter(|p| p.ends_with(".json"))
                    .map(str::to_string)
                    .collect();
                checkout(root, rev, &files, &at)?;
                (
                    laid_out(&at),
                    Origin::Rev {
                        rev: rev.clone(),
                        at: at.clone(),
                    },
                )
            }
            Source::Dir(dir) => {
                if !Path::new(dir).is_dir() {
                    return Err(format!("{dir} is no directory"));
                }
                let dir = normpath(dir);
                let found = if Path::new(&dir).join(ASSETS).is_dir() {
                    laid_out(&dir)
                } else {
                    under(&dir, dir.clone(), normpath(&join(&dir, "centres.json")))
                };
                if Path::new(&found.assets).join("manifest.json").is_file() {
                    (found, Origin::Here)
                } else {
                    // A build of some models: the working tree's manifests with the build's packs over them, and
                    // its centres over the working tree's.
                    let view = laid_out(&at);
                    for (from, rels) in [
                        (
                            &tree.assets,
                            [
                                vec!["manifest.json".to_string()],
                                pack_manifests(&tree.assets),
                            ]
                            .concat(),
                        ),
                        (&found.assets, pack_manifests(&found.assets)),
                    ] {
                        for rel in rels {
                            copy(
                                &Path::new(from).join(&rel),
                                &Path::new(&view.assets).join(&rel),
                            )?;
                        }
                    }
                    let mut centres = crate::checkglb::read_centres(&tree.centres)?;
                    for (k, c) in crate::checkglb::read_centres(&found.centres)?
                        .entries()
                        .unwrap_or(&[])
                    {
                        centres.set(k, c.clone());
                    }
                    let to = Path::new(&view.centres);
                    if let Some(dir) = to.parent() {
                        std::fs::create_dir_all(dir)
                            .map_err(|e| format!("{}: {e}", dir.display()))?;
                    }
                    std::fs::write(to, py::dumps_indent(&centres))
                        .map_err(|e| format!("{}: {e}", to.display()))?;
                    (
                        view,
                        Origin::Over {
                            built: found.assets,
                            tree: tree.assets.clone(),
                        },
                    )
                }
            }
        };
        let stack = crate::checkglb::asset_manifests(&paths)?;
        let (entries, fin) = crate::checkglb::model_entries(&stack);
        let centres = crate::checkglb::read_centres(&paths.centres)?;
        Ok(Side {
            paths,
            origin,
            root: root.to_string(),
            stack,
            entries,
            fin,
            centres,
        })
    }

    /// The file an entry's `url` or `far` names, if it is one of this side's files.
    pub(crate) fn file(&self, key: &str, field: &str) -> Option<String> {
        let (entry, manifest) = self.entries.get(key)?;
        let url = entry
            .get(field)
            .and_then(J::str)
            .filter(|u| !u.is_empty() && !u.contains("://"))?;
        Some(normpath(&join(&dirname(manifest), url)))
    }

    /// Whether a build of some models (`Origin::Over`) has the file an entry of this side names.
    pub(crate) fn built(&self, file: &str) -> bool {
        match &self.origin {
            Origin::Over { built, .. } => Path::new(built)
                .join(relpath(file, &self.paths.assets))
                .is_file(),
            _ => true,
        }
    }

    /// Puts the model files `files` where this side's manifests name them.
    pub(crate) fn fetch(&self, files: &[String]) -> Result<(), String> {
        match &self.origin {
            Origin::Here => Ok(()),
            Origin::Rev { rev, at } => {
                let paths: Vec<String> = files.iter().map(|f| relpath(f, at)).collect();
                checkout(&self.root, rev, &paths, at)
            }
            Origin::Over { built, tree } => {
                for file in files {
                    let rel = relpath(file, &self.paths.assets);
                    let from = [built, tree]
                        .into_iter()
                        .map(|base| Path::new(base).join(&rel))
                        .find(|p| p.is_file());
                    if let Some(from) = from {
                        copy(&from, Path::new(file))?;
                    }
                }
                Ok(())
            }
        }
    }

    /// A manifest's path from the assets root.
    pub(crate) fn manifest_name(&self, path: &str) -> String {
        relpath(path, &self.paths.assets)
    }
}

/// The folders an entry's model is of: its file's under `models/` and its manifest's under `packs/`.
pub(crate) fn families(entry: &J, manifest: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(url) = entry.get("url").and_then(J::str) {
        let dir = dirname(url);
        if let Some(name) = dir
            .rsplit('/')
            .next()
            .filter(|n| !n.is_empty() && *n != "models")
        {
            out.push(name.to_string());
        }
    }
    let dir = dirname(manifest);
    if dirname(&dir).ends_with("packs") {
        out.push(dir.rsplit('/').next().unwrap_or("").to_string());
    }
    out
}

/// A family's name as `--family` compares it: case and underscores aside, with or without `pack` before it.
pub(crate) fn family_name(name: &str) -> String {
    let plain: String = name
        .chars()
        .filter(|c| *c != '_')
        .collect::<String>()
        .to_lowercase();
    plain
        .strip_prefix("pack")
        .filter(|p| !p.is_empty())
        .unwrap_or(&plain)
        .to_string()
}
