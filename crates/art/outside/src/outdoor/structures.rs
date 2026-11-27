//! O9: the garden's built things — the carport, the gazebo, the pergola, the awning and the solar array.

use super::common::M;
use artkit::math;
use artkit::parts::{join_all, settle};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fextrude, fsphere, settle};

/// Four square corner posts of width `t` on a `w` x `d` plan (the carport's and the gazebo's), each `hs` high so a
/// roof can fall.
fn corner_posts(f: &GameFrame, w: f64, d: f64, t: f64, hs: [f64; 4]) {
    for (i, (sx, sz)) in [(-1.0f64, -1.0f64), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            f,
            format!("post{i}"),
            t,
            hs[i],
            t,
            (sx * (w / 2.0 - t / 2.0), hs[i] / 2.0, sz * (d / 2.0 - t / 2.0)),
            M.struct_timber,
            bevel = 0.008
        );
    }
}

// --- O9: the carport: four posts, two beams with knee braces, the rafters and a roof that falls to one side ------
pub fn carport() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (w, d, h) = (2.90f64, 4.90f64, 2.26f64);
    // the roof falls 0.14 m across its width, high on the left
    let slope = 0.14 / (w - 0.14);
    let ang = slope.atan().to_degrees();
    let post_h = |sx: f64| h - slope * sx * (w / 2.0 - 0.07);
    corner_posts(&f, w, d, 0.14, [post_h(-1.0), post_h(1.0), post_h(-1.0), post_h(1.0)]);
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        let top = post_h(sx);
        fbox!(
            f,
            format!("beam{i}"),
            0.12,
            0.20,
            d,
            (sx * (w / 2.0 - 0.06), top + 0.10, 0),
            M.struct_timber,
            bevel = 0.008
        );
        // knee braces from each end post up to the beam, leaning in along the beam
        for (k, sz) in [-1.0f64, 1.0].into_iter().enumerate() {
            fbox!(
                f,
                format!("brace{i}{k}"),
                0.08,
                0.62,
                0.08,
                (sx * (w / 2.0 - 0.07), top - 0.22, sz * (d / 2.0 - 0.07 - 0.22)),
                M.struct_timber,
                bevel = 0.006,
                rot = (-sz * 45.0, 0, 0)
            );
        }
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
            bevel = 0.006,
            rot = (0, ang, 0)
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
        bevel = 0.006,
        rot = (0, ang, 0)
    );
    // fascia boards along the roof's front and back edges
    for (i, sz) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("fascia{i}"),
            w + 0.10,
            0.16,
            0.03,
            (0, h + 0.28, sz * (d / 2.0 + 0.04)),
            M.struct_timber,
            bevel = 0.004,
            rot = (0, ang, 0)
        );
    }
    settle(&[join_all("body")])
}

// --- O9: the gazebo: four posts, its ring beam, a hipped roof with its finial, balusters and a bench along the back ----
pub fn gazebo() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (w, d, h) = (2.84, 2.84, 2.30);
    corner_posts(&f, w, d, 0.13, [h; 4]);
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
    // the roof: a hipped pyramid (a four-sided cone turned to the plan) on a thin eaves skirt, a finial on its peak
    let rise = 0.47;
    fbox!(f, "skirt", 2.96, 0.07, 2.96, (0, h + 0.195, 0), M.struct_roof, bevel = 0.006);
    fcone!(
        f,
        "roof",
        2.96 / 2.0 * 2f64.sqrt(),
        0.05,
        rise,
        (0, h + 0.23 + rise / 2.0, 0),
        M.struct_roof,
        verts = 4,
        smooth = false,
        rot = (0, 0, 45)
    );
    fsphere!(f, "finial", 0.07, (0, h + 0.23 + rise + 0.03, 0), M.struct_roof);
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
        // balusters between the two rails
        for j in 1..7 {
            let u = (j as f64 / 7.0 - 0.5) * (len - 0.26);
            let (bx, bz) = if along_x { (u, 0.0) } else { (0.0, u) };
            fbox!(
                f,
                format!("baluster{i}{j}"),
                0.04,
                0.70,
                0.04,
                (sx * (w / 2.0 - 0.065) + bx, 0.675, sz * (d / 2.0 - 0.065) + bz),
                M.struct_timber,
                bevel = 0.004
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

// --- O9: the awning: a rail on the wall, its striped fabric sloping down to the front bar, the arms and the valance ---------
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
    // the fabric: eight stripes in one sloping sheet from the rail down to the front bar
    let (z0, z1, y0, y1) = (back + 0.07, back + 0.88, 2.175, 1.995);
    let (len, ang) = ((z1 - z0).hypot(y0 - y1), ((y0 - y1) / (z1 - z0)).atan().to_degrees());
    for i in 0..8 {
        fbox!(
            f,
            format!("fabric{i}"),
            1.88 / 8.0,
            0.02,
            len,
            (-0.94 + 1.88 / 8.0 * (i as f64 + 0.5), (y0 + y1) / 2.0, (z0 + z1) / 2.0),
            if i % 2 == 0 {
                M.awning_stripe
            } else {
                M.awning_stripe2
            },
            bevel = 0.004,
            rot = (ang, 0, 0)
        );
    }
    // a fabric cheek at each end, hanging under the sheet down to the arm
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fextrude!(
            f,
            format!("cheek{i}"),
            &[(z0, y0 - 0.02), (z1, y1 - 0.02), (z0, 2.00)],
            0.012,
            (sx * 0.94, 0, 0),
            M.awning_stripe,
            plane = Facing::Side,
            bevel = 0.004
        );
    }
    // the folding arms: from the wall bracket out to the front bar, sloping with the sheet
    let (al, aa) = ((z1 - z0).hypot(2.00 - 1.975), ((2.00 - 1.975) / (z1 - z0)).atan().to_degrees());
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("arm{i}"),
            0.04,
            0.04,
            al,
            (sx * 0.90, 1.9875, (z0 + z1) / 2.0),
            M.panel_frame,
            bevel = 0.005,
            rot = (aa, 0, 0)
        );
        fbox!(
            f,
            format!("bracket{i}"),
            0.05,
            0.14,
            0.04,
            (sx * 0.90, 2.11, back + 0.02),
            M.panel_frame,
            bevel = 0.005
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
    settle(&[join_all("body")])
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
