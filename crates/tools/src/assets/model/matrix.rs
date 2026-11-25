//! A node's matrix.

use super::*;

pub(super) fn local(node: &J) -> Mat {
    let get = |key: &str, default: &[f32]| -> Vec<f32> {
        node.get(key)
            .and_then(J::as_array)
            .map(|a| a.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect())
            .unwrap_or_else(|| default.to_vec())
    };
    let t = get("translation", &[0.0, 0.0, 0.0]);
    let q = get("rotation", &[0.0, 0.0, 0.0, 1.0]);
    let s = get("scale", &[1.0, 1.0, 1.0]);
    artkit::glb::trs(
        [t[0], t[1], t[2]],
        [q[0], q[1], q[2], q[3]],
        [s[0], s[1], s[2]],
    )
}
