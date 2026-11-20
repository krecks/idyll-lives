//! The hard-edge rule over the built world, as the game draws it.

use super::*;

// ---- edges --------------------------------------------------------------------------------------------

/// The parts a camera draws (not the shadow map's own), as the edge rule reads them: what hides what behind it.
pub(super) fn drawn(parts: &[Part], surfaces: &Surfaces) -> Vec<Mesh> {
    parts
        .iter()
        .filter(|p| !p.shadow_only)
        .map(|p| Mesh {
            opaque: opaque(&p.mesh.material, surfaces),
            ..p.mesh.clone()
        })
        .collect()
}

/// The cutaway's look bits (web/src/render/shared/geometry.ts `LOOK`) for a camera the way `way` from what it looks at,
/// as the renderer sets them (`PlayCanvasRenderer.updateCutaway`: a bit where the camera looks more than 0.3 along
/// an axis). The dump's z is the game's mirrored.
pub(super) fn look_bits(way: V3) -> u8 {
    let flat = (way[0] * way[0] + way[2] * way[2]).sqrt().max(1e-12);
    let (dx, dz) = (-way[0] / flat, way[2] / flat);
    u8::from(dx > 0.3)
        | u8::from(dx < -0.3) << 1
        | u8::from(dz > 0.3) << 2
        | u8::from(dz < -0.3) << 3
}

/// A house's parts as drawn with the camera near (no roofs) on storey `storey` (those above it hidden), the walls
/// cut away for a camera looking `look` (0: every wall standing): a vertex whose two masks both have a bit of
/// `look` drops to the stub's height over its storey's floor, or goes with its triangles (`shaders/cutaway.ts`).
pub(super) fn cut_away(
    parts: &[Part],
    surfaces: &Surfaces,
    index: &Index,
    storey: u32,
    look: u8,
) -> Vec<Mesh> {
    parts
        .iter()
        .filter(|p| !p.shadow_only && !p.roof && p.storey <= storey)
        .map(|p| {
            let mut mesh = Mesh {
                opaque: opaque(&p.mesh.material, surfaces),
                ..p.mesh.clone()
            };
            if look == 0 || p.cut.len() != mesh.positions.len() {
                return mesh;
            }
            let cut = |i: usize| p.cut[i][0] & look != 0 && p.cut[i][2] & look != 0;
            let top = p.storey as f64 * index.storey + index.stub;
            for (i, at) in mesh.positions.iter_mut().enumerate() {
                if cut(i) && p.cut[i][1] == 1 {
                    at[1] = at[1].min(top);
                }
            }
            mesh.triangles.retain(|t| {
                !t.iter()
                    .any(|&i| cut(i as usize) && p.cut[i as usize][1] == 2)
            });
            mesh
        })
        .collect()
}

/// The line an edge lies on, to the millimetre: its part and material, its direction and where it passes
/// nearest the origin. The same edge found in two states of a house is on the same line.
pub(super) type Line = (String, String, [i64; 3], [i64; 3]);

pub(super) fn line_of(e: &Edge) -> Line {
    let first = e
        .along
        .iter()
        .find(|c| c.abs() > 1e-6)
        .copied()
        .unwrap_or(1.0);
    let along = e.along.map(|c| if first < 0.0 { -c } else { c });
    let on = (0..3).fold(0.0, |sum, k| sum + e.at[k] * along[k]);
    let foot = [0, 1, 2].map(|k| e.at[k] - along[k] * on);
    (
        e.part.clone(),
        e.material.clone(),
        along.map(|c| (c * 1e3).round() as i64),
        foot.map(|c| (c * 1e3).round() as i64),
    )
}

/// A sample's hard edges that show unbevelled, by the line they lie on: how many, how long together, and the
/// longest of them.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Edges(pub BTreeMap<Line, (usize, f64, Option<Edge>)>);

impl Edges {
    /// One state's edges by line.
    pub(super) fn of(edges: Vec<Edge>) -> Edges {
        let mut out = Edges::default();
        for e in edges {
            let slot = out.0.entry(line_of(&e)).or_default();
            slot.0 += 1;
            slot.1 += e.length;
            if slot.2.as_ref().is_none_or(|l| e.length > l.length) {
                slot.2 = Some(e);
            }
        }
        out
    }

