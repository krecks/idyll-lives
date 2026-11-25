//! A shaker door.

use super::*;
use crate::forms::{self, Extrude, Holes, Loft, rounded_rect};
use crate::materials::Mat;
use crate::math::P3;
use crate::place::{Coord, Facing, GameFrame};
use crate::py::min_of;
use crate::scene::Obj;
use crate::shapes::{Cube, SHARP, options};

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
            bevel: min_of(&[0.004, t * 0.2]),
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
    let b = min_of(&[rail * 0.7, pw * 0.25, ph * 0.25]);
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
