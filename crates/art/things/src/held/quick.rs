//! The quick foods, each its own (review M-held-01: they borrowed the plates and the prep bowl), and the dishes
//! emptied, each the dish that went in (review M-held-02).

use super::M;
use super::common::{_object, _spoon, F, WELL, done, loft, plate0, rrect};
use super::dishes::{_paddle, _patty, _pie_dish};
use super::plates::_halves;
use artkit::forms::rounded_rect;
use artkit::kit::{self, fillet};
use artkit::materials::Mat;
use artkit::math::{P3, TAU, V3, cos, radians, sin};
use artkit::mesh::{Mesh, normal_of};
use artkit::place::Facing;
use artkit::py;
use artkit::scene::Obj;
use artkit::{cylinder, fextrude, fsweep, jitter, sweep};

/// A glazed dessert bowl 12.5 cm across and 5 cm deep, its rim rolled, its inside floor at 1.2 cm.
fn _dessert_bowl() {
    let mut mats = vec![M.bowl; 6];
    mats.push(M.china_well);
    lathe!(
        "bowl",
        &[
            (0.0, 0.0),
            (0.035, 0.0),
            (0.04, 0.006),
            (0.0612, 0.047),
            (0.0615, 0.051),
            (0.0565, 0.051),
            (0.053, 0.044),
            (0.0, 0.012)
        ],
        mats,
        segments = 20
    );
}

/// A dessert bowl of ice cream from the freezer: three scoops (vanilla, strawberry, chocolate), each with the
/// ragged lip a scoop leaves, chocolate sauce trickled over the vanilla and the strawberry, and a fan wafer. (No spoon:
/// the one in the hand eats it, actions.ts `EATEN_WITH`.)
pub fn ice_cream() -> Vec<Obj> {
    _dessert_bowl();
    for (i, (x, z, m, s)) in [
        (-0.018, 0.012, M.vanilla, 1.0),
        (0.02, 0.008, M.strawberry, 0.95),
        (0.0, -0.022, M.choc_ice, 0.9),
    ]
    .into_iter()
    .enumerate()
    {
        let sc = lathe!(
            format!("scoop{i}"),
            &[
                (0.0, 0.0),
                (0.027 * s, 0.0),
                (0.031 * s, 0.006 * s),
                (0.03 * s, 0.022 * s),
                (0.02 * s, 0.039 * s),
                (0.0, 0.044 * s)
            ],
            m,
            at = (x, 0.026, z),
            segments = 10,
            wave = (5, vec![(2, 0.12)]),
            rot = (0.0, 0.0, (25 * i) as f64)
        );
        jitter!(sc, 0.0012, size = 0.01, seed = 1.3 + i as f64);
        if m != M.choc_ice {
            _sauce(&format!("sauce{i}"), (x, 0.026, z), s, i as f64);
        }
    }
    box_!(
        "wafer",
        0.036,
        0.004,
        0.026,
        (-0.004, 0.077, -0.004),
        M.wafer,
        yaw = 30.0,
        tilt = (55.0, 10.0),
        bevel = 0.0008
    );
    done("iceCream")
}

/// Chocolate sauce trickled over a scoop `s` its size whose base is at the game point `at`: a line zigzagging over
/// its top and running down its side in a drip, lying on it (over its jitter).
fn _sauce(name: &str, at: (f64, f64, f64), s: f64, seed: f64) {
    // the scoop's height (its profile, `ice_cream`) at (`u`, `w`) from its middle, and 2.5 mm over it
    let surface = |u: f64, w: f64| {
        let r = py::hypot(u, w) / s;
        let h = if r < 0.02 {
            0.044 - 0.005 * r / 0.02
        } else {
            0.039 - 0.017 * (r - 0.02) / 0.01
        };
        at.1 + h * s + 0.0025
    };
    let (c, d) = (cos(seed * 1.9), sin(seed * 1.9));
    let path = [
        (-0.022, -0.006),
        (-0.012, 0.012),
        (-0.004, -0.014),
        (0.006, 0.013),
        (0.014, -0.01),
        (0.022, 0.006),
        (0.027, 0.008),
        (0.031, 0.009),
    ];
    let mut pts = Vec::new();
    for (u, w) in path {
        let (u, w) = ((u * c - w * d) * s, (u * d + w * c) * s);
        pts.push((at.0 + u, surface(u, w), at.2 + w));
    }
    kit::tube!(F, name, &pts, 0.0024, M.chocolate, verts = 4, round = 1);
}

