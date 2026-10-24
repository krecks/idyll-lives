//! `cargo tex`: the texture scripts (was `tools/art/*.py`), one subcommand each, with their arguments and paths.
//!
//! - `clouds <out.png>`: the sky's tileable cloud layer (`clouds.py`).
//! - `process <src_root> <dst_root>`: Poly Haven / ambientCG maps -> the game's albedo/normal/arm JPGs (`process.py`).
//! - `house <src_root> <dst_root> [set...]`: the house and interior textures (`process_house.py`).
//! - `materials`: writes `materials.json` in the current directory (`materials.py`).
//! - `ph <outdir> <res> <maps,comma> <id>...`: downloads Poly Haven maps (`ph_tex.py`).
//!
//! The numbers follow numpy and Pillow step by step (float64 as numpy evaluates it, numpy's pairwise sums and
//! Python's compensated `sum()`, Pillow's fixed-point resampling and box blur, libjpeg-turbo's decoder and
//! encoder), so the outputs are the scripts' to the byte; see `pil` and `jpeg`.

mod jpeg;
pub mod pil;

use crate::pcg64::Pcg64;
use pil::{Filter, Img};
use std::fmt::Write as _;
use std::path::Path;

pub const USAGE: &str = "usage: cargo tex <command> ...
  clouds <out.png>                          the sky's tileable cloud layer
  process <src_root> <dst_root>             Poly Haven/ambientCG maps -> albedo/normal/arm JPGs
  house <src_root> <dst_root> [set...]      the house and interior textures
  materials                                 writes materials.json (PBR material + environment entries)
  ph <outdir> <res> <maps,comma> <id>...    downloads Poly Haven texture maps (CC0)";

pub fn main_with(args: &[String]) -> i32 {
    let a: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let res = match a.as_slice() {
        ["clouds", out] => clouds(Path::new(out)),
        ["process", src, dst] => process(Path::new(src), Path::new(dst)),
        ["house", src, dst, only @ ..] => house(Path::new(src), Path::new(dst), only),
        ["materials"] => std::fs::write("materials.json", materials_json()).map_err(|e| format!("materials.json: {e}")),
        ["ph", out, res, maps, ids @ ..] => ph(Path::new(out), res, maps, ids),
        ["-h" | "--help" | "help", ..] => {
            println!("{USAGE}");
            return 0;
        }
        _ => {
            eprintln!("{USAGE}");
            return 2;
        }
    };
    match res {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("tex: {e}");
            1
        }
    }
}

// --- numpy's arithmetic ---------------------------------------------------------------------------

/// numpy's pairwise summation (`np.add.reduce` on a contiguous float64 array, so `a.sum()` and `a.mean()`).
pub fn np_sum(a: &[f64]) -> f64 {
    let n = a.len();
    if n < 8 {
        let mut r = 0.0;
        for &v in a {
            r += v;
        }
        r
    } else if n <= 128 {
        let mut r = [0.0f64; 8];
        r.copy_from_slice(&a[..8]);
        let mut i = 8;
        while i < n - n % 8 {
            for j in 0..8 {
                r[j] += a[i + j];
            }
            i += 8;
        }
        let mut res = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]));
        while i < n {
            res += a[i];
            i += 1;
        }
        res
    } else {
        let mut n2 = n / 2;
        n2 -= n2 % 8;
        np_sum(&a[..n2]) + np_sum(&a[n2..])
    }
}

pub fn np_mean(a: &[f64]) -> f64 {
    np_sum(a) / a.len() as f64
}

/// Python's built-in `sum()` of floats (3.12+: Neumaier's compensated summation).
pub fn py_fsum(items: impl IntoIterator<Item = f64>) -> f64 {
    let mut it = items.into_iter();
    let Some(mut f) = it.next() else { return 0.0 };
    let mut c = 0.0f64;
    for x in it {
        let t = f + x;
        if f.abs() >= x.abs() {
            c += (f - t) + x;
        } else {
            c += (x - t) + f;
        }
        f = t;
    }
    if c != 0.0 && c.is_finite() {
        f += c;
    }
    f
}

// --- clouds.py ------------------------------------------------------------------------------------

const CW: usize = 1024;
const CH: usize = 256;

/// `np.linspace(0, stop, num, endpoint)`.
fn linspace(stop: f64, num: usize, endpoint: bool) -> Vec<f64> {
    let div = if endpoint { num - 1 } else { num };
    let step = stop / div as f64;
    let mut y: Vec<f64> = (0..num).map(|i| i as f64 * step + 0.0).collect();
    if endpoint && num > 1 {
        y[num - 1] = stop;
    }
    y
}

