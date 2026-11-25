//! L1, the sofas and sectionals. Each is built where the game places it, its back on the wall's face, and settled
//! in place. Two nodes: the sofa (upholstery, frame, back cushions, pillows, legs) and its seat cushions (`seat`,
//! origin on their underside), which `sink_in` squashes 2 cm while it's sat or dozed on.

use super::common::{
    Cushion, cabriole, cushion, leg, listed, panel, pillow, plain, seat, settled, swept, taper_leg,
};
use super::{F, M};
use crate::common::eased_prism;
use artkit::forms::{catmull, rounded_rect};
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, V3, cos, exp, pow, radians, sin};
use artkit::parts::{join, new_parts};
use artkit::py;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, soft};

/// A sofa or a sectional's module `w` wide (the old `sofa_run`): a base, arms on the `arms` sides (-1 at -x), back
/// rests along the back and/or a side ('left' at -x, 'right' at +x), seat and back cushions, pillows (x, depth,
/// material, turn) and tapered legs. Returns the seat.
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    up: Mat,
    cush: Mat,
    legs: Mat,
    w: f64,
    arms: &[i32],
    backs: &[&str],
    depth: f64,
    z: f64,
    n: Option<usize>,
    pillows: &[(f64, f64, Mat, i32)],
) -> (Obj, f64) {
    let (z0, z1) = (z - depth / 2.0, z + depth / 2.0);
    let (mut sx0, mut sx1, mut sz0) = (-w / 2.0, w / 2.0, z0);
    let sides: Vec<i32> = backs
        .iter()
        .filter(|b| **b == "left" || **b == "right")
        .map(|b| if *b == "left" { -1 } else { 1 })
        .collect();
    let has_back = backs.contains(&"back");
    if has_back {
        sz0 = z0 + 0.2;
    }
    for &s in &sides {
        (sx0, sx1) = if s < 0 {
            (sx0 + 0.2, sx1)
        } else {
            (sx0, sx1 - 0.2)
        };
    }
    for &s in arms {
        (sx0, sx1) = if s < 0 {
            (sx0 + 0.17, sx1)
        } else {
            (sx0, sx1 - 0.17)
        };
    }
    let n = n.unwrap_or_else(|| py::round((sx1 - sx0) / 0.9).max(1) as usize);
    let cw = (sx1 - sx0) / n as f64;
    let lean = n.min(3); // few cushions: plumper; a long run keeps its triangles down
    let still = new_parts(|| {
        // the base, a welt round its top edge; track arms boxed down to the base's foot, welted round their sides
        panel(
            "base",
            w,
            0.22,
            depth,
            (0.0, 0.21, z),
            up,
            0.035,
            0.0,
            Some(0.005),
            plain(0.035),
        );
        if has_back {
            panel(
                "back",
                w,
                0.5,
                0.2,
                (0.0, 0.57, z0 + 0.1),
                up,
                0.06,
                0.0,
                None,
                plain(0.06),
            );
        }
        for &s in &sides {
            let sf = f64::from(s);
            panel(
                &format!("side_back{s}"),
                0.2,
                0.5,
                depth,
                (sf * (w / 2.0 - 0.1), 0.57, z),
                up,
                0.06,
                0.0,
                None,
                plain(0.06),
            );
            cushion(
                &format!("side_cushion{s}"),
                0.16,
                0.4,
                depth - 0.24,
                (sf * (w / 2.0 - 0.27), 0.62, z + 0.1),
                cush,
                Cushion {
                    rot: P3::from((0, -s * 9, 0)),
                    puff: 0.028,
                    piping: None,
                    flat: Some(2),
                    ..Cushion::default()
                },
            );
        }
        for &s in arms {
            panel(
                &format!("arm{s}"),
                0.17,
                0.52,
                depth,
                (f64::from(s) * (w / 2.0 - 0.085), 0.36, z),
                up,
                0.065,
                0.0,
                Some(0.006),
                plain(0.065),
            );
        }
        if has_back {
            for i in 0..n {
                cushion(
                    &format!("back_cushion{i}"),
                    cw - 0.03,
                    0.4,
                    0.16,
                    (sx0 + cw * (i as f64 + 0.5), 0.62, sz0 + 0.07),
                    cush,
                    Cushion {
                        rot: P3::from((-9, 0, 0)),
                        puff: 0.03,
                        piping: if lean < 3 { Some(0.006) } else { None },
                        flat: Some(if lean < 3 { 3 } else { 2 }),
                        ..Cushion::default()
                    },
                );
            }
        }
        for (i, &(px, pz, mat, turn)) in pillows.iter().enumerate() {
            pillow(&format!("pillow{i}"), px, 0.62, pz, mat, turn, 0.4, 0.36);
        }
        for lx in [-w / 2.0 + 0.06, w / 2.0 - 0.06] {
            for lz in [z0 + 0.06, z1 - 0.06] {
                taper_leg(
                    &format!("leg{}{}", float_repr(lx), float_repr(lz)),
                    lx,
                    lz,
                    0.1,
                    legs,
                    0.022,
                    None,
                );
            }
        }
    });
    join(&still, "sofa", Some((0, 0, 0).into()));
    let cushions = || {
        for i in 0..n {
            cushion(
                &format!("seat{i}"),
                cw - 0.02,
                0.13,
                z1 - sz0 - 0.02,
                (sx0 + cw * (i as f64 + 0.5), 0.385, (sz0 + z1) / 2.0 + 0.005),
                cush,
                Cushion {
                    keep_top: true,
                    ..Cushion::default()
                },
            );
        }
        0.45
    };
    seat(cushions, 0.32, (sz0 + z1) / 2.0)
}

