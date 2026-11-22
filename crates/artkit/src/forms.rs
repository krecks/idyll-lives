//! Forms: the shapes the primitives can't make, each one mesh part (Blender axes, metres), deterministic, with its
//! material, as the shapes' builders are. A profile swept along a path (`sweep`: rails, piping, bent tube, a
//! cabriole leg, a branch), rings lofted into a body (`loft`, `ring`: a bottle, a lamp base, a rounded cabinet),
//! an outline with holes extruded into a slab (`extrude`: a shaped apron, a headboard, a door with a window),
//! outlines (`rounded_rect`, `smooth_outline`, `circle`), and what is done to a part after (`bisect`, `mirrored`,
//! `jitter`).
//!
//! Smooth forms are shaded smooth with their edges over `angle` degrees kept sharp, so a cap or a square
//! profile stays crisp without a bevel.

use crate::bevel::{EdgeOrder, bevel_edges};
use crate::materials::Mat;
use crate::math::{self, Axis, M4, Num, P3, TAU, V3, v3};
use crate::mesh::{Mesh, UvWay};
use crate::noise::noise;
use crate::scene::{self, Obj};
use crate::shapes::{SHARP, options, round_angle, shade, sides};
use crate::uv;
use std::rc::Rc;

// ---- profiles (2D) ------------------------------------------------------------------------------------

/// A round profile of radius `r`, `verts` points.
pub fn circle(r: f64, verts: usize) -> Vec<(f64, f64)> {
    (0..verts)
        .map(|i| {
            (
                r * math::cos(TAU * i as f64 / verts as f64),
                r * math::sin(TAU * i as f64 / verts as f64),
            )
        })
        .collect()
}

/// A w × h rectangle with corners of radius `r` (`corner` segments each), centred, anticlockwise.
pub fn rounded_rect(w: f64, h: f64, r: f64, corner: usize) -> Vec<(f64, f64)> {
    let min = |a: f64, b: f64| if b < a { b } else { a };
    let r = min(min(r, w / 2.0), h / 2.0);
    let mut out = Vec::with_capacity(4 * (corner + 1));
    for (cx, cy, a0) in [
        (w / 2.0 - r, h / 2.0 - r, 0.0),
        (-w / 2.0 + r, h / 2.0 - r, 90.0),
        (-w / 2.0 + r, -h / 2.0 + r, 180.0),
        (w / 2.0 - r, -h / 2.0 + r, 270.0),
    ] {
        for i in 0..=corner {
            let a = math::radians(a0 + 90.0 * i as f64 / corner as f64);
            out.push((cx + r * math::cos(a), cy + r * math::sin(a)));
        }
    }
    dedupe(&out, 1e-7)
}

/// Chaikin's corner cutting: each pass replaces every corner by two points a quarter in, rounding it.
pub fn smooth_outline(points: &[(f64, f64)], passes: usize, closed: bool) -> Vec<(f64, f64)> {
    let mut pts = points.to_vec();
    // (each point as a 2D Vector: single precision)
    let f = |p: (f64, f64)| (p.0 as f32, p.1 as f32);
    for _ in 0..passes {
        let mut out = if closed { Vec::new() } else { vec![pts[0]] };
        let n = pts.len();
        for i in 0..if closed { n } else { n - 1 } {
            let (a, b) = (f(pts[i]), f(pts[(i + 1) % n]));
            out.push((
                (a.0 * 0.75 + b.0 * 0.25) as f64,
                (a.1 * 0.75 + b.1 * 0.25) as f64,
            ));
            out.push((
                (a.0 * 0.25 + b.0 * 0.75) as f64,
                (a.1 * 0.25 + b.1 * 0.75) as f64,
            ));
        }
        if !closed {
            out.push(pts[n - 1]);
        }
        pts = out;
    }
    pts
}

/// Whether 2D point `p` is inside the closed `outline` (even-odd).
fn inside(p: (f64, f64), outline: &[(f64, f64)]) -> bool {
    let (x, y) = p;
    let mut inside = false;
    for i in 0..outline.len() {
        let ((x0, y0), (x1, y1)) = (outline[i], outline[(i + 1) % outline.len()]);
        if (y0 > y) != (y1 > y) && x < x0 + (y - y0) * (x1 - x0) / (y1 - y0) {
            inside = !inside;
        }
    }
    inside
}

