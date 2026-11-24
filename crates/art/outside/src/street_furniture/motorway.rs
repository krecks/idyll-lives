//! The motorway's direction sign: a blue panel on two galvanised posts, beside the carriageway before an exit and at
//! the ramps (render/shared/motorway.ts `signSpots`).

use super::common::own;
use artkit::far::low;
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{cosd, sind};
use artkit::parts::join_all;
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, settle};

/// The panel (m): its width and height, how high its foot stands, its depth; the posts' radius and how far apart.
const PANEL_W: f64 = 3.0;
const PANEL_H: f64 = 1.6;
const PANEL_FOOT: f64 = 2.3;
const PANEL_D: f64 = 0.04;
const POST_R: f64 = 0.065;
const POST_X: f64 = 1.1;

pub fn motorway_sign() -> Vec<Obj> {
    let m = own("motorwaySign");
    let f = GameFrame::new(0.0);
    let steel = m.body;
    let mid = PANEL_FOOT + PANEL_H / 2.0;
    let top = PANEL_FOOT + PANEL_H;
    // (Its face at +z, towards the traffic coming; the posts behind it.)
    let face = 0.0;
    let back = face - PANEL_D;
    let post_z = back - 0.02 - POST_R;
    {
        // the posts, each on a concrete foot, up to just under the panel's top, capped
        for x in [-POST_X, POST_X] {
            fcyl!(
                f,
                format!("post{}", float_repr(x)),
                POST_R,
                top - 0.05,
                (x, (top - 0.05) / 2.0, post_z),
                steel,
                verts = low(16, 6)
            );
            fcyl!(
                f,
                format!("foot{}", float_repr(x)),
                0.16,
                0.12,
                (x, 0.06, post_z),
                m.granite,
                verts = low(16, 6),
                bevel = 0.012
            );
            fcyl!(
                f,
                format!("cap{}", float_repr(x)),
                POST_R + 0.008,
                0.03,
                (x, top - 0.035, post_z),
                steel,
                verts = low(16, 6),
                bevel = 0.006
            );
        }
        // two rails across between the posts, the panel clamped to them
        for (k, y) in [PANEL_FOOT + 0.3, top - 0.3].into_iter().enumerate() {
            fbox!(
                f,
                format!("rail{k}"),
                PANEL_W - 0.2,
                0.08,
                0.05,
                (0, y, back - 0.025),
                steel,
                bevel = 0.006
            );
        }
        // the panel: its galvanised back, the blue face in a white border
        fbox!(
            f,
            "panelBack",
            PANEL_W,
            PANEL_H,
            PANEL_D * 0.5,
            (0, mid, back + PANEL_D * 0.25),
            steel,
            bevel = 0.006
        );
        fbox!(
            f,
            "panel",
            PANEL_W - 0.02,
            PANEL_H - 0.02,
            PANEL_D * 0.5,
            (0, mid, face - PANEL_D * 0.25),
            m.sign_blue,
            bevel = 0.004
        );
        fextrude!(
            f,
            "border",
            &rounded_rect(PANEL_W - 0.08, PANEL_H - 0.08, 0.08, low(3, 1)),
            0.006,
            (0, mid, face + 0.003),
            m.sign_white,
            holes = [rounded_rect(PANEL_W - 0.2, PANEL_H - 0.2, 0.05, low(3, 1))],
            bevel = 0.0
        );
        // its message (as one reads it from the front, where the game's +x is on the left): two lines of a place's name
        // (bars a letter high) and the motorway's number in its shield on the left, an arrow up and off to the right,
        // the exit's way
        for (k, (w, y)) in [(1.5, mid + 0.28), (1.15, mid - 0.02)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                f,
                format!("line{k}"),
                w,
                0.16,
                0.006,
                (1.15 - w / 2.0, y, face + 0.003),
                m.sign_white,
                bevel = 0.0
            );
        }
        fextrude!(
            f,
            "shield",
            &rounded_rect(0.42, 0.3, 0.05, low(3, 1)),
            0.006,
            (0.82, mid - 0.4, face + 0.003),
            m.sign_white,
            bevel = 0.0
        );
        fbox!(
            f,
            "number",
            0.26,
            0.14,
            0.006,
            (0.82, mid - 0.4, face + 0.006),
            m.sign_blue,
            bevel = 0.0
        );
        // (An arrow up, leant 35° towards the reader's right, the game's -x: its shaft and its head, each convex.)
        let (c, sn) = (cosd(35.0), sind(35.0));
        let turn = |pts: &[(f64, f64)]| -> Vec<(f64, f64)> {
            pts.iter()
                .map(|&(x, y)| (x * c - y * sn, x * sn + y * c))
                .collect()
        };
        for (name, outline) in [
            (
                "arrowShaft",
                turn(&[(-0.07, -0.42), (0.07, -0.42), (0.07, 0.04), (-0.07, 0.04)]),
            ),
            (
                "arrowHead",
                turn(&[(-0.26, 0.02), (0.26, 0.02), (0.0, 0.32)]),
            ),
        ] {
            fextrude!(
                f,
                name,
                &outline,
                0.006,
                (-1.0, mid, face + 0.003),
                m.sign_white,
                bevel = 0.0
            );
        }
    }
    join_all("motorwaySign");
    settle!(centre = false);
    meshes()
}