/// The dessert bowl eaten from: a film of melted ice cream lying on its sloping inside, a streak of chocolate, a
/// drip of strawberry run down from the rim, and the spoon left in it.
pub fn bowl_dirty() -> Vec<Obj> {
    _dessert_bowl();
    _film(
        "melt",
        (0.004, -0.002),
        0.036,
        0.03,
        M.melt,
        0.5,
        15.0,
        0.0012,
        10,
    );
    _film(
        "melt_choc",
        (-0.016, -0.014),
        0.016,
        0.009,
        M.choc_ice,
        1.5,
        -30.0,
        0.0016,
        10,
    );
    _film(
        "drip",
        (0.034, 0.02),
        0.016,
        0.006,
        M.strawberry,
        2.5,
        -30.0,
        0.0016,
        10,
    );
    _spoon((0.012, 0.024, 0.012), 30.0, 33.0, "spoon", 0.11);
    done("bowlDirty")
}

/// A film of something melted on the dessert bowl's inside (`_dessert_bowl`), about `rx` by `rz` round the game
/// (x, depth) `at`, lying `lift` over the glaze: its floor's middle at 1.2 cm, the sides rising 0.6 m a metre out.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn _film(
    name: &str,
    at: (f64, f64),
    rx: f64,
    rz: f64,
    material: Mat,
    seed: f64,
    yaw: f64,
    lift: f64,
    n: usize,
) -> Obj {
    let a0 = radians(yaw);
    let ring: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let a = TAU * i as f64 / n as f64;
            let k = 1.0 + 0.16 * sin(3.0 * a + seed) + 0.09 * sin(5.0 * a + 2.3 * seed);
            let (u, w) = (rx * k * cos(a), rz * k * sin(a));
            (
                at.0 + u * cos(a0) - w * sin(a0),
                at.1 + u * sin(a0) + w * cos(a0),
            )
        })
        .collect();
    let vert = |x: f64, z: f64| V3::from(F.at(x, 0.012 + 0.6038 * py::hypot(x, z) + lift, z));
    let mut verts = vec![vert(at.0, at.1)];
    verts.extend(
        ring.iter()
            .map(|&(x, z)| vert((at.0 + x) / 2.0, (at.1 + z) / 2.0)),
    );
    verts.extend(ring.iter().map(|&(x, z)| vert(x, z)));
    let (mid, half, rim) = (0u32, |i: usize| 1 + i as u32, |i: usize| (1 + n + i) as u32);
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for i in 0..n {
        let j = (i + 1) % n;
        faces.push(vec![mid, half(i), half(j)]);
        faces.push(vec![half(i), rim(i), rim(j), half(j)]);
    }
    for f in &mut faces {
        let pts: Vec<V3> = f.iter().map(|&v| verts[v as usize]).collect();
        if normal_of(&pts).z < 0.0 {
            f[1..].reverse();
        }
    }
    let mut mesh = Mesh::from_pydata(verts, &faces);
    mesh.slot(material);
    for f in &mut mesh.faces {
        (f.mat, f.smooth) = (0, true);
    }
    _object(name, mesh, P3::ZERO, P3::ZERO)
}

