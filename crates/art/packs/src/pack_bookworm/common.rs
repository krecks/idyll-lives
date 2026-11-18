//! What the bookworm pack is built from: its palette, books, china and lamps shared by the six, and the kit's parts
//! it uses.

use artkit::forms::{Sweep, loft as loft_, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{P3, V3, cos, degrees, pow, sin, tan};
use artkit::mesh::Mesh;
use artkit::place::{GameFrame, Spot};
use artkit::rand::Random;
use artkit::scene::{self, Obj};
use artkit::shapes::shade;
use artkit::{fbox, fcyl, fextrude, fsweep, palette, revolve, torus};
use std::cell::Cell;

palette! { M:
    body = ("#9A6A45", wood), // the library walnut: cases, shelves, the bureau; the main part
    panel = ("#B88A60", wood), // drawer fronts, inset panels, the bureau's flap
    honey = ("#C79A62", wood), // the ladder, the bureau chair, the trolley's shelves
    walnut = ("#8E6243", satin), // the lounge's wood, plain (the lounge's fabric takes its textured draw)
    back = ("#E8DCC6", matte), // the cases' backs
    brass = ("#C9A54A", metal),
    iron = ("#3F4247", metal), // the fire grate, the hood and flue
    dark = ("#2F3236", satin), // castors, spectacles, the typewriter platen
    stone = ("#B9B0A2", satin), // the hearth stone
    fabric = ("#C9B48A", fabric), // the nook's seat cushion and throw
    throw = ("#7A2E2E", fabric),
    cushion = ("#E6DCC6", satin), // the nook's back cushions
    pillow_a = ("#2F5D62", satin),
    pillow_b = ("#B5653F", satin),
    velvet = ("#4F6B55", fabric), // the wing chairs
    rug = ("#9C4A3A", fabric),
    rug_inner = ("#C9A46E", fabric),
    seatpad = ("#7A2E2E", satin), // the bureau chair's pad
    linen = ("#EDE3CF", satin), // the trolley's cloth
    shade = ("#E9D5A8", satin), // lamp shades
    lamp_amber = ("#D6A64A", satin),
    porcelain = ("#F6F3EA", gloss),
    glaze = ("#2F5D62", gloss), // the tea set's band, the biscuit tin
    cake = ("#C8964F", matte),
    icing = ("#F4E9D8", satin),
    page = ("#F4EEDF", matte),
    globe = ("#6E9FB4", gloss),
    bust = ("#E9E4DA", satin),
    log = ("#7A5A40", satin),
    wicker = ("#C9A46E", satin),
    flame = ("#F28C28", gloss),
    flame_core = ("#FFC24A", gloss),
    ember = ("#3A2420", satin),
    mug_a = ("#2F5D62", gloss),
    mug_b = ("#D6A64A", gloss),
    // book spines: plain satin, so a wall of books costs no draw call of its own
    book_a = ("#7A2E2E", satin),
    book_b = ("#2F5D62", satin),
    book_c = ("#B5653F", satin),
    book_d = ("#D6A64A", satin),
    book_e = ("#3E5A3A", satin),
    book_f = ("#E6DCC6", satin),
    book_g = ("#5A4A7A", satin),
    // added with the detail pass
    land = ("#C9B47A", satin), // the globe's continents
    log_end = ("#B89270", satin), // the logs' cut ends
    tea = ("#7A4A26", gloss), // tea in the cups
    ink = ("#1E2A3A", gloss),
}

/// The book spines' colours, chosen from by name.
pub const BOOKS: [Mat; 7] = [
    M.book_a, M.book_b, M.book_c, M.book_d, M.book_e, M.book_f, M.book_g,
];
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

// --- books, china and lamps, shared by the six -------------------------------------------------------------------

/// Gives a book's box its page edges in cream on the same mesh (no more triangles): the top of a standing book,
/// the three sides but the spine (its front) of one lying down.
pub fn pages(obj: Obj, lying: bool) -> Obj {
    obj.edit(|o| {
        let k = o.mesh.slot(M.page);
        for f in 0..o.mesh.faces.len() {
            let n = o.mesh.face_normal(f);
            if if lying {
                n.z.abs() < 0.5 && n.y > -0.5
            } else {
                n.z > 0.5
            } {
                o.mesh.faces[f].mat = k;
            }
        }
    });
    obj
}

#[allow(clippy::too_many_arguments)]
pub fn book(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    at: (f64, f64, f64),
    cover: Mat,
    rot: (f64, f64, f64),
    lying: bool,
) -> Obj {
    pages(fbox!(f, name, w, h, d, at, cover, rot = rot), lying)
}

/// A row of books standing on `y` from `x0` towards `x1` (game), spines to the front, backs on `z_back`;
/// the gap left at the end takes a short pile lying flat (`stack`) or a book leaning on the row (`lean`).
#[allow(clippy::too_many_arguments)]
pub fn books(
    f: &GameFrame,
    rnd: &mut Random,
    x0: f64,
    x1: f64,
    y: f64,
    z_back: f64,
    depth: f64,
    h_max: f64,
    fill: f64,
    stack: bool,
    lean: bool,
) {
    let (mut x, end) = (x0 + 0.004, x0 + (x1 - x0) * fill);
    while x < end {
        let t = rnd.uniform(0.022, 0.05);
        let h = h_max * rnd.uniform(0.66, 1.0);
        let d = depth * rnd.uniform(0.78, 1.0);
        let n = name("book");
        let cover = rnd.choice(&BOOKS);
        book(
            f,
            &n,
            t,
            h,
            d,
            (x + t / 2.0, y + h / 2.0, z_back + d / 2.0),
            cover,
            (0.0, 0.0, 0.0),
            false,
        );
        x += t + 0.002;
    }
    if stack && x1 - x > 0.16 {
        let w = if x1 - x - 0.02 < 0.2 {
            x1 - x - 0.02
        } else {
            0.2
        };
        pile(
            f,
            rnd,
            x + 0.01 + w / 2.0,
            z_back + depth * 0.45,
            y,
            3,
            w,
            depth * 0.85,
        );
    } else if lean && x1 - x > 0.08 {
        // a book leaning on the row, on its bottom edge nearest it, its top against the last one
        let (h, t) = (h_max * 0.9, 0.03);
        let s = (x1 - x - t - 0.008) / h;
        let a = libm::asin(if s < 0.45 { s } else { 0.45 });
        let px = x + h * sin(a);
        let c = (
            px + t / 2.0 * cos(a) - h / 2.0 * sin(a),
            y + t / 2.0 * sin(a) + h / 2.0 * cos(a),
        );
        let n = name("book");
        let cover = rnd.choice(&BOOKS);
        book(
            f,
            &n,
            t,
            h,
            depth * 0.9,
            (c.0, c.1, z_back + depth * 0.45),
            cover,
            (0.0, degrees(a), 0.0),
            false,
        );
    }
}

/// `n` books lying in a pile on `y`, centred on (x, z); returns the height of its top.
#[allow(clippy::too_many_arguments)]
pub fn pile(
    f: &GameFrame,
    rnd: &mut Random,
    x: f64,
    z: f64,
    y: f64,
    n: usize,
    w: f64,
    d: f64,
) -> f64 {
    let mut y = y;
    for _ in 0..n {
        let h = rnd.uniform(0.025, 0.045);
        let nm = name("pile");
        let bw = w * rnd.uniform(0.85, 1.05);
        let bd = d * rnd.uniform(0.9, 1.05);
        let bx = x + rnd.uniform(-0.01, 0.01);
        let cover = rnd.choice(&BOOKS);
        let turn = rnd.uniform(-10.0, 10.0);
        book(
            f,
            &nm,
            bw,
            h,
            bd,
            (bx, y + h / 2.0, z),
            cover,
            (0.0, 0.0, turn),
            true,
        );
        y += h;
    }
    y
}

/// A mug standing on `y`: a turned body with its lip and a hollow, tea in it, a handle in one bent piece.
pub fn mug(f: &GameFrame, x: f64, y: f64, z: f64, material: Mat, fill: Option<Mat>) {
    revolve!(
        name("mug"),
        &[
            (0.0, 0.0),
            (0.036, 0.0),
            (0.04, 0.006),
            (0.042, 0.088),
            (0.039, 0.092),
            (0.035, 0.088),
            (0.033, 0.07),
            (0.0, 0.07)
        ],
        at = f.at(x, y, z),
        material = material,
        segments = 14
    );
    fcyl!(
        f,
        name("mug_tea"),
        0.033,
        0.002,
        (x, y + 0.072, z),
        fill.unwrap_or(M.tea),
        verts = 12
    );
    fsweep!(
        f,
        name("mug_handle"),
        &fillet(
            &[
                (x + 0.036, y + 0.074, z),
                (x + 0.066, y + 0.07, z),
                (x + 0.064, y + 0.028, z),
                (x + 0.038, y + 0.022, z)
            ],
            0.012,
            3
        ),
        0.0065,
        material,
        verts = 6
    );
}

/// A brass table lamp standing on `y`: a stepped round foot, a stem with a collar, a socket, a drum shade open
/// top and bottom with rolled brass rims, a finial over it.
pub fn lamp(f: &GameFrame, x: f64, y: f64, z: f64, h: f64, shade_: Option<Mat>) {
    let shade_ = shade_.unwrap_or(M.shade);
    revolve!(
        name("lamp_foot"),
        &[
            (0.0, 0.0),
            (0.07, 0.0),
            (0.07, 0.008),
            (0.062, 0.014),
            (0.05, 0.02),
            (0.03, 0.03),
            (0.02, 0.04),
            (0.012, 0.045),
            (0.0, 0.045)
        ],
        at = f.at(x, y, z),
        material = M.brass,
        segments = 14
    );
    tube(
        f,
        &name("lamp_stem"),
        &[(x, y + 0.04, z), (x, y + h * 0.62, z)],
        0.008,
        M.brass,
        Some(8),
        1,
    );
    revolve!(
        name("lamp_collar"),
        &[
            (0.0, 0.0),
            (0.014, 0.0),
            (0.017, 0.008),
            (0.014, 0.016),
            (0.0, 0.016)
        ],
        at = f.at(x, y + h * 0.3, z),
        material = M.brass,
        segments = 10
    );
    revolve!(
        name("lamp_socket"),
        &[
            (0.0, 0.0),
            (0.018, 0.0),
            (0.018, 0.05),
            (0.012, 0.06),
            (0.0, 0.06)
        ],
        at = f.at(x, y + h * 0.62, z),
        material = M.brass,
        segments = 10
    );
    let (rb, rt, sy, sh) = (0.3 * h, 0.2 * h, y + h * 0.56, h * 0.42);
    revolve!(
        name("lamp_shade"),
        &[
            (rb, 0.0),
            (rt, sh),
            (rt - 0.004, sh),
            (rb - 0.004, 0.0),
            (rb, 0.0)
        ],
        at = f.at(x, sy, z),
        material = shade_,
        segments = 24
    );
    for (r, yy) in [(rb - 0.002, sy + 0.003), (rt - 0.002, sy + sh - 0.003)] {
        torus!(
            name("lamp_rim"),
            r,
            0.0035,
            at = f.at(x, yy, z),
            material = M.brass,
            major_segments = 24,
            minor_segments = 3
        );
    }
    revolve!(
        name("lamp_finial"),
        &[
            (0.0, 0.0),
            (0.006, 0.0),
            (0.011, 0.01),
            (0.006, 0.02),
            (0.0, 0.022)
        ],
        at = f.at(x, sy + sh, z),
        material = M.brass,
        segments = 8
    );
}

/// A small turned knob on a front at game depth `face` (kit.knob's shape in fewer rings: these are many).
#[allow(clippy::too_many_arguments)]
pub fn knob(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    face: f64,
    r: f64,
    material: Option<Mat>,
    segments: usize,
) -> Obj {
    let prof = [
        (0.0, 0.0),
        (r * 0.7, 0.0),
        (r * 0.38, r * 0.45),
        (r * 0.9, r * 0.95),
        (r, r * 1.3),
        (r * 0.6, r * 1.75),
        (0.0, r * 1.85),
    ];
    revolve!(
        name,
        &prof,
        at = f.at(x, y, face),
        material = material.unwrap_or(M.brass),
        segments = segments,
        rot = (90, 0, 0)
    )
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

// KIT: kit.cabriole
/// A cabriole leg from the frame at height `h` over (x, z) down to the floor: out by (sx, sz) at the knee,
/// slim at the ankle, flaring into a pad foot that stands on the floor (its lowest point at height 0).
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
    fsweep!(
        f,
        name,
        &[
            (x, h, z),
            (x + sx, h * 0.62, z + sz),
            (x + sx * 0.4, 0.06, z + sz * 0.4),
            (x + sx * 0.55, 0.02, z + sz * 0.55),
            (x + sx * 0.55, 0.0, z + sz * 0.55)
        ],
        r,
        material,
        verts = verts,
        resolution = 2,
        scale = [1.0, 1.05, 0.7, 1.1, 1.15]
    )
}
