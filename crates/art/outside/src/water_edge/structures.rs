//! The jetty and the beach hut: the set's built things.

use super::{F, M};
use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::json::float_repr;
use artkit::parts::{join_all, settle_all};
use artkit::scene::Obj;
use artkit::shapes::SHARP;
use artkit::{fbox, fcyl, fsphere, loft};

/// The jetty: twelve planks across two bearers on eight posts, with a mooring bollard at each front corner.
pub fn jetty() -> Vec<Obj> {
    // the deck: planks across the width, a gap between them
    for i in 0..12 {
        let z = -1.76 + i as f64 * 0.32;
        fbox!(
            F,
            format!("plank{i}"),
            1.92,
            0.05,
            0.30,
            (0, 0.56, z),
            M.plank,
            bevel = 0.006
        );
    }
    // the bearers under the deck, and the posts that carry them down to the water's floor
    for x in [-0.78, 0.78] {
        fbox!(
            F,
            format!("bearer{}", float_repr(x)),
            0.14,
            0.14,
            3.7,
            (x, 0.475, 0),
            M.post,
            bevel = 0.008
        );
        for z in [-1.7, -0.55, 0.55, 1.7] {
            fcyl!(
                F,
                format!("post{}{}", float_repr(x), float_repr(z)),
                0.09,
                0.5,
                (x, 0.25, z),
                M.post,
                verts = 10,
                bevel = 0.004
            );
        }
    }
    // a bollard at each front corner, rounded on top
    for x in [-0.78, 0.78] {
        fcyl!(
            F,
            format!("bollard{}", float_repr(x)),
            0.08,
            0.34,
            (x, 0.755, 1.62),
            M.post,
            verts = 10,
            bevel = 0.004
        );
        fsphere!(
            F,
            format!("bollard_cap{}", float_repr(x)),
            0.09,
            (x, 0.93, 1.62),
            M.post,
            segments = 10,
            rings = 5,
            scale = (1, 1, 0.6)
        );
    }
    join_all("jetty");
    settle_all()
}

