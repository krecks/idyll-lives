//! The kit's parts the kids' things use (gamekit's kit.py), each marked `KIT:` until the kernel has them.

use super::common::P;
use artkit::forms::{Extrude, Loft, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{P3, V3, pow, tan};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::shapes::{Cube, Revolve, revolve};

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
    x: f64,
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
pub fn shaker(
    frame: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    at: P,
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
    let (x, y, z) = at;
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
