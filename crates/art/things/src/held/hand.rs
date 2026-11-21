//! The props in hand (G-70): drawn in a resident's hands while they use a thing. Each is built as it lies or stands
//! on a table, in the game's axes, its origin where characters.ts holds it from (actions.ts `HAND_PROPS`): the middle
//! of its base for the cups and the bag, the butt of a cue, the middle of a grip for the rest. What is held along its
//! length (cutlery, a dart, a cue, a bat) points to the front (+depth).

use super::M;
use super::common::{F, done, loft};
use super::dishes::_bag_ring;
use super::tools::_along;
use artkit::forms::rounded_rect;
use artkit::kit::fillet;
use artkit::math::{PI, TAU, cos, degrees, radians, sin};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsphere, fsweep, jitter, sphere};

/// A glazed mug 8 cm across and 9.5 cm high, a blue band round it, its handle to the right (+x, the side of the
/// hand that holds it), coffee in it with a little crema.
pub fn mug() -> Vec<Obj> {
    lathe!(
        "mug",
        &[
            (0.0, 0.0),
            (0.037, 0.0),
            (0.0405, 0.004),
            (0.041, 0.06),
            (0.041, 0.072),
            (0.041, 0.092),
            (0.0395, 0.095),
            (0.0365, 0.0935),
            (0.036, 0.082),
            (0.03, 0.0822),
            (0.0, 0.0824)
        ],
        [
            M.mug, M.mug, M.mug, M.mug_band, M.mug, M.mug, M.mug, M.mug, M.coffee, M.crema
        ],
        segments = 18
    );
    let path = fillet(
        &[
            (0.038, 0.078, 0.0),
            (0.066, 0.074, 0.0),
            (0.066, 0.03, 0.0),
            (0.038, 0.024, 0.0),
        ],
        0.012,
        3,
    );
    fsweep!(
        F,
        "handle",
        &path,
        0.0,
        M.mug,
        profile = rounded_rect(0.007, 0.012, 0.003, 1)
    );
    done("mug")
}

/// A tumbler 7 cm across and 11 cm high of something orange and cold, two cubes of ice at the top.
pub fn glass() -> Vec<Obj> {
    lathe!(
        "glass",
        &[
            (0.0, 0.0),
            (0.03, 0.0),
            (0.0315, 0.006),
            (0.0325, 0.012),
            (0.035, 0.09),
            (0.0355, 0.11),
            (0.0335, 0.1105),
            (0.033, 0.092),
            (0.0, 0.092)
        ],
        [
            M.glass, M.glass, M.glass, M.juice, M.glass, M.glass, M.glass, M.juice
        ],
        segments = 18
    );
    for (i, (x, z, yaw)) in [(-0.01, 0.006, 20.0), (0.012, -0.008, 65.0)]
        .into_iter()
        .enumerate()
    {
        box_!(
            format!("ice{i}"),
            0.016,
            0.012,
            0.016,
            (x, 0.094, z),
            M.ice,
            yaw = yaw,
            tilt = (8.0, -6.0),
            bevel = 0.002
        );
    }
    done("glass")
}

/// A stemmed glass of red wine, 19 cm high: a round foot, a stem, a bowl filled to its widest (the wine seen
/// through the glass).
pub fn wine_glass() -> Vec<Obj> {
    let mut mats = vec![M.glass; 6];
    mats.extend([
        M.wine, M.wine, M.glass, M.glass, M.glass, M.glass, M.glass, M.wine,
    ]);
    lathe!(
        "glass",
        &[
            (0.0, 0.0),
            (0.034, 0.0),
            (0.0345, 0.003),
            (0.008, 0.007),
            (0.004, 0.02),
            (0.004, 0.078),
            (0.012, 0.088),
            (0.034, 0.112),
            (0.0415, 0.14),
            (0.039, 0.172),
            (0.036, 0.19),
            (0.0345, 0.19),
            (0.0375, 0.172),
            (0.0395, 0.142),
            (0.0, 0.142)
        ],
        mats,
        segments = 18
    );
    done("wineGlass")
}

/// A 330 ml can of soda, 6.6 cm across and 12 cm high: red with a white band, its top and bottom pressed in,
/// a ring pull.
pub fn can() -> Vec<Obj> {
    lathe!(
        "can",
        &[
            (0.0, 0.003),
            (0.024, 0.0),
            (0.031, 0.004),
            (0.033, 0.012),
            (0.033, 0.05),
            (0.033, 0.075),
            (0.033, 0.108),
            (0.0285, 0.118),
            (0.027, 0.1205),
            (0.0255, 0.1195),
            (0.0245, 0.117),
            (0.0, 0.117)
        ],
        [
            M.can_top, M.can_top, M.can, M.can, M.can_band, M.can, M.can_top, M.can_top, M.can_top,
            M.can_top, M.can_top
        ],
        segments = 18
    );
    fextrude!(
        F,
        "pull",
        &[
            (-0.006, -0.004),
            (0.006, -0.004),
            (0.008, 0.008),
            (0.0, 0.014),
            (-0.008, 0.008)
        ],
        0.0012,
        (0.0, 0.1178, 0.006),
        M.can_top,
        plane = Facing::Top,
        holes = vec![vec![(-0.003, 0.004), (0.003, 0.004), (0.0, 0.009)]]
    );
    done("can")
}

