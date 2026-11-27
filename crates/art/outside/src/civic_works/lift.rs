//! The car lift: 3x5, a two-post lift with a hatchback up on its arms, its wheels on. The posts stand either side
//! of the middle (game x +-1.32), the car along the depth, its nose to the front, its sills 1.27 m up on the arms'
//! four pads.

use super::common::{F, arc, own_palette, tyre};
use artkit::far::{is_low, low};
use artkit::json::float_repr;
use artkit::math::{Axis, P3, atan2, degrees, radians};
use artkit::parts::{join, join_all, new_parts, settle_all};
use artkit::place::Facing;
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{cube, fbox, fcyl, fextrude, kit_tube, loft, revolve, ring};

own_palette! { LIFT:
    body = ("#2F5F9E", gloss),        // the lift: its posts, carriages and arms
    car = ("#C24A3A", gloss),         // the car's paint
    alloy = ("#C9CDD1", metal),       // its wheels' rims
    go = ("#3FAE5A", gloss),          // the lift's up button
}

pub const CL_POST: f64 = 1.32; // the posts' middles, game x
pub const CL_POST_H: f64 = 2.75; // their tops
pub const CL_ARMS: f64 = 1.15; // the arms' height (their pads' tops 0.12 higher, where the sills rest)
pub const CAR_Y: f64 = 1.1; // the car's own ground: its sills 0.17 over it
pub const CAR_HALF: f64 = 0.86; // half its body's width
pub const CAR_FA: f64 = 1.27; // its front axle's depth
pub const CAR_RA: f64 = -1.25; // its rear axle's depth
pub const CAR_WR: f64 = 0.31; // its wheels' radius
pub const CAR_BELT: f64 = 0.94; // its waist line, over its own ground

// The hatchback's stations, rear to front: (depth, half width, underside, top), game metres over the car's own
// ground (`CAR_Y`). Each is one rounded section of a loft, so the body is smooth and cheap.
const BODY: [(f64, f64, f64, f64); 8] = [
    (-1.97, 0.78, 0.24, 0.86),
    (-1.92, 0.84, 0.19, 0.93),
    (-1.6, 0.86, 0.17, 0.95),
    (0.0, 0.86, 0.17, 0.94),
    (1.0, 0.86, 0.17, 0.93),
    (1.5, 0.85, 0.17, 0.85),
    (1.86, 0.82, 0.2, 0.7),
    (1.97, 0.76, 0.27, 0.58),
];
// The glasshouse's, over the waist line: (depth, half width, top).
const HOUSE: [(f64, f64, f64); 8] = [
    (0.98, 0.76, 0.96),
    (0.6, 0.73, 1.2),
    (0.3, 0.68, 1.4),
    (-0.5, 0.66, 1.45),
    (-1.45, 0.64, 1.44),
    (-1.75, 0.62, 1.36),
    (-1.88, 0.6, 1.12),
    (-1.9, 0.58, 1.0),
];
const LOFT_SMOOTH: f64 = 80.0; // shaded smooth round every turn under this (degrees): no hard edge to bevel
const CYL_BEVEL: f64 = 0.004; // the chamfer on a cylinder's rims (a pixel at the play camera)
const SECTION_VERTS: usize = 16; // points round each section
const BODY_SQUARE: f64 = 4.0; // how square the body's sections are (2 an ellipse)
const HOUSE_SQUARE: f64 = 3.0; // the glasshouse's, rounder
const ROOF_SQUARE: f64 = 2.0; // the roof's: an ellipse
// the roof's stations: (depth, half width, depth of section): the top of a tall ellipse standing in the glass, so
// only a smooth cap shows over it; its ends shrunk, no wide flat cap
const ROOF: [(f64, f64, f64); 6] = [
    (0.42, 0.04, 0.02),
    (0.36, 0.4, 0.12),
    (0.2, 0.58, 0.22),
    (-1.0, 0.6, 0.25),
    (-1.5, 0.55, 0.2),
    (-1.58, 0.04, 0.02),
];
const ROOF_TOP: f64 = 1.475; // the roof panel's top
const ARCH_R: f64 = 0.36; // the empty wheel arches' radius
const ARCH_CUT: f64 = 0.17; // where an arch opens at the sill

/// One loft section: a rounded loop at `z`, `hw` either side of the middle, from `y0` to `y1` (game).
fn section(z: f64, hw: f64, (y0, y1): (f64, f64), square: f64) -> Vec<P3> {
    ring!(hw, (y1 - y0) / 2.0, verts = SECTION_VERTS, square = square)
        .iter()
        .map(|p| F.at(p.x, (y0 + y1) / 2.0 + p.y, z))
        .collect()
}

