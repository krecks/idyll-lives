//! KD2: the giant teddy bear and the kids' bean bag.

use super::common::{P, arc, board, place, rumple, slab};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, TAU, cos, pow, sin};
use artkit::motion::{Squash, squash};
use artkit::parts::join;
use artkit::py::hypot;
use artkit::scene::{Obj, meshes};
use artkit::{fsphere, fsweep, loft, ring};

/// A body lofted up through `rings` [(height, half width, half depth)], round the upright at game depth `z`.
fn upright(name: &str, rings: &[(f64, f64, f64)], material: Mat, z: f64, n: usize) -> Obj {
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(y, rx, rz)| ring!(rx, rz, z = y, verts = n, at = (0.0, -z)))
        .collect();
    loft!(name, &sections, material = material)
}

pub fn teddy_bear() -> Vec<Obj> {
    // the body: a pear sitting on the floor; the head: round, fuller at the cheeks
    upright(
        "body",
        &[
            (0.02, 0.004, 0.004),
            (0.03, 0.14, 0.12),
            (0.08, 0.215, 0.18),
            (0.2, 0.248, 0.21),
            (0.32, 0.24, 0.2),
            (0.42, 0.2, 0.17),
            (0.5, 0.14, 0.12),
            (0.54, 0.06, 0.05),
            (0.55, 0.004, 0.004),
        ],
        M.fur,
        -0.02,
        16,
    );
    fsphere!(
        F,
        "belly",
        0.15,
        (0, 0.27, 0.15),
        M.fur_light,
        scale = (0.95, 0.4, 0.95),
        segments = 12,
        rings = 7
    );
    upright(
        "head",
        &[
            (0.5, 0.004, 0.004),
            (0.51, 0.09, 0.08),
            (0.55, 0.16, 0.14),
            (0.62, 0.2, 0.18),
            (0.69, 0.205, 0.185),
            (0.76, 0.185, 0.165),
            (0.82, 0.14, 0.125),
            (0.86, 0.07, 0.06),
            (0.87, 0.004, 0.004),
        ],
        M.fur,
        0.0,
        12,
    );
    fsphere!(
        F,
        "muzzle",
        0.08,
        (0, 0.635, 0.155),
        M.fur_light,
        scale = (1, 0.75, 0.72),
        segments = 12,
        rings = 8
    );
    fsphere!(
        F,
        "nose",
        0.026,
        (0, 0.668, 0.21),
        M.eye,
        scale = (1.3, 0.8, 0.85),
        segments = 10,
        rings = 6
    );
    for s in [-1, 1] {
        let sf = s as f64;
        // the mouth stitched under the nose
        fsweep!(
            F,
            format!("mouth{s}"),
            &[
                (0.0, 0.648, 0.214),
                (0.0, 0.628, 0.212),
                (sf * 0.016, 0.62, 0.207),
                (sf * 0.03, 0.628, 0.198)
            ],
            0.0025,
            M.eye,
            verts = 4,
            resolution = 2
        );
        fsphere!(
            F,
            format!("eye{s}"),
            0.02,
            (sf * 0.075, 0.73, 0.163),
            M.eye,
            segments = 8,
            rings = 5
        );
        fsphere!(
            F,
            format!("ear{s}"),
            0.07,
            (sf * 0.15, 0.85, -0.02),
            M.fur,
            scale = (1, 0.55, 1),
            segments = 10,
            rings = 6
        );
        fsphere!(
            F,
            format!("ear_in{s}"),
            0.045,
            (sf * 0.15, 0.85, 0.0),
            M.fur_light,
            scale = (1, 0.3, 1),
            segments = 8,
            rings = 5
        );
        // the arms reaching forward, a paw pad on each
        fsweep!(
            F,
            format!("arm{s}"),
            &[
                (sf * 0.17, 0.46, 0.02),
                (sf * 0.235, 0.36, 0.06),
                (sf * 0.255, 0.28, 0.12)
            ],
            0.068,
            M.fur,
            verts = 10,
            resolution = 2,
            scale = [0.95, 0.92, 0.85]
        );
        fsphere!(
            F,
            format!("paw{s}"),
            0.058,
            (sf * 0.26, 0.27, 0.13),
            M.fur,
            segments = 10,
            rings = 7
        );
        fsphere!(
            F,
            format!("paw_pad{s}"),
            0.036,
            (sf * 0.245, 0.25, 0.17),
            M.fur_light,
            scale = (0.9, 0.4, 0.9),
            rot = (-30, 0, s * 20),
            segments = 8,
            rings = 5
        );
        // the legs out in front, a pad on each foot
        fsweep!(
            F,
            format!("leg{s}"),
            &[
                (sf * 0.13, 0.1, 0.0),
                (sf * 0.14, 0.088, 0.18),
                (sf * 0.145, 0.086, 0.3)
            ],
            0.085,
            M.fur,
            verts = 10,
            resolution = 2,
            scale = [1.0, 0.95, 0.9]
        );
        fsphere!(
            F,
            format!("foot{s}"),
            0.077,
            (sf * 0.145, 0.086, 0.3),
            M.fur,
            scale = (1, 0.45, 1),
            segments = 10,
            rings = 7
        );
        fsphere!(
            F,
            format!("pad{s}"),
            0.06,
            (sf * 0.145, 0.09, 0.33),
            M.fur_light,
            scale = (1, 0.25, 1.1),
            segments = 10,
            rings = 5
        );
        // the bow at the neck: two loops of ribbon and their tails
        let lp: Vec<P> = (0..10)
            .map(|i| TAU * i as f64 / 10.0)
            .map(|a| {
                (
                    sf * (0.012 + 0.07 * (1.0 - cos(a)) / 2.0),
                    0.505 + 0.032 * sin(a),
                    0.19 - 0.012 * (1.0 - cos(a)) / 2.0,
                )
            })
            .collect();
        fsweep!(
            F,
            format!("bow{s}"),
            &lp,
            0,
            M.bow,
            profile = rounded_rect(0.006, 0.026, 0.002, 1),
            up = (0, 0, 1),
            closed = true
        );
        fsweep!(
            F,
            format!("bow_tail{s}"),
            &[
                (sf * 0.008, 0.5, 0.19),
                (sf * 0.03, 0.46, 0.195),
                (sf * 0.045, 0.42, 0.185)
            ],
            0,
            M.bow,
            profile = rounded_rect(0.005, 0.022, 0.002, 1),
            up = (0, 0, 1),
            resolution = 2
        );
    }
    fsphere!(
        F,
        "knot",
        0.02,
        (0, 0.505, 0.193),
        M.bow,
        scale = (1, 0.8, 1.1),
        segments = 10,
        rings = 6
    );
    let bear = join(&meshes(), "bear", Some(P3::ZERO));
    place(0.0);
    squash(
        bear,
        "squish",
        (1.06, 1.06, 0.9),
        Squash {
            seconds: 0.5,
            ..Squash::default()
        },
    );
    meshes()
}

