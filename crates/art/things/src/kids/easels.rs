//! KD3: the chalkboard easel and the kids' easel.

use super::common::{Frame, add, board, mul, place, rect, slab, still, turned};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{Axis, PI, cos, radians, sin, tan};
use artkit::motion::{Ease, Grow, Path, grow, keys};
use artkit::parts::{join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fcyl, fextrude, fsweep, revolve};

const EASEL_TILT: f64 = 14.0;

/// A picture of flat `pieces` [(outline, material)] on `frame`, joined with its origin on its left edge
/// and shrunk to a speck there, so `draw_on` draws it from left to right.
fn drawing(name: &str, pieces: &[(Vec<(f64, f64)>, Mat)], frame: &Frame, left: f64) -> Obj {
    let parts = new_parts(|| {
        for (k, (pts, m)) in pieces.iter().enumerate() {
            slab(
                &format!("{name}{k}"),
                pts,
                frame,
                *m,
                0.0008 * k as f64,
                0.002,
            ); // each over the last
        }
    });
    let o = add(&[frame.o, mul(frame.s, left)]);
    let obj = join(&parts, name, Some(F.at(o.0, o.1, o.2)));
    obj.set_scale((0.001, 0.001, 0.001))
}

/// The drawing's clip: it grows along x from its left edge, its height and depth back to full in the
/// first frames. Squeezed along x only, at rest it showed as a hairline of colours down the board.
fn draw_on(part: Obj, clip: &str, seconds: f64) {
    grow(
        part,
        clip,
        Grow {
            seconds,
            axis: Axis::X,
        },
    );
    for axis in [1, 2] {
        keys(
            part,
            clip,
            Path::Scale,
            axis,
            &[
                (0.0, 0.001, Ease::Smooth),
                (0.1, 1.0, Ease::Smooth),
                (seconds, 1.0, Ease::Smooth),
            ],
        );
    }
}

/// Where a board leaning back by `tilt` degrees through (height 0.5, depth z0) is at height `y`, `d` in front of
/// its middle plane.
fn tilted(y: f64, z0: f64, tilt: f64, d: f64) -> f64 {
    let t = radians(tilt);
    z0 - (y - 0.5) * tan(t) + d / cos(t)
}

