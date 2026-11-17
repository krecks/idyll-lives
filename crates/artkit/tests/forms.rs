//! The kernel's shapes and forms against the ones gamekit made in Blender 5.2 (tests/golden/forms.txt: each as a
//! family script called it, its mesh as Blender held it).
mod common;

use artkit::forms::by;
use artkit::materials::{Finish, Finishes};
use artkit::math::{Axis, P3, TAU, V3, cos, p3, pow, sin, v3};
use artkit::mesh::UvWay;
use artkit::noise::noise_at;
use artkit::parts::{hinged, join, join_all, new_parts, settle, turn};
use artkit::scene::{self, Obj};
use artkit::{boulder, cone, cube, cylinder, jitter, loft, palette, revolve, ring, sphere, sweep};
use common::*;

palette! { M: bark = ("#E4D8C8", bark), leaf = ("#558238", foliage), body = ("#C0683F", satin), metal = ("#D0D3D6", chrome) }

fn finishes() -> Finishes {
    let f = |roughness, metallic, textured| Finish {
        roughness,
        metallic,
        textured,
    };
    [
        ("bark", f(0.6, 0.0, true)),
        ("foliage", f(0.9, 0.0, true)),
        ("satin", f(0.42, 0.0, false)),
        ("chrome", f(0.2, 1.0, false)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

const TRUNK: [(f64, f64, f64); 5] = [
    (0.0, 0.0, 0.0),
    (0.0, 0.0, 0.3),
    (0.05, -0.03, 1.1),
    (-0.02, 0.04, 1.9),
    (0.03, 0.0, 2.7),
];
const TAPER: [f64; 5] = [1.5, 1.08, 0.95, 0.88, 0.72];

/// The scripts' `lump` (families/plants.py): a smooth sphere pushed about by noise, its bottom at `at`.
#[allow(clippy::too_many_arguments)]
fn lump(
    name: &str,
    r: f64,
    at: (f64, f64, f64),
    material: artkit::materials::Mat,
    scale: (f64, f64, f64),
    seed: f64,
    noise: f64,
    segments: usize,
    rings: usize,
) -> Obj {
    let obj = sphere!(
        name,
        r,
        material = material,
        segments = segments,
        rings = rings,
        scale = scale
    );
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            let p = *v;
            let n = noise_at(
                p.x as f64 / r * 1.3 + seed,
                p.y as f64 / r * 1.3 - seed,
                p.z as f64 / r * 1.3 + seed * 0.5,
            );
            *v = p * (1.0 + noise * n);
        }
        let bottom = o
            .mesh
            .verts
            .iter()
            .map(|v| v.z)
            .fold(f32::INFINITY, f32::min);
        o.mesh.translate(V3::from(at) - v3(0, 0, bottom));
    });
    obj
}

fn column() -> Obj {
    let radius = |z: f64| {
        if z < 1.4 {
            0.2 + 0.2 * (z - 0.75) / 0.65
        } else {
            0.4 * (1.0 - pow((z - 1.4) / 6.55, 1.6)) + 0.02
        }
    };
    let mut rings: Vec<Vec<P3>> = Vec::new();
    for k in 0..14 {
        let z = 0.75 + (7.95 - 0.75) * pow(k as f64 / 13.0, 0.9);
        let r = radius(z);
        rings.push(
            (0..10)
                .map(|v| {
                    let a = TAU * v as f64 / 10.0;
                    let wob = 1.0 + 0.12 * noise_at(cos(a) * 2.0, sin(a) * 2.0, z * 1.3);
                    p3(r * wob * cos(a), r * wob * sin(a), z)
                })
                .collect(),
        );
    }
    let last = rings.len() - 1;
    rings[last] = vec![p3(0.0, 0.0, 7.95); 10];
    loft!("column", &rings, material = M.leaf)
}

fn joined() -> Obj {
    sweep!(
        "trunk",
        &TRUNK,
        0.3,
        material = M.bark,
        verts = 5,
        resolution = 1,
        scale = TAPER
    );
    cone!(
        "k",
        0.3,
        0.03,
        0.85,
        at = (0.05, 0.02, 2.0),
        material = M.leaf,
        verts = 10
    );
    cube!(
        "b",
        (0.4, 0.3, 0.2),
        at = (0.1, 0.2, 0.3),
        material = M.body,
        rot = (0, 0, 25)
    );
    cylinder!(
        "c",
        0.05,
        0.4,
        at = (0.3, 0.1, 0.2),
        material = M.metal,
        verts = 12,
        rot = (90, 0, 30)
    );
    lump(
        "spray",
        0.19,
        (0.3, -0.2, 2.5),
        M.leaf,
        (1.0, 1.0, 2.6),
        7.3,
        0.25,
        6,
        5,
    );
    join_all("joined")
}

