//! The microwave counter and the small appliances: the coffee machine, the breakfast counter and two dishwashers,
//! each on a base unit under a worktop. A file an appliance; the base unit they share is here.

use crate::common::shaker;
use crate::kitchen_cooking::M;
use artkit::fbox;
use artkit::materials::Mat;
use artkit::place::GameFrame;

mod breakfast_counter;
mod coffee_machine;
mod dishwasher;
mod microwave;

pub use breakfast_counter::breakfast_counter;
pub use coffee_machine::coffee_machine;
pub use dishwasher::{dishwasher, dishwasher_compact};
pub use microwave::microwave;

/// A base unit like the old part's `k_base`: a plinth, a carcass and a worktop to 0.95 with a rounded front edge,
/// a shaker drawer over two shaker doors on the front (the front face at depth 0.165), bar pulls.
pub(super) fn counter(f: &GameFrame, w: f64, body: Option<Mat>, top: Option<Mat>) {
    let (body, top) = (body.unwrap_or(M.cab), top.unwrap_or(M.worktop));
    fbox!(
        f,
        "plinth",
        w - 0.04,
        0.10,
        0.54,
        (0, 0.05, -0.19),
        M.dark,
        bevel = 0.005
    );
    fbox!(
        f,
        "carcass",
        w - 0.004,
        0.82,
        0.60,
        (0, 0.51, -0.16),
        body,
        bevel = 0.006
    );
    fbox!(
        f,
        "worktop",
        w,
        0.04,
        0.66,
        (0, 0.93, -0.15),
        top,
        bevel = 0.01,
        bevel_segments = 2
    );
    shaker(
        f,
        "drawer",
        w - 0.02,
        0.14,
        (0, 0.84, 0.155),
        body,
        0.02,
        0.04,
        0.004,
    );
    pull!(
        f,
        "drawer_pull",
        (-0.09, 0.84),
        (0.09, 0.84),
        0.165,
        out = 0.02,
        w = 0.012,
        r = None,
        verts = 6
    );
    for s in [-1, 1] {
        let sf = s as f64;
        shaker(
            f,
            &format!("door{s}"),
            w / 2.0 - 0.015,
            0.66,
            (sf * w / 4.0, 0.44, 0.155),
            body,
            0.02,
            0.06,
            0.004,
        );
        pull!(
            f,
            format!("door_pull{s}"),
            (sf * 0.05, 0.62),
            (sf * 0.05, 0.74),
            0.165,
            out = 0.02,
            w = 0.012,
            r = None,
            verts = 6
        );
    }
}
