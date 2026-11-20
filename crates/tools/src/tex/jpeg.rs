//! Baseline JPEG the way Pillow does it with libjpeg-turbo.
//!
//! The encoder makes what `save(path, 'JPEG', quality=q[, optimize=True][, subsampling=0])` makes: YCbCr from
//! libjpeg's fixed-point tables, 4:2:0 chroma (2x2 means with its alternating bias), its edge padding and dummy
//! blocks, or 4:4:4 (`Chroma::Full`), the
//! islow DCT, the quality-scaled Annex K tables and reciprocal quantisation, and Huffman tables optimised per image
//! (`optimize`) or the standard ones.
//!
//! The decoder reads what `Image.open` reads for a baseline file (one interleaved scan, 1x1/2x1/2x2 chroma): the
//! islow IDCT, fancy (triangle) upsampling with libjpeg's edge rows, and its YCbCr -> RGB tables. Anything else
//! (progressive, CMYK, odd sampling) returns `None` for the caller to decode some other way.

/// Annex K.1/K.2 quantisation tables, natural order.
#[rustfmt::skip]
const STD_QUANT: [[u16; 64]; 2] = [
    [16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56, 14, 17, 22, 29, 51, 87, 80, 62,
     18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113, 92, 49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99],
    [17, 18, 24, 47, 99, 99, 99, 99, 18, 21, 26, 66, 99, 99, 99, 99, 24, 26, 56, 99, 99, 99, 99, 99, 47, 66, 99, 99, 99, 99, 99, 99,
     99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99],
];

/// Zigzag position -> natural index.
#[rustfmt::skip]
const NATURAL: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21, 28,
    35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

