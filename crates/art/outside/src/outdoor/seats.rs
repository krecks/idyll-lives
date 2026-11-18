//! OU1's seats: the park bench, the folding bistro set, the picnic table, the garden and Adirondack chairs and
//! the three-seat garden bench.

use super::common::{
    Bar, F, M, bar, board, bolts, cup, end_round, finish, jug, plank, rod, side_slab, v,
};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, PI, V3, cos, cosd, radians, sin, sind};
use artkit::scene::Obj;
use artkit::{cylinder, fbox, fextrude, fsweep, loft, ring, soft};

// --- the park bench: cast-iron ends, four seat slats and three back slats -------------------------------------
const BENCH_SEAT: f64 = 0.45; // the slats' top: `seats.bench` (furniture.json)

/// Game (height, depth) points of a scroll's spiral about (y, z), from radius `r0` at angle `a0` (degrees, 0 towards
/// +depth, 90 up) in towards `r1`, `turns` round (s=1: up first, then back).
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn spiral(
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
fn cast_end(
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

// --- the folding bistro set: a slatted table between two folding chairs --------------------------------------

/// A folding chair centred at game depth `c`, its front towards `facing` (+1: +z): the back's frame one bent tube
/// from the front feet over the top, the back legs another under the seat's front, a tube round the seat, bowed back
/// slats and seat slats with rounded ends.
fn bistro_chair(c: f64, facing: i32) {
    let fc = facing as f64;
    let at = |x: f64, y: f64, lz: f64| (x * fc, y, c + lz * fc);
    for (k, lz) in [-0.14, -0.045, 0.05, 0.145].into_iter().enumerate() {
        plank(
            &format!("chair{}_seat{k}", float_repr(c)),
            at(-0.205, 0.43, lz),
            at(0.205, 0.43, lz),
            0.02,
            M.bistro,
            &end_round(0.085, 0.41, Some(0.025), 2, true),
            (0, 1, 0),
            0.0,
        );
    }
    let lean = (0, cosd(10.0), -sind(10.0) * fc); // up the back, leaning 10°
    for (k, y) in [0.62, 0.74, 0.86].into_iter().enumerate() {
        let lz = 0.2 - 0.45 * y / 0.93 - 0.013; // just behind the frame's uprights
        let bow: Vec<(f64, f64, f64)> = [-0.205, -0.1, 0.0, 0.1, 0.205]
            .iter()
            .map(|&x| at(x, y, lz - 0.015 * (1.0 - artkit::math::pow(x / 0.205, 2.0))))
            .collect();
        let up = v(lean);
        fsweep!(
            F,
            format!("chair{}_back{k}", float_repr(c)),
            &bow,
            0,
            M.bistro,
            profile = rounded_rect(0.07, 0.018, 0.007, 1),
            up = (up.x as f64, up.y as f64, up.z as f64),
            angle = 60
        );
    }
    let x = 0.21;
    super::common::bent(
        &format!("chair{}_frame", float_repr(c)),
        &[
            at(-x, 0.0, 0.2),
            at(-x, 0.93, -0.25),
            at(x, 0.93, -0.25),
            at(x, 0.0, 0.2),
        ],
        0.012,
        M.iron,
        0.06,
        6,
        3,
        1.0,
    );
    super::common::bent(
        &format!("chair{}_legs", float_repr(c)),
        &[
            at(-x + 0.012, 0.0, -0.2),
            at(-x + 0.012, 0.405, 0.16),
            at(x - 0.012, 0.405, 0.16),
            at(x - 0.012, 0.0, -0.2),
        ],
        0.012,
        M.iron,
        0.03,
        6,
        3,
        1.0,
    );
    let ring: Vec<(f64, f64, f64)> = rounded_rect(0.42, 0.37, 0.03, 2)
        .iter()
        .map(|&(u, w)| at(u, 0.414, w))
        .collect();
    fsweep!(
        F,
        format!("chair{}_seat_ring", float_repr(c)),
        &ring,
        0.009,
        M.iron,
        verts = 6,
        closed = true
    );
    rod(
        &format!("chair{}_stretcher", float_repr(c)),
        at(-x, 0.12, 0.13),
        at(x, 0.12, 0.13),
        0.008,
        M.iron,
        6,
    );
    for s in [-1, 1] {
        // the pivots where the legs cross the frame, glides under
        let p = at(s as f64 * (x + 0.006), 0.2, 0.06);
        cylinder!(
            format!("chair{}_pivot{s}", float_repr(c)),
            0.014,
            0.012,
            at = F.at(p.0, p.1, p.2),
            material = M.iron,
            rot = (0, 90, 0),
            verts = 8
        );
    }
}

// The two chairs face each other across the table, their seats' fronts 4 cm under its edges (content `places`).
const BISTRO_CHAIRS: (f64, f64) = (-0.45, 0.45);

pub fn patio_set() -> Vec<Obj> {
    for (k, z) in [-0.24, -0.12, 0.0, 0.12, 0.24].into_iter().enumerate() {
        plank(
            &format!("top{k}"),
            (-0.3, 0.71, z),
            (0.3, 0.71, z),
            0.02,
            M.bistro,
            &end_round(0.11, 0.6, Some(0.02), 2, true),
            (0, 1, 0),
            0.004,
        );
    }
    // the frame under the top, and the folding legs: a U each, crossing at the table's sides, clear of the
    // sitters' knees between them
    let ring: Vec<(f64, f64, f64)> = rounded_rect(0.56, 0.56, 0.04, 2)
        .iter()
        .map(|&(u, w)| (u, 0.69, w))
        .collect();
    bar(
        "top_frame",
        &ring,
        0.03,
        0.012,
        M.iron,
        Bar {
            bend: 0.0,
            r: Some(0.003),
            up: P3::from((0, 0, 1)),
            closed: true,
            ..Bar::default()
        },
    );
    for e in [-1, 1] {
        let ef = e as f64;
        let x = 0.27 - (if e > 0 { 0.014 } else { 0.0 });
        super::common::bent(
            &format!("legs{e}"),
            &[
                (-x, 0.0, ef * 0.23),
                (-x, 0.678, -ef * 0.23),
                (x, 0.678, -ef * 0.23),
                (x, 0.0, ef * 0.23),
            ],
            0.0125,
            M.iron,
            0.03,
            6,
            3,
            1.0,
        );
    }
    for s in [-1, 1] {
        cylinder!(
            format!("hinge{s}"),
            0.016,
            0.04,
            at = F.at(s as f64 * 0.263, 0.339, 0),
            material = M.iron,
            rot = (0, 90, 0),
            verts = 8
        );
    }
    cup("cup", 0.12, 0.72, 0.06, 0.038, 0.085, M.cup, true, 0.0, 12);
    jug("jug", -0.14, 0.72, -0.08, 0.05, 0.16, M.jug);
    bistro_chair(BISTRO_CHAIRS.0, 1);
    bistro_chair(BISTRO_CHAIRS.1, -1);
    finish("patioSet", false)
}

// --- the picnic table: a planked top on A-frames, a bench each side; a gingham cloth and a basket --------------
const PICNIC_TOP: f64 = 0.73; // the top's and the benches' tops: content `surface`, `seats.picnicTable`
const PICNIC_SEAT: f64 = 0.44;

/// A wicker picnic basket standing at game (x, y, z): a tapering body, a rim, two lid flaps, a bent-wood handle
/// and a strap.
fn basket(x: f64, y: f64, z: f64, w: f64, h: f64, d: f64) {
    let b = F.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let rings = [(0.0, 0.9), (0.02, 0.94), (0.7, 1.0), (0.78, 1.0)];
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(t, k)| {
            ring!(
                w / 2.0 * k,
                d / 2.0 * k,
                z = bz + h * t,
                verts = 16,
                square = 7,
                at = (bx, by)
            )
        })
        .collect();
    loft!("basket", &sections, material = M.wicker);
    let rim: Vec<(f64, f64, f64)> = rounded_rect(w + 0.012, d + 0.012, 0.03, 2)
        .iter()
        .map(|&(u, v)| (x + u, y + h * 0.78, z + v))
        .collect();
    fsweep!(
        F,
        "basket_rim",
        &rim,
        0.009,
        M.wicker_rim,
        verts = 5,
        closed = true
    );
    for s in [-1, 1] {
        // the two lid flaps, hinged along the middle
        soft!(
            F,
            format!("basket_lid{s}"),
            w + 0.004,
            0.03,
            d / 2.0 - 0.004,
            (x, y + h * 0.78 + 0.02, z + s as f64 * d / 4.0),
            M.wicker_rim,
            rot = (s * 3, 0, 0),
            round = 0.012,
            round_segments = 1,
            flat = 1
        );
    }
    let arc: Vec<(f64, f64, f64)> = (0..181)
        .step_by(20)
        .map(|a| {
            (
                x + 0.5 * w * cos(radians(a as f64)) * 0.82,
                y + h * 0.8 + 0.156 * sin(radians(a as f64)),
                z,
            )
        })
        .collect();
    fsweep!(
        F,
        "basket_handle",
        &arc,
        0,
        M.handle,
        profile = rounded_rect(0.012, 0.022, 0.005, 1),
        up = (0, 1, 0)
    );
    fbox!(
        F,
        "basket_strap",
        0.03,
        h * 0.45,
        0.006,
        (x, y + h * 0.62, z + d / 2.0 + 0.005),
        M.handle,
        bevel = 0.002,
        bevel_segments = 1
    );
}

pub fn picnic_table() -> Vec<Obj> {
    let (top, seat) = (PICNIC_TOP, PICNIC_SEAT);
    for (k, z) in [-0.29, -0.145, 0.0, 0.145, 0.29].into_iter().enumerate() {
        fbox!(
            F,
            format!("top{k}"),
            2.5,
            0.04,
            0.135,
            (0, top - 0.02, z),
            M.pine,
            bevel = 0.01
        );
    }
    for s in [-1, 1] {
        for (k, z) in [0.6, 0.745].into_iter().enumerate() {
            fbox!(
                F,
                format!("seat{s}{k}"),
                2.5,
                0.04,
                0.13,
                (0, seat - 0.02, s as f64 * z),
                M.pine,
                bevel = 0.01
            );
        }
    }
    // the cleats under the top and the bearers under the seats, their ends cut back underneath
    for x in [-0.95, 0.0, 0.95] {
        let yc = top - 0.065;
        side_slab(
            &format!("cleat{}", float_repr(x)),
            x,
            &[
                (-0.36, yc + 0.025),
                (0.36, yc + 0.025),
                (0.32, yc - 0.025),
                (-0.32, yc - 0.025),
            ],
            0.07,
            M.pine_dark,
            0.006,
        );
    }
    let hw = 0.035 / cosd(38.0);
    for s in [-1, 1] {
        let sf = s as f64;
        let x = sf * 0.95;
        let yb = seat - 0.07;
        side_slab(
            &format!("bearer{s}"),
            x,
            &[
                (-0.81, yb + 0.03),
                (0.81, yb + 0.03),
                (0.77, yb - 0.03),
                (-0.77, yb - 0.03),
            ],
            0.07,
            M.pine_dark,
            0.006,
        );
        // the A-frame's legs, cut level at the foot and under the cleat, either side of the cleat and bearer
        for e in [-1, 1] {
            let (zf, zt) = (e as f64 * 0.66, e as f64 * 0.12);
            side_slab(
                &format!("leg{s}{e}"),
                x + sf * 0.06,
                &[
                    (zf - hw, 0.0),
                    (zf + hw, 0.0),
                    (zt + hw, top - 0.04),
                    (zt - hw, top - 0.04),
                ],
                0.05,
                M.pine_dark,
                0.006,
            );
            // carriage bolts through leg and bearer, leg and cleat
            for y in [yb, top - 0.065] {
                let zl = zf + (zt - zf) * y / (top - 0.04);
                cylinder!(
                    format!("bolt{s}{e}{}", float_repr(y)),
                    0.011,
                    0.006,
                    at = F.at(x + sf * 0.087, y, zl),
                    material = M.grate,
                    rot = (0, 90, 0),
                    verts = 6
                );
            }
        }
        board(
            &format!("brace{s}"),
            (sf * 0.92, seat - 0.07, 0),
            (sf * 0.04, top - 0.07, 0),
            0.05,
            0.05,
            M.pine_dark,
            (0, 1, 0),
            0.006,
        );
    }
    // the cloth: red gingham laid across the table where the two sit, hanging over both long edges
    let (e, hang, ct) = (0.362, 0.07, 0.004);
    let drape = kit::fillet!(
        &[
            (-0.3, top - hang, -e),
            (-0.3, top + ct / 2.0, -e),
            (-0.3, top + ct / 2.0, e),
            (-0.3, top - hang, e),
        ],
        0.012,
        steps = 2
    );
    fsweep!(
        F,
        "cloth",
        &drape,
        0,
        M.cloth,
        profile = rounded_rect(0.7, ct, 0.0015, 1),
        up = (1, 0, 0)
    );
    let (cw, cd) = (0.7 / 8.0, 2.0 * e / 8.0);
    for i in 0..8 {
        for j in 0..8 {
            if (i + j) % 2 == 0 {
                fbox!(
                    F,
                    format!("check{i}{j}"),
                    cw,
                    0.0012,
                    cd,
                    (
                        -0.65 + cw * (i as f64 + 0.5),
                        top + ct + 0.0006,
                        -e + cd * (j as f64 + 0.5)
                    ),
                    M.cloth_check
                );
            }
        }
    }
    for s in [-1, 1] {
        // the checks down the hanging edges
        for i in 0..8 {
            if (i + (if s < 0 { 0 } else { 1 })) % 2 == 0 {
                fbox!(
                    F,
                    format!("check_hang{s}{i}"),
                    cw,
                    hang - 0.02,
                    0.0012,
                    (
                        -0.65 + cw * (i as f64 + 0.5),
                        top - hang / 2.0 - 0.006,
                        s as f64 * (e + ct / 2.0 + 0.0006)
                    ),
                    M.cloth_check
                );
            }
        }
    }
    basket(0.55, top, 0.02, 0.4, 0.2, 0.28);
    cup(
        "cup0",
        -0.45,
        top + ct,
        0.12,
        0.038,
        0.09,
        M.cup,
        true,
        200.0,
        12,
    );
    cup(
        "cup1",
        -0.15,
        top + ct,
        -0.1,
        0.035,
        0.1,
        M.jug,
        false,
        0.0,
        12,
    );
    finish("picnicTable", false)
}

// --- the garden chair: a white slatted armchair, the back leaning, a cup on the arm ----------------------------

/// Back slats up a back leaning `tilt` degrees, from `bottom` (height, depth); `fan` spreads their tops; their tops
/// rounded by `r` (half their width: a full round). Returns each slat's (bottom, top) game points.
#[allow(clippy::too_many_arguments, clippy::type_complexity)] // (as the script called it)
fn back_slats(
    prefix: &str,
    xs: &[f64],
    lengths: &[f64],
    material: Mat,
    fan: f64,
    bottom: (f64, f64),
    tilt: f64,
    w: f64,
    t: f64,
    r: f64,
) -> Vec<((f64, f64, f64), (f64, f64, f64))> {
    let up = (0, cosd(tilt), -sind(tilt));
    let face = (0, sind(tilt), cosd(tilt));
    let (y0, z0) = bottom;
    let mut ends = Vec::new();
    for (k, (&x, &length)) in xs.iter().zip(lengths).enumerate() {
        let top = (x * fan, y0 + length * up.1, z0 + length * up.2);
        plank(
            &format!("{prefix}{k}"),
            (x, y0, z0),
            top,
            t,
            material,
            &end_round(w, length, Some(r), 3, false),
            face,
            0.005,
        );
        ends.push(((x, y0, z0), top));
    }
    ends
}

/// A side board from `front` (height, depth) down to `back`, `w` deep, `t` thick across: its front end cut in a
/// round, its back cut level on the floor.
fn stringer(
    name: &str,
    x: f64,
    front: (f64, f64),
    back: (f64, f64),
    w: f64,
    t: f64,
    material: Mat,
) -> Obj {
    let ((y1, z1), (y0, z0)) = (front, back);
    let d = artkit::math::v3(z1 - z0, y1 - y0, 0);
    let n = artkit::math::v3(-d.y, d.x, 0).normalized() * (w / 2.0);
    let u = d.normalized();
    let mut out = vec![
        (z0 - n.x as f64, y0 - n.y as f64),
        (z0 + n.x as f64, y0 + n.y as f64),
    ];
    for i in 0..7 {
        // the round front end
        let a = PI / 2.0 - PI * i as f64 / 6.0;
        let c = artkit::math::v3(z1, y1, 0) + u * (w / 2.0) * cos(a) * 0.9 + n * sin(a);
        out.push((c.x as f64, c.y as f64));
    }
    side_slab(name, x, &out, t, material, 0.005)
}

/// A curved bracket under an arm at game (x, y = its top, z = its front), `h` tall and `d` deep, `t` thick.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn corbel(name: &str, x: f64, y: f64, z: f64, h: f64, d: f64, t: f64, material: Mat) -> Obj {
    let curve: Vec<(f64, f64)> = (0..6)
        .map(|i| {
            (
                z - d + d * (1.0 - cos(PI / 2.0 * i as f64 / 5.0)),
                y - h * sin(PI / 2.0 * i as f64 / 5.0),
            )
        })
        .collect();
    let mut out = vec![(z - d, y), (z, y), (z, y - h)];
    let rev: Vec<(f64, f64)> = curve.iter().rev().copied().collect();
    out.extend_from_slice(&rev[1..rev.len() - 1]);
    side_slab(name, x, &out, t, material, 0.004)
}

