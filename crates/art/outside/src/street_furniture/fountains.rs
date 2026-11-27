//! The drinking fountain and the town fountain: stone, and water that runs.

use super::common::{SF, far_keeping_feet, fluted, lobed, own, sweep_tube};
use artkit::forms::{Scale, smooth_outline};
use artkit::math::{Axis, P3, cosd, sind};
use artkit::motion::{Grow, Squash, grow, squash};
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fcyl, fsphere, fsweep, kit_fillet, kit_moulding, loft, points, revolve};

// --- the drinking fountain: a stone pillar and bowl, a chrome bubbler -------------------------------------------
const TIP: (i32, f64, f64) = (0, 1.075, -0.06); // the bubbler's mouth (game), where the arc of water starts

pub fn drinking_fountain() -> Vec<Obj> {
    let m = own("drinkingFountain");
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            f,
            "slab",
            0.56,
            0.08,
            0.56,
            (0, 0.04, 0),
            m.granite,
            bevel = 0.015
        );
        // the pillar: a moulded base, a fluted shaft narrowing up, a moulded capital under the bowl
        revolve!(
            "pillar_base",
            &[
                (0.0, 0.07),
                (0.2, 0.07),
                (0.2, 0.1),
                (0.188, 0.112),
                (0.175, 0.12),
                (0.18, 0.135),
                (0.162, 0.15),
                (0.152, 0.17),
                (0.0, 0.17)
            ],
            material = m.body,
            segments = 24
        );
        let secs: Vec<_> = [(0.152, 0.165), (0.143, 0.48), (0.136, 0.78)]
            .iter()
            .map(|&(r, y)| fluted(r, y, 16, 0.06, 3, 0.0))
            .collect();
        loft!("pillar", &secs, material = m.body);
        artkit::bevel::eased(revolve!(
            "capital",
            &[
                (0.0, 0.775),
                (0.14, 0.775),
                (0.15, 0.79),
                (0.142, 0.8),
                (0.15, 0.82),
                (0.19, 0.86),
                (0.19, 0.88),
                (0.0, 0.88)
            ],
            material = m.body,
            segments = 24
        ));
        // the bowl: a dish with a rolled lip, falling to the drain
        revolve!(
            "bowl",
            &[
                (0.0, 0.86),
                (0.16, 0.86),
                (0.205, 0.9),
                (0.222, 0.94),
                (0.228, 0.97),
                (0.225, 0.99),
                (0.214, 0.998),
                (0.2, 0.995),
                (0.19, 0.975),
                (0.15, 0.955),
                (0.08, 0.942),
                (0.0, 0.938)
            ],
            material = m.body,
            segments = 32
        );
        fcyl!(
            f,
            "drain",
            0.03,
            0.006,
            (0, 0.94, 0.02),
            m.chrome,
            verts = 12
        );
        for k in 0..3i32 {
            // the drain's grating
            fbox!(
                f,
                format!("grate{k}"),
                0.044 - 0.012 * (k - 1).abs() as f64,
                0.004,
                0.005,
                (0, 0.944, 0.008 + 0.012 * k as f64),
                m.dark
            );
        }
        // the bubbler: a chrome riser at the back of the bowl, bent forward to its mouth, a guard arched over it
        revolve!(
            "bubbler_foot",
            &[
                (0.0, 0.95),
                (0.034, 0.95),
                (0.032, 0.96),
                (0.022, 0.97),
                (0.0, 0.97)
            ],
            at = f.at(0, 0, -0.12),
            material = m.chrome,
            segments = 12
        );
        fsweep!(
            f,
            "bubbler",
            &kit_fillet!(
                &[(0, 0.955, -0.12), (0, 1.073, -0.12), (0, 1.073, -0.062)],
                0.025
            ),
            0.02,
            m.chrome,
            verts = 10
        );
        fcyl!(
            f,
            "mouth",
            0.011,
            0.006,
            (0, 1.091, -0.074),
            m.dark,
            verts = 8
        );
        fsweep!(
            f,
            "guard",
            &points![
                (-0.03, 1.07, -0.075),
                (-0.03, 1.08, -0.045),
                (0, 1.088, -0.03),
                (0.03, 1.08, -0.045),
                (0.03, 1.07, -0.075)
            ],
            0.005,
            m.chrome,
            verts = 6,
            resolution = 2
        );
        // the foot pedal: a bent lever out of the pillar's foot, a ribbed tread on its end
        fsweep!(
            f,
            "pedal_arm",
            &kit_fillet!(&[(0, 0.11, 0.12), (0, 0.11, 0.17), (0, 0.13, 0.2)], 0.02),
            0.012,
            m.chrome,
            verts = 8
        );
        fbox!(
            f,
            "pedal",
            0.18,
            0.025,
            0.08,
            (0, 0.135, 0.22),
            m.chrome,
            bevel = 0.008
        );
        for k in 0..3 {
            fbox!(
                f,
                format!("tread{k}"),
                0.16,
                0.006,
                0.01,
                (0, 0.149, 0.195 + 0.025 * k as f64),
                m.dark
            );
        }
    });
    join(&still, "drinkingFountain", Some(P3::from((0, 0, 0))));
    // the arc of water from the mouth into the bowl, its origin at the mouth, hidden (scaled to nothing) at rest
    let arc = new_parts(|| {
        sweep_tube(
            &f,
            "arc",
            &[
                TIP,
                (0, 1.115, -0.03),
                (0, 1.13, 0.005),
                (0, 1.115, 0.045),
                (0, 1.065, 0.08),
                (0, 0.985, 0.1),
            ],
            0.013,
            m.spray,
            8,
            Scale::List(vec![1.0, 1.0, 0.95, 0.9, 0.85, 0.75]),
        );
    });
    let water = hinged(join(&arc, "water", None), f.at(TIP.0, TIP.1, TIP.2));
    water.set_scale((0.001, 0.001, 0.001));
    settle_all();
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        grow(
            water,
            "water_on",
            Grow {
                seconds: 0.35,
                axis,
            },
        );
    }
    meshes()
}

