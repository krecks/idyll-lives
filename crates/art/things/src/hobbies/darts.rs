//! H2: the dartboard: a board on the wall in a wooden cabinet, its twenty segments and rings one flat face. Three
//! darts, one part, grow out of the board while a resident plays (`darts`). It hangs at its real height.

use super::M;
use super::common::link;
use artkit::kit;

use artkit::math::{Axis, P3, V3, cosd, sind};
use artkit::motion::{Grow, grow};
use artkit::parts::{Settle, join, new_parts, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, revolve, torus};

/// The bull's height.
const BOARD_C: f64 = 1.725;
/// The board's face, game depth.
const FACE_Z: f64 = -0.367;

fn dart_face(name: &str) -> Obj {
    let mats = [M.board, M.board_light, M.ring_red, M.ring_green];
    let bands = [
        (0.0159, 0.099, 0),
        (0.099, 0.107, 2),
        (0.107, 0.162, 0),
        (0.162, 0.170, 2),
    ];
    let y = -FACE_Z;
    let mut verts: Vec<V3> = Vec::new();
    let mut faces: Vec<(Vec<u32>, u16)> = Vec::new();
    let mut pt = |r: f64, a: f64| {
        verts.push(V3::from((r * cosd(a), y, BOARD_C + r * sind(a))));
        verts.len() as u32 - 1
    };
    for s in 0..20 {
        for sub in 0..2 {
            let a0 = (81 + 18 * s + 9 * sub) as f64;
            for (r0, r1, k) in bands {
                let face = vec![pt(r0, a0), pt(r1, a0), pt(r1, a0 + 9.0), pt(r0, a0 + 9.0)];
                faces.push((face, (k + s % 2) as u16));
            }
        }
    }
    for k in 0..24 {
        let (a0, a1) = ((15 * k) as f64, (15 * k + 15) as f64);
        let face = vec![
            pt(0.00635, a0),
            pt(0.0159, a0),
            pt(0.0159, a1),
            pt(0.00635, a1),
        ];
        faces.push((face, 3));
        let face = vec![pt(0.0, a0), pt(0.00635, a0), pt(0.00635, a1)];
        faces.push((face, 2));
    }
    link(name, verts, &faces, &mats)
}

pub fn dartboard() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            f,
            "back",
            0.6,
            0.6,
            0.02,
            (0, BOARD_C, -0.418),
            M.cabinet,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the cabinet's frame: one moulding round the board, mitred, rounded over at the front
        let frame = [
            (0.31, BOARD_C + 0.31),
            (-0.31, BOARD_C + 0.31),
            (-0.31, BOARD_C - 0.31),
            (0.31, BOARD_C - 0.31),
        ];
        kit::moulding!(
            f,
            "frame",
            &frame,
            &[
                (0.0, 0.0),
                (0.0, 0.042),
                (0.006, 0.05),
                (0.022, 0.05),
                (0.03, 0.044),
                (0.034, 0.03),
                (0.034, 0.0)
            ],
            M.cabinet,
            at = -0.408
        );
        // the board: sisal, its edge rounded, on the back, its front 1 mm behind the printed face (no z-fighting)
        revolve!(
            "board",
            &[
                (0.0, 0.0),
                (0.225, 0.0),
                (0.225, 0.03),
                (0.222, 0.036),
                (0.214, 0.04),
                (0.0, 0.04)
            ],
            at = f.at(0, BOARD_C, -0.408),
            material = M.board,
            segments = 40,
            rot = (90, 0, 0)
        );
        dart_face("face");
        for s in 0..20 {
            let a = (90 + 18 * s) as f64;
            fbox!(
                f,
                format!("number{s}"),
                0.012,
                0.016,
                0.002,
                (-0.197 * cosd(a), BOARD_C + 0.197 * sind(a), FACE_Z),
                M.board_light,
                rot = (0.0, 90.0 - a, 0.0)
            );
        }
        // the spider's outer wire (the radial and inner wires shade the flat face's corners dark: README "Shading by
        // corners")
        torus!(
            "wire",
            0.17,
            0.0011,
            at = f.at(0, BOARD_C, FACE_Z + 0.0008),
            material = M.steel,
            rot = (90, 0, 0),
            major_segments = 40,
            minor_segments = 3
        );
        torus!(
            "number_ring",
            0.212,
            0.0016,
            at = f.at(0, BOARD_C, FACE_Z),
            material = M.steel,
            rot = (90, 0, 0),
            major_segments = 40,
            minor_segments = 3
        );
        // a chalk ledge under the board, chalk on it
        fbox!(
            f,
            "ledge",
            0.5,
            0.012,
            0.04,
            (0, BOARD_C - 0.27, -0.388),
            M.cabinet,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "chalk",
            0.06,
            0.012,
            0.012,
            (0.15, BOARD_C - 0.258, -0.39),
            M.board_light,
            bevel = 0.003,
            bevel_segments = 1
        );
    });
    join(&still, "body", Some(P3::ZERO));
    let d = new_parts(|| {
        let kite = [
            (0.0, 0.003),
            (0.018, 0.015),
            (0.042, 0.017),
            (0.042, -0.017),
            (0.018, -0.015),
            (0.0, -0.003),
        ];
        for (k, (x, by, flight)) in [
            (0.02, BOARD_C + 0.01, M.ball_red),
            (-0.07, BOARD_C + 0.08, M.ball_blue),
            (0.09, BOARD_C - 0.06, M.ball_yellow),
        ]
        .into_iter()
        .enumerate()
        {
            kit::tube!(
                f,
                format!("point{k}"),
                &[(x, by, FACE_Z), (x, by, FACE_Z + 0.02)],
                0.0013,
                M.steel,
                verts = 6,
                round = 1
            );
            revolve!(
                format!("barrel{k}"),
                &[
                    (0.0, 0.0),
                    (0.0025, 0.0),
                    (0.0042, 0.006),
                    (0.0048, 0.02),
                    (0.0046, 0.036),
                    (0.0032, 0.045),
                    (0.0, 0.045)
                ],
                at = f.at(x, by, FACE_Z + 0.02),
                material = M.steel,
                segments = 8,
                rot = (90, 0, 0)
            );
            kit::tube!(
                f,
                format!("shaft{k}"),
                &[(x, by, FACE_Z + 0.065), (x, by, FACE_Z + 0.1)],
                0.0022,
                flight,
                verts = 6,
                round = 1
            );
            fextrude!(
                f,
                format!("flight{k}a"),
                &kite,
                0.0015,
                (x, by, FACE_Z + 0.083),
                flight,
                plane = Facing::Side
            );
            let turned: Vec<(f64, f64)> = kite.iter().map(|&(u, v)| (v, u)).collect();
            fextrude!(
                f,
                format!("flight{k}b"),
                &turned,
                0.0015,
                (x, by, FACE_Z + 0.083),
                flight,
                plane = Facing::Top
            );
        }
    });
    // Hidden at rest by flattening into the board itself (on its face, the flights would show as crosses).
    let darts = join(&d, "darts", Some(f.at(0, BOARD_C, FACE_Z - 0.02)));
    darts.edit(|o| o.scale.y = 0.001);
    // on the wall, at its real height
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: false,
        },
    );
    grow(
        darts,
        "darts",
        Grow {
            seconds: 0.6,
            axis: Axis::Y,
        },
    );
    meshes()
}
