//! Faults found in the bevel by the families built with it, each held to what Blender 5.2 makes (tests/golden).
mod common;

use artkit::forms::{Extrude, Holes, Plane, extrude};
use artkit::materials::{Finish, Finishes, Mat};
use artkit::math::{P3, V3};
use artkit::mesh::Mesh;
use artkit::scene;
use artkit::shapes::{Cube, Round, cube, cylinder};
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
    fn f64(&self, key: &str) -> f64 {
        f64w(&self.kv[key])
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

/// A place: the first of Blender's points within a thousandth of a millimetre of it (a clamped bevel leaves
/// two points in one place, which no numbering tells apart).
type Place = usize;

fn place(p: V3, points: &[V3]) -> Place {
    let apart = |q: &V3| (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0_f32, f32::max);
    points
        .iter()
        .position(|q| apart(q) < 1e-6)
        .unwrap_or(usize::MAX)
}

/// How a mesh's corner normals are off Blender's, each face found by where its corners are (a slab's points come
/// in another order than Blender's): (the most any is turned, in degrees, over the faces that have an area; the
/// share of the mesh's area on faces with a corner over 15 degrees off; the faces that aren't Blender's).
fn normals_off(
    mesh: &Mesh,
    points: &[V3],
    faces: &[Vec<u32>],
    normals: &[V3],
) -> (f64, f64, usize) {
    let mut golden: BTreeMap<Vec<Place>, Vec<Vec<(Place, V3)>>> = BTreeMap::new();
    let mut at = 0;
    for f in faces {
        let corners: Vec<(Place, V3)> = f
            .iter()
            .enumerate()
            .map(|(i, &v)| (place(points[v as usize], points), normals[at + i]))
            .collect();
        let mut key: Vec<Place> = corners.iter().map(|c| c.0).collect();
        key.sort_unstable();
        golden.entry(key).or_default().push(corners);
        at += f.len();
    }
    let mine = artkit::normals::corner_normals(mesh);
    let (mut worst, mut off, mut all, mut missing) = (0.0_f64, 0.0_f64, 0.0_f64, 0);
    for f in 0..mesh.faces.len() {
        let p: Vec<V3> = mesh
            .corners(f)
            .iter()
            .map(|&v| mesh.verts[v as usize])
            .collect();
        let area: f64 = (1..p.len() - 1)
            .map(|i| (p[i] - p[0]).cross(p[i + 1] - p[0]).length() / 2.0)
            .sum();
        all += area;
        if area < 1e-10 {
            continue;
        }
        let places: Vec<Place> = p.iter().map(|&q| place(q, points)).collect();
        let mut key = places.clone();
        key.sort_unstable();
        let Some(want) = golden.get(&key) else {
            missing += 1;
            continue;
        };
        // (two faces on the same points, a clamped bevel's: the one it is nearer)
        let mut turned = f64::INFINITY;
        for want in want {
            let mut most = 0.0_f64;
            for (i, q) in places.iter().enumerate() {
                // (a face that comes by a place twice: which corner is which isn't told)
                if places.iter().filter(|x| *x == q).count() > 1 {
                    continue;
                }
                let (n, w) = (
                    mine[mesh.faces[f].start as usize + i],
                    want.iter().find(|c| c.0 == *q).unwrap().1,
                );
                // (as the file is written: a normal's length isn't kept)
                let cos = n.dot(w) / (n.length() * w.length()).max(1e-30);
                if (0..3).any(|k| n[k] != w[k]) {
                    most = most.max(cos.clamp(-1.0, 1.0).acos().to_degrees());
                }
            }
            turned = turned.min(most);
        }
        worst = worst.max(turned);
        if turned > 15.0 {
            off += area;
        }
    }
    (worst, off / all, missing)
}

#[test]
fn a_bevels_normals_are_read_on_the_part_where_it_is_joined() {
    // Blender keeps a bevel's hardened normals as two angles in the frame of each corner's fan, through a join
    // and a move, and reads them on the mesh as it is when it is written. A plate thinner than two bevels (its
    // sides slivers thinner than its points' rounding once it is moved) and a slab's clamped corners then read
    // otherwise than where the bevel was made: the parts are joined after another, off the origin, as a family
    // joins them.
    let mut report = Vec::new();
    let material = Some(Mat::new("body", "#C0683F", "satin"));
    for r in records("bevel_placed.txt") {
        scene::reset();
        let first = cube(
            "first",
            (0.1, 0.06, 0.04),
            Cube {
                at: (0.05, -0.3, 0.31).into(),
                material,
                ..Cube::default()
            },
        );
        let (at, bevel, bevel_segments) = (
            P3::from((-0.2, 0.632, 0.52)),
            r.f64("width"),
            r.kv["seg"].parse().unwrap(),
        );
        let part = if r.kv["kind"] == "cube" {
            let s: Vec<f64> = r.kv["size"].split(',').map(f64w).collect();
            cube(
                "part",
                (s[0], s[1], s[2]),
                Cube {
                    at,
                    material,
                    bevel,
                    bevel_segments,
                    ..Cube::default()
                },
            )
        } else {
            let loops = r.loops();
            extrude(
                "part",
                &loops[0],
                r.f64("depth"),
                Extrude {
                    at,
                    material,
                    holes: Holes(loops[1..].to_vec()),
                    plane: if r.kv["plane"] == "XY" {
                        Plane::XY
                    } else {
                        Plane::XZ
                    },
                    bevel,
                    bevel_segments,
                    ..Extrude::default()
                },
            )
        };
        let joined = artkit::parts::join(&[first, part], "joined", Some(P3::ZERO));
        let mesh = joined.read(|o| o.mesh.clone());
        // the same points, to the bit
        let (want_points, want_faces) = (r.points("v"), r.faces("f"));
        let bits = |points: &[V3]| {
            let mut b: Vec<[u32; 3]> = points
                .iter()
                .map(|p| [p.x, p.y, p.z].map(|c| (c + 0.0).to_bits()))
                .collect();
            b.sort_unstable();
            b
        };
        if r.name == "slab/chalkboard" {
            // (ten of its profiles' middle points are a unit in the last place off)
            let want: Vec<[f32; 3]> = want_points.iter().map(|p| [p.x, p.y, p.z]).collect();
            let (_, _, far) = pair_points(&mesh.verts, &want, 1e-7).expect("its points");
            assert!(far < 1e-7, "{}: points within {far:e}", r.name);
        } else {
            assert_eq!(bits(&mesh.verts), bits(&want_points), "{}: points", r.name);
        }
        let (turned, share, missing) =
            normals_off(&mesh, &want_points, &want_faces, &r.points("n"));
        // (what keeping the normals as they were given through the move comes to)
        let mut kept = mesh.clone();
        kept.stored = None;
        let before = normals_off(&kept, &want_points, &want_faces, &r.points("n"));
        report.push(format!(
            "{}: normals within {turned:.3} degrees of Blender's, {missing} faces not Blender's (kept as given: within {:.1} degrees, {:.2} % of its surface over 15 off)",
            r.name,
            before.0,
            before.1 * 100.0
        ));
        assert!(
            turned < 0.05 && share == 0.0 && missing == 0,
            "{}",
            report.last().unwrap()
        );
        // the file's normals: unit vectors all, straight up where Blender reads none (as its exporter writes them)
        let none = r.points("n").iter().filter(|n| **n == V3::ZERO).count();
        let file = artkit::glb::read(&artkit::glb::write(&[joined], false, &finishes())).unwrap();
        let written: Vec<[f32; 3]> = file
            .meshes
            .iter()
            .flat_map(|m| &m.primitives)
            .flat_map(|p| p.normals.iter().copied())
            .collect();
        for n in &written {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-4,
                "{}: a normal {n:?} in its file",
                r.name
            );
        }
        let up = written
            .iter()
            .filter(|n| n[0] == 0.0 && n[1].abs() == 1.0 && n[2] == 0.0)
            .count();
        let mine = artkit::normals::corner_normals(&mesh);
        let zero = mine.iter().filter(|n| **n == V3::ZERO).count();
        report.push(format!(
            "{}: Blender reads no normal at {none} corners, the kernel at {zero}; {up} written straight up",
            r.name
        ));
        if r.name == "plate/strip" || r.name == "plate/light" {
            assert!(
                none == 8 && zero == 8 && up > 0,
                "{}",
                report.last().unwrap()
            );
        }
        // (a plate thinner than two bevels never reads as where it was made)
        assert!(
            before.0 > 1.0 || !r.name.starts_with("plate/"),
            "{}",
            report.last().unwrap()
        );
    }
    println!("{}", report.join("\n"));
}

