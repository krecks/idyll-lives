//! KD1: the kids' bed and the race car bed.

use super::common::{P, mattress, moon, soft_pillow, star, teddy, turn_down};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::kit;
use artkit::kit::fillet;
use artkit::materials::Mat;
use artkit::math::{PI, TAU, cos, radians, sin};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, revolve, sphere};

/// The mattress top (furniture.json beds.kidsBed).
const KIDS_TOP: f64 = 0.44;

/// A turned bun foot `h` tall, `r` round.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn bun(name: &str, x: f64, z: f64, h: f64, r: f64, material: Mat, segments: usize) -> Obj {
    kit::turned!(
        F,
        name,
        x,
        0.0,
        z,
        &[
            (0.0, 0.0),
            (r * 0.8, 0.0),
            (r, h * 0.2),
            (r * 1.02, h * 0.45),
            (r * 0.82, h * 0.7),
            (r * 0.9, h * 0.85),
            (r * 0.9, h),
            (0.0, h),
        ],
        material,
        segments = segments
    )
}

pub fn kids_bed() -> Vec<Obj> {
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            bun(
                &format!("foot{sx}{sz}"),
                sx as f64 * 0.4,
                sz as f64 * 0.86,
                0.07,
                0.032,
                M.kids_frame,
                10,
            );
        }
    }
    // the base: a box with a rounded rim round its top, two drawers under the bed on its near side
    fbox!(
        F,
        "base",
        0.94,
        0.24,
        1.88,
        (0, 0.18, 0.01),
        M.kids_frame,
        bevel = 0.02,
        bevel_segments = 2
    );
    let rim = rounded_rect(0.93, 1.85, 0.06, 3);
    let path: Vec<P> = rim.iter().map(|&(u, v)| (u, 0.3, 0.01 + v)).collect();
    fsweep!(
        F,
        "rim",
        &path,
        0,
        M.kids_frame,
        profile = rounded_rect(0.034, 0.03, 0.012, 1),
        up = (0, 0, 1),
        closed = true,
        angle = 60
    );
    for (k, zc) in [-0.42, 0.44].into_iter().enumerate() {
        fbox!(
            F,
            format!("drawer{k}"),
            0.012,
            0.17,
            0.8,
            (-0.474, 0.175, zc),
            M.kids_drawer,
            bevel = 0.006,
            bevel_segments = 1
        );
        fextrude!(
            F,
            format!("drawer_grip{k}"),
            &rounded_rect(0.13, 0.035, 0.017, 3),
            0.004,
            (-0.4785, 0.22, zc),
            M.kids_frame,
            plane = Facing::Side
        );
    }
    mattress(
        "mattress", 0.86, 0.16, 1.8, KIDS_TOP, 0.02, 0.0, None, 0.006, 0.006, 0.03, 3,
    );
    // a headboard with an arched top, a moon and two stars cut through it, and bunting draped across it; its back
    // on the wall's face (depth -0.93)
    let mut outline = vec![(-0.48, 0.07), (0.48, 0.07)];
    outline.extend((0..17).map(|i| {
        (
            0.48 * cos(PI * i as f64 / 16.0),
            0.5 + 0.264 * sin(PI * i as f64 / 16.0),
        )
    }));
    let holes = vec![
        moon(0.0, 0.675, 0.048, 10),
        star(-0.21, 0.655, 0.036, 0.45, 5, 90.0),
        star(0.21, 0.655, 0.036, 0.45, 5, 90.0),
    ];
    fextrude!(
        F,
        "headboard",
        &outline,
        0.04,
        (0, 0, -0.91),
        M.kids_frame,
        holes = holes,
        bevel = 0.008,
        bevel_segments = 2
    );
    let sag: Vec<P> = (0..11)
        .map(|i| {
            (
                -0.43 + 0.86 * i as f64 / 10.0,
                0.6 - 0.035 * sin(PI * i as f64 / 10.0),
                -0.886,
            )
        })
        .collect();
    fsweep!(
        F,
        "bunting",
        &sag,
        0.003,
        M.kids_bedding,
        verts = 5,
        resolution = 2
    );
    let flags = [
        M.pennant_a,
        M.pennant_b,
        M.pennant_c,
        M.pennant_a,
        M.pennant_b,
    ];
    for (k, m) in flags.into_iter().enumerate() {
        let x = -0.3 + 0.15 * k as f64;
        let y = 0.6 - 0.035 * sin(PI * (x + 0.43) / 0.86);
        fextrude!(
            F,
            format!("pennant{k}"),
            &[(-0.042, 0.0), (0.042, 0.0), (0.0, -0.075)],
            0.003,
            (x, y + 0.002, -0.886),
            m,
            rot = (0.0, 0.0, (k as f64 - 2.0) * 2.5)
        );
    }
    // a guard rail along the right side, bent from one tube, two spindles in it
    let pts = fillet(
        &[
            (0.475, 0.3, -0.62),
            (0.475, 0.675, -0.62),
            (0.475, 0.675, 0.22),
            (0.475, 0.3, 0.22),
        ],
        0.06,
        3,
    );
    fsweep!(F, "rail", &pts, 0.015, M.kids_frame, verts = 10);
    for z in [-0.34, -0.06] {
        fsweep!(
            F,
            format!("rail_spindle{}", r(z)),
            &[(0.475, 0.3, z), (0.475, 0.665, z)],
            0.01,
            M.kids_frame,
            verts = 8
        );
    }
    soft_pillow(
        "pillow", 0.58, 0.12, 0.3, 0, KIDS_TOP, -0.72, M.sheet, 8.0, 1.0,
    );
    teddy(-0.33, KIDS_TOP - 0.004, -0.79, M.teddy_small, M.eye, 0.06);
    turn_down(KIDS_TOP, 0.435, -0.5, 0.93, 0.12, M.kids_bedding)
}

