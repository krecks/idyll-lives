//! The wall cabinets (K5): two shaker doors, glazed doors, a corner cabinet with open shelves; their doors swing open
//! while clean things are put away in them. A wall thing sits on the wall face (z = WALL) at its real height.

use super::common::{pull, shaker};
use super::storage::{WALL, case, shelf};
use super::turned::{bowl, jug, lathe, plates};
use super::{M, WALL_DOOR_SECONDS};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::kit;
use artkit::motion::Ease;
use artkit::parts::{join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsphere, settle, swing};

/// A strip light under a wall cabinet, its top at `y`: a slim housing and its diffuser.
fn strip_light(f: &GameFrame, w: f64, y: f64, z: f64) {
    fbox!(
        f,
        "light_housing",
        w,
        0.008,
        0.036,
        (0, y - 0.004, z),
        M.cab,
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        f,
        "light",
        w - 0.02,
        0.006,
        0.024,
        (0, y - 0.009, z),
        M.bulb,
        bevel = 0.002,
        bevel_segments = 1
    );
}

// --- the wall cabinet: 1x1, two doors with pulls under it and a strip light; plates, bowls and mugs
// inside, the doors swinging open while a resident puts the clean things away (`doors_open`)
pub fn wall_cabinet() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        case(
            f, 0.98, 0.72, 0.34, 0, 1.81, WALL, M.cab, None, 0.02, "case",
        );
        shelf(f, "shelf", 0.94, 1.81, WALL + 0.02, 0.3, Some(M.cab), None);
        plates(
            f,
            "plates",
            -0.24,
            1.47,
            WALL + 0.18,
            0.1,
            6,
            M.porcelain,
            0.015,
            12,
        );
        for i in 0..3 {
            bowl(
                f,
                format!("bowl{i}"),
                0.2,
                1.47 + 0.02 * i as f64,
                WALL + 0.18,
                0.075 - 0.007 * i as f64,
                0.06 - 0.005 * i as f64,
                M.porcelain,
                10,
            );
        }
        for (i, x) in [-0.3, -0.16, 0.12, 0.26].into_iter().enumerate() {
            kit::mug!(
                f,
                &format!("mug{i}"),
                x,
                1.82,
                WALL + 0.17,
                0.038,
                0.09,
                M.porcelain,
                (-60 + 40 * i as i32) as f64,
                segments = 10
            );
        }
        strip_light(f, 0.9, 1.45, WALL + 0.07);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let mut doors = Vec::new();
    for s in [-1, 1] {
        let parts = new_parts(|| {
            shaker(
                f,
                &format!("door{s}"),
                0.482,
                0.7,
                s as f64 * 0.244,
                1.81,
                WALL + 0.34,
                M.cab,
                0.06,
                0.02,
                None,
            );
            // on the doors, near their bottom
            pull(
                f,
                format!("door{s}_pull"),
                s as f64 * 0.034,
                1.56,
                WALL + 0.36,
                0.14,
                None,
                true,
                0.019,
                0.005,
                0.0,
            );
        });
        doors.push(join(
            &parts,
            &format!("door{s}"),
            Some(f.at(s as f64 * 0.487, 1.81, WALL + 0.34)),
        ));
    }
    settle!(centre = false, floor = false);
    swing!(
        doors[0],
        "doors_open",
        100,
        seconds = WALL_DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(
        doors[1],
        "doors_open",
        -100,
        seconds = WALL_DOOR_SECONDS,
        ease = Ease::BackOut
    );
    meshes()
}

/// A drinking glass standing on `y`: a thick base, thin walls flaring a little to the rim, hollow.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn tumbler(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64, h: f64, verts: usize) {
    lathe(
        f,
        name,
        &[
            (0.0, 0.0),
            (0.88 * r, 0.0),
            (0.9 * r, 0.12 * h),
            (r, h),
            (0.92 * r, h),
            (0.8 * r, 0.14 * h),
            (0.0, 0.14 * h),
        ],
        x,
        y,
        z,
        M.glass,
        verts,
    );
}

