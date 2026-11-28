pub use core::f64::consts::{FRAC_1_SQRT_2, PI};
pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}
pub fn log(x: f64) -> f64 {
    libm::log(x)
}
pub fn log2(x: f64) -> f64 {
    libm::log2(x)
}
pub fn log10(x: f64) -> f64 {
    libm::log10(x)
}
pub fn tanh(x: f64) -> f64 {
    libm::tanh(x)
}
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}
/// `x ** y` and `Math.pow`.
pub fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}
pub fn hypot(x: f64, y: f64) -> f64 {
    libm::hypot(x, y)
}
/// `Math.sqrt` (correctly rounded everywhere).
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}
/// `Math.sign` (0 and NaN as they are).
pub fn sign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        x
    }
}
/// `Math.round`: the nearest integer, halves up (-2.5 is -2).
pub fn js_round(x: f64) -> f64 {
    let f = x.floor();
    if x - f >= 0.5 { f + 1.0 } else { f }
}
/// `Math.round` as an index or a length (one that comes out negative is 0).
pub fn round_usize(x: f64) -> usize {
    js_round(x).max(0.0) as usize
}
/// `Math.min(a, b)` / `Math.max(a, b)` (no NaN here).
pub fn min(a: f64, b: f64) -> f64 {
    if a < b { a } else { b }
}
pub fn max(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}
/// JavaScript's `ToUint32` (`x >>> 0`).
pub fn to_u32(x: f64) -> u32 {
    if !x.is_finite() {
        return 0;
    }
    x.trunc().rem_euclid(4_294_967_296.0) as u32
}
/// JavaScript's `ToInt32` (`x | 0`).
pub fn to_i32(x: f64) -> i32 {
    to_u32(x) as i32
}
