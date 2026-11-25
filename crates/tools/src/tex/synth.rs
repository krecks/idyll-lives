//! `cargo tex synth`: the procedural textures.
//!
//! Every map is made from a tileable height field: a periodic gradient-noise lattice (both axes wrap),
//! brick, stone, plank and shingle courses, a hex lattice, a woven grid and a field of grass blades or
//! leaves. The height becomes the normal map (Sobel, wrapping); the albedo is either the colour the
//! pattern gives or, for the greyscale "detail" sets, its luminance run through [`super::grey_detail`],
//! so the game's tint sets the hue and the mean lands where the old maps did.
//!
//! The built surfaces (masonry, paving, boards, shingles, plaster, asphalt and the floor coverings) are in
//! [`built`]: drawn in metres at the manifest's tile size, with their own roughness and multi-scale cavity
//! AO ([`Surface`], [`HeightMap::cavity_ao`]), packed into the arm map ([`pack_arm`]).
//!
//! The organic sets (lawn and field earth, foliage, bark, carpet, linen, wallpaper, painted wood and
//! oak) are made in [`organic`] instead: sprites (blades, leaves, fibres) drawn into a z-buffered
//! layer, with their own albedo tone, AO and roughness besides the height.
//!
//! The output keeps the filenames and sizes the game expects: the 31 texture sets under
//! `textures/<set>/`, the sidewalk under `street/sidewalk/`. Everything is original and CC0; there are
//! no downloads and no Python.

mod organic;

use super::grey_detail;
use super::jpeg::{self, Chroma};
use super::pil::Img;
use std::f64::consts::FRAC_1_SQRT_2;
use std::path::Path;

mod built;
#[cfg(test)]
mod standard;

pub use built::features;

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

/// What a built generator makes: the height, the colour (for a grey detail set, the luminance its detail
/// map is made from) and its own roughness and ambient occlusion.
pub struct Surface {
    pub height: HeightMap,
    pub albedo: Img,
    pub rough: Vec<f64>,
    pub ao: Vec<f64>,
}

/// The packed arm map: R = AO times the set's `ao_base`, G = roughness (at least 0.04), B = 0 (not metal).
pub fn pack_arm(w: usize, h: usize, ao: &[f64], rough: &[f64], ao_base: f64) -> Img {
    let mut px = vec![0u8; w * h * 3];
    for i in 0..w * h {
        px[i * 3] = to_u8(ao[i] * ao_base);
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
    /// Ambient-occlusion base: 1 for most sets, lower where the surface sits in shade (it scales a
    /// generator's own AO too).
    pub ao_base: f64,
}

impl Set {
    /// Its maps' directory under `web/public/assets/`, as the manifest names them: `textures/<set>`, the
    /// sidewalk's `street/sidewalk`.
    pub fn dir(&self) -> String {
        if self.name == "sidewalk" {
            "street/sidewalk".into()
        } else {
            format!("textures/{}", self.name)
        }
    }
}

/// The set a manifest material's `texture` (`textures/brick/albedo.jpg`) is the albedo of.
pub fn set_of_texture(texture: &str) -> Option<&'static Set> {
    let dir = texture.strip_suffix("/albedo.jpg")?;
    SETS.iter().find(|s| s.dir() == dir)
}

const fn set(
    name: &'static str,
    albedo: usize,
    normal: usize,
    grey: Option<(f64, f64, f32)>,
    relief: f64,
) -> Set {
    set_ao(name, albedo, normal, grey, relief, 1.0)
}

const fn set_ao(
    name: &'static str,
    albedo: usize,
    normal: usize,
    grey: Option<(f64, f64, f32)>,
    relief: f64,
    ao_base: f64,
) -> Set {
    Set {
        name,
        albedo,
        normal,
        arm: 512,
        grey,
        relief,
        ao_base,
    }
}

