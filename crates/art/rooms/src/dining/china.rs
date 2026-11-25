//! K9 continued: the china cabinet and the bar cart.

use super::common::{fillet, rail, slab, turned};
use super::storage::{DOOR_SECONDS, WALL, door, knob, plate};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{PI, sin};
use artkit::motion::{Ease, Squash, squash};
use artkit::parts::{hinged, join, new_parts, settle};
use artkit::scene::{Obj, meshes};
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, kit, loft, revolve, ring, settle, soft, swing,
};

/// A cove moulding along a cabinet's front and sides at height `y` (its foot), `w` across, its front `d` in
/// front of `z_back`, `depth` high: a bead and a cove flaring out under the cornice's cap, swept round the corners.
#[allow(clippy::too_many_arguments)]
pub(super) fn crown(
    name: &str,
    w: f64,
    d: f64,
    y: f64,
    z_back: f64,
    material: Mat,
    depth: f64,
) -> Obj {
    let prof = [
        (0.0, 0.0),
        (0.0, 0.008),
        (0.008, 0.011),
        (0.018, 0.019),
        (0.026, 0.03),
        (0.03, depth - 0.004), // its top's edges eased
        (0.026, depth),
        (0.004, depth),
        (0.0, depth - 0.004),
    ];
    let zf = z_back + d;
    let path = [
        (-w / 2.0, y, z_back),
        (-w / 2.0, y, zf),
        (w / 2.0, y, zf),
        (w / 2.0, y, z_back),
    ];
    // profile (u, v): u along the frame's normal, as near Blender +Z as it goes (up), v across (outward on the front)
    let profile: Vec<(f64, f64)> = prof.iter().map(|&(u, v)| (v, -u)).collect();
    fsweep!(
        F,
        name,
        &fillet(&path, 0.004, 1, false),
        0.0,
        material,
        profile = profile,
        up = (0, 0, 1),
        angle = 40
    )
}

