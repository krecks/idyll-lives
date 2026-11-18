//! LT2, the wall lights, and LT3, the floor lamps.

use super::common::{
    Drum, F, Lathe, M, WALL, bulb, check_at, drum, lathe, ring_at, socket, turn, turned,
};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{TAU, cos, cosd, pow, sin, sind, sqrt};
use artkit::parts::{join_all, new_parts};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, loft, settle, torus};

// --- LT2: the wall lights ----------------------------------------------------------------------------------
/// A turned brass backplate, a gooseneck arm curving up into a candle cup, and a cone shade, rolled at its top and
/// bottom edges, on a wire fitter.
pub fn wall_sconce() -> Vec<Obj> {
    new_parts(|| {
        turned(
            "plate",
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.06, 0.006),
                (0.054, 0.012),
                (0.03, 0.018),
                (0.016, 0.022),
                (0.0, 0.022),
            ],
            (0, 1.7, WALL),
            M.brass,
            20,
            (90.0, 0.0, 0.0),
        );
        kit::tube!(
            F,
            "arm",
            &[
                (0, 1.7, WALL + 0.02),
                (0, 1.692, -0.33),
                (0, 1.7, -0.278),
                (0, 1.722, -0.26)
            ],
            0.0085,
            M.brass,
            verts = 8,
            round = 3
        );
        turn(
            "cup",
            &[
                (0.0, 0.0),
                (0.012, 0.0),
                (0.024, 0.008),
                (0.026, 0.018),
                (0.02, 0.022),
                (0.02, 0.03),
                (0.0, 0.03),
            ],
            (0, 1.72, -0.26),
            M.brass,
            14,
        );
        for a in [90, 210, 330] {
            let af = a as f64;
            kit::tube!(
                F,
                format!("spoke{a}"),
                &[
                    (0.02 * cosd(af), 1.745, -0.26 + 0.02 * sind(af)),
                    (0.087 * cosd(af), 1.745, -0.26 + 0.087 * sind(af))
                ],
                0.0025,
                M.brass,
                verts = 4,
                round = 1
            );
        }
        bulb(
            "wallSconce",
            0.03,
            (0, 1.775, -0.26),
            None,
            (1.0, 1.0, 1.15),
            10,
            6,
        );
        drum(
            "shade",
            0.1,
            0.075,
            0.18,
            (0, 1.71, -0.26),
            M.shade,
            Drum {
                segments: 24,
                rims: Some(M.shade),
                rim: 0.004,
                ..Drum::default()
            },
        );
    });
    join_all("wallSconce");
    settle!(centre = false, floor = false);
    check_at("wallSconce")
}

const ARM_SHADE: [(f64, f64); 9] = [
    (0.5, -0.5),
    (0.5, -0.47),
    (0.478, -0.44),
    (0.2, 0.4),
    (0.1, 0.5),
    (0.0, 0.5),
    (0.0, 0.42),
    (0.09, 0.42),
    (0.46, -0.5),
];

