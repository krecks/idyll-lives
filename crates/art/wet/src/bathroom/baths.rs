//! The baths: a tub, its taps and the bath water.
//!
//! 2 x 1 tiles, centred; the resident sits along the tub, back at the end without the taps and legs out towards
//! them (furniture.json `seats`: 0.25-0.28 m, so the tub's floor is 10-13 cm below). Each tub is one closed shell
//! lofted through rounded rings (`loft`): up the outside, over the rim and down the inside to the floor, so its
//! inside faces up into the tub. Two moving parts: the tap's stream (`tap_on`, 0.4 s, held 2.4 s so the tap runs
//! while the tub fills) and the bath water (`fill`): a body of water in the tub's shape up to two thirds of its
//! depth, its origin just under the tub's floor and scaled to nothing upright, which waits 0.4 s for the tap and
//! then rises in 2 s; it stays for the whole bath and drains as the bath ends (the game plays `fill` back).

use super::M;
use super::bath_parts::{
    Section, bath_water, draped_towel, duck, fill, folded_towel, ring, tub, waste,
};
use super::common::{LATHE, Lathe, bends, lathe, rose};
use crate::common::stream;
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{cos, radians, sin};
use artkit::parts::{ASIDE, aside, join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, py, soft};

pub fn bathtub() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (rim, floor_y) = (0.58, 0.15);
    let sections: [Section; 10] = [
        (0.70, 0.24, 0.12, 3.0),
        (0.79, 0.33, 0.48, 4.0), // the tub's outside, behind the panels
        (0.91, 0.43, 0.50, 10.0),
        (0.91, 0.43, rim - 0.004, 10.0), // the deck's edge, eased
        (0.906, 0.426, rim, 10.0),
        (0.774, 0.294, rim, 4.0),
        (0.769, 0.289, rim - 0.003, 4.0),
        (0.765, 0.285, rim - 0.012, 4.0), // the deck, and the inside's lip
        (0.71, 0.25, 0.25, 3.5),
        (0.66, 0.2, floor_y, 3.0), // down the inside to the floor
    ];
    let mut rings = None;
    let still = new_parts(|| {
        rings = Some(tub(&sections, rim, floor_y));
        // the panels down to a plinth set back under them, each with a moulded frame on it
        soft!(
            f,
            "panel_front",
            1.82,
            0.45,
            0.03,
            (0, 0.275, 0.415),
            M.bath_panel,
            round = 0.006,
            round_segments = 1
        );
        fextrude!(
            f,
            "panel_moulding",
            &rounded_rect(1.7, 0.33, 0.01, 2),
            0.006,
            (0, 0.285, 0.432),
            M.bath_panel,
            holes = vec![rounded_rect(1.66, 0.29, 0.008, 2)],
            bevel = 0.002,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            soft!(
                f,
                format!("panel_end{s}"),
                0.03,
                0.45,
                0.83,
                (s as f64 * 0.895, 0.275, -0.015),
                M.bath_panel,
                round = 0.006,
                round_segments = 1
            );
        }
        fbox!(f, "plinth", 1.76, 0.05, 0.8, (0, 0.025, -0.025), M.dark);
        waste(&f, -0.55, rim, floor_y, -0.705, M.chrome);
        // a deck mixer at the left end: a pillar, a spout curving over the water, two lever handles
        rose(&f, "tap_rose", -0.85, rim, 0, 0.034, M.chrome, 0.01, 16);
        lathe(
            &f,
            "tap_pillar",
            -0.85,
            0.0,
            &[
                (rim, 0.024, 0.024),
                (rim + 0.08, 0.022, 0.022),
                (rim + 0.095, 0.026, 0.026),
                (rim + 0.11, 0.02, 0.02),
                (rim + 0.113, 0.016, 0.016), // its top eased
            ],
            M.chrome,
            Lathe { verts: 14, ..LATHE },
        );
        kit::tube!(
            f,
            "spout",
            &bends(
                &[
                    (-0.85, rim + 0.09, 0.0),
                    (-0.68, rim + 0.09, 0.0),
                    (-0.68, rim + 0.06, 0.0),
                ],
                0.025,
                4,
            ),
            0.015,
            M.chrome,
            verts = 10,
            round = 1,
            scale = [1.0, 1.0, 0.9]
        );
        for z in [-0.1, 0.1] {
            rose(
                &f,
                format!("handle_rose{}", float_repr(z)),
                -0.85,
                rim,
                z,
                0.026,
                M.chrome,
                0.008,
                12,
            );
            fcyl!(
                f,
                format!("handle{}", float_repr(z)),
                0.018,
                0.04,
                (-0.85, rim + 0.025, z),
                M.chrome,
                verts = 12,
                bevel = 0.004,
                bevel_segments = 1
            );
            kit::tube!(
                f,
                format!("handle_lever{}", float_repr(z)),
                &[
                    (-0.85, rim + 0.045, z),
                    (-0.81, rim + 0.05, z),
                    (-0.79, rim + 0.054, z),
                ],
                0.006,
                M.chrome,
                verts = 8,
                round = 2,
                scale = [1.2, 1.0, 0.8]
            );
        }
        folded_towel(&f, "towel", 0.4, 0.12, 0.55, rim, 0.36, M.towel, 2, 0.024);
        duck(&f, 0.12, rim, 0.36);
    });
    let (floor_ring, top) = rings.unwrap();
    join(&still, "body", Some((0, 0, 0).into()));
    let stream = stream(
        &f,
        "stream",
        (-0.68, rim + 0.045, 0),
        floor_y,
        M.water,
        0.011,
        10,
        0.0,
    );
    let water = bath_water(&f, floor_ring, top, 0.006);
    settle_all();
    fill(stream, water);
    meshes()
}

