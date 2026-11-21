use super::*;

// --- Decoding -------------------------------------------------------------------------------------

/// `jpeg_idct_islow` on dequantised coefficients (natural order), with libjpeg's range limiting.
pub fn idct_islow(c: &[i32; 64], out: &mut [u8; 64]) {
    const CONST_BITS: i32 = 13;
    const PASS1_BITS: i32 = 2;
    let descale = |x: i32, n: i32| (x + (1 << (n - 1))) >> n;
    // `range_limit[x & RANGE_MASK]`: the descaled value as a 10-bit two's complement number, plus 128, clamped.
    let limit = |x: i32| ((((x & 1023) ^ 512) - 512) + 128).clamp(0, 255) as u8;
    let odd_even = |s: [i32; 8]| {
        let (z2, z3) = (s[2], s[6]);
        let z1 = (z2 + z3) * 4433;
        let tmp2 = z1 + z3 * -15137;
        let tmp3 = z1 + z2 * 6270;
        let tmp0 = (s[0] + s[4]) << CONST_BITS;
        let tmp1 = (s[0] - s[4]) << CONST_BITS;
        let (tmp10, tmp13, tmp11, tmp12) = (tmp0 + tmp3, tmp0 - tmp3, tmp1 + tmp2, tmp1 - tmp2);
        let (t0, t1, t2, t3) = (s[7], s[5], s[3], s[1]);
        let z1 = t0 + t3;
        let z2 = t1 + t2;
        let z3 = t0 + t2;
        let z4 = t1 + t3;
        let z5 = (z3 + z4) * 9633;
        let z1 = z1 * -7373;
        let z2 = z2 * -20995;
        let z3 = z3 * -16069 + z5;
        let z4 = z4 * -3196 + z5;
        let t0 = t0 * 2446 + z1 + z3;
        let t1 = t1 * 16819 + z2 + z4;
        let t2 = t2 * 25172 + z2 + z3;
        let t3 = t3 * 12299 + z1 + z4;
        [
            tmp10 + t3,
            tmp11 + t2,
            tmp12 + t1,
            tmp13 + t0,
            tmp13 - t0,
            tmp12 - t1,
            tmp11 - t2,
            tmp10 - t3,
        ]
    };
    let mut ws = [0i32; 64];
    for col in 0..8 {
        let s: [i32; 8] = std::array::from_fn(|k| c[k * 8 + col]);
        if s[1..].iter().all(|&v| v == 0) {
            for k in 0..8 {
                ws[k * 8 + col] = s[0] << PASS1_BITS;
            }
            continue;
        }
        let r = odd_even(s);
        for k in 0..8 {
            ws[k * 8 + col] = descale(r[k], CONST_BITS - PASS1_BITS);
        }
    }
    for row in 0..8 {
        let s: [i32; 8] = std::array::from_fn(|k| ws[row * 8 + k]);
        if s[1..].iter().all(|&v| v == 0) {
            let v = limit(descale(s[0], PASS1_BITS + 3));
            out[row * 8..row * 8 + 8].fill(v);
            continue;
        }
        let r = odd_even(s);
        for k in 0..8 {
            out[row * 8 + k] = limit(descale(r[k], CONST_BITS + PASS1_BITS + 3));
        }
    }
}

const LOOK: u32 = 10;

/// A Huffman table for decoding: a 10-bit lookahead, then code by code (JPEG F.2.2.3).
struct HuffDec {
    look: Vec<(u8, u8)>,
    maxcode: [i32; 18],
    valptr: [i32; 17],
    mincode: [i32; 17],
    vals: Vec<u8>,
}

impl HuffDec {
    fn new(bits: &[u8; 16], vals: Vec<u8>) -> HuffDec {
        let mut look = vec![(0u8, 0u8); 1 << LOOK];
        let (mut maxcode, mut valptr, mut mincode) = ([-1i32; 18], [0i32; 17], [0i32; 17]);
        let (mut code, mut k) = (0i32, 0usize);
        for len in 1..=16 {
            let n = bits[len - 1] as usize;
            valptr[len] = k as i32;
            mincode[len] = code;
            for _ in 0..n {
                if len as u32 <= LOOK {
                    let shift = LOOK - len as u32;
                    for fill in 0..1 << shift {
                        look[((code << shift) | fill) as usize] =
                            (len as u8, vals.get(k).copied().unwrap_or(0));
                    }
                }
                code += 1;
                k += 1;
            }
            maxcode[len] = if n > 0 { code - 1 } else { -1 };
            code <<= 1;
        }
        maxcode[17] = i32::MAX;
        HuffDec {
            look,
            maxcode,
            valptr,
            mincode,
            vals,
        }
    }
}

