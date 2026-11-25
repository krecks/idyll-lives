//! The rose-gold vanity and the moonlight gramophone.

use super::common::{_rose, _rose_leaves, M, WALL};
use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{Axis, M4, P3, PI, TAU, V3, cos, cosd, radians, sin, sind};
use artkit::parts::{aside, hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, rod, settle, slide, soft, spin,
    sweep, swing, torus,
};
use artkit::{forms::eased, shapes::AUTO_WIDTH};

// --- the rose-gold vanity (1x1): a dressing table with an oval mirror, a pouf before it ---------------------
pub fn vanity() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fextrude!(
            f,
            "vanity_top",
            &rounded_rect(0.8, 0.36, 0.06, 4),
            0.05,
            (0, 0.735, -0.25),
            M.ivory,
            plane = Facing::Top,
            bevel = 0.012,
            bevel_segments = 2
        );
        // the apron, its lower edge scalloped either side of the drawer; the drawer's opening
        let mut edge = vec![(0.38, 0.14), (-0.38, 0.14)];
        edge.extend((0..21).map(|i| {
            let x = -0.38 + 0.76 * i as f64 / 20.0;
            (x, 0.012 * sin(PI * x / 0.19).abs())
        }));
        fextrude!(
            f,
            "apron",
            &edge,
            0.3,
            (0, 0.57, -0.25),
            M.ivory,
            plane = Facing::Front,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "drawer_hole",
            0.36,
            0.11,
            0.01,
            (0, 0.635, -0.098),
            M.blushd
        );
        for x in [-0.34, 0.34] {
            for z in [-0.38, -0.12] {
                kit::cabriole!(
                    f,
                    format!("leg{x}{z}"),
                    x,
                    z,
                    0.57,
                    M.rosegold,
                    0.022 * if x > 0.0 { 1.0 } else { -1.0 },
                    0.02 * if z > -0.25 { 1.0 } else { -1.0 },
                    r = 0.018
                );
            }
        }
        // the oval mirror in a moulded rose-gold frame, on a stand rising from the top's back
        let oval: Vec<(f64, f64, f64)> = (0..36)
            .map(|k| {
                let a = TAU * k as f64 / 36.0;
                (0.25 * cos(a), 1.24 + 0.36 * sin(a), -0.405)
            })
            .collect();
        let frame = fsweep!(
            f,
            "mirror_frame",
            &oval,
            0.0,
            M.rosegold,
            closed = true,
            profile = [
                (-0.012, -0.022),
                (0.012, -0.022),
                (0.016, 0.0),
                (0.012, 0.022),
                (-0.012, 0.022)
            ],
            up = (0, 1, 0)
        );
        bevel_edges(frame, AUTO_WIDTH, 1, 60.0, &EdgeOrder::Corners); // its moulding's arrises
        fcyl!(
            f,
            "mirror_back",
            0.25,
            0.012,
            (0, 1.24, -0.418),
            M.rosegold,
            rot = (90, 0, 0),
            scale = (1, 1.44, 1),
            verts = 36,
            bevel = 0.004,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "mirror",
            0.235,
            0.004,
            (0, 1.24, -0.398),
            M.mirror,
            rot = (90, 0, 0),
            scale = (1, 1.44, 1),
            verts = 36
        );
        fsweep!(
            f,
            "mirror_stand",
            &[(0.0, 0.76, -0.41), (0.0, 0.84, -0.412), (0.0, 0.9, -0.41)],
            0.0,
            M.rosegold,
            profile = rounded_rect(0.012, 0.05, 0.005, 1),
            up = (0, 0, 1),
            scale = [1.3, 1.0, 1.1]
        );
        for (k, (x, h, m)) in [
            (-0.3, 0.12, M.perfume_a),
            (-0.22, 0.09, M.perfume_b),
            (-0.15, 0.14, M.perfume_c),
        ]
        .into_iter()
        .enumerate()
        {
            let b = f.at(x, 0.76, -0.22);
            let sections: Vec<Vec<P3>> = [
                (0.0, 0.9, 4),
                (0.04, 1.0, 4),
                (0.7, 1.0, 4),
                (0.82, 0.4, 2),
                (1.0, 0.36, 2),
            ]
            .iter()
            .map(|&(t, kk, sq)| {
                ring!(
                    0.03 * kk,
                    0.022 * kk,
                    z = b.z + h * t,
                    verts = 12,
                    square = sq,
                    at = (b.x, b.y)
                )
            })
            .collect();
            loft!(
                format!("perfume{k}"),
                &sections,
                material = m,
                smooth = false
            );
            revolve!(
                format!("stopper{k}"),
                &[
                    (0.0, 0.0),
                    (0.008, 0.0),
                    (0.008, 0.008),
                    (0.018, 0.02),
                    (0.012, 0.036),
                    (0.0, 0.04)
                ],
                at = f.at(x, 0.76 + h, -0.22),
                material = M.rosegold,
                segments = 8
            );
        }
        soft!(
            f,
            "jewellery_box",
            0.16,
            0.08,
            0.12,
            (0.25, 0.8, -0.22),
            M.blushd,
            round = 0.012,
            round_segments = 1
        );
        soft!(
            f,
            "jewellery_lid",
            0.17,
            0.016,
            0.13,
            (0.25, 0.846, -0.22),
            M.rosegold,
            round = 0.006,
            round_segments = 1
        );
        fsweep!(
            f,
            "necklace",
            &[
                (0.2, 0.86, -0.18),
                (0.24, 0.855, -0.155),
                (0.28, 0.86, -0.16),
                (0.3, 0.84, -0.16),
                (0.31, 0.77, -0.15)
            ],
            0.002,
            M.gold,
            verts = 3,
            resolution = 2
        );
        revolve!(
            "vase",
            &eased(
                &[
                    (0.0, 0.0),
                    (0.03, 0.0),
                    (0.04, 0.07),
                    (0.025, 0.12),
                    (0.03, 0.14),
                    (0.026, 0.14),
                    (0.0, 0.13)
                ],
                AUTO_WIDTH
            ),
            at = f.at(0.1, 0.76, -0.36),
            material = M.ivory,
            segments = 12
        );
        for (k, (x, y, z, m)) in [
            (0.07, 0.92, -0.34, M.rose),
            (0.13, 0.93, -0.37, M.rose_pink),
            (0.1, 0.95, -0.33, M.ivory),
        ]
        .into_iter()
        .enumerate()
        {
            fsweep!(
                f,
                format!("vase_stem{k}"),
                &[(0.1, 0.88, -0.36), (x, y, z)],
                0.003,
                M.leaf,
                verts = 4
            );
            _rose(
                f,
                &format!("vase_rose{k}"),
                x,
                y - 0.005,
                z,
                0.035,
                Some(m),
                7,
                15.0,
                (k * 120) as f64,
            );
        }
        _rose_leaves(f, "vase_leaf", 0.1, 0.9, -0.36, 3, 0.045, 0.0, None);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the pouf before it: drawn with it, not the dressing table's own size
    let pouf = new_parts(|| {
        // buttoned, on a rose-gold ring with four little feet
        revolve!(
            "pouf",
            &[
                (0.0, 0.03),
                (0.19, 0.03),
                (0.21, 0.07),
                (0.212, 0.34),
                (0.2, 0.42),
                (0.16, 0.44),
                (0.06, 0.43),
                (0.0, 0.425)
            ],
            at = f.at(0, 0.0, 0.17),
            material = M.velvet,
            segments = 24
        );
        torus!(
            "pouf_ring",
            0.212,
            0.01,
            at = f.at(0, 0.04, 0.17),
            material = M.rosegold,
            major_segments = 24,
            minor_segments = 5
        );
        for a in (45..360).step_by(90) {
            let af = a as f64;
            fsweep!(
                f,
                format!("pouf_foot{a}"),
                &[
                    (0.17 * cosd(af), 0.04, 0.17 + 0.17 * sind(af)),
                    (0.19 * cosd(af), 0.004, 0.17 + 0.19 * sind(af))
                ],
                0.009,
                M.rosegold,
                verts = 6,
                scale = [1.0, 0.7]
            );
        }
        for a in (0..360).step_by(60) {
            let af = a as f64;
            fsphere!(
                f,
                format!("tuft{a}"),
                0.012,
                (0.11 * cosd(af), 0.432, 0.17 + 0.11 * sind(af)),
                M.blushd,
                scale = (1, 1, 0.6),
                segments = 6,
                rings = 4
            );
        }
        fsphere!(
            f,
            "tuft_mid",
            0.013,
            (0, 0.428, 0.17),
            M.blushd,
            scale = (1, 1, 0.6),
            segments = 6,
            rings = 4
        );
    });
    aside(&pouf, "pouf");
    // The middle drawer, pulled out to the front by `drawer_open`.
    let drawer = new_parts(|| {
        fbox!(
            f,
            "drawer_front",
            0.34,
            0.1,
            0.014,
            (0, 0.635, -0.091),
            M.ivory,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "drawer_box",
            0.32,
            0.08,
            0.26,
            (0, 0.635, -0.22),
            M.ivory
        );
        kit::knob!(
            f,
            "drawer_knob",
            0,
            0.635,
            -0.084,
            0.012,
            M.rosegold,
            segments = 10
        );
        fsweep!(
            f,
            "lipstick",
            &[(-0.08, 0.676, -0.18), (-0.08, 0.676, -0.12)],
            0.009,
            M.rosegold,
            verts = 8
        );
        fsweep!(
            f,
            "lipstick_tip",
            &[(-0.08, 0.676, -0.12), (-0.08, 0.676, -0.1)],
            0.007,
            M.rose,
            verts = 8,
            scale = [1.0, 0.6]
        );
        revolve!(
            "compact",
            &[
                (0.0, 0.0),
                (0.035, 0.0),
                (0.036, 0.006),
                (0.034, 0.012),
                (0.0, 0.013)
            ],
            at = f.at(0.06, 0.676, -0.2),
            material = M.rosegold,
            segments = 14
        );
    });
    let drawer = hinged(join(&drawer, "drawer", None), f.at(0, 0.635, -0.084));
    let objs = settle!(centre = false);
    slide!(drawer, "drawer_open", (0, -0.2, 0), seconds = 0.7);
    objs
}

