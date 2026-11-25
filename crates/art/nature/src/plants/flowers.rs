//! G2, the flower beds: tulips, daffodils, daisies, sunflowers, poppies, marigolds, pansies, lilies, lupins,
//! forget-me-nots and cosmos, each on its soil mound.

use artkit::botany::{clumps, fit_to, lupin_spike, palette, polar};
use artkit::far::{is_low, low};
use artkit::math::{V3, v3};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{
    _put, cupped, cut_leaf, florets, lump, mound, outline, pinnate, rays, rosettes, sphere, spiral,
    stalk, tulip_head, twig,
};

/// Tulips: a soil mound of tulips in five colours, closed cups of petals on straight, slightly bowed stems, broad
/// grey-green leaves at their feet.
pub fn tulips() -> Vec<Obj> {
    let p = palette(&[("leaf", "#5E8A5A"), ("leaf_light", "#6E9A66")]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(51);
    let top = mound!(&p);
    let cols = [
        m("bloom"),
        m("bloom2"),
        m("bloom3"),
        m("bloom4"),
        m("bloom"),
        m("bloom5"),
    ];
    for (i, (x, y, a)) in spiral!(17, 0.34, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.42, 0.55);
        let tip = (
            x + rng.uniform(-0.03, 0.03),
            y + rng.uniform(-0.03, 0.03),
            h,
        );
        stalk!(
            format!("stem{i}"),
            (x, y, top - 0.01),
            tip,
            m("stem"),
            r = 0.005
        );
        let (yaw, tilt) = (rng.uniform(0.0, 360.0), rng.uniform(0.0, 10.0));
        tulip_head!(
            format!("tulip{i}"),
            V3::from(tip) - v3(0, 0, 0.01),
            0.032,
            0.075,
            cols[i % cols.len()],
            yaw = yaw,
            tilt = tilt
        );
        for k in 0..2 {
            let (yaw, t, length) = (
                a + 40.0 + k as f64 * 160.0,
                rng.uniform(55.0, 70.0),
                rng.uniform(0.2, 0.26),
            );
            if k != 0 && i % 2 != 0 {
                continue;
            }
            cut_leaf!(
                format!("leaf{i}_{k}"),
                (x, y, top - 0.005),
                yaw,
                t,
                length,
                &outline!("lance", 0.075, n = 4),
                [m("leaf"), m("leaf_light")][k],
                droop = 0.32,
                fold = 0.35,
                ribs = 3,
                thick = 0.005
            );
        }
    }
    fit_to(&meshes(), 0.84, 0.622, 0.843);
    settle(&[join_all("tulips")])
}

/// Daffodils: pale six-petalled flowers with ruffled orange trumpets, nodding outwards on bowed stems, and
/// blue-green strap leaves.
pub fn daffodils() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#FAE68A"),
        ("centre", "#F6AE1C"),
        ("stem", "#5E8C3A"),
        ("leaf", "#5E8A5A"),
        ("leaf_light", "#6E9A66"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(52);
    let top = mound!(&p);
    for (i, (x, y, a)) in spiral!(13, 0.34, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.34, 0.46);
        let tip = (x, y, h);
        let yaw = a + rng.uniform(-40.0, 40.0);
        stalk!(
            format!("stem{i}"),
            (x, y, top - 0.01),
            tip,
            m("stem"),
            r = 0.004
        );
        rays!(
            format!("petals{i}"),
            tip,
            0.05,
            6,
            m("bloom"),
            yaw = yaw,
            tilt = 70,
            inner = 0.3,
            cup = 0.12,
            width = 0.8,
            shoulder = 0.0
        );
        cupped!(
            format!("trumpet{i}"),
            &[
                (0.0, 0.0),
                (0.014, 0.0),
                (0.018, 0.035),
                (0.026, 0.05),
                (0.02, 0.045)
            ],
            tip,
            m("centre"),
            yaw = yaw,
            tilt = 70,
            segments = low(6, 3)
        );
        for k in 0..2 {
            let (yaw_k, t, length) = (
                yaw + 150.0 + k as f64 * 60.0,
                rng.uniform(70.0, 82.0),
                rng.uniform(0.26, 0.32),
            );
            if is_low() && k != 0 {
                continue;
            }
            cut_leaf!(
                format!("leaf{i}_{k}"),
                (x, y, top),
                yaw_k,
                t,
                length,
                &outline!("strap", 0.032, n = 3),
                [m("leaf"), m("leaf_light")][(i + k) % 2],
                droop = 0.3,
                fold = 0.3,
                ribs = 2,
                thick = 0.005
            );
        }
    }
    fit_to(&meshes(), 0.84, 0.491, 0.84);
    settle(&[join_all("daffodils")])
}

