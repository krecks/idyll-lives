//! The litter bin, the post box and the notice board.

use super::common::own;
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, TAU, cos, radians, sin};
use artkit::motion::{Swing, swing};
use artkit::parts::{hinged, join, join_all, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{
    fbox, fcone, fcyl, fextrude, fsphere, fsweep, kit_fillet, loft, revolve, ring, settle,
};

// --- the litter bin: a slatted green bin on a post ----------------------------------------------------------
/// A ring round a bin of radius `r` at height `y` (Blender): `ribs` slats standing `out` proud, steep-sided, narrow
/// grooves between them.
fn ribbed(r: f64, y: f64, ribs: usize, out: f64, at: (f64, f64)) -> Vec<P3> {
    let mut pts = Vec::new();
    let span = TAU / ribs as f64;
    for k in 0..ribs {
        // each rib's flanks slope at under 60° to its face, as a press leaves them
        for (f, rr) in [(0.02, r), (0.16, r + out), (0.84, r + out), (0.98, r)] {
            let a = span * (k as f64 + f);
            pts.push(P3 {
                x: at.0 + rr * cos(a),
                y: at.1 + rr * sin(a),
                z: y,
            });
        }
    }
    pts
}

pub fn litter_bin() -> Vec<Obj> {
    let m = own("litterBin");
    let f = GameFrame::new(0.0);
    let z = 0.02;
    let P3 { x: bx, y: by_, .. } = f.at(0, 0, z);
    {
        // the post: a square tube with a pyramid cap, the bin hung on it by two straps round its back
        fbox!(
            f,
            "post",
            0.07,
            0.95,
            0.07,
            (0, 0.475, -0.28),
            m.iron,
            bevel = 0.008,
            bevel_segments = 1
        );
        fcone!(
            f,
            "post_cap",
            0.064,
            0.012,
            0.03,
            (0, 0.965, -0.28),
            m.iron,
            verts = 4,
            rot = (0, 0, 45),
            smooth = false
        );
        for y in [0.33, 0.66] {
            let pts: Vec<(f64, f64, f64)> = (200..345)
                .step_by(12)
                .map(|a| {
                    (
                        0.216 * cos(radians(a as f64)),
                        y,
                        z + 0.216 * sin(radians(a as f64)),
                    )
                })
                .collect();
            fsweep!(
                f,
                format!("strap{}", float_repr(y)),
                &pts,
                0,
                m.iron,
                profile = rounded_rect(0.04, 0.008, 0.0, 4)
            );
            fbox!(
                f,
                format!("lug{}", float_repr(y)),
                0.05,
                0.04,
                0.06,
                (0, y, -0.225),
                m.iron,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        // the bin: pressed ribs all round between two rolled beads, a rounded foot, a rolled rim round its dark mouth
        let n = 48;
        let rows = [
            ring!(0.17, 0.17, z = 0.18, verts = n, at = (bx, by_)),
            ring!(0.19, 0.19, z = 0.185, verts = n, at = (bx, by_)),
            ring!(0.2, 0.2, z = 0.2, verts = n, at = (bx, by_)),
            ring!(0.208, 0.208, z = 0.215, verts = n, at = (bx, by_)),
            ribbed(0.2, 0.235, 12, 0.012, (bx, by_)),
            ribbed(0.206, 0.71, 12, 0.012, (bx, by_)),
            ring!(0.214, 0.214, z = 0.735, verts = n, at = (bx, by_)),
            ring!(0.208, 0.208, z = 0.775, verts = n, at = (bx, by_)),
        ];
        loft!("bin", &rows, material = m.body);
        let rim: Vec<(f64, f64, f64)> = (0..24)
            .map(|i| TAU * i as f64 / 24.0)
            .map(|a| (0.216 * cos(a), 0.785, z + 0.216 * sin(a)))
            .collect();
        fsweep!(f, "rim", &rim, 0.021, m.iron, verts = 5, closed = true);
        fcyl!(f, "opening", 0.2, 0.01, (0, 0.785, z), m.dark, verts = 24);
        // a black bag's edge turned over the rim at the back
        let bag: Vec<(f64, f64, f64)> = (210..335)
            .step_by(25)
            .map(|a| {
                (
                    0.17 * cos(radians(a as f64)),
                    0.79,
                    z + 0.17 * sin(radians(a as f64)),
                )
            })
            .collect();
        fsweep!(
            f,
            "bag",
            &bag,
            0,
            m.dark,
            profile = rounded_rect(0.012, 0.012, 0.0, 4),
            resolution = 2
        );
        // the label on its front: a white plate with a bin sign and two lines on it
        artkit::bevel::eased(fextrude!(
            f,
            "label",
            &rounded_rect(0.16, 0.08, 0.01, 2),
            0.012,
            (0, 0.62, z + 0.216),
            m.paper
        ));
        fextrude!(
            f,
            "sign_bin",
            &[
                (-0.012, -0.018),
                (0.012, -0.018),
                (0.016, 0.012),
                (-0.016, 0.012)
            ],
            0.004,
            (-0.045, 0.615, z + 0.223),
            m.dark
        );
        fbox!(
            f,
            "sign_lid",
            0.04,
            0.005,
            0.004,
            (-0.045, 0.633, z + 0.223),
            m.dark
        );
        for (k, w) in [0.07, 0.05].into_iter().enumerate() {
            fbox!(
                f,
                format!("sign_line{k}"),
                w,
                0.008,
                0.004,
                (0.0 + w / 2.0 - 0.01, 0.628 - 0.02 * k as f64, z + 0.223),
                m.dark
            );
        }
    }
    join_all("litterBin");
    settle!(centre = false);
    meshes()
}

// --- the post box: a red pillar box; its slot's flap swings in for a card ------------------------------------
const SLOT_Y: f64 = 1.16; // the flap's hinge (its top edge), game height
const SLOT_Z: f64 = 0.262; // the flap's face, game depth

/// A bead (a round cord `radius` thick) along `path`, [(degrees round from the front, height), ...], on the
/// pillar's face of radius `r`: the door's outline, cast on.
fn on_pillar(
    f: &GameFrame,
    name: &str,
    path: &[(f64, f64)],
    r: f64,
    radius: f64,
    material: Mat,
    closed: bool,
) -> Obj {
    let pts: Vec<(f64, f64, f64)> = path
        .iter()
        .map(|&(a, y)| (r * sin(radians(a)), y, r * cos(radians(a))))
        .collect();
    fsweep!(f, name, &pts, radius, material, verts = 5, closed = closed)
}

pub fn post_box() -> Vec<Obj> {
    let m = own("postBox");
    let f = GameFrame::new(0.0);
    let n = 32;
    let still = new_parts(|| {
        revolve!(
            "plinth",
            &[
                (0.0, 0.0),
                (0.245, 0.0),
                (0.245, 0.05),
                (0.236, 0.062),
                (0.226, 0.068),
                (0.222, 0.082),
                (0.0, 0.082)
            ],
            material = m.dark,
            segments = n
        );
        // the pillar in one turned piece: a foot moulding, the band, the shaft, a stepped cornice and the domed cap
        revolve!(
            "pillar",
            &[
                (0.0, 0.08),
                (0.222, 0.08),
                (0.222, 0.1),
                (0.214, 0.108),
                (0.212, 0.22),
                (0.222, 0.226),
                (0.224, 0.24),
                (0.222, 0.254),
                (0.212, 0.26),
                (0.21, 1.3),
                (0.222, 1.31),
                (0.236, 1.33),
                (0.25, 1.345),
                (0.25, 1.375),
                (0.24, 1.392),
                (0.232, 1.4),
                (0.226, 1.43),
                (0.205, 1.465),
                (0.165, 1.495),
                (0.11, 1.516),
                (0.05, 1.527),
                (0.0, 1.529)
            ],
            material = m.body,
            segments = n
        );
        // the door's outline cast round the front, its keyhole and a little lifting handle
        let door = [
            (-38, 0.32),
            (38, 0.32),
            (38, 0.94),
            (30, 0.99),
            (0, 1.0),
            (-30, 0.99),
            (-38, 0.94),
        ];
        let closed: Vec<(i32, f64)> = door.iter().copied().chain([door[0]]).collect();
        let mut path: Vec<(f64, f64)> = kit_fillet!(&closed, 0.0, steps = 1);
        path.pop();
        on_pillar(&f, "door", &path, 0.211, 0.006, m.body, true);
        fcyl!(
            f,
            "keyhole_plate",
            0.022,
            0.006,
            (0.0, 0.42, 0.212),
            m.body,
            verts = 12,
            rot = (90, 0, 0),
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "keyhole",
            0.006,
            0.018,
            0.004,
            (0.0, 0.42, 0.216),
            m.dark
        );
        // the slot's housing, proud of the pillar: an arched hood, a sill and cheeks round a dark opening
        fextrude!(
            f,
            "hood",
            &[
                (0.145, 0.0),
                (0.145, 0.018),
                (0.08, 0.034),
                (0.0, 0.04),
                (-0.08, 0.034),
                (-0.145, 0.018),
                (-0.145, 0.0)
            ],
            0.09,
            (0, SLOT_Y + 0.01, 0.225),
            m.body,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "sill",
            0.27,
            0.025,
            0.07,
            (0, SLOT_Y - 0.07, 0.23),
            m.body,
            bevel = 0.006
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("cheek{s}"),
                0.025,
                0.1,
                0.07,
                (s as f64 * 0.1225, SLOT_Y - 0.02, 0.23),
                m.body,
                bevel = 0.006
            );
        }
        fbox!(
            f,
            "opening",
            0.22,
            0.08,
            0.02,
            (0, SLOT_Y - 0.025, 0.21),
            m.dark
        );
        // the collection times: a framed white plate with its lines
        artkit::bevel::eased(fextrude!(
            f,
            "times_frame",
            &rounded_rect(0.16, 0.13, 0.012, 2),
            0.012,
            (0, 0.82, 0.208),
            m.body,
            holes = [rounded_rect(0.136, 0.106, 0.006, 1)]
        ));
        // (sharp: a chamfer would draw it back from the frame's inner edge)
        fbox!(
            f,
            "times",
            0.14,
            0.11,
            0.012,
            (0, 0.82, 0.205),
            m.paper,
            bevel = 0
        );
        for k in 0..4 {
            fbox!(
                f,
                format!("times_line{k}"),
                0.1 - 0.02 * (k % 2) as f64,
                0.008,
                0.003,
                (-0.005, 0.85 - 0.02 * k as f64, 0.212),
                m.dark
            );
        }
        // the crest over the slot: a little crown in brass on a plate
        fextrude!(
            f,
            "crest",
            &[
                (-0.04, -0.025),
                (0.04, -0.025),
                (0.045, 0.012),
                (0.026, 0.0),
                (0.014, 0.022),
                (0.0, 0.004),
                (-0.014, 0.022),
                (-0.026, 0.0),
                (-0.045, 0.012)
            ],
            0.01,
            (0, 1.265, 0.21),
            m.brass
        );
        fbox!(
            f,
            "crest_band",
            0.08,
            0.012,
            0.012,
            (0, 1.236, 0.211),
            m.brass,
            bevel = 0.003,
            bevel_segments = 1
        );
    });
    join(&still, "postBox", Some(P3::from((0, 0, 0))));
    let flap = new_parts(|| {
        fbox!(
            f,
            "flap",
            0.21,
            0.05,
            0.008,
            (0, SLOT_Y - 0.025, SLOT_Z),
            m.body,
            bevel = 0.003
        );
        fbox!(
            f,
            "flap_lip",
            0.12,
            0.008,
            0.012,
            (0, SLOT_Y - 0.045, SLOT_Z + 0.006),
            m.body,
            bevel = 0.002
        );
        fbox!(
            f,
            "flap_word",
            0.08,
            0.008,
            0.002,
            (0, SLOT_Y - 0.022, SLOT_Z + 0.004),
            m.paper
        ); // "LETTERS"
    });
    let flap = hinged(join(&flap, "flap", None), f.at(0, SLOT_Y, SLOT_Z));
    settle_all();
    swing(
        flap,
        "flap",
        65.0,
        Swing {
            seconds: 0.5,
            axis: Axis::X,
            ..Swing::default()
        },
    ); // its bottom edge swings in, to the back
    meshes()
}

// --- the notice board: a cork board between two posts, under a little roof -----------------------------------
pub fn notice_board() -> Vec<Obj> {
    let m = own("noticeBoard");
    let f = GameFrame::new(0.0);
    let z = -0.05;
    {
        for x in [-0.42, 0.42] {
            fbox!(
                f,
                format!("post{}", float_repr(x)),
                0.08,
                1.95,
                0.08,
                (x, 0.975, z),
                m.body,
                bevel = 0.01
            );
            fbox!(
                f,
                format!("foot{}", float_repr(x)),
                0.1,
                0.03,
                0.1,
                (x, 0.015, z),
                m.iron,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // the board: cork in a moulded frame, a ledge under it
        fbox!(f, "board", 0.8, 0.7, 0.04, (0, 1.36, z - 0.005), m.cork);
        fextrude!(
            f,
            "frame",
            &rounded_rect(0.9, 0.8, 0.01, 1),
            0.07,
            (0, 1.36, z + 0.005),
            m.body,
            holes = [rounded_rect(0.78, 0.68, 0.004, 1)],
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "ledge",
            0.86,
            0.025,
            0.06,
            (0, 0.955, z + 0.02),
            m.body,
            bevel = 0.006
        );
        // the roof: two boarded slopes, each three overlapping boards, a ridge cap and shaped barge boards
        for s in [-1, 1] {
            for k in 0..3 {
                let d = 0.025 + 0.058 * k as f64;
                fbox!(
                    f,
                    format!("board{s}{k}"),
                    1.0,
                    0.02,
                    0.075,
                    (
                        0,
                        2.03 - d * 0.47 - 0.006 * k as f64,
                        z + s as f64 * (d * 0.88 + 0.01)
                    ),
                    m.body,
                    bevel = 0.004,
                    bevel_segments = 1,
                    rot = (s * 28, 0, 0)
                );
            }
        }
        fbox!(
            f,
            "ridge",
            1.02,
            0.03,
            0.04,
            (0, 2.05, z),
            m.body,
            bevel = 0.008
        );
        for x in [-0.5, 0.5] {
            fextrude!(
                f,
                format!("barge{}", float_repr(x)),
                &[
                    (-0.1735, -0.075),
                    (0.0, 0.03),
                    (0.1735, -0.075),
                    (0.1735, -0.105),
                    (0.0, -0.005),
                    (-0.1735, -0.105)
                ],
                0.02,
                (x, 2.0, z),
                m.body,
                plane = Facing::Side,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // the notices, pinned: one a leaflet with tear-off strips along its foot
        let notes = [
            (-0.27, 1.55, 0.18, 0.22, m.note_white),
            (-0.05, 1.5, 0.16, 0.2, m.note_yellow),
            (0.2, 1.56, 0.2, 0.16, m.note_pink),
            (-0.25, 1.2, 0.16, 0.22, m.note_blue),
            (0.0, 1.22, 0.2, 0.18, m.note_white),
            (0.25, 1.24, 0.16, 0.2, m.note_green),
        ];
        for (k, (x, y, w, h, note)) in notes.into_iter().enumerate() {
            let tilt = (k as i32 % 3 - 1) * 3;
            fbox!(
                f,
                format!("note{k}"),
                w,
                h,
                0.004,
                (x, y, z + 0.018),
                note,
                rot = (0, tilt, 0)
            );
            fsphere!(
                f,
                format!("pin{k}"),
                0.012,
                (x, y + h / 2.0 - 0.025, z + 0.024),
                m.pin,
                segments = 8,
                rings = 4
            );
            if k == 1 || k == 4 {
                fsphere!(
                    f,
                    format!("pin{k}b"),
                    0.012,
                    (x + w / 3.0, y - h / 2.0 + 0.03, z + 0.024),
                    m.pin,
                    segments = 8,
                    rings = 4
                );
            }
            for j in 0..3 {
                // lines of writing
                fbox!(
                    f,
                    format!("note{k}_line{j}"),
                    w * (0.7 - 0.15 * (j % 2) as f64),
                    0.008,
                    0.002,
                    (x, y + h * 0.15 - j as f64 * 0.03, z + 0.021),
                    m.iron,
                    rot = (0, tilt, 0)
                );
            }
        }
        for j in 0..6 {
            // the leaflet's tear-off strips
            fbox!(
                f,
                format!("strip{j}"),
                0.022,
                0.05,
                0.004,
                (0.25 - 0.06 + j as f64 * 0.025, 1.24 - 0.12, z + 0.019),
                m.note_green
            );
        }
    }
    join_all("noticeBoard");
    settle!(centre = false);
    meshes()
}
