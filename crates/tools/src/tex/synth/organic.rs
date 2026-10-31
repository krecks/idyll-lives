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
/// `lines` vertical furrows across the tile, each wandering sideways along v (far enough that
/// neighbours meet) and swelling and pinching in width (up to `width` px). Returns the furrow mask
/// as a function of a pixel jitter of its edge, and the distance in px from the nearest furrow's
/// edge (negative inside).
fn furrows(
    n: &Noise,
    u: f64,
    v: f64,
    size: usize,
    lines: i64,
    width: f64,
    salt: f64,
) -> (impl Fn(f64) -> f64, f64) {
    let spacing = size as f64 / lines as f64;
    let xs = u * lines as f64;
    let mut edge = f64::INFINITY;
    let mut wid = 1.0;
    for k in xs.floor() as i64 - 3..=xs.floor() as i64 + 3 {
        let kw = k.rem_euclid(lines) as f64 + salt;
        let base = (n.rand(k.rem_euclid(lines), salt as i64, 71) - 0.5) * 0.5;
        let wander = 0.9 * line_noise(n, v, 2, kw, 2) + 0.35 * line_noise(n, v, 7, kw + 50.0, 3);
        let d = (xs - (k as f64 + 0.5 + base + wander)).abs() * spacing;
        let w = width * smoothstep(-0.35, 0.75, line_noise(n, v, 6, kw + 25.0, 2));
        if d - w < edge {
            edge = d - w;
            wid = w;
        }
    }
    let mask = move |jag: f64| 1.0 - smoothstep(-0.6 * wid, 0.8, edge + jag * 0.5);
    (mask, edge)
}

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
        "bark" => bark(&n, size),
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

// --- the organic generators -------------------------------------------------------------------------

/// The lawn (`grass_lush`, 2 m a tile): tufts of short blades leaning every way over dark thatch,
/// the upper blades lighter, the gaps in shade, the tufts a little uneven in vigour.
fn lawn(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let px_m = size as f64 / 2.0;
    let k = size as f64 / 1024.0;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| {
        let t = n.fbm(u, v, 64, 3);
        (0.04 * t, 0.42 + 0.20 * t, 0.95)
    });
    // Dead blades lying in the thatch: pale, low.
    for _ in 0..(5000.0 * k * k) as usize {
        let b = Blade {
            dir: r.f() * TAU,
            len: r.range(0.02, 0.05) * px_m,
            half: r.range(1.0, 1.6) * k,
            bend: r.range(-0.15, 0.15),
            z0: 0.05,
            z1: 0.10,
            tone: r.range(0.60, 0.90),
            rough: 0.9,
        };
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        c.sprite(x, y, b.len + 2.0, |dx, dy| b.frag(dx, dy));
    }
    // Tufts: a handful of blades from a common crown, leaning outwards; one to a jittered cell
    // (about 1.5 cm), so the sward is even with no bare patches.
    let cells = (130.0 * k) as i64;
    let cell = size as f64 / cells as f64;
    for j in 0..cells {
        for i in 0..cells {
            let x = (i as f64 + 0.5 + r.gauss() * 0.35) * cell;
            let y = (j as f64 + 0.5 + r.gauss() * 0.35) * cell;
            let (u, v) = (x / size as f64, y / size as f64);
            let patch = n.fbm(u, v, 24, 3) - 0.5;
            let lean = r.f() * TAU;
            let m = 4 + (r.f() * 5.0) as usize;
            let crown = r.range(0.20, 0.35);
            let vigour = r.range(0.88, 1.10) * (1.0 + 0.4 * patch);
            for _ in 0..m {
                let dir = lean + r.gauss() * 1.0;
                let len = r.range(0.016, 0.040) * px_m * vigour;
                let b = Blade {
                    dir,
                    len,
                    half: r.range(1.3, 2.1) * k,
                    bend: r.gauss() * 0.15,
                    z0: crown,
                    z1: crown + r.range(0.25, 0.70) * vigour,
                    tone: r.range(0.74, 1.0) * vigour,
                    rough: r.range(0.62, 0.80),
                };
                let (ox, oy) = (r.gauss() * 1.5 * k, r.gauss() * 1.5 * k);
                c.sprite(x + ox, y + oy, len + 3.0, |dx, dy| b.frag(dx, dy));
            }
        }
    }
    finish_layered(c, 0.30, &[(2, 1.2), (6, 0.6)], 0.40)
}

