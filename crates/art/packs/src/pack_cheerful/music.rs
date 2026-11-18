//! The sunburst jukebox and the spotlight karaoke machine.

use super::common::{M, WALL, aim, around, fillet, moulding, name, obj, paint, slab, tomb, tube};
use artkit::forms::{Scale, rounded_rect};
use artkit::math::{Axis, P3, V3, cos, degrees, radians, sin};
use artkit::motion::{Spin, attach, spin};
use artkit::parts::{Settle, hinged, join, new_parts, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::py::hypot;
use artkit::scene::{Obj, meshes};
use artkit::shapes::SHARP;
use artkit::{cylinder, fbox, fcyl, fextrude, fsphere, fsweep, revolve, slide, swing, torus};

pub fn settle_in_place() -> Vec<Obj> {
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    )
}

// --- the sunburst jukebox (1x1): an arch-topped cabinet, a record window, bubble tubes either side -------------
const ARCH: f64 = 1.08; // the height of the arch's centre

pub fn jukebox() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let zf = WALL + 0.46; // the cabinet's front
    let zc = WALL + 0.23;
    let still = new_parts(|| {
        fextrude!(
            f,
            "plinth",
            &rounded_rect(0.74, 0.48, 0.04, 2),
            0.08,
            (0.0, 0.04, WALL + 0.24),
            M.chrome,
            plane = Facing::Top,
            bevel = 0.012,
            bevel_segments = 2
        );
        fextrude!(
            f,
            "cabinet",
            &rounded_rect(0.7, 0.46, 0.05, 3),
            0.8,
            (0.0, 0.48, zc),
            M.cherry,
            plane = Facing::Top,
            bevel = 0.02,
            bevel_segments = 2
        );
        around(
            f,
            "belt",
            0.712,
            0.47,
            0.055,
            0.865,
            WALL,
            zf + 0.006,
            rounded_rect(0.03, 0.012, 0.005, 1),
            M.chrome,
            3,
        );
        // the arched top round the record window, its bands stepping out, a sunburst on the window's back
        fextrude!(
            f,
            "arch",
            &tomb(0.35, 0.88, ARCH, 18),
            0.46,
            (0.0, 0.0, zc),
            M.cherry,
            holes = vec![tomb(0.2, 0.9, ARCH, 18)],
            bevel = 0.02,
            bevel_segments = 2
        );
        fextrude!(
            f,
            "arch_band",
            &tomb(0.3, 0.885, ARCH, 18),
            0.022,
            (0.0, 0.0, zf + 0.011),
            M.amber,
            holes = vec![tomb(0.25, 0.895, ARCH, 18)],
            bevel = 0.005,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "arch_inner",
            &tomb(0.25, 0.895, ARCH, 18),
            0.012,
            (0.0, 0.0, zf + 0.006),
            M.sunny,
            holes = vec![tomb(0.2, 0.9, ARCH, 18)]
        );
        slab(
            f,
            "window",
            &tomb(0.2, 0.9, ARCH, 18),
            WALL,
            WALL + 0.04,
            M.window,
            0.0,
        );
        for k in 0..9 {
            // rays from behind the record
            let a = radians((10 + 20 * k) as f64);
            fbox!(
                f,
                name("ray"),
                0.012,
                0.2,
                0.004,
                (0.14 * cos(a), 0.92 + 0.14 * sin(a), WALL + 0.042),
                M.sunny,
                rot = (0.0, degrees(a) - 90.0, 0.0)
            );
        }
        for k in 0..3 {
            // the record stack in the window's foot
            fcyl!(
                f,
                name("stack"),
                0.13,
                0.008,
                (0.0, 0.906 + 0.009 * k as f64, -0.17),
                M.vinyl,
                verts = 24
            );
        }
        fcyl!(
            f,
            "stack_label",
            0.04,
            0.03,
            (0, 0.92, -0.17),
            M.label,
            verts = 12
        );
        fcyl!(
            f,
            "spindle",
            0.006,
            0.05,
            (0, 0.94, -0.17),
            M.chrome,
            verts = 6
        );
        // the song cards in a chrome frame, two rows of titles; a row of push buttons under it
        fextrude!(
            f,
            "cards_frame",
            &rounded_rect(0.48, 0.12, 0.012, 2),
            0.016,
            (0.0, 0.71, zf + 0.008),
            M.chrome,
            holes = vec![rounded_rect(0.44, 0.09, 0.006, 1)],
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "cards",
            0.44,
            0.09,
            0.01,
            (0.0, 0.71, zf + 0.004),
            M.cards
        );
        for (r, y) in [0.73, 0.69].into_iter().enumerate() {
            for k in 0..5 {
                fbox!(
                    f,
                    name("title"),
                    0.06,
                    0.008,
                    0.002,
                    (-0.17 + 0.085 * k as f64, y, zf + 0.01),
                    [M.coral, M.window][(k + r) % 2]
                );
            }
        }
        for (x, m) in [
            (-0.15, M.label),
            (-0.09, M.coral),
            (-0.03, M.mint),
            (0.03, M.sky),
            (0.09, M.label),
            (0.15, M.coral),
        ] {
            revolve!(
                name("button"),
                &[
                    (0.0, 0.0),
                    (0.016, 0.0),
                    (0.016, 0.01),
                    (0.012, 0.018),
                    (0.0, 0.02)
                ],
                at = f.at(x, 0.625, zf),
                material = m,
                segments = 10,
                rot = (90, 0, 0)
            );
            torus!(
                name("button_ring"),
                0.018,
                0.003,
                at = f.at(x, 0.625, zf + 0.002),
                material = M.chrome,
                rot = (90, 0, 0),
                major_segments = 10,
                minor_segments = 3
            );
        }
        // the speaker: cloth in a moulded brass frame, chrome bars bursting out from its foot
        moulding(
            f,
            "grille_frame",
            &[(-0.26, 0.24), (0.26, 0.24), (0.26, 0.56), (-0.26, 0.56)],
            &[
                (0.0, 0.0),
                (0.0, 0.01),
                (0.008, 0.016),
                (0.02, 0.016),
                (0.026, 0.008),
                (0.026, 0.0),
            ],
            M.brass,
            zf,
            true,
            SHARP,
        );
        fbox!(
            f,
            "grille",
            0.48,
            0.28,
            0.012,
            (0.0, 0.4, zf + 0.004),
            M.grille
        );
        for k in 0..7 {
            let a = radians((30 + 20 * k) as f64);
            let (c, s) = (cos(a), sin(a));
            let first = if c.abs() > 1e-3 { 0.235 / c.abs() } else { 9.0 };
            let reach = if 0.29 / s < first { 0.29 / s } else { first };
            fsweep!(
                f,
                name("burst"),
                &[
                    (0.03 * c, 0.255 + 0.03 * s, zf + 0.014),
                    (reach * c, 0.255 + reach * s, zf + 0.014)
                ],
                0,
                M.chrome,
                profile = rounded_rect(0.008, 0.01, 0.003, 1),
                up = (0, 0, 1)
            );
        }
        fcyl!(
            f,
            "burst_hub",
            0.04,
            0.012,
            (0.0, 0.255, zf + 0.012),
            M.chrome,
            rot = (90, 0, 0),
            verts = 12
        );
        // the bubble tubes at the front corners: turned chrome feet and caps, collars, bubbles rising in them
        for s in [-1.0, 1.0] {
            let (x, z) = (s * 0.33, zf - 0.01);
            revolve!(
                name("tube"),
                &[(0.0, 0.0), (0.035, 0.0), (0.035, 0.86), (0.0, 0.86)],
                at = f.at(x, 0.12, z),
                material = M.amber,
                segments = 14
            );
            revolve!(
                name("tube_foot"),
                &[
                    (0.0, 0.0),
                    (0.045, 0.0),
                    (0.046, 0.02),
                    (0.04, 0.035),
                    (0.042, 0.05),
                    (0.0, 0.05)
                ],
                at = f.at(x, 0.095, z),
                material = M.chrome,
                segments = 14
            );
            revolve!(
                name("tube_cap"),
                &[
                    (0.0, 0.0),
                    (0.042, 0.0),
                    (0.046, 0.016),
                    (0.04, 0.03),
                    (0.024, 0.046),
                    (0.012, 0.062),
                    (0.016, 0.072),
                    (0.0, 0.08)
                ],
                at = f.at(x, 0.97, z),
                material = M.chrome,
                segments = 14
            );
            for y in [0.4, 0.68] {
                torus!(
                    name("tube_collar"),
                    0.037,
                    0.006,
                    at = f.at(x, y, z),
                    material = M.chrome,
                    major_segments = 14,
                    minor_segments = 3
                );
            }
            for k in 0..5 {
                // bubbles rising in the tubes
                fsphere!(
                    f,
                    name("bubble"),
                    0.012,
                    (x, 0.25 + 0.15 * k as f64 + 0.04 * (k % 2) as f64, z + 0.032),
                    M.sunny,
                    segments = 6,
                    rings = 4
                );
            }
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The record on top of the stack, lying flat, hinged on its front edge: `record` swings it up into the window.
    let zr = -0.17 + 0.13;
    let rec = new_parts(|| {
        fcyl!(
            f,
            "record",
            0.13,
            0.008,
            (0, 0.937, -0.17),
            M.vinyl,
            verts = 32
        );
        for r in [0.07, 0.1] {
            torus!(
                name("record_groove"),
                r,
                0.003,
                at = f.at(0, 0.941, -0.17),
                material = M.vinyl,
                major_segments = 24,
                minor_segments = 3
            );
        }
    });
    let rec = hinged(join(&rec, "record", None), f.at(0, 0.937, zr));
    // Its label, a part inside it: `spin` turns it (and a coral stripe across it, to see it turn) while the record plays.
    let lab = new_parts(|| {
        fcyl!(
            f,
            "record_label",
            0.045,
            0.01,
            (0, 0.937, -0.17),
            M.label,
            verts = 16
        );
        fbox!(
            f,
            "label_stripe",
            0.012,
            0.0125,
            0.07,
            (0, 0.937, -0.17),
            M.coral
        );
    });
    let lab = hinged(join(&lab, "record_label", None), f.at(0, 0.937, -0.17));
    let objs = settle_in_place();
    attach(lab, rec);
    swing!(rec, "record", 90.0, seconds = 0.8, axis = Axis::X);
    spin(
        lab,
        "spin",
        Spin {
            turns: -1.0,
            seconds: 1.8,
            axis: Axis::Z,
        },
    ); // 33 1/3 a minute
    objs
}

// --- the spotlight karaoke machine (1x1): a party speaker with a lyrics screen, a mic stand and a spotlight -----

/// A point `dy` up and `dz` out from `c` (game) on a face leaning back by `ang` degrees.
pub fn tilt(c: (f64, f64, f64), ang: f64, dy: f64, dz: f64) -> (f64, f64, f64) {
    let a = radians(ang);
    (
        c.0,
        c.1 + dy * cos(a) + dz * sin(a),
        c.2 - dy * sin(a) + dz * cos(a),
    )
}

const MIC: (f64, f64) = (0.18, 0.2); // the mic stand's foot (x, depth)

/// A speaker driver facing the front at height `y`, its flange on the face at depth `z`: a flange, a rolled
/// rubber surround, the cone dished in and a domed dust cap, one turned piece (the surround in the trim colour).
fn driver(f: &GameFrame, name: &str, y: f64, z: f64, r: f64, segments: usize) -> Obj {
    let prof = [
        (r + 0.02, -0.01),
        (r + 0.02, 0.003),
        (r + 0.008, 0.005),
        (r + 0.002, 0.012),
        (r - 0.008, 0.016),
        (r - 0.017, 0.011),
        (r - 0.022, 0.003),
        (r * 0.36, -0.04),
        (r * 0.33, -0.036),
        (r * 0.24, -0.028),
        (r * 0.12, -0.023),
        (0.0, -0.022),
    ];
    let d = revolve!(
        name,
        &prof,
        at = f.at(0, y, z),
        material = M.cone,
        segments = segments,
        rot = (90, 0, 0)
    );
    paint(d, M.speaker_trim, |c| {
        hypot(c.x as f64, c.y as f64) > r - 0.024
    })
}

pub fn karaoke_machine() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (w, h, d) = (0.5, 0.86, 0.415);
    let zf = WALL + d;
    let still = new_parts(|| {
        fextrude!(
            f,
            "cabinet",
            &rounded_rect(w, d, 0.05, 3),
            h - 0.03,
            (0.0, 0.03 + (h - 0.03) / 2.0, WALL + d / 2.0),
            M.speaker,
            plane = Facing::Top,
            bevel = 0.03,
            bevel_segments = 2
        );
        around(
            f,
            "band",
            w + 0.006,
            d,
            0.053,
            h - 0.12,
            WALL,
            zf + 0.003,
            rounded_rect(0.04, 0.008, 0.003, 1),
            M.speaker_trim,
            3,
        );
        for fx in [-w / 2.0 + 0.06, w / 2.0 - 0.06] {
            for fz in [WALL + 0.06, zf - 0.06] {
                revolve!(
                    name("foot"),
                    &[
                        (0.0, 0.0),
                        (0.022, 0.0),
                        (0.026, 0.02),
                        (0.022, 0.03),
                        (0.0, 0.03)
                    ],
                    at = f.at(fx, 0, fz),
                    material = M.speaker_trim,
                    segments = 10
                );
            }
        }
        driver(f, "woofer", 0.36, zf, 0.17, 24);
        driver(f, "tweeter", 0.66, zf, 0.075, 16);
        fextrude!(
            f,
            "port",
            &rounded_rect(0.22, 0.04, 0.02, 3),
            0.012,
            (0.0, 0.11, zf + 0.002),
            M.cone,
            holes = vec![rounded_rect(0.2, 0.024, 0.012, 3)]
        );
        fbox!(
            f,
            "port_hole",
            0.2,
            0.024,
            0.004,
            (0.0, 0.11, zf - 0.002),
            M.dark
        );
        fbox!(
            f,
            "light_strip",
            0.3,
            0.012,
            0.006,
            (0.0, 0.15, zf + 0.004),
            M.label,
            bevel = 0.003,
            bevel_segments = 1
        );
        for k in 0..3 {
            // the knobs and the volume ring on the top
            fcyl!(
                f,
                name("top_knob"),
                0.014,
                0.014,
                (-0.15 + 0.05 * k as f64, h + 0.007, WALL + 0.36),
                M.steel,
                verts = 10
            );
        }
        for s in [-1.0, 1.0] {
            // a carrying handle in each side
            tube(
                f,
                &name("handle"),
                &fillet(
                    &[
                        (s * (w / 2.0 - 0.004), h - 0.25, WALL + 0.14),
                        (s * (w / 2.0 + 0.014), h - 0.25, WALL + 0.15),
                        (s * (w / 2.0 + 0.014), h - 0.25, WALL + 0.27),
                        (s * (w / 2.0 - 0.004), h - 0.25, WALL + 0.28),
                    ],
                    0.01,
                    3,
                ),
                0.007,
                M.speaker_trim,
                Some(6),
                1,
                Scale::One(1.0),
            );
        }
        // the lyrics screen in a rounded case on a hinged neck, leaning back
        let sc = (0.0, 1.08, -0.2);
        fsweep!(
            f,
            "neck",
            &[(0, 0.86, -0.22), (0, 0.92, -0.215), (0, 0.96, -0.21)],
            0,
            M.dark,
            profile = rounded_rect(0.06, 0.035, 0.012, 2),
            up = (1, 0, 0)
        );
        fcyl!(
            f,
            "neck_hinge",
            0.016,
            0.08,
            (0, 0.95, -0.21),
            M.dark,
            rot = (0, 90, 0),
            verts = 10
        );
        fextrude!(
            f,
            "screen",
            &rounded_rect(0.46, 0.34, 0.025, 3),
            0.04,
            sc,
            M.dark,
            rot = (-10, 0, 0),
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            "screen_face",
            0.42,
            0.29,
            0.004,
            tilt(sc, 10.0, 0.0, 0.021),
            M.lyrics,
            rot = (-10, 0, 0)
        );
        for (dy, wl, m) in [
            (0.06, 0.3, M.white),
            (0.01, 0.24, M.label),
            (-0.04, 0.28, M.white),
            (-0.09, 0.18, M.white),
        ] {
            fbox!(
                f,
                name("lyric"),
                wl,
                0.022,
                0.003,
                tilt(sc, 10.0, dy, 0.024),
                m,
                rot = (-10, 0, 0)
            );
        }
        fsphere!(
            f,
            "bouncing_ball",
            0.012,
            tilt(sc, 10.0, 0.045, 0.026),
            M.label,
            segments = 6,
            rings = 4
        );
        // the round stage rug with its border and star, the mic stand's weighted foot and lower tube
        fcyl!(
            f,
            "stage",
            0.3,
            0.01,
            (0.02, 0.005, 0.1),
            M.stage,
            verts = 32
        );
        torus!(
            "stage_edge",
            0.296,
            0.006,
            at = f.at(0.02, 0.008, 0.1),
            material = M.star,
            major_segments = 32,
            minor_segments = 3
        )
        .set_scale((1, 1, 0.5));
        let star: Vec<(f64, f64)> = (0..10)
            .map(|k| {
                let r = if k % 2 == 0 { 0.13 } else { 0.055 };
                (
                    0.02 + r * sin(std::f64::consts::PI * k as f64 / 5.0),
                    0.1 + r * cos(std::f64::consts::PI * k as f64 / 5.0),
                )
            })
            .collect();
        let mut verts: Vec<P3> = star
            .iter()
            .map(|&(x, z)| P3 {
                x: -x,
                y: -z,
                z: 0.0105,
            })
            .collect();
        verts.extend(star.iter().map(|&(x, z)| P3 {
            x: -x,
            y: -z,
            z: 0.006,
        }));
        let mut faces = vec![(0..10).collect::<Vec<u32>>(), (10..20).rev().collect()];
        faces.extend((0..10).map(|i| vec![i, (i + 1) % 10, 10 + (i + 1) % 10, 10 + i]));
        obj("star", &verts, &faces, &[M.star], None, 35.0, 0.0);
        let (mx, mz) = MIC;
        revolve!(
            "mic_foot",
            &[
                (0.0, 0.0),
                (0.13, 0.0),
                (0.13, 0.008),
                (0.122, 0.014),
                (0.06, 0.022),
                (0.03, 0.03),
                (0.02, 0.05),
                (0.0, 0.05)
            ],
            at = f.at(mx, 0.01, mz),
            material = M.stand,
            segments = 20
        );
        tube(
            f,
            "mic_tube",
            &[(mx, 0.05, mz), (mx, 1.0, mz)],
            0.013,
            M.chrome,
            Some(10),
            1,
            Scale::One(1.0),
        );
        revolve!(
            "mic_clutch",
            &[
                (0.0, 0.0),
                (0.02, 0.0),
                (0.022, 0.006),
                (0.022, 0.034),
                (0.018, 0.04),
                (0.0, 0.04)
            ],
            at = f.at(mx, 0.98, mz),
            material = M.stand,
            segments = 10
        );
        tube(
            f,
            "clutch_screw",
            &[(mx, 1.0, mz), (mx + 0.04, 1.0, mz)],
            0.005,
            M.stand,
            Some(6),
            1,
            Scale::One(1.0),
        );
        // the spotlight on its pole, aimed at the stage: a finned can on a yoke, a lens in its rim
        let (px, pz) = (-0.32, 0.3);
        revolve!(
            "spot_foot",
            &[
                (0.0, 0.0),
                (0.12, 0.0),
                (0.12, 0.008),
                (0.112, 0.014),
                (0.03, 0.026),
                (0.02, 0.04),
                (0.0, 0.04)
            ],
            at = f.at(px, 0, pz),
            material = M.stand,
            segments = 20
        );
        tube(
            f,
            "spot_pole",
            &[(px, 0.03, pz), (px, 1.6, pz)],
            0.012,
            M.stand,
            Some(8),
            1,
            Scale::One(1.0),
        );
        let (head, aim_at) = ((px, 1.66, pz), (0.1, 1.2, 0.9));
        let u = V3::from((aim_at.0 - head.0, aim_at.1 - head.1, aim_at.2 - head.2)).normalized();
        let d_b =
            V3::from(f.at(aim_at.0, aim_at.1, aim_at.2)) - V3::from(f.at(head.0, head.1, head.2));
        let (ux, uy, uz) = (u.x as f64, u.y as f64, u.z as f64);
        let can = revolve!(
            "spot_can",
            &[
                (0.0, -0.1),
                (0.045, -0.1),
                (0.058, -0.085),
                (0.064, -0.06),
                (0.064, 0.07),
                (0.074, 0.082),
                (0.074, 0.094),
                (0.066, 0.094),
                (0.062, 0.09),
                (0.0, 0.09)
            ],
            at = f.at(head.0, head.1, head.2),
            material = M.dark,
            segments = 16
        );
        aim(can, d_b);
        for k in 0..3 {
            let o = 0.04 - 0.025 * k as f64;
            let fin = torus!(
                name("spot_fin"),
                0.068,
                0.004,
                at = f.at(head.0 - ux * o, head.1 - uy * o, head.2 - uz * o),
                material = M.dark,
                major_segments = 16,
                minor_segments = 3
            );
            aim(fin, d_b);
        }
        let lens = cylinder!(
            "spot_lens",
            0.06,
            0.004,
            at = f.at(
                head.0 + ux * 0.088,
                head.1 + uy * 0.088,
                head.2 + uz * 0.088
            ),
            material = M.lens,
            verts = 16
        );
        aim(lens, d_b);
        let side = V3::from((uz, 0.0, -ux)).normalized(); // across the can, level (game)
        let (sx, sy, sz) = (side.x as f64, side.y as f64, side.z as f64);
        let ends: Vec<(f64, f64, f64)> = [-1.0, 1.0]
            .iter()
            .map(|&s| {
                (
                    head.0 + sx * s * 0.085,
                    head.1 + sy * s * 0.085,
                    head.2 + sz * s * 0.085,
                )
            })
            .collect();
        tube(
            f,
            "spot_yoke",
            &fillet(
                &[
                    ends[0],
                    (ends[0].0, 1.6, ends[0].2),
                    (px, 1.6, pz),
                    (ends[1].0, 1.6, ends[1].2),
                    ends[1],
                ],
                0.02,
                3,
            ),
            0.008,
            M.stand,
            Some(6),
            1,
            Scale::One(1.0),
        );
        for &e in &ends {
            fsphere!(
                f,
                name("spot_knob"),
                0.014,
                e,
                M.stand,
                segments = 8,
                rings = 4
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The upper tube, boom and microphone, lowered 25 cm into the tube: `mic_up` raises them to singing height.
    let (mx, mz) = MIC;
    let drop = 0.25;
    let mic = new_parts(|| {
        let top = 1.35 - drop;
        tube(
            f,
            "mic_upper",
            &[(mx, 0.75 - drop, mz), (mx, top, mz)],
            0.008,
            M.chrome,
            Some(8),
            1,
            Scale::One(1.0),
        );
        let tip = (mx - 0.05, top + 0.07, mz + 0.08);
        tube(
            f,
            "mic_boom",
            &fillet(&[(mx, top - 0.02, mz), (mx, top + 0.02, mz), tip], 0.02, 3),
            0.007,
            M.chrome,
            Some(6),
            1,
            Scale::One(1.0),
        );
        let end = (tip.0 - 0.015, tip.1 + 0.05, tip.2 + 0.07);
        fsweep!(
            f,
            "mic_handle",
            &[tip, end],
            0.017,
            M.dark,
            verts = 10,
            scale = [0.7, 1.0]
        );
        let g = (tip.0 - 0.02, tip.1 + 0.068, tip.2 + 0.095);
        fsphere!(f, "mic_grille", 0.03, g, M.steel, segments = 10, rings = 6);
        let band = (
            end.0 + (g.0 - end.0) * 0.35,
            end.1 + (g.1 - end.1) * 0.35,
            end.2 + (g.2 - end.2) * 0.35,
        );
        fsweep!(f, "mic_band", &[end, band], 0.024, M.dark, verts = 10);
    });
    let mic = hinged(join(&mic, "mic", None), f.at(mx, 1.0, mz));
    let objs = settle_in_place();
    slide!(mic, "mic_up", (0, 0, drop), seconds = 0.8);
    objs
}
