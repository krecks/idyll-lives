//! A broadleaf's skeleton, as every tree but the oak grows it (the oak first, by hand): a swept, tapering trunk on
//! root buttresses, limbs found from where the crown's leaf masses are (each limb runs to the middle of the masses
//! on its side and forks into a bough to each), a mass of clumps at each bough's tip. Every random number is drawn
//! the same at low detail, so a far version's masses stand where the model's do. With them: the trees' palette,
//! the lines their perches are found along, and the fruit hung on a crown.

use crate::botany::{CARD_OF, Clad, Tiles, clad};
use crate::bvh::Rays;
use crate::far::{is_low, low};
use crate::materials::{Mat, Palette, hex_of, rgb_of};
use crate::math::{self, P3, PI, TAU, V3, atan2, cos, sin, sqrt, v3};
use crate::motion::{Squash, Stages, squash, stages};
use crate::parts::{join, join_all, new_parts, settle};
use crate::perch::{self, Aim};
use crate::py;
use crate::rand::Random;
use crate::scene::Obj;
use crate::shapes::options;

/// The five leaf shades' names, darkest first.
pub const LEAVES: [&str; 5] = ["leaf", "leaf2", "leaf3", "leaf4", "leaf5"];
/// A card's colour, of its leaf colour's: the atlas is lighter than the tiling leaves the far version's lumps wear
/// and its cards catch more sky, so a tree's cards are toned down until it matches its far version where the near
/// woods meet the far (measured in the gallery, `?far` beside it, from 110 m).
pub const CARD_TONE: f64 = 0.85;
/// The core's colour, of the darkest leaf shade's: the inside of a crown.
pub const CORE_TONE: f64 = 0.6;

/// The trees' materials, with their finishes; each tree takes its own colours (`palette`).
pub fn base() -> Palette {
    let mut colours = vec![
        Mat::new("bark", "#E4D8C8", "bark"), // the bark texture is dark: its colours are pale
        Mat::new("birch_bark", "#ECE8DF", "matte"),
        Mat::new("birch_mark", "#36322E", "matte"),
        Mat::new("leaf", "#558238", "foliage"), // five shades, darkest first (the foliage texture is pale too)
        Mat::new("leaf2", "#64923E", "foliage"),
        Mat::new("leaf3", "#76A246", "foliage"),
        Mat::new("leaf4", "#8AB250", "foliage"),
        Mat::new("leaf5", "#9EC25C", "foliage"),
        Mat::new("leaf_light", "#8AB250", "foliage"), // sprigs (botany's sprigs)
        Mat::new("green", "#6E9A44", "foliage"),      // green clumps among blossom
        Mat::new("green2", "#62903E", "foliage"),
        Mat::new("fruit", "#C8352B", "satin"), // apples, lemons, blooms
        Mat::new("fruit2", "#D8402C", "satin"),
        Mat::new("fruit3", "#B42A26", "satin"),
        Mat::new("fruit4", "#E2B03C", "satin"),
        Mat::new("body", "#C0683F", "satin"), // the lemon tree's pot
        Mat::new("soil", "#5A4030", "matte"),
        Mat::new("stalk", "#5E5040", "satin"), // the picked fruit's stalk: one draw call with the fruit
    ];
    // the leaf cards (botany's clad): each leaf colour's own (`palette` gives them their colours), and the core
    // under them
    for (_, card) in CARD_OF {
        colours.push(Mat::new(card, "#FFFFFF", "leafcards"));
    }
    colours.push(Mat::new("core", "#FFFFFF", "leafcards"));
    Palette::new(&colours)
}

/// An sRGB hex colour, each channel `k` of its value.
pub fn _toned(colour: &str, k: f64) -> String {
    let channels = rgb_of(colour).map(|c| c as f64 / 255.0);
    let mut out = String::from("#");
    for c in channels {
        out.push_str(&format!("{:02X}", 255.min(py::round(c * 255.0 * k))));
    }
    out
}

/// The family's palette with some colours changed; `leaves` gives the leaf shades, darkest first (none: as they
/// are). The leaf cards take their leaf colours' (`CARD_OF`, toned by `CARD_TONE`), the core the darkest's.
pub fn palette(leaves: &[&str], colours: &[(&str, &str)]) -> Palette {
    let mut colours = colours.to_vec();
    colours.extend(LEAVES.iter().copied().zip(leaves.iter().copied()));
    let mut made = base().recoloured(&colours);
    for (leaf, card) in CARD_OF {
        let toned = _toned(&hex_of(made.get(leaf).colour), CARD_TONE);
        made = made.recoloured(&[(card, &toned)]);
    }
    let core = _toned(&hex_of(made.get("leaf").colour), CORE_TONE);
    made.recoloured(&[("core", &core)])
}

