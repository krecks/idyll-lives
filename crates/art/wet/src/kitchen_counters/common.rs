//! What the counters are built from: the frame without its depths, the worktop, shaker fronts with swept pulls, the
//! drawer with its cutlery tray, the two carcasses.

use super::M;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, cos, radians, sin};
use artkit::parts::{join, new_parts};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::{fbox, fextrude, sweep};

/// The frame `f` draws in, without its `depths`: for parts placed at drawn depths (`GameFrame::depth`), so a pull or a
/// door keeps its real thickness on a counter drawn shallower (COUNTER66).
pub fn plain(f: &GameFrame) -> GameFrame {
    GameFrame::new(f.shift)
}

/// Points on a circular arc round (cx, cy) from a0 to a1 degrees, n steps (n + 1 points).
pub fn arc(cx: f64, cy: f64, r: f64, a0: i32, a1: i32, n: i32) -> Vec<(f64, f64)> {
    (0..=n)
        .map(|i| {
            (
                cx + r * cos(radians(a0 as f64 + ((a1 - a0) * i) as f64 / n as f64)),
                cy + r * sin(radians(a0 as f64 + ((a1 - a0) * i) as f64 / n as f64)),
            )
        })
        .collect()
}

/// A bar pull in one piece on a front whose face is at drawn depth `z`: round legs out from the face that bend into
/// the bar, its front `reach` off the face (no farther than the worktop's nose). `base`: how far behind the face the
/// legs start (a pull over a recessed panel).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn pull(
    p: &GameFrame,
    name: impl AsRef<str>,
    x: impl Coord,
    y: f64,
    z: f64,
    length: f64,
    material: Option<Mat>,
    vertical: bool,
    reach: f64,
    r: f64,
    base: f64,
) -> Obj {
    let (out, h, k) = (reach - r, length / 2.0, py::min(0.012, reach * 0.5));
    let pt = |a: f64, d: f64| -> P3 {
        if vertical {
            p.at(x, y + a, z + d)
        } else {
            p.at(x.f() + a, y, z + d)
        }
    };
    let pts = [
        pt(-h, -0.002 - base),
        pt(-h, out - k),
        pt(-h + k, out),
        pt(h - k, out),
        pt(h, out - k),
        pt(h, -0.002 - base),
    ];
    sweep!(
        name,
        &pts,
        r,
        material = material.unwrap_or(M.chrome),
        verts = 8,
        resolution = 2
    )
}

