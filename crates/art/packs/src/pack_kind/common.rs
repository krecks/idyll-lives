//! What the kind pack is built from: its palette, turned and lofted pieces, fluted rings, the tea set, little
//! shrubs, and the kit's parts it uses.

use artkit::forms::{Loft, Sweep, loft as loft_, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{P3, TAU, V3, cos, cosd, pow, radians, sin, sind, tan};
use artkit::mesh::Mesh;
use artkit::place::{GameFrame, Spot};
use artkit::rand::Random;
use artkit::scene::{self, Obj};
use artkit::shapes::{Sphere, shade, sphere};
use artkit::{
    boulder, fbox, fextrude, fsphere, fsweep, jitter, loft, palette, revolve, ring, torus,
};
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

// KIT: kit.tube
/// A round tube of radius `r` through game points, its bends rounded (`round` steps between each pair of
/// points; 1 keeps them sharp), its ends capped. `verts` round: by default 8, and more for one over 13 mm in
/// radius, as many as keep its outline within a millimetre of the circle (`forms::sweep`).
pub fn tube<P: Spot>(
    f: &GameFrame,
    name: &str,
    points: &[P],
    r: f64,
    material: Mat,
    verts: Option<usize>,
    round: usize,
) -> Obj {
    f.sweep(
        name,
        points,
        r,
        material,
        Sweep {
            verts,
            resolution: round,
            least: 8,
            ..Sweep::default()
        },
    )
}

// KIT: kit.fillet
/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
/// it, on a curve that never overshoots, however uneven the spacing: Catmull-Rom loops on a short leg beside a long
/// one). For handles, bent rails and draped cloth swept with `resolution` 1.
// (3.1415 is the script's own limit for a straight corner, not pi.)
#[allow(clippy::approx_constant)]
pub fn fillet(points: &[(f64, f64, f64)], radius: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let pts: Vec<V3> = points.iter().map(|&p| V3::from(p)).collect();
    let mut out = vec![pts[0]];
    for w in pts.windows(3) {
        let (a, p, c) = (w[0], w[1], w[2]);
        let (d1, d2) = (a - p, c - p);
        if d1.length() < 1e-9 || d2.length() < 1e-9 || d1.angle(d2) > 3.1415 {
            out.push(p);
            continue;
        }
        let t = (radius / 1e-6_f64.max(tan(d1.angle(d2) / 2.0)))
            .min(d1.length() * 0.5)
            .min(d2.length() * 0.5);
        let (p1, p2) = (p + d1.normalized() * t, p + d2.normalized() * t);
        for i in 0..=steps {
            let k = i as f64 / steps as f64;
            out.push(p1 * pow(1.0 - k, 2.0) + p * 2 * k * (1.0 - k) + p2 * k * k);
        }
    }
    out.push(pts[pts.len() - 1]);
    out.iter()
        .map(|p| (p.x as f64, p.y as f64, p.z as f64))
        .collect()
}

// KIT: kit.pull
/// A bar pull in one piece: out from a front at game depth `face` on two round stand-offs `length` apart
/// (across it, or `upright`), its bends rounded: a drawer's or a door's handle.
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
    let (h, b) = (length / 2.0, 0.012_f64.min(out * 0.45));
    let ends = [
        (-h, face - 0.003),
        (-h, face + out),
        (h, face + out),
        (h, face - 0.003),
    ];
    let pts: Vec<(f64, f64, f64)> = ends
        .iter()
        .map(|&(u, z)| {
            if upright {
                (x, y + u, z)
            } else {
                (x + u, y, z)
            }
        })
        .collect();
    f.sweep(
        name,
        &fillet(&pts, b, 3),
        r,
        material,
        Sweep {
            verts: Some(verts),
            ..Sweep::default()
        },
    )
}

