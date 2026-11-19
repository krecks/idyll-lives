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

/// The beach hut: a striped cabin on two skids, its door and window to the front, under a shallow gable.
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
    fbox!(F, "floor", 1.8, 0.08, 1.8, (0, 0.16, 0), M.timber, bevel = 0.008);
    fbox!(F, "body", 1.76, 1.62, 1.7, (0, 1.01, 0), M.wall, bevel = 0.012);
    // the stripes: down the front and each side
    for (i, x) in [-0.80, -0.48, -0.16, 0.16, 0.48, 0.80].into_iter().enumerate() {
        fbox!(
            F,
            format!("stripe_front{i}"),
            0.16,
            1.6,
            0.02,
            (x, 1.0, 0.86),
            M.stripe,
            bevel = 0.004
        );
    }
    for (i, z) in [-0.8, -0.48, -0.16, 0.16, 0.48, 0.8].into_iter().enumerate() {
        for x in [-0.89, 0.89] {
            fbox!(
                F,
                format!("stripe_side{i}{}", float_repr(x)),
                0.02,
                1.6,
                0.16,
                (x, 1.0, z),
                M.stripe,
                bevel = 0.004
            );
        }
    }
    // the door, its frame and knob
    fbox!(F, "door", 0.62, 1.45, 0.05, (-0.35, 0.925, 0.875), M.door, bevel = 0.006);
    fbox!(F, "door_frame_l", 0.05, 1.49, 0.04, (-0.685, 0.945, 0.885), M.trim, bevel = 0.003);
    fbox!(F, "door_frame_r", 0.05, 1.49, 0.04, (-0.015, 0.945, 0.885), M.trim, bevel = 0.003);
    fbox!(F, "door_frame_t", 0.72, 0.05, 0.04, (-0.35, 1.695, 0.885), M.trim, bevel = 0.003);
    fsphere!(
        F,
        "knob",
        0.03,
        (-0.10, 1.0, 0.91),
        M.brass,
        segments = 8,
        rings = 4
    );
    // the window and its frame
    fbox!(F, "window", 0.5, 0.42, 0.03, (0.45, 1.32, 0.865), M.glass, bevel = 0.002);
    fbox!(F, "window_l", 0.04, 0.50, 0.05, (0.18, 1.32, 0.87), M.trim, bevel = 0.003);
    fbox!(F, "window_r", 0.04, 0.50, 0.05, (0.72, 1.32, 0.87), M.trim, bevel = 0.003);
    fbox!(F, "window_t", 0.58, 0.04, 0.05, (0.45, 1.55, 0.87), M.trim, bevel = 0.003);
    fbox!(F, "window_b", 0.58, 0.04, 0.05, (0.45, 1.09, 0.87), M.trim, bevel = 0.003);
    // the gable: two panels meeting over the middle, and a cap on the ridge
    fbox!(
        F,
        "roof_back",
        1.96,
        0.05,
        0.98,
        (0, 1.985, -0.44),
        M.roof,
        bevel = 0.008,
        rot = (-20, 0, 0)
    );
    fbox!(
        F,
        "roof_front",
        1.96,
        0.05,
        0.98,
        (0, 1.985, 0.44),
        M.roof,
        bevel = 0.008,
        rot = (20, 0, 0)
    );
    fbox!(F, "ridge", 1.98, 0.06, 0.12, (0, 2.155, 0), M.roof, bevel = 0.008);
    // the gable ends, closing the roof's triangle over the side walls (the ridge runs along x)
    for (i, s) in [-1.0_f64, 1.0].into_iter().enumerate() {
        let (x0, x1) = (s * 0.86, s * 0.88);
        let tri = |x: f64| {
            if s > 0.0 {
                vec![
                    F.at(x, 1.80, -0.85),
                    F.at(x, 2.12, 0.0),
                    F.at(x, 1.80, 0.85),
                ]
            } else {
                vec![
                    F.at(x, 1.80, -0.85),
                    F.at(x, 1.80, 0.85),
                    F.at(x, 2.12, 0.0),
                ]
            }
        };
        let gable = loft!(
            format!("gable{i}"),
            &[tri(x0), tri(x1)],
            material = M.wall
        );
        bevel_edges(gable, 0.004, 2, SHARP, &EdgeOrder::Corners);
    }
    // a step up to the door
    fbox!(F, "step", 0.8, 0.10, 0.18, (-0.35, 0.05, 0.94), M.timber, bevel = 0.008);
    join_all("beachHut");
    settle_all()
}
