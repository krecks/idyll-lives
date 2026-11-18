//! Primitives: every builder adds one mesh part centred on `at` (Blender axes, metres) with its material, and
//! returns it. Sizes are full extents (a cube of size 0.4 is 0.4 m across), matching the game's `box`/`cylinder`
//! parts. Sizes and scales go into the mesh itself, so a `bevel` is in metres on every side. `rot` is in degrees.
//!
//! Shading: a smooth part is smooth across its edges up to `SHARP` degrees and sharp past them (a cylinder's
//! rim, a box's corners), so a cap stays flat instead of shading into the side. Something round with few sides
//! (a 6-sided leg) stays smooth round its sides (`round_angle`). A `bevel` is shaded smooth with hardened
//! normals: the faces either side stay flat and the bevel rounds the edge between them, for the same triangles.
//!
//! The meshes are Blender's own primitives point for point (the same sums in single precision, the same point of
//! each merged pair kept), which the lumps, perches and far versions built on them depend on.

use crate::bevel::{EdgeOrder, bevel_edges};
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

/// gamekit's `shapes._recalc_normals`, which a script may call itself (and `bmesh.ops.recalc_face_normals` over
/// all a mesh's faces): each connected piece of `obj`'s mesh wound one way, looking outwards.
pub fn recalc_normals(obj: Obj) -> Obj {
    obj.edit(|o| o.mesh.recalc_normals());
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
    /// `torus`'s options.
    Torus { at: P3 = P3::ZERO, material: Option<Mat> = None, rot: P3 = P3::ZERO, major_segments: usize = 24, minor_segments: usize = 12, smooth: bool = true }
}
options! {
    /// `boulder`'s options.
    Boulder { at: P3 = P3::ZERO, material: Option<Mat> = None, scale: P3 = P3::ONE, rot: P3 = P3::ZERO, seed: f64 = 0.0, noise: f64 = 0.22, subdivisions: usize = 3, smooth: bool = true }
}

/// A box of `size` (x, y, z) or one number for all three.
pub fn cube(name: impl AsRef<str>, size: impl Into<Size>, o: Cube) -> Obj {
    scene::update(); // (an operator makes it: Blender works the scene out)
    let obj = scene::add_primitive("Cube", name.as_ref(), cube_mesh());
    scale_mesh(obj, size.into().0);
    let bevel = (o.bevel, o.bevel_segments, EdgeOrder::Faces);
    finish(obj, o.at, o.rot, o.material, o.smooth, bevel, SHARP)
}

pub fn sphere(name: impl AsRef<str>, radius: f64, o: Sphere) -> Obj {
    scene::update(); // (an operator makes it: Blender works the scene out)
    let obj = scene::add_primitive(
        "Sphere",
        name.as_ref(),
        // (Blender's operator takes no fewer than three segments or rings: two rings come out as three)
        uv_sphere_mesh(o.segments.max(3), o.rings.max(3), radius as f32),
    );
    scale_mesh(obj, o.scale);
    let bevel = (o.bevel, o.bevel_segments, EdgeOrder::Faces);
    finish(obj, o.at, o.rot, o.material, o.smooth, bevel, CLOSED)
}

/// A cylinder along z, `depth` long.
pub fn cylinder(name: impl AsRef<str>, radius: f64, depth: f64, o: Round) -> Obj {
    round("Cylinder", name.as_ref(), radius, radius, depth, o)
}

/// A cone or frustum along z, radius `r1` at the bottom and `r2` at the top.
pub fn cone(name: impl AsRef<str>, r1: f64, r2: f64, depth: f64, o: Round) -> Obj {
    round("Cone", name.as_ref(), r1, r2, depth, o)
}

