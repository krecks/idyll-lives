//! The microwave counter and the small appliances: the coffee machine, the breakfast counter and two dishwashers,
//! each on a base unit under a worktop.

use super::common::{hollow, knob};
use super::{M, SECONDS};
use crate::common::shaker;
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::kit::fillet;
use artkit::materials::Mat;
use artkit::math::Axis;
use artkit::motion::{Ease, Grow, Spin, grow, spin};
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, slide, soft, swing, torus,
};

// --- microwave counter: 1x1, a microwave on a small base unit ------------------------------------------

/// A microwave on a base unit (a drawer over two doors): a rounded charcoal body with vents down its side, a
/// control column with a display, buttons and a dial, a door with a window and a grip; inside, a bowl of soup on the
/// turntable over its roller ring.
pub fn microwave() -> Vec<Obj> {
    let f = &GameFrame::new(0.15);
    let still = new_parts(|| {
        counter(f, 1.0, None, None);
        // the microwave: a rounded body, hollowed through the front of its left part (the control column on the right
        // as the viewer sees it: game -x)
        let body = soft!(
            f,
            "micro_body",
            0.50,
            0.29,
            0.36,
            (-0.20, 1.105, -0.28),
            M.charcoal,
            round = 0.015,
            round_segments = 2
        );
        hollow(
            body,
            f,
            "micro_cavity",
            0.36,
            0.25,
            0.36,
            (-0.15, 1.10, -0.255),
            Some(M.plate),
        );
        for i in 0..8 {
            fbox!(
                f,
                format!("vent{i}"),
                0.003,
                0.008,
                0.16,
                (-0.451, 1.04 + 0.018 * i as f64, -0.33),
                M.dark
            );
        }
        for s in [-1, 1] {
            for k in [-1, 1] {
                fcyl!(
                    f,
                    format!("foot{s}{k}"),
                    0.012,
                    0.012,
                    (-0.2 + s as f64 * 0.2, 0.956, -0.28 + k as f64 * 0.14),
                    M.dark,
                    verts = 8
                );
            }
        }
        // the control column's face: a display, a grid of buttons, a dial and the door release
        fextrude!(
            f,
            "micro_face",
            &rounded_rect(0.11, 0.26, 0.01, 2),
            0.006,
            (-0.39, 1.105, -0.097),
            M.dark
        );
        fbox!(
            f,
            "micro_screen",
            0.08,
            0.04,
            0.004,
            (-0.39, 1.205, -0.093),
            M.hob,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "micro_digits",
            0.05,
            0.016,
            0.002,
            (-0.39, 1.205, -0.0905),
            M.display
        );
        for r in 0..3 {
            for c in 0..2 {
                fbox!(
                    f,
                    format!("button{r}{c}"),
                    0.03,
                    0.016,
                    0.004,
                    (-0.405 + 0.03 * c as f64, 1.16 - 0.025 * r as f64, -0.093),
                    M.chrome,
                    bevel = 0.0015,
                    bevel_segments = 1
                );
            }
        }
        knob(f, "dial", -0.39, 1.055, -0.087, 0.018, 0.014, None, true);
        fbox!(
            f,
            "release",
            0.07,
            0.022,
            0.006,
            (-0.39, 1.005, -0.092),
            M.chrome,
            bevel = 0.003,
            bevel_segments = 1
        );
        // the turntable's roller ring under the plate
        torus!(
            "rollers",
            0.08,
            0.005,
            at = f.at(-0.14, 0.976, -0.27),
            material = M.dark,
            major_segments = 14,
            minor_segments = 4
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the door, hinged on its left edge (seen from the front): a frame round its window, a grip by the controls
    let parts = new_parts(|| {
        fextrude!(
            f,
            "micro_door",
            &rounded_rect(0.38, 0.28, 0.012, 2),
            0.02,
            (-0.14, 1.1, -0.09),
            M.charcoal,
            holes = [rounded_rect(0.27, 0.19, 0.01, 2)],
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "micro_window",
            0.28,
            0.20,
            0.006,
            (-0.14, 1.10, -0.088),
            M.glass
        );
        fsweep!(
            f,
            "micro_grip",
            &[
                (-0.305, 1.03, -0.082),
                (-0.312, 1.04, -0.068),
                (-0.312, 1.16, -0.068),
                (-0.305, 1.17, -0.082)
            ],
            0.0,
            M.chrome,
            profile = rounded_rect(0.014, 0.01, 0.004, 1),
            up = (1, 0, 0),
            resolution = 2
        );
    });
    let door = hinged(join(&parts, "micro_door", None), f.at(0.05, 1.1, -0.09));
    // the turntable inside, with a bowl of soup on it, a mark so its turn shows
    let parts = new_parts(|| {
        let b = f.at(-0.14, 0.972, -0.27);
        revolve!(
            "turntable",
            &[
                (0.0, 0.0),
                (0.125, 0.0),
                (0.13, 0.008),
                (0.126, 0.012),
                (0.11, 0.006),
                (0.0, 0.006)
            ],
            at = (b.x, b.y, b.z),
            material = M.plate,
            segments = 20
        );
        fbox!(
            f,
            "turntable_mark",
            0.03,
            0.006,
            0.03,
            (-0.22, 0.98, -0.27),
            M.dark,
            bevel = 0.002
        );
        revolve!(
            "soup_bowl",
            &[
                (0.0, 0.0),
                (0.035, 0.0),
                (0.055, 0.04),
                (0.05, 0.04),
                (0.032, 0.008),
                (0.0, 0.008)
            ],
            at = f.at(-0.12, 0.978, -0.26),
            material = M.cup,
            segments = 14
        );
        fcyl!(
            f,
            "soup",
            0.048,
            0.004,
            (-0.12, 1.012, -0.26),
            M.fruit,
            verts = 14
        );
    });
    let turn = join(&parts, "turntable", Some(f.at(-0.14, 0.978, -0.27)));
    settle_all();
    swing!(
        door,
        "door_open",
        -100,
        seconds = 1.0,
        axis = Axis::Z,
        ease = Ease::BackOut
    );
    spin(
        turn,
        "turntable",
        Spin {
            turns: 1.0,
            seconds: 2.0,
            axis: Axis::Z,
        },
    );
    Vec::new()
}

// --- small appliances: a coffee machine, a breakfast counter and two dishwashers ---------------------

/// A base unit like the old part's `k_base`: a plinth, a carcass and a worktop to 0.95 with a rounded front edge,
/// a shaker drawer over two shaker doors on the front (the front face at depth 0.165), bar pulls.
fn counter(f: &GameFrame, w: f64, body: Option<Mat>, top: Option<Mat>) {
    let (body, top) = (body.unwrap_or(M.cab), top.unwrap_or(M.worktop));
    fbox!(
        f,
        "plinth",
        w - 0.04,
        0.10,
        0.54,
        (0, 0.05, -0.19),
        M.dark,
        bevel = 0.005
    );
    fbox!(
        f,
        "carcass",
        w - 0.004,
        0.82,
        0.60,
        (0, 0.51, -0.16),
        body,
        bevel = 0.006
    );
    fbox!(
        f,
        "worktop",
        w,
        0.04,
        0.66,
        (0, 0.93, -0.15),
        top,
        bevel = 0.01,
        bevel_segments = 2
    );
    shaker(
        f,
        "drawer",
        w - 0.02,
        0.14,
        (0, 0.84, 0.155),
        body,
        0.02,
        0.04,
        0.004,
    );
    pull!(
        f,
        "drawer_pull",
        (-0.09, 0.84),
        (0.09, 0.84),
        0.165,
        out = 0.02,
        w = 0.012,
        r = None,
        verts = 6
    );
    for s in [-1, 1] {
        let sf = s as f64;
        shaker(
            f,
            &format!("door{s}"),
            w / 2.0 - 0.015,
            0.66,
            (sf * w / 4.0, 0.44, 0.155),
            body,
            0.02,
            0.06,
            0.004,
        );
        pull!(
            f,
            format!("door_pull{s}"),
            (sf * 0.05, 0.62),
            (sf * 0.05, 0.74),
            0.165,
            out = 0.02,
            w = 0.012,
            r = None,
            verts = 6
        );
    }
}

/// A thin stream from (x, top_y, z) down to bottom_y, joined with its origin at the top and scaled
/// to nothing along its length, so it is hidden until `grow` runs the clip (the `water_on` way).
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn stream(
    f: &GameFrame,
    name: &str,
    x: f64,
    top_y: f64,
    z: f64,
    bottom_y: f64,
    material: Mat,
    r: f64,
) -> Obj {
    let length = artkit::py::max(0.05, top_y - bottom_y);
    let parts = new_parts(|| {
        fcyl!(
            f,
            name,
            r,
            length,
            (x, (top_y + bottom_y) / 2.0, z),
            material,
            verts = 8
        );
    });
    let obj = join(&parts, name, Some(f.at(x, top_y, z)));
    obj.edit(|o| o.scale.z = 0.001); // hidden at rest; `grow` runs it out along its length
    obj
}

// --- coffee machine: 1x1, a counter with an espresso machine on the worktop ----------------------------

/// A counter with an espresso machine on it: a rounded body with a steel cup-warmer and its rail on top, a group
/// head with a portafilter, a slotted drip tray, a pressure gauge, a steam wand and a bean hopper; a canister and a
/// sugar bowl beside it. The cup slides under the spout and the coffee runs.
pub fn coffee_machine() -> Vec<Obj> {
    let f = &GameFrame::new(0.15);
    let still = new_parts(|| {
        counter(f, 1.0, None, None);
        let b = f.at(-0.14, 0.95, -0.26);
        let (bx, by, bz) = (b.x, b.y, b.z);
        let secs: Vec<_> = [(0.0, 0.97), (0.012, 1.0), (0.29, 1.0), (0.3, 0.98)]
            .into_iter()
            .map(|(h, k)| {
                ring!(
                    0.165 * k,
                    0.155 * k,
                    z = bz + h,
                    verts = 24,
                    square = 5.0,
                    at = (bx, by)
                )
            })
            .collect();
        loft!("machine", &secs, material = M.charcoal);
        fbox!(
            f,
            "warming",
            0.32,
            0.012,
            0.3,
            (-0.14, 1.256, -0.26),
            M.steel,
            bevel = 0.004,
            bevel_segments = 1
        );
        kit::tube!(
            f,
            "warming_rail",
            &fillet(
                &[
                    (-0.29, 1.262, -0.14),
                    (-0.29, 1.28, -0.13),
                    (0.01, 1.28, -0.13),
                    (0.01, 1.262, -0.14),
                ],
                0.01,
                2,
            ),
            0.004,
            M.chrome,
            verts = 6,
            round = 1
        );
        // the group head and the portafilter locked in under it, its handle out to the side
        let h = f.at(-0.14, 1.15, -0.11);
        revolve!(
            "group",
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.034, 0.012),
                (0.04, 0.03),
                (0.042, 0.06),
                (0.0, 0.06)
            ],
            at = (h.x, h.y, h.z),
            material = M.steel,
            segments = 14
        );
        revolve!(
            "portafilter",
            &[(0.0, 0.0), (0.03, 0.0), (0.035, 0.022), (0.0, 0.022)],
            at = f.at(-0.14, 1.128, -0.11),
            material = M.steel,
            segments = 12
        );
        fbox!(
            f,
            "spout",
            0.03,
            0.012,
            0.012,
            (-0.14, 1.122, -0.11),
            M.steel,
            bevel = 0.003,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "portafilter_handle",
            &[
                (-0.10, 1.14, -0.1),
                (-0.04, 1.145, -0.06),
                (0.0, 1.15, -0.03)
            ],
            0.011,
            M.dark,
            verts = 8,
            resolution = 2,
            scale = [0.8, 1.0, 1.1]
        );
        // the drip tray with its slotted grille
        fbox!(
            f,
            "drip",
            0.3,
            0.02,
            0.24,
            (-0.14, 0.96, -0.11),
            M.steel,
            bevel = 0.004,
            bevel_segments = 1
        );
        let slots: Vec<Vec<(f64, f64)>> = (0..6)
            .map(|i| {
                rounded_rect(0.02, 0.17, 0.008, 1)
                    .into_iter()
                    .map(|(u, v)| (u - 0.1 + 0.04 * i as f64, v))
                    .collect()
            })
            .collect();
        fextrude!(
            f,
            "drip_grille",
            &rounded_rect(0.28, 0.22, 0.01, 1),
            0.004,
            (-0.14, 0.971, -0.11),
            M.chrome,
            plane = Facing::Top,
            holes = slots
        );
        // the gauge, two buttons, the steam wand down the side, the hopper on top
        revolve!(
            "gauge",
            &[
                (0.0, 0.0),
                (0.034, 0.0),
                (0.034, 0.008),
                (0.029, 0.01),
                (0.0, 0.006)
            ],
            at = f.at(-0.03, 1.2, -0.104),
            material = M.chrome,
            segments = 14,
            rot = (90, 0, 0)
        );
        fcyl!(
            f,
            "gauge_face",
            0.028,
            0.002,
            (-0.03, 1.2, -0.096),
            M.cup,
            verts = 14,
            rot = (90, 0, 0)
        );
        fbox!(
            f,
            "gauge_needle",
            0.003,
            0.02,
            0.002,
            (-0.025, 1.207, -0.094),
            M.heat,
            rot = (0, 30, 0)
        );
        for (i, x) in [-0.23, -0.19].into_iter().enumerate() {
            fcyl!(
                f,
                format!("button{i}"),
                0.011,
                0.01,
                (x, 1.21, -0.101),
                M.chrome,
                verts = 10,
                rot = (90, 0, 0)
            );
        }
        kit::tube!(
            f,
            "steam_wand",
            &[
                (0.02, 1.17, -0.15),
                (0.06, 1.165, -0.13),
                (0.065, 1.08, -0.12),
                (0.06, 1.02, -0.115),
            ],
            0.0055,
            M.chrome,
            verts = 6,
            round = 3
        );
        let h = f.at(-0.06, 1.262, -0.30);
        let (hx, hy, hz) = (h.x, h.y, h.z);
        let secs: Vec<_> = [(0.0, 0.7), (0.03, 0.9), (0.11, 1.2), (0.114, 1.12)]
            .into_iter()
            .map(|(h, k)| {
                ring!(
                    0.045 * k,
                    0.045 * k,
                    z = hz + h,
                    verts = 12,
                    square = 3.0,
                    at = (hx, hy)
                )
            })
            .collect();
        loft!("hopper", &secs, material = M.bean);
        revolve!(
            "hopper_lid",
            &[
                (0.0, 0.0),
                (0.056, 0.0),
                (0.05, 0.016),
                (0.012, 0.02),
                (0.012, 0.024),
                (0.0, 0.024)
            ],
            at = (hx, hy, hz + 0.114),
            material = M.charcoal,
            segments = 12
        );
        // a coffee canister and a sugar bowl on the worktop
        canister(
            f, "canister", 0.25, 0.95, -0.34, 0.06, 0.16, M.bean, M.steel,
        );
        canister(f, "sugar", 0.34, 0.95, -0.2, 0.045, 0.07, M.cup, M.cup);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the cup, on the drip tray's front edge, sliding back under the spout for `brew`
    let parts = new_parts(|| cup(f, "cup", -0.14, 0.965, 0.0));
    let cup = join(&parts, "cup", Some(f.at(-0.14, 0.965, 0.0)));
    let stream = stream(f, "stream", -0.14, 1.122, -0.11, 0.99, M.coffee, 0.008);
    settle_all();
    slide!(cup, "brew", (0, 0.11, 0), seconds = 0.6);
    grow(
        stream,
        "brew",
        Grow {
            seconds: 0.4,
            axis: Axis::Z,
        },
    );
    Vec::new()
}

/// An espresso cup on a saucer on game height `y`, its handle out to the side.
fn cup(f: &GameFrame, name: &str, x: f64, y: f64, z: f64) {
    let b = f.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    revolve!(
        format!("{name}_saucer"),
        &[
            (0.0, 0.0),
            (0.03, 0.0),
            (0.05, 0.008),
            (0.047, 0.01),
            (0.0, 0.004)
        ],
        at = (bx, by, bz),
        material = M.cup,
        segments = 14
    );
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (0.022, 0.0),
            (0.033, 0.02),
            (0.035, 0.05),
            (0.031, 0.05),
            (0.029, 0.03),
            (0.0, 0.03)
        ],
        at = (bx, by, bz + 0.006),
        material = M.cup,
        segments = 14
    );
    fcyl!(
        f,
        format!("{name}_crema"),
        0.03,
        0.002,
        (x, y + 0.048, z),
        M.coffee,
        verts = 12
    );
    fsweep!(
        f,
        format!("{name}_handle"),
        &[
            (x + 0.032, y + 0.045, z),
            (x + 0.05, y + 0.042, z),
            (x + 0.05, y + 0.024, z),
            (x + 0.034, y + 0.02, z)
        ],
        0.004,
        M.cup,
        verts = 5,
        resolution = 2
    );
}

/// A round canister with a lid on game height `y`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn canister(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    lid: Mat,
) {
    let b = f.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (r * 0.96, 0.0),
            (r, 0.006),
            (r, h * 0.86),
            (0.0, h * 0.86)
        ],
        at = (bx, by, bz),
        material = material,
        segments = 14
    );
    revolve!(
        format!("{name}_lid"),
        &[
            (0.0, 0.0),
            (r * 1.03, 0.0),
            (r * 1.03, h * 0.1),
            (r * 0.6, h * 0.14),
            (r * 0.25, h * 0.14),
            (r * 0.25, h * 0.16),
            (0.0, h * 0.16)
        ],
        at = (bx, by, bz + h * 0.84),
        material = lid,
        segments = 14
    );
}

