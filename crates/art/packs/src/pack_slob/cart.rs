//! The late-night microwave cart (1x1): a rolling cart, a microwave on top, frozen dinners below.

use super::common::{_box, _can, _wrapper, M, POP, cut_pocket};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{P3, PI, cos, sin};
use artkit::motion::Ease;
use artkit::parts::{aside, hinged, join, new_parts, settle};
use artkit::place::GameFrame;
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, loft, revolve, ring, soft, swing};

// (x, y, z0, w, h, d)
const MW: (f64, f64, f64, f64, f64, f64) = (-0.02, 0.875, -0.42, 0.44, 0.29, 0.36);

/// A castor: a swivel plate, a fork and a tyred wheel.
fn _castor(f: &GameFrame, name: &str, x: f64, z: f64) {
    fbox!(
        f,
        format!("{name}_plate"),
        0.04,
        0.008,
        0.04,
        (x, 0.074, z),
        M.chrome,
        bevel = 0.003,
        bevel_segments = 1
    );
    fsweep!(
        f,
        format!("{name}_fork"),
        &[(x - 0.018, 0.07, z), (x - 0.018, 0.035, z + 0.008)],
        0.0,
        M.chrome,
        profile = [
            (-0.002, -0.008),
            (0.002, -0.008),
            (0.002, 0.008),
            (-0.002, 0.008)
        ],
        up = (1, 0, 0)
    );
    revolve!(
        format!("{name}_wheel"),
        &[
            (0.0, -0.01),
            (0.018, -0.01),
            (0.026, -0.006),
            (0.027, 0.0),
            (0.026, 0.006),
            (0.018, 0.01),
            (0.0, 0.01)
        ],
        at = f.at(x, 0.027, z + 0.008),
        material = M.tyre,
        segments = 12,
        rot = (0, 90, 0)
    );
}

