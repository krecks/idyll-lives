// Math for whatever decides a town's layout, the same to the last bit on every machine and engine.
// `Math.sin`, `Math.hypot`, `Math.atan2` and the rest are only "implementation-approximated": Bun
// (JavaScriptCore) takes them from the system's libm (Apple's on macOS, glibc's on Linux) and V8 from
// its own fdlibm port, and their last bits differ, which a comparison or a rounding further on turns
// into another town for the same world code. These use only + − × ÷ and `Math.sqrt` (all exactly
// rounded by the standard), after FreeBSD's msun (fdlibm), so a world is the same wherever it is made.
// The generator (`src/game`, and the shared parts of the renderer it runs) uses these; eslint keeps
// `Math.sin` and the like out of it.

const bits = new DataView(new ArrayBuffer(8));
/** The high 32 bits of `x` (sign, exponent, the mantissa's top 20 bits). */
function high(x: number): number {
  bits.setFloat64(0, x);
  return bits.getInt32(0);
}
/** `x` with its high 32 bits set to `h`. */
function withHigh(x: number, h: number): number {
  bits.setFloat64(0, x);
  bits.setInt32(0, h);
  return bits.getFloat64(0);
}
/** 2^k, exactly (-1022 <= k <= 1023). */
function pow2(k: number): number {
  bits.setInt32(0, (k + 1023) << 20);
  bits.setInt32(4, 0);
  return bits.getFloat64(0);
}
const negative = (x: number) => x < 0 || (x === 0 && 1 / x < 0);

/** √(x² + y² (+ z²)), rounded as written (no scaling: the town's lengths are far from overflow). */
export function hypot(x: number, y: number, z = 0): number {
  return Math.sqrt(x * x + y * y + z * z);
}

// --- sin and cos (k_sin.c, k_cos.c, e_rem_pio2.c) ---
const S1 = -1.66666666666666324348e-1, S2 = 8.33333333332248946124e-3, S3 = -1.98412698298579493134e-4;
const S4 = 2.75573137070700676789e-6, S5 = -2.50507602534068634195e-8, S6 = 1.58969099521155010221e-10;
/** sin(x + y) for |x + y| <= π/4 (`y` the tail of a reduced argument). */
function kernelSin(x: number, y: number): number {
  const z = x * x;
  const v = z * x;
  const r = S2 + z * (S3 + z * (S4 + z * (S5 + z * S6)));
  return x - (z * (0.5 * y - v * r) - y - v * S1);
}
const C1 = 4.16666666666666019037e-2, C2 = -1.38888888888741095749e-3, C3 = 2.48015872894767294178e-5;
const C4 = -2.75573143513906633035e-7, C5 = 2.08757232129817482790e-9, C6 = -1.13596475577881948265e-11;
/** cos(x + y) for |x + y| <= π/4. */
function kernelCos(x: number, y: number): number {
  const z = x * x;
  let w = z * z;
  const r = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6));
  const hz = 0.5 * z;
  w = 1 - hz;
  return w + (1 - w - hz + (z * r - x * y));
}
const INV_PIO2 = 6.36619772367581382433e-1;
const PIO2_1 = 1.57079632673412561417, PIO2_1T = 6.07710050650619224932e-11;
const PIO2_2 = 6.07710050630396597660e-11, PIO2_2T = 2.02226624879595063154e-21;
const PIO2_3 = 2.02226624871116645580e-21, PIO2_3T = 8.47842766036889956997e-32;
const ROUNDER = 6755399441055744; // 1.5 × 2^52: adding it rounds to an integer (ties to even)
let reducedTail = 0;
/** x − n·π/2 for the nearest n (returned in `[n mod 4, head]`, the tail in `reducedTail`): Cody–Waite, every round taken. */
function reduce(x: number): [number, number] {
  const fn = x * INV_PIO2 + ROUNDER - ROUNDER;
  let r = x - fn * PIO2_1;
  let w = fn * PIO2_1T;
  let t = r;
  w = fn * PIO2_2;
  r = t - w;
  w = fn * PIO2_2T - (t - r - w);
  t = r;
  w = fn * PIO2_3;
  r = t - w;
  w = fn * PIO2_3T - (t - r - w);
  const y0 = r - w;
  reducedTail = r - y0 - w;
  return [((fn % 4) + 4) % 4, y0];
}
const PIO4 = 0.7853981633974483;

