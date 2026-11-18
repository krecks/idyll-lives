//! Python's arithmetic where Rust's differs: the family code was written in Python, and a model's numbers
//! (a perch to the millimetre, a count of sides) come out of its rounding and its `math` module.

/// `min(a, b)`: the first unless the second is below it, so of two equal ones the first (`min(0.0, -0.0)` is
/// `0.0`, where `f64::min` may give either), and the first if it is not a number.
pub fn min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `max(a, b)`: the first unless the second is above it.
pub fn max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// `min(xs)`, of one number or more: the first of the least.
pub fn min_of(xs: &[f64]) -> f64 {
    xs[1..].iter().fold(xs[0], |m, &x| min(m, x))
}

/// `max(xs)`, of one number or more: the first of the greatest.
pub fn max_of(xs: &[f64]) -> f64 {
    xs[1..].iter().fold(xs[0], |m, &x| max(m, x))
}

/// `round(x)`: to the nearest whole number, a half to the even one.
pub fn round(x: f64) -> i64 {
    x.round_ties_even() as i64
}

/// `round(x, digits)`: the float nearest the decimal that `x`, as it is in binary, rounds to (a tie to the
/// even digit), as CPython's does through its correctly rounded decimal conversion.
pub fn round_to(x: f64, digits: usize) -> f64 {
    if !x.is_finite() {
        return x;
    }
    format!("{x:.digits$}").parse().unwrap_or(x)
}

/// `a % b` for floats: the remainder takes the divisor's sign.
pub fn fmod(a: f64, b: f64) -> f64 {
    let r = libm::fmod(a, b);
    if r != 0.0 && (r < 0.0) != (b < 0.0) {
        r + b
    } else if r == 0.0 {
        0.0_f64.copysign(b)
    } else {
        r
    }
}

/// `a % b` for integers: the remainder takes the divisor's sign.
pub fn imod(a: i64, b: i64) -> i64 {
    a.rem_euclid(b) + if b < 0 && a.rem_euclid(b) != 0 { b } else { 0 }
}

/// `a // b` for integers: rounded down.
pub fn floordiv(a: i64, b: i64) -> i64 {
    a.div_euclid(b) - if b < 0 && a.rem_euclid(b) != 0 { 1 } else { 0 }
}

/// `int(x)`: towards zero.
pub fn int(x: f64) -> i64 {
    x.trunc() as i64
}

/// `sum(floats)`: CPython's (since 3.12), which carries the rounding error along (Neumaier's sum).
pub fn sum(xs: impl IntoIterator<Item = f64>) -> f64 {
    let (mut total, mut c) = (0.0_f64, 0.0_f64);
    for x in xs {
        let t = total + x;
        c += if total.abs() >= x.abs() {
            (total - t) + x
        } else {
            (x - t) + total
        };
        total = t;
    }
    if c != 0.0 && c.is_finite() {
        total + c
    } else {
        total
    }
}

/// `math.hypot(x, y)`: CPython's own (since 3.10), nearer the true length than C's `hypot`: the squares
/// summed without loss, then one correction of the root.
pub fn hypot(x: f64, y: f64) -> f64 {
    norm(&[x, y])
}

/// `math.hypot` of any number of coordinates (`math.dist` of their differences).
pub fn norm(coords: &[f64]) -> f64 {
    let mut max = 0.0_f64;
    let mut nan = false;
    let v: Vec<f64> = coords.iter().map(|c| c.abs()).collect();
    for &x in &v {
        nan |= x.is_nan();
        if x > max {
            max = x;
        }
    }
    vector_norm(&v, max, nan)
}

