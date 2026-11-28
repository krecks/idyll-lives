//! TT2 tableware.

use super::M;
use super::common::{disc, done, oval, spline};
use artkit::kit::fillet;
use artkit::materials::Mat;
use artkit::math::{PI, V3, atan2, cos, cosd, pow, radians, sin, sind};
use artkit::parts::new_parts;
use artkit::scene::{Obj, meshes};
use artkit::{join, rod, settle, slide, sphere, sweep};

/// An apple: round shoulders over a narrower base, a dimple at the top and its stalk.
fn _apple(name: &str, at: (f64, f64, f64), material: Mat, s: f64, turn: f64) {
    let (x, y, z) = at;
    turned!(
        name,
        &[
            (0.0, 0.006 * s),
            (0.02 * s, 0.0),
            (0.037 * s, 0.018 * s),
            (0.04 * s, 0.038 * s),
            (0.032 * s, 0.06 * s),
            (0.014 * s, 0.067 * s),
            (0.0, 0.061 * s)
        ],
        material,
        at = (x, y, z),
        verts = 8,
        rot = (turn, 0.0, 0.0)
    );
    sweep!(
        format!("{name}_stalk"),
        &[(x, y, z + 0.058 * s), (x + 0.003, y, z + 0.078 * s)],
        0.0022,
        material = M.stalk,
        verts = 4
    );
}

/// A banana lying from Blender point `a` to `b`, bowed up `lift` in the middle: five-sided (as a banana's ridges),
/// tapering to a dark tip at each end.
fn _banana(name: &str, a: (f64, f64, f64), b: (f64, f64, f64), lift: f64, material: Mat) {
    let pts: Vec<V3> = [0.0, 0.12, 0.35, 0.65, 0.88, 1.0]
        .into_iter()
        .map(|t| V3::from(a).lerp(V3::from(b), t) + V3::from((0.0, 0.0, lift * sin(PI * t))))
        .collect();
    // five-sided, its ridges soft: two points round each, all on the circle, so each turns 36 degrees
    let section: Vec<(f64, f64)> = (0..10)
        .map(|k| {
            let a = radians(72.0 * (k / 2) as f64 + if k % 2 == 0 { -12.0 } else { 12.0 });
            (0.017 * cos(a), 0.017 * sin(a))
        })
        .collect();
    sweep!(
        name,
        &pts,
        0.0,
        material = material,
        profile = section,
        scale = [0.25, 0.8, 1.0, 1.0, 0.75, 0.2],
        angle = 50
    );
    let n = pts.len();
    for (k, (p, q)) in [(pts[0], pts[1]), (pts[n - 1], pts[n - 2])]
        .into_iter()
        .enumerate()
    {
        let d = (p - q).normalized();
        rod!(
            format!("{name}_tip{k}"),
            p - d * 0.004,
            p + d * 0.006,
            0.0045,
            material = M.stalk,
            verts = 4
        );
    }
}

/// A glazed fruit bowl on a foot ring, with two apples (red and green, stalks), two oranges and two bananas lying
/// over them.
pub fn fruit_bowl() -> Vec<Obj> {
    turned!(
        "bowl",
        &[
            (0.052, 0.0),
            (0.056, 0.012),
            (0.1, 0.04),
            (0.136, 0.084),
            (0.14, 0.09),
            (0.132, 0.09),
            (0.095, 0.046),
            (0.0, 0.03)
        ],
        M.fruit_bowl,
        verts = 16
    );
    _apple("apple_red", (0.05, 0.03, 0.034), M.red, 1.0, 0.0);
    _apple(
        "apple_green",
        (-0.035, 0.065, 0.036),
        M.apple_green,
        1.0,
        8.0,
    );
    for (i, (x, y)) in [(-0.055, -0.03), (0.035, -0.058)].into_iter().enumerate() {
        ball!(
            format!("orange{i}"),
            0.04,
            (x, y, 0.074),
            M.orange,
            scale = (1, 1, 0.94),
            seg = 8,
            rings = 5
        );
        sphere!(
            format!("navel{i}"),
            0.006,
            at = (x, y, 0.111),
            material = M.stalk,
            segments = 4,
            rings = 3,
            scale = (1, 1, 0.4)
        );
    }
    _banana(
        "banana0",
        (-0.085, -0.03, 0.124),
        (0.07, 0.05, 0.134),
        0.05,
        M.yellow,
    );
    _banana(
        "banana1",
        (-0.07, -0.06, 0.114),
        (0.085, 0.02, 0.146),
        0.045,
        M.yellow,
    );
    done("fruitBowl", false)
}

