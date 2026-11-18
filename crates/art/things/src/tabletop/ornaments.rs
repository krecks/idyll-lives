//! TT4 small ornaments.

use super::M;
use super::common::{_place, disc, done};
use artkit::forms::{self, rounded_rect};
use artkit::kit::fillet;
use artkit::kit::{Moulding, moulding};
use artkit::materials::Mat;
use artkit::math::{
    Axis, M4, PI, TAU, V3, acos, asin, atan2, cos, cosd, degrees, radians, sin, sind,
};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{cone, join, rod, settle, spin, sweep};

/// A photo frame leaning back on its strut: a moulded black frame (mitred), a white mount round a photo of a
/// sunny hillside with someone in it, a back board.
pub fn photo_frame() -> Vec<Obj> {
    let (tilt, zc) = (-10.0, 0.116);
    let f = GameFrame::new(0.0);
    let upright = new_parts(|| {
        moulding(
            &f,
            "frame",
            &[
                (0.09, -0.115),
                (0.09, 0.115),
                (-0.09, 0.115),
                (-0.09, -0.115),
            ],
            &[
                (0.0, -0.008),
                (0.0, 0.004),
                (0.004, 0.009),
                (0.012, 0.011),
                (0.018, 0.007),
                (0.022, 0.003),
                (0.022, -0.008),
            ],
            M.frame,
            Moulding::default(),
        );
        box_!("back_board", (0.17, 0.004, 0.22), (0, 0.007, 0), M.frame);
        box_!("mount", (0.142, 0.003, 0.192), (0, 0.0035, 0), M.paper);
        box_!(
            "photo_sky",
            (0.1, 0.002, 0.14),
            (0, 0.0015, 0.01),
            M.photo_sky
        );
        let hills = [
            (-0.05, -0.06),
            (0.05, -0.06),
            (0.05, -0.012),
            (0.03, -0.005),
            (0.012, -0.016),
            (-0.01, -0.006),
            (-0.032, -0.022),
            (-0.05, -0.015),
        ];
        slab!("photo_hill", &hills, 0.002, (0, 0.0005, 0), M.photo_hill);
        turned!(
            "photo_sun",
            &[(0.009, 0.0), (0.009, 0.001)],
            M.yellow,
            at = (0.03, 0.0004, 0.048),
            rot = (90, 0, 0),
            verts = 8
        );
        ball!(
            "photo_head",
            0.012,
            (-0.008, -0.0003, 0.012),
            M.blush,
            scale = (1, 0.12, 1.15),
            seg = 8,
            rings = 5
        );
        ball!(
            "photo_body",
            0.022,
            (-0.008, -0.0003, -0.022),
            M.teapot_trim,
            scale = (1.2, 0.1, 0.95),
            seg = 8,
            rings = 5
        );
    });
    _place(
        &upright,
        M4::translation(V3::from((0.0, 0.0, zc))) * M4::rotation(radians(tilt), Axis::X),
    );
    slab!(
        "strut",
        &[
            (-0.012, -0.08),
            (0.012, -0.08),
            (0.016, 0.08),
            (-0.016, 0.08)
        ],
        0.008,
        (0, 0.048, 0.075),
        M.frame,
        rot = (25, 0, 0)
    );
    done("photoFrame", false)
}

/// The twelve hour marks round a clock's face (radius `r`, its middle at height `z`), flat on it at depth `y`, the
/// quarters longer: one mesh of flat bars facing the front.
fn _ticks(name: &str, r: f64, y: f64, z: f64, material: Mat, s: f64) -> Obj {
    let (mut vs, mut faces) = (Vec::new(), Vec::new());
    for i in 0..12 {
        let a = radians((i * 30) as f64);
        let (length, w) = ((if i % 3 == 0 { 0.014 } else { 0.009 }) * s, 0.0028);
        let (cx, cz) = (r * sin(a), z + r * cos(a));
        let (dx, dz, tx, tz) = (sin(a), cos(a), cos(a), -sin(a));
        let k = vs.len() as u32;
        for (p, q) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            vs.push(V3::from((
                cx + dx * length / 2.0 * p + tx * w / 2.0 * q,
                y,
                cz + dz * length / 2.0 * p + tz * w / 2.0 * q,
            )));
        }
        faces.push(vec![k, k + 1, k + 2, k + 3]);
    }
    let obj = forms::object(name, vs, &faces, Some(material), false, 40.0, false);
    // flat bars: face the front, where they're seen
    let back = artkit::py::sum(
        obj.read(|o| o.mesh.face_normals())
            .iter()
            .map(|n| n.y as f64),
    );
    if back > 0.0 {
        obj.edit(|o| o.mesh.flip_normals());
    }
    obj
}

