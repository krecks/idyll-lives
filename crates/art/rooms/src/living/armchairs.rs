//! L2, the armchairs: built as the sofas are, their seats sinking in (`sink_in`), but for the designer chair
//! (still).

use super::common::{Cushion, cabriole, cushion, panel, plain, round_rings, seat, settled};
use super::sofas::{box_sofa, rolled_arm};
use super::{F, M};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, cos, exp, pow, radians, sin, sqrt};
use artkit::parts::{join, join_all, new_parts, settle};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, loft, soft};

// (0.318 is a depth, not 1/pi)
#[allow(clippy::approx_constant)]
/// The armchair (the old `armchair_bergere`): an arched back in channels inside a walnut frame, padded arms on open
/// walnut supports, cabriole legs; its seat cushion sinks in.
pub fn armchair_bergere() -> Vec<Obj> {
    let zb = -0.445;
    let top = |u: f64| {
        let k = 1.0 - pow((u - 0.5) / 0.56, 2.0);
        0.68 + 0.26 * sqrt(if k > 0.0 { k } else { 0.0 })
    };
    let still = new_parts(|| {
        fbox!(
            F,
            "deck",
            0.66,
            0.16,
            0.715,
            (0, 0.26, -0.058),
            M.linen,
            bevel = 0.05,
            bevel_segments = 3
        );
        // the apron: a serpentine lower edge dropping to a carved lobe in the middle
        let mut outline = vec![(0.34, 0.195), (-0.34, 0.195)];
        outline.extend((0..17).map(|i| {
            let x = -0.34 + 0.68 * f64::from(i) / 16.0;
            (
                x,
                0.152 - 0.008 * cos(TAU * x / 0.34) - 0.032 * exp(-pow(x / 0.045, 2.0)),
            )
        }));
        fextrude!(
            F,
            "apron",
            &outline,
            0.035,
            (0, 0, 0.3),
            M.walnut,
            bevel = 0.005,
            bevel_segments = 1
        );
        // the back: six stuffed channels, each rising from the deck and rolling back over the arch
        for i in 0..6 {
            let u = (f64::from(i) + 0.5) / 6.0;
            let (x, t) = (-0.3 + 0.6 * u, top(u));
            fsweep!(
                F,
                format!("channel{i}"),
                &[
                    (x, 0.31, -0.29),
                    (x, (0.31 + t) / 2.0, -0.32),
                    (x, t - 0.05, -0.345),
                    (x, t - 0.022, -0.385)
                ],
                0.0,
                M.linen,
                profile = rounded_rect(0.112, 0.11, 0.05, 2),
                up = (1, 0, 0),
                resolution = 2,
                scale = [1.0, 1.04, 1.0, 0.9]
            );
        }
        // the show-wood frame round the back: one moulded rail along the arch
        let mut pts = vec![(-0.34, 0.42, zb + 0.055)];
        pts.extend((0..13).map(|i| {
            let k = f64::from(i);
            (-0.34 + 0.68 * k / 12.0, top(k / 12.0) + 0.025, zb + 0.06)
        }));
        pts.push((0.34, 0.42, zb + 0.055));
        fsweep!(
            F,
            "frame",
            &pts,
            0.0,
            M.walnut,
            profile = rounded_rect(0.04, 0.034, 0.012, 1),
            up = (0, 1, 0),
            angle = 60
        );
        for s in [-1, 1] {
            let ax = f64::from(s) * 0.32;
            soft!(
                F,
                format!("arm_pad{s}"),
                0.1,
                0.06,
                0.605,
                (ax, 0.628, -0.022),
                M.linen,
                round = 0.026,
                puff = 0.012,
                flat = 2,
                round_segments = 1
            );
            // the rail runs forward under the pad and rolls down into a scroll
            fsweep!(
                F,
                format!("arm_rail{s}"),
                &[
                    (ax, 0.585, -0.34),
                    (ax, 0.585, 0.24),
                    (ax, 0.59, 0.315),
                    (ax, 0.565, 0.35),
                    (ax, 0.535, 0.335),
                    (ax, 0.548, 0.312)
                ],
                0.0,
                M.walnut,
                profile = rounded_rect(0.032, 0.058, 0.012, 1),
                resolution = 2,
                scale = [1.0, 1.0, 1.05, 1.0, 0.8, 0.55],
                angle = 60
            );
            // a shaped post from the seat rail up to the scroll, waisted in the middle
            fsweep!(
                F,
                format!("arm_post{s}"),
                &[
                    (ax, 0.29, 0.318),
                    (ax, 0.4, 0.33),
                    (ax, 0.5, 0.322),
                    (ax, 0.56, 0.31)
                ],
                0.022,
                M.walnut,
                verts = 8,
                resolution = 2,
                scale = [1.1, 0.78, 0.9, 1.0]
            );
            fbox!(
                F,
                format!("side{s}"),
                0.05,
                0.25,
                0.545,
                (ax, 0.45, -0.052),
                M.linen,
                bevel = 0.02
            );
        }
        soft!(
            F,
            "pillow",
            0.34,
            0.3,
            0.12,
            (0, 0.6, -0.245),
            M.pillow_sage,
            rot = (-13, 0, 0),
            round = 0.05,
            puff = 0.035,
            flat = 2,
            round_segments = 1
        );
        for x in [-0.31, 0.31] {
            for (z, dz) in [(0.3, 1.0), (-0.385, -1.0)] {
                let sx = if x > 0.0 { 1.0 } else { -1.0 };
                cabriole(
                    &F,
                    &format!("leg{}{}", float_repr(x), float_repr(z)),
                    x,
                    z,
                    0.18,
                    M.walnut_legs,
                    0.02 * sx,
                    0.02 * dz,
                    0.024,
                    12,
                );
            }
        }
    });
    join(&still, "chair", Some((0, 0, 0).into()));
    let cushions = || {
        soft!(
            F,
            "seat",
            0.58,
            0.1,
            0.575,
            (0, 0.37, 0.013),
            M.linen_cushion,
            round = 0.035,
            puff = 0.016,
            piping = 0.006
        );
        0.42
    };
    settled(seat(cushions, 0.32, 0.013))
}

