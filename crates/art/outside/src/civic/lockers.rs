//! The lockers: see NOTES.md.

use super::common::{F, M, WALL};
use artkit::json::float_repr;
use artkit::parts::{join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, settle};

// --- the lockers: 2x1 against a wall, five columns of two, vents, locks and name cards -----------------------------
pub fn locker_row() -> Vec<Obj> {
    let d = 0.45;
    let front = WALL + d;
    let parts = new_parts(|| {
        fbox!(
            F,
            "plinth",
            1.88,
            0.1,
            d - 0.04,
            (0, 0.05, WALL + (d - 0.04) / 2.0),
            M.plinth
        );
        fbox!(
            F,
            "carcass",
            1.9,
            1.72,
            d - 0.02,
            (0, 0.1 + 0.86, WALL + (d - 0.02) / 2.0),
            M.frame,
            bevel = 0.004
        );
        fbox!(
            F,
            "top",
            1.92,
            0.025,
            d,
            (0, 1.8325, WALL + d / 2.0),
            M.frame,
            bevel = 0.005
        );
        for c in 0..5 {
            let x = -0.76 + c as f64 * 0.38;
            for t in 0..2 {
                let y0 = 0.12 + t as f64 * 0.85;
                let y = y0 + 0.415;
                fbox!(
                    F,
                    format!("door{c}{t}"),
                    0.36,
                    0.83,
                    0.016,
                    (x, y, front - 0.012),
                    M.locker,
                    bevel = 0.004
                );
                for v in 0..4 {
                    for yy in [y0 + 0.73, y0 + 0.1] {
                        fbox!(
                            F,
                            format!("vent{c}{t}{v}{}", float_repr(yy)),
                            0.18,
                            0.008,
                            0.004,
                            (x, yy + v as f64 * 0.022 - 0.033, front - 0.003),
                            M.slot
                        );
                    }
                }
                artkit::bevel::eased(fcyl!(
                    F,
                    format!("lock{c}{t}"),
                    0.014,
                    0.012,
                    (x + 0.13, y, front - 0.002),
                    M.bell,
                    rot = (90, 0, 0),
                    verts = 10
                ));
                fbox!(
                    F,
                    format!("tag_frame{c}{t}"),
                    0.11,
                    0.05,
                    0.004,
                    (x, y0 + 0.62, front - 0.002),
                    M.frame
                );
                fbox!(
                    F,
                    format!("tag{c}{t}"),
                    0.095,
                    0.036,
                    0.004,
                    (x, y0 + 0.62, front - 0.0005),
                    M.label
                );
                fbox!(
                    F,
                    format!("number{c}{t}"),
                    0.04,
                    0.03,
                    0.003,
                    (x, y0 + 0.77, front - 0.001),
                    M.label
                );
            }
        }
    });
    join(&parts, "lockers", Some((0, 0, 0).into()));
    settle!(centre = false);
    meshes()
}
