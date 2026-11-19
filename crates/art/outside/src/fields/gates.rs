//! The field gate and the fingerpost: oak and iron at the lane, and a post with three painted arms where the lanes
//! meet.

use super::{F, M};
use artkit::parts::{join_all, settle};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere};

/// A five-bar gate between two oak posts, with a diagonal brace, three iron hinges and a latch.
pub fn gate() -> Vec<Obj> {
    // the posts: 0.18 m through, so the hinges and latch stay inside the 0.2 m footprint
    for s in [-1.0, 1.0] {
        let x = s * 0.89;
        fbox!(
            F,
            format!("post{s}"),
            0.12,
            1.30,
            0.18,
            (x, 0.65, 0.0),
            M.oak_dark,
            bevel = 0.010,
            bevel_segments = 2
        );
        fbox!(
            F,
            format!("postCap{s}"),
            0.14,
            0.05,
            0.20,
            (x, 1.325, 0.0),
            M.oak,
            bevel = 0.008,
            bevel_segments = 1
        );
    }
    // the five bars and two stiles of the leaf
    for (i, y) in [0.25, 0.45, 0.65, 0.85, 1.05].iter().enumerate() {
        fbox!(
            F,
            format!("bar{i}"),
            1.62,
            0.07,
            0.05,
            (0.0, *y, 0.0),
            M.oak,
            bevel = 0.006,
            bevel_segments = 1
        );
    }
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            format!("stile{s}"),
            0.07,
            0.94,
            0.06,
            (s * 0.78, 0.65, 0.0),
            M.oak_dark,
            bevel = 0.006,
            bevel_segments = 1
        );
    }
    // the brace, a strip from the lower left to the upper right, cut as one outline so it stays in the leaf's plane
    let (x0, y0, x1, y1) = (-0.68, 0.30, 0.68, 1.02);
    let (dx, dy): (f64, f64) = (x1 - x0, y1 - y0);
    let len = dx.hypot(dy);
    let (px, py) = (-dy / len * 0.035, dx / len * 0.035);
    let brace = [
        (x0 + px, y0 + py),
        (x1 + px, y1 + py),
        (x1 - px, y1 - py),
        (x0 - px, y0 - py),
    ];
    fextrude!(
        F,
        "brace",
        &brace,
        0.05,
        (0.0, 0.0, -0.025),
        M.oak_dark,
        bevel = 0.006,
        bevel_segments = 1
    );
    // three hinges on the left post, their straps reaching onto the leaf
    for (i, y) in [0.35, 0.65, 0.95].iter().enumerate() {
        fbox!(
            F,
            format!("hinge{i}"),
            0.20,
            0.05,
            0.012,
            (-0.78, *y, 0.092),
            M.iron,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    // the latch plate and its handle on the right
    fbox!(
        F,
        "latchPlate",
        0.08,
        0.12,
        0.020,
        (0.84, 0.90, 0.088),
        M.iron,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        "latchHandle",
        0.06,
        0.035,
        0.030,
        (0.80, 0.90, 0.095),
        M.iron,
        bevel = 0.004,
        bevel_segments = 1
    );
    settle(&[join_all("gate")])
}

/// A fingerpost: an iron post on a stone footing, with three white arms at different heights, each pointed the way.
pub fn fingerpost() -> Vec<Obj> {
    fcyl!(
        F,
        "footing",
        0.14,
        0.12,
        (0.0, 0.06, 0.0),
        M.stone,
        verts = 10,
        bevel = 0.006,
        bevel_segments = 2
    );
    fcyl!(F, "post", 0.055, 2.18, (0.0, 1.21, 0.0), M.iron, verts = 10);
    fsphere!(F, "finial", 0.07, (0.0, 2.31, 0.0), M.iron);
    // the three arms: a flat blade with a pointed end, swung out over the lane
    let arm = [
        (0.02, -0.045),
        (0.34, -0.045),
        (0.42, 0.0),
        (0.34, 0.045),
        (0.02, 0.045),
    ];
    for (i, (y, turn)) in [(1.55, 0.0), (1.78, 45.0), (2.01, 180.0)]
        .iter()
        .enumerate()
    {
        fcyl!(
            F,
            format!("collar{i}"),
            0.075,
            0.035,
            (0.0, y - 0.075, 0.0),
            M.iron,
            verts = 10
        );
        fextrude!(
            F,
            format!("arm{i}"),
            &arm,
            0.035,
            (0.0, *y, -0.0175),
            M.paint,
            rot = (0.0, 0.0, *turn),
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    settle(&[join_all("fingerpost")])
}
