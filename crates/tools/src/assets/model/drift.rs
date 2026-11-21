//! How far a copy has drifted from its source.

use super::*;

/// How far a copy is from its source at most: positions (m), normals and texture coordinates (a
/// component's difference).
#[derive(Clone, Copy, Debug, Default)]
pub struct Drift {
    pub position: f32,
    pub normal: f32,
    pub uv: f32,
    pub triangles: usize,
}

/// Compares a copy with its source: the same meshes, primitives and materials, every triangle of the
/// source once in the copy (in any order, started at any corner) with its corners within the
/// quantisation's steps and its shading the same, and the animations the same to the bit.
pub fn compare(source: &Model, copy: &Model) -> Result<Drift, String> {
    if source.meshes.len() != copy.meshes.len() {
        return Err(format!(
            "{} meshes, the source has {}",
            copy.meshes.len(),
            source.meshes.len()
        ));
    }
    if source.animations != copy.animations {
        return Err("its animations differ from the source's".into());
    }
    let mut names = source.nodes.clone();
    names.sort();
    let mut copied = copy.nodes.clone();
    copied.sort();
    copied.dedup();
    names.dedup();
    if names != copied {
        return Err("its nodes' names differ from the source's".into());
    }
    let mut drift = Drift::default();
    for (m, (a, b)) in source.meshes.iter().zip(&copy.meshes).enumerate() {
        if a.len() != b.len() {
            return Err(format!(
                "mesh {m}: {} primitives, the source has {}",
                b.len(),
                a.len()
            ));
        }
        // What 14 bits of the mesh's box allow: a step (half of one from rounding, a quarter from the top bits
        // repeated below, the rest for the floats' own rounding on the way through the nodes). The box is the
        // mesh's own, as stored: a node may squash it (a stream of water flat until it runs), and the step
        // with it only along the squash.
        let reach = source.spans.get(m).copied().unwrap_or(0.0) / 8191.0 + 2e-6;
        for (p, (a, b)) in a.iter().zip(b).enumerate() {
            let at = format!("mesh {m}, primitive {p}");
            if a.material != b.material {
                return Err(format!("{at}: another material than the source's"));
            }
            if a.corners.len() != b.corners.len() {
                return Err(format!(
                    "{at}: {} triangles, the source has {}",
                    b.corners.len() / 3,
                    a.corners.len() / 3
                ));
            }
            // The source's triangles by the cell their middle is in; each of the copy's takes the one it fits
            // best. Where two of the source's are within a step of each other (a sliver, a face made twice)
            // one may be taken by the wrong one of the copy's: its taker then moves on to another it fits.
            let cell = (reach * 4.0).max(1e-5);
            let key = |t: &[Corner]| -> [i64; 3] {
                [0, 1, 2].map(|k| {
                    ((t[0].position[k] + t[1].position[k] + t[2].position[k]) / 3.0 / cell).floor()
                        as i64
                })
            };
            let mut cells: HashMap<[i64; 3], Vec<usize>> = HashMap::new();
            for (i, t) in a.corners.as_chunks::<3>().0.iter().enumerate() {
                cells.entry(key(t)).or_default().push(i);
            }
            let off =
                |u: &[f32], w: &[f32]| u.iter().zip(w).fold(0f32, |m, (p, q)| m.max((p - q).abs()));
            // How far the source's triangle `i` is from `t`, started at the corner where it is nearest (`fits`: within the steps, shaded the same).
            let apart = |i: usize, t: &[Corner]| -> (Drift, bool) {
                let s = &a.corners[i * 3..i * 3 + 3];
                let mut best: Option<(Drift, bool)> = None;
                for turn in 0..3 {
                    let mut d = Drift::default();
                    let mut shaded = true;
                    for c in 0..3 {
                        let (x, y) = (&s[(c + turn) % 3], &t[c]);
                        d.position = d.position.max(off(&x.position, &y.position));
                        d.normal = d.normal.max(off(&x.normal, &y.normal));
                        d.uv = d.uv.max(off(&x.uv, &y.uv));
                        shaded &= x.color == y.color;
                    }
                    let fits =
                        shaded && d.position <= reach && d.normal <= 0.0025 && d.uv <= 0.0003;
                    if best
                        .as_ref()
                        .is_none_or(|(b, f)| (fits, -d.position) > (*f, -b.position))
                    {
                        best = Some((d, fits));
                    }
                }
                best.expect("three turns")
            };
            // The source's triangles `t` fits, the nearest first.
            let fitting = |t: &[Corner]| -> Vec<(usize, Drift)> {
                let k = key(t);
                let mut out = Vec::new();
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            for &i in cells
                                .get(&[k[0] + dx, k[1] + dy, k[2] + dz])
                                .map_or(&[][..], Vec::as_slice)
                            {
                                let (d, fits) = apart(i, t);
                                if fits {
                                    out.push((i, d));
                                }
                            }
                        }
                    }
                }
                out.sort_by(|x, y| x.1.position.total_cmp(&y.1.position));
                out
            };
            const FREE: usize = usize::MAX;
            let mut taker = vec![FREE; a.corners.len() / 3];
            for (n, t) in b.corners.as_chunks::<3>().0.iter().enumerate() {
                let fits = fitting(t);
                let mut took = fits
                    .iter()
                    .find(|(i, _)| taker[*i] == FREE)
                    .map(|&(i, d)| (i, d));
                if took.is_none() {
                    'swap: for &(i, d) in &fits {
                        let other = taker[i];
                        for (j, _) in fitting(&b.corners[other * 3..other * 3 + 3]) {
                            if taker[j] == FREE {
                                taker[j] = other;
                                took = Some((i, d));
                                break 'swap;
                            }
                        }
                    }
                }
                let Some((i, worst)) = took else {
                    // The source's nearest triangle, and how it differs.
                    let nearest = (0..a.corners.len() / 3)
                        .map(|i| (i, apart(i, t).0))
                        .min_by(|x, y| x.1.position.total_cmp(&y.1.position));
                    let said = nearest.map_or(String::new(), |(i, d)| {
                        format!(
                            ": the nearest is {:.6} m off, its normals {:.4}, its texture coordinates {:.5}, its shading {}{}",
                            d.position,
                            d.normal,
                            d.uv,
                            if (0..3).any(|turn| (0..3).all(|c| a.corners[i * 3 + (c + turn) % 3].color == t[c].color)) { "the same" } else { "another" },
                            if taker[i] == FREE { "" } else { ", and another triangle of the copy has taken it" }
                        )
                    });
                    return Err(format!(
                        "{at}: a triangle at {:?} that the source doesn't have (within {reach:.6} m){said}",
                        t[0].position
                    ));
                };
                taker[i] = n;
                drift.position = drift.position.max(worst.position);
                drift.normal = drift.normal.max(worst.normal);
                drift.uv = drift.uv.max(worst.uv);
                drift.triangles += 1;
            }
        }
    }
    Ok(drift)
}