/// A cylinder or a cone: the one mesh, named for the operator that made it.
fn round(kind: &str, name: &str, r1: f64, r2: f64, depth: f64, o: Round) -> Obj {
    scene::update(); // (an operator makes it: Blender works the scene out)
    let obj = scene::add_primitive(
        kind,
        name,
        cone_mesh(o.verts, r1 as f32, r2 as f32, depth as f32),
    );
    scale_mesh(obj, o.scale);
    obj.edit(|p| p.uv = UvWay::Round);
    // (Blender numbers a cylinder's rims before its sides, the bottom's and the top's in turn: a rim's bevel
    // starts from the rim)
    let n = o.verts as u32;
    let rims = if obj.read(|p| p.mesh.verts.len()) == 2 * o.verts {
        // (the bottom's run on round, the top's back)
        let rims = (0..n).flat_map(|i| {
            let j = (i + 1) % n;
            [(2 * i, 2 * j), (2 * j + 1, 2 * i + 1)]
        });
        EdgeOrder::Listed(rims.collect())
    } else {
        EdgeOrder::Faces
    };
    finish(
        obj,
        o.at,
        o.rot,
        o.material,
        o.smooth,
        (o.bevel, o.bevel_segments, rims),
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
    scene::update(); // (its normals are set right by operators: Blender works the scene out)
    let obj = scene::add(name.as_ref(), mesh);
    shade(obj, o.smooth, o.angle.max(round_angle(segments)));
    obj.edit(|p| p.uv = UvWay::Round);
    obj.set_location(o.at)
        .set_rotation(o.rot)
        .set_scale(o.scale)
}

/// A ring lying in the xy plane.
pub fn torus(name: impl AsRef<str>, major_radius: f64, minor_radius: f64, o: Torus) -> Obj {
    scene::update(); // (an operator makes it: Blender works the scene out)
    let obj = scene::add_primitive(
        "Torus",
        name.as_ref(),
        torus_mesh(
            major_radius as f32,
            minor_radius as f32,
            o.major_segments,
            o.minor_segments,
        ),
    );
    let none = (0.0, 2, EdgeOrder::Faces);
    finish(obj, o.at, o.rot, o.material, o.smooth, none, CLOSED)
}

/// `shapes::torus` with its options by name, as the family scripts wrote them: `torus!(name, major, minor, key = value, ...)`.
#[macro_export]
macro_rules! torus {
    ($name:expr, $major:expr, $minor:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $major, $minor);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Torus::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::shapes::torus(args.0, args.1, args.2, o)
    }};
}