const DC_BITS: [[u8; 16]; 2] = [
    [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
    [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
];
const DC_VALS: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const AC_BITS: [[u8; 16]; 2] = [
    [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d],
    [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77],
];
#[rustfmt::skip]
const AC_VALS: [[u8; 162]; 2] = [
    [0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08,
     0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0, 0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28,
     0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
     0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
     0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6,
     0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
     0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA],
    [0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71, 0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91,
     0xA1, 0xB1, 0xC1, 0x09, 0x23, 0x33, 0x52, 0xF0, 0x15, 0x62, 0x72, 0xD1, 0x0A, 0x16, 0x24, 0x34, 0xE1, 0x25, 0xF1, 0x17, 0x18, 0x19, 0x1A, 0x26,
     0x27, 0x28, 0x29, 0x2A, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58,
     0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
     0x88, 0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4,
     0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA,
     0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA],
];

/// `jpeg_set_quality(q, force_baseline=TRUE)` on an Annex K table.
pub fn quant_table(which: usize, quality: u8) -> [u16; 64] {
    let q = quality.clamp(1, 100) as u32;
    let scale = if q < 50 { 5000 / q } else { 200 - q * 2 };
    STD_QUANT[which].map(|v| ((v as u32 * scale + 50) / 100).clamp(1, 255) as u16)
}

/// libjpeg-turbo's `compute_reciprocal` (16-bit DCTELEM): (reciprocal, correction, shift).
fn reciprocal(divisor: u32) -> (u32, u32, u32) {
    if divisor == 1 {
        return (1, 0, 0);
    }
    let b = 31 - divisor.leading_zeros();
    let mut r = 16 + b;
    let mut fq = (1u32 << r) / divisor;
    let fr = (1u32 << r) % divisor;
    let mut c = divisor / 2;
    if fr == 0 {
        fq >>= 1;
        r -= 1;
    } else if fr <= divisor / 2 {
        c += 1;
    } else {
        fq += 1;
    }
    (fq, c, r - 16)
}

/// `jpeg_fdct_islow`: the output is scaled up by 8.
pub fn fdct_islow(d: &mut [i32; 64]) {
    const CONST_BITS: i32 = 13;
    const PASS1_BITS: i32 = 2;
    const F0_298: i32 = 2446;
    const F0_390: i32 = 3196;
    const F0_541: i32 = 4433;
    const F0_765: i32 = 6270;
    const F0_899: i32 = 7373;
    const F1_175: i32 = 9633;
    const F1_501: i32 = 12299;
    const F1_847: i32 = 15137;
    const F1_961: i32 = 16069;
    const F2_053: i32 = 16819;
    const F2_562: i32 = 20995;
    const F3_072: i32 = 25172;
    let descale = |x: i32, n: i32| (x + (1 << (n - 1))) >> n;
    for pass in 0..2 {
        let (stride, step, shift_even, shift_odd) = if pass == 0 {
            (1, 8, 0, CONST_BITS - PASS1_BITS)
        } else {
            (8, 1, PASS1_BITS, CONST_BITS + PASS1_BITS)
        };
        for k in 0..8 {
            let at = |i: usize| k * step + i * stride;
            let tmp0 = d[at(0)] + d[at(7)];
            let tmp7 = d[at(0)] - d[at(7)];
            let tmp1 = d[at(1)] + d[at(6)];
            let tmp6 = d[at(1)] - d[at(6)];
            let tmp2 = d[at(2)] + d[at(5)];
            let tmp5 = d[at(2)] - d[at(5)];
            let tmp3 = d[at(3)] + d[at(4)];
            let tmp4 = d[at(3)] - d[at(4)];
            let tmp10 = tmp0 + tmp3;
            let tmp13 = tmp0 - tmp3;
            let tmp11 = tmp1 + tmp2;
            let tmp12 = tmp1 - tmp2;
            if pass == 0 {
                d[at(0)] = (tmp10 + tmp11) << PASS1_BITS;
                d[at(4)] = (tmp10 - tmp11) << PASS1_BITS;
            } else {
                d[at(0)] = descale(tmp10 + tmp11, shift_even);
                d[at(4)] = descale(tmp10 - tmp11, shift_even);
            }
            let z1 = (tmp12 + tmp13) * F0_541;
            d[at(2)] = descale(z1 + tmp13 * F0_765, shift_odd);
            d[at(6)] = descale(z1 + tmp12 * -F1_847, shift_odd);
            let z1 = tmp4 + tmp7;
            let z2 = tmp5 + tmp6;
            let z3 = tmp4 + tmp6;
            let z4 = tmp5 + tmp7;
            let z5 = (z3 + z4) * F1_175;
            let tmp4 = tmp4 * F0_298;
            let tmp5 = tmp5 * F2_053;
            let tmp6 = tmp6 * F3_072;
            let tmp7 = tmp7 * F1_501;
            let z1 = z1 * -F0_899;
            let z2 = z2 * -F2_562;
            let z3 = z3 * -F1_961 + z5;
            let z4 = z4 * -F0_390 + z5;
            d[at(7)] = descale(tmp4 + z1 + z3, shift_odd);
            d[at(5)] = descale(tmp5 + z2 + z4, shift_odd);
            d[at(3)] = descale(tmp6 + z2 + z3, shift_odd);
            d[at(1)] = descale(tmp7 + z1 + z4, shift_odd);
        }
    }
}

/// A Huffman table: `bits[k]` codes of length k+1, then the symbols in code order.
#[derive(Clone)]
struct Huff {
    bits: [u8; 16],
    vals: Vec<u8>,
}

impl Huff {
    /// (code, length) per symbol, as `jpeg_make_c_derived_tbl` assigns them.
    fn codes(&self) -> [(u16, u8); 256] {
        let mut out = [(0u16, 0u8); 256];
        let mut code = 0u16;
        let mut k = 0;
        for (len, &n) in self.bits.iter().enumerate() {
            for _ in 0..n {
                out[self.vals[k] as usize] = (code, len as u8 + 1);
                code += 1;
                k += 1;
            }
            code <<= 1;
        }
        out
    }

    /// `jpeg_gen_optimal_table`.
    fn optimal(freq_in: &[u64; 257]) -> Huff {
        const MAX_CLEN: usize = 32;
        let mut freq = *freq_in;
        freq[256] = 1;
        let mut bits = [0u8; MAX_CLEN + 1];
        let mut codesize = [0usize; 257];
        let mut others = [-1i32; 257];
        loop {
            let mut c1: i32 = -1;
            let mut v = u64::MAX;
            for (i, &f) in freq.iter().enumerate() {
                if f != 0 && f <= v {
                    v = f;
                    c1 = i as i32;
                }
            }
            let mut c2: i32 = -1;
            v = u64::MAX;
            for (i, &f) in freq.iter().enumerate() {
                if f != 0 && f <= v && i as i32 != c1 {
                    v = f;
                    c2 = i as i32;
                }
            }
            if c2 < 0 {
                break;
            }
            let (mut a, mut b) = (c1 as usize, c2 as usize);
            freq[a] += freq[b];
            freq[b] = 0;
            codesize[a] += 1;
            while others[a] >= 0 {
                a = others[a] as usize;
                codesize[a] += 1;
            }
            others[a] = b as i32;
            codesize[b] += 1;
            while others[b] >= 0 {
                b = others[b] as usize;
                codesize[b] += 1;
            }
        }
        for &s in &codesize {
            if s > 0 {
                bits[s.min(MAX_CLEN)] += 1;
            }
        }
        let mut i = MAX_CLEN;
        while i > 16 {
            while bits[i] > 0 {
                let mut j = i - 2;
                while bits[j] == 0 {
                    j -= 1;
                }
                bits[i] -= 2;
                bits[i - 1] += 1;
                bits[j + 1] += 2;
                bits[j] -= 1;
            }
            i -= 1;
        }
        while bits[i] == 0 {
            i -= 1;
        }
        bits[i] -= 1;
        let mut vals = Vec::new();
        for len in 1..=MAX_CLEN {
            for (sym, &s) in codesize.iter().enumerate().take(256) {
                if s == len {
                    vals.push(sym as u8);
                }
            }
        }
        let mut b16 = [0u8; 16];
        b16.copy_from_slice(&bits[1..17]);
        Huff { bits: b16, vals }
    }
}

struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}

impl BitWriter {
    fn put(&mut self, code: u32, len: u32) {
        self.acc = (self.acc << len) | (code & ((1 << len) - 1)) as u64;
        self.n += len;
        while self.n >= 8 {
            self.n -= 8;
            let b = (self.acc >> self.n) as u8;
            self.out.push(b);
            if b == 0xFF {
                self.out.push(0);
            }
        }
        self.acc &= (1 << self.n) - 1;
    }

    fn flush(&mut self) {
        if self.n > 0 {
            let pad = 8 - self.n;
            self.put((1 << pad) - 1, pad);
        }
    }
}

fn nbits(v: i32) -> u32 {
    32 - v.unsigned_abs().leading_zeros()
}

/// The symbols one block codes to: `(table kind 0 = DC / 1 = AC, symbol, extra bits, extra length)`.
fn block_symbols(block: &[i32; 64], last_dc: &mut i32, mut f: impl FnMut(usize, u8, u32, u32)) {
    let diff = block[0] - *last_dc;
    *last_dc = block[0];
    let n = nbits(diff);
    f(
        0,
        n as u8,
        (if diff < 0 { diff - 1 } else { diff }) as u32 & ((1u32 << n) - 1),
        n,
    );
    let mut run = 0;
    for &k in &NATURAL[1..] {
        let v = block[k];
        if v == 0 {
            run += 1;
            continue;
        }
        while run > 15 {
            f(1, 0xF0, 0, 0);
            run -= 16;
        }
        let n = nbits(v);
        f(
            1,
            (run << 4 | n) as u8,
            (if v < 0 { v - 1 } else { v }) as u32 & ((1u32 << n) - 1),
            n,
        );
        run = 0;
    }
    if run > 0 {
        f(1, 0, 0, 0);
    }
}

/// One 8-bit plane, padded by repeating the last column and row.
struct Plane {
    w: usize,
    h: usize,
    px: Vec<u8>,
}

impl Plane {
    fn padded(src: &[u8], w: usize, h: usize, pw: usize, ph: usize) -> Plane {
        let mut px = vec![0u8; pw * ph];
        for (y, row) in px.chunks_exact_mut(pw).enumerate() {
            let s = &src[y.min(h - 1) * w..][..w];
            let n = w.min(pw);
            row[..n].copy_from_slice(&s[..n]);
            row[n..].fill(s[w - 1]);
        }
        Plane { w: pw, h: ph, px }
    }

    /// `h2v2_downsample`: 2x2 means, the bias alternating 1, 2 along each row.
    fn h2v2(&self) -> Plane {
        let (w, h) = (self.w / 2, self.h / 2);
        let mut px = vec![0u8; w * h];
        for y in 0..h {
            let (r0, r1) = (&self.px[2 * y * self.w..], &self.px[(2 * y + 1) * self.w..]);
            for x in 0..w {
                let bias = if x % 2 == 0 { 1 } else { 2 };
                px[y * w + x] = ((r0[2 * x] as u32
                    + r0[2 * x + 1] as u32
                    + r1[2 * x] as u32
                    + r1[2 * x + 1] as u32
                    + bias)
                    >> 2) as u8;
            }
        }
        Plane { w, h, px }
    }

    fn block(&self, bx: usize, by: usize, quant: &[(u32, u32, u32); 64]) -> [i32; 64] {
        let mut d = [0i32; 64];
        for y in 0..8 {
            for x in 0..8 {
                d[y * 8 + x] = self.px[(by * 8 + y) * self.w + bx * 8 + x] as i32 - 128;
            }
        }
        fdct_islow(&mut d);
        for (v, &(recip, corr, shift)) in d.iter_mut().zip(quant) {
            let t = v.unsigned_abs();
            let q = (((t + corr) * recip) >> (shift + 16)) as i32;
            *v = if *v < 0 { -q } else { q };
        }
        d
    }
}

/// libjpeg's `rgb_ycc_convert` (SCALEBITS 16).
pub fn rgb_to_ycc(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let (r, g, b) = (r as i32, g as i32, b as i32);
    const HALF: i32 = 1 << 15;
    const CBCR: i32 = 128 << 16;
    let y = (19595 * r + 38470 * g + 7471 * b + HALF) >> 16;
    let cb = (-11059 * r - 21709 * g + 32768 * b + CBCR + HALF - 1) >> 16;
    let cr = (32768 * r - 27439 * g - 5329 * b + CBCR + HALF - 1) >> 16;
    (y as u8, cb as u8, cr as u8)
}

/// How much of the colour a JPEG keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chroma {
    /// 4:2:0, Pillow's default (`subsampling=2`): a chroma sample for each 2x2 pixels. Enough for the eye,
    /// which sees colour coarser than brightness.
    Half,
    /// 4:4:4 (`subsampling=0`): every component at full size. For data (a normal map's directions, a packed
    /// map's channels), where each channel is a quantity of its own and averaging them bends it.
    Full,
}

