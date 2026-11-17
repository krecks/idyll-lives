//! The top-freezer fridge: a cabinet and two doors, each door its own node turning on its hinge, with a clip per
//! door: the game opens the fridge door while a resident takes food or a drink out (`motions`). At rest both
//! are closed. Built to the old part's size, in its numbers: the parts are placed in game axes (`F`).

use super::{DOOR_SECONDS, F, FREEZER_ANGLE, FRIDGE_ANGLE, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcone, fcyl, fextrude, fsphere, fsweep, loft, ring, soft, swing};

/// The cabinet as five panels, so the doors open onto a real interior instead of a solid block. The sides are
/// rounded on their vertical edges, as a pressed-steel body is; the top's edges are rounded to meet them.
fn shell() {
    let side = rounded_rect(0.03, 0.622, 0.012, 2);
    fextrude!(
        F,
        "side_left",
        &side,
        1.74,
        (-0.355, 0.93, -0.144),
        M.body,
        plane = Facing::Top,
        bevel = 0.004,
        bevel_segments = 1
    );
    fextrude!(
        F,
        "side_right",
        &side,
        1.74,
        (0.355, 0.93, -0.144),
        M.body,
        plane = Facing::Top,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        "top",
        0.74,
        0.03,
        0.622,
        (0, 1.785, -0.144),
        M.body,
        bevel = 0.012,
        bevel_segments = 2
    );
    fbox!(
        F,
        "bottom",
        0.74,
        0.03,
        0.622,
        (0, 0.075, -0.144),
        M.body,
        bevel = 0.006
    );
    fbox!(
        F,
        "back",
        0.74,
        1.74,
        0.03,
        (0, 0.93, -0.44),
        M.body,
        bevel = 0.006
    );
    // the front of the plinth: a vented kick plate over a dark recess, and the hinges' covers on the right
    fbox!(
        F,
        "kick_recess",
        0.66,
        0.05,
        0.01,
        (0, 0.035, 0.108),
        M.dark
    );
    let slot = rounded_rect(0.07, 0.012, 0.006, 1);
    let slots: Vec<Vec<(f64, f64)>> = (0..6)
        .map(|i| {
            slot.iter()
                .map(|&(u, v)| (u + (i as f64 - 2.5) * 0.1, v))
                .collect()
        })
        .collect();
    fextrude!(
        F,
        "kick_plate",
        &rounded_rect(0.68, 0.05, 0.008, 1),
        0.008,
        (0, 0.035, 0.117),
        M.seal,
        holes = slots
    );
    for y in [1.224, 0.08] {
        fbox!(
            F,
            format!("hinge{}", float_repr(y)),
            0.024,
            0.026,
            0.05,
            (0.356, y, 0.19),
            M.trim,
            bevel = 0.006,
            bevel_segments = 1
        );
    }
}

