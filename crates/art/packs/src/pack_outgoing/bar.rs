//! The home party bar and the conversation sectional.

use super::common::{_bottle, _glass, _stool, BOTTLES, M, WALL};
use artkit::bevel::ease;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3};
use artkit::parts::{aside, hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{
    fbox, fcone, fcyl, fextrude, fsphere, fsweep, leaf, loft, revolve, ring, settle, soft, squash,
    swing, torus,
};

// --- the home party bar (2x1): a back bar of bottles, the front counter with a beer tap, two stools ---------
const TAP: (f64, f64, f64) = (0.24, 1.315, 0.04); // the tap lever's pivot (x, height, depth)

/// The back bar against the wall: a cabinet of four raised-panel doors with brass knobs, two shelves of bottles and
/// glasses on brass uprights, each shelf with a gallery rail and a strip light, a plum panel behind.
fn _back_bar(f: &GameFrame) {
    fbox!(
        f,
        "back_cabinet",
        1.82,
        0.84,
        0.25,
        (0, 0.44, WALL + 0.125),
        M.wood,
        bevel = 0.008,
        bevel_segments = 1
    );
    fbox!(
        f,
        "back_plinth",
        1.78,
        0.03,
        0.23,
        (0, 0.015, WALL + 0.12),
        M.dark
    );
    for x in [-0.68, -0.23, 0.23, 0.68] {
        kit::shaker!(
            f,
            format!("cabinet_door{x}"),
            0.42,
            0.7,
            (x, 0.44, WALL + 0.256),
            0.014,
            M.oak,
            rail = 0.06,
            raised = true
        );
        kit::knob!(
            f,
            format!("cabinet_knob{x}"),
            x + if x < 0.0 { 0.17 } else { -0.17 },
            0.55,
            WALL + 0.263,
            0.012,
            M.brass,
            segments = 8
        );
    }
    fbox!(
        f,
        "back_top",
        1.84,
        0.03,
        0.27,
        (0, 0.875, WALL + 0.135),
        M.wood,
        bevel = 0.008,
        bevel_segments = 2
    );
    for y in [1.28, 1.62] {
        fbox!(
            f,
            format!("shelf{y}"),
            1.8,
            0.03,
            0.18,
            (0, y, WALL + 0.09),
            M.wood,
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(
            f,
            format!("led{y}"),
            1.78,
            0.012,
            0.01,
            (0, y - 0.021, WALL + 0.17),
            M.sunny
        );
        kit::tube!(
            f,
            format!("gallery{y}"),
            &[
                (-0.88, y + 0.015, WALL + 0.17),
                (-0.88, y + 0.06, WALL + 0.17),
                (0.88, y + 0.06, WALL + 0.17),
                (0.88, y + 0.015, WALL + 0.17)
            ],
            0.005,
            M.brass,
            verts = 6,
            round = 1
        );
    }
    for x in [-0.89, 0.89] {
        for z in [WALL + 0.02, WALL + 0.16] {
            fsweep!(
                f,
                format!("upright{x}{z}"),
                &[(x, 0.89, z), (x, 1.82, z)],
                0.012,
                M.brass,
                verts = 8
            );
        }
    }
    fbox!(
        f,
        "mirror_back",
        1.76,
        0.62,
        0.008,
        (0, 1.36, WALL + 0.004),
        M.plum
    );
    for (i, x) in [-0.76, -0.64, -0.52, -0.4, 0.34, 0.46, 0.6, 0.74]
        .into_iter()
        .enumerate()
    {
        _bottle(
            f,
            &format!("bottle{i}"),
            x,
            1.635,
            WALL + 0.09,
            0.26 + 0.04 * (i % 3) as f64,
            BOTTLES[i],
            i % 3 == 1,
        );
    }
    for x in [-0.68, -0.46, 0.42, 0.68] {
        _glass(
            f,
            &format!("shelf_glass{x}"),
            x,
            1.295,
            WALL + 0.09,
            0.13,
            0.035,
            None,
        );
    }
    for (i, x) in [-0.1, 0.0, 0.1, 0.2].into_iter().enumerate() {
        // wine on the cabinet's top
        let b = f.at(x, 0.89, WALL + 0.13);
        let h = 0.3 - 0.02 * (i % 2) as f64;
        let sections: Vec<Vec<P3>> = [
            (0.0, 0.94),
            (0.02, 1.0),
            (0.62, 1.0),
            (0.75, 0.42),
            (0.8, 0.34),
            (1.0, 0.34),
        ]
        .iter()
        .map(|&(t, k)| {
            ring!(
                0.036 * k,
                0.036 * k,
                z = b.z + h * t,
                verts = 8,
                at = (b.x, b.y)
            )
        })
        .collect();
        loft!(
            format!("wine{i}"),
            &sections,
            material = if i % 2 != 0 { M.wine_a } else { M.wine_b }
        );
        fcyl!(
            f,
            format!("wine_foil{i}"),
            0.013,
            0.05,
            (x, 0.89 + h - 0.02, WALL + 0.13),
            if i % 2 != 0 { M.coral } else { M.brass },
            verts = 8
        );
    }
}

/// The front counter: a coral front fluted in one slab (extruded from a scalloped outline), an oak top with a rounded
/// nose, a brass foot rail bent back into the counter at each end.
fn _counter(f: &GameFrame) {
    let (n, w, zf) = (24, 1.5, 0.12);
    // (its four upright corners cut across 4 mm)
    let e = 0.004;
    let mut pts = vec![(-w / 2.0 + e, -0.12), (-w / 2.0, -0.12 + e)];
    for k in 0..=n {
        // the scalloped face: a flute every 1/n of the width
        let x = -w / 2.0 + k as f64 * w / n as f64;
        if k == 0 || k == n {
            let s = if k == 0 { 1.0 } else { -1.0 };
            pts.extend(if k == 0 {
                [(x, zf - e), (x + s * e, zf)]
            } else {
                [(x + s * e, zf), (x, zf - e)]
            });
        } else {
            pts.push((x, zf));
        }
        if k < n {
            pts.push((x + w / n as f64 / 2.0, zf + 0.012));
        }
    }
    pts.extend([(w / 2.0, -0.12 + e), (w / 2.0 - e, -0.12)]);
    pts.reverse();
    fextrude!(
        f,
        "counter",
        &pts,
        0.925,
        (0, 0.4925, 0),
        M.body,
        plane = Facing::Top
    );
    fbox!(
        f,
        "counter_plinth",
        1.48,
        0.03,
        0.22,
        (0, 0.015, 0.0),
        M.dark
    );
    fbox!(
        f,
        "counter_top",
        1.62,
        0.05,
        0.38,
        (0, 0.98, 0.06),
        M.oak,
        bevel = 0.018,
        bevel_segments = 3
    );
    fbox!(f, "counter_cap", 1.5, 0.012, 0.24, (0, 0.961, 0.0), M.body);
    kit::tube!(
        f,
        "foot_rail",
        &kit::fillet!(
            &[
                (-0.72, 0.2, 0.135),
                (-0.72, 0.2, 0.17),
                (0.72, 0.2, 0.17),
                (0.72, 0.2, 0.135)
            ],
            0.03
        ),
        0.015,
        M.brass,
        verts = 10,
        round = 1
    );
    for x in [-0.6, 0.6] {
        fsweep!(
            f,
            format!("rail_bracket{x}"),
            &[(x, 0.08, 0.13), (x, 0.18, 0.165), (x, 0.2, 0.17)],
            0.008,
            M.brass,
            verts = 6,
            resolution = 2
        );
    }
}

/// On the counter: the beer tower over its drip tray, a pint, two cocktails, a shaker and a bowl of limes.
fn _bar_top(f: &GameFrame) {
    let ty = 1.035; // (the counter top 1.005: a bar counter's height)
    soft!(
        f,
        "drip_tray",
        0.16,
        0.02,
        0.1,
        (TAP.0, ty + 0.01, 0.13),
        M.chrome,
        round = 0.006,
        round_segments = 1
    );
    for k in 0..5 {
        fbox!(
            f,
            format!("drip_slot{k}"),
            0.13,
            0.002,
            0.006,
            (TAP.0, ty + 0.021, 0.095 + k as f64 * 0.018),
            M.dark
        );
    }
    let tower = kit::tube!(
        f,
        "tap_tower",
        &kit::fillet!(
            &[
                (TAP.0, ty, 0.0),
                (TAP.0, TAP.1 - 0.01, 0.0),
                (TAP.0, TAP.1 - 0.01, TAP.2 + 0.02)
            ],
            0.03
        ),
        0.024,
        M.chrome,
        verts = 12,
        round = 1
    );
    ease(tower, 60.0, 1);
    revolve!(
        "tap_flange",
        &[
            (0.0, 0.0),
            (0.045, 0.0),
            (0.045, 0.006),
            (0.028, 0.02),
            (0.0, 0.02)
        ],
        at = f.at(TAP.0, ty, 0.0),
        material = M.chrome,
        segments = 14
    );
    fcone!(
        f,
        "tap_spout",
        0.012,
        0.007,
        0.08,
        (TAP.0, TAP.1 - 0.06, TAP.2 + 0.03),
        M.chrome,
        verts = 10
    );
    revolve!(
        "pint",
        &[
            (0.0, 0.0),
            (0.03, 0.0),
            (0.036, 0.13),
            (0.038, 0.14),
            (0.034, 0.14),
            (0.032, 0.02),
            (0.0, 0.02)
        ],
        at = f.at(TAP.0, ty + 0.02, 0.13),
        material = M.glassy,
        segments = 12
    );
    fcone!(
        f,
        "pint_beer",
        0.027,
        0.032,
        0.1,
        (TAP.0, ty + 0.075, 0.13),
        M.beer,
        verts = 12
    );
    revolve!(
        "pint_head",
        &[
            (0.0, 0.0),
            (0.033, 0.0),
            (0.034, 0.012),
            (0.02, 0.022),
            (0.0, 0.024)
        ],
        at = f.at(TAP.0, ty + 0.124, 0.13),
        material = M.cream,
        segments = 12
    );
    _glass(
        f,
        "cocktail_a",
        -0.42,
        ty,
        0.1,
        0.14,
        0.035,
        Some(M.magenta),
    );
    _glass(f, "cocktail_b", 0.02, ty, 0.15, 0.14, 0.035, Some(M.sunny));
    revolve!(
        "shaker",
        &[
            (0.0, 0.0),
            (0.032, 0.0),
            (0.04, 0.12),
            (0.042, 0.13),
            (0.03, 0.15),
            (0.022, 0.19),
            (0.016, 0.2),
            (0.0, 0.205)
        ],
        at = f.at(0.5, ty, 0.04),
        material = M.chrome,
        segments = 12
    );
    revolve!(
        "bowl",
        &[
            (0.0, 0.0),
            (0.05, 0.0),
            (0.09, 0.06),
            // (the rim rolled over in three turns)
            (0.0898, 0.062),
            (0.0883, 0.0633),
            (0.0863, 0.0633),
            (0.045, 0.008),
            (0.0, 0.008)
        ],
        at = f.at(-0.15, ty, 0.04),
        material = M.porcelain,
        segments = 14
    );
    for (dx, dz, m) in [
        (-0.03, 0.0, M.lime),
        (0.03, 0.02, M.lime),
        (0.0, -0.03, M.sunny),
    ] {
        fsphere!(
            f,
            format!("lime{dx}{dz}"),
            0.03,
            (-0.15 + dx, ty + 0.06, 0.04 + dz),
            m,
            scale = (1, 1, 0.85),
            segments = 8,
            rings = 5
        );
    }
}

pub fn party_bar() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _back_bar(f);
        _counter(f);
        _bar_top(f);
        _stool(f, "stool_l", -0.5, 0.28, 0.74);
        _stool(f, "stool_r", 0.5, 0.28, 0.74);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The tap's lever: a tall turned handle on the faucet, its origin on the pivot: `tap` pulls it towards the front.
    let lever = new_parts(|| {
        revolve!(
            "lever",
            &[
                (0.0, 0.0),
                (0.016, 0.0),
                (0.016, 0.03),
                (0.012, 0.035),
                (0.014, 0.05),
                (0.022, 0.16),
                (0.024, 0.18),
                (0.016, 0.2),
                (0.0, 0.205)
            ],
            at = f.at(TAP.0, TAP.1, TAP.2),
            material = M.body,
            segments = 12
        );
    });
    let lever = hinged(join(&lever, "lever", None), f.at(TAP.0, TAP.1, TAP.2));
    let objs = settle!(centre = false);
    swing!(lever, "tap", 40.0, seconds = 0.5, axis = Axis::X);
    objs
}

// --- the conversation sectional (2x2): an L-sofa, a console behind it, a pouf in the free corner ----------

/// An upholstered panel (a base, an arm, a back): firm, its edges rounded (in `steps`, or as close as the curve
/// asks), a welt round its thin side's seams.
#[allow(clippy::too_many_arguments)]
fn _panel(
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    at: (f64, f64, f64),
    mat: Mat,
    round: f64,
    piping: Option<f64>,
    steps: Option<usize>,
) -> Obj {
    soft!(
        GameFrame::new(0.0),
        name,
        w,
        h,
        d,
        at,
        mat,
        round = round,
        piping = piping,
        round_segments = steps,
        flat = 1 // (firm: no crown to step across)
    )
}

pub fn conversation_sectional() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _panel(
            "base",
            1.9,
            0.28,
            0.86,
            (0.0, 0.18, -0.02),
            M.frame,
            0.04,
            None,
            Some(1),
        );
        _panel(
            "chaise_base",
            0.56,
            0.28,
            0.54,
            (0.66, 0.18, 0.66),
            M.frame,
            0.04,
            None,
            Some(1),
        );
        _panel(
            "back",
            1.9,
            0.4,
            0.2,
            (0.0, 0.6, -0.36),
            M.frame,
            0.07,
            None,
            Some(1),
        );
        _panel(
            "arm_l",
            0.18,
            0.3,
            1.38,
            (-0.86, 0.47, 0.06),
            M.frame,
            0.07,
            Some(0.006),
            Some(2),
        );
        _panel(
            "arm_r",
            0.16,
            0.24,
            1.36,
            (0.86, 0.44, 0.25),
            M.frame,
            0.06,
            Some(0.006),
            Some(2),
        );
        for x in [-0.42, 0.3] {
            soft!(
                f,
                format!("back_cushion{x}"),
                0.68,
                0.36,
                0.16,
                (x, 0.68, -0.2),
                M.upholstery,
                rot = (-12, 0, 0),
                round = 0.05,
                puff = 0.03,
                flat = 2,
                resting = true // (unpiped: its cords cost 320 triangles past the aim, 1.5 px at the play camera)
            );
        }
        for (name, w, x, z, m, rot) in [
            ("pillow_l", 0.34, -0.6, -0.08, M.sunny, (-15, 0, -20)),
            ("pillow_r", 0.32, 0.6, -0.06, M.body, (-15, 0, 18)),
        ] {
            soft!(
                f,
                name,
                w,
                0.3,
                0.1,
                (x, 0.66, z),
                m,
                rot = rot,
                round = 0.045,
                puff = 0.035,
                waist = 0.012,
                pinch = 0.006,
                resting = true
            );
        }
        soft!(
            f,
            "pillow_c",
            0.28,
            0.26,
            0.1,
            (0.74, 0.6, 0.32),
            M.cream,
            rot = (-10, 0, 80),
            round = 0.045,
            puff = 0.03,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
        for (x, z) in [
            (-0.86, -0.4),
            (-0.86, 0.36),
            (0.86, -0.4),
            (0.86, 0.88),
            (0.42, 0.88),
        ] {
            fsweep!(
                f,
                format!("foot{x}{z}"),
                &[(x, 0.045, z), (x, 0.0, z)],
                0.025,
                M.brass,
                verts = 10,
                scale = [1.0, 0.75]
            );
        }
        // the pouf with a tray of drinks in the free front-left corner, its top buttoned
        revolve!(
            "pouf",
            &[
                (0.0, 0.0),
                (0.2, 0.0),
                (0.23, 0.06),
                (0.235, 0.18),
                (0.23, 0.3),
                (0.2, 0.35),
                (0.06, 0.36),
                (0.0, 0.355)
            ],
            at = f.at(-0.52, 0, 0.7),
            material = M.pouf,
            segments = 22
        );
        torus!(
            "pouf_welt",
            0.228,
            0.007,
            at = f.at(-0.52, 0.18, 0.7),
            material = M.pouf,
            major_segments = 22,
            minor_segments = 3
        );
        revolve!(
            "tray",
            &[
                (0.0, 0.0),
                (0.15, 0.0),
                (0.15, 0.025),
                (0.142, 0.025),
                (0.14, 0.008),
                (0.0, 0.008)
            ],
            at = f.at(-0.52, 0.355, 0.7),
            material = M.oak_plain,
            segments = 16
        );
        for (x, z, m) in [(-0.46, 0.66, M.porcelain), (-0.58, 0.74, M.magenta)] {
            revolve!(
                format!("cup{x}"),
                &[
                    (0.0, 0.0),
                    (0.03, 0.0),
                    (0.036, 0.09),
                    (0.032, 0.09),
                    (0.028, 0.01),
                    (0.0, 0.01)
                ],
                at = f.at(x, 0.363, z),
                material = m,
                segments = 8
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the console table behind it with its lamp, plant and books: drawn, not the sofa's size (as a desk's chair)
    let console = new_parts(|| {
        // the console behind it: oak top and shelf on brass legs, a lamp, a plant, books and magazines
        fbox!(
            f,
            "console_top",
            1.8,
            0.04,
            0.32,
            (0, 0.82, -0.75),
            M.oak_plain,
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(
            f,
            "console_shelf",
            1.76,
            0.03,
            0.28,
            (0, 0.2, -0.75),
            M.oak_plain,
            bevel = 0.006,
            bevel_segments = 1
        );
        for x in [-0.86, 0.86] {
            for z in [-0.88, -0.62] {
                fsweep!(
                    f,
                    format!("console_leg{x}{z}"),
                    &[(x, 0.0, z), (x, 0.8, z)],
                    0.014,
                    M.brass,
                    verts = 6,
                    scale = [0.7, 1.0]
                );
            }
            fsweep!(
                f,
                format!("console_stretcher{x}"),
                &[(x, 0.1, -0.88), (x, 0.1, -0.62)],
                0.008,
                M.brass,
                verts = 6
            );
        }
        revolve!(
            "lamp_base",
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.075, 0.01),
                (0.11, 0.1),
                (0.1, 0.16),
                (0.07, 0.22),
                (0.025, 0.27),
                (0.022, 0.296),
                (0.018, 0.3),
                (0.0, 0.3)
            ],
            at = f.at(-0.6, 0.84, -0.76),
            material = M.porcelain,
            segments = 12
        );
        revolve!(
            "lamp_shade",
            &[(0.17, 0.0), (0.135, 0.24), (0.13, 0.24), (0.165, 0.005)],
            at = f.at(-0.6, 1.18, -0.76),
            material = M.cream,
            segments = 16
        );
        revolve!(
            "plant_pot",
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.09, 0.17),
                (0.094, 0.18),
                (0.082, 0.18),
                (0.078, 0.16),
                (0.0, 0.16)
            ],
            at = f.at(0.55, 0.84, -0.76),
            material = M.pot,
            segments = 12
        );
        for k in 0..10 {
            leaf!(
                format!("plant_leaf{k}"),
                f.at(0.55, 1.0, -0.76),
                (k * 36 + 10) as f64,
                (35 + 25 * (k % 2)) as f64,
                0.17 - 0.04 * (k % 2) as f64,
                0.09,
                if k % 3 != 0 { M.leaf } else { M.lime },
                shape = "heart",
                droop = 0.4,
                rows = 2,
                ring = 3
            );
        }
        soft!(
            f,
            "book_a",
            0.3,
            0.06,
            0.2,
            (0.1, 0.87, -0.76),
            M.body,
            round = 0.006,
            round_segments = 1
        );
        soft!(
            f,
            "book_b",
            0.28,
            0.05,
            0.19,
            (0.1, 0.925, -0.76),
            M.sunny,
            rot = (0, 0, 8),
            round = 0.006,
            round_segments = 1
        );
        for (i, m) in [M.turq, M.coral, M.sunny, M.plum].into_iter().enumerate() {
            let i = i as i32;
            fbox!(
                f,
                format!("magazine{i}"),
                0.24,
                0.012,
                0.18,
                (0.02 * i as f64, 0.222 + 0.013 * i as f64, -0.75),
                m,
                rot = (0, 0, 6 * i)
            );
        }
    });
    aside(&console, "console");
    // The seat cushions (top 0.45, where residents sit), origin on their underside: `sink_in` presses them.
    let seats = new_parts(|| {
        for (name, w, d, x, z) in [
            ("seat_l", 0.84, 0.66, -0.5, 0.06),
            ("seat_r", 0.64, 0.66, 0.32, 0.06),
            ("seat_chaise", 0.5, 0.54, 0.66, 0.65),
        ] {
            soft!(
                f,
                name,
                w,
                0.11,
                d,
                (x, 0.385 - 0.008, z),
                M.upholstery,
                round = 0.045,
                puff = 0.016,
                piping = 0.006,
                flat = 2,
                resting = true
            );
        }
    });
    let seats = hinged(join(&seats, "seats", None), f.at(0, 0.32, 0.06));
    let objs = settle!(centre = false);
    squash!(seats, "sink_in", (1, 1, 0.8), seconds = 0.6);
    objs
}
