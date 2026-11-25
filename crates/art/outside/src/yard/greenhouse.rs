//! The greenhouse (2x3): a white frame on a dwarf wall, glazed; staging and tomatoes inside; its door and its roof
//! vent open (`door_open`). Built in the script's order, its one long build split where its comments part it.

use super::common::{F, M, blob, pot, prism, rod};
use artkit::far::{is_low, low};
use artkit::kit;
use artkit::math::{Axis, atan2, degrees};
use artkit::motion::{Ease, Swing, swing};
use artkit::parts::numbered;
use artkit::parts::{hinged, join, join_all, new_parts};
use artkit::place::N;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsphere, revolve, settle};

const GHW: f64 = 1.8; // width, depth
const GHD: f64 = 2.8;
const WALL_TOP: f64 = 0.42;
const GH_EAVE: f64 = 1.8;
const GH_RIDGE: f64 = 2.36;
const GH_DOOR: f64 = 0.62;
const VENT_Z: (f64, f64) = (-0.75, 0.15); // the roof vent, depths (left slope)
const HX: f64 = GHW / 2.0;
const HZ: f64 = GHD / 2.0;

fn slope() -> f64 {
    degrees(atan2(GH_RIDGE - GH_EAVE, HX))
}

fn run() -> f64 {
    artkit::py::hypot(HX, GH_RIDGE - GH_EAVE)
}

/// A point down the slope `t` of the way from the ridge to the eave (x on the +x side, height).
fn roof_at(t: f64) -> (f64, f64) {
    (HX * t, GH_RIDGE - (GH_RIDGE - GH_EAVE) * t)
}

/// The dwarf wall, open at the door; a gravel floor and a paved path.
fn wall_and_floor() {
    let (hx, hz) = (HX, HZ);
    let t = 0.1;
    fbox!(
        F,
        "wall_back",
        GHW,
        WALL_TOP,
        t,
        (0, WALL_TOP / 2.0, -hz + t / 2.0),
        M.gh_wall,
        bevel = low(0.01, 0.0)
    );
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            numbered("wall_side"),
            t,
            WALL_TOP,
            GHD - 2.0 * t,
            (s * (hx - t / 2.0), WALL_TOP / 2.0, 0),
            M.gh_wall,
            bevel = low(0.01, 0.0)
        );
        let w = (GHW - GH_DOOR) / 2.0;
        fbox!(
            F,
            numbered("wall_front"),
            w,
            WALL_TOP,
            t,
            (s * (GH_DOOR / 2.0 + w / 2.0), WALL_TOP / 2.0, hz - t / 2.0),
            M.gh_wall,
            bevel = low(0.01, 0.0)
        );
    }
    if !is_low() {
        // the brick courses' mortar lines, and a coping
        for k in 1..5 {
            let y = k as f64 * WALL_TOP / 5.0;
            fbox!(
                F,
                numbered("course"),
                GHW + 0.002,
                0.006,
                t + 0.002,
                (0, y, -hz + t / 2.0),
                M.mortar
            );
            for s in [-1.0, 1.0] {
                fbox!(
                    F,
                    numbered("course"),
                    t + 0.002,
                    0.006,
                    GHD - 2.0 * t,
                    (s * (hx - t / 2.0), y, 0),
                    M.mortar
                );
                let w = (GHW - GH_DOOR) / 2.0;
                fbox!(
                    F,
                    numbered("course"),
                    w + 0.002,
                    0.006,
                    t + 0.002,
                    (s * (GH_DOOR / 2.0 + w / 2.0), y, hz - t / 2.0),
                    M.mortar
                );
            }
        }
    }
    fbox!(
        F,
        "gravel",
        GHW - 2.0 * t,
        0.02,
        GHD - t,
        (0, 0.01, 0.05),
        M.gravel
    );
    fbox!(F, "path", 0.5, 0.025, GHD - t, (0, 0.0125, 0.05), M.paving);
    if !is_low() {
        // the path's slabs
        for k in 1..6 {
            fbox!(
                F,
                numbered("joint"),
                0.5,
                0.003,
                0.008,
                (0, 0.025, -hz + t + k as f64 * (GHD - t) / 6.0),
                M.gravel
            );
        }
    }
}

