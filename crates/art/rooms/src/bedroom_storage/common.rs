//! What the bedroom's storage is built from: the palette, the frame, and the pieces more than one of them takes
//! (fronts, drawers, carcasses, clothes on hangers and folded, books, a mug).

use artkit::forms::smooth_outline;
use artkit::kit;
use artkit::materials::Mat;
use artkit::palette;
use artkit::parts::{join, new_parts};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, revolve, soft};

palette! { M:
    oak = ("#CDB08A", wood),         // oak: the nightstand, the dresser and the low dresser
    oakleg = ("#B08E66", wood),      // the low dresser's legs
    pull = ("#7A5638", wood),        // the dresser's jewellery box
    ebony = ("#2B2C2E", wood),       // the dark gap behind a wood drawer
    brass = ("#B8925A", metal),      // brass handles, knobs and pulls, the bench's frame and the lamp's stem
    iron = ("#2B2C2E", metal),       // black metal: the modern nightstand's and low dresser's pulls, the chest's straps
    white = ("#F1F0EC", satin),      // painted white: the modern nightstand and the two wardrobes
    birch = ("#CDB08A", satin),      // the wardrobe's doors
    plinth = ("#2F3236", satin),     // plinths, and the dark gap behind a painted drawer
    clay = ("#C98F6B", satin),       // the drum nightstand
    clay2 = ("#B87D5C", satin),      // its drawer band and foot
    walnut = ("#8C6444", satin),     // the three-drawer nightstand; the armoire's raised panels
    mahogany = ("#7C563B", satin),   // the armoire
    mahogany2 = ("#684731", satin),  // its cornice, base and feet
    linen = ("#E9E2D4", satin),      // the tall chest of drawers
    cedar = ("#A0603E", wood),       // the blanket chest
    cedar2 = ("#8A5034", wood),      // its plinth and lid edge
    cedarin = ("#C98A5E", wood),     // its lighter inside
    wicker = ("#C9A979", fabric),    // the hamper's woven basket
    wicker2 = ("#B8955F", fabric),   // its lid, rim and bands
    olive = ("#8C9468", fabric),     // the bench's cushion
    olive2 = ("#737B54", fabric),    // its buttons
    porcelain = ("#E8E2D6", gloss),  // the lamp's base
    bulb = ("#FFF1CF", gloss),       // the lamp's bulb
    shade = ("#F4EBD8", fabric, glow("#FFD9A0")),   // the lamp's shade, glowing at night
    teal = ("#4E7A7F", matte),       // a book
    book2 = ("#B5653F", matte),      // more books
    book3 = ("#E3D5B8", matte),
    paper = ("#F2EDE0", matte),      // a book's pages, a box's label
    cream = ("#EFE5D3", gloss),      // a mug
    coffee = ("#4A2E1E", gloss),     // the coffee in it
    tin = ("#4E7A7F", gloss),        // a biscuit tin in the nightstand's drawer
    wrapper = ("#C65D52", satin),    // chocolate bars
    box_ = ("#C9B79C", matte, named("box")), // storage boxes on a wardrobe (`box` is Rust's own word)
    box2 = ("#E6DFD3", matte),
    chrome = ("#C9CCCF", chrome),    // a hanging rail, the hangers and the wardrobes' hinges
    cloth1 = ("#5B7DB1", satin),     // clothes: hanging and folded
    cloth2 = ("#C65D52", satin),
    cloth3 = ("#E8D9B5", satin),
    cloth4 = ("#6F8F6A", satin),
    frame = ("#B08E66", wood),       // a picture frame on the dresser
    print1 = ("#4E7A7F", wood),      // its picture (a stained board: wood, so it draws with the dresser)
    vase = ("#8A5A38", wood),        // a turned wooden vase
}

/// Game numbers as they are: the models stand where the game places them.
pub const F: GameFrame = GameFrame::new(0.0);
pub use artkit::kit::WALL;

pub const DRAWER_SECONDS: f64 = 0.9; // a drawer slides out 30 cm in this long
// seconds the nightstand's drawer stays out (the dressers' and wardrobes' stay open while the resident changes,
// the whole use)
pub const DRAWER_HOLD: f64 = 2.5;
pub const DOOR_SECONDS: f64 = 1.2; // doors swing out 100 degrees in this long
pub const PULL: f64 = 0.30; // how far a drawer slides out

/// The clothes' colours, in turn.
fn clothes(i: i64) -> Mat {
    [M.cloth1, M.cloth2, M.cloth3, M.cloth4][py::imod(i, 4) as usize]
}

/// A top `w` x `d`, `t` thick, its surface at `y`, its edges and corners rounded.
#[allow(clippy::too_many_arguments)]
pub fn top(name: &str, w: f64, t: f64, d: f64, y: f64, z: f64, material: Mat, r: f64) -> Obj {
    soft!(
        F,
        name,
        w,
        t,
        d,
        (0.0, y - t / 2.0, z),
        material,
        round = r.min(t * 0.45)
    )
}

/// What a front is.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Its edges rounded.
    Plain,
    /// A frame round a recessed panel.
    Shaker,
}

