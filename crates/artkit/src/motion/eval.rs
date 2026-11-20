//! Evaluating a curve at a time.

use super::*;

/// The F-curve through `b` at `time` (`fcurve_eval_keyframes`), held level beyond its ends.
pub(super) fn eval(b: &[Bez], time: f32) -> f32 {
    let (first, last) = (b[0], b[b.len() - 1]);
    if time <= first.co[0] {
        return first.co[1];
    }
    if last.co[0] <= time {
        return last.co[1];
    }
    // the key the time is on or before (`BKE_fcurve_bezt_binarysearch_index_ex`)
    let near = |a: f32, b: f32| {
        if a > b {
            a - b <= 0.0001
        } else {
            b - a <= 0.0001
        }
    };
    let (mut lo, mut hi) = (0_i64, b.len() as i64);
    let mut at = None;
    if near(time, first.co[0]) {
        at = Some(0);
    } else if near(time, last.co[0]) {
        at = Some(b.len() - 1);
    } else {
        while lo <= hi {
            let mid = lo + (hi - lo) / 2;
            let midfra = b[mid as usize].co[0];
            if near(time, midfra) {
                at = Some(mid as usize);
                break;
            }
            if time > midfra {
                lo = mid + 1;
            } else {
                hi = mid - 1;
            }
        }
    }
    if let Some(k) = at {
        return b[k].co[1];
    }
    let bezt = b[lo as usize];
    let prev = b[lo as usize - 1];
    if (bezt.co[0] - time).abs() < 1e-8 {
        return bezt.co[1];
    }
    let begin = prev.co[1];
    let change = bezt.co[1] - prev.co[1];
    let duration = bezt.co[0] - prev.co[0];
    let t = time - prev.co[0];
    if duration == 0.0 {
        return prev.co[1];
    }
    match prev.ease {
        Ease::Smooth => {
            let (v1, mut v2, mut v3, v4) = (prev.co, prev.h2, bezt.h1, bezt.co);
            if (v1[1] - v4[1]).abs() < f32::EPSILON
                && (v2[1] - v3[1]).abs() < f32::EPSILON
                && (v3[1] - v4[1]).abs() < f32::EPSILON
            {
                return v1[1];
            }
            correct_bezpart(v1, &mut v2, &mut v3, v4);
            match findzero(time, v1[0], v2[0], v3[0], v4[0]) {
                Some(u) => berekeny(v1[1], v2[1], v3[1], v4[1], u),
                None => 0.0,
            }
        }
        // (the easings are Robert Penner's equations, BSD-3-Clause, as Blender's easing.cc has them)
        Ease::Linear => change * t / duration + begin,
        Ease::In => {
            let t = t / duration;
            change * t * t + begin
        }
        Ease::Out => {
            let t = t / duration;
            -change * t * (t - 2.0) + begin
        }
        Ease::InOut => {
            let t = t / (duration / 2.0);
            if t < 1.0 {
                change / 2.0 * t * t + begin
            } else {
                let t = t - 1.0;
                -change / 2.0 * (t * (t - 2.0) - 1.0) + begin
            }
        }
        Ease::ExpoOut => {
            if t == 0.0 {
                return begin;
            }
            // (2^-10 at the end made up for, so it lands on the key)
            let (pow_min, pow_scale) = (0.000_976_562_5_f32, 1.0_f32 / (1.0 - 0.000_976_562_5_f32));
            change * (1.0 - (powf(2.0, -10.0 * t / duration) - pow_min) * pow_scale) + begin
        }
        Ease::BackOut => {
            let t = t / duration - 1.0;
            let s = prev.back;
            change * (t * t * ((s + 1.0) * t + s) + 1.0) + begin
        }
        Ease::BounceOut => {
            let mut t = t / duration;
            if t < 1.0 / 2.75 {
                return change * (7.5625 * t * t) + begin;
            }
            let lift = if t < 2.0 / 2.75 {
                t -= 1.5 / 2.75;
                0.75
            } else if t < 2.5 / 2.75 {
                t -= 2.25 / 2.75;
                0.9375
            } else {
                t -= 2.625 / 2.75;
                0.984375
            };
            change * ((7.5625 * t) * t + lift) + begin
        }
    }
}

/// `powf`, as the double rounded.
fn powf(x: f32, y: f32) -> f32 {
    crate::math::pow(x as f64, y as f64) as f32
}

