//! The television on its media unit, the flat-screen TV on the wall and the portable TV.

use super::common::call::rbox;
use super::common::{P, WALL, capsule, flat_screen, grille};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{Axis, P3, cosd, sind};
use artkit::parts::{join, join_all, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, jitter, loft, revolve, settle, soft, swing};

// --- the television (1x1): a flat set on its stand on a low oak media unit (the old `tv_set`) -------------
#[allow(clippy::approx_constant)] // (the neck's 0.318 is a depth, not 1/pi)
pub fn tv() -> Vec<Obj> {
    // the unit: a carcass on four turned feet, an oak top board, two doors with slim pulls, a reveal round them
    soft!(
        F,
        "unit",
        0.9,
        0.4,
        0.42,
        (0, 0.24, -0.22),
        M.tv_cabinet,
        round = 0.012,
        round_segments = 2
    );
    soft!(
        F,
        "top",
        0.91,
        0.03,
        0.43,
        (0, 0.455, -0.215),
        M.tv_oak,
        round = 0.009,
        round_segments = 2
    );
    fbox!(F, "reveal", 0.87, 0.31, 0.002, (0, 0.24, -0.0095), M.grille);
    for s in [-1, 1] {
        let sf = s as f64;
        kit::shaker!(
            F,
            &format!("door{s}"),
            0.43,
            0.3,
            (sf * 0.2195, 0.24, -0.004),
            0.012,
            M.tv_oak,
            rail = 0.045,
            recess = 0.003
        );
        kit::pull!(
            F,
            format!("pull{s}"),
            sf * 0.03,
            0.24,
            0.002,
            0.12,
            M.black_metal,
            r = 0.004,
            out = 0.006,
            upright = true
        );
        for z in [-0.4, -0.04] {
            revolve!(
                format!("foot{s}{}", float_repr(z)),
                &[
                    (0.0, 0.0),
                    (0.011, 0.0),
                    (0.013, 0.004),
                    (0.016, 0.034),
                    (0.019, 0.04),
                    (0.0, 0.04)
                ],
                at = F.at(sf * 0.42, 0, z),
                material = M.black_metal,
                segments = 10
            );
        }
    }
    // the set: a slim panel with a deeper back, on a swept neck and an oval foot
    flat_screen(
        "",
        0,
        0.82,
        -0.29,
        0.86,
        0.5,
        M.tv_frame,
        0.025,
        Some((0.6, 0.32)),
        0.0,
    );
    fsweep!(
        F,
        "neck",
        &[(0, 0.66, -0.318), (0, 0.55, -0.325), (0, 0.484, -0.31)],
        0,
        M.tv_frame,
        profile = rounded_rect(0.018, 0.06, 0.007, 1),
        up = (0, 1, 0),
        resolution = 3,
        angle = 60.0
    );
    fextrude!(
        F,
        "foot",
        &rounded_rect(0.26, 0.16, 0.07, 4),
        0.012,
        (0, 0.476, -0.3),
        M.tv_frame,
        plane = Facing::Top,
        bevel = 0.004,
        bevel_segments = 1
    );
    // a soundbar in front of it, a bowl of pebbles on two books beside, the remote
    capsule(
        "soundbar",
        -0.25,
        0.25,
        0.495,
        -0.12,
        0.05,
        0.07,
        M.soundbar,
        Some(0.01),
    );
    grille(
        "soundbar_grille",
        0,
        0.495,
        -0.0855,
        0.46,
        0.03,
        M.grille,
        Some(4),
        1,
    );
    fbox!(
        F,
        "soundbar_led",
        0.012,
        0.003,
        0.002,
        (0.2, 0.476, -0.0855),
        M.led
    );
    for (k, (m, t, turn)) in [(M.retro_bezel, 0.022, 4), (M.rec_rust, 0.018, -6)]
        .into_iter()
        .enumerate()
    {
        rbox!(
            format!("book{k}"),
            0.2,
            t,
            0.15,
            (0.33, 0.47 + 0.022 * k as f64 + t / 2.0, -0.115),
            m,
            r = 0.002,
            rot = (0, 0, turn)
        );
    }
    revolve!(
        "bowl",
        &[
            (0.0, 0.0),
            (0.05, 0.0),
            (0.065, 0.02),
            (0.06, 0.035),
            (0.05, 0.02),
            (0.0, 0.012)
        ],
        at = F.at(0.33, 0.51, -0.11),
        material = M.bowl,
        segments = 20
    );
    for (i, (x, z)) in [(0.32, -0.1), (0.345, -0.09), (0.33, -0.125), (0.31, -0.118)]
        .into_iter()
        .enumerate()
    {
        let p = fsphere!(
            F,
            format!("pebble{i}"),
            0.02,
            (x, 0.536 + 0.004 * (i % 2) as f64, z),
            M.pebble,
            scale = (1.1, 0.9, 0.55),
            segments = 10,
            rings = 6
        );
        jitter!(p, 0.003, size = 0.02, seed = i as i32);
    }
    rbox!(
        "remote",
        0.04,
        0.014,
        0.15,
        (-0.33, 0.477, -0.08),
        M.tv_frame,
        r = 0.006,
        segs = 2,
        rot = (0, 0, 14)
    );
    for k in 0..3 {
        let dz = -0.04 + 0.025 * k as f64;
        fbox!(
            F,
            format!("remote_key{k}"),
            0.02,
            0.002,
            0.012,
            (-0.33 - dz * sind(14.0), 0.4845, -0.08 + dz * cosd(14.0)),
            if k != 0 { M.steel } else { M.led_red },
            rot = (0, 0, 14)
        );
    }
    join_all("tv");
    settle!(centre = false);
    meshes()
}

