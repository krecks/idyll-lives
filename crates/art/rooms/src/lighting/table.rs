//! LT4: the table lamps.

use super::common::{
    Drum, F, Lathe, M, bulb, check_at, drum, lathe, loft_at, near_light, ring_at, socket, trough,
    turn,
};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{cos, radians, sin, sqrt};
use artkit::parts::{join_all, new_parts};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, rod, settle, torus};

const GOURD: [(f64, f64); 10] = [
    (0.0, -0.5),
    (0.32, -0.5),
    (0.46, -0.3),
    (0.5, -0.05),
    (0.4, 0.25),
    (0.24, 0.42),
    (0.26, 0.5),
    (0.2, 0.5),
    (0.18, 0.4),
    (0.0, 0.38),
];
const JAR: [(f64, f64); 8] = [
    (0.0, -0.5),
    (0.3, -0.5),
    (0.48, -0.25),
    (0.5, 0.05),
    (0.38, 0.38),
    (0.15, 0.45),
    (0.12, 0.5),
    (0.0, 0.5),
];

/// An oak side table (a top with rounded corners, an apron, tapered legs, a shelf) with a ceramic gourd lamp on a
/// brass foot ring, a socket, and a drum shade bound at its edges on a wire fitter.
pub fn table_lamp() -> Vec<Obj> {
    new_parts(|| {
        fextrude!(
            F,
            "top",
            &rounded_rect(0.66, 0.42, 0.03, 3),
            0.03,
            (0, 0.485, 0),
            M.oak,
            plane = Facing::Top,
            bevel = 0.008,
            bevel_segments = 2
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("apron_z{s}"),
                0.52,
                0.06,
                0.018,
                (0, 0.44, sf * 0.155),
                M.oak,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("apron_x{s}"),
                0.018,
                0.06,
                0.28,
                (sf * 0.275, 0.44, 0),
                M.oak,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fextrude!(
            F,
            "shelf",
            &rounded_rect(0.58, 0.34, 0.02, 2),
            0.02,
            (0, 0.14, 0),
            M.oak,
            plane = Facing::Top,
            bevel = 0.005,
            bevel_segments = 1
        );
        for x in [-0.28, 0.28] {
            for z in [-0.16, 0.16] {
                loft_at(
                    &format!("leg{}{}", float_repr(x), float_repr(z)),
                    x,
                    z,
                    &[(0.0, 0.013, 0.013, 8.0), (0.47, 0.021, 0.021, 8.0)],
                    M.oak,
                    12,
                );
            }
        }
        turn(
            "foot",
            &[
                (0.0, 0.0),
                (0.052, 0.0),
                (0.054, 0.005),
                (0.05, 0.01),
                (0.0, 0.01),
            ],
            (-0.12, 0.5, 0),
            M.brass,
            24,
        );
        lathe(
            "base",
            0.17,
            0.19,
            (-0.12, 0.605, 0),
            &GOURD,
            M.ceramic,
            Lathe {
                segments: 28,
                ..Lathe::default()
            },
        );
        turn(
            "neck",
            &[
                (0.0, 0.0),
                (0.024, 0.0),
                (0.024, 0.008),
                (0.014, 0.014),
                (0.0, 0.014),
            ],
            (-0.12, 0.695, 0),
            M.brass,
            14,
        );
        socket("socket", 0.02, 0.05, (-0.12, 0.71, 0), M.brass, 12);
        bulb(
            "tableLamp",
            0.035,
            (-0.12, 0.81, 0),
            None,
            (1.0, 1.0, 1.15),
            14,
            8,
        );
        drum(
            "shade",
            0.15,
            0.1275,
            0.2,
            (-0.12, 0.72, 0),
            M.shade,
            Drum {
                segments: 36,
                rims: Some(M.shade),
                spider: Some(M.brass),
                ..Drum::default()
            },
        );
        fcyl!(
            F,
            "diffuser",
            0.143,
            0.004,
            (-0.12, 0.723, 0),
            M.bulb,
            verts = 24
        );
    });
    join_all("tableLamp");
    settle!();
    check_at("tableLamp")
}

/// A blue ginger jar on a turned brass plinth, a brass cap on its neck, a stem and socket, a harp and finial over a
/// drum shade bound at its edges.
pub fn lamp_table_round() -> Vec<Obj> {
    new_parts(|| {
        turn(
            "plinth",
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.072, 0.006),
                (0.068, 0.012),
                (0.058, 0.018),
                (0.0, 0.018),
            ],
            (0, 0, 0),
            M.brass,
            28,
        );
        lathe(
            "jar",
            0.2,
            0.245,
            (0, 0.1405, 0),
            &JAR,
            M.jar,
            Lathe {
                segments: 32,
                ..Lathe::default()
            },
        );
        for (k, (y, r)) in [(0.06, 0.084), (0.2, 0.084)].into_iter().enumerate() {
            ring_at(&format!("jar_band{k}"), r, (0, y, 0), 0.003, M.jar, 32, 4);
        }
        turn(
            "cap",
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.03, 0.008),
                (0.022, 0.016),
                (0.012, 0.02),
                (0.0, 0.02),
            ],
            (0, 0.258, 0),
            M.brass,
            16,
        );
        kit::tube!(
            F,
            "stem",
            &[(0, 0.27, 0), (0, 0.35, 0)],
            0.008,
            M.brass,
            verts = 8,
            round = 1
        );
        socket("socket", 0.018, 0.045, (0, 0.335, 0), M.brass, 12);
        bulb(
            "lampTableRound",
            0.03,
            (0, 0.4, 0),
            None,
            (1.0, 1.0, 1.15),
            14,
            8,
        );
        kit::tube!(
            F,
            "harp",
            &[
                (-0.02, 0.34, 0),
                (-0.05, 0.44, 0),
                (-0.02, 0.545, 0),
                (0.0, 0.55, 0),
                (0.02, 0.545, 0),
                (0.05, 0.44, 0),
                (0.02, 0.34, 0)
            ],
            0.0028,
            M.brass,
            verts = 6,
            round = 3
        );
        turn(
            "finial",
            &[
                (0.0, 0.0),
                (0.005, 0.0),
                (0.01, 0.006),
                (0.013, 0.014),
                (0.01, 0.021),
                (0.004, 0.026),
                (0.0, 0.027),
            ],
            (0, 0.55, 0),
            M.brass,
            12,
        );
        drum(
            "shade",
            0.17,
            0.1445,
            0.22,
            (0, 0.33, 0),
            M.shade,
            Drum {
                segments: 36,
                rims: Some(M.shade),
                ..Drum::default()
            },
        );
        fcyl!(
            F,
            "diffuser",
            0.162,
            0.004,
            (0, 0.34, 0),
            M.bulb,
            verts = 24
        );
    });
    join_all("lampTableRound");
    settle!();
    check_at("lampTableRound")
}

