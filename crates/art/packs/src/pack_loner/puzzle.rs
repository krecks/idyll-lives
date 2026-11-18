//! The jigsaw puzzle table and the bonsai potting bench.

use super::common::{_chair, _desk, _m, _mug, BANDS, M, WALL};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, TAU, acos, cos, cosd, degrees, radians, sin, sind};
use artkit::parts::{hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{
    boulder, fbox, fcyl, fextrude, fsphere, fsweep, grow, jitter, rock, settle, soft, squash, torus,
};

// --- the jigsaw puzzle table (1x1): seated in the middle, a puzzle half done on the table before them ---------
const COLS: usize = 7;
const ROWS: usize = 4;
const PW: f64 = 0.087; // the puzzle's grid: piece pitch across
const PD: f64 = 0.067; // and deep
const PX0: f64 = -0.29; // the first piece's middle (back-left as the puzzler sees it)
const PZ0: f64 = 0.38;
/// The pieces `pieces` lays, in order.
const PATCH: [(usize, usize); 7] = [(4, 1), (4, 2), (5, 1), (5, 2), (5, 3), (6, 1), (6, 2)];
const PY: f64 = 0.729; // the pieces' middle height

/// A jigsaw piece's outline `w` x `d` (centred, game x and depth): a knob out of its right edge, the same shape taken
/// out of its left, so a row of pieces nests.
fn _jigsaw(w: f64, d: f64, r: f64) -> Vec<(f64, f64)> {
    let (hw, hd) = (w / 2.0, d / 2.0);
    let a0 = degrees(acos(-0.7));
    let knob = (0..9)
        .map(|k| -a0 + 2.0 * a0 * k as f64 / 8.0)
        .map(|a| (hw + 0.7 * r + r * cosd(a), r * sind(a)));
    let sock = (0..9)
        .map(|k| a0 - 2.0 * a0 * k as f64 / 8.0)
        .map(|a| (-hw + 0.7 * r + r * cosd(a), r * sind(a)));
    let mut out = vec![(-hw, -hd), (hw, -hd)];
    out.extend(knob);
    out.extend([(hw, hd), (-hw, hd)]);
    out.extend(sock);
    out
}

/// A piece of the puzzle: (i, j) on its grid, its band's colour, unless placed `at` and coloured otherwise.
#[allow(clippy::too_many_arguments)]
fn _piece(
    f: &GameFrame,
    i: usize,
    j: usize,
    y: f64,
    name: &str,
    turn: Option<f64>,
    at: Option<(f64, f64)>,
    colour: Option<Mat>,
    size: (f64, f64, f64),
) -> Obj {
    let m = colour.unwrap_or_else(|| {
        let band = BANDS[j];
        _m(if (i + j).is_multiple_of(2) {
            band.0
        } else {
            band.1
        })
    });
    let (x, z) = at.unwrap_or((PX0 + i as f64 * PW, PZ0 - j as f64 * PD));
    let o = fextrude!(
        f,
        numbered(name),
        &_jigsaw(size.0, size.1, size.2),
        0.008,
        (x, y + 0.0004 * ((i + j) % 2) as f64, z),
        m,
        plane = Facing::Top
    );
    if let Some(turn) = turn {
        o.set_rotation_euler_axis(2, radians(turn));
    }
    o
}

const PIECE: (f64, f64, f64) = (PW - 0.002, PD - 0.002, 0.011);

pub fn puzzle_table() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _desk(f, 0.1, 0.45, 0.86, 0.72, M.wood, M.body, 8);
        _chair(f, -0.06, M.wood, M.body, 6);
        fbox!(
            f,
            "puzzle_mat",
            0.68,
            0.004,
            0.32,
            (0.0, 0.722, 0.28),
            M.dark,
            bevel = 0.0015,
            bevel_segments = 1
        );
        for i in 0..COLS {
            for j in 0..ROWS {
                if !PATCH.contains(&(i, j)) && ![(0, 2), (2, 3)].contains(&(i, j)) {
                    _piece(f, i, j, PY, "piece", None, None, None, PIECE);
                }
            }
        }
        for (k, (x, z, r, m)) in [
            (0.38, 0.19, 20, "sky_a"),
            (0.34, 0.14, -35, "hill_b"),
            (0.4, 0.12, 60, "field_a"),
            (-0.37, 0.16, 10, "sea_a"),
            (-0.33, 0.13, -50, "field_b"),
        ]
        .into_iter()
        .enumerate()
        {
            _piece(
                f,
                k,
                0,
                0.726,
                "loose",
                Some(r as f64),
                Some((x, z)),
                Some(_m(m)),
                (0.056, 0.046, 0.008),
            );
        }
        // the box on the floor, its lid off beside it showing the picture
        fbox!(
            f,
            "puzzle_box",
            0.3,
            0.04,
            0.22,
            (0.05, 0.02, 0.32),
            M.glaze,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "box_inside",
            0.284,
            0.004,
            0.204,
            (0.05, 0.039, 0.32),
            M.dark
        );
        fbox!(
            f,
            "box_picture",
            0.26,
            0.004,
            0.18,
            (0.05, 0.0425, 0.32),
            M.field_b
        );
        fbox!(
            f,
            "box_sky",
            0.26,
            0.005,
            0.07,
            (0.05, 0.0425, 0.375),
            M.sky_a
        );
        fbox!(
            f,
            "box_sea",
            0.26,
            0.005,
            0.04,
            (0.05, 0.0425, 0.315),
            M.sea_a
        );
        fcyl!(
            f,
            "box_sun",
            0.02,
            0.006,
            (0.12, 0.043, 0.37),
            M.dial,
            verts = 10
        );
        // a magnifier lamp clamped at the back-left corner: its clamp, two jointed arms, the ring round its lens
        fbox!(
            f,
            "lamp_clamp",
            0.05,
            0.05,
            0.04,
            (-0.36, 0.71, 0.43),
            M.dark,
            bevel = 0.006,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "lamp_foot",
            0.02,
            0.03,
            (-0.36, 0.735, 0.4),
            M.dark,
            verts = 10
        );
        kit::tube!(
            f,
            "lamp_arm_lo",
            &[(-0.36, 0.745, 0.4), (-0.32, 1.0, 0.38)],
            0.008,
            M.chrome,
            verts = 6,
            round = 1
        );
        kit::tube!(
            f,
            "lamp_arm_hi",
            &[(-0.32, 1.0, 0.38), (-0.2, 0.97, 0.31)],
            0.007,
            M.chrome,
            verts = 6,
            round = 1
        );
        fcyl!(
            f,
            "lamp_joint",
            0.013,
            0.026,
            (-0.32, 1.0, 0.38),
            M.dark,
            rot = (0, 90, 0),
            verts = 8
        );
        torus!(
            "lamp_ring",
            0.07,
            0.014,
            at = f.at(-0.17, 0.95, 0.29),
            material = M.steel,
            rot = (-30, 0, 0),
            major_segments = 24,
            minor_segments = 6
        );
        fcyl!(
            f,
            "lamp_lens",
            0.062,
            0.006,
            (-0.17, 0.95, 0.29),
            M.lens,
            rot = (-30, 0, 0),
            verts = 24
        );
        _mug(f, 0.36, 0.72, 0.38, M.porcelain, 160.0);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The patch of pieces the puzzler lays, built in place and flat across, origin on its left edge: `pieces` grows it
    // across the gap.
    let patch = new_parts(|| {
        for (i, j) in PATCH {
            _piece(f, i, j, PY, "piece", None, None, None, PIECE);
        }
    });
    let x0 = PX0 + 4.0 * PW - PW / 2.0;
    let patch = join(&patch, "pieces", Some(f.at(x0, PY, PZ0 - 2.0 * PD)));
    patch.set_scale_axis(0, 0.001);
    let objs = settle!(centre = false);
    grow!(patch, "pieces", seconds = 0.9, axis = Axis::X);
    objs
}

