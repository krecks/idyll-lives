//! An oven's parts: its door, the cavity behind it with a rack and a lamp, the control fascia, a storage drawer, a
//! stock pot on the hob and a baking tray.

use super::M;
use super::common::hollow;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::parts::{hinged, join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring};

/// A wire rack lying at height `y`: a rounded frame of rod and `bars` rods across it, front to back.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn rack(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    d: f64,
    bars: usize,
    material: Option<Mat>,
) {
    let lp = rounded_rect(w, d, 0.015, 2);
    let pts: Vec<(f64, f64, f64)> = lp.iter().map(|&(u, v)| (x + u, y, z + v)).collect();
    fsweep!(
        f,
        name,
        &pts,
        0.003,
        material.unwrap_or(M.tray),
        verts = 4,
        closed = true
    );
    for i in 0..bars {
        let u = -w / 2.0 + w * (i + 1) as f64 / (bars + 1) as f64;
        fsweep!(
            f,
            format!("{name}_bar{i}"),
            &[(x + u, y, z - d / 2.0), (x + u, y, z + d / 2.0)],
            0.0025,
            material.unwrap_or(M.tray),
            verts = 4,
            caps = false
        );
    }
}

/// An oven door: a dark front framing a window of black glass, a bar handle on posts along its top rail, an inner
/// pane behind (what shows when it's down); hinged along its bottom edge. Returns it; the caller swings it once the
/// model has settled.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn oven_door(
    f: &GameFrame,
    name: &str,
    x: f64,
    w: f64,
    y: f64,
    z: f64,
    hinge_y: f64,
    h: f64,
    out: f64,
) -> Obj {
    let parts = new_parts(|| {
        let window: Vec<(f64, f64)> = rounded_rect(w - 0.14, 0.30, 0.02, 2)
            .into_iter()
            .map(|(u, v)| (u, v + 0.01))
            .collect();
        fextrude!(
            f,
            format!("{name}_front"),
            &rounded_rect(w, h, 0.012, 2),
            0.025,
            (x, y, z),
            M.dark,
            holes = [window],
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            format!("{name}_window"),
            w - 0.13,
            0.31,
            0.008,
            (x, y + 0.01, z - 0.006),
            M.hob
        );
        fbox!(
            f,
            format!("{name}_inner"),
            w - 0.08,
            h - 0.08,
            0.006,
            (x, y, z - 0.014),
            M.tray
        );
        pull!(
            f,
            format!("{name}_handle"),
            (x - (w / 2.0 - 0.06), y + 0.205),
            (x + (w / 2.0 - 0.06), y + 0.205),
            z + 0.0125,
            out = out,
            w = 0.022,
            r = None,
            verts = 8
        );
    });
    hinged(join(&parts, name, None), f.at(x, hinge_y, z))
}

/// An oven's inside, cut out of the carcass behind its door (front at depth `z`): dark enamel walls, a wire rack
/// on runners and a lamp in the back corner.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn oven(
    f: &GameFrame,
    carcass: Obj,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    d: f64,
) {
    hollow(
        carcass,
        f,
        &format!("{name}_cavity"),
        w,
        h,
        d,
        (x, y, z - d / 2.0 + 0.002),
        None,
    );
    rack(
        f,
        &format!("{name}_rack"),
        x,
        y - h * 0.12,
        z - d / 2.0,
        w - 0.02,
        d - 0.04,
        5,
        None,
    );
    for s in [-1, 1] {
        fbox!(
            f,
            format!("{name}_runner{s}"),
            0.012,
            0.008,
            d - 0.04,
            (
                x + s as f64 * (w / 2.0 - 0.006),
                y - h * 0.12 - 0.006,
                z - d / 2.0
            ),
            M.tray
        );
    }
    fcyl!(
        f,
        format!("{name}_lamp"),
        0.02,
        0.008,
        (x - w / 2.0 + 0.05, y + h / 2.0 - 0.05, z - d + 0.004),
        M.plate,
        verts = 10,
        rot = (90, 0, 0)
    );
}

/// A control fascia `w` × `h` on the front at depth `z` (its back), its edges rounded; `display`: a small clock
/// display at that x.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn strip(
    f: &GameFrame,
    name: &str,
    w: f64,
    y: f64,
    z: f64,
    h: f64,
    x: f64,
    display: Option<f64>,
) {
    fextrude!(
        f,
        name,
        &rounded_rect(w, h, 0.008, 2),
        0.012,
        (x, y, z + 0.006),
        M.dark,
        bevel = 0.003,
        bevel_segments = 1
    );
    if let Some(display) = display {
        fbox!(
            f,
            format!("{name}_display"),
            0.07,
            0.025,
            0.004,
            (display, y, z + 0.013),
            M.hob,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            format!("{name}_digits"),
            0.04,
            0.012,
            0.002,
            (display, y, z + 0.0155),
            M.display
        );
    }
}