/// Turns a sprite canvas into maps: the tone darkened with depth (light reaches less far down a
/// sward or a crown; `depth_dark` is the share lost at the bottom), AO from depth and cavities.
fn finish_layered(c: Canvas, depth_dark: f64, cav: &[(usize, f64)], ao_floor: f64) -> Organic {
    let (w, h) = (c.w, c.h);
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for &z in &c.z {
        lo = lo.min(z);
        hi = hi.max(z);
    }
    let occ = cavity(&c.z, w, h, cav);
    let mut tone = vec![0.0; w * h];
    let mut ao = vec![0.0; w * h];
    for i in 0..w * h {
        let d = ((c.z[i] - lo) / (hi - lo).max(1e-9)).clamp(0.0, 1.0);
        let lit = (1.0 - depth_dark) + depth_dark * d.powf(0.8);
        tone[i] = (c.tone[i] * lit * (1.0 - 0.6 * occ[i].min(1.0))).clamp(0.0, 1.0);
        ao[i] = (ao_floor + (1.0 - ao_floor) * d.powf(0.6) - 0.5 * occ[i]).clamp(0.05, 1.0);
    }
    Organic {
        height: HeightMap { w, h, v: c.z },
        albedo: grey_img(&tone, w, h),
        ao,
        rough: c.rough,
    }
}

/// Field earth (`grass`, 2 m a tile, tinted brown by the game for soil and furrows): crumbly soil
/// cracked into irregular clods, strewn with litter (flakes of dead leaf and husk, bits of straw)
/// and grit, the hollows in shade.
fn earth(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let k = size as f64 / 1024.0;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| {
        let (wu, wv) = n.warp(u, v, 0.03, 12);
        // Clods: cells of a warped Voronoi, cracked apart, each heaped unevenly.
        let (d1, d2, ci, cj) = worley(n, wu, wv, 36, 36, 0.95, 3);
        let open = smoothstep(0.45, 0.65, n.fbm(u, v, 6, 3));
        let crack = 1.0 - (1.0 - smoothstep(0.0, 0.12, d2 - d1)) * open;
        let heap = 0.5 + 0.5 * n.rand(ci, cj, 5);
        let clod = crack * (0.6 + 0.4 * (1.0 - d1).max(0.0)) * heap;
        // Crumbs: turbulent noise, cut into bits.
        let t1 = (n.fbm(wu, wv, 96, 3) - 0.5).abs() * 2.0;
        let t2 = (n.fbm(u, v, 220, 2) - 0.5).abs() * 2.0;
        let crumbs = smoothstep(0.05, 0.35, t1) * 0.6 + smoothstep(0.05, 0.4, t2) * 0.4;
        let lumpy = n.fbm(u, v, 10, 4);
        let z = 0.22 * clod + 0.30 * crumbs + 0.30 * lumpy;
        let tone = 0.56 + 0.12 * (crumbs - 0.5) + 0.14 * (n.fbm(u, v, 20, 3) - 0.5) + 0.06 * heap
            - 0.10 * (1.0 - crack);
        (z, tone, 0.90 + 0.08 * crumbs)
    });
    // Litter: small flat flakes, some pale (husk, dry leaf), some dark (rotting).
    for _ in 0..(26000.0 * k * k) as usize {
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let (ang, len) = (r.f() * PI, r.range(2.0, 7.0) * k);
        let wid = len * r.range(0.3, 0.8);
        let tone = if r.f() < 0.7 {
            r.range(0.70, 1.0)
        } else {
            r.range(0.25, 0.45)
        };
        let z0 = r.range(0.45, 0.75);
        let (tx, ty) = (r.gauss() * 0.04, r.gauss() * 0.04);
        let rag = r.f() * 10.0;
        c.sprite(x, y, len + 2.0, |dx, dy| {
            let (ca, sa) = (ang.cos(), ang.sin());
            let (lx, ly) = (dx * ca + dy * sa, -dx * sa + dy * ca);
            // A ragged outline.
            let edge = 1.0 + 0.25 * ((lx * 1.7 + rag).sin() * (ly * 2.3 - rag).cos());
            let d = ((lx / len).powi(2) + (ly / wid).powi(2)).sqrt() / edge;
            let a = ((1.0 - d) * wid + 0.5).clamp(0.0, 1.0);
            Some(Frag {
                z: z0 + tx * lx + ty * ly,
                tone: tone * (0.9 + 0.1 * (1.0 - d)),
                rough: 0.85,
                a,
            })
        });
    }
    // Straw and dead stalks lying on the soil.
    for _ in 0..(700.0 * k * k) as usize {
        let b = Blade {
            dir: r.f() * TAU,
            len: r.range(12.0, 40.0) * k,
            half: r.range(1.0, 1.7) * k,
            bend: r.gauss() * 0.12,
            z0: r.range(0.55, 0.80),
            z1: r.range(0.55, 0.80),
            tone: r.range(0.85, 1.05),
            rough: 0.75,
        };
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        c.sprite(x, y, b.len + 2.0, |dx, dy| b.frag(dx, dy));
    }
    // Grit: small pale stones.
    for _ in 0..(900.0 * k * k) as usize {
        let rad = r.range(1.0, 3.0) * k;
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let tone = r.range(0.75, 1.0);
        c.sprite(x, y, rad + 1.0, |dx, dy| {
            let d = (dx * dx + dy * dy).sqrt() / rad;
            let a = ((1.0 - d) * rad + 0.5).clamp(0.0, 1.0);
            Some(Frag {
                z: 0.55 + 0.25 * (1.0 - d * d).max(0.0).sqrt(),
                tone,
                rough: 0.6,
                a,
            })
        });
    }
    let occ = cavity(&c.z, size, size, &[(1, 2.0), (3, 2.0), (8, 1.0)]);
    let tone: Vec<f64> = c
        .tone
        .iter()
        .zip(&occ)
        .map(|(&t, &o)| t * (1.0 - 0.75 * o.min(1.0)))
        .collect();
    let ao = occ
        .iter()
        .map(|&o| (1.0 - 0.9 * o).clamp(0.2, 1.0))
        .collect();
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: c.z,
        },
        albedo: grey_img(&tone, size, size),
        ao,
        rough: c.rough,
    }
}

