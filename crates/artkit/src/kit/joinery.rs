//! Joinery and the frames built like it: paths bent round their corners, frame-and-panel fronts, mouldings mitred
//! round an outline, and a bench's cast ends.

use crate::bevel::EdgeOrder;
use crate::forms::{self, Extrude, Holes, Loft, Profile, Scale, Sweep, rounded_rect};
use crate::json::float_repr;
use crate::kit::{Scroll, least, most, scroll};
use crate::materials::Mat;
use crate::math::{Num, P3, PI, V3, pow, sin, tan};
use crate::mesh::{Mesh, edge, normal_newell};
use crate::place::{Coord, Facing, GameFrame};
use crate::py;
use crate::scene::Obj;
use crate::shapes::{Cube, SHARP, options};
use std::collections::BTreeMap;

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

// --- cuts ------------------------------------------------------------------------------------------------------

options! {
    /// `cut`'s options. `lined`: the faces the cutter leaves in the body take the cutter's material (the boolean
    /// modifier's TRANSFER: an oven's enamel, a fridge's liner); without, the body's own, as `forms.cut` left them.
    /// `keep_cutter`: the cutter stays in the scene, for the build to take away itself (`forms.cut`'s own option,
    /// and a script that set the modifier up by hand).
    Cut { lined: bool = false, keep_cutter: bool = false }
}

/// Cuts `cutter`'s volume out of `obj` and takes the cutter away: what gamekit's `forms.cut` and the kitchen's
/// boolean modifiers (Blender's exact difference) made of the families' bodies, without a boolean. One of two
/// cuts, by how the cutter lies in the body's own box:
///
/// - it reaches out of the body on one side only: a recess pushed in through that side (an oven's mouth, a
///   fridge's inside), or a hole right through a wall there (a toy box's grip). The cutter is a box, plain or
///   bevelled, or any slab drawn across that side: what counts is its outline seen from there, which is cut out of
///   every face of the body it passes (a flat front, the strips of a lofted shell), each face keeping its place
///   among the body's; the cutter's own faces inside the body line the cut, turned inward;
/// - it covers the body on every side but one: the body is cut off flat at the cutter's face inside it and
///   closed there (half of a hot tub's cover), `edit::bisect_plane` and `edit::holes_fill`.
///
/// The body is cut as its mesh is: a bevel it is still to be given rounds the cut body afterwards, the mouth's
/// rim too (`forms.cut` on a bevelled box); `bevel::apply` first cuts the rounded body (the kitchen's
/// `_hollow`). The cutter is cut with as it would be written, its own bevel made. The body's UVs are made
/// afterwards, as any part's.
///
/// The points are Blender's (the body's own, the cutter's inside it, and where the outline's sides cross the
/// body's edges or its corners land on a face), and so are the faces, each with Blender's corners round it: a face
/// with the whole opening inside it becomes two, one between the opening and the face's nearest side, one round
/// the rest. The corner a face starts at is the kernel's, and so is the order of the points the cut makes (the
/// body's and the cutter's keep theirs, as Blender numbers them): the baked shading, which turns its rays by a
/// point's number, is a level or two of 32 off Blender's at a few points round the cut.
pub fn cut(obj: Obj, cutter: Obj, o: Cut) -> Obj {
    crate::bevel::apply(cutter);
    let into = obj
        .matrix_world()
        .inverted()
        .expect("a body is not flattened")
        * cutter.matrix_world();
    let tool = cutter.read(|c| c.mesh.clone());
    let pts: Vec<V3> = tool.verts.iter().map(|&p| into * p).collect();
    let (lo, hi) = obj.read(|b| extent(&b.mesh.verts));
    let (clo, chi) = extent(&pts);
    // the sides the cutter reaches out of the body on: (axis, -1 below or +1 above)
    let mut out: Vec<(usize, f32)> = Vec::new();
    for k in 0..3 {
        if clo[k] < lo[k] {
            out.push((k, -1.0));
        }
        if chi[k] > hi[k] {
            out.push((k, 1.0));
        }
    }
    let name = obj.name();
    match out.len() {
        1 => {
            let (k, side) = out[0];
            obj.edit(|part| {
                // (a bevel still to be given rounds the cut mesh, whose points are numbered anew)
                if let Some(bevel) = &mut part.bevel {
                    (bevel.order, bevel.sides_from) = (EdgeOrder::Corners, None);
                }
                part.mesh = carve(&part.mesh, &tool, &pts, k, side, o.lined, &name);
            });
        }
        5 => {
            let (k, side) = (0..3)
                .flat_map(|k| [(k, -1.0_f32), (k, 1.0)])
                .find(|s| !out.contains(s))
                .unwrap();
            // the cutter's face inside the body, looking back into the cutter: what lies that way goes
            let mut co = V3::ZERO;
            co[k] = if side > 0.0 { chi[k] } else { clo[k] };
            let mut no = V3::ZERO;
            no[k] = -side;
            // (the body's sharp edges, to find again what the cut leaves of each)
            let sharp: Vec<(V3, V3)> = obj.edit(|part| {
                if let Some(bevel) = &mut part.bevel {
                    (bevel.order, bevel.sides_from) = (EdgeOrder::Corners, None);
                }
                let at = |v: u32| part.mesh.verts[v as usize];
                part.mesh
                    .sharp
                    .iter()
                    .map(|&(v, w)| (at(v), at(w)))
                    .collect()
            });
            let edges = crate::edit::bisect_plane(
                obj,
                co,
                no,
                crate::edit::Bisect {
                    clear_outer: true,
                    ..Default::default()
                },
            );
            let made = crate::edit::holes_fill(obj, &edges);
            obj.edit(|part| {
                // a sharp edge the cut shortened is sharp still: from its end that is left to the cut's point on it
                let mesh = &mut part.mesh;
                let on_cut: std::collections::BTreeSet<u32> =
                    edges.iter().flat_map(|&(v, w)| [v, w]).collect();
                let mut stubs = Vec::new();
                for f in 0..mesh.faces.len() {
                    let c = mesh.corners(f);
                    for (i, &v) in c.iter().enumerate() {
                        let w = c[(i + 1) % c.len()];
                        let (end, cut_at) = match (on_cut.contains(&v), on_cut.contains(&w)) {
                            (false, true) => (v, w),
                            (true, false) => (w, v),
                            _ => continue,
                        };
                        let (p, x) = (mesh.verts[end as usize], mesh.verts[cut_at as usize]);
                        let along_one = sharp.iter().any(|&(from, to)| {
                            let far = if p == from {
                                to
                            } else if p == to {
                                from
                            } else {
                                return false;
                            };
                            // (the cut's point lies between the edge's ends, on their line)
                            let (d, e) = (far - p, x - p);
                            let t = e.dot(d) / d.dot(d);
                            t > 0.0 && t < 1.0 && (e - d * t).length() < 1e-6
                        });
                        if along_one {
                            stubs.push(edge(end, cut_at));
                        }
                    }
                }
                mesh.sharp.extend(stubs);
            });
            obj.edit(|part| {
                let slot = (o.lined && !tool.mats.is_empty()).then(|| part.mesh.slot(tool.mats[0]));
                for f in made {
                    part.mesh.faces[f].smooth = false;
                    if let Some(slot) = slot {
                        part.mesh.faces[f].mat = slot;
                    }
                }
            });
        }
        _ => panic!(
            "{name}: a cutter reaches out of the body on one side (a recess, a hole through) or on all but one (a cut across), not on {}",
            out.len()
        ),
    }
    // (the cutter's materials join the body's, worn or not, as the modifier adds them)
    obj.edit(|part| {
        for &mat in &tool.mats {
            part.mesh.slot(mat);
        }
    });
    if !o.keep_cutter {
        cutter.remove();
    }
    obj
}

