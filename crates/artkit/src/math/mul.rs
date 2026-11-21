//! The products of matrices with vectors and with each other.

use super::*;

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

pub(super) fn mul_m3_m3m3(a: &M3, b: &M3) -> M3 {
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
