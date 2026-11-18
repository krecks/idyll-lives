//! What the kind pack is built from: its palette, turned and lofted pieces, fluted rings, the tea set, little
//! shrubs, and the kit's parts it uses.

use artkit::materials::Mat;
use artkit::math::{P3, TAU, cos, cosd, sin, sind};
use artkit::place::{GameFrame, Spot};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{boulder, fsphere, fsweep, jitter, loft, palette, revolve, ring, torus};
use std::cell::Cell;

palette! { M:
    body = ("#6F8F6A", fabric), // the settee's sage velvet; its main part
    enamel = ("#F1E6C8", gloss), // the range's buttercream enamel; the counter's main part
    cabinet = ("#A8BFA0", satin), // the counter's sage cupboards
    cabinet_panel = ("#B9CDB2", satin),
    butcher = ("#C79A62", satin), // the counter's top and the wall shelf, plain (the towel takes the textured draw)
    towel = ("#E3A0A0", fabric), // the gingham tea towel on the range's rail
    towel_check = ("#FBF6EC", fabric),
    hob = ("#4A4E54", satin),
    grate = ("#2B2C2E", satin),
    oven_dark = ("#2A2622", satin), // inside the oven
    oven_glass = ("#4A3A30", gloss),
    pedestal = ("#C79A62", satin), // the giving jar's turned pedestal, plain (the jar takes the glass draw)
    glass = ("#EEF4F4", glass),
    cork = ("#B5835A", satin),
    ribbon = ("#E3A0A0", satin),
    rocker = ("#8A5A36", wood), // the rocking chair
    cushion = ("#A8BFA0", fabric), // the rocker's seat cushion, the poufs' tops
    throw = ("#F1E6C8", fabric),
    wicker = ("#C9A16A", satin),
    table = ("#C79A62", satin), // the tea table, plain (the poufs and rug take the fabric draw)
    pouf = ("#A8BFA0", fabric), // the left pouf
    pouf_rose = ("#E3A0A0", fabric), // the right pouf
    rug = ("#E8D6C0", fabric),
    rug_edge = ("#C0714C", fabric),
    feeder = ("#9CC3DB", satin), // the feeder house; the station's main part
    roof = ("#C0714C", satin),
    post = ("#F2EFE8", satin),
    tray = ("#C79A62", satin),
    stone = ("#D8D3C8", matte), // the bird bath
    sack = ("#C9B48C", satin),
    quilt = ("#F6EEDC", fabric), // the settee's quilt
    honey = ("#C79A62", satin), // turned legs, the trolley
    porcelain = ("#FBF6EC", gloss),
    rose = ("#E3A0A0", gloss), // tea set bands, the heart tag
    heart = ("#D9483B", gloss),
    brass = ("#C9A54A", metal),
    chrome = ("#D0D3D6", chrome),
    gold = ("#D9B45A", metal), // coins
    note = ("#7FA35B", matte),
    chalk = ("#3E5A48", matte),
    chalk_line = ("#F4F1EA", matte),
    terracotta = ("#C0714C", satin),
    pot = ("#F4F1EA", satin),
    leaf = ("#5E8F4E", satin),
    leaf_light = ("#7FA35B", satin),
    mustard = ("#D6A64A", satin),
    rose_flower = ("#E3A0A0", satin),
    lilac = ("#B48CD6", satin),
    sky = ("#9CC3DB", satin),
    crust = ("#E0B060", matte),
    crust_dark = ("#C98B3A", matte),
    icing = ("#F4C6D0", matte),
    flour = ("#F4EFE2", matte),
    jar_red = ("#C77D4F", gloss),
    jar_gold = ("#E8C46E", gloss),
    jar_white = ("#F4EFE2", gloss),
    sage_pot = ("#A8BFA0", gloss),
    sage_dark = ("#7E9A76", gloss),
    candle = ("#FBF6EC", satin),
    flame = ("#FFD27A", gloss),
    water = ("#BFE3EE", gloss),
    seed = ("#C9A16A", matte),
    tin = ("#B9BEC3", metal),
    needle = ("#C9CCD0", metal),
    bird_brown = ("#8A6A4E", satin),
    bird_orange = ("#E07A2E", satin),
    bird_blue = ("#4F7FB8", satin),
    bird_cream = ("#F4EFE2", satin),
    bird_gold = ("#C8A040", satin),
    bird_yellow = ("#F2D27A", satin),
    bird_grey = ("#9AA0A6", satin),
    bird_pink = ("#E8B4B8", satin),
    dark = ("#3A3D41", satin),
    lantern = ("#FFD9A0", gloss),
    patch_a = ("#E3A0A0", fabric),
    patch_b = ("#D6A64A", fabric),
    patch_c = ("#9CC3DB", fabric),
    patch_d = ("#A8BFA0", fabric),
    pillow_a = ("#E3A0A0", fabric),
    pillow_b = ("#F1E6C8", fabric),
    soil = ("#5C4433", matte), // the potted plants' soil
}

