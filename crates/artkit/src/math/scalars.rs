//! The scalar functions: the `math` module of a Python float.

pub const PI: f64 = std::f64::consts::PI;
pub const TAU: f64 = std::f64::consts::TAU;

pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}
pub fn tan(x: f64) -> f64 {
    libm::tan(x)
}
pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}
pub fn acos(x: f64) -> f64 {
    libm::acos(x)
}
pub fn atan(x: f64) -> f64 {
    libm::atan(x)
}
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}
pub fn ln(x: f64) -> f64 {
    libm::log(x)
}
/// `x ** y` for floats.
pub fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}
/// `math.radians`.
pub fn radians(degrees: f64) -> f64 {
    degrees * (PI / 180.0)
}
/// `math.degrees`.
pub fn degrees(radians: f64) -> f64 {
    radians * (180.0 / PI)
}
pub fn cosd(a: f64) -> f64 {
    cos(radians(a))
}
pub fn sind(a: f64) -> f64 {
    sin(radians(a))
}

// Single-precision functions as Blender's C library (Apple's) works them out, which is not always the nearest
// single: each is the library's own code, read from it.
