//! H3: the workbench's tool board and the tools hung on it.

use super::M;
use super::common::{link, up_quad};
use super::workbench::BOARD_Z;
use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr as r;
use artkit::kit;
use artkit::math::{V3, cos, radians, sin};
use artkit::place::{Facing, GameFrame};
use artkit::{fbox, fcyl, fextrude, fsweep, revolve, torus};

/// The hardboard pegboard on the back posts, a shelf along its top, and the tools hung on it: a saw, a square, two
/// screwdrivers, a spanner, pliers, a mallet and a tape measure.
pub fn tool_board(fr: &GameFrame) {
    // the board's face
    let z = BOARD_Z + 0.0075 + 0.001;
    fbox!(
        fr,
        "board",
        1.46,
        0.6,
        0.015,
        (0, 1.27, BOARD_Z),
        M.pegboard,
        bevel = 0.003,
        bevel_segments = 1
    );
    // its holes: dark squares on its face, one mesh (two triangles each)
    let mut holes = Vec::new();
    for row in 0..8 {
        for col in 0..18 {
            let (x, y) = (-0.68 + 0.08 * col as f64, 1.02 + 0.07 * row as f64);
            holes.push((x - 0.004, x + 0.004, y - 0.004, y + 0.004));
        }
    }
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    for (x0, x1, y0, y1) in holes {
        let start = verts.len() as u32;
        verts.extend(
            [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
                .iter()
                .map(|&(x, y)| V3::from(fr.at(x, y, z - 0.0005))),
        );
        faces.push((up_quad(&verts, start, |n| n.y > 0.0), 0));
    }
    link("pegs", verts, &faces, &[M.hole]);
    // the shelf on two shaped brackets, jars of screws on it
    fbox!(
        fr,
        "board_shelf",
        1.46,
        0.02,
        0.12,
        (0, 1.58, BOARD_Z + 0.065),
        M.bench_frame,
        bevel = 0.003,
        bevel_segments = 1
    );
    for x in [-0.5, 0.5] {
        fextrude!(
            fr,
            format!("shelf_bracket{}", r(x)),
            &[
                (0.0, 0.0),
                (0.0, 0.07),
                (0.1, 0.07),
                (0.1, 0.06),
                (0.06, 0.045),
                (0.025, 0.012),
                (0.012, 0.0)
            ],
            0.02,
            (x, 1.5, BOARD_Z + 0.0075),
            M.bench_frame,
            plane = Facing::Side,
            bevel = 0.002,
            bevel_segments = 1
        );
    }
    for (k, (x, r, h, colour, lid)) in [
        (-0.55, 0.035, 0.09, M.jar, M.ball_red),
        (-0.46, 0.03, 0.07, M.jar, M.vice),
        (0.5, 0.04, 0.1, M.jar, M.foot),
        (0.4, 0.03, 0.06, M.can, M.can),
    ]
    .into_iter()
    .enumerate()
    {
        revolve!(
            format!("shelf_jar{k}"),
            &[
                (0.0, 0.0),
                (r - 0.004, 0.0),
                (r, 0.006),
                (r, h - 0.012),
                (r * 0.8, h - 0.006),
                (0.0, h - 0.006)
            ],
            at = fr.at(x, 1.59, BOARD_Z + 0.06),
            material = colour,
            segments = 12
        );
        fcyl!(
            fr,
            format!("shelf_lid{k}"),
            r * 0.85,
            0.012,
            (x, 1.59 + h - 0.006, BOARD_Z + 0.06),
            lid,
            verts = 12,
            bevel = 0.002,
            bevel_segments = 1
        );
    }
    // a hand saw, its handle up: a tapering blade, the handle with its grip cut through
    fextrude!(
        fr,
        "saw_blade",
        &[(-0.035, 1.37), (-0.055, 0.95), (-0.03, 0.95), (0.045, 1.37)],
        0.0015,
        (-0.47, 0, z + 0.006),
        M.chrome
    );
    let handle = smooth_outline(
        &[
            (-0.06, 1.35),
            (0.06, 1.35),
            (0.065, 1.42),
            (0.04, 1.47),
            (-0.045, 1.47),
            (-0.065, 1.42),
        ],
        1,
        true,
    );
    let grip = [(-0.03, 1.38), (0.03, 1.38), (0.03, 1.44), (-0.03, 1.44)];
    let outline: Vec<(f64, f64)> = handle.iter().map(|&(x, y)| (x - 0.47, y)).collect();
    let hole: Vec<(f64, f64)> = grip.iter().map(|&(x, y)| (x - 0.47, y)).collect();
    fextrude!(
        fr,
        "saw_handle",
        &outline,
        0.022,
        (0, 0, z + 0.014),
        M.drill,
        holes = vec![hole],
        bevel = 0.004,
        bevel_segments = 1
    );
    for y in [1.36, 1.39] {
        fcyl!(
            fr,
            format!("saw_screw{}", r(y)),
            0.006,
            0.026,
            (-0.49, y, z + 0.014),
            M.brass,
            rot = (90, 0, 0),
            verts = 6
        );
    }
    // a carpenter's square
    fextrude!(
        fr,
        "square",
        &[
            (-0.315, 1.1),
            (-0.135, 1.1),
            (-0.135, 1.13),
            (-0.285, 1.13),
            (-0.285, 1.4),
            (-0.315, 1.4)
        ],
        0.004,
        (0, 0, z + 0.004),
        M.chrome
    );
    // two screwdrivers: turned handles, shafts, tips
    for (k, (x, colour)) in [(-0.12, M.ball_red), (-0.06, M.vice)]
        .into_iter()
        .enumerate()
    {
        revolve!(
            format!("driver_handle{k}"),
            &[
                (0.0, 0.0),
                (0.008, 0.0),
                (0.012, 0.012),
                (0.013, 0.08),
                (0.011, 0.095),
                (0.006, 0.1),
                (0.0, 0.1)
            ],
            at = fr.at(x, 1.42, z + 0.016),
            material = colour,
            segments = 8,
            rot = (180, 0, 0)
        );
        kit::tube!(
            fr,
            format!("driver_shaft{k}"),
            &[(x, 1.32, z + 0.016), (x, 1.2, z + 0.016)],
            0.003,
            M.chrome,
            verts = 6,
            round = 1
        );
        fbox!(
            fr,
            format!("driver_tip{k}"),
            0.008,
            0.012,
            0.0015,
            (x, 1.196, z + 0.016),
            M.chrome
        );
    }
    // a combination spanner: open jaw at the bottom, a ring at the top
    let jaw: Vec<(f64, f64)> = (110..431)
        .step_by(40)
        .map(|a| {
            (
                0.02 * cos(radians(a as f64)),
                1.13 + 0.02 * sin(radians(a as f64)),
            )
        })
        .collect();
    let mut spanner = vec![(0.007, 1.13), (0.007, 1.11)];
    spanner.extend_from_slice(&jaw[1..jaw.len() - 1]);
    spanner.extend([(-0.007, 1.11), (-0.007, 1.13)]);
    fextrude!(
        fr,
        "spanner_jaw",
        &spanner,
        0.005,
        (0.04, 0, z + 0.005),
        M.chrome
    );
    fbox!(
        fr,
        "spanner",
        0.014,
        0.2,
        0.004,
        (0.04, 1.25, z + 0.005),
        M.chrome
    );
    torus!(
        "spanner_ring",
        0.014,
        0.005,
        at = fr.at(0.04, 1.36, z + 0.005),
        material = M.chrome,
        rot = (90, 0, 0),
        major_segments = 12,
        minor_segments = 4
    );
    // pliers: two crossed handles, red-sleeved, from the jaws
    for s in [-1, 1] {
        let sf = s as f64;
        kit::tube!(
            fr,
            format!("plier{s}"),
            &[
                (0.16, 1.32, z + 0.008),
                (0.16 + sf * 0.006, 1.27, z + 0.008),
                (0.16 + sf * 0.028, 1.13, z + 0.008)
            ],
            0.005,
            M.chrome,
            verts = 6,
            round = 2
        );
        kit::tube!(
            fr,
            format!("plier_grip{s}"),
            &[
                (0.16 + sf * 0.01, 1.25, z + 0.008),
                (0.16 + sf * 0.028, 1.13, z + 0.008)
            ],
            0.0075,
            M.ball_red,
            verts = 6,
            round = 1
        );
    }
    fcyl!(
        fr,
        "plier_pivot",
        0.008,
        0.012,
        (0.16, 1.3, z + 0.008),
        M.chrome,
        rot = (90, 0, 0),
        verts = 8
    );
    // a mallet and a tape measure
    fsweep!(
        fr,
        "mallet_handle",
        &[(0.3, 1.08, z + 0.013), (0.3, 1.33, z + 0.013)],
        0,
        M.plank,
        profile = rounded_rect(0.024, 0.02, 0.008, 1),
        up = (1, 0, 0),
        scale = [1.0, 0.85]
    );
    let head = fsweep!(
        fr,
        "mallet_head",
        &[(0.235, 1.36, z + 0.032), (0.365, 1.36, z + 0.032)],
        0,
        M.plank,
        profile = rounded_rect(0.06, 0.06, 0.012, 2)
    );
    bevel_edges(head, 0.004, 1, 50.0, &EdgeOrder::Corners); // its faces' edges
    revolve!(
        "tape",
        &[
            (0.0, 0.0),
            (0.035, 0.0),
            (0.04, 0.005),
            (0.04, 0.035),
            (0.035, 0.04),
            (0.0, 0.04)
        ],
        at = fr.at(0.46, 1.3, z),
        material = M.drill,
        segments = 14,
        rot = (90, 0, 0)
    );
    fbox!(
        fr,
        "tape_blade",
        0.02,
        0.008,
        0.012,
        (0.47, 1.256, z + 0.02),
        M.chrome
    );
    fcyl!(
        fr,
        "tape_lock",
        0.008,
        0.006,
        (0.46, 1.3, z + 0.042),
        M.foot,
        rot = (90, 0, 0),
        verts = 8
    );
}
