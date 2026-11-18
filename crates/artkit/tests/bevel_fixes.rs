//! Faults found in the bevel by the families built with it, each held to what Blender 5.2 makes (tests/golden).
mod common;

use artkit::materials::{Finish, Finishes, Mat};
use artkit::math::V3;
use artkit::scene;
use artkit::shapes::{Cube, cube};
use common::*;
use std::collections::BTreeMap;

/// A golden's records: each one's name, its `key=value` words and its lines by their first word.
struct Rec {
    name: String,
    kv: BTreeMap<String, String>,
    lines: BTreeMap<String, String>,
}

impl Rec {
    fn floats(&self, key: &str) -> Vec<f32> {
        let hex = &self.lines[key];
        (0..hex.len() / 8)
            .map(|i| f32w(&hex[i * 8..i * 8 + 8]))
            .collect()
    }
    fn points(&self, key: &str) -> Vec<V3> {
        self.floats(key)
            .chunks(3)
            .map(|c| V3::new(c[0], c[1], c[2]))
            .collect()
    }
    fn faces(&self, key: &str) -> Vec<Vec<u32>> {
        self.lines[key]
            .split(' ')
            .filter(|w| !w.is_empty())
            .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
            .collect()
    }
}

fn records(file: &str) -> Vec<Rec> {
    let mut out: Vec<Rec> = Vec::new();
    for line in golden(file).lines().filter(|l| !l.starts_with('#')) {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        if key == "rec" {
            let mut words = rest.split(' ');
            let name = words.next().unwrap().to_string();
            let kv = words
                .filter_map(|w| w.split_once('='))
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect();
            out.push(Rec {
                name,
                kv,
                lines: BTreeMap::new(),
            });
        } else {
            out.last_mut()
                .unwrap()
                .lines
                .insert(key.to_string(), rest.to_string());
        }
    }
    out
}

fn finishes() -> Finishes {
    let mut finishes = Finishes::new();
    for (name, textured) in [("satin", false), ("wood", true)] {
        finishes.insert(
            name.into(),
            Finish {
                roughness: 0.5,
                metallic: 0.0,
                textured,
            },
        );
    }
    finishes
}

#[test]
fn a_joined_parts_texture_stays_in_its_own_frame_whichever_part_is_bevelled() {
    // a bevelled part on a plain finish joined first, a textured one after it: the textured part's UVs are the
    // ones made in its own frame at the join, and are not made again over the whole at export
    let mut report = Vec::new();
    for r in records("bevel_join_uv.txt") {
        scene::reset();
        scene::set_finishes(finishes());
        let parts: Vec<_> = r
            .name
            .split('+')
            .enumerate()
            .map(|(i, what)| {
                let wood = what.starts_with("wood");
                let material = if wood {
                    Mat::new("wood", "#8A5A2B", "wood")
                } else {
                    Mat::new("body", "#C0683F", "satin")
                };
                let bevel = match (what.ends_with("_bevelled"), wood) {
                    (false, _) => 0.0,
                    (true, false) => 0.01,
                    (true, true) => 0.02,
                };
                let (size, at) = if i == 0 {
                    ((0.2, 0.3, 0.4), (0.0, 0.0, 0.0))
                } else {
                    ((0.5, 0.12, 0.2), (1.0, 0.25, 0.5))
                };
                cube(
                    format!("p{i}"),
                    size,
                    Cube {
                        at: at.into(),
                        material: Some(material),
                        bevel,
                        bevel_segments: 1,
                        ..Cube::default()
                    },
                )
            })
            .collect();
        let joined = artkit::parts::join(&parts, "j", None);
        assert_eq!(
            joined.read(|o| o.mesh.uv_metres),
            r.kv["flag"] == "1",
            "{}: whether its UVs are made",
            r.name
        );
        // (as the model's writing sees it again)
        assert!(artkit::uv::ensure(&[joined]));
        let mesh = joined.read(|o| o.mesh.clone());
        let want_points = r.points("v");
        assert_eq!(mesh.verts.len(), want_points.len(), "{}", r.name);
        for (g, w) in mesh.verts.iter().zip(&want_points) {
            assert!((0..3).all(|i| g[i] == w[i]), "{}: {g:?} {w:?}", r.name);
        }
        // Blender's faces by their corners, each corner's UV by its vertex
        let (want_faces, want_uv) = (r.faces("f"), r.floats("uv"));
        let mut golden: BTreeMap<Vec<u32>, Vec<(u32, [u32; 2])>> = BTreeMap::new();
        let mut at = 0;
        for f in &want_faces {
            let mut key = f.clone();
            key.sort_unstable();
            let corners = f
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    let uv = [want_uv[2 * (at + i)], want_uv[2 * (at + i) + 1]];
                    (v, [uv[0].to_bits(), uv[1].to_bits()])
                })
                .collect();
            golden.insert(key, corners);
            at += f.len();
        }
        let uvs = mesh.uvs.as_ref().expect("a textured model's UVs");
        let slots: Vec<&str> = mesh.mats.iter().map(|m| m.finish).collect();
        let (mut off, mut textured, mut plain_not_zero) = (0, 0, 0);
        let mut far = 0.0_f32;
        for f in 0..mesh.faces.len() {
            let face = mesh.faces[f];
            let mut key = mesh.corners(f).to_vec();
            key.sort_unstable();
            let want = golden
                .get(&key)
                .unwrap_or_else(|| panic!("{}: a face that isn't Blender's", r.name));
            for (i, &v) in mesh.corners(f).iter().enumerate() {
                let got = uvs[face.start as usize + i];
                let want = want.iter().find(|(w, _)| *w == v).unwrap().1;
                if slots[face.mat as usize] == "wood" {
                    textured += 1;
                    off += ([got[0].to_bits(), got[1].to_bits()] != want) as usize;
                    for k in 0..2 {
                        far = far.max((got[k] - f32::from_bits(want[k])).abs());
                    }
                } else {
                    plain_not_zero += (got != [0.0, 0.0] || want != [0, 0]) as usize;
                }
            }
        }
        report.push(format!(
            "{}: {off} of {textured} textured corners not Blender's to the bit (within {far:e}), {plain_not_zero} plain ones not zero",
            r.name
        ));
        // (a bevelled part's texture is carried onto its bevel within a rounding of Blender's)
        let bevelled = r.name.contains("wood_bevelled");
        assert!(
            plain_not_zero == 0 && if bevelled { far < 1e-7 } else { off == 0 },
            "{}",
            report.last().unwrap()
        );
    }
    println!("{}", report.join("\n"));
}
