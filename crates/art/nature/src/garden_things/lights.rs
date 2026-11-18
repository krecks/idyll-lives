//! The garden lamp post and the string lights.

use super::common::{_near_light, _own, F};
use artkit::far::low;
use artkit::kit::fillet;
use artkit::math::{P3, PI, TAU, cos, cosd, p3, pow, sin, sind};
use artkit::motion::{Rock, rock};
use artkit::parts::{Settle, hinged, join, join_all, new_parts, numbered, settle_all, settle_with};
use artkit::py;
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{boulder, fbox, fcone, fcyl, fsphere, fsweep, leaf, loft, revolve, ring, torus};

// --- the garden lamp post (1x1): a lantern of frosted panes on an iron post; the panes glow at night -----------
/// The content's `light.height`: the panes' middle.
const LAMP_Y: f64 = 2.3;

#[allow(non_snake_case)]
pub fn lampPost() -> Vec<Obj> {
    let m = _own("lampPost");
    let (post, frame) = (m.body, m.lantern);
    let sq = pow(2.0, 0.5);
    // A square ring of half side `r` at game height `y`, turned to face the front: a loft section.
    let square = |y: f64, r: f64| -> Vec<P3> {
        [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
            .iter()
            .map(|&(x, z)| F.at(r * x, y, r * z))
            .collect()
    };
    // the base: a plinth, a torus, a bell and a waisted neck, turned in one piece
    revolve!(
        "base",
        &[
            (0.0, 0.0),
            (0.15, 0.0),
            (0.15, 0.022),
            (0.14, 0.03),
            (0.135, 0.045),
            (0.142, 0.06),
            (0.128, 0.075),
            (0.105, 0.09),
            (0.098, 0.13),
            (0.08, 0.18),
            (0.062, 0.24),
            (0.052, 0.3),
            (0.058, 0.315),
            (0.058, 0.335),
            (0.045, 0.345),
            (0.0, 0.345)
        ],
        at = F.at(0, 0, 0),
        material = post,
        segments = low(16, 8)
    );
    // the shaft: fluted, a plain neck and collars over it
    let flutes: Vec<(f64, f64)> = (0..20)
        .map(|k| {
            let r = if k % 2 == 0 { 0.035 } else { 0.03 };
            (r * cos(PI * k as f64 / 10.0), r * sin(PI * k as f64 / 10.0))
        })
        .collect();
    fsweep!(
        F,
        "pole",
        &[(0, 0.34, 0), (0, 1.75, 0)],
        0.0,
        post,
        profile = flutes,
        up = (0, 0, 1),
        angle = 60,
        scale = [1.08, 0.94]
    );
    fcyl!(F, "pole_neck", 0.031, 0.32, (0, 1.9, 0), post, verts = 12);
    for (y, r) in [(0.36, 0.05), (1.76, 0.042), (2.06, 0.05)] {
        revolve!(
            format!("collar{}", artkit::json::float_repr(y)),
            &[
                (0.0, -0.02),
                (r * 0.8, -0.02),
                (r, -0.01),
                (r, 0.01),
                (r * 0.8, 0.02),
                (0.0, 0.02)
            ],
            at = F.at(0, y, 0),
            material = post,
            segments = 12
        );
    }
    // the ladder bar a lamplighter leant his ladder on, its ends turned into balls
    fsweep!(
        F,
        "ladder_bar",
        &[(-0.13, 1.95, 0), (0.13, 1.95, 0)],
        0.009,
        post,
        verts = 8
    );
    for i in [-1, 1] {
        fsphere!(
            F,
            format!("bar_ball{i}"),
            0.017,
            (i as f64 * 0.135, 1.95, 0),
            post,
            segments = 10,
            rings = 6
        );
    }
    fcone!(F, "cup", 0.045, 0.1, 0.07, (0, 2.115, 0), post, verts = 12);
    // four scroll brackets from the collar out under the lantern's corners, each curling back in
    for k in 0..4 {
        let a = (45 + 90 * k) as f64;
        let (c, s) = (cosd(a), sind(a));
        let pts = [
            (0.035, 2.0),
            (0.07, 2.02),
            (0.105, 2.07),
            (0.11, 2.12),
            (0.095, 2.145),
            (0.08, 2.13),
            (0.085, 2.115),
        ];
        let path: Vec<(f64, f64, f64)> = pts.iter().map(|&(r, y)| (r * c, y, r * s)).collect();
        fsweep!(
            F,
            format!("bracket{k}"),
            &path,
            0.0065,
            post,
            verts = 5,
            resolution = 2,
            scale = [1.2, 1.0, 1.0, 1.0, 0.9, 0.8, 0.7]
        );
    }
    // the lantern: four frosted panes, wider at the top, in a frame of corner bars and bands under a flared roof
    fcone!(
        F,
        "panes",
        0.09 * sq,
        0.125 * sq,
        0.32,
        (0, LAMP_Y, 0),
        m.pane,
        verts = 4,
        rot = (0, 0, 45),
        smooth = false
    );
    loft!(
        "lantern_floor",
        &[
            square(LAMP_Y - 0.19, 0.08),
            square(LAMP_Y - 0.17, 0.112),
            square(LAMP_Y - 0.155, 0.112),
            square(LAMP_Y - 0.15, 0.1)
        ],
        material = frame,
        smooth = false
    );
    for k in 0..4 {
        let a = (45 + 90 * k) as f64;
        let (lo, hi) = (0.094 * sq, 0.129 * sq);
        fsweep!(
            F,
            format!("frame{k}"),
            &[
                (lo * cosd(a), LAMP_Y - 0.16, lo * sind(a)),
                (hi * cosd(a), LAMP_Y + 0.16, hi * sind(a))
            ],
            0.0,
            frame,
            profile = [
                (-0.008, -0.008),
                (0.008, -0.008),
                (0.008, 0.008),
                (-0.008, 0.008)
            ],
            up = (0, 0, 1)
        );
    }
    // a band round the panes, two thirds up
    let (y, r) = (LAMP_Y + 0.07, 0.117);
    loft!(
        "band",
        &[square(y - 0.008, r), square(y + 0.008, r + 0.003)],
        material = frame,
        smooth = false
    );
    // the roof: an eave lip, a concave pyramid flaring out over the panes, a vent drum, a crown and a spike
    loft!(
        "roof",
        &[
            square(LAMP_Y + 0.155, 0.13),
            square(LAMP_Y + 0.168, 0.16),
            square(LAMP_Y + 0.185, 0.16),
            square(LAMP_Y + 0.215, 0.12),
            square(LAMP_Y + 0.26, 0.075),
            square(LAMP_Y + 0.31, 0.045),
            square(LAMP_Y + 0.335, 0.03)
        ],
        material = frame,
        smooth = false
    );
    revolve!(
        "crown",
        &[
            (0.0, 0.0),
            (0.04, 0.0),
            (0.042, 0.035),
            (0.032, 0.045),
            (0.022, 0.05),
            (0.024, 0.07),
            (0.014, 0.085),
            (0.006, 0.12),
            (0.0, 0.135)
        ],
        at = F.at(0, LAMP_Y + 0.3, 0),
        material = frame,
        segments = low(12, 6)
    );
    _near_light("lampPost", LAMP_Y);
    join_all("lampPost");
    settle_all();
    meshes()
}

// --- the string lights (2x1): festoon bulbs on a sagging wire between two posts in weighted pots ---------------
const POST_X: f64 = 0.86;
/// The wire's height at the posts, and how far it sags in the middle.
const WIRE_Y: f64 = 2.27;
const SAG: f64 = 0.28;
const BULBS: usize = 9;

fn _wire_y(x: f64) -> f64 {
    WIRE_Y - SAG * (1.0 - pow(x / (POST_X - 0.04), 2.0))
}

#[allow(non_snake_case)]
pub fn stringLights() -> Vec<Obj> {
    let m = _own("stringLights");
    let mut rng = Random::new(23);
    let still = new_parts(|| {
        for i in [-1, 1] {
            let s = i as f64;
            let x = s * POST_X;
            // a tapered pot with a rolled rim, gravel over its soil, a few leaves of ivy at the post's foot
            revolve!(
                format!("pot{i}"),
                &[
                    (0.0, 0.0),
                    (0.095, 0.0),
                    (0.1, 0.012),
                    (0.117, 0.25),
                    (0.125, 0.262),
                    (0.127, 0.29),
                    (0.121, 0.3),
                    (0.108, 0.3),
                    (0.104, 0.28),
                    (0.0, 0.28)
                ],
                at = F.at(x, 0, 0),
                material = m.planter,
                segments = 18
            );
            fcyl!(
                F,
                format!("pot_soil{i}"),
                0.104,
                0.01,
                (x, 0.275, 0),
                m.soil,
                verts = 14
            );
            for k in 0..5 {
                let a = rng.uniform(0.0, TAU);
                let r = rng.uniform(0.05, 0.085);
                boulder!(
                    format!("gravel{i}{k}"),
                    0.014,
                    at = F.at(x + r * cos(a), 0.279, r * sin(a)),
                    material = m.pebble,
                    scale = (1.2, 1, 0.6),
                    seed = k as f64 * 1.7 + s,
                    noise = 0.3,
                    subdivisions = 1
                );
            }
            for k in 0..4 {
                let a = (90 * k + 45) as f64 + rng.uniform(-20.0, 20.0);
                let tilt = rng.uniform(25.0, 45.0);
                leaf!(
                    format!("ivy{i}{k}"),
                    F.at(x + 0.04 * cosd(a), 0.28, 0.04 * sind(a)),
                    -a + 90.0,
                    tilt,
                    0.07,
                    0.045,
                    m.reed,
                    shape = "oval",
                    droop = 0.25,
                    thick = 0.004,
                    fold = 0.15,
                    rows = 3,
                    ring = 3
                );
            }
            // the post: chamfered, a pyramid cap on a collar, a steel eye screwed in for the wire, a hook below it
            fbox!(
                F,
                format!("post{i}"),
                0.07,
                2.12,
                0.07,
                (x, 0.2 + 1.06, 0),
                m.body,
                bevel = 0.008,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("collar{i}"),
                0.082,
                0.02,
                0.082,
                (x, 2.3, 0),
                m.body,
                bevel = 0.004,
                bevel_segments = 1
            );
            fcone!(
                F,
                format!("cap{i}"),
                0.058,
                0.008,
                0.06,
                (x, 2.34, 0),
                m.body,
                verts = 4,
                rot = (0, 0, 45),
                smooth = false
            );
            torus!(
                format!("eye{i}"),
                0.018,
                0.004,
                at = F.at(x - s * 0.045, WIRE_Y, 0),
                material = m.wire,
                rot = (90, 0, 0),
                major_segments = 10,
                minor_segments = 4
            );
            fsweep!(
                F,
                format!("eye_shank{i}"),
                &[
                    (x - s * 0.035, WIRE_Y - 0.018, 0),
                    (x - s * 0.02, WIRE_Y - 0.018, 0)
                ],
                0.003,
                m.wire,
                verts = 4
            );
            let hook = fillet(
                &[
                    (x - s * 0.035, 1.2, 0.0),
                    (x - s * 0.06, 1.2, 0.0),
                    (x - s * 0.06, 1.24, 0.0),
                ],
                0.01,
                3,
            );
            fsweep!(F, format!("hook{i}"), &hook, 0.004, m.wire, verts = 5);
        }
    });
    join(&still, "posts", Some(p3(0, 0, 0)));
    let string = new_parts(|| {
        let x0 = POST_X - 0.04;
        let n = 24;
        // the wire in one piece, sagging between the eyes, a drop to each socket
        let wire: Vec<(f64, f64, i32)> = (0..=n)
            .map(|i| {
                let x = -x0 + 2.0 * x0 * i as f64 / n as f64;
                (x, _wire_y(x), 0)
            })
            .collect();
        fsweep!(F, "wire", &wire, 0.0045, m.wire, verts = 5);
        let mut heights = Vec::new();
        for i in 0..BULBS {
            let x = -x0 + 2.0 * x0 * (i as f64 + 0.5) / BULBS as f64;
            let y = _wire_y(x);
            // the socket: a clip round the wire and a ribbed holder; the bulb a festoon globe on a short neck
            let b = F.at(x, 0, 0);
            let socket: Vec<Vec<P3>> = [
                (-0.05, 0.011),
                (-0.042, 0.014),
                (-0.03, 0.014),
                (-0.026, 0.012),
                (-0.016, 0.012),
                (-0.008, 0.008),
                (0.002, 0.006),
            ]
            .iter()
            .map(|&(dy, r)| ring!(r, r, z = y + dy, verts = 6, at = (b.x, b.y)))
            .collect();
            loft!(numbered("socket"), &socket, material = m.socket);
            let bulb: Vec<Vec<P3>> = [
                (-0.05, 0.009),
                (-0.058, 0.012),
                (-0.068, 0.026),
                (-0.085, 0.035),
                (-0.105, 0.034),
                (-0.122, 0.024),
                (-0.13, 0.01),
                (-0.133, 0.003),
            ]
            .iter()
            .map(|&(dy, r)| ring!(r, r, z = y + dy, verts = 8, at = (b.x, b.y)))
            .collect();
            loft!(numbered("bulb"), &bulb, material = m.bulb);
            heights.push(y - 0.09);
        }
        let n = heights.len() as f64;
        _near_light("stringLights", py::sum(heights) / n);
    });
    let string = hinged(join(&string, "string", None), F.at(0, WIRE_Y, 0));
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    rock(
        string,
        "sway",
        4.0,
        Rock {
            seconds: 3.2,
            ..Rock::default()
        },
    );
    meshes()
}
