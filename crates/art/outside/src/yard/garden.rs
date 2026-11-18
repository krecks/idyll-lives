//! The raised vegetable bed, the garden shed, the compost bin and the rain barrel.

use super::common::{F, M, N, blob, board, fillet, ground, mesh_of, name, pot, prism, rod, tube};
use super::play::settle_in_place;
use artkit::far::{is_low, low};
use artkit::forms::{Scale, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{Axis, M4, P3, PI, TAU, V3, atan2, cos, cosd, degrees, radians, sin, sind};
use artkit::motion::{Ease, Rock, Squash, Stages, Swing, rock, squash, stages, swing};
use artkit::parts::{hinged, join, join_all, measure, new_parts};
use artkit::place::Facing;
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{cone, fbox, fcone, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, torus};

// --- the raised vegetable bed (2x1): boards round the soil, lettuces, carrots and tomato plants --------------
const BED_TOP: f64 = 0.32; // the soil's top
// The growth stages (`grow`, stages, as the plants' edibles): ripe at rest; picked, the plants pulled and the
// crop gone; growing, the plants lower, no crop; planted, seedlings. The plants flatten and sink into the soil
// (shares of their height), so each seedling comes up where its plant stands.
const PLANTS: [(f64, f64, f64); 3] = [(1.0, 1.0, 0.3), (1.0, 1.0, 0.75), (1.0, 1.0, 0.4)]; // picked, growing, planted
const SINK: (f64, f64, f64) = (0.32, 0.03, 0.1);
const CROP: f64 = 0.001;

/// A lettuce standing on height `y`: its outer leaves ruffled round a cupped heart, lofted in one.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn lettuce(name: &str, x: f64, y: f64, z: f64, r: f64, h: f64, material: Mat, seed: f64) -> Obj {
    let b = F.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let n = low(14, 6);
    let rows = [
        (0.0, 0.6, 0.05),
        (0.12, 1.0, 0.16),
        (0.38, 0.92, 0.12),
        (0.66, 0.62, 0.07),
        (0.8, 0.32, 0.02),
        (0.7, 0.0, 0.0),
    ];
    let mut secs = Vec::new();
    for (t, k, wave) in rows {
        let mut lp = Vec::new();
        for i in 0..n {
            let a = TAU * i as f64 / n as f64;
            let w = 1.0 + wave * sin(5.0 * a + seed);
            // (as the script has it: its rings' depth is the soil's height and their height the lettuce's depth)
            lp.push(P3 {
                x: bx + r * k * w * cos(a),
                y: bz + r * k * w * sin(a),
                z: by + h * t + h * wave * 0.4 * cos(3.0 * a + seed),
            });
        }
        secs.push(lp);
    }
    loft!(name, &secs, material = material, angle = 70)
}

/// A carrot's frond from game (x, y, z): a feathery leaf `h` tall, leaning out along `a` degrees.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn frond(name: &str, x: f64, y: f64, z: f64, a: f64, h: f64, material: Mat, lean: f64) -> Obj {
    let mut out = vec![(0.0, 0.0)];
    let steps = low(3, 1);
    for i in 1..=steps {
        let v = h * i as f64 / steps as f64;
        let w = 0.016 * sin(PI * i as f64 / (steps + 1) as f64);
        out.push((w, v - h / steps as f64 * 0.5));
        out.push((w * 0.4, v));
    }
    let back: Vec<(f64, f64)> = out[1..].iter().rev().map(|&(u, v)| (-u, v)).collect();
    out.extend(back);
    let leaf = fextrude!(F, name, &out, 0.003, (x, y, z), material);
    let m = M4::translation(V3::from(F.at(x, y, z)))
        * M4::rotation(radians(a - 90.0), Axis::Z)
        * M4::rotation(lean, Axis::X);
    leaf.set_matrix_world(m)
}