/// Value noise that wraps horizontally, smoothly interpolated to W x H (row-major).
fn tile_noise(rng: &mut Pcg64, cells_x: usize, cells_y: usize) -> Vec<f64> {
    let gw = cells_x + 1;
    let mut g = vec![0.0f64; (cells_y + 1) * gw];
    for y in 0..=cells_y {
        for x in 0..cells_x {
            g[y * gw + x] = rng.random();
        }
        g[y * gw + cells_x] = g[y * gw];
    }
    let xs = linspace(cells_x as f64, CW, false);
    let ys = linspace(cells_y as f64, CH, false);
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let x0: Vec<usize> = xs.iter().map(|&v| v as usize).collect();
    let y0: Vec<usize> = ys.iter().map(|&v| v as usize).collect();
    let fx: Vec<f64> = xs.iter().zip(&x0).map(|(&v, &i)| smooth(v - i as f64)).collect();
    let fy: Vec<f64> = ys.iter().zip(&y0).map(|(&v, &i)| smooth(v - i as f64)).collect();
    let mut out = vec![0.0f64; CW * CH];
    for (r, (&yi, &fyv)) in y0.iter().zip(&fy).enumerate() {
        for (c, (&xi, &fxv)) in x0.iter().zip(&fx).enumerate() {
            let a = g[yi * gw + xi];
            let b = g[yi * gw + xi + 1];
            let cc = g[(yi + 1) * gw + xi];
            let d = g[(yi + 1) * gw + xi + 1];
            let top = a + (b - a) * fxv;
            let bot = cc + (d - cc) * fxv;
            out[r * CW + c] = top + (bot - top) * fyv;
        }
    }
    out
}

/// The cloud layer's RGBA pixels (1024 x 256).
pub fn clouds_rgba() -> Vec<u8> {
    let mut rng = Pcg64::new(7);
    let mut n = vec![0.0f64; CW * CH];
    for o in 0..6 {
        let t = tile_noise(&mut rng, 6 << o, 2 << o);
        let w = 0.5f64.powi(o);
        for (acc, v) in n.iter_mut().zip(&t) {
            *acc += v * w;
        }
    }
    for v in &mut n {
        *v /= 1.97;
    }
    let vs = linspace(1.0, CH, true);
    let mut rgba = vec![0u8; CW * CH * 4];
    for (r, &v) in vs.iter().enumerate() {
        let fade = (v / 0.25).clamp(0.0, 1.0) * ((1.0 - v) / 0.35).clamp(0.0, 1.0);
        for c in 0..CW {
            let nv = n[r * CW + c];
            let dens = ((nv - 0.5) / 0.22).clamp(0.0, 1.0).powf(1.4) * fade;
            let shade = 0.82 + 0.18 * (nv * 1.4 - 0.2).clamp(0.0, 1.0);
            let s = (shade * 255.0) as u8;
            rgba[(r * CW + c) * 4..][..4].copy_from_slice(&[s, s, s, (dens * 255.0) as u8]);
        }
    }
    rgba
}

fn clouds(out: &Path) -> Result<(), String> {
    use image::ImageEncoder;
    let rgba = clouds_rgba();
    let f = std::fs::File::create(out).map_err(|e| format!("{}: {e}", out.display()))?;
    image::codecs::png::PngEncoder::new_with_quality(std::io::BufWriter::new(f), image::codecs::png::CompressionType::Best, image::codecs::png::FilterType::Adaptive)
        .write_image(&rgba, CW as u32, CH as u32, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("{}: {e}", out.display()))?;
    println!("wrote {}", out.display());
    Ok(())
}

// --- process.py -----------------------------------------------------------------------------------

/// `lin`/`srgb` of process.py, in Python floats.
pub fn lin(v: f64) -> f64 {
    let c = v / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

pub fn srgb(c: f64) -> f64 {
    if c <= 0.0031308 { 12.92 * c } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

/// The point LUT that maps greyscale to a detail map with linear mean `target`, given the histogram.
pub fn detail_lut(hist: &[u64; 256], target: f64) -> [u8; 256] {
    let total: u64 = hist.iter().sum();
    let mean_lin = py_fsum(hist.iter().enumerate().map(|(v, &n)| lin(v as f64) * n as f64)) / total as f64;
    let k = target / mean_lin;
    std::array::from_fn(|v| (srgb((lin(v as f64) * k).min(1.0)) * 255.0).round_ties_even().clamp(0.0, 255.0) as u8)
}

/// process.py's `grey_detail`: luminance-only, linear mean ~`target`, as RGB.
fn grey_detail_lut(im: &Img, target: f64) -> Img {
    let g = im.grey();
    g.point(&detail_lut(&g.histogram(), target)).rgb()
}

fn save(im: &Img, path: &Path, size: usize, q: u8) -> Result<(), String> {
    im.resize(size, size, Filter::Lanczos).save_jpeg(path, q, true)
}

fn mkdir(p: &Path) -> Result<(), String> {
    std::fs::create_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))
}

