//! The doorstep delivery locker and the effortless vibration plate.

use super::common::{M, WALL};
use artkit::bevel::ease;
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, cosd, sind};
use artkit::motion::Ease;
use artkit::parts::{aside, hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, rock, settle, soft, swing};

// --- the doorstep delivery locker (1x1): a bank of parcel lockers with a touch screen, an order waiting -----
const COLS: [f64; 3] = [-0.28, 0.0, 0.28];
const ROWS: [(f64, f64); 3] = [(0.34, 0.5), (0.88, 0.5), (1.42, 0.5)]; // (middle, height)
const OPEN: (usize, usize) = (0, 1); // the door that opens: left column, middle row
const FRONT: f64 = 0.0;

/// A locker door, its face pressed with a vent of slots, a recessed pull at its right, a status light and a number
/// plate. Returns nothing; all in the current parts.
#[allow(clippy::too_many_arguments)]
fn _locker_door(f: &GameFrame, name: &str, x: f64, y: f64, h: f64, number: usize, led: Mat) {
    let slots: Vec<Vec<(f64, f64)>> = (0..2)
        .map(|k| {
            rounded_rect(0.14, 0.012, 0.006, 2)
                .iter()
                .map(|&(u, v)| (u, v + 0.06 * k as f64 - 0.03))
                .collect()
        })
        .collect();
    let vent: Vec<Vec<(f64, f64)>> = slots
        .iter()
        .map(|s| s.iter().map(|&(u, v)| (u, v + h / 2.0 - 0.1)).collect())
        .collect();
    fextrude!(
        f,
        name,
        &rounded_rect(0.26, h - 0.03, 0.01, 2),
        0.02,
        (x, y, FRONT + 0.01),
        M.door,
        holes = vent,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        f,
        format!("{name}_vent_back"),
        0.15,
        0.08,
        0.004,
        (x, y + h / 2.0 - 0.1, FRONT + 0.002),
        M.cubby
    );
    fbox!(
        f,
        format!("{name}_pull"),
        0.022,
        0.1,
        0.012,
        (x + 0.1, y, FRONT + 0.024),
        M.steel,
        bevel = 0.005,
        bevel_segments = 1
    );
    fbox!(
        f,
        format!("{name}_pull_well"),
        0.014,
        0.08,
        0.004,
        (x + 0.1, y, FRONT + 0.0305),
        M.cubby
    );
    fsphere!(
        f,
        format!("{name}_led"),
        0.008,
        (x - 0.09, y + h / 2.0 - 0.07, FRONT + 0.021),
        led,
        scale = (1, 0.5, 1),
        segments = 8,
        rings = 4
    );
    fbox!(
        f,
        format!("{name}_plate"),
        0.05,
        0.026,
        0.003,
        (x - 0.07, y - h / 2.0 + 0.06, FRONT + 0.0215),
        M.white,
        bevel = 0.001,
        bevel_segments = 1
    );
    fbox!(
        f,
        format!("{name}_number"),
        0.012 + 0.004 * (number % 3) as f64,
        0.014,
        0.0015,
        (x - 0.07, y - h / 2.0 + 0.06, FRONT + 0.0235),
        M.bezel
    );
}