/// The palette's first `n` leaf shades, darkest first.
pub fn shades(p: &Palette, n: usize) -> Vec<Mat> {
    LEAVES[..n].iter().map(|k| p.get(k)).collect()
}

// ---- perches: where the game's birds sit (the entry's perches, perch.rs) -----------------------------------------
// Found on the built tree, never drawn: lines in from outside towards each clump of its crown (out from the trunk,
// a little up) meet its leaves, and a perch is a few centimetres in from there, on the outer twigs, with nothing
// hanging over it; conifers' on their tiers' rims, a few on bare limbs. Each tree's own random numbers (its seed),
// so the tree's are as they were and the meshes don't change.

/// A clump of a crown: (its middle, its half sizes (none: one that is only aimed at), -1..1 up the crown, whether
/// it is drawn).
pub type Clump = ((f64, f64, f64), Option<(f64, f64, f64)>, f64, bool);
/// A limb: (its line, its radius at each end).
pub type Limb = (Vec<V3>, f64, f64);

options! {
    /// `clump_aims`' options.
    ClumpAims { per: usize = 3, up: (f64, f64) = (0.17, 0.95) }
}

/// `per` lines in towards each drawn clump of a crown (`_mass`'s clumps), out from the trunk +- 50 degrees,
/// `up` radians up from level (steeper over the crown's top).
pub fn clump_aims(clumps: &[Clump], rng: &mut Random, o: ClumpAims) -> Vec<Aim> {
    let mut aims = Vec::new();
    for &((mx, my, mz), _, t, drawn) in clumps {
        if !drawn {
            continue;
        }
        let out = py::hypot(mx, my) > 0.25;
        let lift = (o.up.0, o.up.1 + 0.35 * 0.0_f64.max(t));
        aims.extend(perch::around(
            (mx, my, mz),
            rng,
            o.per,
            lift,
            if out { Some(atan2(my, mx)) } else { None },
            if out { 0.9 } else { PI },
        ));
    }
    aims
}

/// `clump_aims` with its options by name: `clump_aims!(clumps, rng, key = value, ...)`.
#[macro_export]
macro_rules! clump_aims {
    ($clumps:expr, $rng:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($clumps,) {
            (clumps,) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::ClumpAims::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::skeleton::clump_aims(clumps, $rng, o)
            }
        }
    };
}

// ---- broadleaves ----------------------------------------------------------------------------------------------

options! {
    /// `_bough`'s options (`material` None: the palette's bark).
    Bough { verts: usize = 6, res: usize = 3, material: Option<Mat> = None }
}

/// A tapering, curving bark `twig` through Blender `pts`, `r0` across at its start, `r1` at its end.
pub fn _bough<Q: Into<V3> + Copy>(
    name: impl AsRef<str>,
    pts: &[Q],
    r0: f64,
    r1: f64,
    p: &Palette,
    o: Bough,
) -> Obj {
    let material = o.material.unwrap_or_else(|| p.get("bark"));
    let twig = crate::botany::Twig {
        verts: o.verts,
        res: o.res,
    };
    crate::botany::twig(name, pts, r0, r1, material, twig)
}

/// `_bough` with its options by name: `bough!(name, pts, r0, r1, p, key = value, ...)`.
#[macro_export]
macro_rules! bough {
    ($name:expr, $pts:expr, $r0:expr, $r1:expr, $p:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $pts, $r0, $r1, $p) {
            (name, pts, r0, r1, p) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::Bough::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::skeleton::_bough(name, pts, r0, r1, p, o)
            }
        }
    };
}

options! {
    /// `_bend`'s options.
    Bend { lift: f64 = 0.25, wander: f64 = 0.18 }
}

/// Four points from `a` to `b`, bowed up by `lift` x its length and wandering sideways: a limb's line.
pub fn _bend(a: impl Into<V3>, b: impl Into<V3>, rng: &mut Random, o: Bend) -> Vec<V3> {
    let (a, b): (V3, V3) = (a.into(), b.into());
    let d = b - a;
    let side = if d.xy_length() > 1e-6 {
        v3(-d.y, d.x, 0).normalized()
    } else {
        v3(1, 0, 0)
    };
    let mut pts = vec![a];
    for f in [0.33, 0.66] {
        pts.push(
            a + d * f
                + v3(0, 0, o.lift * d.length() * sin(PI * f))
                + side * rng.uniform(-o.wander, o.wander) * d.length() * 0.3,
        );
    }
    pts.push(b);
    pts
}

