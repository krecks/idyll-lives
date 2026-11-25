//! H1: the ukulele and the guitars. Each leans on a black A-frame stand: built upright, its foot at the origin and its
//! face to the front, then leant back onto the stand, a part of its own.

use super::M;
use super::common::{P, frets, guitar_stand, lean, off_stand, strings};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr as r;
use artkit::materials::Mat;
use artkit::math::{P3, PI, asin, cos, cosd, degrees, pow, sin, sind, sqrt};
use artkit::parts::{Settle, join, new_parts, settle_with};
use artkit::place::GameFrame;
use artkit::py::{hypot, sum};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, torus};

/// An instrument's body outline (game x, height), its lower and upper bouts the ellipses `lower` and `upper` (middle
/// height, half-width, half-height) meeting at a rounded waist; from the foot round.
pub fn figure8(lower: P, upper: P, n: usize, passes: usize) -> Vec<(f64, f64)> {
    let ((c1, a1, b1), (c2, a2, b2)) = (lower, upper);
    let half = |c: f64, a: f64, b: f64, y: f64| {
        let t = (y - c) / b;
        let k = 1.0 - t * t;
        a * sqrt(if k > 0.0 { k } else { 0.0 })
    };
    let (mut lo, mut hi) = (c1, c2);
    // the waist: where the bouts are as wide as each other
    for _ in 0..40 {
        let m = (lo + hi) / 2.0;
        (lo, hi) = if half(c1, a1, b1, m) > half(c2, a2, b2, m) {
            (m, hi)
        } else {
            (lo, m)
        };
    }
    let yw = (lo + hi) / 2.0;
    let clamp = |x: f64| {
        let x = if 1.0 < x { 1.0 } else { x };
        if x > -1.0 { x } else { -1.0 }
    };
    let t1 = degrees(asin(clamp((yw - c1) / b1)));
    let t2 = degrees(asin(clamp((yw - c2) / b2)));
    let mut right: Vec<(f64, f64)> = (0..=n)
        .map(|i| -90.0 + (t1 + 90.0) * i as f64 / n as f64)
        .map(|t| (a1 * cosd(t), c1 + b1 * sind(t)))
        .collect();
    right.extend(
        (0..=n)
            .map(|i| t2 + (90.0 - t2) * i as f64 / n as f64)
            .map(|t| (a2 * cosd(t), c2 + b2 * sind(t))),
    );
    let left: Vec<(f64, f64)> = right[1..right.len() - 1]
        .iter()
        .rev()
        .map(|&(x, y)| (-x, y))
        .collect();
    right.extend(left);
    smooth_outline(&right, passes, true)
}

/// A closed outline moved `d` in all round (out, for a negative `d`).
pub fn inset(outline: &[(f64, f64)], d: f64) -> Vec<(f64, f64)> {
    let n = outline.len();
    let area = sum((0..n)
        .map(|i| outline[i].0 * outline[(i + 1) % n].1 - outline[(i + 1) % n].0 * outline[i].1));
    let turn = if area > 0.0 { 1.0 } else { -1.0 };
    (0..n)
        .map(|i| {
            let ((x0, y0), (x1, y1)) = (outline[(i + n - 1) % n], outline[(i + 1) % n]);
            let (dx, dy) = (x1 - x0, y1 - y0);
            let length = hypot(dx, dy);
            let length = if length == 0.0 { 1.0 } else { length };
            (
                outline[i].0 - dy / length * d * turn,
                outline[i].1 + dx / length * d * turn,
            )
        })
        .collect()
}

/// A guitar's neck from height `y0` up to `y1`: flat under the fretboard at depth `z`, rounded behind (a D), `w`
/// across and `d` deep, narrowing by `taper` to the nut.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn neck(
    fr: &GameFrame,
    name: &str,
    y0: f64,
    y1: f64,
    z: f64,
    w: f64,
    d: f64,
    material: Mat,
    taper: f64,
    verts: usize,
) -> Obj {
    let prof: Vec<(f64, f64)> = (0..=verts)
        .map(|i| {
            (
                w / 2.0 * cos(PI * i as f64 / verts as f64),
                d * sin(PI * i as f64 / verts as f64),
            )
        })
        .collect();
    fsweep!(
        fr,
        name,
        &[(0, y0, z), (0, y1, z)],
        0,
        material,
        profile = prof,
        up = (1, 0, 0),
        scale = [1.0, taper],
        angle = 70
    )
}

