//! The rotations between vectors, and the products of quaternions.

use super::*;

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

pub(crate) fn ortho(v: V3) -> V3 {
    match dominant(v) {
        0 => V3::new(-v.y - v.z, v.x, v.x),
        1 => V3::new(v.y, -v.x - v.z, v.y),
        _ => V3::new(v.z, v.z, -v.x - v.y),
    }
}

pub(crate) fn saacos(f: f32) -> f32 {
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

pub(crate) fn axis_angle_normalized_to_quat(axis: V3, angle: f32) -> Quat {
    let phi = 0.5 * angle;
    let (si, co) = (sinf(phi), cosf(phi));
    Quat {
        w: co,
        x: axis.x * si,
        y: axis.y * si,
        z: axis.z * si,
    }
}

pub(crate) fn rotation_between(a: V3, b: V3) -> Quat {
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

pub(crate) fn vec_to_quat(vec: V3, axis: usize, up: usize) -> Quat {
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

pub(crate) fn mul_qt_qtqt(a: Quat, b: Quat) -> Quat {
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
