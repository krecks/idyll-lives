//! O9: the garden's built things — the carport, the gazebo, the pergola, the awning and the solar array.

use super::common::M;
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsphere, settle};

// --- O9: the carport: four posts, two beams and the rafters under its roof --------------------------------
pub fn carport() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (w, d, h) = (2.90, 4.90, 2.35);
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
            (sx * (w / 2.0 - 0.07), h / 2.0, sz * (d / 2.0 - 0.07)),
            M.struct_timber,
            bevel = 0.008
        );
    }
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("beam{i}"),
            0.12,
            0.20,
            d,
            (sx * (w / 2.0 - 0.06), h + 0.10, 0),
            M.struct_timber,
            bevel = 0.008
        );
    }
    for i in 0..5 {
        let z = -d / 2.0 + 0.20 + i as f64 * (d - 0.40) / 4.0;
        fbox!(
            f,
            format!("rafter{i}"),
            w,
            0.12,
            0.10,
            (0, h + 0.26, z),
            M.struct_timber,
            bevel = 0.006
        );
    }
    fbox!(
        f,
        "roof",
        w + 0.10,
        0.05,
        d + 0.10,
        (0, h + 0.345, 0),
        M.struct_roof,
        bevel = 0.006
    );
    settle!();
    meshes()
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
    settle!();
    meshes()
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

// --- O9: the solar array: its frame on the ground and the two panels --------------------------------------
pub fn solar_panels() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    for (i, (sx, sz)) in [(-1.0f64, -1.0f64), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            f,
            format!("leg{i}"),
            0.06,
            0.55,
            0.06,
            (sx * 0.82, 0.275, sz * 0.78),
            M.panel_frame,
            bevel = 0.006
        );
    }
    for (i, sz) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("rail{i}"),
            1.76,
            0.06,
            0.06,
            (0, 0.55, sz * 0.78),
            M.panel_frame,
            bevel = 0.006
        );
    }
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("panel{i}"),
            0.84,
            0.05,
            1.64,
            (sx * 0.43, 0.605, 0),
            M.panel_glass,
            bevel = 0.008
        );
        fbox!(
            f,
            format!("edge{i}"),
            0.86,
            0.02,
            1.66,
            (sx * 0.43, 0.575, 0),
            M.panel_frame,
            bevel = 0.004
        );
    }
    fbox!(
        f,
        "junction",
        0.12,
        0.10,
        0.08,
        (0.75, 0.60, 0.70),
        M.struct_roof,
        bevel = 0.006
    );
    settle!();
    meshes()
}
