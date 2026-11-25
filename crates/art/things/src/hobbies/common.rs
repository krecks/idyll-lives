//! What the hobbies are built from: the helpers hobbies.py shared between its models.

use super::M;
pub use crate::common::P;
use artkit::forms::rounded_rect;
use artkit::kit::{self, fillet};
use artkit::materials::Mat;
use artkit::math::{M4, V3, cos, cosd, degrees, pow, radians, sin, sind};
use artkit::mesh::{Mesh, normal_newell};
use artkit::motion::{Squash, squash};
use artkit::place::GameFrame;
use artkit::scene::{self, Obj};
use artkit::{fbox, fcyl, fsweep, rod, soft, torus};

/// The game direction of an old part's long axis (its local y) under its `rotate`: Euler degrees (yaw y, pitch x,
/// roll z), read as the game reads them (`quatFromEuler` in render/shared/math.ts).
pub fn axis(rx: f64, ry: f64, rz: f64) -> P {
    let (rx, ry, rz) = (radians(rx), radians(ry), radians(rz));
    let (cy, sy, cp, sp, cr, sr) = (
        cos(ry / 2.0),
        sin(ry / 2.0),
        cos(rx / 2.0),
        sin(rx / 2.0),
        cos(rz / 2.0),
        sin(rz / 2.0),
    );
    let x = cy * sp * cr + sy * cp * sr;
    let y = sy * cp * cr - cy * sp * sr;
    let z = cy * cp * sr - sy * sp * cr;
    let w = cy * cp * cr + sy * sp * sr;
    (
        2.0 * (x * y - z * w),
        1.0 - 2.0 * (z * z + x * x),
        2.0 * (y * z + x * w),
    )
}

/// The game end points of an old part's turned cylinder (centre `at`, `length` long, `rotate`).
pub fn ends(at: P, length: f64, rotate: P) -> (P, P) {
    let d = axis(rotate.0, rotate.1, rotate.2);
    (
        (
            at.0 - d.0 * length / 2.0,
            at.1 - d.1 * length / 2.0,
            at.2 - d.2 * length / 2.0,
        ),
        (
            at.0 + d.0 * length / 2.0,
            at.1 + d.1 * length / 2.0,
            at.2 + d.2 * length / 2.0,
        ),
    )
}

/// A part from points and faces made one by one, each face with its material's slot (`materials` in their
/// order), as they were given: what the script built in a bmesh.
pub fn link(name: &str, verts: Vec<V3>, faces: &[(Vec<u32>, u16)], materials: &[Mat]) -> Obj {
    let mut mesh = Mesh {
        verts,
        ..Mesh::default()
    };
    for m in materials {
        mesh.slot(*m);
    }
    for (corners, mat) in faces {
        mesh.push_face(corners, *mat, false);
    }
    scene::add(name, mesh)
}

/// Flat quads lying level, one mesh of two triangles each (the lines between keys, a board's squares):
/// `quads` [(x0, x1, z0, z1, height)], in game axes, facing up.
pub fn flat_quads(
    fr: &GameFrame,
    name: &str,
    quads: &[(f64, f64, f64, f64, f64)],
    material: Mat,
) -> Obj {
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    for &(x0, x1, z0, z1, y) in quads {
        let start = verts.len() as u32;
        verts.extend(
            [(x0, z0), (x1, z0), (x1, z1), (x0, z1)]
                .iter()
                .map(|&(x, z)| V3::from(fr.at(x, y, z))),
        );
        faces.push((up_quad(&verts, start, |n| n.z < 0.0), 0));
    }
    link(name, verts, &faces, &[material])
}

/// A new quad's corners, turned over (as bmesh turns a face, from its first corner) when `over` says its normal is
/// the wrong way.
pub fn up_quad(verts: &[V3], start: u32, over: impl Fn(V3) -> bool) -> Vec<u32> {
    let c: Vec<u32> = (start..start + 4).collect();
    let n = normal_newell(&c.iter().map(|&v| verts[v as usize]).collect::<Vec<V3>>());
    if over(n) {
        vec![c[0], c[3], c[2], c[1]]
    } else {
        c
    }
}

