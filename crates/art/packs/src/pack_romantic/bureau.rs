//! The love-letter bureau and the rose-arbour swing.

use super::common::{_candle, _rose, _velvet_chair, M, WALL};
use artkit::forms::{rect, rounded_rect};
use artkit::kit;
use artkit::math::{P3, PI, cos, cosd, pow, sin, sind};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsweep, leaf, revolve, rock, settle, soft, squash, swing};

// --- the love-letter bureau (1x1): its back on the wall, the desk and its pigeonholes there, the writer's chair in front ---
const DESK_D: f64 = 0.45; // the desk's depth from the wall face
const CHAIR_Z: f64 = 0.2; // the chair's middle depth (the hips at 0.25: `furniture.json`)

pub fn letter_bureau() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    // Its back on the wall, the pigeonhole hutch on the desk's back facing the writer and the room, the chair in
    // front, the writer facing the wall (`furniture.json` `back`; 2026-10-21: it stood the other way round, the
    // hutch's panelled back and the writer's back to the room).
    let hz = WALL + 0.012 + 0.05; // the hutch's middle depth (0.1 deep)
    let hf = hz + 0.05; // its front
    let still = new_parts(|| {
        fbox!(
            f,
            "desk_top",
            0.84,
            0.03,
            DESK_D,
            (0, 0.745, WALL + DESK_D / 2.0),
            M.ivory,
            bevel = 0.01,
            bevel_segments = 2
        );
        let mut apron = vec![(0.4, 0.1), (-0.4, 0.1)];
        apron.extend((0..17).map(|i| {
            let x = -0.4 + 0.8 * i as f64 / 16.0;
            (x, 0.02 + 0.018 * pow(cos(PI * x / 0.4), 2.0))
        }));
        fextrude!(
            f,
            "apron",
            &apron,
            DESK_D - 0.03,
            (0, 0.63, WALL + (DESK_D - 0.03) / 2.0),
            M.ivory
        );
        fbox!(
            f,
            "drawer",
            0.3,
            0.06,
            0.012,
            (0, 0.69, WALL + DESK_D - 0.023),
            M.blush,
            bevel = 0.004,
            bevel_segments = 1
        );
        kit::knob!(
            f,
            "drawer_knob",
            0,
            0.69,
            WALL + DESK_D - 0.017,
            0.01,
            M.gold,
            segments = 8
        );
        for x in [-0.39, 0.39] {
            for z in [WALL + 0.03, WALL + DESK_D - 0.04] {
                fsweep!(
                    f,
                    format!("leg{x}{z}"),
                    &[(x, 0.73, z), (x, 0.08, z), (x, 0.0, z)],
                    0.0,
                    M.ivory,
                    profile = rounded_rect(0.034, 0.034, 0.008, 1),
                    up = (0, 1, 0),
                    scale = [1.0, 0.55, 0.62]
                );
            }
        }
        // the pigeonhole hutch facing the writer, its back on the wall, an arched gold crest
        fbox!(
            f,
            "hutch_back",
            0.8,
            0.38,
            0.02,
            (0, 0.95, WALL + 0.01),
            M.ivory,
            bevel = 0.004,
            bevel_segments = 1
        );
        for x in [-0.4, 0.4] {
            fbox!(
                f,
                format!("hutch_side{x}"),
                0.02,
                0.38,
                0.1,
                (x, 0.95, hz),
                M.ivory,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("hutch_panel{x}"),
                0.004,
                0.28,
                0.07,
                (x * 1.026, 0.95, hz),
                M.blush,
                bevel = 0.002,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "hutch_top",
            0.84,
            0.025,
            0.11,
            (0, 1.15, hz),
            M.ivory,
            bevel = 0.006,
            bevel_segments = 1
        );
        let mut crest = vec![(-0.42, 0.0), (0.42, 0.0)];
        crest.extend((0..17).map(|i| {
            (
                0.42 - 0.84 * i as f64 / 16.0,
                0.012 + 0.012 * sin(PI * i as f64 / 16.0),
            )
        }));
        crest.reverse();
        fextrude!(f, "crest", &crest, 0.11, (0, 1.1625, hz), M.gold);
        for x in [-0.2, 0.0, 0.2] {
            fbox!(
                f,
                format!("divider{x}"),
                0.012,
                0.36,
                0.09,
                (x, 0.95, hz + 0.005),
                M.ivory
            );
        }
        fbox!(
            f,
            "shelf",
            0.78,
            0.012,
            0.09,
            (0, 0.97, hz + 0.005),
            M.ivory
        );
        for (k, (x, y, m)) in [
            (-0.3, 1.0, M.paper),
            (-0.1, 1.0, M.blush),
            (0.3, 1.0, M.envelope),
            (-0.3, 0.8, M.paper),
            (0.1, 0.8, M.envelope),
        ]
        .into_iter()
        .enumerate()
        {
            for j in 0..3 {
                // envelopes standing in the pigeonhole, a ribbon round them
                let jf = j as f64;
                fbox!(
                    f,
                    format!("letter{k}{j}"),
                    0.13,
                    0.085 - 0.01 * jf,
                    0.004,
                    (x + 0.004 * jf, y + 0.018, hz - 0.02 + 0.016 * jf),
                    m
                );
            }
            fbox!(
                f,
                format!("letter_ribbon{k}"),
                0.01,
                0.09,
                0.05,
                (x, y + 0.02, hz - 0.004),
                M.rose
            );
        }
        // on the desk either side of the flap: an inkpot and a quill, a wax seal, a candle
        revolve!(
            "inkpot",
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.03, 0.035),
                (0.012, 0.045),
                (0.012, 0.06),
                (0.015, 0.062),
                (0.0, 0.062)
            ],
            at = f.at(0.33, 0.76, -0.2),
            material = M.ink,
            segments = 12
        );
        fsweep!(
            f,
            "quill",
            &[(0.33, 0.8, -0.2), (0.29, 0.9, -0.225), (0.25, 0.98, -0.25)],
            0.003,
            M.paper,
            verts = 4,
            resolution = 2
        );
        leaf!(
            "quill_vane",
            f.at(0.31, 0.86, -0.212),
            125.0,
            62.0,
            0.15,
            0.04,
            M.paper,
            shape = "lance",
            droop = 0.15,
            fold = 0.05,
            rows = 5,
            ring = 3
        );
        revolve!(
            "wax_seal",
            &[
                (0.0, 0.0),
                (0.026, 0.0),
                (0.028, 0.006),
                (0.024, 0.012),
                (0.0, 0.011)
            ],
            at = f.at(0.32, 0.76, -0.07),
            material = M.rosed,
            segments = 12
        );
        fsweep!(
            f,
            "wax_stick",
            &[(0.29, 0.766, -0.15), (0.31, 0.766, -0.06)],
            0.006,
            M.rose,
            verts = 6
        );
        let (wx, wy, wz) = _candle(f, "desk_candle", -0.33, -0.15, 0.76, 0.14, None);
        revolve!(
            "candle_flame",
            &[(0.0, 0.0), (0.008, 0.006), (0.01, 0.016), (0.0, 0.04)],
            at = f.at(wx, wy + 0.003, wz),
            material = M.flame_core,
            segments = 6
        );
        _velvet_chair(f, "chair", 0.0, CHAIR_Z, Some(M.ivory), None, true);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The writing flap, closed up over the lower middle pigeonholes, hinged on its foot: `fall_front` drops it towards
    // the writer onto the desk, a letter on its inside.
    let flap = new_parts(|| {
        kit::shaker!(
            f,
            "flap",
            0.5,
            0.2,
            (0, 0.87, hf + 0.009),
            0.018,
            M.ivory,
            rail = 0.04,
            panel = M.blush,
            recess = 0.004
        );
        kit::knob!(
            f,
            "flap_knob",
            0,
            0.95,
            hf + 0.018,
            0.01,
            M.gold,
            segments = 8
        );
        fbox!(
            f,
            "flap_letter",
            0.2,
            0.14,
            0.003,
            (-0.08, 0.87, hf - 0.002),
            M.paper,
            rot = (0, 0, 6)
        );
        fbox!(
            f,
            "flap_envelope",
            0.14,
            0.1,
            0.003,
            (0.13, 0.86, hf - 0.002),
            M.envelope,
            rot = (0, 0, -8)
        );
    });
    let flap = hinged(join(&flap, "flap", None), f.at(0, 0.77, hf + 0.009));
    let objs = settle!(centre = false);
    swing!(
        flap,
        "fall_front",
        90.0,
        seconds = 0.8,
        axis = artkit::math::Axis::X
    );
    objs
}

