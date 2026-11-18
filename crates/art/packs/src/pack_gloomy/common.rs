//! What the gloomy pack is built from: its palette, turned and lofted pieces, candles, vases, books and legs, and
//! the kit's parts it uses.

use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, cosd, pow, sind};
use artkit::place::{GameFrame, Spot};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, palette, revolve, ring};
use std::cell::Cell;

palette! { M:
    walnut = ("#7A5234", wood), // the record cabinet; its main part
    slat = ("#8E6443", wood),
    rosewood = ("#8A4A35", wood), // the cello's stool and stand, the writing desk
    ink = ("#3A3550", satin), // legs, the cabinet's inside, trim
    plum = ("#6B4A6E", satin), // cushions and upholstery
    mauve = ("#9C7FA0", satin),
    slate = ("#5E6F8C", metal), // the rain lamp's base and canopy; its main part
    smoke = ("#A7A9B4", satin),
    lamp_light = ("#DDE6F2", satin, glow("#CFE0FF")), // the rain lamp's light under its canopy: it glows at night
    chrome = ("#D0D3D6", chrome),
    brass = ("#B8925A", metal),
    platter = ("#C9CCD0", metal),
    vinyl = ("#26222E", gloss),
    label = ("#C25B7A", gloss),
    page = ("#EFE6D3", matte),
    sleeve_a = ("#5E6F8C", matte),
    sleeve_b = ("#9C7FA0", matte),
    sleeve_c = ("#C9B37E", matte),
    sleeve_d = ("#4A3F5C", matte),
    wax = ("#F3EAD8", satin),
    wick = ("#2E2420", satin),
    candle_flame = ("#FFC45C", gloss),
    flame = ("#FFB347", gloss, glow("#FFC870")), // the desk's candle flame, which glows at night
    vase = ("#B7A9C6", gloss),
    twig = ("#6E5A3E", matte),
    dried = ("#C9A0B8", matte),
    varnish = ("#9A4A26", gloss), // the cello
    ebony = ("#2E2420", gloss),
    bridge = ("#E6D3B0", matte),
    string = ("#E3E3E3", chrome),
    rose = ("#8A2E46", satin),
    stem = ("#5C6B3A", matte),
    inkwell = ("#2E3348", gloss),
    pool = ("#BFE3F2", gloss, glow("#9CCBFF")), // the lit pool and the rain over it glow with the lamp
    rain = ("#A9D8EE", gloss, glow("#BFE4FF")),
    ripple = ("#E6F6FC", gloss),
    pebble = ("#B8B4AE", satin),
    pebble_dark = ("#9EA0A6", satin),
    umbrella = ("#3A3550", satin),
    figure = ("#5E6F8C", satin),
    bench = ("#7E8BA6", satin), // the window seat; its main part
    bench_front = ("#8592AD", satin),
    frame = ("#D8D6DE", satin),
    pane = ("#4F6684", gloss), // the rainy window: opaque gloss, not see-through glass (the wall is behind it)
    drop = ("#E3F0F8", gloss),
    cushion = ("#9C7FA0", fabric), // the window seat's cushion and throw (their own part)
    throw = ("#A7A9B4", fabric),
    pillow_a = ("#5E6F8C", satin),
    pillow_b = ("#C9B37E", satin),
    mug = ("#E8E4EC", gloss),
    stone = ("#BDB7AD", satin), // the soaking tub; its main part
    moon = ("#E4E1EA", gloss),
    ledge = ("#5A4A4E", wood), // the tub's candle ledge
    ledge_top = ("#CFC9C0", satin),
    water = ("#8FC1D4", gloss),
    foam = ("#F2F6F8", gloss),
    towel = ("#EDE8F2", satin),
    leaf = ("#6E8F72", satin),
    moss = ("#5F7A4E", satin), // the moss on the rain lamp's rock
}

/// The wall's face in a 1-deep footprint, game depth.
pub const WALL: f64 = -0.43;
/// In a 2-deep one.
pub const WALL2: f64 = -0.93;

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
) -> Obj {
    revolve!(
        name,
        profile,
        at = f.at(x, y, z),
        material = material,
        segments = segments
    )
}

/// A body lofted through round rings [(radius, height up), ...] standing at game (x, y, z): a vase, a bottle.
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

/// A game (x, depth) offset turned `deg` degrees as a part's Blender z turn turns it.
pub fn turn(dx: f64, dz: f64, deg: f64) -> (f64, f64) {
    let (c, s) = (cosd(deg), sind(deg));
    (dx * c - dz * s, dx * s + dz * c)
}

