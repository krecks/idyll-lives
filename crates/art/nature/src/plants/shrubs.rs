//! G3, the shrubs: the round shrub and its small and flowering cousins, the street's and the garden's hedges, the
//! boxwood ball, the topiary cone, the hydrangea, the rose bush and the azalea.

use artkit::botany::{SHRUB, ShrubMass, fit_to, hedge_top, palette};
use artkit::far::{is_low, low};
use artkit::materials::{Mat, Palette};
use artkit::math::{P3, TAU, V3, asin, atan2, cos, degrees, sin, v3};
use artkit::parts::{join_all, new_parts, settle};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{
    _with_part, florets, hedge_block, jitter, leafy_shrub, loft, lump, on_masses, perches, rays,
    ring, rose_head, rose_leaf, spiral, sprigs, tube, twig, vessel,
};

/// A shrub's leaf masses' colours, darkest first, by name in the palette.
fn greens(p: &Palette, names: [&str; 5]) -> Vec<Mat> {
    names.iter().map(|name| p.get(name)).collect()
}

/// Round shrub: woody stems parting from the ground into broken leaf masses, lighter on top, leaves breaking its
/// outline.
pub fn bush() -> Vec<Obj> {
    let p = palette(&[]);
    leafy_shrub!(
        &p,
        31,
        1.043,
        0.91,
        &greens(&p, ["leaf_dark", "leaf_dark", "leaf", "stem", "leaf_light"]),
        p.get("twig"),
        depth = 1.095,
        masses = 12,
        sats = 1,
        leaves = 80
    );
    let still = join_all("bush");
    perches!(6, 131, &[(0.0, 0.0, 0.45)]);
    settle(&[still])
}

/// Small shrub: the round shrub's smaller, darker cousin (the lot borders and the woods' undergrowth).
pub fn bush_small() -> Vec<Obj> {
    let p = palette(&[]);
    leafy_shrub!(
        &p,
        32,
        0.832,
        0.707,
        &greens(&p, ["leaf_dark", "leaf_dark", "leaf", "leaf", "leaf_light"]),
        p.get("twig"),
        name = "bush_small",
        depth = 0.782,
        stems = 4,
        masses = 9,
        sats = 1,
        leaves = 56,
        leaf_size = 0.12
    );
    let still = join_all("bushSmall");
    perches!(5, 132, &[(0.0, 0.0, 0.35)], per = 24);
    settle(&[still])
}

/// The heading and the lean of a flower facing along `d`, degrees.
fn facing(d: V3) -> (f64, f64) {
    (
        90.0 - degrees(atan2(d.y as f64, d.x as f64)),
        90.0 - degrees(asin((-1.0_f64).max(1.0_f64.min(d.z as f64)))),
    )
}

/// Flowering shrub: the round shrub hung with clusters of pink blossom, five-petalled flowers in small sprays (the
/// lot borders).
pub fn bush_flowering() -> Vec<Obj> {
    let p = palette(&[
        ("blossom", "#F08CBC"),
        ("blossom2", "#F8B4D4"),
        ("blossom3", "#FFF0F6"),
        ("blossom4", "#E870A8"),
    ]);
    let cols = [
        p.get("blossom"),
        p.get("blossom2"),
        p.get("blossom3"),
        p.get("blossom4"),
    ];
    let blooms = |masses: &[ShrubMass], rng: &mut Random| {
        for (i, (at, d, _)) in on_masses!(masses, rng, 26, up = (-0.3, 1.2))
            .into_iter()
            .enumerate()
        {
            let (yaw, tilt) = facing(d);
            for j in 0..3 {
                let off = v3(
                    rng.uniform(-1.0, 1.0),
                    rng.uniform(-1.0, 1.0),
                    rng.uniform(-0.5, 0.5),
                ) * 0.05;
                if is_low() && j != 0 {
                    // a far version: one flower a spray
                    continue;
                }
                rays!(
                    format!("blossom{i}_{j}"),
                    at + off,
                    0.045,
                    5,
                    cols[(i + j) % 4],
                    yaw = yaw,
                    tilt = tilt * 0.8,
                    inner = 0.35,
                    cup = 0.25,
                    width = 0.95,
                    centre = 0.006
                );
            }
        }
    };
    leafy_shrub!(
        &p,
        33,
        1.208,
        1.016,
        &greens(&p, ["leaf_dark", "leaf_dark", "leaf", "stem", "leaf_light"]),
        p.get("twig"),
        name = "bush_flowering",
        depth = 1.268,
        masses = 10,
        sats = 1,
        leaves = 8,
        blooms = blooms
    );
    let still = join_all("bushFlowering");
    perches!(6, 133, &[(0.0, 0.0, 0.5)]);
    settle(&[still])
}

