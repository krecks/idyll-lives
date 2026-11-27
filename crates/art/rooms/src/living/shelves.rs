//! L5, the shelves. Each stands with its back on the wall's face (depth -0.43) and its tops at the content `surface`
//! heights (the low bookcase 0.9, the cube shelf 0.8, the industrial shelving 1.83), built in place and settled there
//! (`centre` on its Model). The books are many small parts, joined into the still body; on the bookcases one of them
//! is a part of its own that tips out 25° from its shelf while a resident reads, studies or does homework there
//! (`book_out`), and back at the end.

use super::common::knob;
use super::{F, M};
use artkit::materials::Mat;
use artkit::math::Axis;
use artkit::parts::{hinged, join};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, settle, swing};

pub(super) use artkit::kit::WALL;
// The helpers moved to shelf_parts.rs; the other living files still reach them here.
pub(super) use super::shelf_parts::*;

/// An open bookcase w × h × d at x, its back on the wall's face: two sides, a top, a bottom board on a recessed kick,
/// a back panel between the sides and n - 1 shelves, n even spaces (the old `bookcase`). Returns the tops books stand
/// on (the bottom's first), the front's depth and the space between shelves.
#[allow(clippy::too_many_arguments)]
pub(super) fn bookcase(
    w: f64,
    h: f64,
    d: f64,
    frame: Mat,
    back: Mat,
    n: i32,
    x: f64,
) -> (Vec<f64>, f64, f64) {
    let (t, top_t, plinth) = (0.022, 0.03, 0.06);
    let (zf, zm) = (WALL + d, WALL + d / 2.0);
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            "side",
            t,
            h,
            d,
            (x + s * (w / 2.0 - t / 2.0), h / 2.0, zm),
            frame,
            bevel = 0.003
        );
    }
    fbox!(
        F,
        "top",
        w,
        top_t,
        d,
        (x, h - top_t / 2.0, zm),
        frame,
        bevel = 0.006,
        bevel_segments = 2
    );
    fbox!(
        F,
        "top_bead",
        w - 2.0 * t,
        0.014,
        0.012,
        (x, h - top_t - 0.007, zf - 0.007),
        frame,
        bevel = 0.005,
        bevel_segments = 2
    );
    fbox!(
        F,
        "kick",
        w - 2.0 * t,
        plinth - t,
        d - 0.02,
        (x, (plinth - t) / 2.0, zm - 0.01),
        frame
    );
    fbox!(
        F,
        "bottom",
        w - 2.0 * t,
        t,
        d,
        (x, plinth - t / 2.0, zm),
        frame,
        bevel = 0.002
    );
    fbox!(
        F,
        "back",
        w - 2.0 * t,
        h - top_t - plinth,
        0.01,
        (x, plinth + (h - top_t - plinth) / 2.0, WALL + 0.005),
        back
    );
    let gap = (h - plinth - top_t) / f64::from(n);
    let mut tops = vec![plinth];
    for i in 1..n {
        let y = plinth + gap * f64::from(i);
        fbox!(
            F,
            "shelf",
            w - 2.0 * t,
            0.022,
            d - 0.012,
            (x, y, zm + 0.004),
            frame,
            bevel = 0.002
        );
        fbox!(
            F,
            "shelf_lip",
            w - 2.0 * t,
            0.032,
            0.01,
            (x, y - 0.005, zf - 0.007),
            frame,
            bevel = 0.003,
            bevel_segments = 1
        );
        tops.push(y + 0.011);
    }
    (tops, zf, gap)
}

/// The still body joined, the book `out` (parts, hinge) a part of its own, settled in place; `book_out` tips it
/// forward about the foot of its spine.
pub(super) fn tips_out(name: &str, out: Option<Out>) -> Vec<Obj> {
    let Some((parts, hinge)) = out else {
        panic!("{name}: no book to tip out");
    };
    let rest: Vec<Obj> = meshes()
        .into_iter()
        .filter(|o| !parts.contains(o))
        .collect();
    join(&rest, name, Some((0, 0, 0).into()));
    let book = hinged(join(&parts, "book", None), F.at(hinge.0, hinge.1, hinge.2));
    settle!(centre = false);
    swing!(book, "book_out", BOOK_TIP, seconds = 0.6, axis = Axis::X);
    Vec::new()
}

