//! The plates of food (stew, pizza, treats, pie, cake, a sandwich, from the barbecue, the tasting menu, a salad), a
//! plate after the meal and a stack of them.

use super::M;
use super::common::{F, G3, WELL, arc, done, plate, plate0};
use super::dishes::{_corn, _patty};
use super::hand::_star;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{PI, TAU, cos, radians, sin, tan};
use artkit::place::Facing;
use artkit::py;
use artkit::scene::Obj;
use artkit::{fextrude, jitter};

/// A plate of stew: chunks of beef and carrot coins in gravy, a dollop of mashed potato swirled to a peak,
/// and green beans.
pub fn meal_plate() -> Vec<Obj> {
    plate0();
    blob!(
        "gravy",
        (0.025, WELL, 0.0),
        0.07,
        0.058,
        0.003,
        M.stew,
        seed = 0.9,
        yaw = 10.0
    );
    for (i, (x, z)) in [(0.035, 0.02), (0.0, -0.02), (0.06, -0.025), (0.01, 0.04)]
        .into_iter()
        .enumerate()
    {
        chunk!(
            format!("beef{i}"),
            0.018,
            (x, WELL + 0.0095, z),
            M.beef,
            yaw = (30 * i) as f64,
            tilt = ((12 * (i as i64 % 3 - 1)) as f64, 9.0)
        );
    }
    for (i, (x, z, t)) in [
        (0.06, 0.03, 10.0),
        (0.03, -0.045, -12.0),
        (-0.005, 0.012, 6.0),
    ]
    .into_iter()
    .enumerate()
    {
        coin!(
            format!("carrot{i}"),
            0.022,
            0.005,
            (x, WELL + 0.003, z),
            (M.carrot, M.carrot_core),
            tilt = (t, -t * 0.5)
        );
    }
    chunk!(
        "potato",
        0.016,
        (0.075, WELL + 0.008, -0.002),
        M.potato,
        yaw = 30.0,
        tilt = (10.0, 5.0)
    );
    let mash = lathe!(
        "mash",
        &[
            (0.0, 0.0),
            (0.036, 0.0),
            (0.037, 0.01),
            (0.031, 0.028),
            (0.018, 0.044),
            (0.006, 0.051),
            (0.0, 0.052)
        ],
        M.potato,
        at = (-0.055, WELL, -0.04),
        segments = 14,
        wave = (7, vec![(2, 0.05), (3, 0.08), (4, 0.1), (5, 0.1)]),
        rot = (0, 0, 15)
    );
    jitter!(mash, 0.0012, size = 0.012, seed = 5.3);
    for (i, (x, z, yaw, b)) in [
        (-0.05, 0.05, 15.0, 0.004),
        (-0.04, 0.066, 5.0, -0.003),
        (-0.066, 0.04, 30.0, 0.003),
        (-0.03, 0.04, 40.0, -0.004),
    ]
    .into_iter()
    .enumerate()
    {
        let a = radians(yaw);
        let (ca, sa) = (cos(a), sin(a));
        let y = WELL + 0.0045 + 0.004 * (i % 2) as f64;
        kit::tube!(
            F,
            format!("bean{i}"),
            &[
                (x - 0.036 * ca, y, z + 0.036 * sa),
                (x + b * sa, y + 0.002, z + b * ca),
                (x + 0.036 * ca, y, z - 0.036 * sa)
            ],
            0.0048,
            M.pea,
            verts = 5,
            round = 1
        );
    }
    leaf!(
        "parsley",
        (0.03, WELL + 0.016, 0.0),
        0.016,
        0.01,
        40,
        M.leaf,
        tilt = 10.0
    );
    done("mealPlate")
}

/// `pts` drawn in towards their middle by a share.
pub fn _shrunk(pts: &[(f64, f64)], by: f64) -> Vec<(f64, f64)> {
    let cx = artkit::py::sum(pts.iter().map(|p| p.0)) / pts.len() as f64;
    let cz = artkit::py::sum(pts.iter().map(|p| p.1)) / pts.len() as f64;
    pts.iter()
        .map(|&(x, z)| (cx + (x - cx) * (1.0 - by), cz + (z - cz) * (1.0 - by)))
        .collect()
}