/// A fork's head outline (u along it, v across) from its neck (u 0, `neck` each side) to the tips (u `length`),
/// `n` tines across `half` each side, split down to `step`.
fn _tines(n: usize, half: f64, length: f64, step: f64, neck: f64) -> Vec<(f64, f64)> {
    let gap = 2.0 * half / (2 * n - 1) as f64;
    let mut pts = vec![(0.0, -neck), (0.014, -half)];
    for k in 0..n {
        let a = -half + (2 * k) as f64 * gap;
        pts.extend([(length, a), (length, a + gap)]);
        if k < n - 1 {
            pts.extend([(step, a + gap), (step, a + 2.0 * gap)]);
        }
    }
    pts.extend([(0.014, half), (0.0, neck)]);
    pts
}

/// A polished dinner fork, 20 cm long, lying flat, its tines to the front (+depth) and held at the origin, 7.7 cm
/// from the handle's end: a handle widening to 1.5 cm at its end, 3 mm thick, a neck, four tines 2.6 cm across. A
/// dinner fork's size, not a dessert fork's, and polished: it is seen from the game camera.
pub fn fork() -> Vec<Obj> {
    let handle = [
        (-0.15, 0.0),
        (-0.145, -0.0075),
        (-0.05, -0.0048),
        (0.0, -0.003),
        (0.0, 0.003),
        (-0.05, 0.0048),
        (-0.145, 0.0075),
    ];
    _along("handle", &handle, 0.003, 0.0, 0.073, M.fork);
    _along(
        "head",
        &_tines(4, 0.013, 0.05, 0.028, 0.003),
        0.0026,
        0.0002,
        0.073,
        M.fork,
    );
    done("fork")
}

/// A steel dessert spoon, 17 cm long, lying flat, its bowl to the front (+depth) and held at the origin, 6 cm
/// from the handle's end.
pub fn spoon() -> Vec<Obj> {
    let handle = [
        (-0.12, 0.0),
        (-0.115, -0.0062),
        (-0.05, -0.0045),
        (0.0, -0.0028),
        (0.0, 0.0028),
        (-0.05, 0.0045),
        (-0.115, 0.0062),
    ];
    _along("handle", &handle, 0.0024, 0.0, 0.06, M.steel);
    sphere!(
        "bowl",
        1.0,
        at = F.at(0.0, 0.0055, 0.082),
        material = M.steel,
        scale = (0.017, 0.025, 0.0055),
        segments = 10,
        rings = 6
    );
    done("spoon")
}

/// The ice cream cart's flat wooden spoon, 9.5 cm, lying flat, its rounded blade to the front (+depth), held at the
/// origin 3 cm from its back end.
pub fn paddle() -> Vec<Obj> {
    _along(
        "paddle",
        &[
            (-0.03, -0.004),
            (0.035, -0.0055),
            (0.055, -0.0075),
            (0.065, -0.006),
            (0.065, 0.006),
            (0.055, 0.0075),
            (0.035, 0.0055),
            (-0.03, 0.004),
        ],
        0.0018,
        0.0,
        0.0,
        M.paddle,
    );
    done("paddle")
}

/// A pair of tapering wooden chopsticks, 23 cm long, side by side, their tips to the front (+depth), held from
/// the origin.
pub fn chopsticks() -> Vec<Obj> {
    for (i, dx) in [-0.006, 0.006].into_iter().enumerate() {
        fsweep!(
            F,
            format!("chopstick{i}"),
            &[(dx * 1.6, 0.0035, -0.06), (dx * 0.5, 0.0035, 0.17)],
            0.0,
            M.chopstick,
            profile = [
                (-0.0035, -0.0035),
                (0.0035, -0.0035),
                (0.0035, 0.0035),
                (-0.0035, 0.0035)
            ],
            scale = [1.0, 0.5]
        );
    }
    done("chopsticks")
}

/// A smartphone 7.2 by 14.8 cm and 8 mm thick, lying on its back, its screen up and its top to the front
/// (+depth): a dark rounded body, the glass, a camera at the back.
pub fn phone() -> Vec<Obj> {
    fextrude!(
        F,
        "body",
        &rounded_rect(0.072, 0.148, 0.009, 3),
        0.0075,
        (0, 0.00375, 0),
        M.phone,
        plane = Facing::Top,
        bevel = 0.002,
        bevel_segments = 1
    );
    fextrude!(
        F,
        "screen",
        &rounded_rect(0.066, 0.14, 0.007, 3),
        0.0008,
        (0, 0.0078, 0),
        M.screen,
        plane = Facing::Top
    );
    fextrude!(
        F,
        "glow",
        &rounded_rect(0.058, 0.118, 0.003, 2),
        0.0004,
        (0, 0.0083, -0.002),
        M.screen_lit,
        plane = Facing::Top
    );
    fextrude!(
        F,
        "camera",
        &rounded_rect(0.022, 0.022, 0.005, 2),
        0.0016,
        (-0.018, -0.0006, 0.054),
        M.phone,
        plane = Facing::Top
    );
    done("phone")
}

