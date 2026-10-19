//! numpy's `default_rng(seed)`: PCG64 (XSL-RR 128/64) seeded through `SeedSequence`, so the
//! tools draw the same numbers the Python scripts did.

const PCG_MULT: u128 = 0x2360_ED05_1FC6_5DA4_4385_DF64_9FCC_F645;

pub struct Pcg64 {
    state: u128,
    inc: u128,
    /// The high half of the last 64-bit draw, kept for the next 32-bit draw (numpy's `has_uint32`).
    half: Option<u32>,
}

impl Pcg64 {
    /// `np.random.default_rng(seed)` for a seed below 2^32 (one entropy word).
    pub fn new(seed: u32) -> Self {
        let s = seed_sequence(&[seed], 8);
        let word = |i: usize| (s[2 * i] as u64 | (s[2 * i + 1] as u64) << 32) as u128;
        let init_state = word(0) << 64 | word(1);
        let init_seq = word(2) << 64 | word(3);
        let mut rng = Pcg64 { state: 0, inc: init_seq << 1 | 1, half: None };
        rng.step();
        rng.state = rng.state.wrapping_add(init_state);
        rng.step();
        rng
    }

    fn step(&mut self) {
        self.state = self.state.wrapping_mul(PCG_MULT).wrapping_add(self.inc);
    }

    pub fn next_u64(&mut self) -> u64 {
        self.step();
        ((self.state >> 64) as u64 ^ self.state as u64).rotate_right((self.state >> 122) as u32)
    }

    pub fn next_u32(&mut self) -> u32 {
        if let Some(h) = self.half.take() {
            return h;
        }
        let v = self.next_u64();
        self.half = Some((v >> 32) as u32);
        v as u32
    }

    /// `rng.random()`: a double in [0, 1).
    pub fn random(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    /// `rng.uniform(low, high)`. numpy's build for arm64 fuses this multiply-add.
    pub fn uniform(&mut self, low: f64, high: f64) -> f64 {
        (high - low).mul_add(self.random(), low)
    }

    /// `rng.integers(low, high)` (int64, high exclusive) for a range below 2^32: Lemire's method on
    /// buffered 32-bit draws, as numpy does.
    pub fn integers(&mut self, low: i64, high: i64) -> i64 {
        let rng = (high - low - 1) as u64;
        assert!(rng < 0xFFFF_FFFF, "integers: range too wide");
        if rng == 0 {
            return low;
        }
        let excl = rng as u32 + 1;
        let mut m = self.next_u32() as u64 * excl as u64;
        if (m as u32) < excl {
            let threshold = (u32::MAX - rng as u32) % excl;
            while (m as u32) < threshold {
                m = self.next_u32() as u64 * excl as u64;
            }
        }
        low + (m >> 32) as i64
    }
}

/// numpy's `SeedSequence(entropy).generate_state(n_words, uint32)` (pool of 4, no spawn key).
fn seed_sequence(entropy: &[u32], n_words: usize) -> Vec<u32> {
    const INIT_A: u32 = 0x43b0_d7e5;
    const MULT_A: u32 = 0x931e_8875;
    const INIT_B: u32 = 0x8b51_f9dd;
    const MULT_B: u32 = 0x58f3_8ded;
    const MIX_L: u32 = 0xca01_f9dd;
    const MIX_R: u32 = 0x4973_f715;
    const POOL: usize = 4;
    let mut hash = INIT_A;
    let mut hashmix = |v: u32| {
        let mut v = v ^ hash;
        hash = hash.wrapping_mul(MULT_A);
        v = v.wrapping_mul(hash);
        v ^ v >> 16
    };
    let mix = |x: u32, y: u32| {
        let r = MIX_L.wrapping_mul(x).wrapping_sub(MIX_R.wrapping_mul(y));
        r ^ r >> 16
    };
    let mut pool = [0u32; POOL];
    for (i, p) in pool.iter_mut().enumerate() {
        *p = hashmix(entropy.get(i).copied().unwrap_or(0));
    }
    for src in 0..POOL {
        for dst in 0..POOL {
            if src != dst {
                let h = hashmix(pool[src]);
                pool[dst] = mix(pool[dst], h);
            }
        }
    }
    for &e in entropy.iter().skip(POOL) {
        for p in pool.iter_mut() {
            let h = hashmix(e);
            *p = mix(*p, h);
        }
    }
    let mut hash = INIT_B;
    (0..n_words)
        .map(|i| {
            let mut v = pool[i % POOL] ^ hash;
            hash = hash.wrapping_mul(MULT_B);
            v = v.wrapping_mul(hash);
            v ^ v >> 16
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::Pcg64;

    // numpy 2.4: default_rng(7).random(3), default_rng(11).uniform(-0.7, 0.7) ×2,
    // default_rng(5).integers(16, 112) ×4.
    #[test]
    fn matches_numpy() {
        let mut r = Pcg64::new(7);
        let got: Vec<f64> = (0..3).map(|_| r.random()).collect();
        assert_eq!(got, [0.625095466604667, 0.8972138009695755, 0.7756856902451935]);
        let mut r = Pcg64::new(11);
        assert_eq!([r.uniform(-0.7, 0.7), r.uniform(-0.7, 0.7)], [-0.5200017161231205, -0.0010109925838390542]);
        let mut r = Pcg64::new(5);
        let got: Vec<i64> = (0..4).map(|_| r.integers(16, 112)).collect();
        assert_eq!(got, [80, 93, 18, 93]);
    }
}