/// A plate: a foot ring under it, the well, a rim rising to its rolled edge.
fn _plate(name: &str, r: f64, z: f64, material: Mat, verts: usize) -> Obj {
    let k = r / 0.135;
    turned!(
        name,
        &[
            (0.075 * k, z),
            (0.08 * k, z + 0.004),
            (0.128 * k, z + 0.013),
            (0.135 * k, z + 0.017),
            (0.132 * k, z + 0.019),
            (0.09 * k, z + 0.01),
            (0.0, z + 0.009)
        ],
        material,
        verts = verts
    )
}

/// The coloured band round a plate's rim (`_plate`'s numbers), a hair proud of it.
fn _rim_band(name: &str, r: f64, z: f64, material: Mat, verts: usize) -> Obj {
    let k = r / 0.135;
    let h = |rr: f64| z + 0.01 + (rr - 0.09 * k) * (0.009 / (0.042 * k)) + 0.0007;
    let band = turned!(
        name,
        &[(0.116 * k, h(0.116 * k)), (0.124 * k, h(0.124 * k))],
        material,
        verts = verts,
        caps = false
    );
    // an open band: make it face up, where it's seen
    let up = artkit::py::sum(
        band.read(|o| o.mesh.face_normals())
            .iter()
            .map(|n| n.z as f64),
    );
    if up < 0.0 {
        band.edit(|o| o.mesh.flip_normals());
    }
    band
}

/// A table fork lying flat, its tines to the back: a handle narrowing to the neck, the head with four tines.
fn _fork(name: &str, at: (f64, f64, f64), material: Mat) {
    let (c, hw, top, gap) = ([0.0081, 0.0027, -0.0027, -0.0081], 0.0019, 0.08, 0.052);
    let mut pts = vec![
        (-0.006, -0.085),
        (0.006, -0.085),
        (0.0078, -0.074),
        (0.0062, -0.02),
        (0.0032, 0.015),
        (0.01, 0.04),
        (0.01, top),
    ];
    for (i, &x) in c.iter().enumerate() {
        pts.push((x - hw, top));
        if i < 3 {
            pts.extend([(x - hw, gap), (c[i + 1] + hw, gap), (c[i + 1] + hw, top)]);
        }
    }
    pts.extend([
        (-0.01, top),
        (-0.01, 0.04),
        (-0.0032, 0.015),
        (-0.0062, -0.02),
        (-0.0078, -0.074),
    ]);
    let mut kept = vec![pts[0]];
    kept.extend(
        (1..pts.len())
            .filter(|&i| pts[i] != pts[i - 1])
            .map(|i| pts[i]),
    );
    slab!(
        name,
        &kept,
        0.0025,
        at,
        material,
        plane = artkit::forms::Plane::XY
    );
}

/// A place setting: a dinner plate with a coloured band round its rim and a side plate on it, a linen napkin with a
/// fork on it, a knife, and a tumbler.
pub fn plate_set() -> Vec<Obj> {
    _plate("plate", 0.135, 0.0, M.plate, 24);
    _rim_band("rim", 0.135, 0.0, M.plate_rim, 24);
    _plate("side_plate", 0.095, 0.009, M.plate, 20);
    _rim_band("side_rim", 0.095, 0.009, M.plate_rim, 20);
    artkit::soft_box!(
        "napkin",
        (0.06, 0.17, 0.005),
        at = (-0.17, -0.01, 0.0025),
        material = M.napkin,
        round = 0.002,
        puff = 0.0015,
        round_segments = 1,
        flat = 1
    );
    _fork("fork", (-0.17, -0.005, 0.0062), M.chrome);
    slab!(
        "knife",
        &[
            (-0.0075, -0.1),
            (0.0075, -0.1),
            (0.008, -0.012),
            (0.006, -0.008),
            (0.009, 0.0),
            (0.009, 0.075),
            (0.006, 0.092),
            (0.0, 0.1),
            (-0.009, 0.096),
            (-0.009, -0.006),
            (-0.0075, -0.012)
        ],
        0.003,
        (0.17, 0.0, 0.0015),
        M.chrome,
        plane = artkit::forms::Plane::XY
    );
    turned!(
        "tumbler",
        &[
            (0.03, 0.0),
            (0.032, 0.004),
            (0.037, 0.1),
            (0.034, 0.1),
            (0.03, 0.014),
            (0.0, 0.012)
        ],
        M.glass_pale,
        at = (0.14, 0.15, 0),
        verts = 14
    );
    done("plateSet", true)
}

