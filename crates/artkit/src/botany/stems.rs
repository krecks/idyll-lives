//! Stems, stalks and canes: swept through a few points, tapering (families/plants.py's `twig` and what is made
//! of it and of `tube` and `leaf`).

use crate::botany::leaf::{Leaf, Tube, leaf, tube};
use crate::far::{is_low, low};
use crate::forms::{Sweep, by, sweep};
use crate::materials::Mat;
use crate::math::{PI, TAU, V3, cos, sin, v3};
use crate::scene::Obj;
use crate::shapes::options;

options! {
    /// `arc`'s options.
    ArcOf { steps: usize = 4 }
}

/// Points from `a` to `b` bowed up by `lift` metres in the middle (an arching stem).
pub fn arc(a: impl Into<V3>, b: impl Into<V3>, lift: f64, o: ArcOf) -> Vec<V3> {
    let (a, b): (V3, V3) = (a.into(), b.into());
    let steps = o.steps;
    (0..=steps)
        .map(|i| {
            a + (b - a) * (i as f64 / steps as f64)
                + v3(0, 0, lift * sin(PI * i as f64 / steps as f64))
        })
        .collect()
}

options! {
    /// `twig`'s options.
    Twig { verts: usize = 5, res: usize = 2 }
}

/// A tapering stem, cane or twig (`sweep`) through Blender `points`, `r0` across at the first and `r1` at the
/// last, curved through them (`res`); a far version three-sided and straight between them.
pub fn twig<P: Into<V3> + Copy>(
    name: impl AsRef<str>,
    points: &[P],
    r0: f64,
    r1: f64,
    material: Mat,
    o: Twig,
) -> Obj {
    let k = r1 / r0;
    sweep(
        name,
        points,
        r0,
        Sweep {
            material: Some(material),
            verts: Some(low(o.verts, 3)),
            resolution: low(o.res, 1),
            scale: by(move |w| 1.0 + (k - 1.0) * w),
            ..Sweep::default()
        },
    )
}

options! {
    /// `stem`'s options.
    Stem { r: f64 = 0.006 }
}

/// A straight, three-sided stem from `a` to `b`, a little thinner at its top.
pub fn stem(
    name: impl AsRef<str>,
    a: impl Into<V3>,
    b: impl Into<V3>,
    material: Mat,
    o: Stem,
) -> Obj {
    let r = o.r;
    let (a, b): (V3, V3) = (a.into(), b.into());
    tube(
        name,
        &[a, b],
        r,
        r * 0.8,
        material,
        Tube {
            verts: 3,
            ..Tube::default()
        },
    )
}

options! {
    /// `blade`'s options.
    Blade { droop: f64 = 0.3 }
}

/// A narrow leaf (grass, a bulb's leaf), cheap: three rows of three.
#[allow(clippy::too_many_arguments)]
pub fn blade(
    name: impl AsRef<str>,
    base: impl Into<V3>,
    yaw: f64,
    tilt: f64,
    length: f64,
    width: f64,
    material: Mat,
    o: Blade,
) -> Obj {
    leaf(
        name,
        base,
        yaw,
        tilt,
        length,
        width,
        material,
        Leaf {
            shape: "strap",
            droop: o.droop,
            thick: 0.006,
            fold: 0.3,
            rows: 3,
            ring: 3,
            ..Leaf::default()
        },
    )
}

options! {
    /// `stalk`'s options.
    Stalk { r: f64 = 0.005, bow: f64 = 0.03 }
}

/// A flower's stalk from `a` up to `b`, bowed a little sideways on its way (two segments, three-sided); none on a
/// far version (too thin to see from afar; nothing stands on the floor on one alone).
pub fn stalk(
    name: impl AsRef<str>,
    a: impl Into<V3>,
    b: impl Into<V3>,
    material: Mat,
    o: Stalk,
) -> Option<Obj> {
    if is_low() {
        return None;
    }
    let (r, bow) = (o.r, o.bow);
    let (a, b): (V3, V3) = (a.into(), b.into());
    let side = (b - a).cross(v3(0, 0, 1));
    let side = if side.length() > 1e-6 {
        side.normalized() * bow
    } else {
        v3(bow, 0, 0)
    };
    Some(twig(
        name,
        &[a, a.lerp(b, 0.5) + side, b],
        r,
        r * 0.75,
        material,
        Twig { verts: 3, res: 1 },
    ))
}

options! {
    /// `ribbed`'s options (`verts` None: two a rib).
    Ribbed { depth: f64 = 0.12, verts: Option<usize> = None }
}

/// A cactus's ribbed cross-section: `ribs` ribs round, `depth` of `r` deep between them.
pub fn ribbed(r: f64, ribs: usize, o: Ribbed) -> Vec<(f64, f64)> {
    let depth = o.depth;
    let verts = match o.verts {
        Some(verts) if verts != 0 => verts,
        _ => ribs * 2,
    };
    let (ribs, n) = (ribs as f64, verts as f64);
    (0..verts)
        .map(|i| {
            let i = i as f64;
            (
                r * (1.0 - depth * (0.5 - 0.5 * cos(TAU * ribs * i / n))) * cos(TAU * i / n),
                r * (1.0 - depth * (0.5 - 0.5 * cos(TAU * ribs * i / n))) * sin(TAU * i / n),
            )
        })
        .collect()
}