/// Tree foliage (1.25 m a tile, on the crowns): layers of small overlapping leaves at every angle,
/// each tilted its own way, the inner ones darker, the gaps in shade.
fn foliage(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let px_m = size as f64 / 1.25;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| (0.0, 0.50 + 0.10 * n.fbm(u, v, 16, 3), 0.9));
    let k = size as f64 / 512.0;
    // Leaves in sprays: a few to a twig, pointing away from it.
    for _ in 0..(7000.0 * k * k) as usize {
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let depth = r.f();
        let out = r.f() * TAU;
        let m = 3 + (r.f() * 4.0) as usize;
        let tone = r.range(0.70, 1.0);
        for _ in 0..m {
            let dir = out + r.gauss() * 0.8;
            let len = r.range(0.020, 0.034) * px_m;
            let leaf = Leaf {
                dir,
                len,
                wid: len * r.range(0.32, 0.46),
                zc: depth + r.range(0.0, 0.12),
                tilt: (r.gauss() * 0.020 / k, r.gauss() * 0.030 / k),
                fold: r.range(0.004, 0.016) / k,
                tone: tone * r.range(0.85, 1.12),
            };
            let off = len * r.range(0.35, 0.6);
            let (cx, cy) = (x + dir.cos() * off, y + dir.sin() * off);
            c.sprite(cx, cy, len * 0.55 + 2.0, |dx, dy| leaf.frag(dx, dy));
        }
    }
    finish_layered(c, 0.35, &[(2, 1.6), (5, 0.8)], 0.35)
}

