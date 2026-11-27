//! The coffee machine: a counter with an espresso machine on it.

use super::counter;
use crate::common::stream;
use crate::kitchen_cooking::M;
use artkit::forms::eased;
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::kit::fillet;
use artkit::materials::Mat;
use artkit::math::Axis;
use artkit::motion::{Grow, grow};
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, loft, revolve, ring, slide};

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
            &eased(
                &[
                    (0.0, 0.0),
                    (0.056, 0.0),
                    (0.05, 0.016),
                    (0.012, 0.02),
                    (0.012, 0.024),
                    (0.0, 0.024)
                ],
                0.002
            ),
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
    let stream = stream(
        f,
        "stream",
        (-0.14, 1.122, -0.11),
        0.99,
        M.coffee,
        0.008,
        8,
        0.05,
    );
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
        &eased(
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.05, 0.008),
                (0.047, 0.01),
                (0.0, 0.004)
            ],
            0.002
        ),
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
        &eased(
            &[
                (0.0, 0.0),
                (r * 1.03, 0.0),
                (r * 1.03, h * 0.1),
                (r * 0.6, h * 0.14),
                (r * 0.25, h * 0.14),
                (r * 0.25, h * 0.16),
                (0.0, h * 0.16)
            ],
            0.002
        ),
        at = (bx, by, bz + h * 0.84),
        material = lid,
        segments = 14
    );
}
