//! The cast and carved stone: the bird bath, the fountain and the lantern.

use super::common::{_hex, _petals, _scalloped};
use super::{F, M};
use crate::songbird;
use artkit::math::{PI, TAU, cos, cosd, p3, radians, sin, sind};
use artkit::motion::{Squash, squash};
use artkit::parts::{join, join_all, new_parts, settle};
use artkit::scene::{Obj, meshes};
use artkit::{cone, cylinder, extrude, loft, revolve, ring, sphere, sweep, torus};

/// A cast-stone bird bath: a moulded round plinth, a fluted column under a turned capital, a shallow bowl whose
/// rim waves in twelve scallops, water in it, a robin dipping its beak at the rim.
pub fn birdbath() -> Vec<Obj> {
    revolve!(
        "plinth",
        &[
            (0.0, 0.0),
            (0.19, 0.0),
            (0.19, 0.02),
            (0.18, 0.03),
            (0.17, 0.032),
            (0.15, 0.05),
            (0.12, 0.062),
            (0.11, 0.075),
            (0.0, 0.075)
        ],
        material = M.stone_dark,
        segments = 24
    );
    revolve!(
        "column_foot",
        &[
            (0.0, 0.07),
            (0.105, 0.07),
            (0.108, 0.085),
            (0.096, 0.1),
            (0.085, 0.105),
            (0.0, 0.105)
        ],
        material = M.stone,
        segments = 20
    );
    let flutes: Vec<(f64, f64)> = (0..16)
        .map(|k| {
            let r = if k % 2 == 0 { 0.078 } else { 0.07 };
            (r * cos(PI * k as f64 / 8.0), r * sin(PI * k as f64 / 8.0))
        })
        .collect();
    sweep!(
        "column",
        &[(0.0, 0.0, 0.1), (0.0, 0.0, 0.36), (0.0, 0.0, 0.56)],
        0.0,
        material = M.stone,
        profile = flutes,
        scale = [1.0, 0.86, 0.95],
        angle = 60
    );
    revolve!(
        "capital",
        &[
            (0.0, 0.55),
            (0.08, 0.55),
            (0.09, 0.565),
            (0.082, 0.58),
            (0.095, 0.6),
            (0.13, 0.625),
            (0.14, 0.64),
            (0.0, 0.64)
        ],
        material = M.stone,
        segments = 20
    );
    // the bowl: under it, out and up to a rolled, scalloped rim, then down inside to the water
    let rows = [
        (0.12, 0.62),
        (0.2, 0.635),
        (0.28, 0.66),
        (0.33, 0.72),
        (0.345, 0.79),
        (0.34, 0.82),
        (0.325, 0.832),
        (0.305, 0.828),
        (0.29, 0.81),
        (0.27, 0.77),
        (0.2, 0.75),
        (0.1, 0.745),
    ];
    loft!(
        "bowl",
        &_scalloped(&rows, 12.0, 0.025, 48, 0.7),
        material = M.stone,
        angle = 70
    );
    cylinder!(
        "water",
        0.268,
        0.012,
        at = (0, 0, 0.792),
        material = M.water,
        verts = 40
    );
    torus!(
        "ripple",
        0.06,
        0.003,
        at = (-0.185, 0.175, 0.799),
        material = M.water,
        major_segments = 16,
        minor_segments = 3
    );
    // a robin on the rim at the back, turned in to drink
    songbird!(
        &F,
        "robin",
        0.24,
        0.832,
        -0.225,
        M.bird_brown,
        M.bird_breast,
        M.beak,
        eye = M.eye,
        turn = -47,
        head = 35
    );
    settle(&[join_all("birdbath")])
}