/// A square stone block with softened edges under a black cap plate, a stem with a switch and a socket, and a tapered
/// square shade bound in black at its edges.
pub fn lamp_table_square() -> Vec<Obj> {
    new_parts(|| {
        fextrude!(
            F,
            "base",
            &rounded_rect(0.12, 0.12, 0.01, 2),
            0.24,
            (0, 0.12, 0),
            M.stone,
            plane = Facing::Top,
            bevel = 0.005,
            bevel_segments = 2
        );
        fextrude!(
            F,
            "base_cap",
            &rounded_rect(0.1, 0.1, 0.008, 2),
            0.016,
            (0, 0.248, 0),
            M.iron,
            plane = Facing::Top,
            bevel = 0.003,
            bevel_segments = 1
        );
        kit::tube!(
            F,
            "stem",
            &[(0, 0.255, 0), (0, 0.345, 0)],
            0.009,
            M.iron,
            verts = 8,
            round = 1
        );
        fbox!(
            F,
            "switch",
            0.022,
            0.03,
            0.016,
            (0, 0.29, 0.012),
            M.iron,
            bevel = 0.005,
            bevel_segments = 1
        );
        socket("socket", 0.018, 0.045, (0, 0.335, 0), M.iron, 12);
        bulb(
            "lampTableSquare",
            0.03,
            (0, 0.4, 0),
            None,
            (1.0, 1.0, 1.15),
            14,
            8,
        );
        drum(
            "shade",
            0.15,
            0.12,
            0.2,
            (0, 0.34, 0),
            M.square_shade,
            Drum {
                segments: 4,
                rot: (0.0, 0.0, 45.0),
                ..Drum::default()
            },
        );
        for (name, y, r) in [("rim_low", 0.34, 0.15), ("rim_high", 0.53, 0.12)] {
            let h = r / sqrt(2.0);
            kit::moulding!(
                F,
                name,
                &[(-h, -h), (h, -h), (h, h), (-h, h)],
                &[(-0.0015, 0.0), (0.006, 0.0), (0.006, 0.01), (-0.0015, 0.01)],
                M.iron,
                at = y,
                plane = Facing::Top
            );
        }
        fbox!(F, "diffuser", 0.2, 0.004, 0.2, (0, 0.35, 0), M.bulb);
    });
    join_all("lampTableSquare");
    settle!();
    check_at("lampTableSquare")
}

