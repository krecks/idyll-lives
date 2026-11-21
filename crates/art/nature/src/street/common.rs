//! What the street's props are built from.

use super::M;
use artkit::far::{is_low, low};
use artkit::materials::Mat;
use artkit::math::{P3, TAU, cos, pow, sin};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, jitter, loft, revolve, ring};

/// A box in game axes; bevelled with one segment (half the triangles of two), none in a far version.
#[allow(clippy::too_many_arguments)]
pub fn _box(
    f: &GameFrame,
    name: impl AsRef<str>,
    w: f64,
    h: f64,
    d: f64,
    at: (impl Coord, impl Coord, impl Coord),
    material: Mat,
    bevel: f64,
) -> Obj {
    fbox!(
        f,
        name,
        w,
        h,
        d,
        at,
        material,
        bevel = low(bevel, 0.0),
        bevel_segments = 1
    )
}

// --- the planter and the flower tub: a concrete tub, a little tree or flowers ---------------------------------
/// The tub's section (half-width, height): a chamfered foot, the body tapering out to a groove, a rim proud of it
/// with chamfered edges, and inside down to under the soil.
// (0.318 is a half-width of the tub, not 1/pi.)
#[allow(clippy::approx_constant)]
const TUB: [(f64, f64); 14] = [
    (0.300, 0.0),
    (0.311, 0.011),
    (0.311, 0.03),
    (0.304, 0.036),
    (0.322, 0.30),
    (0.318, 0.306),
    (0.324, 0.312),
    (0.33, 0.36),
    (0.347, 0.367),
    (0.35, 0.41),
    (0.343, 0.42),
    (0.302, 0.42),
    (0.294, 0.41),
    (0.289, 0.37),
];
const TUB_FAR: [(f64, f64); 7] = [
    (0.300, 0.0),
    (0.311, 0.011),
    (0.33, 0.36),
    (0.35, 0.37),
    (0.35, 0.42),
    (0.298, 0.42),
    (0.289, 0.37),
];

/// The concrete tub (0.7 m square, 0.42 high) in one piece, hollow at the top, and its soil 2 cm below the rim;
/// `groove`: a groove round it under the rim.
pub fn _tub(groove: bool) {
    let n = low(if groove { 24 } else { 16 }, 12);
    let rows: Vec<(f64, f64)> = if is_low() {
        TUB_FAR.to_vec()
    } else if groove {
        TUB.to_vec()
    } else {
        [&TUB[..4], &TUB[7..]].concat()
    };
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(r, y)| ring!(r, r, z = y, verts = n, square = 9))
        .collect();
    loft!("tub", &sections, material = M.concrete);
    let soil = loft!(
        "soil",
        &[
            ring!(0.291, 0.291, z = 0.37, verts = n, square = 9),
            ring!(0.292, 0.292, z = 0.395, verts = n, square = 9)
        ],
        material = M.soil
    );
    if !is_low() {
        jitter!(soil, 0.004, size = 0.08, seed = 3.0);
    }
}

/// A flower open to the sky: `petals` rounded petals round a raised eye, at game (x, y, z), `r` across its
/// petals.
#[allow(clippy::too_many_arguments)]
pub fn _bloom(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    petals: usize,
    mat: Mat,
    eye: Mat,
    tilt: (f64, f64, f64),
    detail: usize,
) {
    let mut pts = Vec::new();
    let n = petals * detail;
    for i in 0..n {
        let a = TAU * i as f64 / n as f64;
        let k = pow((cos(petals as f64 * a / 2.0)).abs(), 0.7);
        let rr = r * (0.42 + 0.58 * k);
        pts.push((rr * cos(a), rr * sin(a)));
    }
    fextrude!(
        f,
        name,
        &pts,
        0.01,
        (x, y, z),
        mat,
        plane = Facing::Top,
        rot = tilt
    );
    fcyl!(
        f,
        format!("{name}_eye"),
        r * 0.3,
        0.014,
        (x, y + 0.002, z),
        eye,
        verts = 5,
        rot = tilt
    );
}

