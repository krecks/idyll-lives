//! The cool-down meditation cushion, the grudge-to-gratitude writing desk and the glacier cold-plunge tub.

use super::common::{M, fluted};
use crate::ease::ease_ends;
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::math::{P3, PI, cosd, pow, sin, sind};
use artkit::motion::{Squash, squash};
use artkit::parts::{hinged, join, new_parts, numbered, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{
    boulder, fbox, fcyl, fextrude, fsphere, fsweep, jitter, kit, loft, ring, settle, slide, soft,
    torus,
};

// --- the cool-down meditation cushion (1x1): a round mat, a zabuton and a tall zafu, a singing bowl ---------
pub fn cool_down_cushion() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fcyl!(
            f,
            "mat",
            0.43,
            0.012,
            (0, 0.006, 0),
            M.calm_light,
            verts = 40
        );
        torus!(
            "mat_edge",
            0.42,
            0.009,
            at = f.at(0, 0.012, 0),
            material = M.calm_deep,
            major_segments = 40,
            minor_segments = 3
        );
        // the zabuton: a plump mat, tufted at four points, piped round
        soft!(
            f,
            "zabuton",
            0.62,
            0.062,
            0.58,
            (0, 0.044, -0.02),
            M.oat,
            round = 0.03,
            puff = 0.008,
            flat = 2
        );
        for sx in [-1.0, 1.0] {
            for sz in [-1.0, 1.0] {
                fsphere!(
                    f,
                    numbered("tuft"),
                    0.012,
                    (sx * 0.2, 0.082, -0.02 + sz * 0.19),
                    M.oat,
                    scale = (1, 1, 0.4),
                    segments = 6,
                    rings = 4
                );
            }
        }
        // the singing bowl on its ring cushion and stand, the mallet beside it
        kit::turned!(
            f,
            "bowl_stand",
            0.32,
            0.012,
            0.3,
            &[
                (0.0, 0.0),
                (0.1, 0.0),
                (0.108, 0.004),
                (0.11, 0.012),
                (0.105, 0.022),
                (0.09, 0.026),
                (0.0, 0.026),
            ],
            M.oak,
            segments = 16
        );
        torus!(
            "bowl_ring",
            0.055,
            0.014,
            at = f.at(0.32, 0.045, 0.3),
            material = M.fire,
            major_segments = 14,
            minor_segments = 6
        );
        kit::turned!(
            f,
            "singing_bowl",
            0.32,
            0.045,
            0.3,
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.08, 0.02),
                (0.1, 0.06),
                (0.102, 0.08),
                (0.096, 0.082),
                (0.092, 0.064),
                (0.075, 0.03),
                (0.045, 0.012),
                (0.0, 0.012),
            ],
            M.brass,
            segments = 20
        );
        fsweep!(
            f,
            "mallet",
            &[(0.2, 0.026, 0.3), (0.07, 0.04, 0.385)],
            0.01,
            M.oak,
            verts = 8,
            scale = [0.8, 1.1]
        );
        fcyl!(
            f,
            "mallet_felt",
            0.017,
            0.06,
            (0.09, 0.037, 0.372),
            M.calm,
            rot = (90, 0, 57),
            verts = 10,
            bevel = 0.005,
            bevel_segments = 1
        );
        // a candle on its dish, a potted plant and an incense stick on a leaf-shaped holder
        kit::turned!(
            f,
            "candle_dish",
            -0.32,
            0.012,
            0.3,
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.065, 0.006),
                (0.064, 0.011),
                (0.06, 0.0125),
                (0.046, 0.006),
                (0.0, 0.006),
            ],
            M.brass,
            segments = 16
        );
        kit::turned!(
            f,
            "candle",
            -0.32,
            0.018,
            0.3,
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.04, 0.064),
                (0.037, 0.07),
                (0.034, 0.0705),
                (0.022, 0.064),
                (0.0, 0.062),
            ],
            M.candle,
            segments = 14
        );
        fsweep!(
            f,
            "wick",
            &[(-0.32, 0.08, 0.3), (-0.319, 0.094, 0.3)],
            0.0018,
            M.dark,
            verts = 4
        );
        fsphere!(
            f,
            "candle_flame",
            0.011,
            (-0.319, 0.108, 0.3),
            M.candle_flame,
            scale = (1, 1, 2.0),
            segments = 8,
            rings = 5
        );
        kit::turned!(
            f,
            "plant_pot",
            -0.3,
            0.0,
            -0.32,
            &[
                (0.0, 0.0),
                (0.045, 0.0),
                (0.052, 0.07),
                (0.062, 0.075),
                (0.062, 0.097),
                (0.059, 0.1),
                (0.054, 0.1),
                (0.05, 0.092),
                (0.0, 0.092),
            ],
            M.pot,
            segments = 16
        );
        for k in 0..7 {
            let a = (k * 51 + 10) as f64;
            let (c, s) = (cosd(a), sind(a));
            let l = 0.07 + 0.039 * ((k * 5) % 3) as f64 / 2.0;
            fsweep!(
                f,
                numbered("leaf"),
                &[
                    (-0.3 + 0.01 * c, 0.09, -0.32 + 0.01 * s),
                    (-0.3 + 0.35 * l * c, 0.09 + l * 1.1, -0.32 + 0.35 * l * s),
                    (
                        -0.3 + 0.8 * l * c,
                        0.09 + l * 1.25 + 0.01,
                        -0.32 + 0.8 * l * s
                    )
                ],
                0,
                M.leaf,
                profile = rounded_rect(0.003, 0.034, 0.001, 1),
                up = (0, 0, 1),
                resolution = 2,
                scale = [0.3, 1.0, 0.15]
            );
        }
        fextrude!(
            f,
            "incense_holder",
            &smooth_outline(
                &[(-0.06, 0.0), (0.0, 0.022), (0.06, 0.0), (0.0, -0.022)],
                2,
                true
            ),
            0.008,
            (0.3, 0.016, -0.3),
            M.brass,
            plane = Facing::Top,
            rot = (0, 0, 40)
        );
        fsweep!(
            f,
            "incense",
            &[(0.3, 0.02, -0.3), (0.33, 0.21, -0.33)],
            0.0028,
            M.ember,
            verts = 4
        );
        fsweep!(
            f,
            "incense_ash",
            &[(0.328, 0.2, -0.328), (0.332, 0.225, -0.332)],
            0.0032,
            M.ash,
            verts = 4
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The zafu, a real one's size (0.38 across, 10 cm on the 8 cm zabuton): its side gathered in pleats into the
    // buttoned middle of its top, a carrying loop, origin on its underside: `sink_in` presses it to the seat height
    // (0.17), sat on cross-legged (a seat under 0.2 m: the floor clips). 2026-10-21: it was a 0.45 m pouf (M-packs-12).
    let P3 { x: bx, y: by, .. } = f.at(0, 0, 0);
    let zafu = new_parts(|| {
        let y0 = 0.08;
        let rings = [
            ring!(0.15, 0.15, z = y0, verts = 40, at = (bx, by)),
            fluted(0.18, y0 + 0.01, 40, (bx, by), 10, 0.03),
            fluted(0.19, y0 + 0.035, 40, (bx, by), 20, 0.035),
            fluted(0.192, y0 + 0.06, 40, (bx, by), 20, 0.035),
            fluted(0.186, y0 + 0.085, 40, (bx, by), 20, 0.03),
            fluted(0.15, y0 + 0.1, 40, (bx, by), 10, 0.05),
            fluted(0.08, y0 + 0.104, 40, (bx, by), 10, 0.08),
            ring!(0.02, 0.02, z = y0 + 0.098, verts = 40, at = (bx, by)),
        ];
        loft!("zafu", &rings, material = M.zafu);
        torus!(
            "zafu_piping",
            0.18,
            0.006,
            at = f.at(0, y0 + 0.095, 0),
            material = M.calm_deep,
            major_segments = 24,
            minor_segments = 3
        );
        fsphere!(
            f,
            "zafu_button",
            0.02,
            (0, y0 + 0.1, 0),
            M.calm_deep,
            scale = (1, 1, 0.45),
            segments = 10,
            rings = 5
        );
        kit::tube!(
            f,
            "zafu_loop",
            &[
                (-0.06, y0 + 0.07, 0.19),
                (0.0, y0 + 0.055, 0.205),
                (0.06, y0 + 0.07, 0.19),
            ],
            0.006,
            M.calm_deep,
            verts = 5,
            round = 2
        );
    });
    let zafu = hinged(join(&zafu, "zafu", None), f.at(0, 0.08, 0));
    let objs = settle_all(); // on its Model centre: the back on the wall face
    squash(
        zafu,
        "sink_in",
        (1.03, 1.03, 0.86),
        Squash {
            seconds: 0.6,
            ..Squash::default()
        },
    );
    objs
}

