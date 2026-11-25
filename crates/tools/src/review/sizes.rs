//! Size rows: one object against the reference, and the tops of them.

use super::*;

/// One object against the reference (`cargo review sizes`).
#[derive(Debug, Default)]
pub struct SizeRow {
    pub id: String,
    pub name: String,
    pub page: String,
    /// The model it draws.
    pub model: String,
    pub kind: Option<String>,
    /// W x H x D (m, game axes: across its front, up, front to back).
    pub size: Option<[f64; 3]>,
    /// The seat, top and mattress the game uses and what its model has there (m; the mattress at slot 0).
    pub seat: Option<(f64, Option<f64>)>,
    pub surface: Option<(f64, Option<f64>)>,
    pub mattress: Option<(f64, Option<f64>)>,
    /// Its model's top where the content has no `surface` and its kind has one (`checkglb::Measure::top`): within
    /// `TOP_NEAR` of the kind's range, else None (and the nearest is in `outside`).
    pub top: Option<f64>,
    /// The H compared where it isn't the whole model's, and what it is (`h_of`).
    pub h: Option<(&'static str, f64)>,
    /// A stair's steps: how many, and the rise of each (m).
    pub steps: Option<(usize, f64)>,
    /// Its plant as measured against its kind's `species` (`species_row`).
    pub species: Option<Shape>,
    /// What is outside its kind's ranges: each a finding the baseline counts (`--check`).
    pub outside: Vec<String>,
    /// Why it couldn't be checked (no kind, a kind `kinds` doesn't have, an unreadable model).
    pub problem: Option<String>,
}

impl SizeRow {
    pub fn status(&self) -> String {
        match (&self.problem, self.outside.is_empty()) {
            (Some(p), _) => p.clone(),
            (None, true) => "ok".into(),
            (None, false) => format!("out: {}", self.outside.join("; ")),
        }
    }
}

/// The height `what` of `heights` (the first of `whats`): the game's and the model's.
fn height_of(heights: &[Height], whats: &[&str]) -> Option<(f64, Option<f64>)> {
    heights
        .iter()
        .find(|h| whats.contains(&h.what))
        .map(|h| (h.expected, h.mesh))
}

/// How far from its kind's top range (a share of each end, as `within`'s tolerance) a top found where the content
/// has none may be and still count as its top: a face looking up any farther is something else (a shelf, a seat).
pub const TOP_NEAR: f64 = 0.20;

/// The H to compare where the model's whole height isn't the kind's (`kind`): a stair's top step (the storey it
/// climbs, which its kind gives, not the handrail over it); for a kind with a top and no seat (a table or a desk
/// alone: a set's kind, with a seat, has the set's ranges), one sat at whose model carries its seats, the table
/// alone (`checkglb::Measure::table`: not its chairs' backs).
fn h_of(m: &Measure, kind: &Kind) -> Option<(&'static str, f64)> {
    match (m.steps, m.table) {
        (Some((top, _)), _) => Some(("top step", top)),
        (None, Some(table)) if kind.surface.is_some() && kind.seat.is_none() => {
            Some(("table", table))
        }
        _ => None,
    }
}

/// `object` (its id, model, category and group) measured (`measure`), against its kind in `reference`. The
/// functional heights are the model's where it has one there (else the game's), strictly in range; the top,
/// where the content has none, the model's top near the kind's range (`TOP_NEAR`).
pub fn size_row(
    object: &Object,
    measure: Option<&Result<Measure, String>>,
    reference: &Reference,
) -> SizeRow {
    let mut row = SizeRow {
        id: object.id.clone(),
        name: object.name.clone(),
        page: object.page.clone(),
        model: object.model.clone(),
        ..SizeRow::default()
    };
    let measured = measure.and_then(|m| m.as_ref().ok());
    match measure {
        Some(Ok(m)) => {
            row.size = Some(m.size);
            row.seat = height_of(&m.heights, &["seat"]);
            row.surface = height_of(&m.heights, &["surface"]);
            row.mattress = height_of(&m.heights, &["mattress", "lower bunk"]);
            row.steps = m.steps.map(|(top, n)| (n, top / n as f64));
        }
        Some(Err(e)) => row.problem = Some(format!("unmeasured: {e}")),
        None => row.problem = Some("unmeasured: no model".into()),
    }
    let Some(kind) = reference.kind_of(&object.id, &object.model, &object.category, &object.group)
    else {
        row.problem = Some("no kind".into());
        return row;
    };
    row.kind = Some(kind.to_string());
    let Some(k) = reference.kind(kind) else {
        row.problem = Some(format!("kind `{kind}` isn't in `kinds`"));
        return row;
    };
    row.h = measured.and_then(|m| h_of(m, k));
    if let Some(size) = row.size {
        let (h_name, h) = match row.h {
            Some((by, h)) => (format!("H ({by})"), h),
            None => ("H".into(), size[1]),
        };
        // A plant's W and H are its crown's and its own, strictly (`species_row`).
        let (w, h_range) = if k.species.is_some() {
            (None, None)
        } else {
            (k.w, k.h)
        };
        for (name, v, range) in [
            ("W", size[0], w),
            (h_name.as_str(), h, h_range),
            ("D", size[2], k.d),
        ] {
            if let Some(range) = range.filter(|r| !within(v, *r, reference.tolerance)) {
                row.outside
                    .push(outside(name, v, range, reference.tolerance));
            }
        }
    }
    if row.problem.is_none() {
        // A top the content hasn't: the model's, if near enough the range to be one.
        let found_top = measured
            .and_then(|m| m.top)
            .filter(|_| row.surface.is_none())
            .map(|(_, h)| h);
        let near = |h: f64, range: [f64; 2]| within(h, range, TOP_NEAR);
        row.top = found_top
            .zip(k.surface)
            .filter(|(h, r)| near(*h, *r))
            .map(|(h, _)| h);
        let top = row
            .surface
            .map(|(game, mesh)| mesh.unwrap_or(game))
            .or(row.top);
        for (name, found, range) in [
            (
                "seat",
                row.seat.map(|(game, mesh)| mesh.unwrap_or(game)),
                k.seat,
            ),
            ("top", top, k.surface),
            (
                "mattress",
                row.mattress.map(|(game, mesh)| mesh.unwrap_or(game)),
                k.mattress,
            ),
        ] {
            let Some(range) = range else { continue };
            match found {
                None => {
                    let nearest = if name == "top" {
                        found_top
                            .map(|h| format!("; the nearest face looking up {h:.3}"))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    row.outside.push(format!(
                        "no {name} to measure ({:.2}–{:.2}{nearest})",
                        range[0], range[1]
                    ));
                }
                Some(v) if !within(v, range, 0.0) => row.outside.push(outside(name, v, range, 0.0)),
                Some(_) => {}
            }
        }
    }
    row
}

/// A size row for a model key the reference lists in `ids` that no object draws: its size, and its seat where its
/// kind has one (`checkglb::Measures::seats`).
fn model_row(
    key: &str,
    size: &Result<[f64; 3], String>,
    seat: Option<f64>,
    reference: &Reference,
) -> SizeRow {
    let object = Object {
        id: key.into(),
        name: String::new(),
        page: "models".into(),
        model: key.into(),
        ..Object::default()
    };
    // (No content says where its sitters' hips are: the seat is the model's, as the game's.)
    let heights = seat
        .map(|h| {
            vec![Height {
                what: "seat",
                expected: h,
                mesh: Some(h),
                ..Height::default()
            }]
        })
        .unwrap_or_default();
    let measure = size.clone().map(|size| Measure {
        model: key.into(),
        size,
        heights,
        ..Measure::default()
    });
    size_row(&object, Some(&measure), reference)
}

fn cell_height(h: Option<(f64, Option<f64>)>) -> String {
    match h {
        None => "—".into(),
        Some((game, Some(mesh))) if (mesh - game).abs() > 0.02 => {
            format!("{mesh:.2} (game {game:.2})")
        }
        Some((_, Some(mesh))) => format!("{mesh:.2}"),
        Some((game, None)) => format!("{game:.2} (no model there)"),
    }
}

/// W × H × D, with the H compared where it isn't the whole model's: `1.62 × 0.80 (table; 1.03 in all) × 0.87`,
/// `0.86 × 2.80 (top step; 14 steps of 0.200; 3.68 in all) × 4.00`.
fn size_cell(r: &SizeRow) -> String {
    let Some(s) = r.size else { return "—".into() };
    let h = match (r.h, r.steps) {
        (Some((by, h)), Some((n, rise))) => {
            format!("{h:.2} ({by}; {n} steps of {rise:.3}; {:.2} in all)", s[1])
        }
        (Some((by, h)), None) => format!("{h:.2} ({by}; {:.2} in all)", s[1]),
        (None, _) => format!("{:.2}", s[1]),
    };
    format!("{:.2} × {h} × {:.2}", s[0], s[2])
}

pub(super) fn size_cells(r: &SizeRow) -> String {
    let name = if r.name.is_empty() {
        String::new()
    } else {
        format!(" {}", r.name)
    };
    let top = match (r.surface, r.top) {
        (None, Some(top)) => format!("{top:.2} (no content `surface`)"),
        (surface, _) => cell_height(surface),
    };
    format!(
        "| `{}`{name} | {} | {} | {} | {top} | {} |",
        r.id,
        r.kind.as_deref().unwrap_or("—"),
        size_cell(r),
        cell_height(r.seat),
        cell_height(r.mattress)
    )
}

pub(super) const SIZE_HEAD: &str = "| Object | Kind | W × H × D (m) | Seat | Top | Mattress |";
pub(super) const SIZE_RULE: &str = "|---|---|---|---|---|---|";

/// The pages in the buy catalogue's order, then any other.
pub(super) fn page_order(pages: impl IntoIterator<Item = String>) -> Vec<String> {
    const ORDER: [&str; 16] = [
        "kitchen",
        "bathroom",
        "bedroom",
        "kids",
        "living",
        "office",
        "fitness",
        "hobbies",
        "decor",
        "tabletop",
        "lighting",
        "outdoor",
        "garden",
        "community",
        "packs",
        "models",
    ];
    let mut all: Vec<String> = pages
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    all.sort_by_key(|p| ORDER.iter().position(|o| o == p).unwrap_or(ORDER.len()));
    all
}

/// {object id: its kind's top range}, for each object whose kind has one: what `checkglb::measures` looks for where
/// the content has no `surface`.
pub fn tops(objects: &[Object], reference: &Reference) -> BTreeMap<String, [f64; 2]> {
    let top = |o: &Object| {
        reference
            .kind_of(&o.id, &o.model, &o.category, &o.group)
            .and_then(|k| reference.kind(k))
            .and_then(|k| k.surface)
    };
    objects
        .iter()
        .filter_map(|o| Some((o.id.clone(), top(o)?)))
        .collect()
}

/// The model keys in `ids` that no object draws (`model_row`), each with its kind's seat range where it has one:
/// what `checkglb::measures` measures a seat on.
pub fn model_seats(objects: &[Object], reference: &Reference) -> BTreeMap<String, [f64; 2]> {
    let drawn: HashSet<&str> = objects.iter().map(|o| o.model.as_str()).collect();
    reference
        .ids
        .iter()
        .filter(|(k, _)| k.contains('.') && !drawn.contains(k.as_str()))
        .filter_map(|(k, kind)| Some((k.clone(), reference.kind(kind)?.seat?)))
        .collect()
}

/// Every object's size row (`size_row`), and one for each model key in `ids` that no object draws.
pub fn size_rows(
    objects: &[Object],
    measures: &checkglb::Measures,
    reference: &Reference,
) -> Vec<SizeRow> {
    let mut rows: Vec<SizeRow> = objects
        .iter()
        .map(|o| size_row(o, measures.objects.get(&o.id), reference))
        .collect();
    let drawn: HashSet<&str> = objects.iter().map(|o| o.model.as_str()).collect();
    for key in reference.ids.keys().filter(|k| {
        k.contains('.') && measures.models.contains_key(*k) && !drawn.contains(k.as_str())
    }) {
        rows.push(model_row(
            key,
            &measures.models[key],
            measures.seats.get(key).copied(),
            reference,
        ));
    }
    rows
}
