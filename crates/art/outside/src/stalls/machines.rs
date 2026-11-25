//! The vending machines' cabinet, against the back of the tile, with a tray below that a snack or a can drops
//! into; and the snack machine, its window full of snacks in their spirals.

use super::common::M;
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, TAU, cos, sin};
use artkit::motion::{Slide, Swing, slide, swing};
use artkit::parts::{join, new_parts};
use artkit::place::GameFrame;
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsweep, settle, soft};

pub const FRONT: f64 = 0.29; // the cabinet's front face
pub const WIN_X: (f64, f64) = (-0.41, 0.16); // the window's sides
const BAY_X: (f64, f64) = (-0.36, 0.06); // the pick-up tray's opening
pub const BAY_Y: (f64, f64) = (0.185, 0.34); // its floor and top
pub const WIN_Y: (f64, f64) = (0.44, 1.72); // the window's bottom and top

/// The machine's still shell: back block, sides, header with its lit sign, the keypad column, the kick panel round
/// the tray's opening with its push flap, and the dark lining inside.
pub fn cabinet(f: &GameFrame, body: Mat) {
    let w = WIN_X.1 - WIN_X.0;
    let cx = (WIN_X.0 + WIN_X.1) / 2.0;
    fbox!(f, "plinth", 0.86, 0.04, 0.68, (0, 0.02, -0.08), M.tray);
    fbox!(
        f,
        "back",
        0.9,
        1.84,
        0.46,
        (0, 0.96, -0.2),
        body,
        bevel = 0.02,
        bevel_segments = 2
    );
    for s in [-1, 1] {
        fbox!(
            f,
            format!("side{s}"),
            0.04,
            1.84,
            0.27,
            (s as f64 * 0.43, 0.96, 0.155),
            body,
            bevel = 0.012,
            bevel_segments = 2
        );
    }
    fbox!(
        f,
        "header",
        0.82,
        0.16,
        0.26,
        (0, 1.80, 0.16),
        body,
        bevel = 0.01,
        bevel_segments = 1
    );
    fextrude!(
        f,
        "header_sign",
        &rounded_rect(0.72, 0.1, 0.02, 2),
        0.01,
        (0, 1.80, FRONT + 0.005),
        M.lit_sign
    );
    for k in 0..6 {
        // its six letters
        let size = if k != 0 { 0.05 } else { 0.06 };
        fbox!(
            f,
            format!("letter{k}"),
            size,
            size,
            0.004,
            (-0.19 + 0.076 * k as f64, 1.80, FRONT + 0.011),
            body
        );
    }
    // the keypad column on the right: a dark panel with a display, twelve keys, a card reader, a coin slot and a
    // coin return
    fbox!(
        f,
        "column",
        0.25,
        1.68,
        0.26,
        (0.285, 0.88, 0.16),
        body,
        bevel = 0.008,
        bevel_segments = 1
    );
    artkit::bevel::eased(fextrude!(
        f,
        "keypad",
        &rounded_rect(0.18, 0.5, 0.015, 2),
        0.02,
        (0.285, 1.2, FRONT + 0.01),
        M.inside
    ));
    fbox!(
        f,
        "screen",
        0.12,
        0.07,
        0.01,
        (0.285, 1.37, FRONT + 0.022),
        M.display
    );
    for r in 0..4 {
        for c in 0..3 {
            fbox!(
                f,
                format!("key{r}{c}"),
                0.035,
                0.03,
                0.01,
                (
                    0.245 + 0.04 * c as f64,
                    1.27 - 0.045 * r as f64,
                    FRONT + 0.022
                ),
                M.button,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
    }
    fbox!(
        f,
        "reader",
        0.1,
        0.1,
        0.012,
        (0.285, 1.03, FRONT + 0.022),
        M.button,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        f,
        "reader_slot",
        0.07,
        0.006,
        0.004,
        (0.285, 1.05, FRONT + 0.029),
        M.tray
    );
    fbox!(
        f,
        "coin_slot",
        0.012,
        0.04,
        0.004,
        (0.285, 1.105, FRONT + 0.022),
        M.tray
    );
    fextrude!(
        f,
        "coin_return",
        &rounded_rect(0.08, 0.06, 0.01, 1),
        0.01,
        (0.285, 0.6, FRONT + 0.004),
        M.inside
    );
    fbox!(
        f,
        "coin_cup",
        0.05,
        0.02,
        0.006,
        (0.285, 0.59, FRONT + 0.01),
        M.tray
    );
    // the kick panel round the tray's opening, a push flap over the opening
    fbox!(
        f,
        "kick",
        w,
        BAY_Y.0 - 0.04,
        0.26,
        (cx, (0.04 + BAY_Y.0) / 2.0, 0.16),
        body
    );
    fbox!(
        f,
        "lip",
        w,
        WIN_Y.0 - BAY_Y.1,
        0.025,
        (cx, (BAY_Y.1 + WIN_Y.0) / 2.0, FRONT - 0.0125),
        body
    );
    for (x0, x1) in [(WIN_X.0, BAY_X.0), (BAY_X.1, WIN_X.1)] {
        fbox!(
            f,
            format!("cheek{}", float_repr(x0)),
            x1 - x0,
            BAY_Y.1 - BAY_Y.0,
            0.26,
            ((x0 + x1) / 2.0, py::sum([BAY_Y.0, BAY_Y.1]) / 2.0, 0.16),
            body
        );
    }
    fbox!(
        f,
        "flap_word",
        0.08,
        0.014,
        0.004,
        (cx, BAY_Y.1 + 0.05, FRONT + 0.001),
        M.label
    ); // "PUSH"
    // the dark inside: the back, the sides and the top behind the window, and the tray's well
    fbox!(
        f,
        "lining_back",
        w,
        WIN_Y.1 - BAY_Y.0,
        0.01,
        (cx, (BAY_Y.0 + WIN_Y.1) / 2.0, 0.035),
        M.inside
    );
    for x in [WIN_X.0 + 0.003, WIN_X.1 - 0.003] {
        fbox!(
            f,
            format!("lining{}", float_repr(x)),
            0.006,
            WIN_Y.1 - BAY_Y.0,
            0.25,
            (x, (BAY_Y.0 + WIN_Y.1) / 2.0, 0.16),
            M.inside
        );
    }
    fbox!(
        f,
        "lining_top",
        w,
        0.006,
        0.25,
        (cx, WIN_Y.1 - 0.003, 0.16),
        M.inside
    );
    fbox!(
        f,
        "tray_floor",
        w,
        0.006,
        0.25,
        (cx, BAY_Y.0 + 0.003, 0.16),
        M.tray
    );
    // the window's frame: a rounded bezel round the glass
    artkit::bevel::eased(fextrude!(
        f,
        "bezel",
        &rounded_rect(w + 0.03, WIN_Y.1 - WIN_Y.0 + 0.03, 0.02, 2),
        0.012,
        (cx, py::sum([WIN_Y.0, WIN_Y.1]) / 2.0, FRONT + 0.002),
        body,
        holes = [rounded_rect(w - 0.01, WIN_Y.1 - WIN_Y.0 - 0.01, 0.012, 2)]
    ));
}

/// A vending machine's spiral, from depth z0 to z1 at (x, y): the wire a snack sits in.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn coil(f: &GameFrame, name: &str, x: f64, y: f64, z0: f64, z1: f64, r: f64, turns: f64) {
    let n = (turns * 5.0) as usize;
    let pts: Vec<(f64, f64, f64)> = (0..=n)
        .map(|i| {
            let (i, n) = (i as f64, n as f64);
            (
                x + r * sin(TAU * turns * i / n),
                y + r * cos(TAU * turns * i / n),
                z0 + (z1 - z0) * i / n,
            )
        })
        .collect();
    fsweep!(f, name, &pts, 0.0035, M.steel, verts = 3);
}

/// A crisp packet standing up: a pillow puffed out front and back, a crimped seal along its top.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn bag(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, w: f64, h: f64, d: f64, material: Mat) {
    soft!(
        f,
        name,
        w,
        h,
        d,
        (x, y, z),
        material,
        round = 0.01,
        puff = (0, 0, d * 0.35),
        round_segments = 1,
        flat = 2
    );
    fbox!(
        f,
        format!("{name}_seal"),
        w * 0.98,
        0.012,
        d * 0.3,
        (x, y + h / 2.0 - 0.004, z),
        material
    );
}