/// Lowers the vertices of `obj` at height `z` (Blender) into icing drips: `n` round its edge, up to `depth`.
fn _drips(obj: Obj, z: f64, depth: f64, n: f64, seed: f64) -> Obj {
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            if (v.z as f64 - z).abs() < 1e-4 {
                let a = atan2(v.y as f64, v.x as f64);
                let s = sin(n * a + seed);
                v.z = (v.z as f64 - depth * pow(if s > 0.0 { s } else { 0.0 }, 2.0)) as f32;
            }
        }
    });
    obj
}

/// A footed cake stand (a turned foot, a knop on its stem, the plate's raised edge), a layer cake on it: the
/// sponge, a band of filling, icing over the top running down in drips, six piped rosettes and a cherry.
pub fn cake_stand() -> Vec<Obj> {
    turned!(
        "stand",
        // (the plate's edge rolled round)
        &artkit::forms::round_at(
            &[
                (0.078, 0.0),
                (0.08, 0.008),
                (0.03, 0.022),
                (0.02, 0.06),
                (0.026, 0.072),
                (0.02, 0.084),
                (0.13, 0.092),
                (0.14, 0.1),
                (0.0, 0.102),
            ],
            7,
            0.001
        ),
        M.cake_stand,
        verts = 32
    );
    turned!(
        "sponge",
        &[(0.107, 0.101), (0.107, 0.192)],
        M.sponge,
        verts = 32
    );
    turned!(
        "filling",
        &[(0.1076, 0.144), (0.1076, 0.152)],
        M.icing,
        verts = 32,
        caps = false
    );
    _drips(
        turned!(
            "icing",
            &[
                (0.1095, 0.188),
                (0.1105, 0.198),
                (0.109, 0.212),
                (0.096, 0.218),
                (0.0, 0.219)
            ],
            M.icing,
            verts = 32
        ),
        0.188,
        0.026,
        7.0,
        0.0,
    );
    for i in 0..6 {
        let a = (i * 60 + 30) as f64;
        turned!(
            format!("rosette{i}"),
            &[(0.011, 0.0), (0.012, 0.004), (0.007, 0.011), (0.0, 0.016)],
            M.icing,
            at = (0.082 * cosd(a), 0.082 * sind(a), 0.216),
            verts = 5,
            rot = (0.0, 0.0, a)
        );
    }
    ball!("cherry", 0.016, (0, 0, 0.234), M.red, seg = 8, rings = 5);
    sweep!(
        "cherry_stalk",
        &[
            (0.0, 0.0, 0.247),
            (0.003, 0.002, 0.255),
            (0.008, 0.004, 0.262)
        ],
        0.0018,
        material = M.stalk,
        verts = 4
    );
    done("cakeStand", false)
}