/// `_bend` with its options by name: `bend!(a, b, rng, key = value, ...)`.
#[macro_export]
macro_rules! bend {
    ($a:expr, $b:expr, $rng:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($a, $b) {
            (a, b) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::Bend::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::skeleton::_bend(a, b, $rng, o)
            }
        }
    };
}

options! {
    /// `_trunk`'s options (`material` None: the palette's bark).
    Stem { verts: usize = 10, wobble: f64 = 0.03, material: Option<Mat> = None }
}

/// The trunk in one swept, tapering piece through Blender `pts` (radius `r` times `scale`, one per point), its
/// bark wobbled by noise sideways (not on a far version: its foot stays where the model's is).
pub fn _trunk(
    p: &Palette,
    pts: &[(f64, f64, f64)],
    r: f64,
    scale: &[f64],
    seed: f64,
    o: Stem,
) -> Obj {
    let stem = crate::sweep!(
        "trunk",
        pts,
        r,
        material = o.material.unwrap_or_else(|| p.get("bark")),
        verts = low(o.verts, 5),
        resolution = low(3, 1),
        scale = scale
    );
    if !is_low() && o.wobble != 0.0 {
        crate::jitter!(stem, o.wobble, size = 0.45, seed = seed, up = 0.0);
    }
    stem
}

/// `_trunk` with its options by name: `trunk!(p, pts, r, scale, seed, key = value, ...)`.
#[macro_export]
macro_rules! trunk {
    ($p:expr, $pts:expr, $r:expr, $scale:expr, $seed:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($p, $pts, $r, $scale, $seed) {
            (p, pts, r, scale, seed) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::Stem::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::skeleton::_trunk(p, pts, r, scale, seed, o)
            }
        }
    };
}

options! {
    /// `_roots`' options (`material` None: the palette's bark).
    Buttress { material: Option<Mat> = None, verts: usize = 5 }
}

/// `n` root buttresses flaring from a trunk `r` round at its foot: from `h` up its side out to the ground about
/// `reach` from its middle, ending just above it.
pub fn _roots(p: &Palette, rng: &mut Random, n: usize, r: f64, h: f64, reach: f64, o: Buttress) {
    for i in 0..n {
        let a = TAU * (i as f64 + rng.uniform(-0.2, 0.2)) / n as f64;
        let out = reach * rng.uniform(0.85, 1.15);
        let (c, s) = (cos(a), sin(a));
        _bough(
            format!("root{i}"),
            &[
                (0.3 * r * c, 0.3 * r * s, h),
                (0.8 * r * c, 0.8 * r * s, h * 0.36),
                (out * c, out * s, r * 0.2),
            ],
            r * 0.38,
            r * 0.16,
            p,
            Bough {
                verts: o.verts,
                res: 1,
                material: o.material,
            },
        );
    }
}

/// `_roots` with its options by name: `roots!(p, rng, n, r, h, reach, key = value, ...)`.
#[macro_export]
macro_rules! roots {
    ($p:expr, $rng:expr, $n:expr, $r:expr, $h:expr, $reach:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($p, $n, $r, $h, $reach) {
            (p, n, r, h, reach) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::Buttress::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::skeleton::_roots(p, $rng, n, r, h, reach, o)
            }
        }
    };
}

options! {
    /// `_mass`'s options.
    Mass { k: usize = 3, size: (f64, f64) = (0.5, 0.64), ring: f64 = 0.42, drop: (f64, f64) = (-0.35, 0.05), squash: (f64, f64) = (0.75, 0.88), noise: f64 = 0.34, far: f64 = 1.12, segments: usize = 6, rings: usize = 5, side: f64 = 0.86 }
}

