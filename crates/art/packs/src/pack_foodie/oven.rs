//! The wood-fired pizza oven (2x1): a brick stand and plastered dome, a prep counter at its right.

use super::common::{_basil, _leaf, _lofted, _pot, M, WALL};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, cosd, sind, sqrt};
use artkit::motion::{Path, keys};
use artkit::parts::{join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, grow, jitter, loft, revolve, ring, settle};

const OX: f64 = -0.38; // the oven's middle (game x)
const HEARTH: f64 = 0.92; // the oven floor and the mouth's sill
const MOUTH_Z: f64 = 0.403; // the mouth's dark face

/// Brick faces in running bond on a front at depth `z`, over the box x0..x1, y0..y1.
#[allow(clippy::too_many_arguments)]
fn _bricks(f: &GameFrame, x0: f64, x1: f64, y0: f64, y1: f64, z: f64, rows: usize, length: f64) {
    let h = (y1 - y0) / rows as f64;
    for r in 0..rows {
        let y = y0 + h * (r as f64 + 0.5);
        let mut x = x0 - if r % 2 != 0 { length / 2.0 } else { 0.0 };
        let mut k = 0;
        while x < x1 - 0.01 {
            let (a, b) = (x.max(x0), (x + length).min(x1));
            if b - a > 0.02 {
                let m = [M.brick, M.brick_light, M.brick][(r + k) % 3];
                fbox!(
                    f,
                    numbered("brick"),
                    b - a - 0.008,
                    h - 0.01,
                    0.01,
                    ((a + b) / 2.0, y, z),
                    m
                );
            }
            x += length;
            k += 1;
        }
    }
}

/// A split log from game point `a` to `b`: a crooked, barked round, its cut ends pale.
fn _log(f: &GameFrame, name: &str, a: (f64, f64, f64), b: (f64, f64, f64), r: f64, seed: f64) {
    let (pa, pb) = ([a.0, a.1, a.2], [b.0, b.1, b.2]);
    let mid: [f64; 3] =
        std::array::from_fn(|i| (pa[i] + pb[i]) / 2.0 + if i == 1 { 0.008 } else { 0.0 });
    let log = fsweep!(
        f,
        name,
        &[a, (mid[0], mid[1], mid[2]), b],
        r,
        M.log,
        verts = 8,
        resolution = 2,
        scale = [1.0, 1.05, 0.95]
    );
    jitter!(log, r * 0.12, size = 0.04, seed = seed);
    for (k, p) in [pa, pb].into_iter().enumerate() {
        let dist = artkit::py::norm(&[p[0] - mid[0], p[1] - mid[1], p[2] - mid[2]]);
        let q: [f64; 3] =
            std::array::from_fn(|i| p[i] + (p[i] - mid[i]) * 0.002 / 1e-6_f64.max(dist));
        fsweep!(
            f,
            format!("{name}_end{k}"),
            &[(p[0], p[1], p[2]), (q[0], q[1], q[2])],
            r * 0.86,
            M.log_end,
            verts = 8
        );
    }
}

/// A tongue of flame standing on `y`: a body lofted from a round foot, swelling, then drawn up into a tip that leans;
/// `flat` thins it front to back (flames behind a grille).
#[allow(clippy::too_many_arguments)]
fn _flame(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    flat: f64,
    turn: f64,
) -> Obj {
    let b = f.at(x, y, z);
    let rows = [
        (0.0, 0.7),
        (0.15, 1.0),
        (0.4, 0.85),
        (0.65, 0.5),
        (0.85, 0.22),
        (1.0, 0.02),
    ];
    let secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(t, s)| {
            let lean = 0.3 * r * t * t;
            ring!(
                r * s,
                r * s * flat,
                z = b.z + h * t,
                verts = 8,
                at = (b.x - lean * cosd(turn), b.y)
            )
        })
        .collect();
    loft!(name, &secs, material = material)
}

