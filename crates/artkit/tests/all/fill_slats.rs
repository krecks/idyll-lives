//! The fire engine's roller shutters (civic_works' `slats`: a thin sawtooth outline, a step out at each slat's foot,
//! extruded along the body) against Blender 5.2's (tests/golden/fill_slats.txt: the shutters of 27 slats and of
//! 22, on the left and on the right; for each, Blender's own `triangle_fill` of the outline, and the slab gamekit's
//! `forms.extrude` made of it: its points, its polygons from the corner each starts at, its triangles, its corners'
//! normals and the levels `shade.bake_ao` gave it alone).
//!
//! The feet of the slats lie in one line, to the bit. Filling the left-hand outline Blender cuts it into as many
//! triangles as it needs; filling the right-hand one, its mirror image, the sweep joins three feet in a line
//! here and there: faces of no area, and Blender's extrusion puts a wall inside the slab under each one's long
//! side. `tri::fill` is Blender's on both sides, those faces too; `forms::extrude` builds the right-hand slab
//! as the left-hand one's mirror image.

use crate::common::*;
use artkit::fextrude;
use artkit::forms::dedupe;
use artkit::materials::Mat;
use artkit::mesh::{Mesh, edge};
use artkit::normals::corner_normals;
use artkit::place::{Facing, GameFrame};
use artkit::scene::{self, Obj};
use artkit::{py, shade, tri};
use std::collections::BTreeMap;

const F: GameFrame = GameFrame::new(0.0);
const SHUTTER: Mat = Mat::new("shutter", "#C3C7CA", "metal");
/// The body's half width and the shutters' top.
const HALF: f64 = 1.15;
const TOP: f64 = 2.72;
/// A slat's height at most, and how far it stands proud.
const PITCH: f64 = 0.07;
const PROUD: f64 = 0.008;

/// The locker a golden mesh is the shutter of: its side (+1 the right), its depths and its foot's height.
fn locker(g: &GoldenMesh) -> (f64, f64, f64, f64) {
    let s: f64 = g.lines["side"].parse().unwrap();
    match g.name.trim_start_matches(|c: char| c.is_alphabetic()) {
        "27" => (s, -0.02, -0.96, 0.84),
        "22" => (s, -1.04, -2.16, 1.2),
        other => panic!("no locker of {other} slats"),
    }
}

/// The shutter's outline as `slats` draws it (game x, height): each slat a step out at its foot, then sloping
/// back in to the next.
fn outline(s: f64, y0: f64) -> Vec<(f64, f64)> {
    let x = s * HALF;
    let n = py::round((TOP - y0) / PITCH).max(1);
    let p = (TOP - y0) / n as f64;
    let mut out = vec![(x, y0)];
    for i in 0..n {
        let ya = y0 + i as f64 * p;
        out.extend([(x + s * PROUD * 0.4, ya), (x + s * PROUD, ya + p * 0.12)]);
    }
    out.extend([(x + s * PROUD * 0.4, TOP), (x, TOP)]);
    out
}

/// The shutter as civic_works builds it.
fn shutter(g: &GoldenMesh) -> Obj {
    let (s, z0, z1, y0) = locker(g);
    fextrude!(
        F,
        &g.name,
        &outline(s, y0),
        (z1 - z0).abs(),
        (0, 0, (z0 + z1) / 2.0),
        SHUTTER,
        plane = Facing::Front
    )
}

fn mesh_of(obj: Obj) -> Mesh {
    obj.read(|o| o.mesh.clone())
}

