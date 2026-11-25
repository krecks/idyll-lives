//! The fern glasshouse's plants: ferns, potted palms and orchids.

use super::common::{_flower, M};
use artkit::materials::Mat;
use artkit::place::GameFrame;
use artkit::rand::Random;
use artkit::{fsweep, leaf};

/// A fern: fronds cut into leaflets, arching out from a pot's soil at (x, y, z).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn _fern(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    material: Mat,
    spread: f64,
    n: usize,
    seed: i128,
) {
    let mut rnd = Random::new(seed);
    for k in 0..n {
        // (the arguments in the script's order: the yaw's draw, the tilt's, then the length's)
        let yaw = (k * 360) as f64 / n as f64 + rnd.uniform(-15.0, 15.0);
        let tilt = rnd.uniform(35.0, 60.0);
        let length = spread * rnd.uniform(0.85, 1.1);
        leaf!(
            format!("{name}{k}"),
            f.at(x, y, z),
            yaw,
            tilt,
            length,
            spread * 0.32,
            material,
            shape = "lance",
            droop = 0.45,
            fold = 0.1,
            notch = (2usize, 0.35),
            rows = 4,
            ring = 3
        );
    }
}

/// A potted palm: a ringed trunk and fronds arching out from its top, each cut into leaflets.
pub fn _palm(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, s: i32) {
    fsweep!(
        f,
        format!("{name}_trunk"),
        &[
            (x, y, z),
            (x + s as f64 * 0.02, y + 0.25, z),
            (x, y + 0.5, z)
        ],
        0.03,
        M.cedar_dark,
        verts = 6,
        resolution = 2,
        scale = [1.1, 0.8]
    );
    for k in 0..7 {
        leaf!(
            format!("{name}_frond{k}"),
            f.at(x, y + 0.49, z),
            (k * 360) as f64 / 7.0 + 20.0,
            (35 + 12 * (k % 2)) as f64,
            0.3,
            0.12,
            M.green_light,
            shape = "lance",
            droop = 0.5,
            fold = 0.15,
            notch = (2usize, 0.3),
            rows = 5,
            ring = 3
        );
    }
}

/// An orchid: two broad leaves at the pot's rim, an arching spike of blooms tied to a stick.
pub fn _orchid(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, bloom: Mat) {
    for k in [-1, 1] {
        leaf!(
            format!("{name}_leaf{k}"),
            f.at(x, y, z),
            (90 * k + 10) as f64,
            20.0,
            0.12,
            0.05,
            M.green_dark,
            droop = 0.3,
            rows = 3,
            ring = 3
        );
    }
    fsweep!(
        f,
        format!("{name}_spike"),
        &[
            (x, y, z),
            (x + 0.01, y + 0.18, z + 0.01),
            (x + 0.04, y + 0.28, z + 0.03)
        ],
        0.004,
        M.stem,
        verts = 4,
        resolution = 2
    );
    for k in 0..3 {
        let kf = k as f64;
        _flower(
            f,
            &format!("{name}_bloom{k}"),
            x + 0.012 + 0.014 * kf,
            y + 0.19 + 0.04 * kf,
            z + 0.025 + 0.004 * kf,
            0.03,
            bloom,
            Some(M.yellow),
            5,
            75.0,
            180.0,
        );
    }
}
