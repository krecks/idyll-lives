//! The street lamp and the traffic light: cast-iron posts.

use super::common::{fluted, own, square};
use artkit::far::{is_low, low};
use artkit::forms::rounded_rect;
use artkit::math::{cos, cosd, pow, radians, sin, sind};
use artkit::parts::{join_all, settle_all};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{
    fbox, fcone, fcyl, fextrude, fsphere, fsweep, kit_scroll, loft, revolve, ring, settle,
};

// --- the street lamp: a cast-iron post with a square lantern ------------------------------------------------
// (the content's `light.height` is 3.5: the lantern's panes run from 3.46 to 3.88 m)

pub fn street_lamp() -> Vec<Obj> {
    let m = own("streetLamp");
    let f = GameFrame::new(0.0);
    let iron = m.body;
    {
        // the base: a square plinth with a chamfer, a fluted bell rising to a moulded collar
        loft!(
            "plinth",
            &[
                ring!(0.19, 0.19, z = 0.0, verts = 24, square = 10),
                ring!(0.19, 0.19, z = 0.035, verts = 24, square = 10),
                ring!(0.172, 0.172, z = 0.06, verts = 24, square = 10),
                ring!(0.16, 0.16, z = 0.075, verts = 24, square = 4)
            ],
            material = iron
        );
        let bell = [
            (0.158, 0.075),
            (0.152, 0.12),
            (0.135, 0.2),
            (0.112, 0.3),
            (0.095, 0.4),
            (0.086, 0.5),
        ];
        let secs: Vec<_> = bell
            .iter()
            .map(|&(r, y)| fluted(r, y, 12, 0.07, 3, 0.0))
            .collect();
        loft!("bell", &secs, material = iron);
        revolve!(
            "collar",
            &[
                (0.0, 0.495),
                (0.098, 0.5),
                (0.104, 0.515),
                (0.092, 0.53),
                (0.082, 0.55),
                (0.09, 0.565),
                (0.07, 0.58),
                (0.0, 0.58)
            ],
            material = iron,
            segments = 16
        );
        // the post: fluted, a collar part way, a moulded capital under the lantern's bracket
        loft!(
            "post",
            &[
                fluted(0.047, 0.575, 12, 0.1, 3, 0.0),
                fluted(0.044, 2.93, 12, 0.1, 3, 0.0)
            ],
            material = iron
        );
        revolve!(
            "collar2",
            &[
                (0.0, 2.92),
                (0.06, 2.925),
                (0.066, 2.945),
                (0.056, 2.965),
                (0.0, 2.965)
            ],
            material = iron,
            segments = 14
        );
        fcyl!(f, "neck", 0.042, 0.42, (0, 3.17, 0), iron, verts = 12);
        // the ladder bar a lamplighter leant on, ball ends
        fsweep!(
            f,
            "ladder_bar",
            &[(-0.29, 3.12, 0), (0.29, 3.12, 0)],
            0.016,
            iron,
            verts = 8
        );
        for s in [-1, 1] {
            fsphere!(
                f,
                format!("ladder_ball{s}"),
                0.0285,
                (s as f64 * 0.3, 3.12, 0),
                iron,
                segments = 10,
                rings = 6
            );
        }
        // four scrolled brackets from the capital out under the lantern's floor
        for k in 0..4 {
            let arm = kit_scroll!(
                f,
                format!("bracket{k}"),
                (0, 3.43, 0.045),
                (3.36, 0.1),
                iron,
                turns = -0.9,
                end = 0.35,
                t = 0.012,
                w = 0.022,
                steps = 10
            );
            arm.set_rotation_euler_axis(2, radians((90 * k) as f64));
        }
        revolve!(
            "capital",
            &[
                (0.0, 3.28),
                (0.05, 3.3),
                (0.062, 3.33),
                (0.075, 3.38),
                (0.11, 3.43),
                (0.11, 3.445),
                (0.0, 3.445)
            ],
            material = iron,
            segments = 16
        );
        // the lantern: four frosted panes, wider at the top, in an iron frame
        let sq = pow(2.0, 0.5);
        fcone!(
            f,
            "panes",
            0.13 * sq,
            0.19 * sq,
            0.42,
            (0, 3.67, 0),
            m.lantern,
            verts = 4,
            rot = (0, 0, 45),
            smooth = false
        );
        loft!(
            "lantern_floor",
            &[
                square(0.12, 3.43),
                square(0.155, 3.455),
                square(0.155, 3.475)
            ],
            material = iron,
            smooth = false
        );
        loft!(
            "lantern_top",
            &[
                square(0.2, 3.875),
                square(0.212, 3.885),
                square(0.212, 3.91)
            ],
            material = iron,
            smooth = false
        );
        for k in 0..4 {
            let a = (45 + 90 * k) as f64;
            let (lo, hi) = (0.137 * sq, 0.197 * sq);
            fsweep!(
                f,
                format!("frame{k}"),
                &[
                    (lo * cosd(a), 3.46, lo * sind(a)),
                    (hi * cosd(a), 3.885, hi * sind(a))
                ],
                0.013,
                iron,
                verts = 6
            );
        }
        // the roof: a hipped canopy, its sides curving in, a vented crown and a finial
        loft!(
            "roof",
            &[
                square(0.233, 3.905),
                square(0.233, 3.925),
                square(0.2, 3.95),
                square(0.14, 3.99),
                square(0.085, 4.04),
                square(0.06, 4.08)
            ],
            material = iron,
            smooth = false
        );
        revolve!(
            "vent",
            &[
                (0.0, 4.07),
                (0.065, 4.075),
                (0.065, 4.09),
                (0.05, 4.1),
                (0.05, 4.13),
                (0.07, 4.14),
                (0.07, 4.152),
                (0.03, 4.165),
                (0.0, 4.165)
            ],
            material = iron,
            segments = 12
        );
        revolve!(
            "finial",
            &[
                (0.0, 4.16),
                (0.014, 4.165),
                (0.012, 4.18),
                (0.032, 4.2),
                (0.026, 4.225),
                (0.008, 4.236),
                (0.0, 4.24)
            ],
            material = iron,
            segments = 10
        );
    }
    join_all("streetLamp");
    settle_all();
    meshes()
}