/// An outline without a point on the one before it (within `eps`), nor a last point on the first.
pub fn dedupe(pts: &[(f64, f64)], eps: f64) -> Vec<(f64, f64)> {
    // (the distance as a 2D Vector's: the differences single, their squares summed in double from the last)
    let apart = |a: (f64, f64), b: (f64, f64)| {
        let (dx, dy) = (a.0 as f32 - b.0 as f32, a.1 as f32 - b.1 as f32);
        ((dy * dy) as f64 + (dx * dx) as f64).sqrt()
    };
    let mut out: Vec<(f64, f64)> = Vec::with_capacity(pts.len());
    for &p in pts {
        if out.last().is_none_or(|&l| apart(p, l) > eps) {
            out.push(p);
        }
    }
    if out.len() > 2 && apart(out[0], out[out.len() - 1]) <= eps {
        out.pop();
    }
    out
}

// ---- paths --------------------------------------------------------------------------------------------

/// The points, with `resolution` - 1 more on a Catmull-Rom curve between each pair (1: as they are). Returns
/// (points, the fraction of the way along the given points at each, 0..1).
/// gamekit's `forms._catmull`, which a script may call itself (a chair's shell through its key rows): the points,
/// with `resolution` - 1 more on a Catmull-Rom curve between each pair (1: as they are), and the fraction of the
/// way along the given points at each, 0..1.
pub fn catmull(pts: &[V3], resolution: usize, closed: bool) -> (Vec<V3>, Vec<f64>) {
    let n = pts.len();
    let spans = if closed { n } else { n - 1 };
    if resolution <= 1 || n < 3 {
        return (
            pts.to_vec(),
            (0..n).map(|i| i as f64 / spans as f64).collect(),
        );
    }
    let at = |i: i64| {
        if closed {
            pts[i.rem_euclid(n as i64) as usize]
        } else {
            pts[i.clamp(0, n as i64 - 1) as usize]
        }
    };
    let (mut out, mut wher) = (Vec::new(), Vec::new());
    for i in 0..spans {
        let j = i as i64;
        let (mut p0, p1, p2, mut p3) = (at(j - 1), at(j), at(j + 1), at(j + 2));
        if !closed && i == 0 {
            p0 = p1 * 2 - p2;
        }
        if !closed && i == spans - 1 {
            p3 = p2 * 2 - p1;
        }
        for k in 0..resolution {
            let t = k as f64 / resolution as f64;
            let (t2, t3) = (t * t, t * t * t);
            out.push(
                0.5 * ((2.0 * p1)
                    + (-p0 + p2) * t
                    + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
                    + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3),
            );
            wher.push((i as f64 + t) / spans as f64);
        }
    }
    if !closed {
        out.push(pts[n - 1]);
        wher.push(1.0);
    }
    (out, wher)
}

/// A (tangent, normal, binormal) frame at each point, carried along without twisting (parallel transport), the
/// first normal as near `up` as the tangent allows.
fn frames(pts: &[V3], closed: bool, up: V3) -> Vec<(V3, V3, V3)> {
    let n = pts.len();
    let tangents: Vec<V3> = (0..n)
        .map(|i| {
            let a = if closed || i > 0 {
                pts[(i + n - 1) % n]
            } else {
                pts[i]
            };
            let b = if closed || i < n - 1 {
                pts[(i + 1) % n]
            } else {
                pts[i]
            };
            let t = b - a;
            if t.length() > 1e-9 {
                t.normalized()
            } else {
                V3::Z
            }
        })
        .collect();
    let mut normal = up - tangents[0] * up.dot(tangents[0]);
    if normal.length() < 1e-6 {
        normal = tangents[0].orthogonal();
    }
    normal = normal.normalized();
    let mut out = Vec::with_capacity(n);
    for (i, &t) in tangents.iter().enumerate() {
        if i > 0 {
            normal = tangents[i - 1].rotation_difference(t) * normal;
            normal = (normal - t * normal.dot(t)).normalized();
        }
        out.push((t, normal, t.cross(normal)));
    }
    out
}

