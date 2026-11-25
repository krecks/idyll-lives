//! KD1: the crib with its changing table.

use super::common::{heart, mattress, moon, place, star, still};
use super::{F, M};
use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::kit::{self, fillet};
use artkit::materials::Mat;
use artkit::math::{Axis, PI, cos, radians, sin};
use artkit::motion::{Spin, spin};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsphere, fsweep, jitter, slide, soft};

// An oblong cot, a 1.18 x 0.58 m mattress (a 60 x 120 cot's; review M-kids-01: it was 0.80 m square) running
// across the left of the 2 x 1 tiles, its back on the wall's face, and a changing chest at the right. The baby lies
// along it (furniture.json `beds.crib` `across`) at slot 0 (game x -0.5), the carers reach in over the
// front rail (characters.ts `CUDDLE.rail`).
/// Where the baby lies (slot 0 of the 2 x 1).
const CRIB_X: f64 = -0.5;
/// The mattress top (furniture.json beds.crib).
const CRIB_TOP: f64 = 0.56;
/// The cot's end posts (game x).
const COT_X: (f64, f64) = (-0.955, 0.275);
/// Its back and front posts (game depth): the back on the wall's face.
const COT_Z: (f64, f64) = (-0.405, 0.225);
const COT_MID: (f64, f64) = (
    (0.0 + COT_X.0 + COT_X.1) / 2.0,
    (0.0 + COT_Z.0 + COT_Z.1) / 2.0,
);
/// Between the posts.
const COT_LEN: f64 = COT_X.1 - COT_X.0 - 0.05;
const COT_DEP: f64 = COT_Z.1 - COT_Z.0 - 0.05;

/// `n` round spindles between `a` and `b` along x (`along` 'x') or depth ('z'), the other coordinate fixed,
/// from rail to rail.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn spindles(
    name: &str,
    a: f64,
    b: f64,
    along: (char, f64),
    material: Mat,
    n: usize,
    y0: f64,
    y1: f64,
) {
    for i in 0..n {
        let t = a + (b - a) * (i as f64 + 0.5) / n as f64;
        let (x, z) = if along.0 == 'x' {
            (t, along.1)
        } else {
            (along.1, t)
        };
        fsweep!(
            F,
            format!("{name}{i}"),
            &[(x, y0, z), (x, y1, z)],
            0.011,
            material,
            verts = 8
        );
    }
}

/// A long side's top and bottom rails and spindles, between the posts: the top rail rounded over to hold.
fn crib_rails(name: &str, z: f64) {
    fsweep!(
        F,
        format!("{name}_top"),
        &[(COT_X.0 + 0.02, 0.9, z), (COT_X.1 - 0.02, 0.9, z)],
        0,
        M.crib,
        profile = rounded_rect(0.042, 0.04, 0.016, 2),
        up = (0, 0, 1)
    );
    fbox!(
        F,
        format!("{name}_bottom"),
        COT_LEN + 0.01,
        0.04,
        0.035,
        (COT_MID.0, 0.47, z),
        M.crib,
        bevel = 0.008,
        bevel_segments = 1
    );
    spindles(
        &format!("{name}_spindle"),
        COT_X.0 + 0.03,
        COT_X.1 - 0.03,
        ('x', z),
        M.crib,
        11,
        0.47,
        0.9,
    );
}

/// A turned finial on a post's top at height `y`: a collar, a waist and a ball.
fn finial(name: &str, x: f64, z: f64, y: f64, material: Mat) -> Obj {
    kit::turned!(
        F,
        name,
        x,
        y,
        z,
        &[
            (0.0, 0.0),
            (0.025, 0.0),
            (0.025, 0.006),
            (0.012, 0.016),
            (0.02, 0.026),
            (0.021, 0.036),
            (0.013, 0.047),
            (0.0, 0.05),
        ],
        material,
        segments = 8
    )
}

/// A solid end board between the posts at `x`, its top arched, a shape cut through it.
fn crib_end(name: &str, x: f64, hole: Vec<(f64, f64)>) {
    let (z0, z1) = (COT_Z.0 + 0.025, COT_Z.1 - 0.025);
    let n = 10;
    let mut outline = vec![(z0, 0.43), (z1, 0.43)];
    outline.extend((0..=n).map(|i| {
        (
            z1 - (z1 - z0) * i as f64 / n as f64,
            0.885 + 0.04 * sin(PI * i as f64 / n as f64),
        )
    }));
    fextrude!(
        F,
        name,
        &outline,
        0.022,
        (x, 0, 0),
        M.crib,
        plane = Facing::Side,
        holes = vec![hole],
        bevel = 0.006,
        bevel_segments = 1
    );
}

