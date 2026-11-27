use super::super::common::{F, M, side_slab, tapered_leg};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{Axis, P3, TAU, cos, cosd, radians, sin, sind};
use artkit::motion::{Swing, swing};
use artkit::parts::{hinged, join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsweep, settle, soft};

// --- OU2: the sun lounger: a slatted platform and cushion; the back reclines flat for sunbathing ---------------
pub(super) const BACK_HINGE: (f64, f64) = (0.29, -0.36); // the back's hinge, (height, depth)

/// A side outline (depth, height) from z0 to z1, y0 to y1, its two ends rounded by `r`.
pub(super) fn rounded_side(z0: f64, z1: f64, y0: f64, y1: f64, r: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for (cz, a0) in [(z1 - r, -90), (z0 + r, 90)] {
        for i in 0..5 {
            let a = radians(a0 as f64 + (180 * i) as f64 / 4.0);
            out.push((cz + r * cos(a), (y0 + y1) / 2.0 + (y1 - y0) / 2.0 * sin(a)));
        }
    }
    out
}

pub fn sun_lounger() -> Vec<Obj> {
    let still = new_parts(|| {
        for s in [-1, 1] {
            let sf = s as f64;
            // the side rails, rounded at their ends; tapered legs with a cross rail between each pair
            side_slab(
                &format!("rail{s}"),
                sf * 0.29,
                &rounded_side(-0.38, 0.94, 0.22, 0.29, 0.03),
                0.04,
                M.lounger,
                0.005,
            );
            for z in [-0.3, 0.86] {
                tapered_leg(
                    &format!("leg{s}{}", float_repr(z)),
                    sf * 0.275,
                    z,
                    0.0,
                    0.235,
                    0.04,
                    0.05,
                    M.lounger,
                    12,
                );
            }
        }
        for z in [-0.3, 0.86] {
            fbox!(
                F,
                format!("leg_rail{}", float_repr(z)),
                0.5,
                0.04,
                0.03,
                (0, 0.1, z),
                M.lounger,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for k in 0..9 {
            fbox!(
                F,
                format!("slat{k}"),
                0.54,
                0.02,
                0.11,
                (0, 0.27, -0.3 + 0.145 * k as f64),
                M.lounger,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        // the cushions: the long one and the seat's end under the back, piped, their middles at the old tops (0.35)
        soft!(
            F,
            "cushion",
            0.56,
            0.052,
            0.95,
            (0, 0.32, 0.445),
            M.lounger_pad,
            round = 0.025,
            puff = 0.008,
            flat = 2,
            piping = 0.004,
            round_segments = 1
        );
        soft!(
            F,
            "seat_end",
            0.56,
            0.052,
            0.32,
            (0, 0.32, -0.198),
            M.lounger_pad,
            round = 0.025,
            puff = 0.008,
            flat = 1,
            piping = 0.004,
            round_segments = 1
        );
        // a beach towel laid along the cushion and over its foot, and one rolled up
        let drape = kit::fillet!(
            &[(0.0, 0.355, 0.55), (0.0, 0.356, 0.915), (0.0, 0.24, 0.93)],
            0.02,
            steps = 2
        );
        fsweep!(
            F,
            "towel",
            &drape,
            0,
            M.towel,
            profile = rounded_rect(0.4, 0.008, 0.003, 1),
            up = (1, 0, 0)
        );
        let roll: Vec<(f64, f64, f64)> = [-0.2, 0.2].iter().map(|&x| (x, 0.375, 0.5)).collect();
        let prof: Vec<(f64, f64)> = (0..10)
            .map(|i| {
                (
                    0.02 * cos(TAU * i as f64 / 10.0) * (1.0 + 0.08 * (i % 2) as f64),
                    0.02 * sin(TAU * i as f64 / 10.0),
                )
            })
            .collect();
        fsweep!(
            F,
            "towel_roll",
            &roll,
            0,
            M.towel,
            profile = prof,
            up = (0, 0, 1)
        );
    });
    join(&still, "lounger", Some(P3::from((0, 0, 0))));
    let (y0, z0) = BACK_HINGE;
    let back = new_parts(|| {
        let a = 35;
        let along = (0, sind(a as f64), -cosd(a as f64)); // up the back
        let face = (0, cosd(a as f64), sind(a as f64));
        let c = (
            0,
            y0 + 0.345 * along.1 + 0.012 * face.1,
            z0 + 0.345 * along.2 + 0.012 * face.2,
        );
        soft!(
            F,
            "back_pad",
            0.56,
            0.68,
            0.07,
            c,
            M.lounger_pad,
            rot = (-(90 - a), 0, 0),
            round = 0.028,
            puff = 0.01,
            flat = 2,
            piping = 0.004,
            round_segments = 1,
            pinch = 0.0
        );
        let mid = (
            0,
            y0 + 0.585 * along.1 + 0.07 * face.1,
            z0 + 0.585 * along.2 + 0.07 * face.2,
        );
        // (the pillow's top edge is the lounger's highest point: neither pinched nor drawn in, nor is the pad)
        soft!(
            F,
            "pillow",
            0.42,
            0.18,
            0.08,
            mid,
            M.pillow,
            rot = (-(90 - a), 0, 0),
            round = 0.035,
            puff = 0.02,
            pinch = 0.0
        );
    });
    let back = hinged(join(&back, "back", None), F.at(0, y0, z0));
    settle!(centre = false);
    swing(
        back,
        "recline",
        -30.0,
        Swing {
            seconds: 1.0,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    meshes()
}