/// A plate of toast: a slice cut corner to corner, its loaf corner rounded, toasted golden with darker crusts
/// and a scorch or two, a pat of butter melting into a pool on each half.
pub fn toast() -> Vec<Obj> {
    plate0();
    let (nx, nz) = (0.05, 0.055);
    let n = py::hypot(nx, nz);
    for (half, pts, cut, sign) in _halves((0.055, 0.05), 0.012) {
        let s = sign as f64;
        _toast_half(
            half,
            &pts,
            cut,
            (s * nx / n * 0.01, s * nz / n * 0.01),
            WELL,
            sign,
        );
    }
    done("toast")
}

/// Half a slice of toast over the outline `pts` (its side `cut` the cut), moved `off`, from height `y`: toasted on
/// top, dark crusts, a scorch, a pat of butter melting into a pool.
pub fn _toast_half(half: &str, pts: &[(f64, f64)], cut: usize, off: (f64, f64), y: f64, sign: i32) {
    let sides: Vec<Mat> = (0..pts.len())
        .map(|k| if k == cut { M.bread } else { M.toast_crust })
        .collect();
    prism!(
        format!("slice_{half}"),
        pts,
        y,
        y + 0.013,
        M.toast,
        M.toast_crust,
        sides = sides,
        at = off
    );
    let cx = artkit::py::sum(pts.iter().map(|p| p.0)) / pts.len() as f64 + off.0;
    let cz = artkit::py::sum(pts.iter().map(|p| p.1)) / pts.len() as f64 + off.1;
    let s = sign as f64;
    blob!(
        format!("scorch_{half}"),
        (cx - 0.015 * s, y + 0.013, cz + 0.012 * s),
        0.01,
        0.006,
        0.0004,
        M.toast_crust,
        seed = s,
        n = 6
    );
    blob!(
        format!("pool_{half}"),
        (cx, y + 0.013, cz),
        0.015,
        0.012,
        0.0006,
        M.butter,
        seed = (sign * 2) as f64,
        n = 7
    );
    box_!(
        format!("butter_{half}"),
        0.018,
        0.005,
        0.016,
        (cx, y + 0.0154, cz),
        M.butter,
        yaw = if sign > 0 { 20.0 } else { -10.0 },
        bevel = 0.0018
    );
}

/// A tall tumbler of berry smoothie, 8 cm across and 14 cm high (about 0.4 L): a thick glass foot, the
/// smoothie seen through the glass up to a frothy top, a striped bendy straw, a raspberry and two mint leaves.
pub fn smoothie() -> Vec<Obj> {
    lathe!(
        "glass",
        &[
            (0.0, 0.0),
            (0.03, 0.0),
            (0.031, 0.004),
            (0.0315, 0.013),
            (0.039, 0.12),
            (0.0398, 0.134),
            (0.04, 0.1395),
            (0.0378, 0.1402),
            (0.0366, 0.126),
            (0.03, 0.1265),
            (0.0, 0.129)
        ],
        [
            M.glass,
            M.glass,
            M.glass,
            M.smoothie,
            M.glass,
            M.glass,
            M.glass,
            M.glass,
            M.smoothie_top,
            M.smoothie_top
        ],
        segments = 20
    );
    let path = fillet(
        &[
            (0.006, 0.1, 0.004),
            (0.011, 0.172, 0.007),
            (0.031, 0.19, 0.016),
        ],
        0.012,
        3,
    );
    kit::tube!(F, "straw", &path, 0.0035, M.straw, verts = 6, round = 1);
    for (i, y) in [0.142, 0.154, 0.166].into_iter().enumerate() {
        let k = (y - 0.1) / 0.072;
        kit::tube!(
            F,
            format!("stripe{i}"),
            &[
                (0.006 + 0.005 * k, y, 0.004 + 0.003 * k),
                (
                    0.006 + 0.005 * (k + 0.06),
                    y + 0.0045,
                    0.004 + 0.003 * (k + 0.06)
                )
            ],
            0.0037,
            M.straw_stripe,
            verts = 6,
            round = 1
        );
    }
    let b = ball!(
        "raspberry",
        0.0085,
        (-0.015, 0.1335, -0.008),
        M.berry,
        seg = 7,
        rings = 5
    );
    jitter!(b, 0.0009, size = 0.003, seed = 2.0);
    for (i, (x, z, yaw)) in [(-0.004, -0.012, 30.0), (-0.022, 0.004, 110.0)]
        .into_iter()
        .enumerate()
    {
        leaf!(
            format!("mint{i}"),
            (x, 0.1305, z),
            0.018,
            0.01,
            yaw,
            M.leaf,
            tilt = 12.0
        );
    }
    done("smoothie")
}

