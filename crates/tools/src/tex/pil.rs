//! The few Pillow operations the texture scripts used, computed the way Pillow computes them (libjpeg-turbo's
//! decoder and encoder, its fixed-point resampling, ITU-R 601 greyscale and extended box blur), so the ports make
//! the same pixels from the same files. Checked against Pillow 12.2 / libjpeg-turbo on arm64: byte-identical
//! JPEGs; the bicubic and box-radius maths fuse multiply-adds as clang does there.

use std::path::Path;

/// An 8-bit image: `ch` 1 (Pillow's "L") or 3 ("RGB"), rows top to bottom, channels interleaved.
#[derive(Clone, Debug, PartialEq)]
pub struct Img {
    pub w: usize,
    pub h: usize,
    pub ch: usize,
    pub px: Vec<u8>,
}

/// Pillow's resampling filters (the two the scripts use).
#[derive(Clone, Copy, Debug)]
pub enum Filter {
    Bicubic,
    Lanczos,
}

impl Filter {
    fn support(self) -> f64 {
        match self {
            Filter::Bicubic => 2.0,
            Filter::Lanczos => 3.0,
        }
    }

    /// Pillow's `bicubic_filter` (a = -0.5) and `lanczos_filter`. The bicubic polynomials use fused multiply-adds
    /// where clang contracts Pillow's C (its arm64 wheels); the difference is in the last bit of a weight.
    fn weight(self, x: f64) -> f64 {
        match self {
            Filter::Bicubic => {
                let a = -0.5;
                let x = x.abs();
                if x < 1.0 {
                    ((a + 2.0f64).mul_add(x, -(a + 3.0)) * x).mul_add(x, 1.0)
                } else if x < 2.0 {
                    (x - 5.0).mul_add(x, 8.0).mul_add(x, -4.0) * a
                } else {
                    0.0
                }
            }
            Filter::Lanczos => {
                if (-3.0..3.0).contains(&x) {
                    sinc(x) * sinc(x / 3.0)
                } else {
                    0.0
                }
            }
        }
    }
}

fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    let x = x * std::f64::consts::PI;
    x.sin() / x
}

const PRECISION_BITS: u32 = 32 - 8 - 2;

/// Pillow's `precompute_coeffs` + `normalize_coeffs_8bpc`: per output pixel the first input pixel and the
/// fixed-point weights.
pub fn coeffs(in_size: usize, out_size: usize, filter: Filter) -> Vec<(usize, Vec<i32>)> {
    let scale = in_size as f32 as f64 / out_size as f64;
    let filterscale = scale.max(1.0);
    let support = filter.support() * filterscale;
    let ss = 1.0 / filterscale;
    (0..out_size)
        .map(|xx| {
            let center = (xx as f64 + 0.5) * scale;
            let xmin = ((center - support + 0.5) as i64).max(0) as usize;
            let xmax = ((center + support + 0.5) as i64).min(in_size as i64) as usize - xmin;
            let mut k: Vec<f64> = (0..xmax).map(|x| filter.weight(((x + xmin) as f64 - center + 0.5) * ss)).collect();
            let ww: f64 = k.iter().sum();
            if ww != 0.0 {
                for v in &mut k {
                    *v /= ww;
                }
            }
            let fixed = k.iter().map(|&v| if v < 0.0 { (-0.5 + v * (1 << PRECISION_BITS) as f64) as i32 } else { (0.5 + v * (1 << PRECISION_BITS) as f64) as i32 }).collect();
            (xmin, fixed)
        })
        .collect()
}

fn clip8(v: i32) -> u8 {
    (v >> PRECISION_BITS).clamp(0, 255) as u8
}

impl Img {
    pub fn new(w: usize, h: usize, ch: usize, fill: u8) -> Img {
        Img { w, h, ch, px: vec![fill; w * h * ch] }
    }

