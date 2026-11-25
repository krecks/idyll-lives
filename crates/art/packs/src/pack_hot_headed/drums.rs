//! The thunder drum kit (2x1): the drummer on the throne at the middle, the kit before them.

use super::common::M;
use artkit::forms::rounded_rect;
use artkit::math::{Axis, P3, TAU, cos, cosd, radians, sin, sind, v3};
use artkit::motion::{Rock, rock};
use artkit::parts::{join, new_parts, numbered, settle_all};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fsphere, fsweep, kit, revolve, torus};

/// A drum shell with chrome hoops, white heads and lugs: `along_z` false a snare or tom (tilted `tilt` about x),
/// true the bass drum lying towards the front.
#[allow(clippy::too_many_arguments)]
fn drum(
    f: &GameFrame,
    x: f64,
    y: f64,
    z: f64,
    dia: f64,
    depth: f64,
    along_z: bool,
    tilt: f64,
    lugs: i32,
) {
    let r = dia / 2.0;
    let rot = if along_z {
        (90.0, 0.0, 0.0)
    } else {
        (tilt, 0.0, 0.0)
    };
    // the shell an open tube: its ends are under the heads
    revolve!(
        numbered("shell"),
        &[(r, -depth / 2.0), (r, depth / 2.0)],
        at = f.at(x, y, z),
        material = M.shell,
        segments = 24,
        rot = rot
    );

    // The game point `t` along the drum's axis from its middle, `u` across in x, `v` out along its other side.
    let along = |t: f64, u: f64, v: f64| -> (f64, f64, f64) {
        if along_z {
            return (x + u, y + v, z + t);
        }
        let (c, s) = (cosd(tilt), sind(tilt)); // the shell turned about x: its axis (height, depth) is (c, s)
        (x + u, y + t * c - v * s, z + t * s + v * c)
    };
    // a head on each end, 3 mm thick over the shell's whole end: its edge a band too narrow to show hard, and the
    // shell's rim under it
    for e in [-1.0, 1.0] {
        fcyl!(
            f,
            numbered("head"),
            r,
            0.003,
            along(e * (depth / 2.0 + 0.0015), 0.0, 0.0),
            M.head,
            rot = rot,
            verts = 24,
            smooth = false
        );
    }
    for e in [-1.0, 1.0] {
        let p = along(e * depth / 2.0, 0.0, 0.0);
        torus!(
            numbered("hoop"),
            r + 0.008,
            0.009,
            at = f.at(p.0, p.1, p.2),
            material = M.chrome,
            rot = rot,
            major_segments = 18,
            minor_segments = 3
        );
    }
    for k in 0..lugs {
        let a = TAU * (k as f64 + 0.5) / lugs as f64;
        let (u, v) = ((r + 0.008) * cos(a), (r + 0.008) * sin(a));
        kit::tube!(
            f,
            numbered("lug"),
            &[along(-depth * 0.2, u, v), along(depth * 0.2, u, v)],
            0.006,
            M.chrome,
            verts = 4,
            round = 1
        );
    }
}

/// A chrome tripod stand: three legs braced to a collar, a tube up to height `h` with a clamp halfway.
fn stand(f: &GameFrame, x: f64, z: f64, h: f64, spread: f64) {
    kit::tube!(
        f,
        numbered("stand_pole"),
        &[(x, 0.1, z), (x, h * 0.55, z)],
        0.011,
        M.chrome,
        verts = 8,
        round = 1
    );
    kit::tube!(
        f,
        numbered("stand_upper"),
        &[(x, h * 0.55, z), (x, h - 0.02, z)],
        0.008,
        M.chrome,
        verts = 8,
        round = 1
    );
    fcyl!(
        f,
        numbered("stand_clamp"),
        0.015,
        0.024,
        (x, h * 0.55, z),
        M.chrome,
        verts = 6
    );
    fcyl!(
        f,
        numbered("stand_hub"),
        0.017,
        0.03,
        (x, 0.22, z),
        M.chrome,
        verts = 6
    );
    for k in 0..3 {
        let a = (90 + 120 * k) as f64;
        let (c, s) = (cosd(a), sind(a));
        kit::tube!(
            f,
            numbered("stand_leg"),
            &[(x, 0.22, z), (x + spread * c, 0.012, z + spread * s)],
            0.0065,
            M.chrome,
            verts = 6,
            round = 1
        );
        kit::tube!(
            f,
            numbered("stand_brace"),
            &[
                (x, 0.13, z),
                (x + spread * 0.5 * c, 0.115, z + spread * 0.5 * s),
            ],
            0.004,
            M.chrome,
            verts = 4,
            round = 1
        );
        fcyl!(
            f,
            numbered("stand_foot"),
            0.011,
            0.014,
            (x + spread * c, 0.007, z + spread * s),
            M.dark,
            verts = 5
        );
    }
}

