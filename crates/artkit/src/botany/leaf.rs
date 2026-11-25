//! Leaves, stems and sprigs: meshes made by hand, point by point (families/plants.py's own shapes), and a
//! moving part made of them.

use crate::far::{is_low, low};
use crate::materials::{Mat, Palette};
use crate::math::{Axis, M4, P3, PI, TAU, V3, cos, pow, radians, sin, v3};
use crate::mesh::Mesh;
use crate::motion::{self, Squash};
use crate::parts::{join, join_all, new_parts, settle};
use crate::rand::Random;
use crate::scene::{self, Obj};
use crate::shapes::options;

pub use crate::noise::noise_at;

options! {
    /// `_mesh`'s options.
    MeshOf { smooth: bool = true }
}

/// A part of the points `verts` and `faces` (each its corners' points), its faces turned to look outwards, all
/// smooth or all flat.
pub fn _mesh<F: AsRef<[I]>, I: Copy + TryInto<u32>>(
    name: impl AsRef<str>,
    verts: impl AsRef<[V3]>,
    faces: impl AsRef<[F]>,
    material: Mat,
    o: MeshOf,
) -> Obj {
    let faces: Vec<Vec<u32>> = faces
        .as_ref()
        .iter()
        .map(|f| {
            f.as_ref()
                .iter()
                .map(|&i| {
                    i.try_into()
                        .unwrap_or_else(|_| panic!("a face's corner is a point's index"))
                })
                .collect()
        })
        .collect();
    let mut mesh = Mesh::from_pydata(verts.as_ref().to_vec(), &faces);
    mesh.recalc_normals();
    mesh.set_material(Some(material));
    for f in &mut mesh.faces {
        f.smooth = o.smooth;
    }
    scene::add(name.as_ref(), mesh)
}

/// A leaf's outline: how wide it is (0..1 of its width) at `t` of the way along it.
pub type Outline = fn(f64) -> f64;

/// The leaves' outlines, by name.
pub const PROFILES: [(&str, Outline); 9] = [
    ("oval", |t| pow(sin(PI * t), 0.7) * (1.0 - 0.2 * t)),
    ("lance", |t| pow(sin(PI * t), 0.9) * (1.0 - 0.45 * t)),
    ("heart", |t| pow(sin(PI * (0.22 + 0.78 * t)), 0.6)),
    ("strap", |t| {
        1.0_f64.min(t * 6.0) * pow(1.0_f64.min((1.0 - t) * 3.0), 0.8)
    }),
    ("petal", |t| pow(sin(PI * pow(t, 0.8)), 0.6)),
    // an apple's, a cherry's: widest below its middle, rounded to the stalk, drawn to a point
    ("ovate", |t| pow(sin(PI * pow(t, 0.72)), 0.75)),
    // a birch's: widest near the stalk, its sides nearly straight from there to a long point
    ("deltoid", |t| {
        pow(1.0_f64.min(t / 0.16), 0.7) * pow((1.0 - t) / 0.84, 1.1)
    }),
    // a magnolia's: widest above its middle, a wedge to the stalk
    ("obovate", |t| pow(sin(PI * pow(t, 1.5)), 0.7)),
    // a lemon's: widest at its middle, pointed at both ends
    ("elliptic", |t| pow(sin(PI * t), 1.15)),
];

/// The outline named `shape` (`PROFILES`).
pub fn profile(shape: &str) -> Outline {
    match PROFILES.iter().find(|(name, _)| *name == shape) {
        Some((_, f)) => *f,
        None => panic!("no leaf shape {shape:?}"),
    }
}

