//! An object's matrix: where it is, how it is turned and how big.
//!
//! Split from `crates/artkit/src/math.rs` (see `mod.rs`).

use super::*;

/// An object's matrix as a script places it: `location`, `rot` in degrees about X, Y and Z, and `scale`.
pub fn object_matrix_of(loc: V3, rot: [f64; 3], scale: [f64; 3]) -> M4 {
    object_matrix(
        loc,
        v3(radians(rot[0]), radians(rot[1]), radians(rot[2])),
        v3(scale[0], scale[1], scale[2]),
    )
}

/// An object's matrix from where it is, how it is turned (Euler angles in `order`, radians as stored) and its scale.
pub fn object_matrix_in(loc: V3, rot: V3, order: EulerOrder, scale: V3) -> M4 {
    if order == EulerOrder::XYZ {
        return object_matrix(loc, rot, scale);
    }
    turned_matrix(loc, &eul_to_mat3(rot, order), scale)
}

/// An object's matrix from where it is, its turn as a quaternion (brought to unit length first, as Blender
/// brings an object's) and its scale.
pub fn object_matrix_quat(loc: V3, quat: Quat, scale: V3) -> M4 {
    turned_matrix(loc, &quat.normalized().to_matrix(), scale)
}

fn turned_matrix(loc: V3, rmat: &M3, scale: V3) -> M4 {
    let smat = M3([
        [scale.x, 0.0, 0.0],
        [0.0, scale.y, 0.0],
        [0.0, 0.0, scale.z],
    ]);
    let mut m = mul_m3_m3m3(rmat, &smat).to_4x4();
    m.0[3] = [loc.x, loc.y, loc.z, 1.0];
    m
}

/// An object's matrix from where it is, how it is turned (Euler XYZ, radians as stored) and its scale.
pub fn object_matrix(loc: V3, rot: V3, scale: V3) -> M4 {
    let (ti, tj, th) = (rot.x as f64, rot.y as f64, rot.z as f64);
    let (ci, cj, ch) = (cos(ti), cos(tj), cos(th));
    let (si, sj, sh) = (sin(ti), sin(tj), sin(th));
    let (cc, cs, sc, ss) = (ci * ch, ci * sh, si * ch, si * sh);
    let rmat = M3([
        [(cj * ch) as f32, (cj * sh) as f32, (-sj) as f32],
        [
            (sj * sc - cs) as f32,
            (sj * ss + cc) as f32,
            (cj * si) as f32,
        ],
        [
            (sj * cc + ss) as f32,
            (sj * cs - sc) as f32,
            (cj * ci) as f32,
        ],
    ]);
    let smat = M3([
        [scale.x, 0.0, 0.0],
        [0.0, scale.y, 0.0],
        [0.0, 0.0, scale.z],
    ]);
    let mut m = mul_m3_m3m3(&rmat, &smat).to_4x4();
    m.0[3] = [loc.x, loc.y, loc.z, 1.0];
    m
}
