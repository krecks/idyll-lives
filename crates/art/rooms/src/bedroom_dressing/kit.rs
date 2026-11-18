//! The kit's parts these rooms use (gamekit's kit.py): bent tube, pulls and knobs, frame-and-panel fronts, mouldings
//! mitred round an outline. Each takes a `GameFrame` (game axes: x, height, depth) and the family's material, and
//! adds its mesh; called as the scripts called them, the options by name: `kit::knob!(F, "knob", x, y, face, r, M.brass)`.
// (parts the other rooms take as they are ported)
#![allow(dead_code)]

use artkit::forms::{self, Extrude, Holes, Loft, Scale, Sweep, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{Num, P3, V3, pow, tan};
use artkit::place::{Coord, Facing, GameFrame, Spot};
use artkit::py;
use artkit::scene::Obj;
use artkit::shapes::{self, Cube, Revolve, SHARP};

/// Python's `min` of floats: the first of the least.
fn least(xs: &[f64]) -> f64 {
    let mut m = xs[0];
    for &x in &xs[1..] {
        if x < m {
            m = x;
        }
    }
    m
}

/// Python's `max` of two floats: the first unless the second is greater.
fn most(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

// KIT: kit.tube
/// `tube`'s options.
#[derive(Clone, Debug)]
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

/// A round tube of radius `r` through game points, its bends rounded (`round` steps between each pair of points;
/// 1 keeps them sharp), its ends capped: a towel rail, a bent-tube frame, a tap's riser, a hose. `verts` round: by
/// default 8, and more for one over 13 mm in radius, as many as keep its outline within a millimetre of the circle
/// (`forms::sweep`).
pub fn tube<P: Spot>(
    frame: &GameFrame,
    name: impl AsRef<str>,
    points: &[P],
    r: f64,
    material: Mat,
    o: Tube,
) -> Obj {
    frame.sweep(
        name,
        points,
        r,
        material,
        Sweep {
            verts: o.verts,
            resolution: o.round,
            scale: o.scale,
            least: 8,
            ..Sweep::default()
        },
    )
}

/// `tube` with its options by name: `kit::tube!(frame, name, points, r, material, key = value, ...)`.
#[allow(unused_macros)]
macro_rules! kit_tube {
    ($frame:expr, $name:expr, $points:expr, $r:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $points, $r, $material);
        #[allow(unused_mut)]
        let mut o = $crate::bedroom_dressing::kit::Tube::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::bedroom_dressing::kit::tube(args.0, args.1, args.2, artkit::forms::num(args.3), args.4, o)
    }};
}
#[allow(unused_imports)]
pub(crate) use kit_tube as tube;

// KIT: kit.fillet
/// A point of a path to bend (`fillet`): a tuple of three numbers or a vector, or a tuple of two for a path drawn
/// flat. The bent path's points are tuples of as many floats.
pub trait Corner: Copy {
    type Out;
    /// As a `mathutils.Vector` (one of two with no third part: its sums come out the same).
    fn vector(self) -> V3;
    /// `tuple(vector)`.
    fn tuple(v: V3) -> Self::Out;
}
impl<A: Num, B: Num, C: Num> Corner for (A, B, C) {
    type Out = (f64, f64, f64);
    fn vector(self) -> V3 {
        self.into()
    }
    fn tuple(v: V3) -> Self::Out {
        (v.x as f64, v.y as f64, v.z as f64)
    }
}
impl<A: Num, B: Num> Corner for (A, B) {
    type Out = (f64, f64);
    fn vector(self) -> V3 {
        (self.0, self.1, 0.0).into()
    }
    fn tuple(v: V3) -> Self::Out {
        (v.x as f64, v.y as f64)
    }
}

/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
/// it, on a curve that never overshoots, however uneven the spacing: Catmull-Rom loops on a short leg beside a long
/// one). For handles, bent rails and draped cloth swept with `resolution` 1.
// (3.1415 is the script's own limit for a straight corner, not pi.)
#[allow(clippy::approx_constant)]
pub fn fillet<P: Corner>(points: &[P], radius: f64, steps: usize) -> Vec<P::Out> {
    let pts: Vec<V3> = points.iter().map(|&p| p.vector()).collect();
    let mut out = vec![pts[0]];
    for corner in pts.windows(3) {
        let (a, p, c) = (corner[0], corner[1], corner[2]);
        let (d1, d2) = (a - p, c - p);
        if d1.length() < 1e-9 || d2.length() < 1e-9 || d1.angle(d2) > 3.1415 {
            out.push(p);
            continue;
        }
        let t = least(&[
            radius / most(1e-6, tan(d1.angle(d2) / 2.0)),
            d1.length() * 0.5,
            d2.length() * 0.5,
        ]);
        let (p1, p2) = (p + d1.normalized() * t, p + d2.normalized() * t);
        for i in 0..=steps {
            let k = i as f64 / steps as f64;
            out.push(p1 * pow(1.0 - k, 2.0) + p * 2 * k * (1.0 - k) + p2 * k * k);
        }
    }
    out.push(pts[pts.len() - 1]);
    out.into_iter().map(P::tuple).collect()
}

