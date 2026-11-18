//! The garden pond (2x2): a stone-edged pool with lilies and reeds, ripples, and a koi that comes up.

use super::common::{_mesh, _own, F, M, random_of};
use crate::star;
use artkit::forms::smooth_outline;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, V3, cos, cosd, degrees, p3, radians, sin, sind};
use artkit::motion::{Ease, Path, Squash, keys, squash};
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{
    boulder, fcyl, fextrude, fsphere, fsweep, jitter, leaf, loft, revolve, ring, slide, swing,
    torus,
};

/// The rim stones' ring (game x, depth radii).
const POND: (f64, f64) = (0.86, 0.76);
/// The surface: the koi lies hidden under it at rest.
const WATER_Y: f64 = 0.07;
/// Where the koi comes up (game x, depth).
const KOI: (f64, f64) = (0.18, 0.12);

/// A flat stone: an irregular rounded outline `r` across its radius (stretched sx by sz), `h` thick, its
/// edges chamfered and its faces a little uneven, its middle at game `at`, turned `yaw` degrees.
#[allow(clippy::too_many_arguments)]
fn _flag(
    name: &str,
    r: f64,
    at: (f64, f64, f64),
    yaw: f64,
    h: f64,
    seed: f64,
    material: Mat,
    sx: f64,
    sz: f64,
    n: usize,
) -> Obj {
    let mut rng = random_of(seed);
    let mut pts = Vec::new();
    for k in 0..n {
        let a = TAU * (k as f64 + rng.uniform(-0.18, 0.18)) / n as f64;
        let rr = r * rng.uniform(0.82, 1.08);
        pts.push((rr * sx * cos(a), rr * sz * sin(a)));
    }
    let (c, s) = (cosd(yaw), sind(yaw));
    let pts: Vec<(f64, f64)> = smooth_outline(&pts, 1, true)
        .iter()
        .map(|&(x, z)| (x * c - z * s, x * s + z * c))
        .collect();
    // a slab lofted through its outline: a little in at the foot, full at the waist, chamfered in to the top
    let (cx, cy, cz) = at;
    let rows = [(0.9, -h / 2.0), (1.0, -h / 2.0 + h * 0.5), (0.86, h / 2.0)];
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(k, y)| {
            pts.iter()
                .map(|&(x, z)| F.at(cx + x * k, cy + y, cz + z * k))
                .collect()
        })
        .collect();
    let stone = loft!(name, &sections, material = material);
    jitter!(stone, 0.007, size = 0.12, seed = seed, up = 0.5)
}

/// The koi, nose to game +x (its node turns it): a lofted white body with an orange saddle and crown, a fanned
/// tail, a dorsal fin and two pectoral fins.
fn _koi(x: f64, y: f64, z: f64) {
    let m = M();
    // (along, half width, half height): nose to the tail's root
    let rows = [
        (0.13, 0.004, 0.004),
        (0.115, 0.018, 0.016),
        (0.09, 0.026, 0.026),
        (0.05, 0.03, 0.034),
        (0.0, 0.029, 0.033),
        (-0.05, 0.022, 0.025),
        (-0.09, 0.012, 0.014),
        (-0.115, 0.006, 0.008),
    ];
    let n = 10;
    let body: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(u, hw, hh)| {
            (0..n)
                .map(|k| {
                    F.at(
                        x + u,
                        y + hh * sin(TAU * k as f64 / n as f64),
                        z + hw * cos(TAU * k as f64 / n as f64),
                    )
                })
                .collect()
        })
        .collect();
    loft!("koi_body", &body, material = m.koi_white);
    // the orange saddle over the back, a hair proud of the white body, and the crown on the head
    for (name, part) in [("koi_patch", &rows[3..7]), ("koi_crown", &rows[1..3])] {
        let sections: Vec<Vec<P3>> = part
            .iter()
            .map(|&(u, hw, hh)| {
                (0..7)
                    .map(|k| {
                        F.at(
                            x + u,
                            y + 0.001 + hh * 1.05 * sin(PI * (k as f64 + 0.5) / 7.0),
                            z + hw * 1.05 * cos(PI * (k as f64 + 0.5) / 7.0),
                        )
                    })
                    .collect()
            })
            .collect();
        loft!(name, &sections, material = m.koi);
    }
    fsweep!(
        F,
        "koi_tail",
        &[(x - 0.11, y, z), (x - 0.165, y + 0.004, z)],
        0.0,
        m.koi,
        profile = [
            (-0.0045, -0.0015),
            (0.0045, -0.0015),
            (0.0045, 0.0015),
            (-0.0045, 0.0015)
        ],
        up = (0, 1, 0),
        scale = [1.0, 7.0]
    );
    fextrude!(
        F,
        "koi_fin",
        &[(0.05, 0.0), (-0.03, 0.0), (-0.05, 0.007), (0.0, 0.011)],
        0.003,
        (x, y + 0.024, z),
        m.koi
    );
    for i in [-1, 1] {
        let side = i as f64;
        fsweep!(
            F,
            format!("koi_side_fin{i}"),
            &[
                (x + 0.07, y - 0.012, z + side * 0.026),
                (x + 0.045, y - 0.02, z + side * 0.058)
            ],
            0.0,
            m.koi_white,
            profile = [
                (-0.008, -0.001),
                (0.008, -0.001),
                (0.008, 0.001),
                (-0.008, 0.001)
            ],
            up = (0, 1, 0),
            scale = [0.5, 1.4]
        );
    }
}