pub fn delivery_locker() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            f,
            "cabinet",
            0.86,
            1.86,
            -WALL,
            (0, 0.93, WALL / 2.0),
            M.locker,
            bevel = 0.02,
            bevel_segments = 2
        );
        fbox!(
            f,
            "plinth",
            0.82,
            0.06,
            0.02,
            (0, 0.03, FRONT + 0.008),
            M.bezel
        );
        for (c, x) in COLS.into_iter().enumerate() {
            for (r, (y, h)) in ROWS.into_iter().enumerate() {
                if (c, r) == (1, 1) {
                    continue;
                }
                if (c, r) == OPEN {
                    // the open locker: a lined cubby, a shelf, the takeout bag waiting in it
                    fbox!(
                        f,
                        "cubby",
                        0.24,
                        h - 0.05,
                        0.004,
                        (x, y, FRONT + 0.002),
                        M.cubby
                    );
                    fbox!(
                        f,
                        "cubby_bag",
                        0.12,
                        0.16,
                        0.004,
                        (x - 0.02, y - 0.1, FRONT + 0.005),
                        M.bag
                    );
                    fbox!(
                        f,
                        "cubby_bag_logo",
                        0.06,
                        0.04,
                        0.002,
                        (x - 0.02, y - 0.09, FRONT + 0.008),
                        M.bag_logo
                    );
                    continue;
                }
                _locker_door(
                    f,
                    &numbered("door"),
                    x,
                    y,
                    h,
                    3 * c + r,
                    if (r + c) % 3 != 0 {
                        M.led_green
                    } else {
                        M.accent
                    },
                );
            }
        }
        // the console: a bezel with the touch screen, its banner and button, a keypad and a camera
        fbox!(
            f,
            "bezel",
            0.26,
            0.47,
            0.03,
            (0, 0.88, FRONT + 0.008),
            M.bezel,
            bevel = 0.012,
            bevel_segments = 2
        );
        fbox!(
            f,
            "touch_screen",
            0.22,
            0.3,
            0.004,
            (0, 0.94, FRONT + 0.025),
            M.screen,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "screen_banner",
            0.18,
            0.05,
            0.003,
            (0, 1.0, FRONT + 0.0275),
            M.white
        );
        fbox!(
            f,
            "screen_button",
            0.08,
            0.08,
            0.003,
            (0, 0.87, FRONT + 0.0275),
            M.accent,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "keypad",
            0.13,
            0.08,
            0.006,
            (0, 0.72, FRONT + 0.024),
            M.steel,
            bevel = 0.003,
            bevel_segments = 1
        );
        for i in 0..3 {
            for j in 0..2 {
                fbox!(
                    f,
                    numbered("key"),
                    0.03,
                    0.022,
                    0.004,
                    (
                        -0.04 + 0.04 * i as f64,
                        0.705 + 0.032 * j as f64,
                        FRONT + 0.028
                    ),
                    M.bezel
                );
            }
        }
        fsphere!(
            f,
            "camera",
            0.01,
            (0, 1.12, FRONT + 0.024),
            M.screen_light,
            scale = (1, 0.5, 1),
            segments = 8,
            rings = 4
        );
        // the canopy over the top: an orange band with the logo, its underside lit
        fbox!(
            f,
            "roof",
            0.86,
            0.1,
            -WALL + 0.02,
            (0, 1.91, WALL / 2.0 + 0.01),
            M.accent,
            bevel = 0.02,
            bevel_segments = 2
        );
        fbox!(
            f,
            "logo",
            0.4,
            0.06,
            0.004,
            (0, 1.91, FRONT + 0.032),
            M.white,
            bevel = 0.002,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "logo_box",
            &[
                (-0.02, -0.018),
                (0.02, -0.018),
                (0.02, 0.012),
                (0.0, 0.022),
                (-0.02, 0.012)
            ],
            0.003,
            (-0.17, 1.91, FRONT + 0.035),
            M.accent
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the order waiting on the doorstep, drawn, not the locker's size
    let order = new_parts(|| {
        // the order waiting on the doorstep: a taped parcel with its label, a takeout bag with its top folded and handles
        fbox!(
            f,
            "parcel",
            0.32,
            0.22,
            0.24,
            (0.2, 0.11, 0.24),
            M.kraft,
            rot = (0, 0, -8),
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "parcel_tape",
            0.33,
            0.02,
            0.05,
            (0.2, 0.221, 0.24),
            M.tape,
            rot = (0, 0, -8)
        );
        fbox!(
            f,
            "parcel_tape_side",
            0.004,
            0.2,
            0.05,
            (0.2 + 0.161 * cosd(-8.0), 0.12, 0.24 + 0.161 * sind(-8.0)),
            M.tape,
            rot = (0, 0, -8)
        );
        fbox!(
            f,
            "parcel_label",
            0.1,
            0.002,
            0.07,
            (0.25, 0.221, 0.31),
            M.white,
            rot = (0, 0, -8)
        );
        soft!(
            f,
            "takeout_bag",
            0.22,
            0.24,
            0.14,
            (-0.22, 0.12, 0.22),
            M.bag,
            rot = (0, 0, 6),
            round = 0.012,
            puff = 0.006,
            flat = 2
        );
        fbox!(
            f,
            "takeout_fold",
            0.222,
            0.03,
            0.12,
            (-0.22, 0.255, 0.22),
            M.bag,
            rot = (0, 0, 6),
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "takeout_logo",
            0.12,
            0.08,
            0.004,
            (-0.215, 0.14, 0.292),
            M.bag_logo,
            rot = (0, 0, 6)
        );
        for dx in [-0.05, 0.05] {
            kit::tube!(
                f,
                numbered("bag_handle"),
                &[
                    (-0.22 + dx, 0.268, 0.2),
                    (-0.22 + dx * 0.9, 0.32, 0.215),
                    (-0.22 + dx * 0.8, 0.268, 0.24)
                ],
                0.005,
                M.tape,
                verts = 4,
                round = 2
            );
        }
    });
    aside(&order, "order");
    // One locker's door, hinged on its left edge: `door_open` swings it open for a takeout.
    let (x, (y, h)) = (COLS[OPEN.0], ROWS[OPEN.1]);
    let door = new_parts(|| _locker_door(f, "locker_door", x, y, h, 1, M.accent));
    let door = hinged(
        join(&door, "locker_door", None),
        f.at(x - 0.13, y, FRONT + 0.01),
    );
    let objs = settle!(centre = false);
    swing!(
        door,
        "door_open",
        105.0,
        seconds = 0.7,
        ease = Ease::BackOut
    );
    objs
}