/// A twin-bell alarm clock: a red drum case rolled at the back with a raised bezel, a cream face with its hour marks,
/// two brass bells on their posts with the hammer between them, a carrying loop, splayed legs on bun feet. 16.5 cm tall
/// in all (a real one 12-18 cm), built at 0.72 of its first size (`S`), which was half again too tall.
pub fn table_clock() -> Vec<Obj> {
    let s = 0.72;
    let (z, r) = (0.112 * s, 0.074 * s);
    let body = new_parts(|| {
        turned!(
            "case",
            &[
                (r * 0.9, -0.025 * s),
                (r, -0.017 * s),
                (r, 0.016 * s),
                (r * 1.01, 0.024 * s),
                (r * 0.87, 0.027 * s),
                (r * 0.86, 0.021 * s)
            ],
            M.clock,
            at = (0.0, 0.0, z),
            rot = (90, 0, 0),
            verts = 20
        );
        let face = disc("face", r * 0.862, (0.0, 0.0, 0.0), M.clock_face, 20, None);
        face.set_rotation_euler((radians(90.0), 0.0, 0.0));
        face.set_location((0.0, -0.0235 * s, z));
        _ticks("ticks", 0.054 * s, -0.0235 * s - 0.0004, z, M.dark, s);
        cyl!(
            "hub",
            0.004,
            0.003,
            (0.0, -0.025 * s - 0.006, z),
            M.red,
            verts = 8,
            rot = (90, 0, 0)
        );
        for k in [-1, 1] {
            let sk = k as f64;
            turned!(
                format!("bell{k}"),
                &[
                    (0.032 * s, 0.0),
                    (0.031 * s, 0.01 * s),
                    (0.022 * s, 0.024 * s),
                    (0.008 * s, 0.03 * s),
                    (0.0, 0.031 * s)
                ],
                M.brass,
                at = (sk * 0.0445 * s, 0.0, z + 0.06 * s),
                rot = (0, k * 30, 0),
                verts = 10
            );
            sweep!(
                format!("leg{k}"),
                &[
                    (sk * 0.04 * s, 0.0, z - 0.05 * s),
                    (sk * 0.056 * s, -0.004 * s, 0.02 * s),
                    (sk * 0.06 * s, -0.008 * s, 0.008 * s)
                ],
                0.005 * s,
                material = M.brass,
                verts = 5
            );
            turned!(
                format!("foot{k}"),
                &[
                    (0.009 * s, 0.0),
                    (0.009 * s, 0.004 * s),
                    (0.005 * s, 0.009 * s),
                    (0.0, 0.0105 * s)
                ],
                M.brass,
                at = (sk * 0.06 * s, -0.008 * s, 0.0),
                verts = 6
            );
        }
        rod!(
            "bell_post",
            (0.0, 0.0, z + r - 0.004 * s),
            (0.0, 0.0, z + 0.1 * s),
            0.0035 * s,
            material = M.brass,
            verts = 6
        );
        ball!(
            "hammer",
            0.007 * s,
            (0.0, 0.0, z + 0.103 * s),
            M.brass,
            seg = 6,
            rings = 4
        );
        sweep!(
            "loop",
            &fillet(
                &[
                    (-0.026 * s, 0.012 * s, z + r * 0.9),
                    (-0.026 * s, 0.012 * s, z + 0.112 * s),
                    (0.026 * s, 0.012 * s, z + 0.112 * s),
                    (0.026 * s, 0.012 * s, z + r * 0.9)
                ],
                0.012 * s,
                2
            ),
            0.0028 * s,
            material = M.brass,
            verts = 4
        );
    });
    join!(&body, "clock", origin = (0, 0, 0));
    // the hour and minute hands: parts of their own round the face's middle, telling the game's time
    // (`motions.clock`), at rest ten past ten
    let mut hands = Vec::new();
    for (name, (w, depth, length), y, angle) in [
        ("hour", (0.005, 0.002, 0.032 * s), -0.025 * s - 0.004, 305.0),
        ("minute", (0.004, 0.002, 0.05 * s), -0.025 * s - 0.005, 60.0),
    ] {
        let hand = new_parts(|| {
            box_!(
                format!("{name}_hand"),
                (w, depth, length),
                (
                    sind(angle) * length / 2.0,
                    y,
                    z + cosd(angle) * length / 2.0
                ),
                M.dark,
                rot = (0.0, angle, 0.0)
            );
        });
        hands.push(hinged(
            join(&hand, &format!("{name}_hand"), None),
            (0.0, y, z),
        ));
    }
    settle!();
    for hand in hands {
        spin!(hand, &hand.name(), turns = 1, seconds = 12, axis = Axis::Y); // once round clockwise; the game holds it at the time
    }
    meshes()
}

