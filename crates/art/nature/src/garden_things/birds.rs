//! The planter box, the birdhouse and the bird feeders.

use super::common::{_name, _own, _prism, F, M, Run, moulding};
use crate::garden_decor::common::fillet;
use crate::songbird;
use artkit::far::{is_low, low};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{P3, TAU, V3, cos, cosd, degrees, p3, sin, sind, v3};
use artkit::motion::{Squash, squash};
use artkit::parts::{Settle, join, join_all, measure, new_parts, settle_all, settle_with};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{dome, fbox, fcyl, fextrude, fsweep, leaf, lump, revolve, star, sweep, torus};

// --- the planter box (1x1): a plank trough on feet, full of leaves and flowers ---------------------------------
/// The depth it is centred on: the trough 10 cm back (as the old part's), leaves over its front.
pub const PLANTER_C: f64 = -0.09;

#[allow(non_snake_case)]
pub fn planterBox() -> Vec<Obj> {
    let m = _own("planterBox");
    let mut rng = Random::new(58);
    let z0 = -0.1; // the old part's trough stands 10 cm back on its tile
    let (w, d, h) = (0.9, 0.42, 0.44);
    let full = !is_low();
    // corner posts, their feet chamfered off the ground
    for i in [-1, 1] {
        for j in [-1, 1] {
            let (s, e) = (i as f64, j as f64);
            fbox!(
                F,
                format!("leg{i}{j}"),
                0.06,
                h,
                0.06,
                (s * (w / 2.0 - 0.03), h / 2.0, z0 + e * (d / 2.0 - 0.03)),
                m.box_trim,
                bevel = low(0.008, 0.0),
                bevel_segments = 1
            );
        }
    }
    // three planks a side, a little gap between
    for k in 0..3 {
        let y = 0.07 + k as f64 * 0.125 + 0.06;
        for j in [-1, 1] {
            let e = j as f64;
            fbox!(
                F,
                format!("plank{k}{j}"),
                w - 0.1,
                0.115,
                0.03,
                (0, y, z0 + e * (d / 2.0 - 0.035)),
                m.body,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
        for i in [-1, 1] {
            let s = i as f64;
            fbox!(
                F,
                format!("end{k}{i}"),
                0.03,
                0.115,
                d - 0.1,
                (s * (w / 2.0 - 0.035), y, z0),
                m.body,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
    }
    fbox!(F, "floor", w - 0.08, 0.02, d - 0.08, (0, 0.08, z0), m.body);
    // a batten down the middle of each long side, screwed to the planks
    for j in [-1, 1] {
        let e = j as f64;
        fbox!(
            F,
            format!("batten{j}"),
            0.06,
            h - 0.1,
            0.02,
            (0, 0.07 + (h - 0.1) / 2.0, z0 + e * (d / 2.0 - 0.01)),
            m.box_trim,
            bevel = low(0.005, 0.0),
            bevel_segments = 1
        );
        if full {
            for k in 0..3 {
                fcyl!(
                    F,
                    format!("screw{j}{k}"),
                    0.006,
                    0.004,
                    (0, 0.13 + k as f64 * 0.125, z0 + e * (d / 2.0 + 0.001)),
                    m.bracket,
                    rot = (90, 0, 0),
                    verts = 6
                );
            }
        }
    }
    // the cap rail: one moulded frame round the top, its corners mitred, its outer edge rounded over
    let (hw, hd) = (w / 2.0 + 0.02, d / 2.0 + 0.02);
    let rail = [(hw, z0 - hd), (hw, z0 + hd), (-hw, z0 + hd), (-hw, z0 - hd)];
    moulding(
        &F,
        "rim",
        &rail,
        &[
            (0.0, 0.0),
            (0.0, low(0.02, 0.03)),
            (low(0.008, 0.0), 0.03),
            (0.07, 0.03),
            (0.07, 0.0),
        ],
        m.box_trim,
        h,
        Run::Top,
        None,
    );
    // steel corner brackets on the front, bolted through
    if full {
        for i in [-1, 1] {
            let s = i as f64;
            let (x, zf) = (s * (w / 2.0 - 0.03), z0 + d / 2.0 + 0.002);
            for y in [0.12, h - 0.06] {
                let at = float_repr(y);
                fbox!(
                    F,
                    format!("bracket{i}{at}"),
                    0.075,
                    0.06,
                    0.004,
                    (x, y, zf),
                    m.bracket
                );
                fbox!(
                    F,
                    format!("bracket_side{i}{at}"),
                    0.004,
                    0.06,
                    0.05,
                    (s * (w / 2.0 + 0.002), y, zf - 0.025),
                    m.bracket
                );
                for b in [-1, 1] {
                    fcyl!(
                        F,
                        format!("bolt{i}{at}{b}"),
                        0.006,
                        0.005,
                        (x + b as f64 * 0.022, y, zf + 0.003),
                        m.bracket,
                        rot = (90, 0, 0),
                        verts = 6
                    );
                }
            }
        }
    }
    fbox!(
        F,
        "soil",
        w - 0.08,
        0.03,
        d - 0.08,
        (0, h - 0.025, z0),
        m.soil
    );
    // leafy clumps along the box, flowers over them, and a few leaves trailing over the front
    let greens = [m.leaf, m.leaf_dark, m.leaf_light];
    let mut clumps: Vec<(Obj, V3)> = Vec::new();
    // two staggered rows of leafy clumps, overlapping into one mass
    for i in 0..12 {
        let row = i % 2;
        let x = -0.36 + i as f64 * 0.065 + rng.uniform(-0.015, 0.015);
        let zz = z0 + (if row != 0 { 0.065 } else { -0.06 }) + rng.uniform(-0.02, 0.02);
        let r = 0.075 * rng.uniform(0.9, 1.15);
        let clump = lump!(
            format!("clump{i}"),
            r,
            F.at(x, h - 0.04, zz),
            greens[i % 3],
            scale = (1.25, 1.15, 1.2),
            seed = i as f64 * 1.3,
            noise = 0.32
        );
        clumps.push((clump, F.at(x, 0, zz).into()));
    }
    // flowers sitting on the clumps, leaning outwards with the leaves under them, a yellow eye in each
    let cols = [m.bloom, m.bloom2, m.bloom3, m.bloom4, m.bloom5];
    for i in 0..22 {
        let (clump, c) = clumps[(i * 5) % 12];
        let (a, rr) = (rng.uniform(0.0, TAU), rng.uniform(0.0, 0.06));
        let mut at = c + v3(rr * sin(a), rr * cos(a), 0);
        let near = clump.read(|o| {
            o.mesh
                .verts
                .iter()
                .filter(|&&v| (v - at).xy_length() < 0.03)
                .map(|v| v.z as f64)
                .fold(None, |top: Option<f64>, z| {
                    Some(top.map_or(z, |t| t.max(z)))
                })
        });
        at.z = (near.unwrap_or(h + 0.15) - 0.006) as f32;
        star!(
            format!("flower{i}"),
            P3::from(at),
            0.032,
            5,
            cols[i % 5],
            yaw = degrees(a),
            tilt = rr / 0.06 * 50.0,
            inner = 0.6,
            cup = 0.15
        );
        if full && i % 2 == 0 {
            dome!(
                format!("eye{i}"),
                P3::from(at + v3(0, 0, 0.003)),
                0.008,
                0.005,
                m.eye,
                yaw = degrees(a),
                tilt = rr / 0.06 * 50.0
            );
        }
    }
    for (i, x) in [-0.3, -0.1, 0.12, 0.3].into_iter().enumerate() {
        for k in 0..2 {
            let size = rng.uniform(0.12, 0.16);
            if is_low() && k != 0 {
                continue;
            }
            leaf!(
                format!("trail{i}_{k}"),
                F.at(x + k as f64 * 0.04, h + 0.03, z0 + d / 2.0 - 0.035),
                (180 + (k as i32 * 2 - 1) * 20) as f64,
                30.0,
                size,
                0.05,
                [m.leaf_light, m.leaf][k],
                shape = "lance",
                droop = 1.1,
                fold = 0.2,
                rows = 5,
                ring = 3
            );
        }
    }
    let planter = join_all("planterBox");
    // centred on PLANTER_C deep, the far version too (its leaves are coarser)
    let (lo, hi) = measure(&[planter]);
    planter.edit(|o| o.loc.y = (o.loc.y as f64 + (-PLANTER_C - (lo[1] + hi[1]) / 2.0)) as f32);
    settle_with(
        &[planter],
        Settle {
            centre: false,
            floor: true,
        },
    )
}

// --- the birdhouse (1x1): a little cream house with a red roof on a post --------------------------------------
pub fn birdhouse() -> Vec<Obj> {
    let m = _own("birdhouse");
    // the post, its edges chamfered, on a little collar of earth; a platform on two braces carries the house
    fbox!(
        F,
        "post",
        0.07,
        1.52,
        0.07,
        (0, 0.76, 0),
        m.post,
        bevel = 0.01,
        bevel_segments = 1
    );
    revolve!(
        "mound",
        &[
            (0.0, 0.0),
            (0.1, 0.0),
            (0.07, 0.025),
            (0.04, 0.035),
            (0.0, 0.035)
        ],
        at = F.at(0, 0, 0),
        material = m.soil,
        segments = 10
    );
    fbox!(
        F,
        "bracket",
        0.2,
        0.025,
        0.2,
        (0, 1.505, 0),
        m.post,
        bevel = 0.006,
        bevel_segments = 1
    );
    for j in [-1, 1] {
        let e = j as f64;
        fsweep!(
            F,
            format!("brace{j}"),
            &[(0, 1.36, e * 0.035), (0, 1.49, e * 0.09)],
            0.0,
            m.post,
            profile = [
                (-0.012, -0.015),
                (0.012, -0.015),
                (0.012, 0.015),
                (-0.012, 0.015)
            ],
            up = (1, 0, 0)
        );
    }
    // the house: a gabled box, its ridge running front to back, clad in lapped boards down its sides
    _prism(
        "house",
        &[
            (-0.12, 1.5),
            (0.12, 1.5),
            (0.12, 1.76),
            (0.0, 1.86),
            (-0.12, 1.76),
        ],
        -0.11,
        0.11,
        m.body,
    );
    for i in [-1, 1] {
        let s = i as f64;
        for k in 0..5 {
            let y = 1.545 + k as f64 * 0.05;
            fbox!(
                F,
                format!("board{i}{k}"),
                0.008,
                0.05,
                0.215,
                (s * 0.124, y, 0),
                m.body,
                rot = (0, -i * 4, 0)
            );
        }
    }
    // the front: a gable board proud of the wall, the round hole through it into the dark, a ring round it
    let gable = [
        (-0.125, 1.52),
        (0.125, 1.52),
        (0.125, 1.765),
        (0.0, 1.865),
        (-0.125, 1.765),
    ];
    let hole: Vec<(f64, f64)> = (0..14)
        .map(|k| {
            (
                0.033 * cos(TAU * k as f64 / 14.0),
                1.68 + 0.033 * sin(TAU * k as f64 / 14.0),
            )
        })
        .collect();
    fextrude!(
        F,
        "front",
        &gable,
        0.012,
        (0, 0, 0.114),
        m.body,
        holes = [hole]
    );
    fcyl!(
        F,
        "hole",
        0.035,
        0.002,
        (0, 1.68, 0.111),
        m.dark,
        rot = (90, 0, 0),
        verts = 14
    );
    torus!(
        "hole_ring",
        0.04,
        0.006,
        at = F.at(0, 1.68, 0.121),
        material = m.post,
        rot = (90, 0, 0),
        major_segments = 14,
        minor_segments = 4
    );
    // the perch: a dowel out under the hole, a robin on it looking along the front
    fsweep!(
        F,
        "perch",
        &[(0, 1.6, 0.11), (0, 1.6, 0.165)],
        0.007,
        m.post,
        verts = 8
    );
    songbird!(
        &F,
        "robin",
        0.0,
        1.607,
        0.142,
        m.robin,
        m.robin_breast,
        m.beak,
        eye = m.dark,
        turn = 90,
        size = 0.85
    );
    // the roof: two overhanging boards with a fascia under their eaves and a rounded ridge cap
    for i in [-1, 1] {
        let s = i as f64;
        fbox!(
            F,
            format!("roof{i}"),
            0.2,
            0.022,
            0.29,
            (s * 0.075, 1.83, 0),
            m.roof,
            rot = (0, -i * 40, 0),
            bevel = 0.005,
            bevel_segments = 1
        );
        // shingle courses: each a strip lapped over the one below
        for k in 0..3 {
            let t = 0.042 + k as f64 * 0.048;
            fbox!(
                F,
                format!("shingle{i}{k}"),
                0.06,
                0.012,
                0.292,
                (s * (0.161 - t * cosd(40.0)), 1.777 + t * sind(40.0), 0),
                m.roof,
                rot = (0, -i * 40, 0),
                bevel = 0.002,
                bevel_segments = 1
            );
        }
    }
    fsweep!(
        F,
        "ridge",
        &[(0, 1.9, -0.15), (0, 1.9, 0.15)],
        0.0,
        m.roof,
        profile = rounded_rect(0.03, 0.034, 0.012, 2),
        up = (0, 0, 1)
    );
    fbox!(
        F,
        "floor",
        0.27,
        0.02,
        0.25,
        (0, 1.505 + 0.012, 0),
        m.body,
        bevel = 0.004,
        bevel_segments = 1
    );
    // corner trim
    for i in [-1, 1] {
        let s = i as f64;
        fbox!(
            F,
            format!("trim{i}"),
            0.02,
            0.26,
            0.02,
            (s * 0.12, 1.63, 0.105),
            m.plate,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("trim_back{i}"),
            0.02,
            0.26,
            0.02,
            (s * 0.12, 1.63, -0.105),
            m.plate,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    join_all("birdhouse");
    settle_all();
    meshes()
}

// --- the bird feeders (1x1): two seed tubes on a shepherd's hook; a robin on one, a blue tit lands for watching -
/// The hook's plane (the old part's).
const FEED_Z: f64 = -0.2;

/// A seed tube hanging from the hook's end at `x`: a wire bail, a domed green lid, the glass tube full of seed
/// with two feeding ports and their pegs, a base and a seed tray with a lip, a perch ring on four spokes.
fn _feeder(x: f64) {
    let m = M();
    let z = FEED_Z;
    let b = F.at(x, 0, z);
    // the bail: a loop over the hook's tip, down to the lid's knob
    fsweep!(
        F,
        _name("hanger"),
        &[
            (x - 0.012, 1.705, z),
            (x - 0.012, 1.765, z),
            (x, 1.782, z),
            (x + 0.012, 1.765, z),
            (x + 0.012, 1.705, z)
        ],
        0.0025,
        m.iron,
        verts = 4,
        resolution = 2
    );
    revolve!(
        _name("cap"),
        &[
            (0.0, 1.645),
            (0.074, 1.645),
            (0.076, 1.655),
            (0.068, 1.668),
            (0.048, 1.69),
            (0.022, 1.703),
            (0.012, 1.712),
            (0.0, 1.716)
        ],
        at = (b.x, b.y, 0),
        material = m.feeder,
        segments = 14
    );
    fcyl!(
        F,
        _name("tube"),
        0.055,
        0.2,
        (x, 1.545, z),
        m.feeder_glass,
        verts = 14
    );
    fcyl!(
        F,
        _name("tube_seed"),
        0.048,
        0.17,
        (x, 1.53, z),
        m.seed,
        verts = 12
    );
    // feeding ports, each with a peg to stand on
    for (y, a) in [(1.6, 0.0), (1.5, 90.0)] {
        let (c, s) = (cosd(a), sind(a));
        for k in [-1.0, 1.0] {
            fsweep!(
                F,
                _name("port"),
                &[
                    (x + k * 0.05 * c, y, z + k * 0.05 * s),
                    (x + k * 0.062 * c, y, z + k * 0.062 * s)
                ],
                0.012,
                m.feeder,
                verts = 8
            );
            fsweep!(
                F,
                _name("peg"),
                &[
                    (x + k * 0.05 * c, y - 0.022, z + k * 0.05 * s),
                    (x + k * 0.1 * c, y - 0.022, z + k * 0.1 * s)
                ],
                0.0025,
                m.iron,
                verts = 4
            );
        }
    }
    revolve!(
        _name("tray"),
        &[
            (0.0, 1.433),
            (0.09, 1.433),
            (0.094, 1.447),
            (0.088, 1.452),
            (0.083, 1.441),
            (0.06, 1.441),
            (0.06, 1.465),
            (0.0, 1.465)
        ],
        at = (b.x, b.y, 0),
        material = m.feeder,
        segments = 16
    );
    torus!(
        _name("perch"),
        0.1,
        0.005,
        at = F.at(x, 1.47, z),
        material = m.iron,
        major_segments = 20,
        minor_segments = 4
    );
    for a in [0.0, 90.0, 180.0, 270.0] {
        fsweep!(
            F,
            _name("spoke"),
            &[
                (x + 0.06 * cosd(a), 1.452, z + 0.06 * sind(a)),
                (x + 0.1 * cosd(a), 1.47, z + 0.1 * sind(a))
            ],
            0.003,
            m.iron,
            verts = 4
        );
    }
}

#[allow(non_snake_case)]
pub fn birdFeederPole() -> Vec<Obj> {
    let m = _own("birdFeederPole");
    let pole = m.body;
    let still = new_parts(|| {
        let z = FEED_Z;
        // the pole: a spike into the lawn, a step bar to tread it in, an acorn finial
        sweep!(
            "pole",
            &[F.at(0, 0, z), F.at(0, 0.06, z), F.at(0, 1.86, z)],
            0.013,
            material = pole,
            verts = 10,
            scale = [0.4, 1.0, 1.0]
        );
        revolve!(
            "finial",
            &[
                (0.0, 0.0),
                (0.016, 0.0),
                (0.016, 0.008),
                (0.022, 0.016),
                (0.02, 0.034),
                (0.011, 0.05),
                (0.0, 0.058)
            ],
            at = F.at(0, 1.855, z),
            material = pole,
            segments = 10
        );
        fsweep!(
            F,
            "foot_bar",
            &fillet(
                &[
                    (-0.085, 0.14, z),
                    (-0.085, 0.18, z),
                    (0.085, 0.18, z),
                    (0.085, 0.14, z)
                ],
                0.015,
                3
            ),
            0.007,
            pole,
            verts = 6
        );
        fcyl!(
            F,
            "collar",
            0.02,
            0.03,
            (0, 1.735, z),
            pole,
            verts = 10,
            bevel = 0.004,
            bevel_segments = 1
        );
        for i in [-1, 1] {
            let s = i as f64;
            // each arm in one piece: out from the pole's collar, curling over and down into a hook
            let pts = [
                F.at(0, 1.735, z),
                F.at(s * 0.06, 1.79, z),
                F.at(s * 0.15, 1.835, z),
                F.at(s * 0.24, 1.85, z),
                F.at(s * 0.3, 1.835, z),
                F.at(s * 0.322, 1.8, z),
                F.at(s * 0.32, 1.775, z),
                F.at(s * 0.305, 1.765, z),
            ];
            sweep!(
                format!("arm{i}"),
                &pts,
                0.009,
                material = pole,
                verts = 6,
                resolution = 3,
                scale = [1.2, 1.0, 1.0, 1.0, 1.0, 1.0, 0.9, 0.8]
            );
            // a scroll under it, stiffening the bend
            sweep!(
                format!("scroll{i}"),
                &[
                    F.at(0, 1.68, z),
                    F.at(s * 0.05, 1.7, z),
                    F.at(s * 0.1, 1.76, z),
                    F.at(s * 0.085, 1.79, z),
                    F.at(s * 0.07, 1.775, z)
                ],
                0.005,
                material = pole,
                verts = 5,
                resolution = 2
            );
            _feeder(s * 0.32);
        }
        // the robin, always there
        songbird!(
            &F,
            "robin",
            0.32,
            1.475,
            FEED_Z + 0.1,
            m.robin,
            m.robin_breast,
            m.beak,
            eye = m.dark,
            turn = -20,
            size = 0.8
        );
    });
    join(&still, "feeders", Some(p3(0, 0, 0)));
    // the blue tit that comes to the other feeder's perch, built full size and scaled to nothing
    let bird = new_parts(|| {
        songbird!(
            &F,
            "tit",
            -0.32,
            1.475,
            FEED_Z + 0.1,
            m.tit,
            m.tit_breast,
            m.beak,
            eye = m.dark,
            turn = 25,
            size = 0.75
        );
    });
    let bird = join(&bird, "bird", Some(F.at(-0.32, 1.47, FEED_Z + 0.1)));
    bird.set_scale((0.001, 0.001, 0.001));
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    squash(
        bird,
        "bird",
        1000,
        Squash {
            seconds: 0.5,
            ..Squash::default()
        },
    );
    meshes()
}
