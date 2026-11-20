//! A node's matrix, and a chain of them.
//!
//! Split from `assets/model.rs` (see `mod.rs`).

use super::*;

pub(crate) fn local(node: &J) -> Mat {
    let get = |key: &str, default: &[f32]| -> Vec<f32> {
        node.get(key)
            .and_then(J::as_array)
            .map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect())
            .unwrap_or_else(|| default.to_vec())
    };
    let t = get("translation", &[0.0, 0.0, 0.0]);
    let q = get("rotation", &[0.0, 0.0, 0.0, 1.0]);
    let s = get("scale", &[1.0, 1.0, 1.0]);
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
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
    [0, 1, 2].map(|i| [r[i][0] * s[0], r[i][1] * s[1], r[i][2] * s[2], t[i]])
}

pub(crate) fn chain(a: &Mat, b: &Mat) -> Mat {
    [0, 1, 2].map(|i| {
        let mut row = [0f32; 4];
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
        row[3] += a[i][3];
        row
    })
}
