//! Bevels, slabs and their fills against Blender 5.2's (tests/golden/bevel.txt: parts of the families as the bevel
//! modifier and `forms.extrude` made them, captured as each family was built; two as they were once joined, changed
//! after their bevel was asked for).
mod common;

use artkit::bevel::{EdgeOrder, Kind, apply, bevelled, carried};
use artkit::forms::{Extrude, Holes, Plane, extrude};
use artkit::math::V3;
use artkit::mesh::Mesh;
use artkit::scene;
use artkit::shapes::{Cube, Round, cube, cylinder};
use common::*;
use std::collections::BTreeMap;

/// One part of the golden: what was asked for, the mesh the modifier was given and what it made.
struct Rec {
    name: String,
    kv: BTreeMap<String, String>,
    lines: BTreeMap<String, String>,
}

impl Rec {
    fn f64(&self, key: &str) -> f64 {
        f64w(&self.kv[key])
    }
    fn f64s(&self, key: &str) -> Vec<f64> {
        self.kv[key].split(',').map(f64w).collect()
    }
    fn points(&self, key: &str) -> Vec<V3> {
        let hex = &self.lines[key];
        let x: Vec<f32> = (0..hex.len() / 8)
            .map(|i| f32w(&hex[i * 8..i * 8 + 8]))
            .collect();
        x.chunks(3).map(|c| V3::new(c[0], c[1], c[2])).collect()
    }
    fn faces(&self, key: &str) -> Vec<Vec<u32>> {
        self.lines[key]
            .split(' ')
            .filter(|w| !w.is_empty())
            .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
            .collect()
    }
    fn loops(&self) -> Vec<Vec<(f64, f64)>> {
        self.lines["loops"]
            .split(" | ")
            .map(|l| {
                l.split(' ')
                    .filter(|w| !w.is_empty())
                    .map(|p| {
                        let (a, b) = p.split_once(',').unwrap();
                        (f64w(a), f64w(b))
                    })
                    .collect()
            })
            .collect()
    }
    fn width(&self) -> f64 {
        self.f64("width")
    }
    fn segments(&self) -> usize {
        self.kv["seg"].parse().unwrap()
    }
    /// The mesh the modifier was given.
    fn base(&self) -> Mesh {
        let mut base = Mesh::from_pydata(self.points("bv"), &self.faces("bf"));
        for (f, c) in base.faces.iter_mut().zip(self.lines["bs"].chars()) {
            f.smooth = c == '1';
        }
        base.sharp = self.lines["be"]
            .split(' ')
            .filter(|w| !w.is_empty())
            .map(|e| {
                let (a, b) = e.split_once('-').unwrap();
                artkit::mesh::edge(a.parse().unwrap(), b.parse().unwrap())
            })
            .collect();
        base
    }
    /// How Blender had numbered its edges.
    fn order(&self) -> EdgeOrder {
        match self.kv["kind"].as_str() {
            "cube" => EdgeOrder::Faces,
            "cylinder" => {
                let n: usize = self.kv["verts"].parse().unwrap();
                EdgeOrder::Listed(artkit::shapes::primitive::cone_edges(n))
            }
            _ => EdgeOrder::Corners,
        }
    }
}

fn records() -> Vec<Rec> {
    let mut out: Vec<Rec> = Vec::new();
    for line in golden("bevel.txt").lines() {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "#" => {}
            "rec" => {
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
            }
            _ => {
                out.last_mut()
                    .unwrap()
                    .lines
                    .insert(key.to_string(), rest.to_string());
            }
        }
    }
    out
}

/// How far a part's points may be from Blender's: a quarter of a thousandth of a millimetre (the jukebox's arch, a
/// cube corner's patch laid onto points a metre from the origin: under a thousandth).
fn reach(name: &str) -> f32 {
    if name.ends_with("jukebox/arch") {
        1e-6
    } else {
        2.5e-7
    }
}

fn apart(a: V3, b: V3) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).abs()).fold(0.0, f32::max)
}

/// The farthest any point is from the golden's point of the same number.
fn in_order(got: &[V3], want: &[V3]) -> f32 {
    assert_eq!(got.len(), want.len(), "points");
    got.iter()
        .zip(want)
        .map(|(a, b)| apart(*a, *b))
        .fold(0.0, f32::max)
}