/// A plate with a slice of pizza: the base, tomato showing at its cut sides, cheese browned in a spot,
/// pepperoni and basil, and its puffed crust along the back.
pub fn pizza_slice() -> Vec<Obj> {
    plate0();
    _pizza_slice(WELL, 0.0);
    done("pizzaSlice")
}

/// A slice of pizza 15 cm from its tip (to the front) to its crust, its base at height `y`, moved `dz` along the
/// depth.
pub fn _pizza_slice(y: f64, dz: f64) {
    let (apex, r, a) = ((0.0, 0.075 + dz), 0.15, 21.0);
    let mut wedge = vec![apex];
    wedge.extend(arc(r, a, apex, 3));
    prism!("base", &wedge, y, y + 0.006, M.dough, M.dough);
    prism!(
        "topping",
        &_shrunk(&wedge, 0.04),
        y + 0.006,
        y + 0.011,
        M.mozzarella,
        M.sauce
    );
    let c = crust!(
        "crust",
        &arc(r - 0.009, a + 0.5, apex, 8),
        y + 0.012,
        0.012,
        M.dough_baked,
        flat = 0.83,
        taper = 0.8
    );
    jitter!(c, 0.0008, size = 0.015, seed = 2.4, up = 0.3);
    for (i, (x, z)) in [(0.0, 0.0), (-0.025, -0.045), (0.028, -0.04)]
        .into_iter()
        .enumerate()
    {
        coin!(
            format!("pepperoni{i}"),
            0.03,
            0.0025,
            (x, y + 0.0108, z + dz),
            M.pepperoni,
            segments = 10
        );
    }
    leaf!(
        "basil",
        (0.005, y + 0.0122, 0.035 + dz),
        0.022,
        0.013,
        30,
        M.leaf,
        tilt = 4.0
    );
    blob!(
        "brown",
        (-0.012, y + 0.0112, 0.02 + dz),
        0.007,
        0.005,
        0.0005,
        M.cheese_brown,
        seed = 1.0,
        n = 6
    );
}

/// A chocolate-chip cookie 7.6 cm across, its base at the game point `at`, tilted (Blender x, y degrees), a chip or
/// two showing on its top.
pub fn _cookie(
    name: &str,
    at: (f64, f64, f64),
    tilt: (f64, f64),
    seed: f64,
    chips: usize,
    yaw: f64,
) {
    let (x, y, z) = at;
    let ck = lathe!(
        name,
        &[
            (0.0, 0.0),
            (0.036, 0.0),
            (0.038, 0.006),
            (0.033, 0.012),
            (0.0, 0.013)
        ],
        M.cookie,
        at = at,
        segments = 10,
        rot = (tilt.0, tilt.1, 0.0)
    );
    jitter!(ck, 0.0009, size = 0.01, seed = seed, up = 0.5);
    let a = radians(yaw);
    for (k, &(u, w)) in [(-0.015, -0.008), (0.012, 0.012)][..chips]
        .iter()
        .enumerate()
    {
        let (dx, dz) = (u * cos(a) - w * sin(a), u * sin(a) + w * cos(a));
        let lift = dz * tan(radians(tilt.0)) - dx * tan(radians(tilt.1));
        chunk!(
            format!("{name}_chip{k}"),
            0.008,
            (x + dx, y + 0.0125 + lift, z + dz),
            M.chocolate,
            yaw = (20 * k) as f64 + yaw,
            tilt = (15.0, 10.0)
        );
    }
}

/// A cupcake in a pleated pink case, its icing piped in a swirl to a peak, a cherry with its stalk on top; its base
/// at the game point `at`.
pub fn _cupcake(name: &str, at: (f64, f64, f64)) {
    let (cx, y, cz) = at;
    lathe!(
        name,
        &[
            (0.0, 0.0),
            (0.022, 0.0),
            (0.0285, 0.025),
            (0.0262, 0.0262),
            (0.0, 0.0262)
        ],
        [M.cupcake_case, M.cupcake_case, M.cupcake_case, M.icing],
        at = at,
        segments = 16,
        wave = (8, vec![(2, 0.07), (3, 0.07)])
    );
    swirl!(
        format!("{name}_icing"),
        (cx, y + 0.024, cz),
        0.019,
        0.025,
        M.icing,
        tube = 0.0095,
        turns = 2.0,
        steps = 10,
        star = 6
    );
    ball!(
        format!("{name}_cherry"),
        0.0075,
        (cx, y + 0.0555, cz),
        M.tomato,
        seg = 6,
        rings = 5
    );
    kit::tube!(
        F,
        format!("{name}_stalk"),
        &[
            (cx, y + 0.0605, cz),
            (cx + 0.005, y + 0.0632, cz - 0.002),
            (cx + 0.011, y + 0.0625, cz - 0.004)
        ],
        0.0008,
        M.leaf,
        verts = 4,
        round = 1
    );
}

