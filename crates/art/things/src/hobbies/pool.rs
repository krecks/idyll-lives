//! H2: the pool table: a lacquered frame on four legs, a baize bed with cushions, six pockets and sights, the fifteen
//! balls racked at the foot and the cue ball at the head. The triangle round the balls is lifted off, carried over the
//! foot rail and hung on the foot end for pool and trick shots (`rack_off`, a path); at the first shot the racked balls
//! go (`racked_off`, squashed away) and the balls spread over the table come up through the cloth (`spread`, grown from
//! inside the slate, where they lie flat at rest), as the break leaves them (review I-hobbies-05). The cue is in the
//! players' hands (held `tool.cue`, one each), not on the cloth.

use super::M;
use super::common::P;
use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::kit::{self, fillet};
use artkit::math::{P3, TAU, cos, exp, pow, sqrt};
use artkit::motion::{Grow, Squash, grow, path, squash};
use artkit::parts::{Settle, hinged, join, new_parts, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::shapes::SHARP;
use artkit::{fbox, fextrude, fsweep, sphere};

const BALL: f64 = 0.0285;
/// The cloth's top: the rails' top 3.5 cm over it, at 0.785 m (a table is 0.74–0.79 to the rail; review M-hobbies-06:
/// the cloth was at 0.785, the rails at 0.82).
const BED: f64 = 0.75;
const RAIL_TOP: f64 = BED + 0.035;
/// The rack's middle, hung on the foot end (game x, height): its apex at the rail's top.
const RACK_HUNG: (f64, f64) = (1.275, BED - 0.185);
/// Where the fifteen balls lie after the break (game x, depth; the cloth runs to x ±1.13, depth ±0.58): most of them
/// down the foot half, where the rack was, a few run to the head end and the cushions.
const SPREAD: [(f64, f64); 15] = [
    (0.42, 0.05),
    (0.78, -0.31),
    (0.95, 0.22),
    (0.31, -0.4),
    (0.62, 0.44),
    (1.02, -0.05),
    (-0.18, 0.21),
    (0.55, -0.08),
    (0.88, 0.47),
    (-0.45, -0.38),
    (0.15, 0.36),
    (0.71, 0.13),
    (1.05, -0.45),
    (-0.82, 0.12),
    (0.36, -0.18),
];
/// The first shot (s into a game): the rack lifted off and hung up (1.4 s) and the player bent over the cue, their
/// stroke through (clips/games.ts `STROKE`, 3.45 s into the 8 s round).
pub const BREAK: f64 = 4.6;

pub fn pool_table() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let colours = [
        M.ball_yellow,
        M.ball_blue,
        M.ball_red,
        M.ball_purple,
        M.ball_black,
        M.ball_orange,
        M.ball_green,
        M.ball_maroon,
        M.ball_yellow,
        M.ball_blue,
        M.ball_red,
        M.ball_purple,
        M.ball_orange,
        M.ball_green,
        M.ball_maroon,
    ];
    let (apex, row) = (0.5, sqrt(3.0) * BALL);
    let still = new_parts(|| {
        fbox!(f, "bed", 2.3, 0.02, 1.2, (0, BED - 0.01, 0), M.cloth);
        fbox!(f, "slate", 2.3, 0.08, 1.2, (0, BED - 0.06, 0), M.pool_frame);
        fbox!(
            f,
            "apron",
            2.4,
            0.18,
            1.3,
            (0, BED - 0.185, 0),
            M.pool_frame,
            bevel = 0.02
        );
        // beads round the apron's foot and under the rails, mitred at the corners
        let box_ = [(1.2, 0.65), (-1.2, 0.65), (-1.2, -0.65), (1.2, -0.65)];
        kit::moulding!(
            f,
            "apron_bead",
            &box_,
            &[
                (0.0, 0.0),
                (-0.012, 0.004),
                (-0.015, 0.012),
                (-0.01, 0.02),
                (0.0, 0.022)
            ],
            M.pool_frame,
            at = BED - 0.275,
            plane = Facing::Top
        );
        // a shaped skirt under the long sides' aprons, dropping to a lobe in the middle
        let edge: Vec<(f64, f64)> = (0..25)
            .map(|i| -0.95 + 1.9 * i as f64 / 24.0)
            .map(|x| {
                (
                    x,
                    BED - 0.29 - 0.012 * cos(TAU * x / 0.475) - 0.035 * exp(-pow(x / 0.12, 2.0)),
                )
            })
            .collect();
        for s in [-1, 1] {
            let mut outline = vec![(0.95, BED - 0.2), (-0.95, BED - 0.2)];
            outline.extend_from_slice(&edge);
            fextrude!(
                f,
                format!("skirt{s}"),
                &outline,
                0.02,
                (0, 0, s as f64 * 0.655),
                M.pool_frame,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // the rails: one moulded frame round the bed, rounded over at the top outside edge
        let rails = [(1.25, 0.7), (-1.25, 0.7), (-1.25, -0.7), (1.25, -0.7)];
        let rails = kit::moulding!(
            f,
            "rails",
            &rails,
            &[
                (0.0, 0.0),
                (0.0, 0.118),
                (0.006, 0.136),
                (0.02, 0.15),
                (0.1, 0.15),
                (0.1, 0.0)
            ],
            M.pool_frame,
            at = RAIL_TOP - 0.15,
            plane = Facing::Top
        );
        // its mitred corners' edges eased
        bevel_edges(rails, 0.004, 1, SHARP, &EdgeOrder::Corners);
        // the cushions: cloth noses along the rails' inside, between the pockets
        // (its rubber nose rounded)
        let nose = crate::common::round_at(
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.038, 0.012),
                (0.026, 0.032),
                (0.012, 0.03),
            ],
            3,
            0.006,
        );
        for s in [-1, 1] {
            let sf = s as f64;
            for (x0, x1) in [(-1.09, -0.07), (0.07, 1.09)] {
                let prof: Vec<(f64, f64)> = nose.iter().map(|&(u, v)| (u, v * sf)).collect();
                fsweep!(
                    f,
                    format!("cushion_long{s}{}", r(x0)),
                    &[(x0, BED, sf * 0.6), (x1, BED, sf * 0.6)],
                    0,
                    M.cloth,
                    profile = prof
                );
            }
            let prof: Vec<(f64, f64)> = nose.iter().map(|&(u, v)| (u, -v * sf)).collect();
            fsweep!(
                f,
                format!("cushion_short{s}"),
                &[(sf * 1.15, BED, -0.54), (sf * 1.15, BED, 0.54)],
                0,
                M.cloth,
                profile = prof
            );
            // the pockets: leather mouths cut into the rails, dark inside
            for x in [-1.15, 0.0, 1.15] {
                let z = sf * if x != 0.0 { 0.6 } else { 0.625 };
                let name = if x == 0.0 {
                    format!("pocket{s}0")
                } else {
                    format!("pocket{s}{}", r(x))
                };
                kit::turned!(
                    f,
                    &name,
                    x,
                    RAIL_TOP - 0.002,
                    z,
                    // (the leather rolled round its lip)
                    &crate::common::round_at(
                        &[
                            (0.0, 0.0),
                            (0.046, 0.0),
                            (0.058, 0.0),
                            (0.064, 0.003),
                            (0.06, 0.006),
                            (0.048, 0.005),
                            (0.0, 0.003),
                        ],
                        3,
                        0.0015
                    ),
                    M.pocket,
                    segments = 12
                );
            }
            // the sights: mother-of-pearl diamonds on the rails' tops
            for k in 1..4 {
                for half in [-1, 1] {
                    fbox!(
                        f,
                        format!("sight{s}{half}{k}"),
                        0.014,
                        0.002,
                        0.014,
                        (
                            half as f64 * 0.2875 * k as f64,
                            RAIL_TOP + 0.0005,
                            sf * 0.65
                        ),
                        M.pearl,
                        rot = (0, 0, 45)
                    );
                }
            }
            for z in [-0.3, 0.0, 0.3] {
                fbox!(
                    f,
                    format!("sight_end{s}{}", r(z)),
                    0.014,
                    0.002,
                    0.014,
                    (sf * 1.2, RAIL_TOP + 0.0005, z),
                    M.pearl,
                    rot = (0, 0, 45)
                );
            }
            // turned legs, heavy, on square feet
            for sz in [-1, 1] {
                kit::turned!(
                    f,
                    format!("leg{s}{sz}"),
                    sf * 1.08,
                    0.02,
                    sz as f64 * 0.5,
                    &[
                        (0.0, 0.0),
                        (0.085, 0.0),
                        (0.08, 0.02),
                        (0.06, 0.05),
                        (0.055, 0.1),
                        (0.075, 0.18),
                        (0.085, 0.23),
                        (0.075, 0.28),
                        (0.055, 0.34),
                        (0.06, 0.38),
                        (0.075, 0.4),
                        (0.075, BED - 0.265),
                        (0.0, BED - 0.265),
                    ],
                    M.pool_frame,
                    segments = 10
                );
                fbox!(
                    f,
                    format!("foot{s}{sz}"),
                    0.19,
                    0.02,
                    0.19,
                    (sf * 1.08, 0.01, sz as f64 * 0.5),
                    M.pool_frame,
                    bevel = 0.006,
                    bevel_segments = 1
                );
            }
        }
        fbox!(
            f,
            "chalk",
            0.022,
            0.022,
            0.022,
            (0.9, RAIL_TOP + 0.011, 0.65),
            M.ball_blue,
            bevel = 0.002,
            bevel_segments = 1
        );
        // the cue ball, at the head
        sphere!(
            "cue_ball",
            BALL,
            at = f.at(-0.6, BED + BALL, 0),
            material = M.ball_white,
            segments = 10,
            rings = 7
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // the racked balls, the apex to the head: squashed away to the rack's middle at the break
    let centre = apex + 4.0 * row * 2.0 / 3.0;
    let rb = new_parts(|| {
        let mut k = 0;
        for i in 0..5 {
            for j in 0..=i {
                sphere!(
                    format!("ball{k}"),
                    BALL,
                    at = f.at(
                        apex + i as f64 * row,
                        BED + BALL,
                        (j as f64 - i as f64 / 2.0) * 2.0 * BALL
                    ),
                    material = colours[k],
                    segments = 10,
                    rings = 7
                );
                k += 1;
            }
        }
    });
    let racked = join(&rb, "racked", Some(f.at(centre, BED + BALL, 0)));
    // the balls after the break, spread over the cloth (none in a pocket's mouth), flat inside the slate at rest
    let sb = new_parts(|| {
        for (k, &(x, z)) in SPREAD.iter().enumerate() {
            sphere!(
                format!("spread{k}"),
                BALL,
                at = f.at(x, BED + BALL, z),
                material = colours[k],
                segments = 10,
                rings = 7
            );
        }
    });
    let spread = join(&sb, "spread", Some(f.at(0, BED - 0.04, 0)));
    spread.edit(|o| o.scale.z = 0.001);
    // the triangle: a frame with rounded corners round the rack
    let corners = [
        (apex, 0.0),
        (apex + 4.0 * row, -4.0 * BALL),
        (apex + 4.0 * row, 4.0 * BALL),
    ];
    let t = 0.016;
    // the centres' triangle's inradius
    let inr = 4.0 * BALL / sqrt(3.0);
    let grow_by = (inr + BALL + t / 2.0) / inr;
    let pts: Vec<(f64, f64)> = corners
        .iter()
        .map(|&(x, z)| (centre + (x - centre) * grow_by, z * grow_by))
        .collect();
    let rp = new_parts(|| {
        let p3: Vec<P> = pts.iter().map(|&(x, z)| (x, BED + 0.015, z)).collect();
        let mid = (
            (p3[2].0 + p3[0].0) / 2.0,
            (p3[2].1 + p3[0].1) / 2.0,
            (p3[2].2 + p3[0].2) / 2.0,
        );
        let mut lp = fillet(&[mid, p3[0], p3[1], p3[2], mid], 0.03, 3);
        lp.pop();
        fsweep!(
            f,
            "rack",
            &lp,
            0,
            M.rack,
            profile = rounded_rect(0.03, t, 0.004, 1),
            closed = true
        );
    });
    let rack = hinged(join(&rp, "rack", None), f.at(centre, BED + 0.015, 0));
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    squash(
        racked,
        "racked_off",
        0.001,
        Squash {
            seconds: 0.15,
            ..Squash::default()
        },
    );
    grow(
        spread,
        "spread",
        Grow {
            seconds: 0.3,
            ..Grow::default()
        },
    );
    // Racked, the triangle is lifted off the balls, carried over the foot rail and hung up by its apex on the table's
    // foot end (its middle 2.5 cm past the rail, at the apron's height), out of the game's way.
    let hung = (RACK_HUNG.0 - centre, RACK_HUNG.1 - (BED + 0.015));
    path(
        rack,
        "rack_off",
        &[
            (0.0, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),
            (0.35, (0.0, 0.0, 0.14), (0.0, 0.0, 0.0)),
            (0.9, (-hung.0, 0.0, 0.14), (0.0, 0.0, 0.0)),
            (1.4, (-hung.0, 0.0, hung.1), (0.0, -90.0, 0.0)),
        ],
        false,
    );
    meshes()
}