// --- the traffic light: a cast-iron pole with a three-lamp head facing the traffic coming in ---------------------
// Its pole's axis is the origin (the snapshot's pole, sim-core `traffic::SignalArm`), its head in front of the pole
// facing +z (towards those coming in). The three lenses are each a material of their own (`lens_red`,
// `lens_amber`, `lens_green`), dark glass with the colour they glow (`glow`): the renderer
// (render/pc/signals.ts) draws each lens's glow apart, only the one the snapshot's light shows.
const SIGNAL_FRONT: f64 = 0.27; // the head's front face (m in front of the pole's axis)

pub fn traffic_light() -> Vec<Obj> {
    let m = own("trafficLight");
    let f = GameFrame::new(0.0);
    let iron = m.body;
    // each lens's middle (m up)
    let signal_lenses = [
        ("red", 2.81, m.lens_red),
        ("amber", 2.52, m.lens_amber),
        ("green", 2.23, m.lens_green),
    ];
    let n = low(16, 8);
    {
        // the base: a chamfered square plinth, a short fluted bell rising to a moulded collar (the lamp's, smaller)
        loft!(
            "plinth",
            &[
                ring!(0.13, 0.13, z = 0.0, verts = low(24, 8), square = 10),
                ring!(0.13, 0.13, z = 0.03, verts = low(24, 8), square = 10),
                ring!(0.115, 0.115, z = 0.05, verts = low(24, 8), square = 10),
                ring!(0.105, 0.105, z = 0.062, verts = low(24, 8), square = 4)
            ],
            material = iron
        );
        let bell = [
            (0.104, 0.06),
            (0.098, 0.1),
            (0.084, 0.18),
            (0.07, 0.28),
            (0.062, 0.36),
        ];
        if is_low() {
            loft!(
                "bell",
                &[square(0.07, 0.06), square(0.05, 0.36)],
                material = iron,
                smooth = false
            );
        } else {
            let secs: Vec<_> = bell
                .iter()
                .map(|&(r, y)| fluted(r, y, 10, 0.07, 3, 0.0))
                .collect();
            loft!("bell", &secs, material = iron);
        }
        revolve!(
            "collar",
            &[
                (0.0, 0.355),
                (0.07, 0.36),
                (0.075, 0.375),
                (0.064, 0.39),
                (0.056, 0.405),
                (0.0, 0.405)
            ],
            material = iron,
            segments = n
        );
        // the pole: fluted as the lamp's, behind the head, up to a turned cap over it
        if is_low() {
            fcyl!(f, "pole", 0.045, 2.62, (0, 1.71, 0), iron, verts = 6);
        } else {
            loft!(
                "pole",
                &[
                    fluted(0.046, 0.4, 10, 0.1, 3, 0.0),
                    fluted(0.043, 3.02, 10, 0.1, 3, 0.0)
                ],
                material = iron
            );
        }
        revolve!(
            "cap",
            &[
                (0.0, 3.01),
                (0.056, 3.015),
                (0.06, 3.03),
                (0.045, 3.045),
                (0.02, 3.07),
                (0.008, 3.09),
                (0.0, 3.095)
            ],
            material = iron,
            segments = n
        );
        // the head's two clamps round the pole, each with a lug out to the head's back
        for (k, y) in [2.2, 2.84].into_iter().enumerate() {
            fcyl!(
                f,
                format!("clamp{k}"),
                0.056,
                0.05,
                (0, y, 0),
                iron,
                verts = n,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
            fbox!(f, format!("lug{k}"), 0.07, 0.05, 0.04, (0, y, 0.06), iron);
        }
        // the head: a housing with rounded edges, a black backboard with a white rim behind it
        fbox!(
            f,
            "housing",
            0.3,
            0.9,
            0.19,
            (0, 2.52, SIGNAL_FRONT - 0.095),
            iron,
            bevel = low(0.025, 0.0),
            bevel_segments = 2
        );
        let board = (0, 2.52, 0.085);
        fextrude!(
            f,
            "rim",
            &rounded_rect(0.52, 1.1, 0.06, low(4, 1)),
            0.012,
            board,
            m.paper
        );
        fextrude!(
            f,
            "board",
            &rounded_rect(0.47, 1.05, 0.04, low(4, 1)),
            0.016,
            board,
            m.board
        );
        // three lenses, each in its bezel under a hood open below
        for (colour, y, lens) in signal_lenses {
            fcyl!(
                f,
                format!("lens_{colour}"),
                0.1,
                0.02,
                (0, y, SIGNAL_FRONT + 0.005),
                lens,
                verts = low(24, 10),
                rot = (90, 0, 0)
            );
            revolve!(
                format!("bezel_{colour}"),
                &[
                    (0.1, -0.014),
                    (0.118, -0.014),
                    (0.12, 0.0),
                    (0.118, 0.014),
                    (0.1, 0.014)
                ],
                at = f.at(0, y, SIGNAL_FRONT + 0.004),
                material = m.board,
                segments = low(24, 10),
                rot = (90, 0, 0)
            );
            if is_low() {
                fbox!(
                    f,
                    format!("hood_{colour}"),
                    0.24,
                    0.012,
                    0.15,
                    (0, y + 0.118, SIGNAL_FRONT + 0.085),
                    m.board
                );
            } else {
                let arc: Vec<f64> = (0..15)
                    .map(|i| radians(-28.0 + (236 * i) as f64 / 14.0))
                    .collect();
                let hood: Vec<(f64, f64)> = arc
                    .iter()
                    .map(|&a| (0.124 * cos(a), 0.124 * sin(a)))
                    .chain(arc.iter().rev().map(|&a| (0.113 * cos(a), 0.113 * sin(a))))
                    .collect();
                fextrude!(
                    f,
                    format!("hood_{colour}"),
                    &hood,
                    0.15,
                    (0, y, SIGNAL_FRONT + 0.085),
                    m.board
                );
            }
        }
    }
    join_all("trafficLight");
    // (Not centred: its pole's axis stays on the origin, where the snapshot stands it.)
    settle!(centre = false);
    meshes()
}