fn changing_table() {
    let (x, z, w) = (0.635, -0.18, 0.62);
    fbox!(
        F,
        "plinth",
        w - 0.06,
        0.05,
        0.44,
        (x, 0.025, z - 0.01),
        M.crib,
        bevel = 0.006,
        bevel_segments = 1
    );
    fbox!(
        F,
        "chest",
        w - 0.02,
        0.73,
        0.5,
        (x, 0.415, z),
        M.crib,
        bevel = 0.01,
        bevel_segments = 2
    );
    for (i, y) in [0.165, 0.4, 0.635].into_iter().enumerate() {
        kit::shaker!(
            F,
            format!("drawer{i}"),
            w - 0.08,
            0.215,
            (x, y, z + 0.259),
            0.018,
            M.crib_wood,
            rail = 0.035,
            recess = 0.004
        );
        kit::knob!(
            F,
            format!("knob{i}"),
            x,
            y,
            z + 0.268,
            0.014,
            M.crib,
            segments = 8
        );
    }
    fbox!(
        F,
        "chest_top",
        w + 0.02,
        0.025,
        0.52,
        (x, 0.7925, z),
        M.crib_wood,
        bevel = 0.008,
        bevel_segments = 2
    );
    // a contoured changing pad: a soft middle between two raised bolsters, a guard rail round its back and sides
    soft!(
        F,
        "pad",
        w - 0.1,
        0.035,
        0.42,
        (x, 0.825, z + 0.01),
        M.crib_bedding,
        round = 0.014,
        puff = 0.006,
        flat = 2
    );
    for s in [-1, 1] {
        soft!(
            F,
            format!("pad_side{s}"),
            0.075,
            0.06,
            0.42,
            (x + s as f64 * (w / 2.0 - 0.085), 0.838, z + 0.01),
            M.crib_bedding,
            round = 0.026,
            flat = 1
        );
    }
    let (gx0, gx1, gz0, gz1) = (
        x - w / 2.0 + 0.0125,
        x + w / 2.0 - 0.0125,
        z - 0.245,
        z + 0.25,
    );
    let pts = fillet(
        &[
            (gx0, 0.845, gz1),
            (gx0, 0.845, gz0),
            (gx1, 0.845, gz0),
            (gx1, 0.845, gz1),
        ],
        0.04,
        3,
    );
    let guard = fsweep!(
        F,
        "guard",
        &pts,
        0,
        M.crib,
        profile = rounded_rect(0.08, 0.025, 0.011, 2),
        up = (0, 0, 1)
    );
    bevel_edges(guard, 0.004, 1, 50.0, &EdgeOrder::Corners); // its ends' edges
}

/// The mobile: a turned hub on the arm's end, four arms curving down, and a toy on a string under each (a star,
/// a heart, a moon and a ball), each facing out, turning about the hub.
fn mobile() -> Obj {
    // over the baby, clear of arms reaching in over the front rail
    let (hx, hy, hz) = (CRIB_X, 0.975, -0.17);
    let parts = new_parts(|| {
        kit::turned!(
            F,
            "hub",
            hx,
            hy - 0.0125,
            hz,
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.032, 0.007),
                (0.026, 0.018),
                (0.012, 0.026),
                (0.0, 0.028),
            ],
            M.mobile,
            segments = 12
        );
        let toys = [
            ("star", M.mobile),
            ("heart", M.mobile_pink),
            ("moon", M.mobile_blue),
            ("ball", M.mobile_pink),
        ];
        for (k, (kind, m)) in toys.into_iter().enumerate() {
            let a = radians((90 * k + 45) as f64);
            let (c, s) = (cos(a), sin(a));
            let (x, z) = (hx + 0.12 * c, hz + 0.12 * s);
            fsweep!(
                F,
                format!("arm{k}"),
                &[
                    (hx, hy, hz),
                    (hx + 0.06 * c, hy + 0.004, hz + 0.06 * s),
                    (x, hy - 0.012, z)
                ],
                0.0045,
                M.mobile,
                verts = 6,
                resolution = 3
            );
            fsweep!(
                F,
                format!("string{k}"),
                &[(x, hy - 0.012, z), (x, 0.872, z)],
                0.0015,
                M.crib,
                verts = 4
            );
            // its face out from the hub
            let turn = (0.0, 0.0, artkit::math::degrees(a) - 90.0);
            if kind == "ball" {
                fsphere!(
                    F,
                    format!("ball{k}"),
                    0.03,
                    (x, 0.845, z),
                    m,
                    segments = 12,
                    rings = 8
                );
                continue;
            }
            let outline = match kind {
                "star" => star(0.0, 0.0, 0.042, 0.45, 5, 90.0),
                "heart" => heart(0.0, 0.0, 0.034, 20),
                _ => moon(0.0, 0.0, 0.036, 10),
            };
            fextrude!(
                F,
                format!("{kind}{k}"),
                &outline,
                0.014,
                (x, 0.84, z),
                m,
                rot = turn,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
    });
    hinged(join(&parts, "mobile", None), F.at(hx, hy, hz))
}

