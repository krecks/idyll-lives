//! The light-up dance floor and the party photo wall.

use super::common::{_speaker, M, PHOTOS, TILES, WALL};
use artkit::forms::rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, cos, radians, sin, sqrt};
use artkit::motion::turns;
use artkit::parts::{hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::{fbox, fcone, fcyl, fsphere, fsweep, grow, loft, revolve, ring, settle, soft};
use artkit::{forms::eased, shapes::AUTO_WIDTH};

// --- the light-up dance floor (2x2): sixteen glowing tiles, a mirror ball on a truss gantry, two speaker towers ---
const TILE: f64 = 0.43;
const PITCH: f64 = 0.45;
const TILE_Y: f64 = 0.06;
const LIFT: f64 = 0.025;

/// A triangular box truss from game point `a` to `b`, `w` across: three chords and zigzag lacing on each face.
fn _truss(
    f: &GameFrame,
    name: &str,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    w: f64,
    material: Mat,
    bays: usize,
) {
    let (a, b) = ([a.0, a.1, a.2], [b.0, b.1, b.2]);
    let d: [f64; 3] = std::array::from_fn(|i| b[i] - a[i]);
    let length = sqrt(py::sum(d.iter().map(|c| c * c)));
    let u: [f64; 3] = std::array::from_fn(|i| d[i] / length);
    // two directions square to the run: one level (across), one the cross of the two
    let side = if u[1].abs() < 0.9 {
        [-u[2], 0.0, u[0]]
    } else {
        [1.0, 0.0, 0.0]
    };
    let n = sqrt(py::sum(side.iter().map(|c| c * c)));
    let side: [f64; 3] = std::array::from_fn(|i| side[i] / n);
    let up = [
        u[1] * side[2] - u[2] * side[1],
        u[2] * side[0] - u[0] * side[2],
        u[0] * side[1] - u[1] * side[0],
    ];
    let r = w / sqrt(3.0);
    let corners: Vec<[f64; 3]> = (0..3)
        .map(|k| {
            let ang = radians((90 + 120 * k) as f64);
            std::array::from_fn(|i| r * cos(ang) * side[i] + r * sin(ang) * up[i])
        })
        .collect();
    let at = |p: [f64; 3], c: &[f64; 3]| (p[0] + c[0], p[1] + c[1], p[2] + c[2]);
    for (k, c) in corners.iter().enumerate() {
        fsweep!(
            f,
            format!("{name}_chord{k}"),
            &[at(a, c), at(b, c)],
            0.012,
            material,
            verts = 6
        );
    }
    for k in 0..3 {
        let (c0, c1) = (corners[k], corners[(k + 1) % 3]);
        let mut pts = Vec::new();
        for j in 0..=bays {
            let t = j as f64 / bays as f64;
            let c = if j % 2 == 0 { c0 } else { c1 };
            pts.push((
                a[0] + d[0] * t + c[0],
                a[1] + d[1] * t + c[1],
                a[2] + d[2] * t + c[2],
            ));
        }
        fsweep!(
            f,
            format!("{name}_lace{k}"),
            &pts,
            0.005,
            material,
            verts = 4
        );
    }
}

pub fn dance_floor() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            f,
            "base",
            1.88,
            0.06,
            1.88,
            (0, 0.03, 0),
            M.base,
            bevel = 0.01,
            bevel_segments = 1
        );
        kit::moulding!(
            f,
            "edge",
            // (its corners cut across 4 mm)
            &[
                (-0.936, -0.94),
                (0.936, -0.94),
                (0.94, -0.936),
                (0.94, 0.936),
                (0.936, 0.94),
                (-0.936, 0.94),
                (-0.94, 0.936),
                (-0.94, -0.936)
            ],
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.03, 0.05),
                (0.01, 0.068),
                (0.0, 0.068)
            ],
            M.steel,
            at = 0.0,
            plane = Facing::Top
        );
        fbox!(f, "grid", 1.82, 0.004, 1.82, (0, 0.062, 0), M.dark);
        for s in [-1, 1] {
            let sf = s as f64;
            _truss(
                f,
                &format!("post{s}"),
                (sf * 0.9, 0.06, 0.0),
                (sf * 0.9, 2.29, 0.0),
                0.09,
                M.steel,
                12,
            );
            soft!(
                f,
                format!("post_foot{s}"),
                0.14,
                0.03,
                0.3,
                (sf * 0.9, 0.075, 0.0),
                M.steel,
                round = 0.01,
                round_segments = 1
            );
            _speaker(
                f,
                &format!("speaker{s}"),
                sf * 0.78,
                0.5,
                -0.8,
                0.24,
                0.88,
                0.24,
                &[0.62, 0.84],
                1,
            );
            // a moving-head light on the beam, angled down onto the floor
            revolve!(
                format!("spot{s}"),
                &eased(
                    &[
                        (0.0, -0.07),
                        (0.05, -0.07),
                        (0.06, 0.04),
                        (0.068, 0.06),
                        (0.06, 0.06),
                        (0.0, 0.05)
                    ],
                    AUTO_WIDTH
                ),
                at = f.at(sf * 0.68, 2.18, 0.0),
                material = M.dark,
                segments = 14,
                rot = (0, s * 140, 0)
            );
            fcyl!(
                f,
                format!("spot_lens{s}"),
                0.058,
                0.006,
                (sf * 0.642, 2.135, 0.0),
                if s < 0 { M.magenta } else { M.turq },
                rot = (0, s * 40, 0),
                verts = 14
            );
            fsweep!(
                f,
                format!("spot_hanger{s}"),
                &[(sf * 0.68, 2.27, 0.0), (sf * 0.68, 2.22, 0.0)],
                0.01,
                M.dark,
                verts = 6
            );
        }
        _truss(
            f,
            "beam",
            (-0.95, 2.29, 0.0),
            (0.95, 2.29, 0.0),
            0.09,
            M.steel,
            12,
        );
        fsweep!(
            f,
            "ball_cord",
            &[(0.0, 2.25, 0.0), (0.0, 2.05, 0.0)],
            0.004,
            M.chrome,
            verts = 5
        );
        revolve!(
            "ball_motor",
            &[(0.0, 0.0), (0.02, 0.0), (0.02, 0.03), (0.0, 0.03)],
            at = f.at(0, 2.03, 0.0),
            material = M.dark,
            segments = 8
        );
        fsphere!(
            f,
            "mirror_ball",
            0.17,
            (0, 1.86, 0.0),
            M.mirror,
            segments = 16,
            rings = 10,
            smooth = false
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The tiles in two checkerboard parts, origins on the floor: `tiles` lifts one set, then the other.
    let mut groups = Vec::new();
    for parity in [0, 1] {
        let tiles = new_parts(|| {
            for i in 0..4 {
                for j in 0..4 {
                    if (i + j) % 2 != parity {
                        continue;
                    }
                    let m = TILES[(i * 3 + j * 2) % TILES.len()];
                    let (x, z) = (
                        -1.5 * PITCH + PITCH * i as f64,
                        -1.5 * PITCH + PITCH * j as f64,
                    );
                    fbox!(
                        f,
                        format!("tile{i}{j}"),
                        TILE,
                        0.014,
                        TILE,
                        (x, TILE_Y + 0.007, z),
                        m,
                        bevel = 0.005,
                        bevel_segments = 1
                    );
                }
            }
        });
        let name = if parity == 0 { "tiles_a" } else { "tiles_b" };
        groups.push(hinged(join(&tiles, name, None), f.at(0, TILE_Y, 0)));
    }
    let objs = settle!(centre = false);
    turns(&groups, "tiles", (0.0, 0.0, LIFT), Some(1.2));
    objs
}

