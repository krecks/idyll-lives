//! The stereo and the boombox: the music the residents play.

use super::common::call::rbox;
use super::common::{axle, capsule, driver, records};
use super::{F, M};
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{Axis, P3, cosd, pow, sind};
use artkit::parts::{hinged, join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, fsweep, revolve, settle, soft, spin, squash, swing};

// --- the stereo (1x1): an oak hi-fi cabinet with an amplifier, a CD player and records in its open front, a
// turntable between two speakers on top ------------------------------------------------------------------------
/// The cabinet's middle depth: its back on the wall face.
const ST_Z: f64 = -0.22;

pub fn stereo() -> Vec<Obj> {
    let z = ST_Z;
    let d = 0.42;
    let still = new_parts(|| {
        // the open cabinet: a shell around two bays
        rbox!(
            "cab_top",
            0.92,
            0.025,
            d,
            (0, 0.5875, z),
            M.stereo_wood,
            r = 0.006
        );
        rbox!(
            "cab_bottom",
            0.92,
            0.025,
            d,
            (0, 0.1525, z),
            M.stereo_wood,
            r = 0.006
        );
        for s in [-1, 1] {
            rbox!(
                format!("cab_side{s}"),
                0.025,
                0.46,
                d,
                (s as f64 * 0.4475, 0.37, z),
                M.stereo_wood,
                r = 0.006
            );
        }
        fbox!(
            F,
            "cab_mid",
            0.02,
            0.41,
            d - 0.02,
            (-0.01, 0.37, z - 0.01),
            M.stereo_wood
        );
        fbox!(
            F,
            "cab_back",
            0.87,
            0.41,
            0.012,
            (0, 0.37, z - d / 2.0 + 0.006),
            M.stereo_inside
        );
        fbox!(
            F,
            "cab_shelf",
            0.43,
            0.018,
            d - 0.02,
            (-0.225, 0.37, z - 0.01),
            M.stereo_wood
        );
        for (i, x) in [-0.4, 0.4].into_iter().enumerate() {
            for (k, s) in [-1, 1].into_iter().enumerate() {
                // tapered legs, splayed a little, in brass shoes
                let sf = s as f64;
                fsweep!(
                    F,
                    format!("leg{i}{k}"),
                    &[(x, 0.14, z + sf * 0.16), (x * 1.04, 0.0, z + sf * 0.17)],
                    0.02,
                    M.stereo_wood,
                    verts = 8,
                    scale = [1.0, 0.65]
                );
            }
        }
        // the amplifier on the shelf: a brushed front, a display, two knobs and a power button
        rbox!(
            "amp",
            0.4,
            0.1,
            0.3,
            (-0.225, 0.43, z),
            M.stereo_dark,
            r = 0.006
        );
        fbox!(
            F,
            "amp_face",
            0.39,
            0.09,
            0.002,
            (-0.225, 0.43, z + 0.1505),
            M.steel_dark
        );
        fbox!(
            F,
            "amp_display",
            0.12,
            0.03,
            0.002,
            (-0.3, 0.44, z + 0.152),
            M.display
        );
        for x in [-0.15, -0.08] {
            revolve!(
                format!("amp_knob{}", float_repr(x)),
                &[
                    (0.0, 0.0),
                    (0.019, 0.0),
                    (0.019, 0.004),
                    (0.016, 0.016),
                    (0.0, 0.017)
                ],
                at = F.at(x, 0.44, z + 0.151),
                material = M.steel,
                segments = 14,
                rot = (90, 0, 0)
            );
        }
        fcyl!(
            F,
            "amp_power",
            0.006,
            0.004,
            (-0.39, 0.41, z + 0.152),
            M.led,
            rot = (90, 0, 0),
            verts = 8
        );
        // the CD player under it, its tray's slot
        rbox!(
            "cd",
            0.4,
            0.07,
            0.3,
            (-0.225, 0.2, z),
            M.stereo_dark,
            r = 0.006
        );
        fbox!(
            F,
            "cd_slot",
            0.16,
            0.012,
            0.002,
            (-0.26, 0.21, z + 0.1505),
            M.stereo_inside
        );
        fbox!(
            F,
            "cd_led",
            0.02,
            0.008,
            0.002,
            (-0.1, 0.2, z + 0.151),
            M.led
        );
        // records on end in the right bay
        let records = records();
        for (i, &rec) in records.iter().enumerate() {
            fbox!(
                F,
                format!("record{i}"),
                0.012,
                0.3,
                0.31,
                (0.04 + i as f64 * 0.036, 0.315, z - 0.02),
                rec,
                rot = (0, if i == records.len() - 1 { 3 } else { 0 }, 0)
            );
        }
        // the turntable on top: a plinth, the arm on its pivot with a counterweight and a headshell, the speed knob
        rbox!(
            "plinth",
            0.4,
            0.07,
            0.32,
            (0, 0.635, z),
            M.stereo_wood,
            r = 0.01
        );
        revolve!(
            "arm_base",
            &[
                (0.0, 0.0),
                (0.022, 0.0),
                (0.022, 0.012),
                (0.012, 0.016),
                (0.01, 0.03),
                (0.0, 0.03)
            ],
            at = F.at(0.15, 0.67, z - 0.1),
            material = M.steel,
            segments = 12
        );
        fsweep!(
            F,
            "tonearm",
            &[
                (0.175, 0.698, z - 0.135),
                (0.15, 0.698, z - 0.1),
                (0.1, 0.699, z - 0.0),
                (0.075, 0.699, z + 0.05)
            ],
            0.004,
            M.steel,
            verts = 6,
            resolution = 2
        );
        rbox!(
            "headshell",
            0.016,
            0.006,
            0.03,
            (0.072, 0.697, z + 0.06),
            M.stereo_dark,
            r = 0.002,
            rot = (0, 0, 25)
        );
        fcyl!(
            F,
            "counterweight",
            0.012,
            0.022,
            (0.178, 0.698, z - 0.14),
            M.steel,
            rot = axle(0.58, -0.81),
            verts = 10
        );
        fcyl!(
            F,
            "speed_knob",
            0.012,
            0.01,
            (-0.16, 0.675, z + 0.12),
            M.steel,
            verts = 10
        );
        // the speakers: oak boxes, dark baffles, a tweeter (the woofers pulse)
        for s in [-1, 1] {
            let x = s as f64 * 0.37;
            rbox!(
                format!("speaker{s}"),
                0.16,
                0.24,
                0.2,
                (x, 0.72, z - 0.02),
                M.stereo_wood,
                r = 0.01
            );
            fbox!(
                F,
                format!("baffle{s}"),
                0.14,
                0.22,
                0.004,
                (x, 0.72, z + 0.081),
                M.stereo_dark
            );
            driver(
                &format!("tweeter{s}"),
                x,
                0.79,
                z + 0.083,
                0.022,
                Some(M.grille),
                None,
                12,
            );
            fcyl!(
                F,
                format!("surround{s}"),
                0.058,
                0.004,
                (x, 0.685, z + 0.084),
                M.grille,
                rot = (90, 0, 0),
                verts = 24
            );
            fcyl!(
                F,
                format!("port{s}"),
                0.01,
                0.003,
                (x + 0.045, 0.62, z + 0.0835),
                M.stereo_inside,
                rot = (90, 0, 0),
                verts = 10
            );
        }
    });
    join(&still, "stereo", Some(P3::ZERO));
    // the record on the turntable: the platter, the vinyl and its label, with a stripe on the label so its turn
    // shows; it goes round at 33 rpm while the music plays (`record`, a loop)
    let (px, py_) = (-0.03, 0.68);
    let record = new_parts(|| {
        revolve!(
            "platter",
            &[
                (0.0, -0.01),
                (0.15, -0.01),
                (0.152, -0.006),
                (0.152, 0.006),
                (0.148, 0.01),
                (0.0, 0.01)
            ],
            at = F.at(px, py_, z),
            material = M.stereo_dark,
            segments = 28
        );
        // the vinyl: a disc with grooved rings, a raised label and the spindle
        revolve!(
            "vinyl",
            &[
                (0.0, 0.0),
                (0.145, 0.0),
                (0.145, 0.0025),
                (0.12, 0.0025),
                (0.119, 0.002),
                (0.09, 0.002),
                (0.089, 0.0025),
                (0.06, 0.0025),
                (0.059, 0.002),
                (0.048, 0.002),
                (0.046, 0.003),
                (0.0, 0.003)
            ],
            at = F.at(px, 0.69, z),
            material = M.vinyl,
            segments = 28
        );
        fcyl!(
            F,
            "vinyl_label",
            0.045,
            0.002,
            (px, 0.6935, z),
            M.label,
            verts = 16
        );
        fcyl!(
            F,
            "spindle",
            0.0035,
            0.012,
            (px, 0.697, z),
            M.steel,
            verts = 8
        );
        fbox!(
            F,
            "label_stripe",
            0.05,
            0.002,
            0.012,
            (px + 0.012, 0.6945, z),
            M.vinyl
        );
        fbox!(
            F,
            "vinyl_sheen",
            0.07,
            0.001,
            0.008,
            (px - 0.085, 0.6927, z + 0.03),
            M.disc,
            rot = (0, 20, 0)
        );
    });
    let record = join(&record, "record", Some(F.at(px, py_, z)));
    // the woofers' cones, one part, its origin on the baffles' face: they pulse forward and back
    let cones = new_parts(|| {
        for s in [-1, 1] {
            let x = s as f64 * 0.37;
            revolve!(
                format!("cone{s}"),
                &[
                    (0.0, 0.0),
                    (0.054, 0.016),
                    (0.054, 0.022),
                    (0.05, 0.02),
                    (0.024, 0.012),
                    (0.02, 0.006),
                    (0.0, 0.004)
                ],
                at = F.at(x, 0.685, z + 0.075),
                material = M.cone,
                segments = 18,
                rot = (90, 0, 0)
            );
            fsphere!(
                F,
                format!("cap{s}"),
                0.02,
                (x, 0.685, z + 0.093),
                M.cone,
                scale = (1, 0.5, 1),
                segments = 10,
                rings = 6
            );
        }
    });
    let cones = join(&cones, "cones", Some(F.at(0, 0.685, z + 0.082)));
    settle!(centre = false);
    spin!(record, "record", turns = 1, seconds = 1.8);
    squash!(cones, "pulse", (1, 1.6, 1), seconds = 0.5, back = true);
    meshes()
}