    /// Decodes a PNG or JPEG as Pillow's `Image.open` would hold it: greyscale as L, colour as RGB (an alpha
    /// channel is dropped, as `convert('RGB')` and `convert('L')` both would). Baseline JPEGs go through `jpeg`
    /// (libjpeg-turbo's pixels); anything else through the `image` crate, whose JPEG decoder can differ from
    /// libjpeg-turbo's by a few levels.
    pub fn open(path: &Path) -> Result<Img, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        if let Some((w, h, ch, px)) = super::jpeg::decode(&bytes) {
            return Ok(Img { w, h, ch, px });
        }
        let im = image::ImageReader::new(std::io::Cursor::new(&bytes))
            .with_guessed_format()
            .map_err(|e| format!("{}: {e}", path.display()))?
            .decode()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let (w, h) = (im.width() as usize, im.height() as usize);
        Ok(match im {
            image::DynamicImage::ImageLuma8(b) => Img { w, h, ch: 1, px: b.into_raw() },
            image::DynamicImage::ImageLumaA8(_) | image::DynamicImage::ImageLuma16(_) | image::DynamicImage::ImageLumaA16(_) => Img { w, h, ch: 1, px: im.to_luma8().into_raw() },
            other => Img { w, h, ch: 3, px: other.to_rgb8().into_raw() },
        })
    }

    /// `convert('RGB')`.
    pub fn rgb(self) -> Img {
        if self.ch == 3 {
            return self;
        }
        Img { w: self.w, h: self.h, ch: 3, px: self.px.iter().flat_map(|&v| [v, v, v]).collect() }
    }

    /// `convert('L')` (and `ImageOps.grayscale`): ITU-R 601-2 luma in Pillow's fixed point.
    pub fn grey(&self) -> Img {
        if self.ch == 1 {
            return self.clone();
        }
        let px = self.px.chunks_exact(3).map(|p| luma(p[0], p[1], p[2])).collect();
        Img { w: self.w, h: self.h, ch: 1, px }
    }

    pub fn histogram(&self) -> [u64; 256] {
        let mut h = [0u64; 256];
        for &v in &self.px {
            h[v as usize] += 1;
        }
        h
    }

    /// `point(lut)`, the same table on every channel.
    pub fn point(&self, lut: &[u8; 256]) -> Img {
        Img { px: self.px.iter().map(|&v| lut[v as usize]).collect(), ..self.clone() }
    }

    /// One channel as an L image (`split()[c]`).
    pub fn channel(&self, c: usize) -> Img {
        Img { w: self.w, h: self.h, ch: 1, px: self.px.iter().skip(c).step_by(self.ch).copied().collect() }
    }

    /// `Image.merge('RGB', (r, g, b))`.
    pub fn merge(chs: [&Img; 3]) -> Img {
        let (w, h) = (chs[0].w, chs[0].h);
        assert!(chs.iter().all(|c| c.ch == 1 && c.w == w && c.h == h), "merge: L images of one size");
        let mut px = Vec::with_capacity(w * h * 3);
        for i in 0..w * h {
            px.extend(chs.iter().map(|c| c.px[i]));
        }
        Img { w, h, ch: 3, px }
    }

    fn row(&self, y: usize) -> &[u8] {
        &self.px[y * self.w * self.ch..(y + 1) * self.w * self.ch]
    }

    /// `transpose(FLIP_TOP_BOTTOM)`.
    pub fn flip_tb(&self) -> Img {
        let px = (0..self.h).rev().flat_map(|y| self.row(y).iter().copied()).collect();
        Img { px, ..self.clone() }
    }

    /// `transpose(ROTATE_90)`: a quarter turn counter-clockwise.
    pub fn rotate90(&self) -> Img {
        let (w, h, ch) = (self.h, self.w, self.ch);
        let mut px = vec![0u8; w * h * ch];
        for y in 0..h {
            for x in 0..w {
                let s = (x * self.w + (self.w - 1 - y)) * ch;
                px[(y * w + x) * ch..][..ch].copy_from_slice(&self.px[s..s + ch]);
            }
        }
        Img { w, h, ch, px }
    }

    /// `crop((x, y, x + w, y + h))` inside the image.
    pub fn crop(&self, x: usize, y: usize, w: usize, h: usize) -> Img {
        let px = (y..y + h).flat_map(|r| self.row(r)[x * self.ch..(x + w) * self.ch].iter().copied()).collect();
        Img { w, h, ch: self.ch, px }
    }

    /// `n` x `m` copies side by side (what pasting the image into a bigger one at every cell makes).
    pub fn tile(&self, n: usize, m: usize) -> Img {
        let mut px = Vec::with_capacity(self.px.len() * n * m);
        for _ in 0..m {
            for y in 0..self.h {
                for _ in 0..n {
                    px.extend_from_slice(self.row(y));
                }
            }
        }
        Img { w: self.w * n, h: self.h * m, ch: self.ch, px }
    }

    /// `resize((w, h), filter)`: Pillow's two-pass fixed-point convolution, horizontal first.
    pub fn resize(&self, w: usize, h: usize, filter: Filter) -> Img {
        if (w, h) == (self.w, self.h) {
            return self.clone();
        }
        const HALF: i32 = 1 << (PRECISION_BITS - 1);
        let ch = self.ch;
        let mut cur = self.clone();
        if w != self.w {
            let k = coeffs(self.w, w, filter);
            let mut px = vec![0u8; w * self.h * ch];
            for (y, out) in px.chunks_exact_mut(w * ch).enumerate() {
                let row = self.row(y);
                for ((xmin, ks), o) in k.iter().zip(out.chunks_exact_mut(ch)) {
                    let src = &row[xmin * ch..(xmin + ks.len()) * ch];
                    if ch == 3 {
                        let mut s = [HALF; 3];
                        for (p, &kv) in src.chunks_exact(3).zip(ks) {
                            s[0] += p[0] as i32 * kv;
                            s[1] += p[1] as i32 * kv;
                            s[2] += p[2] as i32 * kv;
                        }
                        o.copy_from_slice(&s.map(clip8));
                    } else {
                        for (c, oc) in o.iter_mut().enumerate() {
                            let s = src.iter().skip(c).step_by(ch).zip(ks).fold(HALF, |s, (&p, &kv)| s + p as i32 * kv);
                            *oc = clip8(s);
                        }
                    }
                }
            }
            cur = Img { w, h: self.h, ch, px };
        }
        if h != self.h {
            let k = coeffs(self.h, h, filter);
            let stride = w * ch;
            let mut px = vec![0u8; stride * h];
            let mut acc = vec![0i32; stride];
            for ((ymin, ks), out) in k.iter().zip(px.chunks_exact_mut(stride)) {
                acc.fill(HALF);
                for (j, &kv) in ks.iter().enumerate() {
                    for (a, &p) in acc.iter_mut().zip(&cur.px[(ymin + j) * stride..][..stride]) {
                        *a += p as i32 * kv;
                    }
                }
                for (o, &a) in out.iter_mut().zip(&acc) {
                    *o = clip8(a);
                }
            }
            cur = Img { w, h, ch, px };
        }
        cur
    }

    /// `filter(ImageFilter.GaussianBlur(radius))`: three passes of Pillow's extended box blur each way.
    pub fn gaussian_blur(&self, radius: f32) -> Img {
        let r = gaussian_box_radius(radius, 3);
        if r == 0.0 {
            return self.clone();
        }
        let mut cur = self.clone();
        for _ in 0..3 {
            cur = cur.box_blur_rows(r);
        }
        let mut t = cur.transpose();
        for _ in 0..3 {
            t = t.box_blur_rows(r);
        }
        t.transpose()
    }

    fn transpose(&self) -> Img {
        let ch = self.ch;
        let mut px = vec![0u8; self.px.len()];
        for y in 0..self.h {
            for x in 0..self.w {
                px[(x * self.h + y) * ch..][..ch].copy_from_slice(&self.px[(y * self.w + x) * ch..][..ch]);
            }
        }
        Img { w: self.h, h: self.w, ch, px }
    }

    /// `ImagingHorizontalBoxBlur`.
    fn box_blur_rows(&self, float_radius: f32) -> Img {
        let (ww, fw, radius) = box_weights(float_radius);
        let ch = self.ch;
        let lastx = self.w - 1;
        let edge_a = (radius + 1).min(self.w);
        let edge_b = (self.w as i64 - radius as i64 - 1).max(0) as usize;
        let mut px = vec![0u8; self.px.len()];
        for y in 0..self.h {
            let line = self.row(y);
            let out = &mut px[y * self.w * ch..(y + 1) * self.w * ch];
            for c in 0..ch {
                let p = |x: usize| line[x * ch + c] as u32;
                let mut acc: u32 = p(0) * (radius as u32 + 1);
                for x in 0..edge_a.saturating_sub(1) {
                    acc = acc.wrapping_add(p(x));
                }
                acc = acc.wrapping_add(p(lastx).wrapping_mul((radius as i64 - edge_a as i64 + 1) as u32));
                let mut step = |x: usize, sub: usize, add: usize, left: usize, right: usize| {
                    acc = acc.wrapping_add(p(add).wrapping_sub(p(sub)));
                    let bulk = acc.wrapping_mul(ww).wrapping_add((p(left) + p(right)).wrapping_mul(fw));
                    out[x * ch + c] = (bulk.wrapping_add(1 << 23) >> 24) as u8;
                };
                if edge_a <= edge_b {
                    for x in 0..edge_a {
                        step(x, 0, x + radius, 0, x + radius + 1);
                    }
                    for x in edge_a..edge_b {
                        step(x, x - radius - 1, x + radius, x - radius - 1, x + radius + 1);
                    }
                    for x in edge_b..=lastx {
                        step(x, x - radius - 1, lastx, x - radius - 1, lastx);
                    }
                } else {
                    for x in 0..edge_b {
                        step(x, 0, x + radius, 0, x + radius + 1);
                    }
                    for x in edge_b..edge_a {
                        step(x, 0, lastx, 0, lastx);
                    }
                    for x in edge_a..=lastx {
                        step(x, x - radius - 1, lastx, x - radius - 1, lastx);
                    }
                }
            }
        }
        Img { px, ..self.clone() }
    }

    /// Pillow's `save(path, 'JPEG', quality=q, optimize=...)`.
    pub fn save_jpeg(&self, path: &Path, quality: u8, optimize: bool) -> Result<(), String> {
        let bytes = super::jpeg::encode(&self.px, self.w, self.h, self.ch, quality, optimize);
        std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// Pillow's `L24(rgb) >> 16`.
pub fn luma(r: u8, g: u8, b: u8) -> u8 {
    ((r as u32 * 19595 + g as u32 * 38470 + b as u32 * 7471 + 0x8000) >> 16) as u8
}

/// Pillow's `_gaussian_blur_radius`: the extended box radius whose `passes` passes match the Gaussian's variance
/// (Gwosdek et al.), in C's float arithmetic. clang contracts `l * (l + 1) - 3 * sigma2` and
/// `sigma2 - (l + 1) * (l + 1)` into fused multiply-adds, as here.
pub fn gaussian_box_radius(radius: f32, passes: i32) -> f32 {
    let sigma2 = radius * radius / passes as f32;
    let big_l = (12.0 * sigma2 as f64 + 1.0).sqrt() as f32;
    let l = ((big_l as f64 - 1.0) / 2.0).floor() as f32;
    let mut a = 2.0f32.mul_add(l, 1.0) * l.mul_add(l + 1.0, -(3.0 * sigma2));
    a /= 6.0 * (-(l + 1.0)).mul_add(l + 1.0, sigma2);
    l + a
}

/// `ImagingHorizontalBoxBlur`'s weights for a fractional radius: (inner weight, weight of the two edge pixels,
/// integer radius), all over 2^24.
pub fn box_weights(float_radius: f32) -> (u32, u32, usize) {
    let radius = float_radius as usize;
    let ww = ((1u32 << 24) as f32 / (float_radius * 2.0 + 1.0)) as u32;
    let fw = ((1u32 << 24) - (radius as u32 * 2 + 1) * ww) / 2;
    (ww, fw, radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grey_is_itu601_fixed_point() {
        assert_eq!(luma(255, 255, 255), 255);
        assert_eq!(luma(255, 0, 0), 76);
        assert_eq!(luma(0, 255, 0), 150);
        assert_eq!(luma(0, 0, 255), 29);
        assert_eq!(luma(10, 200, 30), 124);
    }

    #[test]
    fn box_radius_maths() {
        // sigma^2 = 48^2 / 3 = 768: L = sqrt(12 * 768 + 1) = 96.005, l = 47, a = 95 * -48 / (6 * -1536) = 0.49479...
        let r = gaussian_box_radius(48.0, 3);
        assert!((r - 47.494_79).abs() < 1e-4, "{r}");
        let (ww, fw, radius) = box_weights(r);
        assert_eq!(radius, 47);
        // The weights cover the box exactly: 95 inner pixels and two fractional ends.
        assert!(((1u32 << 24) - (95 * ww + 2 * fw)) <= 1);
        assert_eq!(gaussian_box_radius(0.0, 3), 0.0);
    }

    #[test]
    fn blur_keeps_flat_images() {
        let im = Img::new(40, 30, 1, 77);
        assert_eq!(im.gaussian_blur(4.0), im);
    }

    #[test]
    fn lanczos_halving_weights_sum_to_one() {
        for (_, k) in coeffs(1024, 512, Filter::Lanczos) {
            let s: i32 = k.iter().sum();
            assert!((s - (1 << PRECISION_BITS)).abs() <= k.len() as i32, "{s}");
        }
    }

    #[test]
    fn rotate_and_flip() {
        let im = Img { w: 2, h: 1, ch: 1, px: vec![1, 2] };
        assert_eq!(im.rotate90(), Img { w: 1, h: 2, ch: 1, px: vec![2, 1] });
        let tall = Img { w: 1, h: 2, ch: 1, px: vec![1, 2] };
        assert_eq!(tall.flip_tb().px, [2, 1]);
    }
}
