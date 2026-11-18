//! `decimate::collapse` and `far::decimate`: what a collapse may not do (move a locked vertex, open a closed mesh,
//! turn a face, smear a material), that it reaches its count and gives the same mesh twice; and the committed
//! models whose far versions Blender's DECIMATE modifier made, decimated by the kernel and held to what
//! `cargo check-glb --against` holds such a far version to (its `decimated` rule): the size, the floor, the count
//! and how far the new surface is from the old far file's.
//!
//!   cargo test -p artkit --profile tools --test decimate -- --ignored --nocapture
//!
//! prints that for all 28 of them, each decimated from Blender's model and held against Blender's far version
//! (git's ref/blender, whatever the working tree has by now), and writes each new far version to
//! $DECIMATE_OUT/<family>/<id>.far.glb if set.

use artkit::family::{Family, Model, build_model};
use artkit::far::{self, Surface};
use artkit::materials::{Finish, Finishes, Mat};
use artkit::math::V3;
use artkit::mesh::{Mesh, edge};
use artkit::{decimate, glb, scene, shade};
use std::collections::{BTreeMap, BTreeSet};

type P = [f64; 3];

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: P, b: P) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: P, b: P) -> P {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn points(mesh: &Mesh, f: usize) -> Vec<P> {
    mesh.corners(f)
        .iter()
        .map(|&v| mesh.verts[v as usize].array().map(f64::from))
        .collect()
}

/// A triangle's normal, twice its area long.
fn normal(mesh: &Mesh, f: usize) -> P {
    let p = points(mesh, f);
    cross(sub(p[1], p[0]), sub(p[2], p[0]))
}

fn area(mesh: &Mesh, f: usize) -> f64 {
    let n = normal(mesh, f);
    dot(n, n).sqrt() / 2.0
}

/// A ball of `rings` by `segments` faces standing on the floor: quads, a fan of triangles round each pole.
fn ball(rings: u32, segments: u32, radius: f64) -> Mesh {
    let mut verts = vec![V3::new(0.0, 0.0, (2.0 * radius) as f32)];
    for i in 1..rings {
        let polar = std::f64::consts::PI * i as f64 / rings as f64;
        for j in 0..segments {
            let turn = std::f64::consts::TAU * j as f64 / segments as f64;
            verts.push(V3::new(
                (radius * polar.sin() * turn.cos()) as f32,
                (radius * polar.sin() * turn.sin()) as f32,
                (radius + radius * polar.cos()) as f32,
            ));
        }
    }
    verts.push(V3::new(0.0, 0.0, 0.0));
    let last = verts.len() as u32 - 1;
    let at = |i: u32, j: u32| 1 + (i - 1) * segments + j % segments;
    let mut faces = Vec::new();
    for j in 0..segments {
        faces.push(vec![0, at(1, j), at(1, j + 1)]);
        for i in 1..rings - 1 {
            faces.push(vec![at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)]);
        }
        faces.push(vec![last, at(rings - 1, j + 1), at(rings - 1, j)]);
    }
    Mesh::from_pydata(verts, &faces)
}

/// A ring (a tyre) lying flat on the floor.
fn ring(around: u32, across: u32, major: f64, minor: f64) -> Mesh {
    let mut verts = Vec::new();
    for i in 0..around {
        let a = std::f64::consts::TAU * i as f64 / around as f64;
        for j in 0..across {
            let b = std::f64::consts::TAU * j as f64 / across as f64;
            let r = major + minor * b.cos();
            verts.push(V3::new(
                (r * a.cos()) as f32,
                (r * a.sin()) as f32,
                (minor + minor * b.sin()) as f32,
            ));
        }
    }
    let at = |i: u32, j: u32| (i % around) * across + j % across;
    let mut faces = Vec::new();
    for i in 0..around {
        for j in 0..across {
            faces.push(vec![at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)]);
        }
    }
    Mesh::from_pydata(verts, &faces)
}

/// A box standing on the floor, `size` wide, deep and high.
fn slab(size: [f64; 3]) -> Mesh {
    let [x, y, z] = [size[0] / 2.0, size[1] / 2.0, size[2]];
    let verts = [(-x, -y, 0.0), (x, -y, 0.0), (x, y, 0.0), (-x, y, 0.0)]
        .into_iter()
        .chain([(-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)])
        .map(V3::from)
        .collect();
    Mesh::from_pydata(
        verts,
        &[
            vec![0, 3, 2, 1],
            vec![4, 5, 6, 7],
            vec![0, 1, 5, 4],
            vec![1, 2, 6, 5],
            vec![2, 3, 7, 6],
            vec![3, 0, 4, 7],
        ],
    )
}

const LEFT: Mat = Mat::new("left", "#B03020", "plain");
const RIGHT: Mat = Mat::new("right", "#2040B0", "plain");

/// A flat sheet of `n` by `n` quads, a metre a side, a metre up: its left half of one material, its right of
/// another, with UVs in metres.
fn sheet(n: u32) -> Mesh {
    let mut mesh = Mesh::default();
    for i in 0..=n {
        for j in 0..=n {
            mesh.verts.push(V3::new(
                (i as f64 / n as f64 - 0.5) as f32,
                (j as f64 / n as f64 - 0.5) as f32,
                1.0,
            ));
        }
    }
    let (left, right) = (mesh.slot(LEFT), mesh.slot(RIGHT));
    let at = |i: u32, j: u32| i * (n + 1) + j;
    for i in 0..n {
        for j in 0..n {
            let mat = if i < n / 2 { left } else { right };
            mesh.push_face(
                &[at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)],
                mat,
                false,
            );
        }
    }
    mesh.uvs = Some(
        mesh.loops
            .iter()
            .map(|&v| {
                let p = mesh.verts[v as usize];
                [p.x, p.y]
            })
            .collect(),
    );
    mesh.uv_metres = true;
    mesh
}

/// Everything a mesh is, as bytes.
fn bytes(mesh: &Mesh) -> Vec<u8> {
    let mut out = Vec::new();
    for v in &mesh.verts {
        out.extend(v.array().iter().flat_map(|x| x.to_bits().to_le_bytes()));
    }
    out.extend(mesh.loops.iter().flat_map(|l| l.to_le_bytes()));
    for f in &mesh.faces {
        out.extend(f.start.to_le_bytes());
        out.extend(f.len.to_le_bytes());
        out.extend(f.mat.to_le_bytes());
        out.push(f.smooth as u8);
    }
    for uv in mesh.uvs.iter().flatten() {
        out.extend(uv.iter().flat_map(|x| x.to_bits().to_le_bytes()));
    }
    for &(a, b) in &mesh.sharp {
        out.extend(a.to_le_bytes());
        out.extend(b.to_le_bytes());
    }
    out
}

