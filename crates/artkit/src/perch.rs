//! Perches: where the game's birds may sit on a model (a tree's outer twigs, a hedge's top), the pack entry's
//! `perches`: [[x, y, z], ...] in the model's own game axes (x, height, depth: Blender (-x, z, -y)), metres, at
//! rest.
//!
//! A build finds them on its own geometry (`surface`: a line in from outside to where it meets the leaves, a
//! little way in) and keeps a spread of them (`spread`), then `keep`s them (`find` does the three); `settle`
//! moves them with the model and the family writes them into the entry. They are data only: finding them casts
//! rays at the built meshes and draws no random number of the build's (each build that has perches gives its own
//! `Random`), so the meshes are what they were. A far version has none.

use crate::bvh::Rays;
use crate::far::is_low;
use crate::math::{self, PI, V3, v3};
use crate::py;
use crate::rand::Random;
use crate::scene;
use crate::shapes::options;

/// A line in towards a point: (the point, the unit vector out from it).
pub type Aim = (V3, V3);

pub fn clear() {
    scene::with(|s| s.perches.clear());
}

/// Keeps `points` (Blender metres) as the model's perches; nothing while a far version is built.
pub fn keep(points: &[V3]) {
    if !is_low() {
        scene::with(|s| s.perches.extend_from_slice(points));
    }
}

/// Moves the perches with the model (`settle`).
pub fn shift(by: V3) {
    scene::with(|s| {
        for p in &mut s.perches {
            *p += by;
        }
    });
}

/// The perches as the pack writes them: game (x, height, depth), to the millimetre.
pub fn game() -> Vec<[f64; 3]> {
    let r = |x: f32| py::round_to(x as f64, 3) + 0.0;
    scene::with(|s| {
        s.perches
            .iter()
            .map(|p| [r(-p.x), r(p.z), r(-p.y)])
            .collect()
    })
}

/// Where lines in from outside meet the model as it stands now: for each (target, direction) in `aims`, the line
/// from `reach` metres out along `direction` back in towards `target` meets its first face. A perch is `inset`
/// metres further in from there (a bird's feet among the outer twigs, its body showing), kept only where that
/// face looks out (towards the line) and not down (its normal's z over `facing`) and nothing hangs over it
/// within `headroom` metres (a clump above it, a tier's rim). Returns the perches.
pub fn surface(aims: &[Aim], inset: f64, reach: f64, headroom: f64, facing: f64) -> Vec<V3> {
    scene::update(); // (Blender works the scene out to cast rays at it)
    let rays = Rays::of_scene();
    let up = V3::Z;
    let mut out = Vec::new();
    for &(target, d) in aims {
        let d = d.normalized();
        let Some(hit) = rays.cast(target + d * reach, -d, reach * 2.0) else {
            continue;
        };
        if hit.normal.dot(d) <= 0.05 || (hit.normal.z as f64) < facing {
            continue;
        }
        if headroom != 0.0
            && rays
                .cast(hit.at + hit.normal * 0.02 + up * 0.01, up, headroom)
                .is_some()
        {
            continue;
        }
        out.push(hit.at - d * inset);
    }
    out
}

/// Which of `points` (on a bare limb's top, Blender metres) stand in the open: nothing over them within
/// `headroom` and not inside the leaves (the first face straight up from them doesn't face up: that is a clump's
/// inside).
pub fn bare(points: &[V3], headroom: f64) -> Vec<V3> {
    scene::update();
    let rays = Rays::of_scene();
    let up = V3::Z;
    points
        .iter()
        .copied()
        .filter(|&p| match rays.cast(p + up * 0.01, up, 30.0) {
            Some(hit) => !(hit.normal.z > 0.0 || (hit.at - p).length() < headroom),
            None => true,
        })
        .collect()
}

