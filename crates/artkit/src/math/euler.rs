//! Euler angles: the orders, and the turns to and from them.

use super::*;

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