/// An open cloth-bound book, 30 cm across and 22 cm from the foot of its pages to their head (+depth), lying on
/// its boards: the pages rising from the spine and curving down to the fore-edges, lines of type in grey bands.
pub fn book() -> Vec<Obj> {
    for s in [-1, 1] {
        let k = s as f64;
        box_!(
            format!("board{s}"),
            0.152,
            0.0032,
            0.224,
            (k * 0.076, 0.0016, 0.0),
            M.cover,
            tilt = (0.0, (-s * 3) as f64),
            bevel = 0.0008
        );
        let prof = [
            (0.0, 0.004),
            (0.002, 0.022),
            (0.03, 0.027),
            (0.08, 0.026),
            (0.125, 0.021),
            (0.146, 0.014),
            (0.148, 0.006),
            (0.146, 0.004),
        ];
        let pages: Vec<(f64, f64)> = prof.iter().map(|&(u, v)| (k * u, v)).collect();
        fextrude!(
            F,
            format!("pages{s}"),
            &pages,
            0.214,
            (0, 0, 0),
            M.page_edge,
            plane = Facing::Front
        );
        // the open page's face: a thin sheet over the block, the type on it
        let top: Vec<(f64, f64)> = prof[1..6]
            .iter()
            .map(|&(u, v)| (k * u, v + 0.0006))
            .collect();
        let sheet: Vec<(f64, f64, f64)> = top.iter().map(|&(x, y)| (x, y, 0.0)).collect();
        fsweep!(
            F,
            format!("sheet{s}"),
            &sheet,
            0.0,
            M.page,
            profile = [
                (-0.0003, -0.107),
                (0.0003, -0.107),
                (0.0003, 0.107),
                (-0.0003, 0.107)
            ],
            up = (0, 0, 1)
        );
        for t in 0..7 {
            let z = -0.075 + 0.025 * t as f64;
            let line: Vec<(f64, f64, f64)> =
                top[1..4].iter().map(|&(x, y)| (x, y + 0.0005, z)).collect();
            fsweep!(
                F,
                format!("type{s}{t}"),
                &line,
                0.0,
                M.print,
                profile = [
                    (-0.0003, -0.006),
                    (0.0003, -0.006),
                    (0.0003, 0.006),
                    (-0.0003, 0.006)
                ],
                up = (0, 0, 1)
            );
        }
    }
    done("book")
}

/// A coin for the fountain, 2.3 cm across: a milled rim and a raised face.
pub fn fountain_coin() -> Vec<Obj> {
    lathe!(
        "coin",
        &[
            (0.0, 0.0),
            (0.0115, 0.0),
            (0.0117, 0.0011),
            (0.0115, 0.0022),
            (0.0098, 0.0022),
            (0.0096, 0.0018),
            (0.0, 0.0019)
        ],
        M.brass,
        segments = 14
    );
    done("coin")
}

/// A small paper bag of bird seed, 8 cm wide and 12 cm high, its top rolled open over the seed.
pub fn seed_bag() -> Vec<Obj> {
    let rings = [
        _bag_ring(0.0, 0.04, 0.025, 0.012),
        _bag_ring(0.085, 0.041, 0.026, 0.012),
        _bag_ring(0.1, 0.043, 0.028, 0.006),
        _bag_ring(0.108, 0.04, 0.025, 0.004),
    ];
    loft("bag", &rings, M.kraft, 30.0);
    let mound = lathe!(
        "seed",
        &[(0.0, 0.0), (0.038, 0.0), (0.028, 0.007), (0.0, 0.011)],
        M.seed,
        at = (0, 0.1075, 0),
        segments = 10,
        scale = (1.0, 0.6)
    );
    jitter!(mound, 0.002, size = 0.006, seed = 3.1);
    done("seedBag")
}

/// A steel dart, 15 cm long, lying along the depth, its point to the front (+depth), held by the barrel at the
/// origin: a knurled barrel, a black shaft and three blue flights.
pub fn dart() -> Vec<Obj> {
    lathe!(
        "barrel",
        &[
            (0.0, -0.02),
            (0.0035, -0.019),
            (0.0042, -0.012),
            (0.0046, 0.0),
            (0.0042, 0.014),
            (0.0032, 0.022),
            (0.0011, 0.026),
            (0.0006, 0.06),
            (0.0, 0.064)
        ],
        vec![M.steel; 8],
        at = (0, 0.0046, 0),
        segments = 8,
        rot = (90, 0, 0),
        wave = (8, vec![(3, 0.06)])
    );
    fsweep!(
        F,
        "shaft",
        &[(0, 0.0046, -0.02), (0, 0.0046, -0.065)],
        0.0021,
        M.dart_shaft,
        verts = 6
    );
    for k in 0..3 {
        let a = radians((90 + 120 * k) as f64);
        fextrude!(
            F,
            format!("flight{k}"),
            &[
                (0.0, 0.0),
                (0.0, 0.004),
                (0.017, 0.016),
                (0.03, 0.016),
                (0.036, 0.0)
            ],
            0.0008,
            (0.0, 0.0046, -0.09),
            M.dart_flight,
            plane = Facing::Side,
            rot = (0.0, degrees(a), 0.0)
        );
    }
    done("dart")
}