/// The bookshelf (base manifest): oak sides and top, a dark back and kick, four shelves of books and a little plant
/// on the top shelf.
pub fn bookshelf() -> Vec<Obj> {
    let d = 0.375;
    let (zf, zm) = (WALL + d, WALL + d / 2.0);
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            "side",
            0.035,
            1.9,
            d,
            (s * 0.4325, 0.95, zm),
            M.oak,
            bevel = 0.006,
            bevel_segments = 1
        );
    }
    fbox!(F, "top", 0.9, 0.035, d, (0, 1.9, zm), M.oak, bevel = 0.006);
    fbox!(
        F,
        "back",
        0.83,
        1.8,
        0.012,
        (0, 0.98, WALL + 0.006),
        M.shelf_back,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        "kick",
        0.83,
        0.075,
        0.02,
        (0, 0.0375, zf - 0.03),
        M.shelf_back
    );
    fbox!(
        F,
        "bottom",
        0.83,
        0.025,
        d - 0.012,
        (0, 0.0875, zm + 0.006),
        M.oak,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        "top_bead",
        0.83,
        0.014,
        0.012,
        (0, 1.8755 - 0.007, zf - 0.007),
        M.oak,
        bevel = 0.005,
        bevel_segments = 2
    );
    for y in [0.52, 0.94, 1.36, 1.76] {
        fbox!(
            F,
            "shelf",
            0.83,
            0.025,
            d - 0.012,
            (0, y - 0.0125, zm + 0.006),
            M.oak,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            F,
            "shelf_lip",
            0.83,
            0.034,
            0.01,
            (0, y - 0.017, zf - 0.006),
            M.oak,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    let front = zf - 0.03;
    hardback_row(
        -0.405, 0.405, 0.1, front, 0.22, 0.32, 1, 0.8, false, None, 0.405,
    );
    hardback_row(
        -0.405, 0.405, 0.52, front, 0.22, 0.3, 2, 0.95, true, None, 0.405,
    );
    let out = hardback_row(
        -0.405,
        0.405,
        0.94,
        front,
        0.22,
        0.32,
        3,
        1.0,
        false,
        Some(9),
        0.405,
    );
    hardback_row(
        -0.405, 0.2, 1.36, front, 0.22, 0.3, 4, 1.0, false, None, 0.405,
    );
    hardback_row(
        -0.405, -0.1, 1.76, front, 0.2, 0.1, 5, 0.9, true, None, 0.405,
    );
    plant(0.24, 1.76, zm, 0.045, 0.06, M.clay, 0.05);
    tips_out("bookshelf", out)
}

/// The low bookcase (the old part): oak, 0.92 wide, its top at 0.9 (content `surface`); two shelves of books, a plant
/// on top.
pub fn bookcase_low() -> Vec<Obj> {
    let (tops, zf, _) = bookcase(0.92, 0.9, 0.38, M.oak, M.oak_back, 2, 0.0);
    let front = zf - 0.025;
    book_row(-0.43, 0.43, tops[0], front, 0.26, 0.29, 11, 1.0, true, None);
    let out = book_row(
        -0.43,
        0.43,
        tops[1],
        front,
        0.26,
        0.28,
        12,
        0.78,
        false,
        Some(4),
    );
    vase(0.33, tops[1], WALL + 0.2, 0.05, 0.16, M.navy_gloss);
    plant(0.3, 0.9, -0.24, 0.06, 0.11, M.pot_white, 0.1);
    tips_out("bookcaseLow", out)
}

/// The library bookcase (the old part): oak, 1.86 wide and 2 high, cupboards under open shelves of books, plants and
/// a vase; brass knobs.
pub fn bookcase_wide() -> Vec<Obj> {
    let (w, h, d) = (1.86, 2.0, 0.42);
    let (zf, zm) = (WALL + d, WALL + d / 2.0);
    let t = 0.022;
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            "side",
            t,
            h,
            d,
            (s * (w / 2.0 - t / 2.0), h / 2.0, zm),
            M.oak,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    fbox!(
        F,
        "top",
        w,
        0.03,
        d,
        (0, h - 0.015, zm),
        M.oak,
        bevel = 0.006,
        bevel_segments = 1
    );
    fbox!(
        F,
        "top_bead",
        w - 2.0 * t,
        0.014,
        0.012,
        (0, h - 0.037, zf - 0.007),
        M.oak,
        bevel = 0.005,
        bevel_segments = 2
    );
    fbox!(
        F,
        "kick",
        w - 2.0 * t,
        0.04,
        d - 0.03,
        (0, 0.02, zm - 0.015),
        M.oak
    );
    fbox!(
        F,
        "bottom",
        w - 2.0 * t,
        t,
        d,
        (0, 0.04 + t / 2.0, zm),
        M.oak,
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        F,
        "back",
        w - 2.0 * t,
        h - 0.09,
        0.01,
        (0, 0.06 + (h - 0.09) / 2.0, WALL + 0.005),
        M.oak_back
    );
    fbox!(
        F,
        "divider",
        t,
        h - 0.09,
        d - 0.01,
        (0, 0.06 + (h - 0.09) / 2.0, zm + 0.005),
        M.oak
    );
    fbox!(
        F,
        "counter",
        w,
        0.03,
        d + 0.025,
        (0, 0.8, zm + 0.0125),
        M.oak,
        bevel = 0.004,
        bevel_segments = 1
    );
    for x in [-0.69, -0.23, 0.23, 0.69] {
        fbox!(
            F,
            "door",
            0.445,
            0.72,
            0.02,
            (x, 0.425, zf + 0.01),
            M.oak,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "door_panel",
            0.36,
            0.6,
            0.008,
            (x, 0.425, zf + 0.022),
            M.oak,
            bevel = 0.005,
            bevel_segments = 1
        );
        let kx = x + if x == -0.69 || x == 0.23 { 0.18 } else { -0.18 };
        knob("knob", kx, 0.66, zf + 0.026, M.brass);
    }
    for x0 in [-0.919, 0.011] {
        for y in [1.2, 1.6] {
            fbox!(
                F,
                "shelf",
                0.908,
                0.022,
                d - 0.012,
                (x0 + 0.454, y, zm + 0.004),
                M.oak,
                bevel = 0.002,
                bevel_segments = 1
            );
            fbox!(
                F,
                "shelf_lip",
                0.908,
                0.032,
                0.01,
                (x0 + 0.454, y - 0.005, zf - 0.007),
                M.oak,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
    }
    let front = zf - 0.03;
    book_row(-0.9, -0.05, 0.815, front, 0.26, 0.3, 21, 1.0, false, None);
    book_row(0.03, 0.6, 0.815, front, 0.26, 0.3, 22, 1.0, true, None);
    book_row(-0.9, -0.3, 1.211, front, 0.26, 0.3, 23, 1.0, false, None);
    let out = book_row(0.03, 0.9, 1.211, front, 0.26, 0.3, 24, 1.0, false, Some(5));
    book_row(-0.9, -0.05, 1.611, front, 0.26, 0.3, 25, 0.9, false, None);
    vase(-0.16, 1.211, zm, 0.06, 0.22, M.vase);
    plant(0.76, 0.815, zm, 0.06, 0.11, M.pot_white, 0.1);
    plant(0.45, 1.611, zm, 0.07, 0.12, M.clay, 0.12);
    frame_photo(0.75, 1.611, WALL + 0.06, M.walnut_warm, 0.16, 0.2);
    tips_out("bookcaseWide", out)
}