/// A teacup on its saucer: the saucer's well, the cup's foot and flaring bowl with tea in it, a loop handle.
fn _cup(name: &str, x: f64, y: f64, z: f64, material: Mat, turn: f64) {
    turned!(
        format!("{name}_saucer"),
        &[
            (0.0, z),
            (0.03, z + 0.001),
            (0.044, z + 0.006),
            (0.047, z + 0.01),
            (0.043, z + 0.01),
            (0.03, z + 0.005),
            (0.0, z + 0.004)
        ],
        material,
        at = (x, y, 0.0),
        verts = 12
    );
    turned!(
        name,
        &[
            (0.016, z + 0.004),
            (0.02, z + 0.008),
            (0.03, z + 0.038),
            (0.032, z + 0.045),
            (0.029, z + 0.045),
            (0.026, z + 0.022),
            (0.0, z + 0.016)
        ],
        material,
        at = (x, y, 0.0),
        verts = 12
    );
    disc(
        &format!("{name}_tea"),
        0.0285,
        (x, y, z + 0.038),
        M.coffee,
        10,
        None,
    );
    let (c, s) = (cosd(turn), sind(turn));
    let lp = fillet(
        &[
            (0.026, z + 0.038, 0.0),
            (0.042, z + 0.04, 0.0),
            (0.042, z + 0.022, 0.0),
            (0.024, z + 0.017, 0.0),
        ],
        0.007,
        1,
    );
    let pts: Vec<(f64, f64, f64)> = lp
        .iter()
        .map(|&(u, v, _)| (x + u * c, y + u * s, v))
        .collect();
    sweep!(
        format!("{name}_handle"),
        &pts,
        0.0035,
        material = material,
        verts = 5
    );
}

/// A tea tray with a gallery round it and hand holes in its ends, a round teapot (a band round its belly, a
/// curving spout, a loop handle, the lid and its knob) and two cups of tea on saucers.
pub fn teapot_set() -> Vec<Obj> {
    slab!(
        "tray",
        &artkit::forms::rounded_rect(0.39, 0.25, 0.02, 2),
        0.008,
        (0, 0, 0.004),
        M.tray,
        plane = artkit::forms::Plane::XY,
        bevel = 0.002
    );
    for s in [-1, 1] {
        box_!(
            format!("tray_side{s}"),
            (0.37, 0.01, 0.026),
            (0.0, s as f64 * 0.123, 0.013),
            M.tray,
            bevel = 0.003,
            segs = 1
        );
        let end = artkit::forms::smooth_outline(
            &[
                (-0.13, 0.0),
                (0.13, 0.0),
                (0.13, 0.032),
                (0.08, 0.05),
                (-0.08, 0.05),
                (-0.13, 0.032),
            ],
            1,
            true,
        );
        let hole: Vec<(f64, f64)> = artkit::forms::rounded_rect(0.08, 0.016, 0.008, 2)
            .into_iter()
            .map(|(u, v)| (u, v + 0.032))
            .collect();
        slab!(
            format!("tray_end{s}"),
            &end,
            0.012,
            (s as f64 * 0.194, 0.0, 0.0),
            M.tray,
            plane = artkit::forms::Plane::YZ,
            bevel = 0.002,
            holes = vec![hole]
        );
    }
    let (x, y) = (0.07, 0.02);
    turned!(
        "pot",
        &[
            (0.045, 0.008),
            (0.05, 0.014),
            (0.068, 0.042),
            (0.073, 0.075),
            (0.066, 0.11),
            (0.05, 0.13),
            (0.04, 0.136)
        ],
        M.teapot,
        at = (x, y, 0.0),
        verts = 14
    );
    turned!(
        "pot_band",
        &[(0.0738, 0.068), (0.0735, 0.08)],
        M.teapot_trim,
        at = (x, y, 0.0),
        verts = 14,
        caps = false
    );
    turned!(
        "pot_lid",
        &[
            (0.042, 0.134),
            (0.045, 0.138),
            (0.037, 0.147),
            (0.02, 0.151),
            (0.0, 0.152)
        ],
        M.teapot,
        at = (x, y, 0.0),
        verts = 14
    );
    turned!(
        "pot_knob",
        &[(0.006, 0.15), (0.012, 0.157), (0.01, 0.164), (0.0, 0.167)],
        M.teapot_trim,
        at = (x, y, 0.0),
        verts = 8
    );
    sweep!(
        "spout",
        &[
            (x - 0.05, y, 0.05),
            (x - 0.085, y, 0.07),
            (x - 0.105, y, 0.1),
            (x - 0.122, y, 0.124)
        ],
        0.014,
        material = M.teapot,
        verts = 8,
        scale = [1.5, 1.0, 0.72, 0.58],
        resolution = 2
    );
    sweep!(
        "pot_handle",
        &fillet(
            &[
                (x + 0.058, y, 0.115),
                (x + 0.1, y, 0.118),
                (x + 0.108, y, 0.07),
                (x + 0.066, y, 0.038)
            ],
            0.02,
            2
        ),
        0.0065,
        material = M.teapot,
        verts = 6
    );
    _cup("cup_a", -0.11, -0.06, 0.008, M.teapot, 200.0);
    _cup("cup_b", -0.1, 0.07, 0.008, M.teapot, 160.0);
    done("teapotSet", false)
}