const CUE: f64 = 1.47;

/// A pool cue, 1.47 m, lying along the depth from its butt at the origin to its tip at the front: a wrapped
/// butt, a joint, a tapering maple shaft, a white ferrule and a chalked tip (as the pool table's was).
pub fn cue() -> Vec<Obj> {
    let along = |t: f64| (0.0, 0.014, CUE * t);
    fsweep!(
        F,
        "cue_butt",
        &[along(0.0), along(0.3)],
        0.014,
        M.cue_butt,
        verts = 8,
        scale = [1.0, 0.86]
    );
    fsweep!(
        F,
        "cue_wrap",
        &[along(0.06), along(0.2)],
        0.0142,
        M.cue_wrap,
        verts = 8,
        scale = [1.0, 0.95]
    );
    fsweep!(
        F,
        "cue_joint",
        &[along(0.3), along(0.32)],
        0.0122,
        M.pearl,
        verts = 8
    );
    fsweep!(
        F,
        "cue_shaft",
        &[(0.0, 0.014, CUE * 0.32), (0.0, 0.014, CUE * 0.99)],
        0.0118,
        M.cue,
        verts = 8,
        scale = [1.0, 0.55]
    );
    fsweep!(
        F,
        "cue_ferrule",
        &[(0.0, 0.014, CUE * 0.99), (0.0, 0.014, CUE * 0.997)],
        0.0066,
        M.pearl,
        verts = 8
    );
    fsphere!(
        F,
        "cue_tip",
        0.0064,
        (0.0, 0.014, CUE * 0.997),
        M.cue_tip,
        segments = 8,
        rings = 4
    );
    done("cue")
}

/// A ping-pong bat lying on its back, its blade to the front (+depth) and its handle at the origin, where it is
/// held: a wooden blade 15 by 16 cm, red rubber on top and black underneath, a flared handle.
pub fn bat() -> Vec<Obj> {
    for (name, y, h, m) in [
        ("rubber_back", 0.0, 0.002, M.rubber_black),
        ("blade", 0.002, 0.006, M.blade),
        ("rubber", 0.008, 0.002, M.rubber),
    ] {
        disc!(name, 0.15, h, (0.0, y, 0.122), m, verts = 20, depth = 0.158);
    }
    fsweep!(
        F,
        "handle",
        &[(0.0, 0.005, 0.05), (0.0, 0.005, 0.0), (0.0, 0.005, -0.048)],
        0.0,
        M.blade,
        profile = rounded_rect(0.024, 0.016, 0.007, 2),
        up = (0, 0, 1),
        scale = [0.85, 1.0, 1.2]
    );
    done("bat")
}

/// A three-piece cocktail shaker, 22 cm high, the bar cart's (the dining things' `barCart`, hidden while it is in a
/// hand): a tapering chrome tin, the strainer and its cap.
pub fn shaker() -> Vec<Obj> {
    lathe!(
        "shaker",
        &[
            (0.0, 0.0),
            (0.034, 0.0),
            (0.036, 0.004),
            (0.04, 0.12),
            (0.042, 0.124),
            (0.037, 0.13),
            (0.03, 0.17),
            (0.032, 0.174),
            (0.018, 0.2),
            (0.012, 0.212),
            (0.014, 0.215),
            (0.0, 0.22)
        ],
        M.chrome,
        segments = 16
    );
    done("shaker")
}

/// A plastic watering can, 5 litres: a round body 15 cm across and 19 cm high, a hoop handle over the top from
/// back to front, held at its middle 26 cm up, and a spout rising forward (+depth) to a rose 30 cm out.
pub fn watering_can() -> Vec<Obj> {
    lathe!(
        "can_body",
        &[
            (0.0, 0.0),
            (0.07, 0.0),
            (0.075, 0.008),
            (0.076, 0.16),
            (0.07, 0.185),
            (0.045, 0.192),
            (0.0, 0.192)
        ],
        M.can_green,
        segments = 14
    );
    fsweep!(
        F,
        "can_handle",
        &[
            (0, 0.17, -0.06),
            (0, 0.24, -0.045),
            (0, 0.262, 0.0),
            (0, 0.24, 0.045),
            (0, 0.18, 0.06)
        ],
        0.011,
        M.can_green,
        verts = 6,
        resolution = 2
    );
    fsweep!(
        F,
        "can_spout",
        &[(0, 0.03, 0.065), (0, 0.11, 0.17), (0, 0.2, 0.27)],
        0.011,
        M.can_green,
        verts = 6,
        scale = [1.2, 1.0, 0.85]
    );
    lathe!(
        "can_rose",
        &[
            (0.0, 0.0),
            (0.012, 0.0),
            (0.028, 0.03),
            (0.028, 0.036),
            (0.0, 0.036)
        ],
        M.rose,
        at = (0, 0.2, 0.27),
        segments = 10,
        rot = (-55, 0, 0)
    );
    done("wateringCan")
}

