//! What the kind pack is built from: its palette, fluted rings, the tea set and little shrubs.

use artkit::materials::Mat;
use artkit::math::{cosd, sind};
use artkit::parts::numbered;
use artkit::place::GameFrame;
use artkit::rand::Random;
use artkit::{boulder, fsphere, fsweep, jitter, kit, palette, torus};

palette! { M:
    body = ("#6F8F6A", fabric), // the settee's sage velvet; its main part
    enamel = ("#F1E6C8", gloss), // the range's buttercream enamel; the counter's main part
    cabinet = ("#A8BFA0", satin), // the counter's sage cupboards
    cabinet_panel = ("#B9CDB2", satin),
    butcher = ("#C79A62", satin), // the counter's top and the wall shelf, plain (the towel takes the textured draw)
    towel = ("#E3A0A0", fabric), // the gingham tea towel on the range's rail
    towel_check = ("#FBF6EC", fabric),
    hob = ("#4A4E54", satin),
    grate = ("#2B2C2E", satin),
    oven_dark = ("#2A2622", satin), // inside the oven
    oven_glass = ("#4A3A30", gloss),
    pedestal = ("#C79A62", satin), // the giving jar's turned pedestal, plain (the jar takes the glass draw)
    glass = ("#EEF4F4", glass),
    cork = ("#B5835A", satin),
    ribbon = ("#E3A0A0", satin),
    rocker = ("#8A5A36", wood), // the rocking chair
    cushion = ("#A8BFA0", fabric), // the rocker's seat cushion, the poufs' tops
    throw = ("#F1E6C8", fabric),
    wicker = ("#C9A16A", satin),
    table = ("#C79A62", satin), // the tea table, plain (the poufs and rug take the fabric draw)
    pouf = ("#A8BFA0", fabric), // the left pouf
    pouf_rose = ("#E3A0A0", fabric), // the right pouf
    rug = ("#E8D6C0", fabric),
    rug_edge = ("#C0714C", fabric),
    feeder = ("#9CC3DB", satin), // the feeder house; the station's main part
    roof = ("#C0714C", satin),
    post = ("#F2EFE8", satin),
    tray = ("#C79A62", satin),
    stone = ("#D8D3C8", matte), // the bird bath
    sack = ("#C9B48C", satin),
    quilt = ("#F6EEDC", fabric), // the settee's quilt
    honey = ("#C79A62", satin), // turned legs, the trolley
    porcelain = ("#FBF6EC", gloss),
    rose = ("#E3A0A0", gloss), // tea set bands, the heart tag
    heart = ("#D9483B", gloss),
    brass = ("#C9A54A", metal),
    chrome = ("#D0D3D6", chrome),
    gold = ("#D9B45A", metal), // coins
    note = ("#7FA35B", matte),
    chalk = ("#3E5A48", matte),
    chalk_line = ("#F4F1EA", matte),
    terracotta = ("#C0714C", satin),
    pot = ("#F4F1EA", satin),
    leaf = ("#5E8F4E", satin),
    leaf_light = ("#7FA35B", satin),
    mustard = ("#D6A64A", satin),
    rose_flower = ("#E3A0A0", satin),
    lilac = ("#B48CD6", satin),
    sky = ("#9CC3DB", satin),
    crust = ("#E0B060", matte),
    crust_dark = ("#C98B3A", matte),
    icing = ("#F4C6D0", matte),
    flour = ("#F4EFE2", matte),
    jar_red = ("#C77D4F", gloss),
    jar_gold = ("#E8C46E", gloss),
    jar_white = ("#F4EFE2", gloss),
    sage_pot = ("#A8BFA0", gloss),
    sage_dark = ("#7E9A76", gloss),
    candle = ("#FBF6EC", satin),
    flame = ("#FFD27A", gloss),
    water = ("#BFE3EE", gloss),
    seed = ("#C9A16A", matte),
    tin = ("#B9BEC3", metal),
    needle = ("#C9CCD0", metal),
    bird_brown = ("#8A6A4E", satin),
    bird_orange = ("#E07A2E", satin),
    bird_blue = ("#4F7FB8", satin),
    bird_cream = ("#F4EFE2", satin),
    bird_gold = ("#C8A040", satin),
    bird_yellow = ("#F2D27A", satin),
    bird_grey = ("#9AA0A6", satin),
    bird_pink = ("#E8B4B8", satin),
    dark = ("#3A3D41", satin),
    lantern = ("#FFD9A0", gloss),
    patch_a = ("#E3A0A0", fabric),
    patch_b = ("#D6A64A", fabric),
    patch_c = ("#9CC3DB", fabric),
    patch_d = ("#A8BFA0", fabric),
    pillow_a = ("#E3A0A0", fabric),
    pillow_b = ("#F1E6C8", fabric),
    soil = ("#5C4433", matte), // the potted plants' soil
}

/// The wall's face in a 1-deep footprint, game depth.
pub use artkit::kit::WALL;