/// Clipped hedge segment of the streets and hedged lots: the garden hedge's block, still (rows overlap, street.ts
/// spaces them about 1.05 m apart).
pub fn street_hedge() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", SHRUB[1]),
        ("leaf_light", SHRUB[2]),
        ("stem", SHRUB[3]),
    ]);
    // 1.55 x 1.0 x 0.85 m over all, as its old part
    hedge_block!(&p, 1.547, 1.019, 0.85, seed = 134);
    let still = join_all("hedge");
    perches!(6, 134, &hedge_top(0.915), per = 6, up = (0.6, 1.4));
    settle(&[still])
}

/// Boxwood ball: a clipped ball of small leaves, a little lumpy where the shears went round, on a few stems; a few
/// shaggy sprigs on top, which a trim takes off.
pub fn boxwood() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", "#3E6B33"),
        ("leaf_dark", "#365F2D"),
        ("leaf_light", "#4C7A3A"),
    ]);
    let made = new_parts(|| {
        // the stems under it
        for i in 0..4 {
            let a = i as f64 * TAU / 4.0 + 0.4;
            twig!(
                format!("stem{i}"),
                &[
                    (0.03 * cos(a), 0.03 * sin(a), 0.0),
                    (0.04 * cos(a), 0.04 * sin(a), 0.05),
                    (0.12 * cos(a), 0.12 * sin(a), 0.16)
                ],
                0.016,
                0.01,
                p.get("twig"),
                verts = 5
            );
        }
        let ball = florets!(
            "ball",
            0.36,
            (0, 0, 0.04),
            p.get("leaf"),
            bumps = 5.0,
            amount = 0.035,
            flat = 0.94,
            segments = low(22, 12),
            rings = low(14, 8),
            seed = 1.0
        );
        // where the shears went round
        jitter!(ball, 0.03, size = 0.3, seed = 2.0);
        florets!(
            "ball2",
            0.16,
            (0.1, 0.1, 0.46),
            p.get("leaf_light"),
            bumps = 4.0,
            amount = 0.06,
            flat = 0.7,
            segments = 10,
            rings = 6,
            seed = 3.0
        );
        florets!(
            "ball3",
            0.14,
            (-0.18, -0.12, 0.36),
            p.get("leaf_dark"),
            bumps = 4.0,
            amount = 0.06,
            flat = 0.75,
            segments = 10,
            rings = 6,
            seed = 4.0
        );
    });
    fit_to(&made, 0.69, 0.71, 0.72);
    let still = join_all("boxwood");
    perches!(4, 135, &[(0.0, 0.0, 0.34)]);
    _with_part!(
        "sprigs",
        still,
        || sprigs!("sprig", &p, (0.02, -0.03, 0.66), 0.12),
        (0.02, -0.03, 0.66),
        "trim"
    )
}

/// Topiary cone: a stone pot with a rolled rim on a foot, a short bare trunk out of mossy soil and a clipped cone
/// of small leaves, a little lumpy, in two greens; shaggy sprigs near its tip, which a trim takes off.
pub fn topiary_cone() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#B9B2A6"),
        ("leaf", "#3E6B33"),
        ("leaf_light", "#4A7838"),
        ("stalk", "#8A7058"),
        ("soil", "#4E4030"),
    ]);
    let top = vessel!(
        &p,
        0.24,
        0.36,
        r_bottom = 0.18,
        lip = 0.018,
        foot = 0.03,
        segments = 20
    );
    // plain: three draw calls
    tube!(
        "trunk",
        &[
            (0.0, 0.0, top - 0.02),
            (0.005, 0.0, top + 0.12),
            (0.0, 0.0, top + 0.2)
        ],
        0.03,
        0.024,
        p.get("stalk"),
        verts = 6
    );
    let n = low(26, 14);
    let mut secs: Vec<Vec<P3>> = Vec::new();
    for (z, r) in [
        (0.18, 0.05),
        (0.2, 0.19),
        (0.27, 0.285),
        (0.42, 0.29),
        (0.68, 0.25),
        (0.96, 0.18),
        (1.2, 0.11),
        (1.38, 0.045),
        (1.46, 0.0_f64),
    ] {
        secs.push(ring!(r.max(0.004), r.max(0.004), z = top + z, verts = n));
    }
    let crown = new_parts(|| {
        let cone = loft!("cone", &secs, material = p.get("leaf"));
        if !is_low() {
            jitter!(cone, 0.022, size = 0.16, seed = 5.0);
        }
    });
    // lighter tufts where it grew back since
    for i in 0..low(14, 0) {
        let (a, z) = (i as f64 * 2.4, 0.32 + (i % 7) as f64 * 0.15);
        let r = 0.285 - (z - 0.32) * 0.22;
        florets!(
            format!("tuft{i}"),
            0.05,
            (r * cos(a), r * sin(a), top + z),
            p.get("leaf_light"),
            bumps = 3.0,
            amount = 0.1,
            flat = 0.6,
            segments = 6,
            rings = 4,
            seed = i as f64
        );
    }
    fit_to(&crown, 0.62, None::<f64>, 0.65);
    let still = join_all("topiaryCone");
    perches!(
        4,
        136,
        &[
            (0.0, 0.0, top + 0.45),
            (0.0, 0.0, top + 0.75),
            (0.0, 0.0, top + 1.05)
        ],
        up = (0.15, 0.7),
        facing = 0.1
    );
    // sprigs reach out of the cone's sides there
    let tip = (0.0, 0.0, top + 1.36);
    _with_part!(
        "sprigs",
        still,
        || sprigs!("sprig", &p, tip, 0.1, n = 5, length = 0.12),
        tip,
        "trim"
    )
}