/// A leaf mass at a bough's tip (Blender `tip`, the bough heading `heading` radians): a clump on the tip and
/// `k` - 1 round it, `ring` out and `drop` below, each `size` metres round (`side` x that off the tip), squashed
/// by `squash`, lighter as `light(z)` (0..1) rises; leaf cards once the tree is clad (botany's leaves). A far version
/// draws one lump spanning the mass's clumps (`far` x their reach), so it keeps the crown's size. Appends each
/// clump to `out`: (middle, half sizes, -1..1 up the crown, drawn), for the perches (`clump_aims`) and what sits on
/// the leaves (blooms).
#[allow(clippy::too_many_arguments)]
pub fn _mass(
    name: &str,
    rng: &mut Random,
    tip: impl Into<V3>,
    heading: f64,
    cols: &[Mat],
    light: &dyn Fn(f64) -> f64,
    out: &mut Vec<Clump>,
    o: Mass,
) {
    let tip: V3 = tip.into();
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    let mut first = None;
    let group = (name, out.len()); // the mass: its clumps' cards share a core
    let shades = cols.len() as i64;
    for m in 0..o.k {
        let at = if m == 0 {
            tip
        } else {
            let b = heading
                + (m as f64 - o.k as f64 / 2.0) * (3.2 / 1.max(o.k as i64 - 1) as f64)
                + rng.uniform(-0.4, 0.4);
            tip + v3(
                o.ring * cos(b),
                o.ring * sin(b),
                rng.uniform(o.drop.0, o.drop.1),
            )
        };
        let s = rng.uniform(o.size.0, o.size.1) * if m == 0 { 1.0 } else { o.side };
        let (sx, sy, sq) = (
            rng.uniform(0.9, 1.15),
            rng.uniform(0.9, 1.15),
            rng.uniform(o.squash.0, o.squash.1),
        );
        let li = 0.0_f64.max(1.0_f64.min(light(at.z as f64)));
        let shade = py::round(li * (shades - 1) as f64 + rng.uniform(-0.6, 0.6));
        let col = cols[0.max((shades - 1).min(shade)) as usize];
        let seed = out.len() as f64 * 1.7 + rng.random();
        let half = [s * sx, s * sy, s * sq];
        out.push((
            (at.x as f64, at.y as f64, at.z as f64),
            Some((half[0], half[1], half[2])),
            2.0 * li - 1.0,
            true,
        ));
        for j in 0..3 {
            lo[j] = lo[j].min(at[j] as f64 - half[j]);
            hi[j] = hi[j].max(at[j] as f64 + half[j]);
        }
        if m == 0 {
            first = Some((col, seed));
        }
        if !is_low() {
            crate::leaves!(
                format!("{name}{}", out.len()),
                s,
                (at.x as f64, at.y as f64, at.z as f64 - s * sq),
                col,
                scale = (sx, sy, sq),
                noise = o.noise,
                seed = seed,
                segments = o.segments,
                rings = o.rings,
                group = group
            );
        }
    }
    if is_low() {
        let [rx, ry, rz] = [0, 1, 2].map(|j| (hi[j] - lo[j]) / 2.0 * o.far);
        let (col, seed) = first.expect("a mass has a clump on its tip");
        crate::lump!(
            format!("{name}{}", out.len()),
            rx,
            (
                (lo[0] + hi[0]) / 2.0,
                (lo[1] + hi[1]) / 2.0,
                (lo[2] + hi[2]) / 2.0 - rz
            ),
            col,
            scale = (1.0, ry / rx, rz / rx),
            noise = o.noise,
            seed = seed,
            segments = o.segments,
            rings = o.rings
        );
    }
}

/// `_mass` with its options by name: `mass!(name, rng, tip, heading, cols, light, out, key = value, ...)`.
#[macro_export]
macro_rules! mass {
    ($name:expr, $rng:expr, $tip:expr, $heading:expr, $cols:expr, $light:expr, $out:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $tip, $heading, $cols, $light) {
            (name, tip, heading, cols, light) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::Mass::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::skeleton::_mass(name, $rng, tip, heading, cols, light, $out, o)
            }
        }
    };
}