/// Whether every edge of the mesh has two faces that run it opposite ways: a closed surface, wound one way.
fn closed(mesh: &Mesh) -> bool {
    let mut runs: BTreeMap<(u32, u32), (u32, u32)> = BTreeMap::new();
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        for i in 0..c.len() {
            let (a, b) = (c[i], c[(i + 1) % c.len()]);
            let e = runs.entry(edge(a, b)).or_default();
            if a < b {
                e.0 += 1;
            } else {
                e.1 += 1;
            }
        }
    }
    !runs.is_empty() && runs.values().all(|&r| r == (1, 1))
}

fn free(mesh: &Mesh) -> Vec<bool> {
    vec![false; mesh.verts.len()]
}

/// No triangle of `mesh` (all of them triangles) is without area or all but flat.
fn assert_no_slivers(mesh: &Mesh, what: &str) {
    for f in 0..mesh.faces.len() {
        assert_eq!(mesh.faces[f].len, 3, "{what}: face {f} is no triangle");
        let p = points(mesh, f);
        let longest = (0..3)
            .map(|k| {
                let d = sub(p[(k + 1) % 3], p[k]);
                dot(d, d)
            })
            .fold(0.0, f64::max);
        let twice = 2.0 * area(mesh, f);
        assert!(
            twice > 1e-12 && twice / longest > 1e-5,
            "{what}: triangle {f} has no area to speak of ({p:?})"
        );
    }
}

#[test]
fn the_count_is_met() {
    for (mesh, what) in [
        (ball(16, 32, 0.5), "a ball"),
        (ring(48, 16, 0.5, 0.15), "a ring"),
        (sheet(24), "a sheet"),
    ] {
        let before = mesh.triangles().len();
        for ratio in [0.5, 0.25, 0.1] {
            let done = decimate::collapse(&mesh, ratio, &free(&mesh));
            let want = (before as f64 * ratio).floor() as usize;
            let got = done.faces.len();
            assert!(
                got <= want && got + 2 >= want,
                "{what} of {before} triangles at {ratio}: {got}, want {want}"
            );
            assert_no_slivers(&done, what);
        }
    }
}

#[test]
fn a_ratio_of_one_only_cuts_the_faces_into_triangles() {
    let mesh = ball(8, 12, 0.5);
    let done = decimate::collapse(&mesh, 1.0, &free(&mesh));
    assert_eq!(done.faces.len(), mesh.triangles().len());
    assert_eq!(done.verts, mesh.verts);
    assert!(closed(&done));
}

#[test]
fn a_locked_vertex_never_moves() {
    let mesh = ball(16, 32, 0.5);
    let locked = far::locks(&mesh, 0.1, 0.02);
    let n = locked.iter().filter(|&&l| l).count();
    assert!(n > 40 && n < mesh.verts.len() / 2, "{n} locked");
    let done = decimate::collapse(&mesh, 0.1, &locked);
    // the vertices left keep their order: the locked ones are all there, each where it was, to the bit
    let mut left = done.verts.iter();
    for (v, _) in mesh.verts.iter().zip(&locked).filter(|(_, l)| **l) {
        assert!(
            left.any(|d| d.array().map(f32::to_bits) == v.array().map(f32::to_bits)),
            "the locked vertex at {v:?} moved or went"
        );
    }
    assert!(closed(&done));
    assert!(done.faces.len() < mesh.triangles().len() / 2);
}

#[test]
fn where_every_vertex_is_locked_nothing_goes() {
    let mesh = ball(8, 12, 0.5);
    let done = decimate::collapse(&mesh, 0.1, &vec![true; mesh.verts.len()]);
    assert_eq!(done.faces.len(), mesh.triangles().len());
    assert_eq!(done.verts, mesh.verts);
}

#[test]
fn the_result_is_the_same_bytes_twice() {
    for mesh in [ball(16, 32, 0.5), ring(48, 16, 0.5, 0.15), sheet(24)] {
        let locked = far::locks(&mesh, 0.01, 0.003);
        let (a, b) = (
            decimate::collapse(&mesh, 0.25, &locked),
            decimate::collapse(&mesh.clone(), 0.25, &locked),
        );
        assert!(bytes(&a) == bytes(&b));
        let (mut c, mut d) = (mesh.clone(), mesh.clone());
        far::decimate(&mut c, 0.25, far::FEET, far::EDGE);
        far::decimate(&mut d, 0.25, far::FEET, far::EDGE);
        assert!(bytes(&c) == bytes(&d));
    }
}

#[test]
fn a_closed_mesh_stays_closed() {
    for (mesh, what) in [
        (ball(16, 32, 0.5), "a ball"),
        (ring(48, 16, 0.5, 0.15), "a ring"),
    ] {
        assert!(closed(&mesh), "{what} as built");
        for ratio in [0.5, 0.25, 0.05, 0.0] {
            let done = decimate::collapse(&mesh, ratio, &free(&mesh));
            assert!(closed(&done), "{what} at {ratio}");
            assert!(
                done.faces.len() >= 4,
                "{what} at {ratio}: a closed mesh has four faces at least"
            );
            // (and it is still one surface of as many holes: vertices - edges + faces is what it was)
            let edges: BTreeSet<(u32, u32)> = (0..done.faces.len())
                .flat_map(|f| {
                    let c = done.corners(f).to_vec();
                    (0..3).map(move |i| edge(c[i], c[(i + 1) % 3]))
                })
                .collect();
            let euler = done.verts.len() as i64 - edges.len() as i64 + done.faces.len() as i64;
            assert_eq!(
                euler,
                if what == "a ball" { 2 } else { 0 },
                "{what} at {ratio}"
            );
        }
    }
}