// KIT: kit.shaker
/// A frame-and-panel front `w` x `h`, `t` thick, centred at game `at`, facing the front (a cabinet's door, a
/// drawer's front): a frame `rail` wide round a panel set back `recess`, flat, or `raised` (its edges chamfered
/// up to a field); `panel`: the panel's own material. Returns (frame, panel).
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
    let hole = rounded_rect(w - 2.0 * rail, h - 2.0 * rail, 0.003, 1);
    let fr = fextrude!(
        f,
        name,
        &rounded_rect(w, h, 0.003, 1),
        t,
        at,
        material,
        holes = vec![hole],
        bevel = if t * 0.2 < 0.004 { t * 0.2 } else { 0.004 },
        bevel_segments = 1
    );
    // (a hair inside the opening, not into the frame: a vertex buried in it would bake black)
    let (pw, ph) = (w - 2.0 * rail - 0.002, h - 2.0 * rail - 0.002);
    if !raised {
        let pn = fbox!(
            f,
            format!("{name}_panel"),
            pw,
            ph,
            t - 2.0 * recess,
            at,
            panel.unwrap_or(material),
            bevel = 0.002,
            bevel_segments = 1
        );
        return (fr, pn);
    }
    // a raised panel: a thin edge inside the frame, rising over a wide shallow chamfer to a flat field
    let (x, y, z) = at;
    let mut b = rail * 0.7;
    for c in [pw * 0.25, ph * 0.25] {
        if c < b {
            b = c;
        }
    }
    let lp = |hw: f64, hh: f64, d: f64| -> Vec<P3> {
        [(hw, hh), (-hw, hh), (-hw, -hh), (hw, -hh)]
            .iter()
            .map(|&(u, v)| f.at(x + u, y + v, d))
            .collect()
    };
    let front = z + t / 2.0;
    let pn = loft_(
        format!("{name}_panel"),
        &[
            lp(pw / 2.0, ph / 2.0, z - t / 2.0 + 0.002),
            lp(pw / 2.0, ph / 2.0, front - recess - 0.004),
            lp(pw / 2.0 - b, ph / 2.0 - b, front - recess * 0.2),
        ],
        artkit::forms::Loft {
            material: Some(panel.unwrap_or(material)),
            smooth: false,
            ..Default::default()
        },
    );
    (fr, pn)
}

// KIT: kit.moulding
/// A moulding run round a closed `outline` with mitred corners: a picture or mirror frame, a panel's bead, a
/// plinth's or a cornice's band. `outline` is game (x, height) points drawn at depth `at` (`front`), or (x, depth)
/// points at height `at`; `profile` is the moulding's closed cross-section [(inset, out), ...]: `inset` metres in
/// from the outline, `out` metres forward of `at` (front) or up from it (top). The corners stay square however
/// wide the profile (a sweep pinches them).
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
    let n = outline.len();
    let area =
        artkit::py::sum((0..n).map(|i| {
            outline[i].0 * outline[(i + 1) % n].1 - outline[(i + 1) % n].0 * outline[i].1
        }));
    let turn = if area > 0.0 { 1.0 } else { -1.0 }; // inward is to the left of an anticlockwise outline
    let inward = |a: (f64, f64), b: (f64, f64)| -> (f64, f64) {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let mut length = pow(dx * dx + dy * dy, 0.5);
        if length == 0.0 {
            length = 1.0;
        }
        (-dy / length * turn, dx / length * turn)
    };
    let (mut vs, mut faces) = (Vec::new(), Vec::new());
    let m = profile.len();
    for i in 0..n {
        let p = outline[i];
        let (n0, n1) = (
            inward(outline[(i + n - 1) % n], p),
            inward(p, outline[(i + 1) % n]),
        );
        let k = 1.0 + n0.0 * n1.0 + n0.1 * n1.1;
        let (mx, my) = ((n0.0 + n1.0) / k, (n0.1 + n1.1) / k);
        for &(inset, out) in profile {
            let (u, v) = (p.0 + inset * mx, p.1 + inset * my);
            vs.push(V3::from(if front {
                f.at(u, v, at + out)
            } else {
                f.at(u, at + out, v)
            }));
        }
    }
    for i in 0..n {
        let (a, b) = (i * m, ((i + 1) % n) * m);
        for j in 0..m {
            let j2 = (j + 1) % m;
            faces.push(vec![
                (a + j) as u32,
                (a + j2) as u32,
                (b + j2) as u32,
                (b + j) as u32,
            ]);
        }
    }
    let mut mesh = Mesh::from_pydata(vs, &faces);
    mesh.set_material(Some(material));
    mesh.recalc_normals();
    shade(scene::add(name, mesh), true, angle)
}

