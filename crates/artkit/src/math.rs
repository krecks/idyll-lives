//! Numbers as the family scripts had them. A Python float is an `f64` and its `math` functions are the ones
//! here (computed in software, the same on every machine); a `mathutils.Vector` is a `V3`, single precision
//! with `mathutils`' own rounding (a scalar is cast down before it multiplies, a dot product sums in double),
//! as are `Quat` and the matrices. A build repeats the scripts' arithmetic bit for bit where that decides where
//! a vertex or a perch ends up.

use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};

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

// Single-precision functions: the double one rounded, which is the nearest single (what Blender's C library
// gives), where a single-precision routine is often one unit out.

/// The sine and cosine of `x` as Blender's C library (Apple's) works both out at once, which is what Blender's
/// code gets wherever it takes the two of one angle: not always the nearest singles (one in a few thousand is a
/// unit in the last place off). In double: the angle in quarter turns, less the nearest whole one, then for each
/// a product of two quadratics in its square, each fused, rounded. (Its code, read from the library; twelve
/// million angles under 120 radians give the library's bits. Over that, where the library takes more care over
/// the quarter turns, the nearest singles.)
pub(crate) fn sincosf(x: f32) -> (f32, f32) {
    let k = f64::from_bits;
    let a = x.abs();
    if a.is_nan() || a >= 120.0 {
        return (libm::sin(x as f64) as f32, libm::cos(x as f64) as f32);
    }
    if a < f32::from_bits(0x3980_0000) {
        // (under 2^-12 the sine is the number itself and the cosine 1, but for their rounding)
        let (big, small) = (f32::from_bits(0x4c80_0000), f32::from_bits(0x3280_0000));
        return (libm::fmaf(x, big, x) * small, (big - a) * small);
    }
    let turns = x as f64 * k(0x3fe4_5f30_6dc9_c883);
    let whole = libm::rint(turns);
    let r = turns - whole;
    let r2 = r * r;
    let s = (r * k(0x3f24_bc89_9499_342c))
        * libm::fma(r2, r2 + k(0xc02c_3d3e_e4f8_713f), k(0x4044_4921_a30f_0d80))
        * libm::fma(r2, r2 + k(0xc02e_f09a_b49e_3a5b), k(0x406e_9726_4e36_bd93));
    let c = k(0x3f4d_9c36_4e97_21d5)
        * libm::fma(r2, r2 + k(0xc020_bc03_9654_0c13), k(0x401d_7821_2789_bc11))
        * libm::fma(r2, r2 + k(0xc02d_6f5a_de6c_8c37), k(0x4062_c6b9_c7f8_8fa0));
    let (mut s, mut c) = (s as f32, c as f32);
    let quarter = whole as i64;
    if quarter & 2 != 0 {
        (s, c) = (-s, -c);
    }
    if quarter & 1 != 0 { (c, -s) } else { (s, c) }
}
pub(crate) fn sinf(x: f32) -> f32 {
    sincosf(x).0
}
pub(crate) fn cosf(x: f32) -> f32 {
    sincosf(x).1
}

/// The quarter turns in an angle, of `sinf_alone` and `cosf_alone`.
const QUARTERS: u64 = 0x3fe4_5f30_6dc9_c883;

/// A sine or a cosine from the angle left once `whole` quarter turns are taken off `x`, for the quarter `quarter`.
fn of_quarter(x: f64, whole: f64, quarter: i64) -> f32 {
    let k = f64::from_bits;
    let r = libm::fma(-whole, k(0x3ff9_21fb_5444_2d18), x);
    let v = if quarter & 1 != 0 {
        cosine_near(r)
    } else {
        sine_near(r)
    };
    (if quarter & 2 != 0 { -v } else { v }) as f32
}

/// The cosine of an angle within an eighth of a turn of none, in double, as the library's own polynomial has it.
fn cosine_near(r: f64) -> f64 {
    let k = f64::from_bits;
    let z = r * r;
    let a = libm::fma(z, k(0x3ef9_9169_fc93_8f4a), k(0xbf56_c07f_1694_5df4));
    let b = libm::fma(z, k(0x3fa5_5553_c789_9699), k(0xbfdf_ffff_fcb8_2e97));
    libm::fma(z, libm::fma(z * z, a, b), 1.0)
}

/// The sine of such an angle.
fn sine_near(r: f64) -> f64 {
    let k = f64::from_bits;
    let z = r * r;
    let r3 = r * z;
    let a = libm::fma(z, k(0xbf29_943e_0fc4_818f), k(0x3f81_1073_afd1_45eb));
    let b = libm::fma(r3, k(0xbfc5_5554_5268_a02a), r);
    libm::fma(z * r3, a, b)
}

/// The sine of `x` as Blender's C library (Apple's) works it out when it is asked for alone: another routine than
/// the one that gives a sine and a cosine at once, and a few in a hundred of its results are a unit in the last
/// place off that one's. (Its code, read from the library; twelve million angles under 120 radians give the
/// library's bits; over that, the nearest single.)
pub(crate) fn sinf_alone(x: f32) -> f32 {
    let a = x.abs();
    if a.is_nan() || a >= 120.0 {
        return libm::sin(x as f64) as f32;
    }
    if a < f32::from_bits(0x3980_0000) {
        return libm::fmaf(x, f32::from_bits(0x4c80_0000), x) * f32::from_bits(0x3280_0000);
    }
    if a < f32::from_bits(0x3f49_0fdb) {
        return sine_near(x as f64) as f32;
    }
    let whole = libm::rint(x as f64 * f64::from_bits(QUARTERS));
    of_quarter(x as f64, whole, whole as i64)
}

