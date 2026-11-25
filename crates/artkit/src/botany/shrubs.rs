//! Shrubs and hedges (families/plants.py's): a woody skeleton under broken leaf masses with leaves breaking its
//! outline (`leafy_shrub`), a clipped block (`hedge_block`), what is spread over them, their perches, and a plant
//! grown from random numbers fitted to its size (`fit_to`).

use crate::botany::flowers::{Florets, florets};
use crate::botany::leaf::{Leaf, leaf};
use crate::botany::lump::{Lump, lump};
use crate::botany::stems::{Twig, twig};
use crate::far::{is_low, low};
use crate::forms::{Jitter, Loft, Ring, jitter, loft, ring};
use crate::materials::{Mat, Palette};
use crate::math::{M4, P3, PI, TAU, V3, atan2, cos, degrees, sin, sqrt, v3};
use crate::motion::{self, Squash, squash};
use crate::parts::{join, join_all, measure, new_parts, settle};
use crate::perch::{self, Find};
use crate::py;
use crate::rand::Random;
use crate::scene::{Obj, meshes};
use crate::shapes::options;

/// The shrubs' greens, darkest first.
pub const SHRUB: [&str; 5] = ["#44703A", "#517E3E", "#5E8C44", "#6E9A4A", "#7EA852"];

/// A side's size as `fit_to` is asked for it: metres, or None (as it is).
pub trait Want {
    fn want(self) -> Option<f64>;
}

impl Want for f64 {
    fn want(self) -> Option<f64> {
        Some(self)
    }
}

impl Want for Option<f64> {
    fn want(self) -> Option<f64> {
        self
    }
}

/// Scales `parts` about the origin so that together they are `w` x `h` x `d` (game: width, height, depth): for a
/// plant grown from random numbers, whose exact reach only shows once it's built (None: as it is). Returns the scale
/// (Blender x, y, z).
pub fn fit_to(parts: &[Obj], w: impl Want, h: impl Want, d: impl Want) -> [f64; 3] {
    let (lo, hi) = measure(parts);
    let mut k = [1.0; 3];
    for (i, want) in [w.want(), d.want(), h.want()].into_iter().enumerate() {
        if let Some(want) = want {
            k[i] = want / (hi[i] - lo[i]);
        }
    }
    let scale = M4::diagonal(v3(k[0], k[1], k[2]));
    for &o in parts {
        assert!(
            o.read(|part| part.quat.is_none()),
            "fit_to: a part turned by a quaternion (a rod) isn't fitted"
        );
        // set as the part's world matrix, Blender takes it apart into a place, a turn and a scale
        let (loc, rot, size) = motion::decompose(&(scale * o.matrix_world()));
        o.edit(|part| (part.loc, part.rot, part.scale) = (loc, rot, size));
    }
    k
}

/// A shrub's leaf mass as its flowers find it: (its middle, its radius, how far it is squashed).
pub type ShrubMass = (V3, f64, f64);

/// What adds a shrub's flowers on its masses, with the shrub's own random numbers, before it is fitted to its size.
pub type Blooms<'a> = Box<dyn FnOnce(&[ShrubMass], &mut Random) + 'a>;

/// `leafy_shrub`'s options (`depth` None: as wide; `leaf_cols` None: the masses' colours but the darkest).
pub struct LeafyShrub<'a> {
    pub name: &'a str,
    pub stems: usize,
    pub masses: usize,
    pub sats: usize,
    pub leaves: usize,
    pub leaf_size: f64,
    pub band: (f64, f64),
    pub skirt: f64,
    pub depth: Option<f64>,
    pub mass_size: f64,
    pub out: f64,
    pub leaf_cols: Option<Vec<Mat>>,
    pub blooms: Option<Blooms<'a>>,
}

impl Default for LeafyShrub<'_> {
    fn default() -> Self {
        LeafyShrub {
            name: "bush",
            stems: 5,
            masses: 9,
            sats: 1,
            leaves: 36,
            leaf_size: 0.15,
            band: (-0.7, 1.0),
            skirt: 0.08,
            depth: None,
            mass_size: 0.36,
            out: 0.68,
            leaf_cols: None,
            blooms: None,
        }
    }
}

