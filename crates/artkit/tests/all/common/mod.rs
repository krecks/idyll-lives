//! Reading the goldens: lines of words, a float as its bits in hex (8 digits single, 16 double).
#![allow(dead_code)]

pub mod cypress;

pub fn golden(name: &str) -> String {
    let path = format!("{}/tests/golden/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

pub fn f32s(words: &[&str]) -> Vec<f32> {
    words
        .iter()
        .map(|w| f32::from_bits(u32::from_str_radix(w, 16).unwrap()))
        .collect()
}

pub fn f64s(words: &[&str]) -> Vec<f64> {
    words
        .iter()
        .map(|w| f64::from_bits(u64::from_str_radix(w, 16).unwrap()))
        .collect()
}

pub fn f32w(w: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(w, 16).unwrap())
}

pub fn f64w(w: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(w, 16).unwrap())
}

pub fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// How many units in the last place two singles are apart (0: the same bits; -0 and 0 are one apart).
pub fn ulps(a: f32, b: f32) -> u64 {
    let key = |x: f32| {
        let b = x.to_bits() as i64;
        if b < 0 { -(b & 0x7fff_ffff) - 1 } else { b }
    };
    (key(a) - key(b)).unsigned_abs()
}

/// A mesh as the goldens write one: its points, its faces' corners, its triangles, and the other lines by name.
#[derive(Default, Clone)]
pub struct GoldenMesh {
    /// The comment line before it: what was built.
    pub what: String,
    pub name: String,
    pub verts: Vec<[f32; 3]>,
    pub faces: Vec<Vec<u32>>,
    pub tris: Vec<[u32; 3]>,
    pub lines: std::collections::BTreeMap<String, String>,
}

impl GoldenMesh {
    pub fn f32s(&self, key: &str) -> Vec<f32> {
        self.lines
            .get(key)
            .map(|l| f32s(&l.split(' ').filter(|w| !w.is_empty()).collect::<Vec<_>>()))
            .unwrap_or_default()
    }
}

/// The meshes of a golden file, in order.
pub fn golden_meshes(name: &str) -> Vec<GoldenMesh> {
    let mut out: Vec<GoldenMesh> = Vec::new();
    let mut what = String::new();
    let mut before: std::collections::BTreeMap<String, String> = Default::default();
    for line in golden(name).lines() {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "#" => {
                what = rest.to_string();
                before.clear();
            }
            "mesh" => {
                let mut m = GoldenMesh {
                    what: what.clone(),
                    name: rest.split(' ').next().unwrap().to_string(),
                    ..Default::default()
                };
                m.lines = std::mem::take(&mut before);
                out.push(m);
            }
            _ if out.last().is_none_or(|m| m.what != what) => {
                before.insert(key.to_string(), rest.to_string());
            }
            "v" => {
                let w: Vec<&str> = rest.split(' ').filter(|w| !w.is_empty()).collect();
                out.last_mut().unwrap().verts =
                    f32s(&w).chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
            }
            "f" => {
                out.last_mut().unwrap().faces = rest
                    .split(' ')
                    .filter(|w| !w.is_empty())
                    .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
                    .collect();
            }
            "t" => {
                let w: Vec<u32> = rest
                    .split(' ')
                    .filter(|w| !w.is_empty())
                    .map(|i| i.parse().unwrap())
                    .collect();
                out.last_mut().unwrap().tris = w.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
            }
            _ => {
                out.last_mut()
                    .unwrap()
                    .lines
                    .insert(key.to_string(), rest.to_string());
            }
        }
    }
    out
}

/// Faces or triangles as a set: each turned to start at its lowest vertex, then sorted.
pub fn as_set<T: AsRef<[u32]>>(faces: &[T]) -> Vec<Vec<u32>> {
    let mut out: Vec<Vec<u32>> = faces
        .iter()
        .map(|f| {
            let f = f.as_ref();
            let at = (0..f.len()).min_by_key(|&i| f[i]).unwrap_or(0);
            f[at..].iter().chain(&f[..at]).copied().collect()
        })
        .collect();
    out.sort();
    out
}

/// Faces as they are (the corner each starts at kept), sorted.
pub fn sorted<T: AsRef<[u32]>>(faces: &[T]) -> Vec<Vec<u32>> {
    let mut out: Vec<Vec<u32>> = faces.iter().map(|f| f.as_ref().to_vec()).collect();
    out.sort();
    out
}

/// How a built mesh's points differ from a golden's: (how many differ, the farthest in units in the last place).
pub fn points_off(got: &[artkit::math::V3], want: &[[f32; 3]]) -> (usize, u64) {
    assert_eq!(got.len(), want.len(), "points");
    let (mut n, mut far) = (0, 0);
    for (g, w) in got.iter().zip(want) {
        let off = (0..3).map(|i| ulps(g[i], w[i])).max().unwrap();
        // (a zero's sign isn't a difference)
        let off = if (0..3).all(|i| g[i] == w[i]) { 0 } else { off };
        n += (off > 0) as usize;
        far = far.max(off);
    }
    (n, far)
}

/// For each of `got`'s points the golden point it is (the nearest, each taken once, none farther than `reach`),
/// with how many aren't the same to the bit and the farthest apart; None if the points don't pair up.
pub fn pair_points(
    got: &[artkit::math::V3],
    want: &[[f32; 3]],
    reach: f32,
) -> Option<(Vec<u32>, usize, f32)> {
    if got.len() != want.len() {
        return None;
    }
    let mut taken = vec![false; want.len()];
    let mut map = Vec::with_capacity(got.len());
    let (mut inexact, mut far) = (0, 0.0_f32);
    for g in got {
        let d = |w: &[f32; 3]| (0..3).map(|i| (g[i] - w[i]).abs()).fold(0.0, f32::max);
        let best = (0..want.len())
            .filter(|&j| !taken[j])
            .min_by(|&a, &b| d(&want[a]).total_cmp(&d(&want[b])))?;
        let off = d(&want[best]);
        if off > reach {
            return None;
        }
        taken[best] = true;
        map.push(best as u32);
        inexact += (off != 0.0) as usize;
        far = far.max(off);
    }
    Some((map, inexact, far))
}

/// `faces` with each vertex renamed by `map`.
pub fn renamed<T: AsRef<[u32]>>(faces: &[T], map: &[u32]) -> Vec<Vec<u32>> {
    faces
        .iter()
        .map(|f| f.as_ref().iter().map(|&v| map[v as usize]).collect())
        .collect()
}