/// A pair of bypass secateurs, 21 cm long, lying flat, closed, the blades to the front (+depth) and held at the
/// origin, the middle of the red grips: the curved cutting blade over the hooked anvil, the pivot bolt, the spring
/// between the grips.
pub fn secateurs() -> Vec<Obj> {
    for side in [-1, 1] {
        let k = side as f64;
        fsweep!(
            F,
            format!("grip{side}"),
            &[
                (k * 0.004, 0.009, -0.075),
                (k * 0.016, 0.009, -0.02),
                (k * 0.009, 0.009, 0.03)
            ],
            0.0,
            M.pruner,
            profile = rounded_rect(0.016, 0.012, 0.005, 1),
            up = (0, 1, 0),
            resolution = 2
        );
    }
    fextrude!(
        F,
        "blade",
        &[
            (-0.009, 0.03),
            (0.006, 0.03),
            (0.01, 0.06),
            (0.004, 0.095),
            (-0.002, 0.098),
            (-0.004, 0.07)
        ],
        0.0028,
        (0, 0.0135, 0),
        M.steel,
        plane = Facing::Top
    );
    fextrude!(
        F,
        "anvil",
        &[
            (0.009, 0.03),
            (-0.004, 0.03),
            (-0.01, 0.06),
            (-0.006, 0.085),
            (0.0, 0.08),
            (0.004, 0.06)
        ],
        0.004,
        (0, 0.007, 0),
        M.iron,
        plane = Facing::Top
    );
    disc!(
        "bolt",
        0.012,
        0.016,
        (0.0, 0.003, 0.034),
        M.steel,
        verts = 8
    );
    done("secateurs")
}

// The bunch's heads, as in the vase of flowers (the tabletop things' `VASE_HEADS`), standing: (x, height, depth,
// petal, kind, size, tip, yaw), in game axes. A hand takes the stems at the origin, so the bunch stands up out of
// the fist.
type Head = (f64, f64, f64, &'static str, &'static str, f64, f64, f64);
const BUNCH: [Head; 6] = [
    (0.0, 0.32, 0.030, "petal_white", "daisy", 0.026, 6.0, 0.0),
    (-0.046, 0.29, 0.050, "petal_rose", "tulip", 0.015, 0.0, 12.0),
    (
        0.043,
        0.272,
        0.020,
        "petal_yellow",
        "daisy",
        0.024,
        -5.0,
        0.0,
    ),
    (
        -0.018,
        0.334,
        0.060,
        "petal_blush",
        "tulip",
        0.014,
        0.0,
        -8.0,
    ),
    (0.026, 0.302, 0.045, "petal_white", "small", 0.017, 4.0, 0.0),
    (
        -0.048,
        0.252,
        0.015,
        "petal_rose",
        "daisy",
        0.019,
        -7.0,
        0.0,
    ),
];
const GRIP: (f64, f64, f64) = (0.0, 0.0, 0.02); // where the stems are gathered, the middle of the bunch's base

/// A flower's petals as a flat outline: `n` petals `r` out, `inner` of that between them (0.42: a daisy).
pub fn _star(r: f64, inner: f64, n: usize) -> Vec<(f64, f64)> {
    (0..2 * n)
        .map(|k| {
            let f = if k % 2 == 0 { 1.0 } else { inner };
            let a = PI * k as f64 / n as f64;
            (r * f * cos(a), r * f * sin(a))
        })
        .collect()
}

/// A bunch of cut flowers for a vase, 36 cm from the stems' cut ends to the tallest bloom, standing: the stems
/// gathered at the origin where a hand takes them and curving up to six heads (three daisies with an eye, two tulip
/// buds, a small daisy) and four leaves. The same heads as the vase of flowers has, so a bunch cut in the garden and
/// one bought arranged look alike (G-10, Q-18).
pub fn cut_flowers() -> Vec<Obj> {
    for (i, &(x, y, z, petal, kind, r, tip, yaw)) in BUNCH.iter().enumerate() {
        fsweep!(
            F,
            format!("stem{i}"),
            &[
                GRIP,
                (
                    x * 0.45,
                    GRIP.1 + (y - GRIP.1) * 0.55,
                    GRIP.2 + (z - GRIP.2) * 0.55
                ),
                (x, y - 0.004, z)
            ],
            0.0028,
            M.stem_green,
            verts = 4,
            resolution = 1
        );
        if kind == "tulip" {
            // (A closed bud: a cup swelling from the stem and closing over the top.)
            lathe!(
                format!("bud{i}"),
                &[
                    (0.0, 0.0),
                    (0.005, 0.001),
                    (0.012, 0.008),
                    (0.0145, 0.018),
                    (0.0135, 0.028),
                    (0.008, 0.036),
                    (0.0, 0.038)
                ],
                M.palette().get(petal),
                at = (x, y - 0.002, z),
                segments = 6,
                rot = (tip, 0.0, yaw)
            );
            continue;
        }
        fextrude!(
            F,
            format!("face{i}"),
            &_star(r, 0.42, 6),
            0.004,
            (x, y, z),
            M.palette().get(petal),
            plane = Facing::Front,
            rot = (tip, 0.0, yaw)
        );
        ball!(
            format!("eye{i}"),
            r * 0.34,
            (x, y + 0.003, z),
            M.flower_eye,
            scale = (1.0, 1.0, 0.5),
            seg = 6,
            rings = 3
        );
    }
    for (i, (x, y, z, yaw)) in [
        (0.012, 0.10, 0.02, 40.0),
        (-0.03, 0.07, 0.03, -30.0),
        (0.035, 0.16, 0.04, 20.0),
        (-0.014, 0.19, 0.05, -55.0),
    ]
    .into_iter()
    .enumerate()
    {
        leaf!(format!("leaf{i}"), (x, y, z), 0.05, 0.017, yaw, M.leaf);
    }
    done("cutFlowers")
}

