//! The little tractor: big lugged rear wheels under swept fenders, a lofted hood with its grille and lamps, a seat
//! and steering wheel under a canopy on two bent roll frames, front weights and a three-point hitch. Its far
//! version is its own build without the small parts.

use super::common::{F, M};
use artkit::forms::rounded_rect;
use artkit::math::{P3, TAU, cos, radians, sin};
use artkit::parts::{join_all, settle};
use artkit::scene::Obj;
use artkit::{
    cube, cylinder, fbox, fcyl, fextrude, fsphere, fsweep, kit_fillet, kit_tube, loft, revolve,
    ring, rod, soft, sweep, torus,
};

// --- the tractor -----------------------------------------------------------------------------------------
// A compact tractor's track: 1.58 m over the rear tyres (it was 1.98, a utility tractor's).
const REAR_R: f64 = 0.725;
const REAR_W: f64 = 0.38;
const REAR_X: f64 = 0.6;
const REAR_Z: f64 = -0.75;
const FRONT_R: f64 = 0.39;
const FRONT_W: f64 = 0.26;
const FRONT_X: f64 = 0.5;
const FRONT_Z: f64 = 1.3;

/// A tyre with a yellow rim and dark hub, its axle along x, on the ground at (x, z); `lugs` chevron tread lugs, each
/// a bar slanting from a shoulder in to the middle, the sides taking turns.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn wheel(name: &str, x: f64, z: f64, r: f64, w: f64, lugs: usize, segments: usize, detail: bool) {
    let at = F.at(x, r, z);
    let hw = w / 2.0;
    let s = if x > 0.0 { 1.0 } else { -1.0 };
    if lugs == 0 {
        cylinder!(
            format!("{name}_tyre"),
            r,
            w,
            at = at,
            material = M.tyre,
            verts = segments,
            rot = (0, 90, 0)
        );
    } else {
        revolve!(
            format!("{name}_tyre"),
            &[
                (r * 0.6, -hw),
                (r - 0.06, -hw),
                (r - 0.015, -hw + 0.03),
                (r, -hw + 0.08),
                (r, hw - 0.08),
                (r - 0.015, hw - 0.03),
                (r - 0.06, hw),
                (r * 0.6, hw),
                (r * 0.6, -hw)
            ],
            at = at,
            material = M.tyre,
            segments = segments,
            rot = (0, 90, 0)
        );
    }
    cylinder!(
        format!("{name}_rim"),
        r * 0.6,
        w - 0.04,
        at = at,
        material = M.rims,
        verts = segments - 4,
        rot = (0, 90, 0)
    );
    cylinder!(
        format!("{name}_dish"),
        r * 0.42,
        w + 0.02,
        at = at,
        material = M.rims,
        verts = segments - 8,
        rot = (0, 90, 0)
    );
    cylinder!(
        format!("{name}_hub"),
        r * 0.16,
        w + 0.04,
        at = at,
        material = M.iron,
        verts = if detail { 12 } else { 8 },
        rot = (0, 90, 0)
    );
    if detail {
        // the rim's lip (a big wheel's), and the wheel nuts round the hub on the outside
        if r > 0.5 {
            torus!(
                format!("{name}_lip"),
                r * 0.6,
                0.012,
                at = F.at(x + s * (hw - 0.02), r, z),
                material = M.rims,
                rot = (0, 90, 0),
                major_segments = segments - 4,
                minor_segments = 4
            );
        }
        for k in 0..6 {
            let a = TAU * k as f64 / 6.0;
            cylinder!(
                format!("{name}_nut{k}"),
                0.016,
                0.03,
                at = F.at(
                    x + s * (hw + 0.005),
                    r + r * 0.25 * sin(a),
                    z + r * 0.25 * cos(a)
                ),
                material = M.iron,
                verts = 6,
                rot = (0, 90, 0)
            );
        }
    }
    let h = if r > 0.5 { 0.05 } else { 0.035 };
    for i in 0..lugs {
        let a = TAU * i as f64 / lugs as f64;
        let side = if i % 2 != 0 { 1.0 } else { -1.0 };
        let mut pts = Vec::new();
        for (u, da) in [(0.47, 0.0), (0.04, if r > 0.5 { 0.16 } else { 0.2 })] {
            // from the shoulder in, slanting forward
            let b = a + da * (r * 0.0 + 1.0) * 0.5;
            pts.push(F.at(
                x + side * u * w,
                r + (r - 0.012) * sin(b),
                z + (r - 0.012) * cos(b),
            ));
        }
        let mid = F.at(x, r + sin(a), z + cos(a));
        let up = P3 {
            x: mid.x - at.x,
            y: mid.y - at.y,
            z: mid.z - at.z,
        };
        sweep!(
            format!("{name}_lug{i}"),
            &pts,
            0.0,
            material = M.tyre,
            up = up,
            smooth = false,
            profile = [
                (h + 0.012, -0.035),
                (h + 0.012, 0.035),
                (0.0, 0.045),
                (0.0, -0.045)
            ]
        );
    }
}