// --- the rose-arbour swing (2x1): a porch swing for two under an arch of climbing roses ---------------------
const BEAM: f64 = 2.1;
// the bloom that is cut, where a child reaches too (`cutRose*` in characters/clips/uses.ts)
const BLOOM: (f64, f64, f64) = (-0.84, 1.15, 0.35);

fn arch() -> Vec<(f64, f64)> {
    (0..9)
        .map(|t| {
            (
                -0.88 + 1.76 * t as f64 / 8.0,
                2.12 + 0.3 * sin(PI * t as f64 / 8.0),
            )
        })
        .collect()
}

/// Climbing roses up a post: a cane winding up it, leaves in threes along it, a bloom every so often.
#[allow(clippy::too_many_arguments)]
fn _climber(f: &GameFrame, name: &str, x: f64, z: f64, y0: f64, y1: f64, k0: i32) {
    let n = 13;
    let cane: Vec<(f64, f64, f64)> = (0..7)
        .map(|t| {
            let tf = t as f64;
            (
                x + 0.03 * cos(tf * 1.7),
                y0 + (y1 - y0) * tf / 6.0 - 0.3 * if t == 0 { 1.0 } else { 0.0 },
                z + 0.03 * sin(tf * 1.7),
            )
        })
        .collect();
    fsweep!(
        f,
        format!("{name}_cane"),
        &cane,
        0.006,
        M.leaf_dark,
        verts = 4,
        resolution = 2
    );
    for i in 0..n {
        let y = y0 + (y1 - y0) * i as f64 / (n - 1) as f64;
        let a = i as f64 * 137.5 + (k0 * 40) as f64;
        for j in 0..3 {
            leaf!(
                format!("{name}_leaf{i}_{j}"),
                f.at(x, y + 0.03 * j as f64, z),
                a + (120 * j) as f64,
                (10 + 15 * ((i + j) % 2)) as f64,
                0.1 - 0.008 * j as f64,
                0.07,
                if (i + j + k0) % 2 != 0 {
                    M.leaf
                } else {
                    M.leaf_dark
                },
                droop = 0.35,
                rows = 3,
                ring = 3
            );
        }
        if i % 3 == 1 {
            _rose(
                f,
                &format!("{name}_rose{i}"),
                x + 0.07 * cosd(a),
                y + 0.02,
                z + 0.07 * sind(a),
                0.045,
                Some(if (i + k0) % 2 != 0 {
                    M.rose
                } else {
                    M.rose_pink
                }),
                7,
                40.0,
                -a + 90.0,
            );
        }
    }
}

