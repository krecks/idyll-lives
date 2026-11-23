//! The car lift: 3x5, a two-post lift with a hatchback up on its arms, its wheels off. The posts stand either side
//! of the middle (game x +-1.32), the car along the depth, its nose to the front, its sills 1.27 m up on the arms'
//! four pads; its wheels lie and lean on the floor round it.

use super::common::{F, arc, own_palette, tyre};
use artkit::far::{is_low, low};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{Axis, M4, TAU, atan2, cos, degrees, radians, sin};
use artkit::parts::{join, join_all, new_parts, settle_all};
use artkit::place::Facing;
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{cube, fbox, fcyl, fextrude, fsweep, kit_tube, revolve};

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
#[allow(dead_code)] // (the script's own: nothing reads it)
pub const CAR_ROOF: f64 = 1.47; // its roof, over its own ground

/// Draws `objs` in towards the middle the higher they are, from none at game height y0 to `k` of their x at y1:
/// a car's glasshouse leaning in to its roof (tumblehome).
pub fn tapered(objs: &[Obj], y0: f64, y1: f64, k: f64) {
    for &o in objs {
        let mw: M4 = o.matrix_world();
        let inv = mw.inverted().expect("a part's matrix inverts");
        o.edit(|part| {
            for v in part.mesh.verts.iter_mut() {
                let mut p = mw * *v;
                let f = ((p.z as f64 - y0) / (y1 - y0)).clamp(0.0, 1.0);
                p.x = (p.x as f64 * (1.0 - k * f)) as f32;
                *v = inv * p;
            }
        });
    }
}