/// A brass ball-and-claw foot: a leg curving out from under the tub down onto a ball gripped by three claws.
fn claw(f: &GameFrame, x: f64, z: f64) {
    let (sx, sz) = (
        if x > 0.0 { 1.0 } else { -1.0 },
        if z > 0.0 { 1.0 } else { -1.0 },
    );
    let (xs, zs) = (float_repr(x), float_repr(z));
    kit::tube!(
        f,
        format!("leg{xs}{zs}"),
        &[
            (x - sx * 0.03, 0.2, z - sz * 0.02),
            (x + sx * 0.0, 0.14, z),
            (x + sx * 0.012, 0.08, z + sz * 0.008),
        ],
        0.03,
        M.feet,
        verts = 10,
        scale = [1.3, 0.85, 0.75]
    );
    fsphere!(
        f,
        format!("ball{xs}{zs}"),
        0.034,
        (x + sx * 0.012, 0.034, z + sz * 0.008),
        M.feet,
        segments = 12,
        rings = 8
    );
    for k in 0..3 {
        let a = radians(((k - 1) * 55) as f64);
        let (dx, dz) = (sx * cos(a) * 0.7 + 0.0, sz * sin(a) * 0.7 + sz * 0.3);
        let l = py::hypot(dx, dz);
        let (dx, dz) = (dx / l, dz / l);
        let (cx, cz) = (x + sx * 0.012, z + sz * 0.008);
        kit::tube!(
            f,
            format!("claw{xs}{zs}{k}"),
            &[
                (cx + dx * 0.012, 0.085, cz + dz * 0.012),
                (cx + dx * 0.03, 0.06, cz + dz * 0.03),
                (cx + dx * 0.036, 0.03, cz + dz * 0.036),
                (cx + dx * 0.03, 0.006, cz + dz * 0.03),
            ],
            0.009,
            M.feet,
            verts = 6,
            round = 2,
            scale = [1.0, 1.0, 0.8, 0.5]
        );
    }
}

