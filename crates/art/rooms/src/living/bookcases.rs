//! L5 continued: the industrial shelving, the tall and corner bookcases, the cube and ladder shelves and the
//! display cabinet.

use super::common::disc;
use super::shelves::{
    WALL, book_row, bookcase, floor, frame_photo, plant, prism, quarter, tips_out, vase,
};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::math::{PI, sin};
use artkit::motion::Ease;
use artkit::parts::{hinged, join, join_all, new_parts};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fcone, fcyl, fextrude, fsphere, rod, soft, sweep, swing};

/// The industrial shelving (the old part): five oak boards on black steel posts and angles, its top at 1.83 (content
/// `surface`); a cross brace behind, books, boxes and a plant.
pub fn shelving_industrial() -> Vec<Obj> {
    let (zb, zf) = (WALL, 0.10);
    let (zm, d) = ((zb + zf) / 2.0, zf - zb);
    for x in [-0.9125, 0.0, 0.9125] {
        for z in [zb + 0.0175, zf - 0.0175] {
            fbox!(
                F,
                "post",
                0.035,
                1.8,
                0.035,
                (x, 0.9, z),
                M.black_metal,
                bevel = 0.004
            );
            fcyl!(
                F,
                "foot",
                0.0175,
                0.012,
                (x, 0.006, z),
                M.black_metal,
                verts = 10
            );
        }
    }
    for y in [0.12, 0.56, 1.0, 1.44, 1.815] {
        fbox!(
            F,
            "board",
            1.86,
            0.03,
            d,
            (0, y, zm),
            M.oak_shelf,
            bevel = 0.004
        );
        for z in [zb + 0.0125, zf - 0.0125] {
            fbox!(
                F,
                "angle",
                1.86,
                0.03,
                0.025,
                (0, y - 0.03, z),
                M.black_metal,
                bevel = 0.003
            );
        }
        for x in [-0.9125, 0.9125] {
            fbox!(
                F,
                "end_angle",
                0.035,
                0.03,
                d - 0.05,
                (x, y - 0.03, zm),
                M.black_metal,
                bevel = 0.003
            );
        }
    }
    for (a, b) in [((-0.89, 0.15), (-0.02, 1.4)), ((-0.02, 0.15), (-0.89, 1.4))] {
        rod!(
            "brace",
            F.at(a.0, a.1, zb + 0.012),
            F.at(b.0, b.1, zb + 0.012),
            0.0075,
            material = M.black_metal,
            verts = 6
        );
    }
    let front = zf - 0.07;
    book_row(-0.86, -0.06, 0.575, front, 0.24, 0.3, 31, 1.0, false, None);
    book_row(0.08, 0.6, 1.015, front, 0.24, 0.3, 32, 1.0, true, None);
    book_row(-0.5, 0.0, 1.455, front, 0.24, 0.3, 33, 1.0, false, None);
    for (x, w, h, mat) in [
        (-0.62, 0.38, 0.26, M.box_sand),
        (-0.21, 0.38, 0.26, M.photo),
        (0.5, 0.36, 0.24, M.box_sand),
    ] {
        fbox!(
            F,
            "storage_box",
            w,
            h,
            0.4,
            (x, 0.135 + h / 2.0, zm),
            mat,
            bevel = 0.012
        );
        fbox!(
            F,
            "box_handle",
            0.1,
            0.03,
            0.004,
            (x, 0.135 + h * 0.75, zm + 0.2),
            M.bin_hole,
            bevel = 0.001
        );
    }
    plant(0.75, 1.015, zm, 0.07, 0.13, M.clay, 0.12);
    vase(-0.6, 1.015, zm, 0.07, 0.14, M.vase);
    frame_photo(0.4, 1.455, zm - 0.1, M.black_metal, 0.2, 0.26);
    join_all("shelvingIndustrial");
    floor();
    Vec::new()
}

/// The tall bookcase (the old part): walnut, 0.9 wide and 2 high, six spaces of books and ornaments, a dark back.
pub fn bookcase_tall() -> Vec<Obj> {
    let (tops, zf, _gap) = bookcase(0.9, 2.0, 0.34, M.walnut_warm, M.walnut_back, 6, 0.0);
    let (front, zm) = (zf - 0.02, WALL + 0.17);
    let mut out = None;
    for (i, &y) in tops.iter().enumerate() {
        if i == 0 || i == 3 {
            vase(
                -0.22,
                y,
                zm,
                0.06,
                0.2,
                if i == 0 { M.vase } else { M.red_gloss },
            );
            fbox!(
                F,
                "storage_box",
                0.2,
                0.1,
                0.16,
                (0.18, y + 0.05, zm),
                M.clay,
                bevel = 0.008
            );
            if i == 3 {
                frame_photo(0.18, y + 0.1, zm - 0.02, M.brass, 0.13, 0.16);
            }
        } else {
            let got = book_row(
                -0.42,
                0.42,
                y,
                front,
                0.24,
                0.26,
                40 + i as i128,
                if i % 2 != 0 { 0.9 } else { 1.0 },
                i == 2,
                if i == 4 { Some(5) } else { None },
            );
            out = got.or(out);
        }
    }
    tips_out("bookcaseTall", out)
}