options! {
    /// `_branches`' options (`leader_z` None: no leader; `material` None: the palette's bark; `mass`: `_mass`'s).
    Branches { leader_z: Option<f64> = None, bottom: f64 = -0.3, spread: f64 = 0.9, axis: f64 = 0.35, limb_r: (f64, f64) = (0.12, 0.05), bough_r: (f64, f64) = (0.05, 0.018), fork: (f64, f64) = (0.45, 0.6), lift: f64 = 0.15, wander: f64 = 0.2, rise: f64 = 0.15, verts: usize = 5, bough_verts: usize = 4, material: Option<Mat> = None, mass: Mass = Mass::default(), prefix: &'static str = "" }
}

/// A broadleaf's limbs and leaf masses. `tips` bough tips spread evenly over the crown's ellipsoid (`centre`,
/// `radii`; `spread` x out, from its top down to `bottom`, -1..1); `limbs` limbs from the trunk at heights `base_z`
/// (a range), each carrying the tips on its side: it runs `fork` of the way to their middle, bowed up by `lift` and
/// wandering `wander`, and forks into a bough to each tip, bowed up `rise`. The tips over the middle (nearer the
/// axis than `axis` x the radius) go on a leader from `leader_z` (none: the nearest limb's). A leaf mass at each
/// tip (`_mass`, `mass` its options). Returns (the clumps, as `_mass` lists them; the limbs as (their line, radius at
/// each end)). A far version leaves out the boughs (drawing their numbers).
#[allow(clippy::too_many_arguments)]
pub fn _branches(
    p: &Palette,
    rng: &mut Random,
    cols: &[Mat],
    centre: (f64, f64, f64),
    radii: (f64, f64, f64),
    tips: usize,
    limbs: usize,
    base_z: (f64, f64),
    o: Branches,
) -> (Vec<Clump>, Vec<Limb>) {
    let (cx, cy, cz) = centre;
    let (rx, ry, rz) = radii;
    let material = o.material.unwrap_or_else(|| p.get("bark"));
    let full = !is_low();
    let ga = PI * (3.0 - sqrt(5.0));
    let off = rng.random() * TAU;
    let mut points = Vec::new();
    for i in 0..tips {
        let t = 1.0 - (i as f64 + 0.5) / tips as f64 * (1.0 - o.bottom);
        let ring = sqrt(0.0_f64.max(1.0 - t * t));
        let a = off + ga * i as f64 + rng.uniform(-0.3, 0.3);
        let k = o.spread * rng.uniform(0.9, 1.08);
        points.push((
            v3(
                cx + rx * ring * cos(a) * k,
                cy + ry * ring * sin(a) * k,
                cz + rz * t * k,
            ),
            a,
            ring * k,
        ));
    }
    let heads: Vec<f64> = (0..limbs)
        .map(|j| off + TAU * (j as f64 + 0.5) / limbs as f64 + rng.uniform(-0.3, 0.3))
        .collect();
    let mut groups: Vec<Vec<V3>> = vec![Vec::new(); limbs + 1];
    for &(tip, a, rr) in &points {
        if o.leader_z.is_some() && rr < o.axis {
            groups[limbs].push(tip);
        } else {
            let turn = |j: usize| (py::fmod(a - heads[j] + PI, TAU) - PI).abs();
            let mut nearest = 0;
            for j in 1..limbs {
                if turn(j) < turn(nearest) {
                    nearest = j;
                }
            }
            groups[nearest].push(tip);
        }
    }

    let light = move |z: f64| (z - (cz - rz)) / (2.0 * rz);
    let (mut clumps, mut made) = (Vec::new(), Vec::new());
    for (j, group) in groups.iter().enumerate() {
        let z0 = if j < limbs {
            Some(rng.uniform(base_z.0, base_z.1))
        } else {
            o.leader_z
        };
        if group.is_empty() {
            continue;
        }
        let base = v3(0.0, 0.0, z0.expect("tips on a leader: it has a height"));
        let mid = math::sum(group.iter().copied()) / group.len();
        let f = base + (mid - base) * rng.uniform(o.fork.0, o.fork.1);
        let r0 = o.limb_r.0 * if j == limbs { 1.15 } else { 1.0 };
        let line = _bend(
            base,
            f,
            rng,
            Bend {
                lift: o.lift,
                wander: o.wander,
            },
        );
        _bough(
            format!("{}limb{j}", o.prefix),
            &line,
            r0,
            o.limb_r.1,
            p,
            Bough {
                verts: o.verts,
                res: 2,
                material: Some(material),
            },
        );
        made.push((line, r0, o.limb_r.1));
        for (i, &tip) in group.iter().enumerate() {
            let m = f
                + (tip - f) * 0.5
                + v3(
                    rng.uniform(-0.15, 0.15),
                    rng.uniform(-0.15, 0.15),
                    o.rise * (tip - f).length(),
                );
            if full {
                _bough(
                    format!("{}b{j}_{i}", o.prefix),
                    &[f, m, tip],
                    o.bough_r.0,
                    o.bough_r.1,
                    p,
                    Bough {
                        verts: o.bough_verts,
                        res: 2,
                        material: Some(material),
                    },
                );
            }
            _mass(
                &format!("{}clump", o.prefix),
                rng,
                tip,
                atan2(tip.y as f64, tip.x as f64),
                cols,
                &light,
                &mut clumps,
                o.mass.clone(),
            );
        }
    }
    (clumps, made)
}

/// `_branches` with its options by name:
/// `branches!(p, rng, cols, centre, radii, tips, limbs, base_z, key = value, ...)`.
#[macro_export]
macro_rules! branches {
    ($p:expr, $rng:expr, $cols:expr, $centre:expr, $radii:expr, $tips:expr, $limbs:expr, $base_z:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($p, $cols, $centre, $radii, $tips, $limbs, $base_z) {
            (p, cols, centre, radii, tips, limbs, base_z) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::Branches::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::skeleton::_branches(p, $rng, cols, centre, radii, tips, limbs, base_z, o)
            }
        }
    };
}

