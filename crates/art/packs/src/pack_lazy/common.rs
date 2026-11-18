//! What the lazy pack is built from: its palette, turned and lofted pieces, a mug, a band of a sphere (the pod's
//! hood).

use artkit::materials::Mat;
use artkit::math::{P3, V3, cos, cosd, radians, sin, sind};
use artkit::mesh::Mesh;
use artkit::place::{Coord, GameFrame};
use artkit::scene::{self, Obj};
use artkit::{fsweep, loft, palette, revolve, ring};
use std::cell::Cell;

palette! { M:
    body = ("#E9E4F4", satin),        // the bed's puffy base and cloud headboard; the main part
    upholstery = ("#F4F2FA", fabric), // the bed's mattress
    pillow = ("#FFFFFF", fabric),
    throw = ("#CFC6EA", fabric),
    blush = ("#F2C6CF", fabric),
    lilac = ("#CFC6EA", fabric),
    cloud = ("#F7F7FA", satin),
    birch = ("#E3CFA8", satin),
    shell = ("#F4F4F2", gloss),       // the nap pod's egg
    seat = ("#7E95B8", fabric),       // the nap pod's cushions and hood
    hood = ("#8FA6C6", fabric),
    shag = ("#E6DCCB", fabric),
    chrome = ("#D0D3D6", chrome),
    oak = ("#CDB08A", satin),         // the quilt table's top, plain (the quilt takes the fabric draw)
    quilt = ("#C96A5B", fabric),      // the kotatsu quilt
    quilt_band = ("#E8B07A", fabric),
    pouf = ("#E8D7B8", fabric),
    pouf_top = ("#F3E7CF", fabric),
    sage = ("#B9C7A8", fabric),
    heater = ("#5A2A24", satin),      // under the quilt
    porcelain = ("#F2F2F0", gloss),
    mandarin = ("#F28C28", gloss),
    biscuit = ("#E8C27A", satin),
    locker = ("#EEF2F0", satin),      // the delivery locker's cabinet
    door = ("#CFE7DC", satin),        // its doors
    accent = ("#F28C28", satin),      // its roof band; the plate's grips
    steel = ("#8A9096", metal),
    bezel = ("#2B3440", satin),
    screen = ("#2E7DB8", gloss),
    screen_light = ("#3C8DBF", gloss),
    white = ("#F2F2F0", gloss),
    led_green = ("#7FB800", gloss),
    cubby = ("#3A3F44", satin),
    kraft = ("#C8A27A", matte),
    tape = ("#E9DFC8", matte),
    bag = ("#F2EEE6", matte),
    bag_logo = ("#D7263D", matte),
    plate = ("#F2F2F0", gloss),       // the vibration plate's base and column
    lime = ("#9BD45A", satin),
    pad = ("#3D4A57", matte),
    pad_inner = ("#4C5B6A", matte),
    readout = ("#6CC3F0", gloss),
    bottle = ("#7FC4E8", gloss),
    wood = ("#B07A4E", wood),         // the smart hub's side table
    speaker = ("#BFC3C7", satin),
    ring = ("#4FD1C5", gloss),
    app_yellow = ("#F2C14E", gloss),
    leaf = ("#6E9E58", satin),        // the mandarins' leaves
}

/// The wall's face in a 1-deep footprint, game depth.
pub const WALL: f64 = -0.43;

thread_local! {
    static N: Cell<u32> = const { Cell::new(0) };
}

/// `stem` numbered: each part its own name.
pub fn _name(stem: &str) -> String {
    N.with(|n| {
        n.set(n.get() + 1);
        format!("{stem}{}", n.get())
    })
}

/// A turned piece standing at game (x, y, z): `profile` [(radius, height up), ...] from its foot, revolved.
#[allow(clippy::too_many_arguments)]
pub fn _turned(
    f: &GameFrame,
    name: &str,
    x: impl Coord,
    y: f64,
    z: impl Coord,
    profile: &[(f64, f64)],
    material: Mat,
    segments: usize,
    rot: (f64, f64, f64),
) -> Obj {
    revolve!(
        name,
        profile,
        at = f.at(x, y, z),
        material = material,
        segments = segments,
        rot = rot
    )
}