/// Bark (1 m a tile, on the trunks, rings round the trunk along u): long vertical fissures that
/// wander, pinch and meet (so the ridges between them braid), deep and dark, between rounded ridges
/// with a fibrous, cross-checked surface; grey where the ridges weather, warm brown towards the
/// fissures, moss and lichen in patches.
fn bark(n: &Noise, size: usize) -> Organic {
    let px = 1.0 / size as f64;
    let mut z = vec![0.0; size * size];
    let mut rough = vec![0.0; size * size];
    let mut cols = vec![[0.0f64; 3]; size * size];
    for y in 0..size {
        let v = (y as f64 + 0.5) * px;
        for x in 0..size {
            let u = (x as f64 + 0.5) * px;
            let jag = 1.5 * (n.fbm2(u, v, 48, 24, 3) - 0.5) / 0.1;
            let (f1, e1) = furrows(n, u, v, size, 30, 3.6, 0.0);
            let (f2, _) = furrows(n, u, v, size, 64, 2.0, 100.0);
            let f1 = f1(jag);
            // The finer furrows come and go; the finest crazing is a ridged noise.
            let f2 = f2(jag * 0.5) * smoothstep(0.35, 0.55, n.fbm2(u, v, 8, 4, 3));
            let craze = n.fbm2(u, v, 80, 20, 3) - 0.5;
            let f3 = (1.0 - smoothstep(0.0, 0.035, craze.abs()))
                * smoothstep(0.4, 0.6, n.fbm2(u, v, 12, 6, 2));
            let e1 = e1 + jag;
            // Cross checks: short horizontal cracks over the ridges, here and there.
            let cc = n.fbm2(u, v, 10, 72, 3) - 0.5;
            let check = (1.0 - smoothstep(0.0, 0.025, cc.abs()))
                * smoothstep(0.58, 0.70, n.fbm2(u, v, 16, 10, 3))
                * smoothstep(1.0, 4.0, e1);
            let fibre = n.fbm2(u, v, 192, 16, 3) - 0.5;
            let grit = n.fbm(u, v, 128, 2) - 0.5;
            let lump = n.fbm2(u, v, 24, 6, 3) - 0.5;
            // Ridges round over from the fissure's lip to a flattish top.
            let crown = smoothstep(0.0, 8.0, e1);
            let h = 0.10 + 0.45 * (1.0 - f1) + 0.25 * crown - 0.22 * f2 - 0.15 * check - 0.10 * f3
                + 0.10 * fibre
                + 0.05 * grit
                + 0.10 * lump;
            let i = y * size + x;
            z[i] = h;
            let open = (1.0 - f1) * (1.0 - 0.75 * f2) * (1.0 - 0.6 * check) * (1.0 - 0.45 * f3);
            // Colour: a broad drift between grey and warm brown, paler on the crowns.
            let drift = n.fbm2(u, v, 3, 2, 3);
            let warm = n.fbm2(u + 0.3, v, 5, 3, 3);
            let mut top = mix3(
                rgb(104, 86, 66),
                rgb(142, 134, 120),
                smoothstep(0.38, 0.62, drift),
            );
            top = mix3(top, rgb(134, 98, 66), 0.6 * smoothstep(0.55, 0.70, warm));
            top = mix3(
                top,
                rgb(166, 158, 144),
                0.30 * crown * smoothstep(0.45, 0.6, drift),
            );
            let mut col = mix3(rgb(28, 20, 15), top, open.powf(1.2) * (0.6 + 0.4 * crown));
            col = scale3(col, 0.92 + 0.30 * fibre + 0.18 * grit + 0.40 * lump);
            // Moss: green beside the fissures, in patches.
            let mp = smoothstep(0.50, 0.66, n.fbm2(u, v, 4, 2, 4));
            let moss = mp * (0.35 + 0.65 * (1.0 - crown)) * (0.7 + 0.8 * fibre).clamp(0.0, 1.0);
            col = mix3(col, scale3(rgb(78, 92, 40), 0.8 + 0.4 * grit), moss * 0.75);
            // Lichen: pale grey-green flecks on the ridges.
            let lichen = smoothstep(0.60, 0.75, n.fbm(u, v, 10, 4)) * open * crown;
            col = mix3(col, rgb(160, 164, 144), lichen * 0.45);
            cols[i] = col;
            rough[i] = 0.84 + 0.12 * (1.0 - open) - 0.06 * lichen + 0.04 * grit;
        }
    }
    let occ = cavity(&z, size, size, &[(2, 1.5), (6, 1.0)]);
    let mut img = vec![0u8; size * size * 3];
    let mut ao = vec![0.0; size * size];
    for i in 0..size * size {
        let o = occ[i].min(1.0);
        let c = scale3(cols[i], 1.14 * (1.0 - 0.3 * o));
        img[i * 3] = to_u8(c[0]);
        img[i * 3 + 1] = to_u8(c[1]);
        img[i * 3 + 2] = to_u8(c[2]);
        ao[i] =
            ((1.0 - 0.8 * occ[i]) * (0.40 + 0.60 * smoothstep(0.08, 0.55, z[i]))).clamp(0.1, 1.0);
    }
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: z,
        },
        albedo: Img {
            w: size,
            h: size,
            ch: 3,
            px: img,
        },
        ao,
        rough,
    }
}

