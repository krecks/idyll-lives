//! The birdhouse and the bird feeders.

use super::common::{F, M, own, prism};
use artkit::forms::rounded_rect;
use artkit::kit::{self, fillet};
use artkit::math::{TAU, cos, cosd, p3, sin, sind};
use artkit::motion::{Squash, squash};
use artkit::parts::{Settle, join, join_all, new_parts, numbered, settle_all, settle_with};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, revolve, sweep, torus};

// --- the birdhouse (1x1): a little cream house with a red roof on a post --------------------------------------
pub fn birdhouse() -> Vec<Obj> {
    let m = own("birdhouse");
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
            profile = rounded_rect(0.024, 0.03, 0.004, 1),
            up = (1, 0, 0)
        );
    }
    // the house: a gabled box, its ridge running front to back, clad in lapped boards down its sides
    prism(
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
        holes = [hole],
        bevel = 0.003
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
    kit::songbird!(
        F,
        "robin",
        0.0,
        1.607,
        0.142,
        m.robin,
        m.robin_breast,
        m.beak,
        m.dark,
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
    // the gabled box's arrises eased under the roof's overhang
    artkit::bevel::ease(artkit::scene::obj("house"), artkit::shapes::SHARP, 1);
    join_all("birdhouse");
    settle_all();
    meshes()
}

// --- the bird feeders (1x1): two seed tubes on a shepherd's hook; a robin on one, a blue tit lands for watching -
/// The hook's plane (the old part's).
const FEED_Z: f64 = -0.2;

/// A seed tube hanging from the hook's end at `x`: a wire bail, a domed green lid, the glass tube full of seed
/// with two feeding ports and their pegs, a base and a seed tray with a lip, a perch ring on four spokes.
fn feeder(x: f64) {
    let m = M();
    let z = FEED_Z;
    let b = F.at(x, 0, z);
    // the bail: a loop over the hook's tip, down to the lid's knob
    fsweep!(
        F,
        numbered("hanger"),
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
        numbered("cap"),
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
        numbered("tube"),
        0.055,
        0.2,
        (x, 1.545, z),
        m.feeder_glass,
        verts = 14
    );
    fcyl!(
        F,
        numbered("tube_seed"),
        0.048,
        0.17,
        (x, 1.53, z),
        m.seed,
        verts = 12,
        bevel = 0.004
    );
    // feeding ports, each with a peg to stand on
    for (y, a) in [(1.6, 0.0), (1.5, 90.0)] {
        let (c, s) = (cosd(a), sind(a));
        for k in [-1.0, 1.0] {
            fsweep!(
                F,
                numbered("port"),
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
                numbered("peg"),
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
        numbered("tray"),
        &[
            (0.0, 1.433),
            (0.09, 1.433),
            // (a round lip, and the hub's top eased: no turn of 60°)
            (0.094, 1.447),
            (0.0924, 1.4509),
            (0.0885, 1.4525),
            (0.0846, 1.4509),
            (0.083, 1.447),
            (0.083, 1.441),
            (0.06, 1.441),
            (0.06, 1.461),
            (0.056, 1.465),
            (0.0, 1.465)
        ],
        at = (b.x, b.y, 0),
        material = m.feeder,
        segments = 16
    );
    torus!(
        numbered("perch"),
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
            numbered("spoke"),
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

pub fn bird_feeder_pole() -> Vec<Obj> {
    let m = own("birdFeederPole");
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
            feeder(s * 0.32);
        }
        // the robin, always there
        kit::songbird!(
            F,
            "robin",
            0.32,
            1.475,
            FEED_Z + 0.1,
            m.robin,
            m.robin_breast,
            m.beak,
            m.dark,
            turn = -20,
            size = 0.8
        );
    });
    join(&still, "feeders", Some(p3(0, 0, 0)));
    // the blue tit that comes to the other feeder's perch, built full size and scaled to nothing
    let bird = new_parts(|| {
        kit::songbird!(
            F,
            "tit",
            -0.32,
            1.475,
            FEED_Z + 0.1,
            m.tit,
            m.tit_breast,
            m.beak,
            m.dark,
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
