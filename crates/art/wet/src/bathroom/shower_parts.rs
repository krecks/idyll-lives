//! What the showers are built from: tiled walls, the rain, frames, trays, grates, heads, the hand shower, mixers,
//! caddies and curtains.

use super::M;
use super::common::{LATHE, Lathe, bends, lathe, wall_rose};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, TAU, V3, cos, cosd, sin, sind, v3};
use artkit::parts::{join, new_parts};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, py, torus};

/// The rain from a head at game `head` (the head's underside) down to the floor at height `bottom`: thin parallel
/// streams across a disc of radius `r` (`rings` rings round a middle one) or a `grid` (n, spacing), joined with
/// the origin on the head and scaled to nothing along their length. `towards` (a game point on the floor) slants
/// the rain from a wall head onto the resident.
pub fn rain(
    f: &GameFrame,
    head: (f64, f64, f64),
    r: f64,
    bottom: f64,
    rings: usize,
    grid: Option<(usize, f64)>,
    towards: Option<(f64, f64, f64)>,
) -> Obj {
    let (x, y, z) = head;
    let spots: Vec<(f64, f64)> = if let Some((n, step)) = grid {
        let mut spots = Vec::new();
        for i in 0..n {
            for k in 0..n {
                spots.push((
                    (i as f64 - (n - 1) as f64 / 2.0) * step,
                    (k as f64 - (n - 1) as f64 / 2.0) * step,
                ));
            }
        }
        spots
    } else {
        let mut spots = vec![(0.0, 0.0)];
        for ring in 1..=rings {
            for i in 0..6 * ring {
                let a = (360 * i) as f64 / (6 * ring) as f64;
                spots.push((
                    r * ring as f64 / rings as f64 * cosd(a),
                    r * ring as f64 / rings as f64 * sind(a),
                ));
            }
        }
        spots
    };
    let mut aim = V3::ZERO;
    let length = if let Some(towards) = towards {
        aim = V3::from(f.at(towards.0, towards.1, towards.2)) - V3::from(f.at(x, y, z));
        aim.length()
    } else {
        y - bottom
    };
    let parts = new_parts(|| {
        for (i, &(dx, dz)) in spots.iter().enumerate() {
            fcyl!(
                f,
                format!("rain{i}"),
                0.005,
                length,
                (x + dx, y - length / 2.0, z + dz),
                M.water,
                verts = 6,
                smooth = false
            );
        }
    });
    let rain = join(&parts, "water", Some(f.at(x, y, z)));
    if towards.is_some() {
        // turn the straight-down rain about its origin, onto the resident
        rain.edit(|o| o.quat = Some(v3(0, 0, -1).rotation_difference(aim)));
    }
    rain.edit(|o| o.scale.z = 0.001);
    rain
}

/// A square-section frame (a screen's posts and rails) in one piece through game points, its corners bent.
pub fn frame(
    f: &GameFrame,
    name: &str,
    pts: &[(f64, f64, f64)],
    material: Mat,
    t: f64,
    bend: f64,
    up: P3,
) -> Obj {
    fsweep!(
        f,
        name,
        &bends(pts, bend, 3),
        0.0,
        material,
        profile = rounded_rect(t, t, 0.004, 1),
        up = up,
        angle = 50
    )
}

#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn post(f: &GameFrame, name: &str, x: f64, z: f64, y0: f64, y1: f64, material: Mat, t: f64) {
    fbox!(
        f,
        name,
        t,
        y1 - y0,
        t,
        (x, (y0 + y1) / 2.0, z),
        material,
        bevel = 0.004
    );
}

/// A shower tray in one piece: a rounded slab `w` x `d`, `h` high, its top a raised lip `lip` wide round a floor
/// that falls `fall` towards the middle.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn tray(
    f: &GameFrame,
    name: &str,
    w: f64,
    d: f64,
    h: f64,
    material: Mat,
    lip: f64,
    fall: f64,
    square: i32,
) {
    let square = square as f64;
    lathe(
        f,
        name,
        0.0,
        0.0,
        &[
            (0.0, w / 2.0 - 0.004, d / 2.0 - 0.004, square),
            (h - 0.008, w / 2.0, d / 2.0, square),
            (h, w / 2.0 - 0.006, d / 2.0 - 0.006, square),
            (h, w / 2.0 - lip, d / 2.0 - lip, square * 0.8),
            (
                h - fall * 0.6,
                w / 2.0 - lip - 0.012,
                d / 2.0 - lip - 0.012,
                square * 0.7,
            ),
            (h - fall, w / 4.0, d / 4.0, 3.0),
        ],
        material,
        Lathe {
            verts: 40,
            angle: 35.0,
            ..LATHE
        },
    );
}

