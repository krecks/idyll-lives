//! K8 continued: the bentwood and folding chairs.

use super::common::{bowed_pad, finish, rail, ring_loop, turned};
use super::{F, M, SZ};
use artkit::json::float_repr;
use artkit::math::{PI, TAU, cos, radians, sin};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fsweep, kit, soft};

/// Bentwood chair (catalog2): dark steam-bent beech, a round seat ring with a red cane insert, the back legs and
/// the back's loop bent from one length, a smaller loop inside it, a leg ring and knee braces.
pub fn chair_bentwood() -> Vec<Obj> {
    turned(
        "seat",
        &[
            (0.0, 0.0),
            (0.205, 0.0),
            (0.216, 0.01),
            (0.217, 0.022),
            (0.21, 0.033),
            (0.197, 0.035),
            (0.176, 0.035),
            (0.172, 0.03),
            (0.172, 0.02),
            (0.0, 0.02),
        ],
        (0, 0.44, SZ),
        M.frame_bent,
        28,
        35.0,
    );
    turned(
        "cane",
        &[
            (0.0, 0.0),
            (0.174, 0.0),
            (0.174, 0.03),
            (0.16, 0.036),
            (0.08, 0.0395),
            (0.0, 0.04),
        ],
        (0, 0.44, SZ),
        M.seat_bent,
        24,
        35.0,
    );
    ring_loop(
        "seat_rim",
        0.203,
        0.203,
        0.432,
        0.011,
        M.frame_bent,
        0.0,
        SZ,
        6,
        28,
    );
    ring_loop(
        "leg_ring",
        0.166,
        0.166,
        0.2,
        0.011,
        M.frame_bent,
        0.0,
        SZ,
        6,
        24,
    );
    for sx in [-1, 1] {
        let s = f64::from(sx);
        kit::tube!(
            F,
            format!("front{sx}"),
            &[
                (s * 0.18, 0.0, SZ + 0.2),
                (s * 0.155, 0.22, SZ + 0.162),
                (s * 0.14, 0.432, SZ + 0.13),
            ],
            0.016,
            M.frame_bent,
            verts = 8,
            round = 2,
            scale = [0.78, 0.95, 1.05]
        );
        // a knee brace from the leg up under the seat rim
        kit::tube!(
            F,
            format!("brace{sx}"),
            &[
                (s * 0.15, 0.33, SZ + 0.145),
                (s * 0.12, 0.4, SZ + 0.1),
                (s * 0.06, 0.425, SZ + 0.13),
            ],
            0.007,
            M.frame_bent,
            verts = 6,
            round = 3
        );
    }
    // the back legs and the loop of the back in one length: up from the floor past the seat, round the top, down
    let (y0, ry, z0, back, rx) = (0.44, 0.51, SZ - 0.17, 0.05, 0.155);
    let mut arc = Vec::new();
    for i in 1..18 {
        let t = PI * f64::from(i) / 18.0;
        let yy = y0 + ry * sin(t);
        arc.push((rx * cos(t), yy, z0 - back * (yy - y0) / ry));
    }
    let mut pts = vec![
        (0.17, 0.0, SZ - 0.2),
        (0.16, 0.22, SZ - 0.183),
        (0.155, 0.44, z0),
    ];
    pts.extend(arc);
    pts.extend([
        (-0.155, 0.44, z0),
        (-0.16, 0.22, SZ - 0.183),
        (-0.17, 0.0, SZ - 0.2),
    ]);
    let mut taper = vec![0.8, 0.95, 1.05];
    taper.extend((1..18).map(|i| 1.0 - 0.12 * sin(PI * f64::from(i) / 18.0)));
    taper.extend([1.05, 0.95, 0.8]);
    kit::tube!(
        F,
        "back",
        &pts,
        0.015,
        M.frame_bent,
        verts = 8,
        round = 2,
        scale = taper
    );
    // the inner loop, leaning with the back, tied to it each side
    let lean = radians(6.0);
    let inner: Vec<(f64, f64, f64)> = (0..20)
        .map(|i| {
            let a = TAU * f64::from(i) / 20.0;
            (
                0.085 * cos(a),
                0.71 + 0.11 * sin(a) * cos(lean),
                SZ - 0.195 - 0.11 * sin(a) * sin(lean),
            )
        })
        .collect();
    fsweep!(
        F,
        "back_in",
        &inner,
        0.01,
        M.frame_bent,
        verts = 6,
        closed = true
    );
    for s in [-1, 1] {
        let sf = f64::from(s);
        kit::tube!(
            F,
            format!("tie{s}"),
            &[
                (sf * 0.084, 0.71, SZ - 0.195),
                (sf * 0.154, 0.7, SZ - 0.196),
            ],
            0.007,
            M.frame_bent,
            verts = 6
        );
    }
    finish("chairBentwood")
}