pub fn rose_arbour_swing() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let arch = arch();
    let still = new_parts(|| {
        for x in [-0.88, 0.88] {
            for z in [-0.36, 0.36] {
                fbox!(
                    f,
                    format!("post{x}{z}"),
                    0.07,
                    2.12,
                    0.07,
                    (x, 1.06, z),
                    M.frame,
                    bevel = 0.008,
                    bevel_segments = 1
                );
            }
            // the lattice between the posts: diagonal laths each way, one sweep each
            for i in -3..5 {
                for sgn in [-1, 1] {
                    let sf = sgn as f64;
                    let (za, zb) = (-0.33, 0.33);
                    let ya = 0.2 + i as f64 * 0.36;
                    let ((y0, z0), (y1, z1)) = ((ya, za * sf), (ya + 0.66, zb * sf));
                    let (lo, hi) = (0.2, 1.95); // clip to the panel
                    let t0 = 0.0_f64.max((lo - y0) / (y1 - y0));
                    let t1 = 1.0_f64.min((hi - y0) / (y1 - y0));
                    if t1 <= t0 {
                        continue;
                    }
                    let a = (x, y0 + (y1 - y0) * t0, z0 + (z1 - z0) * t0);
                    let b = (x, y0 + (y1 - y0) * t1, z0 + (z1 - z0) * t1);
                    fsweep!(
                        f,
                        format!("lattice{x}{i}{sgn}"),
                        &[a, b],
                        0.0,
                        M.frame,
                        profile = rect(0.01, 0.022),
                        up = (1, 0, 0)
                    );
                }
            }
            for y in [0.2, 1.95] {
                fbox!(
                    f,
                    format!("lattice_rail{x}{y}"),
                    0.03,
                    0.04,
                    0.66,
                    (x, y, 0),
                    M.frame,
                    bevel = 0.006,
                    bevel_segments = 1
                );
            }
        }
        for z in [-0.36, 0.36] {
            // the arch, a curved rafter each side in one sweep
            let rafter: Vec<(f64, f64, f64)> = arch.iter().map(|&(x, y)| (x, y, z)).collect();
            fsweep!(
                f,
                format!("arch{z}"),
                &rafter,
                0.0,
                M.frame,
                profile = rect(0.05, 0.06),
                up = (0, 0, 1),
                resolution = 2
            );
        }
        for &(x, y) in &arch[1..arch.len() - 1] {
            fbox!(
                f,
                format!("slat{x:.2}"),
                0.04,
                0.04,
                0.82,
                (x, y + 0.03, 0),
                M.frame,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "beam",
            1.82,
            0.08,
            0.08,
            (0, BEAM, 0.0),
            M.frame,
            bevel = 0.012,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                f,
                format!("beam_eye{s}"),
                &[(sf * 0.78, BEAM - 0.04, 0.0), (sf * 0.78, BEAM - 0.06, 0.0)],
                0.01,
                M.gold,
                verts = 6
            );
        }
        // climbing roses up the posts and over the arch
        let mut k = 0;
        for x in [-0.82, 0.82] {
            for z in [-0.3, 0.3] {
                _climber(f, &format!("climber{x}{z}"), x, z, 0.35, 1.9, k);
                k += 1;
            }
        }
        for (i, &(x, y)) in arch[1..arch.len() - 1].iter().enumerate() {
            for z in [-0.27, -0.13, 0.0, 0.13, 0.27] {
                leaf!(
                    format!("arch_leaf{i}{z}"),
                    f.at(x, y + 0.06, z),
                    (i * 77) as f64 + z * 300.0,
                    15.0,
                    0.13,
                    0.09,
                    if i % 2 != 0 { M.leaf_dark } else { M.leaf },
                    droop = 0.4,
                    rows = 3,
                    ring = 3
                );
                leaf!(
                    format!("arch_leaf{i}{z}b"),
                    f.at(x, y + 0.06, z),
                    (i * 77) as f64 + z * 300.0 + 150.0,
                    20.0,
                    0.11,
                    0.08,
                    if i % 2 != 0 { M.leaf } else { M.leaf_dark },
                    droop = 0.4,
                    rows = 3,
                    ring = 3
                );
            }
            _rose(
                f,
                &format!("arch_rose{i}"),
                x,
                y + 0.15,
                -0.25 + 0.17 * (i % 4) as f64,
                0.05,
                Some(if i % 2 != 0 { M.rose } else { M.rose_pink }),
                7,
                0.0,
                0.0,
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The swing (seat top 0.45 over z = 0) with its arms, cushions and chains, hung from the beam: `swing` rocks it,
    // and the residents sitting on it ride it.
    let swing = new_parts(|| {
        for k in 0..5 {
            // the seat's slats, a frame round them
            fbox!(
                f,
                format!("seat_slat{k}"),
                1.56,
                0.03,
                0.075,
                (0, 0.385, -0.17 + k as f64 * 0.08),
                M.frame,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        for z in [-0.21, 0.19] {
            fbox!(
                f,
                format!("seat_rail{z}"),
                1.6,
                0.06,
                0.03,
                (0, 0.37, z),
                M.frame,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        soft!(
            f,
            "seat_cushion",
            1.5,
            0.05,
            0.38,
            (0, 0.425, -0.005),
            M.cushion,
            round = 0.02,
            puff = 0.008,
            flat = 2,
            resting = true
        );
        for k in 0..7 {
            // the back's spindles between two rails
            let x = -0.6 + k as f64 * 0.2;
            fbox!(
                f,
                format!("back_spindle{k}"),
                0.03,
                0.38,
                0.02,
                (x, 0.66, -0.215),
                M.frame,
                rot = (-10, 0, 0),
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for y in [0.48, 0.86] {
            fbox!(
                f,
                format!("back_rail{y}"),
                1.6,
                0.04,
                0.035,
                (0, y, -0.215 - (y - 0.66) * 0.18),
                M.frame,
                rot = (-10, 0, 0),
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        soft!(
            f,
            "pillow_l",
            0.36,
            0.3,
            0.1,
            (-0.6, 0.64, -0.15),
            M.rose,
            rot = (-12, 0, -15),
            round = 0.04,
            puff = 0.03,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
        soft!(
            f,
            "pillow_r",
            0.34,
            0.28,
            0.1,
            (0.62, 0.63, -0.15),
            M.ivory,
            rot = (-12, 0, 15),
            round = 0.04,
            puff = 0.03,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                f,
                format!("arm{s}"),
                &[
                    (sf * 0.78, 0.86, -0.25),
                    (sf * 0.78, 0.63, -0.18),
                    (sf * 0.78, 0.62, 0.12),
                    (sf * 0.78, 0.6, 0.19)
                ],
                0.0,
                M.frame,
                profile = rounded_rect(0.05, 0.04, 0.012, 1),
                up = (1, 0, 0),
                resolution = 2
            );
            fbox!(
                f,
                format!("arm_post{s}"),
                0.04,
                0.2,
                0.04,
                (sf * 0.78, 0.5, 0.17),
                M.frame,
                bevel = 0.005,
                bevel_segments = 1
            );
            for (zc, yc) in [(0.17, 0.64), (-0.26, 0.86)] {
                fsweep!(
                    f,
                    format!("chain{s}{zc}"),
                    &[(sf * 0.78, yc, zc), (sf * 0.78, BEAM - 0.06, 0.0)],
                    0.007,
                    M.gold,
                    verts = 5
                );
            }
        }
    });
    let swing = hinged(join(&swing, "swing", None), f.at(0, BEAM, 0.0));
    // The bloom a resident cuts, origin on its stem: `cut` shrinks it away.
    let bloom = new_parts(|| {
        _rose(
            f,
            "bloom",
            BLOOM.0,
            BLOOM.1 - 0.03,
            BLOOM.2,
            0.06,
            Some(M.rose),
            7,
            35.0,
            90.0,
        );
        leaf!(
            "bloom_leaf",
            f.at(BLOOM.0 + 0.01, BLOOM.1 - 0.05, BLOOM.2 - 0.02),
            200.0,
            10.0,
            0.07,
            0.045,
            M.leaf,
            droop = 0.3,
            rows = 3,
            ring = 3
        );
    });
    let bloom = hinged(
        join(&bloom, "bloom", None),
        f.at(BLOOM.0, BLOOM.1 - 0.06, BLOOM.2),
    );
    let objs = settle!(centre = false);
    rock!(swing, "swing", 7.0, seconds = 2.0);
    squash!(bloom, "cut", 0.001, seconds = 0.6);
    objs
}
