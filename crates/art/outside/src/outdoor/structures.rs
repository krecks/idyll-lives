//! O9: the garden's built things — the carport, the gazebo, the pergola, the awning and the solar array.

use super::common::M;
use artkit::math;
use artkit::parts::{join_all, settle};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsphere, settle};

// --- O9: the carport: four posts, eaves beams, two trusses and the rafters under a pitched roof -----------------
/// The roof's pitch (degrees from level), each side of the ridge.
const CARPORT_PITCH: f64 = 20.0;

pub fn carport() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (sin, cos) = (
        math::sin(CARPORT_PITCH.to_radians()),
        math::cos(CARPORT_PITCH.to_radians()),
    );
    // posts 2.15 m to the eaves beams' underside (a car's clear height), the beams 0.18 deep on them
    let (px, pz, h, beam) = (1.28, 2.30, 2.15, 0.18);
    // the rafters' underside sits 2 cm into the beams' top at the posts' line
    let ridge = h + beam - 0.02 + px * sin / cos;
    // a point in one slope (`sx` the side): `s` down the slope from the ridge, `n` out of the rafters' top
    let at = |sx: f64, s: f64, n: f64| (sx * (s * cos + n * sin), ridge - s * sin + n * cos);
    let reach = 1.43 / cos; // the slope's length out to the eaves, the sheet 2 cm in from the footprint's edge
    for (i, (sx, sz)) in [(-1.0f64, -1.0f64), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            f,
            format!("post{i}"),
            0.14,
            h,
            0.14,
            (sx * px, h / 2.0, sz * pz),
            M.struct_timber,
            bevel = 0.008
        );
        // a knee brace from the post to the beam, inwards along the length
        fbox!(
            f,
            format!("brace{i}"),
            0.08,
            0.62,
            0.08,
            (sx * px, h - 0.22, sz * (pz - 0.22)),
            M.struct_timber,
            rot = (-sz * 45.0, 0.0, 0.0),
            bevel = 0.005
        );
    }
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("beam{i}"),
            0.14,
            beam,
            4.90,
            (sx * px, h + beam / 2.0, 0),
            M.struct_timber,
            bevel = 0.008
        );
    }
    // a truss over each pair of posts: the tie beam across between the eaves beams and a king post up to the ridge
    for (i, sz) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("tie{i}"),
            2.0 * px - 0.14,
            0.14,
            0.08,
            (0, h + beam - 0.07, sz * pz),
            M.struct_timber,
            bevel = 0.006
        );
        let top = ridge - 0.10;
        let low = h + beam;
        fbox!(
            f,
            format!("king{i}"),
            0.10,
            top - low,
            0.08,
            (0, (top + low) / 2.0, sz * pz),
            M.struct_timber,
            bevel = 0.006
        );
    }
    fbox!(
        f,
        "ridge",
        0.06,
        0.20,
        4.96,
        (0, ridge - 0.10, 0),
        M.struct_timber,
        bevel = 0.006
    );
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        let tilt = (0.0, -sx * CARPORT_PITCH, 0.0);
        // the rafters, 0.6 m apart, notched over the ridge beam and the eaves beam
        for k in 0..9 {
            let z = -2.40 + k as f64 * 0.60;
            let (x, y) = at(sx, reach / 2.0 + 0.02, -0.06);
            fbox!(
                f,
                format!("rafter{i}{k}"),
                reach - 0.02,
                0.12,
                0.05,
                (x, y, z),
                M.struct_timber,
                rot = tilt,
                bevel = 0.005
            );
        }
        // the barge boards up each gable's edge, the fascia along the eaves, the roof sheet over them all
        for (k, sz) in [-1.0f64, 1.0].into_iter().enumerate() {
            let (x, y) = at(sx, reach / 2.0, -0.04);
            fbox!(
                f,
                format!("barge{i}{k}"),
                reach + 0.02,
                0.18,
                0.03,
                (x, y, sz * 2.465),
                M.struct_timber,
                rot = tilt,
                bevel = 0.006
            );
        }
        let (x, y) = at(sx, reach, -0.05);
        fbox!(
            f,
            format!("fascia{i}"),
            0.03,
            0.16,
            4.96,
            (x + sx * 0.015, y, 0),
            M.struct_timber,
            bevel = 0.006
        );
        let (x, y) = at(sx, reach / 2.0 + 0.01, 0.015);
        fbox!(
            f,
            format!("roof{i}"),
            reach + 0.06,
            0.03,
            5.00,
            (x, y, 0),
            M.struct_roof,
            rot = tilt,
            bevel = 0.006
        );
    }
    fbox!(
        f,
        "ridgeCap",
        0.20,
        0.04,
        5.00,
        (0, ridge + 0.04, 0),
        M.struct_roof,
        bevel = 0.006
    );
    // no clip moves a part: all of it is the still body
    settle(&[join_all("body")])
}