pub fn bean_bag_kids() -> Vec<Obj> {
    // a sack lofted up from the floor, leaning back into its back rest, a seat pressed into its front where the game
    // sits a child (0.24, furniture.json `seats`), creased all over; a handle on its back, a label on its front
    let rings = [
        (0.0, 0.004, 0.004, 0.03),
        (0.004, 0.25, 0.26, 0.03),
        (0.05, 0.305, 0.315, 0.025),
        (0.13, 0.31, 0.315, 0.02),
        (0.2, 0.3, 0.3, 0.012),
        (0.26, 0.28, 0.27, -0.005),
        (0.32, 0.255, 0.22, -0.05),
        (0.38, 0.235, 0.18, -0.09),
        (0.44, 0.205, 0.145, -0.125),
        (0.49, 0.155, 0.11, -0.15),
        (0.515, 0.09, 0.07, -0.16),
        (0.528, 0.004, 0.004, -0.165),
    ];
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(y, rx, rz, z)| ring!(rx, rz, z = y, verts = 24, at = (0.0, -z)))
        .collect();
    let bag = loft!("bag", &sections, material = M.bag);
    bag.edit(|o| {
        for v in o.mesh.verts.iter_mut() {
            let (gx, gz, gy) = (-(v.x as f64), -(v.y as f64), v.z as f64);
            let d = hypot(gx / 0.24, (gz - 0.07) / 0.2);
            if gy > 0.12 && d < 1.0 {
                let k = (gy - 0.12) / 0.12;
                v.z = (v.z as f64 - 0.17 * pow(1.0 - d * d, 2.0) * if k < 1.0 { k } else { 1.0 })
                    as f32;
            }
        }
    });
    rumple(bag, 0.014, 0.03, 7.0, 0.08);
    // (its cloth laid on from the sides, not round it: the seat pressed in stretches a turned layout)
    bag.edit(|o| o.uv = artkit::mesh::UvWay::Box); // up and down only: its bounds (the size and centre checks) stay put
    slab(
        "tag",
        &arc(0.045, 0.0, 330.0, 11, 0.0, 0.0),
        &board((0.0, 0.2, 0.315), 20.0),
        M.bag_patch,
        -0.004,
        0.006,
    );
    fsweep!(
        F,
        "handle",
        &[
            (-0.05, 0.47, -0.27),
            (-0.03, 0.495, -0.29),
            (0.03, 0.495, -0.29),
            (0.05, 0.47, -0.27)
        ],
        0,
        M.bag_patch,
        profile = rounded_rect(0.006, 0.03, 0.002, 1),
        up = (0, 0, 1),
        resolution = 2
    );
    let bag = join(&meshes(), "bag", Some(P3::ZERO));
    place(0.0);
    squash(
        bag,
        "squash",
        (1.04, 1.04, 0.85),
        Squash {
            seconds: 0.5,
            ..Squash::default()
        },
    );
    meshes()
}
