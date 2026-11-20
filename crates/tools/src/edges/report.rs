//! The report of them.
//!
//! Split from `crates/tools/src/edges.rs` (see `mod.rs`).

use super::*;

/// One model's hard edges that show (`hard_edges`).
#[derive(Debug, Clone, Default)]
pub struct Found {
    pub key: String,
    pub family: String,
    pub triangles: usize,
    pub edges: Vec<Edge>,
}

impl Found {
    /// What its edges come to, in metres.
    pub fn length(&self) -> f64 {
        self.edges.iter().fold(0.0, |sum, e| sum + e.length)
    }
}

/// A coordinate to the centimetre, no `-0.00`.
fn cm(v: f64) -> String {
    format!("{:.2}", if v.abs() < 0.005 { 0.0 } else { v })
}

/// Where an edge is, as its model is built (x across, y back, z up: a file's x, -z and y) and which way it runs.
pub(crate) fn place(e: &Edge) -> String {
    let built = |p: V3| [p[0], -p[2], p[1]];
    let (at, along) = (built(e.at), built(e.along));
    let axis = (0..3).find(|&k| along[k].abs() > 0.985);
    format!(
        "({}, {}, {}) {}",
        cm(at[0]),
        cm(at[1]),
        cm(at[2]),
        axis.map_or("slanted", |k| ["along x", "along y", "upright"][k])
    )
}

/// An edge as the reports name it: its part, material, length and place.
fn named(e: &Edge) -> String {
    format!(
        "{} ({}) {:.2} m at {}",
        e.part,
        e.material,
        e.length,
        place(e)
    )
}

/// The gate's line for a model's hard edges (none: no line): how many, how long, the longest.
pub fn line(key: &str, edges: &[Edge]) -> Option<String> {
    let longest = edges.first()?;
    Some(format!(
        "{} hard edge{} of 5 cm or more show{} unbevelled at the play camera, {:.1} m (`cargo review edges {key}` lists them; as built, z up): the longest {}",
        edges.len(),
        if edges.len() == 1 { "" } else { "s" },
        if edges.len() == 1 { "s" } else { "" },
        edges.iter().fold(0.0, |sum, e| sum + e.length),
        named(longest)
    ))
}

/// The report's text: the families and the models with the most; `all`: every model with one; `listed`: every
/// edge of every model given.
pub fn report(found: &[Found], all: bool, listed: bool) -> String {
    use std::fmt::Write as _;
    let mut text = String::new();
    let with: Vec<&Found> = found.iter().filter(|f| !f.edges.is_empty()).collect();
    let edges: usize = with.iter().map(|f| f.edges.len()).sum();
    let metres = with.iter().fold(0.0, |sum, f| sum + f.length());
    let _ = writeln!(
        text,
        "{} models, {} with hard edges of {} cm or more that show unbevelled at the play camera: {edges} edges, {metres:.0} m of them\n",
        found.len(),
        with.len(),
        LENGTH * 100.0,
    );
    if listed {
        for f in found {
            let _ = writeln!(
                text,
                "{} ({}, {} triangles): {} edges, {:.2} m (as built: x across, y back, z up)",
                f.key,
                f.family,
                f.triangles,
                f.edges.len(),
                f.length()
            );
            for e in &f.edges {
                let _ = writeln!(text, "  {}", named(e));
            }
        }
        return text;
    }
    let mut families: BTreeMap<&str, (usize, usize, usize, f64)> = BTreeMap::new();
    for f in found {
        let family = families.entry(&f.family).or_default();
        family.0 += 1;
        family.1 += usize::from(!f.edges.is_empty());
        family.2 += f.edges.len();
        family.3 += f.length();
    }
    let _ = writeln!(
        text,
        "| family | models | with edges | edges | metres |\n|---|---|---|---|---|"
    );
    let mut rows: Vec<_> = families.iter().collect();
    rows.sort_by(|a, b| b.1.3.total_cmp(&a.1.3).then(a.0.cmp(b.0)));
    for (family, (models, with, edges, metres)) in rows {
        let _ = writeln!(
            text,
            "| {family} | {models} | {with} | {edges} | {metres:.1} |"
        );
    }
    let mut worst = with;
    worst.sort_by(|a, b| b.length().total_cmp(&a.length()).then(a.key.cmp(&b.key)));
    let _ = writeln!(
        text,
        "\nThe most{} (places as built: x across, y back, z up):\n\n| model | triangles | edges | metres | the longest |\n|---|---|---|---|---|",
        if all {
            ""
        } else {
            " (25; `--all` for every one, a model's key for each of its edges)"
        },
    );
    for f in worst.iter().take(if all { usize::MAX } else { 25 }) {
        let longest: Vec<String> = f.edges.iter().take(3).map(named).collect();
        let _ = writeln!(
            text,
            "| {} | {} | {} | {:.1} | {} |",
            f.key,
            f.triangles,
            f.edges.len(),
            f.length(),
            longest.join("; ")
        );
    }
    text
}