pub fn armchair_lounge() -> Vec<Obj> {
    settled(box_sofa(M.sage, M.sage_cushion, M.oak_legs, 0.82, &[], 1))
}

/// The leather armchair (the old part): black leather cushions in an open teak frame. Each side's back leg bends up
/// into the raked back post in one piece; the front legs carry paddle arms reaching back to the posts; the back
/// cushion is stitched in three channels.
pub fn armchair_leather() -> Vec<Obj> {
    let prof = rounded_rect(0.042, 0.042, 0.013, 2);
    let rail = rounded_rect(0.05, 0.035, 0.012, 2); // (height, thickness)
    let tilt = radians(14.0);
    let still = new_parts(|| {
        // the back cushion: three stuffed channels stacked along its rake, a welt round it
        for i in 0..3 {
            let o = f64::from(i - 1) * 0.165;
            cushion(
                &format!("back_cushion{i}"),
                0.56,
                0.17,
                0.1,
                (0.0, 0.69 + o * cos(tilt), -0.3 - o * sin(tilt)),
                M.leather,
                Cushion {
                    rot: P3::from((-14, 0, 0)),
                    puff: 0.022,
                    piping: None,
                    flat: Some(2),
                    ..Cushion::default()
                },
            );
        }
        fbox!(
            F,
            "back_board",
            0.54,
            0.48,
            0.03,
            (0, 0.69, -0.345),
            M.leather,
            rot = (-14, 0, 0),
            bevel = 0.01
        );
        fbox!(
            F,
            "front_rail",
            0.62,
            0.05,
            0.036,
            (0, 0.3, 0.335),
            M.teak,
            bevel = 0.012,
            bevel_segments = 2
        );
        fbox!(
            F,
            "back_rail",
            0.6,
            0.04,
            0.036,
            (0, 0.3, -0.33),
            M.teak,
            bevel = 0.01,
            bevel_segments = 2
        );
        fsweep!(
            F,
            "top_rail",
            &[(-0.35, 0.918, -0.432), (0.35, 0.918, -0.432)],
            0.0,
            M.teak,
            profile = &rail,
            up = (0, 0, 1)
        );
        for s in [-1, 1] {
            let x = f64::from(s) * 0.33;
            // the back leg and post in one bent piece
            fsweep!(
                F,
                format!("back_post{s}"),
                &[
                    (x, 0.0, -0.383),
                    (x, 0.16, -0.36),
                    (x, 0.3, -0.338),
                    (x, 0.62, -0.39),
                    (x, 0.928, -0.44)
                ],
                0.0,
                M.teak,
                profile = &prof,
                up = (1, 0, 0),
                resolution = 3,
                scale = [0.85, 0.95, 1.0, 1.0, 0.92]
            );
            // the front leg, tapering to the floor and leaning forward
            fsweep!(
                F,
                format!("front_leg{s}"),
                &[(x, 0.0, 0.37), (x, 0.3, 0.345), (x, 0.62, 0.32)],
                0.0,
                M.teak,
                profile = &prof,
                up = (1, 0, 0),
                resolution = 2,
                scale = [0.82, 1.0, 1.0]
            );
            // the arm: a teak paddle, wide and rounded at the hand, narrowing back to the post
            let paddle = smooth_outline(
                &[
                    (-0.021, -0.37),
                    (0.021, -0.37),
                    (0.035, 0.22),
                    (0.034, 0.33),
                    (0.0, 0.372),
                    (-0.034, 0.33),
                    (-0.035, 0.22),
                ],
                2,
                true,
            );
            fextrude!(
                F,
                format!("arm{s}"),
                &paddle,
                0.03,
                (x, 0.635, 0.0),
                M.teak,
                plane = Facing::Top,
                bevel = 0.009,
                bevel_segments = 2
            );
            fsweep!(
                F,
                format!("side_rail{s}"),
                &[(x, 0.3, 0.33), (x, 0.3, -0.34)],
                0.0,
                M.teak,
                profile = &rail,
                up = (0, 0, 1)
            );
        }
    });
    join(&still, "chair", Some((0, 0, 0).into()));
    let cushions = || {
        cushion(
            "seat",
            0.58,
            0.12,
            0.6,
            (0.0, 0.38, 0.03),
            M.leather,
            Cushion {
                puff: 0.014,
                piping: Some(0.006),
                keep_top: true,
                ..Cushion::default()
            },
        );
        0.44
    };
    settled(seat(cushions, 0.32, 0.03))
}

