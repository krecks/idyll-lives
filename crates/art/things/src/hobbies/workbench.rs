//! H3: the workbench: a thick butcher-block top on an open frame of square posts, rails and a slatted shelf below, the
//! back posts carrying a steel tool board on the wall with its tools hung up; a toolbox, a drill and a tin on the top,
//! and a bench vice holding a plank on edge. The top is at 0.90 m, a standing height (workbenches are 0.85–0.95; review
//! M-hobbies-01: it was a steel desk at 0.78). The vice's front jaw, its screw and handle are a part of their own and
//! slide shut on the plank while a resident tinkers (`vice`).

use super::M;
use super::common::{P, flat_quads};
use super::workbench_board::tool_board;
use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::kit::{self, fillet};
use artkit::math::{P3, PI, cos, sin};
use artkit::parts::{Settle, join, new_parts, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::shapes::SHARP;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, revolve, slide, soft};

/// The top's middle: its back on the wall face.
const BENCH_Z: f64 = -0.0975;
const BENCH_D: f64 = 0.665;
/// The top's surface.
const BENCH_TOP: f64 = 0.90;
/// How far the things on the top stand over where they stood on the old desk.
const BENCH_UP: f64 = BENCH_TOP - 0.7795;
const VICE_X: f64 = 0.1;
const VICE_Y: f64 = BENCH_TOP;
const VICE_GAP: f64 = 0.04;
/// The back jaw's middle.
const VICE_Z: f64 = 0.02;
/// The tool board's middle: its back on the wall face.
pub const BOARD_Z: f64 = -0.4225;