/// A shrub `w` wide (`depth` deep: `w` by default) and `h` tall (fitted to it once built), as the oak is a tree:
/// `stems` woody stems (`wood`) rise from the ground, part and fork out into the crown; `masses` leaf masses on an
/// ellipsoid shell round it (`out` of the crown's radii), each a clump (`mass_size` of the mean radius) with `sats`
/// smaller ones broken off it, lighter higher up (`cols`, darkest first); a dark heart fills the middle from
/// `skirt` x `h` up, so the stems show under it; and `leaves` leaves in sprays of four (`leaf_cols`) stick out of the
/// masses, breaking the outline; `blooms(masses, rng)` adds flowers on them (each mass (middle, radius, squash))
/// before it is fitted to its size. Its own random numbers (`seed`). Returns the masses (middle, radius) for flowers
/// and perches. A far version: the stems unforked and three-sided, the clumps coarser (and a little bigger, for the
/// leaves' reach), no single leaves (their random numbers still drawn).
pub fn leafy_shrub(
    seed: i128,
    w: f64,
    h: f64,
    cols: &[Mat],
    wood: Mat,
    o: LeafyShrub,
) -> Vec<(V3, f64)> {
    let (name, band, skirt) = (o.name, o.band, o.skirt);
    let d = match o.depth {
        Some(depth) if depth != 0.0 => depth,
        _ => w,
    };
    let mut masses_out: Vec<ShrubMass> = Vec::new();
    let made = new_parts(|| {
        let mut rng = Random::new(seed);
        let far = is_low();
        let (rx, ry, rz) = (w / 2.0, d / 2.0, h / 2.0);
        let cz = h / 2.0;
        let m = (rx + ry + rz) / 3.0;
        // the masses, round the shell top first (where they sit decides where the stems go)
        let ga = PI * (3.0 - sqrt(5.0));
        let off = rng.random() * TAU;
        let mut spots: Vec<(V3, f64, f64, f64)> = Vec::new();
        for i in 0..o.masses {
            let t = band.1 - (band.1 - band.0) * (i as f64 + 0.5) / o.masses as f64;
            let ring = sqrt(0.0_f64.max(1.0 - t * t));
            let a = off + ga * i as f64 + rng.uniform(-0.3, 0.3);
            let k = o.out * rng.uniform(0.9, 1.08);
            let s = o.mass_size * m * rng.uniform(0.85, 1.15);
            let at = v3(
                rx * ring * cos(a) * k,
                ry * ring * sin(a) * k,
                cz + rz * t * k,
            );
            spots.push((at, s, t, a));
        }
        // the heart: dark, filling the middle
        let heart_seed = rng.random() * 9.0;
        lump(
            format!("{name}_heart"),
            m * 0.66,
            (0.0, 0.0, h * skirt),
            cols[0],
            Lump {
                scale: P3 {
                    x: rx / m,
                    y: ry / m,
                    z: (h * (0.86 - skirt)) / (m * 1.32),
                },
                noise: 0.25,
                seed: heart_seed,
                segments: low(9, 7),
                rings: low(6, 4),
            },
        );
        // the stems: from the ground, upright at first, parting towards the masses; each forks under two of them
        for i in 0..o.stems {
            let a = off + TAU * (i as f64 + rng.uniform(-0.2, 0.2)) / o.stems as f64;
            let foot = v3(m * 0.08 * cos(a), m * 0.08 * sin(a), 0.0);
            let reach = |sp: &(V3, f64, f64, f64)| {
                let to = v3(cos(a), sin(a), 0) * m;
                v3(sp.0.x - to.x, sp.0.y - to.y, 0).xy_length()
            };
            let mut near: Vec<&(V3, f64, f64, f64)> = spots.iter().collect();
            near.sort_by(|a, b| reach(a).total_cmp(&reach(b)));
            let mid = foot.lerp(near[0].0, 0.45) + v3(0, 0, h * 0.05);
            let r0 = m * rng.uniform(0.028, 0.036);
            twig(
                format!("{name}_stem{i}"),
                &[foot, foot + v3(0, 0, h * 0.08), mid, near[0].0],
                r0,
                r0 * 0.45,
                wood,
                Twig {
                    verts: 4,
                    ..Twig::default()
                },
            );
            let fork = near[1].0.lerp(near[0].0, 0.25);
            if !far {
                twig(
                    format!("{name}_fork{i}"),
                    &[mid, mid.lerp(fork, 0.5) + v3(0, 0, h * 0.04), fork],
                    r0 * 0.6,
                    r0 * 0.3,
                    wood,
                    Twig { verts: 3, res: 1 },
                );
            }
        }
        // the masses
        let top = cols.len() as i64 - 1;
        for (i, &(at, s, t, a)) in spots.iter().enumerate() {
            let light = (t - band.0) / (band.1 - band.0);
            let shade = py::round(1.0 + light * (cols.len() as f64 - 2.0) + rng.uniform(-0.5, 0.5));
            let col = cols[1.max(top.min(shade)) as usize];
            let (sq, seed_i) = (rng.uniform(0.66, 0.8), i as f64 * 1.9 + rng.random());
            lump(
                format!("{name}_mass{i}"),
                s * low(1.0, 1.08),
                (at.x as f64, at.y as f64, at.z as f64 - s * sq),
                col,
                Lump {
                    scale: P3 {
                        x: 1.0,
                        y: 1.0,
                        z: sq,
                    },
                    noise: 0.3,
                    seed: seed_i,
                    segments: 8,
                    rings: 5,
                },
            );
            masses_out.push((at, s, sq));
            for j in 0..o.sats {
                let b = a + rng.uniform(-1.3, 1.3);
                let e = rng.uniform(-0.4, 0.7);
                let ss = s * rng.uniform(0.55, 0.72);
                let p = at + v3(cos(b) * cos(e), sin(b) * cos(e), sin(e)) * s * 0.85;
                let own = cols.iter().position(|c| *c == col).unwrap_or(0) as i64;
                let c2 = cols[1.max(top.min(own + rng.choice(&[-1, 0, 1]))) as usize];
                let sd = i as f64 * 3.1 + j as f64 + rng.random();
                lump(
                    format!("{name}_sat{i}_{j}"),
                    ss * low(1.0, 1.08),
                    (p.x as f64, p.y as f64, p.z as f64 - ss * sq),
                    c2,
                    Lump {
                        scale: P3 {
                            x: 1.0,
                            y: 1.0,
                            z: sq,
                        },
                        noise: 0.32,
                        seed: sd,
                        segments: 6,
                        rings: 4,
                    },
                );
            }
        }
        // sprays of four leaves breaking the outline: out of a mass's outer side, heading away from the middle
        let lc: Vec<Mat> = match &o.leaf_cols {
            Some(given) if !given.is_empty() => given.clone(),
            _ => cols[1..].to_vec(),
        };
        for i in 0..o.leaves / 4 {
            let (at, s, sq) = masses_out[rng.randrange(masses_out.len() as i64) as usize];
            let heading = atan2(at.y as f64, at.x as f64) + rng.uniform(-0.7, 0.7);
            let e = rng.uniform(-0.1, 0.9);
            let dirn = v3(cos(heading) * cos(e), sin(heading) * cos(e), sin(e));
            let base = at + v3(dirn.x, dirn.y, dirn.z as f64 * sq) * s * rng.uniform(0.92, 1.05);
            let yaw = 90.0 - degrees(heading);
            let tilt = degrees(e) * 0.6 + rng.uniform(0.0, 30.0);
            let size = o.leaf_size * rng.uniform(0.8, 1.2);
            if far {
                continue;
            }
            for j in 0..4 {
                let inner = j == 1 || j == 2;
                leaf(
                    format!("{name}_leaf{i}_{j}"),
                    base,
                    yaw + (j as f64 - 1.5) * 32.0,
                    tilt + if inner { 10.0 } else { -8.0 },
                    size * if inner { 1.1 } else { 0.85 },
                    size * 0.55,
                    lc[(i + j) % lc.len()],
                    Leaf {
                        shape: "oval",
                        droop: 0.3,
                        thick: 0.004,
                        fold: 0.3,
                        rows: 2,
                        ring: 3,
                        ..Leaf::default()
                    },
                );
            }
        }
        if let Some(blooms) = o.blooms {
            blooms(&masses_out, &mut rng);
        }
    });
    let k = fit_to(&made, w, h, d);
    masses_out
        .iter()
        .map(|&(p, r, _)| {
            (
                v3(p.x as f64 * k[0], p.y as f64 * k[1], p.z as f64 * k[2]),
                r * py::sum(k) / 3.0,
            )
        })
        .collect()
}