// --- the party photo wall (1x1): a cork board on legs, snapshots, string lights, balloons, a gift ---------
const BOARD_Z: f64 = WALL + 0.07;
const NEW_PHOTO: (f64, f64, f64) = (0.25, 0.88, 4.0); // (x, height, tilt) of the photo that grows

/// A snapshot pinned to the board: a white print, its picture, a round-headed pin.
#[allow(clippy::too_many_arguments)]
fn _photo(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, tilt: f64, material: Mat, pin: Mat) {
    fbox!(
        f,
        name,
        0.17,
        0.2,
        0.004,
        (x, y, z),
        M.paper,
        rot = (0.0, tilt, 0.0),
        bevel = 0.0015,
        bevel_segments = 1
    );
    fbox!(
        f,
        format!("{name}_picture"),
        0.14,
        0.13,
        0.002,
        (x - 0.02 * sin(radians(tilt)), y + 0.02, z + 0.003),
        material,
        rot = (0.0, tilt, 0.0)
    );
    let (px, py) = (x - 0.09 * sin(radians(tilt)), y + 0.09);
    revolve!(
        format!("{name}_pin"),
        &[
            (0.0, 0.0),
            (0.003, 0.0),
            (0.003, 0.006),
            (0.012, 0.008),
            (0.012, 0.014),
            (0.006, 0.02),
            (0.0, 0.021)
        ],
        at = f.at(px, py, z),
        material = pin,
        segments = 8,
        rot = (90, 0, 0)
    );
}