#[test]
fn a_slabs_far_face_is_numbered_as_blenders_extrusion_numbers_it() {
    // Blender's extrusion copies a slab's face triangle by triangle: the copy's points come in the order the
    // triangles first name them (and its rim's edges in the order they first run them, which decides where a
    // bevel of two segments or more starts a fan). The slabs of tests/golden/bevel.txt, as the modifier was given
    // them: every point Blender's, in its order. (The fridge's kick plate, a grille of slots, is filled in another
    // order than Blender fills it: its triangles are Blender's, their order is not, nor its far points' then.)
    let mut seen = 0;
    for r in records("bevel.txt") {
        if r.kv["kind"] != "extrude"
            || r.kv["changed"] != "0"
            || r.name == "kitchen/fridge/kick_plate"
        {
            continue;
        }
        scene::reset();
        let loops = r.loops();
        let slab = extrude(
            "slab",
            &loops[0],
            r.f64("depth"),
            Extrude {
                holes: Holes(loops[1..].to_vec()),
                plane: match r.kv["plane"].as_str() {
                    "XZ" => Plane::XZ,
                    "XY" => Plane::XY,
                    _ => Plane::YZ,
                },
                ..Extrude::default()
            },
        );
        let (got, want) = (slab.read(|o| o.mesh.verts.clone()), r.points("bv"));
        assert_eq!(got.len(), want.len(), "{}", r.name);
        for (i, (g, w)) in got.iter().zip(&want).enumerate() {
            assert!(
                (0..3).all(|k| g[k] == w[k]),
                "{}: point {i} {g:?}, Blender's {w:?}",
                r.name
            );
        }
        seen += 1;
    }
    assert!(seen >= 8, "{seen} slabs");
}

