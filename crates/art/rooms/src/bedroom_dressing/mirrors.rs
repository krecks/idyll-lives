//! The cheval, wall and floor mirrors.

use super::common::{F, M, frame_loop, moulding, pane, rounded, side_knob};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{P3, PI, radians, sin};
use artkit::parts::{join_all, settle_all};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{cylinder, fbox, fcyl, fextrude, fsweep, loft, revolve, ring, settle, soft};

// --- the cheval mirror: 1x1, a tall swing frame between two turned posts on feet -----------------------
// The old part's frame leaned back 6°; this one hangs upright, so the reflection lies on the glass.

const CHEVAL_Y: f64 = 0.95; // the frame's middle and its pivots
const CHEVAL_W: f64 = 0.57; // the frame's moulding, along its middle
const CHEVAL_H: f64 = 1.57;
const CHEVAL_RAIL: f64 = 0.05; // the moulding's width
const POST_X: f64 = 0.35;

/// The cheval mirror.
pub fn mirror() -> Vec<Obj> {
    let rail = CHEVAL_RAIL;
    // the swing frame: a moulding with a cove and a bead, the glass (front, +z) set in behind its lip, the board
    // behind
    frame_loop(
        "frame",
        &rounded(CHEVAL_W, CHEVAL_H, 0.03, 4),
        &moulding(rail, 0.045),
        (0, CHEVAL_Y, 0.0),
        M.frame,
        1.0,
    );
    pane(
        "glass",
        CHEVAL_W - rail + 0.01,
        CHEVAL_H - rail + 0.01,
        0.004,
        (0, CHEVAL_Y, 0.009),
        M.mirror,
    );
    fbox!(
        F,
        "backing",
        CHEVAL_W - 0.02,
        CHEVAL_H - 0.02,
        0.01,
        (0, CHEVAL_Y, -0.005),
        M.backing
    );
    let top = CHEVAL_Y + CHEVAL_H / 2.0 + rail / 2.0;
    // a crest over it, rising to the finial in a shallow arch with a scroll at each end
    let mut crest = vec![(0.33, top + 0.0), (0.33, top + 0.03), (0.3, top + 0.04)];
    crest.extend((0..13).map(|i| {
        (
            0.27 - 0.54 * i as f64 / 12.0,
            top + 0.04 + 0.012 * sin(PI * i as f64 / 12.0),
        )
    }));
    crest.extend([(-0.3, top + 0.04), (-0.33, top + 0.03), (-0.33, top + 0.0)]);
    crest.reverse();
    fextrude!(
        F,
        "crest",
        &crest,
        0.05,
        (0, 0, 0),
        M.frame,
        bevel = 0.008,
        bevel_segments = 2
    );
    kit::turned!(
        F,
        "crest_finial",
        0,
        0,
        0,
        &[
            (0.0, top + 0.05),
            (0.016, top + 0.05),
            (0.01, top + 0.058),
            (0.024, top + 0.075),
            (0.012, top + 0.093),
            (0.0, top + 0.096),
        ],
        M.frame,
        segments = 16
    );
    // two turned posts on their feet, a turned stretcher low between them
    for s in [-1, 1] {
        let x = s as f64 * POST_X;
        let mut foot = vec![
            (0.28, 0.0),
            (0.28, 0.02),
            (0.24, 0.045),
            (0.1, 0.052),
            (-0.1, 0.052),
            (-0.24, 0.045),
            (-0.28, 0.02),
            (-0.28, 0.0),
            (-0.2, 0.0),
            (-0.18, 0.012),
        ];
        foot.extend((0..9).map(|i| {
            (
                -0.16 + 0.32 * i as f64 / 8.0,
                0.012 + 0.008 * sin(PI * i as f64 / 8.0),
            )
        }));
        foot.extend([(0.18, 0.012), (0.2, 0.0)]);
        foot.reverse();
        fextrude!(
            F,
            format!("foot{s}"),
            &foot,
            0.052,
            (x, 0, 0),
            M.frame,
            plane = Facing::Side,
            bevel = 0.008,
            bevel_segments = 2
        );
        kit::turned!(
            F,
            format!("post{s}"),
            x,
            0,
            0,
            &[
                (0.0, 0.05),
                (0.026, 0.05),
                (0.028, 0.07),
                (0.026, 0.1),
                (0.018, 0.125),
                (0.022, 0.15),
                (0.015, 0.18),
                (0.015, 0.5),
                (0.019, 0.55),
                (0.016, 0.6),
                (0.02, 0.86),
                (0.027, 0.89),
                (0.027, 0.92),
                (0.022, 0.94),
                (0.027, 0.97),
                (0.027, 0.99),
                (0.012, 1.0),
                (0.0, 1.0),
            ],
            M.frame,
            segments = 16
        );
        kit::turned!(
            F,
            format!("finial{s}"),
            x,
            0,
            0,
            &[
                (0.0, 0.996),
                (0.013, 0.996),
                (0.009, 1.004),
                (0.024, 1.02),
                (0.026, 1.03),
                (0.02, 1.042),
                (0.008, 1.048),
                (0.0, 1.049),
            ],
            M.frame,
            segments = 14
        );
        // the pivot: an axle from the frame into the post, a turned brass knob outside it
        cylinder!(
            format!("axle{s}"),
            0.008,
            0.07,
            at = F.at(s as f64 * 0.335, CHEVAL_Y, 0),
            material = M.brass,
            rot = (0, 90, 0),
            verts = 10
        );
        side_knob(
            &format!("knob{s}"),
            s as f64 * 0.374,
            CHEVAL_Y,
            0,
            0.014,
            s,
            M.brass,
        );
    }
    fsweep!(
        F,
        "stretcher",
        &[
            (-POST_X, 0.09, 0),
            (-0.2, 0.09, 0),
            (-0.06, 0.09, 0),
            (0.0, 0.09, 0),
            (0.06, 0.09, 0),
            (0.2, 0.09, 0),
            (POST_X, 0.09, 0)
        ],
        0.013,
        M.frame,
        verts = 12,
        scale = [1.0, 0.8, 1.2, 0.85, 1.2, 0.8, 1.0]
    );
    join_all("mirror");
    settle_all()
}