// --- the flat-screen TV (2x1, wall): a thin panel on a bracket, a soundbar under it (the old tv_wall) ------
/// The panel's middle height: its foot 0.92 m up the wall, its top 1.64 m.
const WALL_TV_Y: f64 = 1.28;

pub fn tv_wall() -> Vec<Obj> {
    let y = WALL_TV_Y;
    // the wall bracket: a plate on the wall and two arms out to the set's back
    rbox!(
        "bracket",
        0.3,
        0.2,
        0.008,
        (0, y - 0.02, WALL + 0.004),
        M.black_metal,
        r = 0.002
    );
    for s in [-1, 1] {
        rbox!(
            format!("bracket_arm{s}"),
            0.025,
            0.36,
            0.012,
            (s as f64 * 0.1, y - 0.02, WALL + 0.012),
            M.black_metal,
            r = 0.003
        );
    }
    rbox!(
        "back",
        0.8,
        0.45,
        0.022,
        (0, y - 0.02, WALL + 0.019),
        M.wall_frame,
        r = 0.008,
        segs = 2
    );
    soft!(
        F,
        "panel",
        1.24,
        0.72,
        0.022,
        (0, y, WALL + 0.041),
        M.wall_frame,
        round = 0.005,
        round_segments = 2
    );
    fbox!(
        F,
        "screen",
        1.215,
        0.695,
        0.002,
        (0, y, WALL + 0.0525),
        M.screen
    );
    fbox!(
        F,
        "logo",
        0.05,
        0.008,
        0.002,
        (0, y - 0.352, WALL + 0.0525),
        M.steel
    );
    fbox!(
        F,
        "led",
        0.008,
        0.003,
        0.002,
        (0.5, y - 0.352, WALL + 0.0525),
        M.led_red
    );
    // the cable down the wall to the soundbar, in a slim cover
    rbox!(
        "cable_cover",
        0.03,
        0.12,
        0.012,
        (0.22, 0.88, WALL + 0.006),
        M.wall_frame,
        r = 0.003
    );
    // the soundbar on its own two brackets, its cloth front ribbed
    for s in [-1, 1] {
        rbox!(
            format!("bar_bracket{s}"),
            0.03,
            0.05,
            0.03,
            (s as f64 * 0.3, 0.8, WALL + 0.015),
            M.black_metal,
            r = 0.003
        );
    }
    capsule(
        "soundbar",
        -0.45,
        0.45,
        0.8,
        WALL + 0.065,
        0.06,
        0.07,
        M.wall_bar,
        Some(0.012),
    );
    grille(
        "soundbar_grille",
        0,
        0.8,
        WALL + 0.0995,
        0.84,
        0.036,
        M.grille,
        Some(5),
        1,
    );
    fbox!(
        F,
        "soundbar_led",
        0.01,
        0.003,
        0.0015,
        (0.38, 0.775, WALL + 0.1),
        M.led
    );
    join_all("tvWall");
    settle!(centre = false, floor = false);
    meshes()
}

