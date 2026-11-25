//! The window-seat reading nook and the rare-books library wall.

use super::common::{M, WALL, books, knob, mug};
use artkit::forms::rounded_rect;
use artkit::math::{P3, cos, cosd, pow, radians, sin, sind};
use artkit::motion::{Squash, squash};
use artkit::parts::{hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{
    boulder, fbox, fcyl, fextrude, fsphere, fsweep, jitter, kit, loft, revolve, ring, settle,
    slide, soft, torus,
};

// --- the window-seat reading nook (2x1): a cushioned seat between two little bookcases ---------------------

/// A little open bookcase centred on `x`, its back on the wall: sides with a rounded front edge, a back,
/// shelves with a lipped front, books on each.
#[allow(clippy::too_many_arguments)]
fn case(
    f: &GameFrame,
    rnd: &mut Random,
    x: f64,
    top: f64,
    shelves: &[f64],
    fills: &[f64],
    stacks: &[usize],
    lean: &[usize],
) {
    for dx in [-0.195, 0.195] {
        fbox!(
            f,
            numbered("side"),
            0.03,
            top,
            0.4,
            (x + dx, top / 2.0, WALL + 0.2),
            M.body,
            bevel = 0.006,
            bevel_segments = 1
        );
    }
    fbox!(
        f,
        numbered("back"),
        0.36,
        top - 0.06,
        0.012,
        (x, top / 2.0, WALL + 0.006),
        M.back
    );
    for (k, &y) in shelves.iter().enumerate() {
        fbox!(
            f,
            numbered("shelf"),
            0.36,
            0.025,
            0.38,
            (x, y, WALL + 0.2),
            M.body,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            numbered("shelf_lip"),
            0.36,
            0.035,
            0.012,
            (x, y + 0.005, WALL + 0.392),
            M.panel
        );
        if k < fills.len() {
            books(
                f,
                rnd,
                x - 0.175,
                x + 0.175,
                y + 0.0125,
                WALL + 0.02,
                0.24,
                if k < 3 { 0.3 } else { 0.22 },
                fills[k],
                stacks.contains(&k),
                lean.contains(&k),
            );
        }
    }
}

pub fn reading_nook() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(3);
    let still = new_parts(|| {
        for s in [-1.0, 1.0] {
            case(
                f,
                &mut rnd,
                s * 0.73,
                1.53,
                &[0.04, 0.42, 0.8, 1.18],
                &[0.9, 0.62, 0.95, 0.5],
                &[1, 3],
                if s > 0.0 { &[2] } else { &[] },
            );
            fbox!(
                f,
                numbered("case_kick"),
                0.36,
                0.035,
                0.012,
                (s * 0.73, 0.0175, WALL + 0.39),
                M.panel
            );
        }
        mug(f, 0.64, 0.4325, -0.15, M.mug_a, None);
        // the header over the seat and cases: a frieze, and a cornice moulding run round its front and ends
        fbox!(
            f,
            "header",
            1.82,
            0.12,
            0.38,
            (0, 1.59, WALL + 0.19),
            M.body,
            bevel = 0.006,
            bevel_segments = 1
        );
        kit::moulding!(
            f,
            "cornice",
            &[
                (-0.91, WALL + 0.03),
                (0.91, WALL + 0.03),
                (0.91, WALL + 0.39),
                (-0.91, WALL + 0.39),
            ],
            &[
                (0.02, 0.0),
                (-0.004, 0.0),
                (-0.004, 0.018),
                (-0.016, 0.03),
                (-0.024, 0.05),
                (-0.03, 0.066),
                (-0.03, 0.086), // (its top's arrises chamfered 4 mm)
                (-0.026, 0.09),
                (0.016, 0.09),
                (0.02, 0.086),
            ],
            M.body,
            at = 1.62,
            plane = Facing::Top
        );
        fbox!(
            f,
            "header_bead",
            1.82,
            0.016,
            0.01,
            (0, 1.545, WALL + 0.385),
            M.panel,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the alcove's ceiling and its valance: a board with an arched, stepped lower edge
        fbox!(
            f,
            "alcove_top",
            1.06,
            0.02,
            0.4,
            (0, 1.52, WALL + 0.2),
            M.body
        );
        let edge: Vec<(f64, f64)> = (0..17)
            .map(|i| -0.53 + 1.06 * i as f64 / 16.0)
            .map(|x| (x, 1.4 + 0.07 * pow(1.0 - pow(x / 0.53, 2.0), 0.6)))
            .collect();
        let outline = [vec![(0.53, 1.53), (-0.53, 1.53)], edge.clone()].concat();
        fextrude!(
            f,
            "valance",
            &outline,
            0.025,
            (0, 0, -0.045),
            M.body,
            bevel = 0.004,
            bevel_segments = 1
        );
        let bead: Vec<(f64, f64, f64)> = edge[1..edge.len() - 1]
            .iter()
            .map(|&(x, y)| (x, y - 0.006, -0.03))
            .collect();
        fsweep!(f, "valance_bead", &bead, 0.006, M.panel, verts = 4);
        // a pendant reading light: a cord from a rose to a domed brass shade with a rolled rim and its bulb
        fcyl!(
            f,
            "pendant_rose",
            0.035,
            0.012,
            (0, 1.504, -0.12),
            M.brass,
            verts = 12,
            bevel = 0.003,
            bevel_segments = 1
        );
        kit::tube!(
            f,
            "pendant_cord",
            &[(0.0, 1.5, -0.12), (0.0, 1.27, -0.12)],
            0.003,
            M.dark,
            verts = 5,
            round = 1
        );
        revolve!(
            "pendant",
            &[
                (0.11, 0.0),
                (0.105, 0.012),
                (0.08, 0.05),
                (0.045, 0.085),
                (0.02, 0.105),
                (0.013, 0.127),
                (0.009, 0.13),
                (0.0, 0.13),
                (0.0, 0.12),
                (0.016, 0.1),
                (0.04, 0.08),
                (0.075, 0.045),
                (0.1, 0.008)
            ],
            at = f.at(0, 1.15, -0.12),
            material = M.lamp_amber,
            segments = 14
        );
        torus!(
            "pendant_rim",
            0.108,
            0.005,
            at = f.at(0, 1.152, -0.12),
            material = M.brass,
            major_segments = 20,
            minor_segments = 4
        );
        fsphere!(
            f,
            "pendant_bulb",
            0.03,
            (0, 1.165, -0.12),
            M.page,
            segments = 10,
            rings = 6
        );
        // the back of the seat: two panels (mostly behind the cushions)
        fbox!(
            f,
            "seat_back",
            1.04,
            1.02,
            0.02,
            (0, 0.96, WALL + 0.01),
            M.back
        );
        for x in [-0.26, 0.26] {
            fbox!(
                f,
                numbered("seat_back_panel"),
                0.44,
                0.5,
                0.012,
                (x, 1.12, WALL + 0.026),
                M.panel,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // the seat box with two frame-and-panel drawers and turned knobs, on a recessed kick
        fbox!(
            f,
            "seat_box",
            1.04,
            0.33,
            0.6,
            (0, 0.195, WALL + 0.3),
            M.body,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "seat_lip",
            1.06,
            0.03,
            0.62,
            (0, 0.345, WALL + 0.31),
            M.body,
            bevel = 0.008,
            bevel_segments = 1
        );
        for x in [-0.25, 0.25] {
            kit::shaker!(
                f,
                numbered("drawer"),
                0.46,
                0.22,
                (x, 0.2, 0.18),
                0.02,
                M.panel,
                rail = 0.045
            );
            knob(f, &numbered("knob"), x, 0.2, 0.19, 0.013, None, 8);
        }
        fbox!(
            f,
            "plinth",
            1.0,
            0.03,
            0.02,
            (0, 0.015, 0.155),
            M.panel,
            bevel = 0.004,
            bevel_segments = 1
        );
        // back cushions, stuffed and piped, and two throw pillows
        for x in [-0.25, 0.25] {
            soft!(
                f,
                numbered("back_cushion"),
                0.48,
                0.42,
                0.13,
                (x, 0.68, -0.33),
                M.cushion,
                rot = (-8, 0, 0),
                round = 0.045,
                puff = 0.03,
                piping = 0.005,
                flat = 2,
                resting = true
            );
        }
        soft!(
            f,
            "pillow_l",
            0.3,
            0.28,
            0.1,
            (-0.38, 0.6, -0.22),
            M.pillow_a,
            rot = (-12, -6, 30),
            round = 0.04,
            puff = 0.035,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
        soft!(
            f,
            "pillow_r",
            0.28,
            0.26,
            0.1,
            (0.38, 0.6, -0.23),
            M.pillow_b,
            rot = (-12, 6, -28),
            round = 0.04,
            puff = 0.035,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The seat cushion and the throw folded over it, joined with their origin on the underside: `sink_in`
    // presses them 2 cm while a resident is curled up on it.
    let seat = new_parts(|| {
        soft!(
            f,
            "seat_cushion",
            1.02,
            0.078,
            0.6,
            (0, 0.399, -0.125),
            M.fabric,
            round = 0.03,
            puff = 0.012,
            piping = 0.005,
            flat = 3,
            resting = true
        );
        let (t, z0, z1, x) = (0.012, -0.4, 0.172, 0.28);
        let y = 0.45 + t / 2.0;
        let pts = [
            (x, y, z0),
            (x, y + 0.004, (z0 + z1) / 2.0),
            (x, y + 0.002, z1 - 0.03),
            (x, y - 0.012, z1 + 0.002),
            (x, y - 0.04, z1 + t / 2.0 + 0.006),
            (x, y - 0.15, z1 + t / 2.0 + 0.008),
        ];
        let throw = fsweep!(
            f,
            "throw",
            &pts,
            0,
            M.throw,
            profile = rounded_rect(0.42, t, t / 2.0, 1),
            up = (1, 0, 0),
            resolution = 3
        );
        jitter!(throw, 0.004, size = 0.12, seed = 2.0, up = 0.6);
    });
    let seat = hinged(join(&seat, "seat", None), f.at(0, 0.36, -0.125));
    let objs = settle!(centre = false);
    squash(
        seat,
        "sink_in",
        (1, 1, 0.78),
        Squash {
            seconds: 0.6,
            ..Squash::default()
        },
    );
    objs
}

// --- the rare-books library wall (2x1): floor-to-ceiling shelves, a brass rail and a rolling ladder ---------
const LADDER_TOP: (f64, f64) = (1.98, -0.02); // (height, depth) of the rails' top
const LADDER_FOOT: (f64, f64) = (0.02, 0.36); // and foot

/// (height, depth) a fraction `t` down the ladder from its top.
fn ladder_at(t: f64) -> (f64, f64) {
    (
        LADDER_TOP.0 + (LADDER_FOOT.0 - LADDER_TOP.0) * t,
        LADDER_TOP.1 + (LADDER_FOOT.1 - LADDER_TOP.1) * t,
    )
}

/// A little marble bust on `y`: a turned socle, sloping shoulders, a neck and a head with hair.
fn bust(f: &GameFrame, x: f64, y: f64, z: f64, s: f64) {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = f.at(x, y, z);
    revolve!(
        numbered("bust_socle"),
        &[
            (0.0, 0.0),
            (0.035 * s, 0.0),
            (0.035 * s, 0.008 * s),
            (0.022 * s, 0.02 * s),
            (0.026 * s, 0.03 * s),
            (0.0, 0.03 * s)
        ],
        at = (bx, by, bz),
        material = M.bust,
        segments = 12
    );
    let chest: Vec<Vec<P3>> = [
        (0.028, 0.03, 0.024, 2.0),
        (0.04, 0.06, 0.03, 3.0),
        (0.08, 0.065, 0.034, 3.4),
        (0.1, 0.056, 0.03, 2.8),
        (0.11, 0.03, 0.022, 2.0),
        (0.112, 0.02, 0.018, 2.0),
    ]
    .iter()
    .map(|&(h, rx, ry, sq)| {
        ring!(
            rx * s,
            ry * s,
            z = bz + h * s,
            verts = 14,
            square = sq,
            at = (bx, by)
        )
    })
    .collect();
    loft!(numbered("bust_chest"), &chest, material = M.bust);
    let head: Vec<Vec<P3>> = [
        (0.11, 0.016, 0.017, 0.0),
        (0.13, 0.017, 0.018, -0.002),
        (0.14, 0.026, 0.03, -0.006),
        (0.16, 0.03, 0.036, -0.002),
        (0.18, 0.03, 0.035, 0.0),
        (0.195, 0.022, 0.026, 0.002),
        (0.202, 0.004, 0.004, 0.002),
    ]
    .iter()
    .map(|&(h, rx, ry, cy)| {
        ring!(
            rx * s,
            ry * s,
            z = bz + h * s,
            verts = 12,
            square = 2.2,
            at = (bx, by + cy * s)
        )
    })
    .collect();
    loft!(numbered("bust_head"), &head, material = M.bust);
    let hair = boulder!(
        numbered("bust_hair"),
        0.034 * s,
        at = (bx, by + 0.006 * s, bz + 0.172 * s),
        material = M.bust,
        scale = (0.95, 1.05, 0.85),
        seed = 2.0,
        noise = 0.15,
        subdivisions = 2
    );
    jitter!(hair, 0.002 * s, size = 0.01, seed = 1.0);
}

/// A desk globe on `y`: a turned foot and stem, the ball tilted on its axis, land on it, a brass half meridian.
fn globe(f: &GameFrame, x: f64, y: f64, z: f64) {
    revolve!(
        "globe_foot",
        &[
            (0.0, 0.0),
            (0.06, 0.0),
            (0.06, 0.008),
            (0.045, 0.016),
            (0.014, 0.03),
            (0.01, 0.06),
            (0.0, 0.06)
        ],
        at = f.at(x, y, z),
        material = M.brass,
        segments = 16
    );
    let c = (x, y + 0.17, z);
    fsphere!(f, "globe", 0.1, c, M.globe, segments = 16, rings = 10);
    for (a, b, r) in [
        (30.0, 20.0, 0.04),
        (160.0, -15.0, 0.035),
        (250.0, 35.0, 0.03),
    ] {
        fsphere!(
            f,
            numbered("land"),
            r,
            (
                c.0 + 0.088 * cosd(a) * cosd(b),
                c.1 + 0.088 * sind(b),
                c.2 + 0.088 * sind(a) * cosd(b)
            ),
            M.land,
            scale = (1, 0.45, 1.3),
            rot = (b, 0.0, -a),
            segments = 8,
            rings = 5
        );
    }
    let tilt = 23.5;
    let pts: Vec<(f64, f64, f64)> = (-180..1)
        .step_by(15)
        .map(|a| {
            (
                c.0 + 0.115 * sin(radians(a as f64 + tilt)),
                c.1 + 0.115 * cos(radians(a as f64 + tilt)),
                c.2,
            )
        })
        .collect();
    fsweep!(
        f,
        "globe_meridian",
        &pts,
        0,
        M.brass,
        profile = rounded_rect(0.012, 0.004, 0.0015, 1),
        up = (0, 1, 0)
    );
}

pub fn library_wall() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(11);
    let still = new_parts(|| {
        // the cornice: a frieze under a crown moulding run round its front and ends; a moulded plinth
        fbox!(
            f,
            "frieze",
            1.82,
            0.1,
            0.38,
            (0, 2.22, WALL + 0.19),
            M.panel,
            bevel = 0.004,
            bevel_segments = 1
        );
        kit::moulding!(
            f,
            "cornice",
            &[
                (-0.91, WALL + 0.03),
                (0.91, WALL + 0.03),
                (0.91, WALL + 0.39),
                (-0.91, WALL + 0.39),
            ],
            &[
                (0.03, 0.0),
                (-0.004, 0.0),
                (-0.004, 0.012),
                (-0.012, 0.022),
                (-0.022, 0.04),
                (-0.03, 0.058),
                (-0.03, 0.07),
                (-0.022, 0.074),
                (-0.022, 0.08),
                (0.026, 0.08), // (its top's inner arris chamfered 4 mm)
                (0.03, 0.076),
            ],
            M.body,
            at = 2.26,
            plane = Facing::Top
        );
        fbox!(
            f,
            "plinth",
            1.84,
            0.1,
            0.37,
            (0, 0.05, WALL + 0.185),
            M.body
        );
        kit::moulding!(
            f,
            "plinth_mould",
            // (its corners chamfered 4 mm)
            &[
                (-0.906, WALL + 0.012),
                (0.906, WALL + 0.012),
                (0.91, WALL + 0.016),
                (0.91, WALL + 0.376),
                (0.906, WALL + 0.38),
                (-0.906, WALL + 0.38),
                (-0.91, WALL + 0.376),
                (-0.91, WALL + 0.016),
            ],
            &[
                (0.01, 0.0),
                (-0.012, 0.0),
                (-0.012, 0.09),
                (-0.006, 0.1),
                (0.0, 0.11),
                (0.01, 0.11),
            ],
            M.body,
            at = 0.0,
            plane = Facing::Top
        );
        fbox!(f, "back", 1.8, 2.16, 0.012, (0, 1.18, WALL + 0.006), M.back);
        for x in [-0.91, -0.3, 0.3, 0.91] {
            fbox!(
                f,
                numbered("upright"),
                0.04,
                2.14,
                0.38,
                (x, 1.17, WALL + 0.19),
                M.body,
                bevel = 0.006,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("upright_bead"),
                0.024,
                2.1,
                0.008,
                (x, 1.17, WALL + 0.383),
                M.panel,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        let ys = [0.1, 0.52, 0.92, 1.32, 1.72, 2.12];
        for y in ys {
            fbox!(
                f,
                numbered("shelf"),
                1.8,
                0.03,
                0.37,
                (0, y, WALL + 0.19),
                M.body,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("shelf_edge"),
                1.8,
                0.04,
                0.012,
                (0, y - 0.004, WALL + 0.374),
                M.panel,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        let bays = [(-0.89, -0.32), (-0.28, 0.28), (0.32, 0.89)];
        for (i, &y) in ys[..ys.len() - 1].iter().enumerate() {
            for (j, &(a, b)) in bays.iter().enumerate() {
                if (i, j) == (3, 2) {
                    // the globe's bay
                    books(
                        f,
                        &mut rnd,
                        a,
                        b,
                        y + 0.015,
                        WALL + 0.02,
                        0.28,
                        0.32,
                        0.4,
                        false,
                        false,
                    );
                    continue;
                }
                if (i, j) == (2, 0) {
                    // the bust's bay
                    books(
                        f,
                        &mut rnd,
                        a + 0.16,
                        b,
                        y + 0.015,
                        WALL + 0.02,
                        0.28,
                        0.32,
                        0.9,
                        false,
                        false,
                    );
                    continue;
                }
                let k = (i * 3 + j) % 4;
                books(
                    f,
                    &mut rnd,
                    a,
                    b,
                    y + 0.015,
                    WALL + 0.02,
                    0.28,
                    0.34,
                    [0.95, 0.7, 0.85, 0.55][k],
                    k == 3,
                    k == 1,
                );
            }
        }
        // a globe on its stand and a marble bust
        globe(f, 0.72, 1.335, -0.25);
        fbox!(
            f,
            "bust_plinth",
            0.12,
            0.06,
            0.12,
            (-0.8, 0.98, -0.28),
            M.bust,
            bevel = 0.008,
            bevel_segments = 1
        );
        bust(f, -0.8, 1.01, -0.28, 1.1);
        // the brass rail the ladder hangs on, turned finials at its ends, on brackets from round wall plates
        kit::tube!(
            f,
            "rail",
            &[
                (-0.9, LADDER_TOP.0, LADDER_TOP.1),
                (0.9, LADDER_TOP.0, LADDER_TOP.1),
            ],
            0.0125,
            M.brass,
            verts = 10,
            round = 1
        );
        for s in [-1.0, 1.0] {
            revolve!(
                numbered("rail_finial"),
                &[
                    (0.0, 0.0),
                    (0.016, 0.0),
                    (0.02, 0.008),
                    (0.012, 0.018),
                    (0.008, 0.024),
                    (0.0, 0.026)
                ],
                at = f.at(s * 0.9, LADDER_TOP.0, LADDER_TOP.1),
                material = M.brass,
                segments = 10,
                rot = (0.0, s * 90.0, 0.0)
            );
        }
        for x in [-0.6, 0.0, 0.6] {
            kit::tube!(
                f,
                numbered("bracket"),
                &[
                    (x, LADDER_TOP.0, WALL + 0.38),
                    (x, LADDER_TOP.0, LADDER_TOP.1),
                ],
                0.008,
                M.brass,
                verts = 6,
                round = 1
            );
            fcyl!(
                f,
                numbered("bracket_rose"),
                0.022,
                0.01,
                (x, LADDER_TOP.0, WALL + 0.385),
                M.brass,
                rot = (90, 0, 0),
                verts = 10
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The rolling ladder: two stiles, seven treads, hooks over the rail and wheels in forks at its feet, all wood (a
    // metal on it would cost the ladder a second draw).
    let ladder = new_parts(|| {
        for x in [0.15, 0.55] {
            let (top, foot) = (
                ladder_at(0.015),
                (LADDER_FOOT.0 + 0.05, LADDER_FOOT.1 - 0.008),
            );
            fsweep!(
                f,
                numbered("stile"),
                &[(x, foot.0, foot.1), (x, top.0, top.1)],
                0,
                M.honey,
                profile = rounded_rect(0.05, 0.032, 0.008, 1),
                up = (1, 0, 0)
            );
            let hook = kit::fillet!(
                &[
                    (x, 1.93, 0.0),
                    (x, 2.0, -0.002),
                    (x, 2.004, -0.032),
                    (x, 1.975, -0.036),
                ],
                0.012
            );
            kit::tube!(
                f,
                numbered("hook"),
                &hook,
                0.007,
                M.body,
                verts = 6,
                round = 1
            );
            fbox!(
                f,
                numbered("fork"),
                0.04,
                0.05,
                0.03,
                (x, 0.055, LADDER_FOOT.1),
                M.body,
                bevel = 0.006,
                bevel_segments = 1
            );
            fcyl!(
                f,
                numbered("wheel"),
                0.03,
                0.02,
                (x, 0.03, LADDER_FOOT.1),
                M.body,
                rot = (0, 90, 0),
                verts = 14,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for k in 1..8 {
            let (y, z) = ladder_at(k as f64 / 8.0);
            fbox!(
                f,
                numbered("tread"),
                0.38,
                0.025,
                0.08,
                (0.35, y, z),
                M.honey,
                bevel = 0.006,
                bevel_segments = 1
            );
            if k == 2 || k == 6 {
                kit::tube!(
                    f,
                    numbered("tie"),
                    &[(0.15, y - 0.03, z + 0.005), (0.55, y - 0.03, z + 0.005)],
                    0.005,
                    M.body,
                    verts = 5,
                    round = 1
                );
            }
        }
    });
    let ladder = join(&ladder, "ladder", None);
    let objs = settle!(centre = false);
    slide!(ladder, "ladder", (0.4, 0, 0), seconds = 1.2); // 40 cm along the rail, towards the middle
    objs
}