/// A plate of treats: chocolate-chip cookies, one leaning on the others, and a cupcake in a pleated case,
/// its icing piped in a swirl to a peak, a cherry on top (all in the well, clear of the rim).
pub fn treat_plate() -> Vec<Obj> {
    plate0();
    for (i, (at, tilt)) in [
        ((-0.05, WELL, 0.022), (0.0, 0.0)),
        ((0.036, WELL, 0.036), (0.0, 0.0)),
        ((-0.01, WELL + 0.012, -0.004), (9.0, -6.0)),
    ]
    .into_iter()
    .enumerate()
    {
        _cookie(
            &format!("cookie{i}"),
            at,
            tilt,
            i as f64 + 0.5,
            if i == 2 { 2 } else { 1 },
            (10 * i) as f64,
        );
    }
    _cupcake("cupcake", (0.042, WELL, -0.05));
    done("treatPlate")
}

/// A plate with a slice of cherry pie: pastry under and over the red filling, its crimped edge along the
/// back, filling run out at its tip, and a piped rosette of cream with a mint leaf.
pub fn pie_slice() -> Vec<Obj> {
    plate0();
    let (apex, r, a) = ((-0.015, 0.055), 0.11, 24.0); // (a sixth of the 26 cm pie: 11 cm from its tip to its crust)
    let mut wedge = vec![apex];
    wedge.extend(arc(r, a, apex, 3));
    prism!(
        "pastry_base",
        &wedge,
        WELL,
        WELL + 0.008,
        M.pastry,
        M.pastry_baked
    );
    prism!(
        "filling",
        &_shrunk(&wedge, 0.03),
        WELL + 0.008,
        WELL + 0.036,
        M.filling,
        M.filling
    );
    let mut sides = vec![M.pastry_baked];
    sides.extend(vec![M.pastry; wedge.len() - 1]);
    prism!(
        "pastry_top",
        &wedge,
        WELL + 0.036,
        WELL + 0.043,
        M.pastry,
        M.pastry,
        sides = sides
    );
    let c = crust!(
        "edge",
        &arc(r - 0.008, a + 0.5, apex, 10),
        WELL + 0.039,
        0.011,
        M.pastry_baked,
        flat = 0.85,
        crimp = 5,
        bulge = 0.18,
        taper = 0.85
    );
    jitter!(c, 0.0005, size = 0.015, seed = 3.0);
    for (i, (x, z)) in [(-0.015, -0.01), (-0.015, -0.04)].into_iter().enumerate() {
        box_!(
            format!("slit{i}"),
            0.022,
            0.002,
            0.004,
            (x, WELL + 0.0435, z),
            M.filling
        );
    }
    blob!(
        "ooze",
        (-0.015, WELL, 0.063),
        0.014,
        0.01,
        0.0014,
        M.filling,
        seed = 0.6,
        n = 7
    );
    swirl!(
        "cream",
        (0.07, WELL, 0.035),
        0.014,
        0.03,
        M.cream,
        tube = 0.008,
        turns = 1.8,
        steps = 11
    );
    leaf!(
        "mint",
        (0.066, WELL + 0.03, 0.033),
        0.016,
        0.009,
        50,
        M.leaf,
        tilt = 20.0
    );
    done("pieSlice")
}