/// Each point's golden point (the nearest, each taken once) and the farthest apart.
fn paired(got: &[V3], want: &[V3]) -> (Vec<u32>, f32) {
    assert_eq!(got.len(), want.len(), "points");
    let mut taken = vec![false; want.len()];
    let mut map = Vec::new();
    let mut far = 0.0_f32;
    for g in got {
        let best = (0..want.len())
            .filter(|&j| !taken[j])
            .min_by(|&a, &b| apart(*g, want[a]).total_cmp(&apart(*g, want[b])))
            .unwrap();
        taken[best] = true;
        map.push(best as u32);
        far = far.max(apart(*g, want[best]));
    }
    (map, far)
}

/// The most any corner's normal is turned from Blender's (degrees), over the faces that have an area, and how many
/// faces aren't Blender's at all.
fn normals_off(mesh: &Mesh, map: &[u32], r: &Rec) -> (f32, usize) {
    let (of, on) = (r.faces("of"), r.points("on"));
    let mut golden: BTreeMap<Vec<u32>, Vec<(u32, V3)>> = BTreeMap::new();
    let mut at = 0;
    for f in &of {
        let mut key = f.clone();
        key.sort_unstable();
        golden.insert(
            key,
            f.iter()
                .enumerate()
                .map(|(i, &v)| (v, on[at + i]))
                .collect(),
        );
        at += f.len();
    }
    let mine = mesh.normals.as_ref().expect("a bevel's normals");
    let (mut worst, mut missing) = (0.0_f32, 0);
    for f in 0..mesh.faces.len() {
        let c: Vec<u32> = mesh.corners(f).iter().map(|&v| map[v as usize]).collect();
        let p: Vec<V3> = mesh
            .corners(f)
            .iter()
            .map(|&v| mesh.verts[v as usize])
            .collect();
        let area: f64 = (1..p.len() - 1)
            .map(|i| (p[i] - p[0]).cross(p[i + 1] - p[0]).length() / 2.0)
            .sum();
        let mut key = c.clone();
        key.sort_unstable();
        let Some(g) = golden.get(&key) else {
            missing += (area >= 1e-10) as usize;
            continue;
        };
        if area < 1e-10 {
            continue;
        }
        let start = mesh.faces[f].start as usize;
        for (i, v) in c.iter().enumerate() {
            let want = g.iter().find(|(w, _)| w == v).unwrap().1;
            let cos = mine[start + i].dot(want).clamp(-1.0, 1.0);
            worst = worst.max(cos.acos().to_degrees() as f32);
        }
    }
    (worst, missing)
}

#[test]
fn a_bevel_cuts_blenders_mesh_as_blender_does() {
    let mut report = Vec::new();
    let mut bad = 0;
    for r in records().iter().filter(|r| r.width() != 0.0) {
        let made = bevelled(
            &r.base(),
            r.width(),
            r.segments(),
            r.f64("angle"),
            &r.order(),
        )
        .unwrap();
        let want = r.points("ov");
        let kind = r.kv["kind"].as_str();
        // a box's and a cylinder's points come in Blender's order; a slab's (given here without the way Blender
        // numbers its edges) are the same points
        let (map, far) = if kind == "extrude" {
            paired(&made.mesh.verts, &want)
        } else {
            (
                (0..want.len() as u32).collect(),
                in_order(&made.mesh.verts, &want),
            )
        };
        let (turned, missing) = normals_off(&made.mesh, &map, r);
        assert_eq!(made.kinds.len(), made.mesh.faces.len());
        assert_eq!(made.source.len(), made.mesh.loops.len());
        report.push(format!("{} {kind} {} segments: points within {far:e}, normals within {turned:.3} degrees, {missing} faces not Blender's", r.name, r.segments()));
        // Blender reads the normals back in the fans of the mesh it leaves, the mean of what was stored where a
        // fan's corners were given unlike ones. On a slab given here as Blender had it, without the way its edges
        // ran, a fan can start elsewhere than Blender's, and that mean then reads on the other side of the middle
        // (the bookshelf's top, by 21 degrees round four of its profiles' middle points).
        // (The jukebox's arch, 20 mm of bevel on a 20 mm wall: its corners where the face's triangles meet take
        // a cube corner's patch, as in Blender; its normals round the clamped rim are within 7.5 degrees.)
        let turn = match r.name.rsplit('/').next().unwrap() {
            "top" => 21.0,
            "arch" => 7.5,
            _ => 0.1,
        };
        bad += (far > reach(&r.name) || turned > turn || missing > 0) as usize;
    }
    println!("{}", report.join("\n"));
    assert_eq!(bad, 0, "\n{}", report.join("\n"));
}

