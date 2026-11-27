//! The freezers: the chest freezer and the upright one, each a shell with its liner, opening its lid or its door
//! for `frozenTreat`.

use super::M;
use artkit::math::{Axis, P3};
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, slide, swing};

/// The chest freezer: a low box on the floor with its lid hinged at the back.
pub fn chest_freezer() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    // the box's middle (depth): its back on the wall's face, 0.68 m deep
    let z = -0.09;
    let still = new_parts(|| {
        // the shell: back, front, the two ends and the floor of the box, and its liner inside
        fbox!(
            f,
            "back",
            1.90,
            0.80,
            0.05,
            (0, 0.42, z - 0.315),
            M.body,
            bevel = 0.010
        );
        fbox!(
            f,
            "front",
            1.90,
            0.80,
            0.05,
            (0, 0.42, z + 0.315),
            M.body,
            bevel = 0.010
        );
        for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
            fbox!(
                f,
                format!("end{i}"),
                0.05,
                0.80,
                0.68,
                (sx * 0.925, 0.42, z),
                M.body,
                bevel = 0.010
            );
        }
        fbox!(
            f,
            "floor",
            1.90,
            0.06,
            0.68,
            (0, 0.03, z),
            M.body,
            bevel = 0.006
        );
        fbox!(f, "liner", 1.78, 0.70, 0.56, (0, 0.40, z), M.liner);
        // the rim: a frame on the walls' tops, so the opening shows the liner and the baskets
        for (k, dz) in [-0.315, 0.315].into_iter().enumerate() {
            fbox!(
                f,
                format!("rim_side{k}"),
                1.88,
                0.020,
                0.05,
                (0, 0.815, z + dz),
                M.trim
            );
        }
        for (k, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
            fbox!(
                f,
                format!("rim_end{k}"),
                0.05,
                0.020,
                0.58,
                (sx * 0.915, 0.815, z),
                M.trim
            );
        }
        fbox!(
            f,
            "badge",
            0.10,
            0.030,
            0.006,
            (0.78, 0.55, z + 0.342),
            M.chrome
        );
        // two wire baskets hung on the rim inside: a frame each, seen when the lid is up
        for (i, bx) in [-0.42, 0.42].into_iter().enumerate() {
            for (k, (w, d, dz)) in [(0.80, 0.014, 0.14), (0.80, 0.014, -0.14)]
                .into_iter()
                .enumerate()
            {
                fbox!(
                    f,
                    format!("basket{i}_side{k}"),
                    w,
                    0.05,
                    d,
                    (bx, 0.775, z + dz),
                    M.shelf,
                    bevel = 0.004
                );
            }
            for (k, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
                fbox!(
                    f,
                    format!("basket{i}_end{k}"),
                    0.014,
                    0.05,
                    0.28,
                    (bx + sx * 0.393, 0.775, z),
                    M.shelf,
                    bevel = 0.004
                );
            }
        }
    });
    join(&still, "body", Some(P3::ZERO));
    let parts = new_parts(|| {
        fbox!(
            f,
            "lid",
            1.86,
            0.05,
            0.64,
            (0, 0.845, z),
            M.body,
            bevel = 0.020
        );
        fbox!(
            f,
            "handle",
            0.62,
            0.035,
            0.030,
            (0, 0.860, z + 0.345),
            M.chrome,
            bevel = 0.010
        );
        // the hinges' knuckles, round the hinge line 1 cm in from the lid's back: it stays off the wall as it lifts
        for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
            fbox!(
                f,
                format!("hinge{i}"),
                0.08,
                0.030,
                0.030,
                (sx * 0.60, 0.845, z - 0.31),
                M.chrome,
                bevel = 0.008
            );
        }
    });
    let lid = hinged(join(&parts, "lid", None), f.at(0, 0.845, z - 0.31));
    settle_all();
    swing!(lid, "lid_open", -85, axis = Axis::X, seconds = 1.2);
    Vec::new()
}

