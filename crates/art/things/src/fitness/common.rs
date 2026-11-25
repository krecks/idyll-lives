//! What the home gym is built from: the helpers fitness.py shared between its models.

use super::M;
pub use crate::common::P;
use artkit::materials::Mat;
use artkit::math::{Axis, M4, P3, cos, radians, sin};
use artkit::place::{GameFrame, N};
use artkit::{fcyl, revolve};

/// `_on`: a game point `d` (x, y, z) from `c` on a face tilted `tilt` degrees about the game's x axis.
pub fn on(c: (N, f64, f64), tilt: i32, d: (N, f64, f64)) -> (N, f64, f64) {
    let a = radians(tilt as f64);
    (
        c.0 + d.0,
        c.1 + d.1 * cos(a) - d.2 * sin(a),
        c.2 + d.1 * sin(a) + d.2 * cos(a),
    )
}

/// `_plate`: a weight plate on a bar along game x at (x, y, z): a turned disc `r` round and `t` thick, a raised
/// rim, a dished face and a hub, both faces alike; `ring`: a coloured band inside the rim.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn plate(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    t: f64,
    material: Mat,
    ring: Option<Mat>,
) {
    let prof = [
        (0.0, -t / 2.0 + 0.006),
        (0.03, -t / 2.0 + 0.006),
        (0.032, -t / 2.0),
        (r - 0.012, -t / 2.0),
        (r, -t / 2.0 + 0.006),
        (r, t / 2.0 - 0.006),
        (r - 0.012, t / 2.0),
        (0.032, t / 2.0),
        (0.03, t / 2.0 - 0.006),
        (0.0, t / 2.0 - 0.006),
    ];
    revolve!(
        name,
        &prof,
        at = f.at(x, y, z),
        material = material,
        segments = 24,
        rot = (0, 90, 0)
    );
    if let Some(ring) = ring {
        for s in [-1, 1] {
            revolve!(
                format!("{name}_ring{s}"),
                &[
                    (r * 0.62, 0.0),
                    (r * 0.78, 0.0),
                    (r * 0.78, 0.0015),
                    (r * 0.62, 0.0015)
                ],
                at = f.at(x + s as f64 * (t / 2.0 - 0.0005), y, z),
                material = ring,
                segments = 24,
                rot = (0, -90 * s, 0)
            );
        }
    }
}

/// `_dumbbell`: a hex dumbbell: a knurled chrome handle and two hexagonal heads `w` across their flats, chamfered,
/// with chrome end caps, lying along game x.
pub fn dumbbell(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, w: f64) {
    let rot: P3 = (0, 90, 0).into();
    let off = |k: f64| -> P { (x + k, y, z) };
    fcyl!(
        f,
        format!("{name}_grip"),
        0.014,
        0.1,
        (x, y, z),
        M.chrome,
        rot = rot,
        verts = 8
    );
    fcyl!(
        f,
        format!("{name}_knurl"),
        0.0145,
        0.07,
        (x, y, z),
        M.frame,
        rot = rot,
        verts = 8
    );
    // the hexagon's corner radius
    let r = w / 2.0 / cos(std::f64::consts::PI / 6.0);
    for s in [-1, 1] {
        let sf = s as f64;
        let head = [
            (0.0, -0.0225),
            (r * 0.85, -0.0225),
            (r, -0.016),
            (r, 0.016),
            (r * 0.85, 0.0225),
            (0.0, 0.0225),
        ];
        let (ox, oy, oz) = off(sf * 0.068);
        let obj = revolve!(
            format!("{name}_head{s}"),
            &head,
            at = f.at(ox, oy, oz),
            material = M.plate,
            segments = 6,
            rot = rot,
            smooth = false
        );
        // a flat underneath, not a corner
        let turn = M4::rotation(radians(30.0), Axis::Z);
        obj.edit(|o| o.mesh.transform(&turn));
        fcyl!(
            f,
            format!("{name}_cap{s}"),
            0.016,
            0.004,
            off(sf * 0.0915),
            M.chrome,
            rot = rot,
            verts = 8
        );
    }
}