/// A drawer's or a door's front `w` x `h`, 0.02 thick, centred at game (x, y, z): plain (its edges rounded) or
/// shaker (a frame round a recessed panel).
#[allow(clippy::too_many_arguments)]
pub fn front(
    name: &str,
    w: f64,
    h: f64,
    x: impl Coord,
    y: f64,
    z: f64,
    material: Mat,
    style: Style,
    rail: Option<f64>,
) {
    if style == Style::Plain {
        soft!(
            F,
            name,
            w,
            h,
            0.02,
            (x, y, z),
            material,
            round = 0.006,
            round_segments = 1
        );
        return;
    }
    kit::shaker!(
        F,
        name,
        w,
        h,
        (x, y, z),
        0.02,
        material,
        rail = rail.unwrap_or(0.06_f64.min(h * 0.22)),
        raised = false
    );
}

/// A stack of `n` folded clothes, bottom at height `y`, each a soft fold, a little askew.
#[allow(clippy::too_many_arguments)]
pub fn folded(name: &str, x: f64, y: f64, z: f64, w: f64, d: f64, n: usize, h: f64) {
    for i in 0..n {
        let odd = (i % 2) as f64;
        soft!(
            F,
            format!("{name}{i}"),
            w - 0.01 * odd,
            h,
            d,
            (x + 0.005 * odd, y + h * (i as f64 + 0.5), z),
            clothes(i as i64 + py::int(x * 10.0)),
            round = 0.012,
            round_segments = 1,
            rot = (0, 0, 2 * ((i % 3) as i32 - 1))
        );
    }
}

/// A garment on a hanger, hung side-on from a rail at (rail_y, z): the wire hook over the rail and the hanger's
/// shoulders `half` either way, and the garment below them, `length` long, its shoulders sloping.
#[allow(clippy::too_many_arguments)]
fn hanger(name: &str, x: f64, rail_y: f64, z: f64, half: f64, length: f64, material: Mat) {
    let top = rail_y - 0.045;
    let hook = [
        (x, rail_y - 0.004, z + 0.016),
        (x, rail_y + 0.02, z + 0.006),
        (x, rail_y + 0.02, z - 0.008),
        (x, rail_y + 0.004, z - 0.018),
        (x, rail_y - 0.03, z),
        (x, top - 0.035, z - half),
        (x, top - 0.035, z + half),
        (x, rail_y - 0.03, z),
    ];
    fsweep!(
        F,
        format!("{name}_hanger"),
        &kit::fillet(&hook, 0.006, 2),
        0.0035,
        M.chrome,
        verts = 4
    );
    let hd = half + 0.015;
    let outline = [
        (-hd, top - 0.06),
        (-0.04, top),
        (0.04, top),
        (hd, top - 0.06),
        (hd * 0.94, top - length),
        (-hd * 0.94, top - length),
    ];
    fextrude!(
        F,
        name,
        &smooth_outline(&outline, 1, true),
        0.035,
        (x, 0, z),
        material,
        plane = Facing::Side,
        bevel = 0.008,
        bevel_segments = 1
    );
}

/// `n` garments on hangers along a rail at `rail_y` from x0 to x1, hung side-on (across the depth).
#[allow(clippy::too_many_arguments)]
pub fn hanging(
    name: &str,
    x0: f64,
    x1: f64,
    rail_y: f64,
    z: f64,
    n: usize,
    depth: f64,
    length: f64,
) {
    let step = (x1 - x0) / 1.max(n as i64 - 1) as f64;
    for i in 0..n {
        let long = length
            - 0.12 * f64::from(u8::from(i % 3 == 1))
            - 0.25 * f64::from(u8::from(i % 4 == 3));
        hanger(
            &format!("{name}{i}"),
            x0 + i as f64 * step,
            rail_y,
            z,
            depth / 2.0 - 0.02,
            long,
            clothes(i as i64),
        );
    }
}

/// The dark opening a drawer leaves in the carcass when it slides out, on the carcass face behind the drawer front
/// (part of the still body).
pub fn gap(name: &str, w: f64, h: f64, x: impl Coord, y: f64, front: f64, material: Mat) {
    fbox!(
        F,
        format!("{name}_gap"),
        w - 0.02,
        h - 0.02,
        0.002,
        (x, y, front + 0.001),
        material
    );
}

