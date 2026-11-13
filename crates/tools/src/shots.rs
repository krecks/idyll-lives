//! `cargo shots`: numbers from pictures, so that "the lawn is less even" or "the shade is deeper" is
//! measured and not only said (was `tools/bench/stats.mjs` and `render.mjs --compare`). For the PNGs of
//! shot folders (`tools/bench/render.mjs --shots`, `play.mjs`):
//!
//! ```text
//! cargo shots stats <dir> [--json]            a table of the folder's pictures
//! cargo shots stats <dirA> <dirB> [--json]    the pictures both have: A, B and the change
//! cargo shots steady <dirA> <dirB> [--json]   how much a picture changes when the camera moves a quarter of a pixel
//! cargo shots compare <dirA> <dirB> [--out <dir>]
//!                                             per picture both have: how many pixels differ, by how much
//! ```
//!
//! **stats.** The columns (brightness and contrast in the picture's own 0–255 values, as the screen shows them):
//! - `p5` / `p50` / `p95`: brightness at the 5th, 50th and 95th percentile;
//! - `clip`: % of pixels with a channel at 250 or more; `dark`: % at brightness 5 or less;
//! - `sat`: mean saturation, 0–100;
//! - `even`: % of the frame within a step of its most common tone (brightness and colour in 16 steps each,
//!   after an 8 px blur): a picture that is mostly one lawn scores high;
//! - `fine`: contrast finer than 8 px (texture, noise, edges);
//! - `c8` / `c64` / `c256`: contrast between 8 px blocks inside their 64 px block, 64 px blocks inside their
//!   256 px block, and between 256 px blocks: detail, things, and the picture's big shapes of light.
//!
//! These describe a picture; none of them says it is better. They are for checking what the eye claims.
//!
//! **steady.** A and B are the same build's shots of the same poses, B with `--extra 'tx=0.004&tz=0.004'`
//! (4 mm at the play camera, where a pixel is 16 mm; both, because a pose that sets one of them itself keeps
//! its own). A steady picture hardly changes; detail finer than a pixel (grass blades, a tiling texture far
//! off) comes out different every time, which in play is shimmer as the camera moves, and which one still
//! does not show. Compare the figure before a change and after it.
//!
//! **compare.** Per picture: the share of pixels differing by more than 2 and by more than 16 of 255, the
//! largest difference, the mean one and the PSNR; with `--out`, a picture of where they differ (a dim copy
//! of the first, the differences in red) for each that does.

use serde_json::{Value as J, json};
use std::path::Path;

pub const USAGE: &str = "usage: cargo shots stats <dir> [<dirB>] [--json]
       cargo shots steady <dirA> <dirB> [--json]
       cargo shots compare <dirA> <dirB> [--out <dir>]";

/// A picture's red, green and blue (no alpha) and its brightness (Rec. 709, of the encoded values).
pub struct Picture {
    pub rgb: Vec<u8>,
    pub luma: Vec<f32>,
    pub w: usize,
    pub h: usize,
}

impl Picture {
    pub fn of(rgb: Vec<u8>, w: usize, h: usize) -> Picture {
        let luma = rgb
            .as_chunks::<3>()
            .0
            .iter()
            .map(|p| (0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64) as f32)
            .collect();
        Picture { rgb, luma, w, h }
    }

    pub fn load(file: &Path) -> Result<Picture, String> {
        let image = image::open(file)
            .map_err(|e| format!("{}: {e}", file.display()))?
            .to_rgb8();
        let (w, h) = (image.width() as usize, image.height() as usize);
        Ok(Picture::of(image.into_raw(), w, h))
    }
}

/// Block means of `src` (w × h) in blocks of `size`; the last row and column of blocks may be narrower.
struct Blocks {
    mean: Vec<f32>,
    w: usize,
    h: usize,
}

fn blocks(src: &[f32], w: usize, h: usize, size: usize) -> Blocks {
    let (bw, bh) = (w.div_ceil(size), h.div_ceil(size));
    let mut sum = vec![0f64; bw * bh];
    let mut count = vec![0u32; bw * bh];
    for y in 0..h {
        let row = (y / size) * bw;
        for x in 0..w {
            let b = row + x / size;
            sum[b] += src[y * w + x] as f64;
            count[b] += 1;
        }
    }
    Blocks {
        mean: sum
            .iter()
            .zip(&count)
            .map(|(s, c)| (s / *c as f64) as f32)
            .collect(),
        w: bw,
        h: bh,
    }
}

