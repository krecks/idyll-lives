//! `cargo characters ratios`: the proportions of every body as built, and its ear, beside a real adult's,
//! to tune `rig.rs`, the hands and feet and the ears (head.rs) by (docs/review/residents.md, M-residents-01). Reads the generator,
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
    println!();
    println!(
        "{:<18} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "ear", "high mm", "wide", "out", "top/hh", "low/hh", "eye/hh", "turned"
    );
    for family in FAMILIES {
        for stage in STAGES {
            let e = ear(family, stage);
            println!(
                "{:<18} {:>7.1} {:>7.1} {:>7.1} {:>7.3} {:>7.3} {:>7.3} {:>7.0}",
                format!("{family}.{stage}"),
                e[0],
                e[1],
                e[2],
                e[3],
                e[4],
                e[5],
                e[6]
            );
        }
    }
    println!(
        "{:<18} {:>7.1} {:>7.1} {:>7.1} {:>7} {:>7} {:>7} {:>7}",
        "a real adult", 62.0, 34.0, 19.0, "brow", "nose", "", 0
    );
}

/// One body's right ear as drawn: how high and how wide (front to back) it is and how far it stands off
/// the head beside it (mm), its top, its bottom and the eyes above the chin (shares of the head's height),
/// and how many corners of its triangles are shaded as facing away from the triangle (the skin folded
/// over itself there: none, on an ear that is sound).
pub fn ear(family: &str, stage: &str) -> [f64; 7] {
    let h = head_frame(family, stage);
    let s = body::surface(family, stage);
    let r = region("ear");
    let own: Vec<_> = s
        .pos
        .iter()
        .zip(&s.region)
        .filter(|(v, w)| v.x > 0.0 && w[r] > 0.5)
        .map(|(v, _)| *v)
        .collect();
    let span = |k: usize| {
        own.iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v[k]), hi.max(v[k]))
            })
    };
    let (y, z) = (span(1), span(2));
    let mut turned = 0;
    for t in body::tris(family).chunks(3) {
        if !t.iter().any(|&i| s.region[i][r] > 0.05 && s.pos[i].x > 0.0) {
            continue;
        }
        let face = (s.pos[t[1]] - s.pos[t[0]]).cross(&(s.pos[t[2]] - s.pos[t[0]]));
        turned += t.iter().filter(|&&i| s.nor[i].dot(&face) < 0.0).count();
    }
    // (The head beside the ear: its skin without the ears.)
    let out = own
        .iter()
        .map(|&v| h.sdf(v))
        .fold(f64::NEG_INFINITY, f64::max);
    [
        (y.1 - y.0) * 1000.0,
        (z.1 - z.0) * 1000.0,
        out * 1000.0,
        (y.1 - h.chin_y) / h.hh,
        (y.0 - h.chin_y) / h.hh,
        (h.eyes[1].y - h.chin_y) / h.hh,
        turned as f64,
    ]
}

/// `cargo characters faces`: every identity channel (identity.rs) as the generator builds it, per family:
/// how far it moves the skin and each of the face's parts at most (mm, at 1), how many of the skin's
/// vertices it moves, and the eyeballs (mm, and their size in per cent).
pub fn print_faces() {
    for family in FAMILIES {
        println!(
            "{family:<22} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}  eyes",
            "skin", "verts", "lids", "lips", "lashes", "brows", "tongue", "teeth"
        );
        for ch in super::identity::channels(family).iter() {
            let mm = |d: &[super::V3]| d.iter().map(|v| v.norm()).fold(0.0, f64::max) * 1000.0;
            let moved = ch.skin.0.iter().filter(|v| v.norm() > 0.0).count();
            let part = |name: &str| mm(&ch.parts.iter().find(|p| p.0 == name).expect("part").1);
            println!(
                "  {:<20} {:>6.2} {:>6} {:>6.2} {:>6.2} {:>6.2} {:>6.2} {:>6.2} {:>6.2}  {:.2} {:.2} {:.2} mm, {:+.1} %",
                ch.name,
                mm(&ch.skin.0),
                moved,
                mm(&ch.parts[0].1[..ch.lids]),
                mm(&ch.parts[0].1[ch.lids..]),
                part("lashes"),
                part("brows"),
                part("tongue"),
                part("teeth"),
                ch.eye[0] * 1000.0,
                ch.eye[1] * 1000.0,
                ch.eye[2] * 1000.0,
                ch.eye[3] * 100.0
            );
        }
    }
}
