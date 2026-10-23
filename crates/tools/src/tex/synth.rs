//! `cargo tex synth`: the procedural textures.
//!
//! Every map is made from a tileable height field: a periodic gradient-noise lattice (both axes wrap),
//! brick, stone, plank and shingle courses, a hex lattice, a woven grid and a field of grass blades or
//! leaves. The height becomes the normal map (Sobel, wrapping) and the packed arm map (AO from a
//! blurred height, roughness from height and edges); the albedo is either the colour the pattern gives
//! or, for the greyscale "detail" sets, the height run through [`super::grey_detail`], so the game's
//! tint sets the hue and the mean lands where the old maps did.
//!
//! The output keeps the filenames and sizes the game expects: the 19 texture sets under
//! `textures/<set>/`, the sidewalk under `street/sidewalk/`. Everything is original and CC0; there are
//! no downloads and no Python.

use super::grey_detail;
use super::pil::{Filter, Img};
use crate::content;
use std::f64::consts::TAU;
use std::path::Path;

// --- periodic noise -------------------------------------------------------------------------------

/// A seeded, periodic gradient-noise lattice. Every coordinate is wrapped modulo its integer period,
/// so a field evaluated over `0..1` on both axes tiles exactly.
#[derive(Clone, Copy)]
pub struct Noise {
    seed: u64,
}

const GRAD: [(f64, f64); 8] = [
    (1.0, 0.0),
    (-1.0, 0.0),
    (0.0, 1.0),
    (0.0, -1.0),
    (0.707_106_781_186_547_6, 0.707_106_781_186_547_6),
    (-0.707_106_781_186_547_6, 0.707_106_781_186_547_6),
    (0.707_106_781_186_547_6, -0.707_106_781_186_547_6),
    (-0.707_106_781_186_547_6, -0.707_106_781_186_547_6),
];

