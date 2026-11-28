//! Easing hard edges to the gate's pixel: the eased profiles (`forms::eased`, `eased_loop`, `round_at`, `inset`) and
//! what the kernel does to a bevel so a part takes one cleanly (its UVs, a seam, a bevel it can't build).

use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::{eased, eased_loop, inset, round_at};
use artkit::materials::{Finish, Finishes, Mat};
use artkit::math::V3;
use artkit::mesh::Mesh;
use artkit::scene::{self, Obj};
use artkit::shapes::{Cube, cube};

fn wood() -> Mat {
    let finishes: Finishes = [(
        "wood".to_string(),
        Finish {
            roughness: 0.8,
            metallic: 0.0,
            textured: true,
        },
    )]
    .into_iter()
    .collect();
    scene::set_finishes(finishes);
    Mat::new("wood", "#8A5A2B", "wood")
}

/// The area of each of `mesh`'s faces in its UVs (its first triangle's, for a quad as flat in UV as in space).
fn uv_areas(mesh: &artkit::mesh::Mesh) -> Vec<f32> {
    let uvs = mesh.uvs.as_ref().expect("UVs");
    mesh.faces
        .iter()
        .map(|f| {
            let at = f.start as usize;
            let (a, b, c) = (uvs[at], uvs[at + 1], uvs[at + 2]);
            ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
        })
        .collect()
}

fn turns(p: &[(f64, f64)], closed: bool) -> Vec<f64> {
    let n = p.len();
    let corners = if closed { 0..n } else { 1..n - 1 };
    corners
        .map(|i| {
            let (a, b, c) = (p[(i + n - 1) % n], p[i], p[(i + 1) % n]);
            let (u, v) = ((b.0 - a.0, b.1 - a.1), (c.0 - b.0, c.1 - b.1));
            (u.0 * v.1 - u.1 * v.0)
                .atan2(u.0 * v.0 + u.1 * v.1)
                .to_degrees()
                .abs()
        })
        .collect()
}

#[test]
fn eased_rounds_only_sharp_corners() {
    // a cup: its foot (63°) and its square rim round in two steps each; the shallow bend stays a point
    let rim = eased(
        &[
            (0.0, 0.0),
            (0.05, 0.0),
            (0.06, 0.02),
            (0.06, 0.1),
            (0.055, 0.1),
        ],
        0.003,
    );
    assert!(rim.len() == 9 && rim.contains(&(0.06, 0.02)) && !rim.contains(&(0.05, 0.0)));
    assert!(
        rim.iter()
            .all(|&(u, v)| u <= 0.06 + 1e-6 && v <= 0.1 + 1e-6)
    );
    // no corner turns 60° or more: none is hard
    assert!(turns(&rim, false).iter().all(|&t| t <= 50.0 + 1e-9));
}

#[test]
fn eased_loop_rounds_every_square_corner() {
    let sq = [(0.0, 0.0), (0.0, 0.01), (0.02, 0.01), (0.02, 0.0)];
    let out = eased_loop(&sq, 0.003);
    assert_eq!(out.len(), 4 * 3); // each 90° corner in two steps, three points
    assert!(!out.contains(&(0.0, 0.0)));
    assert!(turns(&out, true).iter().all(|&t| t <= 50.0 + 1e-9));
}

#[test]
fn round_at_rolls_one_corner_on_its_radius() {
    let p = [(0.0, 0.0), (0.1, 0.0), (0.1, 0.05)];
    let out = round_at(&p, 1, 0.01);
    assert_eq!(out.len(), 2 + 5);
    // the arc's ends on the legs a radius from the corner, its points a radius from (0.09, 0.01)
    assert!((out[1].0 - 0.09).abs() < 1e-12 && out[1].1.abs() < 1e-12);
    assert!((out[5].0 - 0.1).abs() < 1e-12 && (out[5].1 - 0.01).abs() < 1e-12);
    for q in &out[1..6] {
        assert!(((q.0 - 0.09).hypot(q.1 - 0.01) - 0.01).abs() < 1e-12);
    }
}

#[test]
fn inset_moves_a_square_in() {
    let sq = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    let small = inset(&sq, 0.1);
    assert!((small[0].0 - 0.1).abs() < 1e-12 && (small[0].1 - 0.1).abs() < 1e-12);
    assert!((small[2].0 - 0.9).abs() < 1e-12 && (small[2].1 - 0.9).abs() < 1e-12);
    // wound the other way round, the same; a negative inset moves it out
    let rev: Vec<_> = sq.iter().rev().copied().collect();
    assert!((inset(&rev, 0.1)[0].0 - 0.1).abs() < 1e-12);
    assert!((inset(&sq, -0.1)[0].0 + 0.1).abs() < 1e-12);
}

#[test]
fn a_one_strip_chamfer_on_a_textured_box_takes_the_texture_as_a_face() {
    // a post chamfered in one strip: the strips' UVs are laid on them (a box a face), none without area (smeared:
    // carried from the faces they cut, as Blender carries them, each strip had none), its faces' as before
    scene::reset();
    let post = cube(
        "post",
        (0.06, 0.06, 0.8),
        Cube {
            material: Some(wood()),
            bevel: 0.004,
            bevel_segments: 1,
            ..Cube::default()
        },
    );
    let joined = artkit::parts::join(&[post], "j", None);
    let mesh = joined.read(|o| o.mesh.clone());
    assert_eq!(mesh.faces.len(), 6 + 12 + 8);
    assert!(
        uv_areas(&mesh).iter().all(|&a| a > 1e-7),
        "{:?}",
        uv_areas(&mesh)
    );
    // a side's UVs are its corners' own coordinates across it, the grain up the post
    let uvs = mesh.uvs.as_ref().unwrap();
    let side = (0..mesh.faces.len())
        .find(|&f| mesh.face_normal(f).x > 0.999)
        .unwrap();
    let face = mesh.faces[side];
    for (i, &v) in mesh.corners(side).iter().enumerate() {
        let p = mesh.verts[v as usize];
        assert_eq!(uvs[face.start as usize + i], [p.y, p.z]);
    }
}