/// Handles that would loop the curve back on itself pulled in to the next key's time (`BKE_fcurve_correct_bezpart`).
fn correct_bezpart(v1: [f32; 2], v2: &mut [f32; 2], v3: &mut [f32; 2], v4: [f32; 2]) {
    let h1 = [v1[0] - v2[0], v1[1] - v2[1]];
    let h2 = [v4[0] - v3[0], v4[1] - v3[1]];
    let len = v4[0] - v1[0];
    let (len1, len2) = (h1[0].abs(), h2[0].abs());
    if len1 + len2 == 0.0 {
        return;
    }
    if len1 > len {
        let fac = len / len1;
        *v2 = [v1[0] - fac * h1[0], v1[1] - fac * h1[1]];
    }
    if len2 > len {
        let fac = len / len2;
        *v3 = [v4[0] - fac * h2[0], v4[1] - fac * h2[1]];
    }
}

/// Where along the Bézier its time is `x`: the first root of the cubic in 0..1 (`findzero`, `solve_cubic`).
fn findzero(x: f32, q0: f32, q1: f32, q2: f32, q3: f32) -> Option<f32> {
    let c0 = (q0 - x) as f64;
    let c1 = (3.0 * (q1 - q0)) as f64;
    let c2 = (3.0 * (q0 - 2.0 * q1 + q2)) as f64;
    let c3 = (q3 - q0 + 3.0 * (q1 - q2)) as f64;
    let small = -1.0e-10_f64 as f32;
    let ok = |o: f32| o >= small && o <= 1.000001;
    let sqrt3d = |d: f64| {
        if d == 0.0 {
            0.0
        } else if d < 0.0 {
            -crate::math::exp(crate::math::ln(-d) / 3.0)
        } else {
            crate::math::exp(crate::math::ln(d) / 3.0)
        }
    };
    let mut roots: Vec<f32> = Vec::new();
    if c3 != 0.0 {
        let mut a = c2 / c3;
        let b = c1 / c3;
        let c = c0 / c3;
        a /= 3.0;
        let p = b / 3.0 - a * a;
        let q = (2.0 * a * a * a - a * b + c) / 2.0;
        let d = q * q + p * p * p;
        if d > 0.0 {
            let t = d.sqrt();
            roots.push((sqrt3d(-q + t) + sqrt3d(-q - t) - a) as f32);
        } else if d == 0.0 {
            let t = sqrt3d(-q);
            roots.push((2.0 * t - a) as f32);
            roots.push((-t - a) as f32);
        } else {
            let phi = crate::math::acos(-q / (-(p * p * p)).sqrt());
            let t = (-p).sqrt();
            let p = crate::math::cos(phi / 3.0);
            let q = (3.0 - 3.0 * p * p).sqrt();
            roots.push((2.0 * t * p - a) as f32);
            roots.push((-t * (p + q) - a) as f32);
            roots.push((-t * (p - q) - a) as f32);
        }
    } else if c2 != 0.0 {
        let (a, b, c) = (c2, c1, c0);
        let p = b * b - 4.0 * a * c;
        if p > 0.0 {
            let p = p.sqrt();
            roots.push(((-b - p) / (2.0 * a)) as f32);
            roots.push(((-b + p) / (2.0 * a)) as f32);
        } else if p == 0.0 {
            roots.push((-b / (2.0 * a)) as f32);
        }
    } else if c1 != 0.0 {
        roots.push((-c0 / c1) as f32);
    } else if c0 == 0.0 {
        return Some(0.0);
    }
    roots.into_iter().find(|&o| ok(o))
}

/// The Bézier's value at `t` along it (`berekeny`).
fn berekeny(f1: f32, f2: f32, f3: f32, f4: f32, t: f32) -> f32 {
    let c0 = f1;
    let c1 = 3.0 * (f2 - f1);
    let c2 = 3.0 * (f1 - 2.0 * f2 + f3);
    let c3 = f4 - f1 + 3.0 * (f2 - f3);
    c0 + t * c1 + t * t * c2 + t * t * t * c3
}

/// Blender's `back` for a back ease-out that goes `overshoot` (a share of the move) past its end: the peak of
/// 1 + (s + 1)u³ + su² (u = t - 1), found by halving.
pub fn back_amount(overshoot: f64) -> f64 {
    let peak = |s: f64| {
        let u = -2.0 * s / (3.0 * (s + 1.0));
        (s + 1.0) * u * u * u + s * u * u
    };
    let (mut lo, mut hi) = (0.0, 10.0);
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if peak(mid) < overshoot {
            lo = mid
        } else {
            hi = mid
        }
    }
    (lo + hi) / 2.0
}
