//! The bakery case: 2 x 1, a cream cabinet under a glass display, sitting back on its tiles as the game places it;
//! its back door slides aside.

use super::common::{M, arc, fluted};
use artkit::forms::by;
use artkit::math::{P3, PI, cos, radians, sin};
use artkit::motion::{Slide, slide};
use artkit::parts::{join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, kit_shaker, loft, revolve, settle, soft};

const DOOR_SECONDS: f64 = 1.0; // the bakery case's back door slides aside this long

// --- the bakery case: 2x1, a cream cabinet under a glass display, its back doors sliding ------------
pub fn bakery_case() -> Vec<Obj> {
    let f = GameFrame::new(0.04); // 4 cm forward: the worktop's back on the wall face (-0.43)
    let (z, front, deck) = (-0.04, 0.36, 0.78);
    let back = -0.43; // the display's back face, before the frame's 4 cm shift forward
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            1.86,
            0.06,
            0.76,
            (0, 0.03, z),
            M.rim,
            bevel = 0.004
        );
        fbox!(
            f,
            "cabinet",
            1.9,
            0.68,
            0.8,
            (0, 0.4, z),
            M.case,
            bevel = 0.01
        );
        fbox!(
            f,
            "deck",
            1.94,
            0.04,
            0.86,
            (0, 0.76, z),
            M.rim,
            bevel = 0.008
        );
        // the front: a band of colour over three raised panels
        fbox!(
            f,
            "band",
            1.9,
            0.08,
            0.016,
            (0, 0.62, front + 0.008),
            M.stripe,
            bevel = 0.003
        );
        for (i, x) in [-0.62, 0.0, 0.62].into_iter().enumerate() {
            kit_shaker!(
                f,
                format!("panel{i}"),
                0.56,
                0.4,
                (x, 0.33, front + 0.01),
                0.02,
                M.case,
                rail = 0.06,
                raised = true
            );
        }
        // the display: walnut ends and a glass front curving up and back to the glass top, rails, a mirror-back frame
        let front_arc = arc(0.38, deck, 0.2, 1.29, 8, 0.0);
        for s in [-1, 1] {
            let x = s as f64 * 0.935;
            let outline: Vec<(f64, f64)> = [(-0.45, deck)]
                .into_iter()
                .chain(front_arc.iter().copied())
                .chain([(-0.45, 1.29)])
                .collect();
            crate::eased(fextrude!(
                f,
                format!("end{s}"),
                &outline,
                0.03,
                (x, 0, 0),
                M.rim,
                plane = Facing::Side
            ));
        }
        let outer = arc(0.37, deck, 0.2, 1.265, 8, 0.0);
        let inner = arc(0.37, deck, 0.2, 1.265, 8, 0.01);
        let glass: Vec<(f64, f64)> = outer
            .iter()
            .copied()
            .chain(inner.iter().rev().copied())
            .collect();
        fextrude!(
            f,
            "glass_front",
            &glass,
            1.84,
            (0, 0, 0),
            M.glass,
            plane = Facing::Side
        );
        fbox!(
            f,
            "glass_top",
            1.84,
            0.01,
            0.62,
            (0, 1.265, -0.115),
            M.glass
        );
        fbox!(
            f,
            "rail_front",
            1.9,
            0.03,
            0.05,
            (0, 1.275, 0.18),
            M.rim,
            bevel = 0.004
        );
        fbox!(
            f,
            "rail_back",
            1.9,
            0.03,
            0.05,
            (0, 1.275, back),
            M.rim,
            bevel = 0.004
        );
        fbox!(
            f,
            "track",
            1.84,
            0.02,
            0.05,
            (0, deck + 0.01, back),
            M.rim,
            bevel = 0.003
        );
        fbox!(
            f,
            "sill",
            1.88,
            0.02,
            0.05,
            (0, deck + 0.01, 0.36),
            M.rim,
            bevel = 0.004
        );
        // the fixed back pane, in the rear track on the left
        fbox!(
            f,
            "glass_back",
            0.93,
            0.44,
            0.008,
            (-0.455, 1.02, back - 0.01),
            M.glass
        );
        // a glass shelf over the back row, on two brackets
        fbox!(
            f,
            "glass_shelf",
            1.84,
            0.01,
            0.28,
            (0, 1.03, -0.25),
            M.glass
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("bracket{s}"),
                0.02,
                0.025,
                0.28,
                (s as f64 * 0.905, 1.015, -0.25),
                M.rim
            );
        }
        // the bakes: a front row of buns, tarts and loaves on doilies with price cards, cakes and biscuits behind,
        // and croissants and cupcakes on the shelf
        for i in 0..6 {
            let x = -0.75 + i as f64 * 0.3;
            fcyl!(
                f,
                format!("doily{i}"),
                0.11,
                0.006,
                (x, deck + 0.003, 0.14),
                M.tray,
                verts = 14
            );
            fbox!(
                f,
                format!("tray{i}"),
                0.26,
                0.012,
                0.2,
                (x, deck + 0.006, -0.2),
                M.tray,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("card{i}"),
                0.06,
                0.04,
                0.004,
                (x + 0.09, deck + 0.03, 0.25),
                M.tray,
                rot = (-20, 0, 0)
            );
            fbox!(
                f,
                format!("card_price{i}"),
                0.04,
                0.008,
                0.002,
                (x + 0.09, deck + 0.03, 0.2525),
                M.choc,
                rot = (-20, 0, 0)
            );
            let k = i % 3;
            if k == 0 {
                for (j, (dx, dz)) in [(-0.045, -0.03), (0.045, -0.03), (0.0, 0.04)]
                    .into_iter()
                    .enumerate()
                {
                    fsphere!(
                        f,
                        format!("bun{i}{j}"),
                        0.045,
                        (x + dx, deck + 0.03, 0.14 + dz),
                        M.bun,
                        scale = (1, 1, 0.7),
                        segments = 10,
                        rings = 5
                    );
                    fsphere!(
                        f,
                        format!("glaze{i}{j}"),
                        0.022,
                        (x + dx, deck + 0.058, 0.14 + dz),
                        M.cream,
                        scale = (1, 1, 0.3),
                        segments = 6,
                        rings = 3
                    );
                }
            } else if k == 1 {
                let P3 {
                    x: bx,
                    y: by_,
                    z: bz,
                } = f.at(x, deck + 0.006, 0.14);
                let secs: Vec<Vec<P3>> = [(0.075, 0.0), (0.086, 0.028), (0.08, 0.03)]
                    .into_iter()
                    .map(|(r, h)| fluted(r, bz + h, 14, 0.08, 3))
                    .collect();
                loft!(
                    format!("tart{i}"),
                    &secs,
                    material = M.pastry,
                    at = (bx, by_, 0)
                );
                fcyl!(
                    f,
                    format!("tart_fill{i}"),
                    0.07,
                    0.008,
                    (x, deck + 0.032, 0.14),
                    if i < 3 { M.jam } else { M.lemon },
                    verts = 14
                );
            } else {
                soft!(
                    f,
                    format!("loaf{i}"),
                    0.19,
                    0.08,
                    0.1,
                    (x, deck + 0.046, 0.14),
                    M.crust,
                    round = 0.035,
                    puff = 0.012,
                    flat = 2
                );
                for j in 0..3 {
                    fbox!(
                        f,
                        format!("score{i}{j}"),
                        0.012,
                        0.004,
                        0.08,
                        (x - 0.05 + j as f64 * 0.05, deck + 0.093, 0.14),
                        M.pastry,
                        rot = (0, 20, 0)
                    );
                }
            }
            if i % 2 == 0 {
                let cake = if i != 2 { M.icing } else { M.choc };
                fcyl!(
                    f,
                    format!("cake{i}"),
                    0.09,
                    0.07,
                    (x, deck + 0.047, -0.2),
                    M.sponge,
                    verts = 14
                );
                fcyl!(
                    f,
                    format!("cake_top{i}"),
                    0.088,
                    0.012,
                    (x, deck + 0.087, -0.2),
                    cake,
                    verts = 14,
                    bevel = 0.005,
                    bevel_segments = 1
                );
                fcyl!(
                    f,
                    format!("cake_band{i}"),
                    0.0905,
                    0.01,
                    (x, deck + 0.045, -0.2),
                    cake,
                    verts = 14
                );
                for j in 0..3 {
                    let a = radians((j * 120 + 30) as f64);
                    fsphere!(
                        f,
                        format!("cherry{i}{j}"),
                        0.013,
                        (x + 0.05 * sin(a), deck + 0.1, -0.2 + 0.05 * cos(a)),
                        M.jam,
                        segments = 6,
                        rings = 4
                    );
                }
            } else {
                for j in 0..4 {
                    fcyl!(
                        f,
                        format!("biscuit{i}{j}"),
                        0.04,
                        0.012,
                        (
                            x - 0.055 + (j % 2) as f64 * 0.11,
                            deck + 0.02 + (j / 2) as f64 * 0.012,
                            -0.2 + (j % 2) as f64 * 0.02
                        ),
                        if j % 2 != 0 { M.pastry } else { M.choc },
                        verts = 8
                    );
                }
            }
        }
        for i in 0..5 {
            let x = -0.72 + i as f64 * 0.36;
            if i % 2 == 0 {
                for (j, dx) in [-0.06, 0.06].into_iter().enumerate() {
                    // a croissant: a crescent of pastry, plump in the middle, curling to its tips
                    let t = if j != 0 { -1.0 } else { 1.0 };
                    let pts: Vec<(f64, f64, f64)> = (-80..81)
                        .step_by(32)
                        .map(|a| {
                            let a = a as f64;
                            (
                                x + dx + 0.06 * cos(radians(a)) * t,
                                1.06 - 0.01 * (1.0 - sin(radians(a)).abs()),
                                -0.27 + 0.04 * sin(radians(a)),
                            )
                        })
                        .collect();
                    fsweep!(
                        f,
                        format!("croissant{i}{j}"),
                        &pts,
                        0.026,
                        M.crust,
                        verts = 6,
                        resolution = 2,
                        scale = by(|f| 0.35 + 0.65 * sin(PI * f))
                    );
                }
            } else {
                for (j, dx) in [-0.065, 0.0, 0.065].into_iter().enumerate() {
                    let P3 {
                        x: bx,
                        y: by_,
                        z: bz,
                    } = f.at(x + dx, 1.035, -0.24);
                    let secs: Vec<Vec<P3>> = [(0.022, 0.0), (0.03, 0.04)]
                        .into_iter()
                        .map(|(r, h)| fluted(r, bz + h, 8, 0.12, 2))
                        .collect();
                    loft!(
                        format!("cupcake{i}{j}"),
                        &secs,
                        material = M.tray,
                        at = (bx, by_, 0)
                    );
                    revolve!(
                        format!("frosting{i}{j}"),
                        &[
                            (0.0, 0.038),
                            (0.032, 0.04),
                            (0.03, 0.055),
                            (0.022, 0.058),
                            (0.02, 0.07),
                            (0.01, 0.076),
                            (0.004, 0.088),
                            (0.0, 0.09)
                        ],
                        at = (bx, by_, bz),
                        material = if j != 1 { M.cream } else { M.icing },
                        segments = 8
                    );
                }
            }
        }
    });
    join(&still, "case", Some(P3::from((0, 0, 0))));

    // the back door: the right-hand pane, in the front track, sliding left in front of the fixed one
    let door = new_parts(|| {
        fbox!(
            f,
            "door_glass",
            0.93,
            0.44,
            0.008,
            (0.455, 1.02, back + 0.008),
            M.glass
        );
    });
    let door = join(&door, "back_door", Some(f.at(0.455, 1.02, back + 0.008)));

    settle!(centre = false);
    slide(
        door,
        "door",
        (0.84, 0, 0),
        Slide {
            seconds: DOOR_SECONDS,
            ..Slide::default()
        },
    ); // Blender +x is the game's -x: to the left
    meshes()
}