// --- the boombox (1x1): a twin-speaker cassette radio on a record crate turned on its side ---------------------
/// The boombox's foot: the crate's top.
const BB_Y: f64 = 0.42;
const BB_Z: f64 = -0.08;

pub fn boombox() -> Vec<Obj> {
    let (y0, z) = (BB_Y, BB_Z);
    // the boombox's front
    let zf = z + 0.07;
    let still = new_parts(|| {
        // the crate on its side, open to the front: slatted sides, top and back, corner posts, records inside
        rbox!(
            "crate_bottom",
            0.6,
            0.02,
            0.44,
            (0, 0.01, 0),
            M.crate_,
            r = 0.003
        );
        for s in [-1, 1] {
            let sf = s as f64;
            for y in [0.1, 0.3] {
                rbox!(
                    format!("crate_side{s}{}", float_repr(y)),
                    0.018,
                    0.13,
                    0.44,
                    (sf * 0.291, y, 0),
                    M.crate_,
                    r = 0.003
                );
            }
            for zz in [-0.205, 0.205] {
                let post = rbox!(
                    format!("crate_post{s}{}", float_repr(zz)),
                    0.03,
                    0.42,
                    0.03,
                    (sf * 0.285, 0.21, zz),
                    M.crate_dark,
                    r = 0.003
                );
                // (chamfered before its UVs are made, so the chamfers get their own)
                artkit::bevel::apply(post);
            }
        }
        for x in [-0.15, 0.15] {
            rbox!(
                format!("crate_top{}", float_repr(x)),
                0.27,
                0.018,
                0.44,
                (x, 0.411, 0),
                M.crate_,
                r = 0.003
            );
        }
        for y in [0.1, 0.3] {
            rbox!(
                format!("crate_back{}", float_repr(y)),
                0.6,
                0.13,
                0.018,
                (0, y, -0.211),
                M.crate_,
                r = 0.003
            );
        }
        let records = records();
        for i in 0..11 {
            fbox!(
                F,
                format!("record{i}"),
                0.01,
                0.3,
                0.3,
                (-0.24 + i as f64 * 0.033, 0.17, -0.04),
                records[i % records.len()],
                rot = (0, if i == 10 { -6 } else { 0 }, 0)
            );
        }
        for (i, (x, zz, turn)) in [(0.1, 0.1, 12), (0.13, 0.12, -20)].into_iter().enumerate() {
            rbox!(
                format!("tape_case{i}"),
                0.11,
                0.017,
                0.07,
                (x, 0.4285 + i as f64 * 0.017, zz),
                if i != 0 { M.cassette } else { M.rec_cream },
                r = 0.003,
                rot = (0, 0, turn)
            );
        }
        // the boombox: a rounded case, a metal top, two speakers either side of the cassette deck
        soft!(
            F,
            "case",
            0.52,
            0.26,
            0.14,
            (0, y0 + 0.13, z),
            M.boombox_body,
            round = 0.03,
            round_segments = 2
        );
        capsule(
            "top_strip",
            -0.25,
            0.25,
            y0 + 0.264,
            z,
            0.02,
            0.12,
            M.boombox_trim,
            Some(0.008),
        );
        for s in [-1, 1] {
            let x = s as f64 * 0.17;
            fcyl!(
                F,
                format!("speaker{s}"),
                0.075,
                0.006,
                (x, y0 + 0.12, zf + 0.003),
                M.grille,
                rot = (90, 0, 0),
                verts = 20
            );
            // the grille's ribs
            for k in 0..5i32 {
                fbox!(
                    F,
                    format!("speaker_rib{s}{k}"),
                    0.13 - 0.03 * pow((k - 2).abs() as f64, 1.4),
                    0.004,
                    0.002,
                    (x, y0 + 0.12 + 0.025 * (k - 2) as f64, zf + 0.007),
                    M.grille_dark
                );
            }
            let ring: Vec<(f64, f64, f64)> = (0..360)
                .step_by(18)
                .map(|a| {
                    let a = a as f64;
                    (x + 0.078 * cosd(a), y0 + 0.12 + 0.078 * sind(a), zf + 0.006)
                })
                .collect();
            fsweep!(
                F,
                format!("ring{s}"),
                &ring,
                0.007,
                M.boombox_trim,
                verts = 5,
                closed = true
            );
            fcyl!(
                F,
                format!("cap{s}"),
                0.022,
                0.012,
                (x, y0 + 0.12, zf + 0.008),
                M.cone,
                rot = (90, 0, 0),
                verts = 12
            );
            fcyl!(
                F,
                format!("tweeter{s}"),
                0.016,
                0.006,
                (x, y0 + 0.225, zf + 0.002),
                M.boombox_trim,
                rot = (90, 0, 0),
                verts = 10
            );
        }
        // the handle: one bent bar over the top on two posts, a rubber grip in the middle
        fsweep!(
            F,
            "handle",
            &kit::fillet(
                &[
                    (-0.19, y0 + 0.265, z),
                    (-0.19, y0 + 0.33, z),
                    (0.19, y0 + 0.33, z),
                    (0.19, y0 + 0.265, z)
                ],
                0.035,
                3
            ),
            0.012,
            M.boombox_trim,
            verts = 8
        );
        capsule(
            "handle_grip",
            -0.1,
            0.1,
            y0 + 0.33,
            z,
            0.03,
            0.03,
            M.cassette,
            Some(0.014),
        );
        fsweep!(
            F,
            "aerial",
            &[
                (0.22, y0 + 0.275, z - 0.05),
                (0.25, y0 + 0.4, z - 0.065),
                (0.28, y0 + 0.52, z - 0.08)
            ],
            0.005,
            M.boombox_trim,
            verts = 6,
            scale = [1.0, 0.8, 0.6]
        );
        fcyl!(
            F,
            "aerial_hinge",
            0.01,
            0.016,
            (0.22, y0 + 0.27, z - 0.05),
            M.boombox_trim,
            rot = (0, 90, 0),
            verts = 8
        );
        fsphere!(
            F,
            "aerial_tip",
            0.007,
            (0.28, y0 + 0.52, z - 0.08),
            M.boombox_trim,
            segments = 8,
            rings = 5
        );
        // the deck: a recess, the cassette in it, a tuning display above and piano keys on top
        fbox!(
            F,
            "deck",
            0.13,
            0.09,
            0.004,
            (0, y0 + 0.125, zf + 0.001),
            M.stereo_inside
        );
        rbox!(
            "tape",
            0.1,
            0.064,
            0.012,
            (0, y0 + 0.125, zf - 0.004),
            M.cassette,
            r = 0.003
        );
        fbox!(
            F,
            "tape_label",
            0.08,
            0.02,
            0.001,
            (0, y0 + 0.142, zf + 0.002),
            M.rec_cream
        );
        for x in [-0.025, 0.025] {
            fcyl!(
                F,
                format!("reel{}", float_repr(x)),
                0.011,
                0.004,
                (x, y0 + 0.125, zf + 0.003),
                M.rec_cream,
                rot = (90, 0, 0),
                verts = 10
            );
        }
        fbox!(
            F,
            "display",
            0.1,
            0.022,
            0.003,
            (0, y0 + 0.21, zf + 0.001),
            M.display
        );
        fbox!(
            F,
            "display_needle",
            0.002,
            0.018,
            0.001,
            (0.02, y0 + 0.21, zf + 0.003),
            M.led_red
        );
        for i in 0..5 {
            rbox!(
                format!("button{i}"),
                0.028,
                0.012,
                0.02,
                (-0.075 + i as f64 * 0.035, y0 + 0.275, z + 0.035),
                [M.cassette, M.cassette, M.led_red, M.cassette, M.cassette][i],
                r = 0.003
            );
        }
        for x in [-0.22, 0.22] {
            fcyl!(
                F,
                format!("dial{}", float_repr(x)),
                0.014,
                0.012,
                (x, y0 + 0.23, zf + 0.003),
                M.boombox_trim,
                rot = (90, 0, 0),
                verts = 12
            );
        }
    });
    join(&still, "boombox", Some(P3::ZERO));
    // the cassette door, hinged along its foot: `cassette` tips it open towards the front
    let hy = y0 + 0.125 - 0.045;
    let door = new_parts(|| {
        rbox!(
            "door_frame",
            0.13,
            0.09,
            0.008,
            (0, y0 + 0.125, zf + 0.006),
            M.cassette,
            r = 0.003
        );
        fbox!(
            F,
            "door_window",
            0.1,
            0.05,
            0.003,
            (0, y0 + 0.13, zf + 0.011),
            M.window
        );
        fbox!(
            F,
            "door_tab",
            0.04,
            0.008,
            0.004,
            (0, y0 + 0.165, zf + 0.011),
            M.boombox_trim
        );
    });
    let door = hinged(join(&door, "door", None), F.at(0, hy, zf + 0.006));
    settle!(centre = false);
    swing!(door, "cassette", 32.0, seconds = 0.5, axis = Axis::X);
    meshes()
}
