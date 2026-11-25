//! The fire engine: 3x6, a UK fire appliance, its cab to the front. 2.5 m wide over its mirrors, 3.2 m high to its
//! ladder, 5.84 m long: a crew cab over the front axle, a body of roller-shuttered lockers, a triple ladder and two
//! hard suction hoses on the roof, blue lights on the cab.

use super::common::{F, arc, own_palette, slats, slats_back, tyre};
use artkit::far::{is_low, low};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{Axis, TAU, atan2, cos, degrees, sin};
use artkit::parts::{join_all, settle_all};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, kit_tube, revolve};

own_palette! { ENGINE:
    body = ("#B8221E", gloss),        // the red paint
    shutter = ("#C3C7CA", metal),     // the lockers' roller shutters
    stripe = ("#F2D33A", gloss),      // the reflective band
    ladder = ("#B9BDC1", metal),      // the ladders
}

pub const FE_FRONT: f64 = 2.86; // the cab's face
pub const FE_BACK: f64 = -2.88; // the body's back (the bumpers stand a little past them)
pub const FE_CAB: f64 = 0.92; // the cab's back
pub const FE_BODY: f64 = 0.86; // the body's front
pub const FE_FA: f64 = 2.0; // the front axle's depth
pub const FE_RA: f64 = -1.6; // the rear axle's depth
pub const FE_R: f64 = 0.5; // the wheels' radius
pub const FE_SILL: f64 = 0.72; // the cab's and the body's underside
pub const FE_HALF: f64 = 1.15; // the body's half width (the cab's 1.175)
pub const FE_ROOF: f64 = 2.85; // the body's roof (the cab's 2.9)

/// A wheel on the axle at depth z: the tyre, a silver rim dished in to the hub, ten nuts. `inner`: the inner one
/// of a rear pair, plainer (only its tread shows).
pub fn fe_wheel(name: &str, x: f64, z: f64, m: &ENGINE, inner: bool) {
    let s = if x > 0.0 { 1 } else { -1 };
    let w = 0.3;
    tyre(
        &format!("{name}_tyre"),
        x,
        FE_R,
        z,
        FE_R,
        w,
        m.rubber,
        None,
        Some(if inner { low(16, 12) } else { low(24, 16) }),
        Axis::X,
    );
    if inner {
        return;
    }
    revolve!(
        format!("{name}_rim"),
        &[
            (0.0, -w / 2.0 + 0.04),
            (0.33, -w / 2.0 + 0.03),
            (0.335, w / 2.0 - 0.012),
            (0.3, w / 2.0 - 0.02),
            (0.13, w / 2.0 - 0.06),
            (0.08, w / 2.0 - 0.03),
            (0.0, w / 2.0 - 0.03)
        ],
        at = F.at(x, FE_R, z),
        material = m.steel,
        rot = (0, -90 * s, 0),
        segments = low(16, 12)
    );
    if is_low() {
        return;
    }
    let sf = s as f64;
    for k in 0..6 {
        let a = TAU * k as f64 / 6.0;
        fcyl!(
            F,
            format!("{name}_nut{k}"),
            0.014,
            0.03,
            (
                x + sf * (w / 2.0 - 0.02),
                FE_R + 0.1 * sin(a),
                z + 0.1 * cos(a)
            ),
            m.chrome,
            rot = (0, 90, 0),
            verts = 6
        );
    }
    crate::eased(fcyl!(
        F,
        format!("{name}_cap"),
        0.055,
        0.04,
        (x + sf * (w / 2.0 - 0.02), FE_R, z),
        m.chrome,
        rot = (0, 90, 0),
        verts = 12
    ));
}