/// A drain's grate on a dark well at height `y`: a plate with slots cut across it, or a round one.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn grate(
    f: &GameFrame,
    name: &str,
    x: impl Coord,
    y: f64,
    z: f64,
    w: f64,
    d: f64,
    material: Mat,
    slots: usize,
    round_: bool,
) {
    fbox!(
        f,
        format!("{name}_well"),
        w - 0.004,
        0.004,
        d - 0.004,
        (x, y - 0.001, z),
        M.dark
    );
    let middle = (slots - 1) as f64 / 2.0;
    if round_ {
        let holes: Vec<Vec<(f64, f64)>> = (0..slots)
            .map(|k| {
                rounded_rect(0.006, d * 0.6, 0.003, 1)
                    .into_iter()
                    .map(|(u, v)| (u + (k as f64 - middle) * w / (slots + 1) as f64, v))
                    .collect()
            })
            .collect();
        let outline: Vec<(f64, f64)> = (0..20)
            .map(|i| {
                (
                    w / 2.0 * cos(TAU * i as f64 / 20.0),
                    w / 2.0 * sin(TAU * i as f64 / 20.0),
                )
            })
            .collect();
        fextrude!(
            f,
            name,
            &outline,
            0.004,
            (x, y + 0.002, z),
            material,
            plane = Facing::Top,
            holes = holes
        );
    } else {
        let holes: Vec<Vec<(f64, f64)>> = (0..slots)
            .map(|k| {
                rounded_rect(w / (slots as f64 * 2.2), d - 0.024, 0.003, 1)
                    .into_iter()
                    .map(|(u, v)| (u + (k as f64 - middle) * w / (slots as f64 + 0.6), v))
                    .collect()
            })
            .collect();
        fextrude!(
            f,
            name,
            &rounded_rect(w, d, 0.004, 1),
            0.004,
            (x, y + 0.002, z),
            material,
            plane = Facing::Top,
            holes = holes
        );
    }
}

/// A round shower head, its face at height `y` (where the rain starts): a disc rounding over into a shallow dome,
/// its nozzle plate dark.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn head(
    f: &GameFrame,
    name: &str,
    x: impl Coord,
    y: f64,
    z: f64,
    r: f64,
    material: Mat,
    t: f64,
) {
    lathe(
        f,
        name,
        x.f(),
        z,
        &[
            (y + 0.001, r * 0.94, r * 0.94),
            (y + t * 0.2, r, r),
            (y + t * 0.75, r, r),
            (y + t, r * 0.82, r * 0.82),
            (y + t * 1.25, r * 0.25, r * 0.25),
        ],
        material,
        Lathe {
            verts: 28,
            angle: 50.0,
            ..LATHE
        },
    );
    fcyl!(
        f,
        format!("{name}_nozzles"),
        r * 0.78,
        0.002,
        (x, y, z),
        M.dark,
        verts = 20
    );
}

/// A hand shower on a slide rail on the wall (from `y0` to `y1` at depth `z`), its hose looping down from
/// `hose_from` (a game point) up to its handle.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn hand_shower(
    f: &GameFrame,
    x: f64,
    y0: f64,
    y1: f64,
    z: f64,
    material: Mat,
    hose_from: (f64, f64, f64),
) {
    kit::tube!(
        f,
        "riser",
        &[(x, y0, z), (x, y1, z)],
        0.011,
        material,
        verts = 10,
        round = 1
    );
    for y in [y0, y1] {
        wall_rose(
            f,
            format!("riser_bracket{}", float_repr(y)),
            x,
            y,
            z - 0.03,
            0.02,
            material,
            0.03,
            12,
        );
    }
    fbox!(
        f,
        "slider",
        0.03,
        0.04,
        0.03,
        (x, y0 + (y1 - y0) * 0.6, z + 0.012),
        material,
        bevel = 0.008,
        bevel_segments = 1
    );
    let hy = y0 + (y1 - y0) * 0.6;
    kit::tube!(
        f,
        "hand_shower",
        &[
            (x, hy - 0.06, z + 0.03),
            (x, hy + 0.08, z + 0.06),
            (x, hy + 0.12, z + 0.075),
        ],
        0.013,
        material,
        verts = 10,
        round = 2,
        scale = [0.8, 1.0, 1.1]
    );
    let (hx, hy0, hz) = hose_from;
    fcyl!(
        f,
        "hand_head",
        0.04,
        0.02,
        (x, hy + 0.135, z + 0.08),
        material,
        rot = (-30, 0, 0),
        verts = 16,
        bevel = 0.006,
        bevel_segments = 1
    );
    kit::tube!(
        f,
        "hose",
        &[
            (hx, hy0, hz),
            (hx - 0.01, hy0 - 0.2, hz + 0.02),
            ((hx + x) / 2.0, hy0 - 0.38, hz + 0.05),
            (x + 0.01, hy0 - 0.25, z + 0.045),
            (x, hy - 0.06, z + 0.03),
        ],
        0.006,
        material,
        verts = 6
    );
}