/// A frog squatting on a lily pad at game (x, y, z), looking `turn` degrees round from the front.
fn _frog(x: f64, y: f64, z: f64, turn: f64) {
    let m = M();
    let (c, s) = (cosd(turn), sind(turn));
    let at = |u: f64, v: f64, w: f64| (x + v * s + u * c, y + w, z + v * c - u * s);
    let rows = [
        (-0.03, 0.012, 0.008, 0.012),
        (-0.018, 0.022, 0.016, 0.018),
        (0.0, 0.025, 0.019, 0.02),
        (0.018, 0.021, 0.013, 0.019),
        (0.03, 0.011, 0.006, 0.016),
    ];
    let n = 8;
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(v, hw, hh, wc)| {
            (0..n)
                .map(|k| {
                    let (gx, gy, gz) = at(
                        hw * cos(TAU * k as f64 / n as f64),
                        v,
                        wc + hh * sin(TAU * k as f64 / n as f64),
                    );
                    F.at(gx, gy, gz)
                })
                .collect()
        })
        .collect();
    loft!("frog", &sections, material = m.frog);
    for i in [-1, 1] {
        let k = i as f64;
        fsphere!(
            F,
            format!("frog_eye{i}"),
            0.0065,
            at(k * 0.012, 0.016, 0.031),
            m.frog,
            segments = 6,
            rings = 4
        );
        fsphere!(
            F,
            format!("frog_pupil{i}"),
            0.0035,
            at(k * 0.0148, 0.0185, 0.0335),
            m.dark,
            segments = 6,
            rings = 4
        );
        // the haunches folded at its sides, the front legs straight down
        fsweep!(
            F,
            format!("frog_leg{i}"),
            &[
                at(k * 0.02, -0.02, 0.012),
                at(k * 0.03, -0.002, 0.007),
                at(k * 0.02, 0.014, 0.003)
            ],
            0.006,
            m.frog,
            verts = 6,
            scale = [1.2, 1.0, 0.6]
        );
        fsweep!(
            F,
            format!("frog_arm{i}"),
            &[at(k * 0.013, 0.014, 0.012), at(k * 0.016, 0.022, 0.001)],
            0.003,
            m.frog,
            verts = 5
        );
    }
}

