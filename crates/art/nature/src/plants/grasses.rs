//! G3, the rest of the shrubs: the lavender, the garden fern, the ornamental grass and the hosta.

use artkit::botany::{fit_to, palette, polar};
use artkit::far::{is_low, low};
use artkit::forms::by;
use artkit::materials::Mat;
use artkit::math::{Axis, M4, PI, V3, atan2, cos, degrees, radians, sin, v3};
use artkit::parts::{join, join_all, new_parts, settle};
use artkit::py;
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{
    blade, cone, cupped, cut_leaf, florets, leaf, lump, outline, pinnate, spiral, squash, sweep,
    tube, twig,
};

/// Lavender: a grey-green mound of narrow leaves and a spray of thin stems, each with a spike of purple whorls; a
/// few at the front, which are cut.
pub fn lavender() -> Vec<Obj> {
    // spikes all `bloom`: the colour part
    let p = palette(&[
        ("leaf", "#748C70"),
        ("leaf_light", "#82987C"),
        ("stem", "#7E947A"),
        ("stalk", "#7E947A"),
        ("bloom", "#7E64B8"),
    ]);
    let mut rng = Random::new(75);
    lump!(
        "mound",
        0.29,
        (0, 0, 0),
        p.get("leaf"),
        scale = (1, 1, 0.55),
        seed = 1.0,
        segments = 10,
        rings = 6
    );
    let mut r2 = Random::new(175);
    // narrow leaves standing out of it
    for i in 0..22 {
        let a = i as f64 * 137.5 + r2.uniform(-10.0, 10.0);
        let rr = r2.uniform(0.07, 0.2);
        let base = V3::from(polar(rr, a, 0.04 + (0.24 - rr) * 0.5));
        let (yaw, tilt, length) = (
            a + r2.uniform(-25.0, 25.0),
            r2.uniform(35.0, 70.0),
            r2.uniform(0.08, 0.13),
        );
        if is_low() {
            continue;
        }
        blade!(
            format!("needle{i}"),
            base,
            yaw,
            tilt,
            length,
            0.014,
            [p.get("leaf"), p.get("leaf_light")][i % 2],
            droop = 0.15
        );
    }
    let bloom = p.get("bloom");
    // (the stems' spots, drawn before any of them is made)
    let pts: Vec<(usize, (f64, f64, f64))> = {
        let spots = spiral!(24, 0.27, &mut rng);
        spots.into_iter().enumerate().collect()
    };
    let mut spray = |points: &[(usize, (f64, f64, f64))], stem_mat: Mat, cut: bool| {
        for &(i, (x, y, _)) in points {
            let heading = degrees(atan2(x, y));
            let lean = radians(rng.uniform(4.0, 22.0) * (0.4 + py::hypot(x, y) * 3.0));
            let d = v3(
                sin(radians(heading)) * sin(lean),
                cos(radians(heading)) * sin(lean),
                cos(lean),
            );
            let base = v3(x * 0.5, y * 0.5, 0.16);
            // (English lavender in flower stands 0.3-0.6 m)
            let length = rng.uniform(0.4, 0.56) * 0.5;
            let tip = base + d * length;
            let along = d.to_track_quat_zy().to_matrix().to_4x4();
            if is_low() {
                let stem = cone!(
                    format!("stem{i}"),
                    0.012,
                    0.002,
                    length + 0.14,
                    material = bloom,
                    verts = 3
                );
                stem.edit(|o| {
                    o.mesh
                        .transform(&(M4::translation(base + d * (length + 0.14) / 2) * along))
                });
                continue;
            }
            let pre = if cut { "cut_" } else { "" };
            tube!(
                format!("{pre}stem{i}"),
                &[base, tip],
                0.004,
                0.003,
                stem_mat,
                verts = 3
            );
            let spike = florets!(
                format!("{pre}spike{i}"),
                0.017,
                (0, 0, 0),
                bloom,
                bumps = 2.2,
                amount = 0.3,
                flat = 4.2,
                segments = 5,
                rings = 6,
                seed = i as f64 * 0.9
            );
            spike.edit(|o| o.mesh.transform(&(M4::translation(tip - d * 0.01) * along)));
        }
    };
    // the three nearest the front (-y)
    let mut front = pts.clone();
    front.sort_by(|a, b| a.1.1.total_cmp(&b.1.1));
    front.truncate(3);
    let rest: Vec<(usize, (f64, f64, f64))> =
        pts.iter().copied().filter(|p| !front.contains(p)).collect();
    spray(&rest, p.get("stem"), false);
    // (built here, so all of it is fitted together)
    let cut = new_parts(|| spray(&front, p.get("stalk"), true));
    let k = fit_to(&meshes(), 0.49, 0.56, 0.55);
    if is_low() {
        return settle(&[join_all("lavender")]);
    }
    let others: Vec<Obj> = meshes().into_iter().filter(|o| !cut.contains(o)).collect();
    let still = join(&others, "lavender", Some((0, 0, 0).into()));
    let stems = join(&cut, "stems", Some((0.0, -0.12 * k[1], 0.2 * k[2]).into()));
    settle(&[still, stems]);
    squash!(stems, "cut", 0.001, seconds = 0.8);
    vec![still, stems]
}