/// The wall's face in a 1-deep footprint, game depth.
pub const WALL: f64 = -0.43;

thread_local! {
    static N: Cell<u32> = const { Cell::new(0) };
}

/// `stem` numbered: each part its own name.
pub fn name(stem: &str) -> String {
    N.with(|n| {
        n.set(n.get() + 1);
        format!("{stem}{}", n.get())
    })
}

/// A turned piece standing at game (x, y, z): `profile` [(radius, height up), ...] from its foot, revolved.
#[allow(clippy::too_many_arguments)]
pub fn turned(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    profile: &[(f64, f64)],
    material: Mat,
    segments: usize,
    rot: (f64, f64, f64),
) -> Obj {
    revolve!(
        name,
        profile,
        at = f.at(x, y, z),
        material = material,
        segments = segments,
        rot = rot
    )
}

/// A body lofted through round rings [(radius, height up), ...] standing at game (x, y, z).
#[allow(clippy::too_many_arguments)]
pub fn lathe(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    rings: &[(f64, f64)],
    material: Mat,
    verts: usize,
    square: f64,
) -> Obj {
    let P3 { x: bx, y: by, .. } = f.at(x, 0, z);
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(r, h)| {
            ring!(
                r,
                r,
                z = y + h,
                verts = verts,
                square = square,
                at = (bx, by)
            )
        })
        .collect();
    loft!(name, &sections, material = material)
}

/// A ring of Blender points of radius `r` at height `y`, in and out `depth` (a fraction) `flutes` times round.
pub fn fluted(
    r: f64,
    y: f64,
    verts: usize,
    at: (f64, f64),
    flutes: i32,
    depth: f64,
    sy: f64,
) -> Vec<P3> {
    (0..verts)
        .map(|k| {
            let (k, v) = (k as f64, verts as f64);
            let flute = 1.0 + depth * cos(flutes as f64 * TAU * k / v);
            P3 {
                x: at.0 + r * flute * cos(TAU * k / v),
                y: at.1 + sy * r * flute * sin(TAU * k / v),
                z: y,
            }
        })
        .collect()
}

