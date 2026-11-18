//! K8 continued: the upholstered, bentwood, folding and carver chairs and the dining bench.

use super::chairs::UP;
use super::common::{apron, crest, finish, rail, ring_loop, slab, sq_leg, stretcher, turned};
use super::{F, M, SZ};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{PI, TAU, V3, cos, radians, sin};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, kit, soft, sphere};

/// Upholstered chair (catalog2): a stuffed, piped seat and back in oatmeal fabric on walnut legs, seat at 0.48:
/// tapered front legs, sabre back legs, the back buttoned.
pub fn chair_upholstered() -> Vec<Obj> {
    soft!(
        F,
        "seat",
        0.46,
        0.085,
        0.44,
        (0, 0.4275, SZ + 0.01),
        M.upholstery,
        round = 0.03,
        puff = 0.01,
        piping = 0.005
    );
    let back = soft!(
        F,
        "back",
        0.44,
        0.5,
        0.075,
        (0, 0.74, SZ - 0.19),
        M.upholstery,
        rot = (-7, 0, 0),
        round = 0.03,
        puff = 0.012,
        piping = 0.005
    );
    // three buttons in a row across the back's front, pulled in a little
    for (k, bx) in [-0.11, 0.0, 0.11].into_iter().enumerate() {
        let p = back.matrix_world() * V3::from((-bx, -0.045, 0.06));
        sphere!(
            format!("button{k}"),
            0.011,
            at = p,
            material = M.upholstery,
            scale = (1, 0.5, 1),
            segments = 8,
            rings = 5
        );
    }
    for x in [-0.19, 0.19] {
        let xs = float_repr(x);
        kit::tube!(
            F,
            format!("front{xs}"),
            &[(x, 0.0, SZ + 0.17), (x, 0.4, SZ + 0.165)],
            0.02,
            M.legs_walnut,
            verts = 10,
            scale = [0.6, 1.0]
        );
        kit::tube!(
            F,
            format!("back_leg{xs}"),
            &[
                (x, 0.0, SZ - 0.252),
                (x, 0.2, SZ - 0.212),
                (x, 0.4, SZ - 0.172),
            ],
            0.017,
            M.legs_walnut,
            verts = 10,
            round = 3,
            scale = [0.72, 0.92, 1.05]
        );
    }
    finish("chairUpholstered")
}

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
    let tilt = radians(25.0);
    fsweep!(
        F,
        "back",
        &[
            (-0.205, 0.77, SZ - 0.165),
            (0.0, 0.77, SZ - 0.18),
            (0.205, 0.77, SZ - 0.165)
        ],
        0.0,
        M.seat_teal,
        profile = rounded_rect(0.2, 0.022, 0.01, 2),
        up = (0.0, sin(tilt), cos(tilt)),
        resolution = 5,
        angle = 60
    );
    finish("chairFolding")
}

