//! L1 continued: the designer sofa, the three-seater, the sectional's modules and its chaise.

use super::common::{Cushion, cushion, panel, plain, seat, settled, swept, taper_leg};
use super::sofas::run;
use super::{F, M};
use artkit::forms::{catmull, rounded_rect};
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, V3};
use artkit::parts::{join, new_parts};
use artkit::scene::Obj;
use artkit::soft::SoftBox;
use artkit::{fcyl, fsweep};

// (0.318 is a depth, not 1/pi)
#[allow(clippy::approx_constant)]
/// The designer sofa (the old `design_sofa`): a slim deck on splayed brass legs, a tilted back with buttons, slim
/// arms with brass caps and a brass rail along the front.
pub fn sofa_design() -> Vec<Obj> {
    let z = -0.04;
    let still = new_parts(|| {
        panel(
            "deck",
            1.84,
            0.1,
            0.8,
            (0.0, 0.27, z),
            M.teal,
            0.03,
            0.0,
            Some(0.005),
            plain(0.03),
        );
        // the back: one buttoned panel, tilted, tufted in two staggered rows, a covered button in each dent
        panel(
            "back",
            1.84,
            0.4,
            0.14,
            (0.0, 0.52, z - 0.318),
            M.teal,
            0.05,
            0.014,
            Some(0.006),
            SoftBox {
                rot: P3::from((-8, 0, 0)),
                tufts: Some((8, 2)),
                tuft: 0.014,
                diamond: true,
                button: Some(M.teal),
                resting: true,
                ..SoftBox::default()
            },
        );
        // a brass rail along the front, its ends turning back into the arms
        fsweep!(
            F,
            "front_rail",
            &[
                (-0.84, 0.22, z + 0.33),
                (-0.84, 0.22, z + 0.38),
                (-0.8, 0.22, z + 0.395),
                (0.8, 0.22, z + 0.395),
                (0.84, 0.22, z + 0.38),
                (0.84, 0.22, z + 0.33)
            ],
            0.013,
            M.brass,
            verts = 8,
            resolution = 2
        );
        for s in [-1, 1] {
            let sf = f64::from(s);
            panel(
                &format!("arm{s}"),
                0.1,
                0.32,
                0.8,
                (sf * 0.88, 0.38, z),
                M.teal,
                0.04,
                0.0,
                Some(0.005),
                plain(0.04),
            );
            // the arm cap: a brass strip rounded over the arm, its front end rolled down
            // straight to the roll, curved over it (one span 60 cm long would overshoot the roll and fold it)
            let given = [
                (sf * 0.88, 0.556, z + 0.24),
                (sf * 0.88, 0.556, z + 0.3),
                (sf * 0.88, 0.55, z + 0.36),
                (sf * 0.88, 0.53, z + 0.375),
            ]
            .map(|(x, y, z)| V3::from(F.at(x, y, z)));
            let mut pts = vec![V3::from(F.at(sf * 0.88, 0.555, z - 0.33))];
            pts.extend(catmull(&given, 2, false).0);
            swept(
                &format!("arm_cap{s}"),
                &pts,
                &rounded_rect(0.11, 0.022, 0.01, 2),
                &vec![1.0; pts.len()],
                V3::from((1, 0, 0)),
                60.0,
                0.003,
                M.brass,
            );
            for lz in [z - 0.3, z + 0.3] {
                let dz = if lz > z { 0.035 } else { -0.035 };
                let at = format!("{s}{}", float_repr(lz));
                // a splayed brass leg tapering to a point foot, a cup where it meets the deck
                fsweep!(
                    F,
                    format!("leg{at}"),
                    &[
                        (sf * 0.78, 0.23, lz),
                        (sf * 0.79, 0.12, lz + dz * 0.5),
                        (sf * 0.8, 0.008, lz + dz),
                        (sf * 0.8, 0.0, lz + dz)
                    ],
                    0.019,
                    M.brass_legs,
                    verts = 10,
                    scale = [1.0, 0.8, 0.55, 0.5]
                );
                fcyl!(
                    F,
                    format!("leg_cup{at}"),
                    0.026,
                    0.02,
                    (sf * 0.78, 0.225, lz),
                    M.brass_legs,
                    verts = 12,
                    bevel = 0.004
                );
            }
        }
    });
    join(&still, "sofa", Some((0, 0, 0).into()));
    let cushions = || {
        for x in [-0.42, 0.42] {
            cushion(
                &format!("seat{}", float_repr(x)),
                0.82,
                0.11,
                0.66,
                (x, 0.37, z + 0.06),
                M.teal_cushion,
                Cushion {
                    puff: 0.014,
                    keep_top: true,
                    ..Cushion::default()
                },
            );
        }
        0.425
    };
    settled(seat(cushions, 0.315, z + 0.06))
}