pub fn bathtub_clawfoot() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (rim, floor_y) = (0.7, 0.17);
    let sections: [Section; 10] = [
        (0.6, 0.24, 0.14, 2.3),
        (0.75, 0.33, 0.3, 2.4),
        (0.84, 0.385, 0.6, 2.5), // the outside
        (0.865, 0.398, 0.665, 2.5),
        (0.852, 0.392, rim, 2.5), // the rolled top
        (0.815, 0.358, rim, 2.5),
        (0.795, 0.342, 0.665, 2.5),
        (0.74, 0.3, 0.45, 2.4),
        (0.64, 0.22, 0.22, 2.3),
        (0.56, 0.17, floor_y, 2.3), // the inside
    ];
    let mut rings = None;
    let still = new_parts(|| {
        rings = Some(tub(&sections, rim, floor_y));
        // the rolled rim: a round bead swept all the way round the top
        let bead: Vec<(f64, f64, f64)> = ring(0.834, 0.374, 2.5, 48)
            .into_iter()
            .map(|(x, z)| (x, rim - 0.008, z))
            .collect();
        fsweep!(
            f,
            "rim_roll",
            &bead,
            0.021,
            M.porcelain,
            verts = 8,
            closed = true,
            angle = 60
        );
        for x in [-0.5, 0.5] {
            for z in [-0.16, 0.16] {
                claw(&f, x, z);
            }
        }
        waste(&f, -0.45, rim, floor_y, -0.775, M.brass);
        // a towel over the back rim at the right end
        draped_towel(
            &f,
            "towel",
            0.5,
            0.32,
            &[
                (rim + 0.008, -0.31),
                (rim + 0.012, -0.37),
                (rim, -0.415),
                (rim - 0.04, -0.425),
                (rim - 0.24, -0.424),
            ],
            M.towel,
            0.009, // a terry towel's one layer: its cut end too thin to show an edge
        );
    });
    let (floor_ring, top) = rings.unwrap();
    join(&still, "body", Some((0, 0, 0).into()));
    // the taps: drawn, not the tub's size
    let taps = new_parts(|| {
        // a brass filler on the left end: a riser up over the rim and out over the water, two cross handles on a bar
        kit::tube!(
            f,
            "filler",
            &bends(
                &[
                    (-0.785, rim - 0.01, 0.0),
                    (-0.785, 1.0, 0.0),
                    (-0.6, 1.0, 0.0),
                    (-0.6, 0.93, 0.0),
                ],
                0.04,
                5,
            ),
            0.016,
            M.brass,
            verts = 10,
            round = 1
        );
        fcyl!(
            f,
            "filler_aerator",
            0.019,
            0.022,
            (-0.6, 0.93, 0),
            M.brass,
            verts = 10
        );
        kit::tube!(
            f,
            "filler_bar",
            &[(-0.785, 0.8, -0.09), (-0.785, 0.8, 0.09)],
            0.013,
            M.brass,
            verts = 10,
            round = 1
        );
        for z in [-0.08, 0.08] {
            let zs = float_repr(z);
            fcyl!(
                f,
                format!("filler_stem{zs}"),
                0.01,
                0.04,
                (-0.785, 0.82, z),
                M.brass,
                verts = 8
            );
            rose(
                &f,
                format!("filler_hub{zs}"),
                -0.785,
                0.835,
                z,
                0.014,
                M.brass,
                0.012,
                10,
            );
            for a in [0, 90] {
                let (ca, sa) = (cos(radians(a as f64)), sin(radians(a as f64)));
                kit::tube!(
                    f,
                    format!("filler_cross{zs}{a}"),
                    &[
                        (-0.785 - ca * 0.032, 0.845, z - sa * 0.032),
                        (-0.785 + ca * 0.032, 0.845, z + sa * 0.032),
                    ],
                    0.006,
                    M.brass,
                    verts = 6,
                    round = 1
                );
                for s in [-1, 1] {
                    fsphere!(
                        f,
                        format!("filler_tip{zs}{a}{s}"),
                        0.008,
                        (
                            -0.785 + s as f64 * ca * 0.034,
                            0.845,
                            z + s as f64 * sa * 0.034
                        ),
                        M.brass,
                        segments = 6,
                        rings = 4
                    );
                }
            }
        }
        // a telephone hand shower on its cradle on the riser
        kit::tube!(
            f,
            "cradle",
            &bends(
                &[
                    (-0.785, 0.9, 0.0),
                    (-0.785, 0.9, 0.05),
                    (-0.785, 0.93, 0.07),
                ],
                0.015,
                3,
            ),
            0.008,
            M.brass,
            verts = 8,
            round = 1
        );
        kit::tube!(
            f,
            "handset",
            &[(-0.785, 0.93, 0.03), (-0.785, 0.945, 0.11)],
            0.012,
            M.brass,
            verts = 10,
            round = 1,
            scale = [0.9, 1.1]
        );
        fcyl!(
            f,
            "handset_head",
            0.022,
            0.03,
            (-0.785, 0.955, 0.125),
            M.brass,
            rot = (-60, 0, 0),
            verts = 12,
            bevel = 0.005,
            bevel_segments = 1
        );
    });
    aside(&taps, "taps");
    let stream = stream(
        &f,
        // the taps' water: drawn with them, not the tub's size
        &format!("stream{ASIDE}"),
        (-0.6, 0.919, 0),
        floor_y,
        M.water,
        0.011,
        10,
        0.0,
    );
    let water = bath_water(&f, floor_ring, top, 0.006);
    settle_all();
    fill(stream, water);
    meshes()
}

