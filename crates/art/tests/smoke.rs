//! `cargo art` from end to end on a model that needs no leaf cards: the Italian cypress (the kernel's own, kept
//! with its tests), built by the runner under another root and compared with the committed files and pack entry,
//! which the trees family built, triangle for triangle.
#[path = "../../artkit/tests/all/common/cypress.rs"]
mod cypress;

use ll_art::{Options, run_family};
use ll_tools::art::{J, loads};
use ll_tools::checkglb::{self, Paths};
use std::collections::BTreeMap;

/// A .glb's JSON and its binary chunk.
fn read(path: &str) -> (J, Vec<u8>) {
    let b = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let n = u32::from_le_bytes(b[12..16].try_into().unwrap()) as usize;
    (loads(&b[20..20 + n]).unwrap(), b[20 + n + 8..].to_vec())
}

fn int(j: &J, key: &str) -> usize {
    j.get(key)
        .and_then(J::num)
        .map(|n| n.f() as usize)
        .unwrap_or(0)
}

/// An accessor's numbers, a row an element.
fn rows(j: &J, bin: &[u8], accessor: usize) -> Vec<Vec<f64>> {
    let a = &j.get("accessors").and_then(J::list).unwrap()[accessor];
    let view = &j.get("bufferViews").and_then(J::list).unwrap()[int(a, "bufferView")];
    let per = match a.get("type").and_then(J::str).unwrap() {
        "SCALAR" => 1,
        "VEC2" => 2,
        "VEC3" => 3,
        _ => 4,
    };
    let (kind, at) = (int(a, "componentType"), int(view, "byteOffset"));
    let size = if kind == 5126 || kind == 5125 { 4 } else { 2 };
    (0..int(a, "count"))
        .map(|i| {
            (0..per)
                .map(|k| {
                    let o = at + (i * per + k) * size;
                    match kind {
                        5126 => f32::from_le_bytes(bin[o..o + 4].try_into().unwrap()) as f64,
                        5125 => u32::from_le_bytes(bin[o..o + 4].try_into().unwrap()) as f64,
                        _ => u16::from_le_bytes(bin[o..o + 2].try_into().unwrap()) as f64,
                    }
                })
                .collect()
        })
        .collect()
}

/// What of a file the comparison looks at, a material at a time: its triangles' corners (position, shading level).
struct Seen {
    node: String,
    translation: Vec<f64>,
    materials: Vec<(String, Vec<f64>)>,
    corners: BTreeMap<String, Vec<([f64; 3], f64)>>,
    has_uvs: bool,
}

fn see(path: &str) -> Seen {
    let (j, bin) = read(path);
    let node = &j.get("nodes").and_then(J::list).unwrap()[0];
    let numbers = |v: Option<&J>| {
        v.and_then(J::list)
            .unwrap_or(&[])
            .iter()
            .map(|n| n.num().unwrap().f())
            .collect::<Vec<f64>>()
    };
    let mats = j.get("materials").and_then(J::list).unwrap();
    let mut seen = Seen {
        node: node.get("name").and_then(J::str).unwrap().to_string(),
        translation: numbers(node.get("translation")),
        materials: mats
            .iter()
            .map(|m| {
                (
                    m.get("name").and_then(J::str).unwrap().to_string(),
                    numbers(
                        m.get("pbrMetallicRoughness")
                            .and_then(|p| p.get("baseColorFactor")),
                    ),
                )
            })
            .collect(),
        corners: BTreeMap::new(),
        has_uvs: false,
    };
    let mesh = &j.get("meshes").and_then(J::list).unwrap()[0];
    for p in mesh.get("primitives").and_then(J::list).unwrap() {
        let attrs = p.get("attributes").unwrap();
        seen.has_uvs |= attrs.get("TEXCOORD_0").is_some();
        let (pos, col) = (
            rows(&j, &bin, int(attrs, "POSITION")),
            rows(&j, &bin, int(attrs, "COLOR_0")),
        );
        let name = seen.materials[int(p, "material")].0.clone();
        let corners = seen.corners.entry(name).or_default();
        for i in rows(&j, &bin, int(p, "indices")) {
            let v = i[0] as usize;
            corners.push((
                [pos[v][0], pos[v][1], pos[v][2]],
                col[v][0] * 32.0 / 65535.0,
            ));
        }
    }
    seen
}

#[test]
fn the_cypress_built_here_is_the_committed_one() {
    let repo = Paths::repo();
    let family = cypress::family();
    let root = std::env::temp_dir().join(format!("ll-art-smoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let options = Options {
        out: Some(root.to_string_lossy().into_owned()),
        ..Options::default()
    };
    run_family(&repo, &family, &options).unwrap();
    let out = Paths::at(&root.to_string_lossy());
    // its pack entry: the committed one to the letter (its perches where they stood)
    let entry = |assets: &str| {
        let pack =
            loads(&std::fs::read(format!("{assets}/packs/trees/manifest.json")).unwrap()).unwrap();
        format!(
            "{:?}",
            pack.get("entries")
                .and_then(|e| e.get("model.cypress"))
                .unwrap()
        )
    };
    assert_eq!(entry(&out.assets), entry(&repo.assets));
    for (file, most_off) in [("cypress.glb", 0.0), ("cypress.far.glb", 0.0)] {
        let (new, old) = (
            see(&format!("{}/models/trees/{file}", out.assets)),
            see(&format!("{}/models/trees/{file}", repo.assets)),
        );
        assert_eq!(
            (&new.node, &new.translation, &new.materials, new.has_uvs),
            (&old.node, &old.translation, &old.materials, old.has_uvs),
            "{file}"
        );
        let (mut triangles, mut worst, mut levels, mut corners) = (0, 0.0_f64, 0.0_f64, 0usize);
        for (material, was) in &old.corners {
            let now = new
                .corners
                .get(material)
                .unwrap_or_else(|| panic!("{file}: no {material}"));
            assert_eq!(now.len(), was.len(), "{file}: {material}'s triangles");
            triangles += now.len() / 3;
            // each corner of the committed file has one here at its place, shaded alike
            for (p, level) in was {
                let (near, shade) = now
                    .iter()
                    .map(|(q, l)| {
                        (
                            (0..3).map(|i| (p[i] - q[i]).abs()).fold(0.0, f64::max),
                            (l - level).abs(),
                        )
                    })
                    .fold((f64::MAX, f64::MAX), |a, b| {
                        if b.0 < a.0 || (b.0 == a.0 && b.1 < a.1) {
                            b
                        } else {
                            a
                        }
                    });
                worst = worst.max(near);
                levels += shade;
                corners += 1;
            }
        }
        println!(
            "{file}: {triangles} triangles as committed; its corners within {worst} m of the committed ones, their shading {:.4} levels off a corner",
            levels / corners as f64
        );
        assert!(
            worst <= most_off,
            "{file}: a corner {worst} m from any committed one"
        );
        assert!(
            levels / corners as f64 <= 0.01,
            "{file}: shading {} levels off a corner",
            levels / corners as f64
        );
    }
    // the gate passes what was written (the repo's content, the files and manifests under the other root)
    let mixed = Paths {
        assets: out.assets.clone(),
        centres: out.centres.clone(),
        ..Paths::repo()
    };
    let report = checkglb::gate(&mixed, &["model.cypress".to_string()]);
    assert_eq!(report.failed, 0, "{}", report.text);
    assert!(
        report.checked.iter().any(|k| k == "model.cypress"),
        "{}",
        report.text
    );
    let _ = std::fs::remove_dir_all(&root);
}