/// The liner, shelves with trimmed fronts, two crisper drawers and food, seen once the doors are open. The
/// shelves stop short of the door's bins.
fn interior() {
    fbox!(
        F,
        "liner_back",
        0.678,
        1.678,
        0.006,
        (0, 0.93, -0.421),
        M.liner,
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        F,
        "liner_left",
        0.006,
        1.678,
        0.578,
        (-0.337, 0.93, -0.134),
        M.liner,
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        F,
        "liner_right",
        0.006,
        1.678,
        0.578,
        (0.337, 0.93, -0.134),
        M.liner,
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        F,
        "liner_top",
        0.678,
        0.006,
        0.578,
        (0, 1.767, -0.134),
        M.liner,
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        F,
        "liner_bottom",
        0.678,
        0.006,
        0.578,
        (0, 0.093, -0.134),
        M.liner,
        bevel = 0.002,
        bevel_segments = 1
    );
    // the divider between the freezer (top) and the fridge (bottom), the freezer's floor
    fbox!(
        F,
        "divider",
        0.672,
        0.02,
        0.55,
        (0, 1.234, -0.13),
        M.shelf,
        bevel = 0.002,
        bevel_segments = 1
    );
    // three shelves in the fridge, one in the freezer, each with a trim along its front
    for (i, y) in [0.37, 0.66, 0.95, 1.52].into_iter().enumerate() {
        fbox!(
            F,
            format!("shelf{i}"),
            0.672,
            0.008,
            0.49,
            (0, y, -0.16),
            M.shelf
        );
        fbox!(
            F,
            format!("shelf_trim{i}"),
            0.672,
            0.022,
            0.01,
            (0, y - 0.004, 0.083),
            M.trim,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    // two crisper drawers under the lowest shelf, a grip cut along each front's top
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            format!("crisper{s}"),
            0.322,
            0.24,
            0.46,
            (s * 0.166, 0.226, -0.16),
            M.shelf
        );
        let grip: Vec<(f64, f64)> = rounded_rect(0.14, 0.022, 0.011, 2)
            .iter()
            .map(|&(u, v)| (u, v + 0.09))
            .collect();
        fextrude!(
            F,
            format!("crisper_front{s}"),
            &rounded_rect(0.322, 0.25, 0.012, 1),
            0.012,
            (s * 0.166, 0.226, 0.076),
            M.carton,
            holes = [grip]
        );
    }
    // the lamp's housing and the thermostat's dial under the divider
    fbox!(
        F,
        "lamp",
        0.16,
        0.035,
        0.09,
        (0.17, 1.205, -0.2),
        M.liner,
        bevel = 0.008,
        bevel_segments = 1
    );
    fcyl!(
        F,
        "dial",
        0.022,
        0.014,
        (0.17, 1.18, -0.2),
        M.trim,
        verts = 12
    );
    fridge_food();
    freezer_food();
}

/// Normal food, in the bottom (fridge) compartment: dairy, produce and eggs.
fn fridge_food() {
    let (top, mid, low) = (0.954, 0.664, 0.374); // the fridge shelf surfaces
    fbox!(
        F,
        "milk",
        0.085,
        0.20,
        0.085,
        (-0.20, top + 0.10, -0.14),
        M.carton,
        bevel = 0.006
    );
    fcone!(
        F,
        "milk_gable",
        0.062,
        0.0,
        0.055,
        (-0.20, top + 0.2275, -0.14),
        M.carton,
        verts = 4,
        rot = (0, 0, 45)
    );
    bottle(
        "juice",
        (0.02, top, -0.14),
        0.032,
        0.22,
        M.bottle,
        M.carton,
        10,
        F,
    );
    for (i, x) in [-0.20, -0.11, -0.02].into_iter().enumerate() {
        fsphere!(
            F,
            format!("apple{i}"),
            0.038,
            (x, mid + 0.038, -0.15),
            M.produce,
            segments = 10,
            rings = 6
        );
    }
    fsphere!(
        F,
        "orange",
        0.042,
        (0.13, mid + 0.042, -0.15),
        M.produce2,
        segments = 10,
        rings = 6
    );
    fbox!(
        F,
        "egg_carton",
        0.16,
        0.03,
        0.11,
        (-0.16, low + 0.015, -0.14),
        M.carton,
        bevel = 0.006
    );
    for (i, x) in [-0.205, -0.155, -0.105].into_iter().enumerate() {
        fsphere!(
            F,
            format!("egg{i}"),
            0.017,
            (x, low + 0.03 + 0.013, -0.16),
            M.egg,
            segments = 8,
            rings = 5
        );
    }
    fbox!(
        F,
        "tub",
        0.13,
        0.06,
        0.13,
        (0.14, low + 0.03, -0.14),
        M.shelf,
        bevel = 0.01
    );
}