/// The hatchback, lifted: a body drawn from the side, its arches empty, a glasshouse of tinted glass leaning in
/// under a roof in the car's colour, pillars, lamps, plates, mirrors, the brakes and springs in its empty arches and
/// an exhaust under it.
pub fn car(m: &LIFT) {
    let y = CAR_Y;
    let paint = m.car;
    let mut lower = vec![(-1.93, y + 0.17)];
    lower.extend(
        arc(CAR_RA, CAR_WR, 0.36, 0.17, low(12, 6))
            .into_iter()
            .map(|(z, yy)| (z, yy + y)),
    );
    lower.extend(
        arc(CAR_FA, CAR_WR, 0.36, 0.17, low(12, 6))
            .into_iter()
            .map(|(z, yy)| (z, yy + y)),
    );
    lower.extend([
        (1.9, y + 0.17),
        (1.96, y + 0.28),
        (1.98, y + 0.52),
        (1.93, y + 0.65),
        (1.62, y + 0.76),
        (1.0, y + 0.93),
        (-1.86, y + CAR_BELT + 0.01),
        (-1.95, y + 0.9),
        (-1.97, y + 0.6),
        (-1.95, y + 0.3),
    ]);
    fextrude!(
        F,
        "car_body",
        &lower,
        2.0 * CAR_HALF,
        (0, 0, 0),
        paint,
        plane = Facing::Side,
        bevel = low(0.05, 0.0),
        bevel_segments = low(3, 1)
    );
    let house = new_parts(|| {
        let top = [
            (0.96, y + 0.92),
            (0.3, y + 1.42),
            (-1.45, y + 1.44),
            (-1.8, y + 1.32),
            (-1.9, y + 0.92),
        ];
        fextrude!(
            F,
            "car_glass",
            &top,
            1.6,
            (0, 0, 0),
            m.tinted,
            plane = Facing::Side,
            bevel = low(0.02, 0.0),
            bevel_segments = 1
        );
        fextrude!(
            F,
            "car_roof",
            &[
                (0.36, y + 1.4),
                (0.3, y + 1.47),
                (-1.45, y + 1.475),
                (-1.56, y + 1.44),
                (-1.5, y + 1.4)
            ],
            1.64,
            (0, 0, 0),
            paint,
            plane = Facing::Side,
            bevel = low(0.025, 0.0),
            bevel_segments = low(2, 1)
        );
        fbox!(
            F,
            "car_spoiler",
            1.3,
            0.03,
            0.18,
            (0, y + 1.47, -1.53),
            paint,
            bevel = low(0.012, 0.0),
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let x = s as f64 * 0.805;
            fsweep!(
                F,
                format!("a_pillar{s}"),
                &[(x, y + 0.92, 0.97), (x, y + 1.43, 0.3)],
                0,
                paint,
                profile = rounded_rect(0.07, 0.03, 0.0, 1),
                up = (0, 0, 1)
            );
            fbox!(
                F,
                format!("b_pillar{s}"),
                0.02,
                0.5,
                0.09,
                (x, y + 1.17, -0.18),
                paint
            );
            fextrude!(
                F,
                format!("c_pillar{s}"),
                &[
                    (-1.25, y + 0.93),
                    (-1.42, y + 1.43),
                    (-1.82, y + 1.31),
                    (-1.9, y + 0.93)
                ],
                0.014,
                (x, 0, 0),
                paint,
                plane = Facing::Side
            );
            fbox!(
                F,
                format!("sill_trim{s}"),
                0.014,
                0.025,
                1.0,
                (x, y + 0.935, 0.4),
                m.chrome
            );
        }
    });
    tapered(&house, y + 0.92, y + 1.48, 0.17);
    for s in [-1, 1] {
        let sf = s as f64;
        let xs = sf * (CAR_HALF + 0.004);
        if !is_low() {
            for z in [0.98, -0.17, -1.22] {
                // the doors' seams
                fbox!(
                    F,
                    format!("car_seam{s}{}", float_repr(z)),
                    0.004,
                    0.62,
                    0.008,
                    (xs, y + 0.6, z),
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
                    (xs + sf * 0.004, y + 0.84, z),
                    m.chrome
                );
            }
        }
        fbox!(
            F,
            format!("car_mirror{s}"),
            0.11,
            0.09,
            0.12,
            (sf * 0.88, y + 1.0, 0.85),
            paint,
            bevel = low(0.02, 0.0),
            bevel_segments = 1
        );
        // the empty arches: black liners, and the brakes, hubs and springs left on the axles
        for zc in [CAR_FA, CAR_RA] {
            let zcs = float_repr(zc);
            let pts: Vec<(f64, f64, f64)> = arc(zc, CAR_WR, 0.345, 0.2, low(10, 5))
                .into_iter()
                .map(|(z, yy)| (sf * 0.74, yy + y, z))
                .collect();
            fsweep!(
                F,
                format!("liner{s}{zcs}"),
                &pts,
                0,
                m.plastic,
                profile = [
                    (-0.004, -0.12),
                    (0.004, -0.12),
                    (0.004, 0.12),
                    (-0.004, 0.12)
                ],
                up = (0, 0, 1)
            );
            fcyl!(
                F,
                format!("disc{s}{zcs}"),
                0.14,
                0.022,
                (sf * 0.76, y + CAR_WR, zc),
                m.steel,
                rot = (0, 90, 0),
                verts = low(20, 8)
            );
            fcyl!(
                F,
                format!("hub{s}{zcs}"),
                0.065,
                0.06,
                (sf * 0.77, y + CAR_WR, zc),
                m.iron,
                rot = (0, 90, 0),
                verts = low(12, 6)
            );
            fbox!(
                F,
                format!("caliper{s}{zcs}"),
                0.06,
                0.1,
                0.09,
                (sf * 0.765, y + CAR_WR + 0.06, zc - 0.11),
                m.red_lens,
                bevel = low(0.015, 0.0),
                bevel_segments = 1
            );
            if !is_low() {
                for k in 0..4 {
                    let a = TAU * k as f64 / 4.0 + 0.4;
                    fcyl!(
                        F,
                        format!("stud{s}{zcs}{k}"),
                        0.009,
                        0.04,
                        (sf * 0.8, y + CAR_WR + 0.045 * sin(a), zc + 0.045 * cos(a)),
                        m.chrome,
                        rot = (0, 90, 0),
                        verts = 6
                    );
                }
                fcyl!(
                    F,
                    format!("strut{s}{zcs}"),
                    0.03,
                    0.4,
                    (sf * 0.64, y + CAR_WR + 0.24, zc),
                    m.iron,
                    verts = 8
                );
                fcyl!(
                    F,
                    format!("spring{s}{zcs}"),
                    0.055,
                    0.22,
                    (sf * 0.64, y + CAR_WR + 0.26, zc),
                    m.iron,
                    verts = 10
                );
            }
            fbox!(
                F,
                format!("arm_lower{s}{zcs}"),
                0.32,
                0.04,
                0.06,
                (sf * 0.58, y + CAR_WR - 0.02, zc),
                m.iron
            );
        }
    }
    // its front and back: lamps, grille, plates (white at the front, yellow at the back), bumpers' lower lips
    for s in [-1, 1] {
        let sf = s as f64;
        fextrude!(
            F,
            format!("headlamp{s}"),
            &[(-0.2, -0.03), (0.15, -0.05), (0.2, 0.04), (-0.18, 0.05)],
            0.03,
            (sf * 0.6, y + 0.63, 1.94),
            m.lamp,
            rot = (-25, 0, 0)
        );
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
        m.dark,
        bevel = low(0.01, 0.0),
        bevel_segments = 1
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
    if !is_low() {
        for (k, (yy, z)) in [(0.3, 1.983), (0.5, -1.983)].into_iter().enumerate() {
            fbox!(
                F,
                format!("plate_text{k}"),
                0.4,
                0.045,
                0.004,
                (0, y + yy, z),
                m.ink
            );
        }
    }
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
    kit_tube!(
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
    );
    fcyl!(
        F,
        "silencer",
        0.1,
        0.5,
        (0.35, y + 0.18, -1.35),
        m.steel,
        rot = (90, 0, 0),
        verts = low(14, 6)
    );
    fbox!(
        F,
        "fuel_tank",
        0.8,
        0.12,
        0.5,
        (-0.2, y + 0.22, -0.75),
        m.iron,
        bevel = low(0.03, 0.0),
        bevel_segments = 1
    );
}

/// A car wheel off its car: the tyre on a silver alloy rim, five dark windows between its spokes and a centre
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
            if is_low() { None } else { Some(2) },
            Some(low(32, 12)),
            Axis::Z,
        );
        revolve!(
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
            segments = low(24, 10)
        );
        if !is_low() {
            for k in 0..5 {
                let a = TAU * k as f64 / 5.0;
                let mut win: Vec<(f64, f64)> = [-0.32, 0.32]
                    .iter()
                    .map(|&d| (0.055 * cos(a + d), 0.055 * sin(a + d)))
                    .collect();
                win.extend(
                    [0.36, 0.0, -0.36]
                        .iter()
                        .map(|&d| (0.165 * cos(a + d), 0.165 * sin(a + d))),
                );
                fextrude!(
                    F,
                    format!("{name}_window{k}"),
                    &win,
                    0.006,
                    (0, 0, 0.0995),
                    m.dark,
                    plane = Facing::Front
                );
            }
        }
        fcyl!(
            F,
            format!("{name}_cap"),
            0.04,
            0.02,
            (0, 0, 0.1),
            m.chrome,
            rot = (90, 0, 0),
            verts = low(12, 6)
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
        verts = low(12, 6)
    );
    fcyl!(
        F,
        format!("ram_rod{s}"),
        0.022,
        0.25,
        (x, CL_POST_H - 0.15, -0.19),
        m.chrome,
        verts = low(10, 6)
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
                verts = low(12, 6)
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
        verts = low(12, 6)
    );
    fcyl!(
        F,
        format!("pad_stem{s}{t}"),
        0.03,
        0.08,
        (x1, CL_ARMS + 0.07, z1),
        m.chrome,
        verts = low(10, 6)
    );
    fcyl!(
        F,
        format!("pad{s}{t}"),
        0.07,
        0.035,
        (x1, CL_ARMS + 0.1, z1),
        m.rubber,
        verts = low(16, 8)
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
        plane = Facing::Side
    );
    car(m);
    // the wheels: two stacked under its nose, one lying under its tail, one leaning on the right post
    for (k, (x, y, z, rx, rz)) in [
        (-0.45, 0.1, 1.75, 90.0, 0.0),
        (-0.42, 0.3, 1.73, 90.0, 0.0),
        (0.42, 0.1, -1.72, 90.0, 0.0),
        (CL_POST - 0.24, 0.315, 0.42, -12.0, 0.0),
    ]
    .into_iter()
    .enumerate()
    {
        let w = alloy(&format!("wheel{k}"), m);
        w.set_rotation_euler((radians(rx), 0.0, radians(rz)));
        w.set_location(F.at(x, y, z));
    }
    join_all(if is_low() { "carLift_far" } else { "carLift" });
    settle_all();
    meshes()
}
