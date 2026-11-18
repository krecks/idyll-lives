//! LT1: the ceiling lights. The two pendants residents walk under hang clear of their heads: their lowest point
//! 2.1 m up (a room's 2.8 m ceiling, a 0.7 m drop; the reference's least clearance where people walk). The
//! chandelier and the industrial pendant hang lower, over a table or a sofa only (content `headHeight`, Q-12).

use super::common::{
    CEILING, DOME, F, Lathe, M, bulb, canopy, check_at, hang, lathe, loft_at, near_light, pear,
    ring_at, socket, turn,
};
use artkit::forms::{Plane, rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{Axis, P3, PI, V3, asin, cos, cosd, pow, sin, sind, sqrt};
use artkit::parts::{hinged, join, join_all, new_parts};
use artkit::py;
use artkit::scene::{Obj, RotationMode};
use artkit::{cylinder, extrude, fbox, fsphere, fsweep, revolve, settle, spin, sweep, torus};

/// The white dome on a brass socket and a black cord: a rolled lip round the dome, a brass collar on its crown, a
/// pear bulb showing below.
pub fn ceiling_pendant() -> Vec<Obj> {
    new_parts(|| {
        hang("ceilingPendant", 2.4, None, 0.06, None);
        socket("socket", 0.026, 0.05, (0, 2.35, 0), M.brass, 16);
        lathe(
            "dome",
            0.43,
            0.24,
            (0, 2.24, 0),
            &DOME,
            M.dome,
            Lathe {
                segments: 40,
                ..Lathe::default()
            },
        );
        ring_at("dome_lip", 0.209, (0, 2.124, 0), 0.006, M.dome, 40, 6);
        turn(
            "collar",
            &[
                (0.0, 0.0),
                (0.064, 0.0),
                (0.064, 0.004),
                (0.052, 0.012),
                (0.034, 0.018),
                (0.0, 0.018),
            ],
            (0, 2.344, 0),
            M.brass,
            24,
        );
        pear("ceilingPendant", 0.04, (0, 2.162, 0), 14);
    });
    join_all("ceilingPendant");
    settle!(floor = false);
    check_at("ceilingPendant")
}

/// A square lantern on a short stem: four frosted sides, each parted in four panes by black glazing bars, between
/// black corner posts, under a hipped lid with a knob; a diffuser below.
pub fn ceiling_light() -> Vec<Obj> {
    new_parts(|| {
        loft_at(
            "canopy",
            0,
            0,
            &[
                (2.772, 0.052, 0.052, 6.0),
                (2.785, 0.062, 0.062, 8.0),
                (2.8, 0.062, 0.062, 8.0),
            ],
            M.iron,
            16,
        );
        kit::tube!(
            F,
            "stem",
            &[(0, 2.772, 0), (0, 2.555, 0)],
            0.009,
            M.iron,
            verts = 8,
            round = 1
        );
        turn(
            "stem_collar",
            &[
                (0.0, 0.0),
                (0.016, 0.0),
                (0.016, 0.012),
                (0.01, 0.02),
                (0.0, 0.02),
            ],
            (0, 2.548, 0),
            M.iron,
            12,
        );
        // the hipped lid, curving up from its eaves to a knob
        loft_at(
            "lid",
            0,
            0,
            &[
                (2.5, 0.18, 0.18, 14.0),
                (2.512, 0.18, 0.18, 14.0),
                (2.518, 0.168, 0.168, 12.0),
                (2.53, 0.12, 0.12, 9.0),
                (2.542, 0.065, 0.065, 6.0),
                (2.55, 0.03, 0.03, 4.0),
            ],
            M.iron,
            24,
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("side_z{s}"),
                0.32,
                0.29,
                0.006,
                (0, 2.355, sf * 0.163),
                M.panel
            );
            fbox!(
                F,
                format!("side_x{s}"),
                0.006,
                0.29,
                0.32,
                (sf * 0.163, 2.355, 0),
                M.panel
            );
            for t in [-1, 1] {
                fbox!(
                    F,
                    format!("post{s}{t}"),
                    0.016,
                    0.31,
                    0.016,
                    (sf * 0.17, 2.355, t as f64 * 0.17),
                    M.iron,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
            // the glazing bars across each side: a cross over the frosted pane, and a rail under the lid
            fbox!(
                F,
                format!("bar_z{s}"),
                0.32,
                0.008,
                0.008,
                (0, 2.355, sf * 0.168),
                M.iron
            );
            fbox!(
                F,
                format!("mullion_z{s}"),
                0.008,
                0.29,
                0.008,
                (0, 2.355, sf * 0.168),
                M.iron
            );
            fbox!(
                F,
                format!("bar_x{s}"),
                0.008,
                0.008,
                0.32,
                (sf * 0.168, 2.355, 0),
                M.iron
            );
            fbox!(
                F,
                format!("mullion_x{s}"),
                0.008,
                0.29,
                0.008,
                (sf * 0.168, 2.355, 0),
                M.iron
            );
            fbox!(
                F,
                format!("head_z{s}"),
                0.34,
                0.016,
                0.01,
                (0, 2.494, sf * 0.168),
                M.iron,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("head_x{s}"),
                0.01,
                0.016,
                0.34,
                (sf * 0.168, 2.494, 0),
                M.iron,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            F,
            "rim",
            0.36,
            0.012,
            0.36,
            (0, 2.206, 0),
            M.iron,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(F, "diffuser", 0.32, 0.004, 0.32, (0, 2.214, 0), M.bulb);
        bulb(
            "ceilingLight",
            0.05,
            (0, 2.32, 0),
            None,
            (1.0, 1.0, 1.2),
            14,
            8,
        );
    });
    join_all("ceilingLight");
    settle!(floor = false);
    check_at("ceilingLight")
}

/// A turned brass column on a rod, three S-scrolled arms down to a ring of ten globes and three curved spokes to a
/// ring of six above, each globe in a bobeche; a drop globe and a finial under the column.
pub fn chandelier() -> Vec<Obj> {
    new_parts(|| {
        canopy("canopy", 0.08, 0.03, M.brass, 16, CEILING);
        kit::tube!(
            F,
            "rod",
            &[(0, CEILING - 0.03, 0), (0, 2.09, 0)],
            0.005,
            M.brass,
            verts = 8,
            round = 1
        );
        torus!(
            "loop",
            0.016,
            0.004,
            at = F.at(0, 2.515, 0),
            material = M.brass,
            rot = (90, 0, 0),
            major_segments = 12,
            minor_segments = 4
        );
        // the column, turned, its widest bulge the hub the arms spring from
        turn(
            "column",
            &[
                (0.0, 0.0),
                (0.01, 0.0),
                (0.02, 0.02),
                (0.028, 0.05),
                (0.03, 0.07),
                (0.026, 0.09),
                (0.012, 0.12),
                (0.009, 0.16),
                (0.014, 0.19),
                (0.008, 0.22),
                (0.0, 0.22),
            ],
            (0, 2.28, 0),
            M.brass,
            12,
        );
        turn(
            "collar",
            &[(0.0, 0.0), (0.014, 0.0), (0.014, 0.012), (0.0, 0.012)],
            (0, 2.204, 0),
            M.brass,
            10,
        );
        turn(
            "finial",
            &[
                (0.0, 0.0),
                (0.004, 0.0),
                (0.012, 0.012),
                (0.016, 0.024),
                (0.008, 0.036),
                (0.006, 0.05),
                (0.0, 0.05),
            ],
            (0, 2.04, 0),
            M.brass,
            10,
        );
        ring_at("ring_low", 0.435, (0, 2.05, 0), 0.013, M.brass, 36, 4);
        ring_at("ring_high", 0.27, (0, 2.21, 0), 0.011, M.brass, 28, 4);
        for a in [30, 150, 270] {
            let (c, s) = (cosd(a as f64), sind(a as f64));
            let pts: Vec<(f64, f64, f64)> = [
                (0.02, 2.35),
                (0.12, 2.39),
                (0.25, 2.35),
                (0.36, 2.2),
                (0.435, 2.062),
            ]
            .iter()
            .map(|&(r, y)| (r * c, y, r * s))
            .collect();
            kit::tube!(
                F,
                format!("arm{a}"),
                &pts,
                0.006,
                M.brass,
                verts = 5,
                round = 4
            );
        }
        for a in [90, 210, 330] {
            let (c, s) = (cosd(a as f64), sind(a as f64));
            let pts: Vec<(f64, f64, f64)> = [(0.012, 2.21), (0.14, 2.235), (0.27, 2.215)]
                .iter()
                .map(|&(r, y)| (r * c, y, r * s))
                .collect();
            kit::tube!(
                F,
                format!("spoke{a}"),
                &pts,
                0.0045,
                M.brass,
                verts = 5,
                round = 3
            );
        }
        for (ring, n, r, y, size) in [(0.435, 10, 0.035, 2.1, 0.02), (0.27, 6, 0.03, 2.25, 0.017)] {
            for k in 0..n {
                let a = (360 * k) as f64 / n as f64 + if n == 6 { 15.0 } else { 0.0 };
                let (x, z) = (ring * cosd(a), ring * sind(a));
                turn(
                    &format!("cup{}_{k}", float_repr(y)),
                    &[
                        (0.0, 0.0),
                        (size, size * 0.6),
                        (size, size * 0.85),
                        (0.0, size * 0.85),
                    ],
                    (x, y - r - 0.014, z),
                    M.brass,
                    6,
                );
                fsphere!(
                    F,
                    format!("globe{}_{k}", float_repr(y)),
                    r,
                    (x, y, z),
                    M.bulb,
                    segments = 8,
                    rings = 6
                );
            }
        }
        bulb(
            "chandelier",
            0.03,
            (0, 2.12, 0),
            None,
            (1.0, 1.0, 1.0),
            10,
            6,
        ); // a drop under the hub
    });
    join_all("chandelier");
    settle!(floor = false);
    check_at("chandelier")
}

/// A green enamel dome, white inside, a rolled rim, on a black fitter and a ribbed socket: an Edison bulb shows
/// below.
pub fn pendant_industrial() -> Vec<Obj> {
    new_parts(|| {
        hang("pendantIndustrial", 2.24, Some(M.iron), 0.06, None);
        socket("socket", 0.026, 0.05, (0, 2.205, 0), M.iron, 16);
        turn(
            "fitter",
            &[
                (0.0, 0.0),
                (0.058, 0.0),
                (0.058, 0.006),
                (0.044, 0.012),
                (0.032, 0.026),
                (0.03, 0.03),
                (0.0, 0.03),
            ],
            (0, 2.18, 0),
            M.iron,
            24,
        );
        lathe(
            "dome",
            0.38,
            0.24,
            (0, 2.07, 0),
            &DOME,
            M.industrial,
            Lathe {
                segments: 40,
                liner: Some(M.liner),
                ..Lathe::default()
            },
        );
        ring_at("dome_rim", 0.185, (0, 1.954, 0), 0.006, M.industrial, 40, 6);
        pear("pendantIndustrial", 0.045, (0, 1.975, 0), 16);
    });
    join_all("pendantIndustrial");
    settle!(floor = false);
    check_at("pendantIndustrial")
}

/// A white fan: a canopy and downrod, a motor with a band round its waist, a switch cup and a glass light dome
/// below, and five wooden blades with rounded tips on curved irons. The blades turn (`blades_turn`, ambient).
pub fn ceiling_fan() -> Vec<Obj> {
    let still = new_parts(|| {
        canopy("canopy", 0.08, 0.05, M.white, 20, CEILING);
        kit::tube!(
            F,
            "downrod",
            &[(0, 2.75, 0), (0, 2.615, 0)],
            0.013,
            M.white,
            verts = 10,
            round = 1
        );
        turn(
            "ball_cover",
            &[
                (0.0, 0.0),
                (0.016, 0.0),
                (0.024, 0.01),
                (0.025, 0.02),
                (0.016, 0.03),
                (0.0, 0.03),
            ],
            (0, 2.725, 0),
            M.white,
            14,
        );
        turn(
            "coupler",
            &[
                (0.0, 0.0),
                (0.022, 0.0),
                (0.024, 0.022),
                (0.016, 0.03),
                (0.0, 0.03),
            ],
            (0, 2.6, 0),
            M.white,
            14,
        );
        turn(
            "motor",
            &[
                (0.0, 0.0),
                (0.1, 0.0),
                (0.124, 0.008),
                (0.13, 0.025),
                (0.13, 0.05),
                (0.134, 0.054),
                (0.134, 0.066),
                (0.128, 0.07),
                (0.114, 0.095),
                (0.085, 0.112),
                (0.04, 0.12),
                (0.0, 0.12),
            ],
            (0, 2.5, 0),
            M.white,
            32,
        );
        turn(
            "switch_cup",
            &[
                (0.0, 0.0),
                (0.076, 0.0),
                (0.094, 0.01),
                (0.1, 0.022),
                (0.1, 0.04),
                (0.0, 0.04),
            ],
            (0, 2.46, 0),
            M.white,
            28,
        );
        bulb(
            "ceilingFan",
            0.09,
            (0, 2.46, 0),
            None,
            (1.0, 1.0, 0.5),
            24,
            10,
        );
    });
    join(&still, "ceilingFan", Some(P3::ZERO));
    // The blades reach from just outside the motor (0.135 m out) to 0.485 m, their tips' corners under 0.487 m: as
    // they turn they sweep a circle 0.974 m across, inside the 1 x 1 tile with its axis on the tile's middle.
    let blade: Vec<(f64, f64)> = smooth_outline(
        &[
            (-0.175, -0.042),
            (0.12, -0.05),
            (0.175, -0.05),
            (0.175, 0.05),
            (0.12, 0.05),
            (-0.175, 0.042),
        ],
        1,
        true,
    )
    .into_iter()
    .map(|(u, v)| (u.min(0.175), v))
    .collect();
    let rotor = new_parts(|| {
        for k in 0..5 {
            let a = 72 * k;
            let af = a as f64;
            extrude!(
                format!("blade{k}"),
                &blade,
                0.012,
                at = (0.31 * cosd(af), 0.31 * sind(af), 2.54),
                material = M.blade,
                plane = Plane::XY,
                rot = (10, 0, a),
                bevel = 0.003,
                bevel_segments = 1
            );
            // the iron: a flat strap curving down from under the motor onto the blade's root
            let pts: Vec<(f64, f64, f64)> =
                [(0.1, 2.552), (0.15, 2.546), (0.2, 2.531), (0.25, 2.528)]
                    .iter()
                    .map(|&(r, h)| (r * cosd(af), r * sind(af), h))
                    .collect();
            sweep!(
                format!("iron{k}"),
                &pts,
                0.0,
                profile = rounded_rect(0.008, 0.042, 0.003, 1),
                material = M.white,
                resolution = 3
            );
        }
    });
    let blades = hinged(join(&rotor, "blades", None), F.at(0, 2.54, 0));
    // Left where it's built, its axis on the middle: five blades reach farther one way than the other, so centring
    // its box would put the axis off the middle (the Model's `centre` is the box's middle instead).
    settle!(centre = false, floor = false);
    spin!(
        blades,
        "blades_turn",
        turns = 1.0,
        seconds = 2.5,
        axis = Axis::Z
    );
    check_at("ceilingFan")
}

/// Puts `obj` (built along Blender +z) at Blender point `at`, its +z along `direction` (Blender).
fn aim(obj: Obj, at: V3, direction: V3) -> Obj {
    obj.edit(|o| o.loc = at);
    obj.set_rotation_mode(RotationMode::Quaternion);
    obj.set_rotation_quaternion(direction.to_track_quat(Axis::Z, Axis::Y))
}

const SPOT: [(f64, f64); 17] = [
    (0.0, 0.0),
    (0.024, 0.0),
    (0.034, 0.008),
    (0.04, 0.02),
    (0.045, 0.03),
    (0.045, 0.038),
    (0.041, 0.042),
    (0.045, 0.046),
    (0.045, 0.054),
    (0.041, 0.058),
    (0.045, 0.062),
    (0.045, 0.158),
    (0.043, 0.162),
    (0.043, 0.17),
    (0.036, 0.17),
    (0.034, 0.162),
    (0.0, 0.162),
];

/// A white track (a channel with a slot underneath, end caps and a live end at its middle) with three black spots
/// on adapters, each aimed its own way: a can with cooling fins, a lip round its glowing lens.
pub fn track_lights() -> Vec<Obj> {
    new_parts(|| {
        let slot = [
            (0.02, -0.025),
            (0.02, 0.025),
            (-0.02, 0.025),
            (-0.02, 0.007),
            (-0.013, 0.007),
            (-0.013, -0.007),
            (-0.02, -0.007),
            (-0.02, -0.025),
        ];
        fsweep!(
            F,
            "track",
            &[(-0.462, 2.78, 0), (0.462, 2.78, 0)],
            0,
            M.white,
            profile = slot,
            up = (0, 0, 1)
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("end{s}"),
                0.016,
                0.042,
                0.052,
                (s as f64 * 0.462, 2.78, 0),
                M.white,
                bevel = 0.005,
                bevel_segments = 2
            );
        }
        fbox!(
            F,
            "canopy",
            0.12,
            0.02,
            0.12,
            (0, 2.79, 0),
            M.white,
            bevel = 0.006,
            bevel_segments = 2
        );
        fbox!(
            F,
            "live_end",
            0.09,
            0.03,
            0.054,
            (0, 2.765, 0),
            M.white,
            bevel = 0.008,
            bevel_segments = 2
        );
        for (k, (x, aim_at)) in [
            (-0.34, (-0.2, -1.0, 0.34)),
            (0.0, (0.16, -1.0, -0.26)),
            (0.34, (-0.26, -1.0, 0.42)),
        ]
        .into_iter()
        .enumerate()
        {
            let n = sqrt(py::sum([
                aim_at.0 * aim_at.0,
                aim_at.1 * aim_at.1,
                aim_at.2 * aim_at.2,
            ]));
            let d = [aim_at.0 / n, aim_at.1 / n, aim_at.2 / n];
            let top = (x, 2.68, 0);
            fbox!(
                F,
                format!("adapter{k}"),
                0.05,
                0.02,
                0.034,
                (x, 2.752, 0),
                M.iron,
                bevel = 0.005,
                bevel_segments = 1
            );
            kit::tube!(
                F,
                format!("stem{k}"),
                &[(x, 2.745, 0), top],
                0.008,
                M.iron,
                verts = 8,
                round = 1
            );
            fsphere!(
                F,
                format!("knuckle{k}"),
                0.014,
                top,
                M.iron,
                segments = 10,
                rings = 6
            );
            let (a, b) = (
                F.at(top.0, top.1, top.2),
                F.at(top.0 + d[0], top.1 + d[1], top.2 as f64 + d[2]),
            );
            let direction = V3::from(b) - V3::from(a);
            let can = revolve!(format!("head{k}"), &SPOT, material = M.iron, segments = 16);
            aim(can, V3::from(a), direction);
            let lens = cylinder!(
                format!("lens{k}"),
                0.035,
                0.004,
                material = M.bulb,
                verts = 16
            );
            aim(lens, V3::from(a) + direction * 0.166, direction);
            near_light("trackLights", top.1 + 0.17 * d[1], None);
        }
    });
    join_all("trackLights");
    settle!(floor = false);
    check_at("trackLights")
}

/// A paper globe on wire ribs, the paper sagging a little between them, open at the top and the bottom on wire
/// rims; hanging still from a white cord (a paper globe indoors hangs still: no draught), its bottom 2.1 m up.
pub fn paper_lantern() -> Vec<Obj> {
    let (y, big_r, k) = (2.37, 0.27, 0.93); // the globe's middle, its radius, its height over its width
    let ribs = [-0.2, -0.12, -0.04, 0.04, 0.12, 0.2];
    new_parts(|| {
        canopy("canopy", 0.04, 0.02, M.white, 16, CEILING);
        kit::tube!(
            F,
            "cord",
            &[(0, CEILING - 0.02, 0), (0, y + 0.255, 0)],
            0.004,
            M.white,
            verts = 6,
            round = 1
        );
        turn(
            "cap",
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.044, 0.004),
                (0.04, 0.012),
                (0.012, 0.016),
                (0.0, 0.016),
            ],
            (0, y + 0.246, 0),
            M.white,
            16,
        );
        // the paper: a turned skin from the bottom opening to the top one, pinched in at each rib, and back down
        // inside, coarser, so it shows a lining through the openings
        let th0 = asin(0.06 / big_r);
        let mut outer: Vec<(f64, f64)> = Vec::new();
        for i in 0..37 {
            let th = th0 + (PI - 2.0 * th0) * i as f64 / 36.0;
            let dz = -big_r * k * cos(th);
            let gap = ribs
                .iter()
                .map(|&r| (dz - r).abs())
                .fold(f64::INFINITY, |m, g| if g < m { g } else { m });
            let sag = if dz.abs() < 0.24 {
                1.0 - 0.016 * pow(sin(PI * gap.min(0.04) / 0.08), 2.0)
            } else {
                1.0
            };
            outer.push((big_r * sin(th) * sag, dz));
        }
        let inner: Vec<(f64, f64)> = (0..9)
            .map(|i| {
                let th = PI - th0 - (PI - 2.0 * th0) * i as f64 / 8.0;
                (big_r * sin(th) - 0.004, -big_r * k * cos(th))
            })
            .collect();
        near_light("paperLantern", y, None);
        let mut skin = outer.clone();
        skin.extend(&inner);
        skin.push(outer[0]);
        revolve!(
            "paperLantern_bulb",
            &skin,
            at = F.at(0, y, 0),
            material = M.paper,
            segments = 24
        );
        for (i, &dy) in ribs.iter().enumerate() {
            let r = big_r * sqrt(0.0_f64.max(1.0 - pow(dy / (big_r * k), 2.0))) + 0.002;
            ring_at(&format!("rib{i}"), r, (0, y + dy, 0), 0.003, M.rib, 24, 3);
        }
        for (i, (r, dz)) in [outer[0], outer[outer.len() - 1]].into_iter().enumerate() {
            ring_at(
                &format!("opening{i}"),
                r - 0.001,
                (0, y + dz, 0),
                0.003,
                M.rib,
                16,
                3,
            );
        }
        kit::tube!(
            F,
            "bulb_holder",
            &[(0, y + 0.25, 0), (0, y + 0.05, 0)],
            0.006,
            M.white,
            verts = 6,
            round = 1
        );
        fsphere!(
            F,
            "lamp_bulb",
            0.03,
            (0, y + 0.02, 0),
            M.bulb,
            segments = 10,
            rings = 6
        );
    });
    join_all("paperLantern");
    settle!(floor = false);
    check_at("paperLantern")
}