options! {
    /// `recessed_box`'s options: the box's `bevel`; `lining`, the recess's own material; `round_mouth`, the bevel
    /// made after the cut, so it rounds the mouth's rim and the recess's corners too (`forms.cut` on a bevelled
    /// box) and not the box alone.
    RecessedBox { bevel: f64 = 0.0, bevel_segments: usize = 2, lining: Option<Mat> = None, round_mouth: bool = false }
}

/// A box (game `w` x `h` x `d` at `at`) with a box-shaped recess in it: `cavity` (its width, height and depth)
/// at game `cavity_at`, reaching out of the box through the one face it opens: an oven's carcass and its mouth,
/// a dishwasher's tub. Reaching out of the opposite face too, it is a hole right through. `cut` of the two.
#[allow(clippy::too_many_arguments)] // (as the scripts wrote the two boxes)
pub fn recessed_box(
    frame: &GameFrame,
    name: impl AsRef<str>,
    w: f64,
    h: f64,
    d: f64,
    at: (impl Coord, impl Coord, impl Coord),
    material: Mat,
    cavity: (f64, f64, f64),
    cavity_at: (impl Coord, impl Coord, impl Coord),
    o: RecessedBox,
) -> Obj {
    let name = name.as_ref();
    let body = frame.box_(
        name,
        w,
        h,
        d,
        at,
        material,
        Cube {
            bevel: o.bevel,
            bevel_segments: o.bevel_segments,
            ..Cube::default()
        },
    );
    if !o.round_mouth {
        crate::bevel::apply(body);
    }
    let cutter = frame.box_(
        format!("{name}_cavity"),
        cavity.0,
        cavity.1,
        cavity.2,
        cavity_at,
        o.lining.unwrap_or(material),
        Cube::default(),
    );
    cut(
        body,
        cutter,
        Cut {
            lined: o.lining.is_some(),
            ..Cut::default()
        },
    )
}

/// The lowest and highest of each coordinate.
fn extent(points: &[V3]) -> ([f32; 3], [f32; 3]) {
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for p in points {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    (lo, hi)
}

/// A point seen along the way the cutter is pushed in.
type P2 = [f64; 2];

/// Twice the area of the triangle a, b, c: over zero where c is to the left of a to b. Its sign is exact for
/// points that were singles.
fn turn(a: P2, b: P2, c: P2) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// Twice a polygon's area, over zero where it runs anticlockwise.
fn area2(p: &[P2]) -> f64 {
    (0..p.len())
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum()
}

/// How far along a to b the point p lies (0 at a, 1 at b).
fn along(a: P2, b: P2, p: P2) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy)
}

/// Cutter points this near seen from the front are one corner of its outline (m): a bevelled box's front and
/// back corners, each rounded by itself, a unit in the last place apart.
const SNAP: f64 = 1e-6;

/// A new point of a cut body, by what makes it: where a body's edge crosses a side of the outline, an outline's
/// corner on a body's edge or inside a face, a cutter's own point.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Made {
    Crossing(u32, u32, usize),
    OnEdge(u32, u32, usize),
    InFace(usize, usize),
    Tool(usize),
}