// KIT: kit.pull
/// `pull`'s options.
#[derive(Clone, Debug)]
pub struct Pull {
    pub r: f64,
    pub out: f64,
    pub upright: bool,
    pub verts: usize,
}

impl Default for Pull {
    fn default() -> Pull {
        Pull {
            r: 0.006,
            out: 0.03,
            upright: false,
            verts: 8,
        }
    }
}

/// A bar pull in one piece: out from a front at game depth `face` on two round stand-offs `length` apart (across
/// it, or `upright`), its bends rounded: a drawer's or a door's handle.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn pull(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x: f64,
    y: f64,
    face: f64,
    length: f64,
    material: Mat,
    o: Pull,
) -> Obj {
    let (h, b) = (length / 2.0, least(&[0.012, o.out * 0.45]));
    let ends = [
        (-h, face - 0.003),
        (-h, face + o.out),
        (h, face + o.out),
        (h, face - 0.003),
    ];
    let pts = ends.map(|(u, z)| {
        if o.upright {
            (x, y + u, z)
        } else {
            (x + u, y, z)
        }
    });
    frame.sweep(
        name,
        &fillet(&pts, b, 3),
        o.r,
        material,
        Sweep {
            verts: Some(o.verts),
            ..Sweep::default()
        },
    )
}

/// `pull` with its options by name: `kit::pull!(frame, name, x, y, face, length, material, key = value, ...)`.
#[allow(unused_macros)]
macro_rules! kit_pull {
    ($frame:expr, $name:expr, $x:expr, $y:expr, $face:expr, $length:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $x, $y, $face, $length, $material);
        #[allow(unused_mut)]
        let mut o = $crate::bedroom_dressing::kit::Pull::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::bedroom_dressing::kit::pull(args.0, args.1, artkit::forms::num(args.2), artkit::forms::num(args.3), artkit::forms::num(args.4), artkit::forms::num(args.5), args.6, o)
    }};
}
#[allow(unused_imports)]
pub(crate) use kit_pull as pull;

// KIT: kit.knob
/// A turned knob on a front at game depth `face`: a rose against it, a waisted neck and a domed head `r` round
/// (`segments` round: 12 in the scripts).
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn knob(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x: impl Coord,
    y: impl Coord,
    face: impl Coord,
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
    shapes::revolve(
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

/// `knob` as the scripts called it: `kit::knob!(frame, name, x, y, face, r, material)`.
#[allow(unused_macros)]
macro_rules! kit_knob {
    ($frame:expr, $name:expr, $x:expr, $y:expr, $face:expr, $r:expr, $material:expr $(,)?) => {
        $crate::bedroom_dressing::kit::knob(
            &$frame,
            $name,
            $x,
            $y,
            $face,
            artkit::forms::num($r),
            $material,
            12,
        )
    };
}
#[allow(unused_imports)]
pub(crate) use kit_knob as knob;

// KIT: kit.shaker
/// `shaker`'s options (`panel`: the panel's own material).
#[derive(Clone, Debug)]
pub struct Shaker {
    pub rail: f64,
    pub panel: Option<Mat>,
    pub recess: f64,
    pub raised: bool,
}

impl Default for Shaker {
    fn default() -> Shaker {
        Shaker {
            rail: 0.06,
            panel: None,
            recess: 0.005,
            raised: false,
        }
    }
}

/// A frame-and-panel front `w` x `h`, `t` thick, centred at game `at`, facing the front (a cabinet's door, a
/// drawer's front): a frame `rail` wide round a panel set back `recess`, flat, or `raised` (its edges chamfered up
/// to a field); `panel`: the panel's own material. Returns (frame, panel).
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn shaker(
    frame: &GameFrame,
    name: impl AsRef<str>,
    w: f64,
    h: f64,
    at: (impl Coord, impl Coord, impl Coord),
    t: f64,
    material: Mat,
    o: Shaker,
) -> (Obj, Obj) {
    let name = name.as_ref();
    let (rail, recess) = (o.rail, o.recess);
    let hole = rounded_rect(w - 2.0 * rail, h - 2.0 * rail, 0.003, 1);
    let fr = frame.extrude(
        name,
        &rounded_rect(w, h, 0.003, 1),
        t,
        at,
        material,
        Extrude {
            holes: Holes(vec![hole]),
            bevel: least(&[0.004, t * 0.2]),
            bevel_segments: 1,
            ..Extrude::default()
        },
    );
    // (a hair inside the opening, not into the frame: a vertex buried in it would bake black, `shade`)
    let (pw, ph) = (w - 2.0 * rail - 0.002, h - 2.0 * rail - 0.002);
    let panel = o.panel.unwrap_or(material);
    if !o.raised {
        let pn = frame.box_(
            format!("{name}_panel"),
            pw,
            ph,
            t - 2.0 * recess,
            at,
            panel,
            Cube {
                bevel: 0.002,
                bevel_segments: 1,
                ..Cube::default()
            },
        );
        return (fr, pn);
    }
    // a raised panel: a thin edge inside the frame, rising over a wide shallow chamfer to a flat field
    let (x, y, z) = (at.0.f(), at.1.f(), at.2.f());
    let b = least(&[rail * 0.7, pw * 0.25, ph * 0.25]);
    let ring = |hw: f64, hh: f64, d: f64| -> Vec<P3> {
        [(hw, hh), (-hw, hh), (-hw, -hh), (hw, -hh)]
            .iter()
            .map(|&(u, v)| frame.at(x + u, y + v, d))
            .collect()
    };
    let front = z + t / 2.0;
    let pn = forms::loft(
        format!("{name}_panel"),
        &[
            ring(pw / 2.0, ph / 2.0, z - t / 2.0 + 0.002),
            ring(pw / 2.0, ph / 2.0, front - recess - 0.004),
            ring(pw / 2.0 - b, ph / 2.0 - b, front - recess * 0.2),
        ],
        Loft {
            material: Some(panel),
            smooth: false,
            ..Loft::default()
        },
    );
    (fr, pn)
}

/// `shaker` with its options by name: `kit::shaker!(frame, name, w, h, at, t, material, key = value, ...)`.
#[allow(unused_macros)]
macro_rules! kit_shaker {
    ($frame:expr, $name:expr, $w:expr, $h:expr, $at:expr, $t:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $w, $h, $at, $t, $material);
        #[allow(unused_mut)]
        let mut o = $crate::bedroom_dressing::kit::Shaker::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::bedroom_dressing::kit::shaker(args.0, args.1, artkit::forms::num(args.2), artkit::forms::num(args.3), args.4, artkit::forms::num(args.5), args.6, o)
    }};
}
#[allow(unused_imports)]
pub(crate) use kit_shaker as shaker;