// the continents, roughly: (longitude, latitude, reach in degrees)
const CONTINENTS: [(f64, f64, f64); 13] = [
    (-100.0, 48.0, 24.0),
    (-80.0, 30.0, 14.0),
    (-60.0, -15.0, 20.0),
    (-68.0, -38.0, 10.0),
    (12.0, 50.0, 14.0),
    (35.0, 58.0, 18.0),
    (20.0, 8.0, 22.0),
    (25.0, -20.0, 14.0),
    (85.0, 50.0, 30.0),
    (110.0, 28.0, 16.0),
    (78.0, 20.0, 10.0),
    (135.0, -25.0, 13.0),
    (-42.0, 72.0, 9.0),
];

/// A face's centre as Blender gives it (`MeshPolygon.center`): a triangle's and a quad's corners summed and divided,
/// a larger face's each weighted by one over their number, in single precision.
fn face_centre(verts: &[V3], corners: &[u32]) -> V3 {
    let p = |i: usize| verts[corners[i] as usize];
    match corners.len() {
        3 => {
            let (a, b, c) = (p(0), p(1), p(2));
            V3::new(
                (a.x + b.x + c.x) / 3.0,
                (a.y + b.y + c.y) / 3.0,
                (a.z + b.z + c.z) / 3.0,
            )
        }
        4 => {
            let (a, b, c, d) = (p(0), p(1), p(2), p(3));
            V3::new(
                (a.x + b.x + c.x + d.x) / 4.0,
                (a.y + b.y + c.y + d.y) / 4.0,
                (a.z + b.z + c.z + d.z) / 4.0,
            )
        }
        n => {
            let w = 1.0_f32 / n as f32;
            let mut c = V3::new(0.0, 0.0, 0.0);
            for i in 0..n {
                let q = p(i);
                c = V3::new(c.x + q.x * w, c.y + q.y * w, c.z + q.z * w);
            }
            c
        }
    }
}

/// `min(hi, max(lo, x))` as Python takes it.
fn clamp(x: f64) -> f64 {
    let lo = if x > -1.0 { x } else { -1.0 };
    if lo < 1.0 { lo } else { 1.0 }
}

/// A desk globe: the blue sea with its continents (faces of the sphere coloured land), tilted on its axis in a brass
/// meridian ring with pins at the poles, on a turned wooden column and foot.
pub fn globe() -> Vec<Obj> {
    turned!(
        "base",
        &[
            (0.068, 0.0),
            (0.068, 0.008),
            (0.062, 0.014),
            (0.03, 0.022),
            (0.02, 0.026)
        ],
        M.wood_dark,
        verts = 12
    );
    turned!(
        "column",
        &[
            (0.016, 0.024),
            (0.011, 0.045),
            (0.015, 0.056),
            (0.008, 0.066),
            (0.006, 0.077)
        ],
        M.wood_dark,
        verts = 8
    );
    let (z, r) = (0.19, 0.105);
    // (its sides between corners)
    let sea = ball!(
        "sea",
        r / cosd(11.25),
        (0.0, 0.0, z),
        M.globe,
        rot = (23, 0, 0),
        seg = 16,
        rings = 12
    );
    sea.edit(|o| {
        let land = o.mesh.slot(M.land);
        for f in 0..o.mesh.faces.len() {
            let corners = o.mesh.corners(f).to_vec();
            let centre = face_centre(&o.mesh.verts, &corners);
            let c = centre.normalized();
            let (lat, lon) = (
                degrees(asin(clamp(c.z as f64))),
                degrees(atan2(c.y as f64, c.x as f64)),
            );
            for (clon, clat, reach) in CONTINENTS {
                let d = degrees(acos(clamp(
                    sin(radians(lat)) * sin(radians(clat))
                        + cos(radians(lat)) * cos(radians(clat)) * cos(radians(lon - clon)),
                )));
                if d < reach {
                    o.mesh.faces[f].mat = land;
                    break;
                }
            }
        }
    });
    let ring: Vec<(f64, f64, f64)> = (0..18)
        .map(|i| {
            (
                0.0,
                0.117 * cos(TAU * i as f64 / 18.0),
                z + 0.117 * sin(TAU * i as f64 / 18.0),
            )
        })
        .collect();
    let profile: Vec<(f64, f64)> = rounded_rect(0.004, 0.008, 0.001, 1)
        .into_iter()
        .step_by(2)
        .collect();
    sweep!(
        "meridian",
        &ring,
        0.0,
        profile = profile,
        material = M.brass,
        closed = true,
        up = (0, 1, 0)
    );
    let a = radians(23.0);
    for k in [-1.0, 1.0] {
        rod!(
            format!("pin{}", k as i32),
            (0.0, k * -r * 0.98 * sin(a), z + k * r * 0.98 * cos(a)),
            (0.0, k * -0.118 * sin(a), z + k * 0.118 * cos(a)),
            0.003,
            material = M.brass,
            verts = 6
        );
    }
    done("globe", false)
}