/// A value given at each point of an upright `line` (Blender points), at height `z`, linear between.
pub fn _along(line: &[(f64, f64, f64)], values: &[f64], z: f64) -> f64 {
    for i in 0..line.len().min(values.len()).saturating_sub(1) {
        let ((a, va), (b, vb)) = ((line[i], values[i]), (line[i + 1], values[i + 1]));
        if z <= b.2 {
            return va + (vb - va) * 0.0_f64.max((z - a.2) / (b.2 - a.2));
        }
    }
    values[values.len() - 1]
}

options! {
    /// `_hearts`' options.
    Heart { out: f64 = 0.4, prefix: &'static str = "heart" }
}

/// `n` dark lumps round the crown's middle, `out` x its radius out: they close its heart, the gaps between the
/// masses stay.
pub fn _hearts(
    rng: &mut Random,
    cols: &[Mat],
    n: usize,
    centre: (f64, f64, f64),
    radii: (f64, f64, f64),
    size: f64,
    o: Heart,
) {
    let (cx, cy, cz) = centre;
    let off = rng.random() * TAU;
    for i in 0..n {
        let a = off + TAU * (i as f64 + 0.5) / n as f64 + rng.uniform(-0.3, 0.3);
        let (s, seed) = (size * rng.uniform(0.9, 1.1), rng.random() * 9.0);
        let at = (
            cx + radii.0 * o.out * cos(a),
            cy + radii.1 * o.out * sin(a),
            cz + radii.2 * rng.uniform(-0.45, 0.05) - s * 0.85,
        );
        let col = cols[rng.randrange(2) as usize];
        crate::leaves!(
            format!("{}{i}", o.prefix),
            s,
            at,
            col,
            scale = (1, 1, 0.85),
            noise = 0.3,
            seed = seed,
            segments = 6,
            rings = 5
        );
    }
}

/// `_hearts` with its options by name: `hearts!(rng, cols, n, centre, radii, size, key = value, ...)`.
#[macro_export]
macro_rules! hearts {
    ($rng:expr, $cols:expr, $n:expr, $centre:expr, $radii:expr, $size:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($cols, $n, $centre, $radii, $size) {
            (cols, n, centre, radii, size) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::Heart::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::skeleton::_hearts($rng, cols, n, centre, radii, size, o)
            }
        }
    };
}

options! {
    /// `limb_points`' options.
    LimbPoints { along: [f64; 3] = [0.45, 0.6, 0.75] }
}

/// Points on top of the limbs `_branches` returns, `along` them (for perches on bare limbs).
pub fn limb_points(limbs: &[Limb], o: LimbPoints) -> Vec<V3> {
    let mut out = Vec::new();
    for (line, r0, r1) in limbs {
        for f in o.along {
            let i = (py::int(f * (line.len() - 1) as f64) as usize).min(line.len() - 2);
            let k = f * (line.len() - 1) as f64 - i as f64;
            let d = (line[i + 1] - line[i]).normalized(); // (on a steep limb its top is higher over its axis)
            let lift = (r0 + (r1 - r0) * f)
                / 0.35_f64.max(sqrt(0.0_f64.max(1.0 - d.z as f64 * d.z as f64)));
            out.push(line[i] + (line[i + 1] - line[i]) * k + v3(0, 0, lift + 0.01));
        }
    }
    out
}

/// `limb_points` with its options by name: `limb_points!(limbs, key = value, ...)`.
#[macro_export]
macro_rules! limb_points {
    ($limbs:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($limbs,) {
            (limbs,) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::LimbPoints::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::skeleton::limb_points(limbs, o)
            }
        }
    };
}

// ---- fruit ----------------------------------------------------------------------------------------------------

/// A fruit tree's fruit follows its ripeness (content `grows`, G-68): the fruit hung over the crown and the one at
/// the front a resident picks (`pick`: it shrinks away) are moving parts with one growth clip, `ripen`
/// (`motions.growth`, motion's stages): in fruit at rest; picked bare, and growing its fruit again, none (they
/// shrink into the crown's middle). (picked, growing, planted).
pub const BARE: (f64, f64, f64) = (0.001, 0.001, 0.001);

/// `_fruiting`'s options: `picked` makes the fruit at the front, a part of its own with its origin at `at`;
/// `tiles` is where the tree's leaf cards' pictures are in the atlas (`clad`'s).
#[derive(Default)]
pub struct Fruiting<'a> {
    pub picked: Option<&'a dyn Fn()>,
    pub at: Option<(f64, f64, f64)>,
    pub tiles: Tiles,
}

