//! G1, the houseplants: the potted plant, the monstera, the fiddle-leaf fig, the snake plant, the cacti, the
//! succulents, the fern on its stand, the parlour palm, the orchid and the bonsai.

use artkit::botany::{_flower, fiddle, fit_to, palette, polar, sword};
use artkit::far::{is_low, low};
use artkit::forms::by;
use artkit::kit::fillet;
use artkit::math::{M4, P3, TAU, V3, atan2, cos, degrees, radians, sin, v3};
use artkit::parts::{join, join_all, new_parts, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{
    arc, boulder, cone, cube, cut_leaf, cylinder, florets, jitter, leaf, loft, lump, outline,
    pinnate, rays, revolve, ribbed, ring, slits, soil_top, sphere, squash, sweep, tube, twig,
    vessel,
};

/// Potted plant: a terracotta pot with a rolled rim on its saucer, and a lush arrowhead plant, three rings of
/// heart-shaped leaves on arching stalks round a few young ones still rolled up in the middle.
pub fn plant() -> Vec<Obj> {
    let p = super::plain_leaves(palette(&[
        ("body", "#C46A43"),
        // (plain, so as dark as the foliage texture drew them)
        ("leaf", "#37632B"),
        ("leaf_dark", "#2E5626"),
        ("leaf_light", "#4C7A3A"),
        ("stem", "#5E8044"),
    ]));
    let ys = vessel!(
        &p,
        0.19,
        0.32,
        r_bottom = 0.14,
        lip = 0.016,
        foot = 0.0,
        saucer = 0.035,
        segments = 18
    );
    let cols = [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")];
    let mut k = 0;
    // count, heading offset, stalk length, stalk tilt, leaf length, width, tilt, droop
    for (n, off, sl, st, ll, lw, lt, dr) in [
        (8, 0, 0.17, 30.0, 0.22, 0.17, 0.0, 0.32),
        (7, 22, 0.28, 56.0, 0.23, 0.17, 22.0, 0.24),
        (5, 10, 0.37, 74.0, 0.22, 0.16, 44.0, 0.12),
    ] {
        for i in 0..n {
            let yaw = off as f64 + (i * 360) as f64 / n as f64 + ((k * 37) % 17) as f64;
            let a = V3::from(polar(0.03, yaw, ys));
            let tip = a + V3::from(polar(sl * cos(radians(st)), yaw, sl * sin(radians(st))));
            twig!(
                format!("stalk{k}"),
                &arc!(a, tip, 0.03, steps = 2),
                0.006,
                0.004,
                p.get("stem"),
                verts = 3,
                res = 1
            );
            cut_leaf!(
                format!("leaf{k}"),
                tip,
                yaw,
                lt,
                ll,
                &outline!("heart", lw, n = 6),
                cols[k % 3],
                droop = dr,
                fold = 0.22,
                ribs = 4
            );
            k += 1;
        }
    }
    // young leaves, still rolled, in the middle
    for i in 0..low(3, 0) {
        let yaw = (i * 120 + 40) as f64;
        let young = cone!(
            format!("young{i}"),
            0.012,
            0.002,
            0.14,
            at = (0, 0, 0),
            material = p.get("leaf_light"),
            verts = 5
        );
        let d = V3::from(polar(sin(0.35), yaw, cos(0.35)));
        young.edit(|o| {
            o.mesh.transform(
                &(M4::translation(v3(0, 0, ys + 0.01) + d * 0.08)
                    * d.to_track_quat_zy().to_matrix().to_4x4()),
            )
        });
    }
    fit_to(&meshes(), 0.772, 0.825, 0.791);
    settle(&[join_all("plant")])
}

/// Monstera: a white pot, a moss pole, and big glossy leaves split from their edges almost to the midrib, on long
/// arching stems, with a couple of aerial roots reaching down.
pub fn monstera() -> Vec<Obj> {
    let p = super::plain_leaves(palette(&[
        ("body", "#EDEBE6"),
        ("leaf", "#2F6B35"),
        ("leaf_dark", "#28602E"),
        ("leaf_light", "#3B7A3C"),
        ("stem", "#4E7A3A"),
        ("moss", "#6E6A3E"),
    ]));
    let ys = vessel!(
        &p,
        0.18,
        0.33,
        r_bottom = 0.14,
        lip = 0.0,
        foot = 0.02,
        segments = 18
    );
    let pole = cylinder!(
        "pole",
        0.024,
        0.62,
        at = (0.02, 0.04, ys + 0.28),
        material = p.get("moss"),
        verts = low(8, 5)
    );
    if !is_low() {
        jitter!(pole, 0.006, size = 0.05, seed = 3.0);
    }
    let mut rng = Random::new(91);
    let cols = [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")];
    for i in 0..8 {
        let yaw = i as f64 / 8.0 * 720.0 + rng.uniform(-15.0, 15.0);
        let out = rng.uniform(0.07, 0.13);
        let h = rng.uniform(0.3, 0.6);
        let (tilt, length, width) = (
            rng.uniform(5.0, 25.0),
            rng.uniform(0.29, 0.33),
            rng.uniform(0.27, 0.31),
        );
        let tip = V3::from(polar(out, yaw, ys + h));
        twig!(
            format!("stem{i}"),
            &arc!((0, 0, ys), tip, 0.06, steps = 3),
            0.009,
            0.006,
            p.get("stem"),
            verts = 4,
            res = 1
        );
        cut_leaf!(
            format!("leaf{i}"),
            tip,
            yaw,
            tilt,
            length,
            &slits!("heart", width, 5, depth = 0.62, gap = 0.22),
            cols[i % 3],
            droop = 0.5,
            fold = 0.1,
            ribs = 5,
            thick = 0.006
        );
    }
    // aerial roots
    for i in 0..low(2, 0) {
        let a = (40 + i * 150) as f64;
        let top = V3::from(polar(0.05, a, ys + 0.22));
        twig!(
            format!("root{i}"),
            &[
                top,
                top + V3::from(polar(0.08, a, -0.06)),
                V3::from(polar(0.15, a, ys - 0.005))
            ],
            0.005,
            0.004,
            p.get("stem"),
            verts = 3
        );
    }
    fit_to(&meshes(), 0.784, 0.911, 0.791);
    settle(&[join_all("monstera")])
}

/// Fiddle-leaf fig: a cream pot with a rolled rim, a slim, slightly crooked trunk in bark, and big leathery,
/// wavy-edged fiddle-shaped leaves spiralling up it on short stalks.
pub fn fiddle_fig() -> Vec<Obj> {
    let p = super::plain_leaves(palette(&[
        ("body", "#D9CDB4"),
        ("leaf", "#2E5A2A"),
        ("leaf_dark", "#2A5226"),
        ("leaf_light", "#386832"),
        ("bark", "#8A7058"),
    ]));
    let ys = vessel!(
        &p,
        0.2,
        0.38,
        r_bottom = 0.15,
        lip = 0.016,
        foot = 0.025,
        segments = 20
    );
    let mut rng = Random::new(92);
    let trunk_pts = [
        (0.0, 0.0, ys - 0.02),
        (0.01, 0.0, 0.8),
        (-0.012, 0.01, 1.2),
        (0.0, 0.0, 1.5),
    ];
    let stem = sweep!(
        "trunk",
        &trunk_pts,
        0.022,
        material = p.get("bark"),
        verts = low(7, 4),
        resolution = low(3, 1),
        scale = by(|w| 1.0 - 0.55 * w)
    );
    if !is_low() {
        jitter!(stem, 0.004, size = 0.08, seed = 2.0, up = 0.0);
    }
    let n = 22;
    for i in 0..n {
        let t = i as f64 / n as f64;
        let z = ys + 0.28 + t * (1.46 - ys - 0.28);
        let yaw = i as f64 * 137.5 + rng.uniform(-10.0, 10.0);
        let s = 1.05 - t * 0.3;
        let (tilt, length, width) = (
            rng.uniform(15.0, 40.0),
            rng.uniform(0.26, 0.3) * s,
            rng.uniform(0.2, 0.23) * s,
        );
        let r_at = 0.02 * (1.0 - 0.5 * t);
        let base = V3::from(polar(r_at, yaw, z));
        let end = base + V3::from(polar(0.035, yaw, 0.02));
        twig!(
            format!("stalk{i}"),
            &[base, end],
            0.005,
            0.004,
            p.get("leaf_light"),
            verts = 3,
            res = 1
        );
        cut_leaf!(
            format!("leaf{i}"),
            end,
            yaw,
            tilt,
            length,
            &outline!(fiddle, width, n = 7, wave = 0.05, waves = 5),
            [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")][i % 3],
            droop = 0.22,
            fold = 0.16,
            ribs = 5,
            thick = 0.008
        );
    }
    cut_leaf!(
        "top",
        (0, 0, 1.49),
        20.0,
        70.0,
        0.14,
        &outline!(fiddle, 0.09, n = 5),
        p.get("leaf_light"),
        droop = 0.1,
        ribs = 3
    );
    fit_to(&meshes(), 0.528, 1.626, 0.527);
    settle(&[join_all("fiddleFig")])
}

/// Snake plant: a charcoal pot and stiff, upright, slightly twisting sword-shaped blades, banded dark green with
/// pale yellow-green edges, and a couple of young ones coming up.
pub fn snake_plant() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#2E2E30"),
        ("leaf", "#2F5A34"),
        ("leaf_dark", "#2A4E30"),
        ("leaf_light", "#B8C060"),
    ]);
    let ys = vessel!(
        &p,
        0.15,
        0.29,
        r_bottom = 0.13,
        lip = 0.012,
        foot = 0.0,
        segments = 18
    );
    let mut rng = Random::new(93);
    let cols = [p.get("leaf"), p.get("leaf_dark")];
    for i in 0..12 {
        let (x, y) = (rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08));
        let yaw = degrees(atan2(x, y)) + rng.uniform(-20.0, 20.0);
        let (tilt, mut length, width, twist) = (
            rng.uniform(78.0, 88.0),
            rng.uniform(0.46, 0.64),
            rng.uniform(0.07, 0.09),
            rng.uniform(-50.0, 50.0),
        );
        if i >= 10 {
            // young ones
            length *= 0.55;
        }
        let base = (x, y, ys - 0.02);
        cut_leaf!(
            format!("blade{i}"),
            base,
            yaw,
            tilt,
            length,
            &outline!(sword, width, n = 6),
            cols[i % 2],
            fold = 0.3,
            ribs = 5,
            thick = 0.014,
            twist = twist
        );
        if !is_low() {
            // the pale edge, just wider, just behind
            cut_leaf!(
                format!("edge{i}"),
                (
                    x - 0.0015 * sin(radians(yaw)),
                    y - 0.0015 * cos(radians(yaw)),
                    ys - 0.02
                ),
                yaw,
                tilt,
                length * 1.01,
                &outline!(sword, width * 1.14, n = 6),
                p.get("leaf_light"),
                fold = 0.3,
                ribs = 2,
                thick = 0.004,
                twist = twist
            );
        }
    }
    // the pot's square-cut rims eased (KIT: botany's `vessel` cuts them square)
    crate::chamfer(&["pot", "saucer"]);
    fit_to(&meshes(), 0.341, 0.905, 0.32);
    settle(&[join_all("snakePlant")])
}