// --- the wall mirror: on the wall layer, a black metal frame and a small ledge --------------------------
// Its back on the wall face (depth -0.43), at the old part's height: the frame from 1.0 to 1.9 m.

use artkit::kit::WALL;
const WM_Y: f64 = 1.45;
const WM_W: f64 = 0.62;
const WM_H: f64 = 0.9;
const WM_RIM: f64 = 0.03;
const WM_T: f64 = 0.035;

/// The wall mirror.
pub fn mirror_wall() -> Vec<Obj> {
    let zf = WALL + WM_T / 2.0;
    frame_loop(
        "frame",
        &rounded(WM_W - WM_RIM, WM_H - WM_RIM, 0.012, 3),
        &moulding(WM_RIM, WM_T),
        (0, WM_Y, zf),
        M.wall_frame,
        1.0,
    );
    pane(
        "glass",
        WM_W - 2.0 * WM_RIM + 0.006,
        WM_H - 2.0 * WM_RIM + 0.006,
        0.004,
        (0, WM_Y, WALL + WM_T - 0.008),
        M.mirror,
    );
    fbox!(
        F,
        "backing",
        WM_W - 0.04,
        WM_H - 0.04,
        0.012,
        (0, WM_Y, WALL + 0.008),
        M.wall_frame
    );
    // the ledge under it: a tray with a rolled lip, on two bent brackets; a cup, a toothbrush and the toothpaste
    let shelf_y = 0.972;
    // its section (depth out from the wall's face less 4.25 cm, height): from the floor's front, round the lip,
    // along the tray's top to the back; its corners eased (with height as eased's v: the underside's stay sharp)
    let tray: Vec<(f64, f64)> = artkit::forms::eased(
        &[
            (0.045, 0.0),
            (0.0472, 0.006),
            (0.0468, 0.022),
            (0.0428, 0.026),
            (0.0388, 0.024),
            (0.0398, 0.012),
            (-0.0425, 0.012),
            (-0.0425, 0.0),
        ],
        0.003,
    );
    // lofted along x, its ends chamfered 2 mm (its lip is too thin for more)
    let ring = |o: &[(f64, f64)], x: f64| -> Vec<P3> {
        o.iter()
            .map(|&(v, u)| F.at(x, shelf_y - 0.006 + u, WALL + 0.0425 + v))
            .collect()
    };
    let end = artkit::forms::inset(&tray, 0.002);
    loft!(
        "shelf",
        &[
            ring(&end, -0.26),
            ring(&tray, -0.258),
            ring(&tray, 0.258),
            ring(&end, 0.26)
        ],
        material = M.wall_frame,
        angle = 35
    );
    for s in [-1, 1] {
        let sx = s as f64;
        fsweep!(
            F,
            format!("bracket{s}"),
            &kit::fillet(
                &[
                    (sx * 0.2, shelf_y - 0.036, WALL + 0.002),
                    (sx * 0.2, shelf_y - 0.012, WALL + 0.002),
                    (sx * 0.2, shelf_y - 0.012, WALL + 0.06)
                ],
                0.016,
                3
            ),
            0,
            M.wall_frame,
            profile = rounded_rect(0.004, 0.012, 0.0015, 1),
            up = (0, 1, 0)
        );
    }
    let top = shelf_y + 0.006;
    revolve!(
        "cup",
        &[
            (0.0, 0.0),
            (0.019, 0.0),
            (0.021, 0.004),
            (0.024, 0.082),
            (0.0245, 0.085),
            (0.0215, 0.085),
            (0.018, 0.012),
            (0.0, 0.012)
        ],
        at = F.at(-0.16, top, -0.368),
        material = M.cup,
        segments = 16
    );
    fsweep!(
        F,
        "toothbrush",
        &[
            (-0.165, top + 0.015, -0.368),
            (-0.16, top + 0.07, -0.369),
            (-0.152, top + 0.11, -0.371),
            (-0.149, top + 0.135, -0.372)
        ],
        0.0045,
        M.brush,
        verts = 6,
        resolution = 2,
        scale = [1.0, 0.8, 0.75, 0.9]
    );
    fbox!(
        F,
        "toothbrush_head",
        0.01,
        0.028,
        0.008,
        (-0.148, top + 0.122, -0.3665),
        M.brush,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        "toothbrush_bristles",
        0.008,
        0.024,
        0.01,
        (-0.148, top + 0.122, -0.3585),
        M.bristle,
        bevel = 0.002,
        bevel_segments = 1
    );
    let b = F.at(0.12, 0, -0.385);
    let (bx, by) = (b.x, b.y);
    let secs: Vec<Vec<P3>> = [
        (0.004, 0.002, 0.016),
        (0.01, 0.01, 0.016),
        (0.11, 0.016, 0.015),
        (0.125, 0.012, 0.012),
    ]
    .iter()
    .map(|&(yy, rx, ry)| ring!(rx, ry, z = top + yy, verts = 12, at = (bx + 0.0, by)))
    .collect();
    loft!("paste", &secs, material = M.paste, rot = (0, 0, 0));
    revolve!(
        "paste_cap",
        &[(0.0, 0.0), (0.011, 0.0), (0.011, 0.018), (0.0, 0.018)],
        at = F.at(0.12, top + 0.125, -0.385),
        material = M.paste_cap,
        segments = 10
    );
    join_all("mirrorWall");
    settle!(centre = false, floor = false)
}