// --- the grudge-to-gratitude writing desk (1x1): the writer sits in the middle, the desk before them -------

/// A hairpin leg: one steel rod from under the top at depth z0, down to a point on the floor splayed `splay` out
/// in x, and back up to z1.
#[allow(clippy::too_many_arguments)]
fn hairpin(
    f: &GameFrame,
    name: &str,
    x: f64,
    z0: f64,
    z1: f64,
    top: f64,
    material: artkit::materials::Mat,
    splay: f64,
) -> Obj {
    let sx = splay.copysign(x);
    let zm = (z0 + z1) / 2.0;
    let pts = [
        (x, top, z0),
        (x + sx, 0.01, zm - 0.012),
        (x + sx, 0.004, zm),
        (x + sx, 0.01, zm + 0.012),
        (x, top, z1),
    ];
    fsweep!(
        f,
        name,
        &kit::fillet!(&pts, 0.012, steps = 2),
        0.0065,
        material,
        verts = 6
    )
}

pub fn grudge_journal() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            f,
            "top",
            0.84,
            0.03,
            0.3,
            (0, 0.73, 0.3),
            M.body,
            bevel = 0.008,
            bevel_segments = 2
        );
        for x in [-0.37, 0.37] {
            hairpin(
                f,
                &numbered("leg"),
                x,
                0.18,
                0.42,
                0.715,
                M.steel_dark,
                0.03,
            );
            fbox!(
                f,
                numbered("leg_plate"),
                0.05,
                0.006,
                0.27,
                (x, 0.712, 0.3),
                M.steel_dark
            );
        }
        // the drawer's case, hung under the top, open towards the writer
        fbox!(
            f,
            "case_top",
            0.36,
            0.012,
            0.28,
            (0.22, 0.709, 0.3),
            M.drawer
        );
        fbox!(
            f,
            "case_floor",
            0.36,
            0.012,
            0.28,
            (0.22, 0.561, 0.3),
            M.drawer,
            bevel = 0.003,
            bevel_segments = 1
        );
        for x in [0.045, 0.395] {
            fbox!(
                f,
                numbered("case_side"),
                0.012,
                0.16,
                0.28,
                (x, 0.635, 0.3),
                M.drawer,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "case_back",
            0.36,
            0.16,
            0.012,
            (0.22, 0.635, 0.434),
            M.drawer
        );
        // the open journal with its ribbon, the pen, crumpled pages, a waste bin, a ribbed cactus and a red desk lamp
        fbox!(
            f,
            "journal_cover",
            0.36,
            0.008,
            0.24,
            (-0.02, 0.749, 0.28),
            M.journal,
            bevel = 0.003,
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            let pts: Vec<(f64, f64, f64)> = (0..7)
                .map(|k| {
                    let lift = pow(sin(PI / 2.0 * (k as f64 / 3.0).min(1.0)), 0.6);
                    (
                        -0.02 + s * 0.165 * k as f64 / 6.0,
                        0.756 + 0.009 * lift - 0.002 * (k == 6) as i32 as f64,
                        0.28,
                    )
                })
                .collect();
            let prof = rounded_rect(0.008, 0.214, 0.003, 1);
            fsweep!(
                f,
                numbered("page"),
                &pts,
                0,
                M.paper,
                profile = prof,
                up = (0, 0, 1)
            );
        }
        fbox!(
            f,
            "ribbon",
            0.008,
            0.002,
            0.2,
            (-0.015, 0.765, 0.25),
            M.fire,
            rot = (0, 0, 4)
        );
        fsweep!(
            f,
            "pen",
            &[(0.13, 0.757, 0.21), (0.21, 0.757, 0.27)],
            0.006,
            M.dark,
            verts = 8,
            scale = [0.5, 1.0]
        );
        fbox!(
            f,
            "pen_clip",
            0.03,
            0.002,
            0.004,
            (0.19, 0.763, 0.256),
            M.steel,
            rot = (0, 0, -37)
        );
        for (k, (x, y, z, r)) in [
            (-0.3, 0.78, 0.22, 0.035),
            (-0.25, 0.775, 0.36, 0.03),
            (0.3, 0.035, 0.3, 0.035),
            (-0.36, 0.02, 0.08, 0.03),
        ]
        .into_iter()
        .enumerate()
        {
            let c = boulder!(
                numbered("crumple"),
                r,
                at = f.at(x, y - r, z),
                material = M.paper,
                seed = x * 10.0,
                noise = 0.35,
                subdivisions = 1
            );
            jitter!(c, r * 0.18, size = r * 0.6, seed = k as f64);
        }
        kit::turned!(
            f,
            "bin",
            -0.3,
            0.0,
            0.3,
            &[
                (0.0, 0.0),
                (0.085, 0.0),
                (0.1, 0.25),
                (0.106, 0.26),
                (0.094, 0.26),
                (0.08, 0.008),
                (0.0, 0.008),
            ],
            M.steel,
            segments = 20
        );
        boulder!(
            "bin_paper",
            0.05,
            at = f.at(-0.31, 0.21, 0.3),
            material = M.paper,
            seed = 3.0,
            noise = 0.35,
            subdivisions = 1
        );
        kit::turned!(
            f,
            "cactus_pot",
            0.3,
            0.745,
            0.38,
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.05, 0.07),
                (0.056, 0.072),
                (0.056, 0.082),
                (0.053, 0.085),
                (0.048, 0.085),
                (0.0, 0.08),
            ],
            M.ember,
            segments = 14
        );
        let P3 { x: cbx, y: cby, .. } = f.at(0.3, 0, 0.38);
        loft!(
            "cactus",
            &[
                fluted(0.03, 0.82, 48, (cbx, cby), 8, 0.12),
                fluted(0.037, 0.86, 48, (cbx, cby), 8, 0.12),
                fluted(0.036, 0.91, 48, (cbx, cby), 8, 0.12),
                fluted(0.026, 0.945, 48, (cbx, cby), 8, 0.12),
                fluted(0.01, 0.962, 48, (cbx, cby), 8, 0.1)
            ],
            material = M.cactus
        );
        fsphere!(
            f,
            "cactus_flower",
            0.011,
            (0.3, 0.965, 0.38),
            M.ember,
            scale = (1, 1, 0.6),
            segments = 8,
            rings = 4
        );
        // the lamp: a weighted foot, two arms jointed with knobs, a deep conical shade
        let (lx, lz) = (0.33, 0.18);
        kit::turned!(
            f,
            "lamp_foot",
            lx,
            0.745,
            lz,
            &[
                (0.0, 0.0),
                (0.055, 0.0),
                (0.058, 0.008),
                (0.05, 0.016),
                (0.012, 0.02),
                (0.0, 0.02),
            ],
            M.dark,
            segments = 16
        );
        kit::tube!(
            f,
            "lamp_arm",
            &[(lx, 0.76, lz), (0.3, 0.975, 0.24)],
            0.007,
            M.dark,
            verts = 6,
            round = 1
        );
        kit::tube!(
            f,
            "lamp_arm2",
            &[(0.3, 0.975, 0.24), (0.225, 0.96, 0.2)],
            0.007,
            M.dark,
            verts = 6,
            round = 1
        );
        for (jx, jy, jz) in [(lx, 0.765, lz), (0.3, 0.975, 0.24), (0.225, 0.96, 0.2)] {
            fcyl!(
                f,
                numbered("lamp_joint"),
                0.012,
                0.024,
                (jx, jy, jz),
                M.steel_dark,
                rot = (0, 90, 0),
                verts = 8
            );
        }
        kit::tube!(
            f,
            "lamp_spring",
            &[(lx + 0.012, 0.8, lz + 0.01), (0.312, 0.94, 0.232)],
            0.003,
            M.steel,
            verts = 4,
            round = 1
        );
        kit::turned!(
            f,
            "lamp_shade",
            0.2,
            0.905,
            0.19,
            &[
                (0.062, 0.0),
                (0.058, 0.004),
                (0.05, 0.022),
                (0.026, 0.078),
                (0.02, 0.088),
                (0.0, 0.09),
            ],
            M.fire,
            segments = 18,
            rot = (25.0, 0.0, -20.0)
        );
        // the chair, the writer's seat at 0.45: a padded seat, a curved back rail, tapered legs and stretchers
        fbox!(
            f,
            "chair_seat",
            0.42,
            0.04,
            0.4,
            (0, 0.405, 0.0),
            M.body,
            bevel = 0.01,
            bevel_segments = 2
        );
        soft!(
            f,
            "chair_pad",
            0.38,
            0.03,
            0.36,
            (0, 0.438, 0.01),
            M.calm,
            round = 0.014,
            puff = 0.004,
            flat = 2
        );
        for dx in [-0.18, 0.18] {
            fsweep!(
                f,
                numbered("front_leg"),
                &[(dx, 0.39, 0.16), (dx, 0.0, 0.165)],
                0,
                M.body,
                profile = rounded_rect(0.034, 0.034, 0.008, 1),
                up = (0, 0, 1),
                scale = [1.0, 0.72]
            );
            fsweep!(
                f,
                numbered("back_leg"),
                &[(dx, 0.0, -0.165), (dx, 0.4, -0.18), (dx, 0.9, -0.2)],
                0,
                M.body,
                profile = rounded_rect(0.034, 0.034, 0.008, 1),
                up = (0, 0, 1),
                resolution = 2,
                scale = [0.75, 1.0, 0.9]
            );
            kit::tube!(
                f,
                numbered("stretcher"),
                &[(dx, 0.14, 0.16), (dx, 0.14, -0.168)],
                0.008,
                M.body,
                verts = 6,
                round = 1
            );
        }
        let rail: Vec<(f64, f64, f64)> = [-0.2, -0.13, -0.065, 0.0, 0.065, 0.13, 0.2]
            .iter()
            .map(|&x| {
                (
                    x,
                    0.74 + 0.012 * (1.0 - pow(x / 0.2, 2.0)),
                    -0.185 - 0.02 * (1.0 - pow(x / 0.2, 2.0)),
                )
            })
            .collect();
        let back = fsweep!(
            f,
            "chair_back",
            &rail,
            0,
            M.calm,
            profile = rounded_rect(0.26, 0.04, 0.016, 2),
            up = (0, 0, 1)
        );
        ease_ends(back);
        let low: Vec<(f64, f64, f64)> = rail.iter().map(|&(x, y, z)| (x, y - 0.155, z)).collect();
        fsweep!(
            f,
            "chair_back_rail",
            &low,
            0,
            M.body,
            profile = rounded_rect(0.03, 0.022, 0.008, 1),
            up = (0, 0, 1)
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The drawer: its front and pull towards the writer, the tray and the letters in it; `drawer_open` slides it out
    // towards the writer.
    let drawer = new_parts(|| {
        fbox!(
            f,
            "drawer_front",
            0.34,
            0.13,
            0.016,
            (0.22, 0.635, 0.162),
            M.drawer,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "drawer_inset",
            0.3,
            0.09,
            0.004,
            (0.22, 0.635, 0.153),
            M.drawer,
            bevel = 0.002,
            bevel_segments = 1
        );
        let pts = [
            (0.16, 0.66, 0.153),
            (0.16, 0.66, 0.13),
            (0.28, 0.66, 0.13),
            (0.28, 0.66, 0.153),
        ]; // a bar pull towards the writer
        fsweep!(
            f,
            "drawer_pull",
            &kit::fillet!(&pts, 0.009),
            0.005,
            M.steel,
            verts = 6
        );
        fbox!(
            f,
            "drawer_tray",
            0.31,
            0.01,
            0.25,
            (0.22, 0.577, 0.29),
            M.drawer
        );
        for x in [0.07, 0.37] {
            fbox!(
                f,
                numbered("drawer_side"),
                0.008,
                0.08,
                0.25,
                (x, 0.615, 0.29),
                M.drawer
            );
        }
        fbox!(
            f,
            "drawer_back",
            0.31,
            0.08,
            0.008,
            (0.22, 0.615, 0.41),
            M.drawer
        );
        for (k, (dx, dz, r)) in [(-0.05, 0.25, 5), (0.04, 0.3, -8), (0.0, 0.34, 3)]
            .into_iter()
            .enumerate()
        {
            let k = k as f64;
            fbox!(
                f,
                numbered("letter"),
                0.16,
                0.012,
                0.11,
                (0.22 + dx, 0.588 + 0.012 * k, dz),
                M.paper,
                rot = (0, 0, r)
            );
            fbox!(
                f,
                numbered("letter_flap"),
                0.15,
                0.002,
                0.04,
                (0.22 + dx, 0.595 + 0.012 * k, dz - 0.03),
                M.journal,
                rot = (0, 0, r)
            );
        }
    });
    let drawer = join(&drawer, "drawer", None);
    let objs = settle!(centre = false);
    slide!(drawer, "drawer_open", (0, 0.17, 0), seconds = 0.6); // game -z: out towards the writer
    objs
}

