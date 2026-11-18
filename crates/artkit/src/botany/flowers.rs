//! Flowers: a face of separate petals (`rays`), a star, a centre, a tulip's cup, a spike or a ball of florets
//! (families/plants.py's flower shapes).

use crate::botany::leaf::{_mesh, MeshOf, polar};
use crate::botany::pots::{_put, Put};
use crate::far::{is_low, low};
use crate::forms::{Loft, Ring, loft, ring};
use crate::materials::{Mat, Palette};
use crate::math::{Axis, M4, P3, PI, TAU, V3, cos, pow, radians, sin, v3};
use crate::noise::noise_at;
use crate::scene::Obj;
use crate::shapes::{Sphere, options, sphere};

options! {
    /// `rays`' options.
    Rays { yaw: f64 = 0.0, tilt: f64 = 0.0, inner: f64 = 0.3, cup: f64 = 0.15, droop: f64 = 0.0, thick: f64 = 0.006, width: f64 = 0.75, centre: f64 = 0.0, shoulder: f64 = 0.0 }
}

/// A flower's face of `n` separate petals out to `r`, each rounded at its tip (two points) and narrowing to the
/// middle (`inner` of `r`), cupped up by `cup` of their reach or hanging back by `droop` at their tips, `thick` at
/// the middle; a closed sheet with no walls (6 triangles a petal; 10 with a `shoulder`, where a strap-shaped petal's
/// sides start, a share of `r`). Faces up, leaning `tilt` towards `yaw`. A far version has a third of the petals,
/// wider. `centre`: a raised boss over the middle, that high.
pub fn rays(
    name: impl AsRef<str>,
    at: impl Into<V3>,
    r: f64,
    n: usize,
    material: Mat,
    o: Rays,
) -> Obj {
    let (mut n, mut width) = (n, o.width);
    if is_low() {
        (n, width) = (3.max(n / 3), 1.0_f64.min(width * 1.8));
    }
    let mut pts: Vec<(f64, f64)> = Vec::new();
    for k in 0..n {
        let a = TAU * k as f64 / n as f64;
        let half = PI / n as f64 * width;
        if o.shoulder != 0.0 {
            pts.push((r * o.shoulder, a - half * 0.8));
        }
        pts.extend([(r, a - half * 0.6), (r, a + half * 0.6)]);
        if o.shoulder != 0.0 {
            pts.push((r * o.shoulder, a + half * 0.8));
        }
        pts.push((r * o.inner, a + PI / n as f64)); // between two petals, in at the middle
    }
    let mut verts = Vec::with_capacity(pts.len() + 2);
    for &(rr, a) in &pts {
        let f = rr / r;
        let z = (o.cup - o.droop) * rr * f;
        verts.push(v3(rr * cos(a), rr * sin(a), z));
    }
    let m = verts.len();
    verts.extend([v3(0, 0, o.thick / 2.0 + o.centre), v3(0, 0, -o.thick / 2.0)]);
    let (ct, cb) = (m, m + 1);
    let mut faces = Vec::with_capacity(2 * m);
    for k in 0..m {
        let j = (k + 1) % m;
        faces.extend([[ct, k, j], [cb, j, k]]);
    }
    let obj = _mesh(name, verts, faces, material, MeshOf { smooth: true });
    obj.edit(|part| part.mesh.shade_smooth(70.0));
    _put(
        obj,
        at,
        Put {
            yaw: o.yaw,
            tilt: o.tilt,
            ..Put::default()
        },
    )
}

options! {
    /// `dome`'s options.
    Dome { yaw: f64 = 0.0, tilt: f64 = 0.0, lift: f64 = 0.004 }
}

/// A flower's centre: a low dome on its face (none on a far version: too small to see).
pub fn dome(
    name: impl AsRef<str>,
    at: impl Into<V3>,
    r: f64,
    h: f64,
    material: Mat,
    o: Dome,
) -> Option<Obj> {
    if is_low() {
        return None;
    }
    let obj = sphere(
        name,
        1.0,
        Sphere {
            material: Some(material),
            segments: low(6, 4),
            rings: 3,
            scale: P3 { x: r, y: r, z: h },
            ..Sphere::default()
        },
    );
    Some(_put(
        obj,
        at,
        Put {
            yaw: o.yaw,
            tilt: o.tilt,
            lift: o.lift,
        },
    ))
}

options! {
    /// `star`'s options.
    Star { yaw: f64 = 0.0, tilt: f64 = 0.0, inner: f64 = 0.5, depth: f64 = 0.008, cup: f64 = 0.2 }
}

