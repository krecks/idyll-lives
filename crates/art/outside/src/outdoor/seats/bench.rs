use super::super::common::{Bar, F, M, bar, bolts, end_round, finish, plank};
use artkit::fbox;
use artkit::json::float_repr;
use artkit::math::{cosd, sind};
use artkit::scene::Obj;

// --- the park bench: cast-iron ends, four seat slats and three back slats -------------------------------------
pub(super) const BENCH_SEAT: f64 = 0.45; // the slats' top: `seats.bench` (furniture.json)

/// Game (height, depth) points of a scroll's spiral about (y, z), from radius `r0` at angle `a0` (degrees, 0 towards
/// +depth, 90 up) in towards `r1`, `turns` round (s=1: up first, then back).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub(super) fn spiral(
    y: f64,
    z: f64,
    r0: f64,
    r1: f64,
    a0: f64,
    turns: f64,
    n: usize,
    s: f64,
) -> Vec<(f64, f64)> {
    (0..=n)
        .map(|i| {
            let (i, nf) = (i as f64, n as f64);
            (
                y + (r0 + (r1 - r0) * i / nf) * sind(a0 + s * 360.0 * turns * i / nf),
                z + (r0 + (r1 - r0) * i / nf) * cosd(a0 + s * 360.0 * turns * i / nf),
            )
        })
        .collect()
}

/// A cast-iron bench end in the side plane at game x, each line one cast piece: the back leg sweeping up into the
/// back's support (`back`, (height, depth) points from the foot), the front leg (`front`, from the foot; with an
/// `arm`, rising into it and running back to the support), the seat rail (`rail`: height, depth from, to), an arched
/// stretcher (`stretcher`: height, depth from, to), a pad under each foot; `arm` (height, depth from, to): a flat
/// armrest along the arm's top; `scroll` (height, depth, radius): a scroll in the angle of the leg and the arm.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub(super) fn cast_end(
    p: &str,
    x: f64,
    back: &[(f64, f64)],
    front: &[(f64, f64)],
    rail: (f64, f64, f64),
    stretcher: (f64, f64, f64),
    arm: Option<(f64, f64, f64)>,
    scroll: Option<(f64, f64, f64)>,
) {
    let mat = M.iron;
    let pts = |yz: &[(f64, f64)]| -> Vec<(f64, f64, f64)> {
        yz.iter().map(|&(y, z)| (x, y, z)).collect()
    };
    bar(
        &format!("{p}back"),
        &pts(back),
        0.03,
        0.044,
        mat,
        Bar {
            bend: 0.12,
            steps: 3,
            r: Some(0.008),
            ..Bar::default()
        },
    );
    bar(
        &format!("{p}front"),
        &pts(front),
        0.03,
        0.04,
        mat,
        Bar {
            bend: 0.05,
            steps: 3,
            r: Some(0.008),
            ..Bar::default()
        },
    );
    let (y, z0, z1) = rail;
    bar(
        &format!("{p}rail"),
        &pts(&[(y, z0), (y, z1)]),
        0.026,
        0.04,
        mat,
        Bar {
            bend: 0.0,
            r: Some(0.006),
            ..Bar::default()
        },
    );
    let (y, z0, z1) = stretcher;
    bar(
        &format!("{p}stretcher"),
        &pts(&[(y - 0.01, z0), (y + 0.035, (z0 + z1) / 2.0), (y - 0.01, z1)]),
        0.02,
        0.026,
        mat,
        Bar {
            bend: 0.0,
            r: Some(0.005),
            resolution: 3,
            ..Bar::default()
        },
    );
    for (_fy, fz) in [back[0], front[0]] {
        fbox!(
            F,
            format!("{p}foot{}", float_repr(fz)),
            0.05,
            0.016,
            0.075,
            (x, 0.008, fz),
            mat,
            bevel = 0.006,
            bevel_segments = 1
        );
    }
    if let Some((y, z0, z1)) = arm {
        plank(
            &format!("{p}armrest"),
            (x, y, z0),
            (x, y, z1),
            0.014,
            mat,
            &end_round(0.056, z1 - z0, None, 3, false),
            (0, 1, 0),
            0.004,
        );
    }
    if let Some((y, z, r)) = scroll {
        bar(
            &format!("{p}scroll"),
            &pts(&spiral(y, z, r, r * 0.25, 0.0, 1.15, 13, 1.0)),
            0.02,
            0.012,
            mat,
            Bar {
                bend: 0.0,
                r: Some(0.0),
                ..Bar::default()
            },
        );
    }
}

pub fn bench() -> Vec<Obj> {
    for s in [-1, 1] {
        let x = s as f64 * 0.78;
        // the back leg splays back at the foot, rises in line with the back slats and curls back at the top
        cast_end(
            &format!("end{s}"),
            x,
            &[
                (0.0, -0.135),
                (0.08, -0.118),
                (0.4, -0.15),
                (0.78, -0.192),
                (0.85, -0.198),
                (0.868, -0.218),
                (0.852, -0.232),
            ],
            &[
                (0.0, 0.205),
                (0.42, 0.212),
                (0.6, 0.235),
                (0.66, 0.256),
                (0.664, 0.2),
                (0.664, -0.18),
            ],
            (0.395, -0.17, 0.236),
            (0.1, -0.125, 0.208),
            Some((0.678, -0.175, 0.275)),
            Some((0.565, 0.17, 0.048)),
        );
        // carriage bolts through the slats into the ends
        let seat: Vec<(f64, f64, f64)> = [-0.1, 0.0, 0.1, 0.2]
            .iter()
            .map(|&z| (x, BENCH_SEAT + 0.002, z))
            .collect();
        bolts(&format!("bolt_seat{s}"), &seat, M.iron, true);
        let back: Vec<(f64, f64, f64)> = [(0.6, -0.2), (0.72, -0.2168), (0.84, -0.2336)]
            .iter()
            .map(|&(y, z)| (x, y, z + 0.017))
            .collect();
        bolts(&format!("bolt_back{s}"), &back, M.iron, false);
    }
    for (k, z) in [-0.1, 0.0, 0.1, 0.2].into_iter().enumerate() {
        fbox!(
            F,
            format!("seat{k}"),
            1.74,
            0.035,
            0.085,
            (0, BENCH_SEAT - 0.0175, z),
            M.park_slats,
            bevel = 0.012
        );
    }
    for (k, (y, z)) in [(0.6, -0.2), (0.72, -0.2168), (0.84, -0.2336)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            F,
            format!("back{k}"),
            1.74,
            0.085,
            0.03,
            (0, y, z),
            M.park_slats,
            rot = (-8, 0, 0),
            bevel = 0.012
        );
    }
    finish("bench", false)
}