#[test]
fn a_bevelled_slabs_points_come_in_blenders_order() {
    // The baked shading turns each point's rays by the point's number, and the faces name their points by it: a
    // slab whose bevel has Blender's points in another order bakes other levels all over. Blender starts each
    // corner's new points from an edge its own numbering of the mesh's edges picks (`bevel::bevelled`); the slabs
    // of tests/golden/bevel_order.txt and the bevelled ones of bevel.txt, built by `extrude` as asked for: every
    // point where Blender has the point of its number.
    let mut report = Vec::new();
    let (mut seen, mut bad) = (0, 0);
    for file in ["bevel_order.txt", "bevel.txt"] {
        for r in records(file) {
            if r.kv["kind"] != "extrude" || r.kv["changed"] != "0" || r.f64("width") == 0.0 {
                continue;
            }
            scene::reset();
            let loops = r.loops();
            let slab = extrude(
                "slab",
                &loops[0],
                r.f64("depth"),
                Extrude {
                    holes: Holes(loops[1..].to_vec()),
                    plane: match r.kv["plane"].as_str() {
                        "XZ" => Plane::XZ,
                        "XY" => Plane::XY,
                        _ => Plane::YZ,
                    },
                    bevel: r.f64("width"),
                    bevel_segments: r.kv["seg"].parse().unwrap(),
                    smooth: r.kv["smooth"] == "1",
                    angle: r.f64("sharp"),
                    ..Extrude::default()
                },
            );
            let got = artkit::bevel::apply(slab).read(|o| o.mesh.verts.clone());
            let want = r.points("ov");
            assert_eq!(got.len(), want.len(), "{}", r.name);
            let apart = |g: &V3, w: &V3| (0..3).map(|k| (g[k] - w[k]).abs()).fold(0.0, f32::max);
            let exact = got
                .iter()
                .zip(&want)
                .filter(|(g, w)| apart(g, w) == 0.0)
                .count();
            let out_of_place = got
                .iter()
                .zip(&want)
                .filter(|(g, w)| apart(g, w) > 1e-6)
                .count();
            report.push(format!(
                "{} ({} outlines, {} segments): {out_of_place} of {} points out of place, {exact} Blender's to the bit",
                r.name,
                loops.len(),
                r.kv["seg"],
                got.len()
            ));
            seen += 1;
            bad += (out_of_place > 0) as usize;
        }
    }
    println!("{}", report.join("\n"));
    assert!(seen >= 20, "{seen} slabs");
    assert_eq!(bad, 0, "\n{}", report.join("\n"));
}

