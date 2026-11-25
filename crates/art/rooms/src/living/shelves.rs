//! L5, the shelves. Each stands with its back on the wall's face (depth -0.43) and its tops at the content `surface`
//! heights (the low bookcase 0.9, the cube shelf 0.8, the industrial shelving 1.83), built in place and settled there
//! (`centre` on its Model). The books are many small parts, joined into the still body; on the bookcases one of them
//! is a part of its own that tips out 25° from its shelf while a resident reads, studies or does homework there
//! (`book_out`), and back at the end.

use super::common::{
    BOARD, PAGES_IN, SPINE, SPINE_STEPS, book, bulge, disc, hardback, hardback_end, knob, mesh,
    prism_faces,
};
use super::{F, M};
use artkit::forms::{Holes, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{Axis, PI, TAU, cos, radians, sin, tan};
use artkit::parts::{hinged, join, new_parts, turn};
use artkit::place::Facing;
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsphere, fsweep, jitter, settle, swing};

pub(super) use artkit::kit::WALL;
pub(super) const BOOK_TIP: f64 = 25.0;

/// The covers the books are drawn from, by chance.
pub(super) fn books() -> [Mat; 7] {
    [
        M.book_a, M.book_b, M.book_c, M.book_d, M.book_e, M.book_f, M.book_g,
    ]
}

/// A book built apart: its parts and its hinge, the foot of its spine.
pub(super) type Out = (Vec<Obj>, (f64, f64, f64));

/// A row of books on a shelf at height `y`, from x0 towards x1 (game), spines at `front`; the same books for the
/// same `seed`. Now and then a gap, and (`stack`) two or three lying flat at the end. `out`: the index of a book
/// built apart, returned as (its parts, its hinge: the foot of its spine) for `book_out`.
#[allow(clippy::too_many_arguments)]
pub(super) fn book_row(
    x0: f64,
    x1: f64,
    y: f64,
    front: f64,
    depth: f64,
    hmax: f64,
    seed: i128,
    fill: f64,
    stack: bool,
    out: Option<usize>,
) -> Option<Out> {
    shelf_row(x0, x1, y, front, depth, hmax, seed, fill, stack, out, None)
}

/// `book_row` of hardbacks (`hardback`: rounded spines, the page block set in), the same books for the same `seed`,
/// some pushed further back than others; where a row ends short of its shelf, `room` (no stack), one more leans on
/// the last.
#[allow(clippy::too_many_arguments)]
pub(super) fn hardback_row(
    x0: f64,
    x1: f64,
    y: f64,
    front: f64,
    depth: f64,
    hmax: f64,
    seed: i128,
    fill: f64,
    stack: bool,
    out: Option<usize>,
    room: f64,
) -> Option<Out> {
    shelf_row(
        x0,
        x1,
        y,
        front,
        depth,
        hmax,
        seed,
        fill,
        stack,
        out,
        Some(room),
    )
}

