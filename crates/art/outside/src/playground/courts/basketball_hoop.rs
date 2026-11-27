use super::super::common::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{Axis, P3, TAU, cos, sin};
use artkit::motion::{Rock, Squash, rock, squash};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, settle, soft, torus};

// A hoop on a post with a ballast base at the back of its tile; the rim at 3.05 m (regulation, park and driveway
// hoops alike) reaches forward from a 1.2 x 0.8 m board (a park one's; they were 2.94 m and 0.95 x 0.62). The net
// is its own part, hung from the rim: `swish` sways and stretches it once a ball goes through.
pub(super) const RIM: (f64, f64) = (3.05, 0.27); // (height, depth) of the rim's middle
pub(super) const BOARD_W: f64 = 1.2; // the backboard: its width, height and middle's height
pub(super) const BOARD_H: f64 = 0.8;
pub(super) const BOARD_Y: f64 = 3.3;

/// A diamond-mesh net: strands zigzagging from the rim (r 0.21) to a narrower bottom ring (r 0.13), each strand
/// one cord, a hem round the bottom.
pub(super) fn net(name: &str) -> Obj {
    let (y0, y1, r0, r1, n) = (RIM.0 - 0.01, RIM.0 - 0.38, 0.205, 0.13, 12);
    let z = RIM.1;
    let parts = new_parts(|| {
        let rows = 4;
        for i in 0..n {
            for d in [-1, 1] {
                let mut pts = Vec::new();
                for j in 0..=rows {
                    let t = j as f64 / rows as f64;
                    let a = TAU * (i as f64 + d as f64 * (if j % 2 == 1 { 0.5 } else { 0.0 }))
                        / n as f64;
                    let r = r0 + (r1 - r0) * t;
                    pts.push((r * cos(a), y0 + (y1 - y0) * t, z + r * sin(a)));
                }
                fsweep!(F, format!("{name}{i}{d}"), &pts, 0.006, M.net, verts = 4);
            }
        }
        torus!(
            format!("{name}_hem"),
            r1,
            0.008,
            at = F.at(0, y1, z),
            material = M.net,
            major_segments = 16,
            minor_segments = 4
        );
    });
    join(&parts, name, None)
}

