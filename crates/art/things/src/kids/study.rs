//! KD3: the kids' study and art things.

use super::common::{fillet, heart, place};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::parts::join_all;
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fextrude, fsweep};

pub fn kids_chair() -> Vec<Obj> {
    // a rounded seat, legs splayed a little, the back legs running up into posts behind a back with a heart cut
    // through it, rungs along the sides and across the front
    fextrude!(F, "seat", &rounded_rect(0.32, 0.3, 0.05, 3), 0.03, (0, 0.29, 0.05), M.kchair_seat, plane = Facing::Top, bevel = 0.008, bevel_segments = 2);
    for sx in [-1.0, 1.0] {
        fsweep!(F, format!("leg{sx}"), &[(sx * 0.13, 0.28, 0.165), (sx * 0.135, 0.0, 0.172)], 0.0175, M.kchair, verts = 8, scale = [1.0, 0.85]);
        fsweep!(F, format!("post{sx}"), &fillet(&[(sx * 0.135, 0.0, -0.075), (sx * 0.13, 0.28, -0.07), (sx * 0.13, 0.5, -0.1)], 0.06, 3), 0.016, M.kchair, verts = 8);
        fsweep!(F, format!("rung{sx}"), &[(sx * 0.133, 0.1, -0.072), (sx * 0.133, 0.1, 0.168)], 0.009, M.kchair, verts = 6);
    }
    fsweep!(F, "rung_front", &[(-0.133, 0.14, 0.168), (0.133, 0.14, 0.168)], 0.008, M.kchair, verts = 6);
    fextrude!(F, "back", &rounded_rect(0.32, 0.18, 0.06, 3), 0.028, (0, 0.48, -0.1), M.kchair, rot = (-6, 0, 0), holes = [heart(0.0, 0.005, 0.04, 20)], bevel = 0.008, bevel_segments = 2);
    join_all("kidsChair");
    place(0.0);
    meshes()
}