pub fn sofa_three_seat() -> Vec<Obj> {
    settled(run(
        M.moss,
        M.moss_cushion,
        M.oak_legs,
        2.86,
        &[-1, 1],
        &["back"],
        0.88,
        -0.04,
        Some(3),
        &[
            (-1.0, -0.08, M.pillow_cream, 14),
            (1.0, -0.08, M.pillow_mustard, -14),
        ],
    ))
}

/// A module of the sectional: `arms`, `backs`, its depth, where it stands and its pillows.
pub(super) fn sectional(
    arms: &[i32],
    backs: &[&str],
    depth: f64,
    z: f64,
    pillows: &[(f64, f64, Mat, i32)],
) -> Vec<Obj> {
    settled(run(
        M.greige,
        M.greige_cushion,
        M.espresso_legs,
        1.0,
        arms,
        backs,
        depth,
        z,
        None,
        pillows,
    ))
}

pub fn sectional_middle() -> Vec<Obj> {
    sectional(&[], &["back"], 0.88, -0.04, &[])
}

pub fn sectional_arm_left() -> Vec<Obj> {
    sectional(&[1], &["back"], 0.88, -0.04, &[])
}

pub fn sectional_arm_right() -> Vec<Obj> {
    sectional(&[-1], &["back"], 0.88, -0.04, &[])
}

pub fn sectional_corner() -> Vec<Obj> {
    sectional(
        &[],
        &["back", "right"],
        0.89,
        0.015,
        &[(0.1, -0.03, M.pillow_mustard, -35)],
    )
}

/// The sectional's chaise (1 x 2): a long seat with a back at the wall and an arm along +x, a throw over its foot.
pub fn chaise() -> Vec<Obj> {
    let still = new_parts(|| {
        panel(
            "base",
            0.96,
            0.22,
            1.9,
            (0.0, 0.21, 0.0),
            M.greige,
            0.035,
            0.0,
            Some(0.005),
            plain(0.035),
        );
        panel(
            "back",
            0.96,
            0.5,
            0.2,
            (0.0, 0.57, -0.85),
            M.greige,
            0.06,
            0.0,
            None,
            plain(0.06),
        );
        panel(
            "arm",
            0.17,
            0.52,
            1.9,
            (0.395, 0.36, 0.0),
            M.greige,
            0.065,
            0.0,
            Some(0.006),
            plain(0.065),
        );
        cushion(
            "back_cushion",
            0.74,
            0.4,
            0.16,
            (-0.08, 0.62, -0.68),
            M.greige_cushion,
            Cushion {
                rot: P3::from((-9, 0, 0)),
                puff: 0.03,
                ..Cushion::default()
            },
        );
        for lx in [-0.42, 0.42] {
            for lz in [-0.88, 0.88] {
                taper_leg(
                    &format!("leg{}{}", float_repr(lx), float_repr(lz)),
                    lx,
                    lz,
                    0.1,
                    M.espresso_legs,
                    0.022,
                    None,
                );
            }
        }
    });
    join(&still, "sofa", Some((0, 0, 0).into()));
    let cushions = || {
        cushion(
            "seat",
            0.76,
            0.13,
            1.66,
            (-0.08, 0.385, 0.1),
            M.greige_cushion,
            Cushion {
                puff: 0.02,
                keep_top: true,
                ..Cushion::default()
            },
        );
        kit::throw!(
            F,
            "throw",
            -0.06,
            0.52,
            0.469,
            0.36,
            0.935,
            M.throw_oat,
            0.1,
            turn = -3.0,
            seed = 5.0
        );
        0.45
    };
    settled(seat(cushions, 0.32, 0.1))
}
