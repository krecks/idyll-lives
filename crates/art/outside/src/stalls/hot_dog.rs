//! The hot dog stand: a red cart with a hot plate across its 2 x 1 tiles, under a striped awning on four posts; its
//! hot plate's lid lifts as the vendor reaches in.

use super::common::{M, valance, wheel};
use artkit::forms::{by, rounded_rect};
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, pow, sin};
use artkit::motion::{Swing, swing};
use artkit::parts::{hinged, join, new_parts, turn};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fcyl, fextrude, fsweep, kit_pull, loft, ring, settle, soft};

// --- the hot dog stand: a red cart with a hot plate, under a striped awning on four posts -----------------
/// A sausage lying along the depth (or across, `Axis::X`): a tube bowed a little, rounded off at its ends.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn sausage(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    length: f64,
    r: f64,
    material: Mat,
    along: Axis,
    bend: f64,
) -> Obj {
    let mut pts = Vec::new();
    for i in 0..7 {
        let fi = i as f64 / 6.0 - 0.5;
        let off = bend * (1.0 - pow(2.0 * fi, 2.0));
        pts.push(if along == Axis::Z {
            (x + off, y, z + fi * length)
        } else {
            (x + fi * length, y + off, z)
        });
    }
    fsweep!(
        f,
        name,
        &pts,
        r,
        material,
        verts = 8,
        resolution = 1,
        scale = by(|t| 0.35_f64.max(1.0_f64.min(sin(PI * t) * 2.2)))
    )
}

/// A split bun lying along the depth or across: two soft halves, a little apart.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn bun(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, length: f64, material: Mat, along: Axis) {
    for s in [-1, 1] {
        let sf = s as f64;
        if along == Axis::Z {
            soft!(
                f,
                format!("{name}{s}"),
                0.034,
                0.04,
                length,
                (x + sf * 0.02, y, z),
                material,
                round = 0.017,
                round_segments = 1,
                flat = 1
            );
        } else {
            soft!(
                f,
                format!("{name}{s}"),
                length,
                0.04,
                0.034,
                (x, y, z + sf * 0.02),
                material,
                round = 0.017,
                round_segments = 1,
                flat = 1
            );
        }
    }
}

/// A squeeze bottle standing at game (x, y, z): a soft body, a shoulder, a cap and a long nozzle.
fn squeezy(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, material: Mat, cap: Mat) {
    let P3 {
        x: bx,
        y: by_,
        z: bz,
    } = f.at(x, y, z);
    let rows = [
        (0.0, 0.034),
        (0.01, 0.04),
        (0.12, 0.04),
        (0.15, 0.03),
        (0.165, 0.016),
    ];
    let secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(h, r)| ring!(r, r, z = bz + h, verts = 10, at = (bx, by_)))
        .collect();
    loft!(name, &secs, material = material);
    fcyl!(
        f,
        format!("{name}_cap"),
        0.018,
        0.016,
        (x, y + 0.172, z),
        cap,
        verts = 8
    );
    fcone!(
        f,
        format!("{name}_nozzle"),
        0.01,
        0.003,
        0.045,
        (x, y + 0.2, z),
        cap,
        verts = 6
    );
}