// --- the moonlight gramophone (1x1): a horn gramophone on a record cabinet -----------------------------------
const RECORD: (f64, f64, f64) = (0.02, 0.886, -0.16);
const ARM: (f64, f64, f64) = (0.17, 0.9, -0.31); // the tone arm's pivot (x, height, depth)

pub fn gramophone() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (rx, ry, rz) = RECORD;
    let still = new_parts(|| {
        fbox!(
            f,
            "cabinet",
            0.62,
            0.66,
            0.4,
            (0, 0.38, WALL + 0.21),
            M.cabinet,
            bevel = 0.008,
            bevel_segments = 1
        );
        kit::moulding!(
            f,
            "cabinet_top",
            &[
                (-0.33, WALL),
                (0.33, WALL),
                (0.33, WALL + 0.44),
                (-0.33, WALL + 0.44)
            ],
            &[
                (0.0, 0.0),
                (0.02, 0.0),
                (0.02, 0.03),
                (0.006, 0.03),
                (0.0, 0.02)
            ],
            M.walnut,
            at = 0.71,
            plane = Facing::Top
        );
        fbox!(
            f,
            "cabinet_top_board",
            0.62,
            0.03,
            0.4,
            (0, 0.725, WALL + 0.22),
            M.walnut
        );
        for s in [-1, 1] {
            let sf = s as f64;
            kit::shaker!(
                f,
                format!("cabinet_door{s}"),
                0.28,
                0.5,
                (sf * 0.15, 0.4, WALL + 0.416),
                0.012,
                M.door,
                rail = 0.045,
                panel = M.cabinet,
                raised = true
            );
            kit::knob!(
                f,
                format!("door_knob{s}"),
                sf * 0.03,
                0.42,
                WALL + 0.422,
                0.011,
                M.gold,
                segments = 8
            );
        }
        for x in [-0.27, 0.27] {
            for z in [WALL + 0.06, WALL + 0.36] {
                revolve!(
                    format!("foot{x}{z}"),
                    &[
                        (0.0, 0.0),
                        (0.02, 0.0),
                        (0.03, 0.03),
                        (0.025, 0.05),
                        (0.0, 0.05)
                    ],
                    at = f.at(x, 0, z),
                    material = M.gold,
                    segments = 10
                );
            }
        }
        // the gramophone: a walnut case with brass corners and band, its crank; the horn on a swept neck from the back
        fbox!(
            f,
            "case",
            0.38,
            0.13,
            0.36,
            (0, 0.805, -0.18),
            M.walnut,
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(f, "case_band", 0.385, 0.02, 0.365, (0, 0.82, -0.18), M.gold);
        for x in [-0.19, 0.19] {
            for z in [-0.36, 0.0] {
                fbox!(
                    f,
                    format!("case_corner{x}{z}"),
                    0.022,
                    0.13,
                    0.022,
                    (x, 0.805, z),
                    M.gold,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
        }
        revolve!(
            "crank_boss",
            &[
                (0.0, 0.0),
                (0.024, 0.0),
                (0.024, 0.01),
                (0.014, 0.02),
                (0.0, 0.02)
            ],
            at = f.at(-0.19, 0.8, -0.18),
            material = M.gold,
            segments = 10,
            rot = (0, 90, 0)
        );
        fsweep!(
            f,
            "crank",
            &kit::fillet!(
                &[
                    (-0.205, 0.8, -0.18),
                    (-0.24, 0.8, -0.18),
                    (-0.24, 0.74, -0.15),
                    (-0.27, 0.74, -0.15)
                ],
                0.012
            ),
            0.005,
            M.gold,
            verts = 5
        );
        fsweep!(
            f,
            "crank_handle",
            &[(-0.27, 0.74, -0.15), (-0.31, 0.74, -0.15)],
            0.009,
            M.walnut,
            verts = 6
        );
        revolve!(
            "arm_base",
            &[
                (0.0, 0.0),
                (0.032, 0.0),
                (0.03, 0.012),
                (0.018, 0.02),
                (0.016, 0.03),
                (0.0, 0.03)
            ],
            at = f.at(ARM.0, 0.87, ARM.2),
            material = M.gold,
            segments = 12
        );
        fsweep!(
            f,
            "horn_neck",
            &[
                (0.0, 0.85, -0.3),
                (0.0, 0.92, -0.36),
                (0.0, 0.98, -0.35),
                (0.0, 1.02, -0.32)
            ],
            0.022,
            M.horn,
            verts = 10,
            resolution = 3,
            scale = [1.0, 0.9, 0.8, 0.75]
        );
        let horn = [
            (0.02, 0.0),
            (0.03, 0.08),
            (0.06, 0.2),
            (0.12, 0.32),
            (0.23, 0.42),
            (0.24, 0.43),
            (0.225, 0.43),
            (0.115, 0.335),
            (0.055, 0.215),
            (0.025, 0.09),
            (0.012, 0.0),
        ];
        revolve!(
            "horn",
            &eased(&horn, AUTO_WIDTH),
            at = f.at(0.0, 1.01, -0.32), // (2026-11-25: 4 cm lower, a gramophone's height)
            material = M.horn,
            segments = 28,
            rot = (34, 0, 0)
        );
        let h0 = f.at(0.0, 1.01, -0.32);
        let tilt =
            M4::translation(V3::from((h0.x, h0.y, h0.z))) * M4::rotation(radians(34.0), Axis::X);
        for k in 0..8 {
            // the seams between the horn's petals
            let a = TAU * (k as f64 + 0.5) / 8.0;
            let pts: Vec<V3> = horn[..6]
                .iter()
                .map(|&(r, h)| tilt * V3::from((r * cos(a) * 1.01, r * sin(a) * 1.01, h)))
                .collect();
            sweep!(
                format!("horn_seam{k}"),
                &pts,
                0.0035,
                material = M.gold,
                verts = 4
            );
        }
        // a rose on its top (the record sleeves leant on the cabinet: below)
        fsweep!(
            f,
            "rose_stem",
            &[
                (-0.22, 0.745, -0.06),
                (-0.16, 0.746, -0.03),
                (-0.1, 0.745, 0.0)
            ],
            0.004,
            M.leaf,
            verts = 5,
            resolution = 2
        );
        _rose(
            f,
            "cabinet_rose",
            -0.24,
            0.742,
            -0.07,
            0.05,
            None,
            7,
            70.0,
            240.0,
        );
        _rose_leaves(f, "rose_leaf", -0.15, 0.748, -0.025, 2, 0.04, 0.0, None);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the record sleeves leant on the cabinet: drawn with it, not the gramophone's own size
    let sleeves = new_parts(|| {
        fbox!(
            f,
            "sleeve_a",
            0.3,
            0.3,
            0.012,
            (0.22, 0.15, 0.2),
            M.sleeve_a,
            rot = (-12, 20, 0)
        );
        fbox!(
            f,
            "sleeve_b",
            0.3,
            0.3,
            0.012,
            (0.2, 0.15, 0.22),
            M.sleeve_b,
            rot = (-18, 14, 0)
        );
        fcyl!(
            f,
            "sleeve_disc",
            0.07,
            0.004,
            (0.22, 0.155, 0.208),
            M.paper,
            rot = (-12 + 90, 20, 0),
            verts = 14
        );
    });
    aside(&sleeves, "sleeves");
    // The platter and record, origin on the spindle: `platter` turns them while the record plays.
    let platter = new_parts(|| {
        fcyl!(
            f,
            "platter",
            0.15,
            0.012,
            (rx, ry - 0.008, rz),
            M.gold,
            verts = 32
        );
        let mut grooves = vec![(0.0, 0.0), (0.145, 0.0)];
        for k in 0..7 {
            for j in [0, 1] {
                let kf = k as f64;
                grooves.push((
                    0.145 - 0.012 * kf - if j != 0 { 0.006 } else { 0.0 },
                    if j != 0 { 0.006 } else { 0.0075 },
                ));
            }
        }
        grooves.extend([(0.05, 0.007), (0.0, 0.007)]);
        revolve!(
            "record",
            &grooves,
            at = f.at(rx, ry - 0.002, rz),
            material = M.record,
            segments = 32
        );
        fcyl!(
            f,
            "label",
            0.05,
            0.009,
            (rx, ry + 0.003, rz),
            M.rose,
            verts = 20
        );
        fbox!(
            f,
            "label_mark",
            0.04,
            0.0095,
            0.008,
            (rx + 0.02, ry + 0.003, rz),
            M.paper
        );
        fcyl!(
            f,
            "spindle",
            0.005,
            0.02,
            (rx, ry + 0.01, rz),
            M.gold,
            verts = 6
        );
    });
    let platter = hinged(join(&platter, "platter", None), f.at(rx, ry, rz));
    // The tone arm, resting off the record beside it: `needle` swings it onto the record about its pivot.
    let (ax, ay, az) = ARM;
    let arm = new_parts(|| {
        revolve!(
            "arm_pivot",
            &[
                (0.0, 0.0),
                (0.02, 0.0),
                (0.02, 0.03),
                (0.014, 0.04),
                (0.0, 0.042)
            ],
            at = f.at(ax, ay - 0.012, az),
            material = M.gold,
            segments = 10
        );
        fsweep!(
            f,
            "arm_tube",
            &[
                (ax, ay + 0.02, az),
                (ax + 0.012, ay + 0.03, az + 0.1),
                (ax + 0.02, ay + 0.03, az + 0.2)
            ],
            0.008,
            M.gold,
            verts = 8,
            resolution = 3,
            scale = [1.0, 0.85, 0.7]
        );
        revolve!(
            "sound_box",
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.032, 0.006),
                (0.032, 0.014),
                (0.03, 0.018),
                (0.0, 0.018)
            ],
            at = f.at(ax + 0.02, ay + 0.025, az + 0.212),
            material = M.gold,
            segments = 14,
            rot = (90, 0, 0)
        );
        fcyl!(
            f,
            "sound_face",
            0.024,
            0.004,
            (ax + 0.02, ay + 0.025, az + 0.231),
            M.paper,
            rot = (90, 0, 0),
            verts = 12
        );
        rod!(
            "stylus",
            f.at(ax + 0.02, ay + 0.02, az + 0.22),
            f.at(ax + 0.02, ry + 0.006, az + 0.22),
            0.002,
            material = M.chrome,
            verts = 4
        );
    });
    let arm = hinged(join(&arm, "tone_arm", None), f.at(ax, ay, az));
    let objs = settle!(centre = false);
    swing!(arm, "needle", 30.0, seconds = 0.8);
    spin!(platter, "platter", turns = 1.0, seconds = 1.8);
    objs
}