pub fn raised_bed() -> Vec<Obj> {
    let mut rnd = Random::new(7);
    let still = new_parts(|| {
        for z in [-0.4, 0.4] {
            for y in [0.085, 0.255] {
                fbox!(
                    F,
                    name("board"),
                    1.84,
                    0.162,
                    0.06,
                    (0, y, z),
                    M.bed,
                    bevel = low(0.01, 0.0),
                    bevel_segments = 1
                );
            }
        }
        for x in [-0.89, 0.89] {
            for y in [0.085, 0.255] {
                fbox!(
                    F,
                    name("board"),
                    0.06,
                    0.162,
                    0.74,
                    (x, y, 0),
                    M.bed,
                    bevel = low(0.01, 0.0),
                    bevel_segments = 1
                );
            }
            for z in [-0.395, 0.395] {
                fbox!(
                    F,
                    name("post"),
                    0.09,
                    0.36,
                    0.09,
                    (x * 0.995, 0.18, z),
                    M.bed_post,
                    bevel = low(0.01, 0.0),
                    bevel_segments = 1
                );
                if !is_low() {
                    // a pyramid cap on each post
                    cone!(
                        name("post_cap"),
                        0.064,
                        0.0,
                        0.024,
                        at = F.at(x * 0.995, 0.372, z),
                        material = M.bed_post,
                        verts = 4,
                        rot = (0, 0, 45)
                    );
                }
            }
        }
        ground(
            "soil",
            1.74,
            0.76,
            BED_TOP - 0.005,
            M.soil,
            low((14, 6), (2, 1)),
            0.012,
            5.0,
            (0.0, 0.0),
            0.0,
        );
        fbox!(
            F,
            "label",
            0.07,
            0.05,
            0.006,
            (-0.8, BED_TOP + 0.1, 0.34),
            M.trim,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            F,
            "label_stick",
            0.012,
            0.12,
            0.004,
            (-0.8, BED_TOP + 0.02, 0.34),
            M.trim
        );
        for x in [-0.45, 0.0, 0.45] {
            // the tomatoes' canes
            rod(
                &name("cane"),
                (x + 0.14, BED_TOP - 0.02, 0.22),
                (x + 0.15, BED_TOP + 0.2, 0.22),
                0.005,
                M.haft,
                4,
            );
        }
    });
    if !is_low() {
        join(&still, "bed", Some(P3::from((0, 0, 0))));
    }
    let xs: Vec<f64> = (0..6).map(|i| -0.7 + i as f64 * 0.28).collect();
    let jit: Vec<(f64, f64)> = (0..18)
        .map(|_| (rnd.uniform(-0.03, 0.03), rnd.uniform(-0.03, 0.03)))
        .collect();
    let alt: Vec<f64> = xs.iter().step_by(2).copied().collect();
    let plants = new_parts(|| {
        for (i, &x) in xs.iter().enumerate() {
            // lettuces, at the back
            let (dx, dz) = jit[i];
            lettuce(
                &name("lettuce"),
                x + dx,
                BED_TOP - 0.01,
                -0.24 + dz,
                0.11,
                0.15,
                [M.lettuce, M.lettuce_dark][i % 2],
                i as f64 * 1.7,
            );
        }
        for (i, &x) in xs.iter().enumerate() {
            // carrot tops, in the middle
            let (dx, dz) = jit[6 + i];
            for k in 0..low(4, 2) {
                frond(
                    &name("frond"),
                    x + dx,
                    BED_TOP,
                    dz,
                    (k * 90 + i * 25) as f64,
                    0.17 + 0.02 * (k % 2) as f64,
                    M.carrot_top,
                    0.22,
                );
            }
        }
        for (i, &x) in alt.iter().enumerate() {
            // bushy tomato plants up their canes, at the front
            let (dx, dz) = jit[12 + i];
            for k in 0..3 {
                let a = (k * 120 + i * 40) as f64;
                blob(
                    x + 0.14 + dx + 0.06 * cosd(a),
                    BED_TOP - 0.01 + k as f64 * 0.04,
                    0.22 + dz + 0.06 * sind(a),
                    0.09 - 0.012 * k as f64,
                    M.leaf,
                    (20 + i * 3 + k) as f64,
                    (1.0, 1.0, 0.9),
                    0.45,
                );
            }
        }
    });
    let crop = new_parts(|| {
        for (i, &x) in xs.iter().enumerate() {
            // carrot shoulders
            let (dx, dz) = jit[6 + i];
            fcone!(
                F,
                name("carrot"),
                0.028,
                0.02,
                0.035,
                (x + dx, BED_TOP + 0.012, dz),
                M.carrot,
                verts = low(8, 5)
            );
        }
        for (i, &x) in alt.iter().enumerate() {
            // tomatoes on the bushes, each with its green calyx
            let (dx, dz) = jit[12 + i];
            for k in 0..4 {
                let a = (k * 90 + 20 + i * 30) as f64;
                let p = (
                    x + 0.14 + dx + 0.12 * cosd(a),
                    BED_TOP + 0.08 + 0.04 * (k % 2) as f64,
                    0.22 + dz + 0.12 * sind(a),
                );
                fsphere!(
                    F,
                    name("tomato"),
                    0.035,
                    p,
                    M.tomato,
                    scale = (1, 1, 0.85),
                    segments = low(8, 6),
                    rings = low(5, 4)
                );
                if !is_low() {
                    fcone!(
                        F,
                        name("calyx"),
                        0.016,
                        0.004,
                        0.012,
                        (p.0, p.1 + 0.03, p.2),
                        M.green,
                        verts = 5
                    );
                }
            }
        }
    });
    if is_low() {
        // a far version: all one still part, ripe
        return settle_in_place(&[join_all("raisedBed")]);
    }
    let origin = F.at(0, BED_TOP, 0);
    let plants = join(&plants, "plants", Some(origin));
    let crop = join(&crop, "crop", Some(origin));
    let objs = settle_in_place(&meshes());
    let height = measure(&[plants]).1[2] - plants.location().z as f64; // the plants' top over the soil
    let sink = (SINK.0 * height, SINK.1 * height, SINK.2 * height);
    stages(
        plants,
        "grow",
        PLANTS[0],
        PLANTS[1],
        PLANTS[2],
        Stages {
            sink: Some(sink),
            ..Stages::default()
        },
    );
    stages(crop, "grow", CROP, CROP, CROP, Stages::default());
    squash(
        crop,
        "harvest",
        0.001,
        Squash {
            seconds: 1.0,
            ..Squash::default()
        },
    ); // the ripe crop picked away (and back after, if not picked)
    // Tending: the plants and their crop rustle, 3° back and forth about the soil line through the bed's middle, on
    // top of the growth stage (objectMotion.ts plays a use clip after it).
    for part in [plants, crop] {
        rock(
            part,
            "rustle",
            3.0,
            Rock {
                seconds: 0.8,
                axis: Axis::X,
            },
        );
    }
    objs
}