/// The hood: a rounded box, lofted, its nose rounding in to the grille.
fn hood(detail: bool) {
    let n = if detail { 20 } else { 10 };
    let mut secs = Vec::new();
    for (z, rx, ry) in [
        (-0.1, 0.39, 0.25),
        (1.3, 0.39, 0.25),
        (1.52, 0.375, 0.235),
        (1.62, 0.34, 0.205),
    ] {
        secs.push(
            ring!(rx, ry, z = 0, verts = n, square = 6)
                .iter()
                .map(|p| F.at(p.x, 0.97 + p.y, z))
                .collect::<Vec<P3>>(),
        );
    }
    loft!("hood", &secs, material = M.tractor, angle = 50);
}

/// The tractor (`detail`), or its far version.
pub fn tractor_detail(detail: bool) -> Vec<Obj> {
    let green = M.tractor;
    for s in [-1, 1] {
        let sf = s as f64;
        wheel(
            &format!("rear{s}"),
            sf * REAR_X,
            REAR_Z,
            REAR_R,
            REAR_W,
            if detail { 24 } else { 0 },
            if detail { 28 } else { 16 },
            detail,
        );
        wheel(
            &format!("front{s}"),
            sf * FRONT_X,
            FRONT_Z,
            FRONT_R,
            FRONT_W,
            if detail { 18 } else { 0 },
            if detail { 28 } else { 14 },
            detail,
        );
        // the fender over each rear wheel: a rounded sheet with a rolled outer edge, and a step
        let step = if detail { 12 } else { 26 };
        let arc: Vec<(f64, f64, f64)> = (0..)
            .map(|k| 160 - k * step)
            .take_while(|&a| a > 4)
            .map(|a| {
                (
                    sf * 0.595,
                    REAR_R + 0.82 * sin(radians(a as f64)),
                    REAR_Z + 0.82 * cos(radians(a as f64)),
                )
            })
            .collect();
        fsweep!(
            F,
            format!("fender{s}"),
            &arc,
            0.0,
            green,
            profile = rounded_rect(0.39, 0.04, 0.015, if detail { 2 } else { 1 }),
            up = (1, 0, 0),
            angle = 50
        );
        if detail {
            let lip: Vec<(f64, f64, f64)> =
                arc.iter().map(|&(_, y, z)| (sf * 0.79, y, z)).collect();
            fsweep!(F, format!("fender_lip{s}"), &lip, 0.02, green, verts = 6);
        }
        fbox!(
            F,
            format!("step{s}"),
            0.3,
            0.04,
            0.3,
            (sf * 0.38, 0.62, 0.08),
            M.iron,
            bevel = if detail { 0.01 } else { 0.0 },
            bevel_segments = 1
        );
    }
    // chassis, axles, transmission housing under the cab, platform
    fbox!(
        F,
        "chassis",
        0.56,
        0.32,
        2.6,
        (0, 0.55, 0.2),
        M.iron,
        bevel = 0.02
    );
    fbox!(
        F,
        "front_axle",
        1.0,
        0.1,
        0.12,
        (0, FRONT_R, FRONT_Z),
        M.iron
    );
    fbox!(F, "rear_axle", 1.2, 0.16, 0.16, (0, REAR_R, REAR_Z), M.iron);
    fbox!(
        F,
        "housing",
        0.74,
        0.55,
        0.9,
        (0, 0.85, -0.7),
        green,
        bevel = 0.05,
        bevel_segments = 3
    );
    fbox!(F, "platform", 1.0, 0.05, 0.55, (0, 1.12, -0.35), M.iron);
    // the hood, its grille and lamps
    hood(detail);
    fbox!(F, "grille", 0.6, 0.38, 0.04, (0, 0.95, 1.61), M.iron);
    // the canopy on its two bent frames, the seat
    for sx in [-1, 1] {
        let x = sx as f64 * 0.6;
        kit_tube!(
            F,
            format!("rops{sx}"),
            &kit_fillet!(
                &[
                    (x, 1.13, -1.1),
                    (x, 2.37, -1.1),
                    (x, 2.37, 0.06),
                    (x, 1.13, 0.06)
                ],
                0.1,
                steps = if detail { 3 } else { 1 }
            ),
            0.032,
            green,
            verts = if detail { 8 } else { 4 },
            round = 1
        );
    }
    fbox!(
        F,
        "seat_back",
        0.46,
        0.34,
        0.08,
        (0, 1.55, -0.86),
        M.seat,
        bevel = if detail { 0.03 } else { 0.0 }
    );
    if !detail {
        // from afar: the lamps, exhaust and canopy, the front weights one block (as far out as they are, so it
        // stands where the tractor does); no grille bars, vents or levers
        cylinder!(
            "exhaust",
            0.045,
            0.75,
            at = F.at(0.24, 1.55, 1.22),
            material = M.iron,
            verts = 6
        );
        fbox!(F, "canopy", 1.32, 0.08, 1.38, (0, 2.42, -0.52), green);
        fbox!(F, "weights", 0.64, 0.22, 0.13, (0, 0.6, 1.69), M.iron);
        return settle(&[join_all("tractor_far")]);
    }
    soft!(
        F,
        "canopy",
        1.32,
        0.08,
        1.38,
        (0, 2.42, -0.52),
        green,
        round = 0.035,
        round_segments = 2
    );
    let holes: Vec<Vec<(f64, f64)>> = [-0.14, -0.07, 0.0, 0.07, 0.14]
        .iter()
        .map(|&dy| {
            rounded_rect(0.52, 0.042, 0.018, 2)
                .iter()
                .map(|&(u, v)| (u, v + dy))
                .collect()
        })
        .collect();
    fextrude!(
        F,
        "grille_frame",
        &rounded_rect(0.64, 0.42, 0.04, 2),
        0.03,
        (0, 0.95, 1.635),
        green,
        holes = holes
    );
    for s in [-1, 1] {
        let sf = s as f64;
        cylinder!(
            format!("lamp{s}"),
            0.055,
            0.04,
            at = F.at(sf * 0.26, 1.12, 1.63),
            material = M.lamp,
            verts = 12,
            rot = (90, 0, 0)
        );
        torus!(
            format!("lamp_bezel{s}"),
            0.058,
            0.012,
            at = F.at(sf * 0.26, 1.12, 1.65),
            material = green,
            rot = (90, 0, 0),
            major_segments = 12,
            minor_segments = 4
        );
        for i in 0..4 {
            fbox!(
                F,
                format!("vent{s}{i}"),
                0.02,
                0.05,
                0.25,
                (sf * 0.392, 1.0 + i as f64 * 0.07, 1.1),
                M.iron
            );
        }
    }
    // a fuel cap on the hood, and the front weights: five suitcase weights on their bracket, each with its handle
    fcyl!(
        F,
        "fuel_cap",
        0.045,
        0.03,
        (0.0, 1.225, 0.1),
        M.iron,
        verts = 10
    );
    fbox!(
        F,
        "weight_bracket",
        0.66,
        0.06,
        0.1,
        (0, 0.47, 1.64),
        M.iron
    );
    for k in 0..5 {
        let x = -0.26 + 0.13 * k as f64;
        fbox!(
            F,
            format!("weight{k}"),
            0.12,
            0.22,
            0.13,
            (x, 0.6, 1.69),
            M.iron,
            bevel = 0.015,
            bevel_segments = 1
        );
        fsweep!(
            F,
            format!("weight_handle{k}"),
            &kit_fillet!(
                &[
                    (x, 0.7, 1.68),
                    (x, 0.75, 1.68),
                    (x, 0.75, 1.73),
                    (x, 0.7, 1.73)
                ],
                0.015,
                steps = 1
            ),
            0.01,
            M.iron,
            verts = 4
        );
    }
    // the exhaust stack with its muffler and rain flap, and the air pre-cleaner
    cylinder!(
        "exhaust",
        0.045,
        0.75,
        at = F.at(0.24, 1.55, 1.22),
        material = M.iron,
        verts = 10
    );
    cylinder!(
        "muffler",
        0.065,
        0.3,
        at = F.at(0.24, 1.45, 1.22),
        material = M.iron,
        verts = 10
    );
    cube!(
        "exhaust_cap",
        (0.1, 0.1, 0.008),
        at = F.at(0.24, 1.94, 1.21),
        material = M.iron,
        rot = (18, 0, 0)
    );
    cylinder!(
        "precleaner",
        0.055,
        0.3,
        at = F.at(-0.24, 1.33, 1.22),
        material = M.iron,
        verts = 10
    );
    cylinder!(
        "precleaner_bowl",
        0.075,
        0.08,
        at = F.at(-0.24, 1.5, 1.22),
        material = M.lamp,
        verts = 12
    );
    // the dashboard with its gauges, the steering wheel on three spokes, the seat
    fbox!(
        F,
        "dash",
        0.5,
        0.3,
        0.14,
        (0, 1.32, -0.05),
        green,
        bevel = 0.03
    );
    for (k, x) in [-0.1, 0.1].into_iter().enumerate() {
        cylinder!(
            format!("gauge{k}"),
            0.035,
            0.02,
            at = F.at(x, 1.42, -0.125),
            material = M.seat,
            verts = 10,
            rot = (80, 0, 0)
        );
    }
    rod!(
        "column",
        F.at(0, 1.4, -0.08),
        F.at(0, 1.62, -0.28),
        0.025,
        material = M.iron,
        verts = 8
    );
    torus!(
        "steering",
        0.17,
        0.018,
        at = F.at(0, 1.64, -0.3),
        material = M.seat,
        rot = (-55, 0, 0),
        major_segments = 18,
        minor_segments = 6
    );
    let hub = F.at(0, 1.64, -0.3);
    for k in 0..3 {
        let a = TAU * k as f64 / 3.0 + 0.5;
        let tip = F.at(
            0.165 * cos(a),
            1.64 + 0.165 * sin(a) * cos(radians(55.0)),
            -0.3 - 0.165 * sin(a) * sin(radians(55.0)),
        );
        rod!(
            format!("spoke{k}"),
            hub,
            tip,
            0.01,
            material = M.seat,
            verts = 4
        );
    }
    cylinder!(
        "seat_post",
        0.04,
        0.2,
        at = F.at(0, 1.22, -0.62),
        material = M.iron,
        verts = 8
    );
    fbox!(
        F,
        "seat_pan",
        0.48,
        0.03,
        0.44,
        (0, 1.31, -0.62),
        M.iron,
        bevel = 0.01,
        bevel_segments = 1
    );
    soft!(
        F,
        "seat",
        0.46,
        0.08,
        0.42,
        (0, 1.36, -0.62),
        M.seat,
        round = 0.03,
        puff = 0.012,
        round_segments = 1,
        flat = 2
    );
    soft!(
        F,
        "seat_back_pad",
        0.42,
        0.3,
        0.05,
        (0, 1.56, -0.81),
        M.seat,
        round = 0.02,
        puff = 0.01,
        round_segments = 1,
        flat = 2
    );
    // tail lamps on the rear frame, the drawbar and the three-point hitch with its top link, and the PTO stub
    for s in [-1, 1] {
        fbox!(
            F,
            format!("tail{s}"),
            0.07,
            0.1,
            0.03,
            (s as f64 * 0.6, 1.5, -1.145),
            M.tail_lamp,
            bevel = 0.008,
            bevel_segments = 1
        );
    }
    fbox!(F, "drawbar", 0.14, 0.06, 0.32, (0, 0.5, -1.38), M.iron);
    for s in [-1, 1] {
        let sf = s as f64;
        rod!(
            format!("link{s}"),
            F.at(sf * 0.2, 0.6, -1.0),
            F.at(sf * 0.3, 0.42, -1.5),
            0.03,
            material = M.iron,
            verts = 6
        );
        fsphere!(
            F,
            format!("link_ball{s}"),
            0.04,
            (sf * 0.3, 0.42, -1.5),
            M.iron,
            segments = 8,
            rings = 5
        );
    }
    rod!(
        "top_link",
        F.at(0, 1.0, -1.12),
        F.at(0, 0.85, -1.48),
        0.025,
        material = M.iron,
        verts = 6
    );
    rod!(
        "pto",
        F.at(0, 0.62, -1.13),
        F.at(0, 0.62, -1.24),
        0.03,
        material = M.iron,
        verts = 8
    );
    settle(&[join_all("tractor")])
}

pub fn tractor() -> Vec<Obj> {
    tractor_detail(true)
}

/// The tractor from afar: plain cylinder tyres without lugs, plain fenders and roll frames, the weights one block, no
/// grille, vents or levers.
pub fn tractor_far() -> Vec<Obj> {
    tractor_detail(false)
}
