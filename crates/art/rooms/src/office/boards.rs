//! O3: the boards on the wall: the whiteboard and the corkboard.

use super::common::{Run, moulding};
use super::{F, M, WALL};
use artkit::materials::Mat;
use artkit::math::{Axis, TAU, cos, cosd, sin, sind};
use artkit::motion::{Grow, grow};
use artkit::parts::{Settle, hinged, join, join_all, new_parts, settle_with};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsweep, revolve, soft};

/// A whiteboard marker lying across at (x, y, z): a barrel, its coloured cap with a clip and an end plug.
pub(super) fn marker(name: &str, x: f64, y: f64, z: f64, ink: Mat) {
    let turn = 0.0;
    revolve!(
        name,
        &[
            (0.0, -0.05),
            (0.008, -0.05),
            (0.009, -0.045),
            (0.009, 0.02),
            (0.006, 0.03),
            (0.0, 0.032)
        ],
        at = F.at(x, y, z),
        material = M.label,
        segments = 10,
        rot = (0, 90, turn)
    );
    revolve!(
        format!("{name}_cap"),
        &[
            (0.0, 0.012),
            (0.0098, 0.012),
            (0.0098, 0.048),
            (0.008, 0.052),
            (0.0, 0.053)
        ],
        at = F.at(x, y, z),
        material = ink,
        segments = 10,
        rot = (0, 90, turn)
    );
    fcyl!(
        F,
        format!("{name}_plug"),
        0.0085,
        0.004,
        (x + 0.05, y, z),
        ink,
        rot = (0, 90, 0),
        verts = 10
    );
}

