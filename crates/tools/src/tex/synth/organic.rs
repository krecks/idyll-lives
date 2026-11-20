//! The organic sets: lawn and field earth, leaves, bark, carpet, linen, wallpaper, painted and oiled
//! wood.
//!
//! These are built differently from the hard-surface sets. Grass blades, leaves and carpet fibres are
//! drawn one by one into a z-buffered [`Canvas`] (each sprite wins where it stands highest, with a
//! one-pixel soft edge), so blades cross, leaves overlap and gaps fall into shade the way they do in a
//! photograph. Each set keeps its own albedo tone apart from its height (a leaf's veins are lighter,
//! not higher), its own ambient occlusion (from how deep a pixel sits in the layer, not only from a
//! blurred height) and its own roughness, and [`Organic::maps`] turns them into the albedo, normal and
//! packed arm maps. The greyscale detail sets still go through [`grey_detail`], so the game's tint and
//! the target means in the `Set` table hold; sizes of sprites are given in metres via the manifest's
//! tile size, so the features have the size of the real thing in the game.
//!
//! Everything wraps: sprites are drawn with wrapping pixel indices and every noise is periodic, so the
//! maps tile exactly. A fixed-seed generator per set keeps the output deterministic.

use super::{HeightMap, Noise, Set, lerp, seed, smoothstep};
use crate::tex::grey_detail;
use crate::tex::pil::Img;
use std::f64::consts::{PI, TAU};

/// The organic sets' names (the ones [`generate`] makes).
#[cfg(test)]
const NAMES: [&str; 9] = [
    "bark",
    "carpet",
    "fabric_linen",
    "foliage",
    "grass",
    "grass_lush",
    "painted_wood",
    "wallpaper",
    "wood_oak",
];

/// What an organic generator makes: the height (for the normal map), the albedo (a colour image, or
/// the grey tone a detail set's albedo is made from) and the AO and roughness of the arm map.
pub struct Organic {
    pub height: HeightMap,
    pub albedo: Img,
    ao: Vec<f64>,
    rough: Vec<f64>,
}

impl Organic {
    /// The albedo, normal and arm maps, at the generated size. Of the set's row in `SETS` the
    /// detail-map grading (`grey`) and the normal's `relief` apply; AO and roughness are the set's own.
    pub fn maps(mut self, set: &Set) -> (Img, Img, Img) {
        self.height.stretch();
        let albedo = match set.grey {
            Some((target, contrast, flatten)) => {
                grey_detail(&self.albedo, target, contrast, flatten)
            }
            None => self.albedo,
        };
        let normal = self.height.normal(set.relief);
        let (w, h) = (self.height.w, self.height.h);
        let mut px = vec![0u8; w * h * 3];
        for i in 0..w * h {
            px[i * 3] = to_u8(self.ao[i]);
            px[i * 3 + 1] = to_u8(self.rough[i]);
        }
        (albedo, normal, Img { w, h, ch: 3, px })
    }
}

/// The organic set `name` at `size` px, or `None` for the other sets.
pub fn generate(name: &str, size: usize) -> Option<Organic> {
    let n = Noise::new(seed(name));
    let mut r = Rng(seed(name) ^ 0x6f72_6761_6e69_6321);
    Some(match name {
        "bark" => bark(&n, &mut r, size),
        "carpet" => carpet(&n, &mut r, size),
        "fabric_linen" => linen(&n, size),
        "foliage" => foliage(&n, &mut r, size),
        "grass" => earth(&n, &mut r, size),
        "grass_lush" => lawn(&n, &mut r, size),
        "painted_wood" => painted_wood(&n, &mut r, size),
        "wallpaper" => wallpaper(&n, &mut r, size),
        "wood_oak" => oak(&n, size),
        _ => return None,
    })
}

// --- organic sets: helpers -------------------------------------------------------------------------