/// How a sweep thickens along the way: a number, a list (one per point, linear between) or a function of how far
/// along (0..1), so a branch or a cabriole leg tapers.
#[derive(Clone)]
pub enum Scale {
    One(f64),
    List(Vec<f64>),
    By(Rc<dyn Fn(f64) -> f64>),
}

/// A scale that is a function of how far along the path, 0..1: `scale = by(move |w| 1.0 + (k - 1.0) * w)`.
pub fn by(f: impl Fn(f64) -> f64 + 'static) -> Scale {
    Scale::By(Rc::new(f))
}

impl std::fmt::Debug for Scale {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Scale::One(k) => write!(f, "{k}"),
            Scale::List(l) => write!(f, "{l:?}"),
            Scale::By(_) => write!(f, "by(..)"),
        }
    }
}
/// A closed 2D profile [(u, v), ...] about a sweep's path, or none (a round tube).
#[derive(Clone, Debug, Default)]
pub struct Profile(pub Option<Vec<(f64, f64)>>);

options! {
    /// `sweep`'s options.
    Sweep { profile: Profile = Profile(None), material: Option<Mat> = None, verts: Option<usize> = None, scale: Scale = Scale::One(1.0), resolution: usize = 1, closed: bool = false, caps: bool = true, up: P3 = P3 { x: 0.0, y: 0.0, z: 1.0 }, smooth: bool = true, angle: Option<f64> = None, least: usize = 12 }
}

/// A profile swept along a path through `points` (Blender, metres): a tube of `radius` (`verts` round: by
/// default `least`, 12, and more for one over 3 cm in radius at its thickest, as many as keep its outline within
/// a millimetre of the circle, `shapes::sides`), or any closed `profile` [(u, v), ...] about the path, u along the
/// frame's normal (as near `up` as it goes), v across. `resolution` > 1 curves the path through the points
/// (Catmull-Rom), that many steps between each pair. `scale` thickens it along the way (`Scale`). `closed`
/// joins the path's ends (a ring, piping round a cushion); otherwise the ends are capped flat (`caps`). Smooth,
/// edges turning more than `angle` degrees stay sharp: by default `SHARP` for a `profile`, and for a round tube
/// enough that its sides stay round (`round_angle`).
pub fn sweep<P: Into<V3> + Copy>(
    name: impl AsRef<str>,
    points: &[P],
    radius: f64,
    o: Sweep,
) -> Obj {
    let given: Vec<V3> = points.iter().map(|&p| p.into()).collect();
    let (pts, wher) = catmull(&given, o.resolution, o.closed);
    let scales: Vec<f64> = match &o.scale {
        Scale::By(f) => wher.iter().map(|&w| f(w)).collect(),
        Scale::List(list) => {
            let last = list.len() - 1;
            wher.iter()
                .map(|&w| {
                    if last == 0 {
                        return list[0];
                    }
                    let f = w * last as f64;
                    let i = (f as usize).min(last - 1);
                    list[i] + (list[(i + 1).min(last)] - list[i]) * (f - i as f64)
                })
                .collect()
        }
        Scale::One(k) => vec![*k; pts.len()],
    };
    let mut verts = o.verts;
    if verts.is_none() && o.profile.0.is_none() {
        let thickest = scales
            .iter()
            .map(|k| k.abs())
            .fold(f64::NEG_INFINITY, f64::max);
        verts = Some(sides(radius * thickest, o.least, 64));
    }
    let prof = match &o.profile.0 {
        Some(p) if !p.is_empty() => p.clone(),
        _ => circle(radius, verts.unwrap_or(o.least)),
    };
    let fr = frames(&pts, o.closed, o.up.into());
    let m = prof.len();
    let mut vs = Vec::with_capacity(pts.len() * m);
    for ((p, (_, nrm, bi)), s) in pts.iter().zip(&fr).zip(&scales) {
        for &(u, v) in &prof {
            vs.push(*p + (*nrm * u + *bi * v) * *s);
        }
    }
    let rings = pts.len();
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for i in 0..if o.closed { rings } else { rings - 1 } {
        let (a, b) = (i * m, ((i + 1) % rings) * m);
        for k in 0..m {
            let k2 = (k + 1) % m;
            faces.push(vec![
                (a + k) as u32,
                (a + k2) as u32,
                (b + k2) as u32,
                (b + k) as u32,
            ]);
        }
    }
    if o.caps && !o.closed {
        faces.push((0..m as u32).rev().collect());
        faces.push((((rings - 1) * m) as u32..(rings * m) as u32).collect());
    }
    let angle = o.angle.unwrap_or(if o.profile.0.is_some() {
        SHARP
    } else {
        round_angle(verts.unwrap_or(o.least))
    });
    let obj = object(
        name.as_ref(),
        vs,
        &faces,
        o.material,
        o.smooth,
        angle,
        false,
    );
    uv::along(obj, rings, &prof, &scales, &pts, o.closed)
}

