//! D2: the fireplace.

use super::common::{F, M, WALL, disc, rect, shape};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::kit;
use artkit::math::{P3, PI, V3, sin, v3};
use artkit::parts::{join, new_parts, set_origin};
use artkit::place::Facing;
use artkit::scene::{Obj, RotationMode, meshes};
use artkit::{cylinder, fbox, fextrude, fsweep, grow, jitter, loft, revolve, ring, settle, sweep};

const HEARTH: f64 = 0.42; // the raised hearth's top: the content's `seat` height, where residents warming up sit
const MANTEL_Z: f64 = WALL + 0.38; // the mantel's things stand here, in front of the flue (its front WALL + 0.28)
// its front edge, game depth: 0.46 m in front of the firebox's opening (at the tile's middle), so a resident
// sitting on its edge (content `seat.forward` 0.36) has the fire behind her
const HEARTH_FRONT: f64 = 0.45;

/// A split log from game point `a` to `b`: a barked, slightly crooked round, its cut ends pale.
fn log(name: &str, a: (f64, f64, f64), b: (f64, f64, f64), r: f64, seed: f64) {
    let (va, vb) = (V3::from(F.at(a.0, a.1, a.2)), V3::from(F.at(b.0, b.1, b.2)));
    let mid = (va + vb) / 2 + v3(0.0, 0.0, r * 0.15);
    let l = sweep!(
        name,
        &[va, mid, vb],
        r,
        material = M.log,
        verts = 10,
        resolution = 2,
        scale = [1.0, 1.06, 0.94]
    );
    jitter!(l, r * 0.12, size = 0.03, seed = seed);
    let d = (vb - va).normalized();
    for (k, (p, s)) in [(va, -1), (vb, 1)].into_iter().enumerate() {
        let end = cylinder!(
            format!("{name}_end{k}"),
            r * 0.86,
            0.004,
            material = M.log_end,
            verts = 10
        );
        end.edit(|o| o.loc = p + d * s * 0.001);
        end.set_rotation_mode(RotationMode::Quaternion);
        end.set_rotation_quaternion(d.to_track_quat(artkit::math::Axis::Z, artkit::math::Axis::Y));
    }
}

