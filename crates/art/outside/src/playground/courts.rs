//! The games: the basketball hoop, the ping-pong table and the park chess table.

use super::common::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, TAU, cos, sin};
use artkit::motion::{Rock, Slide, Squash, rock, slide, squash};
use artkit::parts::{hinged, join, join_all, new_parts};
use artkit::place::Facing;
use artkit::place::N;
use artkit::scene::{Obj, meshes};
use artkit::{
    cube, fbox, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, settle, soft, sphere, torus,
};

// A hoop on a post with a ballast base at the back of its tile; the rim at 3.05 m (regulation, park and driveway
// hoops alike) reaches forward from a 1.2 x 0.8 m board (a park one's; they were 2.94 m and 0.95 x 0.62). The net
// is its own part, hung from the rim: `swish` sways and stretches it once a ball goes through.
const RIM: (f64, f64) = (3.05, 0.27); // (height, depth) of the rim's middle
const BOARD_W: f64 = 1.2; // the backboard: its width, height and middle's height
const BOARD_H: f64 = 0.8;
const BOARD_Y: f64 = 3.3;

/// A diamond-mesh net: strands zigzagging from the rim (r 0.21) to a narrower bottom ring (r 0.13), each strand
/// one cord, a hem round the bottom.
fn net(name: &str) -> Obj {
    let (y0, y1, r0, r1, n) = (RIM.0 - 0.01, RIM.0 - 0.38, 0.205, 0.13, 12);
    let z = RIM.1;
    let parts = new_parts(|| {
        let rows = 4;
        for i in 0..n {
            for d in [-1, 1] {
                let mut pts = Vec::new();
                for j in 0..=rows {
                    let t = j as f64 / rows as f64;
                    let a = TAU * (i as f64 + d as f64 * (if j % 2 == 1 { 0.5 } else { 0.0 }))
                        / n as f64;
                    let r = r0 + (r1 - r0) * t;
                    pts.push((r * cos(a), y0 + (y1 - y0) * t, z + r * sin(a)));
                }
                fsweep!(F, format!("{name}{i}{d}"), &pts, 0.006, M.net, verts = 4);
            }
        }
        torus!(
            format!("{name}_hem"),
            r1,
            0.008,
            at = F.at(0, y1, z),
            material = M.net,
            major_segments = 16,
            minor_segments = 4
        );
    });
    join(&parts, name, None)
}

