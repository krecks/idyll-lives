//! The heated quilt table and the smart-home command hub.

use super::common::{M, WALL};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, TAU, cos, cosd, sind};
use artkit::parts::{aside, hinged, join, new_parts, numbered, turn};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{bevel::ease, forms::eased, shapes::AUTO_WIDTH};
use artkit::{fbox, fcyl, fsphere, fsweep, kit, settle, soft, swing, torus};

// --- the heated quilt table (2x1): a kotatsu in front, two poufs (tops 0.45) at x = ±0.5 behind it ---------
const TOP_Y: f64 = 0.6; // the underside of the table top
const TZ0: f64 = -0.03; // the top's back edge (depth)
const TZ1: f64 = 0.47; // and its front: 0.5 deep, so the sitters' legs and feet go under it
const FLAP_Z: f64 = TZ0 - 0.012; // the quilt's back edge, hung over the sitters' laps
const SEAT_Z: f64 = -0.2; // the poufs' middles (depth): the hips there (`furniture.json` `lazy.quiltTable`)

/// A curtain of quilt from height y1 down to y0 along the line from game (x, depth) `a` to `b`, hanging `out` past
/// it (outwards, to its left going from a to b), in `folds` soft folds.
#[allow(clippy::too_many_arguments)]
fn _drape(
    f: &GameFrame,
    name: &str,
    a: (f64, f64),
    b: (f64, f64),
    y0: f64,
    y1: f64,
    out: f64,
    material: Mat,
    folds: i32,
    amp: f64,
    t: f64,
) -> Obj {
    let ((ax, az), (bx, bz)) = (a, b);
    let l = artkit::py::hypot(bx - ax, bz - az);
    let (ux, uz) = ((bx - ax) / l, (bz - az) / l);
    let (nx, nz) = (-uz, ux); // outwards
    let n = folds * 4;
    let mut pts = Vec::new();
    for k in 0..=n {
        let fr = k as f64 / n as f64;
        let w = out
            + amp
                * (0.5 - 0.5 * cos(TAU * folds as f64 * fr))
                * 1.0_f64.min(4.0 * fr).min(4.0 * (1.0 - fr));
        pts.push((
            ax + ux * l * fr + nx * w,
            (y0 + y1) / 2.0,
            az + uz * l * fr + nz * w,
        ));
    }
    let prof = rounded_rect(y1 - y0, t, t * 0.45, 1);
    ease(
        fsweep!(f, name, &pts, 0.0, material, profile = prof, up = (0, 0, 1)),
        60.0,
        1,
    )
}

