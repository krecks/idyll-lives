//! The islands and bars (K6). A freestanding island: cupboards at the front (+Z), where residents stand to use it, as
//! the kitchen island; the worktop overhangs the back (-Z) 30 cm for stools, its edge on the wall face. A bar: a
//! slatted body, a top to 1.06 overhanging the front (+Z) and a foot rail.

use super::common::{arc, drawer, pull, shaker};
use super::wall::corbel;
use super::{M, SECONDS};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, slide};

/// An island's worktop, `w` by `d` centred at depth `z`, 0.90 to 0.95: its corners rounded in plan, its edges eased
/// all round (it stands free).
fn island_top(f: &GameFrame, w: f64, d: f64, z: f64, material: Mat) {
    fextrude!(
        f,
        "worktop",
        &rounded_rect(w, d, 0.025, 3),
        0.05,
        (0, 0.925, z),
        material,
        plane = Facing::Top,
        bevel = 0.008,
        bevel_segments = 2,
        smooth = true,
        angle = 50
    );
}

/// The back of an island's carcass (the stools' side, its face at depth `z`): `n` applied shaker panels between y0
/// and y1, and corbels at `corbels` (x) holding the overhang up.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn island_back(
    f: &GameFrame,
    w: f64,
    z: f64,
    y0: f64,
    y1: f64,
    n: i64,
    material: Mat,
    corbels: &[f64],
) {
    let pw = (w - 0.04) / n as f64;
    for i in 0..n {
        let x = -(w - 0.04) / 2.0 + pw * (i as f64 + 0.5);
        fextrude!(
            f,
            format!("back_panel{i}"),
            &rounded_rect(pw - 0.03, y1 - y0, 0.004, 1),
            0.01,
            (x, (y0 + y1) / 2.0, z - 0.005),
            material,
            holes = vec![rounded_rect(pw - 0.13, y1 - y0 - 0.1, 0.003, 1)],
            bevel = 0.003,
            bevel_segments = 1
        );
        // the field inside, its corners clear of the frame (the baked shading darkens a face by its corners)
        fbox!(
            f,
            format!("back_field{i}"),
            pw - 0.14,
            y1 - y0 - 0.11,
            0.006,
            (x, (y0 + y1) / 2.0, z - 0.003),
            material,
            bevel = 0.002,
            bevel_segments = 1
        );
    }
    for (i, &x) in corbels.iter().enumerate() {
        corbel(f, &format!("corbel{i}"), x, 0.9, z, 0.3, -0.24, material);
    }
}

/// A freestanding island unit as catalog2's `island_unit`: worktop to 0.95, shaker cupboards at the front, applied
/// panels on its ends and its back, corbels under the overhang.
fn island(f: &GameFrame, w: f64, body: Option<Mat>, top: Option<Mat>, handles: Option<Mat>) {
    let (body, top, handles) = (
        body.unwrap_or(M.cab),
        top.unwrap_or(M.worktop),
        handles.unwrap_or(M.chrome),
    );
    fbox!(
        f,
        "carcass",
        w - 0.16,
        0.82,
        0.58,
        (0, 0.51, 0.16),
        body,
        bevel = 0.012
    );
    fbox!(
        f,
        "plinth",
        w - 0.2,
        0.10,
        0.52,
        (0, 0.05, 0.16),
        M.dark,
        bevel = 0.004
    );
    island_top(f, w - 0.04, 0.90, 0.02, top);
    let n = 1.max(py::round((w - 0.16) / 0.46));
    let dw = (w - 0.18) / n as f64;
    for i in 0..n {
        let x = -(w - 0.18) / 2.0 + dw * (i as f64 + 0.5);
        shaker(
            f,
            &format!("door{i}"),
            dw - 0.006,
            0.72,
            x,
            0.5,
            0.445,
            body,
            0.06,
            0.02,
            None,
        );
        pull(
            f,
            format!("door{i}_pull"),
            x,
            0.83,
            0.465,
            0.14,
            Some(handles),
            false,
            0.016,
            0.005,
            0.0,
        );
    }
    let ex = (w - 0.16) / 2.0;
    for s in [-1, 1] {
        fbox!(
            f,
            format!("end{s}"),
            0.04,
            0.92,
            0.6,
            (s as f64 * ex, 0.46, 0.15),
            body,
            bevel = 0.008
        );
        // an applied frame on the end's outer face
        fextrude!(
            f,
            format!("end_frame{s}"),
            &rounded_rect(0.52, 0.74, 0.004, 1),
            0.01,
            (s as f64 * (ex + 0.025), 0.5, 0.15),
            body,
            holes = vec![rounded_rect(0.42, 0.64, 0.003, 1)],
            plane = Facing::Side,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            format!("end_field{s}"),
            0.006,
            0.63,
            0.41,
            (s as f64 * (ex + 0.023), 0.5, 0.15),
            body,
            bevel = 0.002,
            bevel_segments = 1
        );
    }
    let mut corbels: Vec<f64> = [-1, 1].map(|s| s as f64 * (ex - 0.06)).to_vec();
    if w > 2.0 {
        corbels.push(0.0);
    }
    island_back(
        f,
        w - 0.16,
        -0.13,
        0.14,
        0.86,
        1.max(py::round((w - 0.16) / 0.7)),
        body,
        &corbels,
    );
}

