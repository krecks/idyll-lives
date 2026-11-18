//! What the hot-headed pack is built from: its palette, turned and lofted pieces, fluted rings, chains, and the
//! kit's parts it uses.

use artkit::forms::Sweep;
use artkit::materials::Mat;
use artkit::math::{P3, Quat, TAU, V3, cos, pow, sin, tan};
use artkit::place::{GameFrame, Spot};
use artkit::scene::Obj;
use artkit::{loft, palette, revolve, ring, torus};
use std::cell::Cell;

palette! { M:
    body = ("#C49A6C", wood),        // the grudge desk's oak top and its chair; the desk's main part
    bag = ("#C8402F", leather),      // the heavy bag's leather
    crate_ = ("#A87547", wood),      // the smash crate and its lid (`CRATE_MAT`: `crate` is Rust's)
    shell = ("#C8402F", gloss),      // the drum shells
    zafu = ("#5F7FA0", fabric),      // the meditation cushion
    cedar = ("#C98B58", wood),       // the plunge tub's cladding
    fire = ("#C8402F", satin),       // red details: weight plate, gloves, lamp shade
    ember = ("#E07A2E", satin),
    flame = ("#F2B84B", gloss),      // logos, hazard tape
    steel = ("#AEB4BA", metal),
    steel_dark = ("#7E858C", metal),
    chrome = ("#D0D3D6", chrome),
    cymbal = ("#C9A24A", metal),
    brass = ("#C9A54A", metal),
    dark = ("#2F3236", satin),       // straps, castors, the drum throne, pen
    rubber = ("#3E4A55", satin),     // floor mats, the drum rug
    rug = ("#6E3A2E", satin),
    head = ("#F4F1EA", satin),       // drum heads
    porcelain = ("#F4F1EA", gloss),  // plates and shards
    plate_blue = ("#9CC3DB", gloss),
    plate_gold = ("#F2D7A0", gloss),
    plate_green = ("#C8E0C0", gloss),
    crate_dark = ("#7C5432", wood),  // the crate's posts and floor
    straw = ("#D2B872", matte),
    oak = ("#B88A5C", satin),        // plain wood: stool, bowl stand, drum sticks, mallet
    calm = ("#5F7FA0", satin),       // the chair's cushion
    calm_light = ("#A9C2D6", fabric),  // the meditation mat
    calm_deep = ("#4E6E8E", fabric),   // the zafu's bands and the mat's edge
    oat = ("#E6DCC8", fabric),       // the zabuton
    candle = ("#FBF6EC", satin),
    candle_flame = ("#FFD27A", gloss),
    leaf = ("#5E8F4E", satin),
    pot = ("#F4F1EA", satin),
    paper = ("#F4EFE2", matte),
    journal = ("#7A3A2A", satin),
    drawer = ("#E6E3DC", satin),     // the desk's drawer case and the chiller
    cactus = ("#6E9E58", satin),
    shell_white = ("#F2F2F0", satin),  // the plunge tub's liner and rims
    water = ("#BFE6F0", gloss),
    ice = ("#F4FAFC", gloss),
    cover = ("#5F7FA0", satin),      // the plunge cover's slats
    screen = ("#1F3A40", gloss),
    readout = ("#7FD1F0", gloss),
    hose = ("#5F6368", satin),
    towel = ("#F4EFE2", satin),
    amp = ("#E4E1DA", satin),
    speaker = ("#3A3D41", satin),
    soil = ("#4A3A2E", matte),       // the cushion's plant pot and the cactus's
    ash = ("#8E8A84", matte),        // the incense's ash
}

/// The smash crate's material, named as the pack names it.
pub const CRATE_MAT: Mat = Mat {
    name: "crate",
    ..M.crate_
};

/// The family's palette, the crate's material under its own name.
pub fn palette() -> artkit::materials::Palette {
    let colours: Vec<Mat> = M
        .palette()
        .colours
        .iter()
        .map(|&c| if c.name == "crate_" { CRATE_MAT } else { c })
        .collect();
    artkit::materials::Palette::new(&colours)
}

thread_local! {
    static N: Cell<u32> = const { Cell::new(0) };
}