pub fn basketball_hoop() -> Vec<Obj> {
    let still = new_parts(|| {
        // the base: a moulded ballast tank, its fill cap, two wheels at its front edge for rolling it
        let p = F.at(0, 0, -0.17);
        let (bx, by) = (p.x, p.y);
        loft!(
            "ballast",
            &[
                ring!(0.31, 0.3, z = 0.0, verts = 24, square = 6, at = (bx, by)),
                ring!(0.31, 0.3, z = 0.18, verts = 24, square = 6, at = (bx, by)),
                ring!(0.28, 0.27, z = 0.22, verts = 24, square = 6, at = (bx, by)),
                ring!(0.2, 0.19, z = 0.235, verts = 24, square = 5, at = (bx, by))
            ],
            material = M.post,
            angle = 50
        );
        fcyl!(
            F,
            "ballast_cap",
            0.05,
            0.02,
            (0.16, 0.23, -0.05),
            M.steel,
            verts = 12,
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            fcyl!(
                F,
                format!("wheel{s}"),
                0.05,
                0.035,
                (s as f64 * 0.24, 0.05, 0.14),
                M.rubber,
                rot = (0, 90, 0),
                verts = 14,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        // the pole in two telescoping sections, a clamp collar between them; a cap on top
        fcyl!(
            F,
            "pole",
            0.05,
            1.85,
            (0, 0.21 + 0.925, -0.3),
            M.post,
            verts = 14
        );
        fcyl!(
            F,
            "pole_upper",
            0.044,
            1.36,
            (0, 2.0 + 0.68, -0.3),
            M.post,
            verts = 14
        );
        revolve!(
            "pole_collar",
            &[
                (0.0, -0.04),
                (0.058, -0.04),
                (0.062, -0.03),
                (0.062, 0.03),
                (0.055, 0.04),
                (0.0, 0.04)
            ],
            at = F.at(0, 2.02, -0.3),
            material = M.steel,
            segments = 14
        );
        fsweep!(
            F,
            "collar_knob",
            &[(0, 2.02, -0.24), (0, 2.02, -0.2)],
            0.012,
            M.rubber,
            verts = 6
        );
        fcyl!(
            F,
            "pole_cap",
            0.055,
            0.03,
            (0, 3.36, -0.3),
            M.steel,
            verts = 14,
            bevel = 0.006,
            bevel_segments = 1
        );
        // two bent arms from the pole to the board's back, a strut between them
        for (y, to) in [(3.3, 3.45), (3.0, 3.08)] {
            fsweep!(
                F,
                format!("arm{}", float_repr(y)),
                &kit::fillet!(&[(0.0, y, -0.3), (0.0, y, -0.2), (0.0, to, 0.0)], 0.04),
                0.025,
                M.post,
                verts = 8
            );
        }
        fsweep!(
            F,
            "arm_strut",
            &[(0, 3.06, -0.18), (0, 3.38, -0.12)],
            0.016,
            M.post,
            verts = 6
        );
        // the backboard: a white board in a steel frame, its border and the shooter's square, a pad along its foot
        let (bw, bh, by) = (BOARD_W, BOARD_H, BOARD_Y);
        fbox!(
            F,
            "board",
            bw - 0.02,
            bh - 0.02,
            0.025,
            (0, by, 0.04),
            M.board,
            bevel = 0.004,
            bevel_segments = 1
        );
        artkit::bevel::eased(kit::moulding!(
            F,
            "board_frame",
            &[
                (bw / 2.0, by - bh / 2.0),
                (bw / 2.0, by + bh / 2.0),
                (-bw / 2.0, by + bh / 2.0),
                (-bw / 2.0, by - bh / 2.0),
            ],
            &[(0.0, -0.02), (0.0, 0.016), (0.022, 0.016), (0.022, -0.02)],
            M.post,
            at = 0.04
        ));
        for y in [by + bh / 2.0 - 0.045, by - bh / 2.0 + 0.045] {
            fbox!(
                F,
                format!("border{}", float_repr(y)),
                bw - 0.07,
                0.03,
                0.028,
                (0, y, 0.04),
                M.board_mark
            );
        }
        for x in [-bw / 2.0 + 0.045, bw / 2.0 - 0.045] {
            fbox!(
                F,
                format!("border{}", float_repr(x)),
                0.03,
                bh - 0.06,
                0.028,
                (x, by, 0.04),
                M.board_mark
            );
        }
        let (tw, th) = (0.59, 0.45); // the shooter's square, 0.59 x 0.45 m as on a regulation board, its foot level with the rim
        let ty = RIM.0 + th / 2.0;
        fextrude!(
            F,
            "target",
            &rounded_rect(tw, th, 0.004, 1),
            0.028,
            (0, ty, 0.045),
            M.board_mark,
            holes = [rounded_rect(tw - 0.05, th - 0.05, 0.003, 1)]
        );
        soft!(
            F,
            "pad",
            bw,
            0.04,
            0.06,
            (0, by - bh / 2.0 - 0.01, 0.04),
            M.rubber,
            round = 0.015
        );
        // the rim on a flanged bracket with its spring housing, little hooks for the net under it
        fbox!(
            F,
            "rim_bracket",
            0.16,
            0.12,
            0.05,
            (0, RIM.0 - 0.04, 0.075),
            M.rim,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "rim_spring",
            0.06,
            0.07,
            0.04,
            (0, RIM.0 - 0.05, 0.1),
            M.steel,
            bevel = 0.006,
            bevel_segments = 1
        );
        fextrude!(
            F,
            "rim_flange",
            &[(0.06, 0.0), (-0.06, 0.0), (-0.03, 0.12), (0.03, 0.12)],
            0.006,
            (0, RIM.0 - 0.008, 0.06),
            M.rim,
            plane = Facing::Top
        );
        torus!(
            "rim",
            0.22,
            0.012,
            at = F.at(0, RIM.0, RIM.1),
            material = M.rim,
            major_segments = 28,
            minor_segments = 6
        );
        for k in 0..12 {
            let a = TAU * k as f64 / 12.0;
            let (x, z) = (0.215 * cos(a), RIM.1 + 0.215 * sin(a));
            fsweep!(
                F,
                format!("net_hook{k}"),
                &[
                    (x, RIM.0 - 0.005, z),
                    (x * 0.97, RIM.0 - 0.022, z - 0.0),
                    (x * 0.95, RIM.0 - 0.014, z)
                ],
                0.0025,
                M.rim,
                verts = 3
            );
        }
    });
    join(&still, "hoop", Some(P3::from((0, 0, 0))));
    // the ball, at rest on the ground in front: its seams. A part of its own, gone while someone plays (it is in
    // their hands: actions.ts `PROPS.basketball`).
    let ball_parts = new_parts(|| {
        fsphere!(
            F,
            "ball",
            0.12,
            (0.3, 0.12, 0.32),
            M.ball,
            segments = 16,
            rings = 10
        );
        for (i, rot) in [(90, 0, 30), (0, 0, 0), (90, 0, 120)]
            .into_iter()
            .enumerate()
        {
            torus!(
                format!("seam{i}"),
                0.1205,
                0.0035,
                at = F.at(0.3, 0.12, 0.32),
                material = M.rubber,
                rot = rot,
                major_segments = 20,
                minor_segments = 3
            );
        }
    });
    let ball = hinged(join(&ball_parts, "ball", None), F.at(0.3, 0.12, 0.32));
    let net = hinged(net("net"), F.at(0, RIM.0, RIM.1));
    settle!(centre = false);
    rock(
        net,
        "swish",
        9.0,
        Rock {
            seconds: 0.6,
            axis: Axis::X,
        },
    );
    squash(
        net,
        "swish",
        (0.92, 0.92, 1.18),
        Squash {
            seconds: 0.6,
            back: true,
            ..Default::default()
        },
    );
    squash(
        ball,
        "ball_taken",
        0.001,
        Squash {
            seconds: 0.2,
            ..Default::default()
        },
    );
    meshes()
}
