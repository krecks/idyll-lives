//! The breakfast counter: a toaster, a blender and a fruit bowl on a counter.

use super::counter;
use crate::kitchen_cooking::M;
use crate::kitchen_cooking::common::knob;
use artkit::forms::eased;
use artkit::forms::rounded_rect;
use artkit::math::Axis;
use artkit::motion::{Spin, spin};
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fsphere, fsweep, loft, revolve, ring, slide, soft};

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
            &eased(
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
                0.002
            ),
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