/// RMS of `fine` minus the block of `coarse` it lies in (`ratio`: fine blocks per coarse block, each way;
/// none: one block for all).
fn band(
    fine: &[f32],
    fw: usize,
    fh: usize,
    coarse: &[f32],
    coarse_w: usize,
    ratio: Option<usize>,
) -> f64 {
    let mut s = 0f64;
    for y in 0..fh {
        let row = ratio.map_or(0, |r| (y / r) * coarse_w);
        for x in 0..fw {
            let d = fine[y * fw + x] as f64 - coarse[row + ratio.map_or(0, |r| x / r)] as f64;
            s += d * d;
        }
    }
    (s / (fw * fh) as f64).sqrt()
}

/// A brightness as the whole number it is nearest (a half up), 255 at most.
fn level(v: f32) -> usize {
    ((v as f64 + 0.5).floor() as usize).min(255)
}

/// What `stats` says of a picture (the module's header).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub p5: u32,
    pub p50: u32,
    pub p95: u32,
    pub clip: f64,
    pub dark: f64,
    pub sat: f64,
    pub even: f64,
    pub fine: f64,
    pub c8: f64,
    pub c64: f64,
    pub c256: f64,
}

const COLUMNS: [&str; 11] = [
    "p5", "p50", "p95", "clip", "dark", "sat", "even", "fine", "c8", "c64", "c256",
];

impl Stats {
    fn column(&self, c: &str) -> f64 {
        match c {
            "p5" => self.p5 as f64,
            "p50" => self.p50 as f64,
            "p95" => self.p95 as f64,
            "clip" => self.clip,
            "dark" => self.dark,
            "sat" => self.sat,
            "even" => self.even,
            "fine" => self.fine,
            "c8" => self.c8,
            "c64" => self.c64,
            _ => self.c256,
        }
    }

    /// The columns as JSON, in the order the JavaScript tool wrote them (the percentiles and `dark` first).
    fn json(&self) -> Vec<(String, Out)> {
        [
            "p5", "p50", "p95", "dark", "clip", "sat", "even", "fine", "c8", "c64", "c256",
        ]
        .iter()
        .map(|c| (c.to_string(), Out::Value(number(self.column(c)))))
        .collect()
    }
}

/// A number as JSON has it from JavaScript: a whole one without a point.
fn number(v: f64) -> J {
    if v.fract() == 0.0 && v.abs() < 9e15 {
        json!(v as i64)
    } else {
        json!(v)
    }
}

pub fn measure(p: &Picture) -> Stats {
    let (w, h, n) = (p.w, p.h, p.w * p.h);
    let mut hist = [0u32; 256];
    for &v in &p.luma {
        hist[level(v)] += 1;
    }
    let at = |q: f64| -> u32 {
        let mut acc = 0u64;
        for (v, count) in hist.iter().enumerate() {
            acc += *count as u64;
            if acc as f64 >= q * n as f64 {
                return v as u32;
            }
        }
        255
    };
    let mut clip = 0usize;
    let mut sat = 0f64;
    for px in p.rgb.as_chunks::<3>().0 {
        let max = px[0].max(px[1]).max(px[2]);
        let min = px[0].min(px[1]).min(px[2]);
        if max >= 250 {
            clip += 1;
        }
        if max > 0 {
            sat += (max - min) as f64 / max as f64;
        }
    }
    // The most common tone: brightness and the two colour differences of the 8 px blocks, in 16 steps each.
    let channel = |c: usize| {
        let one: Vec<f32> = (0..n).map(|i| p.rgb[i * 3 + c] as f32).collect();
        blocks(&one, w, h, 8).mean
    };
    let (r, g, b) = (channel(0), channel(1), channel(2));
    let step = |v: f64| ((v / 16.0).floor() as i64).clamp(0, 15);
    let tones: Vec<[i64; 3]> = (0..r.len())
        .map(|i| {
            let y = 0.2126 * r[i] as f64 + 0.7152 * g[i] as f64 + 0.0722 * b[i] as f64;
            [
                step(y),
                step((b[i] as f64 - y) / 1.8556 + 128.0),
                step((r[i] as f64 - y) / 1.5748 + 128.0),
            ]
        })
        .collect();
    // (The first tone seen wins a tie, as a Map kept in the order of its first keys gave it.)
    let mut bins: Vec<([i64; 3], usize)> = Vec::new();
    let mut place: std::collections::HashMap<[i64; 3], usize> = std::collections::HashMap::new();
    for t in &tones {
        match place.get(t) {
            Some(&i) => bins[i].1 += 1,
            None => {
                place.insert(*t, bins.len());
                bins.push((*t, 1));
            }
        }
    }
    let mut mode = [0i64; 3];
    let mut most = 0usize;
    for (tone, count) in &bins {
        if *count > most {
            mode = *tone;
            most = *count;
        }
    }
    let even = tones
        .iter()
        .filter(|t| (0..3).all(|k| (t[k] - mode[k]).abs() <= 1))
        .count() as f64
        / tones.len().max(1) as f64;

    let b8 = blocks(&p.luma, w, h, 8);
    let b64 = blocks(&p.luma, w, h, 64);
    let b256 = blocks(&p.luma, w, h, 256);
    let all = [(b256.mean.iter().map(|v| *v as f64).sum::<f64>() / b256.mean.len() as f64) as f32];
    Stats {
        p5: at(0.05),
        p50: at(0.5),
        p95: at(0.95),
        dark: 100.0 * hist[..6].iter().sum::<u32>() as f64 / n as f64,
        clip: 100.0 * clip as f64 / n as f64,
        sat: 100.0 * sat / n as f64,
        even: 100.0 * even,
        fine: band(&p.luma, w, h, &b8.mean, b8.w, Some(8)),
        c8: band(&b8.mean, b8.w, b8.h, &b64.mean, b64.w, Some(8)),
        c64: band(&b64.mean, b64.w, b64.h, &b256.mean, b256.w, Some(4)),
        c256: band(&b256.mean, b256.w, b256.h, &all, 1, None),
    }
}