/// A shaker door or drawer front, its back at drawn depth `zb`: a frame `rail` wide round a flat panel set 8 mm in,
/// its edges eased; `mid`, a rail across at that height too (a tall door's two panels).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn shaker(
    p: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    x: impl Coord,
    y: f64,
    zb: f64,
    material: Mat,
    rail: f64,
    t: f64,
    mid: Option<f64>,
) {
    let mut holes: Vec<Vec<(f64, f64)>> = Vec::new();
    match mid {
        None => holes.push(rounded_rect(w - 2.0 * rail, h - 2.0 * rail, 0.002, 1)),
        Some(mid) => {
            let m = mid - y; // the middle rail's centre, from the door's
            for (lo, hi) in [
                (-h / 2.0 + rail, m - rail / 2.0),
                (m + rail / 2.0, h / 2.0 - rail),
            ] {
                holes.push(
                    rounded_rect(w - 2.0 * rail, hi - lo, 0.002, 1)
                        .into_iter()
                        .map(|(u, v)| (u, v + (lo + hi) / 2.0))
                        .collect(),
                );
            }
        }
    }
    fextrude!(
        p,
        name,
        &rounded_rect(w, h, 0.004, 1),
        t,
        (x, y, zb + t / 2.0),
        material,
        holes = holes.clone(),
        bevel = 0.003,
        bevel_segments = 1
    );
    // each panel: a margin under the frame's edge, and the field inside it standing a little proud with its edges
    // eased, its corners clear of the frame (the baked shading darkens a face by its corners)
    let tp = t - 0.008;
    for (i, hole) in holes.iter().enumerate() {
        let us: Vec<f64> = hole.iter().map(|&(u, _)| u).collect();
        let vs: Vec<f64> = hole.iter().map(|&(_, v)| v).collect();
        let (pw, ph, pv) = (
            py::max_of(&us) - py::min_of(&us),
            py::max_of(&vs) - py::min_of(&vs),
            (py::max_of(&vs) + py::min_of(&vs)) / 2.0,
        );
        let inner = rounded_rect(pw - 0.02, ph - 0.02, 0.002, 1);
        fextrude!(
            p,
            format!("{name}_panel{i}"),
            &rounded_rect(pw + 0.006, ph + 0.006, 0.002, 1),
            tp,
            (x, y + pv, zb + tp / 2.0),
            material,
            holes = vec![inner]
        );
        fbox!(
            p,
            format!("{name}_field{i}"),
            pw - 0.02,
            ph - 0.02,
            tp + 0.003,
            (x, y + pv, zb + (tp + 0.003) / 2.0),
            material,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
}

/// A worktop from game depth `zb` (its back, on the wall) to `zf` (its front), drawn through f's depths, y0 to y1
/// high: the front edge rolled over (postformed), its underside eased; the ends cut square, so the tops of a run
/// meet without a seam.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn worktop(
    f: &GameFrame,
    name: &str,
    w: f64,
    x: impl Coord,
    zb: f64,
    zf: f64,
    y0: f64,
    y1: f64,
    material: Mat,
    r: f64,
) -> Obj {
    let (zb, zf) = (f.depth(zb), f.depth(zf));
    let mut prof = vec![(zb, y0), (zf - 0.004, y0), (zf, y0 + 0.004)];
    prof.extend(arc(zf - r, y1 - r, r, 0, 90, 4));
    prof.push((zb, y1));
    fextrude!(
        plain(f),
        name,
        &prof,
        w,
        (x, 0, 0),
        material,
        plane = Facing::Side,
        smooth = true
    )
}

/// Two shaker doors on the carcass front at drawn depth `front`, a 4 mm gap between them, their pulls upright on the
/// meeting stiles near the top.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn doors(
    p: &GameFrame,
    w: f64,
    front: f64,
    y0: f64,
    y1: f64,
    body: Option<Mat>,
    hm: Option<Mat>,
    hy: Option<f64>,
    x: f64,
    reach: f64,
) {
    let (body, hm) = (body.unwrap_or(M.cab), hm.unwrap_or(M.chrome));
    let dw = (w - 0.004) / 2.0;
    for s in [-1, 1] {
        let cx = x + s as f64 * (dw + 0.004) / 2.0;
        shaker(
            p,
            &format!("door{s}"),
            dw,
            y1 - y0,
            cx,
            (y0 + y1) / 2.0,
            front,
            body,
            0.06,
            0.02,
            None,
        );
        pull(
            p,
            format!("door_pull{s}"),
            x + s as f64 * 0.034,
            hy.unwrap_or(y1 - 0.10),
            front + 0.02,
            0.14,
            Some(hm),
            true,
            reach,
            0.005,
            0.0,
        );
    }
}

/// A flat outline (u, v).
type Outline = Vec<(f64, f64)>;