/// `os.walk(dst)` top-down, printing each file's path less `dst` and its size, as process.py ends.
fn print_tree(dst: &Path) -> Result<(), String> {
    fn walk(dir: &Path, dst: &str, out: &mut String) -> Result<(), String> {
        let mut dirs = Vec::new();
        for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let e = e.map_err(|e| e.to_string())?;
            let p = e.path();
            if p.is_dir() {
                dirs.push(p);
            } else {
                let size = e.metadata().map_err(|e| e.to_string())?.len();
                let _ = writeln!(out, "{} {size}", p.display().to_string().replace(dst, ""));
            }
        }
        for d in dirs {
            walk(&d, dst, out)?;
        }
        Ok(())
    }
    let mut out = String::new();
    walk(dst, &dst.display().to_string(), &mut out)?;
    print!("{out}");
    Ok(())
}

/// name, source dir, albedo size, normal size, greyscale.
const PROCESS_SETS: [(&str, &str, usize, usize, bool); 8] = [
    ("floor_laminate", "ph/laminate_floor_02", 1024, 1024, false),
    ("wall_plaster", "ph/painted_plaster_wall", 1024, 512, true),
    ("foliage", "ph/forest_leaves_03", 512, 512, true),
    ("road_asphalt", "ph/asphalt_02", 1024, 1024, false),
    ("path_hexpaving", "ph/hexagonal_concrete_paving", 1024, 1024, false),
    ("fabric_linen", "ph/rough_linen", 512, 512, true),
    ("wood_oak", "ph/oak_veneer_01", 512, 512, true),
    ("bark", "ph/bark_brown_02", 512, 512, false),
];

fn process(src: &Path, dst: &Path) -> Result<(), String> {
    for (name, d, asize, nsize, grey) in PROCESS_SETS {
        let out = dst.join(name);
        mkdir(&out)?;
        let s = src.join(d);
        let diff = Img::open(&s.join("Diffuse.jpg"))?.rgb();
        save(&if grey { grey_detail_lut(&diff, 0.8) } else { diff }, &out.join("albedo.jpg"), asize, 85)?;
        save(&Img::open(&s.join("nor_gl.jpg"))?.rgb(), &out.join("normal.jpg"), nsize, 88)?;
        save(&Img::open(&s.join("arm.jpg"))?.rgb(), &out.join("arm.jpg"), 512, 85)?;
    }
    // ambientCG grass: greyscale detail + packed AO/roughness.
    let g = src.join("acg/Grass001");
    let out = dst.join("grass");
    mkdir(&out)?;
    save(&grey_detail_lut(&Img::open(&g.join("Color.jpg"))?.rgb(), 0.75), &out.join("albedo.jpg"), 1024, 85)?;
    save(&Img::open(&g.join("NormalGL.jpg"))?.rgb(), &out.join("normal.jpg"), 1024, 88)?;
    let ao = Img::open(&g.join("AmbientOcclusion.jpg"))?.grey().resize(512, 512, Filter::Bicubic);
    let ro = Img::open(&g.join("Roughness.jpg"))?.grey().resize(512, 512, Filter::Bicubic);
    Img::merge([&ao, &ro, &Img::new(512, 512, 1, 0)]).save_jpeg(&out.join("arm.jpg"), 85, false)?;
    print_tree(dst)
}

// --- process_house.py -----------------------------------------------------------------------------

fn to_lin(a: f64) -> f64 {
    if a <= 0.04045 { a / 12.92 } else { ((a + 0.055) / 1.055).powf(2.4) }
}

fn to_srgb(a: f64) -> f64 {
    if a <= 0.0031308 { 12.92 * a } else { 1.055 * a.powf(1.0 / 2.4) - 0.055 }
}

/// Gaussian blur that wraps around the edges, so tiling stays seamless (blur a 3x3 tiling, keep the middle).
pub fn wrap_blur(img: &Img, radius: f32) -> Img {
    img.tile(3, 3).gaussian_blur(radius).crop(img.w, img.h, img.w, img.h)
}

