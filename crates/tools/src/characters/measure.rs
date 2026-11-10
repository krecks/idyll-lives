//! `cargo characters ratios`: the proportions of every body as built, beside a real adult's, to tune
//! `rig.rs` and the hands and feet by (docs/review/residents.md, M-residents-01). Reads the generator,
//! not a built set, so it shows a change before the build.

use super::body::{self, region};
use super::head::head_frame;
use super::rig::{FAMILIES, STAGES, b, proportions};
use super::v3;

/// A real adult's numbers, for the table's last line (anthropometric averages: a head of 1/7.5 of
/// the height, a hand of 0.108 and a foot of 0.152 of it, the foot 0.055 wide; a 24 mm eyeball with
/// a 12 mm iris, the pupils 63 mm apart).
const REAL: [f64; 8] = [7.5, 24.0, 12.0, 63.0, 0.108, 0.152, 0.055, 0.0];

/// One body's numbers: heads in the height, the eyeball and iris across and the eyes apart (mm), the
/// hand's and the foot's length and the foot's breadth as shares of the height, and the height (m).
pub fn row(family: &str, stage: &str) -> [f64; 8] {
    let p = proportions(family, stage);
    let h = head_frame(family, stage);
    let s = body::surface(family, stage);
    let height = s.pos.iter().map(|v| v.y).fold(0.0, f64::max);
    let of = |name: &str| {
        let r = region(name);
        s.pos
            .iter()
            .zip(&s.region)
            // The left side (-x), where the vertex is mostly that region.
            .filter(move |(v, w)| v.x < 0.0 && w[r] > 0.5)
            .map(|(v, _)| *v)
    };
    // The hand along the arm (it hangs 45 degrees out at rest), from the wrist to the fingertips.
    let wrist = s.p[b("hand_l")];
    let along = v3(-(0.5f64).sqrt(), -(0.5f64).sqrt(), 0.0);
    let hand = of("hand")
        .map(|v| (v - wrist).dot(&along))
        .fold(0.0, f64::max);
    let span = |it: &mut dyn Iterator<Item = f64>| {
        let (lo, hi) = it.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
            (lo.min(x), hi.max(x))
        });
        (hi - lo).max(0.0)
    };
    let foot_len = span(&mut of("foot").map(|v| v.z));
    let foot_wide = span(&mut of("foot").map(|v| v.x));
    [
        1.0 / p.hh,
        h.eye_r * 2000.0,
        h.iris_r * 2000.0,
        (h.eyes[1].x - h.eyes[0].x).abs() * 1000.0,
        hand / height,
        foot_len / height,
        foot_wide / height,
        height,
    ]
}

/// Prints the table.
pub fn print() {
    println!(
        "{:<18} {:>6} {:>7} {:>6} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "body", "heads", "eye mm", "iris", "apart", "hand/H", "foot/H", "wide/H", "H m"
    );
    let line = |name: &str, r: [f64; 8]| {
        println!(
            "{:<18} {:>6.2} {:>7.1} {:>6.1} {:>7.1} {:>7.3} {:>7.3} {:>7.3} {:>7.3}",
            name, r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]
        );
    };
    for family in FAMILIES {
        for stage in STAGES {
            line(&format!("{family}.{stage}"), row(family, stage));
        }
    }
    line("a real adult", REAL);
}