fn to_u8(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// SplitMix64: the sprites' placements, sizes and tones.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    fn f(&mut self) -> f64 {
        (self.next() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    /// Uniform in [a, b).
    fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.f()
    }

    /// Roughly normal (mean 0, sd 1): the sum of four uniforms.
    fn gauss(&mut self) -> f64 {
        (self.f() + self.f() + self.f() + self.f() - 2.0) * 1.732
    }
}

/// One pixel of a sprite: its height, albedo tone, roughness and coverage (0..1, the soft edge).
struct Frag {
    z: f64,
    tone: f64,
    rough: f64,
    a: f64,
}

/// A z-buffered, wrapping layer of sprites: height, tone and roughness per pixel.
struct Canvas {
    w: usize,
    h: usize,
    z: Vec<f64>,
    tone: Vec<f64>,
    rough: Vec<f64>,
}

impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            z: vec![0.0; w * h],
            tone: vec![0.0; w * h],
            rough: vec![0.0; w * h],
        }
    }

    /// A ground layer from a per-pixel function of (u, v).
    fn fill(&mut self, f: impl Fn(f64, f64) -> (f64, f64, f64)) {
        for y in 0..self.h {
            let v = (y as f64 + 0.5) / self.h as f64;
            for x in 0..self.w {
                let u = (x as f64 + 0.5) / self.w as f64;
                let (z, t, r) = f(u, v);
                let i = y * self.w + x;
                self.z[i] = z;
                self.tone[i] = t;
                self.rough[i] = r;
            }
        }
    }

    /// Draws a sprite centred on pixel position (cx, cy) within `rad` px: `f` gets the offset of the
    /// pixel centre from (cx, cy) and returns the fragment there, if any. A fragment higher than what
    /// is there covers it by its coverage.
    fn sprite(&mut self, cx: f64, cy: f64, rad: f64, mut f: impl FnMut(f64, f64) -> Option<Frag>) {
        let (x0, x1) = ((cx - rad).floor() as i64, (cx + rad).ceil() as i64);
        let (y0, y1) = ((cy - rad).floor() as i64, (cy + rad).ceil() as i64);
        for y in y0..=y1 {
            let yi = y.rem_euclid(self.h as i64) as usize;
            let dy = y as f64 + 0.5 - cy;
            for x in x0..=x1 {
                let dx = x as f64 + 0.5 - cx;
                let Some(fr) = f(dx, dy) else { continue };
                if fr.a <= 0.0 {
                    continue;
                }
                let i = yi * self.w + x.rem_euclid(self.w as i64) as usize;
                if fr.z > self.z[i] {
                    let a = fr.a.min(1.0);
                    self.z[i] = lerp(self.z[i], fr.z, a);
                    self.tone[i] = lerp(self.tone[i], fr.tone, a);
                    self.rough[i] = lerp(self.rough[i], fr.rough, a);
                }
            }
        }
    }
}

impl Canvas {
    /// Adds `f(d)` to the height within `rad` px of (cx, cy), `d` the distance over `rad` (0..1).
    fn add(&mut self, cx: f64, cy: f64, rad: f64, f: impl Fn(f64) -> f64) {
        let (x0, x1) = ((cx - rad).floor() as i64, (cx + rad).ceil() as i64);
        let (y0, y1) = ((cy - rad).floor() as i64, (cy + rad).ceil() as i64);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d =
                    ((x as f64 + 0.5 - cx).powi(2) + (y as f64 + 0.5 - cy).powi(2)).sqrt() / rad;
                if d < 1.0 {
                    let i = y.rem_euclid(self.h as i64) as usize * self.w
                        + x.rem_euclid(self.w as i64) as usize;
                    self.z[i] += f(d);
                }
            }
        }
    }
}

/// A wrapping box blur, three passes (close to a gaussian of sd ~ `r`).
fn soften(v: &[f64], w: usize, h: usize, r: usize) -> Vec<f64> {
    let mut m = HeightMap {
        w,
        h,
        v: v.to_vec(),
    };
    for _ in 0..3 {
        m = m.blur(r.max(1));
    }
    m.v
}