// --- O9: the gazebo: four posts, its ring beam, the stepped roof with its finial and a bench along the back -----------------------
pub fn gazebo() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (w, d, h) = (2.84, 2.84, 2.30);
    for (i, (sx, sz)) in [(-1.0f64, -1.0f64), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            f,
            format!("post{i}"),
            0.13,
            h,
            0.13,
            (sx * (w / 2.0 - 0.065), h / 2.0, sz * (d / 2.0 - 0.065)),
            M.struct_timber,
            bevel = 0.008
        );
    }
    for (i, sz) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("beam{i}"),
            w + 0.13,
            0.16,
            0.11,
            (0, h + 0.08, sz * d / 2.0),
            M.struct_timber,
            bevel = 0.006
        );
    }
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("side{i}"),
            0.11,
            0.16,
            d,
            (sx * w / 2.0, h + 0.08, 0),
            M.struct_timber,
            bevel = 0.006
        );
    }
    // the roof: a broad skirt, the lantern over it and its own cap
    fbox!(
        f,
        "roof",
        2.96,
        0.10,
        2.96,
        (0, h + 0.21, 0),
        M.struct_roof,
        bevel = 0.008
    );
    fbox!(
        f,
        "lantern",
        1.50,
        0.30,
        1.50,
        (0, h + 0.41, 0),
        M.struct_timber,
        bevel = 0.006
    );
    fbox!(
        f,
        "cap",
        1.90,
        0.09,
        1.90,
        (0, h + 0.605, 0),
        M.struct_roof,
        bevel = 0.008
    );
    fsphere!(f, "finial", 0.07, (0, h + 0.72, 0), M.struct_roof);
    // the railing: two rails on the back and the two sides
    for (i, (sx, sz, len, along_x)) in [
        (0.0f64, -1.0f64, w, true),
        (-1.0, 0.0, d, false),
        (1.0, 0.0, d, false),
    ]
    .into_iter()
    .enumerate()
    {
        for (k, y) in [1.05, 0.30].into_iter().enumerate() {
            let (bw, bd) = if along_x { (len, 0.06) } else { (0.06, len) };
            fbox!(
                f,
                format!("rail{i}{k}"),
                bw,
                0.07,
                bd,
                (sx * (w / 2.0 - 0.065), y, sz * (d / 2.0 - 0.065)),
                M.struct_timber,
                bevel = 0.005
            );
        }
    }
    // the bench along the back, inside the rail: three boards at 0.45 m, two back boards on the back legs, three
    // frames (content `places` sit three at depth -1.10, 0.75 m apart)
    let (front, rear) = (-1.30 + 0.42, -1.30);
    for (k, dz) in [0.065, 0.205, 0.345].into_iter().enumerate() {
        fbox!(
            f,
            format!("bench_board{k}"),
            2.54,
            0.035,
            0.125,
            (0, 0.4325, rear + dz),
            M.struct_timber,
            bevel = 0.006
        );
    }
    for (k, y) in [0.62, 0.79].into_iter().enumerate() {
        fbox!(
            f,
            format!("bench_back{k}"),
            2.54,
            0.10,
            0.025,
            (0, y, rear + 0.0625),
            M.struct_timber,
            bevel = 0.006
        );
    }
    for (k, x) in [-1.2f64, 0.0, 1.2].into_iter().enumerate() {
        fbox!(
            f,
            format!("bench_leg{k}a"),
            0.05,
            0.415,
            0.05,
            (x, 0.2075, front - 0.04),
            M.struct_timber,
            bevel = 0.005
        );
        fbox!(
            f,
            format!("bench_leg{k}b"),
            0.05,
            0.85,
            0.05,
            (x, 0.425, rear + 0.025),
            M.struct_timber,
            bevel = 0.005
        );
        fbox!(
            f,
            format!("bench_bearer{k}"),
            0.05,
            0.06,
            0.40,
            (x, 0.385, rear + 0.21),
            M.struct_timber,
            bevel = 0.005
        );
    }
    // no clip moves a part: all of it is the still body
    settle(&[join_all("body")])
}

