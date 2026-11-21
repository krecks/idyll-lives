//! The sauna cabin for three: spruce boards, a glass front with a glass door, two benches, a stove with stones, a
//! bucket and ladle.

use artkit::math::P3;
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, numbered};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{boulder, fbox, fcyl, kit, palette, settle, soft, swing, torus};

palette! { S:
    body = ("#E3C796", satin),   // the spruce: walls, roof, benches, floor; the main part the player colours
    trim = ("#B98955", satin),   // the darker posts, plinth, fascia, frames, the door's handle
    glass = ("#DCE6E8", glass),  // the front and the door
    heater = ("#8A8F95", metal), // the stove's steel, the door's hinges
    stones = ("#8C8A85", matte),
    copper = ("#B87333", metal), // the bucket's bands, the thermometer's rim
    cream = ("#F2ECDF", satin),  // a towel, the thermometer's face
    needle = ("#C8402F", satin),
}

pub fn palette() -> artkit::materials::Palette {
    S.palette()
}

/// The cabin's back, on the wall's face (2 deep), and its front face; its half width.
pub const BACK: f64 = -0.93;
const FRONT: f64 = 0.95;
const HALF: f64 = 0.95;
/// The walls' thickness, and the height of the boards' top (the roof above).
const WALL: f64 = 0.045;
const TOP: f64 = 2.0;
/// The lower bench's top: the three places sit on it (furniture.json `seats.sauna`), their middles 0.35 m back.
pub const SEAT: f64 = 0.45;
const SEAT_Z: f64 = -0.35;
/// The upper bench's top.
const UPPER: f64 = 0.9;
/// The glass front's plane (depth), and the door's hinge and free edges (x).
const GLASS_Z: f64 = 0.925;
const DOOR_L: f64 = -0.89;
const DOOR_R: f64 = -0.24;
/// The side walls' boards under their glass: six, up to 0.83 m, under the bench line.
const SIDE_BOARDS: usize = 6;
/// The bar across the glazed roof (depth), over the lower bench's front edge.
const ROOF_BAR: f64 = -0.12;

