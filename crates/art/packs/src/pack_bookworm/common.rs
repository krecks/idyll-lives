//! What the bookworm pack is built from: its palette, books, china and lamps shared by the six.

use artkit::materials::Mat;
use artkit::math::{cos, degrees, sin};
use artkit::parts::numbered;
use artkit::place::GameFrame;
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fsweep, kit, palette, revolve, torus};

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
pub use artkit::kit::WALL;

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
        let n = numbered("book");
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
        let n = numbered("book");
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
        let nm = numbered("pile");
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
        numbered("mug"),
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
        numbered("mug_tea"),
        0.033,
        0.002,
        (x, y + 0.072, z),
        fill.unwrap_or(M.tea),
        verts = 12
    );
    fsweep!(
        f,
        numbered("mug_handle"),
        &kit::fillet!(
            &[
                (x + 0.036, y + 0.074, z),
                (x + 0.066, y + 0.07, z),
                (x + 0.064, y + 0.028, z),
                (x + 0.038, y + 0.022, z)
            ],
            0.012
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
        numbered("lamp_foot"),
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
    kit::tube!(
        f,
        numbered("lamp_stem"),
        &[(x, y + 0.04, z), (x, y + h * 0.62, z)],
        0.008,
        M.brass,
        verts = 8,
        round = 1
    );
    revolve!(
        numbered("lamp_collar"),
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
        numbered("lamp_socket"),
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
        numbered("lamp_shade"),
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
            numbered("lamp_rim"),
            r,
            0.0035,
            at = f.at(x, yy, z),
            material = M.brass,
            major_segments = 24,
            minor_segments = 3
        );
    }
    revolve!(
        numbered("lamp_finial"),
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