/// A body lofted through rings [(radius, height up), ...] standing at game (x, y, z); `sy` squashes them in depth.
#[allow(clippy::too_many_arguments)]
pub fn _lathe(
    f: &GameFrame,
    name: &str,
    x: impl Coord,
    y: f64,
    z: impl Coord,
    rings: &[(f64, f64)],
    material: Mat,
    verts: usize,
    square: f64,
    sy: f64,
) -> Obj {
    let P3 { x: bx, y: by, .. } = f.at(x, 0, z);
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(r, h)| {
            ring!(
                r,
                r * sy,
                z = y + h,
                verts = verts,
                square = square,
                at = (bx, by)
            )
        })
        .collect();
    loft!(name, &sections, material = material)
}

/// A mug standing on `y`: hollow, its handle out at `turn` degrees (0: +x, 90: to the front).
#[allow(clippy::too_many_arguments)]
pub fn _mug(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    turn: f64,
) {
    _turned(
        f,
        name,
        x,
        y,
        z,
        &[
            (0.0, 0.0),
            (0.88 * r, 0.0),
            (r, 0.06 * h),
            (r, h),
            (0.86 * r, h),
            (0.86 * r, 0.15 * h),
            (0.0, 0.15 * h),
        ],
        material,
        14,
        (0.0, 0.0, 0.0),
    );
    let (c, s) = (cosd(turn), sind(turn));
    let pts: Vec<(f64, f64, f64)> = [
        (r - 0.003, 0.8),
        (r + 0.016, 0.82),
        (r + 0.024, 0.55),
        (r + 0.015, 0.27),
        (r - 0.003, 0.25),
    ]
    .iter()
    .map(|&(d, t)| (x + c * d, y + t * h, z + s * d))
    .collect();
    fsweep!(
        f,
        format!("{name}_handle"),
        &pts,
        0.0045,
        material,
        verts = 5,
        resolution = 2
    );
}

/// A closed shell of a sphere about `centre` (game): points at side angle `sides` (from the middle towards ±x) and
/// pitch `pitch` (from the top, negative towards the back), between radii `r_in` and `r_out`. A pram-hood shape;
/// turning it about the centre's x axis keeps it on its sphere.
#[allow(clippy::too_many_arguments)]
pub fn _sphere_band(
    name: &str,
    f: &GameFrame,
    centre: (f64, f64, f64),
    r_in: f64,
    r_out: f64,
    sides: (f64, f64),
    pitch: (f64, f64),
    material: Mat,
    n_side: usize,
    n_pitch: usize,
) -> Obj {
    let (a0, a1) = (radians(sides.0), radians(sides.1));
    let (p0, p1) = (radians(pitch.0), radians(pitch.1));
    let (cx, cy, cz) = centre;
    let mut verts: Vec<V3> = Vec::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();
    let mut grid = |r: f64| -> usize {
        let base = verts.len();
        for i in 0..=n_side {
            let a = a0 + (a1 - a0) * i as f64 / n_side as f64;
            for j in 0..=n_pitch {
                let p = p0 + (p1 - p0) * j as f64 / n_pitch as f64;
                verts.push(V3::from(f.at(
                    cx + r * sin(a),
                    cy + r * cos(a) * cos(p),
                    cz + r * cos(a) * sin(p),
                )));
            }
        }
        base
    };
    let idx = |base: usize, i: usize, j: usize| (base + i * (n_pitch + 1) + j) as u32;
    let (o, n) = (grid(r_out), grid(r_in));
    for i in 0..n_side {
        for j in 0..n_pitch {
            faces.push(vec![
                idx(o, i, j),
                idx(o, i + 1, j),
                idx(o, i + 1, j + 1),
                idx(o, i, j + 1),
            ]);
            faces.push(vec![
                idx(n, i, j + 1),
                idx(n, i + 1, j + 1),
                idx(n, i + 1, j),
                idx(n, i, j),
            ]);
        }
    }
    for i in 0..n_side {
        // the two pitch edges
        for j in [0, n_pitch] {
            faces.push(vec![
                idx(o, i, j),
                idx(n, i, j),
                idx(n, i + 1, j),
                idx(o, i + 1, j),
            ]);
        }
    }
    for j in 0..n_pitch {
        // the two side edges
        for i in [0, n_side] {
            faces.push(vec![
                idx(o, i, j),
                idx(o, i, j + 1),
                idx(n, i, j + 1),
                idx(n, i, j),
            ]);
        }
    }
    let mut mesh = Mesh::from_pydata(verts, &faces);
    mesh.set_material(Some(material));
    mesh.recalc_normals();
    // (every face smooth, no edge kept sharp)
    mesh.shade_smooth(180.0);
    scene::add(name, mesh)
}