/// A balloon: lofted, rounder at the top and narrowing to its knot; its string swept in a loose curl to `tie`.
#[allow(clippy::too_many_arguments)]
fn _balloon(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    material: Mat,
    tie: (f64, f64, f64),
) {
    let b = f.at(x, y, z);
    let rows = [
        (-1.15, 0.0),
        (-1.1, 0.12),
        (-0.95, 0.42),
        (-0.6, 0.78),
        (0.0, 1.0),
        (0.55, 0.92),
        (0.9, 0.6),
        (1.0, 0.0),
    ];
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(h, k)| ring!(r * k, r * k, z = b.z + r * h, verts = 12, at = (b.x, b.y)))
        .collect();
    loft!(name, &sections, material = material);
    fcone!(
        f,
        format!("{name}_knot"),
        0.012,
        0.004,
        0.02,
        (x, y - r * 1.2, z),
        material,
        verts = 6
    );
    let (tx, ty, tz) = tie;
    let mut pts = Vec::new();
    for k in 0..7 {
        let t = k as f64 / 6.0;
        pts.push((
            x + (tx - x) * t + 0.02 * sin(t * 9.0),
            y - r * 1.22 + (ty - y + r * 1.22) * t,
            z + (tz - z) * t + 0.02 * cos(t * 9.0),
        ));
    }
    fsweep!(
        f,
        format!("{name}_string"),
        &pts,
        0.0018,
        M.paper,
        verts = 3,
        resolution = 2
    );
}