/// The classic sofa (the old `sofa_classic`): a camel back in channels under a walnut rail, scroll arms with carved
/// faces, an apron rail with a shell, cabriole legs; two seat cushions.
pub fn sofa_classic() -> Vec<Obj> {
    let still = new_parts(|| {
        fbox!(
            F,
            "deck",
            1.7,
            0.18,
            0.775,
            (0, 0.27, -0.048),
            M.linen,
            bevel = 0.05,
            bevel_segments = 3
        );
        // the apron: a serpentine lower edge dropping to a lobe in the middle, a carved shell on it
        let mut outline = vec![(0.86, 0.195), (-0.86, 0.195)];
        outline.extend((0..29).map(|i| {
            let x = -0.86 + 1.72 * f64::from(i) / 28.0;
            (
                x,
                0.152 - 0.009 * cos(TAU * x / 0.43) - 0.03 * exp(-pow(x / 0.07, 2.0)),
            )
        }));
        fextrude!(
            F,
            "apron",
            &outline,
            0.035,
            (0, 0, 0.33),
            M.walnut,
            bevel = 0.005,
            bevel_segments = 1
        );
        let mut fan = vec![(0.0, 0.0)];
        fan.extend((0..11).map(|i| {
            let k = 0.05 * (1.0 - 0.12 * f64::from(i % 2));
            let a = PI * f64::from(i) / 10.0;
            (k * cos(a), k * sin(a))
        }));
        fextrude!(
            F,
            "shell",
            &fan,
            0.018,
            (0, 0.14, 0.354),
            M.walnut,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "back_board",
            1.7,
            0.42,
            0.1,
            (0, 0.53, -0.385),
            M.linen,
            bevel = 0.04,
            bevel_segments = 3
        );
        // the camel back: seven stuffed channels rising from the deck and rolling back under the rail
        for i in 0..7 {
            let u = (f64::from(i) + 0.5) / 7.0;
            let (x, t) = (-0.84 + 1.68 * u, hump(u));
            channel(
                &format!("channel{i}"),
                [
                    (x, 0.33, -0.27),
                    (x, (0.33 + t) / 2.0, -0.3),
                    (x, t - 0.09, -0.32),
                    (x, t - 0.035, -0.37),
                ],
                (0.236, 0.15, 0.065, 2),
                &[1.0, 1.03, 1.0, 0.85],
                M.linen,
            );
        }
        // the show-wood rail along the hump: one moulded piece
        let rail: Vec<(f64, f64, f64)> = (0..15)
            .map(|i| {
                let k = f64::from(i);
                (-0.84 + 1.68 * k / 14.0, hump(k / 14.0) + 0.015, -0.375)
            })
            .collect();
        fsweep!(
            F,
            "rail",
            &rail,
            0.0,
            M.walnut,
            profile = rounded_rect(0.046, 0.038, 0.013, 1),
            up = (0, 1, 0),
            resolution = 2,
            angle = 60
        );
        for s in [-1, 1] {
            let sf = f64::from(s);
            let ax = sf * 0.84;
            // the rolled arm: its cross-section (a panel rising into a roll that leans out) swept front to back
            let mut section = vec![(0.24, sf * -0.085), (0.24, sf * 0.07)];
            section.extend((-40..230).step_by(18).map(|a| {
                let a = radians(f64::from(a));
                (0.6 + 0.1 * sin(a), sf * (0.1 * cos(a)))
            }));
            section.push((0.5, sf * -0.085)); // (height, x out)
            eased_prism(
                &F,
                &format!("arm{s}"),
                &section,
                ax,
                -0.39,
                0.32,
                0.004,
                M.linen,
            );
            fcyl!(
                F,
                format!("scroll_face{s}"),
                0.085,
                0.03,
                (ax, 0.6, 0.335),
                M.walnut,
                rot = (90, 0, 0),
                verts = 20,
                bevel = 0.008
            );
            fcyl!(
                F,
                format!("scroll_boss{s}"),
                0.03,
                0.012,
                (ax, 0.6, 0.354),
                M.walnut,
                rot = (90, 0, 0),
                verts = 12,
                bevel = 0.004
            );
            // a shaped post from the apron up to the scroll face, waisted in the middle
            fsweep!(
                F,
                format!("arm_post{s}"),
                &[
                    (ax, 0.2, 0.338),
                    (ax, 0.33, 0.346),
                    (ax, 0.44, 0.34),
                    (ax, 0.52, 0.337)
                ],
                0.024,
                M.walnut,
                verts = 8,
                resolution = 2,
                scale = [1.1, 0.75, 0.95, 1.0]
            );
        }
        soft!(
            F,
            "pillow_l",
            0.36,
            0.34,
            0.12,
            (-0.6, 0.6, -0.215),
            M.pillow_rust,
            rot = (-13, 0, 18),
            round = 0.05,
            puff = 0.035,
            flat = 2
        );
        soft!(
            F,
            "pillow_r",
            0.34,
            0.32,
            0.12,
            (0.6, 0.6, -0.215),
            M.pillow_sage,
            rot = (-13, 0, -18),
            round = 0.05,
            puff = 0.035,
            flat = 2
        );
        for x in [-0.82, 0.82] {
            for (z, dz) in [(0.33, 1.0), (-0.38, -1.0)] {
                let sx = if x > 0.0 { 1.0 } else { -1.0 };
                cabriole(
                    &format!("leg{}{}", float_repr(x), float_repr(z)),
                    x,
                    z,
                    0.18,
                    M.walnut_legs,
                    0.025 * sx,
                    0.02 * dz,
                    0.024,
                );
            }
        }
    });
    join(&still, "sofa", Some((0, 0, 0).into()));
    let cushions = || {
        for s in [-1, 1] {
            soft!(
                F,
                format!("seat{s}"),
                0.78,
                0.1,
                0.595,
                (f64::from(s) * 0.4, 0.39, 0.033),
                M.linen_cushion,
                round = 0.035,
                puff = 0.016,
                piping = 0.006
            );
        }
        0.44
    };
    settled(seat(cushions, 0.34, 0.033))
}