/// The detail map's grey levels before the soft clip's gain is solved: linear, contrast applied.
fn detail_linear(g: &Img, contrast: f64, flatten: f32) -> Vec<f64> {
    let mut lin: Vec<f64> = g.px.iter().map(|&v| to_lin(v as f64 / 255.0).max(1e-4)).collect();
    if flatten > 0.0 {
        let low: Vec<f64> = wrap_blur(g, flatten).px.iter().map(|&v| to_lin(v as f64 / 255.0)).collect();
        let low_mean = np_mean(&low);
        for (l, &lo) in lin.iter_mut().zip(&low) {
            *l = *l / lo.max(1e-4) * low_mean;
        }
    }
    let logs: Vec<f64> = lin.iter().map(|v| v.ln()).collect();
    let m = np_mean(&logs).exp();
    let lm = m.ln();
    logs.iter().map(|&ll| (lm + contrast * (ll - lm)).exp()).collect()
}

/// process_house.py's `grey_detail`: luminance-only detail map with linear mean ~`target`; `contrast` < 1
/// softens the detail, `flatten` > 0 divides out blotches larger than ~`flatten` px; highlights soft-clipped.
pub fn grey_detail(im: &Img, target: f64, contrast: f64, flatten: f32) -> Img {
    let g = im.grey();
    let lin = detail_linear(&g, contrast, flatten);
    let soft = |x: f64| if x < 0.9 { x } else { 0.9 + 0.1 * (1.0 - (-(x - 0.9) / 0.1).exp()) };
    let mut k = 1.0f64;
    let mut y = Vec::new();
    for _ in 0..30 {
        y = lin.iter().map(|&l| soft(l * k)).collect();
        k *= target / np_mean(&y);
    }
    let px = y.iter().map(|&v| (to_srgb(v.clamp(0.0, 1.0)) * 255.0).round_ties_even() as u8).collect();
    Img { w: g.w, h: g.h, ch: 1, px }.rgb()
}

