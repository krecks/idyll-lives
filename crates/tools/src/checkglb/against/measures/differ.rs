//! How two parts differ.

use super::*;
use crate::checkglb::against::{
    AREA, BOUNDS, Finding, Model, NORMAL_AWAY, NORMAL_SHARE, NORMAL_TURN, Numbers, SHADE_LEVELS,
    SHADE_MEAN, SHADE_MOST, SURFACE_FITTED, SURFACE_MAX, SURFACE_MEAN, TEXELS, TRIANGLES, UV_OFF,
    UV_SHARE, found, mm, only, point, share,
};
use crate::checkglb::near::{P3, Part};
use std::collections::{BTreeMap, BTreeSet};

/// The parts: the same nodes' triangles of the same materials with the same attributes, about as many triangles
/// and as much area a material, each node's box, and each part's surface, normals, texture and shading against the
/// other build's.
pub(crate) fn parts_differ(
    old: &Model,
    new: &Model,
    out: &mut Vec<Finding>,
    numbers: &mut Numbers,
) {
    // (A part of a node the other file has not is the nodes' finding.)
    let names_of = |r: &Model, other: &Model| -> Vec<String> {
        r.parts
            .iter()
            .filter(|p| other.tree.paths.contains(&p.path))
            .map(Part::name)
            .collect()
    };
    let (on, nn) = (names_of(old, new), names_of(new, old));
    only(
        "materials",
        "parts (node / material) drawn",
        on.iter(),
        nn.iter(),
        out,
    );

    // A material's triangles and area, over every node.
    let by_material = |r: &Model| -> BTreeMap<String, (usize, f64)> {
        let mut m: BTreeMap<String, (usize, f64)> = BTreeMap::new();
        for p in &r.parts {
            let e = m.entry(p.material.clone()).or_default();
            e.0 += p.triangles.len();
            e.1 += p.area();
        }
        m
    };
    let (om, nm) = (by_material(old), by_material(new));
    for (material, (ta, aa)) in &om {
        let Some((tb, ab)) = nm.get(material) else {
            continue;
        };
        let delta = if *ta == 0 {
            0.0
        } else {
            *tb as f64 / *ta as f64 - 1.0
        };
        if delta.abs() > numbers.triangles.abs() {
            numbers.triangles = delta;
        }
        if delta < TRIANGLES.0 || delta > TRIANGLES.1 || (*ta == 0) != (*tb == 0) {
            out.push(found(
                "triangles",
                format!(
                    "material {material}: {tb} triangles, was {ta} ({:+.1} %; {:+.0} % to {:+.0} %)",
                    delta * 100.0,
                    TRIANGLES.0 * 100.0,
                    TRIANGLES.1 * 100.0
                ),
            ));
        }
        let off = (aa - ab).abs() / aa.max(*ab).max(1e-12);
        if off > AREA && aa.max(*ab) > 1e-8 {
            out.push(found(
                "area",
                format!(
                    "material {material}: {:.4} m², was {:.4} ({} off, at most {})",
                    ab,
                    aa,
                    share(off),
                    share(AREA)
                ),
            ));
        }
    }

    // Each node's box.
    let boxes = |r: &Model| -> BTreeMap<String, (P3, P3)> {
        let mut m: BTreeMap<String, (P3, P3)> = BTreeMap::new();
        for p in &r.parts {
            let Some((lo, hi)) = p.bounds() else { continue };
            m.entry(p.path.clone())
                .and_modify(|(l, h)| {
                    *l = std::array::from_fn(|k| l[k].min(lo[k]));
                    *h = std::array::from_fn(|k| h[k].max(hi[k]));
                })
                .or_insert((lo, hi));
        }
        m
    };
    let (ob, nb) = (boxes(old), boxes(new));
    for (path, (alo, ahi)) in &ob {
        let Some((blo, bhi)) = nb.get(path) else {
            continue;
        };
        let mut worst = (0.0, String::new());
        for k in 0..3 {
            for (side, a, b) in [("lowest", alo[k], blo[k]), ("highest", ahi[k], bhi[k])] {
                if (a - b).abs() > worst.0 {
                    worst = (
                        (a - b).abs(),
                        format!("its {side} {} is {b:.4} m, was {a:.4}", ["x", "y", "z"][k]),
                    );
                }
            }
        }
        if worst.0 > BOUNDS {
            out.push(found(
                "bounds",
                format!(
                    "{path}: {} ({} off, at most {})",
                    worst.1,
                    mm(worst.0),
                    mm(BOUNDS)
                ),
            ));
        }
    }

    // The parts a clip of either build moves: a node it names and all under it.
    let moved: BTreeSet<&String> = old
        .clips
        .iter()
        .chain(&new.clips)
        .flat_map(|c| c.tracks.keys().map(|(node, _)| node))
        .collect();
    let moving = |path: &str| {
        moved.iter().any(|m| {
            path == m.as_str()
                || path
                    .strip_prefix(m.as_str())
                    .is_some_and(|rest| rest.starts_with('/'))
        })
    };
    for a in &old.parts {
        let Some(b) = new
            .parts
            .iter()
            .find(|p| p.path == a.path && p.material == a.material)
        else {
            continue;
        };
        let name = a.name();
        for (what, x, y) in [
            ("NORMAL", a.normals.is_some(), b.normals.is_some()),
            ("TEXCOORD_0", a.uvs.is_some(), b.uvs.is_some()),
            ("COLOR_0", a.greys.is_some(), b.greys.is_some()),
        ] {
            if x != y {
                let side = if x { "old" } else { "new" };
                out.push(found(
                    "materials",
                    format!("{name}: has {what} only in the {side} file"),
                ));
            }
        }
        // Texels a metre: the texture's length over the surface's.
        let density = |p: &Part| {
            let area = p.area();
            if area > 0.0 {
                (p.uv_area() / area).sqrt()
            } else {
                0.0
            }
        };
        let (da, db) = (density(a), density(b));
        if da.max(db) > 1e-9 {
            let off = (da - db).abs() / da.max(db);
            if off > TEXELS {
                out.push(found(
                    "texels",
                    format!(
                        "{name}: its texture runs {db:.4} of itself a metre, was {da:.4} ({} off, at most {})",
                        share(off),
                        share(TEXELS)
                    ),
                ));
            }
        }
        let (ab, ba) = both_ways(a, b);
        let fitted = moving(&a.path) || fruit(&a.material);
        let limit = if fitted { SURFACE_FITTED } else { SURFACE_MAX };
        let mean = ab.mean.max(ba.mean);
        numbers.surface_mean = numbers.surface_mean.max(mean);
        if mean > SURFACE_MEAN {
            out.push(found(
                "surface",
                format!(
                    "{name}: {} from the other build's on average, at most {}",
                    mm(mean),
                    mm(SURFACE_MEAN)
                ),
            ));
        }
        let (max, at, of, from) = if ab.max >= ba.max {
            (ab.max, ab.max_at, "old", "new")
        } else {
            (ba.max, ba.max_at, "new", "old")
        };
        numbers.surface_max = numbers.surface_max.max(max);
        if max > limit {
            out.push(found(
                "surface",
                format!(
                    "{name}: the {of} file's point {} is {} from the {from} surface, at most {}{}",
                    point(at),
                    mm(max),
                    mm(limit),
                    if !fitted {
                        ""
                    } else if moving(&a.path) {
                        " (a moving part)"
                    } else {
                        " (fruit)"
                    }
                ),
            ));
        }
        let worse = if ab.turned >= ba.turned { &ab } else { &ba };
        numbers.turned = numbers.turned.max(worse.turned);
        if worse.turned > NORMAL_SHARE {
            out.push(found(
                "normals",
                format!(
                    "{name}: {} of its surface has its normal turned over {NORMAL_TURN}° (the most {:.1}° at {}), at most {}",
                    share(worse.turned),
                    worse.turn,
                    point(worse.turn_at),
                    share(NORMAL_SHARE)
                ),
            ));
        }
        if ab.away + ba.away > 0 {
            let (w, of) = if ab.away > 0 {
                (&ab, "old")
            } else {
                (&ba, "new")
            };
            out.push(found(
                "normals",
                format!(
                    "{name}: faces the other way at {} points (the {of} file's {}): a normal turned over {NORMAL_AWAY}°",
                    ab.away + ba.away,
                    point(w.away_at)
                ),
            ));
        }
        let worse = [&ab, &ba]
            .into_iter()
            .filter_map(|w| w.uv)
            .max_by(|x, y| x.0.total_cmp(&y.0));
        if let Some((off, furthest, at)) = worse {
            numbers.uv_off = numbers.uv_off.max(off);
            if off > 1.0 - UV_SHARE {
                out.push(found(
                    "uv",
                    format!(
                        "{name}: {} of its surface has its texture over {} off (the most {} at {}), at most {}",
                        share(off),
                        mm(UV_OFF),
                        mm(furthest),
                        point(at),
                        share(1.0 - UV_SHARE)
                    ),
                ));
            }
        }
        let worse = [&ab, &ba]
            .into_iter()
            .filter_map(|w| w.shade)
            .max_by(|x, y| x.0.total_cmp(&y.0));
        if let Some((mean, most, largest, at)) = worse {
            numbers.shade = numbers.shade.max(mean);
            if mean > SHADE_MEAN || most > SHADE_MOST {
                out.push(found(
                    "shading",
                    format!(
                        "{name}: its baked shading is {mean:.2} greys of {SHADE_LEVELS} off on average and {most:.2} over 95 % of it \
                         (the most {largest:.1} at {}), at most {SHADE_MEAN} and {SHADE_MOST}",
                        point(at)
                    ),
                ));
            }
        }
    }
}
