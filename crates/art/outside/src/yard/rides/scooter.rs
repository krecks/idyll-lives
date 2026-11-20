//! The child's scooter and the electric scooter.

use crate::yard::common::{F, M, arc_points, board};
use artkit::far::{is_low, low};
use artkit::forms::rounded_rect;
use artkit::math::{P3, cosd, sind};
use artkit::parts::{Settle, join, join_all, new_parts, numbered, settle_with};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, revolve, soft, torus};

// The e-scooter: a low deck between two small solid wheels, a raked stem from the front up to T-bars.
const SC_R: f64 = 0.105; // its wheels' radius
const SC_REAR: f64 = -0.36;
const SC_FRONT: f64 = 0.37;
const SC_DECK: (f64, f64, f64) = (-0.25, 0.25, 0.125); // the deck's ends (x) and its top (with the grip tape)
const SC_TOP: (f64, f64) = (0.284, 1.015); // the stem's top, under the bars
const SC_STAND: (f64, f64, f64) = (0.1, 0.072, 0.07); // the side stand's pivot, under the deck's left edge (+z: the rider's left)

/// The point of the steering axis (front axle up to the stem's top) at height `h`: (x, h).
fn sc_axis(h: f64) -> (f64, f64) {
    let t = (h - SC_R) / (SC_TOP.1 - SC_R);
    (SC_FRONT + (SC_TOP.0 - SC_FRONT) * t, h)
}

/// A scooter wheel at axle (x, SC_R, 0): a solid tyre, a rim with five spokes round a hub, or the hub motor's drum
/// filling it (`motor`); the back one's brake disc on its left (+z).
fn sc_wheel(x: f64, motor: bool) {
    let y = SC_R;
    torus!(
        numbered("tyre"),
        SC_R - 0.025,
        0.025,
        at = F.at(x, y, 0),
        material = M.tyre,
        rot = (90, 0, 0),
        major_segments = low(32, 12),
        minor_segments = low(8, 4)
    );
    let rim = SC_R - 0.048;
    if motor {
        // the motor: a drum as wide as the tyre, a lip either side
        revolve!(
            numbered("motor"),
            &[
                (0.0, -0.034),
                (0.03, -0.034),
                (0.046, -0.03),
                (0.054, -0.024),
                (rim + 0.004, -0.02),
                (rim + 0.004, 0.02),
                (0.054, 0.024),
                (0.046, 0.03),
                (0.03, 0.034),
                (0.0, 0.034)
            ],
            at = F.at(x, y, 0),
            material = M.scooter_metal,
            segments = low(20, 8),
            rot = (90, 0, 0)
        );
        if !is_low() {
            for s in [-1.0, 1.0] {
                fcyl!(
                    F,
                    numbered("motor_cap"),
                    0.022,
                    0.006,
                    (x, y, s * 0.036),
                    M.scooter_trim,
                    rot = (90, 0, 0),
                    verts = 10
                );
            }
        }
    } else {
        revolve!(
            numbered("rim"),
            &[
                (rim - 0.012, -0.02),
                (rim + 0.004, -0.02),
                (rim + 0.004, 0.02),
                (rim - 0.012, 0.02)
            ],
            at = F.at(x, y, 0),
            material = M.scooter_metal,
            segments = low(24, 8),
            rot = (90, 0, 0)
        );
        fcyl!(
            F,
            numbered("hub"),
            0.02,
            0.05,
            (x, y, 0),
            M.scooter_metal,
            rot = (90, 0, 0),
            verts = low(12, 6)
        );
        if !is_low() {
            for i in 0..5 {
                let a = 72 * i + 18;
                let m = 0.5 * (0.02 + rim - 0.012);
                fbox!(
                    F,
                    numbered("rim_spoke"),
                    rim - 0.012 - 0.015,
                    0.012,
                    0.016,
                    (x + m * cosd(a as f64), y + m * sind(a as f64), 0),
                    M.scooter_metal,
                    rot = (0, a, 0)
                );
            }
        }
        fcyl!(
            F,
            numbered("brake_disc"),
            0.05,
            0.003,
            (x, y, 0.03),
            M.rim,
            rot = (90, 0, 0),
            verts = low(16, 8)
        );
    }
    fcyl!(
        F,
        numbered("axle"),
        0.009,
        0.1,
        (x, y, 0),
        M.rim,
        rot = (90, 0, 0),
        verts = low(8, 4)
    );
}

