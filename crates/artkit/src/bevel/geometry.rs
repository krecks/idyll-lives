//! The small geometry the bevel leans on: angles, normals and line intersections.

use super::*;

/// The faces' normals as the mesh the bevel works on keeps them (Blender's BMesh): a triangle or a quad of no
/// area has none, where a mesh's own is straight up.
pub(super) fn face_normals(mesh: &Mesh) -> Vec<V3> {
    (0..mesh.faces.len())
        .map(|f| {
            let p: Vec<V3> = mesh
                .corners(f)
                .iter()
                .map(|&v| mesh.verts[v as usize])
                .collect();
            match p.len() {
                3 => normal_tri(p[0], p[1], p[2]),
                4 => normalize_v3((p[0] - p[2]).cross(p[1] - p[3])).0,
                _ => normal_newell(&p),
            }
        })
        .collect()
}

pub(super) fn mid(a: V3, b: V3) -> V3 {
    V3::new(0.5 * (a.x + b.x), 0.5 * (a.y + b.y), 0.5 * (a.z + b.z))
}

pub(super) fn madd(a: V3, b: V3, k: f32) -> V3 {
    V3::new(a.x + b.x * k, a.y + b.y * k, a.z + b.z * k)
}

pub(super) fn saasin(f: f32) -> f32 {
    if f <= -1.0 {
        -std::f32::consts::FRAC_PI_2
    } else if f >= 1.0 {
        std::f32::consts::FRAC_PI_2
    } else {
        // (the single-precision routine: nearer Blender's C library's than the double one rounded)
        libm::asinf(f)
    }
}

/// The angle between two unit vectors, as Blender takes it.
pub(super) fn angle_normalized(a: V3, b: V3) -> f32 {
    if dot_v3(a, b) >= 0.0 {
        2.0 * saasin(len_v3(b - a) / 2.0)
    } else {
        std::f32::consts::PI - 2.0 * saasin(len_v3(-b - a) / 2.0)
    }
}

pub(super) fn angle(a: V3, b: V3) -> f32 {
    angle_normalized(normalize_v3(a).0, normalize_v3(b).0)
}

/// Where two lines pass nearest each other: (how they lie: 0 one line, 1 they meet, 2 skew; the point on the
/// first; the point on the second).
pub(super) fn isect_line_line(v1: V3, v2: V3, v3: V3, v4: V3) -> (u8, V3, V3) {
    let c = v3 - v1;
    let a = v2 - v1;
    let b = v4 - v3;
    let ab = a.cross(b);
    let d = dot_v3(c, ab);
    let div = dot_v3(ab, ab);
    if div == 0.0 {
        return (0, v1, v3);
    }
    if d.abs() <= 0.000_001 {
        let cb = c.cross(b);
        let i1 = v1 + mul_v3_fl(a, dot_v3(cb, ab) / div);
        return (1, i1, i1);
    }
    let t = v1 - v3;
    let n = a.cross(b);
    // the offset between the planes the lines lie in
    let t = mul_v3_fl(n, dot_v3(t, n) / dot_v3(n, n));
    let (v3t, v4t) = (v3 + t, v4 + t);
    let c = v3t - v1;
    let a = v2 - v1;
    let b = v4t - v3t;
    let ab = a.cross(b);
    let cb = c.cross(b);
    let i1 = v1 + mul_v3_fl(a, dot_v3(cb, ab) / dot_v3(ab, ab));
    (2, i1, i1 - t)
}