/// A daisy chain, 16 cm across, coiled flat in the hand: a ring of plaited stems with eleven daisies round it,
/// heads up, and the two stem ends showing where the chain was closed (Q-18, the daisies' one real use left).
pub fn daisy_chain() -> Vec<Obj> {
    let r = 0.062;
    let chain: Vec<(f64, f64, f64)> = (0..25)
        .map(|k| {
            (
                r * cos(TAU * k as f64 / 24.0),
                0.004,
                r * sin(TAU * k as f64 / 24.0),
            )
        })
        .collect();
    fsweep!(
        F,
        "chain",
        &chain,
        0.0034,
        M.stem_green,
        verts = 5,
        resolution = 1
    );
    for k in 0..11 {
        let a = TAU * k as f64 / 11.0;
        let (x, z) = (r * cos(a), r * sin(a));
        fextrude!(
            F,
            format!("face{k}"),
            &_star(0.019, 0.42, 6),
            0.003,
            (x, 0.009, z),
            M.petal_white,
            plane = Facing::Top,
            rot = (0, 0, 18 * k)
        );
        ball!(
            format!("eye{k}"),
            0.0065,
            (x, 0.0115, z),
            M.flower_eye,
            scale = (1.0, 0.5, 1.0),
            seg = 6,
            rings = 3
        );
    }
    // the ends, plaited back under the last daisies
    for (i, (a, sgn)) in [(0.0, 1.0), (PI, -1.0)].into_iter().enumerate() {
        fsweep!(
            F,
            format!("end{i}"),
            &[
                (r * 0.92 * cos(a), 0.006, r * 0.92 * sin(a)),
                (r * 1.14 * cos(a), 0.004, r * 1.14 * sin(a)),
                (
                    r * 1.1 * cos(a + sgn * 0.35),
                    0.003,
                    r * 1.1 * sin(a + sgn * 0.35)
                )
            ],
            0.0026,
            M.stem_green,
            verts = 4,
            resolution = 1
        );
    }
    done("daisyChain")
}

/// An artist's brush, 24 cm long, lying along the depth, its bristles to the front (+depth), held at the origin
/// 6 cm behind the ferrule: a lacquered wooden handle swelling and tapering, a steel ferrule, bristles dipped in
/// blue.
pub fn paintbrush() -> Vec<Obj> {
    lathe!(
        "brush_handle",
        &[
            (0.0, -0.17),
            (0.0025, -0.168),
            (0.0042, -0.1),
            (0.0048, -0.04),
            (0.004, 0.0),
            (0.0036, 0.04),
            (0.0, 0.042)
        ],
        M.handle_wood,
        at = (0, 0.005, 0),
        segments = 8,
        rot = (90, 0, 0)
    );
    lathe!(
        "brush_ferrule",
        &[
            (0.0, 0.04),
            (0.0037, 0.04),
            (0.0042, 0.058),
            (0.0036, 0.062),
            (0.0, 0.062)
        ],
        M.steel,
        at = (0, 0.005, 0),
        segments = 8,
        rot = (90, 0, 0)
    );
    lathe!(
        "brush_tip",
        &[
            (0.0, 0.062),
            (0.0036, 0.062),
            (0.0042, 0.07),
            (0.0028, 0.078),
            (0.0002, 0.084),
            (0.0, 0.084)
        ],
        [
            M.bristle,
            M.bristle,
            M.paint_blue,
            M.paint_blue,
            M.paint_blue
        ],
        at = (0, 0.005, 0),
        segments = 8,
        rot = (90, 0, 0)
    );
    done("paintbrush")
}

/// A stick of white chalk, 8 cm long and 1 cm across, lying along the depth, held at its middle, its tip
/// worn down at the front.
pub fn chalk() -> Vec<Obj> {
    fsweep!(
        F,
        "chalk",
        &[(0, 0.005, -0.042), (0, 0.005, 0.03), (0, 0.005, 0.042)],
        0.005,
        M.chalk,
        verts = 8,
        scale = [1.0, 1.0, 0.55]
    );
    done("chalk")
}

