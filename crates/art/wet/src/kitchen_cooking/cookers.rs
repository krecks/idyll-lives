//! The cookers: the electric cooker, the gas range, the oven tower, the range cooker and the hob counter.

use super::common::{burner, flame, hob_ring, hot_ring, knob, knob_obj, light};
use super::ovens::{drawer_front, oven, oven_door, oven_face, stock_pot, strip, tray};
use super::{COUNTER66, M, SECONDS};
use crate::common::shaker;
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::kit::fillet;
use artkit::math::{Axis, cos, radians, sin};
use artkit::parts::{join, new_parts, set_origin, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsweep, slide, swing};

// --- electric cooker: 1x1, four ceramic rings and an oven ---------------------------------------------

/// A freestanding electric cooker: a black ceramic hob with four printed zones, a rounded splash guard, a control
/// fascia with knobs and a clock, an oven (a window in its door, a rack and a lamp inside) over a storage drawer.
pub fn stove_electric() -> Vec<Obj> {
    let f = &GameFrame::new(0.1295);
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            0.56,
            0.05,
            0.54,
            (0, 0.025, -0.17),
            M.dark,
            bevel = 0.005
        );
        let carcass = fbox!(
            f,
            "carcass",
            0.60,
            0.82,
            0.60,
            (0, 0.46, -0.16),
            M.enamel,
            bevel = 0.01
        );
        oven(f, carcass, "oven", 0.0, 0.43, 0.14, 0.48, 0.38, 0.46);
        fbox!(
            f,
            "hob",
            0.60,
            0.02,
            0.60,
            (0, 0.88, -0.16),
            M.hob,
            bevel = 0.006,
            bevel_segments = 2
        );
        fbox!(
            f,
            "back",
            0.60,
            0.09,
            0.10,
            (0, 0.935, -0.41),
            M.enamel,
            bevel = 0.02,
            bevel_segments = 3
        );
        fbox!(
            f,
            "back_vent",
            0.4,
            0.004,
            0.025,
            (0, 0.9775, -0.41),
            M.dark
        );
        strip(f, "strip", 0.56, 0.845, 0.136, 0.06, 0.0, Some(0.0));
        for (i, x) in [-0.07, 0.07, 0.21].into_iter().enumerate() {
            knob(
                f,
                &format!("knob{}", i + 1),
                x,
                0.845,
                0.155,
                0.022,
                0.03,
                None,
                true,
            );
        }
        drawer_front(f, "drawer", 0.56, 0.10, (0, 0.12, 0.15), M.enamel, 0.3);
        for (i, (x, z)) in [(-0.16, -0.30), (0.16, -0.30), (-0.16, -0.05), (0.16, -0.05)]
            .into_iter()
            .enumerate()
        {
            hob_ring(
                f,
                &format!("ring{i}"),
                x,
                z,
                0.8915,
                if i != 1 { 0.07 } else { 0.06 },
                i == 0 || i == 2,
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let door = oven_door(f, "oven_door", 0.0, 0.56, 0.43, 0.152, 0.17, 0.50, 0.0345);
    let knob = knob_obj(f, "knob0", -0.21, 0.845, 0.155, 0.022, 0.03);
    // the front ring on the knob's side glows red while it's on, as the hob counter's does
    let ring = hot_ring(f, "hot_ring", -0.16, 0.892, -0.05, 0.066);
    settle_all();
    swing!(door, "oven_open", 85, seconds = SECONDS, axis = Axis::X);
    swing!(knob, "burner_on", 90, seconds = 0.5, axis = Axis::Y);
    light(ring);
    Vec::new()
}

// --- gas range: 1x1, four burners with flames, a taller back guard and an oven -------------------------

/// A gas range: a dark hob with four burners under cast-iron pan supports (a stock pot on one), a splash guard, a
/// control fascia with five knobs, an oven with a window over a storage drawer.
pub fn stove_gas() -> Vec<Obj> {
    let f = &GameFrame::new(0.112);
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            0.88,
            0.06,
            0.58,
            (0, 0.03, -0.17),
            M.dark,
            bevel = 0.005
        );
        let carcass = fbox!(
            f,
            "carcass",
            0.92,
            0.78,
            0.64,
            (0, 0.45, -0.14),
            M.enamel,
            bevel = 0.01
        );
        oven(f, carcass, "oven", 0.0, 0.42, 0.18, 0.78, 0.38, 0.5);
        fbox!(
            f,
            "hob",
            0.92,
            0.03,
            0.64,
            (0, 0.855, -0.14),
            M.dark,
            bevel = 0.006,
            bevel_segments = 2
        );
        fbox!(
            f,
            "back",
            0.92,
            0.18,
            0.04,
            (0, 0.96, -0.45),
            M.enamel,
            bevel = 0.01,
            bevel_segments = 2
        );
        strip(f, "strip", 0.86, 0.80, 0.18, 0.06, 0.0, None);
        for (i, x) in [-0.16, 0.0, 0.16, 0.32].into_iter().enumerate() {
            knob(
                f,
                &format!("knob{}", i + 1),
                x,
                0.80,
                0.207,
                0.022,
                0.03,
                None,
                true,
            );
        }
        drawer_front(f, "drawer", 0.88, 0.10, (0, 0.11, 0.17), M.enamel, 0.4);
        for (i, (x, z)) in [(-0.26, -0.30), (0.26, -0.30), (-0.26, -0.02), (0.26, -0.02)]
            .into_iter()
            .enumerate()
        {
            burner(f, &format!("burner{i}"), x, z, 0.87, 0.07);
        }
        // a stock pot on the back left pan support; the front left burner is the cook's (the flame lights under their
        // pan) and the two on the right are left free, for what they set down (the frying pan that stood there made
        // three pans of it while they cooked)
        stock_pot(f, "pot", -0.26, 0.891, -0.30, 0.13, 0.18, M.pot);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let door = oven_door(f, "oven_door", 0.0, 0.88, 0.42, 0.17, 0.17, 0.50, 0.0345);
    // the knob stands on the fascia, in front of where the old one was: it turns about the same front-to-back axis
    let knob = knob_obj(f, "knob0", -0.32, 0.80, 0.207, 0.022, 0.03);
    set_origin(knob, f.at(-0.32, 0.80, 0.165));
    let flame = flame(f, "flame", -0.26, 0.885, -0.02, 0.075);
    settle_all();
    swing!(door, "oven_open", 85, seconds = SECONDS, axis = Axis::X);
    swing!(knob, "burner_on", 90, seconds = 0.5, axis = Axis::Y);
    light(flame);
    Vec::new()
}