/// The still part, and the fruit `fruit()` hangs over the crown (origin `crown_mid`) as a moving part following
/// the tree's ripeness; with `picked`, the one at the front it makes (origin `at`) a second one that also shrinks
/// away while it's picked (an apple picked to eat). The fruit hung (on the crown's lumps: `_hang`), the still
/// part's leaves are drawn as cards (`clad`, palette `p`, `seed`). The far version all one still part, in fruit.
pub fn _fruiting(
    p: &Palette,
    name: &str,
    still: Obj,
    seed: i128,
    crown_mid: (f64, f64, f64),
    fruit: impl FnOnce(),
    o: Fruiting,
) -> Vec<Obj> {
    if is_low() {
        fruit();
        if let Some(picked) = o.picked {
            picked();
        }
        return settle(&[join_all(&still.name())]);
    }
    let cards = || Clad {
        more: None,
        tiles: o.tiles.clone(),
    };
    let parts = new_parts(fruit);
    let hung = join(&parts, &format!("{name}_fruit"), Some(crown_mid.into()));
    let Some(picked) = o.picked else {
        let still = clad(p, still, seed, cards());
        settle(&[still, hung]);
        stages(hung, "ripen", BARE.0, BARE.1, BARE.2, Stages::default());
        return vec![still, hung];
    };
    let parts = new_parts(picked);
    let front = join(&parts, &format!("{name}_part"), o.at.map(P3::from));
    let still = clad(p, still, seed, cards());
    settle(&[still, hung, front]);
    squash(
        front,
        "pick",
        0.001,
        Squash {
            seconds: 0.8,
            ..Squash::default()
        },
    );
    stages(hung, "ripen", BARE.0, BARE.1, BARE.2, Stages::default());
    stages(front, "ripen", BARE.0, BARE.1, BARE.2, Stages::default());
    vec![still, hung, front]
}

/// `_fruiting` with its options by name:
/// `fruiting!(p, name, still, seed, crown_mid, fruit, picked = || ..., at = (x, y, z), tiles = ...)`.
#[macro_export]
macro_rules! fruiting {
    (@set $o:ident, picked, $v:expr) => {
        $o.picked = Some(&$v);
    };
    (@set $o:ident, $k:ident, $v:expr) => {
        $o.$k = $crate::Arg::arg($v);
    };
    ($p:expr, $name:expr, $still:expr, $seed:expr, $crown_mid:expr, $fruit:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($p, $name, $still, $seed, $crown_mid, $fruit) {
            (p, name, still, seed, crown_mid, fruit) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::Fruiting::default();
                $($crate::fruiting!(@set o, $k, $v);)*
                $crate::botany::skeleton::_fruiting(p, name, still, seed, crown_mid, fruit, o)
            }
        }
    };
}

options! {
    /// `_hang`'s options.
    Hang { band: (f64, f64) = (-0.6, 0.5), flat: f64 = 1.0, segments: usize = 6, rings: usize = 4, sink: f64 = -0.1 }
}

/// Fruit hung over a crown as it stands: `n` lines in towards the crown's middle (`centre`) from round its
/// ellipsoid (`radii`), from `band` (-1..1) up it, each meeting its leaves; a fruit `size` across there, its middle
/// `sink` of itself in from them (out, if less than 0), so none floats in a gap or hides inside.
#[allow(clippy::too_many_arguments)]
pub fn _hang(
    rng: &mut Random,
    n: usize,
    centre: (f64, f64, f64),
    radii: (f64, f64, f64),
    size: f64,
    materials: &[Mat],
    name: &str,
    o: Hang,
) {
    let rays = Rays::of_scene();
    let c = V3::from(centre);
    let ga = PI * (3.0 - sqrt(5.0));
    let mut spots = Vec::new();
    for i in 0..n {
        let t = o.band.0 + (o.band.1 - o.band.0) * (i as f64 + 0.5) / n as f64;
        let a = i as f64 * ga + rng.uniform(-0.3, 0.3);
        let ring = sqrt(0.0_f64.max(1.0 - t * t));
        let d = v3(
            radii.0 * ring * cos(a),
            radii.1 * ring * sin(a),
            radii.2 * t,
        )
        .normalized();
        if let Some(hit) = rays.cast(c + d * 8, -d, 8.0) {
            spots.push(hit.at - d * size * o.sink);
        }
    }
    for (i, spot) in spots.iter().enumerate() {
        crate::lump!(
            format!("{name}{i}"),
            size / 2.0,
            (spot.x, spot.y, spot.z as f64 - size / 2.0 * o.flat),
            materials[i % materials.len()],
            scale = (1, 1, o.flat),
            seed = i as f64 * 0.77,
            segments = o.segments,
            rings = o.rings,
            noise = 0.06
        );
    }
}

