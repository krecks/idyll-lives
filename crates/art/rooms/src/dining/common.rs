//! What the dining things are built from.

use super::F;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{M4, P3, PI, TAU, V3, atan2, cos, pow, sin, tan};
use artkit::parts::{Settle, join_all, settle_with};
use artkit::place::{Facing, GameFrame, Spot};
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{cube, fextrude, fsweep, loft, revolve};

/// A game position or direction as a Blender vector.
pub fn v(p: impl Spot) -> V3 {
    p.through(&F).into()
}

/// A board from game point `a` to `b`, `w` wide and `t` thick, its broad face turned towards game direction
/// `face`: slats, rails and legs at any angle.
#[allow(clippy::too_many_arguments)]
pub fn board(
    name: &str,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    w: f64,
    t: f64,
    material: Mat,
    face: (i32, i32, i32),
    bevel: f64,
) -> Obj {
    let (a, b) = (v(a), v(b));
    let z = (b - a).normalized();
    let x = v(face).cross(z).normalized();
    let y = z.cross(x);
    let obj = cube!(
        name,
        (w, t, (b - a).length()),
        material = material,
        bevel = bevel
    );
    // (the translation times the turn: its columns x, y, z and its place the middle)
    let mid = (a + b) / 2;
    let m = M4([
        [x.x, x.y, x.z, 0.0],
        [y.x, y.y, y.z, 0.0],
        [z.x, z.y, z.z, 0.0],
        [mid.x, mid.y, mid.z, 1.0],
    ]);
    set_matrix_world(obj, &m)
}

/// Game points of a polyline whose corners are bent round, about `bend` in radius (a curve through each corner,
/// `steps` segments, never past half way along a side): the path of a tube bent from one length.
pub fn fillet(
    pts: &[(f64, f64, f64)],
    bend: f64,
    steps: usize,
    closed: bool,
) -> Vec<(f64, f64, f64)> {
    let p: Vec<V3> = pts.iter().map(|&q| V3::from(q)).collect();
    let n = p.len();
    let mut out = if closed { Vec::new() } else { vec![p[0]] };
    let corners: Vec<usize> = if closed {
        (0..n).collect()
    } else {
        (1..n - 1).collect()
    };
    for i in corners {
        let (a, q, b) = (p[(i + n - 1) % n], p[i], p[(i + 1) % n]);
        let (u, w) = (a - q, b - q);
        let (lu, lw) = (u.length(), w.length());
        let (u, w) = (u.normalized(), w.normalized());
        let ang = u.angle(w);
        if ang > PI - 1e-3 {
            out.push(q);
            continue;
        }
        let t = (bend / tan(ang / 2.0)).min(0.45 * lu).min(0.45 * lw);
        let (t1, t2) = (q + u * t, q + w * t);
        for k in 0..=steps {
            let s = k as f64 / steps as f64;
            out.push(t1 * pow(1.0 - s, 2.0) + q * (2.0 * (1.0 - s) * s) + t2 * (s * s));
        }
    }
    if !closed {
        out.push(p[n - 1]);
    }
    out.iter()
        .map(|q| (q.x as f64, q.y as f64, q.z as f64))
        .collect()
}

/// A round tube of radius `r` through game points `pts`, bent round each corner (`fillet`): one length of bent
/// rod, rail or bentwood. Open ones are `tube`; a closed loop is swept round on itself.
#[allow(clippy::too_many_arguments)]
pub fn rail(
    name: &str,
    pts: &[(f64, f64, f64)],
    r: f64,
    material: Mat,
    bend: f64,
    verts: usize,
    steps: usize,
    closed: bool,
) -> Obj {
    let path = fillet(pts, bend, steps, closed);
    if closed {
        return fsweep!(F, name, &path, r, material, verts = verts, closed = true);
    }
    tube(&F, name, &path, r, material, Some(verts), 1, 1.0)
}

