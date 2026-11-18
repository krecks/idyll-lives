//! The tall and wall storage (K5): what it is built from, and the tall larders (`pantry`, `pantry_tall`), whose doors
//! swing open (`doors_open`) for a snack. A wall thing sits on the wall face (z = WALL) at its real height:
//! `on_wall`, centred in x and depth.

use super::common::{pull, shaker};
use super::turned::{bowl, can, canister, lathe, plates};
use super::{DOOR_SECONDS, M};
use artkit::materials::Mat;
use artkit::math::P3;
use artkit::motion::Ease;
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::{Coord, GameFrame};
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, loft, ring, sweep, swing};

pub const WALL: f64 = -0.43; // the back face of a wall thing, in game depth (docs/design/objects.md)

/// An open case: a back at depth `zback`, two sides, a top and a bottom, open at the front.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn case(
    f: &GameFrame,
    w: f64,
    h: f64,
    d: f64,
    x: impl Coord,
    y: f64,
    zback: f64,
    material: Mat,
    back: Option<Mat>,
    t: f64,
    p: &str,
) {
    fbox!(
        f,
        format!("{p}_back"),
        w,
        h,
        t,
        (x, y, zback + t / 2.0),
        back.unwrap_or(material),
        bevel = 0.004
    );
    for s in [-1, 1] {
        fbox!(
            f,
            format!("{p}_side{s}"),
            t,
            h,
            d,
            (x.f() + s as f64 * (w / 2.0 - t / 2.0), y, zback + d / 2.0),
            material,
            bevel = 0.004
        );
    }
    fbox!(
        f,
        format!("{p}_top"),
        w - 2.0 * t,
        t,
        d,
        (x, y + h / 2.0 - t / 2.0, zback + d / 2.0),
        material,
        bevel = 0.004
    );
    fbox!(
        f,
        format!("{p}_bottom"),
        w - 2.0 * t,
        t,
        d,
        (x, y - h / 2.0 + t / 2.0, zback + d / 2.0),
        material,
        bevel = 0.004
    );
}

