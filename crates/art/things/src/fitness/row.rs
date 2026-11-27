//! The rowing machine: a front housing with the fan, a rail on the floor, a sliding seat and a handle on its cable.

use super::M;
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsphere, settle};

/// The machine's back edge (game depth): on the wall's face in a 2-deep footprint.
const BACK: f64 = -0.93;

pub fn rowing_machine() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    // the front housing: the fan in its cage, on the floor
    fbox!(
        f,
        "housing",
        0.52,
        0.44,
        0.34,
        (0, 0.22, BACK + 0.19),
        M.body,
        bevel = 0.03
    );
    fbox!(
        f,
        "cage",
        0.30,
        0.30,
        0.02,
        (0, 0.24, BACK + 0.37),
        M.dark,
        bevel = 0.01
    );
    fbox!(
        f,
        "footplate",
        0.40,
        0.26,
        0.05,
        (0, 0.30, BACK + 0.42),
        M.pads,
        bevel = 0.01
    );
    // the rail: two beams on the floor, back to the housing
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("rail{i}"),
            0.07,
            0.11,
            1.34,
            (sx * 0.11, 0.055, BACK + 0.36 + 0.67),
            M.steel,
            bevel = 0.008
        );
    }
    // the rear foot
    fbox!(
        f,
        "foot",
        0.34,
        0.07,
        0.12,
        (0, 0.035, BACK + 1.66),
        M.dark,
        bevel = 0.01
    );
    // the seat: a pad on its carriage
    fbox!(
        f,
        "seat",
        0.40,
        0.08,
        0.28,
        (0, 0.21, BACK + 0.98),
        M.pads,
        bevel = 0.02
    );
    fbox!(
        f,
        "carriage",
        0.30,
        0.06,
        0.22,
        (0, 0.14, BACK + 0.98),
        M.dark,
        bevel = 0.008
    );
    // the handle on its cable, over the front of the rail
    fbox!(
        f,
        "handle",
        0.56,
        0.035,
        0.035,
        (0, 0.52, BACK + 0.62),
        M.chrome,
        bevel = 0.01
    );
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("grip{i}"),
            0.09,
            0.045,
            0.045,
            (sx * 0.235, 0.52, BACK + 0.62),
            M.dark,
            bevel = 0.012
        );
    }
    fbox!(f, "cable", 0.02, 0.02, 0.22, (0, 0.48, BACK + 0.50), M.dark);
    fsphere!(f, "pivot", 0.03, (0, 0.44, BACK + 0.40), M.chrome);
    settle!();
    meshes()
}