/// The corner bookcase (the old part): a white bookcase with a teal back and, round its right end, quarter-round open
/// shelves to the wall, with a plant and a vase on them.
pub fn bookcase_corner() -> Vec<Obj> {
    let (x0, x1, r) = (-0.42, 0.14, 0.34); // the bookcase's sides; the end shelves' radius
    let (tops, zf, _gap) = bookcase(
        x1 - x0,
        1.9,
        r,
        M.shelf_white,
        M.shelf_teal,
        5,
        (x0 + x1) / 2.0,
    );
    quarter("end_kick", x1, WALL, r - 0.02, 0.0, 0.038, M.shelf_white);
    for &y in &tops {
        quarter("end_shelf", x1, WALL, r, y - 0.022, y, M.shelf_white);
    }
    quarter("end_top", x1, WALL, r, 1.87, 1.9, M.shelf_white);
    fbox!(
        F,
        "end_back",
        r - 0.01,
        1.87 - 0.06,
        0.01,
        (
            x1 + (r - 0.01) / 2.0,
            0.06 + (1.87 - 0.06) / 2.0,
            WALL + 0.005
        ),
        M.shelf_teal
    );
    let front = zf - 0.02;
    let mut out = None;
    for (i, &y) in tops.iter().enumerate() {
        let got = book_row(
            x0 + 0.03,
            x1 - 0.03,
            y,
            front,
            0.22,
            0.27,
            50 + i as i128,
            if i != 1 { 0.95 } else { 0.7 },
            false,
            if i == 3 { Some(3) } else { None },
        );
        out = got.or(out);
    }
    plant(x1 + 0.12, tops[2], WALL + 0.12, 0.055, 0.1, M.clay, 0.1);
    vase(x1 + 0.12, tops[3], WALL + 0.12, 0.05, 0.2, M.blue_gloss);
    book_row(
        x1 + 0.03,
        x1 + 0.2,
        tops[1],
        WALL + 0.24,
        0.16,
        0.1,
        59,
        0.0,
        true,
        None,
    );
    tips_out("bookcaseCorner", out)
}

/// The cube shelf (the old part): four open white cubes, a pale back, two fabric bins, books and a box; its top at 0.8
/// (content `surface`).
pub fn cube_shelf() -> Vec<Obj> {
    let (w, h, d, t) = (0.8, 0.8, 0.36, 0.022);
    let (zf, zm) = (WALL + d, WALL + d / 2.0);
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            "side",
            t,
            h,
            d,
            (s * (w / 2.0 - t / 2.0), h / 2.0, zm),
            M.shelf_white,
            bevel = 0.003
        );
    }
    for y in [t / 2.0, h - t / 2.0] {
        fbox!(
            F,
            "board",
            w - 2.0 * t,
            t,
            d,
            (0, y, zm),
            M.shelf_white,
            bevel = 0.003
        );
    }
    fbox!(
        F,
        "middle",
        w - 2.0 * t,
        t,
        d - 0.01,
        (0, h / 2.0, zm + 0.005),
        M.shelf_white,
        bevel = 0.002
    );
    fbox!(
        F,
        "upright",
        t,
        h - 2.0 * t,
        d - 0.01,
        (0, h / 2.0, zm + 0.005),
        M.shelf_white,
        bevel = 0.002
    );
    fbox!(
        F,
        "back",
        w - 2.0 * t,
        h - 2.0 * t,
        0.01,
        (0, h / 2.0, WALL + 0.005),
        M.cube_back
    );
    for (x, y, mat) in [
        (-0.19, t, M.bin_sage),
        (0.19, h / 2.0 + t / 2.0, M.bin_clay),
    ] {
        soft!(
            F,
            "bin",
            0.33,
            0.31,
            0.31,
            (x, y + 0.155, zm + 0.01),
            mat,
            round = 0.02,
            puff = (0.006, 0.0, 0.006),
            flat = 3
        );
        let rim: Vec<_> = rounded_rect(0.315, 0.295, 0.02, 2)
            .iter()
            .map(|&(u, v)| F.at(x + u, y + 0.305, zm + 0.01 + v))
            .collect();
        sweep!(
            "bin_rim",
            &rim,
            0.008,
            material = mat,
            verts = 5,
            closed = true
        );
        fbox!(
            F,
            "bin_hole",
            0.1,
            0.03,
            0.006,
            (x, y + 0.25, zm + 0.166),
            M.bin_hole,
            bevel = 0.002
        );
    }
    book_row(
        -0.37,
        -0.03,
        h / 2.0 + t / 2.0,
        zf - 0.03,
        0.26,
        0.3,
        61,
        1.0,
        false,
        None,
    );
    fbox!(
        F,
        "lidded_box",
        0.22,
        0.12,
        0.24,
        (0.19, t + 0.06, zm),
        M.vase,
        bevel = 0.012
    );
    book_row(
        0.1,
        0.22,
        t + 0.12,
        zf - 0.07,
        0.17,
        0.1,
        62,
        0.0,
        true,
        None,
    );
    join_all("cubeShelf");
    floor();
    Vec::new()
}