// ---- lofts --------------------------------------------------------------------------------------------

options! {
    /// `ring`'s options.
    Ring { z: f64 = 0.0, verts: usize = 24, square: f64 = 2.0, at: (f64, f64) = (0.0, 0.0) }
}

/// A loop of `verts` points round (at.0, at.1) at height `z`: an ellipse of radii rx, ry (`square` 2), squarer
/// towards a rounded rectangle as `square` grows (4: a soft square, 10: nearly sharp corners).
pub fn ring(rx: f64, ry: f64, o: Ring) -> Vec<P3> {
    let e = 2.0 / o.square;
    (0..o.verts)
        .map(|i| {
            let a = TAU * i as f64 / o.verts as f64;
            let (c, s) = (math::cos(a), math::sin(a));
            P3 {
                x: o.at.0 + rx * math::pow(c.abs(), e).copysign(c),
                y: o.at.1 + ry * math::pow(s.abs(), e).copysign(s),
                z: o.z,
            }
        })
        .collect()
}

options! {
    /// `loft`'s options.
    Loft { material: Option<Mat> = None, caps: bool = true, smooth: bool = true, angle: f64 = SHARP, at: P3 = P3::ZERO, rot: P3 = P3::ZERO }
}

/// A body through `sections`, each a closed loop of Blender points with the same count (`ring`), bottom first:
/// a bottle, a vase, a lamp's base, a rounded fridge. The first and last loops are capped flat (`caps`); a loop
/// shrunk to a point closes the body there instead.
pub fn loft(name: impl AsRef<str>, sections: &[Vec<P3>], o: Loft) -> Obj {
    let name = name.as_ref();
    let m = sections[0].len();
    assert!(
        sections.iter().all(|s| s.len() == m),
        "{name}: every section needs the same number of points ({m})"
    );
    let vs: Vec<V3> = sections.iter().flatten().map(|&p| p.into()).collect();
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for i in 0..sections.len() - 1 {
        let (a, b) = (i * m, (i + 1) * m);
        for k in 0..m {
            let k2 = (k + 1) % m;
            faces.push(vec![
                (a + k) as u32,
                (a + k2) as u32,
                (b + k2) as u32,
                (b + k) as u32,
            ]);
        }
    }
    if o.caps {
        faces.push((0..m as u32).rev().collect());
        faces.push((((sections.len() - 1) * m) as u32..(sections.len() * m) as u32).collect());
    }
    let obj = object(name, vs, &faces, o.material, o.smooth, o.angle, true);
    let level = |sec: &Vec<P3>| {
        let hi = sec.iter().map(|p| p.z).fold(f64::NEG_INFINITY, f64::max);
        let lo = sec.iter().map(|p| p.z).fold(f64::INFINITY, f64::min);
        hi - lo < 1e-6
    };
    if sections.iter().all(level) {
        // level rings up its z (a drum, a vase): textured round it
        obj.edit(|p| p.uv = UvWay::Round);
    }
    obj.set_location(o.at).set_rotation(o.rot)
}

// ---- slabs --------------------------------------------------------------------------------------------

