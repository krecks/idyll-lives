//! The kit's cuts (`kit::cut`, `kit::recessed_box`) against Blender 5.2's exact boolean (tests/golden/kit_cuts.txt):
//! what the family scripts' cuts made there, one of each kind. `forms.cut` ("G.cut": the chef's stove's oven mouth,
//! cut before the carcass's bevel; the slob's microwave, a lofted case; half of the hot tub's cover; the toy box's
//! two grips), kitchen_cooking's `_hollow` (an oven's, the microwave's and the dishwasher's cavity) and the two
//! cuts kitchen.py's `_soft` applies (the retro fridge's lofted shell; the mini fridge's bevelled cabinet, by a
//! bevelled cutter).
//!
//! Each record has the body as it went into the cut (body_in) and the cutter, in their own frames at their places
//! (loc: place, turn, scale), the mesh the cut left (out), and the part as the export sees it with its shading
//! baked alone (final: ao, a level of 32 a point); mods: the body's modifiers when the cut was made.
//!
//! The cut is held to Blender's: the same points (as a set), every face with Blender's corners round it (from
//! whichever it starts at), its material, its smooth flag and the sharp edges, so the same area on every plane of
//! every material and the same triangles' count; Blender's normals at the corners of the flat faces; and its
//! shading within `LEVELS_OFF` at every point (the bake turns its rays by a point's number, and the points the cut
//! makes are numbered the kernel's way).
mod common;

use artkit::bevel::{self, EdgeOrder};
use artkit::kit::{self, Cut, RecessedBox};
use artkit::materials::Mat;
use artkit::math::V3;
use artkit::mesh::{Mesh, edge};
use artkit::place::GameFrame;
use artkit::scene::{self, Obj};
use common::*;
use std::collections::BTreeMap;

/// A record: its heading, and its meshes by name.
struct Record {
    what: String,
    mods: String,
    meshes: BTreeMap<String, GoldenMesh>,
}

fn records() -> Vec<Record> {
    let mut out: Vec<Record> = Vec::new();
    for g in golden_meshes("kit_cuts.txt") {
        if out.last().is_none_or(|r| r.what != g.what) {
            out.push(Record {
                what: g.what.clone(),
                mods: g.lines.get("mods").cloned().unwrap_or_default(),
                meshes: BTreeMap::new(),
            });
        }
        out.last_mut().unwrap().meshes.insert(g.name.clone(), g);
    }
    out
}

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// A golden mesh's materials ("name:finish" each).
fn mats(g: &GoldenMesh) -> Vec<Mat> {
    g.lines["mats"]
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let (name, finish) = w.split_once(':').unwrap();
            Mat::new(leak(name), "#808080", leak(finish))
        })
        .collect()
}

fn slots(g: &GoldenMesh) -> Vec<u16> {
    g.lines["m"]
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(|w| w.parse().unwrap())
        .collect()
}

/// A golden mesh as a part at its place: its faces' materials and shading, its sharp edges, its corners' normals.
fn part(name: &str, g: &GoldenMesh, own_normals: bool) -> Obj {
    let verts: Vec<V3> = g.verts.iter().map(|p| V3::new(p[0], p[1], p[2])).collect();
    let mut mesh = Mesh::from_pydata(verts, &g.faces);
    mesh.mats = mats(g);
    let (m, s) = (slots(g), g.lines["s"].as_bytes());
    for (f, face) in mesh.faces.iter_mut().enumerate() {
        (face.mat, face.smooth) = (m[f], s[f] == b'1');
    }
    for e in g.lines["e"].split(' ').filter(|w| !w.is_empty()) {
        let (a, b) = e.split_once('-').unwrap();
        mesh.sharp
            .insert(edge(a.parse().unwrap(), b.parse().unwrap()));
    }
    if own_normals {
        mesh.normals = Some(
            g.f32s("cn")
                .chunks(3)
                .map(|c| V3::new(c[0], c[1], c[2]))
                .collect(),
        );
    }
    let obj = scene::add(name, mesh);
    let loc = g.f32s("loc");
    obj.set_location((loc[0] as f64, loc[1] as f64, loc[2] as f64));
    obj
}