/// Cactus trio: three little terracotta pots with rolled rims, topped with grit: a ribbed column cactus with two
/// arms and a pink flower, a ribbed barrel with a crown of yellow flowers, and a prickly pear's pads.
pub fn cacti() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#C0683F"),
        ("flesh", "#4E8A4A"),
        ("flesh_light", "#6A9E5A"),
        ("bloom", "#E8508A"),
        ("bloom_accent", "#F2C14E"),
        ("sand", "#B8A88A"),
    ]);
    // game (x, depth) -> Blender (-x, -depth)
    let pots = [
        ((0.17, -0.05), 0.105, 0.18),
        ((-0.14, 0.08), 0.115, 0.2),
        ((-0.12, -0.17), 0.08, 0.13),
    ];
    let mut tops = Vec::new();
    for (i, ((x, y), r, h)) in pots.into_iter().enumerate() {
        let mut top = 0.0;
        let parts = new_parts(|| {
            top = vessel!(
                &p,
                r,
                h,
                r_bottom = r * 0.75,
                lip = 0.01,
                segments = 10,
                soil = false
            );
            soil_top!(
                &p,
                r - 0.01,
                top,
                segments = low(8, 6),
                mound = 0.006,
                seed = i as f64,
                material = p.get("sand")
            );
        });
        for o in parts {
            o.edit(|part| {
                part.loc.x = (part.loc.x as f64 + x) as f32;
                part.loc.y = (part.loc.y as f64 + y) as f32;
            });
        }
        tops.push(((x, y), top));
    }
    // the column, two arms and a pink flower
    let ((x, y), z) = tops[0];
    let n = low(6, 4);
    let secs: Vec<Vec<P3>> = [
        (0.0, 0.042),
        (0.12, 0.045),
        (0.26, 0.044),
        (0.32, 0.038),
        (0.35, 0.024),
        (0.36, 0.008),
    ]
    .iter()
    .map(|&(hz, rr)| {
        ribbed!(rr, n)
            .iter()
            .map(|&(u, v)| P3::from((x + u, y + v, z - 0.01 + hz)))
            .collect()
    })
    .collect();
    loft!("column", &secs, material = p.get("flesh"));
    for (name, pts, r) in [
        (
            "armL",
            [
                (x + 0.03, y, z + 0.12),
                (x + 0.1, y, z + 0.13),
                (x + 0.11, y, z + 0.2),
                (x + 0.11, y, z + 0.25),
            ],
            0.024,
        ),
        (
            "armR",
            [
                (x - 0.03, y, z + 0.18),
                (x - 0.09, y, z + 0.19),
                (x - 0.095, y, z + 0.25),
                (x - 0.095, y, z + 0.28),
            ],
            0.02,
        ),
    ] {
        let path = if !is_low() {
            fillet(&pts, 0.03, 2)
        } else {
            pts.to_vec()
        };
        sweep!(
            name,
            &path,
            0.02,
            profile = ribbed!(r, n),
            material = p.get("flesh"),
            scale = by(|w| 1.0 - 0.35 * 0.0_f64.max((w - 0.75) / 0.25))
        );
    }
    rays!(
        "flower",
        (x, y, z + 0.345),
        0.026,
        8,
        p.get("bloom"),
        inner = 0.3,
        cup = 0.6,
        width = 0.9,
        centre = 0.004
    );
    // the barrel, ribbed, with a crown of yellow flowers
    let ((x, y), z) = tops[1];
    let m = low(10, 6);
    let secs: Vec<Vec<P3>> = [
        (0.0, 0.06),
        (0.03, 0.09),
        (0.08, 0.1),
        (0.13, 0.085),
        (0.16, 0.05),
        (0.17, 0.02),
    ]
    .iter()
    .map(|&(hz, rr)| {
        // (six points a rib: rounded ribs, no turn of 60° at their ridges)
        ribbed!(rr, m, depth = 0.14, verts = m * 6)
            .iter()
            .map(|&(u, v)| P3::from((x + u, y + v, z + hz)))
            .collect()
    })
    .collect();
    loft!("barrel", &secs, material = p.get("flesh_light"));
    for i in 0..low(3, 1) {
        let a = (i * 120) as f64;
        rays!(
            format!("crown{i}"),
            v3(x, y, z + 0.165) + V3::from(polar(0.025, a, 0.0)),
            0.018,
            6,
            p.get("bloom_accent"),
            yaw = a,
            tilt = 25,
            inner = 0.3,
            cup = 0.5
        );
    }
    // the prickly pear's pads
    let ((x, y), z) = tops[2];
    for (i, (dx, dz, w, h, r)) in [
        (0.0, 0.06, 0.11, 0.14, 8),
        (0.05, 0.16, 0.08, 0.1, -28),
        (-0.045, 0.14, 0.07, 0.09, 30),
    ]
    .into_iter()
    .enumerate()
    {
        sphere!(
            format!("pad{i}"),
            0.5,
            at = (x + dx, y, z + dz),
            material = p.get("flesh_light"),
            segments = low(8, 6),
            rings = low(5, 4),
            scale = (w, 0.032, h),
            rot = (0, r, 0)
        );
    }
    // the pot's square-cut rims eased (KIT: botany's `vessel` cuts them square)
    crate::chamfer(&["pot", "saucer"]);
    fit_to(&meshes(), 0.568, 0.521, 0.465);
    settle(&[join_all("cacti")])
}