// --- the garden shed (2x2): sage weatherboards, a felt roof, a window with a window box; the door opens ------
const SW: f64 = 1.8; // the walls' width and depth, and their middle (depth)
const SD: f64 = 1.6;
const SZ: f64 = -0.08;
const EAVE: f64 = 1.72; // the walls' top and the roof's ridge
const RIDGE: f64 = 2.3;
const DOOR_X: f64 = -0.3; // the doorway: its middle, width, top (from 0.08)
const DOOR_W: f64 = 0.68;
const DOOR_H: f64 = 1.48;
const FRONT_Z: f64 = SZ + SD / 2.0; // the front wall's face
const OVER: f64 = 0.08; // the roof's overhang at the eaves
const LAP: f64 = 0.16; // the weatherboards' pitch

/// A weatherboarded wall's section from height y0 to y1, `t` thick: each board's foot `out` proud of the one under
/// it (a saw tooth outside, flat inside), the boards on the walls' one grid (from `base`, LAP apart), as (out from the
/// inside face, height) points round it.
fn laps(y0: f64, y1: f64, t: f64, out: f64, base: f64) -> Vec<(f64, f64)> {
    let cuts: Vec<f64> = (1..40)
        .map(|k| base + k as f64 * LAP)
        .filter(|&c| y0 + 1e-6 < c && c < y1 - 1e-6)
        .collect();
    let first = base + 0.0_f64.max(((y0 - base) / LAP + 1e-6).floor()) * LAP;
    // how far the board the section starts in has thinned by y0
    let f0 = (y0 - first) / LAP;
    let mut pts = vec![(0.0, y0), (t + out * (1.0 - f0), y0)];
    for c in cuts {
        pts.push((t, c));
        pts.push((t + out, c));
    }
    pts.push((t, y1));
    pts.push((0.0, y1));
    pts
}

/// A weatherboarded wall from a0 to a1 along game x (`along_x`, its inside face at depth `at`, facing `side` ±1 in
/// depth) or along depth (its inside face at x `at`, facing `side` ±1 in x), from height y0 to y1: one sweep
/// of the lapped section.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn lap_wall(
    name: &str,
    along_x: bool,
    a0: f64,
    a1: f64,
    at: f64,
    y0: f64,
    y1: f64,
    side: i32,
    t: f64,
) -> Obj {
    let prof = laps(y0, y1, t, 0.02, 0.08);
    if along_x {
        return wall_sweep(
            name,
            [(a0, 0.0, at), (a1, 0.0, at)],
            &prof,
            V3::from((0, 0, side)),
        );
    }
    wall_sweep(
        name,
        [(at, 0.0, a0), (at, 0.0, a1)],
        &prof,
        V3::from((side, 0, 0)),
    )
}

/// Sweeps a wall's section [(out, height)] along a straight game path at floor level: `out` along game direction
/// `out_dir`, height up; cut every `step` along, so the baked shading has corners along the wall, not only at its
/// buried ends ("Shading by corners").
fn wall_sweep(name: &str, path: [(f64, f64, f64); 2], prof: &[(f64, f64)], o: V3) -> Obj {
    let step = 0.3;
    let (a, b) = (V3::from(path[0]), V3::from(path[1]));
    let n = if !is_low() {
        1.max(artkit::py::round((b - a).length() / step)) as usize
    } else {
        1
    };
    let mut vs = Vec::new();
    for i in 0..=n {
        let p = a + (b - a) * (i as f64 / n as f64);
        for &(u, v) in prof {
            let q = p + o * u;
            vs.push(V3::from(F.at(q.x as f64, v, q.z as f64)));
        }
    }
    let m = prof.len() as u32;
    let mut faces = Vec::new();
    for i in 0..n as u32 {
        for k in 0..m {
            let k2 = (k + 1) % m;
            faces.push(vec![
                i * m + k,
                i * m + k2,
                (i + 1) * m + k2,
                (i + 1) * m + k,
            ]);
        }
    }
    faces.push((0..m).rev().collect());
    faces.push((n as u32 * m..(n as u32 + 1) * m).collect());
    let mats = vec![0; faces.len()];
    mesh_of(name, vs, &faces, &[M.shed], &mats, false)
}

