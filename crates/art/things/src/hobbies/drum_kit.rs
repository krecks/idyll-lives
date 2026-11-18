//! H1: the drum kit, spread over two tiles as a real five-piece is (review M-hobbies-05: it was packed onto one, the toms
//! on the kick and the floor tom under the ride): a 22" kick to the front with two rack toms on a mount on it, a snare
//! and a hi-hat to the drummer's left (game -x: they face the front, +z), a 16" floor tom to the right, the crash and
//! ride cymbals on boom stands over them, and the throne behind (content `seat`: 0.52 m, `DRUM_THRONE` behind the middle).
//! Built in place about the throne. The two cymbals are parts of their own, their origins on their stands' tops, and
//! wobble while a resident drums (`cymbals`).

use super::M;
use super::common::P;
use super::drums::{cymbal, drum, drum_stand, pedal};
use super::kit::call::tube;
use super::kit::fillet;
use artkit::math::{Axis, P3, cosd, sind};
use artkit::motion::{Rock, rock};
use artkit::parts::{Settle, hinged, join, new_parts, settle_with};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsweep, loft, ring, torus};

/// The throne's depth (content `seat.forward`).
const DRUM_THRONE: f64 = -0.4;
/// The cymbals' middles, on their booms.
const CRASH: P = (-0.42, 1.18, DRUM_THRONE + 0.62);
const RIDE: P = (0.52, 1.1, DRUM_THRONE + 0.52);