// --- KD1: the race car bed -----------------------------------------------------------------------------------
/// The mattress top (the renderer's default bed height).
const CAR_TOP: f64 = 0.5;
/// The cockpit's sides, either side of the mattress.
const CAR_SILL: f64 = 0.45;

fn car_arc(cz: f64, cy: f64, r: f64, a0: f64, a1: f64, n: usize) -> Vec<(f64, f64)> {
    (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f64 / n as f64;
            (cz + r * cos(radians(a)), cy + r * sin(radians(a)))
        })
        .collect()
}

/// The car's side seen from the right (game depth, height): a low nose rounding up to the bonnet, the sill along
/// the cockpit, the tail rising behind, and an arch over each wheel.
fn car_side() -> Vec<(f64, f64)> {
    let mut out = vec![(-0.93, 0.6), (-0.93, 0.09)];
    out.extend_from_slice(&car_arc(-0.91, 0.09, 0.02, 180.0, 270.0, 2)[1..]);
    for wz in [-0.6, 0.6] {
        out.extend(car_arc(wz, 0.18, 0.215, 211.6, -31.6, 12));
    }
    out.extend(car_arc(0.9, 0.14, 0.07, 270.0, 360.0, 4));
    out.extend(car_arc(0.89, 0.46, 0.08, 0.0, 90.0, 4));
    out.extend([(0.66, 0.542), (0.63, 0.5), (0.6, CAR_SILL + 0.02)]);
    out.extend([(-0.74, CAR_SILL + 0.02), (-0.79, CAR_SILL + 0.06)]);
    out.extend(car_arc(-0.86, 0.56, 0.05, 30.0, 90.0, 3));
    out
}

/// A wheel at (x, z): a rounded tyre and a dished hub with a cap, its axle across.
fn wheel(name: &str, x: f64, z: f64, side: i32) {
    let tyre = vec![
        (0.105, -0.04),
        (0.16, -0.04),
        (0.174, -0.035),
        (0.18, -0.02),
        (0.18, 0.02),
        (0.174, 0.035),
        (0.16, 0.04),
        (0.105, 0.04),
    ];
    let hub = vec![
        (0.0, 0.0425),
        (0.028, 0.0425),
        (0.034, 0.036),
        (0.08, 0.032),
        (0.098, 0.036),
        (0.106, 0.03),
        (0.106, -0.03),
        (0.0, -0.03),
    ];
    for (part, prof, m) in [("tyre", tyre, M.dark), ("hub", hub, M.hub)] {
        revolve!(
            format!("{part}{name}"),
            &prof,
            at = F.at(x, 0.18, z),
            material = m,
            segments = 16,
            rot = (0, -90 * side, 0)
        );
    }
}