/// Cutlery as flat outlines (game x across, depth along; the handle at +depth, towards whoever opens the drawer).
pub fn cutlery_outline(kind: &str, s: f64) -> Vec<(f64, f64)> {
    let (right, tip): (Outline, Outline) = if kind == "spoon" {
        let mut right = vec![
            (0.006, 0.085),
            (0.0068, 0.072),
            (0.0045, 0.01),
            (0.0028, -0.026),
        ];
        right.extend([30, 60, 90, 120, 150].map(|a| {
            (
                0.017 * sin(radians(a as f64)),
                -0.057 + 0.027 * cos(radians(a as f64)),
            )
        }));
        (right, vec![(0.0, -0.085)])
    } else if kind == "fork" {
        (
            vec![
                (0.006, 0.085),
                (0.0068, 0.072),
                (0.0045, 0.01),
                (0.003, -0.02),
                (0.0105, -0.042),
                (0.011, -0.085),
            ],
            vec![
                (0.0075, -0.085),
                (0.0075, -0.058),
                (0.0048, -0.058),
                (0.0048, -0.085),
                (0.0014, -0.085),
                (0.0014, -0.058),
                (-0.0014, -0.058),
                (-0.0014, -0.085),
                (-0.0048, -0.085),
                (-0.0048, -0.058),
                (-0.0075, -0.058),
                (-0.0075, -0.085),
            ],
        )
    } else {
        // a knife: a round-ended blade, its edge on the right
        let right = [
            (0.006, 0.085),
            (0.0065, 0.07),
            (0.006, 0.005),
            (0.004, -0.002),
            (0.009, -0.012),
            (0.0085, -0.06),
            (0.005, -0.08),
        ];
        let tip = [(0.0, -0.087)];
        let left = [
            (-0.004, -0.08),
            (-0.004, -0.002),
            (-0.006, 0.005),
            (-0.0065, 0.07),
            (-0.006, 0.085),
        ];
        return std::iter::once((0.0, 0.088))
            .chain(right)
            .chain(tip)
            .chain(left)
            .map(|(u, v)| (u * s, v * s))
            .collect();
    };
    let left: Vec<(f64, f64)> = right.iter().rev().map(|&(u, v)| (-u, v)).collect();
    std::iter::once((0.0, 0.088))
        .chain(right)
        .chain(tip)
        .chain(left)
        .map(|(u, v)| (u * s, v * s))
        .collect()
}

/// A flat piece (cutlery, a spatula) lying at drawn (x, y, z), `t` thick, its outline in (x, depth).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn flat(
    p: &GameFrame,
    name: impl AsRef<str>,
    outline: &[(f64, f64)],
    x: f64,
    y: f64,
    z: f64,
    material: Mat,
    t: f64,
) -> Obj {
    fextrude!(
        p,
        name,
        outline,
        t,
        (x, y + t / 2.0, z),
        material,
        plane = Facing::Top
    )
}

/// A cutlery tray standing at height `y` with its front at drawn depth `z`: a pale wooden tray of four compartments
/// across, spoons, forks and knives in three, a ladle-length serving spoon in the fourth.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn cutlery(p: &GameFrame, name: &str, x: f64, y: f64, z: f64, width: f64, depth: f64) {
    let zc = z - depth / 2.0;
    fbox!(
        p,
        format!("{name}_floor"),
        width,
        0.006,
        depth,
        (x, y + 0.003, zc),
        M.spoon
    );
    for i in 0..5 {
        fbox!(
            p,
            format!("{name}_wall{i}"),
            0.006,
            0.045,
            depth,
            (
                x - width / 2.0 + 0.003 + (width - 0.006) * i as f64 / 4.0,
                y + 0.0225,
                zc
            ),
            M.spoon
        );
    }
    for s in [-1, 1] {
        fbox!(
            p,
            format!("{name}_end{s}"),
            width,
            0.045,
            0.006,
            (x, y + 0.0225, zc + s as f64 * (depth / 2.0 - 0.003)),
            M.spoon
        );
    }
    let cw = (width - 0.006) / 4.0;
    for (i, kind) in ["spoon", "fork", "knife"].into_iter().enumerate() {
        let cx = x - width / 2.0 + 0.003 + cw * (i as f64 + 0.5);
        for j in 0..2 {
            flat(
                p,
                format!("{name}_{kind}{j}"),
                &cutlery_outline(kind, 1.0),
                cx + (j as f64 - 0.5) * 0.012,
                y + 0.006 + j as f64 * 0.0028,
                zc + 0.01 - 0.02 * j as f64,
                M.steel,
                0.0025,
            );
        }
    }
    flat(
        p,
        format!("{name}_server"),
        &cutlery_outline("spoon", 1.55),
        x + width / 2.0 - cw / 2.0,
        y + 0.006,
        zc,
        M.steel,
        0.0025,
    );
}