export function sin(x: number): number {
  if (Math.abs(x) <= PIO4) return Math.abs(x) < 7.450580596923828e-9 ? x : kernelSin(x, 0);
  if (!Number.isFinite(x)) return NaN;
  const [n, y] = reduce(x);
  const y1 = reducedTail;
  switch (n) {
    case 0: return kernelSin(y, y1);
    case 1: return kernelCos(y, y1);
    case 2: return -kernelSin(y, y1);
    default: return -kernelCos(y, y1);
  }
}

export function cos(x: number): number {
  if (Math.abs(x) <= PIO4) return Math.abs(x) < 7.450580596923828e-9 ? 1 : kernelCos(x, 0);
  if (!Number.isFinite(x)) return NaN;
  const [n, y] = reduce(x);
  const y1 = reducedTail;
  switch (n) {
    case 0: return kernelCos(y, y1);
    case 1: return -kernelSin(y, y1);
    case 2: return -kernelCos(y, y1);
    default: return kernelSin(y, y1);
  }
}

// --- atan and atan2 (s_atan.c, e_atan2.c) ---
const ATAN_HI = [4.63647609000806093515e-1, 7.85398163397448278999e-1, 9.82793723247329054082e-1, 1.57079632679489655800];
const ATAN_LO = [2.26987774529616870924e-17, 3.06161699786838301793e-17, 1.39033110312309984516e-17, 6.12323399573676603587e-17];
const AT = [
  3.33333333333329318027e-1, -1.99999999998764832476e-1, 1.42857142725034663711e-1, -1.11111104054623557880e-1, 9.09088713343650656196e-2,
  -7.69187620504482999495e-2, 6.66107313738753120669e-2, -5.83357013379057348645e-2, 4.97687799461593236017e-2, -3.65315727442169155270e-2,
  1.62858201153657823623e-2,
];

export function atan(x: number): number {
  if (Number.isNaN(x)) return x;
  const ax = Math.abs(x);
  if (ax >= 7.378697629483821e19) return x < 0 ? -ATAN_HI[3] - ATAN_LO[3] : ATAN_HI[3] + ATAN_LO[3]; // 2^66
  let id: number;
  let v: number;
  if (ax < 0.4375) {
    if (ax < 3.725290298461914e-9) return x; // 2^-28
    id = -1;
    v = x;
  } else if (ax < 1.1875) {
    if (ax < 0.6875) [id, v] = [0, (2 * ax - 1) / (2 + ax)];
    else [id, v] = [1, (ax - 1) / (ax + 1)];
  } else if (ax < 2.4375) [id, v] = [2, (ax - 1.5) / (1 + 1.5 * ax)];
  else [id, v] = [3, -1 / ax];
  const z = v * v;
  const w = z * z;
  const s1 = z * (AT[0] + w * (AT[2] + w * (AT[4] + w * (AT[6] + w * (AT[8] + w * AT[10])))));
  const s2 = w * (AT[1] + w * (AT[3] + w * (AT[5] + w * (AT[7] + w * AT[9]))));
  if (id < 0) return v - v * (s1 + s2);
  const r = ATAN_HI[id] - (v * (s1 + s2) - ATAN_LO[id] - v);
  return x < 0 ? -r : r;
}

const PI = 3.1415926535897931160, PI_LO = 1.2246467991473531772e-16, PIO2 = 1.57079632679489655800;