/// A thermostatic bar mixer on the wall at depth `z`: a bar on two wall plates, a knob at each end, an outlet
/// under it.
pub fn bar_mixer(f: &GameFrame, x: f64, y: f64, z: f64, w: f64, material: Mat) {
    for s in [-1, 1] {
        wall_rose(
            f,
            format!("mixer_plate{s}"),
            x + s as f64 * (w / 2.0 - 0.03),
            y,
            z,
            0.026,
            material,
            0.03,
            14,
        );
    }
    kit::tube!(
        f,
        "mixer",
        &[(x - w / 2.0, y, z + 0.045), (x + w / 2.0, y, z + 0.045)],
        0.022,
        material,
        verts = 14,
        round = 1
    );
    for s in [-1, 1] {
        fcyl!(
            f,
            format!("mixer_knob{s}"),
            0.026,
            0.03,
            (x + s as f64 * (w / 2.0 + 0.015), y, z + 0.045),
            material,
            rot = (0, 90, 0),
            verts = 16,
            bevel = 0.006,
            bevel_segments = 1
        );
    }
    fcyl!(
        f,
        "mixer_outlet",
        0.01,
        0.03,
        (x + w / 4.0, y - 0.035, z + 0.045),
        material,
        verts = 10
    );
}

/// A wire shelf on the wall: a tray, a rail round its front on two posts.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn caddy(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, w: f64, d: f64, material: Mat) {
    fbox!(
        f,
        format!("{name}_tray"),
        w,
        0.008,
        d,
        (x, y, z),
        material,
        bevel = 0.003,
        bevel_segments = 1
    );
    kit::tube!(
        f,
        format!("{name}_rail"),
        &[
            (x - w / 2.0, y + 0.04, z - d / 2.0 + 0.005),
            (x - w / 2.0, y + 0.04, z + d / 2.0),
            (x + w / 2.0, y + 0.04, z + d / 2.0),
            (x + w / 2.0, y + 0.04, z - d / 2.0 + 0.005),
        ],
        0.004,
        material,
        verts = 6,
        round = 2
    );
    for s in [-1, 1] {
        kit::tube!(
            f,
            format!("{name}_post{s}"),
            &[
                (x + s as f64 * w / 2.0, y, z + d / 2.0),
                (x + s as f64 * w / 2.0, y + 0.04, z + d / 2.0),
            ],
            0.003,
            material,
            verts = 6,
            round = 1
        );
    }
}

/// A curtain hanging in `waves` soft folds from game point `a` (x, z) to `b`, from `y0` up to `y1`: one wavy sheet,
/// thick `t`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn curtain(
    f: &GameFrame,
    name: &str,
    a: (f64, f64),
    b: (f64, f64),
    y0: f64,
    y1: f64,
    waves: usize,
    depth: f64,
    t: f64,
    steps: usize,
) -> Obj {
    let (ax, az) = a;
    let (bx, bz) = b;
    let l = py::hypot(bx - ax, bz - az);
    let (ux, uz) = ((bx - ax) / l, (bz - az) / l); // along the rail
    let (nx, nz) = (-uz, ux); // across it
    let n = waves * steps;
    let mid: Vec<(f64, f64)> = (0..=n)
        .map(|k| {
            let (k, n, waves) = (k as f64, n as f64, waves as f64);
            (
                ax + ux * l * k / n + nx * depth / 2.0 * sin(TAU * waves * k / n),
                az + uz * l * k / n + nz * depth / 2.0 * sin(TAU * waves * k / n),
            )
        })
        .collect();
    let (cx, cz) = ((ax + bx) / 2.0, (az + bz) / 2.0);
    let front = mid
        .iter()
        .map(|&(x, z)| (x + nx * t / 2.0 - cx, z + nz * t / 2.0 - cz));
    let back = mid
        .iter()
        .rev()
        .map(|&(x, z)| (x - nx * t / 2.0 - cx, z - nz * t / 2.0 - cz));
    let outline: Vec<(f64, f64)> = front.chain(back).collect();
    fextrude!(
        f,
        name,
        &outline,
        y1 - y0,
        (cx, (y0 + y1) / 2.0, cz),
        M.curtain,
        plane = Facing::Top,
        smooth = true,
        angle = 60
    )
}

/// `n` rings along a rail from game point `a` (x, z) to `b` at height `y`, each round the rail.
pub fn curtain_rings(
    f: &GameFrame,
    name: &str,
    a: (f64, f64),
    b: (f64, f64),
    y: f64,
    n: usize,
    material: Mat,
) {
    let (ax, az) = a;
    let (bx, bz) = b;
    let along_x = (bx - ax).abs() > (bz - az).abs();
    for k in 0..n {
        let fr = (k as f64 + 0.5) / n as f64;
        torus!(
            format!("{name}{k}"),
            0.016,
            0.0022,
            at = f.at(ax + (bx - ax) * fr, y - 0.004, az + (bz - az) * fr),
            material = material,
            rot = if along_x { (0, 90, 0) } else { (90, 0, 0) },
            major_segments = 8,
            minor_segments = 3
        );
    }
}