/// A shaker drawer front, its face at drawn depth `zf`, with a bar pull and the box behind it (hidden inside the
/// carcass at rest, a cutlery tray in it), as its own object, for `drawer_open`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn drawer(
    p: &GameFrame,
    name: &str,
    w: f64,
    y: f64,
    zf: f64,
    material: Mat,
    hm: Option<Mat>,
    handle_w: f64,
    contents: bool,
    reach: f64,
    rail: f64,
    h: f64,
) -> Obj {
    let zb = zf - 0.02;
    let parts = new_parts(|| {
        shaker(
            p,
            &format!("{name}_front"),
            w,
            h,
            0,
            y,
            zb,
            material,
            rail,
            0.02,
            None,
        );
        pull(
            p,
            format!("{name}_pull"),
            0,
            y,
            zf,
            handle_w,
            hm,
            false,
            reach,
            0.005,
            0.008,
        );
        let f = p;
        fbox!(
            f,
            format!("{name}_bottom"),
            w - 0.06,
            0.012,
            0.5,
            (0, y - 0.074, zb - 0.25),
            material
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("{name}_side{s}"),
                0.014,
                0.14,
                0.5,
                (s as f64 * (w / 2.0 - 0.037), y - 0.01, zb - 0.25),
                material,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            format!("{name}_back"),
            w - 0.06,
            0.14,
            0.014,
            (0, y - 0.01, zb - 0.493),
            material,
            bevel = 0.003,
            bevel_segments = 1
        );
        if contents {
            cutlery(
                p,
                &format!("{name}_tray"),
                -w / 2.0 + 0.26,
                y - 0.068,
                zb - 0.02,
                0.4,
                0.34,
            );
        }
    });
    join(&parts, name, Some(p.at(0, y, zf - 0.01)))
}

/// A base unit as the catalogue's `k_base`: worktop to 0.95, front face at depth 0.14.
pub fn unit66(f: &GameFrame, w: f64, body: Option<Mat>, top: Option<Mat>) {
    let (body, top) = (body.unwrap_or(M.cab), top.unwrap_or(M.worktop));
    fbox!(
        f,
        "carcass",
        w - 0.004,
        0.82,
        0.6,
        (0, 0.51, -0.16),
        body,
        bevel = 0.01
    );
    fbox!(
        f,
        "plinth",
        w - 0.04,
        0.10,
        0.56,
        (0, 0.05, -0.18),
        M.dark,
        bevel = 0.004
    );
    worktop(f, "worktop", w, 0, -0.48, 0.18, 0.91, 0.95, top, 0.012);
}

/// A base unit as catalog2's `counter_body`: worktop to 0.95, front face at depth 0.24 (`worktop` false: none, for
/// a unit under a worktop laid over more than it).
pub fn unit79(f: &GameFrame, w: f64, x: f64, body: Option<Mat>, top: Option<Mat>, with_top: bool) {
    let (body, top) = (body.unwrap_or(M.cab), top.unwrap_or(M.worktop));
    fbox!(
        f,
        "carcass",
        w - 0.004,
        0.8,
        0.7,
        (x, 0.5, -0.11),
        body,
        bevel = 0.01
    );
    fbox!(
        f,
        "plinth",
        w - 0.004,
        0.10,
        0.64,
        (x, 0.05, -0.14),
        M.dark,
        bevel = 0.004
    );
    if with_top {
        worktop(f, "worktop", w, x, -0.48, 0.31, 0.90, 0.95, top, 0.012);
    }
}