/// A cymbal centred on (x, y, z): a domed bell rising out of a bow that thins to its edge, a felt and its nut.
fn cymbal(f: &GameFrame, name_: &str, x: f64, y: f64, z: f64, d: f64) {
    let r = d / 2.0;
    revolve!(
        name_,
        &[
            (0.0, 0.03),
            (0.035, 0.027),
            (0.048, 0.014),
            (r * 0.6, 0.005),
            (r, 0.0),
            (r, -0.003),
            (r * 0.6, 0.002),
            (0.048, 0.01),
            (0.035, 0.022),
            (0.0, 0.025)
        ],
        at = f.at(x, y, z),
        material = M.cymbal,
        segments = 18
    );
    fcyl!(
        f,
        numbered("felt"),
        0.014,
        0.022,
        (x, y + 0.042, z),
        M.dark,
        verts = 8
    );
    fcyl!(
        f,
        numbered("wingnut"),
        0.006,
        0.014,
        (x, y + 0.058, z),
        M.chrome,
        verts = 6
    );
    fbox!(
        f,
        numbered("wing"),
        0.03,
        0.01,
        0.003,
        (x, y + 0.062, z),
        M.chrome
    );
}

/// A foot pedal along the depth, its heel at the back: a heel plate, a footboard tilting up, its frame and cam.
fn pedal(f: &GameFrame, x: f64, z: f64, length: f64) {
    let (z0, z1) = (z - length / 2.0, z + length / 2.0);
    fbox!(
        f,
        numbered("pedal_heel"),
        0.075,
        0.012,
        0.06,
        (x, 0.018, z0 + 0.03),
        M.chrome,
        bevel = 0.003,
        bevel_segments = 1
    );
    fsweep!(
        f,
        numbered("pedal_board"),
        &[(x, 0.026, z0 + 0.05), (x, 0.06, z1 - 0.03)],
        0,
        M.dark,
        profile = rounded_rect(0.008, 0.065, 0.003, 1),
        scale = [0.85, 1.1]
    );
    fbox!(
        f,
        numbered("pedal_base"),
        0.11,
        0.01,
        0.06,
        (x, 0.017, z1),
        M.chrome,
        bevel = 0.003,
        bevel_segments = 1
    );
    for s in [-1.0, 1.0] {
        kit::tube!(
            f,
            numbered("pedal_post"),
            &[(x + s * 0.05, 0.02, z1), (x + s * 0.05, 0.13, z1)],
            0.006,
            M.chrome,
            verts = 6,
            round = 1
        );
    }
    fcyl!(
        f,
        numbered("pedal_cam"),
        0.02,
        0.1,
        (x, 0.12, z1),
        M.chrome,
        rot = (0, 90, 0),
        verts = 10
    );
}

const CRASH: (f64, f64, f64, f64) = (-0.5, 1.22, 0.22, 0.4); // x, height, depth, diameter
const RIDE: (f64, f64, f64, f64) = (0.66, 1.12, 0.18, 0.44);