/// Succulent bowl: a shallow glazed bowl with a rolled rim, grit and pebbles, five fat-leaved rosettes in
/// blue-green and pink, and a spiky upright one.
pub fn succulents() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#7FA6B8"),
        ("flesh", "#5E8F55"),
        ("flesh_light", "#8FB5A0"),
        ("flesh_pink", "#C99AA8"),
        ("sand", "#C8B894"),
    ]);
    revolve!(
        "bowl",
        &[
            (0.0, 0.0),
            (0.17, 0.0),
            (0.19, 0.008),
            (0.27, 0.085),
            (0.3, 0.11),
            (0.312, 0.122),
            // (its rim rolled round: no turn of 60°)
            (0.311, 0.127),
            (0.306, 0.13),
            (0.292, 0.128),
            (0.276, 0.104),
            (0.0, 0.094)
        ],
        material = p.get("body"),
        segments = low(20, 12)
    );
    soil_top!(
        &p,
        0.278,
        0.104,
        segments = low(20, 10),
        mound = 0.01,
        seed = 2.0,
        material = p.get("sand")
    );
    let mut rng = Random::new(95);
    // pebbles
    for i in 0..low(7, 0) {
        let (a, rr) = (rng.uniform(0.0, TAU), rng.uniform(0.08, 0.24));
        lump!(
            format!("pebble{i}"),
            0.012,
            (rr * cos(a), rr * sin(a), 0.104),
            p.get("body"),
            scale = (1.4, 1, 0.6),
            seed = i as f64,
            noise = 0.2,
            segments = 5,
            rings = 3
        );
    }
    let cols = [
        p.get("flesh_light"),
        p.get("flesh_pink"),
        p.get("flesh"),
        p.get("flesh_light"),
        p.get("flesh_pink"),
    ];
    let mut spots = vec![(0.0, 0.0)];
    spots.extend((0..6).map(|i| {
        let a = i as f64 * TAU / 6.0 + 0.3;
        (0.17 * sin(a), 0.17 * cos(a))
    }));
    for (n, (x, y)) in spots.into_iter().enumerate() {
        let s = rng.uniform(0.85, 1.15) * if n == 0 { 1.15 } else { 1.0 };
        if n == 3 || n == 6 {
            // spiky ones: narrow upright leaves
            for i in 0..6 {
                let yaw = (i * 60 + n * 13) as f64;
                cut_leaf!(
                    format!("spike{n}_{i}"),
                    (x, y, 0.1),
                    yaw,
                    (62 + (i % 2) * 14) as f64,
                    0.1 * s,
                    &outline!(sword, 0.03 * s, n = 3),
                    p.get("flesh"),
                    fold = 0.3,
                    ribs = 2,
                    thick = 0.012
                );
            }
            continue;
        }
        for (tier, (k, length, tilt)) in [(6, 0.085, 24.0), (4, 0.064, 46.0), (3, 0.04, 70.0)]
            .into_iter()
            .enumerate()
        {
            for i in 0..k {
                let yaw = (i * 360) as f64 / k as f64 + (tier * 30 + n * 13) as f64;
                leaf!(
                    format!("r{n}_{tier}_{i}"),
                    (x, y, 0.1 + tier as f64 * 0.01),
                    yaw,
                    tilt,
                    length * s,
                    length * 0.62 * s,
                    cols[n % cols.len()],
                    shape = "oval",
                    thick = 0.024 * s,
                    fold = 0.2,
                    rows = 2,
                    ring = 4,
                    droop = -0.1
                );
            }
        }
    }
    fit_to(&meshes(), 0.62, 0.17, 0.62);
    settle(&[join_all("succulents")])
}

