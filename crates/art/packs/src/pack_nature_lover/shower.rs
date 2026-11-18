//! The cedar outdoor shower (1x1): slatted cedar walls on two sides, a duckboard, a copper rain head.

use super::common::{_pot, _rosette, M, WALL};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, cosd, sind, sqrt};
use artkit::parts::{join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{boulder, fbox, fcyl, fextrude, fsweep, grow, jitter, revolve, rod, settle, soft};

const HEAD: (f64, f64, f64) = (0.15, 2.0, -0.08); // the rain head's underside

/// A towel folded over a bar at height `top` (running along depth from z0 to z1 at x): hanging down both sides.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn _towel(
    f: &GameFrame,
    name: &str,
    x: f64,
    top: f64,
    z0: f64,
    z1: f64,
    mat: Mat,
    hang: f64,
) -> Obj {
    let t = 0.012;
    let pts = [
        (x - 0.03, top - hang, 0),
        (x - 0.028, top - 0.04, 0),
        (x - 0.018, top + 0.004, 0),
        (x, top + 0.014, 0),
        (x + 0.018, top + 0.004, 0),
        (x + 0.028, top - 0.04, 0),
        (x + 0.03, top - hang * 0.8, 0),
    ];
    let path: Vec<(f64, f64, f64)> = pts
        .iter()
        .map(|&(px, py, _)| (px, py, (z0 + z1) / 2.0))
        .collect();
    let obj = fsweep!(
        f,
        name,
        &path,
        0.0,
        mat,
        profile = rounded_rect(t, z1 - z0, t / 2.0, 1),
        up = (1, 0, 0),
        resolution = 2
    );
    jitter!(obj, 0.003, size = 0.1, seed = name.len() as f64, up = 0.3);
    obj
}