pub fn vending_snack() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let snacks = [
        M.snack_a, M.snack_b, M.snack_c, M.snack_d, M.snack_e, M.snack_f, M.snack_g,
    ];
    let cx = (WIN_X.0 + WIN_X.1) / 2.0;
    let mut drop = None;
    let still = new_parts(|| {
        cabinet(&f, M.snack_red);
        // five shelves of snacks, two deep, each in its spiral, a label strip with numbers along each shelf
        for row in 0..5 {
            let yb = 0.47 + row as f64 * 0.25;
            fbox!(
                f,
                format!("shelf{row}"),
                WIN_X.1 - WIN_X.0,
                0.015,
                0.19,
                (cx, yb, 0.125),
                M.steel
            );
            fbox!(
                f,
                format!("labels{row}"),
                WIN_X.1 - WIN_X.0,
                0.025,
                0.004,
                (cx, yb - 0.004, 0.222),
                M.label
            );
            for col in 0..5 {
                let x = -0.355 + col as f64 * 0.115;
                fbox!(
                    f,
                    format!("number{row}{col}"),
                    0.03,
                    0.012,
                    0.003,
                    (x, yb - 0.004, 0.2245),
                    M.inside
                );
                coil(
                    &f,
                    &format!("coil{row}{col}"),
                    x,
                    yb + 0.04,
                    0.05,
                    0.215,
                    0.03,
                    2.0,
                );
                let m = snacks[(row * 5 + col * 3) % snacks.len()];
                let h = 0.14 - 0.02 * ((row + col) % 2) as f64;
                fbox!(
                    f,
                    format!("back{row}{col}"),
                    0.09,
                    h,
                    0.03,
                    (x, yb + 0.0075 + h / 2.0, 0.12),
                    m
                );
                if (row, col) == (1, 1) {
                    drop = Some((x, yb + 0.0075 + h / 2.0, h, m));
                    continue;
                }
                if (row + col) % 2 != 0 {
                    bag(
                        &f,
                        &format!("snack{row}{col}"),
                        x,
                        yb + 0.0075 + h / 2.0,
                        0.195,
                        0.09,
                        h,
                        0.03,
                        m,
                    );
                } else {
                    fbox!(
                        f,
                        format!("snack{row}{col}"),
                        0.09,
                        h,
                        0.03,
                        (x, yb + 0.0075 + h / 2.0, 0.195),
                        m
                    );
                    fbox!(
                        f,
                        format!("snack_label{row}{col}"),
                        0.06,
                        h * 0.35,
                        0.002,
                        (x, yb + 0.0075 + h * 0.55, 0.211),
                        M.label
                    );
                }
            }
        }
        fbox!(
            f,
            "window",
            WIN_X.1 - WIN_X.0,
            WIN_Y.1 - WIN_Y.0,
            0.01,
            (cx, py::sum([WIN_Y.0, WIN_Y.1]) / 2.0, FRONT - 0.015),
            M.glass
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the snack that drops: the front one of the second shelf's second column (a crisp packet)
    let (x, y, h, m) = drop.expect("the second shelf's second snack is the one that drops");
    let parts = new_parts(|| bag(&f, "snack", x, y, 0.195, 0.09, h, 0.03, m));
    let snack = join(&parts, "snack", Some(f.at(x, y, 0.195)));
    settle!(centre = false);
    let fall = y - (BAY_Y.0 + 0.006 + h / 2.0);
    slide(
        snack,
        "drop",
        (0, -0.05, 0),
        Slide {
            seconds: 0.15,
            ..Slide::default()
        },
    ); // pushed off the shelf to the front
    slide(
        snack,
        "drop",
        (0.0, 0.0, -fall),
        Slide {
            seconds: 0.9,
            ..Slide::default()
        },
    ); // and down past the shelves into the tray
    swing(
        snack,
        "drop",
        -15.0,
        Swing {
            seconds: 0.9,
            axis: Axis::X,
            ..Swing::default()
        },
    ); // tipping back as it lands
    meshes()
}