/// A surface as its area on each plane of each material: (material, the plane's normal to a thousandth and its
/// distance to a tenth of a millimetre) to square metres.
fn surface(
    verts: &[[f32; 3]],
    faces: &[Vec<u32>],
    mat: &dyn Fn(usize) -> String,
) -> BTreeMap<(String, [i64; 4]), f64> {
    let mut out = BTreeMap::new();
    for (f, c) in faces.iter().enumerate() {
        let p: Vec<[f64; 3]> = c
            .iter()
            .map(|&v| verts[v as usize].map(|x| x as f64))
            .collect();
        let mut n = [0.0_f64; 3];
        for i in 0..p.len() {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            n[0] += (a[1] - b[1]) * (a[2] + b[2]);
            n[1] += (a[2] - b[2]) * (a[0] + b[0]);
            n[2] += (a[0] - b[0]) * (a[1] + b[1]);
        }
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len == 0.0 {
            continue;
        }
        let unit = n.map(|x| x / len);
        let d: f64 = p
            .iter()
            .map(|q| (q[0] * unit[0] + q[1] * unit[1] + q[2] * unit[2]) / p.len() as f64)
            .sum();
        let key = [
            (unit[0] * 1e3).round() as i64,
            (unit[1] * 1e3).round() as i64,
            (unit[2] * 1e3).round() as i64,
            (d * 1e4).round() as i64,
        ];
        *out.entry((mat(f), key)).or_insert(0.0) += len / 2.0;
    }
    out
}

/// The plane a face lies on, as `surface` names it.
fn plane(verts: &[[f32; 3]], c: &[u32]) -> [i64; 4] {
    let faces = [c.to_vec()];
    let s = surface(verts, &faces, &|_| String::new());
    s.keys().next().map_or([0; 4], |k| k.1)
}

struct Held {
    points: usize,
    inexact: usize,
    far: f32,
    triangles: (usize, usize),
    planes: usize,
    area_off: f64,
    normals: usize,
    normals_off: f64,
    /// Blender's faces that are the kernel's too (the same corners in the same order round), and how many it has.
    faces: (usize, usize),
    /// Of those, the ones that start at the corner Blender's does.
    starts: usize,
    /// Sharp edges: Blender's the kernel lacks, the kernel's Blender lacks.
    sharp: (usize, usize),
    /// Faces of both whose smooth flag differs.
    smooth: usize,
    /// Corners of the faces of both whose normal is more than a degree off Blender's, and the most any is.
    turned: (usize, f64),
}

