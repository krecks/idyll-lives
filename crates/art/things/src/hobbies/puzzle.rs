//! H2: the jigsaw table: a small table to the front with a puzzle board on it, half the picture done, loose pieces and
//! the box; a stool behind. A patch of finished puzzle spreads across while a resident plays (`pieces`), from the done
//! half's edge.

use super::M;
use super::common::stool;
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::kit;
use artkit::math::{Axis, P3, PI, cosd, sin, sind};
use artkit::motion::{Grow, grow};
use artkit::parts::{Settle, join, new_parts, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude};

/// The board's top.
const PUZZLE_Y: f64 = 0.7585;
/// Where the done half ends, game x.
const PUZZLE_EDGE: f64 = 0.0;

/// The jigsaw's picture between game x `x0` and `x1`: sky, a hill, a band of flowers.
fn picture(fr: &GameFrame, name: &str, x0: f64, x1: f64) {
    let (w, x) = (x1 - x0, (x0 + x1) / 2.0);
    fbox!(
        fr,
        format!("{name}_sky"),
        w,
        0.0015,
        0.18,
        (x, PUZZLE_Y + 0.0008, 0.11),
        M.sky
    );
    fbox!(
        fr,
        format!("{name}_hill"),
        w,
        0.0015,
        0.14,
        (x, PUZZLE_Y + 0.0008, 0.27),
        M.hill
    );
    fbox!(
        fr,
        format!("{name}_flowers"),
        w,
        0.0015,
        0.04,
        (x, PUZZLE_Y + 0.0008, 0.36),
        M.blush
    );
}

/// A jigsaw piece's outline (centred, `s` a side): a knob out on one edge, a socket in another.
fn jigsaw_piece(s: f64) -> Vec<(f64, f64)> {
    let h = s / 2.0;
    let mut out = vec![(-h, -h), (h, -h)];
    out.extend(
        (-130..131)
            .step_by(52)
            .map(|a| (h + 0.007 + 0.009 * cosd(a as f64), 0.009 * sind(a as f64))),
    );
    out.push((h, h));
    out.extend(
        (0..)
            .map(|i| 50 - 56 * i)
            .take_while(|&a| a > -231)
            .map(|a| (0.009 * cosd(a as f64), h - 0.007 + 0.009 * sind(a as f64))),
    );
    out.push((-h, h));
    out
}

pub fn puzzle_table() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        fextrude!(
            f,
            "top",
            &rounded_rect(0.86, 0.5, 0.02, 2),
            0.035,
            (0, 0.735, 0.2),
            M.table_wood,
            plane = Facing::Top,
            bevel = 0.006,
            bevel_segments = 1
        );
        for sx in [-1, 1] {
            for z in [0.0, 0.4] {
                kit::turned!(
                    f,
                    format!("leg{sx}{}", r(z)),
                    sx as f64 * 0.38,
                    0.0,
                    z,
                    &[
                        (0.0, 0.0),
                        (0.019, 0.0),
                        (0.023, 0.02),
                        (0.018, 0.05),
                        (0.021, 0.42),
                        (0.028, 0.52),
                        (0.024, 0.57),
                        (0.028, 0.6),
                        (0.028, 0.72),
                        (0.0, 0.72),
                    ],
                    M.table_wood,
                    segments = 10
                );
            }
            fextrude!(
                f,
                format!("apron_side{sx}"),
                &[
                    (0.37, 0.72),
                    (0.03, 0.72),
                    (0.03, 0.675),
                    (0.12, 0.665),
                    (0.2, 0.662),
                    (0.28, 0.665),
                    (0.37, 0.675)
                ],
                0.02,
                (sx as f64 * 0.38, 0, 0),
                M.table_wood,
                plane = Facing::Side,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for z in [0.0, 0.4] {
            let edge: Vec<(f64, f64)> = (0..13)
                .map(|i| {
                    (
                        0.36 - 0.72 * i as f64 / 12.0,
                        0.672 - 0.012 * sin(PI * i as f64 / 12.0),
                    )
                })
                .collect();
            let mut outline = vec![(0.36, 0.72), (-0.36, 0.72)];
            outline.extend(edge.iter().rev());
            fextrude!(
                f,
                format!("apron{}", r(z)),
                &outline,
                0.02,
                (0, 0, z),
                M.table_wood,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // the puzzle board with a raised rim, half the picture done
        fbox!(
            f,
            "board",
            0.58,
            0.006,
            0.4,
            (0, 0.7555, 0.2),
            M.puzzle_board,
            bevel = 0.002,
            bevel_segments = 1
        );
        kit::moulding!(
            f,
            "board_rim",
            &[(0.29, 0.4), (-0.29, 0.4), (-0.29, 0.0), (0.29, 0.0)],
            &[
                (0.0, 0.0),
                (0.0, 0.01),
                (0.004, 0.012),
                (0.008, 0.01),
                (0.008, 0.0)
            ],
            M.puzzle_board,
            at = 0.7525,
            plane = Facing::Top
        );
        picture(&f, "done", -0.27, PUZZLE_EDGE);
        fcyl!(
            f,
            "sun",
            0.035,
            0.0015,
            (-0.17, PUZZLE_Y + 0.0012, 0.08),
            M.sun,
            verts = 16,
            smooth = false
        );
        for (k, (x, z, yaw, colour)) in [
            (-0.36, 0.36, 23, M.blush),
            (-0.21, 0.43, 42, M.sky),
            (0.36, 0.05, 78, M.hill),
            (0.33, 0.17, 72, M.hill),
            (0.2, 0.31, 31, M.sky),
            (0.24, 0.12, 55, M.blush),
        ]
        .into_iter()
        .enumerate()
        {
            let y = if f64::abs(x) > 0.29 { 0.7555 } else { 0.7615 };
            fextrude!(
                f,
                format!("loose{k}"),
                &jigsaw_piece(0.045),
                0.004,
                (x, y, z),
                colour,
                plane = Facing::Top,
                rot = (0, 0, yaw)
            );
        }
        // the box: its lid on, a little proud of the base, the picture on it
        fbox!(
            f,
            "box",
            0.136,
            0.032,
            0.196,
            (0.365, 0.7685, 0.3),
            M.card,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "lid",
            0.142,
            0.016,
            0.202,
            (0.365, 0.7845, 0.3),
            M.card,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "box_picture",
            0.12,
            0.0015,
            0.09,
            (0.365, 0.793, 0.26),
            M.sky
        );
        fbox!(
            f,
            "box_picture_hill",
            0.12,
            0.0015,
            0.08,
            (0.365, 0.793, 0.345),
            M.hill
        );
        stool(&f, -0.25, 0.48, 0.18, M.chair_wood, M.chair_wood, 0.0);
    });
    join(&still, "body", Some(P3::ZERO));
    let p = new_parts(|| {
        picture(&f, "patch", PUZZLE_EDGE, 0.15);
        fbox!(
            f,
            "patch_cloud",
            0.06,
            0.0015,
            0.03,
            (0.09, PUZZLE_Y + 0.0012, 0.08),
            M.label
        );
    });
    let patch = join(&p, "patch", Some(f.at(PUZZLE_EDGE, PUZZLE_Y, 0.2)));
    patch.edit(|o| o.scale.x = 0.001);
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    grow(
        patch,
        "pieces",
        Grow {
            seconds: 1.2,
            axis: Axis::X,
        },
    );
    meshes()
}