/// Daisies: low rosettes of leaves and white daisies of separate ray petals round a raised yellow eye, on thin
/// stems.
pub fn daisies() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#FFFFFF"),
        ("centre", "#F6C21E"),
        ("stem", "#5E8C3A"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(53);
    let top = mound!(&p);
    rosettes!(
        &mut rng,
        5,
        0.3,
        top - 0.01,
        &[m("leaf"), m("leaf_dark"), m("leaf_light")],
        leaves = 4,
        length = 0.17,
        width = 0.065,
        tilt = (15.0, 35.0)
    );
    for (i, (x, y, _)) in spiral!(15, 0.38, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.25, 0.36);
        let (yaw, tilt) = (rng.uniform(0.0, 360.0), rng.uniform(0.0, 30.0));
        stalk!(
            format!("stem{i}"),
            (x, y, top),
            (x, y, h),
            m("stem"),
            r = 0.004
        );
        rays!(
            format!("daisy{i}"),
            (x, y, h),
            0.045,
            10,
            m("bloom"),
            yaw = yaw,
            tilt = tilt,
            inner = 0.32,
            cup = 0.06,
            width = 0.75,
            shoulder = 0.0
        );
        if !is_low() {
            let eye = sphere!(
                format!("eye{i}"),
                1.0,
                material = m("centre"),
                segments = 5,
                rings = 3,
                scale = (0.015, 0.015, 0.009)
            );
            _put!(eye, (x, y, h), yaw = yaw, tilt = tilt, lift = 0.004);
        }
    }
    fit_to(&meshes(), 0.84, 0.376, 0.84);
    settle(&[join_all("daisies")])
}

/// Sunflowers: four tall, hairy stalks with big, drooping heart-shaped leaves, their heavy heads turned to the
/// front: two rows of golden petals round a domed brown disc of seeds, green behind.
pub fn sunflowers() -> Vec<Obj> {
    let p = super::leafy_petals(
        palette(&[
            ("bloom", "#F5C518"),
            ("bloom2", "#E8A812"),
            ("centre_dark", "#5A3A1E"),
            ("stem", "#5E8C3A"),
        ]),
        &["bloom", "bloom2"],
    );
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(54);
    let top = mound!(&p);
    for (i, ((x, y), h)) in [(0.18, 0.12), (-0.17, 0.06), (0.0, -0.18), (0.04, 0.3)]
        .into_iter()
        .zip([1.85, 1.6, 1.4, 1.95])
        .enumerate()
    {
        let tip = v3(x * 1.2, y * 1.2 - 0.04, h);
        twig!(
            format!("stalk{i}"),
            &[
                v3(x, y, top - 0.01),
                v3(x, y, 0).lerp(tip, 0.5) + v3(0.01, 0, 0),
                tip
            ],
            0.02,
            0.013,
            m("stem"),
            verts = 6,
            res = 2
        );
        for k in 0..4 {
            let t = 0.25 + k as f64 * 0.16;
            let base = v3(x, y, 0).lerp(tip, t);
            let yaw = rng.uniform(0.0, 360.0) + (k * 120) as f64;
            let end = base + V3::from(polar(0.05, yaw, 0.02));
            twig!(
                format!("petiole{i}_{k}"),
                &[base, end],
                0.006,
                0.004,
                m("stem"),
                verts = 3,
                res = 1
            );
            cut_leaf!(
                format!("leaf{i}_{k}"),
                end,
                yaw,
                25.0,
                0.22 - k as f64 * 0.015,
                &outline!("heart", 0.18 - k as f64 * 0.012, n = 6, teeth = 0.08),
                [m("leaf"), m("leaf_dark"), m("leaf_light")][k % 3],
                droop = 0.42,
                fold = 0.18,
                ribs = 4
            );
        }
        let yaw = 180.0 + rng.uniform(-35.0, 35.0);
        rays!(
            format!("petals{i}"),
            tip,
            0.17,
            18,
            m("bloom"),
            yaw = yaw,
            tilt = 62,
            inner = 0.5,
            cup = 0.05,
            droop = 0.06,
            width = 0.7,
            thick = 0.012
        );
        if !is_low() {
            rays!(
                format!("petals_b{i}"),
                tip - V3::from(polar(0.006, yaw, 0.0)),
                0.15,
                18,
                m("bloom2"),
                yaw = yaw + 10.0,
                tilt = 62,
                inner = 0.5,
                cup = 0.08,
                width = 0.75,
                thick = 0.01
            );
        }
        let disc = florets!(
            format!("seeds{i}"),
            0.088,
            (0, 0, -0.012),
            m("centre_dark"),
            bumps = 6.0,
            amount = 0.05,
            flat = 0.28,
            segments = low(14, 8),
            rings = low(6, 4),
            seed = i as f64
        );
        _put!(disc, tip, yaw = yaw, tilt = 62);
        cupped!(
            format!("back{i}"),
            &[
                (0.0, -0.03),
                (0.04, -0.028),
                (0.08, -0.012),
                (0.09, 0.0),
                (0.0, 0.0)
            ],
            tip,
            m("stem"),
            yaw = yaw,
            tilt = 62,
            segments = low(10, 5)
        );
    }
    crate::box_uvs(&["stalk", "petiole", "back"]);
    fit_to(&meshes(), 0.84, 2.108, 0.907);
    settle(&[join_all("sunflowers")])
}