/// Frozen food, in the top (freezer) compartment: a pizza, ice cream and peas.
fn freezer_food() {
    let (floor, shelf) = (1.244, 1.524); // the divider's top and the freezer shelf
    fbox!(
        F,
        "pizza",
        0.26,
        0.035,
        0.26,
        (-0.13, floor + 0.0175, -0.14),
        M.frozen,
        bevel = 0.006
    );
    fbox!(
        F,
        "ice_cream",
        0.12,
        0.10,
        0.12,
        (0.14, shelf + 0.05, -0.14),
        M.icecream,
        bevel = 0.01
    );
    fbox!(
        F,
        "peas",
        0.11,
        0.15,
        0.06,
        (-0.14, shelf + 0.075, -0.16),
        M.peas,
        bevel = 0.008
    );
}

/// A bottle standing at game point `foot`: a round body, a shoulder, a neck and its cap.
#[allow(clippy::too_many_arguments)]
fn bottle(
    name: &str,
    foot: (f64, f64, f64),
    r: f64,
    h: f64,
    material: Mat,
    cap: Mat,
    verts: usize,
    frame: GameFrame,
) {
    let at = frame.at(foot.0, foot.1, foot.2);
    let (bx, by, bz) = (at.x, at.y, at.z);
    let rings = [
        (0.0, 0.9),
        (0.025, 1.0),
        (0.62, 1.0),
        (0.78, 0.5),
        (0.86, 0.36),
        (0.94, 0.36),
    ];
    let sections: Vec<_> = rings
        .iter()
        .map(|&(t, k)| ring!(r * k, r * k, z = bz + h * t, verts = verts, at = (bx, by)))
        .collect();
    loft!(name, &sections, material = material);
    fcyl!(
        frame,
        format!("{name}_cap"),
        r * 0.4,
        h * 0.06,
        (foot.0, foot.1 + h * 0.97, foot.2),
        cap,
        verts = verts
    );
}

/// A bin on a door's inside: a tray with a lip along its inner edge, from game x0 to x1, z0 (inner) to z1.
fn door_bin(name: &str, y: f64, x0: f64, x1: f64, z0: f64, z1: f64, frame: GameFrame) {
    let (w, d) = (x1 - x0, z1 - z0);
    fbox!(
        frame,
        format!("{name}_tray"),
        w,
        0.01,
        d,
        ((x0 + x1) / 2.0, y + 0.005, (z0 + z1) / 2.0),
        M.shelf
    );
    fbox!(
        frame,
        format!("{name}_lip"),
        w,
        0.05,
        0.008,
        ((x0 + x1) / 2.0, y + 0.025, z0 + 0.004),
        M.shelf,
        bevel = 0.003,
        bevel_segments = 1
    );
    for s in [x0, x1] {
        fbox!(
            frame,
            format!("{name}_end{}", float_repr(s)),
            0.008,
            0.05,
            d,
            (s, y + 0.025, (z0 + z1) / 2.0),
            M.shelf
        );
    }
}

/// A bar handle in one piece: out from the door face at `face` on rounded stand-offs, up, and back in.
fn handle(name: &str, x: f64, y0: f64, y1: f64, face: f64) {
    let out = face + 0.041;
    fsweep!(
        F,
        name,
        &[
            (x, y0, face - 0.004),
            (x, y0, out - 0.012),
            (x, y0 + 0.012, out),
            (x, y1 - 0.012, out),
            (x, y1, out - 0.012),
            (x, y1, face - 0.004)
        ],
        0.0,
        M.chrome,
        profile = rounded_rect(0.016, 0.022, 0.007, 1),
        up = (1, 0, 0),
        resolution = 2,
        angle = 60
    );
}