/// A takeaway noodle pail of white card with a red band: its four flaps standing open, a wire handle, noodles
/// and spring onion heaped in its top. (No chopsticks: the pair in the hand eats them, actions.ts `EATEN_WITH`.)
pub fn noodles() -> Vec<Obj> {
    let ring = |y: f64, hw: f64| -> Vec<(f64, f64, f64)> {
        rrect(hw, hw, 16, 9.0)
            .into_iter()
            .map(|(x, z)| (x, y, z))
            .collect()
    };
    let side = |y: f64| 0.035 + 0.015 * y / 0.1;
    loft(
        "pail",
        &[ring(0.0, side(0.0)), ring(0.05, side(0.05))],
        M.paper,
        40.0,
    );
    loft(
        "band",
        &[
            ring(0.05, side(0.05) + 0.0004),
            ring(0.068, side(0.068) + 0.0004),
        ],
        M.paper_red,
        40.0,
    );
    loft(
        "pail_top",
        &[
            ring(0.068, side(0.068)),
            ring(0.1, side(0.1)),
            ring(0.1, side(0.1) - 0.0025),
            ring(0.088, side(0.088) - 0.0025),
        ],
        M.paper,
        30.0,
    );
    // the flaps: a trapezoid of card standing up from the front and back edges, leaning out a little (the side
    // ones are tucked in, where the wire handle hooks in)
    let tab = [(-0.038, 0.0), (0.038, 0.0), (0.022, 0.026), (-0.022, 0.026)];
    for (i, yaw) in [0, 180].into_iter().enumerate() {
        let z = if yaw == 0 { 0.0488 } else { -0.0488 };
        fextrude!(
            F,
            format!("flap{i}"),
            &tab,
            0.0012,
            (0, 0.0995, z),
            M.paper,
            plane = Facing::Front,
            rot = (2, 0, yaw)
        );
    }
    let mound = lathe!(
        "noodles",
        &[
            (0.0, 0.0),
            (0.044, 0.0),
            (0.035, 0.008),
            (0.018, 0.014),
            (0.0, 0.016)
        ],
        M.noodle,
        at = (0, 0.086, 0),
        segments = 10
    );
    jitter!(mound, 0.0025, size = 0.01, seed = 0.8);
    for (i, pts) in [
        [
            (-0.03, 0.093, 0.0),
            (-0.018, 0.1, 0.012),
            (0.0, 0.097, 0.02),
            (0.02, 0.102, 0.008),
        ],
        [
            (0.03, 0.094, -0.02),
            (0.01, 0.101, -0.012),
            (-0.008, 0.098, -0.026),
            (-0.028, 0.096, -0.018),
        ],
    ]
    .into_iter()
    .enumerate()
    {
        kit::tube!(
            F,
            format!("noodle{i}"),
            &pts,
            0.0026,
            M.noodle,
            verts = 4,
            round = 2
        );
    }
    for (i, (x, z)) in [(-0.01, -0.02), (0.02, 0.0), (-0.025, 0.02)]
        .into_iter()
        .enumerate()
    {
        cylinder!(
            format!("onion{i}"),
            0.0045,
            0.0025,
            at = F.at(x, 0.103, z),
            material = M.lettuce,
            verts = 6,
            rot = (20, 10 * i as i32, 0)
        );
    }
    // the wire handle, from side to side over the top, hooked into the card below the band
    let pts: Vec<(f64, f64, f64)> = (0..=180)
        .step_by(15)
        .map(|a| {
            let a = radians(a as f64);
            (-0.0488 * cos(a), 0.088 + 0.07 * sin(a), 0.0)
        })
        .collect();
    kit::tube!(F, "wire", &pts, 0.0011, M.wire, verts = 4, round = 1);
    done("noodles")
}