pub fn drum_kit() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let t = DRUM_THRONE;
    // the kick: its middle and radius
    let (kz, ky, kr) = (t + 0.66, 0.29, 0.28);
    let still = new_parts(|| {
        // the kick drum lies on its side, its front head to the front with a port and the maker's badge
        fcyl!(
            f,
            "kick",
            kr,
            0.4,
            (0, ky, kz),
            M.shell,
            rot = (90, 0, 0),
            verts = 32
        );
        for (k, z) in [kz - 0.2, kz + 0.2].into_iter().enumerate() {
            fcyl!(
                f,
                format!("kick_head{k}"),
                kr - 0.01,
                0.006,
                (0, ky, z),
                M.head,
                rot = (90, 0, 0),
                verts = 28,
                smooth = false
            );
            torus!(
                format!("kick_hoop{k}"),
                kr - 0.005,
                0.013,
                at = f.at(0, ky, z),
                material = M.shell,
                rot = (90, 0, 0),
                major_segments = 24,
                minor_segments = 3
            );
        }
        fcyl!(
            f,
            "kick_logo",
            0.1,
            0.002,
            (0, ky, kz + 0.2045),
            M.shell,
            rot = (90, 0, 0),
            verts = 24,
            smooth = false
        );
        fbox!(
            f,
            "kick_badge",
            0.1,
            0.022,
            0.002,
            (0, ky + 0.01, kz + 0.2058),
            M.head
        );
        fcyl!(
            f,
            "kick_port",
            0.05,
            0.002,
            (0.13, ky - 0.1, kz + 0.2045),
            M.seat,
            rot = (90, 0, 0),
            verts = 16,
            smooth = false
        );
        for k in 0..8 {
            let a = 22.5 + (45 * k) as f64;
            let (c, s) = (cosd(a), sind(a));
            fbox!(
                f,
                format!("kick_lug{k}"),
                0.016,
                0.016,
                0.1,
                ((kr + 0.003) * c, ky + (kr + 0.003) * s, kz),
                M.chrome,
                rot = (0.0, -a, 0.0)
            );
            for e in [-1, 1] {
                let ef = e as f64;
                tube!(
                    f,
                    format!("kick_rod{k}{e}"),
                    &[
                        ((kr + 0.008) * c, ky + (kr + 0.008) * s, kz + ef * 0.06),
                        ((kr + 0.008) * c, ky + (kr + 0.008) * s, kz + ef * 0.2)
                    ],
                    0.003,
                    M.chrome,
                    verts = 4,
                    round = 1
                );
            }
        }
        for s in [-1, 1] {
            let sf = s as f64;
            tube!(
                f,
                format!("spur{s}"),
                &fillet(
                    &[
                        (sf * 0.25, 0.2, kz + 0.1),
                        (sf * 0.3, 0.12, kz + 0.13),
                        (sf * 0.34, 0.012, kz + 0.16)
                    ],
                    0.04,
                    3
                ),
                0.008,
                M.chrome,
                verts = 6,
                round = 1
            );
            fcyl!(
                f,
                format!("spur_foot{s}"),
                0.011,
                0.016,
                (sf * 0.34, 0.008, kz + 0.16),
                M.seat,
                verts = 8
            );
        }
        // the pedal and its beater, on the batter head
        pedal(&f, "pedal", 0.0, kz - 0.33, 0.22);
        tube!(
            f,
            "beater",
            &[(0, 0.12, kz - 0.22), (0, 0.27, kz - 0.215)],
            0.004,
            M.chrome,
            verts = 6,
            round = 1
        );
        fcyl!(
            f,
            "beater_head",
            0.024,
            0.03,
            (0, 0.28, kz - 0.217),
            M.head,
            rot = (90, 0, 0),
            verts = 12,
            bevel = 0.008,
            bevel_segments = 1
        );
        // two rack toms (10" and 12") on a mount on the kick, tilted to the drummer
        fbox!(
            f,
            "tom_clamp",
            0.05,
            0.03,
            0.05,
            (0, ky + kr + 0.005, kz - 0.02),
            M.chrome,
            bevel = 0.006,
            bevel_segments = 1
        );
        tube!(
            f,
            "tom_mount",
            &[(0.0, ky + kr - 0.01, kz - 0.02), (0.0, 0.72, kz - 0.08)],
            0.013,
            M.chrome,
            verts = 8,
            round = 1
        );
        for (k, (x, r, h)) in [(-0.15, 0.13, 0.2), (0.15, 0.155, 0.22)]
            .into_iter()
            .enumerate()
        {
            drum(
                &f,
                &format!("tom{k}"),
                r,
                h,
                (x, 0.78, kz - 0.12),
                (-20.0, 0.0, 0.0),
                &[1],
                6,
                false,
            );
            tube!(
                f,
                format!("tom_arm{k}"),
                &fillet(
                    &[
                        (0.0, 0.72, kz - 0.08),
                        (x * 0.6, 0.72, kz - 0.12),
                        (x * 0.6, 0.74, kz - 0.12)
                    ],
                    0.02,
                    3
                ),
                0.009,
                M.chrome,
                verts = 6,
                round = 1
            );
        }
        // the snare on its stand, between the drummer's knees and the hi-hat, a pair of sticks on it
        drum(
            &f,
            "snare",
            0.18,
            0.13,
            (-0.26, 0.63, t + 0.3),
            (0.0, 0.0, 0.0),
            &[1, -1],
            8,
            false,
        );
        fbox!(
            f,
            "snare_strainer",
            0.02,
            0.05,
            0.025,
            (-0.26 + 0.19, 0.63, t + 0.3),
            M.chrome,
            bevel = 0.004,
            bevel_segments = 1
        );
        drum_stand(&f, "snare_stand", -0.26, t + 0.3, 0.56, 0.2);
        for s in [-1, 1] {
            fbox!(
                f,
                format!("snare_basket{s}"),
                0.2,
                0.012,
                0.012,
                (-0.26, 0.558, t + 0.3 + s as f64 * 0.06),
                M.chrome,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for (k, (dz, yaw)) in [(-0.03, 8.0), (0.03, -5.0)].into_iter().enumerate() {
            let (x0, x1) = (-0.26 - 0.2, -0.26 + 0.18);
            let pts = [
                (x0, 0.7, t + 0.3 + dz - 0.02 * sind(yaw)),
                (x1, 0.7, t + 0.3 + dz + 0.02 * sind(yaw)),
            ];
            fsweep!(
                f,
                format!("stick{k}"),
                &pts,
                0.007,
                M.cue,
                verts = 6,
                scale = [1.0, 0.55]
            );
        }
        // the floor tom on three legs, to the right
        let (fx, fz) = (0.46, t + 0.3);
        drum(
            &f,
            "floor_tom",
            0.2,
            0.36,
            (fx, 0.42, fz),
            (0.0, 0.0, 0.0),
            &[1],
            8,
            false,
        );
        for k in 0..3 {
            let a = (30 + 120 * k) as f64;
            tube!(
                f,
                format!("floor_leg{k}"),
                &fillet(
                    &[
                        (fx + 0.215 * cosd(a), 0.48, fz + 0.215 * sind(a)),
                        (fx + 0.245 * cosd(a), 0.42, fz + 0.245 * sind(a)),
                        (fx + 0.25 * cosd(a), 0.012, fz + 0.25 * sind(a))
                    ],
                    0.02,
                    3
                ),
                0.007,
                M.chrome,
                verts = 6,
                round = 1
            );
            fcyl!(
                f,
                format!("floor_foot{k}"),
                0.011,
                0.016,
                (fx + 0.25 * cosd(a), 0.008, fz + 0.25 * sind(a)),
                M.seat,
                verts = 8
            );
        }
        // the hi-hat: two 14" cymbals shut on their pole, a clutch, and its pedal, by the left foot
        let (hx, hz) = (-0.52, t + 0.22);
        drum_stand(&f, "hihat", hx, hz, 0.94, 0.2);
        cymbal(
            &f,
            "hihat_bottom",
            0.18,
            (hx, 0.885, hz),
            (0.0, 0.0, 0.0),
            18,
        );
        cymbal(&f, "hihat_top", 0.18, (hx, 0.906, hz), (0.0, 0.0, 0.0), 20);
        fcyl!(
            f,
            "hihat_clutch",
            0.012,
            0.03,
            (hx, 0.94, hz),
            M.chrome,
            verts = 8
        );
        pedal(&f, "hihat_pedal", hx, hz - 0.12, 0.2);
        // the crash's and the ride's boom stands
        for (name, x, z, top, bx, bz) in [
            ("crash", -0.62, t + 0.74, 1.1, CRASH.0, CRASH.2),
            ("ride", 0.72, t + 0.62, 1.02, RIDE.0, RIDE.2),
        ] {
            drum_stand(&f, &format!("{name}_stand"), x, z, top, 0.22);
            fcyl!(
                f,
                format!("{name}_tilter"),
                0.016,
                0.03,
                (x, top, z),
                M.chrome,
                verts = 10,
                bevel = 0.003,
                bevel_segments = 1
            );
            let back = (x - (bx - x) * 0.25, top - 0.02, z - (bz - z) * 0.25);
            tube!(
                f,
                format!("{name}_boom"),
                &[back, (bx, top + 0.08, bz)],
                0.008,
                M.chrome,
                verts = 6,
                round = 1
            );
            tube!(
                f,
                format!("{name}_weight"),
                &[
                    back,
                    (
                        back.0 + (x - bx) * 0.05,
                        back.1 - 0.004,
                        back.2 + (z - bz) * 0.05
                    )
                ],
                0.018,
                M.chrome,
                verts = 10,
                round = 1
            );
        }
        // the throne: a round padded seat on a three-legged stand
        let p = f.at(0, 0, t);
        let (tx, tz) = (p.x, p.y);
        let rings = [
            (0.17, 0.452),
            (0.198, 0.462),
            (0.204, 0.48),
            (0.2, 0.5),
            (0.185, 0.513),
            (0.13, 0.52),
            (0.0, 0.521),
        ];
        let sections: Vec<Vec<P3>> = rings
            .iter()
            .map(|&(r, y)| ring!(r, r, z = y, verts = 16, at = (tx, tz)))
            .collect();
        loft!("throne", &sections, material = M.seat);
        fcyl!(
            f,
            "throne_base",
            0.07,
            0.02,
            (0.0, 0.444, t),
            M.chrome,
            verts = 14
        );
        drum_stand(&f, "throne_stand", 0.0, t, 0.44, 0.22);
    });
    join(&still, "body", Some(P3::ZERO));
    let c = new_parts(|| {
        cymbal(&f, "crash", 0.2, CRASH, (-14.0, 10.0, 0.0), 22);
        fcyl!(
            f,
            "crash_felt",
            0.012,
            0.02,
            (CRASH.0, CRASH.1 + 0.02, CRASH.2),
            M.seat,
            verts = 8
        );
        fcyl!(
            f,
            "crash_nut",
            0.008,
            0.012,
            (CRASH.0, CRASH.1 + 0.035, CRASH.2),
            M.chrome,
            verts = 6
        );
    });
    let crash = hinged(join(&c, "crash", None), f.at(CRASH.0, CRASH.1, CRASH.2));
    let r = new_parts(|| {
        cymbal(&f, "ride", 0.25, RIDE, (-10.0, -8.0, 0.0), 32);
        fcyl!(
            f,
            "ride_felt",
            0.012,
            0.02,
            (RIDE.0, RIDE.1 + 0.02, RIDE.2),
            M.seat,
            verts = 8
        );
        fcyl!(
            f,
            "ride_nut",
            0.008,
            0.012,
            (RIDE.0, RIDE.1 + 0.035, RIDE.2),
            M.chrome,
            verts = 6
        );
    });
    let ride = hinged(join(&r, "ride", None), f.at(RIDE.0, RIDE.1, RIDE.2));
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    rock(
        crash,
        "cymbals",
        7.0,
        Rock {
            seconds: 1.0,
            axis: Axis::X,
        },
    );
    rock(
        ride,
        "cymbals",
        4.0,
        Rock {
            seconds: 1.0,
            axis: Axis::Y,
        },
    );
    meshes()
}