// --- the portable TV (1x1): a dark plastic monobloc case, its CRT behind a thin rounded mask, on two pads -----
// The set of the reference photo: no cabinet and no legs, the controls and the stub aerial on the top right, the
// back on the wall face.
/// The case.
const PT_W: f64 = 0.86;
const PT_H: f64 = 0.66;
const PT_D: f64 = 0.40;
/// Its back, on the wall face.
const PT_BACK: f64 = WALL;
/// Its front.
const PT_FRONT: f64 = PT_BACK + PT_D;
/// Its middle depth.
const PT_MID_Z: f64 = (PT_BACK + PT_FRONT) / 2.0;
/// The base band's thickness, its top at `PT_BASE_H`.
const PT_BASE_T: f64 = 0.05;
/// The base band's top, where the case's bottom sits.
const PT_BASE_H: f64 = 0.095;
/// Its middle height: it stands on the base band and two pads.
const PT_MID: f64 = PT_BASE_H + PT_H / 2.0;
/// Its top.
const PT_TOP: f64 = PT_BASE_H + PT_H;
/// The screen's middle height: a chin below it, a narrow brow above.
const PT_SC_Y: f64 = 0.44;
/// The glass, and the mask round it.
const PT_SC_W: f64 = 0.68;
const PT_SC_H: f64 = 0.50;
const PT_BEZ: f64 = 0.02;
/// The aerial's foot, on the case's top right.
const PT_AERIAL: P = (0.33, PT_TOP, PT_BACK + 0.13);

