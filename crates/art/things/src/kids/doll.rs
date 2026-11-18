//! KD2: the doll house.

use super::common::{place, plane, rect, slab, still};
use super::kit::shaker;
use super::{F, M};
use artkit::materials::Mat;
use artkit::math::{atan2, cos, degrees, radians, sin, tan};
use artkit::parts::{hinged, join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fcyl, fextrude, fsphere, soft, sphere, swing, torus};

const ROOF_RISE: f64 = 0.18;

/// A window on a front at depth `z`: a moulded frame round its glass, glazing bars, a sill, and shutters.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn window(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    frame: Mat,
    glass: Mat,
    shutters: Option<Mat>,
) {
    fextrude!(
        F,
        format!("{name}_frame"),
        &rect(-w / 2.0, -h / 2.0, w / 2.0, h / 2.0),
        0.008,
        (x, y, z),
        frame,
        holes = vec![rect(
            -w / 2.0 + 0.012,
            -h / 2.0 + 0.012,
            w / 2.0 - 0.012,
            h / 2.0 - 0.012
        )],
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        F,
        format!("{name}_glass"),
        w - 0.022,
        h - 0.022,
        0.004,
        (x, y, z - 0.002),
        glass
    );
    fbox!(
        F,
        format!("{name}_bar_v"),
        0.007,
        h - 0.022,
        0.004,
        (x, y, z + 0.001),
        frame
    );
    fbox!(
        F,
        format!("{name}_bar_h"),
        w - 0.022,
        0.007,
        0.004,
        (x, y, z + 0.001),
        frame
    );
    fbox!(
        F,
        format!("{name}_sill"),
        w + 0.02,
        0.01,
        0.016,
        (x, y - h / 2.0 - 0.004, z + 0.004),
        frame,
        bevel = 0.003,
        bevel_segments = 1
    );
    if let Some(shutters) = shutters {
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("{name}_shutter{s}"),
                w * 0.42,
                h,
                0.005,
                (x + sf * (w / 2.0 + w * 0.21 + 0.004), y, z - 0.001),
                shutters,
                bevel = 0.0015,
                bevel_segments = 1
            );
            for k in 0..3 {
                fbox!(
                    F,
                    format!("{name}_louvre{s}{k}"),
                    w * 0.32,
                    0.004,
                    0.003,
                    (
                        x + sf * (w / 2.0 + w * 0.21 + 0.004),
                        y - h / 3.0 + h / 3.0 * k as f64,
                        z + 0.002
                    ),
                    shutters
                );
            }
        }
    }
}