/// The painted shell and its glasshouse: the body and the roof lofted smooth, the glass tinted under it.
fn car_shell(m: &LIFT) {
    let y = CAR_Y;
    let body: Vec<_> = BODY
        .iter()
        .map(|&(z, hw, lo, hi)| section(z, hw, (y + lo, y + hi), BODY_SQUARE))
        .collect();
    loft!("car_body", &body, material = m.car, angle = LOFT_SMOOTH);
    let house: Vec<_> = HOUSE
        .iter()
        .map(|&(z, hw, top)| section(z, hw, (y + CAR_BELT - 0.04, y + top), HOUSE_SQUARE))
        .collect();
    loft!(
        "car_glass",
        &house,
        material = m.tinted,
        angle = LOFT_SMOOTH
    );
    let roof: Vec<_> = ROOF
        .iter()
        .map(|&(z, hw, t)| section(z, hw, (y + ROOF_TOP - t, y + ROOF_TOP), ROOF_SQUARE))
        .collect();
    loft!("car_roof", &roof, material = m.car, angle = LOFT_SMOOTH);
    for s in [-1, 1] {
        fbox!(
            F,
            format!("car_mirror{s}"),
            0.11,
            0.09,
            0.12,
            (s as f64 * 0.88, y + 1.0, 0.85),
            m.car,
            bevel = low(0.02, 0.0),
            bevel_segments = 1
        );
    }
}

/// What shows in the empty arches and along the sides: a dark arch on each flank, and in it the brake disc, hub and
/// caliper, a strut with its spring; the doors' seams and handles.
fn car_running_gear(m: &LIFT) {
    let y = CAR_Y;
    for s in [-1, 1] {
        let sf = s as f64;
        if !is_low() {
            for z in [0.98, -0.17, -1.22] {
                fbox!(
                    F,
                    format!("car_seam{s}{}", float_repr(z)),
                    0.004,
                    0.62,
                    0.008,
                    (sf * (CAR_HALF + 0.001), y + 0.6, z),
                    m.dark
                );
            }
            for z in [0.0, -1.1] {
                fbox!(
                    F,
                    format!("car_handle{s}{}", float_repr(z)),
                    0.012,
                    0.025,
                    0.13,
                    (sf * (CAR_HALF + 0.004), y + 0.84, z),
                    m.chrome
                );
            }
        }
        for zc in [CAR_FA, CAR_RA] {
            let zcs = float_repr(zc);
            // the arch: a flat dark D on the flank, its round top the wheel's
            let arch: Vec<(f64, f64)> = arc(zc, CAR_WR, ARCH_R, ARCH_CUT, low(14, 8))
                .into_iter()
                .map(|(z, yy)| (z, yy + y))
                .collect();
            fextrude!(
                F,
                format!("arch{s}{zcs}"),
                &arch,
                0.03,
                (sf * (CAR_HALF - 0.005), 0, 0),
                m.plastic,
                plane = Facing::Side,
                bevel = low(CYL_BEVEL, 0.0),
                bevel_segments = 1
            );
        }
    }
}

/// Its front and back: lamps, grille, plates (white at the front, yellow at the back), and under it an exhaust,
/// silencer and tank.
fn car_ends_and_underside(m: &LIFT) {
    let y = CAR_Y;
    for s in [-1, 1] {
        let sf = s as f64;
        artkit::bevel::eased(fextrude!(
            F,
            format!("headlamp{s}"),
            &[(-0.2, -0.03), (0.15, -0.05), (0.2, 0.04), (-0.18, 0.05)],
            0.03,
            (sf * 0.6, y + 0.63, 1.94),
            m.lamp,
            rot = (-25, 0, 0)
        ));
        fbox!(
            F,
            format!("taillamp{s}"),
            0.24,
            0.12,
            0.03,
            (sf * 0.66, y + 0.84, -1.94),
            m.red_lens,
            bevel = low(0.01, 0.0),
            bevel_segments = 1
        );
    }
    fbox!(
        F,
        "car_grille",
        0.9,
        0.12,
        0.02,
        (0, y + 0.45, 1.975),
        m.dark
    );
    fbox!(
        F,
        "plate_front",
        0.52,
        0.11,
        0.012,
        (0, y + 0.3, 1.976),
        m.white
    );
    fbox!(
        F,
        "plate_back",
        0.52,
        0.11,
        0.012,
        (0, y + 0.5, -1.975),
        m.yellow
    );
    fbox!(
        F,
        "car_underbody",
        1.5,
        0.03,
        3.3,
        (0, y + 0.18, -0.02),
        m.dark
    );
    // the exhaust under it, out at the back on the left
    artkit::bevel::eased(kit_tube!(
        F,
        "exhaust",
        &[
            (0.2, y + 0.17, 1.5),
            (0.2, y + 0.15, 0.6),
            (0.35, y + 0.15, -0.6),
            (0.35, y + 0.15, -1.7),
            (0.45, y + 0.2, -1.98)
        ],
        0.025,
        m.steel,
        round = low(2, 1)
    ));
    fcyl!(
        F,
        "silencer",
        0.1,
        0.5,
        (0.35, y + 0.18, -1.35),
        m.steel,
        rot = (90, 0, 0),
        verts = low(12, 6),
    );
    fbox!(
        F,
        "fuel_tank",
        0.8,
        0.12,
        0.5,
        (-0.2, y + 0.22, -0.75),
        m.iron
    );
}