/// A storage drawer's front with a long bar pull along its top.
pub fn drawer_front(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    at: (i32, f64, f64),
    material: Mat,
    pull: f64,
) {
    let (x, y, z) = at;
    let x = x as f64;
    fextrude!(
        f,
        name,
        &rounded_rect(w, h, 0.008, 2),
        0.02,
        at,
        material,
        bevel = 0.004,
        bevel_segments = 1
    );
    pull!(
        f,
        format!("{name}_pull"),
        (x - pull / 2.0, y + h / 2.0 - 0.035),
        (x + pull / 2.0, y + h / 2.0 - 0.035),
        z + 0.01,
        out = 0.02,
        w = 0.014,
        r = None,
        verts = 6
    );
}

/// A stock pot standing on game height `y`: a rolled rim, loop handles each side, a lid with a knob.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn stock_pot(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64, h: f64, material: Mat) {
    let b = f.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (r * 0.96, 0.0),
            (r, 0.01),
            (r, h - 0.006),
            (r * 1.03, h),
            (r * 0.97, h),
            (r * 0.97, h * 0.6),
            (0.0, h * 0.6)
        ],
        at = (bx, by, bz),
        material = material,
        segments = 16
    );
    revolve!(
        format!("{name}_lid"),
        &[
            (0.0, 0.0),
            (r * 1.02, 0.0),
            (r * 0.98, 0.012),
            (r * 0.5, 0.022),
            (0.0, 0.024)
        ],
        at = (bx, by, bz + h),
        material = material,
        segments = 16
    );
    fsphere!(
        f,
        format!("{name}_knob"),
        0.014,
        (x, y + h + 0.03, z),
        M.pan,
        segments = 8,
        rings = 5,
        scale = (1, 1, 0.7)
    );
    for s in [-1, 1] {
        let s = s as f64;
        fsweep!(
            f,
            format!("{name}_handle{}", s as i32),
            &[
                (x + s * r * 0.98, y + h - 0.035, z - 0.03),
                (x + s * (r + 0.025), y + h - 0.03, z - 0.025),
                (x + s * (r + 0.025), y + h - 0.03, z + 0.025),
                (x + s * r * 0.98, y + h - 0.035, z + 0.03)
            ],
            0.005,
            material,
            verts = 6,
            resolution = 2
        );
    }
}

/// An oven's still front (one that never opens): as `oven_door`, joined into the body.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn oven_face(f: &GameFrame, name: &str, x: f64, w: f64, y: f64, z: f64, h: f64, out: f64) {
    let window: Vec<(f64, f64)> = rounded_rect(w - 0.14, 0.30, 0.02, 2)
        .into_iter()
        .map(|(u, v)| (u, v + 0.01))
        .collect();
    fextrude!(
        f,
        format!("{name}_front"),
        &rounded_rect(w, h, 0.012, 2),
        0.025,
        (x, y, z),
        M.dark,
        holes = [window],
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        f,
        format!("{name}_window"),
        w - 0.13,
        0.31,
        0.008,
        (x, y + 0.01, z - 0.006),
        M.hob
    );
    pull!(
        f,
        format!("{name}_handle"),
        (x - (w / 2.0 - 0.06), y + 0.205),
        (x + (w / 2.0 - 0.06), y + 0.205),
        z + 0.0125,
        out = out,
        w = 0.022,
        r = None,
        verts = 8
    );
}

/// A baking tray on game height `y`: a flat floor inside a rolled rim `h` high, in one piece.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn tray(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, w: f64, d: f64, h: f64) {
    let b = f.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let ring =
        |a: f64, b: f64, t: f64| ring!(a, b, z = bz + t, verts = 20, square = 10.0, at = (bx, by));
    loft!(
        name,
        &[
            ring(w / 2.0 - 0.006, d / 2.0 - 0.006, 0.0),
            ring(w / 2.0, d / 2.0, h),
            ring(w / 2.0 - 0.008, d / 2.0 - 0.008, h),
            ring(w / 2.0 - 0.014, d / 2.0 - 0.014, 0.004),
            ring(0.0, 0.0, 0.004)
        ],
        material = M.tray
    );
}