/// A sheet of music facing the front: paper and four staves.
pub fn sheet_music(fr: &GameFrame, name: &str, w: f64, h: f64, at: P, rot: P, depth: f64) {
    let (x, y, z) = at;
    fbox!(fr, name, w, h, depth, at, M.sheet, rot = rot);
    let tilt = radians(rot.0);
    for k in 0..4 {
        let v = h * (0.32 - 0.2 * k as f64);
        fbox!(
            fr,
            format!("{name}_staff{k}"),
            w * 0.8,
            h * 0.06,
            0.001,
            (x, y + v * cos(tilt), z - v * sin(tilt) + 0.0015 * cos(tilt)),
            M.ink,
            rot = rot
        );
    }
}

/// A side chair behind a hobby table, facing the front: its seat at depth `z` with a cushion (its top at 0.475 m,
/// where the game seats residents), back posts rising in one piece from the floor, tapered front legs, stretchers,
/// and a curved top rail and mid rail with three slats between.
pub fn chair(fr: &GameFrame, z: f64, frame: Mat, pad: Mat, w: f64, d: f64) {
    let back = z - d / 2.0 + 0.02;
    let post = rounded_rect(0.032, 0.032, 0.008, 1);
    fbox!(
        fr,
        "chair_seat",
        w,
        0.035,
        d,
        (0, 0.4225, z),
        frame,
        bevel = 0.006,
        bevel_segments = 1
    );
    soft!(
        fr,
        "chair_pad",
        w - 0.04,
        0.03,
        d - 0.05,
        (0, 0.457, z + 0.01),
        pad,
        round = 0.012,
        puff = 0.005,
        flat = 2,
        round_segments = 1
    );
    for s in [-1, 1] {
        let x = s as f64 * (w / 2.0 - 0.03);
        fsweep!(
            fr,
            format!("chair_post{s}"),
            &[
                (x, 0.0, back + 0.012),
                (x, 0.42, back + 0.004),
                (x, 0.92, back - 0.004)
            ],
            0,
            frame,
            profile = &post,
            up = (1, 0, 0),
            scale = [0.85, 1.0, 0.9]
        );
        fsweep!(
            fr,
            format!("chair_leg{s}"),
            &[(x, 0.405, z + d / 2.0 - 0.03), (x, 0.0, z + d / 2.0 - 0.03)],
            0,
            frame,
            profile = &post,
            up = (1, 0, 0),
            scale = [1.0, 0.75]
        );
        fbox!(
            fr,
            format!("chair_stretcher{s}"),
            0.02,
            0.02,
            d - 0.08,
            (x, 0.14, z),
            frame
        );
    }
    fbox!(
        fr,
        "chair_stretcher_front",
        w - 0.08,
        0.02,
        0.02,
        (0, 0.18, z + d / 2.0 - 0.03),
        frame
    );
    // how far back the rails curve at x across
    let bow = |x: f64| back + 0.012 - 0.02 * (1.0 - pow(x / (w / 2.0), 2.0));
    let xs: Vec<f64> = (0..7)
        .map(|i| -(w / 2.0 - 0.045) + (w - 0.09) * i as f64 / 6.0)
        .collect();
    let top: Vec<P> = xs.iter().map(|&x| (x, 0.85, bow(x))).collect();
    fsweep!(
        fr,
        "chair_rail_top",
        &top,
        0,
        frame,
        profile = rounded_rect(0.09, 0.022, 0.01, 1)
    );
    let mid: Vec<P> = xs.iter().map(|&x| (x, 0.62, bow(x))).collect();
    fsweep!(
        fr,
        "chair_rail_mid",
        &mid,
        0,
        frame,
        profile = rounded_rect(0.035, 0.02, 0.006, 1)
    );
    for k in [-1, 0, 1] {
        let x = k as f64 * 0.1;
        fbox!(
            fr,
            format!("chair_slat{k}"),
            0.03,
            0.2,
            0.012,
            (x, 0.72, bow(x)),
            frame
        );
    }
}