pub fn kids_bed_car() -> Vec<Obj> {
    // the body: the two sides (arched over the wheels), the bonnet and tail between them, the deck under the
    // mattress; the tail end's back on the wall's face (depth -0.93)
    let side = car_side();
    for s in [-1, 1] {
        fextrude!(
            F,
            format!("side{s}"),
            &side,
            0.06,
            (s as f64 * 0.44, 0, 0),
            M.car,
            plane = Facing::Side,
            bevel = 0.02,
            bevel_segments = 1
        );
    }
    let mut nose = vec![(0.6, 0.12), (0.92, 0.12), (0.97, 0.17)];
    nose.extend(car_arc(0.89, 0.46, 0.08, 0.0, 90.0, 4));
    nose.extend([(0.66, 0.542), (0.6, 0.5)]);
    fextrude!(
        F,
        "bonnet",
        &nose,
        0.84,
        (0, 0, 0),
        M.car,
        plane = Facing::Side,
        bevel = 0.03,
        bevel_segments = 2
    );
    let mut tail = vec![(-0.93, 0.32), (-0.79, 0.32), (-0.79, CAR_SILL + 0.06)];
    tail.extend(car_arc(-0.86, 0.56, 0.05, 30.0, 90.0, 3));
    tail.push((-0.93, 0.61));
    fextrude!(
        F,
        "tail",
        &tail,
        0.84,
        (0, 0, 0),
        M.car,
        plane = Facing::Side,
        bevel = 0.02,
        bevel_segments = 2
    );
    fbox!(
        F,
        "deck",
        0.84,
        0.26,
        1.42,
        (0, 0.21, -0.08),
        M.car,
        bevel = 0.01,
        bevel_segments = 1
    );
    mattress(
        "mattress", 0.8, 0.16, 1.4, CAR_TOP, -0.08, 0.0, None, 0.006, 0.006, 0.03, 3,
    );
    // the bonnet: two stripes over it, a grille with bars, headlights in their rims
    for s in [-1, 1] {
        let sf = s as f64;
        fextrude!(
            F,
            format!("stripe{s}"),
            &rounded_rect(0.08, 0.22, 0.01, 1),
            0.004,
            (sf * 0.09, 0.543, 0.77),
            M.car_stripe,
            plane = Facing::Top
        );
        sphere!(
            format!("headlight{s}"),
            0.06,
            at = F.at(sf * 0.3, 0.45, 0.965),
            material = M.headlight,
            scale = (1, 0.35, 0.8),
            segments = 12,
            rings = 6
        );
        let rim: Vec<P> = (0..12)
            .map(|i| {
                (
                    sf * 0.3 + 0.064 * cos(TAU * i as f64 / 12.0),
                    0.45 + 0.05 * sin(TAU * i as f64 / 12.0),
                    0.97,
                )
            })
            .collect();
        fsweep!(
            F,
            format!("headlight_rim{s}"),
            &rim,
            0.007,
            M.hub,
            verts = 5,
            closed = true
        );
        // a number roundel on each side, a 7 on it
        fcyl!(
            F,
            format!("roundel{s}"),
            0.07,
            0.006,
            (sf * 0.473, 0.3, 0.0),
            M.car_stripe,
            rot = (0, 90, 0),
            verts = 20
        );
        let seven = [
            (-0.03, 0.035),
            (0.032, 0.035),
            (0.032, 0.022),
            (-0.004, -0.04),
            (-0.02, -0.04),
            (0.014, 0.022),
            (-0.03, 0.022),
        ];
        let number: Vec<(f64, f64)> = seven.iter().map(|&(u, v)| (u * sf, v)).collect();
        fextrude!(
            F,
            format!("number{s}"),
            &number,
            0.004,
            (sf * 0.477, 0.3, 0.0),
            M.dark,
            plane = Facing::Side
        );
        // a racing stripe down each side, under the sill
        fextrude!(
            F,
            format!("side_stripe{s}"),
            &rounded_rect(1.0, 0.03, 0.012, 2),
            0.003,
            (sf * 0.4715, 0.4, -0.02),
            M.car_stripe,
            plane = Facing::Side
        );
    }
    fextrude!(
        F,
        "grille",
        &rounded_rect(0.38, 0.11, 0.03, 3),
        0.012,
        (0, 0.3, 0.97),
        M.dark,
        plane = Facing::Front
    );
    for k in 0..3 {
        fbox!(
            F,
            format!("grille_bar{k}"),
            0.34,
            0.01,
            0.008,
            (0, 0.27 + 0.03 * k as f64, 0.978),
            M.hub,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    // the tail's spoiler: a wing on two struts between end plates
    for s in [-1, 1] {
        let sf = s as f64;
        fextrude!(
            F,
            format!("strut{s}"),
            &[(-0.885, 0.6), (-0.865, 0.6), (-0.885, 0.72), (-0.905, 0.72)],
            0.03,
            (sf * 0.3, 0, 0),
            M.dark,
            plane = Facing::Side,
            bevel = 0.004,
            bevel_segments = 1
        );
        fextrude!(
            F,
            format!("endplate{s}"),
            &rounded_rect(0.09, 0.09, 0.02, 2),
            0.018,
            (sf * 0.4, 0.73, -0.885),
            M.dark,
            plane = Facing::Side,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    // an aerofoil, thick at its leading edge (u up, v back from it)
    let wing = vec![
        (0.0, 0.0),
        (0.012, 0.004),
        (0.018, 0.02),
        (0.016, 0.045),
        (0.008, 0.077),
        (0.002, 0.077),
        (-0.004, 0.05),
        (-0.006, 0.02),
        (-0.004, 0.004),
    ];
    fsweep!(
        F,
        "wing",
        &[(-0.39, 0.725, -0.852), (0.39, 0.725, -0.852)],
        0,
        M.dark,
        profile = wing,
        up = (0, 0, 1),
        angle = 50
    );
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            wheel(&format!("{sx}{sz}"), sx as f64 * 0.45, sz as f64 * 0.6, sx);
        }
    }
    soft_pillow(
        "pillow", 0.5, 0.12, 0.26, 0, CAR_TOP, -0.64, M.sheet, 12.0, 2.0,
    );
    turn_down(CAR_TOP, 0.4, -0.42, 0.66, 0.1, M.car_bedding)
}