/// A pillar candle standing on `y`: its top melted into a hollow round the wick, wax run down its side in
/// `drips`, a still flame (`lit`); `holder`: on a brass dish (its rim 3 cm out), which lifts it 6 mm.
#[allow(clippy::too_many_arguments)]
pub fn candle(
    f: &GameFrame,
    x: f64,
    y: f64,
    z: f64,
    h: f64,
    r: f64,
    lit: bool,
    holder: bool,
    drips: usize,
    seed: i64,
) -> f64 {
    let mut y = y;
    if holder {
        let rr = r + 0.028;
        turned(
            f,
            &name("candle_dish"),
            x,
            y,
            z,
            &[
                (0.0, 0.0),
                (rr - 0.008, 0.0),
                (rr, 0.006),
                (rr - 0.003, 0.011),
                (r + 0.004, 0.006),
                (0.0, 0.006),
            ],
            M.brass,
            12,
        );
        y += 0.006;
    }
    turned(
        f,
        &name("candle"),
        x,
        y,
        z,
        &[
            (0.0, 0.0),
            (r, 0.0),
            (r, h - 0.006),
            (r * 0.94, h),
            (r * 0.55, h - 0.007),
            (0.0, h - 0.009),
        ],
        M.wax,
        12,
    );
    let seed = if seed != 0 {
        seed
    } else {
        artkit::py::int(x.abs() * 1000.0 + z.abs() * 300.0 + h * 50.0)
    };
    let mut rnd = Random::new(seed as i128);
    for _ in 0..drips {
        let a = rnd.uniform(0.0, 360.0);
        let (c, s) = (cosd(a), sind(a));
        let l = rnd.uniform(0.25, 0.55) * h;
        fsweep!(
            f,
            name("drip"),
            &[
                (x + (r - 0.004) * c, y + h - 0.003, z + (r - 0.004) * s),
                (x + (r + 0.001) * c, y + h - 0.008, z + (r + 0.001) * s),
                (x + (r + 0.0015) * c, y + h - l, z + (r + 0.0015) * s)
            ],
            0.0035,
            M.wax,
            verts = 6,
            scale = [0.8, 1.0, 1.35]
        );
    }
    fsweep!(
        f,
        name("wick"),
        &[
            (x, y + h - 0.009, z),
            (x + 0.001, y + h + 0.002, z),
            (x + 0.003, y + h + 0.008, z)
        ],
        0.0016,
        M.wick,
        verts = 4
    );
    if lit {
        fsphere!(
            f,
            name("candle_flame"),
            0.008,
            (x + 0.003, y + h + 0.026, z),
            M.candle_flame,
            scale = (1, 1, 2.2),
            segments = 7,
            rings = 5
        );
    }
    y + h
}

/// A ceramic vase: a full belly, a narrow neck, a flared lip, hollow at the mouth. Returns its top.
pub fn vase(f: &GameFrame, x: f64, y: f64, z: f64, s: f64) -> f64 {
    lathe(
        f,
        &name("vase"),
        x,
        y,
        z,
        &[
            (0.042 * s, 0.0),
            (0.06 * s, 0.025 * s),
            (0.07 * s, 0.075 * s),
            (0.064 * s, 0.12 * s),
            (0.042 * s, 0.158 * s),
            (0.026 * s, 0.18 * s),
            (0.028 * s, 0.192 * s),
            (0.034 * s, 0.2 * s),
            (0.026 * s, 0.198 * s),
            (0.02 * s, 0.17 * s),
        ],
        M.vase,
        12,
        2.0,
    );
    y + 0.2 * s
}

