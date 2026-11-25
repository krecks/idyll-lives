//! The stacked laundry centre and the rainfall steam shower.

use super::common::{_bottle, _towels, M, TOWELS, WALL};
use crate::pack_slob::common::_box;
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::P3;
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, settle};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, grow, loft, revolve, ring, rod, settle, soft, swing};

// --- the stacked laundry centre (2x1): washer and dryer at the left, a folding counter with cubbies, a rail ---
const WX: f64 = -0.56; // the stack's middle
const WW: f64 = 0.68; // its width
const WH: f64 = 1.72; // height
const WD: f64 = 0.58; // and depth
const CX: f64 = 0.45; // the counter's middle

/// A porthole door: a chrome ring turned with a lip, a domed window, the handle recess on its right.
fn _porthole(f: &GameFrame, name: &str, cy: f64, zf: f64) {
    revolve!(
        name,
        &[
            (0.17, 0.0),
            (0.235, 0.0),
            (0.24, 0.012),
            (0.232, 0.026),
            (0.2, 0.03),
            (0.18, 0.024),
            (0.17, 0.012)
        ],
        at = f.at(WX, cy, zf),
        material = M.chrome,
        segments = 28,
        rot = (90, 0, 0)
    );
    revolve!(
        format!("{name}_window"),
        &[
            (0.0, 0.022),
            (0.175, 0.008),
            (0.175, 0.014),
            (0.1, 0.026),
            (0.0, 0.03)
        ],
        at = f.at(WX, cy, zf),
        material = M.porthole,
        segments = 24,
        rot = (90, 0, 0)
    );
    soft!(
        f,
        format!("{name}_handle"),
        0.04,
        0.11,
        0.04,
        (WX + 0.25, cy, zf + 0.025),
        M.chrome,
        round = 0.014,
        round_segments = 1
    );
}

/// One of the stack: a rounded cabinet, a control strip with its detergent drawer, display and dial, the drum behind
/// the porthole, and the porthole door unless `door` (the washer's: its moving part).
fn _appliance(f: &GameFrame, name: &str, y0: f64, hh: f64, zf: f64, door: bool) -> f64 {
    _box(
        f,
        &format!("{name}_cabinet"),
        WW,
        hh - 0.006,
        WD - 0.02,
        (WX, y0 + hh / 2.0, WALL + (WD - 0.02) / 2.0),
        M.appliance,
        10.0,
        24,
        0.006,
    );
    soft!(
        f,
        format!("{name}_strip"),
        WW - 0.03,
        0.1,
        0.012,
        (WX, y0 + hh - 0.075, zf - 0.004),
        M.trim,
        round = 0.005,
        round_segments = 1
    );
    soft!(
        f,
        format!("{name}_drawer"),
        0.2,
        0.07,
        0.012,
        (WX - 0.2, y0 + hh - 0.075, zf + 0.004),
        M.appliance,
        round = 0.005,
        round_segments = 1
    );
    fbox!(
        f,
        format!("{name}_drawer_grip"),
        0.08,
        0.012,
        0.006,
        (WX - 0.2, y0 + hh - 0.1, zf + 0.01),
        M.dark
    );
    soft!(
        f,
        format!("{name}_display"),
        0.16,
        0.05,
        0.006,
        (WX + 0.1, y0 + hh - 0.075, zf + 0.004),
        M.screen,
        round = 0.003,
        round_segments = 1
    );
    fbox!(
        f,
        format!("{name}_display_glow"),
        0.07,
        0.02,
        0.002,
        (WX + 0.08, y0 + hh - 0.075, zf + 0.008),
        M.screen_glow
    );
    revolve!(
        format!("{name}_dial"),
        &[
            (0.0, 0.0),
            (0.034, 0.0),
            (0.034, 0.012),
            (0.03, 0.02),
            (0.0, 0.022)
        ],
        at = f.at(WX + 0.25, y0 + hh - 0.075, zf),
        material = M.chrome,
        segments = 16,
        rot = (90, 0, 0)
    );
    let cy = y0 + (hh - 0.13) / 2.0 + 0.03;
    revolve!(
        format!("{name}_drum"),
        &[
            (0.0, -0.06),
            (0.17, -0.06),
            (0.19, -0.02),
            (0.2, 0.0),
            (0.21, 0.004),
            (0.21, 0.0)
        ],
        at = f.at(WX, cy, zf - 0.004),
        material = M.drum,
        segments = 24,
        rot = (90, 0, 0)
    );
    if !door {
        _porthole(f, &format!("{name}_porthole"), cy, zf);
    }
    cy
}

