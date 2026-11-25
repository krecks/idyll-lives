//! The printing press: 2x2, a small web press rolling out the paper. A reel of newsprint behind it; the web runs
//! over a guide roller into the printing unit between two cast side frames (ink rollers under a hood, the plate and
//! blanket cylinders, the impression cylinder under them) and out to the front, where the cut pages pile on the
//! delivery table. The plate and the blanket cylinders turn against each other while a resident watches it run
//! (`rollers`, a loop). Its front to the watcher (+z).

use super::common::{F, own_palette};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::materials::{Mat, Palette};
use artkit::math::{Axis, atan2, cos, degrees, radians, sin};
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::Facing;
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::shapes::sides;
use artkit::{fbox, fcyl, fextrude, spin};

own_palette! { PRESS:
    body = ("#6A8C80", satin),        // the press's machine green
    plate = ("#B9BEC2", metal),       // the plate cylinder's aluminium plate
    blanket = ("#2E3F63", matte),     // the blanket's rubber
    roller = ("#3B3E42", satin),      // the ink rollers' rubber
    go = ("#3FAE5A", gloss),          // the run button
}
/// Grey type, on the blanket and the pages (`type` is no name a field can have).
pub const TYPE: Mat = Mat::new("type", "#8C9196", "matte");

/// The press's palette: the shared colours and its own, in the script's order.
pub fn palette() -> Palette {
    let own = PRESS.palette();
    let at = own
        .colours
        .iter()
        .position(|m| m.name == "roller")
        .expect("roller");
    let mut colours = own.colours;
    colours.insert(at, TYPE);
    Palette::new(&colours)
}

pub const PP_FRAME: f64 = 0.66; // the side frames' inner faces, game x
pub const PP_PLATE: (f64, f64, f64) = (1.2, 0.0, 0.12); // the plate cylinder: height, depth, radius
pub const PP_BLANKET: (f64, f64, f64) = (0.99, 0.13, 0.12); // the blanket cylinder, against it below and in front
pub const PP_IMPRESSION: (f64, f64, f64) = (0.79, 0.27, 0.115);
pub const PP_TURN: f64 = 1.6; // seconds a turn

/// A cylinder across the press between its frames, its axle's ends in the frames: its surface, the dark gap where
/// the plate's (or the blanket's) ends are clamped, and `marks` (the printed blocks it carries) round it. Hinged on
/// its axle, to turn.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn press_cylinder(
    name: &str,
    y: f64,
    z: f64,
    r: f64,
    surface: Mat,
    marks: &[(i32, f64, f64)],
    m: &PRESS,
) -> Obj {
    let l = 2.0 * PP_FRAME - 0.02;
    let parts = new_parts(|| {
        fcyl!(
            F,
            name,
            r,
            l,
            (0, y, z),
            surface,
            rot = (0, 90, 0),
            verts = sides(r, 16, 32)
        );
        fcyl!(
            F,
            format!("{name}_axle"),
            0.035,
            2.0 * PP_FRAME + 0.1,
            (0, y, z),
            m.steel,
            rot = (0, 90, 0),
            verts = 10
        );
        for s in [-1, 1] {
            // its bearers: steel rings at its ends
            fcyl!(
                F,
                format!("{name}_bearer{s}"),
                r + 0.004,
                0.03,
                (s as f64 * (PP_FRAME - 0.03), y, z),
                m.steel,
                rot = (0, 90, 0),
                verts = sides(r, 16, 32)
            );
        }
        fbox!(
            F,
            format!("{name}_gap"),
            l - 0.08,
            0.012,
            0.03,
            (0, y + r - 0.003, z),
            m.dark
        );
        fbox!(
            F,
            format!("{name}_clamp"),
            l - 0.08,
            0.01,
            0.012,
            (0, y + r - 0.001, z - 0.021),
            m.steel
        );
        for (k, &(a, w, x)) in marks.iter().enumerate() {
            // print blocks: columns of grey type and a black headline
            let t = radians(a as f64);
            fbox!(
                F,
                format!("{name}_mark{k}"),
                w,
                0.004,
                0.05,
                (x, y + (r + 0.0005) * cos(t), z - (r + 0.0005) * sin(t)),
                if k % 3 == 0 { m.ink } else { TYPE },
                rot = (-a, 0, 0)
            );
        }
    });
    hinged(join(&parts, name, None), F.at(0, y, z))
}

