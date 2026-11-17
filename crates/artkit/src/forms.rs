//! Forms: the shapes the primitives can't make, each one mesh part (Blender axes, metres), deterministic, with its
//! material, as the shapes' builders are. A profile swept along a path (`sweep`: rails, piping, bent tube, a
//! cabriole leg, a branch), rings lofted into a body (`loft`, `ring`: a bottle, a lamp base, a rounded cabinet),
//! and what is done to a part after (`jitter`).
//!
//! Smooth forms are shaded smooth with their edges over `angle` degrees kept sharp, so a cap or a square
//! profile stays crisp without a bevel.

use crate::materials::Mat;
use crate::math::{self, Num, P3, TAU, V3, v3};
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

// ---- paths --------------------------------------------------------------------------------------------

/// The points, with `resolution` - 1 more on a Catmull-Rom curve between each pair (1: as they are). Returns
/// (points, the fraction of the way along the given points at each, 0..1).
fn catmull(pts: &[V3], resolution: usize, closed: bool) -> (Vec<V3>, Vec<f64>) {
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

// ---- after making -------------------------------------------------------------------------------------

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

// ---- objects ------------------------------------------------------------------------------------------

/// A part of the points `vs` and `faces`, its faces turned to look outwards, shaded.
pub(crate) fn object(
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

/// A number or a point given where either will do.
pub fn num(x: impl Num) -> f64 {
    x.f()
}