/// The cosine of `x` as that library works it out when it is asked for alone (`sinf_alone`).
pub(crate) fn cosf_alone(x: f32) -> f32 {
    let a = x.abs();
    if a.is_nan() || a >= 120.0 {
        return libm::cos(x as f64) as f32;
    }
    if a < f32::from_bits(0x3980_0000) {
        return (f32::from_bits(0x4c80_0000) - a) * f32::from_bits(0x3280_0000);
    }
    if a < f32::from_bits(0x3f49_0fdb) {
        return cosine_near(a as f64) as f32;
    }
    let whole = libm::rint(a as f64 * f64::from_bits(QUARTERS));
    of_quarter(a as f64, whole, whole as i64 + 1)
}
/// (Of -1 it is the single just under pi, as Blender's C library has it: no angle over a half turn.)
fn acosf(x: f32) -> f32 {
    if x == -1.0 {
        return f32::from_bits(0x4049_0fda);
    }
    libm::acos(x as f64) as f32
}
/// The arc sine as Blender's C library (Apple's) works it out, which is not always the nearest single: in double,
/// a polynomial in x squared up to a half and in 1 - |x| over it, each step fused, then rounded. (Its code, read
/// from the library; every single from -1 to 1 gives the library's bits.)
fn asinf(x: f32) -> f32 {
    let (a, k) = (x.abs(), f64::from_bits);
    if a >= 1.0 {
        return if a == 1.0 {
            (k(0x3ff9_21fb_4400_0000) as f32).copysign(x)
        } else {
            f32::NAN
        };
    }
    if a > 0.5 {
        let t = (1.0 - a) as f64;
        let root = t.sqrt();
        let p3 = libm::fma(t, k(0x3f5e_e93c_b295_7053), k(0x3f61_4018_6da8_6810));
        let p5 = libm::fma(t, k(0x3f80_7e5f_a0a3_5d8b), k(0x3f9b_21ec_14e8_2132));
        let p7 = libm::fma(t, k(0x3fbe_2b8c_c317_3f9e), k(0x3ff6_a09e_64c7_c6e1));
        let t2 = t * t;
        let p5 = libm::fma(t2, p3, p5);
        let p7 = libm::fma(t2, p5, p7);
        return (libm::fma(-p7, root, k(0x3ff9_21fb_5444_2d18)) as f32).copysign(x);
    }
    if a >= f32::from_bits(0x3200_0000) {
        let xd = x as f64;
        let z = xd * xd;
        let p3 = libm::fma(z, k(0x3fa1_5e1a_450e_8b98), k(0x3f91_69f7_6663_7e01));
        let p5 = libm::fma(z, k(0x3f9f_e10b_ae95_83ba), k(0x3fa6_d55e_6da5_eec2));
        let p7 = libm::fma(z, k(0x3fb3_3344_81d0_bc2f), k(0x3fc5_5555_4d58_f2f9));
        let z2 = z * z;
        let p5 = libm::fma(z2, p3, p5);
        let p7 = libm::fma(z2, p5, p7);
        return libm::fma(xd, z * p7, xd) as f32;
    }
    // (under 2^-27 it is the number itself)
    libm::fmaf(x, f32::from_bits(0x4d00_0000), x) * f32::from_bits(0x3200_0000)
}
fn atan2f(y: f32, x: f32) -> f32 {
    libm::atan2(y as f64, x as f64) as f32
}

/// A number a script wrote, whole or not: `v3(0, 0.5, 1)`.
pub trait Num: Copy {
    fn f(self) -> f64;
}
impl Num for f64 {
    fn f(self) -> f64 {
        self
    }
}
impl Num for f32 {
    fn f(self) -> f64 {
        self as f64
    }
}
impl Num for i32 {
    fn f(self) -> f64 {
        self as f64
    }
}
impl Num for i64 {
    fn f(self) -> f64 {
        self as f64
    }
}
impl Num for usize {
    fn f(self) -> f64 {
        self as f64
    }
}

/// A point or a size as a script wrote it, a tuple of three Python floats: `at=(0.1, 0, 2.5)`.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct P3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub fn p3(x: impl Num, y: impl Num, z: impl Num) -> P3 {
    P3 {
        x: x.f(),
        y: y.f(),
        z: z.f(),
    }
}

impl P3 {
    pub const ZERO: P3 = P3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: P3 = P3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    pub fn array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

impl<A: Num, B: Num, C: Num> From<(A, B, C)> for P3 {
    fn from(t: (A, B, C)) -> P3 {
        p3(t.0, t.1, t.2)
    }
}
impl<A: Num> From<[A; 3]> for P3 {
    fn from(t: [A; 3]) -> P3 {
        p3(t[0], t[1], t[2])
    }
}
impl From<V3> for P3 {
    fn from(v: V3) -> P3 {
        p3(v.x, v.y, v.z)
    }
}
impl Index<usize> for P3 {
    type Output = f64;
    fn index(&self, i: usize) -> &f64 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("a point has three coordinates, not a {i}th"),
        }
    }
}

/// `mathutils.Vector` of three: single precision.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// `Vector((x, y, z))`: each rounded to single precision.
pub fn v3(x: impl Num, y: impl Num, z: impl Num) -> V3 {
    V3 {
        x: x.f() as f32,
        y: y.f() as f32,
        z: z.f() as f32,
    }
}

impl<A: Num, B: Num, C: Num> From<(A, B, C)> for V3 {
    fn from(t: (A, B, C)) -> V3 {
        v3(t.0, t.1, t.2)
    }
}
impl<A: Num> From<[A; 3]> for V3 {
    fn from(t: [A; 3]) -> V3 {
        v3(t[0], t[1], t[2])
    }
}
impl From<P3> for V3 {
    fn from(p: P3) -> V3 {
        v3(p.x, p.y, p.z)
    }
}

impl V3 {
    pub const ZERO: V3 = V3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const Z: V3 = V3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    pub const fn new(x: f32, y: f32, z: f32) -> V3 {
        V3 { x, y, z }
    }