/// A flat ring of tube in the floor's plane, `rx` by `rz` about (x, z) at height `y`: a foot ring, a rim.
#[allow(clippy::too_many_arguments)]
pub fn ring_loop(
    name: &str,
    rx: f64,
    rz: f64,
    y: f64,
    minor: f64,
    material: Mat,
    x: f64,
    z: f64,
    verts: usize,
    n: usize,
) -> Obj {
    let pts: Vec<(f64, f64, f64)> = (0..n)
        .map(|i| {
            let a = TAU * i as f64 / n as f64;
            (x + rx * cos(a), y, z + rz * sin(a))
        })
        .collect();
    fsweep!(F, name, &pts, minor, material, verts = verts, closed = true)
}

/// A turned part (a leg, a finial, a column, a foot, a plate), `profile` [(radius, height), ...] from the
/// bottom, standing at game point `at`; edges turning more than `angle` stay crisp.
pub fn turned(
    name: &str,
    profile: &[(f64, f64)],
    at: impl Spot,
    material: Mat,
    segments: usize,
    angle: f64,
) -> Obj {
    let obj = revolve!(
        name,
        profile,
        at = at.through(&F),
        material = material,
        segments = segments
    );
    obj.edit(|o| o.mesh.shade_smooth(angle));
    obj
}

/// A table top or a seat board, `w` by `d` with corners of radius `r`, its top at `y_top`, `t` thick, its edge
/// moulded: `edge` [(inset, fraction of the thickness up), ...] from the bottom (by default a rounded nose with
/// a chamfer underneath). One piece, lofted round.
#[allow(clippy::too_many_arguments)]
pub fn slab(
    name: &str,
    w: f64,
    d: f64,
    y_top: f64,
    t: f64,
    material: Mat,
    r: f64,
    edge: Option<&[(f64, f64)]>,
    corner: usize,
    at: (f64, f64),
) -> Obj {
    let edge = edge.unwrap_or(&[
        (0.006, 0.0),
        (0.002, 0.3),
        (0.0, 0.62),
        (0.0015, 0.88),
        (0.0045, 1.0),
    ]);
    let mut sections: Vec<Vec<P3>> = Vec::new();
    for &(inset, f) in edge {
        let y = y_top - t + f * t;
        let outline = rounded_rect(
            w - 2.0 * inset,
            d - 2.0 * inset,
            (r - inset).max(0.002),
            corner,
        );
        sections.push(
            outline
                .iter()
                .map(|&(u, v)| F.at(at.0 + u, y, at.1 + v))
                .collect(),
        );
    }
    loft!(name, &sections, material = material, angle = 50)
}

/// A round top `rad` across its radius, its top at `y_top`, `t` thick, moulded as `slab`.
#[allow(clippy::too_many_arguments)]
pub fn round_slab(
    name: &str,
    rad: f64,
    y_top: f64,
    t: f64,
    material: Mat,
    segments: usize,
    edge: Option<&[(f64, f64)]>,
    at: (f64, f64),
) -> Obj {
    let edge = edge.unwrap_or(&[
        (0.006, 0.0),
        (0.002, 0.3),
        (0.0, 0.62),
        (0.0015, 0.88),
        (0.0045, 1.0),
    ]);
    let mut prof = vec![(0.0, 0.0)];
    prof.extend(edge.iter().map(|&(i, f)| (rad - i, f * t)));
    prof.push((0.0, t));
    turned(
        name,
        &prof,
        (at.0, y_top - t, at.1),
        material,
        segments,
        50.0,
    )
}