/// A round stool behind a hobby table: a seat of radius `r` with its top at `top`, its edge rounded, on four splayed
/// legs joined by a foot ring.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn stool(fr: &GameFrame, z: f64, top: f64, r: f64, legs: Mat, seat: Mat, x: f64) {
    kit::turned!(
        fr,
        "stool_seat",
        x,
        top - 0.04,
        z,
        &[
            (0.0, 0.0),
            (r - 0.012, 0.0),
            (r, 0.01),
            (r, 0.028),
            (r - 0.01, 0.04),
            (0.0, 0.04),
        ],
        seat,
        segments = 20
    );
    for k in 0..4 {
        let a = (45 + 90 * k) as f64;
        kit::tube!(
            fr,
            format!("stool_leg{k}"),
            &[
                (x + 0.1 * cosd(a), top - 0.04, z + 0.1 * sind(a)),
                (x + 0.15 * cosd(a), 0.0, z + 0.15 * sind(a))
            ],
            0.016,
            legs,
            verts = 8,
            round = 1,
            scale = [1.0, 0.8]
        );
    }
    torus!(
        "stool_ring",
        0.131,
        0.007,
        at = fr.at(x, 0.16, z),
        material = legs,
        major_segments = 20,
        minor_segments = 4
    );
}

/// Leans an upright instrument back by `degrees` and stands its foot at game `foot`. The lean is the node's turn,
/// not baked into its mesh: the game takes the part upright (its foot at its origin, its face to the front) to draw
/// it in a resident's hands (content `held`).
pub fn lean(fr: &GameFrame, obj: Obj, degrees: f64, foot: P) -> Obj {
    obj.edit(|o| o.rot.x = radians(-degrees) as f32);
    obj.set_location(fr.at(foot.0, foot.1, foot.2))
}

/// The instrument hidden while it's played (`off_stand`, quickly: the game draws it in the hands then).
pub fn off_stand(obj: Obj) {
    squash(
        obj,
        "off_stand",
        0.001,
        Squash {
            seconds: 0.2,
            ..Squash::default()
        },
    );
}

