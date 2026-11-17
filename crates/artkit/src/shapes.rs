//! Primitives: every builder adds one mesh part centred on `at` (Blender axes, metres) with its material, and
//! returns it. Sizes are full extents (a cube of size 0.4 is 0.4 m across), matching the game's `box`/`cylinder`
//! parts. Sizes and scales go into the mesh itself. `rot` is in degrees.
//!
//! Shading: a smooth part is smooth across its edges up to `SHARP` degrees and sharp past them (a cylinder's
//! rim, a box's corners), so a cap stays flat instead of shading into the side. Something round with few sides
//! (a 6-sided leg) stays smooth round its sides (`round_angle`).
//!
//! The meshes are Blender's own primitives point for point (the same sums in single precision, the same point of
//! each merged pair kept), which the lumps, perches and far versions built on them depend on.

use crate::far::is_low;
use crate::materials::Mat;
use crate::math::{
    self, M3, M4, Num, P3, PI, TAU, V3, cosf, len_v3, mul_m4_m4m4, mul_m4_v3, sinf, v3,
};
use crate::mesh::{Mesh, UvWay};
use crate::scene::{self, Obj};
use crate::weld;

/// Degrees: an edge turning more than this stays sharp on a smooth part (a profile's 45° chamfer stays crisp).
pub const SHARP: f64 = 40.0;
/// A closed round thing (a sphere, a ring, a boulder) has no edge to keep sharp: smooth all over.
pub const CLOSED: f64 = 180.0;
/// Metres: how far a round thing's flat sides may stand in from the curve they are drawn for.
pub const FACET: f64 = 0.001;
/// Degrees: a lathe's profile is often a few points round a curve (a jar's shoulder turns 53°).
pub const LATHE_SHARP: f64 = 70.0;

/// How many flat sides a round thing of `radius` needs for its outline to stand no more than `FACET` in from
/// the circle (a polygon's sides stand r × (1 − cos(π / n)) in), in fours (so it stays the same mirrored and
/// turned a quarter): never fewer than `least` (what the part took before it was counted) nor more than `most`.
/// A far version takes `least`: nobody sees its outline to a millimetre.
pub fn sides(radius: f64, least: usize, most: usize) -> usize {
    let radius = radius.abs();
    if is_low() || radius <= FACET {
        return least;
    }
    let n = (PI / math::acos(1.0 - FACET / radius) - 1e-9).ceil();
    least.max(most.min(4 * (n / 4.0).ceil() as usize))
}

/// The sharp angle for a smooth thing `sides` round: its sides stay smooth (a hexagonal leg turns 60° a side),
/// its caps' rims (90°) stay sharp.
pub fn round_angle(sides: usize) -> f64 {
    80.0_f64.min(SHARP.max(360.0 / sides.max(1) as f64 + 10.0))
}

/// Shades `obj` smooth, its edges turning more than `angle` degrees kept sharp, or (`smooth` false) flat.
pub fn shade(obj: Obj, smooth: bool, angle: f64) -> Obj {
    obj.edit(|o| {
        if smooth {
            o.mesh.shade_smooth(angle)
        } else {
            o.mesh.shade_flat()
        }
    });
    obj
}

/// A size: one number for all three sides, or (x, y, z).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size(pub P3);

impl From<f64> for Size {
    fn from(s: f64) -> Size {
        Size(P3 { x: s, y: s, z: s })
    }
}
impl From<i32> for Size {
    fn from(s: i32) -> Size {
        Size::from(s as f64)
    }
}
impl<A: Num, B: Num, C: Num> From<(A, B, C)> for Size {
    fn from(t: (A, B, C)) -> Size {
        Size(t.into())
    }
}
impl From<P3> for Size {
    fn from(p: P3) -> Size {
        Size(p)
    }
}

