//! H3: the easel: three legs, a tray, a canvas leant back on it, a palette and a jar of brushes. The painting is one
//! part, built on the canvas and grown across it from its left edge while a resident paints (`painting`).

use super::M;
use super::common::{P, ends, rr};
use artkit::forms::smooth_outline;
use artkit::math::{Axis, P3, PI, TAU, cos, radians, sin};
use artkit::motion::{Grow, grow};
use artkit::parts::{Settle, join, new_parts, settle_with};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, revolve};

/// The canvas leans back.
const EASEL_TILT: f64 = -7.29;
/// Its middle.
const CANVAS: P = (0.0, 1.204, 0.041);

/// Game point `u` across, `v` up the canvas from its middle and `out` off its face.
fn on_canvas<U: Coord>(u: U, v: f64, out: f64) -> (U, f64, f64) {
    let t = radians(EASEL_TILT);
    let (up, normal) = ((cos(t), sin(t)), (-sin(t), cos(t)));
    (
        u,
        CANVAS.1 + v * up.0 + out * normal.0,
        CANVAS.2 + v * up.1 + out * normal.1,
    )
}

pub fn easel() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let tilt = (EASEL_TILT, 0.0, 0.0);
    let leg = rr(0.034, 0.024, 0.005, 1);
    let still = new_parts(|| {
        // three legs of square section, tapering to their feet, a brass-capped hinge where they meet
        for (name, at, length, rotate) in [
            ("leg-1", (-0.175, 0.866, 0.05), 1.74, (11.03, -131.35, 0.0)),
            ("leg1", (0.175, 0.866, 0.05), 1.74, (11.03, 131.35, 0.0)),
            ("leg_back", (0.0, 0.826, -0.24), 1.66, (11.12, 0.0, 0.0)),
        ] {
            let (a, b) = ends(at, length, rotate);
            let (foot, top) = if a.1 < b.1 { (a, b) } else { (b, a) };
            fsweep!(
                f,
                name,
                &[foot, top],
                0,
                M.easel_wood,
                profile = &leg,
                up = (1, 0, 0),
                scale = [0.8, 1.0]
            );
            fcyl!(
                f,
                format!("{name}_shoe"),
                0.014,
                0.012,
                (foot.0, 0.006, foot.2),
                M.foot,
                verts = 8
            );
        }
        fbox!(
            f,
            "head",
            0.12,
            0.08,
            0.07,
            (0, 1.69, -0.06),
            M.easel_dark,
            bevel = 0.012
        );
        fcyl!(
            f,
            "hinge",
            0.012,
            0.13,
            (0, 1.69, -0.095),
            M.easel_metal,
            rot = (0, 90, 0),
            verts = 8
        );
        // the mast, with the canvas clamp slid down onto the canvas and its wing nut
        fbox!(
            f,
            "mast",
            0.04,
            1.0,
            0.025,
            on_canvas(0, 0.05, -0.03),
            M.easel_wood,
            rot = tilt,
            bevel = 0.005
        );
        let clamp = [
            (0.05, -0.03),
            (0.05, 0.03),
            (-0.05, 0.03),
            (-0.05, -0.03),
            (-0.03, -0.03),
            (-0.03, 0.0),
            (0.03, 0.0),
            (0.03, -0.03),
        ];
        fextrude!(
            f,
            "clamp",
            &clamp,
            0.05,
            (0, 1.581, -0.007),
            M.easel_dark,
            rot = tilt,
            bevel = 0.004,
            bevel_segments = 1
        );
        revolve!(
            "wing_nut",
            &[
                (0.0, 0.0),
                (0.012, 0.0),
                (0.012, 0.008),
                (0.006, 0.012),
                (0.006, 0.02),
                (0.0, 0.02)
            ],
            at = f.at(0, 1.581, -0.032),
            material = M.easel_metal,
            segments = 10,
            rot = (-90.0 + EASEL_TILT, 0.0, 0.0)
        );
        fbox!(
            f,
            "wing",
            0.05,
            0.014,
            0.003,
            (0, 1.581, -0.05),
            M.easel_metal,
            rot = tilt,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "canvas",
            0.56,
            0.72,
            0.025,
            (0, CANVAS.1, CANVAS.2),
            M.canvas,
            rot = tilt,
            bevel = 0.004
        );
        fbox!(
            f,
            "bar",
            0.47,
            0.03,
            0.02,
            (0, 0.45, 0.102),
            M.easel_wood,
            rot = tilt,
            bevel = 0.005
        );
        // the tray: a ledge with a lip and a groove for brushes, a knob that sets its height
        fextrude!(
            f,
            "tray",
            &[
                (0.06, 0.8175),
                (0.154, 0.8175),
                (0.156, 0.82),
                (0.156, 0.872),
                (0.148, 0.876),
                (0.142, 0.872),
                (0.142, 0.842),
                (0.12, 0.842),
                (0.115, 0.835),
                (0.105, 0.835),
                (0.1, 0.842),
                (0.06, 0.842)
            ],
            0.64,
            (0, 0, 0),
            M.easel_dark,
            plane = Facing::Side,
            bevel = 0.002,
            bevel_segments = 1
        );
        revolve!(
            "tray_knob",
            &[
                (0.0, 0.0),
                (0.012, 0.0),
                (0.014, 0.01),
                (0.01, 0.02),
                (0.0, 0.022)
            ],
            at = f.at(0, 0.83, 0.156),
            material = M.easel_metal,
            segments = 10,
            rot = (90, 0, 0)
        );
        // the palette (a thumb hole through it) and its paint, two tubes of paint on the tray
        let pal = smooth_outline(
            &[
                (0.09, 0.0),
                (0.07, 0.04),
                (0.0, 0.05),
                (-0.07, 0.04),
                (-0.09, 0.0),
                (-0.07, -0.04),
                (-0.02, -0.045),
                (0.0, -0.03),
                (0.03, -0.045),
                (0.08, -0.03),
            ],
            2,
            true,
        );
        let thumb: Vec<(f64, f64)> = (0..10)
            .map(|i| {
                (
                    0.045 + 0.009 * cos(TAU * i as f64 / 10.0),
                    0.01 + 0.009 * sin(TAU * i as f64 / 10.0),
                )
            })
            .collect();
        fextrude!(
            f,
            "palette",
            &pal,
            0.006,
            (0.12, 0.845, 0.1),
            M.palette,
            holes = vec![thumb],
            plane = Facing::Top,
            bevel = 0.0015,
            bevel_segments = 1
        );
        for (k, (x, z, colour)) in [
            (0.07, 0.09, M.ball_red),
            (0.095, 0.115, M.sun),
            (0.125, 0.12, M.river),
            (0.155, 0.11, M.label),
            (0.18, 0.09, M.tree),
        ]
        .into_iter()
        .enumerate()
        {
            fsphere!(
                f,
                format!("dab{k}"),
                0.01,
                (x, 0.849, z),
                colour,
                scale = (1, 1, 0.45),
                segments = 8,
                rings = 5
            );
        }
        for (k, (x, colour)) in [(-0.2, M.ball_blue), (-0.12, M.ball_red)]
            .into_iter()
            .enumerate()
        {
            fsweep!(
                f,
                format!("paint_tube{k}"),
                &[(x, 0.8525, 0.065), (x, 0.8525, 0.125)],
                0.01,
                M.can,
                verts = 8,
                scale = [0.35, 1.0]
            );
            fcyl!(
                f,
                format!("paint_cap{k}"),
                0.007,
                0.012,
                (x, 0.8525, 0.131),
                colour,
                rot = (90, 0, 0),
                verts = 8
            );
            fbox!(
                f,
                format!("paint_label{k}"),
                0.016,
                0.004,
                0.03,
                (x, 0.862, 0.1),
                colour
            );
        }
        // the jar of brushes on the floor: a turned jar, water in it, three brushes with ferrules and bristles
        revolve!(
            "jar",
            &[
                (0.0, 0.0),
                (0.033, 0.0),
                (0.035, 0.004),
                (0.035, 0.064),
                (0.038, 0.068),
                (0.036, 0.072),
                (0.03, 0.072),
                (0.03, 0.004),
                (0.0, 0.004)
            ],
            at = f.at(0.33, 0.0, 0.3),
            material = M.jar,
            segments = 14
        );
        fcyl!(
            f,
            "jar_water",
            0.03,
            0.002,
            (0.33, 0.05, 0.3),
            M.river,
            verts = 12,
            smooth = false
        );
        for (k, (at, rotate, colour)) in [
            ((0.3, 0.165, 0.3), (7.59, -90.0, 0.0), M.palette),
            ((0.36, 0.165, 0.325), (9.85, 50.19, 0.0), M.ball_red),
            ((0.33, 0.165, 0.27), (7.59, 180.0, 0.0), M.foot),
        ]
        .into_iter()
        .enumerate()
        {
            let (a, b) = ends(at, 0.272, rotate);
            let (lo, hi) = if a.1 < b.1 { (a, b) } else { (b, a) };
            let on = |t: f64| {
                (
                    lo.0 + (hi.0 - lo.0) * t,
                    lo.1 + (hi.1 - lo.1) * t,
                    lo.2 + (hi.2 - lo.2) * t,
                )
            };
            fsweep!(
                f,
                format!("brush{k}"),
                &[lo, on(0.5), on(0.82)],
                0.004,
                colour,
                verts = 6,
                scale = [0.7, 1.0, 0.85]
            );
            fsweep!(
                f,
                format!("ferrule{k}"),
                &[on(0.82), on(0.9)],
                0.0042,
                M.easel_metal,
                verts = 6,
                scale = [1.0, 0.9]
            );
            fsweep!(
                f,
                format!("bristles{k}"),
                &[on(0.9), on(0.96), on(1.0)],
                0.004,
                M.easel_dark,
                verts = 6,
                scale = [1.0, 1.1, 0.2]
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    let p = new_parts(|| {
        // the painting: a sky, a green hill with a rolling skyline, the sun, two trees and a winding stream
        fbox!(
            f,
            "sky",
            0.52,
            0.3,
            0.003,
            on_canvas(0, 0.1925, 0.014),
            M.sky,
            rot = tilt
        );
        let mut hill = vec![(0.26, -0.33), (0.26, 0.05)];
        hill.extend((1..12).map(|i| {
            (
                0.26 - 0.52 * i as f64 / 12.0,
                0.03 + 0.03 * sin(PI * i as f64 / 6.0 + 0.6),
            )
        }));
        hill.extend([(-0.26, 0.05), (-0.26, -0.33)]);
        fextrude!(
            f,
            "grass",
            &hill,
            0.003,
            on_canvas(0, 0.0, 0.0152),
            M.hill,
            rot = tilt
        );
        fcyl!(
            f,
            "paint_sun",
            0.035,
            0.003,
            on_canvas(0.16, 0.242, 0.016),
            M.sun,
            rot = (90.0 + EASEL_TILT, 0.0, 0.0),
            verts = 16,
            smooth = false
        );
        fsphere!(
            f,
            "tree",
            0.17,
            on_canvas(-0.11, 0.042, 0.016),
            M.tree,
            scale = (1, 0.012, 0.58),
            rot = tilt,
            segments = 14,
            rings = 8
        );
        fsphere!(
            f,
            "tree2",
            0.12,
            on_canvas(0.15, 0.032, 0.016),
            M.tree_light,
            scale = (1, 0.012, 0.5),
            rot = tilt,
            segments = 12,
            rings = 8
        );
        fbox!(
            f,
            "trunk",
            0.02,
            0.07,
            0.003,
            on_canvas(-0.11, -0.03, 0.0162),
            M.fretboard,
            rot = tilt
        );
        let stream: Vec<(f64, f64)> = (0..9)
            .map(|i| {
                (
                    -0.05 + 0.3 * i as f64 / 8.0 + 0.03 * sin(i as f64),
                    -0.12 - 0.2 * i as f64 / 8.0,
                )
            })
            .collect();
        let mut outline: Vec<(f64, f64)> = stream
            .iter()
            .enumerate()
            .map(|(i, &(x, y))| (x + 0.012 + 0.02 * i as f64 / 8.0, y))
            .collect();
        outline.extend(
            stream
                .iter()
                .enumerate()
                .rev()
                .map(|(i, &(x, y))| (x - 0.012 - 0.02 * i as f64 / 8.0, y)),
        );
        fextrude!(
            f,
            "stream",
            &outline,
            0.003,
            on_canvas(0, 0.0, 0.0162),
            M.river,
            rot = tilt
        );
    });
    let left = on_canvas(-0.26, 0.0, 0.014);
    let painting = join(&p, "painting", Some(f.at(left.0, left.1, left.2)));
    painting.edit(|o| o.scale.x = 0.001);
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    grow(
        painting,
        "painting",
        Grow {
            seconds: 1.2,
            axis: Axis::X,
        },
    );
    meshes()
}