// --- the standing mirror: 1x1, a full-length mirror upright on a brass stand ----------------------------
// Where the game places it: the frame a little behind the middle, the stand's strut behind it. The
// old part leaned back 8°; this one stands upright on two sled feet, held by a strut from behind.

const SM_Z: f64 = -0.05; // the frame's middle, in depth
const SM_W: f64 = 0.56;
const SM_H: f64 = 1.66;
const SM_RIM: f64 = 0.028;
const SM_T: f64 = 0.035;
const SM_Y0: f64 = 0.12; // the frame's bottom
const FOOT_X: f64 = 0.22;
const FOOT_Z0: f64 = -0.38;
const FOOT_Z1: f64 = 0.07;

/// The standing mirror.
pub fn mirror_floor() -> Vec<Obj> {
    let y = SM_Y0 + SM_H / 2.0;
    frame_loop(
        "frame",
        &rounded(SM_W - SM_RIM, SM_H - SM_RIM, 0.03, 4),
        &moulding(SM_RIM, SM_T),
        (0, y, SM_Z),
        M.stand,
        1.0,
    );
    pane(
        "glass",
        SM_W - 2.0 * SM_RIM + 0.006,
        SM_H - 2.0 * SM_RIM + 0.006,
        0.004,
        (0, y, SM_Z + SM_T / 2.0 - 0.0105),
        M.mirror,
    );
    fbox!(
        F,
        "backing",
        SM_W - 0.03,
        SM_H - 0.03,
        0.012,
        (0, y, SM_Z - 0.008),
        M.stand_back
    );
    // the stand: two sled feet turned up at the toe, on glides; a clamp up from each to the frame, tightened by a
    // knob; a rear bar, and a strut folding at a hinge up to a plate on the frame's back
    for s in [-1, 1] {
        let x = s as f64 * FOOT_X;
        fsweep!(
            F,
            format!("foot{s}"),
            &kit::fillet(
                &[
                    (x, 0.012, FOOT_Z0),
                    (x, 0.012, FOOT_Z1 - 0.03),
                    (x, 0.018, FOOT_Z1)
                ],
                0.03,
                3
            ),
            0,
            M.stand,
            profile = rounded_rect(0.016, 0.024, 0.006, 2),
            up = (0, 0, 1)
        );
        for z in [FOOT_Z0 + 0.03, FOOT_Z1 - 0.05] {
            fcyl!(
                F,
                format!("glide{s}{}", float_repr(z)),
                0.009,
                0.006,
                (x, 0.003, z),
                M.stand_back,
                verts = 10
            );
        }
        soft!(
            F,
            format!("clamp{s}"),
            0.026,
            SM_Y0 + 0.04 - 0.02,
            SM_T + 0.012,
            (x, (0.02 + SM_Y0 + 0.04) / 2.0, SM_Z),
            M.stand,
            round = 0.006,
            round_segments = 1
        );
        let k = kit::knob!(
            F,
            format!("clamp_knob{s}"),
            x,
            SM_Y0 + 0.01,
            SM_Z - SM_T / 2.0 - 0.006,
            0.01,
            M.stand
        );
        k.set_rotation_euler((radians(-90.0), 0.0, 0.0)); // out to the back
    }
    fsweep!(
        F,
        "rear_bar",
        &[
            (-FOOT_X, 0.012, FOOT_Z0 + 0.02),
            (FOOT_X, 0.012, FOOT_Z0 + 0.02)
        ],
        0.012,
        M.stand,
        verts = 12
    );
    let back = SM_Z - SM_T / 2.0;
    let mid = (0, 0.71, (FOOT_Z0 + 0.02 + back - 0.006) / 2.0);
    kit::tube!(
        F,
        "strut_low",
        &[(0, 0.02, FOOT_Z0 + 0.02), mid],
        0.011,
        M.stand,
        verts = 12,
        round = 1
    );
    kit::tube!(
        F,
        "strut_high",
        &[mid, (0, 1.40, back - 0.006)],
        0.01,
        M.stand,
        verts = 12,
        round = 1
    );
    cylinder!(
        "strut_hinge",
        0.014,
        0.034,
        at = F.at(mid.0, mid.1, mid.2),
        material = M.stand,
        rot = (0, 90, 0),
        verts = 12
    );
    soft!(
        F,
        "strut_plate",
        0.07,
        0.06,
        0.008,
        (0, 1.40, back - 0.004),
        M.stand,
        round = 0.003,
        round_segments = 1
    );
    join_all("mirrorFloor");
    settle!(centre = false)
}