/// Fern on a stand: a white pot on a round wooden plant stand with three splayed, tapering legs, long fronds of
/// leaflets arching over the rim and hanging down.
pub fn boston_fern() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#EDEBE6"),
        ("leaf", "#5E9A3E"),
        ("leaf_dark", "#558E38"),
        ("leaf_light", "#78B24E"),
        ("wood", "#7A5A40"),
    ]);
    revolve!(
        "stand_top",
        &[
            (0.0, 0.538),
            (0.19, 0.538),
            (0.2, 0.548),
            (0.2, 0.562),
            (0.192, 0.572),
            (0.0, 0.572)
        ],
        material = p.get("wood"),
        segments = low(20, 10)
    );
    for i in 0..3 {
        let a = i as f64 * TAU / 3.0 + 0.5;
        twig!(
            format!("stand_leg{i}"),
            &[
                (0.12 * cos(a), 0.12 * sin(a), 0.545),
                (0.185 * cos(a), 0.185 * sin(a), 0.0)
            ],
            0.02,
            0.013,
            p.get("wood"),
            verts = 6,
            res = 1
        );
    }
    let ys = vessel!(
        &p,
        0.145,
        0.24,
        z0 = 0.572,
        r_bottom = 0.11,
        lip = 0.0,
        foot = 0.015,
        segments = 16
    );
    let mut rng = Random::new(96);
    let cols = [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")];
    for i in 0..9 {
        let yaw = i as f64 * 40.0 + rng.uniform(-10.0, 10.0);
        let (tilt, length, droop) = (
            rng.uniform(40.0, 62.0),
            rng.uniform(0.43, 0.49),
            rng.uniform(0.92, 1.04),
        );
        cut_leaf!(
            format!("frond{i}"),
            (0, 0, ys + 0.02),
            yaw,
            tilt,
            length,
            &pinnate!(8, 0.14, "lance", lean = 0.5, fill = 0.75),
            cols[i % 3],
            droop = droop,
            fold = 0.12,
            ribs = 4
        );
    }
    for i in 0..3 {
        let yaw = (i * 120 + 30) as f64;
        cut_leaf!(
            format!("crown{i}"),
            (0, 0, ys + 0.02),
            yaw,
            68.0,
            0.28,
            &pinnate!(6, 0.1, "lance", lean = 0.5, fill = 0.75),
            p.get("leaf_light"),
            droop = 0.5,
            fold = 0.12,
            ribs = 3
        );
    }
    // the pot's square-cut rims eased (KIT: botany's `vessel` cuts them square)
    crate::chamfer(&["pot", "saucer"]);
    fit_to(&meshes(), 0.734, 0.979, 0.792);
    settle(&[join_all("bostonFern")])
}

