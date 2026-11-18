//! K8 continued: the chairs whose seat is a solidified surface: the moulded chair's shell and the Windsor chair's
//! saddle seat.

use super::common::{finish, rail, stretcher, surface};
use super::{F, M, SZ};
use artkit::forms::catmull;
use artkit::math::{PI, V3, exp, pow, sin, sqrt};
use artkit::scene::Obj;
use artkit::{kit, py};

/// Moulded chair (catalog2): a gloss shell in one piece, its seat rolling over at the front and curving up into a
/// back that wraps round the sitter, on a black sled frame of two bent tubes.
pub fn chair_modern() -> Vec<Obj> {
    // the shell's side section (height, depth, half width, how far its sides wrap forward), front lip to top of back
    let keys = [
        (0.434, SZ + 0.225, 0.2, 0.0),
        (0.458, SZ + 0.209, 0.214, 0.0),
        (0.464, SZ + 0.15, 0.218, 0.0),
        (0.461, SZ + 0.02, 0.215, 0.0),
        (0.466, SZ - 0.11, 0.21, 0.012),
        (0.5, SZ - 0.175, 0.205, 0.035),
        (0.58, SZ - 0.2, 0.208, 0.055),
        (0.69, SZ - 0.228, 0.214, 0.065),
        (0.787, SZ - 0.262, 0.208, 0.06),
    ];
    // (the script's four-number vectors curved as one: the curve is worked out a number at a time, so the first
    // three and the fourth go through it apart)
    let (abc, _) = catmull(&keys.map(|k| V3::from((k.0, k.1, k.2))), 2, false);
    let (wraps, _) = catmull(&keys.map(|k| V3::from((k.3, 0.0, 0.0))), 2, false);
    let mut grid = Vec::new();
    for (p, q) in abc.iter().zip(&wraps) {
        let (y, zz, hw, wrap) = (p.x as f64, p.y as f64, p.z as f64, q.x as f64);
        grid.push(
            (0..13)
                .map(|i| {
                    let u = f64::from(i) / 6.0 - 1.0;
                    (u * hw, y, zz + wrap * u * u)
                })
                .collect::<Vec<_>>(),
        );
    }
    surface("shell", &grid, 0.012, M.shell, 60.0);
    // the sleds: each side one tube bent round, a runner on the floor, the legs and a rail under the seat; a cross
    // rail between them and two mounting pads
    for s in [-1, 1] {
        let x = f64::from(s) * 0.19;
        rail(
            &format!("sled{s}"),
            &[
                (x, 0.011, SZ - 0.17),
                (x, 0.011, SZ + 0.17),
                (x, 0.444, SZ + 0.155),
                (x, 0.444, SZ - 0.13),
            ],
            0.011,
            M.legs_black,
            0.035,
            8,
            3,
            true,
        );
    }
    kit::tube!(
        F,
        "cross",
        &[(-0.19, 0.444, SZ), (0.19, 0.444, SZ)],
        0.009,
        M.legs_black,
        verts = 8
    );
    finish("chairModern")
}

