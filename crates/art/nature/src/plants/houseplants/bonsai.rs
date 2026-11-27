use artkit::botany::palette;
use artkit::far::{is_low, low};
use artkit::forms::by;
use artkit::math::{P3, cos, sin};
use artkit::parts::{join, join_all, new_parts, settle};
use artkit::scene::Obj;
use artkit::{boulder, cube, florets, jitter, leaf, loft, ring, squash, sweep, tube, twig};

/// Bonsai: a blue glazed tray with a rim and little feet, moss and two stones, on a wooden bonsai stand at bench
/// height (its top 0.76 m, a shelf below, rails under the top); a gnarled trunk on spreading surface roots, its
/// branches carrying cloud-pruned pads of foliage, and one stray sprig, which shrinks away while a resident prunes
/// it.
pub fn bonsai() -> Vec<Obj> {
    use artkit::json::float_repr as fr;
    // (the stand and the trunk plain, its pads still the foliage's dapple: three draw calls, not four)
    let p = super::super::plain_wood(palette(&[
        ("body", "#3E5A78"),
        ("wood", "#6A4A32"),
        ("bark", "#6E5A4C"),
        ("leaf", "#3A6A30"),
        ("leaf_dark", "#346028"),
        ("leaf_light", "#467A38"),
        ("stem", "#5E5040"),
        ("moss", "#5E7A34"),
    ]));
    let (wood, body) = (p.get("wood"), p.get("body"));
    let up = 0.38; // the stand's top over the old knee-high table's
    cube!(
        "table_top",
        (0.62, 0.4, 0.04),
        at = (0, 0, 0.36 + up),
        material = wood,
        bevel = low(0.008, 0.0)
    );
    for x in [-0.27, 0.27] {
        for y in [-0.16, 0.16] {
            cube!(
                format!("table_leg{}{}", fr(x), fr(y)),
                (0.045, 0.045, 0.34 + up),
                at = (x, y, (0.34 + up) / 2.0),
                material = wood,
                bevel = low(0.005, 0.0)
            );
        }
    }
    for y in [-0.16, 0.16] {
        cube!(
            format!("rail{}", fr(y)),
            (0.5, 0.022, 0.06),
            at = (0, y, 0.31 + up),
            material = wood
        );
    }
    for x in [-0.27, 0.27] {
        cube!(
            format!("rail_end{}", fr(x)),
            (0.022, 0.28, 0.06),
            at = (x, 0, 0.31 + up),
            material = wood
        );
    }
    cube!(
        "table_shelf",
        (0.56, 0.34, 0.025),
        at = (0, 0, 0.16),
        material = wood,
        bevel = low(0.005, 0.0)
    );
    let z0 = 0.38 + up;
    let secs: Vec<Vec<P3>> = [
        (0.012, 0.2, 0.125),
        (0.02, 0.215, 0.138),
        (0.062, 0.232, 0.152),
        // (its rim's arrises eased: no turn of 60°)
        (0.067, 0.232, 0.152),
        (0.07, 0.228, 0.148),
        (0.07, 0.222, 0.142),
        (0.067, 0.218, 0.138),
        (0.055, 0.215, 0.135),
    ]
    .iter()
    .map(|&(z, rx, ry)| ring!(rx, ry, z = z0 + z, verts = low(24, 12), square = 6))
    .collect();
    loft!("tray", &secs, material = body);
    for x in [-0.16, 0.16] {
        for y in [-0.09, 0.09] {
            cube!(
                format!("tray_foot{}{}", fr(x), fr(y)),
                (0.03, 0.03, 0.014),
                at = (x, y, z0 + 0.007),
                material = body
            );
        }
    }
    cube!(
        "moss",
        (0.42, 0.26, 0.02),
        at = (0, 0, z0 + 0.06),
        material = p.get("moss"),
        bevel = low(0.008, 0.0)
    );
    for (i, (x, y, r)) in [(0.13, -0.06, 0.03), (-0.15, 0.05, 0.022)]
        .into_iter()
        .enumerate()
    {
        boulder!(
            format!("stone{i}"),
            r,
            at = (x, y, z0 + 0.062),
            material = body,
            scale = (1.3, 1, 0.7),
            seed = i as f64 + 1.0,
            subdivisions = low(2, 1)
        );
    }
    // Bonsai trunks are wood here, not bark: one draw call fewer, and a trunk this short hardly bends in the wind.
    let trunk = sweep!(
        "trunk",
        &[
            (0.02, 0.0, 0.44 + up),
            (-0.05, 0.0, 0.58 + up),
            (0.04, 0.01, 0.7 + up),
            (-0.06, 0.0, 0.8 + up)
        ],
        0.034,
        material = wood,
        verts = low(9, 5),
        resolution = low(3, 1),
        scale = by(|w| 1.0 - 0.55 * w)
    );
    if !is_low() {
        jitter!(trunk, 0.006, size = 0.05, seed = 1.0);
    }
    // surface roots, spreading over the moss
    for i in 0..low(4, 0) {
        let a = i as f64 * 1.6 + 0.3;
        twig!(
            format!("root{i}"),
            &[
                (0.02, 0.0, 0.47 + up),
                (0.02 + 0.06 * cos(a), 0.05 * sin(a), 0.45 + up),
                (0.02 + 0.12 * cos(a), 0.08 * sin(a), 0.442 + up)
            ],
            0.014,
            0.004,
            wood,
            verts = 4
        );
    }
    let branches = [
        [
            (-0.05, 0.0, 0.58 + up),
            (0.08, 0.02, 0.64 + up),
            (0.18, 0.03, 0.68 + up),
        ],
        [
            (0.03, 0.01, 0.69 + up),
            (-0.12, -0.02, 0.73 + up),
            (-0.2, -0.03, 0.72 + up),
        ],
        [
            (0.0, 0.01, 0.66 + up),
            (0.03, 0.05, 0.71 + up),
            (0.03, 0.08, 0.73 + up),
        ],
        [
            (-0.05, 0.0, 0.79 + up),
            (-0.08, -0.02, 0.82 + up),
            (-0.1, 0.0, 0.84 + up),
        ],
    ];
    for (i, pts) in branches.iter().enumerate() {
        twig!(
            format!("branch{i}"),
            pts,
            if i < 2 { 0.015 } else { 0.01 },
            0.006,
            wood,
            verts = 5
        );
    }
    // cloud-pruned pads: each a few flattened clumps, lighter on top
    for (i, (x, y, z, r, sx, _sy)) in [
        (-0.07, 0.0, 0.83, 0.12, 1.4, 1.0),
        (0.17, 0.03, 0.7, 0.09, 1.3, 1.0),
        (-0.19, -0.03, 0.75, 0.08, 1.3, 0.9),
        (0.03, 0.07, 0.73, 0.07, 1.2, 1.0),
    ]
    .into_iter()
    .enumerate()
    {
        for j in 0..3 {
            let (dx, dy) = (
                (j as f64 - 1.0) * r * 0.55 * sx,
                (if j == 1 { 0.3 } else { -0.2 }) * r * 0.5,
            );
            florets!(
                format!("pad{i}_{j}"),
                r * if j != 1 { 0.75 } else { 0.85 },
                (
                    x + dx,
                    y + dy,
                    z + up - 0.035 + if j == 1 { 0.012 } else { 0.0 }
                ),
                [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")][(i + j) % 3],
                bumps = 3.5,
                amount = 0.12,
                flat = 0.45,
                segments = 10,
                rings = 6,
                seed = i as f64 * 1.9 + j as f64
            );
        }
    }
    let still = join_all("bonsai");
    // The stray sprig, sticking up out of the top pad: a twig and a tuft, all on foliage (one draw call).
    let parts = new_parts(|| {
        tube!(
            "sprig_twig",
            &[
                (-0.06, -0.03, 0.83 + up),
                (-0.11, -0.09, 0.88 + up),
                (-0.17, -0.15, 0.92 + up)
            ],
            0.006,
            0.004,
            p.get("stem"),
            verts = 4
        );
        for (i, (yaw, tilt)) in [
            (200.0, 20.0),
            (260.0, 35.0),
            (320.0, 15.0),
            (230.0, 65.0),
            (150.0, 30.0),
        ]
        .into_iter()
        .enumerate()
        {
            leaf!(
                format!("sprig_leaf{i}"),
                (-0.17, -0.15, 0.915 + up),
                yaw,
                tilt,
                0.065,
                0.04,
                p.get("leaf_light"),
                shape = "oval",
                rows = 3,
                ring = 4
            );
        }
    });
    if is_low() {
        // a far version: all one still part
        return settle(&[join_all("bonsai")]);
    }
    let sprig = join(&parts, "sprig", Some(P3::from((-0.06, -0.03, 0.83 + up))));
    settle(&[still, sprig]);
    squash!(sprig, "trim", 0.001, seconds = 0.8);
    vec![still, sprig]
}