/// Where an outline is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Plane {
    /// Facing the front: u across, v up, thick front to back (an apron, a headboard, a chair's back).
    #[default]
    XZ,
    /// Lying flat: thick upwards (a shaped top, a tray).
    XY,
    /// Facing the side: u back to front, v up (a chair's side, a rocker).
    YZ,
}

impl Plane {
    /// (Blender axis of its u, of its v, of the thickness).
    fn axes(self) -> (usize, usize, usize) {
        match self {
            Plane::XZ => (0, 2, 1),
            Plane::XY => (0, 1, 2),
            Plane::YZ => (1, 2, 0),
        }
    }
}

/// The outlines inside a slab's outline that are cut out of it.
#[derive(Clone, Debug, Default)]
pub struct Holes(pub Vec<Vec<(f64, f64)>>);

impl crate::Arg<Holes> for Vec<Vec<(f64, f64)>> {
    fn arg(self) -> Holes {
        Holes(self)
    }
}
impl crate::Arg<Holes> for &[Vec<(f64, f64)>] {
    fn arg(self) -> Holes {
        Holes(self.to_vec())
    }
}
impl<const N: usize> crate::Arg<Holes> for [Vec<(f64, f64)>; N] {
    fn arg(self) -> Holes {
        Holes(self.to_vec())
    }
}

options! {
    /// `extrude`'s options.
    Extrude { at: P3 = P3::ZERO, material: Option<Mat> = None, holes: Holes = Holes(Vec::new()), plane: Plane = Plane::XZ, rot: P3 = P3::ZERO, bevel: f64 = 0.0, bevel_segments: usize = 2, smooth: bool = false, angle: f64 = SHARP }
}

/// Whether one of `tris` (corners among the outlines' points, numbered on through them) has no area: its three
/// points in one line.
fn has_flat_face(loops: &[Vec<[f32; 2]>], tris: &[[u32; 3]]) -> bool {
    let xy: Vec<[f64; 2]> = loops
        .iter()
        .flatten()
        .map(|p| [p[0] as f64, p[1] as f64])
        .collect();
    tris.iter().any(|t| {
        let [a, b, c] = t.map(|v| xy[v as usize]);
        (b[0] - a[0]) * (c[1] - a[1]) == (b[1] - a[1]) * (c[0] - a[0])
    })
}

