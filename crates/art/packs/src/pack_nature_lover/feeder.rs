//! The bird feeder post (1x1): a cedar post, a feeder house on a tray, a seed tube, flowers round its foot.

use super::common::{_flower, _m, _rosette, _sq, M};
use artkit::kit;
use artkit::math::{P3, TAU, cos, cosd, sin, sind};
use artkit::parts::{join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, jitter, revolve, settle, soft, squash};

const PZ: f64 = -0.05; // the post's depth

/// The tray on braces, its lip mitred round; the house on it: gable walls cut with a round entry, a pitched roof
/// of two boards with a ridge cap; seed heaped on the tray.
fn _feeder_house(f: &GameFrame) {
    fbox!(
        f,
        "tray",
        0.36,
        0.03,
        0.3,
        (0, 1.5, PZ),
        M.cedar,
        bevel = 0.006,
        bevel_segments = 1
    );
    kit::moulding!(
        f,
        "tray_lip",
        &[
            (-0.18, PZ - 0.15),
            (0.18, PZ - 0.15),
            (0.18, PZ + 0.15),
            (-0.18, PZ + 0.15)
        ],
        &_sq_profile(0.018, 0.03),
        M.cedar,
        at = 1.515,
        plane = Facing::Top
    );
    for s in [-1, 1] {
        // braces under the tray
        fsweep!(
            f,
            format!("brace{s}"),
            &[
                (0.0, 1.32, PZ + s as f64 * 0.03),
                (0.0, 1.48, PZ + s as f64 * 0.13)
            ],
            0.0,
            M.cedar_dark,
            profile = _sq(0.03, None),
            up = (1, 0, 0)
        );
    }
    let seed = soft!(
        f,
        "seed",
        0.3,
        0.02,
        0.22,
        (0, 1.524, PZ + 0.03),
        M.seed,
        round = 0.008,
        puff = (0, 0.008, 0),
        flat = 3,
        round_segments = 1
    );
    jitter!(seed, 0.004, size = 0.03, seed = 1.0);
    let gable = [
        (-0.12, 0.0),
        (0.12, 0.0),
        (0.12, 0.17),
        (0.0, 0.24),
        (-0.12, 0.17),
    ];
    let hole: Vec<(f64, f64)> = (0..12)
        .map(|k| {
            (
                0.03 * cos(TAU * k as f64 / 12.0),
                0.11 + 0.03 * sin(TAU * k as f64 / 12.0),
            )
        })
        .collect();
    for (z, holes) in [(PZ + 0.08, vec![hole]), (PZ - 0.08, vec![])] {
        fextrude!(
            f,
            format!("gable{}", artkit::json::float_repr(z)),
            &gable,
            0.02,
            (0, 1.515, z),
            M.house,
            holes = holes,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    for s in [-1, 1] {
        fbox!(
            f,
            format!("house_side{s}"),
            0.02,
            0.17,
            0.14,
            (s as f64 * 0.11, 1.6, PZ),
            M.house
        );
    }
    fbox!(
        f,
        "house_inside",
        0.2,
        0.15,
        0.004,
        (0, 1.6, PZ + 0.065),
        M.dark
    ); // seen through the entry
    fsweep!(
        f,
        "perch_dowel",
        &[(0, 1.585, PZ + 0.09), (0, 1.585, PZ + 0.14)],
        0.005,
        M.cedar_dark,
        verts = 6
    );
    for s in [-1, 1] {
        // the roof's two boards, then its ridge cap
        fbox!(
            f,
            format!("roof{s}"),
            0.17,
            0.022,
            0.29,
            (s as f64 * 0.075, 1.712, PZ),
            M.roof,
            rot = (0, -s * 30, 0),
            bevel = 0.005,
            bevel_segments = 1
        );
    }
    fsweep!(
        f,
        "ridge",
        &[(0, 1.77, PZ - 0.155), (0, 1.77, PZ + 0.155)],
        0.0,
        M.cedar_dark,
        profile = vec![(0.0, 0.013), (0.022, -0.006), (-0.022, -0.006)],
        up = (0, 0, 1)
    );
}

/// A moulding's profile (inset, up): a square-edged lip `w` wide and `h` high, its top edge eased.
fn _sq_profile(w: f64, h: f64) -> Vec<(f64, f64)> {
    vec![
        (0.0, 0.0),
        (w, 0.0),
        (w, h - 0.004),
        (w - 0.004, h),
        (0.004, h),
        (0.0, h - 0.004),
    ]
}

/// A seed tube hanging from the post's arm: a clear body full of seed, a domed lid, a dish with two feeding ports
/// and a perch through them.
fn _seed_tube(f: &GameFrame, x: f64) {
    fsweep!(
        f,
        "arm",
        &[(0.04, 1.3, PZ), (x - 0.04, 1.31, PZ), (x, 1.33, PZ)],
        0.0,
        M.cedar_dark,
        profile = _sq(0.03, Some(0.035)),
        up = (0, 0, 1),
        resolution = 2
    );
    fsweep!(
        f,
        "hanger",
        &kit::fillet!(&[(x, 1.33, PZ), (x, 1.3, PZ), (x - 0.01, 1.27, PZ)], 0.01),
        0.003,
        M.steel,
        verts = 4
    );
    revolve!(
        "seed_tube",
        &[(0.0, 0.0), (0.04, 0.0), (0.04, 0.24), (0.0, 0.24)],
        at = f.at(x, 1.01, PZ),
        material = M.tube,
        segments = 14
    );
    revolve!(
        "tube_lid",
        &[
            (0.0, 0.0),
            (0.046, 0.0),
            (0.046, 0.012),
            (0.03, 0.03),
            (0.008, 0.034),
            (0.0, 0.034)
        ],
        at = f.at(x, 1.245, PZ),
        material = M.roof,
        segments = 14
    );
    fcyl!(
        f,
        "tube_seed",
        0.036,
        0.2,
        (x, 1.11, PZ),
        M.seed,
        verts = 12
    );
    revolve!(
        "tube_dish",
        &[
            (0.0, -0.01),
            (0.05, -0.01),
            (0.062, 0.012),
            (0.056, 0.016),
            (0.042, 0.004),
            (0.0, 0.004)
        ],
        at = f.at(x, 0.995, PZ),
        material = M.roof,
        segments = 14
    );
    for s in [-1, 1] {
        fcyl!(
            f,
            format!("port{s}"),
            0.014,
            0.012,
            (x + s as f64 * 0.042, 1.04, PZ),
            M.roof,
            rot = (0, 90, 0),
            verts = 10
        );
    }
    fsweep!(
        f,
        "perch",
        &[(x - 0.09, 1.02, PZ), (x + 0.09, 1.02, PZ)],
        0.0045,
        M.steel,
        verts = 6
    );
}

pub fn bird_feeder() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            f,
            "post",
            0.08,
            1.49,
            0.08,
            (0, 0.745, PZ),
            M.cedar_dark,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            "post_socket",
            0.11,
            0.14,
            0.11,
            (0, 0.07, PZ),
            M.steel,
            bevel = 0.006,
            bevel_segments = 1
        );
        _feeder_house(f);
        _seed_tube(f, 0.25);
        kit::songbird!(
            f,
            "goldfinch",
            0.3,
            1.025,
            PZ,
            M.bird_yellow,
            M.bird_yellow,
            M.beak,
            M.eye,
            turn = 60,
            size = 0.85
        );
        // a round bed at its foot, edged with stone, a ring of flowering clumps in it
        let ring: Vec<(f64, f64)> = (0..360)
            .step_by(15)
            .map(|a| (0.33 * cosd(a as f64), PZ + 0.33 * sind(a as f64)))
            .collect();
        kit::moulding!(
            f,
            "bed_edge",
            &ring,
            &[
                (-0.015, 0.0),
                (0.02, 0.0),
                (0.02, 0.025),
                (0.012, 0.04),
                (-0.007, 0.04),
                (-0.015, 0.025)
            ],
            M.pebble,
            at = 0.0,
            plane = Facing::Top
        );
        fcyl!(
            f,
            "bed_soil",
            0.32,
            0.024,
            (0, 0.012, PZ),
            M.soil,
            verts = 24
        );
        for i in 0..8 {
            let a = (i * 45) as f64 + 22.5;
            let (x, z) = (cosd(a) * 0.22, PZ + sind(a) * 0.22);
            _rosette(
                f,
                &format!("clump{i}_"),
                x,
                0.02,
                z,
                10,
                0.09,
                0.06,
                &[M.leaf, M.leaf_light],
                (32.0, 66.0),
                0.3,
                "oval",
                0.3,
                None,
                3,
                3,
                30 + i as i128,
                0.0,
            );
            for k in 0..3 {
                let (fx, fz) = (
                    x + 0.035 * cosd(a + (120 * k) as f64),
                    z + 0.035 * sind(a + (120 * k) as f64),
                );
                let fy = 0.14 + 0.03 * (k % 2) as f64;
                fsweep!(
                    f,
                    format!("clump_stem{i}{k}"),
                    &[(x, 0.02, z), (fx, fy, fz)],
                    0.003,
                    M.stem,
                    verts = 3
                );
                _flower(
                    f,
                    &format!("clump_flower{i}{k}"),
                    fx,
                    fy,
                    fz,
                    0.034,
                    _m(["pink", "yellow", "lilac", "white"][(i + k) % 4]),
                    Some(if (i + k) % 4 != 1 { M.yellow } else { M.carrot }),
                    5,
                    25.0,
                    -a + 90.0,
                );
            }
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The bluebird that comes to the tray, built at full size and scaled to nothing: `bird` brings it in.
    let bird = new_parts(|| {
        kit::songbird!(
            f,
            "bluebird",
            -0.1,
            1.535,
            0.1,
            M.bird_blue,
            M.bird_breast,
            M.beak,
            M.eye,
            turn = -20,
            head = 20
        );
    });
    let bird = join(&bird, "bird", Some(f.at(-0.1, 1.535, 0.1)));
    bird.set_scale((0.001, 0.001, 0.001));
    let objs = settle!(centre = false);
    squash!(bird, "bird", 1000, seconds = 0.5);
    objs
}