/// A built mesh against Blender's: the same points as a set (none farther than `reach`), the same area on every
/// plane of every material (within a square millimetre), the triangles a material within the comparison's rule
/// (a fifth fewer to a tenth more), and Blender's normals at the corners of its flat faces. Returns the points'
/// pairing too.
fn held(what: &str, mesh: &Mesh, g: &GoldenMesh, reach: f32) -> (Held, Vec<u32>) {
    assert_eq!(mesh.verts.len(), g.verts.len(), "{what}: how many points");
    let (map, inexact, far) =
        pair_points(&mesh.verts, &g.verts, reach).unwrap_or_else(|| panic!("{what}: points apart"));
    let verts: Vec<[f32; 3]> = mesh.verts.iter().map(|v| [v.x, v.y, v.z]).collect();
    let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect();
    let (gm, gs) = (mats(g), slots(g));
    let ours = surface(&verts, &faces, &|f| {
        mesh.mats[mesh.faces[f].mat as usize].name.to_string()
    });
    let theirs = surface(&g.verts, &g.faces, &|f| gm[gs[f] as usize].name.to_string());
    let mut area_off = 0.0_f64;
    for key in ours.keys().chain(theirs.keys()) {
        let (a, b) = (
            ours.get(key).copied().unwrap_or(0.0),
            theirs.get(key).copied().unwrap_or(0.0),
        );
        area_off = area_off.max((a - b).abs());
        assert!(
            (a - b).abs() < 1e-6,
            "{what}: {} m2 on {key:?}, Blender's {} m2",
            a,
            b
        );
    }
    // the triangles a material
    let mut count: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for (f, face) in mesh.faces.iter().enumerate() {
        count
            .entry(mesh.mats[face.mat as usize].name.to_string())
            .or_default()
            .0 += faces[f].len() - 2;
    }
    for (f, c) in g.faces.iter().enumerate() {
        count
            .entry(gm[gs[f] as usize].name.to_string())
            .or_default()
            .1 += c.len() - 2;
    }
    let mut triangles = (0, 0);
    for (name, &(a, b)) in &count {
        assert!(
            a as f64 >= b as f64 * 0.8 && a as f64 <= b as f64 * 1.1,
            "{what}: {a} triangles of {name}, Blender's {b}"
        );
        triangles = (triangles.0 + a, triangles.1 + b);
    }
    assert_eq!(triangles.1, g.tris.len(), "{what}: Blender's triangles");
    // Blender's normals at the corners of its flat faces, by point and plane
    let cn: Vec<[f32; 3]> = g.f32s("cn").chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
    let mut flat: BTreeMap<(u32, [i64; 4]), [f32; 3]> = BTreeMap::new();
    let mut at = 0;
    for c in &g.faces {
        let own = &cn[at..at + c.len()];
        at += c.len();
        let key = plane(&g.verts, c);
        let unit = [
            key[0] as f64 / 1e3,
            key[1] as f64 / 1e3,
            key[2] as f64 / 1e3,
        ];
        let along =
            |n: &[f32; 3]| n[0] as f64 * unit[0] + n[1] as f64 * unit[1] + n[2] as f64 * unit[2];
        if own.iter().all(|n| along(n) > 0.999_99) {
            for (&v, n) in c.iter().zip(own) {
                flat.insert((v, key), *n);
            }
        }
    }
    let built = artkit::normals::corner_normals(mesh);
    let (mut normals, mut normals_off) = (0, 0.0_f64);
    for (f, face) in mesh.faces.iter().enumerate() {
        let key = plane(&verts, &faces[f]);
        for (i, &v) in faces[f].iter().enumerate() {
            let Some(want) = flat.get(&(map[v as usize], key)) else {
                continue;
            };
            let got = built[face.start as usize + i];
            let dot = got.x as f64 * want[0] as f64
                + got.y as f64 * want[1] as f64
                + got.z as f64 * want[2] as f64;
            let off = dot.clamp(-1.0, 1.0).acos().to_degrees();
            normals += 1;
            normals_off = normals_off.max(off);
        }
    }
    assert!(normals > 0, "{what}: no flat face's corner found");
    assert!(
        normals_off < 0.05,
        "{what}: a flat face's normal {normals_off} degrees off Blender's"
    );
    // Blender's faces, sharp edges and smooth flags, by its points
    let named = renamed(&faces, &map);
    let key = |f: &[u32]| as_set(&[f.to_vec()]).remove(0);
    let theirs_smooth: BTreeMap<Vec<u32>, bool> = g
        .faces
        .iter()
        .zip(g.lines["s"].bytes())
        .map(|(f, s)| (key(f), s == b'1'))
        .collect();
    // every corner's normal against Blender's at that point of that face
    let mut theirs_normal: BTreeMap<(Vec<u32>, u32), [f32; 3]> = BTreeMap::new();
    let mut at = 0;
    for c in &g.faces {
        for (i, &v) in c.iter().enumerate() {
            theirs_normal.insert((key(c), v), cn[at + i]);
        }
        at += c.len();
    }
    let mut turned = (0, 0.0_f64);
    for (f, face) in mesh.faces.iter().enumerate() {
        for (i, &v) in named[f].iter().enumerate() {
            let Some(want) = theirs_normal.get(&(key(&named[f]), v)) else {
                continue;
            };
            let got = built[face.start as usize + i];
            let dot = got.x as f64 * want[0] as f64
                + got.y as f64 * want[1] as f64
                + got.z as f64 * want[2] as f64;
            let off = dot.clamp(-1.0, 1.0).acos().to_degrees();
            turned = (turned.0 + (off > 1.0) as usize, turned.1.max(off));
            if off > 1.0 && std::env::var("CUTS_TURNED").is_ok() {
                println!(
                    "  turned {off:.1} at {:?} on a face of {} corners of {} (smooth {}): {:?}, Blender's {:?}",
                    mesh.verts[faces[f][i] as usize],
                    faces[f].len(),
                    mesh.mats[face.mat as usize].name,
                    face.smooth,
                    got,
                    want
                );
            }
        }
    }
    let as_they_start: std::collections::BTreeSet<&Vec<u32>> = g.faces.iter().collect();
    let (mut shared, mut smooth, mut starts) = (0, 0, 0);
    for (f, face) in mesh.faces.iter().enumerate() {
        if let Some(&s) = theirs_smooth.get(&key(&named[f])) {
            shared += 1;
            smooth += (s != face.smooth) as usize;
            starts += as_they_start.contains(&named[f]) as usize;
        }
    }
    let their_sharp: std::collections::BTreeSet<(u32, u32)> = g.lines["e"]
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(|e| {
            let (a, b) = e.split_once('-').unwrap();
            edge(a.parse().unwrap(), b.parse().unwrap())
        })
        .collect();
    let our_sharp: std::collections::BTreeSet<(u32, u32)> = mesh
        .sharp
        .iter()
        .map(|&(a, b)| edge(map[a as usize], map[b as usize]))
        .collect();
    let sharp = (
        their_sharp.difference(&our_sharp).count(),
        our_sharp.difference(&their_sharp).count(),
    );
    (
        Held {
            turned,
            faces: (shared, g.faces.len()),
            starts,
            sharp,
            smooth,
            points: mesh.verts.len(),
            inexact,
            far,
            triangles,
            planes: theirs.len(),
            area_off,
            normals,
            normals_off,
        },
        map,
    )
}