/// Folding chair (catalog2): a steel frame of two bent tubes crossing at rivets, a padded teal seat on a pan and
/// a curved back panel, rubber feet.
pub fn chair_folding() -> Vec<Obj> {
    let (xf, xr) = (0.215, 0.2);
    // the front legs rise into the back posts and bend across the top: one tube
    rail(
        "frame_front",
        &[
            (-xf, 0.012, SZ + 0.22),
            (-xf, 0.88, SZ - 0.2),
            (xf, 0.88, SZ - 0.2),
            (xf, 0.012, SZ + 0.22),
        ],
        0.011,
        M.frame_steel,
        0.05,
        8,
        3,
        false,
    );
    // the rear legs rise under the seat's front and bend across it: the other tube
    rail(
        "frame_rear",
        &[
            (-xr, 0.012, SZ - 0.2),
            (-xr, 0.43, SZ + 0.13),
            (xr, 0.43, SZ + 0.13),
            (xr, 0.012, SZ - 0.2),
        ],
        0.011,
        M.frame_steel,
        0.04,
        8,
        3,
        false,
    );
    kit::tube!(
        F,
        "stretcher",
        &[(-xf, 0.1, SZ + 0.15), (xf, 0.1, SZ + 0.15)],
        0.009,
        M.frame_steel,
        verts = 8
    );
    kit::tube!(
        F,
        "stretcher_rear",
        &[(-xr, 0.1, SZ - 0.15), (xr, 0.1, SZ - 0.15)],
        0.009,
        M.frame_steel,
        verts = 8
    );
    for s in [-1, 1] {
        let sf = f64::from(s);
        fcyl!(
            F,
            format!("pivot{s}"),
            0.012,
            0.012,
            (sf * (xf + 0.011), 0.29, SZ - 0.02),
            M.frame_steel,
            rot = (0, 90, 0),
            verts = 10,
            bevel = 0.003,
            bevel_segments = 1
        );
        // the seat's hinge brackets at the back, on the back posts
        fbox!(
            F,
            format!("bracket{s}"),
            0.006,
            0.05,
            0.04,
            (sf * (xf - 0.013), 0.43, SZ - 0.13),
            M.frame_steel,
            bevel = 0.002
        );
        // rubber feet on the four tube ends
        for (zz, xx) in [(SZ + 0.22, sf * xf), (SZ - 0.2, sf * xr)] {
            fbox!(
                F,
                format!("foot{s}{}", float_repr(zz)),
                0.03,
                0.022,
                0.055,
                (xx, 0.011, zz),
                M.cap,
                bevel = 0.008,
                bevel_segments = 2
            );
        }
    }
    fbox!(
        F,
        "pan",
        0.41,
        0.012,
        0.37,
        (0, 0.437, SZ + 0.02),
        M.frame_steel,
        bevel = 0.005,
        bevel_segments = 1
    );
    soft!(
        F,
        "seat",
        0.42,
        0.03,
        0.38,
        (0, 0.445, SZ + 0.02),
        M.seat_teal,
        round = 0.012,
        puff = 0.005
    );
    // the back: a padded board, its ends rounded, bowed round the sitter and leant back 25°
    bowed_pad(
        "back",
        0.41,
        0.2,
        0.022,
        (0.0, 0.77, SZ - 0.165),
        0.015,
        25.0,
        M.seat_teal,
    );
    finish("chairFolding")
}
