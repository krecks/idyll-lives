//! L5: the books, vases, plants and frames that stand on shelves, and the shelving bars and corners.

use super::common::{
    BOARD, PAGES_IN, SPINE, SPINE_STEPS, book, bulge, disc, hardback, hardback_end, mesh,
};
use super::{F, M};
use artkit::forms::{Holes, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{Axis, PI, TAU, cos, radians, sin, tan};
use artkit::parts::{join, new_parts, turn};
use artkit::place::Facing;
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsphere, fsweep, jitter};

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
            segments = 8,
            rings = 5
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
/// height) points from x = a0 to a1 (a ladder's rail); the edges round its two end faces chamfered `c` (the outline
/// set in at each end, all of it `c` in): a board's arrises eased.
pub(super) fn eased_bar(
    name: &str,
    outline: &[(f64, f64)],
    (a0, a1): (f64, f64),
    c: f64,
    material: Mat,
    along_x: bool,
) -> Obj {
    let small = artkit::forms::inset(outline, c);
    let k = outline.len() as u32;
    let mut verts: Vec<(f64, f64, f64)> = Vec::new();
    for (a, o) in [
        (a0, &small),
        (a0 + c, &outline.to_vec()),
        (a1 - c, &outline.to_vec()),
        (a1, &small),
    ] {
        verts.extend(
            o.iter()
                .map(|&(p, q)| if along_x { (a, q, p) } else { (p, a, q) }),
        );
    }
    let mut faces = vec![(0..k).collect::<Vec<u32>>(), (3 * k..4 * k).rev().collect()];
    for r in 0..3 {
        for i in 0..k {
            let (j, at) = ((i + 1) % k, r * k);
            faces.push(vec![at + i, at + j, at + k + j, at + k + i]);
        }
    }
    mesh(name, &verts, &faces, material)
}

/// A quarter-round board from y0 to y1, its edges eased: its corner on (cx, cz), round from +x (along the wall) to the front.
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
    eased_bar(name, &pts, (y0, y1), 0.004, material, false)
}