/// A slab `depth` thick from a closed 2D `outline` [(u, v), ...] and any `holes` (outlines inside it): the
/// outline's (0, 0) goes on `at`, and the slab is centred on it through its thickness. `plane` is where the
/// outline is drawn (`Plane`). `bevel` rounds its edges, as the primitives' do.
///
/// Its two faces are cut into triangles as Blender cut them (`tri::fill`): a bevel follows the cuts.
///
/// But where Blender's fill leaves faces of no area (among points in one line down a long outline: the feet of a
/// roller shutter's slats) and fills the outline's mirror image without any, the slab is that one's, mirrored:
/// Blender's extrusion walls the slab off inside under such faces, which shades it in blotches, and the two sides
/// of a model, drawn as each other's mirror image, are then built as each other's.
pub fn extrude(name: impl AsRef<str>, outline: &[(f64, f64)], depth: f64, o: Extrude) -> Obj {
    let name = name.as_ref();
    let loops: Vec<Vec<(f64, f64)>> = std::iter::once(outline)
        .chain(o.holes.0.iter().map(Vec::as_slice))
        .map(|l| dedupe(l, 1e-7))
        .collect();
    let mut flat: Vec<Vec<[f32; 2]>> = loops
        .iter()
        .map(|l| l.iter().map(|p| [p.0 as f32, p.1 as f32]).collect())
        .collect();
    let mut filled = crate::tri::fill(&flat);
    // (built as its mirror image's, u the other way, and turned back once its mesh is made)
    let mut mirrored = false;
    if has_flat_face(&flat, &filled) {
        let other: Vec<Vec<[f32; 2]>> = flat
            .iter()
            .map(|l| l.iter().map(|p| [-p[0], p[1]]).collect())
            .collect();
        let its = crate::tri::fill(&other);
        if !has_flat_face(&other, &its) {
            (flat, filled, mirrored) = (other, its, true);
        }
    }
    let xy: Vec<[f32; 2]> = flat.iter().flatten().copied().collect();
    let n = xy.len() as u32;
    // the fill covers the holes too: take out what lies inside one
    let tris: Vec<[u32; 3]> = filled
        .into_iter()
        .filter(|t| {
            let (a, b, c) = (xy[t[0] as usize], xy[t[1] as usize], xy[t[2] as usize]);
            let third = 1.0_f32 / 3.0;
            let u = ((a[0] + b[0] + c[0]) * third) as f64;
            let centre = (
                if mirrored { -u } else { u },
                ((a[1] + b[1] + c[1]) * third) as f64,
            );
            !o.holes.0.iter().any(|h| inside(centre, h))
        })
        .collect();
    // one face at -depth / 2, its copy at +depth / 2, and a wall along every side of the outlines
    let (iu, iv, iw) = o.plane.axes();
    let half = -(depth / 2.0) as f32;
    let (lo, hi) = (0.0_f32 + half, depth as f32 + half);
    // (Blender's extrusion copies the face triangle by triangle: the copy's points are numbered as the triangles
    // first name them, and its sides as they first run them)
    let mut far = vec![u32::MAX; xy.len()];
    let mut named = n;
    for &v in tris.iter().flatten() {
        if far[v as usize] == u32::MAX {
            far[v as usize] = named;
            named += 1;
        }
    }
    for f in &mut far {
        if *f == u32::MAX {
            (*f, named) = (named, named + 1);
        }
    }
    let mut verts = vec![V3::ZERO; 2 * xy.len()];
    for (i, p) in xy.iter().enumerate() {
        for (at, w) in [(i, lo), (far[i] as usize, hi)] {
            let mut co = [0.0_f32; 3];
            (co[iu], co[iv], co[iw]) = (p[0], p[1], w);
            verts[at] = V3::new(co[0], co[1], co[2]);
        }
    }
    let far = |v: u32| far[v as usize];
    let mut faces: Vec<Vec<u32>> = tris.iter().map(|t| t.to_vec()).collect();
    faces.extend(tris.iter().map(|t| t.iter().map(|&v| far(v)).collect()));
    let mut rims = Vec::new();
    let mut at = 0u32;
    for l in &loops {
        let m = l.len() as u32;
        for i in 0..m {
            let (a, b) = (at + i, at + (i + 1) % m);
            faces.push(vec![a, b, far(b), far(a)]);
            rims.push((a, b));
        }
        at += m;
    }
    // (Blender numbers the outlines' sides first, a face's before its copy's: a rim's bevel starts from them)
    let mut order = rims.clone();
    for t in &tris {
        for i in 0..3 {
            let (a, b) = (t[i], t[(i + 1) % 3]);
            for side in [(a, b), (b, a)] {
                let copy = (far(side.0), far(side.1));
                if rims.contains(&side) && !order.contains(&copy) {
                    order.push(copy);
                }
            }
        }
    }
    for &(a, b) in &rims {
        if !order.contains(&(far(a), far(b))) {
            order.push((far(a), far(b)));
        }
    }
    let sides_from = faces.len() - rims.len();
    let mut mesh = Mesh::from_pydata(verts, &faces);
    mesh.set_material(o.material);
    mesh.recalc_normals();
    // (Blender's extrusion makes each wall from the end of its side where the face's triangle along it starts,
    // looking inwards as the outline was drawn; drawn facing the front, it looks out as it is and stays, and
    // lying flat or facing the side it is turned round. So of the walls made here from the outline's own
    // direction, one that looks out facing the front starts a corner on, and one turned round lying flat or
    // facing the side a corner back; the others start where Blender's do)
    for (f, made) in faces.iter().enumerate().skip(sides_from) {
        let face = mesh.faces[f];
        let corners = &mut mesh.loops[face.start as usize..(face.start + face.len) as usize];
        match (o.plane == Plane::XZ, corners == made.as_slice()) {
            (true, true) => corners.rotate_left(1),
            (false, false) => corners.rotate_right(1),
            _ => {}
        }
    }
    if mirrored {
        for v in &mut mesh.verts {
            v[iu] = -v[iu];
        }
        mesh.flip_normals();
    }
    let obj = scene::add(name, mesh);
    shade(obj, o.smooth, o.angle);
    if o.bevel != 0.0 {
        bevel_edges(
            obj,
            o.bevel,
            o.bevel_segments,
            30.0,
            &EdgeOrder::Listed(order),
        );
        obj.edit(|part| {
            if let Some(bevel) = &mut part.bevel {
                bevel.sides_from = Some(sides_from);
            }
        });
    }
    obj.set_location(o.at).set_rotation(o.rot)
}