// --- O9: the pergola: four posts, two beams and the rafters over them -------------------------------------
pub fn pergola() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (w, d, h) = (2.80, 1.80, 2.10);
    for (i, (sx, sz)) in [(-1.0f64, -1.0f64), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            f,
            format!("post{i}"),
            0.11,
            h,
            0.11,
            (sx * (w / 2.0 - 0.055), h / 2.0, sz * (d / 2.0 - 0.055)),
            M.struct_timber,
            bevel = 0.006
        );
    }
    for (i, sz) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("beam{i}"),
            w + 0.12,
            0.18,
            0.11,
            (0, h + 0.09, sz * (d / 2.0 - 0.055)),
            M.struct_timber,
            bevel = 0.006
        );
    }
    for i in 0..5 {
        let x = -w / 2.0 + i as f64 * w / 4.0;
        fbox!(
            f,
            format!("rafter{i}"),
            0.09,
            0.14,
            d + 0.10,
            (x, h + 0.25, 0),
            M.struct_timber,
            bevel = 0.005
        );
    }
    settle!();
    meshes()
}

// --- O9: the awning: a rail on the wall, its slats stepping down to the front bar and the valance ---------
pub fn awning() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let back = -0.43;
    fbox!(
        f,
        "rail",
        1.90,
        0.07,
        0.070,
        (0, 2.18, back + 0.035),
        M.panel_frame,
        bevel = 0.006
    );
    for i in 0..5 {
        let y = 2.160 - 0.035 * i as f64;
        let z = back + 0.100 + 0.175 * i as f64;
        fbox!(
            f,
            format!("slat{i}"),
            1.88,
            0.030,
            0.200,
            (0, y, z),
            if i % 2 == 0 {
                M.awning_stripe
            } else {
                M.awning_stripe2
            },
            bevel = 0.006
        );
    }
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("arm{i}"),
            0.05,
            0.05,
            0.86,
            (sx * 0.90, 2.00, back + 0.470),
            M.panel_frame,
            bevel = 0.006
        );
    }
    fbox!(
        f,
        "bar",
        1.90,
        0.06,
        0.06,
        (0, 1.97, back + 0.880),
        M.panel_frame,
        bevel = 0.006
    );
    fbox!(
        f,
        "valance",
        1.90,
        0.14,
        0.020,
        (0, 1.89, back + 0.880),
        M.awning_stripe,
        bevel = 0.004
    );
    settle!();
    meshes()
}

// --- O9: the solar array: two 60-cell modules tilted at the sun on a low braced ground frame -------------
/// The modules' tilt (degrees from level), rising to the back.
const SOLAR_TILT: f64 = 30.0;