/// A stuffed channel of a back: a rounded rectangle (w across, d thick, corners r in `segs` steps) swept up from the deck through
/// `given` (game) and rolled back (no tighter than its thickness: inside, it would fold), thickening by `list`
/// along it, its end chamfered 4 mm.
pub(super) fn channel(
    name: &str,
    given: [(f64, f64, f64); 4],
    (w, d, r, segs): (f64, f64, f64, usize),
    list: &[f64],
    mat: Mat,
) -> Obj {
    let (mut pts, wher) = catmull(&given.map(|(x, y, z)| V3::from(F.at(x, y, z))), 2, false);
    let mut scales: Vec<f64> = wher.iter().map(|&w| listed(list, w)).collect();
    // the last ring 4 mm on, shrunk 4 mm on its thinner side
    let (end, k) = (pts[pts.len() - 1], scales[scales.len() - 1]);
    pts.push(end + (end - pts[pts.len() - 2]).normalized() * 0.004);
    scales.push(k * (1.0 - 0.004 / (d / 2.0 * k)));
    let profile = rounded_rect(w, d, r, segs);
    // stuffed, it shades round over its corners' steps
    swept(
        name,
        &pts,
        &profile,
        &scales,
        V3::from((1, 0, 0)),
        60.0,
        mat,
    )
}