// --- the bonsai potting bench (1x1): a teak bench against the wall, a bonsai in a glazed pot on it -------------
const TWIG: (f64, f64, f64) = (0.19, 1.16, -0.2); // where the leggy twig leaves the top-right pad

/// A cloud of foliage: a few jittered, flattened clumps around (x, y, z), the top one the fullest.
#[allow(clippy::too_many_arguments)]
fn _pad(f: &GameFrame, x: f64, y: f64, z: f64, w: f64, d: f64, seed: i32) {
    let mut rnd = Random::new(seed as i128);
    let s = seed as f64;
    let o = boulder!(
        numbered("pad"),
        0.5,
        at = f.at(x, y - 0.03, z),
        material = M.leaf,
        scale = (w, d, 0.07),
        seed = s * 1.3,
        noise = 0.25,
        subdivisions = 2
    );
    jitter!(o, 0.006, size = 0.04, seed = s);
    for k in 0..3 {
        let px = x + rnd.uniform(-w, w) * 0.3;
        let pz = z + rnd.uniform(-d, d) * 0.25;
        let o = boulder!(
            numbered("pad"),
            0.5,
            at = f.at(px, y - 0.01, pz),
            material = M.leaf,
            scale = (w * 0.55, d * 0.55, 0.06),
            seed = s * 2.1 + k as f64,
            noise = 0.3,
            subdivisions = 1
        );
        jitter!(o, 0.005, size = 0.03, seed = (seed + k) as f64);
    }
}