/// A wide flat arm from depth z0 (the back) to z1, `back_w` wide at the back widening to `front_w`, its front end
/// rounded.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn arm_board(
    name: &str,
    x: f64,
    y: f64,
    z0: f64,
    z1: f64,
    back_w: f64,
    front_w: f64,
    material: Mat,
    t: f64,
) -> Obj {
    let (l, r) = (z1 - z0, front_w * 0.35);
    let mut out = vec![
        (-back_w / 2.0, 0.0),
        (back_w / 2.0, 0.0),
        (front_w / 2.0, l * 0.55),
    ];
    out.extend((0..4).map(|i| {
        (
            (front_w / 2.0 - r) + r * cos(PI / 2.0 * i as f64 / 3.0),
            l - r + r * sin(PI / 2.0 * i as f64 / 3.0),
        )
    }));
    let mirrored: Vec<(f64, f64)> = out[3..].iter().rev().map(|&(u, v)| (-u, v)).collect();
    out.extend(mirrored);
    out.push((-front_w / 2.0, l * 0.55));
    plank(
        name,
        (x, y, z0),
        (x, y, z1),
        t,
        material,
        &out,
        (0, 1, 0),
        0.006,
    )
}

/// The garden and Adirondack chairs' shared frame: a seat sloping back (its front slat rounded over), wide shaped
/// arms on front legs with curved brackets, back legs and stringers rounded at the front; the back slats are the
/// caller's.
fn slatted_chair(prefix: &str, material: Mat, fan: f64) {
    for (k, z) in [-0.12, 0.0, 0.12, 0.24].into_iter().enumerate() {
        fbox!(
            F,
            format!("{prefix}seat{k}"),
            0.56,
            0.025,
            0.11,
            (0, 0.4 - z * 0.15, z),
            material,
            rot = (-8.5, 0, 0),
            bevel = if k == 3 { 0.012 } else { 0.008 },
            bevel_segments = 2
        );
    }
    board(
        &format!("{prefix}back_rail"),
        (-0.3, 0.56, -0.25),
        (0.3, 0.56, -0.25),
        0.06,
        0.03,
        material,
        (0, sind(22.0), cosd(22.0)),
        0.006,
    );
    let wide = fan > 1.0;
    for s in [-1, 1] {
        let sf = s as f64;
        let ax = sf * (if wide { 0.37 } else { 0.36 });
        arm_board(
            &format!("{prefix}arm{s}"),
            ax,
            0.62,
            if wide { -0.37 } else { -0.35 },
            if wide { 0.33 } else { 0.31 },
            0.09,
            if wide { 0.14 } else { 0.12 },
            material,
            0.025,
        );
        fbox!(
            F,
            format!("{prefix}leg_front{s}"),
            0.06,
            0.61,
            0.06,
            (sf * 0.36, 0.305, 0.26),
            material,
            bevel = 0.01
        );
        fbox!(
            F,
            format!("{prefix}leg_back{s}"),
            0.06,
            0.42,
            0.06,
            (sf * 0.32, 0.21, -0.24),
            material,
            bevel = 0.01
        );
        stringer(
            &format!("{prefix}stringer{s}"),
            sf * 0.3,
            (0.425, 0.268),
            (0.32, -0.28),
            0.08,
            0.05,
            material,
        );
        corbel(
            &format!("{prefix}arm_brace{s}"),
            sf * 0.36,
            0.608,
            0.22,
            0.13,
            0.09,
            0.04,
            material,
        );
    }
}

