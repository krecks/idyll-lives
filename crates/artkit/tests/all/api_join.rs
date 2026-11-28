//! Parts joined as Blender joins them (tests/golden/api_join.txt, from Blender 5.2): each part's points go from its
//! own frame into the first part's, through Blender's own inverse of the first's matrix, and stay as they are where
//! that comes to the identity; then the first part's turn and scale go into the mesh. The pond's ripples (three
//! rings in one place, scaled alike), parts joined onto a scaled first part, a turned one, one turned by a
//! quaternion and scaled, and plants fitted to a size by their matrices; and the inverse alone.

use crate::common::{f32s, f64s, golden};
use artkit::materials::Mat;
use artkit::math::{M4, invert_m4, v3};
use artkit::parts::{join, measure};
use artkit::scene::{self, Obj};
use artkit::{cube, cylinder, revolve, rod, sphere, torus};

const BODY: Mat = Mat::new("body", "#F3F3F1", "gloss");

fn matrix(words: &[&str]) -> M4 {
    let w = f32s(words);
    let mut m = M4::IDENTITY;
    for r in 0..4 {
        for c in 0..4 {
            m.0[c][r] = w[r * 4 + c];
        }
    }
    m
}

/// How many of `got` are not `want` to the bit (a zero's sign is no difference), and the farthest apart.
fn off(got: &[f32], want: &[f32]) -> (usize, f32) {
    assert_eq!(got.len(), want.len());
    let mut n = 0;
    let mut far = 0.0_f32;
    for (g, w) in got.iter().zip(want) {
        if g != w {
            n += 1;
            far = far.max((g - w).abs());
        }
    }
    (n, far)
}

fn parts_of(case: &str) -> (Vec<Obj>, &'static str) {
    match case {
        "pond" => {
            let rings = [0.1, 0.19, 0.28]
                .iter()
                .enumerate()
                .map(|(k, &r)| {
                    torus!(
                        format!("ring{k}"),
                        r,
                        0.006,
                        at = (-0.18, -0.12, 0.07),
                        material = BODY,
                        major_segments = 22,
                        minor_segments = 4
                    )
                    .set_scale((1, 0.9, 0.35))
                })
                .collect();
            (rings, "ripples")
        }
        "scaled" => {
            let parts = vec![
                cube!(
                    "a",
                    (0.2, 0.3, 0.4),
                    at = (0.5, -0.25, 0.75),
                    material = BODY,
                    bevel = 0
                ),
                cylinder!(
                    "b",
                    0.05,
                    0.4,
                    at = (-0.2, 0.3, 0.2),
                    material = BODY,
                    verts = 8,
                    rot = (0, 90, 0)
                ),
                sphere!(
                    "c",
                    0.1,
                    at = (0.13, 0.07, 0.31),
                    material = BODY,
                    segments = 8,
                    rings = 4
                ),
            ];
            parts[0].set_scale((1.7, 0.6, 1.3));
            parts[2].set_scale((0.5, 2.0, 1.0));
            (parts, "scaled")
        }
        "turned" => (
            vec![
                cube!(
                    "a",
                    (0.2, 0.3, 0.4),
                    at = (0.5, -0.25, 0.75),
                    material = BODY,
                    rot = (17, -40, 109),
                    bevel = 0
                ),
                cylinder!(
                    "b",
                    0.05,
                    0.4,
                    at = (-0.2, 0.3, 0.2),
                    material = BODY,
                    verts = 8,
                    rot = (0, 90, 0)
                ),
                cube!(
                    "c",
                    (0.1, 0.1, 0.1),
                    at = (0.2, 0.2, 0.2),
                    material = BODY,
                    rot = (17, -40, 109),
                    bevel = 0
                ),
            ],
            "turned",
        ),
        "rod" => {
            let parts = vec![
                rod!(
                    "a",
                    (0.1, 0.2, 0.3),
                    (0.4, -0.1, 0.9),
                    0.02,
                    material = BODY,
                    verts = 8
                ),
                cube!(
                    "b",
                    (0.1, 0.1, 0.1),
                    at = (0.2, 0.2, 0.2),
                    material = BODY,
                    bevel = 0
                ),
                rod!(
                    "c",
                    (0.1, 0.2, 0.3),
                    (0.4, -0.1, 0.9),
                    0.03,
                    material = BODY,
                    verts = 6
                ),
            ];
            parts[0].set_scale((1.1, 0.9, 1.0));
            (parts, "rod")
        }
        "fit_to" => {
            let parts = vec![
                revolve!(
                    "pot",
                    &[(0.0, 0.0), (0.1, 0.0), (0.12, 0.2), (0.0, 0.2)],
                    material = BODY,
                    segments = 8
                ),
                sphere!(
                    "bush",
                    0.15,
                    at = (0.02, -0.01, 0.33),
                    material = BODY,
                    segments = 8,
                    rings = 5,
                    scale = (1, 1.1, 0.9)
                ),
                cube!(
                    "tag",
                    (0.04, 0.002, 0.06),
                    at = (0.09, -0.02, 0.24),
                    material = BODY,
                    rot = (0, 10, 25),
                    bevel = 0
                ),
            ];
            // plants.py's fit_to: every part's matrix scaled about the origin to the size wanted
            let (lo, hi) = measure(&parts);
            let k: Vec<f64> = [0.3, 0.3, 0.5]
                .iter()
                .enumerate()
                .map(|(i, want)| want / (hi[i] - lo[i]))
                .collect();
            let scale = M4::diagonal(v3(k[0], k[1], k[2]));
            for &o in &parts {
                o.set_matrix_world(scale * o.matrix_world());
            }
            (parts, "fitted")
        }
        other => panic!("no case {other}"),
    }
}