/// Parlour palm: a cream pot with a rolled rim, slender arching stalks, each a feathery frond of narrow leaflets,
/// and a young frond still upright in the middle.
pub fn palm() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#D9CDB4"),
        ("leaf", "#4A8A3A"),
        ("leaf_dark", "#3E7A34"),
        ("leaf_light", "#5A9A42"),
        ("stem", "#5E7E3A"),
    ]);
    let ys = vessel!(
        &p,
        0.19,
        0.33,
        r_bottom = 0.14,
        lip = 0.014,
        foot = 0.0,
        segments = 18
    );
    let mut rng = Random::new(97);
    let cols = [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")];
    for i in 0..9 {
        let young = i == 8;
        let yaw = i as f64 * 45.0 + rng.uniform(-15.0, 15.0);
        let mut up = rng.uniform(72.0, 86.0);
        let mut length = rng.uniform(0.3, 0.48);
        let (mut flen, mut droop) = (rng.uniform(0.3, 0.36), rng.uniform(0.75, 0.95));
        if young {
            (up, length, flen, droop) = (88.0, 0.32, 0.22, 0.2);
        }
        let tip = V3::from(polar(
            length * cos(radians(up)),
            yaw,
            ys + length * sin(radians(up)),
        ));
        twig!(
            format!("stalk{i}"),
            &[
                v3(0, 0, ys - 0.01),
                v3(0, 0, ys).lerp(tip, 0.5) + V3::from(polar(0.01, yaw, 0.0)),
                tip
            ],
            0.006,
            0.004,
            p.get("stem"),
            verts = 3,
            res = 1
        );
        cut_leaf!(
            format!("frond{i}"),
            tip,
            yaw,
            up - 18.0,
            flen,
            &pinnate!(10, 0.26, "lance", lean = 0.85, fill = 0.3),
            cols[i % 3],
            droop = droop,
            fold = 0.1,
            ribs = 5
        );
    }
    // the pot's square-cut rims eased (KIT: botany's `vessel` cuts them square)
    crate::chamfer(&["pot", "saucer"]);
    fit_to(&meshes(), 0.770, 0.893, 0.693);
    settle(&[join_all("palm")])
}