// KIT: kit.songbird
/// A small songbird (a robin, a tit, a sparrow) perched with its feet at game (x, y, z), facing the front turned
/// `turn` degrees towards +x: a lofted body tapering into a fanned tail, a `breast` patch, a pointed `beak`, `eye`s,
/// thin legs down to the perch. `size` 1 is a robin (14 cm long); `head` tips the head down (degrees: drinking,
/// pecking); `low` takes fewer sides and no eyes (a far version). About 350 triangles. Returns its pieces.
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
    let (c, s) = (cos(radians(turn)), sin(radians(turn)));
    let dip = radians(head);
    // The game point `u` across (to the bird's right), `v` forward and `w` up from its feet (at size 1).
    let at = |u: f64, v: f64, w: f64| -> P3 {
        let (mut v, mut w) = (v, w);
        if v > 0.03 {
            // the head and beak tip down about the neck
            let (dv, dw) = (v - 0.03, w - 0.048);
            (v, w) = (
                0.03 + dv * cos(dip) + dw * sin(dip),
                0.048 - dv * sin(dip) + dw * cos(dip),
            );
        }
        let (u, v, w) = (u * size, v * size, w * size);
        f.at(x + v * s + u * c, y + w, z + v * c - u * s)
    };
    // (forward, half width, half height, centre height): from the tail's root to the face
    let rows = [
        (-0.05, 0.010, 0.010, 0.040),
        (-0.035, 0.022, 0.024, 0.036),
        (-0.012, 0.029, 0.031, 0.034),
        (0.012, 0.027, 0.030, 0.038),
        (0.03, 0.02, 0.022, 0.048),
        (0.045, 0.02, 0.02, 0.058),
        (0.06, 0.016, 0.015, 0.06),
        (0.07, 0.007, 0.007, 0.059),
    ];
    let n = if low { 6 } else { 10 };
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(v, hw, hh, wc)| {
            (0..n)
                .map(|k| {
                    at(
                        hw * cos(TAU * k as f64 / n as f64),
                        v,
                        wc + hh * sin(TAU * k as f64 / n as f64),
                    )
                })
                .collect()
        })
        .collect();
    let mut made = vec![loft_(
        name,
        &sections,
        Loft {
            material: Some(body),
            ..Loft::default()
        },
    )];
    made.push(artkit::forms::sweep(
        format!("{name}_beak"),
        &[at(0.0, 0.066, 0.058), at(0.0, 0.088, 0.057)],
        0.0055 * size,
        Sweep {
            material: Some(beak),
            verts: Some(if low { 5 } else { 6 }),
            scale: artkit::forms::Scale::List(vec![1.0, 0.15]),
            ..Sweep::default()
        },
    ));
    let breast_at = V3::from(at(0.0, 0.01, 0.034));
    made.push(sphere(
        format!("{name}_breast"),
        1.0,
        Sphere {
            at: breast_at.into(),
            material: Some(breast),
            rot: P3 {
                x: 0.0,
                y: 0.0,
                z: -turn,
            },
            scale: P3 {
                x: 0.026 * size,
                y: 0.027 * size,
                z: 0.025 * size,
            },
            segments: if low { 8 } else { 10 },
            rings: if low { 5 } else { 6 },
            ..Sphere::default()
        },
    ));
    // the tail: a thin fan, cocked a little up
    made.push(artkit::forms::sweep(
        format!("{name}_tail"),
        &[at(0.0, -0.04, 0.041), at(0.0, -0.088, 0.054)],
        0.0,
        Sweep {
            material: Some(body),
            profile: artkit::forms::Profile(Some(vec![
                (-0.0018, -0.008),
                (0.0018, -0.008),
                (0.0018, 0.008),
                (-0.0018, 0.008),
            ])),
            up: P3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            scale: artkit::forms::Scale::List(vec![0.9 * size, 1.8 * size]),
            ..Sweep::default()
        },
    ));
    if let Some(eye) = eye
        && !low
    {
        for k in [-1, 1] {
            made.push(sphere(
                format!("{name}_eye{k}"),
                0.0042 * size,
                Sphere {
                    at: at(k as f64 * 0.0148, 0.055, 0.062),
                    material: Some(eye),
                    segments: 6,
                    rings: 4,
                    ..Sphere::default()
                },
            ));
        }
    }
    for k in [-1, 1] {
        let kf = k as f64;
        made.push(artkit::forms::sweep(
            format!("{name}_leg{k}"),
            &[
                at(kf * 0.008, 0.0, 0.026),
                at(kf * 0.009, 0.004, 0.0),
                at(kf * 0.009, 0.016, 0.0),
            ],
            0.0016 * size,
            Sweep {
                material: Some(beak),
                verts: Some(4),
                ..Sweep::default()
            },
        ));
    }
    made
}