/// The sandwich and toast halves: a slice cut corner to corner, its loaf corner rounded. Each half's outline,
/// the index of its cut side, and which way it is drawn back from the cut.
pub type Half = (&'static str, Vec<(f64, f64)>, usize, i32);

pub fn _halves(s: (f64, f64), rc: f64) -> [Half; 2] {
    let (hw, hd) = s;
    let k = rc * (1.0 - 0.5_f64.sqrt());
    let a = vec![
        (-hw, -hd + rc),
        (-hw + k, -hd + k),
        (-hw + rc, -hd),
        (hw, -hd),
        (-hw, hd),
    ];
    let b = vec![
        (hw, -hd),
        (hw, hd - rc),
        (hw - k, hd - k),
        (hw - rc, hd),
        (-hw, hd),
    ];
    [("a", a, 3, -1), ("b", b, 4, 1)]
}

/// A lettuce leaf's outline: `pts` grown by a share, each side waved out and in `n` times.
fn _frill(pts: &[(f64, f64)], grow: f64, amp: f64, n: usize) -> Vec<(f64, f64)> {
    let g = _shrunk(pts, -grow);
    let cx = artkit::py::sum(g.iter().map(|p| p.0)) / g.len() as f64;
    let cz = artkit::py::sum(g.iter().map(|p| p.1)) / g.len() as f64;
    let mut out = Vec::new();
    for (i, &p) in g.iter().enumerate() {
        let q = g[(i + 1) % g.len()];
        for k in 0..n {
            let f = k as f64 / n as f64;
            let (x, z) = (p.0 + (q.0 - p.0) * f, p.1 + (q.1 - p.1) * f);
            let mut d = py::hypot(x - cx, z - cz);
            if d == 0.0 {
                d = 1.0;
            }
            let w = amp * if k % 2 != 0 { 1.0 } else { -0.4 };
            out.push((x + (x - cx) / d * w, z + (z - cz) / d * w));
        }
    }
    out
}

/// A sandwich cut corner to corner: two halves of bread, their crusts round the outside and the crumb at the
/// cut and on top, lettuce frilled out past the bread, cheese and tomato.
pub fn sandwich() -> Vec<Obj> {
    plate0();
    let (nx, nz) = (0.052, 0.058);
    let n = py::hypot(nx, nz);
    for (half, pts, cut, sign) in _halves((0.058, 0.052), 0.012) {
        let s = sign as f64;
        _sandwich_half(
            half,
            &pts,
            cut,
            (s * nx / n * 0.008, s * nz / n * 0.008),
            WELL,
            if sign > 0 { 3.0 } else { 7.0 },
        );
    }
    done("sandwich")
}

/// Half a sandwich over the outline `pts` (its side `cut` the cut, the crumb showing; crusts round the rest), moved
/// `off`, from height `y`: bread, lettuce frilled out past it, cheese, tomato, bread.
pub fn _sandwich_half(
    half: &str,
    pts: &[(f64, f64)],
    cut: usize,
    off: (f64, f64),
    y: f64,
    seed: f64,
) {
    let mut y = y;
    for (layer, h, m, grow) in [
        ("bread_lo", 0.019, M.bread, 0.0),
        ("lettuce", 0.006, M.lettuce, 0.08),
        ("cheese", 0.004, M.cheese, 0.05),
        ("tomato", 0.007, M.tomato, -0.04),
        ("bread_hi", 0.019, M.bread, 0.0),
    ] {
        if m == M.bread {
            let sides: Vec<Mat> = (0..pts.len())
                .map(|k| if k == cut { M.bread } else { M.crust })
                .collect();
            prism!(
                format!("{layer}_{half}"),
                pts,
                y,
                y + h,
                M.bread,
                M.crust,
                sides = sides,
                at = off,
                bevel = 0.003
            );
        } else if m == M.lettuce {
            let outline: Vec<(f64, f64)> = _frill(pts, 0.07, 0.004, 4)
                .iter()
                .map(|&(x, z)| (x + off.0, z + off.1))
                .collect();
            let lt = fextrude!(
                F,
                format!("lettuce_{half}"),
                &outline,
                0.004,
                (0, y + 0.003, 0),
                M.lettuce,
                plane = Facing::Top
            );
            jitter!(lt, 0.0016, size = 0.01, seed = seed, up = 1.0);
        } else {
            prism!(
                format!("{layer}_{half}"),
                &_shrunk(pts, -grow),
                y,
                y + h,
                m,
                m,
                at = off
            );
        }
        y += h;
    }
}

/// A steel table fork lying with its tines at the game point `at` (its underside there), handle out along
/// `yaw` (degrees): handle, neck, a head of four tines; `pitch` raises the handle's end (degrees).
fn _fork(name: &str, at: (f64, f64, f64), yaw: f64, pitch: f64) -> Obj {
    let o = [
        (0.15, -0.0055),
        (0.15, 0.0055),
        (0.08, 0.0042),
        (0.046, 0.0026),
        (0.03, 0.0095),
        (0.021, 0.00975),
        (0.0, 0.00975),
        (0.0, 0.00675),
        (0.017, 0.00675),
        (0.017, 0.00425),
        (0.0, 0.00425),
        (0.0, 0.00125),
        (0.017, 0.00125),
        (0.017, -0.00125),
        (0.0, -0.00125),
        (0.0, -0.00425),
        (0.017, -0.00425),
        (0.017, -0.00675),
        (0.0, -0.00675),
        (0.0, -0.00975),
        (0.021, -0.00975),
        (0.03, -0.0095),
        (0.046, -0.0026),
        (0.08, -0.0042),
    ];
    let reversed: Vec<(f64, f64)> = o.iter().rev().copied().collect();
    fextrude!(
        F,
        name,
        &reversed,
        0.0022,
        (at.0, at.1 + 0.0011, at.2),
        M.steel,
        plane = Facing::Top,
        rot = (0.0, pitch, yaw)
    )
}

/// A table knife lying with its tip at the game point `at`: a rounded blade and a handle.
fn _knife(name: &str, at: (f64, f64, f64), yaw: f64, pitch: f64) -> Obj {
    let o = [
        (0.0, 0.0),
        (0.006, -0.0075),
        (0.09, -0.0085),
        (0.098, -0.0055),
        (0.2, -0.0065),
        (0.205, -0.004),
        (0.205, 0.004),
        (0.2, 0.0065),
        (0.098, 0.0055),
        (0.09, 0.0035),
        (0.02, 0.0045),
    ];
    fextrude!(
        F,
        name,
        &o,
        0.0022,
        (at.0, at.1 + 0.0011, at.2),
        M.steel,
        plane = Facing::Top,
        rot = (0.0, pitch, yaw)
    )
}

/// A plate with a slice of the café's layer cake: two sponge layers with jam and cream between, white icing over
/// the top and round its back (the cut sides show the layers), a strawberry standing on it, its green calyx spread on
/// the icing and its seeds showing.
pub fn cake_slice() -> Vec<Obj> {
    plate0();
    let (apex, r, a) = ((-0.012, 0.055), 0.115, 20.0);
    let mut wedge = vec![apex];
    wedge.extend(arc(r, a, apex, 3));
    let mut y = WELL;
    for (name, h, top, cut) in [
        ("sponge0", 0.03, M.sponge, M.sponge_crumb),
        ("jam", 0.004, M.cake_jam, M.cake_jam),
        ("cream", 0.006, M.cake_icing, M.cake_icing),
        ("sponge1", 0.03, M.sponge, M.sponge_crumb),
    ] {
        // (Its cut sides the layer's own; its round back iced.)
        let mut sides = vec![cut];
        sides.extend(vec![M.cake_icing; wedge.len() - 2]);
        sides.push(cut);
        // (the sponge's edges soft)
        let bevel = if h > 0.01 { 0.002 } else { 0.0 };
        prism!(
            name,
            &wedge,
            y,
            y + h,
            top,
            cut,
            sides = sides,
            bevel = bevel
        );
        y += h;
    }
    let mut sides = vec![M.sponge_crumb];
    sides.extend(vec![M.cake_icing; wedge.len() - 2]);
    sides.push(M.sponge_crumb);
    prism!(
        "icing",
        &wedge,
        y,
        y + 0.005,
        M.cake_icing,
        M.cake_icing,
        sides = sides
    );
    let back = arc(r, a, apex, 3);
    let (bx, bz) = (
        back[1].0 * 0.45 + apex.0 * 0.55,
        back[1].1 * 0.45 + apex.1 * 0.55,
    );
    let top = y + 0.005;
    // The strawberry: a cone with round shoulders, its tip up, on a star of a calyx.
    lathe!(
        "berry",
        &[
            (0.0, 0.0),
            (0.0115, 0.002),
            (0.0135, 0.009),
            (0.011, 0.018),
            (0.0055, 0.026),
            (0.0, 0.03)
        ],
        M.cake_berry,
        at = (bx, top, bz),
        segments = 8
    );
    fextrude!(
        F,
        "calyx",
        &_star(0.016, 0.38, 5),
        0.0012,
        (bx, top + 0.0012, bz),
        M.leaf,
        plane = Facing::Top,
        rot = (0, 0, 15)
    );
    for (k, (a, h)) in [
        (20.0, 0.012),
        (140.0, 0.014),
        (260.0, 0.011),
        (80.0, 0.021),
        (200.0, 0.02),
        (320.0, 0.019),
    ]
    .into_iter()
    .enumerate()
    {
        let r = if h < 0.015 { 0.0128 } else { 0.0098 };
        box_!(
            format!("seed{k}"),
            0.0016,
            0.0024,
            0.0016,
            (bx + r * cos(radians(a)), top + h, bz + r * sin(radians(a))),
            M.berry_seed,
            yaw = a
        );
    }
    done("cakeSlice")
}

/// Something fresh from the garden: a salad on the dinner plate, eaten with a fork: a heap of torn leaves in three
/// greens, wedges of tomato, cucumber and radish sliced over it.
pub fn fresh_plate() -> Vec<Obj> {
    plate0();
    #[allow(clippy::too_many_arguments)] // (as the script called it)
    fn greens(
        name: &str,
        at: (f64, f64, f64),
        r: f64,
        yaw: f64,
        tilt: f64,
        material: Mat,
        seed: f64,
    ) {
        let pts: Vec<(f64, f64)> = (0..10)
            .map(|k| {
                let f = if k % 2 == 0 { 1.0 } else { 0.8 };
                let a = TAU * k as f64 / 10.0;
                (r * f * cos(a), r * 0.72 * f * sin(a))
            })
            .collect();
        let g = fextrude!(
            F,
            name,
            &pts,
            0.0018,
            (at.0, at.1 + 0.0009, at.2),
            material,
            plane = Facing::Top,
            rot = (tilt, 0.0, yaw)
        );
        jitter!(g, 0.0032, size = 0.014, seed = seed, up = 1.0);
    }
    for (i, (x, y, z, r, yaw, tilt, m)) in [
        (-0.035, 0.006, 0.022, 0.05, 20.0, 16.0, M.lettuce),
        (0.035, 0.006, -0.018, 0.048, 110.0, -18.0, M.lettuce_pale),
        (0.005, 0.006, 0.048, 0.045, 160.0, 20.0, M.leaf),
        (-0.02, 0.016, -0.032, 0.044, 70.0, -22.0, M.lettuce),
        (0.01, 0.026, 0.006, 0.04, 140.0, 14.0, M.lettuce_pale),
        (-0.03, 0.03, 0.002, 0.034, 40.0, -16.0, M.leaf),
    ]
    .into_iter()
    .enumerate()
    {
        greens(
            &format!("greens{i}"),
            (x, WELL + y, z),
            r * 1.15,
            yaw,
            tilt,
            m,
            1.1 * i as f64 + 0.4,
        );
    }
    // Tomato wedges lying on their skins, the cut flesh up.
    for (i, (x, z, yaw, y)) in [
        (0.055, 0.03, 30.0, 0.004),
        (0.04, 0.055, 100.0, 0.006),
        (-0.06, 0.045, -20.0, 0.006),
    ]
    .into_iter()
    .enumerate()
    {
        let a = radians(yaw);
        let (ca, sa) = (cos(a), sin(a));
        let half: Vec<(f64, f64)> = (0..5)
            .map(|k| {
                (
                    0.024 * cos(PI * k as f64 / 4.0),
                    0.011 * sin(PI * k as f64 / 4.0),
                )
            })
            .collect();
        let pts: Vec<(f64, f64)> = half
            .iter()
            .map(|&(u, w)| (x + u * ca - w * sa, z + u * sa + w * ca))
            .collect();
        let mut sides = vec![M.tomato; 4];
        sides.push(M.tomato_flesh);
        prism!(
            format!("tomato{i}"),
            &pts,
            WELL + y,
            WELL + y + 0.013,
            M.tomato_flesh,
            M.tomato,
            sides = sides
        );
    }
    for (i, (x, y, z, t)) in [
        (-0.008, 0.036, -0.004, 8.0),
        (0.025, 0.031, 0.02, -10.0),
        (-0.045, 0.022, -0.045, 12.0),
        (0.062, 0.012, -0.04, -8.0),
    ]
    .into_iter()
    .enumerate()
    {
        coin!(
            format!("cucumber{i}"),
            0.03,
            0.004,
            (x, WELL + y, z),
            (M.cucumber, M.cucumber_flesh),
            segments = 8,
            cup = 0.0,
            tilt = (t, -t * 0.7)
        );
    }
    for (i, (x, y, z, t)) in [
        (0.012, 0.037, -0.028, -10.0),
        (-0.03, 0.03, 0.035, 12.0),
        (0.045, 0.016, -0.065, 6.0),
    ]
    .into_iter()
    .enumerate()
    {
        coin!(
            format!("radish{i}"),
            0.022,
            0.0035,
            (x, WELL + y, z),
            (M.radish, M.radish_white),
            segments = 8,
            cup = 0.0,
            tilt = (t, t * 0.5)
        );
    }
    done("freshPlate")
}

/// A plate from the barbecue (`grillPlatter` serves it), eaten with a fork: a sausage marked by the bars, a marked
/// burger, a piece of corn with butter melting on it, and a sprig of parsley.
pub fn bbq_plate() -> Vec<Obj> {
    plate0();
    roll!(
        "sausage",
        0.014,
        0.13,
        (M.sausage, M.char),
        (-0.035, WELL + 0.014, 0.045),
        yaw = -15.0,
        segments = 8,
        marks = vec![(-0.55, -0.4), (-0.05, 0.1), (0.45, 0.6)],
        bend = 0.006
    );
    _patty("patty", (0.035, WELL, -0.02), 0.085, 0.024, true);
    _corn(
        "corn",
        0.02,
        0.075,
        (-0.05, WELL + 0.02, -0.045),
        25.0,
        true,
        false,
        7,
        10,
    );
    box_!(
        "butter",
        0.014,
        0.005,
        0.012,
        (-0.052, WELL + 0.0405, -0.045),
        M.butter,
        yaw = 40.0,
        bevel = 0.0018
    );
    leaf!(
        "parsley",
        (0.075, WELL + 0.0005, 0.045),
        0.022,
        0.014,
        40,
        M.leaf,
        tilt = 6.0
    );
    done("bbqPlate")
}

/// A plate from the tasting menu (`tastingBoard` serves it), small portions eaten with a fork: a wedge of cheese and
/// a cube, a fold of salami, two crackers, a few grapes, a dab of fig jam.
pub fn tasting_plate() -> Vec<Obj> {
    plate0();
    prism!(
        "cheese",
        &[(-0.075, -0.04), (-0.02, -0.022), (-0.07, 0.022)],
        WELL,
        WELL + 0.03,
        M.cheese_cut,
        M.cheese_cut,
        sides = vec![M.cheese_cut, M.cheese, M.cheese_cut]
    );
    dice!(
        "cube",
        0.016,
        (-0.01, WELL + 0.0068, 0.0),
        M.cheese_cut,
        yaw = 30.0
    );
    for i in 0..3 {
        coin!(
            format!("salami{i}"),
            0.04,
            0.0035,
            (
                0.02 + 0.012 * i as f64,
                WELL + 0.0035 * (i % 2) as f64,
                -0.05 + 0.008 * i as f64
            ),
            M.salami,
            segments = 10,
            cup = 0.25,
            tilt = (10.0, -12.0)
        );
    }
    for (i, (x, z, yaw)) in [(0.055, 0.03, 15.0), (0.07, 0.045, 40.0)]
        .into_iter()
        .enumerate()
    {
        box_!(
            format!("cracker{i}"),
            0.036,
            0.005,
            0.036,
            (x, WELL + 0.0025 + 0.005 * i as f64, z),
            M.cracker,
            yaw = yaw,
            bevel = 0.0012
        );
    }
    for (i, (x, z, y)) in [
        (-0.03, 0.05, 0.0),
        (-0.012, 0.058, 0.0),
        (-0.022, 0.07, 0.0),
        (-0.02, 0.058, 0.012),
    ]
    .into_iter()
    .enumerate()
    {
        ball!(
            format!("grape{i}"),
            0.0125,
            (x, WELL + 0.0115 + y, z),
            M.grape,
            scale = (1.0, 1.0, 0.92),
            seg = 6,
            rings = 4
        );
    }
    blob!(
        "jam",
        (0.03, WELL, 0.0),
        0.016,
        0.012,
        0.004,
        M.jam,
        seed = 1.2,
        n = 7
    );
    leaf!(
        "basil",
        (0.0, WELL + 0.0005, 0.035),
        0.02,
        0.012,
        60,
        M.leaf,
        tilt = 4.0
    );
    done("tastingPlate")
}

/// A plate after the meal: a smear of gravy, a streak of sauce, crumbs, a pea left over, and the knife and
/// fork laid side by side.
pub fn plate_dirty() -> Vec<Obj> {
    plate0();
    _leftovers(0.0, 0.0, 0.0, 4);
    // side by side at twenty past four, the knife's handle resting on the rim
    _fork("fork", (-0.0642, WELL + 0.0003, -0.0301), 35.0, 0.0);
    _knife("knife", (-0.067, WELL + 0.0003, -0.061), 35.0, 1.5);
    done("plateDirty")
}

/// What a meal leaves on a plate whose foot is at the game point (`x`, `y`, `z`): a smear of gravy, a streak of
/// sauce, crumbs and a pea.
fn _leftovers(x: f64, y: f64, z: f64, crumbs: usize) {
    blob!(
        "smear",
        (x + 0.02, y + WELL, z + 0.02),
        0.065,
        0.045,
        0.0016,
        M.smear,
        seed = 1.4,
        yaw = -15.0
    );
    blob!(
        "sauce",
        (x - 0.04, y + WELL + 0.0004, z + 0.045),
        0.035,
        0.011,
        0.0016,
        M.smear_sauce,
        seed = 0.2,
        yaw = 25.0,
        n = 7
    );
    for (i, &(u, w)) in [(-0.05, -0.02), (0.06, 0.05), (0.0, 0.07), (-0.07, 0.02)][..crumbs]
        .iter()
        .enumerate()
    {
        chunk!(
            format!("crumb{i}"),
            0.008,
            (x + u, y + WELL + 0.004, z + w),
            M.crumb,
            yaw = (40 * i) as f64
        );
    }
    ball!(
        "pea",
        0.0042,
        (x + 0.045, y + WELL + 0.0036, z - 0.03),
        M.pea,
        seg = 6,
        rings = 4
    );
}

/// How far each plate of a stack stands on the one under it: its foot in the well below.
const STACK: f64 = 0.0095;

/// Three dirty plates stacked to be carried to the sink in both hands (the sim shows it when two or more dirty
/// plates are carried at once): each the dinner plate, a little askew on the one below, its foot in that one's well;
/// the top one's leftovers and fork, a drip of gravy down the middle one's rim and a scrap of lettuce hanging over
/// its edge.
pub fn plate_stack_dirty() -> Vec<Obj> {
    let stack = [(0.0, 0.0), (0.007, -0.004), (-0.004, 0.006)];
    for (k, &(x, z)) in stack.iter().enumerate() {
        plate(
            &format!("plate{k}"),
            artkit::Arg::<G3>::arg((x, STACK * k as f64, z)),
            k < 2,
        );
    }
    let (x, z) = stack[2];
    _leftovers(x, 2.0 * STACK, z, 2);
    _fork(
        "fork",
        (x - 0.0642, 2.0 * STACK + WELL + 0.0003, z - 0.0301),
        35.0,
        0.0,
    );
    // (Where the middle plate shows past the top one's rim: its +x edge and its back.)
    blob!(
        "drip",
        (0.007 + 0.139, STACK + 0.0165, -0.004 + 0.01),
        0.004,
        0.01,
        0.0012,
        M.smear,
        seed = 0.9,
        n = 6
    );
    leaf!(
        "lettuce",
        (0.004, STACK + 0.0168, -0.004 - 0.138),
        0.03,
        0.018,
        70,
        M.lettuce,
        tilt = -18.0
    );
    done("plateStackDirty")
}