/// A door, rounded and proud of the opening, its handle on the right and its hinge on the left as seen from
/// the front: a dark seal round its back, an inner liner and the bins on its inside (`bins`: their heights).
fn door(
    name: &str,
    y: f64,
    h: f64,
    handle_y0: f64,
    handle_y1: f64,
    badge: bool,
    bins: &[f64],
) -> Obj {
    let parts = new_parts(|| {
        soft!(
            F,
            name,
            0.728,
            h,
            0.04,
            (0, y, 0.195),
            M.body,
            round = 0.014
        );
        handle(&format!("handle_{name}"), -0.3, handle_y0, handle_y1, 0.215);
        if badge {
            soft!(
                F,
                "badge",
                0.1,
                0.018,
                0.006,
                (0.25, 1.74, 0.216),
                M.trim,
                round = 0.003,
                round_segments = 1
            );
        }
        let seal = rounded_rect(0.7, h - 0.024, 0.03, 2);
        let path: Vec<(f64, f64, f64)> = seal.iter().map(|&(u, v)| (u, y + v, 0.172)).collect();
        fsweep!(
            F,
            format!("{name}_seal"),
            &path,
            0.009,
            M.gasket,
            verts = 5,
            closed = true
        );
        fbox!(
            F,
            format!("{name}_liner"),
            0.64,
            h - 0.08,
            0.008,
            (0, y, 0.172),
            M.liner
        );
        for (i, &by) in bins.iter().enumerate() {
            door_bin(&format!("{name}_bin{i}"), by, -0.3, 0.3, 0.1, 0.168, F);
        }
    });
    hinged(join(&parts, name, None), F.at(0.364, y, 0.195))
}

pub fn fridge() -> Vec<Obj> {
    // the cabinet on a recessed plinth, opening onto a liner, shelves, drawers and food: the part that stays still
    let cabinet = new_parts(|| {
        fbox!(
            F,
            "plinth",
            0.68,
            0.06,
            0.57,
            (0, 0.03, -0.17),
            M.trim,
            bevel = 0.006
        );
        shell();
        interior();
        fbox!(
            F,
            "gap",
            0.7,
            0.014,
            0.03,
            (0, 1.224, 0.187),
            M.seal,
            bevel = 0.004
        );
    });
    join(&cabinet, "cabinet", Some((0, 0, 0).into()));
    // the freezer door over the fridge door, a clip each; the fridge door's bins hold bottles, jars and butter
    let freezer = door("door_freezer", 1.51, 0.556, 1.274, 1.524, true, &[1.33]);
    let stock = new_parts(|| {
        fbox!(
            F,
            "ice_tray",
            0.24,
            0.035,
            0.05,
            (-0.12, 1.358, 0.134),
            M.frozen,
            bevel = 0.006,
            bevel_segments = 1
        );
    });
    let fridge_door = door(
        "door_fridge",
        0.643,
        1.146,
        0.804,
        1.164,
        false,
        &[0.2, 0.55, 0.9],
    );
    let stock2 = new_parts(|| {
        for (i, (x, mat)) in [(-0.22, M.bottle), (-0.13, M.wine2), (-0.04, M.produce2)]
            .into_iter()
            .enumerate()
        {
            bottle(
                &format!("door_bottle{i}"),
                (x, 0.21, 0.134),
                0.03,
                0.27 - 0.03 * i as f64,
                mat,
                M.carton,
                10,
                F,
            );
        }
        for (i, x) in [0.1, 0.19].into_iter().enumerate() {
            fcyl!(
                F,
                format!("jar{i}"),
                0.034,
                0.09,
                (x, 0.605, 0.134),
                if i != 0 { M.produce } else { M.egg },
                verts = 10,
                bevel = 0.004,
                bevel_segments = 1
            );
            fcyl!(
                F,
                format!("jar_lid{i}"),
                0.036,
                0.016,
                (x, 0.658, 0.134),
                M.trim,
                verts = 10
            );
        }
        fbox!(
            F,
            "butter",
            0.12,
            0.05,
            0.05,
            (-0.15, 0.935, 0.134),
            M.egg,
            bevel = 0.008,
            bevel_segments = 1
        );
    });
    // the door's stock moves with it
    let freezer = join(&[vec![freezer], stock].concat(), "door_freezer", None);
    let fridge_door = join(&[vec![fridge_door], stock2].concat(), "door_fridge", None);
    settle_all(); // where the game places it (`centre`), its back on the wall
    swing!(
        freezer,
        "freezer_open",
        FREEZER_ANGLE,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(
        fridge_door,
        "fridge_open",
        FRIDGE_ANGLE,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    Vec::new()
}