pub fn garden_chair() -> Vec<Obj> {
    slatted_chair("", M.garden_white, 1.0);
    let xs = [-0.24, -0.12, 0.0, 0.12, 0.24];
    back_slats(
        "back",
        &xs,
        &[0.74, 0.79, 0.81, 0.79, 0.74],
        M.garden_white,
        1.0,
        (0.29, -0.123),
        22.0,
        0.1,
        0.022,
        0.012,
    );
    // the top rail: one board, its top edge arched, across the slats' fronts, leaning with them
    let mut arch: Vec<(f64, f64)> = [-0.3, 0.3].iter().map(|&u| (u, -0.03)).collect();
    arch.extend((0..9).map(|i| {
        (
            0.3 - 0.6 * i as f64 / 8.0,
            0.018 + 0.016 * sin(PI * i as f64 / 8.0),
        )
    }));
    fextrude!(
        F,
        "top_rail",
        &arch,
        0.025,
        (0, 0.95, -0.366),
        M.garden_white,
        bevel = 0.006,
        bevel_segments = 1,
        rot = (-22, 0, 0)
    );
    cup("cup", 0.36, 0.6325, 0.18, 0.045, 0.1, M.cup, true, 60.0, 12);
    finish("gardenChair", true)
}

pub fn adirondack_chair() -> Vec<Obj> {
    slatted_chair("", M.adirondack, 1.6);
    let xs = [-0.18, -0.09, 0.0, 0.09, 0.18];
    let ends = back_slats(
        "back",
        &xs,
        &[0.78, 0.84, 0.87, 0.84, 0.78],
        M.adirondack,
        1.6,
        (0.29, -0.123),
        22.0,
        0.085,
        0.022,
        0.042,
    );
    // the upper back rail, bowed, behind the slats half way up
    let face = V3::from((0, sind(22.0), cosd(22.0)));
    let mid: Vec<(f64, f64, f64)> = ends
        .iter()
        .map(|&(b, t)| {
            let p = V3::from(b) + (V3::from(t) - V3::from(b)) * 0.52 - face * 0.022;
            (p.x as f64, p.y as f64, p.z as f64)
        })
        .collect();
    let mut rail = vec![(-0.27, mid[0].1, mid[0].2 + 0.012)];
    rail.extend_from_slice(&mid);
    rail.push((0.27, mid[mid.len() - 1].1, mid[mid.len() - 1].2 + 0.012));
    let up = v((0, cosd(22.0), -sind(22.0)));
    fsweep!(
        F,
        "back_rail_upper",
        &rail,
        0,
        M.adirondack,
        profile = rounded_rect(0.06, 0.022, 0.006, 1),
        up = (up.x as f64, up.y as f64, up.z as f64),
        resolution = 2,
        angle = 60
    );
    finish("adirondackChair", true)
}