pub fn crib() -> Vec<Obj> {
    let (cx, cz) = COT_MID;
    for x in [COT_X.0, COT_X.1] {
        for z in [COT_Z.0, COT_Z.1] {
            fbox!(
                F,
                format!("post{}{}", r(x), r(z)),
                0.05,
                0.92,
                0.05,
                (x, 0.46, z),
                M.crib,
                bevel = 0.01,
                bevel_segments = 2
            );
            finial(&format!("cap{}{}", r(x), r(z)), x, z, 0.92, M.crib);
        }
    }
    fbox!(
        F,
        "platform",
        COT_LEN + 0.04,
        0.04,
        COT_DEP + 0.04,
        (cx, 0.44, cz),
        M.crib,
        bevel = 0.008,
        bevel_segments = 1
    );
    mattress(
        "mattress",
        COT_LEN,
        0.1,
        COT_DEP,
        CRIB_TOP,
        cz,
        cx,
        Some(M.crib_bedding),
        0.006,
        0.006,
        0.025,
        2,
    );
    // the blanket folded at the foot end (the baby's head to the left, furniture.json beds.crib `across`), a band near
    // its fold
    let b = soft!(
        F,
        "blanket",
        0.2,
        0.028,
        COT_DEP - 0.08,
        (COT_X.1 - 0.17, CRIB_TOP + 0.017, cz),
        M.crib_blanket,
        round = 0.012,
        puff = 0.005,
        flat = 2
    );
    jitter!(b, 0.002, size = 0.1, seed = 3.0, up = 0.5);
    fbox!(
        F,
        "blanket_stripe",
        0.03,
        0.036,
        COT_DEP - 0.076,
        (COT_X.1 - 0.22, CRIB_TOP + 0.017, cz),
        M.mobile_blue,
        bevel = 0.012,
        bevel_segments = 2
    );
    crib_rails("back", COT_Z.0);
    // the ends: solid boards between the posts, arched, a heart cut through the head's and a star the foot's
    crib_end(
        "end_head",
        COT_X.0,
        heart(0.0, 0.0, 0.07, 20)
            .iter()
            .map(|&(u, v)| (COT_Z.0 + 0.315 + u, 0.71 + v))
            .collect(),
    );
    crib_end(
        "end_foot",
        COT_X.1,
        star(0.0, 0.0, 0.08, 0.45, 5, 90.0)
            .iter()
            .map(|&(u, v)| (COT_Z.0 + 0.315 + u, 0.7 + v))
            .collect(),
    );
    // the mobile's arm, bent from one tube and clamped to the back rail
    fbox!(
        F,
        "clamp",
        0.035,
        0.07,
        0.06,
        (CRIB_X, 0.9, COT_Z.0 + 0.005),
        M.mobile,
        bevel = 0.008,
        bevel_segments = 1
    );
    let pts = fillet(
        &[
            (CRIB_X, 0.88, -0.37),
            (CRIB_X, 1.0, -0.37),
            (CRIB_X, 1.0, -0.17),
            (CRIB_X, 0.985, -0.17),
        ],
        0.05,
        3,
    );
    fsweep!(F, "arm", &pts, 0.008, M.mobile, verts = 8);
    changing_table();
    still("body", None);

    let parts = new_parts(|| crib_rails("front", COT_Z.1));
    let side = join(&parts, "front_rail", None);
    let mobile = mobile();
    place(0.0); // built in place: the back posts on the wall's face
    slide!(side, "side_down", (0, 0, -0.25), seconds = 0.9);
    spin(
        mobile,
        "mobile",
        Spin {
            turns: 1.0,
            seconds: 6.0,
            axis: Axis::Z,
        },
    );
    meshes()
}