/// A woven basket: a rounded body in ridges, a dark rim and two hand holes' grips.
#[allow(clippy::too_many_arguments)]
fn _basket(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    d: f64,
    material: Mat,
) {
    let b = f.at(x, y, z);
    let mut rings = Vec::new();
    for k in 0..7 {
        let t = k as f64 / 6.0;
        let e = if k % 2 != 0 { 0.006 } else { 0.0 };
        rings.push(ring!(
            w / 2.0 * (0.94 + 0.06 * t) + e,
            d / 2.0 * (0.94 + 0.06 * t) + e,
            z = b.z + h * t,
            verts = 20,
            square = 5,
            at = (b.x, b.y)
        ));
    }
    loft!(name, &rings, material = material);
    loft!(
        format!("{name}_rim"),
        &[
            ring!(
                w / 2.0 + 0.008,
                d / 2.0 + 0.008,
                z = b.z + h - 0.004,
                verts = 20,
                square = 5,
                at = (b.x, b.y)
            ),
            ring!(
                w / 2.0 + 0.008,
                d / 2.0 + 0.008,
                z = b.z + h + 0.012,
                verts = 20,
                square = 5,
                at = (b.x, b.y)
            )
        ],
        material = M.dark
    );
}

/// A shirt on a wooden hanger, seen edge on from the room's side: a hanger hooked over the rail, the shirt's outline
/// (shoulders, sleeves, hem) a thin slab across the depth.
fn _shirt(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, material: Mat) {
    fsweep!(
        f,
        format!("{name}_hook"),
        &[
            (x, y - 0.03, z),
            (x, y + 0.012, z),
            (x, y + 0.018, z + 0.012),
            (x, y + 0.008, z + 0.02)
        ],
        0.0025,
        M.chrome,
        verts = 4
    );
    fsweep!(
        f,
        format!("{name}_hanger"),
        &[
            (x, y - 0.075, z - 0.125),
            (x, y - 0.03, z),
            (x, y - 0.075, z + 0.125)
        ],
        0.0,
        M.oak,
        profile = rounded_rect(0.012, 0.02, 0.004, 1),
        up = (1, 0, 0),
        resolution = 2
    );
    let outline = smooth_outline(
        &[
            (-0.04, -0.03),
            (-0.11, -0.05),
            (-0.14, -0.18),
            (-0.11, -0.19),
            (-0.09, -0.12),
            (-0.095, -0.44),
            (0.095, -0.44),
            (0.09, -0.12),
            (0.11, -0.19),
            (0.14, -0.18),
            (0.11, -0.05),
            (0.04, -0.03),
            (0.0, -0.06),
        ],
        1,
        true,
    );
    let slab: Vec<(f64, f64)> = outline
        .iter()
        .map(|&(u, v)| (z + u, y - 0.07 + v))
        .collect();
    fextrude!(
        f,
        name,
        &slab,
        0.022,
        (x, 0, 0),
        material,
        plane = Facing::Side
    );
}