pub fn hot_dog_stand() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        soft!(
            f,
            "stand",
            1.5,
            0.66,
            0.66,
            (-0.1, 0.62, -0.05),
            M.stand,
            round = 0.035,
            round_segments = 2,
            flat = 1
        );
        // the steel top with a rolled front edge, a lower shelf rail along the front
        soft!(
            f,
            "top",
            1.56,
            0.05,
            0.72,
            (-0.1, 0.975, -0.05),
            M.steel,
            round = 0.02,
            round_segments = 2,
            flat = 1
        );
        fbox!(
            f,
            "chassis",
            1.4,
            0.04,
            0.5,
            (-0.1, 0.27, -0.05),
            M.steel,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "trim",
            1.52,
            0.03,
            0.68,
            (-0.1, 0.3, -0.05),
            M.steel,
            bevel = 0.008,
            bevel_segments = 1
        );
        for x in [-0.68, 0.5] {
            for z in [0.24, -0.34] {
                wheel(
                    &f,
                    &format!("wheel{}{}", float_repr(x), float_repr(z)),
                    x,
                    z,
                    0.2,
                    6,
                );
            }
        }
        // the sign on the front: a framed plate, a hot dog in its bun with a mustard zigzag
        crate::eased(fextrude!(
            f,
            "sign_frame",
            &rounded_rect(0.66, 0.26, 0.02, 2),
            0.016,
            (-0.1, 0.72, 0.284),
            M.steel,
            holes = [rounded_rect(0.62, 0.22, 0.012, 2)]
        ));
        fbox!(
            f,
            "sign",
            0.62,
            0.22,
            0.02,
            (-0.1, 0.72, 0.29),
            M.sign,
            bevel = 0.004,
            bevel_segments = 1
        );
        soft!(
            f,
            "sign_bun",
            0.44,
            0.07,
            0.02,
            (-0.1, 0.705, 0.302),
            M.bun,
            round = 0.01,
            round_segments = 2,
            flat = 1
        );
        sausage(
            &f,
            "sign_sausage",
            -0.1,
            0.735,
            0.31,
            0.5,
            0.024,
            M.sausage,
            Axis::X,
            0.0,
        );
        let zigzag: Vec<(f64, f64, f64)> = (0..13)
            .map(|i| {
                (
                    -0.1 - 0.18 + 0.03 * i as f64,
                    0.758 + 0.006 * (if i % 2 == 0 { 1.0 } else { -1.0 }),
                    0.334,
                )
            })
            .collect();
        fsweep!(f, "sign_mustard", &zigzag, 0.004, M.mustard, verts = 4);
        // the hot plate: a steel tray, a dark ribbed plate, sausages and buns under the lid
        fbox!(
            f,
            "plate_tray",
            0.6,
            0.04,
            0.4,
            (-0.35, 1.02, -0.1),
            M.steel,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(f, "plate", 0.54, 0.008, 0.34, (-0.35, 1.042, -0.1), M.grill);
        for k in 0..8 {
            fbox!(
                f,
                format!("rib{k}"),
                0.54,
                0.004,
                0.006,
                (-0.35, 1.047, -0.25 + 0.042 * k as f64),
                M.grill
            );
        }
        for k in 0..5 {
            sausage(
                &f,
                &format!("sausage{k}"),
                -0.55 + 0.08 * k as f64,
                1.066,
                -0.12,
                0.22,
                0.018,
                M.sausage,
                Axis::Z,
                0.006,
            );
        }
        for (k, x) in [-0.6, -0.13].into_iter().enumerate() {
            bun(&f, &format!("bun{k}"), x, 1.07, -0.12, 0.2, M.bun, Axis::Z);
        }
        // sauces, a napkin box with a napkin pulled up, a stack of paper trays at the right
        for (x, m) in [(0.3, M.mustard), (0.42, M.ketchup)] {
            squeezy(&f, &format!("bottle{}", float_repr(x)), x, 1.0, 0.1, m, m);
        }
        fbox!(
            f,
            "napkins",
            0.14,
            0.08,
            0.1,
            (0.36, 1.04, -0.15),
            M.steel,
            bevel = 0.006,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "napkin",
            &[
                (-0.04, 0.0),
                (0.04, 0.0),
                (0.03, 0.05),
                (0.0, 0.06),
                (-0.035, 0.045)
            ],
            0.004,
            (0.36, 1.075, -0.15),
            M.sign
        );
        for k in 0..3 {
            fbox!(
                f,
                format!("tray{k}"),
                0.16,
                0.012,
                0.08,
                (0.12, 1.006 + 0.012 * k as f64, -0.2),
                M.sign,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // the posts and the awning: six stripes leaning down to the front, a scalloped valance along it
        for x in [-0.82, 0.62] {
            for (z, top) in [(-0.4, 2.35), (0.38, 2.23)] {
                fsweep!(
                    f,
                    format!("post{}{}", float_repr(x), float_repr(z)),
                    &[(x, 1.0, z), (x, top, z)],
                    0.02,
                    M.steel,
                    verts = 8
                );
                fcyl!(
                    f,
                    format!("post_foot{}{}", float_repr(x), float_repr(z)),
                    0.03,
                    0.02,
                    (x, 1.01, z),
                    M.steel,
                    verts = 8
                );
            }
        }
        let mut mats = Vec::new();
        for i in 0..6 {
            let x = -0.85 + 0.25 * i as f64 + 0.125;
            let m = if i % 2 != 0 { M.awning } else { M.awning_white };
            mats.push(m);
            fbox!(
                f,
                format!("stripe{i}"),
                0.25,
                0.04,
                0.98,
                (x, 2.3, 0.0),
                m,
                rot = (8, 0, 0),
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        valance(&f, "flap", -0.85, 0.65, 2.25, 0.488, 0.1, &mats, 2);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the hot plate's lid, hinged on its front edge: a rounded steel hood, a bar handle along its back edge, where
    // the vendor behind the cart lifts it, so it opens away from them (built with the handle at the front, turned)
    let parts = new_parts(|| {
        soft!(
            f,
            "lid",
            0.6,
            0.1,
            0.4,
            (-0.35, 1.09, -0.1),
            M.steel,
            round = 0.035,
            round_segments = 2,
            flat = 1
        );
        kit_pull!(
            f,
            "lid_handle",
            -0.35,
            1.1,
            0.1,
            0.22,
            M.steel,
            r = 0.008,
            out = 0.03
        );
    });
    let lid = turn(
        hinged(join(&parts, "lid", None), f.at(-0.35, 1.04, -0.1)),
        180.0,
        Axis::Z,
    );
    let lid = hinged(lid, f.at(-0.35, 1.04, 0.1));
    settle!(centre = false);
    swing(
        lid,
        "lid_open",
        75.0,
        Swing {
            seconds: 0.9,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    meshes()
}
