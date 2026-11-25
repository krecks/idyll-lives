//! KD2: the play tent.

use super::common::{Frame, P, add, mul, place, plane, slab, still};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{P3, PI, cos, radians, sin, sqrt};
use artkit::motion::{Squash, squash};
use artkit::parts::{hinged, join};
use artkit::py::imod;
use artkit::py::sum;
use artkit::scene::{Obj, meshes};
use artkit::{fcyl, fsweep, loft, soft, torus};

const TENT_R: f64 = 0.46;
const TENT_H: f64 = 1.3;
/// How far up the canvas runs (of the height); above it the poles cross bare, lashed together.
const CANVAS_TOP: f64 = 0.86;
/// The side between corners 1 and 2 faces the front (+depth).
const FRONT_SIDE: usize = 1;

/// The frame on face `k` of the six (0 the front), from the middle of its base edge up to the apex.
fn tent_face(k: i32) -> (Frame, f64) {
    let th = radians((90 + 60 * k) as f64);
    let a = TENT_R * cos(radians(30.0));
    let base = (a * cos(th), 0.0, a * sin(th));
    let up = (-base.0, TENT_H, -base.2);
    let length = sqrt(sum([up.0 * up.0, up.1 * up.1, up.2 * up.2]));
    (
        plane(base, (sin(th), 0.0, -cos(th)), mul(up, 1.0 / length)),
        length,
    )
}

const DOOR: [(f64, f64); 6] = [
    (-0.16, 0.0),
    (0.16, 0.0),
    (0.13, 0.35),
    (0.05, 0.56),
    (-0.05, 0.56),
    (-0.13, 0.35),
];

/// A point on the canvas: on side `i` (corner i to i + 1), `u` along it, `t` of the way up; the cloth between
/// the poles sags in a little (not the front, taut round its door), `out` metres off it.
fn canvas_point(i: usize, u: f64, t: f64, out: f64) -> P {
    let (a0, a1) = (radians((60 * i) as f64), radians((60 * (i + 1)) as f64));
    let x = TENT_R * ((1.0 - u) * cos(a0) + u * cos(a1));
    let z = TENT_R * ((1.0 - u) * sin(a0) + u * sin(a1));
    let sag = if i == FRONT_SIDE {
        0.0
    } else {
        let k = t / 0.7;
        0.035 * sin(PI * u) * (0.4 + 0.6 * sin(PI * if k < 1.0 { k } else { 1.0 }))
    };
    let k = (1.0 - t) * (1.0 - sag) + out / TENT_R;
    (x * k, TENT_H * t, z * k)
}

fn canvas_ring(t: f64, out: f64, n: usize) -> Vec<P3> {
    (0..6)
        .flat_map(|i| (0..n).map(move |j| (i, j)))
        .map(|(i, j)| {
            let p = canvas_point(i, j as f64 / n as f64, t, out);
            F.at(p.0, p.1, p.2)
        })
        .collect()
}