pub fn photo_wall() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let zb = BOARD_Z;
    let still = new_parts(|| {
        fbox!(f, "cork", 0.82, 1.0, 0.03, (0, 1.2, zb), M.cork);
        kit::moulding!(
            f,
            "board_frame",
            &[(-0.44, 0.685), (0.44, 0.685), (0.44, 1.715), (-0.44, 1.715)],
            // (its back corners cut across 4 mm)
            &[
                (0.004, -0.03),
                (0.046, -0.03),
                (0.05, -0.026),
                (0.05, 0.02),
                (0.035, 0.034),
                (0.008, 0.034),
                (0.0, 0.024), // (the inner chamfer under 60°: at 0.02 it turned exactly 60°)
                (0.0, -0.026)
            ],
            M.wood,
            at = zb
        );
        for x in [-0.42, 0.42] {
            fbox!(
                f,
                format!("leg{x}"),
                0.05,
                1.74,
                0.05,
                (x, 0.87, zb),
                M.wood,
                bevel = 0.008,
                bevel_segments = 2 // (one segment lays its wood as lines on two corners)
            );
            soft!(
                f,
                format!("foot{x}"),
                0.06,
                0.04,
                0.34,
                (x, 0.02, WALL + 0.17),
                M.wood,
                round = 0.012,
                round_segments = 1
            );
        }
        let spots = [
            (-0.27, 1.45, 6),
            (0.0, 1.5, -4),
            (0.26, 1.44, 8),
            (-0.24, 1.15, -7),
            (0.03, 1.18, 3),
            (0.28, 1.12, -5),
            (-0.25, 0.86, 4),
            (0.0, 0.88, -8),
        ];
        for (i, (x, y, r)) in spots.into_iter().enumerate() {
            _photo(
                f,
                &format!("photo{i}"),
                x,
                y,
                zb + 0.017,
                r as f64,
                PHOTOS[i],
                [M.pin_a, M.pin_b, M.sunny][i % 3],
            );
        }
        // string lights draped across the top: one wire in a sag, a socket and a bulb at each dip
        let pts: Vec<(f64, f64)> = (0..17)
            .map(|t| {
                let tf = t as f64;
                (
                    -0.42 + 0.84 * tf / 16.0,
                    1.66 - 0.09 * sin(PI * tf / 16.0) - 0.012 * (t % 2) as f64,
                )
            })
            .collect();
        let wire: Vec<(f64, f64, f64)> = pts.iter().map(|&(x, yy)| (x, yy, zb + 0.04)).collect();
        fsweep!(f, "wire", &wire, 0.0035, M.string, verts = 4);
        for &(x, yy) in pts[2..pts.len() - 1].iter().step_by(2) {
            fcyl!(
                f,
                format!("socket{x:.2}"),
                0.008,
                0.018,
                (x, yy - 0.012, zb + 0.045),
                M.string,
                verts = 6
            );
            fsphere!(
                f,
                format!("bulb{x:.2}"),
                0.018,
                (x, yy - 0.036, zb + 0.048),
                M.bulb,
                scale = (1, 1, 1.3),
                segments = 8,
                rings = 5
            );
        }
        // three balloons tied to the top rail
        for (x, y, z, m) in [
            (0.3, 1.92, -0.25, M.coral),
            (0.32, 2.04, -0.18, M.turq),
            (0.2, 2.0, -0.15, M.sunny),
        ] {
            _balloon(
                f,
                &format!("balloon{x}{y}"),
                x,
                y,
                z,
                0.1,
                m,
                (0.36, 1.715, zb + 0.03),
            );
        }
        // a gift on the floor in front: a box, a ribbon both ways, a bow of two loops and tails
        fbox!(
            f,
            "gift",
            0.3,
            0.2,
            0.22,
            (0.15, 0.1, -0.05),
            M.sunny,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "ribbon_a",
            0.04,
            0.204,
            0.224,
            (0.15, 0.101, -0.05),
            M.magenta
        );
        fbox!(
            f,
            "ribbon_b",
            0.304,
            0.204,
            0.04,
            (0.15, 0.101, -0.05),
            M.magenta
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                f,
                format!("bow{s}"),
                &[
                    (0.15, 0.2, -0.05),
                    (0.15 + sf * 0.04, 0.24, -0.06),
                    (0.15 + sf * 0.075, 0.225, -0.05),
                    (0.15 + sf * 0.05, 0.205, -0.04),
                    (0.15, 0.2, -0.05)
                ],
                0.0,
                M.magenta,
                profile = rect(0.004, 0.035),
                up = (0, 0, 1),
                resolution = 2
            );
            fsweep!(
                f,
                format!("bow_tail{s}"),
                &[
                    (0.15, 0.2, -0.05),
                    (0.15 + sf * 0.03, 0.202, -0.02),
                    (0.15 + sf * 0.05, 0.201, 0.0)
                ],
                0.0,
                M.magenta,
                profile = rect(0.003, 0.03),
                up = (0, 1, 0)
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // A new snapshot in the bottom row, origin on its middle and built at full size, then shrunk to nothing: `photo`
    // grows it onto the board.
    let (px, py, pr) = NEW_PHOTO;
    let photo = new_parts(|| _photo(f, "new_photo", px, py, zb + 0.017, pr, M.magenta, M.pin_b));
    let photo = hinged(join(&photo, "photo", None), f.at(px, py, zb + 0.017));
    photo.set_scale_axis(0, 0.001);
    photo.set_scale_axis(2, 0.001);
    let objs = settle!(centre = false);
    grow!(photo, "photo", seconds = 0.8, axis = Axis::X);
    grow!(photo, "photo", seconds = 0.8, axis = Axis::Z);
    objs
}