macro_rules! options {
    ($(#[$doc:meta])* $name:ident { $($field:ident : $ty:ty = $default:expr),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Debug)]
        pub struct $name {
            $(pub $field: $ty,)*
        }
        impl Default for $name {
            fn default() -> $name {
                $name { $($field: $default,)* }
            }
        }
    };
}
pub(crate) use options;

options! {
    /// `cube`'s options.
    Cube { at: P3 = P3::ZERO, material: Option<Mat> = None, rot: P3 = P3::ZERO, smooth: bool = false, bevel: f64 = 0.0, bevel_segments: usize = 2 }
}
options! {
    /// `sphere`'s options.
    Sphere { at: P3 = P3::ZERO, material: Option<Mat> = None, scale: P3 = P3::ONE, rot: P3 = P3::ZERO, segments: usize = 24, rings: usize = 16, smooth: bool = true, bevel: f64 = 0.0, bevel_segments: usize = 2 }
}
options! {
    /// `cylinder`'s and `cone`'s options.
    Round { at: P3 = P3::ZERO, material: Option<Mat> = None, scale: P3 = P3::ONE, rot: P3 = P3::ZERO, verts: usize = 24, smooth: bool = true, bevel: f64 = 0.0, bevel_segments: usize = 2 }
}
options! {
    /// `rod`'s options.
    Rod { material: Option<Mat> = None, verts: usize = 12, smooth: bool = true }
}
options! {
    /// `revolve`'s options (`segments` None: by its radius, `sides`).
    Revolve { at: P3 = P3::ZERO, material: Option<Mat> = None, segments: Option<usize> = None, scale: P3 = P3::ONE, rot: P3 = P3::ZERO, smooth: bool = true, angle: f64 = LATHE_SHARP }
}
options! {
    /// `boulder`'s options.
    Boulder { at: P3 = P3::ZERO, material: Option<Mat> = None, scale: P3 = P3::ONE, rot: P3 = P3::ZERO, seed: f64 = 0.0, noise: f64 = 0.22, subdivisions: usize = 3, smooth: bool = true }
}

/// A box of `size` (x, y, z) or one number for all three.
pub fn cube(name: impl AsRef<str>, size: impl Into<Size>, o: Cube) -> Obj {
    let obj = scene::add(name.as_ref(), cube_mesh());
    scale_mesh(obj, size.into().0);
    finish(obj, o.at, o.rot, o.material, o.smooth, o.bevel, SHARP)
}

pub fn sphere(name: impl AsRef<str>, radius: f64, o: Sphere) -> Obj {
    let obj = scene::add(
        name.as_ref(),
        uv_sphere_mesh(o.segments, o.rings, radius as f32),
    );
    scale_mesh(obj, o.scale);
    finish(obj, o.at, o.rot, o.material, o.smooth, o.bevel, CLOSED)
}

/// A cylinder along z, `depth` long.
pub fn cylinder(name: impl AsRef<str>, radius: f64, depth: f64, o: Round) -> Obj {
    cone(name, radius, radius, depth, o)
}

/// A cone or frustum along z, radius `r1` at the bottom and `r2` at the top.
pub fn cone(name: impl AsRef<str>, r1: f64, r2: f64, depth: f64, o: Round) -> Obj {
    let obj = scene::add(
        name.as_ref(),
        cone_mesh(o.verts, r1 as f32, r2 as f32, depth as f32),
    );
    scale_mesh(obj, o.scale);
    obj.edit(|p| p.uv = UvWay::Round);
    finish(
        obj,
        o.at,
        o.rot,
        o.material,
        o.smooth,
        o.bevel,
        round_angle(o.verts),
    )
}

/// A cylinder of `radius` from point `a` to point `b` (both in metres).
pub fn rod(name: impl AsRef<str>, a: impl Into<V3>, b: impl Into<V3>, radius: f64, o: Rod) -> Obj {
    let (va, vb): (V3, V3) = (a.into(), b.into());
    let d = vb - va;
    let obj = cylinder(
        name,
        radius,
        d.length(),
        Round {
            at: ((va + vb) / 2.0).into(),
            material: o.material,
            verts: o.verts,
            smooth: o.smooth,
            ..Round::default()
        },
    );
    obj.edit(|p| p.quat = Some(d.to_track_quat_zy()));
    obj
}

/// Surface of revolution around z. `profile` is [(r, z), ...] in metres, top last. Smooth, it stays smooth
/// where the profile turns less than `angle` degrees (a few points round a jar's shoulder) and sharp where it
/// turns more (a rim, a step, the edge of its base). `segments` round: by default 24, and more for one over
/// 12 cm in radius, as many as keep its outline within a millimetre of the circle (`sides`).
pub fn revolve(name: impl AsRef<str>, profile: &[(f64, f64)], o: Revolve) -> Obj {
    let segments = o.segments.unwrap_or_else(|| {
        let widest = profile.iter().map(|p| p.0.abs()).fold(0.0, f64::max);
        sides(widest * o.scale.x.abs().max(o.scale.y.abs()), 24, 64)
    });
    let n = profile.len();
    let mut verts = Vec::with_capacity(segments * n);
    for s in 0..segments {
        let a = TAU * s as f64 / segments as f64;
        let (ca, sa) = (math::cos(a), math::sin(a));
        for &(r, z) in profile {
            verts.push(v3(r * ca, r * sa, z));
        }
    }
    let mut faces = Vec::new();
    for s in 0..segments {
        let s2 = (s + 1) % segments;
        for i in 0..n - 1 {
            if profile[i].0.abs() < 1e-6 && profile[i + 1].0.abs() < 1e-6 {
                continue; // both ends on the axis: the quad collapses
            }
            faces.push(vec![
                (s * n + i) as u32,
                (s * n + i + 1) as u32,
                (s2 * n + i + 1) as u32,
                (s2 * n + i) as u32,
            ]);
        }
    }
    let mut mesh = Mesh::from_pydata(verts, &faces);
    mesh.set_material(o.material);
    mesh.recalc_normals();
    let obj = scene::add(name.as_ref(), mesh);
    shade(obj, o.smooth, o.angle.max(round_angle(segments)));
    obj.edit(|p| p.uv = UvWay::Round);
    obj.set_location(o.at)
        .set_rotation(o.rot)
        .set_scale(o.scale)
}

/// A lumpy, deterministic boulder: an icosphere displaced by noise, bottom on z = 0.
pub fn boulder(name: impl AsRef<str>, radius: f64, o: Boulder) -> Obj {
    let obj = scene::add(
        name.as_ref(),
        ico_sphere_mesh(o.subdivisions, radius as f32),
    );
    finish(obj, o.at, o.rot, o.material, o.smooth, 0.0, CLOSED);
    obj.edit(|p| {
        for v in &mut p.mesh.verts {
            let n = crate::noise::noise(v3(
                v.x as f64 * 1.6 + o.seed,
                v.y as f64 * 1.6 - o.seed,
                v.z as f64 * 1.6 + o.seed * 0.5,
            ));
            *v *= 1.0 + o.noise * n;
        }
        let low = p
            .mesh
            .verts
            .iter()
            .map(|v| v.z)
            .fold(f32::INFINITY, f32::min);
        for v in &mut p.mesh.verts {
            v.z -= low;
        }
    });
    obj.set_scale(o.scale)
}

/// Scales the new part's mesh (not the part).
fn scale_mesh(obj: Obj, scale: P3) {
    if scale != P3::ONE {
        obj.edit(|o| o.mesh.transform(&M4::diagonal(scale.into())));
    }
}

fn finish(
    obj: Obj,
    at: P3,
    rot: P3,
    material: Option<Mat>,
    smooth: bool,
    bevel: f64,
    angle: f64,
) -> Obj {
    assert!(
        bevel == 0.0,
        "{}: bevelled edges aren't built yet",
        obj.name()
    );
    obj.edit(|o| o.mesh.set_material(material));
    if smooth {
        shade(obj, true, angle);
    }
    obj.set_location(at).set_rotation(rot)
}

// ---- Blender's primitives ----

/// Blender's primitive meshes as its operators make them, before a builder scales, shades and places them.
pub mod primitive {
    use crate::mesh::Mesh;

    /// `primitive_uv_sphere_add(segments, ring_count, radius)`.
    pub fn uv_sphere(segments: usize, rings: usize, radius: f32) -> Mesh {
        super::uv_sphere_mesh(segments, rings, radius)
    }
    /// `primitive_cone_add(vertices, radius1, radius2, depth)`.
    pub fn cone(vertices: usize, r1: f32, r2: f32, depth: f32) -> Mesh {
        super::cone_mesh(vertices, r1, r2, depth)
    }
    /// `primitive_cube_add(size=1)`.
    pub fn cube() -> Mesh {
        super::cube_mesh()
    }
    /// `primitive_ico_sphere_add(subdivisions, radius)`.
    pub fn ico_sphere(subdivisions: usize, radius: f32) -> Mesh {
        super::ico_sphere_mesh(subdivisions, radius)
    }
}

/// The sine and cosine of `numerator` / `denominator` of a turn, as Blender takes them for points round a
/// circle: from the nearest eighth, so they come out the same mirrored.
pub(crate) fn sin_cos_from_fraction(numerator: i32, denominator: i32) -> (f32, f32) {
    let mut num = numerator * 8;
    let octant = num / denominator;
    let den = denominator * 8;
    let (mut cos_sign, mut swap) = (1.0_f32, false);
    match octant {
        0 => {}
        1 | 2 => {
            num = den / 4 - num;
            swap = true;
        }
        3 | 4 => {
            num = den / 2 - num;
            cos_sign = -1.0;
        }
        5 | 6 => {
            num -= den * 3 / 4;
            cos_sign = -1.0;
            swap = true;
        }
        7 => num -= den,
        _ => {}
    }
    let phi = (num as f32 / den as f32) * (2.0 * PI) as f32;
    let (s, c) = (sinf(phi), cos_sign * cosf(phi));
    if swap { (c, s) } else { (s, c) }
}

/// `primitive_cube_add(size=1)`.
pub(crate) fn cube_mesh() -> Mesh {
    let mut verts = Vec::with_capacity(8);
    for x in [-0.5_f32, 0.5] {
        for y in [-0.5_f32, 0.5] {
            for z in [-0.5_f32, 0.5] {
                verts.push(V3::new(x, y, z));
            }
        }
    }
    let faces: [[u32; 4]; 6] = [
        [0, 1, 3, 2],
        [2, 3, 7, 6],
        [6, 7, 5, 4],
        [4, 5, 1, 0],
        [2, 6, 4, 0],
        [7, 3, 1, 5],
    ];
    Mesh::from_pydata(verts, &faces.map(|f| f.to_vec()))
}

/// `primitive_cone_add` (a cylinder with both radii the same), its ends each one polygon.
pub(crate) fn cone_mesh(segs: usize, r1: f32, r2: f32, depth: f32) -> Mesh {
    let half = 0.5 * depth;
    let mut verts = Vec::with_capacity(segs * 2);
    for i in 0..segs {
        let (s, c) = sin_cos_from_fraction(i as i32, segs as i32);
        verts.push(V3::new(r1 * s, r1 * c, -half));
        verts.push(V3::new(r2 * s, r2 * c, half));
    }
    let n = segs as u32;
    let mut faces: Vec<Vec<u32>> = Vec::with_capacity(segs + 2);
    for i in 1..n {
        faces.push(vec![2 * i - 2, 2 * i - 1, 2 * i + 1, 2 * i]);
    }
    faces.push(vec![2 * n - 2, 2 * n - 1, 1, 0]);
    // the ends: the top from its second point back round, the bottom from its first on
    faces.push((0..n).map(|k| 2 * ((n + 1 - k) % n) + 1).collect());
    faces.push((0..n).map(|k| 2 * k).collect());
    let mut mesh = Mesh::from_pydata(verts, &faces);
    // an end drawn to a point is one point: the first of its ring at the bottom, the last of it at the top
    if r1 == 0.0 || r2 == 0.0 {
        let target: Vec<i32> = (0..2 * n as i32)
            .map(|v| match (v % 2 == 0, r1 == 0.0, r2 == 0.0) {
                (true, true, _) => 0,
                (false, _, true) => 2 * n as i32 - 1,
                _ => -1,
            })
            .collect();
        mesh.weld(&target);
    }
    mesh
}

/// `primitive_uv_sphere_add`: a meridian turned round the axis a segment at a time, each from the last, and
/// the seam and the poles merged.
pub(crate) fn uv_sphere_mesh(seg: usize, tot: usize, rad: f32) -> Mesh {
    let ring = tot + 1;
    let mut verts = Vec::with_capacity((seg + 1) * ring);
    for a in 0..=tot {
        let (s, c) = sin_cos_from_fraction(a as i32, 2 * tot as i32);
        verts.push(V3::new(0.0, rad * s, rad * c));
    }
    let pivot = verts[tot];
    // a turn about z by one segment, about the last point of the meridian
    let angle = (-(PI * 2.0) / seg as f64) as f32;
    let (si, co) = (sinf(angle), cosf(angle));
    let ico = 1.0 - co;
    let cmat = M3([[co, si, 0.0], [-si, co, 0.0], [0.0, 0.0, ico + co]]);
    let mut tmat = M4::translation(pivot);
    let mut m = mul_m4_m4m4(&tmat, &cmat.to_4x4());
    tmat.0[3] = [-pivot.x, -pivot.y, -pivot.z, 1.0];
    m = mul_m4_m4m4(&m, &tmat);
    for k in 0..seg {
        for a in 0..ring {
            let v = verts[k * ring + a];
            verts.push(mul_m4_v3(&m, v));
        }
    }
    let mut faces = Vec::with_capacity(seg * tot);
    for k in 0..seg {
        for a in (0..tot).rev() {
            let (p, q) = ((k * ring + a) as u32, ((k + 1) * ring + a) as u32);
            faces.push(vec![p + 1, p, q, q + 1]);
        }
    }
    // the merge reaches a third of the shortest side
    let phid = PI as f32 / tot as f32;
    let len = 2.0 * rad * libm::sinf(phid / 2.0);
    let vec = V3::new(rad * libm::sinf(phid), 0.0, rad * libm::cosf(phid));
    let vec2 = crate::math::mul_m3_v3(&cmat, vec);
    let len2 = len_v3(vec2 - vec);
    let mut mesh = Mesh::from_pydata(verts, &faces);
    let target = weld::doubles(&mesh.verts, len.min(len2) / 3.0);
    mesh.weld(&target);
    mesh
}

const ICO_VERTS: [[f32; 3]; 12] = [
    [0.0, 0.0, -200.0],
    [144.72, -105.144, -89.443],
    [-55.277, -170.128, -89.443],
    [-178.885, 0.0, -89.443],
    [-55.277, 170.128, -89.443],
    [144.72, 105.144, -89.443],
    [55.277, -170.128, 89.443],
    [-144.72, -105.144, 89.443],
    [-144.72, 105.144, 89.443],
    [55.277, 170.128, 89.443],
    [178.885, 0.0, 89.443],
    [0.0, 0.0, 200.0],
];
const ICO_FACES: [[u32; 3]; 20] = [
    [0, 1, 2],
    [1, 0, 5],
    [0, 2, 3],
    [0, 3, 4],
    [0, 4, 5],
    [1, 5, 10],
    [2, 1, 6],
    [3, 2, 7],
    [4, 3, 8],
    [5, 4, 9],
    [1, 10, 6],
    [2, 6, 7],
    [3, 7, 8],
    [4, 8, 9],
    [5, 9, 10],
    [6, 10, 11],
    [7, 6, 11],
    [8, 7, 11],
    [9, 8, 11],
    [10, 9, 11],
];

/// `primitive_ico_sphere_add`: an icosahedron, each side cut into a grid of 2^(subdivisions - 1) a side and the
/// grid's points pushed out onto the sphere. (The points are Blender's to rounding, in an order of their own.)
#[allow(clippy::needless_range_loop)]
pub(crate) fn ico_sphere_mesh(subdivisions: usize, rad: f32) -> Mesh {
    let k = rad / 200.0;
    let base: Vec<V3> = ICO_VERTS
        .iter()
        .map(|v| V3::new(k * v[0], k * v[1], k * v[2]))
        .collect();
    let cuts = 1usize << subdivisions.saturating_sub(1);
    if cuts == 1 {
        return Mesh::from_pydata(base, &ICO_FACES.map(|f| f.to_vec()));
    }
    // (cut, every point is pushed out onto the sphere, the icosahedron's own too)
    let mut verts: Vec<V3> = base
        .iter()
        .map(|v| on_sphere_of(rad, [v.x as f64, v.y as f64, v.z as f64]))
        .collect();
    let mut on_edge: std::collections::BTreeMap<(u32, u32, usize), u32> =
        std::collections::BTreeMap::new();
    let mut faces = Vec::new();
    let on_sphere = |p: [f64; 3]| on_sphere_of(rad, p);
    for f in ICO_FACES {
        let c: Vec<[f64; 3]> = f
            .iter()
            .map(|&i| {
                let v = base[i as usize];
                [v.x as f64, v.y as f64, v.z as f64]
            })
            .collect();
        // the grid point i along the first side and j along the second, of `cuts`
        let mut grid = vec![vec![0u32; cuts + 1]; cuts + 1];
        for i in 0..=cuts {
            for j in 0..=cuts - i {
                let l = cuts - i - j;
                let corner = [(l, f[0]), (i, f[1]), (j, f[2])];
                let index = if let Some(&(_, v)) = corner.iter().find(|(w, _)| *w == cuts) {
                    v
                } else {
                    let p = [0, 1, 2].map(|d| {
                        (c[0][d] * l as f64 + c[1][d] * i as f64 + c[2][d] * j as f64) / cuts as f64
                    });
                    // a point on a side is shared with the face across it
                    let side = if j == 0 {
                        Some((f[0], f[1], i))
                    } else if l == 0 {
                        Some((f[1], f[2], j))
                    } else if i == 0 {
                        Some((f[2], f[0], l))
                    } else {
                        None
                    };
                    match side {
                        Some((a, b, t)) => {
                            let key = if a < b { (a, b, t) } else { (b, a, cuts - t) };
                            *on_edge.entry(key).or_insert_with(|| {
                                verts.push(on_sphere(p));
                                verts.len() as u32 - 1
                            })
                        }
                        None => {
                            verts.push(on_sphere(p));
                            verts.len() as u32 - 1
                        }
                    }
                };
                grid[i][j] = index;
            }
        }
        for i in 0..cuts {
            for j in 0..cuts - i {
                faces.push(vec![grid[i][j], grid[i + 1][j], grid[i][j + 1]]);
                if i + j + 1 < cuts {
                    faces.push(vec![grid[i + 1][j], grid[i + 1][j + 1], grid[i][j + 1]]);
                }
            }
        }
    }
    Mesh::from_pydata(verts, &faces)
}

fn on_sphere_of(rad: f32, p: [f64; 3]) -> V3 {
    let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    let r = rad as f64;
    v3(p[0] / len * r, p[1] / len * r, p[2] / len * r)
}