#[test]
fn parts_are_joined_as_blender_joins_them() {
    let text = golden("api_join.txt");
    let mut case = "";
    let mut built: Option<(Vec<Obj>, &str)> = None;
    let (mut inverses, mut meshes) = (0, 0);
    for line in text.lines() {
        let w: Vec<&str> = line.split(' ').collect();
        match w[0] {
            "case" => {
                case = w[1];
                scene::reset();
                built = (case != "inverses").then(|| parts_of(case));
            }
            "k" => {
                // (the scale the fitting came to, from the parts' measure: Blender's to the bit)
                let want = f64s(&w[1..]);
                let first = built.as_ref().unwrap().0[0];
                let got = first.scale();
                assert_eq!(got.x, want[0] as f32, "{case}: the fitted scale");
            }
            "inverse" => {
                let (m, want) = (matrix(&w[3..19]), matrix(&w[20..36]));
                let got = invert_m4(&m).unwrap();
                assert_eq!(
                    off(&got.rows(), &want.rows()),
                    (0, 0.0),
                    "{case} {}: the inverse",
                    w[1]
                );
                inverses += 1;
            }
            "world" => {
                let o = scene::obj(w[1]);
                let want = matrix(&w[2..18]);
                assert_eq!(
                    off(&o.matrix_world().rows(), &want.rows()),
                    (0, 0.0),
                    "{case} {}: its matrix",
                    w[1]
                );
            }
            "verts" if !w[1].ends_with(".joined") => {
                let (parts, name) = built.take().unwrap();
                let joined = join(&parts, name, None);
                let got: Vec<f32> =
                    joined.read(|o| o.mesh.verts.iter().flat_map(|v| v.array()).collect());
                let (n, far) = off(&got, &f32s(&w[2..]));
                println!(
                    "{case}: {} numbers, {n} not Blender's to the bit, the farthest {far:e}",
                    got.len()
                );
                assert_eq!((n, far), (0, 0.0), "{case}: the joined points");
                meshes += 1;
            }
            _ => {}
        }
    }
    assert_eq!((inverses, meshes), (12, 5));
}