options! {
    /// `on_masses`' options.
    OnMasses { up: (f64, f64) = (0.1, 1.0) }
}

/// `n` spots on a shrub's masses (`leafy_shrub`), on their outer, upper sides: (point, the outward direction,
/// the mass's radius), drawing the same random numbers every build.
pub fn on_masses(
    masses: &[ShrubMass],
    rng: &mut Random,
    n: usize,
    o: OnMasses,
) -> Vec<(V3, V3, f64)> {
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let (at, s, sq) = if i < masses.len() {
            masses[i % masses.len()]
        } else {
            masses[rng.randrange(masses.len() as i64) as usize]
        };
        let heading = atan2(at.y as f64, at.x as f64) + rng.uniform(-0.9, 0.9);
        let e = rng.uniform(o.up.0, o.up.1);
        let d = v3(cos(heading) * cos(e), sin(heading) * cos(e), sin(e));
        out.push((at + v3(d.x, d.y, d.z as f64 * sq) * s * 1.1, d, s));
    }
    out
}

options! {
    /// `dots`' options.
    Dots { band: (f64, f64) = (-0.3, 0.95), out: f64 = 1.0, flat: f64 = 0.8, segments: usize = 8, rings: usize = 5, far_every: usize = 1 }
}

/// Flowers spread over a shrub's crown (an ellipsoid at `centre`, `radii`), on its upper part; a far version builds
/// every `far_every`th (drawing every random number).
#[allow(clippy::too_many_arguments)]
pub fn dots(
    rng: &mut Random,
    n: usize,
    centre: (f64, f64, f64),
    radii: (f64, f64, f64),
    size: f64,
    materials: &[Mat],
    name: &str,
    o: Dots,
) {
    let (band, out) = (o.band, o.out);
    let (cx, cy, cz) = centre;
    let ga = PI * (3.0 - sqrt(5.0));
    for i in 0..n {
        let t = band.0 + (band.1 - band.0) * (i as f64 + 0.5) / n as f64; // height on the ellipsoid, -1 .. 1
        let a = i as f64 * ga + rng.uniform(-0.3, 0.3);
        let ring = sqrt(0.0_f64.max(1.0 - t * t));
        let at = (
            cx + radii.0 * ring * cos(a) * out,
            cy + radii.1 * ring * sin(a) * out,
            cz + radii.2 * t * out - size / 2.0,
        );
        if is_low() && i % o.far_every != 0 {
            continue;
        }
        lump(
            format!("{name}{i}"),
            size / 2.0,
            at,
            materials[i % materials.len()],
            Lump {
                scale: P3 {
                    x: 1.0,
                    y: 1.0,
                    z: o.flat,
                },
                seed: i as f64 * 0.77,
                segments: o.segments,
                rings: o.rings,
                ..Lump::default()
            },
        );
    }
}