/// `forms::extrude` with its options by name, as the family scripts wrote them: `extrude!(name, outline, depth, key = value, ...)`.
#[macro_export]
macro_rules! extrude {
    ($name:expr, $outline:expr, $depth:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $outline, $depth);
        #[allow(unused_mut)]
        let mut o = $crate::forms::Extrude::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::forms::extrude(args.0, args.1, args.2, o)
    }};
}

// ---- after making -------------------------------------------------------------------------------------

/// Cuts `obj` (in place) by the plane through `co` facing `no` (Blender world), dropping what lies in front
/// of it; `cap` closes the cut with a flat face of its own: half of a lofted disc, a lump cut off at the ground.
/// Blender's `bisect_plane` with `clear_outer` and `holes_fill` (`edit::bisect_plane`, `edit::holes_fill`,
/// which a build calls itself for another `dist`, or to cut its triangles after): only the points the cut makes
/// are on the plane (`dist` 0).
pub fn bisect(obj: Obj, co: impl Into<P3>, no: impl Into<P3>, cap: bool) -> Obj {
    let world = obj.matrix_world();
    obj.edit(|part| {
        // (a bevel still to be given rounds the cut mesh, whose points are numbered anew)
        if let Some(bevel) = &mut part.bevel {
            (bevel.order, bevel.sides_from) = (EdgeOrder::Corners, None);
        }
        part.mesh.transform(&world);
    });
    let cut = crate::edit::bisect_plane(
        obj,
        co,
        no,
        crate::edit::Bisect {
            clear_outer: true,
            ..Default::default()
        },
    );
    if cap {
        let made = crate::edit::holes_fill(obj, &cut);
        obj.edit(|part| {
            for f in made {
                part.mesh.faces[f].smooth = false;
            }
        });
    }
    if let Some(back) = world.inverted() {
        obj.edit(|part| part.mesh.transform(&back));
    }
    obj
}

options! {
    /// `jitter`'s options.
    Jitter { size: f64 = 0.3, seed: f64 = 0.0, up: f64 = 1.0 }
}

/// Moves `obj`'s vertices by smooth noise up to `amount` metres, in features about `size` across, the same
/// every build for a `seed`: bark, a lumpy stone, a hand-made look. `up` scales the vertical part (0: only
/// sideways).
pub fn jitter(obj: Obj, amount: f64, o: Jitter) -> Obj {
    let world = obj.world_points();
    obj.edit(|part| {
        for (v, p) in part.mesh.verts.iter_mut().zip(&world) {
            let q = v3(
                p.x as f64 / o.size + o.seed,
                p.y as f64 / o.size - o.seed,
                p.z as f64 / o.size + o.seed * 0.5,
            );
            // three samples of the noise far apart
            let n = [(0.0, 0.0, 0.0), (31.4, 17.7, 5.9), (-13.3, 47.1, 29.2)]
                .map(|off| noise(q + V3::from(off)));
            *v += v3(n[0], n[1], n[2] * o.up) * amount;
        }
    });
    obj
}

