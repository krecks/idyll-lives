//! The whirlpool tub (2x2): an acrylic tub in a slatted cedar surround, two bathers on its bench.

use super::common::M;
use artkit::kit;
use artkit::math::{P3, cos, radians, sin};
use artkit::parts::{Settle, join, new_parts, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, grow, leaf, loft, revolve, ring, soft, squash};

const RIM: f64 = 0.72; // the rim's top
const FLOOR: f64 = 0.2; // the footwell's floor top
const WATER: f64 = 0.65; // the water's top when full
const IN: f64 = 0.7; // the basin's half width inside
const OUT: f64 = 0.9; // the cladding's face

/// The acrylic shell in one piece (a loft through rounded squares, as the kitchen sinks' bowls): under the rim's
/// edge, over its broad flat top, rolling over into the basin and down its walls to the footwell's floor.
fn _shell(f: &GameFrame) {
    let ring = |r: f64, y: f64, sq: i32| -> Vec<P3> { ring!(r, r, z = y, verts = 32, square = sq) };
    loft!(
        "shell",
        &[
            ring(0.9, RIM - 0.075, 12),
            ring(0.93, RIM - 0.06, 12),
            ring(0.93, RIM - 0.02, 12),
            ring(0.918, RIM - 0.002, 12),
            ring(0.9, RIM, 12),
            ring(IN + 0.03, RIM, 7),
            ring(IN + 0.004, RIM - 0.01, 7),
            ring(IN, RIM - 0.04, 7),
            ring(IN - 0.004, FLOOR + 0.06, 7),
            ring(IN - 0.03, FLOOR + 0.006, 7),
            ring(IN - 0.08, FLOOR, 7),
            ring(0.0, FLOOR, 7)
        ],
        material = M.shell
    );
    // the moulded bench along the back and its two back rests, leaning up the back wall
    soft!(
        f,
        "bench",
        2.0 * IN - 0.06,
        0.26,
        0.74,
        (0, FLOOR + 0.12, -IN + 0.37),
        M.shell,
        round = 0.04
    );
    for x in [-0.5, 0.5] {
        soft!(
            f,
            format!("backrest{x}"),
            0.5,
            0.3,
            0.07,
            (x, 0.58, -IN + 0.045),
            M.shell,
            rot = (12, 0, 0),
            round = 0.03,
            puff = 0.012,
            flat = 2,
            round_segments = 1
        );
    }
}

/// A jet: a chrome ring round a dark nozzle, set in the shell.
fn _jet(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, rot: (i32, i32, i32)) {
    fcyl!(
        f,
        name,
        0.024,
        0.01,
        (x, y, z),
        M.chrome,
        rot = rot,
        verts = 12,
        bevel = 0.003,
        bevel_segments = 1
    );
    fcyl!(
        f,
        format!("{name}_nozzle"),
        0.011,
        0.012,
        (x, y, z),
        M.plinth,
        rot = rot,
        verts = 8
    );
}

/// The cedar surround: on each side tongue-and-groove boards, one slab extruded upright from an outline grooved
/// between the boards; posts at the corners, a foot rail round the bottom, a plinth under it.
fn _cladding(f: &GameFrame) {
    let (n, span, t) = (11, 1.74, 0.02);
    let mut pts = vec![(-span / 2.0, OUT - t)];
    for k in 0..n {
        // along the face: each board, a V-groove after it
        let x0 = -span / 2.0 + k as f64 * span / n as f64;
        pts.extend([
            (x0 + 0.006, OUT),
            (x0 + span / n as f64 - 0.006, OUT),
            (x0 + span / n as f64, OUT - 0.01),
        ]);
    }
    let last = pts.len() - 1;
    pts[last] = (span / 2.0, OUT - t);
    let face = pts;
    for (i, rot) in [0, 90, 180, 270].into_iter().enumerate() {
        let (c, s) = (cos(radians(rot as f64)), sin(radians(rot as f64)));
        let outline: Vec<(f64, f64)> = face
            .iter()
            .rev()
            .map(|&(x, z)| (x * c - z * s, x * s + z * c))
            .collect();
        fextrude!(
            f,
            format!("cladding{i}"),
            &outline,
            0.6,
            (0, 0.35, 0),
            M.wood,
            plane = Facing::Top
        );
    }
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            fbox!(
                f,
                format!("post{sx}{sz}"),
                0.08,
                0.6,
                0.08,
                (sx as f64 * (OUT - 0.03), 0.35, sz as f64 * (OUT - 0.03)),
                M.wood_dark,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
    }
    kit::moulding!(
        f,
        "foot_rail",
        &[(-0.92, -0.92), (0.92, -0.92), (0.92, 0.92), (-0.92, 0.92)],
        &[
            (0.0, 0.0),
            (0.03, 0.0),
            (0.03, 0.04),
            (0.012, 0.055),
            (0.0, 0.055),
        ],
        M.wood_dark,
        at = 0.05,
        plane = Facing::Top
    );
    fbox!(f, "plinth", 1.78, 0.05, 1.78, (0, 0.025, 0), M.plinth);
}