/// A two-tier stone fountain: an octagonal step, a round basin with a moulded rim, a turned stem with four
/// spouts pouring into it, an upper bowl with a scalloped lip spilling over in four streams, a tiered finial. The
/// water moves all the time near the camera (`flow`, ambient): the jet bubbling up, rings on the basins.
pub fn fountain() -> Vec<Obj> {
    revolve!(
        "plinth",
        &[
            (0.0, 0.0),
            (0.47, 0.0),
            (0.47, 0.035),
            (0.455, 0.05),
            (0.42, 0.06),
            (0.41, 0.09),
            (0.0, 0.09)
        ],
        material = M.stone_dark,
        segments = 8,
        smooth = false
    );
    revolve!(
        "basin",
        &[
            (0.34, 0.09),
            (0.43, 0.09),
            (0.44, 0.105),
            (0.43, 0.12),
            (0.445, 0.27),
            (0.465, 0.285),
            (0.465, 0.31),
            (0.45, 0.335),
            (0.42, 0.35),
            (0.385, 0.35),
            (0.37, 0.33),
            (0.375, 0.3),
            (0.39, 0.12),
            (0.34, 0.11)
        ],
        material = M.stone,
        segments = 32
    );
    cylinder!(
        "water",
        0.385,
        0.014,
        at = (0, 0, 0.29),
        material = M.water,
        verts = 32
    );
    revolve!(
        "stem",
        &[
            (0.0, 0.28),
            (0.13, 0.29),
            (0.135, 0.31),
            (0.115, 0.33),
            (0.1, 0.38),
            (0.092, 0.45),
            (0.1, 0.5),
            (0.115, 0.52),
            (0.105, 0.535),
            (0.13, 0.56),
            (0.2, 0.6),
            (0.21, 0.625),
            (0.0, 0.625)
        ],
        material = M.stone,
        segments = 24
    );
    // four spouts round the stem, each pouring an arc of water down into the basin
    for k in 0..4 {
        let a = radians((45 + 90 * k) as f64);
        let (c, s) = (cos(a), sin(a));
        sweep!(
            format!("spout{k}"),
            &[
                (0.08 * c, 0.08 * s, 0.47),
                (0.115 * c, 0.115 * s, 0.47),
                (0.13 * c, 0.13 * s, 0.462)
            ],
            0.014,
            material = M.stone,
            verts = 8,
            scale = [1.3, 1.0, 0.8]
        );
        sweep!(
            format!("stream{k}"),
            &[
                (0.135 * c, 0.135 * s, 0.458),
                (0.17 * c, 0.17 * s, 0.44),
                (0.2 * c, 0.2 * s, 0.38),
                (0.215 * c, 0.215 * s, 0.296)
            ],
            0.007,
            material = M.water,
            verts = 6,
            resolution = 3,
            scale = [1.0, 1.1, 1.2, 1.5]
        );
    }
    // the upper bowl: out and up to a scalloped lip, down inside to its water; four streams spill over the lip
    let rows = [
        (0.12, 0.6),
        (0.19, 0.625),
        (0.235, 0.67),
        (0.25, 0.715),
        (0.245, 0.745),
        (0.225, 0.752),
        (0.205, 0.735),
        (0.18, 0.7),
        (0.08, 0.69),
    ];
    loft!(
        "bowl2",
        &_scalloped(&rows, 8.0, 0.04, 32, 0.66),
        material = M.stone,
        angle = 70
    );
    cylinder!(
        "water2",
        0.195,
        0.012,
        at = (0, 0, 0.73),
        material = M.water,
        verts = 32
    );
    for k in 0..4 {
        let a = TAU * k as f64 / 4.0;
        let (c, s) = (cos(a), sin(a));
        sweep!(
            format!("spill{k}"),
            &[
                (0.225 * c, 0.225 * s, 0.738),
                (0.262 * c, 0.262 * s, 0.742),
                (0.285 * c, 0.285 * s, 0.7),
                (0.31 * c, 0.31 * s, 0.5),
                (0.325 * c, 0.325 * s, 0.296)
            ],
            0.0,
            material = M.water,
            profile = [
                (-0.004, -0.02),
                (0.004, -0.02),
                (0.004, 0.02),
                (-0.004, 0.02)
            ],
            up = (0, 0, 1),
            resolution = 3,
            scale = [1.0, 1.0, 0.8, 0.65, 0.9]
        );
    }
    // the finial: a turned spindle in tiers, a knob on top (the jet comes up out of it)
    revolve!(
        "finial",
        &[
            (0.0, 0.735),
            (0.07, 0.74),
            (0.072, 0.758),
            (0.05, 0.775),
            (0.045, 0.81),
            (0.062, 0.83),
            (0.06, 0.845),
            (0.04, 0.86),
            (0.036, 0.895),
            (0.045, 0.905),
            (0.03, 0.915),
            (0.0, 0.92)
        ],
        material = M.stone,
        segments = 16
    );
    sphere!(
        "knob",
        0.05,
        at = (0, 0, 0.945),
        material = M.stone,
        segments = 14,
        rings = 8
    );
    let body = join(&meshes(), "fountain", Some(p3(0, 0, 0)));
    // The water moves all the time (`flow`, ambient): the jet bubbles up from the knob and falls back, and a ring
    // on each basin's water swells out from where the water falls and settles back.
    let jet = new_parts(|| {
        let sections: Vec<_> = [
            (0.97, 0.022),
            (1.0, 0.014),
            (1.06, 0.012),
            (1.085, 0.02),
            (1.11, 0.03),
            (1.13, 0.024),
            (1.142, 0.008),
        ]
        .iter()
        .map(|&(z, r)| ring!(r, r, z = z, verts = 10))
        .collect();
        loft!("jet", &sections, material = M.water);
    });
    let jet = join(&jet, "jet", Some(p3(0, 0, 0.955)));
    let rings = new_parts(|| {
        torus!(
            "ripple2",
            0.1,
            0.006,
            at = (0, 0, 0.742),
            material = M.water,
            major_segments = 24,
            minor_segments = 4
        );
        torus!(
            "ripple",
            0.24,
            0.007,
            at = (0, 0, 0.298),
            material = M.water,
            major_segments = 32,
            minor_segments = 4
        );
    });
    let rings = join(&rings, "ripples", Some(p3(0, 0, 0.3)));
    let objs = settle(&[body, jet, rings]);
    let flow = Squash {
        seconds: 1.2,
        back: true,
        ..Squash::default()
    };
    squash(jet, "flow", (1, 1, 1.35), flow.clone());
    squash(rings, "flow", (1.4, 1.4, 1), flow);
    objs
}

