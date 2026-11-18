//! The calls by name evaluate what they are given as Python does: the arguments given by place first, left to
//! right, then the options in the order written. A call that draws random numbers for both draws them in the
//! script's order.

use artkit::materials::Mat;
use artkit::math::Axis;
use artkit::place::GameFrame;
use artkit::scene;
use artkit::{cube, cylinder, fbox, fsweep, revolve, soft_box, sphere, sweep, swing};
use std::cell::RefCell;

const BODY: Mat = Mat::new("body", "#F3F3F1", "gloss");

thread_local! {
    static SEEN: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

/// The value, with a note that it was worked out now.
fn at<T>(what: &'static str, value: T) -> T {
    SEEN.with(|s| s.borrow_mut().push(what));
    value
}

fn seen() -> Vec<&'static str> {
    SEEN.with(|s| std::mem::take(&mut *s.borrow_mut()))
}

#[test]
fn a_call_by_name_works_out_its_arguments_in_pythons_order() {
    scene::reset();
    cube!(
        at("name", "c"),
        at("size", (0.1, 0.2, 0.3)),
        at = at("at", (0, 0, 1)),
        material = at("material", BODY),
        rot = at("rot", (0, 0, 30))
    );
    assert_eq!(seen(), ["name", "size", "at", "material", "rot"]);
    sphere!(
        at("name", "s"),
        at("radius", 0.1),
        segments = at("segments", 8),
        rings = at("rings", 4),
        material = at("material", BODY)
    );
    assert_eq!(seen(), ["name", "radius", "segments", "rings", "material"]);
    cylinder!(
        at("name", "y"),
        at("radius", 0.1),
        at("depth", 0.2),
        verts = at("verts", 8)
    );
    assert_eq!(seen(), ["name", "radius", "depth", "verts"]);
    revolve!(
        at("name", "r"),
        at("profile", &[(0.0, 0.0), (0.1, 0.0), (0.0, 0.2)]),
        segments = at("segments", 8),
        material = at("material", BODY)
    );
    assert_eq!(seen(), ["name", "profile", "segments", "material"]);
    let door = sweep!(
        at("name", "w"),
        at("points", &[(0.0, 0.0, 0.0), (0.0, 0.0, 0.5)]),
        at("radius", 0.02),
        verts = at("verts", 6),
        material = at("material", BODY)
    );
    assert_eq!(seen(), ["name", "points", "radius", "verts", "material"]);
    soft_box!(
        at("name", "p"),
        at("size", (0.4, 0.4, 0.1)),
        round = at("round", 0.02),
        puff = at("puff", 0.01)
    );
    assert_eq!(seen(), ["name", "size", "round", "puff"]);
    let frame = GameFrame::new(0.1);
    fbox!(
        at("frame", &frame),
        at("name", "b"),
        at("w", 0.1),
        at("h", 0.2),
        at("d", 0.3),
        at("at", (0, 0.5, 0)),
        at("material", BODY),
        bevel = at("bevel", 0.01),
        bevel_segments = at("bevel_segments", 1)
    );
    assert_eq!(
        seen(),
        [
            "frame",
            "name",
            "w",
            "h",
            "d",
            "at",
            "material",
            "bevel",
            "bevel_segments"
        ]
    );
    fsweep!(
        frame,
        at("name", "t"),
        at("points", &[(0, 0.0, 0.0), (0, 0.5, 0.0)]),
        at("radius", 0.02),
        at("material", BODY),
        verts = at("verts", 6)
    );
    assert_eq!(seen(), ["name", "points", "radius", "material", "verts"]);
    // (the frame is still the script's to use: a call borrows it)
    assert_eq!(frame.shift, 0.1);
    swing!(
        at("obj", door),
        at("clip", "open"),
        at("degrees", -90),
        seconds = at("seconds", 0.5),
        axis = at("axis", Axis::X)
    );
    assert_eq!(seen(), ["obj", "clip", "degrees", "seconds", "axis"]);
}