/// A teapot standing on `y`, spout to the right (+x game), `s` its scale: a round body with a band, a lid with a
/// knob, a curving spout and a looped handle.
pub fn teapot(f: &GameFrame, x: f64, y: f64, z: f64, s: f64) {
    lathe(
        f,
        &name("pot"),
        x,
        y,
        z,
        &[
            (0.045 * s, 0.0),
            (0.06 * s, 0.008 * s),
            (0.077 * s, 0.035 * s),
            (0.08 * s, 0.06 * s),
            (0.07 * s, 0.092 * s),
            (0.05 * s, 0.108 * s),
            (0.04 * s, 0.11 * s),
        ],
        M.porcelain,
        18,
        2.0,
    );
    turned(
        f,
        &name("pot_lid"),
        x,
        y + 0.108 * s,
        z,
        &[
            (0.0, 0.0),
            (0.042 * s, 0.0),
            (0.042 * s, 0.004 * s),
            (0.03 * s, 0.012 * s),
            (0.008 * s, 0.014 * s),
            (0.0, 0.014 * s),
        ],
        M.porcelain,
        18,
        (0.0, 0.0, 0.0),
    );
    fsphere!(
        f,
        name("pot_knob"),
        0.012 * s,
        (x, y + 0.13 * s, z),
        M.rose,
        scale = (1, 1, 0.85),
        segments = 8,
        rings = 5
    );
    torus!(
        name("pot_band"),
        0.079 * s,
        0.005 * s,
        at = f.at(x, y + 0.055 * s, z),
        material = M.rose,
        major_segments = 18,
        minor_segments = 3
    );
    fsweep!(
        f,
        name("spout"),
        &[
            (x + 0.06 * s, y + 0.035 * s, z),
            (x + 0.1 * s, y + 0.06 * s, z),
            (x + 0.12 * s, y + 0.1 * s, z),
            (x + 0.135 * s, y + 0.115 * s, z)
        ],
        0.016 * s,
        M.porcelain,
        verts = 8,
        resolution = 2,
        scale = [1.0, 0.7, 0.5, 0.45]
    );
    fsweep!(
        f,
        name("pot_handle"),
        &[
            (x - 0.065 * s, y + 0.095 * s, z),
            (x - 0.11 * s, y + 0.1 * s, z),
            (x - 0.125 * s, y + 0.065 * s, z),
            (x - 0.1 * s, y + 0.03 * s, z),
            (x - 0.072 * s, y + 0.025 * s, z)
        ],
        0.008 * s,
        M.porcelain,
        verts = 6,
        resolution = 2
    );
}

/// A cup on its saucer, a rose band round its rim, its handle out at `turn` degrees (0: +x).
pub fn cup(f: &GameFrame, x: f64, y: f64, z: f64, turn: f64) {
    let none = (0.0, 0.0, 0.0);
    turned(
        f,
        &name("saucer"),
        x,
        y,
        z,
        &[
            (0.0, 0.0),
            (0.03, 0.0),
            (0.05, 0.006),
            (0.052, 0.009),
            (0.034, 0.005),
            (0.0, 0.005),
        ],
        M.porcelain,
        16,
        none,
    );
    turned(
        f,
        &name("cup"),
        x,
        y + 0.006,
        z,
        &[
            (0.0, 0.0),
            (0.022, 0.0),
            (0.038, 0.045),
            (0.034, 0.045),
            (0.02, 0.006),
            (0.0, 0.006),
        ],
        M.porcelain,
        14,
        none,
    );
    torus!(
        name("cup_band"),
        0.0365,
        0.003,
        at = f.at(x, y + 0.049, z),
        material = M.rose,
        major_segments = 14,
        minor_segments = 3
    );
    let (c, s) = (cosd(turn), sind(turn));
    let pts: Vec<(f64, f64, f64)> = [(0.034, 0.036), (0.05, 0.034), (0.05, 0.018), (0.03, 0.014)]
        .iter()
        .map(|&(d, h)| (x + c * d, y + 0.006 + h, z + s * d))
        .collect();
    fsweep!(
        f,
        name("cup_handle"),
        &pts,
        0.0035,
        M.porcelain,
        verts = 5,
        resolution = 2
    );
}