fn shed_walls() {
    let t = 0.04;
    let base = 0.08;
    let h = EAVE - base;
    let yc = base + h / 2.0;
    fbox!(
        F,
        "plinth",
        SW + 0.06,
        base,
        SD + 0.06,
        (0, base / 2.0, SZ),
        M.plinth,
        bevel = 0.01,
        bevel_segments = 1
    );
    let (zb, zf) = (SZ - SD / 2.0, FRONT_Z);
    if is_low() {
        fbox!(F, "wall_back", SW, h, t, (0, yc, zb + t / 2.0), M.shed);
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                name("wall_side"),
                t,
                h,
                SD - 2.0 * t,
                (s * (SW / 2.0 - t / 2.0), yc, SZ),
                M.shed
            );
        }
    } else {
        // weatherboards: each wall one sweep of the lapped section, the boards' feet proud of the ones under them
        lap_wall(
            "wall_back",
            true,
            -SW / 2.0,
            SW / 2.0,
            zb + t,
            base,
            EAVE,
            -1,
            t,
        );
        for s in [-1, 1] {
            lap_wall(
                &name("wall_side"),
                false,
                zb + t,
                zf - t,
                s as f64 * (SW / 2.0 - t),
                base,
                EAVE,
                s,
                t,
            );
        }
    }
    // the front, round the doorway
    let fz = FRONT_Z - t / 2.0;
    let (x0, x1) = (DOOR_X - DOOR_W / 2.0, DOOR_X + DOOR_W / 2.0);
    if is_low() {
        let left = x0 + SW / 2.0;
        let right = SW / 2.0 - x1;
        fbox!(
            F,
            "wall_front_l",
            left,
            h,
            t,
            (-SW / 2.0 + left / 2.0, yc, fz),
            M.shed
        );
        fbox!(
            F,
            "wall_front_r",
            right,
            h,
            t,
            (SW / 2.0 - right / 2.0, yc, fz),
            M.shed
        );
        fbox!(
            F,
            "wall_front_top",
            DOOR_W,
            EAVE - (base + DOOR_H),
            t,
            (DOOR_X, (EAVE + base + DOOR_H) / 2.0, fz),
            M.shed
        );
    } else {
        lap_wall(
            "wall_front_l",
            true,
            -SW / 2.0,
            x0,
            zf - t,
            base,
            EAVE,
            1,
            t,
        );
        lap_wall("wall_front_r", true, x1, SW / 2.0, zf - t, base, EAVE, 1, t);
        lap_wall(
            "wall_front_top",
            true,
            x0,
            x1,
            zf - t,
            base + DOOR_H,
            EAVE,
            1,
            t,
        );
    }
    // the gables
    for (z, z0) in [(FRONT_Z - t, FRONT_Z), (SZ - SD / 2.0, SZ - SD / 2.0 + t)] {
        prism(
            &name("gable"),
            &[
                (N::F(-SW / 2.0), EAVE),
                (N::F(SW / 2.0), EAVE),
                (N::I(0), RIDGE),
            ],
            z.min(z0),
            z.max(z0),
            M.shed,
        );
    }
    // corner boards and the door and window frames
    for s in [-1.0, 1.0] {
        for z in [FRONT_Z + 0.012, SZ - SD / 2.0 - 0.012] {
            fbox!(
                F,
                name("corner"),
                0.07,
                h,
                0.024,
                (s * (SW / 2.0 - 0.025), yc, z),
                M.trim,
                bevel = low(0.004, 0.0),
                bevel_segments = 1
            );
            fbox!(
                F,
                name("corner_side"),
                0.024,
                h,
                0.07,
                (
                    s * (SW / 2.0 + 0.012),
                    yc,
                    z - 0.023 * (if z > SZ { 1.0 } else { -1.0 })
                ),
                M.trim,
                bevel = low(0.004, 0.0),
                bevel_segments = 1
            );
        }
    }
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            name("door_trim"),
            0.06,
            DOOR_H,
            0.03,
            (
                DOOR_X + s * (DOOR_W / 2.0 + 0.03),
                base + DOOR_H / 2.0,
                FRONT_Z + 0.01
            ),
            M.trim,
            bevel = 0.005
        );
    }
    fbox!(
        F,
        "door_head",
        DOOR_W + 0.18,
        0.07,
        0.035,
        (DOOR_X, base + DOOR_H + 0.035, FRONT_Z + 0.012),
        M.trim,
        bevel = 0.005
    );
    if !is_low() {
        // a drip cap over the door head
        fbox!(
            F,
            "door_drip",
            DOOR_W + 0.2,
            0.015,
            0.05,
            (DOOR_X, base + DOOR_H + 0.077, FRONT_Z + 0.02),
            M.trim,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
}

fn shed_roof() {
    let run = artkit::py::hypot(
        SW / 2.0 + OVER,
        (RIDGE - EAVE) * (SW / 2.0 + OVER) / (SW / 2.0),
    );
    let slope = degrees(atan2(RIDGE - EAVE, SW / 2.0));
    for s in [-1.0, 1.0] {
        let x = s * (SW / 2.0 + OVER) / 2.0;
        let y = RIDGE - (RIDGE - EAVE) * ((SW / 2.0 + OVER) / 2.0) / (SW / 2.0) + 0.03;
        fbox!(
            F,
            name("roof"),
            run,
            0.04,
            SD + 0.24,
            (x, y, SZ),
            M.roof,
            rot = (0.0, -s * slope, 0.0),
            bevel = low(0.008, 0.0)
        );
        if !is_low() {
            // the felt's laps: a strip proud at every course down the slope, a drip edge along the eave
            for k in 1..4 {
                let f = k as f64 / 4.0;
                let (cx, cy) = (
                    s * (SW / 2.0 + OVER) * f,
                    RIDGE - (RIDGE - EAVE) * (SW / 2.0 + OVER) * f / (SW / 2.0) + 0.052,
                );
                fbox!(
                    F,
                    name("felt_lap"),
                    0.05,
                    0.006,
                    SD + 0.24,
                    (cx, cy, SZ),
                    M.roof,
                    rot = (0.0, -s * slope, 0.0)
                );
            }
            let (ex, ey) = (
                s * (SW / 2.0 + OVER),
                RIDGE - (RIDGE - EAVE) * (SW / 2.0 + OVER) / (SW / 2.0) + 0.01,
            );
            fbox!(
                F,
                name("drip"),
                0.03,
                0.05,
                SD + 0.24,
                (ex - s * 0.004, ey, SZ),
                M.roof,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for z in [FRONT_Z + 0.12, SZ - SD / 2.0 - 0.12] {
            // barge boards on the gables' edges
            fbox!(
                F,
                name("barge"),
                run - 0.04,
                0.1,
                0.025,
                (x, y - 0.045, z),
                M.trim,
                rot = (0.0, -s * slope, 0.0)
            );
        }
    }
    fbox!(
        F,
        "ridge_cap",
        0.14,
        0.04,
        SD + 0.26,
        (0, RIDGE + 0.05, SZ),
        M.roof,
        bevel = low(0.012, 0.0),
        bevel_segments = 1
    );
    if !is_low() {
        // a finial where the barge boards meet
        for z in [FRONT_Z + 0.115, SZ - SD / 2.0 - 0.115] {
            fbox!(
                F,
                name("finial"),
                0.05,
                0.16,
                0.035,
                (0, RIDGE - 0.02, z),
                M.trim,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
    }
}

/// In the shadow behind the doorway: dark lining, a shelf of pots, a fork and a rake on the wall.
fn shed_inside() {
    let t = 0.04;
    let (lz0, lz1) = (SZ - SD / 2.0 + t, FRONT_Z - t);
    fbox!(
        F,
        "in_back",
        SW - 2.0 * t,
        EAVE - 0.08,
        0.005,
        (0, (EAVE + 0.08) / 2.0, lz0 + 0.003),
        M.inside
    );
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            name("in_side"),
            0.005,
            EAVE - 0.08,
            lz1 - lz0,
            (s * (SW / 2.0 - t - 0.003), (EAVE + 0.08) / 2.0, SZ),
            M.inside
        );
    }
    fbox!(
        F,
        "in_floor",
        SW - 2.0 * t,
        0.005,
        lz1 - lz0,
        (0, 0.083, SZ),
        M.inside
    );
    fbox!(
        F,
        "in_ceiling",
        SW - 2.0 * t,
        0.005,
        lz1 - lz0 + 0.04,
        (0, EAVE - 0.003, SZ),
        M.inside
    );
    fbox!(
        F,
        "in_front",
        SW - 2.0 * t,
        EAVE - 0.08,
        0.005,
        (0, (EAVE + 0.08) / 2.0, lz1 - 0.003),
        M.inside
    );
    fbox!(
        F,
        "shelf",
        1.0,
        0.03,
        0.3,
        (-0.3, 1.0, lz0 + 0.15),
        M.box_wood
    );
    fbox!(
        F,
        "bench",
        1.0,
        0.04,
        0.5,
        (-0.3, 0.8, lz0 + 0.25),
        M.box_wood
    );
    for x in [-0.65, -0.45, -0.25] {
        pot(x, 1.015, lz0 + 0.14, 0.06, 0.1, None);
    }
    for (x, head) in [(0.0, "fork"), (-0.75, "rake")] {
        rod(
            &name("tool"),
            (x, 0.1, lz0 + 0.3),
            (x + 0.04, 1.25, lz0 + 0.06),
            0.012,
            M.haft,
            6,
        );
        let fork = head == "fork";
        fbox!(
            F,
            name(head),
            if fork { 0.14 } else { 0.3 },
            if fork { 0.12 } else { 0.03 },
            0.02,
            (x, 0.15, lz0 + 0.29),
            M.axe
        );
    }
}

/// A black T-hinge on a door at game (x = its knuckle, y, z = the door's face): a strap tapering to a rounded end
/// across the door, a plate on the frame, and three bolt heads.
fn t_hinge(name: &str, x: f64, y: f64, z: f64, length: f64, material: Mat) {
    let strap = [
        (0.0, -0.022),
        (length - 0.02, -0.01),
        (length, -0.006),
        (length + 0.006, 0.0),
        (length, 0.006),
        (length - 0.02, 0.01),
        (0.0, 0.022),
        (-0.03, 0.04),
        (-0.05, 0.04),
        (-0.05, -0.04),
        (-0.03, -0.04),
    ];
    let out: Vec<(f64, f64)> = strap.iter().map(|&(u, v)| (x + u, y + v)).collect();
    fextrude!(
        F,
        name,
        &out,
        0.005,
        (0, 0, z),
        material,
        bevel = 0.0015,
        bevel_segments = 1
    );
    for (k, u) in [0.08, length * 0.55, length - 0.03].into_iter().enumerate() {
        fcyl!(
            F,
            format!("{name}_bolt{k}"),
            0.007,
            0.006,
            (x + u, y, z + 0.004),
            material,
            rot = (90, 0, 0),
            verts = 6
        );
    }
}

pub fn shed_small() -> Vec<Obj> {
    shed_walls();
    shed_roof();
    if !is_low() {
        shed_inside();
    }
    // the window, right of the door: a moulded white frame mitred round four dark panes and their bars; a sill
    let (wx, wy) = (0.42, 1.12);
    let profile: &[(f64, f64)] = if !is_low() {
        &[
            (0.0, 0.0),
            (0.0, 0.03),
            (0.012, 0.04),
            (0.045, 0.04),
            (0.055, 0.03),
            (0.055, 0.0),
        ]
    } else {
        &[(0.0, 0.0), (0.0, 0.04), (0.055, 0.04), (0.055, 0.0)]
    };
    crate::playground::common::moulding(
        &F,
        "window_frame",
        &[
            (wx + 0.28, wy - 0.23),
            (wx + 0.28, wy + 0.23),
            (wx - 0.28, wy + 0.23),
            (wx - 0.28, wy - 0.23),
        ],
        profile,
        M.trim,
        FRONT_Z - 0.002,
        Facing::Front,
        artkit::shapes::SHARP,
    );
    fbox!(
        F,
        "window_pane",
        0.46,
        0.36,
        0.02,
        (wx, wy, FRONT_Z + 0.002),
        M.pane
    );
    fbox!(
        F,
        "window_bar_v",
        0.022,
        0.36,
        0.02,
        (wx, wy, FRONT_Z + 0.014),
        M.trim
    );
    fbox!(
        F,
        "window_bar_h",
        0.46,
        0.022,
        0.02,
        (wx, wy, FRONT_Z + 0.014),
        M.trim
    );
    fextrude!(
        F,
        "window_sill",
        &[(-0.33, 0.0), (0.33, 0.0), (0.33, 0.035), (-0.33, 0.035)],
        0.08,
        (wx, wy - 0.262, FRONT_Z + 0.04),
        M.trim,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    // the window box on brackets, in flower
    let b = F.at(wx, 0, FRONT_Z + 0.095);
    let (bx, by) = (b.x, b.y);
    let bx_ = [(-0.3, -0.085), (0.3, -0.085), (0.3, 0.085), (-0.3, 0.085)];
    let secs: Vec<Vec<P3>> = [(-0.43, 0.9), (-0.29, 1.0), (-0.285, 0.96)]
        .iter()
        .map(|&(dy, k)| {
            bx_.iter()
                .map(|&(u, v)| P3 {
                    x: bx - u * k,
                    y: by - v * k,
                    z: wy + dy,
                })
                .collect()
        })
        .collect();
    loft!("window_box", &secs, material = M.box_wood);
    for s in [-1.0, 1.0] {
        fextrude!(
            F,
            name("bracket"),
            &[(0.0, 0.0), (0.0, 0.14), (0.15, 0.14), (0.15, 0.12)],
            0.03,
            (wx + s * 0.22, wy - 0.57, FRONT_Z),
            M.box_wood,
            plane = Facing::Side,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
    }
    let flowers = [M.flower_pink, M.flower_yellow, M.flower_red, M.flower_lilac];
    for i in 0..4 {
        let x = wx - 0.21 + i as f64 * 0.14;
        blob(
            x,
            wy - 0.3,
            FRONT_Z + 0.1,
            0.07,
            M.green,
            (30 + i) as f64,
            (1.0, 1.0, 0.8),
            0.35,
        );
        for k in 0..3 {
            fsphere!(
                F,
                name("flower"),
                0.025,
                (
                    x + (k - 1) as f64 * 0.035,
                    wy - 0.2 + 0.01 * (k % 2) as f64,
                    FRONT_Z + 0.09 + 0.03 * (k - 1) as f64
                ),
                flowers[((i + k) % 4) as usize],
                segments = low(8, 4),
                rings = low(5, 3)
            );
        }
    }
    if !is_low() {
        join(&meshes(), "shed", Some(P3::from((0, 0, 0))));
    }
    // The door: ledged and braced boards, black T-hinges on the left, a ring latch on the right; it swings out.
    let door = new_parts(|| {
        let (y0, x0) = (0.09, DOOR_X - DOOR_W / 2.0);
        let dz = FRONT_Z + 0.015;
        for k in 0..5 {
            let w = (DOOR_W - 0.02) / 5.0;
            fbox!(
                F,
                name("door_board"),
                w - 0.006,
                DOOR_H - 0.02,
                0.03,
                (
                    x0 + 0.01 + w * (k as f64 + 0.5),
                    y0 + (DOOR_H - 0.02) / 2.0,
                    dz
                ),
                M.shed_door,
                bevel = low(0.004, 0.0),
                bevel_segments = 1
            );
        }
        for y in [0.3, 1.27] {
            fbox!(
                F,
                name("ledge"),
                DOOR_W - 0.08,
                0.1,
                0.02,
                (DOOR_X, y, dz + 0.024),
                M.shed_door,
                bevel = low(0.005, 0.0),
                bevel_segments = 1
            );
            if is_low() {
                fbox!(
                    F,
                    name("hinge"),
                    0.3,
                    0.03,
                    0.008,
                    (x0 + 0.15, y, dz + 0.038),
                    M.iron
                );
            } else {
                t_hinge(&name("hinge"), x0 + 0.02, y, dz + 0.0365, 0.32, M.iron);
            }
        }
        board(
            "brace",
            (x0 + 0.08, 0.36, dz + 0.024),
            (x0 + DOOR_W - 0.08, 1.21, dz + 0.024),
            0.09,
            0.02,
            M.shed_door,
            (0, 0, 1),
            low(0.005, 0.0),
        );
        let lx = x0 + DOOR_W - 0.08;
        fextrude!(
            F,
            "latch_plate",
            &rounded_rect(0.06, 0.15, 0.012, low(2, 1)),
            0.006,
            (lx, 0.86, dz + 0.018),
            M.iron
        );
        if !is_low() {
            torus!(
                "latch_ring",
                0.03,
                0.005,
                at = F.at(lx, 0.83, dz + 0.026),
                material = M.iron,
                rot = (90, 0, 0),
                major_segments = 12,
                minor_segments = 4
            );
            fcyl!(
                F,
                "latch_boss",
                0.012,
                0.012,
                (lx, 0.86, dz + 0.026),
                M.iron,
                rot = (90, 0, 0),
                verts = 8
            );
            fbox!(
                F,
                "latch_bar",
                0.12,
                0.012,
                0.012,
                (lx + 0.05, 0.9, dz + 0.024),
                M.iron,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
    });
    if is_low() {
        return settle_in_place(&[join_all("shedSmall")]);
    }
    let door = hinged(
        join(&door, "door", None),
        F.at(DOOR_X - DOOR_W / 2.0, 0.8, FRONT_Z + 0.015),
    );
    let objs = settle_in_place(&meshes());
    swing(
        door,
        "door_open",
        100.0,
        Swing {
            seconds: 1.0,
            ease: Ease::BackOut,
            ..Swing::default()
        },
    );
    objs
}

// --- the compost bin (1x1): a slatted box on corner posts, the heap inside; the lid lifts ---------------------
const CB: (f64, f64, f64) = (0.8, 0.76, -0.08); // width, depth, middle (depth)
const CB_TOP: f64 = 0.72;

pub fn compost_bin() -> Vec<Obj> {
    let (w, d, cz) = CB;
    let (z0, z1) = (cz - d / 2.0, cz + d / 2.0);
    for x in [-w / 2.0 + 0.03, w / 2.0 - 0.03] {
        for z in [z0 + 0.03, z1 - 0.03] {
            fbox!(
                F,
                name("post"),
                0.06,
                CB_TOP - 0.012,
                0.06,
                (x, (CB_TOP - 0.012) / 2.0, z),
                M.bin,
                bevel = low(0.008, 0.0),
                bevel_segments = 1
            );
            if !is_low() {
                // each post's top cut to a shallow point
                cone!(
                    name("post_cap"),
                    0.042,
                    0.0,
                    0.012,
                    at = F.at(x, CB_TOP - 0.006, z),
                    material = M.bin,
                    verts = 4,
                    rot = (0, 0, 45)
                );
            }
        }
    }
    for (k, y) in [0.08, 0.25, 0.42, 0.59].into_iter().enumerate() {
        fbox!(
            F,
            name("slat_back"),
            w - 0.04,
            0.12,
            0.025,
            (0, y + 0.02, z0 + 0.01),
            M.bin,
            bevel = low(0.006, 0.0),
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                name("slat_side"),
                0.025,
                0.12,
                d - 0.12,
                (s * (w / 2.0 - 0.01), y + 0.02, cz),
                M.bin,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
        if k < 3 {
            // the front boards: the top one lifted out
            fbox!(
                F,
                name("slat_front"),
                w - 0.12,
                0.12,
                0.025,
                (0, y + 0.02, z1 - 0.01),
                M.bin,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
    }
    if !is_low() {
        // galvanised nails in the slats' ends
        for y in [0.08, 0.25, 0.42, 0.59] {
            for s in [-1.0, 1.0] {
                for z in [z0 + 0.03, z1 - 0.03] {
                    fcyl!(
                        F,
                        name("nail"),
                        0.005,
                        0.004,
                        (s * (w / 2.0 + 0.001), y + 0.02, z),
                        M.can,
                        rot = (0, 90, 0),
                        verts = 5
                    );
                }
            }
        }
    }
    for s in [-1.0, 1.0] {
        // the front boards' guide battens
        fbox!(
            F,
            name("guide"),
            0.03,
            CB_TOP - 0.06,
            0.03,
            (s * (w / 2.0 - 0.075), (CB_TOP - 0.06) / 2.0, z1 - 0.035),
            M.bin,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
    }
    // the heap: a mound of compost, peelings and greens on top, a few lumps
    blob(0.0, 0.02, cz, 0.36, M.compost, 2.0, (1.0, 1.0, 0.82), 0.25);
    for (k, (x, z, m)) in [
        (0.12, cz + 0.06, M.greens),
        (-0.15, cz - 0.08, M.peel),
        (0.02, cz - 0.14, M.greens),
        (-0.05, cz + 0.12, M.peel),
        (0.18, cz - 0.1, M.compost),
        (-0.2, cz + 0.08, M.greens),
    ]
    .into_iter()
    .enumerate()
    {
        blob(
            x,
            0.56 - 0.02 * (k / 4) as f64,
            z,
            0.05,
            m,
            (9 + k) as f64,
            (1.4, 1.0, 0.4),
            0.5,
        );
    }
    if !is_low() {
        for (k, (x, z)) in [(0.06, cz - 0.02), (-0.1, cz + 0.02), (0.12, cz + 0.14)]
            .into_iter()
            .enumerate()
        {
            fcone!(
                F,
                name("shell"),
                0.025,
                0.018,
                0.014,
                (x, 0.585, z),
                M.shell,
                verts = 6,
                rot = ((10 * k) as i32, 0, 0)
            );
        }
    }
    if !is_low() {
        join(&meshes(), "bin", Some(P3::from((0, 0, 0))));
    }
    // The lid: four boards on two battens, a wooden grip, hinged along the back.
    let lid = new_parts(|| {
        for k in 0..4 {
            let bw = (d + 0.04) / 4.0;
            fbox!(
                F,
                name("lid_board"),
                w + 0.04,
                0.025,
                bw - 0.008,
                (0, CB_TOP + 0.0125, z0 - 0.02 + bw * (k as f64 + 0.5)),
                M.bin,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                name("batten"),
                0.05,
                0.03,
                d - 0.04,
                (s * (w / 2.0 - 0.1), CB_TOP - 0.012, cz),
                M.bin,
                bevel = low(0.004, 0.0),
                bevel_segments = 1
            );
        }
        fextrude!(
            F,
            "grip",
            &[
                (-0.1, 0.0),
                (0.1, 0.0),
                (0.085, 0.03),
                (0.06, 0.035),
                (-0.06, 0.035),
                (-0.085, 0.03)
            ],
            0.04,
            (0, CB_TOP + 0.025, z1 - 0.03),
            M.bin,
            bevel = low(0.006, 0.0),
            bevel_segments = 1
        );
    });
    if is_low() {
        return settle_in_place(&[join_all("compostBin")]);
    }
    let lid = hinged(join(&lid, "lid", None), F.at(0, CB_TOP, z0 - 0.02));
    let objs = settle_in_place(&meshes());
    swing(
        lid,
        "lid_open",
        -75.0,
        Swing {
            seconds: 0.8,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    objs
}

// --- the rain barrel (1x1): a ribbed green barrel with a lid and its inlet, a brass tap ------------------------
// Free-standing, so no downpipe (Q-06): one ended in mid-air wherever the barrel stood. It stands in the middle of its
// tile (it stood 0.15 m back, the pipe against the wall).
const RB_Z: f64 = 0.0;

pub fn rain_barrel() -> Vec<Obj> {
    // the barrel: moulded in one, a foot ring, two hoop bands, ribs running up between them, a rolled top edge
    let b = F.at(0, 0, RB_Z);
    let (bx, by) = (b.x, b.y);
    let n = low(32, 12);
    let mut rows = vec![
        (0.0, 0.292, false),
        (0.008, 0.302, false),
        (0.05, 0.302, false),
        (0.07, 0.28, false),
        (0.25, 0.285, true),
        (0.285, 0.296, false),
        (0.315, 0.296, false),
        (0.34, 0.286, true),
        (0.62, 0.283, true),
        (0.645, 0.293, false),
        (0.675, 0.293, false),
        (0.7, 0.282, true),
        (0.88, 0.272, true),
        (0.9, 0.28, false),
        (0.93, 0.28, false),
        (0.935, 0.26, false),
    ];
    if is_low() {
        rows = [0, 2, 3, 5, 6, 9, 10, 12, 13, 14, 15]
            .iter()
            .map(|&i| rows[i])
            .collect();
    }
    let mut secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(y, r, _)| ring!(r, r, z = y, verts = n, at = (bx, by)))
        .collect();
    if !is_low() {
        // the ribs: every other point a little proud where `rib`
        secs = secs
            .iter()
            .zip(&rows)
            .map(|(sec, &(_, _, rib))| {
                sec.iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let k = 1.0 + if rib && i % 2 == 1 { 0.025 } else { 0.0 };
                        P3 {
                            x: bx + (p.x - bx) * k,
                            y: by + (p.y - by) * k,
                            z: p.z,
                        }
                    })
                    .collect()
            })
            .collect();
    }
    loft!("barrel", &secs, material = M.barrel, angle = 35);
    // the lid: a domed disc with a raised rim, a mesh inlet at the back and its ring
    revolve!(
        "lid",
        &[
            (0.0, 0.0),
            (0.262, 0.0),
            (0.27, 0.012),
            (0.262, 0.024),
            (0.24, 0.026),
            (0.12, 0.034),
            (0.0, 0.036)
        ],
        at = F.at(0, 0.925, RB_Z),
        material = M.barrel_dark,
        segments = low(24, 10)
    );
    fcyl!(
        F,
        "lid_inlet",
        0.065,
        0.016,
        (0, 0.962, RB_Z - 0.1),
        M.barrel_dark,
        verts = low(12, 6)
    );
    if !is_low() {
        fcyl!(
            F,
            "inlet_mesh",
            0.05,
            0.004,
            (0, 0.969, RB_Z - 0.1),
            M.tyre,
            verts = 12
        );
        for k in 0..3 {
            let a = 120 * k + 30;
            fbox!(
                F,
                name("lid_grip"),
                0.08,
                0.008,
                0.012,
                (0.13 * cosd(a as f64), 0.958, RB_Z + 0.13 * sind(a as f64)),
                M.barrel_dark,
                rot = (0, 0, a),
                bevel = 0.003,
                bevel_segments = 1
            );
        }
    }
    // the brass tap at the front: a flange, a body, a spout curving down, a lever on top
    fcyl!(
        F,
        "tap_flange",
        0.032,
        0.012,
        (0, 0.17, RB_Z + 0.29),
        M.brass,
        rot = (90, 0, 0),
        verts = low(10, 6)
    );
    tube(
        &F,
        "tap_spout",
        &fillet(
            &[
                (0.0, 0.17, RB_Z + 0.29),
                (0.0, 0.17, RB_Z + 0.355),
                (0.0, 0.12, RB_Z + 0.367),
            ],
            0.03,
            3,
        ),
        0.016,
        M.brass,
        Some(low(10, 6)),
        1,
        Scale::One(1.0),
    );
    fcyl!(
        F,
        "tap_body",
        0.024,
        0.05,
        (0, 0.2, RB_Z + 0.33),
        M.brass,
        verts = low(10, 6)
    );
    fsweep!(
        F,
        "tap_lever",
        &[(0, 0.23, RB_Z + 0.33), (0, 0.235, RB_Z + 0.37)],
        0,
        M.brass,
        profile = rounded_rect(0.012, 0.03, 0.004, 1),
        up = (1, 0, 0),
        scale = [1.0, 1.2]
    );
    settle_in_place(&[join_all("rainBarrel")])
}
