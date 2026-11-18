//! The kit's parts the hobbies use (gamekit's kit.py), each marked `KIT:` until the kernel has them.

use super::common::P;
use artkit::forms::{Extrude, Loft, Scale, Sweep, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{P3, V3, pow, tan};
use artkit::place::{Coord, GameFrame, Spot};
use artkit::py::sum;
use artkit::scene::Obj;
use artkit::shapes::{Cube, Revolve, revolve};

/// `tube`'s options.
pub struct Tube {
    pub verts: Option<usize>,
    pub round: usize,
    pub scale: Scale,
}

impl Default for Tube {
    fn default() -> Tube {
        Tube {
            verts: None,
            round: 3,
            scale: Scale::One(1.0),
        }
    }
}

// KIT: kit.tube
/// A round tube of radius `r` through game points, its bends rounded (`round` steps between each pair of points;
/// 1 keeps them sharp), its ends capped: a towel rail, a bent-tube frame, a tap's riser, a hose. `verts` round: by
/// default 8, and more for one over 13 mm in radius, as many as keep its outline within a millimetre of the circle.
pub fn tube<S: Spot>(
    frame: &GameFrame,
    name: impl AsRef<str>,
    points: &[S],
    r: f64,
    material: Mat,
    o: Tube,
) -> Obj {
    let s = Sweep {
        verts: o.verts,
        resolution: o.round,
        scale: o.scale,
        least: 8,
        ..Sweep::default()
    };
    frame.sweep(name, points, r, material, s)
}

// KIT: kit.fillet
/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round it, on a curve
/// that never overshoots, however uneven the spacing). For handles, bent rails and draped cloth swept with
/// `resolution` 1.
// (3.1415 is the kit's own limit for a straight corner, not pi.)
#[allow(clippy::approx_constant)]
pub fn fillet(points: &[P], radius: f64, steps: usize) -> Vec<P> {
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

// KIT: kit.knob
/// A turned knob on a front at game depth `face`: a rose against it, a waisted neck and a domed head `r` round.
#[allow(clippy::too_many_arguments)] // (as the kit called it)
pub fn knob(
    frame: &GameFrame,
    name: &str,
    x: impl Coord,
    y: f64,
    face: f64,
    r: f64,
    material: Mat,
    segments: usize,
) -> Obj {
    let prof = [
        (0.0, 0.0),
        (r * 0.75, 0.0),
        (r * 0.7, r * 0.15),
        (r * 0.36, r * 0.35),
        (r * 0.42, r * 0.75),
        (r * 0.95, r * 1.05),
        (r, r * 1.35),
        (r * 0.8, r * 1.7),
        (r * 0.4, r * 1.88),
        (0.0, r * 1.92),
    ];
    revolve(
        name,
        &prof,
        Revolve {
            at: frame.at(x, y, face),
            material: Some(material),
            segments: Some(segments),
            rot: (90, 0, 0).into(),
            ..Revolve::default()
        },
    )
}

// KIT: kit.shaker
/// A frame-and-panel front `w` x `h`, `t` thick, centred at game `at`, facing the front (a cabinet's door, a
/// drawer's front): a frame `rail` wide round a panel set back `recess`, flat, or `raised` (its edges chamfered
/// up to a field); `panel`: the panel's own material. Returns (frame, panel).
#[allow(clippy::too_many_arguments)] // (as the kit called it)
pub fn shaker<A: Coord, B: Coord, C: Coord>(
    frame: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    at: (A, B, C),
    t: f64,
    material: Mat,
    rail: f64,
    panel: Option<Mat>,
    recess: f64,
    raised: bool,
) -> (Obj, Obj) {
    let hole = rounded_rect(w - 2.0 * rail, h - 2.0 * rail, 0.003, 1);
    let o = Extrude {
        holes: artkit::forms::Holes(vec![hole]),
        bevel: if t * 0.2 < 0.004 { t * 0.2 } else { 0.004 },
        bevel_segments: 1,
        ..Extrude::default()
    };
    let fr = frame.extrude(name, &rounded_rect(w, h, 0.003, 1), t, at, material, o);
    // (a hair inside the opening, not into the frame: a vertex buried in it would bake black, shade.rs)
    let (pw, ph) = (w - 2.0 * rail - 0.002, h - 2.0 * rail - 0.002);
    if !raised {
        let o = Cube {
            bevel: 0.002,
            bevel_segments: 1,
            ..Cube::default()
        };
        return (
            fr,
            frame.box_(
                format!("{name}_panel"),
                pw,
                ph,
                t - 2.0 * recess,
                at,
                panel.unwrap_or(material),
                o,
            ),
        );
    }
    // a raised panel: a thin edge inside the frame, rising over a wide shallow chamfer to a flat field
    let (x, y, z) = (at.0.f(), at.1.f(), at.2.f());
    let b = py_min(py_min(rail * 0.7, pw * 0.25), ph * 0.25);
    let lp = |hw: f64, hh: f64, d: f64| -> Vec<P3> {
        [(hw, hh), (-hw, hh), (-hw, -hh), (hw, -hh)]
            .iter()
            .map(|&(u, v)| frame.at(x + u, y + v, d))
            .collect()
    };
    let front = z + t / 2.0;
    let pn = artkit::forms::loft(
        format!("{name}_panel"),
        &[
            lp(pw / 2.0, ph / 2.0, z - t / 2.0 + 0.002),
            lp(pw / 2.0, ph / 2.0, front - recess - 0.004),
            lp(pw / 2.0 - b, ph / 2.0 - b, front - recess * 0.2),
        ],
        Loft {
            material: Some(panel.unwrap_or(material)),
            smooth: false,
            ..Loft::default()
        },
    );
    (fr, pn)
}

/// Python's `min` of two: the first unless the second is less.
fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Where a moulding is run: round an outline of game (x, height) at a depth, or of (x, depth) at a height.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Run {
    Front,
    Top,
}

/// `moulding`'s options.
pub struct Moulding {
    pub at: f64,
    pub plane: Run,
    pub angle: f64,
}

impl Default for Moulding {
    fn default() -> Moulding {
        Moulding {
            at: 0.0,
            plane: Run::Front,
            angle: artkit::shapes::SHARP,
        }
    }
}

// KIT: kit.moulding
/// A moulding run round a closed `outline` with mitred corners: a picture or mirror frame, a panel's bead, a
/// plinth's or a cornice's band. `outline` is game (x, height) points drawn at depth `at` (`Run::Front`), or (x, depth)
/// points at height `at` (`Run::Top`); `profile` is the moulding's closed cross-section [(inset, out), ...]: `inset`
/// metres in from the outline, `out` metres forward of `at` (front) or up from it (top). The corners stay square however
/// wide the profile (a sweep pinches them).
#[allow(clippy::too_many_arguments)] // (as the kit called it)
pub fn moulding(
    frame: &GameFrame,
    name: &str,
    outline: &[(f64, f64)],
    profile: &[(f64, f64)],
    material: Mat,
    at: f64,
    plane: Run,
    angle: f64,
) -> Obj {
    let n = outline.len();
    let area = sum((0..n)
        .map(|i| outline[i].0 * outline[(i + 1) % n].1 - outline[(i + 1) % n].0 * outline[i].1));
    // inward is to the left of an anticlockwise outline
    let turn = if area > 0.0 { 1.0 } else { -1.0 };
    let inward = |a: (f64, f64), b: (f64, f64)| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let length = pow(dx * dx + dy * dy, 0.5);
        let length = if length == 0.0 { 1.0 } else { length };
        (-dy / length * turn, dx / length * turn)
    };
    let mut vs: Vec<V3> = Vec::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();
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
            vs.push(
                if plane == Run::Front {
                    frame.at(u, v, at + out)
                } else {
                    frame.at(u, at + out, v)
                }
                .into(),
            );
        }
    }
    for i in 0..n {
        let (a, b) = ((i * m) as u32, (((i + 1) % n) * m) as u32);
        for j in 0..m as u32 {
            let j2 = (j + 1) % m as u32;
            faces.push(vec![a + j, a + j2, b + j2, b + j]);
        }
    }
    artkit::forms::object(name, vs, &faces, Some(material), true, angle, false)
}

/// The kit's parts called as the scripts called them, their options by name.
pub mod call {
    /// `tube` with its options by name, as the scripts called it: `tube!(F, name, points, r, material, verts = 8, round = 1)`.
    macro_rules! tube {
        ($frame:expr, $name:expr, $points:expr, $r:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
            #[allow(unused_mut)]
            let mut o = $crate::hobbies::kit::Tube::default();
            $(o.$k = artkit::Arg::arg($v);)*
            $crate::hobbies::kit::tube(&$frame, $name, $points, $r, $material, o)
        }};
    }
    pub(crate) use tube;

    /// `moulding` with its options by name: `moulding!(F, name, outline, profile, material, at = 0.1, plane = Run::Top)`.
    macro_rules! moulding {
        ($frame:expr, $name:expr, $outline:expr, $profile:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
            #[allow(unused_mut)]
            let mut o = $crate::hobbies::kit::Moulding::default();
            $(o.$k = artkit::Arg::arg($v);)*
            $crate::hobbies::kit::moulding(&$frame, $name, $outline, $profile, $material, o.at, o.plane, o.angle)
        }};
    }
    pub(crate) use moulding;
}