/// A shelf across a case, from the back face `zback`; `lip`, a strip standing up along its front in that material
/// (a larder's, to keep the tins on).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn shelf(
    f: &GameFrame,
    name: &str,
    w: f64,
    y: f64,
    zback: f64,
    d: f64,
    material: Option<Mat>,
    lip: Option<Mat>,
) {
    fbox!(
        f,
        name,
        w,
        0.02,
        d,
        (0, y, zback + d / 2.0),
        material.unwrap_or(M.worktop),
        bevel = 0.003
    );
    if let Some(lip) = lip {
        fbox!(
            f,
            format!("{name}_lip"),
            w,
            0.035,
            0.012,
            (0, y + 0.0075, zback + d - 0.006),
            lip,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
}

/// A round thing turned about its own axis (Blender z before `rot`), its foot on `at` (Blender): for one hung or
/// lying on its side (a pan facing the front).
pub fn lathe_at(
    name: impl AsRef<str>,
    profile: &[(f64, f64)],
    at: P3,
    material: Mat,
    verts: usize,
    rot: (i32, i32, i32),
) -> Obj {
    let sections: Vec<Vec<P3>> = profile
        .iter()
        .map(|&(r, h)| ring!(py::max(r, 1e-5), py::max(r, 1e-5), z = h, verts = verts))
        .collect();
    loft!(name, &sections, material = material, at = at, rot = rot)
}

/// A glass jar standing on `y` at depth `z`: its body rounding into a shoulder and neck, what is in it, and a screw
/// lid.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn jar(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    colour: Mat,
    h: f64,
    r: f64,
    lid: bool,
    lidmat: Option<Mat>,
    verts: usize,
) {
    lathe(
        f,
        format!("{name}_glass"),
        &[
            (0.0, 0.0),
            (r, 0.0),
            (r, 0.78 * h),
            (0.8 * r, 0.9 * h),
            (0.0, 0.9 * h),
        ],
        x,
        y,
        z,
        M.glass,
        verts,
    );
    fcyl!(
        f,
        format!("{name}_in"),
        r * 0.86,
        h * 0.6,
        (x, y + h * 0.32, z),
        colour,
        verts = verts
    );
    if lid {
        lathe(
            f,
            format!("{name}_lid"),
            &[
                (0.0, 0.9 * h),
                (0.86 * r, 0.9 * h),
                (0.86 * r, h + 0.012),
                (0.0, h + 0.012),
            ],
            x,
            y,
            z,
            lidmat.unwrap_or(M.chrome),
            verts,
        );
    }
}

/// A woven basket sitting on `y`: a rounded body narrowing to its foot, ribbed with the weave, hollow, a rolled rim
/// round its top.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn basket(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    x: f64,
    y: f64,
    z: impl Coord,
    material: Option<Mat>,
    verts: usize,
) {
    let material = material.unwrap_or(M.basket);
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = f.at(x, y, z);
    let mut secs: Vec<Vec<P3>> = Vec::new();
    for i in 0..7 {
        let t = i as f64 / 6.0;
        let k = 0.88 + 0.12 * t;
        let bump = if i % 2 != 0 { 0.005 } else { 0.0 };
        secs.push(ring!(
            w / 2.0 * k + bump,
            d / 2.0 * k + bump,
            z = bz + h * t,
            verts = verts,
            square = 5,
            at = (bx, by)
        ));
    }
    secs.push(ring!(
        w / 2.0 - 0.012,
        d / 2.0 - 0.012,
        z = bz + h,
        verts = verts,
        square = 5,
        at = (bx, by)
    ));
    secs.push(ring!(
        w / 2.0 - 0.016,
        d / 2.0 - 0.016,
        z = bz + 0.03,
        verts = verts,
        square = 5,
        at = (bx, by)
    ));
    loft!(name, &secs, material = material);
    let rim: Vec<P3> = ring!(
        w / 2.0 + 0.002,
        d / 2.0 + 0.002,
        z = 0,
        verts = verts,
        square = 5,
        at = (bx, by)
    )
    .into_iter()
    .map(|p| P3 {
        x: p.x,
        y: p.y,
        z: bz + h,
    })
    .collect();
    sweep!(
        format!("{name}_rim"),
        &rim,
        0.008,
        material = material,
        verts = 6,
        closed = true
    );
}

/// A cornice capping a tall unit, `t` thick from `y` up: a fillet under a moulded cap with a rounded nosing.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn cornice(f: &GameFrame, w: f64, d: f64, y: f64, z: f64, material: Mat, t: f64) {
    fbox!(
        f,
        "cornice_fillet",
        w - 0.016,
        t * 0.35,
        d - 0.008,
        (0, y + t * 0.175, z - 0.004),
        material,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        f,
        "cornice",
        w,
        t * 0.65,
        d,
        (0, y + t * 0.675, z),
        material,
        bevel = t * 0.25,
        bevel_segments = 3
    );
}

/// A packet of food standing on `y`: a box with a label band round it.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn box_food(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    x: f64,
    y: f64,
    z: f64,
    material: Mat,
    label: Option<Mat>,
) {
    fbox!(
        f,
        name,
        w,
        h,
        d,
        (x, y + h / 2.0, z),
        material,
        bevel = 0.004,
        bevel_segments = 1
    );
    if let Some(label) = label {
        fbox!(
            f,
            format!("{name}_label"),
            w + 0.002,
            h * 0.3,
            d + 0.002,
            (x, y + h * 0.55, z),
            label
        );
    }
}