/// Garden fern: a shuttlecock of arching fronds, each cut into leaflets along its length, round a low tuft, with a
/// few young fronds still coiled up in the middle.
pub fn garden_fern() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", "#5E9A3E"),
        ("leaf_dark", "#558E38"),
        ("leaf_light", "#78B24E"),
    ]);
    let mut rng = Random::new(76);
    lump!(
        "crown",
        0.09,
        (0, 0, 0),
        p.get("leaf_dark"),
        scale = (1, 1, 0.7),
        segments = 8,
        rings = 5
    );
    for i in 0..8 {
        let yaw = 90.0 + i as f64 / 8.0 * 360.0 + rng.uniform(-12.0, 12.0);
        let (tilt, length, droop, twist) = (
            rng.uniform(42.0, 56.0),
            rng.uniform(0.54, 0.64),
            rng.uniform(0.38, 0.48),
            rng.uniform(-15.0, 15.0),
        );
        cut_leaf!(
            format!("frond{i}"),
            (0, 0, 0.1),
            yaw,
            tilt,
            length,
            &pinnate!(12, 0.3, "lance", lean = 0.5, fill = 0.8),
            [p.get("leaf"), p.get("leaf_dark"), p.get("leaf")][i % 3],
            droop = droop,
            fold = 0.12,
            ribs = 6,
            twist = twist
        );
        let (tilt, length) = (rng.uniform(65.0, 78.0), rng.uniform(0.32, 0.38));
        if i % 2 != 0 {
            // four inner fronds
            continue;
        }
        cut_leaf!(
            format!("inner{i}"),
            (0, 0, 0.14),
            yaw + 20.0,
            tilt,
            length,
            &pinnate!(8, 0.2, "lance", lean = 0.5, fill = 0.8),
            p.get("leaf_light"),
            droop = 0.45,
            fold = 0.12,
            ribs = 4
        );
    }
    // fiddleheads: young fronds still coiled
    for i in 0..low(2, 0) {
        let a = i as f64 * 2.1;
        let base = v3(0.03 * cos(a), 0.03 * sin(a), 0.08);
        let mut pts = vec![base, base + v3(0, 0, 0.12)];
        for k in 0..6 {
            let t = k as f64 / 5.0 * PI * 1.6;
            let rr = 0.03 * (1.0 - k as f64 / 7.0);
            pts.push(
                base + v3(
                    cos(a) * rr * sin(t),
                    sin(a) * rr * sin(t),
                    0.15 + rr * cos(t),
                ),
            );
        }
        sweep!(
            format!("fiddle{i}"),
            &pts,
            0.008,
            material = p.get("leaf_light"),
            verts = 4,
            scale = by(|w| 1.0 - 0.5 * w)
        );
    }
    fit_to(&meshes(), 0.793, 0.478, 0.932);
    settle(&[join_all("fern")])
}

/// Ornamental grass: a fountain of long, thin, arching blades and feathery pale plumes on tall stems.
pub fn ornamental_grass() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", "#9AAA5A"),
        ("leaf_dark", "#8A9E4E"),
        ("leaf_light", "#B2B86A"),
        ("stem", "#A6B062"),
        ("bloom", "#E2D6A8"),
        ("stalk", "#B8B070"),
    ]);
    let mut rng = Random::new(77);
    lump!(
        "tuft",
        0.15,
        (0, 0, 0),
        p.get("leaf_dark"),
        scale = (1, 1, 0.5),
        segments = 8,
        rings = 5
    );
    let cols = [
        p.get("leaf"),
        p.get("leaf_light"),
        p.get("leaf_dark"),
        p.get("stem"),
    ];
    for (i, (x, y, _)) in spiral!(54, 0.13, &mut rng).into_iter().enumerate() {
        let out = py::hypot(x, y);
        let (length, yaw, droop) = (
            rng.uniform(0.7, 1.08),
            degrees(atan2(x, y)) + rng.uniform(-25.0, 25.0),
            rng.uniform(0.18, 0.3),
        );
        let tilt = 88.0 - out * 150.0 - rng.uniform(0.0, 10.0);
        if is_low() && i % 3 != 0 {
            continue;
        }
        leaf!(
            format!("blade{i}"),
            (x, y, 0),
            yaw,
            tilt,
            length,
            low(0.024, 0.04),
            cols[i % 4],
            shape = "strap",
            droop = droop,
            thick = 0.004,
            fold = 0.35,
            rows = 4,
            ring = 3
        );
    }
    for i in 0..7 {
        let yaw = (i * 360) as f64 / 7.0 + rng.uniform(-15.0, 15.0);
        let lean = radians(rng.uniform(6.0, 16.0));
        let d = v3(
            sin(radians(yaw)) * sin(lean),
            cos(radians(yaw)) * sin(lean),
            cos(lean),
        );
        let tip = d * 0.96;
        twig!(
            format!("plume_stem{i}"),
            &[v3(0, 0, 0), d * 0.5, tip],
            0.004,
            0.003,
            p.get("stem"),
            verts = 3,
            res = 1
        );
        let plume = florets!(
            format!("plume{i}"),
            0.032,
            (0, 0, 0),
            p.get("bloom"),
            bumps = 2.6,
            amount = 0.3,
            flat = 4.0,
            segments = 6,
            rings = 7,
            seed = i as f64 * 1.1
        );
        // nodding a little
        let bend = M4::rotation(radians(12.0), Axis::X);
        plume.edit(|o| {
            o.mesh.transform(
                &(M4::translation(tip - d * 0.03)
                    * d.to_track_quat_zy().to_matrix().to_4x4()
                    * bend),
            )
        });
    }
    fit_to(&meshes(), 1.218, 1.21, 1.28);
    settle(&[join_all("grass")])
}

