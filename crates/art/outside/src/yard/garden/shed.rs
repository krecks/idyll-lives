//! The shed's walls, roof and inside, and the hinges on its door.

use super::*;

// --- the garden shed (2x2): sage weatherboards, a felt roof, a window with a window box; the door opens ------
const SW: f64 = 1.8; // the walls' width and depth, and their middle (depth)
const SD: f64 = 1.6;
const SZ: f64 = -0.08;
const EAVE: f64 = 1.72; // the walls' top and the roof's ridge
const RIDGE: f64 = 2.3;
pub(super) const DOOR_X: f64 = -0.3; // the doorway: its middle, width, top (from 0.08)
pub(super) const DOOR_W: f64 = 0.68;
pub(super) const DOOR_H: f64 = 1.48;
pub(super) const FRONT_Z: f64 = SZ + SD / 2.0; // the front wall's face
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
    artkit::forms::object(name, vs, &faces, Some(M.shed), false, SHARP, false)
}

pub(super) fn shed_walls() {
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
                numbered("wall_side"),
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
                &numbered("wall_side"),
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
            &numbered("gable"),
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
                numbered("corner"),
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
                numbered("corner_side"),
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
            numbered("door_trim"),
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

pub(super) fn shed_roof() {
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
            numbered("roof"),
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
                    numbered("felt_lap"),
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
                numbered("drip"),
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
                numbered("barge"),
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
                numbered("finial"),
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
pub(super) fn shed_inside() {
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
            numbered("in_side"),
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
            &numbered("tool"),
            (x, 0.1, lz0 + 0.3),
            (x + 0.04, 1.25, lz0 + 0.06),
            0.012,
            M.haft,
            6,
        );
        let fork = head == "fork";
        fbox!(
            F,
            numbered(head),
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
pub(super) fn t_hinge(name: &str, x: f64, y: f64, z: f64, length: f64, material: Mat) {
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