// --- the kitchen island: 2x1, two stools at the front, a drawer for a sandwich -----------------------
pub fn kitchen_island() -> Vec<Obj> {
    let f = &GameFrame::new(-0.0495);
    let still = new_parts(|| {
        // the cupboards face the front (+Z), where residents stand to use the island; the worktop
        // overhangs the back (-Z), where the stools are set (housegen sets them at a used table's back)
        fbox!(
            f,
            "carcass",
            1.84,
            0.86,
            0.56,
            (0, 0.47, 0.16),
            M.cab,
            bevel = 0.014
        );
        fbox!(
            f,
            "plinth",
            1.82,
            0.08,
            0.54,
            (0, 0.04, 0.16),
            M.dark,
            bevel = 0.004
        );
        island_top(f, 1.94, 0.82, 0.04, M.worktop);
        for s in [-1, 1] {
            // two doors under the drawers
            shaker(
                f,
                &format!("door{s}"),
                0.856,
                0.6,
                s as f64 * 0.47,
                0.35,
                0.43,
                M.cab,
                0.06,
                0.02,
                None,
            );
            pull(
                f,
                format!("door{s}_pull"),
                s as f64 * 0.47,
                0.615,
                0.45,
                0.14,
                None,
                false,
                0.019,
                0.005,
                0.0,
            );
            // a fixed drawer front either side of the one that opens, to match it
            shaker(
                f,
                &format!("drawer{s}"),
                0.443,
                0.17,
                s as f64 * 0.6775,
                0.77,
                0.43,
                M.cab,
                0.04,
                0.02,
                None,
            );
            pull(
                f,
                format!("drawer{s}_pull"),
                s as f64 * 0.6775,
                0.77,
                0.45,
                0.14,
                None,
                false,
                0.019,
                0.005,
                0.008,
            );
        }
        island_back(f, 1.84, -0.12, 0.1, 0.84, 3, M.cab, &[-0.86, 0.0, 0.86]);
        // no stools of its own, and a bare top: stools and the things on its top (a fruit bowl, mugs)
        // are things of their own (housegen sets them), so nothing is doubled or stands in a stool
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the drawer over the doors, at the front where residents stand
    let drawer = drawer(
        f, "drawer", 0.9, 0.77, 0.45, M.cab, None, 0.3, true, 0.019, 0.04, 0.17,
    );
    settle_all();
    // sandwich slides the drawer out to the front
    slide!(drawer, "drawer_open", (0, -0.25, 0), seconds = SECONDS);
    meshes()
}

// --- the small island: 1x1, cupboards at the front, a bare top (static) -----------------------------
pub fn island_small() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        island(f, 1.0, None, None, Some(M.brass)); // a bare top: housegen sets things on it
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the long island: 3x1, six cupboards at the front, a bare top (static) ---------------------------
pub fn island_long() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        island(f, 3.0, None, None, Some(M.metal)); // a bare top: housegen sets things on it
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- a bar: a slatted body, a top to 1.06 overhanging the front (+Z) and a foot rail (static) ---------
/// A bar as catalog2's `bar`: a body faced with rounded staves, a top at 1.06 overhanging the front (+Z), its nose
/// rolled and its ends square (so a run's tops meet), a brass foot rail on curved brackets.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn bar(
    f: &GameFrame,
    bw: f64,
    bx: f64,
    end: bool,
    body: Option<Mat>,
    top: Option<Mat>,
    rail: Option<Mat>,
) {
    let (body, top, rail) = (
        body.unwrap_or(M.walnut),
        top.unwrap_or(M.oak),
        rail.unwrap_or(M.brass),
    );
    fbox!(
        f,
        "body",
        bw,
        0.98,
        0.5,
        (bx, 0.49, -0.12),
        body,
        bevel = 0.01
    );
    let mut prof = vec![(-0.48, 1.0), (0.38, 1.0)];
    prof.extend_from_slice(&arc(0.38, 1.02, 0.02, -90, 0, 2)[1..]);
    prof.extend(arc(0.375, 1.035, 0.025, 0, 90, 3));
    prof.push((-0.468, 1.06));
    prof.extend_from_slice(&arc(-0.468, 1.048, 0.012, 90, 180, 2)[1..]);
    fextrude!(
        f,
        "top",
        &prof,
        1.0,
        (0, 0, 0),
        top,
        plane = Facing::Side,
        smooth = true
    );
    fbox!(
        f,
        "ledge",
        bw,
        0.04,
        0.26,
        (bx, 0.92, -0.36),
        top,
        bevel = 0.006
    );
    let n = py::int(bw / 0.1);
    for i in 0..n {
        // swept, so each stave has rings up its length for the baked shading to darken at its ends only
        let x = -0.5 + bw / n as f64 * (i as f64 + 0.5);
        let pts: Vec<(f64, f64, f64)> = (0..5)
            .map(|k| (x, 0.01 + 0.92 * k as f64 / 4.0, 0.135))
            .collect();
        fsweep!(
            f,
            format!("slat{i}"),
            &pts,
            0.0,
            body,
            profile = rounded_rect(bw / n as f64 - 0.012, 0.022, 0.0095, 3),
            up = (1, 0, 0),
            angle = 50
        );
    }
    // a rail across the top of the staves and a kick board along their foot
    fbox!(
        f,
        "apron",
        bw,
        0.06,
        0.03,
        (bx, 0.9, 0.15),
        body,
        bevel = 0.006,
        bevel_segments = 2
    );
    fbox!(
        f,
        "kick",
        bw,
        0.07,
        0.026,
        (bx, 0.035, 0.148),
        body,
        bevel = 0.005,
        bevel_segments = 1
    );
    let x1 = -0.5 + bw + if end { 0.02 } else { 0.0 };
    kit::tube!(
        f,
        "rail",
        &[(-0.5, 0.2, 0.26), (x1, 0.2, 0.26)],
        0.02,
        rail,
        verts = 12,
        round = 1
    );
    for x in [-0.38, -0.5 + bw - 0.12] {
        let xs = artkit::json::float_repr(x);
        fsweep!(
            f,
            format!("bracket{xs}"),
            &[
                (x, 0.12, 0.146),
                (x, 0.13, 0.2),
                (x, 0.165, 0.245),
                (x, 0.18, 0.26)
            ],
            0.009,
            rail,
            verts = 8,
            resolution = 2
        );
        fcyl!(
            f,
            format!("bracket_rose{xs}"),
            0.022,
            0.008,
            (x, 0.12, 0.148),
            rail,
            rot = (90, 0, 0),
            verts = 12
        );
    }
    if end {
        let mut side = vec![(-0.48, 0.0), (0.4, 0.0), (0.4, 0.985)];
        side.extend_from_slice(&arc(0.385, 0.985, 0.015, 0, 90, 2)[1..]);
        side.push((-0.48, 1.0));
        fextrude!(
            f,
            "waterfall",
            &side,
            0.06,
            (0.47, 0, 0),
            top,
            plane = Facing::Side,
            smooth = true
        );
    }
}

pub fn bar_segment() -> Vec<Obj> {
    let f = &GameFrame::new(0.40);
    let still = new_parts(|| {
        bar(f, 1.0, 0.0, false, None, None, None);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

pub fn bar_end() -> Vec<Obj> {
    let f = &GameFrame::new(0.40);
    let still = new_parts(|| {
        bar(f, 0.92, -0.04, true, None, None, None);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}
