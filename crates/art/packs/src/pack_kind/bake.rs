//! The neighbourly baking counter and the community giving jar.

use super::common::{M, WALL, bush};
use crate::ease::{EASE, eased};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::math::{Axis, P3, PI, TAU, cos, cosd, pow, sin, sind, sqrt};
use artkit::parts::{aside, hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, jitter, kit, settle, slide, swing, torus};

// --- the neighbourly baking counter (2x1): a cream retro range beside a sage counter laid with bakes --------
const RANGE: (f64, f64, f64, f64) = (0.6, 0.62, 0.92, 0.6); // x, width, hob height, depth

pub fn neighbourly_oven() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (rx, rw, rh, rd) = RANGE;
    let zf = WALL + rd; // the range's front
    let (cx0, cx1) = (-0.91, rx - rw / 2.0 - 0.01);
    let (cw, cxm, cd) = (cx1 - cx0, (cx0 + cx1) / 2.0, 0.56);
    let still = new_parts(|| {
        // the counter: a recessed plinth, the carcass, three shaker doors with brass knobs, a butcher-block top
        fbox!(
            f,
            "plinth",
            cw - 0.02,
            0.08,
            cd - 0.05,
            (cxm, 0.04, WALL + (cd - 0.05) / 2.0),
            M.grate
        );
        fbox!(
            f,
            "carcass",
            cw,
            0.78,
            cd,
            (cxm, 0.47, WALL + cd / 2.0),
            M.cabinet,
            bevel = 0.008,
            bevel_segments = 1
        );
        for i in 0..3 {
            let x = cx0 + cw / 6.0 * (2 * i + 1) as f64;
            kit::shaker!(
                f,
                numbered("door"),
                cw / 3.0 - 0.03,
                0.66,
                (x, 0.45, WALL + cd + 0.008),
                0.016,
                M.cabinet,
                rail = 0.055,
                panel = M.cabinet_panel,
                raised = true
            );
            kit::turned!(
                f,
                numbered("knob"),
                x + if i % 2 != 0 { 0.13 } else { -0.13 },
                0.68,
                WALL + cd + 0.016,
                &[
                    (0.0, 0.0),
                    (0.01, 0.0),
                    (0.006, 0.01),
                    (0.015, 0.02),
                    (0.01, 0.028),
                    (0.0, 0.029),
                ],
                M.brass,
                segments = 10,
                rot = (-90.0, 0.0, 0.0)
            );
        }
        fbox!(
            f,
            "worktop",
            cw + 0.01,
            0.045,
            cd + 0.03,
            (cxm, 0.8825, WALL + (cd + 0.03) / 2.0),
            M.butcher,
            bevel = 0.008,
            bevel_segments = 2
        );
        for k in 1..9 {
            // the butcher block's strips
            fbox!(
                f,
                numbered("strip"),
                0.003,
                0.0015,
                cd + 0.022,
                (cx0 + k as f64 * cw / 9.0, 0.9052, WALL + (cd + 0.03) / 2.0),
                M.crust_dark
            );
        }
    });
    // the bakes on the worktop: drawn, not the counter's size
    let bakes = new_parts(|| {
        let top = 0.905; // the worktop's top
        // the bakes: a lattice pie, a wire rack of biscuits with the rolling pin in front, a pink cake on its stand; the
        // worktop's right end, by the range, is left free (a patch of flour on it) for the cook's mixing bowl or board and
        // what they set down behind it (`STATION_SPOTS`, `SET_DOWN_SPOTS`: the pin lay where the bowl goes)
        let (px, pz) = (cxm - 0.28, -0.06);
        kit::turned!(
            f,
            "pie_dish",
            px,
            top,
            pz,
            &[
                (0.0, 0.0),
                (0.12, 0.0),
                (0.16, 0.05),
                (0.165, 0.052),
                (0.15, 0.052),
                (0.11, 0.008),
                (0.0, 0.008),
            ],
            M.terracotta,
            segments = 20
        );
        kit::turned!(
            f,
            "pie_crust",
            px,
            top,
            pz,
            &[
                (0.0, 0.052),
                (0.1, 0.048),
                (0.15, 0.04),
                (0.152, 0.03),
                (0.0, 0.03),
            ],
            M.crust,
            segments = 20
        );
        torus!(
            "pie_rim",
            0.148,
            0.01,
            at = f.at(px, top + 0.052, pz),
            material = M.crust_dark,
            major_segments = 24,
            minor_segments = 3
        );
        for i in -2..3 {
            let lift = pow(0.145, 2.0) - pow(i as f64 * 0.05, 2.0);
            let l = 2.0 * sqrt(if lift > 0.0 { lift } else { 0.0 });
            for (k, along_x) in [true, false].into_iter().enumerate() {
                let (i_, k_) = (i as f64, k as f64);
                let pts: Vec<(f64, f64, f64)> = [0.0, 0.5, 1.0]
                    .iter()
                    .map(|&t| {
                        if along_x {
                            (
                                px + (t - 0.5) * l,
                                top + 0.058 + 0.004 * cos(PI * (2.0 * t + i_ + k_)),
                                pz + i_ * 0.05,
                            )
                        } else {
                            (
                                px + i_ * 0.05,
                                top + 0.058 + 0.004 * cos(PI * (2.0 * t + i_ + k_ + 1.0)),
                                pz + (t - 0.5) * l,
                            )
                        }
                    })
                    .collect();
                fsweep!(
                    f,
                    numbered("lattice"),
                    &pts,
                    0,
                    M.crust_dark,
                    profile = vec![
                        (-0.004, -0.01),
                        (0.004, -0.01),
                        (0.004, 0.01),
                        (-0.004, 0.01)
                    ],
                    up = (0, 0, 1)
                );
            }
        }
        let (rx0, rz0) = (cxm + 0.1, -0.1);
        for k in 0..5 {
            let z = rz0 - 0.1 + 0.05 * k as f64;
            kit::tube!(
                f,
                numbered("rack_bar"),
                &[(rx0 - 0.15, top + 0.016, z), (rx0 + 0.15, top + 0.016, z)],
                0.0025,
                M.chrome,
                verts = 4,
                round = 1
            );
        }
        for x in [rx0 - 0.15, rx0 + 0.15] {
            kit::tube!(
                f,
                numbered("rack_end"),
                &kit::fillet!(
                    &[
                        (x, top, rz0 - 0.11),
                        (x, top + 0.016, rz0 - 0.11),
                        (x, top + 0.016, rz0 + 0.11),
                        (x, top, rz0 + 0.11),
                    ],
                    0.01
                ),
                0.003,
                M.chrome,
                verts = 4,
                round = 1
            );
        }
        for (x, z) in [
            (0.07, -0.15),
            (0.15, -0.06),
            (0.23, -0.14),
            (0.1, -0.03),
            (0.22, -0.04),
            (0.05, -0.09),
        ] {
            kit::turned!(
                f,
                numbered("biscuit"),
                rx0 - 0.15 + x,
                top + 0.019,
                z,
                &[(0.0, 0.0), (0.03, 0.004), (0.026, 0.012), (0.0, 0.012)],
                M.crust_dark,
                segments = 10
            );
        }
        kit::turned!(
            f,
            "rolling_pin",
            rx0 - 0.15,
            top + 0.025,
            0.065,
            &[
                (0.0, 0.0),
                (0.008, 0.0),
                (0.008, 0.05),
                (0.014, 0.056),
                (0.025, 0.065),
                (0.025, 0.235),
                (0.014, 0.244),
                (0.008, 0.25),
                (0.008, 0.3),
                (0.0, 0.3),
            ],
            M.honey,
            segments = 14,
            rot = (0.0, -90.0, 0.0)
        );
        fextrude!(
            f,
            "flour_dust",
            &smooth_outline(
                &[
                    (-0.1, -0.06),
                    (0.04, -0.075),
                    (0.11, -0.02),
                    (0.08, 0.06),
                    (-0.03, 0.07),
                    (-0.11, 0.02)
                ],
                2,
                true
            ),
            0.002,
            (0.135, top + 0.001, -0.03),
            M.flour,
            plane = Facing::Top
        );
        kit::turned!(
            f,
            "cake_stand",
            cxm - 0.42,
            top,
            -0.3,
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.05, 0.01),
                (0.02, 0.025),
                (0.016, 0.06),
                (0.02, 0.07),
                (0.12, 0.074),
                (0.122, 0.082),
                (0.11, 0.085),
                (0.0, 0.085),
            ],
            M.porcelain,
            segments = 16
        );
        kit::turned!(
            f,
            "cake",
            cxm - 0.42,
            top + 0.085,
            -0.3,
            &[
                (0.0, 0.0),
                (0.09, 0.0),
                (0.09, 0.08),
                (0.084, 0.092),
                (0.07, 0.095),
                (0.0, 0.095),
            ],
            M.icing,
            segments = 20
        );
        for k in 0..8 {
            // icing run down the cake's side
            let a = (k * 45 + 10) as f64;
            fsweep!(
                f,
                numbered("drip"),
                &[
                    (
                        cxm - 0.42 + 0.088 * cosd(a),
                        top + 0.177,
                        -0.3 + 0.088 * sind(a)
                    ),
                    (
                        cxm - 0.42 + 0.092 * cosd(a),
                        top + 0.15 - 0.012 * (k % 3) as f64,
                        -0.3 + 0.092 * sind(a)
                    )
                ],
                0.007,
                M.flour,
                verts = 5,
                scale = [1.0, 1.25]
            );
        }
        fsphere!(
            f,
            "cherry",
            0.014,
            (cxm - 0.42, top + 0.19, -0.3),
            M.heart,
            segments = 8,
            rings = 5
        );
    });
    // a beadboard back rising from the worktop's back (a dresser's), carrying the shelf with three jars: it stands
    // free or on the wall (2026-10-21: the shelf hung on the air over a free-standing counter); a wall shelf, drawn
    // with the counter, not its size
    let shelf = new_parts(|| {
        let top = 0.905; // the worktop's top
        fbox!(
            f,
            "backboard",
            cw,
            1.62 - top,
            0.018,
            (cxm, (top + 1.62) / 2.0, WALL + 0.009),
            M.cabinet,
            bevel = 0.004,
            bevel_segments = 1
        );
        for i in 1..8 {
            fbox!(
                f,
                numbered("bead"),
                0.006,
                1.62 - top - 0.04,
                0.004,
                (cx0 + i as f64 * cw / 8.0, (top + 1.62) / 2.0, WALL + 0.02),
                M.cabinet_panel
            );
        }
        fsweep!(
            f,
            "backboard_cap",
            &[
                (cx0 - 0.01, 1.6325, WALL + 0.02),
                (cx1 + 0.01, 1.6325, WALL + 0.02)
            ],
            0,
            M.butcher,
            profile = rounded_rect(0.025, 0.04, 0.008, 1),
            up = (0, 0, 1)
        );
        fbox!(
            f,
            "shelf",
            cw,
            0.03,
            0.2,
            (cxm, 1.5, WALL + 0.1),
            M.butcher,
            bevel = 0.006,
            bevel_segments = 1
        );
        for x in [cx0 + 0.06, cx1 - 0.06] {
            fextrude!(
                f,
                numbered("bracket"),
                &[
                    (0.0, 0.0),
                    (0.0, 0.12),
                    (0.16, 0.12),
                    (0.15, 0.1),
                    (0.08, 0.08),
                    (0.03, 0.03)
                ],
                0.025,
                (x, 1.365, WALL),
                M.butcher,
                plane = Facing::Side,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for (dx, h, m) in [
            (-0.4, 0.16, M.jar_white),
            (-0.15, 0.2, M.jar_red),
            (0.1, 0.14, M.jar_gold),
        ] {
            kit::lathe!(
                f,
                numbered("jar"),
                cxm + dx,
                1.515,
                WALL + 0.1,
                &[
                    (0.045, 0.0),
                    (0.05, 0.006),
                    (0.055, 0.02),
                    (0.055, h - 0.035),
                    (0.046, h - 0.022),
                    (0.042, h - 0.02),
                    (0.0, h - 0.02),
                ],
                m,
                verts = 12
            );
            kit::turned!(
                f,
                numbered("jar_lid"),
                cxm + dx,
                1.515 + h - 0.0225,
                WALL + 0.1,
                &[
                    (0.0, 0.0),
                    (0.05, 0.0),
                    (0.051, 0.018),
                    (0.046, 0.025),
                    (0.0, 0.025),
                ],
                M.cork,
                segments = 12
            );
            // (plain, not the towel's check: the door with the towel takes the fabric draw)
            fbox!(
                f,
                numbered("jar_label"),
                0.05,
                0.04,
                0.004,
                (cxm + dx, 1.515 + h * 0.45, WALL + 0.155),
                M.porcelain
            );
        }
    });
    aside(&shelf, "shelf");
    let range = new_parts(|| {
        // the range: an enamel body, a hob with four burners under cast-iron grates, a backguard with a clock, a knob
        // panel, a chrome rail, a stock pot (the gingham tea towel is the door's, over its handle)
        fbox!(
            f,
            "range",
            rw,
            rh - 0.04,
            rd - 0.03,
            (rx, (rh - 0.04) / 2.0, WALL + (rd - 0.03) / 2.0),
            M.enamel,
            bevel = 0.02,
            bevel_segments = 2
        );
        fbox!(
            f,
            "kick",
            rw - 0.06,
            0.06,
            0.02,
            (rx, 0.03, zf - 0.035),
            M.hob
        );
        fbox!(
            f,
            "hob",
            rw + 0.004,
            0.035,
            rd - 0.01,
            (rx, rh - 0.0175, WALL + (rd - 0.01) / 2.0),
            M.hob,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "backguard",
            rw,
            0.16,
            0.04,
            (rx, rh + 0.08, WALL + 0.02),
            M.enamel,
            bevel = 0.012,
            bevel_segments = 2
        );
        fcyl!(
            f,
            "clock",
            0.045,
            0.008,
            (rx, rh + 0.09, WALL + 0.043),
            M.porcelain,
            rot = (90, 0, 0),
            verts = 16
        );
        torus!(
            "clock_rim",
            0.046,
            0.004,
            at = f.at(rx, rh + 0.09, WALL + 0.045),
            material = M.chrome,
            rot = (90, 0, 0),
            major_segments = 16,
            minor_segments = 3
        );
        for (a, l) in [(60.0, 0.025), (-60.0, 0.035)] {
            fbox!(
                f,
                numbered("clock_hand"),
                0.004,
                l,
                0.002,
                (
                    rx + l / 2.0 * sind(a),
                    rh + 0.09 + l / 2.0 * cosd(a),
                    WALL + 0.048
                ),
                M.grate,
                rot = (0.0, -a, 0.0)
            );
        }
        for dx in [-0.15, 0.15] {
            for dz in [-0.12, 0.12] {
                let (bx, bz) = (rx + dx, WALL + rd / 2.0 + dz);
                kit::turned!(
                    f,
                    numbered("burner"),
                    bx,
                    rh,
                    bz,
                    &[(0.0, 0.0), (0.065, 0.0), (0.05, 0.01), (0.0, 0.016)],
                    M.grate,
                    segments = 12
                );
                fcyl!(
                    f,
                    numbered("burner_cap"),
                    0.03,
                    0.006,
                    (bx, rh + 0.019, bz),
                    M.chrome,
                    verts = 12
                );
                for a in [0.0, 90.0] {
                    let (c, s) = (cosd(a), sind(a));
                    kit::tube!(
                        f,
                        numbered("grate"),
                        &kit::fillet!(
                            &[
                                (bx - 0.1 * c, rh + 0.004, bz - 0.1 * s),
                                (bx - 0.1 * c, rh + 0.026, bz - 0.1 * s),
                                (bx + 0.1 * c, rh + 0.026, bz + 0.1 * s),
                                (bx + 0.1 * c, rh + 0.004, bz + 0.1 * s),
                            ],
                            0.01
                        ),
                        0.0055,
                        M.grate,
                        verts = 4,
                        round = 1
                    );
                }
            }
        }
        fbox!(
            f,
            "knob_panel",
            rw - 0.03,
            0.1,
            0.015,
            (rx, rh - 0.1, zf - 0.03 + 0.0075),
            M.enamel,
            bevel = 0.004,
            bevel_segments = 1
        );
        for i in 0..5 {
            kit::turned!(
                f,
                numbered("range_knob"),
                rx - 0.22 + i as f64 * 0.11,
                rh - 0.1,
                zf - 0.015,
                &[
                    (0.0, 0.0),
                    (0.02, 0.0),
                    (0.017, 0.012),
                    (0.014, 0.026),
                    (0.0, 0.027),
                ],
                M.chrome,
                segments = 10,
                rot = (-90.0, 0.0, 0.0)
            );
        }
        fbox!(
            f,
            "oven_mouth",
            rw - 0.1,
            0.42,
            0.006,
            (rx, 0.37, zf - 0.03 + 0.003),
            M.oven_dark
        );
        fbox!(
            f,
            "oven_rack",
            rw - 0.14,
            0.01,
            0.004,
            (rx, 0.3, zf - 0.03 + 0.007),
            M.chrome
        );
        kit::tube!(
            f,
            "rail",
            &kit::fillet!(
                &[
                    (rx - rw / 2.0 + 0.06, 0.7, zf - 0.016),
                    (rx - rw / 2.0 + 0.06, 0.7, zf + 0.012),
                    (rx + rw / 2.0 - 0.06, 0.7, zf + 0.012),
                    (rx + rw / 2.0 - 0.06, 0.7, zf - 0.016),
                ],
                0.012
            ),
            0.009,
            M.chrome,
            verts = 8,
            round = 1
        );
    });
    // the stock pot on the hob: drawn, not the counter's size
    let pot = new_parts(|| {
        let (sx, sz) = (rx + 0.15, WALL + rd / 2.0 - 0.12);
        kit::turned!(
            f,
            "stock_pot",
            sx,
            rh + 0.026,
            sz,
            &[
                (0.0, 0.0),
                (0.105, 0.0),
                (0.11, 0.008),
                (0.11, 0.13),
                (0.114, 0.138),
                (0.104, 0.14),
                (0.1, 0.135),
                (0.0, 0.135),
            ],
            M.sage_pot,
            segments = 18
        );
        kit::turned!(
            f,
            "stock_lid",
            sx,
            rh + 0.163,
            sz,
            &[
                (0.0, 0.0),
                (0.112, 0.0),
                (0.1, 0.012),
                (0.03, 0.02),
                (0.0, 0.02),
            ],
            M.sage_dark,
            segments = 18
        );
        fsphere!(
            f,
            "stock_knob",
            0.014,
            (sx, rh + 0.186, sz),
            M.sage_dark,
            segments = 8,
            rings = 5
        );
        for s in [-1.0, 1.0] {
            kit::tube!(
                f,
                numbered("stock_handle"),
                &kit::fillet!(
                    &[
                        (sx + s * 0.108, rh + 0.13, sz),
                        (sx + s * 0.135, rh + 0.13, sz),
                        (sx + s * 0.135, rh + 0.11, sz),
                        (sx + s * 0.108, rh + 0.11, sz),
                    ],
                    0.006
                ),
                0.005,
                M.sage_dark,
                verts = 4,
                round = 1
            );
        }
    });
    join(&[still, range].concat(), "body", Some(P3::ZERO));
    aside(&bakes, "bakes");
    aside(&pot, "pot");
    // The oven door, hinged on its foot: `oven_open` drops it forward while a pie or treats bake. A window in a raised
    // frame, a chrome bar handle on stand-offs.
    let door = new_parts(|| {
        fbox!(
            f,
            "oven_door",
            rw - 0.04,
            0.5,
            0.02,
            (rx, 0.38, zf - 0.02),
            M.enamel,
            bevel = 0.01,
            bevel_segments = 2
        );
        kit::moulding!(
            f,
            "oven_frame",
            &[
                (rx + 0.2, 0.25),
                (rx + 0.2, 0.47),
                (rx - 0.2, 0.47),
                (rx - 0.2, 0.25),
            ],
            &[
                (-0.012, 0.0),
                (-0.012, 0.004),
                (0.0, 0.008),
                (0.012, 0.008),
                (0.012, 0.0),
            ],
            M.enamel,
            at = zf - 0.01
        );
        fbox!(
            f,
            "oven_window",
            rw - 0.22,
            0.2,
            0.006,
            (rx, 0.36, zf - 0.009),
            M.oven_glass
        );
        kit::pull!(
            f,
            "door_handle",
            rx,
            0.58,
            zf - 0.01,
            rw - 0.16,
            M.chrome,
            r = 0.009,
            out = 0.032,
            verts = 8
        );
        // the gingham tea towel folded over the handle, so it drops with the door (hung on the rail above, the door
        // swung up through it as it opened)
        let (tx, bz) = (rx + 0.1, zf + 0.022); // its middle; the handle bar's depth
        let pts = [
            (tx, 0.40, bz + 0.012),
            (tx, 0.56, bz + 0.013),
            (tx, 0.588, bz + 0.011),
            (tx, 0.595, bz),
            (tx, 0.588, bz - 0.011),
            (tx, 0.56, bz - 0.013),
            (tx, 0.50, bz - 0.014),
        ];
        let towel = fsweep!(
            f,
            "towel",
            &pts,
            0,
            M.towel,
            profile = rounded_rect(0.16, 0.006, 0.003, 1),
            up = (1, 0, 0),
            resolution = 2
        );
        jitter!(towel, 0.002, size = 0.06, seed = 1.0, up = 0.3);
        for k in 0..4 {
            fbox!(
                f,
                numbered("towel_check"),
                0.16,
                0.02,
                0.003,
                (tx, 0.43 + k as f64 * 0.04, bz + 0.0165),
                M.towel_check
            );
        }
        for k in 0..3 {
            fbox!(
                f,
                numbered("towel_check_v"),
                0.026,
                0.15,
                0.0028,
                (tx - 0.053 + 0.053 * k as f64, 0.48, bz + 0.0163),
                M.towel_check
            );
        }
    });
    let door = hinged(join(&door, "oven_door", None), f.at(rx, 0.13, zf - 0.02));
    let objs = settle!(centre = false);
    swing!(door, "oven_open", 85.0, seconds = 0.8, axis = Axis::X);
    objs
}

// --- the community giving jar (1x1): a glass jar of coins on a turned pedestal ----------------------------
const JAR: (f64, f64) = (0.0, -0.05); // x, depth of the jar's axis
const LID_TOP: f64 = 1.28;

/// A heart outline `s` across, its point down.
fn heart(s: f64) -> Vec<(f64, f64)> {
    let mut pts = Vec::new();
    for k in 0..16 {
        let t = TAU * k as f64 / 16.0;
        pts.push((
            s * 0.5 * pow(sin(t), 3.0),
            s * 0.4 * (13.0 * cos(t) - 5.0 * cos(2.0 * t) - 2.0 * cos(3.0 * t) - cos(4.0 * t))
                / 16.0,
        ));
    }
    pts
}

pub fn giving_jar() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (jx, jz) = JAR;
    let still = new_parts(|| {
        kit::turned!(
            f,
            "pedestal",
            jx,
            0.0,
            jz,
            &eased(
                &[
                    (0.0, 0.0),
                    (0.22, 0.0),
                    (0.22, 0.03),
                    (0.205, 0.045),
                    (0.15, 0.06),
                    (0.13, 0.09),
                    (0.09, 0.12),
                    (0.07, 0.17),
                    (0.075, 0.2),
                    (0.058, 0.24),
                    (0.053, 0.5),
                    (0.06, 0.66),
                    (0.075, 0.72),
                    (0.1, 0.76),
                    (0.17, 0.8),
                    (0.185, 0.815),
                    (0.185, 0.835),
                    (0.18, 0.86),
                    (0.0, 0.86),
                ],
                EASE
            ),
            M.pedestal,
            segments = 28
        );
        torus!(
            "pedestal_bead",
            0.062,
            0.009,
            at = f.at(jx, 0.28, jz),
            material = M.pedestal,
            major_segments = 20,
            minor_segments = 4
        );
        // coins heaped in the jar, notes among them
        for i in 0..22 {
            let a = i as f64 * 2.4;
            let rr = 0.02 + 0.1 * ((i * 7) % 6) as f64 / 6.0;
            fcyl!(
                f,
                numbered("coin"),
                0.022,
                0.006,
                (
                    jx + rr * cos(a),
                    0.872 + 0.01 * (i / 5) as f64 + 0.004 * (rr < 0.06) as i32 as f64,
                    jz + rr * sin(a)
                ),
                M.gold,
                rot = ((i * 23) % 30, (i * 17) % 25, 0),
                verts = 12
            );
        }
        for (k, (dx, dz)) in [(-0.02, 0.02), (0.05, -0.04)].into_iter().enumerate() {
            fsweep!(
                f,
                numbered("note"),
                &[
                    (jx + dx - 0.05, 0.91, jz + dz),
                    (jx + dx, 0.915 + 0.01 * k as f64, jz + dz + 0.01),
                    (jx + dx + 0.05, 0.93, jz + dz)
                ],
                0,
                M.note,
                profile = rounded_rect(0.002, 0.06, 0.001, 1),
                up = (0, 0, 1),
                resolution = 2
            );
        }
        // the cork lid and its slot, a ribbon round the neck tied in a bow, a heart tag on a string
        kit::turned!(
            f,
            "cork",
            jx,
            LID_TOP - 0.05,
            jz,
            &eased(
                &[
                    (0.0, 0.0),
                    (0.1, 0.0),
                    (0.106, 0.04),
                    (0.103, 0.05),
                    (0.0, 0.05),
                ],
                EASE
            ),
            M.cork,
            segments = 22
        );
        fbox!(
            f,
            "slot",
            0.07,
            0.004,
            0.012,
            (jx, LID_TOP + 0.001, jz),
            M.dark
        );
        torus!(
            "ribbon",
            0.112,
            0.01,
            at = f.at(jx, 1.2, jz),
            material = M.ribbon,
            major_segments = 24,
            minor_segments = 4
        );
        let bz = jz + 0.115;
        for s in [-1.0, 1.0] {
            kit::tube!(
                f,
                numbered("bow_loop"),
                &[
                    (jx, 1.2, bz),
                    (jx + s * 0.03, 1.225, bz + 0.012),
                    (jx + s * 0.055, 1.21, bz + 0.008),
                    (jx + s * 0.04, 1.192, bz + 0.006),
                    (jx, 1.2, bz),
                ],
                0.007,
                M.ribbon,
                verts = 5,
                round = 2
            );
            fsweep!(
                f,
                numbered("bow_tail"),
                &[
                    (jx, 1.195, bz + 0.004),
                    (jx + s * 0.02, 1.16, bz + 0.012),
                    (jx + s * 0.03, 1.13, bz + 0.016)
                ],
                0,
                M.ribbon,
                profile = rounded_rect(0.003, 0.016, 0.001, 1),
                up = (0, 1, 0),
                resolution = 2
            );
        }
        fsphere!(
            f,
            "bow_knot",
            0.012,
            (jx, 1.2, bz + 0.004),
            M.ribbon,
            segments = 8,
            rings = 5
        );
        kit::tube!(
            f,
            "tag_string",
            &[
                (jx, 1.19, bz + 0.004),
                (jx + 0.004, 1.14, bz + 0.03),
                (jx, 1.11, bz + 0.04),
            ],
            0.0025,
            M.ribbon,
            verts = 4,
            round = 2
        );
        fextrude!(
            f,
            "heart_tag",
            &heart(0.075),
            0.01,
            (jx, 1.075, jz + 0.155),
            M.heart,
            bevel = 0.003,
            bevel_segments = 1
        );
        // a chalk sign framed on the pedestal, a chalked heart and lines
        fextrude!(
            f,
            "sign_frame",
            &rounded_rect(0.24, 0.18, 0.012, 2),
            0.014,
            (jx, 0.62, jz + 0.074),
            M.pedestal,
            holes = vec![rounded_rect(0.205, 0.145, 0.006, 1)],
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(f, "sign", 0.21, 0.15, 0.006, (jx, 0.62, jz + 0.07), M.chalk);
        for (y, w) in [(0.6, 0.14), (0.575, 0.1)] {
            fbox!(
                f,
                numbered("chalk_line"),
                w,
                0.008,
                0.003,
                (jx, y, jz + 0.0745),
                M.chalk_line
            );
        }
        let hl: Vec<(f64, f64, f64)> = heart(0.04)
            .iter()
            .map(|&(u, v)| (jx + u, v + 0.66, jz + 0.075))
            .collect();
        fsweep!(
            f,
            "chalk_heart",
            &hl,
            0.0022,
            M.chalk_line,
            verts = 4,
            closed = true
        );
        // the glass jar over the coins (the glass draw): a thick rim and a lip under the cork
        kit::turned!(
            f,
            "jar",
            jx,
            0.86,
            jz,
            &[
                (0.0, 0.0),
                (0.13, 0.0),
                (0.145, 0.012),
                (0.15, 0.076),
                (0.15, 0.285),
                (0.12, 0.325),
                (0.108, 0.334),
                (0.112, 0.34),
                (0.112, 0.352),
                (0.104, 0.36),
                (0.0, 0.36),
            ],
            M.glass,
            segments = 28
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // the flowering pot plant at its foot: drawn, not the jar's size
    let plant = new_parts(|| {
        kit::turned!(
            f,
            "plant_pot",
            0.31,
            0.0,
            0.291,
            &eased(
                &[
                    (0.0, 0.0),
                    (0.058, 0.0),
                    (0.075, 0.13),
                    (0.085, 0.135),
                    (0.085, 0.16),
                    (0.075, 0.16),
                    (0.07, 0.148),
                    (0.0, 0.148),
                ],
                EASE
            ),
            M.terracotta,
            segments = 18
        );
        bush(f, 0.31, 0.15, 0.291, 0.1, M.mustard, 3);
    });
    aside(&plant, "plant");
    // A coin standing in the slot: `coin` drops it through into the jar.
    let coin = new_parts(|| {
        fcyl!(
            f,
            "donation",
            0.024,
            0.005,
            (jx, LID_TOP + 0.016, jz),
            M.gold,
            rot = (90, 0, 0),
            verts = 16
        );
        fcyl!(
            f,
            "donation_face",
            0.018,
            0.0055,
            (jx, LID_TOP + 0.016, jz),
            M.mustard,
            rot = (90, 0, 0),
            verts = 12
        );
    });
    let coin = join(&coin, "coin", None);
    let objs = settle!(centre = false);
    slide!(coin, "coin", (0, 0, -0.1), seconds = 0.6);
    objs
}