    /// With another state's: a line both have keeps the one that shows more of it.
    pub(super) fn or(&mut self, other: Edges) {
        for (line, found) in other.0 {
            match self.0.get_mut(&line) {
                Some(mine) if mine.1 >= found.1 => {}
                Some(mine) => *mine = found,
                None => {
                    self.0.insert(line, found);
                }
            }
        }
    }

    pub fn count(&self) -> usize {
        self.0.values().map(|v| v.0).sum()
    }

    /// Their length together (m).
    pub fn length(&self) -> f64 {
        self.0.values().fold(0.0, |sum, v| sum + v.1)
    }

    /// The longest edge of each line, the longest first.
    pub(super) fn longest(&self) -> Vec<&Edge> {
        let mut out: Vec<&Edge> = self.0.values().filter_map(|v| v.2.as_ref()).collect();
        out.sort_by(|a, b| {
            b.length
                .total_cmp(&a.length)
                .then_with(|| a.at.map(f64::to_bits).cmp(&b.at.map(f64::to_bits)))
        });
        out
    }
}

/// The hard edges a house shows (`edges::hard_edges`'s rule) in the states the play camera sees it in:
///
/// - from 15 m out and further, whole, its roof on (`ROOF_MIN_DISTANCE`);
/// - nearer, for each storey in view (those above it hidden, no roof): with its walls standing, which the steep
///   views look down into; and with the walls on the camera's side cut away, each of the eight ways the cutaway
///   stands (`look_bits`) seen from the views that make it stand so.
///
/// An edge on the same line in several states counts once, by the state that shows most of it.
pub(crate) fn house_edges(
    parts: &[Part],
    surfaces: &Surfaces,
    index: &Index,
    storeys: u32,
) -> Edges {
    let mut out = Edges::of(edges::hard_edges(&drawn(parts, surfaces)));
    for storey in 0..storeys.max(1) {
        out.or(Edges::of(edges::hard_edges(&cut_away(
            parts, surfaces, index, storey, 0,
        ))));
        // (The looks the play camera's views make: one bit square on to a wall, two from a corner.)
        for look in [1, 2, 4, 8, 5, 6, 9, 10] {
            let meshes = cut_away(parts, surfaces, index, storey, look);
            out.or(Edges::of(edges::hard_edges_from(&meshes, &|way| {
                look_bits(way) == look
            })));
        }
    }
    out
}

/// A sample's hard edges: a house's by `house_edges`, anything else's as a model's.
pub(crate) fn sample_edges(
    sample: &Listed,
    parts: &[Part],
    surfaces: &Surfaces,
    index: &Index,
) -> Edges {
    if sample.kind == "house" {
        house_edges(parts, surfaces, index, sample.storeys.unwrap_or(1))
    } else {
        Edges::of(edges::hard_edges(&drawn(parts, surfaces)))
    }
}

/// Which way an edge runs: upright, level or slanted.
pub(crate) fn runs(e: &Edge) -> &'static str {
    match e.along[1].abs() {
        y if y > 0.985 => "upright",
        y if y < 0.17 => "level",
        _ => "slanted",
    }
}

/// Where an edge is, in the game's own x and z (the dump's z mirrored back), to the centimetre.
pub(super) fn place(e: &Edge) -> String {
    let cm = |v: f64| format!("{:.2}", if v.abs() < 0.005 { 0.0 } else { v });
    format!(
        "{} ({}) {:.2} m {} at ({}, {}, {})",
        e.part,
        e.material,
        e.length,
        runs(e),
        cm(e.at[0]),
        cm(e.at[1]),
        cm(-e.at[2])
    )
}

/// The gate's line for a sample's hard edges (none: no line).
pub(super) fn edges_line(edges: &Edges) -> Option<String> {
    let longest = edges.longest();
    let first = longest.first()?;
    let n = edges.count();
    Some(format!(
        "{n} hard edge{} of 5 cm or more show{} unbevelled at the play camera, {:.1} m (`--report` says where; x, height, z): the longest {}",
        if n == 1 { "" } else { "s" },
        if n == 1 { "s" } else { "" },
        edges.length(),
        place(first)
    ))
}