pub fn tv_retro() -> Vec<Obj> {
    let still = new_parts(|| {
        // the shell: one soft case, its corners well rounded, on an inset base band
        soft!(
            F,
            "case",
            PT_W,
            PT_H,
            PT_D,
            (0, PT_MID, PT_MID_Z),
            M.retro_shell,
            round = 0.05,
            round_segments = 3
        );
        soft!(
            F,
            "base",
            PT_W - 0.06,
            PT_BASE_T,
            PT_D - 0.05,
            (0, PT_BASE_H - PT_BASE_T / 2.0, PT_MID_Z),
            M.retro_shell,
            round = 0.014,
            round_segments = 2
        );
        // two pads at the front, a rail at the back
        for s in [-1, 1] {
            soft!(
                F,
                format!("pad{s}"),
                0.11,
                0.045,
                0.08,
                (s as f64 * 0.28, 0.0225, PT_FRONT - 0.09),
                M.retro_bezel,
                round = 0.012,
                round_segments = 2
            );
        }
        soft!(
            F,
            "rail",
            0.66,
            0.045,
            0.06,
            (0, 0.0225, PT_BACK + 0.09),
            M.retro_bezel,
            round = 0.012,
            round_segments = 2
        );
        // the screen: a thin rounded mask proud of the case, the tube's glass domed out of it
        let hole = rounded_rect(PT_SC_W, PT_SC_H, 0.05, 4);
        fextrude!(
            F,
            "mask",
            &rounded_rect(PT_SC_W + 2.0 * PT_BEZ, PT_SC_H + 2.0 * PT_BEZ, 0.055, 4),
            0.028,
            (0, PT_SC_Y, PT_FRONT + 0.012),
            M.retro_bezel,
            holes = vec![hole.clone()],
            bevel = 0.006,
            bevel_segments = 2
        );
        let glass = PT_FRONT - 0.005;
        // the first two sections are the same size: a lip at the mask, so the glass's edge doesn't taper into it
        let tube: Vec<Vec<P3>> = [
            (0.0, 1.015),
            (0.006, 1.015),
            (0.012, 1.0),
            (0.017, 0.94),
            (0.021, 0.80),
            (0.024, 0.45),
        ]
        .iter()
        .map(|&(t, k)| {
            hole.iter()
                .map(|&(u, v)| (-u * k, -(glass + t), PT_SC_Y + v * k).into())
                .collect()
        })
        .collect();
        loft!("tube", &tube, material = M.crt);
        // the chin: a badge and the standby light
        fbox!(
            F,
            "badge",
            0.07,
            0.012,
            0.002,
            (0, 0.132, PT_FRONT + 0.0005),
            M.steel
        );
        fbox!(
            F,
            "led",
            0.006,
            0.002,
            0.0015,
            (0.29, 0.132, PT_FRONT + 0.0005),
            M.led_red
        );
        // the controls on the right side: a turned knob over two buttons, on a pad
        rbox!(
            "ctl_pad",
            0.012,
            0.13,
            0.11,
            (PT_W / 2.0 - 0.004, 0.58, -0.15),
            M.retro_bezel,
            r = 0.006,
            segs = 2
        );
        fcyl!(
            F,
            "knob",
            0.024,
            0.03,
            (PT_W / 2.0 + 0.008, 0.615, -0.155),
            M.retro_bezel,
            rot = (0, 90, 0),
            verts = 16,
            bevel = 0.004
        );
        fbox!(
            F,
            "knob_mark",
            0.003,
            0.03,
            0.004,
            (PT_W / 2.0 + 0.024, 0.615, -0.155),
            M.steel
        );
        for (k, y) in [0.555, 0.515].into_iter().enumerate() {
            fcyl!(
                F,
                format!("button{k}"),
                0.012,
                0.014,
                (PT_W / 2.0 + 0.004, y, -0.15),
                M.retro_bezel,
                rot = (0, 90, 0),
                verts = 12
            );
        }
        // the back: a cover with vents
        soft!(
            F,
            "back",
            0.70,
            0.5,
            0.02,
            (0, 0.42, PT_BACK + 0.012),
            M.retro_bezel,
            round = 0.01,
            round_segments = 2
        );
        for k in 0..5 {
            fbox!(
                F,
                format!("vent{k}"),
                0.5,
                0.012,
                0.004,
                (0, 0.3 + 0.055 * k as f64, PT_BACK + 0.021),
                M.grille
            );
        }
        fsphere!(
            F,
            "aerial_foot",
            0.022,
            PT_AERIAL,
            M.retro_bezel,
            scale = (1, 0.6, 1),
            segments = 14,
            rings = 8
        );
    });
    join(&still, "tvRetro", Some(P3::ZERO));
    // the aerial: a telescopic stub lying back along the case's top, its own part, lifted when watched
    let aerial = new_parts(|| {
        // a thick foot section, then the thin rod: `scale` is two rings a point
        let knee = (PT_AERIAL.0 + 0.014, PT_AERIAL.1 + 0.008, PT_AERIAL.2 - 0.05);
        let tip = (PT_AERIAL.0 + 0.03, PT_AERIAL.1 + 0.022, PT_BACK + 0.025);
        fsweep!(
            F,
            "aerial_rod",
            &[PT_AERIAL, knee, tip],
            0.006,
            M.steel,
            verts = 8,
            scale = [1.7, 1.7, 0.9, 0.9, 0.7, 0.7]
        );
        fsphere!(
            F,
            "aerial_ball",
            0.009,
            tip,
            M.steel,
            segments = 10,
            rings = 6
        );
    });
    let aerial = join(
        &aerial,
        "aerial",
        Some(F.at(PT_AERIAL.0, PT_AERIAL.1, PT_AERIAL.2)),
    );
    settle!(centre = false);
    // about X: its tip, at the back (+Y), rises
    swing!(aerial, "antenna", 35.0, seconds = 0.9, axis = Axis::X);
    meshes()
}