// --- the glacier cold-plunge tub (1x2): the bather sits in it at the middle; the chiller at the back --------
const TUB: (f64, f64, f64) = (-0.55, 0.45, 0.62); // back depth, front depth, rim height

pub fn cold_plunge() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (zb, zf, h) = TUB;
    let (zc, d) = ((zb + zf) / 2.0, zf - zb);
    let mut rnd = Random::new(7);
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            0.86,
            0.06,
            d + 0.02,
            (0, 0.03, zc),
            M.hose,
            bevel = 0.01,
            bevel_segments = 1
        );
        // the liner: walls round the water, so the bather sits down into it
        fbox!(
            f,
            "liner_floor",
            0.72,
            0.04,
            d - 0.12,
            (0, 0.08, zc),
            M.shell_white
        );
        for x in [-0.395, 0.395] {
            fbox!(
                f,
                numbered("wall"),
                0.05,
                h - 0.06,
                d - 0.02,
                (x, 0.06 + (h - 0.06) / 2.0, zc),
                M.shell_white
            );
        }
        for z in [zb + 0.025, zf - 0.025] {
            fbox!(
                f,
                numbered("wall"),
                0.74,
                h - 0.06,
                0.05,
                (0, 0.06 + (h - 0.06) / 2.0, z),
                M.shell_white
            );
        }
        // cedar cladding, eight boards round the outside, each a hair proud or shy of the next; corner posts
        for i in 0..8 {
            let y = 0.1 + i as f64 * 0.065;
            let j = rnd.uniform(-0.0015, 0.0015);
            fbox!(
                f,
                numbered("board_side"),
                0.022,
                0.058,
                d - 0.03,
                (-0.43 + 0.011 + j, y, zc),
                M.cedar,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("board_side"),
                0.022,
                0.058,
                d - 0.03,
                (0.43 - 0.011 - j, y, zc),
                M.cedar,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("board_front"),
                0.81,
                0.058,
                0.022,
                (0, y, zf + 0.001 + j),
                M.cedar,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("board_back"),
                0.81,
                0.058,
                0.022,
                (0, y, zb - 0.001 - j),
                M.cedar,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for x in [-0.409, 0.409] {
            for z in [zb + 0.009, zf - 0.009] {
                fbox!(
                    f,
                    numbered("corner"),
                    0.042,
                    0.53,
                    0.042,
                    (x, 0.335, z),
                    M.cedar,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
        }
        // two steel bands round the cladding
        for y in [0.2, 0.5] {
            let lp: Vec<(f64, f64, f64)> = rounded_rect(0.865, 2.0 * (zf + 0.0145 - zc), 0.03, 2)
                .iter()
                .map(|&(u, v)| (u, y, v + zc))
                .collect();
            fsweep!(
                f,
                numbered("band"),
                &lp,
                0,
                M.steel,
                profile = rounded_rect(0.03, 0.004, 0.001, 1),
                up = (0, 0, 1),
                closed = true
            );
        }
        // the rim, one moulded piece round the top
        let lp: Vec<(f64, f64, f64)> = rounded_rect(0.8, d - 0.05, 0.04, 3)
            .iter()
            .map(|&(u, v)| (u, h + 0.005, v + zc))
            .collect();
        fsweep!(
            f,
            "rim",
            &lp,
            0,
            M.shell_white,
            profile = rounded_rect(0.03, 0.07, 0.01, 1),
            up = (0, 0, 1),
            closed = true
        );
        // the icy water and floating ice
        fbox!(f, "water", 0.72, 0.012, d - 0.1, (0, h - 0.03, zc), M.water);
        for (x, z, r) in [
            (-0.2, 0.15, 10),
            (0.15, -0.3, 40),
            (0.22, 0.25, -20),
            (-0.1, -0.2, 70),
            (0.05, 0.05, 15),
            (-0.25, -0.38, 30),
        ] {
            fbox!(
                f,
                numbered("ice"),
                0.06,
                0.045,
                0.06,
                (x, h - 0.022, z),
                M.ice,
                rot = (8, 5, r),
                bevel = 0.012,
                bevel_segments = 1
            );
        }
        // the cover's roller housing on the back rim, end caps, a slot for the cover
        fbox!(
            f,
            "housing",
            0.86,
            0.1,
            0.12,
            (0, h + 0.05, zb + 0.05),
            M.cedar,
            bevel = 0.012,
            bevel_segments = 2
        );
        for x in [-0.424, 0.424] {
            fcyl!(
                f,
                numbered("housing_cap"),
                0.048,
                0.012,
                (x, h + 0.05, zb + 0.05),
                M.hose,
                rot = (0, 90, 0),
                verts = 14
            );
        }
        fbox!(
            f,
            "housing_lip",
            0.8,
            0.02,
            0.01,
            (0, h + 0.03, zb + 0.112),
            M.dark
        );
        // the chiller behind: a slotted grille, a display with buttons, hoses arching into the tub; a towel and a bottle
        fbox!(
            f,
            "chiller",
            0.7,
            0.6,
            0.29,
            (0, 0.3, -0.78),
            M.drawer,
            bevel = 0.025,
            bevel_segments = 2
        );
        let slots: Vec<Vec<(f64, f64)>> = (0..6)
            .map(|k| {
                rounded_rect(0.52, 0.018, 0.009, 2)
                    .iter()
                    .map(|&(u, v)| (u, v - 0.13 + 0.052 * k as f64))
                    .collect()
            })
            .collect();
        fextrude!(
            f,
            "grille",
            &rounded_rect(0.6, 0.36, 0.012, 2),
            0.008,
            (0, 0.28, -0.634),
            M.steel,
            holes = slots
        );
        fbox!(
            f,
            "grille_back",
            0.58,
            0.34,
            0.004,
            (0, 0.28, -0.639),
            M.dark
        );
        fbox!(
            f,
            "display",
            0.16,
            0.07,
            0.008,
            (0.2, 0.52, -0.632),
            M.screen,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "readout",
            0.1,
            0.025,
            0.004,
            (0.2, 0.52, -0.627),
            M.readout
        );
        for k in 0..3 {
            fcyl!(
                f,
                numbered("button"),
                0.009,
                0.006,
                (-0.04 + 0.03 * k as f64, 0.52, -0.634),
                if k == 0 { M.readout } else { M.dark },
                rot = (90, 0, 0),
                verts = 8
            );
        }
        for x in [-0.22, 0.0] {
            kit::tube!(
                f,
                numbered("hose"),
                &kit::fillet!(
                    &[
                        (x, 0.5, -0.64),
                        (x, 0.66, -0.62),
                        (x, 0.66, zb - 0.03),
                        (x, 0.56, zb - 0.01),
                    ],
                    0.04
                ),
                0.022,
                M.hose,
                verts = 10,
                round = 1
            );
            fcyl!(
                f,
                numbered("hose_clamp"),
                0.027,
                0.014,
                (x, 0.5, -0.648),
                M.steel,
                rot = (90, 0, 0),
                verts = 10
            );
        }
        for sx in [-1.0, 1.0] {
            fbox!(
                f,
                numbered("chiller_foot"),
                0.06,
                0.012,
                0.25,
                (sx * 0.3, 0.006, -0.78),
                M.dark
            );
        }
        soft!(
            f,
            "towel",
            0.36,
            0.04,
            0.2,
            (0.17, 0.62, -0.79),
            M.towel,
            round = 0.016,
            puff = 0.006,
            flat = 2
        );
        fbox!(
            f,
            "towel_band",
            0.364,
            0.01,
            0.03,
            (0.17, 0.62, -0.72),
            M.cover
        );
        kit::lathe!(
            f,
            "bottle",
            -0.22,
            0.6,
            -0.79,
            &[
                (0.045, 0.0),
                (0.048, 0.006),
                (0.048, 0.17),
                (0.04, 0.195),
                (0.026, 0.21),
                (0.022, 0.24),
                (0.0, 0.24),
            ],
            M.calm,
            verts = 14
        );
        fcyl!(
            f,
            "bottle_cap",
            0.024,
            0.02,
            (-0.22, 0.85, -0.79),
            M.dark,
            verts = 10,
            bevel = 0.004,
            bevel_segments = 1
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The slatted cover, rolled out over the water from its housing, origin at the housing: `cover_off`
    // rolls it back in.
    let cz0 = zb + 0.11;
    let cover = new_parts(|| {
        let n = 14;
        let sl = (zf - 0.03 - cz0) / n as f64;
        for k in 0..n {
            fbox!(
                f,
                numbered("slat"),
                0.8,
                0.024,
                sl - 0.004,
                (0, h + 0.03, cz0 + sl * (k as f64 + 0.5)),
                M.cover,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "cover_pull",
            0.24,
            0.025,
            0.03,
            (0, h + 0.05, zf - 0.04),
            M.dark,
            bevel = 0.01,
            bevel_segments = 1
        );
    });
    let cover = join(&cover, "cover", Some(f.at(0, h + 0.03, cz0)));
    let objs = settle!(centre = false);
    squash(
        cover,
        "cover_off",
        (1, 0.04, 1),
        Squash {
            seconds: 0.9,
            ..Squash::default()
        },
    );
    objs
}