/// A painted chimney breast with fielded panels and a crown moulding, an oak mantel shelf with a moulded edge on
/// scroll corbels, a moulded surround with corner blocks and a keystone round a firebox lined in sooty brick, a raised
/// hearth bench of stone slabs, an iron grate with finials holding split logs, and things on the mantel. The flames
/// grow on the logs while someone warms up by it.
pub fn fireplace() -> Vec<Obj> {
    let still = new_parts(|| {
        // the chimney breast, built round the firebox's opening (x ±0.44, from the hearth bench up 0.66 m)
        let top = HEARTH + 0.66;
        for side in [-1, 1] {
            fbox!(
                F,
                format!("breast{side}"),
                0.48,
                1.25,
                0.4,
                (side as f64 * 0.68, 0.625, WALL + 0.2),
                M.surround,
                bevel = 0.012
            );
        }
        fbox!(
            F,
            "breast_top",
            0.9,
            1.25 - top,
            0.4,
            (0, (1.25 + top) / 2.0, WALL + 0.2),
            M.surround,
            bevel = 0.012
        );
        fbox!(
            F,
            "firebox_back",
            0.9,
            top - HEARTH + 0.02,
            0.1,
            (0, (top + HEARTH) / 2.0, WALL + 0.05),
            M.soot
        );
        // sooty brick lining the back, in stretcher bond
        for row in 0..10 {
            let y = HEARTH + 0.03 + row as f64 * 0.065;
            let off = if row % 2 == 1 { 0.1 } else { 0.0 };
            for i in -3..4 {
                let (x0, x1) = (
                    (-0.43_f64).max(i as f64 * 0.2 + off - 0.096),
                    0.43_f64.min(i as f64 * 0.2 + off + 0.096),
                );
                if x1 - x0 > 0.03 && y < top - 0.02 {
                    fbox!(
                        F,
                        format!("brick{row}_{i}"),
                        x1 - x0,
                        0.058,
                        0.01,
                        ((x0 + x1) / 2.0, y, WALL + 0.105),
                        M.brick
                    );
                }
            }
        }
        for side in [-1, 1] {
            fbox!(
                F,
                format!("firebox_side{side}"),
                0.012,
                top - HEARTH,
                0.3,
                (side as f64 * 0.434, (top + HEARTH) / 2.0, WALL + 0.25),
                M.firebox
            );
        }
        fbox!(
            F,
            "firebox_top",
            0.88,
            0.012,
            0.3,
            (0, top - 0.006, WALL + 0.25),
            M.firebox
        );
        fbox!(
            F,
            "flue",
            1.6,
            1.55,
            0.28,
            (0, 2.025, WALL + 0.14),
            M.surround,
            bevel = 0.012
        );
        kit::moulding!(
            F,
            "crown",
            &[
                (-0.8, WALL + 0.02),
                (0.8, WALL + 0.02),
                (0.8, WALL + 0.28),
                (-0.8, WALL + 0.28)
            ],
            &crate::common::eased_loop(
                &[
                    (0.03, 0.0),
                    (0.0, 0.0),
                    (-0.006, 0.01),
                    (-0.014, 0.022),
                    (-0.018, 0.04),
                    (0.03, 0.04)
                ],
                0.003
            ),
            M.surround,
            at = 2.7,
            plane = Facing::Top
        );
        for side in [-1, 1] {
            // fielded panels on the breast either side of the opening
            fextrude!(
                F,
                format!("panel{side}"),
                &rounded_rect(0.3, 0.64, 0.02, 2),
                0.012,
                (side as f64 * 0.7, 0.8, WALL + 0.402),
                M.surround,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        fextrude!(
            F,
            "panel_flue",
            &rounded_rect(1.1, 1.0, 0.03, 2),
            0.012,
            (0, 2.0, WALL + 0.282),
            M.surround,
            bevel = 0.005,
            bevel_segments = 1
        );
        // the mantel: an oak shelf with a rounded nosing and a cove under it, on two scroll corbels
        let shelf = smooth_outline(
            &[
                (-0.24, -0.031),
                (-0.236, -0.035), // its back on the wall eased too
                (0.19, -0.035),
                (0.205, -0.028),
                (0.215, -0.012),
                (0.232, -0.002),
                (0.24, 0.012),
                (0.24, 0.028),
                (0.233, 0.035),
                (-0.236, 0.035),
                (-0.24, 0.031),
            ],
            1,
            true,
        );
        fextrude!(
            F,
            "mantel",
            &shelf,
            1.96,
            (0, 1.285, WALL + 0.24),
            M.oak,
            plane = Facing::Side,
            bevel = 0.003,
            bevel_segments = 1
        );
        for side in [-1, 1] {
            let corbel = smooth_outline(
                &[
                    (-0.05, 0.04),
                    (0.03, 0.04),
                    (0.035, 0.0),
                    (0.012, -0.02),
                    (0.02, -0.045),
                    (0.0, -0.06),
                    (-0.02, -0.05),
                    (-0.05, -0.06),
                ],
                1,
                true,
            );
            fextrude!(
                F,
                format!("corbel{side}"),
                &corbel,
                0.1,
                (side as f64 * 0.8, 1.21, WALL + 0.41),
                M.oak,
                plane = Facing::Side
            );
        }
        // the hearth: a stone bench from the wall out to 0.46 m in front of the opening, the firebox's floor, its top
        // three slabs with a nosing; residents warming up sit on its front edge, their backs to the fire (a real raised
        // hearth runs 0.4-0.5 m out)
        fbox!(
            F,
            "hearth",
            1.94,
            HEARTH - 0.04,
            HEARTH_FRONT - WALL - 0.01,
            (
                0,
                (HEARTH - 0.04) / 2.0,
                (HEARTH_FRONT + WALL) / 2.0 - 0.005
            ),
            M.hearth,
            bevel = 0.01
        );
        for (i, (x0, x1)) in [(-0.98, -0.33), (-0.33, 0.33), (0.33, 0.98)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                F,
                format!("slab{i}"),
                x1 - x0 - 0.004,
                0.04,
                HEARTH_FRONT - WALL,
                ((x0 + x1) / 2.0, HEARTH - 0.02, (HEARTH_FRONT + WALL) / 2.0),
                M.hearth,
                bevel = 0.008,
                bevel_segments = 2
            );
        }
        // the surround: moulded jambs and lintel, square corner blocks with rosettes, a keystone
        let jamb = smooth_outline(
            &[
                (-0.047, 0.0),
                (0.047, 0.0), // its back on the breast eased
                (0.05, 0.003),
                (0.05, 0.02),
                (0.035, 0.03),
                (-0.035, 0.03),
                (-0.05, 0.02),
                (-0.05, 0.003),
            ],
            1,
            true,
        );
        let jamb_profile: Vec<(f64, f64)> = jamb.iter().map(|&(u, v)| (v, u)).collect();
        for side in [-1, 1] {
            let sf = side as f64;
            fsweep!(
                F,
                format!("surround_jamb{side}"),
                &[
                    (sf * 0.49, HEARTH, WALL + 0.4),
                    (sf * 0.49, top, WALL + 0.4)
                ],
                0,
                M.surround,
                profile = jamb_profile.clone(),
                up = (0, -1, 0)
            );
            fbox!(
                F,
                format!("corner_block{side}"),
                0.12,
                0.12,
                0.034,
                (sf * 0.49, top + 0.05, WALL + 0.417),
                M.surround,
                bevel = 0.006,
                bevel_segments = 1
            );
            disc(
                &format!("rosette{side}"),
                sf * 0.49,
                top + 0.05,
                WALL + 0.434,
                0.035,
                M.surround,
                0.008,
                16,
            );
        }
        fsweep!(
            F,
            "surround_lintel",
            &[
                (-0.43, top + 0.05, WALL + 0.4),
                (0.43, top + 0.05, WALL + 0.4)
            ],
            0,
            M.surround,
            profile = jamb_profile,
            up = (0, -1, 0)
        );
        fextrude!(
            F,
            "keystone",
            &[
                (-0.035, -0.06),
                (0.035, -0.06),
                (0.045, 0.06),
                (-0.045, 0.06)
            ],
            0.04,
            (0, top + 0.05, WALL + 0.42),
            M.surround,
            bevel = 0.005,
            bevel_segments = 1
        );
        // the grate and the logs
        let y = HEARTH;
        for s in [-1, 1] {
            let sf = s as f64;
            kit::tube!(
                F,
                format!("grate_bar{s}"),
                &[
                    (-0.3, y + 0.05, WALL + 0.25 + sf * 0.08),
                    (0.3, y + 0.05, WALL + 0.25 + sf * 0.08)
                ],
                0.007,
                M.grate,
                verts = 6,
                round = 1
            );
            kit::tube!(
                F,
                format!("grate_leg{s}"),
                &[
                    (sf * 0.28, y, WALL + 0.16),
                    (sf * 0.28, y + 0.05, WALL + 0.17),
                    (sf * 0.28, y + 0.05, WALL + 0.33),
                    (sf * 0.28, y + 0.12, WALL + 0.37)
                ],
                0.007,
                M.grate,
                verts = 6,
                round = 2
            );
            revolve!(
                format!("grate_finial{s}"),
                &[
                    (0.0, 0.0),
                    (0.012, 0.0),
                    (0.016, 0.012),
                    (0.01, 0.024),
                    (0.014, 0.034),
                    (0.0, 0.044)
                ],
                at = F.at(sf * 0.28, y + 0.12, WALL + 0.37),
                material = M.grate,
                segments = 10
            );
        }
        for k in 0..5 {
            let x = -0.22 + 0.11 * k as f64;
            kit::tube!(
                F,
                format!("grate_slat{k}"),
                &[(x, y + 0.05, WALL + 0.17), (x, y + 0.05, WALL + 0.33)],
                0.005,
                M.grate,
                verts = 5,
                round = 1
            );
        }
        log(
            "log0",
            (-0.25, y + 0.09, WALL + 0.21),
            (0.2, y + 0.09, WALL + 0.29),
            0.045,
            0.3,
        );
        log(
            "log1",
            (-0.18, y + 0.09, WALL + 0.31),
            (0.26, y + 0.09, WALL + 0.2),
            0.042,
            1.7,
        );
        log(
            "log2",
            (-0.12, y + 0.16, WALL + 0.25),
            (0.14, y + 0.17, WALL + 0.26),
            0.038,
            2.9,
        );
        // on the mantel
        let top = 1.32;
        revolve!(
            "vase",
            &crate::common::eased(
                &[
                    (0.0, 0.0),
                    (0.036, 0.0),
                    (0.04, 0.006),
                    (0.055, 0.07),
                    (0.05, 0.12),
                    (0.034, 0.18),
                    (0.026, 0.215),
                    (0.03, 0.232),
                    (0.034, 0.24),
                    (0.026, 0.24),
                    (0.0, 0.22)
                ],
                0.002
            ),
            at = F.at(-0.65, top, MANTEL_Z),
            material = M.pot,
            segments = 20
        );
        for (k, (x, h)) in [(-0.4, 0.12), (-0.3, 0.18)].into_iter().enumerate() {
            revolve!(
                format!("candle_holder{k}"),
                &[
                    (0.0, 0.0),
                    (0.04, 0.0),
                    (0.04, 0.006),
                    (0.016, 0.014),
                    (0.012, 0.03),
                    (0.018, 0.04),
                    (0.03, 0.044),
                    (0.03, 0.05),
                    (0.0, 0.05)
                ],
                at = F.at(x, top, MANTEL_Z),
                material = M.brass,
                segments = 14
            );
            revolve!(
                format!("candle{k}"),
                &[
                    (0.0, 0.0),
                    (0.024, 0.0),
                    (0.024, h - 0.006),
                    (0.02, h),
                    (0.004, h + 0.002),
                    (0.0, h + 0.002)
                ],
                at = F.at(x, top + 0.05, MANTEL_Z),
                material = M.candle,
                segments = 12
            );
            kit::tube!(
                F,
                format!("wick{k}"),
                &[
                    (x, top + 0.05 + h, MANTEL_Z),
                    (x, top + 0.062 + h, MANTEL_Z)
                ],
                0.0015,
                M.soot,
                verts = 4,
                round = 1
            );
        }
        let pic = new_parts(|| {
            kit::moulding!(
                F,
                "picture_frame",
                &rect(0.3, 0.38, top + 0.19, 0.55),
                &crate::common::eased_loop(
                    &[
                        (0.0, 0.0),
                        (0.0, 0.02),
                        (0.005, 0.025),
                        (0.03, 0.025),
                        (0.035, 0.018),
                        (0.035, 0.0)
                    ],
                    0.003
                ),
                M.frame_white,
                at = WALL + 0.3
            );
            fbox!(
                F,
                "picture_art",
                0.24,
                0.32,
                0.004,
                (0.55, top + 0.19, WALL + 0.304),
                M.picture
            );
            shape(
                "picture_hill",
                &[
                    (0.43, top + 0.04),
                    (0.67, top + 0.04),
                    (0.67, top + 0.12),
                    (0.6, top + 0.16),
                    (0.5, top + 0.11),
                    (0.43, top + 0.14),
                ],
                WALL + 0.306,
                M.olive,
                0.001,
                0,
            );
        });
        let art = join(&pic, "picture", None);
        set_origin(art, F.at(0.55, top, WALL + 0.31));
        art.set_rotation_euler_axis(0, artkit::math::radians(-3.0));
    });
    join(&still, "body", Some(P3::ZERO));

    // the flames: an ember bed and tongues of fire, each lofted narrowing and leaning as it rises, joined at the logs'
    // top, grown from nothing
    let base = HEARTH + 0.12;
    let parts = new_parts(|| {
        fbox!(
            F,
            "embers",
            0.5,
            0.02,
            0.16,
            (0, base, WALL + 0.25),
            M.ember,
            bevel = 0.008
        );
        for (k, (x, h, r, z, mat)) in [
            (-0.14, 0.26, 0.07, 0.24, M.flame),
            (0.0, 0.36, 0.09, 0.26, M.flame),
            (0.13, 0.28, 0.07, 0.25, M.flame),
            (-0.06, 0.2, 0.05, 0.3, M.flame),
            (0.07, 0.22, 0.05, 0.21, M.flame),
            (-0.07, 0.17, 0.045, 0.24, M.flame_core),
            (0.06, 0.21, 0.05, 0.24, M.flame_core),
        ]
        .into_iter()
        .enumerate()
        {
            let b = F.at(x, 0, WALL + z);
            let (bx, by) = (b.x, b.y);
            let lean = 0.02 * if k % 2 == 1 { 1.0 } else { -1.0 };
            let secs: Vec<Vec<P3>> = [
                (0.0, 1.0),
                (0.2, 0.95),
                (0.45, 0.7),
                (0.7, 0.4),
                (0.9, 0.14),
                (1.0, 0.02),
            ]
            .iter()
            .map(|&(t, w)| {
                ring!(
                    r * w,
                    r * w * 0.45,
                    z = base + h * t,
                    verts = 10,
                    at = (bx + lean * sin(PI * t) * (1.0 + t), by)
                )
            })
            .collect();
            loft!(format!("flame{k}"), &secs, material = mat);
        }
    });
    let flames = join(&parts, "flames", Some(F.at(0, base, WALL + 0.25)));
    flames.set_scale_axis(2, 0.001);
    settle!(centre = false);
    grow!(flames, "flames", seconds = 0.8);
    meshes()
}