/// A mug: a thick rim, coffee in it, a loop handle in one piece.
fn _mug(name: &str, x: f64, y: f64, turn: f64, material: Mat) {
    turned!(
        name,
        &[
            (0.036, 0.0),
            (0.04, 0.005),
            (0.04, 0.087),
            (0.0385, 0.09),
            (0.035, 0.09),
            (0.034, 0.076)
        ],
        material,
        at = (x, y, 0.0),
        verts = 12
    );
    disc(
        &format!("{name}_coffee"),
        0.0342,
        (x, y, 0.078),
        M.coffee,
        10,
        None,
    );
    let (c, s) = (cosd(turn), sind(turn));
    let lp = fillet(
        &[
            (0.035, 0.074, 0.0),
            (0.066, 0.07, 0.0),
            (0.066, 0.026, 0.0),
            (0.035, 0.022, 0.0),
        ],
        0.014,
        2,
    );
    let pts: Vec<(f64, f64, f64)> = lp
        .iter()
        .map(|&(u, v, _)| (x + u * c, y + u * s, v))
        .collect();
    bar!(
        format!("{name}_handle"),
        &pts,
        oval(0.009, 0.014, 6),
        material,
        up = (-s, c, 0.0)
    );
}

/// Three mugs of coffee, their handles every way, a tea bag's string and tag hanging over one.
pub fn mugs() -> Vec<Obj> {
    _mug("mug_a", 0.068, -0.03, 150.0, M.mug_a);
    _mug("mug_b", -0.068, -0.04, 20.0, M.mug_b);
    _mug("mug_c", 0.0, 0.06, 80.0, M.mug_c);
    let a = radians(232.0);
    let (rx, ry) = (-0.068 + 0.04 * artkit::math::cos(a), -0.04 + 0.04 * sin(a));
    sweep!(
        "string",
        &[
            (-0.068, -0.04, 0.08),
            (rx, ry, 0.0885),
            (
                rx + 0.0008 * artkit::math::cos(a),
                ry + 0.0008 * sin(a),
                0.06
            )
        ],
        0.0008,
        material = M.paper,
        verts = 3
    );
    box_!(
        "tag",
        (0.012, 0.002, 0.016),
        (
            rx + 0.0016 * artkit::math::cos(a),
            ry + 0.0016 * sin(a),
            0.052
        ),
        M.yellow,
        rot = (0, 0, 232 - 90)
    );
    done("mugs", false)
}

/// A wine glass, open at its rim (see-through): a foot, a thin stem, the bowl; red wine in it.
fn _wine_glass(name: &str, x: f64, y: f64) {
    turned!(
        name,
        &{
            let mut rows = vec![
                (0.0, 0.0),
                (0.03, 0.0),
                (0.03, 0.003),
                (0.006, 0.008),
                (0.0045, 0.07),
            ];
            // (the bowl on a curve)
            rows.extend(
                spline(
                    &[
                        (0.0045, 0.07),
                        (0.014, 0.086),
                        (0.028, 0.1),
                        (0.035, 0.125),
                        (0.033, 0.17),
                    ],
                    2,
                )
                .split_off(1),
            );
            rows.push((0.031, 0.169));
            rows
        },
        M.glass,
        at = (x, y, 0.0),
        verts = 20,
        caps = false
    );
    turned!(
        format!("{name}_wine"),
        &[(0.0, 0.088), (0.02, 0.098), (0.03, 0.114), (0.0318, 0.122)],
        M.wine,
        at = (x, y, 0.0),
        verts = 20
    );
}