/// Carver chair (catalog2): an oak armchair, padded and piped sage seat and back, the cushion's top at 0.475 (sat
/// on at 0.47, a dining chair's height, so it goes at the head of the same table): arms that sweep forward and
/// roll down into their posts, a bowed crest rail, back legs bending back into the posts.
pub fn chair_arm() -> Vec<Obj> {
    let (x, zf, zb) = (0.24, 0.23, -0.105);
    slab(
        "seat",
        0.54,
        0.44,
        0.425,
        0.05,
        M.frame_oak,
        0.025,
        None,
        3,
        (0.0, SZ),
    );
    soft!(
        F,
        "cushion",
        0.46,
        0.06,
        0.4,
        (0, 0.437, SZ + 0.01),
        M.seat_sage,
        round = 0.022,
        puff = 0.008,
        piping = 0.005
    );
    soft!(
        F,
        "back_pad",
        0.42,
        0.3,
        0.05,
        (0, 0.70, SZ - 0.17),
        M.seat_sage,
        rot = (-6, 0, 0),
        round = 0.02,
        puff = 0.01
    );
    for s in [-1, 1] {
        let sf = f64::from(s);
        let sx = sf * x;
        sq_leg(
            &format!("front{s}"),
            sx,
            zf,
            0.0,
            0.4,
            0.03,
            0.042,
            M.frame_oak,
            0.007,
            0.003,
            None,
        );
        fsweep!(
            F,
            format!("back{s}"),
            &[
                (sx, 0.0, zb),
                (sx, 0.03, zb),
                (sx, 0.4, zb - 0.025),
                (sx, 0.66, zb - 0.045),
                (sx, 0.918, zb - 0.07)
            ],
            0.0,
            M.frame_oak,
            profile = rounded_rect(0.04, 0.04, 0.009, 1),
            up = (-1, 0, 0),
            resolution = 3,
            scale = [0.92, 0.92, 1.0, 1.0, 0.95],
            angle = 60
        );
        // the arm: from the back post forward, rolling down over its front into the post, one shaped piece
        let ax = sf * 0.255;
        fsweep!(
            F,
            format!("arm{s}"),
            &[
                (ax, 0.655, zb - 0.03),
                (ax, 0.662, 0.05),
                (ax, 0.665, SZ + 0.15),
                (ax, 0.64, SZ + 0.2),
                (ax, 0.56, SZ + 0.192),
                (ax, 0.42, SZ + 0.18)
            ],
            0.0,
            M.frame_oak,
            profile = rounded_rect(0.032, 0.058, 0.012, 2),
            up = (0, 0, 1),
            resolution = 3,
            scale = [0.9, 1.0, 1.1, 1.05, 0.8, 0.85],
            angle = 60
        );
        stretcher(
            &format!("stretcher{s}"),
            (sx, 0.18, zf),
            (sx, 0.18, zb - 0.01),
            0.012,
            M.frame_oak,
            1.3,
            8,
        );
        apron(
            &format!("apron_s{s}"),
            zf - zb - 0.03,
            0.05,
            0.02,
            (sf * (x - 0.004), 0.4, (zf + zb) / 2.0),
            M.frame_oak,
            Facing::Side,
            0.012,
            10,
            0.003,
        );
    }
    crest(
        "top_rail",
        x + 0.01,
        0.88,
        zb - 0.064,
        0.075,
        0.035,
        M.frame_oak,
        0.012,
        1.12,
        UP,
        3,
    );
    apron(
        "apron_f",
        2.0 * x - 0.03,
        0.05,
        0.02,
        (0, 0.4, zf - 0.005),
        M.frame_oak,
        Facing::Front,
        0.016,
        10,
        0.003,
    );
    stretcher(
        "stretcher_front",
        (-x, 0.2, zf),
        (x, 0.2, zf),
        0.012,
        M.frame_oak,
        1.3,
        8,
    );
    finish("chairArm")
}

/// Dining bench (catalog2, 2x1, two sitters): an oak plank with a rounded edge on splayed darker legs, through a
/// shaped cleat at each end, an arched rail under it and pegged stretchers.
pub fn bench_dining() -> Vec<Obj> {
    for (k, dz) in [-0.12, 0.0, 0.12].into_iter().enumerate() {
        // three planks, a rounded groove between each
        slab(
            &format!("seat{k}"),
            1.6,
            0.12,
            0.465,
            0.05,
            M.seat_oak,
            0.01,
            Some(&[
                (0.004, 0.0),
                (0.0, 0.25),
                (0.0, 0.8),
                (0.0025, 0.95),
                (0.005, 1.0),
            ]),
            3,
            (0.0, SZ + dz),
        );
    }
    apron(
        "rail",
        1.36,
        0.06,
        0.03,
        (0, 0.415, SZ),
        M.legs_oakd,
        Facing::Front,
        0.022,
        14,
        0.003,
    );
    for x in [-0.68, 0.68] {
        let s = if x > 0.0 { 1.0 } else { -1.0 };
        let xs = float_repr(x);
        fextrude!(
            F,
            format!("cleat{xs}"),
            &[
                (-0.15, 0.0175),
                (0.15, 0.0175),
                (0.15, 0.0),
                (0.12, -0.0175),
                (-0.12, -0.0175),
                (-0.15, 0.0)
            ],
            0.07,
            (x, 0.3975, SZ),
            M.legs_oakd,
            plane = Facing::Side,
            bevel = 0.004,
            bevel_segments = 1
        );
        for dz in [-0.12, 0.12] {
            sq_leg(
                &format!("leg{xs}{}", float_repr(dz)),
                x + s * 0.05,
                SZ + dz * 1.25,
                0.0,
                0.415,
                0.04,
                0.046,
                M.legs_oakd,
                0.008,
                0.003,
                Some((x, SZ + dz)),
            );
        }
        kit::tube!(
            F,
            format!("stretcher{xs}"),
            &[
                (x + s * 0.025, 0.15, SZ - 0.142),
                (x + s * 0.025, 0.15, SZ + 0.142),
            ],
            0.014,
            M.legs_oakd,
            verts = 8
        );
        for dz in [-0.1, 0.1] {
            fcyl!(
                F,
                format!("peg{xs}{}", float_repr(dz)),
                0.006,
                0.05,
                (x + s * 0.025, 0.15, SZ + dz * 1.42),
                M.seat_oak,
                rot = (0, 90, 0),
                verts = 6
            );
        }
    }
    finish("benchDining")
}