/// The hatchback, lifted: a smooth lofted body and roof over a glasshouse of tinted glass, dark arches on its
/// flanks with the brakes and springs in them, lamps, plates, mirrors, an exhaust under it.
pub fn car(m: &LIFT) {
    car_shell(m);
    car_running_gear(m);
    car_ends_and_underside(m);
}

/// A car wheel off its car: the tyre on a silver alloy rim, dished, and a centre
/// cap, its face to the front, its middle at the game origin. One object, placed by its caller.
pub fn alloy(name: &str, m: &LIFT) -> Obj {
    let parts = new_parts(|| {
        tyre(
            &format!("{name}_tyre"),
            0,
            0,
            0,
            CAR_WR,
            0.2,
            m.rubber,
            None,
            Some(low(20, 10)),
            Axis::Z,
        );
        artkit::bevel::eased(revolve!(
            format!("{name}_rim"),
            &[
                (0.0, 0.06),
                (0.17, 0.065),
                (0.2, 0.09),
                (0.205, 0.095),
                (0.205, -0.09),
                (0.0, -0.07)
            ],
            at = F.at(0, 0, 0),
            material = m.alloy,
            rot = (90, 0, 0),
            segments = low(20, 10)
        ));
        fcyl!(
            F,
            format!("{name}_cap"),
            0.04,
            0.02,
            (0, 0, 0.1),
            m.chrome,
            rot = (90, 0, 0),
            verts = low(12, 6),
            bevel = low(CYL_BEVEL, 0.0),
            bevel_segments = 1
        );
    });
    join(&parts, name, Some((0, 0, 0).into()))
}

