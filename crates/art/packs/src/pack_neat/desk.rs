//! The label-and-sort organizer desk and the tidy entry console.

use super::common::{_flat, _m, M, PASTELS, WALL};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, cosd, sind};
use artkit::parts::{join, new_parts, settle};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{
    cupped, fbox, fextrude, fsweep, jitter, leaf, loft, revolve, ring, settle, slide, soft, torus,
};

// --- the label-and-sort organizer desk (2x1): its back on the wall, the hutch of labelled bins on the desk's back,
// open towards the sitter and the room; the chair in front, the sitter facing the wall (2026-10-21: it stood the
// other way round, its backs to the room) ------------------------------------------------------------------------
const HD: f64 = 0.16; // the hutch's own depth
const HZ: f64 = WALL + 0.012 + HD / 2.0; // the hutch's middle depth, its back panel on the wall face
const DZ: f64 = WALL + 0.24; // the desk top's middle depth
const DD: f64 = 0.48; // and its depth: from the wall face to 0.05
const CZ: f64 = 0.25; // the chair seat's middle depth (the hips 4 cm nearer the desk: `furniture.json`)
const BIN: (f64, f64) = (0.0, 0.87); // the bin `drawer_open` pulls out (x, middle height): the lower row's middle

/// A storage bin: lofted, a little wider at its mouth, a label holder and a grip cut in its front.
fn _bin(f: &GameFrame, name: &str, x: f64, y: f64, material: Mat) {
    let b = f.at(x, y - 0.095, HZ - 0.005);
    let sections: Vec<Vec<P3>> = [(0.0, 0.94), (0.005, 0.97), (0.18, 1.0), (0.19, 1.02)]
        .iter()
        .map(|&(h, k)| {
            ring!(
                0.13 * k,
                (HD - 0.02) / 2.0 * k,
                z = b.z + h,
                verts = 16,
                square = 6,
                at = (b.x, b.y)
            )
        })
        .collect();
    loft!(name, &sections, material = material);
    let face = HZ - 0.005 + (HD - 0.02) / 2.0;
    soft!(
        f,
        format!("{name}_label"),
        0.1,
        0.04,
        0.006,
        (x, y + 0.035, face + 0.002),
        M.chrome,
        round = 0.003,
        round_segments = 1
    );
    fbox!(
        f,
        format!("{name}_card"),
        0.085,
        0.028,
        0.002,
        (x, y + 0.035, face + 0.005),
        M.paper
    );
    soft!(
        f,
        format!("{name}_grip"),
        0.08,
        0.022,
        0.006,
        (x, y - 0.04, face + 0.001),
        M.dark,
        round = 0.008,
        round_segments = 1
    );
}

/// A neat little chair, its back to the room: a padded seat (top 0.45) and back on splayed birch legs.
fn _chair(f: &GameFrame, x: f64, z: f64, mat: Mat, legs: Mat) {
    soft!(
        f,
        "chair_seat",
        0.46,
        0.06,
        0.44,
        (x, 0.415, z),
        mat,
        round = 0.025,
        puff = 0.006,
        flat = 2
    );
    soft!(
        f,
        "chair_back",
        0.44,
        0.3,
        0.05,
        (x, 0.74, z + 0.185),
        mat,
        rot = (-6, 0, 0),
        round = 0.022,
        puff = 0.006,
        flat = 2
    );
    for dx in [-0.19, 0.19] {
        for dz in [-0.17, 0.17] {
            fsweep!(
                f,
                format!("chair_leg{dx}{dz}"),
                &[(x + dx, 0.385, z + dz), (x + dx * 1.12, 0.0, z + dz * 1.15)],
                0.016,
                legs,
                verts = 8,
                scale = [1.0, 0.7]
            );
        }
        fsweep!(
            f,
            format!("chair_post{dx}"),
            &[(x + dx, 0.42, z + 0.18), (x + dx, 0.62, z + 0.2)],
            0.012,
            legs,
            verts = 8
        );
    }
}

