//! The regions of a body, and the weights a point there is given.

use super::*;

/// How far in front of the neck's joint the neck's upper ring (under the jaw) has its middle, of the
/// neck's radius: with it behind the joint the nape stood behind the skull's base, a wedge down the back
/// of the head that the hair's nape tips landed on.
pub(crate) const NAPE: f64 = 0.15;
const SPINE: [&str; 4] = ["pelvis", "spine_01", "spine_02", "spine_03"];
pub const REGIONS: [&str; 8] = ["torso", "arm", "hand", "leg", "foot", "neck", "head", "ear"];
pub const NR: usize = REGIONS.len();
/// The arm's root loop on the chest rings (k: torso ring index): right side, then mirrored.
pub(crate) fn arm_k(s: &str) -> (usize, usize, usize) {
    if s == "r" { (2, 3, 4) } else { (10, 9, 8) }
}
/// The thighs' first rings keep this far (x thigh radius) from the midline, either side of the crotch.
pub(crate) const CROTCH_GAP: f64 = 0.1;
/// How much wider than the palm's columns the knuckles' are (and the fingers, which sit between
/// them): fingers the palm's width apart touch, and a hand a few pixels wide then reads as a mitten.
/// 6 % more (they were 2 % less, 0.98) opens a gap between them and still leaves the knuckles about
/// the palm's width (its widest ring) rather than a fan.
pub(crate) const KNUCKLE_SPREAD: f64 = 1.06;

pub fn region(name: &str) -> usize {
    REGIONS.iter().position(|&r| r == name).expect("region")
}

/// Weighted sum of bone -> weight lists: mix(&[(0.5, a), (0.5, b)]).
pub(crate) fn mix(pairs: &[(f64, &Weights)]) -> Weights {
    let mut out: Weights = Vec::new();
    for &(k, w) in pairs {
        for &(n, v) in w {
            match out.iter_mut().find(|e| e.0 == n) {
                Some(e) => e.1 += k * v,
                None => out.push((n, 0.0 + k * v)),
            }
        }
    }
    out
}

pub(crate) fn one(name: &'static str) -> Weights {
    vec![(name, 1.0)]
}

/// Leaks a formatted bone name (the names are a fixed small set; cage weights hold `&'static str`).
pub(crate) fn bone(name: String) -> &'static str {
    super::rig::BONES[b(&name)]
}

/// Bone weights along the spine by height (hat functions between the segments' middles).
pub(crate) fn spine_weights(p: &[V3; NB]) -> impl Fn(f64) -> Weights {
    let ys: Vec<f64> = SPINE
        .iter()
        .chain(["neck_01"].iter())
        .map(|n| p[b(n)].y)
        .collect();
    let mids: Vec<f64> = (0..4).map(|i| (ys[i] + ys[i + 1]) / 2.0).collect();
    move |y: f64| {
        if y <= mids[0] {
            return one("pelvis");
        }
        if y >= mids[3] {
            return one("spine_03");
        }
        let i = mids.partition_point(|&m| m < y) - 1;
        let t = (y - mids[i]) / (mids[i + 1] - mids[i]);
        vec![(SPINE[i], 1.0 - t), (SPINE[i + 1], t)]
    }
}

/// The numbers the cage is built from (metres), per family and stage.
pub(crate) struct Shape {
    pub(crate) h: f64,
    pub(crate) female: bool,
    pub(crate) yg: f64,
    /// How grown the body is (`maturity`): how far it shows the families' grown differences.
    pub(crate) m: f64,
    pub(crate) r_th: f64,
    pub(crate) r_a: f64,
    pub(crate) r_n: f64,
    pub(crate) waist: f64,
    pub(crate) belly: f64,
}