pub fn sauna() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let inner = HALF - WALL; // the side walls' inner faces
    let back_in = BACK + WALL; // the back wall's inner face
    let still = new_parts(|| {
        // The plinth, and the floor of spruce boards along the depth.
        fbox!(
            f,
            "plinth",
            2.0 * HALF,
            0.05,
            FRONT - BACK,
            (0, 0.025, (BACK + FRONT) / 2.0),
            S.trim,
            bevel = 0.006,
            bevel_segments = 1
        );
        let floor_w = 2.0 * inner / 9.0;
        for k in 0..9 {
            fbox!(
                f,
                numbered("floor_board"),
                floor_w - 0.008,
                0.012,
                GLASS_Z - back_in - 0.02,
                (
                    -inner + floor_w * (k as f64 + 0.5),
                    0.056,
                    (back_in + GLASS_Z) / 2.0 - 0.01
                ),
                S.body,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // The walls: horizontal boards, each a hair proud or shy of the next, so the seams show inside and out; the
        // side walls' boards stop under the bench line (`SIDE_BOARDS`), glass above them.
        let (y0, n) = (0.05, 15);
        let bh = (TOP - y0) / n as f64;
        for k in 0..n {
            let y = y0 + bh * (k as f64 + 0.5);
            let t = WALL - 0.004 * (k % 2) as f64;
            for s in [-1.0, 1.0].into_iter().filter(|_| k < SIDE_BOARDS) {
                fbox!(
                    f,
                    numbered("side_board"),
                    t,
                    bh - 0.002,
                    GLASS_Z - BACK - 0.04,
                    (s * (HALF - WALL / 2.0), y, (BACK + GLASS_Z) / 2.0 - 0.02),
                    S.body,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
            fbox!(
                f,
                numbered("back_board"),
                2.0 * inner,
                bh - 0.002,
                t,
                (0, y, BACK + WALL / 2.0),
                S.body,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // Each side wall glazed above its boards: a pane in a spruce frame (a sill rail on the boards, a head rail
        // under the roof, an upright in the middle), so a bather on an end seat, 0.4 m in from the wall, shows from a
        // camera on that side (behind 2 m of boards they were hidden).
        let sill = y0 + bh * SIDE_BOARDS as f64;
        let (gz0, gz1) = (BACK + 0.07, FRONT - 0.07);
        for s in [-1.0, 1.0] {
            let x = s * (HALF - WALL / 2.0);
            fbox!(
                f,
                numbered("side_pane"),
                0.008,
                TOP - sill - 0.04,
                gz1 - gz0 + 0.02,
                (x, (sill + TOP) / 2.0, (gz0 + gz1) / 2.0),
                S.glass
            );
            for (y, h) in [(sill + 0.025, 0.05), (TOP - 0.03, 0.06)] {
                fbox!(
                    f,
                    numbered("side_rail"),
                    WALL + 0.01,
                    h,
                    gz1 - gz0,
                    (x, y, (gz0 + gz1) / 2.0),
                    S.trim,
                    bevel = 0.008,
                    bevel_segments = 1
                );
            }
            fbox!(
                f,
                numbered("side_mullion"),
                WALL + 0.01,
                TOP - sill - 0.11,
                0.05,
                (x, (sill + 0.05 + TOP - 0.06) / 2.0, (gz0 + gz1) / 2.0),
                S.trim,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        // The front's frame: corner posts, a lintel, a sill, a post between the door and the fixed pane.
        for s in [-1.0, 1.0] {
            fbox!(
                f,
                numbered("corner_post"),
                0.075,
                TOP - 0.05,
                0.07,
                (
                    s * (HALF - 0.0325),
                    0.05 + (TOP - 0.05) / 2.0,
                    FRONT - 0.035
                ),
                S.trim,
                bevel = 0.008,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("back_post"),
                0.075,
                TOP - 0.05,
                0.07,
                (s * (HALF - 0.0325), 0.05 + (TOP - 0.05) / 2.0, BACK + 0.035),
                S.trim,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "lintel",
            2.0 * HALF - 0.14,
            0.08,
            0.06,
            (0, TOP - 0.04, FRONT - 0.035),
            S.trim,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "sill",
            2.0 * HALF - 0.14,
            0.05,
            0.06,
            (0, 0.075, FRONT - 0.035),
            S.trim,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "mullion",
            0.06,
            TOP - 0.17,
            0.06,
            (
                DOOR_R + 0.04,
                0.1 + (TOP - 0.17) / 2.0 - 0.005,
                FRONT - 0.035
            ),
            S.trim,
            bevel = 0.008,
            bevel_segments = 1
        );
        // the fixed pane, beside the door, and the stop the door closes on
        let (px0, px1) = (DOOR_R + 0.07, HALF - 0.07);
        fbox!(
            f,
            "pane",
            px1 - px0 + 0.02,
            TOP - 0.18,
            0.008,
            ((px0 + px1) / 2.0, 0.1 + (TOP - 0.18) / 2.0, GLASS_Z),
            S.glass
        );
        // the door's hinges on the left post
        for y in [0.35, 1.65] {
            fbox!(
                f,
                numbered("hinge"),
                0.03,
                0.08,
                0.02,
                (DOOR_L - 0.01, y, GLASS_Z + 0.01),
                S.heater,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // The roof: glazed whole, one pane in a slim spruce frame (round its edge, a bar over the post between the
        // door and the pane, one across over the bench's front), a fascia round it all. Every seat on the back bench
        // is seen from above through it, whichever side a bather sits on (under a deck of boards over the back, those
        // on the side away from the camera were hidden).
        fbox!(
            f,
            "roof_glass",
            2.0 * HALF - 0.06,
            0.008,
            FRONT - BACK - 0.06,
            (0, TOP + 0.03, (BACK + FRONT) / 2.0),
            S.glass
        );
        for (name, z) in [
            ("roof_rail", BACK + 0.025),
            ("roof_rail", ROOF_BAR),
            ("roof_rail", FRONT - 0.025),
        ] {
            fbox!(
                f,
                numbered(name),
                2.0 * HALF + 0.02,
                0.05,
                0.05,
                (0, TOP + 0.025, z),
                S.trim,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        for x in [-HALF + 0.025, DOOR_R + 0.04, HALF - 0.025] {
            fbox!(
                f,
                numbered("roof_bar"),
                0.05,
                0.05,
                FRONT - BACK - 0.1,
                (x, TOP + 0.025, (BACK + FRONT) / 2.0),
                S.trim,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "fascia",
            2.0 * HALF + 0.04,
            0.07,
            0.02,
            (0, TOP + 0.015, FRONT + 0.015),
            S.trim,
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            fbox!(
                f,
                numbered("fascia_side"),
                0.02,
                0.07,
                FRONT - BACK + 0.005,
                (s * (HALF + 0.01), TOP + 0.015, (BACK + FRONT + 0.005) / 2.0),
                S.trim,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // The lower bench (top 0.45) where the three sit, its slats along the cabin over a skirt of upright boards;
        // the upper bench behind and above it, its own skirt the lower bench's backrest.
        let slat = |name: &str, z: f64, top: f64| {
            fbox!(
                f,
                numbered(name),
                2.0 * inner - 0.004,
                0.034,
                0.09,
                (0, top - 0.017, z),
                S.body,
                bevel = 0.01,
                bevel_segments = 2
            );
        };
        for k in 0..4 {
            slat("bench_slat", SEAT_Z - 0.165 + 0.11 * k as f64, SEAT);
        }
        for k in 0..3 {
            slat("upper_slat", back_in + 0.05 + 0.11 * k as f64, UPPER);
        }
        let skirt = |name: &str, z: f64, y0: f64, y1: f64| {
            let n = 14;
            let w = 2.0 * inner / n as f64;
            for k in 0..n {
                fbox!(
                    f,
                    numbered(name),
                    w - 0.006,
                    y1 - y0,
                    0.022,
                    (-inner + w * (k as f64 + 0.5), (y0 + y1) / 2.0, z),
                    S.body,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
        };
        skirt("bench_skirt", SEAT_Z + 0.2, 0.062, SEAT - 0.034);
        skirt("upper_skirt", SEAT_Z - 0.22, SEAT - 0.034, UPPER - 0.034);
        // backrest slats on the back wall over the upper bench
        for y in [1.17, 1.33] {
            fbox!(
                f,
                numbered("back_slat"),
                2.0 * inner - 0.1,
                0.07,
                0.025,
                (0, y, back_in + 0.0125),
                S.body,
                bevel = 0.01,
                bevel_segments = 2
            );
        }
        // The stove in the front right corner: a steel case with vents and a basket of stones, behind a guard of
        // spruce slats on its two open sides.
        let (hx, hz) = (0.74, 0.6);
        fbox!(
            f,
            "heater",
            0.28,
            0.56,
            0.3,
            (hx, 0.062 + 0.28, hz),
            S.heater,
            bevel = 0.015,
            bevel_segments = 2
        );
        fbox!(
            f,
            "heater_basket",
            0.3,
            0.09,
            0.32,
            (hx, 0.062 + 0.56 + 0.04, hz),
            S.heater,
            bevel = 0.008,
            bevel_segments = 1
        );
        for k in 0..4 {
            fbox!(
                f,
                numbered("heater_vent"),
                0.004,
                0.012,
                0.2,
                (hx - 0.141, 0.2 + 0.05 * k as f64, hz),
                S.stones
            );
        }
        for (k, (dx, dz)) in [
            (-0.09, -0.09),
            (0.0, -0.1),
            (0.09, -0.08),
            (-0.08, 0.0),
            (0.01, 0.01),
            (0.09, 0.02),
            (-0.09, 0.1),
            (0.0, 0.09),
            (0.08, 0.1),
            (-0.04, -0.04),
            (0.05, -0.03),
            (-0.03, 0.05),
            (0.05, 0.06),
        ]
        .into_iter()
        .enumerate()
        {
            boulder!(
                numbered("stone"),
                0.05,
                at = f.at(hx + dx, 0.66 + 0.015 * (k / 9) as f64, hz + dz),
                material = S.stones,
                seed = k as f64 * 1.7,
                subdivisions = 1,
                scale = (1, 1, 0.75)
            );
        }
        // the guard: upright slats on two rails, along the stove's left side and across its back
        let gx = hx - 0.2;
        let gz = hz - 0.21;
        for k in 0..4 {
            fbox!(
                f,
                numbered("guard"),
                0.025,
                0.66,
                0.025,
                (gx, 0.062 + 0.33, gz + 0.04 + 0.13 * k as f64),
                S.body,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        for k in 0..3 {
            fbox!(
                f,
                numbered("guard"),
                0.025,
                0.66,
                0.025,
                (gx + 0.09 + 0.1 * k as f64, 0.062 + 0.33, gz),
                S.body,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        for y in [0.2, 0.6] {
            fbox!(
                f,
                numbered("guard_rail"),
                inner - gx + 0.0125,
                0.04,
                0.02,
                ((gx + inner) / 2.0 - 0.006, y, gz - 0.0225),
                S.body,
                bevel = 0.005,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("guard_rail"),
                0.02,
                0.04,
                0.5,
                (gx - 0.0225, y, gz + 0.24),
                S.body,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        // the bucket on the floor by the stove, banded in copper, its ladle in it
        let (bx, bz) = (0.32, 0.6);
        kit::turned!(
            f,
            "bucket",
            bx,
            0.062,
            bz,
            &[
                (0.0, 0.0),
                (0.09, 0.0),
                (0.11, 0.18),
                (0.1, 0.18),
                (0.082, 0.015),
                (0.0, 0.015)
            ],
            S.trim,
            segments = 18
        );
        for y in [0.03, 0.14] {
            torus!(
                numbered("bucket_band"),
                0.0905 + y * 0.11,
                0.005,
                at = f.at(bx, 0.062 + y, bz),
                material = S.copper,
                major_segments = 18,
                minor_segments = 3
            );
        }
        kit::tube!(
            f,
            "ladle",
            &[
                (bx - 0.02, 0.14, bz - 0.01),
                (bx + 0.03, 0.3, bz - 0.04),
                (bx + 0.08, 0.46, bz - 0.07)
            ],
            0.009,
            S.trim,
            verts = 6,
            round = 1
        );
        // a thermometer on the back wall between the backrests' ends
        fcyl!(
            f,
            "thermometer",
            0.07,
            0.02,
            (-0.55, 1.6, back_in + 0.01),
            S.copper,
            rot = (90, 0, 0),
            verts = 20,
            bevel = 0.006,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "thermometer_face",
            0.058,
            0.004,
            (-0.55, 1.6, back_in + 0.022),
            S.cream,
            rot = (90, 0, 0),
            verts = 20
        );
        fbox!(
            f,
            "thermometer_needle",
            0.005,
            0.045,
            0.002,
            (-0.54, 1.612, back_in + 0.025),
            S.needle,
            rot = (0, -30, 0)
        );
        // a towel folded on the upper bench at the left end
        soft!(
            f,
            "towel",
            0.4,
            0.04,
            0.28,
            (-0.66, UPPER + 0.02, back_in + 0.17),
            S.cream,
            round = 0.012,
            puff = 0.004,
            resting = true
        );
    });
    join(&still, "cabin", Some(P3::ZERO));
    // The glass door, hinged on its left, a spruce handle outside on two stand-offs: `door_open` swings it out.
    let door = new_parts(|| {
        fbox!(
            f,
            "door_glass",
            DOOR_R - DOOR_L,
            TOP - 0.2,
            0.01,
            (
                (DOOR_L + DOOR_R) / 2.0,
                0.1 + (TOP - 0.2) / 2.0 + 0.005,
                GLASS_Z + 0.005
            ),
            S.glass
        );
        let px = DOOR_R - 0.07;
        kit::tube!(
            f,
            "handle",
            &kit::fillet!(
                &[
                    (px, 0.8, GLASS_Z + 0.01),
                    (px, 0.8, GLASS_Z + 0.055),
                    (px, 1.3, GLASS_Z + 0.055),
                    (px, 1.3, GLASS_Z + 0.01)
                ],
                0.03
            ),
            0.015,
            S.trim,
            verts = 8,
            round = 1
        );
        for y in [0.8, 1.3] {
            fcyl!(
                f,
                numbered("handle_rose"),
                0.022,
                0.006,
                (px, y, GLASS_Z + 0.013),
                S.trim,
                rot = (90, 0, 0),
                verts = 10
            );
        }
    });
    let door = hinged(join(&door, "door", None), f.at(DOOR_L, 1.0, GLASS_Z));
    let objs = settle!(centre = false);
    swing!(door, "door_open", 60.0, seconds = 0.8, ease = Ease::BackOut);
    objs
}