options! {
    /// `perches`' options.
    Perches { per: usize = 14, up: (f64, f64) = (0.35, 1.25), inset: f64 = 0.02, gap: f64 = 0.2, facing: f64 = 0.2, bare: Vec<V3> = Vec::new() }
}

/// A shrub's or a hedge's perches (the entry's `perches`, perch.rs), found on it as it stands: where `per` lines
/// in from round and above towards each of `centres` meet its leaves, `inset` metres in, facing up (their normal's z
/// over `facing`), with nothing over them, spread over it at least `gap` apart; `bare` adds points on bare stems that
/// stand in the open (a rose's canes). Up to `n`. Its own random numbers (`seed`), so the shrub's are as they
/// were; nothing on a far version (`perch::find`).
pub fn perches<C: Into<V3> + Copy>(n: usize, seed: i128, centres: &[C], o: Perches) {
    let mut rng = Random::new(seed);
    let mut aims = Vec::new();
    for &c in centres {
        aims.extend(perch::around(c, &mut rng, o.per, o.up, None, PI));
    }
    perch::find(
        n,
        &aims,
        Find {
            inset: o.inset,
            gap: o.gap,
            facing: o.facing,
            reach: 4.0,
            headroom: 0.15,
            limbs: o.bare,
            limbs_headroom: 0.15,
            ..Find::default()
        },
    );
}

