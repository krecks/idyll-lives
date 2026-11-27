//! D4: wall fittings — the venetian blinds, the coat hooks, and the door's gadgets: the bell, the alarm keypad and the camera.

use super::common::{M, WALL};
use artkit::kit;
use artkit::math::{Axis, P3};
use artkit::motion::{FLAT, flat};
use artkit::parts::{hinged, join, join_all, new_parts, turn};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, grow, settle, squash, swing};

// --- D4: the venetian blinds: a head rail on the wall, slats, a bottom rail and their cords -------------
// At night (`close`) the slats shut: one part can't turn each slat about its own length, so the open slats thin to
// nothing front to back while the shut ones, tilted and overlapping, come down out of the head rail over them.
pub fn blinds() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let z = WALL + 0.035;
    let still = new_parts(|| {
        fbox!(
            f,
            "head",
            0.90,
            0.055,
            0.070,
            (0, 1.925, z),
            M.blind_rail,
            bevel = 0.006
        );
        fbox!(
            f,
            "bottom",
            0.90,
            0.028,
            0.060,
            (0, 1.160, z),
            M.blind_rail,
            bevel = 0.004
        );
        for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
            fbox!(
                f,
                format!("cord{i}"),
                0.006,
                0.700,
                0.004,
                (sx * 0.28, 1.530, z + 0.030),
                M.blind_rail
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    let open = new_parts(|| {
        for i in 0..8 {
            fbox!(
                f,
                format!("slat{i}"),
                0.86,
                0.008,
                0.055,
                (0, 1.870 - 0.085 * i as f64, z),
                M.blind_slat
            );
        }
    });
    let open = hinged(join(&open, "slats", None), f.at(0, 1.870, z));
    // shut: each slat turned 70° about its length, wide enough to overlap the next
    let shut = new_parts(|| {
        for i in 0..8 {
            fbox!(
                f,
                format!("vane{i}"),
                0.86,
                0.004,
                0.095,
                (0, 1.870 - 0.085 * i as f64, z),
                M.blind_slat,
                rot = (70, 0, 0)
            );
        }
    });
    // (from inside the head rail, where it hides flat at rest)
    let shut = hinged(join(&shut, "vanes", None), f.at(0, 1.925, z));
    settle!();
    flat(shut, Axis::Z);
    squash!(open, "close", (1, FLAT, 1), seconds = 1.2);
    grow!(shut, "close", seconds = 1.2, axis = Axis::Z);
    meshes()
}

// --- D4: the coat hooks: a painted board on the wall and four J hooks, each on a round rose ----------------
// A hook runs out from the board at 1.60 m (the height `coatHooks:fetch` reaches for), dips and turns up to a
// ball tip, so a coat's loop stays on it.
pub fn coat_hooks() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let face = WALL + 0.022;
    fbox!(
        f,
        "board",
        0.90,
        0.14,
        0.022,
        (0, 1.60, WALL + 0.011),
        M.hook_board,
        bevel = 0.006
    );
    for i in 0..4 {
        let x = -0.315 + 0.21 * i as f64;
        fcyl!(
            f,
            format!("rose{i}"),
            0.014,
            0.006,
            (x, 1.60, face + 0.003),
            M.gadget_chrome,
            rot = (90, 0, 0),
            verts = 16,
            bevel = 0.002
        );
        let pts: Vec<(f64, f64, f64)> = [
            (1.600, 0.000),
            (1.596, 0.030),
            (1.590, 0.058),
            (1.596, 0.076),
            (1.614, 0.083),
            (1.636, 0.080),
        ]
        .iter()
        .map(|&(y, d)| (x, y, face + d))
        .collect();
        kit::tube!(
            f,
            format!("hook{i}"),
            &pts,
            0.0065,
            M.gadget_chrome,
            round = 4
        );
        fsphere!(
            f,
            format!("tip{i}"),
            0.0085,
            (x, 1.638, face + 0.080),
            M.gadget_chrome,
            segments = 10,
            rings = 6
        );
    }
    join_all("coatHooks");
    settle!();
    meshes()
}

// --- D4: the doorbell: a brass plate with its button, beside the door --------------------------------------
pub fn doorbell() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    fbox!(
        f,
        "plate",
        0.10,
        0.16,
        0.024,
        (0, 1.35, WALL + 0.012),
        M.gilt,
        bevel = 0.004
    );
    fbox!(
        f,
        "button",
        0.036,
        0.036,
        0.012,
        (0, 1.34, WALL + 0.030),
        M.gadget_chrome,
        bevel = 0.004
    );
    fsphere!(f, "lens", 0.008, (0, 1.40, WALL + 0.026), M.cam_lens);
    settle!();
    meshes()
}

// --- D4: the burglar alarm: the keypad by the door, its keys and its armed light ---------------------------
pub fn burglar_alarm() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    fbox!(
        f,
        "case",
        0.22,
        0.28,
        0.050,
        (0, 1.70, WALL + 0.025),
        M.gadget_white,
        bevel = 0.008
    );
    fbox!(
        f,
        "keypad",
        0.16,
        0.13,
        0.008,
        (0, 1.66, WALL + 0.054),
        M.gadget_dark,
        bevel = 0.003
    );
    for r in 0..3 {
        for c in 0..4 {
            fbox!(
                f,
                format!("key{r}{c}"),
                0.026,
                0.020,
                0.004,
                (
                    -0.051 + 0.034 * c as f64,
                    1.700 - 0.036 * r as f64,
                    WALL + 0.059
                ),
                M.gadget_white
            );
        }
    }
    fsphere!(f, "led", 0.007, (0.085, 1.81, WALL + 0.052), M.alarm_led);
    settle!();
    meshes()
}