pub(super) const LADDER_H: f64 = 1.8;

/// The depth of the ladder's back at height y: 32 cm out from the wall at the floor, on it at the top.
pub(super) fn ladder_back(y: f64) -> f64 {
    WALL + 0.32 * (1.0 - y / LADDER_H)
}

/// The ladder shelf (the old part): two oak rails leaning on the wall, four boards through them narrowing upwards, a
/// plant and a basket, books, vases and a picture.
pub fn ladder_shelf() -> Vec<Obj> {
    let t = 0.036; // the rails, front to back
    let outline = [
        (ladder_back(0.0), 0.0),
        (ladder_back(0.0) + t, 0.0),
        (ladder_back(LADDER_H) + t, LADDER_H),
        (ladder_back(LADDER_H), LADDER_H),
    ];
    for x in [-0.3, 0.3] {
        prism("rail", &outline, x - 0.0175, x + 0.0175, M.oak, true);
    }
    let mut tops = Vec::new();
    for (i, y) in [0.36, 0.76, 1.14, 1.5].into_iter().enumerate() {
        // through the rails, 6 cm proud of their fronts, back to the wall (or 36 cm less 6 a board)
        let zf = ladder_back(y) + t + 0.06;
        let d = (0.36 - i as f64 * 0.06).min(zf - WALL - 0.004);
        fbox!(
            F,
            "board",
            0.62,
            0.025,
            d,
            (0, y, zf - d / 2.0),
            M.oak_shelf,
            bevel = 0.004
        );
        tops.push((y + 0.0125, zf - d, zf));
    }
    let (y, zb, zf) = tops[0];
    plant(-0.12, y, (zb + zf) / 2.0, 0.065, 0.12, M.porcelain, 0.11);
    fbox!(
        F,
        "basket",
        0.18,
        0.12,
        0.2,
        (0.14, y + 0.06, (zb + zf) / 2.0),
        M.basket,
        bevel = 0.015
    );
    let (y, zb, zf) = tops[1];
    book_row(0.0, 0.26, y, zf - 0.03, 0.18, 0.22, 71, 1.0, false, None);
    vase(-0.16, y, (zb + zf) / 2.0, 0.05, 0.16, M.porcelain);
    let (y, zb, zf) = tops[2];
    vase(-0.15, y, (zb + zf) / 2.0, 0.05, 0.18, M.clay);
    frame_photo(0.13, y, zb + 0.05, M.walnut_warm, 0.16, 0.2);
    let (y, zb, zf) = tops[3];
    book_row(-0.22, -0.02, y, zf - 0.02, 0.11, 0.1, 72, 0.0, true, None);
    plant(0.14, y, (zb + zf) / 2.0, 0.04, 0.06, M.clay, 0.05);
    join_all("ladderShelf");
    floor();
    Vec::new()
}

// The display cabinet's glazed door, hinged on its left (game -x): a part of its own, still for now, for a (+use)
// swing about DOOR_HINGE (the plan's L5).
pub(super) const DOOR_HINGE: (f64, f64) = (-0.4, -0.022); // (game x, depth)

