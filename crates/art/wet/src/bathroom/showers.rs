//! The showers: a tray, walls, a screen or curtain, and a head that rains while a resident showers.
//!
//! Each fills its tile, as its old part does (the resident stands inside, at the middle). The rain is the moving
//! part: thin streams built full length below the head, joined with the origin on the head and scaled to nothing
//! along their length, so they are hidden at rest; `water_on` grows them down for the whole use (the sinks'
//! `water_on` way). Glass is its own draw call: tray, walls and metal (1), glass (1), the rain (1).

use super::M;
use super::common::{bends, bottle, wall_rose};
use super::shower_parts::{
    bar_mixer, caddy, curtain, curtain_rings, frame, grate, hand_shower, head, rain, tray,
};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{Axis, cos, radians, sin};
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, grow, slide};

// The base shower's sliding door goes this far left (Blender +x is the game's -x), behind the fixed panel; the
// stall's front curtain is gathered to this part of its length at rest.
const SHOWER_SLIDE: f64 = 0.41;
const STALL_GATHER: f64 = 0.32;

pub fn shower() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let top = 2.15;
    let still = new_parts(|| {
        tray(&f, "tray", 0.9, 0.9, 0.058, M.tray, 0.035, 0.012, 10);
        grate(
            &f, "drain", 0.1, 0.047, 0.1, 0.09, 0.09, M.fittings, 5, true,
        );
        kit::tiled_walls!(
            &f,
            (-0.45, 0.45),
            (-0.45, 0.45),
            (0.058, top),
            0.03,
            0.3,
            0.005,
            M.grout,
            M.tile
        );
        // a glass side, and a fixed panel in front (the sliding door, inside it, is its own part)
        fbox!(
            f,
            "glass_side",
            0.01,
            2.0,
            0.83,
            (0.435, 1.09, -0.005),
            M.glass
        );
        fbox!(
            f,
            "glass_fixed",
            0.46,
            2.0,
            0.01,
            (-0.195, 1.09, 0.437),
            M.glass
        );
        // the frame: the side's posts and top rail bent in one piece, the front's top rail and its track
        frame(
            &f,
            "frame",
            &[
                (0.435, 0.058, -0.42),
                (0.435, top - 0.0125, -0.42),
                (0.435, top - 0.0125, 0.435),
                (0.435, 0.058, 0.435),
            ],
            M.frame,
            0.025,
            0.012,
            (1, 0, 0).into(),
        );
        fsweep!(
            f,
            "rail_front",
            &[(0.42, top - 0.0125, 0.435), (-0.414, top - 0.0125, 0.435)],
            0.0,
            M.frame,
            profile = rounded_rect(0.025, 0.025, 0.004, 1)
        );
        fbox!(
            f,
            "rail_bottom",
            0.85,
            0.02,
            0.03,
            (0.003, 0.068, 0.435),
            M.frame,
            bevel = 0.004
        );
        fbox!(
            f,
            "door_stop",
            0.02,
            0.05,
            0.02,
            (-0.404, 0.11, 0.42),
            M.frame,
            bevel = 0.005,
            bevel_segments = 1
        );
        // a slide rail with the hand shower and its hose, a bar mixer, and the overhead head on an arm from the wall
        bar_mixer(&f, 0.12, 1.1, -0.414, 0.2, M.fittings);
        hand_shower(
            &f,
            -0.15,
            0.9,
            1.7,
            -0.384,
            M.fittings,
            (0.17, 1.065, -0.37),
        );
        wall_rose(
            &f, "arm_rose", 0, 2.03, -0.414, 0.025, M.fittings, 0.012, 14,
        );
        kit::tube!(
            f,
            "arm",
            &[
                (0, 2.03, -0.41),
                (0, 2.03, -0.2),
                (0, 2.018, -0.16),
                (0, 2.012, -0.15),
            ],
            0.011,
            M.fittings,
            verts = 10
        );
        head(&f, "head", 0, 1.989, -0.15, 0.12, M.fittings, 0.022);
        // a corner shelf with two bottles
        let mut shelf = vec![(0.0, 0.0)];
        shelf.extend(
            (0..=90)
                .step_by(15)
                .map(|a| (0.2 * cos(radians(a as f64)), 0.2 * sin(radians(a as f64)))),
        );
        fextrude!(
            f,
            "corner_shelf",
            &shelf,
            0.01,
            (-0.414, 1.25, -0.414),
            M.frame,
            plane = Facing::Top,
            bevel = 0.003,
            bevel_segments = 1
        );
        bottle(
            &f,
            "bottle0",
            (-0.36, 1.255, -0.36),
            0.026,
            0.16,
            M.teal,
            M.bottle,
            12,
            false,
        );
        bottle(
            &f,
            "bottle1",
            (-0.29, 1.255, -0.375),
            0.022,
            0.12,
            M.bottle,
            M.sage,
            12,
            true,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the sliding door, inside the fixed panel on the right: it slides left behind it to let the resident in and
    // out (`door_open`). All glass, its grip a bar of smoked glass on the inside by its leading edge, so the door
    // is one draw call (Q-05: the shower's fourth).
    let parts = new_parts(|| {
        fbox!(
            f,
            "glass_door",
            0.44,
            1.98,
            0.01,
            (0.205, 1.08, 0.423),
            M.glass
        );
        fsweep!(
            f,
            "door_grip",
            &bends(
                &[
                    (0.37, 0.85, 0.418),
                    (0.37, 0.85, 0.398),
                    (0.37, 1.25, 0.398),
                    (0.37, 1.25, 0.418)
                ],
                0.01,
                4
            ),
            0.009,
            M.grip,
            verts = 8
        );
    });
    let door = join(&parts, "door", Some(f.at(0.205, 1.08, 0.423)));
    let rain = rain(&f, (0.0, 1.988, -0.15), 0.095, 0.06, 2, None, None);
    settle_all();
    slide!(door, "door_open", (SHOWER_SLIDE, 0, 0), seconds = 0.7);
    grow!(rain, "water_on", seconds = 0.5);
    meshes()
}

pub fn shower_stall() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let rail = 1.95;
    let still = new_parts(|| {
        tray(&f, "tray", 0.9, 0.9, 0.08, M.tray, 0.045, 0.012, 14);
        grate(
            &f, "drain", 0.05, 0.069, 0.05, 0.09, 0.09, M.fittings, 5, true,
        );
        kit::tiled_walls!(
            &f,
            (-0.45, 0.45),
            (-0.45, 0.45),
            (0.08, 2.08),
            0.04,
            0.25,
            0.005,
            M.grout,
            M.tile
        );
        // an L-shaped rail at the open sides, bent round the corner; the curtain hangs along the whole side, and its
        // front is its own part
        kit::tube!(
            f,
            "rail",
            &bends(
                &[
                    (0.42, rail, -0.398),
                    (0.42, rail, 0.42),
                    (-0.398, rail, 0.42),
                ],
                0.04,
                5,
            ),
            0.011,
            M.frame,
            verts = 10,
            round = 1
        );
        wall_rose(
            &f,
            "rail_mount_side",
            0.42,
            rail,
            -0.404,
            0.025,
            M.frame,
            0.012,
            14,
        );
        fcyl!(
            f,
            "rail_mount_front",
            0.025,
            0.012,
            (-0.398, rail, 0.42),
            M.frame,
            rot = (0, 90, 0),
            verts = 14,
            bevel = 0.004,
            bevel_segments = 1
        );
        curtain(
            &f,
            "curtain_side",
            (0.42, -0.39),
            (0.42, 0.39),
            0.22,
            rail - 0.03,
            6,
            0.035,
            0.004,
            6,
            1.0,
        );
        curtain_rings(
            &f,
            "ring_side",
            (0.42, -0.39),
            (0.42, 0.39),
            rail,
            12,
            M.frame,
        );
        // a riser on the back wall up to a tilted head, a lever mixer, a wire caddy with bottles
        // (bent round 3.5 cm, wider than the tube: bent tighter, its inside folded through itself)
        kit::tube!(
            f,
            "riser",
            &bends(
                &[
                    (-0.2, 1.08, -0.39),
                    (-0.2, 1.96, -0.39),
                    (-0.2, 1.96, -0.33),
                ],
                0.035,
                5,
            ),
            0.013,
            M.fittings,
            verts = 10,
            round = 1
        );
        fcyl!(
            f,
            "head",
            0.07,
            0.03,
            (-0.2, 1.96, -0.31),
            M.fittings,
            rot = (-25, 0, 0),
            verts = 20,
            bevel = 0.008,
            bevel_segments = 2
        );
        wall_rose(&f, "mixer", -0.2, 1.05, -0.404, 0.05, M.fittings, 0.018, 20);
        kit::tube!(
            f,
            "lever",
            &[
                (-0.2, 1.05, -0.38),
                (-0.2, 1.06, -0.35),
                (-0.16, 1.07, -0.345),
                (-0.12, 1.075, -0.345),
            ],
            0.008,
            M.fittings,
            verts = 8,
            round = 2,
            scale = [1.3, 1.1, 1.0, 0.8]
        );
        caddy(&f, "caddy", 0.15, 1.15, -0.355, 0.24, 0.1, M.frame);
        bottle(
            &f,
            "bottle0",
            (0.1, 1.154, -0.36),
            0.025,
            0.15,
            M.teal,
            M.bottle,
            12,
            false,
        );
        bottle(
            &f,
            "bottle1",
            (0.18, 1.154, -0.36),
            0.03,
            0.12,
            M.bottle,
            M.teal,
            12,
            true,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the front of the curtain, built drawn across the front and gathered into the right corner at rest (scaled
    // along the rail about the corner): drawn across once the resident is in (`curtain_draw`, a grow along the
    // rail), back as the use ends
    let parts = new_parts(|| {
        curtain(
            &f,
            "curtain",
            (0.41, 0.42),
            (-0.41, 0.42),
            0.22,
            rail - 0.03,
            7,
            0.035,
            0.004,
            10, // gathered, its folds turn three times as sharp: more steps round them
            STALL_GATHER,
        );
        curtain_rings(&f, "ring", (0.41, 0.42), (-0.41, 0.42), rail, 14, M.curtain);
    });
    let curtain = join(&parts, "curtain", Some(f.at(0.41, rail, 0.42)));
    curtain.edit(|o| o.scale.x = STALL_GATHER as f32);
    let rain = rain(
        &f,
        (-0.2, 1.935, -0.3),
        0.045,
        0.08,
        1,
        None,
        Some((-0.05, 0.085, -0.02)),
    );
    settle_all();
    grow!(curtain, "curtain_draw", seconds = 0.9, axis = Axis::X);
    grow!(rain, "water_on", seconds = 0.5);
    meshes()
}