pub fn whiteboard() -> Vec<Obj> {
    let (y, z) = (1.4, WALL);
    let face = z + 0.0145; // the board's writing face
    let still = new_parts(|| {
        fbox!(F, "board", 0.88, 0.58, 0.014, (0, y, z + 0.0075), M.board);
        // the frame: one aluminium profile round the board, mitred at the corners, grey caps on them
        let frame = [
            (0.4625, y - 0.3125),
            (0.4625, y + 0.3125),
            (-0.4625, y + 0.3125),
            (-0.4625, y - 0.3125),
        ];
        moulding(
            &F,
            "frame",
            &frame,
            &[
                (0.0, 0.0),
                (0.0, 0.02),
                (0.006, 0.025),
                (0.024, 0.025),
                (0.026, 0.019),
                (0.026, 0.0),
            ],
            M.aluminium,
            z,
            Run::Front,
            50.0,
        );
        for s in [-1, 1] {
            for t in [-1, 1] {
                soft!(
                    F,
                    format!("corner{s}{t}"),
                    0.036,
                    0.036,
                    0.03,
                    (f64::from(s) * 0.4475, y + f64::from(t) * 0.2975, z + 0.015),
                    M.plastic,
                    round = 0.01,
                    round_segments = 2
                );
            }
        }
        // the marker tray on two brackets: a J-shaped trough, three markers and an eraser
        let trough = [
            (0.0, 0.0),
            (0.0, 0.07),
            (0.012, 0.07),
            (0.026, 0.066),
            (0.026, 0.06),
            (0.014, 0.062),
            (0.008, 0.058),
            (0.006, 0.006),
        ];
        let trough: Vec<(f64, f64)> = trough.iter().map(|&(u, v)| (u, -v)).collect();
        fsweep!(
            F,
            "tray",
            &[(-0.3, 1.068, z), (0.3, 1.068, z)],
            0.0,
            M.aluminium,
            profile = trough,
            up = (0, 0, 1),
            angle = 50
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("tray_end{s}"),
                0.006,
                0.03,
                0.07,
                (f64::from(s) * 0.303, 1.083, z + 0.035),
                M.plastic,
                bevel = 0.002,
                bevel_segments = 1
            );
        }
        for (k, (x, m)) in [(-0.2, M.ink_red), (-0.12, M.ink_blue), (-0.04, M.ink_black)]
            .into_iter()
            .enumerate()
        {
            marker(
                &format!("marker{k}"),
                x + 0.03,
                1.084,
                z + 0.035 + 0.006 * (k as f64 - 1.0),
                m,
            );
        }
        fbox!(
            F,
            "eraser",
            0.12,
            0.022,
            0.05,
            (0.18, 1.093, z + 0.037),
            M.plastic,
            bevel = 0.008,
            bevel_segments = 2
        );
        fbox!(
            F,
            "eraser_felt",
            0.116,
            0.008,
            0.046,
            (0.18, 1.079, z + 0.037),
            M.note2
        );
        // what's on it at rest: a heading, a sticky note and a sheet held up by two magnets
        scrawl(
            "heading",
            &[
                (0.1, 1.6),
                (0.14, 1.606),
                (0.18, 1.598),
                (0.22, 1.607),
                (0.26, 1.6),
                (0.32, 1.604),
            ],
            face,
            M.ink_black,
            0.004,
        );
        fbox!(
            F,
            "heading_rule",
            0.3,
            0.003,
            0.002,
            (0.2, 1.585, face),
            M.ink_black
        );
        fbox!(
            F,
            "sticky",
            0.1,
            0.1,
            0.003,
            (-0.33, 1.6, face + 0.0005),
            M.note,
            rot = (0, 4, 0)
        );
        fbox!(
            F,
            "sheet",
            0.16,
            0.21,
            0.002,
            (-0.34, 1.33, face + 0.001),
            M.paper,
            rot = (0, -3, 0)
        );
        for (k, (x, yy, m)) in [(-0.38, 1.43, M.ink_red), (-0.3, 1.43, M.ink_blue)]
            .into_iter()
            .enumerate()
        {
            fcyl!(
                F,
                format!("magnet{k}"),
                0.011,
                0.008,
                (x, yy, face + 0.006),
                m,
                rot = (90, 0, 0),
                verts = 12,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for j in 0..4 {
            fbox!(
                F,
                format!("sheet_line{j}"),
                0.11 - 0.02 * f64::from(j % 2),
                0.003,
                0.0004,
                (-0.345, 1.38 - 0.03 * f64::from(j), face + 0.0022),
                M.pen,
                rot = (0, -3, 0)
            );
        }
    });
    join(&still, "whiteboard", Some((0, 0, 0).into()));
    // the plan, drawn while a resident plans: lines and a chart grow across the board from its left (game +x,
    // as seen from the front), out of nothing: scaled to a point on the left edge at rest
    let left = 0.36;
    let parts = new_parts(|| {
        for (k, (x0, yy, w, m)) in [
            (0.05, 1.52, 0.3, M.ink_blue),
            (0.09, 1.48, 0.26, M.ink_blue),
            (0.13, 1.44, 0.22, M.ink_blue),
        ]
        .into_iter()
        .enumerate()
        {
            // handwriting: a wavering line of loops
            let pts: Vec<(f64, f64)> = (0..17)
                .map(|i| {
                    let i = f64::from(i);
                    (
                        x0 + w * i / 16.0,
                        yy + 0.008 * sin(i * 2.1) + 0.004 * sin(i * 5.3),
                    )
                })
                .collect();
            scrawl(&format!("line{k}"), &pts, face + 0.001, m, 0.0035);
        }
        scrawl(
            "underline",
            &[(-0.11, 1.32), (0.1, 1.322), (0.35, 1.318)],
            face + 0.001,
            M.ink_black,
            0.004,
        );
        // an arrow and a ring round a word
        scrawl(
            "arrow",
            &[(-0.25, 1.52), (-0.15, 1.48), (-0.06, 1.45)],
            face + 0.001,
            M.ink_red,
            0.004,
        );
        scrawl(
            "arrow_head",
            &[(-0.08, 1.475), (-0.06, 1.45), (-0.09, 1.442)],
            face + 0.001,
            M.ink_red,
            0.004,
        );
        let ring: Vec<(f64, f64)> = (0..15)
            .map(|i| {
                let i = f64::from(i);
                (
                    0.3 + 0.045 * cos(TAU * i / 14.0),
                    1.2 + 0.03 * sin(TAU * i / 14.0),
                )
            })
            .collect();
        scrawl("ring", &ring, face + 0.001, M.ink_blue, 0.0035);
        for (k, h) in [0.08, 0.14, 0.11, 0.19].into_iter().enumerate() {
            fbox!(
                F,
                format!("bar{k}"),
                0.035,
                h,
                0.002,
                (-0.08 - 0.06 * k as f64, 1.17 + h / 2.0, face + 0.001),
                M.ink_red
            );
        }
        fbox!(
            F,
            "axis",
            0.28,
            0.004,
            0.002,
            (-0.17, 1.168, face + 0.0012),
            M.ink_black
        );
        fbox!(
            F,
            "axis_up",
            0.004,
            0.24,
            0.002,
            (-0.03, 1.29, face + 0.0012),
            M.ink_black
        );
    });
    let scribble = hinged(join(&parts, "scribble", None), F.at(left, y, face));
    scribble.edit(|o| {
        o.scale.z = 0.001;
        o.scale.x = 0.001;
    });
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: false,
        },
    );
    grow(
        scribble,
        "scribble",
        Grow {
            seconds: 1.2,
            axis: Axis::X,
        },
    );
    grow(
        scribble,
        "scribble",
        Grow {
            seconds: 1.2,
            axis: Axis::Z,
        },
    );
    meshes()
}

/// A pen stroke on a board through game (x, height) points at depth `face`: a flat ribbon `w` wide.
pub(super) fn scrawl(name: &str, pts: &[(f64, f64)], face: f64, material: Mat, w: f64) -> Obj {
    let path: Vec<(f64, f64, f64)> = pts.iter().map(|&(x, yy)| (x, yy, face)).collect();
    fsweep!(
        F,
        name,
        &path,
        0.0,
        material,
        profile = [
            (-w / 2.0, 0.0),
            (0.0, -0.0006),
            (w / 2.0, 0.0),
            (0.0, 0.0006)
        ],
        up = (0, 0, 1),
        caps = true
    )
}