/// The camel back's top at `u` across (0 to 1).
pub(super) fn hump(u: f64) -> f64 {
    0.74 + 0.16 * exp(-pow((u - 0.5) / 0.22, 2.0))
}

/// The loveseat and the lounge chair (the old `sofa`, box arms): a base on tapered legs, box arms, a back, `n` seat
/// and back cushions, `w` wide and 0.84 deep; pillows (x, material, turn) against the back.
pub fn box_sofa(
    up: Mat,
    cush: Mat,
    legs: Mat,
    w: f64,
    pillows: &[(f64, Mat, i32)],
    n: usize,
) -> (Obj, f64) {
    let (z, depth, back_h) = (-0.04, 0.84, 0.82);
    let (z0, z1) = (z - depth / 2.0, z + depth / 2.0);
    let (aw, bt) = (0.17, 0.2);
    let (sx0, sx1, sz0) = (-w / 2.0 + aw, w / 2.0 - aw, z0 + bt);
    let cw = (sx1 - sx0) / n as f64;
    let still = new_parts(|| {
        // a welted base between track arms, a firm back behind plump back cushions, splayed tapered legs
        panel(
            "base",
            w - 2.0 * aw + 0.04,
            0.22,
            depth - 0.01,
            (0.0, 0.21, z),
            up,
            0.03,
            0.0,
            Some(0.005),
            plain(0.03),
        );
        panel(
            "back",
            w - 0.01,
            back_h - 0.38,
            bt,
            (0.0, (back_h - 0.08 + 0.3) / 2.0, z0 + bt / 2.0),
            up,
            0.06,
            0.0,
            None,
            plain(0.06),
        );
        for s in [-1, 1] {
            panel(
                &format!("arm{s}"),
                aw,
                0.52,
                depth,
                (f64::from(s) * (w / 2.0 - aw / 2.0), 0.36, z),
                up,
                0.065,
                0.0,
                Some(0.006),
                plain(0.065),
            );
        }
        for i in 0..n {
            cushion(
                &format!("back_cushion{i}"),
                cw - 0.025,
                back_h - 0.44,
                0.16,
                (
                    sx0 + cw * (i as f64 + 0.5),
                    (back_h + 0.44) / 2.0 + 0.01,
                    sz0 + 0.07,
                ),
                cush,
                Cushion {
                    rot: P3::from((-9, 0, 0)),
                    puff: 0.03,
                    ..Cushion::default()
                },
            );
        }
        for (i, &(px, mat, turn)) in pillows.iter().enumerate() {
            pillow(
                &format!("pillow{i}"),
                px,
                0.64,
                sz0 + 0.14,
                mat,
                turn,
                0.4,
                0.36,
            );
        }
        for lx in [-w / 2.0 + 0.07, w / 2.0 - 0.07] {
            for lz in [z0 + 0.07, z1 - 0.07] {
                taper_leg(
                    &format!("leg{}{}", float_repr(lx), float_repr(lz)),
                    lx,
                    lz,
                    0.1,
                    legs,
                    0.02,
                    Some((0.012_f64.copysign(lx), 0.012_f64.copysign(lz))),
                );
            }
        }
    });
    join(&still, "sofa", Some((0, 0, 0).into()));
    let cushions = || {
        for i in 0..n {
            cushion(
                &format!("seat{i}"),
                cw - 0.015,
                0.13,
                z1 - sz0 - 0.02,
                (sx0 + cw * (i as f64 + 0.5), 0.385, (sz0 + z1) / 2.0 + 0.005),
                cush,
                Cushion {
                    keep_top: true,
                    ..Cushion::default()
                },
            );
        }
        0.45
    };
    seat(cushions, 0.32, (sz0 + z1) / 2.0)
}

pub fn loveseat() -> Vec<Obj> {
    settled(box_sofa(
        M.sage,
        M.sage_cushion,
        M.oak_legs,
        1.84,
        &[(-0.55, M.pillow_cream, 10)],
        2,
    ))
}

