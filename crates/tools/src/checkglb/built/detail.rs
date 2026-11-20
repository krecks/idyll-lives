//! What a house has of the detail a real one has.

use super::*;
use crate::vec3::dot;

// ---- what a house has of the detail a real one has ------------------------------------------------------

/// What a house's meshes have of the detail the eye misses at the play camera: measured, to say what is there
/// before any is added.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Detail {
    /// The walls' trim (`trim`: posts, corner boards, frames, sills, glazing bars, the frieze): its area (m²), and how
    /// much of it is neither plumb nor level (a bevel's or a chamfer's face, a moulding's or a cornice's profile).
    pub trim: f64,
    pub trim_shaped: f64,
    /// Convex edges of 5 cm or more that turn 15° to 59° (an eased arris, a chamfer's two): upright ones (a diagonal
    /// wall's corners are such) and the others, in metres. Not the roofs'.
    pub eased_upright: f64,
    pub eased: f64,
    /// Each storey's roof: its ridges and its hips (m, where two slopes meet level and slanting), the cap shingles
    /// over them (m²: the roof's faces in another tone than its slopes'), and how far its slopes reach under its
    /// eaves (m, under the foot of its fascias: none on a roof that stands as one is built).
    pub roofs: BTreeMap<u32, [f64; 4]>,
    /// How much of the roofs' trim looks up (m²: a gutter's trough would).
    pub troughs: f64,
}

/// `parts`' detail (`Detail`).
pub(crate) fn detail(parts: &[Part], surfaces: &Surfaces) -> Detail {
    let mut out = Detail::default();
    let mm = |p: V3| p.map(|c| (c * 1e4).round() as i64);
    let normal = |p: [V3; 3]| -> Option<(V3, f64)> {
        let (e1, e2) = (
            [0, 1, 2].map(|k| p[1][k] - p[0][k]),
            [0, 1, 2].map(|k| p[2][k] - p[0][k]),
        );
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let l = dot(n, n).sqrt();
        (l > 1e-12).then(|| (n.map(|c| c / l), l / 2.0))
    };
    // Each welded edge's faces as (normal, the face's third corner): the walls' and what stands on them, and each
    // storey's roof slopes.
    type Sides = HashMap<([i64; 3], [i64; 3]), Vec<(V3, V3)>>;
    let mut walls = Sides::new();
    let mut roofs: BTreeMap<u32, Sides> = BTreeMap::new();
    // Each storey's roof: the foot of its fascias (the roofs' upright trim) and the lowest its shingles reach.
    let mut eaves: BTreeMap<u32, [f64; 2]> = BTreeMap::new();
    for part in parts.iter().filter(|p| !p.shadow_only) {
        let material = &part.mesh.material;
        if !part.mesh.edged || !opaque(material, surfaces) {
            continue;
        }
        let shingles = material == "material.roof";
        // The roof's own tone: the vertex colour most of its area has.
        let mut tones: BTreeMap<[u8; 4], f64> = BTreeMap::new();
        let faces: Vec<([V3; 3], V3, f64, [u8; 4])> = part
            .mesh
            .triangles
            .iter()
            .filter_map(|t| {
                let p = t.map(|i| part.mesh.positions[i as usize]);
                let (n, area) = normal(p)?;
                Some((
                    p,
                    n,
                    area,
                    part.colors.get(t[0] as usize).copied().unwrap_or([255; 4]),
                ))
            })
            .collect();
        for f in &faces {
            *tones.entry(f.3).or_default() += f.2;
        }
        let tone = tones
            .iter()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(c, _)| *c);
        for (p, n, area, colour) in faces {
            let own = Some(colour) == tone;
            if part.mesh.part == "trim" {
                out.trim += area;
                if n[1].abs() > 0.02 && n[1].abs() < 0.98 {
                    out.trim_shaped += area;
                }
            }
            if shingles && part.roof && !own {
                out.roofs.entry(part.storey).or_default()[2] += area;
            }
            if part.roof && !shingles && n[1] > 0.5 {
                out.troughs += area;
            }
            if part.roof && (shingles || n[1].abs() < 0.05) {
                let low = p.iter().fold(f64::INFINITY, |low, at| low.min(at[1]));
                let slot = &mut eaves.entry(part.storey).or_insert([f64::INFINITY; 2])
                    [usize::from(shingles)];
                *slot = slot.min(low);
            }
            // (A roof's edges are its slopes' in its own tone; the soffits and fascias under it are no wall's.)
            let sides = match (shingles, part.roof) {
                (true, true) if own && n[1] > UP => roofs.entry(part.storey).or_default(),
                (false, false) => &mut walls,
                _ => continue,
            };
            for k in 0..3 {
                let (a, b) = (mm(p[k]), mm(p[(k + 1) % 3]));
                sides
                    .entry((a.min(b), a.max(b)))
                    .or_default()
                    .push((n, p[(k + 2) % 3]));
            }
        }
    }
    let ends = |key: &([i64; 3], [i64; 3])| -> (V3, V3) {
        (key.0.map(|c| c as f64 / 1e4), key.1.map(|c| c as f64 / 1e4))
    };
    // (Convex: one face's far corner lies behind the other.)
    let convex = |a: V3, faces: &[(V3, V3)]| {
        dot(faces[0].0, [0, 1, 2].map(|k| faces[1].1[k] - a[k])) < -1e-6
    };
    for (key, faces) in walls.iter().filter(|(_, f)| f.len() == 2) {
        let (a, b) = ends(key);
        let d = [0, 1, 2].map(|k| b[k] - a[k]);
        let l = dot(d, d).sqrt();
        let turn = dot(faces[0].0, faces[1].0)
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees();
        if l >= edges::LENGTH && (15.0..60.0).contains(&turn) && convex(a, faces) {
            if d[1].abs() / l > 0.985 {
                out.eased_upright += l;
            } else {
                out.eased += l;
            }
        }
    }
    for (storey, sides) in &roofs {
        for (key, faces) in sides.iter().filter(|(_, f)| f.len() == 2) {
            let (a, b) = ends(key);
            let d = [0, 1, 2].map(|k| b[k] - a[k]);
            let l = dot(d, d).sqrt();
            if l < 1e-6 || dot(faces[0].0, faces[1].0) > 0.999 || !convex(a, faces) {
                continue;
            }
            out.roofs.entry(*storey).or_default()[usize::from(d[1].abs() / l >= 0.02)] += l;
        }
    }
    for (storey, [fascia, shingles]) in eaves {
        if fascia.is_finite() && shingles.is_finite() {
            out.roofs.entry(storey).or_default()[3] = (fascia - shingles).max(0.0);
        }
    }
    out
}