/// Poppies: feathery grey-green leaves and red cups of four broad petals with dark middles on thin wavering
/// stems, a few nodding buds.
pub fn poppies() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#EC3A2E"),
        ("stem", "#6E9464"),
        ("leaf", "#5E8456"),
        ("leaf_light", "#6E9464"),
        ("centre_dark", "#1E1A1A"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(55);
    let top = mound!(&p);
    rosettes!(
        &mut rng,
        6,
        0.3,
        top - 0.01,
        &[m("leaf"), m("leaf_light")],
        leaves = 3,
        length = 0.22,
        width = 0.1,
        tilt = (20.0, 40.0),
        pinnae = 3
    );
    for (i, (x, y, _)) in spiral!(16, 0.38, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.4, 0.61);
        let tip = (
            x + rng.uniform(-0.04, 0.04),
            y + rng.uniform(-0.04, 0.04),
            h,
        );
        let (yaw, tilt) = (rng.uniform(0.0, 360.0), rng.uniform(0.0, 30.0));
        stalk!(
            format!("stem{i}"),
            (x, y, top),
            tip,
            m("stem"),
            r = 0.0035,
            bow = 0.04
        );
        if i % 5 == 4 {
            // a bud, nodding
            lump!(
                format!("bud{i}"),
                0.014,
                V3::from(tip) - v3(0, 0, 0.03),
                m("stem"),
                scale = (1, 1, 1.4),
                segments = low(6, 4),
                rings = low(4, 3)
            );
            continue;
        }
        rays!(
            format!("poppy{i}"),
            tip,
            0.045,
            4,
            m("bloom"),
            yaw = yaw,
            tilt = tilt,
            inner = 0.2,
            cup = 0.75,
            width = 1.25,
            thick = 0.008
        );
        if !is_low() {
            let eye = sphere!(
                format!("eye{i}"),
                1.0,
                material = m("centre_dark"),
                segments = 5,
                rings = 3,
                scale = (0.011, 0.011, 0.012)
            );
            _put!(eye, tip, yaw = yaw, tilt = tilt, lift = 0.01);
        }
    }
    fit_to(&meshes(), 0.84, 0.657, 0.84);
    settle(&[join_all("poppies")])
}