pub fn cedar_shower() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        for i in 0..12 {
            // back wall slats
            fbox!(
                f,
                format!("slat_b{i}"),
                0.065,
                1.9,
                0.03,
                (-0.41 + i as f64 * 0.074, 1.0, WALL + 0.015),
                if i % 2 != 0 {
                    M.cedar_wood
                } else {
                    M.cedar_wood_dark
                },
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for i in 0..11 {
            // left wall slats
            fbox!(
                f,
                format!("slat_l{i}"),
                0.03,
                1.9,
                0.065,
                (-0.43, 1.0, -0.35 + i as f64 * 0.077),
                if i % 2 != 0 {
                    M.cedar_wood
                } else {
                    M.cedar_wood_dark
                },
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for y in [0.06, 1.97] {
            // the rails behind the slats, a post at the open corner
            fbox!(
                f,
                format!("rail_b{}", float_repr(y)),
                0.9,
                0.05,
                0.04,
                (0.0, y, WALL + 0.045),
                M.cedar_wood_dark,
                bevel = 0.008,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("rail_l{}", float_repr(y)),
                0.04,
                0.05,
                0.86,
                (-0.4, y, 0.0),
                M.cedar_wood_dark,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "corner_post",
            0.06,
            1.98,
            0.06,
            (-0.42, 0.99, 0.42),
            M.cedar_wood_dark,
            bevel = 0.008,
            bevel_segments = 1
        );
        for i in 0..7 {
            // the duckboard on two bearers
            fbox!(
                f,
                format!("duck{i}"),
                0.78,
                0.03,
                0.085,
                (0.02, 0.05, -0.33 + i as f64 * 0.11),
                M.cedar_wood,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        for x in [-0.3, 0.32] {
            fbox!(
                f,
                format!("bearer{}", float_repr(x)),
                0.06,
                0.035,
                0.74,
                (x, 0.0175, 0.0),
                M.cedar_wood_dark,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        // the copper pipe in one piece: up from its floor flange, bent forward under the top and down to the rain head
        let (x, top) = (HEAD.0, 2.064);
        kit::tube!(
            f,
            "pipe",
            &kit::fillet!(
                &[
                    (x, 0.05, -0.395),
                    (x, top, -0.395),
                    (x, top, HEAD.2),
                    (x, HEAD.1 + 0.03, HEAD.2)
                ],
                0.05,
                steps = 4
            ),
            0.018,
            M.copper,
            verts = 10,
            round = 1
        );
        for y in [0.6, 1.6] {
            // clips holding it to the back wall
            fbox!(
                f,
                format!("pipe_clip{}", float_repr(y)),
                0.05,
                0.025,
                0.035,
                (x, y, -0.405),
                M.copper,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        revolve!(
            "flange",
            &[
                (0.0, 0.0),
                (0.034, 0.0),
                (0.034, 0.008),
                (0.022, 0.02),
                (0.0, 0.02)
            ],
            at = f.at(x, 0.065, -0.395),
            material = M.copper,
            segments = 12
        );
        revolve!(
            "rain_head",
            &[
                (0.0, 0.03),
                (0.035, 0.03),
                (0.06, 0.02),
                (0.15, 0.006),
                (0.152, 0.0),
                (0.0, 0.0)
            ],
            at = f.at(x, HEAD.1 + 0.002, HEAD.2),
            material = M.copper,
            segments = 24
        );
        fcyl!(
            f,
            "rain_face",
            0.135,
            0.004,
            (x, HEAD.1, HEAD.2),
            M.steel,
            verts = 24
        );
        revolve!(
            "valve",
            &[
                (0.0, 0.0),
                (0.045, 0.0),
                (0.045, 0.006),
                (0.028, 0.02),
                (0.022, 0.04),
                (0.0, 0.042)
            ],
            at = f.at(x, 1.1, -0.395),
            material = M.copper,
            segments = 14,
            rot = (90, 0, 0)
        );
        fsweep!(
            f,
            "valve_lever",
            &[
                (x, 1.1, -0.36),
                (x + 0.04, 1.1, -0.35),
                (x + 0.1, 1.09, -0.345)
            ],
            0.0,
            M.copper,
            profile = rounded_rect(0.01, 0.016, 0.004, 1),
            up = (0, 1, 0),
            resolution = 2
        );
        // a towel over a bar on the left wall, a soap shelf, a potted plant and pebbles
        fsweep!(
            f,
            "towel_bar",
            &[
                (-0.385, 1.5, 0.13),
                (-0.37, 1.5, 0.13),
                (-0.37, 1.5, 0.37),
                (-0.385, 1.5, 0.37)
            ],
            0.009,
            M.copper,
            verts = 8
        );
        _towel(f, "towel", -0.37, 1.5, 0.15, 0.35, M.towel, 0.35);
        fextrude!(
            f,
            "soap_shelf",
            &smooth_outline(
                &[
                    (-0.415, -0.25),
                    (-0.3, -0.25),
                    (-0.3, -0.15),
                    (-0.415, -0.15)
                ],
                1,
                true
            ),
            0.025,
            (0, 1.0, 0),
            M.cedar_wood_dark,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        soft!(
            f,
            "soap",
            0.08,
            0.03,
            0.05,
            (-0.36, 1.027, -0.2),
            M.soap,
            round = 0.012,
            round_segments = 1
        );
        let sy = _pot(f, "plant_pot", 0.3, 0.0, 0.3, 0.2, 0.18, None, true, None);
        _rosette(
            f,
            "plant",
            0.3,
            sy,
            0.3,
            9,
            0.2,
            0.08,
            &[M.green_light, M.green],
            (55.0, 78.0),
            0.35,
            "lance",
            0.3,
            None,
            4,
            3,
            3,
            0.0,
        );
        for (i, (px, pz)) in [(0.12, 0.4), (0.38, 0.06), (-0.2, 0.4), (0.38, -0.14)]
            .into_iter()
            .enumerate()
        {
            boulder!(
                format!("pebble{i}"),
                0.04,
                at = f.at(px, 0.0, pz),
                material = [M.pebble, M.pebble_b][i % 2],
                seed = i as f64,
                subdivisions = 1,
                scale = (1.2, 1, 0.6)
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The rain: streaks falling from the head to the duckboard, origin at the head: `water_on` grows them down.
    let (x, y, z) = HEAD;
    let rain = new_parts(|| {
        let mut rnd = Random::new(6);
        for k in 0..18 {
            let r = 0.11 * sqrt((k as f64 + 0.5) / 18.0);
            let a = k as f64 * 137.5;
            let (px, pz) = (x + r * cosd(a), z + r * sind(a));
            let h = rnd.uniform(1.86, 1.93);
            rod!(
                format!("stream{k}"),
                f.at(px, y, pz),
                f.at(px, y - h, pz),
                0.005,
                material = M.water,
                verts = 4
            );
        }
    });
    let rain = join(&rain, "water", Some(f.at(x, y, z)));
    rain.set_scale_axis(2, 0.001);
    let objs = settle!(centre = false);
    grow!(rain, "water_on", seconds = 0.6);
    objs
}