/// The entropy-coded data with its byte stuffing taken out, the byte offsets where each restart interval begins,
/// and zeros after the end (what libjpeg feeds once it meets a marker).
fn unstuff(d: &[u8], mut pos: usize) -> (Vec<u8>, Vec<usize>) {
    let mut out = Vec::with_capacity(d.len().saturating_sub(pos) + 16);
    let mut rst = Vec::new();
    while pos < d.len() {
        let b = d[pos];
        if b != 0xFF {
            out.push(b);
            pos += 1;
            continue;
        }
        match d.get(pos + 1) {
            Some(0x00) => {
                out.push(0xFF);
                pos += 2;
            }
            Some(0xD0..=0xD7) => {
                rst.push(out.len());
                pos += 2;
            }
            Some(0xFF) => pos += 1,
            _ => break,
        }
    }
    out.extend_from_slice(&[0; 16]);
    (out, rst)
}

struct BitReader {
    buf: Vec<u8>,
    rst: Vec<usize>,
    next_rst: usize,
    /// The read position in bits.
    pos: usize,
}

impl BitReader {
    /// The next 32 bits (zeros past the end).
    fn peek(&self) -> u32 {
        let i = self.pos >> 3;
        let w = match self.buf.get(i..i + 8) {
            Some(b) => u64::from_be_bytes(b.try_into().unwrap()),
            None => 0,
        };
        ((w << (self.pos & 7)) >> 32) as u32
    }

    fn bits(&mut self, k: u32) -> i32 {
        if k == 0 {
            return 0;
        }
        let v = (self.peek() >> (32 - k)) as i32;
        self.pos += k as usize;
        v
    }

    fn symbol(&mut self, h: &HuffDec) -> Option<u8> {
        let p = self.peek();
        let (len, v) = h.look[(p >> (32 - LOOK)) as usize];
        if len > 0 {
            self.pos += len as usize;
            return Some(v);
        }
        let mut len = LOOK as usize + 1;
        let mut code = (p >> (32 - len)) as i32;
        while code > h.maxcode[len] {
            len += 1;
            if len > 16 {
                return None;
            }
            code = (p >> (32 - len)) as i32;
        }
        self.pos += len;
        h.vals
            .get((h.valptr[len] + code - h.mincode[len]) as usize)
            .copied()
    }

    fn extend(&mut self, s: u8) -> i32 {
        let v = self.bits(s as u32);
        if s > 0 && v < 1 << (s - 1) {
            v - (1 << s) + 1
        } else {
            v
        }
    }

    /// A symbol and the value its extra bits code (`dc`: the symbol is the bit count; else its low nibble),
    /// from one look at the next 32 bits when the code is short.
    fn coded(&mut self, h: &HuffDec, dc: bool) -> Option<(u8, i32)> {
        let p = self.peek();
        let (len, sym) = h.look[(p >> (32 - LOOK)) as usize];
        if len == 0 {
            let sym = self.symbol(h)?;
            return Some((sym, self.extend(if dc { sym } else { sym & 15 })));
        }
        let s = if dc { sym } else { sym & 15 } as u32;
        self.pos += (len as u32 + s) as usize;
        if s == 0 {
            return Some((sym, 0));
        }
        let v = ((p << len) >> (32 - s)) as i32;
        Some((
            sym,
            if v < 1 << (s - 1) {
                v - (1 << s) + 1
            } else {
                v
            },
        ))
    }

    /// At a restart: drop the rest of the byte and go to where the next interval begins.
    fn restart(&mut self) {
        let next = self
            .rst
            .get(self.next_rst)
            .copied()
            .unwrap_or(self.pos.div_ceil(8));
        self.next_rst += 1;
        self.pos = next.max(self.pos.div_ceil(8)) * 8;
    }
}