// --- the garden bench: three seat slats and three back slats on three iron frames ------------------------------
// 1.9 m on its two tiles: the content's three sitters, spread across them, 0.67 m apart.
const GARDEN_BENCH_END: f64 = 0.92; // its end frames, either side

pub fn bench_garden3() -> Vec<Obj> {
    for (k, z) in [-0.08, 0.04, 0.16].into_iter().enumerate() {
        fbox!(
            F,
            format!("seat{k}"),
            1.9,
            0.03,
            0.1,
            (0, 0.45, z),
            M.bench_slats,
            bevel = 0.01
        );
    }
    let backs: Vec<(f64, f64)> = [0.6, 0.72, 0.84]
        .iter()
        .map(|&y| (y, -0.2 - (y - 0.6) * 0.15))
        .collect();
    for (k, &(y, z)) in backs.iter().enumerate() {
        fbox!(
            F,
            format!("back{k}"),
            1.9,
            0.09,
            0.025,
            (0, y, z),
            M.bench_slats,
            rot = (-12, 0, 0),
            bevel = 0.01
        );
    }
    // the support rises in line with the back slats, just in front of them, and curls back over the top
    let back = [
        (0.0, -0.205),
        (0.08, -0.17),
        (0.43, -0.16),
        (0.8, -0.214),
        (0.87, -0.234),
        (0.886, -0.28),
        (0.868, -0.313),
    ];
    for x in [-GARDEN_BENCH_END, 0.0, GARDEN_BENCH_END] {
        let end = x.abs() > 0.0;
        let front: &[(f64, f64)] = if end {
            &[
                (0.0, 0.2),
                (0.44, 0.205),
                (0.62, 0.235),
                (0.664, 0.26),
                (0.664, 0.2),
                (0.664, -0.21),
            ]
        } else {
            &[(0.0, 0.19), (0.435, 0.2)]
        };
        cast_end(
            &format!("frame{}", float_repr(x)),
            x,
            &back,
            front,
            (0.42, -0.2, 0.215),
            (0.1, -0.19, 0.196),
            if end {
                Some((0.678, -0.215, 0.28))
            } else {
                None
            },
            if end { Some((0.555, 0.15, 0.05)) } else { None },
        );
        let seat: Vec<(f64, f64, f64)> =
            [-0.08, 0.04, 0.16].iter().map(|&z| (x, 0.467, z)).collect();
        bolts(&format!("bolt_seat{}", float_repr(x)), &seat, M.iron, true);
        let bb: Vec<(f64, f64, f64)> = backs.iter().map(|&(y, z)| (x, y, z + 0.014)).collect();
        bolts(&format!("bolt_back{}", float_repr(x)), &bb, M.iron, false);
    }
    finish("benchGarden3", false)
}
