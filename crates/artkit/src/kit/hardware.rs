//! Hardware on a front: bar pulls and turned knobs.

use crate::forms::Sweep;
use crate::kit::fillet;
use crate::materials::Mat;
use crate::place::{Coord, GameFrame};
use crate::py::min_of;
use crate::scene::Obj;
use crate::shapes::{self, Revolve, options};

options! {
    /// `pull`'s options.
    Pull { r: f64 = 0.006, out: f64 = 0.03, upright: bool = false, verts: usize = 8 }
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
    let (h, b) = (length / 2.0, min_of(&[0.012, o.out * 0.45]));
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

options! {
    /// `knob`'s options.
    Knob { segments: usize = 12 }
}

/// A turned knob on a front at game depth `face`: a rose against it, a waisted neck and a domed head `r` round.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn knob(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x: impl Coord,
    y: impl Coord,
    face: impl Coord,
    r: f64,
    material: Mat,
    o: Knob,
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
            segments: Some(o.segments),
            rot: (90, 0, 0).into(),
            ..Revolve::default()
        },
    )
}