/// The beach hut: a cabin on two skids under a gable to the front, the door in the middle of it between two small
/// windows, white bargeboards along the gable; stripes down the sides and the back. Its walls are a door's height
/// to the eaves (2 m) and its ridge as high as Hove's huts' (2.46 m).
pub fn beach_hut() -> Vec<Obj> {
    // skids, floor and the painted body
    for x in [-0.62, 0.62] {
        fbox!(
            F,
            format!("skid{}", float_repr(x)),
            0.12,
            0.12,
            1.86,
            (x, 0.06, 0),
            M.timber,
            bevel = 0.01
        );
    }
    fbox!(
        F,
        "floor",
        1.8,
        0.08,
        1.8,
        (0, 0.16, 0),
        M.timber,
        bevel = 0.008
    );
    fbox!(
        F,
        "body",
        1.76,
        1.8,
        1.7,
        (0, 1.1, 0),
        M.wall,
        bevel = 0.012
    );
    // the stripes: down each side and the back (the front is the door's)
    for (i, z) in [-0.8, -0.48, -0.16, 0.16, 0.48, 0.8]
        .into_iter()
        .enumerate()
    {
        for x in [-0.89, 0.89] {
            fbox!(
                F,
                format!("stripe_side{i}{}", float_repr(x)),
                0.02,
                1.78,
                0.16,
                (x, 1.09, z),
                M.stripe,
                bevel = 0.004
            );
        }
    }
    for (i, x) in [-0.80, -0.48, -0.16, 0.16, 0.48, 0.80]
        .into_iter()
        .enumerate()
    {
        fbox!(
            F,
            format!("stripe_back{i}"),
            0.16,
            1.78,
            0.02,
            (x, 1.09, -0.86),
            M.stripe,
            bevel = 0.004
        );
    }
    // the door in the middle, its frame and knob
    fbox!(
        F,
        "door",
        0.76,
        1.72,
        0.05,
        (0, 1.07, 0.875),
        M.door,
        bevel = 0.006
    );
    fbox!(
        F,
        "door_frame_l",
        0.05,
        1.76,
        0.04,
        (-0.405, 1.09, 0.885),
        M.trim,
        bevel = 0.003
    );
    fbox!(
        F,
        "door_frame_r",
        0.05,
        1.76,
        0.04,
        (0.405, 1.09, 0.885),
        M.trim,
        bevel = 0.003
    );
    fbox!(
        F,
        "door_frame_t",
        0.86,
        0.05,
        0.04,
        (0, 1.955, 0.885),
        M.trim,
        bevel = 0.003
    );
    fsphere!(
        F,
        "knob",
        0.03,
        (0.3, 1.05, 0.91),
        M.brass,
        segments = 8,
        rings = 4
    );
    // a small window either side of the door, dark from outside, in a white frame
    for x in [-0.64, 0.64] {
        let n = float_repr(x);
        fbox!(
            F,
            format!("window{n}"),
            0.22,
            0.38,
            0.02,
            (x, 1.55, 0.86),
            M.pane
        );
        fbox!(
            F,
            format!("window_l{n}"),
            0.04,
            0.46,
            0.05,
            (x - 0.13, 1.55, 0.87),
            M.trim,
            bevel = 0.003
        );
        fbox!(
            F,
            format!("window_r{n}"),
            0.04,
            0.46,
            0.05,
            (x + 0.13, 1.55, 0.87),
            M.trim,
            bevel = 0.003
        );
        fbox!(
            F,
            format!("window_t{n}"),
            0.30,
            0.04,
            0.05,
            (x, 1.76, 0.87),
            M.trim,
            bevel = 0.003
        );
        fbox!(
            F,
            format!("window_b{n}"),
            0.30,
            0.04,
            0.05,
            (x, 1.34, 0.87),
            M.trim,
            bevel = 0.003
        );
    }
    // the gable: two panels at 27 degrees meeting over the middle, the ridge running front to back, a cap on it
    for (i, s) in [-1.0_f64, 1.0].into_iter().enumerate() {
        fbox!(
            F,
            format!("roof{i}"),
            1.08,
            0.05,
            1.9,
            (s * 0.48, 2.232, 0),
            M.roof,
            bevel = 0.008,
            rot = (0, -27.0 * s, 0)
        );
        // a white bargeboard under each slope's front and back edge
        for z in [-0.93, 0.93] {
            fbox!(
                F,
                format!("barge{i}{}", float_repr(z)),
                1.0,
                0.07,
                0.025,
                (s * 0.46, 2.175, z),
                M.trim,
                bevel = 0.004,
                rot = (0, -27.0 * s, 0)
            );
        }
    }
    fbox!(
        F,
        "ridge",
        0.12,
        0.06,
        1.92,
        (0, 2.48, 0),
        M.roof,
        bevel = 0.008
    );
    // the gable ends, closing the roof's triangle over the front and back walls
    for (i, s) in [-1.0_f64, 1.0].into_iter().enumerate() {
        let (z0, z1) = (s * 0.83, s * 0.85);
        let tri = |z: f64| {
            if s > 0.0 {
                vec![
                    F.at(-0.85, 1.98, z),
                    F.at(0.85, 1.98, z),
                    F.at(0.0, 2.43, z),
                ]
            } else {
                vec![
                    F.at(-0.85, 1.98, z),
                    F.at(0.0, 2.43, z),
                    F.at(0.85, 1.98, z),
                ]
            }
        };
        let gable = loft!(format!("gable{i}"), &[tri(z0), tri(z1)], material = M.wall);
        bevel_edges(gable, 0.004, 2, SHARP, &EdgeOrder::Corners);
    }
    // a step up to the door
    fbox!(
        F,
        "step",
        0.9,
        0.10,
        0.14,
        (0, 0.05, 0.92),
        M.timber,
        bevel = 0.008
    );
    join_all("beachHut");
    settle_all()
}