/// Cavity occlusion: how far a pixel sits below its surroundings at a few scales (0 = open).
fn cavity(z: &[f64], w: usize, h: usize, radii: &[(usize, f64)]) -> Vec<f64> {
    let mut occ = vec![0.0; w * h];
    for &(r, k) in radii {
        let low = soften(z, w, h, r);
        for i in 0..w * h {
            occ[i] += (low[i] - z[i]).max(0.0) * k;
        }
    }
    occ
}

/// A grey image from tones in 0..1 (sRGB-encoded levels, as the detail maps are read).
fn grey_img(tone: &[f64], w: usize, h: usize) -> Img {
    Img {
        w,
        h,
        ch: 1,
        px: tone.iter().map(|&t| to_u8(t)).collect(),
    }
}

/// Periodic 1-D noise along a line (a thread, a fibre), in -1..1: `period` cycles over the tile.
fn line_noise(n: &Noise, t: f64, period: i64, line: f64, oct: u32) -> f64 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut norm = 0.0;
    let (mut f, mut p) = (period as f64, period);
    for o in 0..oct {
        sum += amp * n.perlin2(t * f, line * 1.618 + o as f64 * 17.0, p, 4096, 40 + o);
        norm += amp;
        amp *= 0.5;
        f *= 2.0;
        p *= 2;
    }
    sum / norm * 1.6
}