/// Up to `n` of `points` spread round the model: the highest first, then each the farthest from those kept (none
/// nearer than `gap` metres to another). Deterministic.
pub fn spread(points: &[V3], n: usize, gap: f64) -> Vec<V3> {
    if points.is_empty() {
        return Vec::new();
    }
    let key = |p: &V3| (p.z, -p.x, -p.y);
    let mut first = 0;
    for (i, p) in points.iter().enumerate() {
        if key(p) > key(&points[first]) {
            first = i;
        }
    }
    let mut kept = vec![points[first]];
    let mut near: Vec<f64> = points.iter().map(|&p| (p - kept[0]).length()).collect();
    while kept.len() < n {
        let mut i = 0;
        for k in 1..points.len() {
            if near[k] > near[i] {
                i = k;
            }
        }
        if near[i] <= gap.max(1e-6) {
            break;
        }
        kept.push(points[i]);
        for k in 0..points.len() {
            near[k] = near[k].min((points[k] - points[i]).length());
        }
    }
    kept
}

options! {
    /// `find`'s options.
    Find { inset: f64 = 0.03, gap: f64 = 0.45, facing: f64 = 0.2, reach: f64 = 8.0, headroom: f64 = 0.25, extra: Vec<Aim> = Vec::new(), limbs: Vec<V3> = Vec::new(), limbs_n: Option<usize> = None, limbs_headroom: f64 = 0.3 }
}

/// Keeps up to `n` perches on the model as it stands (`keep`; nothing on a far version): where `aims` meet its
/// leaves (`surface`: `inset` metres in, on leaves facing up over `facing` (the cypress's stand upright), within
/// `reach`, `headroom` clear), where `extra` do (twice as far in, at any facing: a conifer's top), and those of
/// `limbs` (points on bare limbs or canes) that stand in the open (`bare`, `limbs_headroom` clear), spread over it
/// at least `gap` apart. With `limbs_n`, up to that many of the limbs' are spread on their own and kept beside the
/// leaves' (a tree's, which its crown's would crowd out); without, they are spread with the leaves' (a rose's
/// canes).
pub fn find(n: usize, aims: &[Aim], o: Find) {
    if is_low() {
        return;
    }
    let mut leaves = surface(aims, o.inset, o.reach, o.headroom, o.facing);
    leaves.extend(surface(&o.extra, o.inset * 2.0, o.reach, o.headroom, -1.0));
    let mut open_limbs = bare(&o.limbs, o.limbs_headroom);
    let kept = match o.limbs_n {
        None => {
            let mut all = leaves.clone();
            all.extend_from_slice(&open_limbs);
            spread(&all, n, o.gap)
        }
        Some(limbs_n) => {
            open_limbs = if limbs_n > 0 {
                spread(&open_limbs, limbs_n, o.gap)
            } else {
                Vec::new()
            };
            let mut kept = spread(&leaves, n.saturating_sub(open_limbs.len()), o.gap);
            kept.extend_from_slice(&open_limbs);
            kept
        }
    };
    scene::log(format!(
        "[art] perches: {} kept, of {} on the leaves and {} on bare limbs",
        kept.len(),
        leaves.len(),
        open_limbs.len()
    ));
    keep(&kept);
}

/// `n` lines in towards `centre` from round it, `elevation` radians up from level (drawn from `rng`, the
/// perches' own), each heading `heading` +- `turn` (None, pi: anywhere).
pub fn around(
    centre: impl Into<V3>,
    rng: &mut Random,
    n: usize,
    elevation: (f64, f64),
    heading: Option<f64>,
    turn: f64,
) -> Vec<Aim> {
    let centre: V3 = centre.into();
    (0..n)
        .map(|_| {
            let a = heading.unwrap_or(0.0) + rng.uniform(-turn, turn);
            let e = rng.uniform(elevation.0, elevation.1);
            (
                centre,
                v3(
                    math::cos(a) * math::cos(e),
                    math::sin(a) * math::cos(e),
                    math::sin(e),
                ),
            )
        })
        .collect()
}

/// `around` anywhere round `centre`.
pub fn all_around(
    centre: impl Into<V3>,
    rng: &mut Random,
    n: usize,
    elevation: (f64, f64),
) -> Vec<Aim> {
    around(centre, rng, n, elevation, None, PI)
}