/// A flower's face: a thin star of `n` petals, `r` across its radius, the petals lifted by `cup` (a fraction of
/// their reach), facing up and leaning `tilt` degrees towards `yaw`.
pub fn star(
    name: impl AsRef<str>,
    at: impl Into<V3>,
    r: f64,
    n: usize,
    material: Mat,
    o: Star,
) -> Obj {
    let n = low(n, 3.max(n / 3));
    let (depth, cup) = (o.depth, o.cup);
    let (mut top, mut bottom) = (Vec::new(), Vec::new());
    for k in 0..2 * n {
        let rr = if k % 2 == 0 { r } else { r * o.inner };
        let a = PI * k as f64 / n as f64;
        let z = cup * rr;
        top.push(v3(rr * cos(a), rr * sin(a), z + depth / 2.0));
        bottom.push(v3(rr * cos(a), rr * sin(a), z - depth / 2.0));
    }
    let mut verts = top;
    verts.extend(bottom);
    verts.extend([v3(0, 0, depth / 2.0), v3(0, 0, -depth / 2.0)]);
    let m = 2 * n;
    let (ct, cb) = (2 * m, 2 * m + 1);
    let mut faces: Vec<Vec<usize>> = Vec::with_capacity(3 * m);
    for k in 0..m {
        let j = (k + 1) % m;
        faces.extend([
            vec![ct, k, j],
            vec![cb, m + j, m + k],
            vec![k, m + k, m + j, j],
        ]);
    }
    _put(
        _mesh(name, verts, faces, material, MeshOf { smooth: false }),
        at,
        Put {
            yaw: o.yaw,
            tilt: o.tilt,
            ..Put::default()
        },
    )
}

options! {
    /// `tulip_head`'s options.
    TulipHead { yaw: f64 = 0.0, tilt: f64 = 0.0 }
}

/// A tulip: a closed cup of three petals round, their tips standing up over the gaps between them.
pub fn tulip_head(
    name: impl AsRef<str>,
    at: impl Into<V3>,
    r: f64,
    h: f64,
    material: Mat,
    o: TulipHead,
) -> Obj {
    let n = low(6, 3);
    let mut secs: Vec<Vec<P3>> = Vec::new();
    for (z, rr, tips) in [
        (0.0, r * 0.3, 0.0),
        (h * 0.3, r * 0.85, 0.0),
        (h * 0.65, r, 0.0),
        (h, r * 0.7, h * 0.16),
    ] {
        secs.push(
            (0..n)
                .map(|k| P3 {
                    x: rr * cos(TAU * k as f64 / n as f64),
                    y: rr * sin(TAU * k as f64 / n as f64),
                    z: z + if k % 2 == 0 { tips } else { 0.0 },
                })
                .collect(),
        );
    }
    let obj = loft(
        name,
        &secs,
        Loft {
            material: Some(material),
            ..Loft::default()
        },
    );
    obj.edit(|part| part.mesh.transform(&M4::rotation(radians(30.0), Axis::Z)));
    _put(
        obj,
        at,
        Put {
            yaw: o.yaw,
            tilt: o.tilt,
            ..Put::default()
        },
    )
}

options! {
    /// `lupin_spike`'s options.
    LupinSpike { seed: f64 = 0.0 }
}

/// A lupin's spike of pea flowers: a tapering column, bumpy with whorls of florets, `h` tall from `at`.
pub fn lupin_spike(
    name: impl AsRef<str>,
    at: impl Into<P3>,
    h: f64,
    r: f64,
    material: Mat,
    o: LupinSpike,
) -> Obj {
    let _ = o.seed;
    let at: P3 = at.into();
    let rings = low(8, 4);
    let n = low(6, 4);
    let mut secs = Vec::with_capacity(rings + 1);
    for k in 0..=rings {
        let t = k as f64 / rings as f64;
        let rr = r
            * pow(1.0 - t, 0.8)
            * (1.0
                + if !is_low() {
                    0.25 * sin(t * PI * 9.0)
                } else {
                    0.0
                })
            + 0.002;
        secs.push(ring(
            rr,
            rr,
            Ring {
                z: at.z + h * t,
                verts: n,
                at: (at.x, at.y),
                ..Ring::default()
            },
        ));
    }
    loft(
        name,
        &secs,
        Loft {
            material: Some(material),
            ..Loft::default()
        },
    )
}

