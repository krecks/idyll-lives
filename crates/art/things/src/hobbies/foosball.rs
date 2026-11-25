//! H2: the foosball table: a birch cabinet on legs, a pitch with its lines and goals, and the eight rods of a real
//! table, 15 cm apart (review M-hobbies-07: it had four, every handle on the front): from the left goal, blue's goalie
//! and defence, red's attack, blue's five midfielders, red's, blue's attack, red's defence and goalie, each team's
//! handles on its own side: blue's to the front, where the players stand (characters.ts `STEP_IN` lanes: one at the
//! goalie and defence, `FOOS_LANES` in clips/games.ts; one at the midfield and attack), red's to the back, short there so
//! the table can stand against a wall. One rod in each blue player's hands is a part of its own and kicks back and
//! forth while played (`rods`, a loop); the others stay still.

use super::M;
use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::kit::{self};
use artkit::math::{Axis, P3, cos, radians, sin};
use artkit::motion::{Rock, rock};
use artkit::parts::{Settle, hinged, join, new_parts, settle_with};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, revolve};

const ROD_Y: f64 = 0.84;

fn foos_rods() -> Vec<f64> {
    (0..8).map(|k| -0.525 + 0.15 * k as f64).collect()
}

/// Per rod: blue (front handles) or red, and its men (depth across the pitch).
const FOOS_TEAMS: [(bool, &[f64]); 8] = [
    (true, &[0.0]),
    (true, &[-0.12, 0.12]),
    (false, &[-0.18, 0.0, 0.18]),
    (true, &[-0.24, -0.12, 0.0, 0.12, 0.24]),
    (false, &[-0.24, -0.12, 0.0, 0.12, 0.24]),
    (true, &[-0.18, 0.0, 0.18]),
    (false, &[-0.12, 0.12]),
    (false, &[0.0]),
];
/// The rods that kick: blue's defence and attack.
const FOOS_KICKING: [usize; 2] = [1, 5];

/// A foosball man's outline (game x across the pitch, height from the rod), as seen from the side: a round head,
/// shoulders, the rod through the chest, legs to two flat kicking feet.
fn foos_man() -> Vec<(f64, f64)> {
    let head = (-55..=235).step_by(58).map(|a| {
        (
            0.017 * cos(radians(a as f64)),
            0.054 + 0.017 * sin(radians(a as f64)),
        )
    });
    let right = [
        (0.021, -0.093),
        (0.021, -0.085),
        (0.011, -0.078),
        (0.011, -0.02),
        (0.017, -0.008),
        (0.019, 0.028),
        (0.011, 0.038),
    ];
    let mut out = right.to_vec();
    out.extend(head);
    out.extend(right.iter().rev().map(|&(x, y)| (-x, y)));
    out
}

fn rod_men(fr: &GameFrame, name: &str, x: f64, front: bool, men: &[f64]) {
    let team = if front { M.ball_blue } else { M.ball_red };
    let grip = [
        (0.0, 0.0),
        (0.017, 0.0),
        (0.02, 0.012),
        (0.018, 0.03),
        (0.021, 0.05),
        (0.018, 0.07),
        (0.02, 0.085),
        (0.012, 0.09),
        (0.0, 0.09),
    ];
    if front {
        // the handle to the front, the bar's back end on the back wall's outside
        kit::tube!(
            fr,
            format!("{name}_bar"),
            &[(x, ROD_Y, -0.375), (x, ROD_Y, 0.415)],
            0.008,
            M.chrome,
            verts = 8,
            round = 1
        );
        revolve!(
            format!("{name}_handle"),
            &grip,
            at = fr.at(x, ROD_Y, 0.415),
            material = M.grip,
            segments = 8,
            rot = (90, 0, 0)
        );
    } else {
        // the handle to the back, short, inside the wall's face
        kit::tube!(
            fr,
            format!("{name}_bar"),
            &[(x, ROD_Y, -0.378), (x, ROD_Y, 0.375)],
            0.008,
            M.chrome,
            verts = 8,
            round = 1
        );
        let mut short = grip[..3].to_vec();
        short.extend([(0.02, 0.04), (0.012, 0.05), (0.0, 0.05)]);
        revolve!(
            format!("{name}_handle"),
            &short,
            at = fr.at(x, ROD_Y, -0.378),
            material = M.grip,
            segments = 8,
            rot = (-90, 0, 0)
        );
    }
    for z in [-0.37, 0.37] {
        fcyl!(
            fr,
            format!("{name}_bumper{}", r(z)),
            0.016,
            0.016,
            (x, ROD_Y, z),
            M.grip,
            rot = (90, 0, 0),
            verts = 6
        );
    }
    for &z in men {
        let outline: Vec<(f64, f64)> = foos_man()
            .iter()
            .map(|&(u, v)| (x + u, ROD_Y + v))
            .collect();
        let man = fextrude!(
            fr,
            format!("{name}_man{}", r(z)),
            &outline,
            0.03,
            (0, 0, z),
            team
        );
        // its face's edges chamfered, as a moulded man's are (only those: the turns round its outline stay)
        bevel_edges(man, 0.003, 1, 80.0, &EdgeOrder::Corners);
    }
}