/// Orchid: a small white pot of bark on a turned wooden pedestal, three broad, fleshy leaves, silvery aerial
/// roots, and a staked, arching spike of moth-orchid flowers and buds.
pub fn orchid() -> Vec<Obj> {
    let p = super::plain_leaves(palette(&[
        ("body", "#F2F0EC"),
        ("leaf", "#2E5A30"),
        ("stem", "#4E6A3A"),
        ("wood", "#5A4030"),
        ("bloom", "#F6F0F4"),
        ("bloom_accent", "#C2367A"),
        ("soil", "#6A4A30"),
        ("flesh", "#AAB8A0"),
    ]));
    revolve!(
        "pedestal",
        &[
            (0.0, 0.0),
            (0.15, 0.0),
            (0.15, 0.03),
            (0.13, 0.05),
            (0.105, 0.08),
            (0.095, 0.32),
            (0.1, 0.52),
            (0.12, 0.56),
            (0.13, 0.585),
            (0.16, 0.6),
            // (its top's arris eased: two turns of 45°)
            (0.16, 0.626),
            (0.156, 0.63),
            (0.0, 0.63)
        ],
        material = p.get("wood"),
        segments = low(16, 8)
    );
    let ys = vessel!(
        &p,
        0.085,
        0.16,
        z0 = 0.63,
        r_bottom = 0.07,
        lip = 0.008,
        segments = 16,
        soil = false
    );
    soil_top!(
        &p,
        0.077,
        ys,
        segments = low(12, 6),
        mound = 0.012,
        seed = 4.0
    );
    for (i, yaw) in [10.0, 150.0, 260.0].into_iter().enumerate() {
        cut_leaf!(
            format!("leaf{i}"),
            (0, 0, ys + 0.01),
            yaw,
            18.0,
            0.22,
            &outline!("oval", 0.095, n = 6),
            p.get("leaf"),
            droop = 0.35,
            fold = 0.22,
            ribs = 5,
            thick = 0.012
        );
    }
    // aerial roots, over the rim
    for i in 0..low(3, 0) {
        let a = (60 + i * 110) as f64;
        let top = V3::from(polar(0.03, a, ys + 0.015));
        twig!(
            format!("root{i}"),
            &[
                top,
                top + V3::from(polar(0.06, a, 0.015)),
                top + V3::from(polar(0.1, a, -0.03))
            ],
            0.005,
            0.004,
            p.get("flesh"),
            verts = 3
        );
    }
    cylinder!(
        "stake",
        0.003,
        0.36,
        at = (-0.03, 0.01, ys + 0.18),
        material = p.get("wood"),
        verts = low(5, 3)
    );
    let spike: Vec<V3> = [
        (0.0, 0.0, ys),
        (-0.03, 0.01, ys + 0.18),
        (-0.06, 0.0, ys + 0.34),
        (-0.16, -0.01, ys + 0.43),
        (-0.27, -0.02, ys + 0.4),
    ]
    .into_iter()
    .map(V3::from)
    .collect();
    twig!(
        "spike",
        &spike,
        0.005,
        0.003,
        p.get("stem"),
        verts = 4,
        res = 2
    );
    for i in 0..8 {
        let t = 0.42 + i as f64 * 0.075;
        let k = (spike.len() - 2).min((t * (spike.len() - 1) as f64) as usize);
        let f = t * (spike.len() - 1) as f64 - k as f64;
        let at = spike[k].lerp(spike[k + 1], f) + v3(0, -0.03, -0.03);
        if i < 6 {
            _flower(
                &format!("flower{i}"),
                at,
                (180 + 12 * (i % 3 - 1)) as f64,
                if i < 5 { 0.05 } else { 0.04 },
                &p,
            );
        } else {
            // buds at the end
            sphere!(
                format!("bud{i}"),
                0.012,
                at = at + v3(0, 0.02, 0.025),
                material = p.get("bloom"),
                segments = low(6, 4),
                rings = low(4, 3),
                scale = (1, 1, 1.3)
            );
        }
    }
    // the pot's square-cut rims eased (KIT: botany's `vessel` cuts them square)
    crate::chamfer(&["pot", "saucer"]);
    fit_to(&meshes(), 0.431, 1.198, 0.381);
    settle(&[join_all("orchid")])
}

