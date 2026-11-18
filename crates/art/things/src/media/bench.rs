//! The low TV bench (2x1): a white bench on oak legs, a TV lift in a hood at its back.

use super::common::WALL;
use super::common::call::rbox;
use super::{F, M};
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, TAU, cos, pow, sin};
use artkit::parts::{join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, fsweep, revolve, settle, slide, soft};

const HOOD_W: f64 = 1.3;
const HOOD_Z1: f64 = -0.31;
const HOOD_TOP: f64 = 0.8;
/// The carcass's depth, its back on the wall face.
const BENCH_D: f64 = 0.43;
/// The carcass's middle depth: its legs splay away from it.
const BENCH_Z: f64 = WALL + BENCH_D / 2.0;
/// How far a leg's foot stands out from its top.
const SPLAY: f64 = 0.006;
const LIFT: f64 = 0.6;

/// `_plant`: a pot of leaves: a rimmed pot `r` round and `h` high on `y`, leaves arching out of it.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn plant(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    pot: Mat,
    leaf: Mat,
    n: usize,
    length: f64,
) {
    revolve!(
        format!("{name}_pot"),
        &[
            (0.0, 0.0),
            (r * 0.78, 0.0),
            (r * 0.95, h * 0.85),
            (r * 1.06, h * 0.87),
            (r * 1.06, h),
            (r * 0.92, h),
            (r * 0.9, h * 0.9),
            (0.0, h * 0.9)
        ],
        at = F.at(x, y, z),
        material = pot,
        segments = 18
    );
    for k in 0..n {
        let kf = k as f64;
        let a = TAU * kf / n as f64 + 0.35 * sin(kf * 2.3);
        let (c, s) = (cos(a), sin(a));
        let l = length * (0.75 + 0.25 * pow(sin(kf * 1.7), 2.0));
        let lift = 0.8 + 0.2 * cos(kf * 1.3);
        let y0 = y + h * 0.9;
        let pts = [
            (x + c * 0.01, y0, z + s * 0.01),
            (x + c * l * 0.3, y0 + l * lift * 0.75, z + s * l * 0.3),
            (x + c * l * 0.6, y0 + l * lift, z + s * l * 0.6),
            (x + c * l * 0.9, y0 + l * lift * 0.8, z + s * l * 0.9),
        ];
        fsweep!(
            F,
            format!("{name}_leaf{k}"),
            &pts,
            0,
            leaf,
            profile = [(0.0, -0.018), (0.003, 0.0), (0.0, 0.018), (-0.0015, 0.0)],
            up = (0, 0, 1),
            resolution = 2,
            scale = [0.4, 1.0, 0.8, 0.15]
        );
    }
}