#[test]
fn no_face_is_turned() {
    let radius = 0.5;
    let mesh = ball(16, 32, radius);
    let outwards = |mesh: &Mesh, what: &str| {
        for f in 0..mesh.faces.len() {
            let p = points(mesh, f);
            let n = cross(sub(p[1], p[0]), sub(p[2], p[0]));
            let out = sub(p[0], [0.0, 0.0, radius]);
            assert!(dot(n, out) > 0.0, "{what}: face {f} looks inwards");
        }
    };
    outwards(&mesh, "as built");
    for ratio in [0.5, 0.25, 0.1, 0.02] {
        let done = decimate::collapse(&mesh, ratio, &free(&mesh));
        outwards(&done, &format!("at {ratio}"));
        // and none is far off the ball: a spike would be
        for v in &done.verts {
            let off = sub(v.array().map(f64::from), [0.0, 0.0, radius]);
            let r = dot(off, off).sqrt();
            assert!(
                (0.5 * radius..=1.2 * radius).contains(&r),
                "at {ratio}: a vertex {r} from the middle"
            );
        }
    }
}

#[test]
fn a_part_thicker_than_the_reach_is_not_squashed() {
    // a post: no corner of it can go without leaving a side by its whole width
    let post = slab([0.1, 0.1, 2.0]);
    let done = decimate::collapse(&post, 0.0, &free(&post));
    assert_eq!(done.faces.len(), 12);
    assert_eq!(done.verts, post.verts);
    // a ball asked for no triangles at all: it is brought down as far as its surface stays within the first
    // reach, and no further (the second would not meet the count either)
    let (radius, reach) = (0.5, decimate::REACH[0]);
    let ball = ball(16, 32, radius);
    let done = decimate::collapse(&ball, 0.0, &free(&ball));
    assert!(
        done.faces.len() > 8 && done.faces.len() < ball.triangles().len() / 4,
        "{}",
        done.faces.len()
    );
    for v in &done.verts {
        let off = sub(v.array().map(f64::from), [0.0, 0.0, radius]);
        let r = dot(off, off).sqrt();
        assert!(
            (r - radius).abs() <= reach + 1e-6,
            "a vertex {r} from the middle"
        );
    }
}

#[test]
fn a_thin_panel_goes_flat_and_keeps_its_outline() {
    // a board 2 cm thick: its two sides close up (thinner than the reach), its outline stays
    let panel = slab([1.0, 0.02, 0.6]);
    let done = decimate::collapse(&panel, 0.0, &free(&panel));
    assert_eq!(done.faces.len(), 4);
    assert_no_slivers(&done, "the panel");
    for v in &done.verts {
        assert!(
            (v.x.abs() - 0.5).abs() < 1e-6 && (v.z == 0.0 || (v.z - 0.6).abs() < 1e-6),
            "{v:?}"
        );
    }
    // seen from the front it is as large as it was, and from the back
    for side in [-1.0, 1.0] {
        let seen: f64 = (0..done.faces.len())
            .map(|f| normal(&done, f)[1] * side)
            .filter(|&n| n > 0.0)
            .map(|n| n / 2.0)
            .sum();
        assert!((seen - 0.6).abs() < 1e-6, "{seen}");
    }
    // its two sides lie on one another: each has its own four vertices and its own normal at them, so neither is
    // shaded as the other; no edge is sharp
    assert_eq!(done.verts.len(), 8);
    assert!(done.sharp.is_empty());
    let normals = artkit::normals::corner_normals(&done);
    for f in 0..done.faces.len() {
        let own = normal(&done, f)[1].signum();
        for l in done.faces[f].start..done.faces[f].start + 3 {
            // (to within the board's thickness over its length: its corners closed up where they cost least)
            assert!(
                normals[l as usize].y * own as f32 > 0.999,
                "{:?}",
                normals[l as usize]
            );
        }
    }
    for (e, on) in faces_on_edges(&done) {
        assert_eq!(
            on.len(),
            if on.iter().any(|&f| on.iter().any(|&g| g != f)) {
                2
            } else {
                1
            },
            "{e:?}"
        );
    }
    assert_eq!(written(&done), 8);
}

/// The faces on each of a mesh's edges.
fn faces_on_edges(mesh: &Mesh) -> BTreeMap<(u32, u32), Vec<usize>> {
    let mut on: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        for i in 0..c.len() {
            on.entry(edge(c[i], c[(i + 1) % c.len()]))
                .or_default()
                .push(f);
        }
    }
    on
}

/// How many vertices a file of `mesh` has: one for each normal and UV a vertex has at its corners.
fn written(mesh: &Mesh) -> usize {
    let normals = artkit::normals::corner_normals(mesh);
    let mut corners = BTreeSet::new();
    for (l, &v) in mesh.loops.iter().enumerate() {
        let n = normals[l].array().map(|x| (x * 1e4).round() as i64);
        let uv = mesh.uvs.as_ref().map(|uv| uv[l].map(f32::to_bits));
        corners.insert((v, n, uv));
    }
    corners.len()
}

#[test]
fn a_vertex_is_written_once_where_the_models_was() {
    // a smooth ball: no edge of it is sharp, before or after, and each vertex has one normal
    let mut smooth = ball(16, 32, 0.5);
    smooth.shade_smooth(180.0);
    let done = decimate::collapse(&smooth, 0.25, &free(&smooth));
    assert!(done.sharp.is_empty());
    assert_eq!(written(&done), done.verts.len());
    // the same ball shaded flat: every edge of it is sharp once its faces have turned from one another
    let flat = ball(16, 32, 0.5);
    let done = decimate::collapse(&flat, 0.25, &free(&flat));
    assert!(done.sharp.len() > done.faces.len());
    assert!(written(&done) > 2 * done.verts.len());
    // a ball smooth but for its equator, with a texture laid on it from above that runs on over all of it: a
    // vertex of the equator is written twice (a normal each side), every other once, and the texture is not torn
    let (rings, segments) = (16, 32);
    let mut halves = ball(rings, segments, 0.5);
    halves.shade_smooth(180.0);
    let at = |j: u32| 1 + (rings / 2 - 1) * segments + j % segments;
    for j in 0..segments {
        halves.sharp.insert(edge(at(j), at(j + 1)));
    }
    halves.uvs = Some(
        halves
            .loops
            .iter()
            .map(|&v| [halves.verts[v as usize].x, halves.verts[v as usize].y])
            .collect(),
    );
    halves.uv_metres = true;
    let done = decimate::collapse(&halves, 0.25, &free(&halves));
    let mut on_the_line: BTreeSet<u32> = BTreeSet::new();
    for &(a, b) in &done.sharp {
        on_the_line.extend([a, b]);
    }
    assert!(!on_the_line.is_empty() && on_the_line.len() < done.verts.len() / 4);
    assert_eq!(written(&done), done.verts.len() + on_the_line.len());
    let mut uv_at: BTreeMap<u32, BTreeSet<[u32; 2]>> = BTreeMap::new();
    for (l, &v) in done.loops.iter().enumerate() {
        uv_at
            .entry(v)
            .or_default()
            .insert(done.uvs.as_ref().unwrap()[l].map(f32::to_bits));
    }
    assert!(uv_at.values().all(|uvs| uvs.len() == 1));
}