fn build(name: &str) -> Obj {
    let pad = [
        (0.1 * 0.5, 0.0),
        (-0.05 * 0.1, 0.6 * 0.3),
        (-0.6 * 0.1, 0.3),
        (-0.3 * 0.1, 0.0),
        (-0.6 * 0.1, -0.3),
        (-0.05 * 0.1, -0.6 * 0.3),
    ];
    match name {
        "sphere" => sphere!(
            "s",
            0.3,
            at = (0.1, 0.2, 0.5),
            material = M.leaf,
            scale = (1, 1.2, 2.6),
            segments = 10,
            rings = 6
        ),
        "sphere_rot" => sphere!(
            "s",
            0.11,
            at = (0.1, -0.2, 0.5),
            material = M.body,
            rot = (20, 30, 40),
            segments = 8,
            rings = 5
        ),
        "cylinder" => cylinder!(
            "c",
            0.05,
            0.4,
            at = (0.3, 0.1, 0.2),
            material = M.metal,
            verts = 12,
            rot = (90, 0, 30)
        ),
        "cylinder6" => cylinder!("c", 0.02, 0.7, material = M.body, verts = 6),
        "cone" => cone!(
            "k",
            0.3,
            0.03,
            0.85,
            at = (0.05, 0.02, 2.0),
            material = M.leaf,
            verts = 10
        ),
        "cone_point" => cone!("k", 0.2, 0.0, 0.5, material = M.leaf, verts = 6),
        "cube" => cube!(
            "b",
            (0.4, 0.3, 0.2),
            at = (0.1, 0.2, 0.3),
            material = M.body,
            rot = (0, 0, 25)
        ),
        "cube_smooth" => cube!("b", 0.4, material = M.body, smooth = true),
        "revolve" => revolve!(
            "r",
            &[
                (0.0, 0.0),
                (0.22, 0.0),
                (0.24, 0.02),
                (0.3, 0.47),
                (0.31, 0.5),
                (0.28, 0.5),
                (0.27, 0.46),
                (0.0, 0.46)
            ],
            material = M.body,
            segments = 12
        ),
        "revolve_auto" => revolve!(
            "r",
            &[
                (0.0, 0.0),
                (0.018, 0.0),
                (0.022, 0.05),
                (0.016, 0.1),
                (0.026, 0.6),
                (0.03, 0.72),
                (0.0, 0.72)
            ],
            at = (0.5, 0.3, 0),
            material = M.body
        ),
        "sweep_trunk" => sweep!(
            "trunk",
            &TRUNK,
            0.3,
            material = M.bark,
            verts = 11,
            resolution = 3,
            scale = TAPER
        ),
        "sweep_trunk_low" => sweep!(
            "trunk",
            &TRUNK,
            0.3,
            material = M.bark,
            verts = 5,
            resolution = 1,
            scale = TAPER
        ),
        "sweep_trunk_jitter" => jitter!(
            sweep!(
                "trunk",
                &TRUNK,
                0.3,
                material = M.bark,
                verts = 11,
                resolution = 3,
                scale = TAPER
            ),
            0.035,
            size = 0.45,
            seed = 4.0,
            up = 0.0
        ),
        "sweep_bough" => sweep!(
            "b",
            &[(0.12, 0.05, 0.55), (0.34, 0.2, 0.2), (0.6, 0.31, 0.085)],
            0.17,
            material = M.bark,
            verts = 5,
            resolution = 1,
            scale = by(|w| 1.0 + (0.07 / 0.17 - 1.0) * w)
        ),
        "sweep_limb" => sweep!(
            "l",
            &[
                v3(0.02, 0.0, 2.1),
                v3(0.4, 0.3, 2.9),
                v3(0.9, 0.5, 3.5),
                v3(1.2, 0.7, 4.0)
            ],
            0.16,
            material = M.bark,
            verts = 5,
            resolution = 3,
            scale = by(|w| 1.0 + (0.075 / 0.16 - 1.0) * w)
        ),
        "sweep_auto" => sweep!(
            "a",
            &[(0.0, 0.0, 0.0), (0.1, 0.0, 0.4), (0.3, 0.1, 0.5)],
            0.045,
            material = M.metal,
            resolution = 4
        ),
        "sweep_profile" => sweep!(
            "p",
            &[
                v3(0.05, 0.02, 3.0),
                v3(0.5, 0.3, 2.95),
                v3(0.9, 0.55, 2.7),
                v3(1.2, 0.8, 2.3),
                v3(1.4, 0.9, 2.1)
            ],
            0.0,
            material = M.leaf,
            profile = pad,
            scale = [0.35, 0.9, 1.0, 0.75, 0.25],
            up = (0, 0, 1)
        ),
        "sweep_closed" => {
            let pts: Vec<(f64, f64, f64)> = (0..24)
                .map(|k| {
                    (
                        0.25 * cos(TAU * k as f64 / 24.0),
                        0.25 * sin(TAU * k as f64 / 24.0),
                        0.36,
                    )
                })
                .collect();
            sweep!(
                "ring",
                &pts,
                0.012,
                material = M.body,
                verts = 5,
                closed = true
            )
        }
        "sweep_square" => sweep!(
            "rail",
            &[(0.0, 0.0, 0.0), (0.0, 0.0, 0.5), (0.3, 0.0, 0.8)],
            0.02,
            profile = [(-0.02, -0.01), (0.02, -0.01), (0.02, 0.01), (-0.02, 0.01)],
            material = M.body,
            resolution = 3,
            up = (0, 1, 0)
        ),
        "loft" => loft!(
            "l",
            &[
                ring!(0.2, 0.15, z = 0.0, verts = 12),
                ring!(0.25, 0.2, z = 0.3, verts = 12, square = 4.0),
                ring!(0.1, 0.1, z = 0.5, verts = 12, at = (0.02, 0.0))
            ],
            material = M.body,
            at = (0.1, 0, 0),
            rot = (0, 0, 15)
        ),
        "loft_column" => column(),
        "lump" => lump(
            "spray",
            0.19,
            (0.3, -0.2, 2.5),
            M.leaf,
            (1.0, 1.0, 2.6),
            7.3,
            0.25,
            6,
            5,
        ),
        "lump10" => lump(
            "clump",
            0.5,
            (0.3, -0.2, 2.5),
            M.leaf,
            (1.1, 0.95, 0.8),
            1.7,
            0.34,
            10,
            6,
        ),
        "boulder" => boulder!(
            "rock",
            0.4,
            at = (0.1, 0.1, 0),
            material = M.body,
            seed = 3.0,
            scale = (1.2, 1.0, 0.7)
        ),
        "joined" => joined(),
        "joined_settled" => {
            let o = joined();
            settle(&[o]);
            o
        }
        "hinged" => {
            let door = new_parts(|| {
                cube!(
                    "door",
                    (0.4, 0.03, 0.8),
                    at = (0.1, -0.2, 0.5),
                    material = M.body
                );
                cylinder!(
                    "knob",
                    0.02,
                    0.05,
                    at = (0.25, -0.24, 0.5),
                    material = M.metal,
                    verts = 8,
                    rot = (90, 0, 0)
                );
            });
            let o = hinged(join(&door, "door", None), (-0.1, -0.2, 0.1));
            turn(o, 30.0, Axis::Z)
        }
        other => panic!("{other}"),
    }
}