/// Headrests on the back rim, the control panel, towels, candles and a plant on the rim's corners.
fn _tub_things(f: &GameFrame) {
    for x in [-0.5, 0.5] {
        soft!(
            f,
            format!("headrest{x}"),
            0.36,
            0.08,
            0.17,
            (x, RIM + 0.035, -0.79),
            M.pillow,
            rot = (-14, 0, 0),
            round = 0.035,
            puff = 0.012,
            flat = 2
        );
    }
    fbox!(
        f,
        "panel",
        0.12,
        0.012,
        0.16,
        (0.815, RIM + 0.004, -0.25),
        M.plinth,
        bevel = 0.004,
        bevel_segments = 1
    );
    for k in 0..3 {
        fcyl!(
            f,
            format!("button{k}"),
            0.013,
            0.008,
            (0.815, RIM + 0.012, -0.3 + 0.05 * k as f64),
            M.chrome,
            verts = 10
        );
    }
    soft!(
        f,
        "towel_a",
        0.4,
        0.07,
        0.19,
        (-0.6, RIM + 0.035, 0.815),
        M.towel,
        round = 0.03,
        puff = 0.008,
        flat = 2
    );
    soft!(
        f,
        "towel_b",
        0.38,
        0.06,
        0.18,
        (-0.6, RIM + 0.1, 0.815),
        M.towel2,
        round = 0.026,
        puff = 0.008,
        flat = 2
    );
    for (x, z, h) in [(0.78, 0.8, 0.12), (0.64, 0.84, 0.08), (0.8, 0.66, 0.06)] {
        // pillar candles, their tops dished
        revolve!(
            format!("candle{x}"),
            &[
                (0.0, 0.0),
                (0.035, 0.0),
                (0.035, h),
                (0.03, h - 0.004),
                (0.0, h - 0.008)
            ],
            at = f.at(x, RIM, z),
            material = M.candle,
            segments = 14
        );
        fcyl!(
            f,
            format!("wick{x}"),
            0.002,
            0.014,
            (x, RIM + h, z),
            M.plinth,
            verts = 4
        );
    }
    let (px, pz) = (-0.81, -0.81); // a little potted plant: leaves arching from a clay pot
    revolve!(
        "plant_pot",
        &[
            (0.0, 0.0),
            (0.055, 0.0),
            (0.07, 0.1),
            (0.078, 0.104),
            (0.078, 0.115),
            (0.066, 0.115),
            (0.062, 0.1),
            (0.0, 0.1)
        ],
        at = f.at(px, RIM, pz),
        material = M.terracotta,
        segments = 14
    );
    let base = f.at(px, RIM + 0.1, pz);
    for k in 0..7 {
        leaf!(
            format!("leaf{k}"),
            base,
            k as f64 * 360.0 / 7.0 + 10.0,
            (55 + 12 * (k % 2)) as f64,
            0.15,
            0.05,
            M.leaf,
            shape = "lance",
            droop = 0.3,
            rows = 4,
            ring = 4
        );
    }
}

pub fn whirlpool_tub() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _shell(f);
        for x in [-0.5, 0.5] {
            // jets: in the back rests, the walls and the footwell
            for dx in [-0.12, 0.12] {
                _jet(
                    f,
                    &format!("jet_back{x}{dx}"),
                    x + dx,
                    0.56,
                    -IN + 0.082,
                    (78, 0, 0),
                );
            }
            _jet(f, &format!("jet_front{x}"), x, 0.34, IN - 0.006, (90, 0, 0));
        }
        for s in [-1, 1] {
            for z in [-0.35, 0.25] {
                _jet(
                    f,
                    &format!("jet_side{s}{z}"),
                    s as f64 * (IN - 0.006),
                    0.52,
                    z,
                    (0, 90, 0),
                );
            }
        }
        _cladding(f);
        _tub_things(f);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The water: the basin's inside up to WATER, its origin just under the footwell's floor; scaled flat there at
    // rest, `fill` raises it while residents soak.
    let water = new_parts(|| {
        loft!(
            "water",
            &[
                ring!(
                    IN - 0.004,
                    IN - 0.004,
                    z = FLOOR - 0.01,
                    verts = 32,
                    square = 7
                ),
                ring!(IN - 0.004, IN - 0.004, z = WATER, verts = 32, square = 7)
            ],
            material = M.water
        );
    });
    let water = join(&water, "water", Some(f.at(0, FLOOR - 0.01, 0)));
    water.set_scale_axis(2, 0.001);
    // Bubbles welling up at the surface over the jets: domes just above and below it, joined with their origin
    // under the floor and scaled flat there; `bubbles` swells them up through the water and back down, over and
    // over while the soak lasts. Its rise takes as long as `fill`, from the same origin, so on the first round the
    // bubbles ride up on the rising water.
    let foam = new_parts(|| {
        let mut rnd = Random::new(4);
        for (cx, cz, n) in [
            (-0.5, 0.3, 7),
            (0.5, 0.3, 7),
            (-0.5, -0.1, 5),
            (0.5, -0.1, 5),
            (0.0, 0.5, 5),
            (-0.62, 0.55, 3),
            (0.62, 0.55, 3),
        ] {
            for i in 0..n {
                let r = rnd.uniform(0.018, 0.042);
                let at = (
                    cx + rnd.uniform(-0.12, 0.12),
                    WATER + rnd.uniform(-0.012, 0.004),
                    cz + rnd.uniform(-0.1, 0.1),
                );
                fsphere!(
                    f,
                    format!("bubble{cx}{cz}{i}"),
                    r,
                    at,
                    M.foam,
                    scale = (1, 1, 0.7),
                    segments = 7,
                    rings = 4
                );
            }
        }
    });
    let foam = join(&foam, "bubbles", Some(f.at(0, FLOOR - 0.01, 0)));
    foam.set_scale_axis(2, 0.001);
    let objs = settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    grow!(water, "fill", seconds = 0.8);
    squash!(foam, "bubbles", (1, 1, 1000), seconds = 1.6, back = true);
    objs
}
