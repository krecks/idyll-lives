//! The baked shading where parts lie exactly on one another, against Blender 5.2's (tests/golden/shade_ties.txt:
//! each a joined mesh as Blender held it, with the levels gamekit's own `shade.bake_ao` baked on it; shade_rays.txt:
//! some of those bakes' rays that met two faces within a hundredth of a millimetre of each other, with the face
//! Blender's `BVHTree.ray_cast` gave each, how far and its normal). A ray that runs into two faces in one plane, an
//! edge or a corner is given a face by the order Blender's tree comes to its triangles in, so the kernel builds and
//! walks that tree (`bvh::Polygons`).

use crate::common::*;
use artkit::bvh::Polygons;
use artkit::math::V3;
use artkit::mesh::Mesh;
use artkit::{scene, shade};

fn case(name: &str) -> GoldenMesh {
    golden_meshes("shade_ties.txt")
        .into_iter()
        .find(|m| m.name == name)
        .unwrap_or_else(|| panic!("no mesh {name} in the golden"))
}

fn points(m: &GoldenMesh) -> Vec<V3> {
    m.verts.iter().map(|v| V3::new(v[0], v[1], v[2])).collect()
}

/// The mesh baked by the kernel against Blender's levels, vertex for vertex.
fn bakes_as_blender(name: &str) {
    let m = case(name);
    scene::reset();
    let mut mesh = Mesh::from_pydata(points(&m), &m.faces);
    // (a bevel's hardened normals, where Blender's mesh had them: a vertex's normal is the mean of its corners')
    let own = m.f32s("cn");
    if !own.is_empty() {
        mesh.normals = Some(own.chunks(3).map(|c| V3::new(c[0], c[1], c[2])).collect());
    }
    let obj = scene::add(name, mesh);
    let reach = m.lines.get("reach").map(|w| f64w(w));
    shade::bake_ao(&[obj], reach, m.lines["ground"] == "1");
    let got: Vec<i64> = obj
        .read(|o| o.mesh.ao.clone().unwrap())
        .iter()
        .map(|a| (a * 32.0).round() as i64)
        .collect();
    let want: Vec<i64> = m.lines["ao"]
        .split(' ')
        .map(|l| l.parse().unwrap())
        .collect();
    assert_eq!(got.len(), want.len());
    let off: Vec<(usize, i64, i64)> = (0..got.len())
        .filter(|&i| got[i] != want[i])
        .map(|i| (i, got[i], want[i]))
        .collect();
    let sum: i64 = off.iter().map(|o| (o.1 - o.2).abs()).sum();
    assert!(
        off.is_empty(),
        "{name}: {} of {} vertices off Blender's level, {sum} greys in all (vertex, got, Blender's): {:?}",
        off.len(),
        got.len(),
        &off[..off.len().min(12)]
    );
}

#[test]
fn key_strips_lying_on_a_bevelled_base() {
    bakes_as_blender("strip");
}

#[test]
fn boxes_standing_on_a_box() {
    bakes_as_blender("stack");
}

#[test]
fn a_band_round_a_pot_and_its_soil_on_its_floor() {
    bakes_as_blender("band");
}

#[test]
fn a_panel_in_its_frame() {
    bakes_as_blender("panel");
}

#[test]
fn a_bowl_standing_in_its_ring_cushion() {
    bakes_as_blender("ring");
}

#[test]
fn a_board_inlaid_in_a_table_top() {
    bakes_as_blender("inlay");
}

#[test]
fn a_slab_in_the_band_round_its_edge() {
    bakes_as_blender("moulding");
}

#[test]
fn rays_near_a_tie_meet_the_face_blenders_met() {
    let mut trees: std::collections::BTreeMap<String, Polygons> = Default::default();
    let (mut n, mut hits) = (0, 0);
    for line in golden("shade_rays.txt").lines() {
        let Some(line) = line.strip_prefix("ray ") else {
            continue;
        };
        let (ray, met) = line.split_once(" | ").unwrap();
        let w: Vec<&str> = ray.split(' ').collect();
        let tree = trees.entry(w[0].to_string()).or_insert_with(|| {
            let m = case(w[0]);
            Polygons::new(&points(&m), &m.faces)
        });
        let (o, d) = (f32s(&w[1..4]), f32s(&w[4..7]));
        let got = tree.ray_cast(
            V3::new(o[0], o[1], o[2]),
            V3::new(d[0], d[1], d[2]),
            f64w(w[7]),
        );
        n += 1;
        let m: Vec<&str> = met.split(' ').collect();
        let hit = got.unwrap_or_else(|| panic!("ray {n} ({}): met nothing", w[0]));
        assert_eq!(
            hit.index,
            m[0].parse::<u32>().unwrap(),
            "ray {n} ({}): the face",
            w[0]
        );
        assert_eq!(
            hit.dist.to_bits(),
            f32w(m[1]).to_bits(),
            "ray {n} ({}): how far",
            w[0]
        );
        assert_eq!(
            bits(&hit.normal.array()),
            bits(&f32s(&m[2..5])),
            "ray {n} ({}): the face's normal",
            w[0]
        );
        hits += 1;
    }
    assert!(hits >= 100, "{hits} rays");
}