pub fn doll_house() -> Vec<Obj> {
    let z = -0.16;
    fbox!(
        F,
        "base",
        0.8,
        0.04,
        0.42,
        (0, 0.02, -0.15),
        M.house_floor,
        bevel = 0.008,
        bevel_segments = 2
    );
    fbox!(
        F,
        "back",
        0.76,
        0.66,
        0.02,
        (0, 0.37, -0.34),
        M.house,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        "paper_down",
        0.72,
        0.31,
        0.004,
        (0, 0.205, -0.328),
        M.wallpaper
    );
    fbox!(
        F,
        "paper_up",
        0.72,
        0.30,
        0.004,
        (0, 0.53, -0.328),
        M.wallpaper_up
    );
    for s in [-1, 1] {
        fbox!(
            F,
            format!("side{s}"),
            0.02,
            0.66,
            0.38,
            (s as f64 * 0.37, 0.37, z),
            M.house,
            bevel = 0.004,
            bevel_segments = 1
        );
        // corner boards up the front edges of the sides
        fbox!(
            F,
            format!("corner_board{s}"),
            0.024,
            0.66,
            0.016,
            (s as f64 * 0.37, 0.37, 0.03),
            M.house_trim,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    fbox!(
        F,
        "floor_down",
        0.72,
        0.006,
        0.36,
        (0, 0.043, z),
        M.house_floor
    );
    fbox!(
        F,
        "floor_up",
        0.72,
        0.02,
        0.37,
        (0, 0.37, z),
        M.house_floor,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        "ceiling",
        0.76,
        0.02,
        0.38,
        (0, 0.69, z),
        M.house,
        bevel = 0.003,
        bevel_segments = 1
    );
    // the gables, a round window in the front one, and the roof: two slabs with courses of shingles, a ridge, a
    // chimney
    let gable = [(-0.38, 0.0), (0.38, 0.0), (0.0, ROOF_RISE)];
    for (k, zg) in [0.03, -0.35].into_iter().enumerate() {
        slab(
            &format!("gable{k}"),
            &gable,
            &plane((0.0, 0.7, zg), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)),
            M.house,
            0.0,
            0.02,
        );
    }
    torus!(
        "gable_window",
        0.042,
        0.008,
        at = F.at(0, 0.77, 0.052),
        material = M.house_trim,
        rot = (90, 0, 0),
        major_segments = 16,
        minor_segments = 4
    );
    fcyl!(
        F,
        "gable_glass",
        0.036,
        0.008,
        (0, 0.77, 0.05),
        M.house_window,
        rot = (90, 0, 0),
        verts = 16
    );
    let a = degrees(atan2(ROOF_RISE, 0.38));
    let (ca, sa) = (cos(radians(a)), sin(radians(a)));
    let run = 0.23 * ca;
    for s in [-1, 1] {
        let sf = s as f64;
        let y = 0.7 + ROOF_RISE - run * tan(radians(a)) + 0.015 / ca;
        fbox!(
            F,
            format!("roof{s}"),
            0.46,
            0.03,
            0.44,
            (sf * run, y, z),
            M.roof,
            rot = (0.0, -sf * a, 0.0),
            bevel = 0.006,
            bevel_segments = 1
        );
        // four courses of shingles, each a strip lying on the roof, its lower edge proud of the one under it
        for k in 0..4 {
            let d = 0.075 + 0.1 * k as f64; // down the slope from the ridge, the strip's middle
            fbox!(
                F,
                format!("shingles{s}{k}"),
                0.11,
                0.008,
                0.43,
                (
                    sf * (d * ca + 0.034 * sa),
                    0.7 + ROOF_RISE - d * sa + 0.034 * ca,
                    z
                ),
                M.roof,
                rot = (0.0, -sf * (a + 7.0), 0.0),
                bevel = 0.003,
                bevel_segments = 1
            );
        }
    }
    fcyl!(
        F,
        "ridge",
        0.012,
        0.44,
        (0, 0.894, z),
        M.house_trim,
        rot = (90, 0, 0),
        verts = 8
    );
    fbox!(
        F,
        "chimney",
        0.07,
        0.12,
        0.08,
        (0.2, 0.82, -0.25),
        M.roof,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        "chimney_cap",
        0.085,
        0.015,
        0.095,
        (0.2, 0.8875, -0.25),
        M.house_trim,
        bevel = 0.003,
        bevel_segments = 1
    );
    // the rooms: downstairs a sofa, a table with a lamp, a rug and the stairs; upstairs a bed, a cupboard and a picture
    fbox!(
        F,
        "rug",
        0.26,
        0.004,
        0.16,
        (-0.05, 0.048, -0.12),
        M.wallpaper_up,
        bevel = 0.0015,
        bevel_segments = 1
    );
    soft!(
        F,
        "sofa",
        0.2,
        0.05,
        0.08,
        (-0.17, 0.073, -0.28),
        M.toy_blue,
        round = 0.012,
        round_segments = 1,
        flat = 1
    );
    soft!(
        F,
        "sofa_back",
        0.2,
        0.07,
        0.03,
        (-0.17, 0.125, -0.31),
        M.toy_blue,
        round = 0.012,
        round_segments = 1,
        flat = 1
    );
    for s in [-1, 1] {
        fbox!(
            F,
            format!("sofa_arm{s}"),
            0.025,
            0.065,
            0.09,
            (-0.17 + s as f64 * 0.1, 0.09, -0.285),
            M.toy_blue,
            bevel = 0.008,
            bevel_segments = 1
        );
    }
    fcyl!(
        F,
        "table",
        0.045,
        0.01,
        (0.06, 0.1, -0.2),
        M.house_door,
        verts = 14
    );
    fcyl!(
        F,
        "table_leg",
        0.006,
        0.055,
        (0.06, 0.0735, -0.2),
        M.house_door,
        verts = 6
    );
    fcone!(
        F,
        "lamp",
        0.018,
        0.01,
        0.022,
        (0.06, 0.122, -0.2),
        M.wallpaper,
        verts = 10
    );
    for k in 0..6 {
        // the stairs up the right wall
        let kf = k as f64;
        fbox!(
            F,
            format!("stair{k}"),
            0.1,
            0.05 * (kf + 1.0),
            0.045,
            (0.3, 0.025 * (kf + 1.0) + 0.046, -0.06 - 0.045 * kf),
            M.house_floor
        );
    }
    soft!(
        F,
        "bed",
        0.2,
        0.045,
        0.12,
        (-0.18, 0.405, -0.26),
        M.toy_pink,
        round = 0.012,
        round_segments = 1,
        flat = 1
    );
    fbox!(
        F,
        "bed_head",
        0.012,
        0.07,
        0.13,
        (-0.285, 0.415, -0.26),
        M.house_door,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        "bed_pillow",
        0.05,
        0.02,
        0.08,
        (-0.25, 0.437, -0.26),
        M.house_trim,
        bevel = 0.008,
        bevel_segments = 1
    );
    fbox!(
        F,
        "cupboard",
        0.1,
        0.14,
        0.06,
        (0.2, 0.45, -0.3),
        M.toy_yellow,
        bevel = 0.006,
        bevel_segments = 1
    );
    fbox!(
        F,
        "cupboard_split",
        0.003,
        0.12,
        0.062,
        (0.2, 0.45, -0.3),
        M.house_door
    );
    fbox!(
        F,
        "picture_frame",
        0.11,
        0.09,
        0.005,
        (0.0, 0.56, -0.324),
        M.house_door,
        bevel = 0.002,
        bevel_segments = 1
    );
    fbox!(
        F,
        "picture",
        0.09,
        0.07,
        0.006,
        (0.0, 0.56, -0.324),
        M.toy_green
    );
    still("body", None);
    // the front wall, hinged on its left edge: a panelled door under a little porch roof, windows with shutters,
    // a flower box
    let parts = new_parts(|| {
        fbox!(
            F,
            "front",
            0.76,
            0.66,
            0.02,
            (0, 0.37, 0.04),
            M.house,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "skirt",
            0.76,
            0.04,
            0.008,
            (0, 0.06, 0.052),
            M.house_trim,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            F,
            "band",
            0.76,
            0.014,
            0.008,
            (0, 0.37, 0.052),
            M.house_trim,
            bevel = 0.002,
            bevel_segments = 1
        );
        shaker(
            &F,
            "door",
            0.11,
            0.19,
            (-0.2, 0.135, 0.054),
            0.008,
            M.house_door,
            0.02,
            None,
            0.002,
            false,
        );
        fextrude!(
            F,
            "door_frame",
            &[
                (0.072, 0.0),
                (0.072, 0.21),
                (-0.072, 0.21),
                (-0.072, 0.0),
                (-0.058, 0.0),
                (-0.058, 0.196),
                (0.058, 0.196),
                (0.058, 0.0)
            ],
            0.006,
            (-0.2, 0.04, 0.054),
            M.house_trim
        );
        sphere!(
            "door_knob",
            0.007,
            at = F.at(-0.165, 0.135, 0.062),
            material = M.toy_yellow,
            segments = 6,
            rings = 4
        );
        fextrude!(
            F,
            "porch",
            &[(-0.09, 0.0), (0.09, 0.0), (0.0, 0.045)],
            0.02,
            (-0.2, 0.255, 0.064),
            M.roof,
            bevel = 0.003,
            bevel_segments = 1
        );
        for (k, (x, y)) in [(0.18, 0.2), (-0.18, 0.54), (0.18, 0.54)]
            .into_iter()
            .enumerate()
        {
            window(
                &format!("window{k}"),
                x,
                y,
                0.054,
                0.12,
                0.13,
                M.house_trim,
                M.house_window,
                Some(M.wallpaper_up),
            );
        }
        fbox!(
            F,
            "flower_box",
            0.14,
            0.03,
            0.022,
            (0.18, 0.11, 0.061),
            M.house_door,
            bevel = 0.004,
            bevel_segments = 1
        );
        for k in 0..5 {
            let kf = k as f64;
            fsphere!(
                F,
                format!("leaf{k}"),
                0.012,
                (0.13 + 0.025 * kf, 0.128, 0.063),
                M.grass,
                segments = 5,
                rings = 3
            );
            fsphere!(
                F,
                format!("flower{k}"),
                0.009,
                (0.13 + 0.025 * kf, 0.136, 0.066),
                [M.toy_pink, M.toy_yellow][k % 2],
                segments = 5,
                rings = 3
            );
        }
        fbox!(
            F,
            "step",
            0.16,
            0.02,
            0.012,
            (-0.2, 0.035, 0.057),
            M.house_trim,
            bevel = 0.003,
            bevel_segments = 1
        );
    });
    let front = hinged(join(&parts, "front_wall", None), F.at(-0.38, 0.37, 0.04));
    place(-0.05);
    swing!(front, "front_open", 120.0, seconds = 1.0);
    meshes()
}