fn rgb(r: u8, g: u8, b: u8) -> [f64; 3] {
    [r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0]
}

fn mix3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

fn scale3(c: [f64; 3], k: f64) -> [f64; 3] {
    [c[0] * k, c[1] * k, c[2] * k]
}

/// Frieze carpet (1 m a tile): twisted yarn ends, each tuft's strands splayed about a lean that
/// drifts across the carpet (the pile's shading), the tips catching the light.
fn carpet(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let k = size as f64 / 512.0;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| (0.0, 0.50 + 0.1 * n.fbm(u, v, 64, 2), 0.95));
    // Tufts on a jittered grid (one each ~6 mm).
    let cells = (110.0 * k) as i64;
    for j in 0..cells {
        for i in 0..cells {
            let (x, y) = (
                (i as f64 + 0.5 + r.gauss() * 0.35) * size as f64 / cells as f64,
                (j as f64 + 0.5 + r.gauss() * 0.35) * size as f64 / cells as f64,
            );
            let (u, v) = (x / size as f64, y / size as f64);
            let lean = n.fbm(u, v, 3, 3) * TAU * 2.0 + r.gauss() * 0.35;
            let tip = r.range(0.75, 1.0);
            let tone = r.range(0.80, 1.0);
            for _ in 0..6 {
                let dir = lean + r.gauss() * 0.5;
                let len = r.range(3.5, 8.0) * k;
                let b = Blade {
                    dir,
                    len,
                    half: r.range(0.7, 1.1) * k,
                    bend: r.gauss() * 0.30,
                    z0: tip - 0.40,
                    z1: tip * r.range(0.85, 1.0),
                    tone: tone * r.range(0.88, 1.08),
                    rough: 0.95,
                };
                let (ox, oy) = (r.gauss() * 1.5 * k, r.gauss() * 1.5 * k);
                c.sprite(x + ox, y + oy, len + 2.0, |dx, dy| b.frag(dx, dy));
            }
        }
    }
    finish_layered(c, 0.30, &[(1, 1.0), (3, 0.6)], 0.45)
}