    pub fn array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    /// `a.dot(b)`: the products in single precision, summed in double from the last.
    pub fn dot(self, o: V3) -> f64 {
        let mut d = 0.0_f64;
        d += (self.z * o.z) as f64;
        d += (self.y * o.y) as f64;
        d += (self.x * o.x) as f64;
        d
    }

    pub fn cross(self, o: V3) -> V3 {
        V3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    /// `v.length`.
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    /// `v.xy.length`.
    pub fn xy_length(self) -> f64 {
        let mut d = 0.0_f64;
        d += (self.y * self.y) as f64;
        d += (self.x * self.x) as f64;
        d.sqrt()
    }

    /// `v.normalized()`: zero for a vector of no length.
    pub fn normalized(self) -> V3 {
        let sq = |x: f32| x as f64 * x as f64;
        let d = sq(self.z) + sq(self.y) + sq(self.x);
        if d > 1.0e-35 {
            let k = 1.0 / (d.sqrt() as f32);
            V3::new(self.x * k, self.y * k, self.z * k)
        } else {
            V3::ZERO
        }
    }

    /// `a.lerp(b, t)`.
    pub fn lerp(self, o: V3, t: f64) -> V3 {
        let t = 1.0 - t as f32;
        let s = 1.0 - t;
        V3::new(
            o.x * s + self.x * t,
            o.y * s + self.y * t,
            o.z * s + self.z * t,
        )
    }

    /// `v.orthogonal()`: some vector square to it.
    pub fn orthogonal(self) -> V3 {
        ortho(self)
    }

    /// `a.rotation_difference(b)`: the turn that takes `a`'s direction onto `b`'s.
    pub fn rotation_difference(self, o: V3) -> Quat {
        rotation_between(normalize_v3(self).0, normalize_v3(o).0)
    }

    /// `v.to_track_quat('Z', 'Y')`: the turn that points +Z along `v`, +Y kept up.
    pub fn to_track_quat_zy(self) -> Quat {
        vec_to_quat(-self, 2, 1)
    }

    /// `v.to_track_quat(track, up)`: the turn that points the `track` axis along `v`, the `up` axis kept up
    /// (`'X', 'Z'`: a chain's links along the chain).
    pub fn to_track_quat(self, track: Axis, up: Axis) -> Quat {
        vec_to_quat(-self, track.index(), up.index())
    }

    /// `a.angle(b)`, radians.
    pub fn angle(self, o: V3) -> f64 {
        let f = |a: f32, b: f32| a as f64 * b as f64;
        let (mut dot, mut da, mut db) = (0.0, 0.0, 0.0);
        for i in 0..3 {
            da += f(self[i], self[i]);
            db += f(o[i], o[i]);
            dot += f(self[i], o[i]);
        }
        saacos((dot / (da.sqrt() * db.sqrt())) as f32) as f64
    }
}

impl Index<usize> for V3 {
    type Output = f32;
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("a vector has three coordinates, not a {i}th"),
        }
    }
}
impl IndexMut<usize> for V3 {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("a vector has three coordinates, not a {i}th"),
        }
    }
}
impl Add for V3 {
    type Output = V3;
    fn add(self, o: V3) -> V3 {
        V3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for V3 {
    type Output = V3;
    fn sub(self, o: V3) -> V3 {
        V3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl Neg for V3 {
    type Output = V3;
    fn neg(self) -> V3 {
        V3::new(-self.x, -self.y, -self.z)
    }
}
/// `v * k`: the factor cast to single precision first.
impl<N: Num> Mul<N> for V3 {
    type Output = V3;
    fn mul(self, k: N) -> V3 {
        let k = k.f() as f32;
        V3::new(self.x * k, self.y * k, self.z * k)
    }
}
impl Mul<V3> for f64 {
    type Output = V3;
    fn mul(self, v: V3) -> V3 {
        v * self
    }
}
/// `v / k`: times the reciprocal of the divisor in single precision.
impl<N: Num> Div<N> for V3 {
    type Output = V3;
    fn div(self, k: N) -> V3 {
        let k = 1.0 / (k.f() as f32);
        V3::new(self.x * k, self.y * k, self.z * k)
    }
}
impl AddAssign for V3 {
    fn add_assign(&mut self, o: V3) {
        *self = *self + o;
    }
}
impl SubAssign for V3 {
    fn sub_assign(&mut self, o: V3) {
        *self = *self - o;
    }
}
impl<N: Num> MulAssign<N> for V3 {
    fn mul_assign(&mut self, k: N) {
        *self = *self * k;
    }
}

/// `sum(vectors, Vector())`.
pub fn sum(vs: impl IntoIterator<Item = V3>) -> V3 {
    vs.into_iter().fold(V3::ZERO, |a, b| a + b)
}

// ---- Blender's own vector arithmetic (single precision throughout) ----

pub(crate) fn dot_v3(a: V3, b: V3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub(crate) fn len_v3(a: V3) -> f32 {
    dot_v3(a, a).sqrt()
}

pub(crate) fn mul_v3_fl(a: V3, k: f32) -> V3 {
    V3::new(a.x * k, a.y * k, a.z * k)
}

/// The unit vector along `a` and its length; zero for none.
pub(crate) fn normalize_v3(a: V3) -> (V3, f32) {
    let d = dot_v3(a, a);
    if d > 1.0e-35 {
        let d = d.sqrt();
        (mul_v3_fl(a, 1.0 / d), d)
    } else {
        (V3::ZERO, 0.0)
    }
}

fn dominant(v: V3) -> usize {
    let (x, y, z) = (v.x.abs(), v.y.abs(), v.z.abs());
    if x > y {
        if x > z { 0 } else { 2 }
    } else if y > z {
        1
    } else {
        2
    }
}

fn ortho(v: V3) -> V3 {
    match dominant(v) {
        0 => V3::new(-v.y - v.z, v.x, v.x),
        1 => V3::new(v.y, -v.x - v.z, v.y),
        _ => V3::new(v.z, v.z, -v.x - v.y),
    }
}

fn saacos(f: f32) -> f32 {
    acosf(f.clamp(-1.0, 1.0))
}

fn saasin(f: f32) -> f32 {
    if f <= -1.0 {
        -std::f32::consts::FRAC_PI_2
    } else if f >= 1.0 {
        std::f32::consts::FRAC_PI_2
    } else {
        asinf(f)
    }
}

/// Blender's `angle_normalized_v3v3`: the angle between two unit vectors.
pub(crate) fn angle_normalized_v3v3(a: V3, b: V3) -> f32 {
    angle_normalized(a, b)
}

fn angle_normalized(a: V3, b: V3) -> f32 {
    if dot_v3(a, b) >= 0.0 {
        2.0 * saasin(len_v3(b - a) / 2.0)
    } else {
        std::f32::consts::PI - 2.0 * saasin(len_v3(-b - a) / 2.0)
    }
}

fn axis_angle_normalized_to_quat(axis: V3, angle: f32) -> Quat {
    let phi = 0.5 * angle;
    let (si, co) = (sinf(phi), cosf(phi));
    Quat {
        w: co,
        x: axis.x * si,
        y: axis.y * si,
        z: axis.z * si,
    }
}

fn rotation_between(a: V3, b: V3) -> Quat {
    let (axis, len) = normalize_v3(a.cross(b));
    if len > f32::EPSILON {
        axis_angle_normalized_to_quat(axis, angle_normalized(a, b))
    } else if dot_v3(a, b) > 0.0 {
        Quat::IDENTITY
    } else {
        // opposed: half a turn about anything square to them
        let (axis, _) = normalize_v3(ortho(a));
        axis_angle_normalized_to_quat(axis, std::f32::consts::PI)
    }
}

fn vec_to_quat(vec: V3, axis: usize, up: usize) -> Quat {
    let len = len_v3(vec);
    if len == 0.0 {
        return Quat::IDENTITY;
    }
    let t = -vec;
    let (mut nor, mut co) = match axis {
        0 => {
            let mut n = V3::new(0.0, -t.z, t.y);
            if t.y.abs() + t.z.abs() < 1e-4 {
                n.y = 1.0;
            }
            (n, t.x)
        }
        1 => {
            let mut n = V3::new(t.z, 0.0, -t.x);
            if t.x.abs() + t.z.abs() < 1e-4 {
                n.z = 1.0;
            }
            (n, t.y)
        }
        _ => {
            let mut n = V3::new(-t.y, t.x, 0.0);
            if t.x.abs() + t.y.abs() < 1e-4 {
                n.x = 1.0;
            }
            (n, t.z)
        }
    };
    co /= len;
    nor = normalize_v3(nor).0;
    let q = axis_angle_normalized_to_quat(nor, saacos(co));
    if axis == up {
        return q;
    }
    let m = q.to_matrix();
    let fp = m.0[2];
    let angle = match (axis, up) {
        (0, 1) => 0.5 * atan2f(fp[2], fp[1]),
        (0, _) => -0.5 * atan2f(fp[1], fp[2]),
        (1, 0) => -0.5 * atan2f(fp[2], fp[0]),
        (1, _) => 0.5 * atan2f(fp[0], fp[2]),
        (_, 0) => 0.5 * atan2f(-fp[1], -fp[0]),
        _ => -0.5 * atan2f(-fp[0], -fp[1]),
    };
    let co = cosf(angle);
    let si = sinf(angle) / len;
    let q2 = Quat {
        w: co,
        x: t.x * si,
        y: t.y * si,
        z: t.z * si,
    };
    mul_qt_qtqt(q2, q)
}

fn mul_qt_qtqt(a: Quat, b: Quat) -> Quat {
    let t0 = a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z;
    let t1 = a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y;
    let t2 = a.w * b.y + a.y * b.w + a.z * b.x - a.x * b.z;
    let z = a.z * b.w + a.w * b.z + a.x * b.y - a.y * b.x;
    Quat {
        w: t0,
        x: t1,
        y: t2,
        z,
    }
}

/// `mathutils.Quaternion`: (w, x, y, z), single precision.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Quat {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat {
        w: 1.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn array(self) -> [f32; 4] {
        [self.w, self.x, self.y, self.z]
    }

    /// `Quaternion(axis, angle)`: a turn of `angle` radians about `axis` (of any length; none for no axis).
    pub fn from_axis_angle(axis: impl Into<V3>, angle: f64) -> Quat {
        let (unit, len) = normalize_v3(axis.into());
        if len == 0.0 {
            return Quat::IDENTITY;
        }
        axis_angle_normalized_to_quat(unit, angle_wrap(angle as f32))
    }

    /// The quaternion at unit length, as Blender takes an object's before it turns it (`normalize_qt`).
    pub fn normalized(self) -> Quat {
        let len = (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len == 0.0 {
            return Quat {
                w: 0.0,
                x: 1.0,
                y: 0.0,
                z: 0.0,
            };
        }
        let k = 1.0 / len;
        Quat {
            w: self.w * k,
            x: self.x * k,
            y: self.y * k,
            z: self.z * k,
        }
    }

    /// The turn as Euler angles in `order`, radians (`quat_to_eulO`).
    pub fn to_euler(self, order: EulerOrder) -> V3 {
        mat3_normalized_to_eul(&self.to_matrix(), order)
    }

    /// `q.to_matrix()`.
    pub fn to_matrix(self) -> M3 {
        let s = std::f64::consts::SQRT_2;
        let (q0, q1, q2, q3) = (
            s * self.w as f64,
            s * self.x as f64,
            s * self.y as f64,
            s * self.z as f64,
        );
        let (qda, qdb, qdc) = (q0 * q1, q0 * q2, q0 * q3);
        let (qaa, qab, qac) = (q1 * q1, q1 * q2, q1 * q3);
        let (qbb, qbc, qcc) = (q2 * q2, q2 * q3, q3 * q3);
        M3([
            [
                (1.0 - qbb - qcc) as f32,
                (qdc + qab) as f32,
                (-qdb + qac) as f32,
            ],
            [
                (-qdc + qab) as f32,
                (1.0 - qaa - qcc) as f32,
                (qda + qbc) as f32,
            ],
            [
                (qdb + qac) as f32,
                (-qda + qbc) as f32,
                (1.0 - qaa - qbb) as f32,
            ],
        ])
    }
}

/// `q @ v`: the vector turned.
impl Mul<V3> for Quat {
    type Output = V3;
    fn mul(self, r: V3) -> V3 {
        let q = self;
        let t0 = -q.x * r.x - q.y * r.y - q.z * r.z;
        let t1 = q.w * r.x + q.y * r.z - q.z * r.y;
        let t2 = q.w * r.y + q.z * r.x - q.x * r.z;
        let r2 = q.w * r.z + q.x * r.y - q.y * r.x;
        let (r0, r1) = (t1, t2);
        let t1 = t0 * -q.x + r0 * q.w - r1 * q.z + r2 * q.y;
        let t2 = t0 * -q.y + r1 * q.w - r2 * q.x + r0 * q.z;
        let z = t0 * -q.z + r2 * q.w - r0 * q.y + r1 * q.x;
        V3::new(t1, t2, z)
    }
}

/// `a @ b`: the turn `b`, then `a`.
impl Mul<Quat> for Quat {
    type Output = Quat;
    fn mul(self, o: Quat) -> Quat {
        mul_qt_qtqt(self, o)
    }
}

/// The order an object's Euler angles turn it in (its `rotation_mode`): 'XYZ' unless a script says otherwise.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum EulerOrder {
    #[default]
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

impl EulerOrder {
    /// The order's three axes and whether it is an odd one (Blender's `rotOrders`).
    fn axes(self) -> ([usize; 3], bool) {
        match self {
            EulerOrder::XYZ => ([0, 1, 2], false),
            EulerOrder::XZY => ([0, 2, 1], true),
            EulerOrder::YXZ => ([1, 0, 2], true),
            EulerOrder::YZX => ([1, 2, 0], false),
            EulerOrder::ZXY => ([2, 0, 1], false),
            EulerOrder::ZYX => ([2, 1, 0], true),
        }
    }
}

/// Euler angles in `order` (radians, as stored) as a turn's matrix (`eulO_to_mat3`).
pub fn eul_to_mat3(e: V3, order: EulerOrder) -> M3 {
    let ([i, j, k], odd) = order.axes();
    let (ti, tj, th) = if odd {
        (-e[i] as f64, -e[j] as f64, -e[k] as f64)
    } else {
        (e[i] as f64, e[j] as f64, e[k] as f64)
    };
    let (ci, cj, ch) = (cos(ti), cos(tj), cos(th));
    let (si, sj, sh) = (sin(ti), sin(tj), sin(th));
    let (cc, cs, sc, ss) = (ci * ch, ci * sh, si * ch, si * sh);
    let mut m = [[0.0_f32; 3]; 3];
    m[i][i] = (cj * ch) as f32;
    m[j][i] = (sj * sc - cs) as f32;
    m[k][i] = (sj * cc + ss) as f32;
    m[i][j] = (cj * sh) as f32;
    m[j][j] = (sj * ss + cc) as f32;
    m[k][j] = (sj * cs - sc) as f32;
    m[i][k] = (-sj) as f32;
    m[j][k] = (cj * si) as f32;
    m[k][k] = (cj * ci) as f32;
    M3(m)
}

/// Euler angles in `order` as a quaternion (`eulO_to_quat`).
pub fn eul_to_quat(e: V3, order: EulerOrder) -> Quat {
    let ([i, j, k], odd) = order.axes();
    let ti = e[i] as f64 * 0.5;
    let tj = e[j] as f64 * if odd { -0.5 } else { 0.5 };
    let th = e[k] as f64 * 0.5;
    let (ci, cj, ch) = (cos(ti), cos(tj), cos(th));
    let (si, sj, sh) = (sin(ti), sin(tj), sin(th));
    let (cc, cs, sc, ss) = (ci * ch, ci * sh, si * ch, si * sh);
    let mut a = [0.0_f64; 3];
    a[i] = cj * sc - sj * cs;
    a[j] = cj * ss + sj * cc;
    a[k] = cj * cs - sj * sc;
    let mut q = [
        (cj * cc + sj * ss) as f32,
        a[0] as f32,
        a[1] as f32,
        a[2] as f32,
    ];
    if odd {
        q[j + 1] = -q[j + 1];
    }
    Quat {
        w: q[0],
        x: q[1],
        y: q[2],
        z: q[3],
    }
}

/// A turn's matrix (its columns unit vectors) as Euler angles in `order`: the smaller of its two readings
/// (`mat3_normalized_to_eulO`).
pub fn mat3_normalized_to_eul(m: &M3, order: EulerOrder) -> V3 {
    let ([i, j, k], odd) = order.axes();
    let m = &m.0;
    let cy = crate::py::hypot(m[i][i] as f64, m[i][j] as f64) as f32;
    let (mut e1, mut e2) = ([0.0_f32; 3], [0.0_f32; 3]);
    if cy > 0.0000375_f64 as f32 {
        e1[i] = atan2f(m[j][k], m[k][k]);
        e1[j] = atan2f(-m[i][k], cy);
        e1[k] = atan2f(m[i][j], m[i][i]);
        e2[i] = atan2f(-m[j][k], -m[k][k]);
        e2[j] = atan2f(-m[i][k], -cy);
        e2[k] = atan2f(-m[i][j], -m[i][i]);
    } else {
        e1[i] = atan2f(-m[k][j], m[j][j]);
        e1[j] = atan2f(-m[i][k], cy);
        e1[k] = 0.0;
        e2 = e1;
    }
    if odd {
        e1 = e1.map(|x| -x);
        e2 = e2.map(|x| -x);
    }
    let d = |e: [f32; 3]| e[0].abs() + e[1].abs() + e[2].abs();
    let e = if d(e1) > d(e2) { e2 } else { e1 };
    V3::new(e[0], e[1], e[2])
}

/// `mathutils.Matrix` of 3 x 3, kept as Blender keeps it: `m.0[column][row]`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct M3(pub [[f32; 3]; 3]);

/// `mathutils.Matrix` of 4 x 4, kept as Blender keeps it: `m.0[column][row]`; the last column is where it moves to.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct M4(pub [[f32; 4]; 4]);

/// 'X', 'Y' or 'Z'.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn index(self) -> usize {
        self as usize
    }
}

fn angle_wrap(angle: f32) -> f32 {
    let (pi, tau) = (std::f32::consts::PI, std::f32::consts::PI * 2.0);
    let a = angle + pi;
    (a - tau * (a / tau).floor()) - pi
}

impl M3 {
    pub const IDENTITY: M3 = M3([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);

    /// `Matrix.Rotation(angle, 3, axis)`, radians.
    pub fn rotation(angle: f64, axis: Axis) -> M3 {
        let a = angle_wrap(angle as f32);
        let (c, s) = (cosf(a), sinf(a));
        M3(match axis {
            Axis::X => [[1.0, 0.0, 0.0], [0.0, c, s], [0.0, -s, c]],
            Axis::Y => [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]],
            Axis::Z => [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]],
        })
    }

    pub fn to_4x4(self) -> M4 {
        let m = self.0;
        M4([
            [m[0][0], m[0][1], m[0][2], 0.0],
            [m[1][0], m[1][1], m[1][2], 0.0],
            [m[2][0], m[2][1], m[2][2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Row by row, as `mathutils` prints a matrix.
    pub fn rows(self) -> [f32; 9] {
        let m = self.0;
        [
            m[0][0], m[1][0], m[2][0], m[0][1], m[1][1], m[2][1], m[0][2], m[1][2], m[2][2],
        ]
    }

    pub fn transposed(self) -> M3 {
        let m = self.0;
        M3([
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]],
        ])
    }
}

/// `m @ v`: each row's products in single precision, summed in double.
impl Mul<V3> for M3 {
    type Output = V3;
    fn mul(self, v: V3) -> V3 {
        let m = self.0;
        let row = |r: usize| {
            let mut d = 0.0_f64;
            for c in 0..3 {
                d += (m[c][r] * v[c]) as f64;
            }
            d as f32
        };
        V3::new(row(0), row(1), row(2))
    }
}

/// `a @ b`.
impl Mul<M3> for M3 {
    type Output = M3;
    fn mul(self, o: M3) -> M3 {
        let mut out = [[0.0_f32; 3]; 3];
        for (c, col) in out.iter_mut().enumerate() {
            for (r, cell) in col.iter_mut().enumerate() {
                let mut d = 0.0_f64;
                for i in 0..3 {
                    d += (self.0[i][r] * o.0[c][i]) as f64;
                }
                *cell = d as f32;
            }
        }
        M3(out)
    }
}

impl M4 {
    pub const IDENTITY: M4 = M4([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    /// `Matrix.Translation(v)`.
    pub fn translation(v: V3) -> M4 {
        let mut m = M4::IDENTITY;
        m.0[3] = [v.x, v.y, v.z, 1.0];
        m
    }

    /// `Matrix.Diagonal((x, y, z, 1))`.
    pub fn diagonal(v: V3) -> M4 {
        let mut m = M4::IDENTITY;
        m.0[0][0] = v.x;
        m.0[1][1] = v.y;
        m.0[2][2] = v.z;
        m
    }

    /// `Matrix.Rotation(angle, 4, axis)`, radians.
    pub fn rotation(angle: f64, axis: Axis) -> M4 {
        M3::rotation(angle, axis).to_4x4()
    }

    pub fn to_3x3(self) -> M3 {
        let m = self.0;
        M3([
            [m[0][0], m[0][1], m[0][2]],
            [m[1][0], m[1][1], m[1][2]],
            [m[2][0], m[2][1], m[2][2]],
        ])
    }

    /// `m.translation`.
    pub fn location(self) -> V3 {
        V3::new(self.0[3][0], self.0[3][1], self.0[3][2])
    }

    /// Row by row, as `mathutils` prints a matrix.
    pub fn rows(self) -> [f32; 16] {
        let m = self.0;
        let mut out = [0.0; 16];
        for r in 0..4 {
            for c in 0..4 {
                out[r * 4 + c] = m[c][r];
            }
        }
        out
    }

    pub fn is_identity(self) -> bool {
        self == M4::IDENTITY
    }

    /// `m.inverted()`: the adjugate over the determinant; None for a matrix that flattens space.
    pub fn inverted(self) -> Option<M4> {
        let m = self.0;
        let det2 = |a: f32, b: f32, c: f32, d: f32| a * d - b * c;
        let det3 = |a: [f32; 3], b: [f32; 3], c: [f32; 3]| {
            a[0] * det2(b[1], b[2], c[1], c[2]) - b[0] * det2(a[1], a[2], c[1], c[2])
                + c[0] * det2(a[1], a[2], b[1], b[2])
        };
        // row `r` without column `skip`
        let row = |r: usize, skip: usize| {
            let v: Vec<f32> = (0..4).filter(|&c| c != skip).map(|c| m[c][r]).collect();
            [v[0], v[1], v[2]]
        };
        // the determinant of what is left without row `r` and column `c`
        let minor = |r: usize, c: usize| {
            let rows: Vec<[f32; 3]> = (0..4).filter(|&k| k != r).map(|k| row(k, c)).collect();
            det3(rows[0], rows[1], rows[2])
        };
        let det = m[0][0] * minor(0, 0) - m[0][1] * minor(1, 0) + m[0][2] * minor(2, 0)
            - m[0][3] * minor(3, 0);
        if det == 0.0 {
            return None;
        }
        let mut out = [[0.0_f32; 4]; 4];
        for (c, col) in out.iter_mut().enumerate() {
            for (r, cell) in col.iter_mut().enumerate() {
                let d = minor(c, r);
                *cell = if (c + r) % 2 == 1 { -d } else { d } / det;
            }
        }
        Some(M4(out))
    }
}

/// `m @ v`: the point moved; each row's products in single precision, summed in double.
impl Mul<V3> for M4 {
    type Output = V3;
    fn mul(self, v: V3) -> V3 {
        let m = self.0;
        let p = [v.x, v.y, v.z, 1.0];
        let row = |r: usize| {
            let mut d = 0.0_f64;
            for c in 0..4 {
                d += (m[c][r] * p[c]) as f64;
            }
            d as f32
        };
        V3::new(row(0), row(1), row(2))
    }
}

/// `a @ b`.
impl Mul<M4> for M4 {
    type Output = M4;
    fn mul(self, o: M4) -> M4 {
        let mut out = [[0.0_f32; 4]; 4];
        for (c, col) in out.iter_mut().enumerate() {
            for (r, cell) in col.iter_mut().enumerate() {
                let mut d = 0.0_f64;
                for i in 0..4 {
                    d += (self.0[i][r] * o.0[c][i]) as f64;
                }
                *cell = d as f32;
            }
        }
        M4(out)
    }
}

// ---- Blender's own matrix arithmetic: what moves a mesh's vertices ----

/// A point through `m`, single precision, as Blender transforms a mesh.
pub(crate) fn mul_m4_v3(m: &M4, v: V3) -> V3 {
    let m = &m.0;
    V3::new(
        v.x * m[0][0] + v.y * m[1][0] + m[2][0] * v.z + m[3][0],
        v.x * m[0][1] + v.y * m[1][1] + m[2][1] * v.z + m[3][1],
        v.x * m[0][2] + v.y * m[1][2] + m[2][2] * v.z + m[3][2],
    )
}

/// A direction through `m`.
pub(crate) fn mul_m3_v3(m: &M3, v: V3) -> V3 {
    let m = &m.0;
    V3::new(
        m[0][0] * v.x + m[1][0] * v.y + m[2][0] * v.z,
        m[0][1] * v.x + m[1][1] * v.y + m[2][1] * v.z,
        m[0][2] * v.x + m[1][2] * v.y + m[2][2] * v.z,
    )
}

fn mul_m3_m3m3(a: &M3, b: &M3) -> M3 {
    let (a, b) = (&a.0, &b.0);
    let mut out = [[0.0_f32; 3]; 3];
    for (c, col) in out.iter_mut().enumerate() {
        for (r, cell) in col.iter_mut().enumerate() {
            *cell = b[c][0] * a[0][r] + b[c][1] * a[1][r] + b[c][2] * a[2][r];
        }
    }
    M3(out)
}

/// `a` after `b`, as Blender multiplies object matrices: each column's four products summed in two pairs.
pub(crate) fn mul_m4_m4m4(a: &M4, b: &M4) -> M4 {
    let (a, b) = (&a.0, &b.0);
    let mut out = [[0.0_f32; 4]; 4];
    for (c, col) in out.iter_mut().enumerate() {
        for (r, cell) in col.iter_mut().enumerate() {
            *cell =
                (b[c][0] * a[0][r] + b[c][1] * a[1][r]) + (b[c][2] * a[2][r] + b[c][3] * a[3][r]);
        }
    }
    M4(out)
}

/// An object's matrix as a script places it: `location`, `rot` in degrees about X, Y and Z, and `scale`.
pub fn object_matrix_of(loc: V3, rot: [f64; 3], scale: [f64; 3]) -> M4 {
    object_matrix(
        loc,
        v3(radians(rot[0]), radians(rot[1]), radians(rot[2])),
        v3(scale[0], scale[1], scale[2]),
    )
}

/// An object's matrix from where it is, how it is turned (Euler angles in `order`, radians as stored) and its scale.
pub fn object_matrix_in(loc: V3, rot: V3, order: EulerOrder, scale: V3) -> M4 {
    if order == EulerOrder::XYZ {
        return object_matrix(loc, rot, scale);
    }
    turned_matrix(loc, &eul_to_mat3(rot, order), scale)
}

/// An object's matrix from where it is, its turn as a quaternion (brought to unit length first, as Blender
/// brings an object's) and its scale.
pub fn object_matrix_quat(loc: V3, quat: Quat, scale: V3) -> M4 {
    turned_matrix(loc, &quat.normalized().to_matrix(), scale)
}

fn turned_matrix(loc: V3, rmat: &M3, scale: V3) -> M4 {
    let smat = M3([
        [scale.x, 0.0, 0.0],
        [0.0, scale.y, 0.0],
        [0.0, 0.0, scale.z],
    ]);
    let mut m = mul_m3_m3m3(rmat, &smat).to_4x4();
    m.0[3] = [loc.x, loc.y, loc.z, 1.0];
    m
}

/// An object's matrix from where it is, how it is turned (Euler XYZ, radians as stored) and its scale.
pub fn object_matrix(loc: V3, rot: V3, scale: V3) -> M4 {
    let (ti, tj, th) = (rot.x as f64, rot.y as f64, rot.z as f64);
    let (ci, cj, ch) = (cos(ti), cos(tj), cos(th));
    let (si, sj, sh) = (sin(ti), sin(tj), sin(th));
    let (cc, cs, sc, ss) = (ci * ch, ci * sh, si * ch, si * sh);
    let rmat = M3([
        [(cj * ch) as f32, (cj * sh) as f32, (-sj) as f32],
        [
            (sj * sc - cs) as f32,
            (sj * ss + cc) as f32,
            (cj * si) as f32,
        ],
        [
            (sj * cc + ss) as f32,
            (sj * cs - sc) as f32,
            (cj * ci) as f32,
        ],
    ]);
    let smat = M3([
        [scale.x, 0.0, 0.0],
        [0.0, scale.y, 0.0],
        [0.0, 0.0, scale.z],
    ]);
    let mut m = mul_m3_m3m3(&rmat, &smat).to_4x4();
    m.0[3] = [loc.x, loc.y, loc.z, 1.0];
    m
}

#[cfg(test)]
mod tests {
    use super::asinf;

    #[test]
    fn the_arc_sine_is_the_c_librarys_where_it_isnt_the_nearest_single() {
        // (number, its arc sine) as bits, from the library: twelve the nearest single isn't, then the ends
        let pairs: [(u32, u32); 17] = [
            (0x3ed2_bd22, 0x3ed9_31ce),
            (0x3f19_1eec, 0x3f24_2347),
            (0xbf0c_511d, 0xbf14_81d5),
            (0x3f07_91bc, 0x3f0e_df06),
            (0xbf00_1edd, 0xbf06_2e37),
            (0x3f43_6e01, 0x3f5e_59b6),
            (0x3f62_a526, 0x3f8b_29a7),
            (0x3f3d_93df, 0x3f55_7880),
            (0x3f1e_bde4, 0x3f2b_39d3),
            (0x3f24_8edd, 0x3f32_ba3e),
            (0x3f32_6871, 0x3f45_6521),
            (0x3f70_fc51, 0x3f9d_0187),
            (0x3f80_0000, 0x3fc9_0fda),
            (0xbf80_0000, 0xbfc9_0fda),
            (0x3f00_0000, 0x3f06_0a92),
            (0x3100_0000, 0x3100_0000),
            (0x0000_0000, 0x0000_0000),
        ];
        for (x, want) in pairs {
            assert_eq!(asinf(f32::from_bits(x)).to_bits(), want, "asinf of {x:08x}");
        }
    }
}

#[cfg(test)]
mod sines {
    use super::{cosf_alone, sincosf, sinf_alone};

    #[test]
    fn sines_and_cosines_are_the_c_librarys_where_they_arent_the_nearest_singles() {
        // (angle, sine, cosine) as bits, from the library's routine for the two at once: six where one of them isn't
        // the nearest single, then the ends
        let both: [(u32, u32, u32); 18] = [
            (0x3e55_95a1, 0x3e54_0a08, 0x3f7a_7397),
            (0xbf3c_b831, 0xbf2c_15c0, 0x3f3d_8874),
            (0x4050_3362, 0xbde3_f7ba, 0xbf7e_68b9),
            (0x41d9_7ed4, 0x3f62_ab2a, 0xbeed_f81f),
            (0xc208_9987, 0xbeca_f4af, 0xbf6b_0735),
            (0x42ef_216c, 0x3e3c_1dad, 0x3f7b_a4a6),
            (0x0000_0000, 0x0000_0000, 0x3f80_0000),
            (0x8000_0000, 0x8000_0000, 0x3f80_0000),
            (0x3727_c5ac, 0x3727_c5ac, 0x3f80_0000),
            (0xb951_b717, 0xb951_b717, 0x3f80_0000),
            (0x3980_0000, 0x3980_0000, 0x3f7f_ffff),
            (0x3f00_0000, 0x3ef5_7744, 0x3f60_a940),
            (0x3fc9_0fdb, 0x3f80_0000, 0xb33b_bd2e),
            (0x4049_0fdb, 0xb3bb_bd2e, 0xbf80_0000),
            (0xc096_cbe4, 0x3f80_0000, 0x324c_de2e),
            (0x40c9_0fdb, 0x343b_bd2e, 0x3f80_0000),
            (0x42c8_0000, 0xbf01_a12e, 0x3f5c_c0ee),
            (0x42ef_ffff, 0x3f14_a287, 0x3f50_6e74),
        ];
        for (x, sin, cos) in both {
            let (s, c) = sincosf(f32::from_bits(x));
            assert_eq!((s.to_bits(), c.to_bits()), (sin, cos), "of {x:08x}");
        }
        // and from its routines for one alone
        let alone: [(u32, u32, u32); 16] = [
            (0x3e55_95a1, 0x3e54_0a08, 0x3f7a_7397),
            (0xbf3c_b831, 0xbf2c_15c0, 0x3f3d_8874),
            (0x4050_3362, 0xbde3_f7ba, 0xbf7e_68ba),
            (0x41d9_7ed4, 0x3f62_ab2a, 0xbeed_f81f),
            (0xc208_9987, 0xbeca_f4af, 0xbf6b_0735),
            (0x42ef_216c, 0x3e3c_1dad, 0x3f7b_a4a6),
            (0x0000_0000, 0x0000_0000, 0x3f80_0000),
            (0x8000_0000, 0x8000_0000, 0x3f80_0000),
            (0x3727_c5ac, 0x3727_c5ac, 0x3f80_0000),
            (0x3980_0000, 0x3980_0000, 0x3f80_0000),
            (0x3f00_0000, 0x3ef5_7744, 0x3f60_a940),
            (0x3f49_0fdb, 0x3f35_04f3, 0x3f35_04f3),
            (0x3fc9_0fdb, 0x3f80_0000, 0xb33b_bd2e),
            (0x4049_0fdb, 0xb3bb_bd2e, 0xbf80_0000),
            (0xc096_cbe4, 0x3f80_0000, 0x324c_de2e),
            (0x42ef_ffff, 0x3f14_a287, 0x3f50_6e74),
        ];
        for (x, sin, cos) in alone {
            let x = f32::from_bits(x);
            assert_eq!(
                (sinf_alone(x).to_bits(), cosf_alone(x).to_bits()),
                (sin, cos),
                "of {x:?} alone"
            );
        }
    }
}
