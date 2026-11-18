//! KD1: the high chair.

use super::common::{P, place, still, turned};
use super::kit::fillet;
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::parts::{join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, slide, soft};

/// The point at height `y` on a high chair's leg from (sx * 0.25, foot, zf) up to (sx * 0.14, top, zt).
fn on_leg(sx: i32, zf: f64, zt: f64, y: f64, top: f64, foot: f64) -> P {
    let t = (y - foot) / (top - foot);
    (
        sx as f64 * (0.25 + (0.14 - 0.25) * t),
        y,
        zf + (zt - zf) * t,
    )
}

pub fn high_chair() -> Vec<Obj> {
    // the legs splayed wide at the floor, 0.54 m each way, so it can't tip (review M-kids-04: 0.44 x 0.47); each
    // tapers to a rubber foot, a stretcher between each side's pair, a footrest across the front pair
    let legs = [(-0.2, -0.07), (0.32, 0.17)];
    for sx in [-1, 1] {
        let sf = sx as f64;
        for (zf, zt) in legs {
            fsweep!(
                F,
                format!("leg{sx}{}", r(zf)),
                &[(sf * 0.14, 0.575, zt), (sf * 0.2495, 0.012, zf)],
                0.0175,
                M.chair_legs,
                verts = 10,
                scale = [1.15, 1.0]
            );
            fcyl!(
                F,
                format!("foot{sx}{}", r(zf)),
                0.016,
                0.014,
                (sf * 0.25, 0.007, zf),
                M.dark,
                verts = 8
            );
        }
        let (a, b) = (
            on_leg(sx, legs[0].0, legs[0].1, 0.2, 0.575, 0.008),
            on_leg(sx, legs[1].0, legs[1].1, 0.2, 0.575, 0.008),
        );
        fsweep!(
            F,
            format!("stretcher{sx}"),
            &[a, b],
            0.011,
            M.chair_legs,
            verts = 8
        );
    }
    let (fa, fb) = (
        on_leg(-1, legs[1].0, legs[1].1, 0.32, 0.575, 0.008),
        on_leg(1, legs[1].0, legs[1].1, 0.32, 0.575, 0.008),
    );
    fsweep!(F, "footrest_bar", &[fa, fb], 0.012, M.chair_legs, verts = 8);
    fextrude!(
        F,
        "footrest",
        &rounded_rect(0.34, 0.09, 0.03, 3),
        0.02,
        (0, 0.34, 0.225),
        M.chair_white,
        plane = Facing::Top,
        bevel = 0.006,
        bevel_segments = 1
    );
    // the seat: a moulded shell, a cushion in it; a curved back with its own cushion, rolled arms
    soft!(
        F,
        "seat",
        0.38,
        0.05,
        0.36,
        (0, 0.58, 0.05),
        M.chair_white,
        round = 0.02,
        flat = 1
    );
    soft!(
        F,
        "cushion",
        0.31,
        0.03,
        0.3,
        (0, 0.618, 0.06),
        M.chair_cushion,
        round = 0.012,
        puff = 0.007,
        piping = 0.004,
        flat = 2,
        round_segments = 1
    );
    fextrude!(
        F,
        "back",
        &rounded_rect(0.36, 0.32, 0.07, 3),
        0.03,
        (0, 0.78, -0.12),
        M.chair_white,
        rot = (-6, 0, 0),
        bevel = 0.01,
        bevel_segments = 2
    );
    soft!(
        F,
        "back_cushion",
        0.28,
        0.23,
        0.022,
        (0, 0.77, -0.097),
        M.chair_cushion,
        rot = (-6, 0, 0),
        round = 0.01,
        puff = 0.006,
        piping = 0.004,
        flat = 2,
        round_segments = 1
    );
    for s in [-1, 1] {
        let sf = s as f64;
        let pts = fillet(
            &[
                (sf * 0.17, 0.6, -0.1),
                (sf * 0.17, 0.75, -0.1),
                (sf * 0.17, 0.75, 0.16),
                (sf * 0.17, 0.6, 0.2),
            ],
            0.05,
            3,
        );
        fsweep!(
            F,
            format!("arm{s}"),
            &pts,
            0,
            M.chair_white,
            profile = rounded_rect(0.034, 0.04, 0.014, 1),
            up = (1, 0, 0)
        );
    }
    // a harness: two straps down from the back to a buckle on the cushion
    for s in [-1, 1] {
        let sf = s as f64;
        fsweep!(
            F,
            format!("strap{s}"),
            &[
                (sf * 0.07, 0.85, -0.098),
                (sf * 0.06, 0.7, -0.09),
                (sf * 0.03, 0.645, 0.0),
                (0.0, 0.64, 0.07)
            ],
            0,
            M.chair_cushion,
            profile = rounded_rect(0.004, 0.03, 0.002, 1),
            up = (0, 1, 0),
            resolution = 2
        );
    }
    fbox!(
        F,
        "buckle",
        0.04,
        0.012,
        0.04,
        (0, 0.645, 0.08),
        M.dark,
        bevel = 0.005,
        bevel_segments = 1
    );
    still("body", None);
    // the tray, its own part, sliding out to let the baby in: a rim round it, a cup well, a bowl
    let parts = new_parts(|| {
        fextrude!(
            F,
            "tray",
            &rounded_rect(0.44, 0.17, 0.05, 4),
            0.02,
            (0, 0.792, 0.2),
            M.chair_white,
            plane = Facing::Top,
            bevel = 0.006,
            bevel_segments = 1
        );
        let rim = rounded_rect(0.424, 0.154, 0.042, 3);
        let path: Vec<P> = rim.iter().map(|&(u, v)| (u, 0.804, 0.2 + v)).collect();
        fsweep!(
            F,
            "tray_rim",
            &path,
            0.006,
            M.chair_white,
            verts = 6,
            closed = true
        );
        fextrude!(
            F,
            "tray_mat",
            &rounded_rect(0.24, 0.11, 0.03, 3),
            0.003,
            (-0.05, 0.8035, 0.2),
            M.tray_mat,
            plane = Facing::Top
        );
        turned(
            "bowl",
            0.13,
            0.2,
            &[
                (0.0, 0.0),
                (0.022, 0.0),
                (0.034, 0.012),
                (0.04, 0.03),
                (0.036, 0.031),
                (0.03, 0.016),
                (0.0, 0.012),
            ],
            M.toy_green,
            12,
            0.802,
        );
        fsweep!(
            F,
            "spoon",
            &[(0.06, 0.806, 0.17), (0.1, 0.81, 0.23), (0.15, 0.83, 0.235)],
            0.0035,
            M.toy_yellow,
            verts = 5
        );
    });
    let tray = join(&parts, "tray", None);
    place(0.0);
    // out along the arms as the baby is put in (and taken out), then back in front of them
    slide!(tray, "tray_out", (0, -0.16, 0), seconds = 0.5);
    meshes()
}