#[test]
fn a_bevelled_box_and_cylinder_are_blenders_point_for_point() {
    let mut report = Vec::new();
    let mut bad = 0;
    for r in records()
        .iter()
        .filter(|r| r.width() != 0.0 && r.kv["changed"] == "0")
    {
        scene::reset();
        let obj = match r.kv["kind"].as_str() {
            "cube" => {
                let s = r.f64s("size");
                cube(
                    "part",
                    (s[0], s[1], s[2]),
                    Cube {
                        bevel: r.width(),
                        bevel_segments: r.segments(),
                        ..Cube::default()
                    },
                )
            }
            "cylinder" => {
                let s = r.f64s("scale");
                cylinder(
                    "part",
                    r.f64("radius"),
                    r.f64("depth"),
                    Round {
                        verts: r.kv["verts"].parse().unwrap(),
                        scale: (s[0], s[1], s[2]).into(),
                        bevel: r.width(),
                        bevel_segments: r.segments(),
                        ..Round::default()
                    },
                )
            }
            _ => continue,
        };
        let got = apply(obj).read(|o| o.mesh.clone());
        let far = in_order(&got.verts, &r.points("ov"));
        assert!(
            got.faces.iter().all(|f| f.smooth),
            "{}: a bevelled part is shaded smooth",
            r.name
        );
        report.push(format!(
            "{}: points within {far:e} of Blender's, in its order",
            r.name
        ));
        bad += (far > 1e-7) as usize;
    }
    println!("{}", report.join("\n"));
    assert_eq!(bad, 0, "\n{}", report.join("\n"));
}

#[test]
fn a_slab_is_blenders_cut_for_cut() {
    let mut report = Vec::new();
    let mut bad = 0;
    for r in records()
        .iter()
        .filter(|r| r.kv["kind"] == "extrude" && r.kv["changed"] == "0")
    {
        let loops = r.loops();
        scene::reset();
        let plane = match r.kv["plane"].as_str() {
            "XZ" => Plane::XZ,
            "XY" => Plane::XY,
            _ => Plane::YZ,
        };
        let obj = extrude(
            "slab",
            &loops[0],
            r.f64("depth"),
            Extrude {
                holes: Holes(loops[1..].to_vec()),
                plane,
                bevel: r.width(),
                bevel_segments: r.segments(),
                smooth: r.kv["smooth"] == "1",
                angle: r.f64("sharp"),
                ..Extrude::default()
            },
        );
        let got = apply(obj).read(|o| o.mesh.clone());
        let bevelled = r.width() != 0.0;
        let want = r.points(if bevelled { "ov" } else { "bv" });
        let (map, far) = paired(&got.verts, &want);
        // the same triangles on its two faces (a plain slab's own; a bevelled one's are compared as its normals are)
        let mut faces_differ = 0;
        if !bevelled {
            let tris = |faces: Vec<Vec<u32>>| {
                let mut t: Vec<Vec<u32>> = faces
                    .into_iter()
                    .filter(|f| f.len() == 3)
                    .map(|mut f| {
                        f.sort_unstable();
                        f
                    })
                    .collect();
                t.sort();
                t
            };
            let mine = tris(
                (0..got.faces.len())
                    .map(|f| got.corners(f).iter().map(|&v| map[v as usize]).collect())
                    .collect(),
            );
            faces_differ = (mine != tris(r.faces("bf"))) as usize;
        }
        report.push(format!("{} ({} outlines): points within {far:e} of Blender's, triangles differ: {faces_differ}", r.name, loops.len()));
        bad += (far > reach(&r.name) || faces_differ > 0) as usize;
    }
    println!("{}", report.join("\n"));
    assert_eq!(bad, 0, "\n{}", report.join("\n"));
}