/// Food on a larder's shelves (their tops `tops`, bottom up), at depth `z`: tins, canisters, packets, bowls.
fn larder_stock(f: &GameFrame, tops: (f64, f64, f64, f64), z: f64) {
    let (a, b, c, d) = tops;
    for (i, (x, m)) in [(-0.31, M.rust), (-0.21, M.cream), (-0.11, M.teal)]
        .into_iter()
        .enumerate()
    {
        can(f, format!("can{i}"), x, a, z, 0.042, 0.11, m, 8);
    }
    can(f, "can3", -0.21, a + 0.11, z, 0.042, 0.11, M.rust, 8);
    box_food(
        f,
        "cereal0",
        0.17,
        0.27,
        0.07,
        0.1,
        a,
        z - 0.04,
        M.yellow,
        Some(M.porcelain),
    );
    box_food(
        f,
        "cereal1",
        0.15,
        0.24,
        0.065,
        0.28,
        a,
        z - 0.03,
        M.sage,
        Some(M.cream),
    );
    for (i, (x, r, h)) in [
        (-0.29, 0.06, 0.2),
        (-0.15, 0.055, 0.16),
        (-0.03, 0.05, 0.12),
    ]
    .into_iter()
    .enumerate()
    {
        canister(
            f,
            &format!("canister{i}"),
            x,
            b,
            z,
            r,
            h,
            M.porcelain,
            M.spoon,
            12,
        );
    }
    box_food(
        f,
        "packet0",
        0.12,
        0.18,
        0.08,
        0.15,
        b,
        z,
        M.rust,
        Some(M.porcelain),
    );
    box_food(
        f,
        "packet1",
        0.12,
        0.15,
        0.08,
        0.29,
        b,
        z + 0.02,
        M.teal,
        Some(M.porcelain),
    );
    for i in 0..3 {
        bowl(
            f,
            format!("bowl{i}"),
            -0.24,
            c + 0.02 * i as f64,
            z,
            0.09 - 0.008 * i as f64,
            0.075 - 0.006 * i as f64,
            M.cream,
            10,
        );
    }
    lathe(
        f,
        "casserole",
        &[
            (0.0, 0.0),
            (0.09, 0.0),
            (0.095, 0.01),
            (0.095, 0.09),
            (0.0, 0.09),
        ],
        0.05,
        c,
        z,
        M.rust,
        12,
    );
    lathe(
        f,
        "casserole_lid",
        &[
            (0.0, 0.088),
            (0.098, 0.088),
            (0.09, 0.1),
            (0.04, 0.115),
            (0.012, 0.117),
            (0.016, 0.13),
            (0.0, 0.132),
        ],
        0.05,
        c,
        z,
        M.rust,
        12,
    );
    can(f, "can4", 0.27, c, z, 0.045, 0.12, M.yellow, 8);
    box_food(
        f,
        "packet2",
        0.2,
        0.18,
        0.07,
        -0.2,
        d,
        z - 0.03,
        M.cream,
        Some(M.rust),
    );
    box_food(
        f,
        "packet3",
        0.16,
        0.16,
        0.07,
        0.02,
        d,
        z - 0.02,
        M.porcelain,
        Some(M.teal),
    );
    plates(f, "platter", 0.24, d, z, 0.11, 3, M.porcelain, 0.013, 12);
}

/// The two shaker doors of a larder, `w` wide each, on the case front at depth `front`, a middle rail at `mid`, a
/// long pull on each meeting stile; each its own part hinged on its outer edge.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn tall_doors(
    f: &GameFrame,
    w: f64,
    h: f64,
    y: f64,
    front: f64,
    mid: f64,
    pull_y: f64,
    pull_len: f64,
    hm: Mat,
) -> Vec<Obj> {
    let mut doors = Vec::new();
    for s in [-1, 1] {
        let parts = new_parts(|| {
            shaker(
                f,
                &format!("door{s}_front"),
                w - 0.002,
                h,
                s as f64 * (w / 2.0 + 0.001),
                y,
                front - 0.01,
                M.cab,
                0.06,
                0.02,
                Some(mid),
            );
            pull(
                f,
                format!("door{s}_pull"),
                s as f64 * 0.034,
                pull_y,
                front + 0.01,
                pull_len,
                Some(hm),
                true,
                0.019,
                0.005,
                0.0,
            );
        });
        doors.push(join(
            &parts,
            &format!("door{s}"),
            Some(f.at(s as f64 * w, y, front)),
        ));
    }
    doors
}