pub fn laundry_center() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let zf = WALL + WD; // the stack's front
    let hh = WH / 2.0;
    let mut cy = 0.0;
    let still = new_parts(|| {
        cy = _appliance(f, "washer", 0.0, hh, zf, true); // the washer, its door the moving part
        fbox!(
            f,
            "kick",
            WW - 0.03,
            0.05,
            0.012,
            (WX, 0.03, zf + 0.002),
            M.trim,
            bevel = 0.004,
            bevel_segments = 1
        );
        _appliance(f, "dryer", hh, hh, zf, false); // the dryer
        // a slim filler between the stack and the counter, a towel over its bar
        fbox!(
            f,
            "filler",
            0.2,
            0.9,
            0.56,
            (-0.12, 0.45, WALL + 0.28),
            M.body,
            bevel = 0.006,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "filler_bar",
            &[
                (-0.19, 0.7, WALL + 0.56),
                (-0.19, 0.7, WALL + 0.585),
                (-0.05, 0.7, WALL + 0.585),
                (-0.05, 0.7, WALL + 0.56)
            ],
            0.007,
            M.chrome,
            verts = 6
        );
        fsweep!(
            f,
            "filler_towel",
            &[
                (-0.12, 0.45, WALL + 0.575),
                (-0.12, 0.68, WALL + 0.576),
                (-0.12, 0.712, WALL + 0.585),
                (-0.12, 0.7, WALL + 0.598),
                (-0.12, 0.5, WALL + 0.6)
            ],
            0.0,
            M.towel_a,
            profile = rounded_rect(0.008, 0.13, 0.004, 1),
            up = (0, 1, 0),
            resolution = 2
        );
        // the counter: open cubbies under a top, towels above and baskets below
        fbox!(
            f,
            "counter_top",
            0.94,
            0.04,
            0.58,
            (CX, 0.92, WALL + 0.29),
            M.birch_plain,
            bevel = 0.008,
            bevel_segments = 2
        );
        for y in [0.02, 0.42] {
            fbox!(
                f,
                format!("cubby_shelf{y}"),
                0.9,
                0.04,
                0.56,
                (CX, y, WALL + 0.28),
                M.body,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        for x in [CX - 0.44, CX, CX + 0.44] {
            fbox!(
                f,
                format!("cubby_side{x}"),
                0.03,
                0.88,
                0.56,
                (x, 0.45, WALL + 0.28),
                M.body,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "cubby_back",
            0.9,
            0.88,
            0.02,
            (CX, 0.45, WALL + 0.01),
            M.tile
        );
        _towels(
            f,
            "cubby_towel_l",
            CX - 0.22,
            -0.14,
            0.44,
            4,
            0.26,
            0.2,
            &TOWELS,
        );
        _towels(
            f,
            "cubby_towel_r",
            CX + 0.22,
            -0.14,
            0.44,
            3,
            0.26,
            0.2,
            &[M.towel_w, M.towel_c, M.towel_w],
        );
        for (x, m) in [(CX - 0.22, M.wicker), (CX + 0.22, M.wicker_b)] {
            _basket(
                f,
                &format!("basket{x}"),
                x,
                0.04,
                -0.16,
                0.34,
                0.29,
                0.42,
                m,
            );
        }
        // on the counter: a folded stack, detergents, a mini ironing board and its iron
        _towels(
            f,
            "counter_towel",
            CX - 0.18,
            -0.2,
            0.94,
            3,
            0.3,
            0.22,
            &[M.towel_w, M.towel_a, M.towel_w],
        );
        _bottle(
            f,
            "detergent",
            CX + 0.32,
            -0.3,
            0.94,
            M.bottle_a,
            0.24,
            0.08,
            Some(M.teal),
        );
        fsweep!(
            f,
            "detergent_handle",
            &[
                (CX + 0.32, 1.1, -0.26),
                (CX + 0.32, 1.12, -0.24),
                (CX + 0.32, 1.05, -0.24),
                (CX + 0.32, 1.03, -0.26)
            ],
            0.008,
            M.bottle_a,
            verts = 5,
            resolution = 2
        );
        _bottle(
            f,
            "softener",
            CX + 0.2,
            -0.34,
            0.94,
            M.bottle_b,
            0.2,
            0.08,
            Some(M.coral),
        );
        let board = smooth_outline(
            &[
                (-0.21, -0.1),
                (0.12, -0.1),
                (0.21, -0.04),
                (0.21, 0.04),
                (0.12, 0.1),
                (-0.21, 0.1),
            ],
            2,
            true,
        );
        let board: Vec<(f64, f64)> = board
            .iter()
            .map(|&(u, v)| (CX + 0.12 + u, 0.03 + v))
            .collect();
        fextrude!(
            f,
            "ironing_board",
            &board,
            0.02,
            (0, 0.95, 0),
            M.towel_c,
            plane = Facing::Top,
            bevel = 0.006,
            bevel_segments = 1
        );
        let b = f.at(CX + 0.12, 0.96, 0.03);
        let iron: Vec<Vec<P3>> = [(0.0, 1.0), (0.025, 0.98), (0.04, 0.85)]
            .iter()
            .map(|&(h, k)| {
                [
                    (0.07, 0.0),
                    (0.02, 0.045),
                    (-0.06, 0.045),
                    (-0.065, 0.0),
                    (-0.06, -0.045),
                    (0.02, -0.045),
                ]
                .iter()
                .map(|&(px, py)| P3::from((b.x + px * k, b.y + py * k, b.z + h)))
                .collect()
            })
            .collect();
        loft!("iron", &iron, material = M.appliance);
        fsweep!(
            f,
            "iron_handle",
            &[
                (CX + 0.18, 1.0, 0.03),
                (CX + 0.14, 1.04, 0.03),
                (CX + 0.06, 1.04, 0.03),
                (CX + 0.05, 1.0, 0.03)
            ],
            0.0,
            M.teal,
            profile = rounded_rect(0.014, 0.02, 0.006, 1),
            up = (0, 0, 1),
            resolution = 2
        );
        // the rail over the counter on two posts, shirts on hangers
        kit::tube!(
            f,
            "rail",
            &kit::fillet!(
                &[
                    (CX - 0.42, 0.94, -0.29),
                    (CX - 0.42, 1.72, -0.29),
                    (CX + 0.42, 1.72, -0.29),
                    (CX + 0.42, 0.94, -0.29)
                ],
                0.04,
                steps = 3
            ),
            0.01,
            M.chrome,
            verts = 8,
            round = 1
        );
        for x in [CX - 0.42, CX + 0.42] {
            fcyl!(
                f,
                format!("rail_foot{x}"),
                0.03,
                0.01,
                (x, 0.945, -0.29),
                M.chrome,
                verts = 12
            );
        }
        for (i, m) in [M.towel_c, M.towel_w, M.towel_b, M.towel_f]
            .into_iter()
            .enumerate()
        {
            _shirt(
                f,
                &format!("shirt{i}"),
                CX - 0.27 + i as f64 * 0.18,
                1.72,
                -0.29,
                m,
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The washer's porthole door, hinged on its left: `door_open` swings it open.
    let door = new_parts(|| _porthole(f, "door", cy, zf));
    let door = hinged(join(&door, "door", None), f.at(WX - 0.235, cy, zf + 0.005));
    let objs = settle!(centre = false);
    swing!(
        door,
        "door_open",
        100.0,
        seconds = 0.8,
        ease = Ease::BackOut
    );
    objs
}

// --- the rainfall steam shower (1x1): a glass cabin, tiled back and side, a ceiling rain head, a teak bench ---
const S: f64 = 0.45; // half the cabin's width
const T: f64 = 0.07; // the tray's top
const HC: f64 = 2.18; // the cabin's height
const HEAD_Y: f64 = 2.075; // the rain head's face
const DOOR_HALF: f64 = 0.28; // half the door's width, across the middle of the front

pub fn rain_shower_tower() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let w = 2.0 * S;
    let still = new_parts(|| {
        _box(
            f,
            "tray",
            w,
            T - 0.012,
            w,
            (0, (T - 0.012) / 2.0, 0),
            M.body,
            12.0,
            24,
            0.006,
        );
        soft!(
            f,
            "tray_floor",
            w - 0.06,
            0.012,
            w - 0.06,
            (0, T - 0.018, 0),
            M.body,
            round = 0.006,
            round_segments = 1
        );
        fbox!(
            f,
            "drain",
            0.5,
            0.004,
            0.05,
            (0.0, T - 0.01, -0.3),
            M.chrome,
            bevel = 0.002,
            bevel_segments = 1
        );
        for k in 0..9 {
            // the linear drain's slots
            fbox!(
                f,
                format!("drain_slot{k}"),
                0.035,
                0.002,
                0.02,
                (-0.2 + k as f64 * 0.05, T - 0.008, -0.3),
                M.dark
            );
        }
        fbox!(
            f,
            "back_wall",
            w,
            HC - T,
            0.03,
            (0, T + (HC - T) / 2.0, -S + 0.015),
            M.tile
        );
        fbox!(
            f,
            "side_wall",
            0.03,
            HC - T,
            w - 0.03,
            (-S + 0.015, T + (HC - T) / 2.0, 0.015),
            M.tile
        );
        for k in 1..10 {
            // tile joints: courses, and a joint between each tile
            let y = T + k as f64 * (HC - T) / 10.0;
            fbox!(
                f,
                format!("joint_b{k}"),
                w - 0.04,
                0.006,
                0.004,
                (0, y, -S + 0.031),
                M.joint
            );
            fbox!(
                f,
                format!("joint_s{k}"),
                0.004,
                0.006,
                w - 0.07,
                (-S + 0.031, y, 0.015),
                M.joint
            );
        }
        for k in [1, 2, 3] {
            let kf = k as f64;
            fbox!(
                f,
                format!("joint_bv{k}"),
                0.006,
                HC - T,
                0.004,
                (-S + kf * w / 4.0, T + (HC - T) / 2.0, -S + 0.031),
                M.joint
            );
            fbox!(
                f,
                format!("joint_sv{k}"),
                0.004,
                HC - T,
                0.006,
                (-S + 0.031, T + (HC - T) / 2.0, -S + kf * w / 4.0 + 0.03),
                M.joint
            );
        }
        // a niche in the side wall with bottles in it
        fbox!(
            f,
            "niche_back",
            0.006,
            0.3,
            0.36,
            (-S + 0.032, 1.25, 0.1),
            M.joint
        );
        for y in [1.1, 1.4] {
            fbox!(
                f,
                format!("niche_ledge{y}"),
                0.06,
                0.02,
                0.38,
                (-S + 0.06, y, 0.1),
                M.tile,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for (k, (z, m)) in [(0.0, M.bottle_c), (0.08, M.bottle_b), (0.2, M.bottle_a)]
            .into_iter()
            .enumerate()
        {
            _bottle(
                f,
                &format!("niche_bottle{k}"),
                -S + 0.07,
                z,
                1.11,
                m,
                0.16 - 0.02 * k as f64,
                0.055,
                None,
            );
        }
        // the glass: the right side and the front, a narrow fixed pane either side of the door (its own part)
        fbox!(
            f,
            "glass_side",
            0.012,
            HC - T - 0.06,
            w - 0.06,
            (S - 0.012, T + (HC - T - 0.06) / 2.0, 0.0),
            M.glass
        );
        let pw = S - 0.03 - DOOR_HALF - 0.004;
        for x in [-S + 0.03 + pw / 2.0, S - 0.03 - pw / 2.0] {
            fbox!(
                f,
                format!("glass_front{x}"),
                pw,
                HC - T - 0.08,
                0.012,
                (x, T + (HC - T - 0.08) / 2.0 + 0.01, S - 0.012),
                M.glass
            );
        }
        let prof = rounded_rect(0.03, 0.03, 0.006, 1);
        fsweep!(
            f,
            "frame_bar",
            &[
                (S - 0.015, T, -S + 0.035),
                (S - 0.015, HC - 0.015, -S + 0.035),
                (S - 0.015, HC - 0.015, S - 0.015),
                (-S + 0.03, HC - 0.015, S - 0.015)
            ],
            0.0,
            M.chrome,
            profile = prof,
            up = (1, 0, 0)
        );
        fbox!(
            f,
            "frame_post",
            0.03,
            HC - T,
            0.03,
            (S - 0.015, T + (HC - T) / 2.0, S - 0.015),
            M.chrome,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "frame_sill",
            w - 0.03,
            0.02,
            0.03,
            (0.015, T + 0.01, S - 0.015),
            M.chrome,
            bevel = 0.004,
            bevel_segments = 1
        );
        for y in [T + 0.2, HC - 0.25] {
            // the door's hinges on the right pane
            soft!(
                f,
                format!("hinge{y}"),
                0.05,
                0.05,
                0.03,
                (DOOR_HALF + 0.01, y, S - 0.012),
                M.chrome,
                round = 0.008,
                round_segments = 1
            );
        }
        // the rain head on its arm from the back wall
        soft!(
            f,
            "rain_head",
            0.44,
            0.02,
            0.44,
            (0, HEAD_Y + 0.012, 0.0),
            M.chrome,
            round = 0.008,
            round_segments = 1
        );
        fbox!(
            f,
            "rain_face",
            0.4,
            0.004,
            0.4,
            (0, HEAD_Y + 0.001, 0.0),
            M.steel
        );
        kit::tube!(
            f,
            "rain_arm",
            &kit::fillet!(
                &[
                    (0.0, HEAD_Y + 0.02, 0.0),
                    (0.0, HEAD_Y + 0.085, 0.0),
                    (0.0, HEAD_Y + 0.085, -S + 0.03)
                ],
                0.04
            ),
            0.014,
            M.chrome,
            verts = 8,
            round = 1
        );
        revolve!(
            "rain_rose",
            &[(0.0, 0.0), (0.035, 0.0), (0.035, 0.008), (0.0, 0.012)],
            at = f.at(0, HEAD_Y + 0.085, -S + 0.03),
            material = M.chrome,
            segments = 12,
            rot = (90, 0, 0)
        );
        // the steam column with its jets, the control panel
        soft!(
            f,
            "steam_column",
            0.16,
            1.2,
            0.04,
            (-0.24, 1.15, -S + 0.05),
            M.appliance,
            round = 0.012,
            round_segments = 1
        );
        for k in 0..5 {
            revolve!(
                format!("jet{k}"),
                &[
                    (0.0, 0.0),
                    (0.02, 0.0),
                    (0.016, 0.01),
                    (0.008, 0.012),
                    (0.0, 0.008)
                ],
                at = f.at(-0.24, 0.75 + k as f64 * 0.18, -S + 0.07),
                material = M.chrome,
                segments = 10,
                rot = (90, 0, 0)
            );
        }
        soft!(
            f,
            "controls",
            0.2,
            0.12,
            0.012,
            (0.22, 1.25, -S + 0.036),
            M.screen,
            round = 0.006,
            round_segments = 1
        );
        fbox!(
            f,
            "controls_line",
            0.12,
            0.04,
            0.004,
            (0.22, 1.27, -S + 0.043),
            M.teal
        );
        for k in 0..3 {
            fcyl!(
                f,
                format!("control_button{k}"),
                0.01,
                0.006,
                (0.16 + k as f64 * 0.06, 1.215, -S + 0.043),
                M.chrome,
                rot = (90, 0, 0),
                verts = 8
            );
        }
        // the teak bench along the back: slats on two shaped legs, towels and bottles on it
        for k in 0..4 {
            fbox!(
                f,
                format!("bench_slat{k}"),
                0.72,
                0.03,
                0.06,
                (0.0, 0.455, -0.345 + k as f64 * 0.07),
                M.teak,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for x in [-0.3, 0.3] {
            fextrude!(
                f,
                format!("bench_leg{x}"),
                &smooth_outline(
                    &[
                        (-0.36, 0.0),
                        (-0.12, 0.0),
                        (-0.16, 0.08),
                        (-0.32, 0.08),
                        (-0.32, 0.37),
                        (-0.12, 0.37),
                        (-0.12, 0.44),
                        (-0.36, 0.44)
                    ],
                    0,
                    true
                ),
                0.035,
                (x, 0, 0),
                M.teak,
                plane = Facing::Side,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        _towels(
            f,
            "bench_towel",
            0.2,
            -0.26,
            0.47,
            2,
            0.24,
            0.2,
            &[M.pad, M.pastel_a],
        ); // satin: no fabric draw
        _bottle(
            f,
            "bench_bottle_a",
            -0.22,
            -0.3,
            0.47,
            M.bottle_c,
            0.16,
            0.06,
            None,
        );
        _bottle(
            f,
            "bench_bottle_b",
            -0.12,
            -0.3,
            0.47,
            M.bottle_b,
            0.18,
            0.06,
            None,
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The rain: streaks from the head's face to the tray, origin at the head: `water_on` grows them down.
    let rain = new_parts(|| {
        let mut rnd = Random::new(3);
        for i in 0..5 {
            for j in 0..5 {
                let x = -0.16 + i as f64 * 0.08 + rnd.uniform(-0.01, 0.01);
                let z = -0.16 + j as f64 * 0.08 + rnd.uniform(-0.01, 0.01);
                let a = f.at(x, HEAD_Y, z);
                let b = f.at(x, T + rnd.uniform(0.0, 0.03), z);
                rod!(
                    format!("stream{i}{j}"),
                    a,
                    b,
                    0.004,
                    material = M.water,
                    verts = 4
                );
            }
        }
    });
    let rain = join(&rain, "water", Some(f.at(0, HEAD_Y, 0)));
    rain.set_scale_axis(2, 0.001);
    // The glass door across the middle of the front, hinged on its right: `door_open` swings it out to let the bather
    // in and out (a fourth draw: Q-05). All glass, no pull, to keep it one draw.
    let door = new_parts(|| {
        fbox!(
            f,
            "glass_door",
            2.0 * DOOR_HALF,
            HC - T - 0.1,
            0.012,
            (0, T + (HC - T - 0.1) / 2.0 + 0.02, S - 0.012),
            M.glass
        );
    });
    let door = hinged(join(&door, "door", None), f.at(DOOR_HALF, T, S - 0.012));
    let objs = settle(&meshes()); // on its Model centre: the back on the wall face
    grow!(rain, "water_on", seconds = 0.6);
    swing!(
        door,
        "door_open",
        -95.0,
        seconds = 0.8,
        ease = Ease::BackOut
    );
    objs
}