/// The part's shading, baked alone, against Blender's: (how many points differ, the most levels of 32 apart).
fn shading(what: &str, obj: Obj, g: &GoldenMesh, reach: f32) -> (usize, i64, f64) {
    artkit::shade::bake_ao(&[obj], None, false);
    let (verts, ao) = obj.read(|o| (o.mesh.verts.clone(), o.mesh.ao.clone().unwrap()));
    let (map, _, _) = pair_points(&verts, &g.verts, reach)
        .unwrap_or_else(|| panic!("{what}: final points apart"));
    let want: Vec<i64> = g.lines["ao"]
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(|w| w.parse().unwrap())
        .collect();
    let (mut differ, mut most, mut sum) = (0, 0, 0);
    for (v, &a) in ao.iter().enumerate() {
        let off = ((a * 32.0).round() as i64 - want[map[v] as usize]).abs();
        differ += (off > 0) as usize;
        most = most.max(off);
        sum += off;
    }
    (differ, most, sum as f64 / ao.len() as f64)
}

/// A bevel modifier of a record's mods: (width, segments, angle in degrees).
fn bevel_of(mods: &str) -> Option<(f64, usize, f64)> {
    let w = mods.split(' ').find(|w| w.starts_with("bevel:"))?;
    let p: Vec<&str> = w.split(':').collect();
    Some((f64w(p[1]), p[2].parse().unwrap(), f64w(p[3])))
}

/// How far a point the cut makes may be from Blender's, metres: where an edge of a lofted shell crosses a side of
/// the opening, the crossing is worked out in doubles and rounded, Blender's in whole fractions and rounded.
const REACH: f32 = 2e-7;

/// How many levels of 32 a point's baked shading may be from Blender's, and all points on average.
const LEVELS_OFF: i64 = 4;
const LEVELS_MEAN: f64 = 0.25;

#[test]
fn cuts_are_blenders() {
    let all = records();
    assert_eq!(all.len(), 10);
    for r in &all {
        scene::reset();
        let (kind, _) = r.what.split_once(' ').unwrap();
        // (a bevel made before the cut left the body normals of its own, and the cutter its own likewise)
        let rounded = kind != "G.cut" && bevel_of(&r.mods).is_some();
        let body = part("body", &r.meshes["body_in"], rounded);
        // (forms.cut cuts the body before its bevel, the others after: body_in is the mesh either way)
        let pending = if kind == "G.cut" {
            bevel_of(&r.mods)
        } else {
            None
        };
        if let Some((width, segments, angle)) = pending {
            bevel::bevel_edges(body, width, segments, angle, &EdgeOrder::Corners);
        }
        let cutter = part(
            "cutter",
            &r.meshes["cutter"],
            r.meshes["cutter"].verts.len() > 8 && rounded,
        );
        kit::cut(
            body,
            cutter,
            Cut {
                lined: kind != "G.cut",
                ..Cut::default()
            },
        );
        assert!(!cutter.is_alive(), "{}: the cutter is gone", r.what);
        let mesh = body.read(|o| o.mesh.clone());
        let (h, _) = held(&r.what, &mesh, &r.meshes["out"], REACH);
        assert_eq!(
            (h.faces.0, h.faces.1),
            (mesh.faces.len(), mesh.faces.len()),
            "{}: Blender's faces",
            r.what
        );
        assert_eq!(
            (h.sharp, h.smooth),
            ((0, 0), 0),
            "{}: sharp edges, smooth flags",
            r.what
        );
        // as the export sees it: its bevel made, its shading baked alone
        bevel::apply(body);
        let last = &r.meshes["final"];
        let shade = if body.read(|o| o.mesh.verts.len()) == last.verts.len() {
            let (differ, most, mean) = shading(&r.what, body, last, 2e-6);
            assert!(
                most <= LEVELS_OFF,
                "{}: shading {most} levels off Blender's",
                r.what
            );
            assert!(
                mean <= LEVELS_MEAN,
                "{}: shading {mean} levels off Blender's on average",
                r.what
            );
            format!("shading: {differ} points off, {most} levels at most, {mean:.3} on average")
        } else {
            format!(
                "after its bevel {} points, Blender's {}",
                body.read(|o| o.mesh.verts.len()),
                last.verts.len()
            )
        };
        println!(
            "{}: {} points ({} not to the bit, the farthest {:e} m), {} triangles (Blender's {}), {} planes (the most off {:e} m2), {} flat corners' normals (the most off {:.4} degrees), {} of Blender's {} faces ({} from its corner), sharp edges {:?}, smooth flags off {}, corners turned {:?}; {shade}",
            r.what,
            h.points,
            h.inexact,
            h.far,
            h.triangles.0,
            h.triangles.1,
            h.planes,
            h.area_off,
            h.normals,
            h.normals_off,
            h.faces.0,
            h.faces.1,
            h.starts,
            h.sharp,
            h.smooth,
            h.turned
        );
    }
}