/// Nearest and second-nearest jittered points of a wrapping grid (`cols` x `rows` cells over the
/// tile), with the nearest cell's index, in units of a cell's width.
fn worley(
    n: &Noise,
    u: f64,
    v: f64,
    cols: i64,
    rows: i64,
    jitter: f64,
    salt: u32,
) -> (f64, f64, i64, i64) {
    let x = u * cols as f64;
    let y = v * rows as f64;
    let (cx, cy) = (x.floor() as i64, y.floor() as i64);
    let (mut d1, mut d2, mut id) = (f64::INFINITY, f64::INFINITY, (0, 0));
    let aspect = rows as f64 / cols as f64;
    for dj in -1..=1 {
        for di in -1..=1 {
            let (i, j) = (cx + di, cy + dj);
            let (wi, wj) = (i.rem_euclid(cols), j.rem_euclid(rows));
            let px = i as f64 + 0.5 + (n.rand(wi, wj, salt) - 0.5) * jitter;
            let py = j as f64 + 0.5 + (n.rand(wi, wj, salt + 1) - 0.5) * jitter;
            let d = ((x - px).powi(2) + ((y - py) / aspect).powi(2)).sqrt();
            if d < d1 {
                d2 = d1;
                d1 = d;
                id = (wi, wj);
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    (d1, d2, id.0, id.1)
}

// --- grass blades and leaves --------------------------------------------------------------------------

/// A grass blade seen from above: from its base (at the sprite's centre) along `dir`, curving
/// sideways by `bend`, tapering to a point; rising from `z0` at the base to `z1` at the tip.
#[derive(Clone, Copy)]
struct Blade {
    dir: f64,
    len: f64,
    half: f64,
    bend: f64,
    z0: f64,
    z1: f64,
    tone: f64,
    rough: f64,
}

impl Blade {
    fn frag(&self, dx: f64, dy: f64) -> Option<Frag> {
        let (c, s) = (self.dir.cos(), self.dir.sin());
        let along = dx * c + dy * s;
        let t = along / self.len;
        if !(-0.02..=1.0).contains(&t) {
            return None;
        }
        let t = t.max(0.0);
        let side = -dx * s + dy * c - self.bend * self.len * t * t;
        // Full width for most of the length, then a long point.
        let w = self.half * ((1.0 - t) * 2.2).min(1.0) * (1.0 - 0.25 * t);
        let a = (w + 0.5 - side.abs()).clamp(0.0, 1.0);
        if a <= 0.0 {
            return None;
        }
        let across = (side.abs() / w.max(0.3)).min(1.0);
        // A folded blade: the midrib a little higher than the edges.
        let z = lerp(self.z0, self.z1, t) + 0.05 * (1.0 - across * across);
        let midrib = 1.0 - smoothstep(0.0, 0.35, across);
        let tone = self.tone * (0.86 + 0.24 * t) * (1.0 + 0.10 * midrib) * (1.0 - 0.10 * across);
        Some(Frag {
            z,
            tone,
            rough: self.rough,
            a,
        })
    }
}

/// A leaf seen from above: an ovate, pointed blade about its centre, tilted (`tilt`, a slope across
/// it), folded along the midrib and drooping at the ends, with lighter veins.
#[derive(Clone, Copy)]
struct Leaf {
    dir: f64,
    len: f64,
    wid: f64,
    zc: f64,
    tilt: (f64, f64),
    fold: f64,
    tone: f64,
}

impl Leaf {
    fn frag(&self, dx: f64, dy: f64) -> Option<Frag> {
        let (c, s) = (self.dir.cos(), self.dir.sin());
        let lx = dx * c + dy * s;
        let ly = -dx * s + dy * c;
        let half = self.len * 0.5;
        let sx = lx / half;
        if !(-1.0..=1.0).contains(&sx) {
            return None;
        }
        // Ovate outline: widest a little below the middle, pointed at the tip (+x).
        let shape = (1.0 - sx).powf(0.9) * (1.0 + sx).powf(0.55) / 1.06;
        let hw = self.wid * 0.5 * shape;
        let a = (hw + 0.5 - ly.abs()).clamp(0.0, 1.0);
        if a <= 0.0 {
            return None;
        }
        let across = (ly.abs() / hw.max(0.4)).min(1.0);
        let z = self.zc + self.tilt.0 * lx + self.tilt.1 * ly + self.fold * across * self.wid
            - 0.10 * sx * sx * self.wid;
        // Veins: the midrib and pairs of side veins running out towards the tip.
        let midrib = 1.0 - smoothstep(0.0, 0.7, ly.abs());
        let vp = (lx - ly.abs() * 0.9) / (self.len * 0.14);
        let side = (1.0 - smoothstep(0.0, 0.16, (vp - vp.round()).abs())) * (1.0 - across) * 0.6;
        let edge = smoothstep(0.65, 1.0, across);
        let tone = self.tone * (1.0 + 0.16 * midrib + 0.07 * side - 0.10 * edge);
        Some(Frag {
            z,
            tone,
            rough: 0.62 + 0.18 * edge - 0.08 * midrib,
            a,
        })
    }
}

mod sets;
use sets::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn organic_sets_are_known_sets() {
        for name in NAMES {
            assert!(super::super::SETS.iter().any(|s| s.name == name), "{name}");
            assert!(generate(name, 64).is_some(), "{name}");
        }
        assert!(generate("brick", 64).is_none());
    }

    #[test]
    fn organic_maps_tile() {
        // The wrap seam of every map is no larger than its busiest interior step.
        for name in NAMES {
            let set = super::super::SETS.iter().find(|s| s.name == name).unwrap();
            let (albedo, normal, arm) = generate(name, 128).unwrap().maps(set);
            for img in [&albedo, &normal, &arm] {
                let (w, h, ch) = (img.w, img.h, img.ch);
                let at = |x: usize, y: usize, c: usize| img.px[(y * w + x) * ch + c] as i32;
                let (mut seam, mut inner) = (0, 0);
                for y in 0..h {
                    for c in 0..ch {
                        seam = seam.max((at(0, y, c) - at(w - 1, y, c)).abs());
                        for x in 1..w {
                            inner = inner.max((at(x, y, c) - at(x - 1, y, c)).abs());
                        }
                    }
                }
                for x in 0..w {
                    for c in 0..ch {
                        seam = seam.max((at(x, 0, c) - at(x, h - 1, c)).abs());
                    }
                }
                assert!(seam <= inner + 1, "{name}: seam {seam} > interior {inner}");
            }
        }
    }
}