/// Linen (0.4 m a tile): a plain weave of slubby threads, each a little uneven in tone and not quite
/// straight, the slubs (short thick stretches) standing out paler, a fuzz of loose fibre over all.
fn linen(n: &Noise, size: usize) -> Organic {
    let threads = 150i64;
    let tn = threads as f64;
    let mut z = vec![0.0; size * size];
    let mut tone = vec![0.0; size * size];
    let mut rough = vec![0.0; size * size];
    for y in 0..size {
        let v = (y as f64 + 0.5) / size as f64;
        for x in 0..size {
            let u = (x as f64 + 0.5) / size as f64;
            // Warp threads run along v (at integer x), weft along u; both drift a little.
            // Spacing uneven too (the yarn is), so the weave is no perfect grid.
            let xs =
                u * tn + 0.25 * line_noise(n, v, 3, 0.0, 2) + 0.6 * line_noise(n, u, 37, 7.0, 1);
            let ys =
                v * tn + 0.25 * line_noise(n, u, 3, 1.0, 2) + 0.6 * line_noise(n, v, 37, 9.0, 1);
            let (i, j) = (xs.floor() as i64, ys.floor() as i64);
            let (wi, wj) = (i.rem_euclid(threads), j.rem_euclid(threads));
            // A thread's cross-section where it is `off` from its middle, and how slubby it is here.
            let thread = |idx: i64, along: f64, off: f64, salt: f64| {
                let s = line_noise(n, along, 20, idx as f64 + salt, 2);
                let slub = smoothstep(0.35, 0.9, s);
                let thin = smoothstep(-0.4, -0.9, s);
                let thick = 0.42 * (1.0 + 0.40 * slub - 0.25 * thin);
                let d = (off / thick).abs();
                let streak = line_noise(n, along, 6, idx as f64 + salt + 250.0, 2);
                (
                    (1.0 - d * d).max(0.0).sqrt(),
                    slub - 0.5 * thin + 0.6 * streak,
                )
            };
            let (pw, sw) = thread(wi, v, xs - i as f64 - 0.5, 0.0);
            let (pf, sf) = thread(wj, u, ys - j as f64 - 0.5, 500.0);
            let sign_i = if wi % 2 == 0 { 1.0 } else { -1.0 };
            let sign_j = if wj % 2 == 0 { 1.0 } else { -1.0 };
            let warp_h = pw * (0.62 + 0.38 * sign_i * (PI * ys).sin()) * (1.0 + 0.2 * sw);
            let weft_h = pf * (0.62 - 0.38 * sign_j * (PI * xs).sin()) * (1.0 + 0.2 * sf);
            let fuzz = n.fbm(u, v, 256, 2) - 0.5;
            let i_ = y * size + x;
            let (h, t) = if warp_h >= weft_h {
                (
                    warp_h,
                    0.62 + 0.04 * (n.rand(wi, 1, 11) - 0.5) + 0.10 * sw + 0.05 * pw,
                )
            } else {
                (
                    weft_h,
                    0.60 + 0.04 * (n.rand(wj, 2, 11) - 0.5) + 0.10 * sf + 0.05 * pf,
                )
            };
            z[i_] = h + 0.06 * fuzz;
            tone[i_] = t + 0.05 * fuzz + 0.03 * (n.fbm(u, v, 12, 3) - 0.5);
            rough[i_] = 0.78 + 0.14 * (1.0 - h.clamp(0.0, 1.0));
        }
    }
    let occ = cavity(&z, size, size, &[(1, 1.0), (3, 0.5)]);
    let ao = occ
        .iter()
        .map(|&o| (1.0 - 0.6 * o).clamp(0.5, 1.0))
        .collect();
    let tone: Vec<f64> = tone
        .iter()
        .zip(&occ)
        .map(|(&t, &o)| t * (1.0 - 0.12 * o.min(1.0)))
        .collect();
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: z,
        },
        albedo: grey_img(&tone, size, size),
        ao,
        rough,
    }
}

/// Wallpaper (2 m a tile; the game draws its stripes over it): ingrain paper, fine fibres and the
/// odd embedded chip under a coat of paint that has dried a little unevenly.
fn wallpaper(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let k = size as f64 / 1024.0;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| {
        let (wu, wv) = n.warp(u, v, 0.04, 8);
        let mottle = n.fbm(wu, wv, 6, 5) - 0.5;
        let paper = n.fbm(u, v, 192, 3) - 0.5;
        let cloud = n.fbm(u, v, 24, 3) - 0.5;
        (
            0.30 + 0.08 * paper,
            0.70 + 0.10 * mottle + 0.04 * cloud + 0.05 * paper,
            0.82 + 0.08 * mottle,
        )
    });
    // Ingrain chips: small flat flakes under the paint.
    for _ in 0..(26000.0 * k * k) as usize {
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let (ang, len, wid) = (r.f() * PI, r.range(1.5, 4.0) * k, r.range(0.8, 1.8) * k);
        let lift = r.range(0.10, 0.30);
        c.sprite(x, y, len + 2.0, |dx, dy| {
            let (ca, sa) = (ang.cos(), ang.sin());
            let (lx, ly) = (dx * ca + dy * sa, -dx * sa + dy * ca);
            let d = ((lx / len).powi(2) + (ly / wid).powi(2)).sqrt();
            let a = ((1.0 - d) * wid + 0.5).clamp(0.0, 1.0);
            Some(Frag {
                z: 0.30 + lift * (1.0 - d * d).max(0.0).sqrt(),
                tone: 0.70,
                rough: 0.78,
                a,
            })
        });
    }
    let occ = cavity(&c.z, size, size, &[(1, 1.5), (3, 0.8)]);
    let ao = occ
        .iter()
        .map(|&o| (1.0 - 0.6 * o).clamp(0.5, 1.0))
        .collect();
    let tone: Vec<f64> = c
        .tone
        .iter()
        .zip(&occ)
        .map(|(&t, &o)| t * (1.0 - 0.15 * o.min(1.0)))
        .collect();
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: c.z,
        },
        albedo: grey_img(&tone, size, size),
        ao,
        rough: c.rough,
    }
}