#[test]
fn a_material_keeps_its_ground_and_a_sheet_its_outline() {
    let mesh = sheet(24);
    let done = decimate::collapse(&mesh, 0.05, &free(&mesh));
    assert!(done.faces.len() <= mesh.triangles().len() / 20);
    let mut ground = [0.0_f64; 2];
    for f in 0..done.faces.len() {
        let mat = done.faces[f].mat as usize;
        ground[mat] += area(&done, f);
        // each face on its own side of the line between the two
        for p in points(&done, f) {
            assert!(
                if mat == 0 {
                    p[0] <= 1e-6
                } else {
                    p[0] >= -1e-6
                },
                "face {f} of material {mat} reaches {p:?}"
            );
            assert!((p[2] - 1.0).abs() < 1e-6);
        }
        assert!(normal(&done, f)[2] > 0.0, "face {f} is turned");
        // and its texture where it was: a metre of it a metre of the sheet
        let at = done.faces[f].start as usize;
        for (k, &v) in done.corners(f).iter().enumerate() {
            let (p, uv) = (done.verts[v as usize], done.uvs.as_ref().unwrap()[at + k]);
            assert!(
                (uv[0] - p.x).abs() < 1e-5 && (uv[1] - p.y).abs() < 1e-5,
                "face {f}: {uv:?} at {p:?}"
            );
        }
    }
    assert!(
        (ground[0] - 0.5).abs() < 1e-6 && (ground[1] - 0.5).abs() < 1e-6,
        "{ground:?}"
    );
    assert_eq!(done.mats.len(), 2);
    // the sheet is flat and its texture runs on over it: a vertex is written once a material
    assert!(done.sharp.is_empty());
    let on_the_line = done.verts.iter().filter(|v| v.x.abs() < 1e-6).count();
    assert_eq!(written(&done), done.verts.len());
    assert!(on_the_line >= 2);
}

#[test]
fn a_sharp_edge_left_stays_sharp() {
    // a ball with its equator sharp: decimated, the edges left along the equator are sharp and no other is
    let (rings, segments) = (16, 32);
    let mut mesh = ball(rings, segments, 0.5);
    mesh.shade_smooth(180.0);
    let at = |j: u32| 1 + (rings / 2 - 1) * segments + j % segments;
    for j in 0..segments {
        mesh.sharp.insert(edge(at(j), at(j + 1)));
    }
    let done = decimate::collapse(&mesh, 0.25, &free(&mesh));
    assert!(!done.sharp.is_empty());
    assert!(done.faces.iter().all(|f| f.smooth));
    for &(a, b) in &done.sharp {
        for v in [a, b] {
            // (within a ring of it: nothing holds a collapse's point to a line the ball has no crease along)
            assert!(
                (done.verts[v as usize].z - 0.5).abs() < 0.1,
                "{:?}",
                done.verts[v as usize]
            );
        }
    }
}

#[test]
fn a_far_version_keeps_its_feet_and_its_box() {
    // two halves of a ball that meet in a seam of doubled vertices, as joined parts do
    let whole = ball(16, 32, 0.5);
    let mut mesh = Mesh::default();
    for half in 0..2 {
        let base = mesh.verts.len() as u32;
        mesh.verts.extend(&whole.verts);
        for f in 0..whole.faces.len() {
            let c = whole.corners(f);
            let middle = c.iter().map(|&v| whole.verts[v as usize].z).sum::<f32>() / c.len() as f32;
            if (middle > 0.5) == (half == 0) {
                mesh.push_face(&c.iter().map(|&v| v + base).collect::<Vec<_>>(), 0, true);
            }
        }
    }
    assert!(!closed(&mesh));
    let before = mesh.bounds(&artkit::math::M4::IDENTITY).unwrap();
    let triangles = mesh.triangles().len();
    let locked = far::locks(&mesh, far::FEET, far::EDGE);
    let kept: Vec<V3> = mesh
        .verts
        .iter()
        .zip(&locked)
        .filter(|(_, l)| **l)
        .map(|(v, _)| *v)
        .collect();
    assert!(
        kept.len() >= 6,
        "the ball's poles and the ends of its equator are locked"
    );
    let turned = far::decimate(&mut mesh, 0.25, far::FEET, far::EDGE);
    assert_eq!(turned, 0);
    assert!(closed(&mesh), "the seam is merged before it is decimated");
    assert_eq!(mesh.bounds(&artkit::math::M4::IDENTITY).unwrap(), before);
    assert!(mesh.faces.len() <= triangles / 4 && mesh.faces.len() + 2 >= triangles / 4);
    assert!(mesh.faces.iter().all(|f| f.smooth));
    for v in kept {
        assert!(
            mesh.verts.contains(&v),
            "the locked vertex at {v:?} moved or went"
        );
    }
}

/// A fluted shaft standing on the floor: `flutes` shallow flutes round it, three points a flute, smooth.
fn shaft(flutes: u32, rings: u32, radius: f64, height: f64) -> Mesh {
    let around = flutes * 3;
    let mut verts = Vec::new();
    for i in 0..=rings {
        for j in 0..around {
            let turn = std::f64::consts::TAU * j as f64 / around as f64;
            let r = radius * (1.0 - 0.06 * (0.5 + 0.5 * (flutes as f64 * turn).cos()));
            verts.push(V3::new(
                (r * turn.cos()) as f32,
                (r * turn.sin()) as f32,
                (height * i as f64 / rings as f64) as f32,
            ));
        }
    }
    let at = |i: u32, j: u32| i * around + j % around;
    let mut faces = Vec::new();
    for i in 0..rings {
        for j in 0..around {
            faces.push(vec![at(i, j), at(i, j + 1), at(i + 1, j + 1), at(i + 1, j)]);
        }
    }
    let mut mesh = Mesh::from_pydata(verts, &faces);
    mesh.shade_smooth(180.0);
    mesh
}