#[test]
fn a_bevels_hardened_normals_are_stored_as_blenders_two_angles() {
    // Blender keeps each hardened normal as two angles of sixteen bits in the frame of its corner's fan. The
    // frame's own vectors are made unit vectors by division (the fan's sides and normal, handed to it by the
    // bevel, by the reciprocal): made by the reciprocal too, a normal half way round a fan (a corner patch's
    // on a profile's middle point) was stored a unit or two off. tests/golden/bevel_angles.txt: boxes in one to
    // four segments, one cut back for want of room, and cylinders: every pair Blender's, but on the eight-sided
    // cylinder in two segments, whose corner patches' points are a unit in the last place off Blender's and
    // eight of its angles a unit or two with them.
    let mut report = Vec::new();
    let mut bad = 0;
    for r in records("bevel_angles.txt") {
        scene::reset();
        let (bevel, bevel_segments) =
            (r.kv["bevel"].parse().unwrap(), r.kv["seg"].parse().unwrap());
        let part = if r.kv["kind"] == "cube" {
            let s: Vec<f64> = r.kv["size"]
                .split(',')
                .map(|x| x.parse().unwrap())
                .collect();
            cube(
                "part",
                (s[0], s[1], s[2]),
                Cube {
                    bevel,
                    bevel_segments,
                    ..Cube::default()
                },
            )
        } else {
            artkit::shapes::cylinder(
                "part",
                r.kv["radius"].parse().unwrap(),
                r.kv["depth"].parse().unwrap(),
                artkit::shapes::Round {
                    verts: r.kv["verts"].parse().unwrap(),
                    bevel,
                    bevel_segments,
                    ..Default::default()
                },
            )
        };
        let mesh = artkit::bevel::apply(part).read(|o| o.mesh.clone());
        let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
            .map(|f| mesh.corners(f).to_vec())
            .collect();
        assert_eq!(faces, r.faces("f"), "{}: faces", r.name);
        let want: Vec<[i16; 2]> = r.lines["c"]
            .split(' ')
            .map(|p| {
                let (a, b) = p.split_once(',').unwrap();
                [a.parse().unwrap(), b.parse().unwrap()]
            })
            .collect();
        let stored = mesh.stored.as_ref().expect("a bevel's stored normals");
        let off = stored
            .iter()
            .zip(&want)
            .filter(|(s, w)| **s != Some(**w))
            .count();
        let far = stored
            .iter()
            .zip(&want)
            .map(|(s, w)| {
                let s = s.unwrap_or([0, 0]);
                (s[0] as i32 - w[0] as i32)
                    .abs()
                    .max((s[1] as i32 - w[1] as i32).abs())
            })
            .max()
            .unwrap_or(0);
        report.push(format!(
            "{} ({} in {}): {off} of {} corners' angles not Blender's (the farthest {far} of 32,767)",
            r.name,
            r.kv["kind"],
            r.kv["seg"],
            want.len()
        ));
        bad += if r.name == "cyl8" {
            far > 2 || off > 8
        } else {
            off > 0
        } as usize;
    }
    println!("{}", report.join("\n"));
    assert_eq!(bad, 0, "\n{}", report.join("\n"));
}

#[test]
fn a_bevel_of_a_width_below_zero_is_none() {
    // Blender's modifier clamps a width below zero to none: the part is only shaded as a bevelled one is
    for r in records("bevel_none.txt") {
        scene::reset();
        let part = match r.name.as_str() {
            "cube" => cube(
                "p",
                (0.2, 0.3, 0.4),
                Cube {
                    bevel: -0.01,
                    bevel_segments: 1,
                    ..Cube::default()
                },
            ),
            "cylinder" => cylinder(
                "c",
                0.1,
                0.3,
                Round {
                    verts: 8,
                    bevel: -0.004,
                    ..Round::default()
                },
            ),
            _ => {
                let (w, h, c) = (0.2, 0.1, 0.01);
                let outline = [
                    (w, h - c),
                    (w - c, h),
                    (-w + c, h),
                    (-w, h - c),
                    (-w, -h + c),
                    (-w + c, -h),
                    (w - c, -h),
                    (w, -h + c),
                ];
                extrude(
                    "s",
                    &outline,
                    0.02,
                    Extrude {
                        bevel: -0.004,
                        bevel_segments: 1,
                        ..Extrude::default()
                    },
                )
            }
        };
        assert_eq!(artkit::parts::game_size(&[part]).len(), 3);
        let joined = artkit::parts::join(&[part], "j", None);
        let mesh = joined.read(|o| o.mesh.clone());
        let (want_points, want_faces) = (r.points("v"), r.faces("f"));
        assert_eq!(mesh.verts.len(), want_points.len(), "{}: points", r.name);
        assert!(
            mesh.normals.is_none() && mesh.stored.is_none(),
            "{}: no normals of its own",
            r.name
        );
        assert!(mesh.faces.iter().all(|f| f.smooth), "{}: smooth", r.name);
        let (turned, _, missing) = normals_off(&mesh, &want_points, &want_faces, &r.points("n"));
        assert!(
            turned < 0.01 && missing == 0,
            "{}: {turned} {missing}",
            r.name
        );
    }
}