/// `stem` numbered: each part its own name.
pub fn name(stem: &str) -> String {
    N.with(|n| {
        n.set(n.get() + 1);
        format!("{stem}{}", n.get())
    })
}

/// A turned piece standing at game (x, y, z): `profile` [(radius, height up), ...] from its foot, revolved.
#[allow(clippy::too_many_arguments)]
pub fn turned(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    profile: &[(f64, f64)],
    material: Mat,
    segments: usize,
    rot: (f64, f64, f64),
) -> Obj {
    revolve!(
        name,
        profile,
        at = f.at(x, y, z),
        material = material,
        segments = segments,
        rot = rot
    )
}

/// A body lofted through round rings [(radius, height up), ...] standing at game (x, y, z).
#[allow(clippy::too_many_arguments)]
pub fn lathe(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    rings: &[(f64, f64)],
    material: Mat,
    verts: usize,
    square: f64,
) -> Obj {
    let P3 { x: bx, y: by, .. } = f.at(x, 0, z);
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(r, h)| {
            ring!(
                r,
                r,
                z = y + h,
                verts = verts,
                square = square,
                at = (bx, by)
            )
        })
        .collect();
    loft!(name, &sections, material = material)
}

/// A ring of Blender points of radius `r` at height `y`, in and out `depth` (a fraction) `flutes` times round: the
/// pleats of a zafu, the ribs of a cactus.
pub fn fluted(r: f64, y: f64, verts: usize, at: (f64, f64), flutes: i32, depth: f64) -> Vec<P3> {
    (0..verts)
        .map(|k| {
            let k = k as f64;
            let v = verts as f64;
            let flute = 1.0 + depth * cos(flutes as f64 * TAU * k / v);
            P3 {
                x: at.0 + r * flute * cos(TAU * k / v),
                y: at.1 + r * flute * sin(TAU * k / v),
                z: y,
            }
        })
        .collect()
}

/// A chain of oval links from game point `a` to `b`, each turned a quarter to the last.
#[allow(clippy::too_many_arguments)]
pub fn chain(
    f: &GameFrame,
    name_: &str,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    material: Mat,
    pitch: f64,
    r: f64,
    t: f64,
) -> Vec<Obj> {
    let (a, b) = (V3::from(f.at(a.0, a.1, a.2)), V3::from(f.at(b.0, b.1, b.2)));
    let d = b - a;
    let n = 2.max(artkit::py::round(d.length() / pitch));
    let q0 = to_track_quat_xz(d);
    let mut made = Vec::new();
    for k in 0..n {
        let c = a + d * ((k as f64 + 0.5) / n as f64);
        let link = torus!(
            name(name_),
            r,
            t,
            at = c,
            material = material,
            major_segments = 8,
            minor_segments = 3
        );
        let q = mul_qt_qtqt(
            q0,
            quaternion((1.0, 0.0, 0.0), std::f64::consts::PI / 2.0 * (k % 2) as f64),
        );
        link.edit(|o| o.quat = Some(q));
        link.set_scale((d.length() / n as f64 / (2.0 * r) * 1.25, 0.75, 1.0));
        made.push(link);
    }
    made
}

// KIT: mathutils Vector.to_track_quat('X', 'Z')
/// The turn that points +X along `vec`, +Z kept up (Blender's `vec_to_quat`).
fn to_track_quat_xz(vec: V3) -> Quat {
    let len = dot(vec, vec).sqrt();
    if len == 0.0 {
        return Quat::IDENTITY;
    }
    let t = V3 {
        x: -vec.x,
        y: -vec.y,
        z: -vec.z,
    };
    let mut nor = V3 {
        x: 0.0,
        y: -t.z,
        z: t.y,
    };
    if t.y.abs() + t.z.abs() < 1e-4 {
        nor.y = 1.0;
    }
    let co = t.x / len;
    let nor = normalized(nor);
    let q = axis_angle_normalized_to_quat(nor, saacos(co));
    let m = q.to_matrix();
    let fp = m.0[2];
    let angle = -0.5 * (libm::atan2(fp[1] as f64, fp[2] as f64) as f32);
    let co = libm::cos(angle as f64) as f32;
    let si = libm::sin(angle as f64) as f32 / len;
    let q2 = Quat {
        w: co,
        x: t.x * si,
        y: t.y * si,
        z: t.z * si,
    };
    mul_qt_qtqt(q2, q)
}