/// Hosta: a clump of broad heart-shaped leaves with wavy edges, folded along their midribs, on stalks, and a few
/// tall stems of nodding lilac bells.
pub fn hosta() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", "#5E8A5E"),
        ("leaf_dark", "#4E7A4A"),
        ("leaf_light", "#8EAE6C"),
        ("stem", "#6A9464"),
        ("bloom", "#B8A6D8"),
    ]);
    let mut rng = Random::new(78);
    lump!(
        "crown",
        0.13,
        (0, 0, 0),
        p.get("leaf_dark"),
        scale = (1, 1, 0.5),
        segments = 8,
        rings = 5
    );
    for i in 0..14 {
        let yaw = i as f64 / 14.0 * 720.0 + rng.uniform(-10.0, 10.0);
        let up = i >= 7;
        let length = rng.uniform(0.26, 0.32);
        let reach = if up { 0.1 } else { 0.13 };
        let rise = radians(if up {
            rng.uniform(40.0, 55.0)
        } else {
            rng.uniform(22.0, 32.0)
        });
        let tilt = if up {
            rng.uniform(25.0, 40.0)
        } else {
            rng.uniform(5.0, 15.0)
        };
        let base = v3(0, 0, 0.05);
        let end = base + V3::from(polar(reach * cos(rise), yaw, reach * sin(rise) + 0.04));
        twig!(
            format!("stalk{i}"),
            &[base, base.lerp(end, 0.5) + v3(0, 0, 0.02), end],
            0.008,
            0.006,
            p.get("stem"),
            verts = 3,
            res = 1
        );
        cut_leaf!(
            format!("leaf{i}"),
            end,
            yaw,
            tilt,
            length,
            &outline!("heart", 0.2, n = 7, wave = 0.06, waves = 6),
            [p.get("leaf"), p.get("leaf_light"), p.get("leaf")][i % 3],
            droop = 0.3,
            fold = 0.32,
            ribs = 6,
            thick = 0.006
        );
    }
    for i in 0..3 {
        let top = v3(
            rng.uniform(-0.08, 0.08),
            rng.uniform(-0.08, 0.08),
            // (a medium hosta's scapes just over its leaves: 0.3-0.5 m in all)
            rng.uniform(0.66, 0.8) * 0.55,
        );
        twig!(
            format!("scape{i}"),
            &[v3(0, 0, 0.1), top * 0.5 + v3(0, 0, 0.05), top],
            0.005,
            0.004,
            p.get("stem"),
            verts = 3
        );
        // bells nodding off one side, up the stem
        for k in 0..6 {
            if is_low() && k % 3 != 0 {
                continue;
            }
            let z = top.z as f64 - 0.02 - k as f64 * 0.035;
            let turn = (i * 90 + k * 50) as f64;
            let side = V3::from(polar(0.022, turn, 0.0));
            let at = v3(top.x, top.y, z) + side;
            cupped!(
                format!("bell{i}_{k}"),
                &[
                    (0.0, 0.0),
                    (0.006, 0.0),
                    (0.013, 0.02),
                    (0.018, 0.036),
                    (0.012, 0.033),
                    (0.0, 0.012)
                ],
                at,
                p.get("bloom"),
                yaw = turn,
                tilt = 150,
                segments = 5
            );
        }
    }
    fit_to(&meshes(), 0.621, 0.395, 0.675);
    settle(&[join_all("hosta")])
}
