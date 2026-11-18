//! H2: the chess table, for two (M-hobbies-11): a pedestal table in the middle of its 2 x 1 tiles with a board and a
//! game in play, and a chair at each end facing over it, where the content's two `places` seat the players (x = -0.66
//! and 0.66 m, their knees under the top): white plays from the left. A black pawn is its own part and slides one
//! square towards white while they play (`move`).

use super::M;
use super::common::{chair, flat_quads, rr, turned};
use artkit::materials::Mat;
use artkit::math::{Axis, M4, P3, V3, cosd, radians, sind, v3};
use artkit::parts::{Settle, hinged, join, new_parts, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsweep, revolve, slide};

/// The chairs' middles, either side (the content's `places`).
const CHESS_CHAIR: f64 = 0.66;
const SQUARE: f64 = 0.043;
/// The board's top.
const BOARD_Y: f64 = 0.744;

/// Game (x, depth) of a square's middle: columns across, rows from the back.
fn square_at(col: i32, row: i32) -> (f64, f64) {
    (-0.1505 + SQUARE * col as f64, 0.0995 + SQUARE * row as f64)
}

/// A turned chess piece on a square: pawn, rook, knight, bishop, queen or king; or standing at `at` (game x, height,
/// depth), off the board. The knight's head is carved in profile, facing the other side.
fn piece(
    fr: &GameFrame,
    name: &str,
    kind: &str,
    col: i32,
    row: i32,
    material: Mat,
    at: Option<(f64, f64, f64)>,
) {
    let (x, z) = match at {
        None => square_at(col, row),
        Some(a) => (a.0, a.2),
    };
    let y = at.map_or(BOARD_Y, |a| a.1);
    let mut profile = vec![(0.0, 0.0), (0.013, 0.0), (0.013, 0.004), (0.009, 0.008)];
    profile.extend_from_slice(match kind {
        "pawn" => &[
            (0.005, 0.022),
            (0.008, 0.025),
            (0.0075, 0.031),
            (0.005, 0.038),
            (0.0, 0.041),
        ][..],
        "rook" => &[
            (0.008, 0.038),
            (0.011, 0.04),
            (0.011, 0.05),
            (0.008, 0.05),
            (0.008, 0.046),
            (0.0, 0.046),
        ],
        "bishop" => &[
            (0.006, 0.034),
            (0.01, 0.036),
            (0.005, 0.039),
            (0.008, 0.046),
            (0.006, 0.054),
            (0.002, 0.057),
            (0.003, 0.06),
            (0.0, 0.062),
        ],
        "queen" => &[
            (0.006, 0.042),
            (0.011, 0.044),
            (0.006, 0.047),
            (0.009, 0.058),
            (0.011, 0.064),
            (0.006, 0.066),
            (0.004, 0.07),
            (0.0, 0.071),
        ],
        "king" => &[
            (0.006, 0.046),
            (0.011, 0.048),
            (0.006, 0.051),
            (0.009, 0.062),
            (0.009, 0.066),
            (0.0, 0.067),
        ],
        _ => &[(0.008, 0.014), (0.0, 0.014)],
    });
    revolve!(
        name,
        &profile,
        at = fr.at(x, y, z),
        material = material,
        segments = if kind == "pawn" { 6 } else { 8 }
    );
    if kind == "king" {
        fbox!(
            fr,
            format!("{name}_cross"),
            0.005,
            0.02,
            0.005,
            (x, y + 0.075, z),
            material
        );
        fbox!(
            fr,
            format!("{name}_arm"),
            0.014,
            0.005,
            0.005,
            (x, y + 0.077, z),
            material
        );
    }
    if kind == "knight" {
        let facing = if material == M.piece_dark { -1.0 } else { 1.0 };
        let head = [
            (-0.009, 0.012),
            (0.008, 0.012),
            (0.009, 0.026),
            (0.013, 0.036),
            (0.017, 0.044),
            (0.015, 0.05),
            (0.008, 0.051),
            (0.002, 0.056),
            (-0.004, 0.054),
            (-0.01, 0.046),
            (-0.012, 0.03),
        ];
        let outline: Vec<(f64, f64)> = head.iter().map(|&(u, v)| (u * facing, v)).collect();
        fextrude!(
            fr,
            format!("{name}_head"),
            &outline,
            0.011,
            (x, y, z),
            material,
            plane = Facing::Side
        );
    }
}