/// A little flowering shrub: lumpy satin leaf clumps (no foliage draw) dotted with flowers.
#[allow(clippy::too_many_arguments)]
pub fn bush(f: &GameFrame, x: f64, y: f64, z: f64, r: f64, flower: Mat, seed: i32) {
    let mut rnd = Random::new(seed as i128);
    for (k, (dx, dz, fr)) in [(0.0, 0.0, 1.0), (0.05, 0.03, 0.75), (-0.05, -0.02, 0.8)]
        .into_iter()
        .enumerate()
    {
        let o = boulder!(
            name("clump"),
            r * fr * 0.9,
            at = f.at(x + dx, y, z + dz),
            material = if k != 0 { M.leaf } else { M.leaf_light },
            scale = (1, 1, 0.85),
            seed = seed as f64 + k as f64 * 1.7,
            noise = 0.3,
            subdivisions = 2
        );
        jitter!(o, r * 0.08, size = r * 0.5, seed = (seed + k as i32) as f64);
    }
    for k in 0..4 {
        let a = rnd.uniform(0.0, 360.0);
        fsphere!(
            f,
            name("flower"),
            0.018,
            (
                x + 0.06 * cosd(a),
                y + r * (1.25 + 0.2 * (k % 2) as f64),
                z + 0.05 * sind(a)
            ),
            flower,
            scale = (1, 1, 0.6),
            segments = 7,
            rings = 4
        );
    }
}

/// The kit's tube (`artkit::kit::tube`), as the script called it.
pub fn tube<P: Spot>(
    f: &GameFrame,
    name: &str,
    points: &[P],
    r: f64,
    material: Mat,
    verts: Option<usize>,
    round: usize,
) -> Obj {
    artkit::kit::tube(
        f,
        name,
        points,
        r,
        material,
        artkit::kit::Tube {
            verts,
            round,
            ..artkit::kit::Tube::default()
        },
    )
}

/// The kit's fillet (`artkit::kit::fillet`): the path through `points` with each inner corner rounded.
pub fn fillet(points: &[(f64, f64, f64)], radius: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    artkit::kit::fillet(points, radius, steps)
}

/// The kit's bar pull (`artkit::kit::pull`), as the script called it.
#[allow(clippy::too_many_arguments)]
pub fn pull(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    face: f64,
    length: f64,
    material: Mat,
    r: f64,
    out: f64,
    upright: bool,
    verts: usize,
) -> Obj {
    artkit::kit::pull(
        f,
        name,
        x,
        y,
        face,
        length,
        material,
        artkit::kit::Pull {
            r,
            out,
            upright,
            verts,
        },
    )
}

/// The kit's frame-and-panel front (`artkit::kit::shaker`), as the script called it: (frame, panel).
#[allow(clippy::too_many_arguments)]
pub fn shaker(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    at: (f64, f64, f64),
    t: f64,
    material: Mat,
    rail: f64,
    panel: Option<Mat>,
    recess: f64,
    raised: bool,
) -> (Obj, Obj) {
    artkit::kit::shaker(
        f,
        name,
        w,
        h,
        at,
        t,
        material,
        artkit::kit::Shaker {
            rail,
            panel,
            recess,
            raised,
        },
    )
}

/// The kit's moulding (`artkit::kit::moulding`), as the script called it: on the front at depth `at`, or on top
/// at height `at` (`front` false).
#[allow(clippy::too_many_arguments)]
pub fn moulding(
    f: &GameFrame,
    name: &str,
    outline: &[(f64, f64)],
    profile: &[(f64, f64)],
    material: Mat,
    at: f64,
    front: bool,
    angle: f64,
) -> Obj {
    let plane = if front {
        artkit::place::Facing::Front
    } else {
        artkit::place::Facing::Top
    };
    artkit::kit::moulding(
        f,
        name,
        outline,
        profile,
        material,
        artkit::kit::Moulding { at, plane, angle },
    )
}

/// The kit's songbird (`artkit::kit::songbird`), as the script called it (its turn a whole number of degrees).
#[allow(clippy::too_many_arguments)]
pub fn songbird(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    body: Mat,
    breast: Mat,
    beak: Mat,
    eye: Option<Mat>,
    turn: f64,
    size: f64,
    head: f64,
    low: bool,
) -> Vec<Obj> {
    let turn = artkit::Arg::arg(turn as i32);
    artkit::kit::songbird(
        f,
        name,
        x,
        y,
        z,
        body,
        breast,
        beak,
        eye,
        artkit::kit::Songbird {
            turn,
            size,
            head,
            low,
        },
    )
}