/// A moulded plywood shell: a section `w` across whose sides curl up `dish` towards the sitter, `t` thick, swept
/// along a game path (the seat front to back, a back bottom to top).
fn shell(name: &str, path: &[(i32, f64, f64)], w: f64, dish: f64, t: f64, mat: Mat) -> Obj {
    let n = 8;
    let face: Vec<(f64, f64)> = (0..=n)
        .map(|i| {
            let i = f64::from(i);
            (
                -w / 2.0 + w * i / f64::from(n),
                -dish * pow(2.0 * i / f64::from(n) - 1.0, 2.0),
            )
        })
        .collect();
    let mut profile = face.clone();
    profile.extend(face.iter().rev().map(|&(u, v)| (u, v + t)));
    fsweep!(
        F,
        name,
        path,
        0.0,
        mat,
        profile = profile,
        up = (1, 0, 0),
        resolution = 2,
        scale = 1.0,
        angle = 60
    )
}

/// The designer lounge chair (the old part): moulded walnut shells dished towards the sitter, piped leather cushions,
/// cast aluminium arms in one bent piece and a five-star swivel base with a waisted column. Still.
pub fn armchair_design() -> Vec<Obj> {
    shell(
        "seat_shell",
        &[
            (0, 0.37, 0.31),
            (0, 0.35, 0.26),
            (0, 0.34, 0.02),
            (0, 0.33, -0.25),
        ],
        0.66,
        0.035,
        0.022,
        M.ply,
    );
    cushion(
        "seat",
        0.6,
        0.1,
        0.56,
        (0.0, 0.398, 0.03),
        M.hide,
        Cushion {
            rot: P3::from((-5, 0, 0)),
            puff: 0.016,
            keep_top: true,
            ..Cushion::default()
        },
    );
    shell(
        "back_shell",
        &[(0, 0.41, -0.29), (0, 0.6, -0.355), (0, 0.79, -0.42)],
        0.66,
        0.04,
        0.022,
        M.ply,
    );
    cushion(
        "back",
        0.58,
        0.37,
        0.1,
        (0.0, 0.62, -0.3),
        M.hide,
        Cushion {
            rot: P3::from((-18, 0, 0)),
            puff: 0.022,
            flat: Some(2),
            ..Cushion::default()
        },
    );
    shell(
        "head_shell",
        &[(0, 0.765, -0.405), (0, 0.86, -0.425), (0, 0.955, -0.45)],
        0.6,
        0.035,
        0.022,
        M.ply,
    );
    cushion(
        "head",
        0.54,
        0.16,
        0.09,
        (0.0, 0.86, -0.38),
        M.hide,
        Cushion {
            rot: P3::from((-12, 0, 0)),
            puff: 0.02,
            piping: None,
            flat: Some(2),
            ..Cushion::default()
        },
    );
    for s in [-1, 1] {
        let sf = f64::from(s);
        cushion(
            &format!("arm_pad{s}"),
            0.075,
            0.045,
            0.4,
            (sf * 0.34, 0.57, -0.02),
            M.hide,
            Cushion {
                puff: 0.008,
                piping: None,
                flat: Some(1),
                rs: Some(1),
                ..Cushion::default()
            },
        );
        // the arm: one cast piece up from the seat's side, along under the pad, and down to the back
        fsweep!(
            F,
            format!("arm{s}"),
            &[
                (sf * 0.31, 0.33, 0.08),
                (sf * 0.335, 0.5, 0.05),
                (sf * 0.34, 0.545, 0.0),
                (sf * 0.34, 0.545, -0.15),
                (sf * 0.335, 0.5, -0.2),
                (sf * 0.31, 0.4, -0.26)
            ],
            0.012,
            M.alu,
            verts = 6,
            resolution = 2
        );
    }
    // the swivel: a waisted column on a hub, five flat cast spokes tapering to their glides
    let column: [&[f64]; 6] = [
        &[0.07, 0.045],
        &[0.1, 0.034],
        &[0.2, 0.03],
        &[0.28, 0.038],
        &[0.3, 0.06],
        &[0.315, 0.06],
    ];
    loft!(
        "column",
        &round_rings(&column, 20, 2.0, (0.0, 0.0)),
        material = M.alu
    );
    let hub: [&[f64]; 4] = [&[0.045, 0.05], &[0.07, 0.055], &[0.1, 0.045], &[0.11, 0.03]];
    loft!(
        "hub",
        &round_rings(&hub, 20, 2.0, (0.0, 0.0)),
        material = M.alu
    );
    for i in 0..5 {
        let a = radians(f64::from(i * 72 + 90));
        let (c, sn) = (cos(a), sin(a));
        let tip = (c * 0.36, 0.035, sn * 0.36);
        fsweep!(
            F,
            format!("star{i}"),
            &[
                (c * 0.04, 0.075, sn * 0.04),
                (c * 0.2, 0.06, sn * 0.2),
                (c * 0.345, 0.04, sn * 0.345)
            ],
            0.0,
            M.alu,
            profile = rounded_rect(0.03, 0.045, 0.01, 1),
            up = (0, 0, 1),
            resolution = 1,
            scale = [1.0, 0.8, 0.62]
        );
        fcyl!(
            F,
            format!("glide{i}"),
            0.022,
            0.024,
            (tip.0, 0.012, tip.2),
            M.black_satin,
            verts = 10
        );
    }
    join_all("armchairDesign");
    settle(&meshes()); // on its Model's `centre`, its back on the wall face
    Vec::new()
}

