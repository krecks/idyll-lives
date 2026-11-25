//! The raised veggie bed (2x1): a cedar box of lettuces, carrots, a courgette and staked tomatoes.

use super::common::{_flower, _rosette, M};
use artkit::forms::{rect, rounded_rect};
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, cos, sin};
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{bevel::ease, shapes::AUTO_WIDTH};
use artkit::{
    fbox, fsphere, fsweep, jitter, leaf, loft, revolve, ring, rock, soft, squash, stages, star,
    sweep,
};

const H: f64 = 0.45; // the box's height
const SOIL: f64 = H - 0.03;
const TOMATOES: [f64; 2] = [0.3, 0.68]; // the staked plants' x, at depth -0.2

/// Two boards a side between corner posts with pyramid caps, a mitred cap rail round the top, and the soil, mounded
/// and uneven; two stakes for the tomatoes and a seed packet on a stick.
fn _bed_box(f: &GameFrame) {
    for z in [-0.41, 0.41] {
        for y in [0.11, 0.33] {
            fbox!(
                f,
                format!("board{}{}", float_repr(z), float_repr(y)),
                1.8,
                0.205,
                0.04,
                (0, y, z),
                M.cedar,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
    }
    for x in [-0.89, 0.89] {
        for y in [0.11, 0.33] {
            fbox!(
                f,
                format!("end{}{}", float_repr(x), float_repr(y)),
                0.04,
                0.205,
                0.78,
                (x, y, 0),
                M.cedar,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for z in [-0.41, 0.41] {
            let (px, pz) = (x * 0.99, z * 0.99);
            fbox!(
                f,
                format!("post{}{}", float_repr(x), float_repr(z)),
                0.07,
                H + 0.02,
                0.07,
                (px, (H + 0.02) / 2.0, pz),
                M.cedar_dark,
                bevel = 0.008,
                bevel_segments = 1
            );
            let P3 {
                x: bx,
                y: by,
                z: bz,
            } = f.at(px, H + 0.02, pz);
            loft!(
                format!("post_cap{}{}", float_repr(x), float_repr(z)),
                &[
                    ring!(0.04, 0.04, z = bz, verts = 4, square = 2, at = (bx, by)),
                    ring!(0.04, 0.04, z = bz + 0.008, verts = 4, at = (bx, by)),
                    ring!(0.004, 0.004, z = bz + 0.022, verts = 4, at = (bx, by))
                ],
                material = M.cedar_dark,
                smooth = false,
                rot = (0, 0, 0)
            );
        }
    }
    let cap = [
        (0.0, 0.0),
        (0.06, 0.0),
        (0.06, 0.026),
        (0.052, 0.035),
        (0.008, 0.035),
        (0.0, 0.026),
    ]; // (in, up)
    kit::moulding!(
        f,
        "cap_rail",
        &[
            (-0.935, -0.455),
            (0.935, -0.455),
            (0.935, 0.455),
            (-0.935, 0.455)
        ],
        &cap,
        M.cedar_dark,
        at = H - 0.0175,
        plane = Facing::Top
    );
    let soil = soft!(
        f,
        "soil",
        1.74,
        0.03,
        0.78,
        (0, SOIL - 0.015, 0),
        M.soil,
        round = 0.01,
        puff = (0, 0.012, 0),
        flat = 6,
        round_segments = 1
    );
    jitter!(soil, 0.006, size = 0.08, seed = 2.0, up = 1.0);
    for x in TOMATOES {
        // the stakes, tapering to their tops
        ease(
            fsweep!(
                f,
                format!("stake{}", float_repr(x)),
                &[(x, SOIL - 0.02, -0.22), (x, H + 0.85, -0.22)],
                0.0,
                M.cedar_dark,
                profile = rounded_rect(0.022, 0.022, AUTO_WIDTH, 1),
                up = (0, 1, 0),
                scale = [1.0, 0.7]
            ),
            60.0,
            1,
        );
    }
    fsweep!(
        f,
        "label_stick",
        &[(-0.85, SOIL - 0.02, 0.36), (-0.85, H + 0.02, 0.36)],
        0.0,
        M.label,
        profile = rect(0.012, 0.006),
        up = (0, 1, 0)
    );
    fbox!(
        f,
        "seed_packet",
        0.08,
        0.12,
        0.006,
        (-0.85, H + 0.08, 0.364),
        M.label,
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        f,
        "packet_picture",
        0.06,
        0.05,
        0.002,
        (-0.85, H + 0.09, 0.368),
        M.tomato
    );
}

/// A lettuce: cupped leaves round a tight heart.
fn _lettuce(f: &GameFrame, name: &str, x: f64, z: f64, y: f64, mats: &[Mat], seed: i128) {
    _rosette(
        f,
        name,
        x,
        y,
        z,
        12,
        0.14,
        0.12,
        mats,
        (20.0, 62.0),
        0.3,
        "oval",
        0.5,
        None,
        3,
        4,
        seed,
        0.0,
    );
    fsphere!(
        f,
        format!("{name}heart"),
        0.035,
        (x, y + 0.035, z),
        mats[0],
        scale = (1, 1, 0.9),
        segments = 8,
        rings = 5
    );
}

/// A tomato plant climbing its stake: a stem winding up it, side shoots and toothed leaves all the way.
fn _tomato_plant(f: &GameFrame, x: f64, z: f64) {
    let mut rnd = Random::new(artkit::py::int(x * 100.0) as i128);
    let pts: Vec<(f64, f64, f64)> = (0..7)
        .map(|k| {
            let k = k as f64;
            (
                x + 0.03 * cos(k * 1.4),
                SOIL + k * 0.14,
                z + 0.03 * sin(k * 1.4),
            )
        })
        .collect();
    fsweep!(
        f,
        format!("tomato_stem{}", float_repr(x)),
        &pts,
        0.008,
        M.leaf_dark,
        verts = 5,
        resolution = 2,
        scale = [1.0, 0.6]
    );
    for k in 0..18 {
        let t = (k + 1) as f64 / 19.0;
        let a = k as f64 * 137.5 + rnd.uniform(-20.0, 20.0);
        let base = f.at(
            x + 0.03 * cos(t * 6.0 * 1.4),
            SOIL + 0.06 + t * 0.78,
            z + 0.03 * sin(t * 6.0 * 1.4),
        );
        let tilt = rnd.uniform(10.0, 35.0);
        leaf!(
            format!("tomato_leaf{}_{k}", float_repr(x)),
            base,
            a,
            tilt,
            0.2 * (1.1 - 0.4 * t),
            0.12,
            M.leaf_dark,
            shape = "lance",
            droop = 0.4,
            fold = 0.2,
            rows = 5,
            ring = 3,
            notch = (2usize, 0.5)
        );
    }
}

/// A courgette's big lobed leaves on long stalks, sprawling over the bed's front right.
fn _courgette_plant(f: &GameFrame) {
    for (k, (x, z, a)) in [
        (0.42, 0.14, 150),
        (0.54, 0.2, 200),
        (0.66, 0.12, 120),
        (0.5, 0.06, 60),
        (0.6, 0.26, 240),
    ]
    .into_iter()
    .enumerate()
    {
        let base = f.at(0.54, SOIL, 0.17);
        let tip = f.at(x, SOIL + 0.1, z);
        sweep!(
            format!("courgette_stalk{k}"),
            &[
                base,
                P3 {
                    x: (base.x + tip.x) / 2.0,
                    y: (base.y + tip.y) / 2.0,
                    z: tip.z + 0.04
                },
                tip
            ],
            0.006,
            material = M.leaf,
            verts = 4,
            resolution = 2
        );
        leaf!(
            format!("courgette_leaf{k}"),
            tip,
            a as f64,
            10.0,
            0.17,
            0.19,
            M.leaf,
            shape = "heart",
            droop = 0.3,
            fold = 0.15,
            rows = 4,
            ring = 4
        );
    }
}

pub fn veggie_bed() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _bed_box(f);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The crop, origin on the soil: the plants (foliage) and the fruit (plain), each following the crop's growth
    // stage (`grow`): the plants low when just planted, the fruit only when ripe.
    let plants = new_parts(|| {
        for i in 0..3 {
            // lettuces, front row (the fourth is picked)
            let mats = if i % 2 != 0 {
                [M.leaf_pale, M.leaf_light]
            } else {
                [M.leaf_light, M.leaf_pale]
            };
            _lettuce(
                f,
                &format!("lettuce{i}_"),
                -0.7 + i as f64 * 0.22,
                0.22,
                SOIL - 0.005,
                &mats,
                (i * 3) as i128,
            );
        }
        for i in 0..6 {
            // feathery carrot tops, middle row
            _rosette(
                f,
                &format!("carrot_top{i}_"),
                -0.72 + i as f64 * 0.13,
                SOIL + 0.01,
                -0.05,
                4,
                0.17,
                0.05,
                &[M.leaf_light, M.leaf],
                (62.0, 78.0),
                0.25,
                "lance",
                0.1,
                Some((2, 0.3)),
                5,
                3,
                10 + i as i128,
                0.0,
            );
        }
        for x in TOMATOES {
            _tomato_plant(f, x, -0.2);
        }
        _courgette_plant(f);
    });
    let plants = join(&plants, "plants", Some(f.at(0, SOIL, 0)));
    let fruit = new_parts(|| {
        for x in TOMATOES {
            // trusses of tomatoes hanging off the plant
            for j in 0..5 {
                let a = j as f64 * 1.3 + x;
                let (tx, ty, tz) = (
                    x + cos(a) * 0.1,
                    SOIL + 0.24 + j as f64 * 0.11,
                    -0.2 + sin(a) * 0.1,
                );
                fsphere!(
                    f,
                    format!("tomato{}{j}", float_repr(x)),
                    0.032,
                    (tx, ty, tz),
                    M.tomato,
                    scale = (1, 1, 0.85),
                    segments = 10,
                    rings = 6
                );
                star!(
                    format!("calyx{}{j}", float_repr(x)),
                    f.at(tx, ty + 0.026, tz),
                    0.02,
                    5,
                    M.stem,
                    inner = 0.3,
                    depth = 0.003,
                    cup = 0.1
                );
            }
        }
        for i in 0..6 {
            // the carrots' orange shoulders out of the soil
            revolve!(
                format!("carrot{i}"),
                &[
                    (0.0, -0.03),
                    (0.016, -0.01),
                    (0.021, 0.008),
                    (0.014, 0.018),
                    (0.0, 0.02)
                ],
                at = f.at(-0.72 + i as f64 * 0.13, SOIL, -0.05),
                material = M.carrot,
                segments = 8
            );
        }
        for (k, (pts, r)) in [
            (
                [
                    (0.36, SOIL + 0.035, 0.36),
                    (0.46, SOIL + 0.04, 0.34),
                    (0.58, SOIL + 0.035, 0.3),
                ],
                0.034,
            ),
            (
                [
                    (0.66, SOIL + 0.03, 0.36),
                    (0.72, SOIL + 0.032, 0.28),
                    (0.75, SOIL + 0.03, 0.2),
                ],
                0.028,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            fsweep!(
                f,
                format!("courgette{k}"),
                &pts,
                r,
                M.courgette,
                verts = 8,
                resolution = 3,
                scale = [0.75, 1.0, 0.85]
            );
            _flower(
                f,
                &format!("courgette_flower{k}"),
                pts[0].0 - 0.03,
                SOIL + 0.04,
                pts[0].2 + 0.01,
                0.04,
                M.yellow,
                None,
                5,
                60.0,
                200.0,
            );
        }
    });
    let fruit = join(&fruit, "fruit", Some(f.at(0, SOIL, 0)));
    // The lettuce picked for a snack, origin just under the soil: `harvest` presses it away; it follows the
    // plants' growth too.
    let lettuce = new_parts(|| {
        _lettuce(
            f,
            "picked",
            -0.04,
            0.22,
            SOIL - 0.02,
            &[M.leaf_light, M.leaf_pale],
            7,
        );
    });
    // (flattened, it is under the soil)
    let lettuce = join(&lettuce, "lettuce", Some(f.at(-0.04, SOIL - 0.02, 0.22)));
    let objs = settle_all(); // on its Model centre: the back on the wall face
    stages!(
        plants,
        "grow",
        picked = 1,
        growing = (1, 1, 0.6),
        planted = (1, 1, 0.25)
    );
    stages!(
        lettuce,
        "grow",
        picked = 1,
        growing = (1, 1, 0.6),
        planted = (1, 1, 0.25)
    );
    stages!(
        fruit,
        "grow",
        picked = 0.001,
        growing = 0.001,
        planted = 0.001
    );
    squash!(lettuce, "harvest", (1, 1, 0.001), seconds = 0.8); // down into the soil (only Z: see the notes)
    // Tending: the plants and their fruit rustle, swaying 3° back and forth about the soil line through the bed's
    // middle (their shared origin), on top of the growth stage (objectMotion.ts plays a use clip after it).
    for part in [plants, fruit] {
        rock!(part, "rustle", 3.0, seconds = 0.8, axis = Axis::X);
    }
    objs
}