pub fn bonsai_bench() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // the bench: a slatted teak top and shelf on a sage frame, a back board with a rail
        for k in 0..5 {
            fbox!(
                f,
                numbered("bench_slat"),
                0.86,
                0.04,
                0.082,
                (0, 0.8, WALL + 0.045 + 0.0875 * k as f64),
                M.teak,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for k in 0..4 {
            fbox!(
                f,
                numbered("shelf_slat"),
                0.8,
                0.03,
                0.088,
                (0, 0.26, WALL + 0.055 + 0.0967 * k as f64),
                M.teak,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "bench_back",
            0.86,
            0.26,
            0.03,
            (0, 0.95, WALL + 0.015),
            M.body,
            bevel = 0.008,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "bench_back_rail",
            &[(-0.43, 1.095, WALL + 0.02), (0.43, 1.095, WALL + 0.02)],
            0.0,
            M.teak,
            profile = rounded_rect(0.03, 0.04, 0.012, 2),
            up = (0, 0, 1)
        );
        for x in [-0.405, 0.405] {
            for z in [WALL + 0.025, WALL + 0.415] {
                fbox!(
                    f,
                    numbered("leg"),
                    0.045,
                    0.78,
                    0.045,
                    (x, 0.39, z),
                    M.body,
                    bevel = 0.008,
                    bevel_segments = 1
                );
            }
            fbox!(
                f,
                numbered("side_rail"),
                0.03,
                0.05,
                0.36,
                (x, 0.7, WALL + 0.22),
                M.body,
                bevel = 0.006,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("shelf_rail"),
                0.03,
                0.04,
                0.36,
                (x, 0.23, WALL + 0.22),
                M.body,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "front_rail",
            0.78,
            0.05,
            0.03,
            (0, 0.7, WALL + 0.415),
            M.body,
            bevel = 0.006,
            bevel_segments = 1
        );
        // the bonsai: a glazed tray pot on feet, soil and moss, roots gripping it, a twisting tapered trunk with its
        // branches, pads of leaves
        kit::lathe!(
            f,
            "pot",
            0,
            0.82,
            -0.18,
            &[
                (0.17, 0.0),
                (0.19, 0.012),
                (0.19, 0.07),
                (0.2, 0.08),
                (0.18, 0.08),
                (0.175, 0.07),
            ],
            M.glaze,
            verts = 24,
            square = 7.0,
            sy = 0.68
        );
        for x in [-0.13, 0.13] {
            for z in [-0.25, -0.11] {
                fbox!(
                    f,
                    numbered("pot_foot"),
                    0.03,
                    0.012,
                    0.03,
                    (x, 0.826, z),
                    M.glaze,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
        }
        fbox!(f, "soil", 0.34, 0.01, 0.22, (0, 0.89, -0.18), M.soil);
        boulder!(
            "moss",
            0.5,
            at = f.at(0.1, 0.892, -0.15),
            material = M.leaf,
            scale = (0.12, 0.1, 0.025),
            seed = 4.0,
            noise = 0.3,
            subdivisions = 1
        );
        for (dx, dz) in [(-0.07, 0.02), (0.05, -0.03), (-0.02, -0.05), (0.04, 0.04)] {
            fsweep!(
                f,
                numbered("root"),
                &[
                    (-0.02, 0.93, -0.18),
                    (-0.02 + dx * 0.6, 0.905, -0.18 + dz * 0.6),
                    (-0.02 + dx, 0.895, -0.18 + dz)
                ],
                0.012,
                M.trunk,
                verts = 6,
                scale = [1.0, 0.6],
                resolution = 2
            );
        }
        let trunk = [
            (-0.02, 0.9, -0.18),
            (0.05, 0.99, -0.19),
            (-0.03, 1.08, -0.17),
            (0.02, 1.17, -0.18),
            (0.09, 1.22, -0.2),
        ];
        let t = fsweep!(
            f,
            "trunk",
            &trunk,
            0.034,
            M.trunk,
            verts = 10,
            resolution = 3,
            scale = [1.0, 0.75, 0.62, 0.5, 0.3]
        );
        jitter!(t, 0.003, size = 0.03, seed = 2.0);
        fsweep!(
            f,
            "branch_l",
            &[
                (0.05, 0.99, -0.19),
                (-0.05, 1.04, -0.16),
                (-0.15, 1.05, -0.15)
            ],
            0.013,
            M.trunk,
            verts = 6,
            resolution = 2,
            scale = [1.0, 0.4]
        );
        fsweep!(
            f,
            "branch_r",
            &[
                (-0.03, 1.08, -0.17),
                (0.07, 1.11, -0.19),
                (0.17, 1.12, -0.2)
            ],
            0.011,
            M.trunk,
            verts = 6,
            resolution = 2,
            scale = [1.0, 0.4]
        );
        _pad(f, -0.17, 1.08, -0.15, 0.2, 0.16, 1);
        _pad(f, 0.18, 1.14, -0.2, 0.18, 0.15, 2);
        _pad(f, 0.08, 1.26, -0.2, 0.22, 0.18, 3);
        _pad(f, -0.04, 1.19, -0.16, 0.14, 0.12, 4);
        // tools: a coil of copper wire, a mister, a pair of shears
        let coil: Vec<(f64, f64, f64)> = (0..37)
            .map(|k| {
                let a = TAU * k as f64 / 12.0;
                (
                    0.3 + 0.035 * cos(a),
                    0.825 + 0.002 * k as f64,
                    -0.05 + 0.035 * sin(a),
                )
            })
            .collect();
        fsweep!(f, "wire", &coil, 0.004, M.copper, verts = 4);
        kit::lathe!(
            f,
            "mister",
            -0.32,
            0.82,
            -0.06,
            &[
                (0.036, 0.0),
                (0.04, 0.008),
                (0.04, 0.11),
                (0.03, 0.125),
                (0.016, 0.135),
                (0.016, 0.165),
                (0.0, 0.165),
            ],
            M.porcelain,
            verts = 14
        );
        fbox!(
            f,
            "mister_head",
            0.04,
            0.03,
            0.05,
            (-0.32, 1.0, -0.055),
            M.dark,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "mister_nozzle",
            0.012,
            0.012,
            0.03,
            (-0.32, 1.005, -0.025),
            M.dark
        );
        fsweep!(
            f,
            "mister_trigger",
            &[
                (-0.32, 0.995, -0.035),
                (-0.32, 0.96, -0.03),
                (-0.32, 0.94, -0.04)
            ],
            0.0,
            M.dark,
            profile = rounded_rect(0.008, 0.016, 0.003, 1),
            up = (1, 0, 0),
            resolution = 2
        );
        for (k, a) in [25, -10].into_iter().enumerate() {
            let kf = k as f64;
            let blade = [
                (0.0, -0.06),
                (0.006, -0.02),
                (0.005, 0.05),
                (0.0, 0.06),
                (-0.004, 0.05),
                (-0.004, -0.02),
            ];
            fextrude!(
                f,
                numbered("shear_blade"),
                &blade,
                0.004,
                (0.27 + 0.02 * kf, 0.822 + 0.004 * kf, -0.34),
                M.steel,
                plane = Facing::Top,
                rot = (0, 0, a)
            );
            torus!(
                numbered("shear_loop"),
                0.016,
                0.005,
                at = f.at(0.25 + 0.04 * kf, 0.826, -0.27),
                material = M.terra,
                major_segments = 10,
                minor_segments = 4
            );
        }
        // spare terracotta pots on the shelf, a sack of soil on the floor with its top rolled
        for (x, h) in [(-0.25, 0.14), (0.0, 0.12), (0.24, 0.16)] {
            let r = h * 0.55;
            kit::turned!(
                f,
                numbered("spare_pot"),
                x,
                0.275,
                -0.2,
                &[
                    (0.0, 0.0),
                    (r * 0.72, 0.0),
                    (r * 0.9, h * 0.8),
                    (r, h * 0.8),
                    (r, h),
                    (r * 0.88, h),
                    (r * 0.62, 0.012),
                    (0.0, 0.012),
                ],
                M.terra,
                segments = 16
            );
        }
        soft!(
            f,
            "sack",
            0.24,
            0.27,
            0.12,
            (0.26, 0.135, 0.25),
            M.sack,
            round = 0.04,
            puff = 0.01,
            flat = 2
        );
        torus!(
            "sack_roll",
            0.11,
            0.022,
            at = f.at(0.26, 0.285, 0.25),
            material = M.sack,
            major_segments = 16,
            minor_segments = 5
        )
        .set_scale((1.05, 0.5, 1.0));
        fsphere!(
            f,
            "sack_soil",
            0.09,
            (0.26, 0.28, 0.25),
            M.soil,
            scale = (1.0, 0.55, 0.2),
            segments = 10,
            rings = 5
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The leggy twig the gardener trims: a shoot and a tuft, origin where it leaves the pad: `trim` shrinks it away.
    let twig = new_parts(|| {
        fsweep!(
            f,
            "twig",
            &[TWIG, (0.25, 1.21, -0.185), (0.31, 1.25, -0.17)],
            0.006,
            M.leaf,
            verts = 6,
            resolution = 2,
            scale = [1.0, 0.6]
        );
        for (k, (x, y, z, s)) in [(0.32, 1.26, -0.17, 0.08), (0.26, 1.215, -0.18, 0.05)]
            .into_iter()
            .enumerate()
        {
            let o = boulder!(
                numbered("twig_tuft"),
                0.5,
                at = f.at(x, y - s * 0.4, z),
                material = M.leaf,
                scale = (s, s * 0.85, s * 0.6),
                seed = 7.0 + k as f64,
                noise = 0.3,
                subdivisions = 1
            );
            jitter!(o, 0.004, size = 0.03, seed = k as f64);
        }
    });
    let twig = hinged(join(&twig, "twig", None), f.at(TWIG.0, TWIG.1, TWIG.2));
    let objs = settle!(centre = false);
    squash!(twig, "trim", 0.001, seconds = 0.7);
    rock!(twig, "mist", 10.0, seconds = 1.2, axis = Axis::Y); // nodding under the mister while it is misted and fed
    objs
}