#[allow(clippy::too_many_arguments)]
fn shelf_row(
    x0: f64,
    x1: f64,
    y: f64,
    front: f64,
    depth: f64,
    hmax: f64,
    seed: i128,
    fill: f64,
    stack: bool,
    out: Option<usize>,
    room: Option<f64>,
) -> Option<Out> {
    let fine = room.is_some();
    let one = if fine { hardback } else { book };
    let mut rnd = Random::new(seed);
    // (the hardbacks' own variation draws from a generator of its own, so the books are the plain row's)
    let mut more = Random::new(seed * 7 + 3);
    let end = x0 + (x1 - x0) * fill - if stack { 0.24 } else { 0.0 };
    let (mut x, mut k, mut got, mut last) = (x0, 0, None, 0.0);
    loop {
        let (t, h, d) = (
            rnd.uniform(0.022, 0.044),
            hmax * rnd.uniform(0.74, 1.0),
            depth * rnd.uniform(0.86, 1.0),
        );
        let (mut cover, band, gap) = (
            rnd.choice(&books()),
            rnd.random() < 0.35,
            rnd.random() < 0.07,
        );
        if x + t > end {
            break;
        }
        if gap && Some(k) != out && k != 0 {
            x += rnd.uniform(0.03, 0.06);
            continue;
        }
        let front = if fine && Some(k) != out {
            front - more.uniform(0.0, 0.012)
        } else {
            front
        };
        if Some(k) == out {
            cover = M.book_a; // a red one, to be seen
            let parts = new_parts(|| one(x + t / 2.0, y, front, t, h, d, cover, band));
            got = Some((parts, (x + t / 2.0, y, front)));
        } else {
            one(x + t / 2.0, y, front, t, h, d, cover, band);
        }
        last = h;
        x += t + 0.0015;
        k += 1;
    }
    if fine && !stack && k > 0 {
        // one leaning on the last, resting on its foot's inner corner, if it fits on the shelf
        let (t, h, d) = (
            rnd.uniform(0.024, 0.04),
            hmax * rnd.uniform(0.8, 0.95),
            depth * rnd.uniform(0.86, 1.0),
        );
        let (cover, lean) = (rnd.choice(&books()), rnd.uniform(14.0, 22.0));
        let front = front - more.uniform(0.0, 0.01);
        let a = radians(lean);
        let px = x + last.min(h * cos(a)) * tan(a);
        if px + t * cos(a) < room.unwrap_or(x1) - 0.01 {
            let parts = new_parts(|| hardback(px + t / 2.0, y, front, t, h, d, cover, false));
            turn(
                join(&parts, "leaning", Some(F.at(px, y, front))),
                lean,
                Axis::Y,
            );
        }
    }
    if stack && fine {
        let (mut top, cx) = (y, if x0 > x { x0 } else { x } + 0.13);
        for _ in 0..rnd.randint(2, 3) {
            let (w, t, d) = (
                rnd.uniform(0.16, 0.21),
                rnd.uniform(0.024, 0.04),
                depth * rnd.uniform(0.8, 0.95),
            );
            let at = (cx + rnd.uniform(-0.01, 0.01), top + t / 2.0, front);
            let cover = rnd.choice(&books());
            let turned = rnd.uniform(-5.0, 5.0);
            let parts = new_parts(|| lying(at, w, t, d, cover));
            turn(
                join(&parts, "lying", Some(F.at(at.0, at.1, front - d / 2.0))),
                turned,
                Axis::Z,
            );
            top += t;
        }
    } else if stack {
        let (mut top, cx) = (y, if x0 > x { x0 } else { x } + 0.13);
        for _ in 0..rnd.randint(2, 3) {
            let (w, t, d) = (
                rnd.uniform(0.16, 0.21),
                rnd.uniform(0.024, 0.04),
                depth * rnd.uniform(0.8, 0.95),
            );
            let at = (
                cx + rnd.uniform(-0.01, 0.01),
                top + t / 2.0,
                front - d / 2.0,
            );
            let cover = rnd.choice(&books());
            fbox!(
                F,
                "lying",
                w,
                t,
                d,
                at,
                cover,
                rot = (0.0, 0.0, rnd.uniform(-5.0, 5.0))
            );
            top += t;
        }
    }
    got
}

/// A hardback lying on its side, `w` long, `t` thick and `d` deep, its middle at (x, y) and its spine's crown at
/// depth `z`, facing the front.
fn lying(at: (f64, f64, f64), w: f64, t: f64, d: f64, cover: Mat) {
    let (x, y, z) = at;
    let bu = bulge(t);
    // (the end's (across, out) as the side's (depth, height), turned back to counter-clockwise)
    let mut end: Vec<(f64, f64)> = hardback_end(t, d, bu, BOARD, SPINE, SPINE_STEPS)
        .into_iter()
        .map(|(a, o)| (o, a))
        .collect();
    end.reverse();
    fextrude!(
        F,
        "book",
        &end,
        w,
        (x, y, z),
        cover,
        plane = Facing::Side,
        smooth = true
    );
    let pd = d - bu - SPINE - PAGES_IN;
    fbox!(
        F,
        "pages",
        w - 2.0 * PAGES_IN,
        t - 2.0 * BOARD - 0.0006,
        pd,
        (x, y, z - bu - SPINE - pd / 2.0),
        M.pages
    );
}

/// A vase standing on `y`: a foot, a belly, a neck flaring to a lip, open at the mouth.
#[allow(clippy::too_many_arguments)]
pub(super) fn vase(x: f64, y: f64, z: f64, r: f64, h: f64, mat: Mat) -> Obj {
    disc(
        "vase",
        &[
            (y, r * 0.5),
            (y + h * 0.015, r * 0.6),
            (y + h * 0.12, r * 0.82),
            (y + h * 0.38, r),
            (y + h * 0.62, r * 0.8),
            (y + h * 0.8, r * 0.48),
            (y + h * 0.92, r * 0.5),
            (y + h, r * 0.62),
            (y + h, r * 0.5),
            (y + h * 0.9, r * 0.38),
        ],
        mat,
        x,
        z,
        16,
    )
}

