//! The single-precision sines and cosines, as `mathutils` rounds them.

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
/// The arc cosine as Blender's C library (Apple's) works it out, which is not always the nearest single (one in a
/// few thousand is a unit in the last place off): in double, up to a half a quarter turn less the arc sine; over
/// it the arc sine's polynomial in 1 - |x| by the root of that, taken from a half turn for a number under none;
/// then rounded. Of -1 it is the single just under pi: no angle over a half turn. (Its code, read from the
/// library; every single from -1 to 1 gives the library's bits.)
pub(crate) fn acosf(x: f32) -> f32 {
    let (a, k) = (x.abs(), f64::from_bits);
    if a >= 1.0 {
        return if a > 1.0 {
            f32::NAN
        } else if x > 0.0 {
            0.0
        } else {
            k(0x4009_21fb_4400_0000) as f32
        };
    }
    if a > 0.5 {
        let t = (1.0 - a) as f64;
        let (far, root) = (arc_far(t), t.sqrt());
        return if x > 0.0 {
            far * root
        } else {
            libm::fma(-far, root, k(HALF_TURN))
        } as f32;
    }
    if a >= f32::from_bits(0x3b00_0000) {
        return (k(QUARTER_TURN) - arc_near(x as f64)) as f32;
    }
    // (under 2^-9 the arc sine is the number itself)
    (k(QUARTER_TURN) - x as f64) as f32
}

/// A quarter turn and a half turn, in double.
const QUARTER_TURN: u64 = 0x3ff9_21fb_5444_2d18;
const HALF_TURN: u64 = 0x4009_21fb_5444_2d18;

/// The polynomial in `t`, 1 less a number over a half, whose product with the root of `t` is how far that
/// number's arc sine is from a quarter turn, as the library has it, each step fused.
fn arc_far(t: f64) -> f64 {
    let k = f64::from_bits;
    let p3 = libm::fma(t, k(0x3f5e_e93c_b295_7053), k(0x3f61_4018_6da8_6810));
    let p5 = libm::fma(t, k(0x3f80_7e5f_a0a3_5d8b), k(0x3f9b_21ec_14e8_2132));
    let p7 = libm::fma(t, k(0x3fbe_2b8c_c317_3f9e), k(0x3ff6_a09e_64c7_c6e1));
    let t2 = t * t;
    let p5 = libm::fma(t2, p3, p5);
    libm::fma(t2, p5, p7)
}

/// The arc sine of a number up to a half, in double, as the library's polynomial has it.
fn arc_near(x: f64) -> f64 {
    let k = f64::from_bits;
    let z = x * x;
    let p3 = libm::fma(z, k(0x3fa1_5e1a_450e_8b98), k(0x3f91_69f7_6663_7e01));
    let p5 = libm::fma(z, k(0x3f9f_e10b_ae95_83ba), k(0x3fa6_d55e_6da5_eec2));
    let p7 = libm::fma(z, k(0x3fb3_3344_81d0_bc2f), k(0x3fc5_5555_4d58_f2f9));
    let z2 = z * z;
    let p5 = libm::fma(z2, p3, p5);
    let p7 = libm::fma(z2, p5, p7);
    libm::fma(x, z * p7, x)
}

/// The arc sine as Blender's C library (Apple's) works it out, which is not always the nearest single: in double,
/// a polynomial in x squared up to a half and in 1 - |x| over it, each step fused, then rounded. (Its code, read
/// from the library; every single from -1 to 1 gives the library's bits.)
pub(crate) fn asinf(x: f32) -> f32 {
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
        return (libm::fma(-arc_far(t), t.sqrt(), k(QUARTER_TURN)) as f32).copysign(x);
    }
    if a >= f32::from_bits(0x3200_0000) {
        return arc_near(x as f64) as f32;
    }
    // (under 2^-27 it is the number itself)
    libm::fmaf(x, f32::from_bits(0x4d00_0000), x) * f32::from_bits(0x3200_0000)
}

/// The arc tangent of y over x as Blender's C library (Apple's) works it out, which is not always the nearest
/// single (about one in thirty-five is a unit in the last place off): in double, the smaller of the two over the
/// larger, the arc tangent of that as a product of four quadratics in its square, each fused, and that taken
/// from the quarter or half turn its eighth of the circle starts at, then rounded. A half turn is the single
/// just under pi (the single nearest pi is over it). (Its code, read from the library; 1.7 thousand million
/// pairs, of any size and sign, give the library's bits.)
pub(crate) fn atan2f(y: f32, x: f32) -> f32 {
    let k = f64::from_bits;
    if y.is_nan() || x.is_nan() {
        return y + x;
    }
    let (yd, xd) = (y as f64, x as f64);
    let (quarter, eighth, three) = (
        k(QUARTER_TURN),
        k(0x3fe9_21fb_5444_2d18),
        k(0x4002_d97c_7f33_21d2),
    );
    // to y's side
    let side = |a: f64| if y.is_sign_negative() { -a } else { a };
    let under_half = k(0x4009_21fb_4008_0000);
    let a = if y == x {
        if y < 0.0 {
            -three
        } else if y > 0.0 {
            eighth
        } else if x.is_sign_negative() {
            side(under_half)
        } else {
            return y;
        }
    } else if y < x {
        if -x < y {
            arc_tangent(yd / xd)
        } else if -x == y {
            -eighth
        } else {
            -quarter - arc_tangent(xd / yd)
        }
    } else if -x < y {
        quarter - arc_tangent(xd / yd)
    } else if -x == y {
        three
    } else {
        let t = yd / xd;
        if t.abs() < k(0x3e90_0000_0000_0000) {
            side(under_half)
        } else {
            arc_tangent(t) + side(k(HALF_TURN))
        }
    };
    a as f32
}

/// The arc tangent of a number from -1 to 1, in double, as the library's `atan2f` has it.
fn arc_tangent(t: f64) -> f64 {
    let k = f64::from_bits;
    let z = t * t;
    let a0 = libm::fma(z + k(0x4002_6093_7207_6e2c), z, k(0x4003_8f4c_070b_63c1));
    let a1 = libm::fma(z + k(0xc007_92d5_bf17_c0bc), z, k(0x4015_e431_68d7_c2b6));
    let b0 = libm::fma(z + k(0x3f95_3db4_9b9f_cf6b), z, k(0x400e_4fa4_7972_9042));
    let b1 = libm::fma(z + k(0xc013_e370_1467_26b3), z, k(0x401a_e0fb_08f2_dc00));
    ((a0 * b0) * k(0x3f68_0bc0_fe23_203d)) * ((a1 * b1) * t)
}
