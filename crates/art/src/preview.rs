//! `cargo art <family> --preview [--sheet]` for a family built here: each model built in memory, nothing written to
//! the assets, and drawn from its file as the game will read it (`artkit::preview`) into
//! tools/art/blender/previews/<family>/ (ignored by git): <id>.png (the front three-quarter), <id>@<clip>.png
//! (each clip where it is most open), <id>@perches.png and <id>@perches.top.png (a red ball on each perch),
//! <id>.far.png, and with --sheet <id>.sheet.png (four sides, the baked shading, each clip along its length).

use crate::{build_all, finishes, to_j};
use artkit::family::Family;
use ll_tools::checkglb::Paths;
use ll_tools::checkglb::preview::{Subject, draw_all};
use std::path::Path;

/// What `--lit` gets for a family built here: the game is the lit view.
pub const LIT: &str = concat!(
    "[gamekit] --lit: no lit pictures of a family built here; ",
    "the game's gallery is the lit view (bun tools/bench/check.mjs gallery)"
);

/// Builds `family`'s models (`only` these ids, if given) and draws them into `into` (the previews folder, its
/// family's folder under it); what failed, with why.
pub fn run(
    repo: &Paths,
    family: &Family,
    only: Option<&[String]>,
    sheet: bool,
    into: &Path,
) -> Result<(), String> {
    let finishes = finishes(repo)?;
    if let Some(only) = only
        && let Some(odd) = only
            .iter()
            .find(|id| !family.models.iter().any(|m| m.id == *id))
    {
        return Err(format!("{}: no model {odd}", family.name));
    }
    let ids: Vec<usize> = (0..family.models.len())
        .filter(|&i| only.is_none_or(|o| o.iter().any(|id| id == family.models[i].id)))
        .collect();
    let mut subjects = Vec::new();
    let mut failed = Vec::new();
    for b in build_all(family, &ids, &finishes) {
        b.log.iter().for_each(|l| println!("{l}"));
        match b.output {
            Ok(o) => subjects.push(Subject {
                key: b.key,
                // (by family: an id may be in two, and one family's pictures would overwrite the other's)
                stem: into.join(family.name).join(b.id),
                glb: o.glb,
                far: o.far,
                entry: to_j(&o.entry),
            }),
            Err(why) => {
                println!("[gamekit] {}: {why}", b.id);
                failed.push(b.id.to_string());
            }
        }
    }
    failed.extend(draw_all(
        repo,
        &subjects,
        sheet,
        into.parent().unwrap_or(into),
    ));
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("{}: {}", family.name, failed.join("; ")))
    }
}
