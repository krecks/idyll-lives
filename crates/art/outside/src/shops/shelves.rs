//! The grocery shelves: 2 x 1, a metal gondola of tins, packets and jars, sandwiches at the bottom, sitting back
//! on its tiles as the game places it.

use super::common::{M, bottle, prism};
use artkit::math::P3;
use artkit::parts::{join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, settle};

// --- the grocery shelves: 2x1, a metal gondola of tins, packets and jars, sandwiches at the bottom ---
pub fn grocery_shelf() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let goods = [
        M.good1, M.good2, M.good3, M.good4, M.good5, M.good6, M.good7, M.good8, M.good9,
    ];
    let still = new_parts(|| {
        // the gondola: shaped sides (their front edge rounded off at the top), a slotted back, a kick plate on
        // feet, a top cap carrying a coloured header strip
        let side = [
            (-0.43, 0.0),
            (0.04, 0.0),
            (0.04, 1.6),
            (0.0, 1.68),
            (-0.08, 1.7),
            (-0.43, 1.7),
        ];
        for s in [-1, 1] {
            fextrude!(
                f,
                format!("side{s}"),
                &side,
                0.04,
                (s as f64 * 0.94, 0, 0),
                M.shelf,
                plane = Facing::Side,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "back",
            1.84,
            1.62,
            0.025,
            (0, 0.89, -0.4175),
            M.shelf,
            bevel = 0.004
        );
        for k in 0..9 {
            // the back's slots, between the shelves
            fbox!(
                f,
                format!("slot{k}"),
                0.012,
                1.4,
                0.004,
                (-0.8 + 0.2 * k as f64, 0.92, -0.404),
                M.trim
            );
        }
        fbox!(
            f,
            "kick",
            1.84,
            0.1,
            0.44,
            (0, 0.05, -0.19),
            M.trim,
            bevel = 0.004
        );
        fbox!(
            f,
            "cap",
            1.92,
            0.04,
            0.48,
            (0, 1.70, -0.19),
            M.trim,
            bevel = 0.006
        );
        fbox!(f, "header", 1.84, 0.03, 0.003, (0, 1.70, 0.0495), M.good5);
        let mut k: usize = 1;
        for (row, y) in [0.115, 0.52, 0.92, 1.32].into_iter().enumerate() {
            fbox!(
                f,
                format!("shelf{row}"),
                1.84,
                0.025,
                0.44,
                (0, y, -0.19),
                M.shelf,
                bevel = 0.003
            );
            // a price channel along the front: a dark lip holding a white strip and its tags
            fbox!(
                f,
                format!("lip{row}"),
                1.84,
                0.045,
                0.012,
                (0, y - 0.004, 0.036),
                M.trim,
                bevel = 0.003
            );
            fbox!(
                f,
                format!("strip{row}"),
                1.8,
                0.022,
                0.003,
                (0, y - 0.004, 0.0425),
                M.label
            );
            for t in 0..4 {
                fbox!(
                    f,
                    format!("tag{row}{t}"),
                    0.06,
                    0.03,
                    0.004,
                    (-0.66 + t as f64 * 0.44, y - 0.004, 0.045),
                    [M.good5, M.good1][t % 2]
                );
            }
            let (mut x, top) = (-0.89, y + 0.0125);
            let mut n = 0;
            while x < 0.84 {
                k = (k * 7 + 5) % 23;
                let c = goods[k % goods.len()];
                let name = format!("g{row}_{n}");
                n += 1;
                let w;
                if row == 0 {
                    // sandwich packs: triangles standing on their long side, two deep
                    let cx;
                    (w, cx) = (0.15, x + 0.075);
                    for (j, z) in [-0.23, -0.06].into_iter().enumerate() {
                        prism(
                            &format!("{name}_{j}"),
                            &[(cx - 0.07, top), (cx + 0.07, top), (cx, top + 0.12)],
                            z - 0.035,
                            z + 0.035,
                            M.bread,
                        );
                    }
                    fbox!(
                        f,
                        format!("{name}_fill"),
                        0.075,
                        0.014,
                        0.072,
                        (cx, top + 0.05, -0.06),
                        M.filling
                    );
                    fbox!(
                        f,
                        format!("{name}_sticker"),
                        0.05,
                        0.03,
                        0.074,
                        (cx, top + 0.022, -0.06),
                        c
                    );
                } else if k.is_multiple_of(3) {
                    // tins, two deep, a label round each
                    w = 0.1;
                    for (j, z) in [-0.28, -0.15, -0.02].into_iter().enumerate() {
                        fcyl!(
                            f,
                            format!("{name}_{j}"),
                            0.045,
                            0.12,
                            (x + w / 2.0, top + 0.06, z),
                            c,
                            verts = 10
                        );
                        fcyl!(
                            f,
                            format!("{name}_lid{j}"),
                            0.04,
                            0.006,
                            (x + w / 2.0, top + 0.123, z),
                            M.tinLid,
                            verts = 10
                        );
                    }
                    fcyl!(
                        f,
                        format!("{name}_label"),
                        0.0455,
                        0.05,
                        (x + w / 2.0, top + 0.06, -0.02),
                        M.label,
                        verts = 10
                    );
                } else if k.is_multiple_of(5) {
                    // bottles of sauce and oil
                    w = 0.11;
                    for (j, z) in [-0.2, -0.02].into_iter().enumerate() {
                        bottle(
                            &format!("{name}_{j}"),
                            (x + w / 2.0, top, z),
                            0.04,
                            0.2 + 0.02 * (k % 2) as f64,
                            c,
                            M.tinLid,
                            &f,
                        );
                    }
                } else {
                    // packets, a label on each and a window on some
                    w = (0.13 + (k % 3) as f64 * 0.03).min(0.92 - x);
                    let h = 0.18 + (k % 4) as f64 * 0.04;
                    fbox!(
                        f,
                        name.as_str(),
                        w - 0.02,
                        h,
                        0.3,
                        (x + w / 2.0, top + h / 2.0, -0.15),
                        c,
                        bevel = 0.006,
                        bevel_segments = 1
                    );
                    fbox!(
                        f,
                        format!("{name}_label"),
                        w - 0.06,
                        h * 0.3,
                        0.004,
                        (x + w / 2.0, top + h * 0.55, 0.002),
                        M.label
                    );
                    fbox!(
                        f,
                        format!("{name}_band"),
                        w - 0.02,
                        0.02,
                        0.004,
                        (x + w / 2.0, top + h * 0.85, 0.002),
                        if c.name != M.good7.name {
                            M.good7
                        } else {
                            M.good1
                        }
                    );
                }
                x += w + 0.015;
            }
        }
    });
    join(&still, "shelves", Some(P3::from((0, 0, 0))));
    settle!(centre = false);
    meshes()
}
