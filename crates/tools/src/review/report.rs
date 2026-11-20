//! The sizes report.
//!
//! Split from `review.rs` (see `mod.rs`).

use super::*;

/// `sizes`'s report of `rows`.
pub fn report(
    objects: &[Object],
    rows: &[SizeRow],
    reference: &Reference,
) -> (String, BTreeMap<String, String>, i32) {
    let mut out = format!(
        "# Sizes against {REFERENCE}\n\nOverall W × H × D within ±{:.0} % of the kind's range (H: a stair's top step; a table's or a desk's without the seats its model carries); seat, top and mattress (slot 0's) heights (the model's, where it has one; a top the content hasn't, the model's near the range) strictly within it.\n",
        reference.tolerance * 100.0
    );
    let mut files = BTreeMap::new();
    for page in page_order(rows.iter().map(|r| r.page.clone())) {
        let here: Vec<&SizeRow> = rows.iter().filter(|r| r.page == page).collect();
        let odd: Vec<&&SizeRow> = here
            .iter()
            .filter(|r| !r.outside.is_empty() && r.problem.is_none())
            .collect();
        let _ = write!(
            out,
            "\n## {page}: {} of {} outside\n",
            odd.len(),
            here.len()
        );
        if !odd.is_empty() {
            let _ = write!(out, "\n{SIZE_HEAD} Outside |\n{SIZE_RULE}---|\n");
            for r in odd {
                let _ = writeln!(out, "{} {} |", size_cells(r), r.outside.join("; "));
            }
        }
        let mut md = format!(
            "# Sizes: {page}\n\n`cargo review sizes --md`, against {REFERENCE} (±{:.0} % on W × H × D; functional heights strict).\n\n{SIZE_HEAD} Status |\n{SIZE_RULE}---|\n",
            reference.tolerance * 100.0
        );
        for r in &here {
            let _ = writeln!(md, "{} {} |", size_cells(r), r.status());
        }
        files.insert(format!("sizes-{page}.md"), md);
    }
    let plants: Vec<&SizeRow> = rows.iter().filter(|r| r.species.is_some()).collect();
    if !plants.is_empty() {
        let _ = write!(
            out,
            "\n## Species: {} of {} plants outside\n\nEach against its kind's `species` and its `h` and `w`, strictly (`species.rs`): the plant's H (from the foot of its trunk or stems), its crown's widest, crown W/H, the clear trunk under the leaves (of H), the bark's luminance, the colours on it that aren't a leaf's green and the leaf cards' triangles by the atlas tile they are on.\n\n{SPECIES_HEAD}\n",
            plants.iter().filter(|r| !r.outside.is_empty()).count(),
            plants.len()
        );
        for r in &plants {
            let _ = writeln!(out, "{}", species_cells(r, reference));
        }
    }
    let mut problems: Vec<&SizeRow> = rows
        .iter()
        .filter(|r| {
            r.problem
                .as_deref()
                .is_some_and(|p| !p.starts_with("unmeasured"))
        })
        .collect();
    problems.sort_by_key(|r| r.page != "models");
    let unmeasured: Vec<&SizeRow> = rows
        .iter()
        .filter(|r| {
            r.problem
                .as_deref()
                .is_some_and(|p| p.starts_with("unmeasured"))
        })
        .collect();
    if !unmeasured.is_empty() {
        let _ = write!(out, "\n## Not measured ({})\n\n", unmeasured.len());
        for r in &unmeasured {
            let _ = writeln!(
                out,
                "- `{}` ({}): {}",
                r.id,
                r.page,
                r.problem.as_deref().unwrap_or_default()
            );
        }
    }
    if !problems.is_empty() {
        let _ = write!(
            out,
            "\n## Error: {} objects map to no kind\n\n",
            problems.len()
        );
        for r in &problems {
            let o = objects.iter().find(|o| o.id == r.id);
            let at = o
                .map(|o| format!("{}, group `{}`, {}", o.category, o.group, o.model))
                .unwrap_or_else(|| r.page.clone());
            let _ = writeln!(
                out,
                "- `{}` ({at}): {}",
                r.id,
                r.problem.as_deref().unwrap_or_default()
            );
        }
    }
    let bad: Vec<String> = reference
        .kinds
        .iter()
        .filter_map(|(name, k)| Some((name, k.species.as_ref()?.problems())))
        .flat_map(|(name, p)| p.into_iter().map(move |p| format!("- `{name}`: {p}")))
        .collect();
    if !bad.is_empty() {
        let _ = write!(
            out,
            "\n## Error: {} `species` entries aren't what a species is\n\n{}\n",
            bad.len(),
            bad.join("\n")
        );
    }
    (
        out,
        files,
        if problems.is_empty() && bad.is_empty() {
            0
        } else {
            1
        },
    )
}