// --- breakfast counter: 1x1, a counter with a toaster and a blender on the worktop ---------------------

/// A counter with a rounded two-slot toaster (its browning dial and crumb tray), a blender (a buttoned base and a
/// jug with a handle) and a fruit bowl: oranges, an apple and a banana. The toast goes down and pops up; the
/// blender's lid lifts while the smoothie swirls.
pub fn breakfast_counter() -> Vec<Obj> {
    let f = &GameFrame::new(0.15);
    let still = new_parts(|| {
        counter(f, 1.0, None, None);
        // the toaster: a rounded body, two slots in its top, a dial and a crumb tray
        soft!(
            f,
            "toaster",
            0.28,
            0.18,
            0.16,
            (0.30, 1.04, -0.30),
            M.toaster,
            round = 0.045,
            round_segments = 3
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("slot{s}"),
                0.12,
                0.012,
                0.032,
                (0.30, 1.126, -0.30 + s as f64 * 0.035),
                M.dark,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        knob(
            f,
            "dial",
            0.3,
            1.0,
            -0.215,
            0.014,
            0.012,
            Some(M.chrome),
            true,
        );
        fbox!(
            f,
            "crumb_tray",
            0.18,
            0.012,
            0.01,
            (0.30, 0.962, -0.218),
            M.chrome,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "lever_slot",
            0.006,
            0.1,
            0.012,
            (0.441, 1.06, -0.30),
            M.dark
        );
        // the blender: a tapered base with buttons, the jug with its handle
        let b = f.at(0.0, 0.95, -0.32);
        let (bx, by, bz) = (b.x, b.y, b.z);
        let secs: Vec<_> = [(0.0, 1.0), (0.09, 0.92), (0.12, 0.8)]
            .into_iter()
            .map(|(h, k)| {
                ring!(
                    0.08 * k,
                    0.08 * k,
                    z = bz + h,
                    verts = 16,
                    square = 4.0,
                    at = (bx, by)
                )
            })
            .collect();
        loft!("blender", &secs, material = M.charcoal);
        for (i, x) in [-0.03, 0.0, 0.03].into_iter().enumerate() {
            fbox!(
                f,
                format!("button{i}"),
                0.018,
                0.012,
                0.006,
                (x, 1.0, -0.247),
                if i != 0 { M.chrome } else { M.display },
                bevel = 0.002,
                bevel_segments = 1
            );
        }
        let secs: Vec<_> = [
            (0.12, 0.95),
            (0.13, 1.0),
            (0.38, 1.25),
            (0.38, 1.18),
            (0.135, 0.92),
        ]
        .into_iter()
        .map(|(h, k)| {
            ring!(
                0.06 * k,
                0.06 * k,
                z = bz + h,
                verts = 16,
                square = 3.0,
                at = (bx, by)
            )
        })
        .collect();
        loft!("jug", &secs, material = M.jug);
        fsweep!(
            f,
            "jug_handle",
            &[
                (-0.07, 1.30, -0.32),
                (-0.115, 1.29, -0.32),
                (-0.118, 1.14, -0.32),
                (-0.066, 1.11, -0.32)
            ],
            0.0,
            M.jug,
            profile = rounded_rect(0.022, 0.014, 0.006, 1),
            up = (0, 0, 1),
            resolution = 2
        );
        // the fruit bowl: two oranges, an apple and a banana
        let b = f.at(-0.28, 0.95, 0.05);
        revolve!(
            "bowl",
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.12, 0.045),
                (0.13, 0.06),
                (0.122, 0.06),
                (0.11, 0.048),
                (0.055, 0.008),
                (0.0, 0.008)
            ],
            at = (b.x, b.y, b.z),
            material = M.jug,
            segments = 18
        );
        for (i, (x, z)) in [(-0.31, 0.02), (-0.24, 0.09)].into_iter().enumerate() {
            fsphere!(
                f,
                format!("orange{i}"),
                0.04,
                (x, 1.0, z),
                M.fruit,
                segments = 10,
                rings = 7
            );
        }
        fsphere!(
            f,
            "apple",
            0.038,
            (-0.22, 1.0, 0.0),
            M.toaster,
            segments = 10,
            rings = 7,
            scale = (1, 1, 0.92)
        );
        fcyl!(
            f,
            "apple_stalk",
            0.003,
            0.016,
            (-0.22, 1.042, 0.0),
            M.bean,
            verts = 5,
            smooth = false
        );
        fsweep!(
            f,
            "banana",
            &[
                (-0.38, 1.02, 0.07),
                (-0.33, 1.0, 0.12),
                (-0.26, 1.005, 0.15),
                (-0.2, 1.03, 0.15)
            ],
            0.017,
            M.banana,
            verts = 6,
            resolution = 3,
            scale = [0.35, 1.0, 1.0, 0.45]
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the toaster's carriage, its lever and its toast standing up at rest: it goes down for toast, and
    // coming back when the use ends pops the toast up (one part, so the smoothie's swirl has a draw call)
    let parts = new_parts(|| {
        fbox!(
            f,
            "lever",
            0.024,
            0.024,
            0.036,
            (0.456, 1.08, -0.30),
            M.dark,
            bevel = 0.008,
            bevel_segments = 2
        );
        for s in [-1, 1] {
            soft!(
                f,
                format!("toast{s}"),
                0.10,
                0.12,
                0.022,
                (0.30, 1.11, -0.30 + s as f64 * 0.035),
                M.bread,
                round = 0.01,
                round_segments = 2,
                puff = 0.004,
                flat = 1
            );
            fbox!(
                f,
                format!("toast_face{s}"),
                0.086,
                0.044,
                0.023,
                (0.30, 1.142, -0.30 + s as f64 * 0.035),
                M.crumb
            );
        }
    });
    let toast = join(&parts, "toast", Some(f.at(0.30, 1.05, -0.30)));
    // the blender's lid, lifting off for `blend`
    let parts = new_parts(|| {
        revolve!(
            "lid",
            &[
                (0.0, 0.0),
                (0.077, 0.0),
                (0.078, 0.022),
                (0.07, 0.03),
                (0.0, 0.03)
            ],
            at = f.at(0.0, 1.33, -0.32),
            material = M.charcoal,
            segments = 16
        );
        fcyl!(
            f,
            "lid_knob",
            0.02,
            0.02,
            (0.0, 1.37, -0.32),
            M.charcoal,
            verts = 10,
            bevel = 0.004,
            bevel_segments = 1
        );
    });
    let lid = join(&parts, "lid", Some(f.at(0.0, 1.33, -0.32)));
    // the smoothie at the jug's brim, under the lid, with a streak so its swirl shows once the lid is up
    let parts = new_parts(|| {
        fcyl!(
            f,
            "smoothie",
            0.07,
            0.004,
            (0.0, 1.332, -0.32),
            M.fruit,
            verts = 16
        );
        fbox!(
            f,
            "smoothie_streak",
            0.05,
            0.002,
            0.012,
            (0.02, 1.335, -0.32),
            M.toaster,
            bevel = 0.001
        );
    });
    let swirl = join(&parts, "smoothie", Some(f.at(0.0, 1.332, -0.32)));
    settle_all();
    slide!(toast, "toast_down", (0, 0, -0.05), seconds = 0.5);
    slide!(lid, "blend", (0, 0, 0.10), seconds = 0.8);
    spin(
        swirl,
        "blades",
        Spin {
            turns: 1.0,
            seconds: 1.0,
            axis: Axis::Z,
        },
    );
    Vec::new()
}

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
    let f = &GameFrame::with_depths(0.085, &super::COUNTER66);
    dishwasher_unit(
        f, 1.0, 0.95, 0.79, -0.085, 0.25, 0.66, M.cab, M.steel, M.cab, 0.45, true,
    );
    Vec::new()
}