pub fn pond() -> Vec<Obj> {
    let m = _own("pond");
    let mut rng = Random::new(101);
    let still = new_parts(|| {
        let (rx, rz) = POND;
        // the water: an opaque gloss block to the surface, its sides hidden under the stones; deeper in the middle
        fcyl!(
            F,
            "water",
            1.0,
            WATER_Y,
            (0, WATER_Y / 2.0, 0),
            m.water,
            verts = 32,
            scale = (rx - 0.04, rz - 0.04, 1)
        );
        fcyl!(
            F,
            "deep",
            1.0,
            0.002,
            (0.05, WATER_Y + 0.001, 0.03),
            m.water_deep,
            verts = 24,
            scale = (0.5, 0.42, 1)
        );
        // the margin: a band of dark wet mud at the water's edge, under the coping stones' lips
        torus!(
            "margin",
            1.0,
            0.03,
            at = F.at(0, WATER_Y - 0.012, 0),
            material = m.mud,
            major_segments = 32,
            minor_segments = 4
        )
        .set_scale((rx - 0.12, rz - 0.12, 0.6));
        // the rim: big flat coping stones lapping over the edge, smaller ones set behind in their gaps, pebbles
        let n = 15;
        for i in 0..n {
            let a = i as f64 / n as f64 * TAU + rng.uniform(-0.05, 0.05);
            let r = rng.uniform(0.17, 0.2);
            let (x, z) = ((rx - 0.03) * cos(a), (rz - 0.03) * sin(a));
            let h = rng.uniform(0.07, 0.09);
            _flag(
                &format!("stone{i}"),
                r,
                (x, WATER_Y + h / 2.0 - 0.025, z),
                -degrees(a),
                h,
                i as f64 * 1.37 + 3.0,
                m.body,
                0.72,
                1.15,
                6,
            );
        }
        for i in 0..n {
            let a = (i as f64 + 0.5) / n as f64 * TAU + rng.uniform(-0.08, 0.08);
            let r = rng.uniform(0.1, 0.13);
            let (x, z) = ((rx + 0.06) * cos(a), (rz + 0.08) * sin(a));
            let h = rng.uniform(0.04, 0.055);
            _flag(
                &format!("back_stone{i}"),
                r,
                (x, h / 2.0, z),
                -degrees(a),
                h,
                i as f64 * 2.11 + 40.0,
                m.body,
                0.8,
                1.1,
                5,
            );
        }
        for i in 0..12 {
            let a = (i as f64 + 0.3) / 12.0 * TAU + rng.uniform(-0.1, 0.1);
            let (x, z) = ((rx + 0.1) * cos(a), (rz + 0.1) * sin(a));
            boulder!(
                format!("pebble{i}"),
                rng.uniform(0.035, 0.05),
                at = F.at(x, 0, z),
                material = m.pebble,
                scale = (1.2, 1, 0.6),
                seed = i as f64 * 2.9,
                noise = 0.3,
                subdivisions = 1
            );
        }
        // lily pads, each with a notch, two flowers, a bud and a frog on a pad of its own
        for (i, (x, z, r)) in [
            (-0.3, 0.2, 0.11),
            (0.28, -0.24, 0.09),
            (0.42, 0.3, 0.08),
            (-0.16, -0.36, 0.085),
            (-0.02, 0.44, 0.07),
            (-0.44, -0.08, 0.075),
            (0.0, 0.3, 0.09),
        ]
        .into_iter()
        .enumerate()
        {
            _pad(
                &format!("pad{i}"),
                r,
                (x, WATER_Y + 0.003, z),
                rng.uniform(0.0, 360.0),
                m.pad,
                11,
            );
        }
        for ((x, z), petal) in [((-0.27, 0.2), m.lily), ((0.26, -0.22), m.lily_white)] {
            for (k, rr) in [0.058, 0.042, 0.027].into_iter().enumerate() {
                let k_ = k as f64;
                star!(
                    format!("lily{}_{k}", float_repr(x)),
                    F.at(x, WATER_Y + 0.01 + 0.011 * k_, z),
                    rr,
                    8,
                    petal,
                    yaw = 22.5 * k_,
                    inner = 0.4,
                    depth = 0.006,
                    cup = 0.5 + 0.35 * k_
                );
            }
            fsphere!(
                F,
                format!("lily_heart{}", float_repr(x)),
                0.011,
                (x, WATER_Y + 0.037, z),
                m.lily_heart,
                segments = 6,
                rings = 4
            );
        }
        revolve!(
            "lily_bud",
            &[
                (0.0, 0.0),
                (0.016, 0.014),
                (0.015, 0.036),
                (0.006, 0.056),
                (0.0, 0.06)
            ],
            at = F.at(0.4, WATER_Y - 0.004, 0.26),
            material = m.lily,
            segments = 8
        );
        _frog(0.0, WATER_Y + 0.006, 0.3, 25.0);
        // reeds and bulrushes at the back left, rising out of the shallows by the stones
        for i in 0..11 {
            let (x, z) = (
                -0.6 + rng.uniform(-0.12, 0.12),
                -0.44 + rng.uniform(-0.1, 0.08),
            );
            let (yaw, tilt, length, droop) = (
                rng.uniform(0.0, 360.0),
                rng.uniform(70.0, 84.0),
                rng.uniform(0.45, 0.7),
                rng.uniform(0.03, 0.12),
            );
            leaf!(
                format!("reed{i}"),
                F.at(x, WATER_Y - 0.01, z),
                yaw,
                tilt,
                length,
                0.035,
                m.reed,
                shape = "strap",
                droop = droop,
                thick = 0.006,
                fold = 0.3,
                rows = 4,
                ring = 3
            );
        }
        for i in 0..3 {
            let (x, z) = (-0.62 + i as f64 * 0.07, -0.46 + (i % 2) as f64 * 0.06);
            let h = 0.6 + i as f64 * 0.06;
            fsweep!(
                F,
                format!("rush_stem{i}"),
                &[
                    (x, WATER_Y - 0.01, z),
                    (x + 0.008, h * 0.6, z),
                    (x + 0.012, h + 0.08, z)
                ],
                0.005,
                m.reed,
                verts = 5,
                resolution = 2,
                scale = [1.0, 0.9, 0.5]
            );
            let b = F.at(x + 0.011, 0, z);
            let sections: Vec<Vec<P3>> = [
                (-0.058, 0.005),
                (-0.05, 0.015),
                (0.0, 0.018),
                (0.05, 0.015),
                (0.058, 0.005),
            ]
            .iter()
            .map(|&(dy, r)| ring!(r, r, z = h + dy, verts = 8, at = (b.x, b.y)))
            .collect();
            loft!(format!("rush{i}"), &sections, material = m.rush);
        }
        // flag irises at the front right: sword leaves in a fan and two flowers
        for i in 0..7 {
            let yaw = 150.0 + i as f64 * 14.0 + rng.uniform(-6.0, 6.0);
            let base = F.at(
                0.64 + rng.uniform(-0.03, 0.03),
                WATER_Y - 0.01,
                0.36 + rng.uniform(-0.03, 0.03),
            );
            let (tilt, length) = (rng.uniform(72.0, 84.0), rng.uniform(0.3, 0.4));
            leaf!(
                format!("iris{i}"),
                base,
                yaw,
                tilt,
                length,
                0.028,
                m.reed,
                shape = "strap",
                droop = 0.04,
                thick = 0.005,
                fold = 0.15,
                rows = 3,
                ring = 3
            );
        }
        for (i, (x, z, h)) in [(0.62, 0.36, 0.4), (0.68, 0.33, 0.34)]
            .into_iter()
            .enumerate()
        {
            fsweep!(
                F,
                format!("iris_stem{i}"),
                &[(x, WATER_Y, z), (x, h, z)],
                0.004,
                m.reed,
                verts = 4
            );
            for k in 0..3 {
                let a = (120 * k + 30 * i) as f64;
                fsweep!(
                    F,
                    format!("iris_fall{i}{k}"),
                    &[
                        (x, h, z),
                        (x + 0.028 * sind(a), h + 0.008, z + 0.028 * cosd(a)),
                        (x + 0.042 * sind(a), h - 0.022, z + 0.042 * cosd(a))
                    ],
                    0.0,
                    m.iris,
                    profile = [
                        (-0.009, -0.0015),
                        (0.009, -0.0015),
                        (0.009, 0.0015),
                        (-0.009, 0.0015)
                    ],
                    up = (0, 1, 0),
                    scale = [0.4, 1.2, 0.5]
                );
                let b = a + 60.0;
                fsweep!(
                    F,
                    format!("iris_std{i}{k}"),
                    &[
                        (x, h, z),
                        (x + 0.012 * sind(b), h + 0.04, z + 0.012 * cosd(b))
                    ],
                    0.0,
                    m.iris,
                    profile = [
                        (-0.006, -0.0015),
                        (0.006, -0.0015),
                        (0.006, 0.0015),
                        (-0.006, 0.0015)
                    ],
                    up = (0, 1, 0),
                    scale = [0.5, 1.0]
                );
            }
        }
    });
    join(&still, "pond", Some(p3(0, 0, 0)));
    // the ripples: three faint rings round where the koi comes up, spreading and settling all the time
    let rings = new_parts(|| {
        for (k, r) in [0.1, 0.19, 0.28].into_iter().enumerate() {
            torus!(
                format!("ring{k}"),
                r,
                0.006,
                at = F.at(KOI.0, WATER_Y, KOI.1),
                material = m.ripple,
                major_segments = 22,
                minor_segments = 4
            )
            .set_scale((1, 0.9, 0.35));
        }
    });
    let ripples = hinged(join(&rings, "ripples", None), F.at(KOI.0, WATER_Y, KOI.1));
    // the koi, under the surface at rest (the opaque water hides it); it rises until its back is out
    let (x, z) = KOI;
    let y = 0.035;
    let fish = new_parts(|| _koi(x, y, z));
    let koi = hinged(join(&fish, "koi", None), F.at(x, y, z));
    // heading a little to the front (a node turn, not baked)
    koi.edit(|o| o.rot.z = radians(25.0) as f32);
    settle_all();
    squash(
        ripples,
        "ripple",
        (1.3, 1.3, 1),
        Squash {
            seconds: 3.0,
            back: true,
            ..Squash::default()
        },
    );
    slide!(koi, "fish", (0, 0, 0.042), seconds = 1.2);
    swing!(koi, "fish", 30, seconds = 1.2);
    // for watching: up and down again every so often (`every`), from rest to rest
    let z = koi.location().z as f64;
    keys(
        koi,
        "surface",
        Path::Location,
        2,
        &[
            (0.0, z, Ease::Smooth),
            (1.2, z + 0.04, Ease::Smooth),
            (2.4, z, Ease::Smooth),
        ],
    );
    meshes()
}

