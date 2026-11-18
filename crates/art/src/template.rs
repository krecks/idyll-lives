//! The family to copy: a toy chest whose lid swings open and whose drawer slides out. It is no family of the
//! registry (`cargo art template --preview` draws it; nothing writes it), and a test builds it
//! (tests/template.rs). To start a family, copy it to `crates/art/<group>/src/<family>/mod.rs` and rename the
//! family, its models and their keys (crates/art/README.md, "A new family"). The rules (size, budgets, finishes,
//! moving parts) are docs/design/objects.md's.

use artkit::family::{Family, Model};
use artkit::json;
use artkit::math::Axis;
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsweep, palette, slide, soft, swing};

/// Its name on the command line, `cargo art template --preview`.
pub const NAME: &str = "template";

// 1. The palette: one material per colour, named for what it is, with the finish the game draws it with. The names
//    are the material names in the file and the keys of the pack's `materials`.
palette! { M:
    body = ("#7FA7C9", satin),   // the painted box; the main part the player colours (`paint` below)
    trim = ("#F1E9DA", satin),   // plinth and lid
    inside = ("#C9B79A", matte), // seen with the lid open
    brass = ("#C9A54A", metal),  // knob and handle
}

// Sizes in metres, placed in game numbers through a `GameFrame`: (x to the right, height, depth to the front). The
// origin is the middle of the footprint, on the floor (`settle_all` puts it there).
const F: GameFrame = GameFrame::new(0.0);
const W: f64 = 0.8;
const D: f64 = 0.45;
const H: f64 = 0.5;
/// The lid's thickness.
const LID: f64 = 0.04;
/// The drawer's height, above a 4 cm plinth.
const DRAWER: f64 = 0.12;
const WALL: f64 = 0.02;

/// The chest above the drawer: a floor and four walls, so the open lid shows an inside.
fn box_walls() {
    let (y0, y1) = (0.04 + DRAWER, H - LID);
    let h = y1 - y0;
    fbox!(F, "floor", W, WALL, D, (0, y0 + WALL / 2.0, 0), M.inside);
    for (name, z) in [
        ("wall_front", D / 2.0 - WALL / 2.0),
        ("wall_back", -(D / 2.0 - WALL / 2.0)),
    ] {
        fbox!(
            F,
            name,
            W,
            h,
            WALL,
            (0, y0 + h / 2.0, z),
            M.body,
            bevel = 0.004
        );
    }
    for (name, x) in [
        ("wall_left", -(W / 2.0 - WALL / 2.0)),
        ("wall_right", W / 2.0 - WALL / 2.0),
    ] {
        fbox!(
            F,
            name,
            WALL,
            h,
            D - 2.0 * WALL,
            (x, y0 + h / 2.0, 0),
            M.body,
            bevel = 0.004
        );
    }
    // the lining, a little in from the walls
    let lining = (W - 2.0 * WALL - 0.004, D - 2.0 * WALL - 0.004);
    fbox!(
        F,
        "lining",
        lining.0,
        0.004,
        lining.1,
        (0, y0 + WALL + 0.002, 0),
        M.inside
    );
}

/// The chest: its still body, the lid on its hinge and the drawer, each a part of its own, and a clip for each.
pub fn chest() -> Vec<Obj> {
    // 2. The part that stays still, from boxes and forms (`extrude`, `sweep`, `soft`, `loft`), joined into one
    //    part with its origin on the floor's middle. `new_parts` gives what is made inside it.
    let body = new_parts(|| {
        // the plinth: its front outline, an arch cut up between two feet, extruded front to back
        let arch: Vec<(f64, f64)> = (-10..=10)
            .map(|i| {
                let t = i as f64 / 10.0;
                (t * 0.28, 0.025 * (1.0 - t * t))
            })
            .collect();
        let half = (W - 0.04) / 2.0;
        let mut outline = vec![(-half, 0.0)];
        outline.extend(arch);
        outline.extend([(half, 0.0), (half, 0.04), (-half, 0.04)]);
        fextrude!(
            F,
            "plinth",
            &outline,
            D - 0.04,
            (0, 0, 0),
            M.trim,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "drawer_case",
            W,
            DRAWER,
            D,
            (0, 0.04 + DRAWER / 2.0, 0),
            M.body,
            bevel = 0.004
        );
        box_walls();
    });
    join(&body, "body", Some((0, 0, 0).into()));

    // 3. A moving part: its own part, joined from its pieces, its origin on the hinge: the lid hinges on its back
    //    edge, at the top of the walls.
    let lid = new_parts(|| {
        soft!(
            F,
            "lid",
            W + 0.02,
            LID,
            D + 0.02,
            (0, H - LID / 2.0, 0),
            M.trim,
            round = 0.012
        );
        fbox!(
            F,
            "lid_knob",
            0.08,
            0.02,
            0.02,
            (0, H - LID / 2.0, D / 2.0 + 0.02),
            M.brass,
            bevel = 0.004
        );
    });
    let lid = hinged(join(&lid, "lid", None), F.at(0, H - LID, -(D / 2.0 + 0.01)));

    // A drawer slides from where it is: its origin can stay where it is.
    let drawer = new_parts(|| {
        let y = 0.04 + DRAWER / 2.0;
        fbox!(
            F,
            "drawer_front",
            W - 0.06,
            DRAWER - 0.02,
            0.02,
            (0, y, D / 2.0 + 0.01),
            M.body,
            bevel = 0.004
        );
        // a bow handle: one tube swept through four points, its bends curved
        let handle = [
            (-0.07, y, D / 2.0 + 0.012),
            (-0.055, y, D / 2.0 + 0.035),
            (0.055, y, D / 2.0 + 0.035),
            (0.07, y, D / 2.0 + 0.012),
        ];
        fsweep!(
            F,
            "drawer_handle",
            &handle,
            0.007,
            M.brass,
            verts = 8,
            resolution = 4
        );
        // the tray behind the front, hidden in the case until the drawer is out
        fbox!(
            F,
            "drawer_tray",
            W - 0.1,
            DRAWER - 0.04,
            D - 0.08,
            (0, y, -0.02),
            M.inside
        );
    });
    let drawer = join(&drawer, "drawer", None);

    // 4. On the floor, centred: it moves everything, hinges included, and never scales. Before the clips, which
    //    key the rest pose where it is.
    settle_all();

    // 5. A clip a moving part, named for the pack's `motions`. The first key is the rest pose (closed); the clip
    //    opens it. A turn is about the part's own axis, and an offset is in the kernel's axes (x to the left, y to
    //    the back, z up): about x, a negative angle lifts the lid's front; -y pulls the drawer out to the front.
    swing!(lid, "lid_open", -100, seconds = 1.0, axis = Axis::X);
    slide!(drawer, "drawer_open", (0, -0.3, 0), seconds = 0.8);
    meshes()
}

/// The family: its folder and keys are the name's (models/template/<id>.glb, model.template.<id>).
pub fn family() -> Family {
    Family::new(
        NAME,
        M.palette(),
        vec![
            // 6. The size it must come out at (game width, height, depth; knob and handle included), within 2 %.
            Model::new("chest", chest, (0.82, 0.5, 0.51))
                // which use plays which clip (an interaction's id or its animation tag), and for how long after
                // the use starts (seconds of the residents' time; absent: until the use ends)
                .motions(
                    json!({"uses": {"fetch": "lid_open", "putAway": "drawer_open"}, "hold": 2.5}),
                )
                // the materials the player's colour goes on
                .paint(&[("body", "Body")]),
        ],
        "Template models (crates/art/src/template.rs, built by cargo art): what the family is.",
    )
}