/// A lumpy, deterministic boulder: an icosphere displaced by noise, bottom on z = 0.
pub fn boulder(name: impl AsRef<str>, radius: f64, o: Boulder) -> Obj {
    scene::update(); // (an operator makes it: Blender works the scene out)
    let obj = scene::add_primitive(
        "Icosphere",
        name.as_ref(),
        ico_sphere_mesh(o.subdivisions, radius as f32),
    );
    let none = (0.0, 2, EdgeOrder::Faces);
    finish(obj, o.at, o.rot, o.material, o.smooth, none, CLOSED);
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
    (bevel, bevel_segments, order): (f64, usize, EdgeOrder),
    angle: f64,
) -> Obj {
    obj.edit(|o| o.mesh.set_material(material));
    if smooth {
        shade(obj, true, angle);
    }
    if bevel != 0.0 {
        bevel_edges(obj, bevel, bevel_segments, SHARP, &order);
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
    /// `primitive_torus_add(major_radius, minor_radius, major_segments, minor_segments)`.
    pub fn torus(major: f32, minor: f32, major_segments: usize, minor_segments: usize) -> Mesh {
        super::torus_mesh(major, minor, major_segments, minor_segments)
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
    // (each point through the operator's matrix, which moves nothing: a zero comes out of it without its sign)
    let placed = |x: f32, y: f32, z: f32| mul_m4_v3(&M4::IDENTITY, V3::new(x, y, z));
    for i in 0..segs {
        let (s, c) = sin_cos_from_fraction(i as i32, segs as i32);
        verts.push(placed(r1 * s, r1 * c, -half));
        verts.push(placed(r2 * s, r2 * c, half));
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

/// `primitive_torus_add` (Blender's add_mesh_torus.py): rings of `minor_seg` points round the cross section, each
/// ring turned about z by its share of the turn, the quads from each ring to the next; flat-shaded.
pub(crate) fn torus_mesh(
    major_rad: f32,
    minor_rad: f32,
    major_seg: usize,
    minor_seg: usize,
) -> Mesh {
    let (major_rad, minor_rad) = (major_rad as f64, minor_rad as f64);
    let pi_2 = PI * 2.0;
    let mut verts = Vec::with_capacity(major_seg * minor_seg);
    let mut faces: Vec<Vec<u32>> = Vec::with_capacity(major_seg * minor_seg);
    let mut i1 = 0;
    let tot_verts = major_seg * minor_seg;
    for major_index in 0..major_seg {
        let matrix = M3::rotation(
            (major_index as f64 / major_seg as f64) * pi_2,
            math::Axis::Z,
        );
        for minor_index in 0..minor_seg {
            let angle = pi_2 * minor_index as f64 / minor_seg as f64;
            verts.push(
                matrix
                    * v3(
                        major_rad + (math::cos(angle) * minor_rad),
                        0.0,
                        math::sin(angle) * minor_rad,
                    ),
            );
            let (mut i2, mut i3, mut i4) = if minor_index + 1 == minor_seg {
                let i2 = major_index * minor_seg;
                (i2, i1 + minor_seg, i2 + minor_seg)
            } else {
                (i1 + 1, i1 + minor_seg, i1 + minor_seg + 1)
            };
            for i in [&mut i2, &mut i3, &mut i4] {
                if *i >= tot_verts {
                    *i -= tot_verts;
                }
            }
            faces.push(vec![i1 as u32, i3 as u32, i4 as u32, i2 as u32]);
            i1 += 1;
        }
    }
    Mesh::from_pydata(verts, &faces)
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

/// `primitive_ico_sphere_add`: an icosahedron whose edges are cut and whose sides are filled with a grid, as Blender's
/// `subdivide_edges` (grid fill, onto a sphere) does: the edges in the order the faces made them, each cut from its first
/// point to its second, each cut a step of the way left on the edge; then the sides' inner points row by row
/// between the cut points (pushed out onto the sphere already), the last side first. Every point is then pushed out
/// onto the sphere.
pub(crate) fn ico_sphere_mesh(subdivisions: usize, rad: f32) -> Mesh {
    let rad_div = rad / 200.0;
    let base: Vec<V3> = ICO_VERTS
        .iter()
        .map(|v| V3::new(rad_div * v[0], rad_div * v[1], rad_div * v[2]))
        .collect();
    // (the operator's matrix, the identity, turns a negative zero positive)
    let placed = |verts: Vec<V3>| -> Vec<V3> {
        verts
            .into_iter()
            .map(|v| mul_m4_v3(&M4::IDENTITY, v))
            .collect()
    };
    if subdivisions <= 1 {
        return Mesh::from_pydata(placed(base), &ICO_FACES.map(|f| f.to_vec()));
    }
    let cuts = (1usize << (subdivisions - 1)) - 1;
    let onto = |p: V3| {
        let d = crate::math::dot_v3(p, p);
        if d > 1.0e-35 {
            p * (rad / d.sqrt())
        } else {
            V3::ZERO
        }
    };
    // a cut of the way from `a` to `b` (`BM_edge_split`)
    let split = |a: V3, b: V3, fac: f32| {
        let d = b - a;
        V3::new(a.x + d.x * fac, a.y + d.y * fac, a.z + d.z * fac)
    };
    let mut verts: Vec<V3> = base.iter().map(|&v| onto(v)).collect();
    // the edges, (first point, second point), in the order the faces made them, and the points cut on each
    let mut edges: Vec<(u32, u32)> = Vec::new();
    for f in ICO_FACES {
        // (each face makes its closing edge first: last corner to first, then round)
        for k in 0..3 {
            let (a, b) = (f[(k + 2) % 3], f[k]);
            if !edges.contains(&(a, b)) && !edges.contains(&(b, a)) {
                edges.push((a, b));
            }
        }
    }
    let mut cut: Vec<Vec<u32>> = Vec::with_capacity(edges.len());
    for &(a, b) in &edges {
        let (mut at, end) = (base[a as usize], base[b as usize]);
        let mut points = Vec::with_capacity(cuts);
        for i in 0..cuts {
            at = split(at, end, 1.0 / (cuts + 1 - i) as f32);
            verts.push(onto(at));
            points.push(verts.len() as u32 - 1);
        }
        cut.push(points);
    }
    // the points cut on the side from `a` to `b`, from `a` on
    let side = |a: u32, b: u32| -> Vec<u32> {
        match edges.iter().position(|&e| e == (a, b)) {
            Some(e) => cut[e].clone(),
            None => {
                let e = edges.iter().position(|&e| e == (b, a)).unwrap_or(0);
                cut[e].iter().rev().copied().collect()
            }
        }
    };
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for f in ICO_FACES.iter().rev() {
        let (s0, s1, s2) = (side(f[0], f[1]), side(f[1], f[2]), side(f[2], f[0]));
        // rows from the third corner down to the side from the first to the second, each from the side the first
        // corner is on to the other
        let mut lines: Vec<Vec<u32>> = vec![vec![f[2]]];
        for i in 0..cuts {
            let (a, b) = (s2[i], s1[cuts - 1 - i]);
            let mut row = vec![a];
            let (mut at, end) = (verts[a as usize], verts[b as usize]);
            for j in 0..i {
                at = split(at, end, 1.0 / (i + 1 - j) as f32);
                verts.push(onto(at));
                row.push(verts.len() as u32 - 1);
            }
            row.push(b);
            lines.push(row);
        }
        let mut bottom = vec![f[0]];
        bottom.extend(&s0);
        bottom.push(f[1]);
        lines.push(bottom);
        for r in 0..lines.len() - 1 {
            let (up, down) = (&lines[r], &lines[r + 1]);
            for j in 0..up.len() {
                faces.push(vec![up[j], down[j], down[j + 1]]);
                if j + 1 < up.len() {
                    faces.push(vec![up[j], down[j + 1], up[j + 1]]);
                }
            }
        }
    }
    Mesh::from_pydata(placed(verts), &faces)
}