/// A teapot standing on `y`, spout to the right (+x game), `s` its scale: a round body with a band, a lid with a
/// knob, a curving spout and a looped handle.
pub fn teapot(f: &GameFrame, x: f64, y: f64, z: f64, s: f64) {
    kit::lathe!(
        f,
        numbered("pot"),
        x,
        y,
        z,
        &[
            (0.045 * s, 0.0),
            (0.06 * s, 0.008 * s),
            (0.077 * s, 0.035 * s),
            (0.08 * s, 0.06 * s),
            (0.07 * s, 0.092 * s),
            (0.05 * s, 0.108 * s),
            (0.04 * s, 0.11 * s),
        ],
        M.porcelain,
        verts = 18
    );
    kit::turned!(
        f,
        numbered("pot_lid"),
        x,
        y + 0.108 * s,
        z,
        &[
            (0.0, 0.0),
            (0.042 * s, 0.0),
            (0.042 * s, 0.004 * s),
            (0.03 * s, 0.012 * s),
            (0.008 * s, 0.014 * s),
            (0.0, 0.014 * s),
        ],
        M.porcelain,
        segments = 18
    );
    fsphere!(
        f,
        numbered("pot_knob"),
        0.012 * s,
        (x, y + 0.13 * s, z),
        M.rose,
        scale = (1, 1, 0.85),
        segments = 8,
        rings = 5
    );
    torus!(
        numbered("pot_band"),
        0.079 * s,
        0.005 * s,
        at = f.at(x, y + 0.055 * s, z),
        material = M.rose,
        major_segments = 18,
        minor_segments = 3
    );
    fsweep!(
        f,
        numbered("spout"),
        &[
            (x + 0.06 * s, y + 0.035 * s, z),
            (x + 0.1 * s, y + 0.06 * s, z),
            (x + 0.12 * s, y + 0.1 * s, z),
            (x + 0.135 * s, y + 0.115 * s, z)
        ],
        0.016 * s,
        M.porcelain,
        verts = 8,
        resolution = 2,
        scale = [1.0, 0.7, 0.5, 0.45]
    );
    fsweep!(
        f,
        numbered("pot_handle"),
        &[
            (x - 0.065 * s, y + 0.095 * s, z),
            (x - 0.11 * s, y + 0.1 * s, z),
            (x - 0.125 * s, y + 0.065 * s, z),
            (x - 0.1 * s, y + 0.03 * s, z),
            (x - 0.072 * s, y + 0.025 * s, z)
        ],
        0.008 * s,
        M.porcelain,
        verts = 6,
        resolution = 2
    );
}

/// A cup on its saucer, a rose band round its rim, its handle out at `turn` degrees (0: +x).
pub fn cup(f: &GameFrame, x: f64, y: f64, z: f64, turn: f64) {
    kit::turned!(
        f,
        numbered("saucer"),
        x,
        y,
        z,
        &[
            (0.0, 0.0),
            (0.03, 0.0),
            (0.05, 0.006),
            (0.052, 0.009),
            (0.034, 0.005),
            (0.0, 0.005),
        ],
        M.porcelain,
        segments = 16
    );
    kit::turned!(
        f,
        numbered("cup"),
        x,
        y + 0.006,
        z,
        &[
            (0.0, 0.0),
            (0.022, 0.0),
            (0.038, 0.045),
            (0.034, 0.045),
            (0.02, 0.006),
            (0.0, 0.006),
        ],
        M.porcelain,
        segments = 14
    );
    torus!(
        numbered("cup_band"),
        0.0365,
        0.003,
        at = f.at(x, y + 0.049, z),
        material = M.rose,
        major_segments = 14,
        minor_segments = 3
    );
    let (c, s) = (cosd(turn), sind(turn));
    let pts: Vec<(f64, f64, f64)> = [(0.034, 0.036), (0.05, 0.034), (0.05, 0.018), (0.03, 0.014)]
        .iter()
        .map(|&(d, h)| (x + c * d, y + 0.006 + h, z + s * d))
        .collect();
    fsweep!(
        f,
        numbered("cup_handle"),
        &pts,
        0.0035,
        M.porcelain,
        verts = 5,
        resolution = 2
    );
}

/// A little flowering shrub: lumpy satin leaf clumps (no foliage draw) dotted with flowers.
#[allow(clippy::too_many_arguments)]
pub fn bush(f: &GameFrame, x: f64, y: f64, z: f64, r: f64, flower: Mat, seed: i32) {
    let mut rnd = Random::new(seed as i128);
    for (k, (dx, dz, fr)) in [(0.0, 0.0, 1.0), (0.05, 0.03, 0.75), (-0.05, -0.02, 0.8)]
        .into_iter()
        .enumerate()
    {
        let o = boulder!(
            numbered("clump"),
            r * fr * 0.9,
            at = f.at(x + dx, y, z + dz),
            material = if k != 0 { M.leaf } else { M.leaf_light },
            scale = (1, 1, 0.85),
            seed = seed as f64 + k as f64 * 1.7,
            noise = 0.3,
            subdivisions = 2
        );
        jitter!(o, r * 0.08, size = r * 0.5, seed = (seed + k as i32) as f64);
    }
    for k in 0..4 {
        let a = rnd.uniform(0.0, 360.0);
        fsphere!(
            f,
            numbered("flower"),
            0.018,
            (
                x + 0.06 * cosd(a),
                y + r * (1.25 + 0.2 * (k % 2) as f64),
                z + 0.05 * sind(a)
            ),
            flower,
            scale = (1, 1, 0.6),
            segments = 7,
            rings = 4
        );
    }
}
