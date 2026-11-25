//! A matrix taken apart: where it is, how it is turned and how big.

use super::*;

/// A matrix's place, Euler XYZ turn and scale (`mat4_to_loc_rot_size` and the turn through a quaternion, as an
/// object's are).
pub fn decompose(m: &M4) -> (V3, V3, V3) {
    let (loc, q, scale) = loc_quat_size(m);
    (loc, q.to_euler(EulerOrder::XYZ), scale)
}

/// A matrix's place, turn and scale as an object set to it takes them (`BKE_object_apply_mat4`): the turn a
/// quaternion, with the object's delta turn (none) taken off it as its mode has it: `euler` the order of an
/// object turned by Euler angles, None for one turned by a quaternion.
pub fn decompose_quat(m: &M4, euler: Option<EulerOrder>) -> (V3, Quat, V3) {
    let (loc, q, scale) = loc_quat_size(m);
    let delta = match euler {
        Some(order) => crate::math::eul_to_quat(V3::ZERO, order),
        None => Quat::IDENTITY.normalized(),
    };
    let inverse = Quat {
        w: delta.w,
        x: -delta.x,
        y: -delta.y,
        z: -delta.z,
    };
    (loc, inverse * q, scale)
}

/// `mat4_to_loc_rot_size`, the turn through `mat3_normalized_to_quat`.
fn loc_quat_size(m: &M4) -> (V3, Quat, V3) {
    let col = |c: usize| V3::new(m.0[c][0], m.0[c][1], m.0[c][2]);
    let (mut rot, mut size) = ([V3::ZERO; 3], [0.0_f32; 3]);
    for c in 0..3 {
        let (n, len) = crate::math::normalize_v3(col(c));
        rot[c] = n;
        size[c] = len;
    }
    if crate::math::dot_v3(rot[0].cross(rot[1]), rot[2]) < 0.0 {
        for c in 0..3 {
            rot[c] = -rot[c];
            size[c] = -size[c];
        }
    }
    let [w, x, y, z] = mat3_normalized_to_quat([rot[0].array(), rot[1].array(), rot[2].array()]);
    (
        V3::new(m.0[3][0], m.0[3][1], m.0[3][2]),
        Quat { w, x, y, z },
        V3::new(size[0], size[1], size[2]),
    )
}

/// Blender's `mat3_normalized_to_quat`: `m[column][row]`, a turned and mirrored one taken as turned.
fn mat3_normalized_to_quat(mut m: [[f32; 3]; 3]) -> [f32; 4] {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[1][0] * (m[0][1] * m[2][2] - m[0][2] * m[2][1])
        + m[2][0] * (m[0][1] * m[1][2] - m[0][2] * m[1][1]);
    if !det.is_finite() {
        m = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    } else if det < 0.0 {
        for c in &mut m {
            for x in c.iter_mut() {
                *x = -*x;
            }
        }
    }
    let mut q = [0.0_f32; 4];
    // (Mike Day's method, the larger diagonal first, w kept non-negative)
    let (big, s_index, sign_neg, rest): (f32, usize, bool, [(usize, f32); 3]);
    if m[2][2] < 0.0 {
        if m[0][0] > m[1][1] {
            big = 1.0 + m[0][0] - m[1][1] - m[2][2];
            s_index = 1;
            sign_neg = m[1][2] < m[2][1];
            rest = [
                (0, m[1][2] - m[2][1]),
                (2, m[0][1] + m[1][0]),
                (3, m[2][0] + m[0][2]),
            ];
        } else {
            big = 1.0 - m[0][0] + m[1][1] - m[2][2];
            s_index = 2;
            sign_neg = m[2][0] < m[0][2];
            rest = [
                (0, m[2][0] - m[0][2]),
                (1, m[0][1] + m[1][0]),
                (3, m[1][2] + m[2][1]),
            ];
        }
    } else if m[0][0] < -m[1][1] {
        big = 1.0 - m[0][0] - m[1][1] + m[2][2];
        s_index = 3;
        sign_neg = m[0][1] < m[1][0];
        rest = [
            (0, m[0][1] - m[1][0]),
            (1, m[2][0] + m[0][2]),
            (2, m[1][2] + m[2][1]),
        ];
    } else {
        big = 1.0 + m[0][0] + m[1][1] + m[2][2];
        s_index = 0;
        sign_neg = false;
        rest = [
            (1, m[1][2] - m[2][1]),
            (2, m[2][0] - m[0][2]),
            (3, m[0][1] - m[1][0]),
        ];
    }
    let mut s = 2.0 * big.sqrt();
    if sign_neg {
        s = -s;
    }
    q[s_index] = 0.25 * s;
    let s = 1.0 / s;
    for (i, v) in rest {
        q[i] = v * s;
    }
    if big == 1.0 && rest.iter().all(|&(i, _)| q[i] == 0.0) {
        q[s_index] = 1.0;
    }
    let len_squared = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    if (len_squared - 1.0).abs() >= 0.0002 * 3.0 {
        let k = 1.0 / len_squared.sqrt();
        for x in &mut q {
            *x *= k;
        }
    }
    q
}