/// The wingback armchair (the old part): a tall back arched at the top, shaped wings sweeping forward into rolled
/// arms with welted fronts, a piped seat cushion on a deck raised on walnut legs, cabrioles in front.
pub fn armchair_wingback() -> Vec<Obj> {
    let (z, depth, w) = (-0.06, 0.8, 0.84);
    let (z0, z1) = (z - depth / 2.0, z + depth / 2.0);
    let sz0 = z0 + 0.2;
    let still = new_parts(|| {
        panel(
            "base",
            w - 0.04,
            0.15,
            depth,
            (0.0, 0.245, z),
            M.forest,
            0.03,
            0.0,
            Some(0.005),
            plain(0.03),
        );
        // the back: an arched slab, rounded all round
        let mut arch = vec![(-0.42, 0.17), (0.42, 0.17)];
        arch.extend((0..17).map(|i| {
            let i = f64::from(i);
            (
                0.42 - 0.84 * i / 16.0,
                0.99 + 0.11 * pow(sin(PI * i / 16.0), 0.7),
            )
        }));
        fextrude!(
            F,
            "back",
            &arch,
            0.17,
            (0, 0, z0 + 0.09),
            M.forest,
            bevel = 0.05,
            bevel_segments = 2
        );
        for s in [-1, 1] {
            // a wing: a shaped side panel from the back, curving down and forward onto the arm
            let wing = smooth_outline(
                &[
                    (-0.46, 0.6),
                    (-0.46, 1.0),
                    (-0.38, 1.03),
                    (-0.27, 1.0),
                    (-0.19, 0.9),
                    (-0.14, 0.76),
                    (-0.1, 0.62),
                ],
                2,
                true,
            );
            fextrude!(
                F,
                format!("wing{s}"),
                &wing,
                0.1,
                (f64::from(s) * 0.37, 0, 0),
                M.forest,
                plane = Facing::Side,
                bevel = 0.04,
                bevel_segments = 2
            );
            rolled_arm(
                &format!("arm{s}"),
                f64::from(s) * 0.34,
                s,
                z0 + 0.06,
                z1,
                0.17,
                0.64,
                0.068,
                M.forest,
                0.01,
            );
        }
        cushion(
            "back_cushion",
            0.5,
            0.48,
            0.15,
            (0.0, 0.66, sz0 + 0.06),
            M.forest_cushion,
            Cushion {
                rot: P3::from((-9, 0, 0)),
                puff: 0.03,
                piping: None,
                ..Cushion::default()
            },
        );
        for x in [-0.36, 0.36] {
            let xs = float_repr(x);
            // cabrioles in front, plain swept legs raked back behind
            cabriole(
                &F,
                &format!("leg_front{xs}"),
                x,
                z1 - 0.06,
                0.17,
                M.walnut_warm,
                0.015_f64.copysign(x),
                0.02,
                0.02,
                12,
            );
            fsweep!(
                F,
                format!("leg_back{xs}"),
                &[(x, 0.17, z0 + 0.06), (x * 1.01, 0.0, z0 + 0.045)],
                0.0,
                M.walnut_warm,
                profile = rounded_rect(0.034, 0.034, 0.008, 1),
                up = (1, 0, 0),
                scale = [1.0, 0.7]
            );
        }
    });
    join(&still, "sofa", Some((0, 0, 0).into()));
    let cushions = || {
        cushion(
            "seat",
            0.5,
            0.13,
            z1 - sz0 - 0.02,
            (0.0, 0.385, (sz0 + z1) / 2.0 + 0.005),
            M.forest_cushion,
            Cushion {
                keep_top: true,
                ..Cushion::default()
            },
        );
        0.45
    };
    settled(seat(cushions, 0.32, (sz0 + z1) / 2.0))
}