/// A pot of `r` and `h` on `y`, its rim rolled, with a bushy plant in it (plain satin leaves): clumps of foliage,
/// lumpy, and a few long leaves reaching out of them.
#[allow(clippy::too_many_arguments)]
pub(super) fn plant(x: f64, y: f64, z: f64, r: f64, h: f64, pot: Mat, leaves: f64) {
    disc(
        "pot",
        &[
            (y, r * 0.74),
            (y + h * 0.04, r * 0.8),
            (y + h * 0.84, r * 0.97),
            (y + h * 0.88, r * 1.04),
            (y + h, r * 1.04),
            (y + h, r * 0.9),
            (y + h * 0.86, r * 0.88),
        ],
        pot,
        x,
        z,
        16,
    );
    let clumps = [
        (0.0, 0.55, 0.0, 1.0),
        (0.55, 0.3, 0.2, 0.7),
        (-0.5, 0.35, -0.15, 0.75),
        (0.1, 0.25, -0.5, 0.6),
        (-0.15, 0.3, 0.5, 0.62),
    ];
    for (i, (dx, dy, dz, s)) in clumps.into_iter().enumerate() {
        let clump = fsphere!(
            F,
            "leaves",
            leaves * s,
            (x + dx * leaves, y + h + dy * leaves, z + dz * leaves),
            if i % 2 == 0 { M.plant } else { M.leaf_light },
            segments = 10,
            rings = 6
        );
        jitter!(
            clump,
            leaves * 0.12,
            size = leaves * 0.35,
            seed = x * 7.0 + y * 3.0 + i as f64,
            up = 0.0
        );
    }
    for i in 0..5 {
        let a = TAU * (f64::from(i) + 0.3) / 5.0;
        let (c, sn) = (cos(a), sin(a));
        fsweep!(
            F,
            "leaf",
            &[
                (x + c * r * 0.3, y + h * 0.9, z + sn * r * 0.3),
                (
                    x + c * leaves * 0.9,
                    y + h + leaves * 0.75,
                    z + sn * leaves * 0.9
                ),
                (
                    x + c * leaves * 1.35,
                    y + h + leaves * 0.55,
                    z + sn * leaves * 1.35
                )
            ],
            0.0,
            if i % 2 != 0 { M.leaf_light } else { M.plant },
            profile = [(0.0, -0.016), (0.003, 0.0), (0.0, 0.016), (-0.002, 0.0)],
            up = (0, 0, 1),
            angle = 180.0, // a leaf: thin, its margins shade round
            resolution = 2,
            scale = [0.3, 1.0, 0.2]
        );
    }
}

/// A picture frame standing on `y`, leaning back against what is behind it: a moulded frame round a photo.
#[allow(clippy::too_many_arguments)]
pub(super) fn frame_photo(x: f64, y: f64, z: f64, mat: Mat, w: f64, h: f64) {
    let lean = -8;
    fextrude!(
        F,
        "photo_frame",
        &rounded_rect(w, h, 0.004, 1),
        0.02,
        (x, y + h / 2.0, z),
        mat,
        rot = (lean, 0, 0),
        holes = Holes(vec![rounded_rect(w - 0.04, h - 0.04, 0.002, 1)]),
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        "photo",
        w - 0.036,
        h - 0.036,
        0.004,
        (x, y + h / 2.0, z - 0.002),
        M.photo,
        rot = (lean, 0, 0)
    );
}

/// The game outline `outline` [(p, q)] extruded: (x, depth) points from height a0 to a1 (`along_x` false); (depth,
/// height) points from x = a0 to a1 (a ladder's rail).
pub(super) fn prism(
    name: &str,
    outline: &[(f64, f64)],
    a0: f64,
    a1: f64,
    material: Mat,
    along_x: bool,
) -> Obj {
    let mut verts: Vec<(f64, f64, f64)> = Vec::new();
    for a in [a0, a1] {
        verts.extend(
            outline
                .iter()
                .map(|&(p, q)| if along_x { (a, q, p) } else { (p, a, q) }),
        );
    }
    mesh(name, &verts, &prism_faces(outline.len()), material)
}

/// A quarter-round board from y0 to y1: its corner on (cx, cz), round from +x (along the wall) to the front.
#[allow(clippy::too_many_arguments)]
pub(super) fn quarter(
    name: &str,
    cx: f64,
    cz: f64,
    r: f64,
    y0: f64,
    y1: f64,
    material: Mat,
) -> Obj {
    let segments = 10;
    let mut pts = vec![(cx, cz)];
    pts.extend((0..=segments).map(|i| {
        let a = PI / 2.0 * f64::from(i) / f64::from(segments);
        (cx + r * cos(a), cz + r * sin(a))
    }));
    prism(name, &pts, y0, y1, material, false)
}

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
            bevel = 0.006
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
        bevel = 0.003
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
            bevel = 0.003
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
            bevel = 0.003
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
        bevel_segments = 2
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
        bevel = 0.002
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
        bevel = 0.004
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
            bevel = 0.006
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
            bevel_segments = 2
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
                bevel = 0.002
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
