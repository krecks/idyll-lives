//! The rest: the wall bars, the altar, the lockers, the mannequin, the luggage trolley and the vault door.

use super::common::{F, M, WALL, open_book, rod, tube};
use artkit::forms::smooth_outline;
use artkit::json::float_repr;
use artkit::math::{P3, TAU, cos, radians, sin};
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, settle, soft, torus};

// --- the wall bars: 1x1 on a wall, beech rungs between two uprights, the top ones standing out -------------------
/// The rungs' depth at height `y`: against the wall low down, standing out over the top.
pub fn bars_z(y: f64) -> f64 {
    if y <= 2.0 {
        return WALL + 0.09;
    }
    WALL + 0.09 + (y - 2.0) / 0.5 * 0.12
}

pub fn wall_bars() -> Vec<Obj> {
    let parts = new_parts(|| {
        let up = smooth_outline(
            &[
                (WALL + 0.02, 0.0),
                (WALL + 0.16, 0.0),
                (WALL + 0.16, 2.0),
                (WALL + 0.28, 2.5),
                (WALL + 0.14, 2.5),
                (WALL + 0.02, 2.0),
            ],
            1,
            true,
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fextrude!(
                F,
                format!("upright{s}"),
                &up,
                0.07,
                (sf * 0.43, 0, 0),
                M.oak,
                plane = Facing::Side,
                bevel = 0.008
            );
            for y in [0.12, 1.2, 2.3] {
                fbox!(
                    F,
                    format!("bracket{s}{}", float_repr(y)),
                    0.09,
                    0.08,
                    0.02,
                    (sf * 0.43, y, WALL + 0.01),
                    M.steel,
                    bevel = 0.003
                );
            }
        }
        for i in 0..16 {
            let y = 0.15 + i as f64 * 0.152;
            crate::eased(fcyl!(
                F,
                format!("rung{i}"),
                0.018,
                0.82,
                (0, y, bars_z(y)),
                M.oak,
                rot = (0, 90, 0),
                verts = 12
            ));
        }
    });
    join(&parts, "bars", Some((0, 0, 0).into()));
    settle!(centre = false, floor = false);
    meshes()
}

// --- the altar: 2x1, stone under a white linen and a frontal, two candlesticks and a cross ------------------------
pub fn candlestick(name: &str, x: f64, y: f64, z: f64) {
    crate::eased(revolve!(
        name,
        &[
            (0.0, 0.0),
            (0.08, 0.0),
            (0.08, 0.015),
            (0.06, 0.03),
            (0.03, 0.05),
            (0.02, 0.1),
            (0.03, 0.12),
            (0.016, 0.14),
            (0.014, 0.3),
            (0.024, 0.32),
            (0.05, 0.34),
            (0.05, 0.35),
            (0.0, 0.35)
        ],
        at = F.at(x, y, z),
        material = M.brass,
        segments = 16
    ));
    crate::eased(fcyl!(
        F,
        format!("{name}_candle"),
        0.022,
        0.24,
        (x, y + 0.35 + 0.12, z),
        M.candle,
        verts = 12
    ));
    fsphere!(
        F,
        format!("{name}_flame"),
        0.012,
        (x, y + 0.6 + 0.018, z),
        M.flame,
        scale = (1, 1, 2.2),
        segments = 8,
        rings = 6
    );
}