/// A baby's bottle, 17 cm with its teat, standing: a clear body two thirds full of milk, a blue screw ring, a
/// honey-coloured teat.
pub fn baby_bottle() -> Vec<Obj> {
    lathe!(
        "bottle",
        &[
            (0.0, 0.0),
            (0.027, 0.0),
            (0.029, 0.006),
            (0.029, 0.08),
            (0.029, 0.11),
            (0.026, 0.118),
            (0.026, 0.122),
            (0.0, 0.122)
        ],
        [
            M.milk, M.milk, M.milk, M.bottle, M.bottle, M.bottle, M.bottle
        ],
        segments = 14
    );
    lathe!(
        "ring",
        &[
            (0.0, 0.118),
            (0.0285, 0.118),
            (0.0295, 0.124),
            (0.0285, 0.138),
            (0.012, 0.139),
            (0.0, 0.139)
        ],
        M.bottle_ring,
        segments = 14
    );
    lathe!(
        "teat",
        &[
            (0.0, 0.138),
            (0.012, 0.138),
            (0.011, 0.148),
            (0.006, 0.155),
            (0.0045, 0.165),
            (0.003, 0.17),
            (0.0, 0.171)
        ],
        M.teat,
        segments = 10
    );
    done("babyBottle")
}

/// A postcard, 14.8 by 10.5 cm, lying flat, picture up and its long side across, held at the middle of its back
/// (near) edge at the origin, the card reaching to the front: a seaside view in bands (sky, sea, sand).
pub fn postcard() -> Vec<Obj> {
    box_!("card", 0.148, 0.0008, 0.105, (0, 0.0004, 0.0525), M.card);
    for (name, z, d, m) in [
        ("sky", 0.078, 0.045, M.card_sky),
        ("sea", 0.047, 0.017, M.card_sea),
        ("sand", 0.026, 0.026, M.card_sand),
    ] {
        box_!(name, 0.136, 0.0004, d, (0, 0.001, z), m);
    }
    done("postcard")
}

/// The phone box's receiver, 21 cm, lying on its cups: a black bar along the depth, the earpiece's cup at the front
/// (+depth) and the mouthpiece's at the back, both turned down; held at the middle of its bar. Drawn on the box's hook
/// at rest by code (actions.ts `REST_PROPS`), in the caller's hand while they call.
pub fn receiver() -> Vec<Obj> {
    fsweep!(
        F,
        "handset_bar",
        &[(0, 0.04, -0.075), (0, 0.048, 0.0), (0, 0.04, 0.075)],
        0.0,
        M.handset,
        profile = rounded_rect(0.036, 0.022, 0.008, 1),
        up = (0, 0, 1),
        resolution = 2
    );
    for (name, z) in [("earpiece", 0.08), ("mouthpiece", -0.08)] {
        lathe!(
            name,
            &[
                (0.0, 0.0),
                (0.024, 0.0),
                (0.026, 0.006),
                (0.022, 0.03),
                (0.012, 0.042),
                (0.0, 0.044)
            ],
            M.handset,
            at = (0, 0, z),
            segments = 12
        );
    }
    done("receiver")
}

/// A ballpoint pen, 14 cm, lying along the depth, its point to the front (+depth), held at the origin 3.5 cm behind
/// the point: a navy barrel, a chrome cone and point, a clip at its back end.
pub fn pen() -> Vec<Obj> {
    lathe!(
        "pen",
        &[
            (0.0, -0.105),
            (0.0042, -0.104),
            (0.0045, -0.09),
            (0.0045, 0.0),
            (0.0040, 0.012),
            (0.0, 0.013)
        ],
        M.pen,
        at = (0, 0.0045, 0),
        segments = 8,
        rot = (90, 0, 0)
    );
    lathe!(
        "pen_tip",
        &[
            (0.0, 0.012),
            (0.004, 0.012),
            (0.0012, 0.032),
            (0.0004, 0.035),
            (0.0, 0.0352)
        ],
        M.chrome,
        at = (0, 0.0045, 0),
        segments = 8,
        rot = (90, 0, 0)
    );
    fbox!(
        F,
        "pen_clip",
        0.0018,
        0.002,
        0.045,
        (0, 0.0095, -0.08),
        M.chrome
    );
    done("pen")
}

/// A yellow pencil, 17 cm, six-sided, lying along the depth, sharpened to the front (+depth), held at the origin
/// 3.5 cm behind the point: the painted shaft, the bare wood cone, the lead.
pub fn pencil() -> Vec<Obj> {
    lathe!(
        "pencil",
        &[(0.0, -0.135), (0.0038, -0.135), (0.0038, 0.0), (0.0, 0.0)],
        M.pencil,
        at = (0, 0.0038, 0),
        segments = 6,
        rot = (90, 0, 0)
    );
    lathe!(
        "pencil_cone",
        &[(0.0, 0.0), (0.0038, 0.0), (0.0012, 0.026), (0.0, 0.026)],
        M.pencil_wood,
        at = (0, 0.0038, 0),
        segments = 6,
        rot = (90, 0, 0)
    );
    lathe!(
        "pencil_lead",
        &[(0.0, 0.026), (0.0012, 0.026), (0.0, 0.035)],
        M.graphite,
        at = (0, 0.0038, 0),
        segments = 6,
        rot = (90, 0, 0)
    );
    done("pencil")
}

