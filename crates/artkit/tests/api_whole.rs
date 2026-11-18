//! Numbers a script wrote whole in one row and as floats in the next, through `place::N`, against Python and Blender
//! 5.2 (tests/golden/api_whole.txt, written by a script outside the repo that calls gamekit with the numbers as
//! they stand here): `F.at` under three frames, a box, a cylinder, a cube, a sweep, a tube and a filleted path through
//! such points, the names an f-string gives them, and Python's sums. A whole zero negates to a plain zero and a float
//! zero to a negative one: the parts' places and points are Blender's to the bit only if each number is held as it
//! was written.
mod common;

use artkit::materials::Mat;
use artkit::math::P3;
use artkit::place::{Coord, GameFrame, N};
use artkit::scene::{self, Obj};
use artkit::{cube, fbox, fcyl, fsweep, kit, points};
use common::*;

fn mat() -> Mat {
    Mat::new("craft_metal", "#2B2C2E", "metal")
}

/// A number as the golden writes it, which is as the script wrote it: `0` whole, `0.0` and `1e-05` not.
fn n(word: &str) -> N {
    match word.parse::<i32>() {
        Ok(i) => N::I(i),
        Err(_) => N::F(word.parse().unwrap()),
    }
}

fn point(word: &str) -> (N, N, N) {
    let c: Vec<N> = word.split(',').map(n).collect();
    (c[0], c[1], c[2])
}

fn frame(name: &str) -> GameFrame {
    match name {
        "plain" => GameFrame::new(0.0),
        "shift" => GameFrame::new(0.094),
        "depths" => GameFrame::with_depths(0.1, &[(-0.4, -0.3), (0.4, 0.2)]),
        other => panic!("no frame {other:?}"),
    }
}

fn hex64(words: &str) -> Vec<u64> {
    words
        .split([' ', ','])
        .map(|w| u64::from_str_radix(w, 16).unwrap())
        .collect()
}

/// The part's place and points against the two lines the golden has after a call.
fn same(what: &str, obj: Obj, loc: &str, verts: &str) {
    let want: Vec<f32> = f32s(&loc[4..].split(' ').collect::<Vec<_>>());
    let l = obj.location();
    assert_eq!(bits(&[l.x, l.y, l.z]), bits(&want), "{what}: its place");
    let flat = f32s(&verts[2..].split(' ').collect::<Vec<_>>());
    let want: Vec<[f32; 3]> = flat.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
    let got = obj.read(|o| o.mesh.verts.clone());
    assert_eq!(got.len(), want.len(), "{what}: its points");
    assert_eq!(points_off(&got, &want), (0, 0), "{what}: its points");
}

#[test]
fn a_number_goes_through_the_frame_as_python_wrote_it() {
    let golden = golden("api_whole.txt");
    let mut seen = 0;
    for line in golden.lines().filter(|l| l.starts_with("at ")) {
        let (call, want) = line.split_once(" = ").unwrap();
        let words: Vec<&str> = call.split(' ').collect();
        let (x, y, z) = point(words[2]);
        let p = frame(words[1]).at(x, y, z);
        assert_eq!(
            [p.x.to_bits(), p.y.to_bits(), p.z.to_bits()],
            hex64(want)[..],
            "{call}"
        );
        seen += 1;
    }
    assert_eq!(seen, 30);
    // the same as the bare numbers give
    const F: GameFrame = GameFrame::new(0.0);
    let bare = [F.at(0, 0.5, 0), F.at(0.0, 0.5, 0.0)];
    let held = [
        F.at(N::I(0), N::F(0.5), N::I(0)),
        F.at(N::F(0.0), N::F(0.5), N::F(0.0)),
    ];
    for (a, b) in bare.iter().zip(&held) {
        assert_eq!(
            [a.x.to_bits(), a.y.to_bits(), a.z.to_bits()],
            [b.x.to_bits(), b.y.to_bits(), b.z.to_bits()]
        );
    }
    assert!(N::I(0).negated().is_sign_positive() && N::F(0.0).negated().is_sign_negative());
    assert!(N::from(0).is_whole() && !N::from(0.0).is_whole());
}

#[test]
fn its_names_and_sums_are_pythons() {
    let golden = golden("api_whole.txt");
    let (mut names, mut sums) = (0, 0);
    for line in golden.lines() {
        let (call, want) = match line.split_once(" = ") {
            Some(parts) => parts,
            None => continue,
        };
        let words: Vec<&str> = call.split(' ').collect();
        match words[0] {
            "str" => {
                assert_eq!(format!("bar{}", n(words[1])), want, "{line}");
                names += 1;
            }
            "neg" => {
                let got = -n(words[1]);
                assert_eq!(got.to_string(), want, "{line}");
                assert_eq!(got.is_whole(), n(want).is_whole(), "{line}");
                names += 1;
            }
            "op" => {
                let (a, b) = (n(words[1]), n(words[3]));
                let got = match words[2] {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => a / b,
                    other => panic!("no sum {other:?}"),
                };
                assert_eq!(got.to_string(), want, "{line}");
                assert_eq!(got.is_whole(), n(want).is_whole(), "{line}");
                sums += 1;
            }
            _ => {}
        }
    }
    assert_eq!(names, 24);
    assert!(sums > 500, "{sums}");
    // a bare number on the right, as a script multiplied a row's place by a side
    assert!((N::I(3) * -1).is_whole() && (N::I(3) * -1 == N::I(-3)));
    assert_eq!((N::F(0.47) * -1).to_string(), "-0.47");
    assert_eq!((N::I(1) + 0.5).to_string(), "1.5");
    // Python's `==`: by value
    assert!(N::I(2) == N::F(2.0) && N::I(0) < N::F(0.5));
}

