use super::*;

/// A node at rest: where its origin is in its parent, how it is turned (x, y, z, w) and scaled.
#[derive(Clone, Copy)]
pub struct Rest {
    pub translation: P3,
    pub rotation: [f64; 4],
    pub scale: P3,
}

fn numbers<const N: usize>(v: Option<&J>, default: [f64; N]) -> R<[f64; N]> {
    let Some(v) = v else { return Ok(default) };
    let list: Vec<f64> = v
        .list()
        .unwrap_or(&[])
        .iter()
        .filter_map(|x| x.num().map(|n| n.f()))
        .collect();
    list.try_into()
        .map_err(|_| format!("a node's transform is no list of {N} numbers"))
}

/// Every node's rest (a `matrix` taken apart into the three).
pub fn rests(gltf: &J) -> R<Vec<Rest>> {
    let mut out = Vec::new();
    for n in gltf.get("nodes").and_then(J::list).unwrap_or(&[]) {
        if let Some(m) = n.get("matrix") {
            let m: [f64; 16] = numbers(Some(m), [0.0; 16])?;
            let col = |c: usize| [m[c * 4], m[c * 4 + 1], m[c * 4 + 2]];
            let scale = [length(col(0)), length(col(1)), length(col(2))];
            let r = [unit(col(0)), unit(col(1)), unit(col(2))];
            // (Shepperd's method, from the larger of the four.)
            let trace = r[0][0] + r[1][1] + r[2][2];
            let rotation = if trace > 0.0 {
                let s = (trace + 1.0).sqrt() * 2.0;
                [
                    (r[1][2] - r[2][1]) / s,
                    (r[2][0] - r[0][2]) / s,
                    (r[0][1] - r[1][0]) / s,
                    s / 4.0,
                ]
            } else if r[0][0] > r[1][1] && r[0][0] > r[2][2] {
                let s = (1.0 + r[0][0] - r[1][1] - r[2][2]).sqrt() * 2.0;
                [
                    s / 4.0,
                    (r[1][0] + r[0][1]) / s,
                    (r[2][0] + r[0][2]) / s,
                    (r[1][2] - r[2][1]) / s,
                ]
            } else if r[1][1] > r[2][2] {
                let s = (1.0 + r[1][1] - r[0][0] - r[2][2]).sqrt() * 2.0;
                [
                    (r[1][0] + r[0][1]) / s,
                    s / 4.0,
                    (r[2][1] + r[1][2]) / s,
                    (r[2][0] - r[0][2]) / s,
                ]
            } else {
                let s = (1.0 + r[2][2] - r[0][0] - r[1][1]).sqrt() * 2.0;
                [
                    (r[2][0] + r[0][2]) / s,
                    (r[2][1] + r[1][2]) / s,
                    s / 4.0,
                    (r[0][1] - r[1][0]) / s,
                ]
            };
            out.push(Rest {
                translation: [m[12], m[13], m[14]],
                rotation,
                scale,
            });
        } else {
            out.push(Rest {
                translation: numbers(n.get("translation"), [0.0; 3])?,
                rotation: numbers(n.get("rotation"), [0.0, 0.0, 0.0, 1.0])?,
                scale: numbers(n.get("scale"), [1.0; 3])?,
            });
        }
    }
    Ok(out)
}

/// Every node's place in the model at rest, as a column-major 4x4 matrix.
pub fn worlds(tree: &Tree, rests: &[Rest]) -> Vec<[f64; 16]> {
    let local = |r: &Rest| -> [f64; 16] {
        let [x, y, z, w] = r.rotation;
        let rot = [
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y + z * w),
                2.0 * (x * z - y * w),
            ],
            [
                2.0 * (x * y - z * w),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z + x * w),
            ],
            [
                2.0 * (x * z + y * w),
                2.0 * (y * z - x * w),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ];
        let mut m = [0.0; 16];
        for c in 0..3 {
            for k in 0..3 {
                m[c * 4 + k] = rot[c][k] * r.scale[c];
            }
            m[12 + c] = r.translation[c];
        }
        m[15] = 1.0;
        m
    };
    let mul = |a: &[f64; 16], b: &[f64; 16]| -> [f64; 16] {
        std::array::from_fn(|i| {
            let (c, r) = (i / 4, i % 4);
            (0..4).map(|k| a[k * 4 + r] * b[c * 4 + k]).sum()
        })
    };
    (0..rests.len())
        .map(|i| {
            let mut m = local(&rests[i]);
            let mut k = i;
            while let Some(p) = tree.parents[k] {
                m = mul(&local(&rests[p]), &m);
                k = p;
            }
            m
        })
        .collect()
}