options! {
    /// `leaf`'s options.
    Leaf { shape: &'static str = "oval", droop: f64 = 0.0, thick: f64 = 0.008, fold: f64 = 0.25, rows: usize = 6, ring: usize = 6, notch: Option<(usize, f64)> = None, twist: f64 = 0.0 }
}

/// A closed, thin leaf from `base` (Blender metres), heading `yaw` degrees round from the back (+y) and `tilt`
/// degrees up from level, `length` long and `width` across at its widest (`shape`: `PROFILES`). `droop` bends it
/// down evenly along its length, keeping the length (about `droop` x 115 degrees by the tip), `fold` lifts its edges
/// into a V, `notch` (every, factor) narrows every nth row (a monstera's cuts, a fern's or palm's leaflets), `twist`
/// turns it about its midrib by the tip (degrees).
#[allow(clippy::too_many_arguments)]
pub fn leaf(
    name: impl AsRef<str>,
    base: impl Into<V3>,
    yaw: f64,
    tilt: f64,
    length: f64,
    width: f64,
    material: Mat,
    o: Leaf,
) -> Obj {
    let (mut rows, mut ring, mut notch) = (o.rows, o.ring, o.notch);
    if is_low() {
        (rows, ring) = (2.max(rows / 2), 3);
        notch = if rows >= 4 { notch } else { None };
    }
    let prof = profile(o.shape);
    let bend = 2.0 * o.droop;
    // the midrib's point and the normal square to it at `t`: an arc turning down by `bend` radians
    let along = |t: f64| {
        let a = -bend * t;
        if bend < 1e-4 {
            return (v3(0, length * t, 0), v3(0, 0, 1));
        }
        (
            v3(
                0,
                length * sin(bend * t) / bend,
                -length * (1.0 - cos(bend * t)) / bend,
            ),
            v3(0, -sin(a), cos(a)),
        )
    };
    let mut verts = vec![v3(0, 0, 0)];
    for i in 1..=rows {
        let t = i as f64 / (rows + 1) as f64;
        let mut hw = width / 2.0 * prof(t);
        if let Some((every, factor)) = notch
            && i % every == 0
        {
            hw *= factor;
        }
        let ht = (o.thick / 2.0 * prof(t)).max(0.0012);
        let (c, n) = along(t);
        let tw = radians(o.twist * t);
        for j in 0..ring {
            let a = TAU * j as f64 / ring as f64;
            let (x, z) = (hw * cos(a), ht * sin(a) + o.fold * (hw * cos(a)).abs());
            verts.push(c + v3(x * cos(tw) - z * sin(tw), 0, 0) + n * (x * sin(tw) + z * cos(tw)));
        }
    }
    verts.push(along(1.0).0);
    let (mut faces, tip) = (Vec::new(), verts.len() - 1);
    for j in 0..ring {
        let k = (j + 1) % ring;
        faces.push(vec![0, 1 + k, 1 + j]);
        let last = 1 + (rows - 1) * ring;
        faces.push(vec![tip, last + j, last + k]);
        for i in 0..rows - 1 {
            let (a0, b0) = (1 + i * ring, 1 + (i + 1) * ring);
            faces.push(vec![a0 + j, a0 + k, b0 + k, b0 + j]);
        }
    }
    let m = M4::translation(base.into())
        * M4::rotation(radians(-yaw), Axis::Z)
        * M4::rotation(radians(tilt), Axis::X);
    let verts: Vec<V3> = verts.iter().map(|&v| m * v).collect();
    _mesh(name, verts, faces, material, MeshOf::default())
}

options! {
    /// `tube`'s options.
    Tube { verts: usize = 6, cap: bool = true }
}

/// A swept tube through `points` (Blender metres), radius `r0` at the first and `r1` at the last.
pub fn tube<P: Into<V3> + Copy>(
    name: impl AsRef<str>,
    points: &[P],
    r0: f64,
    r1: f64,
    material: Mat,
    o: Tube,
) -> Obj {
    let (verts, cap) = (low(o.verts, 3), low(o.cap, false)); // far: ends hidden in the ground or a flower
    let pts: Vec<V3> = points.iter().map(|&p| p.into()).collect();
    let n = pts.len();
    let (mut ring_verts, mut faces) = (Vec::new(), Vec::new());
    let mut normal: Option<V3> = None;
    for (i, &p) in pts.iter().enumerate() {
        let t = (pts[(i + 1).min(n - 1)] - pts[i.saturating_sub(1)]).normalized();
        let mut carried = normal.unwrap_or_else(|| t.orthogonal().normalized());
        carried = (carried - t * carried.dot(t)).normalized(); // carried along, square to the tangent
        normal = Some(carried);
        let side = t.cross(carried);
        let r = r0 + (r1 - r0) * i as f64 / (n - 1) as f64;
        for j in 0..verts {
            let a = TAU * j as f64 / verts as f64;
            ring_verts.push(p + (carried * cos(a) + side * sin(a)) * r);
        }
    }
    for i in 0..n - 1 {
        for j in 0..verts {
            let k = (j + 1) % verts;
            faces.push(vec![
                i * verts + j,
                i * verts + k,
                (i + 1) * verts + k,
                (i + 1) * verts + j,
            ]);
        }
    }
    if cap {
        ring_verts.extend([pts[0], pts[n - 1]]);
        let (a, b) = (ring_verts.len() - 2, ring_verts.len() - 1);
        for j in 0..verts {
            let k = (j + 1) % verts;
            faces.push(vec![a, k, j]);
            faces.push(vec![b, (n - 1) * verts + j, (n - 1) * verts + k]);
        }
    }
    _mesh(name, ring_verts, faces, material, MeshOf::default())
}

/// The point `r` out from the z axis, `deg` degrees round from the back (+y), at height `z`.
pub fn polar(r: f64, deg: f64, z: f64) -> (f64, f64, f64) {
    (r * sin(radians(deg)), r * cos(radians(deg)), z)
}

options! {
    /// `sprigs`' options.
    Sprigs { n: usize = 5, length: f64 = 0.09 }
}

/// Shaggy sprigs sticking up out of a clipped crown round `centre` (the part a trim takes off), in the palette
/// `p`'s `leaf_light` and `leaf`.
pub fn sprigs(name: &str, p: &Palette, centre: impl Into<V3>, spread: f64, o: Sprigs) {
    let centre: V3 = centre.into();
    let mut rng = Random::new(name.chars().count() as i128);
    for i in 0..o.n {
        let a = i as f64 / o.n as f64 * 360.0 + rng.uniform(-20.0, 20.0);
        let base = centre + V3::from(polar(spread * rng.uniform(0.3, 1.0), a, 0.0));
        for k in 0..3 {
            let (tilt, size) = (rng.uniform(45.0, 75.0), o.length * rng.uniform(0.8, 1.2));
            if is_low() && k != 0 {
                continue; // a far version: one leaf a sprig
            }
            leaf(
                format!("{name}{i}_{k}"),
                base,
                a + (k as f64 - 1.0) * 70.0,
                tilt,
                size,
                o.length * 0.45,
                [p.get("leaf_light"), p.get("leaf")][k % 2],
                Leaf {
                    shape: "lance",
                    droop: 0.2,
                    fold: 0.2,
                    rows: 2,
                    ring: 3,
                    ..Leaf::default()
                },
            );
        }
    }
}

/// Joins what `build` makes into a moving part `name` (origin at `origin`), settles, and shrinks it away
/// (clip `clip`): the still part and the moving one.
pub fn _with_part(
    name: &str,
    still: Obj,
    build: impl FnOnce(),
    origin: impl Into<P3>,
    clip: &str,
) -> Vec<Obj> {
    if is_low() {
        // a far version: all one still part
        build();
        return settle(&[join_all(&still.name())]);
    }
    let parts = new_parts(build);
    let part = join(&parts, name, Some(origin.into()));
    settle(&[still, part]);
    motion::squash(
        part,
        clip,
        0.001,
        Squash {
            seconds: 0.8,
            ..Squash::default()
        },
    );
    vec![still, part]
}