/// A point of a face's part inside the outline: a corner of the face, a corner of the outline, or where a side
/// of each cross; the face's sides and the outline's it lies on.
struct Meet {
    at: P2,
    corner: Option<usize>,
    crossing: Option<(usize, usize)>,
    outline: Option<usize>,
    on_face: Vec<usize>,
    on_outline: Vec<usize>,
}

/// What a recess leaves of a body: `tool` (its points `pts` in the body's frame) pushed in along axis `k` from
/// the side `side` (-1: from below, +1: from above).
fn carve(
    body: &Mesh,
    tool: &Mesh,
    pts: &[V3],
    k: usize,
    side: f32,
    lined: bool,
    name: &str,
) -> Mesh {
    let (a, b) = ((k + 1) % 3, (k + 2) % 3);
    let flat = |p: V3| -> P2 { [p[a] as f64, p[b] as f64] };
    // how deep a point lies: growing from the side the cutter reaches out of
    let depth = |p: V3| -> f64 { -(side as f64) * p[k] as f64 };

    // the cutter's outline seen along the axis: its points gathered into corners, the hull of those
    let mut reps: Vec<usize> = Vec::new();
    let mut cluster = vec![0usize; pts.len()];
    for (i, &p) in pts.iter().enumerate() {
        let q = flat(p);
        let found = reps.iter().position(|&r| {
            let other = flat(pts[r]);
            (other[0] - q[0]).abs() <= SNAP && (other[1] - q[1]).abs() <= SNAP
        });
        match found {
            Some(c) => {
                cluster[i] = c;
                // (the corner stands where its foremost point does: the cutter's edge the body's face cuts)
                if depth(p) < depth(pts[reps[c]]) {
                    reps[c] = i;
                }
            }
            None => {
                cluster[i] = reps.len();
                reps.push(i);
            }
        }
    }
    let mut order: Vec<usize> = (0..reps.len()).collect();
    order.sort_by(|&x, &y| {
        let (p, q) = (flat(pts[reps[x]]), flat(pts[reps[y]]));
        p[0].total_cmp(&q[0]).then(p[1].total_cmp(&q[1]))
    });
    let mut hull: Vec<usize> = Vec::new();
    for pass in 0..2 {
        let start = hull.len();
        let run: Vec<usize> = if pass == 0 {
            order.clone()
        } else {
            order.iter().rev().copied().collect()
        };
        for c in run {
            let p = flat(pts[reps[c]]);
            while hull.len() >= start + 2
                && turn(
                    flat(pts[reps[hull[hull.len() - 2]]]),
                    flat(pts[reps[hull[hull.len() - 1]]]),
                    p,
                ) <= 0.0
            {
                hull.pop();
            }
            hull.push(c);
        }
        hull.pop();
    }
    let m = hull.len();
    assert!(m >= 3, "{name}: the cutter has an outline");
    let q: Vec<P2> = hull.iter().map(|&c| flat(pts[reps[c]])).collect();
    let mut hull_at: Vec<Option<usize>> = vec![None; reps.len()];
    for (j, &c) in hull.iter().enumerate() {
        hull_at[c] = Some(j);
    }
    let (t0, t1) = pts
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &p| {
            (l.min(depth(p)), h.max(depth(p)))
        });
    let (qlo, qhi) = q
        .iter()
        .fold(([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]), |(l, h), p| {
            (
                [l[0].min(p[0]), l[1].min(p[1])],
                [h[0].max(p[0]), h[1].max(p[1])],
            )
        });

    let n_old = body.verts.len() as u32;
    let mut new_pts: Vec<V3> = Vec::new();
    let mut made: BTreeMap<Made, u32> = BTreeMap::new();
    // the new points on each of the body's edges, by how far along it from its lower-numbered end
    let mut on_edge: BTreeMap<(u32, u32), Vec<(f64, u32)>> = BTreeMap::new();
    // a new point's edge of the body, and how far along it
    let mut edge_of: BTreeMap<u32, (u32, u32, f64)> = BTreeMap::new();
    // the points along each side of the outline, on the faces the cutter enters by (0) and leaves by (1)
    let mut rim: [Vec<Vec<(f64, u32)>>; 2] = [vec![Vec::new(); m], vec![Vec::new(); m]];
    let mut corner: [Vec<Option<u32>>; 2] = [vec![None; m], vec![None; m]];
    // each cut face: what is left of it, and its corners seen along the axis (anticlockwise)
    let mut pieces: BTreeMap<usize, Vec<Vec<u32>>> = BTreeMap::new();
    let mut sheets: [Vec<(usize, Vec<P2>)>; 2] = [Vec::new(), Vec::new()];
    let mut by_face: Vec<Vec<[u32; 3]>> = vec![Vec::new(); body.faces.len()];
    for (f, t) in body.triangles() {
        by_face[f as usize].push(t);
    }
    // where the line along the axis through a point seen from the front meets a face: on the triangle Blender
    // cuts the face into that it passes through
    let lift = |f: usize, at: P2| -> f32 {
        let c = body.corners(f);
        let first = body.verts[c[0] as usize][k];
        if c.iter().all(|&v| body.verts[v as usize][k] == first) {
            return first;
        }
        let tris = &by_face[f];
        let holds = |t: &[u32; 3]| {
            let p = t.map(|i| flat(body.verts[c[i as usize] as usize]));
            let (d0, d1, d2) = (
                turn(p[0], p[1], at),
                turn(p[1], p[2], at),
                turn(p[2], p[0], at),
            );
            (d0 >= 0.0 && d1 >= 0.0 && d2 >= 0.0) || (d0 <= 0.0 && d1 <= 0.0 && d2 <= 0.0)
        };
        let t = tris.iter().find(|t| holds(t)).unwrap_or(&tris[0]);
        let p = t.map(|i| {
            let v = body.verts[c[i as usize] as usize];
            [v[a] as f64, v[b] as f64, v[k] as f64]
        });
        let (u, w) = (
            [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]],
            [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]],
        );
        let n = [
            u[1] * w[2] - u[2] * w[1],
            u[2] * w[0] - u[0] * w[2],
            u[0] * w[1] - u[1] * w[0],
        ];
        (p[0][2] - (n[0] * (at[0] - p[0][0]) + n[1] * (at[1] - p[0][1])) / n[2]) as f32
    };

    for f in 0..body.faces.len() {
        let ids: Vec<u32> = body.corners(f).to_vec();
        let mut p: Vec<P2> = ids.iter().map(|&v| flat(body.verts[v as usize])).collect();
        let twice = area2(&p);
        if twice.abs() < 1e-14 {
            continue; // (a face seen on edge has nothing cut out of it)
        }
        let (plo, phi) =
            p.iter()
                .fold(([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]), |(l, h), c| {
                    (
                        [l[0].min(c[0]), l[1].min(c[1])],
                        [h[0].max(c[0]), h[1].max(c[1])],
                    )
                });
        if plo[0] >= qhi[0] || phi[0] <= qlo[0] || plo[1] >= qhi[1] || phi[1] <= qlo[1] {
            continue;
        }
        let (dmin, dmax) = ids
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &v| {
                let d = depth(body.verts[v as usize]);
                (l.min(d), h.max(d))
            });
        if dmin >= t1 || dmax <= t0 {
            continue; // (beyond the cutter's end)
        }
        // anticlockwise, as the outline is
        let mut ids = ids;
        let turned = twice < 0.0;
        if turned {
            p.reverse();
            ids.reverse();
        }
        let n = p.len();
        let convex = (0..n).all(|i| turn(p[i], p[(i + 1) % n], p[(i + 2) % n]) >= 0.0);
        // each of the face's corners against the outline's sides, and the outline's against the face's
        let sp: Vec<Vec<f64>> = (0..m)
            .map(|j| (0..n).map(|i| turn(q[j], q[(j + 1) % m], p[i])).collect())
            .collect();
        let sq: Vec<Vec<f64>> = (0..n)
            .map(|i| (0..m).map(|j| turn(p[i], p[(i + 1) % n], q[j])).collect())
            .collect();
        let mut meets: Vec<Meet> = Vec::new();
        for i in 0..n {
            if (0..m).all(|j| sp[j][i] >= 0.0) {
                meets.push(Meet {
                    at: p[i],
                    corner: Some(i),
                    crossing: None,
                    outline: (0..m).find(|&j| q[j] == p[i]),
                    on_face: vec![(i + n - 1) % n, i],
                    on_outline: (0..m).filter(|&j| sp[j][i] == 0.0).collect(),
                });
            }
        }
        for j in 0..m {
            if (0..n).all(|i| sq[i][j] >= 0.0) && !p.contains(&q[j]) {
                meets.push(Meet {
                    at: q[j],
                    corner: None,
                    crossing: None,
                    outline: Some(j),
                    on_face: (0..n).filter(|&i| sq[i][j] == 0.0).collect(),
                    on_outline: vec![(j + m - 1) % m, j],
                });
            }
        }
        for i in 0..n {
            let i1 = (i + 1) % n;
            for j in 0..m {
                let j1 = (j + 1) % m;
                if sp[j][i] * sp[j][i1] < 0.0 && sq[i][j] * sq[i][j1] < 0.0 {
                    let t = sp[j][i] / (sp[j][i] - sp[j][i1]);
                    meets.push(Meet {
                        at: [
                            p[i][0] + (p[i1][0] - p[i][0]) * t,
                            p[i][1] + (p[i1][1] - p[i][1]) * t,
                        ],
                        corner: None,
                        crossing: Some((i, j)),
                        outline: None,
                        on_face: vec![i],
                        on_outline: vec![j],
                    });
                }
            }
        }
        if meets.len() < 3 {
            continue;
        }
        let middle = meets.iter().fold([0.0; 2], |s, x| {
            [
                s[0] + x.at[0] / meets.len() as f64,
                s[1] + x.at[1] / meets.len() as f64,
            ]
        });
        meets.sort_by(|x, y| {
            let angle = |z: &Meet| (z.at[1] - middle[1]).atan2(z.at[0] - middle[0]);
            angle(x).total_cmp(&angle(y))
        });
        let inside: Vec<P2> = meets.iter().map(|x| x.at).collect();
        if area2(&inside) < 1e-12 {
            continue; // (the outline only touches the face)
        }
        assert!(
            convex,
            "{name}: the cutter opens a face that turns in on itself (face {f}): cut it before that face is"
        );
        assert!(
            dmax < t1 && dmin > t0,
            "{name}: the cutter's end lies in a face of the body (face {f})"
        );
        let entering = (side > 0.0) != turned;
        let kind = usize::from(!entering);

        // the points' numbers in the cut body
        let mut number = |key: Made, at: &mut dyn FnMut() -> V3| -> u32 {
            *made.entry(key).or_insert_with(|| {
                new_pts.push(at());
                n_old + new_pts.len() as u32 - 1
            })
        };
        let mut gid: Vec<u32> = Vec::with_capacity(meets.len());
        for x in &meets {
            let id = if let Some(i) = x.corner {
                ids[i]
            } else if let Some((i, j)) = x.crossing {
                let (v, w) = (ids[i], ids[(i + 1) % n]);
                let (lo, hi, slo, shi) = if v < w {
                    (v, w, sp[j][i], sp[j][(i + 1) % n])
                } else {
                    (w, v, sp[j][(i + 1) % n], sp[j][i])
                };
                let t = slo / (slo - shi);
                let id = number(Made::Crossing(lo, hi, j), &mut || {
                    let (from, to) = (body.verts[lo as usize], body.verts[hi as usize]);
                    let mut at = V3::ZERO;
                    for c in 0..3 {
                        at[c] = (from[c] as f64 + (to[c] as f64 - from[c] as f64) * t) as f32;
                    }
                    // (on a side of the outline that runs along an axis, the point is on it to the bit)
                    let j1 = (j + 1) % m;
                    if q[j][0] == q[j1][0] {
                        at[a] = q[j][0] as f32;
                    }
                    if q[j][1] == q[j1][1] {
                        at[b] = q[j][1] as f32;
                    }
                    at
                });
                let list = on_edge.entry((lo, hi)).or_default();
                if !list.iter().any(|e| e.1 == id) {
                    list.push((t, id));
                    edge_of.insert(id, (lo, hi, t));
                }
                id
            } else {
                let j = x.outline.unwrap();
                if let Some(&i) = x.on_face.first() {
                    let (v, w) = (ids[i], ids[(i + 1) % n]);
                    let (lo, hi) = (v.min(w), v.max(w));
                    let (from, to) = (body.verts[lo as usize], body.verts[hi as usize]);
                    let t = along(flat(from), flat(to), q[j]);
                    let id = number(Made::OnEdge(lo, hi, j), &mut || {
                        let mut at = V3::ZERO;
                        at[k] = (from[k] as f64 + (to[k] as f64 - from[k] as f64) * t) as f32;
                        (at[a], at[b]) = (q[j][0] as f32, q[j][1] as f32);
                        at
                    });
                    let list = on_edge.entry((lo, hi)).or_default();
                    if !list.iter().any(|e| e.1 == id) {
                        list.push((t, id));
                        edge_of.insert(id, (lo, hi, t));
                    }
                    id
                } else {
                    number(Made::InFace(f, j), &mut || {
                        let mut at = V3::ZERO;
                        at[k] = lift(f, q[j]);
                        (at[a], at[b]) = (q[j][0] as f32, q[j][1] as f32);
                        at
                    })
                }
            };
            gid.push(id);
            if let Some(j) = x.outline {
                corner[kind][j] = Some(id);
            }
            for &j in &x.on_outline {
                let t = match x.outline {
                    Some(c) if c == j => 0.0,
                    Some(c) if c == (j + 1) % m => 1.0,
                    _ => along(q[j], q[(j + 1) % m], x.at),
                };
                if !rim[kind][j].iter().any(|e| e.1 == id) {
                    rim[kind][j].push((t, id));
                }
            }
        }

        // what is left of the face: its own outline with the cut's points on it, and the cut's the other way
        // round; a stretch that is both is the opening reaching the face's side, and goes
        let mut next: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        let mut join = |from: u32, to: u32| {
            if let Some(back) = next.get_mut(&to)
                && let Some(i) = back.iter().position(|&v| v == from)
            {
                back.remove(i);
                return;
            }
            next.entry(from).or_default().push(to);
        };
        let mut place: BTreeMap<u32, P2> = BTreeMap::new();
        for (i, &v) in ids.iter().enumerate() {
            place.insert(v, p[i]);
        }
        for (x, &id) in meets.iter().zip(&gid) {
            place.insert(id, x.at);
        }
        for i in 0..n {
            let i1 = (i + 1) % n;
            let mut on: Vec<(f64, u32)> = meets
                .iter()
                .zip(&gid)
                .filter(|(x, _)| x.corner.is_none() && x.on_face.contains(&i))
                .map(|(x, &id)| (along(p[i], p[i1], x.at), id))
                .collect();
            on.sort_by(|x, y| x.0.total_cmp(&y.0));
            let mut from = ids[i];
            for (_, id) in on {
                join(from, id);
                from = id;
            }
            join(from, ids[i1]);
        }
        for i in 0..gid.len() {
            join(gid[(i + 1) % gid.len()], gid[i]);
        }
        let mut outer: Vec<Vec<u32>> = Vec::new();
        let mut holes: Vec<Vec<u32>> = Vec::new();
        while let Some(start) = next.iter().find(|(_, to)| !to.is_empty()).map(|(&v, _)| v) {
            let mut ring = Vec::new();
            let mut v = start;
            loop {
                let to = next.get_mut(&v).filter(|to| !to.is_empty());
                let Some(to) = to else {
                    panic!("{name}: the cut leaves face {f} open");
                };
                assert!(
                    to.len() == 1,
                    "{name}: the opening touches a side of face {f} at one point"
                );
                ring.push(v);
                v = to.pop().unwrap();
                if v == start {
                    break;
                }
            }
            let flat_ring: Vec<P2> = ring.iter().map(|v| place[v]).collect();
            if area2(&flat_ring) > 0.0 {
                outer.push(ring);
            } else {
                holes.push(ring);
            }
        }
        let mut left: Vec<Vec<u32>> = Vec::new();
        if holes.is_empty() {
            left = outer;
        } else {
            assert!(
                outer.len() == 1 && holes.len() == 1,
                "{name}: the opening lies whole in face {f}"
            );
            let (rest, beside) = round_hole(&outer[0], &holes[0], &place);
            left.push(rest);
            left.push(beside);
        }
        if turned {
            for ring in &mut left {
                ring.reverse();
            }
        }
        pieces.insert(f, left);
        sheets[kind].push((f, p));
    }
    assert!(
        !sheets[0].is_empty(),
        "{name}: the cutter passes no face of the body on the side it reaches out of"
    );
    let through = !sheets[1].is_empty();
    let point = |id: u32, new_pts: &[V3]| -> V3 {
        if id < n_old {
            body.verts[id as usize]
        } else {
            new_pts[(id - n_old) as usize]
        }
    };
    for kind in 0..2 {
        for (j, side_rim) in rim[kind].iter_mut().enumerate() {
            side_rim.sort_by(|x, y| x.0.total_cmp(&y.0));
            if kind == 0 || through {
                assert!(
                    corner[kind][j].is_some()
                        && corner[kind][(j + 1) % m].is_some()
                        && side_rim.first().map(|e| e.1) == corner[kind][j]
                        && side_rim.last().map(|e| e.1) == corner[kind][(j + 1) % m],
                    "{name}: the cutter's outline lies on the body's faces all round"
                );
            }
        }
    }

    // the cutter's points inside the body: behind the faces it enters by, before those it leaves by
    let surface = |kind: usize, i: usize, new_pts: &[V3]| -> Option<f64> {
        if let Some(j) = hull_at[cluster[i]] {
            return corner[kind][j].map(|id| depth(point(id, new_pts)));
        }
        let at = flat(pts[i]);
        sheets[kind].iter().find_map(|(f, p)| {
            let n = p.len();
            (0..n)
                .all(|c| turn(p[c], p[(c + 1) % n], at) >= 0.0)
                .then(|| {
                    let mut on = V3::ZERO;
                    on[k] = lift(*f, at);
                    depth(on)
                })
        })
    };
    let reach: Vec<(f64, Option<f64>)> = (0..pts.len())
        .map(|i| {
            let enter = surface(0, i, &new_pts).unwrap_or_else(|| {
                panic!("{name}: the cutter's outline lies on the body's faces all round")
            });
            (
                enter,
                if through {
                    surface(1, i, &new_pts)
                } else {
                    None
                },
            )
        })
        .collect();
    let kept: Vec<bool> = (0..pts.len())
        .map(|i| {
            let d = depth(pts[i]);
            d > reach[i].0 && reach[i].1.is_none_or(|leave| d < leave)
        })
        .collect();
    let mut tool_id: Vec<Option<u32>> = vec![None; pts.len()];
    for i in 0..pts.len() {
        if kept[i] {
            new_pts.push(pts[i]);
            tool_id[i] = Some(n_old + new_pts.len() as u32 - 1);
            made.insert(Made::Tool(i), n_old + new_pts.len() as u32 - 1);
        }
    }

    // the cut body: its faces in their order, each cut one's pieces in its place
    let mut verts = body.verts.clone();
    verts.extend(new_pts.iter().copied());
    let mut new = Mesh {
        verts,
        mats: body.mats.clone(),
        ..Mesh::default()
    };
    let old_normals = body.normals.as_ref();
    let mut normals: Vec<V3> = Vec::new();
    // What a bevel stored for the body's corners stays theirs, to be read on the cut mesh as Blender reads it (a
    // corner whose fan now runs on into the cut's smooth faces turns with them); the cut's own corners have
    // nothing stored: two zeros, their fan's own normal.
    let old_stored = body.stored.as_ref().filter(|_| old_normals.is_some());
    let mut stored: Vec<Option<[i16; 2]>> = Vec::new();
    for f in 0..body.faces.len() {
        let face = body.faces[f];
        let c = body.corners(f);
        let own = |v: u32| -> Option<V3> {
            let at = c.iter().position(|&x| x == v)?;
            old_normals.map(|n| n[face.start as usize + at])
        };
        // a point on one of the face's edges takes the nearer end's normal; one inside it, the face's
        let stored_at = |v: u32| -> Option<[i16; 2]> {
            match (c.iter().position(|&x| x == v), old_stored) {
                (Some(at), Some(s)) => s[face.start as usize + at],
                _ => Some([0, 0]),
            }
        };
        let normal_at = |v: u32| -> V3 {
            if let Some(n) = own(v) {
                return n;
            }
            match edge_of.get(&v) {
                Some(&(lo, hi, t)) if own(lo).is_some() && own(hi).is_some() => {
                    own(if t < 0.5 { lo } else { hi }).unwrap()
                }
                _ => body.face_normal(f),
            }
        };
        match pieces.get(&f) {
            Some(left) => {
                for ring in left {
                    new.push_face(ring, face.mat, face.smooth);
                    if old_normals.is_some() {
                        normals.extend(ring.iter().map(|&v| normal_at(v)));
                        stored.extend(ring.iter().map(|&v| stored_at(v)));
                    }
                }
            }
            None => {
                // (a face beside a cut one takes the points the cut put on the edge they share)
                let mut ring: Vec<u32> = Vec::with_capacity(c.len());
                for (i, &v) in c.iter().enumerate() {
                    let w = c[(i + 1) % c.len()];
                    ring.push(v);
                    if let Some(on) = on_edge.get(&edge(v, w)) {
                        let mut on = on.clone();
                        on.sort_by(|x, y| x.0.total_cmp(&y.0));
                        if v > w {
                            on.reverse();
                        }
                        ring.extend(on.iter().map(|e| e.1));
                    }
                }
                if old_normals.is_some() {
                    normals.extend(ring.iter().map(|&v| normal_at(v)));
                    stored.extend(ring.iter().map(|&v| stored_at(v)));
                }
                new.push_face(&ring, face.mat, face.smooth);
            }
        }
    }

    // the cutter's faces inside the body, turned inward: whole, or from the rim back
    let slot_of = |new: &mut Mesh, mat: u16| -> u16 {
        if lined && !tool.mats.is_empty() {
            new.slot(tool.mats[(mat as usize).min(tool.mats.len() - 1)])
        } else if (mat as usize) < body.mats.len() {
            mat
        } else {
            0
        }
    };
    let chain = |kind: usize, j: usize| -> Vec<u32> { rim[kind][j].iter().map(|e| e.1).collect() };
    for f in 0..tool.faces.len() {
        let face = tool.faces[f];
        let c = tool.corners(f);
        let tool_normal = {
            let p: Vec<V3> = c.iter().map(|&v| pts[v as usize]).collect();
            normal_newell(&p)
        };
        if c.iter().all(|&v| kept[v as usize]) {
            let ring: Vec<u32> = (0..c.len())
                .map(|i| tool_id[c[(c.len() - i) % c.len()] as usize].unwrap())
                .collect();
            let slot = slot_of(&mut new, face.mat);
            new.push_face(&ring, slot, face.smooth);
            if old_normals.is_some() {
                normals.extend((0..c.len()).map(|i| {
                    let at = face.start as usize + (c.len() - i) % c.len();
                    match &tool.normals {
                        Some(n) => V3::new(-n[at].x, -n[at].y, -n[at].z),
                        None => V3::new(-tool_normal.x, -tool_normal.y, -tool_normal.z),
                    }
                }));
                stored.extend((0..c.len()).map(|i| {
                    let at = face.start as usize + (c.len() - i) % c.len();
                    tool.stored.as_ref().map_or(Some([0, 0]), |s| s[at])
                }));
            }
            continue;
        }
        // a face along the axis, between two corners of the outline next to each other: a wall of the cut
        let ends: Vec<Option<usize>> = c.iter().map(|&v| hull_at[cluster[v as usize]]).collect();
        let Some(j0) = ends[0] else {
            assert!(
                !c.iter().any(|&v| kept[v as usize]),
                "{name}: the body's face cuts the cutter's end aslant"
            );
            continue;
        };
        let other = ends.iter().flatten().copied().find(|&j| j != j0);
        let (Some(j1), true) = (
            other,
            ends.iter()
                .all(|e| e.is_some_and(|j| j == j0 || Some(j) == other)),
        ) else {
            assert!(
                !c.iter().any(|&v| kept[v as usize]),
                "{name}: the body's face cuts the cutter's end aslant"
            );
            continue;
        };
        let j = if (j0 + 1) % m == j1 {
            j0
        } else if (j1 + 1) % m == j0 {
            j1
        } else {
            assert!(
                !c.iter().any(|&v| kept[v as usize]),
                "{name}: the body's face cuts the cutter's end aslant"
            );
            continue;
        };
        let at_end = |end: usize| -> Vec<u32> {
            let mut v: Vec<u32> = c
                .iter()
                .copied()
                .filter(|&v| hull_at[cluster[v as usize]] == Some(end))
                .collect();
            v.sort_by(|&x, &y| depth(pts[x as usize]).total_cmp(&depth(pts[y as usize])));
            v
        };
        let (near, far) = (at_end(j), at_end((j + 1) % m));
        let before = |end: &[u32]| {
            end.iter()
                .any(|&v| depth(pts[v as usize]) < reach[v as usize].0)
        };
        let beyond = |end: &[u32]| {
            end.iter().any(|&v| {
                reach[v as usize]
                    .1
                    .is_some_and(|leave| depth(pts[v as usize]) > leave)
            })
        };
        let (enters, leaves) = (before(&near) && before(&far), beyond(&near) && beyond(&far));
        let inside = |end: &[u32]| -> Vec<u32> {
            end.iter()
                .filter(|&&v| kept[v as usize])
                .map(|&v| tool_id[v as usize].unwrap())
                .collect()
        };
        let mut ring: Vec<u32> = Vec::new();
        if enters {
            ring.extend(chain(0, j));
        }
        ring.extend(inside(&far));
        if leaves {
            ring.extend(chain(1, j).into_iter().rev());
        }
        ring.extend(inside(&near).into_iter().rev());
        ring.dedup();
        if ring.len() < 3 || !(enters || leaves) {
            assert!(
                !c.iter().any(|&v| kept[v as usize]),
                "{name}: the body's face cuts the cutter's end aslant"
            );
            continue;
        }
        let p: Vec<V3> = ring.iter().map(|&v| new.verts[v as usize]).collect();
        let mut normal = normal_newell(&p);
        if normal.dot(tool_normal) > 0.0 {
            ring.reverse();
            normal = V3::new(-normal.x, -normal.y, -normal.z);
        }
        let slot = slot_of(&mut new, face.mat);
        new.push_face(&ring, slot, face.smooth);
        if old_normals.is_some() {
            normals.extend(ring.iter().map(|_| normal));
            stored.extend(ring.iter().map(|_| Some([0, 0])));
        }
    }

    // the sharp edges: the body's, in as many stretches as the cut left them in, and the cutter's inside it
    let mut present: std::collections::BTreeSet<(u32, u32)> = std::collections::BTreeSet::new();
    for f in 0..new.faces.len() {
        let c = new.corners(f);
        for (i, &v) in c.iter().enumerate() {
            present.insert(edge(v, c[(i + 1) % c.len()]));
        }
    }
    for &(v, w) in &body.sharp {
        let mut run = vec![v];
        if let Some(on) = on_edge.get(&(v, w)) {
            let mut on = on.clone();
            on.sort_by(|x, y| x.0.total_cmp(&y.0));
            run.extend(on.iter().map(|e| e.1));
        }
        run.push(w);
        for pair in run.windows(2) {
            if present.contains(&edge(pair[0], pair[1])) {
                new.sharp.insert(edge(pair[0], pair[1]));
            }
        }
    }
    for &(v, w) in &tool.sharp {
        let stands = |v: u32| -> Option<u32> {
            tool_id[v as usize].or_else(|| hull_at[cluster[v as usize]].and_then(|j| corner[0][j]))
        };
        if let (Some(x), Some(y), true) =
            (stands(v), stands(w), kept[v as usize] || kept[w as usize])
            && present.contains(&edge(x, y))
        {
            new.sharp.insert(edge(x, y));
        }
    }
    if old_normals.is_some() {
        new.normals = Some(normals);
        if old_stored.is_some() {
            new.stored = Some(stored);
        }
    }

    // the body's points inside the cutter are no face's any more
    // and the points are numbered as Blender's solver mostly leaves them: the body's and the cutter's by the
    // numbers they had, one of the body's before the cutter's of the same number, then the points the cut made
    let mut used = vec![false; new.verts.len()];
    for &v in &new.loops {
        used[v as usize] = true;
    }
    let mut renumber = vec![u32::MAX; new.verts.len()];
    let mut verts = Vec::with_capacity(new.verts.len());
    let mut take = |v: u32, verts: &mut Vec<V3>| {
        if used[v as usize] && renumber[v as usize] == u32::MAX {
            renumber[v as usize] = verts.len() as u32;
            verts.push(new.verts[v as usize]);
        }
    };
    for i in 0..(n_old as usize).max(pts.len()) {
        if i < n_old as usize {
            take(i as u32, &mut verts);
        }
        if let Some(id) = tool_id.get(i).copied().flatten() {
            take(id, &mut verts);
        }
    }
    for v in n_old..new.verts.len() as u32 {
        take(v, &mut verts);
    }
    new.verts = verts;
    for v in &mut new.loops {
        *v = renumber[*v as usize];
    }
    new.sharp = new
        .sharp
        .iter()
        .map(|&(v, w)| edge(renumber[v as usize], renumber[w as usize]))
        .collect();
    new
}