#[derive(Clone, Copy)]
enum Op {
    FlipV,
    Stagger(&'static [usize], u32),
    Stack2V,
}

/// `np.roll(a, shift, axis=1)` on rows `r0..r1` of an RGB/L image, in place from `src`.
fn roll_rows_x(src: &Img, dst: &mut Img, r0: usize, r1: usize, shift: usize) {
    let (w, ch) = (src.w, src.ch);
    for y in r0..r1 {
        for x in 0..w {
            let to = (x + shift) % w;
            dst.px[(y * w + to) * ch..][..ch].copy_from_slice(&src.px[(y * w + x) * ch..][..ch]);
        }
    }
}

/// `np.roll(a, shift, axis=0)`.
fn roll_y(im: &Img, shift: i64) -> Img {
    let h = im.h as i64;
    let stride = im.w * im.ch;
    let mut px = vec![0u8; im.px.len()];
    for y in 0..h {
        let to = (y + shift).rem_euclid(h) as usize;
        px[to * stride..(to + 1) * stride].copy_from_slice(&im.px[y as usize * stride..(y as usize + 1) * stride]);
    }
    Img { px, ..im.clone() }
}

fn orient(im: Img, op: Op, normal: bool) -> Img {
    match op {
        Op::FlipV => {
            let im = im.flip_tb();
            if normal {
                let inv: [u8; 256] = std::array::from_fn(|v| 255 - v as u8);
                Img::merge([&im.channel(0), &im.channel(1).point(&inv), &im.channel(2)])
            } else {
                im
            }
        }
        Op::Stagger(cuts, seed) => {
            // Shift each board strip sideways by its own amount (wrapping, so it stays seamless); the same seed
            // gives every map the same shifts.
            let (h, w) = (im.h, im.w);
            let a = roll_y(&im, -(cuts[0] as i64));
            let mut rows: Vec<usize> = cuts.iter().map(|c| c - cuts[0]).collect();
            rows.push(h);
            let mut rng = Pcg64::new(seed);
            let mut out = a.clone();
            for r in rows.windows(2) {
                let shift = rng.integers((w / 8) as i64, (w - w / 8) as i64) as usize;
                roll_rows_x(&a, &mut out, r[0], r[1], shift);
            }
            roll_y(&out, cuts[0] as i64)
        }
        Op::Stack2V => im.tile(1, 2),
    }
}

fn save_house(im: &Img, path: &Path, size: usize, q: u8) -> Result<(), String> {
    let im = if (im.w, im.h) != (size, size) { im.resize(size, size, Filter::Lanczos) } else { im.clone() };
    im.save_jpeg(path, q, true)
}

fn load_maps(src: &Path, kind: &str, d: &str) -> Result<(Img, Img, Img), String> {
    let s = src.join(kind).join(d);
    if kind == "ph" {
        return Ok((Img::open(&s.join("Diffuse.jpg"))?.rgb(), Img::open(&s.join("nor_gl.jpg"))?.rgb(), Img::open(&s.join("arm.jpg"))?.rgb()));
    }
    let rough = Img::open(&s.join("Roughness.jpg"))?.grey();
    let ao_path = s.join("AmbientOcclusion.jpg");
    let mut ao = if ao_path.exists() { Img::open(&ao_path)?.grey() } else { Img::new(rough.w, rough.h, 1, 255) };
    if (ao.w, ao.h) != (rough.w, rough.h) {
        ao = ao.resize(rough.w, rough.h, Filter::Lanczos);
    }
    let arm = Img::merge([&ao, &rough, &Img::new(rough.w, rough.h, 1, 0)]); // AO, roughness, metal 0
    Ok((Img::open(&s.join("Color.jpg"))?.rgb(), Img::open(&s.join("NormalGL.jpg"))?.rgb(), arm))
}

/// WoodSiding009 is 1024x512 with 8 boards; board edges (steepest displacement drop) at these rows, and again
/// 512 lower once stacked.
const SIDING_CUTS: [usize; 16] = [3, 66, 131, 195, 257, 319, 386, 452, 515, 578, 643, 707, 769, 831, 898, 964];

struct HouseSet {
    name: &'static str,
    /// "ph" (Poly Haven) or "acg" (ambientCG).
    kind: &'static str,
    src: &'static str,
    albedo: usize,
    normal: usize,
    /// Greyscale detail (target linear mean, contrast, flatten radius), or None to keep the colour.
    grey: Option<(f64, f64, f32)>,
    ops: &'static [Op],
}

const fn set(name: &'static str, kind: &'static str, src: &'static str, albedo: usize, normal: usize, grey: Option<(f64, f64, f32)>, ops: &'static [Op]) -> HouseSet {
    HouseSet { name, kind, src, albedo, normal, grey, ops }
}

const HOUSE_SETS: [HouseSet; 10] = [
    set("siding_wood", "acg", "WoodSiding009", 1024, 1024, Some((0.68, 1.0, 0.0)), &[Op::Stack2V, Op::Stagger(&SIDING_CUTS, 7), Op::FlipV]),
    set("brick", "ph", "red_brick_03", 1024, 1024, None, &[]),
    set("roof_shingles", "ph", "grey_roof_tiles_02", 1024, 1024, Some((0.6, 1.0, 0.0)), &[Op::FlipV]),
    set("stone_foundation", "ph", "rustic_stone_wall_02", 1024, 1024, None, &[]),
    set("floor_wood", "acg", "WoodFloor040", 1024, 1024, None, &[]),
    set("floor_tile", "acg", "Tiles141", 1024, 1024, None, &[]),
    set("carpet", "acg", "Carpet012", 512, 512, Some((0.6, 1.0, 48.0)), &[]),
    set("wallpaper", "acg", "Plaster001", 1024, 512, Some((0.75, 0.6, 0.0)), &[]),
    set("painted_wood", "acg", "PaintedWood008A", 512, 512, Some((0.75, 0.45, 0.0)), &[]),
    set("grass_lush", "acg", "Grass004", 1024, 1024, Some((0.7, 1.0, 40.0)), &[]),
];

/// Roughness remap (arm G, min..max -> lo..hi): the scanned maps of these read too glossy.
const ROUGH: [(&str, f64, f64); 2] = [("grass_lush", 0.75, 1.0), ("brick", 0.7, 0.95)];

pub fn remap_rough(arm: &Img, lo: f64, hi: f64) -> Img {
    let g: Vec<f64> = arm.px.iter().skip(1).step_by(3).map(|&v| v as f64).collect();
    let (min, max) = g.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &v| (a.min(v), b.max(v)));
    let span = (max - min).max(1.0);
    let mut out = arm.clone();
    for (i, v) in g.iter().enumerate() {
        out.px[i * 3 + 1] = ((lo + (v - min) / span * (hi - lo)) * 255.0).clamp(0.0, 255.0).round_ties_even() as u8;
    }
    out
}