#[test]
fn a_round_part_stays_smooth_and_as_light_as_it_was() {
    // a fluted shaft, its flutes' middles darker than their edges in the model's baked shading
    let (flutes, radius) = (16, 0.14);
    let mut model = shaft(flutes, 4, radius, 0.7);
    let dark = |v: &V3| {
        let r = (v.x as f64).hypot(v.y as f64);
        r < radius * 0.955
    };
    model.ao = Some(
        model
            .verts
            .iter()
            .map(|v| if dark(v) { 0.5 } else { 1.0 })
            .collect(),
    );
    let sunk = model.verts.iter().filter(|v| dark(v)).count();
    assert_eq!(sunk, model.verts.len() / 3);
    let mut done = model.clone();
    far::decimate(&mut done, 0.25, far::FEET, far::EDGE);
    // (its two rims are its far version's foot and its top: they stay, and what is between them goes)
    assert!(
        done.faces.len() * 2 <= model.triangles().len(),
        "{}",
        done.faces.len()
    );
    // it is as smooth as it was: a normal a vertex, and each within a flute's turn of straight out from the axis,
    // though its faces lie as its flutes were (up to 30 degrees off)
    assert!(done.sharp.is_empty());
    assert_eq!(written(&done), done.verts.len());
    let normals = artkit::normals::corner_normals(&done);
    for (l, &v) in done.loops.iter().enumerate() {
        let (p, n) = (done.verts[v as usize], normals[l]);
        let out = (p.x as f64).hypot(p.y as f64);
        let cosine = (n.x as f64 * p.x as f64 + n.y as f64 * p.y as f64) / out;
        assert!(cosine > 0.9 && n.z.abs() < 0.2, "{n:?} at {p:?}");
    }
    // and as light: no vertex left is as dark as a flute's middle was, most are near the light of its edges
    let shade = done.card_shade.as_ref().unwrap();
    assert_eq!(shade.len(), done.verts.len());
    let mean = shade.iter().sum::<f32>() / shade.len() as f32;
    assert!(shade.iter().all(|&s| s > 0.6 && s <= 1.0), "{shade:?}");
    assert!(mean > 0.8, "{mean}");
    // a mesh shaded alike all over is shaded so still
    let mut even = shaft(flutes, 4, radius, 0.7);
    even.ao = Some(vec![0.75; even.verts.len()]);
    far::decimate(&mut even, 0.25, far::FEET, far::EDGE);
    assert!(
        even.card_shade
            .as_ref()
            .unwrap()
            .iter()
            .all(|&s| (s - 0.75).abs() < 1e-6)
    );
    // and one with no shading is given none
    let mut bare = shaft(flutes, 4, radius, 0.7);
    far::decimate(&mut bare, 0.25, far::FEET, far::EDGE);
    assert!(bare.card_shade.is_none() && bare.ao.is_none());
}

// A model that asks for a decimated far version.

artkit::palette! { LAMP: glass = ("#DCE6EA", plain), iron = ("#30343A", plain) }

/// A globe on a post, and water that is hidden at rest.
fn lamp() -> Vec<artkit::scene::Obj> {
    let mut globe = ball(16, 32, 0.25);
    globe.translate(V3::new(0.0, 0.0, 1.0));
    globe.set_material(Some(LAMP.glass));
    globe.shade_smooth(180.0);
    scene::add("globe", globe);
    let mut post = slab([0.1, 0.1, 1.05]);
    post.set_material(Some(LAMP.iron));
    scene::add("post", post);
    let mut water = ball(8, 12, 0.5);
    water.set_material(Some(LAMP.glass));
    scene::add("water", water).set_scale((0.001, 0.001, 0.001));
    Vec::new()
}

/// A line of the build's log without the kernel's mark before it.
fn said_of(line: &str) -> &str {
    line.split_once("] ").map_or(line, |(_, said)| said)
}

fn plain() -> Finishes {
    let mut finishes = Finishes::new();
    finishes.insert(
        "plain".into(),
        Finish {
            roughness: 0.6,
            metallic: 0.0,
            textured: false,
        },
    );
    finishes
}

#[test]
fn a_model_that_asks_for_a_share_of_its_triangles_gets_its_far_version() {
    scene::set_finishes(plain());
    let family = Family::new(
        "lamps",
        LAMP.palette(),
        vec![Model::new("lamp", lamp, (0.5, 1.5, 0.5)).far_ratio(0.25)],
        "a lamp alone",
    );
    scene::take_log();
    let out = build_model(&family, &family.models[0]).unwrap();
    let log = scene::take_log();
    let (model, far) = (
        glb::read(&out.glb).unwrap(),
        glb::read(out.far.as_ref().unwrap()).unwrap(),
    );
    assert_eq!(model.nodes.len(), 3);
    assert_eq!(triangles(&model).len(), 960 + 12 + 168);
    // one part, without the water: a quarter of the globe's and the post's triangles
    assert_eq!(far.nodes.len(), 1);
    assert_eq!(far.nodes[0].name, "lamp_far");
    let left = triangles(&far);
    assert!(left.len() <= 243 && left.len() >= 241, "{}", left.len());
    assert_eq!(far.materials.len(), 2);
    // as large as the model, on the floor
    let ((lo, hi), (mlo, mhi)) = (bounds(&left), bounds(&triangles(&model)));
    assert_eq!((lo, hi), (mlo, mhi));
    assert_eq!(lo[2], 0.0);
    // the post is whole: decimating the globe was enough
    let iron = far.materials.iter().position(|m| m.name == "iron").unwrap();
    let post: usize = far.meshes[0]
        .primitives
        .iter()
        .filter(|p| p.material == Some(iron))
        .map(|p| p.indices.len() / 3)
        .sum();
    assert_eq!(post, 12);
    let said = |line: &str| {
        assert!(
            log.iter().any(|l| said_of(l) == line),
            "no {line:?} in {log:#?}"
        )
    };
    said(&format!("lamp_far: {} faces, 0 turned back", left.len()));
    said(&format!("lamp.far: {} triangles (25% kept)", left.len()));
    said("built its far version -> models/lamps/lamp.far.glb");
    // its shading is the model's, kept by its vertices: none of it darker than the model's darkest
    let greys = |m: &glb::Model| -> Vec<f32> {
        m.meshes
            .iter()
            .flat_map(|me| me.primitives.iter())
            .flat_map(|p| p.colours.as_ref().unwrap().iter().map(|c| c[0]))
            .collect()
    };
    let darkest = |m: &glb::Model| greys(m).into_iter().fold(1.0, f32::min);
    assert!(
        darkest(&far) >= darkest(&model) - 1e-4,
        "{} against {}",
        darkest(&far),
        darkest(&model)
    );
    assert!(greys(&far).iter().any(|&g| g < 0.99), "it is shaded");
    // and the same file a second time
    assert!(build_model(&family, &family.models[0]).unwrap().far == out.far);
}