/// A book lying open at game (x, y, z), its spine along the depth: a cover `2w` x `d`, each half of pages rising in
/// a curve from the spine and falling to its edge; `lines` (a material): five ruled lines or staves on each page.
#[allow(clippy::too_many_arguments)]
pub fn open_book(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    d: f64,
    cover: Mat,
    page: Mat,
    lines: Option<Mat>,
    turn_: f64,
) {
    fbox!(
        f,
        format!("{name}_cover"),
        2.0 * w + 0.012,
        0.004,
        d + 0.01,
        (x, y + 0.002, z),
        cover,
        rot = (0.0, 0.0, turn_)
    );
    let lift = |t: f64| 0.004 + 0.003 + 0.012 * (1.0 - pow(1.0 - t, 3.0)) * (1.0 - 0.35 * t);
    let n = 8;
    for s in [-1, 1] {
        let sf = s as f64;
        let top: Vec<(f64, f64)> = (0..=n)
            .map(|k| (sf * w * k as f64 / n as f64, lift(k as f64 / n as f64)))
            .collect();
        let mut outline = vec![(0.0, 0.004), (sf * w, 0.004)];
        outline.extend(top.iter().rev());
        if s < 0 {
            outline.reverse();
        }
        let o = fextrude!(
            f,
            format!("{name}_pages{s}"),
            &outline,
            d,
            (x, y, z),
            page,
            smooth = false
        );
        o.edit(|p| p.rot.z = artkit::math::radians(turn_) as f32);
        if let Some(lines) = lines {
            for k in 0..5 {
                let lz = -d * 0.32 + d * 0.16 * k as f64;
                let pts: Vec<(f64, f64, f64)> = (0..3)
                    .map(|i| {
                        let t = 0.12 + 0.76 * i as f64 / 2.0;
                        let (dx, dz) = turn(sf * w * t, lz, turn_);
                        (x + dx, y + lift(t) + 0.0007, z + dz)
                    })
                    .collect();
                fsweep!(
                    f,
                    format!("{name}_line{s}{k}"),
                    &pts,
                    0,
                    lines,
                    profile = rounded_rect(0.0012, 0.0025, 0.0004, 1),
                    up = (0, 0, 1)
                );
            }
        }
    }
}

/// A closed book lying flat on `y`, its spine at its -x side: boards, spine and the block of pages between.
#[allow(clippy::too_many_arguments)]
pub fn book(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    d: f64,
    cover: Mat,
    turn_: f64,
    page: Option<Mat>,
) {
    let page = page.unwrap_or(M.page);
    for (k, (dx, dy, bw, bh, bd, m)) in [
        (0.0, 0.0015, w, 0.003, d, cover),
        (0.0, h - 0.0015, w, 0.003, d, cover),
        (-w / 2.0 + 0.002, h / 2.0, 0.004, h, d, cover),
        (0.002, h / 2.0, w - 0.006, h - 0.005, d - 0.006, page),
    ]
    .into_iter()
    .enumerate()
    {
        let (ox, oz) = turn(dx, 0.0, turn_);
        fbox!(
            f,
            format!("{name}{k}"),
            bw,
            bh,
            bd,
            (x + ox, y + dy, z + oz),
            m,
            rot = (0.0, 0.0, turn_)
        );
    }
}

/// A tapered leg from `top` down to the floor, splayed by (dx, dz) at its foot; `foot`: a metal ferrule.
#[allow(clippy::too_many_arguments)]
pub fn leg(
    f: &GameFrame,
    name: &str,
    x: f64,
    z: f64,
    top: f64,
    material: Mat,
    r: f64,
    splay: (f64, f64),
    foot: Option<Mat>,
    verts: usize,
) {
    let (sx, sz) = splay;
    fsweep!(
        f,
        name,
        &[
            (x, top, z),
            (x + sx * 0.45, top * 0.5, z + sz * 0.45),
            (x + sx, 0.0, z + sz)
        ],
        r,
        material,
        verts = verts,
        scale = [1.0, 0.82, 0.58]
    );
    if let Some(foot) = foot {
        fcyl!(
            f,
            format!("{name}_ferrule"),
            r * 0.63,
            0.03,
            (x + sx, 0.015, z + sz),
            foot,
            verts = 6
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

/// The kit's cabriole leg (`artkit::kit::cabriole`), as the script called it.
#[allow(clippy::too_many_arguments)]
pub fn cabriole(
    f: &GameFrame,
    name: &str,
    x: f64,
    z: f64,
    h: f64,
    material: Mat,
    sx: f64,
    sz: f64,
    r: f64,
    verts: usize,
) -> Obj {
    artkit::kit::cabriole(
        f,
        name,
        x,
        z,
        h,
        material,
        sx,
        sz,
        artkit::kit::Cabriole { r, verts },
    )
}

/// The kit's iron scroll (`artkit::kit::scroll`), as the script called it.
#[allow(clippy::too_many_arguments)]
pub fn scroll(
    f: &GameFrame,
    name: &str,
    start: (f64, f64, f64),
    centre: (f64, f64),
    material: Mat,
    turns: f64,
    end: f64,
    t: f64,
    w: f64,
    steps: usize,
) -> Obj {
    artkit::kit::scroll(
        f,
        name,
        start,
        centre,
        material,
        artkit::kit::Scroll {
            turns,
            end,
            t,
            w,
            steps,
            ..artkit::kit::Scroll::default()
        },
    )
}