/// `_hang` with its options by name: `hang!(rng, n, centre, radii, size, materials, name, key = value, ...)`.
#[macro_export]
macro_rules! hang {
    ($rng:expr, $n:expr, $centre:expr, $radii:expr, $size:expr, $materials:expr, $name:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($n, $centre, $radii, $size, $materials, $name) {
            (n, centre, radii, size, materials, name) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::skeleton::Hang::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::skeleton::_hang($rng, n, centre, radii, size, materials, name, o)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colour_is_toned_a_channel_at_a_time_halves_to_the_even() {
        // (the oak's darkest leaf: its green comes to 110.5)
        assert_eq!(_toned("#558238", CARD_TONE), "#486E30");
        assert_eq!(_toned("#558238", CORE_TONE), "#334E22");
        assert_eq!(_toned("#FFF2F8", 1.3), "#FFFFFF");
    }

    #[test]
    fn a_palette_takes_a_species_colours_and_tones_its_cards() {
        let p = palette(
            &["#3A6A30", "#467838", "#548640", "#62944A"],
            &[("bark", "#8E867C")],
        );
        let hex = |name: &str| hex_of(p.get(name).colour);
        assert_eq!(
            (hex("leaf"), hex("leaf4"), hex("leaf5")),
            ("#3A6A30".into(), "#62944A".into(), "#9EC25C".into())
        );
        assert_eq!(
            (hex("bark"), p.get("bark").finish),
            ("#8E867C".into(), "bark")
        );
        assert_eq!(
            (hex("card"), hex("card5"), hex("cardg2"), hex("core")),
            (
                "#315A29".into(),
                "#86A54E".into(),
                "#537A35".into(),
                "#23401D".into()
            )
        );
        assert_eq!(p.get("core").finish, "leafcards");
        assert_eq!(shades(&p, 4).len(), 4);
        // with nothing changed it is the family's, its cards toned
        assert_eq!(hex_of(palette(&[], &[]).get("leaf").colour), "#558238");
    }

    #[test]
    fn a_value_along_a_line_is_linear_between_its_points() {
        let line = [(0.0, 0.0, 0.0), (0.0, 0.0, 1.0), (0.1, 0.0, 3.0)];
        let values = [1.0, 0.5, 0.25];
        assert_eq!(_along(&line, &values, -1.0), 1.0);
        assert_eq!(_along(&line, &values, 0.5), 0.75);
        assert_eq!(_along(&line, &values, 2.0), 0.375);
        assert_eq!(_along(&line, &values, 9.0), 0.25);
    }

    #[test]
    fn a_limb_bends_between_its_ends_on_two_draws() {
        let (mut rng, mut same) = (Random::new(5), Random::new(5));
        let line = _bend(
            (0.0, 0.0, 1.0),
            v3(1.0, 0.5, 2.0),
            &mut rng,
            Bend::default(),
        );
        assert_eq!(line.len(), 4);
        assert_eq!((line[0], line[3]), (v3(0.0, 0.0, 1.0), v3(1.0, 0.5, 2.0)));
        // bowed up: its middle points over the straight line
        assert!(line[1].z > 1.33 && line[2].z > 1.66);
        same.random();
        same.random();
        assert_eq!(rng.random(), same.random());
    }

    #[test]
    fn lines_aim_at_the_clumps_that_are_drawn() {
        let clumps: Vec<Clump> = vec![
            ((1.0, 0.0, 3.0), None, 0.5, true),
            ((0.0, 0.1, 4.0), None, 1.0, true),
            ((2.0, 0.0, 3.0), None, 0.0, false),
        ];
        let aims = clump_aims(&clumps, &mut Random::new(1), ClumpAims::default());
        assert_eq!(aims.len(), 6);
        // out from the trunk towards a clump off the axis, within 0.9 radians of its heading
        for (at, d) in &aims[..3] {
            assert_eq!(*at, v3(1.0, 0.0, 3.0));
            assert!(d.x > 0.0 && (d.y as f64).atan2(d.x as f64).abs() <= 0.9 + 1e-6);
            assert!(d.z > 0.0);
        }
        let limbs: Vec<Limb> = vec![(vec![v3(0, 0, 1), v3(1, 0, 1), v3(2, 0, 1)], 0.1, 0.05)];
        let on = limb_points(&limbs, LimbPoints::default());
        assert_eq!(on.len(), 3);
        // on top of the limb: its radius there and a centimetre over its axis
        assert!((on[0].z as f64 - (1.0 + 0.1 + (0.05 - 0.1) * 0.45 + 0.01)).abs() < 1e-6);
        assert!((on[0].x as f64 - 0.9).abs() < 1e-6);
    }
}