/// A push pin in a board at depth `face`: a round grip on a collar, its point in the cork.
pub(super) fn pushpin(name: &str, x: f64, y: f64, face: f64, material: Mat) {
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (0.0025, 0.0),
            (0.0025, 0.003),
            (0.006, 0.004),
            (0.005, 0.008),
            (0.008, 0.01),
            (0.0085, 0.0128),
            (0.006, 0.0144),
            (0.0, 0.0149)
        ],
        at = F.at(x, y, face),
        material = material,
        segments = 10,
        rot = (90, 0, 0)
    );
}

/// A note pinned at its top: a flat sheet hanging down the board, its foot curling away from it.
#[allow(clippy::too_many_arguments)]
pub(super) fn note(
    name: &str,
    x: f64,
    y: f64,
    face: f64,
    w: f64,
    h: f64,
    material: Mat,
    turn: f64,
) -> Obj {
    let curl = 0.012;
    let (c, s) = (cosd(turn), sind(turn));
    let mut pts = Vec::new();
    for i in 0..5 {
        let t = f64::from(i) / 4.0;
        let dy = h / 2.0 - h * t;
        let out = curl * artkit::math::pow(super::common::pmax(0.0, (t - 0.55) / 0.45), 2.0);
        pts.push((x - s * dy, y + c * dy, face + 0.0012 + out));
    }
    fsweep!(
        F,
        name,
        &pts,
        0.0,
        material,
        profile = [
            (-0.0006, -w / 2.0),
            (-0.0006, w / 2.0),
            (0.0006, w / 2.0),
            (0.0006, -w / 2.0)
        ],
        up = (0, 1, 0),
        angle = 30
    )
}

pub fn corkboard() -> Vec<Obj> {
    let (y, z) = (1.45, WALL);
    let face = z + 0.016; // the cork's face
    new_parts(|| {
        fbox!(F, "cork", 0.68, 0.46, 0.016, (0, y, z + 0.008), M.cork);
        let frame = [
            (0.37, y - 0.26),
            (0.37, y + 0.26),
            (-0.37, y + 0.26),
            (-0.37, y - 0.26),
        ];
        moulding(
            &F,
            "frame",
            &frame,
            &[
                (0.0, 0.0),
                (0.0, 0.022),
                (0.008, 0.03),
                (0.026, 0.03),
                (0.035, 0.022),
                (0.035, 0.0),
            ],
            M.cork_frame,
            z,
            Run::Front,
            50.0,
        );
        for (k, (x, yy, m, r, pin)) in [
            (-0.2, 1.55, M.note, 6, M.pin),
            (0.0, 1.5, M.note2, -4, M.ink_blue),
            (0.2, 1.56, M.note3, 3, M.pin),
            (-0.15, 1.35, M.note4, -6, M.note),
            (0.18, 1.34, M.note2, 4, M.pin),
        ]
        .into_iter()
        .enumerate()
        {
            let r = f64::from(r);
            note(&format!("note{k}"), x, yy, face, 0.14, 0.12, m, r);
            pushpin(
                &format!("pin{k}"),
                x - sind(r) * 0.045,
                yy + cosd(r) * 0.045,
                face + 0.0015,
                pin,
            );
        }
        // a photo, its white border round the picture, and a ticket
        fbox!(
            F,
            "photo",
            0.1,
            0.12,
            0.002,
            (-0.02, 1.33, face + 0.002),
            M.label,
            rot = (0, 8, 0)
        );
        fbox!(
            F,
            "photo_picture",
            0.084,
            0.084,
            0.0006,
            (-0.0185, 1.337, face + 0.0033),
            M.ink_blue,
            rot = (0, 8, 0)
        );
        fbox!(
            F,
            "photo_hill",
            0.084,
            0.03,
            0.0006,
            (-0.0175, 1.31, face + 0.0036),
            M.leafy,
            rot = (0, 8, 0)
        );
        pushpin("pin_photo", -0.025, 1.38, face + 0.0022, M.ink_red);
        fbox!(
            F,
            "ticket",
            0.12,
            0.05,
            0.0015,
            (0.06, 1.62, face + 0.001),
            M.note4,
            rot = (0, -10, 0)
        );
        pushpin("pin_ticket", 0.06, 1.63, face + 0.002, M.note);
        // a few lines on the notes
        for (k, (x, yy, r)) in [(-0.2, 1.55, 6), (0.0, 1.5, -4), (0.18, 1.34, 4)]
            .into_iter()
            .enumerate()
        {
            let rf = f64::from(r);
            for j in 0..3 {
                let dy = 0.02 - 0.025 * f64::from(j);
                fbox!(
                    F,
                    format!("scrawl{k}{j}"),
                    0.09 - 0.02 * f64::from(i32::from(j == 2)),
                    0.004,
                    0.001,
                    (x - sind(rf) * dy, yy + cosd(rf) * dy, face + 0.0025),
                    M.ink_black,
                    rot = (0, r, 0)
                );
            }
        }
    });
    join_all("corkboard");
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: false,
        },
    );
    meshes()
}