/// Painted wood (1 m a tile, on trims): boards' grain showing through satin paint, brush marks
/// along it, paint cracked along the odd grain line, a few knots bleeding through, dents and dings.
fn painted_wood(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let k = size as f64 / 512.0;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| {
        // Grain along u: fine, slightly wavy lines, the latewood a little raised.
        let (wu, wv) = n.warp(u, v, 0.03, 2);
        let phase = wv * 46.0 + 2.5 * n.fbm2(wu, wv, 2, 3, 4);
        let late = smoothstep(0.55, 0.95, 0.5 + 0.5 * (phase * TAU).sin());
        let brush = n.fbm2(u, v, 3, 140, 3) - 0.5;
        let fine = n.fbm(u, v, 128, 2) - 0.5;
        // Paint checked along a few grain lines, in broken stretches.
        let line = phase.floor() as i64;
        let along = phase - line as f64 - 0.5;
        let check = (1.0 - smoothstep(0.02, 0.06, along.abs()))
            * smoothstep(
                0.55,
                0.65,
                line_noise(n, u, 3, line.rem_euclid(46) as f64, 2) * 0.5 + 0.5,
            )
            * if n.rand(line.rem_euclid(46), 0, 77) < 0.10 {
                1.0
            } else {
                0.0
            };
        let grime = n.fbm(u, v, 8, 4) - 0.5 + 0.5 * (n.fbm(u, v, 40, 3) - 0.5);
        (
            0.5 + 0.08 * late + 0.10 * brush + 0.03 * fine - 0.25 * check,
            0.72 - 0.012 * late + 0.015 * brush + 0.06 * grime - 0.12 * check,
            0.38 + 0.10 * brush.abs() + 0.05 * late + 0.06 * grime,
        )
    });
    // Knots under the paint: a faint raised ring and a stain.
    for _ in 0..(3.0 * k * k).ceil() as usize {
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let (rx, ry) = (r.range(8.0, 16.0) * k, r.range(4.0, 7.0) * k);
        for yy in (y - ry * 2.0) as i64..=(y + ry * 2.0) as i64 {
            for xx in (x - rx * 2.0) as i64..=(x + rx * 2.0) as i64 {
                let d = (((xx as f64 + 0.5 - x) / rx).powi(2)
                    + ((yy as f64 + 0.5 - y) / ry).powi(2))
                .sqrt();
                let i = yy.rem_euclid(size as i64) as usize * size
                    + xx.rem_euclid(size as i64) as usize;
                let ring = (-((d - 1.0) / 0.25).powi(2)).exp();
                c.z[i] += 0.08 * ring;
                c.tone[i] -= 0.05 * (1.0 - smoothstep(0.4, 1.4, d)) * (0.6 + 0.4 * (d * 9.0).cos());
            }
        }
    }
    // Dents and dings: shallow dimples, the deeper ones with dirt in.
    for _ in 0..(60.0 * k * k) as usize {
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let rad = r.range(1.0, 3.5) * k;
        let depth = r.range(0.08, 0.30);
        c.add(x, y, rad, |d| -depth * (1.0 - d * d));
        let i = (y as usize % size) * size + (x as usize % size);
        c.tone[i] -= depth * 0.3;
    }
    let occ = cavity(&c.z, size, size, &[(1, 1.0), (4, 0.5)]);
    let ao = occ
        .iter()
        .map(|&o| (1.0 - 0.5 * o).clamp(0.6, 1.0))
        .collect();
    let tone: Vec<f64> = c
        .tone
        .iter()
        .zip(&occ)
        .map(|(&t, &o)| t * (1.0 - 0.3 * o.min(1.0)))
        .collect();
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: c.z,
        },
        albedo: grey_img(&tone, size, size),
        ao,
        rough: c.rough,
    }
}