/// A moth orchid's flower: three narrow sepals, two broad round petals across them and a dark lip (the palette
/// `p`'s `bloom` and `bloom_accent`), facing `facing` degrees round (yaw), upright.
pub fn _flower(name: &str, at: impl Into<V3>, facing: f64, size: f64, p: &Palette) {
    let at: V3 = at.into();
    rays(
        format!("{name}_sepals"),
        at,
        size,
        3,
        p.get("bloom"),
        Rays {
            yaw: facing,
            tilt: 90.0,
            inner: 0.15,
            cup: 0.12,
            width: 0.55,
            thick: 0.004,
            ..Rays::default()
        },
    );
    rays(
        format!("{name}_petals"),
        at + V3::from(polar(0.002, facing, 0.0)),
        size * 0.95,
        2,
        p.get("bloom"),
        Rays {
            yaw: facing,
            tilt: 90.0,
            inner: 0.1,
            cup: 0.1,
            width: 0.75,
            thick: 0.004,
            ..Rays::default()
        },
    );
    sphere(
        format!("{name}_lip"),
        size * 0.26,
        Sphere {
            at: (at + V3::from(polar(0.008, facing, -size * 0.2))).into(),
            material: Some(p.get("bloom_accent")),
            segments: low(6, 4),
            rings: low(4, 3),
            scale: P3 {
                x: 1.0,
                y: 1.0,
                z: 1.3,
            },
            ..Sphere::default()
        },
    );
}

options! {
    /// `florets`' options.
    Florets { bumps: f64 = 3.0, amount: f64 = 0.14, flat: f64 = 0.85, segments: usize = 10, rings: usize = 7, seed: f64 = 0.0 }
}

/// A ball of florets (a hydrangea's mophead, a marigold's pompom, a geranium's truss): a sphere `r` round, its
/// surface bumpy with small heads (`bumps` across it, standing `amount` of `r` proud), squashed to `flat`, its
/// bottom at `at`.
pub fn florets(name: impl AsRef<str>, r: f64, at: impl Into<V3>, material: Mat, o: Florets) -> Obj {
    let (bumps, amount, seed) = (o.bumps, o.amount, o.seed);
    let (segments, rings) = (
        low(o.segments, 6.max(o.segments / 2)),
        low(o.rings, 3.max(o.rings / 2)),
    );
    let obj = sphere(
        name,
        r,
        Sphere {
            material: Some(material),
            segments,
            rings,
            scale: P3 {
                x: 1.0,
                y: 1.0,
                z: o.flat,
            },
            ..Sphere::default()
        },
    );
    let at: V3 = at.into();
    let far = is_low();
    obj.edit(|part| {
        if !far {
            for v in &mut part.mesh.verts {
                let p = *v;
                let n = noise_at(
                    p.x as f64 / r * bumps + seed,
                    p.y as f64 / r * bumps - seed,
                    p.z as f64 / r * bumps + seed * 0.7,
                );
                *v = p * (1.0 + amount * (n.abs() * 2.0 - 0.6));
            }
        }
        let bottom = part
            .mesh
            .verts
            .iter()
            .map(|v| v.z)
            .fold(f32::INFINITY, f32::min);
        part.mesh.translate(at - v3(0, 0, bottom));
    });
    obj
}

/// A rose's cup of petals, its rim curled in over a tight middle (the profile turns back in and up again).
pub const ROSE: [(f64, f64); 8] = [
    (0.0, 0.0),
    (0.02, 0.004),
    (0.048, 0.026),
    (0.058, 0.052),
    (0.046, 0.068),
    (0.026, 0.056),
    (0.012, 0.064),
    (0.0, 0.054),
];

options! {
    /// `rose_head`'s options.
    RoseHead { yaw: f64 = 0.0, tilt: f64 = 0.0 }
}

/// A rose: an open cup of five broad outer petals round a tighter cup of four, turned a little.
pub fn rose_head(name: &str, at: impl Into<V3>, r: f64, material: Mat, o: RoseHead) {
    let at: V3 = at.into();
    rays(
        format!("{name}_out"),
        at,
        r,
        5,
        material,
        Rays {
            yaw: o.yaw,
            tilt: o.tilt,
            inner: 0.25,
            cup: 0.75,
            width: 1.05,
            thick: 0.01,
            ..Rays::default()
        },
    );
    if is_low() {
        return;
    }
    rays(
        format!("{name}_in"),
        at + v3(0, 0, r * 0.12),
        r * 0.62,
        4,
        material,
        Rays {
            yaw: o.yaw + 30.0,
            tilt: o.tilt,
            inner: 0.2,
            cup: 1.6,
            width: 1.1,
            thick: 0.012,
            centre: r * 0.15,
            ..Rays::default()
        },
    );
}

/// The meadow's flowers' colours, in the palette's names.
pub const MEADOW_FLOWERS: [&str; 5] = ["bloom5", "bloom2", "bloom3", "bloom", "bloom4"];
