//! The dishwashers: a door dropping open over a sliding lower rack.

use crate::common::shaker;
use crate::kitchen_cooking::common::hollow;
use crate::kitchen_cooking::{M, SECONDS};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::Axis;
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, revolve, slide, soft, swing};

// --- dishwashers: a door dropping open over a sliding lower rack, a cabinet beside it under the worktop --

/// A dishwasher's wire basket at height `y` (its floor): a floor frame with bars across, a top rail, four posts.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn basket(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, w: f64, d: f64, h: f64, bars: usize) {
    for (k, yy) in [y, y + h].into_iter().enumerate() {
        let pts: Vec<(f64, f64, f64)> = rounded_rect(w, d, 0.02, 2)
            .into_iter()
            .map(|(u, v)| (x + u, yy, z + v))
            .collect();
        fsweep!(
            f,
            format!("{name}_rail{k}"),
            &pts,
            0.004,
            M.tray,
            verts = 4,
            closed = true
        );
    }
    for i in 0..bars {
        let u = -w / 2.0 + w * (i + 1) as f64 / (bars + 1) as f64;
        fsweep!(
            f,
            format!("{name}_bar{i}"),
            &[(x + u, y, z - d / 2.0), (x + u, y, z + d / 2.0)],
            0.003,
            M.tray,
            verts = 4,
            caps = false
        );
    }
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            let (fx, fz) = (sx as f64, sz as f64);
            fsweep!(
                f,
                format!("{name}_post{sx}{sz}"),
                &[
                    (x + fx * (w / 2.0 - 0.006), y, z + fz * (d / 2.0 - 0.006)),
                    (
                        x + fx * (w / 2.0 - 0.006),
                        y + h,
                        z + fz * (d / 2.0 - 0.006)
                    )
                ],
                0.003,
                M.tray,
                verts = 4,
                caps = false
            );
        }
    }
}

