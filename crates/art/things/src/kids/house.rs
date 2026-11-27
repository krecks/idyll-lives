//! The playhouse: a board hut with its doorway, a window in its side, corner posts and its roof.

use super::M;
use artkit::parts::join_all;
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, settle};

pub fn playhouse() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (w, d, h) = (1.16, 1.40, 1.40);
    let mid = 0.06 + h / 2.0;
    fbox!(
        f,
        "floor",
        w,
        0.06,
        d,
        (0, 0.03, 0),
        M.ph_board,
        bevel = 0.006
    );
    fbox!(
        f,
        "back",
        w,
        h,
        0.06,
        (0, mid, -d / 2.0 + 0.03),
        M.ph_board,
        bevel = 0.006
    );
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("side{i}"),
            0.06,
            h,
            d,
            (sx * (w / 2.0 - 0.03), mid, 0),
            M.ph_board,
            bevel = 0.006
        );
    }
    // the front: a panel either side of the doorway, and the header over it
    let door = 0.66;
    let side = (w - door) / 2.0;
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("front{i}"),
            side,
            h,
            0.06,
            (sx * (door / 2.0 + side / 2.0), mid, d / 2.0 - 0.03),
            M.ph_board,
            bevel = 0.006
        );
    }
    let door_h = 1.05;
    fbox!(
        f,
        "header",
        door,
        h - door_h,
        0.06,
        (0, 0.06 + door_h + (h - door_h) / 2.0, d / 2.0 - 0.03),
        M.ph_board,
        bevel = 0.006
    );
    // the window in the left side, seen from the front right (the front either side of the doorway is too narrow for one)
    fbox!(
        f,
        "window",
        0.020,
        0.30,
        0.34,
        (-(w / 2.0 - 0.005), 1.05, 0),
        M.ph_glass,
        bevel = 0.004
    );
    for (i, (sx, sz)) in [(-1.0f64, -1.0f64), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            f,
            format!("post{i}"),
            0.09,
            h,
            0.09,
            (sx * (w / 2.0 - 0.045), mid, sz * (d / 2.0 - 0.045)),
            M.house_trim,
            bevel = 0.006
        );
    }
    fbox!(
        f,
        "roof",
        w + 0.14,
        0.09,
        d + 0.14,
        (0, 0.06 + h + 0.045, 0),
        M.ph_roof,
        bevel = 0.010
    );
    fbox!(
        f,
        "ridge",
        w + 0.16,
        0.05,
        0.16,
        (0, 0.06 + h + 0.115, 0),
        M.ph_roof,
        bevel = 0.008
    );
    join_all("playhouse");
    settle!();
    meshes()
}
