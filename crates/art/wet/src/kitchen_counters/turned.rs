//! What is turned: a round thing lofted about the upright, and the plates, bowls, mugs, jugs, canisters and tins
//! turned with it.

use artkit::materials::Mat;
use artkit::math::{P3, cos, radians, sin};
use artkit::place::{Coord, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::{fsweep, loft, ring};

/// A round thing turned about the upright through drawn (x, y, z), its foot there: `profile` [(radius, height), ...]
/// from the bottom up, inside and out (a radius of 0 closes it).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn lathe(
    p: &GameFrame,
    name: impl AsRef<str>,
    profile: &[(f64, f64)],
    x: impl Coord,
    y: f64,
    z: impl Coord,
    material: Mat,
    verts: usize,
) -> Obj {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = p.at(x, y, z);
    let sections: Vec<Vec<P3>> = profile
        .iter()
        .map(|&(r, h)| {
            ring!(
                py::max(r, 1e-5),
                py::max(r, 1e-5),
                z = bz + h,
                verts = verts,
                at = (bx, by)
            )
        })
        .collect();
    loft!(name, &sections, material = material)
}

/// A stack of `n` plates standing on `y`: each a foot, a rim and a lip, the top one's well.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn plates(
    p: &GameFrame,
    name: impl AsRef<str>,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    n: i32,
    material: Mat,
    h: f64,
    verts: usize,
) -> Obj {
    let mut prof = vec![(0.0, 0.0), (0.55 * r, 0.0)];
    for i in 0..n {
        let b = i as f64 * h;
        prof.extend([(0.62 * r, b + 0.003), (r, b + 0.75 * h), (0.97 * r, b + h)]);
    }
    prof.extend([
        (0.62 * r, n as f64 * h - 0.004),
        (0.0, n as f64 * h - 0.004),
    ]);
    lathe(p, name, &prof, x, y, z, material, verts)
}

/// A bowl standing on `y`, hollow: its foot, a curved side, a rolled lip.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn bowl(
    p: &GameFrame,
    name: impl AsRef<str>,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    verts: usize,
) -> Obj {
    let prof = [
        (0.0, 0.0),
        (0.42 * r, 0.0),
        (0.48 * r, 0.08 * h),
        (0.8 * r, 0.35 * h),
        (0.97 * r, 0.75 * h),
        (r, h),
        (0.92 * r, h),
        (0.86 * r, 0.72 * h),
        (0.6 * r, 0.3 * h),
        (0.0, 0.2 * h),
    ];
    lathe(p, name, &prof, x, y, z, material, verts)
}

/// A mug standing on `y`: hollow, its handle out at `turn` degrees (0: +x, 90: to the front).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn mug(
    p: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    turn: f64,
    verts: usize,
) {
    lathe(
        p,
        name,
        &[
            (0.0, 0.0),
            (0.88 * r, 0.0),
            (r, 0.06 * h),
            (r, h),
            (0.86 * r, h),
            (0.86 * r, 0.15 * h),
            (0.0, 0.15 * h),
        ],
        x,
        y,
        z,
        material,
        verts,
    );
    let (c, s) = (cos(radians(turn)), sin(radians(turn)));
    let pts: Vec<(f64, f64, f64)> = [
        (r - 0.003, 0.8),
        (r + 0.016, 0.82),
        (r + 0.024, 0.55),
        (r + 0.015, 0.27),
        (r - 0.003, 0.25),
    ]
    .into_iter()
    .map(|(d, t)| (x + c * d, y + t * h, z + s * d))
    .collect();
    fsweep!(
        p,
        format!("{name}_handle"),
        &pts,
        0.0045,
        material,
        verts = 6,
        resolution = 2
    );
}

/// A pitcher standing on `y`: a full belly narrowing to a flared neck, hollow at the top, a handle at `turn`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn jug(
    p: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    turn: f64,
    verts: usize,
) {
    lathe(
        p,
        name,
        &[
            (0.0, 0.0),
            (0.8 * r, 0.0),
            (0.95 * r, 0.1 * h),
            (r, 0.38 * h),
            (0.78 * r, 0.75 * h),
            (0.74 * r, 0.88 * h),
            (0.84 * r, h),
            (0.76 * r, h),
            (0.66 * r, 0.86 * h),
            (0.0, 0.84 * h),
        ],
        x,
        y,
        z,
        material,
        verts,
    );
    let (c, s) = (cos(radians(turn)), sin(radians(turn)));
    let pts: Vec<(f64, f64, f64)> = [
        (0.72 * r, 0.9),
        (0.9 * r + 0.03, 0.88),
        (r + 0.03, 0.6),
        (r + 0.01, 0.32),
        (0.92 * r, 0.25),
    ]
    .into_iter()
    .map(|(d, t)| (x + c * d, y + t * h, z + s * d))
    .collect();
    fsweep!(
        p,
        format!("{name}_handle"),
        &pts,
        0.006,
        material,
        verts = 6,
        resolution = 2
    );
}

/// A lidded canister standing on `y`: a straight body, a lid with a knob.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn canister(
    p: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    lid: Mat,
    verts: usize,
) {
    lathe(
        p,
        name,
        &[
            (0.0, 0.0),
            (0.92 * r, 0.0),
            (r, 0.04 * h),
            (r, 0.86 * h),
            (0.0, 0.86 * h),
        ],
        x,
        y,
        z,
        material,
        verts,
    );
    lathe(
        p,
        format!("{name}_lid"),
        &[
            (0.0, 0.84 * h),
            (1.04 * r, 0.84 * h),
            (1.04 * r, 0.9 * h),
            (0.3 * r, 0.95 * h),
            (0.3 * r, h),
            (0.0, h),
        ],
        x,
        y,
        z,
        lid,
        verts,
    );
}

/// A tin can standing on `y`: a body with a ridge round its middle and a rolled rim round its lid.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn can(
    p: &GameFrame,
    name: impl AsRef<str>,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    verts: usize,
) -> Obj {
    lathe(
        p,
        name,
        &[
            (0.0, 0.0),
            (0.96 * r, 0.0),
            (r, 0.03 * h),
            (r, 0.48 * h),
            (0.97 * r, 0.5 * h),
            (r, 0.52 * h),
            (r, 0.95 * h),
            (1.03 * r, 0.97 * h),
            (1.03 * r, h),
            (0.0, h),
        ],
        x,
        y,
        z,
        material,
        verts,
    )
}