// KIT: mathutils Quaternion(axis, angle)
/// The turn by `angle` (radians) about `axis`.
fn quaternion(axis: (f64, f64, f64), angle: f64) -> Quat {
    axis_angle_normalized_to_quat(normalized(V3::from(axis)), angle as f32)
}

fn dot(a: V3, b: V3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn normalized(v: V3) -> V3 {
    let d = dot(v, v);
    if d > 1.0e-35 {
        let k = 1.0 / d.sqrt();
        V3 {
            x: v.x * k,
            y: v.y * k,
            z: v.z * k,
        }
    } else {
        V3::ZERO
    }
}

fn saacos(f: f32) -> f32 {
    libm::acos(f.clamp(-1.0, 1.0) as f64) as f32
}

fn axis_angle_normalized_to_quat(axis: V3, angle: f32) -> Quat {
    let phi = 0.5 * angle;
    let (si, co) = (libm::sin(phi as f64) as f32, libm::cos(phi as f64) as f32);
    Quat {
        w: co,
        x: axis.x * si,
        y: axis.y * si,
        z: axis.z * si,
    }
}

// KIT: mathutils Quaternion @ Quaternion
fn mul_qt_qtqt(a: Quat, b: Quat) -> Quat {
    let t0 = a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z;
    let t1 = a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y;
    let t2 = a.w * b.y + a.y * b.w + a.z * b.x - a.x * b.z;
    let z = a.z * b.w + a.w * b.z + a.x * b.y - a.y * b.x;
    Quat {
        w: t0,
        x: t1,
        y: t2,
        z,
    }
}

// --- the kit's parts ------------------------------------------------------------------------------------------

// KIT: kit.tube
/// A round tube of radius `r` through game points, its bends rounded (`round` steps between each pair of
/// points; 1 keeps them sharp), its ends capped. `verts` round: by default 8, and more for one over 13 mm in
/// radius, as many as keep its outline within a millimetre of the circle (`forms::sweep`).
pub fn tube<P: Spot>(
    f: &GameFrame,
    name: &str,
    points: &[P],
    r: f64,
    material: Mat,
    verts: Option<usize>,
    round: usize,
) -> Obj {
    f.sweep(
        name,
        points,
        r,
        material,
        Sweep {
            verts,
            resolution: round,
            least: 8,
            ..Sweep::default()
        },
    )
}

// KIT: kit.fillet
/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
/// it, on a curve that never overshoots, however uneven the spacing: Catmull-Rom loops on a short leg beside a long
/// one). For handles, bent rails and draped cloth swept with `resolution` 1.
// (3.1415 is the script's own limit for a straight corner, not pi.)
#[allow(clippy::approx_constant)]
pub fn fillet(points: &[(f64, f64, f64)], radius: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let pts: Vec<V3> = points.iter().map(|&p| V3::from(p)).collect();
    let mut out = vec![pts[0]];
    for w in pts.windows(3) {
        let (a, p, c) = (w[0], w[1], w[2]);
        let (d1, d2) = (a - p, c - p);
        if d1.length() < 1e-9 || d2.length() < 1e-9 || d1.angle(d2) > 3.1415 {
            out.push(p);
            continue;
        }
        let t = (radius / 1e-6_f64.max(tan(d1.angle(d2) / 2.0)))
            .min(d1.length() * 0.5)
            .min(d2.length() * 0.5);
        let (p1, p2) = (p + d1.normalized() * t, p + d2.normalized() * t);
        for i in 0..=steps {
            let k = i as f64 / steps as f64;
            out.push(p1 * pow(1.0 - k, 2.0) + p * 2 * k * (1.0 - k) + p2 * k * k);
        }
    }
    out.push(pts[pts.len() - 1]);
    out.iter()
        .map(|p| (p.x as f64, p.y as f64, p.z as f64))
        .collect()
}