/// The frame: sills, corner posts, glazing bars, eaves, rafters, the ridge; the vent's frame on the left slope (the
/// vent's glass is its own part).
fn frame() {
    let (hx, hz) = (HX, HZ);
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            numbered("sill"),
            0.06,
            0.04,
            GHD,
            (s * (hx - 0.03), WALL_TOP + 0.02, 0),
            M.gh_frame
        );
        fbox!(
            F,
            numbered("eave"),
            0.06,
            0.05,
            GHD,
            (s * (hx - 0.03), GH_EAVE, 0),
            M.gh_frame
        );
    }
    fbox!(
        F,
        "sill_back",
        GHW,
        0.04,
        0.06,
        (0, WALL_TOP + 0.02, -hz + 0.03),
        M.gh_frame
    );
    for z in [-hz + 0.025, hz - 0.025] {
        fbox!(
            F,
            numbered("eave_end"),
            GHW,
            0.04,
            0.05,
            (0, GH_EAVE, z),
            M.gh_frame
        );
    }
    let bars: Vec<f64> = (0..5)
        .map(|k| -hz + 0.025 + k as f64 * (GHD - 0.05) / 4.0)
        .collect();
    for &z in &bars {
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                numbered("bar"),
                0.04,
                GH_EAVE - WALL_TOP,
                0.04,
                (s * (hx - 0.025), (GH_EAVE + WALL_TOP) / 2.0, z),
                M.gh_frame
            );
            kit::board(
                &F,
                &numbered("rafter"),
                (s * 0.03, GH_RIDGE, z),
                (s * (hx - 0.02), GH_EAVE + 0.01, z),
                0.04,
                0.04,
                M.gh_frame,
                (0, 0, 1),
                0.004,
            );
        }
    }
    fbox!(
        F,
        "ridge",
        0.06,
        0.06,
        GHD + 0.04,
        (0, GH_RIDGE, 0),
        M.gh_frame
    );
    for z in [-hz + 0.01, hz - 0.01] {
        // a finial at each end of the ridge
        crate::eased(revolve!(
            numbered("finial"),
            &[
                (0.0, 0.0),
                (0.018, 0.0),
                (0.012, 0.015),
                (0.016, 0.024),
                (0.0, 0.036)
            ],
            at = F.at(0, GH_RIDGE + 0.03, z),
            material = M.gh_frame,
            segments = low(8, 4)
        ));
    }
    if !is_low() {
        for z in (0..4).map(|k| -hz + 0.025 + (GHD - 0.05) * k as f64 / 4.0) {
            for s in [-1.0, 1.0] {
                // a glazing bar across each side's panes
                fbox!(
                    F,
                    numbered("bar_h"),
                    0.03,
                    0.025,
                    (GHD - 0.05) / 4.0 - 0.04,
                    (s * (hx - 0.022), 1.1, z + (GHD - 0.05) / 8.0),
                    M.gh_frame
                );
            }
        }
    }
    for x in [-0.45, 0.45] {
        // the back's and the front's glazing bars
        fbox!(
            F,
            numbered("bar_back"),
            0.04,
            GH_EAVE - WALL_TOP,
            0.04,
            (x, (GH_EAVE + WALL_TOP) / 2.0, -hz + 0.025),
            M.gh_frame
        );
    }
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            numbered("door_post"),
            0.05,
            GH_EAVE,
            0.05,
            (s * (GH_DOOR / 2.0 + 0.025), GH_EAVE / 2.0, hz - 0.025),
            M.gh_frame
        );
        fbox!(
            F,
            numbered("sill_front"),
            (GHW - GH_DOOR) / 2.0,
            0.04,
            0.06,
            (s * (GHW + GH_DOOR) / 4.0, WALL_TOP + 0.02, hz - 0.03),
            M.gh_frame
        );
    }
    fbox!(
        F,
        "transom",
        GH_DOOR,
        0.05,
        0.05,
        (0, GH_EAVE - 0.04, hz - 0.025),
        M.gh_frame
    );
    // the vent's frame on the left slope (the vent's glass is its own part)
    let (vz0, vz1) = VENT_Z;
    for z in [vz0, vz1] {
        kit::board(
            &F,
            &numbered("vent_rafter"),
            (-0.03, GH_RIDGE, z),
            (-(hx - 0.02), GH_EAVE + 0.01, z),
            0.03,
            0.04,
            M.gh_frame,
            (0, 0, 1),
            0.004,
        );
    }
}

