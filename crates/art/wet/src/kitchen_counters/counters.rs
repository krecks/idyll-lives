//! The base counters (K4): the plain counter, the corner counter, the counter end, the outer corner counter, the
//! drawer counter and the open-shelf counter.

use super::common::{doors, drawer, plain, pull, shaker, unit66, unit79, worktop};
use super::turned::{bowl, can, canister, jug, lathe, plates};
use super::{COUNTER66, M, SECONDS};
use artkit::forms::rounded_rect;
use artkit::math::{cos, radians, sin};
use artkit::parts::{aside, join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, revolve, slide};

// --- the plain counter: 1x1, a drawer over two doors, the drawer sliding out for a sandwich -----------
pub fn counter() -> Vec<Obj> {
    let f = &GameFrame::new(0.15);
    let still = new_parts(|| {
        unit66(f, 1.0, None, None);
        doors(
            f,
            0.974,
            0.14,
            0.12,
            0.72,
            None,
            None,
            Some(0.62),
            0.0,
            0.019,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let drawer = drawer(
        f, "drawer", 0.974, 0.815, 0.16, M.cab, None, 0.18, true, 0.019, 0.04, 0.17,
    );
    settle_all();
    slide!(drawer, "drawer_open", (0, -0.25, 0), seconds = SECONDS);
    meshes()
}

/// A stoneware crock on the worktop at (cx, y, cz), a wooden spoon, a slotted spatula and a whisk standing in it
/// (their tops at 1.26).
fn utensil_crock(f: &GameFrame, cx: f64, y: f64, cz: f64) {
    lathe(
        f,
        "crock",
        &[
            (0.0, 0.0),
            (0.06, 0.0),
            (0.066, 0.008),
            (0.07, 0.03),
            (0.064, 0.15),
            (0.066, 0.172),
            (0.07, 0.18),
            (0.062, 0.18),
            (0.058, 0.16),
            (0.056, 0.03),
            (0.0, 0.03),
        ],
        cx,
        y,
        cz,
        M.porcelain,
        16,
    );
    fsweep!(
        f,
        "spoon_handle",
        &[
            (cx - 0.01, y + 0.04, cz),
            (cx - 0.024, y + 0.16, cz + 0.006),
            (cx - 0.034, y + 0.248, cz + 0.01)
        ],
        0.0065,
        M.spoon,
        verts = 6,
        scale = [1.0, 0.9, 0.75]
    );
    fsphere!(
        f,
        "spoon_bowl",
        0.028,
        (cx - 0.037, y + 0.276, cz + 0.011),
        M.spoon,
        scale = (0.8, 0.4, 1.25),
        segments = 10,
        rings = 6
    );
    fsweep!(
        f,
        "spatula_handle",
        &[
            (cx + 0.012, y + 0.04, cz - 0.004),
            (cx + 0.026, y + 0.16, cz - 0.01),
            (cx + 0.034, y + 0.215, cz - 0.012)
        ],
        0.0065,
        M.spoon,
        verts = 6,
        scale = [1.0, 0.9, 0.8]
    );
    let slots: Vec<Vec<(f64, f64)>> = [-0.013, 0.0, 0.013]
        .into_iter()
        .map(|du| {
            rounded_rect(0.007, 0.04, 0.0035, 2)
                .into_iter()
                .map(|(u, v)| (u + du, v))
                .collect()
        })
        .collect();
    fextrude!(
        f,
        "spatula",
        &rounded_rect(0.052, 0.07, 0.014, 2),
        0.004,
        (cx + 0.036, y + 0.248, cz - 0.012),
        M.spoon,
        holes = slots,
        rot = (0, 0, 8)
    );
    fcyl!(
        f,
        "whisk_handle",
        0.008,
        0.12,
        (cx + 0.004, y + 0.11, cz + 0.022),
        M.steel,
        verts = 8
    );
    for i in 0..3 {
        let a = radians((60 * i) as f64);
        let (c, s) = (cos(a), sin(a));
        let lp = [
            (0.003, 0.17),
            (0.018, 0.215),
            (0.024, 0.255),
            (0.016, 0.292),
            (0.0, 0.308),
            (-0.016, 0.292),
            (-0.024, 0.255),
            (-0.018, 0.215),
            (-0.003, 0.17),
        ];
        let pts: Vec<(f64, f64, f64)> = lp
            .into_iter()
            .map(|(d, t)| (cx + 0.004 + c * d, y + t, cz + 0.022 + s * d))
            .collect();
        fsweep!(
            f,
            format!("whisk_wire{i}"),
            &pts,
            0.0015,
            M.steel,
            verts = 4,
            resolution = 2
        );
    }
}

// --- the corner counter: 1x1, a deep L unit with a drawer, two doors and a utensil pot ----------------
pub fn counter_corner() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // its back on the wall face
        fbox!(
            f,
            "carcass",
            1.0,
            0.82,
            0.89,
            (0, 0.51, 0.015),
            M.cab,
            bevel = 0.012
        );
        fbox!(
            f,
            "plinth",
            0.96,
            0.10,
            0.85,
            (0, 0.05, -0.005),
            M.dark,
            bevel = 0.006
        );
        worktop(
            f, "worktop", 1.0, 0, -0.43, 0.50, 0.91, 0.95, M.worktop, 0.012,
        );
        doors(
            f,
            0.974,
            0.46,
            0.12,
            0.72,
            None,
            None,
            Some(0.62),
            0.0,
            0.019,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // a crock of wooden spoons, a spatula and a whisk on the worktop: drawn, not the counter's size
    let crock = new_parts(|| utensil_crock(f, -0.3, 0.95, -0.3));
    aside(&crock, "crock");
    let drawer = drawer(
        f, "drawer", 0.974, 0.815, 0.48, M.cab, None, 0.18, true, 0.019, 0.04, 0.17,
    );
    settle_all();
    slide!(drawer, "drawer_open", (0, -0.25, 0), seconds = SECONDS);
    meshes()
}

/// The counter end's open unit of oak at x (0.16 wide): a back, two sides, a bottom and two shelves, each shelf with
/// a lip along its front.
fn shelf_unit(f: &GameFrame, x: f64) {
    fbox!(
        f,
        "end_back",
        0.16,
        0.8,
        0.03,
        (x, 0.5, -0.43),
        M.oak,
        bevel = 0.004
    );
    for s in [-1, 1] {
        fbox!(
            f,
            format!("end_side{s}"),
            0.012,
            0.8,
            0.7,
            (x + s as f64 * 0.074, 0.5, -0.11),
            M.oak,
            bevel = 0.004
        );
    }
    fbox!(
        f,
        "end_bottom",
        0.16,
        0.05,
        0.7,
        (x, 0.125, -0.11),
        M.oak,
        bevel = 0.004
    );
    for (i, y) in [0.38, 0.66].into_iter().enumerate() {
        fbox!(
            f,
            format!("end_shelf{i}"),
            0.136,
            0.025,
            0.66,
            (x, y, -0.11),
            M.oak,
            bevel = 0.004
        );
        fbox!(
            f,
            format!("end_lip{i}"),
            0.136,
            0.018,
            0.012,
            (x, y + 0.018, 0.21),
            M.oak,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
}

// --- the counter end: 1x1, a unit with one door and an open end unit of two shelves -------------------
pub fn counter_end() -> Vec<Obj> {
    let f = &GameFrame::with_depths(0.085, &COUNTER66);
    let (p, front) = (&plain(f), f.depth(0.24));
    let still = new_parts(|| {
        unit79(f, 0.84, -0.08, None, None, false);
        shaker(
            p, "door", 0.808, 0.74, -0.08, 0.49, front, M.cab, 0.06, 0.02, None,
        );
        pull(
            p,
            "door_pull",
            -0.452,
            0.74,
            front + 0.02,
            0.14,
            None,
            true,
            0.019,
            0.005,
            0.0,
        );
        // one worktop over the unit and the open end, as a run's
        worktop(
            f, "worktop", 1.0, 0, -0.48, 0.31, 0.90, 0.95, M.worktop, 0.012,
        );
        // the open end unit, a jug, a canister and two tins on its shelves
        shelf_unit(f, 0.42);
        jug(
            f,
            "jug",
            0.42,
            0.3925,
            0.0,
            0.05,
            0.12,
            M.porcelain,
            -90.0,
            14,
        );
        canister(
            f, "jar", 0.42, 0.6725, -0.15, 0.05, 0.12, M.cream, M.spoon, 14,
        );
        can(f, "tin", 0.42, 0.15, -0.05, 0.05, 0.09, M.rust, 12);
        can(f, "tin2", 0.42, 0.15, 0.08, 0.045, 0.11, M.teal, 12);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the outer corner counter: 1x1, a short unit with a side panel, a plate and a lemon ---------------
pub fn counter_corner_outer() -> Vec<Obj> {
    let f = &GameFrame::with_depths(0.085, &COUNTER66);
    let (p, front) = (&plain(f), f.depth(0.24));
    let still = new_parts(|| {
        fbox!(
            f,
            "carcass",
            0.74,
            0.8,
            0.7,
            (-0.13, 0.5, -0.11),
            M.cab,
            bevel = 0.01
        );
        fbox!(
            f,
            "plinth",
            0.70,
            0.10,
            0.64,
            (-0.15, 0.05, -0.14),
            M.dark,
            bevel = 0.004
        );
        worktop(
            f, "worktop", 0.81, -0.095, -0.48, 0.31, 0.90, 0.95, M.worktop, 0.012,
        );
        shaker(
            p, "door", 0.708, 0.74, -0.13, 0.49, front, M.cab, 0.06, 0.02, None,
        );
        pull(
            p,
            "door_pull",
            -0.452,
            0.74,
            front + 0.02,
            0.14,
            None,
            true,
            0.019,
            0.005,
            0.0,
        );
        // the end panel that closes the run, a towel rail on its outer face with a tea towel over it
        fbox!(
            f,
            "panel",
            0.02,
            0.74,
            0.66,
            (0.25, 0.49, -0.11),
            M.cab,
            bevel = 0.006
        );
        let zr = f.depth(0.12);
        fsweep!(
            p,
            "towel_rail",
            &[
                (0.258, 0.74, zr - 0.13),
                (0.274, 0.74, zr - 0.122),
                (0.274, 0.74, zr + 0.03),
                (0.258, 0.74, zr + 0.04)
            ],
            0.005,
            M.chrome,
            verts = 8,
            resolution = 2
        );
        let zt = zr - 0.045;
        fsweep!(
            p,
            "towel",
            &[
                (0.266, 0.56, zt),
                (0.266, 0.735, zt),
                (0.2675, 0.7475, zt),
                (0.274, 0.7505, zt),
                (0.2805, 0.7475, zt),
                (0.282, 0.735, zt),
                (0.282, 0.6, zt)
            ],
            0.0,
            M.towel,
            profile = rounded_rect(0.15, 0.003, 0.0014, 1),
            up = (0, 1, 0),
            resolution = 2
        );
        // a plate with two lemons and a leaf on the worktop
        plates(
            f,
            "plate",
            0.12,
            0.95,
            0.08,
            0.12,
            1,
            M.porcelain,
            0.016,
            20,
        );
        for (i, (x, z, t)) in [(0.10, 0.07, 10), (0.16, 0.12, 70)].into_iter().enumerate() {
            revolve!(
                format!("lemon{i}"),
                &[
                    (0.0, -0.05),
                    (0.008, -0.046),
                    (0.022, -0.035),
                    (0.033, -0.015),
                    (0.035, 0.0),
                    (0.033, 0.015),
                    (0.022, 0.035),
                    (0.008, 0.046),
                    (0.0, 0.05)
                ],
                at = f.at(x, 1.0, z),
                material = M.yellow,
                segments = 10,
                rot = (0, 90, t)
            );
        }
        fsphere!(
            f,
            "leaf",
            0.03,
            (0.06, 0.968, 0.11),
            M.leaf,
            scale = (1.4, 0.12, 0.6),
            rot = (0, 0, 35),
            segments = 8,
            rings = 4
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the drawer counter: 1x1, four deep drawers under the worktop (static) ----------------------------
pub fn counter_drawers() -> Vec<Obj> {
    let f = &GameFrame::with_depths(0.085, &COUNTER66);
    let (p, front) = (&plain(f), f.depth(0.24));
    let still = new_parts(|| {
        unit79(f, 1.0, 0.0, None, None, true);
        for (i, y) in [0.2125, 0.3975, 0.5825].into_iter().enumerate() {
            shaker(
                p,
                &format!("drawer{i}"),
                0.97,
                0.173,
                0,
                y,
                front,
                M.cab,
                0.04,
                0.02,
                None,
            );
            pull(
                p,
                format!("drawer_pull{i}"),
                0,
                y,
                front + 0.02,
                0.30,
                None,
                false,
                0.019,
                0.005,
                0.008,
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the top drawer with its box and a cutlery tray behind, sliding out while a resident puts clean dishes away
    let drawer = drawer(
        p,
        "drawer",
        0.97,
        0.7675,
        front + 0.02,
        M.cab,
        None,
        0.30,
        true,
        0.019,
        0.04,
        0.173,
    );
    settle_all();
    slide!(drawer, "drawer_open", (0, -0.25, 0), seconds = SECONDS);
    meshes()
}

// --- the open-shelf counter: 1x1, an open carcass with two shelves of plates and pots (static) --------
pub fn counter_open_shelf() -> Vec<Obj> {
    let f = &GameFrame::with_depths(0.085, &COUNTER66);
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            0.996,
            0.10,
            0.64,
            (0, 0.05, -0.14),
            M.dark,
            bevel = 0.004
        );
        worktop(
            f, "worktop", 1.0, 0, -0.48, 0.31, 0.90, 0.95, M.worktop, 0.012,
        );
        // an open carcass: a back, two sides, a top and a bottom, with two shelves between
        // (dark inside, the cabinet's colour outside, for one standing free)
        fbox!(
            f,
            "back",
            0.996,
            0.8,
            0.02,
            (0, 0.5, -0.42),
            M.soil,
            bevel = 0.004
        );
        fbox!(
            f,
            "back_skin",
            0.996,
            0.8,
            0.02,
            (0, 0.5, -0.44),
            M.cab,
            bevel = 0.004
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("side{s}"),
                0.03,
                0.8,
                0.66,
                (s as f64 * 0.483, 0.5, -0.11),
                M.cab,
                bevel = 0.004
            );
        }
        fbox!(
            f,
            "top",
            0.996,
            0.03,
            0.7,
            (0, 0.885, -0.11),
            M.cab,
            bevel = 0.004
        );
        fbox!(
            f,
            "bottom",
            0.996,
            0.05,
            0.7,
            (0, 0.125, -0.11),
            M.cab,
            bevel = 0.004
        );
        for (i, y) in [0.16, 0.5].into_iter().enumerate() {
            fbox!(
                f,
                format!("shelf{i}"),
                0.93,
                0.025,
                0.66,
                (0, y, -0.11),
                M.oak,
                bevel = 0.004
            );
            fbox!(
                f,
                format!("shelf_lip{i}"),
                0.93,
                0.04,
                0.02,
                (0, y + 0.0075, 0.21),
                M.oak,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // plates, nested bowls and a serving dish below; a jug, a lidded casserole and a dish above
        let (lo, hi) = (0.1725, 0.5125);
        plates(
            f,
            "plates0",
            -0.32,
            lo,
            -0.1,
            0.1,
            5,
            M.porcelain,
            0.013,
            12,
        );
        for i in 0..3 {
            bowl(
                f,
                format!("bowl{i}"),
                -0.08,
                lo + 0.022 * i as f64,
                -0.1,
                0.075 - 0.008 * i as f64,
                0.07 - 0.006 * i as f64,
                M.cream,
                10,
            );
        }
        plates(
            f,
            "plates1",
            0.14,
            lo,
            -0.1,
            0.085,
            4,
            M.porcelain,
            0.013,
            12,
        );
        bowl(f, "bowl3", 0.35, lo, -0.12, 0.065, 0.075, M.cream, 10);
        jug(f, "jug", -0.3, hi, -0.1, 0.07, 0.16, M.sage, 200.0, 12);
        lathe(
            f,
            "pot",
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.075, 0.008),
                (0.075, 0.1),
                (0.0, 0.1),
            ],
            -0.08,
            hi,
            -0.1,
            M.teal,
            12,
        );
        lathe(
            f,
            "pot_lid",
            &[
                (0.0, 0.098),
                (0.078, 0.098),
                (0.074, 0.11),
                (0.04, 0.125),
                (0.012, 0.127),
                (0.016, 0.142),
                (0.0, 0.145),
            ],
            -0.08,
            hi,
            -0.1,
            M.teal,
            12,
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("pot_handle{s}"),
                0.024,
                0.014,
                0.04,
                (-0.08 + s as f64 * 0.086, hi + 0.085, -0.1),
                M.teal,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        bowl(f, "dish0", 0.25, hi, -0.1, 0.11, 0.05, M.porcelain, 12);
        bowl(
            f,
            "dish1",
            0.25,
            hi + 0.04,
            -0.1,
            0.1,
            0.045,
            M.porcelain,
            12,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}