// --- D4: the security camera: a bracket under the eaves and its body with the lens -------------------------
pub fn cctv_camera() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    fbox!(
        f,
        "mount",
        0.08,
        0.12,
        0.020,
        (0, 2.00, WALL + 0.010),
        M.gadget_white,
        bevel = 0.004
    );
    fbox!(
        f,
        "arm",
        0.030,
        0.030,
        0.120,
        (0, 2.00, WALL + 0.080),
        M.gadget_white,
        bevel = 0.004
    );
    fbox!(
        f,
        "body",
        0.10,
        0.10,
        0.200,
        (0, 1.97, WALL + 0.240),
        M.gadget_white,
        bevel = 0.020
    );
    fbox!(
        f,
        "hood",
        0.11,
        0.020,
        0.210,
        (0, 2.03, WALL + 0.240),
        M.gadget_white,
        bevel = 0.006
    );
    fbox!(
        f,
        "lens",
        0.062,
        0.062,
        0.012,
        (0, 1.97, WALL + 0.345),
        M.cam_lens,
        bevel = 0.006
    );
    settle!();
    meshes()
}

// --- D4: the shutters: two louvred panels, folded back flat on the wall either side of the window by day ------
// and swung shut over it at night (`close`), each on its hinge at the window's edge. A panel is the same both sides
// (frame, core and louvres centred on its hinge line), so folded back it shows its louvres too.
pub fn shutters() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    // the hinge line's depth: the frame's back 1 cm off the wall's face
    let z = WALL + 0.035;
    let knuckles = new_parts(|| {
        for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
            for k in 0..2 {
                fbox!(
                    f,
                    format!("knuckle{i}{k}"),
                    0.016,
                    0.08,
                    0.016,
                    (sx * 0.445, 1.24 + 0.42 * k as f64, z),
                    M.gadget_chrome,
                    bevel = 0.003
                );
            }
        }
    });
    join(&knuckles, "body", Some(P3::ZERO));
    let mut panels = Vec::new();
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        let x = sx * 0.225;
        let parts = new_parts(|| {
            fbox!(
                f,
                format!("core{i}"),
                0.37,
                0.69,
                0.012,
                (x, 1.45, z),
                M.blind_rail,
                bevel = 0
            );
            // the stiles and rails of its frame, proud of the louvres
            for (k, dx) in [-0.20, 0.20].into_iter().enumerate() {
                fbox!(
                    f,
                    format!("stile{i}{k}"),
                    0.035,
                    0.76,
                    0.050,
                    (x + dx, 1.45, z),
                    M.hook_board,
                    bevel = 0.004
                );
            }
            for (k, y) in [1.085, 1.815].into_iter().enumerate() {
                fbox!(
                    f,
                    format!("rail{i}{k}"),
                    0.365,
                    0.035,
                    0.050,
                    (x, y, z),
                    M.hook_board,
                    bevel = 0.004
                );
            }
            for k in 0..6 {
                fbox!(
                    f,
                    format!("louvre{i}{k}"),
                    0.365,
                    0.030,
                    0.042,
                    (x, 1.16 + 0.098 * k as f64, z),
                    M.blind_rail,
                    bevel = 0.004,
                    rot = (20, 0, 0)
                );
            }
        });
        // hinged at the window's edge and folded back flat on the wall beside it, at rest
        let panel = hinged(
            join(&parts, &format!("shutter{i}"), None),
            f.at(sx * 0.445, 1.45, z),
        );
        panels.push((turn(panel, 180.0, Axis::Z), sx));
    }
    settle!();
    for (panel, sx) in panels {
        swing!(panel, "close", 180.0 * sx, seconds = 1.2);
    }
    meshes()
}

// --- D4: the hall stand: a mirror on its board, the hooks under it and the bench with its shoe shelf -------
pub fn hall_stand() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let back = WALL;
    fbox!(
        f,
        "board",
        0.90,
        1.70,
        0.030,
        (0, 0.95, back + 0.015),
        M.oak,
        bevel = 0.006
    );
    fbox!(
        f,
        "glass",
        0.50,
        0.58,
        0.010,
        (0, 1.45, back + 0.035),
        M.hall_glass,
        bevel = 0.003
    );
    for (i, (dx, dy, w, h)) in [
        (0.0, 0.31, 0.58, 0.035),
        (0.0, -0.31, 0.58, 0.035),
        (-0.27, 0.0, 0.035, 0.62),
        (0.27, 0.0, 0.035, 0.62),
    ]
    .into_iter()
    .enumerate()
    {
        fbox!(
            f,
            format!("bead{i}"),
            w,
            h,
            0.016,
            (dx, 1.45 + dy, back + 0.038),
            M.hook_board,
            bevel = 0.003
        );
    }
    for (i, x) in [-0.25f64, 0.0, 0.25].into_iter().enumerate() {
        fbox!(
            f,
            format!("arm{i}"),
            0.018,
            0.022,
            0.075,
            (x, 1.05, back + 0.030 + 0.0375),
            M.gadget_chrome,
            bevel = 0.004
        );
        fsphere!(
            f,
            format!("ball{i}"),
            0.016,
            (x, 1.05, back + 0.112),
            M.gadget_chrome
        );
    }
    // the bench, on its two front legs, and the shoe shelf under it
    fbox!(
        f,
        "seat",
        0.90,
        0.045,
        0.32,
        (0, 0.45, back + 0.19),
        M.oak,
        bevel = 0.008
    );
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("leg{i}"),
            0.045,
            0.44,
            0.045,
            (sx * 0.41, 0.22, back + 0.31),
            M.oak,
            bevel = 0.005
        );
    }
    fbox!(
        f,
        "shelf",
        0.84,
        0.020,
        0.28,
        (0, 0.12, back + 0.20),
        M.oak,
        bevel = 0.004
    );
    settle!();
    meshes()
}
