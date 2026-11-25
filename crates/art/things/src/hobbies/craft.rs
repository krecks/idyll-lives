//! H3: the craft table: a birch top on black hairpin legs, the scrapbook, a stack of paper, a pencil pot, washi tape and
//! a box; a stool behind. The scrapbook's cover swings open over its spine for scrapbooking (`book_open`), showing the
//! photos on its first page.

use super::M;
use super::common::{P, ends, flat_quads, stool};
use crate::common::round_at;
use artkit::forms::{circle, rounded_rect};
use artkit::json::float_repr as r;
use artkit::kit::{self, fillet};
use artkit::math::{Axis, P3};
use artkit::parts::{Settle, hinged, join, new_parts, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, revolve, swing, torus};

const TABLE_TOP: f64 = 0.7525;
/// The scrapbook's middle (game x, depth).
const BOOK: (f64, f64) = (-0.1, 0.2);
const BOOK_W: f64 = 0.3;
const BOOK_D: f64 = 0.22;
const PAGES_TOP: f64 = TABLE_TOP + 0.004 + 0.015;

pub fn craft_table() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (bx, bz) = BOOK;
    let still = new_parts(|| {
        // a birch top, its plywood edge showing, on four hairpin legs (each one rod bent in a U), a cutting mat on it
        fextrude!(
            f,
            "top",
            &rounded_rect(1.7, 0.5, 0.015, 2),
            0.035,
            (0, 0.735, 0.2),
            M.craft_top,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        for sx in [-1, 1] {
            for z in [0.0, 0.4] {
                let x = sx as f64 * 0.78;
                fbox!(
                    f,
                    format!("plate{sx}{}", r(z)),
                    0.08,
                    0.006,
                    0.08,
                    (x, 0.7145, z),
                    M.craft_metal,
                    bevel = 0.002,
                    bevel_segments = 1
                );
                let pin = [
                    (x, 0.712, z - 0.03),
                    (x, 0.006, z - 0.004),
                    (x, 0.006, z + 0.004),
                    (x, 0.712, z + 0.03),
                ];
                kit::tube!(
                    f,
                    format!("pin{sx}{}", r(z)),
                    &fillet(&pin, 0.006, 3),
                    0.0055,
                    M.craft_metal,
                    verts = 6,
                    round = 1
                );
            }
        }
        fbox!(
            f,
            "mat",
            0.6,
            0.002,
            0.4,
            (-0.05, TABLE_TOP + 0.001, 0.19),
            M.tree,
            bevel = 0.001,
            bevel_segments = 1
        );
        let mut grid: Vec<(f64, f64, f64, f64, f64)> = (0..12)
            .map(|k| {
                (
                    -0.34 + 0.05 * k as f64,
                    -0.338 + 0.05 * k as f64,
                    0.0,
                    0.38,
                    TABLE_TOP + 0.0025,
                )
            })
            .collect();
        grid.extend((0..8).map(|k| {
            (
                -0.34,
                0.24,
                0.02 + 0.05 * k as f64,
                0.022 + 0.05 * k as f64,
                TABLE_TOP + 0.0025,
            )
        }));
        flat_quads(&f, "mat_grid", &grid, M.label);
        // the scrapbook: back cover, pages (their edges in leaves), spine with rings, the first page's photos
        fbox!(
            f,
            "back_cover",
            BOOK_W,
            0.004,
            BOOK_D,
            (bx, TABLE_TOP + 0.002, bz),
            M.scrapbook,
            bevel = 0.0015
        );
        fbox!(
            f,
            "pages",
            BOOK_W - 0.01,
            0.015,
            BOOK_D - 0.01,
            (bx + 0.003, TABLE_TOP + 0.0115, bz),
            M.pages
        );
        for k in 0..3 {
            fbox!(
                f,
                format!("page_edge{k}"),
                0.0005,
                0.0006,
                BOOK_D - 0.012,
                (
                    bx + BOOK_W / 2.0 - 0.0018,
                    TABLE_TOP + 0.007 + 0.004 * k as f64,
                    bz
                ),
                M.paper
            );
        }
        fbox!(
            f,
            "spine",
            0.008,
            0.023,
            BOOK_D,
            (bx - BOOK_W / 2.0, TABLE_TOP + 0.0115, bz),
            M.scrapbook,
            bevel = 0.002
        );
        for k in 0..4 {
            torus!(
                format!("ring{k}"),
                0.01,
                0.0015,
                at = f.at(
                    bx - BOOK_W / 2.0 + 0.004,
                    TABLE_TOP + 0.0125,
                    bz - 0.08 + 0.053 * k as f64
                ),
                material = M.craft_metal,
                rot = (90, 0, 0),
                major_segments = 8,
                minor_segments = 3
            );
        }
        for (k, (dx, dz, w, d, colour)) in [
            (-0.06, -0.04, 0.1, 0.07, M.photo),
            (0.07, 0.03, 0.08, 0.1, M.photo_warm),
            (-0.05, 0.06, 0.07, 0.05, M.tape_gold),
        ]
        .into_iter()
        .enumerate()
        {
            fbox!(
                f,
                format!("page_photo{k}"),
                w,
                0.001,
                d,
                (bx + dx, PAGES_TOP + 0.0004, bz + dz),
                colour,
                rot = (0, 0, 5 - 4 * k as i32)
            );
        }
        // paper, a pencil pot of pencils, a craft knife and a ruler on the mat, washi tape, glue, a box of buttons
        fbox!(
            f,
            "paper",
            0.4,
            0.008,
            0.3,
            (0.25, TABLE_TOP + 0.004, 0.15),
            M.paper,
            rot = (0, 0, 8),
            bevel = 0.002
        );
        fbox!(
            f,
            "paper2",
            0.3,
            0.004,
            0.22,
            (0.27, TABLE_TOP + 0.01, 0.16),
            M.blush,
            rot = (0, 0, -6),
            bevel = 0.0015
        );
        revolve!(
            "pencil_pot",
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.042, 0.1),
                (0.037, 0.1),
                (0.035, 0.006),
                (0.0, 0.006)
            ],
            at = f.at(0.5, TABLE_TOP, 0.33),
            material = M.cup,
            segments = 14
        );
        for (k, (at, rotate, colour)) in [
            ((0.485, 0.83, 0.32), (12.6, -153.43, 0.0), M.ball_red),
            ((0.515, 0.825, 0.34), (13.95, 26.57, 0.0), M.sun),
            ((0.5, 0.828, 0.31), (10.0, 90.0, 0.0), M.tape_blue),
        ]
        .into_iter()
        .enumerate()
        {
            let (a, b) = ends(at, 0.15, rotate);
            let (lo, hi): (P, P) = if a.1 < b.1 { (a, b) } else { (b, a) };
            fsweep!(
                f,
                format!("pencil{k}"),
                &[lo, hi],
                0,
                colour,
                profile = circle(0.004, 6)
            );
            let tip = (
                hi.0 + (hi.0 - lo.0) * 0.06,
                hi.1 + (hi.1 - lo.1) * 0.06,
                hi.2 + (hi.2 - lo.2) * 0.06,
            );
            fsweep!(
                f,
                format!("pencil_tip{k}"),
                &[hi, tip],
                0.004,
                M.paper,
                verts = 6,
                scale = [1.0, 0.15]
            );
        }
        fextrude!(
            f,
            "ruler",
            &rounded_rect(0.3, 0.03, 0.002, 1),
            0.002,
            (-0.05, TABLE_TOP + 0.003, 0.36),
            M.chrome,
            plane = Facing::Top,
            rot = (0, 0, 3)
        );
        fsweep!(
            f,
            "knife",
            &[
                (0.14, TABLE_TOP + 0.007, 0.35),
                (0.25, TABLE_TOP + 0.007, 0.375)
            ],
            0.006,
            M.ball_yellow,
            verts = 6
        );
        fsweep!(
            f,
            "knife_blade",
            &[
                (0.25, TABLE_TOP + 0.007, 0.375),
                (0.27, TABLE_TOP + 0.007, 0.38)
            ],
            0.004,
            M.chrome,
            verts = 4,
            scale = [1.0, 0.1]
        );
        for (k, (x, z, colour)) in [
            (0.65, 0.22, M.tape_blue),
            (0.74, 0.14, M.tape_gold),
            (0.66, 0.08, M.blush),
        ]
        .into_iter()
        .enumerate()
        {
            revolve!(
                format!("tape{k}"),
                // (the roll's top edges worn round)
                &round_at(
                    &round_at(
                        &[
                            (0.026, 0.0),
                            (0.052, 0.0),
                            (0.052, 0.015),
                            (0.026, 0.015),
                            (0.026, 0.0),
                        ],
                        3,
                        0.0015
                    ),
                    2,
                    0.0015
                ),
                at = f.at(x, TABLE_TOP, z),
                material = colour,
                segments = 16
            );
        }
        revolve!(
            "glue",
            &[
                (0.0, 0.0),
                (0.02, 0.0),
                (0.02, 0.07),
                (0.012, 0.08),
                (0.006, 0.09),
                (0.0, 0.092)
            ],
            at = f.at(0.6, TABLE_TOP, 0.36),
            material = M.label,
            segments = 10
        );
        fcyl!(
            f,
            "glue_cap",
            0.008,
            0.015,
            (0.6, TABLE_TOP + 0.09, 0.36),
            M.ball_red,
            verts = 8
        );
        fbox!(
            f,
            "box",
            0.16,
            0.05,
            0.1,
            (0.72, TABLE_TOP + 0.025, 0.37),
            M.scrapbook,
            bevel = 0.01
        );
        fbox!(
            f,
            "box_lid",
            0.165,
            0.012,
            0.105,
            (0.72, TABLE_TOP + 0.054, 0.37),
            M.tape_gold,
            bevel = 0.004
        );
        stool(&f, -0.25, 0.48, 0.19, M.craft_metal, M.craft_top, 0.0);
    });
    join(&still, "body", Some(P3::ZERO));
    let c = new_parts(|| {
        fbox!(
            f,
            "cover",
            BOOK_W,
            0.004,
            BOOK_D,
            (bx, PAGES_TOP + 0.002, bz),
            M.scrapbook,
            bevel = 0.0015
        );
        fbox!(
            f,
            "cover_label",
            0.12,
            0.002,
            0.07,
            (bx + 0.02, PAGES_TOP + 0.005, bz - 0.02),
            M.pages,
            bevel = 0.0008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "cover_photo",
            0.08,
            0.002,
            0.06,
            (bx - 0.05, PAGES_TOP + 0.005, bz + 0.05),
            M.photo,
            rot = (0, 0, -6)
        );
        for (k, colour) in [M.tape_blue, M.tape_gold].into_iter().enumerate() {
            let o = 0.03 * (2 * k as i32 - 1) as f64;
            fbox!(
                f,
                format!("cover_tape{k}"),
                0.03,
                0.0025,
                0.01,
                (bx - 0.05 + o, PAGES_TOP + 0.0055, bz + 0.05 + o),
                colour,
                rot = (0, 0, 40)
            );
        }
        fbox!(
            f,
            "inside_photo",
            0.1,
            0.001,
            0.08,
            (bx + 0.03, PAGES_TOP - 0.0003, bz),
            M.photo_warm
        );
    });
    let cover = hinged(
        join(&c, "cover", None),
        f.at(bx - BOOK_W / 2.0, PAGES_TOP + 0.004, bz),
    );
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    swing!(cover, "book_open", 176.0, seconds = 1.0, axis = Axis::Y);
    meshes()
}