/// The glass: sides above the wall, back, front either side of the door, gables, the roof round the vent.
fn glass() {
    let (hx, hz) = (HX, HZ);
    let (vz0, vz1) = VENT_Z;
    let gh = GH_EAVE - WALL_TOP - 0.04;
    let gy = WALL_TOP + 0.04 + gh / 2.0;
    for s in [-1.0, 1.0] {
        fbox!(
            F,
            numbered("glass_side"),
            0.01,
            gh,
            GHD - 0.06,
            (s * (hx - 0.025), gy, 0),
            M.glass
        );
        fbox!(
            F,
            numbered("glass_front"),
            (GHW - GH_DOOR) / 2.0 - 0.08,
            gh,
            0.01,
            (s * (GHW + GH_DOOR) / 4.0, gy, hz - 0.025),
            M.glass
        );
    }
    fbox!(
        F,
        "glass_back",
        GHW - 0.06,
        gh,
        0.01,
        (0, gy, -hz + 0.025),
        M.glass
    );
    for z in [-hz + 0.025, hz - 0.025] {
        prism(
            &numbered("glass_gable"),
            &[
                (N::F(-hx + 0.03), GH_EAVE + 0.02),
                (N::F(hx - 0.03), GH_EAVE + 0.02),
                (N::I(0), GH_RIDGE - 0.03),
            ],
            z - 0.005,
            z + 0.005,
            M.glass,
        );
    }
    // the roof: the right slope whole; the left round its vent
    let (mx, my) = roof_at(0.5);
    let (run, slope) = (run(), slope());
    fbox!(
        F,
        "glass_roof_r",
        run,
        0.01,
        GHD - 0.04,
        (mx, my + 0.025, 0),
        M.glass,
        rot = (0.0, -slope, 0.0)
    );
    for (z0, z1) in [(-hz + 0.02, vz0 - 0.015), (vz1 + 0.015, hz - 0.02)] {
        fbox!(
            F,
            numbered("glass_roof_l"),
            run,
            0.01,
            z1 - z0,
            (-mx, my + 0.025, (z0 + z1) / 2.0),
            M.glass,
            rot = (0.0, slope, 0.0)
        );
    }
}

/// The staging: a slatted bench down each side, pots of seedlings; tomatoes in grow bags at the back.
fn staging() {
    for s in [-1, 1] {
        let x = s as f64 * 0.56;
        for k in 0..low(5, 2) {
            fbox!(
                F,
                numbered("slat"),
                if !is_low() { 0.09 } else { 0.5 },
                0.025,
                1.9,
                (
                    if !is_low() {
                        x - 0.2 + k as f64 * 0.1
                    } else {
                        x
                    },
                    0.78,
                    0.25
                ),
                M.staging
            );
        }
        for z in [-0.65, 1.15] {
            for e in [-1.0, 1.0] {
                fbox!(
                    F,
                    numbered("leg"),
                    0.04,
                    0.76,
                    0.04,
                    (x + e * 0.2, 0.38, z),
                    M.staging
                );
            }
        }
        for (k, z) in [-0.5, -0.1, 0.3, 0.7, 1.05].into_iter().enumerate() {
            let dx = if k % 2 == 1 { 0.08 } else { -0.08 };
            pot(x + dx, 0.79, z, 0.07, 0.11, None);
            blob(
                x + dx,
                0.88,
                z,
                0.08,
                M.green,
                (40 + k as i32 + 5 * s) as f64,
                (1.0, 1.0, 0.9),
                0.4,
            );
        }
    }
    for (k, x) in [-0.45, 0.0, 0.45].into_iter().enumerate() {
        // tomato plants on canes along the back
        fbox!(
            F,
            numbered("grow_bag"),
            0.36,
            0.1,
            0.24,
            (x, 0.07, -1.18),
            M.compost,
            bevel = low(0.03, 0.0)
        );
        rod(
            &numbered("cane"),
            (x, 0.1, -1.18),
            (x, 1.55, -1.2),
            0.008,
            M.haft,
            4,
        );
        for j in 0..low(5, 3) {
            blob(
                x + if j % 2 == 1 { 0.05 } else { -0.05 },
                0.15 + j as f64 * 0.27,
                -1.16,
                0.12 - 0.01 * j as f64,
                M.green,
                (60 + 7 * k + j) as f64,
                (1.0, 0.7, 1.4),
                0.45,
            );
        }
        for j in 0..4 {
            fsphere!(
                F,
                numbered("tomato"),
                0.032,
                (
                    x + 0.08 * if j % 2 == 1 { 1.0 } else { -1.0 },
                    0.5 + j as f64 * 0.22,
                    -1.1
                ),
                M.tomato,
                segments = low(10, 5),
                rings = low(6, 3)
            );
        }
    }
}