/// The display cabinet (the old part): a walnut vitrine on a plinth and feet under a cornice, a pale back, three
/// glass shelves of ornaments behind a brass-framed glass door (its own part, still).
pub fn display_cabinet() -> Vec<Obj> {
    let zm = WALL + 0.22;
    let body = new_parts(|| {
        fbox!(
            F,
            "base",
            0.84,
            0.12,
            0.44,
            (0, 0.12, zm),
            M.vitrine,
            bevel = 0.01
        );
        // the cornice: a cove swelling out under a square cap, and a bead under it
        let mut cove = vec![(WALL, 1.85), (WALL + 0.44, 1.85)];
        cove.extend((1..5).map(|i| {
            let i = f64::from(i);
            (
                WALL + 0.44 + 0.01 * sin(PI / 2.0 * i / 4.0),
                1.85 + 0.045 * i / 4.0,
            )
        }));
        cove.push((WALL, 1.895));
        fextrude!(
            F,
            "cornice",
            &cove,
            0.84,
            (0, 0, 0),
            M.vitrine,
            plane = Facing::Side,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            F,
            "cap",
            0.86,
            0.015,
            0.45,
            (0, 1.9025, zm + 0.005),
            M.vitrine,
            bevel = 0.005,
            bevel_segments = 2
        );
        fbox!(
            F,
            "cornice_bead",
            0.8,
            0.012,
            0.012,
            (0, 1.846, WALL + 0.438),
            M.vitrine,
            bevel = 0.005,
            bevel_segments = 2
        );
        fbox!(
            F,
            "base_bead",
            0.8,
            0.012,
            0.012,
            (0, 0.174, WALL + 0.438),
            M.vitrine,
            bevel = 0.005,
            bevel_segments = 2
        );
        for x in [-0.37, 0.37] {
            for z in [WALL + 0.03, -0.02] {
                // a turned bun foot
                disc(
                    "foot",
                    &[
                        (0.0, 0.014),
                        (0.006, 0.02),
                        (0.022, 0.025),
                        (0.04, 0.021),
                        (0.052, 0.016),
                        (0.06, 0.02),
                    ],
                    M.vitrine,
                    x,
                    z,
                    12,
                );
            }
        }
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                "side",
                0.024,
                1.67,
                0.4,
                (s * 0.388, 1.015, WALL + 0.2),
                M.vitrine,
                bevel = 0.003
            );
        }
        fbox!(
            F,
            "back",
            0.752,
            1.67,
            0.012,
            (0, 1.015, WALL + 0.006),
            M.vitrine_back
        );
        for y in [0.6, 1.05, 1.5] {
            fbox!(
                F,
                "shelf",
                0.752,
                0.01,
                0.36,
                (0, y, WALL + 0.192),
                M.shelf_glass
            );
        }
        let z = WALL + 0.2;
        vase(-0.2, 0.18, z, 0.07, 0.24, M.navy_gloss);
        fsphere!(
            F,
            "globe",
            0.07,
            (0.18, 0.25, z),
            M.brass,
            segments = 16,
            rings = 10
        );
        fcone!(
            F,
            "cup",
            0.035,
            0.05,
            0.1,
            (-0.18, 0.655, z),
            M.cream_gloss,
            verts = 16
        );
        frame_photo(0.16, 0.605, z - 0.06, M.brass, 0.16, 0.2);
        vase(0.0, 1.055, z, 0.05, 0.2, M.red_gloss);
        fsphere!(
            F,
            "bowl",
            0.05,
            (0.2, 1.105, z),
            M.blue_gloss,
            segments = 14,
            rings = 8
        );
        fcyl!(
            F,
            "plate",
            0.1,
            0.012,
            (-0.12, 1.615, WALL + 0.05),
            M.porcelain,
            rot = (78, 0, 0),
            verts = 24
        );
        fbox!(
            F,
            "plate_stand",
            0.06,
            0.02,
            0.05,
            (-0.12, 1.515, WALL + 0.06),
            M.brass
        );
        fcyl!(
            F,
            "candlestick",
            0.012,
            0.16,
            (0.18, 1.585, z),
            M.brass,
            verts = 10
        );
        fcyl!(
            F,
            "candle_foot",
            0.035,
            0.012,
            (0.18, 1.511, z),
            M.brass,
            verts = 14
        );
    });
    join(&body, "cabinet", Some((0, 0, 0).into()));
    let (hx, hz) = DOOR_HINGE;
    let parts = new_parts(|| {
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                "stile",
                0.022,
                1.67,
                0.016,
                (s * 0.389, 1.015, hz),
                M.brass,
                bevel = 0.003
            );
        }
        for y in [0.191, 1.839] {
            fbox!(
                F,
                "rail",
                0.756,
                0.022,
                0.016,
                (0, y, hz),
                M.brass,
                bevel = 0.003
            );
        }
        fbox!(F, "pane", 0.758, 1.628, 0.005, (0, 1.015, hz), M.glass);
        fsphere!(
            F,
            "knob",
            0.013,
            (0.36, 1.0, hz + 0.022),
            M.brass,
            segments = 12,
            rings = 8
        );
        fcyl!(
            F,
            "knob_stem",
            0.005,
            0.016,
            (0.36, 1.0, hz + 0.012),
            M.brass,
            rot = (90, 0, 0),
            verts = 8
        );
    });
    let door = hinged(join(&parts, "door", None), F.at(hx, 1.015, hz));
    floor();
    swing!(
        door,
        "door_open",
        -100.0,
        seconds = 1.0,
        ease = Ease::BackOut
    );
    Vec::new()
}
