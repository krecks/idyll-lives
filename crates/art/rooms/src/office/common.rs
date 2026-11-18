//! What the office things are built from: the frame's helpers and the kit's parts the office uses.

use super::F;
use artkit::materials::Mat;
use artkit::math::{P3, atan2, degrees, p3};
use artkit::parts::{join, set_origin};
use artkit::place::Spot;
use artkit::scene::Obj;
use artkit::{fbox, fsweep, kit, soft};

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
    fsweep!(F, name, &kit::fillet!(&pts, b), r, m, verts = 8)
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