fn vector_norm(v: &[f64], max: f64, nan: bool) -> f64 {
    if max.is_infinite() {
        return max;
    }
    if nan {
        return f64::NAN;
    }
    if max == 0.0 || v.len() <= 1 {
        return max;
    }
    let (_, max_e) = libm::frexp(max);
    if max_e < -1023 {
        let scaled: Vec<f64> = v.iter().map(|x| x / f64::MIN_POSITIVE).collect();
        return f64::MIN_POSITIVE * vector_norm(&scaled, max / f64::MIN_POSITIVE, nan);
    }
    let scale = libm::ldexp(1.0, -max_e);
    let (mut csum, mut frac1, mut frac2) = (1.0_f64, 0.0_f64, 0.0_f64);
    // (a product and its rounding error; a sum and its rounding error)
    let mul = |x: f64, y: f64| {
        let z = x * y;
        (z, x.mul_add(y, -z))
    };
    let sum = |a: f64, b: f64| {
        let x = a + b;
        (x, (a - x) + b)
    };
    for &x in v {
        let x = x * scale;
        let pr = mul(x, x);
        let sm = sum(csum, pr.0);
        csum = sm.0;
        frac1 += pr.1;
        frac2 += sm.1;
    }
    let mut h = (csum - 1.0 + (frac1 + frac2)).sqrt();
    let pr = mul(-h, h);
    let sm = sum(csum, pr.0);
    csum = sm.0;
    frac1 += pr.1;
    frac2 += sm.1;
    let x = csum - 1.0 + (frac1 + frac2);
    h += x / (2.0 * h);
    h / scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_as_python_does() {
        assert_eq!(round(0.5), 0);
        assert_eq!(round(1.5), 2);
        assert_eq!(round(2.5), 2);
        assert_eq!(round(-0.5), 0);
        assert_eq!(round(-1.5), -2);
        // 2.675 is a little under 2.675 in binary, 1.0005 a little under too, 0.0625 a tie
        assert_eq!(round_to(2.675, 2), 2.67);
        assert_eq!(round_to(1.0005, 3), 1.0);
        assert_eq!(round_to(0.0625, 3), 0.062);
        assert_eq!(round_to(-0.0004, 3) + 0.0, 0.0);
        assert!((round_to(-0.0004, 3) + 0.0).is_sign_positive());
    }

    #[test]
    fn the_least_and_the_greatest_are_the_first_of_their_equals() {
        // CPython 3.13: min(0.0, -0.0), min(-0.0, 0.0), max(-0.0, 0.0), max(0.0, -0.0)
        assert!(min(0.0, -0.0).is_sign_positive() && min(-0.0, 0.0).is_sign_negative());
        assert!(max(-0.0, 0.0).is_sign_negative() && max(0.0, -0.0).is_sign_positive());
        assert_eq!(
            (min(2.0, 1.5), min(1.5, 2.0), max(2.0, 1.5), max(1.5, 2.0)),
            (1.5, 1.5, 2.0, 2.0)
        );
        // min(nan, 1.0) is nan and min(1.0, nan) is 1.0: nothing is below or above what is not a number
        assert!(min(f64::NAN, 1.0).is_nan() && max(f64::NAN, 1.0).is_nan());
        assert_eq!((min(1.0, f64::NAN), max(1.0, f64::NAN)), (1.0, 1.0));
        assert_eq!(min_of(&[0.012, 0.0135, 0.012]), 0.012);
        assert!(
            min_of(&[0.0, -0.0, 0.0]).is_sign_positive() && max_of(&[-0.0, 0.0]).is_sign_negative()
        );
        assert_eq!((min_of(&[3.0]), max_of(&[1.0, 3.0, 2.0])), (3.0, 3.0));
    }

    #[test]
    fn remainders_take_the_divisors_sign() {
        assert_eq!(fmod(-1.0, 3.0), 2.0);
        assert_eq!(fmod(1.0, -3.0), -2.0);
        assert_eq!(fmod(5.5, 2.0), 1.5);
        assert_eq!(imod(-1, 5), 4);
        assert_eq!(imod(7, -5), -3);
        assert_eq!(floordiv(-1, 5), -1);
        assert_eq!(floordiv(7, -5), -2);
        assert_eq!(floordiv(7, 2), 3);
        assert_eq!(int(-2.7), -2);
    }
}