#[test]
fn a_bevel_keeps_flat_faces_flat_and_rounds_between_them() {
    // a box, one segment: each of its six faces keeps its normal at all its corners, each strip's corners take
    // the normal of the face they touch, each corner triangle's the three faces'
    scene::reset();
    let obj = cube(
        "box",
        (0.4, 0.3, 0.2),
        Cube {
            bevel: 0.01,
            bevel_segments: 1,
            ..Cube::default()
        },
    );
    // (asked for, the bevel is still to be given: the part keeps its mesh until it is joined or written)
    assert_eq!(obj.read(|o| o.mesh.verts.len()), 8);
    let mesh = apply(obj).read(|o| o.mesh.clone());
    assert_eq!((mesh.verts.len(), mesh.faces.len()), (24, 26));
    let made = bevelled(
        &artkit::shapes::primitive::cube(),
        0.1,
        1,
        40.0,
        &EdgeOrder::Faces,
    )
    .unwrap();
    assert_eq!(made.kinds.iter().filter(|k| **k == Kind::Corner).count(), 8);
    assert_eq!(made.kinds.iter().filter(|k| **k == Kind::Strip).count(), 12);
    assert_eq!(made.kinds.iter().filter(|k| **k == Kind::Cut).count(), 6);
    let normals = made.mesh.normals.as_ref().unwrap();
    for (f, kind) in made.kinds.iter().enumerate() {
        let face = made.mesh.faces[f];
        for l in face.start..face.start + face.len {
            let n = normals[l as usize];
            // every corner's normal is one of the six faces'
            let axis = (0..3).filter(|&i| n[i].abs() > 0.999).count();
            assert_eq!(axis, 1, "{kind:?} {n:?}");
        }
    }
    // two segments on a box: 54 quads, a quarter turn round each edge
    let made = bevelled(
        &artkit::shapes::primitive::cube(),
        0.1,
        2,
        40.0,
        &EdgeOrder::Faces,
    )
    .unwrap();
    assert_eq!((made.mesh.verts.len(), made.mesh.faces.len()), (56, 54));
    assert!(made.mesh.faces.iter().all(|f| f.len == 4));
    // nothing to bevel on a flat sheet; an open edge isn't bevelled
    let sheet = Mesh::from_pydata(
        vec![
            V3::ZERO,
            V3::new(1.0, 0.0, 0.0),
            V3::new(1.0, 1.0, 0.0),
            V3::new(0.0, 1.0, 0.0),
        ],
        &[vec![0, 1, 2, 3]],
    );
    assert_eq!(
        bevelled(&sheet, 0.1, 1, 40.0, &EdgeOrder::Corners)
            .unwrap()
            .mesh
            .verts
            .len(),
        4
    );
}