/// Clipped hedge: a squared-off block of small leaves with a rounded top on bare stems; shaggy sprigs along it,
/// which a trim takes off.
pub fn garden_hedge() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", SHRUB[1]),
        ("leaf_light", SHRUB[2]),
        ("stem", SHRUB[3]),
    ]);
    hedge_block!(&p, 1.546, 1.12, 0.92, seed = 137);
    let still = join_all("hedge");
    perches!(6, 137, &hedge_top(1.12), per = 6, up = (0.6, 1.4));
    let at = (0.1, 0.0, 1.06);
    _with_part!(
        "sprigs",
        still,
        || sprigs!("sprig", &p, at, 0.4, n = 7),
        at,
        "trim"
    )
}

/// Hydrangea: a leafy shrub of broad leaves on woody stems, covered in big blue mopheads of florets (all one
/// material: "Flowers" colours every one).
pub fn hydrangea() -> Vec<Obj> {
    let p = palette(&[("blossom", "#9CB8F4")]);
    let blossom = p.get("blossom");
    let blooms = |masses: &[ShrubMass], rng: &mut Random| {
        for (i, (at, d, _)) in on_masses!(masses, rng, 12, up = (0.0, 1.2))
            .into_iter()
            .enumerate()
        {
            let r = rng.uniform(0.11, 0.14);
            florets!(
                format!("mophead{i}"),
                r,
                at + d * r * 0.4 - v3(0, 0, r * 0.7),
                blossom,
                bumps = 3.2,
                amount = 0.16,
                flat = 0.82,
                segments = 10,
                rings = 7,
                seed = i as f64 * 1.3
            );
        }
    };
    leafy_shrub!(
        &p,
        72,
        1.196,
        0.957,
        &greens(&p, ["leaf_dark", "leaf_dark", "leaf", "stem", "leaf_light"]),
        p.get("twig"),
        name = "hydrangea",
        depth = 1.182,
        masses = 11,
        sats = 1,
        leaves = 48,
        leaf_size = 0.17,
        blooms = blooms
    );
    let still = join_all("hydrangea");
    perches!(5, 138, &[(0.0, 0.0, 0.45)]);
    settle(&[still])
}