pub const SETS: [Set; 35] = [
    set("bark", 512, 512, None, 0.010),
    set_ao("brick", 1024, 1024, None, 0.0041, 0.945),
    set("carpet", 512, 512, Some((0.58, 0.85, 48.0)), 0.0052),
    set("fabric_linen", 512, 512, Some((0.80, 0.45, 0.0)), 0.0006),
    set("leather", 512, 512, Some((0.97, 0.6, 0.0)), 0.0026),
    set("floor_laminate", 1024, 1024, None, 0.0012),
    set("floor_tile", 1024, 1024, None, 0.0006),
    set("floor_wood", 1024, 1024, None, 0.0014),
    set("foliage", 512, 512, Some((0.58, 1.10, 0.0)), 0.012),
    set("grass", 1024, 1024, Some((0.75, 0.55, 0.0)), 0.0040),
    set("grass_lush", 1024, 1024, Some((0.70, 0.70, 24.0)), 0.005),
    set("painted_wood", 512, 512, Some((0.75, 1.0, 0.0)), 0.0007),
    set("path_hexpaving", 1024, 1024, None, 0.0014),
    set("path_concrete", 1024, 1024, None, 0.0012),
    set("path_flagstone", 1024, 1024, None, 0.006),
    set("path_gravel", 1024, 1024, None, 0.003),
    set_ao("road_asphalt", 1024, 1024, None, 0.0026, 0.8),
    set_ao(
        "roof_shingles",
        1024,
        1024,
        Some((0.60, 1.0, 0.0)),
        0.008,
        0.845,
    ),
    set("siding_wood", 1024, 1024, Some((0.68, 1.0, 0.0)), 0.016),
    set("stone_foundation", 1024, 1024, None, 0.0104),
    set("wall_plaster", 1024, 512, Some((0.80, 1.0, 0.0)), 0.0034),
    set("wallpaper", 1024, 512, Some((0.75, 1.0, 0.0)), 0.0015),
    set("wood_oak", 512, 512, Some((0.80, 0.60, 0.0)), 0.0003),
    set("floor_parquet", 1024, 1024, None, 0.0005),
    set("floor_checker", 1024, 1024, None, 0.0008),
    set("floor_hex", 1024, 512, None, 0.001),
    set("floor_terrazzo", 1024, 512, None, 0.0005),
    set("floor_marble", 1024, 1024, None, 0.0004),
    set("floor_cork", 512, 512, None, 0.0008),
    set("floor_lino", 1024, 512, None, 0.0002),
    set("floor_concrete", 1024, 1024, None, 0.0008),
    set("floor_pavers", 1024, 512, None, 0.0028),
    set("floor_encaustic", 1024, 512, None, 0.0008),
    set("floor_sisal", 512, 512, None, 0.0018),
    set_ao("sidewalk", 512, 512, Some((0.73, 1.0, 0.0)), 0.003, 0.82),
];

/// `cargo tex synth [set...]`.
pub fn run(only: &[&str]) -> Result<(), String> {
    for name in only {
        if *name != "sky" && !SETS.iter().any(|s| s.name == *name) && variant(name).is_none() {
            return Err(format!("unknown texture set: {name}"));
        }
    }
    let assets = crate::repo_root().join("web/public/assets");
    for set in SETS {
        if !only.is_empty() && !only.contains(&set.name) {
            continue;
        }
        write_set(&assets, &set)?;
    }
    // A set's variants, when named (`floor_wood.calm`): beside the set, in `textures/floor_wood.calm`.
    for set in only.iter().filter_map(|name| variant(name)) {
        write_set(&assets, &set)?;
    }
    if only.contains(&"sky") {
        println!("sky: textures/sky/clouds.png is `cargo tex clouds`, unchanged");
    }
    Ok(())
}

/// A set's variant by its name (`floor_wood.calm`: `built::variants`), as a set of that name with the set's own
/// sizes and response: a candidate for a look sheet, made only when named and kept out of git.
fn variant(name: &str) -> Option<Set> {
    let (base, which) = name.split_once('.')?;
    let set = SETS.iter().find(|s| s.name == base)?;
    (!which.is_empty() && built::variants(base).contains(&which)).then(|| Set {
        name: Box::leak(name.to_string().into_boxed_str()),
        ..*set
    })
}

/// A set's maps at their sizes: (albedo, normal, arm).
pub fn maps(set: &Set) -> (Img, Img, Img) {
    // The organic sets (`organic`) make their own albedo tone, AO and roughness.
    let (albedo, normal, arm) = match organic::generate(set.name, set.albedo) {
        Some(o) => o.maps(set),
        None => {
            let surface = generate(set.name, set.albedo);
            let mut height = surface.height;
            height.stretch();
            let albedo = match set.grey {
                Some((target, contrast, flatten)) => {
                    grey_detail(&surface.albedo, target, contrast, flatten)
                }
                None => surface.albedo,
            };
            let normal = height.normal(set.relief);
            let arm = pack_arm(height.w, height.h, &surface.ao, &surface.rough, set.ao_base);
            (albedo, normal, arm)
        }
    };
    let normal = if set.normal == set.albedo {
        normal
    } else {
        normal.resize(set.normal, set.normal)
    };
    let arm = if set.arm == set.albedo {
        arm
    } else {
        arm.resize(set.arm, set.arm)
    };
    (albedo, normal, arm)
}

/// The qualities a normal map is stored at, the lowest that keeps it ([`normal_jpeg`]) taken.
const NORMAL_QUALITY: [u8; 8] = [88, 90, 92, 94, 95, 96, 97, 98];

/// 8-bit storage keeps a unit normal within 1 % of its length; past 10 % the storage bent its direction too.
/// At least this share of a stored normal map's texels must keep within it.
const UNIT: (f64, f64) = (0.10, 0.95);
/// A tiling height's slopes sum to nothing, so its normals average flat; a mean slope over 0.02 (1.1°) tilts
/// the whole surface's light by about 2 % where it falls at 45°, as much as the eye tells apart.
const LEVEL: f64 = 0.02;

