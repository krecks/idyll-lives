//! Joinery and the frames built like it: paths bent round their corners, frame-and-panel fronts, mouldings mitred
//! round an outline, and a bench's cast ends.

use crate::forms::{self, Extrude, Holes, Loft, Profile, Scale, Sweep, rounded_rect};
use crate::json::float_repr;
use crate::kit::{Scroll, least, most, scroll};
use crate::materials::Mat;
use crate::math::{Num, P3, PI, V3, pow, sin, tan};
use crate::place::{Coord, Facing, GameFrame};
use crate::py;
use crate::scene::Obj;
use crate::shapes::{Cube, SHARP, options};

/// A point of a path to bend (`fillet`): a tuple of three numbers or a vector, or a tuple of two for a path drawn
/// flat (an outline, a lathe's profile). The bent path's points are tuples of as many floats.
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
impl Corner for V3 {
    type Out = (f64, f64, f64);
    fn vector(self) -> V3 {
        self
    }
    fn tuple(v: V3) -> Self::Out {
        (v.x as f64, v.y as f64, v.z as f64)
    }
}
impl Corner for P3 {
    type Out = (f64, f64, f64);
    fn vector(self) -> V3 {
        self.into()
    }
    fn tuple(v: V3) -> Self::Out {
        (v.x as f64, v.y as f64, v.z as f64)
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

options! {
    /// `shaker`'s options (`panel`: the panel's own material).
    Shaker { rail: f64 = 0.06, panel: Option<Mat> = None, recess: f64 = 0.005, raised: bool = false }
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

options! {
    /// `moulding`'s options (`plane`: `Facing::Front` or `Facing::Top`).
    Moulding { at: f64 = 0.0, plane: Facing = Facing::Front, angle: f64 = SHARP }
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

options! {
    /// `bench_end`'s options.
    BenchEnd { arm: Option<f64> = None, t: f64 = 0.026, w: f64 = 0.045, foot: f64 = 0.06, steps: usize = 3, corner: usize = 1, scrolls: bool = true }
}

/// A cast-iron bench end in one plane at game x, each member a flat bar `t` thick and `w` deep: the back leg
/// rising in one curve into the upright, leaning back to `top` (height, depth); the front leg, straight up and,
/// with an `arm` height, bent back into the armrest that meets the upright, its front curling down in a scroll;
/// the seat rail between them at `seat` (its top, under the slats); pad feet `foot` long at `front` and `back`
/// (their depths). `steps` rounds the curves (1 for a far version), `corner` the bars' edges (0: square),
/// `scrolls` the curls.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn bench_end<X: Coord>(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x: X,
    seat: f64,
    front: f64,
    back: f64,
    top: (impl Num, impl Num),
    material: Mat,
    o: BenchEnd,
) -> Vec<Obj> {
    let name = name.as_ref();
    let (t, w, steps) = (o.t, o.w, o.steps);
    let prof = rounded_rect(
        t,
        w,
        if o.corner != 0 {
            least(&[t, w]) * 0.3
        } else {
            0.0
        },
        o.corner.max(1),
    );
    let (ty, tz) = (top.0.f(), top.1.f());
    let rail = seat - w / 2.0;
    let bz = back + 0.045; // where the upright passes the seat rail
    let mut made = Vec::new();
    // a bar of the profile through game points at x
    let bar = |name: String, pts: &[P3], resolution: usize, scale: f64| {
        forms::sweep(
            name,
            pts,
            0.0,
            Sweep {
                material: Some(material),
                profile: Profile(Some(prof.clone())),
                up: P3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                resolution,
                scale: Scale::One(scale),
                ..Sweep::default()
            },
        )
    };
    // the back leg into the upright: evenly spaced points, curved through (no corner to overshoot)
    let pts = [
        frame.at(x, 0.0, back),
        frame.at(x, seat * 0.5, back + 0.03),
        frame.at(x, rail, bz),
        frame.at(x, rail + (ty - rail) * 0.5, (bz + tz) / 2.0 + 0.012),
        frame.at(x, ty, tz),
    ];
    made.push(bar(format!("{name}_back"), &pts, steps, 1.0));

    // the upright's depth at height y (above the seat), for the arm to meet
    let upright_z = |y: f64| {
        let f = (y - rail) / (ty - rail);
        bz + (tz - bz) * f + 0.012 * sin(PI * f)
    };
    let fz = front - 0.015;
    match o.arm {
        None => {
            let pts = [frame.at(x, 0.0, front), frame.at(x, rail, fz)];
            made.push(bar(format!("{name}_front"), &pts, 1, 1.0));
        }
        Some(arm) => {
            let pts = fillet(
                &[
                    (x.f(), 0.0, front),
                    (x.f(), rail, fz),
                    (x.f(), arm - w / 2.0, fz - 0.005),
                    (x.f(), arm - w / 2.0, upright_z(arm) + w / 2.0),
                ],
                0.05,
                steps.max(1),
            );
            let pts: Vec<P3> = pts.iter().map(|p| frame.at(p.0, p.1, p.2)).collect();
            made.push(bar(format!("{name}_front"), &pts, 1, 1.0));
            if o.scrolls {
                let r = 0.038;
                made.push(scroll(
                    frame,
                    format!("{name}_scroll"),
                    (x, arm - w / 2.0, fz - 0.012),
                    (arm - w / 2.0 - r, fz - 0.012),
                    material,
                    Scroll {
                        turns: -1.2,
                        t,
                        w: w * 0.75,
                        steps: 4 * steps.max(1),
                        ..Scroll::default()
                    },
                ));
            }
        }
    }
    if o.scrolls {
        // a stretcher sagging between the legs, and a curl in the angle of the front leg and the seat rail
        let pts = [
            frame.at(x, 0.13, front - 0.01),
            frame.at(x, 0.1, (front + back) / 2.0),
            frame.at(x, 0.13, back + 0.02),
        ];
        made.push(bar(format!("{name}_stretcher"), &pts, steps, 0.75));
        made.push(scroll(
            frame,
            format!("{name}_bracket"),
            (x, rail - w * 0.6, fz - w * 0.4),
            (rail - w * 0.6 - 0.035, fz - 0.06),
            material,
            Scroll {
                turns: 0.85,
                t,
                w: w * 0.6,
                steps: 4 * steps.max(1),
                ..Scroll::default()
            },
        ));
    }
    let pts = [frame.at(x, rail, fz + 0.01), frame.at(x, rail, bz - 0.01)];
    made.push(bar(format!("{name}_rail"), &pts, 1, 1.0));
    for z in [front, back] {
        made.push(frame.box_(
            format!("{name}_foot{}", float_repr(z)),
            t + 0.02,
            0.02,
            o.foot,
            (x, 0.01, z),
            material,
            Cube {
                bevel: if o.corner != 0 { 0.006 } else { 0.0 },
                bevel_segments: 1,
                ..Cube::default()
            },
        ));
    }
    made
}
