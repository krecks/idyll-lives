//! KD3: the picture-book shelf.

use super::common::{P, board, place, rect, slab, star, still};
use super::{F, M};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::materials::Mat;
use artkit::math::{Axis, cos, radians, sin};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, swing};

/// (height, front lip's depth)
const SHELVES: [(f64, f64); 3] = [(0.12, -0.1), (0.40, -0.17), (0.68, -0.24)];
const BOOK_TILT: f64 = 14.0;
/// Half a book's height: the top row's tops clear the top board.
const BOOK_H: f64 = 0.115;

/// A picture book stood face out, leaning back; its middle at (x, y, z): two boards round a block of pages that
/// shows white along its top, a picture and a title band on its front.
fn book(name: &str, x: f64, y: f64, z: f64, cover: Mat, picture: Mat) {
    let frame = board((x, y, z), BOOK_TILT);
    slab(
        name,
        &rect(-0.09, -BOOK_H, 0.09, BOOK_H),
        &frame,
        cover,
        0.006,
        0.004,
    );
    slab(
        &format!("{name}_back"),
        &rect(-0.09, -BOOK_H, 0.09, BOOK_H),
        &frame,
        cover,
        -0.01,
        0.004,
    );
    slab(
        &format!("{name}_spine"),
        &rect(-0.093, -BOOK_H, -0.087, BOOK_H),
        &frame,
        cover,
        -0.01,
        0.02,
    );
    slab(
        &format!("{name}_pages"),
        &rect(-0.087, 0.003 - BOOK_H, 0.086, BOOK_H - 0.003),
        &frame,
        M.paper,
        -0.006,
        0.012,
    );
    slab(
        &format!("{name}_pic"),
        &rect(-0.055, -0.03, 0.055, 0.075),
        &frame,
        picture,
        0.01,
        0.002,
    );
    slab(
        &format!("{name}_title"),
        &rect(-0.06, -0.085, 0.06, -0.06),
        &frame,
        M.paper,
        0.01,
        0.002,
    );
}

pub fn kids_bookshelf() -> Vec<Obj> {
    // the sides: a curve from the foot up to the top, set back as it rises, a star cut through each
    let smooth = smooth_outline(
        &[
            (-0.46, 0.0),
            (-0.06, 0.0),
            (-0.06, 0.2),
            (-0.15, 0.62),
            (-0.235, 0.9),
            (-0.46, 0.9),
        ],
        2,
        true,
    );
    let mut side = vec![(-0.46, 0.0), (-0.06, 0.0)];
    side.extend(smooth.into_iter().filter(|p| p.1 > 0.02 && p.0 > -0.44));
    side.push((-0.46, 0.9));
    for s in [-1, 1] {
        fextrude!(
            F,
            format!("side{s}"),
            &side,
            0.025,
            (s as f64 * 0.3725, 0, 0),
            M.kshelf,
            plane = Facing::Side,
            holes = vec![star(-0.34, 0.78, 0.045, 0.45, 5, 90.0)],
            bevel = 0.006,
            bevel_segments = 1
        );
    }
    fbox!(
        F,
        "back",
        0.72,
        0.9,
        0.02,
        (0, 0.45, -0.45),
        M.kshelf,
        bevel = 0.004,
        bevel_segments = 1
    );
    fextrude!(
        F,
        "top",
        &rounded_rect(0.77, 0.26, 0.02, 2),
        0.025,
        (0, 0.9125, -0.33),
        M.kshelf_top,
        plane = Facing::Top,
        bevel = 0.008,
        bevel_segments = 2
    );
    fbox!(
        F,
        "plinth",
        0.7,
        0.06,
        0.02,
        (0, 0.03, -0.07),
        M.kshelf_top,
        bevel = 0.004,
        bevel_segments = 1
    );
    let lips = [M.toy_red, M.toy_blue, M.toy_yellow];
    let covers = [
        (M.book_teal, M.toy_yellow),
        (M.book_rust, M.toy_blue),
        (M.book_blue, M.toy_red),
        (M.book_ochre, M.toy_green),
        (M.book_sage, M.toy_pink),
        (M.book_rose, M.toy_blue),
        (M.book_rust, M.toy_green),
        (M.book_blue, M.toy_yellow),
        (M.book_teal, M.toy_red),
    ];
    let mut out: Option<(Vec<Obj>, P)> = None;
    for (i, (y, z)) in SHELVES.into_iter().enumerate() {
        fbox!(
            F,
            format!("shelf{i}"),
            0.72,
            0.02,
            z + 0.44,
            (0, y, (z - 0.44) / 2.0),
            M.kshelf,
            bevel = 0.003,
            bevel_segments = 1
        );
        fextrude!(
            F,
            format!("lip{i}"),
            &rounded_rect(0.72, 0.08, 0.025, 3),
            0.015,
            (0, y + 0.04, z),
            lips[i],
            bevel = 0.004,
            bevel_segments = 1
        );
        for (j, x) in [-0.22, 0.0, 0.22].into_iter().enumerate() {
            let (cover, picture) = covers[3 * i + j];
            let mut bz = z - 0.04;
            let by = y + 0.01 + BOOK_H * cos(radians(BOOK_TILT));
            bz -= BOOK_H * sin(radians(BOOK_TILT));
            if (i, j) == (1, 1) {
                let parts = new_parts(|| book(&format!("book{i}{j}"), x, by, bz, cover, picture));
                out = Some((parts, (x, y + 0.01, z - 0.03)));
            } else {
                book(&format!("book{i}{j}"), x, by, bz, cover, picture);
            }
        }
    }
    let (book_parts, pivot) = out.expect("the middle shelf's middle book");
    let still_parts: Vec<Obj> = meshes()
        .into_iter()
        .filter(|o| !book_parts.contains(o))
        .collect();
    still("body", Some(&still_parts));
    let book = hinged(
        join(&book_parts, "book", None),
        F.at(pivot.0, pivot.1, pivot.2),
    );
    place(0.03);
    swing!(book, "book_out", 20.0, seconds = 0.6, axis = Axis::X);
    meshes()
}