fn house(src: &Path, dst: &Path, only: &[&str]) -> Result<(), String> {
    for HouseSet { name, kind, src: d, albedo: asize, normal: nsize, grey, ops } in HOUSE_SETS {
        if !only.is_empty() && !only.contains(&name) {
            continue;
        }
        let out = dst.join(name);
        mkdir(&out)?;
        let (mut alb, mut nor, mut arm) = load_maps(src, kind, d)?;
        for &op in ops {
            alb = orient(alb, op, false);
            nor = orient(nor, op, true);
            arm = orient(arm, op, false);
        }
        if let Some((target, contrast, flatten)) = grey {
            alb = grey_detail(&alb, target, contrast, flatten);
        }
        if let Some(&(_, lo, hi)) = ROUGH.iter().find(|r| r.0 == name) {
            arm = remap_rough(&arm, lo, hi);
        }
        save_house(&alb, &out.join("albedo.jpg"), asize, 85)?;
        save_house(&nor, &out.join("normal.jpg"), nsize, 88)?;
        save_house(&arm, &out.join("arm.jpg"), 512, 85)?;
        let mut sz = 0;
        for e in std::fs::read_dir(&out).map_err(|e| format!("{}: {e}", out.display()))? {
            sz += e.and_then(|e| e.metadata()).map_err(|e| e.to_string())?.len();
        }
        println!("{name:<18} {kind}:{d:<22} {} KB", sz / 1024);
    }
    Ok(())
}

// --- materials.py ---------------------------------------------------------------------------------

/// A JSON value written the way Python's `json.dump(..., indent=2)` writes it, keys in insertion order.
#[derive(Clone, Debug)]
pub enum J {
    Str(&'static str),
    Owned(String),
    Int(i64),
    Float(f64),
    Obj(Vec<(&'static str, J)>),
}

impl J {
    fn set(&mut self, key: &'static str, v: J) {
        let J::Obj(m) = self else { panic!("set on a non-object") };
        match m.iter_mut().find(|(k, _)| *k == key) {
            Some(slot) => slot.1 = v,
            None => m.push((key, v)),
        }
    }

    fn write(&self, out: &mut String, depth: usize) {
        match self {
            J::Str(s) => out.push_str(&crate::content::json_str(s)),
            J::Owned(s) => out.push_str(&crate::content::json_str(s)),
            J::Int(i) => {
                let _ = write!(out, "{i}");
            }
            J::Float(f) => out.push_str(&py_float(*f)),
            J::Obj(m) if m.is_empty() => out.push_str("{}"),
            J::Obj(m) => {
                out.push('{');
                for (i, (k, v)) in m.iter().enumerate() {
                    out.push_str(if i == 0 { "\n" } else { ",\n" });
                    out.push_str(&"  ".repeat(depth + 1));
                    out.push_str(&crate::content::json_str(k));
                    out.push_str(": ");
                    v.write(out, depth + 1);
                }
                out.push('\n');
                out.push_str(&"  ".repeat(depth));
                out.push('}');
            }
        }
    }
}

/// Python's `repr(float)` for the plain values used here (shortest round-trip, always with a point).
pub fn py_float(f: f64) -> String {
    let s = format!("{f}");
    if s.contains(['.', 'e', 'i', 'N']) { s } else { format!("{s}.0") }
}

fn surf(color: &'static str, tex: &str, uv: f64, kw: &[(&'static str, J)]) -> J {
    let mut e = vec![
        ("type", J::Str("material")),
        ("color", J::Str(color)),
        ("texture", J::Owned(format!("textures/{tex}/albedo.jpg"))),
        ("normal", J::Str("normal.jpg")),
        ("arm", J::Str("arm.jpg")),
        ("uvScale", J::Float(uv)),
    ];
    e.extend(kw.iter().cloned());
    J::Obj(e)
}

fn plain(color: &'static str, kw: &[(&'static str, J)]) -> J {
    let mut e = vec![("type", J::Str("material")), ("color", J::Str(color))];
    e.extend(kw.iter().cloned());
    J::Obj(e)
}