pub fn organizer_desk() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            f,
            "desk_top",
            1.6,
            0.035,
            DD,
            (0, 0.745, DZ),
            M.birch,
            bevel = 0.01,
            bevel_segments = 2
        );
        for x in [-0.78, 0.78] {
            fbox!(
                f,
                format!("desk_side{x}"),
                0.04,
                0.73,
                DD - 0.02,
                (x, 0.365, DZ - 0.01),
                M.body,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "modesty",
            1.52,
            0.6,
            0.015,
            (0, 0.42, WALL + 0.02),
            M.birch,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the hutch: two rows of five cubbies, open towards the sitter
        fbox!(
            f,
            "hutch_shelf",
            1.6,
            0.02,
            HD,
            (0, 0.99, HZ),
            M.body,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "hutch_top",
            1.6,
            0.02,
            HD,
            (0, 1.22, HZ),
            M.body,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "hutch_back",
            1.6,
            0.5,
            0.012,
            (0, 1.0, HZ - HD / 2.0 + 0.004),
            M.birch
        );
        for i in 0..6 {
            fbox!(
                f,
                format!("divider{i}"),
                0.02,
                0.48,
                HD,
                (-0.79 + i as f64 * 0.316, 1.0, HZ),
                M.body,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for (r, y) in [0.87, 1.105].into_iter().enumerate() {
            for i in 0..5 {
                let x = -0.632 + i as f64 * 0.316;
                if (x, y) != BIN {
                    _bin(
                        f,
                        &format!("bin{r}{i}"),
                        x,
                        y,
                        _m(PASTELS[(i + r * 2) % PASTELS.len()]),
                    );
                }
            }
        }
        // on the desk in front of the hutch: the ledger, the label maker, a calculator, a pen cup
        let z = -0.1;
        soft!(
            f,
            "ledger",
            0.36,
            0.02,
            0.24,
            (0.32, 0.773, z),
            M.ledger,
            round = 0.006,
            round_segments = 1
        );
        fbox!(
            f,
            "ledger_pages",
            0.33,
            0.012,
            0.21,
            (0.32, 0.789, z),
            M.paper,
            bevel = 0.002,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "ledger_ribbon",
            &[
                (0.3, 0.796, z - 0.1),
                (0.31, 0.796, z + 0.05),
                (0.32, 0.79, z + 0.13)
            ],
            0.0,
            M.coral,
            profile = _flat(0.012, 0.002),
            up = (0, 0, 1),
            resolution = 2
        );
        let b = f.at(-0.4, 0.763, z);
        let maker: Vec<Vec<P3>> = [(0.0, 0.92), (0.006, 1.0), (0.04, 1.0), (0.052, 0.86)]
            .iter()
            .map(|&(h, k)| {
                ring!(
                    0.05 * k,
                    0.08 * k,
                    z = b.z + h,
                    verts = 14,
                    square = 3,
                    at = (b.x, b.y)
                )
            })
            .collect();
        loft!("label_maker", &maker, material = M.coral);
        soft!(
            f,
            "label_screen",
            0.06,
            0.004,
            0.05,
            (-0.4, 0.816, z - 0.035),
            M.screen,
            round = 0.002,
            round_segments = 1
        );
        for i in 0..3 {
            for j in 0..2 {
                fbox!(
                    f,
                    format!("label_key{i}{j}"),
                    0.014,
                    0.004,
                    0.012,
                    (-0.42 + i as f64 * 0.02, 0.816, z + 0.02 + j as f64 * 0.02),
                    M.paper
                );
            }
        }
        fbox!(
            f,
            "label_tape",
            0.03,
            0.004,
            0.06,
            (-0.4, 0.766, z - 0.11),
            M.paper
        );
        soft!(
            f,
            "calculator",
            0.12,
            0.016,
            0.16,
            (-0.62, 0.77, z),
            M.trim,
            round = 0.006,
            round_segments = 1
        );
        fbox!(
            f,
            "calc_screen",
            0.09,
            0.003,
            0.035,
            (-0.62, 0.779, z - 0.05),
            M.screen
        );
        for i in 0..4 {
            for j in 0..4 {
                fbox!(
                    f,
                    format!("calc_key{i}{j}"),
                    0.018,
                    0.004,
                    0.016,
                    (-0.65 + i as f64 * 0.022, 0.779, z - 0.01 + j as f64 * 0.022),
                    M.paper
                );
            }
        }
        revolve!(
            "pen_cup",
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.042, 0.11),
                (0.036, 0.11),
                (0.034, 0.01),
                (0.0, 0.01)
            ],
            at = f.at(0.62, 0.763, z - 0.06),
            material = M.pastel_f,
            segments = 14
        );
        for (k, m) in ["pastel_d", "ledger", "coral"].into_iter().enumerate() {
            let kf = k as f64;
            fsweep!(
                f,
                format!("pen{k}"),
                &[
                    (0.61 + 0.012 * kf, 0.775, z - 0.065 + 0.008 * kf),
                    (0.598 + 0.02 * kf, 0.94 - 0.01 * kf, z - 0.08 + 0.016 * kf)
                ],
                0.006,
                _m(m),
                verts = 6,
                scale = [1.0, 0.9]
            );
        }
        _chair(f, 0.0, CZ, M.pastel_c, M.birch);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The bin in the lower row's middle: `drawer_open` slides it out towards the sitter.
    let drawer = new_parts(|| {
        _bin(f, "pull_bin", BIN.0, BIN.1, M.pastel_b);
        for k in 0..3 {
            // labelled cards inside
            fbox!(
                f,
                format!("card{k}"),
                0.2,
                0.1,
                0.004,
                (
                    BIN.0 + 0.01 * k as f64,
                    BIN.1 + 0.05 + 0.01 * (k % 2) as f64,
                    HZ + 0.025 - 0.03 * k as f64
                ),
                [M.paper, M.pastel_d, M.paper][k]
            );
        }
    });
    let drawer = join(&drawer, "bin", None);
    let objs = settle!(centre = false);
    slide!(drawer, "drawer_open", (0, -0.13, 0), seconds = 0.6);
    objs
}

// --- the tidy entry console (1x1): a slim birch console, a shoe shelf, a key tray drawer, a leaning mirror -----
const DRAWER: (f64, f64) = (-0.2, 0.74); // the key tray drawer's middle (x, height)
const FZ: f64 = WALL + 0.3; // the console's front
const PANEL_TOP: f64 = 1.78; // the back panel's top, under its cap

/// A shoe: a lofted upper rising to the heel, its opening dark, on a white sole.
fn _shoe(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, material: Mat) {
    let b = f.at(x, y + 0.012, z);
    let upper: Vec<Vec<P3>> = [(0.0, 1.0), (0.035, 1.0), (0.05, 0.85)]
        .iter()
        .map(|&(h, k)| {
            [
                (0.0, -0.11),
                (0.03, -0.09),
                (0.038, 0.0),
                (0.034, 0.09),
                (0.0, 0.105),
                (-0.034, 0.09),
                (-0.038, 0.0),
                (-0.03, -0.09),
            ]
            .iter()
            .map(|&(px, py)| P3::from((b.x + px * k, b.y + py, b.z + h)))
            .collect()
        })
        .collect();
    loft!(name, &upper, material = material);
    soft!(
        f,
        format!("{name}_opening"),
        0.05,
        0.004,
        0.09,
        (x, y + 0.064, z - 0.045),
        M.dark,
        round = 0.002,
        round_segments = 1
    );
    soft!(
        f,
        format!("{name}_sole"),
        0.08,
        0.014,
        0.225,
        (x, y + 0.007, z),
        M.body,
        round = 0.006,
        round_segments = 1
    );
}

pub fn entry_console() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (dx, dy) = DRAWER;
    let still = new_parts(|| {
        fbox!(
            f,
            "top",
            0.86,
            0.035,
            0.3,
            (0, 0.8, WALL + 0.15),
            M.birch,
            bevel = 0.01,
            bevel_segments = 2
        );
        for k in 0..3 {
            // the shoe shelf: three slats
            fbox!(
                f,
                format!("shoe_slat{k}"),
                0.82,
                0.022,
                0.075,
                (0, 0.22, WALL + 0.05 + k as f64 * 0.1),
                M.birch,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        for x in [-0.41, 0.41] {
            for z in [WALL + 0.02, FZ - 0.02] {
                fsweep!(
                    f,
                    format!("leg{x}{z}"),
                    &[(x, 0.0, z), (x, 0.79, z)],
                    0.0,
                    M.body,
                    profile = rounded_rect(0.03, 0.03, 0.008, 1),
                    up = (0, 1, 0),
                    scale = [0.75, 1.0]
                );
            }
            fbox!(
                f,
                format!("shelf_rail{x}"),
                0.03,
                0.03,
                0.26,
                (x, 0.22, WALL + 0.15),
                M.body,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // the apron round the drawer's opening, and its housing
        fbox!(
            f,
            "apron_r",
            0.82 - (dx + 0.17 + 0.41),
            0.08,
            0.02,
            ((dx + 0.17 + 0.41) / 2.0, 0.74, FZ - 0.02),
            M.body,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "apron_l",
            0.07,
            0.08,
            0.02,
            (-0.375, 0.74, FZ - 0.02),
            M.body,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "housing",
            0.36,
            0.01,
            0.26,
            (dx, 0.695, WALL + 0.15),
            M.body
        );
        fbox!(
            f,
            "apron_back",
            0.82,
            0.08,
            0.02,
            (0, 0.74, WALL + 0.02),
            M.body
        );
        // shoes, neatly paired
        for (i, m) in ["shoe_a", "shoe_b", "shoe_c"].into_iter().enumerate() {
            let x = -0.27 + i as f64 * 0.27;
            for ddx in [-0.045, 0.045] {
                _shoe(f, &format!("shoe{i}{ddx}"), x + ddx, 0.231, -0.27, _m(m));
            }
        }
        // on top: a bowl, a vase of tulips
        revolve!(
            "bowl",
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.07, 0.03),
                (0.08, 0.058),
                (0.074, 0.06),
                (0.064, 0.032),
                (0.044, 0.012),
                (0.0, 0.012)
            ],
            at = f.at(0.02, 0.8175, -0.27),
            material = M.appliance,
            segments = 18
        );
        revolve!(
            "vase",
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.06, 0.09),
                (0.03, 0.19),
                (0.034, 0.22),
                (0.03, 0.22),
                (0.0, 0.2)
            ],
            at = f.at(0.28, 0.8175, -0.3),
            material = M.bottle_b,
            segments = 14
        );
        for k in 0..5 {
            let a = (k * 72) as f64;
            let (x, z) = (0.28 + 0.035 * cosd(a), -0.3 + 0.035 * sind(a));
            fsweep!(
                f,
                format!("tulip_stem{k}"),
                &[
                    (0.28, 0.98, -0.3),
                    (0.28 + 0.02 * cosd(a), 1.1, -0.3 + 0.02 * sind(a)),
                    (x, 1.2, z)
                ],
                0.004,
                M.stem,
                verts = 4,
                resolution = 2
            );
            cupped!(
                format!("tulip{k}"),
                &[
                    (0.0, 0.0),
                    (0.016, 0.004),
                    (0.024, 0.03),
                    (0.02, 0.06),
                    (0.012, 0.064),
                    (0.014, 0.05),
                    (0.0, 0.04)
                ],
                f.at(x, 1.198, z),
                _m(["coral", "pastel_b", "pad", "coral", "pastel_e"][k]),
                yaw = -a + 90.0,
                tilt = 10.0,
                segments = 6
            );
            leaf!(
                format!("tulip_leaf{k}"),
                f.at(0.28, 1.0, -0.3),
                (k * 72 + 36) as f64,
                70.0,
                0.12,
                0.03,
                M.stem,
                shape = "strap",
                droop = 0.3,
                rows = 3,
                ring = 3
            );
        }
        // the back panel rising from the top (a hall stand's), standing free or on the wall: the mirror on it, the coat
        // hooks with a tote and a scarf (2026-10-21: they hung on the air over a free-standing console)
        let (pb, pt) = (0.8175, PANEL_TOP);
        fbox!(
            f,
            "back_panel",
            0.82,
            pt - pb,
            0.018,
            (0, (pb + pt) / 2.0, WALL + 0.009),
            M.birch,
            bevel = 0.004,
            bevel_segments = 1
        );
        for x in [-0.41, 0.41] {
            fbox!(
                f,
                format!("panel_post{x}"),
                0.03,
                pt - 0.79,
                0.03,
                (x, (0.79 + pt) / 2.0, WALL + 0.02),
                M.body,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "panel_cap",
            0.87,
            0.025,
            0.045,
            (0, pt + 0.0125, WALL + 0.0225),
            M.body,
            bevel = 0.008,
            bevel_segments = 2
        );
        let pz = WALL + 0.018;
        kit::moulding!(
            f,
            "mirror_frame",
            &[(-0.2, 0.9), (0.3, 0.9), (0.3, 1.76), (-0.2, 1.76)],
            &[
                (0.0, 0.0),
                (0.035, 0.0),
                (0.035, 0.012),
                (0.024, 0.026),
                (0.008, 0.026),
                (0.0, 0.016)
            ],
            M.body,
            at = pz
        );
        fbox!(
            f,
            "mirror",
            0.44,
            0.8,
            0.006,
            (0.05, 1.33, pz + 0.004),
            M.mirror
        );
        fbox!(
            f,
            "hook_board",
            0.05,
            0.4,
            0.02,
            (-0.33, 1.42, pz + 0.01),
            M.oak,
            bevel = 0.006,
            bevel_segments = 1
        );
        for y in [1.5, 1.32] {
            // J-hooks
            fsweep!(
                f,
                format!("hook{y}"),
                &[
                    (-0.33, y, pz + 0.02),
                    (-0.33, y - 0.01, pz + 0.05),
                    (-0.33, y - 0.02, pz + 0.06),
                    (-0.33, y + 0.005, pz + 0.07)
                ],
                0.006,
                M.brass,
                verts = 6,
                resolution = 2
            );
        }
        soft!(
            f,
            "tote",
            0.2,
            0.24,
            0.06,
            (-0.31, 1.2, pz + 0.06),
            M.wicker,
            round = 0.02,
            puff = 0.01,
            flat = 2,
            round_segments = 1
        );
        fsweep!(
            f,
            "tote_handle",
            &[
                (-0.38, 1.32, pz + 0.06),
                (-0.34, 1.33, pz + 0.068),
                (-0.33, 1.315, pz + 0.07),
                (-0.28, 1.33, pz + 0.068),
                (-0.24, 1.32, pz + 0.06)
            ],
            0.006,
            M.oak,
            verts = 5,
            resolution = 2
        );
        let scarf = fsweep!(
            f,
            "scarf",
            &[
                (-0.3, 1.04, pz + 0.075),
                (-0.305, 1.25, pz + 0.07),
                (-0.33, 1.49, pz + 0.075),
                (-0.335, 1.5, pz + 0.06),
                (-0.33, 1.49, pz + 0.045),
                (-0.31, 1.28, pz + 0.04)
            ],
            0.0,
            M.coral,
            profile = _flat(0.06, 0.01),
            up = (0, 0, 1),
            resolution = 2
        );
        jitter!(scarf, 0.003, size = 0.08, seed = 4.0);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The key tray drawer: front, knob, the tray and keys inside: `tray` slides it out.
    let drawer = new_parts(|| {
        fbox!(
            f,
            "drawer_front",
            0.32,
            0.07,
            0.018,
            (dx, dy, FZ - 0.009),
            M.body,
            bevel = 0.005,
            bevel_segments = 1
        );
        kit::knob!(f, "drawer_knob", dx, dy, FZ, 0.012, M.brass, segments = 10);
        fbox!(
            f,
            "drawer_box",
            0.3,
            0.05,
            0.24,
            (dx, dy - 0.005, FZ - 0.14),
            M.trim
        );
        soft!(
            f,
            "key_tray",
            0.24,
            0.012,
            0.16,
            (dx, dy + 0.026, FZ - 0.12),
            M.oak,
            round = 0.005,
            round_segments = 1
        );
        torus!(
            "key_ring",
            0.018,
            0.0025,
            at = f.at(dx - 0.03, dy + 0.034, FZ - 0.11),
            material = M.brass,
            major_segments = 10,
            minor_segments = 4
        );
        for k in 0..2 {
            let kf = k as f64;
            fextrude!(
                f,
                format!("key{k}"),
                &[
                    (0.0, -0.006),
                    (0.04, -0.004),
                    (0.045, 0.0),
                    (0.04, 0.004),
                    (0.0, 0.006)
                ],
                0.003,
                (dx - 0.01 + 0.005 * kf, dy + 0.034, FZ - 0.1 + 0.012 * kf),
                M.brass,
                plane = Facing::Top
            );
        }
    });
    let drawer = join(&drawer, "drawer", None);
    let objs = settle(&meshes()); // on its Model centre: the back on the wall face
    slide!(drawer, "tray", (0, -0.2, 0), seconds = 0.6);
    objs
}
