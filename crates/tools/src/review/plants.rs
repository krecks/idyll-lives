//! Measuring the plants of a build.

use super::*;

/// Measures each row whose kind has a `species` from its model (`species::read`) and adds what is off
/// (`species_row`).
pub(crate) fn measure_plants(
    s: &Sources,
    rows: &mut [SizeRow],
    reference: &Reference,
) -> Result<(), String> {
    let plant = |r: &SizeRow| {
        r.kind
            .as_deref()
            .and_then(|k| reference.kind(k))
            .is_some_and(|k| k.species.is_some())
            && r.problem.is_none()
    };
    if !rows.iter().any(plant) {
        return Ok(());
    }
    let files = species::model_files(&s.paths)?;
    let assets = Path::new(&s.paths.assets);
    let base: Value = serde_json::from_str(&s.read("web/public/assets/manifest.json")?)
        .map_err(|e| format!("manifest.json: {e}"))?;
    let finishes = crate::colours::finishes(&base, assets);
    let atlas = Atlas::read(&s.root.to_string_lossy())?;
    for r in rows.iter_mut().filter(|r| plant(r)) {
        let shape = match files.get(&r.model) {
            Some((path, entry)) => species::read(path, entry.get("materials"), &finishes)
                .map(|p| species::measure(&p, &atlas)),
            None => Err(format!("no manifest has a model file for {}", r.model)),
        };
        species_row(r, reference, shape, &atlas);
    }
    Ok(())
}

/// The checkout's files the review reads.
pub(crate) struct Sources {
    pub(crate) root: PathBuf,
    pub(crate) paths: Paths,
}

impl Sources {
    pub(crate) fn repo() -> Sources {
        let root = PathBuf::from(checkglb::repo_root());
        Sources {
            paths: Paths::repo(),
            root,
        }
    }

    pub(crate) fn read(&self, rel: &str) -> Result<String, String> {
        std::fs::read_to_string(self.root.join(rel)).map_err(|e| format!("{rel}: {e}"))
    }
}