/// A square leg tapering from `s_foot` across at `y0` to `s_top` at `y1`, its corners rounded by `r`; `foot`:
/// a small chamfer at the floor. `at_top` (x, z): where its top is, for a splayed leg.
#[allow(clippy::too_many_arguments)]
pub fn sq_leg(
    name: &str,
    x: f64,
    z: f64,
    y0: f64,
    y1: f64,
    s_foot: f64,
    s_top: f64,
    material: Mat,
    r: f64,
    foot: f64,
    at_top: Option<(f64, f64)>,
) -> Obj {
    let (xt, zt) = at_top.unwrap_or((x, z));
    let mut rings = Vec::new();
    if foot != 0.0 {
        rings.push((s_foot - 2.0 * foot, y0, x, z));
        rings.push((s_foot, y0 + foot, x, z));
    } else {
        rings.push((s_foot, y0, x, z));
    }
    rings.push((s_top, y1, xt, zt));
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(s, y, cx, cz)| {
            rounded_rect(s, s, r.min(s * 0.45), 1)
                .iter()
                .map(|&(u, v)| F.at(cx + u, y, cz + v))
                .collect()
        })
        .collect();
    loft!(name, &sections, material = material, angle = 50)
}

/// An apron rail `length` long and `h` high (its top at at.1), `t` thick, its lower edge arched up by `arch`
/// in the middle between the legs: along x (`plane` front) or along the depth (side).
#[allow(clippy::too_many_arguments)]
pub fn apron(
    name: &str,
    length: f64,
    h: f64,
    t: f64,
    at: impl Spot,
    material: Mat,
    plane: Facing,
    arch: f64,
    steps: usize,
    bevel: f64,
) -> Obj {
    let half = length / 2.0;
    let lower: Vec<(f64, f64)> = (0..=steps)
        .map(|i| {
            (
                -half + length * i as f64 / steps as f64,
                -h + arch * pow(sin(PI * i as f64 / steps as f64), 1.5),
            )
        })
        .collect();
    let outline: Vec<(f64, f64)> = if arch != 0.0 {
        [vec![(half, 0.0), (-half, 0.0)], lower].concat()
    } else {
        vec![(half, 0.0), (-half, 0.0), (-half, -h), (half, -h)]
    };
    fextrude!(
        F,
        name,
        &outline,
        t,
        at,
        material,
        plane = plane,
        bevel = bevel,
        bevel_segments = 1
    )
}

/// A turned stretcher from game point `a` to `b`, swelling to `swell` times `r` in the middle.
pub fn stretcher(
    name: &str,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    r: f64,
    material: Mat,
    swell: f64,
    verts: usize,
) -> Obj {
    let m = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0, (a.2 + b.2) / 2.0);
    tube(
        &F,
        name,
        &[a, m, b],
        r,
        material,
        Some(verts),
        3,
        [0.8, swell, 0.8],
    )
}

/// A back rail from -x to x at height `y`, bowed back by `bow` in the middle (round the sitter's back), `h` high
/// and `t` thick, taller by `arch` in the middle.
#[allow(clippy::too_many_arguments)]
pub fn crest(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    h: f64,
    t: f64,
    material: Mat,
    bow: f64,
    arch: f64,
    up: (f64, f64, f64),
    resolution: usize,
) -> Obj {
    fsweep!(
        F,
        name,
        &[(-x, y, z), (0.0, y, z - bow), (x, y, z)],
        0.0,
        material,
        profile = rounded_rect(h, t, (t * 0.45).min(0.01), 1),
        up = up,
        resolution = resolution,
        scale = [1.0, arch, 1.0],
        angle = 60
    )
}

/// Joins the model into one, its feet cut flat on the floor (a leg swept up from it at a slant dips a few mm
/// below: cut, it stands where it was built, so its seat stays at its height), and settles it there.
pub fn finish(name: &str) -> Vec<Obj> {
    let obj = join_all(name);
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            v.z = v.z.max(0.0);
        }
    });
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    meshes()
}

/// Gamekit's `kit.tube`: a round tube of radius `r` through game points, its bends rounded (`round` steps between
/// each pair of points; 1 keeps them sharp), its ends capped. `verts` round: by default 8, and more for one over
/// 13 mm in radius, as many as keep its outline within a millimetre of the circle.
// KIT: kit.tube
#[allow(clippy::too_many_arguments)]
pub fn tube<P: Spot>(
    frame: &GameFrame,
    name: &str,
    points: &[P],
    r: f64,
    material: Mat,
    verts: Option<usize>,
    round: usize,
    scale: impl artkit::Arg<artkit::forms::Scale>,
) -> Obj {
    fsweep!(
        frame,
        name,
        points,
        r,
        material,
        verts = verts,
        resolution = round,
        scale = scale,
        least = 8
    )
}