pub fn basketball_hoop() -> Vec<Obj> {
    let still = new_parts(|| {
        // the base: a moulded ballast tank, its fill cap, two wheels at its front edge for rolling it
        let p = F.at(0, 0, -0.17);
        let (bx, by) = (p.x, p.y);
        loft!(
            "ballast",
            &[
                ring!(0.31, 0.3, z = 0.0, verts = 24, square = 6, at = (bx, by)),
                ring!(0.31, 0.3, z = 0.18, verts = 24, square = 6, at = (bx, by)),
                ring!(0.28, 0.27, z = 0.22, verts = 24, square = 6, at = (bx, by)),
                ring!(0.2, 0.19, z = 0.235, verts = 24, square = 5, at = (bx, by))
            ],
            material = M.post,
            angle = 50
        );
        fcyl!(
            F,
            "ballast_cap",
            0.05,
            0.02,
            (0.16, 0.23, -0.05),
            M.steel,
            verts = 12,
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            fcyl!(
                F,
                format!("wheel{s}"),
                0.05,
                0.035,
                (s as f64 * 0.24, 0.05, 0.14),
                M.rubber,
                rot = (0, 90, 0),
                verts = 14,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        // the pole in two telescoping sections, a clamp collar between them; a cap on top
        fcyl!(
            F,
            "pole",
            0.05,
            1.85,
            (0, 0.21 + 0.925, -0.3),
            M.post,
            verts = 14
        );
        fcyl!(
            F,
            "pole_upper",
            0.044,
            1.36,
            (0, 2.0 + 0.68, -0.3),
            M.post,
            verts = 14
        );
        revolve!(
            "pole_collar",
            &[
                (0.0, -0.04),
                (0.058, -0.04),
                (0.062, -0.03),
                (0.062, 0.03),
                (0.055, 0.04),
                (0.0, 0.04)
            ],
            at = F.at(0, 2.02, -0.3),
            material = M.steel,
            segments = 14
        );
        fsweep!(
            F,
            "collar_knob",
            &[(0, 2.02, -0.24), (0, 2.02, -0.2)],
            0.012,
            M.rubber,
            verts = 6
        );
        fcyl!(
            F,
            "pole_cap",
            0.055,
            0.03,
            (0, 3.36, -0.3),
            M.steel,
            verts = 14,
            bevel = 0.006,
            bevel_segments = 1
        );
        // two bent arms from the pole to the board's back, a strut between them
        for (y, to) in [(3.3, 3.45), (3.0, 3.08)] {
            fsweep!(
                F,
                format!("arm{}", float_repr(y)),
                &kit::fillet!(&[(0.0, y, -0.3), (0.0, y, -0.2), (0.0, to, 0.0)], 0.04),
                0.025,
                M.post,
                verts = 8
            );
        }
        fsweep!(
            F,
            "arm_strut",
            &[(0, 3.06, -0.18), (0, 3.38, -0.12)],
            0.016,
            M.post,
            verts = 6
        );
        // the backboard: a white board in a steel frame, its border and the shooter's square, a pad along its foot
        let (bw, bh, by) = (BOARD_W, BOARD_H, BOARD_Y);
        fbox!(
            F,
            "board",
            bw - 0.02,
            bh - 0.02,
            0.025,
            (0, by, 0.04),
            M.board,
            bevel = 0.004,
            bevel_segments = 1
        );
        kit::moulding!(
            F,
            "board_frame",
            &[
                (bw / 2.0, by - bh / 2.0),
                (bw / 2.0, by + bh / 2.0),
                (-bw / 2.0, by + bh / 2.0),
                (-bw / 2.0, by - bh / 2.0),
            ],
            &[(0.0, -0.02), (0.0, 0.016), (0.022, 0.016), (0.022, -0.02)],
            M.post,
            at = 0.04
        );
        for y in [by + bh / 2.0 - 0.045, by - bh / 2.0 + 0.045] {
            fbox!(
                F,
                format!("border{}", float_repr(y)),
                bw - 0.07,
                0.03,
                0.028,
                (0, y, 0.04),
                M.board_mark
            );
        }
        for x in [-bw / 2.0 + 0.045, bw / 2.0 - 0.045] {
            fbox!(
                F,
                format!("border{}", float_repr(x)),
                0.03,
                bh - 0.06,
                0.028,
                (x, by, 0.04),
                M.board_mark
            );
        }
        let (tw, th) = (0.59, 0.45); // the shooter's square, 0.59 x 0.45 m as on a regulation board, its foot level with the rim
        let ty = RIM.0 + th / 2.0;
        fextrude!(
            F,
            "target",
            &rounded_rect(tw, th, 0.004, 1),
            0.028,
            (0, ty, 0.045),
            M.board_mark,
            holes = [rounded_rect(tw - 0.05, th - 0.05, 0.003, 1)]
        );
        soft!(
            F,
            "pad",
            bw,
            0.04,
            0.06,
            (0, by - bh / 2.0 - 0.01, 0.04),
            M.rubber,
            round = 0.015
        );
        // the rim on a flanged bracket with its spring housing, little hooks for the net under it
        fbox!(
            F,
            "rim_bracket",
            0.16,
            0.12,
            0.05,
            (0, RIM.0 - 0.04, 0.075),
            M.rim,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "rim_spring",
            0.06,
            0.07,
            0.04,
            (0, RIM.0 - 0.05, 0.1),
            M.steel,
            bevel = 0.006,
            bevel_segments = 1
        );
        fextrude!(
            F,
            "rim_flange",
            &[(0.06, 0.0), (-0.06, 0.0), (-0.03, 0.12), (0.03, 0.12)],
            0.006,
            (0, RIM.0 - 0.008, 0.06),
            M.rim,
            plane = Facing::Top
        );
        torus!(
            "rim",
            0.22,
            0.012,
            at = F.at(0, RIM.0, RIM.1),
            material = M.rim,
            major_segments = 28,
            minor_segments = 6
        );
        for k in 0..12 {
            let a = TAU * k as f64 / 12.0;
            let (x, z) = (0.215 * cos(a), RIM.1 + 0.215 * sin(a));
            fsweep!(
                F,
                format!("net_hook{k}"),
                &[
                    (x, RIM.0 - 0.005, z),
                    (x * 0.97, RIM.0 - 0.022, z - 0.0),
                    (x * 0.95, RIM.0 - 0.014, z)
                ],
                0.0025,
                M.rim,
                verts = 3
            );
        }
    });
    join(&still, "hoop", Some(P3::from((0, 0, 0))));
    // the ball, at rest on the ground in front: its seams. A part of its own, gone while someone plays (it is in
    // their hands: actions.ts `PROPS.basketball`).
    let ball_parts = new_parts(|| {
        fsphere!(
            F,
            "ball",
            0.12,
            (0.3, 0.12, 0.32),
            M.ball,
            segments = 16,
            rings = 10
        );
        for (i, rot) in [(90, 0, 30), (0, 0, 0), (90, 0, 120)]
            .into_iter()
            .enumerate()
        {
            torus!(
                format!("seam{i}"),
                0.1205,
                0.0035,
                at = F.at(0.3, 0.12, 0.32),
                material = M.rubber,
                rot = rot,
                major_segments = 20,
                minor_segments = 3
            );
        }
    });
    let ball = hinged(join(&ball_parts, "ball", None), F.at(0.3, 0.12, 0.32));
    let net = hinged(net("net"), F.at(0, RIM.0, RIM.1));
    settle!(centre = false);
    rock(
        net,
        "swish",
        9.0,
        Rock {
            seconds: 0.6,
            axis: Axis::X,
        },
    );
    squash(
        net,
        "swish",
        (0.92, 0.92, 1.18),
        Squash {
            seconds: 0.6,
            back: true,
            ..Default::default()
        },
    );
    squash(
        ball,
        "ball_taken",
        0.001,
        Squash {
            seconds: 0.2,
            ..Default::default()
        },
    );
    meshes()
}

// A weatherproof table at the ITTF's size (2.74 x 1.525 m, its top 0.76 m up) on 3 x 2 tiles, the players at its two
// ends (content `places`): a green top on two concrete legs, a steel net 15.25 cm high reaching 15.25 cm past each
// side, and a ball. Still. The bats are in the players' hands (held `tool.bat`; review I-community-06: they lay on
// the top while it was played). (It was 1.9 x 0.94 m, its top at 0.78.)
const PONG: (f64, f64, f64) = (2.74, 0.76, 1.525); // the top's length, height and width

pub fn ping_pong_table() -> Vec<Obj> {
    let (length, top, width) = PONG;
    let (hl, hw) = (length / 2.0, width / 2.0);
    // the top: a slab with a band round its edge, a steel frame under it
    fbox!(
        F,
        "top",
        length,
        0.03,
        width,
        (0, top - 0.015, 0),
        M.table_top,
        bevel = 0.006,
        bevel_segments = 1
    );
    kit::moulding!(
        F,
        "top_band",
        &[(hl, -hw), (hl, hw), (-hl, hw), (-hl, -hw)],
        &[
            (0.0, -0.035),
            (0.0, -0.002),
            (0.006, -0.002),
            (0.006, -0.035),
        ],
        M.steel,
        at = top,
        plane = Facing::Top
    );
    for z in [-hw + 0.1, hw - 0.1] {
        fbox!(
            F,
            format!("apron{}", float_repr(z)),
            length - 0.3,
            0.06,
            0.04,
            (0, top - 0.06, z),
            M.steel,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    // two concrete legs: a tapered pier with an arch through it, on a broad foot
    for x in [-0.85, 0.85] {
        let leg = [
            (-0.5, 0.04),
            (0.5, 0.04),
            (0.42, top - 0.03),
            (-0.42, top - 0.03),
        ];
        let arch: Vec<(f64, f64)> = (0..11)
            .map(|k| {
                (
                    0.18 * cos(PI * k as f64 / 10.0),
                    0.04 + 0.32 * sin(PI * k as f64 / 10.0),
                )
            })
            .collect();
        let mut hole = vec![(0.18, 0.12)];
        hole.extend(
            arch[1..arch.len() - 1]
                .iter()
                .map(|&(px, py)| (px, py + 0.08)),
        );
        hole.push((-0.18, 0.12));
        fextrude!(
            F,
            format!("leg{}", float_repr(x)),
            &leg,
            0.12,
            (x, 0, 0),
            M.concrete,
            plane = Facing::Side,
            holes = [hole],
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("foot{}", float_repr(x)),
            0.22,
            0.04,
            1.12,
            (x, 0.02, 0),
            M.concrete,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("cap{}", float_repr(x)),
            0.16,
            0.02,
            0.88,
            (x, top - 0.04, 0),
            M.steel,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    let y = top + 0.001;
    for (w, d, x, z) in [
        (length, 0.02, N::I(0), N::F(hw - 0.01)),
        (length, 0.02, N::I(0), N::F(-hw + 0.01)),
        (0.02, width, N::F(hl - 0.01), N::I(0)),
        (0.02, width, N::F(-hl + 0.01), N::I(0)),
        (length, 0.003, N::I(0), N::I(0)),
    ] {
        fbox!(F, format!("line{x}{z}"), w, 0.003, d, (x, y, z), M.lines);
    }
    // the steel net: a perforated plate (rows of holes as a darker band) between two posts clamped to the top
    let net = width + 2.0 * 0.1525;
    fbox!(
        F,
        "net",
        0.012,
        0.13,
        net - 0.04,
        (0, top + 0.07, 0),
        M.steel
    );
    for k in 0..3 {
        fbox!(
            F,
            format!("net_slot{k}"),
            0.014,
            0.012,
            net - 0.08,
            (0, top + 0.03 + k as f64 * 0.035, 0),
            M.rubber
        );
    }
    fbox!(
        F,
        "net_band",
        0.026,
        0.02,
        net,
        (0, top + 0.1425, 0),
        M.lines,
        bevel = 0.004,
        bevel_segments = 1
    );
    for z in [-net / 2.0 + 0.015, net / 2.0 - 0.015] {
        fbox!(
            F,
            format!("net_post{}", float_repr(z)),
            0.04,
            0.17,
            0.03,
            (0, top + 0.08, z),
            M.steel,
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("net_clamp{}", float_repr(z)),
            0.06,
            0.05,
            0.05,
            (0, top - 0.015, (hw - 0.005).copysign(z)),
            M.steel,
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("net_arm{}", float_repr(z)),
            0.03,
            0.02,
            z.abs() - hw + 0.03,
            (0, top + 0.005, ((z.abs() + hw) / 2.0).copysign(z)),
            M.steel
        );
    }
    // the ball
    fsphere!(
        F,
        "ball",
        0.02,
        (0.5, top + 0.02, -0.15),
        M.ping,
        segments = 10,
        rings = 6
    );
    join_all("pingPongTable");
    settle!(centre = false);
    meshes()
}

// A stone table with the board inlaid, its top 0.7 m square, and a stone stool either side of it, left and right,
// where the content's two places seat the players facing each other across the board (x = ±0.6 m, 0.47 m up, on
// 2 x 1 tiles: their knees clear of each other under the top; they were ±0.42 on one tile). White plays from the
// left. A white pawn slides one square on (`move`) as they play.
const BOARD: (f64, f64) = (0.762, 0.0); // (height, depth) of the board's middle
const SQUARE: f64 = 0.05;
const STOOL: f64 = 0.6; // the stools' middles, either side (the content's `places`)
const CHESS_TOP: f64 = 0.7; // the top, square

/// The game (x, depth) of square (file i, rank j), rank 0 nearest white on the left, file 0 on white's
/// left (to the front).
fn square(i: i32, j: i32) -> (f64, f64) {
    (
        -3.5 * SQUARE + j as f64 * SQUARE,
        BOARD.1 + 3.5 * SQUARE - i as f64 * SQUARE,
    )
}

/// The two stools: a turned stone mushroom, a moulded foot, a waisted stem, a seat with a rounded edge.
fn stools(segments: usize) {
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

fn pedestal(segments: usize) {
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
fn piece(name: &str, kind: &str, i: i32, j: i32, material: Mat, low: bool) {
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

const WHITE: [(&str, i32, i32); 8] = [
    ("king", 4, 0),
    ("rook", 0, 0),
    ("pawn", 1, 1),
    ("pawn", 5, 1),
    ("pawn", 6, 2),
    ("queen", 2, 3),
    ("knight", 5, 2),
    ("bishop", 3, 2),
];
const BLACK: [(&str, i32, i32); 8] = [
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
fn top() {
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