/// Bonsai: a blue glazed tray with a rim and little feet, moss and two stones, on a wooden bonsai stand at bench
/// height (its top 0.76 m, a shelf below, rails under the top); a gnarled trunk on spreading surface roots, its
/// branches carrying cloud-pruned pads of foliage, and one stray sprig, which shrinks away while a resident prunes
/// it.
pub fn bonsai() -> Vec<Obj> {
    use artkit::json::float_repr as fr;
    // (plain leaves, as the other houseplants': with the stand's wood, three draw calls, not four)
    let p = super::plain_leaves(palette(&[
        ("body", "#3E5A78"),
        ("wood", "#6A4A32"),
        ("bark", "#6E5A4C"),
        ("leaf", "#3A6A30"),
        ("leaf_dark", "#346028"),
        ("leaf_light", "#467A38"),
        ("stem", "#5E5040"),
        ("moss", "#5E7A34"),
    ]));
    let (wood, body) = (p.get("wood"), p.get("body"));
    let up = 0.38; // the stand's top over the old knee-high table's
    cube!(
        "table_top",
        (0.62, 0.4, 0.04),
        at = (0, 0, 0.36 + up),
        material = wood,
        bevel = low(0.008, 0.0)
    );
    for x in [-0.27, 0.27] {
        for y in [-0.16, 0.16] {
            cube!(
                format!("table_leg{}{}", fr(x), fr(y)),
                (0.045, 0.045, 0.34 + up),
                at = (x, y, (0.34 + up) / 2.0),
                material = wood,
                bevel = low(0.005, 0.0)
            );
        }
    }
    for y in [-0.16, 0.16] {
        cube!(
            format!("rail{}", fr(y)),
            (0.5, 0.022, 0.06),
            at = (0, y, 0.31 + up),
            material = wood
        );
    }
    for x in [-0.27, 0.27] {
        cube!(
            format!("rail_end{}", fr(x)),
            (0.022, 0.28, 0.06),
            at = (x, 0, 0.31 + up),
            material = wood
        );
    }
    cube!(
        "table_shelf",
        (0.56, 0.34, 0.025),
        at = (0, 0, 0.16),
        material = wood,
        bevel = low(0.005, 0.0)
    );
    let z0 = 0.38 + up;
    let secs: Vec<Vec<P3>> = [
        (0.012, 0.2, 0.125),
        (0.02, 0.215, 0.138),
        (0.062, 0.232, 0.152),
        // (its rim's arrises eased: no turn of 60°)
        (0.067, 0.232, 0.152),
        (0.07, 0.228, 0.148),
        (0.07, 0.222, 0.142),
        (0.067, 0.218, 0.138),
        (0.055, 0.215, 0.135),
    ]
    .iter()
    .map(|&(z, rx, ry)| ring!(rx, ry, z = z0 + z, verts = low(24, 12), square = 6))
    .collect();
    loft!("tray", &secs, material = body);
    for x in [-0.16, 0.16] {
        for y in [-0.09, 0.09] {
            cube!(
                format!("tray_foot{}{}", fr(x), fr(y)),
                (0.03, 0.03, 0.014),
                at = (x, y, z0 + 0.007),
                material = body
            );
        }
    }
    cube!(
        "moss",
        (0.42, 0.26, 0.02),
        at = (0, 0, z0 + 0.06),
        material = p.get("moss"),
        bevel = low(0.008, 0.0)
    );
    for (i, (x, y, r)) in [(0.13, -0.06, 0.03), (-0.15, 0.05, 0.022)]
        .into_iter()
        .enumerate()
    {
        boulder!(
            format!("stone{i}"),
            r,
            at = (x, y, z0 + 0.062),
            material = body,
            scale = (1.3, 1, 0.7),
            seed = i as f64 + 1.0,
            subdivisions = low(2, 1)
        );
    }
    // Bonsai trunks are wood here, not bark: one draw call fewer, and a trunk this short hardly bends in the wind.
    let trunk = sweep!(
        "trunk",
        &[
            (0.02, 0.0, 0.44 + up),
            (-0.05, 0.0, 0.58 + up),
            (0.04, 0.01, 0.7 + up),
            (-0.06, 0.0, 0.8 + up)
        ],
        0.034,
        material = wood,
        verts = low(9, 5),
        resolution = low(3, 1),
        scale = by(|w| 1.0 - 0.55 * w)
    );
    if !is_low() {
        jitter!(trunk, 0.006, size = 0.05, seed = 1.0);
    }
    // surface roots, spreading over the moss
    for i in 0..low(4, 0) {
        let a = i as f64 * 1.6 + 0.3;
        twig!(
            format!("root{i}"),
            &[
                (0.02, 0.0, 0.47 + up),
                (0.02 + 0.06 * cos(a), 0.05 * sin(a), 0.45 + up),
                (0.02 + 0.12 * cos(a), 0.08 * sin(a), 0.442 + up)
            ],
            0.014,
            0.004,
            wood,
            verts = 4
        );
    }
    let branches = [
        [
            (-0.05, 0.0, 0.58 + up),
            (0.08, 0.02, 0.64 + up),
            (0.18, 0.03, 0.68 + up),
        ],
        [
            (0.03, 0.01, 0.69 + up),
            (-0.12, -0.02, 0.73 + up),
            (-0.2, -0.03, 0.72 + up),
        ],
        [
            (0.0, 0.01, 0.66 + up),
            (0.03, 0.05, 0.71 + up),
            (0.03, 0.08, 0.73 + up),
        ],
        [
            (-0.05, 0.0, 0.79 + up),
            (-0.08, -0.02, 0.82 + up),
            (-0.1, 0.0, 0.84 + up),
        ],
    ];
    for (i, pts) in branches.iter().enumerate() {
        twig!(
            format!("branch{i}"),
            pts,
            if i < 2 { 0.015 } else { 0.01 },
            0.006,
            wood,
            verts = 5
        );
    }
    // cloud-pruned pads: each a few flattened clumps, lighter on top
    for (i, (x, y, z, r, sx, _sy)) in [
        (-0.07, 0.0, 0.83, 0.12, 1.4, 1.0),
        (0.17, 0.03, 0.7, 0.09, 1.3, 1.0),
        (-0.19, -0.03, 0.75, 0.08, 1.3, 0.9),
        (0.03, 0.07, 0.73, 0.07, 1.2, 1.0),
    ]
    .into_iter()
    .enumerate()
    {
        for j in 0..3 {
            let (dx, dy) = (
                (j as f64 - 1.0) * r * 0.55 * sx,
                (if j == 1 { 0.3 } else { -0.2 }) * r * 0.5,
            );
            florets!(
                format!("pad{i}_{j}"),
                r * if j != 1 { 0.75 } else { 0.85 },
                (
                    x + dx,
                    y + dy,
                    z + up - 0.035 + if j == 1 { 0.012 } else { 0.0 }
                ),
                [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")][(i + j) % 3],
                bumps = 3.5,
                amount = 0.12,
                flat = 0.45,
                segments = 10,
                rings = 6,
                seed = i as f64 * 1.9 + j as f64
            );
        }
    }
    let still = join_all("bonsai");
    // The stray sprig, sticking up out of the top pad: a twig and a tuft, all plain (one draw call).
    let parts = new_parts(|| {
        tube!(
            "sprig_twig",
            &[
                (-0.06, -0.03, 0.83 + up),
                (-0.11, -0.09, 0.88 + up),
                (-0.17, -0.15, 0.92 + up)
            ],
            0.006,
            0.004,
            p.get("stem"),
            verts = 4
        );
        for (i, (yaw, tilt)) in [
            (200.0, 20.0),
            (260.0, 35.0),
            (320.0, 15.0),
            (230.0, 65.0),
            (150.0, 30.0),
        ]
        .into_iter()
        .enumerate()
        {
            leaf!(
                format!("sprig_leaf{i}"),
                (-0.17, -0.15, 0.915 + up),
                yaw,
                tilt,
                0.065,
                0.04,
                p.get("leaf_light"),
                shape = "oval",
                rows = 3,
                ring = 4
            );
        }
    });
    if is_low() {
        // a far version: all one still part
        return settle(&[join_all("bonsai")]);
    }
    let sprig = join(&parts, "sprig", Some(P3::from((-0.06, -0.03, 0.83 + up))));
    settle(&[still, sprig]);
    squash!(sprig, "trim", 0.001, seconds = 0.8);
    vec![still, sprig]
}
