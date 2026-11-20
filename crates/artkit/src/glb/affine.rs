//! Affine matrices: composing them, and applying them to points and normals.
//!
//! Split from `glb.rs` (see `mod.rs`).

use super::*;

/// The matrix product a·b: `b`, then `a`.
pub fn compose(a: &Affine, b: &Affine) -> Affine {
    let mut out = [[0.0; 4]; 3];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, v) in row.iter_mut().enumerate() {
            *v = a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c];
        }
        row[3] += a[r][3];
    }
    out
}

/// A point through `m`.
pub fn apply(m: &Affine, p: [f32; 3]) -> [f32; 3] {
    m.map(|r| r[0] * p[0] + r[1] * p[1] + r[2] * p[2] + r[3])
}

/// A normal through `m` (its inverse transpose: right under a scale that differs by axis), unit length.
pub fn apply_normal(m: &Affine, n: [f32; 3]) -> [f32; 3] {
    let a = |r: usize, c: usize| m[r][c];
    // the cofactors of the 3×3 part: its inverse transpose times its determinant
    let cof = [
        [
            a(1, 1) * a(2, 2) - a(1, 2) * a(2, 1),
            a(1, 2) * a(2, 0) - a(1, 0) * a(2, 2),
            a(1, 0) * a(2, 1) - a(1, 1) * a(2, 0),
        ],
        [
            a(0, 2) * a(2, 1) - a(0, 1) * a(2, 2),
            a(0, 0) * a(2, 2) - a(0, 2) * a(2, 0),
            a(0, 1) * a(2, 0) - a(0, 0) * a(2, 1),
        ],
        [
            a(0, 1) * a(1, 2) - a(0, 2) * a(1, 1),
            a(0, 2) * a(1, 0) - a(0, 0) * a(1, 2),
            a(0, 0) * a(1, 1) - a(0, 1) * a(1, 0),
        ],
    ];
    let det = a(0, 0) * cof[0][0] + a(0, 1) * cof[0][1] + a(0, 2) * cof[0][2];
    let sign = if det < 0.0 { -1.0 } else { 1.0 };
    let v = cof.map(|r| (r[0] * n[0] + r[1] * n[1] + r[2] * n[2]) * sign);
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len == 0.0 { v } else { v.map(|x| x / len) }
}

/// Translation, rotation (x, y, z, w) and scale as a matrix: T·R·S.
pub fn trs(t: [f32; 3], q: [f32; 4], s: [f32; 3]) -> Affine {
    let [x, y, z, w] = q;
    let r = [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y - z * w),
            2.0 * (x * z + y * w),
        ],
        [
            2.0 * (x * y + z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z - x * w),
        ],
        [
            2.0 * (x * z - y * w),
            2.0 * (y * z + x * w),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ];
    let mut out = [[0.0; 4]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = r[i][j] * s[j];
        }
        out[i][3] = t[i];
    }
    out
}

/// A clip at a time: (animation, seconds from its start); None, the rest pose.
pub type Pose = Option<(usize, f32)>;