#[test]
fn a_ring_turned_from_a_closed_profile_is_welded_so_it_takes_a_bevel() {
    // a ring's square section, closed on its first point: one point a corner each way round, and a chamfer all round
    scene::reset();
    let section = [
        (0.1, 0.0),
        (0.12, 0.0),
        (0.12, 0.02),
        (0.1, 0.02),
        (0.1, 0.0),
    ];
    let ring = artkit::shapes::revolve(
        "ring",
        &section,
        artkit::shapes::Revolve {
            segments: Some(16),
            ..Default::default()
        },
    );
    assert_eq!(ring.read(|o| o.mesh.verts.len()), 4 * 16);
    bevel_edges(ring, 0.002, 1, artkit::shapes::SHARP, &EdgeOrder::Corners);
    artkit::bevel::apply(ring);
    assert_eq!(ring.read(|o| o.mesh.faces.len()), 16 * (4 + 4));
}

#[test]
fn a_bevel_that_would_end_at_a_lone_corner_is_left_off_those_edges() {
    // a wedge (a right triangle 90°, 60°, 30° drawn 0.3 long): its two sharp long edges turn over 100°, its ends'
    // edges 90°, so each long edge's bevel would end where no other goes. Refused by `bevelled`, left off by a part's
    scene::reset();
    let tri = [(0.0, 0.0), (0.1, 0.0), (0.0, 0.1 * 3f64.sqrt())];
    let mut verts = Vec::new();
    for z in [0.0, 0.3] {
        verts.extend(
            tri.iter()
                .map(|&(x, y)| V3::new(x as f32, y as f32, z as f32)),
        );
    }
    let faces = vec![
        vec![0, 2, 1],
        vec![3, 4, 5],
        vec![0, 1, 4, 3],
        vec![1, 2, 5, 4],
        vec![2, 0, 3, 5],
    ];
    let mut mesh = Mesh::from_pydata(verts, &faces);
    mesh.recalc_normals();
    assert!(artkit::bevel::bevelled(&mesh, 0.004, 1, 100.0, &EdgeOrder::Corners).is_err());
    let wedge = scene::add("wedge", mesh);
    bevel_edges(wedge, 0.004, 1, 100.0, &EdgeOrder::Corners);
    artkit::bevel::apply(wedge);
    assert_eq!(
        wedge.read(|o| (o.mesh.verts.len(), o.mesh.faces.len())),
        (6, 5)
    );
}

#[test]
fn a_boulder_is_scaled_in_its_mesh() {
    // so its UVs, made on its mesh, lay the texture at its own size
    scene::reset();
    let flat = artkit::shapes::boulder(
        "moss",
        0.1,
        artkit::shapes::Boulder {
            scale: artkit::math::p3(1.3, 1.0, 0.25),
            ..Default::default()
        },
    );
    let round = artkit::shapes::boulder("moss", 0.1, Default::default());
    let height = |o: Obj| {
        let zs: Vec<f32> = o.world_points().iter().map(|p| p.z).collect();
        zs.iter().copied().fold(f32::MIN, f32::max) - zs.iter().copied().fold(f32::MAX, f32::min)
    };
    assert_eq!(flat.scale(), V3::new(1.0, 1.0, 1.0));
    assert!((height(flat) - height(round) * 0.25).abs() < 1e-6);
}

#[test]
fn botanys_pots_beds_and_blooms_come_eased_and_laid_flat() {
    // a pot's and its saucer's rims chamfered, a raised bed's boards chamfered (as its posts were), a cupped bloom
    // laid a box a face
    scene::reset();
    let p = artkit::materials::Palette::new(&[
        Mat::new("body", "#C0683F", "satin"),
        Mat::new("soil", "#5A4030", "matte"),
    ]);
    artkit::botany::vessel(
        &p,
        0.1,
        0.2,
        artkit::botany::Vessel {
            saucer: 0.02,
            ..Default::default()
        },
    );
    artkit::botany::pot(&p, 0.1, 0.2, Default::default());
    artkit::botany::raised_bed(&p, 0.9, 0.9, 0.2);
    let bloom = artkit::botany::cupped(
        "bloom",
        &[(0.0, 0.0), (0.02, 0.01), (0.03, 0.04)],
        (0.0, 0.0, 0.5),
        Mat::new("petal", "#F0E0E0", "satin"),
        artkit::botany::Cupped {
            tilt: 30.0,
            ..Default::default()
        },
    );
    for part in scene::meshes() {
        let name = part.name();
        let eased = part.read(|o| o.bevel.is_some());
        let shown = ["pot", "saucer", "board", "post"]
            .iter()
            .any(|n| name.starts_with(n));
        assert_eq!(eased, shown, "{name}");
    }
    assert_eq!(bloom.read(|o| o.uv), artkit::mesh::UvWay::Box);
}
