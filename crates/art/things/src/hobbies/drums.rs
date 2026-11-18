//! H1: the drum kit's parts: a cymbal, a drum with its heads and lugs, a chrome stand, a foot pedal.

use super::M;
use super::common::{P, rr, to_euler_degrees};
use super::kit::call::tube;
use artkit::math::{Axis, M4, TAU, V3, cos, cosd, radians, sin, sind};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{cube, cylinder, fbox, fcyl, fsweep, revolve, torus};

/// A cymbal: a domed bell rising out of a bow that thins to its edge.
pub fn cymbal(fr: &GameFrame, name: &str, r: f64, at: P, rot: P, segments: usize) -> Obj {
    let profile = [
        (0.0, 0.021),
        (0.024, 0.018),
        (0.034, 0.009),
        (r * 0.6, 0.0035),
        (r, 0.0),
        (r, -0.0015),
        (r * 0.6, 0.002),
        (0.034, 0.006),
        (0.024, 0.015),
        (0.0, 0.018),
    ];
    revolve!(
        name,
        &profile,
        at = fr.at(at.0, at.1, at.2),
        material = M.cymbal,
        segments = segments,
        rot = rot
    )
}

/// A drum along its own axis: the shell, its heads (1 top, -1 bottom) in their hoops, and its lugs (`rods`: each with
/// a tension rod into each hoop).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn drum(
    fr: &GameFrame,
    name: &str,
    r: f64,
    h: f64,
    at: P,
    rot: P,
    heads: &[i32],
    lugs: usize,
    rods: bool,
) {
    let tilt = M4::rotation(radians(rot.0), Axis::X);
    let centre = V3::from(fr.at(at.0, at.1, at.2));
    let at_local = |x: f64, y: f64, z: f64| centre + tilt * V3::from((x, y, z));
    fcyl!(fr, name, r, h, at, M.shell, rot = rot, verts = 24);
    for &s in heads {
        let c = at_local(0.0, 0.0, s as f64 * (h / 2.0 + 0.002));
        cylinder!(
            format!("{name}_head{s}"),
            r - 0.004,
            0.004,
            at = c,
            material = M.head,
            rot = rot,
            verts = 24,
            smooth = false
        );
        torus!(
            format!("{name}_hoop{s}"),
            r + 0.002,
            0.007,
            at = c,
            material = M.chrome,
            rot = rot,
            major_segments = 20,
            minor_segments = 3
        );
    }
    for k in 0..lugs {
        let a = TAU * (k as f64 + 0.5) / lugs as f64;
        let turn = to_euler_degrees(tilt * M4::rotation(a, Axis::Z));
        let rr = r + 0.006;
        cube!(
            format!("{name}_lug{k}"),
            (0.01, 0.014, h * 0.32),
            at = at_local(rr * cos(a), rr * sin(a), 0.0),
            material = M.chrome,
            rot = turn
        );
        for &s in if rods { heads } else { &[] } {
            cylinder!(
                format!("{name}_rod{k}{s}"),
                0.0025,
                0.03,
                at = at_local(
                    (r + 0.009) * cos(a),
                    (r + 0.009) * sin(a),
                    s as f64 * (h / 2.0 - 0.01)
                ),
                material = M.chrome,
                rot = rot,
                verts = 4
            );
        }
    }
}

/// A chrome stand: a tripod of legs braced to a collar, a lower tube and a thinner one sliding out of it at a clamp,
/// rubber feet.
pub fn drum_stand(fr: &GameFrame, name: &str, x: f64, z: f64, top: f64, spread: f64) {
    let mid = 0.26 + (top - 0.26) * 0.55;
    tube!(
        fr,
        format!("{name}_pole"),
        &[(x, 0.14, z), (x, mid, z)],
        0.011,
        M.chrome,
        verts = 8,
        round = 1
    );
    tube!(
        fr,
        format!("{name}_upper"),
        &[(x, mid, z), (x, top, z)],
        0.0075,
        M.chrome,
        verts = 8,
        round = 1
    );
    fcyl!(
        fr,
        format!("{name}_clamp"),
        0.015,
        0.026,
        (x, mid, z),
        M.chrome,
        verts = 8
    );
    fcyl!(
        fr,
        format!("{name}_hub"),
        0.017,
        0.035,
        (x, 0.26, z),
        M.chrome,
        verts = 8
    );
    fcyl!(
        fr,
        format!("{name}_collar"),
        0.015,
        0.02,
        (x, 0.16, z),
        M.chrome,
        verts = 6
    );
    for k in 0..3 {
        let a = (90 + 120 * k) as f64;
        let (c, s) = (cosd(a), sind(a));
        let (fx, fz) = (x + spread * c, z + spread * s);
        tube!(
            fr,
            format!("{name}_leg{k}"),
            &[(x, 0.26, z), (fx, 0.01, fz)],
            0.0065,
            M.chrome,
            verts = 6,
            round = 1
        );
        tube!(
            fr,
            format!("{name}_brace{k}"),
            &[
                (x, 0.16, z),
                (x + spread * 0.5 * c, 0.135, z + spread * 0.5 * s)
            ],
            0.004,
            M.chrome,
            verts = 4,
            round = 1
        );
        fcyl!(
            fr,
            format!("{name}_foot{k}"),
            0.01,
            0.014,
            (fx, 0.007, fz),
            M.seat,
            verts = 6
        );
    }
}

/// A foot pedal along the depth, its heel at the back (game -z): a heel plate, a footboard tilting up to the front,
/// and the frame's two posts, axle and cam over its toe.
pub fn pedal(fr: &GameFrame, name: &str, x: f64, z: f64, length: f64) {
    let (z0, z1) = (z - length / 2.0, z + length / 2.0);
    fbox!(
        fr,
        format!("{name}_heel"),
        0.075,
        0.012,
        0.06,
        (x, 0.006, z0 + 0.03),
        M.chrome,
        bevel = 0.003,
        bevel_segments = 1
    );
    fsweep!(
        fr,
        format!("{name}_board"),
        &[(x, 0.016, z0 + 0.05), (x, 0.05, z1 - 0.03)],
        0,
        M.chrome,
        profile = rr(0.008, 0.065, 0.003, 1),
        scale = [0.85, 1.1]
    );
    fbox!(
        fr,
        format!("{name}_base"),
        0.11,
        0.01,
        0.06,
        (x, 0.005, z1),
        M.chrome,
        bevel = 0.003,
        bevel_segments = 1
    );
    for s in [-1, 1] {
        let sf = s as f64;
        tube!(
            fr,
            format!("{name}_post{s}"),
            &[(x + sf * 0.05, 0.01, z1), (x + sf * 0.05, 0.13, z1)],
            0.006,
            M.chrome,
            verts = 6,
            round = 1
        );
    }
    tube!(
        fr,
        format!("{name}_axle"),
        &[(x - 0.05, 0.12, z1), (x + 0.05, 0.12, z1)],
        0.006,
        M.chrome,
        verts = 6,
        round = 1
    );
    fcyl!(
        fr,
        format!("{name}_cam"),
        0.02,
        0.02,
        (x, 0.12, z1),
        M.chrome,
        rot = (0, 90, 0),
        verts = 10
    );
    tube!(
        fr,
        format!("{name}_chain"),
        &[(x, 0.05, z1 - 0.03), (x, 0.11, z1 - 0.012)],
        0.004,
        M.seat,
        verts = 4,
        round = 1
    );
}