/// A stored normal map's (RGB texels, the renderer's `2 c - 1`) share of texels within [`UNIT`] of unit
/// length, and its mean slope (x / z, y / z over the texels that face out).
fn normal_stats(px: &[u8]) -> (f64, [f64; 2]) {
    let (mut unit, mut n) = (0usize, 0usize);
    let (mut sx, mut sy) = (0f64, 0f64);
    for t in px.as_chunks::<3>().0 {
        let [x, y, z] = t.map(|c| c as f64 / 255.0 * 2.0 - 1.0);
        if ((x * x + y * y + z * z).sqrt() - 1.0).abs() <= UNIT.0 {
            unit += 1;
        }
        if z > 0.05 {
            sx += x / z;
            sy += y / z;
            n += 1;
        }
    }
    let n = n.max(1) as f64;
    (unit as f64 / (px.len() / 3) as f64, [sx / n, sy / n])
}

/// A normal map as a JPEG: 4:4:4 (its channels are a direction's components, each a quantity of its own,
/// which 4:2:0 would average two of, bending every normal), at the lowest of [`NORMAL_QUALITY`] whose decoded
/// map keeps what `standard.rs` holds a stored normal map to: [`UNIT`] and [`LEVEL`]. (The highest when none
/// does: the standard's test then says which.)
fn normal_jpeg(normal: &Img) -> (Vec<u8>, u8) {
    let keeps = |px: &[u8]| {
        let (unit, [sx, sy]) = normal_stats(px);
        unit >= UNIT.1 && sx.abs() <= LEVEL && sy.abs() <= LEVEL
    };
    let mut stored = (Vec::new(), 0);
    for q in NORMAL_QUALITY {
        let bytes = jpeg::encode(&normal.px, normal.w, normal.h, 3, q, true, Chroma::Full);
        let kept = jpeg::decode(&bytes).is_some_and(|(.., px)| keeps(&px));
        stored = (bytes, q);
        if kept {
            break;
        }
    }
    stored
}

fn write_set(assets: &Path, set: &Set) -> Result<(), String> {
    let (albedo, normal, arm) = maps(set);
    let dir = assets.join(set.dir());
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    // The albedo is for the eye (4:2:0, as any photograph); the normal and arm maps are data, each channel
    // its own quantity: 4:4:4 (the arm map's ambient occlusion and roughness come back within half the
    // error, and its blue, metalness, stays nearer 0).
    albedo.save_jpeg(&dir.join("albedo.jpg"), 85, true, Chroma::Half)?;
    let (normal_bytes, quality) = normal_jpeg(&normal);
    let path = dir.join("normal.jpg");
    std::fs::write(&path, normal_bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    arm.save_jpeg(&dir.join("arm.jpg"), 85, true, Chroma::Full)?;
    let mut bytes = 0;
    for f in ["albedo.jpg", "normal.jpg", "arm.jpg"] {
        bytes += std::fs::metadata(dir.join(f))
            .map_err(|e| e.to_string())?
            .len();
    }
    println!(
        "{:<18} {}x{}  {} KB  (normal q{quality})",
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

/// A built set's surface ([`built`]; the organic sets are [`organic`]'s).
fn generate(name: &str, size: usize) -> Surface {
    // (A variant has its set's seed: its boards lie where the set's do.)
    let n = Noise::new(seed(name.split('.').next().unwrap_or(name)));
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
        // (The organic sets' maps: `organic_maps_tile`.)
        for set in SETS.iter().filter(|s| !organic::NAMES.contains(&s.name)) {
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

    /// A stone that crosses the wrap is one stone: drawn in the tile's coordinates its texture jumps there, and
    /// summed over the column the wrap's step is larger than any column's inside (at its size: smaller, the
    /// joints the cells' borders bunch near weigh more).
    #[test]
    fn a_flagstone_is_whole_across_the_wrap() {
        const N: usize = 1024;
        let albedo = generate("path_flagstone", N).albedo;
        let at = |x: usize, y: usize, c: usize| albedo.px[(y * N + x) * 3 + c] as i32;
        let col = |a: usize, b: usize| -> i32 {
            (0..N * 3)
                .map(|i| (at(a, i / 3, i % 3) - at(b, i / 3, i % 3)).abs())
                .sum()
        };
        let row = |a: usize, b: usize| -> i32 {
            (0..N * 3)
                .map(|i| (at(i / 3, a, i % 3) - at(i / 3, b, i % 3)).abs())
                .sum()
        };
        let (cw, ci) = (col(0, N - 1), (1..N).map(|x| col(x, x - 1)).max().unwrap());
        let (rw, ri) = (row(0, N - 1), (1..N).map(|y| row(y, y - 1)).max().unwrap());
        assert!(
            cw <= ci && rw <= ri,
            "wrap steps {cw}, {rw} > interior {ci}, {ri}"
        );
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
    fn leather_has_creases_and_an_uneven_sheen() {
        let Surface { height, rough, .. } = generate("leather", 256);
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