/// A snow globe: a turned wooden base with a brass band, the glass dome, inside it a snowy mound with a fir tree in
/// three tiers, a little red cabin with snow on its roof, a snowman and snowflakes in the water.
pub fn snow_globe() -> Vec<Obj> {
    turned!(
        "base",
        &[
            (0.064, 0.0),
            (0.064, 0.008),
            (0.06, 0.03),
            (0.054, 0.05),
            (0.046, 0.052),
            (0.045, 0.062),
            (0.0, 0.063)
        ],
        M.wood_dark,
        verts = 16
    );
    turned!(
        "band",
        &[(0.0634, 0.017), (0.0634, 0.019), (0.0617, 0.027)],
        M.brass,
        verts = 16
    );
    let z = 0.112;
    ball!("dome", 0.064, (0.0, 0.0, z), M.glass, seg = 14, rings = 9);
    turned!(
        "ground",
        &[(0.04, 0.06), (0.039, 0.066), (0.028, 0.073), (0.0, 0.077)],
        M.snow,
        verts = 10
    );
    rod!(
        "trunk",
        (-0.008, 0.008, 0.07),
        (-0.008, 0.008, 0.082),
        0.004,
        material = M.wood_dark,
        verts = 4
    );
    for (i, (z, r, h)) in [
        (0.079, 0.021, 0.03),
        (0.097, 0.016, 0.026),
        (0.113, 0.011, 0.022),
    ]
    .into_iter()
    .enumerate()
    {
        cone!(
            format!("tree{i}"),
            r,
            0.0,
            h,
            at = (-0.008, 0.008, z + h / 2.0),
            material = M.fir,
            verts = 7
        );
    }
    box_!("cabin", (0.02, 0.016, 0.015), (0.02, 0.012, 0.074), M.red);
    slab!(
        "roof",
        &[(-0.014, 0.0), (0.014, 0.0), (0.0, 0.011)],
        0.02,
        (0.02, 0.012, 0.081),
        M.snow
    );
    ball!(
        "snowman_base",
        0.008,
        (0.01, -0.022, 0.077),
        M.snow,
        seg = 6,
        rings = 4
    );
    ball!(
        "snowman_head",
        0.0055,
        (0.01, -0.022, 0.089),
        M.snow,
        seg = 6,
        rings = 4
    );
    for (i, p) in [
        (0.03, -0.02, 0.13),
        (-0.035, 0.0, 0.15),
        (0.01, 0.03, 0.16),
        (-0.02, -0.03, 0.1),
        (0.035, 0.02, 0.095),
        (-0.03, 0.025, 0.125),
    ]
    .into_iter()
    .enumerate()
    {
        ball!(format!("flake{i}"), 0.0035, p, M.snow, seg = 4, rings = 2);
    }
    done("snowGlobe", false)
}