pub fn fire_engine() -> Vec<Obj> {
    let m = &ENGINE;
    let red = m.body;
    let hx = FE_HALF + 0.025; // the cab's half width
    // the cab: one slab drawn from the side, the front wheel's arch cut out of it, its windscreen raked back
    let mut cab = vec![(FE_CAB, FE_SILL)];
    cab.extend(arc(FE_FA, FE_R, 0.57, FE_SILL, low(14, 6)));
    cab.extend([
        (FE_FRONT, FE_SILL),
        (FE_FRONT, 1.62),
        (FE_FRONT - 0.03, 1.7),
        (FE_FRONT - 0.15, 2.7),
        (FE_FRONT - 0.21, 2.84),
        (FE_FRONT - 0.33, 2.9),
        (FE_CAB + 0.06, 2.9),
        (FE_CAB, 2.85),
    ]);
    fextrude!(
        F,
        "cab",
        &cab,
        2.0 * hx,
        (0, 0, 0),
        red,
        plane = Facing::Side,
        bevel = low(0.035, 0.0),
        bevel_segments = 2
    );
    // the body: the lockers' box, the rear wheel's arch cut out of it
    let mut body = vec![(FE_BACK, FE_SILL)];
    body.extend(arc(FE_RA, FE_R, 0.58, FE_SILL, low(14, 6)));
    body.extend([(FE_BODY, FE_SILL), (FE_BODY, FE_ROOF), (FE_BACK, FE_ROOF)]);
    fextrude!(
        F,
        "body",
        &body,
        2.0 * FE_HALF,
        (0, 0, 0),
        red,
        plane = Facing::Side,
        bevel = low(0.02, 0.0),
        bevel_segments = 2
    );
    fbox!(
        F,
        "cab_gap",
        2.0 * FE_HALF - 0.04,
        FE_ROOF - FE_SILL - 0.1,
        FE_CAB - FE_BODY + 0.02,
        (0, (FE_ROOF + FE_SILL) / 2.0, (FE_CAB + FE_BODY) / 2.0),
        m.dark
    );
    // the chassis under it all, the bumpers front and back
    for s in [-1, 1] {
        fbox!(
            F,
            format!("rail{s}"),
            0.12,
            0.26,
            5.5,
            (s as f64 * 0.45, 0.6, 0.1),
            m.iron
        );
    }
    fbox!(
        F,
        "bumper",
        2.36,
        0.36,
        0.14,
        (0, 0.55, FE_FRONT - 0.01),
        m.plastic,
        bevel = low(0.03, 0.0),
        bevel_segments = 2
    );
    fbox!(
        F,
        "rear_bumper",
        2.2,
        0.26,
        0.1,
        (0, 0.57, FE_BACK + 0.01),
        m.yellow,
        bevel = low(0.015, 0.0),
        bevel_segments = 1
    );
    // the wheels: singles at the front, pairs at the back
    for s in [-1, 1] {
        let sf = s as f64;
        fe_wheel(&format!("front{s}"), sf * 1.0, FE_FA, m, false);
        fe_wheel(&format!("rear{s}"), sf * 1.0, FE_RA, m, false);
        fe_wheel(&format!("rear_in{s}"), sf * 0.68, FE_RA, m, true);
        fbox!(
            F,
            format!("axle_f{s}"),
            0.6,
            0.1,
            0.1,
            (sf * 0.55, FE_R, FE_FA),
            m.iron
        );
        fbox!(
            F,
            format!("axle_r{s}"),
            0.5,
            0.14,
            0.14,
            (sf * 0.5, FE_R, FE_RA),
            m.iron
        );
    }
    // the windscreen, the side windows and the doors' seams
    let t = atan2(0.12, 1.0);
    fbox!(
        F,
        "windscreen",
        2.12,
        0.98,
        0.012,
        (0, 2.19, FE_FRONT - 0.09 + 0.008),
        m.tinted,
        rot = (-degrees(t), 0, 0)
    );
    for s in [-1, 1] {
        let sf = s as f64;
        let xs = sf * (hx + 0.004);
        for (k, (z0, z1)) in [(1.86, 2.62), (0.98, 1.78)].into_iter().enumerate() {
            fextrude!(
                F,
                format!("window{s}{k}"),
                &rounded_rect(z1 - z0, 0.72, 0.05, low(3, 1)),
                0.01,
                (xs, 2.3, (z0 + z1) / 2.0),
                m.tinted,
                plane = Facing::Side
            );
        }
        if !is_low() {
            for z in [0.95, 1.82, 2.66] {
                fbox!(
                    F,
                    format!("seam{s}{}", float_repr(z)),
                    0.004,
                    1.9,
                    0.01,
                    (xs, 1.76, z),
                    m.dark
                );
            }
            for (k, z) in [1.9, 1.02].into_iter().enumerate() {
                fbox!(
                    F,
                    format!("door_handle{s}{k}"),
                    0.02,
                    0.03,
                    0.16,
                    (sf * (hx + 0.012), 1.72, z + 0.1),
                    m.chrome
                );
            }
            for (k, z) in [1.76, 0.97].into_iter().enumerate() {
                // grab rails beside the doors
                kit_tube!(
                    F,
                    format!("grab{s}{k}"),
                    &[
                        (sf * (hx + 0.005), 1.25, z),
                        (sf * (hx + 0.04), 1.3, z),
                        (sf * (hx + 0.04), 2.0, z),
                        (sf * (hx + 0.005), 2.05, z)
                    ],
                    0.014,
                    m.chrome,
                    round = 1
                );
            }
        }
        // the steps under the crew door, the mirror on its arms
        fbox!(
            F,
            format!("step{s}"),
            0.24,
            0.05,
            0.62,
            (sf * (hx - 0.1), 0.48, 1.18),
            m.plastic
        );
        fbox!(
            F,
            format!("step2{s}"),
            0.2,
            0.04,
            0.62,
            (sf * (hx - 0.12), 0.2 + 0.06, 1.18),
            m.plastic
        );
        kit_tube!(
            F,
            format!("mirror_arm{s}"),
            &[
                (sf * hx, 2.2, FE_FRONT - 0.32),
                (sf * 1.24, 2.3, FE_FRONT - 0.32),
                (sf * 1.24, 2.0, FE_FRONT - 0.32)
            ],
            0.015,
            m.chrome,
            round = 1
        );
        fbox!(
            F,
            format!("mirror{s}"),
            0.05,
            0.36,
            0.2,
            (sf * 1.225, 2.1, FE_FRONT - 0.3),
            m.plastic,
            bevel = low(0.02, 0.0),
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("mirror_glass{s}"),
            0.004,
            0.32,
            0.16,
            (sf * 1.225, 2.1, FE_FRONT - 0.402),
            m.tinted
        );
        // the reflective band along the cab and the body, under the windows and over the lockers
        fbox!(
            F,
            format!("band_cab{s}"),
            0.004,
            0.08,
            FE_FRONT - FE_CAB - 0.06,
            (sf * (hx + 0.003), 1.42, (FE_FRONT + FE_CAB) / 2.0 - 0.02),
            m.stripe
        );
        fbox!(
            F,
            format!("band_top{s}"),
            0.004,
            0.06,
            FE_BODY - FE_BACK - 0.06,
            (sf * (FE_HALF + 0.003), 2.79, (FE_BODY + FE_BACK) / 2.0),
            m.stripe
        );
        // the arches' black trims
        for (zc, r) in [(FE_FA, 0.57), (FE_RA, 0.58)] {
            let pts: Vec<(f64, f64, f64)> = arc(zc, FE_R, r + 0.02, FE_SILL + 0.02, low(10, 6))
                .into_iter()
                .map(|(z, y)| (sf * (if zc > 0.0 { hx } else { FE_HALF }), y, z))
                .collect();
            crate::eased(fsweep!(
                F,
                format!("arch{s}{}", float_repr(zc)),
                &pts,
                0,
                m.plastic,
                profile = rounded_rect(0.06, 0.05, 0.0, 1),
                up = (0, 0, 1)
            ));
        }
        // the side guard between the wheels: two rails on brackets
        for y in [0.4, 0.62] {
            fbox!(
                F,
                format!("guard{s}{}", float_repr(y)),
                0.03,
                0.06,
                2.3,
                (sf * (FE_HALF - 0.04), y, 0.25 - 0.08),
                m.steel,
                bevel = low(0.008, 0.0),
                bevel_segments = 1
            );
        }
        for z in [-0.82, 0.25, 1.32] {
            fbox!(
                F,
                format!("guard_post{s}{}", float_repr(z)),
                0.03,
                0.34,
                0.05,
                (sf * (FE_HALF - 0.06), 0.55, z),
                m.iron
            );
        }
    }
    // the lockers: roller shutters along each side and at the back, each with its drop bar
    let lockers = [
        (0.8, 0.04, 0.84),
        (-0.02, -0.96, 0.84),
        (-1.04, -2.16, 1.2),
        (-2.24, -2.82, 0.84),
    ];
    for s in [-1, 1] {
        let sf = s as f64;
        for (k, (z0, z1, y0)) in lockers.into_iter().enumerate() {
            slats(
                &format!("shutter{s}{k}"),
                sf * FE_HALF,
                s,
                z1,
                z0,
                y0,
                2.72,
                m.shutter,
                0.07,
                0.008,
            );
            fbox!(
                F,
                format!("drop_bar{s}{k}"),
                0.035,
                0.04,
                z0 - z1 - 0.04,
                (sf * (FE_HALF + 0.02), y0 + 0.03, (z0 + z1) / 2.0),
                m.chrome
            );
        }
    }
    slats_back(
        "shutter_back",
        FE_BACK,
        -0.8,
        0.8,
        0.84,
        2.72,
        m.shutter,
        0.07,
        0.008,
    );
    fbox!(
        F,
        "drop_bar_back",
        1.56,
        0.04,
        0.035,
        (0, 0.87, FE_BACK - 0.02),
        m.chrome
    );
    // the rear: tail lamps on the corners, the chevrons on the bumper
    for s in [-1, 1] {
        let sf = s as f64;
        for (k, (y, mat)) in [(1.25, m.amber), (1.1, m.red_lens), (0.95, m.red_lens)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                F,
                format!("tail{s}{k}"),
                0.18,
                0.13,
                0.03,
                (sf * 0.98, y, FE_BACK - 0.01),
                mat
            );
        }
        fbox!(
            F,
            format!("reverse{s}"),
            0.12,
            0.08,
            0.03,
            (sf * 0.98, 1.4, FE_BACK - 0.01),
            m.lamp
        );
    }
    if !is_low() {
        for k in -4..5 {
            let x0 = k as f64 * 0.24;
            let s = if k >= 0 { 1.0 } else { -1.0 };
            fextrude!(
                F,
                format!("chevron{k}"),
                &[
                    (x0 - 0.04, 0.44),
                    (x0 + 0.04, 0.44),
                    (x0 + 0.04 + s * 0.12, 0.7),
                    (x0 - 0.04 + s * 0.12, 0.7)
                ],
                0.004,
                (0, 0, FE_BACK - 0.041),
                red,
                plane = Facing::Front
            );
        }
    }
    // the front: grille, headlamps, indicators, blue lights in the grille, a white sign over the windscreen
    fbox!(
        F,
        "grille",
        1.5,
        0.5,
        0.02,
        (0, 1.18, FE_FRONT + 0.005),
        m.dark
    );
    if !is_low() {
        for k in 0..5 {
            fbox!(
                F,
                format!("grille_bar{k}"),
                1.46,
                0.025,
                0.02,
                (0, 0.98 + k as f64 * 0.1, FE_FRONT + 0.016),
                m.chrome
            );
        }
    }
    for s in [-1, 1] {
        let sf = s as f64;
        crate::eased(fextrude!(
            F,
            format!("head_bezel{s}"),
            &rounded_rect(0.36, 0.2, 0.04, low(3, 1)),
            0.03,
            (sf * 0.88, 0.93, FE_FRONT + 0.01),
            m.chrome
        ));
        fextrude!(
            F,
            format!("head_lens{s}"),
            &rounded_rect(0.32, 0.16, 0.03, low(3, 1)),
            0.03,
            (sf * 0.88, 0.93, FE_FRONT + 0.018),
            m.lamp
        );
        fbox!(
            F,
            format!("indicator{s}"),
            0.1,
            0.16,
            0.03,
            (sf * 1.1, 0.93, FE_FRONT + 0.01),
            m.amber,
            bevel = low(0.01, 0.0),
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("grille_blue{s}"),
            0.16,
            0.07,
            0.03,
            (sf * 0.5, 0.86, FE_FRONT + 0.012),
            m.blue_lens,
            bevel = low(0.01, 0.0),
            bevel_segments = 1
        );
        fcyl!(
            F,
            format!("fog{s}"),
            0.05,
            0.03,
            (sf * 0.85, 0.55, FE_FRONT + 0.045),
            m.lamp,
            rot = (90, 0, 0),
            verts = low(16, 8)
        );
    }
    fbox!(
        F,
        "sign",
        1.5,
        0.14,
        0.03,
        (0, 2.79, FE_FRONT - 0.2),
        m.white,
        rot = (-degrees(t), 0, 0),
        bevel = low(0.01, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "sign_line",
        1.2,
        0.035,
        0.03,
        (0, 2.79, FE_FRONT - 0.196),
        red,
        rot = (-degrees(t), 0, 0)
    );
    // the light bar on the cab's roof: blue lenses either side of a white middle, on a black base
    fbox!(
        F,
        "lightbar_base",
        2.0,
        0.05,
        0.32,
        (0, 2.925, FE_FRONT - 0.48),
        m.plastic,
        bevel = low(0.015, 0.0),
        bevel_segments = 1
    );
    for k in 0..6 {
        let x = -0.83 + k as f64 * 0.332;
        fbox!(
            F,
            format!("light{k}"),
            0.3,
            0.1,
            0.28,
            (x, 3.0, FE_FRONT - 0.48),
            if k == 2 || k == 3 {
                m.lamp
            } else {
                m.blue_lens
            },
            bevel = low(0.03, 0.0),
            bevel_segments = 1
        );
    }
    // the roof: the ladders on two gantries, the hard suction hoses, a walkway, beacons at the back
    fbox!(
        F,
        "walkway",
        2.0 * FE_HALF - 0.1,
        0.01,
        FE_BODY - FE_BACK - 0.1,
        (0, FE_ROOF + 0.005, (FE_BODY + FE_BACK) / 2.0),
        m.steel
    );
    // the gantries stop short of the suction hoses beside them (the nearer at x -0.82)
    for z in [0.55, -2.55] {
        let zs = float_repr(z);
        fbox!(
            F,
            format!("gantry{zs}"),
            1.0,
            0.04,
            0.08,
            (-0.2, 2.97, z),
            m.iron
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("gantry_post{zs}{s}"),
                0.05,
                0.12,
                0.06,
                (-0.2 + s as f64 * 0.45, 2.9, z),
                m.iron
            );
        }
    }
    fbox!(
        F,
        "ladder_rest",
        0.7,
        0.07,
        0.08,
        (-0.2, 2.94, 2.25),
        m.iron
    );
    ladder("ladder", -0.2, 3.0, 2.45, -2.86, 0.56, m.ladder, m, true);
    ladder("ladder2", -0.2, 3.09, 2.25, -2.6, 0.46, m.ladder, m, false);
    ladder("roof_ladder", 0.78, 2.9, 0.4, -2.4, 0.34, m.ladder, m, true);
    for (k, x) in [-0.82, -0.98].into_iter().enumerate() {
        let r = 0.065;
        fsweep!(
            F,
            format!("suction{k}"),
            &[(x, FE_ROOF + r + 0.01, 0.45), (x, FE_ROOF + r + 0.01, -2.7)],
            r,
            m.rubber,
            verts = low(10, 6)
        );
        if !is_low() {
            for j in 0..6 {
                fcyl!(
                    F,
                    format!("suction_rib{k}{j}"),
                    r + 0.006,
                    0.03,
                    (x, FE_ROOF + r + 0.01, 0.1 - j as f64 * 0.5),
                    m.rubber,
                    rot = (90, 0, 0),
                    verts = 8
                );
            }
            for z in [0.47, -2.72] {
                fcyl!(
                    F,
                    format!("suction_end{k}{}", float_repr(z)),
                    r + 0.012,
                    0.06,
                    (x, FE_ROOF + r + 0.01, z),
                    m.brass,
                    rot = (90, 0, 0),
                    verts = 10
                );
            }
        }
    }
    for s in [-1, 1] {
        crate::eased(fcyl!(
            F,
            format!("beacon{s}"),
            0.07,
            0.12,
            (s as f64 * 1.0, FE_ROOF + 0.06, FE_BACK + 0.12),
            m.blue_lens,
            verts = low(14, 8)
        ));
    }
    join_all(if is_low() {
        "fireEngine_far"
    } else {
        "fireEngine"
    });
    settle_all();
    meshes()
}

/// A ladder lying on the roof along the depth from z0 to z1, its stiles' bottom at height `y`, `w` wide.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn ladder(
    name: &str,
    x: f64,
    y: f64,
    z0: f64,
    z1: f64,
    w: f64,
    material: Mat,
    _m: &ENGINE,
    rungs: bool,
) {
    for s in [-1, 1] {
        fbox!(
            F,
            format!("{name}_stile{s}"),
            0.04,
            0.08,
            z0 - z1,
            (x + s as f64 * w / 2.0, y + 0.04, (z0 + z1) / 2.0),
            material,
            bevel = low(0.006, 0.0),
            bevel_segments = 1
        );
    }
    if rungs {
        let n = artkit::py::int((z0 - z1) / low(0.28, 0.56));
        for k in 0..n + 1 {
            fcyl!(
                F,
                format!("{name}_rung{k}"),
                0.015,
                w - 0.04,
                (
                    x,
                    y + 0.04,
                    z1 + 0.1 + k as f64 * (z0 - z1 - 0.2) / n as f64
                ),
                material,
                rot = (0, 90, 0),
                verts = low(6, 4)
            );
        }
    }
}
