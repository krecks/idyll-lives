//! LT2: the wall lights.

use super::common::{Drum, F, Lathe, M, WALL, bulb, check_at, drum, lathe};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{cosd, sind};
use artkit::parts::{join_all, new_parts};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, settle, torus};

// --- LT2: the wall lights ----------------------------------------------------------------------------------
/// A turned brass backplate, a gooseneck arm curving up into a candle cup, and a cone shade, rolled at its top and
/// bottom edges, on a wire fitter.
pub fn wall_sconce() -> Vec<Obj> {
    new_parts(|| {
        kit::turned!(
            F,
            "plate",
            0,
            1.7,
            WALL,
            &[
                (0.0, 0.0),
                (0.057, 0.0),
                (0.06, 0.003), // its rim on the wall eased
                (0.06, 0.006),
                (0.054, 0.012),
                (0.03, 0.018),
                (0.016, 0.022),
                (0.0, 0.022),
            ],
            M.brass,
            segments = 20,
            rot = (90.0, 0.0, 0.0)
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
        kit::turned!(
            F,
            "cup",
            0,
            1.72,
            -0.26,
            &[
                (0.0, 0.0),
                (0.012, 0.0),
                (0.024, 0.008),
                (0.026, 0.018),
                (0.02, 0.022),
                (0.02, 0.03),
                (0.0, 0.03),
            ],
            M.brass,
            segments = 14
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

const ARM_SHADE: [(f64, f64); 10] = [
    (0.485, -0.5),
    (0.5, -0.481), // the rim's outer corner eased
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
        kit::turned!(
            F,
            "plate",
            0,
            1.55,
            WALL,
            &[
                (0.0, 0.0),
                (0.052, 0.0),
                (0.055, 0.003), // its rim on the wall eased
                (0.055, 0.006),
                (0.048, 0.014),
                (0.022, 0.02),
                (0.0, 0.02),
            ],
            M.iron,
            segments = 22,
            rot = (90.0, 0.0, 0.0)
        );
        kit::turned!(
            F,
            "pivot",
            0,
            1.55,
            WALL + 0.018,
            &[(0.0, 0.0), (0.016, 0.0), (0.016, 0.03), (0.0, 0.03)],
            M.brass,
            segments = 12,
            rot = (90.0, 0.0, 0.0)
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
                (-0.055, 0.009),
                (-0.051, 0.0125), // its back on the wall eased
                (0.035, 0.0125),
                (0.055, -0.004),
                (0.055, -0.022),
                (0.046, -0.022),
                (0.044, -0.006),
                (0.03, -0.0125),
                (-0.051, -0.0125),
                (-0.055, -0.009),
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
            plane = Facing::Side,
            bevel = 0.003,
            bevel_segments = 1
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