export function atan2(y: number, x: number): number {
  if (Number.isNaN(x) || Number.isNaN(y)) return NaN;
  if (x === 1) return atan(y);
  const m = (negative(y) ? 1 : 0) | (negative(x) ? 2 : 0);
  if (y === 0) return m === 2 ? PI : m === 3 ? -PI : y;
  if (x === 0) return y < 0 ? -PIO2 : PIO2;
  if (!Number.isFinite(x)) {
    if (!Number.isFinite(y)) return [PI / 4, -PI / 4, (3 * PI) / 4, (-3 * PI) / 4][m];
    return [0, -0, PI, -PI][m];
  }
  if (!Number.isFinite(y)) return y < 0 ? -PIO2 : PIO2;
  const a = Math.abs(y / x);
  let z: number;
  if (a > 1.152921504606847e18) return y < 0 ? -(PIO2 + 0.5 * PI_LO) : PIO2 + 0.5 * PI_LO; // 2^60
  if (m >= 2 && a < 8.673617379884035e-19) z = 0; // 2^-60
  else z = atan(a);
  switch (m) {
    case 0: return z;
    case 1: return -z;
    case 2: return PI - (z - PI_LO);
    default: return z - PI_LO - PI;
  }
}

// --- asin and acos (e_asin.c, e_acos.c) ---
const PS0 = 1.66666666666666657415e-1, PS1 = -3.25565818622400915405e-1, PS2 = 2.01212532134862925881e-1;
const PS3 = -4.00555345006794114027e-2, PS4 = 7.91534994289814532176e-4, PS5 = 3.47933107596021167570e-5;
const QS1 = -2.40339491173441421878, QS2 = 2.02094576023350569471, QS3 = -6.88283971605453293030e-1, QS4 = 7.70381505559019352791e-2;
const PIO2_LO = 6.12323399573676603587e-17;
/** The rational part both share, of z = x² or (1 − |x|)/2. */
const ratio = (z: number) => (z * (PS0 + z * (PS1 + z * (PS2 + z * (PS3 + z * (PS4 + z * PS5)))))) / (1 + z * (QS1 + z * (QS2 + z * (QS3 + z * QS4))));
/** `s` cut to its top 26 bits (Veltkamp), so its square is exact. */
function headOf(s: number): number {
  const c = 134217729 * s;
  return c - (c - s);
}

export function acos(x: number): number {
  const ax = Math.abs(x);
  if (ax >= 1) return x === 1 ? 0 : x === -1 ? PI + 2 * PIO2_LO : NaN;
  if (ax < 0.5) {
    if (ax <= 6.938893903907228e-18) return PIO2 + PIO2_LO; // 2^-57
    return PIO2 - (x - (PIO2_LO - x * ratio(x * x)));
  }
  if (x < 0) {
    const z = (1 + x) * 0.5;
    const s = Math.sqrt(z);
    const w = ratio(z) * s - PIO2_LO;
    return PI - 2 * (s + w);
  }
  const z = (1 - x) * 0.5;
  const s = Math.sqrt(z);
  const df = headOf(s);
  const c = (z - df * df) / (s + df);
  return 2 * (df + (ratio(z) * s + c));
}

export function asin(x: number): number {
  const ax = Math.abs(x);
  if (ax >= 1) return ax === 1 ? x * PIO2 + x * PIO2_LO : NaN;
  if (ax < 0.5) return ax < 7.450580596923828e-9 ? x : x + x * ratio(x * x);
  const z = (1 - ax) * 0.5;
  const s = Math.sqrt(z);
  const r = ratio(z);
  let t: number;
  if (ax >= 0.975) t = PIO2 - (2 * (s + s * r) - PIO2_LO);
  else {
    const df = headOf(s);
    const c = (z - df * df) / (s + df);
    const p = 2 * s * r - (PIO2_LO - 2 * c);
    const q = PIO2 / 2 - 2 * df;
    t = PIO2 / 2 - (p - q);
  }
  return x < 0 ? -t : t;
}

// --- exp and log (e_exp.c, e_log.c), and the powers made of them ---
const LN2_HI = 6.93147180369123816490e-1, LN2_LO = 1.90821492927058770002e-10, INV_LN2 = 1.44269504088896338700;
const P1 = 1.66666666666666019037e-1, P2 = -2.77777777770155933842e-3, P3 = 6.61375632143793436117e-5;
const P4 = -1.65339022054652515390e-6, P5 = 4.13813679705723846039e-8;