pub fn solar_panels() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (sin, cos) = (
        math::sin(SOLAR_TILT.to_radians()),
        math::cos(SOLAR_TILT.to_radians()),
    );
    // a point in the modules' plane: `s` up the slope from their middle, `n` out of their face
    let at = |s: f64, n: f64| (0.80 + s * sin + n * cos, -s * cos + n * sin);
    let tilt = (SOLAR_TILT, 0.0, 0.0);
    let (len, wide) = (1.66, 1.00);
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        let x = sx * (wide + 0.02) / 2.0;
        // the module: its aluminium rim, the glass set in it, the cells' gaps on the glass (6 across, 10 up)
        let (y, z) = at(0.0, -0.0175);
        fbox!(
            f,
            format!("module{i}"),
            wide,
            0.035,
            len,
            (x, y, z),
            M.panel_frame,
            rot = tilt,
            bevel = 0.004
        );
        let (y, z) = at(0.0, 0.003);
        fbox!(
            f,
            format!("glass{i}"),
            wide - 0.03,
            0.006,
            len - 0.03,
            (x, y, z),
            M.panel_glass,
            rot = tilt,
            bevel = 0
        );
        let (y, z) = at(0.0, 0.0065);
        for c in 1..6 {
            let cx = x - (wide - 0.03) / 2.0 + c as f64 * (wide - 0.03) / 6.0;
            fbox!(
                f,
                format!("gapUp{i}"),
                0.008,
                0.002,
                len - 0.04,
                (cx, y, z),
                M.panel_frame,
                rot = tilt,
                bevel = 0
            );
        }
        for r in 1..10 {
            let (y, z) = at(-(len - 0.03) / 2.0 + r as f64 * (len - 0.03) / 10.0, 0.0065);
            fbox!(
                f,
                format!("gapAcross{i}"),
                wide - 0.04,
                0.002,
                0.008,
                (x, y, z),
                M.panel_frame,
                rot = tilt,
                bevel = 0
            );
        }
    }
    // two purlins across under the modules, on two A-frames: a rafter up the slope, a short post at the front, a
    // tall one at the back, a foot rail between them on the ground and a brace from the front foot up the back post
    for (i, s) in [-0.55f64, 0.55].into_iter().enumerate() {
        let (y, z) = at(s, -0.055);
        fbox!(
            f,
            format!("purlin{i}"),
            2.02,
            0.04,
            0.04,
            (0, y, z),
            M.panel_frame,
            rot = tilt,
            bevel = 0.004
        );
    }
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        let x = sx * 0.70;
        let (y, z) = at(0.0, -0.105);
        fbox!(
            f,
            format!("rafter{i}"),
            0.05,
            0.06,
            1.52,
            (x, y, z),
            M.panel_frame,
            rot = tilt,
            bevel = 0.004
        );
        let (front, back) = (at(-0.65, -0.135), at(0.65, -0.135));
        for (name, (top, z)) in [("postFront", front), ("postBack", back)] {
            fbox!(
                f,
                format!("{name}{i}"),
                0.05,
                top,
                0.05,
                (x, top / 2.0, z),
                M.panel_frame,
                bevel = 0.004
            );
        }
        fbox!(
            f,
            format!("foot{i}"),
            0.06,
            0.04,
            front.1 - back.1 + 0.05,
            (x, 0.02, (front.1 + back.1) / 2.0),
            M.panel_frame,
            bevel = 0.004
        );
        // the brace: from the front post's foot to 0.8 m up the back post
        let (dz, dy) = (front.1 - back.1, 0.78);
        let angle = math::atan2(dy, dz).to_degrees();
        let length = math::sqrt(dz * dz + dy * dy);
        fbox!(
            f,
            format!("brace{i}"),
            0.04,
            0.03,
            length,
            (x, 0.05 + dy / 2.0, (front.1 + back.1) / 2.0),
            M.panel_frame,
            rot = (angle, 0.0, 0.0),
            bevel = 0.004
        );
    }
    settle(&[join_all("body")])
}
