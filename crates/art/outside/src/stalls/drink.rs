//! The drinks machine: a cabinet against the back of its tile, a lit front with three cans, a can dropping to the
//! tray.

use super::common::{M, settle_in_place};
use super::machines::{BAY_Y, FRONT, WIN_X, WIN_Y, cabinet, sum};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, radians};
use artkit::motion::{Slide, slide};
use artkit::parts::{join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, revolve};

/// A drinks can about Blender point `at` (its middle), upright unless turned: a body necked in at the top and the
/// bottom, its lid's rim and a ring pull.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn can(
    name: &str,
    at: P3,
    r: f64,
    h: f64,
    material: Mat,
    top: Mat,
    verts: usize,
    rot: (f64, f64, f64),
) -> (Obj, Obj) {
    let prof = [
        (0.0, -h / 2.0),
        (r * 0.8, -h / 2.0),
        (r, -h / 2.0 + r * 0.25),
        (r, h / 2.0 - r * 0.45),
        (r * 0.82, h / 2.0 - r * 0.05),
        (r * 0.84, h / 2.0),
        (0.0, h / 2.0),
    ];
    let body = revolve!(name, &prof, material = material, segments = verts);
    let lid = revolve!(
        format!("{name}_lid"),
        &[
            (0.0, h / 2.0 - 0.004),
            (r * 0.78, h / 2.0 - 0.004),
            (r * 0.8, h / 2.0 + 0.002),
            (r * 0.7, h / 2.0 + 0.002),
            (r * 0.68, h / 2.0 - 0.001),
            (0.0, h / 2.0 - 0.001)
        ],
        material = top,
        segments = verts
    );
    for o in [body, lid] {
        o.set_location(at);
        o.set_rotation_euler((radians(rot.0), radians(rot.1), radians(rot.2)));
    }
    (body, lid)
}

pub fn vending_drink() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let cx = (WIN_X.0 + WIN_X.1) / 2.0;
    let still = new_parts(|| {
        cabinet(&f, M.drink_blue);
        // the lit front, its bands, three big cans standing proud of it with a lit button and a price under each
        fbox!(
            f,
            "panel",
            WIN_X.1 - WIN_X.0,
            WIN_Y.1 - WIN_Y.0,
            0.012,
            (cx, sum(WIN_Y) / 2.0, FRONT - 0.006),
            M.lit_panel
        );
        fbox!(
            f,
            "band_top",
            WIN_X.1 - WIN_X.0,
            0.12,
            0.012,
            (cx, 1.6, FRONT + 0.004),
            M.band
        );
        fbox!(
            f,
            "band_low",
            WIN_X.1 - WIN_X.0,
            0.07,
            0.012,
            (cx, 0.7, FRONT + 0.004),
            M.band
        );
        // bubbles rising on the panel behind the cans
        for (k, (x, y, r)) in [
            (-0.36, 1.45, 0.02),
            (-0.2, 1.48, 0.014),
            (-0.02, 1.44, 0.018),
            (0.1, 1.47, 0.012),
            (-0.29, 1.04, 0.012),
            (-0.08, 1.06, 0.016),
            (0.08, 1.02, 0.012),
        ]
        .into_iter()
        .enumerate()
        {
            fcyl!(
                f,
                format!("bubble{k}"),
                r,
                0.004,
                (x, y, FRONT + 0.001),
                M.band,
                verts = 10,
                rot = (90, 0, 0)
            );
        }
        for (i, m) in [M.snack_c, M.snack_a, M.can_green].into_iter().enumerate() {
            let x = -0.315 + i as f64 * 0.19;
            can(
                &format!("can{i}"),
                f.at(x, 1.2, FRONT + 0.01),
                0.055,
                0.25,
                m,
                M.steel,
                16,
                (0.0, 0.0, 0.0),
            );
            fcyl!(
                f,
                format!("can_label{i}"),
                0.0565,
                0.07,
                (x, 1.19, FRONT + 0.01),
                M.label,
                verts = 16
            );
            fextrude!(
                f,
                format!("button{i}"),
                &rounded_rect(0.1, 0.05, 0.012, 2),
                0.016,
                (x, 0.95, FRONT + 0.006),
                M.button
            );
            fbox!(
                f,
                format!("button_light{i}"),
                0.06,
                0.01,
                0.004,
                (x, 0.95, FRONT + 0.015),
                M.display
            );
            fbox!(
                f,
                format!("price{i}"),
                0.08,
                0.025,
                0.006,
                (x, 1.03, FRONT + 0.003),
                M.display
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the can that drops: lying across, hidden inside above the tray until it falls into it
    let (x, y, z) = (-0.15, 0.4, 0.2);
    let parts = new_parts(|| {
        can(
            "can",
            f.at(x, y, z),
            0.033,
            0.12,
            M.snack_c,
            M.steel,
            12,
            (0.0, 90.0, 0.0),
        );
    });
    let can = join(&parts, "can", Some(f.at(x, y, z)));
    settle_in_place();
    slide(
        can,
        "drop",
        (0.0, 0.0, -(y - (BAY_Y.0 + 0.006 + 0.033))),
        Slide {
            seconds: 0.7,
            ..Slide::default()
        },
    ); // down into the tray
    slide(
        can,
        "drop",
        (0, -0.04, 0),
        Slide {
            seconds: 0.9,
            ..Slide::default()
        },
    ); // rolling to its front
    meshes()
}