pub fn quilt_table() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (tz, td) = ((TZ0 + TZ1) / 2.0, TZ1 - TZ0);
    let still = new_parts(|| {
        fbox!(
            f,
            "top",
            1.74,
            0.035,
            td,
            (0, TOP_Y + 0.0175, tz),
            M.oak,
            bevel = 0.012,
            bevel_segments = 2
        );
        for x in [-0.82, 0.82] {
            // the legs, inside the quilt
            for z in [TZ0 + 0.05, TZ1 - 0.05] {
                fbox!(
                    f,
                    numbered("leg"),
                    0.05,
                    TOP_Y,
                    0.05,
                    (x, TOP_Y / 2.0, z),
                    M.oak
                );
            }
        }
        fbox!(
            f,
            "heater",
            0.36,
            0.05,
            0.3,
            (0, TOP_Y - 0.025, tz),
            M.heater,
            bevel = 0.01,
            bevel_segments = 1
        ); // under the top
        // the quilt, puffed over the top and hanging in folds down its front and ends to the floor: hollow, the legs go
        // in from the back under its flap (2026-10-21: a solid block 0.4 deep, the feet out of its front: I-packs-01)
        soft!(
            f,
            "quilt",
            1.8,
            0.03,
            td + 0.03,
            (0, TOP_Y + 0.05, tz),
            M.quilt,
            round = 0.015,
            puff = 0.008,
            flat = 2
        );
        for i in 1..6 {
            // its quilting, stitched in rows
            fbox!(
                f,
                numbered("quilt_seam"),
                0.004,
                0.004,
                td + 0.02,
                (-0.9 + 0.3 * i as f64, TOP_Y + 0.069, tz),
                M.quilt_band
            );
        }
        _drape(
            f,
            "quilt_front",
            (-0.879, TZ1 - 0.006),
            (0.879, TZ1 - 0.006),
            0.01,
            TOP_Y + 0.065,
            0.008,
            M.quilt,
            7,
            0.012,
            0.022,
        );
        for s in [-1, 1] {
            let (a, b) = if s < 0 {
                ((-0.879, TZ0 - 0.005), (-0.879, TZ1 - 0.006))
            } else {
                ((0.879, TZ1 - 0.006), (0.879, TZ0 - 0.005))
            };
            _drape(
                f,
                &numbered("quilt_end"),
                a,
                b,
                0.01,
                TOP_Y + 0.065,
                0.008,
                M.quilt,
                3,
                0.012,
                0.022,
            );
        }
        _drape(
            f,
            "quilt_band",
            (-0.879, TZ1 - 0.006),
            (0.879, TZ1 - 0.006),
            0.015,
            0.045,
            0.01,
            M.quilt_band,
            7,
            0.012,
            0.026,
        );
        // a bowl of mandarins, two mugs, a plate of biscuits on the quilt's top
        let ty = TOP_Y + 0.067;
        kit::turned!(
            f,
            "bowl",
            0,
            ty,
            0.25,
            &eased(
                &[
                    (0.0, 0.0),
                    (0.05, 0.0),
                    (0.06, 0.008),
                    (0.12, 0.055),
                    (0.13, 0.065),
                    (0.122, 0.067),
                    (0.055, 0.016),
                    (0.0, 0.016),
                ],
                AUTO_WIDTH
            ),
            M.porcelain,
            segments = 22
        );
        for (dx, dz, dy) in [
            (-0.05, 0.22, 0.0),
            (0.05, 0.27, 0.0),
            (0.0, 0.31, 0.0),
            (-0.02, 0.2, 0.04),
            (0.04, 0.2, 0.0),
        ] {
            fsphere!(
                f,
                numbered("mandarin"),
                0.035,
                (dx, ty + 0.035 + dy, dz),
                M.mandarin,
                scale = (1, 1, 0.85),
                segments = 10,
                rings = 6
            );
        }
        fsphere!(
            f,
            "mandarin_leaf",
            0.016,
            (-0.02, ty + 0.1, 0.21),
            M.leaf,
            scale = (1.6, 0.6, 0.2),
            rot = (0, 15, 30),
            segments = 6,
            rings = 4
        );
        for s in [-1, 1] {
            let x = s as f64 * 0.5;
            kit::mug!(
                f,
                numbered("mug"),
                x + s as f64 * 0.12,
                ty,
                0.12,
                0.042,
                0.09,
                M.porcelain,
                (90 - 70 * s) as f64,
                segments = 12,
                verts = 5
            );
            // the pouf, its seat cushion (top 0.45) and a back cushion against the wall
            soft!(
                f,
                numbered("pouf"),
                0.5,
                0.38,
                0.44,
                (x, 0.2, SEAT_Z),
                M.pouf,
                round = 0.12,
                puff = (0.03, 0.0, 0.008),
                flat = 2
            );
            soft!(
                f,
                numbered("pouf_top"),
                0.46,
                0.05,
                0.42,
                (x, 0.42, SEAT_Z),
                M.pouf_top,
                round = 0.022,
                puff = 0.008,
                piping = 0.005,
                flat = 2,
                resting = true
            );
            soft!(
                f,
                numbered("back_cushion"),
                0.44,
                0.3,
                0.064,
                (x, 0.6, WALL + 0.05),
                if s < 0 { M.sage } else { M.lilac },
                rot = (5, 0, 0),
                round = 0.03,
                puff = 0.012,
                flat = 2
            );
        }
        kit::turned!(
            f,
            "snack_plate",
            0.6,
            ty,
            0.32,
            &eased(
                &[
                    (0.0, 0.0),
                    (0.06, 0.0),
                    (0.1, 0.01),
                    (0.105, 0.013),
                    (0.06, 0.006),
                    (0.0, 0.006),
                ],
                AUTO_WIDTH
            ),
            M.porcelain,
            segments = 18
        );
        for (dx, dz) in [(-0.02, -0.01), (0.03, 0.02), (0.0, 0.035)] {
            kit::turned!(
                f,
                numbered("biscuit"),
                0.6 + dx,
                ty + 0.006,
                0.32 + dz,
                &[(0.0, 0.0), (0.026, 0.002), (0.022, 0.012), (0.0, 0.012)],
                M.biscuit,
                segments = 10
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The quilt's back edge, hung from the table top: `quilt` lifts it over the sitters' laps.
    let flap = new_parts(|| {
        soft!(
            f,
            "quilt_flap",
            1.62,
            0.29,
            0.025,
            (0, TOP_Y - 0.1, FLAP_Z),
            M.quilt,
            round = 0.01,
            puff = 0.006,
            flat = 2
        );
        fsweep!(
            f,
            "flap_band",
            &[
                (-0.81, TOP_Y - 0.235, FLAP_Z),
                (0.81, TOP_Y - 0.235, FLAP_Z)
            ],
            0.0,
            M.quilt_band,
            profile = rounded_rect(0.03, 0.03, 0.008, 1),
            up = (0, 0, 1)
        );
        fsweep!(
            f,
            "flap_hem",
            &[(-0.81, TOP_Y + 0.03, FLAP_Z), (0.81, TOP_Y + 0.03, FLAP_Z)],
            0.0,
            M.quilt_band,
            profile = rounded_rect(0.03, 0.028, 0.008, 1),
            up = (0, 0, 1)
        );
    });
    let flap = hinged(
        join(&flap, "quilt_flap", None),
        f.at(0, TOP_Y + 0.05, FLAP_Z),
    );
    let objs = settle!(centre = false); // built in place: the poufs under the sitters (`furniture.json`), the back on the wall
    swing!(flap, "quilt", 70.0, seconds = 0.8, axis = Axis::X);
    objs
}

// --- the smart-home command hub (1x1): a side table with a smart speaker, a tablet on its dock, a robot ------
const TABLE: (f64, f64, f64) = (0.6, 0.6, 0.44); // width, top height, depth

pub fn smart_hub() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (w, h, d) = TABLE;
    let z = WALL + d / 2.0;
    let still = new_parts(|| {
        fbox!(
            f,
            "table_top",
            w,
            0.03,
            d,
            (0, h - 0.015, z),
            M.wood,
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(
            f,
            "apron",
            w - 0.06,
            0.06,
            d - 0.06,
            (0, h - 0.06, z),
            M.wood,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "shelf",
            w - 0.07,
            0.02,
            d - 0.07,
            (0, 0.14, z),
            M.wood,
            bevel = 0.005,
            bevel_segments = 1
        );
        for lx in [-w / 2.0 + 0.05, w / 2.0 - 0.05] {
            for lz in [z - d / 2.0 + 0.05, z + d / 2.0 - 0.05] {
                fsweep!(
                    f,
                    numbered("leg"),
                    &[
                        (lx, h - 0.03, lz),
                        (
                            lx + 0.025_f64.copysign(lx),
                            0.0,
                            lz + 0.02_f64.copysign(lz - z)
                        )
                    ],
                    0.0,
                    M.wood,
                    profile = rounded_rect(0.034, 0.034, 0.012, 2),
                    up = (0, 0, 1),
                    scale = [1.0, 0.6]
                );
            }
        }
        // the smart speaker: a rounded fabric-covered cylinder, its light ring and buttons on top
        kit::lathe!(
            f,
            "speaker",
            -0.14,
            h,
            z - 0.02,
            &[
                (0.055, 0.0),
                (0.063, 0.008),
                (0.066, 0.03),
                (0.066, 0.18),
                (0.062, 0.196),
                (0.055, 0.2),
                (0.0, 0.2),
            ],
            M.speaker,
            verts = 24
        );
        torus!(
            "speaker_ring",
            0.056,
            0.005,
            at = f.at(-0.14, h + 0.2, z - 0.02),
            material = M.ring,
            major_segments = 24,
            minor_segments = 4
        );
        for k in 0..4 {
            let a = (45 + 90 * k) as f64;
            fcyl!(
                f,
                numbered("speaker_button"),
                0.007,
                0.003,
                (
                    -0.14 + 0.03 * cosd(a),
                    h + 0.2015,
                    z - 0.02 + 0.03 * sind(a)
                ),
                M.bezel,
                verts = 8
            );
        }
        // the tablet's dock, a few books on the shelf, a robot vacuum on the floor in front
        fbox!(
            f,
            "dock",
            0.2,
            0.025,
            0.1,
            (0.12, h + 0.0125, z + 0.02),
            M.white,
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(
            f,
            "dock_slot",
            0.18,
            0.004,
            0.016,
            (0.12, h + 0.025, z + 0.02),
            M.bezel
        );
        for (k, (bh, m, r)) in [(0.04, M.screen_light, 0), (0.035, M.app_yellow, 6)]
            .into_iter()
            .enumerate()
        {
            let kf = k as f64;
            let y = 0.15 + 0.04 * kf + bh / 2.0;
            fbox!(
                f,
                numbered("book"),
                0.22 - 0.02 * kf,
                bh,
                0.16 - 0.01 * kf,
                (-0.1, y, z),
                m,
                rot = (0, 0, r),
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("book_pages"),
                0.21 - 0.02 * kf,
                bh - 0.008,
                0.164 - 0.01 * kf,
                (-0.096, y, z),
                M.white,
                rot = (0, 0, r)
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the robot vacuum on the floor in front: drawn, not the hub's size
    let robot = new_parts(|| {
        kit::turned!(
            f,
            "robot",
            0,
            0.0,
            0.26,
            &[
                (0.0, 0.008),
                (0.14, 0.008),
                (0.15, 0.02),
                (0.15, 0.05),
                (0.14, 0.065),
                (0.0, 0.065),
            ],
            M.white,
            segments = 32
        );
        fcyl!(
            f,
            "robot_band",
            0.152,
            0.014,
            (0, 0.03, 0.26),
            M.bezel,
            verts = 32
        );
        fcyl!(
            f,
            "robot_turret",
            0.04,
            0.012,
            (0, 0.071, 0.2),
            M.bezel,
            verts = 14
        );
        fcyl!(
            f,
            "robot_button",
            0.025,
            0.004,
            (0, 0.067, 0.29),
            M.ring,
            verts = 12
        );
        for s in [-1, 1] {
            fcyl!(
                f,
                numbered("robot_wheel"),
                0.018,
                0.012,
                (s as f64 * 0.1, 0.018, 0.26),
                M.bezel,
                rot = (0, 90, 0),
                verts = 10
            );
        }
        for k in 0..3 {
            let a = 30 + 120 * k;
            fbox!(
                f,
                numbered("brush"),
                0.05,
                0.003,
                0.006,
                (
                    0.11 + 0.025 * cosd(a as f64),
                    0.006,
                    0.36 + 0.025 * sind(a as f64)
                ),
                M.bezel,
                rot = (0, 0, a)
            );
        }
    });
    aside(&robot, "robot");
    // The tablet, built standing up on its hinge in the dock, then laid back to rest: `screen_up` tilts it up.
    let (hy, hz) = (h + 0.02, z + 0.02);
    let tab = new_parts(|| {
        fbox!(
            f,
            "tablet",
            0.26,
            0.18,
            0.012,
            (0.12, hy + 0.09, hz),
            M.bezel,
            bevel = 0.006,
            bevel_segments = 2
        );
        fbox!(
            f,
            "tablet_screen",
            0.23,
            0.15,
            0.003,
            (0.12, hy + 0.09, hz + 0.006),
            M.screen_light,
            bevel = 0.002,
            bevel_segments = 1
        );
        for (m, ax, ay) in [
            (M.app_yellow, 0.06, 0.13),
            (M.led_green, 0.105, 0.13),
            (M.ring, 0.15, 0.13),
            (M.white, 0.06, 0.085),
            (M.app_yellow, 0.105, 0.085),
        ] {
            fbox!(
                f,
                numbered("app"),
                0.032,
                0.032,
                0.002,
                (ax, hy + ay, hz + 0.008),
                m,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "app_bar",
            0.2,
            0.025,
            0.002,
            (0.12, hy + 0.035, hz + 0.008),
            M.white,
            bevel = 0.004,
            bevel_segments = 1
        );
        fsphere!(
            f,
            "tablet_camera",
            0.003,
            (0.12, hy + 0.172, hz + 0.0065),
            M.screen,
            segments = 6,
            rings = 3
        );
    });
    let tab = hinged(join(&tab, "tablet", None), f.at(0.12, hy, hz));
    turn(tab, -72.0, Axis::X);
    let objs = settle!(centre = false);
    swing!(tab, "screen_up", 57.0, seconds = 0.8, axis = Axis::X);
    objs
}