/// The upright freezer: a tall cabinet whose door, hinged on its left with one bar handle up its right, opens on a
/// white liner and five clear drawers; once the door is open the middle drawer slides out a little.
pub fn upright_freezer() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    // the drawers: five fronts up the liner's face, 0.27 m tall at a 0.324 m pitch, each 2 cm thick
    let (drawers, pitch, tall, front_z) = (5, 0.324, 0.27, 0.175);
    let drawer_y = |i: usize| 0.10 + tall / 2.0 + pitch * i as f64;
    const OUT: usize = 2;
    let still = new_parts(|| {
        // the shell: back, the two sides, its top and floor, and the liner with its drawers
        fbox!(
            f,
            "back",
            0.60,
            1.80,
            0.04,
            (0, 0.92, -0.40),
            M.body,
            bevel = 0.010
        );
        for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
            fbox!(
                f,
                format!("side{i}"),
                0.04,
                1.80,
                0.66,
                (sx * 0.28, 0.92, -0.10),
                M.body,
                bevel = 0.010
            );
        }
        fbox!(
            f,
            "top",
            0.60,
            0.04,
            0.66,
            (0, 1.80, -0.10),
            M.body,
            bevel = 0.008
        );
        fbox!(
            f,
            "floor",
            0.60,
            0.06,
            0.66,
            (0, 0.03, -0.10),
            M.body,
            bevel = 0.006
        );
        fbox!(
            f,
            "liner",
            0.52,
            1.70,
            0.58,
            (0, 0.91, -0.13),
            M.white_liner
        );
        for i in (0..drawers).filter(|&i| i != OUT) {
            drawer_front(&f, i, drawer_y(i), tall, front_z);
        }
        fbox!(
            f,
            "plinth",
            0.56,
            0.07,
            0.60,
            (0, 0.035, -0.12),
            M.trim,
            bevel = 0.006
        );
        fbox!(
            f,
            "badge",
            0.09,
            0.028,
            0.006,
            (0.18, 1.72, 0.245),
            M.chrome
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // the drawer that slides out: its front, its tub behind it (inside the liner while shut) and two packs in it
    let y = drawer_y(OUT);
    let drawer = new_parts(|| {
        drawer_front(&f, OUT, y, tall, front_z);
        let back = front_z - 0.01;
        for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
            fbox!(
                f,
                format!("tub_side{i}"),
                0.008,
                tall - 0.04,
                0.30,
                (sx * 0.22, y - 0.02, back - 0.15),
                M.drawer,
                bevel = 0
            );
        }
        fbox!(
            f,
            "tub_floor",
            0.45,
            0.008,
            0.30,
            (0, y - tall / 2.0 + 0.004, back - 0.15),
            M.drawer,
            bevel = 0
        );
        fbox!(
            f,
            "pack_peas",
            0.16,
            0.14,
            0.10,
            (-0.10, y - tall / 2.0 + 0.078, back - 0.07),
            M.peas,
            bevel = 0.01
        );
        fbox!(
            f,
            "pack_ice",
            0.14,
            0.12,
            0.10,
            (0.09, y - tall / 2.0 + 0.068, back - 0.08),
            M.icecream,
            bevel = 0.01
        );
    });
    let drawer = join(&drawer, "drawer", None);
    let parts = new_parts(|| {
        fbox!(
            f,
            "door",
            0.56,
            1.62,
            0.030,
            (0, 0.96, 0.22),
            M.body,
            bevel = 0.012
        );
        // its inside: a white panel inside its seal
        fbox!(
            f,
            "door_liner",
            0.48,
            1.52,
            0.012,
            (0, 0.96, 0.199),
            M.white_liner,
            bevel = 0.004
        );
        // one bar handle up the free edge, on two feet
        fbox!(
            f,
            "handle",
            0.026,
            0.50,
            0.02,
            (0.225, 1.20, 0.25),
            M.chrome,
            bevel = 0.008
        );
        for (i, y) in [0.99, 1.41].into_iter().enumerate() {
            fbox!(
                f,
                format!("foot{i}"),
                0.018,
                0.03,
                0.01,
                (0.225, y, 0.237),
                M.chrome,
                bevel = 0
            );
        }
    });
    let door = hinged(join(&parts, "door", None), f.at(-0.28, 0.96, 0.235));
    settle_all();
    swing!(door, "door_open", 100, seconds = 1.2, ease = Ease::BackOut);
    slide!(drawer, "drawer_open", (0, -0.14, 0), seconds = 0.6);
    Vec::new()
}

/// A clear drawer's front on the freezer's liner, with a grip along its top.
fn drawer_front(f: &GameFrame, i: usize, y: f64, tall: f64, z: f64) {
    fbox!(
        f,
        format!("drawer{i}"),
        0.46,
        tall,
        0.02,
        (0, y, z),
        M.drawer,
        bevel = 0.006
    );
    fbox!(
        f,
        format!("grip{i}"),
        0.30,
        0.022,
        0.008,
        (0, y + tall / 2.0 - 0.025, z + 0.013),
        M.shelf,
        bevel = 0.004
    );
}