pub fn printing_press() -> Vec<Obj> {
    let m = &PRESS;
    let green = m.body;
    let still = new_parts(|| {
        // the base: a cabinet on a dark plinth, panels and vents on its front
        fbox!(F, "plinth", 1.44, 0.08, 1.06, (0, 0.04, -0.1), m.dark);
        fbox!(
            F,
            "base",
            1.5,
            0.56,
            1.12,
            (0, 0.36, -0.1),
            green,
            bevel = 0.02,
            bevel_segments = 2
        );
        for (k, x) in [-0.37, 0.37].into_iter().enumerate() {
            fbox!(
                F,
                format!("panel_seam{k}"),
                0.66,
                0.42,
                0.004,
                (x, 0.36, 0.461),
                m.dark
            );
            fbox!(
                F,
                format!("panel{k}"),
                0.64,
                0.4,
                0.006,
                (x, 0.36, 0.463),
                green,
                bevel = 0.002,
                bevel_segments = 1
            );
            for j in 0..5 {
                fbox!(
                    F,
                    format!("vent{k}{j}"),
                    0.3,
                    0.012,
                    0.004,
                    (x, 0.24 + j as f64 * 0.025, 0.4665),
                    m.dark
                );
            }
            fcyl!(
                F,
                format!("panel_lock{k}"),
                0.012,
                0.01,
                (x + 0.27, 0.5, 0.468),
                m.chrome,
                rot = (90, 0, 0),
                verts = 8
            );
        }
        fbox!(
            F,
            "deck",
            1.5,
            0.03,
            1.12,
            (0, 0.655, -0.1),
            m.iron,
            bevel = 0.006,
            bevel_segments = 1
        );
        // the side frames: cast plates rising over the cylinders, a hole for each axle
        let frame = smooth_outline(
            &[
                (-0.62, 0.66),
                (0.44, 0.66),
                (0.44, 0.92),
                (0.3, 1.24),
                (0.12, 1.42),
                (-0.22, 1.47),
                (-0.5, 1.4),
                (-0.62, 1.2),
            ],
            2,
            true,
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fextrude!(
                F,
                format!("frame{s}"),
                &frame,
                0.07,
                (sf * (PP_FRAME + 0.035), 0, 0),
                green,
                plane = Facing::Side,
                bevel = 0.01,
                bevel_segments = 1
            );
            for (k, (y, z, _r)) in [PP_PLATE, PP_BLANKET, PP_IMPRESSION]
                .into_iter()
                .enumerate()
            {
                fcyl!(
                    F,
                    format!("boss{s}{k}"),
                    0.06,
                    0.02,
                    (sf * (PP_FRAME + 0.075), y, z),
                    m.iron,
                    rot = (0, 90, 0),
                    verts = 14
                );
            }
        }
        // the impression cylinder (steel, smooth: it shows no turning), the ink rollers in a pyramid under a hood,
        // the ink fountain at the back
        let (y, z, r) = PP_IMPRESSION;
        fcyl!(
            F,
            "impression",
            r,
            2.0 * PP_FRAME - 0.02,
            (0, y, z),
            m.steel,
            rot = (0, 90, 0),
            verts = sides(r, 16, 32)
        );
        for (k, (yy, zz, rr)) in [
            (1.36, -0.08, 0.045),
            (1.37, 0.03, 0.04),
            (1.43, -0.17, 0.04),
            (1.45, -0.05, 0.045),
            (1.33, -0.24, 0.04),
        ]
        .into_iter()
        .enumerate()
        {
            fcyl!(
                F,
                format!("ink_roller{k}"),
                rr,
                2.0 * PP_FRAME - 0.03,
                (0, yy, zz),
                if k % 2 != 0 { m.steel } else { m.roller },
                rot = (0, 90, 0),
                verts = 14
            );
        }
        fextrude!(
            F,
            "fountain",
            &[
                (-0.36, 1.3),
                (-0.22, 1.3),
                (-0.2, 1.36),
                (-0.4, 1.42),
                (-0.44, 1.42)
            ],
            2.0 * PP_FRAME - 0.02,
            (0, 0, 0),
            m.iron,
            plane = Facing::Side
        );
        fbox!(
            F,
            "fountain_ink",
            2.0 * PP_FRAME - 0.06,
            0.01,
            0.1,
            (0, 1.39, -0.36),
            m.ink
        );
        // the hood over the ink rollers, open to the front where the cylinders show
        let steps: Vec<i32> = (0..11).map(|k| 70 + 10 * k).collect();
        let mut hood: Vec<(f64, f64)> = steps
            .iter()
            .map(|&a| {
                (
                    -0.16 + 0.3 * cos(radians(a as f64)),
                    1.3 + 0.21 * sin(radians(a as f64)),
                )
            })
            .collect();
        hood.extend(steps.iter().rev().map(|&a| {
            (
                -0.16 + 0.288 * cos(radians(a as f64)),
                1.3 + 0.198 * sin(radians(a as f64)),
            )
        }));
        fextrude!(
            F,
            "hood",
            &hood,
            2.0 * PP_FRAME - 0.01,
            (0, 0, 0),
            green,
            plane = Facing::Side
        );
        // the reel stand behind it: two A legs, the reel of newsprint on its shaft, a brake disc
        let (ry, rz, rr) = (0.4, -0.7, 0.22);
        for s in [-1, 1] {
            let sf = s as f64;
            fextrude!(
                F,
                format!("reel_leg{s}"),
                &[
                    (-0.88, 0.0),
                    (-0.56, 0.0),
                    (rz + 0.05, ry + 0.06),
                    (rz - 0.05, ry + 0.06)
                ],
                0.06,
                (sf * 0.6, 0, 0),
                green,
                plane = Facing::Side,
                bevel = 0.008,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("reel_foot{s}"),
                0.12,
                0.03,
                0.36,
                (sf * 0.6, 0.015, rz),
                m.iron
            );
        }
        fcyl!(
            F,
            "reel",
            rr,
            1.0,
            (0, ry, rz),
            m.paper,
            rot = (0, 90, 0),
            verts = sides(rr, 24, 40)
        );
        fcyl!(
            F,
            "reel_core",
            0.05,
            1.04,
            (0, ry, rz),
            m.dark,
            rot = (0, 90, 0),
            verts = 12
        );
        fcyl!(
            F,
            "reel_shaft",
            0.025,
            1.3,
            (0, ry, rz),
            m.steel,
            rot = (0, 90, 0),
            verts = 10
        );
        fcyl!(
            F,
            "brake",
            0.11,
            0.025,
            (-0.66, ry, rz),
            m.iron,
            rot = (0, 90, 0),
            verts = 16
        );
        // the web: up from the reel over the guide roller at the back of the frames, and out of the cylinders'
        // nip to the cutter at the front
        fcyl!(
            F,
            "guide",
            0.04,
            2.0 * PP_FRAME - 0.02,
            (0, 1.16, -0.56),
            m.steel,
            rot = (0, 90, 0),
            verts = 12
        );
        let a0 = (rz + rr * 0.15, ry + rr * 0.99);
        let a1 = (-0.56 - 0.03, 1.16 + 0.03);
        let l = py::hypot(a1.0 - a0.0, a1.1 - a0.1);
        let ang = degrees(atan2(a1.1 - a0.1, a1.0 - a0.0));
        fbox!(
            F,
            "web_in",
            0.96,
            0.002,
            l,
            (0, (a0.1 + a1.1) / 2.0, (a0.0 + a1.0) / 2.0),
            m.paper,
            rot = (ang, 0, 0)
        );
        fbox!(F, "web_out", 0.96, 0.002, 0.2, (0, 0.7, 0.44), m.paper);
        // the cutter at the front and the delivery table with its pile of pages, a few fanned on top
        fbox!(
            F,
            "cutter",
            1.2,
            0.14,
            0.14,
            (0, 0.74, 0.5),
            green,
            bevel = 0.015,
            bevel_segments = 1
        );
        fbox!(F, "cutter_slot", 1.0, 0.01, 0.004, (0, 0.72, 0.571), m.dark);
        fbox!(
            F,
            "table",
            1.0,
            0.025,
            0.36,
            (0, 0.6, 0.75),
            m.steel,
            bevel = 0.006,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("table_leg{s}"),
                0.04,
                0.6,
                0.04,
                (sf * 0.45, 0.3, 0.9),
                m.iron
            );
            fbox!(
                F,
                format!("table_side{s}"),
                0.015,
                0.1,
                0.36,
                (sf * 0.49, 0.66, 0.75),
                m.steel
            );
        }
        fbox!(
            F,
            "pile",
            0.56,
            0.14,
            0.32,
            (0, 0.6125 + 0.07, 0.75),
            m.paper,
            bevel = 0.004,
            bevel_segments = 1
        );
        for k in 0..3 {
            let yk = 0.7525 + 0.003 * k as f64;
            let rot = (0, 4 - 5 * k, 0);
            fbox!(
                F,
                format!("page{k}"),
                0.56,
                0.002,
                0.34,
                (0.01 * k as f64, yk, 0.75 + 0.012 * k as f64),
                m.paper,
                rot = rot
            );
        }
        // the top page's front: its headline and columns
        fbox!(
            F,
            "headline",
            0.44,
            0.003,
            0.05,
            (0.02, 0.7585, 0.66),
            m.ink,
            rot = (0, -6, 0)
        );
        for j in 0..4 {
            fbox!(
                F,
                format!("column{j}"),
                0.1,
                0.003,
                0.2,
                (-0.15 + j as f64 * 0.12, 0.7585, 0.8),
                TYPE,
                rot = (0, -6, 0)
            );
        }
        fbox!(
            F,
            "photo",
            0.12,
            0.0032,
            0.1,
            (0.15, 0.759, 0.74),
            m.ink,
            rot = (0, -6, 0)
        );
        // the control panel on the right frame's front: a sloping desk of buttons, a speed dial, the stop
        fbox!(
            F,
            "control_post",
            0.12,
            0.5,
            0.12,
            (-0.78, 0.9, 0.3),
            green,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            F,
            "control_desk",
            0.26,
            0.2,
            0.08,
            (-0.78, 1.2, 0.33),
            m.plastic,
            rot = (-30, 0, 0),
            bevel = 0.01,
            bevel_segments = 1
        );
        for (k, (x, mat)) in [(-0.71, m.go), (-0.78, m.amber), (-0.85, m.white)]
            .into_iter()
            .enumerate()
        {
            fcyl!(
                F,
                format!("button{k}"),
                0.016,
                0.02,
                (x, 1.22, 0.375),
                mat,
                rot = (60, 0, 0),
                verts = 10
            );
        }
        fcyl!(
            F,
            "stop",
            0.028,
            0.03,
            (-0.78, 1.14, 0.42),
            m.red_lens,
            rot = (60, 0, 0),
            verts = 12
        );
        // the drive's belt guard on the left frame
        fextrude!(
            F,
            "belt_guard",
            &rounded_rect(0.7, 0.5, 0.18, 4),
            0.1,
            (0.76, 1.0, -0.1),
            m.body,
            plane = Facing::Side,
            bevel = 0.01,
            bevel_segments = 1
        );
    });
    join(&still, "printingPress", Some((0, 0, 0).into()));
    // the plate and blanket cylinders, each turning on its axle
    let page: Vec<(i32, f64, f64)> = [30, 75, 200, 250, 300]
        .iter()
        .flat_map(|&a| [-0.3, 0.3].map(|x| (a, 0.3, x)))
        .collect();
    let (y, z, r) = PP_PLATE;
    let plate = press_cylinder("plate_cylinder", y, z, r, m.plate, &page, m);
    let (y, z, r) = PP_BLANKET;
    let blanket = press_cylinder("blanket_cylinder", y, z, r, m.blanket, &page, m);
    settle_all();
    spin!(
        plate,
        "rollers",
        turns = 1,
        seconds = PP_TURN,
        axis = Axis::X
    );
    spin!(
        blanket,
        "rollers",
        turns = -1,
        seconds = PP_TURN,
        axis = Axis::X
    );
    meshes()
}