pub fn kids_chalkboard() -> Vec<Obj> {
    let t = radians(EASEL_TILT);
    // two frames leaning together, a slate in each; their stiles run down to the floor as feet
    for (k, (z, tilt)) in [(0.11, -EASEL_TILT), (-0.11, EASEL_TILT)]
        .into_iter()
        .enumerate()
    {
        fextrude!(
            F,
            format!("frame{k}"),
            &rounded_rect(0.62, 0.96, 0.03, 2),
            0.03,
            (0, 0.5, z),
            M.chalk_frame,
            rot = (tilt, 0, 0),
            holes = vec![rounded_rect(0.52, 0.8, 0.006, 1)],
            bevel = 0.006,
            bevel_segments = 2
        );
        fbox!(
            F,
            format!("slate{k}"),
            0.522,
            0.802,
            0.036,
            (0, 0.5, z),
            M.slate,
            rot = (tilt, 0, 0)
        );
    }
    // the hinge bar across the top, a turned knob on each end
    fcyl!(
        F,
        "top_bar",
        0.015,
        0.6,
        (0, 0.975, 0.0),
        M.chalk_frame,
        rot = (0, 90, 0),
        verts = 10
    );
    for s in [-1, 1] {
        let sf = s as f64;
        revolve!(
            format!("bar_end{s}"),
            &[
                (0.0, 0.0),
                (0.017, 0.0),
                (0.019, 0.01),
                (0.014, 0.02),
                (0.0, 0.03)
            ],
            at = F.at(sf * 0.3, 0.975, 0.0),
            material = M.chalk_frame,
            segments = 10,
            rot = (0, -90 * s, 0)
        );
        // a cord between the frames, so they can't spread
        let zi = tilted(0.45, 0.11, EASEL_TILT, -0.015);
        fsweep!(
            F,
            format!("cord{s}"),
            &[
                (sf * 0.285, 0.45, zi),
                (sf * 0.285, 0.415, 0.0),
                (sf * 0.285, 0.45, -zi)
            ],
            0.003,
            M.slate,
            verts = 4,
            resolution = 3
        );
    }
    // the chalk tray across the front: chalk, a duster
    fbox!(
        F,
        "tray",
        0.54,
        0.02,
        0.07,
        (0, 0.102, 0.25),
        M.chalk_frame,
        bevel = 0.006,
        bevel_segments = 2
    );
    fbox!(
        F,
        "tray_lip",
        0.54,
        0.03,
        0.01,
        (0, 0.12, 0.285),
        M.chalk_frame,
        bevel = 0.004,
        bevel_segments = 1
    );
    for s in [-1, 1] {
        fbox!(
            F,
            format!("tray_end{s}"),
            0.01,
            0.025,
            0.07,
            (s as f64 * 0.265, 0.12, 0.25),
            M.chalk_frame,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    for (k, (x, m)) in [
        (-0.12, M.chalk_white),
        (0.0, M.chalk_yellow),
        (0.1, M.chalk_pink),
    ]
    .into_iter()
    .enumerate()
    {
        fcyl!(
            F,
            format!("chalk{k}"),
            0.007,
            0.08,
            (x, 0.12, 0.258),
            m,
            rot = (0, 90, 10 * (k as i32 - 1)),
            verts = 6
        );
    }
    fbox!(
        F,
        "duster",
        0.08,
        0.022,
        0.035,
        (0.19, 0.123, 0.252),
        M.chalk_frame,
        bevel = 0.005,
        bevel_segments = 1
    );
    fbox!(
        F,
        "duster_felt",
        0.078,
        0.008,
        0.033,
        (0.19, 0.11, 0.252),
        M.slate,
        bevel = 0.002,
        bevel_segments = 1
    );
    // the board's face: a sun drawn already
    let face = board(
        (0.0, 0.5 + 0.0185 * sin(t), 0.11 + 0.0185 * cos(t)),
        EASEL_TILT,
    );
    let sun: Vec<(f64, f64)> = (0..360)
        .step_by(30)
        .map(|a| {
            (
                -0.13 + 0.055 * cos(radians(a as f64)),
                0.25 + 0.055 * sin(radians(a as f64)),
            )
        })
        .collect();
    slab("sun", &sun, &face, M.chalk_yellow, 0.0, 0.002);
    for k in 0..6 {
        let a = radians((60 * k + 15) as f64);
        let (c, s) = (cos(a), sin(a));
        slab(
            &format!("ray{k}"),
            &[
                (-0.13 + 0.07 * c - 0.004 * s, 0.25 + 0.07 * s + 0.004 * c),
                (-0.13 + 0.07 * c + 0.004 * s, 0.25 + 0.07 * s - 0.004 * c),
                (-0.13 + 0.1 * c + 0.004 * s, 0.25 + 0.1 * s - 0.004 * c),
                (-0.13 + 0.1 * c - 0.004 * s, 0.25 + 0.1 * s + 0.004 * c),
            ],
            &face,
            M.chalk_yellow,
            0.0,
            0.002,
        );
    }
    still("body", None);
    // the drawing the resident makes: a house in the grass
    let house = [
        (rect(0.0, -0.13, 0.16, -0.01), M.chalk_pink),
        (
            vec![(-0.02, -0.01), (0.18, -0.01), (0.08, 0.09)],
            M.chalk_white,
        ),
        (rect(0.06, -0.13, 0.1, -0.06), M.chalk_blue),
        (rect(0.02, -0.04, 0.05, -0.02), M.chalk_blue),
        (rect(-0.2, -0.15, 0.22, -0.135), M.chalk_green),
    ];
    let drawing = drawing("drawing", &house, &face, -0.2);
    place(0.0);
    draw_on(drawing, "drawing", 2.5);
    meshes()
}

/// An easel's leg from the floor at depth z0 up to the hinge at z1, 1.2 m up: a rounded batten.
fn leg(name: &str, x: f64, z0: f64, z1: f64) {
    fsweep!(
        F,
        name,
        &[(x, 0.0, z0), (x, 1.2, z1)],
        0,
        M.easel,
        profile = rounded_rect(0.03, 0.036, 0.01, 2),
        up = (1, 0, 0)
    );
}

pub fn kids_easel() -> Vec<Obj> {
    let t = radians(EASEL_TILT);
    for s in [-1, 1] {
        let sf = s as f64;
        leg(&format!("leg_front{s}"), sf * 0.3, 0.3, 0.0);
        leg(&format!("leg_back{s}"), sf * 0.3, -0.3, 0.0);
        fbox!(
            F,
            format!("board{s}"),
            0.58,
            0.64,
            0.01,
            (0, 0.82, sf * 0.104),
            M.easel,
            rot = (-sf * EASEL_TILT, 0.0, 0.0),
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("paper{s}"),
            0.56,
            0.62,
            0.012,
            (0, 0.82, sf * 0.11),
            M.paper,
            rot = (-sf * EASEL_TILT, 0.0, 0.0)
        );
        // the tray: a shelf with a lip and ends
        fbox!(
            F,
            format!("tray{s}"),
            0.6,
            0.03,
            0.1,
            (0, 0.48, sf * 0.21),
            M.easel,
            bevel = 0.006,
            bevel_segments = 2
        );
        fbox!(
            F,
            format!("tray_lip{s}"),
            0.6,
            0.035,
            0.012,
            (0, 0.5, sf * 0.255),
            M.easel,
            bevel = 0.004,
            bevel_segments = 1
        );
        // a spring clip holding the paper, and a stay folding between the legs
        fbox!(
            F,
            format!("clip{s}"),
            0.12,
            0.04,
            0.02,
            (0, 1.12, sf * 0.035),
            M.easel,
            rot = (-sf * EASEL_TILT, 0.0, 0.0),
            bevel = 0.006,
            bevel_segments = 1
        );
        fcyl!(
            F,
            format!("clip_spring{s}"),
            0.008,
            0.1,
            (0, 1.142, sf * 0.03),
            M.handle,
            rot = (0, 90, 0),
            verts = 8
        );
        fsweep!(
            F,
            format!("stay{s}"),
            &[
                (sf * 0.3, 0.4, 0.2),
                (sf * 0.3, 0.37, 0.0),
                (sf * 0.3, 0.4, -0.2)
            ],
            0.005,
            M.handle,
            verts = 5,
            resolution = 2
        );
    }
    fsweep!(
        F,
        "top_bar",
        &[(-0.315, 1.17, 0.0), (0.315, 1.17, 0.0)],
        0.015,
        M.easel,
        verts = 8
    );
    for s in [-1, 1] {
        revolve!(
            format!("bar_end{s}"),
            &[(0.0, 0.0), (0.017, 0.0), (0.019, 0.008), (0.0, 0.016)],
            at = F.at(s as f64 * 0.315, 1.17, 0.0),
            material = M.easel,
            segments = 8,
            rot = (0, -90 * s, 0)
        );
    }
    // open paint pots on the front tray, each with its paint, a jar of brushes
    for (k, m) in [M.ball_red, M.ball_yellow, M.ball_blue, M.ball_green]
        .into_iter()
        .enumerate()
    {
        let x = -0.2 + 0.1 * k as f64;
        turned(
            &format!("pot{k}"),
            x,
            0.21,
            &[
                (0.0, 0.0),
                (0.024, 0.0),
                (0.026, 0.045),
                (0.022, 0.045),
                (0.02, 0.006),
                (0.0, 0.006),
            ],
            M.paper,
            12,
            0.495,
        );
        fcyl!(
            F,
            format!("paint{k}"),
            0.021,
            0.004,
            (x, 0.53, 0.21),
            m,
            verts = 12
        );
    }
    turned(
        "jar",
        0.22,
        0.21,
        &[
            (0.0, 0.0),
            (0.028, 0.0),
            (0.03, 0.07),
            (0.026, 0.07),
            (0.024, 0.006),
            (0.0, 0.006),
        ],
        M.water,
        12,
        0.495,
    );
    for (k, (dx, dz, m)) in [
        (-0.01, 0.0, M.paint_red),
        (0.008, -0.006, M.paint_blue),
        (0.0, 0.008, M.paint_yellow),
    ]
    .into_iter()
    .enumerate()
    {
        let (a, b) = (
            (0.22 + dx * 0.5, 0.505, 0.21 + dz * 0.5),
            (0.22 + dx * 3.0, 0.66, 0.21 + dz * 3.0),
        );
        fsweep!(F, format!("brush{k}"), &[a, b], 0.0035, M.easel, verts = 5);
        fcone!(
            F,
            format!("bristles{k}"),
            0.004,
            0.0015,
            0.022,
            (b.0, b.1 + 0.009, b.2),
            m,
            verts = 6
        );
    }
    still("body", None);
    // the painting: a sky, a sun, a flower and some grass, painted on from the left
    let face = board(
        (0.0, 0.82 + 0.0065 * sin(t), 0.11 + 0.0065 * cos(t)),
        EASEL_TILT,
    );
    let circle = |cx: f64, cy: f64, r: f64| -> Vec<(f64, f64)> {
        let n = 12;
        (0..n)
            .map(|i| {
                (
                    cx + r * cos(2.0 * PI * i as f64 / n as f64),
                    cy + r * sin(2.0 * PI * i as f64 / n as f64),
                )
            })
            .collect()
    };
    let mut pieces = vec![
        (rect(-0.24, 0.12, 0.24, 0.25), M.paint_blue),
        (circle(0.15, 0.18, 0.05), M.paint_yellow),
        (rect(-0.24, -0.27, 0.24, -0.18), M.paint_green),
        (rect(-0.07, -0.2, -0.05, 0.02), M.paint_green),
        (circle(-0.06, 0.06, 0.035), M.paint_yellow),
    ];
    for k in 0..5 {
        let a = 2.0 * PI * k as f64 / 5.0;
        pieces.push((
            circle(-0.06 + 0.055 * cos(a), 0.06 + 0.055 * sin(a), 0.03),
            M.paint_red,
        ));
    }
    let painting = drawing("painting", &pieces, &face, -0.25);
    place(0.0);
    draw_on(painting, "painting", 2.5);
    meshes()
}