/// A cup trophy: a two-step black plinth with a brass plaque, a turned stem with a knop, a flaring cup with a
/// rolled rim and two scrolled handles.
pub fn trophy() -> Vec<Obj> {
    box_!(
        "base",
        (0.1, 0.1, 0.03),
        (0, 0, 0.015),
        M.trophy_base,
        bevel = 0.004,
        segs = 1
    );
    box_!(
        "base_top",
        (0.08, 0.08, 0.012),
        (0, 0, 0.036),
        M.trophy_base,
        bevel = 0.002,
        segs = 1
    );
    box_!(
        "plaque",
        (0.06, 0.002, 0.018),
        (0, -0.0505, 0.015),
        M.trophy
    );
    turned!(
        "stem",
        &[
            (0.03, 0.042),
            (0.03, 0.047),
            (0.012, 0.058),
            (0.009, 0.085),
            (0.018, 0.095),
            (0.009, 0.104),
            (0.014, 0.11)
        ],
        M.trophy,
        verts = 12
    );
    turned!(
        "cup",
        &[
            (0.014, 0.108),
            (0.03, 0.118),
            (0.05, 0.145),
            (0.06, 0.19),
            (0.063, 0.218),
            (0.065, 0.226),
            (0.06, 0.226),
            (0.057, 0.205),
            (0.0, 0.2)
        ],
        M.trophy,
        verts = 14
    );
    for s in [-1, 1] {
        let k = s as f64;
        sweep!(
            format!("handle{s}"),
            &[
                (k * 0.056, 0.0, 0.205),
                (k * 0.066, 0.0, 0.202),
                (k * 0.0695, 0.0, 0.18),
                (k * 0.062, 0.0, 0.158),
                (k * 0.05, 0.0, 0.142)
            ],
            0.0045,
            material = M.trophy,
            verts = 6,
            resolution = 2,
            scale = [1.0, 1.1, 1.0, 0.9, 0.8]
        );
    }
    done("trophy", false)
}

/// A flat band `w` wide and `t` thick round a circle of radius `r` in the x-z plane (Blender), turning half over on
/// its way round (a Mobius band), `n` sections.
fn _mobius(name: &str, r: f64, w: f64, t: f64, n: usize, material: Mat) -> Obj {
    let prof = [
        (-w / 2.0, -t / 2.0),
        (w / 2.0, -t / 2.0),
        (w / 2.0, t / 2.0),
        (-w / 2.0, t / 2.0),
    ];
    let (mut vs, mut faces) = (Vec::new(), Vec::new());
    for i in 0..n {
        let (phi, tw) = (TAU * i as f64 / n as f64, PI * i as f64 / n as f64);
        let (c, rad) = (
            V3::from((r * cos(phi), 0.0, r * sin(phi))),
            V3::from((cos(phi), 0.0, sin(phi))),
        );
        for (u, v) in prof {
            let (uu, vv) = (u * cos(tw) - v * sin(tw), u * sin(tw) + v * cos(tw));
            vs.push(c + rad * vv + V3::from((0.0, uu, 0.0)));
        }
    }
    let n = n as u32;
    for i in 0..n {
        for k in 0..4 {
            let k2 = (k + 1) % 4;
            if i < n - 1 {
                faces.push(vec![
                    i * 4 + k,
                    i * 4 + k2,
                    (i + 1) * 4 + k2,
                    (i + 1) * 4 + k,
                ]);
            } else {
                // round to the start, half turned: corner k meets corner k + 2
                faces.push(vec![i * 4 + k, i * 4 + k2, (k2 + 2) % 4, (k + 2) % 4]);
            }
        }
    }
    forms::object(name, vs, &faces, Some(material), true, 40.0, false)
}

/// An abstract sculpture: a black band standing on edge and turning half over on its way round (a Mobius band), a
/// brass ball resting inside it, on a pale plinth with a brass name plate.
pub fn sculpture_small() -> Vec<Obj> {
    box_!(
        "plinth",
        (0.12, 0.12, 0.05),
        (0, 0, 0.025),
        M.plinth,
        bevel = 0.004,
        segs = 1
    );
    box_!("plate", (0.05, 0.002, 0.012), (0, -0.0605, 0.03), M.brass);
    let band = _mobius("ring", 0.071, 0.036, 0.008, 28, M.sculpture);
    let low = band.read(|o| {
        o.mesh
            .verts
            .iter()
            .map(|v| v.z)
            .fold(f32::INFINITY, f32::min)
    }) as f64;
    band.set_location((0.0, 0.0, 0.048 - low));
    band.set_rotation_euler((0.0, 0.0, radians(30.0)));
    ball!(
        "ball",
        0.03,
        (0.012, -0.01, 0.08),
        M.brass,
        seg = 10,
        rings = 7
    );
    done("sculptureSmall", false)
}