/// Game points of the takeaway box's lid as if shut (x, height above the rim, depth from the hinge forward),
/// turned open about the hinge along the box's back edge by `angle` degrees.
fn _hinged(points: &[(f64, f64, f64)], angle: f64, hinge: (f64, f64)) -> Vec<(f64, f64, f64)> {
    let (c, s) = (cos(radians(angle)), sin(radians(angle)));
    points
        .iter()
        .map(|&(x, h, d)| (x, hinge.0 + h * c + d * s, hinge.1 - h * s + d * c))
        .collect()
}

/// An open kraft takeaway box, its sides flared, its lid standing open at the back: a burger (sesame bun,
/// patty, cheese, lettuce, tomato) and a heap of fries.
pub fn takeout() -> Vec<Obj> {
    let ring = |y: f64, hw: f64, hd: f64| -> Vec<(f64, f64, f64)> {
        rrect(hw, hd, 12, 8.0)
            .into_iter()
            .map(|(x, z)| (x, y, z))
            .collect()
    };
    loft(
        "tray",
        &[
            ring(0.0, 0.085, 0.06),
            ring(0.045, 0.095, 0.07),
            ring(0.045, 0.091, 0.066),
            ring(0.004, 0.081, 0.056),
        ],
        M.kraft,
        30.0,
    );
    // the lid, shallow, its inside to the front
    let mut lid = Vec::new();
    for (h, hw, hd) in [
        (0.0, 0.095, 0.07),
        (0.005, 0.089, 0.064),
        (0.0035, 0.085, 0.06),
        (0.0, 0.091, 0.066),
    ] {
        let pts: Vec<(f64, f64, f64)> = rrect(hw, hd, 12, 8.0)
            .into_iter()
            .map(|(x, z)| (x, h, 0.07 + z))
            .collect();
        lid.push(_hinged(&pts, 110.0, (0.045, -0.07)));
    }
    loft("lid", &lid, M.kraft_fold, 30.0);
    let y = 0.0045;
    _burger(-0.04, y, 0.005);
    // the fries, heaped beside it
    for i in 0..8 {
        let a = (37 * i) as f64;
        box_!(
            format!("fry{i}"),
            0.056 - 0.004 * (i % 3) as f64,
            0.008,
            0.008,
            (
                0.045 + 0.01 * cos(radians(a * 3.0)),
                y + 0.016 + 0.009 * (i % 3) as f64,
                0.01 + 0.03 * sin(radians(a * 2.0))
            ),
            M.fries,
            yaw = a,
            tilt = (0.0, (18 + 9 * (i % 3)) as f64)
        );
    }
    done("takeout")
}