/// `recessed_box` from numbers: the electric stove's carcass and oven cavity as the script's two boxes.
#[test]
fn a_recessed_box_is_the_ovens() {
    let all = records();
    let r = all
        .iter()
        .find(|r| r.what.contains("stoveElectric"))
        .unwrap();
    scene::reset();
    let (body, cutter) = (&r.meshes["body_in"], &r.meshes["cutter"]);
    let size = |g: &GoldenMesh| -> [f64; 3] {
        let mut hi = [f32::NEG_INFINITY; 3];
        for p in &g.verts {
            for k in 0..3 {
                hi[k] = hi[k].max(p[k]);
            }
        }
        hi.map(|x| x as f64 * 2.0)
    };
    // (Blender's x, y, z are the game's -x, -depth and height)
    let game = |g: &GoldenMesh| -> (f64, f64, f64) {
        let loc = g.f32s("loc");
        (-loc[0] as f64, loc[2] as f64, -loc[1] as f64)
    };
    let (b, c) = (size(body), size(cutter));
    let (width, segments, _) = bevel_of(&r.mods).unwrap();
    let made = kit::recessed_box(
        &GameFrame::new(0.0),
        "carcass",
        b[0],
        b[2],
        b[1],
        game(body),
        mats(body)[0],
        (c[0], c[2], c[1]),
        game(cutter),
        RecessedBox {
            bevel: width,
            bevel_segments: segments,
            lining: Some(mats(cutter)[0]),
            ..RecessedBox::default()
        },
    );
    let mesh = made.read(|o| o.mesh.clone());
    let (h, _) = held("recessed_box", &mesh, &r.meshes["out"], REACH);
    println!(
        "recessed_box: {} points ({} not to the bit, the farthest {:e} m), {} triangles (Blender's {}), {} of Blender's {} faces, corners turned {:?}",
        h.points, h.inexact, h.far, h.triangles.0, h.triangles.1, h.faces.0, h.faces.1, h.turned
    );
    assert_eq!((h.sharp, h.smooth, h.turned.0), ((0, 0), 0, 0));
    assert_eq!(mesh.mats.len(), 2);
}

/// The slob's microwave as its script builds it: the case a loft through its four rings, the cutter a box, and
/// nothing of Blender's but the numbers: the cut's normals at every corner are Blender's too.
#[test]
fn a_lofted_case_is_cut_as_the_script_cuts_it() {
    let all = records();
    let r = all
        .iter()
        .find(|r| r.what.contains("microwaveCart"))
        .unwrap();
    scene::reset();
    let (body, cutter) = (&r.meshes["body_in"], &r.meshes["cutter"]);
    let sections: Vec<Vec<artkit::math::P3>> = body
        .verts
        .chunks(24)
        .map(|ring| {
            ring.iter()
                .map(|p| (p[0] as f64, p[1] as f64, p[2] as f64).into())
                .collect()
        })
        .collect();
    let case = artkit::loft!("mw_case", &sections, material = mats(body)[0]);
    let mut hi = [f32::NEG_INFINITY; 3];
    for p in &cutter.verts {
        for k in 0..3 {
            hi[k] = hi[k].max(p[k]);
        }
    }
    let loc = cutter.f32s("loc");
    let tool = artkit::cube!(
        "mw_cut",
        (hi[0] as f64 * 2.0, hi[1] as f64 * 2.0, hi[2] as f64 * 2.0),
        at = (loc[0] as f64, loc[1] as f64, loc[2] as f64),
        material = mats(cutter)[0],
        bevel = 0
    );
    kit::cut!(case, tool);
    let mesh = case.read(|o| o.mesh.clone());
    let (h, _) = held("the slob's microwave", &mesh, &r.meshes["out"], REACH);
    println!(
        "the slob's microwave, built here: {} points ({} not to the bit), {} of Blender's {} faces, sharp edges {:?}, smooth flags off {}, corners turned {:?}",
        h.points, h.inexact, h.faces.0, h.faces.1, h.sharp, h.smooth, h.turned
    );
    assert_eq!((h.sharp, h.smooth, h.turned.0), ((0, 0), 0, 0));
}