// --- the street-name sign: two green blades crossing on a pole -----------------------------------------------
/// Words of dark green lettering on a blade's white panel, on one face (+1 front, -1 back): each word a capital,
/// then small letters on the same baseline, a few with ascenders.
pub fn _lettering(
    f: &GameFrame,
    name: &str,
    length: f64,
    y: f64,
    along_x: bool,
    face: i32,
    rng: &mut Random,
) {
    let face = face as f64;
    let mut x = -length / 2.0;
    let mut k = 0;
    let mut start = true;
    while x < length / 2.0 - 0.04 {
        let w = rng.choice(&[0.03, 0.035, 0.04]);
        let h = rng.choice(&[0.06, 0.06, 0.07]); // (drawn as before, so the rest of the sign's numbers stay put)
        if rng.random() < 0.14 && k != 0 {
            x += 0.04; // a space between words
            start = true;
        }
        let h = if start {
            0.075
        } else if h > 0.065 {
            0.068
        } else {
            0.05
        };
        start = false;
        let c = x + w / 2.0;
        if along_x {
            _box(
                f,
                format!("{name}{k}"),
                w,
                h,
                0.004,
                (c, y - 0.0375 + h / 2.0, face * 0.019),
                M.sign,
                0.0,
            );
        } else {
            _box(
                f,
                format!("{name}{k}"),
                0.004,
                h,
                w,
                (face * 0.019, y - 0.0375 + h / 2.0, c),
                M.sign,
                0.0,
            );
        }
        x += w + 0.012;
        k += 1;
    }
}

/// A round steel post from a flanged foot bolted to the ground up to `top`: the foot's profile turned, a collar
/// where the post leaves it.
pub fn _post(f: &GameFrame, top: f64, foot: f64, r: f64) {
    _post_turned(f, top, foot, r, 6);
}

/// `_post`, its far version turned in `far` sides (8 keeps the foot as deep as the full one's from the front).
pub fn _post_turned(f: &GameFrame, top: f64, foot: f64, r: f64, far: usize) {
    let n = low(12, far);
    let c = (r + 0.01).min(foot);
    let prof = [
        (0.0, 0.0),
        (foot, 0.0),
        (foot, 0.01),
        (c.max(foot - 0.012), 0.016),
        (c, 0.03),
        (c.min(r + 0.006), 0.09),
        (c, 0.1),
        (c, 0.115),
        (r, 0.125),
        (r, top),
        (0.0, top),
    ];
    let far = [
        (0.0, 0.0),
        (foot, 0.0),
        (c, 0.03),
        (r, 0.12),
        (r, top),
        (0.0, top),
    ];
    revolve!(
        "post",
        if !is_low() { &prof[..] } else { &far[..] },
        material = M.pole,
        segments = n
    );
    if !is_low() && foot > r + 0.03 {
        // the foot's four bolts
        for k in 0..4 {
            let a = TAU * (k as f64 + 0.5) / 4.0;
            fcyl!(
                f,
                format!("bolt{k}"),
                0.008,
                0.012,
                ((foot - 0.018) * cos(a), 0.014, (foot - 0.018) * sin(a)),
                M.pole,
                verts = 6
            );
        }
    }
}

/// A clamp band round the post at height y.
pub fn _band(f: &GameFrame, name: impl AsRef<str>, y: f64, r: f64, h: f64) {
    fcyl!(
        f,
        name,
        r,
        h,
        (0, y, 0),
        M.pole,
        verts = low(12, 6),
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use artkit::far::far_of;
    use artkit::scene::{self, meshes};

    #[test]
    fn a_far_tub_has_fewer_sides_and_smooth_soil() {
        scene::reset();
        _tub(true);
        let near: usize = meshes()
            .iter()
            .map(|o| o.read(|p| p.mesh.verts.len()))
            .sum();
        scene::reset();
        far_of(|| _tub(true));
        let far: usize = meshes()
            .iter()
            .map(|o| o.read(|p| p.mesh.verts.len()))
            .sum();
        assert_eq!((near, far), (24 * 14 + 24 * 2, 12 * 7 + 12 * 2));
    }
}