/// A burger 8.8 cm across, its base at the game point (`bx`, `y`, `bz`): a sesame bun, a patty, cheese, lettuce
/// and tomato.
pub fn _burger(bx: f64, y: f64, bz: f64) {
    lathe!(
        "bun_base",
        &[
            (0.0, 0.0),
            (0.04, 0.0),
            (0.0425, 0.008),
            (0.04, 0.016),
            (0.0, 0.016)
        ],
        M.bun_base,
        at = (bx, y, bz),
        segments = 10
    );
    _patty("patty", (bx, y + 0.016, bz), 0.09, 0.015, false);
    box_!(
        "cheese",
        0.07,
        0.002,
        0.07,
        (bx, y + 0.032, bz),
        M.cheese,
        yaw = 45.0,
        tilt = (2.0, -2.0)
    );
    let outline: Vec<(f64, f64)> = (0..14)
        .map(|k| {
            let r = 0.048 * (1.0 + 0.1 * (k % 2) as f64);
            let a = TAU * k as f64 / 14.0;
            (r * cos(a), r * sin(a))
        })
        .collect();
    let lt = fextrude!(
        F,
        "lettuce",
        &outline,
        0.003,
        (bx, y + 0.0345, bz),
        M.lettuce,
        plane = Facing::Top
    );
    jitter!(lt, 0.0015, size = 0.01, seed = 2.2);
    coin!(
        "tomato",
        0.07,
        0.004,
        (bx, y + 0.036, bz),
        M.tomato,
        segments = 10,
        cup = 0.0
    );
    lathe!(
        "bun_top",
        &[
            (0.0, 0.0),
            (0.043, 0.0),
            (0.044, 0.008),
            (0.035, 0.026),
            (0.018, 0.034),
            (0.0, 0.036)
        ],
        [M.bun_base, M.bun, M.bun, M.bun, M.bun],
        at = (bx, y + 0.04, bz),
        segments = 12
    );
    for (i, (dx, dz)) in [(0.0, 0.0), (0.016, 0.01), (-0.014, 0.012), (0.01, -0.017)]
        .into_iter()
        .enumerate()
    {
        let h = 0.036 - 0.01 * (dx * dx + dz * dz) / 0.0004;
        box_!(
            format!("sesame{i}"),
            0.004,
            0.0015,
            0.0025,
            (bx + dx, y + 0.04 + h, bz + dz),
            M.sesame,
            yaw = (37 * i) as f64,
            tilt = (-dz * 900.0, dx * 900.0)
        );
    }
}

/// The tasting board or cheese board cleared: a cheese rind, a bare grape stalk, a broken cracker, crumbs and a
/// smear of jam.
pub fn board_dirty() -> Vec<Obj> {
    _paddle("board", M.board_dark, 0.0, 0.028);
    blob!(
        "smear",
        (-0.06, 0.028, 0.0),
        0.07,
        0.04,
        0.0014,
        M.grease,
        seed = 0.8,
        yaw = 12.0
    );
    blob!(
        "jam",
        (0.03, 0.028, 0.065),
        0.02,
        0.012,
        0.0014,
        M.jam,
        seed = 1.6,
        yaw = -20.0,
        n = 7
    );
    prism!(
        "rind",
        &[(-0.14, -0.05), (-0.11, -0.04), (-0.135, -0.02)],
        0.028,
        0.04,
        M.cheese,
        M.cheese
    );
    kit::tube!(
        F,
        "stalk",
        &[
            (0.13, 0.031, -0.03),
            (0.11, 0.0325, -0.03),
            (0.08, 0.031, -0.02)
        ],
        0.0018,
        M.leaf,
        verts = 4,
        round = 2
    );
    for (i, (x, z, dx, dz)) in [
        (0.11, -0.03, -0.004, 0.02),
        (0.095, -0.026, 0.006, 0.018),
        (0.09, -0.023, -0.01, -0.015),
    ]
    .into_iter()
    .enumerate()
    {
        kit::tube!(
            F,
            format!("twig{i}"),
            &[(x, 0.0315, z), (x + dx, 0.0305, z + dz)],
            0.0011,
            M.leaf,
            verts = 4,
            round = 1
        );
    }
    box_!(
        "cracker",
        0.03,
        0.005,
        0.022,
        (-0.03, 0.0305, 0.07),
        M.cracker,
        yaw = 25.0,
        bevel = 0.0012
    );
    for (i, (x, z)) in [
        (-0.02, 0.05),
        (0.03, 0.03),
        (-0.09, 0.06),
        (0.05, -0.06),
        (0.0, -0.03),
        (-0.12, 0.02),
    ]
    .into_iter()
    .enumerate()
    {
        chunk!(
            format!("crumb{i}"),
            0.007,
            (x, 0.031, z),
            M.cracker,
            yaw = (33 * i) as f64
        );
    }
    done("boardDirty")
}