/// How much a picture changed between two shots: the mean of the brightness difference, its 99th
/// percentile, and the share of pixels changed by more than 8.
pub fn moved(a: &Picture, b: &Picture) -> Result<(f64, u32, f64), String> {
    if a.w != b.w || a.h != b.h {
        return Err("differ in size".into());
    }
    let mut hist = [0u32; 256];
    let mut sum = 0f64;
    for (x, y) in a.luma.iter().zip(&b.luma) {
        let d = (*x as f64 - *y as f64).abs();
        sum += d;
        hist[((d + 0.5).floor() as usize).min(255)] += 1;
    }
    let n = a.luma.len() as f64;
    let mut acc = 0u64;
    let mut p99 = 255;
    for (v, count) in hist.iter().enumerate() {
        acc += *count as u64;
        if acc as f64 >= 0.99 * n {
            p99 = v as u32;
            break;
        }
    }
    Ok((
        sum / n,
        p99,
        100.0 * hist[9..].iter().sum::<u32>() as f64 / n,
    ))
}

/// What `compare` says of two pictures of one size, and the picture of where they differ.
pub struct Difference {
    /// Shares of the pixels (0..1) differing by more than 2 and by more than 16 in a channel.
    pub over2: f64,
    pub over16: f64,
    pub max: u8,
    pub mean: f64,
    /// None: the same.
    pub psnr: Option<f64>,
    pub picture: Vec<u8>,
}

pub fn difference(a: &Picture, b: &Picture) -> Difference {
    let pixels = a.w * a.h;
    let mut picture = vec![0u8; pixels * 3];
    let (mut over2, mut over16, mut max) = (0usize, 0usize, 0u8);
    let (mut sum, mut squares) = (0f64, 0f64);
    for i in 0..pixels {
        let mut d = 0u8;
        for c in 0..3 {
            let e = a.rgb[i * 3 + c].abs_diff(b.rgb[i * 3 + c]);
            d = d.max(e);
            sum += e as f64;
            squares += e as f64 * e as f64;
        }
        if d > 2 {
            over2 += 1;
        }
        if d > 16 {
            over16 += 1;
        }
        max = max.max(d);
        // The difference picture: a dim copy of the first, differences in red.
        let grey =
            ((a.rgb[i * 3] as u32 + a.rgb[i * 3 + 1] as u32 + a.rgb[i * 3 + 2] as u32) / 12) as u8;
        picture[i * 3] = if d > 2 {
            (64 + d as u32 * 8).min(255) as u8
        } else {
            grey
        };
        picture[i * 3 + 1] = if d > 2 { 0 } else { grey };
        picture[i * 3 + 2] = if d > 2 { 0 } else { grey };
    }
    let mse = squares / (pixels * 3) as f64;
    Difference {
        over2: over2 as f64 / pixels as f64,
        over16: over16 as f64 / pixels as f64,
        max,
        mean: sum / (pixels * 3) as f64,
        psnr: (mse > 0.0).then(|| 10.0 * (255.0f64 * 255.0 / mse).log10()),
        picture,
    }
}