/// The table built as it was, white to the front (its middle at depth 0.25), then turned a quarter so white faces the
/// left-hand chair; the chairs built facing the front, turned to face it and moved out to their places.
pub fn chess_table() -> Vec<Obj> {
    // the table's middle on the footprint's
    let f = GameFrame::new(-0.25);
    let turn = M4::rotation(radians(90.0), Axis::Z);
    let still = new_parts(|| {
        // the top 0.48 × 0.64 m (0.64 between the players once turned), room at each player's end of the board for
        // the pieces they have taken and their hands (review M-hobbies-02: it was 0.46 × 0.40, the board filling it);
        // the board inlaid, framed in the table's wood
        fextrude!(
            f,
            "top",
            &rr(0.48, 0.64, 0.03, 3),
            0.03,
            (0, 0.725, 0.25),
            M.table_wood,
            plane = Facing::Top,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "board",
            0.37,
            0.004,
            0.37,
            (0, 0.742, 0.2505),
            M.table_wood,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "board_dark",
            0.344,
            0.0005,
            0.344,
            (0, 0.74425, 0.2505),
            M.square_dark
        );
        let mut quads = Vec::new();
        for col in 0..8 {
            for row in 0..8 {
                if (col + row) % 2 == 0 {
                    let (x, z) = square_at(col, row);
                    quads.push((
                        x - SQUARE / 2.0,
                        x + SQUARE / 2.0,
                        z - SQUARE / 2.0,
                        z + SQUARE / 2.0,
                        0.7448,
                    ));
                }
            }
        }
        flat_quads(&f, "squares", &quads, M.square_light);
        fbox!(
            f,
            "apron",
            0.4,
            0.05,
            0.56,
            (0, 0.685, 0.25),
            M.table_wood,
            bevel = 0.004,
            bevel_segments = 1
        );
        // a turned pedestal on three swept snake feet
        turned(
            &f,
            "pedestal",
            0,
            0.04,
            0.25,
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.05, 0.03),
                (0.036, 0.05),
                (0.03, 0.1),
                (0.044, 0.16),
                (0.048, 0.2),
                (0.034, 0.26),
                (0.025, 0.42),
                (0.03, 0.55),
                (0.042, 0.6),
                (0.06, 0.62),
                (0.06, 0.64),
                (0.0, 0.64),
            ],
            M.table_wood,
            10,
            P3::ONE,
        );
        for k in 0..3 {
            let a = (90 + 120 * k) as f64;
            let (c, s) = (cosd(a), sind(a));
            let pts = [
                (0.03 * c, 0.13, 0.25 + 0.03 * s),
                (0.09 * c, 0.11, 0.25 + 0.09 * s),
                (0.15 * c, 0.055, 0.25 + 0.15 * s),
                (0.2 * c, 0.016, 0.25 + 0.2 * s),
                (0.22 * c, 0.012, 0.25 + 0.22 * s),
            ];
            fsweep!(
                f,
                format!("foot{k}"),
                &pts,
                0,
                M.table_wood,
                profile = rr(0.05, 0.032, 0.012, 1),
                scale = [1.0, 0.95, 0.75, 0.5, 0.48]
            );
        }
        // the game: the white side to the front (to the left once turned)
        for (kind, col, row) in [
            ("rook", 0, 7),
            ("knight", 6, 7),
            ("bishop", 3, 6),
            ("king", 4, 7),
        ] {
            piece(
                &f,
                &format!("w_{kind}"),
                kind,
                col,
                row,
                M.piece_light,
                None,
            );
        }
        for (k, (col, row)) in [(1, 6), (2, 5), (4, 4), (6, 6)].into_iter().enumerate() {
            piece(
                &f,
                &format!("w_pawn{k}"),
                "pawn",
                col,
                row,
                M.piece_light,
                None,
            );
        }
        for (kind, col, row) in [
            ("rook", 7, 0),
            ("knight", 1, 0),
            ("bishop", 5, 3),
            ("king", 4, 0),
            ("queen", 3, 1),
        ] {
            piece(&f, &format!("b_{kind}"), kind, col, row, M.piece_dark, None);
        }
        for (k, (col, row)) in [(2, 1), (3, 3), (6, 2)].into_iter().enumerate() {
            piece(
                &f,
                &format!("b_pawn{k}"),
                "pawn",
                col,
                row,
                M.piece_dark,
                None,
            );
        }
        // the taken pieces beside the board, at the end of the one who took them
        for (k, (kind, x, z)) in [("pawn", 0.14, -0.03), ("pawn", 0.19, -0.05)]
            .into_iter()
            .enumerate()
        {
            piece(
                &f,
                &format!("taken_w{k}"),
                kind,
                0,
                0,
                M.piece_light,
                Some((x, 0.74, z)),
            );
        }
        for (k, (kind, x, z)) in [("pawn", -0.14, 0.53), ("pawn", -0.19, 0.55)]
            .into_iter()
            .enumerate()
        {
            piece(
                &f,
                &format!("taken_b{k}"),
                kind,
                0,
                0,
                M.piece_dark,
                Some((x, 0.74, z)),
            );
        }
    });
    let table = join(&still, "table", Some(P3::ZERO));
    table.edit(|o| o.mesh.transform(&turn));
    let mut parts = vec![table];
    for s in [-1, 1] {
        let chair_parts = new_parts(|| {
            chair(
                &GameFrame::new(0.0),
                0.0,
                M.chair_wood,
                M.seat_pad,
                0.42,
                0.38,
            )
        });
        let c = join(&chair_parts, &format!("chair{s}"), Some(P3::ZERO));
        // facing the table: the left one (game x -0.66) turned to face right, the right one left
        let m = M4::translation(v3(-s as f64 * CHESS_CHAIR, 0, 0))
            * M4::rotation(radians((90 * s) as f64), Axis::Z);
        c.edit(|o| o.mesh.transform(&m));
        parts.push(c);
    }
    join(&parts, "body", Some(P3::ZERO));
    let p = new_parts(|| piece(&f, "pawn", "pawn", 5, 1, M.piece_dark, None));
    let pawn = join(&p, "pawn", Some(P3::ZERO));
    pawn.edit(|o| o.mesh.transform(&turn));
    let (x, z) = square_at(5, 1);
    hinged(pawn, turn * V3::from(f.at(x, BOARD_Y, z)));
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    // one square on, towards white (game -x)
    slide!(pawn, "move", (SQUARE, 0, 0), seconds = 0.8);
    meshes()
}