/// The pizza's round board after it: crust ends left at the edge, a grease smear and a smudge of sauce, crumbs
/// and the pizza wheel.
pub fn pizza_board_dirty() -> Vec<Obj> {
    lathe!(
        "board",
        &[
            (0.0, 0.0),
            (0.197, 0.0),
            (0.2, 0.004),
            (0.2, 0.016),
            (0.197, 0.02),
            (0.0, 0.02)
        ],
        M.peel,
        segments = 28
    );
    blob!(
        "grease",
        (-0.01, 0.02, 0.01),
        0.075,
        0.055,
        0.0012,
        M.grease,
        seed = 0.9,
        yaw = 10.0,
        n = 10
    );
    blob!(
        "sauce",
        (0.05, 0.0208, -0.04),
        0.03,
        0.015,
        0.0012,
        M.smear_sauce,
        seed = 1.9,
        yaw = -25.0,
        n = 7
    );
    for (i, (a, length)) in [(30.0, 0.06), (150.0, 0.05), (250.0, 0.055)]
        .into_iter()
        .enumerate()
    {
        let (c, s) = (cos(radians(a)), sin(radians(a)));
        let cr = roll!(
            format!("crust{i}"),
            0.009,
            length,
            M.dough_baked,
            (0.13 * c, 0.029, 0.13 * s),
            yaw = a + 90.0,
            segments = 7,
            bend = -0.006
        );
        jitter!(cr, 0.0008, size = 0.01, seed = i as f64 + 0.3, up = 0.3);
    }
    for (i, (x, z)) in [
        (-0.05, 0.04),
        (0.03, 0.08),
        (0.08, 0.02),
        (-0.02, -0.07),
        (-0.08, -0.02),
    ]
    .into_iter()
    .enumerate()
    {
        chunk!(
            format!("crumb{i}"),
            0.008,
            (x, 0.0235, z),
            M.crumb,
            yaw = (40 * i) as f64
        );
    }
    // the pizza wheel, lying on its side: the blade, its guard and hub, and a handle
    let (wx, wz) = (-0.06, -0.06);
    disc!(
        "wheel",
        0.064,
        0.0016,
        (wx, 0.0205, wz),
        M.steel,
        verts = 16
    );
    disc!("hub", 0.016, 0.006, (wx, 0.0205, wz), M.steel, verts = 8);
    fsweep!(
        F,
        "handle",
        &[
            (wx - 0.006, 0.026, wz),
            (wx - 0.03, 0.0275, wz - 0.004),
            (wx - 0.085, 0.0275, wz - 0.014)
        ],
        0.0,
        M.grip,
        profile = rounded_rect(0.014, 0.018, 0.006, 1),
        scale = [0.7, 1.0, 1.05]
    );
    done("pizzaBoardDirty")
}

/// The pie server's blade, its tip at the origin.
const BLADE: [(f64, f64); 7] = [
    (0.0, 0.0),
    (0.06, -0.028),
    (0.066, -0.026),
    (0.07, -0.012),
    (0.07, 0.012),
    (0.066, 0.026),
    (0.06, 0.028),
];

