//! `cargo tex synth`: the procedural textures.
//!
//! Every map is made from a tileable height field: a periodic gradient-noise lattice (both axes wrap),
//! brick, stone, plank and shingle courses, a hex lattice, a woven grid and a field of grass blades or
//! leaves. The height becomes the normal map (Sobel, wrapping) and the packed arm map (AO from a
//! blurred height, roughness from height and edges); the albedo is either the colour the pattern gives
//! or, for the greyscale "detail" sets, the height run through [`super::grey_detail`], so the game's
//! tint sets the hue and the mean lands where the old maps did.
//!
//! The built surfaces (masonry, paving, boards, shingles, plaster, asphalt) are in [`built`]: drawn
//! in metres at the manifest's tile size, with their own roughness and multi-scale cavity AO
//! ([`Surface`], [`HeightMap::cavity_ao`]) instead of the ones [`HeightMap::arm`] derives.
//!
//! The organic sets (lawn and field earth, foliage, bark, carpet, linen, wallpaper, painted wood and
//! oak) are made in [`organic`] instead: sprites (blades, leaves, fibres) drawn into a z-buffered
//! layer, with their own albedo tone, AO and roughness besides the height.
//!
//! The output keeps the filenames and sizes the game expects: the 20 texture sets under
//! `textures/<set>/`, the sidewalk under `street/sidewalk/`. Everything is original and CC0; there are
//! no downloads and no Python.

mod organic;

use super::grey_detail;
use super::pil::{Filter, Img};
use crate::content;
use std::f64::consts::FRAC_1_SQRT_2;
use std::path::Path;

mod built;

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
    (FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
];

