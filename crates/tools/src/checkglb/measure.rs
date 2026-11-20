//! Measures: how big a model is drawn and the tops it offers.

use super::*;

/// A model file read: its size at rest (m, game axes: across its front x, up, front to back, as the gate's
/// summary has it), the file and its entry (for `heights`).
pub(crate) struct Model {
    pub(crate) size: [f64; 3],
    pub(crate) file: Glb,
    pub(crate) entry: J,
}

/// A content object's model, its size (`Model`), its functional heights (`heights`, a mattress at slot 0 only)
/// and what `cargo review sizes` measures besides.
#[derive(Default)]
pub struct Measure {
    pub model: String,
    pub size: [f64; 3],
    pub heights: Vec<Height>,
    /// Its top where the content has no `surface` but its kind has one (`top_near`): where (m, game axes: right
    /// of the footprint's middle, forward of it) and how high.
    pub top: Option<([f64; 2], f64)>,
    /// How high it reaches without its seats, for one sat at that has a top (`Sampler::height_by`).
    pub table: Option<f64>,
    /// A stair's (content `stairs`) top step's height and how many steps it has (`Sampler::steps`).
    pub steps: Option<(f64, usize)>,
}

/// The top of an object with no content `surface` whose kind has one, `range` (m), measured as `heights`
/// measures a seat: on vertical lines near the middle, the face looking up nearest the range. The lines are at
/// the middle of its faces' box seen from above (a thing against a wall stands off its footprint's middle) and
/// at the eight points a quarter of its width and depth from there: a worktop beside a basin or a hob in the
/// middle, while a quarter in is still on a table's top, inside its rim or on its board. Lines within
/// `SEAT_CLEAR` of a seat's hips (`seats`) are left out (a chair's seat or back is no top), unless none of the
/// others finds a face (a stash box sat on: its top is its seat). Of the faces found, the highest that doesn't
/// stand on another: one with a face 2 cm or more under it on its line that is what another line found is a
/// thing on the top (a vase, a tool, a plate: the top goes on under it), while a basin's or a hob's floor is
/// lower than the worktop round it, which has no top under it. (Where, how high); None: no face looks up there.
pub(crate) fn top_near(
    s: &Sampler,
    range: [f64; 2],
    seats: &[[f64; 2]],
) -> Option<([f64; 2], f64)> {
    // (line, the face nearest the range there, every face there).
    let on = |lines: Vec<[f64; 2]>| -> Vec<([f64; 2], f64, Vec<f64>)> {
        lines
            .iter()
            .filter_map(|p| Some((*p, s.top_in(p[0], p[1], range)?, s.tops_at(p[0], p[1]))))
            .collect()
    };
    let mut found = on(top_lines(s, seats));
    if found.is_empty() {
        found = on(top_lines(s, &[]));
    }
    let stands = |(_, h, faces): &([f64; 2], f64, Vec<f64>)| {
        faces
            .iter()
            .any(|q| *q <= h - 0.02 && found.iter().any(|(_, other, _)| (other - q).abs() <= 0.01))
    };
    found
        .iter()
        .filter(|f| !stands(f))
        .map(|(p, h, _)| (*p, *h))
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// The vertical lines `top_near` looks along, those within `SEAT_CLEAR` of `seats` left out (m, game axes;
/// across, then front to back).
pub(crate) fn top_lines(s: &Sampler, seats: &[[f64; 2]]) -> Vec<[f64; 2]> {
    const SEAT_CLEAR: f64 = 0.3;
    let Some((lo, hi)) = s.extent() else {
        return Vec::new();
    };
    let mid = [(lo[0] + hi[0]) / 2.0, (lo[2] + hi[2]) / 2.0];
    let quarter = [(hi[0] - lo[0]) / 4.0, (hi[2] - lo[2]) / 4.0];
    let grid = [-1.0, 0.0, 1.0]
        .iter()
        .flat_map(|i| [-1.0, 0.0, 1.0].map(|k| [mid[0] + i * quarter[0], mid[1] + k * quarter[1]]));
    grid.filter(|p| seats.iter().all(|s| dist(p, s) >= SEAT_CLEAR))
        .collect()
}

/// Measures content object `f` drawn by the model `m` (its `key`) for `cargo review sizes`; `top` is its kind's
/// top range.
fn measure(
    key: &str,
    m: &Model,
    f: &Facts,
    furniture: &Furniture,
    top: Option<[f64; 2]>,
) -> R<Measure> {
    let mut out = Measure {
        model: key.into(),
        size: m.size,
        heights: heights(&m.file, &m.entry, &f.object, furniture, false)?,
        ..Measure::default()
    };
    let seats = seat_of(&f.object, furniture)
        .map(|s| s.points)
        .unwrap_or_default();
    let top = top.filter(|_| f.object.opt("surface").is_none());
    if top.is_none() && seats.is_empty() && !f.stairs {
        return Ok(out);
    }
    let Some(s) = Sampler::new(&m.file, &m.entry, None)? else {
        return Ok(out);
    };
    out.top = top.and_then(|range| top_near(&s, range, &seats));
    let on = out
        .heights
        .iter()
        .find(|h| h.what == "surface")
        .and_then(|h| h.mesh)
        .or(out.top.map(|(_, h)| h));
    if !seats.is_empty()
        && let Some(on) = on
    {
        out.table = s.height_by(on);
    }
    if f.stairs {
        out.steps = s.steps();
    }
    Ok(out)
}

/// Every model the manifests have a file for, measured ({key: its size, or why it can't be read}), and every
/// content object's measure ({id: …}; an object whose model can't be read has the reason).
pub struct Measures {
    pub models: BTreeMap<String, Result<[f64; 3], String>>,
    pub objects: BTreeMap<String, Result<Measure, String>>,
}

/// The most draw calls a model may take: the pack entry's `draws` for the one model whose moving parts can't fit
/// in four (the grandfather clock's trunk door, which swings open while it is wound), four for every other.
pub(crate) fn draws_allowed(entry: &J) -> Result<usize, String> {
    match entry.get("draws") {
        None => Ok(DRAWS.1),
        Some(v) => match v.num().map(|n| n.f()) {
            Some(n) if n.fract() == 0.0 && (DRAWS.0 as f64..=8.0).contains(&n) => Ok(n as usize),
            _ => Err(format!(
                "`draws` is a whole number of draw calls, {} to 8 (one per finish group)",
                DRAWS.0
            )),
        },
    }
}

/// Reads and measures a model file of the manifests (`path`) with its entry: its size at rest, times `scale`.
pub(crate) fn model(entry: &J, path: &str) -> R<Model> {
    let file = parse_glb(&std::fs::read(path).map_err(|e| py::os_error(&e, path))?)?;
    // Its own size: what it carries (`ASIDE`) is drawn but isn't the piece of furniture.
    let ((lo, hi), _) = bounds(&file.gltf, &HashMap::new(), &aside_nodes(&file.gltf))?;
    let fit = entry.get("fit").is_some_and(J::truthy);
    let scale = if fit {
        1.0
    } else {
        entry.get("scale").and_then(J::num).map_or(1.0, Num::f)
    };
    Ok(Model {
        size: [0, 1, 2].map(|k| (hi[k].f() - lo[k].f()) * scale),
        file,
        entry: entry.clone(),
    })
}

/// Measures every model and content object (`Measures`), for `cargo review sizes`; `tops`: {object id: its
/// kind's top range}, for those whose content has no `surface` (`top_near`).
pub fn measures(paths: &Paths, tops: &BTreeMap<String, [f64; 2]>) -> Result<Measures, String> {
    let stack = asset_manifests(paths)?;
    let defs = content(paths)?;
    let facts = content_objects(&defs)?;
    let furniture = Furniture::read(paths)?;
    let (entries, _) = model_entries(&stack);
    let mut read: BTreeMap<String, Result<Model, String>> = BTreeMap::new();
    for (k, (entry, manifest)) in &entries {
        let url = entry
            .get("url")
            .filter(|u| u.truthy())
            .map(J::key)
            .unwrap_or_default();
        let found = if url.is_empty() || url.contains("://") {
            Err(format!(
                "its entry has no file here ({})",
                if url.is_empty() { "no `url`" } else { &url }
            ))
        } else {
            let path = normpath(&join(&dirname(manifest), &url));
            model(entry, &path).map_err(|e| format!("cannot read {path}: {e}"))
        };
        read.insert(k.clone(), found);
    }
    let mut objects = BTreeMap::new();
    for (k, each) in &facts {
        for f in each {
            let measured = match read.get(k) {
                None => Err(format!("no manifest has a model entry {k}")),
                Some(Err(e)) => Err(e.clone()),
                Some(Ok(m)) => measure(k, m, f, &furniture, tops.get(&str_of(&f.id)).copied()),
            };
            objects.insert(str_of(&f.id), measured);
        }
    }
    let models = read
        .into_iter()
        .map(|(k, m)| (k, m.map(|m| m.size)))
        .collect();
    Ok(Measures { models, objects })
}