impl Noise {
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// SplitMix64's finaliser over the lattice point and octave.
    fn hash(&self, x: i64, y: i64, oct: u32) -> u64 {
        let mut h = self.seed
            ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
            ^ (oct as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
        h ^= h >> 30;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= h >> 27;
        h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^ (h >> 31)
    }

    /// A uniform draw in [0, 1) for a lattice cell.
    pub fn rand(&self, x: i64, y: i64, oct: u32) -> f64 {
        (self.hash(x, y, oct) >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    fn grad(h: u64, x: f64, y: f64) -> f64 {
        let (gx, gy) = GRAD[(h & 7) as usize];
        gx * x + gy * y
    }

    /// Perlin noise with independent x and y periods. Returns roughly -1..1.
    pub fn perlin2(&self, x: f64, y: f64, px: i64, py: i64, oct: u32) -> f64 {
        let xi = x.floor();
        let yi = y.floor();
        let xf = x - xi;
        let yf = y - yi;
        let (xi, yi) = (xi as i64, yi as i64);
        let x0 = xi.rem_euclid(px);
        let y0 = yi.rem_euclid(py);
        let x1 = (x0 + 1).rem_euclid(px);
        let y1 = (y0 + 1).rem_euclid(py);
        let n00 = Self::grad(self.hash(x0, y0, oct), xf, yf);
        let n10 = Self::grad(self.hash(x1, y0, oct), xf - 1.0, yf);
        let n01 = Self::grad(self.hash(x0, y1, oct), xf, yf - 1.0);
        let n11 = Self::grad(self.hash(x1, y1, oct), xf - 1.0, yf - 1.0);
        let u = fade(xf);
        let v = fade(yf);
        lerp(lerp(n00, n10, u), lerp(n01, n11, u), v)
    }

    /// Fractal noise with independent x and y base frequencies, in 0..1.
    pub fn fbm2(&self, u: f64, v: f64, bx: i64, by: i64, oct: u32) -> f64 {
        let (mut fx, mut fy) = (bx as f64, by as f64);
        let (mut px, mut py) = (bx, by);
        let mut sum = 0.0;
        let mut amp = 1.0;
        let mut norm = 0.0;
        for o in 0..oct {
            sum += amp * self.perlin2(u * fx, v * fy, px, py, o);
            norm += amp;
            amp *= 0.5;
            fx *= 2.0;
            fy *= 2.0;
            px *= 2;
            py *= 2;
        }
        0.5 + 0.5 * sum / norm
    }

    /// Isotropic fractal noise in 0..1.
    pub fn fbm(&self, u: f64, v: f64, base: i64, oct: u32) -> f64 {
        self.fbm2(u, v, base, base, oct)
    }

    /// Domain warp: offsets in UV, periodic on both axes.
    pub fn warp(&self, u: f64, v: f64, amp: f64, base: i64) -> (f64, f64) {
        let dx = self.fbm2(u, v, base, base, 3) - 0.5;
        let dy = self.fbm2(u + 0.37, v + 0.71, base, base, 3) - 0.5;
        (
            (u + dx * amp).rem_euclid(1.0),
            (v + dy * amp).rem_euclid(1.0),
        )
    }
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// --- height fields --------------------------------------------------------------------------------

/// A float height field in 0..1, sampled on a wrapping grid.
#[derive(Clone)]
pub struct HeightMap {
    pub w: usize,
    pub h: usize,
    pub v: Vec<f64>,
}

impl HeightMap {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            v: vec![0.0; w * h],
        }
    }

    /// The sample at (x, y), wrapping on both axes.
    pub fn at(&self, x: isize, y: isize) -> f64 {
        let xi = x.rem_euclid(self.w as isize) as usize;
        let yi = y.rem_euclid(self.h as isize) as usize;
        self.v[yi * self.w + xi]
    }

    /// Stretch the field to the full 0..1 range, so each pattern uses the same normal strength.
    pub fn stretch(&mut self) {
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for &v in &self.v {
            lo = lo.min(v);
            hi = hi.max(v);
        }
        let span = hi - lo;
        if span < 1e-12 {
            self.v.fill(0.5);
        } else {
            for v in &mut self.v {
                *v = (*v - lo) / span;
            }
        }
    }

    /// A tangent-space normal map in the OpenGL convention (+Y up), from wrapping Sobel differences.
    /// `relief` is the height of a full 0..1 swing in UV units (so 0.02 is a 2 cm bump on a 1 m tile).
    pub fn normal(&self, relief: f64) -> Img {
        let mut px = vec![0u8; self.w * self.h * 3];
        for y in 0..self.h {
            for x in 0..self.w {
                let (x, y) = (x as isize, y as isize);
                let du = (self.at(x + 1, y) - self.at(x - 1, y)) * 0.5 * self.w as f64;
                let dv = (self.at(x, y + 1) - self.at(x, y - 1)) * 0.5 * self.h as f64;
                // GL maps: red = -du, green = +dv (rows run down the image, +Y runs up).
                let nx = -du * relief;
                let ny = dv * relief;
                let inv = 1.0 / (nx * nx + ny * ny + 1.0).sqrt();
                let i = (y as usize * self.w + x as usize) * 3;
                px[i] = to_u8(nx * inv * 0.5 + 0.5);
                px[i + 1] = to_u8(ny * inv * 0.5 + 0.5);
                px[i + 2] = to_u8(inv * 0.5 + 0.5);
            }
        }
        Img {
            w: self.w,
            h: self.h,
            ch: 3,
            px,
        }
    }

    /// The packed arm map: R = AO from a blurred height, G = roughness from height and edges, B = 0.
    pub fn arm(&self, rough: f64, rough_var: f64, ao_gain: f64, ao_base: f64) -> Img {
        let radius = (self.w / 16).max(2);
        let low = self.blur(radius);
        let mut px = vec![0u8; self.w * self.h * 3];
        for y in 0..self.h {
            for x in 0..self.w {
                let (xi, yi) = (x as isize, y as isize);
                let h = self.v[y * self.w + x];
                let edge = (self.at(xi + 1, yi) - self.at(xi - 1, yi)).abs()
                    + (self.at(xi, yi + 1) - self.at(xi, yi - 1)).abs();
                let ao = (ao_base - ((low.v[y * self.w + x] - h).max(0.0) + edge * 0.12) * ao_gain)
                    .clamp(0.05, 1.0);
                let g = (rough + (0.5 - h) * rough_var + edge * 0.12).clamp(0.04, 0.99);
                let i = (y * self.w + x) * 3;
                px[i] = to_u8(ao);
                px[i + 1] = to_u8(g);
                px[i + 2] = 0;
            }
        }
        Img {
            w: self.w,
            h: self.h,
            ch: 3,
            px,
        }
    }

    /// A wrapping box blur of the height, for ambient occlusion.
    fn blur(&self, radius: usize) -> HeightMap {
        let (w, h) = (self.w, self.h);
        let r = radius as isize;
        let n = (2 * radius + 1) as f64;
        let mut tmp = vec![0.0f64; w * h];
        for y in 0..h as isize {
            let mut acc: f64 = (-r..=r).map(|dx| self.at(dx, y)).sum();
            for x in 0..w as isize {
                tmp[y as usize * w + x as usize] = acc / n;
                acc += self.at(x + r + 1, y) - self.at(x - r, y);
            }
        }
        let mut out = vec![0.0f64; w * h];
        for x in 0..w as isize {
            let mut acc: f64 = (-r..=r)
                .map(|dy| tmp[dy.rem_euclid(h as isize) as usize * w + x as usize])
                .sum();
            for y in 0..h as isize {
                out[y as usize * w + x as usize] = acc / n;
                acc += tmp[(y + r + 1).rem_euclid(h as isize) as usize * w + x as usize]
                    - tmp[(y - r).rem_euclid(h as isize) as usize * w + x as usize];
            }
        }
        HeightMap { w, h, v: out }
    }

    /// The height as an 8-bit greyscale image (the detail sets then pass it through `grey_detail`).
    pub fn grey(&self) -> Img {
        Img {
            w: self.w,
            h: self.h,
            ch: 1,
            px: self.v.iter().map(|&v| to_u8(v)).collect(),
        }
    }
}

fn to_u8(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn render(w: usize, h: usize, f: impl Fn(f64, f64) -> (f64, [f64; 3])) -> (HeightMap, Img) {
    let mut hm = HeightMap::new(w, h);
    let mut px = vec![0u8; w * h * 3];
    for y in 0..h {
        let v = (y as f64 + 0.5) / h as f64;
        for x in 0..w {
            let u = (x as f64 + 0.5) / w as f64;
            let (height, c) = f(u, v);
            hm.v[y * w + x] = height;
            let i = (y * w + x) * 3;
            px[i] = to_u8(c[0]);
            px[i + 1] = to_u8(c[1]);
            px[i + 2] = to_u8(c[2]);
        }
    }
    (hm, Img { w, h, ch: 3, px })
}

// --- colour ---------------------------------------------------------------------------------------

fn rgb(r: u8, g: u8, b: u8) -> [f64; 3] {
    [r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0]
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

fn mul(c: [f64; 3], k: f64) -> [f64; 3] {
    [c[0] * k, c[1] * k, c[2] * k]
}

// --- pattern primitives ---------------------------------------------------------------------------

struct Cell {
    d1: f64,
    d2: f64,
    rnd: f64,
}

/// The nearest two centres of a hex lattice, in scaled coordinates. The tile is a slightly stretched
/// hexagon grid (12 columns, 14 rows) so it fits a square exactly and wraps on both axes.
fn hex_cell(n: &Noise, u: f64, v: f64, cols: i64, rows: i64, seed: u32) -> Cell {
    let x = u * cols as f64;
    let y = v * rows as f64;
    let cj = y.floor() as i64;
    let (mut d1, mut d2) = (f64::INFINITY, f64::INFINITY);
    let mut rnd = 0.0;
    for dj in -1..=1 {
        let j = cj + dj;
        let off = if j.rem_euclid(2) == 0 { 0.0 } else { 0.5 };
        let ci = (x - off).floor() as i64;
        for di in -1..=1 {
            let i = ci + di;
            let wi = i.rem_euclid(cols);
            let wj = j.rem_euclid(rows);
            let cx = i as f64 + 0.5 + off + (n.rand(wi, wj, seed) - 0.5) * 0.08;
            let cy = j as f64 + 0.5 + (n.rand(wi, wj, seed + 1) - 0.5) * 0.08;
            let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
            if d < d1 {
                d2 = d1;
                d1 = d;
                rnd = n.rand(wi, wj, seed + 2);
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    Cell { d1, d2, rnd }
}

/// The nearest plank in a row of staggered boards: its row, its segment, the local position and the
/// distance to the nearest joint.
struct Plank {
    row: i64,
    seg: i64,
    fx: f64,
    fy: f64,
    joint: f64,
}

fn plank_at(n: &Noise, u: f64, v: f64, rows: i64, cuts: i64, seed: u32) -> Plank {
    let yy = v * rows as f64;
    let row = yy.floor() as i64;
    let fy = yy - row as f64;
    let mut cs = vec![0.0, 1.0];
    for k in 0..cuts {
        cs.push(0.12 + 0.76 * n.rand(row, k, seed));
    }
    cs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let uu = u.rem_euclid(1.0);
    let mut seg = 0;
    for (i, &c) in cs.iter().enumerate().take(cs.len() - 1) {
        if uu >= c {
            seg = i as i64;
        }
    }
    let mut joint = uu.min(1.0 - uu);
    for &c in &cs[1..cs.len() - 1] {
        joint = joint.min((uu - c).abs());
    }
    Plank {
        row,
        seg,
        fx: uu,
        fy,
        joint,
    }
}

/// Long wood grain: rings across the board, fine fibres along it.
fn wood_grain(n: &Noise, u: f64, v: f64, freq: i64) -> f64 {
    let (wu, wv) = n.warp(u, v, 0.16, 4);
    let fine = n.fbm2(wu, wv, 2, freq, 5);
    let ring = 0.5 + 0.5 * ((wv * freq as f64 + fine * 3.0) * TAU).sin();
    let fibre = n.fbm2(wu, wv, 64, 8, 3);
    (0.55 * ring + 0.45 * fibre).clamp(0.0, 1.0)
}

/// A field of grass blades: one per jittered cell, mostly upright, with a soft taper.
fn blade_field(n: &Noise, u: f64, v: f64, cols: i64, rows: i64, seed: u32) -> (f64, f64) {
    let ci = (u * cols as f64).floor() as i64;
    let cj = (v * rows as f64).floor() as i64;
    let (mut best, mut tone) = (0.0, 0.0);
    for dj in -1..=1 {
        for di in -1..=1 {
            let (i, j) = (ci + di, cj + dj);
            let wi = i.rem_euclid(cols);
            let wj = j.rem_euclid(rows);
            let cx = (i as f64 + 0.5 + (n.rand(wi, wj, seed) - 0.5) * 0.9) / cols as f64;
            let cy = (j as f64 + 0.5 + (n.rand(wi, wj, seed + 1) - 0.5) * 0.9) / rows as f64;
            let ang = std::f64::consts::FRAC_PI_2 + (n.rand(wi, wj, seed + 2) - 0.5) * 1.1;
            let len = (1.8 + 1.0 * n.rand(wi, wj, seed + 3)) / cols as f64;
            let wid = (0.12 + 0.08 * n.rand(wi, wj, seed + 4)) / cols as f64;
            let dx = (u - cx + 0.5).rem_euclid(1.0) - 0.5;
            let dy = (v - cy + 0.5).rem_euclid(1.0) - 0.5;
            let (ca, sa) = (ang.cos(), ang.sin());
            let lx = dx * ca + dy * sa;
            let ly = -dx * sa + dy * ca;
            let t = lx.clamp(-len * 0.5, len * 0.5);
            let d = ((lx - t).powi(2) + ly * ly).sqrt();
            let along = (t / len + 0.5).clamp(0.0, 1.0);
            let cov = (1.0 - smoothstep(0.0, wid, d))
                * (1.0 - (2.0 * t / len).abs()).clamp(0.0, 1.0)
                * (0.35 + 0.65 * along);
            if cov > best {
                best = cov;
                tone = n.rand(wi, wj, seed + 5);
            }
        }
    }
    (best, tone)
}

/// A field of overlapping leaves: ellipses with a central vein, gaps showing through.
fn leaf_field(n: &Noise, u: f64, v: f64, cols: i64, rows: i64, seed: u32) -> (f64, f64) {
    let ci = (u * cols as f64).floor() as i64;
    let cj = (v * rows as f64).floor() as i64;
    let (mut best, mut tone) = (0.0, 0.0);
    for dj in -1..=1 {
        for di in -1..=1 {
            let (i, j) = (ci + di, cj + dj);
            let wi = i.rem_euclid(cols);
            let wj = j.rem_euclid(rows);
            let cx = (i as f64 + 0.5 + (n.rand(wi, wj, seed) - 0.5) * 0.9) / cols as f64;
            let cy = (j as f64 + 0.5 + (n.rand(wi, wj, seed + 1) - 0.5) * 0.9) / rows as f64;
            let ang = n.rand(wi, wj, seed + 2) * TAU;
            let len = (1.6 + 0.8 * n.rand(wi, wj, seed + 3)) / cols as f64;
            let wid = len * (0.24 + 0.10 * n.rand(wi, wj, seed + 4));
            let dx = u - cx;
            let dy = v - cy;
            let (ca, sa) = (ang.cos(), ang.sin());
            let lx = dx * ca + dy * sa;
            let ly = -dx * sa + dy * ca;
            let e = (lx / (len * 0.5)).abs().powf(1.25) + (ly / wid).powi(2);
            let cov = 1.0 - smoothstep(0.55, 1.0, e);
            let vein = (1.0 - smoothstep(0.0, 0.22, (ly / wid).abs())) * cov;
            let h = cov * (0.72 + 0.28 * vein);
            if h > best {
                best = h;
                tone = n.rand(wi, wj, seed + 5);
            }
        }
    }
    (best, tone)
}

// --- the sets -------------------------------------------------------------------------------------

/// Sizes, greyscale handling and material response of each set.
#[derive(Clone, Copy)]
pub struct Set {
    pub name: &'static str,
    pub albedo: usize,
    pub normal: usize,
    pub arm: usize,
    /// `Some((linear mean, contrast, flatten radius))` for a greyscale detail map.
    pub grey: Option<(f64, f64, f32)>,
    /// Height of a full 0..1 swing, in UV units (metres, for the manifest's tile sizes).
    pub relief: f64,
    /// (base roughness, height variation, AO gain).
    pub rough: (f64, f64, f64),
    /// Ambient-occlusion base: 1 for most sets, lower where the surface sits in shade.
    pub ao_base: f64,
}

const fn set(
    name: &'static str,
    albedo: usize,
    normal: usize,
    grey: Option<(f64, f64, f32)>,
    relief: f64,
    rough: (f64, f64, f64),
) -> Set {
    set_ao(name, albedo, normal, grey, relief, rough, 1.0)
}

const fn set_ao(
    name: &'static str,
    albedo: usize,
    normal: usize,
    grey: Option<(f64, f64, f32)>,
    relief: f64,
    rough: (f64, f64, f64),
    ao_base: f64,
) -> Set {
    Set {
        name,
        albedo,
        normal,
        arm: 512,
        grey,
        relief,
        rough,
        ao_base,
    }
}

pub const SETS: [Set; 20] = [
    set("bark", 512, 512, None, 0.0058, (0.84, 0.20, 2.3)),
    set_ao("brick", 1024, 1024, None, 0.0041, (0.81, 0.12, 4.5), 0.945),
    set(
        "carpet",
        512,
        512,
        Some((0.58, 1.40, 48.0)),
        0.0052,
        (0.87, 0.30, 5.0),
    ),
    set(
        "fabric_linen",
        512,
        512,
        Some((0.80, 0.04, 0.0)),
        0.0006,
        (0.60, 0.10, 1.5),
    ),
    set(
        "floor_laminate",
        1024,
        1024,
        None,
        0.0012,
        (0.40, 0.05, 0.1),
    ),
    set("floor_tile", 1024, 1024, None, 0.0006, (0.57, 0.15, 0.05)),
    set("floor_wood", 1024, 1024, None, 0.0014, (0.62, 0.05, 0.1)),
    set(
        "foliage",
        512,
        512,
        Some((0.58, 1.10, 0.0)),
        0.008,
        (0.74, 0.20, 4.0),
    ),
    set(
        "grass",
        1024,
        1024,
        Some((0.75, 0.30, 0.0)),
        0.0019,
        (0.45, 0.15, 1.6),
    ),
    set(
        "grass_lush",
        1024,
        1024,
        Some((0.70, 0.70, 24.0)),
        0.0020,
        (0.74, 0.10, 1.7),
    ),
    set(
        "painted_wood",
        512,
        512,
        Some((0.75, 0.04, 0.0)),
        0.0007,
        (0.39, 0.05, 0.4),
    ),
    set(
        "path_hexpaving",
        1024,
        1024,
        None,
        0.0014,
        (0.90, 0.12, 0.2),
    ),
    set("road_asphalt", 1024, 1024, None, 0.0026, (0.76, 0.15, 4.0)),
    set_ao(
        "roof_shingles",
        1024,
        1024,
        Some((0.60, 0.36, 0.0)),
        0.0043,
        (0.61, 0.20, 20.0),
        0.845,
    ),
    set(
        "siding_wood",
        1024,
        1024,
        Some((0.68, 0.14, 0.0)),
        0.0050,
        (0.76, 0.10, 0.3),
    ),
    set(
        "stone_foundation",
        1024,
        1024,
        None,
        0.0104,
        (0.75, 0.15, 2.8),
    ),
    set(
        "wall_plaster",
        1024,
        512,
        Some((0.80, 0.12, 0.0)),
        0.0034,
        (0.91, 0.03, 0.6),
    ),
    set(
        "wallpaper",
        1024,
        512,
        Some((0.75, 0.05, 0.0)),
        0.0025,
        (0.78, 0.05, 0.05),
    ),
    set(
        "wood_oak",
        512,
        512,
        Some((0.80, 0.11, 0.0)),
        0.0003,
        (0.52, 0.20, 0.2),
    ),
    set_ao(
        "sidewalk",
        512,
        512,
        Some((0.73, 0.30, 0.0)),
        0.003,
        (0.86, 0.03, 0.3),
        0.82,
    ),
];

/// `cargo tex synth [set...]`.
pub fn run(only: &[&str]) -> Result<(), String> {
    for name in only {
        if *name != "sky" && !SETS.iter().any(|s| s.name == *name) {
            return Err(format!("unknown texture set: {name}"));
        }
    }
    let root = content::root().join("web/public/assets/textures");
    for set in SETS {
        if !only.is_empty() && !only.contains(&set.name) {
            continue;
        }
        write_set(&root, &set)?;
    }
    if only.contains(&"sky") {
        println!("sky: textures/sky/clouds.png is `cargo tex clouds`, unchanged");
    }
    Ok(())
}

fn write_set(root: &Path, set: &Set) -> Result<(), String> {
    let (mut height, albedo) = generate(set.name, set.albedo);
    height.stretch();
    let albedo = match set.grey {
        Some((target, contrast, flatten)) => grey_detail(&height.grey(), target, contrast, flatten),
        None => albedo,
    };
    let normal = height.normal(set.relief);
    let arm = height.arm(set.rough.0, set.rough.1, set.rough.2, set.ao_base);
    let dir = if set.name == "sidewalk" {
        root.join("../street/sidewalk")
    } else {
        root.join(set.name)
    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    albedo.save_jpeg(&dir.join("albedo.jpg"), 85, true)?;
    let normal = if set.normal == set.albedo {
        normal
    } else {
        normal.resize(set.normal, set.normal, Filter::Lanczos)
    };
    normal.save_jpeg(&dir.join("normal.jpg"), 88, true)?;
    let arm = if set.arm == set.albedo {
        arm
    } else {
        arm.resize(set.arm, set.arm, Filter::Lanczos)
    };
    arm.save_jpeg(&dir.join("arm.jpg"), 85, true)?;
    let mut bytes = 0;
    for f in ["albedo.jpg", "normal.jpg", "arm.jpg"] {
        bytes += std::fs::metadata(dir.join(f))
            .map_err(|e| e.to_string())?
            .len();
    }
    println!(
        "{:<18} {}x{}  {} KB",
        set.name,
        set.albedo,
        set.normal,
        bytes / 1024
    );
    Ok(())
}

fn seed(name: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in name.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn generate(name: &str, size: usize) -> (HeightMap, Img) {
    let n = Noise::new(seed(name));
    match name {
        "bark" => bark(&n, size),
        "brick" => brick(&n, size),
        "carpet" => carpet(&n, size),
        "fabric_linen" => fabric(&n, size),
        "floor_laminate" => planks(
            &n,
            size,
            3,
            1,
            18,
            41,
            rgb(150, 118, 90),
            rgb(172, 140, 110),
            rgb(116, 92, 68),
            false,
        ),
        "floor_tile" => floor_tile(&n, size),
        "floor_wood" => planks(
            &n,
            size,
            8,
            2,
            22,
            43,
            rgb(170, 124, 82),
            rgb(194, 148, 100),
            rgb(120, 88, 58),
            true,
        ),
        "foliage" => foliage(&n, size),
        "grass" => grass(&n, size, 192, 0.75),
        "grass_lush" => grass(&n, size, 224, 1.0),
        "painted_wood" => painted_wood(&n, size),
        "path_hexpaving" => hexpaving(&n, size),
        "road_asphalt" => asphalt(&n, size),
        "roof_shingles" => roof(&n, size),
        "siding_wood" => siding(&n, size),
        "sidewalk" => sidewalk(&n, size),
        "stone_foundation" => stone(&n, size),
        "wall_plaster" => plaster(&n, size),
        "wallpaper" => wallpaper(&n, size),
        "wood_oak" => wood_oak(&n, size),
        _ => unreachable!("unknown set {name}"),
    }
}

// --- generators -----------------------------------------------------------------------------------

fn brick(n: &Noise, size: usize) -> (HeightMap, Img) {
    let (rows, cols) = (14i64, 4i64);
    render(size, size, |u, v| {
        let yy = v * rows as f64;
        let row = yy.floor() as i64;
        let fy = yy - row as f64;
        let off = if row.rem_euclid(2) == 0 { 0.0 } else { 0.5 };
        let xx = u * cols as f64 + off;
        let col = xx.floor() as i64;
        let fx = xx - col as f64;
        let dx = fx.min(1.0 - fx);
        let dy = fy.min(1.0 - fy);
        let face = smoothstep(0.0, 0.045, dx) * smoothstep(0.0, 0.07, dy);
        let dome = 0.07 * (1.0 - (2.0 * fx - 1.0).powi(2)) * (1.0 - (2.0 * fy - 1.0).powi(2));
        let grain = n.fbm2(u, v, 48, 48, 4) - 0.5;
        let h = 0.10 + 0.90 * face + 0.05 * grain * face + dome * face;
        let wi = col.rem_euclid(cols);
        let wj = row.rem_euclid(rows);
        let base = match (n.rand(wi, wj, 11) * 5.0) as usize {
            0 => rgb(108, 80, 68),
            1 => rgb(124, 94, 78),
            2 => rgb(94, 72, 62),
            3 => rgb(116, 88, 74),
            _ => rgb(102, 78, 68),
        };
        let shade = 0.86 + 0.28 * n.rand(wi, wj, 12);
        let speck = n.fbm2(u, v, 160, 160, 2) - 0.5;
        let spot = smoothstep(0.60, 0.78, n.fbm2(u, v, 64, 64, 3));
        let brickc = mul(mul(base, shade), 1.0 + 0.44 * speck);
        let brickc = mix(brickc, rgb(58, 42, 36), spot * 0.35);
        let pits = smoothstep(0.54, 0.70, n.fbm2(u, v, 96, 96, 2));
        let brickc = mix(brickc, rgb(48, 36, 30), pits * 0.50);
        let mortar = mul(
            rgb(178, 174, 164),
            0.88 + 0.24 * (n.fbm2(u, v, 64, 64, 3) - 0.5),
        );
        let edge = 0.62 + 0.38 * face;
        (h, mix(mortar, mul(brickc, edge), face))
    })
}

fn stone(n: &Noise, size: usize) -> (HeightMap, Img) {
    let rows = 18i64;
    render(size, size, |u, v| {
        let yy = v * rows as f64;
        let row = yy.floor() as i64;
        let fy = yy - row as f64;
        let wr = row.rem_euclid(rows);
        let off = n.rand(wr, 0, 22);
        let uu = (u + off).rem_euclid(1.0);
        let nj = 4 + (n.rand(wr, 1, 24) * 4.0) as i64;
        let mut edges = [0.0f64; 9];
        let mut acc = 0.0;
        for k in 0..nj {
            acc += 0.55 + 0.90 * n.rand(wr, k + 2, 25);
            edges[(k + 1) as usize] = acc;
        }
        for e in edges.iter_mut().take(nj as usize + 1) {
            *e /= acc;
        }
        let mut seg = 0i64;
        for k in 0..nj {
            if uu >= edges[k as usize] {
                seg = k;
            }
        }
        let (left, right) = (edges[seg as usize], edges[seg as usize + 1]);
        let d = (uu - left).min(right - uu);
        let face = smoothstep(0.0, 0.010, d) * smoothstep(0.0, 0.045, fy.min(1.0 - fy));
        let r = n.rand(wr, seg + 10, 26);
        let grain = n.fbm2(u, v, 48, 48, 4) - 0.5;
        let h = 0.10 + 0.90 * face + 0.04 * grain * face + 0.05 * (r - 0.5);
        let base = match (r * 7.0) as usize {
            0 => rgb(146, 134, 116),
            1 => rgb(178, 164, 140),
            2 => rgb(120, 110, 96),
            3 => rgb(162, 148, 124),
            4 => rgb(136, 124, 108),
            5 => rgb(188, 174, 150),
            _ => rgb(154, 140, 118),
        };
        let mottle = n.fbm2(u, v, 64, 64, 3) - 0.5;
        let c = mul(base, 0.84 + 0.32 * mottle);
        let moss = smoothstep(0.62, 0.82, n.fbm2(u, v, 8, 8, 4)) * smoothstep(0.5, 0.0, face);
        let c = mix(c, rgb(96, 104, 76), moss * 0.30);
        let mortar = mul(
            rgb(118, 110, 98),
            0.90 + 0.20 * (n.fbm2(u, v, 64, 64, 3) - 0.5),
        );
        (h, mix(mortar, c, face))
    })
}

fn floor_tile(n: &Noise, size: usize) -> (HeightMap, Img) {
    let (cols, rows) = (6i64, 6i64);
    render(size, size, |u, v| {
        let x = u * cols as f64;
        let y = v * rows as f64;
        let (ix, iy) = (x.floor() as i64, y.floor() as i64);
        let (fx, fy) = (x - ix as f64, y - iy as f64);
        let d = fx.min(1.0 - fx).min(fy.min(1.0 - fy));
        let face = smoothstep(0.0, 0.028, d);
        let bevel = smoothstep(0.0, 0.07, d);
        let grain = n.fbm2(u, v, 32, 32, 4) - 0.5;
        let h = 0.12 + 0.88 * face + 0.06 * bevel + 0.03 * grain * face;
        let r = n.rand(ix.rem_euclid(cols), iy.rem_euclid(rows), 31);
        let base = mix(rgb(200, 194, 182), rgb(236, 230, 218), r);
        let mottle = n.fbm2(u, v, 8, 8, 4) - 0.5;
        let speck = smoothstep(0.60, 0.72, n.fbm2(u, v, 192, 192, 2));
        let c = mul(base, 1.00 + 0.08 * mottle + 0.03 * grain);
        let c = mix(c, rgb(128, 118, 102), speck * 0.38);
        let stain = smoothstep(0.58, 0.75, n.fbm2(u, v, 16, 16, 4));
        let c = mix(c, rgb(118, 102, 84), stain * 0.30);
        let pits = smoothstep(0.62, 0.78, n.fbm2(u, v, 64, 64, 2));
        let c = mix(c, rgb(74, 66, 56), pits * 0.60);
        let grout = mul(
            rgb(160, 156, 146),
            0.9 + 0.2 * (n.fbm2(u, v, 64, 64, 3) - 0.5),
        );
        (h, mix(grout, c, face))
    })
}

fn hexpaving(n: &Noise, size: usize) -> (HeightMap, Img) {
    let (cols, rows) = (12i64, 14i64);
    render(size, size, |u, v| {
        let c = hex_cell(n, u, v, cols, rows, 51);
        let edge = smoothstep(0.0, 0.05, c.d2 - c.d1);
        let grain = n.fbm2(u, v, 48, 48, 4) - 0.5;
        let h = 0.12 + 0.88 * edge + 0.03 * grain;
        let base = mix(rgb(160, 148, 126), rgb(182, 168, 142), c.rnd);
        let fine = n.fbm2(u, v, 128, 128, 3) - 0.5;
        let c = mul(base, 0.90 + 0.20 * n.fbm2(u, v, 16, 16, 4) + 0.08 * fine);
        let grout = rgb(82, 74, 64);
        (h, mix(grout, c, edge))
    })
}

fn planks(
    n: &Noise,
    size: usize,
    rows: i64,
    cuts: i64,
    freq: i64,
    sd: u32,
    lo: [f64; 3],
    hi: [f64; 3],
    joint: [f64; 3],
    vertical: bool,
) -> (HeightMap, Img) {
    render(size, size, |u0, v0| {
        let (u, v) = if vertical { (v0, u0) } else { (u0, v0) };
        let p = plank_at(n, u, v, rows, cuts, sd);
        let edge_y = smoothstep(0.0, 0.06, p.fy.min(1.0 - p.fy));
        let edge_x = smoothstep(0.0, 0.008, p.joint);
        let edge = edge_x * edge_y;
        let grain = wood_grain(n, p.fx + 0.17 * p.seg as f64, v, freq);
        let h = 0.12 + 0.88 * edge + 0.12 * (grain - 0.5) * edge;
        let r = n.rand(p.row.rem_euclid(rows), p.seg, sd + 1);
        let base = mix(lo, hi, r);
        let c = mul(base, 0.95 + 0.16 * grain);
        (h, mix(joint, c, edge))
    })
}

fn siding(n: &Noise, size: usize) -> (HeightMap, Img) {
    let rows = 8i64;
    render(size, size, |u, v| {
        let yy = v * rows as f64;
        let row = yy.floor() as i64;
        let fy = yy - row as f64;
        let edge_y = smoothstep(0.0, 0.08, fy.min(1.0 - fy));
        let jx = 0.15 + 0.70 * n.rand(row, 0, 71);
        let d = (u - jx).abs();
        let d = d.min(1.0 - d);
        let edge_x = smoothstep(0.0, 0.008, d);
        let grain = n.fbm2(u, v, 4, 32, 4) - 0.5;
        let h = 0.16 + 0.84 * edge_y * edge_x + 0.08 * grain * edge_y * edge_x;
        (h, [0.5; 3])
    })
}

fn roof(n: &Noise, size: usize) -> (HeightMap, Img) {
    let (cols, rows) = (8i64, 8i64);
    render(size, size, |u, v| {
        let yy = v * rows as f64;
        let row = yy.floor() as i64;
        let fy = yy - row as f64;
        let off = if row.rem_euclid(2) == 0 { 0.0 } else { 0.5 };
        let xx = u * cols as f64 + off;
        let col = xx.floor() as i64;
        let fx = xx - col as f64;
        let ex = smoothstep(0.0, 0.07, fx.min(1.0 - fx));
        let ey = smoothstep(0.0, 0.12, fy.min(1.0 - fy));
        let r = n.rand(col.rem_euclid(cols), row.rem_euclid(rows), 81);
        let grain = n.fbm2(u, v, 4, 96, 3) - 0.5;
        let h = 0.12
            + 0.88 * ex * ey
            + 0.18 * (r - 0.5)
            + 0.12 * grain
            + 0.04 * (n.fbm2(u, v, 64, 64, 3) - 0.5);
        (h, [0.5; 3])
    })
}

fn carpet(n: &Noise, size: usize) -> (HeightMap, Img) {
    render(size, size, |u, v| {
        let a = n.fbm2(u, v, 64, 64, 4) - 0.5;
        let b = n.fbm2(u, v, 8, 128, 3) - 0.5;
        let c = n.fbm2(u, v, 256, 256, 2) - 0.5;
        let d = n.fbm2(u, v, 512, 512, 1) - 0.5;
        let h = 0.5 + 0.10 * a + 0.04 * b + 0.20 * c + 0.16 * d;
        (h, [0.5; 3])
    })
}

fn fabric(n: &Noise, size: usize) -> (HeightMap, Img) {
    let threads = 96i64;
    render(size, size, |u, v| {
        let x = u * threads as f64;
        let y = v * threads as f64;
        let (ix, iy) = (x.floor() as i64, y.floor() as i64);
        let (fx, fy) = (x - ix as f64, y - iy as f64);
        let warp = 0.5 + 0.5 * (fx * TAU).cos();
        let weft = 0.5 + 0.5 * (fy * TAU).cos();
        let over = (ix + iy).rem_euclid(2) == 0;
        let (a, b) = if over { (0.18, 0.12) } else { (0.12, 0.18) };
        let fuzz = n.fbm2(u, v, 256, 256, 2) - 0.5;
        let slub = n.fbm2(u, v, 96, 6, 3) - 0.5;
        let h = 0.5 + a * warp + b * weft + 0.08 * fuzz + 0.05 * slub;
        (h, [0.5; 3])
    })
}

fn foliage(n: &Noise, size: usize) -> (HeightMap, Img) {
    render(size, size, |u, v| {
        let (leaf, tone) = leaf_field(n, u, v, 96, 96, 91);
        let (small, tone2) = leaf_field(n, u, v, 192, 192, 97);
        let cover =
            (0.60 * leaf * (0.55 + 0.45 * tone) + 0.30 * small * (0.55 + 0.45 * tone2)).min(1.0);
        let base = n.fbm2(u, v, 32, 32, 3) - 0.5;
        let shade = 0.55 + 0.45 * n.fbm2(u, v, 8, 8, 4);
        let h = (0.22 + 0.70 * cover) * shade + 0.05 * base;
        let c = mix(
            rgb(74, 74, 74),
            rgb(204, 204, 204),
            tone * 0.65 + tone2 * 0.35,
        );
        (h, c)
    })
}

fn grass(n: &Noise, size: usize, cells: i64, lush: f64) -> (HeightMap, Img) {
    render(size, size, |u, v| {
        let (blade, tone) = blade_field(n, u, v, cells, cells, 101);
        let blade = blade * (0.62 + 0.38 * tone);
        let clump = n.fbm2(u, v, 8, 8, 4) - 0.5;
        let base = n.fbm2(u, v, 32, 32, 3) - 0.5;
        let h = 0.22 + 0.78 * blade + (0.08 - 0.06 * lush) * clump + 0.06 * base;
        let c = mix(rgb(90, 90, 90), rgb(170, 170, 170), tone);
        (h, c)
    })
}

fn plaster(n: &Noise, size: usize) -> (HeightMap, Img) {
    render(size, size, |u, v| {
        let (wu, wv) = n.warp(u, v, 0.08, 6);
        let trowel = (u * 5.0 + n.fbm2(u, v, 4, 4, 3) * 2.0) * TAU;
        let h = 0.5
            + 0.20 * (n.fbm2(wu, wv, 6, 6, 5) - 0.5)
            + 0.10 * (n.fbm2(u, v, 24, 24, 4) - 0.5)
            + 0.03 * trowel.sin();
        (h, [0.5; 3])
    })
}

fn wallpaper(n: &Noise, size: usize) -> (HeightMap, Img) {
    render(size, size, |u, v| {
        let h = 0.5
            + 0.08 * (n.fbm2(u, v, 64, 64, 3) - 0.5)
            + 0.05 * (n.fbm2(u, v, 8, 8, 4) - 0.5)
            + 0.02 * (n.fbm2(u, v, 128, 4, 3) - 0.5);
        (h, [0.5; 3])
    })
}

fn painted_wood(n: &Noise, size: usize) -> (HeightMap, Img) {
    render(size, size, |u, v| {
        let g = wood_grain(n, u, v, 32);
        let h = 0.5 + 0.18 * (g - 0.5) + 0.06 * (n.fbm2(u, v, 64, 64, 3) - 0.5);
        (h, [0.5; 3])
    })
}

fn wood_oak(n: &Noise, size: usize) -> (HeightMap, Img) {
    render(size, size, |u, v| {
        let g = wood_grain(n, u, v, 18);
        let h = 0.12 + 0.88 * g + 0.06 * (n.fbm2(u, v, 128, 16, 3) - 0.5);
        (h, [0.5; 3])
    })
}

fn bark(n: &Noise, size: usize) -> (HeightMap, Img) {
    render(size, size, |u, v| {
        let (wu, wv) = n.warp(u, v, 0.02, 8);
        let base = n.fbm2(wu, wv, 160, 24, 4);
        let ridge = smoothstep(0.42, 0.62, base);
        let crev = smoothstep(0.40, 0.28, base);
        let fine = n.fbm2(u, v, 256, 16, 3) - 0.5;
        let pit = smoothstep(0.64, 0.80, n.fbm2(u, v, 128, 128, 2));
        let h = 0.12 + 0.78 * ridge + 0.14 * fine - 0.05 * pit;
        let c = mix(rgb(72, 60, 46), rgb(138, 120, 94), ridge);
        let c = mix(c, rgb(34, 27, 20), crev * 0.20);
        let mottle = n.fbm2(u, v, 8, 8, 3) - 0.5;
        let c = mul(c, 0.90 + 0.20 * (fine + 0.5) + 0.24 * mottle);
        let c = mix(c, rgb(38, 32, 26), pit * 0.35);
        let moss = smoothstep(0.55, 0.78, n.fbm2(u, v, 6, 6, 4)) * smoothstep(0.55, 0.15, ridge);
        let c = mix(c, rgb(76, 88, 56), moss * 0.30);
        (h, c)
    })
}

fn sidewalk(n: &Noise, size: usize) -> (HeightMap, Img) {
    let (cols, rows) = (3i64, 6i64);
    render(size, size, |u, v| {
        let yy = v * rows as f64;
        let row = yy.floor() as i64;
        let fy = yy - row as f64;
        let off = if row.rem_euclid(2) == 0 { 0.0 } else { 0.5 };
        let xx = u * cols as f64 + off;
        let col = xx.floor() as i64;
        let fx = xx - col as f64;
        let jy = smoothstep(0.0, 0.018, fy.min(1.0 - fy));
        let jx = smoothstep(0.0, 0.010, fx.min(1.0 - fx));
        let face = jx * jy;
        let r = n.rand(col.rem_euclid(cols), row.rem_euclid(rows), 121);
        let grain = n.fbm2(u, v, 256, 256, 3) - 0.5;
        let speck = n.fbm2(u, v, 128, 128, 2) - 0.5;
        let fine = n.fbm2(u, v, 512, 512, 1) - 0.5;
        let mottle = n.fbm2(u, v, 16, 16, 4) - 0.5;
        let h = 0.35
            + 0.65 * face
            + 0.10 * (r - 0.5)
            + 0.05 * grain
            + 0.05 * speck
            + 0.03 * fine
            + 0.05 * mottle;
        let base = mix(rgb(200, 200, 200), rgb(226, 226, 226), r);
        let c = mul(base, 0.96 + 0.08 * mottle + 0.05 * grain);
        let joint = rgb(155, 155, 155);
        (h, mix(joint, c, face))
    })
}

fn asphalt(n: &Noise, size: usize) -> (HeightMap, Img) {
    render(size, size, |u, v| {
        let base = n.fbm2(u, v, 8, 8, 4) - 0.5;
        let agg = n.fbm2(u, v, 128, 128, 3) - 0.5;
        let fine = n.fbm2(u, v, 512, 512, 1) - 0.5;
        let h = 0.5 + 0.12 * base + 0.20 * agg + 0.08 * fine;
        let sp = n.fbm2(u, v, 256, 256, 2);
        let light = smoothstep(0.48, 0.62, sp);
        let dark = 1.0 - smoothstep(0.38, 0.50, sp);
        let mut c = mix(rgb(80, 80, 82), rgb(140, 140, 142), light * 0.90);
        c = mix(c, rgb(52, 52, 54), dark * 0.65);
        let crack = 1.0 - smoothstep(0.0, 0.02, (n.fbm2(u, v, 6, 6, 4) - 0.5).abs());
        (h, mul(c, 1.0 - 0.22 * crack))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn max_step_f64(v: &[f64], w: usize, h: usize) -> f64 {
        let mut m = 0.0f64;
        for y in 0..h {
            for x in 1..w {
                m = m.max((v[y * w + x] - v[y * w + x - 1]).abs());
            }
        }
        for y in 1..h {
            for x in 0..w {
                m = m.max((v[y * w + x] - v[(y - 1) * w + x]).abs());
            }
        }
        m
    }

    fn seam_f64(v: &[f64], w: usize, h: usize) -> f64 {
        let mut m = 0.0f64;
        for y in 0..h {
            m = m.max((v[y * w] - v[y * w + w - 1]).abs());
        }
        for x in 0..w {
            m = m.max((v[x] - v[(h - 1) * w + x]).abs());
        }
        m
    }

    fn max_step_u8(v: &[u8], w: usize, h: usize, ch: usize) -> u8 {
        let mut m = 0u8;
        for y in 0..h {
            for x in 1..w {
                for c in 0..ch {
                    let a = v[(y * w + x) * ch + c];
                    let b = v[(y * w + x - 1) * ch + c];
                    m = m.max(a.abs_diff(b));
                }
            }
        }
        for y in 1..h {
            for x in 0..w {
                for c in 0..ch {
                    let a = v[(y * w + x) * ch + c];
                    let b = v[((y - 1) * w + x) * ch + c];
                    m = m.max(a.abs_diff(b));
                }
            }
        }
        m
    }

    fn seam_u8(v: &[u8], w: usize, h: usize, ch: usize) -> u8 {
        let mut m = 0u8;
        for y in 0..h {
            for c in 0..ch {
                m = m.max(v[(y * w) * ch + c].abs_diff(v[(y * w + w - 1) * ch + c]));
            }
        }
        for x in 0..w {
            for c in 0..ch {
                m = m.max(v[x * ch + c].abs_diff(v[((h - 1) * w + x) * ch + c]));
            }
        }
        m
    }

    #[test]
    fn every_set_tiles() {
        for set in SETS {
            let (height, albedo) = generate(set.name, 128);
            let hs = seam_f64(&height.v, 128, 128);
            let hm = max_step_f64(&height.v, 128, 128);
            // The wrap column is one ordinary sample step from the first column, so allow it to be
            // a hair above the busiest interior step (a hard seam is far larger than this).
            assert!(
                hs <= hm * 1.005 + 1e-9,
                "{}: height seam {hs} > interior {hm}",
                set.name
            );
            let cs = seam_u8(&albedo.px, 128, 128, 3);
            let cm = max_step_u8(&albedo.px, 128, 128, 3);
            assert!(
                cs <= cm + 1,
                "{}: albedo seam {cs} > interior {cm}",
                set.name
            );
        }
    }

    #[test]
    fn normal_of_flat_height_is_flat() {
        let mut h = HeightMap::new(8, 8);
        h.v.fill(0.5);
        let n = h.normal(0.02);
        assert!(
            n.px.as_chunks::<3>()
                .0
                .iter()
                .all(|p| p == &[128, 128, 255])
        );
    }

    #[test]
    fn arm_of_flat_height_is_flat() {
        let mut h = HeightMap::new(16, 16);
        h.v.fill(0.5);
        let a = h.arm(0.8, 0.1, 2.0, 1.0);
        assert!(
            a.px.as_chunks::<3>()
                .0
                .iter()
                .all(|p| p[0] == 255 && p[2] == 0)
        );
        assert!(a.px.as_chunks::<3>().0.iter().all(|p| p[1] == to_u8(0.8)));
    }

    #[test]
    fn set_names_are_unique() {
        for (i, a) in SETS.iter().enumerate() {
            for b in &SETS[i + 1..] {
                assert_ne!(a.name, b.name);
            }
        }
    }
}