// --- the larder: 1x1, a tall cupboard with two doors that swing open for a snack -----------------------
pub fn pantry() -> Vec<Obj> {
    let f = &GameFrame::new(0.141);
    let still = new_parts(|| {
        case(
            f, 0.92, 1.92, 0.6, 0, 1.06, -0.46, M.cab, None, 0.02, "case",
        );
        fbox!(
            f,
            "plinth",
            0.88,
            0.1,
            0.56,
            (0, 0.05, -0.18),
            M.dark,
            bevel = 0.004
        );
        cornice(f, 0.94, 0.62, 2.02, -0.15, M.cab, 0.04);
        for (i, y) in [0.5, 0.88, 1.26, 1.64].into_iter().enumerate() {
            shelf(
                f,
                &format!("shelf{i}"),
                0.86,
                y,
                -0.44,
                0.56,
                None,
                Some(M.cab),
            );
        }
        larder_stock(f, (0.51, 0.89, 1.27, 1.65), -0.17);
        basket(
            f,
            "floor_basket",
            0.34,
            0.16,
            0.3,
            -0.18,
            0.12,
            -0.18,
            None,
            16,
        );
        basket(f, "basket", 0.36, 0.2, 0.3, -0.2, 2.06, -0.2, None, 16);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let doors = tall_doors(f, 0.456, 1.88, 1.06, 0.14, 1.0, 1.05, 0.3, M.chrome);
    settle_all();
    swing!(
        doors[0],
        "doors_open",
        100,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(
        doors[1],
        "doors_open",
        -100,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    meshes()
}

// --- the tall larder: 1x1, two doors over two drawers, a basket on top ---------------------------------
pub fn pantry_tall() -> Vec<Obj> {
    let f = &GameFrame::new(0.104);
    let still = new_parts(|| {
        case(
            f, 0.92, 2.06, 0.66, 0, 1.08, -0.45, M.cab, None, 0.02, "case",
        );
        fbox!(
            f,
            "plinth",
            0.9,
            0.1,
            0.6,
            (0, 0.05, -0.14),
            M.dark,
            bevel = 0.004
        );
        cornice(f, 0.96, 0.7, 2.11, -0.11, M.cab, 0.05);
        for (i, y) in [0.24, 0.48].into_iter().enumerate() {
            shaker(
                f,
                &format!("drawer{i}"),
                0.9,
                0.22,
                0,
                y,
                0.2,
                M.cab,
                0.045,
                0.02,
                None,
            );
            pull(
                f,
                format!("drawer{i}_pull"),
                0,
                y,
                0.22,
                0.3,
                Some(M.brass),
                false,
                0.019,
                0.005,
                0.008,
            );
        }
        for (i, y) in [0.74, 1.12, 1.5, 1.88].into_iter().enumerate() {
            shelf(
                f,
                &format!("shelf{i}"),
                0.86,
                y,
                -0.43,
                0.62,
                None,
                Some(M.cab),
            );
        }
        larder_stock(f, (0.75, 1.13, 1.51, 1.89), -0.14);
        basket(f, "basket", 0.3, 0.22, 0.26, -0.22, 2.16, -0.16, None, 16);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let doors = tall_doors(f, 0.453, 1.44, 1.34, 0.21, 1.36, 0.8, 0.3, M.brass);
    settle_all();
    swing!(
        doors[0],
        "doors_open",
        100,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(
        doors[1],
        "doors_open",
        -100,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    meshes()
}