/// A face with a hole whole inside it (`outer` anticlockwise, `hole` the other way round, each point's place
/// in `place`) as the two faces Blender leaves: one between the face's side nearest the hole and the hole, the
/// rest round the hole's other sides. (The rest, the one beside.)
fn round_hole(outer: &[u32], hole: &[u32], place: &BTreeMap<u32, P2>) -> (Vec<u32>, Vec<u32>) {
    let (n, h) = (outer.len(), hole.len());
    let at = |v: u32| place[&v];
    let apart = |x: P2, y: P2| ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt();
    // how far a point is from a stretch
    let off = |from: P2, to: P2, x: P2| {
        let t = along(from, to, x).clamp(0.0, 1.0);
        apart(
            [
                from[0] + (to[0] - from[0]) * t,
                from[1] + (to[1] - from[1]) * t,
            ],
            x,
        )
    };
    // the face's side nearest the hole; of sides as near as each other (a hole in the middle) neither, as
    // Blender's solver leaves a carcass's oven mouth
    let near: Vec<f64> = (0..n)
        .map(|i| {
            hole.iter()
                .map(|&v| off(at(outer[i]), at(outer[(i + 1) % n]), at(v)))
                .fold(f64::INFINITY, f64::min)
        })
        .collect();
    let alone = |i: usize| (0..n).all(|j| j == i || (near[j] - near[i]).abs() > 1e-7);
    let side = (0..n)
        .filter(|&i| alone(i))
        .min_by(|&x, &y| near[x].total_cmp(&near[y]))
        .unwrap_or_else(|| (0..n).min_by(|&x, &y| near[x].total_cmp(&near[y])).unwrap());
    let (from, to) = (outer[side], outer[(side + 1) % n]);
    // the hole's corner nearest a point of those that see it: the point is outside one of the corner's two sides
    let nearest = |x: u32| -> usize {
        (0..h)
            .filter(|&i| {
                let (before, here, after) = (
                    at(hole[(i + h - 1) % h]),
                    at(hole[i]),
                    at(hole[(i + 1) % h]),
                );
                turn(before, here, at(x)) > 0.0 || turn(here, after, at(x)) > 0.0
            })
            .min_by(|&i, &j| apart(at(hole[i]), at(x)).total_cmp(&apart(at(hole[j]), at(x))))
            .expect("a hole's corner is seen from outside it")
    };
    let first = nearest(from);
    let mut second = nearest(to);
    if second == first {
        second = (first + h - 1) % h;
    }
    // beside: from the hole along the face's side, then back along the hole; the rest: from the hole round the
    // face, then round the hole (each from the corner Blender's most often starts at)
    let mut beside = vec![hole[first], from, to];
    let mut i = second;
    while i != first {
        beside.push(hole[i]);
        i = (i + 1) % h;
    }
    let mut rest = vec![hole[second]];
    rest.extend((0..n).map(|i| outer[(side + 1 + i) % n]));
    let mut i = first;
    while i != second {
        rest.push(hole[i]);
        i = (i + 1) % h;
    }
    (rest, beside)
}