// --- the effortless vibration plate (1x1): a deck at 0.125 the resident stands on, a column with a console --
const DECK: (f64, f64, f64) = (0.0, 0.105, 0.12); // the deck's middle, at its underside
const BAR_Y: f64 = 1.1; // the handlebar's height: a grown-up's waist on the deck

pub fn vibe_plate() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (dx, dy, dz) = DECK;
    let still = new_parts(|| {
        soft!(
            f,
            "base",
            0.62,
            dy - 0.012,
            0.6,
            (0, dy / 2.0 + 0.006, dz - 0.02),
            M.plate,
            round = 0.04,
            round_segments = 2,
            flat = 1
        );
        let band: Vec<(f64, f64, f64)> = rounded_rect(0.622, 0.6, 0.04, 3)
            .iter()
            .map(|&(u, v)| (u, 0.026, v + dz - 0.02))
            .collect();
        fsweep!(
            f,
            "band",
            &band,
            0.0,
            M.lime,
            profile = rounded_rect(0.02, 0.008, 0.003, 1),
            up = (0, 0, 1),
            closed = true
        );
        for sx in [-1, 1] {
            for sz in [-1, 1] {
                fcyl!(
                    f,
                    numbered("foot"),
                    0.03,
                    0.012,
                    (sx as f64 * 0.25, 0.006, dz - 0.02 + sz as f64 * 0.24),
                    M.pad,
                    verts = 10
                );
            }
        }
        // the column at the plate's far end, which the stander faces (they step on from the tile in front and face
        // in), curving up to the console; the handlebar bent from one tube, its arms reaching back to either side of
        // them at waist height, the grips by their hands (2026-10-21: a short bar at the hip, 30 cm ahead: I-packs-04)
        ease(
            fsweep!(
                f,
                "column",
                &[
                    (0.0, 0.09, -0.235),
                    (0.0, 0.5, -0.27),
                    (0.0, BAR_Y - 0.02, -0.32)
                ],
                0.0,
                M.plate,
                profile = rounded_rect(0.07, 0.09, 0.03, 3),
                up = (0, 1, 0),
                resolution = 2,
                scale = [1.15, 1.0, 0.9]
            ),
            60.0,
            1,
        );
        fcyl!(
            f,
            "column_collar",
            0.06,
            0.02,
            (0, 0.1, -0.236),
            M.lime,
            verts = 16,
            bevel = 0.004,
            bevel_segments = 1
        );
        let bar = kit::fillet!(
            &[
                (-0.26, BAR_Y - 0.02, -0.07),
                (-0.26, BAR_Y, -0.32),
                (0.26, BAR_Y, -0.32),
                (0.26, BAR_Y - 0.02, -0.07)
            ],
            0.07,
            steps = 4
        );
        kit::tube!(f, "handlebar", &bar, 0.02, M.plate, verts = 10, round = 1);
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                f,
                numbered("grip"),
                &[
                    (sf * 0.26, BAR_Y - 0.0185, -0.2),
                    (sf * 0.26, BAR_Y - 0.02, -0.06)
                ],
                0.029,
                M.lime,
                verts = 12,
                scale = [1.0, 1.0]
            );
            fsphere!(
                f,
                numbered("grip_end"),
                0.029,
                (sf * 0.26, BAR_Y - 0.02, -0.06),
                M.lime,
                scale = (1, 1, 1),
                segments = 12,
                rings = 6
            );
        }
        // the console, tilted towards the stander: its screen and three buttons
        fbox!(
            f,
            "console",
            0.22,
            0.14,
            0.04,
            (0, BAR_Y + 0.09, -0.32),
            M.bezel,
            rot = (30, 0, 0),
            bevel = 0.015,
            bevel_segments = 2
        );
        fbox!(
            f,
            "console_screen",
            0.17,
            0.08,
            0.004,
            (0, BAR_Y + 0.1, -0.298),
            M.readout,
            rot = (30, 0, 0),
            bevel = 0.003,
            bevel_segments = 1
        );
        for k in 0..3 {
            fcyl!(
                f,
                numbered("console_button"),
                0.009,
                0.006,
                (-0.05 + 0.05 * k as f64, BAR_Y + 0.045, -0.29),
                if k == 1 { M.lime } else { M.white },
                rot = (120, 0, 0),
                verts = 8
            );
        }
        kit::lathe!(
            f,
            "bottle",
            0.32,
            0.0,
            -0.32,
            &[
                (0.032, 0.0),
                (0.035, 0.006),
                (0.035, 0.08),
                (0.03, 0.1),
                (0.035, 0.12),
                (0.035, 0.165),
                (0.022, 0.195),
                (0.018, 0.2),
                (0.0, 0.2),
            ],
            M.bottle,
            verts = 14
        );
        kit::turned!(
            f,
            "bottle_cap",
            0.32,
            0.2,
            -0.32,
            &[
                (0.0, 0.0),
                (0.019, 0.0),
                (0.019, 0.016),
                (0.012, 0.022),
                (0.0, 0.022),
            ],
            M.white,
            segments = 12
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The deck the resident stands on: `shake` rocks it to and fro, quickly, while they shake. A ribbed rubber pad
    // with two lime footprints.
    let deck = new_parts(|| {
        fbox!(
            f,
            "deck",
            0.54,
            0.012,
            0.48,
            (dx, dy + 0.006, dz),
            M.pad,
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(
            f,
            "deck_pad",
            0.5,
            0.008,
            0.44,
            (dx, dy + 0.016, dz),
            M.pad_inner,
            bevel = 0.006,
            bevel_segments = 1
        );
        for k in 0..9 {
            fbox!(
                f,
                numbered("rib"),
                0.47,
                0.003,
                0.012,
                (dx, dy + 0.0205, dz - 0.2 + 0.05 * k as f64),
                M.pad,
                bevel = 0.0012,
                bevel_segments = 1
            );
        }
        let foot = smooth_outline(
            &[
                (0.0, -0.11),
                (0.035, -0.09),
                (0.045, 0.0),
                (0.05, 0.08),
                (0.03, 0.11),
                (-0.01, 0.11),
                (-0.04, 0.07),
                (-0.035, 0.0),
                (-0.03, -0.09),
            ],
            2,
            true,
        );
        for s in [-1, 1] {
            let sf = s as f64;
            let mark: Vec<(f64, f64)> = foot.iter().map(|&(u, v)| (sf * u, v)).collect();
            fextrude!(
                f,
                numbered("foot_mark"),
                &mark,
                0.004,
                (dx + sf * 0.13, dy + 0.0215, dz),
                M.lime,
                plane = Facing::Top
            );
        }
    });
    let deck = hinged(join(&deck, "deck", None), f.at(dx, dy, dz));
    let objs = settle!(centre = false);
    rock!(deck, "shake", 3.0, seconds = 0.25, axis = Axis::Y);
    objs
}