pub fn foosball() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let rods_x = foos_rods();
    let still = new_parts(|| {
        fbox!(f, "pitch", 1.32, 0.02, 0.64, (0, 0.71, 0), M.field);
        fbox!(
            f,
            "box_floor",
            1.4,
            0.04,
            0.72,
            (0, 0.68, 0),
            M.foos,
            bevel = 0.01
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                f,
                format!("side{s}"),
                1.4,
                0.24,
                0.04,
                (0, 0.8, sf * 0.34),
                M.foos,
                bevel = 0.01
            );
            fbox!(
                f,
                format!("end{s}"),
                0.04,
                0.24,
                0.64,
                (sf * 0.68, 0.8, 0),
                M.foos,
                bevel = 0.01
            );
            // a framed panel on each long side, bearings where the rods come through
            kit::shaker!(
                f,
                format!("side_panel{s}"),
                1.24,
                0.19,
                (0, 0.795, sf * 0.365),
                0.012,
                M.foos,
                rail = 0.035,
                panel = M.foos_trim
            );
            for &x in &rods_x {
                fcyl!(
                    f,
                    format!("bearing{s}{}", r(x)),
                    0.017,
                    0.012,
                    (x, ROD_Y, sf * 0.374),
                    M.chrome,
                    rot = (90, 0, 0),
                    verts = 8
                );
            }
            // the goal mouths, the ball return's slot in each end
            fbox!(
                f,
                format!("goal{s}"),
                0.006,
                0.07,
                0.18,
                (sf * 0.658, 0.755, 0),
                M.grip
            );
            fbox!(
                f,
                format!("return{s}"),
                0.004,
                0.05,
                0.09,
                (sf * 0.7005, 0.75, 0.18),
                M.grip,
                bevel = 0.0015,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("box_line{s}"),
                0.12,
                0.0015,
                0.004,
                (sf * 0.6, 0.7205, 0.14),
                M.line
            );
            fbox!(
                f,
                format!("box_line_b{s}"),
                0.12,
                0.0015,
                0.004,
                (sf * 0.6, 0.7205, -0.14),
                M.line
            );
            fbox!(
                f,
                format!("box_front{s}"),
                0.004,
                0.0015,
                0.28,
                (sf * 0.54, 0.7205, 0),
                M.line
            );
            for sz in [-1, 1] {
                let szf = sz as f64;
                // square legs tapering to levelling feet, a cap on each corner post
                fsweep!(
                    f,
                    format!("leg{s}{sz}"),
                    &[(sf * 0.6, 0.68, szf * 0.3), (sf * 0.6, 0.03, szf * 0.3)],
                    0,
                    M.foos_trim,
                    profile = rounded_rect(0.08, 0.08, 0.01, 1),
                    up = (1, 0, 0),
                    scale = [1.0, 0.8]
                );
                fcyl!(
                    f,
                    format!("leg_foot{s}{sz}"),
                    0.03,
                    0.03,
                    (sf * 0.6, 0.015, szf * 0.3),
                    M.grip,
                    verts = 8
                );
                fbox!(
                    f,
                    format!("corner{s}{sz}"),
                    0.06,
                    0.02,
                    0.06,
                    (sf * 0.68, 0.93, szf * 0.34),
                    M.foos_trim,
                    bevel = 0.006,
                    bevel_segments = 1
                );
            }
            fbox!(
                f,
                format!("stretcher{s}"),
                0.04,
                0.04,
                0.52,
                (sf * 0.6, 0.12, 0),
                M.foos_trim
            );
            // a score rail of sliding beads at each end
            kit::tube!(
                f,
                format!("score{s}"),
                &[(sf * 0.62, 0.935, -0.12), (sf * 0.62, 0.935, 0.12)],
                0.003,
                M.chrome,
                verts = 6,
                round = 1
            );
            for k in 0..5 {
                let z = -0.1 + 0.025 * k as f64 + if k > 1 { 0.06 } else { 0.0 };
                fcyl!(
                    f,
                    format!("bead{s}{k}"),
                    0.011,
                    0.016,
                    (sf * 0.62, 0.935, z),
                    M.ball_white,
                    rot = (90, 0, 0),
                    verts = 8
                );
            }
        }
        fbox!(
            f,
            "stretcher_long",
            1.16,
            0.04,
            0.04,
            (0, 0.12, 0),
            M.foos_trim
        );
        fbox!(f, "half_line", 0.004, 0.0015, 0.62, (0, 0.7205, 0), M.line);
        revolve!(
            "centre_circle",
            &[(0.077, 0.0), (0.083, 0.0), (0.083, 0.0015), (0.077, 0.0015)],
            at = f.at(0, 0.72, 0),
            material = M.line,
            segments = 24
        );
        fsphere!(
            f,
            "ball",
            0.018,
            (0.05, 0.738, 0.05),
            M.ball_white,
            segments = 10,
            rings = 6
        );
        for (k, (&x, &(front, men))) in rods_x.iter().zip(FOOS_TEAMS.iter()).enumerate() {
            if !FOOS_KICKING.contains(&k) {
                rod_men(&f, &format!("rod{k}"), x, front, men);
            }
        }
    });
    join(&still, "body", Some(P3::ZERO));
    let mut rods = Vec::new();
    for k in FOOS_KICKING {
        let (x, (front, men)) = (rods_x[k], FOOS_TEAMS[k]);
        let p = new_parts(|| rod_men(&f, &format!("rod{k}"), x, front, men));
        rods.push(hinged(
            join(&p, &format!("rod{k}"), None),
            f.at(x, ROD_Y, 0),
        ));
    }
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    for rod in rods {
        rock(
            rod,
            "rods",
            55.0,
            Rock {
                seconds: 0.8,
                axis: Axis::Y,
            },
        );
    }
    meshes()
}
