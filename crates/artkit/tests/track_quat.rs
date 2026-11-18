//! A tracking turn, a turn about an axis and the product of two against Blender 5.2's mathutils, to the bit
//! (tests/golden/track_quat.txt: `v.to_track_quat(track, up)`, `Quaternion(axis, angle)` and `q @ r` as the heavy
//! bag's chains call them in pack_hotHeaded.py, each link's object matrix from its `rotation_quaternion`, then a
//! spread: every track and up of X, Y and Z, directions along the axes and within the 1e-4 of them, axes that are
//! not unit vectors, angles of every quarter, past a turn and negative).
mod common;

use artkit::math::{Axis, Quat, V3, object_matrix_quat};
use common::*;
use std::collections::BTreeMap;

fn v(w: &[f32]) -> V3 {
    V3::new(w[0], w[1], w[2])
}
fn q(w: &[f32]) -> Quat {
    Quat {
        w: w[0],
        x: w[1],
        y: w[2],
        z: w[3],
    }
}
fn axis(c: u8) -> Axis {
    match c {
        b'X' => Axis::X,
        b'Y' => Axis::Y,
        _ => Axis::Z,
    }
}

#[test]
fn tracking_turns_are_mathutils_to_the_bit() {
    // per operation: (lines, lines off, the farthest off in units in the last place)
    let mut seen: BTreeMap<String, (usize, usize, u64)> = BTreeMap::new();
    let mut shown = 0;
    for line in golden("track_quat.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let (ins, outs) = line.split_once(" | ").unwrap();
        let w: Vec<&str> = ins.split(' ').collect();
        let want = f32s(&outs.split(' ').collect::<Vec<_>>());
        let (op, got) = match w[0] {
            "track" => {
                let pair = w[1].as_bytes();
                let got = v(&f32s(&w[2..5])).to_track_quat(axis(pair[0]), axis(pair[1]));
                (format!("track {}", w[1]), got.array().to_vec())
            }
            "turn" => {
                let got = Quat::from_axis_angle(v(&f32s(&w[1..4])), f64w(w[4]));
                ("turn".to_string(), got.array().to_vec())
            }
            "qmul" => {
                let got = q(&f32s(&w[1..5])) * q(&f32s(&w[5..9]));
                ("qmul".to_string(), got.array().to_vec())
            }
            "basis" => {
                let f = f32s(&w[1..11]);
                let got = object_matrix_quat(v(&f[..3]), q(&f[3..7]), v(&f[7..]));
                ("basis".to_string(), got.rows().to_vec())
            }
            other => panic!("{other}?"),
        };
        let off = got
            .iter()
            .zip(&want)
            .map(|(a, b)| ulps(*a, *b))
            .max()
            .unwrap();
        if off > 0 && shown < 12 {
            shown += 1;
            println!("{line}\n  got {:08x?}", bits(&got));
        }
        let e = seen.entry(op).or_default();
        e.0 += 1;
        e.1 += (off > 0) as usize;
        e.2 = e.2.max(off);
    }
    let report: Vec<String> = seen
        .iter()
        .map(|(k, (n, bad, far))| format!("{k}: {bad}/{n} off, by up to {far}"))
        .collect();
    println!("{}", report.join("\n"));
    assert!(
        seen.values().all(|e| e.1 == 0),
        "not to the bit:\n{}",
        report.join("\n")
    );
    let lines = |k: &str| seen[k].0;
    assert!(lines("track XZ") >= 500 && lines("track ZY") >= 500);
    assert!(lines("turn") >= 1000 && lines("qmul") >= 150 && lines("basis") >= 20);
    assert_eq!(seen.len(), 9, "the six tracking pairs and three more");
}