export function exp(x: number): number {
  if (Number.isNaN(x)) return x;
  if (x > 7.09782712893383973096e2) return Infinity;
  if (x < -7.45133219101941108420e2) return 0;
  const ax = Math.abs(x);
  let hi = 0, lo = 0, k = 0;
  if (ax > 0.34657359027997264) { // ln 2 / 2
    if (ax < 1.0397207708399179) { // 3 ln 2 / 2
      k = x < 0 ? -1 : 1;
      hi = x - k * LN2_HI;
      lo = k * LN2_LO;
    } else {
      k = Math.trunc(INV_LN2 * x + (x < 0 ? -0.5 : 0.5));
      hi = x - k * LN2_HI;
      lo = k * LN2_LO;
    }
    x = hi - lo;
  } else if (ax < 3.725290298461914e-9) return 1 + x; // 2^-28
  const t = x * x;
  const c = x - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
  if (k === 0) return 1 - ((x * c) / (c - 2) - x);
  const y = 1 - (lo - (x * c) / (2 - c) - hi);
  if (k >= -1021) return k === 1024 ? y * 2 * pow2(1023) : y * pow2(k);
  return y * pow2(k + 1000) * pow2(-1000);
}

const LG1 = 6.666666666666735130e-1, LG2 = 3.999999999940941908e-1, LG3 = 2.857142874366239149e-1, LG4 = 2.222219843214978396e-1;
const LG5 = 1.818357216161805012e-1, LG6 = 1.531383769920937332e-1, LG7 = 1.479819860511658591e-1;

export function log(x: number): number {
  let hx = high(x);
  let k = 0;
  if (hx < 0x00100000) {
    if (x === 0) return -Infinity;
    if (x < 0 || Number.isNaN(x)) return NaN;
    k -= 54;
    x *= 18014398509481984; // 2^54
    hx = high(x);
  }
  if (hx >= 0x7ff00000) return x + x;
  k += (hx >> 20) - 1023;
  hx &= 0x000fffff;
  const i = (hx + 0x95f64) & 0x100000;
  x = withHigh(x, hx | (i ^ 0x3ff00000));
  k += i >> 20;
  const f = x - 1;
  const dk = k;
  if ((0x000fffff & (2 + hx)) < 3) {
    if (f === 0) return k === 0 ? 0 : dk * LN2_HI + dk * LN2_LO;
    const R = f * f * (0.5 - 0.33333333333333333 * f);
    return k === 0 ? f - R : dk * LN2_HI - (R - dk * LN2_LO - f);
  }
  const s = f / (2 + f);
  const z = s * s;
  const w = z * z;
  const t1 = w * (LG2 + w * (LG4 + w * LG6));
  const t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
  const R = t2 + t1;
  if (((hx - 0x6147a) | (0x6b851 - hx)) > 0) {
    const hfsq = 0.5 * f * f;
    return k === 0 ? f - (hfsq - s * (hfsq + R)) : dk * LN2_HI - (hfsq - (s * (hfsq + R) + dk * LN2_LO) - f);
  }
  return k === 0 ? f - s * (f - R) : dk * LN2_HI - (s * (f - R) - dk * LN2_LO - f);
}

/** x^y: by multiplying for a whole `y` up to 64 either way, else e^(y ln x) (x > 0). */
export function pow(x: number, y: number): number {
  if (y === 0) return 1;
  if (Number.isNaN(x) || Number.isNaN(y)) return NaN;
  if (Number.isInteger(y) && Math.abs(y) <= 64) {
    let n = Math.abs(y);
    let b = x;
    let r = 1;
    while (n > 0) {
      if (n & 1) r *= b;
      n >>= 1;
      if (n) b *= b;
    }
    return y < 0 ? 1 / r : r;
  }
  if (x === 0) return y > 0 ? 0 : Infinity;
  if (x < 0) return NaN;
  return exp(y * log(x));
}

/** The cube root: e^(ln|x| / 3), then a Newton step. */
export function cbrt(x: number): number {
  if (x === 0 || !Number.isFinite(x)) return x;
  const ax = Math.abs(x);
  let t = exp(log(ax) / 3);
  t -= (t * t * t - ax) / (3 * t * t);
  return x < 0 ? -t : t;
}
