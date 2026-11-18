//! What is written of a mesh with faces Blender's validate takes out, against the files Blender 5.2's exporter
//! wrote of the same meshes through gamekit's export_glb (tests/golden/validate.txt: each case's triangles in the
//! scene before the export, then each primitive of the file as its triangles' positions, normals, UVs and shading).
mod common;

use artkit::botany::{self, fiddle};
use artkit::materials::{Finish, Finishes, Mat};
use artkit::math::{V3, v3};
use artkit::mesh::Mesh;
use artkit::parts::join_all;
use artkit::rand::Random;
use artkit::scene::{self, Obj};
use artkit::{cut_leaf, family, glb, outline, perch, pinnate, slits, validate};
use common::*;
use std::collections::BTreeMap;

/// The finishes of the base manifest the plants' palette wears.
fn finishes() -> Finishes {
    let f = |roughness, textured| Finish {
        roughness,
        metallic: 0.0,
        textured,
    };
    [
        ("bark", f(0.6, true)),
        ("foliage", f(0.9, true)),
        ("wood", f(0.6, true)),
        ("satin", f(0.42, false)),
        ("matte", f(0.75, false)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

/// One triangle of a file: its corners' positions, then what else each corner carries (normal, UV, shading).
#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct Tri {
    p: [[u32; 3]; 3],
    rest: [Vec<f32>; 3],
}

/// A triangle from the corner whose position sorts lowest, wound as it was.
fn tri(p: [[f32; 3]; 3], rest: [Vec<f32>; 3]) -> Tri {
    // (a zero's sign isn't a difference)
    let p = p.map(|c| c.map(|x| (x + 0.0).to_bits()));
    let at = (0..3).min_by_key(|&i| p[i]).unwrap();
    let turn = |i: usize| (at + i) % 3;
    Tri {
        p: [p[turn(0)], p[turn(1)], p[turn(2)]],
        rest: [0, 1, 2].map(|i| rest[turn(i)].clone()),
    }
}

/// A primitive's triangles, sorted.
type Prims = BTreeMap<(String, String), Vec<Tri>>;

/// A golden case: its triangles in the scene, and its file's primitives by (part, material).
struct Case {
    name: String,
    scene: usize,
    file: usize,
    prims: Prims,
}

fn cases() -> Vec<Case> {
    let mut out: Vec<Case> = Vec::new();
    let mut lines: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut key = None;
    let text = golden("validate.txt");
    // a primitive's lines, once they are all read
    fn close(
        case: &mut Case,
        key: &mut Option<(String, String)>,
        lines: &mut BTreeMap<&str, Vec<&str>>,
    ) {
        let Some(key) = key.take() else { return };
        let p = f32s(&lines["p"]);
        let n = f32s(&lines["n"]);
        let u = lines.get("u").map(|u| f32s(u));
        // (a file written unshaded has no shading: white)
        let c: Vec<f32> = match lines.get("c") {
            Some(c) => c
                .iter()
                .map(|w| w.parse::<f32>().unwrap() / 65535.0)
                .collect(),
            None => vec![1.0; p.len() / 3],
        };
        let mut tris: Vec<Tri> = (0..p.len() / 9)
            .map(|t| {
                let corner = |i: usize| {
                    let k = t * 3 + i;
                    let mut rest = n[k * 3..k * 3 + 3].to_vec();
                    rest.extend(u.as_ref().map_or([0.0, 0.0], |u| [u[k * 2], u[k * 2 + 1]]));
                    rest.push(c[k]);
                    ([p[k * 3], p[k * 3 + 1], p[k * 3 + 2]], rest)
                };
                let (a, b, c) = (corner(0), corner(1), corner(2));
                tri([a.0, b.0, c.0], [a.1, b.1, c.1])
            })
            .collect();
        tris.sort_by(|a, b| a.partial_cmp(b).unwrap());
        case.prims.insert(key, tris);
        lines.clear();
    }
    for line in text.lines() {
        let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
        let words: Vec<&str> = rest.split(' ').filter(|w| !w.is_empty()).collect();
        match word {
            "#" => {
                if let Some(case) = out.last_mut() {
                    close(case, &mut key, &mut lines);
                }
                out.push(Case {
                    name: rest.to_string(),
                    scene: 0,
                    file: 0,
                    prims: Prims::new(),
                });
            }
            "scene" => out.last_mut().unwrap().scene = rest.parse().unwrap(),
            "file" => out.last_mut().unwrap().file = rest.parse().unwrap(),
            "prim" => {
                close(out.last_mut().unwrap(), &mut key, &mut lines);
                key = Some((words[0].to_string(), words[1].to_string()));
            }
            _ => {
                lines.insert(word, words);
            }
        }
    }
    if let Some(case) = out.last_mut() {
        close(case, &mut key, &mut lines);
    }
    out
}

/// A written file's primitives by (part, material), as the golden has them.
fn written(bytes: &[u8]) -> Prims {
    let model = glb::read(bytes).unwrap();
    let mut out = Prims::new();
    for node in &model.nodes {
        let Some(mesh) = node.mesh else { continue };
        for prim in &model.meshes[mesh].primitives {
            let material = model.materials[prim.material.unwrap()].name.clone();
            let corner = |i: u32| {
                let i = i as usize;
                let mut rest = prim.normals[i].to_vec();
                rest.extend(prim.uvs.as_ref().map_or([0.0, 0.0], |u| u[i]));
                rest.push(prim.colours.as_ref().map_or(1.0, |c| c[i][0]));
                (prim.positions[i], rest)
            };
            let mut tris: Vec<Tri> = prim
                .indices
                .chunks(3)
                .map(|t| {
                    let (a, b, c) = (corner(t[0]), corner(t[1]), corner(t[2]));
                    tri([a.0, b.0, c.0], [a.1, b.1, c.1])
                })
                .collect();
            tris.sort_by(|a, b| a.partial_cmp(b).unwrap());
            out.insert((node.name.clone(), material), tris);
        }
    }
    out
}

/// A mesh as given, not validated: `from_pydata`, its materials, each face's.
fn raw(name: &str, verts: &[[f64; 3]], faces: &[&[u32]], mats: &[Mat], face_mats: &[u16]) -> Obj {
    let verts: Vec<V3> = verts.iter().map(|p| v3(p[0], p[1], p[2])).collect();
    let faces: Vec<Vec<u32>> = faces.iter().map(|f| f.to_vec()).collect();
    let mut mesh = Mesh::from_pydata(verts, &faces);
    for &m in mats {
        mesh.set_material(Some(m));
    }
    for (face, &k) in mesh.faces.iter_mut().zip(face_mats) {
        face.mat = k;
    }
    scene::add(name, mesh)
}

/// Builds a case in the empty scene: its parts, whether it is shaded (and written by the families' export), and
/// whether it stands on the ground.
fn build(name: &str) -> (Vec<Obj>, bool, bool) {
    let p = botany::palette(&[]);
    let m = |k: &str| p.get(k);
    const NAN: f64 = f64::NAN;
    match name {
        "frond" => {
            let o = cut_leaf!(
                "frond",
                (0, 0, 0.8),
                10.0,
                50.0,
                0.45,
                &pinnate!(8, 0.14, "lance", lean = 0.5, fill = 0.75),
                m("leaf"),
                droop = 1.0,
                fold = 0.12,
                ribs = 4
            );
            (vec![o], true, false)
        }
        "monstera" => {
            let o = cut_leaf!(
                "leaf",
                v3(0.21, 0.13, 0.52),
                57.0,
                31.0,
                0.36,
                &slits!("heart", 0.34, 5, depth = 0.62, gap = 0.22),
                m("leaf_dark"),
                droop = 0.5,
                fold = 0.1,
                ribs = 5,
                thick = 0.006
            );
            (vec![o], true, false)
        }
        "fiddle" => {
            let o = cut_leaf!(
                "leaf",
                v3(0.1, -0.05, 1.1),
                211.0,
                24.0,
                0.3,
                &outline!(fiddle, 0.21, n = 7, wave = 0.05, waves = 5),
                m("leaf_light"),
                droop = 0.22,
                fold = 0.16,
                ribs = 5,
                thick = 0.008
            );
            (vec![o], true, false)
        }
        "fern" => {
            // the Boston fern's first three fronds and a crown, joined as it joins them
            let mut rng = Random::new(96);
            let cols = [m("leaf"), m("leaf_dark"), m("leaf_light")];
            for i in 0..3 {
                let yaw = i as f64 * 40.0 + rng.uniform(-10.0, 10.0);
                let (tilt, length, droop) = (
                    rng.uniform(40.0, 62.0),
                    rng.uniform(0.43, 0.49),
                    rng.uniform(0.92, 1.04),
                );
                cut_leaf!(
                    format!("frond{i}"),
                    (0, 0, 0.26),
                    yaw,
                    tilt,
                    length,
                    &pinnate!(8, 0.14, "lance", lean = 0.5, fill = 0.75),
                    cols[i % 3],
                    droop = droop,
                    fold = 0.12,
                    ribs = 4
                );
            }
            cut_leaf!(
                "crown0",
                (0, 0, 0.26),
                30.0,
                68.0,
                0.28,
                &pinnate!(6, 0.1, "lance", lean = 0.5, fill = 0.75),
                m("leaf_light"),
                droop = 0.5,
                fold = 0.12,
                ribs = 3
            );
            (vec![join_all("fern")], true, true)
        }
        // a face again the other way round, from another corner, the same way; a quad's bow tie, which stays
        "reversed" => {
            let v = [
                [0.0, 0.0, 0.0],
                [0.1, 0.0, 0.0],
                [0.0, 0.1, 0.0],
                [0.2, 0.0, 0.0],
                [0.3, 0.0, 0.0],
                [0.3, 0.1, 0.02],
                [0.2, 0.1, 0.0],
                [0.0, 0.0, 0.1],
                [0.1, 0.0, 0.1],
                [0.0, 0.1, 0.1],
            ];
            let f: [&[u32]; 9] = [
                &[0, 1, 2],
                &[2, 1, 0],
                &[3, 4, 5, 6],
                &[5, 4, 3, 6],
                &[3, 5, 4, 6],
                &[1, 2, 0],
                &[7, 8, 9],
                &[6, 5, 4, 3],
                &[4, 3, 6, 5],
            ];
            (
                vec![raw("reversed", &v, &f, &[m("stalk")], &[])],
                true,
                false,
            )
        }
        // the doubled faces on two materials: the first stays, and a material left with no face has no primitive
        "materials" => {
            let v = [
                [0.0, 0.0, 0.0],
                [0.1, 0.0, 0.0],
                [0.0, 0.1, 0.0],
                [0.2, 0.0, 0.05],
                [0.3, 0.0, 0.05],
                [0.2, 0.1, 0.05],
                [0.0, 0.0, 0.2],
                [0.1, 0.0, 0.2],
                [0.0, 0.1, 0.2],
            ];
            let f: [&[u32]; 6] = [
                &[0, 1, 2],
                &[0, 2, 1],
                &[3, 4, 5],
                &[5, 4, 3],
                &[2, 0, 1],
                &[6, 7, 8],
            ];
            let mats = [m("stalk"), m("body"), m("soil")];
            (
                vec![raw("materials", &v, &f, &mats, &[0, 1, 1, 0, 2, 1])],
                true,
                false,
            )
        }
        "plain" => {
            let v = [
                [-0.1, -0.1, 0.0],
                [0.1, -0.1, 0.0],
                [0.1, 0.1, 0.0],
                [-0.1, 0.1, 0.0],
                [0.0, 0.0, 0.15],
            ];
            let f: [&[u32]; 5] = [
                &[3, 2, 1, 0],
                &[0, 1, 4],
                &[1, 2, 4],
                &[2, 3, 4],
                &[3, 0, 4],
            ];
            (vec![raw("plain", &v, &f, &[m("stalk")], &[])], true, true)
        }
        // a face with a vertex twice (next to itself, or not) and one of two corners go; two points in one place
        // and the triangle of no area on them stay
        "bad" => {
            let v = [
                [0.0, 0.0, 0.0],
                [0.1, 0.0, 0.0],
                [0.0, 0.1, 0.0],
                [0.2, 0.0, 0.0],
                [0.3, 0.0, 0.0],
                [0.3, 0.1, 0.0],
                [0.2, 0.1, 0.0],
                [0.0, 0.0, 0.1],
                [0.1, 0.0, 0.1],
                [0.1, 0.0, 0.1],
                [0.0, 0.1, 0.1],
                [0.4, 0.0, 0.0],
                [0.5, 0.0, 0.0],
                [0.4, 0.1, 0.0],
            ];
            let f: [&[u32]; 8] = [
                &[0, 1, 2],
                &[3, 4, 4, 5],
                &[3, 4, 5, 4],
                &[7, 8, 9, 10],
                &[11, 12, 13],
                &[12, 11, 13],
                &[3, 5, 6],
                &[11, 12],
            ];
            (vec![raw("bad", &v, &f, &[m("stalk")], &[])], false, false)
        }
        // a point with a coordinate that is no number, or endless, is put at the origin
        "nan" => {
            let v = [
                [0.0, 0.0, 0.0],
                [0.1, 0.0, 0.0],
                [0.0, 0.1, 0.0],
                [0.2, 0.0, 0.0],
                [0.3, NAN, 0.0],
                [0.2, 0.1, 0.0],
                [0.4, 0.0, 0.0],
                [0.5, 0.0, 0.0],
                [f64::INFINITY, 0.1, 0.3],
            ];
            let f: [&[u32]; 3] = [&[0, 1, 2], &[3, 4, 5], &[6, 7, 8]];
            (vec![raw("nan", &v, &f, &[m("stalk")], &[])], false, false)
        }
        _ => panic!("{name}: no such case"),
    }
}

#[test]
fn the_file_has_what_blender_s_has() {
    scene::set_finishes(finishes());
    let all = cases();
    assert_eq!(all.len(), 9);
    let mut report = Vec::new();
    for case in &all {
        let name = &case.name;
        scene::reset();
        perch::clear();
        botany::clear();
        let (objs, shaded, ground) = build(name);
        let in_scene = || -> usize {
            objs.iter()
                .map(|o| o.read(|p| p.mesh.triangles().len()))
                .sum()
        };
        assert_eq!(in_scene(), case.scene, "{name}: triangles in the scene");
        let bytes = if shaded {
            family::export(&objs, ground)
        } else {
            // (as export_glb(shade=False) writes it: validated, not shaded)
            validate::meshes(&objs);
            glb::write(&objs, false, &finishes())
        };
        // (Blender's exporter leaves the scene's mesh validated too)
        assert_eq!(in_scene(), case.file, "{name}: triangles left in the scene");
        let got = written(&bytes);
        let keys = |p: &Prims| p.keys().cloned().collect::<Vec<_>>();
        assert_eq!(keys(&got), keys(&case.prims), "{name}: its primitives");
        let (mut far_n, mut far_u, mut far_c, mut off_c) = (0.0_f32, 0.0_f32, 0.0_f32, 0);
        for (key, want) in &case.prims {
            let got = &got[key];
            assert_eq!(got.len(), want.len(), "{name} {key:?}: triangles");
            for (g, w) in got.iter().zip(want) {
                assert_eq!(g.p, w.p, "{name} {key:?}: a triangle's points");
                for (g, w) in g.rest.iter().zip(&w.rest) {
                    let far = |r: std::ops::Range<usize>| {
                        r.map(|i| (g[i] - w[i]).abs()).fold(0.0, f32::max)
                    };
                    far_n = far_n.max(far(0..3));
                    far_u = far_u.max(far(3..5));
                    far_c = far_c.max(far(5..6));
                    off_c += (far(5..6) > 1e-4) as usize;
                }
            }
        }
        // what stays is shaded, mapped and lit as Blender's: the shading was baked with the doubles still there
        assert!(far_n <= 2e-4, "{name}: a normal {far_n} off");
        assert!(far_u <= 2e-6, "{name}: a UV {far_u} off");
        assert_eq!(off_c, 0, "{name}: the shading, up to {far_c} off");
        report.push(format!(
            "{name}: {} -> {} triangles; normals within {far_n:e}, UVs {far_u:e}, shading {far_c:e}",
            case.scene, case.file
        ));
    }
    println!("{}", report.join("\n"));
}

#[test]
fn a_whole_mesh_is_left_as_it_is() {
    scene::set_finishes(finishes());
    scene::reset();
    let (objs, _, _) = build("plain");
    let before = objs[0].read(|p| p.mesh.clone());
    let mut mesh = before.clone();
    assert!(!validate::validate(&mut mesh));
    assert_eq!(mesh.loops, before.loops);
    assert_eq!(mesh.faces, before.faces);
}

#[test]
fn what_stays_keeps_what_its_corners_carry() {
    scene::set_finishes(finishes());
    scene::reset();
    let (objs, _, _) = build("materials");
    let mut mesh = objs[0].read(|p| p.mesh.clone());
    let n = mesh.loops.len();
    mesh.faces[3].smooth = true;
    mesh.faces[5].smooth = true;
    mesh.sharp.insert((0, 1));
    mesh.uvs = Some((0..n).map(|l| [l as f32, 0.5]).collect());
    mesh.normals = Some((0..n).map(|l| v3(0, 0, l as f64)).collect());
    mesh.stored = Some((0..n).map(|l| Some([l as i16, 1])).collect());
    mesh.ao = Some(vec![0.25; mesh.verts.len()]);
    let before = mesh.clone();
    assert!(validate::validate(&mut mesh));
    // faces 0, 2 and 5 stay: corners 0..3, 6..9 and 15..18
    let corners: Vec<usize> = (0..3).chain(6..9).chain(15..18).collect();
    assert_eq!(mesh.faces.len(), 3);
    for (f, old) in [0, 2, 5].into_iter().enumerate() {
        let (face, was) = (mesh.faces[f], before.faces[old]);
        assert_eq!((face.start, face.len), (f as u32 * 3, 3));
        assert_eq!((face.mat, face.smooth), (was.mat, was.smooth));
    }
    let pick = |all: &[u32]| corners.iter().map(|&l| all[l]).collect::<Vec<_>>();
    assert_eq!(mesh.loops, pick(&before.loops));
    let uvs = before.uvs.as_ref().unwrap();
    let want: Vec<[f32; 2]> = corners.iter().map(|&l| uvs[l]).collect();
    assert_eq!(mesh.uvs.as_ref().unwrap(), &want);
    let normals = before.normals.as_ref().unwrap();
    let want: Vec<V3> = corners.iter().map(|&l| normals[l]).collect();
    assert_eq!(mesh.normals.as_ref().unwrap(), &want);
    let stored = before.stored.as_ref().unwrap();
    let want: Vec<Option<[i16; 2]>> = corners.iter().map(|&l| stored[l]).collect();
    assert_eq!(mesh.stored.as_ref().unwrap(), &want);
    assert_eq!(mesh.sharp, before.sharp);
    assert_eq!(mesh.ao, before.ao);
    assert_eq!(mesh.verts, before.verts);
    assert_eq!(mesh.mats.len(), 3);
    // a second time there is nothing to do
    assert!(!validate::validate(&mut mesh));
}