#[test]
fn parts_built_through_it_are_blenders() {
    const F: GameFrame = GameFrame::new(0.0);
    const S: GameFrame = GameFrame::new(0.094);
    let golden = golden("api_whole.txt");
    let lines: Vec<&str> = golden.lines().collect();
    let mut seen = [0; 7];
    let mut posts = 0;
    for (i, line) in lines.iter().enumerate() {
        let words: Vec<&str> = line.split(' ').collect();
        scene::reset();
        match words[0] {
            "box" => {
                let (w, d, at) = (n(words[1]), n(words[3]), point(words[4]));
                let name = format!("bar{}{}", at.0, at.2);
                assert_eq!(format!("name {name}"), lines[i + 3]);
                let b = fbox!(F, &name, w, 0.02, d, at, mat());
                same(line, b, lines[i + 1], lines[i + 2]);
                seen[0] += 1;
            }
            "cyl" => {
                let (h, at) = (n(words[2]), point(words[3]));
                let c = fcyl!(S, format!("post{posts}"), 0.03, h, at, mat(), verts = 6);
                same(line, c, lines[i + 1], lines[i + 2]);
                posts += 1;
                seen[1] += 1;
            }
            "cube" => {
                let (size, at) = (point(words[1]), point(words[2]));
                // (Blender's axes: nothing is negated, and a point is one of floats)
                let c = cube!("cube", size, at = at, material = mat());
                same(line, c, lines[i + 1], lines[i + 2]);
                seen[2] += 1;
            }
            "sweep" | "tube" | "bent" | "fillet" => {
                let path: Vec<(N, N, N)> = words[1..].iter().map(|w| point(w)).collect();
                match words[0] {
                    "sweep" => {
                        let s = fsweep!(S, "sweep", &path, 0.012, mat(), verts = 6, resolution = 1);
                        same(line, s, lines[i + 1], lines[i + 2]);
                        seen[3] += 1;
                    }
                    "tube" => {
                        let t = kit::tube!(F, "tube", &path, 0.012, mat());
                        same(line, t, lines[i + 1], lines[i + 2]);
                        seen[4] += 1;
                    }
                    "bent" => {
                        let bent = kit::fillet!(&path, 0.006);
                        let t = kit::tube!(S, "bent", &bent, 0.0055, mat(), verts = 6, round = 1);
                        same(line, t, lines[i + 1], lines[i + 2]);
                        seen[5] += 1;
                    }
                    _ => {
                        let bent = kit::fillet!(&path, 0.05);
                        let got: Vec<u64> = bent
                            .iter()
                            .flat_map(|p| [p.0.to_bits(), p.1.to_bits(), p.2.to_bits()])
                            .collect();
                        assert_eq!(got, hex64(&lines[i + 1][3..]), "{line}");
                        seen[6] += 1;
                    }
                }
            }
            _ => {}
        }
    }
    assert_eq!(seen, [6, 6, 6, 3, 3, 3, 3]);
}

#[test]
fn a_list_of_points_is_written_as_the_script_wrote_it() {
    const F: GameFrame = GameFrame::new(0.0);
    let golden = golden("api_whole.txt");
    let lines: Vec<&str> = golden.lines().collect();
    let at = lines
        .iter()
        .position(|l| *l == "tube 0,0.2,0.0 0,0.8,0 0.3,0.9,0")
        .unwrap();
    let path = points![(0, 0.2, 0.0), (0, 0.8, 0), (0.3, 0.9, 0)];
    let whole: Vec<[bool; 3]> = path
        .iter()
        .map(|p| [p.0.is_whole(), p.1.is_whole(), p.2.is_whole()])
        .collect();
    assert_eq!(
        whole,
        [
            [true, false, false],
            [true, false, true],
            [false, false, true]
        ]
    );
    scene::reset();
    let t = kit::tube!(F, "tube", &path, 0.012, mat());
    same(lines[at], t, lines[at + 1], lines[at + 2]);
    // two numbers a point: an outline, a row of a table
    let flat = points![(0, 0.47), (0.47, 0)];
    assert!(flat[0].0.is_whole() && !flat[0].1.is_whole() && flat[1].1.is_whole());
    // and where a point of floats is asked for, it is one
    let p: P3 = (N::I(1), N::F(0.5), N::I(0)).into();
    assert_eq!((p.x, p.y, p.z), (1.0, 0.5, 0.0));
}
