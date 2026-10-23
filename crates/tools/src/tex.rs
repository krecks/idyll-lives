//! `cargo tex`: the texture scripts (was `tools/art/*.py`), one subcommand each, with their arguments and paths.
//!
//! - `clouds <out.png>`: the sky's tileable cloud layer (`clouds.py`).
//! - `synth [set...]`: the procedural albedo/normal/arm maps (`synth.rs`); no downloads.
//! - `materials`: writes `materials.json` in the current directory (`materials.py`).
//!
//! The numbers follow numpy and Pillow step by step (float64 as numpy evaluates it, numpy's pairwise sums and
//! Python's compensated `sum()`, Pillow's fixed-point resampling and box blur, libjpeg-turbo's decoder and
//! encoder), so the outputs are the scripts' to the byte; see `pil` and `jpeg`.

mod jpeg;
pub mod pil;
pub mod synth;

use crate::pcg64::Pcg64;
use pil::Img;
use std::fmt::Write as _;
use std::path::Path;

pub const USAGE: &str = "usage: cargo tex <command> ...
  clouds <out.png>                          the sky's tileable cloud layer
  synth [set...]                            procedural albedo/normal/arm maps (no downloads)
  materials                                 writes materials.json (PBR material + environment entries)";
pub fn main_with(args: &[String]) -> i32 {
    let a: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let res = match a.as_slice() {
        ["clouds", out] => clouds(Path::new(out)),
        ["synth", only @ ..] => synth::run(only),
        ["materials"] => std::fs::write("materials.json", materials_json()).map_err(|e| format!("materials.json: {e}")),
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

// --- detail maps ----------------------------------------------------------------------------------

/// `lin`/`srgb` for the detail maps, in Python floats.
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

/// `grey_detail`: luminance-only detail map with linear mean ~`target`; `contrast` < 1
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
        ("environment.sky", J::Obj(vec![("type", s("image")), ("url", s("env/belfast_open_field.env"))])),
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
    // House exterior/interior surfaces. uvScale = 1 / real-world tile size in metres; greyscale detail maps
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

    /// `tests/fixtures/tex/materials.json` is what materials.py wrote.
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

}