/// A bottle of red (shoulders, a long neck, a label, the foil capsule) and two glasses poured.
pub fn wine_glasses() -> Vec<Obj> {
    turned!(
        "bottle",
        &[
            (0.036, 0.0),
            (0.04, 0.005),
            (0.04, 0.19),
            (0.032, 0.222),
            (0.016, 0.248),
            (0.0145, 0.26),
            (0.0145, 0.3)
        ],
        M.bottle,
        at = (0, 0.07, 0),
        verts = 16
    );
    turned!(
        "label",
        &[(0.0406, 0.075), (0.0406, 0.15)],
        M.paper,
        at = (0, 0.07, 0),
        verts = 16,
        caps = false
    );
    turned!(
        "foil",
        &[
            (0.0152, 0.252),
            (0.0158, 0.256),
            (0.0156, 0.299),
            (0.0, 0.301)
        ],
        M.wine,
        at = (0, 0.07, 0),
        verts = 8
    );
    _wine_glass("glass_a", -0.07, -0.04);
    _wine_glass("glass_b", 0.07, -0.05);
    done("wineGlasses", false)
}

const CANDLES: [(f64, f64); 2] = [(0.05, 0.18), (-0.05, 0.14)]; // (x, the stick's height)
const FLAME_RISE: f64 = 0.034; // a flame's 3 cm, from inside the candle's top to over its wick

/// Two turned brass candlesticks (a stepped foot, a knop on the stem, a drip pan at the top) with tall candles,
/// a run of wax down one. By day the wicks are dark; from dusk the flames rise out of the candles' tops (`lit`, a
/// `night` clip) and glow (`glow`), the brightest thing on the table.
pub fn candlesticks() -> Vec<Obj> {
    let still = new_parts(|| {
        for (i, &(x, h)) in CANDLES.iter().enumerate() {
            turned!(
                format!("stick{i}"),
                &[
                    (0.04, 0.0), // a foot 8 cm across, a candlestick's least
                    (0.04, 0.006),
                    (0.026, 0.016),
                    (0.009, 0.03),
                    (0.008, h * 0.45),
                    (0.016, h * 0.5),
                    (0.008, h * 0.56),
                    (0.008, h - 0.022),
                    (0.022, h - 0.012),
                    (0.022, h),
                    (0.0, h)
                ],
                M.brass,
                at = (x, 0.0, 0.0),
                verts = 10
            );
            turned!(
                format!("candle{i}"),
                &[
                    (0.011, h - 0.01),
                    (0.011, h + 0.132),
                    (0.0095, h + 0.139),
                    (0.0, h + 0.14)
                ],
                M.candle,
                at = (x, 0.0, 0.0),
                verts = 8
            );
            sweep!(
                format!("wick{i}"),
                &[
                    (x, 0.0, h + 0.138),
                    (x, 0.0, h + 0.146),
                    (x + 0.002, 0.0, h + 0.149)
                ],
                0.0012,
                material = M.dark,
                verts = 3
            );
        }
        let (x, h) = CANDLES[0];
        sweep!(
            "drip",
            &[
                (x - 0.008, -0.007, h + 0.139),
                (x - 0.0105, -0.0035, h + 0.125),
                (x - 0.011, -0.003, h + 0.1)
            ],
            0.0028,
            material = M.candle,
            verts = 4,
            scale = [1.0, 0.9, 0.7]
        );
    });
    join!(&still, "candlesticks", origin = (0, 0, 0));
    // the flames, built lit and sunk into the candles at rest (hidden inside the wax, 1.3 cm across to their
    // 2.2): the same rise lifts both, the candles being the same length
    let lit = new_parts(|| {
        for (i, &(x, h)) in CANDLES.iter().enumerate() {
            turned!(
                format!("flame{i}"),
                &[
                    (0.0, 0.0),
                    (0.0055, 0.004),
                    (0.0065, 0.009),
                    (0.0035, 0.018),
                    (0.0, 0.026)
                ],
                M.flame,
                at = (x, 0.0, h + 0.145 - FLAME_RISE),
                verts = 6
            );
        }
    });
    let flames = join!(&lit, "flames", origin = (0, 0, 0));
    settle!();
    slide!(flames, "lit", (0.0, 0.0, FLAME_RISE), seconds = 0.6);
    meshes()
}