/// A black swing-arm lamp: a turned wall plate, a lower arm and an upper arm bent to the shade, brass knuckles with
/// thumb screws between them, and a tilted cone shade with a rolled rim and a switch on its crown.
pub fn wall_lamp_arm() -> Vec<Obj> {
    new_parts(|| {
        turned(
            "plate",
            &[
                (0.0, 0.0),
                (0.055, 0.0),
                (0.055, 0.006),
                (0.048, 0.014),
                (0.022, 0.02),
                (0.0, 0.02),
            ],
            (0, 1.55, WALL),
            M.iron,
            22,
            (90.0, 0.0, 0.0),
        );
        turned(
            "pivot",
            &[(0.0, 0.0), (0.016, 0.0), (0.016, 0.03), (0.0, 0.03)],
            (0, 1.55, WALL + 0.018),
            M.brass,
            12,
            (90.0, 0.0, 0.0),
        );
        kit::tube!(
            F,
            "arm_low",
            &[(0, 1.55, WALL + 0.04), (0, 1.58, -0.31), (0, 1.6, -0.25)],
            0.0085,
            M.iron,
            verts = 8
        );
        fsphere!(
            F,
            "knuckle",
            0.017,
            (0, 1.6, -0.25),
            M.brass,
            segments = 12,
            rings = 8
        );
        kit::tube!(
            F,
            "thumb_screw",
            &[(0.017, 1.6, -0.25), (0.034, 1.6, -0.25)],
            0.004,
            M.brass,
            verts = 6,
            round = 1
        );
        fcyl!(
            F,
            "thumb",
            0.009,
            0.006,
            (0.036, 1.6, -0.25),
            M.brass,
            rot = (0, 90, 0),
            verts = 10
        );
        kit::tube!(
            F,
            "arm_high",
            &[
                (0, 1.6, -0.25),
                (0, 1.7, -0.235),
                (0, 1.765, -0.2),
                (0, 1.775, -0.185)
            ],
            0.0075,
            M.iron,
            verts = 8
        );
        lathe(
            "shade",
            0.2,
            0.16,
            (0, 1.78, -0.18),
            &ARM_SHADE,
            M.shade_metal,
            Lathe {
                segments: 28,
                rot: (-35.0, 0.0, 0.0),
                top: true,
                liner: Some(M.liner),
            },
        );
        fcyl!(
            F,
            "switch",
            0.007,
            0.01,
            (0, 1.781, -0.176),
            M.brass,
            rot = (-35, 0, 0),
            verts = 10
        );
        bulb(
            "wallLampArm",
            0.032,
            (0, 1.69, -0.125),
            None,
            (1.0, 1.0, 1.15),
            12,
            6,
        );
    });
    join_all("wallLampArm");
    settle!(centre = false, floor = false);
    check_at("wallLampArm")
}