pub fn play_tent() -> Vec<Obj> {
    let canvas: Vec<Vec<P3>> = [0.0, 0.12, 0.26, 0.4, 0.55, 0.7, CANVAS_TOP]
        .iter()
        .map(|&t| canvas_ring(t, 0.0, 4))
        .collect();
    loft!("canvas", &canvas, material = M.canvas);
    // a hem round its foot
    let hem: Vec<Vec<P3>> = [0.0, 0.045]
        .iter()
        .map(|&t| canvas_ring(t, 0.004, 4))
        .collect();
    loft!("hem", &hem, material = M.flap, caps = false, smooth = false);
    let apex = (0.0, TENT_H, 0.0);
    let corners: Vec<P> = (0..6)
        .map(|i| {
            (
                TENT_R * cos(radians((60 * i) as f64)),
                0.0,
                TENT_R * sin(radians((60 * i) as f64)),
            )
        })
        .collect();
    for (i, &c) in corners.iter().enumerate() {
        let d = add(&[apex, mul(c, -1.0)]);
        let length = sqrt(sum([d.0 * d.0, d.1 * d.1, d.2 * d.2]));
        let foot = add(&[c, mul(d, -0.012 / length)]);
        let top = add(&[apex, mul(d, 0.28 / length)]);
        fsweep!(
            F,
            format!("pole{i}"),
            &[add(&[foot, (0.0, 0.01, 0.0)]), top],
            0.0125,
            M.poles,
            verts = 8,
            scale = [1.0, 0.8]
        );
    }
    // the rope lashing the poles where they cross
    for (k, y) in [1.135, 1.16, 1.185].into_iter().enumerate() {
        let r = TENT_R * (1.0 - y / TENT_H) + 0.014;
        torus!(
            format!("lashing{k}"),
            r,
            0.005,
            at = F.at(0, y, 0),
            material = M.canvas,
            major_segments = 12,
            minor_segments = 4
        );
    }
    // a round padded mat on the floor, two cushions on it
    kit::turned!(
        F,
        "floor_mat",
        0,
        0.0,
        0,
        &[
            (0.0, 0.0),
            (0.345, 0.0),
            (0.362, 0.008),
            (0.365, 0.02),
            (0.35, 0.03),
            (0.0, 0.031),
        ],
        M.tent_mat,
        segments = 24
    );
    soft!(
        F,
        "cushion0",
        0.3,
        0.12,
        0.09,
        (-0.11, 0.09, -0.2),
        M.flap,
        rot = (-12, 0, 18),
        round = 0.04,
        puff = 0.025,
        flat = 2
    );
    soft!(
        F,
        "cushion1",
        0.26,
        0.11,
        0.08,
        (0.14, 0.085, -0.18),
        M.tent_mat,
        rot = (-10, 0, -22),
        round = 0.035,
        puff = 0.022,
        flat = 2
    );
    let (front, _) = tent_face(0);
    slab("door", &DOOR, &front, M.tent_door, 0.002, 0.003);
    let roll_at = add(&[front.o, mul(front.u, 0.575), mul(front.n, 0.02)]);
    fcyl!(
        F,
        "roll",
        0.022,
        0.2,
        (0, 0.0, 0),
        M.flap,
        verts = 10,
        rot = (0, 90, 0)
    )
    .set_location(F.at(roll_at.0, roll_at.1, roll_at.2));
    // the ties the flap is rolled up with, hanging from the roll
    for s in [-1, 1] {
        let sf = s as f64;
        let p = add(&[roll_at, mul(front.s, sf * 0.06)]);
        fsweep!(
            F,
            format!("tie{s}"),
            &[
                add(&[p, mul(front.n, 0.022)]),
                add(&[p, mul(front.u, -0.06), mul(front.n, 0.026)]),
                add(&[
                    p,
                    mul(front.u, -0.12),
                    mul(front.s, sf * 0.01),
                    mul(front.n, 0.024)
                ])
            ],
            0,
            M.canvas,
            profile = rounded_rect(0.003, 0.016, 0.001, 1),
            up = (0, 0, 1),
            resolution = 2
        );
    }
    // bunting on a string round the front three faces
    let flags = [M.toy_red, M.toy_yellow, M.toy_blue, M.toy_green, M.toy_red];
    let tb = 0.7 / tent_face(0).1;
    let mut string: Vec<P> = [0, 1, 2]
        .iter()
        .flat_map(|&i| (0..4).map(move |j| canvas_point(i, j as f64 / 4.0, tb, 0.007)))
        .collect();
    string.push(canvas_point(3, 0.0, tb, 0.007));
    fsweep!(F, "bunting", &string, 0.002, M.canvas, verts = 4);
    for k in [-1, 0, 1] {
        let (frame, _) = tent_face(k);
        let js: Vec<i64> = if k == 0 { (-1..2).collect() } else { vec![0] };
        for j in js {
            let m = flags[((k + 1) * 2) as usize + imod(j, 2) as usize];
            let on = plane(
                add(&[frame.o, mul(frame.u, 0.7), mul(frame.s, 0.075 * j as f64)]),
                frame.s,
                frame.u,
            );
            slab(
                &format!("flag{k}{j}"),
                &[(-0.035, 0.0), (0.035, 0.0), (0.0, -0.06)],
                &on,
                m,
                0.003,
                0.003,
            );
        }
    }
    still("body", None);
    // the door flap: rolled up for play (squashed to its top edge along the canvas)
    let door: Vec<(f64, f64)> = DOOR.iter().map(|&(x, y)| (x * 1.12, y * 1.02)).collect();
    let flap = slab("flap", &door, &front, M.flap, 0.006, 0.006);
    let top = add(&[front.o, mul(front.u, 0.575), mul(front.n, 0.009)]);
    let flap = hinged(join(&[flap], "flap", None), F.at(top.0, top.1, top.2));
    place(0.0);
    squash(
        flap,
        "flap_open",
        (1, 0.1, 0.1),
        Squash {
            seconds: 0.8,
            ..Squash::default()
        },
    );
    meshes()
}