pub fn bathtub_freestanding() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (rim, floor_y) = (0.58, 0.15);
    let sections: [Section; 11] = [
        (0.66, 0.27, 0.0, 2.2),
        (0.7, 0.3, 0.03, 2.2),
        (0.86, 0.395, 0.45, 2.3), // the outside
        (0.895, 0.41, 0.56, 2.3),
        (0.889, 0.4075, rim - 0.006, 2.3), // the rim, rolled
        (0.882, 0.4035, rim, 2.3),
        (0.85, 0.372, rim, 2.3),
        (0.832, 0.356, rim - 0.025, 2.3),
        (0.77, 0.315, 0.35, 2.25),
        (0.67, 0.245, 0.18, 2.2),
        (0.61, 0.205, floor_y, 2.2), // the inside
    ];
    let mut rings = None;
    let still = new_parts(|| {
        rings = Some(tub(&sections, rim, floor_y));
        waste(&f, 0.5, rim, floor_y, 0.84, M.chrome);
        // a towel over the back rim at the left end
        draped_towel(
            &f,
            "towel",
            -0.5,
            0.32,
            &[
                (rim + 0.008, -0.3),
                (rim + 0.012, -0.36),
                (rim, -0.41),
                (rim - 0.04, -0.415),
                (rim - 0.22, -0.41),
            ],
            M.towel,
            0.009, // a terry towel's one layer: its cut end too thin to show an edge
        );
    });
    let (floor_ring, top) = rings.unwrap();
    join(&still, "body", Some((0, 0, 0).into()));
    // the floor-standing filler: drawn, not the tub's size
    let filler = new_parts(|| {
        // a floor-standing filler at the right end: a rose on the floor, a column with its mixer and a handset,
        // bent over into a spout above the water
        rose(&f, "filler_base", 0.93, 0.0, 0.12, 0.07, M.chrome, 0.02, 20);
        kit::tube!(
            f,
            "filler",
            &bends(
                &[
                    (0.93, 0.02, 0.12),
                    (0.93, 0.9, 0.12),
                    (0.61, 0.9, 0.0),
                    (0.61, 0.82, 0.0),
                ],
                0.06,
                6,
            ),
            0.019,
            M.chrome,
            verts = 12,
            round = 1
        );
        fcyl!(
            f,
            "filler_aerator",
            0.021,
            0.02,
            (0.61, 0.822, 0.0),
            M.chrome,
            verts = 12
        );
        lathe(
            &f,
            "filler_mixer",
            0.93,
            0.12,
            &[
                (0.7, 0.024, 0.024),
                (0.71, 0.03, 0.03),
                (0.77, 0.03, 0.03),
                (0.78, 0.024, 0.024),
            ],
            M.chrome,
            Lathe { verts: 14, ..LATHE },
        );
        kit::tube!(
            f,
            "filler_lever",
            &[(0.93, 0.775, 0.12), (0.93, 0.79, 0.15), (0.93, 0.8, 0.19)],
            0.007,
            M.chrome,
            verts = 8,
            round = 2,
            scale = [1.2, 1.0, 0.8]
        );
        kit::tube!(
            f,
            "handset",
            &[(0.955, 0.5, 0.12), (0.955, 0.66, 0.12)],
            0.012,
            M.chrome,
            verts = 10,
            round = 1,
            scale = [0.9, 1.1]
        );
        fcyl!(
            f,
            "handset_head",
            0.022,
            0.03,
            (0.955, 0.675, 0.12),
            M.chrome,
            verts = 12,
            bevel = 0.005,
            bevel_segments = 1
        );
        kit::tube!(
            f,
            "hose",
            &[
                (0.955, 0.5, 0.12),
                (0.97, 0.3, 0.13),
                (0.95, 0.1, 0.12),
                (0.93, 0.03, 0.12),
            ],
            0.006,
            M.chrome,
            verts = 6
        );
    });
    aside(&filler, "filler");
    let stream = stream(
        &f,
        // the taps' water: drawn with them, not the tub's size
        &format!("stream{ASIDE}"),
        (0.61, 0.81, 0.0),
        floor_y,
        M.water,
        0.011,
        10,
        0.0,
    );
    let water = bath_water(&f, floor_ring, top, 0.006);
    settle_all();
    fill(stream, water);
    meshes()
}