/// A folder's PNGs, in order.
fn pngs(dir: &Path) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|f| f.ends_with(".png"))
        .collect();
    names.sort();
    Ok(names)
}

fn shared(a: &Path, b: &Path) -> Result<Vec<String>, String> {
    let other = pngs(b)?;
    Ok(pngs(a)?.into_iter().filter(|f| other.contains(f)).collect())
}

fn digits(c: &str) -> usize {
    if c.starts_with('p') {
        0
    } else if c == "clip" || c == "dark" {
        2
    } else {
        1
    }
}

fn fixed(v: f64, c: &str) -> String {
    format!("{v:.*}", digits(c))
}

fn signed(v: f64, c: &str) -> String {
    format!(
        "{}{v:.*}",
        if v >= 0.0 { "+" } else { "" },
        digits(c).max(1)
    )
}

/// JSON whose objects keep their keys' order (the workspace's `serde_json` sorts them).
enum Out {
    Value(J),
    Object(Vec<(String, Out)>),
}

fn text(v: &str) -> Out {
    Out::Value(json!(v))
}

fn write(out: &mut String, value: &Out, depth: usize) {
    match value {
        Out::Value(v) => out.push_str(&v.to_string()),
        Out::Object(pairs) => {
            out.push('{');
            for (i, (key, v)) in pairs.iter().enumerate() {
                out.push_str(if i == 0 { "\n" } else { ",\n" });
                out.push_str(&" ".repeat(depth + 1));
                out.push_str(&json!(key).to_string());
                out.push_str(": ");
                write(out, v, depth + 1);
            }
            out.push('\n');
            out.push_str(&" ".repeat(depth));
            out.push('}');
        }
    }
}

/// A list of objects as JSON indented by one space (as the JavaScript tool wrote it).
fn pretty(rows: &[Vec<(String, Out)>]) -> String {
    if rows.is_empty() {
        return "[]".into();
    }
    let mut out = String::from("[");
    for (i, row) in rows.iter().enumerate() {
        out.push_str(if i == 0 { "\n " } else { ",\n " });
        // (Each row an object of its own, a level in.)
        let mut one = String::new();
        write(
            &mut one,
            &Out::Object(row.iter().map(|(k, v)| (k.clone(), clone(v))).collect()),
            1,
        );
        out.push_str(&one);
    }
    out.push_str("\n]");
    out
}

fn clone(v: &Out) -> Out {
    match v {
        Out::Value(j) => Out::Value(j.clone()),
        Out::Object(pairs) => {
            Out::Object(pairs.iter().map(|(k, v)| (k.clone(), clone(v))).collect())
        }
    }
}

fn header() -> String {
    format!(
        "| picture | {} |\n|---|{}|",
        COLUMNS.join(" | "),
        COLUMNS.map(|_| "---").join("|")
    )
}

fn stats(dirs: &[&str], as_json: bool) -> Result<(), String> {
    if let [dir] = dirs {
        let dir = Path::new(dir);
        let mut rows = Vec::new();
        for f in pngs(dir)? {
            rows.push((measure(&Picture::load(&dir.join(&f))?), f));
        }
        if as_json {
            let rows: Vec<Vec<(String, Out)>> = rows
                .iter()
                .map(|(s, f)| {
                    let mut row = vec![("picture".to_string(), text(f))];
                    row.extend(s.json());
                    row
                })
                .collect();
            println!("{}", pretty(&rows));
        } else {
            println!("{}", header());
            for (s, f) in rows {
                println!(
                    "| {f} | {} |",
                    COLUMNS.map(|c| fixed(s.column(c), c)).join(" | ")
                );
            }
        }
        return Ok(());
    }
    let (a, b) = (Path::new(dirs[0]), Path::new(dirs[1]));
    let mut rows = Vec::new();
    for f in shared(a, b)? {
        rows.push((
            measure(&Picture::load(&a.join(&f))?),
            measure(&Picture::load(&b.join(&f))?),
            f,
        ));
    }
    if as_json {
        let rows: Vec<Vec<(String, Out)>> = rows
            .iter()
            .map(|(x, y, f)| {
                vec![
                    ("picture".to_string(), text(f)),
                    ("a".to_string(), Out::Object(x.json())),
                    ("b".to_string(), Out::Object(y.json())),
                ]
            })
            .collect();
        println!("{}", pretty(&rows));
    } else {
        println!(
            "A = {}\nB = {}\n\nEach cell: A → B (change).\n",
            dirs[0], dirs[1]
        );
        println!("{}", header());
        for (x, y, f) in rows {
            let cells = COLUMNS.map(|c| {
                format!(
                    "{} → {} ({})",
                    fixed(x.column(c), c),
                    fixed(y.column(c), c),
                    signed(y.column(c) - x.column(c), c)
                )
            });
            println!("| {f} | {} |", cells.join(" | "));
        }
    }
    Ok(())
}