/// Marigolds: bushy clumps of dark, finely cut leaves covered in ruffled orange and yellow pompoms.
pub fn marigolds() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", "#2F5A28"),
        ("leaf_dark", "#3A6A2E"),
        ("leaf_light", "#467634"),
        ("blossom", "#F8901C"),
        ("blossom2", "#FCBA1E"),
        ("blossom3", "#F0761A"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(56);
    let top = mound!(&p);
    clumps(
        &mut rng,
        7,
        0.33,
        0.27,
        0.17,
        top + 0.07,
        &[m("leaf"), m("leaf_dark"), m("leaf_light")],
    );
    // cut leaves sticking out round the clumps
    for (i, (x, y, a)) in spiral!(7, 0.38, &mut rng).into_iter().enumerate() {
        let (yaw, t) = (a + rng.uniform(-30.0, 30.0), rng.uniform(15.0, 40.0));
        if is_low() {
            continue;
        }
        cut_leaf!(
            format!("leaf{i}"),
            (x * 0.8, y * 0.8, top + 0.06),
            yaw,
            t,
            0.13,
            &pinnate!(3, 0.06, "lance", lean = 0.6, fill = 0.6),
            m("leaf_light"),
            droop = 0.3,
            ribs = 2
        );
    }
    for (i, (x, y, _)) in spiral!(16, 0.38, &mut rng).into_iter().enumerate() {
        let z = rng.uniform(0.19, 0.27);
        florets!(
            format!("pom{i}"),
            0.05,
            (x, y, z),
            [m("blossom"), m("blossom2"), m("blossom3")][i % 3],
            bumps = 4.5,
            amount = 0.18,
            flat = 0.75,
            segments = 8,
            rings = 5,
            seed = i as f64 * 0.7
        );
    }
    fit_to(&meshes(), 0.885, 0.34, 0.84);
    settle(&[join_all("marigolds")])
}

/// Pansies: a low carpet of small round leaves and flat, overlapping five-petalled faces turned outwards, each
/// with a dark blotch at its heart.
pub fn pansies() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#5B3A8E"),
        ("bloom2", "#F2C94C"),
        ("bloom3", "#3F5FB0"),
        ("bloom4", "#8E2A4A"),
        ("bloom5", "#F4F0E8"),
        ("centre", "#F2C94C"),
        ("centre_dark", "#3A2A4E"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(57);
    let top = mound!(&p);
    rosettes!(
        &mut rng,
        8,
        0.34,
        top - 0.01,
        &[m("leaf"), m("leaf_dark"), m("leaf_light")],
        leaves = 5,
        length = 0.13,
        width = 0.08,
        shape = "oval",
        tilt = (10.0, 30.0)
    );
    let faces = [
        (m("bloom"), m("centre")),
        (m("bloom2"), m("centre_dark")),
        (m("bloom3"), m("centre")),
        (m("bloom4"), m("centre")),
        (m("bloom5"), m("centre_dark")),
    ];
    for (i, (x, y, a)) in spiral!(15, 0.36, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.17, 0.23);
        let (petal, eye) = faces[i % faces.len()];
        let (yaw, tilt) = (a + rng.uniform(-30.0, 30.0), rng.uniform(35.0, 60.0));
        // on stems: without, the faces stood in the air
        stalk!(
            format!("stem{i}"),
            (x, y, top + 0.02),
            (x, y, h),
            m("stem"),
            r = 0.004
        );
        rays!(
            format!("pansy{i}"),
            (x, y, h),
            0.045,
            5,
            petal,
            yaw = yaw,
            tilt = tilt,
            inner = 0.25,
            cup = 0.05,
            width = 1.35,
            thick = 0.006
        );
        if !is_low() {
            rays!(
                format!("eye{i}"),
                (x, y, h),
                0.02,
                3,
                eye,
                yaw = yaw,
                tilt = tilt,
                inner = 0.3,
                cup = 0.05,
                width = 1.2,
                thick = 0.012
            );
        }
    }
    fit_to(&meshes(), 0.84, 0.264, 0.84);
    settle(&[join_all("pansies")])
}