/// The angle between two rotations (x, y, z, w), in degrees.
pub fn turn(a: [f64; 4], b: [f64; 4]) -> f64 {
    let norm = |q: [f64; 4]| q.iter().map(|v| v * v).sum::<f64>().sqrt();
    let (la, lb) = (norm(a), norm(b));
    if la == 0.0 || lb == 0.0 {
        return 0.0;
    }
    // (q and -q are one rotation.)
    let d = (a.iter().zip(&b).map(|(x, y)| x * y).sum::<f64>() / (la * lb)).abs();
    (2.0 * d.min(1.0).acos()).to_degrees()
}

/// One node's keys for its place, its turn or its size in a clip.
pub struct Track {
    pub times: Vec<f64>,
    pub keys: Vec<Vec<f64>>,
    /// The value holds until the next key (STEP).
    pub step: bool,
}

impl Track {
    /// The value at `t` seconds, as the glTF loader reads it: held before the first key and after the last, and
    /// between two keys a straight line, or for a turn the arc between them. None for a track without keys.
    pub fn at(&self, t: f64, rotation: bool) -> Option<Vec<f64>> {
        let n = self.times.len().min(self.keys.len());
        if n == 0 {
            return None;
        }
        if t <= self.times[0] {
            return Some(self.keys[0].clone());
        }
        if t >= self.times[n - 1] {
            return Some(self.keys[n - 1].clone());
        }
        let i = self.times[..n].partition_point(|x| *x <= t) - 1;
        let (a, b) = (&self.keys[i], &self.keys[i + 1]);
        let span = self.times[i + 1] - self.times[i];
        if self.step || span <= 0.0 || a.len() != b.len() {
            return Some(a.clone());
        }
        let u = (t - self.times[i]) / span;
        if !rotation {
            return Some(a.iter().zip(b).map(|(x, y)| x + (y - x) * u).collect());
        }
        let mut d: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let sign = if d < 0.0 { -1.0 } else { 1.0 };
        d = d.abs().min(1.0);
        let theta = d.acos();
        let (wa, wb) = if theta < 1e-6 {
            (1.0 - u, u)
        } else {
            (
                ((1.0 - u) * theta).sin() / theta.sin(),
                (u * theta).sin() / theta.sin(),
            )
        };
        Some(
            a.iter()
                .zip(b)
                .map(|(x, y)| x * wa + sign * y * wb)
                .collect(),
        )
    }
}

/// A clip: its name, when its first and last keys are (s), and its tracks by the node's path and what of it moves
/// (`translation`, `rotation`, `scale`).
pub struct Clip {
    pub name: String,
    pub start: f64,
    pub end: f64,
    pub tracks: BTreeMap<(String, String), Track>,
}

/// The file's clips, in its order.
pub fn clips(g: &Glb, tree: &Tree) -> R<Vec<Clip>> {
    let mut out = Vec::new();
    for (i, a) in g
        .gltf
        .get("animations")
        .and_then(J::list)
        .unwrap_or(&[])
        .iter()
        .enumerate()
    {
        let mut clip = Clip {
            name: match a.get("name") {
                Some(name) => name.key(),
                None => format!("animation{i}"),
            },
            start: f64::INFINITY,
            end: f64::NEG_INFINITY,
            tracks: BTreeMap::new(),
        };
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let node = match target.get("node") {
                Some(J::Int(i)) if *i >= 0 && (*i as usize) < tree.paths.len() => {
                    tree.paths[*i as usize].clone()
                }
                Some(_) => return Err("a clip moves a node the file has not".into()),
                None => continue,
            };
            let sampler = super::super::glb::index(key(a, "samplers")?, key(c, "sampler")?)?;
            let times: Vec<f64> = values(g, key(sampler, "input")?)?
                .iter()
                .filter_map(|t| t.first().copied())
                .collect();
            let mut keys = values(g, key(sampler, "output")?)?;
            let kind = sampler
                .get("interpolation")
                .and_then(J::str)
                .unwrap_or("LINEAR");
            if kind == "CUBICSPLINE" {
                // (In-tangent, value, out-tangent: the game plays the values as straight lines.)
                keys = keys.into_iter().skip(1).step_by(3).collect();
            }
            for t in &times {
                clip.start = clip.start.min(*t);
                clip.end = clip.end.max(*t);
            }
            clip.tracks.insert(
                (node, target.get("path").map(J::key).unwrap_or_default()),
                Track {
                    times,
                    keys,
                    step: kind == "STEP",
                },
            );
        }
        if clip.start > clip.end {
            (clip.start, clip.end) = (0.0, 0.0);
        }
        out.push(clip);
    }
    Ok(out)
}
