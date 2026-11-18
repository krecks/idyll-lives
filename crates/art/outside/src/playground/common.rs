//! What the playground is built from: its palette, its frame, and the few shapes it makes itself.

use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{V3, pow, tan};
use artkit::mesh::Mesh;
use artkit::palette;
use artkit::place::{Coord, Facing, GameFrame};
use artkit::scene::{self, Obj};
use artkit::shapes::{self, CLOSED};

palette! { M:
    body = ("#2E6FA8", metal),         // painted steel frames: the slide's tower, the seesaw's stand
    chute = ("#F2C14E", gloss),        // the slide's chute
    roof = ("#C8402F", gloss),         // the slide's roof
    tread = ("#C9A06A", wood),         // the slide's deck and ladder rungs
    rail = ("#C3C7CB", metal),         // galvanised rails, handles and poles
    steel = ("#5C6168", metal),        // axles, brackets, hubs, base plates
    rubber = ("#2A2C2E", matte),       // bumpers, grips, the ball's seams
    plank = ("#F2C14E", gloss),        // the seesaw's plank
    seat = ("#D9483B", gloss),         // the seesaw's seats
    horse = ("#D9483B", gloss),        // the spring rider's horse
    saddle = ("#2E6FA8", gloss),       // its saddle
    mane = ("#F2F0EA", matte),         // its mane and tail
    eye = ("#1A1A1A", gloss),
    grip = ("#F2C14E", gloss),         // the spring rider's handles and foot pegs
    spring = ("#3C6B3A", metal),       // the spring rider's coil
    platform = ("#C8402F", gloss),     // the merry-go-round's deck
    deck = ("#F2C14E", gloss),         // its stripes
    timber = ("#C49A62", wood),        // the climbing frame
    rope = ("#D8C9A6", fabric),        // its net
    shingle = ("#B5463A", satin),      // its roof
    flag = ("#F2C14E", satin),
    post = ("#3E4347", metal),         // the hoop's post, arm and ballast
    board = ("#F4F4F2", gloss),        // the backboard
    board_mark = ("#C8402F", gloss),   // its border and target square
    rim = ("#E0602A", gloss),          // the hoop
    net = ("#F2F2F0", fabric),         // the net (its own part: one draw)
    ball = ("#D86A2A", matte),
    table_top = ("#2F6B4F", satin),    // the ping-pong table's top
    lines = ("#F2F2F0", satin),        // its white lines
    concrete = ("#A9ADB0", matte),     // its legs
    ping = ("#F7F5EE", gloss),         // the ping-pong ball
    stone = ("#C9C2B4", matte),        // the chess table's top
    stool = ("#8E8C88", matte),        // its pedestal and stools
    square_light = ("#E8DFC8", satin),
    square_dark = ("#3A3330", satin),
    piece_white = ("#F2EDE2", gloss),
    piece_black = ("#2A2624", gloss),
}

pub const F: GameFrame = GameFrame::new(0.0);
pub const X: V3 = V3::new(1.0, 0.0, 0.0);

/// A float in a part's name, as Python writes it.
pub fn fr(x: f64) -> String {
    float_repr(x)
}

/// A number as a script wrote it, whole or not: a part's name and the sign of a negated zero follow it.
#[derive(Clone, Copy, Debug)]
pub enum N {
    I(i32),
    F(f64),
}

impl Coord for N {
    fn f(self) -> f64 {
        match self {
            N::I(i) => i as f64,
            N::F(x) => x,
        }
    }
    fn negated(self) -> f64 {
        match self {
            N::I(i) => i.negated(),
            N::F(x) => -x,
        }
    }
}

impl std::fmt::Display for N {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            N::I(i) => write!(f, "{i}"),
            N::F(x) => write!(f, "{}", float_repr(*x)),
        }
    }
}

// KIT: forms._object (artkit's forms::object is the crate's own)
/// A part of the points `vs` and `faces`, its faces turned to look outwards, shaded.
pub fn object(
    name: &str,
    vs: Vec<V3>,
    faces: &[Vec<u32>],
    material: Mat,
    smooth: bool,
    angle: f64,
) -> Obj {
    let mut mesh = Mesh::from_pydata(vs, faces);
    mesh.set_material(Some(material));
    mesh.recalc_normals();
    shapes::shade(scene::add(name, mesh), smooth, angle)
}

/// A mesh object from points and faces (Blender world), its normals pointing out.
pub fn mesh(name: &str, verts: Vec<V3>, faces: &[Vec<u32>], material: Mat, smooth: bool) -> Obj {
    object(name, verts, faces, material, smooth, CLOSED)
}

// KIT: kit.fillet
/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
/// it, on a curve that never overshoots, however uneven the spacing: Catmull-Rom loops on a short leg beside a long
/// one). For handles, bent rails and draped cloth swept with `resolution` 1.
#[allow(clippy::approx_constant)] // (the script's 3.1415, not pi)
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

// KIT: kit.moulding
/// A moulding run round a closed `outline` with mitred corners: a picture or mirror frame, a panel's bead, a
/// plinth's or a cornice's band. `outline` is game (x, height) points drawn at depth `at` (front), or (x, depth)
/// points at height `at` (top); `profile` is the moulding's closed cross-section [(inset, out), ...]: `inset`
/// metres in from the outline, `out` metres forward of `at` (front) or up from it (top). The corners stay
/// square however wide the profile (a sweep pinches them).
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn moulding(
    frame: &GameFrame,
    name: &str,
    outline: &[(f64, f64)],
    profile: &[(f64, f64)],
    material: Mat,
    at: f64,
    plane: Facing,
    angle: f64,
) -> Obj {
    let n = outline.len();
    let area =
        artkit::py::sum((0..n).map(|i| {
            outline[i].0 * outline[(i + 1) % n].1 - outline[(i + 1) % n].0 * outline[i].1
        }));
    let turn = if area > 0.0 { 1.0 } else { -1.0 }; // inward is to the left of an anticlockwise outline
    let inward = |a: (f64, f64), b: (f64, f64)| {
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
            vs.push(V3::from(if plane == Facing::Front {
                frame.at(u, v, at + out)
            } else {
                frame.at(u, at + out, v)
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
    object(name, vs, &faces, material, true, angle)
}