/// The three-seat sofa (the old `sofa3`): a deep base, three seat and three back cushions, two pillows and a throw
/// folded over the right-hand seat.
pub fn sofa_three() -> Vec<Obj> {
    let still = new_parts(|| {
        panel(
            "base",
            2.66,
            0.2,
            0.86,
            (0.0, 0.22, -0.03),
            M.stone,
            0.04,
            0.0,
            Some(0.005),
            plain(0.04),
        );
        panel(
            "back",
            2.8,
            0.5,
            0.22,
            (0.0, 0.62, -0.35),
            M.stone,
            0.08,
            0.0,
            None,
            plain(0.08),
        );
        for s in [-1, 1] {
            rolled_arm(
                &format!("arm{s}"),
                f64::from(s) * 1.31,
                s,
                -0.46,
                0.39,
                0.12,
                0.64,
                0.08,
                M.stone,
                0.02,
            );
        }
        for (i, x) in [-0.8, 0.0, 0.8].into_iter().enumerate() {
            cushion(
                &format!("back_cushion{i}"),
                0.74,
                0.4,
                0.16,
                (x, 0.62, -0.2),
                M.stone_cushion,
                Cushion {
                    rot: P3::from((-10, 0, 0)),
                    puff: 0.03,
                    ..Cushion::default()
                },
            );
        }
        pillow("pillow_l", -0.98, 0.65, -0.12, M.pillow_sage, 10, 0.4, 0.34);
        pillow("pillow_r", 0.98, 0.65, -0.12, M.pillow_sage, -10, 0.4, 0.34);
        for x in [-1.32, 1.32] {
            for z in [-0.38, 0.32] {
                leg(
                    &format!("leg{}{}", float_repr(x), float_repr(z)),
                    x,
                    z,
                    0.12,
                    M.oak_dark_legs,
                    0.025,
                    (0.0, 0.0),
                    10,
                );
            }
        }
    });
    join(&still, "sofa", Some((0, 0, 0).into()));
    let cushions = || {
        for (i, x) in [-0.8, 0.0, 0.8].into_iter().enumerate() {
            cushion(
                &format!("seat{i}"),
                0.78,
                0.12,
                0.62,
                (x, 0.38, 0.06),
                M.stone_cushion,
                Cushion {
                    keep_top: true,
                    ..Cushion::default()
                },
            );
        }
        kit::throw!(
            F,
            "throw",
            0.86,
            0.6,
            0.458,
            -0.15,
            0.375,
            M.throw_sage,
            0.1,
            turn = 4.0,
            seed = 3.0
        );
        0.44
    };
    settled(seat(cushions, 0.32, 0.06))
}

/// A rolled (English) arm in one piece: its section, a panel rising from `y0` into a roll of radius `r` whose top is
/// `top`, leaning out by `lean`, swept from depth z0 to z1 at x `ax` (side `s`); a welt round its front.
#[allow(clippy::too_many_arguments)]
pub fn rolled_arm(
    name: &str,
    ax: f64,
    s: i32,
    z0: f64,
    z1: f64,
    y0: f64,
    top: f64,
    r: f64,
    mat: Mat,
    lean: f64,
) {
    let s = f64::from(s);
    let (cy, inner) = (top - r, -r * 1.1);
    let mut section = vec![(y0, s * inner), (y0, s * (lean + r * 0.92))];
    section.extend((-40..231).step_by(18).map(|a| {
        let a = radians(f64::from(a));
        (cy + r * sin(a), s * (lean + r * cos(a)))
    }));
    section.push((cy - r * 1.4, s * inner)); // (height, x out)
    eased_prism(&F, name, &section, ax, z0, z1, 0.004, mat);
    // a welt round the scroll of its front face, a little in from its edge, running down its outer side
    let mut pts = vec![(ax + s * (lean + r * 0.92) * 0.97, y0 + 0.03, z1 - 0.002)];
    pts.extend(
        section[2..section.len() - 1]
            .iter()
            .map(|&(y, x)| (ax + x * 0.9, cy + (y - cy) * 0.9, z1 - 0.002)),
    );
    fsweep!(
        F,
        format!("{name}_welt"),
        &pts,
        0.006,
        mat,
        verts = 5,
        resolution = 2
    );
}