/// Rose bush: an open shrub of upright, arching canes from a low leafy base, compound leaves of broad leaflets
/// along them, each cane tipped by an open red rose; the rose at the front, which a prune cuts.
pub fn rosebush() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", "#2C4C26"),
        ("leaf_dark", "#365A2C"),
        ("leaf_light", "#3F6632"),
        ("stem", "#4A7038"),
        ("bloom", "#C8203A"),
    ]);
    let mut rng = Random::new(73);
    let leafy = [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")];
    for (i, (x, y, _)) in spiral!(3, 0.16, &mut rng).into_iter().enumerate() {
        let seed = i as f64 * 1.3 + rng.random();
        lump!(
            format!("base{i}"),
            0.18,
            (x, y, 0),
            leafy[i % 3],
            scale = (1, 1, 0.7),
            seed = seed,
            noise = 0.32,
            segments = 8,
            rings = 5
        );
    }
    let mut canes = Vec::new();
    for (x, y, a) in spiral!(9, 0.3, &mut rng) {
        let h = rng.uniform(0.6, 0.84);
        let (foot, tip) = (v3(x * 0.3, y * 0.3, 0.05), v3(x * 1.3, y * 1.3, h));
        let bend = v3(x * 1.05, y * 1.05, h * 0.6);
        canes.push((foot, bend, tip, a, rng.uniform(8.0, 28.0)));
    }
    // the cane nearest the front (-y)
    let mut front = 0;
    for (i, cane) in canes.iter().enumerate() {
        if cane.2.y < canes[front].2.y {
            front = i;
        }
    }
    for (i, &(foot, bend, tip, a, tilt)) in canes.iter().enumerate() {
        twig!(
            format!("cane{i}"),
            &[foot, bend, tip - v3(0, 0, 0.02)],
            0.01,
            0.005,
            p.get("stem"),
            verts = 4
        );
        // leaves along the cane, turning round it
        for (k, f) in [0.35, 0.62, 0.86].into_iter().enumerate() {
            let at = if f < 0.6 {
                foot.lerp(bend, f / 0.6)
            } else {
                bend.lerp(tip, (f - 0.6) / 0.4)
            };
            let heading = degrees(a) + (k as f64 * 2.0 - 1.0) * 70.0 + rng.uniform(-20.0, 20.0);
            let tilt_k = rng.uniform(5.0, 30.0);
            if is_low() && k == 1 {
                // a far version: the outer two
                continue;
            }
            rose_leaf!(
                format!("leaves{i}_{k}"),
                at,
                heading,
                tilt_k,
                0.13,
                leafy[(i + k) % 3]
            );
        }
        if i != front {
            rose_head!(
                &format!("rose{i}"),
                tip,
                0.05,
                p.get("bloom"),
                yaw = degrees(a),
                tilt = tilt
            );
        }
    }
    // Perches: on the canes (half way up their arch) and the leafy base.
    let bare: Vec<V3> = canes
        .iter()
        .map(|&(_, bend, tip, _, _)| bend.lerp(tip, 0.45) + v3(0, 0, 0.012))
        .collect();
    perches!(5, 140, &[(0.0, 0.0, 0.3)], per = 8, bare = bare);
    let (_, _, tip, a, tilt) = canes[front];
    _with_part!(
        "rose",
        join_all("rosebush"),
        || rose_head!(
            "cut_rose",
            tip,
            0.05,
            p.get("bloom"),
            yaw = degrees(a),
            tilt = tilt
        ),
        tip,
        "cut"
    )
}

/// Azalea: a wide, low mound of small dark leaves smothered in trusses of pink flowers, each a few open
/// five-petalled flowers (one material, the colour part), down its sides as well as on top; shaggy sprigs on top,
/// which a trim takes off.
pub fn azalea() -> Vec<Obj> {
    let p = palette(&[("blossom", "#F0549C")]);
    let blossom = p.get("blossom");
    let blooms = |masses: &[ShrubMass], rng: &mut Random| {
        for (i, (at, d, _)) in on_masses!(masses, rng, 34, up = (-0.35, 1.2))
            .into_iter()
            .enumerate()
        {
            let (yaw, tilt) = facing(d);
            for j in 0..2 {
                let off = v3(
                    rng.uniform(-1.0, 1.0),
                    rng.uniform(-1.0, 1.0),
                    rng.uniform(-0.4, 0.4),
                ) * 0.04;
                if is_low() && j != 0 {
                    // a far version: one flower a truss
                    continue;
                }
                rays!(
                    format!("flowers{i}_{j}"),
                    at + off,
                    0.06,
                    5,
                    blossom,
                    yaw = yaw + (j * 36) as f64,
                    tilt = tilt * 0.85,
                    inner = 0.3,
                    cup = 0.3,
                    width = 0.95,
                    centre = 0.008
                );
            }
        }
    };
    leafy_shrub!(
        &p,
        74,
        1.158,
        1.013,
        &greens(&p, ["leaf_dark", "leaf_dark", "leaf", "stem", "leaf_light"]),
        p.get("twig"),
        name = "azalea",
        depth = 1.152,
        masses = 11,
        sats = 1,
        leaves = 16,
        leaf_size = 0.1,
        blooms = blooms,
        band = (-0.6, 1.0)
    );
    let still = join_all("azalea");
    perches!(5, 139, &[(0.0, 0.0, 0.45)]);
    let at = (-0.05, 0.05, 0.94);
    _with_part!(
        "sprigs",
        still,
        || sprigs!("sprig", &p, at, 0.14),
        at,
        "trim"
    )
}