/// The retro mini fridge's cabinet as its script builds it: a box rounded in four strips, cut by a box rounded in
/// two, each bevel made first.
#[test]
fn a_rounded_cabinet_is_cut_by_a_rounded_box() {
    let all = records();
    let r = all
        .iter()
        .find(|r| r.what.contains("fridgeMiniRetro"))
        .unwrap();
    scene::reset();
    let (body, cutter) = (&r.meshes["body_in"], &r.meshes["cutter"]);
    let size = |g: &GoldenMesh| -> (f64, f64, f64) {
        let mut hi = [f32::NEG_INFINITY; 3];
        for p in &g.verts {
            for k in 0..3 {
                hi[k] = hi[k].max(p[k]);
            }
        }
        (hi[0] as f64 * 2.0, hi[1] as f64 * 2.0, hi[2] as f64 * 2.0)
    };
    let at = |g: &GoldenMesh| -> (f64, f64, f64) {
        let loc = g.f32s("loc");
        (loc[0] as f64, loc[1] as f64, loc[2] as f64)
    };
    let (width, segments, _) = bevel_of(&r.mods).unwrap();
    let cabinet = artkit::cube!(
        "cabinet",
        size(body),
        at = at(body),
        material = mats(body)[0],
        bevel = width,
        bevel_segments = segments
    );
    bevel::apply(cabinet);
    let tool = artkit::cube!(
        "cavity",
        size(cutter),
        at = at(cutter),
        material = mats(cutter)[0],
        bevel = 0.015
    );
    kit::cut!(cabinet, tool, lined = true);
    let mesh = cabinet.read(|o| o.mesh.clone());
    let (h, _) = held("the mini fridge's cabinet", &mesh, &r.meshes["out"], REACH);
    println!(
        "the mini fridge's cabinet, built here: {} points ({} not to the bit, the farthest {:e} m), {} of Blender's {} faces, sharp edges {:?}, smooth flags off {}, corners turned {:?}",
        h.points, h.inexact, h.far, h.faces.0, h.faces.1, h.sharp, h.smooth, h.turned
    );
    assert_eq!((h.sharp, h.smooth), ((0, 0), 0));
    assert!(
        h.turned.1 < 12.0,
        "the cut's own corners, before they are weighed"
    );
    // and as the script's _soft leaves it: smooth by 40 degrees, the corners' normals weighed by their faces
    // (the edges are marked by the corners' normals as the cut left them, what the bevels stored read on the cut
    // mesh; the weighted normals are then the corners' own, and nothing stored reads over them)
    cabinet.edit(|o| {
        o.mesh.shade_smooth_by_angle(40.0);
        o.mesh.stored = None;
    });
    artkit::weighted_normal!(cabinet, keep_sharp = true);
    let mesh = cabinet.read(|o| o.mesh.clone());
    let mut soft = r.meshes["soft"].clone();
    soft.tris = r.meshes["final"].tris.clone();
    for key in ["m", "mats"] {
        soft.lines
            .insert(key.into(), r.meshes["final"].lines[key].clone());
    }
    soft.lines
        .insert("cn".into(), r.meshes["final"].lines["cn"].clone());
    let (h, _) = held("the mini fridge's cabinet, soft", &mesh, &soft, REACH);
    println!(
        "the mini fridge's cabinet, soft: sharp edges {:?}, smooth flags off {}, corners turned {:?}",
        h.sharp, h.smooth, h.turned
    );
    assert_eq!((h.sharp, h.smooth, h.turned.0), ((0, 0), 0, 0));
}