/// Lilies: six tall stems clothed in narrow leaves, each with two open lilies, six petals curling back from a
/// trumpet, and a bud.
pub fn lilies() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#F07A2A"),
        ("bloom2", "#F5EFE4"),
        ("bloom3", "#E98AA8"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(58);
    let top = mound!(&p);
    let cols = [m("bloom"), m("bloom2"), m("bloom3")];
    for (i, (x, y, a)) in spiral!(6, 0.27, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.71, 0.87);
        let tip = v3(x * 1.15, y * 1.15, h);
        stalk!(
            format!("stem{i}"),
            (x, y, top - 0.01),
            tip,
            m("stem"),
            r = 0.007,
            bow = 0.02
        );
        for k in 0..4 {
            let (t, yaw) = (0.15 + k as f64 * 0.17, rng.uniform(0.0, 360.0));
            if is_low() && k % 2 != 0 {
                // a far version: three leaves a stem
                continue;
            }
            cut_leaf!(
                format!("leaf{i}_{k}"),
                v3(x, y, 0).lerp(tip, t),
                yaw,
                35.0,
                0.17,
                &outline!("lance", 0.03, n = 3),
                m("leaf"),
                droop = 0.3,
                ribs = 2,
                thick = 0.004
            );
        }
        for k in 0..2 {
            let yaw = a + (k as f64 * 2.0 - 1.0) * 50.0 + rng.uniform(-15.0, 15.0);
            let at = tip - v3(0, 0, 0.03 * k as f64);
            rays!(
                format!("lily{i}_{k}"),
                at,
                0.075,
                6,
                cols[i % 3],
                yaw = yaw,
                tilt = 55,
                inner = 0.18,
                cup = 1.1,
                droop = 0.55,
                width = 0.7,
                shoulder = 0.4
            );
            if !is_low() {
                cupped!(
                    format!("throat{i}_{k}"),
                    &[
                        (0.0, 0.0),
                        (0.008, 0.0),
                        (0.016, 0.03),
                        (0.02, 0.04),
                        (0.0, 0.02)
                    ],
                    at,
                    cols[i % 3],
                    yaw = yaw,
                    tilt = 55,
                    segments = 4
                );
            }
        }
        lump!(
            format!("bud{i}"),
            0.012,
            tip + v3(0, 0, 0.005),
            m("stem"),
            scale = (1, 1, 2.4),
            segments = low(5, 4),
            rings = 3
        );
    }
    fit_to(&meshes(), 0.84, 0.939, 0.869);
    settle(&[join_all("lilies")])
}

