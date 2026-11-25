//! What the nature lover pack is built from: its palette, flower pots, leaf rosettes and flower faces.

use artkit::materials::Mat;
use artkit::place::GameFrame;
use artkit::rand::Random;
use artkit::{fsphere, leaf, palette, revolve, star};

palette! { M:
    cedar = ("#C27C4E", satin),      // plain cedar: the veggie bed, the feeder, the glasshouse bench; the main part
    cedar_dark = ("#A3653E", satin),
    cedar_wood = ("#C27C4E", wood),  // textured cedar: the hammock's stand and the shower's slats
    cedar_wood_dark = ("#A3653E", wood),
    bench = ("#D9B98C", satin),      // the potting bench's worktop and shelves
    sage = ("#8FA88A", satin),       // the potting bench's frame, the feeder's roof trim
    walnut = ("#8A5E3C", satin),     // the terrarium's pedestal
    frame = ("#F2F2F0", satin),      // the glasshouse's white frame
    stone = ("#C9C2B4", satin),
    gravel = ("#B8A88E", matte),
    soil = ("#5A4030", matte),
    terra = ("#C7744F", satin),      // terracotta pots
    porcelain = ("#F2F2F0", gloss),
    canvas = ("#EDE4D3", fabric),    // the hammock's sling
    stripe = ("#7FA87A", fabric),
    rope = ("#D9CDB4", fabric),
    pillow = ("#E8B07A", fabric),
    cushion = ("#E8B07A", satin),    // the glasshouse bench's cushions
    copper = ("#C27A4A", metal),
    steel = ("#8A9096", metal),
    can = ("#8EA5A0", metal),
    dark = ("#3A3D41", satin),
    label = ("#F2F2F0", matte),
    towel = ("#F2F2F0", satin),
    soap = ("#F2D6A0", satin),
    sack = ("#D8C49C", matte),
    seed = ("#C9A66B", matte),
    house = ("#E8E0CF", satin),      // the feeder's little house
    roof = ("#5E7F9A", satin),
    tube = ("#E6F0F2", gloss),       // the seed tube, plain (no glass draw)
    pebble = ("#B8B4AE", satin),
    pebble_b = ("#9EA0A6", satin),
    rug = ("#7FA87A", matte),
    water = ("#BFE3F2", gloss),
    mister = ("#8EC6D8", gloss),
    glass = ("#DCEBEA", glass),
    // greens: foliage where it takes the model's textured draw, satin where the budget is spent elsewhere
    leaf = ("#5E8F4E", foliage),
    leaf_light = ("#78A85A", foliage),
    leaf_pale = ("#9BCB5A", foliage),
    leaf_dark = ("#4F8A45", foliage),
    moss = ("#4F8A3C", foliage),
    green = ("#5E8F4E", satin),
    green_light = ("#78A85A", satin),
    green_dark = ("#4F8A45", satin),
    stem = ("#5C7A3A", satin),
    // fruit, flowers and birds
    tomato = ("#D9483B", gloss, roughness(0.3)),
    carrot = ("#F28C28", satin),
    courgette = ("#4F7D3A", satin),
    pink = ("#F28DB2", satin),
    yellow = ("#F6C744", satin),
    lilac = ("#B8A1E3", satin),
    white = ("#FFFFFF", satin),
    orchid = ("#E88CC2", satin),
    orchid_b = ("#C98BE0", satin),
    bird_blue = ("#4F7FC4", satin),
    bird_breast = ("#D9733B", satin),
    bird_yellow = ("#E8C23A", satin),
    beak = ("#E8B34A", satin),
    eye = ("#1E1C1A", gloss),        // the birds' eyes
    toadstool = ("#E4572E", satin),
}

/// The wall's face in a 1-deep footprint, game depth.
pub const WALL: f64 = -0.43;

/// A flower pot standing on `y`, `d` across its rim: a tapered wall under a rolled rim, open below the soil (never
/// seen), and its soil a little mound inside the rim; returns the soil's height.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn _pot(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    d: f64,
    h: f64,
    material: Option<Mat>,
    soil: bool,
    segments: Option<usize>,
) -> f64 {
    let r = d / 2.0;
    let seg = segments.unwrap_or(if d < 0.15 { 10 } else { 14 });
    revolve!(
        name,
        &[
            (r * 0.72, 0.0),
            (r * 0.94, h * 0.78),
            (r * 1.06, h * 0.8),
            (r * 1.08, h * 0.9),
            (r * 1.04, h),
            (r * 0.9, h),
            (r * 0.86, h * 0.8)
        ],
        at = f.at(x, y, z),
        material = material.unwrap_or(M.terra),
        segments = seg
    );
    if soil {
        revolve!(
            format!("{name}_soil"),
            &[
                (r * 0.9, h * 0.8),
                (r * 0.88, h * 0.86),
                (r * 0.5, h * 0.88),
                (0.0, h * 0.9)
            ],
            at = f.at(x, y, z),
            material = M.soil,
            segments = seg
        );
    }
    y + h * 0.88
}

/// `n` leaves fanning out from (x, y, z) all round, alternately steeper and flatter (`tilt`), longer outside:
/// a lettuce, a herb, a clump at a flower's foot.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn _rosette(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    n: usize,
    length: f64,
    width: f64,
    mats: &[Mat],
    tilt: (f64, f64),
    droop: f64,
    shape: &'static str,
    fold: f64,
    notch: Option<(usize, f64)>,
    rows: usize,
    ring: usize,
    seed: i128,
    spin: f64,
) {
    let mut rnd = Random::new(seed);
    let base = f.at(x, y, z);
    for k in 0..n {
        let inner = k % 2;
        // (the arguments in the script's order: the yaw's draw before the length's)
        let yaw = spin + k as f64 * 137.5 + rnd.uniform(-12.0, 12.0);
        let length = length * (if inner != 0 { 0.78 } else { 1.0 }) * rnd.uniform(0.9, 1.1);
        leaf!(
            format!("{name}{k}"),
            base,
            yaw,
            if inner != 0 { tilt.1 } else { tilt.0 },
            length,
            width,
            mats[k % mats.len()],
            shape = shape,
            droop = droop * (if inner != 0 { 0.6 } else { 1.0 }),
            fold = fold,
            rows = rows,
            ring = ring,
            notch = notch
        );
    }
}

/// A flower's face (`plants.star`) on (x, y, z), a dot of `centre` in its middle.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn _flower(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    material: Mat,
    centre: Option<Mat>,
    petals: usize,
    tilt: f64,
    yaw: f64,
) {
    star!(
        name,
        f.at(x, y, z),
        r,
        petals,
        material,
        yaw = yaw,
        tilt = tilt,
        inner = 0.45,
        depth = 0.006,
        cup = 0.25
    );
    if let Some(centre) = centre {
        fsphere!(
            f,
            format!("{name}_eye"),
            r * 0.3,
            (x, y + 0.006, z),
            centre,
            scale = (1, 1, 0.6),
            segments = 5,
            rings = 3
        );
    }
}