pub fn microwave_cart() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(4);
    let (x, y, z0, w, h, d) = MW;
    let (zf, pw) = (z0 + d, 0.12);
    let still = new_parts(|| {
        for sy in [0.86, 0.42, 0.1] {
            // three trays, each with a raised lip
            let (tw, td) = if sy > 0.5 { (0.7, 0.46) } else { (0.66, 0.42) };
            let b = f.at(0, sy - 0.015, -0.22);
            // the lip's top rounded over in 30° steps, 9 mm thick
            let mut lip = vec![(0.0, 0.98), (0.006, 1.0)];
            for i in 0..=6 {
                let a = PI * i as f64 / 6.0;
                lip.push((
                    0.0505 + 0.0045 * sin(a),
                    1.0 - 0.0045 * (1.0 - cos(a)) / (tw / 2.0),
                ));
            }
            lip.extend([(0.012, 0.975), (0.012, 0.0)]);
            let sections: Vec<Vec<P3>> = lip
                .iter()
                .map(|&(hh, k)| {
                    ring!(
                        tw / 2.0 * k,
                        td / 2.0 * k,
                        z = b.z + hh,
                        verts = 20,
                        square = 8,
                        at = (b.x, b.y)
                    )
                })
                .collect();
            loft!(format!("tray{sy}"), &sections, material = M.white);
            fbox!(
                f,
                format!("tray_band{sy}"),
                tw - 0.04,
                0.02,
                0.006,
                (0, sy + 0.012, -0.22 + td / 2.0 + 0.001),
                M.body
            );
        }
        for cx in [-0.33, 0.33] {
            for cz in [-0.42, -0.02] {
                fsweep!(
                    f,
                    format!("post{cx}{cz}"),
                    &[(cx, 0.075, cz), (cx, 0.9, cz)],
                    0.016,
                    M.body,
                    verts = 10
                );
                _castor(f, &format!("castor{cx}{cz}"), cx, cz);
            }
        }
        kit::tube!(
            f,
            "handle",
            &kit::fillet!(
                &[
                    (0.33, 0.9, -0.4),
                    (0.37, 1.01, -0.4),
                    (0.37, 1.01, -0.04),
                    (0.33, 0.9, -0.04),
                ],
                0.03
            ),
            0.012,
            M.chrome,
            verts = 8,
            round = 1
        );
        fsweep!(
            f,
            "handle_grip",
            &[(0.37, 1.01, -0.33), (0.37, 1.01, -0.11)],
            0.016,
            M.tyre,
            verts = 10
        );
        // the microwave: its case with a cavity cut in (a turntable and a bowl inside), the keypad panel, feet
        let case = _box(
            f,
            "mw_case",
            w,
            h - 0.012,
            d - 0.01,
            (x, y + 0.006 + (h - 0.012) / 2.0, z0 + (d - 0.01) / 2.0),
            M.gloss_white,
            10.0,
            24,
            0.01,
        );
        let cav = (w - pw - 0.06, h - 0.07, d - 0.08);
        cut_pocket(
            case,
            fbox!(
                f,
                "mw_cut",
                cav.0,
                cav.1,
                cav.2 + 0.02,
                (x - pw / 2.0 - 0.01, y + h / 2.0, zf - cav.2 / 2.0),
                M.dark
            ),
        );
        // the cavity's rolled front flange, lapping the mouth's edge by 4 mm
        fextrude!(
            f,
            "mw_flange",
            &rounded_rect(cav.0 + 0.012, cav.1 + 0.012, 0.006, 2),
            0.003,
            (x - pw / 2.0 - 0.01, y + h / 2.0, zf - 0.0085), // (the case's front is 1 cm behind `zf`)
            M.gloss_white,
            holes = [rounded_rect(cav.0 - 0.008, cav.1 - 0.008, 0.003, 1)]
        );
        fbox!(
            f,
            "mw_cavity",
            cav.0,
            cav.1,
            0.004,
            (x - pw / 2.0 - 0.01, y + h / 2.0, zf - cav.2 - 0.004),
            M.panel
        );
        fcyl!(
            f,
            "mw_plate",
            0.1,
            0.006,
            (x - pw / 2.0 - 0.01, y + 0.042, zf - cav.2 / 2.0),
            M.panel,
            verts = 16
        );
        revolve!(
            "mw_bowl",
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.06, 0.05),
                (0.055, 0.05),
                (0.036, 0.006),
                (0.0, 0.006)
            ],
            at = f.at(x - pw / 2.0 - 0.01, y + 0.045, zf - cav.2 / 2.0),
            material = M.pop_orange,
            segments = 12
        );
        fcyl!(
            f,
            "mw_noodles",
            0.052,
            0.01,
            (x - pw / 2.0 - 0.01, y + 0.085, zf - cav.2 / 2.0),
            M.cardboard,
            verts = 12
        );
        soft!(
            f,
            "mw_keypad",
            pw,
            h - 0.04,
            0.01,
            (x + w / 2.0 - pw / 2.0 - 0.01, y + h / 2.0, zf - 0.002),
            M.panel,
            round = 0.004,
            round_segments = 1
        );
        fbox!(
            f,
            "mw_display",
            pw - 0.03,
            0.04,
            0.004,
            (x + w / 2.0 - pw / 2.0 - 0.01, y + h - 0.06, zf + 0.004),
            M.display
        );
        fbox!(
            f,
            "mw_digits",
            pw - 0.07,
            0.016,
            0.003,
            (x + w / 2.0 - pw / 2.0 - 0.01, y + h - 0.06, zf + 0.006),
            M.lcd
        );
        for i in 0..3 {
            for j in 0..3 {
                soft!(
                    f,
                    format!("mw_key{i}{j}"),
                    0.022,
                    0.016,
                    0.006,
                    (
                        x + w / 2.0 - pw / 2.0 - 0.04 + i as f64 * 0.03,
                        y + h - 0.12 - j as f64 * 0.03,
                        zf + 0.004
                    ),
                    M.white,
                    round = 0.003,
                    round_segments = 1
                );
            }
        }
        revolve!(
            "mw_dial",
            &[
                (0.0, 0.0),
                (0.024, 0.0),
                (0.024, 0.012),
                (0.02, 0.02),
                (0.0, 0.022)
            ],
            at = f.at(x + w / 2.0 - pw / 2.0 - 0.01, y + 0.06, zf + 0.003),
            material = M.white,
            segments = 12,
            rot = (90, 0, 0)
        );
        for fx in [x - w / 2.0 + 0.04, x + w / 2.0 - 0.04] {
            fcyl!(
                f,
                format!("mw_foot{fx}"),
                0.012,
                0.008,
                (fx, y + 0.004, zf - 0.04),
                M.tyre,
                verts = 8
            );
        }
        // frozen dinners, a pizza box, the cool box below, cans, a cutlery jar and dropped wrappers
        let mut yy = 0.435;
        for i in 0..4 {
            let dx = rnd.uniform(-0.02, 0.02);
            let turn = rnd.uniform(-10.0, 10.0);
            soft!(
                f,
                format!("dinner{i}"),
                0.26,
                0.045,
                0.2,
                (-0.16 + dx, yy + 0.0225, -0.22),
                POP[i % 6],
                rot = (0.0, 0.0, turn),
                round = 0.006,
                round_segments = 1
            );
            fbox!(
                f,
                format!("dinner_label{i}"),
                0.12,
                0.046,
                0.1,
                (-0.16, yy + 0.0225, -0.22),
                M.white
            );
            yy += 0.045;
        }
        soft!(
            f,
            "pizza_box",
            0.3,
            0.04,
            0.3,
            (0.16, 0.455, -0.22),
            M.cardboard,
            round = 0.004,
            round_segments = 1
        );
        fcyl!(
            f,
            "pizza_logo",
            0.08,
            0.002,
            (0.16, 0.476, -0.22),
            M.pop_red,
            verts = 14
        );
        soft!(
            f,
            "cool_box",
            0.28,
            0.07,
            0.22,
            (0.0, 0.15, -0.22),
            M.pop_blue,
            round = 0.012,
            round_segments = 1
        );
        soft!(
            f,
            "cool_lid",
            0.29,
            0.02,
            0.23,
            (0.0, 0.195, -0.22),
            M.pop_orange,
            round = 0.008,
            round_segments = 1
        );
        fsweep!(
            f,
            "cool_handle",
            &[
                (-0.08, 0.205, -0.22),
                (-0.06, 0.225, -0.22),
                (0.06, 0.225, -0.22),
                (0.08, 0.205, -0.22)
            ],
            0.007,
            M.pop_orange,
            verts = 5,
            resolution = 2
        );
        _can(f, "can_a", 0.26, -0.05, 0.875, M.pop_red, false, 0.0);
        _can(f, "can_b", -0.29, -0.08, 0.875, M.pop_green, false, 0.0);
        revolve!(
            "jar",
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.035, 0.11),
                (0.037, 0.12),
                (0.031, 0.12),
                (0.0, 0.11)
            ],
            at = f.at(0.29, 0.875, -0.38),
            material = M.gloss_white,
            segments = 10
        );
        for (k, dx) in [-0.01, 0.01].into_iter().enumerate() {
            fsweep!(
                f,
                format!("fork{k}"),
                &[(0.29 + dx, 0.91, -0.38), (0.29 + 3.0 * dx, 1.03, -0.38)],
                0.0,
                M.chrome,
                profile = [
                    (-0.002, -0.005),
                    (0.002, -0.005),
                    (0.002, 0.005),
                    (-0.002, 0.005)
                ],
                up = (0, 0, 1),
                scale = [1.0, 1.6]
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the wrappers dropped on the floor in front: drawn, not the cart's size
    let dropped = new_parts(|| {
        _wrapper(f, "wrapper_a", 0.3, 0.3, M.bag_yellow, 1, 0.0);
        _wrapper(f, "wrapper_b", -0.25, 0.28, M.bag_red, 2, 0.0);
    });
    aside(&dropped, "dropped");
    // The door with its window and handle, hinged on its left edge: `door_open` swings it open to the left.
    let dw = w - pw - 0.02;
    let dx0 = x - w / 2.0 + 0.005;
    let door = new_parts(|| {
        kit::shaker!(
            f,
            "mw_door",
            dw,
            h - 0.05,
            (dx0 + dw / 2.0, y + h / 2.0, zf + 0.006),
            0.016,
            M.gloss_white,
            rail = 0.04,
            panel = M.window,
            recess = 0.004
        );
        fsweep!(
            f,
            "mw_handle",
            &[
                (dx0 + dw - 0.02, y + 0.06, zf + 0.012),
                (dx0 + dw - 0.02, y + 0.07, zf + 0.03),
                (dx0 + dw - 0.02, y + h - 0.07, zf + 0.03),
                (dx0 + dw - 0.02, y + h - 0.06, zf + 0.012)
            ],
            0.0,
            M.chrome,
            profile = rounded_rect(0.02, 0.012, 0.005, 1),
            up = (1, 0, 0),
            resolution = 2
        );
    });
    let door = hinged(
        join(&door, "door", None),
        f.at(dx0, y + h / 2.0, zf + 0.004),
    );
    let objs = settle(&meshes()); // on its Model centre: the back on the wall face
    swing!(
        door,
        "door_open",
        -100.0,
        seconds = 0.6,
        ease = Ease::BackOut
    );
    objs
}