fn steady(a: &str, b: &str, as_json: bool) -> Result<(), String> {
    let (a, b) = (Path::new(a), Path::new(b));
    let mut rows = Vec::new();
    for f in shared(a, b)? {
        let (x, y) = (Picture::load(&a.join(&f))?, Picture::load(&b.join(&f))?);
        let (mean, p99, over8) = moved(&x, &y)
            .map_err(|e| format!("{} and {} {e}", a.join(&f).display(), b.join(&f).display()))?;
        rows.push((f, mean, p99, over8));
    }
    if as_json {
        let rows: Vec<Vec<(String, Out)>> = rows
            .iter()
            .map(|(f, mean, p99, over8)| {
                vec![
                    ("picture".to_string(), text(f)),
                    ("mean".to_string(), Out::Value(number(*mean))),
                    ("p99".to_string(), Out::Value(json!(p99))),
                    ("over8".to_string(), Out::Value(number(*over8))),
                ]
            })
            .collect();
        println!("{}", pretty(&rows));
    } else {
        println!(
            "| picture | mean change | 99th percentile | % of pixels changed by more than 8 |\n|---|---|---|---|"
        );
        for (f, mean, p99, over8) in rows {
            println!("| {f} | {mean:.2} | {p99} | {over8:.2} |");
        }
    }
    Ok(())
}

fn compare(a: &str, b: &str, out: Option<&str>) -> Result<(), String> {
    let (a, b) = (Path::new(a), Path::new(b));
    if let Some(out) = out {
        std::fs::create_dir_all(out).map_err(|e| format!("{out}: {e}"))?;
    }
    let names = shared(a, b)?;
    println!(
        "| picture | pixels differing (>2 / >16) | max | mean | PSNR dB |\n|---|---|---|---|---|"
    );
    let mut worst = 0f64;
    for name in &names {
        let (x, y) = (Picture::load(&a.join(name))?, Picture::load(&b.join(name))?);
        if x.w != y.w || x.h != y.h {
            println!("| {name} | sizes differ | | | |");
            continue;
        }
        let d = difference(&x, &y);
        worst = worst.max(d.over2);
        println!(
            "| {name} | {:.3}% / {:.3}% | {} | {:.3} | {} |",
            100.0 * d.over2,
            100.0 * d.over16,
            d.max,
            d.mean,
            d.psnr.map_or("∞".to_string(), |p| format!("{p:.1}"))
        );
        if let Some(out) = out.filter(|_| d.over2 > 0.0) {
            let file = Path::new(out).join(name);
            image::RgbImage::from_raw(x.w as u32, x.h as u32, d.picture)
                .ok_or("the difference picture has no size")?
                .save(&file)
                .map_err(|e| format!("{}: {e}", file.display()))?;
        }
    }
    println!(
        "\n{} pictures compared; the most changed has {:.3}% of its pixels off by more than 2.",
        names.len(),
        100.0 * worst
    );
    Ok(())
}