// KIT: kit.moulding
/// `moulding`'s options (`plane`: `Facing::Front` or `Facing::Top`).
#[derive(Clone, Debug)]
pub struct Moulding {
    pub at: f64,
    pub plane: Facing,
    pub angle: f64,
}

impl Default for Moulding {
    fn default() -> Moulding {
        Moulding {
            at: 0.0,
            plane: Facing::Front,
            angle: SHARP,
        }
    }
}

/// A moulding run round a closed `outline` with mitred corners: a picture or mirror frame, a panel's bead, a
/// plinth's or a cornice's band. `outline` is game (x, height) points drawn at depth `at` (`Facing::Front`), or
/// (x, depth) points at height `at` (`Facing::Top`); `profile` is the moulding's closed cross-section
/// [(inset, out), ...]: `inset` metres in from the outline, `out` metres forward of `at` (front) or up from it
/// (top). The corners stay square however wide the profile (a sweep pinches them).
pub fn moulding(
    frame: &GameFrame,
    name: impl AsRef<str>,
    outline: &[(f64, f64)],
    profile: &[(f64, f64)],
    material: Mat,
    o: Moulding,
) -> Obj {
    let n = outline.len();
    let area =
        py::sum((0..n).map(|i| {
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
    let m = profile.len();
    let mut vs: Vec<V3> = Vec::with_capacity(n * m);
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
            let at = if o.plane == Facing::Front {
                frame.at(u, v, o.at + out)
            } else {
                frame.at(u, o.at + out, v)
            };
            vs.push(at.into());
        }
    }
    let mut faces: Vec<Vec<u32>> = Vec::with_capacity(n * m);
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
    forms::object(
        name.as_ref(),
        vs,
        &faces,
        Some(material),
        true,
        o.angle,
        false,
    )
}

/// `moulding` with its options by name: `kit::moulding!(frame, name, outline, profile, material, key = value, ...)`;
/// `plane = Facing::Top` for the script's `plane='top'`.
#[allow(unused_macros)]
macro_rules! kit_moulding {
    ($frame:expr, $name:expr, $outline:expr, $profile:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $outline, $profile, $material);
        #[allow(unused_mut)]
        let mut o = $crate::bedroom_dressing::kit::Moulding::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::bedroom_dressing::kit::moulding(args.0, args.1, args.2, args.3, args.4, o)
    }};
}
#[allow(unused_imports)]
pub(crate) use kit_moulding as moulding;
