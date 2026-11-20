//! Y up: the turn from Blender's Z up to glTF's.

use super::*;

/// Blender (x, y, z) as the file's (x, z, -y).
pub(crate) fn y_up(v: V3) -> [f32; 3] {
    [v.x, v.z, -v.y]
}

/// A normal as the files carry them: to four decimals, then a unit vector again (corners that all but agree
/// share a vertex), and only then turned +Y up, as Blender's exporter does it (its squares are summed x, y, z as
/// Blender has them: summed as the file has them, one normal in nine comes out a unit in the last place off).
pub(crate) fn rounded_normal(n: V3) -> [f32; 3] {
    let r = n.array().map(|x| (x * 10000.0).round_ties_even() / 10000.0);
    let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    // (no normal at all, which Blender reads at a corner whose fan has no frame, a clamped bevel's: straight up,
    // as its exporter writes it; a file never carries a normal that isn't a unit vector, so one that isn't a
    // number, or has no length that can be told, is written so too)
    let unit = if len > 0.0 && len.is_finite() {
        r.map(|x| x / len)
    } else {
        [0.0, 0.0, 1.0]
    };
    // (no zero in a file of Blender's is a negative one)
    y_up(V3::new(unit[0], unit[1], unit[2])).map(|x| x + 0.0)
}

/// A part's turn as the file's quaternion (x, y, z, w), +Y up: its quaternion's, or its Euler angles' in its order.
pub(crate) fn turn_y_up(rot: V3, order: EulerOrder, quat: Option<crate::math::Quat>) -> [f32; 4] {
    match (quat, order) {
        (Some(q), _) => [q.x, q.z, -q.y, q.w],
        (None, EulerOrder::XYZ) => quat_y_up(rot),
        (None, order) => {
            let q = crate::math::eul_to_quat(rot, order);
            [q.x, q.z, -q.y, q.w]
        }
    }
}

/// A turn (Euler XYZ, radians) as the file's quaternion (x, y, z, w), +Y up.
fn quat_y_up(rot: V3) -> [f32; 4] {
    let (hx, hy, hz) = (rot.x as f64 / 2.0, rot.y as f64 / 2.0, rot.z as f64 / 2.0);
    let (cx, sx, cy, sy, cz, sz) = (cos(hx), sin(hx), cos(hy), sin(hy), cos(hz), sin(hz));
    let w = cx * cy * cz + sx * sy * sz;
    let x = sx * cy * cz - cx * sy * sz;
    let y = cx * sy * cz + sx * cy * sz;
    let z = cx * cy * sz - sx * sy * cz;
    [x as f32, z as f32, -y as f32, w as f32]
}
