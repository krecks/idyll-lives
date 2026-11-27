//! The freezers: the chest freezer and the upright one, each a shell with its liner, opening its lid or its door
//! for `frozenTreat`.

use super::M;
use artkit::math::Axis;
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, swing};

/// The chest freezer: a low box on the floor with its lid hinged at the back.
pub fn chest_freezer() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let z = 0.01;
    // the shell: back, front, the two ends and the floor of the box, and its liner inside
    fbox!(
        f,
        "back",
        1.90,
        0.80,
        0.05,
        (0, 0.42, z - 0.415),
        M.body,
        bevel = 0.010
    );
    fbox!(
        f,
        "front",
        1.90,
        0.80,
        0.05,
        (0, 0.42, z + 0.415),
        M.body,
        bevel = 0.010
    );
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("end{i}"),
            0.05,
            0.80,
            0.88,
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
        0.88,
        (0, 0.03, z),
        M.body,
        bevel = 0.006
    );
    fbox!(f, "liner", 1.78, 0.70, 0.76, (0, 0.40, z), M.liner);
    fbox!(f, "rim", 1.88, 0.020, 0.86, (0, 0.815, z), M.trim);
    fbox!(
        f,
        "badge",
        0.10,
        0.030,
        0.006,
        (0.78, 0.55, z + 0.442),
        M.chrome
    );
    let parts = new_parts(|| {
        fbox!(
            f,
            "lid",
            1.86,
            0.05,
            0.84,
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
            (0, 0.860, z + 0.445),
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
                (sx * 0.60, 0.845, z - 0.41),
                M.chrome,
                bevel = 0.008
            );
        }
    });
    let lid = hinged(join(&parts, "lid", None), f.at(0, 0.845, z - 0.41));
    settle_all();
    swing!(lid, "lid_open", -85, axis = Axis::X, seconds = 1.2);
    Vec::new()
}

/// The upright freezer: a tall cabinet whose door is hinged on its left, three baskets behind it.
pub fn upright_freezer() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    // the shell: back, the two sides, its top and floor, and the liner with its baskets
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
    fbox!(f, "liner", 0.50, 1.68, 0.58, (0, 0.90, -0.13), M.liner);
    for (i, y) in [0.55, 0.95, 1.35].into_iter().enumerate() {
        fbox!(
            f,
            format!("basket{i}"),
            0.44,
            0.12,
            0.020,
            (0, y, 0.17),
            M.chrome,
            bevel = 0.004
        );
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
        fbox!(f, "seam", 0.56, 0.012, 0.028, (0, 1.30, 0.222), M.trim);
        for i in 0..3 {
            fbox!(
                f,
                format!("handle{i}"),
                0.40,
                0.028,
                0.028,
                (0, 1.44 - 0.36 * i as f64, 0.245),
                M.chrome,
                bevel = 0.008
            );
        }
    });
    let door = hinged(join(&parts, "door", None), f.at(-0.28, 0.96, 0.235));
    settle_all();
    swing!(door, "door_open", 100, seconds = 1.2, ease = Ease::BackOut);
    Vec::new()
}