/// materials.py's entries, in its order.
pub fn materials() -> J {
    let f = J::Float;
    let i = J::Int;
    let s = J::Str;
    let with = |mut e: J, k: &'static str, v: J| {
        e.set(k, v);
        e
    };
    let mut e = J::Obj(vec![
        ("environment.sky", J::Obj(vec![("type", s("image")), ("url", s("env/belfast_open_field.pcenv"))])),
        ("material.wall", surf("#F4F0E8", "wall_plaster", 0.5, &[("normalStrength", f(0.7))])),
        ("material.floor", surf("#FFFFFF", "floor_laminate", 0.55, &[])),
        ("material.ground", with(surf("#FFFFFF", "grass", 0.3, &[("normalStrength", f(0.8)), ("projection", s("xz"))]), "sideColor", s("#C9B391"))),
        ("material.road", surf("#FFFFFF", "road_asphalt", 0.2, &[("projection", s("xz"))])),
        ("material.path", surf("#FFFFFF", "path_hexpaving", 0.7, &[("projection", s("xz"))])),
        // Farm fields (town.ts `farmstead`): tilled earth, the greyscale grain tinted; furrows darker.
        ("material.soil", surf("#8A6646", "grass", 0.5, &[("projection", s("xz")), ("roughness", i(1))])),
        ("material.furrow", surf("#664A33", "grass", 0.5, &[("projection", s("xz")), ("roughness", i(1))])),
        ("material.finish.matte", plain("#FFFFFF", &[("roughness", f(0.75))])),
        ("material.finish.satin", plain("#FFFFFF", &[("roughness", f(0.42))])),
        ("material.finish.gloss", plain("#FFFFFF", &[("roughness", f(0.12))])),
        ("material.finish.metal", plain("#FFFFFF", &[("roughness", f(0.32)), ("metallic", i(1))])),
        ("material.finish.chrome", plain("#FFFFFF", &[("roughness", f(0.1)), ("metallic", i(1))])),
        ("material.finish.fabric", surf("#FFFFFF", "fabric_linen", 2.5, &[("roughness", i(1)), ("metallic", i(0))])),
        ("material.finish.wood", surf("#FFFFFF", "wood_oak", 1.2, &[("roughness", f(0.8)), ("metallic", i(0))])),
        ("material.finish.bark", surf("#FFFFFF", "bark", 1.0, &[("metallic", i(0))])),
        ("material.finish.foliage", surf("#FFFFFF", "foliage", 0.8, &[("roughness", f(0.9)), ("metallic", i(0))])),
        ("material.finish.glass", plain("#DCE6E8", &[("roughness", f(0.05)), ("alpha", f(0.28))])),
        ("material.finish.skin", plain("#FFFFFF", &[("roughness", f(0.55))])),
    ]);
    // House exterior/interior surfaces (house). uvScale = 1 / real-world tile size in metres; greyscale detail maps
    // (colour #FFFFFF) are tinted by the game per house/room; roughness multiplies the arm map's roughness.
    let house = [
        ("material.wall.siding", surf("#FFFFFF", "siding_wood", 0.4, &[("roughness", i(1)), ("normalStrength", i(1))])),
        ("material.wall.brick", surf("#FFFFFF", "brick", 1.0, &[("roughness", i(1)), ("normalStrength", i(1))])),
        ("material.roof", surf("#FFFFFF", "roof_shingles", 0.667, &[("roughness", i(1)), ("normalStrength", i(1))])),
        ("material.wall.stone", surf("#FFFFFF", "stone_foundation", 0.667, &[("roughness", i(1)), ("normalStrength", i(1))])),
        ("material.floor.wood", surf("#FFFFFF", "floor_wood", 0.53, &[("roughness", i(1)), ("normalStrength", f(0.8))])),
        ("material.floor.tile", surf("#FFFFFF", "floor_tile", 0.5, &[("roughness", f(0.7)), ("normalStrength", i(1))])),
        ("material.floor.carpet", surf("#FFFFFF", "carpet", 1.0, &[("roughness", i(1)), ("normalStrength", f(0.8))])),
        ("material.wall.interior", surf("#FFFFFF", "wallpaper", 0.5, &[("roughness", i(1)), ("normalStrength", f(0.5))])),
        ("material.trim", surf("#FFFFFF", "painted_wood", 1.0, &[("roughness", i(1)), ("normalStrength", f(0.6))])),
    ];
    for (k, v) in house {
        e.set(k, v);
    }
    // Lawn: greyscale, blotch-flattened Grass004 (1.4 m) shown at 2 m for the 10-60 m camera; vertex colours tint it.
    e.set("material.ground", with(surf("#FFFFFF", "grass_lush", 0.5, &[("normalStrength", f(0.8)), ("projection", s("xz"))]), "sideColor", s("#C9B391")));
    // Character and house finishes without textures (glossy hair, window glass).
    e.set("material.finish.hair", plain("#FFFFFF", &[("roughness", f(0.34))]));
    e.set("material.finish.skin", plain("#FFFFFF", &[("roughness", f(0.48))]));
    e.set("material.window", plain("#22303E", &[("roughness", f(0.05)), ("metallic", f(0.1))]));
    e.set("material.window.inner", plain("#D8E8F4", &[("roughness", f(0.25))]));
    e
}