pub fn thunder_drums() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            f,
            "rug",
            1.88,
            0.012,
            0.9,
            (0, 0.006, 0),
            M.rug,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(f, "rug_band", 1.8, 0.0135, 0.82, (0, 0.00675, 0), M.ember);
        fbox!(f, "rug_field", 1.74, 0.015, 0.76, (0, 0.0075, 0), M.rug);
        // the bass drum, its port and logo, spurs; the pedal and beater on its batter head
        drum(f, 0.0, 0.29, 0.27, 0.52, 0.34, true, 0.0, 8);
        fcyl!(
            f,
            "bass_logo",
            0.1,
            0.004,
            (0.0, 0.29, 0.443),
            M.flame,
            rot = (90, 0, 0),
            verts = 20,
            smooth = false
        );
        fcyl!(
            f,
            "bass_port",
            0.045,
            0.004,
            (0.12, 0.2, 0.443),
            M.speaker,
            rot = (90, 0, 0),
            verts = 14,
            smooth = false
        );
        for s in [-1.0, 1.0] {
            kit::tube!(
                f,
                numbered("spur"),
                &kit::fillet!(
                    &[
                        (s * 0.22, 0.16, 0.36),
                        (s * 0.27, 0.1, 0.38),
                        (s * 0.3, 0.012, 0.4),
                    ],
                    0.03
                ),
                0.008,
                M.chrome,
                verts = 6,
                round = 1
            );
            fcyl!(
                f,
                numbered("spur_foot"),
                0.011,
                0.014,
                (s * 0.3, 0.007, 0.4),
                M.dark,
                verts = 6
            );
        }
        pedal(f, 0.0, 0.02, 0.2);
        kit::tube!(
            f,
            "beater",
            &[(0.0, 0.12, 0.12), (0.0, 0.27, 0.11)],
            0.004,
            M.chrome,
            verts = 5,
            round = 1
        );
        fcyl!(
            f,
            "beater_head",
            0.022,
            0.03,
            (0.0, 0.28, 0.108),
            M.head,
            rot = (90, 0, 0),
            verts = 10
        );
        // the rack toms on their mount, the floor tom on its legs, the snare on its stand
        drum(f, -0.15, 0.68, 0.24, 0.26, 0.18, false, -18.0, 6);
        drum(f, 0.15, 0.68, 0.24, 0.28, 0.2, false, -18.0, 6);
        fbox!(
            f,
            "tom_clamp",
            0.05,
            0.03,
            0.05,
            (0.0, 0.555, 0.27),
            M.chrome,
            bevel = 0.006,
            bevel_segments = 1
        );
        kit::tube!(
            f,
            "tom_mount",
            &[(0.0, 0.55, 0.27), (0.0, 0.64, 0.26)],
            0.013,
            M.chrome,
            verts = 8,
            round = 1
        );
        for s in [-1.0, 1.0] {
            kit::tube!(
                f,
                numbered("tom_arm"),
                &kit::fillet!(
                    &[
                        (0.0, 0.64, 0.26),
                        (s * 0.09, 0.64, 0.26),
                        (s * 0.09, 0.66, 0.255),
                    ],
                    0.02
                ),
                0.008,
                M.chrome,
                verts = 6,
                round = 1
            );
        }
        drum(f, 0.42, 0.43, 0.12, 0.38, 0.34, false, 0.0, 8);
        for a in [30.0, 150.0, 270.0] {
            kit::tube!(
                f,
                numbered("tom_leg"),
                &kit::fillet!(
                    &[
                        (0.42 + 0.205 * cosd(a), 0.46, 0.12 + 0.205 * sind(a)),
                        (0.42 + 0.23 * cosd(a), 0.4, 0.12 + 0.23 * sind(a)),
                        (0.42 + 0.24 * cosd(a), 0.012, 0.12 + 0.24 * sind(a)),
                    ],
                    0.02
                ),
                0.007,
                M.chrome,
                verts = 6,
                round = 1
            );
            fcyl!(
                f,
                numbered("tom_foot"),
                0.011,
                0.014,
                (0.42 + 0.24 * cosd(a), 0.007, 0.12 + 0.24 * sind(a)),
                M.dark,
                verts = 6
            );
        }
        drum(f, -0.32, 0.6, 0.14, 0.34, 0.14, false, -6.0, 8);
        stand(f, -0.32, 0.14, 0.53, 0.16);
        fbox!(
            f,
            "snare_strainer",
            0.02,
            0.045,
            0.025,
            (-0.32 + 0.18, 0.6, 0.14),
            M.chrome,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the hi-hat, a pair of cymbals on its stand with a pedal
        stand(f, -0.66, 0.06, 0.9, 0.17);
        cymbal(f, "hihat_low", -0.66, 0.82, 0.06, 0.32);
        cymbal(f, "hihat_top", -0.66, 0.855, 0.06, 0.32);
        pedal(f, -0.66, -0.06, 0.18);
        // the crash's and the ride's stands
        stand(f, CRASH.0, CRASH.2, CRASH.1, 0.17);
        stand(f, RIDE.0, RIDE.2, RIDE.1, 0.17);
        // the throne, top 0.47: a round padded seat on a tripod; a pair of sticks on it
        kit::lathe!(
            f,
            "throne_seat",
            0.0,
            0.4,
            0.0,
            &[
                (0.15, 0.0),
                (0.175, 0.012),
                (0.182, 0.035),
                (0.178, 0.055),
                (0.16, 0.066),
                (0.1, 0.07),
                (0.0, 0.07),
            ],
            M.dark,
            verts = 24
        );
        fcyl!(
            f,
            "throne_base",
            0.06,
            0.02,
            (0, 0.39, 0.0),
            M.chrome,
            verts = 12
        );
        stand(f, 0.0, 0.0, 0.4, 0.2);
        for (dz, r) in [(0.03, 70.0), (0.0, 80.0)] {
            // across the floor tom's head
            let a = radians(r);
            fsweep!(
                f,
                numbered("stick"),
                &[
                    (0.42 - 0.2 * sin(a), 0.611, 0.12 + dz - 0.2 * cos(a)),
                    (0.42 + 0.2 * sin(a), 0.611, 0.12 + dz + 0.2 * cos(a))
                ],
                0.007,
                M.oak,
                verts = 6,
                scale = [1.0, 0.55]
            );
        }
        // the monitor amp at the back: a cabinet with corner caps, its grille cloth in a frame, the speaker, knobs and a
        // handle on top
        let (ax, ay, az) = (0.78, 0.3, -0.28);
        fbox!(
            f,
            "amp",
            0.34,
            0.6,
            0.3,
            (ax, ay, az),
            M.amp,
            bevel = 0.02,
            bevel_segments = 2
        );
        fbox!(
            f,
            "amp_grille",
            0.28,
            0.4,
            0.008,
            (ax, 0.27, az + 0.149),
            M.speaker,
            bevel = 0.006,
            bevel_segments = 1
        );
        kit::turned!(
            f,
            "amp_cone",
            ax,
            0.3,
            az + 0.153,
            &[
                (0.11, 0.0),
                (0.1, 0.004),
                (0.05, 0.012),
                (0.03, 0.012),
                (0.0, 0.02),
            ],
            M.dark,
            segments = 20,
            rot = (-90.0, 0.0, 0.0)
        );
        fsphere!(
            f,
            "amp_dust_cap",
            0.035,
            (ax, 0.3, az + 0.163),
            M.fire,
            scale = (1, 0.5, 1),
            segments = 12,
            rings = 6
        );
        fbox!(
            f,
            "amp_panel",
            0.3,
            0.07,
            0.006,
            (ax, 0.53, az + 0.15),
            M.speaker,
            bevel = 0.003,
            bevel_segments = 1
        );
        for k in 0..4 {
            fcyl!(
                f,
                numbered("amp_knob"),
                0.011,
                0.016,
                (ax - 0.09 + 0.06 * k as f64, 0.53, az + 0.158),
                M.chrome,
                rot = (90, 0, 0),
                verts = 8
            );
        }
        fsphere!(
            f,
            "amp_light",
            0.006,
            (ax + 0.13, 0.53, az + 0.155),
            M.fire,
            segments = 6,
            rings = 4
        );
        kit::tube!(
            f,
            "amp_handle",
            &kit::fillet!(
                &[
                    (ax - 0.08, 0.6, az),
                    (ax - 0.08, 0.63, az),
                    (ax + 0.08, 0.63, az),
                    (ax + 0.08, 0.6, az),
                ],
                0.015
            ),
            0.008,
            M.dark,
            verts = 6,
            round = 1
        );
        for sx in [-1.0, 1.0] {
            for sz in [-1.0, 1.0] {
                fbox!(
                    f,
                    numbered("amp_corner"),
                    0.04,
                    0.04,
                    0.04,
                    (ax + sx * 0.155, 0.02, az + sz * 0.135),
                    M.dark
                );
            }
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The crash and the ride, each tilted on its stand's tip: `cymbals` rocks both while the kit is played.
    let mut moving = Vec::new();
    for (cname, (x, y, z, dia), tilt) in [("crash", CRASH, 14.0), ("ride", RIDE, 10.0)] {
        let c = new_parts(|| cymbal(f, cname, x, 0.0, 0.0, dia));
        let part = join(&c, cname, Some(f.at(x, 0.0, 0.0)));
        part.set_location(f.at(x, y, z));
        part.edit(|o| o.rot = v3(radians(tilt), 0, 0)); // tipped towards the room
        moving.push(part);
    }
    let objs = settle_all(); // on its Model centre: the back on the wall face
    rock(
        moving[0],
        "cymbals",
        9.0,
        Rock {
            seconds: 0.8,
            axis: Axis::X,
        },
    );
    rock(
        moving[1],
        "cymbals",
        -6.0,
        Rock {
            seconds: 0.8,
            axis: Axis::X,
        },
    );
    objs
}