/// Lupins: hand-shaped leaves of narrow leaflets on stalks and tall tapering spikes of pea flowers in purples,
/// pinks, white and blue.
pub fn lupins() -> Vec<Obj> {
    let p = palette(&[
        ("blossom", "#8E6CE0"),
        ("blossom2", "#F08CC4"),
        ("blossom3", "#F4F0F2"),
        ("blossom4", "#6C8CF0"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(59);
    let top = mound!(&p);
    let cols = [m("leaf"), m("leaf_dark"), m("leaf_light")];
    // the leaves: leaflets fanned round a stalk's end
    for (i, (x, y, a)) in spiral!(10, 0.3, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.12, 0.22);
        let end = v3(x * 1.15, y * 1.15, h);
        stalk!(
            format!("petiole{i}"),
            (x * 0.6, y * 0.6, top),
            end,
            m("stem"),
            r = 0.004
        );
        let tilt = rng.uniform(5.0, 25.0);
        rays!(
            format!("leaf{i}"),
            end,
            0.085,
            9,
            cols[i % 3],
            yaw = a,
            tilt = tilt,
            inner = 0.08,
            cup = 0.15,
            droop = 0.2,
            width = 0.42,
            thick = 0.004
        );
    }
    let cols = [m("blossom"), m("blossom2"), m("blossom3"), m("blossom4")];
    for (i, (x, y, _)) in spiral!(9, 0.3, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.66, 0.99);
        let tip = (
            x + rng.uniform(-0.03, 0.03),
            y + rng.uniform(-0.03, 0.03),
            h,
        );
        stalk!(
            format!("stem{i}"),
            (x, y, top + 0.02),
            (tip.0, tip.1, h - 0.34),
            m("stem"),
            r = 0.007
        );
        lupin_spike(
            format!("spike{i}"),
            (tip.0, tip.1, h - 0.36),
            0.36,
            0.045,
            cols[i % 4],
        );
    }
    fit_to(&meshes(), 0.84, 0.987, 0.84);
    settle(&[join_all("lupins")])
}

/// Forget-me-nots: leafy clumps under drifts of tiny sky-blue flowers, a few open ones standing out.
pub fn forget_me_nots() -> Vec<Obj> {
    let p = palette(&[
        ("blossom", "#8CB4F8"),
        ("blossom2", "#A4C4FC"),
        ("blossom3", "#7AA0F0"),
        ("centre", "#F6E27A"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(60);
    let top = mound!(&p);
    clumps(
        &mut rng,
        7,
        0.33,
        0.28,
        0.15,
        top + 0.05,
        &[m("leaf"), m("leaf_dark"), m("leaf_light")],
    );
    let cols = [m("blossom"), m("blossom2"), m("blossom3")];
    for (i, (x, y, _)) in spiral!(16, 0.36, &mut rng).into_iter().enumerate() {
        let z = rng.uniform(0.17, 0.23);
        florets!(
            format!("drift{i}"),
            0.06,
            (x, y, z),
            cols[i % 3],
            bumps = 6.0,
            amount = 0.22,
            flat = 0.4,
            segments = 8,
            rings = 4,
            seed = i as f64 * 0.9
        );
    }
    // single flowers standing out of the drifts
    for (i, (x, y, _)) in spiral!(10, 0.34, &mut rng).into_iter().enumerate() {
        let (h, yaw, tilt) = (
            rng.uniform(0.22, 0.27),
            rng.uniform(0.0, 360.0),
            rng.uniform(0.0, 30.0),
        );
        if is_low() {
            continue;
        }
        rays!(
            format!("flower{i}"),
            (x, y, h),
            0.02,
            5,
            cols[i % 3],
            yaw = yaw,
            tilt = tilt,
            inner = 0.3,
            cup = 0.1,
            width = 1.1,
            centre = 0.003
        );
    }
    fit_to(&meshes(), 0.866, 0.28, 0.84);
    settle(&[join_all("forgetMeNots")])
}

/// Cosmos: feathery leaves cut into threads and tall thin stems of pink, white and magenta flowers, eight broad
/// petals round a yellow eye.
pub fn cosmos() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#E77FB0"),
        ("bloom2", "#F6EEF2"),
        ("bloom3", "#C2367A"),
        ("bloom4", "#F2A6CA"),
        ("centre", "#F2C12E"),
        ("stem", "#6E9E44"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(61);
    let top = mound!(&p);
    for (i, (x, y, _)) in spiral!(9, 0.3, &mut rng).into_iter().enumerate() {
        let (yaw, t, length) = (
            rng.uniform(0.0, 360.0),
            rng.uniform(58.0, 78.0),
            rng.uniform(0.28, 0.38),
        );
        cut_leaf!(
            format!("frond{i}"),
            (x, y, top),
            yaw,
            t,
            length,
            &pinnate!(5, 0.16, "lance", lean = 0.8, fill = 0.25),
            [m("leaf_light"), m("leaf")][i % 2],
            droop = 0.3,
            fold = 0.1,
            ribs = 3
        );
    }
    let cols = [m("bloom"), m("bloom2"), m("bloom3"), m("bloom4")];
    for (i, (x, y, _)) in spiral!(14, 0.36, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.55, 0.87);
        let tip = (x * 1.1, y * 1.1, h);
        let (yaw, tilt) = (rng.uniform(0.0, 360.0), rng.uniform(10.0, 40.0));
        stalk!(
            format!("stem{i}"),
            (x, y, top - 0.01),
            tip,
            m("stem"),
            r = 0.0035,
            bow = 0.04
        );
        rays!(
            format!("cosmos{i}"),
            tip,
            0.05,
            8,
            cols[i % 4],
            yaw = yaw,
            tilt = tilt,
            inner = 0.25,
            cup = 0.1,
            width = 0.9,
            shoulder = 0.0
        );
        if !is_low() {
            let eye = sphere!(
                format!("eye{i}"),
                1.0,
                material = m("centre"),
                segments = 5,
                rings = 3,
                scale = (0.012, 0.012, 0.008)
            );
            _put!(eye, tip, yaw = yaw, tilt = tilt, lift = 0.004);
        }
    }
    fit_to(&meshes(), 0.885, 0.879, 0.84);
    settle(&[join_all("cosmos")])
}