/// The scooter's still body: deck and grip tape, the back fork, the fenders, the neck and the head tube, the fork,
/// the stem with its latch, the bars and grips, a brake lever and a throttle, a bell, the display, the lamps.
fn sc_frame() {
    let (x0, x1, top) = SC_DECK;
    let l = x1 - x0;
    // the deck: a rounded slab, its edges rounded, grip tape on top, a trim strip along each side
    fextrude!(
        F,
        "deck",
        &rounded_rect(l, 0.165, 0.05, low(4, 1)),
        0.058,
        ((x0 + x1) / 2.0, top - 0.004 - 0.029, 0),
        M.scooter,
        plane = Facing::Top,
        bevel = low(0.012, 0.0),
        bevel_segments = low(2, 1)
    );
    fextrude!(
        F,
        "grip_tape",
        &rounded_rect(l - 0.04, 0.135, 0.035, low(4, 1)),
        0.004,
        ((x0 + x1) / 2.0, top - 0.002, 0),
        M.grip_tape,
        plane = Facing::Top
    );
    if !is_low() {
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                numbered("deck_trim"),
                l - 0.08,
                0.008,
                0.004,
                ((x0 + x1) / 2.0, top - 0.03, s * 0.0835),
                M.scooter_trim
            );
        }
    }
    // the back fork: a plate either side from the deck's tail to the axle; the fender over the wheel, a light at its end
    for s in [-1.0, 1.0] {
        board(
            &numbered("rear_fork"),
            (x0 + 0.03, top - 0.03, s * 0.042),
            (SC_REAR, SC_R, s * 0.042),
            0.04,
            0.008,
            M.scooter,
            (0, 0, 1),
            low(0.003, 0.0),
        );
    }
    let prof = vec![
        (-0.032, 0.0),
        (-0.028, 0.008),
        (0.0, 0.011),
        (0.028, 0.008),
        (0.032, 0.0),
        (0.0, 0.004),
    ];
    for (x, (a0, a1)) in [(SC_REAR, (28.0, 152.0)), (SC_FRONT, (45.0, 140.0))] {
        fsweep!(
            F,
            numbered("fender"),
            &arc_points((x, SC_R, 0.0), SC_R + 0.016, a0, a1, low(10, 4), true),
            0,
            M.scooter,
            profile = &prof,
            up = (0, 1, 0)
        );
    }
    let (ex, ey) = (
        SC_REAR + (SC_R + 0.016) * cosd(152.0),
        SC_R + (SC_R + 0.016) * sind(152.0),
    );
    fbox!(
        F,
        "tail_light",
        0.012,
        0.022,
        0.04,
        (ex - 0.004, ey + 0.004, 0),
        M.reflector,
        bevel = low(0.003, 0.0),
        bevel_segments = 1
    );
    // the neck from the deck's front up to the head tube; the head tube, the fork blades to the axle
    let (hx0, hy0) = sc_axis(0.2);
    let (hx1, hy1) = sc_axis(0.32);
    fsweep!(
        F,
        "neck",
        &[
            (x1 - 0.06, top - 0.03, 0),
            (x1 + 0.03, top - 0.015, 0),
            (hx0 - 0.01, hy0 + 0.01, 0)
        ],
        0.024,
        M.scooter,
        verts = low(10, 6),
        resolution = low(3, 1),
        scale = [1.0, 0.95, 0.85]
    );
    fsweep!(
        F,
        "head_tube",
        &[(hx0, hy0, 0), (hx1, hy1, 0)],
        0.026,
        M.scooter,
        verts = low(12, 6)
    );
    let (cx, cy) = sc_axis(0.19);
    for s in [-1.0, 1.0] {
        fsweep!(
            F,
            numbered("fork"),
            &[(cx, cy, s * 0.02), (SC_FRONT, SC_R, s * 0.042)],
            0.009,
            M.scooter,
            verts = low(6, 4),
            scale = [1.2, 0.9]
        );
    }
    fbox!(
        F,
        "fork_crown",
        0.04,
        0.025,
        0.1,
        (cx, cy, 0),
        M.scooter,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    // the stem: the folding latch over the head tube, then the stem up to the bars
    let (lx, ly) = sc_axis(0.36);
    let mid = sc_axis(0.6);
    fsweep!(
        F,
        "stem",
        &[(hx1, hy1, 0), (mid.0, mid.1, 0), (SC_TOP.0, SC_TOP.1, 0)],
        0.019,
        M.scooter,
        verts = low(12, 6),
        scale = [1.0, 0.95, 0.88]
    );
    fbox!(
        F,
        "latch",
        0.05,
        0.07,
        0.05,
        (lx + 0.012, ly, 0),
        M.scooter_trim,
        rot = (0, 5, 0),
        bevel = low(0.01, 0.0),
        bevel_segments = low(2, 1)
    );
    fcyl!(
        F,
        "latch_hinge",
        0.014,
        0.07,
        (lx + 0.02, ly - 0.03, 0),
        M.rim,
        rot = (90, 0, 0),
        verts = low(8, 4)
    );
    // the T-bars and grips; a brake lever on the left (+z), a thumb throttle on the right, a bell
    let (bx, by) = (SC_TOP.0, SC_TOP.1 + 0.015);
    fsweep!(
        F,
        "bars",
        &[(bx, by, -0.29), (bx, by, 0.29)],
        0.012,
        M.scooter,
        verts = low(10, 5)
    );
    fbox!(
        F,
        "bar_clamp",
        0.04,
        0.04,
        0.06,
        (bx, by - 0.005, 0),
        M.scooter,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    for s in [-1.0, 1.0] {
        fsweep!(
            F,
            numbered("grip"),
            &[(bx, by, s * 0.17), (bx, by, s * 0.29)],
            0.017,
            M.grip_tape,
            verts = low(10, 5),
            scale = [1.0, 1.08]
        );
        fcyl!(
            F,
            numbered("grip_end"),
            0.018,
            0.008,
            (bx, by, s * 0.294),
            M.scooter_trim,
            rot = (90, 0, 0),
            verts = low(10, 5)
        );
    }
    fsweep!(
        F,
        "brake_lever",
        &[
            (bx + 0.01, by, 0.15),
            (bx + 0.035, by + 0.002, 0.2),
            (bx + 0.04, by, 0.26)
        ],
        0.005,
        M.rim,
        verts = low(6, 4),
        resolution = low(2, 1)
    );
    fbox!(
        F,
        "brake_mount",
        0.03,
        0.03,
        0.025,
        (bx + 0.006, by, 0.155),
        M.scooter,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "throttle",
        0.035,
        0.012,
        0.025,
        (bx - 0.018, by + 0.004, -0.155),
        M.scooter,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        "bell",
        0.015,
        0.012,
        (bx, by + 0.016, 0.11),
        M.rim,
        verts = low(8, 5)
    );
    // the display on the bars' clamp, tilted up to the rider; the headlight on the stem's front
    soft!(
        F,
        "display",
        0.06,
        0.022,
        0.075,
        (bx - 0.012, by + 0.024, 0),
        M.scooter,
        round = 0.008,
        flat = 1,
        round_segments = low(2, 1)
    );
    fbox!(
        F,
        "screen",
        0.04,
        0.003,
        0.05,
        (bx - 0.014, by + 0.0355, 0),
        M.screen
    );
    let (hx, hy) = sc_axis(0.9);
    fcyl!(
        F,
        "headlight",
        0.022,
        0.03,
        (hx + 0.03, hy, 0),
        M.scooter,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    fcyl!(
        F,
        "headlight_glass",
        0.018,
        0.004,
        (hx + 0.046, hy, 0),
        M.lamp,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    fbox!(
        F,
        "headlight_mount",
        0.03,
        0.02,
        0.025,
        (hx + 0.012, hy, 0),
        M.scooter
    );
    if !is_low() {
        // the motor's cable down the fork, the brake's down the stem
        fsweep!(
            F,
            "motor_cable",
            &[
                (SC_FRONT - 0.01, SC_R + 0.02, 0.05),
                (cx + 0.02, cy, 0.04),
                (hx1 + 0.03, hy1, 0.02),
                (hx1 + 0.02, hy1 + 0.1, 0.0)
            ],
            0.004,
            M.tyre,
            verts = 4,
            resolution = 2
        );
        let end = sc_axis(0.7);
        fsweep!(
            F,
            "brake_cable",
            &[
                (bx + 0.01, by - 0.01, 0.15),
                (bx + 0.05, by - 0.12, 0.04),
                (end.0, end.1, 0.0)
            ],
            0.003,
            M.tyre,
            verts = 4,
            resolution = 2
        );
    }
}

/// The side stand, down: a short leg from its pivot under the deck's left edge out and back to a foot.
fn sc_stand() {
    let (px, py, pz) = SC_STAND;
    fbox!(
        F,
        "stand_mount",
        0.04,
        0.02,
        0.02,
        (px, py + 0.006, pz - 0.008),
        M.scooter,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fsweep!(
        F,
        "stand_leg",
        &[
            (px, py, pz),
            (px - 0.07, 0.03, pz + 0.07),
            (px - 0.1, 0.012, pz + 0.105)
        ],
        0.008,
        M.scooter,
        verts = low(8, 4),
        resolution = low(2, 1),
        scale = [1.0, 0.9, 0.85]
    );
    fbox!(
        F,
        "stand_foot",
        0.035,
        0.012,
        0.03,
        (px - 0.104, 0.006, pz + 0.11),
        M.tyre,
        bevel = low(0.003, 0.0),
        bevel_segments = 1
    );
}

pub fn e_scooter() -> Vec<Obj> {
    let frame = new_parts(sc_frame);
    let front = new_parts(|| sc_wheel(SC_FRONT, true));
    let rear = new_parts(|| sc_wheel(SC_REAR, false));
    let kick = new_parts(sc_stand);
    let how = Settle {
        centre: false,
        floor: true,
    };
    if is_low() {
        return settle_with(&[join_all("eScooter")], how);
    }
    join(&frame, "frame", Some(P3::from((0, 0, 0))));
    join(&front, "wheel_front", Some(F.at(SC_FRONT, SC_R, 0)));
    join(&rear, "wheel_rear", Some(F.at(SC_REAR, SC_R, 0)));
    join(
        &kick,
        "stand",
        Some(F.at(SC_STAND.0, SC_STAND.1, SC_STAND.2)),
    );
    settle_with(&meshes(), how)
}