/// The lamp with a flag on its globe: drawn with it, no part of its size.
fn lamp_with_a_flag() -> Vec<artkit::scene::Obj> {
    lamp();
    let mut flag = slab([0.3, 0.02, 0.4]);
    flag.translate(V3::new(0.0, 0.0, 1.5));
    flag.set_material(Some(LAMP.iron));
    let flag = scene::add("flag", flag);
    artkit::parts::aside(&[flag], "flag");
    Vec::new()
}

#[test]
fn what_a_model_carries_aside_is_in_its_far_version() {
    scene::set_finishes(plain());
    let family = Family::new(
        "lamps",
        LAMP.palette(),
        vec![Model::new("flagged", lamp_with_a_flag, (0.5, 1.5, 0.5)).far_ratio(0.25)],
        "a lamp with a flag",
    );
    scene::take_log();
    // (the model is 1.5 m tall, 1.9 m with its flag: its far version is not held to the 1.5 m again)
    let out = build_model(&family, &family.models[0]).unwrap();
    let log = scene::take_log();
    let (model, far) = (
        glb::read(&out.glb).unwrap(),
        glb::read(out.far.as_ref().unwrap()).unwrap(),
    );
    assert!(model.nodes.iter().any(|n| n.name == "flag_aside"));
    // one part, the flag joined into it with the rest, without the water
    assert_eq!(far.nodes.len(), 1);
    assert_eq!(far.nodes[0].name, "flagged_far");
    let left = triangles(&far);
    let before = 960 + 12 + 12;
    assert!(
        left.len() <= before / 4 && left.len() + 2 >= before / 4,
        "{}",
        left.len()
    );
    let (lo, hi) = bounds(&left);
    assert_eq!((lo[2], hi[2]), (0.0, f64::from(1.9_f32)));
    assert_eq!((lo, hi), bounds(&triangles(&model)));
    // the flag whole (its corners are the far version's highest and so locked, and it is thinner than a
    // collapse may squash only across), the post whole
    let top: Vec<&[V3; 3]> = left
        .iter()
        .filter(|t| t.iter().all(|p| p.z >= 1.5))
        .collect();
    assert!(top.len() >= 4, "{} triangles of the flag", top.len());
    assert!(
        log.iter()
            .any(|l| said_of(l) == format!("flagged.far: {} triangles (25% kept)", left.len())),
        "{log:#?}"
    );
    assert!(
        !log.iter().any(|l| said_of(l).starts_with("flagged.far ")),
        "its size is not said again: {log:#?}"
    );
}

// The committed models.

/// Linear 0..1 as the sRGB byte.
fn srgb(c: f32) -> u8 {
    let c = c as f64;
    let s = if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

/// The file's (x, y, z), +Y up, as Blender's.
fn blender(p: [f32; 3]) -> V3 {
    V3::new(p[0], -p[2], p[1])
}

/// Whether a node, or one it hangs from, is hidden at rest (scaled to nothing: water that grows from it).
fn hidden(model: &glb::Model, n: usize) -> bool {
    let mut parent = vec![None; model.nodes.len()];
    for (p, node) in model.nodes.iter().enumerate() {
        for &c in &node.children {
            parent[c] = Some(p);
        }
    }
    let mut at = Some(n);
    while let Some(n) = at {
        if model.nodes[n].scale.iter().any(|k| k.abs() < 0.01) {
            return true;
        }
        at = parent[n];
    }
    false
}

/// A model's file as `make_far` joins the model: every part that shows at rest, where it stands, as one mesh
/// in Blender's axes; a vertex the file has in several (one a normal) is one, and the edges it is split along
/// are sharp.
fn joined(model: &glb::Model) -> Mesh {
    let world = model.world(None);
    let mut mesh = Mesh::default();
    let mut index: BTreeMap<[u32; 3], u32> = BTreeMap::new();
    // each edge's corners as the file numbers them: more than one pair, and it is split there
    type Corners = BTreeSet<(usize, u32, u32)>;
    let mut pairs: BTreeMap<(u32, u32), Corners> = BTreeMap::new();
    let mut part = 0;
    let mut seen: BTreeSet<[u32; 3]> = BTreeSet::new();
    for n in model.drawn() {
        let Some(m) = model.nodes[n].mesh.and_then(|m| model.meshes.get(m)) else {
            continue;
        };
        if hidden(model, n) {
            continue;
        }
        for p in &m.primitives {
            part += 1;
            let mat = match p.material.and_then(|i| model.materials.get(i)) {
                Some(material) => Mat {
                    name: Box::leak(material.name.clone().into_boxed_str()),
                    colour: [
                        srgb(material.base[0]),
                        srgb(material.base[1]),
                        srgb(material.base[2]),
                    ],
                    ..LEFT
                },
                None => LEFT,
            };
            let slot = mesh.slot(mat);
            let ids: Vec<u32> = p
                .positions
                .iter()
                .map(|&v| {
                    let at = blender(glb::apply(&world[n], v));
                    *index
                        .entry(at.array().map(f32::to_bits))
                        .or_insert_with(|| {
                            mesh.verts.push(at);
                            mesh.verts.len() as u32 - 1
                        })
                })
                .collect();
            for t in p.indices.as_chunks::<3>().0 {
                let c = [ids[t[0] as usize], ids[t[1] as usize], ids[t[2] as usize]];
                let mut sorted = c;
                sorted.sort_unstable();
                // (as merging the joined parts' doubled vertices leaves it: no face without area, and one of
                // two on the same three vertices, where two parts lie against each other)
                if c[0] == c[1] || c[1] == c[2] || c[2] == c[0] || !seen.insert(sorted) {
                    continue;
                }
                mesh.push_face(&c, slot, true);
                for k in 0..3 {
                    let (a, b) = (k, (k + 1) % 3);
                    let (a, b) = if c[a] < c[b] { (a, b) } else { (b, a) };
                    pairs
                        .entry((c[a], c[b]))
                        .or_default()
                        .insert((part, t[a], t[b]));
                }
            }
        }
    }
    mesh.sharp = pairs
        .into_iter()
        .filter(|(_, p)| p.len() > 1)
        .map(|(e, _)| e)
        .collect();
    mesh
}

/// A file's triangles at rest, in Blender's axes.
fn triangles(model: &glb::Model) -> Vec<[V3; 3]> {
    let world = model.world(None);
    let mut out = Vec::new();
    for n in model.drawn() {
        let Some(m) = model.nodes[n].mesh.and_then(|m| model.meshes.get(m)) else {
            continue;
        };
        for p in &m.primitives {
            let at = |i: u32| blender(glb::apply(&world[n], p.positions[i as usize]));
            out.extend(
                p.indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .map(|t| [at(t[0]), at(t[1]), at(t[2])]),
            );
        }
    }
    out
}

fn bounds(tris: &[[V3; 3]]) -> (P, P) {
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for p in tris.iter().flatten() {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k] as f64);
            hi[k] = hi[k].max(p[k] as f64);
        }
    }
    (lo, hi)
}