/// China cabinet (catalog2): a cream two-door base with raised panels on a shaped plinth, a moulded worktop, a
/// glazed hutch with plates on show behind doors with glazing bars, a cornice of mouldings on top.
pub fn china_cabinet() -> Vec<Obj> {
    let (z_base, z_hutch) = (WALL + 0.24, WALL + 0.18);
    let still = new_parts(|| {
        let mut kick = vec![
            (-0.47, 0.06),
            (0.47, 0.06),
            (0.47, 0.0),
            (0.4, 0.0),
            (0.38, 0.02),
        ];
        kick.extend((0..11).map(|i| {
            let i = f64::from(i);
            (0.34 - 0.68 * i / 10.0, 0.02 + 0.012 * sin(PI * i / 10.0))
        }));
        kick.extend([(-0.38, 0.02), (-0.4, 0.0), (-0.47, 0.0)]);
        fextrude!(
            F,
            "plinth",
            &kick,
            0.46,
            (0, 0, z_base - 0.01),
            M.plinth,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "base",
            0.94,
            0.8,
            0.48,
            (0, 0.46, z_base),
            M.body_china,
            bevel = 0.012
        );
        slab(
            "worktop",
            0.98,
            0.51,
            0.9,
            0.04,
            M.top_china,
            0.01,
            None,
            3,
            (0.0, z_base + 0.015),
        );
        for s in [-1, 1] {
            // two base doors with brass knobs
            let sf = f64::from(s);
            fextrude!(
                F,
                format!("door{s}"),
                &rounded_rect(0.45, 0.7, 0.006, 1),
                0.02,
                (sf * 0.235, 0.46, z_base + 0.25),
                M.body_china,
                bevel = 0.005,
                bevel_segments = 1
            );
            fextrude!(
                F,
                format!("door_panel{s}"),
                &rounded_rect(0.35, 0.58, 0.005, 1),
                0.012,
                (sf * 0.235, 0.46, z_base + 0.262),
                M.body_china,
                bevel = 0.005,
                bevel_segments = 2
            );
            knob(
                &format!("knob{s}"),
                sf * 0.035,
                0.6,
                z_base + 0.26,
                M.brass,
                0.016,
                None,
            );
        }
        // the hutch: sides, top, back, shelves of plates
        let (hy0, hy1) = (0.9, 2.04);
        // the back: blue behind the plates, the body's colour outside (seen when it stands free)
        fbox!(
            F,
            "hutch_back",
            0.88,
            hy1 - hy0,
            0.01,
            (0, (hy0 + hy1) / 2.0, WALL + 0.015),
            M.back_blue
        );
        fbox!(
            F,
            "hutch_back_out",
            0.88,
            hy1 - hy0,
            0.01,
            (0, (hy0 + hy1) / 2.0, WALL + 0.005),
            M.body_china
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("hutch_side{s}"),
                0.03,
                hy1 - hy0,
                0.36,
                (f64::from(s) * 0.445, (hy0 + hy1) / 2.0, z_hutch),
                M.body_china,
                bevel = 0.006
            );
        }
        // the cornice: a cap with a moulded edge over a cove moulding round the front and sides
        slab(
            "crown",
            0.98,
            0.42,
            2.1,
            0.06,
            M.top_china,
            0.006,
            Some(&[
                (0.004, 0.0),
                (0.0, 0.2),
                (0.0, 0.45),
                (0.008, 0.6),
                (0.008, 0.75),
                (0.0, 0.85),
                (0.003, 1.0),
            ]),
            3,
            (0.0, z_hutch + 0.01),
        );
        crown("cove", 0.89, 0.4, 1.995, WALL - 0.02, M.top_china, 0.045);
        for (k, y) in [1.04, 1.39, 1.73].into_iter().enumerate() {
            fbox!(
                F,
                format!("shelf{k}"),
                0.86,
                0.02,
                0.32,
                (0, y, z_hutch - 0.01),
                M.body_china,
                bevel = 0.004
            );
            for (j, x) in [-0.3, -0.1, 0.1, 0.3].into_iter().enumerate() {
                let m = if (j + k) % 2 == 0 {
                    M.porcelain
                } else {
                    M.plate_blue
                };
                plate(
                    &format!("plate{k}{j}"),
                    x,
                    y + 0.095,
                    WALL + 0.06,
                    0.085,
                    m,
                    14,
                    Some((78, 0, 0).into()),
                );
            }
            fbox!(
                F,
                format!("rail{k}"),
                0.86,
                0.012,
                0.01,
                (0, y + 0.04, WALL + 0.11),
                M.body_china
            );
        }
        // cups on the lowest shelf, before the plates
        for (j, x) in [-0.2, 0.0, 0.2].into_iter().enumerate() {
            turned(
                &format!("cup{j}"),
                &[
                    (0.0, 0.0),
                    (0.022, 0.0),
                    (0.024, 0.004),
                    (0.034, 0.05),
                    (0.031, 0.05),
                    (0.02, 0.008),
                    (0.0, 0.008),
                ],
                (x, 1.05, WALL + 0.21),
                if j % 2 != 0 {
                    M.porcelain
                } else {
                    M.plate_blue
                },
                12,
                35.0,
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let zf = z_hutch + 0.18;
    let glazing = |x0: f64, x1: f64, y0: f64, y1: f64, zf: f64| {
        let xm = (x0 + x1) / 2.0;
        let x0s = float_repr(x0);
        fbox!(
            F,
            format!("pane{x0s}"),
            x1 - x0 - 0.06,
            y1 - y0 - 0.06,
            0.006,
            (xm, (y0 + y1) / 2.0, zf - 0.01),
            M.glass
        );
        // glazing bars: one upright and two across, dividing the glass into six lights
        fbox!(
            F,
            format!("bar_v{x0s}"),
            0.016,
            y1 - y0 - 0.08,
            0.012,
            (xm, (y0 + y1) / 2.0, zf - 0.01),
            M.body_china,
            bevel = 0.003
        );
        for dy in [-0.115, 0.17] {
            fbox!(
                F,
                format!("bar{x0s}{}", float_repr(dy)),
                x1 - x0 - 0.06,
                0.016,
                0.012,
                (xm, (y0 + y1) / 2.0 + dy, zf - 0.01),
                M.body_china,
                bevel = 0.003
            );
        }
    };
    let mut glazed_doors = Vec::new();
    for (s, (x0, x1)) in [(-1, (-0.43, -0.005)), (1, (0.005, 0.43))] {
        let glazed = |a: f64, b: f64, c: f64, e: f64, z: f64| {
            glazing(a, b, c, e, z);
            knob(
                &format!("knob_g{s}"),
                f64::from(s) * 0.04,
                1.45,
                z - 0.002,
                M.brass,
                0.013,
                None,
            );
        };
        glazed_doors.push(door(
            &format!("glass_door{s}"),
            x0,
            x1,
            0.93,
            2.0,
            zf,
            M.body_china,
            s < 0,
            Some(&glazed),
            0.045,
        ));
    }
    settle(&meshes()); // where the game places it, against the wall (its Model's centre)
    // the left glazed door (hinged on its left) swings open onto the plates while clean things are put away
    swing!(
        glazed_doors[0],
        "door_open",
        100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    meshes()
}

/// A bottle standing at (x, y, z): a round body, a shoulder, a long neck and a cork; `label`: a paper band.
#[allow(clippy::too_many_arguments)]
pub(super) fn bottle(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    cap: Mat,
    label: Option<Mat>,
) {
    let verts = 12;
    let at = F.at(x, y, z);
    let (bx, by, bz) = (at.x, at.y, at.z);
    let rings = [
        (0.0, 0.85),
        (0.02, 1.0),
        (0.55, 1.0),
        (0.66, 0.78),
        (0.74, 0.4),
        (0.93, 0.36),
        (0.95, 0.42),
        (0.97, 0.36),
    ];
    let sections: Vec<_> = rings
        .iter()
        .map(|&(t, k)| ring!(r * k, r * k, z = bz + h * t, verts = verts, at = (bx, by)))
        .collect();
    loft!(name, &sections, material = material);
    fcyl!(
        F,
        format!("{name}_cork"),
        r * 0.34,
        0.03,
        (x, y + h + 0.01, z),
        cap,
        verts = 8
    );
    if let Some(label) = label {
        fcyl!(
            F,
            format!("{name}_label"),
            r * 1.02,
            h * 0.22,
            (x, y + h * 0.3, z),
            label,
            verts = verts
        );
    }
}

/// Bar cart (catalog2): two glass tiers in brass frames on castors, a handle bent from one tube, bottles, glasses,
/// an ice bucket and a basket; the shaker is taken off it while a resident mixes a drink.
pub fn bar_cart() -> Vec<Obj> {
    let still = new_parts(|| {
        for (k, y) in [0.3, 0.76].into_iter().enumerate() {
            slab(
                &format!("tray{k}"),
                0.68,
                0.38,
                y + 0.006,
                0.012,
                M.tray,
                0.02,
                Some(&[(0.002, 0.0), (0.0, 0.3), (0.0, 0.7), (0.002, 1.0)]),
                3,
                (0.0, 0.0),
            );
            // the gallery: a rounded rectangle of brass tube round the glass, standing a little above it
            rail(
                &format!("rim{k}"),
                &[
                    (-0.35, y + 0.012, -0.2),
                    (0.35, y + 0.012, -0.2),
                    (0.35, y + 0.012, 0.2),
                    (-0.35, y + 0.012, 0.2),
                ],
                0.011,
                M.frame_brass,
                0.03,
                8,
                3,
                true,
            );
        }
        for x in [-0.35, 0.35] {
            for z in [-0.2, 0.2] {
                let at = format!("{}{}", float_repr(x), float_repr(z));
                kit::tube!(
                    F,
                    format!("post{at}"),
                    &[(x, 0.06, z), (x, 0.8, z)],
                    0.011,
                    M.frame_brass,
                    verts = 10
                );
                turned(
                    &format!("finial{at}"),
                    &[
                        (0.0, 0.0),
                        (0.012, 0.0),
                        (0.017, 0.008),
                        (0.018, 0.016),
                        (0.012, 0.026),
                        (0.006, 0.03),
                        (0.0, 0.031),
                    ],
                    (x, 0.792, z),
                    M.frame_brass,
                    10,
                    35.0,
                );
                // the castor: a swivel fork in brass, a black wheel on an axle
                turned(
                    &format!("socket{at}"),
                    &[
                        (0.0, 0.0),
                        (0.016, 0.0),
                        (0.018, 0.006),
                        (0.014, 0.02),
                        (0.0, 0.02),
                    ],
                    (x, 0.048, z),
                    M.frame_brass,
                    10,
                    35.0,
                );
                fbox!(
                    F,
                    format!("fork{at}"),
                    0.03,
                    0.022,
                    0.04,
                    (x, 0.04, z),
                    M.frame_brass,
                    bevel = 0.006,
                    bevel_segments = 1
                );
                revolve!(
                    format!("castor{at}"),
                    &[
                        (0.0, -0.008),
                        (0.018, -0.009),
                        (0.024, -0.007),
                        (0.025, 0.0),
                        (0.024, 0.007),
                        (0.018, 0.009),
                        (0.0, 0.008)
                    ],
                    at = F.at(x, 0.025, z),
                    material = M.castor,
                    segments = 14,
                    rot = (0, 90, 0)
                );
            }
        }
        // a gallery rail round the lower tier on short balusters, so nothing rolls off
        rail(
            "gallery",
            &[
                (-0.35, 0.345, -0.2),
                (0.35, 0.345, -0.2),
                (0.35, 0.345, 0.2),
                (-0.35, 0.345, 0.2),
            ],
            0.006,
            M.frame_brass,
            0.03,
            6,
            3,
            true,
        );
        for x in [-0.12, 0.12] {
            for z in [-0.2, 0.2] {
                kit::tube!(
                    F,
                    format!("baluster{}{}", float_repr(x), float_repr(z)),
                    &[(x, 0.312, z), (x, 0.345, z)],
                    0.005,
                    M.frame_brass,
                    verts = 6
                );
            }
        }
        // the handle: bent from one tube, out from the posts and across
        rail(
            "handle",
            &[
                (-0.35, 0.78, -0.2),
                (-0.395, 0.84, -0.2),
                (-0.395, 0.84, 0.2),
                (-0.35, 0.78, 0.2),
            ],
            0.011,
            M.frame_brass,
            0.03,
            8,
            3,
            false,
        );
        // bottles on the top tier, labelled
        for (i, (x, m)) in [
            (-0.24, M.bottle_green),
            (-0.15, M.bottle_red),
            (-0.06, M.bottle_amber),
        ]
        .into_iter()
        .enumerate()
        {
            bottle(
                &format!("bottle{i}"),
                x,
                0.766,
                -0.08,
                0.035,
                0.3,
                m,
                M.basket,
                Some(M.porcelain),
            );
        }
        // two wine glasses
        for (i, (x, z)) in [(0.2, 0.1), (0.28, 0.0)].into_iter().enumerate() {
            turned(
                &format!("wine_glass{i}"),
                &[
                    (0.0, 0.0),
                    (0.03, 0.0),
                    (0.03, 0.004),
                    (0.006, 0.01),
                    (0.004, 0.07),
                    (0.012, 0.08),
                    (0.033, 0.11),
                    (0.04, 0.16),
                    (0.036, 0.16),
                    (0.03, 0.12),
                    (0.012, 0.09),
                    (0.0, 0.088),
                ],
                (x, 0.766, z),
                M.glass,
                12,
                35.0,
            );
        }
        // a lemon dish on the top tier; an ice bucket with tongs and a woven basket on the lower one
        turned(
            "dish",
            &crate::common::eased(
                &[
                    (0.0, 0.0),
                    (0.035, 0.0),
                    (0.055, 0.012),
                    (0.062, 0.016),
                    (0.056, 0.016),
                    (0.035, 0.006),
                    (0.0, 0.006),
                ],
                0.002,
            ),
            (0.1, 0.766, -0.1),
            M.porcelain,
            16,
            35.0,
        );
        for (i, (dx, dz)) in [(0.0, 0.0), (0.035, 0.025)].into_iter().enumerate() {
            fsphere!(
                F,
                format!("lemon{i}"),
                0.025,
                (0.1 + dx, 0.8, -0.1 + dz),
                M.lemon,
                scale = (1, 1, 0.85),
                segments = 10,
                rings = 6
            );
        }
        turned(
            "ice_bucket",
            &[
                (0.0, 0.0),
                (0.065, 0.0),
                (0.068, 0.008),
                (0.08, 0.14),
                (0.084, 0.15),
                (0.075, 0.15),
                (0.064, 0.02),
                (0.0, 0.02),
            ],
            (0.0, 0.306, 0.0),
            M.shaker,
            18,
            35.0,
        );
        for s in [-1, 1] {
            fcyl!(
                F,
                format!("bucket_ear{s}"),
                0.014,
                0.008,
                (f64::from(s) * 0.083, 0.43, 0.0),
                M.shaker,
                rot = (0, 90, 0),
                verts = 8
            );
        }
        for i in 0..5 {
            // ice in the bucket
            fbox!(
                F,
                format!("ice{i}"),
                0.03,
                0.03,
                0.03,
                (
                    -0.03 + 0.015 * f64::from(i),
                    0.43 + 0.01 * f64::from(i % 2),
                    -0.02 + 0.01 * f64::from(i % 3)
                ),
                M.tray,
                rot = (20 * i, 10 * i, 30 * i),
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        soft!(
            F,
            "basket",
            0.18,
            0.1,
            0.16,
            (0.2, 0.356, 0.05),
            M.basket,
            round = 0.02,
            puff = (0.006, 0.0, 0.006)
        );
        rail(
            "basket_rim",
            &[
                (0.11, 0.405, -0.03),
                (0.29, 0.405, -0.03),
                (0.29, 0.405, 0.13),
                (0.11, 0.405, 0.13),
            ],
            0.006,
            M.basket,
            0.02,
            5,
            3,
            true,
        );
        rail(
            "basket_handle",
            &[
                (0.2, 0.405, -0.03),
                (0.2, 0.46, -0.03),
                (0.2, 0.46, 0.13),
                (0.2, 0.405, 0.13),
            ],
            0.005,
            M.basket,
            0.04,
            5,
            3,
            false,
        );
    });
    join(&still, "cart", Some((0, 0, 0).into()));
    // the cocktail shaker, standing at the front of the top tier, in the middle, before the resident who
    // mixes from the front: its own part, its origin at its foot, squashed away while it is in their hands
    let (sx, sz) = (0.0, 0.1);
    let shake = new_parts(|| {
        turned(
            "shaker",
            &[
                (0.0, 0.0),
                (0.034, 0.0),
                (0.036, 0.004),
                (0.04, 0.12),
                (0.042, 0.124),
                (0.037, 0.13),
                (0.03, 0.17),
                (0.032, 0.174),
                (0.018, 0.2),
                (0.012, 0.212),
                (0.014, 0.215),
                (0.0, 0.22),
            ],
            (sx, 0.766, sz),
            M.shaker,
            16,
            35.0,
        );
    });
    let shaker = hinged(join(&shake, "shaker", None), F.at(sx, 0.766, sz));
    settle!(centre = false);
    squash(
        shaker,
        "shake",
        0.001,
        Squash {
            seconds: 0.2,
            ..Squash::default()
        },
    );
    meshes()
}