/// Encodes 8-bit greyscale (`ch` 1) or RGB (`ch` 3) pixels (`chroma` only matters for RGB).
pub fn encode(
    px: &[u8],
    w: usize,
    h: usize,
    ch: usize,
    quality: u8,
    optimize: bool,
    chroma: Chroma,
) -> Vec<u8> {
    assert!(ch == 1 || ch == 3, "jpeg: 1 or 3 channels");
    assert!(w > 0 && h > 0 && w <= 65535 && h <= 65535, "jpeg: bad size");
    let colour = ch == 3;
    let half = colour && chroma == Chroma::Half;
    // Each component is padded by repeating its last column and row out to whole blocks (chroma: the full-size
    // plane to an even height and to twice its block width before the 2x2 means, then its last row); blocks past
    // a component's last whole block column or row in an MCU are libjpeg's dummies: no AC, the DC before them.
    let (wib, hib) = (w.div_ceil(8), h.div_ceil(8));
    let ycc = |px: &[u8]| {
        let mut ycc = [vec![0u8; w * h], vec![0u8; w * h], vec![0u8; w * h]];
        for i in 0..w * h {
            let (y, cb, cr) = rgb_to_ycc(px[3 * i], px[3 * i + 1], px[3 * i + 2]);
            ycc[0][i] = y;
            ycc[1][i] = cb;
            ycc[2][i] = cr;
        }
        ycc
    };
    let planes: Vec<Plane> = if half {
        let ycc = ycc(px);
        let (cw, chh) = (w.div_ceil(2), h.div_ceil(2));
        let sub = |p: &[u8]| {
            let ds = Plane::padded(p, w, h, cw.div_ceil(8) * 16, chh * 2).h2v2();
            Plane::padded(&ds.px, ds.w, ds.h, ds.w, chh.div_ceil(8) * 8)
        };
        vec![
            Plane::padded(&ycc[0], w, h, wib * 8, hib * 8),
            sub(&ycc[1]),
            sub(&ycc[2]),
        ]
    } else if colour {
        ycc(px)
            .iter()
            .map(|p| Plane::padded(p, w, h, wib * 8, hib * 8))
            .collect()
    } else {
        vec![Plane::padded(px, w, h, wib * 8, hib * 8)]
    };
    let tables: Vec<[u16; 64]> = if colour {
        vec![quant_table(0, quality), quant_table(1, quality)]
    } else {
        vec![quant_table(0, quality)]
    };
    let divisors: Vec<[(u32, u32, u32); 64]> = tables
        .iter()
        .map(|t| t.map(|q| reciprocal(q as u32 * 8)))
        .collect();
    let table_of = |comp: usize| usize::from(comp > 0);
    // The blocks in scan order, each with its component.
    let mut blocks: Vec<(usize, [i32; 64])> = Vec::with_capacity(wib * hib * planes.len());
    if half {
        for my in 0..h.div_ceil(16) {
            for mx in 0..w.div_ceil(16) {
                let mut last_dc = 0;
                for (by, bx) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                    let (bx, by) = (mx * 2 + bx, my * 2 + by);
                    let b = if bx < wib && by < hib {
                        planes[0].block(bx, by, &divisors[0])
                    } else {
                        let mut d = [0i32; 64];
                        d[0] = last_dc;
                        d
                    };
                    last_dc = b[0];
                    blocks.push((0, b));
                }
                blocks.push((1, planes[1].block(mx, my, &divisors[1])));
                blocks.push((2, planes[2].block(mx, my, &divisors[1])));
            }
        }
    } else {
        // One block of each component per MCU.
        for by in 0..hib {
            for bx in 0..wib {
                for (comp, plane) in planes.iter().enumerate() {
                    blocks.push((comp, plane.block(bx, by, &divisors[table_of(comp)])));
                }
            }
        }
    }
    let ntab = if colour { 2 } else { 1 };
    let huffs: Vec<[Huff; 2]> = if optimize {
        let mut freq = vec![[[0u64; 257]; 2]; ntab];
        let mut last = [0i32; 3];
        for (comp, b) in &blocks {
            let t = table_of(*comp);
            block_symbols(b, &mut last[*comp], |kind, sym, _, _| {
                freq[t][kind][sym as usize] += 1
            });
        }
        freq.iter()
            .map(|f| [Huff::optimal(&f[0]), Huff::optimal(&f[1])])
            .collect()
    } else {
        (0..ntab)
            .map(|t| {
                [
                    Huff {
                        bits: DC_BITS[t],
                        vals: DC_VALS.to_vec(),
                    },
                    Huff {
                        bits: AC_BITS[t],
                        vals: AC_VALS[t].to_vec(),
                    },
                ]
            })
            .collect()
    };
    let codes: Vec<[[(u16, u8); 256]; 2]> =
        huffs.iter().map(|h| [h[0].codes(), h[1].codes()]).collect();

    let mut out = Vec::with_capacity(w * h / 4);
    let seg = |out: &mut Vec<u8>, marker: u8, body: &[u8]| {
        out.extend_from_slice(&[0xFF, marker]);
        out.extend_from_slice(&((body.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(body);
    };
    out.extend_from_slice(&[0xFF, 0xD8]);
    seg(
        &mut out,
        0xE0,
        &[b'J', b'F', b'I', b'F', 0, 1, 1, 0, 0, 1, 0, 1, 0, 0],
    );
    for (n, t) in tables.iter().enumerate() {
        let mut body = vec![n as u8];
        body.extend(NATURAL.iter().map(|&k| t[k] as u8));
        seg(&mut out, 0xDB, &body);
    }
    let ncomp = planes.len();
    let mut sof = vec![8];
    sof.extend_from_slice(&(h as u16).to_be_bytes());
    sof.extend_from_slice(&(w as u16).to_be_bytes());
    sof.push(ncomp as u8);
    for c in 0..ncomp {
        let samp = if half && c == 0 { 0x22 } else { 0x11 };
        sof.extend_from_slice(&[c as u8 + 1, samp, table_of(c) as u8]);
    }
    seg(&mut out, 0xC0, &sof);
    for (t, h) in huffs.iter().enumerate() {
        for (kind, hk) in h.iter().enumerate() {
            let mut body = vec![(kind as u8) << 4 | t as u8];
            body.extend_from_slice(&hk.bits);
            body.extend_from_slice(&hk.vals);
            seg(&mut out, 0xC4, &body);
        }
    }
    let mut sos = vec![ncomp as u8];
    for c in 0..ncomp {
        let t = table_of(c) as u8;
        sos.extend_from_slice(&[c as u8 + 1, t << 4 | t]);
    }
    sos.extend_from_slice(&[0, 63, 0]);
    seg(&mut out, 0xDA, &sos);
    let mut bw = BitWriter { out, acc: 0, n: 0 };
    let mut last = [0i32; 3];
    for (comp, b) in &blocks {
        let c = &codes[table_of(*comp)];
        block_symbols(b, &mut last[*comp], |kind, sym, extra, n| {
            let (code, len) = c[kind][sym as usize];
            bw.put(code as u32, len as u32);
            if n > 0 {
                bw.put(extra, n);
            }
        });
    }
    bw.flush();
    let mut out = bw.out;
    out.extend_from_slice(&[0xFF, 0xD9]);
    out
}

mod decode;
pub use decode::decode;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let (w, h) = (37, 21);
        // A smooth gradient (4:2:0 keeps it) comes back within a few levels.
        let px: Vec<u8> = (0..w * h * 3)
            .map(|i| ((i / 3 % w) * 2 + (i / 3 / w) * 2 + (i % 3) * 30) as u8)
            .collect();
        let (dw, dh, ch, out) = decode(&encode(&px, w, h, 3, 95, true, Chroma::Half)).unwrap();
        assert_eq!((dw, dh, ch), (w, h, 3));
        let err: f64 = px
            .iter()
            .zip(&out)
            .map(|(&a, &b)| (a as f64 - b as f64).powi(2))
            .sum::<f64>()
            / px.len() as f64;
        assert!(err < 10.0, "{err}");
        let grey: Vec<u8> = (0..w * h).map(|i| (i % 256) as u8).collect();
        let (_, _, ch, out) = decode(&encode(&grey, w, h, 1, 90, false, Chroma::Half)).unwrap();
        assert_eq!(ch, 1);
        assert!(grey.iter().zip(&out).all(|(&a, &b)| a.abs_diff(b) < 40));
    }

    #[test]
    fn full_chroma_keeps_colour_detail() {
        // Blue alternating each pixel under a flat red and green: 4:2:0 averages it away, 4:4:4 keeps it
        // (and both come back as whole pictures, the odd size padded).
        let (w, h) = (35, 19);
        let px: Vec<u8> = (0..w * h)
            .flat_map(|i| [128, 128, if (i % w + i / w) % 2 == 0 { 96 } else { 160 }])
            .collect();
        let error = |chroma| {
            let (dw, dh, ch, out) = decode(&encode(&px, w, h, 3, 90, true, chroma)).unwrap();
            assert_eq!((dw, dh, ch), (w, h, 3));
            px.iter()
                .zip(&out)
                .skip(2)
                .step_by(3)
                .map(|(&a, &b)| a.abs_diff(b) as f64)
                .sum::<f64>()
                / (w * h) as f64
        };
        let (half, full) = (error(Chroma::Half), error(Chroma::Full));
        assert!(half > 20.0, "{half}");
        assert!(full < 8.0, "{full}");
    }

    #[test]
    fn quality_scaling() {
        assert_eq!(quant_table(0, 85)[..4], [5, 3, 3, 5]);
        assert_eq!(quant_table(1, 88)[..4], [4, 4, 6, 11]);
    }

    #[test]
    fn reciprocal_matches_division() {
        for q in [8u32, 24, 40, 88, 792, 2040] {
            let (recip, corr, shift) = reciprocal(q);
            for t in 0..20000u32 {
                assert_eq!(
                    ((t + corr) * recip) >> (shift + 16),
                    (t + q / 2) / q,
                    "q {q} t {t}"
                );
            }
        }
    }

    #[test]
    fn flat_block_is_dc_only() {
        let mut d = [10i32; 64];
        fdct_islow(&mut d);
        assert_eq!(d[0], 10 * 64);
        assert!(d[1..].iter().all(|&v| v == 0));
    }
}