/// `obj.matrix_world = m` for a part with no parent: Blender takes the matrix apart into a place, an Euler turn and
/// a scale (`BKE_object_apply_mat4`).
pub fn set_matrix_world(obj: Obj, m: &M4) -> Obj {
    let (loc, rot, scale) = decompose(m);
    obj.edit(|o| {
        o.loc = loc;
        o.rot = rot;
        o.scale = scale;
        o.quat = None;
    });
    obj
}

// KIT: motion::decompose (with its mat3_normalized_to_quat and quat_to_eul), private to artkit
/// A matrix's place, Euler XYZ turn and scale (`mat4_to_loc_rot_size` and the turn through a quaternion, as an
/// object's are).
fn decompose(m: &M4) -> (V3, V3, V3) {
    let col = |c: usize| V3::new(m.0[c][0], m.0[c][1], m.0[c][2]);
    let dot = |a: V3, b: V3| a.x * b.x + a.y * b.y + a.z * b.z;
    let (mut rot, mut size) = ([V3::ZERO; 3], [0.0_f32; 3]);
    for c in 0..3 {
        let a = col(c);
        let d = dot(a, a);
        (rot[c], size[c]) = if d > 1.0e-35 {
            let d = d.sqrt();
            let k = 1.0 / d;
            (V3::new(a.x * k, a.y * k, a.z * k), d)
        } else {
            (V3::ZERO, 0.0)
        };
    }
    if dot(rot[0].cross(rot[1]), rot[2]) < 0.0 {
        for c in 0..3 {
            rot[c] = -rot[c];
            size[c] = -size[c];
        }
    }
    let q = mat3_normalized_to_quat([rot[0].array(), rot[1].array(), rot[2].array()]);
    let e = quat_to_eul(q);
    (
        V3::new(m.0[3][0], m.0[3][1], m.0[3][2]),
        e,
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

/// Blender's `quat_to_eulO` for the XYZ order: through the turn's matrix, the smaller of its two Euler readings.
fn quat_to_eul(q: [f32; 4]) -> V3 {
    let m = artkit::math::Quat {
        w: q[0],
        x: q[1],
        y: q[2],
        z: q[3],
    }
    .to_matrix()
    .0;
    let atan2f = |y: f32, x: f32| atan2(y as f64, x as f64) as f32;
    let cy = py::hypot(m[0][0] as f64, m[0][1] as f64) as f32;
    let (e1, e2) = if cy > 0.0000375_f64 as f32 {
        (
            [
                atan2f(m[1][2], m[2][2]),
                atan2f(-m[0][2], cy),
                atan2f(m[0][1], m[0][0]),
            ],
            [
                atan2f(-m[1][2], -m[2][2]),
                atan2f(-m[0][2], -cy),
                atan2f(-m[0][1], -m[0][0]),
            ],
        )
    } else {
        let e = [atan2f(-m[2][1], m[1][1]), atan2f(-m[0][2], cy), 0.0];
        (e, e)
    };
    let d = |e: [f32; 3]| e[0].abs() + e[1].abs() + e[2].abs();
    let e = if d(e1) > d(e2) { e2 } else { e1 };
    V3::new(e[0], e[1], e[2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use artkit::math::{object_matrix, v3};

    #[test]
    fn a_matrix_set_as_a_parts_world_comes_back_as_its_place_turn_and_scale() {
        let (loc, rot, scale) = (v3(0.3, -1.2, 0.5), v3(0.2, -0.7, 1.9), v3(1.0, 1.0, 1.0));
        let (l, r, s) = decompose(&object_matrix(loc, rot, scale));
        assert_eq!(l, loc);
        for i in 0..3 {
            assert!((r[i] - rot[i]).abs() < 1e-6 && (s[i] - scale[i]).abs() < 1e-6);
        }
    }
}
