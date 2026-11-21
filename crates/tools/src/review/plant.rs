//! Plants: their row and the species columns.

use super::*;

/// A plant's `row` (its kind's `species` in `reference`) with its model measured (`species::measure`): what is
/// outside the block, or the kind's `h` and `w` strictly, joins its findings; a model that can't be read is its
/// problem. A row of no such kind is left as it is.
pub fn species_row(
    row: &mut SizeRow,
    reference: &Reference,
    shape: Result<Shape, String>,
    atlas: &Atlas,
) {
    let Some(k) = row.kind.as_deref().and_then(|k| reference.kind(k)) else {
        return;
    };
    let Some(sp) = &k.species else { return };
    match shape {
        Ok(shape) => {
            row.outside
                .extend(species::findings(sp, k.h, k.w, &shape, atlas));
            row.species = Some(shape);
        }
        Err(e) => row.problem = Some(format!("unmeasured: {e}")),
    }
}

/// `cargo review sizes` over `objects` (`size_rows`, no plant measured): (stdout, the files for `--md`, exit
/// code: 1 if an object maps to no kind or a kind's `species` isn't one).
pub fn sizes(
    objects: &[Object],
    measures: &checkglb::Measures,
    reference: &Reference,
) -> (String, BTreeMap<String, String>, i32) {
    report(objects, &size_rows(objects, measures, reference), reference)
}

pub(super) const SPECIES_HEAD: &str = "| Object | Kind | Species | Plant H | Crown W | Crown W/H | Clear trunk | Bark | Colours | Leaf cards | Outside |\n|---|---|---|---|---|---|---|---|---|---|---|";

/// A plant's row of the species table: each measure with its range.
pub(super) fn species_cells(r: &SizeRow, reference: &Reference) -> String {
    let k = r.kind.as_deref().and_then(|k| reference.kind(k));
    let sp = k.and_then(|k| k.species.as_ref());
    let Some(s) = &r.species else {
        return String::new();
    };
    let cell = |v: Option<f64>, range: Option<[f64; 2]>, digits: usize| match (v, range) {
        (None, _) => "—".to_string(),
        (Some(v), None) => format!("{v:.digits$}"),
        (Some(v), Some([lo, hi])) => format!("{v:.digits$} ({lo:.digits$}–{hi:.digits$})"),
    };
    let ratio = s.crown.filter(|_| s.height > 0.0).map(|c| c / s.height);
    let colours = if s.colours.is_empty() {
        "—".to_string()
    } else {
        s.colours
            .iter()
            .map(|c| format!("{} {:.0}°", c.material, c.hue))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let tiles = if s.tiles.is_empty() {
        "—".to_string()
    } else {
        s.tiles
            .iter()
            .map(|(class, n)| {
                let class = if class.is_empty() { "none" } else { class };
                format!("{n} {class}")
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "| `{}` | {} | {} | {} | {} | {} | {} | {} | {colours} | {tiles} | {} |",
        r.id,
        r.kind.as_deref().unwrap_or("—"),
        sp.map_or("—".into(), |sp| format!("*{}*", sp.latin)),
        cell(Some(s.height), k.and_then(|k| k.h), 2),
        cell(s.crown, k.and_then(|k| k.w), 2),
        cell(ratio, sp.and_then(|sp| sp.crown), 2),
        cell(s.clear, sp.and_then(|sp| sp.clear), 2),
        cell(s.bark, sp.and_then(|sp| sp.bark.as_ref()?.luminance), 3),
        if r.outside.is_empty() {
            "ok".to_string()
        } else {
            r.outside.join("; ")
        }
    )
}

pub(super) fn short(v: f32) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