/// Position dots on a fretboard between the frets `frets` (its face at depth `z`).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn dots(
    fr: &GameFrame,
    name: &str,
    frets: &[i32],
    scale: f64,
    nut: f64,
    z: f64,
    material: Mat,
    r: f64,
) {
    for &k in frets {
        let y = nut - scale * (1.0 - pow(2.0, -(k as f64 - 0.5) / 12.0));
        fcyl!(
            fr,
            format!("{name}{k}"),
            r,
            0.001,
            (0, y, z),
            material,
            rot = (90, 0, 0),
            verts = 6,
            smooth = false
        );
    }
}

fn ukulele_part(fr: &GameFrame) -> Obj {
    let p = new_parts(|| {
        let body = figure8((0.1, 0.115, 0.1), (0.25, 0.09, 0.08), 6, 1);
        fextrude!(
            fr,
            "body",
            &body,
            0.065,
            (0, 0, 0),
            M.uke,
            bevel = 0.009,
            bevel_segments = 2
        );
        fextrude!(
            fr,
            "top",
            &inset(&body, 0.006),
            0.004,
            (0, 0, 0.033),
            M.uke_top
        );
        fcyl!(
            fr,
            "hole",
            0.028,
            0.004,
            (0, 0.205, 0.0345),
            M.hole,
            rot = (90, 0, 0),
            verts = 18,
            smooth = false
        );
        torus!(
            "rosette",
            0.032,
            0.0025,
            at = fr.at(0, 0.205, 0.035),
            material = M.fretboard,
            rot = (90, 0, 0),
            major_segments = 24,
            minor_segments = 4
        );
        fextrude!(
            fr,
            "bridge",
            &rounded_rect(0.07, 0.012, 0.005, 2),
            0.008,
            (0, 0.07, 0.039),
            M.fretboard,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(fr, "saddle", 0.04, 0.003, 0.004, (0, 0.072, 0.044), M.bone);
        neck(fr, "neck", 0.315, 0.565, 0.015, 0.04, 0.02, M.uke, 0.9, 8);
        fbox!(
            fr,
            "fretboard",
            0.038,
            0.23,
            0.005,
            (0, 0.445, 0.0175),
            M.fretboard
        );
        frets(fr, 12, 0.35, 0.56, 0.038, 0.0205);
        dots(fr, "dot", &[5, 7, 10], 0.35, 0.56, 0.0205, M.bone, 0.0025);
        fbox!(fr, "nut", 0.038, 0.004, 0.004, (0, 0.561, 0.021), M.bone);
        // the head: a crown at its top
        let head = [
            (0.019, 0.558),
            (0.03, 0.575),
            (0.031, 0.632),
            (0.02, 0.65),
            (0.0, 0.642),
            (-0.02, 0.65),
            (-0.031, 0.632),
            (-0.03, 0.575),
            (-0.019, 0.558),
        ];
        fextrude!(
            fr,
            "head",
            &smooth_outline(&head, 1, true),
            0.016,
            (0, 0, 0.002),
            M.fretboard,
            bevel = 0.003,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            for y in [0.588, 0.622] {
                fcyl!(
                    fr,
                    format!("peg{s}{}", r(y)),
                    0.004,
                    0.022,
                    (s as f64 * 0.036, y, 0.0),
                    M.fret,
                    rot = (0, 90, 0),
                    verts = 8
                );
                fbox!(
                    fr,
                    format!("peg_button{s}{}", r(y)),
                    0.004,
                    0.016,
                    0.01,
                    (s as f64 * 0.049, y, 0.0),
                    M.bone
                );
            }
        }
        strings(fr, 4, 0.0075, 0.0085, 0.072, 0.561, 0.0465, 0.0235);
    });
    join(&p, "ukulele", Some(P3::ZERO))
}

pub fn ukulele() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| guitar_stand(&f, 0.12, 0.2, 0.62, -0.005, 0.07, 0.17, 0.06));
    join(&still, "stand", Some(P3::ZERO));
    let uke = lean(
        &f,
        ukulele_part(&GameFrame::new(0.0)),
        15.0,
        (0.0, 0.215, 0.135),
    );
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    off_stand(uke);
    meshes()
}