/// A whiteboard marker, 13.5 cm and 1.8 cm across, lying along the depth, its felt tip to the front (+depth),
/// held at the origin 4 cm behind it, its blue cap pushed onto the back end.
pub fn marker() -> Vec<Obj> {
    lathe!(
        "marker",
        &[
            (0.0, -0.1),
            (0.009, -0.1),
            (0.009, 0.0),
            (0.006, 0.012),
            (0.0, 0.012)
        ],
        M.marker,
        at = (0, 0.009, 0),
        segments = 10,
        rot = (90, 0, 0)
    );
    lathe!(
        "marker_cap",
        &[
            (0.0, -0.12),
            (0.0095, -0.12),
            (0.0095, -0.072),
            (0.0, -0.072)
        ],
        M.marker_cap,
        at = (0, 0.009, 0),
        segments = 10,
        rot = (90, 0, 0)
    );
    lathe!(
        "marker_tip",
        &[
            (0.0, 0.012),
            (0.0032, 0.012),
            (0.0032, 0.028),
            (0.002, 0.034),
            (0.0, 0.035)
        ],
        M.marker_cap,
        at = (0, 0.009, 0),
        segments = 8,
        rot = (90, 0, 0)
    );
    done("marker")
}

/// A basketball, 24 cm across, standing on the table: pebbled orange leather and the black seams, one round its
/// middle, one over the top and two curving either side.
pub fn basketball() -> Vec<Obj> {
    ball!("ball", 0.12, (0, 0.1254, 0), M.ball, seg = 16, rings = 10);
    let round: Vec<(f64, f64, f64)> = (0..360)
        .step_by(30)
        .map(|a| {
            (
                0.1232 * cos(radians(a as f64)),
                0.1254,
                0.1232 * sin(radians(a as f64)),
            )
        })
        .collect();
    fsweep!(
        F,
        "seam_round",
        &round,
        0.0026,
        M.ball_seam,
        verts = 4,
        closed = true
    );
    let over: Vec<(f64, f64, f64)> = (0..360)
        .step_by(30)
        .map(|a| {
            (
                0.0,
                0.1254 + 0.1232 * sin(radians(a as f64)),
                0.1232 * cos(radians(a as f64)),
            )
        })
        .collect();
    fsweep!(
        F,
        "seam_over",
        &over,
        0.0026,
        M.ball_seam,
        verts = 4,
        closed = true
    );
    for side in [-1, 1] {
        let mut pts = Vec::new();
        for a in (0..360).step_by(30) {
            let r = radians(a as f64);
            // a ring cut 7 cm in from the side, bowed in the middle (the ball's curved seams)
            let (x, y, z) = (side as f64 * 0.085, 0.085 * sin(r), 0.085 * cos(r));
            let n = (x * x + y * y + z * z).sqrt();
            pts.push((x / n * 0.1232, 0.1254 + y / n * 0.1232, z / n * 0.1232));
        }
        fsweep!(
            F,
            format!("seam_side{side}"),
            &pts,
            0.0026,
            M.ball_seam,
            verts = 4,
            closed = true
        );
    }
    done("basketball")
}

/// A steam iron, 26 cm long, standing on its soleplate, its point to the front (+depth): the steel plate, the white
/// shell over it, the handle arched over the back half (held by its middle, 13.8 cm up: actions.ts `PROPS.iron`,
/// grip `level`), a teal water window and dial. The ironing board's own (crates/art/wet/src/laundry) rests at its
/// end and is hidden while this one is in the hand.
pub fn steam_iron() -> Vec<Obj> {
    // the soleplate: square at the heel, curving to the point
    let mut sole: Vec<(f64, f64)> = vec![(-0.055, -0.1), (0.055, -0.1)];
    for k in 0..=10 {
        let a = PI / 2.0 * (1.0 - 2.0 * k as f64 / 10.0);
        sole.push((0.055 * sin(a), 0.04 + 0.12 * cos(a).powf(1.6)));
    }
    fextrude!(
        F,
        "iron_sole",
        &sole,
        0.012,
        (0, 0, 0),
        M.steel,
        plane = Facing::Top,
        bevel = 0.003,
        bevel_segments = 1
    );
    let shell: Vec<(f64, f64)> = sole
        .iter()
        .map(|p| (p.0 * 0.9, p.1 * 0.92 - 0.004))
        .collect();
    fextrude!(
        F,
        "iron_shell",
        &shell,
        0.05,
        (0, 0.012, 0),
        M.iron_shell,
        plane = Facing::Top,
        bevel = 0.016,
        bevel_segments = 2
    );
    fsweep!(
        F,
        "iron_handle",
        &fillet(
            &[
                (0.0, 0.06, -0.085),
                (0.0, 0.125, -0.07),
                (0.0, 0.125, 0.08),
                (0.0, 0.06, 0.1)
            ],
            0.03,
            3
        ),
        0.0,
        M.iron_shell,
        profile = rounded_rect(0.022, 0.03, 0.009, 2),
        up = (1, 0, 0)
    );
    fbox!(
        F,
        "iron_window",
        0.004,
        0.03,
        0.07,
        (0.047, 0.045, 0.04),
        M.iron_trim,
        bevel = 0.002,
        bevel_segments = 1
    );
    disc!(
        "iron_dial",
        0.016,
        0.012,
        (0.0, 0.06, -0.035),
        M.iron_trim,
        verts = 12
    );
    done("iron")
}