impl Noise {
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// An independent field from the same seed (`fbm2` salts only by octave, so two layers at one
    /// frequency would otherwise be the same noise).
    pub fn layer(&self, k: u64) -> Self {
        Self::new(self.seed ^ k.wrapping_mul(0xD6E8_FEB8_6659_FD93).rotate_left(17))
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

    /// Ambient occlusion from cavities at several scales: each `(radius px, gain)` term darkens a
    /// pixel by how far it sits below the (twice box-blurred, so tent-weighted) height around it.
    /// 1 on open ground; the generator chooses the scales, so a mortar joint and a tile's whole
    /// face each get the occlusion of their own size.
    pub fn cavity_ao(&self, terms: &[(usize, f64)]) -> Vec<f64> {
        let mut ao = vec![1.0f64; self.w * self.h];
        for &(radius, gain) in terms {
            let low = self.blur(radius.max(1)).blur(radius.max(1));
            for (a, (&lo, &h)) in ao.iter_mut().zip(low.v.iter().zip(&self.v)) {
                *a -= (lo - h).max(0.0) * gain;
            }
        }
        for a in &mut ao {
            *a = a.clamp(0.05, 1.0);
        }
        ao
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

/// `f(x, y)` for every pixel, rows split over the machine's threads (the result does not depend on the
/// split: each pixel is computed on its own).
pub fn par_map<T: Send + Clone + Default>(
    w: usize,
    h: usize,
    f: impl Fn(usize, usize) -> T + Sync,
) -> Vec<T> {
    let mut out = vec![T::default(); w * h];
    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(16);
    let rows = h.div_ceil(threads).max(1);
    std::thread::scope(|sc| {
        for (k, chunk) in out.chunks_mut(rows * w).enumerate() {
            let f = &f;
            sc.spawn(move || {
                for (i, slot) in chunk.iter_mut().enumerate() {
                    *slot = f(i % w, k * rows + i / w);
                }
            });
        }
    });
    out
}

/// What a generator makes: the height, the colour (or, with `grey_from_albedo`, the luminance the
/// grey detail map is made from instead of the height) and optionally its own roughness and ambient
/// occlusion (else [`HeightMap::arm`] derives both from the height with the set's numbers).
pub struct Surface {
    pub height: HeightMap,
    pub albedo: Img,
    pub rough: Option<Vec<f64>>,
    pub ao: Option<Vec<f64>>,
    pub grey_from_albedo: bool,
}

impl Surface {
    pub fn plain((height, albedo): (HeightMap, Img)) -> Self {
        Self {
            height,
            albedo,
            rough: None,
            ao: None,
            grey_from_albedo: false,
        }
    }
}

/// The packed arm map from explicit fields: R = AO, G = roughness, B = 0 (not metal).
pub fn pack_arm(w: usize, h: usize, ao: &[f64], rough: &[f64]) -> Img {
    let mut px = vec![0u8; w * h * 3];
    for i in 0..w * h {
        px[i * 3] = to_u8(ao[i]);
        px[i * 3 + 1] = to_u8(rough[i].clamp(0.04, 1.0));
    }
    Img { w, h, ch: 3, px }
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
    /// (base roughness, height variation, AO gain), for a generator that leaves roughness and AO to
    /// [`HeightMap::arm`].
    pub rough: (f64, f64, f64),
    /// Ambient-occlusion base: 1 for most sets, lower where the surface sits in shade (it scales a
    /// generator's own AO too).
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

pub const SETS: [Set; 21] = [
    set("bark", 512, 512, None, 0.010, (0.84, 0.20, 2.3)),
    set_ao("brick", 1024, 1024, None, 0.0041, (0.81, 0.12, 4.5), 0.945),
    set(
        "carpet",
        512,
        512,
        Some((0.58, 0.85, 48.0)),
        0.0052,
        (0.87, 0.30, 5.0),
    ),
    set(
        "fabric_linen",
        512,
        512,
        Some((0.80, 0.45, 0.0)),
        0.0006,
        (0.60, 0.10, 1.5),
    ),
    set(
        "leather",
        512,
        512,
        Some((0.97, 0.6, 0.0)),
        0.0026,
        (0.44, 0.0, 0.0),
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
        0.012,
        (0.74, 0.20, 4.0),
    ),
    set(
        "grass",
        1024,
        1024,
        Some((0.75, 0.55, 0.0)),
        0.0040,
        (0.45, 0.15, 1.6),
    ),
    set(
        "grass_lush",
        1024,
        1024,
        Some((0.70, 0.70, 24.0)),
        0.005,
        (0.74, 0.10, 1.7),
    ),
    set(
        "painted_wood",
        512,
        512,
        Some((0.75, 1.0, 0.0)),
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
    set_ao(
        "road_asphalt",
        1024,
        1024,
        None,
        0.0026,
        (0.76, 0.15, 4.0),
        0.8,
    ),
    set_ao(
        "roof_shingles",
        1024,
        1024,
        Some((0.60, 1.0, 0.0)),
        0.008,
        (0.61, 0.20, 20.0),
        0.845,
    ),
    set(
        "siding_wood",
        1024,
        1024,
        Some((0.68, 1.0, 0.0)),
        0.016,
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
        Some((0.80, 1.0, 0.0)),
        0.0034,
        (0.91, 0.03, 0.6),
    ),
    set(
        "wallpaper",
        1024,
        512,
        Some((0.75, 1.0, 0.0)),
        0.0015,
        (0.78, 0.05, 0.05),
    ),
    set(
        "wood_oak",
        512,
        512,
        Some((0.80, 0.60, 0.0)),
        0.0003,
        (0.52, 0.20, 0.2),
    ),
    set_ao(
        "sidewalk",
        512,
        512,
        Some((0.73, 1.0, 0.0)),
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
    // The organic sets (`organic`) make their own albedo tone, AO and roughness.
    let (albedo, normal, arm) = match organic::generate(set.name, set.albedo) {
        Some(o) => o.maps(set),
        None => {
            let surface = generate(set.name, set.albedo);
            let mut height = surface.height;
            height.stretch();
            let albedo = match set.grey {
                Some((target, contrast, flatten)) if surface.grey_from_albedo => {
                    grey_detail(&surface.albedo, target, contrast, flatten)
                }
                Some((target, contrast, flatten)) => {
                    grey_detail(&height.grey(), target, contrast, flatten)
                }
                None => surface.albedo,
            };
            let normal = height.normal(set.relief);
            let arm = match (&surface.ao, &surface.rough) {
                (Some(ao), Some(rough)) => {
                    let ao: Vec<f64> = ao.iter().map(|a| a * set.ao_base).collect();
                    pack_arm(height.w, height.h, &ao, rough)
                }
                _ => height.arm(set.rough.0, set.rough.1, set.rough.2, set.ao_base),
            };
            (albedo, normal, arm)
        }
    };
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

fn generate(name: &str, size: usize) -> Surface {
    if let Some(o) = organic::generate(name, size) {
        return Surface::plain((o.height, o.albedo.rgb()));
    }
    let n = Noise::new(seed(name));
    built::generate(name, &n, size).unwrap_or_else(|| unreachable!("unknown set {name}"))
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
            let Surface { height, albedo, .. } = generate(set.name, 128);
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
    fn leather_has_creases_and_an_uneven_sheen() {
        let Surface { height, rough, .. } = generate("leather", 256);
        let rough = rough.expect("leather makes its own roughness");
        let (lo, hi) = rough.iter().fold((f64::INFINITY, 0.0f64), |(lo, hi), &r| {
            (lo.min(r), hi.max(r))
        });
        // Polished where it is sat on, duller between and in the creases.
        assert!((0.33..=0.38).contains(&lo), "smoothest {lo}");
        assert!((0.46..=0.51).contains(&hi), "roughest {hi}");
        // The creases are the deepest thing in it: a twentieth of the hide lies well under its middle.
        let mut h = height.v.clone();
        h.sort_by(f64::total_cmp);
        let (deep, middle) = (h[h.len() / 20], h[h.len() / 2]);
        assert!(middle - deep > 0.12, "creases {deep} under {middle}");
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