pub fn pizza_oven() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let front = 0.43;
    let still = new_parts(|| {
        // the stand: two brick pillars, a lintel over the log niche, the hearth slab on top
        for s in [-1, 1] {
            let x = OX + s as f64 * 0.45;
            fbox!(
                f,
                numbered("pillar"),
                0.16,
                0.7,
                front - WALL,
                (x, 0.35, (front + WALL) / 2.0),
                M.brick
            );
            _bricks(f, x - 0.08, x + 0.08, 0.0, 0.7, front + 0.004, 9, 0.16);
        }
        fbox!(
            f,
            "lintel",
            1.06,
            0.16,
            front - WALL,
            (OX, 0.78, (front + WALL) / 2.0),
            M.brick
        );
        _bricks(f, OX - 0.53, OX + 0.53, 0.7, 0.86, front + 0.004, 2, 0.2);
        fbox!(
            f,
            "niche_back",
            0.76,
            0.7,
            0.02,
            (OX, 0.35, WALL + 0.01),
            M.brick_dark
        );
        fbox!(
            f,
            "niche_floor",
            0.76,
            0.03,
            front - WALL - 0.02,
            (OX, 0.015, (front + WALL) / 2.0 - 0.01),
            M.stone
        );
        for i in 0..3 {
            for j in 0..(4 - i) {
                let (x, y) = (
                    OX - 0.2 + j as f64 * 0.13 + i as f64 * 0.065,
                    0.08 + i as f64 * 0.088,
                );
                _log(
                    f,
                    &numbered("log"),
                    (x, y, -0.26),
                    (x + 0.01 * ((i + j) % 3 - 1) as f64, y, 0.34),
                    0.05,
                    (i * 4 + j) as f64,
                );
            }
        }
        fextrude!(
            f,
            "hearth",
            &rounded_rect(1.12, front - WALL + 0.02, 0.02, 2),
            0.06,
            (OX, HEARTH - 0.03, (front + WALL) / 2.0 + 0.01),
            M.stone,
            plane = Facing::Top,
            bevel = 0.01,
            bevel_segments = 1
        );
        // the dome, the brick arch of its mouth and the flue
        let dome = revolve!(
            "dome",
            &[
                (0.0, 0.0),
                (0.46, 0.0),
                (0.458, 0.06),
                (0.448, 0.14),
                (0.42, 0.24),
                (0.38, 0.33),
                (0.32, 0.42),
                (0.24, 0.5),
                (0.14, 0.565),
                (0.06, 0.594),
                (0.0, 0.6)
            ],
            at = f.at(OX, HEARTH, -0.06),
            material = M.plaster,
            segments = 32,
            scale = (1, 0.82, 1)
        );
        jitter!(dome, 0.006, size = 0.12, seed = 3.0);
        let mut arch: Vec<(f64, f64)> = (0..181)
            .step_by(10)
            .map(|a| (0.3 * cosd(a as f64), 0.3 * sind(a as f64)))
            .collect();
        arch.extend(
            (0..181)
                .rev()
                .step_by(10)
                .map(|a| (0.2 * cosd(a as f64), 0.2 * sind(a as f64))),
        );
        fextrude!(
            f,
            "arch",
            &arch,
            0.14,
            (OX, HEARTH, 0.33),
            M.brick,
            bevel = 0.006,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "mouth",
            0.2,
            0.006,
            (OX, HEARTH, MOUTH_Z),
            M.mouth,
            rot = (90, 0, 0),
            verts = 24
        );
        for k in 0..9_usize {
            let a = 10 + 20 * k as i32;
            fbox!(
                f,
                numbered("voussoir"),
                0.075,
                0.09,
                0.012,
                (
                    OX + 0.25 * cosd(a as f64),
                    HEARTH + 0.25 * sind(a as f64),
                    MOUTH_Z + 0.003
                ),
                [M.brick_light, M.brick_dark][k % 2],
                rot = (0, a - 90, 0),
                bevel = 0.002,
                bevel_segments = 1
            );
        }
        for (k, (dx, r)) in [(-0.07, 0.04), (0.0, 0.05), (0.08, 0.035)]
            .into_iter()
            .enumerate()
        {
            let ember = fsphere!(
                f,
                numbered("ember"),
                r,
                (OX + dx, HEARTH + 0.004, MOUTH_Z - 0.01),
                M.ember,
                scale = (1, 0.3, 0.45),
                segments = 8,
                rings = 4
            );
            jitter!(ember, r * 0.1, size = 0.02, seed = k as f64);
        }
        _lofted(
            f,
            "flue",
            OX + 0.12,
            1.42,
            -0.2,
            &[(0.0, 0.08), (0.04, 0.075), (0.06, 0.07), (0.42, 0.07)],
            M.iron,
            14,
        );
        revolve!(
            "flue_cap",
            &[
                (0.0, 0.0),
                (0.075, 0.0),
                (0.075, 0.03),
                (0.13, 0.05),
                (0.13, 0.06),
                (0.04, 0.1),
                (0.0, 0.11)
            ],
            at = f.at(OX + 0.12, 1.79, -0.2),
            material = M.iron,
            segments = 14
        );
        // the prep counter: a stone cabinet with an oak frame-and-panel door, its pull, a worktop
        let cx = 0.56;
        fbox!(
            f,
            "counter",
            0.74,
            0.84,
            0.62,
            (cx, 0.42, WALL + 0.31),
            M.stone,
            bevel = 0.008,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "counter_top",
            &rounded_rect(0.78, 0.66, 0.02, 2),
            0.05,
            (cx, 0.865, WALL + 0.32),
            M.worktop,
            plane = Facing::Top,
            bevel = 0.01,
            bevel_segments = 2
        );
        kit::shaker!(
            f,
            "counter_door",
            0.6,
            0.62,
            (cx, 0.43, WALL + 0.63),
            0.02,
            M.door_oak,
            rail = 0.07
        );
        kit::pull!(
            f,
            "counter_pull",
            cx,
            0.66,
            WALL + 0.64,
            0.12,
            M.iron,
            r = 0.006,
            out = 0.025
        );
        // on it: a ball of dough on a floured board, a bowl of sauce, oil, a pot of basil; its front right left free, for
        // what a cook sets down while grilling in the oven (`SET_DOWN_SPOTS`)
        let dough = fsphere!(
            f,
            "dough_ball",
            0.07,
            (cx + 0.04, 0.915, -0.25),
            M.dough,
            scale = (1, 1, 0.55),
            segments = 12,
            rings = 7
        );
        jitter!(dough, 0.004, size = 0.04);
        fcyl!(
            f,
            "flour_board",
            0.16,
            0.012,
            (cx + 0.04, 0.896, -0.25),
            M.board,
            verts = 18,
            bevel = 0.004,
            bevel_segments = 1
        );
        revolve!(
            "bowl_sauce",
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.06, 0.04),
                (0.062, 0.05),
                (0.056, 0.05),
                (0.05, 0.044),
                (0.0, 0.044)
            ],
            at = f.at(cx + 0.3, 0.89, -0.19),
            material = M.cup,
            segments = 14
        );
        fcyl!(
            f,
            "sauce",
            0.05,
            0.004,
            (cx + 0.3, 0.93, -0.19),
            M.tomato,
            verts = 12
        );
        _lofted(
            f,
            "oil",
            cx + 0.14,
            0.89,
            -0.37,
            &[
                (0.0, 0.03),
                (0.005, 0.035),
                (0.15, 0.035),
                (0.19, 0.014),
                (0.24, 0.012),
            ],
            M.oil,
            10,
        );
        let top = _pot(f, cx + 0.27, -0.33, 0.89, 0.06, 0.1);
        _basil(f, cx + 0.27, -0.33, top, 0.75);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The door: an iron grille over the mouth, the fire showing through it; `door_off` lifts it aside.
    let (r, dz) = (0.19, MOUTH_Z + 0.012);
    let door = new_parts(|| {
        // its rim in one bent bar: round the arch and back along the sill
        let mut rim: Vec<(f64, f64, f64)> = (0..181)
            .rev()
            .step_by(15)
            .map(|a| {
                (
                    OX + r * cosd(a as f64),
                    HEARTH + 0.012 + r * sind(a as f64),
                    dz,
                )
            })
            .collect();
        rim.push((OX - r + 0.004, HEARTH + 0.012, dz));
        kit::tube!(f, "door_arc", &rim, 0.009, M.iron, verts = 6, round = 1);
        for dx in [-0.12, -0.06, 0.0, 0.06, 0.12] {
            let top = HEARTH + 0.01 + sqrt(r * r - dx * dx);
            kit::tube!(
                f,
                numbered("door_bar"),
                &[(OX + dx, HEARTH + 0.012, dz), (OX + dx, top, dz)],
                0.006,
                M.iron,
                verts = 6,
                round = 1
            );
        }
        kit::tube!(
            f,
            "door_rail",
            &[(OX - 0.17, HEARTH + 0.1, dz), (OX + 0.17, HEARTH + 0.1, dz)],
            0.006,
            M.iron,
            verts = 6,
            round = 1
        );
        kit::pull!(
            f,
            "door_handle",
            OX,
            HEARTH + 0.1,
            dz,
            0.16,
            M.door_oak,
            r = 0.011,
            out = 0.05
        );
    });
    let door = join(&door, "oven_door", Some(f.at(OX, HEARTH, dz)));
    // The peel, a pizza on it, lying on the counter: `door_off` carries it round to the mouth and in.
    let (px, pz) = (0.48, -0.1);
    let peel = new_parts(|| {
        let blade = smooth_outline(
            &[
                (-0.15, -0.15),
                (0.15, -0.15),
                (0.15, 0.1),
                (0.04, 0.16),
                (-0.04, 0.16),
                (-0.15, 0.1),
            ],
            1,
            true,
        );
        fextrude!(
            f,
            "peel_blade",
            &blade,
            0.01,
            (px, 0.896, pz),
            M.peel,
            plane = Facing::Top,
            bevel = 0.003,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "peel_handle",
            &[(px, 0.9, pz + 0.15), (px, 0.903, pz + 0.48)],
            0.0,
            M.peel,
            profile = rounded_rect(0.02, 0.035, 0.008, 1),
            up = (0, 0, 1),
            scale = [1.0, 0.8]
        );
        revolve!(
            "pizza",
            &[
                (0.0, 0.0),
                (0.13, 0.0),
                (0.135, 0.008),
                (0.128, 0.018),
                (0.116, 0.016),
                (0.11, 0.01),
                (0.0, 0.01)
            ],
            at = f.at(px, 0.901, pz),
            material = M.crust,
            segments = 20
        );
        fcyl!(
            f,
            "pizza_sauce",
            0.11,
            0.004,
            (px, 0.912, pz),
            M.tomato,
            verts = 20
        );
        for k in 0..7 {
            let (a, d) = ((k * 51) as f64, 0.04 + 0.04 * (k % 2) as f64);
            let mozz = fcyl!(
                f,
                numbered("mozzarella"),
                0.018,
                0.006,
                (px + d * cosd(a), 0.915, pz + d * sind(a)),
                M.mozzarella,
                verts = 8
            );
            jitter!(mozz, 0.002, size = 0.01, seed = k as f64);
        }
        for k in 0..4 {
            let a = (k * 90 + 30) as f64;
            _leaf(
                f,
                &numbered("pizza_basil"),
                px + 0.05 * cosd(a),
                0.918,
                pz + 0.05 * sind(a),
                0.03,
                a,
                0.0,
                M.basil,
            );
        }
    });
    let peel = join(&peel, "peel", Some(f.at(px, 0.896, pz)));
    // The fire in the mouth, flat flames behind the grille, built full height and flattened onto the sill: `flames`
    // grows them.
    let fire = new_parts(|| {
        for (k, (dx, rr, h, m)) in [
            (-0.1, 0.035, 0.12, M.flame),
            (-0.04, 0.045, 0.2, M.flame),
            (0.03, 0.045, 0.17, M.flame),
            (0.1, 0.035, 0.11, M.flame),
            (-0.02, 0.03, 0.12, M.flame_core),
            (0.05, 0.025, 0.09, M.flame_core),
        ]
        .into_iter()
        .enumerate()
        {
            let z = MOUTH_Z + 0.004 + 0.002 * if k > 3 { 1.0 } else { 0.0 };
            _flame(
                f,
                &numbered("flame"),
                OX + dx,
                HEARTH,
                z,
                rr,
                h,
                m,
                0.25,
                (60 * k) as f64,
            );
        }
    });
    let fire = join(&fire, "flames", Some(f.at(OX, HEARTH, MOUTH_Z)));
    fire.set_scale_axis(2, 0.001);
    let objs = settle!(centre = false);
    // door_off: the door lifts off the mouth and is set down on the ground before the stand's left pillar; the peel
    // follows, round in front of the oven and in through the mouth (Blender axes: x = -game x, y = -game depth, z =
    // height).
    for (i, (lift, down)) in [(0.0, 0.42), (-0.06, -0.06), (0.03, -(HEARTH - 0.005))]
        .into_iter()
        .enumerate()
    {
        let rest = door.location()[i] as f64;
        keys(
            door,
            "door_off",
            Path::Location,
            i,
            &[
                (0.0, rest),
                (0.2, rest + lift),
                (0.6, rest + down),
                (1.4, rest + down),
            ],
        );
    }
    let l = peel.location();
    let (lx, ly, lz) = (l.x as f64, l.y as f64, l.z as f64);
    // game path: up off the counter, forward, along to the oven's middle, back into the mouth
    let (gx, gz) = (OX - px, 0.25 - pz);
    let keys_x = [
        (0.0, lx),
        (0.35, lx),
        (0.6, lx),
        (1.0, lx - gx),
        (1.4, lx - gx),
    ];
    let keys_y = [
        (0.0, ly),
        (0.35, ly),
        (0.6, ly - 0.62),
        (1.0, ly - 0.62),
        (1.4, ly - gz),
    ];
    let keys_z = [
        (0.0, lz),
        (0.35, lz + 0.06),
        (0.6, lz + 0.06),
        (1.0, lz + 0.06),
        (1.4, lz + 0.03),
    ];
    for (i, points) in [keys_x, keys_y, keys_z].iter().enumerate() {
        keys(peel, "door_off", Path::Location, i, points);
    }
    grow!(fire, "flames", seconds = 0.8);
    objs
}