/// How far `from`'s surface is from `onto`'s (m): the mean over points spread evenly on it (10,000 a square
/// metre, 2,000 at least and 50,000 at most, as the comparison takes them), and the most over those and its corners.
fn across(from: &[[V3; 3]], onto: &Surface) -> (f64, f64, P) {
    let corners = |t: &[V3; 3]| t.map(|v| v.array().map(f64::from));
    let areas: Vec<f64> = from
        .iter()
        .map(|t| {
            let [a, b, c] = corners(t);
            let n = cross(sub(b, a), sub(c, a));
            dot(n, n).sqrt() / 2.0
        })
        .collect();
    let total: f64 = areas.iter().sum();
    let n = ((total * 1e4).round() as usize).clamp(2000, 50000);
    let (mut sum, mut most, mut worst) = (0.0, 0.0_f64, [0.0; 3]);
    let (mut t, mut before) = (0, 0.0);
    for i in 0..n {
        let target = (i as f64 + 0.5) / n as f64 * total;
        while t + 1 < areas.len() && before + areas[t] < target {
            before += areas[t];
            t += 1;
        }
        let (mut u, mut v) = (
            (0.5 + i as f64 * 0.754_877_666_246_692_7).fract(),
            (0.5 + i as f64 * 0.569_840_290_998_053_2).fract(),
        );
        if u + v > 1.0 {
            (u, v) = (1.0 - u, 1.0 - v);
        }
        let [a, b, c] = corners(&from[t]);
        let p: P = std::array::from_fn(|k| a[k] * (1.0 - u - v) + b[k] * u + c[k] * v);
        let d = onto.distance(p).unwrap();
        sum += d;
        if d > most {
            (most, worst) = (d, p);
        }
    }
    for (t, area) in from.iter().zip(&areas) {
        if *area > 0.0 {
            for p in corners(t) {
                let d = onto.distance(p).unwrap();
                if d > most {
                    (most, worst) = (d, p);
                }
            }
        }
    }
    (sum / n as f64, most, worst)
}

/// What the comparison's `decimated` rule measures of a new far version against the old one and the model.
struct Measured {
    model: usize,
    old: usize,
    new: usize,
    turned: usize,
    /// Each side of its box against the old far version's, as a share.
    size: f64,
    /// Its lowest point over the model's (m).
    floor: f64,
    mean: f64,
    most: f64,
    /// And against the model's own surface.
    model_mean: f64,
    model_most: f64,
}

impl Measured {
    /// The rule: each side within 2 %, the lowest point within 0.5 cm of the model's, at most a tenth more
    /// triangles, no point further than 60 mm from the old surface.
    fn passes(&self) -> bool {
        self.size <= 0.02
            && self.floor.abs() <= 0.005
            && self.new as f64 <= self.old as f64 * 1.1
            && self.most <= 0.06
    }

    fn row(&self, name: &str) -> String {
        format!(
            "{name:<34} {:>6} {:>6} {:>6} {:>5.1} % {:>6.2} % {:>+7.2} mm {:>6.2} mm {:>6.1} mm {:>6.2} mm {:>6.1} mm {:>3} {}",
            self.model,
            self.old,
            self.new,
            100.0 * self.new as f64 / self.model as f64,
            100.0 * self.size,
            1000.0 * self.floor,
            1000.0 * self.mean,
            1000.0 * self.most,
            1000.0 * self.model_mean,
            1000.0 * self.model_most,
            self.turned,
            if self.passes() { "ok" } else { "FAILS" }
        )
    }
}

const HEAD: &str = "model                               model    old    new   kept    size      floor    mean     most   (model: mean     most) turned";

