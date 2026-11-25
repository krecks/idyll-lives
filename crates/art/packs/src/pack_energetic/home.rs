//! The power blender station and the contrast rain shower.

use super::common::{M, WALL, lofted};
use artkit::forms::{by, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{P3, PI, cosd, sin, sind};
use artkit::motion::{Ease, Grow, Spin, grow, spin};
use artkit::parts::{hinged, join, new_parts, numbered, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{
    fbox, fcone, fcyl, fextrude, fsphere, fsweep, jitter, kit, loft, revolve, ring, settle, slide,
    swing,
};

// --- the power blender station (1x1): a kitchen base unit, the blender, fruit and two smoothies ----------------
const TOP: f64 = 0.9;
const JAR: (f64, f64) = (-0.18, -0.12); // the blender's middle (x, depth)

/// A glass (see-through, like the jug: the same draw) with a drink in it and a straw.
#[allow(clippy::too_many_arguments)]
fn glass(f: &GameFrame, x: f64, z: f64, drink: Mat, y: f64, h: f64, r: f64, straw: Option<Mat>) {
    revolve!(
        numbered("drink"),
        &[
            (0.0, 0.0),
            (r * 0.8, 0.0),
            (r * 0.85, h * 0.72),
            (0.0, h * 0.72)
        ],
        at = f.at(x, y + 0.006, z),
        material = drink,
        segments = 12
    );
    revolve!(
        numbered("glass"),
        &[
            (0.0, 0.0),
            (r * 0.88, 0.0),
            (r, h),
            (r * 0.92, h),
            (r * 0.84, 0.008),
            (0.0, 0.008)
        ],
        at = f.at(x, y, z),
        material = M.jar,
        segments = 14
    );
    if let Some(straw) = straw {
        kit::tube!(
            f,
            numbered("straw"),
            &[
                (x + 0.008, y + 0.02, z),
                (x + 0.02, y + h + 0.05, z - 0.008),
            ],
            0.004,
            straw,
            verts = 5,
            round = 1
        );
    }
}

/// A banana from game point `a` to `b`, curving up by `sag` in the middle, tapering to its ends, a dark stalk.
fn banana(
    f: &GameFrame,
    name: &str,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    sag: f64,
    r: f64,
    material: Mat,
) {
    let mid = (
        (a.0 + b.0) / 2.0 + 0.0,
        (a.1 + b.1) / 2.0 + sag,
        (a.2 + b.2) / 2.0 + 0.0,
    );
    fsweep!(
        f,
        name,
        &[a, mid, b],
        r,
        material,
        verts = 6,
        resolution = 3,
        scale = by(|f| 0.35 + 0.65 * sin(PI * f))
    );
    let tip = (
        b.0 + (b.0 - mid.0) * 0.15,
        b.1 + (b.1 - mid.1) * 0.15,
        b.2 + (b.2 - mid.2) * 0.15,
    );
    fsweep!(
        f,
        format!("{name}_stalk"),
        &[b, tip],
        r * 0.3,
        M.stem,
        verts = 4
    );
}

pub fn power_blender() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (w, h, d) = (0.82, TOP, 0.59);
    let (z0, t) = (WALL, 0.035);
    let zc = z0 + d - 0.035;
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            w - 0.06,
            0.08,
            d - 0.1,
            (0.0, 0.04, z0 + (d - 0.1) / 2.0),
            M.plinth,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "carcass",
            w - 0.01,
            h - t - 0.08,
            zc - z0,
            (0.0, 0.08 + (h - t - 0.08) / 2.0, (z0 + zc) / 2.0),
            M.blend_cab,
            bevel = 0.006,
            bevel_segments = 1
        );
        // the worktop, its front rolled (postformed)
        fsweep!(
            f,
            "worktop",
            &[
                (-w / 2.0, h - t / 2.0, z0 + d / 2.0),
                (w / 2.0, h - t / 2.0, z0 + d / 2.0)
            ],
            0,
            M.worktop,
            profile = vec![
                (-t / 2.0, d / 2.0),
                (t / 2.0, d / 2.0),
                (t / 2.0, -d / 2.0 + 0.016),
                (t / 2.0 - 0.006, -d / 2.0 + 0.004),
                (0.0, -d / 2.0),
                (-t / 2.0 + 0.006, -d / 2.0 + 0.004),
                (-t / 2.0, -d / 2.0 + 0.014)
            ],
            up = (0, 0, 1)
        );
        kit::shaker!(
            f,
            "drawer",
            w - 0.03,
            0.15,
            (0.0, h - t - 0.095, zc + 0.01),
            0.02,
            M.blend_door,
            rail = 0.04
        );
        kit::pull!(
            f,
            "drawer_pull",
            0.0,
            h - t - 0.095,
            zc + 0.02,
            0.16,
            M.chrome,
            r = 0.006,
            out = 0.026
        );
        let dw = (w - 0.03) / 2.0;
        for s in [-1.0, 1.0] {
            let cx = s * (dw / 2.0 + 0.005);
            let dh = h - t - 0.29;
            kit::shaker!(
                f,
                numbered("door"),
                dw,
                dh,
                (cx, 0.1 + dh / 2.0, zc + 0.01),
                0.02,
                M.blend_door,
                rail = 0.055
            );
            let hx = cx - s * (dw / 2.0 - 0.045);
            kit::pull!(
                f,
                numbered("handle"),
                hx,
                h - t - 0.29,
                zc + 0.02,
                0.12,
                M.chrome,
                r = 0.006,
                out = 0.026,
                upright = true
            );
        }
        // the blender: a motor base with its control strip and dials, the glass jug with its handle and spout
        let (jx, jz) = JAR;
        lofted(
            f,
            "motor",
            jx,
            TOP,
            jz,
            &[
                (0.0, 0.09, 0.08),
                (0.06, 0.088, 0.078),
                (0.1, 0.075, 0.068),
                (0.11, 0.07, 0.064),
            ],
            M.motor,
            16,
            4.0,
        );
        fbox!(
            f,
            "motor_strip",
            0.1,
            0.03,
            0.004,
            (jx, TOP + 0.05, jz + 0.08),
            M.lime,
            bevel = 0.002,
            bevel_segments = 1
        );
        for dx in [-0.04, 0.0, 0.04] {
            revolve!(
                numbered("dial"),
                &[
                    (0.0, 0.0),
                    (0.011, 0.0),
                    (0.011, 0.008),
                    (0.008, 0.012),
                    (0.0, 0.012)
                ],
                at = f.at(jx + dx, TOP + 0.025, jz + 0.08),
                material = M.chrome,
                segments = 8,
                rot = (90, 0, 0)
            );
        }
        fcyl!(
            f,
            "jug_seat",
            0.07,
            0.02,
            (jx, TOP + 0.11, jz),
            M.dark,
            verts = 16
        );
        let P3 {
            x: bx,
            y: by_,
            z: bz,
        } = f.at(jx, TOP + 0.12, jz);
        let secs: Vec<Vec<P3>> = [(0.0, 0.058), (0.03, 0.074), (0.22, 0.074), (0.25, 0.07)]
            .iter()
            .map(|&(hh, r)| ring!(r, r, z = bz + hh, verts = 16, square = 3.2, at = (bx, by_)))
            .collect();
        loft!("jug", &secs, material = M.jar);
        fcone!(
            f,
            "jug_spout",
            0.02,
            0.006,
            0.03,
            (jx - 0.075, TOP + 0.36, jz),
            M.jar,
            rot = (0, 55, 0),
            verts = 8
        );
        fsweep!(
            f,
            "jug_handle",
            &kit::fillet!(
                &[
                    (jx + 0.072, TOP + 0.33, jz),
                    (jx + 0.12, TOP + 0.31, jz),
                    (jx + 0.125, TOP + 0.2, jz),
                    (jx + 0.075, TOP + 0.16, jz)
                ],
                0.03,
                steps = 2
            ),
            0,
            M.dark,
            profile = rounded_rect(0.024, 0.014, 0.005, 1),
            up = (0, 1, 0)
        );
        // a fruit bowl with oranges, an apple, a lime and two bananas; a board with lemon halves and a knife
        let bowl = [
            (0.0, 0.05, 0.05),
            (0.008, 0.06, 0.06),
            (0.05, 0.13, 0.13),
            (0.09, 0.16, 0.16),
            (0.1, 0.162, 0.162),
        ];
        lofted(f, "bowl", 0.16, TOP, -0.1, &bowl, M.bowl, 18, 2.0);
        for (x, y, z, r, m) in [
            (0.1, 0.12, -0.08, 0.04, M.orange),
            (0.2, 0.12, -0.14, 0.04, M.apple),
            (0.16, 0.14, -0.04, 0.035, M.lime),
            (0.22, 0.12, -0.05, 0.038, M.orange),
        ] {
            fsphere!(
                f,
                numbered("fruit"),
                r,
                (x, TOP + y, z),
                m,
                segments = 10,
                rings = 6
            );
            fcyl!(
                f,
                numbered("fruit_stalk"),
                0.003,
                0.012,
                (x, TOP + y + r, z),
                M.stem,
                verts = 4
            );
        }
        banana(
            f,
            "banana_a",
            (0.06, TOP + 0.15, -0.14),
            (0.24, TOP + 0.16, -0.1),
            0.03,
            0.02,
            M.banana,
        );
        banana(
            f,
            "banana_b",
            (0.07, TOP + 0.17, -0.12),
            (0.25, TOP + 0.18, -0.07),
            0.035,
            0.018,
            M.banana,
        );
        fextrude!(
            f,
            "board",
            &rounded_rect(0.26, 0.16, 0.02, 2),
            0.015,
            (0.12, TOP + 0.008, 0.12),
            M.chopping,
            plane = Facing::Top,
            holes = vec![rounded_rect(0.03, 0.012, 0.006, 2)],
            bevel = 0.003,
            bevel_segments = 1
        );
        for x in [0.08, 0.16] {
            fcyl!(
                f,
                numbered("lemon_rind"),
                0.032,
                0.012,
                (x, TOP + 0.021, 0.12 + (x - 0.12) * 0.25),
                M.lemon,
                verts = 12
            );
            fcyl!(
                f,
                numbered("lemon_face"),
                0.027,
                0.002,
                (x, TOP + 0.0275, 0.12 + (x - 0.12) * 0.25),
                M.worktop,
                verts = 12
            );
        }
        fbox!(
            f,
            "knife",
            0.13,
            0.003,
            0.022,
            (0.15, TOP + 0.017, 0.17),
            M.chrome,
            rot = (0, 0, 8)
        );
        fbox!(
            f,
            "knife_grip",
            0.07,
            0.014,
            0.018,
            (0.26, TOP + 0.022, 0.155),
            M.dark,
            rot = (0, 0, 8),
            bevel = 0.005,
            bevel_segments = 1
        );
        glass(f, -0.04, 0.12, M.drink_pink, TOP, 0.12, 0.035, Some(M.lime));
        glass(
            f,
            -0.32,
            0.12,
            M.drink_orange,
            TOP,
            0.12,
            0.035,
            Some(M.apple),
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The jug's lid: `lid` lifts it off and, after `hold`, settles it back.
    let (jx, jz) = JAR;
    let lid = new_parts(|| {
        revolve!(
            "lid",
            &[
                (0.0, 0.0),
                (0.072, 0.0),
                (0.074, 0.012),
                (0.07, 0.022),
                (0.03, 0.026),
                (0.0, 0.026)
            ],
            at = f.at(jx, TOP + 0.368, jz),
            material = M.dark,
            segments = 16
        );
        revolve!(
            "lid_cap",
            &[
                (0.0, 0.0),
                (0.026, 0.0),
                (0.026, 0.024),
                (0.02, 0.032),
                (0.0, 0.032)
            ],
            at = f.at(jx, TOP + 0.392, jz),
            material = M.dark,
            segments = 10
        );
    });
    let lid = hinged(join(&lid, "lid", None), f.at(jx, TOP + 0.37, jz));
    // The green smoothie in the glass jug, with streaks of fruit through it and bits at the top, so its swirl shows:
    // `blades` turns it once the lid is back on (the blades themselves would be hidden in it).
    let swirl = new_parts(|| {
        let P3 {
            x: bx,
            y: by_,
            z: bz,
        } = f.at(jx, TOP + 0.13, jz);
        let secs: Vec<Vec<P3>> = [(0.0, 0.054), (0.02, 0.066), (0.16, 0.066)]
            .iter()
            .map(|&(hh, r)| ring!(r, r, z = bz + hh, verts = 14, square = 3.2, at = (bx, by_)))
            .collect();
        loft!("smoothie", &secs, material = M.smoothie);
        for (a, m) in [(20.0, M.orange), (150.0, M.apple), (260.0, M.lime)] {
            let r = 0.062;
            fbox!(
                f,
                numbered("streak"),
                0.016,
                0.12,
                0.006,
                (jx + r * cosd(a), TOP + 0.21, jz + r * sind(a)),
                m,
                rot = (0.0, -a + 90.0, 0.0)
            );
            fsphere!(
                f,
                numbered("bit"),
                0.014,
                (
                    jx + 0.035 * cosd(a + 60.0),
                    TOP + 0.29,
                    jz + 0.035 * sind(a + 60.0)
                ),
                m,
                segments = 6,
                rings = 4
            );
        }
    });
    let swirl = join(&swirl, "smoothie", Some(f.at(jx, TOP + 0.13, jz)));
    let objs = settle!(centre = false);
    slide!(lid, "lid", (0, 0, 0.09), seconds = 0.5);
    spin(
        swirl,
        "blades",
        Spin {
            turns: 1.0,
            seconds: 1.0,
            ..Spin::default()
        },
    );
    objs
}

// --- the contrast rain shower (1x1): a cabin of its own tile, a square rain head, hot and cold dials -----------------

const HEAD: (f64, f64, f64) = (0.0, 2.0, -0.02); // the rain head's underside
const DOOR_HALF: f64 = 0.28; // half the glass door's width, across the middle of the front

pub fn contrast_shower() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (top, s) = (2.15, 0.45);
    let (hx, hy, hz) = HEAD;
    let still = new_parts(|| {
        fextrude!(
            f,
            "tray",
            &rounded_rect(0.9, 0.9, 0.03, 2),
            0.058,
            (0, 0.029, 0),
            M.tray,
            plane = Facing::Top,
            bevel = 0.012,
            bevel_segments = 1
        );
        revolve!(
            "drain",
            &[
                (0.0, 0.0),
                (0.045, 0.0),
                (0.045, 0.004),
                (0.038, 0.006),
                (0.0, 0.006)
            ],
            at = f.at(0.1, 0.057, 0.1),
            material = M.fittings,
            segments = 14
        );
        for k in -2..3 {
            fbox!(
                f,
                numbered("drain_slot"),
                0.05,
                0.002,
                0.004,
                (0.1, 0.0635, 0.1 + 0.012 * k as f64),
                M.grout
            );
        }
        fextrude!(
            f,
            "mat",
            &rounded_rect(0.6, 0.6, 0.04, 2),
            0.012,
            (0, 0.064, 0.02),
            M.shower_mat,
            plane = Facing::Top,
            bevel = 0.003,
            bevel_segments = 1
        );
        for k in 0..5 {
            fbox!(
                f,
                numbered("mat_rib"),
                0.54,
                0.004,
                0.02,
                (0.0, 0.071, -0.22 + k as f64 * 0.11),
                M.tray
            );
        }
        kit::tiled_walls!(
            f,
            (-s, s),
            (-s, s),
            (0.058, top),
            0.03,
            0.3,
            0.005,
            M.grout,
            M.tile,
            thick = 0.008,
            bevel = 0.002
        );
        fbox!(
            f,
            "glass_side",
            0.01,
            2.0,
            0.83,
            (0.435, 1.09, -0.005),
            M.glass
        );
        // a narrow fixed pane either side of the door across the middle (the door its own part, below)
        for (x0, x1) in [(-0.414, -DOOR_HALF - 0.004), (DOOR_HALF + 0.004, 0.4225)] {
            fbox!(
                f,
                numbered("glass_fixed"),
                x1 - x0,
                2.0,
                0.01,
                ((x0 + x1) / 2.0, 1.09, 0.437),
                M.glass
            );
        }
        let prof = rounded_rect(0.025, 0.025, 0.006, 1);
        for (x, z) in [(0.435, 0.435), (0.435, -0.42)] {
            fsweep!(
                f,
                numbered("post"),
                &[(x, 0.058, z), (x, top, z)],
                0,
                M.shower_frame,
                profile = &prof,
                up = (1, 0, 0)
            );
        }
        fsweep!(
            f,
            "rail_top",
            &[
                (-0.42, top - 0.015, 0.435),
                (0.435, top - 0.015, 0.435),
                (0.435, top - 0.015, -0.42)
            ],
            0,
            M.shower_frame,
            profile = rounded_rect(0.03, 0.03, 0.006, 1),
            up = (0, 0, 1)
        );
        fbox!(
            f,
            "rail_bottom",
            0.87,
            0.02,
            0.03,
            (0.015, 0.068, 0.435),
            M.shower_frame,
            bevel = 0.004,
            bevel_segments = 1
        );
        for y in [0.3, 1.85] {
            // the door's hinges on the right pane
            fextrude!(
                f,
                numbered("hinge"),
                &rounded_rect(0.05, 0.05, 0.01, 2),
                0.026,
                (DOOR_HALF + 0.01, y, 0.437),
                M.shower_frame,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // the square rain head on its arm from the wall, the hot and cold dials on their plate, a shelf of bottles
        kit::tube!(
            f,
            "arm",
            &kit::fillet!(
                &[
                    (0.0, 2.1, -0.415),
                    (0.0, 2.1, -0.3),
                    (0.0, 2.04, -0.25),
                    (0.0, 2.04, hz),
                ],
                0.04
            ),
            0.014,
            M.fittings,
            verts = 8,
            round = 1
        );
        revolve!(
            "arm_rose",
            &[
                (0.0, 0.0),
                (0.035, 0.0),
                (0.035, 0.008),
                (0.02, 0.016),
                (0.0, 0.016)
            ],
            at = f.at(0, 2.1, -0.415),
            material = M.fittings,
            segments = 12,
            rot = (90, 0, 0)
        );
        fextrude!(
            f,
            "rain_head",
            &rounded_rect(0.4, 0.4, 0.05, 2),
            0.025,
            (hx, hy + 0.0125, hz),
            M.fittings,
            plane = Facing::Top,
            bevel = 0.006,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "rain_face",
            &rounded_rect(0.36, 0.36, 0.04, 2),
            0.004,
            (hx, hy - 0.001, hz),
            M.rain_face,
            plane = Facing::Top
        );
        for i in 0..5 {
            for k in 0..5 {
                fbox!(
                    f,
                    numbered("nozzle"),
                    0.01,
                    0.003,
                    0.01,
                    (
                        hx - 0.14 + i as f64 * 0.07,
                        hy - 0.003,
                        hz - 0.14 + k as f64 * 0.07
                    ),
                    M.fittings
                );
            }
        }
        fextrude!(
            f,
            "dial_plate",
            &rounded_rect(0.34, 0.12, 0.05, 3),
            0.012,
            (0, 1.15, -0.414),
            M.fittings,
            bevel = 0.004,
            bevel_segments = 1
        );
        for (x, m) in [(-0.1, M.hot), (0.1, M.cold)] {
            revolve!(
                numbered("dial"),
                &[
                    (0.0, 0.0),
                    (0.042, 0.0),
                    (0.042, 0.012),
                    (0.036, 0.026),
                    (0.03, 0.03),
                    (0.0, 0.03)
                ],
                at = f.at(x, 1.15, -0.408),
                material = m,
                segments = 14,
                rot = (90, 0, 0)
            );
            fbox!(
                f,
                numbered("dial_mark"),
                0.008,
                0.05,
                0.006,
                (x, 1.165, -0.376),
                M.fittings
            );
        }
        fbox!(
            f,
            "niche_shelf",
            0.3,
            0.015,
            0.1,
            (-0.2, 1.4, -0.365),
            M.tile,
            bevel = 0.003,
            bevel_segments = 1
        );
        for (x, m, hh) in [
            (-0.29, M.hot, 0.16),
            (-0.22, M.cold, 0.13),
            (-0.15, M.shower_mat, 0.15),
        ] {
            lofted(
                f,
                &numbered("bottle"),
                x,
                1.4075,
                -0.37,
                &[
                    (0.0, 0.024, 0.016),
                    (0.004, 0.026, 0.018),
                    (hh - 0.02, 0.026, 0.018),
                    (hh, 0.014, 0.012),
                ],
                m,
                10,
                3.0,
            );
            fcyl!(
                f,
                numbered("bottle_cap"),
                0.012,
                0.02,
                (x, 1.4075 + hh + 0.01, -0.37),
                M.towel,
                verts = 8
            );
        }
        // a towel hanging from a hook on the back wall
        fcyl!(
            f,
            "hook",
            0.008,
            0.03,
            (0.36, 1.42, -0.405),
            M.fittings,
            rot = (90, 0, 0),
            verts = 8
        );
        let towel = fsweep!(
            f,
            "towel",
            &kit::fillet!(
                &[
                    (0.36, 1.06, -0.4),
                    (0.36, 1.42, -0.402),
                    (0.36, 1.43, -0.38),
                    (0.36, 1.12, -0.36)
                ],
                0.012,
                steps = 2
            ),
            0,
            M.towel,
            profile = rounded_rect(0.012, 0.16, 0.005, 1),
            up = (0, -1, 0)
        );
        jitter!(towel, 0.004, size = 0.08, seed = 5.0, up = 0.3);
    });
    join(&still, "body", Some(P3::ZERO));
    // The rain: a grid of thin streams under the head, origin on the head, scaled to nothing: `water_on` grows them.
    let rain = new_parts(|| {
        for i in 0..5 {
            for k in 0..5 {
                fcyl!(
                    f,
                    numbered("rain"),
                    0.004,
                    hy - 0.07,
                    (
                        hx - 0.14 + i as f64 * 0.07,
                        (hy + 0.07) / 2.0,
                        hz - 0.14 + k as f64 * 0.07
                    ),
                    M.water,
                    verts = 6,
                    smooth = false
                );
            }
        }
    });
    let rain = join(&rain, "water", Some(f.at(hx, hy, hz)));
    rain.edit(|o| o.scale.z = 0.001);
    // The glass door across the middle of the front, hinged on its right: `door_open` swings it out to let the
    // bather in and out (a fourth draw: Q-05). All glass, no handle, to keep it one draw.
    let door = new_parts(|| {
        fbox!(
            f,
            "glass_door",
            2.0 * DOOR_HALF,
            1.98,
            0.01,
            (0, 1.08, 0.437),
            M.glass
        );
    });
    let door = hinged(join(&door, "door", None), f.at(DOOR_HALF, 0.09, 0.437));
    let objs = settle_all(); // on its Model centre: the back on the wall face
    grow(
        rain,
        "water_on",
        Grow {
            seconds: 0.5,
            ..Grow::default()
        },
    );
    swing!(
        door,
        "door_open",
        -95.0,
        seconds = 0.8,
        ease = Ease::BackOut
    );
    objs
}