/// A bulkhead: an oval glass lens in a black frame on a rounded back plate, under a drip hood, behind a cage of bars
/// that arch over the lens, the plate screwed to the wall.
pub fn wall_light_outdoor() -> Vec<Obj> {
    new_parts(|| {
        fextrude!(
            F,
            "plate",
            &rounded_rect(0.2, 0.3, 0.03, 3),
            0.03,
            (0, 2.01, WALL + 0.015),
            M.iron,
            bevel = 0.005,
            bevel_segments = 1
        );
        for y in [1.885, 2.135] {
            fcyl!(
                F,
                format!("screw{}", float_repr(y)),
                0.006,
                0.004,
                (0, y, WALL + 0.031),
                M.iron,
                rot = (90, 0, 0),
                verts = 8
            );
        }
        torus!(
            "frame",
            1.0,
            0.11,
            at = F.at(0, 2.01, WALL + 0.03),
            material = M.iron,
            rot = (90, 0, 0),
            major_segments = 24,
            minor_segments = 5
        )
        .set_scale((0.074, 0.12, 0.12));
        let lens = fsphere!(
            F,
            "lens",
            1.0,
            (0, 2.01, WALL + 0.03),
            M.lens,
            scale = (0.075, 0.06, 0.125),
            segments = 16,
            rings = 8
        );
        // its back half, hidden in the plate, flattened onto the wall face
        lens.edit(|o| {
            for v in &mut o.mesh.verts {
                if v.y > 0.0 {
                    v.y *= 0.5;
                }
            }
        });
        bulb(
            "wallLightOutdoor",
            0.035,
            (0, 2.01, -0.38),
            None,
            (1.0, 0.8, 1.2),
            10,
            6,
        );
        // the drip hood: flat on the wall, rounding down over the lens at the front
        let hood = smooth_outline(
            &[
                (-0.055, 0.0125),
                (0.035, 0.0125),
                (0.055, -0.004),
                (0.055, -0.022),
                (0.046, -0.022),
                (0.044, -0.006),
                (0.03, -0.0125),
                (-0.055, -0.0125),
            ],
            1,
            true,
        );
        fextrude!(
            F,
            "hood",
            &hood,
            0.22,
            (0, 2.175, WALL + 0.055),
            M.iron,
            plane = Facing::Side
        );
        // the cage: bars from post to post arching forward over the lens
        for y in [1.94, 2.01, 2.08] {
            kit::tube!(
                F,
                format!("bar{}", float_repr(y)),
                &[(-0.085, y, -0.36), (0.0, y, -0.327), (0.085, y, -0.36)],
                0.006,
                M.iron,
                verts = 6,
                round = 3
            );
        }
        kit::tube!(
            F,
            "bar_v",
            &[(0, 2.12, -0.385), (0, 2.01, -0.327), (0, 1.9, -0.385)],
            0.006,
            M.iron,
            verts = 6,
            round = 3
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("post{s}"),
                0.012,
                0.18,
                0.06,
                (s as f64 * 0.085, 2.01, -0.37),
                M.iron,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
    });
    join_all("wallLightOutdoor");
    settle!(centre = false, floor = false);
    check_at("wallLightOutdoor")
}

// --- LT3: the floor lamps ----------------------------------------------------------------------------------
/// The classic floor lamp: a dark stepped foot, a pole with turned brass collars, a ribbed socket with its turn knob,
/// a brass harp bowing round the bulb to a finial, a drum shade with corded trims on a wire fitter, a pull chain.
pub fn lamp() -> Vec<Obj> {
    new_parts(|| {
        turn(
            "foot",
            &[
                (0.0, 0.0),
                (0.15, 0.0),
                (0.15, 0.008),
                (0.145, 0.016),
                (0.13, 0.026),
                (0.1, 0.034),
                (0.06, 0.05),
                (0.04, 0.062),
                (0.03, 0.066),
                (0.018, 0.07),
                (0.0, 0.07),
            ],
            (0, 0, 0),
            M.stand,
            32,
        );
        kit::tube!(
            F,
            "pole",
            &[(0, 0.06, 0), (0, 1.44, 0)],
            0.012,
            M.stand,
            verts = 10,
            round = 1
        );
        for y in [0.5, 1.0] {
            turn(
                &format!("collar{}", float_repr(y)),
                &[
                    (0.0, 0.0),
                    (0.016, 0.0),
                    (0.021, 0.006),
                    (0.021, 0.024),
                    (0.016, 0.03),
                    (0.0, 0.03),
                ],
                (0, y - 0.015, 0),
                M.brass,
                14,
            );
        }
        socket("socket", 0.025, 0.08, (0, 1.435, 0), M.brass, 16);
        kit::tube!(
            F,
            "knob_stem",
            &[(0.024, 1.48, 0), (0.042, 1.48, 0)],
            0.003,
            M.brass,
            verts = 6,
            round = 1
        );
        fcyl!(
            F,
            "knob",
            0.008,
            0.012,
            (0.046, 1.48, 0),
            M.brass,
            rot = (0, 90, 0),
            verts = 10
        );
        bulb("lamp", 0.035, (0, 1.54, 0), None, (1.0, 1.0, 1.25), 14, 8);
        fbox!(
            F,
            "harp_saddle",
            0.06,
            0.008,
            0.012,
            (0, 1.448, 0),
            M.brass,
            bevel = 0.003,
            bevel_segments = 1
        );
        kit::tube!(
            F,
            "harp",
            &[
                (-0.028, 1.45, 0),
                (-0.07, 1.6, 0),
                (-0.03, 1.76, 0),
                (0.0, 1.776, 0),
                (0.03, 1.76, 0),
                (0.07, 1.6, 0),
                (0.028, 1.45, 0)
            ],
            0.0035,
            M.brass,
            verts = 6,
            round = 3
        );
        turn(
            "finial",
            &[
                (0.0, 0.0),
                (0.006, 0.0),
                (0.012, 0.006),
                (0.017, 0.016),
                (0.013, 0.026),
                (0.005, 0.031),
                (0.0, 0.032),
            ],
            (0, 1.776, 0),
            M.brass,
            14,
        );
        drum(
            "shade",
            0.22,
            0.158,
            0.34,
            (0, 1.43, 0),
            M.lamp_shade,
            Drum {
                segments: 40,
                ..Drum::default()
            },
        );
        torus!(
            "trim_low",
            0.218,
            0.007,
            at = F.at(0, 1.436, 0),
            material = M.trim,
            major_segments = 40,
            minor_segments = 6
        );
        torus!(
            "trim_high",
            0.157,
            0.007,
            at = F.at(0, 1.764, 0),
            material = M.trim,
            major_segments = 32,
            minor_segments = 6
        );
        kit::tube!(
            F,
            "chain",
            &[(0.04, 1.47, 0), (0.04, 1.31, 0)],
            0.0018,
            M.brass,
            verts = 4,
            round = 1
        );
        turn(
            "pull",
            &[
                (0.0, 0.0),
                (0.004, 0.0),
                (0.008, 0.008),
                (0.006, 0.016),
                (0.002, 0.02),
                (0.0, 0.02),
            ],
            (0.04, 1.292, 0),
            M.brass,
            10,
        );
    });
    join_all("lamp");
    settle!();
    check_at("lamp")
}

/// A square black base with a turned boss, a stem with a coupling and a rocker switch, under a tall square shade
/// bound in black at its top and bottom, a diffuser at its foot and a wire fitter at its top.
pub fn floor_lamp_simple() -> Vec<Obj> {
    new_parts(|| {
        fextrude!(
            F,
            "base",
            &rounded_rect(0.28, 0.28, 0.03, 3),
            0.03,
            (0, 0.015, 0),
            M.iron,
            plane = Facing::Top,
            bevel = 0.006,
            bevel_segments = 2
        );
        turn(
            "boss",
            &[
                (0.0, 0.0),
                (0.034, 0.0),
                (0.034, 0.005),
                (0.02, 0.014),
                (0.014, 0.03),
                (0.0, 0.03),
            ],
            (0, 0.03, 0),
            M.iron,
            16,
        );
        kit::tube!(
            F,
            "stem",
            &[(0, 0.05, 0), (0, 0.87, 0)],
            0.011,
            M.iron,
            verts = 10,
            round = 1
        );
        turn(
            "coupling",
            &[(0.0, 0.0), (0.015, 0.0), (0.015, 0.03), (0.0, 0.03)],
            (0, 0.44, 0),
            M.iron,
            12,
        );
        fbox!(
            F,
            "switch",
            0.026,
            0.05,
            0.018,
            (0, 0.72, 0.014),
            M.iron,
            bevel = 0.006,
            bevel_segments = 2
        );
        drum(
            "shade",
            0.19,
            0.19,
            0.74,
            (0, 0.86, 0),
            M.shade,
            Drum {
                segments: 4,
                t: 0.008,
                rot: (0.0, 0.0, 45.0),
                ..Drum::default()
            },
        );
        let half = 0.19 / sqrt(2.0);
        let square = [(-half, -half), (half, -half), (half, half), (-half, half)];
        for (name, y) in [("rim_low", 0.86), ("rim_high", 1.6 - 0.014)] {
            kit::moulding!(
                F,
                name,
                &square,
                &[(-0.002, 0.0), (0.01, 0.0), (0.01, 0.014), (-0.002, 0.014)],
                M.iron,
                at = y,
                plane = Facing::Top
            );
        }
        fbox!(F, "diffuser", 0.255, 0.004, 0.255, (0, 0.866, 0), M.bulb);
        bulb(
            "floorLampSimple",
            0.045,
            (0, 1.45, 0),
            None,
            (1.0, 1.0, 1.2),
            14,
            8,
        );
        kit::tube!(
            F,
            "holder",
            &[(0, 0.87, 0), (0, 1.4, 0)],
            0.006,
            M.iron,
            verts = 6,
            round = 1
        );
        for a in [0, 90, 180, 270] {
            let af = a as f64;
            kit::tube!(
                F,
                format!("spoke{a}"),
                &[
                    (0.012 * cosd(af), 1.58, 0.012 * sind(af)),
                    (0.13 * cosd(af), 1.58, 0.13 * sind(af))
                ],
                0.002,
                M.iron,
                verts = 4,
                round = 1
            );
        }
        ring_at("washer", 0.014, (0, 1.58, 0), 0.003, M.iron, 10, 4);
    });
    join_all("floorLampSimple");
    settle!();
    check_at("floorLampSimple")
}

/// A point `t` of the way along the cubic Bézier through `p`.
fn bezier(p: &[(f64, f64, f64); 4], t: f64) -> (f64, f64, f64) {
    let u = 1.0 - t;
    let at = |i: usize| {
        let c = |q: (f64, f64, f64)| [q.0, q.1, q.2][i];
        pow(u, 3.0) * c(p[0])
            + 3.0 * u * u * t * c(p[1])
            + 3.0 * u * t * t * c(p[2])
            + pow(t, 3.0) * c(p[3])
    };
    (at(0), at(1), at(2))
}

// The arc lamp reaches over the tile in front of its own (content `overhang` [[0, 1]]): its base in the back left
// corner of its tile, its back on the wall face, the arc rising to about 2 m and coming down over the middle of the
// next tile, where its shade hangs over a sofa's seat, 1.33 m from the base across the floor. Built where it stands
// (settled with `centre = false`), so the shade's middle is the content's `light.at` as it is here.
const ARC_BASE: (f64, f64) = (-0.28, -0.28); // the base's middle (game x, depth): its back on the wall face, -0.43
const ARC_SHADE_AT: (f64, f64) = (0.0, 1.0); // the shade's middle: over the middle of the tile in front
const ARC_SHADE_Y: f64 = 1.7; // the shade's middle height (1.6 to 1.8 m)
const ARC: [(f64, f64, f64); 4] = [
    (ARC_BASE.0, 0.95, ARC_BASE.1),
    (ARC_BASE.0, 2.0, ARC_BASE.1),
    (ARC_SHADE_AT.0, 2.3, ARC_SHADE_AT.1),
    (ARC_SHADE_AT.0, ARC_SHADE_Y + 0.12, ARC_SHADE_AT.1),
];
const ARC_SHADE: [(f64, f64); 8] = [
    (0.5, -0.5),
    (0.42, 0.2),
    (0.12, 0.5),
    (0.0, 0.5),
    (0.0, 0.4),
    (0.1, 0.4),
    (0.38, 0.15),
    (0.46, -0.5),
];

/// A turned marble base in the back corner, a brass stem rising into one sweeping arc over the next tile (a sleeve
/// where the two meet, a dimmer on the stem) and a dome shade with a rolled lip on its socket.
pub fn lamp_floor_arc() -> Vec<Obj> {
    let (bx, bz) = ARC_BASE;
    let (sx, sz) = ARC_SHADE_AT;
    new_parts(|| {
        turn(
            "base",
            &[
                (0.0, 0.0),
                (0.15, 0.0),
                (0.15, 0.034),
                (0.144, 0.047),
                (0.13, 0.055),
                (0.07, 0.06),
                (0.0, 0.06),
            ],
            (bx, 0, bz),
            M.marble,
            36,
        );
        turn(
            "foot_collar",
            &[
                (0.0, 0.0),
                (0.032, 0.0),
                (0.032, 0.01),
                (0.022, 0.026),
                (0.017, 0.04),
                (0.0, 0.04),
            ],
            (bx, 0.06, bz),
            M.arc,
            16,
        );
        let mut arc = vec![(bx, 0.09, bz), (bx, 0.55, bz)];
        arc.extend((0..23).map(|k| bezier(&ARC, k as f64 / 22.0)));
        kit::tube!(F, "arc", &arc, 0.0125, M.arc, verts = 10, round = 2);
        turn(
            "sleeve",
            &[
                (0.0, 0.0),
                (0.017, 0.0),
                (0.019, 0.006),
                (0.019, 0.05),
                (0.017, 0.056),
                (0.0, 0.056),
            ],
            (bx, 0.92, bz),
            M.arc,
            14,
        );
        fbox!(
            F,
            "dimmer",
            0.03,
            0.06,
            0.022,
            (bx, 0.82, bz + 0.018),
            M.arc,
            bevel = 0.008,
            bevel_segments = 2
        );
        fcyl!(
            F,
            "dimmer_knob",
            0.007,
            0.01,
            (bx, 0.83, bz + 0.032),
            M.marble,
            rot = (90, 0, 0),
            verts = 10
        );
        socket(
            "socket",
            0.02,
            0.04,
            (sx, ARC_SHADE_Y + 0.09, sz),
            M.arc,
            12,
        );
        lathe(
            "shade",
            0.36,
            0.2,
            (sx, ARC_SHADE_Y, sz),
            &ARC_SHADE,
            M.arc_shade,
            Lathe {
                segments: 36,
                liner: Some(M.liner),
                ..Lathe::default()
            },
        );
        ring_at(
            "shade_lip",
            0.176,
            (sx, ARC_SHADE_Y - 0.096, sz),
            0.005,
            M.arc_shade,
            36,
            5,
        );
        bulb(
            "lampFloorArc",
            0.05,
            (sx, ARC_SHADE_Y - 0.04, sz),
            None,
            (1.0, 1.0, 0.75),
            14,
            8,
        );
    });
    join_all("lampFloorArc");
    settle!(centre = false);
    check_at("lampFloorArc")
}

/// Three tapered walnut legs on brass ferrules, meeting at a turned brass hub under a linen drum shade bound at its
/// edges, a diffuser and a wire fitter inside.
pub fn lamp_floor_tripod() -> Vec<Obj> {
    new_parts(|| {
        for a in [90, 210, 330] {
            let (c, s) = (cosd(a as f64), sind(a as f64));
            let pts: Vec<(f64, f64, f64)> = [(0.287, 0.03), (0.17, 0.62), (0.022, 1.27)]
                .iter()
                .map(|&(r, y)| (r * c, y, r * s))
                .collect();
            kit::tube!(
                F,
                format!("leg{a}"),
                &pts,
                0.0175,
                M.walnut,
                verts = 10,
                round = 3,
                scale = [0.75, 1.0, 0.8]
            );
            kit::tube!(
                F,
                format!("ferrule{a}"),
                &[(0.3 * c, 0.0, 0.3 * s), (0.285 * c, 0.04, 0.285 * s)],
                0.0125,
                M.brass,
                verts = 10,
                round = 1
            );
        }
        turn(
            "hub",
            &[
                (0.0, 0.0),
                (0.02, 0.0),
                (0.031, 0.012),
                (0.031, 0.058),
                (0.026, 0.066),
                (0.026, 0.076),
                (0.014, 0.08),
                (0.0, 0.08),
            ],
            (0, 1.24, 0),
            M.brass,
            16,
        );
        socket("socket", 0.02, 0.05, (0, 1.32, 0), M.brass, 12);
        bulb(
            "lampFloorTripod",
            0.04,
            (0, 1.4, 0),
            None,
            (1.0, 1.0, 1.2),
            14,
            8,
        );
        drum(
            "shade",
            0.24,
            0.24,
            0.32,
            (0, 1.29, 0),
            M.linen,
            Drum {
                segments: 40,
                rims: Some(M.linen),
                spider: Some(M.brass),
                ..Drum::default()
            },
        );
        fcyl!(F, "diffuser", 0.234, 0.004, (0, 1.3, 0), M.bulb, verts = 32);
    });
    join_all("lampFloorTripod");
    settle!();
    check_at("lampFloorTripod")
}

/// An uplighter: a stepped black base, a brass pole with turned collars and a dimmer, a frosted glass bowl with a
/// petalled rim on a cup.
pub fn lamp_torchiere() -> Vec<Obj> {
    new_parts(|| {
        turn(
            "base",
            &[
                (0.0, 0.0),
                (0.15, 0.0),
                (0.15, 0.012),
                (0.142, 0.024),
                (0.11, 0.034),
                (0.05, 0.04),
                (0.03, 0.05),
                (0.0, 0.05),
            ],
            (0, 0, 0),
            M.iron,
            36,
        );
        kit::tube!(
            F,
            "pole",
            &[(0, 0.05, 0), (0, 1.7, 0)],
            0.0125,
            M.brass,
            verts = 10,
            round = 1
        );
        for y in [0.6, 1.15] {
            turn(
                &format!("collar{}", float_repr(y)),
                &[
                    (0.0, 0.0),
                    (0.017, 0.0),
                    (0.021, 0.006),
                    (0.021, 0.022),
                    (0.017, 0.028),
                    (0.0, 0.028),
                ],
                (0, y, 0),
                M.brass,
                14,
            );
        }
        fbox!(
            F,
            "dimmer",
            0.034,
            0.07,
            0.026,
            (0, 0.95, 0.016),
            M.iron,
            bevel = 0.008,
            bevel_segments = 2
        );
        fcyl!(
            F,
            "dimmer_knob",
            0.009,
            0.012,
            (0, 0.96, 0.034),
            M.brass,
            rot = (90, 0, 0),
            verts = 12
        );
        turn(
            "cup",
            &[
                (0.0, 0.0),
                (0.018, 0.0),
                (0.026, 0.016),
                (0.03, 0.03),
                (0.044, 0.04),
                (0.0, 0.04),
            ],
            (0, 1.69, 0),
            M.brass,
            16,
        );
        // the bowl: lofted from its foot to a rim of eight soft petals, and back down inside
        let b = F.at(0, 0, 0);
        let (bx, by) = (b.x, b.y);
        let petals = |r: f64, y: f64, amp: f64| -> Vec<artkit::math::P3> {
            let n = 48;
            (0..n)
                .map(|i| {
                    let a = TAU * i as f64 / n as f64;
                    artkit::math::P3 {
                        x: bx + r * (1.0 + amp * cos(8.0 * a)) * cos(a),
                        y: by + r * (1.0 + amp * cos(8.0 * a)) * sin(a),
                        z: y,
                    }
                })
                .collect()
        };
        let rings = [
            (0.016, 1.71, 0.0),
            (0.06, 1.725, 0.01),
            (0.11, 1.755, 0.025),
            (0.16, 1.805, 0.04),
            (0.19, 1.85, 0.05),
            (0.172, 1.85, 0.05),
            (0.145, 1.81, 0.04),
            (0.1, 1.77, 0.025),
            (0.05, 1.746, 0.01),
            (0.006, 1.74, 0.0),
        ];
        let secs: Vec<Vec<artkit::math::P3>> =
            rings.iter().map(|&(r, y, amp)| petals(r, y, amp)).collect();
        loft!("bowl", &secs, material = M.bowl);
        bulb(
            "lampTorchiere",
            0.035,
            (0, 1.77, 0),
            None,
            (1.0, 1.0, 1.0),
            14,
            8,
        );
    });
    join_all("lampTorchiere");
    settle!();
    check_at("lampTorchiere")
}