/// A banker's lamp: a stepped brass base with rounded ends, an upright curving forward into the shade's cradle, a
/// green half-round shade (white inside) held by a brass band and bound at its ends, a beaded pull chain.
pub fn lamp_banker() -> Vec<Obj> {
    new_parts(|| {
        loft_at(
            "base",
            0,
            0,
            &[
                (0.0, 0.11, 0.07, 5.0),
                (0.008, 0.11, 0.07, 5.0),
                (0.013, 0.104, 0.064, 5.0),
                (0.018, 0.092, 0.054, 5.0),
                (0.022, 0.088, 0.05, 5.0),
            ],
            M.brass,
            24,
        );
        turn(
            "foot",
            &[
                (0.0, 0.0),
                (0.024, 0.0),
                (0.024, 0.006),
                (0.016, 0.014),
                (0.012, 0.02),
                (0.0, 0.02),
            ],
            (-0.06, 0.02, 0),
            M.brass,
            14,
        );
        kit::tube!(
            F,
            "upright",
            &[
                (-0.06, 0.03, 0.0),
                (-0.06, 0.28, 0.0),
                (-0.06, 0.3, 0.006),
                (-0.06, 0.305, 0.022)
            ],
            0.008,
            M.brass,
            verts = 8,
            round = 3
        );
        trough("shade", 0.06, 0.3, 0.004, F.at(0, 0.33, 0.03), M.banker);
        trough(
            "shade_liner",
            0.0555,
            0.292,
            0.003,
            F.at(0, 0.33, 0.03),
            M.liner,
        );
        // the band over the shade where the upright holds it, and a bound edge at each end
        for (name, x, w) in [
            ("band", -0.06, 0.012),
            ("edge_l", -0.149, 0.004),
            ("edge_r", 0.149, 0.004),
        ] {
            let arc: Vec<(f64, f64, f64)> = (-22..204)
                .step_by(15)
                .map(|a| {
                    let a = radians(a as f64);
                    (x, 0.33 + 0.0605 * sin(a), 0.03 + 0.0605 * cos(a))
                })
                .collect();
            fsweep!(
                F,
                name,
                &arc,
                0,
                M.brass,
                profile = rounded_rect(w, 0.003, 0.0012, 1),
                up = (1, 0, 0),
                resolution = 2
            );
        }
        rod!(
            "lampBanker_bulb",
            F.at(-0.13, 0.327, 0.03),
            F.at(0.13, 0.327, 0.03),
            0.009,
            material = M.bulb,
            verts = 10
        );
        near_light("lampBanker", 0.327, None);
        kit::tube!(
            F,
            "chain",
            &[(0.08, 0.33, 0.07), (0.08, 0.245, 0.07)],
            0.0012,
            M.brass,
            verts = 4,
            round = 1
        );
        for k in 0..5 {
            fsphere!(
                F,
                format!("bead{k}"),
                0.0025,
                (0.08, 0.32 - k as f64 * 0.016, 0.07),
                M.brass,
                segments = 6,
                rings = 4
            );
        }
        turn(
            "pull",
            &[
                (0.0, 0.0),
                (0.003, 0.0),
                (0.007, 0.006),
                (0.006, 0.014),
                (0.002, 0.018),
                (0.0, 0.018),
            ],
            (0.08, 0.228, 0.07),
            M.brass,
            10,
        );
    });
    join_all("lampBanker");
    settle!();
    check_at("lampBanker")
}

const MUSHROOM: [(f64, f64); 8] = [
    (0.5, -0.5),
    (0.46, 0.15),
    (0.3, 0.42),
    (0.0, 0.5),
    (0.0, 0.4),
    (0.28, 0.32),
    (0.44, 0.1),
    (0.47, -0.5),
];

/// A white porcelain foot rising into a slim stem that flares under a coral glass cap with a rolled lip.
pub fn lamp_mushroom() -> Vec<Obj> {
    new_parts(|| {
        turn(
            "base",
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.072, 0.006),
                (0.068, 0.014),
                (0.05, 0.02),
                (0.024, 0.026),
                (0.0, 0.026),
            ],
            (0, 0, 0),
            M.porcelain,
            32,
        );
        turn(
            "stem",
            &[
                (0.0, 0.0),
                (0.02, 0.0),
                (0.0145, 0.03),
                (0.0105, 0.09),
                (0.009, 0.15),
                (0.013, 0.18),
                (0.022, 0.19),
                (0.0, 0.19),
            ],
            (0, 0.02, 0),
            M.porcelain,
            16,
        );
        lathe(
            "cap",
            0.3,
            0.16,
            (0, 0.28, 0),
            &MUSHROOM,
            M.cap,
            Lathe {
                segments: 36,
                ..Lathe::default()
            },
        );
        ring_at("cap_lip", 0.146, (0, 0.202, 0), 0.004, M.cap, 36, 5);
        bulb(
            "lampMushroom",
            0.05,
            (0, 0.22, 0),
            None,
            (1.0, 1.0, 0.5),
            14,
            8,
        );
    });
    join_all("lampMushroom");
    settle!();
    check_at("lampMushroom")
}

