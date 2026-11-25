//! The garage's tyre rack and tool chest.

use super::common::{F, WALL, mark, own_palette, tyre};
use artkit::forms::Holes;
use artkit::math::{Axis, TAU, atan2, cos, degrees, pow, radians, sin, sqrt};
use artkit::parts::{join_all, new_parts};
use artkit::place::Facing;
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, kit_tube, settle};

own_palette! { TYRES:
    body = ("#E07A1F", satin),        // the frames' orange uprights
    beam = ("#5C6168", metal),        // the beams
    label = ("#F2C14E", gloss), label2 = ("#3E7BC8", gloss), label3 = ("#F1F0EC", gloss),
}

own_palette! { CHEST:
    body = ("#C0201C", gloss),        // the chest's red
    foam = ("#2B2C2E", matte),        // the drawers' foam liners
}

// --- the tyre rack: 2x1 against the wall, steel shelves of new tyres ---------------------------------------------
pub const TR_D: f64 = 0.66; // the frames' depth, a tyre's height: back on the wall's face
pub const TR_LEVELS: [f64; 3] = [0.12, 0.76, 1.4];

pub fn tyre_rack() -> Vec<Obj> {
    let m = &TYRES;
    let z0 = WALL;
    let zc = z0 + TR_D / 2.0;
    new_parts(|| {
        for s in [-1, 1] {
            let x = s as f64 * 0.93;
            for (k, z) in [z0 + 0.035, z0 + TR_D - 0.035].into_iter().enumerate() {
                fbox!(
                    F,
                    format!("upright{s}{k}"),
                    0.06,
                    2.0,
                    0.07,
                    (x, 1.0, z),
                    m.body,
                    bevel = 0.004,
                    bevel_segments = 1
                );
                fbox!(
                    F,
                    format!("footplate{s}{k}"),
                    0.1,
                    0.012,
                    0.1,
                    (x, 0.006, z + (if k == 0 { 0.015 } else { -0.015 })),
                    m.iron
                );
                for j in 0..24 {
                    // a slot punched in the upright's inner face
                    let (u, y) = (x - s as f64 * 0.0305, 0.2 + j as f64 * 0.075);
                    mark(
                        &format!("slot{s}{k}{j}"),
                        [
                            (u, y - 0.0125, z - 0.006),
                            (u, y - 0.0125, z + 0.006),
                            (u, y + 0.0125, z + 0.006),
                            (u, y + 0.0125, z - 0.006),
                        ],
                        (-s as f64, 0.0, 0.0),
                        m.dark,
                    );
                }
            }
            for (j, y) in [0.3, 0.9, 1.5, 1.95].into_iter().enumerate() {
                // the frame's bracing
                fbox!(
                    F,
                    format!("brace{s}{j}"),
                    0.03,
                    0.03,
                    TR_D - 0.12,
                    (x, y, zc),
                    m.body
                );
            }
            for (j, (ya, yb)) in [(0.3, 0.9), (0.9, 1.5)].into_iter().enumerate() {
                let l = py::hypot(TR_D - 0.12, yb - ya);
                fbox!(
                    F,
                    format!("diag{s}{j}"),
                    0.025,
                    l,
                    0.025,
                    (x, (ya + yb) / 2.0, zc),
                    m.body,
                    rot = (
                        (if j % 2 != 0 { 1.0 } else { -1.0 })
                            * degrees(atan2(TR_D - 0.12, yb - ya)),
                        0,
                        0
                    )
                );
            }
        }
        let labels = [m.label, m.label2, m.label3];
        for (lv, y) in TR_LEVELS.into_iter().enumerate() {
            // a level: two step beams, front and back, the tyres standing in a row in the gap between them
            for (k, z) in [z0 + 0.06, z0 + TR_D - 0.06].into_iter().enumerate() {
                fbox!(
                    F,
                    format!("beam{lv}{k}"),
                    1.8,
                    0.1,
                    0.05,
                    (0, y, z),
                    m.beam,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
            for t in 0..3 {
                fbox!(
                    F,
                    format!("tag{lv}{t}"),
                    0.08,
                    0.04,
                    0.003,
                    (-0.6 + t as f64 * 0.6, y, z0 + TR_D - 0.034),
                    m.label3
                );
            }
            let (r, w) = (0.31, 0.205);
            let n = 8;
            for t in 0..n {
                let x = -0.79 + t as f64 * (1.58 / (n - 1) as f64);
                let ty = y + 0.05 - (r - sqrt(r * r - pow(TR_D / 2.0 - 0.085, 2.0))) + r; // down between the beams
                // its open bead eased, sixteen round (eased, twenty went past the triangles' aim)
                artkit::bevel::eased(tyre(
                    &format!("tyre{lv}{t}"),
                    x,
                    ty,
                    zc,
                    r,
                    w,
                    m.rubber,
                    None,
                    Some(16),
                    Axis::X,
                ));
                // the tread's label, on its crown to the front
                fbox!(
                    F,
                    format!("label{lv}{t}"),
                    0.12,
                    0.16,
                    0.004,
                    (x, ty, zc + r + 0.001),
                    labels[(t + lv) % 3]
                );
            }
        }
    });
    join_all("tyreRack");
    settle!(centre = false);
    meshes()
}

// --- the tool chest: 1x1 against the wall, a red roller cabinet with a top box ------------------------------------
pub const TC_W: f64 = 0.68;
pub const TC_D: f64 = 0.46;

/// A drawer's front from height y0, `h` high, its face at depth z (`out` further, pulled out): the red front, the
/// aluminium pull along its top edge.
pub fn drawer_front(name: &str, y0: f64, h: f64, z: f64, m: &CHEST, out: f64) {
    fbox!(
        F,
        name,
        TC_W - 0.04,
        h - 0.006,
        0.018,
        (0, y0 + h / 2.0, z + out + 0.009),
        m.body,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        format!("{name}_pull"),
        TC_W - 0.08,
        0.014,
        0.022,
        (0, y0 + h - 0.018, z + out + 0.029),
        m.steel,
        bevel = 0.004,
        bevel_segments = 1
    );
}

/// A combination spanner lying flat at (x, y, z), along the depth turned `turn` degrees: a bar, an open jaw at one
/// end, a ring at the other.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn spanner(name: &str, x: f64, y: f64, z: f64, length: f64, m: &CHEST, turn: f64) -> Vec<Obj> {
    let hw = length * 0.05;
    let out = [
        (-length / 2.0 + hw * 1.6, -hw * 0.5),
        (length / 2.0 - hw * 1.6, -hw * 0.5),
        (length / 2.0 - hw * 1.6, hw * 0.5),
        (-length / 2.0 + hw * 1.6, hw * 0.5),
    ];
    let parts = new_parts(|| {
        fextrude!(
            F,
            format!("{name}_bar"),
            &out,
            0.006,
            (x, y, z),
            m.chrome,
            plane = Facing::Top
        );
        let ring: Vec<(f64, f64)> = (0..10)
            .map(|k| TAU * k as f64 / 10.0)
            .map(|a| {
                (
                    cos(a) * hw * 1.6 - length / 2.0 + hw * 1.6,
                    sin(a) * hw * 1.6,
                )
            })
            .collect();
        let hole: Vec<(f64, f64)> = (0..8)
            .map(|k| TAU * k as f64 / 8.0)
            .map(|a| {
                (
                    cos(a) * hw * 0.9 - length / 2.0 + hw * 1.6,
                    sin(a) * hw * 0.9,
                )
            })
            .collect();
        fextrude!(
            F,
            format!("{name}_ring"),
            &ring,
            0.008,
            (x, y, z),
            m.chrome,
            plane = Facing::Top,
            holes = Holes(vec![hole])
        );
        let jaw = [
            (length / 2.0 - hw * 2.2, -hw * 1.4),
            (length / 2.0, -hw * 1.6),
            (length / 2.0, -hw * 0.6),
            (length / 2.0 - hw * 1.2, -hw * 0.6),
            (length / 2.0 - hw * 1.2, hw * 0.6),
            (length / 2.0, hw * 0.6),
            (length / 2.0, hw * 1.6),
            (length / 2.0 - hw * 2.2, hw * 1.4),
        ];
        fextrude!(
            F,
            format!("{name}_jaw"),
            &jaw,
            0.008,
            (x, y, z),
            m.chrome,
            plane = Facing::Top
        );
    });
    for &p in &parts {
        p.set_rotation_euler_axis(2, p.rotation_euler().z as f64 + radians(turn));
    }
    parts
}

pub fn tool_chest() -> Vec<Obj> {
    let m = &CHEST;
    let z0 = WALL;
    let front = z0 + TC_D;
    let zc = z0 + TC_D / 2.0;
    new_parts(|| {
        // the roller cabinet: a carcass on four castors, its drawers, a push handle on its right side
        fbox!(
            F,
            "cabinet",
            TC_W,
            0.84,
            TC_D,
            (0, 0.13 + 0.42, zc),
            m.body,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            F,
            "cabinet_top",
            TC_W + 0.01,
            0.02,
            TC_D + 0.005,
            (0, 0.98, zc + 0.0025),
            m.rubber,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "gaps",
            TC_W - 0.03,
            0.78,
            0.002,
            (0, 0.55, front + 0.0005),
            m.dark
        );
        let mut y = 0.16;
        let mut open_at = (0.0, 0.0);
        for (k, h) in [0.2, 0.12, 0.12, 0.075, 0.075, 0.075, 0.075]
            .into_iter()
            .enumerate()
        {
            if k == 5 {
                open_at = (y, h);
            } else {
                drawer_front(&format!("drawer{k}"), y, h, front + 0.001, m, 0.0);
            }
            y += h;
        }
        // the third drawer pulled out: its sides and its foam liner of spanners showing
        let (y2, h2) = open_at;
        let out = 0.16;
        drawer_front("drawer_open", y2, h2, front + 0.001, m, out);
        for s in [-1, 1] {
            fbox!(
                F,
                format!("drawer_side{s}"),
                0.012,
                h2 - 0.02,
                out + 0.02,
                (
                    s as f64 * (TC_W / 2.0 - 0.035),
                    y2 + (h2 - 0.02) / 2.0 + 0.004,
                    front + out / 2.0 - 0.01
                ),
                m.steel
            );
        }
        fbox!(
            F,
            "drawer_foam",
            TC_W - 0.08,
            0.012,
            out + 0.02,
            (0, y2 + 0.04, front + out / 2.0 - 0.01),
            m.foam
        );
        for (k, (l, t)) in [
            (0.12, 0),
            (0.14, 0),
            (0.16, 0),
            (0.18, 0),
            (0.2, 0),
            (0.22, 0),
        ]
        .into_iter()
        .enumerate()
        {
            spanner(
                &format!("spanner{k}"),
                -0.23 + k as f64 * 0.09,
                y2 + 0.049,
                front + out / 2.0 - 0.02,
                l,
                m,
                (90 + t) as f64,
            );
        }
        // castors under it: a swivel plate, a fork and a wheel each, brakes on the front pair
        for sx in [-1, 1] {
            for sz in [-1, 1] {
                let (cx, cz) = (
                    sx as f64 * (TC_W / 2.0 - 0.06),
                    zc + sz as f64 * (TC_D / 2.0 - 0.07),
                );
                fbox!(
                    F,
                    format!("castor_plate{sx}{sz}"),
                    0.08,
                    0.008,
                    0.08,
                    (cx, 0.126, cz),
                    m.steel
                );
                fbox!(
                    F,
                    format!("castor_fork{sx}{sz}"),
                    0.05,
                    0.05,
                    0.06,
                    (cx, 0.1, cz - 0.01),
                    m.steel
                );
                fcyl!(
                    F,
                    format!("castor{sx}{sz}"),
                    0.045,
                    0.035,
                    (cx, 0.045, cz - 0.02),
                    m.plastic,
                    rot = (0, 90, 0),
                    verts = 14
                );
                if sz > 0 {
                    fbox!(
                        F,
                        format!("brake{sx}"),
                        0.03,
                        0.012,
                        0.04,
                        (cx, 0.11, cz + 0.045),
                        m.red_lens
                    );
                }
            }
        }
        kit_tube!(
            F,
            "handle",
            &[
                (-TC_W / 2.0, 0.86, zc - 0.15),
                (-TC_W / 2.0 - 0.05, 0.86, zc - 0.15),
                (-TC_W / 2.0 - 0.05, 0.86, zc + 0.15),
                (-TC_W / 2.0, 0.86, zc + 0.15)
            ],
            0.012,
            m.chrome,
            round = 2
        );
        // the top box: drawers under a lid, its back to the wall, a hasp and handles on its ends
        let tb = 0.34;
        let tz = z0 + tb / 2.0;
        fbox!(
            F,
            "topbox",
            TC_W,
            0.36,
            tb,
            (0, 0.99 + 0.18, tz),
            m.body,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            F,
            "top_gaps",
            TC_W - 0.03,
            0.28,
            0.002,
            (0, 1.13, z0 + tb + 0.0005),
            m.dark
        );
        let mut y = 0.995;
        for (k, h) in [0.06, 0.06, 0.06, 0.1].into_iter().enumerate() {
            drawer_front(&format!("top_drawer{k}"), y, h, z0 + tb + 0.001, m, 0.0);
            y += h;
        }
        fbox!(
            F,
            "lid",
            TC_W + 0.006,
            0.08,
            tb + 0.003,
            (0, 1.39, tz + 0.0015),
            m.body,
            bevel = 0.012,
            bevel_segments = 2
        );
        fbox!(
            F,
            "lid_trim",
            TC_W - 0.1,
            0.02,
            0.012,
            (0, 1.39, z0 + tb + 0.008),
            m.steel,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "hasp",
            0.04,
            0.05,
            0.012,
            (0, 1.36, z0 + tb + 0.01),
            m.chrome
        );
        for s in [-1, 1] {
            let sf = s as f64;
            kit_tube!(
                F,
                format!("end_handle{s}"),
                &[
                    (sf * TC_W / 2.0, 1.27, tz - 0.08),
                    (sf * (TC_W / 2.0 + 0.03), 1.27, tz - 0.08),
                    (sf * (TC_W / 2.0 + 0.03), 1.27, tz + 0.08),
                    (sf * TC_W / 2.0, 1.27, tz + 0.08)
                ],
                0.008,
                m.chrome,
                round = 2
            );
        }
        // on the cabinet's top in front of the box: a socket set's case and a spanner left out
        fbox!(
            F,
            "socket_case",
            0.3,
            0.05,
            0.1,
            (0.12, 1.015, front - 0.06),
            m.plastic,
            bevel = 0.01,
            bevel_segments = 1
        );
        spanner("loose", -0.18, 0.993, front - 0.06, 0.2, m, 20.0);
    });
    join_all("toolChest");
    settle!(centre = false);
    meshes()
}
