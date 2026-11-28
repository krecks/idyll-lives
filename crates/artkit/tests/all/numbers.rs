//! The kernel's numbers against the programs the family scripts ran in: CPython's `random`, and Blender 5.2's
//! noise and `mathutils` (captured from the real ones: tests/golden/random.txt, noise.txt, mathutils.txt).

use crate::common::*;
use artkit::math::{Axis, M3, M4, Quat, V3};
use artkit::noise::noise;
use artkit::rand::{Random, float_hash};

#[test]
fn random_draws_what_cpython_draws() {
    let mut floats = 0;
    for line in golden("random.txt").lines() {
        let w: Vec<&str> = line.split(' ').collect();
        if w[0] == "float" {
            // a float seed (as Python writes it, and its bits), its hash, the first 8 random()
            let seed = f64w(w[2]);
            assert_eq!(w[1].parse::<f64>().unwrap().to_bits(), seed.to_bits());
            assert_eq!(
                float_hash(seed),
                w[3].parse::<i64>().unwrap(),
                "hash({seed})"
            );
            let mut r = Random::from_float(seed);
            for x in f64s(&w[4..12]) {
                assert_eq!(r.random().to_bits(), x.to_bits(), "seed {seed}");
            }
            floats += 1;
            continue;
        }
        if w[0] == "long" {
            let mut r = Random::new(w[1].parse().unwrap());
            for _ in 0..1000 {
                r.random();
            }
            for x in f64s(&w[2..]) {
                assert_eq!(r.random().to_bits(), x.to_bits());
            }
            continue;
        }
        let seed: i128 = w[1].parse().unwrap();
        let mut r = Random::new(seed);
        let mut at = 2;
        let mut take = |name: &str, n: usize| {
            assert_eq!(w[at], name);
            at += 1 + n;
            &w[at - n..at]
        };
        for x in f64s(take("random", 8)) {
            assert_eq!(r.random().to_bits(), x.to_bits(), "seed {seed}");
        }
        let u = f64s(take("uniform", 6));
        for x in &u[..4] {
            assert_eq!(r.uniform(-0.3, 0.3).to_bits(), x.to_bits());
        }
        assert_eq!(r.uniform(3.6, 4.4).to_bits(), u[4].to_bits());
        assert_eq!(r.uniform(0.9, 0.1).to_bits(), u[5].to_bits());
        let bits = take("getrandbits", 7);
        for (k, x) in [1, 7, 31, 32, 33, 53, 64].into_iter().zip(bits) {
            assert_eq!(
                r.getrandbits(k),
                x.parse::<u64>().unwrap(),
                "getrandbits({k})"
            );
        }
        let rr: Vec<i64> = take("randrange", 6)
            .iter()
            .map(|x| x.parse().unwrap())
            .collect();
        assert_eq!(r.randrange(2), rr[0]);
        assert_eq!(r.randrange(5), rr[1]);
        assert_eq!(r.randrange_step(3, 17, 1), rr[2]);
        assert_eq!(r.randrange_step(0, 100, 7), rr[3]);
        assert_eq!(r.randrange(1000000007), rr[4]);
        assert_eq!(r.randrange(1 << 40), rr[5]);
        let ri: Vec<i64> = take("randint", 3)
            .iter()
            .map(|x| x.parse().unwrap())
            .collect();
        assert_eq!(r.randint(0, 1), ri[0]);
        assert_eq!(r.randint(-5, 5), ri[1]);
        assert_eq!(r.randint(1, 6), ri[2]);
        let ch: Vec<usize> = take("choice", 5)
            .iter()
            .map(|x| x.parse().unwrap())
            .collect();
        for (n, x) in [1, 2, 3, 7, 100].into_iter().zip(ch) {
            assert_eq!(r.index(n), x);
        }
        for x in f64s(take("random", 3)) {
            assert_eq!(r.random().to_bits(), x.to_bits());
        }
    }
    assert_eq!(floats, 69, "the float seeds checked");
}

#[test]
#[should_panic(expected = "hash(nan)")]
fn a_nan_seeds_nothing() {
    Random::from_float(f64::NAN);
}

#[test]
fn noise_is_blenders_to_the_bit() {
    let text = golden("noise.txt");
    assert_eq!(text.lines().count(), 4096);
    for (i, line) in text.lines().enumerate() {
        let w: Vec<&str> = line.split(' ').collect();
        let p = f32s(&w[..3]);
        let got = noise(V3::new(p[0], p[1], p[2]));
        assert_eq!(
            got.to_bits(),
            f64w(w[3]).to_bits(),
            "sample {i} at {p:?}: {got} for {}",
            f64w(w[3])
        );
    }
}

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
fn m3(w: &[f32]) -> M3 {
    let mut m = [[0.0; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            m[c][r] = w[r * 3 + c];
        }
    }
    M3(m)
}
fn m4(w: &[f32]) -> M4 {
    let mut m = [[0.0; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            m[c][r] = w[r * 4 + c];
        }
    }
    M4(m)
}

/// What an operation gives for a golden line's inputs, as single-precision words or one double.
enum Out {
    F(Vec<f32>),
    D(f64),
    Skip,
}