pub fn main_with(args: &[String]) -> i32 {
    let as_json = args.iter().any(|a| a == "--json");
    let out = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str);
    let mut plain: Vec<&str> = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
        } else if a == "--out" {
            skip = true;
        } else if !a.starts_with("--") {
            plain.push(a);
        }
    }
    let result = match plain.as_slice() {
        ["stats", dirs @ ..] if (1..=2).contains(&dirs.len()) => stats(dirs, as_json),
        ["steady", a, b] => steady(a, b, as_json),
        ["compare", a, b] => compare(a, b, out),
        _ => {
            let help = args
                .iter()
                .any(|a| matches!(a.as_str(), "-h" | "--help" | "help"));
            if help {
                println!("{USAGE}");
            } else {
                eprintln!("{USAGE}");
            }
            return if help { 0 } else { 2 };
        }
    };
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("shots: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture of `w` × `h` from a function of the pixel.
    fn made(w: usize, h: usize, f: impl Fn(usize, usize) -> [u8; 3]) -> Picture {
        let mut rgb = Vec::with_capacity(w * h * 3);
        for y in 0..h {
            for x in 0..w {
                rgb.extend(f(x, y));
            }
        }
        Picture::of(rgb, w, h)
    }

    #[test]
    fn a_flat_picture_is_all_one_tone() {
        let s = measure(&made(64, 48, |_, _| [100, 150, 50]));
        // 0.2126 × 100 + 0.7152 × 150 + 0.0722 × 50 = 132.15
        assert_eq!((s.p5, s.p50, s.p95), (132, 132, 132));
        assert_eq!((s.clip, s.dark, s.even), (0.0, 0.0, 100.0));
        assert!((s.sat - 100.0 * 100.0 / 150.0).abs() < 1e-9);
        assert!(s.fine < 1e-4 && s.c8 < 1e-4 && s.c64 < 1e-4 && s.c256 < 1e-4);
    }

    #[test]
    fn contrast_is_counted_at_its_size() {
        // A checkerboard of single pixels: all of it finer than 8 px.
        let fine = measure(&made(256, 256, |x, y| {
            if (x + y) % 2 == 0 { [0; 3] } else { [200; 3] }
        }));
        assert!((fine.fine - 100.0).abs() < 1e-3, "{fine:?}");
        assert!(fine.c8 < 1e-3 && fine.c64 < 1e-3 && fine.c256 < 1e-3);
        // Halves, left dark and right light: 128 px wide, between the 64 px blocks of each 256 px block.
        let halves = measure(&made(
            256,
            256,
            |x, _| if x < 128 { [0; 3] } else { [200; 3] },
        ));
        assert!(halves.fine < 1e-3 && halves.c8 < 1e-3);
        assert!((halves.c64 - 100.0).abs() < 1e-3, "{halves:?}");
        assert_eq!((halves.p5, halves.p95), (0, 200));
        // (Half the frame is the first tone seen, the dark one.)
        assert_eq!((halves.dark, halves.even), (50.0, 50.0));
    }

    #[test]
    fn clipping_and_saturation() {
        let s = measure(&made(10, 10, |x, _| {
            if x < 5 { [255, 0, 0] } else { [10, 10, 10] }
        }));
        assert_eq!(s.clip, 50.0);
        assert_eq!(s.sat, 50.0);
    }

    #[test]
    fn a_moved_picture() {
        let a = made(100, 10, |_, _| [100; 3]);
        let b = made(100, 10, |x, _| if x < 2 { [120; 3] } else { [101; 3] });
        let (mean, p99, over8) = moved(&a, &b).unwrap();
        assert!((mean - (0.02 * 20.0 + 0.98 * 1.0)).abs() < 1e-4);
        assert_eq!(p99, 20);
        assert!((over8 - 2.0).abs() < 1e-9);
        assert!(moved(&a, &made(10, 10, |_, _| [0; 3])).is_err());
    }

    #[test]
    fn two_pictures_compared() {
        let a = made(10, 10, |_, _| [120, 60, 30]);
        let same = difference(&a, &a);
        assert_eq!((same.over2, same.max, same.psnr), (0.0, 0, None));
        // (A dim copy where nothing differs.)
        assert_eq!(&same.picture[..3], &[17, 17, 17]);
        let b = made(10, 10, |x, y| {
            if (x, y) == (0, 0) {
                [140, 60, 30]
            } else if (x, y) == (1, 0) {
                [123, 60, 30]
            } else {
                [120, 60, 30]
            }
        });
        let d = difference(&a, &b);
        assert_eq!((d.over2, d.over16, d.max), (0.02, 0.01, 20));
        assert_eq!(&d.picture[..6], &[224, 0, 0, 88, 0, 0]);
        assert!(
            (d.psnr.unwrap() - 10.0 * (255.0f64 * 255.0 / (409.0 / 300.0)).log10()).abs() < 1e-9
        );
    }

    #[test]
    fn numbers_as_they_are_written() {
        assert_eq!(fixed(132.0, "p50"), "132");
        assert_eq!(fixed(1.2345, "clip"), "1.23");
        assert_eq!(fixed(41.26, "even"), "41.3");
        assert_eq!(signed(0.0, "p5"), "+0.0");
        assert_eq!(signed(-1.5, "clip"), "-1.50");
        assert_eq!(number(5.0), json!(5));
        assert_eq!(number(5.5), json!(5.5));
    }
}
