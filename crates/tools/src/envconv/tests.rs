use super::*;

fn dirs() -> Vec<[f64; 3]> {
    // A deterministic spread over the sphere (a Fibonacci spiral).
    (0..200)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f64 + 0.5) / 200.0;
            let r = (1.0 - y * y).sqrt();
            let a = i as f64 * 2.399963229728653;
            [r * a.cos(), y, r * a.sin()]
        })
        .collect()
}

fn close(a: [f64; 3], b: [f64; 3], eps: f64) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < eps)
}

fn cos(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A Radiance file of `w` × `h` texels, `px(x, y)` each (RGBE bytes), its scanlines run-length
/// encoded or flat.
fn hdr_file(w: usize, h: usize, rle: bool, px: impl Fn(usize, usize) -> [u8; 4]) -> Vec<u8> {
    let mut out = format!("#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y {h} +X {w}\n").into_bytes();
    for y in 0..h {
        if !rle {
            (0..w).for_each(|x| out.extend_from_slice(&px(x, y)));
            continue;
        }
        out.extend_from_slice(&[2, 2, (w >> 8) as u8, w as u8]);
        for ch in 0..4 {
            let row: Vec<u8> = (0..w).map(|x| px(x, y)[ch]).collect();
            // Runs of equal bytes as runs, the rest as literals (at most 127 each).
            let mut x = 0;
            while x < w {
                let run = row[x..]
                    .iter()
                    .take(127)
                    .take_while(|&&b| b == row[x])
                    .count();
                if run >= 3 {
                    out.extend_from_slice(&[128 + run as u8, row[x]]);
                    x += run;
                } else {
                    let n = (w - x)
                        .min(127)
                        .min(if run == 1 { usize::MAX } else { run });
                    out.push(n as u8);
                    out.extend_from_slice(&row[x..x + n]);
                    x += n;
                }
            }
        }
    }
    out
}

/// A panorama, its top half (the sky) bright and blue, its bottom half dark and green, and one
/// bright texel (16) a little right of the image's middle, above the horizon.
fn test_pano(w: usize, h: usize) -> impl Fn(usize, usize) -> [u8; 4] {
    move |x, y| {
        if x == w / 2 + w / 16 && y == h / 4 {
            [128, 128, 128, 133]
        } else if y < h / 2 {
            [100, 140, 200, 129]
        } else {
            [60, 90, 40, 127]
        }
    }
}

#[test]
fn hdr_files_read_and_bad_ones_fail() {
    let (w, h) = (64, 32);
    let px = test_pano(w, h);
    let flat = read_hdr(&hdr_file(w, h, false, &px)).unwrap();
    let rle = read_hdr(&hdr_file(w, h, true, &px)).unwrap();
    assert_eq!((rle.width, rle.height), (w, h));
    assert_eq!(flat.texels, rle.texels);
    // rgb · 2^(e − 136): 100 · 2^-7 for the sky.
    assert_eq!(rle.texels[0], [100.0 / 128.0, 140.0 / 128.0, 200.0 / 128.0]);
    assert_eq!(
        rle.texels[(h - 1) * w],
        [60.0 / 512.0, 90.0 / 512.0, 40.0 / 512.0]
    );

    let good = hdr_file(w, h, true, &px);
    let header_end = good.windows(2).position(|p| p == b"\n\n").unwrap() + 2;
    let line_end = header_end + good[header_end..].iter().position(|&b| b == b'\n').unwrap() + 1;
    let mut bad_run = good.clone();
    bad_run[line_end + 4] = 255; // a run of 127 in a 64-texel scanline
    let cases: [(&str, Vec<u8>); 8] = [
        ("empty", Vec::new()),
        ("not radiance", b"P6\n1 1\n255\n".to_vec()),
        ("no format", b"#?RADIANCE\n\n-Y 2 +X 2\n".to_vec()),
        (
            "xyze",
            b"#?RADIANCE\nFORMAT=32-bit_rle_xyze\n\n-Y 2 +X 2\n".to_vec(),
        ),
        (
            "flipped",
            b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n+Y 2 +X 2\n".to_vec(),
        ),
        (
            "huge",
            b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y 40000 +X 40000\n".to_vec(),
        ),
        ("cut short", good[..good.len() - 7].to_vec()),
        ("long run", bad_run),
    ];
    for (what, bytes) in cases {
        assert!(read_hdr(&bytes).is_err(), "{what}");
    }
}

/// The panorama's sky comes out on the cube's +y face and in the polynomial's +y, its bright
/// texel where Babylon's mapping puts it, and a sky of one colour is that colour (and a
/// polynomial of that constant: xx = yy = zz, the rest 0, as near as Babylon's rounded
/// constants come).
#[test]
fn babylon_cube_and_polynomial() {
    let (w, h) = (256, 128);
    let pano = read_hdr(&hdr_file(w, h, true, test_pano(w, h))).unwrap();
    let sky = babylon_sky(&pano);
    assert_eq!(sky.cube.size, BABYLON_SIZE);
    let share = |f: usize, c: [f32; 3]| {
        sky.cube.faces[f].iter().filter(|&&t| t == c).count() as f64
            / sky.cube.faces[f].len() as f64
    };
    assert!(
        share(2, [100.0 / 128.0, 140.0 / 128.0, 200.0 / 128.0]) > 0.99
            && share(3, [60.0 / 512.0, 90.0 / 512.0, 40.0 / 512.0]) == 1.0
    );
    let p = sky.polynomial;
    assert!(
        p[1][2] > 0.3 && p[0][2].abs() < 0.01 && p[2][2].abs() < 0.01,
        "{p:?}"
    );
    // The bright texel: in the panorama at u = 9/16, a quarter down. Babylon's
    // `CalcProjectionSpherical` reads the image upside down (`invertY`), so it is at
    // θ = atan2(z, x) = π/8 and φ = acos(y) ≈ 3π/4 of the faces' corner vectors, and the faces
    // went to the GPU as Babylon named them (its "right", corners at z = 1, is +x; its "up",
    // at y = -1, is +y; its "front", at x = 1, is +z): the GPU's (z, -y, x) of those.
    let (theta, phi) = (PI / 8.0, (h - 1 - h / 4) as f64 / h as f64 * PI);
    let corner = [theta.cos() * phi.sin(), phi.cos(), theta.sin() * phi.sin()];
    let want = [corner[2], -corner[1], corner[0]];
    assert!(want[1] > 0.6, "above the horizon");
    let (mut best, mut at) = (0.0, [0.0; 3]);
    for (f, face) in sky.cube.faces.iter().enumerate() {
        for (i, c) in face.iter().enumerate() {
            if c[0] as f64 > best {
                best = c[0] as f64;
                // (Babylon samples a texel at its corner.)
                let (x, y) = (i % BABYLON_SIZE, i / BABYLON_SIZE);
                at = face_dir(
                    f,
                    x as f64 / BABYLON_SIZE as f64 * 2.0 - 1.0,
                    y as f64 / BABYLON_SIZE as f64 * 2.0 - 1.0,
                );
            }
        }
    }
    assert!(
        best == 16.0 && cos(at, want) > 0.999,
        "{best} at {at:?}, want {want:?}"
    );

    let one = Panorama {
        width: 64,
        height: 32,
        texels: vec![[0.5, 1.0, 2.0]; 64 * 32],
    };
    let flat = babylon_sky(&one);
    assert!(
        flat.cube
            .faces
            .iter()
            .flatten()
            .all(|c| *c == [0.5, 1.0, 2.0])
    );
    for ch in 0..3 {
        let v = [0.5, 1.0, 2.0][ch];
        for i in 0..9 {
            let want = if (3..6).contains(&i) { v } else { 0.0 };
            assert!(
                (flat.polynomial[i][ch] - want).abs() < 1e-4 * v,
                "{i} {ch}: {:?}",
                flat.polynomial
            );
        }
    }
}

#[test]
fn faces_and_directions_round_trip() {
    for d in dirs() {
        let (f, s, t) = dir_face(d);
        assert!(close(face_dir(f, s, t), d, 1e-12), "{d:?}");
    }
    // The GPU's convention: +x's s runs along -z, +y's t along +z, +z's s along +x.
    assert!(close(
        face_dir(0, 1.0, 0.0),
        normalize([1.0, 0.0, -1.0]),
        1e-12
    ));
    assert!(close(
        face_dir(2, 0.0, 1.0),
        normalize([0.0, 1.0, 1.0]),
        1e-12
    ));
    assert!(close(
        face_dir(4, 1.0, 0.0),
        normalize([1.0, 0.0, 1.0]),
        1e-12
    ));
}

#[test]
fn rgbe_codec() {
    for c in [
        [0.01f32, 0.02, 0.03],
        [1.0, 0.5, 0.25],
        [40.0, 3.0, 0.1],
        [250.0, 250.0, 240.0],
    ] {
        let back = from_rgbe(to_rgbe(c));
        let max = c[0].max(c[1]).max(c[2]);
        for i in 0..3 {
            assert!(
                (back[i] - c[i]).abs() <= max / 255.0 + 1e-6,
                "{c:?} -> {back:?}"
            );
        }
    }
    assert_eq!(from_rgbe(to_rgbe([0.0; 3])), [0.0; 3]);
}

#[test]
fn harmonics_are_the_polynomial_seen_from_playcanvas() {
    let p = [
        [0.06, 0.07, 0.08],
        [0.37, 0.4, 0.5],
        [0.03, -0.02, 0.04],
        [0.63, 0.6, 0.66],
        [0.67, 0.7, 0.71],
        [0.64, 0.65, 0.68],
        [0.01, 0.03, -0.02],
        [0.02, -0.01, 0.009],
        [0.05, 0.04, -0.03],
    ];
    let sh = sh_from_polynomial(&p);
    for n in dirs() {
        for ch in 0..3 {
            let want = eval_polynomial(&p, babylon_dir_for_normal(n), ch);
            assert!((eval_sh(&sh, n, ch) - want).abs() < 1e-12, "{n:?}");
        }
    }
}

/// Babylon's runtime irradiance (its harmonics, `computeEnvironmentIrradiance`) as the polynomial
/// `babylon_irradiance` gives, at a few directions worked out by hand from Babylon's code.
#[test]
#[allow(clippy::needless_range_loop)]
fn babylon_irradiance_is_its_harmonics() {
    let p = [
        [0.06, 0.07, 0.08],
        [0.37, 0.4, 0.5],
        [0.03, -0.02, 0.04],
        [0.63, 0.6, 0.66],
        [0.67, 0.7, 0.71],
        [0.64, 0.65, 0.68],
        [0.01, 0.03, -0.02],
        [0.02, -0.01, 0.009],
        [0.05, 0.04, -0.03],
    ];
    let q = babylon_irradiance(&p);
    let (c0, c1, c2) = (
        (1.0 / (4.0 * PI)).sqrt(),
        (3.0 / (4.0 * PI)).sqrt(),
        (5.0 / (16.0 * PI)).sqrt(),
    );
    for ch in 0..3 {
        let [x, y, _, xx, yy, zz, ..] = [0, 1, 2, 3, 4, 5, 6, 7, 8].map(|i| p[i][ch]);
        let l00 = (xx + yy) * 0.376127 + zz * 0.376126;
        let l20 = zz * 1.34567 - (xx + yy) * 0.672834;
        // Straight up: l00 + l1-1 − l20 − l22 (each times π and its constant).
        let up = PI
            * (l00 * c0 + y * 0.977204 * c1
                - l20 * c2
                - (xx - yy) * 1.16538 * (15.0 / (16.0 * PI)).sqrt());
        assert!((eval_polynomial(&q, [0.0, 1.0, 0.0], ch) - up).abs() < 1e-12);
        // Along +x: l00 + l11 − l20 + l22.
        let east = PI
            * (l00 * c0 + x * 0.977204 * c1 - l20 * c2
                + (xx - yy) * 1.16538 * (15.0 / (16.0 * PI)).sqrt());
        assert!((eval_polynomial(&q, [1.0, 0.0, 0.0], ch) - east).abs() < 1e-12);
    }
}

/// A synthetic sky (a bright lobe and a gradient) through the whole conversion: the PlayCanvas
/// cube shows at `e` what Babylon shows at `babylon_dir_for_cube(e)`.
#[test]
fn faces_are_the_sky_seen_from_playcanvas() {
    let sun = normalize([0.6, 0.5, -0.62]);
    let sky = |d: [f64; 3]| {
        let k = cos(d, sun).max(0.0).powi(8);
        [
            (1.0 + d[1] + 4.0 * k) as f32,
            (1.0 + 0.5 * d[0] + 3.0 * k) as f32,
            (1.0 + 0.5 * d[2] + 2.0 * k) as f32,
        ]
    };
    let size = 64;
    let mut faces: [Vec<[f32; 3]>; 6] = Default::default();
    for (f, face) in faces.iter_mut().enumerate() {
        for y in 0..size {
            for x in 0..size {
                let s = (x as f64 + 0.5) / size as f64 * 2.0 - 1.0;
                let t = (y as f64 + 0.5) / size as f64 * 2.0 - 1.0;
                face.push(sky(face_dir(f, s, t)));
            }
        }
    }
    let env = BabylonSky {
        cube: Cube { size, faces },
        polynomial: [[0.0; 3]; 9],
    };
    let pc = convert(&env, 32);
    let back = read_pcenv(&write_pcenv(&pc, "test").unwrap()).unwrap();
    for e in dirs() {
        let want = sky(babylon_dir_for_cube(e));
        let got = back.cube.sample(e);
        for i in 0..3 {
            assert!(
                (got[i] - want[i] as f64).abs() < 0.12 * want[i] as f64 + 0.05,
                "{e:?}: {got:?} vs {want:?}"
            );
        }
    }
}

/// The light's mean direction in a cube's faces (every texel by its solid angle), its texel
/// directions taken through `world`.
fn mean_light(cube: &Cube, world: &dyn Fn([f64; 3]) -> [f64; 3]) -> [f64; 3] {
    let mut m = [0.0; 3];
    let n = cube.size;
    for (f, face) in cube.faces.iter().enumerate() {
        for y in 0..n {
            for x in 0..n {
                let s = (x as f64 + 0.5) / n as f64 * 2.0 - 1.0;
                let t = (y as f64 + 0.5) / n as f64 * 2.0 - 1.0;
                let w = (1.0 + s * s + t * t).powf(-1.5);
                let c = face[y * n + x];
                let l = w * (0.2126 * c[0] as f64 + 0.7152 * c[1] as f64 + 0.0722 * c[2] as f64);
                let d = world(face_dir(f, s, t));
                for i in 0..3 {
                    m[i] += d[i] * l;
                }
            }
        }
    }
    normalize(m)
}

/// The direction of three linear terms (x, y, z), each RGB weighted by luminance.
fn light_dir(xyz: [[f64; 3]; 3]) -> [f64; 3] {
    normalize(xyz.map(|c| {
        (0..3)
            .map(|ch| c[ch] * [0.2126, 0.7152, 0.0722][ch])
            .sum::<f64>()
    }))
}

fn horizontal(v: [f64; 3]) -> [f64; 3] {
    normalize([v[0], 0.0, v[2]])
}

/// The committed `.pcenv`: its faces and its harmonics agree on where the light comes from, as
/// PlayCanvas reads them.
#[test]
fn committed_sky_faces_and_harmonics_agree() {
    let pc = read_pcenv(&std::fs::read(pcenv_path()).expect("the .pcenv (cargo envconv)")).unwrap();
    assert_eq!(pc.cube.size, OUT_SIZE);
    // PlayCanvas's environment lookup for world direction d reads its cube at (-d.x, d.y, d.z),
    // so cube direction e is world direction (-e.x, e.y, e.z).
    let atlas = mean_light(&pc.cube, &|e| [-e[0], e[1], e[2]]);
    let sh = light_dir([pc.sh[1], pc.sh[2], pc.sh[3]]);
    assert!(cos(atlas, sh) > 0.99, "cube {atlas:?} vs harmonics {sh:?}");
    assert!(
        cos(horizontal(atlas), horizontal(sh)) > 0.95,
        "cube {atlas:?} vs harmonics {sh:?}"
    );
}

/// The committed `.pcenv` is what the HDRI converts to now (when `cargo envconv` has downloaded
/// it; skipped otherwise, so the tests need no network), and Babylon's cube and polynomial of
/// it agree on where the light comes from.
#[test]
fn committed_sky_is_the_hdri() {
    let Ok(bytes) = std::fs::read(hdri_path()) else {
        eprintln!(
            "committed_sky_is_the_hdri: skipped, no {} (cargo envconv downloads it)",
            hdri_path().display()
        );
        return;
    };
    assert_eq!(
        download::sha256_hex(&bytes),
        HDRI_SHA256,
        "{} isn't the pinned HDRI",
        hdri_path().display()
    );
    let sky = babylon_sky(&read_hdr(&bytes).unwrap());
    let faces = mean_light(&sky.cube, &|d| d);
    let poly = light_dir([sky.polynomial[0], sky.polynomial[1], sky.polynomial[2]]);
    assert!(
        cos(faces, poly) > 0.99,
        "faces {faces:?} vs polynomial {poly:?}"
    );
    assert!(
        cos(horizontal(faces), horizontal(poly)) > 0.95,
        "faces {faces:?} vs polynomial {poly:?}"
    );

    let pc = read_pcenv(&std::fs::read(pcenv_path()).expect("the .pcenv (cargo envconv)")).unwrap();
    let fresh = convert(&sky, OUT_SIZE);
    for ch in 0..3 {
        for i in 0..9 {
            assert!(
                (fresh.sh[i][ch] - pc.sh[i][ch]).abs() < 1e-6,
                "the .pcenv is stale: run cargo envconv"
            );
        }
    }
    for f in 0..6 {
        for (a, b) in fresh.cube.faces[f].iter().zip(&pc.cube.faces[f]) {
            let max = a[0].max(a[1]).max(a[2]);
            assert!(
                (0..3).all(|i| (a[i] - b[i]).abs() <= max / 128.0 + 1e-6),
                "the .pcenv is stale: run cargo envconv"
            );
        }
    }
}
