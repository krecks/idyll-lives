//! The playhouse: a board hut with a framed doorway and step, a framed window in each side, corner posts and a
//! pitched roof over board gables.

use super::M;
use artkit::parts::join_all;
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::place::Facing;
use artkit::{fbox, fextrude, settle};

pub fn playhouse() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (w, d, h) = (1.16, 1.40, 1.20);
    let mid = 0.06 + h / 2.0;
    fbox!(f, "floor", w, 0.06, d, (0, 0.03, 0), M.ph_board, bevel = 0.006);
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
    // a side wall with its window cut out, the white frame round it and the pane set in
    let rect = |cx: f64, cy: f64, a: f64, b: f64| {
        vec![
            (cx - a / 2.0, cy - b / 2.0),
            (cx + a / 2.0, cy - b / 2.0),
            (cx + a / 2.0, cy + b / 2.0),
            (cx - a / 2.0, cy + b / 2.0),
        ]
    };
    let (wc, wy) = (0.0, 0.06 + 0.72);
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        let x = sx * (w / 2.0 - 0.03);
        fextrude!(
            f,
            format!("side{i}"),
            &rect(0.0, 0.06 + h / 2.0, d, h),
            0.06,
            (x, 0, 0),
            M.ph_board,
            plane = Facing::Side,
            holes = vec![rect(wc, wy, 0.40, 0.34)],
            bevel = 0.004
        );
        fextrude!(
            f,
            format!("frame{i}"),
            &rect(wc, wy, 0.48, 0.42),
            0.075,
            (x + sx * 0.008, 0, 0),
            M.house_trim,
            plane = Facing::Side,
            holes = vec![rect(wc, wy, 0.36, 0.30)],
            bevel = 0.004
        );
        fbox!(
            f,
            format!("pane{i}"),
            0.012,
            0.31,
            0.37,
            (x, wy, wc),
            M.ph_glass
        );
        fbox!(
            f,
            format!("sill{i}"),
            0.10,
            0.03,
            0.56,
            (x + sx * 0.04, wy - 0.225, wc),
            M.house_trim,
            bevel = 0.004
        );
    }
    // the front: a panel either side of the doorway, the header over it, the white door frame and a step
    let door = 0.66;
    let side = (w - door) / 2.0;
    let door_h = 0.95;
    let zf = d / 2.0 - 0.03;
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("front{i}"),
            side,
            h,
            0.06,
            (sx * (door / 2.0 + side / 2.0), mid, zf),
            M.ph_board,
            bevel = 0.006
        );
        fbox!(
            f,
            format!("jamb{i}"),
            0.06,
            door_h + 0.04,
            0.075,
            (sx * (door / 2.0 - 0.03), 0.06 + (door_h + 0.04) / 2.0, zf + 0.008),
            M.house_trim,
            bevel = 0.004
        );
    }
    fbox!(
        f,
        "header",
        door,
        h - door_h,
        0.06,
        (0, 0.06 + door_h + (h - door_h) / 2.0, zf),
        M.ph_board,
        bevel = 0.006
    );
    fbox!(
        f,
        "lintel",
        door + 0.06,
        0.06,
        0.075,
        (0, 0.06 + door_h + 0.03, zf + 0.008),
        M.house_trim,
        bevel = 0.004
    );
    fbox!(
        f,
        "step",
        door,
        0.04,
        0.08,
        (0, 0.02, d / 2.0 + 0.03),
        M.ph_board,
        bevel = 0.006
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
    // the roof: a board gable at each end, two sloped slabs over them, a ridge cap
    let top = 0.06 + h;
    let (rise, half) = (0.28, w / 2.0 + 0.07);
    let tri = [(-w / 2.0, 0.0), (w / 2.0, 0.0), (0.0, rise)];
    for (i, sz) in [-1.0f64, 1.0].into_iter().enumerate() {
        fextrude!(
            f,
            format!("gable{i}"),
            &tri,
            0.06,
            (0, top, sz * (d / 2.0 - 0.03)),
            M.ph_board,
            plane = Facing::Front,
            bevel = 0.006
        );
    }
    let len = half.hypot(rise) + 0.02;
    let ang = rise.atan2(half).to_degrees();
    for (i, sx) in [-1.0f64, 1.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("roof{i}"),
            len,
            0.04,
            d + 0.14,
            (sx * half / 2.0, top + rise / 2.0 + 0.02, 0),
            M.ph_roof,
            bevel = 0.006,
            rot = (0, -sx * ang, 0)
        );
    }
    fbox!(
        f,
        "ridge",
        0.09,
        0.05,
        d + 0.14,
        (0, top + rise + 0.035, 0),
        M.ph_roof,
        bevel = 0.008
    );
    join_all("playhouse");
    settle!();
    meshes()
}
