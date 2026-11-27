use super::super::common::{F, M};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::P3;
use artkit::motion::{Slide, slide};
use artkit::parts::{hinged, join, join_all, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{cube, fbox, fsweep, revolve, settle, sphere};

// A stone table with the board inlaid, its top 0.7 m square, and a stone stool either side of it, left and right,
// where the content's two places seat the players facing each other across the board (x = ±0.6 m, 0.47 m up, on
// 2 x 1 tiles: their knees clear of each other under the top; they were ±0.42 on one tile). White plays from the
// left. A white pawn slides one square on (`move`) as they play.
pub(super) const BOARD: (f64, f64) = (0.762, 0.0); // (height, depth) of the board's middle
pub(super) const SQUARE: f64 = 0.05;
pub(super) const STOOL: f64 = 0.6; // the stools' middles, either side (the content's `places`)
pub(super) const CHESS_TOP: f64 = 0.7; // the top, square

/// The game (x, depth) of square (file i, rank j), rank 0 nearest white on the left, file 0 on white's
/// left (to the front).
pub(super) fn square(i: i32, j: i32) -> (f64, f64) {
    (
        -3.5 * SQUARE + j as f64 * SQUARE,
        BOARD.1 + 3.5 * SQUARE - i as f64 * SQUARE,
    )
}

/// The two stools: a turned stone mushroom, a moulded foot, a waisted stem, a seat with a rounded edge.
pub(super) fn stools(segments: usize) {
    for s in [-1, 1] {
        revolve!(
            format!("stool{s}"),
            &[
                (0.0, 0.0),
                (0.11, 0.0),
                (0.11, 0.025),
                (0.095, 0.035),
                (0.075, 0.06),
                (0.062, 0.12),
                (0.058, 0.3),
                (0.066, 0.37),
                (0.09, 0.41),
                (0.12, 0.43),
                (0.125, 0.45),
                (0.12, 0.465),
                (0.1, 0.47),
                (0.0, 0.47)
            ],
            at = F.at(s as f64 * STOOL, 0, 0),
            material = M.stool,
            segments = segments
        );
    }
}

pub(super) fn pedestal(segments: usize) {
    revolve!(
        "pedestal",
        &[
            (0.0, 0.0),
            (0.17, 0.0),
            (0.17, 0.03),
            (0.15, 0.045),
            (0.11, 0.08),
            (0.09, 0.14),
            (0.08, 0.55),
            (0.09, 0.6),
            (0.12, 0.66),
            (0.13, 0.68),
            (0.12, 0.7),
            (0.0, 0.7)
        ],
        at = F.at(0, 0, BOARD.1),
        material = M.stool,
        segments = segments
    );
}

/// A chess piece standing on square (i, j): turned, a knight's head swept, the king's cross, the queen's crown.
pub(super) fn piece(name: &str, kind: &str, i: i32, j: i32, material: Mat, low: bool) {
    let (x, z) = square(i, j);
    let y = BOARD.0 + 0.002;
    let at = F.at(x, y, z);
    let seg: usize = if low { 8 } else { 10 };
    let foot = [(0.0, 0.0), (0.017, 0.0), (0.017, 0.005), (0.014, 0.008)];
    let with =
        |more: &[(f64, f64)]| -> Vec<(f64, f64)> { foot.iter().chain(more).copied().collect() };
    match kind {
        "pawn" => {
            revolve!(
                name,
                &with(&[
                    (0.008, 0.018),
                    (0.006, 0.026),
                    (0.011, 0.029),
                    (0.006, 0.032),
                    (0.0, 0.033)
                ]),
                at = at,
                material = material,
                segments = seg
            );
            sphere!(
                format!("{name}_head"),
                0.011,
                at = (at.x, at.y, at.z + 0.041),
                material = material,
                segments = 8,
                rings = 5
            );
        }
        "rook" => {
            revolve!(
                name,
                &with(&[
                    (0.011, 0.014),
                    (0.01, 0.038),
                    (0.016, 0.042),
                    (0.016, 0.056),
                    (0.011, 0.056),
                    (0.011, 0.05),
                    (0.0, 0.05)
                ]),
                at = at,
                material = material,
                segments = seg
            );
        }
        "knight" => {
            revolve!(
                name,
                &with(&[(0.012, 0.014), (0.0, 0.016)]),
                at = at,
                material = material,
                segments = seg
            );
            let facing = if name.starts_with("black") { -1.0 } else { 1.0 };
            fsweep!(
                F,
                format!("{name}_head"),
                &[
                    (x, y + 0.012, z),
                    (x + facing * 0.004, y + 0.035, z),
                    (x + facing * 0.012, y + 0.05, z),
                    (x + facing * 0.024, y + 0.044, z)
                ],
                0,
                material,
                profile = rounded_rect(0.02, 0.014, 0.005, 1),
                up = (0, 0, 1),
                resolution = 2,
                scale = [1.3, 0.9, 0.9, 0.6]
            );
        }
        "bishop" => {
            revolve!(
                name,
                &with(&[
                    (0.008, 0.02),
                    (0.006, 0.034),
                    (0.011, 0.037),
                    (0.007, 0.04),
                    (0.01, 0.048),
                    (0.008, 0.056),
                    (0.003, 0.062),
                    (0.0, 0.064)
                ]),
                at = at,
                material = material,
                segments = seg
            );
        }
        _ => {
            // king or queen: taller, with a cap
            let h = if kind == "king" { 0.07 } else { 0.064 };
            revolve!(
                name,
                &[
                    (0.0, 0.0),
                    (0.019, 0.0),
                    (0.019, 0.007),
                    (0.012, 0.014),
                    (0.007, h - 0.02),
                    (0.012, h - 0.016),
                    (0.008, h - 0.013),
                    (0.014, h - 0.004),
                    (0.009, h),
                    (0.0, h)
                ],
                at = at,
                material = material,
                segments = seg
            );
            if kind == "king" {
                cube!(
                    format!("{name}_cross"),
                    (0.006, 0.006, 0.022),
                    at = (at.x, at.y, at.z + h + 0.01),
                    material = material
                );
                cube!(
                    format!("{name}_bar"),
                    (0.016, 0.006, 0.006),
                    at = (at.x, at.y, at.z + h + 0.013),
                    material = material
                );
            } else {
                sphere!(
                    format!("{name}_crown"),
                    0.008,
                    at = (at.x, at.y, at.z + h + 0.004),
                    material = material,
                    segments = 8,
                    rings = 5
                );
            }
        }
    }
}

pub(super) const WHITE: [(&str, i32, i32); 8] = [
    ("king", 4, 0),
    ("rook", 0, 0),
    ("pawn", 1, 1),
    ("pawn", 5, 1),
    ("pawn", 6, 2),
    ("queen", 2, 3),
    ("knight", 5, 2),
    ("bishop", 3, 2),
];
pub(super) const BLACK: [(&str, i32, i32); 8] = [
    ("king", 4, 7),
    ("rook", 7, 7),
    ("pawn", 2, 6),
    ("pawn", 3, 5),
    ("pawn", 6, 6),
    ("queen", 5, 5),
    ("knight", 1, 5),
    ("bishop", 4, 6),
];

/// The table's top: a slab with a chamfered edge and a moulded band under it, the board inlaid in a border.
pub(super) fn top() {
    fbox!(
        F,
        "top",
        CHESS_TOP,
        0.05,
        CHESS_TOP,
        (0, 0.735, BOARD.1),
        M.stone,
        bevel = 0.015,
        bevel_segments = 2
    );
    fbox!(
        F,
        "top_under",
        CHESS_TOP - 0.06,
        0.02,
        CHESS_TOP - 0.06,
        (0, 0.705, BOARD.1),
        M.stone,
        bevel = 0.006,
        bevel_segments = 1
    );
    fbox!(
        F,
        "board",
        8.0 * SQUARE + 0.03,
        0.003,
        8.0 * SQUARE + 0.03,
        (0, BOARD.0 - 0.0005, BOARD.1),
        M.square_dark
    );
    fbox!(
        F,
        "board_light",
        8.0 * SQUARE,
        0.003,
        8.0 * SQUARE,
        (0, BOARD.0 + 0.0005, BOARD.1),
        M.square_light
    );
    for i in 0..8 {
        for j in 0..8 {
            if (i + j) % 2 == 0 {
                let (x, z) = square(i, j);
                fbox!(
                    F,
                    format!("sq{i}{j}"),
                    SQUARE,
                    0.003,
                    SQUARE,
                    (x, BOARD.0 + 0.001, z),
                    M.square_dark
                );
            }
        }
    }
}

pub fn park_chess_table() -> Vec<Obj> {
    let still = new_parts(|| {
        pedestal(16);
        top();
        stools(16);
        // a game under way: white on the left, black across; a taken black pawn at white's side of the board
        for (k, (kind, i, j)) in WHITE.into_iter().enumerate() {
            piece(&format!("white{k}"), kind, i, j, M.piece_white, false);
        }
        for (k, (kind, i, j)) in BLACK.into_iter().enumerate() {
            piece(&format!("black{k}"), kind, i, j, M.piece_black, false);
        }
        revolve!(
            "taken",
            &[
                (0.0, 0.0),
                (0.017, 0.0),
                (0.017, 0.005),
                (0.008, 0.02),
                (0.0, 0.022)
            ],
            at = F.at(-0.29, BOARD.0 - 0.002, 0.25),
            material = M.piece_black,
            segments = 10
        );
    });
    join(&still, "table", Some(P3::from((0, 0, 0))));
    let parts = new_parts(|| piece("pawn", "pawn", 4, 1, M.piece_white, false));
    let (x, z) = square(4, 1);
    let pawn = hinged(join(&parts, "pawn", None), F.at(x, BOARD.0, z));
    settle!(centre = false);
    slide(
        pawn,
        "move",
        (-SQUARE, 0, 0),
        Slide {
            seconds: 0.6,
            ..Default::default()
        },
    ); // one square on, towards black (game +x)
    meshes()
}

/// Its far version by hand (decimating it folds the inlaid board into spikes): the table, the board's
/// squares and the stools at fewer sides, the two kings, no other pieces.
pub fn park_chess_table_far() -> Vec<Obj> {
    pedestal(12);
    top();
    stools(12);
    piece("white_king", "king", 4, 0, M.piece_white, true);
    piece("black_king", "king", 4, 7, M.piece_black, true);
    join_all("parkChessTable_far");
    settle!(centre = false);
    meshes()
}