/// One post at game x s * CL_POST: a channel on a foot plate, its carriage on the inner face at the arms'
/// height, the hydraulic ram up its outer side, a cap; the controls on the left one (s -1).
pub fn lift_post(s: i32, m: &LIFT) {
    let sf = s as f64;
    let x = sf * CL_POST;
    fbox!(
        F,
        format!("post{s}"),
        0.24,
        CL_POST_H - 0.015,
        0.3,
        (x, (CL_POST_H + 0.015) / 2.0, 0),
        m.body,
        bevel = low(0.012, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        format!("post_slot{s}"),
        0.004,
        CL_POST_H - 0.4,
        0.08,
        (x - sf * 0.121, CL_POST_H / 2.0, 0),
        m.dark
    );
    fbox!(
        F,
        format!("foot{s}"),
        0.3,
        0.015,
        0.5,
        (x, 0.0075, 0),
        m.iron
    );
    for dz in [-0.2, 0.2] {
        for dx in [-0.1, 0.1] {
            fcyl!(
                F,
                format!("anchor{s}{}{}", float_repr(dz), float_repr(dx)),
                0.016,
                0.012,
                (x + dx, 0.021, dz),
                m.chrome,
                verts = 6
            );
        }
    }
    fbox!(
        F,
        format!("cap{s}"),
        0.28,
        0.06,
        0.34,
        (x, CL_POST_H - 0.03, 0),
        m.iron,
        bevel = low(0.01, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        format!("carriage{s}"),
        0.1,
        0.55,
        0.26,
        (x - sf * 0.17, CL_ARMS + 0.12, 0),
        m.body,
        bevel = low(0.01, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        format!("ram{s}"),
        0.04,
        CL_POST_H - 0.3,
        (x + sf * 0.0, (CL_POST_H - 0.3) / 2.0 + 0.05, -0.19),
        m.steel,
        verts = low(12, 6),
        bevel = low(CYL_BEVEL, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        format!("ram_rod{s}"),
        0.022,
        0.25,
        (x, CL_POST_H - 0.15, -0.19),
        m.chrome,
        verts = low(8, 6)
    );
    if s < 0 {
        fbox!(
            F,
            "control_box",
            0.18,
            0.26,
            0.12,
            (x, 1.25, 0.21),
            m.white,
            bevel = low(0.01, 0.0),
            bevel_segments = 1
        );
        for (k, (y, mat)) in [(1.31, m.go), (1.21, m.red_lens)].into_iter().enumerate() {
            fcyl!(
                F,
                format!("button{k}"),
                0.025,
                0.02,
                (x, y, 0.275),
                mat,
                rot = (90, 0, 0),
                verts = low(12, 6),
            );
        }
        kit_tube!(
            F,
            "cable",
            &[
                (x, 1.12, 0.21),
                (x + 0.03, 0.8, 0.2),
                (x + 0.02, 0.5, 0.19),
                (x, 0.02, 0.18)
            ],
            0.01,
            m.plastic,
            round = low(3, 1)
        );
    }
}

/// An arm from the carriage of post s, to the front (t +1) or back (t -1), its pad under the car's jacking point.
pub fn lift_arm(s: i32, t: i32, m: &LIFT) {
    let (sf, tf) = (s as f64, t as f64);
    let (x0, z0) = (sf * (CL_POST - 0.2), tf * 0.1);
    let (x1, z1) = (sf * 0.64, tf * 0.86);
    let (dx, dz) = (x1 - x0, z1 - z0);
    let l = py::hypot(dx, dz);
    let yaw = degrees(atan2(-dz, -dx));
    cube!(
        format!("arm{s}{t}"),
        (l * 0.6, 0.12, 0.09),
        at = F.at(x0 + dx * 0.3, CL_ARMS, z0 + dz * 0.3),
        material = m.body,
        rot = (0, 0, yaw),
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    cube!(
        format!("arm_in{s}{t}"),
        (l * 0.55, 0.09, 0.07),
        at = F.at(x0 + dx * 0.72, CL_ARMS, z0 + dz * 0.72),
        material = m.steel,
        rot = (0, 0, yaw)
    );
    fcyl!(
        F,
        format!("pivot{s}{t}"),
        0.05,
        0.12,
        (x0, CL_ARMS, z0),
        m.iron,
        verts = low(12, 6),
        bevel = low(CYL_BEVEL, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        format!("pad_stem{s}{t}"),
        0.03,
        0.08,
        (x1, CL_ARMS + 0.07, z1),
        m.chrome,
        verts = low(8, 6)
    );
    fcyl!(
        F,
        format!("pad{s}{t}"),
        0.07,
        0.035,
        (x1, CL_ARMS + 0.1, z1),
        m.rubber,
        verts = low(12, 8),
    );
}

pub fn car_lift() -> Vec<Obj> {
    let m = &LIFT;
    for s in [-1, 1] {
        lift_post(s, m);
        for t in [-1, 1] {
            lift_arm(s, t, m);
        }
    }
    // the floor plate between the posts over the hoses and cables: a low ramp of chequer plate
    fextrude!(
        F,
        "floor_plate",
        &[(-0.17, 0.0), (0.17, 0.0), (0.12, 0.03), (-0.12, 0.03)],
        2.0 * CL_POST - 0.44,
        (0, 0, 0),
        m.steel,
        plane = Facing::Side,
        bevel = low(CYL_BEVEL, 0.0),
        bevel_segments = 1
    );
    car(m);
    // the wheels on the car, hanging under it, their faces out to the sides
    for (k, (sx, z)) in [(-1.0, CAR_FA), (1.0, CAR_FA), (-1.0, CAR_RA), (1.0, CAR_RA)]
        .into_iter()
        .enumerate()
    {
        let w = alloy(&format!("wheel{k}"), m);
        w.set_rotation_euler((radians(0.0), 0.0, radians(sx * 90.0)));
        w.set_location(F.at(sx * (CAR_HALF - 0.06), CAR_Y + CAR_WR, z));
    }
    join_all(if is_low() { "carLift_far" } else { "carLift" });
    settle_all();
    meshes()
}