fn read(name: &str, far: bool) -> glb::Model {
    let path = format!(
        "{}/../../web/public/assets/models/{name}{}.glb",
        env!("CARGO_MANIFEST_DIR"),
        if far { ".far" } else { "" }
    );
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    glb::read(&data).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A model's file (`<family>/<id>`) or its far version's as Blender built it: the reference the port is held
/// against (the revision ref/blender), whatever the working tree has by now.
fn blenders(name: &str, far: bool) -> glb::Model {
    let path = format!(
        "ref/blender:web/public/assets/models/{name}{}.glb",
        if far { ".far" } else { "" }
    );
    let out = std::process::Command::new("git")
        .args(["-C", env!("CARGO_MANIFEST_DIR"), "show", &path])
        .output()
        .unwrap_or_else(|e| panic!("git show {path}: {e}"));
    assert!(
        out.status.success(),
        "git show {path}: no such file or revision"
    );
    glb::read(&out.stdout).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A model's file decimated as its far version is, and that measured against `old`, a far version of it.
fn decimated(model: &glb::Model, old: &glb::Model, ratio: f64) -> (Mesh, Measured) {
    let old = triangles(old);
    let whole = triangles(model);
    let mut mesh = joined(model);
    let turned = far::decimate(&mut mesh, ratio, far::FEET, far::EDGE);
    let new: Vec<[V3; 3]> = (0..mesh.faces.len())
        .map(|f| {
            let c = mesh.corners(f);
            [0, 1, 2].map(|k| mesh.verts[c[k] as usize])
        })
        .collect();
    let ((olo, ohi), (nlo, nhi), (mlo, _)) = (bounds(&old), bounds(&new), bounds(&whole));
    let size = (0..3)
        .map(|k| ((nhi[k] - nlo[k]) - (ohi[k] - olo[k])).abs() / (ohi[k] - olo[k]))
        .fold(0.0, f64::max);
    let (old_surface, new_surface, model_surface) = (
        Surface::new(old.clone()),
        Surface::new(new.clone()),
        Surface::new(whole.clone()),
    );
    let (ab, ba) = (across(&old, &new_surface), across(&new, &old_surface));
    let (am, ma) = (across(&whole, &new_surface), across(&new, &model_surface));
    let measured = Measured {
        model: whole.len(),
        old: old.len(),
        new: new.len(),
        turned,
        size,
        floor: nlo[2] - mlo[2],
        mean: ab.0.max(ba.0),
        most: ab.1.max(ba.1),
        model_mean: am.0.max(ma.0),
        model_most: am.1.max(ma.1),
    };
    (mesh, measured)
}

/// Thin legs, a drinks machine, wheels, glass.
#[test]
fn committed_models_decimated_pass_the_comparisons_rule() {
    println!("{HEAD}");
    for name in [
        "shops/cafeChair",
        "stalls/vendingDrink",
        "stalls/iceCreamCart",
        "street_furniture/phoneBooth",
    ] {
        let (mesh, measured) = decimated(&read(name, false), &read(name, true), 0.25);
        println!("{}", measured.row(name));
        assert!(measured.passes(), "{name}: {}", measured.row(name));
        // a quarter of the model's triangles, or as near as its feet and its box allow
        assert!(
            measured.new * 4 <= measured.model + 8,
            "{name}: {}",
            measured.row(name)
        );
        assert_no_slivers(&mesh, name);
        assert_eq!(measured.turned, 0, "{name}");
    }
}

/// The 28 models whose far versions Blender's DECIMATE modifier made (tools/art/port-accepted.json's
/// `decimated`), each to a quarter as its family script asks.
const DECIMATED: [&str; 28] = [
    "farm/scarecrow",
    "garden_things/lampPost",
    "playground/basketballHoop",
    "playground/climbingFrame",
    "playground/merryGoRound",
    "playground/pingPongTable",
    "playground/seesaw",
    "playground/slide",
    "playground/springRider",
    "shops/bakeryCase",
    "shops/cafeChair",
    "shops/groceryShelf",
    "shops/shopCounter",
    "stalls/hotDogStand",
    "stalls/iceCreamCart",
    "stalls/marketStall",
    "stalls/vendingDrink",
    "stalls/vendingSnack",
    "street_furniture/busStop",
    "street_furniture/drinkingFountain",
    "street_furniture/fountainTown",
    "street_furniture/litterBin",
    "street_furniture/noticeBoard",
    "street_furniture/parkBench",
    "street_furniture/phoneBooth",
    "street_furniture/postBox",
    "street_furniture/streetLamp",
    "street_furniture/townStatue",
];

/// The six whose new far version does not pass the rule against Blender's, and why: Blender's far file is not the
/// model (two), has fewer triangles than the share it was asked for (three), or both (one).
const UNLIKE_BLENDERS: [(&str, &str); 6] = [
    (
        "playground/pingPongTable",
        "Blender's is 29 mm taller than its model and up to 191 mm off it (its top is gone but for the lines); \
         the kernel's has the model's box, is 36 mm off it at most and has 284 triangles to Blender's 267",
    ),
    (
        "shops/groceryShelf",
        "a quarter of the model's triangles to Blender's 21.8 %: its export dropped the faces its decimation \
         left back to back",
    ),
    (
        "stalls/marketStall",
        "a quarter of the model's triangles to Blender's 21.7 %, as the grocery shelf",
    ),
    (
        "stalls/vendingSnack",
        "a quarter of the model's triangles to Blender's 21.9 %, as the grocery shelf",
    ),
    (
        "street_furniture/busStop",
        "Blender's has lost its bench, half of each pane and its posts' feet: up to 386 mm off its model",
    ),
    (
        "street_furniture/noticeBoard",
        "its posts, frame and notes cannot be had in a quarter of their triangles without being squashed: 27.4 % \
         of the model's to Blender's 22.2 %, whose cork board is gone",
    ),
];

#[test]
#[ignore = "all 28 decimated far versions against Blender's (git's ref/blender): a table to read, and files to look at"]
fn every_decimated_far_version() {
    let out = std::env::var("DECIMATE_OUT").ok();
    let finishes = plain();
    scene::set_finishes(finishes.clone());
    println!("{HEAD}");
    let mut unlike = Vec::new();
    for name in DECIMATED {
        let started = std::time::Instant::now();
        let (mesh, measured) = decimated(&blenders(name, false), &blenders(name, true), 0.25);
        println!(
            "{}  ({:.2} s)",
            measured.row(name),
            started.elapsed().as_secs_f64()
        );
        if !measured.passes() {
            unlike.push(name);
        }
        // whatever Blender's is, the kernel's stands where its model does and is nowhere further from it than
        // the rule lets two far versions be
        assert_eq!(measured.floor, 0.0, "{name}");
        assert!(measured.model_most <= 0.06, "{name}");
        assert_eq!(measured.turned, 0, "{name}");
        assert_no_slivers(&mesh, name);
        if let Some(out) = &out {
            // as the runner writes a far version: one part, shaded by itself and the floor
            scene::reset();
            let id = name.split('/').next_back().unwrap();
            let obj = scene::add(&format!("{id}_far"), mesh);
            shade::bake_ao(&[obj], None, true);
            let path = format!("{out}/{name}.far.glb");
            std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
            std::fs::write(&path, glb::write(&[obj], false, &finishes)).unwrap();
        }
    }
    for (name, why) in UNLIKE_BLENDERS {
        println!("{name}: {why}");
    }
    assert_eq!(unlike, UNLIKE_BLENDERS.map(|(name, _)| name));
}