pub fn greenhouse() -> Vec<Obj> {
    let hz = HZ;
    let (run, slope) = (run(), slope());
    wall_and_floor();
    frame();
    glass();
    staging();
    if !is_low() {
        join(
            &meshes(),
            "greenhouse",
            Some(artkit::math::P3::from((0, 0, 0))),
        );
    }
    // The door: a glass leaf hinged on the left post, swinging out; the vent: a glass panel hinged at the ridge,
    // lifting. Their edges are white glass (one draw each: see the notes); `door_open` opens both.
    let door = new_parts(|| {
        let dz = hz - 0.025;
        let dh = GH_EAVE - 0.07;
        fbox!(
            F,
            "door_glass",
            GH_DOOR - 0.04,
            dh - 0.06,
            0.01,
            (0, 0.03 + dh / 2.0, dz),
            M.glass
        );
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                numbered("door_stile"),
                0.03,
                dh,
                0.02,
                (s * (GH_DOOR / 2.0 - 0.035), 0.03 + dh / 2.0, dz),
                M.glass_bar
            );
        }
        for y in [0.045, 0.03 + dh - 0.015, 0.95] {
            fbox!(
                F,
                numbered("door_rail"),
                GH_DOOR - 0.07,
                0.03,
                0.02,
                (0, y, dz),
                M.glass_bar
            );
        }
        fbox!(
            F,
            "door_kick",
            GH_DOOR - 0.07,
            0.24,
            0.014,
            (0, 0.18, dz),
            M.glass_bar
        );
    });
    let (vz0, vz1) = VENT_Z;
    let vent = new_parts(|| {
        let (mx, my) = roof_at(0.5);
        fbox!(
            F,
            "vent_glass",
            run - 0.04,
            0.012,
            vz1 - vz0 - 0.02,
            (-mx, my + 0.032, (vz0 + vz1) / 2.0),
            M.glass,
            rot = (0.0, slope, 0.0)
        );
        for z in [vz0 + 0.02, vz1 - 0.02] {
            fbox!(
                F,
                numbered("vent_edge"),
                run - 0.04,
                0.02,
                0.025,
                (-mx, my + 0.04, z),
                M.glass_bar,
                rot = (0.0, slope, 0.0)
            );
        }
    });
    if is_low() {
        return settle!(&[join_all("greenhouse")], centre = false);
    }
    let door = hinged(
        join(&door, "door", None),
        F.at(-GH_DOOR / 2.0 + 0.02, 1.0, hz - 0.025),
    );
    let vent = hinged(join(&vent, "vent", None), F.at(-0.02, GH_RIDGE + 0.01, 0));
    let objs = settle!(centre = false);
    swing(
        door,
        "door_open",
        70.0,
        Swing {
            seconds: 0.9,
            ease: Ease::BackOut,
            ..Swing::default()
        },
    );
    swing(
        vent,
        "door_open",
        -25.0,
        Swing {
            seconds: 0.9,
            axis: Axis::Y,
            ..Swing::default()
        },
    );
    objs
}