struct Comp {
    id: u8,
    h: usize,
    v: usize,
    tq: usize,
}

/// libjpeg's `h2v1_fancy_upsample` of one row (`w` input samples, `w` >= 2).
fn up_h2v1(inp: &[u8], w: usize, out: &mut [u8]) {
    let p = |i: usize| inp[i] as u32;
    out[0] = inp[0];
    out[1] = ((p(0) * 3 + p(1) + 2) >> 2) as u8;
    for i in 1..w - 1 {
        out[2 * i] = ((p(i) * 3 + p(i - 1) + 1) >> 2) as u8;
        out[2 * i + 1] = ((p(i) * 3 + p(i + 1) + 2) >> 2) as u8;
    }
    let i = w - 1;
    out[2 * i] = ((p(i) * 3 + p(i - 1) + 1) >> 2) as u8;
    out[2 * i + 1] = inp[i];
}

/// libjpeg's `h2v2_fancy_upsample` of one output row from its nearest input row and the next nearest.
fn up_h2v2(near: &[u8], far: &[u8], w: usize, out: &mut [u8]) {
    let cs = |i: usize| near[i] as u32 * 3 + far[i] as u32;
    out[0] = ((cs(0) * 4 + 8) >> 4) as u8;
    out[1] = ((cs(0) * 3 + cs(1) + 7) >> 4) as u8;
    for i in 1..w - 1 {
        out[2 * i] = ((cs(i) * 3 + cs(i - 1) + 8) >> 4) as u8;
        out[2 * i + 1] = ((cs(i) * 3 + cs(i + 1) + 7) >> 4) as u8;
    }
    let i = w - 1;
    out[2 * i] = ((cs(i) * 3 + cs(i - 1) + 8) >> 4) as u8;
    out[2 * i + 1] = ((cs(i) * 4 + 7) >> 4) as u8;
}

/// libjpeg's `ycc_rgb_convert` tables (SCALEBITS 16), per Cb/Cr value: Cr->R, Cb->B, Cr->G, Cb->G (+ 1/2).
fn ycc_tables() -> [[i32; 256]; 4] {
    const HALF: i32 = 1 << 15;
    let x = |i: usize| i as i32 - 128;
    [
        std::array::from_fn(|i| (91881 * x(i) + HALF) >> 16),
        std::array::from_fn(|i| (116130 * x(i) + HALF) >> 16),
        std::array::from_fn(|i| -46802 * x(i)),
        std::array::from_fn(|i| -22554 * x(i) + HALF),
    ]
}