#[test]
fn a_turned_bevelled_part_keeps_its_normals_on_its_faces() {
    // joined into another part's frame, a turned box's hardened normals turn with it
    scene::reset();
    let a = cube("a", 0.2, Cube::default());
    let b = cube(
        "b",
        (0.2, 0.1, 0.1),
        Cube {
            at: (0.5, 0.0, 0.0).into(),
            rot: (0.0, 0.0, 90.0).into(),
            bevel: 0.01,
            bevel_segments: 1,
            ..Cube::default()
        },
    );
    let joined = artkit::parts::join(&[a, b], "joined", None);
    let mesh = joined.read(|o| o.mesh.clone());
    let normals = mesh.normals.as_ref().unwrap();
    let face_normals = mesh.face_normals();
    // b's six cut faces are the ones of four corners whose normals agree: each normal is the face's own
    let mut checked = 0;
    for f in 6..mesh.faces.len() {
        let face = mesh.faces[f];
        let own: Vec<V3> = (face.start..face.start + face.len)
            .map(|l| normals[l as usize])
            .collect();
        if face.len == 4 && own.iter().all(|n| apart(*n, own[0]) < 1e-5) {
            assert!(
                apart(own[0], face_normals[f]) < 1e-5,
                "{:?} {:?}",
                own[0],
                face_normals[f]
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 6);
}

#[test]
fn a_bevel_rounds_the_part_as_it_is_when_it_is_joined() {
    use artkit::math::{TAU, cosd, sind, v3};
    use artkit::noise::noise;
    let records = records();
    let rec = |name: &str| records.iter().find(|r| r.name == name).unwrap();
    let mat = artkit::materials::Mat::new("pit", "#8C867E", "matte");
    // the fire pit's first stone (outdoor.py): a bevelled box turned to face the fire, then pushed about by noise
    // (the script's own `_jitter`). Blender's modifier rounded the pushed box.
    scene::reset();
    let f = artkit::place::GameFrame::new(0.0);
    let (a, w) = (0.0_f64, TAU * 0.39 / 11.0 - 0.012);
    let stone = artkit::fbox!(
        f,
        "stone0_0",
        w,
        0.14 - 0.008,
        0.1,
        (0.39 * cosd(a), 0.07, 0.39 * sind(a)),
        mat,
        rot = (0, 0, a - 90.0),
        bevel = 0.018,
        bevel_segments = 1
    );
    let (amount, size, seed) = (0.008, 0.07, 0.0);
    let world = stone.world_points();
    stone.edit(|part| {
        for (v, p) in part.mesh.verts.iter_mut().zip(&world) {
            let q = v3(
                p.x as f64 / size + seed,
                p.y as f64 / size - seed,
                p.z as f64 / size + seed * 0.5,
            );
            let n = [
                noise(q),
                noise(q + v3(31.4, 0, 0)),
                noise(q + v3(0, 47.1, 0)),
            ];
            *v += v3(n[0], n[1], n[2]) * amount;
        }
    });
    let r = rec("outdoor/firePit/stone0_0@joined");
    let pushed = stone.read(|o| o.mesh.verts.clone());
    assert_eq!(in_order(&pushed, &r.points("bv")), 0.0, "the pushed box");
    let got = apply(stone).read(|o| o.mesh.clone());
    assert_eq!(in_order(&got.verts, &r.points("ov")), 0.0, "the stone");
    let (map, _) = paired(&got.verts, &r.points("ov"));
    let (turned, missing) = normals_off(&got, &map, r);
    assert!(turned < 0.1 && missing == 0, "{turned} {missing}");
    let joined = artkit::parts::join(&[stone], "joined", None);
    let got = joined.read(|o| o.mesh.verts.clone());
    assert_eq!(in_order(&got, &r.points("jv")), 0.0, "the stone, joined");

    // the farmhouse table's brace (dining.py's `_board`): a bevelled box whose part is then turned, by a matrix
    // that leaves its scale a unit in the last place off one. The modifier rounds the box in its own frame; the
    // join takes the rounded box through the part's matrix.
    let r = rec("dining/tableFarmhouse6/brace-1.15-1@joined");
    let s = r.f64s("size");
    scene::reset();
    let brace = cube(
        "brace",
        (s[0], s[1], s[2]),
        Cube {
            bevel: r.width(),
            ..Cube::default()
        },
    );
    // (its place, turn and scale as Blender took the matrix apart)
    let lrs = r.points("lrs");
    brace.edit(|o| (o.loc, o.rot, o.scale) = (lrs[0], lrs[1], lrs[2]));
    assert!(lrs[2].x != 1.0);
    let joined = artkit::parts::join(&[brace], "joined", None);
    let got = joined.read(|o| o.mesh.verts.clone());
    assert_eq!(got.len(), r.points("jv").len());
    assert_eq!(in_order(&got, &r.points("jv")), 0.0, "the brace, joined");
    assert_eq!(joined.read(|o| o.loc), r.points("jl")[0]);
}

#[test]
fn a_bevels_corners_take_their_texture_from_the_face_blender_takes_it_from() {
    // each face of a box and of a 12-sided cylinder numbered in its UVs: the number each new corner comes out with
    let text = golden("bevel_uv.txt");
    let mut recs: Vec<(String, BTreeMap<String, String>)> = Vec::new();
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        if key == "rec" {
            recs.push((rest.to_string(), BTreeMap::new()));
        } else {
            recs.last_mut()
                .unwrap()
                .1
                .insert(key.to_string(), rest.to_string());
        }
    }
    let points = |hex: &str| -> Vec<V3> {
        let x: Vec<f32> = (0..hex.len() / 8)
            .map(|i| f32::from_bits(u32::from_str_radix(&hex[8 * i..8 * i + 8], 16).unwrap()))
            .collect();
        x.chunks(3).map(|c| V3::new(c[0], c[1], c[2])).collect()
    };
    let faces = |s: &str| -> Vec<Vec<u32>> {
        s.split(' ')
            .map(|f| f.split(',').map(|v| v.parse().unwrap()).collect())
            .collect()
    };
    let mut report = Vec::new();
    for (name, lines) in &recs {
        let base = {
            let mut m = Mesh::from_pydata(points(&lines["bv"]), &faces(&lines["bf"]));
            m.shade_smooth(40.0);
            m
        };
        let segments: usize = name[name.len() - 1..].parse().unwrap();
        let (width, order) = if name.starts_with("cube") {
            (0.02, EdgeOrder::Faces)
        } else {
            let n = 12;
            let rims = (0..n).flat_map(|i| {
                let j = (i + 1) % n;
                [(2 * i, 2 * j), (2 * j + 1, 2 * i + 1)]
            });
            (0.01, EdgeOrder::Listed(rims.collect()))
        };
        let made = bevelled(&base, width, segments, 40.0, &order).unwrap();
        let uvs: Vec<[f32; 2]> = (0..base.faces.len())
            .flat_map(|f| std::iter::repeat_n([f as f32, 0.0], base.faces[f].len as usize))
            .collect();
        let got = carried(&base, &uvs, &made.mesh, &made, None);
        let want_points = points(&lines["ov"]);
        let (map, far) = paired(&made.mesh.verts, &want_points);
        assert!(far < 1e-6, "{name}: {far}");
        let want: BTreeMap<Vec<u32>, Vec<f32>> = faces(&lines["of"])
            .into_iter()
            .zip(lines["src"].split(' '))
            .map(|(mut f, s)| {
                f.sort();
                let mut s: Vec<f32> = s.split(',').map(|x| x.parse().unwrap()).collect();
                s.sort_by(f32::total_cmp);
                s.dedup();
                (f, s)
            })
            .collect();
        let mut off = [0; 4];
        let mut all = [0; 4];
        for f in 0..made.mesh.faces.len() {
            let face = made.mesh.faces[f];
            let mut key: Vec<u32> = made
                .mesh
                .corners(f)
                .iter()
                .map(|&v| map[v as usize])
                .collect();
            key.sort();
            let mut mine: Vec<f32> = (face.start..face.start + face.len)
                .map(|l| got[l as usize][0])
                .collect();
            mine.sort_by(f32::total_cmp);
            mine.dedup();
            let kind = match made.kinds[f] {
                Kind::Kept => 0,
                Kind::Cut => 1,
                Kind::Strip => 2,
                Kind::Corner => 3,
            };
            all[kind] += 1;
            if want.get(&key) != Some(&mine) {
                off[kind] += 1;
            }
        }
        report.push(format!(
            "{name}: faces whose texture isn't from Blender's face: {} of {} cut, {} of {} strips, {} of {} of the corners'",
            off[1], all[1], off[2], all[2], off[3], all[3]
        ));
        // the faces themselves, the strips, a corner's polygon and its patch of two or three segments
        assert_eq!(off, [0; 4], "{}", report.last().unwrap());
    }
    println!("{}", report.join("\n"));
}

#[test]
fn a_bevelled_plain_part_joined_first_leaves_the_others_their_texture() {
    use artkit::materials::{Finish, Finishes, Mat};
    // a plain, bevelled part first, a wooden one beside it: the wood's UVs are its own, in its own frame, whether
    // or not the first part has a bevel (they were made again over the whole joined mesh)
    let wood_uvs = |bevel: f64| -> Vec<[u32; 2]> {
        scene::reset();
        let f = |roughness, textured| Finish {
            roughness,
            metallic: 0.0,
            textured,
        };
        let finishes: Finishes = [("matte", f(0.75, false)), ("wood", f(0.8, true))]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        scene::set_finishes(finishes);
        let frame = artkit::place::GameFrame::new(0.0);
        let a = artkit::fbox!(
            frame,
            "a",
            0.2,
            0.2,
            0.2,
            (0, 0.1, 0),
            Mat::new("plinth", "#8C867E", "matte"),
            bevel = bevel,
            bevel_segments = 1
        );
        let b = artkit::fbox!(
            frame,
            "b",
            0.2,
            0.6,
            0.3,
            (1, 0.3, 0),
            Mat::new("body", "#B88A5A", "wood"),
            bevel = 0
        );
        let joined = artkit::parts::join(&[a, b], "j", None);
        joined.read(|o| {
            assert!(o.mesh.uv_metres, "bevel {bevel}: the joined UVs are final");
            let uvs = o.mesh.uvs.as_ref().expect("a textured model has UVs");
            let mut out: Vec<[u32; 2]> = o
                .mesh
                .faces
                .iter()
                .filter(|face| face.mat == 1)
                .flat_map(|face| &uvs[face.start as usize..(face.start + face.len) as usize])
                .map(|uv| [uv[0].to_bits(), uv[1].to_bits()])
                .collect();
            out.sort();
            out
        })
    };
    let (plain, bevelled) = (wood_uvs(0.0), wood_uvs(0.01));
    assert!(plain.iter().any(|uv| *uv != [0, 0]), "the wood has UVs");
    assert_eq!(plain, bevelled);
}