pub fn workbench() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (back, front) = (BENCH_Z - BENCH_D / 2.0, BENCH_Z + BENCH_D / 2.0);
    let still = new_parts(|| {
        // the butcher-block top: the strips' glue lines along it
        fbox!(
            f,
            "top",
            1.62,
            0.05,
            BENCH_D,
            (0, BENCH_TOP - 0.025, BENCH_Z),
            M.bench_top,
            bevel = 0.006
        );
        let lines: Vec<(f64, f64, f64, f64, f64)> = (1..11)
            .map(|k| {
                (
                    -0.809,
                    0.809,
                    back + 0.06 * k as f64 - 0.0006,
                    back + 0.06 * k as f64 + 0.0006,
                    BENCH_TOP + 0.0003,
                )
            })
            .collect();
        flat_quads(&f, "glue_lines", &lines, M.bench_frame);
        // the frame: four posts (the back two up to the tool board's top), rails under the top, stretchers and a
        // slatted shelf low down; bolts where the rails meet the posts
        let (post, rail_y) = (0.07, BENCH_TOP - 0.09);
        for sx in [-1, 1] {
            let x = sx as f64 * 0.73;
            fbox!(
                f,
                format!("post_front{sx}"),
                post,
                BENCH_TOP - 0.05,
                post,
                (x, (BENCH_TOP - 0.05) / 2.0, front - 0.06),
                M.bench_frame,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("post_back{sx}"),
                post,
                1.6,
                post,
                (x, 0.8, back + post / 2.0 + 0.015),
                M.bench_frame,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("rail_side{sx}"),
                0.05,
                0.08,
                BENCH_D - 0.12,
                (x, rail_y, BENCH_Z - 0.01),
                M.bench_frame
            );
            fbox!(
                f,
                format!("stretcher{sx}"),
                0.05,
                0.06,
                BENCH_D - 0.12,
                (x, 0.14, BENCH_Z - 0.01),
                M.bench_frame
            );
            for y in [rail_y, 0.14] {
                fcyl!(
                    f,
                    format!("bolt{sx}{}", r(y)),
                    0.009,
                    0.006,
                    (x, y, front - 0.06 + post / 2.0 + 0.002),
                    M.bench_steel,
                    rot = (90, 0, 0),
                    verts = 6
                );
                fcyl!(
                    f,
                    format!("bolt_side{sx}{}", r(y)),
                    0.009,
                    0.006,
                    (x + sx as f64 * (post / 2.0 + 0.002), y, front - 0.06),
                    M.bench_steel,
                    rot = (0, 90, 0),
                    verts = 6
                );
            }
            fbox!(
                f,
                format!("post_foot{sx}f"),
                post + 0.006,
                0.012,
                post + 0.006,
                (x, 0.006, front - 0.06),
                M.foot
            );
            fbox!(
                f,
                format!("post_foot{sx}b"),
                post + 0.006,
                0.012,
                post + 0.006,
                (x, 0.006, back + post / 2.0 + 0.015),
                M.foot
            );
        }
        for (z, name) in [(front - 0.06, "front"), (back + 0.05, "back")] {
            fbox!(
                f,
                format!("rail_{name}"),
                1.39,
                0.08,
                0.04,
                (0, rail_y, z),
                M.bench_frame
            );
            fbox!(
                f,
                format!("stretcher_{name}"),
                1.39,
                0.06,
                0.04,
                (0, 0.14, z),
                M.bench_frame
            );
        }
        for k in 0..5 {
            fbox!(
                f,
                format!("shelf_slat{k}"),
                1.39,
                0.02,
                0.09,
                (0, 0.18, back + 0.1 + 0.11 * k as f64),
                M.bench_top,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // on the shelf: a crate of offcuts (slatted sides) and a paint can with its handle
        let cz = BENCH_Z;
        fbox!(
            f,
            "crate_floor",
            0.4,
            0.012,
            0.34,
            (-0.4, 0.196, cz),
            M.toolbox
        );
        for s in [-1, 1] {
            let sf = s as f64;
            for y in [0.225, 0.32] {
                fbox!(
                    f,
                    format!("crate_slat{s}{}", r(y)),
                    0.4,
                    0.05,
                    0.012,
                    (-0.4, y, cz + sf * 0.164),
                    M.toolbox,
                    bevel = 0.002,
                    bevel_segments = 1
                );
                fbox!(
                    f,
                    format!("crate_end{s}{}", r(y)),
                    0.012,
                    0.05,
                    0.316,
                    (-0.4 + sf * 0.194, y, cz),
                    M.toolbox,
                    bevel = 0.002,
                    bevel_segments = 1
                );
            }
            fextrude!(
                f,
                format!("crate_grip{s}"),
                &rounded_rect(0.4, 0.06, 0.004, 1),
                0.012,
                (-0.4 + sf * 0.194, 0.37, cz),
                M.toolbox,
                holes = vec![rounded_rect(0.1, 0.022, 0.01, 2)],
                plane = Facing::Side,
                bevel = 0.002,
                bevel_segments = 1
            );
        }
        for (k, dx) in [-0.12, 0.0, 0.1].into_iter().enumerate() {
            let kf = k as f64;
            fbox!(
                f,
                format!("offcut{k}"),
                0.06,
                0.32 - 0.05 * kf,
                0.02,
                (-0.4 + dx, 0.35 + 0.02 * kf, BENCH_Z + 0.04 * kf),
                M.plank,
                rot = (8.0 - 6.0 * kf, 0.0, 0.0),
                bevel = 0.002,
                bevel_segments = 1
            );
        }
        revolve!(
            "paint_can",
            &[
                (0.0, 0.0),
                (0.088, 0.0),
                (0.09, 0.004),
                (0.09, 0.172),
                (0.092, 0.178),
                (0.086, 0.18),
                (0.0, 0.18)
            ],
            at = f.at(0.45, 0.19, BENCH_Z - 0.05),
            material = M.can,
            segments = 16
        );
        fcyl!(
            f,
            "paint_label",
            0.0905,
            0.09,
            (0.45, 0.27, BENCH_Z - 0.05),
            M.vice,
            verts = 16
        );
        let bail: Vec<P> = (0..9)
            .map(|i| {
                (
                    0.45 + 0.09 * cos(PI * i as f64 / 8.0),
                    0.36 + 0.05 * sin(PI * i as f64 / 8.0),
                    BENCH_Z - 0.05,
                )
            })
            .collect();
        kit::tube!(
            f,
            "paint_bail",
            &bail,
            0.0025,
            M.bench_steel,
            verts = 4,
            round = 1
        );
        tool_board(&f);
        // the toolbox: a box with a lid, its edges rounded, a carrying handle on two posts and two latches
        let u = BENCH_UP;
        fbox!(
            f,
            "toolbox",
            0.42,
            0.13,
            0.2,
            (-0.5, 0.845 + u, -0.3),
            M.toolbox,
            bevel = 0.012,
            bevel_segments = 2
        );
        soft!(
            f,
            "toolbox_lid",
            0.43,
            0.05,
            0.21,
            (-0.5, 0.93 + u, -0.3),
            M.toolbox_lid,
            round = 0.014,
            puff = 0.006,
            flat = 2,
            round_segments = 2
        );
        for x in [-0.62, -0.38] {
            fbox!(
                f,
                format!("latch{}", r(x)),
                0.04,
                0.035,
                0.012,
                (x, 0.905 + u, -0.195),
                M.chrome,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        kit::tube!(
            f,
            "toolbox_handle",
            &fillet(
                &[
                    (-0.6, 0.95 + u, -0.3),
                    (-0.6, 0.985 + u, -0.3),
                    (-0.4, 0.985 + u, -0.3),
                    (-0.4, 0.95 + u, -0.3)
                ],
                0.015,
                3
            ),
            0.009,
            M.foot,
            verts = 8,
            round = 1
        );
        // the drill lying on its side: a body, a grip and battery, a chuck and bit
        let drill = fsweep!(
            f,
            "drill_body",
            &[
                (0.37, 0.91 + u, -0.16),
                (0.44, 0.912 + u, -0.16),
                (0.52, 0.91 + u, -0.16),
                (0.545, 0.905 + u, -0.16)
            ],
            0.032,
            M.drill,
            verts = 12,
            resolution = 2,
            scale = [0.8, 1.0, 1.0, 0.6]
        );
        bevel_edges(drill, 0.003, 1, SHARP, &EdgeOrder::Corners); // its ends
        fsweep!(
            f,
            "drill_grip",
            &[(0.495, 0.89 + u, -0.16), (0.51, 0.82 + u, -0.16)],
            0,
            M.foot,
            profile = rounded_rect(0.045, 0.035, 0.012, 2),
            up = (1, 0, 0)
        );
        fbox!(
            f,
            "drill_battery",
            0.09,
            0.035,
            0.07,
            (0.51, 0.797 + u, -0.16),
            M.drill,
            bevel = 0.01,
            bevel_segments = 2
        );
        revolve!(
            "drill_chuck",
            &[
                (0.0, 0.0),
                (0.02, 0.0),
                (0.02, 0.02),
                (0.014, 0.045),
                (0.006, 0.05),
                (0.0, 0.05)
            ],
            at = f.at(0.37, 0.91 + u, -0.16),
            material = M.foot,
            segments = 10,
            rot = (0, -90, 0)
        );
        kit::tube!(
            f,
            "drill_bit",
            &[(0.32, 0.91 + u, -0.16), (0.26, 0.91 + u, -0.16)],
            0.004,
            M.chrome,
            verts = 6,
            round = 1
        );
        revolve!(
            "tin",
            &[
                (0.0, 0.0),
                (0.045, 0.0),
                (0.045, 0.054),
                (0.047, 0.058),
                (0.042, 0.06),
                (0.0, 0.06)
            ],
            at = f.at(0.15, 0.78 + u, -0.32),
            material = M.can,
            segments = 14
        );
        // a claw hammer and a pencil
        fsweep!(
            f,
            "hammer_handle",
            &[(-0.37, 0.791 + u, -0.033), (-0.19, 0.793 + u, 0.033)],
            0,
            M.plank,
            profile = rounded_rect(0.022, 0.03, 0.008, 1),
            scale = [0.9, 1.0]
        );
        fsweep!(
            f,
            "hammer_head",
            &[
                (-0.17, 0.795 + u, -0.025),
                (-0.21, 0.795 + u, 0.085),
                (-0.215, 0.795 + u, 0.1),
                (-0.205, 0.795 + u, 0.11)
            ],
            0.014,
            M.foot,
            verts = 8,
            scale = [1.1, 0.9, 0.7, 0.4]
        );
        fsweep!(
            f,
            "pencil",
            &[(-0.19, 0.785 + u, 0.176), (-0.05, 0.785 + u, 0.124)],
            0.005,
            M.drill,
            verts = 6
        );
        fsweep!(
            f,
            "pencil_tip",
            &[(-0.05, 0.785 + u, 0.124), (-0.035, 0.785 + u, 0.119)],
            0.005,
            M.plank,
            verts = 6,
            scale = [1.0, 0.1]
        );
        // the vice: a base bolted to the top, its back jaw, the plank clamped on edge
        fbox!(
            f,
            "vice_base",
            0.16,
            0.02,
            0.15,
            (VICE_X, VICE_Y + 0.01, VICE_Z + 0.02),
            M.vice,
            bevel = 0.005
        );
        for dx in [-0.06, 0.06] {
            fcyl!(
                f,
                format!("vice_bolt{}", r(dx)),
                0.008,
                0.006,
                (VICE_X + dx, VICE_Y + 0.023, VICE_Z + 0.07),
                M.bench_steel,
                verts = 6
            );
        }
        let jaw = fsweep!(
            f,
            "vice_jaw",
            &[
                (VICE_X - 0.075, VICE_Y + 0.055, VICE_Z),
                (VICE_X + 0.075, VICE_Y + 0.055, VICE_Z)
            ],
            0,
            M.vice,
            profile = vec![
                (-0.035, -0.02),
                (0.035, -0.02),
                (0.035, 0.0),
                (0.015, 0.02),
                (-0.035, 0.02)
            ]
        );
        bevel_edges(jaw, 0.003, 1, SHARP, &EdgeOrder::Corners);
        fbox!(
            f,
            "vice_pad",
            0.13,
            0.03,
            0.004,
            (VICE_X, VICE_Y + 0.07, VICE_Z + 0.022),
            M.bench_steel
        );
        fbox!(
            f,
            "plank",
            0.4,
            0.08,
            0.02,
            (VICE_X - 0.05, VICE_Y + 0.06, VICE_Z + 0.034),
            M.plank,
            bevel = 0.002
        );
    });
    join(&still, "body", Some(P3::ZERO));
    let v = new_parts(|| {
        let z = VICE_Z + 0.044 + VICE_GAP;
        let jaw = fsweep!(
            f,
            "vice_front",
            &[
                (VICE_X - 0.075, VICE_Y + 0.055, z + 0.02),
                (VICE_X + 0.075, VICE_Y + 0.055, z + 0.02)
            ],
            0,
            M.vice,
            profile = vec![
                (-0.035, -0.02),
                (0.015, -0.02),
                (0.035, 0.0),
                (0.035, 0.02),
                (-0.035, 0.02)
            ]
        );
        bevel_edges(jaw, 0.003, 1, SHARP, &EdgeOrder::Corners);
        fbox!(
            f,
            "vice_front_pad",
            0.13,
            0.03,
            0.004,
            (VICE_X, VICE_Y + 0.07, z + 0.002),
            M.bench_steel
        );
        for dx in [-0.05, 0.05] {
            kit::tube!(
                f,
                format!("vice_guide{}", r(dx)),
                &[
                    (VICE_X + dx, VICE_Y + 0.035, VICE_Z - 0.03),
                    (VICE_X + dx, VICE_Y + 0.035, z)
                ],
                0.006,
                M.chrome,
                verts = 8,
                round = 1
            );
        }
        kit::tube!(
            f,
            "vice_screw",
            &[
                (VICE_X, VICE_Y + 0.05, z + 0.04),
                (VICE_X, VICE_Y + 0.05, z + 0.09)
            ],
            0.007,
            M.chrome,
            verts = 8,
            round = 1
        );
        fcyl!(
            f,
            "vice_boss",
            0.014,
            0.02,
            (VICE_X, VICE_Y + 0.05, z + 0.08),
            M.chrome,
            rot = (90, 0, 0),
            verts = 10
        );
        kit::tube!(
            f,
            "vice_handle",
            &[
                (VICE_X - 0.07, VICE_Y + 0.05, z + 0.08),
                (VICE_X + 0.07, VICE_Y + 0.05, z + 0.08)
            ],
            0.005,
            M.chrome,
            verts = 8,
            round = 1
        );
        for dx in [-0.07, 0.07] {
            fsphere!(
                f,
                format!("vice_knob{}", r(dx)),
                0.01,
                (VICE_X + dx, VICE_Y + 0.05, z + 0.08),
                M.foot,
                segments = 8,
                rings = 6
            );
        }
    });
    let jaw = join(&v, "vice_jaw_front", None);
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    slide!(jaw, "vice", (0, VICE_GAP, 0), seconds = 0.9);
    meshes()
}
