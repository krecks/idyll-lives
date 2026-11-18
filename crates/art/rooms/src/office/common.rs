//! What the office things are built from: the frame's helpers and the kit's parts the office uses.

use super::F;
use artkit::forms::{Holes, object, rounded_rect};
use artkit::loft;
use artkit::materials::Mat;
use artkit::math::{P3, V3, atan2, degrees, p3, pow, tan};
use artkit::parts::{join, set_origin};
use artkit::place::{GameFrame, Spot};
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsweep, soft};

/// Python's `max(a, b)`: the first unless the second is strictly beyond it.
pub fn pmax(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Joins `parts` and turns them about game point `pivot`: `x` degrees about Blender X (negative leans a back
/// backwards), `z` about the vertical. The turn is baked in when the part is joined again.
pub fn turned(parts: &[Obj], name: &str, pivot: impl Spot, x: f64, z: f64) -> Obj {
    let obj = join(parts, name, None);
    set_origin(obj, pivot.through(&F));
    // (a joined part is turned by Euler angles, whatever its first part was)
    obj.set_rotation((x, 0, z))
}

/// The rot for a cylinder whose axis runs along the game direction (gx, 0, gz).
pub fn axle(gx: f64, gz: f64) -> P3 {
    p3(0, 90, degrees(atan2(-gz, -gx)))
}

/// A bar pull bent from one rod, out on two stand-offs from a front at game depth `face` (towards -depth when
/// `out` is negative).
#[allow(clippy::too_many_arguments)]
pub fn bar_pull(
    name: &str,
    x: f64,
    y: f64,
    face: f64,
    length: f64,
    m: Mat,
    out: f64,
    r: f64,
) -> Obj {
    let (h, b) = (length / 2.0, 0.01_f64.min(out.abs() * 0.45));
    let into = face - 0.003_f64.copysign(out);
    let ends = [(-h, into), (-h, face + out), (h, face + out), (h, into)];
    let pts: Vec<(f64, f64, f64)> = ends.iter().map(|&(u, zz)| (x + u, y, zz)).collect();
    fsweep!(F, name, &fillet(&pts, b, 3), r, m, verts = 8)
}

/// A desk top `w` x `d`, 35 mm thick, its surface at `y`, its edges softly rounded.
pub fn top(w: f64, d: f64, z: f64, m: Mat, y: f64) -> Obj {
    let t = 0.035;
    soft!(
        F,
        "top",
        w,
        t,
        d,
        (0.0, y - t / 2.0, z),
        m,
        round = 0.008,
        round_segments = 2
    )
}

/// A slab drawer front, its edges rounded, centred at game (x, y, z).
#[allow(clippy::too_many_arguments)]
pub fn front(name: &str, w: f64, h: f64, x: f64, y: f64, z: f64, m: Mat, t: f64) -> Obj {
    fbox!(
        F,
        name,
        w,
        h,
        t,
        (x, y, z),
        m,
        bevel = 0.005,
        bevel_segments = 1
    )
}

/// An open-topped drawer box behind a front at depth `z_front`, running back (towards +z).
#[allow(clippy::too_many_arguments)]
pub fn drawer_box(name: &str, w: f64, h: f64, d: f64, x: f64, y0: f64, z_front: f64, m: Mat) {
    let t = 0.01;
    let zc = z_front + d / 2.0;
    fbox!(
        F,
        format!("{name}_bottom"),
        w,
        t,
        d,
        (x, y0 + t / 2.0, zc),
        m
    );
    for s in [-1, 1] {
        fbox!(
            F,
            format!("{name}_side{s}"),
            t,
            h,
            d,
            (x + f64::from(s) * (w - t) / 2.0, y0 + h / 2.0, zc),
            m,
            bevel = 0.002,
            bevel_segments = 1
        );
    }
    fbox!(
        F,
        format!("{name}_back"),
        w,
        h,
        t,
        (x, y0 + h / 2.0, z_front + d - t / 2.0),
        m
    );
}

// KIT: kit.fillet
/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
/// it, on a curve that never overshoots, however uneven the spacing). For handles, bent rails and draped cloth
/// swept with `resolution` 1.
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
    frame: &GameFrame,
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
    fsweep!(frame, name, &fillet(&pts, b, 3), r, material, verts = verts)
}

// KIT: kit.shaker
/// A frame-and-panel front `w` x `h`, `t` thick, centred at game `at`, facing the front (a cabinet's door, a
/// drawer's front): a frame `rail` wide round a panel set back `recess`, flat, or `raised` (its edges chamfered
/// up to a field); `panel`: the panel's own material. Returns (frame, panel).
#[allow(clippy::too_many_arguments)]
pub fn shaker(
    frame: &GameFrame,
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
        frame,
        name,
        &rounded_rect(w, h, 0.003, 1),
        t,
        at,
        material,
        holes = Holes(vec![hole]),
        bevel = 0.004_f64.min(t * 0.2),
        bevel_segments = 1
    );
    // (a hair inside the opening, not into the frame: a vertex buried in it would bake black, `shade`)
    let (pw, ph) = (w - 2.0 * rail - 0.002, h - 2.0 * rail - 0.002);
    if !raised {
        let pn = fbox!(
            frame,
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
    let b = (rail * 0.7).min(pw * 0.25).min(ph * 0.25);
    let lp = |hw: f64, hh: f64, d: f64| -> Vec<P3> {
        [(hw, hh), (-hw, hh), (-hw, -hh), (hw, -hh)]
            .iter()
            .map(|&(u, v)| frame.at(x + u, y + v, d))
            .collect()
    };
    let front = z + t / 2.0;
    let pn = loft!(
        format!("{name}_panel"),
        &[
            lp(pw / 2.0, ph / 2.0, z - t / 2.0 + 0.002),
            lp(pw / 2.0, ph / 2.0, front - recess - 0.004),
            lp(pw / 2.0 - b, ph / 2.0 - b, front - recess * 0.2),
        ],
        material = panel.unwrap_or(material),
        smooth = false
    );
    (fr, pn)
}

/// Where a moulding's outline is drawn (`moulding`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Run {
    /// Game (x, height) points at depth `at`.
    Front,
    /// Game (x, depth) points at height `at`.
    Top,
}

// KIT: kit.moulding
/// A moulding run round a closed `outline` with mitred corners: a picture or mirror frame, a panel's bead, a
/// plinth's or a cornice's band. `outline` is game (x, height) points drawn at depth `at` (`Run::Front`), or (x,
/// depth) points at height `at` (`Run::Top`); `profile` is the moulding's closed cross-section [(inset, out),
/// ...]: `inset` metres in from the outline, `out` metres forward of `at` (front) or up from it (top). The corners
/// stay square however wide the profile (a sweep pinches them).
#[allow(clippy::too_many_arguments)]
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
    let area: f64 =
        artkit::py::sum((0..n).map(|i| {
            outline[i].0 * outline[(i + 1) % n].1 - outline[(i + 1) % n].0 * outline[i].1
        }));
    let turn = if area > 0.0 { 1.0 } else { -1.0 }; // inward is to the left of an anticlockwise outline
    let inward = |a: (f64, f64), b: (f64, f64)| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        // (`or 1.0`: a side of no length)
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
    object(name, vs, &faces, Some(material), true, angle, false)
}