pub fn tv_stand_low() -> Vec<Obj> {
    let still = new_parts(|| {
        for (i, x) in [-0.84, 0.84].into_iter().enumerate() {
            for (k, z) in [-0.4, -0.04].into_iter().enumerate() {
                // each leg's foot stands out from its top, further out sideways and in depth
                fsweep!(
                    F,
                    format!("leg{i}{k}"),
                    &[
                        (x, 0.06, z),
                        (x + SPLAY.copysign(x), 0.0, z + SPLAY.copysign(z - BENCH_Z))
                    ],
                    0.02,
                    M.bench_oak,
                    verts = 10,
                    scale = [1.0, 0.75]
                );
            }
        }
        soft!(
            F,
            "carcass",
            1.8,
            0.4,
            BENCH_D,
            (0, 0.26, BENCH_Z),
            M.bench,
            round = 0.01,
            round_segments = 2
        );
        fbox!(F, "reveal", 1.78, 0.35, 0.002, (0, 0.26, 0.0005), M.grille);
        // the top: in front of the hood and either side of it
        soft!(
            F,
            "top",
            1.82,
            0.03,
            0.34,
            (0, 0.475, (HOOD_Z1 + 0.03) / 2.0),
            M.bench_oak,
            round = 0.008,
            round_segments = 2
        );
        let side = (1.82 - HOOD_W) / 2.0;
        for s in [-1, 1] {
            soft!(
                F,
                format!("top_back{s}"),
                side,
                0.03,
                HOOD_Z1 - WALL,
                (
                    s as f64 * (HOOD_W + side) / 2.0,
                    0.475,
                    (WALL + HOOD_Z1) / 2.0
                ),
                M.bench_oak,
                round = 0.006,
                round_segments = 2
            );
        }
        // the hood the TV sinks into: four walls, the TV's lid closing its top
        let h = HOOD_TOP - 0.46;
        let yc = 0.46 + h / 2.0;
        rbox!(
            "hood_back",
            HOOD_W,
            h,
            0.012,
            (0, yc, WALL + 0.006),
            M.bench,
            r = 0.003
        );
        rbox!(
            "hood_front",
            HOOD_W,
            h,
            0.012,
            (0, yc, HOOD_Z1 - 0.006),
            M.bench,
            r = 0.003
        );
        for s in [-1, 1] {
            rbox!(
                format!("hood_side{s}"),
                0.012,
                h,
                HOOD_Z1 - WALL,
                (
                    s as f64 * (HOOD_W / 2.0 - 0.006),
                    yc,
                    (WALL + HOOD_Z1) / 2.0
                ),
                M.bench,
                r = 0.003
            );
        }
        // three doors, each framed round a panel, with black bar pulls
        for x in [-0.6, 0.0, 0.6] {
            kit::shaker!(
                F,
                &format!("door{}", float_repr(x)),
                0.58,
                0.34,
                (x, 0.26, 0.01),
                0.02,
                M.bench,
                rail = 0.05,
                recess = 0.004
            );
            kit::pull!(
                F,
                format!("pull{}", float_repr(x)),
                x,
                0.4,
                0.02,
                0.12,
                M.black_metal,
                r = 0.005,
                out = 0.009
            );
        }
        // a plant on the left, a remote and a game controller on the right
        plant(
            "plant", -0.78, 0.49, -0.1, 0.06, 0.1, M.pot, M.plant, 10, 0.13,
        );
        rbox!(
            "remote",
            0.04,
            0.015,
            0.16,
            (0.72, 0.4975, -0.1),
            M.bench_tv,
            r = 0.006,
            segs = 2,
            rot = (0, 0, -12)
        );
        for s in [-1, 1] {
            fsphere!(
                F,
                format!("pad_grip{s}"),
                0.03,
                (0.5 + s as f64 * 0.045, 0.505, -0.14),
                M.bench_tv,
                scale = (1, 1.2, 0.6),
                segments = 10,
                rings = 6
            );
        }
        rbox!(
            "pad",
            0.1,
            0.022,
            0.06,
            (0.5, 0.501, -0.15),
            M.bench_tv,
            r = 0.01,
            segs = 2
        );
        for k in 0..4 {
            fcyl!(
                F,
                format!("pad_button{k}"),
                0.004,
                0.004,
                (
                    0.52 + 0.009 * (k % 2) as f64,
                    0.512,
                    -0.155 + 0.009 * (k / 2) as f64
                ),
                [M.led, M.led_red, M.steel, M.steel][k],
                verts = 8
            );
        }
    });
    join(&still, "tvStandLow", Some(P3::ZERO));
    // the TV, down in the bench at rest, its lid flush with the hood's top
    let zc = (WALL + HOOD_Z1) / 2.0;
    let tv = new_parts(|| {
        let (th, y0) = (0.7, HOOD_TOP - 0.02 - 0.7);
        soft!(
            F,
            "tv_panel",
            1.22,
            th,
            0.035,
            (0, y0 + th / 2.0, zc),
            M.bench_tv,
            round = 0.006,
            round_segments = 2
        );
        fbox!(
            F,
            "tv_screen",
            1.19,
            th - 0.03,
            0.002,
            (0, y0 + th / 2.0 + 0.005, zc + 0.018),
            M.screen
        );
        fbox!(
            F,
            "tv_logo",
            0.05,
            0.006,
            0.001,
            (0, y0 + 0.008, zc + 0.018),
            M.steel
        );
        rbox!(
            "tv_lid",
            HOOD_W - 0.03,
            0.02,
            HOOD_Z1 - WALL - 0.026,
            (0, HOOD_TOP - 0.01, zc),
            M.bench,
            r = 0.004,
            segs = 2
        );
        fbox!(
            F,
            "tv_lid_grip",
            0.2,
            0.002,
            0.03,
            (0, HOOD_TOP + 0.0005, HOOD_Z1 - 0.03),
            M.bench_tv
        );
    });
    let tv = join(&tv, "tv", None);
    settle!(centre = false);
    slide!(tv, "tv_up", (0, 0, LIFT), seconds = 1.6);
    meshes()
}
