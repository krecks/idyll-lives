//! Conifers: whorls of drooping sprays of needles round a trunk, and the lines their perches are found along.

use crate::far::{is_low, low};
use crate::materials::{Mat, Palette};
use crate::math::{TAU, cos, pow, sin, v3};
use crate::perch::{self, Aim};
use crate::rand::Random;
use crate::shapes::options;

options! {
    /// `tier_aims`' options.
    TierAims { per: usize = 4 }
}

/// `per` lines in from round each conifer tier towards its axis a little over its foot (`feet`: (x, y, z) each),
/// from a little above: they meet the tier near its rim, where its boughs end.
pub fn tier_aims(rng: &mut Random, feet: &[(f64, f64, f64)], o: TierAims) -> Vec<Aim> {
    let mut aims = Vec::new();
    for &foot in feet {
        aims.extend(perch::all_around(foot, rng, o.per, (0.15, 0.5)));
    }
    aims
}

/// `tier_aims` with its options by name: `tier_aims!(rng, feet, key = value, ...)`.
#[macro_export]
macro_rules! tier_aims {
    ($rng:expr, $feet:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($feet,) {
            (feet,) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::conifer::TierAims::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::conifer::tier_aims($rng, feet, o)
            }
        }
    };
}

/// A conifer bough's spray across (u up, v across): a ridge along the bough, the needles falling away to each
/// side and turning under, like a gull's wing; a far version's a plain wedge.
pub fn _pad_profile(w: f64, h: f64) -> Vec<(f64, f64)> {
    if is_low() {
        return vec![(h * 0.5, 0.0), (-0.6 * h, w), (-0.6 * h, -w)];
    }
    vec![
        (h * 0.5, 0.0),
        (-0.05 * h, 0.6 * w),
        (-0.6 * h, w),
        (-0.3 * h, 0.0),
        (-0.6 * h, -w),
        (-0.05 * h, -0.6 * w),
    ]
}

options! {
    /// `conifer`'s options.
    Conifer { top_d: f64 = 0.5, droop: f64 = 0.35, lift: f64 = 0.0, thick: f64 = 0.32, fill: f64 = 0.62, bare: usize = 2, offset: f64 = 0.1 }
}

/// A conifer's tiers from `z0` (the first tier's foot), each `step` above the last, `base_d` across narrowing to
/// `top_d`, `h` tall: each a whorl of `boughs[i]` boughs out of the trunk, each a spray of needles
/// (`_pad_profile`) along a bough that runs out from high in the tier, level, then droops `droop` x the tier's
/// height to its foot at the rim (its tip turned up by `lift`), wide in the middle and narrowing to its tip; a
/// dark cone inside closes the whorl. The lowest `bare` tiers show their bare boughs from below. Returns each
/// tier's foot (for perches).
#[allow(clippy::too_many_arguments)]
pub fn conifer(
    p: &Palette,
    rng: &mut Random,
    tiers: usize,
    base_d: f64,
    z0: f64,
    step: f64,
    h: f64,
    cols: &[Mat],
    boughs: &[usize],
    o: Conifer,
) -> Vec<(f64, f64, f64)> {
    let mut feet = Vec::new();
    let full = !is_low();
    for i in 0..tiers {
        let t = i as f64 / 1.max(tiers as i64 - 1) as f64;
        let r = (base_d + (o.top_d - base_d) * pow(t, 0.9)) / 2.0 * rng.uniform(0.95, 1.05);
        let hh = h * (1.0 - 0.35 * t);
        let foot = z0 + i as f64 * step;
        let (cx, cy) = (
            rng.uniform(-o.offset, o.offset),
            rng.uniform(-o.offset, o.offset),
        );
        let n = boughs[i];
        let off = rng.random() * TAU;
        let top = foot + hh * 0.55;
        crate::cone!(
            format!("tier{i}"),
            r * 0.72,
            low(0.03, 0.0),
            hh * 0.85,
            at = (cx * 0.5, cy * 0.5, foot + 0.15 + hh * 0.425),
            material = cols[0],
            verts = low(10, 6)
        );
        for k in 0..n {
            let a = off + TAU * (k as f64 + rng.uniform(-0.2, 0.2)) / n as f64;
            let reach = r * rng.uniform(0.88, 1.06);
            let fall = (top - foot) * o.droop * 2.0 * rng.uniform(0.85, 1.15);
            let col = cols[(i + k + rng.randint(0, 1) as usize) % cols.len()];
            let (c, s_) = (cos(a), sin(a));

            let at = |f: f64, dz: f64| v3(cx * f + reach * f * c, cy * f + reach * f * s_, dz);
            let line = [
                at(0.04, top + 0.05),
                at(0.32, top),
                at(0.6, top - fall * 0.35),
                at(0.84, top - fall * 0.8),
                at(1.0, top - fall + o.lift * hh),
            ];
            let w = (reach * TAU / n as f64 * 0.62).min(reach * 0.55) * rng.uniform(0.9, 1.1);
            let profile = _pad_profile(w, o.thick * hh / 2.1);
            if full {
                crate::sweep!(
                    format!("spray{i}_{k}"),
                    &line,
                    0.0,
                    material = col,
                    profile = profile,
                    scale = [0.35, 0.9, 1.0, 0.75, 0.25],
                    up = (0, 0, 1)
                );
            } else {
                crate::sweep!(
                    format!("spray{i}_{k}"),
                    &[line[0], line[2], line[4]],
                    0.0,
                    material = col,
                    profile = profile,
                    scale = [0.35, 1.0, 0.3],
                    up = (0, 0, 1)
                );
            }
            if full && i < o.bare {
                crate::sweep!(
                    format!("bough{i}_{k}"),
                    &[
                        line[0] + v3(0, 0, -0.06),
                        line[2] + v3(0, 0, -0.1),
                        line[3] + v3(0, 0, -0.12)
                    ],
                    0.035,
                    material = p.get("bark"),
                    verts = 4,
                    scale = [1.0, 0.6, 0.3]
                );
            }
        }
        feet.push((cx, cy, foot + 0.2));
    }
    feet
}

/// `conifer` with its options by name:
/// `conifer!(p, rng, tiers, base_d, z0, step, h, cols, boughs, key = value, ...)`.
#[macro_export]
macro_rules! conifer {
    ($p:expr, $rng:expr, $tiers:expr, $base_d:expr, $z0:expr, $step:expr, $h:expr, $cols:expr, $boughs:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($p, $tiers, $base_d, $z0, $step, $h, $cols, $boughs) {
            (p, tiers, base_d, z0, step, h, cols, boughs) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::conifer::Conifer::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::conifer::conifer(p, $rng, tiers, base_d, z0, step, h, cols, boughs, o)
            }
        }
    };
}

/// A conifer's or the cypress's top, from straight above (a blackbird's song post).
pub fn top() -> Vec<Aim> {
    vec![(v3(0.02, 0.0, 6.0), v3(0, 0, 1))]
}