/// A black A-frame stand: a tube leaning back from a hub at depth `base_z` up to a rubber-lined yoke at (`top_y`,
/// `top_z`), a height collar on it, two legs curving back and out, a front foot, and a cradle bent in one piece across
/// at `cradle_y`, its arms reaching `reach` forward in rubber sleeves; the legs reach `back` behind the hub, the front
/// foot `front` ahead.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn guitar_stand(
    fr: &GameFrame,
    base_z: f64,
    cradle_y: f64,
    top_y: f64,
    top_z: f64,
    reach: f64,
    back: f64,
    front: f64,
) {
    let t = (cradle_y - 0.03) / (top_y - 0.03);
    let cz = base_z + t * (top_z - base_z);
    kit::tube!(
        fr,
        "post",
        &[(0, 0.03, base_z), (0, top_y, top_z)],
        0.011,
        M.stand,
        verts = 10,
        round = 1
    );
    // the height collar and its knob, two thirds up
    let k = 0.6;
    let (cy, cz2) = (0.03 + k * (top_y - 0.03), base_z + k * (top_z - base_z));
    fcyl!(
        fr,
        "collar",
        0.016,
        0.04,
        (0, cy, cz2),
        M.rubber,
        verts = 10,
        bevel = 0.004,
        bevel_segments = 1
    );
    kit::tube!(
        fr,
        "collar_knob",
        &[(0.0, cy, cz2), (0.035, cy, cz2)],
        0.004,
        M.stand,
        verts = 6,
        round = 1
    );
    fcyl!(
        fr,
        "collar_knob_head",
        0.011,
        0.012,
        (0.04, cy, cz2),
        M.rubber,
        rot = (0, 90, 0),
        verts = 10
    );
    fbox!(
        fr,
        "hub",
        0.04,
        0.035,
        0.05,
        (0, 0.03, base_z),
        M.stand,
        bevel = 0.008,
        bevel_segments = 1
    );
    for s in [-1, 1] {
        let sf = s as f64;
        kit::tube!(
            fr,
            format!("leg{s}"),
            &fillet(
                &[
                    (0.0, 0.03, base_z),
                    (sf * 0.06, 0.024, base_z - back * 0.35),
                    (sf * 0.16, 0.012, base_z - back)
                ],
                0.05,
                3
            ),
            0.009,
            M.stand,
            verts = 8,
            round = 1
        );
        fcyl!(
            fr,
            format!("foot{s}"),
            0.013,
            0.02,
            (sf * 0.16, 0.01, base_z - back),
            M.rubber,
            verts = 10,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    kit::tube!(
        fr,
        "foot_front",
        &[(0, 0.022, base_z), (0, 0.012, base_z + front)],
        0.009,
        M.stand,
        verts = 8,
        round = 1
    );
    fcyl!(
        fr,
        "foot_front_pad",
        0.013,
        0.02,
        (0, 0.01, base_z + front),
        M.rubber,
        verts = 10,
        bevel = 0.004,
        bevel_segments = 1
    );
    // the cradle: bent in one piece, its arms sleeved in rubber
    let arm = [
        (-0.1, cradle_y + 0.03, cz + reach),
        (-0.1, cradle_y, cz),
        (0.1, cradle_y, cz),
        (0.1, cradle_y + 0.03, cz + reach),
    ];
    kit::tube!(
        fr,
        "cradle",
        &fillet(&arm, 0.025, 3),
        0.008,
        M.stand,
        verts = 8,
        round = 1
    );
    for s in [-1, 1] {
        let sf = s as f64;
        kit::tube!(
            fr,
            format!("arm{s}"),
            &[
                (sf * 0.1, cradle_y + 0.009, cz + 0.03),
                (sf * 0.1, cradle_y + 0.03, cz + reach + 0.004)
            ],
            0.012,
            M.rubber,
            verts = 8,
            round = 1
        );
    }
    // the yoke: a rubber-lined fork the neck rests in
    let yoke = [
        (-0.03, top_y + 0.045, top_z + 0.01),
        (-0.03, top_y, top_z),
        (0.03, top_y, top_z),
        (0.03, top_y + 0.045, top_z + 0.01),
    ];
    kit::tube!(
        fr,
        "yoke",
        &fillet(&yoke, 0.014, 3),
        0.008,
        M.rubber,
        verts = 8,
        round = 1
    );
}

/// `n` strings from the bridge (height `y0`, depth `z0`, spread `x_bridge`) to the nut (`y1`, `z1`).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn strings(
    fr: &GameFrame,
    n: usize,
    x_bridge: f64,
    x_nut: f64,
    y0: f64,
    y1: f64,
    z0: f64,
    z1: f64,
) {
    for k in 0..n {
        let f = k as f64 - (n as f64 - 1.0) / 2.0;
        rod!(
            format!("string{k}"),
            fr.at(f * x_bridge, y0, z0),
            fr.at(f * x_nut, y1, z1),
            0.0007,
            material = M.string,
            verts = 4
        );
    }
}

/// `n` frets down the fretboard from the nut at height `nut` (scale length `scale`), at depth `z`.
pub fn frets(fr: &GameFrame, n: usize, scale: f64, nut: f64, w: f64, z: f64) {
    for k in 1..=n {
        fbox!(
            fr,
            format!("fret{k}"),
            w,
            0.0015,
            0.002,
            (0, nut - scale * (1.0 - pow(2.0, -(k as f64) / 12.0)), z),
            M.fret
        );
    }
}

/// `matrix.to_euler('XYZ')` in degrees, as mathutils gives it: the turn (single precision) of the two that reach it
/// with the least sum of angles, after each column is made a unit long.
/// (Not `math::mat3_normalized_to_eul` on the matrix as it stands: that leaves out the columns made a unit long, and
/// twelve more of the drum kit's points come off Blender's with it.)
pub fn to_euler_degrees(m: M4) -> P {
    let mut c = [[0.0_f32; 3]; 3];
    for (i, col) in c.iter_mut().enumerate() {
        let v = V3::new(m.0[i][0], m.0[i][1], m.0[i][2]);
        let n = v.normalized();
        *col = [n.x, n.y, n.z];
    }
    let at2 = |y: f32, x: f32| artkit::math::atan2(y as f64, x as f64) as f32;
    let cy = artkit::py::hypot(c[0][0] as f64, c[0][1] as f64) as f32;
    let (e1, e2) = if cy > 16.0 * f32::EPSILON {
        (
            [
                at2(c[1][2], c[2][2]),
                at2(-c[0][2], cy),
                at2(c[0][1], c[0][0]),
            ],
            [
                at2(-c[1][2], -c[2][2]),
                at2(-c[0][2], -cy),
                at2(-c[0][1], -c[0][0]),
            ],
        )
    } else {
        let e = [at2(-c[2][1], c[1][1]), at2(-c[0][2], cy), 0.0];
        (e, e)
    };
    let d1 = e1[0].abs() + e1[1].abs() + e1[2].abs();
    let d2 = e2[0].abs() + e2[1].abs() + e2[2].abs();
    let e = if d1 > d2 { e2 } else { e1 };
    (
        degrees(e[0] as f64),
        degrees(e[1] as f64),
        degrees(e[2] as f64),
    )
}