/// As `_with_part`, for a model fitted to its `size` (`fit_to`) with its moving part: everything built, fitted
/// together, then the part joined apart (origin at `origin`, as fitted), settled and shrunk away.
pub fn _fit_with_part(
    name: &str,
    still_name: &str,
    build: impl FnOnce(),
    origin: impl Into<P3>,
    clip: &str,
    size: (f64, f64, f64),
) -> Vec<Obj> {
    let part = new_parts(build);
    let k = fit_to(&meshes(), size.0, size.1, size.2);
    if is_low() {
        // a far version: all one still part
        return settle(&[join_all(still_name)]);
    }
    let rest: Vec<Obj> = meshes().into_iter().filter(|o| !part.contains(o)).collect();
    let still = join(&rest, still_name, Some(P3::ZERO));
    let origin: P3 = origin.into();
    let moving = join(
        &part,
        name,
        Some(P3 {
            x: origin.x * k[0],
            y: origin.y * k[1],
            z: origin.z * k[2],
        }),
    );
    settle(&[still, moving]);
    squash(
        moving,
        clip,
        0.001,
        Squash {
            seconds: 0.8,
            ..Squash::default()
        },
    );
    vec![still, moving]
}

/// Where a hedge segment's perch lines aim: along its top, a little under it.
pub fn hedge_top(h: f64) -> Vec<(f64, f64, f64)> {
    [-0.55, -0.18, 0.18, 0.55]
        .iter()
        .map(|&x| (x, 0.0, h * 0.55))
        .collect()
}

options! {
    /// `hedge_block`'s options.
    HedgeBlock { name: &'static str = "hedge", seed: i64 = 0 }
}

/// A clipped hedge `length` x `h` x `depth` (fitted to it): a block of small leaves battered in at the foot and
/// rounded over the top, lumpy where the shears went, on bare stems; tufts grown back since the last trim. In the
/// palette `p`'s `leaf`, `twig`, `leaf_light` and `stem`.
pub fn hedge_block(p: &Palette, length: f64, h: f64, depth: f64, o: HedgeBlock) {
    let (name, seed) = (o.name, o.seed);
    let mut rng = Random::new(seed as i128);
    let made = new_parts(|| {
        let n = low(44, 20);
        let mut secs = Vec::new();
        for (z, k) in [
            (0.06, 0.8),
            (0.13, 0.95),
            (0.4, 1.0),
            (0.78, 0.99),
            (0.92, 0.9),
            (0.98, 0.7),
            (1.0, 0.3),
        ] {
            secs.push(ring(
                length / 2.0 * (0.97 + 0.03 * k),
                depth / 2.0 * k,
                Ring {
                    z: h * z,
                    verts: n,
                    square: low(7.0, 5.0),
                    ..Ring::default()
                },
            ));
        }
        let block = loft(
            format!("{name}_block"),
            &secs,
            Loft {
                material: Some(p.get("leaf")),
                ..Loft::default()
            },
        );
        jitter(
            block,
            0.03,
            Jitter {
                size: 0.22,
                seed: seed as f64 * 0.01 + 1.0,
                ..Jitter::default()
            },
        );
        // (Mounds a hand across to an arm's length: seen from above its top is not a ruled strip.)
        jitter(
            block,
            0.075,
            Jitter {
                size: 0.5,
                seed: seed as f64 * 0.01 + 7.0,
                ..Jitter::default()
            },
        );
        for i in 0..7 {
            // stems under it
            let x = (i as f64 - 3.0) / 3.0 * length * 0.4 + rng.uniform(-0.05, 0.05);
            let y = rng.uniform(-0.12, 0.12) * depth;
            twig(
                format!("{name}_stem{i}"),
                &[(x, y, 0.0), (x, y, 0.06), (x * 1.05, y * 1.6, h * 0.16)],
                0.017,
                0.012,
                p.get("twig"),
                Twig {
                    verts: 4,
                    ..Twig::default()
                },
            );
        }
        for i in 0..low(22, 0) {
            // tufts grown back, lighter, on the top and sides
            let x = rng.uniform(-0.45, 0.45) * length;
            let side = rng.choice(&[-1.0, 1.0]);
            let top = rng.random() < 0.5;
            let at = (
                x,
                side * depth * if top { 0.25 } else { 0.47 },
                h * if top { 0.99 } else { rng.uniform(0.35, 0.85) },
            );
            florets(
                format!("{name}_tuft{i}"),
                0.07,
                (at.0, at.1, at.2 - 0.04),
                [p.get("leaf_light"), p.get("stem")][i % 2],
                Florets {
                    bumps: 3.0,
                    amount: 0.12,
                    flat: 0.65,
                    segments: 6,
                    rings: 4,
                    seed: i as f64 * 0.7,
                },
            );
        }
    });
    fit_to(&made, length, h, depth);
}