fn run(op: &str, w: &[&str]) -> Out {
    use Out::*;
    let f = |a: usize, b: usize| f32s(&w[a..b]);
    let d = |a: usize| f64w(w[a]);
    match op {
        "vadd" => F((v(&f(0, 3)) + v(&f(3, 6))).array().to_vec()),
        "vsub" => F((v(&f(0, 3)) - v(&f(3, 6))).array().to_vec()),
        "vneg" => F((-v(&f(0, 3))).array().to_vec()),
        "vmuls" => F((v(&f(0, 3)) * d(3)).array().to_vec()),
        "smulv" => F((d(3) * v(&f(0, 3))).array().to_vec()),
        "vdivs" => F((v(&f(0, 3)) / d(3)).array().to_vec()),
        "vdot" => D(v(&f(0, 3)).dot(v(&f(3, 6)))),
        "vcross" => F(v(&f(0, 3)).cross(v(&f(3, 6))).array().to_vec()),
        "vlen" => D(v(&f(0, 3)).length()),
        "vlen2" => D(v(&f(0, 3)).xy_length()),
        "vnorm" => F(v(&f(0, 3)).normalized().array().to_vec()),
        "vlerp" => F(v(&f(0, 3)).lerp(v(&f(3, 6)), d(6)).array().to_vec()),
        "vorth" => F(v(&f(0, 3)).orthogonal().array().to_vec()),
        "vangle" => D(v(&f(0, 3)).angle(v(&f(3, 6)))),
        "rotdiff" => F(v(&f(0, 3))
            .rotation_difference(v(&f(3, 6)))
            .array()
            .to_vec()),
        "trackzy" => F(v(&f(0, 3)).to_track_quat_zy().array().to_vec()),
        "qmulv" => F((q(&f(0, 4)) * v(&f(4, 7))).array().to_vec()),
        "q2m" => F(q(&f(0, 4)).to_matrix().rows().to_vec()),
        "m3mulv" => F((m3(&f(0, 9)) * v(&f(9, 12))).array().to_vec()),
        "m3mulm3" => F((m3(&f(0, 9)) * m3(&f(9, 18))).rows().to_vec()),
        "mrot3X" => F(M3::rotation(d(0), Axis::X).rows().to_vec()),
        "mrot3Y" => F(M3::rotation(d(0), Axis::Y).rows().to_vec()),
        "mrot3Z" => F(M3::rotation(d(0), Axis::Z).rows().to_vec()),
        "mrot4X" => F(M4::rotation(d(0), Axis::X).rows().to_vec()),
        "mrot4Y" => F(M4::rotation(d(0), Axis::Y).rows().to_vec()),
        "mrot4Z" => F(M4::rotation(d(0), Axis::Z).rows().to_vec()),
        "m4mulv" => F((m4(&f(0, 16)) * v(&f(16, 19))).array().to_vec()),
        "m4mulm4" => F((m4(&f(0, 16)) * m4(&f(16, 32))).rows().to_vec()),
        "m4inv" => F(m4(&f(0, 16)).inverted().unwrap().rows().to_vec()),
        "obmat" => {
            F(
                artkit::math::object_matrix_of(v(&f(0, 3)), [d(3), d(4), d(5)], [d(6), d(7), d(8)])
                    .rows()
                    .to_vec(),
            )
        }
        _ => Skip,
    }
}

#[test]
fn vectors_and_matrices_are_mathutils_to_the_bit() {
    use std::collections::BTreeMap;
    // per operation: (lines, lines off, the farthest off in units in the last place)
    let mut seen: BTreeMap<String, (usize, usize, u64)> = BTreeMap::new();
    for line in golden("mathutils.txt").lines() {
        let (ins, outs) = line.split_once(" | ").unwrap();
        let w: Vec<&str> = ins.split(' ').collect();
        let want: Vec<&str> = outs.split(' ').collect();
        let off = match run(w[0], &w[1..]) {
            Out::Skip => continue,
            Out::D(x) => (x.to_bits() as i64 - f64w(want[0]).to_bits() as i64).unsigned_abs(),
            Out::F(got) => got
                .iter()
                .zip(f32s(&want))
                .map(|(a, b)| ulps(*a, b))
                .max()
                .unwrap(),
        };
        if off > 0 && std::env::var("SHOW").is_ok_and(|s| s == w[0]) {
            println!("{line}");
            if let Out::F(got) = run(w[0], &w[1..]) {
                println!("  got {:?}\n want {:?}", got, f32s(&want));
            }
        }
        let e = seen.entry(w[0].to_string()).or_default();
        e.0 += 1;
        e.1 += (off > 0) as usize;
        e.2 = e.2.max(off);
    }
    let report: Vec<String> = seen
        .iter()
        .map(|(k, (n, bad, far))| format!("{k}: {bad}/{n} off, by up to {far}"))
        .collect();
    println!("{}", report.join("\n"));
    // (A turn between two directions and a tracking turn go through the C library's single-precision arc sine,
    // arc cosine and arc tangent, which are Apple's in the goldens and transcribed from it here.)
    let bad: Vec<&String> = seen
        .iter()
        .filter(|(_, e)| e.1 > 0)
        .map(|(k, _)| k)
        .collect();
    assert!(
        bad.is_empty(),
        "not to the bit: {bad:?}\n{}",
        report.join("\n")
    );
    assert!(seen.len() >= 30, "{} operations checked", seen.len());
}
