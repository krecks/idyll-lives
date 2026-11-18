//! Python's `random.Random(seed)`: the Mersenne Twister as CPython seeds and draws from it, so a build draws
//! the numbers the family scripts drew (a tree's limbs and its perches' lines stand where they stood).

const N: usize = 624;
const M: usize = 397;

/// CPython's `hash()` of a float, on a 64-bit machine: the number modulo 2^61 - 1 under its own sign (a whole
/// number under that is its own hash), -1 made -2, and 314159 for an infinity. A float is a whole number under
/// 2^53 times a power of two, and 2^61 is 1 modulo 2^61 - 1, so the power only turns the number's 61 bits round.
/// Not a NaN's: CPython hashes that by where the object lies, another number for each.
pub fn float_hash(v: f64) -> i64 {
    const BITS: u32 = 61;
    assert!(!v.is_nan(), "hash(nan): no two of CPython's are the same");
    if v.is_infinite() {
        return if v > 0.0 { 314159 } else { -314159 };
    }
    let bits = v.to_bits();
    let (exponent, fraction) = (((bits >> 52) & 0x7ff) as i32, bits & ((1 << 52) - 1));
    // |v| = m * 2^e
    let (m, e) = if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | 1 << 52, exponent - 1075)
    };
    let wide = (m as u128) << e.rem_euclid(BITS as i32);
    let x = ((wide & ((1 << BITS) - 1)) | (wide >> BITS)) as i64;
    let hash = if v.is_sign_negative() { -x } else { x };
    if hash == -1 { -2 } else { hash }
}

/// `random.Random(seed)`: `new` for an integer seed, `from_float` for a float.
#[derive(Clone)]
pub struct Random {
    mt: [u32; N],
    index: usize,
}

impl Random {
    /// Seeded as CPython seeds from an int: its absolute value, in 32-bit words from the lowest, through
    /// `init_by_array`.
    pub fn new(seed: i128) -> Random {
        let mut n = seed.unsigned_abs();
        let mut key = Vec::new();
        while n > 0 {
            key.push(n as u32);
            n >>= 32;
        }
        if key.is_empty() {
            key.push(0);
        }
        let mut mt = [0u32; N];
        mt[0] = 19650218;
        for i in 1..N {
            mt[i] = 1812433253u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..N.max(key.len()) {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1664525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1566083941))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
        }
        mt[0] = 0x8000_0000;
        Random { mt, index: N }
    }

    /// Seeded as CPython seeds from a float: from its hash (`float_hash`) read as an unsigned 64-bit number, so
    /// a negative hash has 2^64 added, it is not made positive. A whole number from 0 to under 2^61 - 1 seeds as
    /// the integer does; no other float does (`Random(-7.0)` is `Random(2**64 - 7)`, `Random(2.0**61)` is
    /// `Random(1)`).
    pub fn from_float(seed: f64) -> Random {
        Random::new(float_hash(seed) as u64 as i128)
    }

    fn word(&mut self) -> u32 {
        if self.index >= N {
            for k in 0..N {
                let y = (self.mt[k] & 0x8000_0000) | (self.mt[(k + 1) % N] & 0x7fff_ffff);
                self.mt[k] =
                    self.mt[(k + M) % N] ^ (y >> 1) ^ if y & 1 == 1 { 0x9908_b0df } else { 0 };
            }
            self.index = 0;
        }
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }

    /// `random()`: 0 <= x < 1, 53 bits of it drawn.
    pub fn random(&mut self) -> f64 {
        let a = (self.word() >> 5) as f64;
        let b = (self.word() >> 6) as f64;
        (a * 67108864.0 + b) * (1.0 / 9007199254740992.0)
    }

    /// `uniform(a, b)`.
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.random()
    }

    /// `getrandbits(k)`, for `k` up to 64.
    pub fn getrandbits(&mut self, k: u32) -> u64 {
        assert!(k <= 64, "getrandbits({k}): more than 64 bits");
        if k == 0 {
            return 0;
        }
        if k <= 32 {
            return (self.word() >> (32 - k)) as u64;
        }
        let low = self.word() as u64;
        low | ((self.word() >> (64 - k)) as u64) << 32
    }

    fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "an empty range");
        let k = 64 - n.leading_zeros();
        loop {
            let r = self.getrandbits(k);
            if r < n {
                return r;
            }
        }
    }

    /// `randrange(stop)`.
    pub fn randrange(&mut self, stop: i64) -> i64 {
        assert!(stop > 0, "randrange({stop}): an empty range");
        self.below(stop as u64) as i64
    }

    /// `randrange(start, stop, step)`, for a positive `step`.
    pub fn randrange_step(&mut self, start: i64, stop: i64, step: i64) -> i64 {
        assert!(
            step > 0 && stop > start,
            "randrange({start}, {stop}, {step}): an empty range"
        );
        let n = (stop - start + step - 1) / step;
        start + step * self.below(n as u64) as i64
    }

    /// `randint(a, b)`: a <= n <= b.
    pub fn randint(&mut self, a: i64, b: i64) -> i64 {
        self.randrange_step(a, b + 1, 1)
    }

    /// `choice(seq)` as an index into a sequence of `len`.
    pub fn index(&mut self, len: usize) -> usize {
        assert!(len > 0, "choice from an empty sequence");
        self.below(len as u64) as usize
    }

    /// `choice(seq)`.
    pub fn choice<T: Copy>(&mut self, seq: &[T]) -> T {
        seq[self.index(seq.len())]
    }
}