// --- oven tower: 1x1, two built-in ovens in a tall cabinet ---------------------------------------------

/// A tall cabinet under a moulded cornice holding two ovens and a control panel between them, a shaker door below
/// and one above; the upper oven's door drops and its tray slides out.
pub fn wall_oven() -> Vec<Obj> {
    let f = &GameFrame::new(0.102);
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            0.66,
            0.06,
            0.54,
            (0, 0.03, -0.16),
            M.dark,
            bevel = 0.005
        );
        let cabinet = fbox!(
            f,
            "cabinet",
            0.70,
            1.98,
            0.60,
            (0, 1.05, -0.15),
            M.cab,
            bevel = 0.006
        );
        oven(f, cabinet, "oven", 0.0, 1.62, 0.15, 0.52, 0.40, 0.46);
        fbox!(f, "cornice", 0.66, 0.05, 0.60, (0, 2.065, -0.16), M.cab);
        let mut cove = vec![(0.0, -0.01), (0.0, 0.004)];
        for a in (0..=90).step_by(18) {
            let a = a as f64;
            cove.push((
                0.004 + 0.056 * sin(radians(a)),
                0.004 + 0.016 * (1.0 - cos(radians(a))),
            ));
        }
        cove.extend([(0.07, 0.02), (0.07, -0.01)]);
        fsweep!(
            f,
            "cornice_moulding",
            &[
                (0.33, 2.04, -0.46),
                (0.33, 2.04, 0.14),
                (-0.33, 2.04, 0.14),
                (-0.33, 2.04, -0.46)
            ],
            0.0,
            M.cab,
            profile = cove,
            up = (0, 0, 1),
            angle = 50.0
        );
        // the lower oven (it stays shut), the controls between the two, the cabinet doors
        oven_face(f, "oven_low", 0.0, 0.60, 0.98, 0.16, 0.50, 0.0735);
        strip(f, "controls", 0.60, 1.30, 0.15, 0.1, 0.0, Some(0.0));
        for s in [-1, 1] {
            knob(
                f,
                &format!("knob{s}"),
                s as f64 * 0.2,
                1.30,
                0.177,
                0.024,
                0.03,
                None,
                true,
            );
        }
        shaker(
            f,
            "door_bottom",
            0.60,
            0.52,
            (0, 0.40, 0.16),
            M.cab,
            0.02,
            0.06,
            0.004,
        );
        pull!(
            f,
            "pull_bottom",
            (0.24, 0.5),
            (0.24, 0.62),
            0.17,
            out = 0.03,
            w = 0.012,
            t = 0.012,
            r = Some(0.005)
        );
        shaker(
            f,
            "door_top",
            0.60,
            0.16,
            (0, 1.96, 0.16),
            M.cab,
            0.02,
            0.04,
            0.004,
        );
        pull!(
            f,
            "pull_top",
            (0.24, 1.9),
            (0.24, 2.0),
            0.17,
            out = 0.03,
            w = 0.012,
            t = 0.012,
            r = Some(0.005)
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the upper oven door (hinged at its bottom) and its tray, empty: it slides out for whatever goes in (a roast, a
    // pie, a meal's dish, none of which the game draws in an oven); its rolls showed for every one (2026-11-03)
    let door = oven_door(f, "oven_door", 0.0, 0.60, 1.62, 0.16, 1.37, 0.50, 0.0735);
    let parts = new_parts(|| tray(f, "tray", 0.0, 1.585, -0.015, 0.46, 0.31, 0.022));
    let tray = join(&parts, "tray", Some(f.at(0, 1.60, -0.17)));
    settle_all();
    swing!(door, "oven_open", 85, seconds = SECONDS, axis = Axis::X);
    slide!(tray, "oven_open", (0, -0.10, 0), seconds = SECONDS);
    Vec::new()
}

// --- range cooker: 2x1, six burners, two ovens and a chrome rail ---------------------------------------

/// A cream range cooker: six burners under cast-iron supports (a stock pot on one), a tall splash back, a control
/// fascia with six knobs, two ovens with framed windows (the left one opens onto a rack), a chrome rail across the
/// front with a tea towel over it.
pub fn range_cooker() -> Vec<Obj> {
    let f = &GameFrame::new(0.1025);
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            1.54,
            0.06,
            0.6,
            (0, 0.03, -0.16),
            M.dark,
            bevel = 0.005
        );
        let carcass = fbox!(
            f,
            "carcass",
            1.58,
            0.80,
            0.66,
            (0, 0.46, -0.14),
            M.enamel2,
            bevel = 0.012
        );
        oven(f, carcass, "oven", -0.40, 0.42, 0.19, 0.62, 0.38, 0.52);
        fbox!(
            f,
            "hob",
            1.58,
            0.03,
            0.66,
            (0, 0.875, -0.14),
            M.dark,
            bevel = 0.006,
            bevel_segments = 2
        );
        fbox!(
            f,
            "back",
            1.58,
            0.24,
            0.04,
            (0, 1.01, -0.46),
            M.enamel2,
            bevel = 0.012,
            bevel_segments = 2
        );
        fbox!(
            f,
            "back_trim",
            1.5,
            0.012,
            0.012,
            (0, 1.115, -0.436),
            M.chrome,
            bevel = 0.004,
            bevel_segments = 1
        );
        strip(f, "strip", 1.5, 0.82, 0.19, 0.06, 0.0, Some(-0.24));
        for (i, x) in [-0.6, -0.36, 0.12, 0.36, 0.6].into_iter().enumerate() {
            knob(
                f,
                &format!("knob{}", i + 1),
                x,
                0.82,
                0.215,
                0.022,
                0.03,
                None,
                true,
            );
        }
        // the right oven stays shut; the left one drops (built below)
        oven_face(f, "oven_right", 0.40, 0.72, 0.42, 0.20, 0.50, 0.0345);
        for (i, (x, z)) in [
            (-0.55, -0.30),
            (0.0, -0.30),
            (0.55, -0.30),
            (-0.55, -0.02),
            (0.0, -0.02),
            (0.55, -0.02),
        ]
        .into_iter()
        .enumerate()
        {
            burner(f, &format!("burner{i}"), x, z, 0.89, 0.07);
        }
        stock_pot(f, "pot", -0.55, 0.911, -0.30, 0.14, 0.19, M.pot);
        // the chrome rail across the front in one piece, posts and all, and a tea towel hung over it
        kit::tube!(
            f,
            "rail",
            &fillet(
                &[
                    (-0.7, 0.90, 0.19),
                    (-0.7, 0.92, 0.263),
                    (0.7, 0.92, 0.263),
                    (0.7, 0.90, 0.19),
                ],
                0.03,
                3,
            ),
            0.012,
            M.chrome,
            verts = 10,
            round = 1
        );
        fsweep!(
            f,
            "towel",
            &[
                (0.24, 0.73, 0.277),
                (0.24, 0.93, 0.276),
                (0.24, 0.935, 0.263),
                (0.24, 0.93, 0.25),
                (0.24, 0.86, 0.246)
            ],
            0.0,
            M.towel,
            profile = rounded_rect(0.17, 0.006, 0.003, 1),
            up = (1, 0, 0),
            resolution = 2,
            angle = 70.0
        );
        for (k, y) in [0.76, 0.79].into_iter().enumerate() {
            fbox!(
                f,
                format!("towel_stripe{k}"),
                0.172,
                0.012,
                0.007,
                (0.24, y, 0.277),
                M.heat
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let door = oven_door(f, "oven_door", -0.40, 0.72, 0.42, 0.20, 0.17, 0.50, 0.0345);
    // the knob turned is the one nearest the middle (the six run left to right, back and front, as the burners): it
    // lights the front middle burner, not the far one at the end (2026-11-03)
    let knob = knob_obj(f, "knob0", -0.12, 0.82, 0.215, 0.022, 0.03);
    let flame = flame(f, "flame", 0.0, 0.905, -0.02, 0.075);
    settle_all();
    swing!(door, "oven_open", 85, seconds = SECONDS, axis = Axis::X);
    swing!(knob, "burner_on", 90, seconds = 0.5, axis = Axis::Y);
    light(flame);
    Vec::new()
}

// --- hob counter: 1x1, an induction hob in a worktop over a base unit -----------------------------------

/// An induction hob in a steel trim set into a worktop, its touch controls printed along the front (no pan on it:
/// the model is no taller than the hob); a control fascia with three knobs over two deep pan drawers.
pub fn cooktop_counter() -> Vec<Obj> {
    let f = &GameFrame::with_depths(0.085, &COUNTER66);
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            0.96,
            0.10,
            0.64,
            (0, 0.05, -0.14),
            M.dark,
            bevel = 0.005
        );
        fbox!(
            f,
            "carcass",
            1.0,
            0.80,
            0.70,
            (0, 0.50, -0.11),
            M.cab,
            bevel = 0.006
        );
        fbox!(
            f,
            "worktop",
            1.0,
            0.05,
            0.79,
            (0, 0.925, -0.085),
            M.worktop,
            bevel = 0.008,
            bevel_segments = 2
        );
        fbox!(
            f,
            "hob",
            0.62,
            0.012,
            0.50,
            (0, 0.948, -0.10),
            M.hob,
            bevel = 0.004,
            bevel_segments = 2
        ); // 4 mm proud: a pan stands on it
        fextrude!(
            f,
            "hob_trim",
            &rounded_rect(0.636, 0.43, 0.012, 2),
            0.006,
            (0, 0.951, -0.10),
            M.steel,
            plane = Facing::Top,
            holes = [rounded_rect(0.616, 0.41, 0.008, 2)]
        );
        for (i, (x, z)) in [(-0.16, -0.24), (0.16, -0.24), (-0.16, 0.03), (0.16, 0.03)]
            .into_iter()
            .enumerate()
        {
            hob_ring(
                f,
                &format!("ring{i}"),
                x,
                z,
                0.9525,
                if i % 3 != 0 { 0.075 } else { 0.065 },
                false,
            );
        }
        // the touch controls: a slider and a row of marks along the front
        fbox!(f, "slider", 0.2, 0.001, 0.012, (0, 0.9545, 0.12), M.grate);
        for i in 0..4 {
            fbox!(
                f,
                format!("touch{i}"),
                0.018,
                0.001,
                0.018,
                (
                    -0.22 + 0.06 * i as f64 + (if i > 1 { 0.26 } else { 0.0 }),
                    0.9545,
                    0.12
                ),
                M.grate
            );
        }
        // the fascia with its knobs, and two deep drawers under it
        fextrude!(
            f,
            "fascia",
            &rounded_rect(0.94, 0.08, 0.008, 2),
            0.012,
            (0, 0.84, 0.245),
            M.cab,
            bevel = 0.003,
            bevel_segments = 1
        );
        for (i, x) in [0.14, -0.14, -0.42].into_iter().enumerate() {
            knob(
                f,
                &format!("knob{}", i + 1),
                x,
                0.84,
                0.252,
                0.022,
                0.03,
                None,
                true,
            );
        }
        for (i, (y, h)) in [(0.28, 0.32), (0.62, 0.30)].into_iter().enumerate() {
            shaker(
                f,
                &format!("drawer{i}"),
                0.94,
                h,
                (0, y, 0.25),
                M.cab,
                0.02,
                0.05,
                0.004,
            );
            pull!(
                f,
                format!("pull{i}"),
                (-0.15, y + h / 2.0 - 0.05),
                (0.15, y + h / 2.0 - 0.05),
                0.256,
                out = 0.016,
                w = 0.012,
                r = None,
                verts = 6
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the ring that glows is the front one at +x, within the cook's reach (it was the back one), and the knob turned the
    // outer one on its side, as a hob's knobs run: it was the far one on the other side (2026-11-03)
    let knob = knob_obj(f, "knob0", 0.42, 0.84, 0.252, 0.022, 0.03);
    let flame = hot_ring(f, "hot_ring", 0.16, 0.9545, 0.03, 0.058);
    settle_all();
    swing!(knob, "burner_on", 90, seconds = 0.5, axis = Axis::Y);
    light(flame);
    Vec::new()
}