/// The pie dish emptied: a red smear of filling over the bottom, pastry crumbs against its sides and the pie
/// server left in it.
pub fn pie_dish_dirty() -> Vec<Obj> {
    _pie_dish(&[M.dish, M.residue], &[(0.099, 0.01), (0.0, 0.01)]);
    blob!(
        "filling",
        (0.01, 0.01, 0.0),
        0.064,
        0.048,
        0.0016,
        M.smear_filling,
        seed = 0.4,
        yaw = 20.0,
        n = 10
    );
    for (i, a) in [20.0, 95.0, 170.0, 230.0, 300.0, 340.0]
        .into_iter()
        .enumerate()
    {
        let (c, s) = (cos(radians(a)), sin(radians(a)));
        chunk!(
            format!("pastry{i}"),
            0.011 + 0.002 * (i % 2) as f64,
            (0.088 * c, 0.016, 0.088 * s),
            M.pastry_baked,
            yaw = a,
            tilt = (15.0, 10.0)
        );
    }
    let blade: Vec<(f64, f64)> = BLADE.iter().map(|&(u, v)| (u - 0.08, v)).collect();
    fextrude!(
        F,
        "server",
        &blade,
        0.0016,
        (0.0, 0.0118, -0.02),
        M.steel,
        plane = Facing::Top,
        rot = (0, 0, -20)
    );
    fsweep!(
        F,
        "server_handle",
        &[
            (-0.0094, 0.0124, -0.0166),
            (0.0235, 0.014, -0.0286),
            (0.094, 0.016, -0.0542)
        ],
        0.0,
        M.steel,
        profile = rounded_rect(0.003, 0.009, 0.0012, 1),
        scale = [0.6, 0.9, 1.1]
    );
    done("pieDishDirty")
}

/// The cake stand emptied (the baked cake's, as the tabletop things' `cakeStand` draws it: a turned foot, a knop on
/// its stem, the plate's raised edge): smears of icing and crumbs of sponge on the plate, the cherry's stalk and the
/// cake server left on it.
pub fn cake_stand_dirty() -> Vec<Obj> {
    let top = 0.102;
    lathe!(
        "stand",
        &[
            (0.0, 0.0),
            (0.078, 0.0),
            (0.08, 0.008),
            (0.03, 0.022),
            (0.02, 0.06),
            (0.026, 0.072),
            (0.02, 0.084),
            (0.13, 0.092),
            (0.14, 0.1),
            (0.0, top)
        ],
        M.china,
        segments = 16
    );
    blob!(
        "icing",
        (0.02, top, 0.01),
        0.07,
        0.05,
        0.0018,
        M.cake_icing,
        seed = 1.1,
        yaw = 15.0,
        n = 10
    );
    blob!(
        "icing_drip",
        (-0.07, top, -0.05),
        0.022,
        0.014,
        0.0014,
        M.cake_icing,
        seed = 3.4,
        yaw = -30.0
    );
    for (i, (x, z)) in [
        (-0.05, 0.06),
        (0.06, -0.06),
        (0.09, 0.03),
        (-0.02, -0.08),
        (0.03, 0.09),
        (-0.09, 0.0),
        (0.07, 0.07),
    ]
    .into_iter()
    .enumerate()
    {
        lump!(
            format!("crumb{i}"),
            0.005 + 0.0015 * (i % 3) as f64,
            (x, top, z),
            M.sponge_crumb,
            scale = (1.2, 0.7, 1.0),
            seed = 0.9 * i as f64
        );
    }
    sweep!(
        "cherry_stalk",
        &[
            (-0.03, top + 0.001, 0.03),
            (-0.024, top + 0.002, 0.036),
            (-0.016, top + 0.001, 0.04)
        ],
        0.0018,
        material = M.stalk,
        verts = 4
    );
    let blade: Vec<(f64, f64)> = BLADE.iter().map(|&(u, v)| (u - 0.08, v)).collect();
    fextrude!(
        F,
        "server",
        &blade,
        0.0016,
        (0.0, top + 0.0018, -0.02),
        M.steel,
        plane = Facing::Top,
        rot = (0, 0, -20)
    );
    fsweep!(
        F,
        "server_handle",
        &[
            (-0.0094, top + 0.0024, -0.0166),
            (0.0235, top + 0.004, -0.0286),
            (0.094, top + 0.006, -0.0542)
        ],
        0.0,
        M.steel,
        profile = rounded_rect(0.003, 0.009, 0.0012, 1),
        scale = [0.6, 0.9, 1.1]
    );
    done("cakeStandDirty")
}