/// Decodes a baseline JPEG to `(width, height, channels, pixels)`, or `None` for what this decoder leaves to
/// others (see the module docs).
pub fn decode(d: &[u8]) -> Option<(usize, usize, usize, Vec<u8>)> {
    if d.get(..2)? != [0xFF, 0xD8] {
        return None;
    }
    let mut qt = [[0u16; 64]; 4];
    let mut dc: [Option<HuffDec>; 4] = Default::default();
    let mut ac: [Option<HuffDec>; 4] = Default::default();
    let (mut w, mut h) = (0usize, 0usize);
    let mut comps: Vec<Comp> = Vec::new();
    let mut restart = 0usize;
    let mut pos = 2;
    loop {
        while *d.get(pos)? != 0xFF {
            pos += 1;
        }
        while *d.get(pos)? == 0xFF {
            pos += 1;
        }
        let m = d[pos];
        pos += 1;
        if m == 0xD8 || (0xD0..=0xD7).contains(&m) || m == 0x01 {
            continue;
        }
        if m == 0xD9 {
            return None;
        }
        let len = u16::from_be_bytes([*d.get(pos)?, *d.get(pos + 1)?]) as usize;
        let seg = d.get(pos + 2..pos + len)?;
        pos += len;
        match m {
            0xC0 | 0xC1 => {
                if seg[0] != 8 {
                    return None;
                }
                h = u16::from_be_bytes([seg[1], seg[2]]) as usize;
                w = u16::from_be_bytes([seg[3], seg[4]]) as usize;
                for c in seg[6..].as_chunks::<3>().0.iter().take(seg[5] as usize) {
                    comps.push(Comp {
                        id: c[0],
                        h: (c[1] >> 4) as usize,
                        v: (c[1] & 15) as usize,
                        tq: (c[2] & 3) as usize,
                    });
                }
            }
            0xC2..=0xCF if m != 0xC4 && m != 0xC8 && m != 0xCC => return None,
            0xC4 => {
                let mut s = seg;
                while s.len() >= 17 {
                    let (class, id) = (s[0] >> 4, (s[0] & 3) as usize);
                    let bits: [u8; 16] = s[1..17].try_into().ok()?;
                    let n: usize = bits.iter().map(|&b| b as usize).sum();
                    let vals = s.get(17..17 + n)?.to_vec();
                    let t = HuffDec::new(&bits, vals);
                    if class == 0 {
                        dc[id] = Some(t)
                    } else {
                        ac[id] = Some(t)
                    }
                    s = &s[17 + n..];
                }
            }
            0xDB => {
                let mut s = seg;
                while !s.is_empty() {
                    let (pq, id) = (s[0] >> 4, (s[0] & 3) as usize);
                    let n = if pq == 0 { 64 } else { 128 };
                    let body = s.get(1..1 + n)?;
                    for (k, &nat) in NATURAL.iter().enumerate() {
                        qt[id][nat] = if pq == 0 {
                            body[k] as u16
                        } else {
                            u16::from_be_bytes([body[2 * k], body[2 * k + 1]])
                        };
                    }
                    s = &s[1 + n..];
                }
            }
            0xDD => restart = u16::from_be_bytes([seg[0], seg[1]]) as usize,
            // Adobe's marker can say RGB or CMYK: leave those to others.
            0xEE => return None,
            0xDA => return scan(d, pos, seg, w, h, &comps, &qt, &dc, &ac, restart),
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn scan(
    d: &[u8],
    pos: usize,
    sos: &[u8],
    w: usize,
    h: usize,
    comps: &[Comp],
    qt: &[[u16; 64]; 4],
    dc: &[Option<HuffDec>; 4],
    ac: &[Option<HuffDec>; 4],
    restart: usize,
) -> Option<(usize, usize, usize, Vec<u8>)> {
    let n = comps.len();
    if !(n == 1 || n == 3)
        || w == 0
        || h == 0
        || sos[0] as usize != n
        || sos.get(1 + 2 * n..4 + 2 * n)? != [0, 63, 0]
    {
        return None;
    }
    if n == 3 && comps.iter().map(|c| c.id).collect::<Vec<_>>() == *b"RGB" {
        return None;
    }
    let mut tabs = Vec::new();
    for (i, c) in comps.iter().enumerate() {
        let (id, t) = (sos[1 + 2 * i], sos[2 + 2 * i]);
        if id != c.id {
            return None;
        }
        tabs.push((
            dc[(t >> 4) as usize & 3].as_ref()?,
            ac[(t & 3) as usize].as_ref()?,
        ));
    }
    // One component is a non-interleaved scan: one block per MCU whatever its sampling says.
    let samp: Vec<(usize, usize)> = if n == 1 {
        vec![(1, 1)]
    } else {
        comps.iter().map(|c| (c.h, c.v)).collect()
    };
    let (hmax, vmax) = (
        samp.iter().map(|s| s.0).max()?,
        samp.iter().map(|s| s.1).max()?,
    );
    let (mcux, mcuy) = (w.div_ceil(8 * hmax), h.div_ceil(8 * vmax));
    let mut planes: Vec<(usize, Vec<u8>)> = samp
        .iter()
        .map(|&(sh, sv)| (mcux * sh * 8, vec![0u8; mcux * sh * 8 * mcuy * sv * 8]))
        .collect();
    let (buf, rst) = unstuff(d, pos);
    let mut br = BitReader {
        buf,
        rst,
        next_rst: 0,
        pos: 0,
    };
    let mut pred = vec![0i32; n];
    let mut coef = [0i32; 64];
    let mut px = [0u8; 64];
    for mcu in 0..mcux * mcuy {
        if restart > 0 && mcu > 0 && mcu % restart == 0 {
            br.restart();
            pred.iter_mut().for_each(|p| *p = 0);
        }
        let (mx, my) = (mcu % mcux, mcu / mcux);
        for (ci, &(sh, sv)) in samp.iter().enumerate() {
            let q = &qt[comps[ci].tq];
            for by in 0..sv {
                for bx in 0..sh {
                    coef.fill(0);
                    pred[ci] += br.coded(tabs[ci].0, true)?.1;
                    coef[0] = pred[ci] * q[0] as i32;
                    let mut k = 1;
                    while k < 64 {
                        let (rs, v) = br.coded(tabs[ci].1, false)?;
                        let (r, s) = ((rs >> 4) as usize, rs & 15);
                        if s == 0 {
                            if r != 15 {
                                break;
                            }
                            k += 16;
                            continue;
                        }
                        k += r;
                        if k > 63 {
                            return None;
                        }
                        coef[NATURAL[k]] = v * q[NATURAL[k]] as i32;
                        k += 1;
                    }
                    idct_islow(&coef, &mut px);
                    let (pw, plane) = &mut planes[ci];
                    let (x0, y0) = ((mx * sh + bx) * 8, (my * sv + by) * 8);
                    for y in 0..8 {
                        plane[(y0 + y) * *pw + x0..][..8].copy_from_slice(&px[y * 8..y * 8 + 8]);
                    }
                }
            }
        }
    }
    if n == 1 {
        let (pw, plane) = &planes[0];
        let px = (0..h)
            .flat_map(|y| plane[y * pw..y * pw + w].iter().copied())
            .collect();
        return Some((w, h, 1, px));
    }
    // Upsample each component to full size (only Y at full size, chroma 1x1, 2x1 or 2x2 below it).
    let mut full: Vec<Vec<u8>> = Vec::new();
    for (ci, &(sh, sv)) in samp.iter().enumerate() {
        let (pw, plane) = &planes[ci];
        let (fx, fy) = (hmax / sh, vmax / sv);
        if hmax % sh != 0 || vmax % sv != 0 {
            return None;
        }
        let (dw, dh) = ((w * sh).div_ceil(hmax), (h * sv).div_ceil(vmax));
        let row = |y: usize| &plane[y * pw..y * pw + pw];
        let ow = dw * fx;
        let mut out = vec![0u8; ow * dh * fy];
        match (fx, fy) {
            (1, 1) => {
                for y in 0..dh {
                    out[y * ow..(y + 1) * ow].copy_from_slice(&row(y)[..dw]);
                }
            }
            (2, 1) if dw >= 2 => {
                for y in 0..dh {
                    up_h2v1(row(y), dw, &mut out[y * ow..(y + 1) * ow]);
                }
            }
            (2, 2) if dw >= 2 => {
                for y in 0..dh {
                    // The rows beyond the edges are the edge rows (libjpeg's context rows).
                    let above = row(y.saturating_sub(1));
                    let below = row((y + 1).min(dh - 1));
                    up_h2v2(row(y), above, dw, &mut out[2 * y * ow..(2 * y + 1) * ow]);
                    up_h2v2(
                        row(y),
                        below,
                        dw,
                        &mut out[(2 * y + 1) * ow..(2 * y + 2) * ow],
                    );
                }
            }
            _ => return None,
        }
        let mut f = Vec::with_capacity(w * h);
        for r in out.chunks_exact(ow).take(h) {
            f.extend_from_slice(&r[..w]);
        }
        full.push(f);
    }
    let [cr_r, cb_b, cr_g, cb_g] = ycc_tables();
    let mut px = vec![0u8; w * h * 3];
    for (i, o) in px.as_chunks_mut::<3>().0.iter_mut().enumerate() {
        let (y, cb, cr) = (full[0][i] as i32, full[1][i] as usize, full[2][i] as usize);
        o[0] = (y + cr_r[cr]).clamp(0, 255) as u8;
        o[1] = (y + ((cb_g[cb] + cr_g[cr]) >> 16)).clamp(0, 255) as u8;
        o[2] = (y + cb_b[cb]).clamp(0, 255) as u8;
    }
    Some((w, h, 3, px))
}