/// Windsor chair (catalog2): a carved saddle seat on splayed baluster-turned legs and a turned H-stretcher, a bow
/// bent from one tapering length over seven spindles.
pub fn chair_windsor() -> Vec<Obj> {
    // the saddle: dished, the rim highest at the back and sides, a pommel at the front
    let n = 12;
    let mut grid = Vec::new();
    for i in 0..=n {
        let v = f64::from(i) / f64::from(n) * 2.0 - 1.0;
        let mut row = Vec::new();
        for j in 0..=n {
            let u = f64::from(j) / f64::from(n) * 2.0 - 1.0;
            let (px, pz) = (
                u * sqrt(py::max(0.0, 1.0 - v * v / 2.0)),
                v * sqrt(py::max(0.0, 1.0 - u * u / 2.0)),
            );
            let r2 = px * px + pz * pz;
            let pommel = 0.004 * exp(-pow(px / 0.15, 2.0)) * py::max(0.0, pz); // never above the rim
            let dish = py::max(
                0.0,
                0.012 * (1.0 - r2) * (1.0 - 0.6 * pow(py::max(0.0, pz), 2.0)) - pommel,
            );
            row.push((
                px * 0.235,
                0.4475 - dish,
                SZ + 0.002 + pz * 0.225 - 0.02 * (1.0 - px * px) * py::max(0.0, -pz),
            ));
        }
        grid.push(row);
    }
    surface("seat", &grid, 0.045, M.seat_windsor, 50.0);
    let turn = [0.8, 1.0, 1.3, 0.85, 0.95, 0.75];
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            let (foot, top) = (
                (f64::from(sx) * 0.22, 0.0, SZ + f64::from(sz) * 0.21),
                (f64::from(sx) * 0.15, 0.43, SZ + f64::from(sz) * 0.13),
            );
            let mut pts: Vec<(f64, f64, f64)> = [0.0, 0.2, 0.42, 0.62, 0.82, 1.0]
                .iter()
                .map(|&w| {
                    (
                        foot.0 + (top.0 - foot.0) * w,
                        foot.1 + (top.1 - foot.1) * w,
                        foot.2 + (top.2 - foot.2) * w,
                    )
                })
                .collect();
            pts[0] = (foot.0, 0.0, foot.2);
            kit::tube!(
                F,
                format!("leg{sx}{sz}"),
                &pts,
                0.018,
                M.frame_windsor,
                verts = 10,
                round = 2,
                scale = turn
            );
        }
    }
    // the H-stretcher, where the splayed legs pass 16 cm up
    let y = 0.16;
    let (lx, lz) = (0.22 - 0.07 * y / 0.43, 0.21 - 0.08 * y / 0.43);
    for sx in [-1, 1] {
        let x = f64::from(sx) * lx;
        stretcher(
            &format!("side{sx}"),
            (x, y, SZ + lz),
            (x, y, SZ - lz),
            0.011,
            M.frame_windsor,
            1.4,
            8,
        );
    }
    stretcher(
        "cross",
        (-lx, y, SZ),
        (lx, y, SZ),
        0.011,
        M.frame_windsor,
        1.4,
        8,
    );
    // the bow: a half ellipse leaning back, 0.21 across and up to 0.97, bent from one length tapering to its top
    let (y0, ry, z0, back, rx) = (0.455, 0.5, SZ - 0.13, 0.08, 0.175);
    let arc: Vec<(f64, f64, f64)> = (0..17)
        .map(|i| {
            let t = PI * f64::from(i) / 16.0;
            let yy = y0 + ry * sin(t);
            (rx * artkit::math::cos(t), yy, z0 - back * (yy - y0) / ry)
        })
        .collect();
    let mut pts = vec![(arc[0].0, 0.43, z0 + 0.005)];
    pts.extend(&arc);
    pts.push((arc[arc.len() - 1].0, 0.43, z0 + 0.005));
    let mut scale = vec![1.05];
    scale.extend((0..17).map(|i| 1.0 - 0.18 * sin(PI * f64::from(i) / 16.0)));
    scale.push(1.05);
    kit::tube!(
        F,
        "bow",
        &pts,
        0.016,
        M.frame_windsor,
        verts = 8,
        round = 2,
        scale = scale
    );
    for (k, sx) in [-0.12, -0.08, -0.04, 0.0, 0.04, 0.08, 0.12]
        .into_iter()
        .enumerate()
    {
        let yy = y0 + ry * sqrt(py::max(0.0, 1.0 - pow(sx / rx, 2.0)));
        kit::tube!(
            F,
            format!("spindle{k}"),
            &[
                (sx * 0.9, 0.455, SZ - 0.115),
                (sx, yy - 0.005, z0 - back * (yy - y0) / ry),
            ],
            0.008,
            M.frame_windsor,
            verts = 6,
            scale = [1.15, 0.75]
        );
    }
    finish("chairWindsor")
}
