use super::super::common::{F, M, cup, end_round, finish, plank, side_slab, v};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{PI, V3, cos, cosd, sin, sind};
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsweep};

// --- the garden chair: a white slatted armchair, the back leaning, a cup on the arm ----------------------------

/// Back slats up a back leaning `tilt` degrees, from `bottom` (height, depth); `fan` spreads their tops; their tops
/// rounded by `r` (half their width: a full round). Returns each slat's (bottom, top) game points.
#[allow(clippy::too_many_arguments, clippy::type_complexity)] // (as the script called it)
pub(super) fn back_slats(
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
pub(super) fn stringer(
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
pub(super) fn corbel(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    h: f64,
    d: f64,
    t: f64,
    material: Mat,
) -> Obj {
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
pub(super) fn slatted_chair(prefix: &str, material: Mat, fan: f64) {
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
    kit::board(
        &F,
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