/// `json.dump(E, f, indent=2)`: no trailing newline.
pub fn materials_json() -> String {
    let mut out = String::new();
    materials().write(&mut out, 0);
    out
}

// --- ph_tex.py ------------------------------------------------------------------------------------

const UA: &str = "idyll-lives-asset-fetch/1.0";

fn get(agent: &ureq::Agent, url: &str) -> Result<Vec<u8>, String> {
    let mut resp = agent.get(url).header("User-Agent", UA).call().map_err(|e| format!("{url}: {e}"))?;
    resp.body_mut().with_config().limit(u64::MAX).read_to_vec().map_err(|e| format!("{url}: {e}"))
}

fn ph(out: &Path, res: &str, maps: &str, ids: &[&str]) -> Result<(), String> {
    let t = Some(std::time::Duration::from_secs(60));
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_connect(t).timeout_send_request(t).timeout_recv_response(t).build().into();
    for id in ids {
        let files: serde_json::Value = serde_json::from_slice(&get(&agent, &format!("https://api.polyhaven.com/files/{id}"))?).map_err(|e| format!("{id}: {e}"))?;
        let dir = out.join(id);
        mkdir(&dir)?;
        for m in maps.split(',') {
            let Some(entry) = files.get(m) else {
                println!("missing {id} {m}");
                continue;
            };
            let u = entry.get(res).and_then(|r| r.get("jpg")).and_then(|j| j.get("url")).and_then(|u| u.as_str()).ok_or_else(|| format!("{id}: no {res} jpg of {m}"))?;
            let dst = dir.join(format!("{m}.jpg"));
            let bytes = get(&agent, u)?;
            std::fs::write(&dst, &bytes).map_err(|e| format!("{}: {e}", dst.display()))?;
            println!("{id} {m} {} {u}", bytes.len());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lut_hits_the_target_mean() {
        // A flat mid-grey: the LUT maps it to the sRGB value whose linear value is the target.
        let mut hist = [0u64; 256];
        hist[128] = 100;
        let lut = detail_lut(&hist, 0.8);
        assert_eq!(lut[128], (srgb(0.8) * 255.0).round() as u8);
        assert_eq!(lut[0], 0);
        assert_eq!(lut[255], 255);
        assert!(lut.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn srgb_round_trip() {
        for v in [0.0, 10.0, 128.0, 255.0] {
            assert!((srgb(lin(v)) * 255.0 - v).abs() < 1e-9);
        }
    }

    #[test]
    fn sums_like_numpy_and_python() {
        let a: Vec<f64> = (0..1000).map(|i| (i as f64 * 0.37).sin() * 10f64.powi(i % 9)).collect();
        // Pairwise by blocks of 8 below 128 elements: (r0+r1)+(r2+r3) ...
        assert_eq!(np_sum(&a[..16]), {
            let r: Vec<f64> = (0..8).map(|j| a[j] + a[j + 8]).collect();
            ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]))
        });
        // Python's sum() is compensated: 0.1 ten times is exactly 1.0 there.
        assert_eq!(py_fsum(std::iter::repeat_n(0.1, 10)), 1.0);
    }

    /// `tests/fixtures/tex/materials.json` is what materials.py wrote (but for the sky's URL, since `cargo envconv`'s `.pcenv`).
    #[test]
    fn materials_bytes() {
        assert_eq!(materials_json(), include_str!("../tests/fixtures/tex/materials.json"));
        assert_eq!(py_float(0.667), "0.667");
        assert_eq!(py_float(2.0), "2.0");
    }

    /// clouds.py made the committed cloud layer; the port makes the same pixels.
    #[test]
    fn clouds_match_the_committed_layer() {
        let path = crate::content::root().join("web/public/assets/textures/sky/clouds.png");
        let committed = image::open(path).unwrap().to_rgba8();
        assert_eq!(committed.dimensions(), (CW as u32, CH as u32));
        assert!(committed.into_raw() == clouds_rgba());
    }

    #[test]
    fn stagger_rolls_strips() {
        let im = Img { w: 16, h: 4, ch: 1, px: (0..64).map(|v| v as u8).collect() };
        const CUTS: [usize; 2] = [1, 3];
        let out = orient(im.clone(), Op::Stagger(&CUTS, 7), false);
        // Every row keeps its pixels, rotated.
        for y in 0..4 {
            let mut a = im.px[y * 16..][..16].to_vec();
            let mut b = out.px[y * 16..][..16].to_vec();
            a.sort();
            b.sort();
            assert_eq!(a, b);
        }
    }
}