/// A drawer front of `w` x `h` whose back is at game depth `front` (the carcass face), centred at (x, y), with the
/// box behind it (inside the carcass at rest) and what's in it, as its own object with its origin on the carcass
/// face: for `drawer_open`. `pulls` makes its pulls.
#[allow(clippy::too_many_arguments)]
pub fn drawer<X: Coord>(
    name: &str,
    w: f64,
    h: f64,
    x: X,
    y: f64,
    front_z: f64,
    depth: f64,
    material: Mat,
    pulls: &dyn Fn(),
    contents: Option<&dyn Fn(f64, f64, f64)>,
    style: Style,
) -> Obj {
    let z = front_z + 0.01;
    let xf = x.f();
    let parts = new_parts(|| {
        front(
            &format!("{name}_front"),
            w,
            h,
            x,
            y,
            z,
            material,
            style,
            None,
        );
        pulls();
        let bh = h - 0.05; // the box's sides, below the front's top edge
        let by = y - h / 2.0 + 0.015 + bh / 2.0;
        let back = z - 0.01 - depth;
        fbox!(
            F,
            format!("{name}_bottom"),
            w - 0.05,
            0.012,
            depth,
            (x, y - h / 2.0 + 0.02, z - 0.01 - depth / 2.0),
            material
        );
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                format!("{name}_side{}", s as i32),
                0.014,
                bh,
                depth,
                (xf + s * (w / 2.0 - 0.032), by, z - 0.01 - depth / 2.0),
                material,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            F,
            format!("{name}_back"),
            w - 0.05,
            bh,
            0.014,
            (x, by, back + 0.007),
            material
        );
        if let Some(contents) = contents {
            contents(xf, y - h / 2.0 + 0.026, z - 0.01 - depth / 2.0);
        }
    });
    join(&parts, name, Some(F.at(x, y, front_z)))
}

/// An open-fronted carcass: two sides, a top, a bottom and a back, `t` thick.
#[allow(clippy::too_many_arguments)]
pub fn shell(w: f64, y0: f64, y1: f64, back: f64, front: f64, material: Mat, t: f64, name: &str) {
    let (d, zc, h, yc) = (front - back, (back + front) / 2.0, y1 - y0, (y0 + y1) / 2.0);
    for s in [-1, 1] {
        fbox!(
            F,
            format!("{name}_side{s}"),
            t,
            h,
            d,
            (s as f64 * (w - t) / 2.0, yc, zc),
            material,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    fbox!(
        F,
        format!("{name}_top"),
        w - 2.0 * t,
        t,
        d,
        (0, y1 - t / 2.0, zc),
        material,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        format!("{name}_bottom"),
        w - 2.0 * t,
        t,
        d,
        (0, y0 + t / 2.0, zc),
        material,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        format!("{name}_back"),
        w - 2.0 * t,
        h - 2.0 * t,
        t,
        (0, yc, back + t / 2.0),
        material
    );
}

/// A row of `n` books standing on a shelf at height `y`, from x0 rightwards, a band on each spine.
#[allow(clippy::too_many_arguments)]
pub fn books(name: &str, x0: f64, y: f64, z: f64, n: usize, h: f64, d: f64) {
    let mut x = x0;
    let mats = [M.teal, M.book2, M.book3];
    for i in 0..n {
        let t = 0.025 + 0.008 * (i % 3) as f64;
        let hh = h - 0.02 * (i % 2) as f64;
        fbox!(
            F,
            format!("{name}{i}"),
            t,
            hh,
            d,
            (x + t / 2.0, y + hh / 2.0, z),
            mats[i % 3],
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("{name}_band{i}"),
            t + 0.002,
            0.012,
            0.004,
            (x + t / 2.0, y + hh * 0.78, z + d / 2.0),
            mats[(i + 1) % 3]
        );
        x += t + 0.003;
    }
}

/// A closed book lying flat at game (x, y: its bottom, z): two boards, a spine on its left, the pages between.
#[allow(clippy::too_many_arguments)]
pub fn book(name: &str, x: f64, y: f64, z: f64, w: f64, t: f64, d: f64, cover: Mat) {
    for (k, yy) in [y + 0.002, y + t - 0.002].into_iter().enumerate() {
        fbox!(
            F,
            format!("{name}_board{k}"),
            w,
            0.004,
            d,
            (x, yy, z),
            cover,
            bevel = 0.0015,
            bevel_segments = 1
        );
    }
    fcyl!(
        F,
        format!("{name}_spine"),
        t / 2.0,
        d,
        (x - w / 2.0 + 0.004, y + t / 2.0, z),
        cover,
        rot = (90, 0, 0),
        verts = 8
    );
    fbox!(
        F,
        format!("{name}_pages"),
        w - 0.01,
        t - 0.006,
        d - 0.008,
        (x + 0.002, y + t / 2.0, z),
        M.paper
    );
}

/// A mug of coffee standing at game (x, y, z), its handle to the left.
pub fn mug(name: &str, x: f64, y: f64, z: f64) {
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (0.037, 0.0),
            (0.04, 0.005),
            (0.041, 0.1),
            (0.036, 0.1),
            (0.035, 0.012),
            (0.0, 0.012)
        ],
        at = F.at(x, y, z),
        material = M.cream,
        segments = 16
    );
    fcyl!(
        F,
        format!("{name}_coffee"),
        0.0355,
        0.004,
        (x, y + 0.083, z),
        M.coffee,
        verts = 16
    );
    fsweep!(
        F,
        format!("{name}_handle"),
        &[
            (x - 0.038, y + 0.078, z),
            (x - 0.062, y + 0.072, z),
            (x - 0.066, y + 0.05, z),
            (x - 0.06, y + 0.028, z),
            (x - 0.038, y + 0.024, z)
        ],
        0.006,
        M.cream,
        verts = 6,
        resolution = 2
    );
}