/// A lily pad: a thin disc `r` across with a notch cut to its middle, its rim turned up a little, lying at game
/// `at`.
fn _pad(name: &str, r: f64, at: (f64, f64, f64), yaw: f64, material: Mat, n: usize) -> Obj {
    let gap = 0.5;
    let (mut top, mut bottom): (Vec<V3>, Vec<V3>) = (Vec::new(), Vec::new());
    for i in 0..=n {
        let a = gap / 2.0 + (TAU - gap) * i as f64 / n as f64;
        let lift = 0.006 * (1.0 + 0.5 * sin(3.0 * a));
        top.push((r * cos(a), r * sin(a), 0.003 + lift).into());
        bottom.push((r * cos(a), r * sin(a), -0.001 + lift).into());
    }
    let mut verts = top;
    verts.extend(bottom);
    verts.extend([V3::from((0, 0, 0.003)), V3::from((0, 0, -0.003))]);
    let m = (n + 1) as u32;
    let n = n as u32;
    let (ct, cb) = (2 * m, 2 * m + 1);
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for i in 0..n {
        faces.extend([
            vec![ct, i, i + 1],
            vec![cb, m + i + 1, m + i],
            vec![i, m + i, m + i + 1, i + 1],
        ]);
    }
    faces.extend([vec![ct, cb, m, 0], vec![ct, n, m + n, cb]]);
    let obj = _mesh(name, verts, &faces, material, false);
    obj.set_location(F.at(at.0, at.1, at.2));
    obj.set_rotation((0.0, 0.0, yaw));
    obj
}