/// A dishwasher `dw` wide at the left of a `w`-wide unit under one worktop: a steel tub (an upper basket of cups and
/// a spray arm in it) whose door drops on its bottom hinge and whose lower basket of plates slides out, a control
/// fascia over the door. Beside it, the rest of the unit: a drawer over a door, or (`drawers`) a stack of three
/// drawers.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn dishwasher_unit(
    f: &GameFrame,
    w: f64,
    h: f64,
    top_d: f64,
    top_z: f64,
    front: f64,
    door_h: f64,
    body: Mat,
    panel: Mat,
    top: Mat,
    dw: f64,
    drawers: bool,
) {
    let mx = -w / 2.0 + dw / 2.0; // the machine's middle
    let cw = w - dw;
    let cx = w / 2.0 - cw / 2.0; // the cabinet's
    // the carcass runs from the back up to the fronts' backs (the compact one stood 2 cm proud of its door and drawers,
    // hiding them), the tub cut out of it behind the door
    let (cb, cf) = (top_z - (top_d - 0.06) / 2.0 - 0.01, front - 0.01);
    let face = cf;
    let tub_d = top_d - 0.12;
    let tz = face - tub_d / 2.0;
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            w - 0.04,
            0.10,
            top_d - 0.12,
            (0, 0.05, top_z - 0.04),
            M.dark,
            bevel = 0.005
        );
        let carcass = fbox!(
            f,
            "carcass",
            w - 0.02,
            h - 0.14,
            cf - cb,
            (0, h / 2.0 - 0.02, (cf + cb) / 2.0),
            body,
            bevel = 0.006
        );
        hollow(
            carcass,
            f,
            "tub",
            dw - 0.08,
            door_h - 0.04,
            tub_d,
            (mx, 0.12 + door_h / 2.0, tz + 0.003),
            Some(M.tray),
        );
        fbox!(
            f,
            "worktop",
            w,
            0.04,
            top_d,
            (0, h - 0.02, top_z),
            top,
            bevel = 0.01,
            bevel_segments = 2
        );
        // the fascia over the door: a display and a button
        fextrude!(
            f,
            "strip",
            &rounded_rect(dw - 0.03, 0.065, 0.008, 2),
            0.012,
            (mx, h - 0.075, front - 0.002),
            M.dark,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "display",
            0.08,
            0.022,
            0.004,
            (mx - dw / 2.0 + 0.12, h - 0.075, front + 0.005),
            M.display
        );
        fsphere!(
            f,
            "button",
            0.011,
            (mx + dw / 2.0 - 0.08, h - 0.075, front + 0.004),
            M.chrome,
            segments = 8,
            rings = 5
        );
        // inside the tub: the spray arm and the upper basket with cups upside down in it
        fbox!(
            f,
            "spray_arm",
            dw - 0.2,
            0.01,
            0.04,
            (mx, 0.18, tz),
            M.tray,
            bevel = 0.004,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "spray_hub",
            0.02,
            0.02,
            (mx, 0.175, tz),
            M.tray,
            verts = 10
        );
        basket(f, "upper", mx, 0.56, tz, dw - 0.14, tub_d - 0.06, 0.1, 3);
        for i in 0..3 {
            revolve!(
                format!("cup{i}"),
                &[
                    (0.0, 0.0),
                    (0.03, 0.0),
                    (0.034, 0.01),
                    (0.028, 0.06),
                    (0.0, 0.06)
                ],
                at = f.at(mx - 0.12 + 0.12 * i as f64, 0.625, tz),
                material = M.cup,
                segments = 10,
                rot = (180, 0, 0)
            );
        }
        // the cabinet beside it, its fronts flush with the door
        if drawers {
            for (k, (y0, y1)) in [(0.12, 0.41), (0.42, 0.71), (0.72, h - 0.07)]
                .into_iter()
                .enumerate()
            {
                shaker(
                    f,
                    &format!("drawer{k}"),
                    cw - 0.03,
                    y1 - y0,
                    (cx, (y0 + y1) / 2.0, front),
                    body,
                    0.02,
                    0.045,
                    0.004,
                );
                pull!(
                    f,
                    format!("drawer_pull{k}"),
                    (cx - (cw - 0.25) / 2.0, y1 - 0.045),
                    (cx + (cw - 0.25) / 2.0, y1 - 0.045),
                    front + 0.01,
                    out = 0.015,
                    w = 0.012,
                    r = None,
                    verts = 6
                );
            }
        } else {
            shaker(
                f,
                "drawer",
                cw - 0.03,
                0.12,
                (cx, h - 0.13, front),
                body,
                0.02,
                0.035,
                0.004,
            );
            pull!(
                f,
                "drawer_pull",
                (cx - (cw - 0.22) / 2.0, h - 0.13),
                (cx + (cw - 0.22) / 2.0, h - 0.13),
                front + 0.01,
                out = 0.015,
                w = 0.012,
                r = None,
                verts = 6
            );
            shaker(
                f,
                "cab_door",
                cw - 0.03,
                h - 0.33,
                (cx, 0.12 + (h - 0.33) / 2.0, front),
                body,
                0.02,
                0.06,
                0.004,
            );
            pull!(
                f,
                "cab_door_pull",
                (cx - cw / 2.0 + 0.06, h - 0.37),
                (cx - cw / 2.0 + 0.06, h - 0.25),
                front + 0.01,
                out = 0.015,
                w = 0.012,
                r = None,
                verts = 6
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the door, hinged on its bottom edge (its bottom at y 0.12): a steel front with a bar handle, its inner panel
    // and the detergent flap on the back
    let parts = new_parts(|| {
        soft!(
            f,
            "door",
            dw - 0.02,
            door_h,
            0.02,
            (mx, 0.12 + door_h / 2.0, front),
            panel,
            round = 0.006,
            round_segments = 1
        );
        pull!(
            f,
            "handle",
            (mx - (dw - 0.2) / 2.0, 0.12 + door_h - 0.05),
            (mx + (dw - 0.2) / 2.0, 0.12 + door_h - 0.05),
            front + 0.01,
            out = 0.02,
            w = 0.016,
            r = None,
            verts = 8
        );
        fbox!(
            f,
            "door_inner",
            dw - 0.1,
            door_h - 0.08,
            0.006,
            (mx, 0.12 + door_h / 2.0, front - 0.012),
            M.tray
        );
        fbox!(
            f,
            "detergent",
            0.12,
            0.07,
            0.008,
            (mx + 0.05, 0.12 + door_h * 0.6, front - 0.016),
            M.plate,
            bevel = 0.003,
            bevel_segments = 1
        );
    });
    let door = hinged(join(&parts, "door", None), f.at(mx, 0.12, front));
    // the lower basket of plates standing in a row and a bowl, sliding out to the front
    let parts = new_parts(|| {
        basket(
            f,
            "rack",
            mx,
            0.27,
            top_z - 0.01,
            dw - 0.14,
            top_d - 0.26,
            0.12,
            4,
        );
        for i in 0..6 {
            fcyl!(
                f,
                format!("plate{i}"),
                0.1,
                0.008,
                (mx - 0.1 + 0.035 * i as f64, 0.375, top_z - 0.06),
                M.plate,
                verts = 16,
                rot = (0, 90, 0)
            );
        }
        revolve!(
            "bowl",
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.07, 0.05),
                (0.064, 0.05),
                (0.036, 0.006),
                (0.0, 0.006)
            ],
            at = f.at(mx + 0.15, 0.275, top_z + 0.06),
            material = M.plate,
            segments = 12
        );
    });
    let rack = join(&parts, "rack", Some(f.at(mx, 0.30, top_z - 0.01)));
    settle_all();
    swing!(door, "door_open", 85, seconds = SECONDS, axis = Axis::X);
    slide!(rack, "door_open", (0, -0.30, 0), seconds = SECONDS);
}

/// A 60 cm dishwasher beside a 40 cm cupboard (a drawer over a door).
pub fn dishwasher() -> Vec<Obj> {
    let f = &GameFrame::new(0.1375);
    dishwasher_unit(
        f, 1.0, 0.95, 0.66, -0.15, 0.175, 0.72, M.cab, M.steel, M.cab, 0.6, false,
    );
    Vec::new()
}

/// A slimline 45 cm dishwasher beside a 55 cm stack of three drawers.
pub fn dishwasher_compact() -> Vec<Obj> {
    let f = &GameFrame::with_depths(0.085, &crate::kitchen_cooking::COUNTER66);
    dishwasher_unit(
        f, 1.0, 0.95, 0.79, -0.085, 0.25, 0.66, M.cab, M.steel, M.cab, 0.45, true,
    );
    Vec::new()
}