#[test]
fn a_bevelled_boxs_faces_come_in_blenders_order() {
    // Blender makes a corner's faces vertex by vertex, then a strip an edge, then the box's own faces again in the
    // order its vertices first come to them, each vertex's faces as its edges are numbered. The baked shading's
    // rays meet the faces in this order, so it is Blender's: every face of the boxes of tests/golden/bevel.txt,
    // corner for corner.
    let mut seen = 0;
    for r in records("bevel.txt") {
        if r.kv["kind"] != "cube" || r.kv["changed"] != "0" {
            continue;
        }
        scene::reset();
        let s: Vec<f64> = r.kv["size"].split(',').map(f64w).collect();
        let part = cube(
            "box",
            (s[0], s[1], s[2]),
            Cube {
                bevel: r.f64("width"),
                bevel_segments: r.kv["seg"].parse().unwrap(),
                ..Cube::default()
            },
        );
        let mesh = artkit::bevel::apply(part).read(|o| o.mesh.clone());
        let got: Vec<Vec<u32>> = (0..mesh.faces.len())
            .map(|f| mesh.corners(f).to_vec())
            .collect();
        assert_eq!(got, r.faces("of"), "{}", r.name);
        seen += 1;
    }
    assert!(seen >= 5, "{seen} boxes");
}

#[test]
fn a_corner_patchs_texture_is_carried_as_blender_carries_it() {
    // a patch of three segments: each quad's corners read the face Blender reads them from, and those on the
    // patch's middle lines are read on the old edge Blender snaps them to
    let mut report = Vec::new();
    for r in records("bevel_patch_uv.txt") {
        scene::reset();
        scene::set_finishes(finishes());
        let s: Vec<f64> = r.kv["size"]
            .split(',')
            .map(|x| x.parse().unwrap())
            .collect();
        let deck = cube(
            "deck",
            (s[0], s[1], s[2]),
            Cube {
                at: (0.1, -0.2, 0.3).into(),
                material: Some(Mat::new("wood", "#8A5A2B", "wood")),
                bevel: r.kv["width"].parse().unwrap(),
                bevel_segments: r.kv["seg"].parse().unwrap(),
                ..Cube::default()
            },
        );
        let joined = artkit::parts::join(&[deck], "j", None);
        assert!(artkit::uv::ensure(&[joined]));
        let mesh = joined.read(|o| o.mesh.clone());
        let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
            .map(|f| mesh.corners(f).to_vec())
            .collect();
        assert_eq!(faces, r.faces("f"), "{}: faces", r.name);
        let (uvs, want) = (mesh.uvs.as_ref().unwrap(), r.floats("uv"));
        let (mut far, mut off, mut area_off, mut area_all) = (0.0_f32, 0, 0.0_f64, 0.0_f64);
        for f in 0..mesh.faces.len() {
            let face = mesh.faces[f];
            let p: Vec<V3> = mesh
                .corners(f)
                .iter()
                .map(|&v| mesh.verts[v as usize])
                .collect();
            let area: f64 = (1..p.len() - 1)
                .map(|i| (p[i] - p[0]).cross(p[i + 1] - p[0]).length() / 2.0)
                .sum();
            let mut worst = 0.0_f32;
            for l in face.start as usize..(face.start + face.len) as usize {
                for k in 0..2 {
                    worst = worst.max((uvs[l][k] - want[2 * l + k]).abs());
                }
            }
            far = far.max(worst);
            area_all += area;
            if worst > 1e-5 {
                off += 1;
                area_off += area;
            }
        }
        report.push(format!(
            "{}: {off} of {} faces with a corner's UV over a hundredth of a millimetre off ({:.2} % of its surface), the farthest {far:e} m",
            r.name,
            mesh.faces.len(),
            100.0 * area_off / area_all
        ));
        assert!(off == 0, "{}", report.last().unwrap());
    }
    println!("{}", report.join("\n"));
}