#[test]
fn shapes_and_forms_are_gamekits_in_blender() {
    scene::set_finishes(finishes());
    let mut report = Vec::new();
    let mut bad = Vec::new();
    for g in golden_meshes("forms.txt") {
        scene::reset();
        let obj = build(&g.what);
        let (mesh, loc, rot, scale, way) =
            obj.read(|o| (o.mesh.clone(), o.loc, o.rot, o.scale, o.uv));
        let mut said = Vec::new();
        let mut fail = |what: String| said.push(what);
        // where it is
        let placed: Vec<&str> = g.lines["loc"].split(' ').collect();
        let want = f32s(&[&placed[..3], &placed[4..7], &placed[8..11]].concat());
        let got = [loc.array(), rot.array(), scale.array()].concat();
        let off = got
            .iter()
            .zip(&want)
            .map(|(a, b)| if a == b { 0 } else { ulps(*a, *b) })
            .max()
            .unwrap();
        if off > 0 {
            fail(format!("placed {off} units off"));
        }
        let way_want = match placed[12] {
            "round" => UvWay::Round,
            "done" => UvWay::Done,
            _ => UvWay::Box,
        };
        if way != way_want {
            fail(format!("uv way {way:?}"));
        }
        // its points: Blender's to the bit (the icosphere's: within rounding of them)
        let loose: f32 = if g.what == "boulder" { 2e-6 } else { 0.0 };
        let exact = pair_points(&mesh.verts, &g.verts, 0.0);
        let Some((map, inexact, far)) =
            exact.or_else(|| pair_points(&mesh.verts, &g.verts, loose.max(3e-7)))
        else {
            report.push(format!(
                "{}: NOT Blender's points ({} for {})",
                g.what,
                mesh.verts.len(),
                g.verts.len()
            ));
            bad.push(g.what.clone());
            continue;
        };
        if far > loose {
            fail(format!("{inexact} points off by up to {far}"));
        }
        let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
            .map(|f| mesh.corners(f).to_vec())
            .collect();
        if g.what != "boulder" {
            if sorted(&renamed(&faces, &map)) != sorted(&g.faces) {
                fail(format!(
                    "faces differ ({} for {})",
                    faces.len(),
                    g.faces.len()
                ));
            }
            if as_set(&renamed(&mesh.triangle_verts(), &map)) != as_set(&g.tris) {
                fail("triangles differ".into());
            }
            // sharp edges
            let mut sharp: Vec<(u32, u32)> = mesh
                .sharp
                .iter()
                .map(|&(a, b)| artkit::mesh::edge(map[a as usize], map[b as usize]))
                .collect();
            sharp.sort_unstable();
            let mut want: Vec<(u32, u32)> = g
                .lines
                .get("e")
                .map(|l| {
                    l.split(' ')
                        .filter(|w| !w.is_empty())
                        .map(|e| {
                            let (a, b) = e.split_once('-').unwrap();
                            artkit::mesh::edge(a.parse().unwrap(), b.parse().unwrap())
                        })
                        .collect()
                })
                .unwrap_or_default();
            want.sort_unstable();
            if sharp != want {
                fail(format!(
                    "sharp edges differ ({} for {})",
                    sharp.len(),
                    want.len()
                ));
            }
        }
        let smooth: String = mesh
            .faces
            .iter()
            .map(|f| if f.smooth { '1' } else { '0' })
            .collect();
        let mut a: Vec<char> = smooth.chars().collect();
        let mut b: Vec<char> = g.lines["s"].chars().collect();
        a.sort_unstable();
        b.sort_unstable();
        if a != b {
            fail("smooth flags differ".into());
        }
        let mats: Vec<&str> = mesh.mats.iter().map(|m| m.name).collect();
        if mats.join(" ") != g.lines["mats"] {
            fail(format!("materials {mats:?}"));
        }
        // per face and corner, where the faces came out in Blender's order
        let in_order = renamed(&faces, &map) == g.faces;
        let mut normals_off = 0.0_f32;
        let mut uv_off = 0.0_f32;
        if in_order {
            let cn = artkit::normals::corner_normals(&mesh);
            for (n, w) in cn.iter().zip(g.f32s("cn").chunks(3)) {
                normals_off =
                    normals_off.max((0..3).map(|i| (n[i] - w[i]).abs()).fold(0.0, f32::max));
            }
            if normals_off > 2e-4 {
                fail(format!("corner normals off by {normals_off}"));
            }
            let want = g.f32s("uv");
            match &mesh.uvs {
                Some(uvs) if !want.is_empty() => {
                    for (u, w) in uvs.iter().zip(want.chunks(2)) {
                        uv_off = uv_off.max((u[0] - w[0]).abs().max((u[1] - w[1]).abs()));
                    }
                    if uv_off > 2e-6 {
                        fail(format!("uvs off by {uv_off}"));
                    }
                }
                Some(_) => fail("uvs where Blender has none".into()),
                // (Blender's own primitives come with an unwrap nothing reads: `uv::ensure` writes over it)
                None => {}
            }
        }
        let vn = artkit::normals::vertex_normals(&mesh);
        let want_vn = g.f32s("vn");
        let mut vn_off = 0.0_f32;
        for (i, n) in vn.iter().enumerate() {
            let w = &want_vn[map[i] as usize * 3..][..3];
            vn_off = vn_off.max((0..3).map(|k| (n[k] - w[k]).abs()).fold(0.0, f32::max));
        }
        if vn_off > 2e-4 && g.what != "boulder" {
            fail(format!("vertex normals off by {vn_off}"));
        }
        report.push(format!(
            "{}: {} points ({inexact} not to the bit, within {far}), faces in order {in_order}, corner normals within {normals_off}, vertex normals within {vn_off}, uvs within {uv_off}{}",
            g.what, mesh.verts.len(), if said.is_empty() { String::new() } else { format!(" -- {}", said.join("; ")) }
        ));
        if !said.is_empty() {
            bad.push(g.what.clone());
        }
    }
    println!("{}", report.join("\n"));
    assert!(bad.is_empty(), "{bad:?}\n{}", report.join("\n"));
    assert_eq!(report.len(), 27);
}