/// Oak (0.83 m a tile, on furniture and panelling): flat-sawn boards glued side by side, the growth
/// rings cut into cathedral arches and straight stripes, the earlywood's open pores in dark dashes.
fn oak(n: &Noise, size: usize) -> Organic {
    // Board edges across u (the tile holds four boards of uneven width).
    let mut edges = [0.0f64; 5];
    let mut acc = 0.0;
    for (b, e) in edges.iter_mut().enumerate().skip(1) {
        acc += 0.8 + 0.4 * n.rand(b as i64, 0, 61);
        *e = acc;
    }
    for e in &mut edges {
        *e /= acc;
    }
    let mut z = vec![0.0; size * size];
    let mut tone = vec![0.0; size * size];
    let mut rough = vec![0.0; size * size];
    for y in 0..size {
        let v = (y as f64 + 0.5) / size as f64;
        for x in 0..size {
            let u = (x as f64 + 0.5) / size as f64;
            let b = (0..4).find(|&b| u < edges[b + 1]).unwrap_or(3);
            let (l, rr) = (edges[b], edges[b + 1]);
            let bi = b as i64;
            // Across the board, in metres from where the pith runs (each board its own cut).
            let width = (rr - l) * 0.83;
            let off = (n.rand(bi, 1, 62) - 0.5) * 0.6;
            let across = ((u - l) / (rr - l) - 0.5 + off) * width;
            // How far out from the pith the cut lies: further at the board's edges (the rings curve
            // away), varying along the board as the log tapers and bends (the arches), and wandering
            // (the log was never a perfect cone), more along the grain than across.
            let curve = 0.06 + 0.10 * n.rand(bi, 2, 63);
            let lean = 0.015 * line_noise(n, v, 1, 10.0 + b as f64, 3);
            let wob = 0.004 * line_noise(n, v, 4, 20.0 + b as f64, 3)
                + 0.07 * (n.fbm2(u, v, 12, 1, 4) - 0.5)
                + 0.006 * (n.fbm2(u, v, 32, 4, 3) - 0.5);
            let rad = 0.2 + across * across / (2.0 * curve) + lean + wob;
            // Growth rings ~6 mm apart (as wide as they can be and still show), their widths uneven.
            let ring_w = 0.005 * (1.0 + 0.35 * line_noise(n, rad * 6.0, 1, 30.0 + b as f64, 2));
            let phase = rad / ring_w;
            let f = phase - phase.floor();
            // Each ring: a dark, porous band of earlywood, then dense latewood paling outwards.
            let early = smoothstep(0.0, 0.06, f) * (1.0 - smoothstep(0.20, 0.38, f));
            let line = 1.0 - smoothstep(0.0, 0.05, f.min(1.0 - f));
            let pores = smoothstep(0.50, 0.70, n.fbm2(u, v, 160, 12, 2)) * early;
            let fleck = n.fbm2(u, v, 64, 4, 3) - 0.5;
            let streak = n.fbm2(u, v, 14, 1, 3) - 0.5;
            let board = (n.rand(bi, 3, 64) - 0.5) * 0.10;
            let t = 0.62 + board - 0.13 * early - 0.06 * line - 0.12 * pores
                + 0.04 * f
                + 0.05 * fleck
                + 0.16 * streak;
            let i = y * size + x;
            // A glue line between boards: a hair darker.
            let joint = (u - l).min(rr - u) * size as f64;
            let seam = 1.0 - smoothstep(0.0, 1.2, joint);
            tone[i] = t * (1.0 - 0.25 * seam);
            z[i] = 0.5 - 0.20 * early - 0.5 * pores + 0.05 * fleck - 0.6 * seam;
            rough[i] = 0.50 + 0.16 * early + 0.12 * pores + 0.05 * fleck;
        }
    }
    let ao = z
        .iter()
        .map(|&h| (0.85 + 0.3 * h).clamp(0.7, 1.0))
        .collect();
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: z,
        },
        albedo: grey_img(&tone, size, size),
        ao,
        rough,
    }
}

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