pub fn drinking_fountain_far() -> Vec<Obj> {
    far_keeping_feet(drinking_fountain, "drinkingFountain")
}

// --- the town fountain: a square stone basin, a two-tier fountain in the middle ------------------------------
const RIM: f64 = 0.5; // the rim's top: the content's `seat.height`, its front at `seat.forward` 1.3
const WATER_Y: f64 = 0.37; // the basin's water
const BOWL_Y: f64 = 1.46; // the lower bowl's water
// (radius, height) along a stream falling from a bowl's lip: out over the lip, then down into the water below
const TRICKLE: [(f64, f64); 4] = [(0.235, 1.995), (0.265, 1.96), (0.29, 1.86), (0.3, BOWL_Y)];
const STREAM: [(f64, f64); 5] = [
    (0.6, 1.49),
    (0.65, 1.45),
    (0.69, 1.33),
    (0.715, 1.05),
    (0.73, WATER_Y),
];

/// The square basin: slab, walls on a plinth band with a raised panel a face between pilasters, a moulded coping with a
/// rounded nose all round (its top the seat, `RIM`), the water.
fn basin(f: &GameFrame, m: &SF) {
    let stone = m.body;
    fbox!(
        f,
        "slab",
        2.9,
        0.06,
        2.9,
        (0, 0.03, 0),
        m.granite,
        bevel = 0.01
    );
    // the basin: its walls on a plinth band, a raised panel on each face between pilasters, a moulded coping
    // with a rounded nose all round (its top the seat, `RIM`)
    for s in [-1, 1] {
        let sf = s as f64;
        fbox!(
            f,
            format!("wall_fb{s}"),
            2.9,
            0.4,
            0.24,
            (0, 0.26, sf * 1.33),
            stone,
            bevel = 0.012
        );
        fbox!(
            f,
            format!("wall_lr{s}"),
            0.24,
            0.4,
            2.42,
            (sf * 1.33, 0.26, 0),
            stone,
            bevel = 0.012
        );
        fbox!(
            f,
            format!("band_fb{s}"),
            2.93,
            0.06,
            0.25,
            (0, 0.09, sf * 1.335),
            stone,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            format!("band_lr{s}"),
            0.25,
            0.06,
            2.43,
            (sf * 1.335, 0.09, 0),
            stone,
            bevel = 0.01,
            bevel_segments = 1
        );
        for k in 0..3 {
            let u = (k - 1) as f64 * 0.9;
            fbox!(
                f,
                format!("panel_fb{s}{k}"),
                0.7,
                0.2,
                0.02,
                (u, 0.28, sf * 1.455),
                stone,
                bevel = 0.006,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("panel_lr{s}{k}"),
                0.02,
                0.2,
                0.7,
                (sf * 1.455, 0.28, u),
                stone,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
    }
    let nose = [
        (0.0, 0.0),
        (-0.02, 0.006),
        (-0.03, 0.025),
        (-0.02, 0.044),
        (0.0, 0.05),
        (0.31, 0.05),
        (0.32, 0.04),
        (0.32, 0.0),
    ];
    kit_moulding!(
        f,
        "coping",
        &[(1.45, 1.45), (-1.45, 1.45), (-1.45, -1.45), (1.45, -1.45)],
        &nose,
        stone,
        at = RIM - 0.05,
        plane = Facing::Top
    );
    fbox!(
        f,
        "basin_water",
        2.42,
        0.02,
        2.42,
        (0, WATER_Y - 0.01, 0),
        m.water
    );
}

/// The fountain in the middle: a moulded foot, a fluted column, the wide lower bowl gadrooned underneath with a rolled
/// lip, a stem, the small top bowl and a knob.
fn pedestal(f: &GameFrame, m: &SF) {
    let stone = m.body;
    artkit::bevel::eased(revolve!(
        "foot",
        &smooth_outline(
            &[
                (0.0, 0.3),
                (0.38, 0.3),
                (0.38, 0.34),
                (0.35, 0.37),
                (0.3, 0.41),
                (0.24, 0.5),
                (0.21, 0.6),
                (0.2, 0.66),
                (0.0, 0.66)
            ],
            1,
            false
        ),
        material = stone,
        segments = 28
    ));
    revolve!(
        "column_base",
        &[
            (0.0, 0.64),
            (0.16, 0.64),
            (0.16, 0.67),
            (0.135, 0.7),
            (0.0, 0.7)
        ],
        material = stone,
        segments = 16
    );
    loft!(
        "column",
        &[
            fluted(0.12, 0.69, 12, 0.1, 3, 0.0),
            fluted(0.115, 1.13, 12, 0.1, 3, 0.0)
        ],
        material = stone
    );
    revolve!(
        "capital",
        &[
            (0.0, 1.12),
            (0.13, 1.12),
            (0.15, 1.15),
            (0.17, 1.2),
            (0.0, 1.2)
        ],
        material = stone,
        segments = 16
    );
    revolve!(
        "bowl",
        &smooth_outline(
            &[
                (0.0, 1.18),
                (0.2, 1.2),
                (0.5, 1.36),
                (0.6, 1.44),
                (0.612, 1.455),
                (0.612, 1.475),
                (0.6, 1.488),
                (0.55, 1.49),
                (0.5, 1.44),
                (0.0, 1.42)
            ],
            1,
            false
        ),
        material = stone,
        segments = 32
    );
    let secs: Vec<_> = [(0.16, 1.19), (0.32, 1.255), (0.47, 1.335), (0.565, 1.4)]
        .iter()
        .map(|&(r, y)| lobed(r, y, 16, 0.12, 2))
        .collect();
    loft!("bowl_lobes", &secs, material = stone);
    fcyl!(
        f,
        "bowl_water",
        0.53,
        0.02,
        (0, BOWL_Y, 0),
        m.water,
        verts = 24
    );
    revolve!(
        "stem",
        &[
            (0.0, 1.44),
            (0.11, 1.44),
            (0.1, 1.47),
            (0.075, 1.52),
            (0.062, 1.62),
            (0.07, 1.72),
            (0.062, 1.8),
            (0.09, 1.85),
            (0.1, 1.88),
            (0.0, 1.88)
        ],
        material = stone,
        segments = 14
    );
    artkit::bevel::eased(revolve!(
        "top_bowl",
        &smooth_outline(
            &[
                (0.0, 1.84),
                (0.08, 1.85),
                (0.22, 1.94),
                (0.244, 1.97),
                (0.244, 1.99),
                (0.21, 2.0),
                (0.0, 1.97)
            ],
            1,
            false
        ),
        material = stone,
        segments = 24
    ));
    let secs: Vec<_> = [(0.085, 1.852), (0.16, 1.895), (0.225, 1.945)]
        .iter()
        .map(|&(r, y)| lobed(r, y, 12, 0.14, 2))
        .collect();
    loft!("top_lobes", &secs, material = stone);
    revolve!(
        "knob",
        &[
            (0.0, 1.97),
            (0.05, 1.97),
            (0.045, 1.99),
            (0.07, 2.03),
            (0.065, 2.07),
            (0.04, 2.1),
            (0.0, 2.114)
        ],
        material = stone,
        segments = 12
    );
}

/// The water spilling over the lips: streams curving out of the top bowl into the lower one, and out of the lower one
/// into the basin, thinning as they fall.
fn falls(f: &GameFrame, m: &SF) {
    for k in 0..6 {
        let a = (30 + 60 * k) as f64;
        let pts: Vec<(f64, f64, f64)> = TRICKLE
            .iter()
            .map(|&(r, y)| (r * cosd(a), y, r * sind(a)))
            .collect();
        fsweep!(
            f,
            format!("trickle{k}"),
            &pts,
            0.014,
            m.spray,
            verts = 5,
            resolution = 2,
            scale = [1.0, 0.95, 0.85, 0.75]
        );
    }
    for k in 0..8 {
        let a = 22.5 + (45 * k) as f64;
        let pts: Vec<(f64, f64, f64)> = STREAM
            .iter()
            .map(|&(r, y)| (r * cosd(a), y, r * sind(a)))
            .collect();
        fsweep!(
            f,
            format!("stream{k}"),
            &pts,
            0.024,
            m.spray,
            verts = 5,
            resolution = 2,
            scale = [1.0, 0.95, 0.85, 0.75, 0.7]
        );
    }
}

pub fn fountain_town() -> Vec<Obj> {
    let m = own("fountainTown");
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        basin(&f, &m);
        pedestal(&f, &m);
        falls(&f, &m);
    });
    join(&still, "fountainTown", Some(P3::from((0, 0, 0))));
    // what moves: the jet on top, rising and falling, and the splashes where the streams land, bobbing with it
    let jet = new_parts(|| {
        fcone!(
            f,
            "jet",
            0.03,
            0.022,
            0.32,
            (0, 2.24, 0),
            m.spray,
            verts = 10
        );
        fsphere!(
            f,
            "jet_crown",
            0.065,
            (0, 2.405, 0),
            m.spray,
            segments = 12,
            rings = 6,
            scale = (1, 1, 1.05)
        );
    });
    let jet = hinged(join(&jet, "jet", None), f.at(0, 2.08, 0));
    let splashes = new_parts(|| {
        for k in 0..8 {
            let a = 22.5 + (45 * k) as f64;
            let (x, z) = (0.73 * cosd(a), 0.73 * sind(a));
            // a crown of spray: a low ring flaring up round where the stream lands
            artkit::bevel::eased(revolve!(
                format!("splash{k}"),
                &[
                    (0.0, -0.02),
                    (0.08, -0.02),
                    (0.1, 0.025),
                    (0.05, 0.012),
                    (0.0, 0.02)
                ],
                at = f.at(x, WATER_Y, z),
                material = m.spray,
                segments = 8
            ));
        }
    });
    let splash = hinged(join(&splashes, "splash", None), f.at(0, WATER_Y, 0));
    settle_all();
    squash(
        jet,
        "water_run",
        (1, 1, 1.3),
        Squash {
            seconds: 1.2,
            back: true,
            ..Squash::default()
        },
    );
    squash(
        splash,
        "water_run",
        (1, 1, 1.8),
        Squash {
            seconds: 1.2,
            back: true,
            ..Squash::default()
        },
    );
    meshes()
}

pub fn fountain_town_far() -> Vec<Obj> {
    far_keeping_feet(fountain_town, "fountainTown")
}