pub fn altar() -> Vec<Obj> {
    let parts = new_parts(|| {
        fbox!(
            F,
            "step",
            1.92,
            0.06,
            0.84,
            (0, 0.03, 0),
            M.stone,
            bevel = 0.008
        );
        fbox!(
            F,
            "body",
            1.7,
            0.92,
            0.7,
            (0, 0.06 + 0.46, -0.02),
            M.stone,
            bevel = 0.006
        );
        fbox!(
            F,
            "mensa",
            1.82,
            0.06,
            0.8,
            (0, 1.01, -0.02),
            M.stone,
            bevel = 0.012
        );
        // the fair linen over the top, hanging down the ends
        fbox!(F, "linen", 1.86, 0.008, 0.82, (0, 1.044, -0.02), M.linen);
        for s in [-1, 1] {
            fbox!(
                F,
                format!("linen_drop{s}"),
                0.008,
                0.4,
                0.82,
                (s as f64 * 0.934, 0.85, -0.02),
                M.linen
            );
        }
        // the frontal down the front, a gold cross and fringe on it
        fbox!(
            F,
            "frontal",
            1.3,
            0.8,
            0.01,
            (0, 0.64, 0.344),
            M.frontal,
            bevel = 0.002
        );
        fbox!(
            F,
            "frontal_band",
            1.3,
            0.06,
            0.012,
            (0, 1.01, 0.346),
            M.linen
        );
        fbox!(F, "fringe", 1.3, 0.04, 0.012, (0, 0.26, 0.346), M.brass);
        fbox!(
            F,
            "frontal_cross_v",
            0.06,
            0.4,
            0.004,
            (0, 0.65, 0.351),
            M.brass
        );
        fbox!(
            F,
            "frontal_cross_h",
            0.26,
            0.06,
            0.004,
            (0, 0.72, 0.351),
            M.brass
        );
        let t = 1.048;
        for s in [-1, 1] {
            candlestick(&format!("candlestick{s}"), s as f64 * 0.62, t, -0.12);
        }
        crate::eased(revolve!(
            "cross_base",
            &[
                (0.0, 0.0),
                (0.09, 0.0),
                (0.09, 0.02),
                (0.06, 0.04),
                (0.04, 0.08),
                (0.0, 0.08)
            ],
            at = F.at(0, t, -0.2),
            material = M.brass,
            segments = 16
        ));
        fbox!(
            F,
            "cross_v",
            0.035,
            0.5,
            0.025,
            (0, t + 0.33, -0.2),
            M.brass,
            bevel = 0.005
        );
        fbox!(
            F,
            "cross_h",
            0.26,
            0.035,
            0.025,
            (0, t + 0.45, -0.2),
            M.brass,
            bevel = 0.005
        );
        open_book("missal", 0.3, 0.22, (0.3, t, 0.08), M.book, 8.0, 0.02);
    });
    join(&parts, "altar", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the lockers: 2x1 against a wall, five columns of two, vents, locks and name cards -----------------------------
pub fn locker_row() -> Vec<Obj> {
    let d = 0.45;
    let front = WALL + d;
    let parts = new_parts(|| {
        fbox!(
            F,
            "plinth",
            1.88,
            0.1,
            d - 0.04,
            (0, 0.05, WALL + (d - 0.04) / 2.0),
            M.plinth
        );
        fbox!(
            F,
            "carcass",
            1.9,
            1.72,
            d - 0.02,
            (0, 0.1 + 0.86, WALL + (d - 0.02) / 2.0),
            M.frame,
            bevel = 0.004
        );
        fbox!(
            F,
            "top",
            1.92,
            0.025,
            d,
            (0, 1.8325, WALL + d / 2.0),
            M.frame,
            bevel = 0.005
        );
        for c in 0..5 {
            let x = -0.76 + c as f64 * 0.38;
            for t in 0..2 {
                let y0 = 0.12 + t as f64 * 0.85;
                let y = y0 + 0.415;
                fbox!(
                    F,
                    format!("door{c}{t}"),
                    0.36,
                    0.83,
                    0.016,
                    (x, y, front - 0.012),
                    M.locker,
                    bevel = 0.004
                );
                for v in 0..4 {
                    for yy in [y0 + 0.73, y0 + 0.1] {
                        fbox!(
                            F,
                            format!("vent{c}{t}{v}{}", float_repr(yy)),
                            0.18,
                            0.008,
                            0.004,
                            (x, yy + v as f64 * 0.022 - 0.033, front - 0.003),
                            M.slot
                        );
                    }
                }
                crate::eased(fcyl!(
                    F,
                    format!("lock{c}{t}"),
                    0.014,
                    0.012,
                    (x + 0.13, y, front - 0.002),
                    M.bell,
                    rot = (90, 0, 0),
                    verts = 10
                ));
                fbox!(
                    F,
                    format!("tag_frame{c}{t}"),
                    0.11,
                    0.05,
                    0.004,
                    (x, y0 + 0.62, front - 0.002),
                    M.frame
                );
                fbox!(
                    F,
                    format!("tag{c}{t}"),
                    0.095,
                    0.036,
                    0.004,
                    (x, y0 + 0.62, front - 0.0005),
                    M.label
                );
                fbox!(
                    F,
                    format!("number{c}{t}"),
                    0.04,
                    0.03,
                    0.003,
                    (x, y0 + 0.77, front - 0.001),
                    M.label
                );
            }
        }
    });
    join(&parts, "lockers", Some((0, 0, 0).into()));
    settle!(centre = false);
    meshes()
}

// --- the mannequin: 1x1, a shop dummy in a jacket and trousers on a chrome stand ---------------------------------
pub fn mannequin() -> Vec<Obj> {
    let parts = new_parts(|| {
        crate::eased(revolve!(
            "base",
            &[
                (0.0, 0.0),
                (0.2, 0.0),
                (0.2, 0.008),
                (0.19, 0.014),
                (0.0, 0.016)
            ],
            material = M.bell,
            segments = 32
        ));
        crate::eased(fcyl!(
            F,
            "rod",
            0.012,
            0.6,
            (0, 0.31, -0.12),
            M.bell,
            verts = 10
        ));
        // the legs, in trousers, and shoes
        for s in [-1, 1] {
            let sf = s as f64;
            crate::eased(fsweep!(
                F,
                format!("leg{s}"),
                &[
                    (sf * 0.085, 0.95, -0.01),
                    (sf * 0.09, 0.55, 0.0),
                    (sf * 0.1, 0.1, -0.02)
                ],
                0.07,
                M.cloth2,
                verts = 12,
                resolution = 2,
                scale = [1.0, 0.72, 0.62]
            ));
            soft!(
                F,
                format!("shoe{s}"),
                0.085,
                0.07,
                0.24,
                (sf * 0.1, 0.051, 0.03),
                M.radio,
                round = 0.03,
                puff = 0.01
            );
        }
        // the body: hips to shoulders, in a jacket
        let P3 { x: bx, y: by, .. } = F.at(0, 0, -0.01);
        let sections: Vec<Vec<P3>> = [
            (0.88, 0.17, 0.11),
            (0.98, 0.17, 0.11),
            (1.1, 0.14, 0.095),
            (1.28, 0.165, 0.11),
            (1.4, 0.19, 0.1),
            (1.44, 0.17, 0.09),
            (1.47, 0.07, 0.06),
        ]
        .iter()
        .map(|&(y, rx, rz)| ring!(rx, rz, z = y, verts = 20, square = 2.2, at = (bx, by)))
        .collect();
        crate::eased(loft!("torso", &sections, material = M.cloth1));
        // the shirt in the jacket's open neck, its buttons
        crate::eased(fextrude!(
            F,
            "shirt",
            &[(-0.06, 1.455), (0.06, 1.455), (0.0, 1.27)],
            0.02,
            (0, 0, 0.088),
            M.cloth4,
            plane = Facing::Front
        ));
        for k in 0..3 {
            let kf = k as f64;
            fsphere!(
                F,
                format!("button{k}"),
                0.009,
                (0, 1.2 - kf * 0.08, 0.097 - kf * 0.002),
                M.radio,
                segments = 8,
                rings = 6
            );
        }
        for s in [-1, 1] {
            let sf = s as f64;
            crate::eased(fsweep!(
                F,
                format!("arm{s}"),
                &[
                    (sf * 0.19, 1.4, -0.01),
                    (sf * 0.23, 1.15, 0.0),
                    (sf * 0.24, 0.9, 0.04)
                ],
                0.045,
                M.cloth1,
                verts = 10,
                resolution = 2,
                scale = [1.0, 0.85, 0.75]
            ));
            fsphere!(
                F,
                format!("hand{s}"),
                0.04,
                (sf * 0.245, 0.84, 0.05),
                M.dummy,
                scale = (0.6, 0.8, 1.3),
                segments = 10,
                rings = 8
            );
        }
        crate::eased(fcyl!(
            F,
            "neck",
            0.045,
            0.12,
            (0, 1.5, -0.01),
            M.dummy,
            verts = 12
        ));
        fsphere!(
            F,
            "head",
            0.1,
            (0, 1.64, 0.0),
            M.dummy,
            scale = (0.82, 0.95, 1.15),
            segments = 18,
            rings = 12
        );
    });
    join(&parts, "mannequin", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the luggage trolley: 1x1, a carpeted deck on castors under a brass arch, two cases on it ---------------------
pub fn luggage_trolley() -> Vec<Obj> {
    let deck = 0.17;
    let parts = new_parts(|| {
        fbox!(
            F,
            "deck",
            0.88,
            0.05,
            0.52,
            (0, deck - 0.025, 0),
            M.carpet,
            bevel = 0.006
        );
        fbox!(
            F,
            "deck_rim",
            0.9,
            0.03,
            0.54,
            (0, deck - 0.06, 0),
            M.brass,
            bevel = 0.006
        );
        for x in [-0.38, 0.38] {
            let xs = float_repr(x);
            for z in [-0.2, 0.2] {
                let zs = float_repr(z);
                fbox!(
                    F,
                    format!("fork{xs}{zs}"),
                    0.03,
                    0.06,
                    0.05,
                    (x, deck - 0.09, z),
                    M.brass
                );
                crate::eased(fcyl!(
                    F,
                    format!("wheel{xs}{zs}"),
                    0.045,
                    0.03,
                    (x, 0.045, z),
                    M.rubber,
                    rot = (0, 90, 0),
                    verts = 14
                ));
            }
        }
        // the uprights and the arch, a hanging rail across
        for s in [-1, 1] {
            let sf = s as f64;
            crate::eased(fcyl!(
                F,
                format!("upright{s}"),
                0.016,
                1.45,
                (sf * 0.42, deck + 0.725, 0),
                M.brass,
                verts = 10
            ));
            fsphere!(
                F,
                format!("knob{s}"),
                0.03,
                (sf * 0.42, deck + 1.46, 0),
                M.brass,
                segments = 12,
                rings = 8
            );
            rod(
                &format!("bumper{s}"),
                (-0.42, deck + 0.08, sf * 0.25),
                (0.42, deck + 0.08, sf * 0.25),
                0.012,
                M.brass,
                8,
            );
        }
        let arch: Vec<(f64, f64, i32)> = (0..181)
            .step_by(15)
            .map(|d| radians(d as f64))
            .map(|a| (0.42 * cos(a), deck + 1.45 + 0.2 * sin(a), 0))
            .collect();
        crate::eased(fsweep!(F, "arch", &arch, 0.016, M.brass, verts = 10));
        rod(
            "hanging_rail",
            (-0.42, deck + 1.3, 0),
            (0.42, deck + 1.3, 0),
            0.013,
            M.brass,
            8,
        );
        fsphere!(
            F,
            "crown",
            0.035,
            (0, deck + 1.67, 0),
            M.brass,
            segments = 12,
            rings = 8
        );
        // a case standing, another lying on the deck, a garment bag on the rail
        soft!(
            F,
            "case_big",
            0.5,
            0.62,
            0.24,
            (-0.12, deck + 0.31, 0.06),
            M.case1,
            round = 0.03,
            puff = 0.01
        );
        tube(
            "case_big_handle",
            &[
                (-0.2, deck + 0.62, 0.06),
                (-0.2, deck + 0.66, 0.06),
                (-0.04, deck + 0.66, 0.06),
                (-0.04, deck + 0.62, 0.06),
            ],
            0.01,
            M.rubber,
            0.015,
            8,
        );
        soft!(
            F,
            "case_small",
            0.36,
            0.18,
            0.42,
            (0.22, deck + 0.09, -0.02),
            M.case2,
            round = 0.03,
            puff = 0.01
        );
        tube(
            "case_small_handle",
            &[
                (0.4, deck + 0.07, -0.08),
                (0.43, deck + 0.07, -0.08),
                (0.43, deck + 0.07, 0.04),
                (0.4, deck + 0.07, 0.04),
            ],
            0.009,
            M.rubber,
            0.012,
            8,
        );
        soft!(
            F,
            "garment_bag",
            0.08,
            0.7,
            0.42,
            (0.28, deck + 1.3 - 0.36, -0.03),
            M.radio,
            round = 0.03,
            puff = 0.012
        );
        tube(
            "bag_hook",
            &[
                (0.28, deck + 1.28, -0.03),
                (0.28, deck + 1.34, -0.03),
                (0.28, deck + 1.33, -0.0),
            ],
            0.004,
            M.bell,
            0.015,
            6,
        );
    });
    join(&parts, "trolley", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the vault door: 2x1 on a wall, a round steel door a foot thick in its frame, its wheel and dial -------------
pub fn vault_door() -> Vec<Obj> {
    let c = 1.0; // the door's middle, its height
    let r = 0.74;
    let parts = new_parts(|| {
        fbox!(
            F,
            "frame",
            1.92,
            1.92,
            0.05,
            (0, 0.96, WALL + 0.025),
            M.vault_dark,
            bevel = 0.01
        );
        crate::eased(revolve!(
            "door",
            &[
                (0.0, 0.0),
                (r, 0.0),
                (r, 0.13),
                (r - 0.03, 0.16),
                (r - 0.12, 0.16),
                (r - 0.14, 0.18),
                (0.0, 0.18)
            ],
            at = F.at(0, c, WALL + 0.05),
            material = M.vault,
            segments = 48,
            rot = (90, 0, 0)
        ));
        crate::eased(revolve!(
            "door_rim",
            &[
                (r + 0.06, 0.0),
                (r + 0.06, 0.03),
                (r + 0.03, 0.06),
                (r + 0.005, 0.06),
                (r + 0.005, 0.0)
            ],
            at = F.at(0, c, WALL + 0.05),
            material = M.vault_dark,
            segments = 48,
            rot = (90, 0, 0)
        ));
        for k in 0..16 {
            let a = TAU * k as f64 / 16.0;
            crate::eased(fcyl!(
                F,
                format!("bolt{k}"),
                0.022,
                0.02,
                (0.66 * cos(a), c + 0.66 * sin(a), WALL + 0.215), // standing 1.5 cm off the face (WALL + 0.21)
                M.bell,
                rot = (90, 0, 0),
                verts = 10
            ));
        }
        // the wheel: a hub, six spokes ending in knobs, a ring
        let wz = WALL + 0.29;
        crate::eased(fcyl!(
            F,
            "hub",
            0.08,
            0.08,
            (0, c, WALL + 0.25),
            M.bell,
            rot = (90, 0, 0),
            verts = 20
        ));
        for k in 0..6 {
            let a = TAU * k as f64 / 6.0 + 0.26;
            rod(
                &format!("spoke{k}"),
                (0.06 * cos(a), c + 0.06 * sin(a), wz),
                (0.38 * cos(a), c + 0.38 * sin(a), wz),
                0.014,
                M.bell,
                10,
            );
            fsphere!(
                F,
                format!("spoke_knob{k}"),
                0.026,
                (0.4 * cos(a), c + 0.4 * sin(a), wz),
                M.bell,
                segments = 12,
                rings = 8
            );
        }
        torus!(
            "wheel",
            0.3,
            0.018,
            at = F.at(0, c, wz),
            material = M.bell,
            rot = (90, 0, 0),
            major_segments = 40,
            minor_segments = 8
        );
        // the dial over it
        crate::eased(fcyl!(
            F,
            "dial",
            0.075,
            0.03,
            (0, c + 0.5, WALL + 0.245),
            M.iron,
            rot = (90, 0, 0),
            verts = 24
        ));
        crate::eased(fcyl!(
            F,
            "dial_face",
            0.06,
            0.006,
            (0, c + 0.5, WALL + 0.262),
            M.bell,
            rot = (90, 0, 0),
            verts = 24
        ));
        fbox!(
            F,
            "dial_mark",
            0.008,
            0.03,
            0.004,
            (0, c + 0.59, WALL + 0.232),
            M.leaflet4
        );
        // the hinge on the right
        crate::eased(fcyl!(
            F,
            "hinge",
            0.06,
            0.9,
            (-0.86, c, WALL + 0.13),
            M.vault_dark,
            verts = 16
        ));
        for y in [c - 0.3, c + 0.3] {
            fbox!(
                F,
                format!("hinge_arm{}", float_repr(y)),
                0.2,
                0.12,
                0.08,
                (-0.74, y, WALL + 0.15),
                M.vault_dark,
                bevel = 0.01
            );
        }
    });
    join(&parts, "vault", Some((0, 0, 0).into()));
    settle!(centre = false, floor = false);
    meshes()
}