/// A copy of `obj` mirrored across the world's plane through the origin normal to `axis` (the model's middle): build
/// one side, mirror it for the other. `name`: by default the part's own with `_mirror`.
pub fn mirrored(obj: Obj, axis: Axis, name: Option<&str>) -> Obj {
    let (mesh, way, bevel) = obj.read(|o| (o.mesh.clone(), o.uv, o.bevel.clone()));
    let name = name.map_or_else(|| format!("{}_mirror", obj.name()), str::to_string);
    // (a copy of the part with a copy of its mesh: each under the next free name after its own)
    let copy = scene::add_as(&name, &obj.data_name(), mesh);
    let mut k = [1.0_f32; 3];
    k[axis.index()] = -1.0;
    let flip = M4::diagonal(V3::new(k[0], k[1], k[2]));
    // set as the copy's world matrix, Blender takes it apart into a place, a turn and a scale (the mirror a
    // negative scale) and puts it back together
    let (loc, rot, scale) = crate::motion::decompose(&(flip * obj.matrix_world()));
    let mw = math::object_matrix(loc, rot, scale);
    // a mirroring matrix turns the faces inside out: bake it into the mesh and turn them back; the copy's world
    // matrix set to the identity, taken apart as Blender takes it (its turn about y a negative zero)
    let (loc, rot, scale) = crate::motion::decompose(&M4::IDENTITY);
    copy.edit(|o| {
        // (the copy has the bevel the part is still to be given, as it had its modifiers)
        (o.uv, o.bevel) = (way, bevel);
        o.mesh.transform(&mw);
        o.mesh.flip_normals();
        (o.loc, o.rot, o.scale) = (loc, rot, scale);
    });
    copy
}

// ---- objects ------------------------------------------------------------------------------------------

/// gamekit's `forms._object`, which a script may call itself: a part of the points `vs` and `faces` (each its
/// corners' points), its faces turned to look outwards, shaded smooth with its edges over `angle` degrees kept
/// sharp, or flat; `merge`: the points on one another made one first.
pub fn object(
    name: &str,
    vs: Vec<V3>,
    faces: &[Vec<u32>],
    material: Option<Mat>,
    smooth: bool,
    angle: f64,
    merge: bool,
) -> Obj {
    let mut mesh = Mesh::from_pydata(vs, faces);
    mesh.set_material(material);
    if merge {
        mesh.remove_doubles(1e-6);
    }
    mesh.recalc_normals();
    let obj = scene::add(name, mesh);
    shade(obj, smooth, angle)
}

options! {
    /// `object_of`'s options.
    ObjectOf { face_mats: Option<Vec<u16>> = None, sharp: f64 = 35.0, bevel: f64 = 0.0 }
}

/// The packs' `_obj`: a part of the Blender points `verts` and `faces` with the materials `mats` in its slots
/// (every face on the first, or face i on `face_mats[i]`), its faces turned to look outwards, smooth with its edges
/// over `sharp` degrees kept sharp, and its edges over 40 degrees rounded in two segments where a `bevel` is
/// given. Not `object`: no operator makes it, so the scene is not worked out, and it takes several materials.
pub fn object_of(name: &str, verts: &[P3], faces: &[Vec<u32>], mats: &[Mat], o: ObjectOf) -> Obj {
    let mut mesh = Mesh::from_pydata(verts.iter().map(|&p| V3::from(p)).collect(), faces);
    for &m in mats {
        mesh.slot(m);
    }
    if let Some(face_mats) = &o.face_mats {
        for (face, &i) in mesh.faces.iter_mut().zip(face_mats) {
            face.mat = i;
        }
    }
    mesh.recalc_normals();
    let obj = scene::add(name, mesh);
    obj.edit(|p| p.mesh.shade_smooth(o.sharp));
    if o.bevel != 0.0 {
        // (a mesh made of points and faces)
        bevel_edges(obj, o.bevel, 2, 40.0, &EdgeOrder::Corners);
    }
    obj
}

/// A bar's square profile, `w` across and `h` the other way, about its middle, from its low left corner round
/// (the packs' `_sq`): what a sweep takes for its `profile`.
pub fn rect(w: f64, h: f64) -> Vec<(f64, f64)> {
    vec![
        (-w / 2.0, -h / 2.0),
        (w / 2.0, -h / 2.0),
        (w / 2.0, h / 2.0),
        (-w / 2.0, h / 2.0),
    ]
}

/// A number or a point given where either will do.
pub fn num(x: impl Num) -> f64 {
    x.f()
}