/// A candle lantern: a black base, four posts and rails round the glass, a roof curving up to a knob and a carrying
/// ring, a latch; a candle with a drip down its side, burning inside.
// (0.318 is the roof's top in metres, not 1/π.)
#[allow(clippy::approx_constant)]
pub fn lamp_lantern() -> Vec<Obj> {
    new_parts(|| {
        fextrude!(
            F,
            "base",
            &rounded_rect(0.16, 0.16, 0.012, 2),
            0.02,
            (0, 0.01, 0),
            M.iron,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(F, "glass", 0.128, 0.22, 0.128, (0, 0.13, 0), M.glass);
        for x in [-0.07, 0.07] {
            for z in [-0.07, 0.07] {
                fbox!(
                    F,
                    format!("post{}{}", float_repr(x), float_repr(z)),
                    0.016,
                    0.22,
                    0.016,
                    (x, 0.13, z),
                    M.iron,
                    bevel = 0.003,
                    bevel_segments = 1
                );
            }
        }
        let frame = [
            (-0.075, -0.075),
            (0.075, -0.075),
            (0.075, 0.075),
            (-0.075, 0.075),
        ];
        let rail = [(0.0, 0.0), (0.012, 0.0), (0.012, 0.01), (0.0, 0.01)];
        kit::moulding!(
            F,
            "rail_low",
            &frame,
            &rail,
            M.iron,
            at = 0.02,
            plane = Facing::Top
        );
        let rim: Vec<(f64, f64)> = frame.iter().map(|&(x, z)| (x * 1.07, z * 1.07)).collect();
        kit::moulding!(
            F,
            "rim",
            &rim,
            &[(0.0, 0.0), (0.018, 0.0), (0.018, 0.012), (0.0, 0.012)],
            M.iron,
            at = 0.24,
            plane = Facing::Top
        );
        loft_at(
            "roof",
            0,
            0,
            &[
                (0.252, 0.074, 0.074, 12.0),
                (0.262, 0.066, 0.066, 10.0),
                (0.28, 0.046, 0.046, 8.0),
                (0.3, 0.026, 0.026, 6.0),
                (0.318, 0.012, 0.012, 4.0),
            ],
            M.iron,
            16,
        );
        turn(
            "knob",
            &[
                (0.0, 0.0),
                (0.014, 0.0),
                (0.014, 0.006),
                (0.008, 0.014),
                (0.0, 0.016),
            ],
            (0, 0.316, 0),
            M.iron,
            10,
        );
        torus!(
            "ring",
            0.025,
            0.004,
            at = F.at(0, 0.345, 0),
            material = M.iron,
            rot = (90, 0, 0),
            major_segments = 14,
            minor_segments = 4
        );
        fbox!(
            F,
            "latch",
            0.014,
            0.024,
            0.006,
            (0.064, 0.15, 0.076),
            M.iron,
            bevel = 0.002,
            bevel_segments = 1
        );
        turn(
            "candle",
            &[
                (0.0, 0.0),
                (0.025, 0.0),
                (0.025, 0.094),
                (0.022, 0.1),
                (0.004, 0.102),
                (0.0, 0.102),
            ],
            (0, 0.02, 0),
            M.candle,
            16,
        );
        kit::tube!(
            F,
            "drip",
            &[
                (0.025, 0.118, 0.004),
                (0.026, 0.09, 0.006),
                (0.0255, 0.06, 0.004)
            ],
            0.004,
            M.candle,
            verts = 5,
            round = 2,
            scale = [1.0, 0.8, 0.6]
        );
        kit::tube!(
            F,
            "wick",
            &[(0, 0.12, 0), (0, 0.13, 0)],
            0.0012,
            M.iron,
            verts = 4,
            round = 1
        );
        bulb(
            "lampLantern",
            0.011,
            (0, 0.14, 0),
            Some(M.flame),
            (1.0, 1.0, 1.7),
            10,
            6,
        );
    });
    join_all("lampLantern");
    settle!();
    check_at("lampLantern")
}