/// A wine glass standing on `y`: a foot, a stem and a bowl.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn wine_glass(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64, h: f64, verts: usize) {
    lathe(
        f,
        name,
        &[
            (0.0, 0.0),
            (0.75 * r, 0.0),
            (0.7 * r, 0.03 * h),
            (0.1 * r, 0.06 * h),
            (0.09 * r, 0.42 * h),
            (0.6 * r, 0.52 * h),
            (r, 0.72 * h),
            (0.86 * r, h),
            (0.8 * r, h),
            (0.9 * r, 0.72 * h),
            (0.0, 0.5 * h),
        ],
        x,
        y,
        z,
        M.glass,
        verts,
    );
}

// --- the glazed wall cabinet: 1x1, glass doors showing glasses and plates; the right one opens while
// clean things are put away (`door_open`) ---------------------------------------------------------------
pub fn wall_cabinet_glass() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        case(
            f,
            0.96,
            0.72,
            0.36,
            0,
            1.81,
            WALL,
            M.cab,
            Some(M.back),
            0.02,
            "case",
        );
        for (i, y) in [1.47, 1.84].into_iter().enumerate() {
            shelf(f, &format!("shelf{i}"), 0.9, y, WALL, 0.32, None, None);
        }
        for (i, x) in [-0.37, -0.28].into_iter().enumerate() {
            tumbler(
                f,
                &format!("glass{i}"),
                x,
                1.48,
                WALL + 0.12 + 0.06 * i as f64,
                0.033,
                0.11,
                10,
            );
        }
        for (i, x) in [-0.17, -0.06].into_iter().enumerate() {
            wine_glass(
                f,
                &format!("wine{i}"),
                x,
                1.48,
                WALL + 0.15,
                0.037,
                0.17,
                10,
            );
        }
        jug(
            f,
            "carafe",
            0.12,
            1.48,
            WALL + 0.16,
            0.05,
            0.2,
            M.glass,
            -90.0,
            12,
        );
        plates(
            f,
            "plates",
            0.24,
            1.85,
            WALL + 0.17,
            0.1,
            5,
            M.porcelain,
            0.016,
            14,
        );
        for (i, x) in [-0.3, -0.12].into_iter().enumerate() {
            bowl(
                f,
                format!("bowl{i}"),
                x,
                1.85,
                WALL + 0.17,
                0.07,
                0.06,
                M.porcelain,
                12,
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let mut doors = Vec::new();
    for s in [-1, 1] {
        let cx = s as f64 * 0.24;
        let parts = new_parts(|| {
            // a frame round the glass, its inner edge eased, a glazing bar across the middle
            fextrude!(
                f,
                format!("door{s}_frame"),
                &rounded_rect(0.46, 0.7, 0.004, 1),
                0.02,
                (cx, 1.81, WALL + 0.37),
                M.cab,
                holes = vec![rounded_rect(0.37, 0.61, 0.003, 1)],
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("door{s}_bar"),
                0.37,
                0.022,
                0.014,
                (cx, 1.81, WALL + 0.373),
                M.cab,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("door{s}_glass"),
                0.38,
                0.62,
                0.006,
                (cx, 1.81, WALL + 0.365),
                M.glass
            );
            pull(
                f,
                format!("door{s}_pull"),
                s as f64 * 0.04,
                1.6,
                WALL + 0.38,
                0.14,
                None,
                true,
                0.026,
                0.005,
                0.0,
            );
        });
        doors.push(join(
            &parts,
            &format!("door{s}"),
            Some(f.at(s as f64 * 0.48, 1.81, WALL + 0.36)),
        ));
    }
    settle!(centre = false, floor = false);
    // only the right door, in front of the plates, swings open: each glazed door is two draws (frame, glass)
    swing!(
        doors[1],
        "door_open",
        -100,
        seconds = WALL_DOOR_SECONDS,
        ease = Ease::BackOut
    );
    meshes()
}

// --- the corner wall cabinet: 1x1, a door on one side and open shelves on the other -------------------
pub fn wall_cabinet_corner() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        case(
            f, 0.6, 0.72, 0.36, -0.18, 1.81, WALL, M.cab, None, 0.02, "case",
        );
        fbox!(
            f,
            "shelf",
            0.56,
            0.02,
            0.32,
            (-0.18, 1.81, WALL + 0.18),
            M.cab,
            bevel = 0.003
        );
        plates(
            f,
            "plates",
            -0.3,
            1.47,
            WALL + 0.19,
            0.1,
            5,
            M.porcelain,
            0.015,
            14,
        );
        for (i, x) in [-0.36, -0.22, -0.08].into_iter().enumerate() {
            kit::mug!(
                f,
                &format!("mug{i}"),
                x,
                1.82,
                WALL + 0.18,
                0.038,
                0.09,
                M.porcelain,
                (-40 + 50 * i as i32) as f64,
                segments = 12
            );
        }
        // the open end: three shelves rounded at their outer front corner, between the case and a side panel
        let shelf = [
            (0.12, -0.17),
            (0.48, -0.17),
            (0.48, 0.0),
            (0.29, 0.17),
            (0.12, 0.17),
        ];
        let round = smooth_outline(
            &[(0.48, -0.03), (0.48, 0.1), (0.36, 0.17), (0.22, 0.17)],
            2,
            false,
        );
        let mut outline: Vec<(f64, f64)> = shelf[..2].to_vec();
        outline.extend_from_slice(&round[1..round.len() - 1]);
        outline.extend_from_slice(&shelf[3..]);
        for (i, y) in [1.46, 1.81, 2.155].into_iter().enumerate() {
            fextrude!(
                f,
                format!("end_shelf{i}"),
                &outline,
                0.025,
                (0, y, WALL + 0.17),
                M.oak,
                plane = Facing::Top,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "end_panel",
            0.03,
            0.72,
            0.34,
            (0.46, 1.81, WALL + 0.17),
            M.oak,
            bevel = 0.006
        );
        lathe(
            f,
            "pot",
            &[
                (0.0, 0.0),
                (0.042, 0.0),
                (0.05, 0.11),
                (0.054, 0.12),
                (0.054, 0.13),
                (0.046, 0.13),
                (0.043, 0.115),
                (0.0, 0.115),
            ],
            0.26,
            1.4725,
            WALL + 0.16,
            M.sage,
            12,
        );
        for (i, (dx, dz, a)) in [
            (0.0, 0.0, 0),
            (0.02, 0.03, 70),
            (-0.025, 0.02, 150),
            (0.01, -0.03, 230),
            (-0.015, -0.015, 300),
        ]
        .into_iter()
        .enumerate()
        {
            fsphere!(
                f,
                format!("leaf{i}"),
                0.04,
                (0.26 + dx, 1.62 + 0.012 * (i % 2) as f64, WALL + 0.16 + dz),
                M.leaf,
                scale = (1.0, 0.45, 0.35),
                rot = (0, 25, a),
                segments = 8,
                rings = 4
            );
        }
        bowl(
            f,
            "bowl",
            0.3,
            1.8225,
            WALL + 0.14,
            0.07,
            0.05,
            M.porcelain,
            12,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let parts = new_parts(|| {
        shaker(
            f,
            "door",
            0.58,
            0.7,
            -0.18,
            1.81,
            WALL + 0.36,
            M.cab,
            0.06,
            0.02,
            None,
        );
        pull(
            f,
            "door_pull",
            0.06,
            1.6,
            WALL + 0.38,
            0.14,
            None,
            true,
            0.026,
            0.005,
            0.0,
        );
    });
    let door = join(&parts, "door", Some(f.at(-0.47, 1.81, WALL + 0.36)));
    settle!(centre = false, floor = false);
    swing!(
        door,
        "door_open",
        100,
        seconds = WALL_DOOR_SECONDS,
        ease = Ease::BackOut
    );
    meshes()
}
