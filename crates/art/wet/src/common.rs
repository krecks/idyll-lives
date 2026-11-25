//! What the families here share: the kitchen's and the cookers' bar handle and shaker front (one function in
//! both scripts), and the stream of water from the kitchen's and the bathroom's taps. Each takes the calling
//! family's own materials.

use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::parts::{join, new_parts};
use artkit::place::{Coord, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::{fcyl, fextrude, fsweep, soft};

/// A bar handle in one piece from face point `a` (x, height) to `b`: out from the face at game depth `face` on
/// rounded stand-offs, along, and back in, its front `out` proud; a rounded `w` × `t` section (`r` None: a round bar
/// of radius w / 2).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn pull(
    f: &GameFrame,
    name: impl AsRef<str>,
    a: (f64, f64),
    b: (f64, f64),
    face: f64,
    material: Mat,
    out: f64,
    w: f64,
    t: f64,
    r: Option<f64>,
    verts: usize,
) -> Obj {
    let ((x0, y0), (x1, y1)) = (a, b);
    let n = py::hypot(x1 - x0, y1 - y0);
    let (ux, uy) = ((x1 - x0) / n, (y1 - y0) / n);
    let (e, o) = (
        py::min(0.012, n / 4.0),
        face + out - (if r.is_none() { w } else { t }) / 2.0,
    );
    let pts = [
        (x0, y0, face - 0.004),
        (x0, y0, o - e),
        (x0 + ux * e, y0 + uy * e, o),
        (x1 - ux * e, y1 - uy * e, o),
        (x1, y1, o - e),
        (x1, y1, face - 0.004),
    ];
    match r {
        None => fsweep!(
            f,
            name,
            &pts,
            w / 2.0,
            material,
            up = (uy, 0, ux),
            resolution = 2,
            // shaded round, as a round bar of so few sides is (`round_angle`), its bends too
            angle = artkit::shapes::round_angle(verts).max(60.0),
            verts = verts
        ),
        Some(r) => fsweep!(
            f,
            name,
            &pts,
            0.0,
            material,
            up = (uy, 0, ux),
            resolution = 2,
            angle = 60.0,
            profile = rounded_rect(w, t, r, 1)
        ),
    }
}

/// `pull` with the script's defaults but for the ones named.
pub struct Pull {
    pub material: Option<Mat>,
    pub out: f64,
    pub w: f64,
    pub t: f64,
    pub r: Option<f64>,
    pub verts: usize,
}

impl Default for Pull {
    fn default() -> Pull {
        Pull {
            material: None,
            out: 0.034,
            w: 0.016,
            t: 0.02,
            r: Some(0.007),
            verts: 8,
        }
    }
}

/// A cabinet door or drawer front: a frame (stiles and rails, one slab with a hole) round a panel set 6 mm back.
/// `at` is the middle of its front face's slab (game x, height, depth).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn shaker<X: Coord, Y: Coord>(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    at: (X, Y, f64),
    material: Mat,
    t: f64,
    rail: f64,
    r: f64,
) {
    let (x, y, z) = at;
    let hole = rounded_rect(w - 2.0 * rail, h - 2.0 * rail, 0.003, 1);
    fextrude!(
        f,
        format!("{name}_frame"),
        &rounded_rect(w, h, r, 1),
        t,
        at,
        material,
        holes = [hole],
        bevel = 0.003,
        bevel_segments = 1
    );
    // the panel a hair inside the hole, lightly puffed: its grid gives the baked shading points away from the frame's
    // edge, so only its border darkens (a flat box's four corners, hidden in the frame, darkened all of it)
    soft!(
        f,
        format!("{name}_panel"),
        w - 2.0 * rail - 0.002,
        h - 2.0 * rail - 0.002,
        t * 0.6,
        (x, y, z + t * 0.2 - 0.006),
        material,
        puff = 0.002,
        round = 0.003,
        round_segments = 1,
        flat = 2
    );
}

/// The stream from a tap: a thin cylinder from the spout tip down to the basin, joined with its origin at the tip
/// and scaled to nothing along its length, so it is hidden until `grow` runs the clip. `material` is the calling
/// family's own water.
pub fn water<X: Coord>(f: &GameFrame, spout: (X, f64, f64), floor_y: f64, material: Mat) -> Obj {
    stream(f, "water", spout, floor_y, material, 0.012, 10, 0.06)
}

/// A stream `r` thick (`verts` round) from a spout's `tip` down to `floor_y` (at least `least` long), joined with
/// its origin at the tip and scaled to nothing along its length, so it is hidden until `grow` runs it out: a tap's,
/// a bath filler's, a coffee machine's.
#[allow(clippy::too_many_arguments)]
pub fn stream<X: Coord, Z: Coord>(
    f: &GameFrame,
    name: &str,
    tip: (X, f64, Z),
    floor_y: f64,
    material: Mat,
    r: f64,
    verts: usize,
    least: f64,
) -> Obj {
    let (x, top_y, z) = tip;
    let length = py::max(least, top_y - floor_y);
    let parts = new_parts(|| {
        fcyl!(
            f,
            name,
            r,
            length,
            (x, (top_y + floor_y) / 2.0, z),
            material,
            verts = verts
        );
    });
    let obj = join(&parts, name, Some(f.at(x, top_y, z)));
    obj.edit(|o| o.scale.z = 0.001);
    obj
}

/// A lathe's profile (or any open path, (u, v)) with each corner that turns 60° or more rounded to about `r`, in
/// enough steps that none turns 60°: an eased rim or lip (the gate's `edges`). The other corners are kept, and so
/// are those at its lowest v (a foot on the floor: an underside no one sees).
pub fn eased(profile: &[(f64, f64)], r: f64) -> Vec<(f64, f64)> {
    let low = profile.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let mut out = vec![profile[0]];
    for w in profile.windows(3) {
        let (a, p, c) = (w[0], w[1], w[2]);
        let (d1, d2) = ((p.0 - a.0, p.1 - a.1), (c.0 - p.0, c.1 - p.1));
        let turn = artkit::math::degrees(artkit::math::atan2(
            d1.0 * d2.1 - d1.1 * d2.0,
            d1.0 * d2.0 + d1.1 * d2.1,
        ))
        .abs();
        if turn < 60.0 || p.1 <= low + 1e-9 {
            out.push(p);
            continue;
        }
        let steps = (turn / 50.0).ceil() as usize;
        out.extend(artkit::kit::fillet(&[a, p, c], r, steps)[1..=steps + 1].iter());
    }
    out.push(profile[profile.len() - 1]);
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn eased_rounds_only_sharp_corners() {
        // a cup's square rim: the two 90° corners round, the shallow bend stays a point
        let rim = super::eased(
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.06, 0.02),
                (0.06, 0.1),
                (0.055, 0.1),
            ],
            0.003,
        );
        assert!(rim.len() == 7 && rim.contains(&(0.06, 0.02)) && rim.contains(&(0.05, 0.0)));
        assert!(
            rim.iter()
                .all(|&(u, v)| u <= 0.06 + 1e-6 && v <= 0.1 + 1e-6)
        );
    }
}