/// A Japanese stone lantern (tōrō): a hexagonal footing under a ring of lotus petals, a round post with a band, a
/// platform on petals, the fire box between six posts (a moon window in four sides, open to the front and back,
/// the warm light inside), a roof with upturned corners curling up, and the jewel on a lotus cup.
pub fn lantern() -> Vec<Obj> {
    let hex = |r: f64, z: f64| _hex(r, z, 6, None, 0.0, 0.0);
    loft!(
        "base",
        &[
            hex(0.23, 0.0),
            hex(0.23, 0.06),
            hex(0.21, 0.085),
            hex(0.17, 0.095)
        ],
        material = M.stone_dark,
        smooth = false
    );
    loft!(
        "base_petals",
        &_petals(0.165, 0.11, 0.09, 0.14, 12, 0.12),
        material = M.stone_dark,
        angle = 60
    );
    revolve!(
        "column",
        &[
            (0.0, 0.13),
            (0.085, 0.13),
            (0.08, 0.15),
            (0.072, 0.28),
            (0.082, 0.29),
            (0.082, 0.31),
            (0.072, 0.32),
            (0.068, 0.48),
            (0.075, 0.5),
            (0.0, 0.5)
        ],
        material = M.stone,
        segments = 16
    );
    loft!(
        "platform_petals",
        &_petals(0.09, 0.17, 0.48, 0.535, 12, 0.12),
        material = M.stone,
        angle = 60
    );
    loft!(
        "platform",
        &[
            hex(0.16, 0.53),
            hex(0.2, 0.545),
            hex(0.2, 0.57),
            hex(0.15, 0.575)
        ],
        material = M.stone,
        smooth = false
    );
    // the fire box: the light inside, six corner posts, side panels pierced by moon windows, open front and back
    loft!(
        "firebox",
        &[hex(0.11, 0.575), hex(0.11, 0.77)],
        material = M.glow,
        smooth = false
    );
    for i in 0..6 {
        let a = (60 * i) as f64;
        let r = 0.135;
        cone!(
            format!("post{i}"),
            0.026,
            0.024,
            0.2,
            at = (r * cosd(a), r * sind(a), 0.67),
            material = M.stone,
            verts = 4,
            smooth = false
        );
    }
    for i in 0..6 {
        let mid = 60 * i + 30;
        if mid == 90 || mid == 270 {
            // the front and back stay open
            continue;
        }
        let hole: Vec<(f64, f64)> = (0..12)
            .map(|k| {
                (
                    0.034 * cos(TAU * k as f64 / 12.0),
                    0.01 + 0.034 * sin(TAU * k as f64 / 12.0),
                )
            })
            .collect();
        let panel = extrude!(
            format!("panel{i}"),
            &[
                (-0.055, -0.09),
                (0.055, -0.09),
                (0.055, 0.09),
                (-0.055, 0.09)
            ],
            0.018,
            material = M.stone,
            holes = [hole]
        );
        panel.set_rotation((0, 0, mid - 90 + 180));
        let mid = mid as f64;
        panel.set_location((0.122 * cosd(mid), 0.122 * sind(mid), 0.67));
    }
    loft!(
        "firebox_top",
        &[hex(0.15, 0.77), hex(0.16, 0.785)],
        material = M.stone,
        smooth = false
    );
    // the roof: a hexagon whose corners turn up at the eaves, sweeping in and up to its top, a curl at each corner
    loft!(
        "roof",
        &[
            _hex(0.15, 0.78, 12, Some(0.13), 0.0, 0.0),
            _hex(0.258, 0.81, 12, Some(0.224), 0.022, 0.0),
            _hex(0.27, 0.835, 12, Some(0.234), 0.03, 0.0),
            _hex(0.2, 0.872, 12, Some(0.18), 0.012, 0.0),
            _hex(0.12, 0.915, 12, Some(0.105), 0.0, 0.0),
            _hex(0.06, 0.945, 12, Some(0.052), 0.0, 0.0)
        ],
        material = M.stone,
        angle = 55
    );
    for i in 0..6 {
        let (c, s) = (cosd((60 * i) as f64), sind((60 * i) as f64));
        sweep!(
            format!("curl{i}"),
            &[
                (0.255 * c, 0.255 * s, 0.86),
                (0.272 * c, 0.272 * s, 0.876),
                (0.268 * c, 0.268 * s, 0.9),
                (0.252 * c, 0.252 * s, 0.9),
                (0.252 * c, 0.252 * s, 0.885)
            ],
            0.011,
            material = M.stone,
            verts = 6,
            resolution = 2,
            scale = [1.2, 1.1, 1.0, 0.9, 0.8]
        );
    }
    revolve!(
        "cup",
        &[
            (0.0, 0.94),
            (0.045, 0.94),
            (0.05, 0.952),
            (0.062, 0.962),
            (0.058, 0.972),
            (0.035, 0.97),
            (0.0, 0.97)
        ],
        material = M.stone,
        segments = 12
    );
    revolve!(
        "jewel",
        &[
            (0.0, 0.96),
            (0.03, 0.965),
            (0.042, 0.985),
            (0.04, 1.005),
            (0.028, 1.02),
            (0.012, 1.03),
            (0.0, 1.035)
        ],
        material = M.stone,
        segments = 12
    );
    settle(&[join_all("lantern")])
}