/// A mesh's faces, each its corners' points.
fn faces(mesh: &Mesh) -> Vec<Vec<u32>> {
    (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect()
}

/// The levels the bake gives `obj` alone, as the golden writes them.
fn levels(obj: Obj) -> Vec<i64> {
    shade::bake_ao(&[obj], None, false);
    obj.read(|o| o.mesh.ao.clone().unwrap())
        .iter()
        .map(|a| (a * 32.0).round() as i64)
        .collect()
}

fn golden_levels(g: &GoldenMesh) -> Vec<i64> {
    g.lines["ao"]
        .split(' ')
        .map(|l| l.parse().unwrap())
        .collect()
}

/// Whether the triangle `t` of the points `p` has no area: its points in one line.
fn flat(p: &[[f32; 3]], t: &[u32]) -> bool {
    let [a, b, c] = [t[0], t[1], t[2]].map(|v| p[v as usize].map(f64::from));
    let (u, v) = (
        [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
        [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
    );
    u[1] * v[2] == u[2] * v[1] && u[2] * v[0] == u[0] * v[2] && u[0] * v[1] == u[1] * v[0]
}

/// How many faces each edge of `faces` is a side of.
fn sides(faces: &[Vec<u32>]) -> BTreeMap<(u32, u32), usize> {
    let mut on = BTreeMap::new();
    for f in faces {
        for i in 0..f.len() {
            *on.entry(edge(f[i], f[(i + 1) % f.len()])).or_insert(0) += 1;
        }
    }
    on
}

#[test]
fn the_fill_is_blenders_on_both_sides() {
    // the triangles in Blender's order, each from the corner Blender's starts at; on the right the faces of no
    // area with them, each once (the sweep closes the same three feet twice, and Blender's mesh takes a face on
    // three points once)
    let mut flat_faces = Vec::new();
    for g in golden_meshes("fill_slats.txt") {
        let (s, _, _, y0) = locker(&g);
        // (as the frame turns a game outline drawn facing the front: x the other way)
        let turned: Vec<(f64, f64)> = outline(s, y0).iter().map(|p| (-p.0, p.1)).collect();
        let points: Vec<[f32; 2]> = dedupe(&turned, 1e-7)
            .iter()
            .map(|p| [p.0 as f32, p.1 as f32])
            .collect();
        let want: Vec<Vec<u32>> = g.lines["fill"]
            .split(' ')
            .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
            .collect();
        let got = tri::fill(std::slice::from_ref(&points));
        assert_eq!(faces_of(&got), want, "{}: the fill", g.name);
        let space: Vec<[f32; 3]> = points.iter().map(|p| [p[0], p[1], 0.0]).collect();
        flat_faces.push((
            g.name.clone(),
            got.len(),
            got.iter().filter(|t| flat(&space, &t[..])).count(),
        ));
    }
    let table: Vec<(&str, usize, usize)> = flat_faces
        .iter()
        .map(|(name, all, flat)| (name.as_str(), *all, *flat))
        .collect();
    assert_eq!(
        table,
        [
            ("left27", 55, 0),
            ("right27", 58, 3),
            ("left22", 45, 0),
            ("right22", 47, 2)
        ]
    );
}

fn faces_of(tris: &[[u32; 3]]) -> Vec<Vec<u32>> {
    tris.iter().map(|t| t.to_vec()).collect()
}

#[test]
fn the_left_hand_shutter_is_blenders() {
    let mut seen = 0;
    for g in golden_meshes("fill_slats.txt")
        .iter()
        .filter(|g| g.lines["side"] == "-1")
    {
        scene::reset();
        let obj = shutter(g);
        let mesh = mesh_of(obj);
        assert_eq!(
            bits(&obj.read(|o| o.loc).array()),
            bits(&g.f32s("at")),
            "{}: where it is",
            g.name
        );
        assert_eq!(mesh.verts.len(), g.verts.len(), "{}: points", g.name);
        for (i, v) in mesh.verts.iter().enumerate() {
            assert_eq!(bits(&v.array()), bits(&g.verts[i]), "{}: point {i}", g.name);
        }
        // Blender's polygons, each from Blender's corner (its extrusion leaves the far face's and the walls among
        // the near face's; here the two faces come first, then the walls), with Blender's normals at its corners
        assert_eq!(mesh.faces.len(), g.faces.len(), "{}: polygons", g.name);
        let starts: Vec<usize> = g
            .faces
            .iter()
            .scan(0, |at, f| {
                let start = *at;
                *at += f.len();
                Some(start)
            })
            .collect();
        let (normals, want) = (corner_normals(&mesh), g.f32s("cn"));
        let mut taken = vec![false; g.faces.len()];
        for f in 0..mesh.faces.len() {
            let b = g
                .faces
                .iter()
                .position(|b| b.as_slice() == mesh.corners(f))
                .unwrap_or_else(|| panic!("{}: face {f} isn't one of Blender's", g.name));
            assert!(
                !std::mem::replace(&mut taken[b], true),
                "{}: face {f} twice",
                g.name
            );
            let start = mesh.faces[f].start as usize;
            for c in 0..mesh.corners(f).len() {
                assert_eq!(
                    bits(&normals[start + c].array()),
                    bits(&want[3 * (starts[b] + c)..3 * (starts[b] + c) + 3]),
                    "{}: face {f}'s normal at corner {c}",
                    g.name
                );
            }
        }
        // its triangles: every face is one, or a wall cut as Blender cuts it
        assert_eq!(
            sorted(&mesh.triangle_verts()),
            sorted(&g.tris),
            "{}: triangles",
            g.name
        );
        assert_eq!(levels(obj), golden_levels(g), "{}: baked levels", g.name);
        seen += 1;
    }
    assert_eq!(seen, 2);
}

#[test]
fn the_right_hand_shutter_is_the_left_hand_ones_mirror_image() {
    let golden = golden_meshes("fill_slats.txt");
    let mut seen = 0;
    for g in golden.iter().filter(|g| g.lines["side"] == "1") {
        let other = golden
            .iter()
            .find(|l| l.name == g.name.replace("right", "left"))
            .unwrap();
        scene::reset();
        let (right, left) = (shutter(g), shutter(other));
        // the left-hand one, x the other way and its faces wound the other way with it
        let mut mirror = mesh_of(left);
        for v in &mut mirror.verts {
            v.x = -v.x;
        }
        mirror.flip_normals();
        let mesh = mesh_of(right);
        assert_eq!(mesh.verts.len(), mirror.verts.len(), "{}: points", g.name);
        for (i, v) in mesh.verts.iter().enumerate() {
            assert_eq!(
                bits(&v.array()),
                bits(&mirror.verts[i].array()),
                "{}: point {i}",
                g.name
            );
        }
        assert_eq!(faces(&mesh), faces(&mirror), "{}: polygons", g.name);
        assert_eq!(
            mesh.triangle_verts(),
            mirror.triangle_verts(),
            "{}: triangles",
            g.name
        );
        let normals = |m: &Mesh| -> Vec<u32> {
            corner_normals(m)
                .iter()
                .flat_map(|n| bits(&n.array()))
                .collect()
        };
        assert_eq!(normals(&mesh), normals(&mirror), "{}: normals", g.name);
        assert_eq!(
            bits(&right.read(|o| o.loc).array()),
            bits(&g.f32s("at")),
            "{}: where it is",
            g.name
        );

        // a whole slab: no face without area, every edge between two faces; a face of the outline cut into two
        // triangles fewer than it has points, and a wall a side
        let round = mesh.verts.len() / 2;
        let all = faces(&mesh);
        let points: Vec<[f32; 3]> = mesh.verts.iter().map(|v| v.array()).collect();
        assert!(
            !all.iter().any(|f| f.len() == 3 && flat(&points, f)),
            "{}: a face of no area",
            g.name
        );
        assert!(
            sides(&all).values().all(|&n| n == 2),
            "{}: an edge not between two faces",
            g.name
        );
        let count = |faces: &[Vec<u32>], corners: usize| -> usize {
            faces.iter().filter(|f| f.len() == corners).count()
        };
        assert_eq!(
            (count(&all, 3), count(&all, 4)),
            (2 * (round - 2), round),
            "{}: triangles and walls",
            g.name
        );

        // Blender's own right-hand slab: the same points (the far face's numbered otherwise), with faces of no
        // area on its two faces and a wall under each one's long side, an edge of one face only
        let at: BTreeMap<[u32; 3], u32> = (0..points.len())
            .map(|i| (points[i].map(f32::to_bits), i as u32))
            .collect();
        let mine = |v: u32| at[&g.verts[v as usize].map(f32::to_bits)];
        assert_eq!(at.len(), g.verts.len(), "{}: Blender's points", g.name);
        let theirs: Vec<Vec<u32>> = g
            .faces
            .iter()
            .map(|f| f.iter().map(|&v| mine(v)).collect())
            .collect();
        let no_area = theirs
            .iter()
            .filter(|f| f.len() == 3 && flat(&points, f))
            .count();
        let walls_more = count(&theirs, 4) - round;
        let loose = sides(&theirs).values().filter(|&&n| n != 2).count();
        // and where Blender's triangles have area they are this slab's, but for a quad at the head of the 22
        // slats cut the other way on each face
        let cut: Vec<Vec<u32>> = as_set(
            &all.iter()
                .filter(|f| f.len() == 3)
                .cloned()
                .collect::<Vec<_>>(),
        );
        let other_cut = as_set(&theirs)
            .iter()
            .filter(|f| f.len() == 3 && !flat(&points, f) && !cut.contains(f))
            .count();
        let want = match g.name.as_str() {
            "right27" => (6, 3, 0),
            _ => (4, 2, 4),
        };
        assert_eq!(
            (no_area, walls_more, other_cut),
            want,
            "{}: Blender's faces of no area, its walls inside, its triangles cut otherwise",
            g.name
        );
        assert!(loose > 0, "{}: Blender's slab is whole", g.name);

        // alone, it bakes as the left-hand one does, to a level (the rays aren't mirrored with it)
        let (mine, its) = (levels(right), levels(left));
        let apart = (0..mine.len()).map(|i| (mine[i] - its[i]).abs()).max();
        assert!(apart <= Some(1), "{}: baked levels {apart:?} apart", g.name);
        seen += 1;
    }
    assert_eq!(seen, 2);
}
