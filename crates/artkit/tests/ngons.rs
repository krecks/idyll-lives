//! Polygons of five corners and more, cut into triangles as Blender 5.2 cuts them (tests/golden/ngons.txt): the
//! undersides of the beds' resting soft boxes as bedroom.py made them (the upholstered queen's duvet whole, then as
//! it lies in the bed), caps with a hollow (a swept pad's ends, slabs in an L, a U and a comb, a star), round caps,
//! polygons that aren't flat, one of no area and one with a point twice. For each, the mesh's own triangles
//! (`mesh.loop_triangles`: what the glTF exporter writes and `scene.ray_cast` hits), triangle for triangle in
//! Blender's order and from Blender's corner, and what rays cast at it from just off its polygons meet in the tree
//! the shading is baked with (`BVHTree.FromPolygons`): the polygon, how far to the bit, and where.
mod common;

use artkit::bvh::Polygons;
use artkit::materials::Mat;
use artkit::math::V3;
use artkit::mesh::Mesh;
use artkit::place::GameFrame;
use artkit::scene;
use artkit::soft::{Puff, SoftBox};
use common::*;

fn points(g: &GoldenMesh) -> Vec<V3> {
    g.verts.iter().map(|v| V3::new(v[0], v[1], v[2])).collect()
}

fn case(name: &str) -> GoldenMesh {
    golden_meshes("ngons.txt")
        .into_iter()
        .find(|g| g.name == name)
        .unwrap_or_else(|| panic!("no mesh {name} in the golden"))
}

#[test]
fn a_mesh_has_the_triangles_blenders_has() {
    let all = golden_meshes("ngons.txt");
    assert_eq!(all.len(), 57);
    let (mut polygons, mut triangles) = (0, 0);
    for g in &all {
        let mesh = Mesh::from_pydata(points(g), &g.faces);
        let want_faces: Vec<u32> = g.lines["tp"]
            .split(' ')
            .map(|w| w.parse().unwrap())
            .collect();
        let got = mesh.triangles();
        let got_faces: Vec<u32> = got.iter().map(|t| t.0).collect();
        assert_eq!(got_faces, want_faces, "{}: each triangle's polygon", g.what);
        let got_verts = mesh.triangle_verts();
        for (i, (got, want)) in got_verts.iter().zip(&g.tris).enumerate() {
            assert_eq!(
                got, want,
                "{}: triangle {i} (of polygon {})",
                g.what, want_faces[i]
            );
        }
        assert_eq!(got_verts.len(), g.tris.len(), "{}", g.what);
        polygons += g.faces.iter().filter(|f| f.len() > 4).count();
        triangles += g.tris.len();
    }
    assert_eq!((polygons, triangles), (POLYGONS, TRIANGLES));
}

/// How many polygons of five corners or more the golden has, and how many triangles in all.
const POLYGONS: usize = 68;
const TRIANGLES: usize = 3001;

#[test]
fn rays_at_a_polygon_meet_what_blenders_tree_gives() {
    let (mut rays, mut hits) = (0, 0);
    for g in golden_meshes("ngons.txt") {
        let Some(line) = g.lines.get("rays") else {
            continue;
        };
        let tree = Polygons::new(&points(&g), &g.faces);
        for (n, ray) in line.split(" | ").enumerate() {
            let w: Vec<&str> = ray.split(' ').collect();
            let (o, d) = (f32s(&w[0..3]), f32s(&w[3..6]));
            let got = tree.ray_cast(V3::new(o[0], o[1], o[2]), V3::new(d[0], d[1], d[2]), 1.0);
            rays += 1;
            if w[6] == "-1" {
                assert!(got.is_none(), "{}: ray {n} met nothing in Blender", g.what);
                continue;
            }
            let hit = got.unwrap_or_else(|| panic!("{}: ray {n} met nothing", g.what));
            assert_eq!(
                hit.index,
                w[6].parse::<u32>().unwrap(),
                "{}: ray {n}: the polygon",
                g.what
            );
            assert_eq!(
                hit.dist.to_bits(),
                f32w(w[7]).to_bits(),
                "{}: ray {n}: how far",
                g.what
            );
            assert_eq!(
                bits(&hit.at.array()),
                bits(&f32s(&w[8..11])),
                "{}: ray {n}: where",
                g.what
            );
            hits += 1;
        }
    }
    assert_eq!((rays, hits), (RAYS, HITS));
}

/// How many rays the golden has, and how many of them met a polygon.
const RAYS: usize = 1584;
const HITS: usize = 1457;

/// The upholstered queen's duvet as `bedroom.py` made it (`_duvet`: the call as tests/golden/soft.txt has it,
/// case 63), built by the kernel: its points are Blender's to the bit and in Blender's order, so its triangles are
/// Blender's as a set, the 46 of its underside among them (the order of its faces is the kernel's own).
#[test]
fn the_queens_duvet_is_cut_as_blenders() {
    let g = case("duvet_queen");
    let d = |bits: u64| f64::from_bits(bits);
    scene::reset();
    let obj = {
        GameFrame::new(0.0).soft(
            "duvet",
            d(0x3ffc_4189_374b_c6a8),
            d(0x3fcc_ed91_6872_b023),
            d(0x3fec_85cd_7b90_0aed),
            (0, d(0x3fdc_7ae1_47ae_147c), d(0x3fe0_8dfe_a279_83c2)),
            Mat::new("bedding_oat", "#808080", "fabric"),
            SoftBox {
                round: d(0x3fae_b851_eb85_1eb8),
                puff: Puff::Faces(d(0x3f88_9374_bc6a_7efa)),
                pinch: Some(0.0),
                rumple: d(0x3f9e_b851_eb85_1eb8),
                rumple_size: d(0x3fd0_a3d7_0a3d_70a4),
                seed: d(0x3feb_5c28_f5c2_8f5c),
                resting: true,
                ..SoftBox::default()
            },
        )
    };
    let mesh = obj.read(|p| p.mesh.clone());
    assert_eq!(points_off(&mesh.verts, &g.verts), (0, 0));
    let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect();
    assert_eq!(sorted(&faces), sorted(&g.faces));
    assert_eq!(sorted(&mesh.triangle_verts()), sorted(&g.tris));
    let under = faces.iter().position(|f| f.len() > 4).unwrap();
    let under_tris: Vec<[u32; 3]> = mesh
        .triangles()
        .into_iter()
        .filter(|t| t.0 as usize == under)
        .map(|(_, t)| t.map(|i| faces[under][i as usize]))
        .collect();
    let golden_under = g.faces.iter().position(|f| f.len() > 4).unwrap();
    let want: Vec<[u32; 3]> = g
        .tris
        .iter()
        .zip(g.lines["tp"].split(' '))
        .filter(|(_, p)| p.parse::<usize>().unwrap() == golden_under)
        .map(|(t, _)| *t)
        .collect();
    assert_eq!(under_tris.len(), 46);
    assert_eq!(under_tris, want);
}
