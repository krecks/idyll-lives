//! What the desks are made of and what lies on them: frames, lamps, mugs, plants, notebooks, pens, monitors,
//! keyboards, mice and laptops.

use super::common::{axle, fillet, turned};
use super::{F, FT, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{TAU, cos, radians, sin};
use artkit::parts::new_parts;
use artkit::place::Coord;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fsphere, fsweep, loft, revolve};

/// A welded frame of square tube: each side one piece (a leg up, a rail along, a leg down) on glides, a back
/// apron, and a front (resident's side) apron with a drawer gap (x0, x1) left open.
pub fn desk_frame(w: f64, legs: Option<Mat>, gap: Option<(f64, f64)>, apron: Option<Mat>) {
    let (legs, apron) = (legs.unwrap_or(M.frame), apron.unwrap_or(M.frame));
    let sq = rounded_rect(0.04, 0.04, 0.007, 1);
    for s in [-1, 1] {
        let x = f64::from(s) * (w / 2.0 - 0.04);
        fsweep!(
            F,
            format!("side{s}"),
            &fillet(
                &[
                    (x, 0.012, 0.06),
                    (x, 0.70, 0.06),
                    (x, 0.70, 0.42),
                    (x, 0.012, 0.42)
                ],
                0.012,
                2
            ),
            0.0,
            legs,
            profile = &sq,
            up = (1, 0, 0),
            angle = 50
        );
        for z in [0.06, 0.42] {
            fcyl!(
                F,
                format!("glide{s}{}", float_repr(z)),
                0.017,
                0.012,
                (x, 0.006, z),
                M.plastic,
                verts = 10
            );
        }
    }
    fbox!(
        F,
        "apron_back",
        w - 0.12,
        0.05,
        0.02,
        (0, 0.695, 0.43),
        apron,
        bevel = 0.004,
        bevel_segments = 1
    );
    let (x0, x1) = gap.unwrap_or((0.0, 0.0));
    for (k, (a, b)) in [(-w / 2.0 + 0.06, x0), (x1, w / 2.0 - 0.06)]
        .into_iter()
        .enumerate()
    {
        if b - a > 0.01 {
            fbox!(
                F,
                format!("apron_front{k}"),
                b - a,
                0.05,
                0.02,
                ((a + b) / 2.0, 0.695, 0.06),
                apron,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
    }
}

/// An architect's lamp on the top: a weighted base, two arms bent at a knuckle with their springs, the shade
/// turned down to the resident's side.
pub fn desk_lamp(x: f64, z: f64) {
    let (m, y) = (M.lamp, FT);
    revolve!(
        "lamp_base",
        &[
            (0.0, 0.0),
            (0.064, 0.0),
            (0.066, 0.008),
            (0.06, 0.018),
            (0.03, 0.024),
            (0.016, 0.034),
            (0.0, 0.034)
        ],
        at = F.at(x, y, z),
        material = m,
        segments = 20
    );
    let (root, elbow, head) = (
        (x, y + 0.03, z),
        (x + 0.04, 1.1, z - 0.05),
        (x + 0.15, 1.068, z - 0.14),
    );
    fsweep!(F, "lamp_arm", &[root, elbow, head], 0.0075, m, verts = 8);
    for (k, (a, b)) in [(root, elbow), (elbow, head)].into_iter().enumerate() {
        let ends: Vec<(f64, f64, f64)> = [0.2, 0.8]
            .iter()
            .map(|&t| {
                (
                    a.0 + (b.0 - a.0) * t + 0.011,
                    a.1 + (b.1 - a.1) * t + 0.0,
                    a.2 + (b.2 - a.2) * t + 0.011,
                )
            })
            .collect();
        fsweep!(
            F,
            format!("lamp_spring{k}"),
            &ends,
            0.0028,
            M.chrome,
            verts = 4
        );
    }
    fcyl!(
        F,
        "lamp_knuckle",
        0.013,
        0.03,
        elbow,
        m,
        rot = axle(0.14, 0.15),
        verts = 12
    );
    fcyl!(
        F,
        "lamp_neck",
        0.011,
        0.024,
        head,
        m,
        rot = axle(0.14, 0.15),
        verts = 10
    );
    revolve!(
        "lamp_shade",
        &[
            (0.0, 0.05),
            (0.016, 0.05),
            (0.024, 0.042),
            (0.06, -0.045),
            (0.063, -0.05),
            (0.057, -0.05),
            (0.02, 0.035),
            (0.0, 0.036)
        ],
        at = F.at(x + 0.17, 1.03, z - 0.16),
        material = m,
        segments = 20,
        rot = (35, 0, 0)
    );
    fsphere!(
        F,
        "lamp_bulb",
        0.022,
        (x + 0.17, 1.012, z - 0.175),
        M.bulb,
        segments = 10,
        rings = 6
    );
}

/// A mug of coffee, hollow, its handle to the right.
pub fn mug(x: f64, z: f64, y: f64) {
    let m = M.mug;
    revolve!(
        "mug",
        &[
            (0.0, 0.0),
            (0.036, 0.0),
            (0.04, 0.006),
            (0.041, 0.095),
            (0.037, 0.095),
            (0.036, 0.012),
            (0.0, 0.012)
        ],
        at = F.at(x, y, z),
        material = m,
        segments = 16
    );
    fcyl!(
        F,
        "mug_coffee",
        0.0365,
        0.004,
        (x, y + 0.078, z),
        M.coffee,
        verts = 16
    );
    fsweep!(
        F,
        "mug_handle",
        &[
            (x + 0.038, y + 0.078, z),
            (x + 0.062, y + 0.072, z),
            (x + 0.066, y + 0.05, z),
            (x + 0.06, y + 0.028, z),
            (x + 0.038, y + 0.024, z)
        ],
        0.0055,
        m,
        verts = 6,
        resolution = 2
    );
}

/// A ribbed cactus in a rimmed pot of soil.
pub fn cactus(x: f64, z: f64, y: f64, r: f64, h: f64) {
    revolve!(
        "pot",
        &[
            (0.0, 0.0),
            (r * 0.78, 0.0),
            (r * 0.94, h * 0.8),
            (r * 1.06, h * 0.82),
            (r * 1.08, h),
            (r * 0.94, h),
            (r * 0.92, h * 0.9),
            (0.0, h * 0.9)
        ],
        at = F.at(x, y, z),
        material = M.pot,
        segments = 16
    );
    fcyl!(
        F,
        "pot_soil",
        r * 0.93,
        0.004,
        (x, y + h * 0.9, z),
        M.soil,
        verts = 16
    );
    let mut secs = Vec::new();
    for (t, k) in [
        (0.0, 0.8),
        (0.12, 0.98),
        (0.5, 1.0),
        (0.8, 0.88),
        (0.93, 0.6),
        (1.0, 0.18),
    ] {
        let mut ring = Vec::new();
        for j in 0..20 {
            let a = TAU * f64::from(j) / 20.0;
            let rr = r * 0.72 * k * (1.0 + 0.1 * cos(10.0 * a));
            ring.push(F.at(x + rr * cos(a), y + h * 0.86 + t * 0.12, z + rr * sin(a)));
        }
        secs.push(ring);
    }
    loft!("cactus", &secs, material = M.cactus);
}

/// A closed notebook lying on `y`: its pages between two covers, a band round it.
#[allow(clippy::too_many_arguments)]
pub fn notebook(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    d: f64,
    t: f64,
    m: Mat,
    turn: f64,
) -> Obj {
    let parts = new_parts(|| {
        fbox!(
            F,
            format!("{name}_pages"),
            w - 0.006,
            t - 0.003,
            d - 0.006,
            (x + 0.002, y + t / 2.0, z),
            M.paper
        );
        for (k, yy) in [y + 0.0008, y + t - 0.0008].into_iter().enumerate() {
            fbox!(
                F,
                format!("{name}_cover{k}"),
                w,
                0.0016,
                d,
                (x, yy, z),
                m,
                bevel = 0.0006,
                bevel_segments = 1
            );
        }
        fbox!(
            F,
            format!("{name}_spine"),
            0.004,
            t,
            d,
            (x - w / 2.0 + 0.001, y + t / 2.0, z),
            m
        );
        fbox!(
            F,
            format!("{name}_band"),
            0.008,
            t + 0.002,
            d + 0.001,
            (x + w * 0.3, y + t / 2.0, z),
            M.pen
        );
    });
    turned(&parts, name, (x, y, z), 0.0, turn)
}

/// A pen lying on `y`, turned `turn` degrees: a barrel, a clip and its tip.
pub fn pen(name: &str, x: f64, y: f64, z: f64, turn: f64) {
    let (m, length) = (M.pen, 0.14);
    let (c, s) = (cos(radians(turn)), sin(radians(turn)));
    let a = (x - c * length / 2.0, y + 0.005, z + s * length / 2.0);
    let b = (x + c * length / 2.0, y + 0.005, z - s * length / 2.0);
    let along = |t: f64, lift: f64| {
        (
            a.0 + (b.0 - a.0) * t,
            a.1 + (b.1 - a.1) * t + lift,
            a.2 + (b.2 - a.2) * t,
        )
    };
    let mid = (
        a.0 + (b.0 - a.0) * 0.85,
        a.1 + (b.1 - a.1) * 0.85,
        a.2 + (b.2 - a.2) * 0.85,
    );
    fsweep!(
        F,
        name,
        &[a, mid, b],
        0.005,
        m,
        verts = 8,
        scale = [1.0, 1.0, 0.3]
    );
    // (the clip's points as the script adds them: 0 to x and depth, 5 mm up)
    let clip = [along(0.04, 0.005), along(0.35, 0.005)];
    let clip = clip.map(|(px, py, pz)| (px + 0.0, py, pz + 0.0));
    fsweep!(
        F,
        format!("{name}_clip"),
        &clip,
        0.0018,
        M.chrome,
        verts = 4
    );
}

/// A flat monitor on its stand, its screen facing `face` (-1: the back, where the desks' chairs sit; 1: the
/// front): a thin rounded bezel with a deeper chin, the screen and its picture, a rounded hump behind, a curved
/// neck down to a foot. Returns its parts (to join or turn).
#[allow(clippy::too_many_arguments)]
pub fn monitor<X: Coord>(
    name: &str,
    x: X,
    z: f64,
    w: f64,
    h: f64,
    cy: f64,
    body: Mat,
    screen: Mat,
    wall: Option<Mat>,
    face: f64,
    foot_y: f64,
    foot: (f64, f64),
) -> Vec<Obj> {
    new_parts(|| {
        fbox!(
            F,
            name,
            w,
            h,
            0.022,
            (x, cy, z),
            body,
            bevel = 0.008,
            bevel_segments = 1
        );
        let sz = z + face * 0.011;
        fbox!(
            F,
            format!("{name}_screen"),
            w - 0.022,
            h - 0.04,
            0.002,
            (x, cy + 0.009, sz),
            screen
        );
        if let Some(wall) = wall {
            fbox!(
                F,
                format!("{name}_wallpaper"),
                w - 0.1,
                h - 0.1,
                0.001,
                (x, cy + 0.012, sz + face * 0.0012),
                wall
            );
        }
        fbox!(
            F,
            format!("{name}_logo"),
            0.03,
            0.004,
            0.001,
            (x, cy - h / 2.0 + 0.01, sz),
            M.chrome
        );
        fbox!(
            F,
            format!("{name}_back"),
            w * 0.5,
            h * 0.55,
            0.03,
            (x, cy - h * 0.05, z - face * 0.02),
            body,
            bevel = 0.012,
            bevel_segments = 2
        );
        fsweep!(
            F,
            format!("{name}_neck"),
            &[
                (x, cy - h * 0.05, z - face * 0.032),
                (x, (cy + foot_y) / 2.0 - 0.03, z - face * 0.04),
                (x, foot_y + 0.01, z - face * 0.01)
            ],
            0.0,
            body,
            profile = rounded_rect(0.016, 0.05, 0.006, 1),
            up = (0, 1, 0),
            resolution = 3,
            angle = 60
        );
        fbox!(
            F,
            format!("{name}_foot"),
            foot.0,
            0.012,
            foot.1,
            (x, foot_y + 0.006, z + face * 0.025),
            body,
            bevel = 0.005,
            bevel_segments = 1
        );
    })
}

/// A keyboard lying on `y`: a low rounded case, four rows of keys and a row with the space bar towards the
/// typist (`face`: -1 the back, 1 the front), the caps in `keys`.
#[allow(clippy::too_many_arguments)]
pub fn keyboard<X: Coord>(
    name: &str,
    x: X,
    y: f64,
    z: f64,
    w: f64,
    d: f64,
    body: Mat,
    keys: Mat,
    face: f64,
    cols: i32,
) {
    fbox!(
        F,
        name,
        w,
        0.014,
        d,
        (x, y + 0.007, z),
        body,
        bevel = 0.005,
        bevel_segments = 1
    );
    let (pw, pd) = ((w - 0.024) / f64::from(cols), (d - 0.024) / 5.0);
    let top = y + 0.014;
    let x = x.f();
    for r in 0..5 {
        let kz = z + face * (d / 2.0 - 0.012 - pd * (f64::from(r) + 0.5));
        if r == 0 {
            // the space bar between modifier keys
            for (k, (a, b)) in [
                (0, 3),
                (3, cols - 4),
                (cols - 4, cols - 2),
                (cols - 2, cols),
            ]
            .into_iter()
            .enumerate()
            {
                fbox!(
                    F,
                    format!("{name}_key{r}{k}"),
                    pw * f64::from(b - a) - 0.003,
                    0.006,
                    pd - 0.003,
                    (
                        x - w / 2.0 + 0.012 + pw * f64::from(a + b) / 2.0,
                        top + 0.002,
                        kz
                    ),
                    keys
                );
            }
            continue;
        }
        for c in 0..cols {
            fbox!(
                F,
                format!("{name}_key{r}{c}"),
                pw - 0.003,
                0.006,
                pd - 0.003,
                (
                    x - w / 2.0 + 0.012 + pw * (f64::from(c) + 0.5),
                    top + 0.002,
                    kz
                ),
                keys
            );
        }
    }
}

/// A mouse on `y`, its buttons away from the hand (`face` the hand's side).
pub fn mouse(name: &str, x: f64, y: f64, z: f64, m: Mat, face: f64) {
    fsphere!(
        F,
        name,
        0.03,
        (x, y + 0.002, z),
        m,
        scale = (1.0, 1.75, 0.66),
        segments = 12,
        rings = 8
    );
    fbox!(
        F,
        format!("{name}_split"),
        0.0015,
        0.004,
        0.03,
        (x, y + 0.021, z - face * 0.022),
        M.pad
    );
}

/// A laptop's keys in their well: six rows of flat caps.
pub fn laptop_keys(x: f64, y: f64, z: f64) {
    fbox!(
        F,
        "laptop_well",
        0.29,
        0.0006,
        0.105,
        (x, y + 0.0001, z),
        M.pad
    );
    for r in 0..6 {
        let n = if r != 0 { 13 } else { 9 };
        let pw = 0.276 / f64::from(n);
        for c in 0..n {
            if r == 0 && c == 4 {
                continue;
            }
            fbox!(
                F,
                format!("laptop_key{r}{c}"),
                pw - 0.0035,
                0.0012,
                0.0125,
                (
                    x - 0.138 + pw * (f64::from(c) + 0.5),
                    y + 0.0006,
                    z - 0.045 + 0.0168 * f64::from(r)
                ),
                M.keycap
            );
        }
    }
    fbox!(
        F,
        "laptop_space",
        0.276 / 9.0 * 1.0 + 0.06,
        0.0012,
        0.0125,
        (x - 0.138 + 0.276 / 9.0 * 4.5, y + 0.0006, z - 0.045),
        M.keycap
    );
}

/// An open laptop on `y` at (x, z), its typist to the front (+depth) turned `turn` degrees: the base with its
/// keys and touchpad, the lid up at 110 degrees on its hinge.
pub fn laptop_open(x: f64, z: f64, y: f64, turn: f64) {
    let parts = new_parts(|| {
        fbox!(
            F,
            "laptop",
            0.32,
            0.012,
            0.22,
            (x, y + 0.006, z),
            M.aluminium,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "laptop_well",
            0.29,
            0.0006,
            0.105,
            (x, y + 0.0121, z - 0.04),
            M.pad
        );
        for r in 0..5 {
            fbox!(
                F,
                format!("laptop_keys{r}"),
                0.27,
                0.0012,
                0.013,
                (x, y + 0.0126, z - 0.08 + 0.0168 * f64::from(r)),
                M.keycap
            );
        }
        fbox!(
            F,
            "laptop_pad",
            0.1,
            0.0006,
            0.06,
            (x, y + 0.0123, z + 0.06),
            M.pad
        );
        let lid = new_parts(|| {
            fbox!(
                F,
                "lid",
                0.32,
                0.007,
                0.22,
                (x, y + 0.0165, z),
                M.aluminium,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                F,
                "lid_screen",
                0.29,
                0.0008,
                0.18,
                (x, y + 0.0127, z + 0.005),
                M.screen
            );
        });
        turned(&lid, "laptop_lid", (x, y + 0.013, z - 0.11), -110.0, 0.0);
    });
    turned(&parts, "laptop", (x, y, z), 0.0, turn);
}
